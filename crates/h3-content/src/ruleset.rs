//! The compiled, immutable ruleset (ADR-0006): content packs are validated once and compiled into
//! dense tables addressed by handles; the hot path never parses or looks up strings.
//!
//! This is the core every context reads (ADR-0015 point 3); the battle's part (creatures' battle
//! rows, the spells' battle magic, obstacles, battle parameters) is `h3-content-battle`'s
//! `BattleRuleset`, compiled on it (R01).

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use h3_kernel::{CanonicalHash, Handle, StateHasher};

use crate::ability::{self, CompiledCreatureAbility, CreatureAbilityDef};
use crate::artifacts::{ArtifactSlot, ArtifactSlotId, ArtifactType, ArtifactTypeId};
use crate::error::CompileError;
use crate::flags::{CreatureFlag, CreatureFlags};
use crate::heroes::{
    HeroClass, HeroClassId, HeroContent, HeroParams, HeroType, HeroTypeId, Refs, SecondarySkill,
    SkillId, Spell, SpellId, SpellSchool,
};
use crate::key::{ContentKey, is_valid_part};
use crate::pack::{ContentPack, CreaturePatch};
use crate::quirks::{DATA_QUIRK_PREFIX, QuirkRegistry, Quirks, REGISTRY};

/// Engine limit for army slots per side (the original uses 7, a ruleset parameter). The battle
/// context sizes its fixed-capacity state from this.
pub const MAX_ARMY_SLOTS: usize = 16;

/// Version of the ruleset hash encoding. Bump when the encoding below changes.
pub const RULESET_HASH_VERSION: u32 = 51;

/// How an army is arranged at the start of a battle (BAT-SET-001): chosen per army in the
/// original (hero or garrison setting) and passed in the battle setup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Formation {
    /// Stacks spread from top to bottom.
    #[default]
    Loose,
    /// Stacks clustered around the middle rows.
    Tight,
}

impl Formation {
    /// Number of formations.
    pub const COUNT: usize = 2;

    /// 0 for loose, 1 for tight.
    pub const fn index(self) -> usize {
        match self {
            Self::Loose => 0,
            Self::Tight => 1,
        }
    }
}

/// A compiled resource kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceInfo {
    /// Key.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
}

/// Handle of a resource kind in a [`Ruleset`].
pub type ResourceId = Handle<ResourceInfo>;

/// A compiled faction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactionInfo {
    /// Key.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
    /// Native terrain of its creatures (BAT-TER-001), if any.
    pub native_terrain: Option<TerrainId>,
}

/// Handle of a faction in a [`Ruleset`].
pub type FactionId = Handle<FactionInfo>;

/// A compiled terrain type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainInfo {
    /// Key.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
}

/// Handle of a terrain type in a [`Ruleset`].
pub type TerrainId = Handle<TerrainInfo>;

/// Most direct upgrades of one creature type.
pub const MAX_UPGRADES: usize = 3;

/// A compiled creature type's core row: its identity and non-battle data (the battle's row is
/// `h3-content-battle`'s `CreatureType`, R01).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureInfo {
    /// Display name, singular.
    pub name: String,
    /// Display name, plural.
    pub plural: String,
    /// Faction, `None` for neutral creatures.
    pub faction: Option<FactionId>,
    /// Level (≥ 1).
    pub level: u8,
    /// Cost per creature, in resource order of the pack (no resource twice).
    pub cost: Vec<(ResourceId, u32)>,
    /// Weekly growth.
    pub growth: u16,
    /// Extra weekly growth from the horde building.
    pub horde_growth: u16,
    /// Fight value.
    pub fight_value: u32,
    /// AI value.
    pub ai_value: u32,
    /// Speed (≥ 1, 0 for war machines): hexes per battle turn, also read on the adventure map.
    pub speed: u8,
    /// Flags.
    pub flags: CreatureFlags,
    /// Adventure-map stack size range `[low, high]`.
    pub adventure_count: [u16; 2],
    /// The creature types it upgrades into, in pack order (E04a T3).
    pub upgrades: Vec<CreatureTypeId>,
    /// Its abilities outside battle, in pack order (pack format v50, E08 increment 9): read by
    /// the game's day, never by the battle.
    pub abilities: Vec<CompiledCreatureAbility>,
}

