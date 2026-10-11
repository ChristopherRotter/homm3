//! The hosted games (ADR-0018 points 3, 6, 8 and 12; E16 M2 T1–T3): the offers, compiled once at
//! start; a game prepared from an offer with secrets from the OS, or restored from its journal
//! after a restart or to resume it, and run on its own OS thread. The thread owns the session core
//! and the game's journal ([`crate::journal`]): it makes each call's entries durable before it
//! sends that call's frames. The connection tasks talk to it over bounded channels, and it tells
//! the lobby where it is through the registry ([`crate::registry`]).
//!
//! - **Verification** (E16 M2 T3, `E16-design.md` §8). A finished game's journal is read back and
//!   re-run in a fresh battle before its replay is published: every checkpoint, the live game's
//!   final hash and result must match, and the ruleset in memory must still hash to its content
//!   hash. A game that does not verify is void: its replay is withheld from the players, the
//!   journal moves to `<data>/void/` and the replay as played goes beside it with the diagnosis.
//! - **Pause** (E16 M2 T3). A game its host paused closes its connections and ends its thread; the
//!   registry keeps it listed as paused, and a seat's join with its token restores it.
//! - **The server's life** (E16 M2 T5a). A running game whose seats all stayed empty for
//!   `[limits] empty_game_ms` pauses at the registry's next tick, as if its host had paused it; a
//!   drain ends a game that waits for its players; a stop ends every game, its journal kept for
//!   the next start (one that never started leaves none).
//! - **A refused join** (E16 M2 T5b). A connection that holds nothing after its message (a join the
//!   session refused) is told so with the answer, and its task leaves the game: it neither keeps
//!   a waiting game listed nor escapes the lobby's deadline.

use std::collections::BTreeMap;
use std::fs;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use h3_battle::{BattleSide, BattleState, LegalCommands};
use h3_content::Ruleset;
use h3_contracts::BattleSetup;
use h3_kernel::{Rng, StreamId};
use h3_protocol::session::{Bot, ConnId, Entry, GameConfig, Journal, Outbound, SeatKind, Session};
use h3_protocol::wire::{self, ClientMessage, GameInfo, GoodbyeReason, Phase, Seat, ServerMessage};
use h3_protocol::{codec, map};
use h3_sim::{Replay, ReplayError, ReplayHeader};
use tokio::sync::{mpsc, oneshot};
use tokio::time::Instant;

use crate::Error;
use crate::config::{self, BotSeat};
use crate::journal::{self, JOURNAL_FORMAT_VERSION, Start, Writer};
use crate::registry::{Hosted, Shared, lock};

/// A frame for a connection task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Frame {
    /// The message's text.
    pub text: String,
    /// Whether the connection closes after it.
    pub close: bool,
    /// `Some(k)` on the last frame of the game's answer to a connection's `k`-th message when the
    /// connection, after it, holds no seat and does not watch (a refused join, E16 M2 T5b): its
    /// task then leaves the game, unless it passed it another message since.
    pub left: Option<u64>,
}

/// A connection's queue of frames from a game (ADR-0018 point 3; E16 M2 T5c): bounded in frames
/// by its channel and in bytes by a count both ends keep, the game adding what it queues and the
/// connection's task taking off what it takes out. Provisional.
#[derive(Debug)]
pub(crate) struct Outbox {
    to: mpsc::Sender<Frame>,
    bytes: Arc<AtomicUsize>,
    limit: usize,
}

/// The connection task's end of its [`Outbox`].
#[derive(Debug)]
pub(crate) struct FromGame {
    from: mpsc::Receiver<Frame>,
    bytes: Arc<AtomicUsize>,
}

/// A connection's queue from a game, of at most `frames` frames and `limit` bytes.
pub(crate) fn outbox(frames: usize, limit: usize) -> (Outbox, FromGame) {
    let (to, from) = mpsc::channel(frames);
    let bytes = Arc::new(AtomicUsize::new(0));
    let from = FromGame {
        from,
        bytes: Arc::clone(&bytes),
    };
    (Outbox { to, bytes, limit }, from)
}

