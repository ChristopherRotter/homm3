//! The registry (E16 M2 T1): the lobby ([`h3_protocol::lobby`]) and each game's inbox, shared by
//! the connection tasks and the games' threads behind one lock. A connection task looks a game up
//! and binds to it, or creates one; a game's thread reports where it is, and leaves when it is not
//! running and no connection is bound to it. Binding and leaving happen under the same lock, so a
//! connection never binds to a game that has left. No file is read or written under the lock (E16
//! M2 T5e): a new or resumed game makes its journal ready on its own thread, its creator's or
//! resumer's join already first in its inbox.
//!
//! At start, before the server listens, the games whose journals the data directory holds are
//! restored and listed again (E16 M2 T2). A paused game has no thread: the registry keeps its
//! seats' tokens, and a join with one restores it from its journal (E16 M2 T3).
//!
//! The server's life (E16 M2 T5a): a sweep ticks every game's thread, which pauses a running game
//! whose seats stayed empty, and voids a paused game nobody resumed in time; a drain refuses new
//! games and ends the waiting ones; a stop ends every game's thread, their journals kept.
//!
//! Provisional (E16 M2 T1–T3, T5e): the lock, the origin (the client's address), a creation whose
//! thread does not start refused as `TooManyGames`, one whose secrets or journal fail on its thread
//! ended with `Goodbye { Shutdown }` (its `GameCreated` sent already), the token checked before a
//! paused game is restored.

use std::collections::BTreeMap;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;
use std::{fs, io};

use h3_protocol::codec;
use h3_protocol::lobby::{self, Lobby};
use h3_protocol::session::{self, ConnId};
use h3_protocol::wire::{ClientMessage, GameInfo, Phase, Refusal, Seat};
use tokio::sync::{mpsc, oneshot};
use tokio::time::Instant;

use crate::archive::Archive;
use crate::game::{self, Offer, Outbox, Prepared, ToGame};
use crate::{Error, config, journal};

/// A game's inbox: messages waiting for the game's thread.
const GAME_INBOX: usize = 1024;

/// The registry, as the tasks and threads share it.
pub(crate) type Shared = Arc<Mutex<Registry>>;

/// Locks the registry (a thread that panicked under the lock leaves it usable).
pub(crate) fn lock(shared: &Shared) -> MutexGuard<'_, Registry> {
    shared.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A hosted game.
#[derive(Debug)]
pub(crate) enum Hosted {
    /// Its thread runs: its inbox, and how many connections are bound to it.
    Live {
        to: mpsc::Sender<ToGame>,
        bound: usize,
    },
    /// Paused (E16 M2 T3): its journal, the seats' tokens that resume it (none for the bot's), and
    /// since when (E16 M2 T5a: its deadline).
    Paused {
        tokens: [Option<String>; 2],
        since: Instant,
    },
}

/// The lobby and the hosted games.
#[derive(Debug)]
pub(crate) struct Registry {
    lobby: Lobby<IpAddr>,
    offers: Vec<Arc<Offer>>,
    games: BTreeMap<u32, Hosted>,
    server: config::Server,
    /// The server's bounds: how many spectators each game takes (E16 M2 T4), the deadlines of
    /// abandoned games (E16 M2 T5a).
    limits: config::Limits,
    /// The finished games' replays (E16 M2 T5d).
    archive: Archive,
}

impl Registry {
    /// A registry of `offers` within `limits`, and no games.
    pub(crate) fn new(
        offers: Vec<Arc<Offer>>,
        limits: &config::Limits,
        server: config::Server,
    ) -> Self {
        let infos = offers.iter().map(|o| o.info.clone()).collect();
        let games = lobby::Limits {
            games: limits.games,
            games_per_origin: limits.games_per_address,
        };
        Self {
            lobby: Lobby::new(infos, games),
            offers,
            games: BTreeMap::new(),
            server,
            limits: *limits,
            archive: Archive::default(),
        }
    }

    /// Reads the archive back from the data directory (E16 M2 T5d; at start, before the restore,
    /// which may finish a game).
    pub(crate) fn load_archive(&mut self) {
        self.archive = Archive::load(&self.server.data, self.limits.replays_kept);
    }

    /// The replay just written to `path` joins the archive, listed or not (E16 M2 T5d).
    pub(crate) fn archived(&mut self, path: PathBuf, listed: bool) {
        self.archive.add(path, listed);
    }

