//! The town ruleset part (ADR-0028 point 5): compiled on the core as the adventure's and the
//! battle's parts are (ADR-0015 point 2), from the enabled packs' `towns.toml` (parsed by
//! `h3-pack-towns`, which documents the file's format). Data only: the rules that read it (the
//! setup's buildings and garrison, the day's income, the siege's facts, capture) come with E09
//! T6 and later (E09 increment 1's design note §5, §6).
//!
//! A part references core entities by handle (a faction by its [`FactionId`]) and its source by
//! key; a building is named by its key within its faction's record and compiled to its
//! [`BuildingId`]. Every enabled core pack has exactly one town pack (an empty one when the pack
//! has no `towns.toml`), so the load order and the dependency closures are the core's.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use h3_content::{ContentKey, CreatureTypeId, FactionId, PackManifest, ResourceId, Ruleset};
use h3_contracts::Moat;
use h3_kernel::hash::hash_of;
use h3_kernel::{CanonicalHash, StateHasher};

use crate::TOWN_RULESET_HASH_VERSION;
use crate::building::{BuildingId, BuildingSet};

mod compile;

/// The levels of a faction's creatures a town record lists, from level 1: the original's seven.
pub const MAX_CREATURE_LEVELS: usize = 7;

/// The map editor's building slots (the h3m's 41 bits per set; E09 survey §5.3): a building's
/// [`BuildingDef::slot`] is below this.
pub const BUILDING_SLOTS: u8 = 41;

/// One pack's town definitions (its `towns.toml`), with the manifest of its core pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TownPack {
    /// The core pack's manifest (identity and dependencies).
    pub manifest: PackManifest,
    /// The part's parameters, if this pack provides them.
    pub params: Option<TownParamsDef>,
    /// The town records this pack defines, one per faction.
    pub towns: Vec<TownDef>,
    /// Changes to town records of packs in its dependency closure (ADR-0006's overrides).
    pub overrides: Vec<TownOverride>,
    /// The creatures this pack names as giving their hero Visions at its expert level, for the
    /// towns (TWN-VEW-003; the AB pack's Rogue, the original's creature 143 in
    /// `hero_visions_level` 0x4E5DE0): creatures of its dependency closure, each named by one
    /// pack at most.
    pub visions: Vec<ContentKey>,
}

impl TownPack {
    /// An empty town pack for the core pack with this manifest.
    pub fn new(manifest: PackManifest) -> Self {
        Self {
            manifest,
            params: None,
            towns: Vec::new(),
            overrides: Vec::new(),
            visions: Vec::new(),
        }
    }
}

/// The part's parameters in a pack: exactly one enabled pack provides them when any pack defines
/// a town record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TownParamsDef {
    /// The core resource the halls and the Grail pay (gold).
    pub gold: ContentKey,
    /// A day's gold by the highest standing hall, levels 1 to 4 (TWN-INC-001).
    pub hall_gold: [u32; 4],
    /// A day's gold more with a building of role `grail` ever built (TWN-INC-001).
    pub grail_gold: u32,
    /// The tiles revealed around a town at the start and at a capture, without a building's
    /// larger [`BuildingRoles::reveal`] (TWN-CAP-001).
    pub reveal_radius: u8,
    /// The chance in 100 that an owned town of the standard set with a Fort starts with its
    /// level-2 dwelling (TWN-SET-003; 0x5C0C5B).
    pub level_2_dwelling_chance: u8,
    /// A neutral town's drawn garrison, levels 1 to 4 (TWN-SET-004; 0x688E84 to 0x688EA4).
    pub neutral_garrison: [GarrisonRow; 4],
    /// The key of the hall-3 building every faction's record has, which a captured Capitol
    /// becomes when its new owner holds another (TWN-CAP-002).
    pub capitol_demoted_to: String,
    /// Visions' factor at its expert level (TWN-VEW-003): a hero with a creature of
    /// [`TownPack::visions`] in its army sees another team's town at the top level within this
    /// many tiles per point of its Power (the original's spell 2's expert effect, read by
    /// `hero_visions_reach` 0x4E5E10; a town part parameter until the adventure map places
    /// Visions, E09 decision 31).
    pub visions_factor: u8,
    /// The least radius of that Visions (TWN-VEW-003; 0x4E5E6F).
    pub visions_min_radius: u8,
}

