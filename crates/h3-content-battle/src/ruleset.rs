//! The battle's compiled, immutable ruleset part (R01; ADR-0006, ADR-0015 point 3): compiled on a
//! core [`Ruleset`] from one [`BattlePack`] per core pack, like the adventure's part. It keeps the
//! core it was compiled on and dereferences to it, so battle code holds one handle; its rows
//! shadow the core's accessors of the same name ([`BattleRuleset::creature`],
//! [`BattleRuleset::spell`], [`BattleRuleset::spells`]) and carry copies of the core fields the
//! battle reads.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ops::Deref;

use h3_content::{
    CompileError, ContentKey, CreatureFlag, CreatureFlags, CreatureInfo, CreatureTypeId,
    QuirkRegistry, REGISTRY, Ruleset, SchoolId, SpellCategory, SpellId, TerrainId,
    override_applies,
};
use h3_kernel::{CanonicalHash, Handle, StateHasher};

use crate::ability::{self, Ability, AbilityDef, CompiledAbility};
use crate::battlefield::{self, Battlefield, BattlefieldId};
use crate::error::BattleCompileError;
use crate::pack::{BattleCreaturePatch, BattlePack, BattleParamsDef, BattleSpellDef};
use crate::params::{self, BattleParams, MAX_AURA_VALUES, SurrenderParams};
use crate::spell::{SpellEffect, SpellMagic, SpellPolarity, SpellTarget, TimedChange};

/// Version of the battle part's hash encoding. Bump when the encoding below changes.
pub const BATTLE_RULESET_HASH_VERSION: u32 = 2;

/// Most spells with a summon effect in a ruleset (the battle keeps a side's stack of each
/// creature its hero can summon from the battle's start, E04b T6b; four in the original).
pub const MAX_SUMMON_SPELLS: usize = 4;

/// A compiled creature type's battle row (the hot path's), indexed by the core's
/// [`CreatureTypeId`]. `speed` and `flags` are copies of the core row's.
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
    /// Speed (≥ 1; 0 for war machines), the core row's.
    pub speed: u8,
    /// Ranged shots per battle (0 for creatures without ammunition).
    pub shots: u16,
    /// Spell casts per battle.
    pub casts: u8,
    /// Flags, the core row's.
    pub flags: CreatureFlags,
}

/// An obstacle type's handle: an index into [`BattleRuleset::obstacles`].
pub type ObstacleTypeId = Handle<ObstacleType>;

/// A compiled obstacle type (BAT-GEO-004).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObstacleType {
    /// Key.
    pub key: ContentKey,
    /// Terrains it can appear on.
    pub terrains: Vec<TerrainId>,
    /// Special battlefields with their own obstacles it can appear on (E04c).
    pub battlefields: Vec<BattlefieldId>,
    /// Usual obstacles: free hexes needed to the right of the anchor.
    pub width: u8,
    /// Usual obstacles: rows needed above the anchor.
    pub height: u8,
    /// Blocked hexes: `(dx, dy)` from the anchor, or `(x, y)` for absolute obstacles.
    pub blocked: Vec<(i8, i8)>,
    /// Fixed position, at most once per battle.
    pub absolute: bool,
}

/// Most hexes a usual (not absolute) obstacle type blocks (BAT-GEO-004): the battle keeps a
/// placed usual obstacle as a removable piece of at most so many hexes (BAT-GEO-003, BAT-MAG-034;
/// the original's usual obstacles block up to eight).
pub const MAX_USUAL_OBSTACLE_HEXES: usize = 8;

/// A compiled spell as the battle reads it, indexed by the core's [`SpellId`]: copies of the core
/// row's fields and what the spell does in battle (E04b). An adventure spell without a battle
/// row has the defaults: neutral, no battle effects, no lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleSpell {
    /// Key (the core row's).
    pub key: ContentKey,
    /// Display name (the core row's).
    pub name: String,
    /// Where it is used (the core row's).
    pub category: SpellCategory,
    /// Spell level (the core row's).
    pub level: u8,
    /// Schools, in pack order (the core row's).
    pub schools: Vec<SchoolId>,
    /// Mana cost by mastery: none, basic, advanced, expert (the core row's).
    pub cost: [u16; 4],
    /// Power factor (the core row's).
    pub power: u16,
    /// Base effect value by mastery (the core row's).
    pub effect: [u16; 4],
    /// Only a grant naming the spell gives it (the core row's; HER-SPL-002).
    pub granted_by_name_only: bool,
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
}

