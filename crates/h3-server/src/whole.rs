//! The whole games (E16 M5 T9a): the map offers, compiled at start, and a whole game's thread,
//! which hosts its `h3_protocol::session::game::GameSession` for connections: every player of the
//! map a seat, taken by account or by token as a battle's; the game starts when every seat is held.
//! Each message's entries are durable in the game's journal (`journal::whole`) before its frames
//! are sent, as a battle's.
//!
//! Not yet (E16 M5 T9b, `SRV-STUB-12`): a whole game's journal is not restored at start (it is
//! kept as it is), a running game whose seats stayed empty does not pause, a paused one keeps its
//! thread, and no record is downloaded.
//!
//! Provisional (the 2a review list): every seat's player human, a map's name from its header as
//! Latin-1 within 64 characters, a draining server's running whole games playing on.

use std::collections::BTreeMap;
use std::fmt;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use h3_contracts::PlayerId;
use h3_game::{GameRuleset, GameSetup, GameState};
use h3_protocol::map::game::{manifest, map_offer};
use h3_protocol::session::game::{GameOptions, GameSession};
use h3_protocol::session::{ConnId, Outbound};
use h3_protocol::wire::game::{MapIdentity, SeatedPlayer};
use h3_protocol::wire::{GoodbyeReason, Phase, WireU64};
use h3_sim::adventure::replay::{MapRef, Replay, ReplayHeader};
use tokio::sync::mpsc;

use crate::accounts::Accounts;
use crate::game::{
    Frame, Hosting, Outbox, Stores, ToGame, close_all, os_u64, shut_down, stop, token, turn_away,
};
use crate::journal::whole::{self as journal, GAME_JOURNAL_FORMAT_VERSION, GameStart};
use crate::registry::{Shared, lock};
use crate::{Error, config};

/// The most characters of a map's name the lobby shows.
const NAME_CHARS: usize = 64;

/// A map a client may create a whole game from, compiled at start.
#[derive(Debug)]
pub struct MapOffer {
    /// The offer's id.
    pub id: String,
    /// The rules its games share.
    pub rules: GameRuleset,
    /// The game its map sets up, at the offer's difficulty, every player a seat.
    pub setup: GameSetup,
    /// The map's file, as a replay names it.
    pub map: MapRef,
    /// What the lobby shows of it.
    pub info: h3_protocol::wire::game::MapOffer,
}

impl MapOffer {
    /// Loads the offer's map on the packs in `packs`: its file's content hash must be the
    /// configured one, its game must start (at the offer's difficulty).
    pub fn load(config: &config::MapOffer, packs: &Path) -> Result<Self, Error> {
        let at = |e: &dyn fmt::Display| {
            Error(format!(
                "map `{}`: {}: {e}",
                config.id,
                config.file.display()
            ))
        };
        let rules = h3_files::adventure::load_rules(packs, &config.packs, &config.profile)
            .map_err(|e| at(&e))?;
        // `read_map` takes a path below a root: the file's name below its directory.
        let root = config.file.parent().unwrap_or_else(|| Path::new("."));
        let name = (config.file.file_name().and_then(|name| name.to_str()))
            .ok_or_else(|| at(&"not a file's name"))?;
        let (bytes, map) = h3_files::adventure::read_map(root, name).map_err(|e| at(&e))?;
        let pinned = (config.hash.strip_prefix("0x"))
            .filter(|hex| hex.len() == 16)
            .and_then(|hex| u64::from_str_radix(hex, 16).ok())
            .ok_or_else(|| at(&"its hash is not `0x` and 16 hex digits"))?;
        if pinned != map.content_hash {
            return Err(at(&format_args!(
                "its content hash is {:#018x}, not the configured {}",
                map.content_hash, config.hash
            )));
        }
        let raw = h3_map::read(&bytes, &h3_map::Limits::default()).map_err(|e| at(&e))?;
        let mut setup = h3_map::import(&raw, &rules).map_err(|e| at(&e))?;
        setup.difficulty = config.difficulty;
        // Every seat's player is a connection's (E16 M5 2a: no computer player before E13).
        for player in &mut setup.players {
            player.human = true;
        }
        GameState::new(&rules, &setup, 0).map_err(|e| at(&format_args!("{e:?}")))?;
        let name = (config.name.clone()).unwrap_or_else(|| latin1(raw.header.name.as_bytes()));
        let identity = MapIdentity {
            content_hash: WireU64(map.content_hash),
            import_version: map.import_version,
        };
        let info = map_offer(&config.id, name, (&rules, &setup), identity);
        Ok(Self {
            id: config.id.clone(),
            rules,
            setup,
            map,
            info,
        })
    }
}

