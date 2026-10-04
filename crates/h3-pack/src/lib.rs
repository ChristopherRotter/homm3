//! Adapter: parses human-editable content packs and scenarios (TOML) into content definitions and
//! battle setups (ADR-0006). Pack text is untrusted input: parsing is strict (unknown fields are
//! errors, numbers must fit their types) and every definition is validated again by the
//! `Ruleset` compiler.
//!
//! Formats: `packs/README.md`; examples: `packs/synthetic/`.

use std::collections::BTreeMap;
use std::fmt;

use h3_content::{
    AbilityDef, Affects, AreaShot, ArtifactClass, ArtifactDef, ArtifactSlotDef,
    ArtifactSlotOverride, AuraTarget, BattleParamsDef, CompileError, ContentKey, ContentPack,
    CreatureDef, CreatureFlag, CreatureInfoDef, CreatureOverride, CreaturePatch, CreatureSelector,
    CreatureStat, FactionDef, Formation, HeroClassDef, HeroEffect, HeroEffectDef, HeroParamsDef,
    HeroTypeDef, KeyError, ObstacleDef, PackManifest, ProfileDef, ResourceDef, Ruleset,
    SecondarySkillDef, SpellCategory, SpellDef, SpellPolarity, SpellSchoolDef, StartingStackDef,
    TerrainDef,
};
use h3_contracts::{
    Army, ArmyStack, BallistaControl, BattleEffect, BattleHero, BattleSetup, BattleSide,
    DamageBonuses, TentControl,
};
use serde::{Deserialize, Serialize};

/// The synthetic content pack shipped with the repo (tests, benchmarks, FFI smoke, replays).
pub const SYNTHETIC_PACK: &str = include_str!("../../../packs/synthetic/pack.toml");
/// Synthetic expansion pack (depends on the synthetic pack; dependencies and overrides).
pub const SYNTHETIC_EXT_PACK: &str = include_str!("../../../packs/synthetic-ext/pack.toml");
/// Harvested pack `base`: The Restoration of Erathia, latest patch (E02, OPEN-QUESTIONS Q13/Q15).
pub const BASE_PACK: &str = include_str!("../../../packs/base/pack.toml");
/// Harvested pack `ab`: Armageddon's Blade (depends on `base`).
pub const AB_PACK: &str = include_str!("../../../packs/ab/pack.toml");
/// Harvested pack `sod`: The Shadow of Death (depends on `ab`).
pub const SOD_PACK: &str = include_str!("../../../packs/sod/pack.toml");
/// Synthetic scenario: three stacks per side.
pub const SYNTHETIC_SKIRMISH: &str =
    include_str!("../../../packs/synthetic/scenarios/skirmish.toml");
/// Synthetic scenario: one stack per side.
pub const SYNTHETIC_DUEL: &str = include_str!("../../../packs/synthetic/scenarios/duel.toml");
/// Synthetic scenario: seven stacks per side.
pub const SYNTHETIC_FULL: &str = include_str!("../../../packs/synthetic/scenarios/full.toml");

/// Errors of this adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackError {
    /// The text is not valid TOML for the expected schema.
    Syntax(String),
    /// A key or id is malformed.
    Key(KeyError),
    /// The packs do not compile into a ruleset.
    Compile(CompileError),
    /// A scenario references a creature the ruleset does not define.
    UnknownCreature(String),
    /// A scenario references a terrain the ruleset does not define.
    UnknownTerrain(String),
    /// A scenario or replay references a hero type the ruleset does not define.
    UnknownHeroType(String),
    /// A battle hero has values the scenario and replay formats cannot hold yet (they arrive
    /// with the rules that read them, E04a T6–T10 and E04b).
    HeroNotWritable(&'static str),
}

impl fmt::Display for PackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax(e) => write!(f, "syntax: {e}"),
            Self::Key(e) => write!(f, "{e}"),
            Self::Compile(e) => write!(f, "{e}"),
            Self::UnknownCreature(k) => write!(f, "unknown creature `{k}`"),
            Self::UnknownTerrain(k) => write!(f, "unknown terrain `{k}`"),
            Self::UnknownHeroType(k) => write!(f, "unknown hero type `{k}`"),
            Self::HeroNotWritable(what) => {
                write!(f, "the file format cannot hold a hero's {what} yet")
            }
        }
    }
}

impl std::error::Error for PackError {}

impl From<KeyError> for PackError {
    fn from(e: KeyError) -> Self {
        Self::Key(e)
    }
}

impl From<CompileError> for PackError {
    fn from(e: CompileError) -> Self {
        Self::Compile(e)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PackFile {
    pack: ManifestDto,
    battle: Option<BattleDto>,
    hero_params: Option<HeroParamsDto>,
    #[serde(default)]
    resource: Vec<NamedDto>,
    #[serde(default)]
    faction: Vec<FactionDto>,
    #[serde(default)]
    terrain: Vec<NamedDto>,
    #[serde(default)]
    obstacle: Vec<ObstacleDto>,
    #[serde(default)]
    creature: Vec<CreatureDto>,
    #[serde(default)]
    creature_override: Vec<OverrideDto>,
    #[serde(default)]
    spell_school: Vec<NamedDto>,
    #[serde(default)]
    spell: Vec<SpellDto>,
    #[serde(default)]
    secondary_skill: Vec<SkillDto>,
    #[serde(default)]
    hero_class: Vec<HeroClassDto>,
    #[serde(default)]
    hero: Vec<HeroDto>,
    #[serde(default)]
    artifact_slot: Vec<ArtifactSlotDto>,
    #[serde(default)]
    artifact_slot_override: Vec<ArtifactSlotOverrideDto>,
    #[serde(default)]
    artifact: Vec<ArtifactDto>,
    #[serde(default)]
    profile: Vec<ProfileDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NamedDto {
    id: String,
    name: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FactionDto {
    id: String,
    name: String,
    native_terrain: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObstacleDto {
    id: String,
    terrains: Vec<String>,
    #[serde(default)]
    width: u8,
    #[serde(default)]
    height: u8,
    blocked: Vec<[i8; 2]>,
    #[serde(default)]
    absolute: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestDto {
    id: String,
    version: String,
    #[serde(default)]
    depends: Vec<String>,
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
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreatureDto {
    id: String,
    attack: u16,
    defense: u16,
    damage: [u16; 2],
    hit_points: u32,
    speed: u8,
    #[serde(default)]
    name: String,
    #[serde(default)]
    plural: String,
    faction: Option<String>,
    #[serde(default = "default_level")]
    level: u8,
    #[serde(default)]
    cost: BTreeMap<String, u32>,
    #[serde(default)]
    growth: u16,
    #[serde(default)]
    horde_growth: u16,
    #[serde(default)]
    fight_value: u32,
    #[serde(default)]
    ai_value: u32,
    #[serde(default)]
    shots: u16,
    #[serde(default)]
    casts: u8,
    #[serde(default)]
    adventure_count: [u16; 2],
    #[serde(default)]
    flags: Vec<String>,
    #[serde(default)]
    abilities: Vec<AbilityDto>,
    #[serde(default)]
    upgrades: Vec<String>,
}

/// One ability (`packs/README.md`, "Abilities"): `kind` plus the parameters of that kind.
/// Parameterless kinds are empty struct variants so that `deny_unknown_fields` rejects stray keys.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum AbilityDto {
    Hate { creature: String, percent: u16 },
    Jousting { percent_per_hex: u16 },
    ChargeImmunity {},
    DefenseReduction { percent: u8 },
    DeathBlow { chance_percent: u8 },
    LifeDrain { percent: u16 },
    Regeneration {},
    DeathStare { chance_percent: u8 },
    ExtraRetaliations { count: u8 },
    UnlimitedRetaliations {},
    StrikeAndReturn {},
    AttacksAllAdjacent {},
    ThreeHeadedAttack {},
    Teleports {},
    MoraleAura { amount: i8, affects: AuraDto },
    LuckAura { amount: i8, affects: AuraDto },
    MinimumMorale { value: i8 },
    MinimumLuck { value: i8 },
    NoDistancePenalty {},
    AreaShot { effect: AreaShotDto },
    FirstAid { min_hp: u16, max_hp: u16 },
    AmmoSupply {},
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
    fn into_def(self, pack: &str) -> Result<AbilityDef, KeyError> {
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
        })
    }
}

fn abilities(pack: &str, list: Vec<AbilityDto>) -> Result<Vec<AbilityDef>, KeyError> {
    list.into_iter().map(|a| a.into_def(pack)).collect()
}

fn default_level() -> u8 {
    1
}

/// `[[creature_override]]`: every field but `target` is optional. `faction = ""` makes the
/// creature neutral.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OverrideDto {
    target: String,
    attack: Option<u16>,
    defense: Option<u16>,
    damage: Option<[u16; 2]>,
    hit_points: Option<u32>,
    speed: Option<u8>,
    name: Option<String>,
    plural: Option<String>,
    faction: Option<String>,
    level: Option<u8>,
    cost: Option<BTreeMap<String, u32>>,
    growth: Option<u16>,
    horde_growth: Option<u16>,
    fight_value: Option<u32>,
    ai_value: Option<u32>,
    shots: Option<u16>,
    casts: Option<u8>,
    adventure_count: Option<[u16; 2]>,
    flags: Option<Vec<String>>,
    abilities: Option<Vec<AbilityDto>>,
    upgrades: Option<Vec<String>>,
}

/// `[[spell]]` (pack format v3, E04a T3): a spell's data row.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SpellDto {
    id: String,
    #[serde(default)]
    name: String,
    category: SpellCategoryDto,
    level: u8,
    #[serde(default)]
    schools: Vec<String>,
    cost: [u16; 4],
    #[serde(default)]
    power: u16,
    effect: [u16; 4],
    polarity: SpellPolarityDto,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum SpellCategoryDto {
    Adventure,
    Combat,
    Ability,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum SpellPolarityDto {
    Positive,
    Neutral,
    Negative,
}

/// `[[secondary_skill]]`: the effects of the three levels.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SkillDto {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    basic: Vec<EffectDto>,
    #[serde(default)]
    advanced: Vec<EffectDto>,
    #[serde(default)]
    expert: Vec<EffectDto>,
}

/// `[[hero_class]]`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HeroClassDto {
    id: String,
    #[serde(default)]
    name: String,
    faction: String,
    primary: [u8; 4],
    primary_gain: [[u8; 4]; 2],
    #[serde(default)]
    skill_weights: BTreeMap<String, u8>,
}

/// `[[hero]]`: a hero type.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HeroDto {
    id: String,
    #[serde(default)]
    name: String,
    class: String,
    #[serde(default)]
    skills: Vec<HeroSkillDto>,
    #[serde(default)]
    spell_book: bool,
    #[serde(default)]
    spells: Vec<String>,
    #[serde(default)]
    army: Vec<StartingStackDto>,
    #[serde(default)]
    war_machines: Vec<String>,
    #[serde(default)]
    specialty: Vec<EffectDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HeroSkillDto {
    skill: String,
    level: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StartingStackDto {
    creature: String,
    count: [u16; 2],
}

/// `[[artifact_slot]]`: a kind of equipment position.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactSlotDto {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default = "one")]
    count: u8,
    #[serde(default = "yes")]
    backpack: bool,
}

fn one() -> u8 {
    1
}

fn yes() -> bool {
    true
}

/// `[hero_params]`: the limits of a hero's details (E04a T4a).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HeroParamsDto {
    primary_min: [u8; 4],
    primary_max: u8,
    backpack: u16,
}

/// `[[artifact_slot_override]]`: changes to a slot of the closure.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactSlotOverrideDto {
    target: String,
    name: Option<String>,
    count: Option<u8>,
}

/// `[[artifact]]`: an artifact type.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactDto {
    id: String,
    #[serde(default)]
    name: String,
    class: ArtifactClassDto,
    #[serde(default)]
    cost: BTreeMap<String, u32>,
    slot: Option<String>,
    war_machine: Option<String>,
    needs_skill: Option<String>,
    #[serde(default)]
    effects: Vec<EffectDto>,
    #[serde(default)]
    parts: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum ArtifactClassDto {
    Treasure,
    Minor,
    Major,
    Relic,
    Special,
}