    /// The answer to `ListReplays`, as a frame's text (E16 M2 T5d).
    pub(crate) fn replays(&self, seq: u32, from: u32) -> String {
        let (total, replays) = self.archive.page(from);
        codec::encode_text(&h3_protocol::wire::ServerMessage::Replays {
            seq,
            total,
            replays,
        })
    }

    /// The file of the listed replay `id` (E16 M2 T5d).
    pub(crate) fn replay(&self, id: &str) -> Option<PathBuf> {
        self.archive.path(id)
    }

    /// The server drains (E16 M2 T5a): the lobby refuses new games, and every game's thread hears
    /// it (a waiting one ends; a running one plays on).
    pub(crate) fn drain(&mut self) {
        self.lobby.drain();
        self.tell_all(|| ToGame::Drain);
    }

    /// How many games wait or run (E16 M2 T5a): a draining server stops at none.
    pub(crate) fn in_play(&self) -> usize {
        self.lobby.in_play()
    }

    /// How many games have a thread (E16 M2 T5a): a stopping server waits for none.
    pub(crate) fn live(&self) -> usize {
        let live = |hosted: &&Hosted| matches!(hosted, Hosted::Live { .. });
        self.games.values().filter(live).count()
    }

    /// Every game's thread ends, their journals kept for the next start (E16 M2 T5a).
    pub(crate) fn stop_all(&mut self) {
        self.tell_all(|| ToGame::Stop);
    }

    /// The deadlines of abandoned games at `now` (E16 M2 T5a): every game's thread is ticked (it
    /// pauses a running game whose seats stayed empty), and a paused game nobody resumed within
    /// `paused_game_ms` is void.
    pub(crate) fn sweep(&mut self, now: Instant) {
        self.tell_all(|| ToGame::Tick);
        let Some(limit) = (self.limits.paused_game_ms > 0)
            .then(|| Duration::from_millis(self.limits.paused_game_ms))
        else {
            return;
        };
        let expired: Vec<u32> = self
            .games
            .iter()
            .filter(|(_, hosted)| {
                matches!(hosted, Hosted::Paused { since, .. }
                    if now.saturating_duration_since(*since) >= limit)
            })
            .map(|(&game, _)| game)
            .collect();
        for game in expired {
            let path = journal::dir(&self.server.data).join(format!("{game}.journal"));
            let why = format!("abandoned: paused for {} ms", self.limits.paused_game_ms);
            journal::void(&self.server.data, &path, &game.to_string(), &why);
            self.leave(game);
        }
    }

    /// Every live game's thread gets `message` (one that is too busy to take it misses it).
    fn tell_all(&self, message: impl Fn() -> ToGame) {
        for hosted in self.games.values() {
            if let Hosted::Live { to, .. } = hosted {
                let _ = to.try_send(message());
            }
        }
    }

    /// The answer to `ListGames`, as a frame's text.
    pub(crate) fn list(&self, seq: u32) -> String {
        codec::encode_text(&self.lobby.list(seq))
    }

    /// A connection bound to game `game` through the inbox `to` leaves it (a connection bound to
    /// the game before it paused does not count for the game a resume restored).
    pub(crate) fn unbind(&mut self, game: u32, to: &mpsc::Sender<ToGame>) {
        if let Some(Hosted::Live { to: inbox, bound }) = self.games.get_mut(&game)
            && inbox.same_channel(to)
        {
            *bound = bound.saturating_sub(1);
        }
    }

    /// Where game `game` is now: its phase, its open seats and whether it takes spectators.
    pub(crate) fn report(&mut self, game: u32, status: (Phase, Vec<Seat>, bool)) {
        self.lobby.update(game, status);
    }

    /// Whether game `game`, in `phase`, leaves now: when it is not running and no connection is
    /// bound to it. Then it is gone from the lobby, and its thread ends.
    pub(crate) fn close_if_idle(&mut self, game: u32, phase: Phase) -> bool {
        let idle = !matches!(self.games.get(&game), Some(Hosted::Live { bound, .. }) if *bound > 0);
        if idle && phase != Phase::Running {
            self.leave(game);
        }
        idle && phase != Phase::Running
    }

    /// Lists a game a restart restored, as `origin` created it, and hosts it (E16 M2 T2, T3).
    pub(crate) fn reopen(&mut self, origin: IpAddr, info: GameInfo, hosted: Hosted) {
        let game = info.game;
        self.lobby.reopen(origin, info);
        self.games.insert(game, hosted);
    }