/// A level's row of a neutral town's drawn garrison (TWN-SET-004): the stack is there with
/// `chance` in 100, its count drawn from `count`'s bounds (inclusive).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GarrisonRow {
    /// The chance in 100 of a stack of the level.
    pub chance: u8,
    /// The stack's least and most creatures, at least 1 and ordered.
    pub count: [u16; 2],
}

/// A faction's town record in a pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TownDef {
    /// The core faction.
    pub faction: ContentKey,
    /// The moat's kind from a Citadel (ADR-0023 point 2's amendment; a Tower's land mines).
    pub moat: Moat,
    /// Whether a walled siege of the town draws the field's natural obstacles (TWN-SIE-005).
    pub siege_obstacles: bool,
    /// The core resources a standing Resource Silo pays a day (TWN-INC-002; 0x688EB4).
    pub silo: Vec<(ContentKey, u32)>,
    /// The faction's creatures by level, from level 1: one to [`MAX_CREATURE_LEVELS`] levels.
    pub creatures: Vec<CreatureLevelDef>,
    /// Every building of the faction, roles or not: a custom set may hold any (E09 survey §5.3),
    /// and the siege's turrets count them all.
    pub buildings: Vec<BuildingDef>,
}

/// A level's creatures in a pack: the faction's creature of that level and its upgrade.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureLevelDef {
    /// A core creature of the faction at the level.
    pub base: ContentKey,
    /// One of `base`'s upgrades, if the level has one.
    pub upgrade: Option<ContentKey>,
}

/// A building of a faction's town record in a pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildingDef {
    /// Unique within the faction's record (`[a-z0-9_]+`).
    pub key: String,
    /// Display name (from the original's text files, the name only: Q13).
    pub name: String,
    /// The map editor's building slot, below [`BUILDING_SLOTS`], if it has one.
    pub slot: Option<u8>,
    /// The buildings of the record it replaces when built (direct; the compiler closes them).
    pub supersedes: Vec<String>,
    /// The building forbidden whenever this one's key is (a horde's upgrade with its horde; E09
    /// survey §5.3), if any.
    pub forbidden_with: Option<String>,
    /// The buildings whose being ever built builds this one (an upgraded horde: its horde and
    /// the level's upgraded dwelling, as the original's `town_build`; E09 survey §16.2, decision
    /// 29); empty for a building only built itself.
    pub auto_with: Vec<String>,
    /// The buildings of the record it requires (direct, in the executable's list order; the
    /// compiler closes them, E09 design §24); empty for none.
    pub requires: Vec<String>,
    /// What it costs: core resources, each once, each amount at least 1; empty for free.
    pub cost: Vec<(ContentKey, u32)>,
    /// What the rules read of it.
    pub roles: BuildingRoles,
}

/// What the rules read of a building, each optional (increment 1's roles, E09 increment 1's
/// design note §2; increment 2's `shipyard`, design §24). Each is read over one of a town's two sets, the buildings standing (built
/// and not superseded) or the buildings ever built, as `h3-pack-towns`' format doc lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BuildingRoles {
    /// The hall level it gives, 1 (Village Hall) to 4 (Capitol).
    pub hall: Option<u8>,
    /// The fort level it gives, 1 (Fort) to 3 (Castle).
    pub fort: Option<u8>,
    /// It counts for a viewer's Tavern level (the Tavern, Castle's Brotherhood of the Sword).
    pub tavern: bool,
    /// The Resource Silo: the record's silo resources a day.
    pub silo: bool,
    /// The Grail building: the Grail's gold a day.
    pub grail: bool,
    /// The Shipyard: at most one building of a record, the one that needs water (TWN-SET-011,
    /// E09 decision 46; the original's building 6).
    pub shipyard: bool,
    /// The creature dwelling of a level, if it is one.
    pub dwelling: Option<Dwelling>,
    /// The defender's morale in a siege, -3 to 3 (the Tavern, the Brotherhood of the Sword).
    pub morale: i8,
    /// The defender's luck in a siege, -3 to 3 (the Fountain of Fortune).
    pub luck: i8,
    /// The Escape Tunnel: the defender may retreat.
    pub escape_tunnel: bool,
    /// The defending hero's bonuses (ADR-0023 point 3).
    pub defender: DefenderBonus,
    /// A reveal larger than the part's radius (the Lookout Tower, a Grail), if any.
    pub reveal: Option<Reveal>,
}

