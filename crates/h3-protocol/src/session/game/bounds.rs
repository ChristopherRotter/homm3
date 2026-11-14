//! A seat's commands bounded (E16 M5 T18; SRV-Q30's major finding, the user's answers of
//! 2026-11-14), sans I/O: the host's bounds ([`CommandBounds`], `h3-server`'s `[limits]`) on how
//! far one player's free commands may grow the game's journal while the others wait.
//!
//! - **A turn's map commands**: a player's accepted commands on the map (an adventure's, a
//!   management one, a town's) in its running turn, counted afresh at each of its turns (by its
//!   day, as its turn's timer: a player acts once a day). Past [`CommandBounds::per_turn`] they are
//!   rejected `TurnLimit`. Ending the turn, answering a decision and a battle's commands are not
//!   counted here and never refused by it: the player can always finish its turn.
//! - **A battle's commands**: a player's accepted battle commands in its open battle (known by
//!   its identity, [`BattleKey`]), counted afresh in each battle. Past
//!   [`CommandBounds::per_battle`] its side is the computer's for the rest of the battle, as a
//!   side whose chess clock ran out (the system's journaled steps, `GameSession::drive`), and its
//!   own battle commands are rejected `BattleLimit`.
//! - **The system's commands in a battle** (E16 M5 T20, SRV-Q35; 2b's reviewer's finding 3): the
//!   system actor's accepted battle commands in one battle, an unowned side's and those it plays
//!   for players (a side past its clock or its bound), counted afresh in each battle. A battle both
//!   of whose sides the system plays has no end the host can force (the engine has no round
//!   limit; Defend against Defend never ends): once they reach
//!   [`CommandBounds::system_per_battle`] with the battle still open, the game is void
//!   ([`Counts::void`]): nothing more is applied or driven, and the host ends it as a battle past
//!   its limit of commands (`h3-server`: `Goodbye { CommandLimit }`, its journal to the void).
//!
//! A seat's counts never hold the system's commands for its player. Every count follows from the
//! journal's commands (each counted as the engine accepts it), so a restore re-runs and counts
//! them again (a game void at its last command is void again) and nothing of them is journaled;
//! the bounds are the hosting's (a restart's new `[limits]` apply to a restored game). A rejection
//! changes nothing and is not journaled.
//!
//! Provisional (the 2b and 2c review lists): the defaults, the commands each count holds, a
//! battle's count by its identity, `GameTurn` leaving out what a bound refuses.

use alloc::vec;
use alloc::vec::Vec;

use h3_contracts::PlayerId;
use h3_game::{Actor, GameCommand, GameState, SystemCommand};

use super::{BattleKey, own_battle, players};
use crate::wire::game::GameReject;

/// The host's bounds on a seat's commands (E16 M5 T18), the module's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandBounds {
    /// A player's map commands in its running turn; past it they are rejected `TurnLimit`.
    pub per_turn: u32,
    /// A player's commands in one of its battles; past it the computer plays its side.
    pub per_battle: u32,
    /// The system's commands in one battle (E16 M5 T20): an unowned side's and those it plays for
    /// players; reaching it with the battle still open voids the game.
    pub system_per_battle: u32,
}

impl CommandBounds {
    /// No bound: an in-process host's, a replay's (it vouches for its commands).
    pub const NONE: Self = Self {
        per_turn: u32::MAX,
        per_battle: u32::MAX,
        system_per_battle: u32::MAX,
    };

    /// The server's by default (provisional; measured 2026-11-14): the golden replays spend at
    /// most 26 map commands in a turn and 348 commands of a side in one battle
    /// (`tests/game_session.rs`), the XL fixtures' random play 16 and 361
    /// (`tests/game_bounds.rs`); both play through these bounds untouched. A turn's bound is far
    /// above, as a large map's late turns hold hundreds (eight heroes, dozens of towns' builds and
    /// hires), a battle's about ten times the most. The system's in one battle (E16 M5 T20,
    /// measured 2026-11-14): at most 505 on the golden replays, 295 on the XL fixtures; its bound
    /// is a hosted battle's limit of commands, about twenty times the most and above a seat's bound
    /// with the computer's answers to it (which count), so that two sides the system plays still
    /// have about two thousand commands to end their battle.
    pub const DEFAULT: Self = Self {
        per_turn: 1_000,
        per_battle: 4_000,
        system_per_battle: 10_000,
    };
}

/// What an accepted command counts against: a player's turn's map commands or its battle's, or
/// the system's commands in a battle (each battle by its key before the command).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Counted {
    Map(PlayerId),
    Battle(PlayerId, BattleKey),
    System(BattleKey),
}

/// A player's map commands in a turn: the turn by its day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TurnCount {
    day: Option<u32>,
    spent: u32,
}

/// The players' counts (the module's).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Counts {
    bounds: CommandBounds,
    /// By player index.
    turns: Vec<TurnCount>,
    /// By player index: its open battle's key and its commands there.
    battles: Vec<Option<(BattleKey, u32)>>,
    /// The battle the system commanded last, while it is open, and its commands there.
    system: Option<(BattleKey, u32)>,
    /// Whether the system's commands reached their bound in a battle still open: the game is
    /// void.
    void: bool,
}

