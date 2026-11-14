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
//!   own battle commands are rejected `BattleLimit`: the battle always ends.
//!
//! The system's commands for a player never count. Both counts follow from the journal's
//! commands (each counted as the engine accepts it), so a restore re-runs and counts them again
//! and nothing of them is journaled; the bounds are the hosting's (a restart's new `[limits]`
//! apply to a restored game). A rejection changes nothing and is not journaled.
//!
//! Provisional (the 2b review list): the defaults, the commands each count holds, a battle's
//! count by its identity, `GameTurn` leaving out what a bound refuses.

use alloc::vec;
use alloc::vec::Vec;

use h3_contracts::PlayerId;
use h3_game::{Actor, GameCommand, GameState};

use super::{BattleKey, own_battle, players};
use crate::wire::game::GameReject;

/// The host's bounds on a seat's commands (E16 M5 T18), the module's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandBounds {
    /// A player's map commands in its running turn; past it they are rejected `TurnLimit`.
    pub per_turn: u32,
    /// A player's commands in one of its battles; past it the computer plays its side.
    pub per_battle: u32,
}

impl CommandBounds {
    /// No bound: an in-process host's, a replay's (it vouches for its commands).
    pub const NONE: Self = Self {
        per_turn: u32::MAX,
        per_battle: u32::MAX,
    };

    /// The server's by default (provisional; measured 2026-11-14): the golden replays spend at
    /// most 26 map commands in a turn and 348 commands of a side in one battle
    /// (`tests/game_session.rs`), the XL fixtures' random play 16 and 361
    /// (`tests/game_bounds.rs`); both play through these bounds untouched. A turn's bound is far
    /// above, as a large map's late turns hold hundreds (eight heroes, dozens of towns' builds and
    /// hires), a battle's about ten times the most.
    pub const DEFAULT: Self = Self {
        per_turn: 1_000,
        per_battle: 4_000,
    };
}

/// What a player's command counts against: its turn's map commands, or its battle's (the
/// battle it commands, by its key before the command).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Counted {
    Map,
    Battle(BattleKey),
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
        }
    }

    /// What `actor`'s `command` counts against if the engine accepts it, in `state` before it: a
    /// player's map command its turn's, its battle command its battle's; nothing else counts.
    pub(super) fn of(
        state: &GameState,
        actor: Actor,
        command: &GameCommand,
    ) -> Option<(PlayerId, Counted)> {
        let Actor::Player(player) = actor else {
            return None;
        };
        let counted = match command {
            GameCommand::Adventure(_) | GameCommand::Manage(_) | GameCommand::Town(_) => {
                Counted::Map
            }
            GameCommand::Battle(_) => {
                Counted::Battle(own_battle(state, player).map(BattleKey::of)?)
            }
            GameCommand::EndTurn | GameCommand::Answer(_) | GameCommand::System(_) => return None,
        };
        Some((player, counted))
    }

    /// Follows the game after an accepted command (and its build): the command's count
    /// (`counted`, [`Counts::of`]'s before it) taken, then each player's turn counted afresh at
    /// each of its turns, and a battle's count dropped once that battle is no longer its open one.
    pub(super) fn sync(&mut self, state: &GameState, counted: Option<(PlayerId, Counted)>) {
        match counted {
            Some((player, Counted::Map)) => {
                let spent = &mut self.turns[player.index()].spent;
                *spent = spent.saturating_add(1);
            }
            Some((player, Counted::Battle(key))) => {
                let battle = &mut self.battles[player.index()];
                let spent = battle
                    .filter(|(held, _)| *held == key)
                    .map_or(0, |(_, n)| n);
                *battle = Some((key, spent.saturating_add(1)));
            }
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
    }

    /// The most commands a player spent now in its running turn, and in its open battle (the
    /// tests' measures of the defaults).
    #[cfg(test)]
    pub(super) fn most(&self) -> (u32, u32) {
        let turn = self.turns.iter().map(|t| t.spent).max().unwrap_or(0);
        let battle = (self.battles.iter().flatten()).map(|(_, n)| *n).max();
        (turn, battle.unwrap_or(0))
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
