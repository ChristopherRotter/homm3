//! Adapter: parses the battle part of human-editable content packs (`battle.toml`, pack format
//! v36: v35, R01, and the special battlefields of E04c T2b) and the battle scenarios (TOML) into the battle's definitions and battle setups
//! (ADR-0006). A pack's `battle.toml` has no manifest: it belongs to the core pack of its folder
//! (`core.toml`, parsed by `h3-pack`), as `adventure.toml` does. Pack text is untrusted input:
//! parsing is strict (unknown fields are errors, numbers must fit their types) and every
//! definition is validated again by the compilers.
//!
//! Formats: `packs/README.md`; examples: `packs/synthetic/`.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use h3_content::{
    CompileError, ContentKey, CreatureFlag, CreatureFlags, KeyError, PackManifest, Ruleset,
};
use h3_content_battle::{
    AbilityDef, AreaShot, AuraTarget, BallisticsRecord, BattleCompileError, BattleCreatureDef,
    BattleCreatureOverride, BattleCreaturePatch, BattlePack, BattleParamsDef, BattleRuleset,
    BattleSpellDef, BattlefieldDef, EffectDuration, EffectEnd, FactionChangeDef, HexStep,
    MASTERY_NAMES, ObstacleDef, ObstacleKind, ObstacleShape, SPECIALIST_LEVELS, SiegeDef,
    SiegeFieldDef, SpecialistBonus, SpellArea, SpellChain, SpellEffect, SpellMagic, SpellPolarity,
    SpellTarget, SurrenderDef, TargetConditions, TargetSide, TimedChange, Usefulness, WallKind,
    WallTarget,
};
use h3_contracts::BattleSetup;
use h3_pack::{PackError, reference, references};
use serde::Deserialize;

mod scenario;

pub use scenario::*;

/// The synthetic pack's battle file (tests, benchmarks, FFI smoke, replays).
pub const SYNTHETIC_BATTLE: &str = include_str!("../../../packs/synthetic/battle.toml");
/// The synthetic expansion pack's battle file.
pub const SYNTHETIC_EXT_BATTLE: &str = include_str!("../../../packs/synthetic-ext/battle.toml");
/// The harvested pack `base`'s battle file (The Restoration of Erathia).
pub const BASE_BATTLE: &str = include_str!("../../../packs/base/battle.toml");
/// The harvested pack `ab`'s battle file (Armageddon's Blade).
pub const AB_BATTLE: &str = include_str!("../../../packs/ab/battle.toml");
/// The harvested pack `sod`'s battle file (The Shadow of Death).
pub const SOD_BATTLE: &str = include_str!("../../../packs/sod/battle.toml");
/// The synthetic pack as `(core.toml, battle.toml)` texts.
pub const SYNTHETIC_PACK: (&str, &str) = (h3_pack::SYNTHETIC_CORE, SYNTHETIC_BATTLE);
/// The synthetic expansion pack (depends on the synthetic pack) as `(core.toml, battle.toml)`.
pub const SYNTHETIC_EXT_PACK: (&str, &str) = (h3_pack::SYNTHETIC_EXT_CORE, SYNTHETIC_EXT_BATTLE);
/// Harvested pack `base` (RoE, latest patch; Q13, Q15) as `(core.toml, battle.toml)`.
pub const BASE_PACK: (&str, &str) = (h3_pack::BASE_CORE, BASE_BATTLE);
/// Harvested pack `ab` (depends on `base`) as `(core.toml, battle.toml)`.
pub const AB_PACK: (&str, &str) = (h3_pack::AB_CORE, AB_BATTLE);
/// Harvested pack `sod` (depends on `ab`) as `(core.toml, battle.toml)`.
pub const SOD_PACK: (&str, &str) = (h3_pack::SOD_CORE, SOD_BATTLE);
/// Synthetic scenario: three stacks per side.
pub const SYNTHETIC_SKIRMISH: &str =
    include_str!("../../../packs/synthetic/scenarios/skirmish.toml");
/// Synthetic scenario: one stack per side.
pub const SYNTHETIC_DUEL: &str = include_str!("../../../packs/synthetic/scenarios/duel.toml");
/// Synthetic scenario: seven stacks per side.
pub const SYNTHETIC_FULL: &str = include_str!("../../../packs/synthetic/scenarios/full.toml");
/// Synthetic scenario: the skirmish's armies at a town (E04c T5).
pub const SYNTHETIC_SIEGE: &str = include_str!("../../../packs/synthetic/scenarios/siege.toml");

/// Errors of this adapter. The core's errors keep their variants (`Syntax`, `Key`, `Compile`),
/// so a pack error reads the same whichever of its files it is in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BattlePackError {
    /// The text is not valid TOML for the expected schema.
    Syntax(String),
    /// A key or id is malformed.
    Key(KeyError),
    /// The packs do not compile into the core ruleset (or the battle part with an error the core
    /// also knows).
    Compile(CompileError),
    /// The battle packs do not compile into the battle part of the ruleset (R01).
    Battle(BattleCompileError),
    /// A scenario references a creature the ruleset does not define.
    UnknownCreature(String),
    /// A scenario references a terrain the ruleset does not define.
    UnknownTerrain(String),
    /// A scenario or replay references a faction the ruleset does not define.
    UnknownFaction(String),
    /// A scenario or replay references a hero type the ruleset does not define.
    UnknownHeroType(String),
    /// A scenario or replay references a spell the ruleset does not define.
    UnknownSpell(String),
    /// A scenario or replay references a spell school the ruleset does not define.
    UnknownSchool(String),
}

