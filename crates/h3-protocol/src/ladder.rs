//! The ladders' queues (E16 M4 T2; ADR-0019 points 5 and 8; `E16-design.md` §11), sans I/O: the
//! host passes the time, each account's rating and whether its ranked game on the ladder is over,
//! and sends what comes out. One per server.
//!
//! - **Waiting.** A logged-in account waits in one ladder's queue at a time (`Queue`), never while
//!   its ranked game on that ladder is not over (the host says so), and leaves it when it asks
//!   (`Dequeue`) or with its connection ([`Queues::closed`]).
//! - **The matcher** ([`Queues::tick`]). Each ladder's waiting players by rating, ties by the order
//!   they came in; neighbours are paired while their ratings are within the window of the shorter
//!   wait: [`WINDOW`] points plus [`WIDEN`] for every [`WIDEN_EVERY_MS`] waited. Both hear
//!   `MatchFound`.
//! - **The answers.** Both accept within the ladder's time: the host creates the game
//!   ([`Matched`]). One declines, does not accept in time or goes away: it leaves the queue, and
//!   the other waits again with its waiting time (`MatchCancelled`).
//! - **The bots** (E16 M4 T5). A ladder's bots wait at all times, outside its queue: a player the
//!   matcher left unpaired after [`Ladder::bot_after_ms`] plays the bot nearest in rating (ties by
//!   the configuration's order), whatever the window. The bot accepts at once; it plays any number
//!   of games at once, and a match given up leaves it as it was. A player who queued without bots
//!   (E16 M4 T5b, SRV-Q21) never plays one, however long it waits; `MatchFound` tells a player
//!   whether its opponent is a bot.
//!
//! Provisional (E16 M4 T2, T5, T5b): the window, the greedy pairing, the queues in memory only (a
//! restart empties them); a bot for whoever waited long enough, people first; the opt-out per
//! queue, not stored.

use alloc::string::String;
use alloc::vec::Vec;

use h3_kernel::order;

use crate::wire::{LadderInfo, Refusal, ServerMessage, UnqueueReason};

/// The rating points two players may be apart however briefly they waited.
pub const WINDOW: u32 = 100;
/// The points the window widens by for every [`WIDEN_EVERY_MS`] the shorter wait lasted.
pub const WIDEN: u32 = 50;
/// How often the window widens, in milliseconds.
pub const WIDEN_EVERY_MS: u64 = 30_000;

/// The window of two players whose shorter wait lasted `waited` milliseconds.
pub fn window(waited: u64) -> u32 {
    let steps = u32::try_from(waited / WIDEN_EVERY_MS).unwrap_or(u32::MAX);
    WINDOW.saturating_add(WIDEN.saturating_mul(steps))
}

/// A ladder as its queue knows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ladder {
    /// Its id, as `Queue` names it.
    pub id: String,
    /// Its name.
    pub name: String,
    /// Its current season.
    pub season: u32,
    /// How long both players have to accept a match, in milliseconds.
    pub accept_ms: u64,
    /// Its bots (E16 M4 T5), in the configuration's order.
    pub bots: Vec<LadderBot>,
    /// How long a player waits without a match before it plays a bot (E16 M4 T5), in
    /// milliseconds.
    pub bot_after_ms: u64,
}

/// One of the server's bots on a ladder (E16 M4 T5): an account that never logs in, whose seat
/// the server's bot plays.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LadderBot {
    /// The account's login name.
    pub login: String,
    /// Its display name.
    pub display: String,
    /// Its rating on the ladder, in whole points (the host keeps it current: [`Queues::rate`]).
    pub rating: i32,
}

/// A player waiting, or matched; or a bot, matched.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Waiting<C> {
    /// The player's connection; `None` for a bot (E16 M4 T5).
    conn: Option<C>,
    login: String,
    rating: i32,
    /// When it came into the queue (the host's milliseconds); a match given up keeps it.
    since: u64,
    /// Its place in the order players came in: unique.
    order: u64,
    /// Whether it may play a bot (E16 M4 T5b).
    bots: bool,
}

/// A match waiting for both players' answers.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Pending<C> {
    /// The ladder's index.
    ladder: usize,
    players: [Waiting<C>; 2],
    accepted: [bool; 2],
    /// Not accepted by both at this time, it is given up.
    due: u64,
}

