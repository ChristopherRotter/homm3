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

use crate::archive::Archive;
use crate::game::{self, Offer, Outbox, Prepared, Stores, ToGame};
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
    /// since when (E16 M2 T5a: its deadline), and an agreed pause's due time (E16 M4 T6: it wakes
    /// then, and has no deadline).
    Paused {
        tokens: [Option<String>; 2],
        accounts: [Option<String>; 2],
        since: Instant,
        due: Option<Instant>,
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
    /// The server's books (E16 M3 T3, M4 T3b): a game's thread names its seats' accounts and asks
    /// their preferences, records its id and rates its result, never under the lock.
    stores: Stores,
    /// The ladders' pools by ladder (E16 M4 T2): a match's game draws from them, and a restored
    /// or resumed game finds its offer among them too.
    pools: BTreeMap<String, Pool>,
    /// The ladders' queues (E16 M4 T2).
    queues: Queues<ConnId>,
    /// Where each connection that queued hears from its queue.
    queued: BTreeMap<ConnId, ToConn>,
    /// The matches' games that are not over, with their ladders and their seats' accounts: an
    /// account waits for no other match on the ladder meanwhile (ADR-0019 point 8). A game that
    /// stopped with its journal kept stays here until the next start ([`Registry::stopped`]).
    ranked: BTreeMap<u32, (String, [String; 2])>,
    /// The queues' clock starts here.
    epoch: Instant,
}

