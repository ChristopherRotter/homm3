//! The whole game's clock (E16 M5 T16; SRV-Q31, SRV-Q32), sans I/O: the host passes in its clock
//! ([`super::GameSession::elapse`]), the session never reads one. Its timers, in milliseconds of
//! the host's clock:
//!
//! - **A turn's timer** per player, the map offer's turn duration ([`Clock::turn_ms`], one of the
//!   original's eleven), started afresh at each of the player's turns. It runs while the game runs
//!   (started, not paused), the player's turn is running, no level-up holds the game and the
//!   player does not fight another player's side; a battle against a monster or a neutral
//!   garrison does not stop it. At its end the server ends the turn: each open decision answered by its default,
//!   a battle played out by the computer for the player's side, then the turn ended, every step
//!   the system actor's command (`h3_game::SystemCommand`), journaled.
//! - **The level-up's time** ([`Clock::level_up_ms`]): while a level-up holds the game
//!   (`h3_game::GameState::waiting_for`), every other timer stops and the owner's time runs,
//!   started afresh at each command the hold admits (each answers a level-up); at its end the
//!   system answers it by its default.
//! - **The chess clocks** of a battle between two players' sides (both sides owned: two heroes,
//!   or a hero and a player's garrison without one, E16 M5 T19), under a limited turn
//!   duration (the user, SRV-Q32: Fischer's): each side starts at [`Allowance::base_ms`] and
//!   gains [`Allowance::increment_ms`] after each of its accepted battle commands; only the side
//!   to act's runs; a side whose clock runs out is the computer's for the rest of the battle (no
//!   increment revives it). The attacker's turn timer stops meanwhile and resumes after the
//!   battle with what it had left (SRV-Q32).
//!
//! **The journal.** The time the running timers counted is an entry
//! ([`super::GameEntry::Elapsed`]), journaled before the next entry that may change what runs (a
//! command, a pause), at a timer's end, and at the latest after [`ELAPSED_EVERY_MS`]: a restore
//! re-runs it with the commands, so the running turn keeps its time left; the server's downtime
//! never counts (the host's clock restarts with the session, its first reading counting nothing),
//! and a crash gives the running timers back at most the time not journaled yet.
//!
//! Provisional (the 2b review list): the defaults, [`ELAPSED_EVERY_MS`], a timer told only to its
//! player, the increment after every battle command (a hero's cast too), a duel's clocks stopped
//! while a level-up holds the game, the level-up's time restarted at each admitted command.

use alloc::vec::Vec;

use h3_battle::BattleSide;
use h3_contracts::PlayerId;
use h3_game::{GameState, OpenBattle};

use super::{BattleKey, own_battle, players};
use crate::wire::game::{Duel, GameClock, Timer};

/// How a game's clock runs (E16 M5 T16), in milliseconds of the host's clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clock {
    /// A turn's duration, the map offer's (one of the original's eleven); `None`: unlimited, no
    /// turn timer and no chess clock (the game waits for its players).
    pub turn_ms: Option<u64>,
    /// The time a level-up's owner has while it holds the game, an offer's setting; `None`: the
    /// game waits for it.
    pub level_up_ms: Option<u64>,
    /// Each side's allowance in a battle between two players' sides, under a limited turn
    /// duration.
    pub battle: Allowance,
}

impl Clock {
    /// No timer at all: the game waits for its players (an in-process host's, a replay's).
    pub const UNLIMITED: Self = Self {
        turn_ms: None,
        level_up_ms: None,
        battle: Allowance::DEFAULT,
    };

    /// A level-up's time by default (SRV-Q31; kept after T15's survey, SRV-Q33: the original's
    /// dialog has the turn's time, 15 s in its last 15 s).
    pub const DEFAULT_LEVEL_UP_MS: u64 = 60_000;
}

/// A side's chess clock in a battle between two players' sides (SRV-Q32, Fischer's).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Allowance {
    /// The time each side starts the battle with.
    pub base_ms: u64,
    /// The time a side gains after each of its accepted battle commands.
    pub increment_ms: u64,
}

impl Allowance {
    /// The default: three minutes, ten seconds a command (SRV-Q32).
    pub const DEFAULT: Self = Self {
        base_ms: 180_000,
        increment_ms: 10_000,
    };
}

/// The most time the running timers count before it is journaled (provisional): a crash gives
/// them back at most this much.
pub const ELAPSED_EVERY_MS: u64 = 15_000;

/// A player's turn timer: the turn it times (by its day: a player acts once a day) and the time
/// it used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TurnTimer {
    day: Option<u32>,
    used: u64,
}

/// A battle's chess clocks, by side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DuelClocks {
    key: BattleKey,
    left: [u64; 2],
}

