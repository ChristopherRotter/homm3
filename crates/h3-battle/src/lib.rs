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

mod after_strike;
mod binds;
mod casters;
mod combat;
mod command;
mod control;
mod created;
mod damage;
mod effects;
mod error;
mod event;
mod flee;
mod geometry;
mod hero;
mod legal;
mod magic;
mod mana;
mod morale;
mod movement;
mod natural;
mod observe;
mod obstacles;
mod raise;
mod state;
mod tactics;
mod terrain;
mod turn;
mod vitality;
mod war;

pub use casters::{CastSpell, creature_cast_spell, creature_cast_targets, stack_spell};
pub use command::{Command, apply, validate};
pub use effects::{ActiveEffect, DamageRoll, EFFECT_CAPACITY, SpellValues, Status};
pub use error::{CommandError, HeroError, SetupError};
pub use event::{Event, EventKind, NoopSink, Sink, StrikeFlags, Visibility};
pub use geometry::{ATTACK_SLOTS, Direction, Field, Footprint, Hex, HexSet};
pub use h3_contracts::{BattleOutcome, BattleResult, BattleSetup, BattleSide};
pub use hero::{HeroView, OwnHeroView};
pub use legal::{LegalCommands, MAX_LEGAL_COMMANDS, legal_commands};
pub use magic::{
    CastTarget, CastTargets, MAX_CAST_TARGETS, MAX_OPENING_SPELLS, MAX_SPELL_BONUSES, Masteries,
    SpellBonus, SpellBonuses, SpellSet, cast_targets, castable_spells, second_targets,
};
pub use morale::{Bearer, Expiry, MODIFIER_CAPACITY, Modifiers, Stat};
pub use natural::{MAX_REMOVABLE_HEXES, MAX_REMOVABLE_OBSTACLES, RemovableObstacle};
pub use observe::{Observation, observe, project};
pub use obstacles::{OBSTACLE_CAPACITY, SpellObstacle};
pub use state::{
    BATTLE_STATE_HASH_VERSION, BattleState, CREATED_SLOT, NO_CORPSE, SIDE_STACK_SLOTS,
    STACK_CAPACITY, Stack, StackFlags, StackId, StackTraits, UNLIMITED_RETALIATIONS,
};
pub use tactics::TacticsPhase;
pub use terrain::generate_obstacles;

#[cfg(test)]
mod tests;