/// A dwelling's level and whether it is the upgraded one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dwelling {
    /// The creature level, from 1, within the record's levels.
    pub level: u8,
    /// The upgraded dwelling: the level's upgrade grows there.
    pub upgraded: bool,
}

/// The defending hero's bonuses a building gives in a siege (ADR-0023 point 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DefenderBonus {
    /// Attack.
    pub attack: u8,
    /// Defense.
    pub defense: u8,
    /// Spell power.
    pub power: u8,
    /// Spell points.
    pub mana: u16,
}

/// A building's reveal at the start and at a capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reveal {
    /// The tiles within this radius of the town.
    Radius(u8),
    /// The whole map (a Grail's).
    All,
}

/// A partial change to a faction's town record of a pack in the overriding pack's dependency
/// closure (ADR-0006: only the listed fields change, applied in load order; the SoD pack's moats,
/// E09 decision 20).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TownOverride {
    /// The core faction whose record changes.
    pub faction: ContentKey,
    /// The fields to change.
    pub patch: TownPatch,
}

/// The changed fields of a [`TownOverride`]. `None` fields stay as they are; the silo replaces
/// the whole list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TownPatch {
    /// New moat.
    pub moat: Option<Moat>,
    /// New natural obstacles in a walled siege.
    pub siege_obstacles: Option<bool>,
    /// New silo resources.
    pub silo: Option<Vec<(ContentKey, u32)>>,
}

/// Why the town packs could not be compiled into a [`TownRuleset`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TownCompileError {
    /// A core pack has no town pack.
    MissingPack(String),
    /// A town pack's manifest names no pack of the core ruleset (id and version).
    UnknownPack(String),
    /// Two town packs belong to one core pack.
    DuplicatePack(String),
    /// A pack's dependency does not load before it.
    UnknownDependency {
        /// The depending pack.
        pack: String,
        /// The dependency.
        dependency: String,
    },
    /// A pack references a key, or overrides a record, of a pack outside its dependency closure.
    ReferenceOutsideDependencies {
        /// The referenced key (a faction's for a record).
        key: String,
        /// The referencing pack.
        pack: String,
    },
    /// A faction the core ruleset does not define.
    UnknownFaction(String),
    /// A resource the core ruleset does not define.
    UnknownResource(String),
    /// A creature the core ruleset does not define.
    UnknownCreature(String),
    /// Two packs define a town record for one faction.
    DuplicateTown(String),
    /// An override names a faction without a town record.
    UnknownTown(String),
    /// A pack defines town records or Visions creatures and no enabled pack provides the
    /// parameters.
    MissingParams,
    /// A second pack provides the parameters.
    DuplicateParams(String),
    /// The parameters are invalid.
    InvalidParams(&'static str),
    /// A faction's town record is invalid.
    InvalidTown {
        /// The faction.
        faction: String,
        /// Why.
        reason: &'static str,
    },
    /// A building of a faction's record is invalid.
    InvalidBuilding {
        /// The faction.
        faction: String,
        /// The building's key as written.
        building: String,
        /// Why.
        reason: &'static str,
    },
}