/// The game's timers (the module's).
#[derive(Debug, Clone)]
pub(super) struct Timers {
    clock: Clock,
    /// By player index.
    turns: Vec<TurnTimer>,
    /// The time the level-up the game waits for used.
    hold: u64,
    /// The open battles between two players' sides.
    duels: Vec<DuelClocks>,
    /// The host's clock the timers stand at: none before its first reading.
    pub(super) at: Option<u64>,
    /// The time counted since the last `Elapsed` entry.
    pub(super) pending: u64,
}

/// A timer that runs.
#[derive(Debug, Clone, Copy)]
enum Running {
    Turn(usize),
    Hold,
    Duel(usize, BattleSide),
}

impl Timers {
    /// The timers of `players` players under `clock`, none started.
    pub(super) fn new(clock: Clock, players: usize) -> Self {
        Self {
            clock,
            turns: alloc::vec![TurnTimer { day: None, used: 0 }; players],
            hold: 0,
            duels: Vec::new(),
            at: None,
            pending: 0,
        }
    }

    /// Follows the game after a change (its build, an accepted command): a turn started afresh
    /// for each player whose turn began, the level-up's time afresh (each command the hold
    /// admits answers a level-up), a duel's clocks for each battle between players opened, closed ones
    /// dropped, and the increment of the side `commanded` (the side of a battle command, before
    /// it) unless its clock ran out.
    pub(super) fn sync(&mut self, state: &GameState, commanded: Option<(BattleKey, BattleSide)>) {
        let day = state.day();
        for (player, timer) in players().zip(&mut self.turns) {
            if state.is_acting(player) && timer.day != Some(day) {
                *timer = TurnTimer {
                    day: Some(day),
                    used: 0,
                };
            }
        }
        // After a command the hold admitted, a new level-up's (or none).
        self.hold = 0;
        if self.clock.turn_ms.is_none() {
            return;
        }
        let open = duels(state);
        self.duels
            .retain(|d| open.iter().any(|(key, _)| *key == d.key));
        for (key, _) in open {
            if !self.duels.iter().any(|d| d.key == key) {
                let base = self.clock.battle.base_ms;
                self.duels.push(DuelClocks {
                    key,
                    left: [base; 2],
                });
            }
        }
        if let Some((key, side)) = commanded
            && let Some(duel) = self.duels.iter_mut().find(|d| d.key == key)
            && duel.left[side.index()] > 0
        {
            let left = &mut duel.left[side.index()];
            *left = left.saturating_add(self.clock.battle.increment_ms);
        }
    }

    /// The timers that run in `state` while the game runs, each with its time left (more than
    /// none).
    fn running(&self, state: &GameState) -> Vec<(Running, u64)> {
        let mut running = Vec::new();
        if state.waiting_for().is_some() {
            if let Some(left) = self.hold_left().filter(|left| *left > 0) {
                running.push((Running::Hold, left));
            }
            return running;
        }
        for (index, player) in players().take(self.turns.len()).enumerate() {
            if let Some(left) = self.turn_left(state, player)
                && left > 0
                && !in_duel(state, player)
            {
                running.push((Running::Turn(index), left));
            }
        }
        let duels = self.duels.iter().enumerate().filter_map(|(index, duel)| {
            let side = battle(state, duel.key).and_then(|b| b.state().side_to_act());
            side.map(|side| (Running::Duel(index, side), duel.left[side.index()]))
        });
        running.extend(duels.filter(|(_, left)| *left > 0));
        running
    }

    /// The time to the next end of a timer that runs in `state` while the game runs; none if
    /// none runs.
    pub(super) fn next_end(&self, state: &GameState) -> Option<u64> {
        self.running(state).into_iter().map(|(_, left)| left).min()
    }

    /// The timers that run in `state` count `ms` (the game runs).
    pub(super) fn advance(&mut self, ms: u64, state: &GameState) {
        for (timer, _) in self.running(state) {
            match timer {
                Running::Turn(index) => {
                    let used = &mut self.turns[index].used;
                    *used = used.saturating_add(ms);
                }
                Running::Hold => self.hold = self.hold.saturating_add(ms),
                Running::Duel(index, side) => {
                    let left = &mut self.duels[index].left[side.index()];
                    *left = left.saturating_sub(ms);
                }
            }
        }
    }

    /// `player`'s turn's time left, while its turn runs under a limited duration.
    fn turn_left(&self, state: &GameState, player: PlayerId) -> Option<u64> {
        let limit = self.clock.turn_ms?;
        // A player of the game.
        let timer = self.turns[player.index()];
        (state.is_acting(player) && timer.day == Some(state.day()))
            .then(|| limit.saturating_sub(timer.used))
    }