impl fmt::Display for BattlePackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax(e) => write!(f, "syntax: {e}"),
            Self::Key(e) => write!(f, "{e}"),
            Self::Compile(e) => write!(f, "{e}"),
            Self::Battle(e) => write!(f, "{e}"),
            Self::UnknownCreature(k) => write!(f, "unknown creature `{k}`"),
            Self::UnknownTerrain(k) => write!(f, "unknown terrain `{k}`"),
            Self::UnknownFaction(k) => write!(f, "unknown faction `{k}`"),
            Self::UnknownHeroType(k) => write!(f, "unknown hero type `{k}`"),
            Self::UnknownSpell(k) => write!(f, "unknown spell `{k}`"),
            Self::UnknownSchool(k) => write!(f, "unknown spell school `{k}`"),
        }
    }
}

impl std::error::Error for BattlePackError {}

impl From<PackError> for BattlePackError {
    fn from(e: PackError) -> Self {
        match e {
            PackError::Syntax(e) => Self::Syntax(e),
            PackError::Key(e) => Self::Key(e),
            PackError::Compile(e) => Self::Compile(e),
        }
    }
}

impl From<KeyError> for BattlePackError {
    fn from(e: KeyError) -> Self {
        Self::Key(e)
    }
}

impl From<CompileError> for BattlePackError {
    fn from(e: CompileError) -> Self {
        Self::Compile(e)
    }
}

impl From<BattleCompileError> for BattlePackError {
    fn from(e: BattleCompileError) -> Self {
        match e {
            BattleCompileError::Content(e) => Self::Compile(e),
            e => Self::Battle(e),
        }
    }
}

/// `battle.toml`: no manifest (the folder's `core.toml` has it).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BattleFile {
    battle: Option<BattleDto>,
    #[serde(default)]
    battlefield: Vec<BattlefieldDto>,
    /// Pack format v37 (E04c T5).
    #[serde(default)]
    siege: Vec<SiegeDto>,
    #[serde(default)]
    obstacle: Vec<ObstacleDto>,
    #[serde(default)]
    creature: Vec<CreatureDto>,
    #[serde(default)]
    creature_override: Vec<OverrideDto>,
    #[serde(default)]
    spell: Vec<SpellDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObstacleDto {
    id: String,
    #[serde(default)]
    terrains: Vec<String>,
    /// Pack format v36 (E04c T2b).
    #[serde(default)]
    battlefields: Vec<String>,
    #[serde(default)]
    width: u8,
    #[serde(default)]
    height: u8,
    blocked: Vec<[i8; 2]>,
    #[serde(default)]
    absolute: bool,
}

/// `[[battlefield]]` (pack format v36, E04c T2b): a special battlefield's rules
/// (`h3_content_battle::BattlefieldDef`).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BattlefieldDto {
    id: String,
    #[serde(default)]
    own_obstacles: bool,
    #[serde(default)]
    blocked: Vec<u8>,
    #[serde(default)]
    morale: Vec<FactionChangeDto>,
    #[serde(default)]
    luck: Vec<FactionChangeDto>,
    #[serde(default)]
    no_morale: bool,
    #[serde(default)]
    no_luck: bool,
    #[serde(default)]
    no_native: bool,
    max_spell_level: Option<u8>,
    #[serde(default)]
    no_creature_casts: bool,
    #[serde(default)]
    expert_schools: Vec<String>,
    #[serde(default)]
    expert_other_casts: bool,
}

/// `[[siege]]` (pack format v37, E04c T5): a town faction's siege (`h3_content_battle::SiegeDef`).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SiegeDto {
    faction: String,
    moat_damage: u32,
    #[serde(default)]
    second_moat: bool,
    mines: Option<String>,
    /// Pack format v39 (E04c T6).
    hit_points: Vec<u8>,
}

/// `siege` in `[battle]` (pack format v37, E04c T5): the siege's field
/// (`h3_content_battle::SiegeFieldDef`).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SiegeFieldDto {
    walls: Vec<u8>,
    gate: u8,
    moat: Vec<u8>,
    second_moat: Vec<u8>,
    moat_defense_penalty: u8,
    mine_power: u8,
    /// Pack format v39 (E04c T6): the wall shots.
    catapult: u8,
    /// Pack format v41 (E04c T7): the keep's least count.
    turret_count: u8,
    targets: Vec<WallTargetDto>,
    missed: Vec<u8>,
    automatic: Vec<u8>,
    fallback: Vec<u8>,
    castle_bonus: u8,
    ballistics: Vec<BallisticsDto>,
}

/// A wall-shot target of `siege` (pack format v39): `{ kind = "wall", wall = 29 }`; a tower's
/// or the keep's turret since v41 (E04c T7): `{ kind = "keep", turret = 254 }`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WallTargetDto {
    kind: String,
    wall: Option<u8>,
    turret: Option<u8>,
}

/// A Ballistics record of `siege` (pack format v39): `{ hit = [5, 10, 25, 50], shots = 1,
/// damage = [10, 60, 30] }`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BallisticsDto {
    hit: [u8; 4],
    shots: u8,
    damage: [u8; 3],
}

