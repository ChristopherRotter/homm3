//! Content definitions, content-pack model and the pure `Ruleset` compiler (ADR-0006).
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
mod artifacts;
mod effect;
mod error;
mod flags;
mod heroes;
mod key;
mod pack;
mod quirks;
mod ruleset;

pub use ability::{Ability, AbilityDef, AreaShot, AuraTarget, CompiledAbility};
pub use artifacts::{
    ArtifactClass, ArtifactSlot, ArtifactSlotId, ArtifactType, ArtifactTypeId, MAX_PARTS,
    MAX_SLOT_COUNT, MAX_SLOT_POSITIONS,
};
pub use effect::{
    Affects, CompiledHeroEffect, CreatureSelector, CreatureStat, FULL_BP, HeroEffect,
    HeroEffectDef, MAX_ALIGNMENT_FACTIONS, MAX_EFFECTS, MAX_SKILL_LEVEL,
};
pub use error::CompileError;
pub use flags::{CreatureFlag, CreatureFlags};
pub use heroes::{
    HeroClass, HeroClassId, HeroParams, HeroType, HeroTypeId, MAX_HERO_SKILLS, MAX_SPELL_LEVEL,
    SchoolId, SecondarySkill, SkillId, Spell, SpellCategory, SpellId, SpellPolarity, SpellSchool,
    StartingStack,
};
pub use key::{ContentKey, KeyError};
pub use pack::{
    ArtifactDef, ArtifactSlotDef, ArtifactSlotOverride, BattleParamsDef, ContentPack, CreatureDef,
    CreatureInfoDef, CreatureOverride, CreaturePatch, FactionDef, HeroClassDef, HeroParamsDef,
    HeroTypeDef, ObstacleDef, PackManifest, ProfileDef, ResourceDef, SecondarySkillDef, SpellDef,
    SpellSchoolDef, StartingStackDef, SurrenderDef, TerrainDef,
};
pub use quirks::{
    NO_LUCK_WITHOUT_HERO, QuirkFlag, QuirkRegistry, Quirks, REGISTRY, SINGLE_PRECISION_RATES,
    UNLIMITED_BALLISTA_SHOTS,
};
pub use ruleset::{
    BattleParams, CreatureInfo, CreatureType, CreatureTypeId, FactionId, FactionInfo, FieldParams,
    Formation, MAX_ARMY_SLOTS, MAX_AURA_VALUES, MAX_FIELD_HEXES, MAX_ROLL_LEVELS, MAX_UPGRADES,
    ObstacleType, RULESET_HASH_VERSION, ResourceId, ResourceInfo, RollTable, Ruleset,
    SurrenderParams, TerrainId, TerrainInfo, WAR_MACHINE_KINDS,
};
