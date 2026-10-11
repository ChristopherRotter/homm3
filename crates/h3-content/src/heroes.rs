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
    self, CompiledHeroEffect, HeroEffect, HeroEffectDef, MAX_SKILL_LEVEL, SpellGrant,
};
use crate::error::CompileError;
use crate::flags::CreatureFlag;
use crate::key::ContentKey;
use crate::pack::{ContentPack, HeroParamsDef};
use crate::ruleset::{CreatureType, CreatureTypeId, FactionId, ResourceId};
use crate::spell::SpellMagic;

/// Highest spell level.
pub const MAX_SPELL_LEVEL: u8 = 5;
/// Most spells in a ruleset (the battle keeps spell books as bitsets, E04b T2b).
pub const MAX_SPELLS: usize = 128;
/// Most spell schools (a hero keeps one mastery per school, its `school_mastery` effects).
pub const MAX_SPELL_SCHOOLS: usize = 4;
/// Most spells with a summon effect in a ruleset (the battle keeps a side's stack of each
/// creature its hero can summon from the battle's start, E04b T6b; four in the original).
pub const MAX_SUMMON_SPELLS: usize = 4;
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

/// Whether a spell is good or bad for its target (the executable's spell table).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpellPolarity {
    /// Helps its target.
    Positive,
    /// Neither (summons, battlefield spells, adventure spells).
    Neutral,
    /// Harms its target.
    Negative,
}

impl SpellPolarity {
    /// All values, in code order.
    pub const ALL: [Self; 3] = [Self::Positive, Self::Neutral, Self::Negative];

    /// Pack name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Positive => "positive",
            Self::Neutral => "neutral",
            Self::Negative => "negative",
        }
    }

    /// Looks a value up by its pack name.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.name() == name)
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

/// A compiled spell: its data row and what it does in battle (E04b).
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
    /// Good or bad for its target.
    pub polarity: SpellPolarity,
    /// Targeting and effects in battle.
    pub magic: SpellMagic,
    /// Spells whose effects a cast ends on each stack it affects (pack format v5; Bless and
    /// Curse, Haste and Slow).
    pub counters: Vec<SpellId>,
    /// The only spells whose effects its dispel effect ends (pack format v7; Cure's fourteen).
    pub dispels: Vec<SpellId>,
    /// Spells whose effects its dispel effect spares when `dispels` is empty (pack format v7;
    /// Dispel spares Poison).
    pub spares: Vec<SpellId>,
    /// The raise spell whose target test a sacrifice effect's target takes under quirk
    /// BAT-QUIRK-5 (pack format v17; Sacrifice: Resurrection, `installation.md` §7.18).
    pub tests_as: Option<SpellId>,
    /// The creature a summon effect summons (pack format v18; the Summon Elemental spells, their
    /// elementals, `installation.md` §7.19).
    pub summons: Option<CreatureTypeId>,
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
/// (`h3-heroes`) and of a battle hero's primary skills.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeroParams {
    /// Lowest effective primary skills (attack, defense, power, knowledge); a lower total
    /// counts as this value (HER-DET-001).
    pub primary_min: [u8; 4],
    /// Highest primary skill, base and effective (HER-DET-001).
    pub primary_max: u8,
    /// Most artifacts in a hero's backpack (HER-ART-002).
    pub backpack: u16,
}

