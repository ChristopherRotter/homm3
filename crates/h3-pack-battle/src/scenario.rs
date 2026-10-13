//! Battle scenarios (`packs/*/scenarios/*.toml`) and the entry types scenario and replay files
//! share: heroes, their battle effects, war machine controls, obstacles.

use std::collections::BTreeMap;
use std::num::NonZeroU32;

use h3_content::{
    CompileError, ContentKey, CreatureFlag, CreatureSelector, CreatureTypeId, Formation,
    MAX_ALIGNMENT_FACTIONS, Ruleset, SchoolId, SpellId, SpellScope,
};
use h3_contracts::{
    Army, ArmyStack, BallistaControl, BattleEffect, BattleHero, BattleSetup, BattleSide,
    DamageBonuses, EagleEye, FactionGroup, Obstacle, Rate, StatOp, TentControl,
};
use h3_pack::{AffectsEntry, StatEntry};
use serde::{Deserialize, Serialize};

use crate::BattlePackError;

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
    /// The natural obstacles, validated by the battle context (pieces since E04b T5b).
    #[serde(default)]
    obstacles: Vec<ObstacleEntry>,
    /// The battle hero leading each side, if any (BAT-HER-001, Q31).
    attacker_hero: Option<HeroEntry>,
    defender_hero: Option<HeroEntry>,
    /// Terrain key of the battlefield (BAT-TER-001).
    terrain: Option<String>,
    /// The special battlefield's key (E04c, BAT-TER-002), resolved by the battle.
    battlefield: Option<String>,
    /// The battle is at an anti-magic garrison (E04c, BAT-MAG-061).
    #[serde(default)]
    anti_magic: bool,
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
    /// What each side's owner can pay for a surrender (BAT-FLE-002; default 0).
    #[serde(default)]
    attacker_funds: u32,
    #[serde(default)]
    defender_funds: u32,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct StackDto {
    slot: u8,
    creature: String,
    count: u32,
}

/// A battle hero as scenario and replay files write it (`[scenario.attacker_hero]`,
/// `[setup.defender_hero]`; OPEN-QUESTIONS Q31): the battle hero itself, no derivation; every
/// field of [`BattleHero`] (the Catapult's control joins with sieges, E04c).
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
    /// Bonus of the side's shots, basis points ([`DamageBonuses::ranged`]; default 0, written
    /// only when set).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub ranged_bp: u32,
    /// Its single-precision value, an `f32` bit pattern (`h3_contracts::Rate::single`; written
    /// only when set, E04b T3c).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ranged_single: Option<NonZeroU32>,
    /// Bonus of the side's melee strikes, basis points ([`DamageBonuses::melee`]; default 0,
    /// written only when set).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub melee_bp: u32,
    /// Its single-precision value (as `ranged_single`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub melee_single: Option<NonZeroU32>,
    /// Reduction of the strikes against the side, basis points ([`DamageBonuses::reduction`];
    /// default 0, written only when set).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub reduction_bp: u32,
    /// Its single-precision value (as `ranged_single`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reduction_single: Option<NonZeroU32>,
    /// Discount on the price of a surrender, basis points ([`BattleHero::surrender_discount_bp`];
    /// default 0, written only when set).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub surrender_discount_bp: u16,
    /// The spell book by spell key ([`h3_contracts::BattleMagic::spell_book`]; default none,
    /// written only when the hero has one).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spell_book: Option<Vec<String>>,
    /// The side commands its Ballista ([`MachineControl::ballista`]; default not, written only
    /// when set).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ballista: Option<BallistaEntry>,
    /// The side commands its First Aid Tent ([`MachineControl::tent`]; default not, written only
    /// when set).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tent: Option<TentEntry>,
    /// The hero's Eagle Eye ([`h3_contracts::BattleMagic::eagle_eye`]; default none, written only
    /// when set; replay format v21, E04b T8d).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eagle_eye: Option<EagleEyeEntry>,
    /// The hero's mastery per spell by spell key, 1 to 3 ([`h3_contracts::BattleMagic::masteries`];
    /// default none, written only when set; replaces the school masteries since replay format
    /// v20, E04b T8a).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub masteries: BTreeMap<String, u8>,
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
    /// Its single-precision value, an `f32` bit pattern (written only when set, E04b T3c).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub single: Option<NonZeroU32>,
}

