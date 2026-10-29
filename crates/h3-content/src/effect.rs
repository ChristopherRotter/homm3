//! Hero effects: the closed set of typed primitives a secondary-skill level, a hero's specialty
//! or an artifact can carry (ADR-0013 §3, ADR-0006 Tier 3).
//!
//! An effect states *what* a source gives and how its value is computed; it does not apply
//! anything. The pure translation in `h3-heroes` (E04a T4) resolves a hero's effects into the
//! battle's own `BattleEffect`s and typed `BattleHero` fields, and the rules that read them carry
//! catalog ids (`HER-…`, `BAT-…`). Percentages are basis points (1/100 %), so every skill and
//! specialty product stays an exact integer.
//!
//! [`HeroEffect`] is generic over how creature types (`C`), secondary skills (`K`), spells (`S`),
//! factions (`F`) and spell schools (`L`) are referenced: [`ContentKey`]s in packs
//! ([`HeroEffectDef`]), handles once compiled ([`CompiledHeroEffect`]).
//!
//! Effects that only matter outside battle (adventure map, towns, outcome application) get
//! variants with the epic that reads them; the pack docs list which sources carry such effects.

use alloc::vec::Vec;

use h3_kernel::{CanonicalHash, StateHasher};

use crate::flags::CreatureFlag;
use crate::heroes::{MAX_SPELL_LEVEL, SchoolId, SkillId, SpellCategory, SpellId};
use crate::key::ContentKey;
use crate::ruleset::{CreatureTypeId, FactionId, Ruleset};

/// Most effects in one list (a skill level, a specialty, an artifact).
pub const MAX_EFFECTS: usize = 16;

/// Most factions in one `alignment_group` effect.
pub const MAX_ALIGNMENT_FACTIONS: usize = 16;

/// Highest secondary-skill level (Expert); levels are 1 (Basic) to 3 (Expert).
pub const MAX_SKILL_LEVEL: u8 = 3;

/// The largest basis-point value of a share that cannot exceed the whole (100 %).
pub const FULL_BP: u16 = 10_000;

/// Whose stacks a morale or luck effect changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Affects {
    /// The stacks of the hero's own side.
    Own,
    /// The stacks of the opposing side.
    Enemy,
    /// The stacks of both sides.
    Both,
}

impl Affects {
    /// All values, in code order.
    pub const ALL: [Self; 3] = [Self::Own, Self::Enemy, Self::Both];

    /// Pack name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Own => "own",
            Self::Enemy => "enemy",
            Self::Both => "both",
        }
    }

    /// Looks a value up by its pack name.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|a| a.name() == name)
    }
}

/// A creature statistic an effect changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CreatureStat {
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

impl CreatureStat {
    /// All values, in code order.
    pub const ALL: [Self; 6] = [
        Self::Attack,
        Self::Defense,
        Self::Speed,
        Self::HitPoints,
        Self::MinDamage,
        Self::MaxDamage,
    ];

    /// Pack name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Attack => "attack",
            Self::Defense => "defense",
            Self::Speed => "speed",
            Self::HitPoints => "hit_points",
            Self::MinDamage => "min_damage",
            Self::MaxDamage => "max_damage",
        }
    }

    /// Looks a value up by its pack name.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.name() == name)
    }
}

/// Which creatures of the hero's army an effect applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreatureSelector<C> {
    /// Every creature, war machines included.
    All,
    /// Creatures with this flag (dragons, living creatures, shooters, …).
    Flag(CreatureFlag),
    /// This creature type and its direct upgrades (the creature's `upgrades` list).
    Creature(C),
}

impl<C> CreatureSelector<C> {
    /// Maps the creature reference.
    pub fn try_map<D, E>(
        &self,
        f: impl FnOnce(&C) -> Result<D, E>,
    ) -> Result<CreatureSelector<D>, E> {
        Ok(match self {
            Self::All => CreatureSelector::All,
            Self::Flag(flag) => CreatureSelector::Flag(*flag),
            Self::Creature(c) => CreatureSelector::Creature(f(c)?),
        })
    }
}

impl CreatureSelector<CreatureTypeId> {
    /// Whether the selector takes creature type `id` of `ruleset`: every creature, a creature
    /// with the flag, or the named creature and its direct upgrades.
    pub fn matches(&self, ruleset: &Ruleset, id: CreatureTypeId) -> bool {
        match *self {
            Self::All => true,
            Self::Flag(flag) => ruleset.creature(id).is_some_and(|c| c.flags.contains(flag)),
            Self::Creature(c) => c == id || ruleset.upgrades(c).contains(&id),
        }
    }
}

/// Levels a [`RaisedCreature::ByLevel`] names a creature for: without the skill, then Basic to
/// Expert.
pub const RAISED_LEVELS: usize = MAX_SKILL_LEVEL as usize + 1;

/// The creature a `raises` effect names (E08 T19a2, decision 30).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaisedCreature<C, K> {
    /// Always this creature (the Necromancy skill: Skeletons).
    Creature(C),
    /// The creature by the hero's level in `skill`: `by_level[0]` without the skill, then Basic
    /// to Expert (the Cloak of the Undead King by the Necromancy level: Skeletons, Walking Dead,
    /// Wights, Liches).
    ByLevel {
        /// The secondary skill whose level picks the creature.
        skill: K,
        /// The creature at each level, none to Expert.
        by_level: [C; RAISED_LEVELS],
    },
}

impl<C, K> RaisedCreature<C, K> {
    /// Maps the references.
    pub fn try_map<D, M, E>(
        &self,
        mut creature: impl FnMut(&C) -> Result<D, E>,
        skill: impl FnOnce(&K) -> Result<M, E>,
    ) -> Result<RaisedCreature<D, M>, E> {
        Ok(match self {
            Self::Creature(c) => RaisedCreature::Creature(creature(c)?),
            Self::ByLevel { skill: k, by_level } => {
                let [a, b, c, d] = by_level;
                RaisedCreature::ByLevel {
                    skill: skill(k)?,
                    by_level: [creature(a)?, creature(b)?, creature(c)?, creature(d)?],
                }
            }
        })
    }
}

impl<C: CanonicalHash, K: CanonicalHash> CanonicalHash for RaisedCreature<C, K> {
    fn hash_into(&self, h: &mut StateHasher) {
        match self {
            Self::Creature(c) => {
                h.write_u8(0);
                c.hash_into(h);
            }
            Self::ByLevel { skill, by_level } => {
                h.write_u8(1);
                skill.hash_into(h);
                for c in by_level {
                    c.hash_into(h);
                }
            }
        }
    }
}

/// Which of the hero's spells a spell-damage bonus applies to (E04b T2a).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellScope<L, S> {
    /// Every spell (Sorcery).
    All,
    /// The spells of this school (the Orbs).
    School(L),
    /// This spell (Ciele's specialty).
    Spell(S),
}

impl<L, S> SpellScope<L, S> {
    /// Maps the references.
    pub fn try_map<L2, S2, E>(
        &self,
        school: impl FnOnce(&L) -> Result<L2, E>,
        spell: impl FnOnce(&S) -> Result<S2, E>,
    ) -> Result<SpellScope<L2, S2>, E> {
        Ok(match self {
            Self::All => SpellScope::All,
            Self::School(l) => SpellScope::School(school(l)?),
            Self::Spell(s) => SpellScope::Spell(spell(s)?),
        })
    }
}

impl<L: CanonicalHash, S: CanonicalHash> CanonicalHash for SpellScope<L, S> {
    /// Tag (0 all, 1 school, 2 spell), then the reference.
    fn hash_into(&self, h: &mut StateHasher) {
        match self {
            Self::All => h.write_u8(0),
            Self::School(l) => {
                h.write_u8(1);
                l.hash_into(h);
            }
            Self::Spell(s) => {
                h.write_u8(2);
                s.hash_into(h);
            }
        }
    }
}

/// Which spells a `spells` effect gives the hero (E04b T8a; HER-SPL-002): the original's artifact
/// spell sets (`installation.md` §7.22).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellGrant<L, S> {
    /// Every spell of this school that is not a creature ability and not
    /// [`granted_by_name_only`](crate::Spell::granted_by_name_only) (the Tomes).
    School(L),
    /// Every spell of this level that is not a creature ability and not granted by name only
    /// (Spellbinder's Hat).
    Level(u8),
    /// This spell (Titan's Thunder, Armageddon's Blade, the Sea Captain's Hat).
    Spell(S),
}

impl<L, S> SpellGrant<L, S> {
    /// Maps the references.
    pub fn try_map<L2, S2, E>(
        &self,
        school: impl FnOnce(&L) -> Result<L2, E>,
        spell: impl FnOnce(&S) -> Result<S2, E>,
    ) -> Result<SpellGrant<L2, S2>, E> {
        Ok(match self {
            Self::School(l) => SpellGrant::School(school(l)?),
            Self::Level(level) => SpellGrant::Level(*level),
            Self::Spell(s) => SpellGrant::Spell(spell(s)?),
        })
    }
}

impl SpellGrant<SchoolId, SpellId> {
    /// Whether the grant gives `spell` of `ruleset` (a spell the ruleset lacks: never): a named
    /// spell always; by school or level only a spell that is not a creature ability and not
    /// granted by name only.
    pub fn gives(self, ruleset: &Ruleset, spell: SpellId) -> bool {
        ruleset.spell(spell).is_some_and(|s| {
            let by_scope = s.category != SpellCategory::Ability && !s.granted_by_name_only;
            match self {
                Self::School(school) => by_scope && s.schools.contains(&school),
                Self::Level(level) => by_scope && s.level == level,
                Self::Spell(named) => named == spell,
            }
        })
    }
}

