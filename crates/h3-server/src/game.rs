//! The hosted games (ADR-0018 points 3, 6, 8 and 12; E16 M2 T1, T2): the offers, compiled once at
//! start; a game prepared from an offer with secrets from the OS, or restored from its journal
//! after a restart, and run on its own OS thread. The thread owns the session core and the game's
//! journal ([`crate::journal`]): it makes each call's entries durable before it sends that call's
//! frames. The connection tasks talk to it over bounded channels, and it tells the lobby where it
//! is through the registry ([`crate::registry`]).

use std::collections::BTreeMap;
use std::fs;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

use h3_battle::{BattleSide, BattleState, LegalCommands};
use h3_content::Ruleset;
use h3_contracts::BattleSetup;
use h3_kernel::{Rng, StreamId};
use h3_protocol::session::{Bot, ConnId, GameConfig, Journal, Outbound, SeatKind, Session};
use h3_protocol::wire::{self, ClientMessage, GameInfo, GoodbyeReason, Phase, Seat, ServerMessage};
use h3_protocol::{codec, map};
use h3_sim::{Replay, ReplayHeader};
use tokio::sync::{mpsc, oneshot};

use crate::Error;
use crate::config::{self, BotSeat};
use crate::journal::{self, JOURNAL_FORMAT_VERSION, Start, Writer};
use crate::registry::{Shared, lock};

/// A frame for a connection task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Frame {
    /// The message's text.
    pub text: String,
    /// Whether the connection closes after it.
    pub close: bool,
}

/// What a connection task tells a game.
#[derive(Debug)]
pub(crate) enum ToGame {
    /// A connection joins the game; its frames go to `to`.
    Open {
        /// The connection.
        conn: ConnId,
        /// Its outbound queue for this game.
        to: mpsc::Sender<Frame>,
    },
    /// A message the connection's hello gate let through.
    Message {
        /// The connection.
        conn: ConnId,
        /// The message.
        message: ClientMessage,
    },
    /// The connection closed, or left for another game.
    Closed {
        /// The connection.
        conn: ConnId,
    },
}

/// 64 bits of OS randomness (ADR-0018 point 8).
pub(crate) fn os_u64() -> Result<u64, Error> {
    getrandom::u64().map_err(|e| Error(format!("OS randomness: {e}")))
}

/// A seat token: 128 bits of OS randomness as 32 hex digits (ADR-0018 point 6).
pub(crate) fn token() -> Result<String, Error> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|e| Error(format!("OS randomness: {e}")))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

/// The sim's random agent as the server's bot (`h3_sim::rollout::choose`).
#[derive(Debug)]
pub(crate) struct RandomBot(pub Rng);

impl Bot for RandomBot {
    fn command(
        &mut self,
        state: &BattleState,
        ruleset: &Ruleset,
        side: BattleSide,
        legal: &LegalCommands,
    ) -> h3_battle::Command {
        h3_sim::rollout::choose(state, ruleset, side, legal, &mut self.0)
    }
}

/// A game configuration the server offers, compiled at start.
#[derive(Debug)]
pub struct Offer {
    /// The offer's id.
    pub id: String,
    /// The scenario's id (the replay file names it).
    pub scenario: String,
    /// The ruleset its games share.
    pub ruleset: Arc<Ruleset>,
    /// The battle, its obstacles not yet placed.
    pub setup: BattleSetup,
    /// Whether each game draws its own obstacles.
    pub random_obstacles: bool,
    /// The seats the server's bot plays.
    pub bots: Vec<BotSeat>,
    /// What the lobby shows of it.
    pub info: wire::Offer,
}