/// Handle of a creature type in a [`Ruleset`] (the battle's part indexes its rows by it too).
pub type CreatureTypeId = Handle<CreatureInfo>;

/// The immutable compiled ruleset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ruleset {
    packs: Vec<(String, String)>,
    profile: ContentKey,
    quirks: Quirks,
    resources: Vec<ResourceInfo>,
    factions: Vec<FactionInfo>,
    terrains: Vec<TerrainInfo>,
    terrain_index: BTreeMap<String, TerrainId>,
    creature_info: Vec<CreatureInfo>,
    creature_keys: Vec<ContentKey>,
    creature_index: BTreeMap<String, CreatureTypeId>,
    hero: HeroContent,
    hash: u64,
}

/// Lookup tables and the dependency closure of the pack being compiled.
struct Scope<'a> {
    pack: &'a str,
    closure: &'a BTreeSet<String>,
    resources: &'a BTreeMap<String, ResourceId>,
    factions: &'a BTreeMap<String, FactionId>,
}

/// Checks that `key` belongs to a pack in `closure` (the closure of `pack`) and resolves it in
/// `index`.
pub(crate) fn resolve_in<T>(
    pack: &str,
    closure: &BTreeSet<String>,
    key: &ContentKey,
    index: &BTreeMap<String, Handle<T>>,
    kind: &'static str,
) -> Result<Handle<T>, CompileError> {
    if !closure.contains(key.pack()) {
        return Err(CompileError::ReferenceOutsideDependencies {
            key: String::from(key.as_str()),
            pack: String::from(pack),
        });
    }
    index
        .get(key.as_str())
        .copied()
        .ok_or_else(|| CompileError::UnknownReference {
            kind,
            key: String::from(key.as_str()),
        })
}

impl Scope<'_> {
    /// Checks that `key` belongs to a pack in the closure and resolves it in `index`.
    fn resolve<T>(
        &self,
        key: &ContentKey,
        index: &BTreeMap<String, Handle<T>>,
        kind: &'static str,
    ) -> Result<Handle<T>, CompileError> {
        resolve_in(self.pack, self.closure, key, index, kind)
    }

    fn faction(&self, key: Option<&ContentKey>) -> Result<Option<FactionId>, CompileError> {
        key.map(|k| self.resolve(k, self.factions, "faction"))
            .transpose()
    }

    fn cost(&self, cost: &[(ContentKey, u32)]) -> Result<Vec<(ResourceId, u32)>, CompileError> {
        let mut out: Vec<(ResourceId, u32)> = Vec::with_capacity(cost.len());
        for (key, amount) in cost {
            let id = self.resolve(key, self.resources, "resource")?;
            if out.iter().any(|(r, _)| *r == id) {
                return Err(CompileError::DuplicateKey(String::from(key.as_str())));
            }
            out.push((id, *amount));
        }
        Ok(out)
    }

    /// Resolves the resources of a creature's abilities outside battle.
    fn abilities(
        &self,
        list: &[CreatureAbilityDef],
    ) -> Result<Vec<CompiledCreatureAbility>, CompileError> {
        list.iter()
            .map(|a| a.try_map(|r| self.resolve(r, self.resources, "resource")))
            .collect()
    }
}

