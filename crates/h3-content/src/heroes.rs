//! Hero content (E04a T3, ADR-0013): spell schools, spells (data rows), secondary skills, hero
//! classes and hero types, compiled from the packs' definitions.
//!
//! The compiler resolves references within each pack's dependency closure (like creatures,
//! ADR-0006) and validates every entry. Nothing here is read on the battle's hot path: the pure
//! translation in `h3-heroes` (E04a T4) turns a hero's details into the battle's `BattleHero`.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use h3_kernel::{CanonicalHash, Handle, StateHasher};

use crate::artifacts::{ArtifactSlot, ArtifactType};
use crate::effect::{
    self, CompiledHeroEffect, HeroEffect, HeroEffectDef, MAX_REGAIN, MAX_SKILL_LEVEL, SpellGrant,
};
use crate::error::CompileError;
use crate::flags::CreatureFlag;
use crate::key::ContentKey;
use crate::pack::{ContentPack, HeroParamsDef};
use crate::ruleset::{CreatureInfo, CreatureTypeId, FactionId, ResourceId};

/// Highest spell level.
pub const MAX_SPELL_LEVEL: u8 = 5;
/// Most spells in a ruleset (the battle keeps spell books as bitsets, E04b T2b).
pub const MAX_SPELLS: usize = 128;
/// Most spell schools (a hero keeps one mastery per school, its `school_mastery` effects).
pub const MAX_SPELL_SCHOOLS: usize = 4;
/// Most secondary skills a hero can have.
pub const MAX_HERO_SKILLS: usize = 8;

/// Where a spell is used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpellCategory {
    /// Cast by heroes on the adventure map.
    Adventure,
    /// Cast by heroes (and spellcasting creatures) in battle.
    Combat,
    /// A creature ability that works like a spell (never in a spell book).
    Ability,
}

impl SpellCategory {
    /// All values, in code order.
    pub const ALL: [Self; 3] = [Self::Adventure, Self::Combat, Self::Ability];

    /// Pack name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Adventure => "adventure",
            Self::Combat => "combat",
            Self::Ability => "ability",
        }
    }

    /// Looks a value up by its pack name.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.name() == name)
    }
}

/// A compiled spell school.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellSchool {
    /// Key.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
    /// Where a damage spell of several schools checks this school's protection under quirk
    /// Q-0007, lower first (pack format v6; BAT-MAG-013).
    pub protection_rank: u8,
}

/// Handle of a spell school in a [`crate::Ruleset`].
pub type SchoolId = Handle<SpellSchool>;

/// A compiled spell: its data row (E04a T3). What it does in battle is `h3-content-battle`'s
/// `BattleSpell` (R01).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spell {
    /// Key.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
    /// Where it is used.
    pub category: SpellCategory,
    /// Spell level.
    pub level: u8,
    /// Schools, in pack order.
    pub schools: Vec<SchoolId>,
    /// Mana cost by mastery: none, basic, advanced, expert.
    pub cost: [u16; 4],
    /// Power factor.
    pub power: u16,
    /// Base effect value by mastery.
    pub effect: [u16; 4],
    /// Only a grant naming the spell gives it, never one by school or level (pack format v30;
    /// Titan's Lightning Bolt, the spell flag `0x2000`, `installation.md` §7.22; HER-SPL-002).
    pub granted_by_name_only: bool,
}

/// Handle of a spell in a [`crate::Ruleset`].
pub type SpellId = Handle<Spell>;

/// A compiled secondary skill.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecondarySkill {
    /// Key.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
    /// Effects at Basic, Advanced and Expert (`levels[level - 1]`).
    pub levels: [Vec<CompiledHeroEffect>; 3],
}

impl SecondarySkill {
    /// The effects at `level` (1..=3); none for other levels.
    pub fn effects(&self, level: u8) -> &[CompiledHeroEffect] {
        usize::from(level)
            .checked_sub(1)
            .and_then(|i| self.levels.get(i))
            .map_or(&[], Vec::as_slice)
    }
}

/// Handle of a secondary skill in a [`crate::Ruleset`].
pub type SkillId = Handle<SecondarySkill>;

