//! Adapter: parses the core part of human-editable content packs (`core.toml`, pack format v35,
//! R01) into content definitions (ADR-0006): the `[pack]` manifest, resources, factions,
//! terrains, creatures (their core data), spell schools, spells (their core data), the hero
//! content and the profiles. The battle part (`battle.toml`, the battle scenarios) is
//! `h3-pack-battle`'s, the adventure part (`adventure.toml`) `h3-pack-adventure`'s. Pack text is
//! untrusted input: parsing is strict (unknown fields are errors, numbers must fit their types)
//! and every definition is validated again by the `Ruleset` compiler.
//!
//! Formats: `packs/README.md`; examples: `packs/synthetic/`.

use std::collections::BTreeMap;
use std::fmt;

use h3_content::{
    Affects, ArtifactClass, ArtifactDef, ArtifactSlotDef, ArtifactSlotOverride, CompileError,
    ContentKey, ContentPack, CreatureDef, CreatureFlag, CreatureInfoDef, CreatureOverride,
    CreaturePatch, CreatureSelector, CreatureStat, FactionDef, HeroClassDef, HeroEffect,
    HeroEffectDef, HeroOverride, HeroParamsDef, HeroPatch, HeroTypeDef, KeyError, PackManifest,
    ProfileDef, RAISED_LEVELS, RaisedCreature, ResourceDef, Ruleset, SecondarySkillDef,
    SpellCategory, SpellDef, SpellGrant, SpellSchoolDef, SpellScope, StartingStackDef, TerrainDef,
};
use serde::{Deserialize, Serialize};

/// The synthetic pack's core file (tests, benchmarks, FFI smoke, replays).
pub const SYNTHETIC_CORE: &str = include_str!("../../../packs/synthetic/core.toml");
/// The synthetic expansion pack's core file (depends on the synthetic pack; dependencies and
/// overrides).
pub const SYNTHETIC_EXT_CORE: &str = include_str!("../../../packs/synthetic-ext/core.toml");
/// Harvested pack `base`'s core file: The Restoration of Erathia, latest patch (E02,
/// OPEN-QUESTIONS Q13/Q15).
pub const BASE_CORE: &str = include_str!("../../../packs/base/core.toml");
/// Harvested pack `ab`'s core file: Armageddon's Blade (depends on `base`).
pub const AB_CORE: &str = include_str!("../../../packs/ab/core.toml");
/// Harvested pack `sod`'s core file: The Shadow of Death (depends on `ab`).
pub const SOD_CORE: &str = include_str!("../../../packs/sod/core.toml");

/// Errors of this adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackError {
    /// The text is not valid TOML for the expected schema.
    Syntax(String),
    /// A key or id is malformed.
    Key(KeyError),
    /// The packs do not compile into a ruleset.
    Compile(CompileError),
}

impl fmt::Display for PackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax(e) => write!(f, "syntax: {e}"),
            Self::Key(e) => write!(f, "{e}"),
            Self::Compile(e) => write!(f, "{e}"),
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

/// `core.toml`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PackFile {
    pack: ManifestDto,
    hero_params: Option<HeroParamsDto>,
    #[serde(default)]
    resource: Vec<NamedDto>,
    #[serde(default)]
    faction: Vec<FactionDto>,
    #[serde(default)]
    terrain: Vec<NamedDto>,
    #[serde(default)]
    creature: Vec<CreatureDto>,
    #[serde(default)]
    creature_override: Vec<OverrideDto>,
    #[serde(default)]
    spell_school: Vec<SchoolDto>,
    #[serde(default)]
    spell: Vec<SpellDto>,
    #[serde(default)]
    secondary_skill: Vec<SkillDto>,
    #[serde(default)]
    hero_class: Vec<HeroClassDto>,
    #[serde(default)]
    hero: Vec<HeroDto>,
    #[serde(default)]
    hero_override: Vec<HeroOverrideDto>,
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
struct SchoolDto {
    id: String,
    name: String,
    /// Pack format v6 (E04b T3b1).
    #[serde(default)]
    protection_rank: u8,
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
struct ManifestDto {
    id: String,
    version: String,
    #[serde(default)]
    depends: Vec<String>,
}

/// `[[creature]]` in `core.toml`: a creature's identity and its non-battle data (its battle data
/// is in the pack's `battle.toml`).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreatureDto {
    id: String,
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
    adventure_count: [u16; 2],
    #[serde(default)]
    flags: Vec<String>,
    #[serde(default)]
    upgrades: Vec<String>,
}

fn default_level() -> u8 {
    1
}