/// A match both players accepted: the host creates its game (sides at random).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Matched<C> {
    /// The ladder's id.
    pub ladder: String,
    /// The players' connections and login names; a bot's connection is `None` (E16 M4 T5): the
    /// server's bot plays its seat.
    pub players: [(Option<C>, String); 2],
}

/// What a connection's message about the queues answers, and what others are told; `C` names a
/// connection.
pub type Told<C> = Vec<(C, ServerMessage)>;

/// The server's ladders and their queues.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Queues<C> {
    ladders: Vec<Ladder>,
    /// Each ladder's waiting players, in the order they came in.
    waiting: Vec<Vec<Waiting<C>>>,
    pending: Vec<Pending<C>>,
    /// The last place in the order handed out.
    order: u64,
    /// Whether the server drains (E16 M2 T5a): no one waits any more.
    draining: bool,
}

impl<C: Copy + Eq> Queues<C> {
    /// The queues of `ladders`, empty.
    pub fn new(ladders: Vec<Ladder>) -> Self {
        let waiting = ladders.iter().map(|_| Vec::new()).collect();
        Self {
            ladders,
            waiting,
            pending: Vec::new(),
            order: 0,
            draining: false,
        }
    }

    /// The answer to `ListLadders`.
    pub fn list(&self, seq: u32) -> ServerMessage {
        let ladders = (self.ladders.iter().zip(&self.waiting).enumerate())
            .map(|(index, (ladder, waiting))| {
                let matched = (self.pending.iter().filter(|p| p.ladder == index))
                    .flat_map(|p| &p.players)
                    .filter(|w| w.conn.is_some())
                    .count();
                let count = waiting.len().saturating_add(matched);
                LadderInfo {
                    id: ladder.id.clone(),
                    name: ladder.name.clone(),
                    season: ladder.season,
                    waiting: u32::try_from(count).unwrap_or(u32::MAX),
                    bots: ladder.bots.iter().map(|bot| bot.display.clone()).collect(),
                }
            })
            .collect();
        ServerMessage::Ladders { seq, ladders }
    }

    /// Connection `conn`, logged in as `login` (or not), asks to wait in the queue of `ladder`
    /// at `now`, rated `rating`, and whether it may play a bot (`bots`); `unfinished`: its ranked
    /// game on that ladder is not over. The answer: `Queued`, or why not.
    pub fn queue(
        &mut self,
        conn: C,
        (seq, ladder, bots): (u32, &str, bool),
        login: Option<&str>,
        (rating, unfinished): (i32, bool),
        now: u64,
    ) -> ServerMessage {
        match self.enter(conn, (ladder, bots), login, (rating, unfinished), now) {
            Ok(()) => ServerMessage::Queued {
                seq,
                ladder: String::from(ladder),
            },
            Err(reason) => ServerMessage::Refused { reason },
        }
    }

    /// Puts the account in the queue, or says why not.
    fn enter(
        &mut self,
        conn: C,
        (ladder, bots): (&str, bool),
        login: Option<&str>,
        (rating, unfinished): (i32, bool),
        now: u64,
    ) -> Result<(), Refusal> {
        let login = login.ok_or(Refusal::NotLoggedIn)?;
        if self.draining {
            return Err(Refusal::Draining);
        }
        let index =
            (self.ladders.iter().position(|l| l.id == ladder)).ok_or(Refusal::NoSuchLadder)?;
        if self.waits(login) {
            return Err(Refusal::AlreadyQueued);
        }
        if unfinished {
            return Err(Refusal::InRankedGame);
        }
        self.order = self.order.saturating_add(1);
        self.waiting[index].push(Waiting {
            conn: Some(conn),
            login: String::from(login),
            rating,
            since: now,
            order: self.order,
            bots,
        });
        Ok(())
    }

    /// Whether the account waits in a queue, matched or not.
    pub fn waits(&self, login: &str) -> bool {
        let pending = self.pending.iter().flat_map(|p| &p.players);
        (self.waiting.iter().flatten().chain(pending)).any(|w| w.login == login)
    }