impl fmt::Display for TownCompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPack(p) => write!(f, "pack `{p}` has no town part"),
            Self::UnknownPack(p) => write!(f, "town part of `{p}`, which the core lacks"),
            Self::DuplicatePack(p) => write!(f, "pack `{p}` has two town parts"),
            Self::UnknownDependency { pack, dependency } => write!(
                f,
                "pack `{pack}` depends on `{dependency}`, which is not loaded"
            ),
            Self::ReferenceOutsideDependencies { key, pack } => write!(
                f,
                "pack `{pack}` references `{key}` outside its dependencies"
            ),
            Self::UnknownFaction(k) => write!(f, "unknown faction `{k}`"),
            Self::UnknownResource(k) => write!(f, "unknown resource `{k}`"),
            Self::UnknownCreature(k) => write!(f, "unknown creature `{k}`"),
            Self::DuplicateTown(k) => write!(f, "faction `{k}` has two town records"),
            Self::UnknownTown(k) => write!(f, "faction `{k}` has no town record to override"),
            Self::MissingParams => f.write_str("no pack provides the town parameters"),
            Self::DuplicateParams(p) => {
                write!(f, "pack `{p}` provides the town parameters a second time")
            }
            Self::InvalidParams(reason) => write!(f, "town parameters: {reason}"),
            Self::InvalidTown { faction, reason } => {
                write!(f, "town record of `{faction}`: {reason}")
            }
            Self::InvalidBuilding {
                faction,
                building,
                reason,
            } => write!(f, "building `{building}` of `{faction}`: {reason}"),
        }
    }
}

impl core::error::Error for TownCompileError {}

/// The compiled parameters ([`TownParamsDef`] with the gold resolved).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TownParams {
    /// The resource the halls and the Grail pay.
    pub gold: ResourceId,
    /// A day's gold by the highest standing hall, levels 1 to 4.
    pub hall_gold: [u32; 4],
    /// A day's gold more with a Grail building ever built.
    pub grail_gold: u32,
    /// The reveal's radius without a larger building's.
    pub reveal_radius: u8,
    /// The chance in 100 of an owned standard town's level-2 dwelling.
    pub level_2_dwelling_chance: u8,
    /// A neutral town's drawn garrison, levels 1 to 4.
    pub neutral_garrison: [GarrisonRow; 4],
    /// The key of the hall a demoted Capitol becomes (each record holds its building as
    /// [`TownRecord::capitol_demoted_to`]).
    pub capitol_demoted_to: String,
    /// Visions' expert factor, tiles per point of Power (TWN-VEW-003).
    pub visions_factor: u8,
    /// Visions' least radius (TWN-VEW-003).
    pub visions_min_radius: u8,
    /// The creatures that give their hero's army Visions at its expert level, every enabled
    /// pack's [`TownPack::visions`] in handle order.
    pub visions_creatures: Vec<CreatureTypeId>,
}

/// A faction's compiled town record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TownRecord {
    /// The faction.
    pub faction: FactionId,
    /// The moat's kind from a Citadel.
    pub moat: Moat,
    /// Whether a walled siege draws natural obstacles.
    pub siege_obstacles: bool,
    /// The silo's resources a day, in the core's resource order, each at least 1.
    pub silo: Vec<(ResourceId, u32)>,
    /// The creatures by level: index 0 is level 1.
    pub creatures: Vec<CreatureLevel>,
    /// The buildings, a [`BuildingId`] indexing the list.
    pub buildings: Vec<Building>,
    /// The record's hall-3 building a demoted Capitol becomes.
    pub capitol_demoted_to: BuildingId,
}

impl TownRecord {
    /// A building of the record.
    pub fn building(&self, id: BuildingId) -> Option<&Building> {
        self.buildings.get(id.index())
    }

    /// The building with this key.
    pub fn building_by_key(&self, key: &str) -> Option<BuildingId> {
        self.buildings
            .iter()
            .position(|b| b.key == key)
            .and_then(BuildingId::from_index)
    }
}

/// A level's compiled creatures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreatureLevel {
    /// The faction's creature of the level.
    pub base: CreatureTypeId,
    /// Its upgrade, if the level has one.
    pub upgrade: Option<CreatureTypeId>,
}