/// Whether an override applies with the profile's `quirks`: an ungated one (`fixes` `None`)
/// always, a gated one only when the profile turns its flag off. A gated override must name a
/// registered data quirk flag (ADR-0007, amendment E04b T8e). The ruleset parts gate the halves
/// of their overrides with it too (R01).
pub fn override_applies(
    fixes: Option<&str>,
    registry: &QuirkRegistry,
    quirks: Quirks,
) -> Result<bool, CompileError> {
    fixes.map_or(Ok(true), |name| {
        registry
            .lookup(name)
            .filter(|_| name.starts_with(DATA_QUIRK_PREFIX))
            .map(|flag| !quirks.is_enabled(flag))
            .ok_or_else(|| CompileError::NotADataQuirk(String::from(name)))
    })
}

/// Resolves an upgrade list against the creatures compiled so far.
fn upgrades(
    keys: &[ContentKey],
    scope: &Scope<'_>,
    creatures: &BTreeMap<String, CreatureTypeId>,
) -> Result<Vec<CreatureTypeId>, CompileError> {
    keys.iter()
        .map(|k| scope.resolve(k, creatures, "creature"))
        .collect()
}

/// Resolves creature flag names.
fn flags(names: &[String]) -> Result<CreatureFlags, CompileError> {
    names
        .iter()
        .try_fold(CreatureFlags::default(), |set, name| {
            CreatureFlag::from_name(name)
                .map(|f| set.with(f))
                .ok_or_else(|| CompileError::UnknownFlag(name.clone()))
        })
}

impl Ruleset {
    /// Compiles the enabled `packs` (in any order) with the profile `profile` (a `pack:name`
    /// key), using the engine's quirk registry. Packs are loaded in dependency order (ADR-0006,
    /// E02 amendment).
    pub fn compile(packs: &[ContentPack], profile: &str) -> Result<Self, CompileError> {
        Self::compile_with(packs, profile, &REGISTRY)
    }

