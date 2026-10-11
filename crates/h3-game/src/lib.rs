//! The whole game (ADR-0017, E08), the first application crate: the game's setup and state,
//! players, turn groups and the calendar, an RNG stream per player, pending decisions, command
//! routing to the contexts and the projections across them, behind one entry point
//! (`apply`, `legal_commands`, `observe`, `project`). It depends on the domain contexts it
//! wires (`h3-adventure` from increment 1; `h3-heroes` and `h3-battle` from increment 3); no
//! domain crate depends on it (ADR-0015 point 1).
//!
//! So far: the game's rules ([`GameRuleset`]) and setup ([`GameSetup`]) with its validation
//! (E08 T6a); the state ([`GameState`]) with the starting armies' draw, the players' streams,
//! sequential turns, the calendar, and [`apply`] with `EndTurn` (T6b); the adventure's commands
//! (`MoveHero`, T7b); [`observe`] and [`project`] for a player [`Viewer`] (T8b1);
//! [`legal_commands`] (T8b2a).
//!
//! Logic crate: pure, deterministic, `no_std` + `alloc` (ADR-0003, ADR-0005). The lint header
//! below is required and checked by `cargo xtask check-arch`.
#![no_std]
#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::float_arithmetic,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::exit
)]
#![warn(missing_docs)]

extern crate alloc;

mod command;
mod legal;
mod observe;
mod ruleset;
mod setup;
mod state;

#[cfg(test)]
mod tests;

pub use command::{Actor, GameCommand, GameCommandError, GameEvent, NoopSink, Sink, apply};
pub use legal::legal_commands;
pub use observe::{GameObservation, PlayerView, Viewer, observe, project};
pub use ruleset::{GameRuleset, RulesetMismatch};
pub use setup::{GameHeroSetup, GameSetup, GameSetupError, HeroArmy, PlayerSetup};
pub use state::GameState;

/// Version of the game state's canonical hash encoding (`docs/08-work-process.md`, "Versions";
/// owned by the adventure lane). Bump it with any change to what the hash covers. v1 (E08 T6b):
/// the first encoding ([`GameState`]'s `CanonicalHash`).
pub const GAME_STATE_HASH_VERSION: u32 = 1;