/// A hero's Eagle Eye ([`EagleEye`]) in scenario and replay files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EagleEyeEntry {
    /// The chance to learn a spell at one cast, basis points.
    pub chance_bp: u32,
    /// Its single-precision value, an `f32` bit pattern (written only when set).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub single: Option<NonZeroU32>,
    /// The spells it may learn, by spell key.
    pub spells: Vec<String>,
}

/// Whether a value is 0 (serde's `skip_serializing_if`).
fn is_zero<T: Default + PartialEq>(value: &T) -> bool {
    *value == T::default()
}

/// Which creatures a battle effect applies to, as scenario and replay files write it
/// ([`CreatureSelector`]): `{ creature = "pack:name" }` for a creature and its direct upgrades,
/// `{ flag = "dragon" }` for the creatures with a flag; omitted, every creature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum SelectorEntry {
    /// The creature type with this key and its direct upgrades.
    Creature(String),
    /// The creatures with this flag.
    Flag(String),
}

/// How a stack statistic effect applies ([`StatOp`]) in scenario and replay files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatOpEntry {
    /// [`StatOp::Add`].
    Add,
    /// [`StatOp::PercentBp`].
    PercentBp,
    /// [`StatOp::PercentBpDown`].
    PercentBpDown,
}

impl From<StatOpEntry> for StatOp {
    fn from(op: StatOpEntry) -> Self {
        match op {
            StatOpEntry::Add => Self::Add,
            StatOpEntry::PercentBp => Self::PercentBp,
            StatOpEntry::PercentBpDown => Self::PercentBpDown,
        }
    }
}

impl From<StatOp> for StatOpEntry {
    fn from(op: StatOp) -> Self {
        match op {
            StatOp::Add => Self::Add,
            StatOp::PercentBp => Self::PercentBp,
            StatOp::PercentBpDown => Self::PercentBpDown,
        }
    }
}

