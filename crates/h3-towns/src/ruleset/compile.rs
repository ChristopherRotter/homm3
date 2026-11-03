//! The town part's compiler (E09 increment 1's design note §2): the load order, each pack's
//! parameters, records and overrides in it, then the checks across packs.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use h3_content::{ContentKey, CreatureTypeId, FactionId, ResourceId, Ruleset};
use h3_contracts::Moat;

use super::{
    BUILDING_SLOTS, Building, BuildingDef, BuildingRoles, CreatureLevel, GarrisonRow,
    MAX_CREATURE_LEVELS, TownCompileError, TownDef, TownPack, TownParams, TownParamsDef,
    TownRecord,
};
use crate::building::{BuildingId, BuildingSet, MAX_BUILDINGS};

type Result<T> = core::result::Result<T, TownCompileError>;

/// A record being compiled: the pack that defined it and its fields (the demoted Capitol's
/// building is resolved once the parameters are known).
struct Entry {
    pack: String,
    key: String,
    faction: FactionId,
    moat: Moat,
    siege_obstacles: bool,
    silo: Vec<(ResourceId, u32)>,
    creatures: Vec<CreatureLevel>,
    buildings: Vec<Building>,
}

/// The pack being compiled: references resolve against the core within its dependency closure.
struct Scope<'a> {
    core: &'a Ruleset,
    pack: &'a str,
    closure: &'a BTreeSet<String>,
}

impl Scope<'_> {
    /// `key` must belong to a pack of the closure.
    fn check(&self, key: &ContentKey) -> Result<()> {
        if self.closure.contains(key.pack()) {
            Ok(())
        } else {
            Err(TownCompileError::ReferenceOutsideDependencies {
                key: String::from(key.as_str()),
                pack: String::from(self.pack),
            })
        }
    }

    fn faction(&self, key: &ContentKey) -> Result<FactionId> {
        let id = self
            .core
            .faction_by_key(key.as_str())
            .ok_or_else(|| TownCompileError::UnknownFaction(String::from(key.as_str())))?;
        self.check(key).map(|()| id)
    }

    fn resource(&self, key: &ContentKey) -> Result<ResourceId> {
        let id = self
            .core
            .resources()
            .iter()
            .position(|r| r.key == *key)
            .and_then(ResourceId::from_index)
            .ok_or_else(|| TownCompileError::UnknownResource(String::from(key.as_str())))?;
        self.check(key).map(|()| id)
    }

    fn creature(&self, key: &ContentKey) -> Result<CreatureTypeId> {
        let id = self
            .core
            .creature_by_key(key.as_str())
            .ok_or_else(|| TownCompileError::UnknownCreature(String::from(key.as_str())))?;
        self.check(key).map(|()| id)
    }

    /// A silo's resources in the core's order: each once, at least 1.
    fn silo(&self, def: &[(ContentKey, u32)], faction: &str) -> Result<Vec<(ResourceId, u32)>> {
        let mut silo = BTreeMap::new();
        for (key, amount) in def {
            let resource = self.resource(key)?;
            if *amount == 0 || silo.insert(resource, *amount).is_some() {
                return Err(invalid_town(
                    faction,
                    "the silo gives each resource once, at least 1",
                ));
            }
        }
        Ok(silo.into_iter().collect())
    }
}

fn invalid_town(faction: &str, reason: &'static str) -> TownCompileError {
    TownCompileError::InvalidTown {
        faction: String::from(faction),
        reason,
    }
}

fn invalid_building(
    faction: &str,
    building: &BuildingDef,
    reason: &'static str,
) -> TownCompileError {
    TownCompileError::InvalidBuilding {
        faction: String::from(faction),
        building: building.key.clone(),
        reason,
    }
}

