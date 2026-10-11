//! The adventure environment (ADR-0017 point 8, E08 T9a; `docs/07-simulation-and-ai.md`): a game
//! on an adventure map as reinforcement learning drives it, through `h3-game` as a player does
//! (turns, days and each player's view), with the adventure lane's own action encoding. This
//! sub-path is the adventure lane's until the AI lane takes `h3-sim` (E05).
//!
//! * [`AdventureEnv`]: reset (a game setup, a seed), step, masks, fork, state hash, a player's
//!   observation (`h3_game`'s view as it is: a flat encoding waits for E13's macro-actions).
//! * [`action`]: the factorized action (an entry and a destination tile) and its masks, built from
//!   the reachable-tiles query (provisional until E13's macro-actions).
//! * [`rollout`]: the uniformly random legal agent.

pub mod action;
pub mod env;
pub mod rollout;

pub use action::{
    ACTION_ENCODING_VERSION, Action, END_TURN, ENTRY_COUNT, EntryMask, HERO_SLOTS, MOVE_BASE,
};
pub use env::{AdventureEnv, StepError, StepResult};
pub use rollout::{RolloutResult, random_rollout};

#[cfg(test)]
mod tests;