/// One battle effect of a hero as scenario and replay files write it: `kind` and the kind's
/// parameters (`packs/README.md`, "Scenarios"): every kind the battle reads (morale and luck,
/// BAT-HER-003; no retreat, BAT-FLE-001; stack statistics, BAT-HER-005; stack traits,
/// BAT-HER-006; alignment groups, BAT-HER-007); later kinds join with their rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// [`BattleEffect::NoRetreat`] (a struct variant, so that unknown keys are rejected).
    NoRetreat {},
    /// [`BattleEffect::StackStat`].
    StackStat {
        /// Which creatures (default every creature; written only when set).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        creatures: Option<SelectorEntry>,
        /// Which statistic.
        stat: StatEntry,
        /// How the amount applies.
        op: StatOpEntry,
        /// Amount (an addition, or basis points).
        amount: i16,
    },
    /// [`BattleEffect::Regeneration`].
    Regeneration {
        /// Which creatures (default every creature; written only when set).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        creatures: Option<SelectorEntry>,
        /// Most hit points regained per turn.
        hit_points: u16,
    },
    /// [`BattleEffect::NoRangePenalty`].
    NoRangePenalty {
        /// Which creatures (default every creature; written only when set).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        creatures: Option<SelectorEntry>,
    },
    /// [`BattleEffect::ShootWhenAdjacent`].
    ShootWhenAdjacent {
        /// Which creatures (default every creature; written only when set).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        creatures: Option<SelectorEntry>,
    },
    /// [`BattleEffect::AlignmentGroup`].
    AlignmentGroup {
        /// The factions by key (at most [`MAX_ALIGNMENT_FACTIONS`]).
        factions: Vec<String>,
    },
    /// [`BattleEffect::SpellDamage`]: `school` or `spell` by key (neither: every spell).
    SpellDamage {
        /// Bonus in basis points.
        bp: u32,
        /// Its single-precision value, an `f32` bit pattern (written only when set).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        single: Option<NonZeroU32>,
        /// The school (written only when set).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        school: Option<String>,
        /// The spell (written only when set).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        spell: Option<String>,
    },
    /// [`BattleEffect::SpellSpecialty`].
    SpellSpecialty {
        /// The spell by key.
        spell: String,
        /// Growth per hero level, basis points.
        bp_per_level: u16,
        /// The hero's level.
        level: u8,
    },
    /// [`BattleEffect::MagicResistance`].
    MagicResistance {
        /// Resistance in basis points.
        bp: u32,
        /// Its single-precision value, an `f32` bit pattern (written only when set).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        single: Option<NonZeroU32>,
    },
    /// [`BattleEffect::SpellDuration`].
    SpellDuration {
        /// Extra rounds.
        rounds: u8,
    },
    /// [`BattleEffect::NoCasting`] (a struct variant, so that unknown keys are rejected).
    NoCasting {},
    /// [`BattleEffect::SpellLevelLimit`].
    SpellLevelLimit {
        /// The highest spell level allowed.
        max_level: u8,
    },
    /// [`BattleEffect::SpellImmunity`].
    SpellImmunity {
        /// The spell by key.
        spell: String,
    },
    /// [`BattleEffect::SpellLevelImmunity`].
    SpellLevelImmunity {
        /// The highest level.
        max_level: u8,
    },
    /// [`BattleEffect::MindImmunity`] (a struct variant, so that unknown keys are rejected).
    MindImmunity {},
    /// [`BattleEffect::NegateImmunities`] (a struct variant, so that unknown keys are rejected).
    NegateImmunities {},
    /// [`BattleEffect::OpeningSpell`].
    OpeningSpell {
        /// The spell by key.
        spell: String,
        /// Its mastery.
        mastery: u8,
        /// Its power.
        power: u16,
    },
}

impl SelectorEntry {
    /// The selector of `entry`, its creature resolved against `ruleset` (omitted: every
    /// creature).
    fn resolve(
        entry: Option<SelectorEntry>,
        ruleset: &Ruleset,
    ) -> Result<CreatureSelector<CreatureTypeId>, BattlePackError> {
        Ok(match entry {
            None => CreatureSelector::All,
            Some(Self::Creature(key)) => CreatureSelector::Creature(
                ruleset
                    .creature_by_key(&key)
                    .ok_or(BattlePackError::UnknownCreature(key))?,
            ),
            Some(Self::Flag(flag)) => CreatureSelector::Flag(
                CreatureFlag::from_name(&flag).ok_or(CompileError::UnknownFlag(flag))?,
            ),
        })
    }

    /// The entry of `selector` (inverse of [`SelectorEntry::resolve`]).
    fn of(
        selector: CreatureSelector<CreatureTypeId>,
        ruleset: &Ruleset,
    ) -> Result<Option<Self>, BattlePackError> {
        Ok(match selector {
            CreatureSelector::All => None,
            CreatureSelector::Creature(c) => Some(Self::Creature(
                ruleset
                    .creature_key(c)
                    .map(|k| k.as_str().to_owned())
                    .ok_or_else(|| BattlePackError::UnknownCreature(format!("#{}", c.raw())))?,
            )),
            CreatureSelector::Flag(flag) => Some(Self::Flag(flag.name().to_owned())),
        })
    }
}

