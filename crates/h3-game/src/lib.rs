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
//! [`legal_commands`] and, per hero, [`legal_moves`] (T8b2a); each player's treasury with the
//! starting resources of the game's difficulty (T13a) and the gains of visits (T13b); pending
//! decisions, the Cartographer's offer, and their answers ([`Answer`], T14b); the border guard's
//! offer, whose acceptance checks that it still stands (T15a); battles (T19a); the heroes'
//! progression (T29) and the level-ups as decisions ([`Answer::First`], [`Answer::Second`],
//! [`default_answer`]; T30).
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
mod treasury;

#[cfg(test)]
mod tests;

pub use command::{Actor, Answer, GameCommand, GameCommandError, GameEvent, NoopSink, Sink, apply};
/// The hero changes a battle's outcome applies, which [`GameEvent::HeroChanged`] carries (E08
/// T19a2a; `h3_heroes`' types).
pub use h3_heroes::{ArtifactPlace, HeroChange};
/// A hero's movement levels from its details ([`GameState::hero`]): whether it flies or walks on
/// water (E08 T42, for the adventure's benchmarks; `h3_heroes`' derivation, HER-ADV-005).
pub use h3_heroes::{Movement, movement};
/// The worn artifacts' positions (E08 T49a, ADR-0013 point 1): a [`HeroArtifactsSetup`]'s are
/// `h3_heroes`' (the map's import equips a map's entries where they are taken, since E08 T49b
/// with Titan's Thunder's Spell Book; `wearing` places a list).
pub use h3_heroes::{Position, equip, positions, takes, wearing};
pub use legal::{default_answer, legal_commands, legal_moves};
pub use observe::{
    DecisionView, GameObservation, HeroInfo, LevelUpView, PlayerView, Viewer, observe, project,
};
pub use ruleset::{GameRuleset, RulesetMismatch};
pub use setup::{
    DEFAULT_DIFFICULTY, GameHeroSetup, GameSetup, GameSetupError, HeroArmy, HeroArtifactsSetup,
    HeroDetailsSetup, PlayerSetup,
};
pub use state::{GameState, OpenBattle};

/// Version of the game state's canonical hash encoding (`docs/08-work-process.md`, "Versions";
/// owned by the adventure lane). Bump it with any change to what the hash covers. v1 (E08 T6b):
/// the first encoding ([`GameState`]'s `CanonicalHash`). v2 (E08 T13a): each player's treasury.
/// v3 (E08 T14b): the pending decisions. v4 (E08 T15a): an offer's kind. v5 (E08 T18b): the
/// store of heroes. v6 (E08 T19a): the battles among the decisions. v7 (E08 T20b1): the
/// wandering monsters' offer kinds and whether a guard's visit waits for an offer's answer. v8
/// (E08 T21a): a guarded pile's offer to fight. v9 (E08 T27): a hero's offers' levels. v10 (E08
/// T29): the heroes' progression (the scenario's level limit and banned skills, the level seeds).
/// v11 (E08 T30): the level-ups among the decisions. v12 (E08 T49a): a hero's worn artifacts by
/// position.
pub const GAME_STATE_HASH_VERSION: u32 = 12;