/// A compiled hero class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeroClass {
    /// Key.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
    /// Faction of its heroes.
    pub faction: FactionId,
    /// Starting primary skills: attack, defense, power, knowledge.
    pub primary: [u8; 4],
    /// Level-up chances in percent per primary skill, for levels 2–9 and 10+.
    pub primary_gain: [[u8; 4]; 2],
    /// Level-up weights of secondary skills, in pack order (no skill twice).
    pub skill_weights: Vec<(SkillId, u8)>,
    /// The levels after which a level-up offers Wisdom and a magic school if it may:
    /// `[wisdom, school]`, each at least 1 (E08 T26b; HER-LVL-004).
    pub offer_intervals: [u8; 2],
}

/// Handle of a hero class in a [`crate::Ruleset`].
pub type HeroClassId = Handle<HeroClass>;

/// One stack of a starting army.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartingStack {
    /// The creature type.
    pub creature: CreatureTypeId,
    /// Size range `[low, high]`.
    pub count: [u16; 2],
}

/// A compiled hero type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeroType {
    /// Key.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
    /// Hero class.
    pub class: HeroClassId,
    /// Starting secondary skills and levels, in display order.
    pub skills: Vec<(SkillId, u8)>,
    /// Starts with a spell book.
    pub spell_book: bool,
    /// Starting spells.
    pub spells: Vec<SpellId>,
    /// Starting army.
    pub army: Vec<StartingStack>,
    /// Starting war machines.
    pub war_machines: Vec<CreatureTypeId>,
    /// The specialty.
    pub specialty: Vec<CompiledHeroEffect>,
}

/// Handle of a hero type in a [`crate::Ruleset`].
pub type HeroTypeId = Handle<HeroType>;

/// Compiled hero parameters (Tier 2, E04a T4a): the limits of a hero's details
/// (`h3-heroes`) and of a battle hero's primary skills; since E08 T26b the experience of every
/// level and the skills a level-up offers when they are due.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeroParams {
    /// Lowest effective primary skills (attack, defense, power, knowledge); a lower total
    /// counts as this value (HER-DET-001).
    pub primary_min: [u8; 4],
    /// Highest primary skill, base and effective (HER-DET-001).
    pub primary_max: u8,
    /// Most artifacts in a hero's backpack (HER-ART-002).
    pub backpack: u16,
    /// The highest spell level a hero may learn without a `learn_level` effect (pack format
    /// v40, E08 T19a2b; the original's 2, Wisdom adding its level), 0..=5.
    pub learn_level: u8,
    /// The experience of levels 1 to `max_level + 1`, rising strictly: the pack's levels, then
    /// each step the previous one + a fifth of it, rounded down (E08 T26b; HER-EXP-002). The last
    /// entry bounds a hero's experience (A-7).
    pub experience: Vec<u32>,
    /// The highest level (E08 T26b; 74, A-7's fix).
    pub max_level: u8,
    /// The skill a level-up offers when the class's Wisdom interval is due (HER-LVL-004).
    pub wisdom: SkillId,
    /// The skills a level-up draws from when the class's school interval is due, in pack order.
    pub schools: Vec<SkillId>,
    /// The most spell points a reward record leaves a hero (E08 T59, pack format v48; the
    /// original's 999, HER-CHG-003): its mana is added and the total held between 0 and this, so
    /// a hero above it falls to it (D-38). It bounds no other gain.
    pub mana_ceiling: u16,
    /// The day's spell point regain without a `mana_regain` effect (E08 increment 9, pack format
    /// v50; HER-MAN-002; the original's 1), 0..=[`crate::MAX_REGAIN`].
    pub base_mana_regain: u16,
    /// Added to a regain grown by a skill specialty on its skill (E08 increment 9, pack format
    /// v50; HER-MAN-002; the original's 1), 0..=[`crate::MAX_REGAIN`].
    pub regain_specialist_bonus: u16,
}