impl Registry {
    /// A registry of `offers` and of the ladders' queues within `limits`, and no games; its games'
    /// books are `stores` (their ids follow those handed out before, E16 M4 T3b).
    pub(crate) fn new(
        offers: Vec<Arc<Offer>>,
        limits: &config::Limits,
        server: config::Server,
        stores: Stores,
    ) -> Self {
        let infos = offers.iter().map(|o| o.info.clone()).collect();
        let games = lobby::Limits {
            games: limits.games,
            games_per_origin: limits.games_per_address,
        };
        let mut lobby = Lobby::new(infos, games);
        lobby.skip(stores.ids.last());
        Self {
            lobby,
            offers,
            games: BTreeMap::new(),
            server,
            limits: *limits,
            archive: Archive::default(),
            pools: stores.ladders.pools(),
            queues: Queues::new(stores.ladders.queues()),
            stores,
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
    /// (whole points), playing a bot or not (`bots`, E16 M4 T5b), and hears from it through `to`
    /// (E16 M4 T2): the answer's text, and whether the connection waits in a queue.
    pub(crate) fn queue(
        &mut self,
        conn: ConnId,
        asked: (u32, &str, bool),
        login: Option<&str>,
        (rating, to): (i32, ToConn),
    ) -> (String, bool) {
        let mine = |accounts: &[String; 2]| accounts.iter().any(|a| Some(a.as_str()) == login);
        let ladder = asked.1;
        let unfinished = (self.ranked.values()).any(|(l, accounts)| l == ladder && mine(accounts));
        let now = self.now();
        let answer = (self.queues).queue(conn, asked, login, (rating, unfinished), now);
        self.queued.insert(conn, to);
        (codec::encode_text(&answer), self.queues.holds(conn))
    }

    /// Connection `conn` leaves its queue (`Dequeue`), or declines its match (`DeclineMatch`, if
    /// `decline`): the answer's text, and whether the connection waits in a queue.
    pub(crate) fn dequeue(&mut self, conn: ConnId, seq: u32, decline: bool) -> (String, bool) {
        let (mut told, now) = (Told::new(), self.now());
        let answer = if decline {
            self.queues.decline(conn, (seq, now), &mut told)
        } else {
            self.queues.dequeue(conn, (seq, now), &mut told)
        };
        self.tell(told);
        (codec::encode_text(&answer), self.queues.holds(conn))
    }

    /// Connection `conn` is gone: it leaves its queue, and the other player of its match waits
    /// again.
    pub(crate) fn closed(&mut self, conn: ConnId) {
        let mut told = Told::new();
        let now = self.now();
        self.queues.closed(conn, now, &mut told);
        self.queued.remove(&conn);
        self.tell(told);
    }

    /// The matcher's turn (E16 M4 T2): unanswered matches given up, the waiting paired; with the
    /// ladders' bots rated as `bots` says (E16 M4 T5: ladder, login name, whole points).
    pub(crate) fn tick_queues(&mut self, bots: &[(String, String, i32)]) {
        for (ladder, login, rating) in bots {
            self.queues.rate(ladder, login, *rating);
        }
        let mut told = Told::new();
        self.queues.tick(self.now(), &mut told);
        self.tell(told);
    }

    /// Creates the game of a match both players accepted (E16 M4 T2): a scenario of its ladder's
    /// pool and the sides at random (OS randomness), both seats its players' accounts from the
    /// start; a bot's seat (`bots`, E16 M4 T5) its account's, played by the server's bot. The
    /// game's id and each player's seat, or why not.
    fn create_match(
        &mut self,
        shared: &Shared,
        ladder: &str,
        (players, bots): ([String; 2], [bool; 2]),
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
        let (accounts, seats, bots) = if random >> 63 == 1 {
            (
                [second, first],
                [Seat::Defender, Seat::Attacker],
                [bots[1], bots[0]],
            )
        } else {
            ([first, second], [Seat::Attacker, Seat::Defender], bots)
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
        let (stores, reserved) = (self.stores.clone(), accounts.clone());
        let watchable = pool.spectators;
        let host = move || (server, limits, stores);
        let prepare = move || {
            let (server, limits, stores) = host();
            let host = (&server, &limits, stores);
            match Prepared::new(offer, game, origin, host, (watchable, bots)) {
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
    /// `paused_game_ms` is void; a game whose agreed pause is over wakes instead (E16 M4 T6).
    pub(crate) fn sweep(&mut self, shared: &Shared, now: Instant) {
        self.tell_all(|| ToGame::Tick);
        let woken: Vec<u32> = (self.games.iter())
            .filter(|(_, hosted)| {
                matches!(hosted, Hosted::Paused { due: Some(due), .. } if now >= *due)
            })
            .map(|(&game, _)| game)
            .collect();
        for game in woken {
            self.wake(shared, game);
        }
        let Some(limit) = (self.limits.paused_game_ms > 0)
            .then(|| Duration::from_millis(self.limits.paused_game_ms))
        else {
            return;
        };
        let expired: Vec<u32> = self
            .games
            .iter()
            .filter(|(_, hosted)| {
                matches!(hosted, Hosted::Paused { since, due: None, .. }
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

    /// Restores the paused game `game` on its own thread, its agreed pause over (E16 M4 T6): it runs
    /// on with its players away until they come back or the grace ends it. A game whose thread does
    /// not start stays paused, for the next sweep.
    fn wake(&mut self, shared: &Shared, game: u32) {
        let prepare = self.restorer(game, true);
        let (to_game, inbox) = mpsc::channel(GAME_INBOX);
        if let Err(e) = game::launch(game, prepare, inbox, Arc::clone(shared)) {
            return eprintln!("h3-server: game {game} not woken: {e}");
        }
        let hosted = Hosted::Live {
            to: to_game,
            bound: 0,
        };
        self.games.insert(game, hosted);
    }

    /// What restores the paused game `game` from its journal on its own thread (E16 M2 T3; woken,
    /// E16 M4 T6: its agreed pause is over): a journal that no longer restores is void.
    fn restorer(
        &self,
        game: u32,
        wake: bool,
    ) -> impl FnOnce() -> Option<Prepared> + Send + 'static {
        let path = journal::dir(&self.server.data).join(format!("{game}.journal"));
        let (offers, server, limits) = (self.offers(), self.server.clone(), self.limits);
        let stores = self.stores.clone();
        move || {
            let prepared = journal::read(&path).and_then(|read| {
                let offer = offers
                    .iter()
                    .find(|o| o.id == read.start.offer)
                    .cloned()
                    .ok_or_else(|| format!("the server offers no `{}` now", read.start.offer))?;
                Prepared::restored(offer, &path, read, (&server, &limits, stores), None)
            });
            match prepared {
                Ok(prepared) if wake => Some(prepared.woken()),
                Ok(prepared) => Some(prepared),
                Err(why) => {
                    journal::void(&server.data, &path, &game.to_string(), &why);
                    None
                }
            }
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
        // A match that is over keeps its players from no other (E16 M4 T2); its thread rated its
        // result before (E16 M4 T3b).
        if status.0 == Phase::Over {
            self.ended(game);
        }
        self.lobby.update(game, status);
    }

    /// Game `game` is over, a match's rated (E16 M4 T3b): it keeps its players from no other
    /// match on its ladder. Its thread says so before its players hear the end (E16 M4 T7), so a
    /// player who queues as soon as it hears it is not refused `InRankedGame`.
    pub(crate) fn ended(&mut self, game: u32) {
        self.ranked.remove(&game);
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
    /// resume it (E16 M3 T3); an agreed pause of `ms` wakes it at its due time (E16 M4 T6).
    pub(crate) fn pause(
        &mut self,
        game: u32,
        (tokens, accounts): ([Option<String>; 2], [Option<String>; 2]),
        agreed: Option<u64>,
    ) {
        self.lobby.update(game, (Phase::Paused, Vec::new(), false));
        let since = Instant::now();
        let due = agreed.and_then(|ms| since.checked_add(Duration::from_millis(ms)));
        let paused = Hosted::Paused {
            tokens,
            accounts,
            since,
            due,
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

    /// Game `game` is gone, but its journal stays for the next start (its journal no longer
    /// writes, or its thread panicked): a match keeps its players from another on its ladder until
    /// then (ADR-0019 point 8; M4's review), its ladder and seats' accounts `ranked` when the
    /// registry does not hold them yet (a game restored at start).
    pub(crate) fn stopped(&mut self, game: u32, ranked: Option<(String, [String; 2])>) {
        let kept = self.ranked.remove(&game).or(ranked);
        self.leave(game);
        if let Some(kept) = kept {
            self.ranked.insert(game, kept);
        }
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
    let stores = registry.stores.clone();
    let prepare = move || {
        let bots = offer.bot_seats();
        match Prepared::new(
            offer,
            game,
            origin,
            (&server, &limits, stores),
            (watchable, bots),
        ) {
            Ok(prepared) => Some(prepared),
            Err(e) => {
                eprintln!("h3-server: game {game} not created: {e}");
                None
            }
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
    let prepare = registry.restorer(game, false);
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
/// not start stops the server's start. Every journal's game id counts as handed out (E16 M4 T3b:
/// a journal of a server from before `next-game`); one that does not record stops the start.
pub(crate) async fn restore(shared: &Shared) -> Result<(), Error> {
    let (data, ids) = {
        let registry = lock(shared);
        (
            registry.server.data.clone(),
            Arc::clone(&registry.stores.ids),
        )
    };
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
        let read = journal::read(&path);
        if let Ok(read) = &read {
            let recorded = ids.reach(read.start.game);
            recorded.map_err(|e| Error(format!("{}: the game's id: {e}", data.display())))?;
        }
        let prepared = read.and_then(|read| {
            let registry = lock(shared);
            let offer = (registry.offers().into_iter())
                .find(|o| o.id == read.start.offer)
                .ok_or_else(|| format!("the server offers no `{}` now", read.start.offer))?;
            let host = (&registry.server, &registry.limits, registry.stores.clone());
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
    lock(shared).lobby.skip(ids.last());
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
        lock(&shared).sweep(&shared, Instant::now());
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
    // A bot has no connection (E16 M4 T5): the server's bot plays its seat.
    let [(a, first), (b, second)] = players;
    let bots = [a.is_none(), b.is_none()];
    let told = match registry.create_match(shared, &ladder, ([first, second], bots)) {
        Ok((game, seats)) => [a, b]
            .into_iter()
            .zip(seats)
            .filter_map(|(conn, seat)| {
                let ladder = ladder.clone();
                Some((conn?, ServerMessage::MatchReady { ladder, game, seat }))
            })
            .collect(),
        Err(reason) => [a, b]
            .into_iter()
            .flatten()
            .map(|conn| (conn, ServerMessage::Unqueued { reason }))
            .collect(),
    };
    registry.tell(told);
    None
}

/// Runs the matcher every [`MATCH_EVERY`] (E16 M4 T2), until the task is aborted; the ladders'
/// bots rated as `ladders` holds them, read outside the registry's lock (E16 M4 T5) and off the
/// task (E16 M4 T7: the ladders' lock waits on a result's sync).
pub(crate) async fn match_every(shared: Shared, ladders: Arc<Ladders>) {
    let mut tick = tokio::time::interval(MATCH_EVERY);
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        tick.tick().await;
        let tables = Arc::clone(&ladders);
        let bots = tokio::task::spawn_blocking(move || tables.bots());
        let bots = bots.await.unwrap_or_default();
        lock(&shared).tick_queues(&bots);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::{Accounts, Ed25519};
    use crate::ids::GameIds;

    #[test]
    fn a_match_that_stopped_with_its_journal_kept_keeps_its_players() {
        // M4's review: a match whose journal no longer writes, or whose thread panicked, keeps its
        // players from another match on its ladder until the next start restores it.
        let data =
            std::env::temp_dir().join(format!("h3-registry-{:016x}", game::os_u64().unwrap()));
        let server = config::Server {
            data: data.clone(),
            ..config::Server::default()
        };
        let stores = Stores {
            accounts: Arc::new(Accounts::open(&data, Box::new(Ed25519)).unwrap()),
            ladders: Arc::new(Ladders::default()),
            ids: Arc::new(GameIds::open(&data).unwrap()),
        };
        let mut registry = Registry::new(vec![], &config::Limits::default(), server, stores);
        let entry = || {
            (
                String::from("arena"),
                [String::from("ann"), String::from("bob")],
            )
        };
        // Restored at start, its journal not writing: named here.
        registry.stopped(3, Some(entry()));
        assert_eq!(registry.ranked.get(&3), Some(&entry()));
        // Held already: kept.
        registry.stopped(3, None);
        assert_eq!(registry.ranked.get(&3), Some(&entry()));
        // Gone with its journal: its players are free.
        registry.leave(3);
        assert!(registry.ranked.is_empty());
        // Another game keeps nothing.
        registry.stopped(4, None);
        assert!(registry.ranked.is_empty());
        let _ = fs::remove_dir_all(&data);
    }
}