    /// Like [`Ruleset::compile`] with an explicit quirk registry (used by tests).
    pub fn compile_with(
        packs: &[ContentPack],
        profile: &str,
        registry: &QuirkRegistry,
    ) -> Result<Self, CompileError> {
        let (order, closures) = load_order(packs)?;
        let pack_ids: Vec<(String, String)> = order
            .iter()
            .map(|p| (p.manifest.id.clone(), p.manifest.version.clone()))
            .collect();

        // The profile comes first: gated overrides apply by its data quirk flags (ADR-0007,
        // amendment E04b T8e).
        let mut profiles = BTreeMap::new();
        for pack in &order {
            for p in &pack.profiles {
                check_namespace(&p.key, &pack.manifest.id)?;
                if profiles.insert(p.key.as_str(), p).is_some() {
                    return Err(CompileError::DuplicateKey(String::from(p.key.as_str())));
                }
            }
        }
        let selected = profiles
            .get(profile)
            .ok_or_else(|| CompileError::UnknownProfile(String::from(profile)))?;
        let mut quirks = Quirks::default();
        for (flag, on) in &selected.quirks {
            let f = registry
                .lookup(flag)
                .ok_or_else(|| CompileError::UnknownQuirk {
                    profile: String::from(profile),
                    flag: flag.clone(),
                })?;
            quirks = quirks.with(f, *on);
        }

        let mut resources: Vec<ResourceInfo> = Vec::new();
        let mut resource_index = BTreeMap::new();
        let mut factions: Vec<FactionInfo> = Vec::new();
        let mut faction_index = BTreeMap::new();
        let mut terrains: Vec<TerrainInfo> = Vec::new();
        let mut terrain_index = BTreeMap::new();
        let mut creature_info = Vec::new();
        let mut creature_keys: Vec<ContentKey> = Vec::new();
        let mut creature_index = BTreeMap::new();
        let mut hero = HeroContent::default();
        for (pack, closure) in order.iter().zip(&closures) {
            let id = &pack.manifest.id;
            for def in &pack.resources {
                check_namespace(&def.key, id)?;
                let handle = ResourceId::from_index(resources.len())
                    .ok_or(CompileError::TooManyEntries("resource"))?;
                insert_unique(&mut resource_index, &def.key, handle)?;
                resources.push(ResourceInfo {
                    key: def.key.clone(),
                    name: def.name.clone(),
                });
            }
            for def in &pack.terrains {
                check_namespace(&def.key, id)?;
                let handle = TerrainId::from_index(terrains.len())
                    .ok_or(CompileError::TooManyEntries("terrain"))?;
                insert_unique(&mut terrain_index, &def.key, handle)?;
                terrains.push(TerrainInfo {
                    key: def.key.clone(),
                    name: def.name.clone(),
                });
            }
            for def in &pack.factions {
                check_namespace(&def.key, id)?;
                let handle = FactionId::from_index(factions.len())
                    .ok_or(CompileError::TooManyEntries("faction"))?;
                insert_unique(&mut faction_index, &def.key, handle)?;
                let native_terrain = def
                    .native_terrain
                    .as_ref()
                    .map(|key| resolve_in(id, closure, key, &terrain_index, "terrain"))
                    .transpose()?;
                factions.push(FactionInfo {
                    key: def.key.clone(),
                    name: def.name.clone(),
                    native_terrain,
                });
            }
            let scope = Scope {
                pack: id,
                closure,
                resources: &resource_index,
                factions: &faction_index,
            };
            for def in &pack.creatures {
                check_namespace(&def.key, id)?;
                let handle = CreatureTypeId::from_index(creature_info.len())
                    .ok_or(CompileError::TooManyCreatures)?;
                insert_unique(&mut creature_index, &def.key, handle)?;
                let info = &def.info;
                creature_info.push(CreatureInfo {
                    name: info.name.clone(),
                    plural: info.plural.clone(),
                    faction: scope.faction(info.faction.as_ref())?,
                    level: info.level,
                    cost: scope.cost(&info.cost)?,
                    growth: info.growth,
                    horde_growth: info.horde_growth,
                    fight_value: info.fight_value,
                    ai_value: info.ai_value,
                    speed: def.speed,
                    flags: flags(&info.flags)?,
                    adventure_count: info.adventure_count,
                    upgrades: Vec::new(),
                    abilities: scope.abilities(&info.abilities)?,
                });
                creature_keys.push(def.key.clone());
            }
            let refs = Refs {
                pack: id,
                closure,
                resources: &resource_index,
                factions: &faction_index,
                creatures: &creature_index,
            };
            hero.add_pack(pack, &refs)?;
            hero.add_overrides(pack, &refs, |fixes| {
                override_applies(fixes, registry, quirks)
            })?;
            // Upgrades may refer to any creature of the closure, including later ones of this
            // pack, so they are resolved once the pack's creatures are registered.
            let first = creature_info.len() - pack.creatures.len();
            for (def, info) in pack
                .creatures
                .iter()
                .zip(creature_info.get_mut(first..).unwrap_or_default())
            {
                info.upgrades = upgrades(&def.info.upgrades, &scope, &creature_index)?;
            }
            for o in &pack.creature_overrides {
                let target = String::from(o.target.as_str());
                let handle = creature_index.get(&target).copied().ok_or_else(|| {
                    CompileError::UnknownReference {
                        kind: "creature",
                        key: target.clone(),
                    }
                })?;
                if !closure.contains(o.target.pack()) {
                    return Err(CompileError::ReferenceOutsideDependencies {
                        key: target,
                        pack: id.clone(),
                    });
                }
                let apply = override_applies(o.fixes.as_deref(), registry, quirks)?;
                // Handles come from `creature_index`, which only holds pushed positions, so the
                // lookup always succeeds. The patch goes onto a copy: a gated override's references
                // are resolved in every profile, and it is kept (and then validated with the
                // creature) only where it applies.
                creature_info
                    .get_mut(handle.index())
                    .map(|info| {
                        let mut patched = info.clone();
                        apply_patch(&o.patch, &mut patched, &scope, &creature_index)?;
                        if apply {
                            *info = patched;
                        }
                        Ok(())
                    })
                    .transpose()?;
            }
        }
        for (i, (info, key)) in creature_info.iter().zip(&creature_keys).enumerate() {
            validate_creature(i, key, info)?;
        }
        hero.validate(&creature_info)?;
        let mut ruleset = Self {
            packs: pack_ids,
            profile: selected.key.clone(),
            quirks,
            resources,
            factions,
            terrains,
            terrain_index,
            creature_info,
            creature_keys,
            creature_index,
            hero,
            hash: 0,
        };
        ruleset.hash = h3_kernel::hash::hash_of(&ruleset);
        Ok(ruleset)
    }