/// `[[creature_override]]` in `core.toml`: the core half of an override (every field but
/// `target` is optional; the battle half is in `battle.toml`). `faction = ""` makes the creature
/// neutral. Since pack format v34 (E04b T8e) `fixes` names the data quirk flag the override
/// fixes: it is applied only in profiles that turn the flag off (ADR-0007).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OverrideDto {
    target: String,
    fixes: Option<String>,
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
    adventure_count: Option<[u16; 2]>,
    flags: Option<Vec<String>>,
    upgrades: Option<Vec<String>>,
}

/// `[[spell]]` in `core.toml` (pack format v3, E04a T3): a spell's data row (its battle magic is
/// in the pack's `battle.toml`).
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
    /// `granted_by_name_only = true`: only a `spells` effect naming the spell gives it (pack
    /// format v30, E04b T8a; Titan's Lightning Bolt).
    #[serde(default)]
    granted_by_name_only: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum SpellCategoryDto {
    Adventure,
    Combat,
    Ability,
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

/// `[[hero_override]]` (pack format v34, E04b T8e): a hero type of the closure gets a new
/// specialty (the whole list), gated by `fixes` as a creature override.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HeroOverrideDto {
    target: String,
    fixes: Option<String>,
    specialty: Option<Vec<EffectDto>>,
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
    /// Pack format v40 (E08 T19a2b).
    learn_level: u8,
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
        level: u8,
        shots: u8,
        double_damage_chance_bp: u16,
    },
    FirstAid {
        heal_bp: u16,
    },
    CreatureStat {
        creatures: Option<SelectorDto>,
        stat: StatEntry,
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
        bp_per_level: u16,
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
        stat: StatEntry,
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
    SchoolMastery {
        school: String,
        level: u8,
    },
    /// `school` or `spell` (at most one; neither: every spell).
    SpellDamage {
        bp: u16,
        school: Option<String>,
        spell: Option<String>,
    },
    SpellBook {},
    MagicResistance {
        bp: u16,
    },
    SpellDuration {
        rounds: u8,
    },
    /// Exactly one of `school`, `level` and `spell` (pack format v30, E04b T8a).
    Spells {
        school: Option<String>,
        level: Option<u8>,
        spell: Option<String>,
    },
    SpellMastery {
        spell: String,
        level: u8,
    },
    Scroll {},
    /// Pack format v31 (E04b T8b).
    NoCasting {},
    SpellLevelLimit {
        level: u8,
    },
    SpellImmunity {
        spell: String,
    },
    SpellLevelImmunity {
        level: u8,
    },
    MindImmunity {},
    NegateImmunities {},
    /// Pack format v33 (E04b T8c, T8d).
    OpeningSpell {
        spell: String,
        mastery: u8,
        power: u16,
    },
    EagleEye {
        bp: u16,
        #[serde(default)]
        max_level: u8,
    },
    /// E08 increment 3 (T18a): Necromancy, Learning, Intelligence, Diplomacy's join bonus.
    Necromancy {
        bp: u16,
        /// Pack format v40 (E08 T19a2b): the share counts only while the hero lacks the skill.
        unless_skill: Option<String>,
    },
    ExperienceGain {
        bp: u16,
    },
    MaxMana {
        bp: u16,
    },
    JoinBonus {
        level: u8,
    },
    /// Pack format v38 (E08 T19a2, decision 30): `creature`, or `skill` with `by_level`.
    Raises {
        #[serde(default)]
        priority: u8,
        creature: Option<String>,
        skill: Option<String>,
        by_level: Option<Vec<String>>,
    },
    Ballistics {
        level: u8,
    },
    /// Pack format v40 (E08 T19a2b): Wisdom.
    LearnLevel {
        levels: u8,
    },
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

/// A creature statistic as packs, scenarios and replays write it ([`CreatureStat`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatEntry {
    /// Attack skill.
    Attack,
    /// Defense skill.
    Defense,
    /// Speed.
    Speed,
    /// Hit points per creature.
    HitPoints,
    /// Minimum damage per creature.
    MinDamage,
    /// Maximum damage per creature.
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
                level,
                shots,
                double_damage_chance_bp,
            } => HeroEffect::Artillery {
                level,
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
                stat: stat.into(),
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
            Self::SpellSpecialty {
                spell,
                bp_per_level,
            } => HeroEffect::SpellSpecialty {
                spell: reference(pack, &spell)?,
                bp_per_level,
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
                stat: stat.into(),
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
            Self::SchoolMastery { school, level } => HeroEffect::SchoolMastery {
                school: reference(pack, &school)?,
                level,
            },
            Self::SpellDamage { bp, school, spell } => HeroEffect::SpellDamage {
                scope: match (school, spell) {
                    (None, None) => SpellScope::All,
                    (Some(l), None) => SpellScope::School(reference(pack, &l)?),
                    (None, Some(s)) => SpellScope::Spell(reference(pack, &s)?),
                    (Some(_), Some(_)) => {
                        return Err(PackError::Syntax(
                            "spell_damage takes `school` or `spell`, not both".into(),
                        ));
                    }
                },
                bp,
            },
            Self::SpellBook {} => HeroEffect::SpellBook,
            Self::MagicResistance { bp } => HeroEffect::MagicResistance { bp },
            Self::SpellDuration { rounds } => HeroEffect::SpellDuration { rounds },
            Self::Spells {
                school,
                level,
                spell,
            } => HeroEffect::Spells {
                grant: match (school, level, spell) {
                    (Some(l), None, None) => SpellGrant::School(reference(pack, &l)?),
                    (None, Some(level), None) => SpellGrant::Level(level),
                    (None, None, Some(s)) => SpellGrant::Spell(reference(pack, &s)?),
                    _ => {
                        return Err(PackError::Syntax(
                            "spells takes exactly one of `school`, `level` and `spell`".into(),
                        ));
                    }
                },
            },
            Self::SpellMastery { spell, level } => HeroEffect::SpellMastery {
                spell: reference(pack, &spell)?,
                level,
            },
            Self::Scroll {} => HeroEffect::Scroll,
            Self::NoCasting {} => HeroEffect::NoCasting,
            Self::SpellLevelLimit { level } => HeroEffect::SpellLevelLimit { level },
            Self::SpellImmunity { spell } => HeroEffect::SpellImmunity {
                spell: reference(pack, &spell)?,
            },
            Self::SpellLevelImmunity { level } => HeroEffect::SpellLevelImmunity { level },
            Self::MindImmunity {} => HeroEffect::MindImmunity,
            Self::NegateImmunities {} => HeroEffect::NegateImmunities,
            Self::OpeningSpell {
                spell,
                mastery,
                power,
            } => HeroEffect::OpeningSpell {
                spell: reference(pack, &spell)?,
                mastery,
                power,
            },
            Self::EagleEye { bp, max_level } => HeroEffect::EagleEye { bp, max_level },
            Self::Necromancy { bp, unless_skill } => HeroEffect::Necromancy {
                bp,
                unless_skill: unless_skill.map(|k| reference(pack, &k)).transpose()?,
            },
            Self::ExperienceGain { bp } => HeroEffect::ExperienceGain { bp },
            Self::MaxMana { bp } => HeroEffect::MaxMana { bp },
            Self::JoinBonus { level } => HeroEffect::JoinBonus { level },
            Self::Raises {
                priority,
                creature,
                skill,
                by_level,
            } => HeroEffect::Raises {
                priority,
                creature: raised(pack, creature, skill, by_level)?,
            },
            Self::Ballistics { level } => HeroEffect::Ballistics { level },
            Self::LearnLevel { levels } => HeroEffect::LearnLevel { levels },
        })
    }
}