    /// Whether connection `conn` waits in a queue, matched or not.
    pub fn holds(&self, conn: C) -> bool {
        let pending = self.pending.iter().flat_map(|p| &p.players);
        (self.waiting.iter().flatten().chain(pending)).any(|w| w.conn == Some(conn))
    }

    /// The bot `login` on `ladder` is rated `rating` now, in whole points (E16 M4 T5): the host
    /// tells it after each of its rated games, or before the matcher's turn.
    /// The same account may play on several ladders, rated on each.
    pub fn rate(&mut self, ladder: &str, login: &str, rating: i32) {
        let ladders = self.ladders.iter_mut().filter(|l| l.id == ladder);
        for bot in ladders
            .flat_map(|l| &mut l.bots)
            .filter(|bot| bot.login == login)
        {
            bot.rating = rating;
        }
    }

    /// Connection `conn` leaves its queue (`Dequeue`), or its match (as a decline): `Dequeued`,
    /// or `Refused { NotQueued }`. The other player of a match waits again (`MatchCancelled` to
    /// `told`).
    pub fn dequeue(&mut self, conn: C, seq: u32, told: &mut Told<C>) -> ServerMessage {
        if self.leave(conn, told) {
            ServerMessage::Dequeued { seq }
        } else {
            ServerMessage::Refused {
                reason: Refusal::NotQueued,
            }
        }
    }

    /// Connection `conn` declines its match (`DeclineMatch`): it leaves the queue (`Dequeued`),
    /// and the other waits again; `Refused { NoMatch }` without a match.
    pub fn decline(&mut self, conn: C, seq: u32, told: &mut Told<C>) -> ServerMessage {
        if self.matched(conn).is_none() {
            return ServerMessage::Refused {
                reason: Refusal::NoMatch,
            };
        }
        self.leave(conn, told);
        ServerMessage::Dequeued { seq }
    }

    /// Connection `conn` accepts its match (`AcceptMatch`): once both did, the match, for the
    /// host to create its game; nothing yet if the other has not; `NoMatch` without a match.
    pub fn accept(&mut self, conn: C) -> Result<Option<Matched<C>>, Refusal> {
        let at = self.matched(conn).ok_or(Refusal::NoMatch)?;
        let pending = &mut self.pending[at];
        for (player, accepted) in pending.players.iter().zip(&mut pending.accepted) {
            *accepted |= player.conn == Some(conn);
        }
        if pending.accepted != [true; 2] {
            return Ok(None);
        }
        let Pending {
            ladder, players, ..
        } = self.pending.remove(at);
        Ok(Some(Matched {
            ladder: self.ladders[ladder].id.clone(),
            players: players.map(|w| (w.conn, w.login)),
        }))
    }

    /// Connection `conn` is gone: out of its queue; the other player of its match waits again.
    pub fn closed(&mut self, conn: C, told: &mut Told<C>) {
        self.leave(conn, told);
    }

    /// The time is `now`: a match not accepted by both by its due time is given up (who did not
    /// accept leaves the queue, `Unqueued { Expired }`; who did waits again, `MatchCancelled`),
    /// then each ladder's waiting players are paired (`MatchFound` to both).
    pub fn tick(&mut self, now: u64, told: &mut Told<C>) {
        let (due, kept): (Vec<_>, Vec<_>) = core::mem::take(&mut self.pending)
            .into_iter()
            .partition(|p| p.due <= now);
        self.pending = kept;
        for pending in due {
            for (player, accepted) in pending.players.into_iter().zip(pending.accepted) {
                // A bot waits on, outside the queue (E16 M4 T5).
                let Some(conn) = player.conn else {
                    continue;
                };
                if accepted {
                    told.push((conn, ServerMessage::MatchCancelled {}));
                    self.requeue(pending.ladder, player);
                } else {
                    let reason = UnqueueReason::Expired;
                    told.push((conn, ServerMessage::Unqueued { reason }));
                }
            }
        }
        for index in 0..self.ladders.len() {
            self.pair(index, now, told);
        }
    }