impl EffectEntry {
    /// The battle effect, its creature references resolved against `ruleset`.
    ///
    /// # Errors
    /// [`BattlePackError::UnknownCreature`], [`BattlePackError::UnknownFaction`]; an unknown flag
    /// ([`CompileError::UnknownFlag`]); [`BattlePackError::Syntax`] for an alignment group of more than
    /// [`MAX_ALIGNMENT_FACTIONS`] factions.
    pub fn effect(&self, ruleset: &Ruleset) -> Result<BattleEffect, BattlePackError> {
        Ok(match self.clone() {
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
            Self::NoRetreat {} => BattleEffect::NoRetreat,
            Self::StackStat {
                creatures,
                stat,
                op,
                amount,
            } => BattleEffect::StackStat {
                creatures: SelectorEntry::resolve(creatures, ruleset)?,
                stat: stat.into(),
                op: op.into(),
                amount,
            },
            Self::Regeneration {
                creatures,
                hit_points,
            } => BattleEffect::Regeneration {
                creatures: SelectorEntry::resolve(creatures, ruleset)?,
                hit_points,
            },
            Self::NoRangePenalty { creatures } => BattleEffect::NoRangePenalty {
                creatures: SelectorEntry::resolve(creatures, ruleset)?,
            },
            Self::ShootWhenAdjacent { creatures } => BattleEffect::ShootWhenAdjacent {
                creatures: SelectorEntry::resolve(creatures, ruleset)?,
            },
            Self::AlignmentGroup { factions } => {
                let ids = factions
                    .into_iter()
                    .map(|k| {
                        ruleset
                            .faction_by_key(&k)
                            .ok_or(BattlePackError::UnknownFaction(k))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                BattleEffect::AlignmentGroup {
                    factions: FactionGroup::new(&ids).ok_or_else(|| {
                        BattlePackError::Syntax(format!(
                            "alignment_group: more than {MAX_ALIGNMENT_FACTIONS} factions"
                        ))
                    })?,
                }
            }
            Self::SpellDamage {
                bp,
                single,
                school,
                spell,
            } => BattleEffect::SpellDamage {
                scope: match (school, spell) {
                    (None, None) => SpellScope::All,
                    (Some(l), None) => SpellScope::School(school_id(ruleset, l)?),
                    (None, Some(s)) => SpellScope::Spell(spell_id(ruleset, s)?),
                    (Some(_), Some(_)) => {
                        return Err(BattlePackError::Syntax(
                            "spell_damage takes `school` or `spell`, not both".into(),
                        ));
                    }
                },
                rate: Rate { bp, single },
            },
            Self::SpellSpecialty {
                spell,
                bp_per_level,
                level,
            } => BattleEffect::SpellSpecialty {
                spell: spell_id(ruleset, spell)?,
                bp_per_level,
                level,
            },
            Self::MagicResistance { bp, single } => BattleEffect::MagicResistance {
                rate: Rate { bp, single },
            },
            Self::SpellDuration { rounds } => BattleEffect::SpellDuration { rounds },
            Self::NoCasting {} => BattleEffect::NoCasting,
            Self::SpellLevelLimit { max_level } => BattleEffect::SpellLevelLimit { max_level },
            Self::SpellImmunity { spell } => BattleEffect::SpellImmunity {
                spell: spell_id(ruleset, spell)?,
            },
            Self::SpellLevelImmunity { max_level } => {
                BattleEffect::SpellLevelImmunity { max_level }
            }
            Self::MindImmunity {} => BattleEffect::MindImmunity,
            Self::NegateImmunities {} => BattleEffect::NegateImmunities,
            Self::OpeningSpell {
                spell,
                mastery,
                power,
            } => BattleEffect::OpeningSpell {
                spell: spell_id(ruleset, spell)?,
                mastery,
                power,
            },
        })
    }

    /// The entry of `effect` (inverse of [`EffectEntry::effect`]).
    ///
    /// # Errors
    /// [`BattlePackError::UnknownCreature`] or [`BattlePackError::UnknownFaction`] for a creature type or
    /// faction outside `ruleset`.
    pub fn of(effect: &BattleEffect, ruleset: &Ruleset) -> Result<Self, BattlePackError> {
        let selector = |creatures| SelectorEntry::of(creatures, ruleset);
        Ok(match *effect {
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
            BattleEffect::NoRetreat => Self::NoRetreat {},
            BattleEffect::StackStat {
                creatures,
                stat,
                op,
                amount,
            } => Self::StackStat {
                creatures: selector(creatures)?,
                stat: stat.into(),
                op: op.into(),
                amount,
            },
            BattleEffect::Regeneration {
                creatures,
                hit_points,
            } => Self::Regeneration {
                creatures: selector(creatures)?,
                hit_points,
            },
            BattleEffect::NoRangePenalty { creatures } => Self::NoRangePenalty {
                creatures: selector(creatures)?,
            },
            BattleEffect::ShootWhenAdjacent { creatures } => Self::ShootWhenAdjacent {
                creatures: selector(creatures)?,
            },
            BattleEffect::SpellDamage { scope, rate } => {
                let (school, spell) = match scope {
                    SpellScope::All => (None, None),
                    SpellScope::School(l) => (Some(school_key(ruleset, l)?), None),
                    SpellScope::Spell(s) => (None, Some(spell_key(ruleset, s)?)),
                };
                Self::SpellDamage {
                    bp: rate.bp,
                    single: rate.single,
                    school,
                    spell,
                }
            }
            BattleEffect::SpellSpecialty {
                spell,
                bp_per_level,
                level,
            } => Self::SpellSpecialty {
                spell: spell_key(ruleset, spell)?,
                bp_per_level,
                level,
            },
            BattleEffect::MagicResistance { rate } => Self::MagicResistance {
                bp: rate.bp,
                single: rate.single,
            },
            BattleEffect::SpellDuration { rounds } => Self::SpellDuration { rounds },
            BattleEffect::NoCasting => Self::NoCasting {},
            BattleEffect::SpellLevelLimit { max_level } => Self::SpellLevelLimit { max_level },
            BattleEffect::SpellImmunity { spell } => Self::SpellImmunity {
                spell: spell_key(ruleset, spell)?,
            },
            BattleEffect::SpellLevelImmunity { max_level } => {
                Self::SpellLevelImmunity { max_level }
            }
            BattleEffect::MindImmunity => Self::MindImmunity {},
            BattleEffect::NegateImmunities => Self::NegateImmunities {},
            BattleEffect::OpeningSpell {
                spell,
                mastery,
                power,
            } => Self::OpeningSpell {
                spell: spell_key(ruleset, spell)?,
                mastery,
                power,
            },
            BattleEffect::AlignmentGroup { factions } => Self::AlignmentGroup {
                factions: factions
                    .factions()
                    .iter()
                    .map(|&f| {
                        ruleset
                            .factions()
                            .get(f.index())
                            .map(|i| i.key.as_str().to_owned())
                            .ok_or_else(|| BattlePackError::UnknownFaction(format!("#{}", f.raw())))
                    })
                    .collect::<Result<_, _>>()?,
            },
        })
    }
}

impl HeroEntry {
    /// The battle hero, its hero type resolved against `ruleset`. The battle checks the rest
    /// when it starts (BAT-HER-001).
    ///
    /// # Errors
    /// [`BattlePackError::UnknownHeroType`].
    pub fn resolve(&self, ruleset: &Ruleset) -> Result<BattleHero, BattlePackError> {
        let hero_type = ruleset
            .hero_type_by_key(&self.hero_type)
            .ok_or_else(|| BattlePackError::UnknownHeroType(self.hero_type.clone()))?;
        let mut hero = BattleHero::new(hero_type);
        hero.attack = self.attack;
        hero.defense = self.defense;
        hero.power = self.power;
        hero.knowledge = self.knowledge;
        hero.tactics = self.tactics;
        hero.magic.mana = self.mana;
        hero.magic.max_mana = self.max_mana;
        let rate = |bp, single| Rate { bp, single };
        hero.damage = DamageBonuses {
            ranged: rate(self.ranged_bp, self.ranged_single),
            melee: rate(self.melee_bp, self.melee_single),
            reduction: rate(self.reduction_bp, self.reduction_single),
        };
        hero.war_machines.ballista = self.ballista.map(BallistaControl::from);
        hero.war_machines.tent = self.tent.map(|t| TentControl {
            heal_bonus: rate(t.heal_bonus_bp, t.single),
        });
        hero.surrender_discount_bp = self.surrender_discount_bp;
        hero.effects = self
            .effects
            .iter()
            .map(|e| e.effect(ruleset))
            .collect::<Result<_, _>>()?;
        hero.magic.spell_book = self
            .spell_book
            .as_ref()
            .map(|book| {
                book.iter()
                    .map(|k| spell_id(ruleset, k.clone()))
                    .collect::<Result<_, _>>()
            })
            .transpose()?;
        hero.magic.masteries = self
            .masteries
            .iter()
            .map(|(k, &level)| Ok((spell_id(ruleset, k.clone())?, level)))
            .collect::<Result<_, BattlePackError>>()?;
        // Spell order, as the Heroes context derives them (keys sort differently).
        hero.magic.masteries.sort_unstable_by_key(|&(s, _)| s);
        hero.magic.eagle_eye = self
            .eagle_eye
            .as_ref()
            .map(|e| {
                Ok::<_, BattlePackError>(EagleEye {
                    chance: rate(e.chance_bp, e.single),
                    spells: e
                        .spells
                        .iter()
                        .map(|k| spell_id(ruleset, k.clone()))
                        .collect::<Result<_, _>>()?,
                })
            })
            .transpose()?;
        Ok(hero)
    }

    /// The entry of `hero` (inverse of [`HeroEntry::resolve`]).
    ///
    /// # Errors
    /// [`BattlePackError::UnknownHeroType`], [`BattlePackError::UnknownSpell`], [`BattlePackError::UnknownSchool`]
    /// and the errors of [`EffectEntry::of`] for references outside `ruleset`.
    pub fn of(hero: &BattleHero, ruleset: &Ruleset) -> Result<Self, BattlePackError> {
        let spell_book = hero
            .magic
            .spell_book
            .as_ref()
            .map(|book| {
                book.iter()
                    .map(|&s| spell_key(ruleset, s))
                    .collect::<Result<_, _>>()
            })
            .transpose()?;
        let masteries = hero
            .magic
            .masteries
            .iter()
            .map(|&(s, level)| Ok((spell_key(ruleset, s)?, level)))
            .collect::<Result<_, BattlePackError>>()?;
        let effects = hero
            .effects
            .iter()
            .map(|e| EffectEntry::of(e, ruleset))
            .collect::<Result<_, _>>()?;
        let eagle_eye = hero
            .magic
            .eagle_eye
            .as_ref()
            .map(|e| {
                Ok::<_, BattlePackError>(EagleEyeEntry {
                    chance_bp: e.chance.bp,
                    single: e.chance.single,
                    spells: e
                        .spells
                        .iter()
                        .map(|&s| spell_key(ruleset, s))
                        .collect::<Result<_, _>>()?,
                })
            })
            .transpose()?;
        let key = ruleset
            .hero_type(hero.hero_type)
            .map(|t| t.key.as_str().to_owned())
            .ok_or_else(|| {
                BattlePackError::UnknownHeroType(format!("#{}", hero.hero_type.raw()))
            })?;
        Ok(Self {
            hero_type: key,
            attack: hero.attack,
            defense: hero.defense,
            power: hero.power,
            knowledge: hero.knowledge,
            tactics: hero.tactics,
            mana: hero.magic.mana,
            max_mana: hero.magic.max_mana,
            ranged_bp: hero.damage.ranged.bp,
            ranged_single: hero.damage.ranged.single,
            melee_bp: hero.damage.melee.bp,
            melee_single: hero.damage.melee.single,
            reduction_bp: hero.damage.reduction.bp,
            reduction_single: hero.damage.reduction.single,
            surrender_discount_bp: hero.surrender_discount_bp,
            ballista: hero.war_machines.ballista.map(BallistaEntry::from),
            tent: hero.war_machines.tent.map(|t| TentEntry {
                heal_bonus_bp: t.heal_bonus.bp,
                single: t.heal_bonus.single,
            }),
            spell_book,
            eagle_eye,
            masteries,
            effects,
        })
    }
}

/// The spell with key `key` in `ruleset`.
fn spell_id(ruleset: &Ruleset, key: String) -> Result<SpellId, BattlePackError> {
    ruleset
        .spell_by_key(&key)
        .ok_or(BattlePackError::UnknownSpell(key))
}

/// The key of spell `id` in `ruleset`.
fn spell_key(ruleset: &Ruleset, id: SpellId) -> Result<String, BattlePackError> {
    ruleset
        .spell(id)
        .map(|s| s.key.as_str().to_owned())
        .ok_or_else(|| BattlePackError::UnknownSpell(format!("#{}", id.raw())))
}

/// The spell school with key `key` in `ruleset`.
fn school_id(ruleset: &Ruleset, key: String) -> Result<SchoolId, BattlePackError> {
    (0u16..)
        .zip(ruleset.spell_schools())
        .find(|(_, s)| s.key.as_str() == key)
        .map(|(i, _)| SchoolId::from_raw(i))
        .ok_or(BattlePackError::UnknownSchool(key))
}

/// The key of spell school `id` in `ruleset`.
fn school_key(ruleset: &Ruleset, id: SchoolId) -> Result<String, BattlePackError> {
    ruleset
        .spell_schools()
        .get(id.index())
        .map(|s| s.key.as_str().to_owned())
        .ok_or_else(|| BattlePackError::UnknownSchool(format!("#{}", id.raw())))
}

/// One natural obstacle in a scenario or replay file (E04b T5b, OPEN-QUESTIONS Q55):
/// `{ hexes = [58, 75], anchor = 58, kind = "base:obstacle_3" }`, the hexes it blocks, for a
/// removable (usual) one its anchor hex, and its obstacle type's key if known (replay format 23,
/// E17 M1, CLT-Q3) ([`Obstacle`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObstacleEntry {
    /// The hexes it blocks.
    pub hexes: Vec<u8>,
    /// Its anchor hex, if it is removable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<u8>,
    /// Its obstacle type's key, if known (the battle checks it against the ruleset).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

impl ObstacleEntry {
    /// The entry of `o`.
    pub fn of(o: &Obstacle) -> Self {
        Self {
            hexes: o.hexes.clone(),
            anchor: o.anchor,
            kind: o.kind.as_ref().map(|k| k.as_str().to_owned()),
        }
    }

    /// The obstacle of the entry, or an error if its type is not a `pack:name` key.
    pub fn to_obstacle(&self) -> Result<Obstacle, BattlePackError> {
        let kind = self
            .kind
            .as_deref()
            .map(|k| {
                ContentKey::parse(k)
                    .map_err(|_| BattlePackError::Syntax(format!("bad obstacle type `{k}`")))
            })
            .transpose()?;
        Ok(Obstacle {
            hexes: self.hexes.clone(),
            anchor: self.anchor,
            kind,
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
    pub(crate) attacker: Vec<(u8, String, u32)>,
    pub(crate) defender: Vec<(u8, String, u32)>,
    pub(crate) obstacles: Vec<Obstacle>,
    pub(crate) heroes: [Option<HeroEntry>; 2],
    pub(crate) terrain: Option<String>,
    pub(crate) battlefield: Option<ContentKey>,
    pub(crate) anti_magic: bool,
    pub(crate) war_machines: [Vec<String>; 2],
    pub(crate) formations: [Formation; 2],
    pub(crate) funds: [u32; 2],
    /// Whether obstacles are generated for the terrain and the battlefield when the battle starts
    /// (the caller does it, with its seed; BAT-GEO-004). Excludes explicit obstacles.
    pub random_obstacles: bool,
}

/// Parses a list of side names (`"attacker"`, `"defender"`, each at most once) into flags indexed
/// by [`BattleSide::index`].
///
/// # Errors
/// An unknown or repeated side name.
pub fn parse_sides(names: &[String]) -> Result<[bool; 2], BattlePackError> {
    let mut sides = [false; 2];
    for name in names {
        let side = match name.as_str() {
            "attacker" => BattleSide::Attacker,
            "defender" => BattleSide::Defender,
            _ => return Err(BattlePackError::Syntax(format!("unknown side `{name}`"))),
        };
        if core::mem::replace(&mut sides[side.index()], true) {
            return Err(BattlePackError::Syntax(format!(
                "side `{name}` listed twice"
            )));
        }
    }
    Ok(sides)
}

/// The formations of a list of side names in tight formation (the other sides are loose).
///
/// # Errors
/// An unknown or repeated side name.
pub fn formations(tight: &[String]) -> Result<[Formation; 2], BattlePackError> {
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
    pub fn resolve(&self, ruleset: &Ruleset) -> Result<BattleSetup, BattlePackError> {
        let army = |stacks: &[(u8, String, u32)]| -> Result<Army, BattlePackError> {
            let stacks = stacks
                .iter()
                .map(|(slot, key, count)| {
                    let creature = ruleset
                        .creature_by_key(key)
                        .ok_or_else(|| BattlePackError::UnknownCreature(key.clone()))?;
                    Ok(ArmyStack {
                        slot: *slot,
                        creature,
                        count: *count,
                    })
                })
                .collect::<Result<Vec<_>, BattlePackError>>()?;
            Ok(Army { stacks })
        };
        let terrain = self
            .terrain
            .as_ref()
            .map(|t| {
                ruleset
                    .terrain_by_key(t)
                    .ok_or_else(|| BattlePackError::UnknownTerrain(t.clone()))
            })
            .transpose()?;
        let machines = |keys: &[String]| -> Result<Vec<_>, BattlePackError> {
            keys.iter()
                .map(|k| {
                    ruleset
                        .creature_by_key(k)
                        .ok_or_else(|| BattlePackError::UnknownCreature(k.clone()))
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
            battlefield: self.battlefield.clone(),
            anti_magic: self.anti_magic,
            war_machines: [
                machines(&self.war_machines[0])?,
                machines(&self.war_machines[1])?,
            ],
            formations: self.formations,
            funds: self.funds,
        })
    }
}

/// Parses a scenario.
pub fn parse_scenario(text: &str) -> Result<ScenarioDef, BattlePackError> {
    let file: ScenarioFile =
        toml::from_str(text).map_err(|e| BattlePackError::Syntax(e.to_string()))?;
    ContentKey::parse(&file.scenario.profile)?;
    if file.scenario.random_obstacles && !file.scenario.obstacles.is_empty() {
        return Err(BattlePackError::Syntax(
            "random_obstacles excludes an obstacles list".to_owned(),
        ));
    }
    if file.scenario.random_obstacles && file.scenario.terrain.is_none() {
        return Err(BattlePackError::Syntax(
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
        obstacles: file
            .scenario
            .obstacles
            .iter()
            .map(ObstacleEntry::to_obstacle)
            .collect::<Result<_, _>>()?,
        heroes: [file.scenario.attacker_hero, file.scenario.defender_hero],
        terrain: file.scenario.terrain,
        battlefield: file
            .scenario
            .battlefield
            .as_deref()
            .map(ContentKey::parse)
            .transpose()?,
        anti_magic: file.scenario.anti_magic,
        war_machines: [
            file.scenario.attacker_war_machines,
            file.scenario.defender_war_machines,
        ],
        formations: formations(&file.scenario.tight_formation)?,
        funds: [file.scenario.attacker_funds, file.scenario.defender_funds],
        random_obstacles: file.scenario.random_obstacles,
    })
}