impl<L: CanonicalHash, S: CanonicalHash> CanonicalHash for SpellGrant<L, S> {
    /// Tag (0 school, 1 level, 2 spell), then the reference or the level.
    fn hash_into(&self, h: &mut StateHasher) {
        match self {
            Self::School(l) => {
                h.write_u8(0);
                l.hash_into(h);
            }
            Self::Level(level) => {
                h.write_u8(1);
                h.write_u8(*level);
            }
            Self::Spell(s) => {
                h.write_u8(2);
                s.hash_into(h);
            }
        }
    }
}

/// One hero effect. `C`, `K`, `S`, `F` and `L` are how creature types, secondary skills, spells,
/// factions and spell schools are referenced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeroEffect<C, K, S, F, L> {
    /// Changes the morale of the affected stacks for the whole battle (Leadership).
    Morale {
        /// Morale change (not 0).
        amount: i8,
        /// Whose stacks.
        affects: Affects,
    },
    /// Changes the luck of the affected stacks for the whole battle (Luck).
    Luck {
        /// Luck change (not 0).
        amount: i8,
        /// Whose stacks.
        affects: Affects,
    },
    /// Raises the base damage of the hero's shots, the Ballista's included (Archery).
    RangedDamage {
        /// Bonus in basis points (≥ 1).
        bp: u16,
    },
    /// Raises the base damage of the hero's melee attacks and retaliations (Offense).
    MeleeDamage {
        /// Bonus in basis points (≥ 1).
        bp: u16,
    },
    /// Reduces the physical damage the hero's stacks take (Armorer).
    DamageReduction {
        /// Reduction in basis points (1..=10 000).
        bp: u16,
    },
    /// Reduces the price of the hero's surrender (Diplomacy).
    SurrenderDiscount {
        /// Discount in basis points (1..=10 000).
        bp: u16,
    },
    /// Gives the hero this level of the Tactics skill (the battle compares both sides').
    Tactics {
        /// Tactics level (1..=3).
        level: u8,
    },
    /// The hero commands its Ballista (Artillery).
    Artillery {
        /// The Artillery level (1..=3; the computer's worth of the Ballista grows with it).
        level: u8,
        /// Shots per turn (≥ 1).
        shots: u8,
        /// Chance of each shot to deal double base damage, basis points (≤ 10 000).
        double_damage_chance_bp: u16,
    },
    /// The hero commands its First Aid Tent, whose largest heal grows by this share of the
    /// tent's own `first_aid` maximum (First Aid).
    FirstAid {
        /// Increase of the maximum heal, basis points of the tent's maximum (≥ 1).
        heal_bp: u16,
    },
    /// Changes a statistic of the selected creatures of the hero's army by a fixed amount
    /// (fixed creature specialties, Sir Mullich, the dragon specialists).
    CreatureStat {
        /// Which creatures.
        creatures: CreatureSelector<C>,
        /// Which statistic.
        stat: CreatureStat,
        /// Change (not 0).
        amount: i16,
    },
    /// Specialty: the scalable values of the hero's effects from `skill` grow by `bp_per_level`
    /// basis points of themselves per hero level (`docs/formats/installation.md` §7.4).
    SkillSpecialty {
        /// The secondary skill.
        skill: K,
        /// Growth per hero level, basis points (≥ 1).
        bp_per_level: u16,
    },
    /// Specialty: attack and defense of `creature` and its direct upgrades grow by `bp_per_step`
    /// basis points of the creature's own value (rounded up) for every full step of as many
    /// hero levels as the creature's level; speed grows by `speed`.
    CreatureSpecialty {
        /// The base creature.
        creature: C,
        /// Growth per step, basis points of the creature's own attack and defense (≥ 1).
        bp_per_step: u16,
        /// Speed bonus.
        speed: u8,
    },
    /// Specialty: the effect of the hero's casts of `spell` on a stack grows by `bp_per_level`
    /// basis points per hero level, divided by the level of the stack's creature (SoD: 3 % per
    /// level; damage since E04b T2, other spell kinds with their tasks).
    SpellSpecialty {
        /// The spell.
        spell: S,
        /// Growth per hero level before the division, basis points (≥ 1).
        bp_per_level: u16,
    },
    /// Changes the hero's primary skills (artifacts; the executable's artifact table). Unlike
    /// every other kind, it counts once per equipped copy (two Quiet Eyes of the Dragon give
    /// +2 attack); the derivation clamps the hero's totals (E04a T10).
    PrimarySkills {
        /// Attack change.
        attack: i8,
        /// Defense change.
        defense: i8,
        /// Spell power change.
        power: i8,
        /// Knowledge change.
        knowledge: i8,
    },
    /// Caps the morale of the affected stacks for the whole battle (Spirit of Oppression).
    MoraleAtMost {
        /// Highest morale.
        value: i8,
        /// Whose stacks.
        affects: Affects,
    },
    /// Caps the luck of the affected stacks for the whole battle (Hourglass of the Evil Hour).
    LuckAtMost {
        /// Highest luck.
        value: i8,
        /// Whose stacks.
        affects: Affects,
    },
    /// Creatures of these factions count as one alignment for the army-alignment morale of the
    /// hero's side (BAT-MOR-002; Angelic Alliance).
    AlignmentGroup {
        /// The factions (2..=16, distinct).
        factions: Vec<F>,
    },
    /// Changes a statistic of the selected creatures of the hero's army by a share of the
    /// creature's own value (Elixir of Life: +25 % hit points).
    CreatureStatPercent {
        /// Which creatures.
        creatures: CreatureSelector<C>,
        /// Which statistic.
        stat: CreatureStat,
        /// Change in basis points of the creature's own value (≥ 1).
        bp: u16,
    },
    /// The top creature of each selected stack regains up to `hit_points` lost hit points at the
    /// start of its turn (Elixir of Life).
    Regeneration {
        /// Which creatures.
        creatures: CreatureSelector<C>,
        /// Most hit points regained per turn (≥ 1).
        hit_points: u16,
    },
    /// The selected creatures' shots have no range penalty (Golden Bow).
    NoRangePenalty {
        /// Which creatures.
        creatures: CreatureSelector<C>,
    },
    /// The selected creatures may shoot while an enemy stands next to them (Bow of the
    /// Sharpshooter).
    ShootWhenAdjacent {
        /// Which creatures.
        creatures: CreatureSelector<C>,
    },
    /// Neither side may retreat or surrender (Shackles of War).
    NoRetreat,
    /// Mastery of a spell school (the magic school skills): the hero casts the school's spells
    /// at this level, the highest such level counting.
    SchoolMastery {
        /// The school.
        school: L,
        /// Mastery, 1 (Basic) to 3 (Expert).
        level: u8,
    },
    /// More damage from the hero's spells of a scope (Sorcery, the Orbs, Ciele's specialty).
    SpellDamage {
        /// Which spells.
        scope: SpellScope<L, S>,
        /// Bonus in basis points (≥ 1).
        bp: u16,
    },
    /// The hero can cast the spells it knows in battle (the Spell Book).
    SpellBook,
    /// The hero's stacks resist hostile spells: their creatures' share a spell affects shrinks
    /// by `bp` (the Resistance skill, Garniture of Interference, Surcoat of Counterpoise, Boots of
    /// Polarity; E04b T2d).
    MagicResistance {
        /// Resistance in basis points (1..=10 000).
        bp: u16,
    },
    /// The hero's timed spells last `rounds` rounds longer (Collar, Ring and Cape of Conjuring,
    /// Ring of the Magi; E04b T3a).
    SpellDuration {
        /// Extra rounds (≥ 1).
        rounds: u8,
    },
    /// The hero has these spells available while it wears the artifact (the Tomes, Spellbinder's
    /// Hat, Titan's Thunder, Armageddon's Blade, the Sea Captain's Hat; E04b T8a, HER-SPL-002).
    Spells {
        /// Which spells.
        grant: SpellGrant<L, S>,
    },
    /// The hero casts `spell` at least at this mastery (Armageddon's Blade: Armageddon at
    /// Expert; E04b T8a, HER-SPL-003).
    SpellMastery {
        /// The spell.
        spell: S,
        /// Mastery, 1 (Basic) to 3 (Expert).
        level: u8,
    },
    /// Each copy of the artifact carries a spell of its own, available to the hero while the copy
    /// is worn (the Spell Scroll; E04b T8a, HER-ART-003, HER-SPL-002).
    Scroll,
    /// Neither side casts in a battle of the hero: no hero's spell, no creature's cast (the Orb
    /// of Inhibition; E04b T8b1, HER-BAT-015).
    NoCasting,
    /// No spell above `level` is cast by a hero or affects a stack in a battle of the hero,
    /// whoever casts it (Recanter's Cloak: 2; E04b T8b1, HER-BAT-015).
    SpellLevelLimit {
        /// The highest spell level allowed (1..=4).
        level: u8,
    },
    /// The stacks the hero controls are never affected by `spell`, whoever casts it (a Pendant,
    /// the Sphere of Permanence, Armageddon's Blade; E04b T8b2, HER-BAT-016).
    SpellImmunity {
        /// The spell.
        spell: S,
    },
    /// The stacks the hero controls are never affected by a spell up to `level` but Dispel
    /// (Power of the Dragon Father: 4; E04b T8b2, HER-BAT-016).
    SpellLevelImmunity {
        /// The highest level (1..=5).
        level: u8,
    },
    /// The stacks the hero controls are immune to mind spells, unless the immunities are negated
    /// (Badge of Courage; E04b T8b2, HER-BAT-016).
    MindImmunity,
    /// The side's spells, and spells at the stacks the hero controls, ignore the creatures'
    /// immunities and resistance and the heroes' resistance (the Orb of Vulnerability; E04b T8b2,
    /// HER-BAT-016).
    NegateImmunities,
    /// At the first activation of a stack its side controls outside the tactics phase, the side
    /// casts `spell` at `mastery` with `power` (its duration) for free, if it affects some stack
    /// (Angelic Alliance's Prayer, Armor of the Damned's Slow, Curse, Weakness, Misfortune; E04b
    /// T8c, HER-BAT-017). In list order.
    OpeningSpell {
        /// The spell.
        spell: S,
        /// Mastery, 0 (none) to 3 (Expert).
        mastery: u8,
        /// Power, the effects' duration in rounds (≥ 1).
        power: u16,
    },
    /// Eagle Eye (E04b T8d, HER-SPL-004): the chance to learn a spell at each cast of the enemy
    /// hero grows by `bp`, and spells up to `max_level` may be learned (the skill: 40 %, 50 %,
    /// 60 % and levels 2, 3, 4; the Bird of Perception, the Stoic Watchman and the Emblem of
    /// Cognizance +5 %, +10 %, +15 % and no level, which need the skill).
    EagleEye {
        /// Chance in basis points (1..=10 000).
        bp: u16,
        /// The highest spell level it lets the hero learn (0 adds none, at most 5).
        max_level: u8,
    },
    /// Necromancy (E08 increment 3): after a won battle the hero raises `bp` of the killed enemy
    /// creatures as undead (the skill: 10 %, 20 %, 30 %; a specialty grows it, the necromancy
    /// artifacts add to it). Applied by the game with the battle's outcome (ADR-0017 point 11).
    Necromancy {
        /// Share in basis points (1..=10 000).
        bp: u16,
        /// The share counts only while the hero lacks this skill (pack format v40, E08 T19a2b:
        /// the Cloak of the Undead King's 30 % for a hero without Necromancy, whose parts give
        /// theirs with the skill).
        unless_skill: Option<K>,
    },
    /// The experience the hero gains grows by `bp` (Learning, E08 increment 3: 5 %, 10 %, 15 %).
    ExperienceGain {
        /// Bonus in basis points (≥ 1).
        bp: u16,
    },
    /// The hero's maximum mana (spell points, Knowledge × 10) grows by `bp` of itself
    /// (Intelligence, E08 increment 3: 25 %, 50 %, 100 %).
    MaxMana {
        /// Bonus in basis points (≥ 1).
        bp: u16,
    },
    /// Wandering creatures are readier to join the hero: `level` counts toward the monster's
    /// answer (Diplomacy, E08 increment 3: 1, 2, 3 by the skill's level; the adventure's
    /// ADV-MON-007 reads the sum through `h3-heroes`).
    JoinBonus {
        /// Bonus (1..=3).
        level: u8,
    },
    /// The creature the hero's Necromancy raises (E08 T19a2, decision 30): of all the hero's
    /// `raises` effects the highest `priority` decides (the Necromancy skill Skeletons at 0, the
    /// Cloak of the Undead King its creature by the Necromancy level at 1; a mod may go higher).
    /// Among equal priorities, if every one comes from an artifact, the lowest artifact id
    /// (that of the artifact or combination part whose list holds the effect), then the lowest
    /// creature id; otherwise the lowest creature id. The share raised is the `necromancy`
    /// kind's; applied by the game with the battle's outcome (ADR-0017 point 11).
    Raises {
        /// Priority: the highest wins.
        priority: u8,
        /// The creature raised.
        creature: RaisedCreature<C, K>,
    },
    /// Gives the hero this level of the Ballistics skill: in a siege the side commands its
    /// Catapult, whose shots the level's record of the battle's table decides (E04c T6, BAT-Q43).
    Ballistics {
        /// Ballistics level (1..=3).
        level: u8,
    },
    /// The highest spell level the hero may learn grows by `levels` over the hero parameters'
    /// `learn_level` (pack format v40, E08 T19a2b; Wisdom: 1, 2, 3, so levels 3 to 5; sources
    /// add, at most level 5).
    LearnLevel {
        /// Levels added (1..=5).
        levels: u8,
    },
    /// The hero's Pathfinding level (pack format v46, E08 increment 5; ADR-0013 point 13): the
    /// adventure's step price reads it (the terrains' costs by level, ADV-MOV-003). Several
    /// sources: the highest (the skill: 1, 2, 3).
    Pathfinding {
        /// Pathfinding level (1..=3).
        level: u8,
    },
    /// The day's land points grow by `bp` of the land table's entry (pack format v46, E08
    /// increment 5; HER-ADV-003). Several sources add, each skill's grown by its skill
    /// specialties (Logistics: 10 %, 20 %, 30 %; its specialists 5 % of the rate per level).
    LandMovement {
        /// Bonus in basis points (≥ 1).
        bp: u16,
    },
    /// Flat points added to the day's land points after the rates (pack format v46, E08
    /// increment 5; HER-ADV-003). Several sources add (Boots of Speed 600, Equestrian's Gloves
    /// 300).
    LandMovementPoints {
        /// Movement points (≥ 1).
        points: u16,
    },
    /// The hero flies on the adventure map (pack format v46, E08 increment 5; HER-ADV-005): the
    /// level indexes the adventure's fly prices (0 to 3, the original's index; Angel Wings 3, the
    /// spell's masteries later). Several sources: the highest.
    Flying {
        /// Flying level (0..=3).
        level: u8,
    },
    /// The hero walks on water (pack format v46, E08 increment 5; HER-ADV-005): the level
    /// indexes the adventure's water-walk prices (0 to 3; Boots of Levitation 3). Several
    /// sources: the highest.
    WaterWalking {
        /// Water-walking level (0..=3).
        level: u8,
    },
    /// The hero's scouting radius grows by `tiles` over the adventure's base radius (pack format
    /// v46, E08 increment 5; HER-ADV-006). Several sources add (Scouting 1, 2, 3; the Speculum
    /// and the Spyglass 1 each).
    ScoutingRadius {
        /// Tiles added (≥ 1).
        tiles: u8,
    },
    /// At a meeting of two heroes of one player each learns the other's learned spells up to this
    /// level and its own learnable level (pack format v47, E08 increment 6; ADR-0013 point 14.3,
    /// HER-SPL-007; Scholar: 2, 3, 4, the original's level + 1, 0x4A2940). Several sources, and
    /// the two heroes' sources: the highest.
    MeetingSpells {
        /// The highest spell level a meeting may teach (1..=5).
        level: u8,
    },
    /// Equipping the artifact puts a Spell Book into the hero's empty spell book position;
    /// removing it takes nothing back (pack format v47, E08 increment 6; ADR-0013 point 3,
    /// HER-ART-007; Titan's Thunder, 0x4E2A64).
    GivesSpellBook,
    /// The hero's Navigation level (pack format v49, E08 increment 8; ADR-0013 points 3 and
    /// 13.6): the index into the adventure's sea table (0 without one; the original's
    /// `sea_movement_by_navigation` 0x698AEC). Several sources: the highest (the skill: 1, 2, 3).
    Navigation {
        /// Navigation level (1..=3).
        level: u8,
    },
    /// Flat points added to the day's sea points after the table's entry (pack format v49, E08
    /// increment 8; HER-ADV-009). Several sources add, each artifact type once (the Necklace of
    /// Ocean Guidance 1 000, the Sea Captain's Hat 500).
    SeaMovementPoints {
        /// Movement points (≥ 1).
        points: u16,
    },
    /// A boarding or a landing converts the hero's points left instead of setting them to 0
    /// (pack format v49, E08 increment 8; ADR-0013 point 13.7, HER-ADV-009; the Admiral's Hat,
    /// tested as `hero_wears_artifact` 0x4D91F0 with 136).
    FreeBoarding,
}