/// The experience of levels 1 to `max_level + 1` from the pack's first levels (E08 T26b): at
/// least two, the first 0, rising strictly, no more than `max_level + 1`; each later step is the
/// previous one + a fifth of it, rounded down (the original's truncated × 1.2, survey
/// `E08-progression.md` §1.1), and every threshold must fit 32 bits.
fn thresholds(given: &[u32], max_level: u8) -> Result<Vec<u32>, CompileError> {
    let rising = CompileError::InvalidHeroParams(
        "experience must start at 0 and rise strictly over at least two levels",
    );
    let [.., before, last] = given else {
        return Err(rising);
    };
    if given.first() != Some(&0) || given.windows(2).any(|w| w.first() >= w.last()) {
        return Err(rising);
    }
    let levels = usize::from(max_level) + 1;
    if given.len() > levels {
        return Err(CompileError::InvalidHeroParams(
            "experience must list at most max_level + 1 levels",
        ));
    }
    let mut table = given.to_vec();
    let (mut at, mut step) = (u64::from(*last), u64::from(last - before));
    while table.len() < levels {
        step += step / 5;
        at += step;
        table.push(u32::try_from(at).map_err(|_| {
            CompileError::InvalidHeroParams("experience must fit 32 bits up to max_level + 1")
        })?);
    }
    Ok(table)
}

/// Validates hero parameters: every minimum at most the maximum (HER-DET-001), the learnable
/// level at most the highest spell level, the experience table ([`thresholds`]); resolves the
/// skills a level-up offers when due.
fn compile_params(
    def: &HeroParamsDef,
    refs: &Refs<'_>,
    skills: &BTreeMap<String, SkillId>,
) -> Result<HeroParams, CompileError> {
    if def.primary_min.iter().any(|&m| m > def.primary_max) {
        return Err(CompileError::InvalidHeroParams(
            "primary_min must not exceed primary_max",
        ));
    }
    if def.learn_level > MAX_SPELL_LEVEL {
        return Err(CompileError::InvalidHeroParams(
            "learn_level must be at most 5",
        ));
    }
    if def.base_mana_regain > MAX_REGAIN || def.regain_specialist_bonus > MAX_REGAIN {
        return Err(CompileError::InvalidHeroParams(
            "base_mana_regain and regain_specialist_bonus must be at most 999",
        ));
    }
    Ok(HeroParams {
        primary_min: def.primary_min,
        primary_max: def.primary_max,
        backpack: def.backpack,
        learn_level: def.learn_level,
        experience: thresholds(&def.experience, def.max_level)?,
        max_level: def.max_level,
        wisdom: refs.resolve(&def.wisdom, skills, "secondary skill")?,
        schools: distinct(refs, &def.schools, skills, "secondary skill")?,
        mana_ceiling: def.mana_ceiling,
        base_mana_regain: def.base_mana_regain,
        regain_specialist_bonus: def.regain_specialist_bonus,
    })
}

/// A table of compiled entries with their key index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Table<T> {
    pub(crate) items: Vec<T>,
    pub(crate) index: BTreeMap<String, Handle<T>>,
}

impl<T> Default for Table<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            index: BTreeMap::new(),
        }
    }
}

impl<T> Table<T> {
    /// Reserves the next handle for `key` (unique, in the pack's namespace).
    pub(crate) fn register(
        &mut self,
        key: &ContentKey,
        pack: &str,
        kind: &'static str,
    ) -> Result<Handle<T>, CompileError> {
        if key.pack() != pack {
            return Err(CompileError::KeyOutsidePack {
                key: String::from(key.as_str()),
                pack: String::from(pack),
            });
        }
        // Entries are pushed in registration order, so the next handle is the number registered.
        let handle =
            Handle::from_index(self.index.len()).ok_or(CompileError::TooManyEntries(kind))?;
        if self
            .index
            .insert(String::from(key.as_str()), handle)
            .is_some()
        {
            return Err(CompileError::DuplicateKey(String::from(key.as_str())));
        }
        Ok(handle)
    }

    pub(crate) fn get(&self, id: Handle<T>) -> Option<&T> {
        self.items.get(id.index())
    }

    pub(crate) fn by_key(&self, key: &str) -> Option<Handle<T>> {
        self.index.get(key).copied()
    }
}

