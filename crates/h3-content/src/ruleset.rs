//! The compiled, immutable ruleset (ADR-0006): content packs are validated once and compiled into
//! dense tables addressed by handles; the hot path never parses or looks up strings.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use h3_kernel::{CanonicalHash, Handle, StateHasher};

use crate::ability::{self, Ability, AbilityDef, CompiledAbility};
use crate::error::CompileError;
use crate::flags::{CreatureFlag, CreatureFlags};
use crate::key::{ContentKey, is_valid_part};
use crate::pack::{BattleParamsDef, ContentPack, CreaturePatch};
use crate::quirks::{QuirkRegistry, Quirks, REGISTRY};

/// Engine limit for army slots per side (the original uses 7, a ruleset parameter). The battle
/// context sizes its fixed-capacity state from this.
pub const MAX_ARMY_SLOTS: usize = 16;

/// Engine limit for battlefield size: hexes are addressed by `u8`.
pub const MAX_FIELD_HEXES: usize = 255;

/// Version of the ruleset hash encoding. Bump when the encoding below changes.
pub const RULESET_HASH_VERSION: u32 = 3;

/// A compiled creature type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreatureType {
    /// Attack skill.
    pub attack: u16,
    /// Defense skill.
    pub defense: u16,
    /// Minimum damage per creature (≥ 1).
    pub damage_min: u16,
    /// Maximum damage per creature (≥ `damage_min`).
    pub damage_max: u16,
    /// Hit points per creature (≥ 1).
    pub hit_points: u32,
    /// Speed (≥ 1).
    pub speed: u8,
    /// Ranged shots per battle (0 for creatures without ammunition).
    pub shots: u16,
    /// Flags.
    pub flags: CreatureFlags,
}

/// Handle of a creature type in a [`Ruleset`].
pub type CreatureTypeId = Handle<CreatureType>;

/// Battlefield dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldParams {
    /// Columns including side columns.
    pub width: u8,
    /// Rows.
    pub height: u8,
    /// Non-standable columns on each edge.
    pub side_columns: u8,
}

/// Compiled battle parameters (Tier 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BattleParams {
    /// Battlefield dimensions.
    pub field: FieldParams,
    /// Army slots per side (≤ [`MAX_ARMY_SLOTS`]).
    pub army_slots: u8,
    /// Damage rolls sampled for large stacks.
    pub damage_roll_samples: u8,
    /// Damage bonus per point of attack over defense, percent.
    pub attack_bonus_per_point_percent: u16,
    /// Cap of that bonus, percent.
    pub attack_bonus_cap_percent: u16,
    /// Damage reduction per point of defense over attack, per mille.
    pub defense_reduction_per_point_permille: u16,
    /// Cap of that reduction, per mille (≤ 1000).
    pub defense_reduction_cap_permille: u16,
    /// Defense bonus when defending, percent.
    pub defend_bonus_percent: u16,
    /// Minimum defense bonus when defending.
    pub defend_bonus_min: u16,
    formation: [[u8; MAX_ARMY_SLOTS]; MAX_ARMY_SLOTS],
}

impl BattleParams {
    /// Starting row of the `index`-th stack (in slot order) of an army with `stacks` stacks.
    pub fn formation_row(&self, stacks: usize, index: usize) -> Option<u8> {
        if index >= stacks || stacks > usize::from(self.army_slots) {
            return None;
        }
        // `1 <= stacks <= army_slots <= MAX_ARMY_SLOTS` here, so the lookups cannot fail.
        self.formation
            .get(stacks - 1)
            .and_then(|rows| rows.get(index))
            .copied()
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
}

/// Handle of a faction in a [`Ruleset`].
pub type FactionId = Handle<FactionInfo>;

/// Compiled non-battle attributes of a creature type (not needed on the battle hot path).
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
    /// Spell casts per battle.
    pub casts: u8,
    /// Adventure-map stack size range `[low, high]`.
    pub adventure_count: [u16; 2],
}

/// The immutable compiled ruleset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ruleset {
    packs: Vec<(String, String)>,
    profile: ContentKey,
    quirks: Quirks,
    resources: Vec<ResourceInfo>,
    factions: Vec<FactionInfo>,
    creatures: Vec<CreatureType>,
    creature_info: Vec<CreatureInfo>,
    creature_abilities: Vec<Vec<CompiledAbility>>,
    creature_keys: Vec<ContentKey>,
    creature_index: BTreeMap<String, CreatureTypeId>,
    battle: BattleParams,
    hash: u64,
}