/// A spell's battle row before the core fields are copied in.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SpellRow {
    polarity: SpellPolarity,
    magic: SpellMagic,
    counters: Vec<SpellId>,
    dispels: Vec<SpellId>,
    spares: Vec<SpellId>,
    tests_as: Option<SpellId>,
    summons: Option<CreatureTypeId>,
}

impl Default for SpellRow {
    fn default() -> Self {
        Self {
            polarity: SpellPolarity::Neutral,
            magic: SpellMagic::default(),
            counters: Vec::new(),
            dispels: Vec::new(),
            spares: Vec::new(),
            tests_as: None,
            summons: None,
        }
    }
}

/// The battle context's immutable compiled ruleset part, with the core it was compiled on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleRuleset {
    core: Arc<Ruleset>,
    battlefields: Vec<Battlefield>,
    obstacles: Vec<ObstacleType>,
    creatures: Vec<CreatureType>,
    abilities: Vec<Vec<CompiledAbility>>,
    spells: Vec<BattleSpell>,
    battle: BattleParams,
    hash: u64,
}

/// A creature's battle row while the packs compile (`speed` and `flags` are copied from the core
/// at the end, after every core override).
type Row = (CreatureType, Vec<CompiledAbility>);

/// The tables being filled while the packs compile, by core handle.
struct Tables<'a> {
    battlefields: Vec<Battlefield>,
    obstacles: Vec<ObstacleType>,
    obstacle_index: BTreeMap<String, Handle<ObstacleType>>,
    creatures: Vec<Option<Row>>,
    spells: Vec<Option<SpellRow>>,
    battle: Option<(&'a BattleParamsDef, Option<SurrenderParams>)>,
}

/// What the pack being compiled may refer to: the core's content of its dependency closure.
struct Scope<'a> {
    pack: &'a str,
    closure: &'a BTreeSet<String>,
    core: &'a Ruleset,
}

impl Scope<'_> {
    /// Checks that `key` belongs to a pack in the closure and resolves it with `lookup`.
    fn resolve<T>(
        &self,
        key: &ContentKey,
        kind: &'static str,
        lookup: impl FnOnce(&str) -> Option<Handle<T>>,
    ) -> Result<Handle<T>, CompileError> {
        if !self.closure.contains(key.pack()) {
            return Err(CompileError::ReferenceOutsideDependencies {
                key: String::from(key.as_str()),
                pack: String::from(self.pack),
            });
        }
        lookup(key.as_str()).ok_or_else(|| CompileError::UnknownReference {
            kind,
            key: String::from(key.as_str()),
        })
    }

    fn creature(&self, key: &ContentKey) -> Result<CreatureTypeId, CompileError> {
        self.resolve(key, "creature", |k| self.core.creature_by_key(k))
    }

    fn spell(&self, key: &ContentKey) -> Result<SpellId, CompileError> {
        self.resolve(key, "spell", |k| self.core.spell_by_key(k))
    }

    /// A battlefield of the closure, among those compiled so far (a pack's own come first).
    fn battlefield(
        &self,
        key: &ContentKey,
        compiled: &[Battlefield],
    ) -> Result<BattlefieldId, CompileError> {
        self.resolve(key, "battlefield", |k| {
            find(compiled.iter().map(|b| &b.key), k)
        })
    }

    fn school(&self, key: &ContentKey) -> Result<SchoolId, CompileError> {
        self.resolve(key, "spell school", |k| {
            find(self.core.spell_schools().iter().map(|s| &s.key), k)
        })
    }

    /// Resolves a list of spells that must be distinct.
    fn spells(&self, keys: &[ContentKey]) -> Result<Vec<SpellId>, CompileError> {
        let mut out: Vec<SpellId> = Vec::with_capacity(keys.len());
        for key in keys {
            let id = self.spell(key)?;
            if out.contains(&id) {
                return Err(CompileError::DuplicateKey(String::from(key.as_str())));
            }
            out.push(id);
        }
        Ok(out)
    }

    /// Resolves the references of an ability list.
    fn abilities(&self, defs: &[AbilityDef]) -> Result<Vec<CompiledAbility>, CompileError> {
        defs.iter()
            .map(|a| {
                a.try_map(
                    |key| self.creature(key),
                    |key| self.spell(key),
                    |key| self.school(key),
                )
            })
            .collect()
    }
}