    /// Game `game` paused (E16 M2 T3): its thread ends, `tokens` resume it.
    pub(crate) fn pause(&mut self, game: u32, tokens: [Option<String>; 2]) {
        self.lobby.update(game, (Phase::Paused, Vec::new(), false));
        let since = Instant::now();
        self.games.insert(game, Hosted::Paused { tokens, since });
    }

    /// Hosts a game whose inbox is `to`, outside the lobby (tests of the connection task).
    #[cfg(test)]
    pub(crate) fn host(&mut self, game: u32, to: mpsc::Sender<ToGame>) {
        self.games.insert(game, Hosted::Live { to, bound: 0 });
    }

    /// Game `game` is gone.
    pub(crate) fn leave(&mut self, game: u32) {
        self.games.remove(&game);
        self.lobby.remove(game);
    }
}

/// A connection's opening of a game it binds to (E16 M2 T5e): the connection, its outbound queue
/// for the game and its join.
#[derive(Debug)]
pub(crate) struct Opening {
    /// The connection.
    pub conn: ConnId,
    /// Its outbound queue for the game.
    pub to: Outbox,
    /// Its join.
    pub join: ClientMessage,
}

impl Opening {
    /// Queues the opening in a game's fresh inbox, which has room for it, before the game is
    /// hosted: it comes before anything another connection sends the game (E16 M2 T5e: the
    /// reviewer's finding; a stranger's join cannot take a new game's creator's seat, nor make a
    /// resuming game pause again).
    fn first(self, inbox: &mpsc::Sender<ToGame>) {
        let Self { conn, to, join } = self;
        let _ = inbox.try_send(ToGame::Open { conn, to });
        let _ = inbox.try_send(ToGame::Message {
            conn,
            message: join,
        });
    }
}

/// Creates a game of `offer` for a client at `origin`, whose creator takes `seat` and chose
/// whether it takes spectators (`watchable`, E16 M2 T4c), and binds the creator (`conn`, its queue
/// `to`) to it: the game's id and inbox, or why not. The creator's join is the game's first
/// message; the game draws its secrets and creates its journal on its own thread
/// ([`game::launch`]), so no file is written under the lock (E16 M2 T5e).
pub(crate) fn create(
    shared: &Shared,
    origin: IpAddr,
    (offer, seat, watchable): (&str, Seat, bool),
    (conn, to): (ConnId, Outbox),
) -> Result<(u32, mpsc::Sender<ToGame>), Refusal> {
    let mut registry = lock(shared);
    let listed = watchable && registry.limits.spectators_per_game > 0;
    let (game, index) = registry.lobby.create(origin, offer, seat, listed)?;
    let offer = Arc::clone(&registry.offers[index]);
    let (server, limits) = (registry.server.clone(), registry.limits);
    let prepare = move || match Prepared::new(offer, game, origin, (&server, &limits), watchable) {
        Ok(prepared) => Some(prepared),
        Err(e) => {
            eprintln!("h3-server: game {game} not created: {e}");
            None
        }
    };
    let (to_game, inbox) = mpsc::channel(GAME_INBOX);
    let join = ClientMessage::Join {
        game,
        seat,
        token: None,
    };
    Opening { conn, to, join }.first(&to_game);
    if let Err(e) = game::launch(game, prepare, inbox, Arc::clone(shared)) {
        eprintln!("h3-server: game {game} not created: {e}");
        registry.lobby.remove(game);
        return Err(Refusal::TooManyGames);
    }
    registry.games.insert(
        game,
        Hosted::Live {
            to: to_game.clone(),
            bound: 1,
        },
    );
    Ok((game, to_game))
}