/// Lookup tables and the dependency closure of the pack being compiled.
struct Scope<'a> {
    pack: &'a str,
    closure: &'a BTreeSet<String>,
    resources: &'a BTreeMap<String, ResourceId>,
    factions: &'a BTreeMap<String, FactionId>,
}

impl Scope<'_> {
    /// Checks that `key` belongs to a pack in the closure and resolves it in `index`.
    fn resolve<T>(
        &self,
        key: &ContentKey,
        index: &BTreeMap<String, Handle<T>>,
        kind: &'static str,
    ) -> Result<Handle<T>, CompileError> {
        if !self.closure.contains(key.pack()) {
            return Err(CompileError::ReferenceOutsideDependencies {
                key: String::from(key.as_str()),
                pack: String::from(self.pack),
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
}

/// Resolves the creature references of an ability list against the creatures compiled so far,
/// restricted to the pack's dependency closure.
fn abilities(
    defs: &[AbilityDef],
    scope: &Scope<'_>,
    creatures: &BTreeMap<String, CreatureTypeId>,
) -> Result<Vec<CompiledAbility>, CompileError> {
    defs.iter()
        .map(|a| a.try_map(|key| scope.resolve(key, creatures, "creature")))
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

        let mut resources: Vec<ResourceInfo> = Vec::new();
        let mut resource_index = BTreeMap::new();
        let mut factions: Vec<FactionInfo> = Vec::new();
        let mut faction_index = BTreeMap::new();
        let mut creatures = Vec::new();
        let mut creature_info = Vec::new();
        let mut creature_abilities: Vec<Vec<CompiledAbility>> = Vec::new();
        let mut creature_keys: Vec<ContentKey> = Vec::new();
        let mut creature_index = BTreeMap::new();
        let mut battle_def: Option<&BattleParamsDef> = None;
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
            for def in &pack.factions {
                check_namespace(&def.key, id)?;
                let handle = FactionId::from_index(factions.len())
                    .ok_or(CompileError::TooManyEntries("faction"))?;
                insert_unique(&mut faction_index, &def.key, handle)?;
                factions.push(FactionInfo {
                    key: def.key.clone(),
                    name: def.name.clone(),
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
                let handle = CreatureTypeId::from_index(creatures.len())
                    .ok_or(CompileError::TooManyCreatures)?;
                insert_unique(&mut creature_index, &def.key, handle)?;
                creatures.push(CreatureType {
                    attack: def.attack,
                    defense: def.defense,
                    damage_min: def.damage_min,
                    damage_max: def.damage_max,
                    hit_points: def.hit_points,
                    speed: def.speed,
                    shots: def.info.shots,
                    flags: flags(&def.info.flags)?,
                });
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
                    casts: info.casts,
                    adventure_count: info.adventure_count,
                });
                creature_keys.push(def.key.clone());
            }
            // Abilities may refer to any creature of the closure, including later ones of this
            // pack, so they are resolved once the pack's creatures are registered.
            for def in &pack.creatures {
                creature_abilities.push(abilities(&def.info.abilities, &scope, &creature_index)?);
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
                // Handles come from `creature_index`, which only holds pushed positions, so the
                // lookup always succeeds.
                let i = handle.index();
                creatures
                    .get_mut(i)
                    .zip(creature_info.get_mut(i))
                    .zip(creature_abilities.get_mut(i))
                    .map(|((stats, info), abils)| {
                        let target = Target {
                            stats,
                            info,
                            abilities: abils,
                        };
                        apply_patch(&o.patch, target, &scope, &creature_index)
                    })
                    .transpose()?;
            }
            if let Some(b) = &pack.battle {
                if battle_def.is_some() {
                    return Err(CompileError::DuplicateBattleParams(id.clone()));
                }
                battle_def = Some(b);
            }
        }
        for (((stats, info), abils), key) in creatures
            .iter()
            .zip(&creature_info)
            .zip(&creature_abilities)
            .zip(&creature_keys)
        {
            validate_creature(key, stats, info, abils)?;
        }
        let battle = compile_battle(battle_def.ok_or(CompileError::MissingBattleParams)?)?;

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

        let mut ruleset = Self {
            packs: pack_ids,
            profile: selected.key.clone(),
            quirks,
            resources,
            factions,
            creatures,
            creature_info,
            creature_abilities,
            creature_keys,
            creature_index,
            battle,
            hash: 0,
        };
        ruleset.hash = h3_kernel::hash::hash_of(&ruleset);
        Ok(ruleset)
    }

    /// The creature type behind a handle, if it belongs to this ruleset.
    pub fn creature(&self, id: CreatureTypeId) -> Option<&CreatureType> {
        self.creatures.get(id.index())
    }

    /// Non-battle attributes of a creature type.
    pub fn creature_info(&self, id: CreatureTypeId) -> Option<&CreatureInfo> {
        self.creature_info.get(id.index())
    }

    /// The abilities of a creature type (empty for unknown handles), in pack order.
    pub fn abilities(&self, id: CreatureTypeId) -> &[CompiledAbility] {
        self.creature_abilities
            .get(id.index())
            .map_or(&[], Vec::as_slice)
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
        self.creatures.len()
    }

    /// Resource kinds, in load order (a [`ResourceId`] indexes this slice).
    pub fn resources(&self) -> &[ResourceInfo] {
        &self.resources
    }

    /// Factions, in load order (a [`FactionId`] indexes this slice).
    pub fn factions(&self) -> &[FactionInfo] {
        &self.factions
    }

    /// Battle parameters.
    pub fn battle(&self) -> &BattleParams {
        &self.battle
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

/// The compiled parts of a creature an override patches.
struct Target<'a> {
    stats: &'a mut CreatureType,
    info: &'a mut CreatureInfo,
    abilities: &'a mut Vec<CompiledAbility>,
}

fn apply_patch(
    p: &CreaturePatch,
    target: Target<'_>,
    scope: &Scope<'_>,
    creatures: &BTreeMap<String, CreatureTypeId>,
) -> Result<(), CompileError> {
    let Target {
        stats,
        info,
        abilities: abils,
    } = target;
    fn set<T: Clone>(field: &mut T, value: Option<&T>) {
        if let Some(v) = value {
            *field = v.clone();
        }
    }
    set(&mut stats.attack, p.attack.as_ref());
    set(&mut stats.defense, p.defense.as_ref());
    set(&mut stats.damage_min, p.damage_min.as_ref());
    set(&mut stats.damage_max, p.damage_max.as_ref());
    set(&mut stats.hit_points, p.hit_points.as_ref());
    set(&mut stats.speed, p.speed.as_ref());
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
    set(&mut stats.shots, p.shots.as_ref());
    set(&mut info.casts, p.casts.as_ref());
    set(&mut info.adventure_count, p.adventure_count.as_ref());
    if let Some(names) = &p.flags {
        stats.flags = flags(names)?;
    }
    if let Some(defs) = &p.abilities {
        *abils = abilities(defs, scope, creatures)?;
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

fn validate_creature(
    key: &ContentKey,
    def: &CreatureType,
    info: &CreatureInfo,
    abils: &[CompiledAbility],
) -> Result<(), CompileError> {
    let invalid = |reason| {
        Err(CompileError::InvalidCreature {
            key: String::from(key.as_str()),
            reason,
        })
    };
    if def.damage_min == 0 {
        return invalid("damage_min must be at least 1");
    }
    if def.damage_max < def.damage_min {
        return invalid("damage_max must not be below damage_min");
    }
    if def.hit_points == 0 {
        return invalid("hit_points must be at least 1");
    }
    if def.speed == 0 {
        return invalid("speed must be at least 1");
    }
    if info.level == 0 {
        return invalid("level must be at least 1");
    }
    if info.adventure_count[1] < info.adventure_count[0] {
        return invalid("adventure_count high must not be below low");
    }
    if let Some(reason) = ability::list_error(abils) {
        return invalid(reason);
    }
    // The executable's multi-head and area-attack bits need the pattern the original hard-codes.
    let pattern = abils
        .iter()
        .any(|a| matches!(a, Ability::AttacksAllAdjacent | Ability::ThreeHeadedAttack));
    if def.flags.contains(CreatureFlag::MultiHeaded) != pattern {
        return invalid("flag multi_headed requires attacks_all_adjacent or three_headed_attack");
    }
    let area = abils.iter().any(|a| matches!(a, Ability::AreaShot { .. }));
    if def.flags.contains(CreatureFlag::AreaAttack) != area {
        return invalid("flag area_attack requires an area_shot ability (and vice versa)");
    }
    Ok(())
}

fn compile_battle(def: &BattleParamsDef) -> Result<BattleParams, CompileError> {
    let invalid = |reason| Err(CompileError::InvalidBattleParams(reason));
    let w = usize::from(def.field_width);
    let h = usize::from(def.field_height);
    let side = usize::from(def.side_columns);
    if h == 0 || w < 2 * side + 2 {
        return invalid("the field needs at least one row and two standable columns");
    }
    if w * h > MAX_FIELD_HEXES {
        return invalid("the field has more than 255 hexes");
    }
    let slots = usize::from(def.army_slots);
    if slots == 0 || slots > MAX_ARMY_SLOTS || slots > h {
        return invalid("army_slots must be between 1 and min(16, field_height)");
    }
    if def.damage_roll_samples == 0 {
        return invalid("damage_roll_samples must be at least 1");
    }
    if def.defense_reduction_cap_permille > 1000 {
        return invalid("defense_reduction_cap_permille must not exceed 1000");
    }
    if def.formation.len() != slots {
        return invalid("formation needs one entry per army size 1..=army_slots");
    }
    let mut formation = [[0u8; MAX_ARMY_SLOTS]; MAX_ARMY_SLOTS];
    for (n, (rows, out)) in def.formation.iter().zip(formation.iter_mut()).enumerate() {
        if rows.len() != n + 1 {
            return invalid("formation entry n must list n rows");
        }
        for (i, (&row, slot)) in rows.iter().zip(out.iter_mut()).enumerate() {
            if usize::from(row) >= h {
                return invalid("formation row outside the field");
            }
            if rows.get(..i).is_some_and(|earlier| earlier.contains(&row)) {
                return invalid("formation rows must be distinct");
            }
            *slot = row;
        }
    }
    Ok(BattleParams {
        field: FieldParams {
            width: def.field_width,
            height: def.field_height,
            side_columns: def.side_columns,
        },
        army_slots: def.army_slots,
        damage_roll_samples: def.damage_roll_samples,
        attack_bonus_per_point_percent: def.attack_bonus_per_point_percent,
        attack_bonus_cap_percent: def.attack_bonus_cap_percent,
        defense_reduction_per_point_permille: def.defense_reduction_per_point_permille,
        defense_reduction_cap_permille: def.defense_reduction_cap_permille,
        defend_bonus_percent: def.defend_bonus_percent,
        defend_bonus_min: def.defend_bonus_min,
        formation,
    })
}

impl CanonicalHash for Ruleset {
    /// Encoding v3: version, packs (id, version) in load order, profile key, quirk bits,
    /// resources and factions (key, name), creatures (key, stats in field order including shots
    /// and flag bits, then the info fields in field order — a faction as presence flag + handle,
    /// the cost as length + pairs — then the abilities as length + each ability's encoding),
    /// battle parameters in field order with the full formation table.
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
        h.write_len(self.factions.len());
        for f in &self.factions {
            f.key.as_str().hash_into(h);
            f.name.as_str().hash_into(h);
        }
        h.write_len(self.creatures.len());
        for (((c, info), abils), key) in self
            .creatures
            .iter()
            .zip(&self.creature_info)
            .zip(&self.creature_abilities)
            .zip(&self.creature_keys)
        {
            key.as_str().hash_into(h);
            h.write_u16(c.attack);
            h.write_u16(c.defense);
            h.write_u16(c.damage_min);
            h.write_u16(c.damage_max);
            h.write_u32(c.hit_points);
            h.write_u8(c.speed);
            h.write_u16(c.shots);
            h.write_u32(c.flags.bits());
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
            h.write_u8(info.casts);
            h.write_u16(info.adventure_count[0]);
            h.write_u16(info.adventure_count[1]);
            h.write_len(abils.len());
            for a in abils {
                a.hash_into(h);
            }
        }
        let b = &self.battle;
        h.write_u8(b.field.width);
        h.write_u8(b.field.height);
        h.write_u8(b.field.side_columns);
        h.write_u8(b.army_slots);
        h.write_u8(b.damage_roll_samples);
        h.write_u16(b.attack_bonus_per_point_percent);
        h.write_u16(b.attack_bonus_cap_percent);
        h.write_u16(b.defense_reduction_per_point_permille);
        h.write_u16(b.defense_reduction_cap_permille);
        h.write_u16(b.defend_bonus_percent);
        h.write_u16(b.defend_bonus_min);
        for row in &b.formation {
            h.write_bytes(row);
        }
    }
}

#[cfg(test)]
mod tests;