/// The handle of the entry with key `key` among `keys` (in handle order).
fn find<'k, T>(keys: impl Iterator<Item = &'k ContentKey>, key: &str) -> Option<Handle<T>> {
    keys.into_iter()
        .position(|k| k.as_str() == key)
        .and_then(Handle::from_index)
}

impl BattleRuleset {
    /// Compiles one battle pack per pack of `core`, in the core's load order, using the engine's
    /// quirk registry (the one [`Ruleset::compile`] uses).
    pub fn compile(core: Arc<Ruleset>, packs: &[BattlePack]) -> Result<Self, BattleCompileError> {
        Self::compile_with(core, packs, &REGISTRY)
    }

    /// Like [`BattleRuleset::compile`] with an explicit quirk registry, the one `core` was
    /// compiled with (used by tests).
    pub fn compile_with(
        core: Arc<Ruleset>,
        packs: &[BattlePack],
        registry: &QuirkRegistry,
    ) -> Result<Self, BattleCompileError> {
        let order = load_order(&core, packs)?;
        let mut tables = Tables {
            battlefields: Vec::new(),
            obstacles: Vec::new(),
            obstacle_index: BTreeMap::new(),
            creatures: (0..core.creature_count()).map(|_| None).collect(),
            spells: (0..core.spells().len()).map(|_| None).collect(),
            battle: None,
        };
        for (pack, closure) in &order {
            let scope = Scope {
                pack: pack.manifest.id.as_str(),
                closure,
                core: &core,
            };
            compile_pack(pack, &scope, registry, &mut tables)?;
        }
        let rows = creature_rows(&core);
        let keys: Vec<&ContentKey> = rows.iter().map(|&(key, _)| key).collect();
        let mut creatures = Vec::with_capacity(rows.len());
        let mut abilities = Vec::with_capacity(rows.len());
        for (row, &(key, info)) in tables.creatures.into_iter().zip(&rows) {
            let (mut stats, abils) =
                row.ok_or_else(|| BattleCompileError::MissingCreature(String::from(key.as_str())))?;
            stats.speed = info.speed;
            stats.flags = info.flags;
            creatures.push(stats);
            abilities.push(abils);
        }
        let mut spells = Vec::with_capacity(tables.spells.len());
        for (row, s) in tables.spells.into_iter().zip(core.spells()) {
            let row = match row {
                Some(row) => row,
                None if s.category == SpellCategory::Adventure => SpellRow::default(),
                None => {
                    return Err(BattleCompileError::MissingSpell(String::from(
                        s.key.as_str(),
                    )));
                }
            };
            spells.push(BattleSpell {
                key: s.key.clone(),
                name: s.name.clone(),
                category: s.category,
                level: s.level,
                schools: s.schools.clone(),
                cost: s.cost,
                power: s.power,
                effect: s.effect,
                granted_by_name_only: s.granted_by_name_only,
                polarity: row.polarity,
                magic: row.magic,
                counters: row.counters,
                dispels: row.dispels,
                spares: row.spares,
                tests_as: row.tests_as,
                summons: row.summons,
            });
        }
        for ((stats, abils), key) in creatures.iter().zip(&abilities).zip(&keys) {
            validate_creature(key, stats, abils)?;
        }
        let (def, surrender) = tables
            .battle
            .ok_or(BattleCompileError::MissingBattleParams)?;
        let battle = params::compile_battle(def, surrender)?;
        validate_spells(&spells)?;
        for h in core.hero_types() {
            if h.army.len() > usize::from(battle.army_slots) {
                return Err(CompileError::InvalidContent {
                    kind: "hero type",
                    key: String::from(h.key.as_str()),
                    reason: "more starting stacks than army slots",
                }
                .into());
            }
        }
        for (abils, key) in abilities.iter().zip(&keys) {
            validate_strike_spells(key, abils, &spells)?;
        }
        validate_raised(&creatures, &abilities, &keys)?;
        for o in &tables.obstacles {
            validate_obstacle(o, &battle)?;
        }
        for b in &tables.battlefields {
            battlefield::validate(b, &battle)?;
        }
        // Battle modifier pools are sized by the number of distinct aura values (ADR-0012 §3).
        let auras: BTreeSet<u32> = abilities
            .iter()
            .flatten()
            .filter_map(Ability::aura_key)
            .collect();
        if auras.len() > MAX_AURA_VALUES {
            return Err(CompileError::TooManyEntries("distinct aura").into());
        }
        let mut part = Self {
            core,
            battlefields: tables.battlefields,
            obstacles: tables.obstacles,
            creatures,
            abilities,
            spells,
            battle,
            hash: 0,
        };
        part.hash = h3_kernel::hash::hash_of(&part);
        Ok(part)
    }

