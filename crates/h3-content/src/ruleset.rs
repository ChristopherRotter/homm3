//! The compiled, immutable ruleset (ADR-0006): content packs are validated once and compiled into
//! dense tables addressed by handles; the hot path never parses or looks up strings.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use h3_kernel::{CanonicalHash, Handle, StateHasher};

use crate::ability::{self, Ability, AbilityDef, CompiledAbility};
use crate::artifacts::{ArtifactSlot, ArtifactSlotId, ArtifactType, ArtifactTypeId};
use crate::effect::MAX_SKILL_LEVEL;
use crate::error::CompileError;
use crate::flags::{CreatureFlag, CreatureFlags};
use crate::heroes::{
    HeroClass, HeroClassId, HeroContent, HeroParams, HeroType, HeroTypeId, Refs, SchoolId,
    SecondarySkill, SkillId, Spell, SpellId, SpellSchool,
};
use crate::key::{ContentKey, is_valid_part};
use crate::pack::{BattleParamsDef, ContentPack, CreaturePatch};
use crate::quirks::{QuirkRegistry, Quirks, REGISTRY};
use crate::spell::{SpellEffect, SpellTarget, TimedChange};

/// Engine limit for army slots per side (the original uses 7, a ruleset parameter). The battle
/// context sizes its fixed-capacity state from this.
pub const MAX_ARMY_SLOTS: usize = 16;

/// Engine limit for the stacks of one side in a battle, those created in it included (the
/// original keeps 21 records per side, `docs/formats/installation.md` §7.7). The battle sizes its
/// fixed-capacity state from this; `BattleParams::side_stack_limit` stays below it.
pub const MAX_SIDE_STACKS: usize = 21;

/// Engine limit for battlefield size: hexes are addressed by `u8`.
pub const MAX_FIELD_HEXES: usize = 255;

/// Version of the ruleset hash encoding. Bump when the encoding below changes.
pub const RULESET_HASH_VERSION: u32 = 42;

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

/// Most distinct morale/luck aura values (kind, amount, affected side) in a ruleset; sizes the
/// battle's modifier pool (ADR-0012 §3).
pub const MAX_AURA_VALUES: usize = 8;

/// Most levels of a morale or luck roll table.
pub const MAX_ROLL_LEVELS: usize = 8;

/// Chances of a morale or luck roll by level: level `l` (1-based) triggers with
/// `chance(l)` out of the die's sides (Tier 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RollTable {
    len: u8,
    chances: [u8; MAX_ROLL_LEVELS],
}

impl RollTable {
    /// The highest level (the number of entries).
    pub const fn levels(&self) -> u8 {
        self.len
    }

    /// The chance of level `level` (1-based, capped at the highest level), or 0 for level 0.
    pub fn chance(&self, level: u8) -> u8 {
        let i = usize::from(level.min(self.len));
        // `i <= len <= MAX_ROLL_LEVELS`; level 0 has no entry.
        i.checked_sub(1)
            .and_then(|i| self.chances.get(i))
            .copied()
            .unwrap_or(0)
    }
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
    /// Shots reach at full damage up to this hex distance (BAT-RNG-002).
    pub ranged_full_damage_distance: u8,
    /// Damage reduction of shots beyond that distance, percent (BAT-RNG-002).
    pub ranged_penalty_percent: u8,
    /// Damage reduction of shooters' melee strikes, percent (BAT-RNG-003).
    pub melee_penalty_percent: u8,
    /// Sides of the morale die (≥ 1).
    pub morale_dice: u8,
    /// Good-morale chances by morale level; morale is at most `levels()` (BAT-MOR-001/003).
    pub good_morale: RollTable,
    /// Bad-morale chances by negative morale level; morale is at least `-levels()`
    /// (BAT-MOR-001/004).
    pub bad_morale: RollTable,
    /// Sides of the luck die (≥ 1).
    pub luck_dice: u8,
    /// Lucky-strike chances by luck level; luck is at most `levels()` (BAT-LCK-001/002).
    pub good_luck: RollTable,
    /// Head hexes `(x, y)` of the attacker's war machines by kind (shooting machine, ammo
    /// supply, first aid); the defender's are mirrored (BAT-WAR-001).
    pub war_machine_positions: [(u8, u8); WAR_MACHINE_KINDS],
    /// Starting rows by formation ([`Formation::index`]), army size and stack index.
    formations: [[[u8; MAX_ARMY_SLOTS]; MAX_ARMY_SLOTS]; Formation::COUNT],
    /// Width in columns of the tactics zone by Tactics level difference minus one, each between
    /// 1 and the number of standable columns (BAT-TAC-001).
    pub tactics_zone_columns: [u8; MAX_SKILL_LEVEL as usize],
    /// How a surrender is priced (BAT-FLE-002); `None`: no side can surrender.
    pub surrender: Option<SurrenderParams>,
    /// How many spells a hero may cast per round (BAT-MAG-001).
    pub hero_casts_per_round: u8,
    /// A side may create a stack (Clone, Summon Elemental) only while it has fewer stacks, dead
    /// ones included (BAT-MAG-037; 20 in the original), at most [`MAX_SIDE_STACKS`].
    pub side_stack_limit: u8,
}