/// `{ factions = ["castle", "rampart"], amount = 1 }`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FactionChangeDto {
    factions: Vec<String>,
    amount: i8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BattleDto {
    field_width: u8,
    field_height: u8,
    side_columns: u8,
    army_slots: u8,
    damage_roll_samples: u8,
    attack_bonus_per_point_percent: u16,
    attack_bonus_cap_percent: u16,
    defense_reduction_per_point_permille: u16,
    defense_reduction_cap_permille: u16,
    defend_bonus_percent: u16,
    defend_bonus_min: u16,
    ranged_full_damage_distance: u8,
    ranged_penalty_percent: u8,
    melee_penalty_percent: u8,
    morale_dice: u8,
    good_morale_chances: Vec<u8>,
    bad_morale_chances: Vec<u8>,
    luck_dice: u8,
    good_luck_chances: Vec<u8>,
    war_machine_positions: Vec<[u8; 2]>,
    loose_formation: Vec<Vec<u8>>,
    tight_formation: Vec<Vec<u8>>,
    tactics_zone_columns: Vec<u8>,
    #[serde(default)]
    surrender: Option<SurrenderDto>,
    hero_casts_per_round: u8,
    side_stack_limit: u8,
    #[serde(default)]
    siege: Option<SiegeFieldDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SurrenderDto {
    resource: String,
    cost_percent: u16,
}

/// `[[creature]]` in `battle.toml`: the battle data of the creature `id` that the pack's
/// `core.toml` defines.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreatureDto {
    id: String,
    attack: u16,
    defense: u16,
    damage: [u16; 2],
    hit_points: u32,
    #[serde(default)]
    shots: u16,
    #[serde(default)]
    casts: u8,
    #[serde(default)]
    abilities: Vec<AbilityDto>,
}