/// An effect as written in a pack (references are keys).
pub type HeroEffectDef = HeroEffect<ContentKey, ContentKey, ContentKey, ContentKey, ContentKey>;
/// An effect with resolved references.
pub type CompiledHeroEffect = HeroEffect<CreatureTypeId, SkillId, SpellId, FactionId, SchoolId>;

impl<C, K, S, F, L> HeroEffect<C, K, S, F, L> {
    /// Pack names, indexed by [`HeroEffect::code`].
    pub const NAMES: [&'static str; 56] = [
        "morale",
        "luck",
        "ranged_damage",
        "melee_damage",
        "damage_reduction",
        "surrender_discount",
        "tactics",
        "artillery",
        "first_aid",
        "creature_stat",
        "skill_specialty",
        "creature_specialty",
        "spell_specialty",
        "primary_skills",
        "morale_at_most",
        "luck_at_most",
        "alignment_group",
        "creature_stat_percent",
        "regeneration",
        "no_range_penalty",
        "shoot_when_adjacent",
        "no_retreat",
        "school_mastery",
        "spell_damage",
        "spell_book",
        "magic_resistance",
        "spell_duration",
        "spells",
        "spell_mastery",
        "scroll",
        "no_casting",
        "spell_level_limit",
        "spell_immunity",
        "spell_level_immunity",
        "mind_immunity",
        "negate_immunities",
        "opening_spell",
        "eagle_eye",
        "necromancy",
        "experience_gain",
        "max_mana",
        "join_bonus",
        "raises",
        "ballistics",
        "learn_level",
        "pathfinding",
        "land_movement",
        "land_movement_points",
        "flying",
        "water_walking",
        "scouting_radius",
        "meeting_spells",
        "gives_spell_book",
        "navigation",
        "sea_movement_points",
        "free_boarding",
    ];