/// One hero effect (`packs/README.md`, "Hero effects"): `kind` plus that kind's parameters.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum EffectDto {
    Morale {
        amount: i8,
        #[serde(default)]
        affects: AffectsEntry,
    },
    Luck {
        amount: i8,
        #[serde(default)]
        affects: AffectsEntry,
    },
    RangedDamage {
        bp: u16,
    },
    MeleeDamage {
        bp: u16,
    },
    DamageReduction {
        bp: u16,
    },
    SurrenderDiscount {
        bp: u16,
    },
    Tactics {
        level: u8,
    },
    Artillery {
        shots: u8,
        double_damage_chance_bp: u16,
    },
    FirstAid {
        heal_bp: u16,
    },
    CreatureStat {
        creatures: Option<SelectorDto>,
        stat: CreatureStatDto,
        amount: i16,
    },
    SkillSpecialty {
        skill: String,
        bp_per_level: u16,
    },
    CreatureSpecialty {
        creature: String,
        bp_per_step: u16,
        #[serde(default)]
        speed: u8,
    },
    SpellSpecialty {
        spell: String,
    },
    PrimarySkills {
        #[serde(default)]
        attack: i8,
        #[serde(default)]
        defense: i8,
        #[serde(default)]
        power: i8,
        #[serde(default)]
        knowledge: i8,
    },
    MoraleAtMost {
        value: i8,
        #[serde(default)]
        affects: AffectsEntry,
    },
    LuckAtMost {
        value: i8,
        #[serde(default)]
        affects: AffectsEntry,
    },
    AlignmentGroup {
        factions: Vec<String>,
    },
    CreatureStatPercent {
        creatures: Option<SelectorDto>,
        stat: CreatureStatDto,
        bp: u16,
    },
    Regeneration {
        creatures: Option<SelectorDto>,
        hit_points: u16,
    },
    NoRangePenalty {
        creatures: Option<SelectorDto>,
    },
    ShootWhenAdjacent {
        creatures: Option<SelectorDto>,
    },
    // A struct variant, so that unknown keys are rejected like everywhere else.
    NoRetreat {},
}

/// Whose stacks a morale or luck effect changes, as files write it (`own` by default, `enemy`,
/// `both`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AffectsEntry {
    /// The hero's own stacks.
    #[default]
    Own,
    /// The enemy's stacks.
    Enemy,
    /// The stacks of both sides.
    Both,
}

impl From<AffectsEntry> for Affects {
    fn from(a: AffectsEntry) -> Self {
        match a {
            AffectsEntry::Own => Self::Own,
            AffectsEntry::Enemy => Self::Enemy,
            AffectsEntry::Both => Self::Both,
        }
    }
}