impl Outbox {
    /// Queues `frame` if the queue has room for it in frames and in bytes; `false` if it has not
    /// (the game then drops the connection). A queue whose task is gone takes it unseen.
    pub(crate) fn offer(&self, frame: Frame) -> bool {
        let len = frame.text.len();
        if self.bytes.load(Ordering::Relaxed).saturating_add(len) > self.limit {
            return false;
        }
        self.bytes.fetch_add(len, Ordering::Relaxed);
        match self.to.try_send(frame) {
            Ok(()) => true,
            Err(e) => {
                self.bytes.fetch_sub(len, Ordering::Relaxed);
                matches!(e, mpsc::error::TrySendError::Closed(_))
            }
        }
    }

    /// The bytes queued (tests).
    #[cfg(test)]
    pub(crate) fn queued(&self) -> usize {
        self.bytes.load(Ordering::Relaxed)
    }

    /// Queues the connection's last frame from the game (a goodbye, small) whatever the bytes
    /// queued; a full queue misses it.
    pub(crate) fn last(&self, frame: Frame) {
        let len = frame.text.len();
        self.bytes.fetch_add(len, Ordering::Relaxed);
        if self.to.try_send(frame).is_err() {
            self.bytes.fetch_sub(len, Ordering::Relaxed);
        }
    }
}

impl FromGame {
    /// The next frame; `None` once the game dropped the queue and it is empty.
    pub(crate) async fn recv(&mut self) -> Option<Frame> {
        let frame = self.from.recv().await?;
        self.bytes.fetch_sub(frame.text.len(), Ordering::Relaxed);
        Some(frame)
    }
}

