//! The adventure environment (ADR-0017 point 8, E08 T9a; `docs/07-simulation-and-ai.md`): a game
//! on an adventure map as reinforcement learning drives it, through `h3-game` as a player does
//! (turns, days and each player's view), with the adventure lane's own action encoding. This
//! sub-path is the adventure lane's until the AI lane takes `h3-sim` (E05).
//!
//! * [`AdventureEnv`]: built from a game setup and a seed with a day limit; reset (a seed), the
//!   seats to move ([`Seat`]: the players and, for a battle's unowned side, the neutral seat as
//!   the system actor, E08 T20b3a), step, masks, fork, state hash, a player's observation
//!   (`h3_game`'s view as it is: a flat encoding waits for E13's macro-actions) and the neutral
//!   seat's (its battle).
//! * [`action`]: the factorized action in two phases (encoding v3, E08 T19b1): on the map an
//!   entry and a destination tile, with masks built from the reachable-tiles query and the open
//!   decision's answers (provisional until E13's macro-actions); in a battle the battle lane's
//!   encoding for the side the player owns.
//! * [`rollout`]: the uniformly random legal agent, the battle's random agent in its battles and
//!   the neutral seat's.
//! * [`replay`]: the adventure replay model (T9b): the rules' identity, the map by content hash,
//!   the seed, the game commands with their actors (battle commands since format v3) and state
//!   hashes.

pub mod action;
pub mod env;
pub mod replay;
pub mod rollout;

pub use action::{
    ACCEPT, ACTION_ENCODING_VERSION, Action, DECLINE, END_TURN, ENTRY_COUNT, EntryMask, HERO_SLOTS,
    MOVE_BASE, MapAction, Phase, Seat,
};
pub use env::{AdventureEnv, StepError, StepResult};
pub use rollout::{RolloutResult, random_rollout};

#[cfg(test)]
mod tests;