/// Compiles the packs into the parameters and the records in faction order.
pub(super) fn compile(
    core: &Ruleset,
    packs: &[TownPack],
) -> Result<(Option<TownParams>, Vec<TownRecord>)> {
    let order = load_order(core, packs)?;
    let mut params = None;
    let mut towns: BTreeMap<FactionId, Entry> = BTreeMap::new();
    for (pack, closure) in &order {
        let scope = Scope {
            core,
            pack: pack.manifest.id.as_str(),
            closure,
        };
        if let Some(def) = &pack.params {
            if params.is_some() {
                return Err(TownCompileError::DuplicateParams(pack.manifest.id.clone()));
            }
            params = Some(compile_params(&scope, def)?);
        }
        for def in &pack.towns {
            let faction = scope.faction(&def.faction)?;
            let entry = compile_town(&scope, faction, def)?;
            if towns.insert(faction, entry).is_some() {
                return Err(TownCompileError::DuplicateTown(String::from(
                    def.faction.as_str(),
                )));
            }
        }
        for o in &pack.overrides {
            let faction = scope.faction(&o.faction)?;
            let key = || String::from(o.faction.as_str());
            let entry = towns
                .get_mut(&faction)
                .ok_or_else(|| TownCompileError::UnknownTown(key()))?;
            if !closure.contains(&entry.pack) {
                return Err(TownCompileError::ReferenceOutsideDependencies {
                    key: key(),
                    pack: pack.manifest.id.clone(),
                });
            }
            if let Some(silo) = &o.patch.silo {
                entry.silo = scope.silo(silo, o.faction.as_str())?;
            }
            entry.moat = o.patch.moat.unwrap_or(entry.moat);
            entry.siege_obstacles = o.patch.siege_obstacles.unwrap_or(entry.siege_obstacles);
        }
    }
    if towns.is_empty() {
        return Ok((params, Vec::new()));
    }
    let params = params.ok_or(TownCompileError::MissingParams)?;
    let records = towns
        .into_values()
        .map(|entry| finish(&params, entry))
        .collect::<Result<Vec<_>>>()?;
    Ok((Some(params), records))
}

/// The packs in the core's load order, each with its dependency closure (its own id included),
/// as `h3-adventure`'s part compiles them.
fn load_order<'a>(
    core: &Ruleset,
    packs: &'a [TownPack],
) -> Result<Vec<(&'a TownPack, BTreeSet<String>)>> {
    let mut by_id: BTreeMap<&str, &TownPack> = BTreeMap::new();
    for pack in packs {
        let id = pack.manifest.id.as_str();
        let known = core
            .packs()
            .iter()
            .any(|(core_id, version)| core_id == id && *version == pack.manifest.version);
        if !known {
            let mut text = String::from(id);
            text.push('@');
            text.push_str(&pack.manifest.version);
            return Err(TownCompileError::UnknownPack(text));
        }
        if by_id.insert(id, pack).is_some() {
            return Err(TownCompileError::DuplicatePack(String::from(id)));
        }
    }
    let mut closures: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    let mut order = Vec::new();
    for (id, _) in core.packs() {
        let pack = *by_id
            .get(id.as_str())
            .ok_or_else(|| TownCompileError::MissingPack(id.clone()))?;
        let mut closure = BTreeSet::from([id.clone()]);
        for dependency in &pack.manifest.depends {
            let inner = closures.get(dependency.as_str()).ok_or_else(|| {
                TownCompileError::UnknownDependency {
                    pack: id.clone(),
                    dependency: dependency.clone(),
                }
            })?;
            closure.extend(inner.iter().cloned());
        }
        closures.insert(id.as_str(), closure.clone());
        order.push((pack, closure));
    }
    Ok(order)
}

fn compile_params(scope: &Scope<'_>, def: &TownParamsDef) -> Result<TownParams> {
    let gold = scope.resource(&def.gold)?;
    if def.level_2_dwelling_chance > 100 {
        return Err(TownCompileError::InvalidParams(
            "the level-2 dwelling's chance is above 100",
        ));
    }
    if def.neutral_garrison.iter().any(|row| row.chance > 100) {
        return Err(TownCompileError::InvalidParams(
            "a garrison row's chance is above 100",
        ));
    }
    if def.neutral_garrison.iter().any(
        |&GarrisonRow {
             count: [least, most],
             ..
         }| least == 0 || least > most,
    ) {
        return Err(TownCompileError::InvalidParams(
            "a garrison row's count needs 1 <= least <= most",
        ));
    }
    Ok(TownParams {
        gold,
        hall_gold: def.hall_gold,
        grail_gold: def.grail_gold,
        reveal_radius: def.reveal_radius,
        level_2_dwelling_chance: def.level_2_dwelling_chance,
        neutral_garrison: def.neutral_garrison,
        capitol_demoted_to: def.capitol_demoted_to.clone(),
    })
}