/// One ability (`packs/README.md`, "Abilities"): `kind` plus the parameters of that kind.
/// Parameterless kinds are empty struct variants so that `deny_unknown_fields` rejects stray keys.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum AbilityDto {
    Hate {
        creature: String,
        percent: u16,
    },
    Jousting {
        percent_per_hex: u16,
    },
    ChargeImmunity {},
    DefenseReduction {
        percent: u8,
    },
    DeathBlow {
        chance_percent: u8,
    },
    LifeDrain {
        percent: u16,
    },
    Regeneration {},
    DeathStare {
        chance_percent: u8,
    },
    ExtraRetaliations {
        count: u8,
    },
    UnlimitedRetaliations {},
    StrikeAndReturn {},
    AttacksAllAdjacent {},
    ThreeHeadedAttack {},
    Teleports {},
    MoraleAura {
        amount: i8,
        affects: AuraDto,
    },
    LuckAura {
        amount: i8,
        affects: AuraDto,
    },
    MinimumMorale {
        value: i8,
    },
    MinimumLuck {
        value: i8,
    },
    NoDistancePenalty {},
    AreaShot {
        effect: AreaShotDto,
    },
    FirstAid {
        min_hp: u16,
        max_hp: u16,
    },
    AmmoSupply {},
    SpellLevelImmunity {
        max_level: u8,
    },
    SchoolImmunity {
        school: String,
    },
    SpellImmunity {
        spell: String,
        /// Pack format v31 (E04b T8b2): no immunity negation lifts it.
        #[serde(default)]
        absolute: bool,
    },
    SpellVulnerability {
        spell: String,
        percent: u16,
    },
    SpellDamageReduction {
        percent: u8,
    },
    MagicResistance {
        percent: u8,
    },
    MagicResistanceAura {
        percent: u8,
    },
    SpellAfterStrike {
        spell: String,
        chance_percent: u8,
        mastery: u8,
        rounds: u8,
        #[serde(default)]
        living: bool,
        #[serde(default)]
        unless_active: bool,
        #[serde(default)]
        defended: bool,
    },
    SpellDamageAfterStrike {
        spell: String,
        chance_percent: u8,
        per_creature: u16,
    },
    DamageAfterStrike {
        chance_percent: u8,
        percent: u8,
    },
    SpellCaster {
        spell: String,
        mastery: u8,
        power: u8,
    },
    RandomSpellCaster {
        mastery: u8,
        power_per_creature: u8,
    },
    DrawnSpell {
        spell: String,
        weight: u8,
    },
    HelpfulSpellCaster {
        mastery: u8,
        power: u8,
    },
    StackRaiser {
        spell: String,
        hit_points_per_creature: u16,
    },
    CorpseRaiser {
        creature: String,
        hit_points_per_creature: u16,
    },
    PeriodicSpellCaster {
        rounds: u8,
        mastery: u8,
        power: u8,
    },
    SpellCost {
        amount: i8,
        affects: AuraDto,
    },
    ManaChannel {
        percent: u8,
    },
    ManaDrain {
        amount: u8,
    },
    InnateSpell {
        spell: String,
        value: u8,
    },
    DamagePenalty {
        creature: String,
        percent: u8,
    },
    DamagePenaltyByFlag {
        flag: String,
        percent: u8,
    },
    Rebirth {
        divisor: u8,
    },
    Fear {
        percent: u8,
    },
    /// Pack format v31 (E04b T8b, BAT-Q21).
    IgnoresMagicBlock {},
    /// Pack format v37 (E04c T5, BAT-SIE-008).
    NoWallPenalty {},
    /// Pack format v39 (E04c T6, BAT-SIE-010).
    WallShots {
        level: u8,
    },
    /// Pack format v41 (E04c T7, BAT-SIE-016).
    Turret {},
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum AuraDto {
    Allies,
    Enemies,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum AreaShotDto {
    Fireball,
    DeathCloud,
}

impl AbilityDto {
    fn into_def(self, pack: &str) -> Result<AbilityDef, BattlePackError> {
        let aura = |a: AuraDto| match a {
            AuraDto::Allies => AuraTarget::Allies,
            AuraDto::Enemies => AuraTarget::Enemies,
        };
        Ok(match self {
            Self::Hate { creature, percent } => AbilityDef::Hate {
                target: reference(pack, &creature)?,
                percent,
            },
            Self::Jousting { percent_per_hex } => AbilityDef::Jousting { percent_per_hex },
            Self::ChargeImmunity {} => AbilityDef::ChargeImmunity,
            Self::DefenseReduction { percent } => AbilityDef::DefenseReduction { percent },
            Self::DeathBlow { chance_percent } => AbilityDef::DeathBlow { chance_percent },
            Self::LifeDrain { percent } => AbilityDef::LifeDrain { percent },
            Self::Regeneration {} => AbilityDef::Regeneration,
            Self::DeathStare { chance_percent } => AbilityDef::DeathStare { chance_percent },
            Self::ExtraRetaliations { count } => AbilityDef::ExtraRetaliations { count },
            Self::UnlimitedRetaliations {} => AbilityDef::UnlimitedRetaliations,
            Self::StrikeAndReturn {} => AbilityDef::StrikeAndReturn,
            Self::AttacksAllAdjacent {} => AbilityDef::AttacksAllAdjacent,
            Self::ThreeHeadedAttack {} => AbilityDef::ThreeHeadedAttack,
            Self::Teleports {} => AbilityDef::Teleports,
            Self::MoraleAura { amount, affects } => AbilityDef::MoraleAura {
                amount,
                affects: aura(affects),
            },
            Self::LuckAura { amount, affects } => AbilityDef::LuckAura {
                amount,
                affects: aura(affects),
            },
            Self::MinimumMorale { value } => AbilityDef::MinimumMorale { value },
            Self::MinimumLuck { value } => AbilityDef::MinimumLuck { value },
            Self::NoDistancePenalty {} => AbilityDef::NoDistancePenalty,
            Self::AreaShot { effect } => AbilityDef::AreaShot {
                effect: match effect {
                    AreaShotDto::Fireball => AreaShot::Fireball,
                    AreaShotDto::DeathCloud => AreaShot::DeathCloud,
                },
            },
            Self::FirstAid { min_hp, max_hp } => AbilityDef::FirstAid { min_hp, max_hp },
            Self::AmmoSupply {} => AbilityDef::AmmoSupply,
            Self::SpellLevelImmunity { max_level } => AbilityDef::SpellLevelImmunity { max_level },
            Self::SchoolImmunity { school } => AbilityDef::SchoolImmunity {
                school: reference(pack, &school)?,
            },
            Self::SpellImmunity { spell, absolute } => AbilityDef::SpellImmunity {
                spell: reference(pack, &spell)?,
                absolute,
            },
            Self::SpellVulnerability { spell, percent } => AbilityDef::SpellVulnerability {
                spell: reference(pack, &spell)?,
                percent,
            },
            Self::SpellDamageReduction { percent } => AbilityDef::SpellDamageReduction { percent },
            Self::MagicResistance { percent } => AbilityDef::MagicResistance { percent },
            Self::MagicResistanceAura { percent } => AbilityDef::MagicResistanceAura { percent },
            Self::SpellAfterStrike {
                spell,
                chance_percent,
                mastery,
                rounds,
                living,
                unless_active,
                defended,
            } => AbilityDef::SpellAfterStrike {
                spell: reference(pack, &spell)?,
                chance_percent,
                mastery,
                rounds,
                living,
                unless_active,
                defended,
            },
            Self::SpellDamageAfterStrike {
                spell,
                chance_percent,
                per_creature,
            } => AbilityDef::SpellDamageAfterStrike {
                spell: reference(pack, &spell)?,
                chance_percent,
                per_creature,
            },
            Self::DamageAfterStrike {
                chance_percent,
                percent,
            } => AbilityDef::DamageAfterStrike {
                chance_percent,
                percent,
            },
            Self::SpellCaster {
                spell,
                mastery,
                power,
            } => AbilityDef::SpellCaster {
                spell: reference(pack, &spell)?,
                mastery,
                power,
            },
            Self::RandomSpellCaster {
                mastery,
                power_per_creature,
            } => AbilityDef::RandomSpellCaster {
                mastery,
                power_per_creature,
            },
            Self::DrawnSpell { spell, weight } => AbilityDef::DrawnSpell {
                spell: reference(pack, &spell)?,
                weight,
            },
            Self::HelpfulSpellCaster { mastery, power } => {
                AbilityDef::HelpfulSpellCaster { mastery, power }
            }
            Self::StackRaiser {
                spell,
                hit_points_per_creature,
            } => AbilityDef::StackRaiser {
                spell: reference(pack, &spell)?,
                hit_points_per_creature,
            },
            Self::CorpseRaiser {
                creature,
                hit_points_per_creature,
            } => AbilityDef::CorpseRaiser {
                creature: reference(pack, &creature)?,
                hit_points_per_creature,
            },
            Self::PeriodicSpellCaster {
                rounds,
                mastery,
                power,
            } => AbilityDef::PeriodicSpellCaster {
                rounds,
                mastery,
                power,
            },
            Self::SpellCost { amount, affects } => AbilityDef::SpellCost {
                amount,
                affects: aura(affects),
            },
            Self::ManaChannel { percent } => AbilityDef::ManaChannel { percent },
            Self::ManaDrain { amount } => AbilityDef::ManaDrain { amount },
            Self::InnateSpell { spell, value } => AbilityDef::InnateSpell {
                spell: reference(pack, &spell)?,
                value,
            },
            Self::DamagePenalty { creature, percent } => AbilityDef::DamagePenalty {
                target: reference(pack, &creature)?,
                percent,
            },
            Self::DamagePenaltyByFlag { flag, percent } => AbilityDef::DamagePenaltyByFlag {
                flag: CreatureFlag::from_name(&flag).ok_or(CompileError::UnknownFlag(flag))?,
                percent,
            },
            Self::Rebirth { divisor } => AbilityDef::Rebirth { divisor },
            Self::Fear { percent } => AbilityDef::Fear { percent },
            Self::IgnoresMagicBlock {} => AbilityDef::IgnoresMagicBlock,
            Self::NoWallPenalty {} => AbilityDef::NoWallPenalty,
            Self::WallShots { level } => AbilityDef::WallShots { level },
            Self::Turret {} => AbilityDef::Turret,
        })
    }
}

fn abilities(pack: &str, list: Vec<AbilityDto>) -> Result<Vec<AbilityDef>, BattlePackError> {
    list.into_iter().map(|a| a.into_def(pack)).collect()
}

/// `[[creature_override]]` in `battle.toml`: the battle half of an override (every field but
/// `target` is optional). Since pack format v34 (E04b T8e) `fixes` names the data quirk flag the
/// override fixes: it is applied only in profiles that turn the flag off (ADR-0007).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OverrideDto {
    target: String,
    fixes: Option<String>,
    attack: Option<u16>,
    defense: Option<u16>,
    damage: Option<[u16; 2]>,
    hit_points: Option<u32>,
    shots: Option<u16>,
    casts: Option<u8>,
    abilities: Option<Vec<AbilityDto>>,
}