    /// The core ruleset this part was compiled on.
    pub const fn core(&self) -> &Arc<Ruleset> {
        &self.core
    }

    /// The content hash of the core ruleset this part was compiled on; a battle bundles the part
    /// only with that ruleset.
    pub fn core_hash(&self) -> u64 {
        self.core.content_hash()
    }

    /// Canonical hash of the compiled part (recorded in replays beside the core's); it shadows
    /// the core's [`Ruleset::content_hash`].
    pub const fn content_hash(&self) -> u64 {
        self.hash
    }

    /// The battle row of the creature type behind a handle, if it belongs to this ruleset (it
    /// shadows the core's [`Ruleset::creature`]).
    pub fn creature(&self, id: CreatureTypeId) -> Option<&CreatureType> {
        self.creatures.get(id.index())
    }

    /// The abilities of a creature type (empty for unknown handles), in pack order.
    pub fn abilities(&self, id: CreatureTypeId) -> &[CompiledAbility] {
        self.abilities.get(id.index()).map_or(&[], Vec::as_slice)
    }

    /// Spells as the battle reads them, in the core's load order (a [`SpellId`] indexes this
    /// slice; it shadows the core's [`Ruleset::spells`]).
    pub fn spells(&self) -> &[BattleSpell] {
        &self.spells
    }

    /// A spell as the battle reads it (it shadows the core's [`Ruleset::spell`]).
    pub fn spell(&self, id: SpellId) -> Option<&BattleSpell> {
        self.spells.get(id.index())
    }

    /// Special battlefields, in load order (a [`BattlefieldId`] indexes this slice).
    pub fn battlefields(&self) -> &[Battlefield] {
        &self.battlefields
    }

    /// A special battlefield.
    pub fn battlefield(&self, id: BattlefieldId) -> Option<&Battlefield> {
        self.battlefields.get(id.index())
    }

    /// The special battlefield with key `key` (`pack:name`).
    pub fn battlefield_by_key(&self, key: &str) -> Option<BattlefieldId> {
        find(self.battlefields.iter().map(|b| &b.key), key)
    }

    /// Obstacle types, in load order (an [`ObstacleTypeId`] indexes this slice).
    pub fn obstacles(&self) -> &[ObstacleType] {
        &self.obstacles
    }

    /// An obstacle type.
    pub fn obstacle(&self, id: ObstacleTypeId) -> Option<&ObstacleType> {
        self.obstacles.get(id.index())
    }

    /// The obstacle type with key `key` (`pack:name`).
    pub fn obstacle_by_key(&self, key: &str) -> Option<ObstacleTypeId> {
        self.obstacles
            .iter()
            .position(|o| o.key.as_str() == key)
            .and_then(ObstacleTypeId::from_index)
    }

    /// Battle parameters.
    pub const fn battle(&self) -> &BattleParams {
        &self.battle
    }
}

impl Deref for BattleRuleset {
    type Target = Ruleset;

    fn deref(&self) -> &Ruleset {
        &self.core
    }
}