impl Offer {
    /// Loads the offer's scenario on the packs in `packs` and checks that its battle starts.
    pub fn load(config: &config::Offer, packs: &Path) -> Result<Self, Error> {
        let at = |e: &dyn std::fmt::Display| {
            Error(format!(
                "offer `{}`: {}: {e}",
                config.id,
                config.scenario.display()
            ))
        };
        let text = fs::read_to_string(&config.scenario).map_err(|e| at(&e))?;
        let def = h3_pack::parse_scenario(&text).map_err(|e| at(&e))?;
        let profile = config.profile.as_deref().unwrap_or(&def.profile);
        let ruleset = h3_cli::load_ruleset(packs, &def.packs, profile).map_err(|e| at(&e))?;
        let setup = def.resolve(&ruleset).map_err(|e| at(&e))?;
        BattleState::new(&ruleset, &setup, 0).map_err(|e| at(&e))?;
        let seats = config.bots.iter().map(|bot| match bot {
            BotSeat::Attacker => Seat::Attacker,
            BotSeat::Defender => Seat::Defender,
        });
        let info = map::offer(&config.id, &def.id, &ruleset, seats.collect());
        Ok(Self {
            id: config.id.clone(),
            scenario: def.id,
            random_obstacles: def.random_obstacles && setup.terrain.is_some(),
            ruleset: Arc::new(ruleset),
            setup,
            bots: config.bots.clone(),
            info,
        })
    }
}

/// A game ready to run: its offer, its battle and its secrets, and how its thread begins.
#[derive(Debug)]
pub(crate) struct Prepared {
    /// What it is made of.
    offer: Arc<Offer>,
    /// The scenario's id, as the replay names it.
    scenario: String,
    /// The game's id, its battle (obstacles placed), seed and checkpoint interval.
    config: GameConfig,
    /// The seats: a token per player, `None` for the bot's.
    tokens: [Option<String>; 2],
    /// The seed of the bot's choices.
    bot_seed: u64,
    /// Where the replay goes at the end.
    replay_path: PathBuf,
    /// The data directory.
    data: PathBuf,
    /// A new game or a restored one.
    begin: Begin,
}

/// How a game's thread begins (E16 M2 T2).
#[derive(Debug)]
enum Begin {
    /// A new game, its journal created and the game listed.
    New(Writer),
    /// A game restored from its journal.
    Restored(Restored),
}

/// What a restored game's thread needs to re-run its journal and list the game again.
#[derive(Debug)]
struct Restored {
    /// The journal's file.
    path: PathBuf,
    /// What it holds.
    read: journal::Read,
    /// The game's inbox, for the registry once the game is listed again.
    to: mpsc::Sender<ToGame>,
    /// Dropped when the re-run is done, however it went: the server listens after every one.
    done: oneshot::Sender<()>,
}

/// Where game `game`'s replay goes in the data directory `data`: a random part keeps the name of a
/// game id that a restart hands out again from another game's file.
fn replay_path(data: &Path, offer: &str, game: u32) -> Result<PathBuf, Error> {
    let random = os_u64()?;
    Ok(data
        .join("replays")
        .join(format!("{offer}-game{game}-{random:016x}.toml")))
}

impl Prepared {
    /// Draws game `game`'s secrets from the OS (ADR-0018 point 8): the battle's seed; the
    /// obstacles from a separate seed that is then forgotten, so that the layout every viewer sees
    /// tells nothing about the battle's seed; a token per player seat; the bot's seed. Checks that
    /// the battle starts, then creates the game's journal (E16 M2 T2) for its creator at `origin`.
    pub(crate) fn new(
        offer: Arc<Offer>,
        game: u32,
        origin: IpAddr,
        server: &config::Server,
    ) -> Result<Self, Error> {
        let mut setup = offer.setup.clone();
        if let (true, Some(terrain)) = (offer.random_obstacles, setup.terrain) {
            let mut rng = Rng::for_stream(os_u64()?, StreamId::OBSTACLES);
            setup.obstacles = h3_battle::generate_obstacles(&offer.ruleset, terrain, &mut rng);
        }
        let seed = os_u64()?;
        BattleState::new(&offer.ruleset, &setup, seed)
            .map_err(|e| Error(format!("offer `{}`, game {game}: {e}", offer.id)))?;
        let mut tokens = [None, None];
        for (side, slot) in [BotSeat::Attacker, BotSeat::Defender]
            .into_iter()
            .zip(&mut tokens)
        {
            if !offer.bots.contains(&side) {
                *slot = Some(token()?);
            }
        }
        let config = GameConfig {
            game,
            setup,
            seed,
            checkpoint_every: server.checkpoint_every,
        };
        let (bot_seed, replay_path) = (os_u64()?, replay_path(&server.data, &offer.id, game)?);
        let begun = Journal {
            seed,
            setup: config.setup.clone(),
            commands: Vec::new(),
            checkpoint_every: config.checkpoint_every,
            checkpoints: Vec::new(),
        };
        let start = Start {
            format: JOURNAL_FORMAT_VERSION,
            game,
            offer: offer.id.clone(),
            origin,
            tokens: tokens.clone(),
            replay: replay_text(&offer.ruleset, &offer.scenario, &begun, (0, None))?,
        };
        let journal = Writer::create(&server.data, &start)
            .map_err(|e| Error(format!("game {game}'s journal: {e}")))?;
        Ok(Self {
            scenario: offer.scenario.clone(),
            offer,
            config,
            tokens,
            bot_seed,
            replay_path,
            data: server.data.clone(),
            begin: Begin::New(journal),
        })
    }

