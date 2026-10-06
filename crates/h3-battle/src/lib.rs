//! Battle bounded context: battlefield, stacks, turn order, commands, damage, retaliation, outcome.
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
#[cfg(test)]
extern crate std;

mod combat;
mod command;
mod damage;
mod effects;
mod error;
mod event;
mod flee;
mod geometry;
mod hero;
mod legal;
mod magic;
mod morale;
mod movement;
mod observe;
mod state;
mod tactics;
mod terrain;
mod turn;
mod war;

pub use command::{Command, apply, validate};
pub use effects::{ActiveEffect, DamageRoll, EFFECT_CAPACITY, SpellValues};
pub use error::{CommandError, HeroError, SetupError};
pub use event::{Event, EventKind, NoopSink, Sink, StrikeFlags, Visibility};
pub use geometry::{ATTACK_SLOTS, Direction, Field, Footprint, Hex, HexSet};
pub use h3_contracts::{BattleOutcome, BattleResult, BattleSetup, BattleSide};
pub use hero::{HeroView, OwnHeroView};
pub use legal::{LegalCommands, MAX_LEGAL_COMMANDS, legal_commands};
pub use magic::{
    CastTarget, CastTargets, MAX_CAST_TARGETS, MAX_SPELL_BONUSES, SpellBonus, SpellBonuses,
    SpellSet, cast_targets, castable_spells,
};
pub use morale::{Bearer, Expiry, MODIFIER_CAPACITY, Modifiers, Stat};
pub use observe::{Observation, observe, project};
pub use state::{
    BATTLE_STATE_HASH_VERSION, BattleState, SIDE_STACK_SLOTS, STACK_CAPACITY, Stack, StackFlags,
    StackId, StackTraits, UNLIMITED_RETALIATIONS,
};
pub use tactics::TacticsPhase;
pub use terrain::generate_obstacles;

#[cfg(test)]
mod tests;