/// Binds a connection that joins `seat` of game `game` with `token`: the game's inbox, and the
/// connection's `opening` back if its task is to pass it (a game that runs), or why not. A paused
/// game is restored first (E16 M2 T3), only for a seat's token: a spectator, the bot's seat, a
/// missing or another token are refused as a running game refuses them. It is restored on its own
/// thread ([`game::launch`]; E16 M2 T5e: no file read under the lock), the opening its first
/// message; a paused journal that no longer restores is void, the game gone, and its joiner hears
/// `Goodbye { Shutdown }`.
pub(crate) fn bind(
    shared: &Shared,
    (game, seat, token): (u32, Seat, Option<&str>),
    opening: Opening,
) -> Result<(mpsc::Sender<ToGame>, Option<Opening>), Refusal> {
    let mut registry = lock(shared);
    let tokens = match registry.games.get_mut(&game) {
        None => return Err(Refusal::NoSuchGame),
        Some(Hosted::Live { to, bound }) => {
            *bound += 1;
            return Ok((to.clone(), Some(opening)));
        }
        Some(Hosted::Paused { tokens, .. }) => tokens,
    };
    let expected = match seat {
        Seat::Attacker => &tokens[0],
        Seat::Defender => &tokens[1],
        Seat::Spectator => return Err(Refusal::NoSpectators),
    };
    match (expected, token) {
        (Some(expected), Some(given)) if session::same_token(expected, given) => {}
        (Some(_), Some(_)) => return Err(Refusal::BadToken),
        (None, _) | (_, None) => return Err(Refusal::SeatTaken),
    }
    let path = journal::dir(&registry.server.data).join(format!("{game}.journal"));
    let (offers, server, limits) = (
        registry.offers.clone(),
        registry.server.clone(),
        registry.limits,
    );
    let prepare = move || {
        let prepared = journal::read(&path).and_then(|read| {
            let offer = offers
                .iter()
                .find(|o| o.id == read.start.offer)
                .cloned()
                .ok_or_else(|| format!("the server offers no `{}` now", read.start.offer))?;
            Prepared::restored(offer, &path, read, (&server, &limits), None)
        });
        match prepared {
            Ok(prepared) => Some(prepared),
            Err(why) => {
                journal::void(&server.data, &path, &game.to_string(), &why);
                None
            }
        }
    };
    let (to_game, inbox) = mpsc::channel(GAME_INBOX);
    opening.first(&to_game);
    if let Err(e) = game::launch(game, prepare, inbox, Arc::clone(shared)) {
        eprintln!("h3-server: game {game} not resumed: {e}");
        return Err(Refusal::NoSuchGame);
    }
    registry.games.insert(
        game,
        Hosted::Live {
            to: to_game.clone(),
            bound: 1,
        },
    );
    Ok((to_game, None))
}

/// Restores the games whose journals the data directory holds (E16 M2 T2; call before the server
/// listens): each re-runs on its own thread, which lists it again if it goes on; this returns once
/// every re-run is done. A journal that does not read, that names an offer the server no longer
/// has, or whose engine or content identity is not the offer's, is void. A game thread that does
/// not start stops the server's start.
pub(crate) async fn restore(shared: &Shared) -> Result<(), Error> {
    let data = lock(shared).server.data.clone();
    let dir = journal::dir(&data);
    let listing = match fs::read_dir(&dir) {
        Ok(listing) => listing,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(Error(format!("{}: {e}", dir.display()))),
    };
    let mut paths: Vec<PathBuf> = listing
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|e| e == "journal"))
        .collect();
    paths.sort();
    let mut restoring = Vec::new();
    for path in paths {
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let (to, inbox) = mpsc::channel(GAME_INBOX);
        let (done, finished) = oneshot::channel();
        let prepared = journal::read(&path).and_then(|read| {
            let registry = lock(shared);
            let offer = registry
                .offers
                .iter()
                .find(|o| o.id == read.start.offer)
                .cloned()
                .ok_or_else(|| format!("the server offers no `{}` now", read.start.offer))?;
            let host = (&registry.server, &registry.limits);
            Prepared::restored(offer, &path, read, host, Some((to, done)))
        });
        match prepared {
            Ok(prepared) => {
                prepared.spawn(inbox, Arc::clone(shared))?;
                restoring.push(finished);
            }
            Err(why) => journal::void(&data, &path, &name, &why),
        }
    }
    for finished in restoring {
        let _ = finished.await;
    }
    Ok(())
}

/// Sweeps the registry for abandoned games (E16 M2 T5a), every quarter of its shortest deadline,
/// at least every minute and at most every 10 ms.
pub(crate) async fn sweep_every(shared: Shared, limits: config::Limits) {
    let shortest = [limits.empty_game_ms, limits.paused_game_ms]
        .into_iter()
        .filter(|&ms| ms > 0)
        .min()
        .unwrap_or(u64::MAX);
    let period = Duration::from_millis((shortest / 4).clamp(10, 60_000));
    let mut tick = tokio::time::interval(period);
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        tick.tick().await;
        lock(&shared).sweep(Instant::now());
    }
}