/// All compiled hero content of a ruleset.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct HeroContent {
    pub(crate) schools: Table<SpellSchool>,
    pub(crate) spells: Table<Spell>,
    pub(crate) skills: Table<SecondarySkill>,
    pub(crate) classes: Table<HeroClass>,
    pub(crate) heroes: Table<HeroType>,
    pub(crate) slots: Table<ArtifactSlot>,
    pub(crate) artifacts: Table<ArtifactType>,
    pub(crate) params: Option<HeroParams>,
}

/// What the pack being compiled may refer to.
pub(crate) struct Refs<'a> {
    pub(crate) pack: &'a str,
    pub(crate) closure: &'a BTreeSet<String>,
    pub(crate) resources: &'a BTreeMap<String, ResourceId>,
    pub(crate) factions: &'a BTreeMap<String, FactionId>,
    pub(crate) creatures: &'a BTreeMap<String, CreatureTypeId>,
}

impl Refs<'_> {
    pub(crate) fn resolve<T>(
        &self,
        key: &ContentKey,
        index: &BTreeMap<String, Handle<T>>,
        kind: &'static str,
    ) -> Result<Handle<T>, CompileError> {
        crate::ruleset::resolve_in(self.pack, self.closure, key, index, kind)
    }
}

pub(crate) fn invalid(kind: &'static str, key: &ContentKey, reason: &'static str) -> CompileError {
    CompileError::InvalidContent {
        kind,
        key: String::from(key.as_str()),
        reason,
    }
}

/// Resolves `(key, value)` pairs, rejecting a key listed twice.
fn pairs<T>(
    refs: &Refs<'_>,
    list: &[(ContentKey, u8)],
    index: &BTreeMap<String, Handle<T>>,
    kind: &'static str,
) -> Result<Vec<(Handle<T>, u8)>, CompileError> {
    let mut out: Vec<(Handle<T>, u8)> = Vec::with_capacity(list.len());
    for (key, value) in list {
        let id = refs.resolve(key, index, kind)?;
        if out.iter().any(|(o, _)| *o == id) {
            return Err(CompileError::DuplicateKey(String::from(key.as_str())));
        }
        out.push((id, *value));
    }
    Ok(out)
}

/// Resolves a list of references.
fn all<T>(
    refs: &Refs<'_>,
    keys: &[ContentKey],
    index: &BTreeMap<String, Handle<T>>,
    kind: &'static str,
) -> Result<Vec<Handle<T>>, CompileError> {
    keys.iter().map(|k| refs.resolve(k, index, kind)).collect()
}

/// Resolves a list of references that must be distinct.
pub(crate) fn distinct<T>(
    refs: &Refs<'_>,
    keys: &[ContentKey],
    index: &BTreeMap<String, Handle<T>>,
    kind: &'static str,
) -> Result<Vec<Handle<T>>, CompileError> {
    let ids = all(refs, keys, index, kind)?;
    for (i, (id, key)) in ids.iter().zip(keys).enumerate() {
        if ids.get(..i).unwrap_or_default().contains(id) {
            return Err(CompileError::DuplicateKey(String::from(key.as_str())));
        }
    }
    Ok(ids)
}

impl HeroContent {
    /// Resolves the references of an effect list (an effect's factions must be distinct).
    pub(crate) fn effects(
        &self,
        refs: &Refs<'_>,
        list: &[HeroEffectDef],
    ) -> Result<Vec<CompiledHeroEffect>, CompileError> {
        list.iter()
            .map(|e| {
                let mut seen: Vec<FactionId> = Vec::new();
                e.try_map(
                    |c| refs.resolve(c, refs.creatures, "creature"),
                    |k| refs.resolve(k, &self.skills.index, "secondary skill"),
                    |s| refs.resolve(s, &self.spells.index, "spell"),
                    |f| {
                        let id = refs.resolve(f, refs.factions, "faction")?;
                        if seen.contains(&id) {
                            return Err(CompileError::DuplicateKey(String::from(f.as_str())));
                        }
                        seen.push(id);
                        Ok(id)
                    },
                    |l| refs.resolve(l, &self.schools.index, "spell school"),
                    |r| refs.resolve(r, refs.resources, "resource"),
                )
            })
            .collect()
    }