/// The price of a surrender (BAT-FLE-002): `cost_percent` of the summed cost in `resource` of the
/// surrendering side's living stacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurrenderParams {
    /// The resource the price is paid in.
    pub resource: ResourceId,
    /// The share of the stacks' cost, percent.
    pub cost_percent: u16,
}

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

/// Kinds of war machines a battle places (shooting machine, ammo supply, first aid; BAT-WAR-001).
pub const WAR_MACHINE_KINDS: usize = 3;

impl BattleParams {
    /// Starting row of the `index`-th stack (in slot order) of an army with `stacks` stacks in
    /// `formation`.
    pub fn formation_row(&self, formation: Formation, stacks: usize, index: usize) -> Option<u8> {
        if index >= stacks || stacks > usize::from(self.army_slots) {
            return None;
        }
        // `1 <= stacks <= army_slots <= MAX_ARMY_SLOTS` here, so the lookups cannot fail.
        self.formations[formation.index()]
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

/// A compiled obstacle type (BAT-GEO-004).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObstacleType {
    /// Key.
    pub key: ContentKey,
    /// Terrains it can appear on.
    pub terrains: Vec<TerrainId>,
    /// Usual obstacles: free hexes needed to the right of the anchor.
    pub width: u8,
    /// Usual obstacles: rows needed above the anchor.
    pub height: u8,
    /// Blocked hexes: `(dx, dy)` from the anchor, or `(x, y)` for absolute obstacles.
    pub blocked: Vec<(i8, i8)>,
    /// Fixed position, at most once per battle.
    pub absolute: bool,
}

/// Most direct upgrades of one creature type.
pub const MAX_UPGRADES: usize = 3;

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
    /// The creature types it upgrades into, in pack order (E04a T3).
    pub upgrades: Vec<CreatureTypeId>,
}

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
    obstacles: Vec<ObstacleType>,
    creatures: Vec<CreatureType>,
    creature_info: Vec<CreatureInfo>,
    creature_abilities: Vec<Vec<CompiledAbility>>,
    creature_keys: Vec<ContentKey>,
    creature_index: BTreeMap<String, CreatureTypeId>,
    hero: HeroContent,
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
}

/// What abilities may refer to: the creatures, spells and spell schools compiled so far.
struct AbilityIndex<'a> {
    creatures: &'a BTreeMap<String, CreatureTypeId>,
    spells: &'a BTreeMap<String, SpellId>,
    schools: &'a BTreeMap<String, SchoolId>,
}