impl Counts {
    /// The counts of `players` players under `bounds`, none started.
    pub(super) fn new(bounds: CommandBounds, players: usize) -> Self {
        Self {
            bounds,
            turns: vec![
                TurnCount {
                    day: None,
                    spent: 0
                };
                players
            ],
            battles: vec![None; players],
            system: None,
            void: false,
        }
    }

    /// What `actor`'s `command` counts against if the engine accepts it, in `state` before it: a
    /// player's map command its turn's, its battle command its battle's, the system's battle
    /// command (an unowned side's, a player's side's) the system's in that battle; nothing else
    /// counts.
    pub(super) fn of(state: &GameState, actor: Actor, command: &GameCommand) -> Option<Counted> {
        let Actor::Player(player) = actor else {
            let battle = match command {
                GameCommand::Battle(_) => state.unowned_battle(),
                GameCommand::System(SystemCommand::Battle { player, .. }) => state.battle(*player),
                _ => None,
            };
            return battle.map(|battle| Counted::System(BattleKey::open(battle)));
        };
        Some(match command {
            GameCommand::Adventure(_) | GameCommand::Manage(_) | GameCommand::Town(_) => {
                Counted::Map(player)
            }
            GameCommand::Battle(_) => {
                Counted::Battle(player, own_battle(state, player).map(BattleKey::of)?)
            }
            GameCommand::EndTurn | GameCommand::Answer(_) | GameCommand::System(_) => return None,
        })
    }

    /// Follows the game after an accepted command (and its build): the command's count
    /// (`counted`, [`Counts::of`]'s before it) taken, then each player's turn counted afresh at
    /// each of its turns, and a battle's count dropped once that battle is no longer its open one;
    /// the game void when the system's count in a battle still open reached its bound.
    pub(super) fn sync(&mut self, state: &GameState, counted: Option<Counted>) {
        let add = |held: Option<(BattleKey, u32)>, key| {
            let spent = held.filter(|(k, _)| *k == key).map_or(0, |(_, n)| n);
            Some((key, spent.saturating_add(1)))
        };
        match counted {
            Some(Counted::Map(player)) => {
                let spent = &mut self.turns[player.index()].spent;
                *spent = spent.saturating_add(1);
            }
            Some(Counted::Battle(player, key)) => {
                let battle = &mut self.battles[player.index()];
                *battle = add(*battle, key);
            }
            Some(Counted::System(key)) => self.system = add(self.system, key),
            None => {}
        }
        let day = state.day();
        let counts = (self.turns.iter_mut()).zip(&mut self.battles);
        for (player, (turn, battle)) in players().zip(counts) {
            if state.is_acting(player) && turn.day != Some(day) {
                *turn = TurnCount {
                    day: Some(day),
                    spent: 0,
                };
            }
            let open = own_battle(state, player).map(BattleKey::of);
            if battle.is_some_and(|(key, _)| Some(key) != open) {
                *battle = None;
            }
        }
        // Every battle has a player's side: one no player holds is over.
        let players = players().take(self.battles.len());
        let mut open = players.filter_map(|player| own_battle(state, player).map(BattleKey::of));
        if let Some((key, spent)) = self.system {
            if open.any(|held| held == key) {
                self.void |= spent >= self.bounds.system_per_battle;
            } else {
                self.system = None;
            }
        }
    }

    /// Whether the game is void (E16 M5 T20): the system's commands in a battle reached their
    /// bound while it stayed open.
    pub(super) const fn void(&self) -> bool {
        self.void
    }

    /// The most commands a player spent now in its running turn and in its open battle, and the
    /// system's in the battle it commands (the tests' measures of the defaults).
    #[cfg(test)]
    pub(super) fn most(&self) -> (u32, u32, u32) {
        let turn = self.turns.iter().map(|t| t.spent).max().unwrap_or(0);
        let battle = (self.battles.iter().flatten()).map(|(_, n)| *n).max();
        let system = self.system.map_or(0, |(_, n)| n);
        (turn, battle.unwrap_or(0), system)
    }

    /// Whether `player`'s side in its open battle spent its bound: the computer's for the rest
    /// of the battle.
    pub(super) fn battle_out(&self, player: PlayerId) -> bool {
        let battle = self.battles.get(player.index()).copied().flatten();
        battle.is_some_and(|(_, spent)| spent >= self.bounds.per_battle)
    }

    /// Why a bound refuses `player`'s `command` in `state`, if one does; none for a player
    /// outside the game (the engine's to reject).
    pub(super) fn refuses(
        &self,
        state: &GameState,
        player: PlayerId,
        command: &GameCommand,
    ) -> Option<GameReject> {
        match command {
            GameCommand::Adventure(_) | GameCommand::Manage(_) | GameCommand::Town(_) => {
                // A turn's count starts afresh with it (`sync`).
                let full = self.turns.get(player.index())?.spent >= self.bounds.per_turn;
                (full && state.is_acting(player)).then_some(GameReject::TurnLimit {})
            }
            GameCommand::Battle(_) => self
                .battle_out(player)
                .then_some(GameReject::BattleLimit {}),
            GameCommand::EndTurn | GameCommand::Answer(_) | GameCommand::System(_) => None,
        }
    }
}
