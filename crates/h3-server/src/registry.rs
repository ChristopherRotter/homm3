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
//! seats' tokens and accounts, and a join with a seat's token, or as its account, restores it from
//! its journal (E16 M2 T3, M3 T3).
//!
//! The server's life (E16 M2 T5a): a sweep ticks every game's thread, which pauses a running game
//! whose seats stayed empty, and voids a paused game nobody resumed in time; a drain refuses new
//! games and ends the waiting ones; a stop ends every game's thread, their journals kept.
//!
//! The ladders' queues (E16 M4 T2; [`h3_protocol::ladder`]) live here too: a queued connection
//! hears from its queue through a channel of its own; a match both players accepted becomes a game
//! of its ladder's pool whose seats are the two accounts' from the start, created as the server's
//! own (no client's limit applies), and listed with its ladder until it is over.
//!
//! Provisional (E16 M2 T1–T3, T5e): the lock, the origin (the client's address), a creation whose
//! thread does not start refused as `TooManyGames`, one whose secrets or journal fail on its thread
//! ended with `Goodbye { Shutdown }` (its `GameCreated` sent already), the token checked before a
//! paused game is restored.

use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv6Addr};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;
use std::{fs, io};

use h3_protocol::codec;
use h3_protocol::ladder::{Matched, Queues, Told};
use h3_protocol::lobby::{self, Lobby};
use h3_protocol::session::{self, ConnId};
use h3_protocol::wire::{
    ClientMessage, GameInfo, Phase, Refusal, Seat, Seated, ServerMessage, UnqueueReason,
};
use tokio::sync::{mpsc, oneshot};
use tokio::time::Instant;

use crate::accounts::Accounts;
use crate::archive::Archive;
use crate::game::{self, Offer, Outbox, Prepared, ToGame};
use crate::ladders::{Ladders, Pool};
use crate::{Error, config, journal};

/// A game's inbox: messages waiting for the game's thread.
const GAME_INBOX: usize = 1024;

/// How often the matcher pairs the waiting players and gives up unanswered matches (E16 M4 T2).
const MATCH_EVERY: Duration = Duration::from_secs(1);

/// Where a queued connection hears from its queue (E16 M4 T2): a frame's text, and whether the
/// connection still waits in a queue after it (its lobby deadline waits meanwhile).
pub(crate) type ToConn = mpsc::Sender<(String, bool)>;

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
    /// Paused (E16 M2 T3): its journal, the seats' tokens that resume it (none for the bot's), the
    /// accounts the seats belong to (E16 M3 T3: an account's seat resumes for the account alone),
    /// and since when (E16 M2 T5a: its deadline).
    Paused {
        tokens: [Option<String>; 2],
        accounts: [Option<String>; 2],
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
    /// The accounts (E16 M3 T3): a game's thread names its seats' accounts and asks their
    /// preferences, never under the lock.
    accounts: Arc<Accounts>,
    /// The ladders' pools by ladder (E16 M4 T2): a match's game draws from them, and a restored
    /// or resumed game finds its offer among them too.
    pools: BTreeMap<String, Pool>,
    /// The ladders' queues (E16 M4 T2).
    queues: Queues<ConnId>,
    /// Where each connection that queued hears from its queue.
    queued: BTreeMap<ConnId, ToConn>,
    /// The matches' games that are not over, with their ladders and their seats' accounts: an
    /// account waits for no other match on the ladder meanwhile (ADR-0019 point 8).
    ranked: BTreeMap<u32, (String, [String; 2])>,
    /// The queues' clock starts here.
    epoch: Instant,
}