    /// The server drains (E16 M2 T5a): every player waiting or matched leaves its queue
    /// (`Unqueued { Draining }`), and no one may wait any more.
    pub fn drain(&mut self, told: &mut Told<C>) {
        self.draining = true;
        let pending = core::mem::take(&mut self.pending);
        let waiting = self.waiting.iter_mut().flat_map(core::mem::take);
        let players = waiting.chain(pending.into_iter().flat_map(|p| p.players));
        for conn in players.filter_map(|player| player.conn) {
            let reason = UnqueueReason::Draining;
            told.push((conn, ServerMessage::Unqueued { reason }));
        }
    }

    /// The index of connection `conn`'s match.
    fn matched(&self, conn: C) -> Option<usize> {
        (self.pending.iter()).position(|p| p.players.iter().any(|w| w.conn == Some(conn)))
    }

    /// Takes connection `conn` out of its queue or its match, the other player of which waits
    /// again; whether it was in one.
    fn leave(&mut self, conn: C, told: &mut Told<C>) -> bool {
        for waiting in &mut self.waiting {
            if let Some(at) = waiting.iter().position(|w| w.conn == Some(conn)) {
                waiting.remove(at);
                return true;
            }
        }
        let Some(at) = self.matched(conn) else {
            return false;
        };
        let pending = self.pending.remove(at);
        for player in pending.players {
            // The other player waits again; a bot waits on, outside the queue (E16 M4 T5).
            if let Some(other) = player.conn.filter(|other| *other != conn) {
                told.push((other, ServerMessage::MatchCancelled {}));
                self.requeue(pending.ladder, player);
            }
        }
        true
    }

    /// A matched player waits again in ladder `index`'s queue, in its place by the order it came
    /// in, its waiting time kept.
    fn requeue(&mut self, index: usize, player: Waiting<C>) {
        let waiting = &mut self.waiting[index];
        let at = waiting.partition_point(|w| w.order < player.order);
        waiting.insert(at, player);
    }

    /// Pairs ladder `index`'s waiting players at `now`: by rating (ties by order), each with the
    /// next while their ratings are within the window of the shorter wait; then each player left
    /// who waited the ladder's `bot_after_ms`, and did not opt out (E16 M4 T5b), with its nearest
    /// bot (E16 M4 T5).
    fn pair(&mut self, index: usize, now: u64, told: &mut Told<C>) {
        let ladder = &self.ladders[index];
        let mut by_rating = core::mem::take(&mut self.waiting[index]);
        // A player's place in the order is unique: so is each key (ADR-0003 §9).
        let _ = order::sort_by_unique_key(&mut by_rating, |w| (w.rating, w.order));
        let mut left = Vec::new();
        let mut players = by_rating.into_iter().peekable();
        while let Some(first) = players.next() {
            let fits = |second: &Waiting<C>| {
                let waited = now.saturating_sub(first.since.max(second.since));
                first.rating.abs_diff(second.rating) <= window(waited)
            };
            let waited = now.saturating_sub(first.since) >= ladder.bot_after_ms;
            // Of equally near bots, the first (`min_by_key` keeps the first of equal keys).
            let bot = (ladder.bots.iter())
                .filter(|_| waited && first.bots)
                .min_by_key(|bot| bot.rating.abs_diff(first.rating));
            let (second, accepted) = match (players.next_if(fits), bot) {
                (Some(second), _) => (second, false),
                (None, Some(bot)) => {
                    let bot = Waiting {
                        conn: None,
                        login: bot.login.clone(),
                        rating: bot.rating,
                        since: now,
                        order: 0,
                        bots: true,
                    };
                    (bot, true)
                }
                (None, None) => {
                    left.push(first);
                    continue;
                }
            };
            // A bot is always the second (E16 M4 T5b): the first is told so.
            let found = ServerMessage::MatchFound {
                ladder: ladder.id.clone(),
                accept_ms: ladder.accept_ms,
                bot: second.conn.is_none(),
            };
            for conn in [first.conn, second.conn].into_iter().flatten() {
                told.push((conn, found.clone()));
            }
            self.pending.push(Pending {
                ladder: index,
                players: [first, second],
                accepted: [false, accepted],
                due: now.saturating_add(ladder.accept_ms),
            });
        }
        let _ = order::sort_by_unique_key(&mut left, |w| w.order);
        self.waiting[index] = left;
    }
}