/// `[[spell]]` in `battle.toml`: the battle magic of the spell `id` that the pack's `core.toml`
/// defines (targeting and effects since format v4, E04b T2a); since format v5 (E04b T3a) mind
/// spells, mass versions, durations, cumulative spells, counters and timed effects; since format
/// v7 (E04b T3b1b) dispel and heal effects, the side by mastery, targets that need an effect,
/// spells that ignore immunity and the dispel lists; since format v9 (E04b T3c) a specialist's
/// bonus.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SpellDto {
    id: String,
    polarity: SpellPolarityDto,
    #[serde(default)]
    target: SpellTargetDto,
    area: Option<SpellAreaDto>,
    chain: Option<SpellChainDto>,
    #[serde(default)]
    side: TargetSideDto,
    #[serde(default)]
    requires: Vec<String>,
    #[serde(default)]
    excludes: Vec<String>,
    /// Pack format v32 (E04b T8b3): a stack's creature needs a maximum damage above 0.
    #[serde(default)]
    needs_damage: bool,
    #[serde(default)]
    effects: Vec<SpellEffectDto>,
    #[serde(default)]
    mind: bool,
    mass: Option<String>,
    fixed_mass: Option<String>,
    #[serde(default)]
    duration: EffectDurationDto,
    fixed_rounds: Option<u16>,
    #[serde(default)]
    ends: EffectEndDto,
    #[serde(default)]
    health_limit: bool,
    #[serde(default)]
    refuses: Vec<String>,
    #[serde(default)]
    cumulative: bool,
    #[serde(default)]
    counters: Vec<String>,
    any_side: Option<String>,
    #[serde(default)]
    needs_effect: bool,
    #[serde(default)]
    ignores_immunity: bool,
    #[serde(default)]
    dispels: Vec<String>,
    #[serde(default)]
    spares: Vec<String>,
    specialist: Option<SpecialistDto>,
    obstacle: Option<ObstacleShapeDto>,
    clears_obstacles: Option<String>,
    /// `removes = { fire = "advanced", wall = "expert" }`: the mastery from which a remove effect
    /// removes a spell obstacle of each kind (pack format v16, E04b T5b).
    #[serde(default)]
    removes: BTreeMap<String, String>,
    /// `temporary_below = "advanced"`: a raise effect below this mastery lasts for the battle
    /// only (pack format v17, E04b T6a).
    temporary_below: Option<String>,
    /// `tests_as = "resurrection"`: the spell whose target test a sacrifice effect's target
    /// takes under quirk BAT-QUIRK-5 (pack format v17, E04b T6a).
    tests_as: Option<String>,
    /// `summons = "fire_elemental"`: the creature a summon effect summons (pack format v18, E04b
    /// T6b).
    summons: Option<String>,
    /// `dispels_positive = true`: the dispel effect ends only positive spells' effects (pack
    /// format v19, E04b T7a1; Dispel Helpful Spells).
    #[serde(default)]
    dispels_positive: bool,
    /// `restores_health = true`: a cast first gives back Poison's lost hit points (pack format
    /// v20, E04b T7a2; Cure).
    #[serde(default)]
    restores_health: bool,
    /// `useful = "<when>"`: when the spell is useful on a stack for a helpful caster (pack
    /// format v24, E04b T7b2b; `Usefulness` names; default `always`).
    useful: Option<String>,
}

/// `obstacle = { steps = ["top_right", "top_left"], hexes = [2, 2, 3, 3], mirrored = true,
/// separate = true, rounds = 2 }` (pack format v13, E04b T5a; `mirrored` and `separate` default
/// false) or `obstacle = { hexes = [4, 4, 6, 8], random = true, separate = true }` (v14, E04b
/// T5c1: `random` defaults false, `rounds` 0, the whole battle; the kinds `pit`, v14, and `mine`,
/// v15, T5c2).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObstacleShapeDto {
    #[serde(default)]
    steps: Vec<String>,
    hexes: [u8; 4],
    #[serde(default)]
    random: bool,
    #[serde(default)]
    mirrored: bool,
    #[serde(default)]
    separate: bool,
    #[serde(default)]
    rounds: u8,
}

impl ObstacleShapeDto {
    fn shape(&self) -> Result<ObstacleShape, BattlePackError> {
        Ok(ObstacleShape {
            steps: self
                .steps
                .iter()
                .map(|n| {
                    HexStep::from_name(n)
                        .ok_or_else(|| BattlePackError::Syntax(format!("unknown hex step `{n}`")))
                })
                .collect::<Result<_, _>>()?,
            hexes: self.hexes,
            random: self.random,
            mirrored: self.mirrored,
            separate: self.separate,
            rounds: self.rounds,
        })
    }
}