    /// Compiles the hero content of one pack (after its creatures).
    pub(crate) fn add_pack(
        &mut self,
        pack: &ContentPack,
        refs: &Refs<'_>,
    ) -> Result<(), CompileError> {
        for def in &pack.spell_schools {
            self.schools.register(&def.key, refs.pack, "spell school")?;
            self.schools.items.push(SpellSchool {
                key: def.key.clone(),
                name: def.name.clone(),
                protection_rank: def.protection_rank,
            });
        }
        for def in &pack.spells {
            self.spells.register(&def.key, refs.pack, "spell")?;
            let schools = distinct(refs, &def.schools, &self.schools.index, "spell school")?;
            self.spells.items.push(Spell {
                key: def.key.clone(),
                name: def.name.clone(),
                category: def.category,
                level: def.level,
                schools,
                cost: def.cost,
                power: def.power,
                effect: def.effect,
                granted_by_name_only: def.granted_by_name_only,
            });
        }
        // Skills may refer to each other (from T3b on), so all are registered first.
        for def in &pack.skills {
            self.skills
                .register(&def.key, refs.pack, "secondary skill")?;
        }
        for def in &pack.skills {
            let [basic, advanced, expert] = &def.levels;
            let levels = [
                self.effects(refs, basic)?,
                self.effects(refs, advanced)?,
                self.effects(refs, expert)?,
            ];
            self.skills.items.push(SecondarySkill {
                key: def.key.clone(),
                name: def.name.clone(),
                levels,
            });
        }
        self.add_artifacts(pack, refs)?;
        if let Some(def) = &pack.hero_params {
            if self.params.is_some() {
                return Err(CompileError::DuplicateHeroParams(String::from(refs.pack)));
            }
            self.params = Some(compile_params(def, refs, &self.skills.index)?);
        }
        for def in &pack.hero_classes {
            self.classes.register(&def.key, refs.pack, "hero class")?;
            self.classes.items.push(HeroClass {
                key: def.key.clone(),
                name: def.name.clone(),
                faction: refs.resolve(&def.faction, refs.factions, "faction")?,
                primary: def.primary,
                primary_gain: def.primary_gain,
                skill_weights: pairs(
                    refs,
                    &def.skill_weights,
                    &self.skills.index,
                    "secondary skill",
                )?,
                offer_intervals: def.offer_intervals,
            });
        }
        for def in &pack.heroes {
            self.heroes.register(&def.key, refs.pack, "hero type")?;
            let creatures: Vec<ContentKey> = def.army.iter().map(|s| s.creature.clone()).collect();
            let army = def
                .army
                .iter()
                .zip(all(refs, &creatures, refs.creatures, "creature")?)
                .map(|(s, creature)| StartingStack {
                    creature,
                    count: s.count,
                })
                .collect();
            let spells = distinct(refs, &def.spells, &self.spells.index, "spell")?;
            let machines = distinct(refs, &def.war_machines, refs.creatures, "creature")?;
            self.heroes.items.push(HeroType {
                key: def.key.clone(),
                name: def.name.clone(),
                class: refs.resolve(&def.class, &self.classes.index, "hero class")?,
                skills: pairs(refs, &def.skills, &self.skills.index, "secondary skill")?,
                spell_book: def.spell_book,
                spells,
                army,
                war_machines: machines,
                specialty: self.effects(refs, &def.specialty)?,
            });
        }
        Ok(())
    }

    /// Applies the pack's hero overrides to hero types of its closure (E04b T8e) and its hero
    /// class overrides to classes of its closure (E08 T26b). A gated one's target and references
    /// are resolved in every profile; it is applied (and then validated with its entry) only where
    /// `applies` says so (ADR-0007, amendment E04b T8e).
    pub(crate) fn add_overrides(
        &mut self,
        pack: &ContentPack,
        refs: &Refs<'_>,
        applies: impl Fn(Option<&str>) -> Result<bool, CompileError>,
    ) -> Result<(), CompileError> {
        for o in &pack.hero_overrides {
            let id = refs.resolve(&o.target, &self.heroes.index, "hero type")?;
            let apply = applies(o.fixes.as_deref())?;
            let specialty = o
                .patch
                .specialty
                .as_ref()
                .map(|list| self.effects(refs, list))
                .transpose()?;
            if let (true, Some(hero), Some(list)) =
                (apply, self.heroes.items.get_mut(id.index()), specialty)
            {
                hero.specialty = list;
            }
        }
        for o in &pack.hero_class_overrides {
            let id = refs.resolve(&o.target, &self.classes.index, "hero class")?;
            let apply = applies(o.fixes.as_deref())?;
            if let (true, Some(class), Some(intervals)) = (
                apply,
                self.classes.items.get_mut(id.index()),
                o.patch.offer_intervals,
            ) {
                class.offer_intervals = intervals;
            }
        }
        Ok(())
    }