fn compile_town(scope: &Scope<'_>, faction: FactionId, def: &TownDef) -> Result<Entry> {
    let name = def.faction.as_str();
    if def.creatures.is_empty() || def.creatures.len() > MAX_CREATURE_LEVELS {
        return Err(invalid_town(name, "needs one to seven creature levels"));
    }
    let creatures = def
        .creatures
        .iter()
        .zip(1_u8..)
        .map(|(level_def, level)| {
            let base = scope.creature(&level_def.base)?;
            let fits = scope
                .core
                .creature(base)
                .is_some_and(|c| c.faction == Some(faction) && c.level == level);
            if !fits {
                return Err(invalid_town(
                    name,
                    "a level's creature is not the faction's of that level",
                ));
            }
            let upgrade = level_def
                .upgrade
                .as_ref()
                .map(|key| {
                    let upgrade = scope.creature(key)?;
                    if scope.core.upgrades(base).contains(&upgrade) {
                        Ok(upgrade)
                    } else {
                        Err(invalid_town(name, "a level's upgrade is not its base's"))
                    }
                })
                .transpose()?;
            Ok(CreatureLevel { base, upgrade })
        })
        .collect::<Result<Vec<_>>>()?;
    let silo = scope.silo(&def.silo, name)?;
    let buildings = compile_buildings(name, &def.buildings, &creatures)?;
    check_roles(name, &buildings)?;
    Ok(Entry {
        pack: String::from(scope.pack),
        key: String::from(name),
        faction,
        moat: def.moat,
        siege_obstacles: def.siege_obstacles,
        silo,
        creatures,
        buildings,
    })
}

/// A building's key: `[a-z0-9_]+`.
fn valid_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
}

/// A building's own checks: its key, slot and role values.
fn check_building(
    faction: &str,
    def: &BuildingDef,
    creatures: &[CreatureLevel],
    slots: &mut BTreeSet<u8>,
) -> Result<()> {
    let bad = |reason| Err(invalid_building(faction, def, reason));
    let roles = &def.roles;
    if !valid_key(&def.key) {
        return bad("a key is `[a-z0-9_]+`");
    }
    if def
        .slot
        .is_some_and(|slot| slot >= BUILDING_SLOTS || !slots.insert(slot))
    {
        return bad("a slot is below 41 and the faction's only one");
    }
    if roles.hall.is_some_and(|level| !(1..=4).contains(&level)) {
        return bad("a hall level is 1 to 4");
    }
    if roles.fort.is_some_and(|level| !(1..=3).contains(&level)) {
        return bad("a fort level is 1 to 3");
    }
    let dwelling_fits = roles.dwelling.is_none_or(|d| {
        usize::from(d.level)
            .checked_sub(1)
            .and_then(|i| creatures.get(i))
            .is_some_and(|level| !d.upgraded || level.upgrade.is_some())
    });
    if !dwelling_fits {
        return bad("a dwelling's level, or its upgrade, is not the faction's");
    }
    if !(-3..=3).contains(&roles.morale) || !(-3..=3).contains(&roles.luck) {
        return bad("morale and luck are -3 to 3");
    }
    Ok(())
}

fn compile_buildings(
    faction: &str,
    defs: &[BuildingDef],
    creatures: &[CreatureLevel],
) -> Result<Vec<Building>> {
    if defs.len() > MAX_BUILDINGS {
        return Err(invalid_town(faction, "has more than 64 buildings"));
    }
    let ids: Vec<BuildingId> = (0..defs.len()).filter_map(BuildingId::from_index).collect();
    let mut index: BTreeMap<&str, BuildingId> = BTreeMap::new();
    let mut slots = BTreeSet::new();
    for (def, &id) in defs.iter().zip(&ids) {
        check_building(faction, def, creatures, &mut slots)?;
        if index.insert(def.key.as_str(), id).is_some() {
            return Err(invalid_building(
                faction,
                def,
                "a key is the faction's only one",
            ));
        }
    }
    let resolve = |def: &BuildingDef, key: &str, reason| {
        index
            .get(key)
            .copied()
            .ok_or_else(|| invalid_building(faction, def, reason))
    };
    let mut direct = Vec::with_capacity(defs.len());
    let mut forbidden = Vec::with_capacity(defs.len());
    let mut auto = Vec::with_capacity(defs.len());
    for (def, &id) in defs.iter().zip(&ids) {
        let mut set = BuildingSet::EMPTY;
        for key in &def.supersedes {
            set = set.with(resolve(
                def,
                key,
                "it supersedes a building the faction lacks",
            )?);
        }
        direct.push(set);
        let mut with = BuildingSet::EMPTY;
        for key in &def.auto_with {
            with = with.with(resolve(
                def,
                key,
                "it is built with a building the faction lacks",
            )?);
        }
        if with.contains(id) {
            return Err(invalid_building(faction, def, "it is built with itself"));
        }
        auto.push(with);
        forbidden.push(
            def.forbidden_with
                .as_deref()
                .map(|key| {
                    resolve(
                        def,
                        key,
                        "it is forbidden with a building the faction lacks",
                    )
                })
                .transpose()?,
        );
    }
    let closed = close(&ids, &direct);
    let mut buildings = Vec::with_capacity(defs.len());
    for ((((def, &id), &supersedes), &forbidden_with), &auto_with) in defs
        .iter()
        .zip(&ids)
        .zip(&closed)
        .zip(&forbidden)
        .zip(&auto)
    {
        if supersedes.contains(id) {
            return Err(invalid_building(
                faction,
                def,
                "it supersedes itself (a cycle)",
            ));
        }
        if forbidden_cycle(&forbidden, id) {
            return Err(invalid_building(
                faction,
                def,
                "it is forbidden with itself (a cycle)",
            ));
        }
        buildings.push(Building {
            key: def.key.clone(),
            name: def.name.clone(),
            slot: def.slot,
            supersedes,
            forbidden_with,
            auto_with,
            roles: def.roles,
        });
    }
    Ok(buildings)
}

