//! The adventure context's commands, their errors and its events (ADR-0017 point 1: `h3-game`
//! checks turn and phase, then passes a player's command here with the player as actor; E08 T7b:
//! `MoveHero`). A rejected command changes nothing and emits nothing. An event carries the players
//! who may see it, resolved when it is emitted (ADR-0016 point 6; E08 T8a).

use alloc::vec::Vec;
use core::fmt;

use h3_contracts::{HeroId, PlayerId};

use crate::setup::{Pos, StandError};

/// A command to the adventure context, issued by a player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdventureCommand {
    /// Moves one of the player's heroes along `path` today (ADV-MOV-003 to ADV-MOV-007): each
    /// tile a neighbour of the one before (the first of the hero's), each step paid from the
    /// points left. The path is validated step by step and rejected whole: a step the hero
    /// cannot pay is a rejection, not the original's stop that wastes the rest (survey §1.3).
    MoveHero {
        /// The hero.
        hero: HeroId,
        /// The tiles it steps onto, in order, without the one it stands on.
        path: Vec<Pos>,
    },
}

/// Why an adventure command was rejected; the state is unchanged. `step` counts a path's steps
/// from 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdventureCommandError {
    /// The hero is not on the map or not the player's (one error for both, so a command tells
    /// nothing about other players' heroes).
    NotYourHero(HeroId),
    /// The path has no step.
    EmptyPath,
    /// A step's tile is not one of the eight around the tile before it on its level
    /// (ADV-MOV-007).
    NotNeighbour {
        /// The step.
        step: usize,
    },
    /// A step's tile is on the map but the player has not explored it (ADV-MOV-008): checked
    /// before anything about the tile, so the error tells nothing about a hidden tile.
    Unexplored {
        /// The step.
        step: usize,
    },
    /// A step's tile may not be stood on (ADV-MOV-007, ADV-MAP-001).
    Blocked {
        /// The step.
        step: usize,
        /// Why.
        error: StandError,
    },
    /// A step's tile is one an object is visited from: a visit ends a move and comes with
    /// increment 2, so no step enters such a tile yet (ADR-0016 point 4).
    Visit {
        /// The step.
        step: usize,
    },
    /// A step costs more than the points left before it (ADV-MOV-006).
    TooFewPoints {
        /// The step.
        step: usize,
        /// Its price.
        cost: u32,
        /// The points left before it.
        points: u32,
    },
}

impl fmt::Display for AdventureCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotYourHero(hero) => write!(f, "hero {} is not the player's", hero.raw()),
            Self::EmptyPath => f.write_str("the path has no step"),
            Self::NotNeighbour { step } => {
                write!(f, "step {step} does not go to a neighbouring tile")
            }
            Self::Unexplored { step } => write!(f, "step {step} goes to an unexplored tile"),
            Self::Blocked { step, error } => write!(f, "step {step}: {error}"),
            Self::Visit { step } => {
                write!(f, "step {step} enters a tile an object is visited from")
            }
            Self::TooFewPoints { step, cost, points } => {
                write!(f, "step {step} costs {cost} with {points} points left")
            }
        }
    }
}

impl core::error::Error for AdventureCommandError {}

/// The players who may see an event (ADR-0016 point 6): a set of players, resolved when the event
/// is emitted with the state at hand, so projecting an event needs no state. A game has at most
/// eight players (the ruleset part's bound, [`crate::AdventureParams::max_players`]).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Viewers(u8);

impl Viewers {
    /// No player.
    pub const NONE: Self = Self(0);

    /// This set and `player` (a player id of eight or more is not a game's: it is left out).
    #[must_use]
    pub const fn with(self, player: PlayerId) -> Self {
        Self(self.0 | bit(player))
    }

    /// Whether `player` may see the event.
    pub const fn contains(self, player: PlayerId) -> bool {
        self.0 & bit(player) != 0
    }

    /// The set as bits, bit `n` for the player with id `n`.
    pub const fn bits(self) -> u8 {
        self.0
    }
}

/// The bit of `player` in [`Viewers`], none for an id of eight or more.
const fn bit(player: PlayerId) -> u8 {
    match 1u8.checked_shl(player.raw() as u32) {
        Some(bit) => bit,
        None => 0,
    }
}

/// Something that happened on the adventure map, with the players who may see it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdventureEvent {
    /// What happened.
    pub kind: AdventureEventKind,
    /// Who may see it.
    pub seen_by: Viewers,
}

/// What happened on the adventure map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdventureEventKind {
    /// A hero made one step of a move (one event per step: the fog changes during a move, so a
    /// viewer learns of the steps it can see, ADR-0016 point 6). Seen by the hero's owner and by
    /// every player that has explored the tile it leaves or the tile it enters (ADV-FOG-004).
    HeroStepped {
        /// The hero.
        hero: HeroId,
        /// The tile it left.
        from: Pos,
        /// The tile it entered.
        to: Pos,
    },
}

/// Receiver of the adventure context's events.
pub trait Sink {
    /// Receives one event.
    fn emit(&mut self, event: AdventureEvent);
}

impl Sink for Vec<AdventureEvent> {
    fn emit(&mut self, event: AdventureEvent) {
        self.push(event);
    }
}

#[cfg(test)]
mod tests;