    /// The core row of the creature type behind a handle, if it belongs to this ruleset (the
    /// battle's part shadows it with its battle row, R01).
    pub fn creature(&self, id: CreatureTypeId) -> Option<&CreatureInfo> {
        self.creature_info.get(id.index())
    }

    /// The core row of a creature type (the same as [`Ruleset::creature`], kept for its readers).
    pub fn creature_info(&self, id: CreatureTypeId) -> Option<&CreatureInfo> {
        self.creature_info.get(id.index())
    }

    /// The key of a creature type.
    pub fn creature_key(&self, id: CreatureTypeId) -> Option<&ContentKey> {
        self.creature_keys.get(id.index())
    }

    /// Looks a creature type up by its `pack:name` key (not for the hot path).
    pub fn creature_by_key(&self, key: &str) -> Option<CreatureTypeId> {
        self.creature_index.get(key).copied()
    }

    /// Number of creature types.
    pub fn creature_count(&self) -> usize {
        self.creature_info.len()
    }

    /// The direct upgrades of a creature type (empty for unknown handles).
    pub fn upgrades(&self, id: CreatureTypeId) -> &[CreatureTypeId] {
        self.creature_info
            .get(id.index())
            .map_or(&[], |i| i.upgrades.as_slice())
    }

    /// Spell schools, in load order (a [`crate::SchoolId`] indexes this slice).
    pub fn spell_schools(&self) -> &[SpellSchool] {
        &self.hero.schools.items
    }

    /// Spells, in load order (a [`SpellId`] indexes this slice).
    pub fn spells(&self) -> &[Spell] {
        &self.hero.spells.items
    }

    /// The spell behind a handle.
    pub fn spell(&self, id: SpellId) -> Option<&Spell> {
        self.hero.spells.get(id)
    }

    /// Looks a spell up by its `pack:name` key.
    pub fn spell_by_key(&self, key: &str) -> Option<SpellId> {
        self.hero.spells.by_key(key)
    }

    /// Secondary skills, in load order (a [`SkillId`] indexes this slice).
    pub fn skills(&self) -> &[SecondarySkill] {
        &self.hero.skills.items
    }

    /// The secondary skill behind a handle.
    pub fn skill(&self, id: SkillId) -> Option<&SecondarySkill> {
        self.hero.skills.get(id)
    }

    /// Looks a secondary skill up by its `pack:name` key.
    pub fn skill_by_key(&self, key: &str) -> Option<SkillId> {
        self.hero.skills.by_key(key)
    }

    /// Hero classes, in load order (a [`HeroClassId`] indexes this slice).
    pub fn hero_classes(&self) -> &[HeroClass] {
        &self.hero.classes.items
    }

    /// The hero class behind a handle.
    pub fn hero_class(&self, id: HeroClassId) -> Option<&HeroClass> {
        self.hero.classes.get(id)
    }

    /// Hero types, in load order (a [`HeroTypeId`] indexes this slice).
    pub fn hero_types(&self) -> &[HeroType] {
        &self.hero.heroes.items
    }

    /// The hero type behind a handle.
    pub fn hero_type(&self, id: HeroTypeId) -> Option<&HeroType> {
        self.hero.heroes.get(id)
    }

    /// Looks a hero type up by its `pack:name` key.
    pub fn hero_type_by_key(&self, key: &str) -> Option<HeroTypeId> {
        self.hero.heroes.by_key(key)
    }