/// A map file's name as the lobby shows it: its bytes as Latin-1, control characters left out,
/// at most [`NAME_CHARS`].
fn latin1(bytes: &[u8]) -> String {
    let chars = bytes.iter().map(|&b| char::from(b));
    chars.filter(|c| !c.is_control()).take(NAME_CHARS).collect()
}

/// A whole game ready to run: its offer, its secrets and how it is hosted.
#[derive(Debug)]
pub(crate) struct Prepared {
    offer: Arc<MapOffer>,
    game: u32,
    /// Where its creation came from.
    origin: IpAddr,
    /// The game's seed.
    seed: u64,
    /// A token per player's seat.
    tokens: Vec<String>,
    options: GameOptions,
    /// The data directory.
    data: PathBuf,
    /// The server's books.
    stores: Stores,
}

impl Prepared {
    /// Draws game `game`'s secrets from the OS (ADR-0018 point 8): its seed and a token per
    /// player's seat; the game's id is the game's for the server's life before anything names it.
    pub(crate) fn new(
        offer: Arc<MapOffer>,
        game: u32,
        origin: IpAddr,
        (server, limits, stores): (&config::Server, &config::Limits, Stores),
    ) -> Result<Self, Error> {
        let seed = os_u64()?;
        let tokens = (offer.setup.players.iter().map(|_| token())).collect::<Result<_, _>>()?;
        (stores.ids.reach(game)).map_err(|e| Error(format!("game {game}'s id: {e}")))?;
        let options = GameOptions {
            game,
            checkpoint_every: server.checkpoint_every,
            record_bytes: limits.record_bytes,
        };
        Ok(Self {
            offer,
            game,
            origin,
            seed,
            tokens,
            options,
            data: server.data.clone(),
            stores,
        })
    }

    /// The journal's first line for the game as `session` began it.
    fn start(&self, session: &GameSession<'_>) -> Result<GameStart, String> {
        let offer = &self.offer;
        let replay = Replay {
            header: ReplayHeader::current(&offer.rules),
            map: offer.map.clone(),
            seed: self.seed,
            difficulty: offer.setup.difficulty,
            commands: Vec::new(),
            checkpoint_every: self.options.checkpoint_every,
            checkpoints: Vec::new(),
            final_day: session.state().day(),
            final_hash: session.state().state_hash(),
        };
        let replay = h3_files::adventure::replay_to_toml(&replay).map_err(|e| e.to_string())?;
        Ok(GameStart {
            format: GAME_JOURNAL_FORMAT_VERSION,
            game: self.game,
            offer: offer.id.clone(),
            origin: self.origin,
            tokens: self.tokens.clone(),
            replay,
        })
    }
}