    /// Why an effect list's `spells` or `spell_mastery` names a creature ability, if it does: a
    /// hero has and casts its spells, never an ability (HER-SPL-001 to HER-SPL-003).
    pub(crate) fn spell_effect_error(&self, list: &[CompiledHeroEffect]) -> Option<&'static str> {
        let ability = |s: SpellId| {
            self.spells
                .get(s)
                .is_some_and(|spell| spell.category == SpellCategory::Ability)
        };
        list.iter()
            .any(|e| match *e {
                HeroEffect::Spells {
                    grant: SpellGrant::Spell(s),
                }
                | HeroEffect::SpellMastery { spell: s, .. } => ability(s),
                _ => false,
            })
            .then_some("spells and spell_mastery name hero spells, not creature abilities")
    }

    /// Validates the compiled content once all packs are loaded (creature overrides of later
    /// packs may change what a hero refers to).
    pub(crate) fn validate(&self, creatures: &[CreatureInfo]) -> Result<(), CompileError> {
        let flag = |id: CreatureTypeId, f: CreatureFlag| {
            creatures
                .get(id.index())
                .is_some_and(|c| c.flags.contains(f))
        };
        if self.schools.items.len() > MAX_SPELL_SCHOOLS {
            return Err(CompileError::TooManyEntries("spell school"));
        }
        if self.spells.items.len() > MAX_SPELLS {
            return Err(CompileError::TooManyEntries("spell"));
        }
        for s in &self.spells.items {
            let ability = s.category == SpellCategory::Ability;
            let lowest = if ability { 0 } else { 1 };
            if !(lowest..=MAX_SPELL_LEVEL).contains(&s.level) {
                return Err(invalid(
                    "spell",
                    &s.key,
                    "level must be 1..=5 (creature abilities: 0..=5)",
                ));
            }
            if ability != s.schools.is_empty() {
                return Err(invalid(
                    "spell",
                    &s.key,
                    "hero spells need a school, creature abilities have none",
                ));
            }
        }
        for skill in &self.skills.items {
            if let Some(reason) = skill.levels.iter().find_map(|l| {
                effect::list_error(l, false)
                    .or_else(|| effect::scroll_error(l))
                    .or_else(|| self.spell_effect_error(l))
            }) {
                return Err(invalid("secondary skill", &skill.key, reason));
            }
        }
        for c in &self.classes.items {
            if c.primary_gain
                .iter()
                .any(|row| row.iter().map(|&p| u16::from(p)).sum::<u16>() != 100)
            {
                return Err(invalid(
                    "hero class",
                    &c.key,
                    "each primary_gain row must sum to 100",
                ));
            }
            if c.offer_intervals.contains(&0) {
                return Err(invalid(
                    "hero class",
                    &c.key,
                    "offer_intervals must be at least 1",
                ));
            }
        }
        for h in &self.heroes.items {
            let reason = if h.skills.len() > MAX_HERO_SKILLS {
                Some("at most 8 secondary skills")
            } else if h
                .skills
                .iter()
                .any(|&(_, l)| !(1..=MAX_SKILL_LEVEL).contains(&l))
            {
                Some("skill levels must be 1..=3")
            } else if !h.spells.is_empty() && !h.spell_book {
                Some("starting spells need a spell book")
            } else if h
                .army
                .iter()
                .any(|s| s.count[0] == 0 || s.count[1] < s.count[0])
            {
                Some("starting stacks need 1 <= low <= high")
            } else if h
                .army
                .iter()
                .any(|s| flag(s.creature, CreatureFlag::WarMachine))
            {
                Some("war machines belong in war_machines, not in the army")
            } else if h
                .war_machines
                .iter()
                .any(|&m| !flag(m, CreatureFlag::WarMachine))
            {
                Some("war_machines must have flag war_machine")
            } else {
                effect::list_error(&h.specialty, true)
                    .or_else(|| effect::scroll_error(&h.specialty))
                    .or_else(|| self.spell_effect_error(&h.specialty))
            };
            if let Some(reason) = reason {
                return Err(invalid("hero type", &h.key, reason));
            }
        }
        self.validate_artifacts(creatures)
    }
}