    /// All artifact slots, in load order (handle order).
    pub fn artifact_slots(&self) -> &[ArtifactSlot] {
        &self.hero.slots.items
    }

    /// The artifact slot behind a handle.
    pub fn artifact_slot(&self, id: ArtifactSlotId) -> Option<&ArtifactSlot> {
        self.hero.slots.get(id)
    }

    /// Looks an artifact slot up by its `pack:name` key.
    pub fn artifact_slot_by_key(&self, key: &str) -> Option<ArtifactSlotId> {
        self.hero.slots.by_key(key)
    }

    /// All artifact types, in load order (handle order).
    pub fn artifacts(&self) -> &[ArtifactType] {
        &self.hero.artifacts.items
    }

    /// The artifact type behind a handle.
    pub fn artifact(&self, id: ArtifactTypeId) -> Option<&ArtifactType> {
        self.hero.artifacts.get(id)
    }

    /// Looks an artifact type up by its `pack:name` key.
    pub fn artifact_by_key(&self, key: &str) -> Option<ArtifactTypeId> {
        self.hero.artifacts.by_key(key)
    }

    /// The hero parameters, if a pack provides them (E04a T4a).
    pub fn hero_params(&self) -> Option<&HeroParams> {
        self.hero.params.as_ref()
    }

    /// The combination artifact `part` is a part of, if any.
    pub fn combination_of(&self, part: ArtifactTypeId) -> Option<ArtifactTypeId> {
        self.hero.combination_of(part)
    }

    /// Resource kinds, in load order (a [`ResourceId`] indexes this slice).
    pub fn resources(&self) -> &[ResourceInfo] {
        &self.resources
    }

    /// Factions, in load order (a [`FactionId`] indexes this slice).
    pub fn factions(&self) -> &[FactionInfo] {
        &self.factions
    }

    /// Looks a faction up by its `pack:name` key (not for the hot path).
    pub fn faction_by_key(&self, key: &str) -> Option<FactionId> {
        self.factions
            .iter()
            .position(|f| f.key.as_str() == key)
            .and_then(FactionId::from_index)
    }

    /// Terrain types, in load order (a [`TerrainId`] indexes this slice).
    pub fn terrains(&self) -> &[TerrainInfo] {
        &self.terrains
    }

    /// Looks a terrain type up by its `pack:name` key.
    pub fn terrain_by_key(&self, key: &str) -> Option<TerrainId> {
        self.terrain_index.get(key).copied()
    }

    /// Enabled quirk flags.
    pub fn quirks(&self) -> Quirks {
        self.quirks
    }

    /// The selected profile.
    pub fn profile(&self) -> &ContentKey {
        &self.profile
    }

    /// Loaded packs as `(id, version)`, in load order.
    pub fn packs(&self) -> &[(String, String)] {
        &self.packs
    }

    /// Canonical hash of the compiled content (recorded in replays; a replay only verifies against
    /// the ruleset it was recorded with).
    pub fn content_hash(&self) -> u64 {
        self.hash
    }
}