    /// Stable code of the variant (canonical hash, duplicate checks).
    pub const fn code(&self) -> u8 {
        match self {
            Self::Morale { .. } => 0,
            Self::Luck { .. } => 1,
            Self::RangedDamage { .. } => 2,
            Self::MeleeDamage { .. } => 3,
            Self::DamageReduction { .. } => 4,
            Self::SurrenderDiscount { .. } => 5,
            Self::Tactics { .. } => 6,
            Self::Artillery { .. } => 7,
            Self::FirstAid { .. } => 8,
            Self::CreatureStat { .. } => 9,
            Self::SkillSpecialty { .. } => 10,
            Self::CreatureSpecialty { .. } => 11,
            Self::SpellSpecialty { .. } => 12,
            Self::PrimarySkills { .. } => 13,
            Self::MoraleAtMost { .. } => 14,
            Self::LuckAtMost { .. } => 15,
            Self::AlignmentGroup { .. } => 16,
            Self::CreatureStatPercent { .. } => 17,
            Self::Regeneration { .. } => 18,
            Self::NoRangePenalty { .. } => 19,
            Self::ShootWhenAdjacent { .. } => 20,
            Self::NoRetreat => 21,
            Self::SchoolMastery { .. } => 22,
            Self::SpellDamage { .. } => 23,
            Self::SpellBook => 24,
            Self::MagicResistance { .. } => 25,
            Self::SpellDuration { .. } => 26,
            Self::Spells { .. } => 27,
            Self::SpellMastery { .. } => 28,
            Self::Scroll => 29,
            Self::NoCasting => 30,
            Self::SpellLevelLimit { .. } => 31,
            Self::SpellImmunity { .. } => 32,
            Self::SpellLevelImmunity { .. } => 33,
            Self::MindImmunity => 34,
            Self::NegateImmunities => 35,
            Self::OpeningSpell { .. } => 36,
            Self::EagleEye { .. } => 37,
            Self::Necromancy { .. } => 38,
            Self::ExperienceGain { .. } => 39,
            Self::MaxMana { .. } => 40,
            Self::JoinBonus { .. } => 41,
            Self::Raises { .. } => 42,
            Self::Ballistics { .. } => 43,
            Self::LearnLevel { .. } => 44,
            Self::Pathfinding { .. } => 45,
            Self::LandMovement { .. } => 46,
            Self::LandMovementPoints { .. } => 47,
            Self::Flying { .. } => 48,
            Self::WaterWalking { .. } => 49,
            Self::ScoutingRadius { .. } => 50,
            Self::MeetingSpells { .. } => 51,
            Self::GivesSpellBook => 52,
            Self::Navigation { .. } => 53,
            Self::SeaMovementPoints { .. } => 54,
            Self::FreeBoarding => 55,
        }
    }

    /// Pack name of the variant.
    pub fn name(&self) -> &'static str {
        // `code` is always below `NAMES.len()`.
        Self::NAMES
            .get(usize::from(self.code()))
            .copied()
            .unwrap_or_default()
    }

    /// Whether the variant describes a hero's specialty (allowed in specialties only).
    pub const fn is_specialty(&self) -> bool {
        matches!(
            self,
            Self::SkillSpecialty { .. }
                | Self::CreatureSpecialty { .. }
                | Self::SpellSpecialty { .. }
        )
    }

    /// Maps the references.
    pub fn try_map<C2, K2, S2, F2, L2, E>(
        &self,
        mut creature: impl FnMut(&C) -> Result<C2, E>,
        skill: impl FnOnce(&K) -> Result<K2, E>,
        spell: impl FnOnce(&S) -> Result<S2, E>,
        faction: impl FnMut(&F) -> Result<F2, E>,
        school: impl FnOnce(&L) -> Result<L2, E>,
    ) -> Result<HeroEffect<C2, K2, S2, F2, L2>, E> {
        Ok(match self {
            Self::Morale { amount, affects } => HeroEffect::Morale {
                amount: *amount,
                affects: *affects,
            },
            Self::Luck { amount, affects } => HeroEffect::Luck {
                amount: *amount,
                affects: *affects,
            },
            Self::RangedDamage { bp } => HeroEffect::RangedDamage { bp: *bp },
            Self::MeleeDamage { bp } => HeroEffect::MeleeDamage { bp: *bp },
            Self::DamageReduction { bp } => HeroEffect::DamageReduction { bp: *bp },
            Self::SurrenderDiscount { bp } => HeroEffect::SurrenderDiscount { bp: *bp },
            Self::Tactics { level } => HeroEffect::Tactics { level: *level },
            Self::Artillery {
                level,
                shots,
                double_damage_chance_bp,
            } => HeroEffect::Artillery {
                level: *level,
                shots: *shots,
                double_damage_chance_bp: *double_damage_chance_bp,
            },
            Self::FirstAid { heal_bp } => HeroEffect::FirstAid { heal_bp: *heal_bp },
            Self::CreatureStat {
                creatures,
                stat,
                amount,
            } => HeroEffect::CreatureStat {
                creatures: creatures.try_map(creature)?,
                stat: *stat,
                amount: *amount,
            },
            Self::SkillSpecialty {
                skill: k,
                bp_per_level,
            } => HeroEffect::SkillSpecialty {
                skill: skill(k)?,
                bp_per_level: *bp_per_level,
            },
            Self::CreatureSpecialty {
                creature: c,
                bp_per_step,
                speed,
            } => HeroEffect::CreatureSpecialty {
                creature: creature(c)?,
                bp_per_step: *bp_per_step,
                speed: *speed,
            },
            Self::SpellSpecialty {
                spell: s,
                bp_per_level,
            } => HeroEffect::SpellSpecialty {
                spell: spell(s)?,
                bp_per_level: *bp_per_level,
            },
            Self::PrimarySkills {
                attack,
                defense,
                power,
                knowledge,
            } => HeroEffect::PrimarySkills {
                attack: *attack,
                defense: *defense,
                power: *power,
                knowledge: *knowledge,
            },
            Self::MoraleAtMost { value, affects } => HeroEffect::MoraleAtMost {
                value: *value,
                affects: *affects,
            },
            Self::LuckAtMost { value, affects } => HeroEffect::LuckAtMost {
                value: *value,
                affects: *affects,
            },
            Self::AlignmentGroup { factions } => HeroEffect::AlignmentGroup {
                factions: factions.iter().map(faction).collect::<Result<_, _>>()?,
            },
            Self::CreatureStatPercent {
                creatures,
                stat,
                bp,
            } => HeroEffect::CreatureStatPercent {
                creatures: creatures.try_map(creature)?,
                stat: *stat,
                bp: *bp,
            },
            Self::Regeneration {
                creatures,
                hit_points,
            } => HeroEffect::Regeneration {
                creatures: creatures.try_map(creature)?,
                hit_points: *hit_points,
            },
            Self::NoRangePenalty { creatures } => HeroEffect::NoRangePenalty {
                creatures: creatures.try_map(creature)?,
            },
            Self::ShootWhenAdjacent { creatures } => HeroEffect::ShootWhenAdjacent {
                creatures: creatures.try_map(creature)?,
            },
            Self::NoRetreat => HeroEffect::NoRetreat,
            Self::SchoolMastery { school: l, level } => HeroEffect::SchoolMastery {
                school: school(l)?,
                level: *level,
            },
            Self::SpellDamage { scope, bp } => HeroEffect::SpellDamage {
                scope: scope.try_map(school, spell)?,
                bp: *bp,
            },
            Self::SpellBook => HeroEffect::SpellBook,
            Self::MagicResistance { bp } => HeroEffect::MagicResistance { bp: *bp },
            Self::SpellDuration { rounds } => HeroEffect::SpellDuration { rounds: *rounds },
            Self::Spells { grant } => HeroEffect::Spells {
                grant: grant.try_map(school, spell)?,
            },
            Self::SpellMastery { spell: s, level } => HeroEffect::SpellMastery {
                spell: spell(s)?,
                level: *level,
            },
            Self::Scroll => HeroEffect::Scroll,
            Self::NoCasting => HeroEffect::NoCasting,
            Self::SpellLevelLimit { level } => HeroEffect::SpellLevelLimit { level: *level },
            Self::SpellImmunity { spell: s } => HeroEffect::SpellImmunity { spell: spell(s)? },
            Self::SpellLevelImmunity { level } => HeroEffect::SpellLevelImmunity { level: *level },
            Self::MindImmunity => HeroEffect::MindImmunity,
            Self::NegateImmunities => HeroEffect::NegateImmunities,
            Self::OpeningSpell {
                spell: s,
                mastery,
                power,
            } => HeroEffect::OpeningSpell {
                spell: spell(s)?,
                mastery: *mastery,
                power: *power,
            },
            Self::EagleEye { bp, max_level } => HeroEffect::EagleEye {
                bp: *bp,
                max_level: *max_level,
            },
            Self::Necromancy { bp, unless_skill } => HeroEffect::Necromancy {
                bp: *bp,
                unless_skill: unless_skill.as_ref().map(skill).transpose()?,
            },
            Self::ExperienceGain { bp } => HeroEffect::ExperienceGain { bp: *bp },
            Self::MaxMana { bp } => HeroEffect::MaxMana { bp: *bp },
            Self::JoinBonus { level } => HeroEffect::JoinBonus { level: *level },
            Self::Raises {
                priority,
                creature: raised,
            } => HeroEffect::Raises {
                priority: *priority,
                creature: raised.try_map(&mut creature, skill)?,
            },
            Self::Ballistics { level } => HeroEffect::Ballistics { level: *level },
            Self::LearnLevel { levels } => HeroEffect::LearnLevel { levels: *levels },
            Self::Pathfinding { level } => HeroEffect::Pathfinding { level: *level },
            Self::LandMovement { bp } => HeroEffect::LandMovement { bp: *bp },
            Self::LandMovementPoints { points } => {
                HeroEffect::LandMovementPoints { points: *points }
            }
            Self::Flying { level } => HeroEffect::Flying { level: *level },
            Self::WaterWalking { level } => HeroEffect::WaterWalking { level: *level },
            Self::ScoutingRadius { tiles } => HeroEffect::ScoutingRadius { tiles: *tiles },
            Self::MeetingSpells { level } => HeroEffect::MeetingSpells { level: *level },
            Self::GivesSpellBook => HeroEffect::GivesSpellBook,
            Self::Navigation { level } => HeroEffect::Navigation { level: *level },
            Self::SeaMovementPoints { points } => HeroEffect::SeaMovementPoints { points: *points },
            Self::FreeBoarding => HeroEffect::FreeBoarding,
        })
    }

    /// Why the parameters are invalid, if they are.
    pub fn parameter_error(&self) -> Option<&'static str> {
        match self {
            Self::Morale { amount: 0, .. } | Self::Luck { amount: 0, .. } => {
                Some("morale and luck amounts must not be 0")
            }
            Self::RangedDamage { bp: 0 } | Self::MeleeDamage { bp: 0 } => {
                Some("damage bonuses must be at least 1 bp")
            }
            Self::DamageReduction { bp } | Self::SurrenderDiscount { bp }
                if !(1..=FULL_BP).contains(bp) =>
            {
                Some("damage_reduction and surrender_discount need 1..=10000 bp")
            }
            Self::Tactics { level } if !(1..=MAX_SKILL_LEVEL).contains(level) => {
                Some("tactics level must be 1..=3")
            }
            Self::Artillery { level, .. } if !(1..=MAX_SKILL_LEVEL).contains(level) => {
                Some("artillery level must be 1..=3")
            }
            Self::Artillery { shots: 0, .. } => Some("artillery shots must be at least 1"),
            Self::Artillery {
                double_damage_chance_bp,
                ..
            } if *double_damage_chance_bp > FULL_BP => {
                Some("artillery double_damage_chance_bp must be at most 10000")
            }
            Self::FirstAid { heal_bp: 0 } => Some("first_aid heal_bp must be at least 1"),
            Self::CreatureStat { amount: 0, .. } => Some("creature_stat amount must not be 0"),
            Self::SkillSpecialty {
                bp_per_level: 0, ..
            } => Some("skill_specialty bp_per_level must be at least 1"),
            Self::CreatureSpecialty { bp_per_step: 0, .. } => {
                Some("creature_specialty bp_per_step must be at least 1")
            }
            Self::PrimarySkills {
                attack: 0,
                defense: 0,
                power: 0,
                knowledge: 0,
            } => Some("primary_skills needs a value that is not 0"),
            Self::AlignmentGroup { factions }
                if !(2..=MAX_ALIGNMENT_FACTIONS).contains(&factions.len()) =>
            {
                Some("alignment_group needs 2..=16 factions")
            }
            Self::CreatureStatPercent { bp: 0, .. } => {
                Some("creature_stat_percent bp must be at least 1")
            }
            Self::Regeneration { hit_points: 0, .. } => {
                Some("regeneration hit_points must be at least 1")
            }
            Self::SpellSpecialty {
                bp_per_level: 0, ..
            } => Some("spell_specialty bp_per_level must be at least 1"),
            Self::SchoolMastery { level, .. } if !(1..=MAX_SKILL_LEVEL).contains(level) => {
                Some("school_mastery level must be 1..=3")
            }
            Self::SpellDamage { bp: 0, .. } => Some("spell_damage bp must be at least 1"),
            Self::MagicResistance { bp } if !(1..=FULL_BP).contains(bp) => {
                Some("magic_resistance bp must be 1..=10000")
            }
            Self::SpellDuration { rounds: 0 } => Some("spell_duration rounds must be at least 1"),
            Self::Spells {
                grant: SpellGrant::Level(level),
            } if !(1..=MAX_SPELL_LEVEL).contains(level) => Some("spells level must be 1..=5"),
            Self::SpellMastery { level, .. } if !(1..=MAX_SKILL_LEVEL).contains(level) => {
                Some("spell_mastery level must be 1..=3")
            }
            Self::SpellLevelLimit { level } if !(1..MAX_SPELL_LEVEL).contains(level) => {
                Some("spell_level_limit level must be 1..=4")
            }
            Self::SpellLevelImmunity { level } if !(1..=MAX_SPELL_LEVEL).contains(level) => {
                Some("spell_level_immunity level must be 1..=5")
            }
            Self::OpeningSpell { mastery, .. } if *mastery > MAX_SKILL_LEVEL => {
                Some("opening_spell mastery must be 0..=3")
            }
            Self::OpeningSpell { power: 0, .. } => Some("opening_spell power must be at least 1"),
            Self::EagleEye { bp, .. } if !(1..=FULL_BP).contains(bp) => {
                Some("eagle_eye bp must be 1..=10000")
            }
            Self::EagleEye { max_level, .. } if *max_level > MAX_SPELL_LEVEL => {
                Some("eagle_eye max_level must be 0..=5")
            }
            Self::Necromancy { bp, .. } if !(1..=FULL_BP).contains(bp) => {
                Some("necromancy bp must be 1..=10000")
            }
            Self::ExperienceGain { bp: 0 } | Self::MaxMana { bp: 0 } => {
                Some("experience_gain and max_mana need at least 1 bp")
            }
            Self::JoinBonus { level } if !(1..=MAX_SKILL_LEVEL).contains(level) => {
                Some("join_bonus level must be 1..=3")
            }
            Self::Ballistics { level } if !(1..=MAX_SKILL_LEVEL).contains(level) => {
                Some("ballistics level must be 1..=3")
            }
            Self::LearnLevel { levels } if !(1..=MAX_SPELL_LEVEL).contains(levels) => {
                Some("learn_level levels must be 1..=5")
            }
            Self::Pathfinding { level } if !(1..=MAX_SKILL_LEVEL).contains(level) => {
                Some("pathfinding level must be 1..=3")
            }
            Self::LandMovement { bp: 0 } => Some("land_movement bp must be at least 1"),
            Self::LandMovementPoints { points: 0 } => {
                Some("land_movement_points points must be at least 1")
            }
            Self::Flying { level } | Self::WaterWalking { level } if *level > MAX_SKILL_LEVEL => {
                Some("flying and water_walking levels must be 0..=3")
            }
            Self::ScoutingRadius { tiles: 0 } => Some("scouting_radius tiles must be at least 1"),
            Self::MeetingSpells { level } if !(1..=MAX_SPELL_LEVEL).contains(level) => {
                Some("meeting_spells level must be 1..=5")
            }
            Self::Navigation { level } if !(1..=MAX_SKILL_LEVEL).contains(level) => {
                Some("navigation level must be 1..=3")
            }
            Self::SeaMovementPoints { points: 0 } => {
                Some("sea_movement_points points must be at least 1")
            }
            _ => None,
        }
    }
}