    /// The level-up's time left, if the game limits it (meaningful while it waits for one).
    fn hold_left(&self) -> Option<u64> {
        let limit = self.clock.level_up_ms?;
        Some(limit.saturating_sub(self.hold))
    }

    /// Whether `player`'s turn ran out while it runs: the system ends it.
    pub(super) fn turn_out(&self, state: &GameState, player: PlayerId) -> bool {
        self.turn_left(state, player) == Some(0)
    }

    /// Whether the level-up's time ran out: the system answers it.
    pub(super) fn hold_out(&self) -> bool {
        self.hold_left() == Some(0)
    }

    /// Whether the side of `battle` ran out of its chess clock: the computer's for the rest of
    /// the battle.
    pub(super) fn side_out(&self, battle: &OpenBattle, side: BattleSide) -> bool {
        let key = BattleKey::of((battle, side));
        (self.duels.iter()).any(|d| d.key == key && d.left[side.index()] == 0)
    }

    /// What `player` may see of the clock in `state`, its running timers shown running if the
    /// game runs (`live`).
    pub(super) fn view(&self, state: &GameState, player: PlayerId, live: bool) -> GameClock {
        let waiting = state.waiting_for();
        let timer = |left: u64, running: bool| Timer {
            left_ms: u32::try_from(left).unwrap_or(u32::MAX),
            running: live && running && left > 0,
        };
        let turn = self.turn_left(state, player).map(|left| {
            let runs = waiting.is_none() && !in_duel(state, player);
            timer(left, runs)
        });
        let battle = own_battle(state, player).and_then(|own| {
            let key = BattleKey::of(own);
            let duel = self.duels.iter().find(|d| d.key == key)?;
            let to_act = own.0.state().side_to_act();
            let side = |side: BattleSide| {
                let runs = waiting.is_none() && to_act == Some(side);
                timer(duel.left[side.index()], runs)
            };
            Some(Duel {
                attacker: side(BattleSide::Attacker),
                defender: side(BattleSide::Defender),
            })
        });
        let level_up = (waiting == Some(player))
            .then(|| self.hold_left().map(|left| timer(left, true)))
            .flatten();
        GameClock {
            turn,
            battle,
            level_up,
            waits: waiting.is_some_and(|owner| owner != player),
        }
    }
}

/// Whether the clock `new` at `now` is `old` as told at `then`: the same timers, each running or
/// stopped alike, a running one ending at the same time, a stopped one with the same time left.
pub(super) fn same(old: &GameClock, then: u64, new: &GameClock, now: u64) -> bool {
    let timer = |old: Option<Timer>, new: Option<Timer>| match (old, new) {
        (None, None) => true,
        (Some(old), Some(new)) => {
            old.running == new.running
                && if new.running {
                    then.saturating_add(old.left_ms.into())
                        == now.saturating_add(new.left_ms.into())
                } else {
                    old.left_ms == new.left_ms
                }
        }
        _ => false,
    };
    let duel = |old: Option<Duel>| old.map(|d| [Some(d.attacker), Some(d.defender)]);
    let (old_duel, new_duel) = (duel(old.battle), duel(new.battle));
    old.waits == new.waits
        && timer(old.turn, new.turn)
        && timer(old.level_up, new.level_up)
        && old_duel.is_some() == new_duel.is_some()
        && (old_duel.unwrap_or_default().into_iter())
            .zip(new_duel.unwrap_or_default())
            .all(|(old, new)| timer(old, new))
}

/// The open battles between two players' sides, each by its key, in the players' order.
fn duels(state: &GameState) -> Vec<(BattleKey, &OpenBattle)> {
    let mut duels: Vec<(BattleKey, &OpenBattle)> = Vec::new();
    for player in players().take(state.players()) {
        if let Some(own) = own_battle(state, player).filter(|(b, _)| is_duel(b)) {
            let key = BattleKey::of(own);
            if !duels.iter().any(|(k, _)| *k == key) {
                duels.push((key, own.0));
            }
        }
    }
    duels
}

/// The open battle known by `key`.
fn battle(state: &GameState, key: BattleKey) -> Option<&OpenBattle> {
    duels(state)
        .into_iter()
        .find_map(|(k, battle)| (k == key).then_some(battle))
}

/// Whether `battle` is between two players' sides.
fn is_duel(battle: &OpenBattle) -> bool {
    battle.owners().iter().all(Option::is_some)
}

/// Whether `player`'s battle is against another player's side.
fn in_duel(state: &GameState, player: PlayerId) -> bool {
    state.battle(player).is_some_and(is_duel)
}