/// Validates the pack set and returns the load order with each pack's dependency closure
/// (itself included). Order: dependencies first; among packs whose dependencies are loaded, the
/// smallest id first (a unique tie-break, so the order is deterministic).
fn load_order(
    packs: &[ContentPack],
) -> Result<(Vec<&ContentPack>, Vec<BTreeSet<String>>), CompileError> {
    let mut by_id: BTreeMap<&str, &ContentPack> = BTreeMap::new();
    for pack in packs {
        let id = pack.manifest.id.as_str();
        if !is_valid_part(id) {
            return Err(CompileError::InvalidPackId(String::from(id)));
        }
        if by_id.insert(id, pack).is_some() {
            return Err(CompileError::DuplicatePack(String::from(id)));
        }
    }
    for pack in packs {
        for dep in &pack.manifest.depends {
            if !by_id.contains_key(dep.as_str()) {
                return Err(CompileError::MissingDependency {
                    pack: pack.manifest.id.clone(),
                    depends: dep.clone(),
                });
            }
        }
    }
    let mut order: Vec<&ContentPack> = Vec::with_capacity(packs.len());
    let mut closures: Vec<BTreeSet<String>> = Vec::with_capacity(packs.len());
    let mut done: BTreeMap<&str, usize> = BTreeMap::new();
    while order.len() < by_id.len() {
        let next = by_id.iter().find(|(id, pack)| {
            !done.contains_key(*id)
                && pack
                    .manifest
                    .depends
                    .iter()
                    .all(|d| done.contains_key(d.as_str()))
        });
        let Some((&id, &pack)) = next else {
            let stuck = by_id.keys().find(|id| !done.contains_key(*id));
            return Err(CompileError::DependencyCycle(String::from(
                stuck.copied().unwrap_or_default(),
            )));
        };
        let mut closure = BTreeSet::new();
        closure.insert(String::from(id));
        // Every dependency is in `done` here (that is how `pack` was chosen).
        let dep_closures: Vec<&BTreeSet<String>> = pack
            .manifest
            .depends
            .iter()
            .filter_map(|d| done.get(d.as_str()).and_then(|&i| closures.get(i)))
            .collect();
        for dep_closure in dep_closures {
            closure.extend(dep_closure.iter().cloned());
        }
        done.insert(id, order.len());
        order.push(pack);
        closures.push(closure);
    }
    Ok((order, closures))
}

fn insert_unique<T>(
    index: &mut BTreeMap<String, Handle<T>>,
    key: &ContentKey,
    handle: Handle<T>,
) -> Result<(), CompileError> {
    if index.insert(String::from(key.as_str()), handle).is_some() {
        return Err(CompileError::DuplicateKey(String::from(key.as_str())));
    }
    Ok(())
}

fn apply_patch(
    p: &CreaturePatch,
    info: &mut CreatureInfo,
    scope: &Scope<'_>,
    creatures: &BTreeMap<String, CreatureTypeId>,
) -> Result<(), CompileError> {
    fn set<T: Clone>(field: &mut T, value: Option<&T>) {
        if let Some(v) = value {
            *field = v.clone();
        }
    }
    set(&mut info.speed, p.speed.as_ref());
    set(&mut info.name, p.name.as_ref());
    set(&mut info.plural, p.plural.as_ref());
    if let Some(faction) = &p.faction {
        info.faction = scope.faction(faction.as_ref())?;
    }
    set(&mut info.level, p.level.as_ref());
    if let Some(cost) = &p.cost {
        info.cost = scope.cost(cost)?;
    }
    set(&mut info.growth, p.growth.as_ref());
    set(&mut info.horde_growth, p.horde_growth.as_ref());
    set(&mut info.fight_value, p.fight_value.as_ref());
    set(&mut info.ai_value, p.ai_value.as_ref());
    set(&mut info.adventure_count, p.adventure_count.as_ref());
    if let Some(names) = &p.flags {
        info.flags = flags(names)?;
    }
    if let Some(keys) = &p.upgrades {
        info.upgrades = upgrades(keys, scope, creatures)?;
    }
    if let Some(list) = &p.abilities {
        info.abilities = scope.abilities(list)?;
    }
    Ok(())
}

fn check_namespace(key: &ContentKey, pack: &str) -> Result<(), CompileError> {
    if key.pack() == pack {
        Ok(())
    } else {
        Err(CompileError::KeyOutsidePack {
            key: String::from(key.as_str()),
            pack: String::from(pack),
        })
    }
}