impl CanonicalHash for HeroContent {
    /// Schools (key, name, protection rank (v18)); spells (key, name, category, level, schools,
    /// cost, power, effect, granted by name only (v39); v43: the battle fields left with the
    /// content split, R01); skills (key, name, the three effect lists); classes (key, name, faction,
    /// primary, gains, weights, offer intervals (v46)); hero types (key, name, class, skills, spell
    /// book, spells, army, war machines, specialty); then the artifact content (`hash_artifacts`);
    /// then the hero parameters as presence flag + fields (v11; v46: the compiled experience
    /// table as length + values, the highest level, Wisdom, the schools).
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_len(self.schools.items.len());
        for s in &self.schools.items {
            s.key.as_str().hash_into(h);
            s.name.as_str().hash_into(h);
            h.write_u8(s.protection_rank);
        }
        h.write_len(self.spells.items.len());
        for s in &self.spells.items {
            s.key.as_str().hash_into(h);
            s.name.as_str().hash_into(h);
            h.write_u8(s.category as u8);
            h.write_u8(s.level);
            s.schools.hash_into(h);
            for v in s.cost.iter().chain([&s.power]).chain(&s.effect) {
                h.write_u16(*v);
            }
            h.write_bool(s.granted_by_name_only);
        }
        h.write_len(self.skills.items.len());
        for s in &self.skills.items {
            s.key.as_str().hash_into(h);
            s.name.as_str().hash_into(h);
            for level in &s.levels {
                level.hash_into(h);
            }
        }
        h.write_len(self.classes.items.len());
        for c in &self.classes.items {
            c.key.as_str().hash_into(h);
            c.name.as_str().hash_into(h);
            c.faction.hash_into(h);
            h.write_bytes(&c.primary);
            h.write_bytes(c.primary_gain.as_flattened());
            h.write_len(c.skill_weights.len());
            for &(skill, w) in &c.skill_weights {
                skill.hash_into(h);
                h.write_u8(w);
            }
            h.write_bytes(&c.offer_intervals);
        }
        h.write_len(self.heroes.items.len());
        for t in &self.heroes.items {
            t.key.as_str().hash_into(h);
            t.name.as_str().hash_into(h);
            t.class.hash_into(h);
            h.write_len(t.skills.len());
            for &(skill, level) in &t.skills {
                skill.hash_into(h);
                h.write_u8(level);
            }
            h.write_bool(t.spell_book);
            t.spells.hash_into(h);
            h.write_len(t.army.len());
            for s in &t.army {
                s.creature.hash_into(h);
                h.write_u16(s.count[0]);
                h.write_u16(s.count[1]);
            }
            t.war_machines.hash_into(h);
            t.specialty.hash_into(h);
        }
        self.hash_artifacts(h);
        h.write_bool(self.params.is_some());
        if let Some(p) = &self.params {
            h.write_bytes(&p.primary_min);
            h.write_u8(p.primary_max);
            h.write_u16(p.backpack);
            h.write_u8(p.learn_level);
            h.write_len(p.experience.len());
            for &e in &p.experience {
                h.write_u32(e);
            }
            h.write_u8(p.max_level);
            p.wisdom.hash_into(h);
            p.schools.hash_into(h);
            h.write_u16(p.mana_ceiling);
            h.write_u16(p.base_mana_regain);
            h.write_u16(p.regain_specialist_bonus);
        }
    }
}

#[cfg(test)]
mod tests;
