//! The battle's ruleset part (R01; ADR-0006, ADR-0015 point 3): the battle rows of creatures and
//! spells, creature abilities (ADR-0012), what spells do in battle (ADR-0014), obstacle types and
//! the battle parameters, compiled into a [`BattleRuleset`] on a core `h3_content::Ruleset` from
//! one [`BattlePack`] per core pack.
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

mod ability;
mod error;
mod pack;
mod params;
mod ruleset;
mod spell;

pub use ability::{
    Ability, AbilityDef, AreaShot, AuraTarget, CompiledAbility, MAX_RAISED_CREATURES,
};
pub use error::BattleCompileError;
pub use pack::{
    BattleCreatureDef, BattleCreatureOverride, BattleCreaturePatch, BattlePack, BattleParamsDef,
    BattleSpellDef, ObstacleDef, SurrenderDef,
};
pub use params::{
    BattleParams, FieldParams, MAX_AURA_VALUES, MAX_FIELD_HEXES, MAX_ROLL_LEVELS, MAX_SIDE_STACKS,
    RollTable, SurrenderParams, WAR_MACHINE_KINDS,
};
pub use ruleset::{
    BATTLE_RULESET_HASH_VERSION, BattleRuleset, BattleSpell, CreatureType, MAX_SUMMON_SPELLS,
    MAX_USUAL_OBSTACLE_HEXES, ObstacleType,
};
pub use spell::{
    EffectDuration, EffectEnd, HexStep, MASTERY_NAMES, MAX_AREA_RADIUS, MAX_CHAIN_TARGETS,
    MAX_MASTERY, MAX_OBSTACLE_HEXES, MAX_SPELL_EFFECTS, ObstacleKind, ObstacleShape,
    SPECIALIST_LEVELS, SpecialistBonus, SpellArea, SpellChain, SpellEffect, SpellMagic,
    SpellPolarity, SpellTarget, TargetConditions, TargetSide, TimedChange, Usefulness,
};