impl CanonicalHash for BattleRuleset {
    /// Encoding v2 (E04c T2b; v1: R01, the content split, without battlefields): version; the
    /// special battlefields (`Battlefield`'s encoding); obstacles (key, terrains as length +
    /// handles, battlefields as length + handles, width, height, blocked pairs, absolute flag); creatures in handle order (attack, defense, damage min and max, hit points, shots,
    /// casts, the abilities as length + each ability's encoding); the battle parameters in field
    /// order (each roll table as length + all 8 entries, the war machine positions, the full
    /// formation tables loose then tight, the tactics zone widths, the surrender as presence
    /// flag + resource handle + percent, casts per round, side stack limit); spells in handle
    /// order (polarity, battle magic, counters, dispels, spares, `tests_as`, `summons`). Neither
    /// the core's hash nor the copies of core fields (speed, flags, the spells' data rows) are
    /// part of it: replays record both hashes.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u32(BATTLE_RULESET_HASH_VERSION);
        h.write_len(self.battlefields.len());
        for b in &self.battlefields {
            b.hash_into(h);
        }
        h.write_len(self.obstacles.len());
        for o in &self.obstacles {
            o.key.as_str().hash_into(h);
            h.write_len(o.terrains.len());
            for t in &o.terrains {
                t.hash_into(h);
            }
            h.write_len(o.battlefields.len());
            for b in &o.battlefields {
                b.hash_into(h);
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
        for (c, abils) in self.creatures.iter().zip(&self.abilities) {
            h.write_u16(c.attack);
            h.write_u16(c.defense);
            h.write_u16(c.damage_min);
            h.write_u16(c.damage_max);
            h.write_u32(c.hit_points);
            h.write_u16(c.shots);
            h.write_u8(c.casts);
            h.write_len(abils.len());
            for a in abils {
                a.hash_into(h);
            }
        }
        self.battle.hash_into(h);
        h.write_len(self.spells.len());
        for s in &self.spells {
            h.write_u8(s.polarity as u8);
            s.magic.hash_into(h);
            s.counters.hash_into(h);
            s.dispels.hash_into(h);
            s.spares.hash_into(h);
            s.tests_as.hash_into(h);
            s.summons.hash_into(h);
        }
    }
}

/// The core's creature keys and rows, in handle order.
fn creature_rows(core: &Ruleset) -> Vec<(&ContentKey, &CreatureInfo)> {
    (0..core.creature_count())
        .filter_map(CreatureTypeId::from_index)
        .filter_map(|id| core.creature_key(id).zip(core.creature(id)))
        .collect()
}

/// Pairs each core pack, in the core's load order, with its battle pack and dependency closure
/// (itself included).
fn load_order<'a>(
    core: &Ruleset,
    packs: &'a [BattlePack],
) -> Result<Vec<(&'a BattlePack, BTreeSet<String>)>, BattleCompileError> {
    let mut by_id: BTreeMap<&str, &BattlePack> = BTreeMap::new();
    for pack in packs {
        let id = pack.manifest.id.as_str();
        let known = core
            .packs()
            .iter()
            .any(|(core_id, version)| core_id == id && *version == pack.manifest.version);
        if !known {
            return Err(BattleCompileError::UnknownPack(format!(
                "{id}@{}",
                pack.manifest.version
            )));
        }
        if by_id.insert(id, pack).is_some() {
            return Err(BattleCompileError::DuplicatePack(String::from(id)));
        }
    }
    let mut closures: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    let mut order = Vec::new();
    for (id, _) in core.packs() {
        let pack = *by_id
            .get(id.as_str())
            .ok_or_else(|| BattleCompileError::MissingPack(id.clone()))?;
        let mut closure = BTreeSet::from([id.clone()]);
        for dependency in &pack.manifest.depends {
            let inner = closures.get(dependency.as_str()).ok_or_else(|| {
                BattleCompileError::UnknownDependency {
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

/// Checks that a pack defines `key` in its own namespace.
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

/// Puts `value` at `id`'s position of `table`, refusing a second one.
fn place<T, V>(
    table: &mut [Option<V>],
    id: Handle<T>,
    key: &ContentKey,
    value: V,
) -> Result<(), CompileError> {
    match table.get_mut(id.index()) {
        Some(slot @ None) => {
            *slot = Some(value);
            Ok(())
        }
        _ => Err(CompileError::DuplicateKey(String::from(key.as_str()))),
    }
}

/// Adds one pack's definitions to the tables.
fn compile_pack<'a>(
    pack: &'a BattlePack,
    scope: &Scope<'_>,
    registry: &QuirkRegistry,
    tables: &mut Tables<'a>,
) -> Result<(), BattleCompileError> {
    let id = scope.pack;
    for def in &pack.battlefields {
        check_namespace(&def.key, id)?;
        if find::<Battlefield>(tables.battlefields.iter().map(|b| &b.key), def.key.as_str())
            .is_some()
        {
            return Err(CompileError::DuplicateKey(String::from(def.key.as_str())).into());
        }
        let b = battlefield::compile(
            def,
            |k| scope.resolve(k, "faction", |k| scope.core.faction_by_key(k)),
            |k| scope.school(k),
        )?;
        tables.battlefields.push(b);
    }
    for def in &pack.obstacles {
        check_namespace(&def.key, id)?;
        let handle = Handle::<ObstacleType>::from_index(tables.obstacles.len())
            .ok_or(CompileError::TooManyEntries("obstacle"))?;
        if tables
            .obstacle_index
            .insert(String::from(def.key.as_str()), handle)
            .is_some()
        {
            return Err(CompileError::DuplicateKey(String::from(def.key.as_str())).into());
        }
        let terrains = def
            .terrains
            .iter()
            .map(|t| scope.resolve(t, "terrain", |k| scope.core.terrain_by_key(k)))
            .collect::<Result<_, _>>()?;
        let battlefields = def
            .battlefields
            .iter()
            .map(|b| scope.battlefield(b, &tables.battlefields))
            .collect::<Result<_, _>>()?;
        tables.obstacles.push(ObstacleType {
            key: def.key.clone(),
            terrains,
            battlefields,
            width: def.width,
            height: def.height,
            blocked: def.blocked.clone(),
            absolute: def.absolute,
        });
    }
    // Abilities may refer to any creature or spell of the closure (the core knows them all,
    // later ones of this pack included).
    for def in &pack.creatures {
        check_namespace(&def.key, id)?;
        let handle = scope.creature(&def.key)?;
        let abils = scope.abilities(&def.abilities)?;
        let stats = CreatureType {
            attack: def.attack,
            defense: def.defense,
            damage_min: def.damage_min,
            damage_max: def.damage_max,
            hit_points: def.hit_points,
            speed: 0,
            shots: def.shots,
            casts: def.casts,
            flags: CreatureFlags::default(),
        };
        place(&mut tables.creatures, handle, &def.key, (stats, abils))?;
    }
    for def in &pack.spells {
        check_namespace(&def.key, id)?;
        let handle = scope.spell(&def.key)?;
        let row = spell_row(def, scope)?;
        place(&mut tables.spells, handle, &def.key, row)?;
    }
    for o in &pack.creature_overrides {
        let handle = scope.creature(&o.target)?;
        let apply = override_applies(o.fixes.as_deref(), registry, scope.core.quirks())?;
        let (stats, abils) = tables
            .creatures
            .get_mut(handle.index())
            .and_then(Option::as_mut)
            .ok_or_else(|| BattleCompileError::MissingCreature(String::from(o.target.as_str())))?;
        // The patch goes onto a copy: a gated override's references are resolved in every
        // profile, and it is kept (and then validated with the creature) only where it applies.
        let (mut s, mut a) = (*stats, abils.clone());
        apply_patch(&o.patch, &mut s, &mut a, scope)?;
        if apply {
            (*stats, *abils) = (s, a);
        }
    }
    if let Some(b) = &pack.battle {
        if tables.battle.is_some() {
            return Err(BattleCompileError::DuplicateBattleParams(String::from(id)));
        }
        // rule: BAT-FLE-002
        let surrender = b
            .surrender
            .as_ref()
            .map(|s| {
                scope
                    .resolve(&s.resource, "resource", |k| {
                        find(scope.core.resources().iter().map(|r| &r.key), k)
                    })
                    .map(|resource| SurrenderParams {
                        resource,
                        cost_percent: s.cost_percent,
                    })
            })
            .transpose()?;
        tables.battle = Some((b, surrender));
    }
    Ok(())
}

/// Resolves a spell's battle row (its lists may name a later spell of the same pack: the core
/// knows every spell).
fn spell_row(def: &BattleSpellDef, scope: &Scope<'_>) -> Result<SpellRow, CompileError> {
    Ok(SpellRow {
        polarity: def.polarity,
        magic: def.magic.clone(),
        summons: def
            .summons
            .as_ref()
            .map(|c| scope.creature(c))
            .transpose()?,
        counters: scope.spells(&def.counters)?,
        dispels: scope.spells(&def.dispels)?,
        spares: scope.spells(&def.spares)?,
        tests_as: def.tests_as.as_ref().map(|t| scope.spell(t)).transpose()?,
    })
}

fn apply_patch(
    p: &BattleCreaturePatch,
    stats: &mut CreatureType,
    abils: &mut Vec<CompiledAbility>,
    scope: &Scope<'_>,
) -> Result<(), CompileError> {
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
    set(&mut stats.shots, p.shots.as_ref());
    set(&mut stats.casts, p.casts.as_ref());
    if let Some(defs) = &p.abilities {
        *abils = scope.abilities(defs)?;
    }
    Ok(())
}

/// Checks a creature's battle row (its core row is the core's to check).
fn validate_creature(
    key: &ContentKey,
    def: &CreatureType,
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
    // Only war machines act automatically; an army stack with these would never act.
    let automatic = abils
        .iter()
        .any(|a| matches!(a, Ability::AmmoSupply | Ability::FirstAid { .. }));
    if automatic && !machine {
        return invalid("ammo_supply and first_aid require flag war_machine");
    }
    Ok(())
}

fn invalid_spell(key: &ContentKey, reason: &'static str) -> CompileError {
    CompileError::InvalidContent {
        kind: "spell",
        key: String::from(key.as_str()),
        reason,
    }
}

/// Checks the spells' battle rows once all packs are loaded.
fn validate_spells(spells: &[BattleSpell]) -> Result<(), CompileError> {
    for (i, s) in spells.iter().enumerate() {
        if let Some(reason) = s.magic.error(s.category != SpellCategory::Adventure) {
            return Err(invalid_spell(&s.key, reason));
        }
        if !s.counters.is_empty()
            && (!s.magic.is_timed() || s.counters.iter().any(|c| c.index() == i))
        {
            return Err(invalid_spell(
                &s.key,
                "counters need a timed effect and may not name the spell itself",
            ));
        }
        let lists = (s.dispels.is_empty(), s.spares.is_empty());
        if lists != (true, true) && (!s.magic.dispels() || lists == (false, false)) {
            return Err(invalid_spell(
                &s.key,
                "dispels or spares (not both) need a dispel effect",
            ));
        }
        let raise_test = s
            .tests_as
            .and_then(|t| spells.get(t.index()))
            .is_none_or(|t| t.magic.raises());
        if s.tests_as.is_some() != s.magic.sacrifices() || !raise_test {
            return Err(invalid_spell(
                &s.key,
                "a sacrifice effect needs tests_as, a spell with a raise effect",
            ));
        }
        if s.summons.is_some() != s.magic.summons() {
            return Err(invalid_spell(
                &s.key,
                "a summon effect and the creature it summons go together",
            ));
        }
    }
    if spells.iter().filter(|s| s.summons.is_some()).count() > MAX_SUMMON_SPELLS {
        return Err(CompileError::TooManyEntries("summon spell"));
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
    spells: &[BattleSpell],
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

/// The creatures `corpse_raiser` abilities raise (BAT-MAG-052): one-hex creatures, at most
/// [`ability::MAX_RAISED_CREATURES`] of them.
fn validate_raised(
    creatures: &[CreatureType],
    abilities: &[Vec<CompiledAbility>],
    keys: &[&ContentKey],
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

/// Checks an obstacle type against the battlefield: at least one terrain or special battlefield
/// and one blocked hex;
/// usual obstacles need a width and height of at least 1, blocked hexes no lower than the
/// anchor (`dy <= 0`) and at most [`MAX_USUAL_OBSTACLE_HEXES`] of them; absolute obstacles block
/// standable hexes of the field.
fn validate_obstacle(o: &ObstacleType, battle: &BattleParams) -> Result<(), BattleCompileError> {
    let invalid = |reason| {
        Err(BattleCompileError::InvalidObstacle {
            key: String::from(o.key.as_str()),
            reason,
        })
    };
    if (o.terrains.is_empty() && o.battlefields.is_empty()) || o.blocked.is_empty() {
        return invalid("an obstacle needs a terrain or a battlefield and one blocked hex");
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
    } else if o.blocked.len() > MAX_USUAL_OBSTACLE_HEXES {
        // The battle would refuse the battlefields generated with it (E04b T11b review).
        return invalid("usual obstacles block at most 8 hexes");
    }
    Ok(())
}

#[cfg(test)]
mod tests;