impl Registry {
    /// A registry of `offers` and of `ladders`' queues within `limits`, and no games; its games'
    /// players are `accounts`.
    pub(crate) fn new(
        (offers, ladders): (Vec<Arc<Offer>>, &Ladders),
        limits: &config::Limits,
        server: config::Server,
        accounts: Arc<Accounts>,
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
            accounts,
            pools: ladders.pools(),
            queues: Queues::new(ladders.queues()),
            queued: BTreeMap::new(),
            ranked: BTreeMap::new(),
            epoch: Instant::now(),
        }
    }

    /// Every offer a game may be of: those clients create games of, then the ladders' pools'.
    fn offers(&self) -> Vec<Arc<Offer>> {
        let pools = self
            .pools
            .values()
            .flat_map(|pool| pool.offers.iter().cloned());
        self.offers.iter().cloned().chain(pools).collect()
    }

    /// The queues' time: milliseconds since the registry began.
    fn now(&self) -> u64 {
        u64::try_from(self.epoch.elapsed().as_millis()).unwrap_or(u64::MAX)
    }

    /// Tells each connection what its queue told it, and whether it waits after it (a connection
    /// gone, or one that reads too slowly, misses it).
    fn tell(&self, told: Told<ConnId>) {
        for (conn, message) in told {
            if let Some(to) = self.queued.get(&conn) {
                let waits = self.queues.holds(conn);
                let _ = to.try_send((codec::encode_text(&message), waits));
            }
        }
    }

    /// The answer to `ListLadders`, as a frame's text (E16 M4 T2).
    pub(crate) fn ladders(&self, seq: u32) -> String {
        codec::encode_text(&self.queues.list(seq))
    }

    /// Connection `conn`, logged in as `login`, asks to wait in `ladder`'s queue, rated `rating`
    /// (whole points), and hears from it through `to` (E16 M4 T2): the answer's text, and whether
    /// the connection waits in a queue.
    pub(crate) fn queue(
        &mut self,
        conn: ConnId,
        (seq, ladder): (u32, &str),
        login: Option<&str>,
        (rating, to): (i32, ToConn),
    ) -> (String, bool) {
        let mine = |accounts: &[String; 2]| accounts.iter().any(|a| Some(a.as_str()) == login);
        let unfinished = (self.ranked.values()).any(|(l, accounts)| l == ladder && mine(accounts));
        let now = self.now();
        let answer = (self.queues).queue(conn, (seq, ladder), login, (rating, unfinished), now);
        self.queued.insert(conn, to);
        (codec::encode_text(&answer), self.queues.holds(conn))
    }

    /// Connection `conn` leaves its queue (`Dequeue`), or declines its match (`DeclineMatch`, if
    /// `decline`): the answer's text, and whether the connection waits in a queue.
    pub(crate) fn dequeue(&mut self, conn: ConnId, seq: u32, decline: bool) -> (String, bool) {
        let mut told = Told::new();
        let answer = if decline {
            self.queues.decline(conn, seq, &mut told)
        } else {
            self.queues.dequeue(conn, seq, &mut told)
        };
        self.tell(told);
        (codec::encode_text(&answer), self.queues.holds(conn))
    }

    /// Connection `conn` is gone: it leaves its queue, and the other player of its match waits
    /// again.
    pub(crate) fn closed(&mut self, conn: ConnId) {
        let mut told = Told::new();
        self.queues.closed(conn, &mut told);
        self.queued.remove(&conn);
        self.tell(told);
    }

    /// The matcher's turn (E16 M4 T2): unanswered matches given up, the waiting paired.
    pub(crate) fn tick_queues(&mut self) {
        let mut told = Told::new();
        self.queues.tick(self.now(), &mut told);
        self.tell(told);
    }

    /// Creates the game of a match both players accepted (E16 M4 T2): a scenario of its ladder's
    /// pool and the sides at random (OS randomness), both seats its players' accounts from the
    /// start. The game's id and each player's seat, or why not.
    fn create_match(
        &mut self,
        shared: &Shared,
        ladder: &str,
        players: [String; 2],
    ) -> Result<(u32, [Seat; 2]), UnqueueReason> {
        let pool = self.pools.get(ladder).cloned();
        let random = game::os_u64().ok();
        let (Some(pool), Some(random)) = (pool, random) else {
            return Err(UnqueueReason::Full);
        };
        let count = u64::try_from(pool.offers.len()).unwrap_or(u64::MAX);
        let index = usize::try_from(random % count.max(1)).unwrap_or_default();
        let Some(offer) = pool.offers.get(index).cloned() else {
            return Err(UnqueueReason::Full);
        };
        let [first, second] = players;
        let (accounts, seats) = if random >> 63 == 1 {
            ([second, first], [Seat::Defender, Seat::Attacker])
        } else {
            ([first, second], [Seat::Attacker, Seat::Defender])
        };
        let listed = pool.spectators && self.limits.spectators_per_game > 0;
        // The server's own: no client's address, and none of the limits per address.
        let origin = IpAddr::V6(Ipv6Addr::UNSPECIFIED);
        let game = (self.lobby)
            .create_ranked(origin, (&offer.id, ladder), listed)
            .map_err(|reason| match reason {
                Refusal::Draining => UnqueueReason::Draining,
                _ => UnqueueReason::Full,
            })?;
        let (server, limits) = (self.server.clone(), self.limits);
        let (shared_accounts, reserved) = (Arc::clone(&self.accounts), accounts.clone());
        let watchable = pool.spectators;
        let host = move || (server, limits, shared_accounts);
        let prepare = move || {
            let (server, limits, accounts) = host();
            match Prepared::new(offer, game, origin, (&server, &limits, accounts), watchable) {
                Ok(prepared) => Some(prepared.reserved(reserved.map(Some))),
                Err(e) => {
                    eprintln!("h3-server: game {game} not created: {e}");
                    None
                }
            }
        };
        let (to_game, inbox) = mpsc::channel(GAME_INBOX);
        if let Err(e) = game::launch(game, prepare, inbox, Arc::clone(shared)) {
            eprintln!("h3-server: game {game} not created: {e}");
            self.lobby.remove(game);
            return Err(UnqueueReason::Full);
        }
        let hosted = Hosted::Live {
            to: to_game,
            bound: 0,
        };
        self.games.insert(game, hosted);
        self.ranked.insert(game, (ladder.to_owned(), accounts));
        Ok((game, seats))
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
        codec::encode_text(&ServerMessage::Replays {
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
        // The queues empty, and match no one any more (E16 M4 T2).
        let mut told = Told::new();
        self.queues.drain(&mut told);
        self.tell(told);
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
        // A match that is over keeps its players from no other (E16 M4 T2); its result is rated
        // from E16 M4 T3b on.
        // stub: SRV-STUB-4
        if status.0 == Phase::Over {
            self.ranked.remove(&game);
        }
        self.lobby.update(game, status);
    }

    /// The seats of game `game` that belong to accounts, with their display names (E16 M3 T3).
    pub(crate) fn seat(&mut self, game: u32, seated: Vec<Seated>) {
        self.lobby.seat(game, seated);
    }

    /// Whether game `game`, in `phase`, leaves now: when it is not running and no connection is
    /// bound to it, unless it is a match's game that waits for its players (E16 M4 T2: they have
    /// not come yet). Then it is gone from the lobby, and its thread ends.
    pub(crate) fn close_if_idle(&mut self, game: u32, phase: Phase) -> bool {
        let idle = !matches!(self.games.get(&game), Some(Hosted::Live { bound, .. }) if *bound > 0);
        let match_waits = phase == Phase::Waiting && self.ranked.contains_key(&game);
        let closes = idle && phase != Phase::Running && !match_waits;
        if closes {
            self.leave(game);
        }
        closes
    }

    /// Lists a game a restart restored, as `origin` created it, and hosts it (E16 M2 T2, T3); a
    /// match's game, its seats `accounts`', keeps them from another match on its ladder (E16 M4
    /// T3a).
    pub(crate) fn reopen(
        &mut self,
        origin: IpAddr,
        info: GameInfo,
        hosted: Hosted,
        accounts: [Option<String>; 2],
    ) {
        let game = info.game;
        if let (Some(ladder), [Some(attacker), Some(defender)]) = (&info.ladder, accounts) {
            self.ranked
                .insert(game, (ladder.clone(), [attacker, defender]));
        }
        self.lobby.reopen(origin, info);
        self.games.insert(game, hosted);
    }

    /// Game `game` paused (E16 M2 T3): its thread ends; `tokens`, and `accounts` for their seats,
    /// resume it (E16 M3 T3).
    pub(crate) fn pause(
        &mut self,
        game: u32,
        (tokens, accounts): ([Option<String>; 2], [Option<String>; 2]),
    ) {
        self.lobby.update(game, (Phase::Paused, Vec::new(), false));
        let since = Instant::now();
        let paused = Hosted::Paused {
            tokens,
            accounts,
            since,
        };
        self.games.insert(game, paused);
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
        self.ranked.remove(&game);
    }
}

/// A connection's opening of a game it binds to (E16 M2 T5e): the connection, its outbound queue
/// for the game, the account it is logged in as (E16 M3 T3) and its join.
#[derive(Debug)]
pub(crate) struct Opening {
    /// The connection.
    pub conn: ConnId,
    /// Its outbound queue for the game.
    pub to: Outbox,
    /// The login name of the account it is logged in as.
    pub login: Option<String>,
    /// Its join.
    pub join: ClientMessage,
}

impl Opening {
    /// Queues the opening in a game's fresh inbox, which has room for it, before the game is
    /// hosted: it comes before anything another connection sends the game (E16 M2 T5e: the
    /// reviewer's finding; a stranger's join cannot take a new game's creator's seat, nor make a
    /// resuming game pause again).
    fn first(self, inbox: &mpsc::Sender<ToGame>) {
        let Self {
            conn,
            to,
            login,
            join,
        } = self;
        let _ = inbox.try_send(ToGame::Open { conn, to, login });
        let _ = inbox.try_send(ToGame::Message {
            conn,
            message: join,
        });
    }
}

/// Creates a game of `offer` for a client at `origin`, whose creator takes `seat` and chose
/// whether it takes spectators (`watchable`, E16 M2 T4c), and binds the creator (`conn`, its queue
/// `to`, its account's `login`) to it: the game's id and inbox, or why not. The creator's join is
/// the game's first message; the game draws its secrets and creates its journal on its own thread
/// ([`game::launch`]), so no file is written under the lock (E16 M2 T5e).
pub(crate) fn create(
    shared: &Shared,
    origin: IpAddr,
    (offer, seat, watchable): (&str, Seat, bool),
    (conn, to, login): (ConnId, Outbox, Option<String>),
) -> Result<(u32, mpsc::Sender<ToGame>), Refusal> {
    let mut registry = lock(shared);
    let listed = watchable && registry.limits.spectators_per_game > 0;
    let (game, index) = registry.lobby.create(origin, offer, seat, listed)?;
    let offer = Arc::clone(&registry.offers[index]);
    let (server, limits) = (registry.server.clone(), registry.limits);
    let accounts = Arc::clone(&registry.accounts);
    let prepare =
        move || match Prepared::new(offer, game, origin, (&server, &limits, accounts), watchable) {
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
    Opening {
        conn,
        to,
        login,
        join,
    }
    .first(&to_game);
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
/// game is restored first (E16 M2 T3), only for a seat's token, or for its account (E16 M3 T3): a
/// spectator, the bot's seat, a missing or another token, another account are refused as a
/// running game refuses them (`session::refusal`). It is restored on its own
/// thread ([`game::launch`]; E16 M2 T5e: no file read under the lock), the opening its first
/// message; a paused journal that no longer restores is void, the game gone, and its joiner hears
/// `Goodbye { Shutdown }`.
pub(crate) fn bind(
    shared: &Shared,
    (game, seat, token): (u32, Seat, Option<&str>),
    opening: Opening,
) -> Result<(mpsc::Sender<ToGame>, Option<Opening>), Refusal> {
    let mut registry = lock(shared);
    let (tokens, accounts) = match registry.games.get_mut(&game) {
        None => return Err(Refusal::NoSuchGame),
        Some(Hosted::Live { to, bound }) => {
            *bound += 1;
            return Ok((to.clone(), Some(opening)));
        }
        Some(Hosted::Paused {
            tokens, accounts, ..
        }) => (tokens, accounts),
    };
    let index = match seat {
        Seat::Attacker => 0,
        Seat::Defender => 1,
        Seat::Spectator => return Err(Refusal::NoSpectators),
    };
    // The bot's seat has no token; a paused game started, so every player's seat was taken.
    let Some(expected) = &tokens[index] else {
        return Err(Refusal::SeatTaken);
    };
    let claim = (expected.as_str(), true, accounts[index].as_deref());
    if let Some(reason) = session::refusal(claim, token, opening.login.as_deref()) {
        return Err(reason);
    }
    let path = journal::dir(&registry.server.data).join(format!("{game}.journal"));
    let (offers, server, limits) = (registry.offers(), registry.server.clone(), registry.limits);
    let accounts = Arc::clone(&registry.accounts);
    let prepare = move || {
        let prepared = journal::read(&path).and_then(|read| {
            let offer = offers
                .iter()
                .find(|o| o.id == read.start.offer)
                .cloned()
                .ok_or_else(|| format!("the server offers no `{}` now", read.start.offer))?;
            Prepared::restored(offer, &path, read, (&server, &limits, accounts), None)
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
            let offer = (registry.offers().into_iter())
                .find(|o| o.id == read.start.offer)
                .ok_or_else(|| format!("the server offers no `{}` now", read.start.offer))?;
            let accounts = Arc::clone(&registry.accounts);
            let host = (&registry.server, &registry.limits, accounts);
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

/// Sweeps the registry for abandoned games (E16 M2 T5a) and the ranked games' absent players (E16
/// M4 T3a: the ladders' `grace`), every quarter of its shortest deadline, at least every minute
/// and at most every 10 ms.
pub(crate) async fn sweep_every(shared: Shared, limits: config::Limits, grace: Vec<u64>) {
    let shortest = [limits.empty_game_ms, limits.paused_game_ms]
        .into_iter()
        .chain(grace)
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

/// Connection `conn` accepts its match (E16 M4 T2): the refusal's text without a match; nothing
/// while the other player has not accepted; once both did, the match's game is created and both
/// hear `MatchReady` (or `Unqueued` when the server has no room for it, or drains).
pub(crate) fn accept(shared: &Shared, conn: ConnId) -> Option<String> {
    let mut registry = lock(shared);
    let Matched { ladder, players } = match registry.queues.accept(conn) {
        Err(reason) => return Some(codec::encode_text(&ServerMessage::Refused { reason })),
        Ok(None) => return None,
        Ok(Some(matched)) => matched,
    };
    let [(a, first), (b, second)] = players;
    let told = match registry.create_match(shared, &ladder, [first, second]) {
        Ok((game, seats)) => [a, b]
            .into_iter()
            .zip(seats)
            .map(|(conn, seat)| {
                let ladder = ladder.clone();
                (conn, ServerMessage::MatchReady { ladder, game, seat })
            })
            .collect(),
        Err(reason) => [a, b]
            .map(|conn| (conn, ServerMessage::Unqueued { reason }))
            .to_vec(),
    };
    registry.tell(told);
    None
}

/// Runs the matcher every [`MATCH_EVERY`] (E16 M4 T2), until the task is aborted.
pub(crate) async fn match_every(shared: Shared) {
    let mut tick = tokio::time::interval(MATCH_EVERY);
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        tick.tick().await;
        lock(&shared).tick_queues();
    }
}