/// Resolves the references of an ability list against the content compiled so far, restricted
/// to the pack's dependency closure.
fn abilities(
    defs: &[AbilityDef],
    scope: &Scope<'_>,
    index: &AbilityIndex<'_>,
) -> Result<Vec<CompiledAbility>, CompileError> {
    defs.iter()
        .map(|a| {
            a.try_map(
                |key| scope.resolve(key, index.creatures, "creature"),
                |key| scope.resolve(key, index.spells, "spell"),
                |key| scope.resolve(key, index.schools, "spell school"),
            )
        })
        .collect()
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

        let mut resources: Vec<ResourceInfo> = Vec::new();
        let mut resource_index = BTreeMap::new();
        let mut factions: Vec<FactionInfo> = Vec::new();
        let mut faction_index = BTreeMap::new();
        let mut terrains: Vec<TerrainInfo> = Vec::new();
        let mut terrain_index = BTreeMap::new();
        let mut obstacles: Vec<ObstacleType> = Vec::new();
        let mut obstacle_index = BTreeMap::new();
        let mut creatures = Vec::new();
        let mut creature_info = Vec::new();
        let mut creature_abilities: Vec<Vec<CompiledAbility>> = Vec::new();
        let mut creature_keys: Vec<ContentKey> = Vec::new();
        let mut creature_index = BTreeMap::new();
        let mut hero = HeroContent::default();
        let mut battle_def: Option<&BattleParamsDef> = None;
        let mut surrender: Option<SurrenderParams> = None;
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
            let terrain =
                |key: &ContentKey| resolve_in(id, closure, key, &terrain_index, "terrain");
            for def in &pack.factions {
                check_namespace(&def.key, id)?;
                let handle = FactionId::from_index(factions.len())
                    .ok_or(CompileError::TooManyEntries("faction"))?;
                insert_unique(&mut faction_index, &def.key, handle)?;
                factions.push(FactionInfo {
                    key: def.key.clone(),
                    name: def.name.clone(),
                    native_terrain: def.native_terrain.as_ref().map(terrain).transpose()?,
                });
            }
            for def in &pack.obstacles {
                check_namespace(&def.key, id)?;
                let handle = Handle::<ObstacleType>::from_index(obstacles.len())
                    .ok_or(CompileError::TooManyEntries("obstacle"))?;
                insert_unique(&mut obstacle_index, &def.key, handle)?;
                obstacles.push(ObstacleType {
                    key: def.key.clone(),
                    terrains: def.terrains.iter().map(terrain).collect::<Result<_, _>>()?,
                    width: def.width,
                    height: def.height,
                    blocked: def.blocked.clone(),
                    absolute: def.absolute,
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
                    upgrades: Vec::new(),
                });
                creature_keys.push(def.key.clone());
            }
            // The pack's spells and schools come first: abilities may name them.
            let refs = Refs {
                pack: id,
                closure,
                resources: &resource_index,
                factions: &faction_index,
                creatures: &creature_index,
            };
            hero.add_pack(pack, &refs)?;
            let index = AbilityIndex {
                creatures: &creature_index,
                spells: &hero.spells.index,
                schools: &hero.schools.index,
            };
            // Abilities and upgrades may refer to any creature of the closure, including later
            // ones of this pack, so they are resolved once the pack's creatures are registered.
            let first = creature_abilities.len();
            for (def, info) in pack
                .creatures
                .iter()
                .zip(creature_info.get_mut(first..).unwrap_or_default())
            {
                creature_abilities.push(abilities(&def.info.abilities, &scope, &index)?);
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
                        apply_patch(&o.patch, target, &scope, &index)
                    })
                    .transpose()?;
            }
            if let Some(b) = &pack.battle {
                if battle_def.is_some() {
                    return Err(CompileError::DuplicateBattleParams(id.clone()));
                }
                battle_def = Some(b);
                // rule: BAT-FLE-002
                surrender = b
                    .surrender
                    .as_ref()
                    .map(|s| {
                        resolve_in(id, closure, &s.resource, &resource_index, "resource").map(
                            |resource| SurrenderParams {
                                resource,
                                cost_percent: s.cost_percent,
                            },
                        )
                    })
                    .transpose()?;
            }
        }
        for (i, (((stats, info), abils), key)) in creatures
            .iter()
            .zip(&creature_info)
            .zip(&creature_abilities)
            .zip(&creature_keys)
            .enumerate()
        {
            validate_creature(i, key, stats, info, abils)?;
        }
        let battle = compile_battle(
            battle_def.ok_or(CompileError::MissingBattleParams)?,
            surrender,
        )?;
        hero.validate(&creatures, battle.army_slots)?;
        for (abils, key) in creature_abilities.iter().zip(&creature_keys) {
            validate_strike_spells(key, abils, &hero.spells.items)?;
        }
        validate_raised(&creatures, &creature_abilities, &creature_keys)?;
        for o in &obstacles {
            validate_obstacle(o, &battle)?;
        }

        let mut profiles = BTreeMap::new();
        for pack in &order {
            for p in &pack.profiles {
                check_namespace(&p.key, &pack.manifest.id)?;
                if profiles.insert(p.key.as_str(), p).is_some() {
                    return Err(CompileError::DuplicateKey(String::from(p.key.as_str())));
                }
            }
        }
        // Battle modifier pools are sized by the number of distinct aura values (ADR-0012 §3).
        let auras: BTreeSet<u32> = creature_abilities
            .iter()
            .flatten()
            .filter_map(Ability::aura_key)
            .collect();
        if auras.len() > MAX_AURA_VALUES {
            return Err(CompileError::TooManyEntries("distinct aura"));
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
            terrains,
            terrain_index,
            obstacles,
            creatures,
            creature_info,
            creature_abilities,
            creature_keys,
            creature_index,
            hero,
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

    /// Obstacle types, in load order.
    pub fn obstacles(&self) -> &[ObstacleType] {
        &self.obstacles
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
    index: &AbilityIndex<'_>,
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
        *abils = abilities(defs, scope, index)?;
    }
    if let Some(keys) = &p.upgrades {
        info.upgrades = upgrades(keys, scope, index.creatures)?;
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
    index: usize,
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
    // War machines may have no damage and speed 0 (First Aid Tent, Ammo Cart; BAT-WAR-001).
    let machine = def.flags.contains(CreatureFlag::WarMachine);
    if def.damage_min == 0 && !(machine && def.damage_max == 0) {
        return invalid("damage_min must be at least 1 (war machines: or damage [0, 0])");
    }
    if def.damage_max < def.damage_min {
        return invalid("damage_max must not be below damage_min");
    }
    if def.hit_points == 0 {
        return invalid("hit_points must be at least 1");
    }
    // War machines never move, also when their side commands them (BAT-WAR-001, BAT-WAR-006).
    if (def.speed == 0) != machine {
        return invalid("speed must be 0 for war machines and at least 1 otherwise");
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
    // Only war machines act automatically; an army stack with these would never act.
    let automatic = abils
        .iter()
        .any(|a| matches!(a, Ability::AmmoSupply | Ability::FirstAid { .. }));
    if automatic && !machine {
        return invalid("ammo_supply and first_aid require flag war_machine");
    }
    Ok(())
}

/// Checks that the spells a creature's strike and cast abilities name do what the abilities need:
/// a spell after a strike has a timed or dispel effect and nothing else (BAT-MAG-040, BAT-MAG-045,
/// BAT-MAG-046), a damage spell after a strike has a damage effect (BAT-MAG-041), a caster's spell
/// has damage, timed, dispel or heal effects only and takes a target stack at the caster's
/// mastery in every profile, without a health limit (the original's casters cast none; a health
/// limit reads the hero, BAT-MAG-048); a random caster's drawn spell likewise, aimed at a stack
/// or a hex and placing no obstacle (BAT-MAG-049); an innate spell has a Fire Shield or Magic
/// Mirror change (BAT-MAG-057, BAT-MAG-058).
fn validate_strike_spells(
    key: &ContentKey,
    abils: &[CompiledAbility],
    spells: &[Spell],
) -> Result<(), CompileError> {
    for a in abils {
        let spell = |id: SpellId| {
            spells
                .get(id.index())
                .map_or(&[][..], |s| &s.magic.effects[..])
        };
        let reason = match *a {
            Ability::SpellAfterStrike { spell: id, .. } => {
                let effects = spell(id);
                let fits = !effects.is_empty()
                    && effects
                        .iter()
                        .all(|e| e.timed().is_some() || *e == SpellEffect::Dispel);
                (!fits)
                    .then_some("spell_after_strike needs a spell with timed or dispel effects only")
            }
            Ability::SpellDamageAfterStrike { spell: id, .. } => (!spell(id)
                .contains(&SpellEffect::Damage))
            .then_some("spell_damage_after_strike needs a spell with a damage effect"),
            Ability::SpellCaster {
                spell: id, mastery, ..
            } => {
                let effects = spell(id);
                let kinds = !effects.is_empty()
                    && effects.iter().all(|e| {
                        matches!(
                            e,
                            SpellEffect::Damage
                                | SpellEffect::Timed(_)
                                | SpellEffect::Dispel
                                | SpellEffect::Heal
                        )
                    });
                let aimed = spells.get(id.index()).is_some_and(|s| {
                    !s.magic.health_limit
                        && [false, true]
                            .iter()
                            .all(|&early| s.magic.target_at(mastery, early) == SpellTarget::Stack)
                });
                (!(kinds && aimed)).then_some(
                    "spell_caster needs a spell with damage, timed, dispel or heal effects and no \
                     health limit that takes a target stack at its mastery",
                )
            }
            Ability::StackRaiser { spell: id, .. } => spells
                .get(id.index())
                .is_none_or(|s| !s.magic.raises())
                .then_some("stack_raiser needs a spell with a raise effect"),
            // The rules read an innate Fire Shield or Magic Mirror only (BAT-MAG-057, BAT-MAG-058).
            Ability::InnateSpell { spell: id, .. } => (!spell(id).iter().any(|e| {
                matches!(
                    e.timed(),
                    Some(TimedChange::FireShield | TimedChange::MagicMirror)
                )
            }))
            .then_some(
                "innate_spell needs a spell with a timed fire_shield or magic_mirror change",
            ),
            Ability::DrawnSpell { spell: id, .. } => {
                // A drawn spell needs its caster (`ability::list_error`): the targets it takes at
                // its caster's mastery, a random caster's a stack or a hex, a helpful caster's a
                // stack, a periodic caster's none, and a periodic caster's does no damage, so its
                // cast at a stack's activation ends no battle (BAT-MAG-049, BAT-MAG-050,
                // BAT-MAG-053).
                let (mastery, targets, damage): (u8, &[SpellTarget], bool) = abils
                    .iter()
                    .find_map(|a| match *a {
                        Ability::RandomSpellCaster { mastery, .. } => {
                            Some((mastery, &[SpellTarget::Stack, SpellTarget::Hex][..], true))
                        }
                        Ability::HelpfulSpellCaster { mastery, .. } => {
                            Some((mastery, &[SpellTarget::Stack][..], true))
                        }
                        Ability::PeriodicSpellCaster { mastery, .. } => {
                            Some((mastery, &[SpellTarget::None][..], false))
                        }
                        _ => None,
                    })
                    .unwrap_or((0, &[], true));
                let effects = spell(id);
                let kinds = !effects.is_empty()
                    && effects.iter().all(|e| {
                        (damage && *e == SpellEffect::Damage)
                            || matches!(
                                e,
                                SpellEffect::Timed(_) | SpellEffect::Dispel | SpellEffect::Heal
                            )
                    });
                let aimed = spells.get(id.index()).is_some_and(|s| {
                    !s.magic.health_limit
                        && s.magic.obstacle.is_none()
                        && [false, true]
                            .iter()
                            .all(|&early| targets.contains(&s.magic.target_at(mastery, early)))
                });
                (!(kinds && aimed)).then_some(
                    "drawn_spell needs a spell with damage, timed, dispel or heal effects and no \
                     health limit or obstacle that takes a target stack or hex (for a helpful \
                     caster a stack; for a periodic caster no target, and no damage) at its \
                     caster's mastery",
                )
            }
            _ => None,
        };
        if let Some(reason) = reason {
            return Err(CompileError::InvalidCreature {
                key: String::from(key.as_str()),
                reason,
            });
        }
    }
    Ok(())
}

/// Checks an obstacle type against the battlefield: at least one terrain and one blocked hex;
/// usual obstacles need a width and height of at least 1 and blocked hexes no lower than the
/// anchor (`dy <= 0`); absolute obstacles block standable hexes of the field.
/// The creatures `corpse_raiser` abilities raise (BAT-MAG-052): one-hex creatures, at most
/// [`ability::MAX_RAISED_CREATURES`] of them.
fn validate_raised(
    creatures: &[CreatureType],
    abilities: &[Vec<CompiledAbility>],
    keys: &[ContentKey],
) -> Result<(), CompileError> {
    let mut raised = BTreeSet::new();
    for (abils, key) in abilities.iter().zip(keys) {
        for a in abils {
            let Ability::CorpseRaiser { creature, .. } = *a else {
                continue;
            };
            if creatures
                .get(creature.index())
                .is_none_or(|c| c.flags.contains(CreatureFlag::TwoHex))
            {
                return Err(CompileError::InvalidCreature {
                    key: String::from(key.as_str()),
                    reason: "corpse_raiser needs a one-hex creature",
                });
            }
            raised.insert(creature.index());
        }
    }
    if raised.len() > ability::MAX_RAISED_CREATURES {
        return Err(CompileError::TooManyEntries("corpse_raiser creature"));
    }
    Ok(())
}

fn validate_obstacle(o: &ObstacleType, battle: &BattleParams) -> Result<(), CompileError> {
    let invalid = |reason| {
        Err(CompileError::InvalidObstacle {
            key: String::from(o.key.as_str()),
            reason,
        })
    };
    if o.terrains.is_empty() || o.blocked.is_empty() {
        return invalid("an obstacle needs at least one terrain and one blocked hex");
    }
    let f = battle.field;
    if o.absolute {
        let standable = |&(x, y): &(i8, i8)| {
            i32::from(x) >= i32::from(f.side_columns)
                && i32::from(x) < i32::from(f.width) - i32::from(f.side_columns)
                && y >= 0
                && i32::from(y) < i32::from(f.height)
        };
        if !o.blocked.iter().all(standable) {
            return invalid("absolute obstacles must block standable hexes of the field");
        }
    } else if o.width == 0 || o.height == 0 || o.blocked.iter().any(|&(_, dy)| dy > 0) {
        return invalid("usual obstacles need width and height >= 1 and blocked dy <= 0");
    }
    Ok(())
}

fn compile_battle(
    def: &BattleParamsDef,
    surrender: Option<SurrenderParams>,
) -> Result<BattleParams, CompileError> {
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
    if def.ranged_penalty_percent > 100 || def.melee_penalty_percent > 100 {
        return invalid("ranged and melee penalties must not exceed 100 percent");
    }
    let table = |dice: u8, chances: &[u8]| -> Result<RollTable, CompileError> {
        if dice == 0 {
            return Err(CompileError::InvalidBattleParams(
                "morale_dice and luck_dice must be at least 1",
            ));
        }
        if chances.len() > MAX_ROLL_LEVELS || chances.iter().any(|&c| c > dice) {
            return Err(CompileError::InvalidBattleParams(
                "morale and luck chances: at most 8 levels, each at most the die's sides",
            ));
        }
        let mut t = RollTable::default();
        for (slot, &c) in t.chances.iter_mut().zip(chances) {
            *slot = c;
        }
        // `chances.len() <= MAX_ROLL_LEVELS`.
        t.len = u8::try_from(chances.len()).unwrap_or(0);
        Ok(t)
    };
    let good_morale = table(def.morale_dice, &def.good_morale_chances)?;
    let bad_morale = table(def.morale_dice, &def.bad_morale_chances)?;
    // A certain freeze would let no stack ever act again (BAT-MOR-004).
    if def.bad_morale_chances.iter().any(|&c| c >= def.morale_dice) {
        return invalid("bad morale chances must be below morale_dice");
    }
    let good_luck = table(def.luck_dice, &def.good_luck_chances)?;
    let mut war_machine_positions = [(0, 0); WAR_MACHINE_KINDS];
    if def.war_machine_positions.len() != WAR_MACHINE_KINDS
        || def
            .war_machine_positions
            .iter()
            .any(|&[x, y]| usize::from(x) >= w || usize::from(y) >= h)
    {
        return invalid("war_machine_positions needs 3 hexes [x, y] on the field");
    }
    // rule: BAT-WAR-001
    // Any machine may be two hexes wide, with its tail on the hex behind its head (x - 1 for
    // the attacker, x + 1 for the mirrored defender): every footprint must fit the field, and
    // no two footprints of either side may share a hex.
    let mut machine_hexes: BTreeSet<(usize, usize)> = BTreeSet::new();
    for &[x, y] in &def.war_machine_positions {
        let (x, y) = (usize::from(x), usize::from(y));
        let Some(tail) = x.checked_sub(1) else {
            return invalid("war machine positions need room for a two-hex machine behind them");
        };
        // `x < w`, so the mirrored head and its tail `w - x` stay on the field.
        let mirrored = w - 1 - x;
        for hex in [(x, y), (tail, y), (mirrored, y), (w - x, y)] {
            if !machine_hexes.insert(hex) {
                return invalid("war machine positions overlap (as two-hex footprints)");
            }
        }
    }
    for (slot, &[x, y]) in war_machine_positions
        .iter_mut()
        .zip(&def.war_machine_positions)
    {
        *slot = (x, y);
    }
    let mut formations = [[[0u8; MAX_ARMY_SLOTS]; MAX_ARMY_SLOTS]; Formation::COUNT];
    for (table, out) in [&def.loose_formation, &def.tight_formation]
        .into_iter()
        .zip(formations.iter_mut())
    {
        compile_formation(table, slots, h, out)?;
    }
    // rule: BAT-TAC-001
    let mut tactics_zone_columns = [0u8; MAX_SKILL_LEVEL as usize];
    if def.tactics_zone_columns.len() != tactics_zone_columns.len()
        || def
            .tactics_zone_columns
            .iter()
            .any(|&c| c == 0 || usize::from(c) > w - 2 * side)
    {
        return invalid(
            "tactics_zone_columns needs one width per skill level, each between 1 and the standable columns",
        );
    }
    tactics_zone_columns.copy_from_slice(&def.tactics_zone_columns);
    // rule: BAT-MAG-037
    if usize::from(def.side_stack_limit) > MAX_SIDE_STACKS {
        return invalid("side_stack_limit must be at most 21");
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
        ranged_full_damage_distance: def.ranged_full_damage_distance,
        ranged_penalty_percent: def.ranged_penalty_percent,
        melee_penalty_percent: def.melee_penalty_percent,
        morale_dice: def.morale_dice,
        good_morale,
        bad_morale,
        luck_dice: def.luck_dice,
        good_luck,
        war_machine_positions,
        formations,
        tactics_zone_columns,
        surrender,
        hero_casts_per_round: def.hero_casts_per_round,
        side_stack_limit: def.side_stack_limit,
    })
}

/// Validates a formation table (one entry per army size `1..=slots`, entry `n` with `n`
/// distinct rows of a field with `height` rows) into `out`.
fn compile_formation(
    table: &[Vec<u8>],
    slots: usize,
    height: usize,
    out: &mut [[u8; MAX_ARMY_SLOTS]; MAX_ARMY_SLOTS],
) -> Result<(), CompileError> {
    let invalid = |m| Err(CompileError::InvalidBattleParams(m));
    if table.len() != slots {
        return invalid("formations need one entry per army size 1..=army_slots");
    }
    for (n, (rows, out)) in table.iter().zip(out.iter_mut()).enumerate() {
        if rows.len() != n + 1 {
            return invalid("formation entry n must list n rows");
        }
        for (i, (&row, slot)) in rows.iter().zip(out.iter_mut()).enumerate() {
            if usize::from(row) >= height {
                return invalid("formation row outside the field");
            }
            if rows.get(..i).is_some_and(|earlier| earlier.contains(&row)) {
                return invalid("formation rows must be distinct");
            }
            *slot = row;
        }
    }
    Ok(())
}

impl CanonicalHash for Ruleset {
    /// Encoding v13: version, packs (id, version) in load order, profile key, quirk bits,
    /// resources (key, name), terrains (key, name), factions (key, name, native terrain as an
    /// optional handle), obstacles (key, terrains, width, height, blocked pairs, absolute flag),
    /// creatures (key, stats in field order including shots
    /// and flag bits, then the info fields in field order — a faction as presence flag + handle,
    /// the cost as length + pairs — then the abilities as length + each ability's encoding),
    /// battle parameters in field order (v4: with the ranged/melee penalty parameters; v5: with
    /// the morale and luck dice and tables, each table as length + all 8 entries; v7: the war
    /// machine positions) with the full formation tables (v8: loose, then tight). v9 (E04a T3):
    /// each creature ends with its upgrades (length + handles), and the hero content follows the
    /// battle parameters (`heroes::HeroContent`'s encoding). v10 (E04a T3b): the hero content
    /// ends with the artifact slots and artifacts, and hero effects have nine more kinds. v11
    /// (E04a T4a): artifact slots carry the backpack flag, and the hero content ends with the
    /// hero parameters. v12 (E04a T8): the battle parameters end with the tactics zone widths
    /// (all entries, no length); v13 (E04a T9b): then the surrender parameters (presence flag,
    /// resource handle, percent). v14 (E04b T2a): each spell ends with its battle magic
    /// (`SpellMagic`'s encoding), and hero effects have three more kinds (`school_mastery`,
    /// `spell_damage`, `spell_book`) and a growth in `spell_specialty`. v15 (E04b T2b): the
    /// battle parameters end with the hero casts per round, and the battle magic has a side.
    /// v17 (E04b T3a): the battle magic's effects are coded as kind and change and end with
    /// mind, mass, duration and cumulative; each spell ends with its counters; hero effects have
    /// `spell_duration`. v24 (E04b T5c1): an obstacle shape has `random`, an obstacle kind
    /// `pit`. v25 (E04b T5b): the battle magic ends with the masteries that remove each
    /// obstacle kind; effect kind `remove_obstacle`. v27 (E04b T6b): the battle parameters end
    /// with the side stack limit; each spell ends with the creature it summons; effect kinds
    /// `clone` and `summon`. v28 (E04b T7a1): the battle magic ends with `dispels_positive`;
    /// abilities `spell_after_strike` (29) and `spell_damage_after_strike` (30). v29 (E04b T7a2): the
    /// battle magic ends with `restores_health`; timed changes `petrify`, `paralyze`, `poison`,
    /// `age` (27 to 30). v30 (E04b T7a3): `spell_after_strike` ends with `defended`; ability
    /// `damage_after_strike` (31); timed change `bind` (31). v31 (E04b T7b1): ability
    /// `spell_caster` (32). v38 (E04b T7c2b, T7c3): abilities `rebirth` (45) and `fear` (46).
    /// v39 (E04b T8a): each spell ends with `granted_by_name_only`; hero effects `spells` (27),
    /// `spell_mastery` (28) and `scroll` (29). v40 (E04b T8b): hero effects `no_casting` (30),
    /// `spell_level_limit` (31), `spell_immunity` (32), `spell_level_immunity` (33),
    /// `mind_immunity` (34) and `negate_immunities` (35); ability `spell_immunity` ends with
    /// `absolute`; ability `ignores_magic_block` (47). v41 (E04b T8b3): target conditions end with
    /// `needs_damage`. v42 (E04b T8c, T8d): hero effects `opening_spell` (36) and `eagle_eye` (37).
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
        h.write_len(self.obstacles.len());
        for o in &self.obstacles {
            o.key.as_str().hash_into(h);
            h.write_len(o.terrains.len());
            for t in &o.terrains {
                t.hash_into(h);
            }
            h.write_u8(o.width);
            h.write_u8(o.height);
            h.write_len(o.blocked.len());
            for &(a, b) in &o.blocked {
                h.write_i32(i32::from(a));
                h.write_i32(i32::from(b));
            }
            h.write_bool(o.absolute);
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
            info.upgrades.hash_into(h);
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
        h.write_u8(b.ranged_full_damage_distance);
        h.write_u8(b.ranged_penalty_percent);
        h.write_u8(b.melee_penalty_percent);
        h.write_u8(b.morale_dice);
        for t in [&b.good_morale, &b.bad_morale] {
            h.write_u8(t.len);
            h.write_bytes(&t.chances);
        }
        h.write_u8(b.luck_dice);
        h.write_u8(b.good_luck.len);
        h.write_bytes(&b.good_luck.chances);
        for &(x, y) in &b.war_machine_positions {
            h.write_u8(x);
            h.write_u8(y);
        }
        for row in b.formations.iter().flatten() {
            h.write_bytes(row);
        }
        h.write_bytes(&b.tactics_zone_columns);
        h.write_u8(u8::from(b.surrender.is_some()));
        if let Some(s) = b.surrender {
            s.resource.hash_into(h);
            h.write_u16(s.cost_percent);
        }
        h.write_u8(b.hero_casts_per_round);
        h.write_u8(b.side_stack_limit);
        self.hero.hash_into(h);
    }
}

#[cfg(test)]
pub(crate) mod tests;