impl<C: PartialEq, K, S: PartialEq, F, L: PartialEq> HeroEffect<C, K, S, F, L> {
    /// Whether two effects may not both appear in one list: the same variant, except morale or
    /// luck (and their caps) for different sides, creature-selecting kinds for different
    /// creatures (or statistics), school masteries for different schools, spell-damage
    /// bonuses for different scopes and spell grants, masteries, immunities and opening spells
    /// for different spells.
    fn collides(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Morale { affects: a, .. }, Self::Morale { affects: b, .. })
            | (Self::Luck { affects: a, .. }, Self::Luck { affects: b, .. })
            | (Self::MoraleAtMost { affects: a, .. }, Self::MoraleAtMost { affects: b, .. })
            | (Self::LuckAtMost { affects: a, .. }, Self::LuckAtMost { affects: b, .. }) => a == b,
            (
                Self::CreatureStatPercent {
                    creatures: a,
                    stat: x,
                    ..
                },
                Self::CreatureStatPercent {
                    creatures: b,
                    stat: y,
                    ..
                },
            ) => a == b && x == y,
            (Self::Regeneration { creatures: a, .. }, Self::Regeneration { creatures: b, .. })
            | (Self::NoRangePenalty { creatures: a }, Self::NoRangePenalty { creatures: b })
            | (
                Self::ShootWhenAdjacent { creatures: a },
                Self::ShootWhenAdjacent { creatures: b },
            ) => a == b,
            (
                Self::CreatureStat {
                    creatures: a,
                    stat: x,
                    ..
                },
                Self::CreatureStat {
                    creatures: b,
                    stat: y,
                    ..
                },
            ) => a == b && x == y,
            (Self::SchoolMastery { school: a, .. }, Self::SchoolMastery { school: b, .. }) => {
                a == b
            }
            (Self::SpellDamage { scope: a, .. }, Self::SpellDamage { scope: b, .. }) => a == b,
            (Self::Spells { grant: a }, Self::Spells { grant: b }) => a == b,
            (Self::SpellMastery { spell: a, .. }, Self::SpellMastery { spell: b, .. })
            | (Self::SpellImmunity { spell: a }, Self::SpellImmunity { spell: b })
            | (Self::OpeningSpell { spell: a, .. }, Self::OpeningSpell { spell: b, .. }) => a == b,
            _ => self.code() == other.code(),
        }
    }
}

/// Why an effect list is invalid, if it is: too long, invalid parameters, an effect listed twice,
/// or a specialty variant where `specialty` is false.
pub(crate) fn list_error<C: PartialEq, K, S: PartialEq, F, L: PartialEq>(
    list: &[HeroEffect<C, K, S, F, L>],
    specialty: bool,
) -> Option<&'static str> {
    if list.len() > MAX_EFFECTS {
        return Some("more than 16 effects");
    }
    if let Some(e) = list.iter().find_map(HeroEffect::parameter_error) {
        return Some(e);
    }
    if !specialty && list.iter().any(HeroEffect::is_specialty) {
        return Some("specialty effects are allowed in a hero's specialty only");
    }
    for (i, a) in list.iter().enumerate() {
        let earlier = list.get(..i).unwrap_or(&[]);
        if earlier.iter().any(|b| a.collides(b)) {
            return Some("an effect is listed twice");
        }
    }
    None
}