/// `specialist = { kind = "creature_level", values = [3, 3, 2, 2, 1, 1, 0] }` (or `{ kind =
/// "fixed", amount = 2 }`, `{ kind = "total", value = 3 }`; pack format v9, E04b T3c).
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum SpecialistDto {
    CreatureLevel { values: [u8; SPECIALIST_LEVELS] },
    Fixed { amount: u16 },
    Total { value: u16 },
}

/// The mastery (1..=3) named `name` in a pack (`basic`, `advanced`, `expert`).
fn mastery_named(name: &str) -> Result<u8, BattlePackError> {
    (1u8..)
        .zip(MASTERY_NAMES)
        .find_map(|(m, n)| (n == name).then_some(m))
        .ok_or_else(|| BattlePackError::Syntax(format!("unknown mastery `{name}`")))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum EffectDurationDto {
    #[default]
    Rounds,
    Permanent,
    OwnTurn,
    Fixed,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum EffectEndDto {
    #[default]
    Never,
    Damaged,
    OwnAttack,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum SpellTargetDto {
    #[default]
    None,
    Stack,
    Hex,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum TargetSideDto {
    #[default]
    Any,
    Enemy,
    Own,
}

/// `area = { inner = 1, radius = [1, 1, 1, 1] }` (`inner` default 0).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SpellAreaDto {
    #[serde(default)]
    inner: u8,
    radius: [u8; 4],
}

/// `chain = { targets = [4, 4, 5, 5], factor_bp = 5000 }`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SpellChainDto {
    targets: [u8; 4],
    factor_bp: u16,
}

/// One spell effect: `kind` plus that kind's parameters.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum SpellEffectDto {
    // A struct variant, so that unknown keys are rejected.
    Damage {},
    Timed { change: String },
    Dispel {},
    Heal {},
    Obstacle { obstacle: String },
    Teleport {},
    RemoveObstacle {},
    Raise {},
    Sacrifice {},
    Clone {},
    Summon {},
    Earthquake {},
}

impl SpellDto {
    /// The battle magic of the spell (creature flag names resolved).
    fn magic(&self) -> Result<SpellMagic, BattlePackError> {
        let flags = |names: &[String]| {
            names
                .iter()
                .try_fold(CreatureFlags::default(), |set, name| {
                    CreatureFlag::from_name(name)
                        .map(|f| set.with(f))
                        .ok_or_else(|| CompileError::UnknownFlag(name.clone()))
                })
        };
        Ok(SpellMagic {
            target: match self.target {
                SpellTargetDto::None => SpellTarget::None,
                SpellTargetDto::Stack => SpellTarget::Stack,
                SpellTargetDto::Hex => SpellTarget::Hex,
            },
            area: self.area.as_ref().map(|a| SpellArea {
                inner: a.inner,
                radius: a.radius,
            }),
            chain: self.chain.as_ref().map(|c| SpellChain {
                targets: c.targets,
                factor_bp: c.factor_bp,
            }),
            side: match self.side {
                TargetSideDto::Any => TargetSide::Any,
                TargetSideDto::Enemy => TargetSide::Enemy,
                TargetSideDto::Own => TargetSide::Own,
            },
            conditions: TargetConditions {
                requires: flags(&self.requires)?,
                excludes: flags(&self.excludes)?,
                needs_damage: self.needs_damage,
            },
            effects: self
                .effects
                .iter()
                .map(|e| match e {
                    SpellEffectDto::Damage {} => Ok(SpellEffect::Damage),
                    SpellEffectDto::Dispel {} => Ok(SpellEffect::Dispel),
                    SpellEffectDto::Heal {} => Ok(SpellEffect::Heal),
                    SpellEffectDto::Teleport {} => Ok(SpellEffect::Teleport),
                    SpellEffectDto::RemoveObstacle {} => Ok(SpellEffect::RemoveObstacle),
                    SpellEffectDto::Raise {} => Ok(SpellEffect::Raise),
                    SpellEffectDto::Sacrifice {} => Ok(SpellEffect::Sacrifice),
                    SpellEffectDto::Clone {} => Ok(SpellEffect::Clone),
                    SpellEffectDto::Summon {} => Ok(SpellEffect::Summon),
                    SpellEffectDto::Earthquake {} => Ok(SpellEffect::Earthquake),
                    SpellEffectDto::Obstacle { obstacle } => ObstacleKind::from_name(obstacle)
                        .map(SpellEffect::Obstacle)
                        .ok_or_else(|| {
                            BattlePackError::Syntax(format!("unknown obstacle kind `{obstacle}`"))
                        }),
                    SpellEffectDto::Timed { change } => TimedChange::from_name(change)
                        .map(SpellEffect::Timed)
                        .ok_or_else(|| {
                            BattlePackError::Syntax(format!("unknown timed change `{change}`"))
                        }),
                })
                .collect::<Result<_, _>>()?,
            mind: self.mind,
            mass_from: self.mass.as_deref().map(mastery_named).transpose()?,
            fixed_mass_from: self.fixed_mass.as_deref().map(mastery_named).transpose()?,
            ends: match self.ends {
                EffectEndDto::Never => EffectEnd::Never,
                EffectEndDto::Damaged => EffectEnd::Damaged,
                EffectEndDto::OwnAttack => EffectEnd::OwnAttack,
            },
            duration: match (&self.duration, self.fixed_rounds) {
                (EffectDurationDto::Rounds, None) => EffectDuration::Rounds,
                (EffectDurationDto::Permanent, None) => EffectDuration::Permanent,
                (EffectDurationDto::OwnTurn, None) => EffectDuration::OwnTurn,
                (EffectDurationDto::Fixed, Some(rounds)) => EffectDuration::Fixed(rounds),
                _ => {
                    return Err(BattlePackError::Syntax(String::from(
                        "`fixed_rounds` goes with `duration = \"fixed\"` and only with it",
                    )));
                }
            },
            health_limit: self.health_limit,
            refuses: self
                .refuses
                .iter()
                .map(|c| {
                    TimedChange::from_name(c).ok_or_else(|| {
                        BattlePackError::Syntax(format!("unknown timed change `{c}`"))
                    })
                })
                .collect::<Result<_, _>>()?,
            cumulative: self.cumulative,
            any_side_from: self.any_side.as_deref().map(mastery_named).transpose()?,
            needs_effect: self.needs_effect,
            ignores_immunity: self.ignores_immunity,
            dispels_positive: self.dispels_positive,
            restores_health: self.restores_health,
            useful: self
                .useful
                .as_deref()
                .map(|name| {
                    Usefulness::from_name(name).ok_or_else(|| {
                        BattlePackError::Syntax(format!("unknown usefulness `{name}`"))
                    })
                })
                .transpose()?
                .unwrap_or_default(),
            specialist: self.specialist.as_ref().map(|b| match *b {
                SpecialistDto::CreatureLevel { values } => SpecialistBonus::CreatureLevel(values),
                SpecialistDto::Fixed { amount } => SpecialistBonus::Fixed(amount),
                SpecialistDto::Total { value } => SpecialistBonus::Total(value),
            }),
            obstacle: self
                .obstacle
                .as_ref()
                .map(ObstacleShapeDto::shape)
                .transpose()?,
            clears_obstacles_from: self
                .clears_obstacles
                .as_deref()
                .map(mastery_named)
                .transpose()?,
            removes_from: self.removes_from()?,
            temporary_below: self
                .temporary_below
                .as_deref()
                .map(mastery_named)
                .transpose()?,
        })
    }

    /// The masteries from which a remove effect removes each obstacle kind (`removes`).
    fn removes_from(&self) -> Result<[Option<u8>; ObstacleKind::COUNT], BattlePackError> {
        let mut out = [None; ObstacleKind::COUNT];
        for (kind, mastery) in &self.removes {
            let k = ObstacleKind::from_name(kind).ok_or_else(|| {
                BattlePackError::Syntax(format!("unknown obstacle kind `{kind}`"))
            })?;
            out[k as usize] = Some(mastery_named(mastery)?);
        }
        Ok(out)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum SpellPolarityDto {
    Positive,
    Neutral,
    Negative,
}

/// The siege's field of `[battle]` (pack format v37; the wall shots since v39).
fn siege_field(s: SiegeFieldDto) -> Result<SiegeFieldDef, BattlePackError> {
    let targets = s
        .targets
        .into_iter()
        .map(|t| {
            WallKind::from_name(&t.kind)
                .map(|kind| WallTarget {
                    kind,
                    wall: t.wall,
                    turret: t.turret,
                })
                .ok_or_else(|| BattlePackError::Syntax(format!("unknown wall kind `{}`", t.kind)))
        })
        .collect::<Result<_, _>>()?;
    Ok(SiegeFieldDef {
        walls: s.walls,
        gate: s.gate,
        moat: s.moat,
        second_moat: s.second_moat,
        moat_defense_penalty: s.moat_defense_penalty,
        mine_power: s.mine_power,
        catapult: s.catapult,
        turret_count: s.turret_count,
        targets,
        missed: s.missed,
        automatic: s.automatic,
        fallback: s.fallback,
        castle_bonus: s.castle_bonus,
        ballistics: s
            .ballistics
            .into_iter()
            .map(|b| BallisticsRecord {
                hit: b.hit,
                shots: b.shots,
                damage: b.damage,
            })
            .collect(),
    })
}

/// Parses the `battle.toml` of the core pack with this manifest (its references resolve in that
/// pack).
pub fn parse_battle(manifest: &PackManifest, text: &str) -> Result<BattlePack, BattlePackError> {
    let file: BattleFile =
        toml::from_str(text).map_err(|e| BattlePackError::Syntax(e.to_string()))?;
    let pack_id = manifest.id.as_str();
    let changes = |list: Vec<FactionChangeDto>| {
        list.into_iter()
            .map(|c| {
                Ok(FactionChangeDef {
                    factions: references(pack_id, &c.factions)?,
                    amount: c.amount,
                })
            })
            .collect::<Result<Vec<_>, KeyError>>()
    };
    let battlefields = file
        .battlefield
        .into_iter()
        .map(|b| {
            Ok(BattlefieldDef {
                key: ContentKey::new(pack_id, &b.id)?,
                own_obstacles: b.own_obstacles,
                blocked: b.blocked,
                morale: changes(b.morale)?,
                luck: changes(b.luck)?,
                no_morale: b.no_morale,
                no_luck: b.no_luck,
                no_native: b.no_native,
                max_spell_level: b.max_spell_level,
                no_creature_casts: b.no_creature_casts,
                expert_schools: references(pack_id, &b.expert_schools)?,
                expert_other_casts: b.expert_other_casts,
            })
        })
        .collect::<Result<Vec<_>, KeyError>>()?;
    let obstacles = file
        .obstacle
        .into_iter()
        .map(|o| {
            Ok(ObstacleDef {
                key: ContentKey::new(pack_id, &o.id)?,
                terrains: references(pack_id, &o.terrains)?,
                battlefields: references(pack_id, &o.battlefields)?,
                width: o.width,
                height: o.height,
                blocked: o.blocked.into_iter().map(|[a, b]| (a, b)).collect(),
                absolute: o.absolute,
            })
        })
        .collect::<Result<Vec<_>, KeyError>>()?;
    let creatures = file
        .creature
        .into_iter()
        .map(|c| {
            Ok(BattleCreatureDef {
                key: ContentKey::new(pack_id, &c.id)?,
                attack: c.attack,
                defense: c.defense,
                damage_min: c.damage[0],
                damage_max: c.damage[1],
                hit_points: c.hit_points,
                shots: c.shots,
                casts: c.casts,
                abilities: abilities(pack_id, c.abilities)?,
            })
        })
        .collect::<Result<Vec<_>, BattlePackError>>()?;
    let creature_overrides = file
        .creature_override
        .into_iter()
        .map(|o| {
            Ok(BattleCreatureOverride {
                target: ContentKey::parse(&o.target)?,
                fixes: o.fixes,
                patch: BattleCreaturePatch {
                    attack: o.attack,
                    defense: o.defense,
                    damage_min: o.damage.map(|d| d[0]),
                    damage_max: o.damage.map(|d| d[1]),
                    hit_points: o.hit_points,
                    shots: o.shots,
                    casts: o.casts,
                    abilities: o.abilities.map(|a| abilities(pack_id, a)).transpose()?,
                },
            })
        })
        .collect::<Result<Vec<_>, BattlePackError>>()?;
    let optional = |text: Option<&str>| text.map(|k| reference(pack_id, k)).transpose();
    let spells = file
        .spell
        .into_iter()
        .map(|s| {
            Ok(BattleSpellDef {
                key: ContentKey::new(pack_id, &s.id)?,
                polarity: match s.polarity {
                    SpellPolarityDto::Positive => SpellPolarity::Positive,
                    SpellPolarityDto::Neutral => SpellPolarity::Neutral,
                    SpellPolarityDto::Negative => SpellPolarity::Negative,
                },
                magic: s.magic()?,
                counters: references(pack_id, &s.counters)?,
                dispels: references(pack_id, &s.dispels)?,
                spares: references(pack_id, &s.spares)?,
                tests_as: optional(s.tests_as.as_deref())?,
                summons: optional(s.summons.as_deref())?,
            })
        })
        .collect::<Result<Vec<_>, BattlePackError>>()?;
    let battle = file
        .battle
        .map(|b| -> Result<BattleParamsDef, BattlePackError> {
            Ok(BattleParamsDef {
                field_width: b.field_width,
                field_height: b.field_height,
                side_columns: b.side_columns,
                army_slots: b.army_slots,
                damage_roll_samples: b.damage_roll_samples,
                attack_bonus_per_point_percent: b.attack_bonus_per_point_percent,
                attack_bonus_cap_percent: b.attack_bonus_cap_percent,
                defense_reduction_per_point_permille: b.defense_reduction_per_point_permille,
                defense_reduction_cap_permille: b.defense_reduction_cap_permille,
                defend_bonus_percent: b.defend_bonus_percent,
                defend_bonus_min: b.defend_bonus_min,
                ranged_full_damage_distance: b.ranged_full_damage_distance,
                ranged_penalty_percent: b.ranged_penalty_percent,
                melee_penalty_percent: b.melee_penalty_percent,
                morale_dice: b.morale_dice,
                good_morale_chances: b.good_morale_chances,
                bad_morale_chances: b.bad_morale_chances,
                luck_dice: b.luck_dice,
                good_luck_chances: b.good_luck_chances,
                war_machine_positions: b.war_machine_positions,
                loose_formation: b.loose_formation,
                tight_formation: b.tight_formation,
                tactics_zone_columns: b.tactics_zone_columns,
                surrender: b
                    .surrender
                    .map(|s| -> Result<SurrenderDef, KeyError> {
                        Ok(SurrenderDef {
                            resource: reference(pack_id, &s.resource)?,
                            cost_percent: s.cost_percent,
                        })
                    })
                    .transpose()?,
                hero_casts_per_round: b.hero_casts_per_round,
                side_stack_limit: b.side_stack_limit,
                siege: b.siege.map(siege_field).transpose()?,
            })
        })
        .transpose()?;
    let sieges = file
        .siege
        .into_iter()
        .map(|s| {
            Ok(SiegeDef {
                faction: reference(pack_id, &s.faction)?,
                moat_damage: s.moat_damage,
                second_moat: s.second_moat,
                mines: s.mines.map(|m| reference(pack_id, &m)).transpose()?,
                hit_points: s.hit_points,
            })
        })
        .collect::<Result<Vec<_>, KeyError>>()?;
    Ok(BattlePack {
        battlefields,
        sieges,
        obstacles,
        creatures,
        creature_overrides,
        spells,
        battle,
        ..BattlePack::new(manifest.clone())
    })
}

/// Parses and compiles packs given as `(core.toml, battle.toml)` texts (any order) with the
/// profile `profile`: the core ruleset first, then the battle part on it (R01).
pub fn compile(packs: &[(&str, &str)], profile: &str) -> Result<BattleRuleset, BattlePackError> {
    let core_packs = packs
        .iter()
        .map(|(core, _)| h3_pack::parse_pack(core))
        .collect::<Result<Vec<_>, _>>()?;
    let battle_packs = core_packs
        .iter()
        .zip(packs)
        .map(|(core, (_, text))| parse_battle(&core.manifest, text))
        .collect::<Result<Vec<_>, _>>()?;
    let core = Arc::new(Ruleset::compile(&core_packs, profile)?);
    Ok(BattleRuleset::compile(core, &battle_packs)?)
}

/// The synthetic ruleset with its scenario resolved: the standard fixture for tests, benchmarks
/// and the C ABI.
pub fn synthetic(scenario_text: &str) -> Result<(BattleRuleset, BattleSetup), BattlePackError> {
    let scenario = parse_scenario(scenario_text)?;
    let ruleset = compile(&[SYNTHETIC_PACK], &scenario.profile)?;
    let setup = scenario.resolve(&ruleset)?;
    Ok((ruleset, setup))
}

#[cfg(test)]
mod tests;