/// Validates hero parameters: every minimum at most the maximum (HER-DET-001).
fn compile_params(def: &HeroParamsDef) -> Result<HeroParams, CompileError> {
    if def.primary_min.iter().any(|&m| m > def.primary_max) {
        return Err(CompileError::InvalidHeroParams(
            "primary_min must not exceed primary_max",
        ));
    }
    Ok(HeroParams {
        primary_min: def.primary_min,
        primary_max: def.primary_max,
        backpack: def.backpack,
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
                polarity: def.polarity,
                magic: def.magic.clone(),
                counters: Vec::new(),
                dispels: Vec::new(),
                spares: Vec::new(),
                tests_as: None,
                summons: def
                    .summons
                    .as_ref()
                    .map(|c| refs.resolve(c, refs.creatures, "creature"))
                    .transpose()?,
                granted_by_name_only: def.granted_by_name_only,
            });
        }
        // Counters and dispel lists may name a later spell of the same pack, so all are
        // registered first.
        let lists = pack
            .spells
            .iter()
            .map(|def| {
                let list = |keys| distinct(refs, keys, &self.spells.index, "spell");
                Ok((
                    (list(&def.counters)?, list(&def.dispels)?),
                    (list(&def.spares)?, list(def.tests_as.as_slice())?),
                ))
            })
            .collect::<Result<Vec<_>, CompileError>>()?;
        let first = self.spells.items.len() - pack.spells.len();
        for (s, ((counters, dispels), (spares, tests_as))) in
            self.spells.items.iter_mut().skip(first).zip(lists)
        {
            s.counters = counters;
            s.dispels = dispels;
            s.spares = spares;
            s.tests_as = tests_as.first().copied();
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
            self.params = Some(compile_params(def)?);
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

    /// Applies the pack's hero overrides to hero types of its closure (E04b T8e). A gated one is
    /// checked in every profile and applied only where `applies` says so (ADR-0007, amendment
    /// E04b T8e).
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
    pub(crate) fn validate(
        &self,
        creatures: &[CreatureType],
        army_slots: u8,
    ) -> Result<(), CompileError> {
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
            if let Some(reason) = s.magic.error(s.category != SpellCategory::Adventure) {
                return Err(invalid("spell", &s.key, reason));
            }
            let own = self.spells.index.get(s.key.as_str()).copied();
            if !s.counters.is_empty()
                && (!s.magic.is_timed() || own.is_some_and(|id| s.counters.contains(&id)))
            {
                return Err(invalid(
                    "spell",
                    &s.key,
                    "counters need a timed effect and may not name the spell itself",
                ));
            }
            let lists = (s.dispels.is_empty(), s.spares.is_empty());
            if lists != (true, true) && (!s.magic.dispels() || lists == (false, false)) {
                return Err(invalid(
                    "spell",
                    &s.key,
                    "dispels or spares (not both) need a dispel effect",
                ));
            }
            let raise_test = s
                .tests_as
                .and_then(|t| self.spells.items.get(t.index()))
                .is_none_or(|t| t.magic.raises());
            if s.tests_as.is_some() != s.magic.sacrifices() || !raise_test {
                return Err(invalid(
                    "spell",
                    &s.key,
                    "a sacrifice effect needs tests_as, a spell with a raise effect",
                ));
            }
            if s.summons.is_some() != s.magic.summons() {
                return Err(invalid(
                    "spell",
                    &s.key,
                    "a summon effect and the creature it summons go together",
                ));
            }
        }
        if self
            .spells
            .items
            .iter()
            .filter(|s| s.summons.is_some())
            .count()
            > MAX_SUMMON_SPELLS
        {
            return Err(CompileError::TooManyEntries("summon spell"));
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
            } else if h.army.len() > usize::from(army_slots) {
                Some("more starting stacks than army slots")
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
    /// Schools (key, name, protection rank (v18)); spells (key, name, category, level, schools, cost, power, effect,
    /// polarity, magic (v14), counters (v17), dispels and spares (v19), tests as (v26), summons (v27), granted by name only (v39)); skills (key, name, the three effect lists); classes (key, name, faction,
    /// primary, gains, weights); hero types (key, name, class, skills, spell book, spells,
    /// army, war machines, specialty); then the artifact content (`hash_artifacts`); then the
    /// hero parameters as presence flag + fields (v11).
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
            h.write_u8(s.polarity as u8);
            s.magic.hash_into(h);
            s.counters.hash_into(h);
            s.dispels.hash_into(h);
            s.spares.hash_into(h);
            s.tests_as.hash_into(h);
            s.summons.hash_into(h);
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
        }
    }
}

#[cfg(test)]
mod tests;