    /// The game a journal holds, to restore (E16 M2 T2): the offer `offer` its first line names,
    /// read from `path`. Why the journal is void otherwise: its replay header does not match the
    /// offer's engine and content (`ReplayHeader::check`), or its replay does not read on them.
    /// The thread drops `done` when its re-run is done, and lists the game with the inbox `to` if
    /// it goes on.
    pub(crate) fn restored(
        offer: Arc<Offer>,
        path: &Path,
        read: journal::Read,
        server: &config::Server,
        (to, done): (mpsc::Sender<ToGame>, oneshot::Sender<()>),
    ) -> Result<Self, String> {
        let doc = h3_cli::ReplayDoc::parse(&read.start.replay).map_err(|e| e.to_string())?;
        let scenario = doc.scenario().to_owned();
        let replay = doc.into_replay(&offer.ruleset).map_err(|e| e.to_string())?;
        replay
            .header
            .check(&offer.ruleset)
            .map_err(|e| e.to_string())?;
        let game = read.start.game;
        let config = GameConfig {
            game,
            setup: replay.setup,
            seed: replay.seed,
            checkpoint_every: replay.checkpoint_every,
        };
        let secrets =
            os_u64().and_then(|seed| Ok((seed, replay_path(&server.data, &offer.id, game)?)));
        let (bot_seed, replay_path) = secrets.map_err(|e| e.0)?;
        Ok(Self {
            scenario,
            offer,
            config,
            tokens: read.start.tokens.clone(),
            bot_seed,
            replay_path,
            data: server.data.clone(),
            begin: Begin::Restored(Restored {
                path: path.to_path_buf(),
                read,
                to,
                done,
            }),
        })
    }

    /// Runs the game on its own OS thread until it leaves the lobby.
    pub(crate) fn spawn(
        self,
        inbox: mpsc::Receiver<ToGame>,
        registry: Shared,
    ) -> Result<(), Error> {
        let game = self.config.game;
        thread::Builder::new()
            .name(format!("game-{game}"))
            .spawn(move || self.run(inbox, &registry))
            .map(|_| ())
            .map_err(|e| Error(format!("game thread: {e}")))
    }