impl Hosting for Prepared {
    /// The game's loop: one message at a time through the session, each call's entries made
    /// durable before its frames are sent (a journal that does not write stops the game: its
    /// players are told the server shuts down, its journal stays). After each, the lobby hears
    /// where the game is and who holds its seats. A game that never started and whose seats are
    /// all empty again is abandoned: it leaves the lobby and its journal goes.
    fn host(self, mut inbox: mpsc::Receiver<ToGame>, registry: &Shared) {
        let (game, offer) = (self.game, Arc::clone(&self.offer));
        let manifest = manifest(&offer.rules, offer.info.map);
        let token = |player: PlayerId| self.tokens[player.index()].clone();
        let session = GameSession::new(
            &offer.rules,
            &offer.setup,
            self.seed,
            manifest,
            self.options,
            token,
        );
        // `MapOffer::load` started this game already.
        let Ok(mut session) = session else {
            lock(registry).leave(game);
            return turn_away(&mut inbox, GoodbyeReason::Shutdown);
        };
        let journal = self
            .start(&session)
            .and_then(|start| journal::create(&self.data, &start).map_err(|e| e.to_string()));
        let mut journal = match journal {
            Ok(journal) => journal,
            Err(e) => {
                eprintln!("h3-server: game {game} not created: its journal: {e}");
                lock(registry).leave(game);
                return turn_away(&mut inbox, GoodbyeReason::Shutdown);
            }
        };
        let mut conns: BTreeMap<ConnId, Outbox> = BTreeMap::new();
        // How many messages each connection passed the game (E16 M2 T5b).
        let mut passed: BTreeMap<ConnId, u64> = BTreeMap::new();
        let mut out = Vec::new();
        let status = |s: &GameSession<'_>| (s.phase(), s.open_seats());
        let mut reported = status(&session);
        let mut named: Vec<(u8, String)> = Vec::new();
        while let Some(event) = inbox.blocking_recv() {
            let mut left = None;
            match event {
                // Nothing changes for the session until the connection's join.
                ToGame::Open { conn, to, login } => {
                    conns.insert(conn, to);
                    if let Some(login) = login {
                        session.logged_in(conn, login);
                    }
                    continue;
                }
                ToGame::Message { conn, message } => {
                    let k = passed.entry(conn).or_default();
                    *k += 1;
                    session.handle(conn, message, &mut out);
                    left = (!session.holds(conn)).then_some((conn, *k));
                }
                ToGame::Closed { conn } => {
                    conns.remove(&conn);
                    passed.remove(&conn);
                    session.disconnected(conn);
                }
                // No sweep pauses a whole game whose seats stayed empty, and a paused one keeps its
                // thread, until E16 M5 T9b with the restore at start (stub: SRV-STUB-12).
                ToGame::Tick => continue,
                ToGame::Drain if session.started() => continue,
                ToGame::Drain | ToGame::Stop => {
                    let started = session.started();
                    return shut_down(game, (&conns, started, Some(journal)), registry, &mut inbox);
                }
            }
            if let Err(e) = journal.append_game(session.drain_entries()) {
                eprintln!("h3-server: game {game}: the journal does not write ({e}): it stops");
                return stop(&conns, registry, game);
            }
            let closed: Vec<ConnId> = out.iter().filter(|o| o.close).map(|o| o.to).collect();
            let last =
                left.and_then(|(conn, k)| Some((out.iter().rposition(|o| o.to == conn)?, k)));
            for (i, outbound) in out.drain(..).enumerate() {
                let left = last.filter(|&(at, _)| at == i).map(|(_, k)| k);
                deliver(&mut conns, &mut session, outbound, left);
            }
            if session.phase() == Phase::Waiting && !session.seated() {
                // A joiner whose join is still on its way hears it; a connection whose join it
                // refused leaves at its refusal and never reads the goodbye.
                close_all(&conns, &closed, GoodbyeReason::Abandoned);
                lock(registry).leave(game);
                // Outside the registry's lock (E16 M2 T5e): a removal is file I/O.
                journal.remove();
                turn_away(&mut inbox, GoodbyeReason::Abandoned);
                return eprintln!("h3-server: game {game} abandoned before it started");
            }
            let now = status(&session);
            let accounts: Vec<(u8, String)> = (session.accounts().into_iter())
                .map(|(player, login)| (player, login.to_owned()))
                .collect();
            // Outside the registry's lock: the accounts' lock waits on the player log's writes.
            let renamed = (accounts != named).then(|| seated(&accounts, &self.stores.accounts));
            let signal = {
                let mut registry = lock(registry);
                if now != reported {
                    registry.report_map(game, now.clone());
                    reported = now;
                }
                if let Some(seated) = renamed {
                    registry.seat_map(game, seated);
                    named = accounts;
                }
                registry.signal()
            };
            // A drain or a stop the full inbox missed (M2 T5e's review).
            let started = session.started();
            if matches!(signal, Some(ToGame::Stop)) || (signal.is_some() && !started) {
                return shut_down(game, (&conns, started, Some(journal)), registry, &mut inbox);
            }
        }
        lock(registry).leave(game);
    }
}

/// The seats of a whole game that belong to accounts (`accounts`: each player with its login
/// name), with the accounts' display names as they stand. Blocks on the player log: never under
/// the registry's lock.
fn seated(accounts: &[(u8, String)], books: &Accounts) -> Vec<SeatedPlayer> {
    let named = accounts.iter().filter_map(|(player, login)| {
        Some(SeatedPlayer {
            player: *player,
            display: books.get(login)?.display,
        })
    });
    named.collect()
}

/// Queues a frame for its connection, with its mark `left` ([`Frame::left`]), as a battle's
/// (`game::deliver`): a full queue drops the connection, and the session forgets it.
fn deliver(
    conns: &mut BTreeMap<ConnId, Outbox>,
    session: &mut GameSession<'_>,
    outbound: Outbound,
    left: Option<u64>,
) {
    let Some(to) = conns.get(&outbound.to) else {
        return;
    };
    let frame = Frame {
        text: outbound.frame,
        close: outbound.close,
        left,
        held: session.holds(outbound.to),
    };
    if !to.offer(frame) {
        conns.remove(&outbound.to);
        session.disconnected(outbound.to);
    }
}