/// Why an effect list of a skill level or a specialty is invalid for its spell effects, if it
/// is: a `scroll` belongs to an artifact (each copy holds its spell, HER-ART-003).
pub(crate) fn scroll_error<C, K, S, F, L>(
    list: &[HeroEffect<C, K, S, F, L>],
) -> Option<&'static str> {
    list.iter()
        .any(|e| matches!(e, HeroEffect::Scroll))
        .then_some("scroll is allowed on artifacts only")
}

const fn signed(v: i8) -> u8 {
    v.to_le_bytes()[0]
}

const fn affects_code(a: Affects) -> u8 {
    a as u8
}

fn hash_selector(creatures: CreatureSelector<CreatureTypeId>, h: &mut StateHasher) {
    match creatures {
        CreatureSelector::All => h.write_u8(0),
        CreatureSelector::Flag(flag) => {
            h.write_u8(1);
            h.write_u32(flag.bit());
        }
        CreatureSelector::Creature(c) => {
            h.write_u8(2);
            c.hash_into(h);
        }
    }
}

impl CanonicalHash for CompiledHeroEffect {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(self.code());
        match self {
            Self::Morale { amount, affects } | Self::Luck { amount, affects } => {
                h.write_u8(signed(*amount));
                h.write_u8(affects_code(*affects));
            }
            Self::RangedDamage { bp }
            | Self::MeleeDamage { bp }
            | Self::DamageReduction { bp }
            | Self::SurrenderDiscount { bp } => h.write_u16(*bp),
            Self::Tactics { level } => h.write_u8(*level),
            Self::Artillery {
                level,
                shots,
                double_damage_chance_bp,
            } => {
                h.write_u8(*level);
                h.write_u8(*shots);
                h.write_u16(*double_damage_chance_bp);
            }
            Self::FirstAid { heal_bp } => h.write_u16(*heal_bp),
            Self::CreatureStat {
                creatures,
                stat,
                amount,
            } => {
                hash_selector(*creatures, h);
                h.write_u8(*stat as u8);
                h.write_bytes(&amount.to_le_bytes());
            }
            Self::SkillSpecialty {
                skill,
                bp_per_level,
            } => {
                skill.hash_into(h);
                h.write_u16(*bp_per_level);
            }
            Self::CreatureSpecialty {
                creature,
                bp_per_step,
                speed,
            } => {
                creature.hash_into(h);
                h.write_u16(*bp_per_step);
                h.write_u8(*speed);
            }
            Self::SpellSpecialty {
                spell,
                bp_per_level,
            } => {
                spell.hash_into(h);
                h.write_u16(*bp_per_level);
            }
            Self::PrimarySkills {
                attack,
                defense,
                power,
                knowledge,
            } => {
                for v in [attack, defense, power, knowledge] {
                    h.write_u8(signed(*v));
                }
            }
            Self::MoraleAtMost { value, affects } | Self::LuckAtMost { value, affects } => {
                h.write_u8(signed(*value));
                h.write_u8(affects_code(*affects));
            }
            Self::AlignmentGroup { factions } => factions.hash_into(h),
            Self::CreatureStatPercent {
                creatures,
                stat,
                bp,
            } => {
                hash_selector(*creatures, h);
                h.write_u8(*stat as u8);
                h.write_u16(*bp);
            }
            Self::Regeneration {
                creatures,
                hit_points,
            } => {
                hash_selector(*creatures, h);
                h.write_u16(*hit_points);
            }
            Self::NoRangePenalty { creatures } | Self::ShootWhenAdjacent { creatures } => {
                hash_selector(*creatures, h);
            }
            Self::NoRetreat
            | Self::SpellBook
            | Self::Scroll
            | Self::NoCasting
            | Self::MindImmunity
            | Self::NegateImmunities
            | Self::GivesSpellBook
            | Self::FreeBoarding => {}
            Self::SpellLevelLimit { level } | Self::SpellLevelImmunity { level } => {
                h.write_u8(*level);
            }
            Self::SpellImmunity { spell } => spell.hash_into(h),
            Self::SchoolMastery { school, level } => {
                school.hash_into(h);
                h.write_u8(*level);
            }
            Self::SpellDamage { scope, bp } => {
                scope.hash_into(h);
                h.write_u16(*bp);
            }
            Self::MagicResistance { bp } => h.write_u16(*bp),
            Self::SpellDuration { rounds } => h.write_u8(*rounds),
            Self::Spells { grant } => grant.hash_into(h),
            Self::SpellMastery { spell, level } => {
                spell.hash_into(h);
                h.write_u8(*level);
            }
            Self::OpeningSpell {
                spell,
                mastery,
                power,
            } => {
                spell.hash_into(h);
                h.write_u8(*mastery);
                h.write_u16(*power);
            }
            Self::EagleEye { bp, max_level } => {
                h.write_u16(*bp);
                h.write_u8(*max_level);
            }
            Self::Necromancy { bp, unless_skill } => {
                h.write_u16(*bp);
                unless_skill.hash_into(h);
            }
            Self::ExperienceGain { bp } | Self::MaxMana { bp } => h.write_u16(*bp),
            Self::JoinBonus { level }
            | Self::Ballistics { level }
            | Self::Pathfinding { level }
            | Self::Flying { level }
            | Self::WaterWalking { level }
            | Self::MeetingSpells { level }
            | Self::Navigation { level } => h.write_u8(*level),
            Self::LearnLevel { levels } => h.write_u8(*levels),
            Self::LandMovement { bp } => h.write_u16(*bp),
            Self::LandMovementPoints { points } | Self::SeaMovementPoints { points } => {
                h.write_u16(*points);
            }
            Self::ScoutingRadius { tiles } => h.write_u8(*tiles),
            Self::Raises { priority, creature } => {
                h.write_u8(*priority);
                creature.hash_into(h);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::vec::Vec;

    use super::*;

    type E = HeroEffect<u8, u8, u8, u8, u8>;

    fn every() -> Vec<E> {
        std::vec![
            E::Morale {
                amount: 1,
                affects: Affects::Own,
            },
            E::Luck {
                amount: -1,
                affects: Affects::Both,
            },
            E::RangedDamage { bp: 1000 },
            E::MeleeDamage { bp: 1000 },
            E::DamageReduction { bp: 500 },
            E::SurrenderDiscount { bp: 2000 },
            E::Tactics { level: 3 },
            E::Artillery {
                level: 2,
                shots: 2,
                double_damage_chance_bp: 5000,
            },
            E::FirstAid { heal_bp: 10_000 },
            E::CreatureStat {
                creatures: CreatureSelector::Creature(1),
                stat: CreatureStat::Attack,
                amount: 5,
            },
            E::SkillSpecialty {
                skill: 2,
                bp_per_level: 500,
            },
            E::CreatureSpecialty {
                creature: 3,
                bp_per_step: 500,
                speed: 1,
            },
            E::SpellSpecialty {
                spell: 4,
                bp_per_level: 300,
            },
            E::PrimarySkills {
                attack: 12,
                defense: -3,
                power: 0,
                knowledge: 0,
            },
            E::MoraleAtMost {
                value: 0,
                affects: Affects::Both,
            },
            E::LuckAtMost {
                value: 0,
                affects: Affects::Both,
            },
            E::AlignmentGroup {
                factions: std::vec![1, 2],
            },
            E::CreatureStatPercent {
                creatures: CreatureSelector::Flag(CreatureFlag::Alive),
                stat: CreatureStat::HitPoints,
                bp: 2500,
            },
            E::Regeneration {
                creatures: CreatureSelector::Flag(CreatureFlag::Alive),
                hit_points: 50,
            },
            E::NoRangePenalty {
                creatures: CreatureSelector::Flag(CreatureFlag::Shooter),
            },
            E::ShootWhenAdjacent {
                creatures: CreatureSelector::Flag(CreatureFlag::Shooter),
            },
            E::NoRetreat,
            E::SchoolMastery {
                school: 5,
                level: 3,
            },
            E::SpellDamage {
                scope: SpellScope::School(5),
                bp: 5000,
            },
            E::SpellBook,
            E::MagicResistance { bp: 500 },
            E::SpellDuration { rounds: 2 },
            E::Spells {
                grant: SpellGrant::School(1),
            },
            E::SpellMastery { spell: 1, level: 3 },
            E::Scroll,
            E::NoCasting,
            E::SpellLevelLimit { level: 2 },
            E::SpellImmunity { spell: 3 },
            E::SpellLevelImmunity { level: 4 },
            E::MindImmunity,
            E::NegateImmunities,
            E::OpeningSpell {
                spell: 4,
                mastery: 3,
                power: 50,
            },
            E::EagleEye {
                bp: 4000,
                max_level: 2,
            },
            E::Necromancy {
                bp: 1000,
                unless_skill: Some(3),
            },
            E::ExperienceGain { bp: 500 },
            E::MaxMana { bp: 2500 },
            E::JoinBonus { level: 1 },
            E::Raises {
                priority: 1,
                creature: RaisedCreature::ByLevel {
                    skill: 2,
                    by_level: [1, 2, 3, 4],
                },
            },
            E::Ballistics { level: 2 },
            E::LearnLevel { levels: 2 },
            E::Pathfinding { level: 1 },
            E::LandMovement { bp: 1000 },
            E::LandMovementPoints { points: 600 },
            E::Flying { level: 3 },
            E::WaterWalking { level: 0 },
            E::ScoutingRadius { tiles: 1 },
            E::MeetingSpells { level: 2 },
            E::GivesSpellBook,
            E::Navigation { level: 1 },
            E::SeaMovementPoints { points: 500 },
            E::FreeBoarding,
        ]
    }

    #[test]
    fn names_follow_codes_and_lists_validate() {
        let all = every();
        for (i, e) in all.iter().enumerate() {
            assert_eq!(usize::from(e.code()), i);
            assert_eq!(e.name(), E::NAMES[i]);
            assert_eq!(e.parameter_error(), None, "{e:?}");
            assert_eq!(e.is_specialty(), (10..=12).contains(&e.code()), "{e:?}");
        }
        let (old, new) = all.split_at(13);
        assert_eq!(list_error(old, true), None);
        for chunk in new.chunks(MAX_EFFECTS) {
            assert_eq!(list_error(chunk, false), None);
        }
        assert_eq!(
            list_error(old, false),
            Some("specialty effects are allowed in a hero's specialty only")
        );
        let mut twice = std::vec![all[2].clone(), all[2].clone()];
        assert_eq!(list_error(&twice, false), Some("an effect is listed twice"));
        // Creature stats collide only for the same creatures and statistic.
        let stat = |creatures, stat| E::CreatureStat {
            creatures,
            stat,
            amount: 1,
        };
        twice = std::vec![
            stat(CreatureSelector::All, CreatureStat::Speed),
            stat(CreatureSelector::All, CreatureStat::Attack),
            stat(
                CreatureSelector::Flag(CreatureFlag::Dragon),
                CreatureStat::Attack
            ),
        ];
        assert_eq!(list_error(&twice, false), None);
        twice.push(stat(CreatureSelector::All, CreatureStat::Speed));
        assert_eq!(list_error(&twice, false), Some("an effect is listed twice"));
        // Morale and luck collide only for the same side.
        let luck = |affects| E::Luck { amount: 1, affects };
        let mut sides = std::vec![luck(Affects::Own), luck(Affects::Enemy)];
        assert_eq!(list_error(&sides, false), None);
        sides.push(luck(Affects::Own));
        assert_eq!(list_error(&sides, false), Some("an effect is listed twice"));
        let morale = |affects| E::Morale { amount: 1, affects };
        let both = [morale(Affects::Both), morale(Affects::Both)];
        assert_eq!(list_error(&both, false), Some("an effect is listed twice"));
        let caps: [fn(Affects) -> E; 2] = [
            |affects| E::LuckAtMost { value: 0, affects },
            |affects| E::MoraleAtMost { value: 0, affects },
        ];
        for cap in caps {
            let mut capped = std::vec![cap(Affects::Own), cap(Affects::Both)];
            assert_eq!(list_error(&capped, false), None);
            capped.push(cap(Affects::Own));
            assert_eq!(
                list_error(&capped, false),
                Some("an effect is listed twice")
            );
        }
        // Creature-selecting kinds collide only for the same creatures (and statistic).
        let shooters = CreatureSelector::Flag(CreatureFlag::Shooter);
        let selecting: [fn(CreatureSelector<u8>) -> E; 4] = [
            |c| E::Regeneration {
                creatures: c,
                hit_points: 1,
            },
            |c| E::NoRangePenalty { creatures: c },
            |c| E::ShootWhenAdjacent { creatures: c },
            |c| E::CreatureStatPercent {
                creatures: c,
                stat: CreatureStat::Speed,
                bp: 1,
            },
        ];
        for make in selecting {
            let mut list = std::vec![make(CreatureSelector::All), make(shooters)];
            assert_eq!(list_error(&list, false), None);
            list.push(make(shooters));
            assert_eq!(list_error(&list, false), Some("an effect is listed twice"));
        }
        let percent = |stat| E::CreatureStatPercent {
            creatures: CreatureSelector::All,
            stat,
            bp: 1,
        };
        let stats = [percent(CreatureStat::Speed), percent(CreatureStat::Attack)];
        assert_eq!(list_error(&stats, false), None);
        let retreat = [E::NoRetreat, E::NoRetreat];
        assert_eq!(
            list_error(&retreat, false),
            Some("an effect is listed twice")
        );
        // School masteries collide only for the same school, spell damage for the same scope.
        let mastery = |school| E::SchoolMastery { school, level: 1 };
        let mut schools = std::vec![mastery(1), mastery(2)];
        assert_eq!(list_error(&schools, false), None);
        schools.push(mastery(1));
        assert_eq!(
            list_error(&schools, false),
            Some("an effect is listed twice")
        );
        let damage = |scope| E::SpellDamage { scope, bp: 1 };
        let mut scopes = std::vec![
            damage(SpellScope::All),
            damage(SpellScope::School(1)),
            damage(SpellScope::Spell(1)),
        ];
        assert_eq!(list_error(&scopes, false), None);
        scopes.push(damage(SpellScope::School(1)));
        assert_eq!(
            list_error(&scopes, false),
            Some("an effect is listed twice")
        );
        let books = [E::SpellBook, E::SpellBook];
        assert_eq!(list_error(&books, false), Some("an effect is listed twice"));
        // Opening spells collide only for the same spell.
        let opening = |spell| E::OpeningSpell {
            spell,
            mastery: 3,
            power: 50,
        };
        let mut openings = std::vec![opening(1), opening(2)];
        assert_eq!(list_error(&openings, false), None);
        openings.push(opening(2));
        assert_eq!(
            list_error(&openings, false),
            Some("an effect is listed twice")
        );
        // Spell grants collide only for the same grant, mastery raises for the same spell.
        let grant = |grant| E::Spells { grant };
        let mut grants = std::vec![
            grant(SpellGrant::School(1)),
            grant(SpellGrant::Level(1)),
            grant(SpellGrant::Spell(1)),
            grant(SpellGrant::Spell(2)),
        ];
        assert_eq!(list_error(&grants, false), None);
        grants.push(grant(SpellGrant::Level(1)));
        assert_eq!(
            list_error(&grants, false),
            Some("an effect is listed twice")
        );
        let raise = |spell| E::SpellMastery { spell, level: 3 };
        let mut raises = std::vec![raise(1), raise(2)];
        assert_eq!(list_error(&raises, false), None);
        raises.push(raise(1));
        assert_eq!(
            list_error(&raises, false),
            Some("an effect is listed twice")
        );
        let immunity = |spell| E::SpellImmunity { spell };
        let mut immunities = std::vec![immunity(1), immunity(2)];
        assert_eq!(list_error(&immunities, false), None);
        immunities.push(immunity(2));
        assert_eq!(
            list_error(&immunities, false),
            Some("an effect is listed twice")
        );
        let scrolls = [E::Scroll, E::Scroll];
        assert_eq!(
            list_error(&scrolls, false),
            Some("an effect is listed twice")
        );
        let long = std::vec![E::Tactics { level: 1 }; MAX_EFFECTS + 1];
        assert_eq!(list_error(&long, false), Some("more than 16 effects"));
        assert_eq!(
            list_error(&[E::Tactics { level: 0 }], false),
            Some("tactics level must be 1..=3")
        );
    }

    #[test]
    fn parameter_errors() {
        let bad = [
            E::Morale {
                amount: 0,
                affects: Affects::Own,
            },
            E::Luck {
                amount: 0,
                affects: Affects::Enemy,
            },
            E::RangedDamage { bp: 0 },
            E::MeleeDamage { bp: 0 },
            E::DamageReduction { bp: 0 },
            E::SurrenderDiscount { bp: 10_001 },
            E::Tactics { level: 4 },
            E::Artillery {
                level: 1,
                shots: 0,
                double_damage_chance_bp: 0,
            },
            E::Artillery {
                level: 1,
                shots: 1,
                double_damage_chance_bp: 10_001,
            },
            E::Artillery {
                level: 0,
                shots: 1,
                double_damage_chance_bp: 0,
            },
            E::Artillery {
                level: 4,
                shots: 1,
                double_damage_chance_bp: 0,
            },
            E::FirstAid { heal_bp: 0 },
            E::CreatureStat {
                creatures: CreatureSelector::All,
                stat: CreatureStat::HitPoints,
                amount: 0,
            },
            E::SkillSpecialty {
                skill: 0,
                bp_per_level: 0,
            },
            E::CreatureSpecialty {
                creature: 0,
                bp_per_step: 0,
                speed: 0,
            },
            E::PrimarySkills {
                attack: 0,
                defense: 0,
                power: 0,
                knowledge: 0,
            },
            E::AlignmentGroup {
                factions: std::vec![1],
            },
            E::AlignmentGroup {
                factions: std::vec![0; MAX_ALIGNMENT_FACTIONS + 1],
            },
            E::CreatureStatPercent {
                creatures: CreatureSelector::All,
                stat: CreatureStat::HitPoints,
                bp: 0,
            },
            E::Regeneration {
                creatures: CreatureSelector::All,
                hit_points: 0,
            },
            E::SpellSpecialty {
                spell: 0,
                bp_per_level: 0,
            },
            E::SchoolMastery {
                school: 0,
                level: 0,
            },
            E::SchoolMastery {
                school: 0,
                level: 4,
            },
            E::SpellDamage {
                scope: SpellScope::All,
                bp: 0,
            },
            E::MagicResistance { bp: 0 },
            E::MagicResistance { bp: 10_001 },
            E::SpellDuration { rounds: 0 },
            E::Spells {
                grant: SpellGrant::Level(0),
            },
            E::Spells {
                grant: SpellGrant::Level(6),
            },
            E::SpellMastery { spell: 0, level: 0 },
            E::SpellMastery { spell: 0, level: 4 },
            E::SpellLevelLimit { level: 0 },
            E::SpellLevelLimit { level: 5 },
            E::SpellLevelImmunity { level: 0 },
            E::SpellLevelImmunity { level: 6 },
            E::OpeningSpell {
                spell: 0,
                mastery: 4,
                power: 1,
            },
            E::OpeningSpell {
                spell: 0,
                mastery: 3,
                power: 0,
            },
            E::EagleEye {
                bp: 0,
                max_level: 2,
            },
            E::EagleEye {
                bp: 10_001,
                max_level: 2,
            },
            E::EagleEye {
                bp: 500,
                max_level: 6,
            },
            E::Necromancy {
                bp: 0,
                unless_skill: None,
            },
            E::Necromancy {
                bp: 10_001,
                unless_skill: Some(1),
            },
            E::ExperienceGain { bp: 0 },
            E::MaxMana { bp: 0 },
            E::JoinBonus { level: 0 },
            E::JoinBonus { level: 4 },
            E::Ballistics { level: 0 },
            E::Ballistics { level: 4 },
            E::LearnLevel { levels: 0 },
            E::LearnLevel { levels: 6 },
            E::Pathfinding { level: 0 },
            E::Pathfinding { level: 4 },
            E::LandMovement { bp: 0 },
            E::LandMovementPoints { points: 0 },
            E::Flying { level: 4 },
            E::WaterWalking { level: 4 },
            E::ScoutingRadius { tiles: 0 },
            E::MeetingSpells { level: 0 },
            E::MeetingSpells { level: 6 },
            E::Navigation { level: 0 },
            E::Navigation { level: 4 },
            E::SeaMovementPoints { points: 0 },
        ];
        for e in bad {
            assert!(e.parameter_error().is_some(), "{e:?}");
        }
    }

    #[test]
    fn names_round_trip() {
        for a in Affects::ALL {
            assert_eq!(Affects::from_name(a.name()), Some(a));
        }
        for s in CreatureStat::ALL {
            assert_eq!(CreatureStat::from_name(s.name()), Some(s));
        }
        assert_eq!(Affects::from_name("allies"), None);
        assert_eq!(CreatureStat::from_name("luck"), None);
    }

    #[test]
    fn every_compiled_kind_hashes_apart() {
        use h3_kernel::Handle;
        let mut seen = alloc::collections::BTreeSet::new();
        for e in every() {
            let compiled: CompiledHeroEffect = e
                .try_map(
                    |v| Ok::<_, ()>(Handle::from_raw(u16::from(*v))),
                    |v| Ok(Handle::from_raw(u16::from(*v))),
                    |v| Ok(Handle::from_raw(u16::from(*v))),
                    |v| Ok(Handle::from_raw(u16::from(*v))),
                    |v| Ok(Handle::from_raw(u16::from(*v))),
                )
                .unwrap();
            assert!(seen.insert(h3_kernel::hash::hash_of(&compiled)), "{e:?}");
        }
        // The two forms of `raises`, and its priority, hash apart too.
        let raises = |priority, creature| CompiledHeroEffect::Raises { priority, creature };
        for e in [
            raises(1, RaisedCreature::Creature(Handle::from_raw(2))),
            raises(0, RaisedCreature::Creature(Handle::from_raw(2))),
            raises(
                1,
                RaisedCreature::ByLevel {
                    skill: Handle::from_raw(2),
                    by_level: [1, 2, 3, 5].map(Handle::from_raw),
                },
            ),
        ] {
            assert!(seen.insert(h3_kernel::hash::hash_of(&e)), "{e:?}");
        }
        // A share with and without its skill condition hashes apart.
        let share = CompiledHeroEffect::Necromancy {
            bp: 1000,
            unless_skill: None,
        };
        assert!(seen.insert(h3_kernel::hash::hash_of(&share)), "{share:?}");
    }

    #[test]
    fn try_map_keeps_parameters_and_maps_references() {
        let plus = |v: &u8| Ok::<u16, ()>(u16::from(*v) + 100);
        for e in every() {
            let m = e.try_map(plus, plus, plus, plus, plus).unwrap();
            assert_eq!(m.code(), e.code());
            assert_eq!(m.parameter_error(), None);
        }
        let mapped = E::CreatureSpecialty {
            creature: 1,
            bp_per_step: 500,
            speed: 1,
        }
        .try_map(plus, plus, plus, plus, plus)
        .unwrap();
        assert_eq!(
            mapped,
            HeroEffect::CreatureSpecialty {
                creature: 101,
                bp_per_step: 500,
                speed: 1
            }
        );
        let fail = |_: &u8| Err::<u16, ()>(());
        assert_eq!(
            E::SpellSpecialty {
                spell: 1,
                bp_per_level: 300
            }
            .try_map(plus, plus, fail, plus, plus),
            Err(())
        );
        let in_school = E::SchoolMastery {
            school: 1,
            level: 2,
        };
        assert_eq!(in_school.try_map(plus, plus, plus, plus, fail), Err(()));
        assert_eq!(
            E::OpeningSpell {
                spell: 1,
                mastery: 3,
                power: 10
            }
            .try_map(plus, plus, fail, plus, plus),
            Err(())
        );
        // A share's skill condition is a skill reference.
        let share = |unless_skill| E::Necromancy {
            bp: 3000,
            unless_skill,
        };
        assert_eq!(
            share(Some(2)).try_map(plus, fail, plus, plus, plus),
            Err(())
        );
        assert_eq!(
            share(Some(2)).try_map(plus, plus, plus, plus, plus),
            Ok(HeroEffect::Necromancy {
                bp: 3000,
                unless_skill: Some(102)
            })
        );
        assert!(share(None).try_map(plus, fail, plus, plus, plus).is_ok());
        for scope in [SpellScope::School(1), SpellScope::Spell(1)] {
            let e = E::SpellDamage { scope, bp: 1 };
            assert_eq!(e.try_map(plus, plus, fail, plus, fail), Err(()), "{e:?}");
        }
        assert_eq!(
            E::SpellDamage {
                scope: SpellScope::Spell(1),
                bp: 7
            }
            .try_map(plus, plus, plus, plus, plus),
            Ok(HeroEffect::SpellDamage {
                scope: SpellScope::Spell(101),
                bp: 7
            })
        );
        assert_eq!(
            E::AlignmentGroup {
                factions: std::vec![1, 2]
            }
            .try_map(plus, plus, plus, plus, plus),
            Ok(HeroEffect::AlignmentGroup {
                factions: std::vec![101, 102]
            })
        );
        assert_eq!(
            E::AlignmentGroup {
                factions: std::vec![1, 2]
            }
            .try_map(plus, plus, plus, fail, plus),
            Err(())
        );
        // Every creature-selecting kind fails with its creature reference.
        let creature = CreatureSelector::Creature(1);
        let selecting = [
            E::CreatureStat {
                creatures: creature,
                stat: CreatureStat::Speed,
                amount: 1,
            },
            E::CreatureStatPercent {
                creatures: creature,
                stat: CreatureStat::Speed,
                bp: 1,
            },
            E::Regeneration {
                creatures: creature,
                hit_points: 1,
            },
            E::NoRangePenalty {
                creatures: creature,
            },
            E::ShootWhenAdjacent {
                creatures: creature,
            },
        ];
        for e in selecting {
            assert_eq!(e.try_map(fail, plus, plus, plus, plus), Err(()), "{e:?}");
        }
        assert_eq!(
            CreatureSelector::<u8>::Flag(CreatureFlag::Dragon).try_map(fail),
            Ok(CreatureSelector::Flag(CreatureFlag::Dragon))
        );
        // Spell grants map their school or spell and keep a level; a mastery raise its spell.
        for (grant, mapped) in [
            (SpellGrant::School(1), SpellGrant::School(101)),
            (SpellGrant::Level(5), SpellGrant::Level(5)),
            (SpellGrant::Spell(2), SpellGrant::Spell(102)),
        ] {
            assert_eq!(
                E::Spells { grant }.try_map(plus, plus, plus, plus, plus),
                Ok(HeroEffect::Spells { grant: mapped })
            );
        }
        assert_eq!(
            E::Spells {
                grant: SpellGrant::School(1)
            }
            .try_map(plus, plus, plus, plus, fail),
            Err(())
        );
        assert_eq!(
            E::Spells {
                grant: SpellGrant::Spell(1)
            }
            .try_map(plus, plus, fail, plus, plus),
            Err(())
        );
        assert_eq!(
            E::SpellMastery { spell: 1, level: 3 }.try_map(plus, plus, fail, plus, plus),
            Err(())
        );
        assert_eq!(
            E::SpellImmunity { spell: 1 }.try_map(plus, plus, fail, plus, plus),
            Err(())
        );
        // `raises` maps its creature, or its skill and every level's creature.
        let single = E::Raises {
            priority: 2,
            creature: RaisedCreature::Creature(1),
        };
        assert_eq!(
            single.try_map(plus, fail, plus, plus, plus),
            Ok(HeroEffect::Raises {
                priority: 2,
                creature: RaisedCreature::Creature(101),
            })
        );
        assert_eq!(single.try_map(fail, plus, plus, plus, plus), Err(()));
        let by_level = E::Raises {
            priority: 1,
            creature: RaisedCreature::ByLevel {
                skill: 5,
                by_level: [1, 2, 3, 4],
            },
        };
        assert_eq!(
            by_level.try_map(plus, plus, plus, plus, plus),
            Ok(HeroEffect::Raises {
                priority: 1,
                creature: RaisedCreature::ByLevel {
                    skill: 105,
                    by_level: [101, 102, 103, 104],
                },
            })
        );
        assert_eq!(by_level.try_map(fail, plus, plus, plus, plus), Err(()));
        assert_eq!(by_level.try_map(plus, fail, plus, plus, plus), Err(()));
        // A failure at any level stops the mapping.
        for bad in 1..=4 {
            let picky = |v: &u8| if *v == bad { Err(()) } else { plus(v) };
            assert_eq!(
                by_level.try_map(picky, plus, plus, plus, plus),
                Err(()),
                "{bad}"
            );
        }
    }
}