    /// The game's loop: one message at a time through the session core, each call's entries made
    /// durable before its frames are sent (a journal that does not write stops the game: its
    /// players are told the server shuts down, the next start restores it). After each, the lobby
    /// hears where the game is; a game that is not running and that no connection is bound to
    /// leaves the lobby and ends (E16 M2 T1); one that never started leaves no journal.
    fn run(self, mut inbox: mpsc::Receiver<ToGame>, registry: &Shared) {
        let Self {
            offer,
            scenario,
            config,
            tokens,
            bot_seed,
            replay_path,
            data,
            begin,
        } = self;
        let game = config.game;
        let mut bots: u64 = 0;
        let seats = tokens.map(|token| match token {
            Some(token) => SeatKind::Player { token },
            None => {
                bots += 1;
                SeatKind::Bot(Box::new(RandomBot(Rng::for_stream(
                    bot_seed.wrapping_add(bots),
                    StreamId::AGENT,
                ))))
            }
        });
        let over = |session: &Session<'_>, journal: Writer| {
            finish(
                (&offer.ruleset, &scenario, &replay_path),
                game,
                session,
                journal,
            )
        };
        let (mut session, journal) = match begin {
            Begin::New(journal) => match Session::new(&offer.ruleset, config, seats) {
                Ok(session) => (session, journal),
                // `Prepared::new` started this battle already.
                Err(_) => {
                    journal.remove();
                    return lock(registry).leave(game);
                }
            },
            Begin::Restored(restored) => {
                let Some(resumed) = resume(&offer, config, seats, restored, &data, registry, over)
                else {
                    return;
                };
                resumed
            }
        };
        let mut journal = Some(journal);
        let mut conns = BTreeMap::new();
        let mut out = Vec::new();
        let mut reported = (session.phase(), session.open_seats());
        while let Some(event) = inbox.blocking_recv() {
            match event {
                ToGame::Open { conn, to } => {
                    conns.insert(conn, to);
                }
                ToGame::Message { conn, message } => session.handle(conn, message, &mut out),
                ToGame::Closed { conn } => {
                    conns.remove(&conn);
                    session.disconnected(conn);
                }
            }
            let entries = session.drain_entries();
            let written = journal.as_mut().map_or(Ok(()), |j| j.append(entries));
            if let Err(e) = written {
                eprintln!("h3-server: game {game}: the journal does not write ({e}): it stops");
                return stop(&conns, registry, game);
            }
            for outbound in out.drain(..) {
                deliver(&mut conns, &mut session, outbound);
            }
            if session.result().is_some()
                && let Some(journal) = journal.take()
            {
                let text = over(&session, journal);
                session.publish_replay(text);
            }
            let now = (session.phase(), session.open_seats());
            let mut registry = lock(registry);
            if now != reported {
                registry.report(game, now.0, now.1.clone());
                reported = now;
            }
            if registry.close_if_idle(game, reported.0) {
                if let (Phase::Waiting, Some(journal)) = (reported.0, journal) {
                    journal.remove();
                }
                return;
            }
        }
        lock(registry).leave(game);
    }
}

/// Re-runs a restored game's journal (E16 M2 T2) and appends what it lacks. A journal that does
/// not re-run is void; a game that never started leaves, its journal with it; one that is over
/// leaves once `over` wrote its replay; a running one is listed again, its seats empty.
fn resume<'r>(
    offer: &'r Offer,
    config: GameConfig,
    seats: [SeatKind; 2],
    restored: Restored,
    data: &Path,
    registry: &Shared,
    over: impl FnOnce(&Session<'r>, Writer) -> String,
) -> Option<(Session<'r>, Writer)> {
    let Restored {
        path,
        read,
        to,
        done,
    } = restored;
    let game = config.game;
    let mut session = match Session::restore(&offer.ruleset, config, seats, &read.entries) {
        Ok(session) => session,
        Err(e) => {
            journal::void(data, &path, &game.to_string(), &format!("{e:?}"));
            return None;
        }
    };
    let reopened = Writer::reopen(&path, read.complete);
    let journal = match reopened.and_then(|mut j| j.append(session.drain_entries()).map(|()| j)) {
        Ok(journal) => journal,
        Err(e) => {
            eprintln!("h3-server: game {game}: the journal does not write ({e}): not restored");
            return None;
        }
    };
    if session.result().is_some() {
        over(&session, journal);
        return None;
    }
    if !session.started() {
        journal.remove();
        return None;
    }
    let info = GameInfo {
        game,
        offer: offer.id.clone(),
        phase: session.phase(),
        open: session.open_seats(),
    };
    lock(registry).reopen(read.start.origin, info, to);
    eprintln!("h3-server: game {game} restored");
    drop(done);
    Some((session, journal))
}

/// The journal stopped: every connection hears the server shuts down (for this game), and the game
/// leaves the lobby; its journal stays for the next start.
fn stop(conns: &BTreeMap<ConnId, mpsc::Sender<Frame>>, registry: &Shared, game: u32) {
    let text = codec::encode_text(&ServerMessage::Goodbye {
        reason: GoodbyeReason::Shutdown,
    });
    for to in conns.values() {
        let _ = to.try_send(Frame {
            text: text.clone(),
            close: true,
        });
    }
    lock(registry).leave(game);
}