/// Each building's superseded set closed: what it supersedes directly and, transitively, what
/// those supersede.
fn close(ids: &[BuildingId], direct: &[BuildingSet]) -> Vec<BuildingSet> {
    direct
        .iter()
        .map(|&start| {
            let mut set = start;
            loop {
                let next = ids
                    .iter()
                    .zip(direct)
                    .filter(|(id, _)| set.contains(**id))
                    .fold(set, |acc, (_, d)| acc.union(*d));
                if next == set {
                    break set;
                }
                set = next;
            }
        })
        .collect()
}

/// Whether following `forbidden_with` from `start` comes back to it.
fn forbidden_cycle(forbidden: &[Option<BuildingId>], start: BuildingId) -> bool {
    let next = |b: BuildingId| forbidden.get(b.index()).copied().flatten();
    let mut current = next(start);
    for _ in 0..forbidden.len() {
        match current {
            Some(b) if b == start => return true,
            Some(b) => current = next(b),
            None => return false,
        }
    }
    false
}

/// The record's roles across its buildings: one building of each hall and fort level, one Grail,
/// one silo; each dwelling once, the level-1 and level-2 ones present.
fn check_roles(faction: &str, buildings: &[Building]) -> Result<()> {
    let count =
        |role: &dyn Fn(&BuildingRoles) -> bool| buildings.iter().filter(|b| role(&b.roles)).count();
    if (1..=4).any(|level| count(&|r| r.hall == Some(level)) != 1) {
        return Err(invalid_town(
            faction,
            "needs one building of each hall level 1 to 4",
        ));
    }
    if (1..=3).any(|level| count(&|r| r.fort == Some(level)) != 1) {
        return Err(invalid_town(
            faction,
            "needs one building of each fort level 1 to 3",
        ));
    }
    if count(&|r| r.grail) != 1 || count(&|r| r.silo) != 1 {
        return Err(invalid_town(
            faction,
            "needs one Grail building and one Resource Silo",
        ));
    }
    let mut dwellings = BTreeSet::new();
    for d in buildings.iter().filter_map(|b| b.roles.dwelling) {
        if !dwellings.insert((d.level, d.upgraded)) {
            return Err(invalid_town(faction, "has a dwelling twice"));
        }
    }
    if !dwellings.contains(&(1, false)) || !dwellings.contains(&(2, false)) {
        return Err(invalid_town(
            faction,
            "needs the level-1 and level-2 dwellings",
        ));
    }
    Ok(())
}

/// The record with its demoted Capitol's building: the parameters' key, a hall-3 building.
fn finish(params: &TownParams, entry: Entry) -> Result<TownRecord> {
    let capitol_demoted_to = entry
        .buildings
        .iter()
        .zip((0..MAX_BUILDINGS).filter_map(BuildingId::from_index))
        .find(|(b, _)| b.key == params.capitol_demoted_to && b.roles.hall == Some(3))
        .map(|(_, id)| id)
        .ok_or_else(|| {
            invalid_town(
                &entry.key,
                "the demoted Capitol's key is not its hall-3 building",
            )
        })?;
    Ok(TownRecord {
        faction: entry.faction,
        moat: entry.moat,
        siege_obstacles: entry.siege_obstacles,
        silo: entry.silo,
        creatures: entry.creatures,
        buildings: entry.buildings,
        capitol_demoted_to,
    })
}