/// A compiled building.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Building {
    /// Its key within the record.
    pub key: String,
    /// Display name.
    pub name: String,
    /// Its map editor slot, if any.
    pub slot: Option<u8>,
    /// Every building it replaces, directly or through another (the closure; never itself).
    pub supersedes: BuildingSet,
    /// The building forbidden whenever this one is, if any (never itself, no cycle).
    pub forbidden_with: Option<BuildingId>,
    /// The buildings that, once each is ever built, make this one ever built too (never itself;
    /// empty for none; read over the ever-built set by the setup and, from increment 2,
    /// construction).
    pub auto_with: BuildingSet,
    /// Every building it requires, directly or through another (the closure, as the original's
    /// `building_requirements` 0x697798; never itself; E09 design §24).
    pub requires: BuildingSet,
    /// What it costs, in the core's resource order, each amount at least 1; empty for free.
    pub cost: Vec<(ResourceId, u32)>,
    /// What the rules read of it.
    pub roles: BuildingRoles,
}

/// The town context's immutable compiled ruleset part: the parameters and every faction's town
/// record, its own canonical hash ([`TOWN_RULESET_HASH_VERSION`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TownRuleset {
    core_hash: u64,
    params: Option<TownParams>,
    towns: Vec<TownRecord>,
    hash: u64,
}

impl TownRuleset {
    /// The part without towns on `core` (what packs without a `towns.toml` compile to; an
    /// encounter's game, which has no towns).
    pub fn empty(core: &Ruleset) -> Self {
        Self::new(core, None, Vec::new())
    }

    /// Compiles one town pack per pack of `core`, in the core's load order.
    ///
    /// # Errors
    ///
    /// A [`TownCompileError`] for the first check that fails (E09 increment 1's design note §2).
    pub fn compile(core: &Ruleset, packs: &[TownPack]) -> Result<Self, TownCompileError> {
        let (params, towns) = compile::compile(core, packs)?;
        Ok(Self::new(core, params, towns))
    }

    /// The part of `params` and `towns` (in faction order) on `core`, hashed; the derivations'
    /// tests build theirs so (`crate::fixture`).
    pub(crate) fn new(core: &Ruleset, params: Option<TownParams>, towns: Vec<TownRecord>) -> Self {
        let mut part = Self {
            core_hash: core.content_hash(),
            params,
            towns,
            hash: 0,
        };
        part.hash = hash_of(&part);
        part
    }

    /// The content hash of the core ruleset this part was compiled on; a game bundles the part
    /// only with that ruleset.
    pub const fn core_hash(&self) -> u64 {
        self.core_hash
    }

    /// Canonical hash of the compiled part (recorded in replays beside the core's).
    pub const fn content_hash(&self) -> u64 {
        self.hash
    }

    /// The parameters: present whenever a town record is.
    pub const fn params(&self) -> Option<&TownParams> {
        self.params.as_ref()
    }

    /// The town records, in the core's faction order (the factions a random town may take,
    /// ADR-0028 point 7).
    pub fn towns(&self) -> &[TownRecord] {
        &self.towns
    }

    /// The town record of `faction`, if it has one.
    pub fn town(&self, faction: FactionId) -> Option<&TownRecord> {
        self.towns
            .binary_search_by_key(&faction, |t| t.faction)
            .ok()
            .and_then(|i| self.towns.get(i))
    }
}

/// A moat's code in the hash.
fn moat_code(moat: Moat) -> u8 {
    match moat {
        Moat::Absent => 0,
        Moat::Harmless => 1,
        Moat::Damaging => 2,
    }
}

/// An optional value: 0 for none, else 1 and the value.
fn hash_option<T>(h: &mut StateHasher, value: Option<T>, write: impl FnOnce(&mut StateHasher, T)) {
    match value {
        None => h.write_u8(0),
        Some(v) => {
            h.write_u8(1);
            write(h, v);
        }
    }
}

