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
mod spell;

pub use ability::{
    Ability, AbilityDef, AreaShot, AuraTarget, CompiledAbility, MAX_RAISED_CREATURES,
};
pub use artifacts::{
    ArtifactClass, ArtifactSlot, ArtifactSlotId, ArtifactType, ArtifactTypeId, MAX_PARTS,
    MAX_SLOT_COUNT, MAX_SLOT_POSITIONS,
};
pub use effect::{
    Affects, CompiledHeroEffect, CreatureSelector, CreatureStat, FULL_BP, HeroEffect,
    HeroEffectDef, MAX_ALIGNMENT_FACTIONS, MAX_EFFECTS, MAX_SKILL_LEVEL, SpellGrant, SpellScope,
};
pub use error::CompileError;
pub use flags::{CreatureFlag, CreatureFlags};
pub use heroes::{
    HeroClass, HeroClassId, HeroParams, HeroType, HeroTypeId, MAX_HERO_SKILLS, MAX_SPELL_LEVEL,
    MAX_SPELL_SCHOOLS, MAX_SPELLS, MAX_SUMMON_SPELLS, SchoolId, SecondarySkill, SkillId, Spell,
    SpellCategory, SpellId, SpellPolarity, SpellSchool, StartingStack,
};
pub use key::{ContentKey, KeyError};
pub use pack::{
    ArtifactDef, ArtifactSlotDef, ArtifactSlotOverride, BattleParamsDef, ContentPack, CreatureDef,
    CreatureInfoDef, CreatureOverride, CreaturePatch, FactionDef, HeroClassDef, HeroOverride,
    HeroParamsDef, HeroPatch, HeroTypeDef, ObstacleDef, PackManifest, ProfileDef, ResourceDef,
    SecondarySkillDef, SpellDef, SpellSchoolDef, StartingStackDef, SurrenderDef, TerrainDef,
};
pub use quirks::{
    ATTACK_FROM_SEEN_PIT, CLONE_ON_ITS_ORIGINAL, COUNTERSTRIKE_EXTRA_ROUND,
    CREATURE_MIRROR_UNTESTED, DATA_QUIRK_PREFIX, DIAGONAL_CHECK_IGNORES_NATIVE,
    DIAGONAL_CHECK_IGNORES_ROADS, DRAWS_AVOID_HIDDEN, EARLY_MASS_VERSION, ELEMENTAL_LEVELS,
    FIRE_SHIELD_FIRST_STAGE, FIRST_PROTECTION, FRENZY_REDUCED_DEFENSE, FRINGE_SHOWS_OBJECTS,
    MASS_CURE_SKIPS_HYPNOTIZED, NATIVE_SKIPS_LEADING_NEUTRALS, NATIVES_SIGHT_AT_START,
    NO_LUCK_WITHOUT_HERO, OBSTACLE_ANCHORS, PERIODIC_FIRST_STACK, QuirkFlag, QuirkRegistry, Quirks,
    REGISTRY, ROLAND_CRUSADER_SPECIALTY, SACRIFICE_TESTS_AS_RESURRECTION, SINGLE_PRECISION_RATES,
    UNLIMITED_BALLISTA_SHOTS,
};
pub use ruleset::{
    BattleParams, CreatureInfo, CreatureType, CreatureTypeId, FactionId, FactionInfo, FieldParams,
    Formation, MAX_ARMY_SLOTS, MAX_AURA_VALUES, MAX_FIELD_HEXES, MAX_ROLL_LEVELS, MAX_SIDE_STACKS,
    MAX_UPGRADES, MAX_USUAL_OBSTACLE_HEXES, ObstacleType, RULESET_HASH_VERSION, ResourceId,
    ResourceInfo, RollTable, Ruleset, SurrenderParams, TerrainId, TerrainInfo, WAR_MACHINE_KINDS,
};
pub use spell::{
    EffectDuration, EffectEnd, HexStep, MASTERY_NAMES, MAX_AREA_RADIUS, MAX_CHAIN_TARGETS,
    MAX_MASTERY, MAX_OBSTACLE_HEXES, MAX_SPELL_EFFECTS, ObstacleKind, ObstacleShape,
    SPECIALIST_LEVELS, SpecialistBonus, SpellArea, SpellChain, SpellEffect, SpellMagic,
    SpellTarget, TargetConditions, TargetSide, TimedChange, Usefulness,
};
