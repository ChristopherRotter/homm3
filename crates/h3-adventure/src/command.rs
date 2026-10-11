//! The adventure context's commands, their errors and its events (ADR-0017 point 1: `h3-game`
//! checks turn and phase, then passes a player's command here with the player as actor; E08 T7b:
//! `MoveHero`). A rejected command changes nothing and emits nothing.

use alloc::vec::Vec;
use core::fmt;

use h3_contracts::HeroId;

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

/// What happened on the adventure map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdventureEvent {
    /// A hero made one step of a move (one event per step: ADR-0016 point 6 resolves a move's
    /// visibility per step, T8).
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