/// What a connection task tells a game.
#[derive(Debug)]
pub(crate) enum ToGame {
    /// A connection joins the game; its frames go to `to`.
    Open {
        /// The connection.
        conn: ConnId,
        /// Its outbound queue for this game.
        to: Outbox,
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
    /// The registry's sweep (E16 M2 T5a): a running game whose seats stayed empty long enough
    /// pauses.
    Tick,
    /// The server drains (E16 M2 T5a): a game that waits for its players ends.
    Drain,
    /// The server stops (E16 M2 T5a): the game ends, its connections hear it, its journal stays.
    Stop,
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
    /// How long its seats may all stay empty while it runs before it pauses (E16 M2 T5a).
    empty_game: Option<Duration>,
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
    /// At start: the game's inbox, for the registry once the game is listed again, and what is
    /// dropped when the re-run is done, however it went (the server listens after every one).
    /// `None` for a paused game a join resumes: the registry hosts it already (E16 M2 T3).
    listing: Option<(mpsc::Sender<ToGame>, oneshot::Sender<()>)>,
}

/// A deadline of `ms` milliseconds; none for 0 (E16 M2 T5a).
pub(crate) fn deadline(ms: u64) -> Option<Duration> {
    (ms > 0).then(|| Duration::from_millis(ms))
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
    /// the battle starts, then creates the game's journal (E16 M2 T2) for its creator at `origin`,
    /// who chose whether it takes spectators (`watchable`, E16 M2 T4c).
    pub(crate) fn new(
        offer: Arc<Offer>,
        game: u32,
        origin: IpAddr,
        (server, limits): (&config::Server, &config::Limits),
        watchable: bool,
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
            shadow: server.shadow_instance,
            spectators: limits.spectators_per_game,
            record_bytes: limits.record_bytes,
            watchable,
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
            empty_game: deadline(limits.empty_game_ms),
            begin: Begin::New(journal),
        })
    }

    /// The game a journal holds, to restore (E16 M2 T2): the offer `offer` its first line names,
    /// read from `path`. Why the journal is void otherwise: its replay header does not match the
    /// offer's engine and content (`ReplayHeader::check`), or its replay does not read on them.
    /// At start (`listing`), the thread drops `done` when its re-run is done, and lists the game
    /// with the inbox `to` if it goes on; without, it resumes a paused game (E16 M2 T3).
    pub(crate) fn restored(
        offer: Arc<Offer>,
        path: &Path,
        read: journal::Read,
        (server, limits): (&config::Server, &config::Limits),
        listing: Option<(mpsc::Sender<ToGame>, oneshot::Sender<()>)>,
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
            shadow: server.shadow_instance,
            spectators: limits.spectators_per_game,
            record_bytes: limits.record_bytes,
            // A game restores only if it started, and its entries hold the setting it started
            // with (`Entry::Spectators`).
            watchable: true,
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
            empty_game: deadline(limits.empty_game_ms),
            begin: Begin::Restored(Restored {
                path: path.to_path_buf(),
                read,
                listing,
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
    /// leaves the lobby and ends (E16 M2 T1); one that never started leaves no journal. A game that
    /// paused closes its connections and ends, listed as paused (E16 M2 T3).
    fn run(self, mut inbox: mpsc::Receiver<ToGame>, registry: &Shared) {
        let Self {
            offer,
            scenario,
            config,
            tokens,
            bot_seed,
            replay_path,
            data,
            empty_game,
            begin,
        } = self;
        let game = config.game;
        let mut bots: u64 = 0;
        let seats = tokens.clone().map(|token| match token {
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
                (&offer.ruleset, &scenario, &replay_path, &data),
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
                let host = (data.as_path(), registry);
                let resumed = resume(&offer, (config, seats, &tokens), restored, host, over);
                let Some(resumed) = resumed else {
                    return turn_away(&mut inbox, GoodbyeReason::Shutdown);
                };
                resumed
            }
        };
        let mut journal = Some(journal);
        let mut conns = BTreeMap::new();
        // How many messages each connection passed the game (E16 M2 T5b).
        let mut passed: BTreeMap<ConnId, u64> = BTreeMap::new();
        let mut out = Vec::new();
        let status = |s: &Session<'_>| (s.phase(), s.open_seats(), s.takes_spectators());
        let mut reported = status(&session);
        let mut empty_since = (!session.seated()).then(Instant::now);
        while let Some(event) = inbox.blocking_recv() {
            let mut left = None;
            match event {
                // Nothing changes for the session until the connection's join.
                ToGame::Open { conn, to } => {
                    conns.insert(conn, to);
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
                ToGame::Tick => {
                    let due = empty_game
                        .zip(empty_since)
                        .is_some_and(|(after, since)| since.elapsed() >= after);
                    if !(due && session.suspend(&mut out)) {
                        continue;
                    }
                    eprintln!("h3-server: game {game}: its seats stayed empty");
                }
                ToGame::Drain if session.started() => continue,
                ToGame::Drain | ToGame::Stop => {
                    close_all(&conns, &[], GoodbyeReason::Shutdown);
                    if !session.started()
                        && let Some(journal) = journal.take()
                    {
                        journal.remove();
                    }
                    lock(registry).leave(game);
                    return turn_away(&mut inbox, GoodbyeReason::Shutdown);
                }
            }
            empty_since = if session.seated() {
                None
            } else {
                empty_since.or_else(|| Some(Instant::now()))
            };
            let entries = session.drain_entries();
            let written = journal.as_mut().map_or(Ok(()), |j| j.append(entries));
            if let Err(e) = written {
                eprintln!("h3-server: game {game}: the journal does not write ({e}): it stops");
                return stop(&conns, registry, game);
            }
            for step in session.drain_mismatches() {
                eprintln!(
                    "h3-server: game {game}: the shadow instance disagreed at command {step}; \
                     both battles were rebuilt from the journal"
                );
            }
            let closed: Vec<ConnId> = out.iter().filter(|o| o.close).map(|o| o.to).collect();
            let last =
                left.and_then(|(conn, k)| Some((out.iter().rposition(|o| o.to == conn)?, k)));
            for (i, outbound) in out.drain(..).enumerate() {
                let left = last.filter(|&(at, _)| at == i).map(|(_, k)| k);
                deliver(&mut conns, &mut session, outbound, left);
            }
            if session.paused() {
                close_all(&conns, &closed, GoodbyeReason::Paused);
                lock(registry).pause(game, tokens);
                turn_away(&mut inbox, GoodbyeReason::Paused);
                return eprintln!("h3-server: game {game} paused");
            }
            if session.result().is_some()
                && let Some(journal) = journal.take()
            {
                match over(&session, journal) {
                    Some(text) => session.publish_replay(text),
                    None => session.withhold_replay(),
                }
            }
            let now = status(&session);
            let mut registry = lock(registry);
            if now != reported {
                registry.report(game, now.clone());
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
/// leaves once `over` wrote its replay. At start, a running one is listed again, its seats empty,
/// and a paused one is listed as paused, its thread ending; a paused one a join resumes goes on
/// (E16 M2 T3).
fn resume<'r>(
    offer: &'r Offer,
    (config, seats, tokens): (GameConfig, [SeatKind; 2], &[Option<String>; 2]),
    restored: Restored,
    (data, registry): (&Path, &Shared),
    over: impl FnOnce(&Session<'r>, Writer) -> Option<String>,
) -> Option<(Session<'r>, Writer)> {
    let Restored {
        path,
        read,
        listing,
    } = restored;
    let game = config.game;
    let gone = || lock(registry).leave(game);
    // A paused game's deadline counts from its pause, the journal's last write (E16 M2 T5a).
    let paused_for = fs::metadata(&path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|written| written.elapsed().ok())
        .unwrap_or_default();
    let now = Instant::now();
    let since = now.checked_sub(paused_for).unwrap_or(now);
    let mut session = match Session::restore(&offer.ruleset, config, seats, &read.entries) {
        Ok(session) => session,
        Err(e) => {
            journal::void(data, &path, &game.to_string(), &format!("{e:?}"));
            gone();
            return None;
        }
    };
    let reopened = Writer::reopen(&path, read.complete);
    let journal = match reopened.and_then(|mut j| j.append(session.drain_entries()).map(|()| j)) {
        Ok(journal) => journal,
        Err(e) => {
            eprintln!("h3-server: game {game}: the journal does not write ({e}): not restored");
            gone();
            return None;
        }
    };
    if session.result().is_some() {
        over(&session, journal);
        gone();
        return None;
    }
    if !session.started() {
        journal.remove();
        gone();
        return None;
    }
    let Some((to, done)) = listing else {
        eprintln!("h3-server: game {game} resumed");
        return Some((session, journal));
    };
    let info = GameInfo {
        game,
        offer: offer.id.clone(),
        phase: session.phase(),
        open: session.open_seats(),
        spectators: session.takes_spectators(),
    };
    let paused = session.paused();
    let hosted = if paused {
        Hosted::Paused {
            tokens: tokens.clone(),
            since,
        }
    } else {
        Hosted::Live { to, bound: 0 }
    };
    lock(registry).reopen(read.start.origin, info, hosted);
    drop(done);
    if paused {
        eprintln!("h3-server: game {game} restored, paused");
        return None;
    }
    eprintln!("h3-server: game {game} restored");
    Some((session, journal))
}

/// Every connection in `conns` but those in `closed` hears `reason`, its last frame from the game.
fn close_all(conns: &BTreeMap<ConnId, Outbox>, closed: &[ConnId], reason: GoodbyeReason) {
    let text = codec::encode_text(&ServerMessage::Goodbye { reason });
    for (conn, to) in conns {
        if !closed.contains(conn) {
            to.last(Frame {
                text: text.clone(),
                close: true,
                left: None,
            });
        }
    }
}

/// The journal stopped: every connection hears the server shuts down (for this game), and the game
/// leaves the lobby; its journal stays for the next start.
fn stop(conns: &BTreeMap<ConnId, Outbox>, registry: &Shared, game: u32) {
    close_all(conns, &[], GoodbyeReason::Shutdown);
    lock(registry).leave(game);
}

/// The game's thread ends (E16 M2 T3): no connection may come any more, and those already on the
/// way hear `reason` (the game paused, or it did not resume: the server shuts down for it).
fn turn_away(inbox: &mut mpsc::Receiver<ToGame>, reason: GoodbyeReason) {
    inbox.close();
    let text = codec::encode_text(&ServerMessage::Goodbye { reason });
    while let Ok(event) = inbox.try_recv() {
        if let ToGame::Open { to, .. } = event {
            to.last(Frame {
                text: text.clone(),
                close: true,
                left: None,
            });
        }
    }
}

/// Queues a frame for its connection, with its mark `left` ([`Frame::left`], E16 M2 T5b). A queue
/// full in frames or in bytes (E16 M2 T5c) drops the connection: the game forgets it, and its
/// task, finding the queue closed, says `TooSlow` (ADR-0018 point 3).
pub(crate) fn deliver(
    conns: &mut BTreeMap<ConnId, Outbox>,
    session: &mut Session<'_>,
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
    };
    if !to.offer(frame) {
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

/// Game `game` is over (E16 M2 T3): it is verified first ([`verify`]). The replay of a game that
/// verifies is written as the CLI's file, then its journal goes (the replay is the finished
/// journal); a replay that does not write keeps the journal, which the next start re-runs to write
/// it again. A game that does not verify is void: its journal moves to `<data>/void/`, and the
/// replay of the game as it was played goes beside it, the diagnosis in its first line. The
/// replay's text for the participants, or `None`: withheld.
fn finish(
    (ruleset, scenario, path, data): (&Ruleset, &str, &Path, &Path),
    game: u32,
    session: &Session<'_>,
    journal: Writer,
) -> Option<String> {
    let journal_path = journal.path().to_path_buf();
    let (diagnosis, why) = match verify(ruleset, &journal_path, session) {
        Ok(text) => {
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
            return Some(text);
        }
        Err(corrupt) => corrupt,
    };
    drop(journal);
    let void = data.join("void");
    let beside = void.join(path.file_name().unwrap_or_default());
    let end = (session.state_hash(), session.result());
    let written = replay_text(ruleset, scenario, session.journal(), end).and_then(|text| {
        journal::create_dir(&void)
            .and_then(|()| fs::write(&beside, format!("# void: {diagnosis}\n{text}")))
            .map_err(|e| Error(e.to_string()))
    });
    journal::void(data, &journal_path, &game.to_string(), &why);
    match written {
        Ok(()) => eprintln!(
            "h3-server: game {game}: the replay as played is {}",
            beside.display()
        ),
        Err(e) => eprintln!("h3-server: game {game}: the replay as played: {}", e.0),
    }
    None
}

/// Verifies a finished game before its replay is published (E16 M2 T3, `E16-design.md` §8): the
/// ruleset in memory still hashes to the content hash it was compiled with, and the journal on
/// disk, re-run in a fresh battle, matches each of its checkpoints and ends in the live game's
/// state hash and result. The replay as the CLI's file; else the diagnosis, which may name hashes
/// (for the void directory, never a log line), and why without them (for the log).
fn verify(
    ruleset: &Ruleset,
    journal: &Path,
    session: &Session<'_>,
) -> Result<String, (String, String)> {
    let alike = |why: String| (why.clone(), why);
    if h3_kernel::hash::hash_of(ruleset) != ruleset.content_hash() {
        return Err(alike(String::from(
            "the ruleset in memory changed since it was compiled",
        )));
    }
    let read =
        journal::read(journal).map_err(|e| alike(format!("the journal does not read: {e}")))?;
    // The first line's replay holds the seed: its errors stay out of the log.
    let unread = |e: h3_cli::CliError| {
        (
            format!("the journal's replay does not read: {e}"),
            String::from("the journal's replay does not read"),
        )
    };
    let doc = h3_cli::ReplayDoc::parse(&read.start.replay).map_err(unread)?;
    let scenario = doc.scenario().to_owned();
    let mut replay = doc.into_replay(ruleset).map_err(unread)?;
    for entry in read.entries {
        match entry {
            Entry::Command(side, command) => replay.commands.push((side, command)),
            Entry::Checkpoint(hash) => replay.checkpoints.push(hash),
            Entry::Taken(_)
            | Entry::Started
            | Entry::Paused
            | Entry::Resumed
            | Entry::Spectators(_) => {}
        }
    }
    replay.final_hash = session.state_hash();
    replay.result = session.result();
    replay
        .verify(ruleset)
        .map_err(|e| (e.to_string(), unhashed(&e)))?;
    h3_cli::replay_to_toml(&replay, &scenario, ruleset)
        .map_err(|e| alike(format!("the replay does not render: {e}")))
}

/// Why a game does not verify, without the hashes (ADR-0018 point 8: no hash in a log line).
fn unhashed(error: &ReplayError) -> String {
    match error {
        ReplayError::HashMismatch { step, .. } => {
            format!("the state hash after {step} commands differs")
        }
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A game whose attacker joins over a queue of `frames` frames and of the bytes `limit` gives
    /// for the join's frames; the queue must hold `Joined` but not the `Snapshot` after it.
    fn a_full_queue_drops_its_connection_and_frees_the_seat(
        frames: usize,
        limit: impl Fn(&[Outbound]) -> usize,
    ) {
        let (r, setup) = h3_pack::synthetic(h3_pack::SYNTHETIC_DUEL).unwrap();
        let players = ["a", "b"].map(|t| SeatKind::Player {
            token: String::from(t),
        });
        let config = GameConfig {
            game: 1,
            setup,
            seed: 1,
            checkpoint_every: 0,
            shadow: false,
            spectators: 0,
            watchable: true,
            record_bytes: 1 << 20,
        };
        let mut session = Session::new(&r, config, players).unwrap();
        let join = |token: Option<&str>| ClientMessage::Join {
            game: 1,
            seat: Seat::Attacker,
            token: token.map(String::from),
        };
        let mut out = Vec::new();
        session.handle(ConnId(1), join(None), &mut out);
        assert_eq!(out.len(), 2, "Joined and Snapshot");
        let (to, mut queue) = outbox(frames, limit(&out));
        let mut conns = BTreeMap::from([(ConnId(1), to)]);
        for outbound in out.drain(..) {
            deliver(&mut conns, &mut session, outbound, None);
        }
        assert!(conns.is_empty(), "the snapshot found the queue full");
        let joined = queue.from.try_recv().unwrap();
        assert!(matches!(
            codec::decode_server(joined.text.as_bytes(), usize::MAX),
            Ok(ServerMessage::Joined { .. })
        ));
        // A frame for a connection the game forgot goes nowhere.
        session.handle(ConnId(1), join(None), &mut out);
        let refused = out.pop().unwrap();
        deliver(&mut conns, &mut session, refused, None);
        // The seat waits for its token.
        session.handle(ConnId(2), join(Some("a")), &mut out);
        assert!(matches!(
            codec::decode_server(out[0].frame.as_bytes(), usize::MAX),
            Ok(ServerMessage::Joined { .. })
        ));
    }

    #[test]
    fn a_queue_full_in_frames_drops_its_connection() {
        a_full_queue_drops_its_connection_and_frees_the_seat(1, |_| usize::MAX);
    }

    /// E16 M2 T5c.
    #[test]
    fn a_queue_full_in_bytes_drops_its_connection() {
        a_full_queue_drops_its_connection_and_frees_the_seat(64, |out| out[0].frame.len());
    }

    /// E16 M2 T5c: the bytes a frame holds are counted while it is queued; a goodbye is queued
    /// whatever the bytes; a frame the closed queue takes is counted nowhere.
    #[tokio::test]
    async fn a_queue_counts_the_bytes_it_holds() {
        let frame = |text: &str| Frame {
            text: String::from(text),
            close: false,
            left: None,
        };
        let (to, mut from) = outbox(8, 10);
        assert!(to.offer(frame("abcdef")) && to.queued() == 6);
        assert!(!to.offer(frame("ghijk")), "11 bytes");
        assert!(to.offer(frame("ghij")) && to.queued() == 10);
        to.last(frame("goodbye"));
        assert_eq!(to.queued(), 17);
        assert_eq!(from.recv().await.unwrap().text, "abcdef");
        assert_eq!(to.queued(), 11);
        let (to, from) = outbox(8, 10);
        drop(from);
        assert!(to.offer(frame("abc")), "a queue whose task is gone");
        assert_eq!(to.queued(), 0);
    }
}