impl CanonicalHash for TownRuleset {
    /// Encoding v4 (E09 T16a; v3, T8c2's, without the requirements, the costs and the role
    /// `shipyard`; v2, T5b2a's, without Visions; v1, T5a1's first, without the automatic set):
    /// version; the parameters (0 for none, else 1, the gold resource's handle, the four hall
    /// amounts, the Grail's, the reveal radius, the level-2 dwelling chance, the four garrison
    /// rows' chance and bounds, the demoted Capitol's key, Visions' factor and least radius, its
    /// creatures as their number and handles); the records in faction order as their number and
    /// each one's faction handle, moat (0 absent, 1 harmless, 2 damaging), natural obstacles, the
    /// silo (length, each resource handle and amount), the creatures (length, each base handle and
    /// the upgrade as an option), the buildings (length, each key, name, slot as an option, the
    /// superseded set's bits, the forbidden-with building as an option, the automatic set's bits
    /// (`auto_with`), the required set's bits (the closure), the cost (length, each resource
    /// handle and amount), then the roles: hall and fort as options, tavern, silo, grail,
    /// shipyard, the dwelling as an option of its level and upgrade flag, morale, luck, the escape tunnel, the defender's
    /// attack, defense, power and mana, the reveal as 0 none, 1 and the radius, 2 all) and the
    /// demoted Capitol's building. The core's hash is not part of it: replays record both.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u32(TOWN_RULESET_HASH_VERSION);
        hash_option(h, self.params.as_ref(), |h, p| {
            p.gold.hash_into(h);
            p.hall_gold.iter().for_each(|&g| h.write_u32(g));
            h.write_u32(p.grail_gold);
            h.write_u8(p.reveal_radius);
            h.write_u8(p.level_2_dwelling_chance);
            for row in &p.neutral_garrison {
                h.write_u8(row.chance);
                row.count.iter().for_each(|&c| h.write_u16(c));
            }
            p.capitol_demoted_to.as_str().hash_into(h);
            h.write_u8(p.visions_factor);
            h.write_u8(p.visions_min_radius);
            h.write_len(p.visions_creatures.len());
            p.visions_creatures.iter().for_each(|c| c.hash_into(h));
        });
        h.write_len(self.towns.len());
        for t in &self.towns {
            t.faction.hash_into(h);
            h.write_u8(moat_code(t.moat));
            h.write_bool(t.siege_obstacles);
            h.write_len(t.silo.len());
            for &(resource, amount) in &t.silo {
                resource.hash_into(h);
                h.write_u32(amount);
            }
            h.write_len(t.creatures.len());
            for level in &t.creatures {
                level.base.hash_into(h);
                hash_option(h, level.upgrade, |h, u| u.hash_into(h));
            }
            h.write_len(t.buildings.len());
            for b in &t.buildings {
                b.key.as_str().hash_into(h);
                b.name.as_str().hash_into(h);
                hash_option(h, b.slot, StateHasher::write_u8);
                h.write_u64(b.supersedes.bits());
                hash_option(h, b.forbidden_with, |h, w| h.write_u8(w.raw()));
                h.write_u64(b.auto_with.bits());
                h.write_u64(b.requires.bits());
                h.write_len(b.cost.len());
                for &(resource, amount) in &b.cost {
                    resource.hash_into(h);
                    h.write_u32(amount);
                }
                hash_roles(h, &b.roles);
            }
            h.write_u8(t.capitol_demoted_to.raw());
        }
    }
}

/// A building's roles in the hash.
fn hash_roles(h: &mut StateHasher, r: &BuildingRoles) {
    hash_option(h, r.hall, StateHasher::write_u8);
    hash_option(h, r.fort, StateHasher::write_u8);
    h.write_bool(r.tavern);
    h.write_bool(r.silo);
    h.write_bool(r.grail);
    h.write_bool(r.shipyard);
    hash_option(h, r.dwelling, |h, d| {
        h.write_u8(d.level);
        h.write_bool(d.upgraded);
    });
    h.write_i32(i32::from(r.morale));
    h.write_i32(i32::from(r.luck));
    h.write_bool(r.escape_tunnel);
    h.write_u8(r.defender.attack);
    h.write_u8(r.defender.defense);
    h.write_u8(r.defender.power);
    h.write_u16(r.defender.mana);
    match r.reveal {
        None => h.write_u8(0),
        Some(Reveal::Radius(radius)) => {
            h.write_u8(1);
            h.write_u8(radius);
        }
        Some(Reveal::All) => h.write_u8(2),
    }
}

#[cfg(test)]
mod tests;