/// Checks a creature's core row (its battle row is checked by the battle's part, R01).
fn validate_creature(
    index: usize,
    key: &ContentKey,
    info: &CreatureInfo,
) -> Result<(), CompileError> {
    let invalid = |reason| {
        Err(CompileError::InvalidCreature {
            key: String::from(key.as_str()),
            reason,
        })
    };
    // War machines never move, also when their side commands them (BAT-WAR-001, BAT-WAR-006).
    let machine = info.flags.contains(CreatureFlag::WarMachine);
    if (info.speed == 0) != machine {
        return invalid("speed must be 0 for war machines and at least 1 otherwise");
    }
    if info.level == 0 {
        return invalid("level must be at least 1");
    }
    if info.adventure_count[1] < info.adventure_count[0] {
        return invalid("adventure_count high must not be below low");
    }
    if info.upgrades.len() > MAX_UPGRADES {
        return invalid("at most 3 upgrades");
    }
    if info.upgrades.iter().any(|u| u.index() == index) {
        return invalid("a creature cannot upgrade into itself");
    }
    for (i, u) in info.upgrades.iter().enumerate() {
        if info.upgrades.get(..i).unwrap_or_default().contains(u) {
            return invalid("an upgrade is listed twice");
        }
    }
    if let Some(reason) = ability::list_error(&info.abilities) {
        return invalid(reason);
    }
    Ok(())
}

impl CanonicalHash for Ruleset {
    /// Encoding v43 (R01, the content split): version, packs (id, version) in load order,
    /// profile key, quirk bits, resources (key, name), terrains (key, name), factions (key, name,
    /// native terrain as an optional handle), creatures (key, name, plural, faction as presence
    /// flag + handle, level, cost as length + pairs, growth, horde growth, fight value, AI value,
    /// speed, flag bits, adventure count, upgrades as length + handles; since v51 the abilities
    /// outside battle as length + (code, fields)), then the hero content
    /// (`heroes::HeroContent`'s encoding: spells without their battle fields). What v42 encoded
    /// and v43 no longer does is the battle part's (`h3-content-battle`'s hash, v1): obstacles,
    /// the creatures' attack, defense, damage, hit points, shots, casts and abilities, the battle
    /// parameters, and the spells' polarity, battle magic, counters, dispels, spares, `tests_as`
    /// and `summons`. The versions before v43 are described in this comment's git history.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u32(RULESET_HASH_VERSION);
        h.write_len(self.packs.len());
        for (id, version) in &self.packs {
            id.as_str().hash_into(h);
            version.as_str().hash_into(h);
        }
        self.profile.as_str().hash_into(h);
        h.write_u64(self.quirks.bits());
        h.write_len(self.resources.len());
        for r in &self.resources {
            r.key.as_str().hash_into(h);
            r.name.as_str().hash_into(h);
        }
        h.write_len(self.terrains.len());
        for t in &self.terrains {
            t.key.as_str().hash_into(h);
            t.name.as_str().hash_into(h);
        }
        h.write_len(self.factions.len());
        for f in &self.factions {
            f.key.as_str().hash_into(h);
            f.name.as_str().hash_into(h);
            f.native_terrain.hash_into(h);
        }
        h.write_len(self.creature_info.len());
        for (info, key) in self.creature_info.iter().zip(&self.creature_keys) {
            key.as_str().hash_into(h);
            info.name.as_str().hash_into(h);
            info.plural.as_str().hash_into(h);
            h.write_bool(info.faction.is_some());
            h.write_u16(info.faction.map_or(0, Handle::raw));
            h.write_u8(info.level);
            h.write_len(info.cost.len());
            for (r, amount) in &info.cost {
                h.write_u16(r.raw());
                h.write_u32(*amount);
            }
            h.write_u16(info.growth);
            h.write_u16(info.horde_growth);
            h.write_u32(info.fight_value);
            h.write_u32(info.ai_value);
            h.write_u8(info.speed);
            h.write_u32(info.flags.bits());
            h.write_u16(info.adventure_count[0]);
            h.write_u16(info.adventure_count[1]);
            info.upgrades.hash_into(h);
            info.abilities.hash_into(h);
        }
        self.hero.hash_into(h);
    }
}

#[cfg(test)]
pub(crate) mod tests;