impl From<Affects> for AffectsEntry {
    fn from(a: Affects) -> Self {
        match a {
            Affects::Own => Self::Own,
            Affects::Enemy => Self::Enemy,
            Affects::Both => Self::Both,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum CreatureStatDto {
    Attack,
    Defense,
    Speed,
    HitPoints,
    MinDamage,
    MaxDamage,
}

/// `creatures = { creature = "behemoth" }` (with its upgrades) or `{ flag = "dragon" }`;
/// omitted: every creature.
#[derive(Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
enum SelectorDto {
    Creature(String),
    Flag(String),
}

impl EffectDto {
    fn into_def(self, pack: &str) -> Result<HeroEffectDef, PackError> {
        let affects = Affects::from;
        Ok(match self {
            Self::Morale { amount, affects: a } => HeroEffect::Morale {
                amount,
                affects: affects(a),
            },
            Self::Luck { amount, affects: a } => HeroEffect::Luck {
                amount,
                affects: affects(a),
            },
            Self::RangedDamage { bp } => HeroEffect::RangedDamage { bp },
            Self::MeleeDamage { bp } => HeroEffect::MeleeDamage { bp },
            Self::DamageReduction { bp } => HeroEffect::DamageReduction { bp },
            Self::SurrenderDiscount { bp } => HeroEffect::SurrenderDiscount { bp },
            Self::Tactics { level } => HeroEffect::Tactics { level },
            Self::Artillery {
                shots,
                double_damage_chance_bp,
            } => HeroEffect::Artillery {
                shots,
                double_damage_chance_bp,
            },
            Self::FirstAid { heal_bp } => HeroEffect::FirstAid { heal_bp },
            Self::CreatureStat {
                creatures,
                stat,
                amount,
            } => HeroEffect::CreatureStat {
                creatures: selector(pack, creatures)?,
                stat: stat.into_stat(),
                amount,
            },
            Self::SkillSpecialty {
                skill,
                bp_per_level,
            } => HeroEffect::SkillSpecialty {
                skill: reference(pack, &skill)?,
                bp_per_level,
            },
            Self::CreatureSpecialty {
                creature,
                bp_per_step,
                speed,
            } => HeroEffect::CreatureSpecialty {
                creature: reference(pack, &creature)?,
                bp_per_step,
                speed,
            },
            Self::SpellSpecialty { spell } => HeroEffect::SpellSpecialty {
                spell: reference(pack, &spell)?,
            },
            Self::PrimarySkills {
                attack,
                defense,
                power,
                knowledge,
            } => HeroEffect::PrimarySkills {
                attack,
                defense,
                power,
                knowledge,
            },
            Self::MoraleAtMost { value, affects: a } => HeroEffect::MoraleAtMost {
                value,
                affects: affects(a),
            },
            Self::LuckAtMost { value, affects: a } => HeroEffect::LuckAtMost {
                value,
                affects: affects(a),
            },
            Self::AlignmentGroup { factions } => HeroEffect::AlignmentGroup {
                factions: references(pack, &factions)?,
            },
            Self::CreatureStatPercent {
                creatures,
                stat,
                bp,
            } => HeroEffect::CreatureStatPercent {
                creatures: selector(pack, creatures)?,
                stat: stat.into_stat(),
                bp,
            },
            Self::Regeneration {
                creatures,
                hit_points,
            } => HeroEffect::Regeneration {
                creatures: selector(pack, creatures)?,
                hit_points,
            },
            Self::NoRangePenalty { creatures } => HeroEffect::NoRangePenalty {
                creatures: selector(pack, creatures)?,
            },
            Self::ShootWhenAdjacent { creatures } => HeroEffect::ShootWhenAdjacent {
                creatures: selector(pack, creatures)?,
            },
            Self::NoRetreat {} => HeroEffect::NoRetreat,
        })
    }
}

impl CreatureStatDto {
    fn into_stat(self) -> CreatureStat {
        match self {
            Self::Attack => CreatureStat::Attack,
            Self::Defense => CreatureStat::Defense,
            Self::Speed => CreatureStat::Speed,
            Self::HitPoints => CreatureStat::HitPoints,
            Self::MinDamage => CreatureStat::MinDamage,
            Self::MaxDamage => CreatureStat::MaxDamage,
        }
    }
}

/// A creature selector; omitted: every creature.
fn selector(
    pack: &str,
    creatures: Option<SelectorDto>,
) -> Result<CreatureSelector<ContentKey>, PackError> {
    Ok(match creatures {
        None => CreatureSelector::All,
        Some(SelectorDto::Creature(c)) => CreatureSelector::Creature(reference(pack, &c)?),
        Some(SelectorDto::Flag(f)) => {
            CreatureSelector::Flag(CreatureFlag::from_name(&f).ok_or(CompileError::UnknownFlag(f))?)
        }
    })
}

fn effects(pack: &str, list: Vec<EffectDto>) -> Result<Vec<HeroEffectDef>, PackError> {
    list.into_iter().map(|e| e.into_def(pack)).collect()
}

fn references(pack: &str, list: &[String]) -> Result<Vec<ContentKey>, KeyError> {
    list.iter().map(|t| reference(pack, t)).collect()
}

/// Resolves a reference: `name` means `<own pack>:name`, `pack:name` is taken as written.
fn reference(pack: &str, text: &str) -> Result<ContentKey, KeyError> {
    if text.contains(':') {
        ContentKey::parse(text)
    } else {
        ContentKey::new(pack, text)
    }
}

fn cost(pack: &str, cost: BTreeMap<String, u32>) -> Result<Vec<(ContentKey, u32)>, KeyError> {
    cost.into_iter()
        .map(|(k, v)| Ok((reference(pack, &k)?, v)))
        .collect()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileDto {
    id: String,
    #[serde(default)]
    quirks: BTreeMap<String, bool>,
}

/// Parses a content pack.
pub fn parse_pack(text: &str) -> Result<ContentPack, PackError> {
    let file: PackFile = toml::from_str(text).map_err(|e| PackError::Syntax(e.to_string()))?;
    let pack_id = file.pack.id;
    let named = |items: Vec<NamedDto>| {
        items
            .into_iter()
            .map(|n| Ok((ContentKey::new(&pack_id, &n.id)?, n.name)))
            .collect::<Result<Vec<_>, KeyError>>()
    };
    let resources = named(file.resource)?
        .into_iter()
        .map(|(key, name)| ResourceDef { key, name })
        .collect();
    let factions = file
        .faction
        .into_iter()
        .map(|f| {
            Ok(FactionDef {
                key: ContentKey::new(&pack_id, &f.id)?,
                name: f.name,
                native_terrain: f
                    .native_terrain
                    .map(|t| reference(&pack_id, &t))
                    .transpose()?,
            })
        })
        .collect::<Result<Vec<_>, KeyError>>()?;
    let terrains = named(file.terrain)?
        .into_iter()
        .map(|(key, name)| TerrainDef { key, name })
        .collect();
    let obstacles = file
        .obstacle
        .into_iter()
        .map(|o| {
            Ok(ObstacleDef {
                key: ContentKey::new(&pack_id, &o.id)?,
                terrains: o
                    .terrains
                    .iter()
                    .map(|t| reference(&pack_id, t))
                    .collect::<Result<_, _>>()?,
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
            Ok(CreatureDef {
                key: ContentKey::new(&pack_id, &c.id)?,
                attack: c.attack,
                defense: c.defense,
                damage_min: c.damage[0],
                damage_max: c.damage[1],
                hit_points: c.hit_points,
                speed: c.speed,
                info: CreatureInfoDef {
                    name: c.name,
                    plural: c.plural,
                    faction: c.faction.map(|f| reference(&pack_id, &f)).transpose()?,
                    level: c.level,
                    cost: cost(&pack_id, c.cost)?,
                    growth: c.growth,
                    horde_growth: c.horde_growth,
                    fight_value: c.fight_value,
                    ai_value: c.ai_value,
                    shots: c.shots,
                    casts: c.casts,
                    adventure_count: c.adventure_count,
                    flags: c.flags,
                    abilities: abilities(&pack_id, c.abilities)?,
                    upgrades: references(&pack_id, &c.upgrades)?,
                },
            })
        })
        .collect::<Result<Vec<_>, PackError>>()?;
    let creature_overrides = file
        .creature_override
        .into_iter()
        .map(|o| {
            let faction = o
                .faction
                .map(|f| (!f.is_empty()).then(|| reference(&pack_id, &f)).transpose())
                .transpose()?;
            Ok(CreatureOverride {
                target: ContentKey::parse(&o.target)?,
                patch: CreaturePatch {
                    attack: o.attack,
                    defense: o.defense,
                    damage_min: o.damage.map(|d| d[0]),
                    damage_max: o.damage.map(|d| d[1]),
                    hit_points: o.hit_points,
                    speed: o.speed,
                    name: o.name,
                    plural: o.plural,
                    faction,
                    level: o.level,
                    cost: o.cost.map(|c| cost(&pack_id, c)).transpose()?,
                    growth: o.growth,
                    horde_growth: o.horde_growth,
                    fight_value: o.fight_value,
                    ai_value: o.ai_value,
                    shots: o.shots,
                    casts: o.casts,
                    adventure_count: o.adventure_count,
                    flags: o.flags,
                    abilities: o.abilities.map(|a| abilities(&pack_id, a)).transpose()?,
                    upgrades: o.upgrades.map(|u| references(&pack_id, &u)).transpose()?,
                },
            })
        })
        .collect::<Result<Vec<_>, PackError>>()?;
    let spell_schools = named(file.spell_school)?
        .into_iter()
        .map(|(key, name)| SpellSchoolDef { key, name })
        .collect();
    let spells = file
        .spell
        .into_iter()
        .map(|s| {
            Ok(SpellDef {
                key: ContentKey::new(&pack_id, &s.id)?,
                name: s.name,
                category: match s.category {
                    SpellCategoryDto::Adventure => SpellCategory::Adventure,
                    SpellCategoryDto::Combat => SpellCategory::Combat,
                    SpellCategoryDto::Ability => SpellCategory::Ability,
                },
                level: s.level,
                schools: references(&pack_id, &s.schools)?,
                cost: s.cost,
                power: s.power,
                effect: s.effect,
                polarity: match s.polarity {
                    SpellPolarityDto::Positive => SpellPolarity::Positive,
                    SpellPolarityDto::Neutral => SpellPolarity::Neutral,
                    SpellPolarityDto::Negative => SpellPolarity::Negative,
                },
            })
        })
        .collect::<Result<Vec<_>, KeyError>>()?;
    let skills = file
        .secondary_skill
        .into_iter()
        .map(|s| {
            Ok(SecondarySkillDef {
                key: ContentKey::new(&pack_id, &s.id)?,
                name: s.name,
                levels: [
                    effects(&pack_id, s.basic)?,
                    effects(&pack_id, s.advanced)?,
                    effects(&pack_id, s.expert)?,
                ],
            })
        })
        .collect::<Result<Vec<_>, PackError>>()?;
    let hero_classes = file
        .hero_class
        .into_iter()
        .map(|c| {
            Ok(HeroClassDef {
                key: ContentKey::new(&pack_id, &c.id)?,
                name: c.name,
                faction: reference(&pack_id, &c.faction)?,
                primary: c.primary,
                primary_gain: c.primary_gain,
                skill_weights: c
                    .skill_weights
                    .into_iter()
                    .map(|(k, w)| Ok((reference(&pack_id, &k)?, w)))
                    .collect::<Result<_, KeyError>>()?,
            })
        })
        .collect::<Result<Vec<_>, KeyError>>()?;
    let heroes = file
        .hero
        .into_iter()
        .map(|h| {
            Ok(HeroTypeDef {
                key: ContentKey::new(&pack_id, &h.id)?,
                name: h.name,
                class: reference(&pack_id, &h.class)?,
                skills: h
                    .skills
                    .into_iter()
                    .map(|s| Ok((reference(&pack_id, &s.skill)?, s.level)))
                    .collect::<Result<_, KeyError>>()?,
                spell_book: h.spell_book,
                spells: references(&pack_id, &h.spells)?,
                army: h
                    .army
                    .into_iter()
                    .map(|s| {
                        Ok(StartingStackDef {
                            creature: reference(&pack_id, &s.creature)?,
                            count: s.count,
                        })
                    })
                    .collect::<Result<_, KeyError>>()?,
                war_machines: references(&pack_id, &h.war_machines)?,
                specialty: effects(&pack_id, h.specialty)?,
            })
        })
        .collect::<Result<Vec<_>, PackError>>()?;
    let artifact_slots = file
        .artifact_slot
        .into_iter()
        .map(|d| {
            Ok(ArtifactSlotDef {
                key: ContentKey::new(&pack_id, &d.id)?,
                name: d.name,
                count: d.count,
                backpack: d.backpack,
            })
        })
        .collect::<Result<Vec<_>, KeyError>>()?;
    let artifact_slot_overrides = file
        .artifact_slot_override
        .into_iter()
        .map(|o| {
            Ok(ArtifactSlotOverride {
                target: reference(&pack_id, &o.target)?,
                name: o.name,
                count: o.count,
            })
        })
        .collect::<Result<Vec<_>, KeyError>>()?;
    let optional = |text: Option<String>| text.map(|t| reference(&pack_id, &t)).transpose();
    let artifacts = file
        .artifact
        .into_iter()
        .map(|a| {
            Ok(ArtifactDef {
                key: ContentKey::new(&pack_id, &a.id)?,
                name: a.name,
                class: match a.class {
                    ArtifactClassDto::Treasure => ArtifactClass::Treasure,
                    ArtifactClassDto::Minor => ArtifactClass::Minor,
                    ArtifactClassDto::Major => ArtifactClass::Major,
                    ArtifactClassDto::Relic => ArtifactClass::Relic,
                    ArtifactClassDto::Special => ArtifactClass::Special,
                },
                cost: cost(&pack_id, a.cost)?,
                slot: optional(a.slot)?,
                war_machine: optional(a.war_machine)?,
                needs_skill: optional(a.needs_skill)?,
                effects: effects(&pack_id, a.effects)?,
                parts: references(&pack_id, &a.parts)?,
            })
        })
        .collect::<Result<Vec<_>, PackError>>()?;
    let profiles = file
        .profile
        .into_iter()
        .map(|p| {
            Ok(ProfileDef {
                key: ContentKey::new(&pack_id, &p.id)?,
                quirks: p.quirks.into_iter().collect(),
            })
        })
        .collect::<Result<Vec<_>, PackError>>()?;
    let battle = file.battle.map(|b| BattleParamsDef {
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
    });
    Ok(ContentPack {
        manifest: PackManifest {
            id: pack_id,
            version: file.pack.version,
            depends: file.pack.depends,
        },
        resources,
        factions,
        terrains,
        obstacles,
        creatures,
        creature_overrides,
        spell_schools,
        spells,
        skills,
        hero_classes,
        heroes,
        artifact_slots,
        artifact_slot_overrides,
        artifacts,
        profiles,
        battle,
        hero_params: file.hero_params.map(|p| HeroParamsDef {
            primary_min: p.primary_min,
            primary_max: p.primary_max,
            backpack: p.backpack,
        }),
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScenarioFile {
    scenario: ScenarioHeader,
    #[serde(default)]
    attacker: Vec<StackDto>,
    #[serde(default)]
    defender: Vec<StackDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScenarioHeader {
    id: String,
    packs: Vec<String>,
    profile: String,
    /// Obstacle hexes (row-major indices), validated by the battle context.
    #[serde(default)]
    obstacles: Vec<u8>,
    /// The battle hero leading each side, if any (BAT-HER-001, Q31).
    attacker_hero: Option<HeroEntry>,
    defender_hero: Option<HeroEntry>,
    /// Terrain key of the battlefield (BAT-TER-001).
    terrain: Option<String>,
    /// Generate obstacles for the terrain when the battle starts (BAT-GEO-004).
    #[serde(default)]
    random_obstacles: bool,
    /// War machines by creature key (BAT-WAR-001).
    #[serde(default)]
    attacker_war_machines: Vec<String>,
    #[serde(default)]
    defender_war_machines: Vec<String>,
    /// Sides in tight formation (`"attacker"`, `"defender"`; the others are loose, BAT-SET-001).
    #[serde(default)]
    tight_formation: Vec<String>,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct StackDto {
    slot: u8,
    creature: String,
    count: u32,
}

/// A battle hero as scenario and replay files write it (`[scenario.attacker_hero]`,
/// `[setup.defender_hero]`; OPEN-QUESTIONS Q31): the battle hero itself, no derivation. It holds
/// the fields the battle reads so far; the rest of [`BattleHero`] is neutral and joins the format
/// with the rule that reads it (E04a T6–T10, E04b).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeroEntry {
    /// Hero type key.
    pub hero_type: String,
    /// Attack (default 0).
    #[serde(default)]
    pub attack: u8,
    /// Defense (default 0).
    #[serde(default)]
    pub defense: u8,
    /// Spell power (default 0).
    #[serde(default)]
    pub power: u8,
    /// Knowledge (default 0).
    #[serde(default)]
    pub knowledge: u8,
    /// Tactics level (default 0).
    #[serde(default)]
    pub tactics: u8,
    /// Spell points (default 0).
    #[serde(default)]
    pub mana: u16,
    /// Maximum spell points (default 0).
    #[serde(default)]
    pub max_mana: u16,
    /// Bonus of the side's shots, basis points ([`DamageBonuses::ranged_bp`]; default 0, written
    /// only when set).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub ranged_bp: u16,
    /// Bonus of the side's melee strikes, basis points ([`DamageBonuses::melee_bp`]; default 0,
    /// written only when set).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub melee_bp: u16,
    /// Reduction of the strikes against the side, basis points ([`DamageBonuses::reduction_bp`];
    /// default 0, written only when set).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub reduction_bp: u16,
    /// The side commands its Ballista ([`MachineControl::ballista`]; default not, written only
    /// when set).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ballista: Option<BallistaEntry>,
    /// The side commands its First Aid Tent ([`MachineControl::tent`]; default not, written only
    /// when set).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tent: Option<TentEntry>,
    /// Battle effects (default none; written only when there are some, after the values).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effects: Vec<EffectEntry>,
}

/// How a hero commands its Ballista ([`BallistaControl`]) in scenario and replay files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BallistaEntry {
    /// Shots per turn.
    pub shots: u8,
    /// Chance of each shot to deal double damage, basis points.
    pub double_damage_chance_bp: u16,
}

impl From<BallistaEntry> for BallistaControl {
    fn from(e: BallistaEntry) -> Self {
        Self {
            shots: e.shots,
            double_damage_chance_bp: e.double_damage_chance_bp,
        }
    }
}

impl From<BallistaControl> for BallistaEntry {
    fn from(c: BallistaControl) -> Self {
        Self {
            shots: c.shots,
            double_damage_chance_bp: c.double_damage_chance_bp,
        }
    }
}

/// How a hero commands its First Aid Tent ([`TentControl`]) in scenario and replay files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TentEntry {
    /// Increase of the tent's largest heal, basis points of the tent's own maximum.
    pub heal_bonus_bp: u32,
}

/// Whether a value is 0 (serde's `skip_serializing_if`).
#[allow(clippy::trivially_copy_pass_by_ref)] // serde passes a reference
fn is_zero(value: &u16) -> bool {
    *value == 0
}

/// One battle effect of a hero as scenario and replay files write it: `kind` and the kind's
/// parameters, named as in the packs (`packs/README.md`, "Hero effects"). It holds the kinds
/// the battle reads so far (morale and luck, BAT-HER-003); later kinds join with their rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EffectEntry {
    /// [`BattleEffect::Morale`].
    Morale {
        /// Morale change.
        amount: i8,
        /// Whose stacks (default `own`).
        #[serde(default)]
        affects: AffectsEntry,
    },
    /// [`BattleEffect::Luck`].
    Luck {
        /// Luck change.
        amount: i8,
        /// Whose stacks (default `own`).
        #[serde(default)]
        affects: AffectsEntry,
    },
    /// [`BattleEffect::MoraleAtMost`].
    MoraleAtMost {
        /// Highest morale.
        value: i8,
        /// Whose stacks (default `own`).
        #[serde(default)]
        affects: AffectsEntry,
    },
    /// [`BattleEffect::LuckAtMost`].
    LuckAtMost {
        /// Highest luck.
        value: i8,
        /// Whose stacks (default `own`).
        #[serde(default)]
        affects: AffectsEntry,
    },
}

impl EffectEntry {
    /// The battle effect.
    pub fn effect(self) -> BattleEffect {
        match self {
            Self::Morale { amount, affects } => BattleEffect::Morale {
                amount,
                affects: affects.into(),
            },
            Self::Luck { amount, affects } => BattleEffect::Luck {
                amount,
                affects: affects.into(),
            },
            Self::MoraleAtMost { value, affects } => BattleEffect::MoraleAtMost {
                value,
                affects: affects.into(),
            },
            Self::LuckAtMost { value, affects } => BattleEffect::LuckAtMost {
                value,
                affects: affects.into(),
            },
        }
    }

    /// The entry of `effect`, if the format holds its kind (inverse of [`EffectEntry::effect`]).
    pub fn of(effect: &BattleEffect) -> Option<Self> {
        Some(match *effect {
            BattleEffect::Morale { amount, affects } => Self::Morale {
                amount,
                affects: affects.into(),
            },
            BattleEffect::Luck { amount, affects } => Self::Luck {
                amount,
                affects: affects.into(),
            },
            BattleEffect::MoraleAtMost { value, affects } => Self::MoraleAtMost {
                value,
                affects: affects.into(),
            },
            BattleEffect::LuckAtMost { value, affects } => Self::LuckAtMost {
                value,
                affects: affects.into(),
            },
            _ => return None,
        })
    }
}

impl HeroEntry {
    /// The battle hero, its hero type resolved against `ruleset`. The battle checks the rest
    /// when it starts (BAT-HER-001).
    ///
    /// # Errors
    /// [`PackError::UnknownHeroType`].
    pub fn resolve(&self, ruleset: &Ruleset) -> Result<BattleHero, PackError> {
        let hero_type = ruleset
            .hero_type_by_key(&self.hero_type)
            .ok_or_else(|| PackError::UnknownHeroType(self.hero_type.clone()))?;
        let mut hero = BattleHero::new(hero_type);
        hero.attack = self.attack;
        hero.defense = self.defense;
        hero.power = self.power;
        hero.knowledge = self.knowledge;
        hero.tactics = self.tactics;
        hero.magic.mana = self.mana;
        hero.magic.max_mana = self.max_mana;
        hero.damage = DamageBonuses {
            ranged_bp: self.ranged_bp,
            melee_bp: self.melee_bp,
            reduction_bp: self.reduction_bp,
        };
        hero.war_machines.ballista = self.ballista.map(BallistaControl::from);
        hero.war_machines.tent = self.tent.map(|t| TentControl {
            heal_bonus_bp: t.heal_bonus_bp,
        });
        hero.effects = self.effects.iter().map(|e| e.effect()).collect();
        Ok(hero)
    }

    /// The entry of `hero` (inverse of [`HeroEntry::resolve`]).
    ///
    /// # Errors
    /// [`PackError::UnknownHeroType`] for a hero type outside `ruleset`;
    /// [`PackError::HeroNotWritable`] if a field the format does not hold yet is not neutral.
    pub fn of(hero: &BattleHero, ruleset: &Ruleset) -> Result<Self, PackError> {
        let unwritable = [
            ("surrender discount", hero.surrender_discount_bp != 0),
            (
                "effects other than morale and luck",
                hero.effects.iter().any(|e| EffectEntry::of(e).is_none()),
            ),
            (
                "spell book or schools",
                hero.magic.spell_book.is_some() || hero.magic.schools != [0; 4],
            ),
        ];
        if let Some((what, _)) = unwritable.into_iter().find(|(_, set)| *set) {
            return Err(PackError::HeroNotWritable(what));
        }
        let key = ruleset
            .hero_type(hero.hero_type)
            .map(|t| t.key.as_str().to_owned())
            .ok_or_else(|| PackError::UnknownHeroType(format!("#{}", hero.hero_type.raw())))?;
        Ok(Self {
            hero_type: key,
            attack: hero.attack,
            defense: hero.defense,
            power: hero.power,
            knowledge: hero.knowledge,
            tactics: hero.tactics,
            mana: hero.magic.mana,
            max_mana: hero.magic.max_mana,
            ranged_bp: hero.damage.ranged_bp,
            melee_bp: hero.damage.melee_bp,
            reduction_bp: hero.damage.reduction_bp,
            ballista: hero.war_machines.ballista.map(BallistaEntry::from),
            tent: hero.war_machines.tent.map(|t| TentEntry {
                heal_bonus_bp: t.heal_bonus_bp,
            }),
            effects: hero.effects.iter().filter_map(EffectEntry::of).collect(),
        })
    }
}

/// A parsed scenario whose creature references are not yet resolved against a ruleset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioDef {
    /// Scenario id.
    pub id: String,
    /// Ids of the enabled packs (any order; the compiler loads dependencies first).
    pub packs: Vec<String>,
    /// Profile key the scenario is meant to run with.
    pub profile: String,
    attacker: Vec<(u8, String, u32)>,
    defender: Vec<(u8, String, u32)>,
    obstacles: Vec<u8>,
    heroes: [Option<HeroEntry>; 2],
    terrain: Option<String>,
    war_machines: [Vec<String>; 2],
    formations: [Formation; 2],
    /// Whether obstacles are generated for the terrain when the battle starts (the caller does
    /// it, with its seed; BAT-GEO-004). Excludes explicit obstacles.
    pub random_obstacles: bool,
}

/// Parses a list of side names (`"attacker"`, `"defender"`, each at most once) into flags indexed
/// by [`BattleSide::index`].
///
/// # Errors
/// An unknown or repeated side name.
pub fn parse_sides(names: &[String]) -> Result<[bool; 2], PackError> {
    let mut sides = [false; 2];
    for name in names {
        let side = match name.as_str() {
            "attacker" => BattleSide::Attacker,
            "defender" => BattleSide::Defender,
            _ => return Err(PackError::Syntax(format!("unknown side `{name}`"))),
        };
        if core::mem::replace(&mut sides[side.index()], true) {
            return Err(PackError::Syntax(format!("side `{name}` listed twice")));
        }
    }
    Ok(sides)
}

/// The formations of a list of side names in tight formation (the other sides are loose).
///
/// # Errors
/// An unknown or repeated side name.
pub fn formations(tight: &[String]) -> Result<[Formation; 2], PackError> {
    Ok(parse_sides(tight)?.map(|t| {
        if t {
            Formation::Tight
        } else {
            Formation::Loose
        }
    }))
}

/// The names of the sides in tight formation (inverse of [`formations`]).
pub fn tight_sides(formations: [Formation; 2]) -> Vec<String> {
    side_names(formations.map(|f| f == Formation::Tight))
}

/// The side names of the flags set in `sides` (inverse of [`parse_sides`]).
pub fn side_names(sides: [bool; 2]) -> Vec<String> {
    BattleSide::BOTH
        .into_iter()
        .filter(|s| sides[s.index()])
        .map(|s| match s {
            BattleSide::Attacker => "attacker".to_owned(),
            BattleSide::Defender => "defender".to_owned(),
        })
        .collect()
}

impl ScenarioDef {
    /// Resolves creature, terrain and hero-type keys against `ruleset`. The battle context
    /// validates the setup itself (slots, counts, heroes) when the battle starts.
    pub fn resolve(&self, ruleset: &Ruleset) -> Result<BattleSetup, PackError> {
        let army = |stacks: &[(u8, String, u32)]| -> Result<Army, PackError> {
            let stacks = stacks
                .iter()
                .map(|(slot, key, count)| {
                    let creature = ruleset
                        .creature_by_key(key)
                        .ok_or_else(|| PackError::UnknownCreature(key.clone()))?;
                    Ok(ArmyStack {
                        slot: *slot,
                        creature,
                        count: *count,
                    })
                })
                .collect::<Result<Vec<_>, PackError>>()?;
            Ok(Army { stacks })
        };
        let terrain = self
            .terrain
            .as_ref()
            .map(|t| {
                ruleset
                    .terrain_by_key(t)
                    .ok_or_else(|| PackError::UnknownTerrain(t.clone()))
            })
            .transpose()?;
        let machines = |keys: &[String]| -> Result<Vec<_>, PackError> {
            keys.iter()
                .map(|k| {
                    ruleset
                        .creature_by_key(k)
                        .ok_or_else(|| PackError::UnknownCreature(k.clone()))
                })
                .collect()
        };
        Ok(BattleSetup {
            attacker: army(&self.attacker)?,
            defender: army(&self.defender)?,
            obstacles: self.obstacles.clone(),
            heroes: [
                self.heroes[0]
                    .as_ref()
                    .map(|h| h.resolve(ruleset))
                    .transpose()?,
                self.heroes[1]
                    .as_ref()
                    .map(|h| h.resolve(ruleset))
                    .transpose()?,
            ],
            terrain,
            war_machines: [
                machines(&self.war_machines[0])?,
                machines(&self.war_machines[1])?,
            ],
            formations: self.formations,
        })
    }
}

/// Parses a scenario.
pub fn parse_scenario(text: &str) -> Result<ScenarioDef, PackError> {
    let file: ScenarioFile = toml::from_str(text).map_err(|e| PackError::Syntax(e.to_string()))?;
    ContentKey::parse(&file.scenario.profile)?;
    if file.scenario.random_obstacles && !file.scenario.obstacles.is_empty() {
        return Err(PackError::Syntax(
            "random_obstacles excludes an obstacles list".to_owned(),
        ));
    }
    if file.scenario.random_obstacles && file.scenario.terrain.is_none() {
        return Err(PackError::Syntax(
            "random_obstacles needs a terrain".to_owned(),
        ));
    }
    let convert = |v: Vec<StackDto>| {
        v.into_iter()
            .map(|s| (s.slot, s.creature, s.count))
            .collect()
    };
    Ok(ScenarioDef {
        id: file.scenario.id,
        packs: file.scenario.packs,
        profile: file.scenario.profile,
        attacker: convert(file.attacker),
        defender: convert(file.defender),
        obstacles: file.scenario.obstacles,
        heroes: [file.scenario.attacker_hero, file.scenario.defender_hero],
        terrain: file.scenario.terrain,
        war_machines: [
            file.scenario.attacker_war_machines,
            file.scenario.defender_war_machines,
        ],
        formations: formations(&file.scenario.tight_formation)?,
        random_obstacles: file.scenario.random_obstacles,
    })
}

/// Parses and compiles the enabled packs (any order) with a profile.
pub fn compile(pack_texts: &[&str], profile: &str) -> Result<Ruleset, PackError> {
    let packs = pack_texts
        .iter()
        .map(|t| parse_pack(t))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Ruleset::compile(&packs, profile)?)
}

/// The synthetic ruleset with its scenario resolved: the standard fixture for tests, benchmarks
/// and the C ABI.
pub fn synthetic(scenario_text: &str) -> Result<(Ruleset, BattleSetup), PackError> {
    let scenario = parse_scenario(scenario_text)?;
    let ruleset = compile(&[SYNTHETIC_PACK], &scenario.profile)?;
    let setup = scenario.resolve(&ruleset)?;
    Ok((ruleset, setup))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_pack_and_scenarios_load() {
        for text in [SYNTHETIC_SKIRMISH, SYNTHETIC_DUEL, SYNTHETIC_FULL] {
            let (ruleset, setup) = synthetic(text).unwrap();
            assert_eq!(ruleset.creature_count(), 6);
            assert!(!setup.attacker.stacks.is_empty());
            assert!(!setup.defender.stacks.is_empty());
        }
        let (ruleset, full) = synthetic(SYNTHETIC_FULL).unwrap();
        let key = |c| ruleset.creature_key(c).unwrap().as_str();
        assert_eq!(
            full.war_machines
                .map(|m| m.into_iter().map(key).collect::<Vec<_>>()),
            [
                ["synthetic:bolt_thrower", "synthetic:aid_tent"],
                ["synthetic:bolt_thrower", "synthetic:supply_wagon"]
            ]
        );
        let (ruleset, setup) = synthetic(SYNTHETIC_SKIRMISH).unwrap();
        assert_eq!(ruleset.profile().as_str(), "synthetic:faithful");
        assert_eq!(ruleset.battle().army_slots, 7);
        let brute = ruleset.creature_by_key("synthetic:brute").unwrap();
        assert_eq!(
            setup.attacker.stacks[1],
            ArmyStack {
                slot: 1,
                creature: brute,
                count: 10
            }
        );
        let spearman = ruleset
            .creature(ruleset.creature_by_key("synthetic:spearman").unwrap())
            .unwrap();
        assert_eq!(
            (
                spearman.damage_min,
                spearman.damage_max,
                spearman.hit_points
            ),
            (1, 3, 10)
        );
        assert!(compile(&[SYNTHETIC_PACK], "synthetic:strict-fixed").is_ok());
    }

    #[test]
    fn strict_parsing() {
        let unknown_field = SYNTHETIC_PACK.replace("speed = 4", "speed = 4\nflying = true");
        assert!(matches!(
            parse_pack(&unknown_field),
            Err(PackError::Syntax(_))
        ));
        let out_of_range = SYNTHETIC_PACK.replace("speed = 4", "speed = 400");
        assert!(matches!(
            parse_pack(&out_of_range),
            Err(PackError::Syntax(_))
        ));
        let bad_name = SYNTHETIC_PACK.replace("id = \"brute\"", "id = \"Brute\"");
        assert!(matches!(parse_pack(&bad_name), Err(PackError::Key(_))));
        let bad_profile = SYNTHETIC_PACK.replace("id = \"faithful\"", "id = \"Faithful\"");
        assert!(matches!(parse_pack(&bad_profile), Err(PackError::Key(_))));
        let quirky = SYNTHETIC_PACK.replace(
            "quirks = { \"quirk.battle.no_luck_without_hero\" = true, \
             \"quirk.battle.single_precision_rates\" = true, \
             \"quirk.battle.unlimited_ballista_shots\" = true }",
            "quirks = { \"quirk.x\" = true }",
        );
        let pack = parse_pack(&quirky).unwrap();
        assert_eq!(pack.profiles[0].quirks, vec![("quirk.x".to_owned(), true)]);
        assert!(matches!(
            compile(&[&quirky], "synthetic:faithful"),
            Err(PackError::Compile(_))
        ));
        let no_battle = parse_pack("[pack]\nid = \"x\"\nversion = \"1\"\n").unwrap();
        assert!(no_battle.battle.is_none() && no_battle.creatures.is_empty());
    }

    #[test]
    fn scenario_errors() {
        assert!(matches!(
            parse_scenario("nonsense = ["),
            Err(PackError::Syntax(_))
        ));
        let bad_profile = SYNTHETIC_DUEL.replace("synthetic:faithful", "faithful");
        assert!(matches!(
            parse_scenario(&bad_profile),
            Err(PackError::Key(_))
        ));
        let unknown = SYNTHETIC_DUEL.replace("synthetic:spearman", "synthetic:dragon");
        assert_eq!(
            synthetic(&unknown),
            Err(PackError::UnknownCreature("synthetic:dragon".to_owned()))
        );
        let unknown_attacker = SYNTHETIC_DUEL.replace("synthetic:brute", "synthetic:dragon");
        assert!(synthetic(&unknown_attacker).is_err());
        let empty =
            parse_scenario("[scenario]\nid = \"e\"\npacks = []\nprofile = \"a:b\"\n").unwrap();
        assert_eq!(empty.id, "e");
        assert!(empty.packs.is_empty());
        assert_eq!(parse_scenario(SYNTHETIC_DUEL).unwrap().packs, ["synthetic"]);
        let header = "[scenario]\nid = \"e\"\npacks = []\nprofile = \"a:b\"\n";
        for (extra, error) in [
            (
                "random_obstacles = true\n",
                "random_obstacles needs a terrain",
            ),
            (
                "random_obstacles = true\nterrain = \"a:t\"\nobstacles = [40]\n",
                "random_obstacles excludes an obstacles list",
            ),
        ] {
            assert_eq!(
                parse_scenario(&format!("{header}{extra}")),
                Err(PackError::Syntax(error.to_owned()))
            );
        }
        let duel = parse_scenario(SYNTHETIC_DUEL).unwrap();
        assert!(duel.random_obstacles);
        let (ruleset, setup) = synthetic(SYNTHETIC_DUEL).unwrap();
        assert_eq!(setup.terrain, ruleset.terrain_by_key("synthetic:meadow"));
        let elsewhere = SYNTHETIC_DUEL.replace("synthetic:meadow", "synthetic:moon");
        assert_eq!(
            synthetic(&elsewhere),
            Err(PackError::UnknownTerrain("synthetic:moon".to_owned()))
        );
        for (sides, err) in [
            ("[\"attacker\", \"defender\"]", None),
            ("[\"left\"]", Some("unknown side `left`")),
            (
                "[\"defender\", \"defender\"]",
                Some("side `defender` listed twice"),
            ),
        ] {
            let text = format!("{header}tight_formation = {sides}\n");
            match err {
                None => assert_eq!(
                    parse_scenario(&text).unwrap().formations,
                    [Formation::Tight; 2]
                ),
                Some(e) => assert_eq!(parse_scenario(&text), Err(PackError::Syntax(e.to_owned()))),
            }
        }
        assert_eq!(
            side_names([false, true]),
            ["defender"],
            "inverse of parse_sides"
        );
        assert_eq!(side_names([true, false]), ["attacker"]);
        let tight = parse_scenario(&format!("{header}tight_formation = [\"defender\"]\n")).unwrap();
        assert_eq!(tight.formations, [Formation::Loose, Formation::Tight]);
        assert_eq!(tight_sides(tight.formations), ["defender"]);
        assert_eq!(
            formations(&["x".to_owned()]),
            Err(PackError::Syntax("unknown side `x`".to_owned()))
        );
    }

    // covers: BAT-HER-001
    #[test]
    fn scenario_heroes() {
        let (ruleset, setup) = synthetic(SYNTHETIC_FULL).unwrap();
        let brea = ruleset.hero_type_by_key("synthetic:brea").unwrap();
        let mut expected = BattleHero::new(brea);
        [
            expected.attack,
            expected.defense,
            expected.power,
            expected.knowledge,
        ] = [2, 2, 1, 1];
        expected.tactics = 2;
        expected.magic.mana = 10;
        expected.magic.max_mana = 10;
        expected.damage = DamageBonuses {
            ranged_bp: 1000,
            melee_bp: 1000,
            reduction_bp: 500,
        };
        expected.war_machines.ballista = Some(BallistaControl {
            shots: 1,
            double_damage_chance_bp: 5000,
        });
        expected.war_machines.tent = Some(TentControl {
            heal_bonus_bp: 10_000,
        });
        expected.effects = vec![
            BattleEffect::Morale {
                amount: 1,
                affects: Affects::Own,
            },
            BattleEffect::Luck {
                amount: 1,
                affects: Affects::Own,
            },
        ];
        assert_eq!(setup.heroes, [Some(expected.clone()), None]);
        // Fields default to 0; the entry is the inverse of the resolution.
        let header = "[scenario]\nid = \"e\"\npacks = []\nprofile = \"a:b\"\n";
        let text = format!("{header}[scenario.defender_hero]\nhero_type = \"synthetic:ansel\"\n");
        let entry = parse_scenario(&text).unwrap().heroes[1].clone().unwrap();
        let ansel = entry.resolve(&ruleset).unwrap();
        assert_eq!(
            ansel,
            BattleHero::new(ruleset.hero_type_by_key("synthetic:ansel").unwrap())
        );
        assert_eq!(HeroEntry::of(&ansel, &ruleset), Ok(entry));
        assert_eq!(
            HeroEntry::of(&expected, &ruleset)
                .unwrap()
                .resolve(&ruleset),
            Ok(expected.clone())
        );
        // Unknown hero types and fields.
        let unknown = SYNTHETIC_FULL.replace("synthetic:brea", "synthetic:nobody");
        assert_eq!(
            synthetic(&unknown),
            Err(PackError::UnknownHeroType("synthetic:nobody".to_owned()))
        );
        let extra = SYNTHETIC_FULL.replace("max_mana = 10", "max_mana = 10\nluck = 1");
        assert!(matches!(parse_scenario(&extra), Err(PackError::Syntax(_))));
        for bad in [
            "kind = \"no_retreat\"",
            "kind = \"luck\", amount = 1, value = 2",
        ] {
            let text = format!(
                "{header}[scenario.defender_hero]\nhero_type = \"synthetic:ansel\"\neffects = [{{ {bad} }}]\n"
            );
            assert!(
                matches!(parse_scenario(&text), Err(PackError::Syntax(_))),
                "{bad}"
            );
        }
        // Every morale and luck kind for every side survives the round trip, written as a list of
        // tables after the values.
        let mut every = expected.clone();
        every.effects = Affects::ALL
            .into_iter()
            .flat_map(|affects| {
                [
                    BattleEffect::Morale {
                        amount: -2,
                        affects,
                    },
                    BattleEffect::Luck { amount: 3, affects },
                    BattleEffect::MoraleAtMost { value: 0, affects },
                    BattleEffect::LuckAtMost { value: 1, affects },
                ]
            })
            .collect();
        let entry = HeroEntry::of(&every, &ruleset).unwrap();
        assert_eq!(entry.resolve(&ruleset), Ok(every));
        let written = toml::to_string(&entry).unwrap();
        assert!(
            written.contains("[[effects]]\nkind = \"morale_at_most\""),
            "{written}"
        );
        assert_eq!(toml::from_str::<HeroEntry>(&written), Ok(entry));
        // Damage bonuses and the Ballista's control are written only when set.
        assert!(written.contains("ranged_bp = 1000\nmelee_bp = 1000\nreduction_bp = 500\n"));
        assert!(
            written.contains("[ballista]\nshots = 1\ndouble_damage_chance_bp = 5000\n"),
            "{written}"
        );
        assert!(
            written.contains("[tent]\nheal_bonus_bp = 10000\n"),
            "{written}"
        );
        let plain = toml::to_string(&HeroEntry::of(&ansel, &ruleset).unwrap()).unwrap();
        assert!(
            !plain.contains("_bp") && !plain.contains("ballista") && !plain.contains("tent"),
            "{plain}"
        );
        let mut stranger = expected.clone();
        stranger.hero_type = h3_content::HeroTypeId::from_raw(99);
        assert_eq!(
            HeroEntry::of(&stranger, &ruleset),
            Err(PackError::UnknownHeroType("#99".to_owned()))
        );
        // Values the format cannot hold yet.
        type Change = (fn(&mut BattleHero), &'static str);
        let changes: [Change; 4] = [
            (|h| h.surrender_discount_bp = 1, "surrender discount"),
            (
                |h| h.effects.push(BattleEffect::NoRetreat),
                "effects other than morale and luck",
            ),
            (
                |h| h.magic.spell_book = Some(vec![]),
                "spell book or schools",
            ),
            (|h| h.magic.schools[0] = 1, "spell book or schools"),
        ];
        for (change, what) in changes {
            let mut hero = expected.clone();
            change(&mut hero);
            assert_eq!(
                HeroEntry::of(&hero, &ruleset),
                Err(PackError::HeroNotWritable(what))
            );
        }
    }

    #[test]
    fn format_v2_fields_and_expansion_packs() {
        let base = compile(&[SYNTHETIC_PACK], "synthetic:faithful").unwrap();
        assert_eq!(base.resources().len(), 2);
        assert_eq!(base.factions()[0].name, "Vale");
        // Terrain content (E03): terrains, native terrain, obstacles.
        let meadow = base.terrain_by_key("synthetic:meadow").unwrap();
        assert_eq!(base.terrains().len(), 2);
        assert_eq!(base.factions()[0].native_terrain, Some(meadow));
        let obstacles: Vec<(&str, bool, usize)> = base
            .obstacles()
            .iter()
            .map(|o| (o.key.as_str(), o.absolute, o.blocked.len()))
            .collect();
        assert_eq!(
            obstacles,
            [
                ("synthetic:boulder", false, 2),
                ("synthetic:crag", false, 3),
                ("synthetic:ridge", true, 4)
            ]
        );
        assert_eq!(base.obstacles()[1].blocked[2], (1, -1));
        let bad_ref =
            SYNTHETIC_PACK.replace("native_terrain = \"meadow\"", "native_terrain = \"Meadow\"");
        assert!(matches!(parse_pack(&bad_ref), Err(PackError::Key(_))));
        let bad_obstacle = SYNTHETIC_PACK.replace("id = \"boulder\"", "id = \"Boulder\"");
        assert!(matches!(parse_pack(&bad_obstacle), Err(PackError::Key(_))));
        let bad_terrain =
            SYNTHETIC_PACK.replace("terrains = [\"meadow\"]", "terrains = [\"Meadow\"]");
        assert!(matches!(parse_pack(&bad_terrain), Err(PackError::Key(_))));
        let bad_faction = SYNTHETIC_PACK.replace("id = \"vale\"", "id = \"Vale\"");
        assert!(matches!(parse_pack(&bad_faction), Err(PackError::Key(_))));
        let brute = base.creature_by_key("synthetic:brute").unwrap();
        let info = base.creature_info(brute).unwrap();
        assert_eq!(
            (info.name.as_str(), info.plural.as_str()),
            ("Brute", "Brutes")
        );
        assert_eq!(info.level, 2);
        // Cost entries come in key order (`coin` before `shard`).
        let costs: Vec<(&str, u32)> = info
            .cost
            .iter()
            .map(|(r, n)| (base.resources()[r.index()].key.as_str(), *n))
            .collect();
        assert_eq!(costs, [("synthetic:coin", 120), ("synthetic:shard", 1)]);
        assert!(
            base.creature(brute)
                .unwrap()
                .flags
                .contains(CreatureFlag::TwoHex)
        );
        let wisp = base.creature_by_key("synthetic:wisp").unwrap();
        assert_eq!(base.creature_info(wisp).unwrap().faction, None);

        // The expansion may be listed first; the compiler loads its dependency first.
        let both = compile(&[SYNTHETIC_EXT_PACK, SYNTHETIC_PACK], "synthetic:faithful").unwrap();
        let ids: Vec<&str> = both.packs().iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(ids, ["synthetic", "synthetic-ext"]);
        assert_eq!(both.creature_count(), 7);
        let wisp_info = both.creature_info(wisp).unwrap();
        let marsh = wisp_info.faction.unwrap();
        assert_eq!(
            both.factions()[marsh.index()].key.as_str(),
            "synthetic-ext:marsh"
        );
        assert_eq!(wisp_info.level, 2);
        let brute_info = both.creature_info(brute).unwrap();
        assert_eq!((brute_info.growth, brute_info.horde_growth), (8, 3));
        assert_eq!(brute_info.cost[0].1, 110);
        // Battle stats are untouched by these overrides.
        assert_eq!(both.creature(brute), base.creature(brute));

        // The expansion alone is rejected: its dependency is not enabled.
        assert_eq!(
            compile(&[SYNTHETIC_EXT_PACK], "synthetic:faithful"),
            Err(PackError::Compile(CompileError::MissingDependency {
                pack: "synthetic-ext".into(),
                depends: "synthetic".into()
            }))
        );
    }

    #[test]
    fn harvested_packs_compile_in_every_allowed_combination() {
        let profile = "base:faithful";
        let roe = compile(&[BASE_PACK], profile).unwrap();
        let ab = compile(&[BASE_PACK, AB_PACK], profile).unwrap();
        let sod = compile(&[SOD_PACK, AB_PACK, BASE_PACK], profile).unwrap();
        // RoE: 8 towns x 14 + 6 neutrals + 3 war machines; AB adds Conflux (10 new) and 13
        // neutrals.
        assert_eq!(roe.creature_count(), 121);
        assert_eq!(ab.creature_count(), 144);
        assert_eq!(sod.creature_count(), 144);
        assert_eq!((roe.factions().len(), ab.factions().len()), (8, 9));
        // SoD requires AB; AB requires base.
        assert!(matches!(
            compile(&[BASE_PACK, SOD_PACK], profile),
            Err(PackError::Compile(CompileError::MissingDependency { .. }))
        ));
        assert!(compile(&[AB_PACK], profile).is_err());
        // Spot checks of the version differences (sources: crates/h3-harvest/data/gapfill.toml).
        let gems_of = |r: &Ruleset, key: &str| {
            let info = r.creature_info(r.creature_by_key(key).unwrap()).unwrap();
            info.cost
                .iter()
                .find(|(res, _)| r.resources()[res.index()].key.as_str() == "base:gems")
                .map_or(0, |(_, n)| *n)
        };
        assert_eq!(gems_of(&ab, "base:angel"), 0);
        assert_eq!(gems_of(&sod, "base:angel"), 1);
        let air = roe.creature_by_key("base:air_elemental").unwrap();
        assert_eq!(roe.creature_info(air).unwrap().faction, None);
        assert_eq!(roe.creature_info(air).unwrap().level, 4);
        assert_eq!(ab.creature_info(air).unwrap().level, 2);
        let lizard = roe.creature_by_key("base:lizardman").unwrap();
        assert_eq!(roe.creature_info(lizard).unwrap().growth, 9);
        let cerberus = roe.creature_by_key("base:cerberus").unwrap();
        assert_eq!(roe.creature(cerberus).unwrap().damage_max, 5);
        assert_eq!(sod.creature(cerberus).unwrap().damage_max, 7);
        assert_eq!(roe.battle().attack_bonus_cap_percent, 300);
        assert_eq!(roe.battle().formation_row(Formation::Loose, 7, 3), Some(5));
        assert_eq!(roe.battle().formation_row(Formation::Tight, 3, 0), Some(4));
        // Abilities (E03 T2): RoE elementals hate their opposites; AB adds the Conflux ones.
        assert_eq!(roe.abilities(air).len(), 1);
        assert_eq!(ab.abilities(air).len(), 2);
        let cavalier = roe.creature_by_key("base:cavalier").unwrap();
        assert_eq!(
            roe.abilities(cavalier),
            [h3_content::Ability::Jousting { percent_per_hex: 5 }]
        );
        // Hero content (E04a T3): RoE has 16 classes and 128 heroes, AB adds Conflux and 28
        // heroes, SoD Titan's Lightning Bolt.
        let counts = |r: &Ruleset| {
            (
                r.skills().len(),
                r.spells().len(),
                r.hero_classes().len(),
                r.hero_types().len(),
            )
        };
        assert_eq!(counts(&roe), (28, 80, 16, 128));
        assert_eq!(counts(&ab), (28, 80, 18, 156));
        assert_eq!(counts(&sod), (28, 81, 18, 156));
        assert_eq!(sod.spell_schools().len(), 4);
        // Hero parameters and the slots whose artifacts never go to the backpack (E04a T4a).
        let params = roe.hero_params().unwrap();
        assert_eq!(
            (params.primary_min, params.primary_max, params.backpack),
            ([0, 0, 1, 1], 99, 64)
        );
        let fixed: Vec<&str> = sod
            .artifact_slots()
            .iter()
            .filter(|s| !s.backpack)
            .map(|s| s.key.as_str())
            .collect();
        assert_eq!(
            fixed,
            [
                "base:war_machine_1",
                "base:war_machine_2",
                "base:war_machine_3",
                "base:war_machine_4",
                "base:spell_book"
            ]
        );
        let archery = sod
            .skill(sod.skill_by_key("base:archery").unwrap())
            .unwrap();
        assert_eq!(archery.effects(3), [HeroEffect::RangedDamage { bp: 5000 }]);
        let pikeman = roe.creature_by_key("base:pikeman").unwrap();
        let halberdier = roe.creature_by_key("base:halberdier").unwrap();
        assert_eq!(roe.upgrades(pikeman), [halberdier]);
        assert!(roe.upgrades(air).is_empty());
        let storm = ab.creature_by_key("ab:storm_elemental").unwrap();
        assert_eq!(ab.upgrades(air), [storm]);
        let orrin = roe
            .hero_type(roe.hero_type_by_key("base:orrin").unwrap())
            .unwrap();
        assert_eq!(orrin.army.len(), 3);
        assert!(sod.hero_type_by_key("ab:lord_haart_death_knight").is_some());
        // Artifacts (E04a T3b): RoE 127 with four misc positions, AB adds two, SoD the twelve
        // combinations and a fifth misc position.
        let misc = |r: &Ruleset| {
            r.artifact_slot(r.artifact_slot_by_key("base:misc").unwrap())
                .unwrap()
                .count
        };
        assert_eq!((roe.artifacts().len(), misc(&roe)), (127, 4));
        assert_eq!((ab.artifacts().len(), misc(&ab)), (129, 4));
        assert_eq!((sod.artifacts().len(), misc(&sod)), (141, 5));
        assert_eq!(sod.artifact_slots().len(), 14);
        let slot = |k: &str| sod.artifact_slot_by_key(k).unwrap();
        let alliance = sod
            .artifact(sod.artifact_by_key("sod:angelic_alliance").unwrap())
            .unwrap();
        assert_eq!(alliance.slot, Some(slot("base:right_hand")));
        assert_eq!(
            alliance.locks,
            [
                "base:torso",
                "base:feet",
                "base:neck",
                "base:left_hand",
                "base:head"
            ]
            .map(slot)
        );
        let sword = sod.artifact_by_key("base:sword_of_judgement").unwrap();
        assert_eq!(
            sod.combination_of(sword),
            sod.artifact_by_key("sod:angelic_alliance")
        );
        let gladius = sod
            .artifact(sod.artifact_by_key("base:titans_gladius").unwrap())
            .unwrap();
        assert_eq!(
            gladius.effects,
            [HeroEffect::PrimarySkills {
                attack: 12,
                defense: -3,
                power: 0,
                knowledge: 0
            }]
        );
        let ballista = sod
            .artifact(sod.artifact_by_key("base:ballista").unwrap())
            .unwrap();
        assert_eq!(ballista.war_machine, sod.creature_by_key("base:ballista"));
        let bow = sod
            .artifact(sod.artifact_by_key("base:bow_of_elven_cherrywood").unwrap())
            .unwrap();
        assert_eq!(bow.needs_skill, sod.skill_by_key("base:archery"));
    }

    #[test]
    fn override_syntax() {
        let with = |extra: &str| std::format!("{SYNTHETIC_EXT_PACK}\n{extra}");
        let full = with(
            "[[creature_override]]\ntarget = \"synthetic:spearman\"\nattack = 9\ndefense = 9\n\
             damage = [2, 6]\nhit_points = 12\nspeed = 5\nname = \"Pike\"\nplural = \"Pikes\"\n\
             faction = \"\"\nshots = 1\ncasts = 2\nhorde_growth = 1\nfight_value = 1\n\
             ai_value = 2\nadventure_count = [1, 2]\nflags = [\"undead\"]\n",
        );
        let r = compile(&[SYNTHETIC_PACK, &full], "synthetic:faithful").unwrap();
        let id = r.creature_by_key("synthetic:spearman").unwrap();
        let stats = r.creature(id).unwrap();
        assert_eq!(
            (
                stats.attack,
                stats.damage_min,
                stats.damage_max,
                stats.speed
            ),
            (9, 2, 6, 5)
        );
        let info = r.creature_info(id).unwrap();
        assert_eq!(info.faction, None); // `faction = ""` makes it neutral
        assert_eq!(
            (info.name.as_str(), stats.shots, info.casts),
            ("Pike", 1, 2)
        );
        assert!(stats.flags.contains(CreatureFlag::Undead));

        let bad_target = with("[[creature_override]]\ntarget = \"spearman\"\n");
        assert!(matches!(parse_pack(&bad_target), Err(PackError::Key(_))));
        let bad_ref =
            with("[[creature_override]]\ntarget = \"synthetic:spearman\"\nfaction = \"A:b\"\n");
        assert!(matches!(parse_pack(&bad_ref), Err(PackError::Key(_))));
        let bad_cost =
            with("[[creature_override]]\ntarget = \"synthetic:spearman\"\ncost = { \"X\" = 1 }\n");
        assert!(matches!(parse_pack(&bad_cost), Err(PackError::Key(_))));
        let bad_resource = SYNTHETIC_PACK.replace("id = \"coin\"", "id = \"Coin\"");
        assert!(matches!(parse_pack(&bad_resource), Err(PackError::Key(_))));
        let bad_flag = SYNTHETIC_PACK.replace("flags = [\"alive\"]", "flags = [\"swims\"]");
        assert!(matches!(
            compile(&[&bad_flag], "synthetic:faithful"),
            Err(PackError::Compile(CompileError::UnknownFlag(_)))
        ));
    }

    #[test]
    fn ability_syntax() {
        use h3_content::{Ability, AreaShot, AuraTarget};
        // Every kind with its parameters; references follow the usual rules.
        let all = "abilities = [\
            { kind = \"hate\", creature = \"wisp\", percent = 50 }, \
            { kind = \"hate\", creature = \"synthetic:brute\", percent = 100 }, \
            { kind = \"jousting\", percent_per_hex = 5 }, { kind = \"charge_immunity\" }, \
            { kind = \"defense_reduction\", percent = 40 }, \
            { kind = \"death_blow\", chance_percent = 20 }, \
            { kind = \"life_drain\", percent = 100 }, { kind = \"regeneration\" }, \
            { kind = \"death_stare\", chance_percent = 10 }, \
            { kind = \"extra_retaliations\", count = 1 }, { kind = \"strike_and_return\" }, \
            { kind = \"teleports\" }, \
            { kind = \"morale_aura\", amount = 1, affects = \"allies\" }, \
            { kind = \"luck_aura\", amount = -1, affects = \"enemies\" }, \
            { kind = \"minimum_morale\", value = 1 }, { kind = \"minimum_luck\", value = 1 }, \
            { kind = \"no_distance_penalty\" }, { kind = \"first_aid\", min_hp = 1, max_hp = 25 }, \
            { kind = \"ammo_supply\" }]\n";
        // The war-machine abilities need flag war_machine.
        let text = SYNTHETIC_PACK.replace(
            "flags = [\"alive\"]\n",
            &std::format!("flags = [\"alive\"]\n{all}"),
        );
        assert!(matches!(
            compile(&[&text], "synthetic:faithful"),
            Err(PackError::Compile(CompileError::InvalidCreature { reason, .. }))
                if reason == "ammo_supply and first_aid require flag war_machine"
        ));
        // (War machines have speed 0, BAT-WAR-001.)
        let text = SYNTHETIC_PACK
            .replace(
                "flags = [\"alive\"]\n",
                &std::format!("flags = [\"alive\", \"war_machine\"]\n{all}"),
            )
            .replace(
                "hit_points = 10\nspeed = 4\n",
                "hit_points = 10\nspeed = 0\n",
            );
        let r = compile(&[&text], "synthetic:faithful").unwrap();
        let spearman = r.creature_by_key("synthetic:spearman").unwrap();
        let wisp = r.creature_by_key("synthetic:wisp").unwrap();
        let list = r.abilities(spearman);
        assert_eq!(list.len(), 19);
        assert_eq!(
            list[17],
            Ability::FirstAid {
                min_hp: 1,
                max_hp: 25
            }
        );
        assert_eq!(list[18], Ability::AmmoSupply);
        assert_eq!(
            list[0],
            Ability::Hate {
                target: wisp,
                percent: 50
            }
        );
        assert_eq!(
            list[13],
            Ability::LuckAura {
                amount: -1,
                affects: AuraTarget::Enemies
            }
        );
        // Patterns and area shots need their flags; overrides replace the whole list.
        let ext = std::format!(
            "{SYNTHETIC_EXT_PACK}\n[[creature_override]]\ntarget = \"synthetic:wisp\"\n\
             flags = [\"multi_headed\", \"area_attack\"]\nabilities = [\
             {{ kind = \"attacks_all_adjacent\" }}, {{ kind = \"area_shot\", effect = \"death_cloud\" }}]\n\
             [[creature_override]]\ntarget = \"synthetic:brute\"\nflags = [\"multi_headed\", \"area_attack\"]\n\
             abilities = [{{ kind = \"three_headed_attack\" }}, {{ kind = \"unlimited_retaliations\" }}, \
             {{ kind = \"area_shot\", effect = \"fireball\" }}]\n"
        );
        let r = compile(&[SYNTHETIC_PACK, &ext], "synthetic:faithful").unwrap();
        assert_eq!(
            r.abilities(wisp),
            [
                Ability::AttacksAllAdjacent,
                Ability::AreaShot {
                    effect: AreaShot::DeathCloud
                }
            ]
        );
        let brute = r.creature_by_key("synthetic:brute").unwrap();
        assert_eq!(r.abilities(brute).len(), 3);
        // Strict parsing: unknown kinds, unknown or missing parameters, bad references.
        for bad in [
            "abilities = [{ kind = \"swims\" }]",
            "abilities = [{ kind = \"teleports\", range = 3 }]",
            "abilities = [{ kind = \"jousting\" }]",
            "abilities = [{ kind = \"morale_aura\", amount = 1, affects = \"everyone\" }]",
            "abilities = [{ kind = \"area_shot\", effect = \"lightning\" }]",
        ] {
            let text = SYNTHETIC_PACK.replace(
                "flags = [\"alive\"]\n",
                &std::format!("flags = [\"alive\"]\n{bad}\n"),
            );
            assert!(
                matches!(parse_pack(&text), Err(PackError::Syntax(_))),
                "{bad}"
            );
        }
        let bad_ref = SYNTHETIC_PACK.replace(
            "flags = [\"alive\"]\n",
            "flags = [\"alive\"]\nabilities = [{ kind = \"hate\", creature = \"A:b\", percent = 1 }]\n",
        );
        assert!(matches!(parse_pack(&bad_ref), Err(PackError::Key(_))));
        let bad_override = std::format!(
            "{SYNTHETIC_EXT_PACK}\n[[creature_override]]\ntarget = \"synthetic:wisp\"\n\
             abilities = [{{ kind = \"hate\", creature = \"A:b\", percent = 1 }}]\n"
        );
        assert!(matches!(parse_pack(&bad_override), Err(PackError::Key(_))));
    }

    // covers: HER-ART-001, HER-ART-002, HER-WAR-001, HER-BAT-001, HER-BAT-002, HER-BAT-003
    // covers: HER-BAT-004, HER-BAT-005
    #[test]
    fn hero_details_fit_the_harvested_slots() {
        use h3_heroes::{DetailsError, HeroDetails, derive_battle_hero, validate, war_machines};
        let profile = "base:faithful";
        let ab = compile(&[BASE_PACK, AB_PACK], profile).unwrap();
        let sod = compile(&[SOD_PACK, AB_PACK, BASE_PACK], profile).unwrap();
        let details = |r: &Ruleset, worn: &[&str], backpack: &[&str]| {
            let art = |k: &&str| r.artifact_by_key(k).unwrap();
            HeroDetails {
                hero_type: r.hero_type_by_key("base:orrin").unwrap(),
                level: 10,
                experience: 20_000,
                primary: [5, 4, 2, 2],
                skills: vec![(r.skill_by_key("base:archery").unwrap(), 3)],
                worn: worn.iter().map(art).collect(),
                backpack: backpack.iter().map(art).collect(),
                spells: vec![r.spell_by_key("base:magic_arrow").unwrap()],
                mana: 20,
            }
        };
        let machines = [
            "base:ballista",
            "base:ammo_cart",
            "base:first_aid_tent",
            "base:catapult",
            "base:spell_book",
        ];
        let misc = ["base:clover_of_fortune"; 5];
        let rings = ["base:quiet_eye_of_the_dragon"; 2];
        // SoD: Angelic Alliance (right hand) with five misc artifacts, both rings, every
        // machine and the book; the Grail in the backpack.
        let full: Vec<&str> = ["sod:angelic_alliance"]
            .into_iter()
            .chain(misc)
            .chain(rings)
            .chain(machines)
            .collect();
        let d = details(&sod, &full, &["base:the_grail"]);
        assert_eq!(validate(&d, &sod), Ok(()));
        let creature = |k: &str| sod.creature_by_key(k).unwrap();
        assert_eq!(
            war_machines(&d, &sod),
            [
                creature("base:ballista"),
                creature("base:ammo_cart"),
                creature("base:first_aid_tent")
            ]
        );
        // In battle the Alliance's six parts give +21 to every primary skill (wiki Angelic
        // Alliance), the two Quiet Eyes +1 attack and +1 defense each; Tactics from the skill.
        let hero = derive_battle_hero(&d, &sod).unwrap();
        assert_eq!(
            [hero.attack, hero.defense, hero.power, hero.knowledge],
            [28, 27, 23, 23]
        );
        assert_eq!(hero.tactics, 0);
        // Orrin's specialty grows Expert Archery's 50 % by 5 % of itself per level: 75 % at
        // level 10 (wiki Hero specialty); the Bow of the Sharpshooter's three parts add 30 %
        // (HER-BAT-004).
        assert_eq!(
            hero.damage,
            DamageBonuses {
                ranged_bp: 7500,
                ..DamageBonuses::default()
            }
        );
        let bow = details(&sod, &["sod:bow_of_the_sharpshooter"], &[]);
        assert_eq!(
            derive_battle_hero(&bow, &sod).unwrap().damage.ranged_bp,
            10_500
        );
        // Artillery: Basic one shot at 50 % double damage, Expert two at 100 % (batch 1a R6, R7;
        // HER-BAT-005); without it Orrin does not command his Ballista.
        assert_eq!(hero.war_machines.ballista, None);
        let mut gunner = details(&sod, &["base:ballista"], &[]);
        let artillery = sod.skill_by_key("base:artillery").unwrap();
        for (level, shots, chance) in [(1, 1, 5000), (3, 2, 10_000)] {
            gunner.skills = vec![(artillery, level)];
            assert_eq!(
                derive_battle_hero(&gunner, &sod)
                    .unwrap()
                    .war_machines
                    .ballista,
                Some(BallistaControl {
                    shots,
                    double_damage_chance_bp: chance
                })
            );
        }
        // First Aid: the tent's largest heal +100/200/300 %; Rion, a First Aid specialist, at
        // level 10 with Expert First Aid: 300 % × 1.5 (the executable: 25 × (1 + 3 × 1.5), 137).
        assert_eq!(hero.war_machines.tent, None);
        let first_aid = sod.skill_by_key("base:first_aid").unwrap();
        gunner.skills = vec![(first_aid, 1)];
        let tent = |d: &HeroDetails| derive_battle_hero(d, &sod).unwrap().war_machines.tent;
        assert_eq!(
            tent(&gunner),
            Some(TentControl {
                heal_bonus_bp: 10_000
            })
        );
        let rion = HeroDetails {
            hero_type: sod.hero_type_by_key("base:rion").unwrap(),
            skills: vec![(first_aid, 3)],
            ..gunner.clone()
        };
        assert_eq!(
            tent(&rion),
            Some(TentControl {
                heal_bonus_bp: 45_000
            })
        );
        // Five Clovers of Fortune give luck +1: identical artifacts do not stack (wiki Artifact).
        let luck = |amount| BattleEffect::Luck {
            amount,
            affects: Affects::Own,
        };
        assert_eq!(hero.effects, [luck(1)]);
        // Expert Leadership (+3), Pendant of Courage (+3 each), Badge of Courage and two Clovers
        // (+1 each, once); Spirit of Oppression caps both sides' morale at 0 (the battle clamps
        // the sums to +3).
        let mut leader = details(
            &sod,
            &[
                "base:spirit_of_oppression",
                "base:pendant_of_courage",
                "base:badge_of_courage",
                "base:clover_of_fortune",
                "base:clover_of_fortune",
            ],
            &[],
        );
        leader
            .skills
            .push((sod.skill_by_key("base:leadership").unwrap(), 3));
        assert_eq!(
            derive_battle_hero(&leader, &sod).unwrap().effects,
            [
                BattleEffect::Morale {
                    amount: 7,
                    affects: Affects::Own
                },
                luck(4),
                BattleEffect::MoraleAtMost {
                    value: 0,
                    affects: Affects::Both
                },
            ]
        );
        let mut tactician = d.clone();
        tactician
            .skills
            .push((sod.skill_by_key("base:tactics").unwrap(), 2));
        assert_eq!(derive_battle_hero(&tactician, &sod).unwrap().tactics, 2);
        // The Alliance locks the head; machines and the book stay out of the backpack.
        let head = details(
            &sod,
            &[&full[..], &["base:helm_of_heavenly_enlightenment"]].concat(),
            &[],
        );
        assert_eq!(
            validate(&head, &sod),
            Err(DetailsError::SlotFull(
                sod.artifact_slot_by_key("base:head").unwrap()
            ))
        );
        for k in machines {
            let d = details(&sod, &[], &[k]);
            assert_eq!(
                validate(&d, &sod),
                Err(DetailsError::NotForBackpack(
                    sod.artifact_by_key(k).unwrap()
                ))
            );
        }
        let crowded = details(&sod, &[], &["base:clover_of_fortune"; 65]);
        assert_eq!(validate(&crowded, &sod), Err(DetailsError::BackpackFull));
        // Before SoD a hero has four misc positions.
        let five = details(&ab, &misc, &[]);
        assert_eq!(
            validate(&five, &ab),
            Err(DetailsError::SlotFull(
                ab.artifact_slot_by_key("base:misc").unwrap()
            ))
        );
        assert_eq!(validate(&details(&ab, &misc[1..], &[]), &ab), Ok(()));
    }

    #[test]
    fn artifact_syntax() {
        use h3_content::ArtifactClass;
        let r = compile(&[SYNTHETIC_PACK, SYNTHETIC_EXT_PACK], "synthetic:faithful").unwrap();
        assert_eq!(r.artifact_slots().len(), 4);
        let charm = r.artifact_slot_by_key("synthetic:charm").unwrap();
        let crown = r.artifact_slot_by_key("synthetic:crown").unwrap();
        assert_eq!(r.artifact_slot(charm).unwrap().count, 3);
        let engine = r.artifact_slot_by_key("synthetic:engine").unwrap();
        assert!(r.artifact_slot(charm).unwrap().backpack);
        assert!(!r.artifact_slot(engine).unwrap().backpack);
        let params = r.hero_params().unwrap();
        assert_eq!(
            (params.primary_min, params.primary_max, params.backpack),
            ([0, 0, 1, 2], 40, 3)
        );
        assert_eq!(r.artifacts().len(), 10);
        let by_key = |k: &str| r.artifact(r.artifact_by_key(k).unwrap()).unwrap();
        let regalia = by_key("synthetic-ext:storm_regalia");
        assert_eq!(regalia.class, ArtifactClass::Relic);
        assert_eq!(regalia.locks, vec![charm, charm, crown]);
        assert_eq!(regalia.parts.len(), 4);
        let lens = by_key("synthetic:sight_lens");
        assert_eq!(lens.needs_skill, r.skill_by_key("synthetic:marksmanship"));
        assert_eq!(lens.cost.len(), 1);
        assert_eq!(
            by_key("synthetic:bolt_thrower_kit").war_machine,
            r.creature_by_key("synthetic:bolt_thrower")
        );
        assert_eq!(by_key("synthetic:relic_stone").slot, None);
        assert_eq!(by_key("synthetic:vigor_stone").cost.len(), 2);
        // Without the expansion the slot keeps its own count.
        let base = compile(&[SYNTHETIC_PACK], "synthetic:faithful").unwrap();
        assert_eq!(base.artifact_slot(charm).unwrap().count, 2);

        // Strict syntax and malformed references at every artifact site.
        let bad_syntax = [
            (
                "class = \"relic\"\ncost = { coin = 4000 }",
                "class = \"epic\"\ncost = { coin = 4000 }",
            ),
            ("count = 2   ", "count = \"two\""),
            ("id = \"engine\"", "id = \"engine\"\nsize = 1"),
            ("slot = \"engine\"", "slot = \"engine\"\nbroken = true"),
            (
                "{ kind = \"no_retreat\" }",
                "{ kind = \"no_retreat\", sides = 2 }",
            ),
        ];
        for (from, to) in bad_syntax {
            assert!(SYNTHETIC_PACK.contains(from), "{from}");
            let text = SYNTHETIC_PACK.replacen(from, to, 1);
            assert!(
                matches!(parse_pack(&text), Err(PackError::Syntax(_))),
                "{to}"
            );
        }
        let bad_refs = [
            ("id = \"crown\"", "id = \"Crown\""),
            ("id = \"relic_stone\"", "id = \"Relic\""),
            ("cost = { coin = 4000 }", "cost = { \"C:x\" = 4000 }"),
            ("slot = \"engine\"", "slot = \"E:x\""),
            ("war_machine = \"bolt_thrower\"", "war_machine = \"B:x\""),
            ("needs_skill = \"marksmanship\"", "needs_skill = \"M:x\""),
            (
                "{ kind = \"no_range_penalty\", creatures = { flag = \"shooter\" } }",
                "{ kind = \"no_range_penalty\", creatures = { creature = \"S:x\" } }",
            ),
        ];
        for (from, to) in bad_refs {
            assert!(SYNTHETIC_PACK.contains(from), "{from}");
            let text = SYNTHETIC_PACK.replacen(from, to, 1);
            assert!(matches!(parse_pack(&text), Err(PackError::Key(_))), "{to}");
        }
        let ext_refs = [
            ("target = \"synthetic:charm\"", "target = \"T:x\""),
            (
                "\"synthetic:lucky_knot\", \"synthetic",
                "\"L:x\", \"synthetic",
            ),
            ("\"synthetic:vale\", \"marsh\"", "\"V:x\", \"marsh\""),
        ];
        for (from, to) in ext_refs {
            assert!(SYNTHETIC_EXT_PACK.contains(from), "{from}");
            let text = SYNTHETIC_EXT_PACK.replacen(from, to, 1);
            assert!(matches!(parse_pack(&text), Err(PackError::Key(_))), "{to}");
        }
    }

    #[test]
    fn hero_content_syntax() {
        use h3_content::SpellCategory;
        let r = compile(&[SYNTHETIC_PACK, SYNTHETIC_EXT_PACK], "synthetic:faithful").unwrap();
        assert_eq!(r.spell_schools().len(), 2);
        assert_eq!(r.spells().len(), 4);
        let ward = r.spell(r.spell_by_key("synthetic:ward").unwrap()).unwrap();
        assert_eq!((ward.level, ward.schools.len(), ward.power), (2, 2, 0));
        assert_eq!(ward.polarity, SpellPolarity::Positive);
        let glare = r.spell(r.spell_by_key("synthetic:glare").unwrap()).unwrap();
        assert_eq!(glare.category, SpellCategory::Ability);
        assert_eq!(r.skills().len(), 10);
        let omens = r.skill(r.skill_by_key("synthetic:omens").unwrap()).unwrap();
        assert_eq!(
            omens.effects(3)[1],
            HeroEffect::Luck {
                amount: -1,
                affects: Affects::Enemy
            }
        );
        assert_eq!(r.hero_classes().len(), 2);
        assert_eq!(r.hero_classes()[1].skill_weights.len(), 4);
        assert_eq!(r.hero_types().len(), 5);
        let wisp = r.creature_by_key("synthetic:wisp").unwrap();
        let corwin = r
            .hero_type(r.hero_type_by_key("synthetic:corwin").unwrap())
            .unwrap();
        assert_eq!(
            corwin.specialty[0],
            HeroEffect::CreatureStat {
                creatures: CreatureSelector::Creature(wisp),
                stat: CreatureStat::Attack,
                amount: 2
            }
        );
        assert_eq!(
            corwin.specialty[3],
            HeroEffect::CreatureStat {
                creatures: CreatureSelector::Flag(CreatureFlag::Flying),
                stat: CreatureStat::Defense,
                amount: 1
            }
        );
        let dara = r
            .hero_type(r.hero_type_by_key("synthetic:dara").unwrap())
            .unwrap();
        assert!(dara.spell_book && dara.spells.len() == 1 && dara.war_machines.len() == 1);
        let lurker = r.creature_by_key("synthetic-ext:lurker").unwrap();
        assert_eq!(r.upgrades(wisp), &[lurker]);
        // Every effect kind parses (the synthetic pack uses each at least once).
        let kinds: std::collections::BTreeSet<&str> = r
            .skills()
            .iter()
            .flat_map(|s| s.levels.iter().flatten())
            .chain(r.hero_types().iter().flat_map(|h| &h.specialty))
            .chain(r.artifacts().iter().flat_map(|a| &a.effects))
            .map(HeroEffect::name)
            .collect();
        assert_eq!(kinds.len(), HeroEffect::<u8, u8, u8, u8>::NAMES.len());

        // Strict syntax: unknown kinds, parameters, flags and selectors are errors.
        let bad_syntax = [
            (
                "{ kind = \"ranged_damage\", bp = 1000 }]",
                "{ kind = \"ranged\", bp = 1000 }]",
            ),
            (
                "{ kind = \"ranged_damage\", bp = 1000 }]",
                "{ kind = \"ranged_damage\", percent = 10 }]",
            ),
            (
                "creatures = { flag = \"flying\" }",
                "creatures = { flag = \"flying\", creature = \"wisp\" }",
            ),
            (
                "creatures = { flag = \"flying\" }",
                "creatures = { kind = \"flying\" }",
            ),
            ("category = \"combat\"", "category = \"battle\""),
            ("polarity = \"neutral\"", "polarity = \"good\""),
            ("affects = \"enemy\"", "affects = \"allies\""),
            ("stat = \"speed\"", "stat = \"luck\""),
            (
                "level = 1 }, { skill = \"marksmanship\"",
                "lvl = 1 }, { skill = \"marksmanship\"",
            ),
            ("count = [4, 7] }", "count = [4, 7], slot = 1 }"),
            (
                "faction = \"vale\"\nprimary",
                "faction = \"vale\"\nprimary_skills = [1, 1, 1, 1]\nprimary",
            ),
        ];
        for (from, to) in bad_syntax {
            assert!(SYNTHETIC_PACK.contains(from), "{from}");
            let text = SYNTHETIC_PACK.replacen(from, to, 1);
            assert!(
                matches!(parse_pack(&text), Err(PackError::Syntax(_))),
                "{to}"
            );
        }
        let unknown_flag = SYNTHETIC_PACK.replace(
            "creatures = { flag = \"flying\" }",
            "creatures = { flag = \"winged\" }",
        );
        assert_eq!(
            parse_pack(&unknown_flag),
            Err(PackError::Compile(CompileError::UnknownFlag(
                "winged".into()
            )))
        );
        // Malformed references at every reference site.
        let bad_refs = [
            ("schools = [\"storm\"]", "schools = [\"S:x\"]"),
            (
                "{ kind = \"ranged_damage\", bp = 1000 }]",
                "{ kind = \"creature_stat\", creatures = { creature = \"W:x\" }, stat = \"speed\", amount = 1 }]",
            ),
            (
                "skill = \"marksmanship\", bp_per_level",
                "skill = \"M:x\", bp_per_level",
            ),
            (
                "creature = \"brute\", bp_per_step",
                "creature = \"B:x\", bp_per_step",
            ),
            ("spell = \"spark\" }", "spell = \"S:x\" }"),
            ("faction = \"vale\"\nprimary", "faction = \"V:x\"\nprimary"),
            ("marksmanship = 6", "\"M:x\" = 6"),
            ("class = \"warden\"", "class = \"W:x\""),
            ("{ skill = \"command\"", "{ skill = \"C:x\""),
            ("spells = [\"spark\"]", "spells = [\"S:x\"]"),
            (
                "{ creature = \"brute\", count",
                "{ creature = \"B:x\", count",
            ),
            ("war_machines = [\"aid_tent\"]", "war_machines = [\"A:x\"]"),
            ("id = \"ansel\"", "id = \"Ansel\""),
            ("id = \"warden\"", "id = \"Warden\""),
            ("id = \"drill\"", "id = \"Drill\""),
            ("id = \"spark\"", "id = \"Spark\""),
            ("id = \"storm\"", "id = \"Storm\""),
        ];
        for (from, to) in bad_refs {
            assert!(SYNTHETIC_PACK.contains(from), "{from}");
            let text = SYNTHETIC_PACK.replacen(from, to, 1);
            assert!(matches!(parse_pack(&text), Err(PackError::Key(_))), "{to}");
        }
        let bad_upgrade =
            SYNTHETIC_EXT_PACK.replace("upgrades = [\"lurker\"]", "upgrades = [\"L:x\"]");
        assert!(matches!(parse_pack(&bad_upgrade), Err(PackError::Key(_))));
        let bad_upgrade = SYNTHETIC_PACK.replacen(
            "flags = [\"alive\"]\n",
            "flags = [\"alive\"]\nupgrades = [\"U:x\"]\n",
            1,
        );
        assert!(matches!(parse_pack(&bad_upgrade), Err(PackError::Key(_))));
    }

    #[test]
    fn error_display() {
        assert_eq!(PackError::Syntax("x".into()).to_string(), "syntax: x");
        assert_eq!(
            PackError::UnknownCreature("a:b".into()).to_string(),
            "unknown creature `a:b`"
        );
        let key = ContentKey::parse("bad").unwrap_err();
        assert!(PackError::from(key).to_string().contains("not a valid key"));
        assert_eq!(
            PackError::from(CompileError::MissingBattleParams).to_string(),
            "no pack provides battle parameters"
        );
        assert_eq!(
            PackError::UnknownTerrain("a:t".into()).to_string(),
            "unknown terrain `a:t`"
        );
        assert_eq!(
            PackError::UnknownHeroType("a:h".into()).to_string(),
            "unknown hero type `a:h`"
        );
        assert_eq!(
            PackError::HeroNotWritable("effects").to_string(),
            "the file format cannot hold a hero's effects yet"
        );
    }
}