/// Queues a frame for its connection. A full queue drops the connection: the game forgets it,
/// and its task, finding the queue closed, says `TooSlow` (ADR-0018 point 3).
pub(crate) fn deliver(
    conns: &mut BTreeMap<ConnId, mpsc::Sender<Frame>>,
    session: &mut Session<'_>,
    outbound: Outbound,
) {
    let Some(to) = conns.get(&outbound.to) else {
        return;
    };
    let frame = Frame {
        text: outbound.frame,
        close: outbound.close,
    };
    if let Err(mpsc::error::TrySendError::Full(_)) = to.try_send(frame) {
        conns.remove(&outbound.to);
        session.disconnected(outbound.to);
    }
}

/// The replay `journal` holds as the CLI's file (ADR-0018 point 12), with the final hash and the
/// result `end`.
fn replay_text(
    ruleset: &Ruleset,
    scenario: &str,
    journal: &Journal,
    end: (u64, Option<h3_battle::BattleResult>),
) -> Result<String, Error> {
    let journal = journal.clone();
    let replay = Replay {
        header: ReplayHeader::current(ruleset),
        seed: journal.seed,
        setup: journal.setup,
        commands: journal.commands,
        checkpoint_every: journal.checkpoint_every,
        checkpoints: journal.checkpoints,
        final_hash: end.0,
        result: end.1,
    };
    h3_cli::replay_to_toml(&replay, scenario, ruleset)
        .map_err(|e| Error(format!("the replay does not render: {e}")))
}

/// Game `game` is over: its replay is written as the CLI's file, then its journal goes (the replay
/// is the finished journal). A replay that does not write keeps the journal, which the next start
/// re-runs to write it again. The replay's text, for the participants.
fn finish(
    (ruleset, scenario, path): (&Ruleset, &str, &Path),
    game: u32,
    session: &Session<'_>,
    journal: Writer,
) -> String {
    let end = (session.state_hash(), session.result());
    let text = replay_text(ruleset, scenario, session.journal(), end).unwrap_or_else(|e| {
        eprintln!("h3-server: game {game}: {e}");
        String::new()
    });
    let written = path
        .parent()
        .map_or(Ok(()), fs::create_dir_all)
        .and_then(|()| fs::write(path, &text));
    match written {
        Ok(()) => {
            eprintln!("h3-server: game {game} over, replay {}", path.display());
            journal.remove();
        }
        Err(e) => eprintln!("h3-server: replay {}: {e}", path.display()),
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_full_queue_drops_its_connection_and_frees_the_seat() {
        let (r, setup) = h3_pack::synthetic(h3_pack::SYNTHETIC_DUEL).unwrap();
        let players = ["a", "b"].map(|t| SeatKind::Player {
            token: String::from(t),
        });
        let config = GameConfig {
            game: 1,
            setup,
            seed: 1,
            checkpoint_every: 0,
        };
        let mut session = Session::new(&r, config, players).unwrap();
        let (to, mut queue) = mpsc::channel(1);
        let mut conns = BTreeMap::from([(ConnId(1), to)]);
        let join = |token: Option<&str>| ClientMessage::Join {
            game: 1,
            seat: Seat::Attacker,
            token: token.map(String::from),
        };
        let mut out = Vec::new();
        session.handle(ConnId(1), join(None), &mut out);
        assert_eq!(out.len(), 2, "Joined and Snapshot");
        for outbound in out.drain(..) {
            deliver(&mut conns, &mut session, outbound);
        }
        assert!(conns.is_empty(), "the snapshot found the queue full");
        let joined = queue.try_recv().unwrap();
        assert!(matches!(
            codec::decode_server(joined.text.as_bytes(), usize::MAX),
            Ok(ServerMessage::Joined { .. })
        ));
        // A frame for a connection the game forgot goes nowhere.
        session.handle(ConnId(1), join(None), &mut out);
        let refused = out.pop().unwrap();
        deliver(&mut conns, &mut session, refused);
        // The seat waits for its token.
        session.handle(ConnId(2), join(Some("a")), &mut out);
        assert!(matches!(
            codec::decode_server(out[0].frame.as_bytes(), usize::MAX),
            Ok(ServerMessage::Joined { .. })
        ));
    }
}