impl From<StatEntry> for CreatureStat {
    fn from(s: StatEntry) -> Self {
        match s {
            StatEntry::Attack => Self::Attack,
            StatEntry::Defense => Self::Defense,
            StatEntry::Speed => Self::Speed,
            StatEntry::HitPoints => Self::HitPoints,
            StatEntry::MinDamage => Self::MinDamage,
            StatEntry::MaxDamage => Self::MaxDamage,
        }
    }
}

impl From<CreatureStat> for StatEntry {
    fn from(s: CreatureStat) -> Self {
        match s {
            CreatureStat::Attack => Self::Attack,
            CreatureStat::Defense => Self::Defense,
            CreatureStat::Speed => Self::Speed,
            CreatureStat::HitPoints => Self::HitPoints,
            CreatureStat::MinDamage => Self::MinDamage,
            CreatureStat::MaxDamage => Self::MaxDamage,
        }
    }
}

/// The creature of a `raises` effect: `creature`, or `skill` with a creature per level in
/// `by_level` (none, Basic, Advanced, Expert).
fn raised(
    pack: &str,
    creature: Option<String>,
    skill: Option<String>,
    by_level: Option<Vec<String>>,
) -> Result<RaisedCreature<ContentKey, ContentKey>, PackError> {
    let syntax = || {
        PackError::Syntax(
            "raises takes `creature`, or `skill` with `by_level` (4 creatures: none, Basic, \
             Advanced, Expert)"
                .into(),
        )
    };
    Ok(match (creature, skill, by_level) {
        (Some(c), None, None) => RaisedCreature::Creature(reference(pack, &c)?),
        (None, Some(k), Some(list)) => RaisedCreature::ByLevel {
            skill: reference(pack, &k)?,
            by_level: <[ContentKey; RAISED_LEVELS]>::try_from(references(pack, &list)?)
                .map_err(|_| syntax())?,
        },
        _ => return Err(syntax()),
    })
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

/// Resolves a list of references ([`reference`]).
pub fn references(pack: &str, list: &[String]) -> Result<Vec<ContentKey>, KeyError> {
    list.iter().map(|t| reference(pack, t)).collect()
}

/// Resolves a reference: `name` means `<own pack>:name`, `pack:name` is taken as written.
pub fn reference(pack: &str, text: &str) -> Result<ContentKey, KeyError> {
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

/// Parses a pack's `core.toml`.
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
    let mut creatures = Vec::with_capacity(file.creature.len());
    for c in file.creature {
        creatures.push(CreatureDef {
            key: ContentKey::new(&pack_id, &c.id)?,
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
                adventure_count: c.adventure_count,
                flags: c.flags,
                upgrades: references(&pack_id, &c.upgrades)?,
            },
        });
    }
    let creature_overrides = file
        .creature_override
        .into_iter()
        .map(|o| {
            Ok(CreatureOverride {
                target: ContentKey::parse(&o.target)?,
                fixes: o.fixes,
                patch: CreaturePatch {
                    speed: o.speed,
                    name: o.name,
                    plural: o.plural,
                    faction: o
                        .faction
                        .map(|f| (!f.is_empty()).then(|| reference(&pack_id, &f)).transpose())
                        .transpose()?,
                    level: o.level,
                    cost: o.cost.map(|c| cost(&pack_id, c)).transpose()?,
                    growth: o.growth,
                    horde_growth: o.horde_growth,
                    fight_value: o.fight_value,
                    ai_value: o.ai_value,
                    adventure_count: o.adventure_count,
                    flags: o.flags,
                    upgrades: o.upgrades.map(|u| references(&pack_id, &u)).transpose()?,
                },
            })
        })
        .collect::<Result<Vec<_>, KeyError>>()?;
    let spell_schools = file
        .spell_school
        .into_iter()
        .map(|d| {
            Ok(SpellSchoolDef {
                key: ContentKey::new(&pack_id, &d.id)?,
                name: d.name,
                protection_rank: d.protection_rank,
            })
        })
        .collect::<Result<Vec<_>, PackError>>()?;
    let mut spells = Vec::with_capacity(file.spell.len());
    for s in file.spell {
        let key = ContentKey::new(&pack_id, &s.id)?;
        let schools = references(&pack_id, &s.schools)?;
        spells.push(SpellDef {
            key,
            name: s.name,
            category: match s.category {
                SpellCategoryDto::Adventure => SpellCategory::Adventure,
                SpellCategoryDto::Combat => SpellCategory::Combat,
                SpellCategoryDto::Ability => SpellCategory::Ability,
            },
            level: s.level,
            schools,
            cost: s.cost,
            power: s.power,
            effect: s.effect,
            granted_by_name_only: s.granted_by_name_only,
        });
    }
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
    let hero_overrides = file
        .hero_override
        .into_iter()
        .map(|o| {
            Ok(HeroOverride {
                target: ContentKey::parse(&o.target)?,
                fixes: o.fixes,
                patch: HeroPatch {
                    specialty: o.specialty.map(|e| effects(&pack_id, e)).transpose()?,
                },
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
    let manifest = PackManifest {
        id: pack_id,
        version: file.pack.version,
        depends: file.pack.depends,
    };
    Ok(ContentPack {
        resources,
        factions,
        terrains,
        creatures,
        creature_overrides,
        spell_schools,
        spells,
        skills,
        hero_classes,
        heroes,
        hero_overrides,
        artifact_slots,
        artifact_slot_overrides,
        artifacts,
        profiles,
        hero_params: file.hero_params.map(|p| HeroParamsDef {
            primary_min: p.primary_min,
            primary_max: p.primary_max,
            backpack: p.backpack,
            learn_level: p.learn_level,
        }),
        ..ContentPack::new(manifest)
    })
}

/// Parses `core.toml` texts (any order) and compiles them with the profile `profile` into the
/// core ruleset (the battle part: `h3-pack-battle`; the adventure part: `h3-pack-adventure`).
pub fn compile(texts: &[&str], profile: &str) -> Result<Ruleset, PackError> {
    let packs = texts
        .iter()
        .map(|t| parse_pack(t))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Ruleset::compile(&packs, profile)?)
}

#[cfg(test)]
mod tests;
