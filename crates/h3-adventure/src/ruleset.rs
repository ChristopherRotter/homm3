//! The adventure context's ruleset part (ADR-0015 point 2, ADR-0006's E08 amendment; E08 T4):
//! what the adventure map needs beside the core ruleset, compiled from the packs'
//! `adventure.toml` files on top of a compiled core [`Ruleset`]. Data only: the rules that read
//! it (the cost of a step, a day's movement points, the reveal) come with movement (T7) and fog
//! of war (T8), so the rule ids `ADV-MOV-001`, `-003`, `-004`, `-007` and `ADV-FOG-001` stay
//! `specified` until then (`docs/rules/catalog/adventure.toml`).
//!
//! Since T12a it holds the object types (`objects.rs`, ADR-0016 point 4), since T13a the starting
//! resources by difficulty, which the game reads (`starting.rs`, GAM-RES-002), since T20a2a the
//! wandering monsters' parameters and the creatures no random monster becomes.
//!
//! A part references core entities by handle (a terrain by its [`TerrainId`]) and its source by
//! key. Every enabled core pack has exactly one adventure pack (an empty one when the pack has no
//! `adventure.toml`), so the load order and the dependency closures are the core's.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::num::{NonZeroU8, NonZeroU16, NonZeroU32};

use h3_content::{ContentKey, CreatureTypeId, PackManifest, ResourceId, Ruleset, TerrainId};
use h3_kernel::hash::hash_of;
use h3_kernel::{CanonicalHash, Handle, StateHasher};

use crate::ADVENTURE_RULESET_HASH_VERSION;

mod objects;
mod rewards;
mod starting;

pub(crate) use objects::Effect;
pub use objects::{
    KEY_COLOURS, MAX_GAIN, MAX_PICKUP_AMOUNT, ObjectKind, ObjectKindDef, ObjectType, ObjectTypeDef,
    ObjectTypeId, Pickup, PickupDef, Visit,
};
use objects::{ObjectTables, Scope, check_object_types, compile_object_types, random_monsters};
pub(crate) use rewards::Resources;
pub use rewards::{
    Chest, ChestDef, Find, FindContent, FindContentDef, FindDef, Gift, GiftDef, Gives, OnceBonus,
    Pyramid, PyramidDef, Turn, TurnDef,
};
use starting::compile_starting;
pub use starting::{MAX_DIFFICULTIES, StartingResources, StartingResourcesDef};

/// Most entries of the land movement table: one per speed a `u8` can hold.
pub const MAX_LAND_POINTS: usize = 256;

/// The largest count of a wandering monster a setup may give (ADV-MON-001): the original keeps
/// the count in the low 12 bits of the monster's tile word, so a larger count would lose its high
/// bits (survey `E08-battles.md` §1.2; an input bound, no shipped map has a count above 3 000).
pub const MAX_MONSTER_COUNT: u32 = 0xFFF;

/// The highest bound a part may set on a game's players: the original's eight colours, and the
/// players an event's [`crate::Viewers`] holds (E08 T8a).
pub const MAX_PLAYERS: u8 = 8;

/// How a hero may cross a terrain (ADV-MOV-007), with the straight cost of a step where it may
/// (ADV-MOV-003).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Passage {
    /// Walked on foot; a straight step costs this many movement points.
    Land(NonZeroU16),
    /// Sailed by boat (boats come after increment 1); a straight step costs this many points.
    Water(NonZeroU16),
    /// Never entered (rock).
    Impassable,
}

impl CanonicalHash for Passage {
    fn hash_into(&self, h: &mut StateHasher) {
        match self {
            Self::Land(cost) => {
                h.write_u8(0);
                h.write_u16(cost.get());
            }
            Self::Water(cost) => {
                h.write_u8(1);
                h.write_u16(cost.get());
            }
            Self::Impassable => h.write_u8(2),
        }
    }
}

/// A pack's adventure data for one core terrain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainMovementDef {
    /// The core terrain.
    pub terrain: ContentKey,
    /// How it is crossed.
    pub passage: Passage,
    /// A land terrain's straight step cost for a hero with Pathfinding at Basic, Advanced and
    /// Expert (the original's `step_cost_table` 0x63E510; E08 T29), each at most the one below
    /// it, the first at most the cost without; `None`: the cost without at every level. Only a
    /// land terrain may have them. Read for the map's terrain shares (ADV-MAP-002) until the
    /// hero's movement skills come (increment 5's sketch).
    pub pathfinding: Option<[NonZeroU16; 3]>,
}

/// A road kind in a pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoadDef {
    /// Namespaced key.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
    /// The straight cost of a step along it (ADV-MOV-003).
    pub cost: NonZeroU16,
}

/// A river kind in a pack (appearance only: rivers change no cost).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiverDef {
    /// Namespaced key.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
}

/// The adventure parameters in a pack; exactly one enabled pack provides them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdventureParamsDef {
    /// The straight cost of a step on terrain native to the whole army (ADV-MOV-004).
    pub native_cost: NonZeroU16,
    /// A day's movement points on land by the army's slowest speed, from speed 0; faster armies
    /// use the last entry (ADV-MOV-001). One to [`MAX_LAND_POINTS`] entries, none 0.
    pub land_points: Vec<u32>,
    /// A hero's scouting radius without bonuses (ADV-FOG-001).
    pub scouting_radius: u8,
    /// The widest and highest map in tiles (a setup bound, ADR-0015 point 6).
    pub max_map_size: NonZeroU16,
    /// The most levels of a map (surface and underground).
    pub max_levels: NonZeroU8,
    /// The most players of a game, at most [`MAX_PLAYERS`]; the adventure keeps a layer per
    /// player (fog of war).
    pub max_players: NonZeroU8,
    /// The slots of a hero's army.
    pub army_slots: NonZeroU8,
    /// The most creatures a stack holds (ADV-ARM-007, E08 decisions 109, 114): a merge past it is
    /// refused, a split keeps both stacks within it, a joining or raised group its creature's
    /// stack cannot take whole goes to a free slot. At least [`MAX_MONSTER_COUNT`], so that any
    /// wandering monster's group fits a slot.
    pub max_stack_count: NonZeroU32,
    /// The chance in percent that a hero's starting army gets the stack of its second, third, …
    /// starting slot; the first slot's stack always comes, slots past the list never
    /// (GAM-SET-001). Each at most 100.
    pub starting_stack_chances: Vec<u8>,
    /// Where a battle on the map is fought when its tile says so (GAM-BAT-001, E08 T19a); none:
    /// on the tile's terrain, without a special battlefield.
    pub battle: Option<BattleTerrainsDef>,
    /// The wandering monsters' parameters (E08 T20a2a); none: a map may hold no monster.
    pub monsters: Option<MonsterParamsDef>,
    /// A flying hero's straight step price by its flying level, 0 to 3 (ADV-MOV-009, E08 T40;
    /// read by the price from T41): the original's `fly_walk_step_cost` (0x6778AC), 140, 140,
    /// 120, 100; a higher level counts as the last.
    pub fly_cost: [NonZeroU16; 4],
    /// A water-walking hero's straight step price from water by its water-walking level, 0 to 3
    /// (ADV-MOV-010, E08 T40; read from T41): the same table in the original, a field of its own
    /// so that data can tell the two apart (VCMI's spells do).
    pub water_walk_cost: [NonZeroU16; 4],
    /// A hero's day's points at sea by its Navigation level, 0 to 3 (HER-ADV-009, E08 T72; read
    /// by `h3_heroes::sea_points`): `movement.txt`'s sea column, the original's
    /// `sea_movement_by_navigation` (0x698AEC), 1 500, 2 250, 3 000, 3 750. None 0.
    pub sea_points: [u32; 4],
    /// The share of a boat's step price from a tile a ground of the sea tops (ADV-SEA-004, E08
    /// T72; read by the prices from T73), as numerator and denominator, the price truncated:
    /// Favorable Winds' two thirds, `[2, 3]` (survey `E08-sea.md` §4.1). The numerator from 1, at
    /// most the denominator.
    pub sea_ground: [u16; 2],
    /// The calendar (GAM-CAL-002, E08 T83a; read by the game's day advance from T85b).
    pub calendar: CalendarParams,
    /// The months' kinds and their monsters (GAM-CAL-003, ADV-MON-014, E08 T83a; read by the
    /// month's step from T85a); none: every month is a normal one.
    pub months: Option<MonthParamsDef>,
}

/// The calendar (GAM-CAL-002, E08 T83a; increment 9's design note §5): a week's days and a month's
/// weeks, the original's 7 and 4 (`new_day` 0x4C7FE0; survey `E08-economy.md` §7.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalendarParams {
    /// The days of a week.
    pub week_days: NonZeroU8,
    /// The weeks of a month.
    pub month_weeks: NonZeroU8,
}

/// The months' parameters in a pack (GAM-CAL-003, ADV-MON-014, E08 T83a; survey
/// `E08-economy.md` §7.4, §10.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonthParamsDef {
    /// The weights of a normal month, a month of a creature and the Plague (the original's draw
    /// of 1 to 10: 5, 4, 1); their sum from 1.
    pub odds: [u8; 3],
    /// The creatures a month of a creature is drawn among, with equal chances (`month_creatures`
    /// 0x63E678: twelve): distinct core creatures, none a war machine, each with an adventure
    /// count from 1; at least one when a creature's month has a weight.
    pub creatures: Vec<ContentKey>,
    /// The object type a month's monster is: a `monster` kind without levels (the map names the
    /// creature), of any pack.
    pub monster: ContentKey,
    /// A free land tile gets a monster with one chance in this many (200).
    pub spawn_one_in: NonZeroU16,
    /// A month's monster's count: this times a draw from its creature's adventure count (2; D-50
    /// as read), at most [`MAX_MONSTER_COUNT`].
    pub spawn_factor: NonZeroU8,
    /// A month's monster's aggression, drawn between these, inclusive (1 to 10), low to high.
    pub spawn_aggression: [i8; 2],
}

/// The compiled [`MonthParamsDef`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonthParams {
    /// The weights of a normal month, a month of a creature and the Plague.
    pub odds: [u8; 3],
    /// The creatures of a month of a creature, in the pack's order.
    pub creatures: Vec<CreatureTypeId>,
    /// The object type of a month's monster.
    pub monster: ObjectTypeId,
    /// One chance in this many for each free land tile.
    pub spawn_one_in: NonZeroU16,
    /// The factor of a month's monster's count.
    pub spawn_factor: NonZeroU8,
    /// A month's monster's aggression bounds.
    pub spawn_aggression: [i8; 2],
}

/// The wandering monsters' parameters in a pack (E08 T20a2a).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonsterParamsDef {
    /// A monster's aggression by its disposition (the map's code, from 0): the bounds, inclusive,
    /// it is drawn between when the game is built (ADV-MON-002). At most 256 rows, each low to
    /// high.
    pub aggression: Vec<[i8; 2]>,
    /// The counts at which each word of a monster's count after the first begins, ascending from
    /// 2: what a player sees of a count is the number of them it reaches (ADV-FOG-006).
    pub count_words: Vec<u32>,
    /// The resource a group's price to join is counted and paid in: its creature's cost in it
    /// times its count (ADV-MON-008, E08 T20b1; the original's gold, creature record +0x38).
    pub price_resource: ContentKey,
    /// A monster's weekly growth in basis points of its value (ADV-MON-013, E08 T83a; read by the
    /// week's step from T85a): 1 000, a tenth, truncated; at most 10 000.
    pub growth_bp: u16,
    /// The parts of a creature a monster's value counts in (the original's sixteenths, 16; D-49,
    /// batch E9's E9-9): from 1 to 256, so that a fraction fits a byte.
    pub growth_unit: NonZeroU16,
    /// The most creatures growth gives a monster (4 000; A-32: the result capped, a larger stack
    /// the map gave becomes it), at most [`MAX_MONSTER_COUNT`].
    pub growth_cap: NonZeroU32,
}

/// The compiled [`MonsterParamsDef`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonsterParams {
    aggression: Vec<[i8; 2]>,
    count_words: Vec<u32>,
    price_resource: ResourceId,
    growth_bp: u16,
    growth_unit: NonZeroU16,
    growth_cap: NonZeroU32,
}

impl MonsterParams {
    /// The bounds of the aggression of a monster of this disposition, if the parameters know it
    /// (ADV-MON-002).
    pub fn aggression(&self, disposition: u8) -> Option<[i8; 2]> {
        self.aggression.get(usize::from(disposition)).copied()
    }

    /// The word of a monster's count that a player sees (ADV-FOG-006): how many of the words'
    /// thresholds the count reaches, 0 for the first word.
    pub fn count_word(&self, count: u32) -> u8 {
        u8::try_from(self.count_words.partition_point(|&t| t <= count)).unwrap_or(u8::MAX)
    }

    /// The resource a group's price to join is paid in (ADV-MON-008).
    pub const fn price_resource(&self) -> ResourceId {
        self.price_resource
    }

    /// A monster's weekly growth in basis points of its value (ADV-MON-013, E08 T83a).
    pub const fn growth_bp(&self) -> u16 {
        self.growth_bp
    }

    /// The parts of a creature a monster's value counts in (ADV-MON-013, E08 T83a).
    pub const fn growth_unit(&self) -> u16 {
        self.growth_unit.get()
    }

    /// The most creatures growth gives a monster (ADV-MON-013, E08 T83a).
    pub const fn growth_cap(&self) -> u32 {
        self.growth_cap.get()
    }
}

/// The terrain and battlefield of a battle whose tile is underground or on the coast, in the
/// original's order (survey `E04c-battlefields.md` §4: the underground first, then the coast).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleTerrainsDef {
    /// The core terrain of a battle on the underground (the original's subterranean).
    pub underground: ContentKey,
    /// The core terrain of a battle on a coast tile (the original's sand).
    pub coast: ContentKey,
    /// The battle part's special battlefield of a battle on a coast tile (the shore); checked
    /// against the battle part when the game bundles the parts.
    pub coast_battlefield: ContentKey,
    /// The core terrain of a battle with a hero in a boat (GAM-BAT-001, E08 T72; read since T74b,
    /// `AdventureState::sea_battle_ground`: the original's water).
    pub sea: ContentKey,
    /// The battle part's battlefield of a battle with one hero in a boat (the ship's deck), as the
    /// coast's.
    pub deck: ContentKey,
    /// The battle part's battlefield of a battle with both heroes in boats (the two boats), as the
    /// coast's.
    pub boats: ContentKey,
}

/// One pack's adventure definitions (its `adventure.toml`), with the manifest of its core pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdventurePack {
    /// The core pack's manifest (identity and dependencies).
    pub manifest: PackManifest,
    /// Movement data of core terrains.
    pub terrains: Vec<TerrainMovementDef>,
    /// Road kinds.
    pub roads: Vec<RoadDef>,
    /// River kinds.
    pub rivers: Vec<RiverDef>,
    /// Object types (E08 T12a).
    pub object_types: Vec<ObjectTypeDef>,
    /// The adventure parameters, if this pack provides them.
    pub params: Option<AdventureParamsDef>,
    /// The starting resources by difficulty, from 0 (easy), if this pack provides them (E08
    /// T13a; empty: it does not).
    pub starting_resources: Vec<StartingResourcesDef>,
    /// Core creatures of the pack's dependency closure that no random monster becomes (E08
    /// T20a2a, ADV-MON-003): the expansions' six the original leaves out.
    pub not_random_monsters: Vec<ContentKey>,
    /// Terrain walkers (E08 T40, ADV-MOV-003): Nomads' sand priced as dirt.
    pub terrain_walkers: Vec<TerrainWalkerDef>,
}

/// A terrain walker in a pack (E08 T40, ADV-MOV-003): an army with a stack of `creature` counts
/// `terrain` as `counts_as` for every price of a step from it, the native terrain's and the
/// diagonal exception's check included (the original's Nomads, creature 142: sand as dirt,
/// `step_cost` 0x4B1740; survey `E08-hero-movement.md` §2.1 point 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainWalkerDef {
    /// The core creature.
    pub creature: ContentKey,
    /// The core terrain it walks as another (a land terrain).
    pub terrain: ContentKey,
    /// The core terrain `terrain` counts as (a land terrain).
    pub counts_as: ContentKey,
}

/// A compiled [`TerrainWalkerDef`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerrainWalker {
    /// The creature.
    pub creature: CreatureTypeId,
    /// The terrain it walks as another.
    pub terrain: TerrainId,
    /// The terrain `terrain` counts as.
    pub counts_as: TerrainId,
}

impl AdventurePack {
    /// An empty adventure pack for the core pack with this manifest.
    pub fn new(manifest: PackManifest) -> Self {
        Self {
            manifest,
            terrains: Vec::new(),
            roads: Vec::new(),
            rivers: Vec::new(),
            object_types: Vec::new(),
            params: None,
            starting_resources: Vec::new(),
            not_random_monsters: Vec::new(),
            terrain_walkers: Vec::new(),
        }
    }
}

/// A compiled terrain's adventure data, indexed like the core's terrains.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainMovement {
    /// The core terrain's key.
    pub key: ContentKey,
    /// How it is crossed.
    pub passage: Passage,
    /// A land terrain's costs by Pathfinding level 1 to 3 as the pack gives them
    /// ([`TerrainMovementDef::pathfinding`]); [`AdventureRuleset::step_cost`] reads them.
    pub pathfinding: Option<[NonZeroU16; 3]>,
}

/// A compiled road kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoadKind {
    /// Key.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
    /// The straight cost of a step along it (ADV-MOV-003).
    pub cost: NonZeroU16,
}

/// Handle of a road kind in an [`AdventureRuleset`].
pub type RoadId = Handle<RoadKind>;

/// A compiled river kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiverKind {
    /// Key.
    pub key: ContentKey,
    /// Display name.
    pub name: String,
}

/// Handle of a river kind in an [`AdventureRuleset`].
pub type RiverId = Handle<RiverKind>;

/// The compiled adventure parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdventureParams {
    native_cost: NonZeroU16,
    land_points: Vec<u32>,
    scouting_radius: u8,
    max_map_size: NonZeroU16,
    max_levels: NonZeroU8,
    max_players: NonZeroU8,
    army_slots: NonZeroU8,
    max_stack_count: NonZeroU32,
    starting_stack_chances: Vec<u8>,
    battle: Option<BattleTerrains>,
    monsters: Option<MonsterParams>,
    fly_cost: [NonZeroU16; 4],
    water_walk_cost: [NonZeroU16; 4],
    sea_points: [u32; 4],
    sea_ground: [u16; 2],
    calendar: CalendarParams,
    months: Option<MonthParams>,
}

/// The compiled [`BattleTerrainsDef`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleTerrains {
    /// The terrain of a battle on the underground.
    pub underground: TerrainId,
    /// The terrain of a battle on a coast tile.
    pub coast: TerrainId,
    /// The special battlefield of a battle on a coast tile (a battle part's key).
    pub coast_battlefield: ContentKey,
    /// The terrain of a battle with a hero in a boat (E08 T72).
    pub sea: TerrainId,
    /// The battlefield of a battle with one hero in a boat (a battle part's key).
    pub deck: ContentKey,
    /// The battlefield of a battle with both heroes in boats (a battle part's key).
    pub boats: ContentKey,
}

impl AdventureParams {
    /// The straight cost of a step on terrain native to the whole army (ADV-MOV-004).
    pub const fn native_cost(&self) -> NonZeroU16 {
        self.native_cost
    }

    /// A day's movement points on land for an army whose slowest speed is `slowest_speed`; a
    /// speed beyond the table takes its last entry (ADV-MOV-001; the original caps the speed at
    /// 20, where `movement.txt` has long reached its last value).
    pub fn land_points(&self, slowest_speed: u8) -> u32 {
        let last = self.land_points.len() - 1;
        self.land_points[usize::from(slowest_speed).min(last)]
    }

    /// The table [`Self::land_points`] reads, from speed 0.
    pub fn land_points_table(&self) -> &[u32] {
        &self.land_points
    }

    /// A hero's scouting radius without bonuses (ADV-FOG-001).
    pub const fn scouting_radius(&self) -> u8 {
        self.scouting_radius
    }

    /// The widest and highest map in tiles.
    pub const fn max_map_size(&self) -> u16 {
        self.max_map_size.get()
    }

    /// The most levels of a map.
    pub const fn max_levels(&self) -> u8 {
        self.max_levels.get()
    }

    /// The most players of a game.
    pub const fn max_players(&self) -> u8 {
        self.max_players.get()
    }

    /// The slots of a hero's army.
    pub const fn army_slots(&self) -> u8 {
        self.army_slots.get()
    }

    /// The most creatures a stack holds (ADV-ARM-007).
    pub const fn max_stack_count(&self) -> u32 {
        self.max_stack_count.get()
    }

    /// The chance in percent of each starting slot after the first (GAM-SET-001).
    pub fn starting_stack_chances(&self) -> &[u8] {
        &self.starting_stack_chances
    }

    /// Where a battle on the map is fought when its tile is underground or on the coast
    /// (GAM-BAT-001), if the packs say.
    pub const fn battle_terrains(&self) -> Option<&BattleTerrains> {
        self.battle.as_ref()
    }

    /// The wandering monsters' parameters, if the packs give them (E08 T20a2a).
    pub const fn monsters(&self) -> Option<&MonsterParams> {
        self.monsters.as_ref()
    }

    /// A flying hero's straight step price at flying level `level` (ADV-MOV-009, E08 T40); a
    /// level above 3 counts as 3.
    pub fn fly_cost(&self, level: u8) -> u32 {
        u32::from(self.fly_cost[usize::from(level.min(3))].get())
    }

    /// A water-walking hero's straight step price from water at water-walking level `level`
    /// (ADV-MOV-010, E08 T40); a level above 3 counts as 3.
    pub fn water_walk_cost(&self, level: u8) -> u32 {
        u32::from(self.water_walk_cost[usize::from(level.min(3))].get())
    }

    /// The day's points at sea by Navigation level, 0 to 3 (HER-ADV-009, E08 T72): the table
    /// `h3_heroes::sea_points` reads.
    pub const fn sea_points(&self) -> &[u32; 4] {
        &self.sea_points
    }

    /// The share of a boat's step price from a tile a ground of the sea tops, as numerator and
    /// denominator (ADV-SEA-004, E08 T72).
    pub const fn sea_ground(&self) -> [u16; 2] {
        self.sea_ground
    }

    /// The calendar (GAM-CAL-002, E08 T83a).
    pub const fn calendar(&self) -> CalendarParams {
        self.calendar
    }

    /// The months' kinds and monsters, if the packs give them (GAM-CAL-003, ADV-MON-014, E08
    /// T83a); none: every month is a normal one.
    pub const fn months(&self) -> Option<&MonthParams> {
        self.months.as_ref()
    }
}

/// Why the adventure packs could not be compiled into an [`AdventureRuleset`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdventureCompileError {
    /// A core pack has no adventure pack.
    MissingPack(String),
    /// An adventure pack's manifest names no pack of the core ruleset (id and version).
    UnknownPack(String),
    /// Two adventure packs belong to one core pack.
    DuplicatePack(String),
    /// A pack's dependency does not load before it.
    UnknownDependency {
        /// The depending pack.
        pack: String,
        /// The dependency.
        dependency: String,
    },
    /// A pack defines a road, river or object type outside its own namespace.
    KeyOutsidePack {
        /// The offending key.
        key: String,
        /// The pack defining it.
        pack: String,
    },
    /// Two road kinds, two river kinds or two object types share a key.
    DuplicateKey(String),
    /// A pack references a key of a pack outside its dependency closure.
    ReferenceOutsideDependencies {
        /// The referenced key.
        key: String,
        /// The referencing pack.
        pack: String,
    },
    /// A terrain the core ruleset does not define.
    UnknownTerrain(String),
    /// A resource the core ruleset does not define.
    UnknownResource(String),
    /// A creature the core ruleset does not define.
    UnknownCreature(String),
    /// A secondary skill the core ruleset does not define (E08 T61a).
    UnknownSkill(String),
    /// An object type no pack defines.
    UnknownObjectType(String),
    /// An object type's parameters are invalid.
    InvalidObjectType {
        /// The object type.
        key: String,
        /// Why.
        reason: &'static str,
    },
    /// Two packs give one terrain its movement data.
    DuplicateTerrain(String),
    /// A core terrain has no movement data.
    MissingTerrain(String),
    /// A terrain's costs by Pathfinding level are given for a terrain not crossed on land, or
    /// one of them is above the cost below it (E08 T29).
    PathfindingCosts(String),
    /// A terrain walker's terrain or the terrain it counts as is not crossed on land, or a second
    /// walker names the same terrain (E08 T40); the payload is its terrain.
    TerrainWalker(String),
    /// No enabled pack provides the adventure parameters.
    MissingParams,
    /// A second pack provides the adventure parameters.
    DuplicateParams(String),
    /// The adventure parameters are invalid.
    InvalidParams(&'static str),
    /// No enabled pack provides the starting resources by difficulty.
    MissingDifficulties,
    /// A second pack provides the starting resources by difficulty.
    DuplicateDifficulties(String),
    /// A difficulty's starting resources are invalid.
    InvalidDifficulty {
        /// The difficulty, from 0.
        row: usize,
        /// Why.
        reason: &'static str,
    },
    /// More entries of a kind (`road`, `river`, `object_type`, `difficulty`) than handles or
    /// indices can address.
    TooManyEntries(&'static str),
}

impl fmt::Display for AdventureCompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPack(p) => write!(f, "pack `{p}` has no adventure part"),
            Self::UnknownPack(p) => write!(f, "adventure part of `{p}`, which the core lacks"),
            Self::DuplicatePack(p) => write!(f, "pack `{p}` has two adventure parts"),
            Self::UnknownDependency { pack, dependency } => {
                write!(
                    f,
                    "pack `{pack}` depends on `{dependency}`, which is not loaded"
                )
            }
            Self::KeyOutsidePack { key, pack } => {
                write!(f, "pack `{pack}` defines `{key}` outside its namespace")
            }
            Self::DuplicateKey(k) => write!(f, "duplicate key `{k}`"),
            Self::ReferenceOutsideDependencies { key, pack } => {
                write!(
                    f,
                    "pack `{pack}` references `{key}` outside its dependencies"
                )
            }
            Self::UnknownTerrain(k) => write!(f, "unknown terrain `{k}`"),
            Self::UnknownCreature(k) => write!(f, "unknown creature `{k}`"),
            Self::UnknownSkill(k) => write!(f, "unknown skill `{k}`"),
            Self::UnknownResource(k) => write!(f, "unknown resource `{k}`"),
            Self::UnknownObjectType(k) => write!(f, "unknown object type `{k}`"),
            Self::InvalidObjectType { key, reason } => {
                write!(f, "object type `{key}`: {reason}")
            }
            Self::DuplicateTerrain(k) => write!(f, "terrain `{k}` has movement data twice"),
            Self::MissingTerrain(k) => write!(f, "terrain `{k}` has no movement data"),
            Self::PathfindingCosts(k) => write!(
                f,
                "terrain `{k}`: costs by Pathfinding level need a land terrain and must not rise"
            ),
            Self::TerrainWalker(k) => write!(
                f,
                "terrain walker of `{k}`: both terrains must be land, one walker per terrain"
            ),
            Self::MissingParams => f.write_str("no pack provides the adventure parameters"),
            Self::DuplicateParams(p) => {
                write!(
                    f,
                    "pack `{p}` provides the adventure parameters a second time"
                )
            }
            Self::InvalidParams(reason) => write!(f, "adventure parameters: {reason}"),
            Self::MissingDifficulties => {
                f.write_str("no pack provides the starting resources by difficulty")
            }
            Self::DuplicateDifficulties(p) => write!(
                f,
                "pack `{p}` provides the starting resources by difficulty a second time"
            ),
            Self::InvalidDifficulty { row, reason } => write!(f, "difficulty {row}: {reason}"),
            Self::TooManyEntries(kind) => write!(f, "too many {kind} entries"),
        }
    }
}

/// The adventure context's immutable compiled ruleset part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdventureRuleset {
    core_hash: u64,
    terrains: Vec<TerrainMovement>,
    roads: Vec<RoadKind>,
    road_index: BTreeMap<String, RoadId>,
    rivers: Vec<RiverKind>,
    river_index: BTreeMap<String, RiverId>,
    object_types: Vec<ObjectType>,
    object_type_index: BTreeMap<String, ObjectTypeId>,
    params: AdventureParams,
    starting_resources: Vec<StartingResources>,
    not_random_monsters: BTreeSet<CreatureTypeId>,
    terrain_walkers: Vec<TerrainWalker>,
    hash: u64,
}

/// The tables being filled while the packs compile.
#[derive(Default)]
struct Tables {
    terrains: BTreeMap<usize, (Passage, Option<[NonZeroU16; 3]>)>,
    roads: Vec<RoadKind>,
    road_index: BTreeMap<String, RoadId>,
    rivers: Vec<RiverKind>,
    river_index: BTreeMap<String, RiverId>,
    objects: ObjectTables,
    /// The parameters with their references resolved.
    params: Option<ResolvedParams>,
    starting_resources: Option<Vec<StartingResources>>,
    not_random_monsters: BTreeSet<CreatureTypeId>,
    terrain_walkers: Vec<TerrainWalker>,
}

/// The parameters with the battle terrains, the monsters' price resource and (E08 T83a) the
/// months' creatures resolved; the months' object type is resolved once every type is in.
struct ResolvedParams {
    def: AdventureParamsDef,
    battle: Option<BattleTerrains>,
    price: Option<ResourceId>,
    month_creatures: Vec<CreatureTypeId>,
}

impl AdventureRuleset {
    /// Compiles one adventure pack per pack of `core`, in the core's load order.
    pub fn compile(core: &Ruleset, packs: &[AdventurePack]) -> Result<Self, AdventureCompileError> {
        let order = load_order(core, packs)?;
        let mut tables = Tables::default();
        for (pack, closure) in &order {
            compile_pack(core, pack, closure, &mut tables)?;
        }
        let terrains = core
            .terrains()
            .iter()
            .enumerate()
            .map(|(index, info)| {
                tables
                    .terrains
                    .get(&index)
                    .map(|&(passage, pathfinding)| TerrainMovement {
                        key: info.key.clone(),
                        passage,
                        pathfinding,
                    })
                    .ok_or_else(|| {
                        AdventureCompileError::MissingTerrain(String::from(info.key.as_str()))
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        check_terrain_walkers(core, &terrains, &tables.terrain_walkers)?;
        let resolved = tables.params.ok_or(AdventureCompileError::MissingParams)?;
        let month_monster = resolved
            .def
            .months
            .as_ref()
            .map(|months| month_monster(&tables.objects, &months.monster))
            .transpose()?;
        let params = validate_params(resolved, month_monster)?;
        check_object_types(
            core,
            &tables.objects.types,
            &tables.not_random_monsters,
            params.army_slots(),
            params.max_stack_count(),
        )?;
        let starting_resources = tables
            .starting_resources
            .ok_or(AdventureCompileError::MissingDifficulties)?;
        let mut part = Self {
            core_hash: core.content_hash(),
            terrains,
            roads: tables.roads,
            road_index: tables.road_index,
            rivers: tables.rivers,
            river_index: tables.river_index,
            object_types: tables.objects.types,
            object_type_index: tables.objects.index,
            params,
            starting_resources,
            not_random_monsters: tables.not_random_monsters,
            terrain_walkers: tables.terrain_walkers,
            hash: 0,
        };
        part.hash = hash_of(&part);
        Ok(part)
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

    /// Every core terrain's movement data, in the core's terrain order.
    pub fn terrains(&self) -> &[TerrainMovement] {
        &self.terrains
    }

    /// How the terrain `terrain` is crossed.
    pub fn passage(&self, terrain: TerrainId) -> Option<Passage> {
        self.terrains
            .get(usize::from(terrain.raw()))
            .map(|t| t.passage)
    }

    /// The straight cost of a step on `terrain` for a hero with Pathfinding at `pathfinding` (0
    /// none to 3 Expert; a higher level counts as Expert), without a road or the army's native
    /// terrain (ADV-MOV-003, the original's `step_cost_table` 0x63E510; E08 T29): a land
    /// terrain's cost at that level, the cost without where the pack gives none, a water
    /// terrain's cost at every level; `None` for an impassable or unknown terrain. Increment 4
    /// reads it for the map's terrain shares only (ADV-MAP-002).
    pub fn step_cost(&self, terrain: TerrainId, pathfinding: u8) -> Option<NonZeroU16> {
        let movement = self.terrains.get(usize::from(terrain.raw()))?;
        let at_level = usize::from(pathfinding.min(3))
            .checked_sub(1)
            .and_then(|i| movement.pathfinding.and_then(|costs| costs.get(i).copied()));
        match movement.passage {
            Passage::Land(cost) => Some(at_level.unwrap_or(cost)),
            Passage::Water(cost) => Some(cost),
            Passage::Impassable => None,
        }
    }

    /// The road kinds.
    pub fn roads(&self) -> &[RoadKind] {
        &self.roads
    }

    /// A road kind.
    pub fn road(&self, id: RoadId) -> Option<&RoadKind> {
        self.roads.get(usize::from(id.raw()))
    }

    /// The road kind with this key.
    pub fn road_by_key(&self, key: &str) -> Option<RoadId> {
        self.road_index.get(key).copied()
    }

    /// The river kinds.
    pub fn rivers(&self) -> &[RiverKind] {
        &self.rivers
    }

    /// A river kind.
    pub fn river(&self, id: RiverId) -> Option<&RiverKind> {
        self.rivers.get(usize::from(id.raw()))
    }

    /// The river kind with this key.
    pub fn river_by_key(&self, key: &str) -> Option<RiverId> {
        self.river_index.get(key).copied()
    }

    /// The object types (ADV-OBJ-002).
    pub fn object_types(&self) -> &[ObjectType] {
        &self.object_types
    }

    /// An object type.
    pub fn object_type(&self, id: ObjectTypeId) -> Option<&ObjectType> {
        self.object_types.get(usize::from(id.raw()))
    }

    /// The object type with this key.
    pub fn object_type_by_key(&self, key: &str) -> Option<ObjectTypeId> {
        self.object_type_index.get(key).copied()
    }

    /// The daily amount of the mine that produces `resource` (E08 T83a; ADV-MIN-003: an
    /// abandoned mine's drawn resource's), if a type's kind is that [`ObjectKind::Mine`]: the
    /// compile keeps one mine per resource, and with an abandoned mine one for every resource.
    pub fn mine_daily(&self, resource: ResourceId) -> Option<u32> {
        self.object_types.iter().find_map(|ty| match ty.kind {
            ObjectKind::Mine {
                resource: r, daily, ..
            } if r == resource => Some(daily),
            _ => None,
        })
    }

    /// The adventure parameters.
    pub const fn params(&self) -> &AdventureParams {
        &self.params
    }

    /// The starting resources by the game's difficulty, from 0 (easy; GAM-RES-002): at least one
    /// row.
    pub fn starting_resources(&self) -> &[StartingResources] {
        &self.starting_resources
    }

    /// The terrain walkers, in load order (E08 T40, ADV-MOV-003): at most one per terrain.
    pub fn terrain_walkers(&self) -> &[TerrainWalker] {
        &self.terrain_walkers
    }

    /// The creatures a random monster of these levels (inclusive) is drawn from (ADV-MON-003,
    /// E08 T20a2a), in the core's order: those of a level between them that are no war machine
    /// and that no pack leaves out (`not_random_monsters`).
    pub fn random_monsters<'a>(
        &'a self,
        core: &'a Ruleset,
        levels: [u8; 2],
    ) -> impl Iterator<Item = CreatureTypeId> + 'a {
        random_monsters(core, &self.not_random_monsters, levels)
    }
}

impl CanonicalHash for AdventureRuleset {
    /// Encoding v14 (E08 T83a; v1 was T4's, v2 T5a's, v3 T6b's, v4 T12a's, v5 T13a's, v6 T19a's,
    /// v7 T20a2a's, v8 T20b1's, v9 T29's, v10 T40's, v11 T50's, v12 T61a's, v13 T72's): version; the terrains in the
    /// core's order (key, passage: 0 land, 1 water with the cost, 2 impassable; since v9 the
    /// costs by Pathfinding level, 0 for none, else 1 and the three costs); the road kinds
    /// (key, name, cost); the river kinds (key, name); the parameters (native cost, the land
    /// table as its length and entries, scouting radius; since v2 the largest map size, levels,
    /// players and the army slots; since v3 the starting stack chances as their length and
    /// entries); since v4 the object types in handle order ([`ObjectType`]'s encoding); since v5
    /// the starting resources by difficulty (their number, then each row's human and computer
    /// amounts as length and entries); since v6 the battle terrains (0 for none, else 1, the
    /// underground's and the coast's terrain, the coast's battlefield key); since v7 (E08 T20a2a)
    /// the monsters' parameters (0 for none, else 1, the aggression rows as their number and each
    /// row's bounds, the count words' thresholds as length and entries) and the creatures no
    /// random monster becomes, as length and handles in order; since v10 (E08 T40) the fly and
    /// the water-walk prices (four each) and the terrain walkers as length and each one's
    /// creature, terrain and the terrain it counts as (a ground's `no_native` in
    /// [`ObjectType`]'s); since v11 (E08 T50) a stack's most creatures after the army slots; since
    /// v13 (E08 T72) the battle terrains' sea terrain, deck and boats after the coast's
    /// battlefield, and the sea points (four) and the sea ground's share (numerator, denominator)
    /// after the terrain walkers; since v14 (E08 T83a) increment 9's seven kinds in
    /// [`ObjectType`]'s, the monsters' growth (basis points, unit, cap) after their price
    /// resource, and last the calendar (a week's days, a month's weeks) and the months (0 for
    /// none, else 1, the three weights, the creatures as length and handles, the monster's type,
    /// the spawn's chance, factor and aggression bounds). The core's hash is not part of it:
    /// replays record both.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u32(ADVENTURE_RULESET_HASH_VERSION);
        h.write_len(self.terrains.len());
        for t in &self.terrains {
            t.key.as_str().hash_into(h);
            t.passage.hash_into(h);
            match t.pathfinding {
                None => h.write_u8(0),
                Some(costs) => {
                    h.write_u8(1);
                    costs.iter().for_each(|cost| h.write_u16(cost.get()));
                }
            }
        }
        h.write_len(self.roads.len());
        for r in &self.roads {
            r.key.as_str().hash_into(h);
            r.name.as_str().hash_into(h);
            h.write_u16(r.cost.get());
        }
        h.write_len(self.rivers.len());
        for r in &self.rivers {
            r.key.as_str().hash_into(h);
            r.name.as_str().hash_into(h);
        }
        h.write_u16(self.params.native_cost.get());
        self.params.land_points.hash_into(h);
        h.write_u8(self.params.scouting_radius);
        h.write_u16(self.params.max_map_size.get());
        h.write_u8(self.params.max_levels.get());
        h.write_u8(self.params.max_players.get());
        h.write_u8(self.params.army_slots.get());
        h.write_u32(self.params.max_stack_count.get());
        self.params.starting_stack_chances.hash_into(h);
        h.write_len(self.object_types.len());
        for ty in &self.object_types {
            ty.hash_into(h);
        }
        h.write_len(self.starting_resources.len());
        for row in &self.starting_resources {
            row.hash_into(h);
        }
        match &self.params.battle {
            None => h.write_u8(0),
            Some(battle) => {
                h.write_u8(1);
                h.write_u16(battle.underground.raw());
                h.write_u16(battle.coast.raw());
                battle.coast_battlefield.as_str().hash_into(h);
                h.write_u16(battle.sea.raw());
                battle.deck.as_str().hash_into(h);
                battle.boats.as_str().hash_into(h);
            }
        }
        match &self.params.monsters {
            None => h.write_u8(0),
            Some(monsters) => {
                h.write_u8(1);
                h.write_len(monsters.aggression.len());
                for [low, high] in &monsters.aggression {
                    h.write_i32(i32::from(*low));
                    h.write_i32(i32::from(*high));
                }
                monsters.count_words.hash_into(h);
                monsters.price_resource.hash_into(h);
                h.write_u16(monsters.growth_bp);
                h.write_u16(monsters.growth_unit.get());
                h.write_u32(monsters.growth_cap.get());
            }
        }
        h.write_len(self.not_random_monsters.len());
        for id in &self.not_random_monsters {
            id.hash_into(h);
        }
        for cost in self
            .params
            .fly_cost
            .iter()
            .chain(&self.params.water_walk_cost)
        {
            h.write_u16(cost.get());
        }
        h.write_len(self.terrain_walkers.len());
        for walker in &self.terrain_walkers {
            walker.creature.hash_into(h);
            walker.terrain.hash_into(h);
            walker.counts_as.hash_into(h);
        }
        for &points in &self.params.sea_points {
            h.write_u32(points);
        }
        for &part in &self.params.sea_ground {
            h.write_u16(part);
        }
        h.write_u8(self.params.calendar.week_days.get());
        h.write_u8(self.params.calendar.month_weeks.get());
        match &self.params.months {
            None => h.write_u8(0),
            Some(months) => {
                h.write_u8(1);
                months.odds.iter().for_each(|&w| h.write_u8(w));
                months.creatures.hash_into(h);
                months.monster.hash_into(h);
                h.write_u16(months.spawn_one_in.get());
                h.write_u8(months.spawn_factor.get());
                h.write_i32(i32::from(months.spawn_aggression[0]));
                h.write_i32(i32::from(months.spawn_aggression[1]));
            }
        }
    }
}

/// Pairs each core pack, in the core's load order, with its adventure pack and dependency
/// closure (itself included).
fn load_order<'a>(
    core: &Ruleset,
    packs: &'a [AdventurePack],
) -> Result<Vec<(&'a AdventurePack, BTreeSet<String>)>, AdventureCompileError> {
    let mut by_id: BTreeMap<&str, &AdventurePack> = BTreeMap::new();
    for pack in packs {
        let id = pack.manifest.id.as_str();
        let known = core
            .packs()
            .iter()
            .any(|(core_id, version)| core_id == id && *version == pack.manifest.version);
        if !known {
            return Err(AdventureCompileError::UnknownPack(format!(
                "{id}@{}",
                pack.manifest.version
            )));
        }
        if by_id.insert(id, pack).is_some() {
            return Err(AdventureCompileError::DuplicatePack(String::from(id)));
        }
    }
    let mut closures: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    let mut order = Vec::new();
    for (id, _) in core.packs() {
        let pack = *by_id
            .get(id.as_str())
            .ok_or_else(|| AdventureCompileError::MissingPack(id.clone()))?;
        let mut closure = BTreeSet::from([id.clone()]);
        for dependency in &pack.manifest.depends {
            let inner = closures.get(dependency.as_str()).ok_or_else(|| {
                AdventureCompileError::UnknownDependency {
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

/// Adds one pack's definitions to the tables.
fn compile_pack(
    core: &Ruleset,
    pack: &AdventurePack,
    closure: &BTreeSet<String>,
    tables: &mut Tables,
) -> Result<(), AdventureCompileError> {
    let id = pack.manifest.id.as_str();
    for def in &pack.roads {
        check_namespace(&def.key, id)?;
        let handle = RoadId::from_index(tables.roads.len())
            .ok_or(AdventureCompileError::TooManyEntries("road"))?;
        insert_unique(&mut tables.road_index, &def.key, handle)?;
        tables.roads.push(RoadKind {
            key: def.key.clone(),
            name: def.name.clone(),
            cost: def.cost,
        });
    }
    for def in &pack.rivers {
        check_namespace(&def.key, id)?;
        let handle = RiverId::from_index(tables.rivers.len())
            .ok_or(AdventureCompileError::TooManyEntries("river"))?;
        insert_unique(&mut tables.river_index, &def.key, handle)?;
        tables.rivers.push(RiverKind {
            key: def.key.clone(),
            name: def.name.clone(),
        });
    }
    for def in &pack.terrains {
        let key = def.terrain.as_str();
        let terrain = core_terrain(core, id, closure, &def.terrain)?;
        if !pathfinding_fits(def) {
            return Err(AdventureCompileError::PathfindingCosts(String::from(key)));
        }
        if tables
            .terrains
            .insert(usize::from(terrain.raw()), (def.passage, def.pathfinding))
            .is_some()
        {
            return Err(AdventureCompileError::DuplicateTerrain(String::from(key)));
        }
    }
    compile_object_types(core, id, closure, &pack.object_types, &mut tables.objects)?;
    for key in &pack.not_random_monsters {
        if !closure.contains(key.pack()) {
            return Err(AdventureCompileError::ReferenceOutsideDependencies {
                key: String::from(key.as_str()),
                pack: String::from(id),
            });
        }
        let creature = core
            .creature_by_key(key.as_str())
            .ok_or_else(|| AdventureCompileError::UnknownCreature(String::from(key.as_str())))?;
        tables.not_random_monsters.insert(creature);
    }
    for def in &pack.terrain_walkers {
        if !closure.contains(def.creature.pack()) {
            return Err(AdventureCompileError::ReferenceOutsideDependencies {
                key: String::from(def.creature.as_str()),
                pack: String::from(id),
            });
        }
        let creature = core.creature_by_key(def.creature.as_str()).ok_or_else(|| {
            AdventureCompileError::UnknownCreature(String::from(def.creature.as_str()))
        })?;
        tables.terrain_walkers.push(TerrainWalker {
            creature,
            terrain: core_terrain(core, id, closure, &def.terrain)?,
            counts_as: core_terrain(core, id, closure, &def.counts_as)?,
        });
    }
    if !pack.starting_resources.is_empty() {
        if tables.starting_resources.is_some() {
            return Err(AdventureCompileError::DuplicateDifficulties(String::from(
                id,
            )));
        }
        let scope = Scope {
            core,
            pack: id,
            closure,
        };
        tables.starting_resources = Some(compile_starting(&scope, &pack.starting_resources)?);
    }
    if let Some(params) = &pack.params {
        if tables.params.is_some() {
            return Err(AdventureCompileError::DuplicateParams(String::from(id)));
        }
        let battle = params
            .battle
            .as_ref()
            .map(|def| {
                for field in [&def.coast_battlefield, &def.deck, &def.boats] {
                    if !closure.contains(field.pack()) {
                        return Err(AdventureCompileError::ReferenceOutsideDependencies {
                            key: String::from(field.as_str()),
                            pack: String::from(id),
                        });
                    }
                }
                Ok(BattleTerrains {
                    underground: core_terrain(core, id, closure, &def.underground)?,
                    coast: core_terrain(core, id, closure, &def.coast)?,
                    coast_battlefield: def.coast_battlefield.clone(),
                    sea: core_terrain(core, id, closure, &def.sea)?,
                    deck: def.deck.clone(),
                    boats: def.boats.clone(),
                })
            })
            .transpose()?;
        let scope = Scope {
            core,
            pack: id,
            closure,
        };
        let price = params
            .monsters
            .as_ref()
            .map(|monsters| scope.resource(&monsters.price_resource))
            .transpose()?;
        let mut month_creatures = Vec::new();
        if let Some(months) = &params.months {
            scope.reach(&months.monster)?;
            for key in &months.creatures {
                let creature = scope.creature(key)?;
                check_month_creature(core, &scope, creature, months.spawn_factor)?;
                month_creatures.push(creature);
            }
        }
        tables.params = Some(ResolvedParams {
            def: params.clone(),
            battle,
            price,
            month_creatures,
        });
    }
    Ok(())
}

/// Whether a month's creature may be a month's monster (E08 T83a): no war machine, an adventure
/// count from 1, and the count's highest times the spawn's factor within [`MAX_MONSTER_COUNT`].
fn check_month_creature(
    core: &Ruleset,
    scope: &Scope<'_>,
    creature: CreatureTypeId,
    factor: NonZeroU8,
) -> Result<(), AdventureCompileError> {
    let [low, high] = core
        .creature(creature)
        .map_or([0, 0], |info| info.adventure_count);
    let reason = if scope.war_machine(creature) {
        Some("a months.creatures entry is a war machine")
    } else if low == 0 {
        Some("a months.creatures entry has no adventure count")
    } else if u32::from(high) * u32::from(factor.get()) > MAX_MONSTER_COUNT {
        Some("a months.creatures entry's count times spawn_factor is above MAX_MONSTER_COUNT")
    } else {
        None
    };
    reason.map_or(Ok(()), |reason| {
        Err(AdventureCompileError::InvalidParams(reason))
    })
}

/// The months' object type (E08 T83a): a monster whose creature the map names.
fn month_monster(
    objects: &ObjectTables,
    key: &ContentKey,
) -> Result<ObjectTypeId, AdventureCompileError> {
    let id = objects
        .index
        .get(key.as_str())
        .copied()
        .ok_or_else(|| AdventureCompileError::UnknownObjectType(String::from(key.as_str())))?;
    let named = objects
        .types
        .get(usize::from(id.raw()))
        .is_some_and(|ty| matches!(ty.kind, ObjectKind::Monster { levels: None }));
    if named {
        Ok(id)
    } else {
        Err(AdventureCompileError::InvalidParams(
            "months.monster is not a monster whose creature the map names",
        ))
    }
}

/// Checks the terrain walkers once every pack's terrains are known (E08 T40): each walker's
/// terrain and the terrain it counts as are crossed on land, and no two walkers name one terrain
/// (an army holding both creatures would count it as two).
fn check_terrain_walkers(
    core: &Ruleset,
    terrains: &[TerrainMovement],
    walkers: &[TerrainWalker],
) -> Result<(), AdventureCompileError> {
    let land = |terrain: TerrainId| {
        terrains
            .get(usize::from(terrain.raw()))
            .is_some_and(|t| matches!(t.passage, Passage::Land(_)))
    };
    let mut seen = BTreeSet::new();
    for walker in walkers {
        if !(land(walker.terrain) && land(walker.counts_as) && seen.insert(walker.terrain.raw())) {
            let key = core
                .terrains()
                .get(usize::from(walker.terrain.raw()))
                .map_or("", |info| info.key.as_str());
            return Err(AdventureCompileError::TerrainWalker(String::from(key)));
        }
    }
    Ok(())
}

/// Whether a terrain's costs by Pathfinding level fit (E08 T29): none, or a land terrain's, each
/// at most the cost below it.
fn pathfinding_fits(def: &TerrainMovementDef) -> bool {
    match (def.passage, def.pathfinding) {
        (_, None) => true,
        (Passage::Land(cost), Some(costs)) => core::iter::once(cost)
            .chain(costs)
            .zip(costs)
            .all(|(below, at)| at <= below),
        (Passage::Water(_) | Passage::Impassable, Some(_)) => false,
    }
}

/// The core terrain `key` names, which `pack` (with dependency closure `closure`) may reference.
fn core_terrain(
    core: &Ruleset,
    pack: &str,
    closure: &BTreeSet<String>,
    key: &ContentKey,
) -> Result<TerrainId, AdventureCompileError> {
    if !closure.contains(key.pack()) {
        return Err(AdventureCompileError::ReferenceOutsideDependencies {
            key: String::from(key.as_str()),
            pack: String::from(pack),
        });
    }
    core.terrain_by_key(key.as_str())
        .ok_or_else(|| AdventureCompileError::UnknownTerrain(String::from(key.as_str())))
}

fn validate_params(
    resolved: ResolvedParams,
    month_monster: Option<ObjectTypeId>,
) -> Result<AdventureParams, AdventureCompileError> {
    let ResolvedParams {
        def,
        battle,
        price,
        month_creatures,
    } = resolved;
    let reason = if def.land_points.is_empty() {
        Some("land_points is empty")
    } else if def.land_points.len() > MAX_LAND_POINTS {
        Some("land_points has more entries than speeds")
    } else if def.land_points.contains(&0) {
        Some("a land_points entry is 0")
    } else if def.starting_stack_chances.iter().any(|&c| c > 100) {
        Some("a starting_stack_chances entry is above 100")
    } else if def.max_players.get() > MAX_PLAYERS {
        Some("max_players is above MAX_PLAYERS")
    } else if def.max_stack_count.get() < MAX_MONSTER_COUNT {
        Some("max_stack_count is below MAX_MONSTER_COUNT")
    } else if def.sea_points.contains(&0) {
        Some("a sea_points entry is 0")
    } else if !(1 <= def.sea_ground[0] && def.sea_ground[0] <= def.sea_ground[1]) {
        Some("sea_ground is not 1 <= numerator <= denominator")
    } else if let Some(reason) = def.monsters.as_ref().and_then(check_monster_params) {
        Some(reason)
    } else if let Some(months) = &def.months {
        check_month_params(months, def.monsters.is_some())
    } else {
        None
    };
    match reason {
        Some(reason) => Err(AdventureCompileError::InvalidParams(reason)),
        None => Ok(AdventureParams {
            native_cost: def.native_cost,
            land_points: def.land_points,
            scouting_radius: def.scouting_radius,
            max_map_size: def.max_map_size,
            max_levels: def.max_levels,
            max_players: def.max_players,
            army_slots: def.army_slots,
            max_stack_count: def.max_stack_count,
            starting_stack_chances: def.starting_stack_chances,
            battle,
            monsters: def.monsters.zip(price).map(|(m, price)| MonsterParams {
                aggression: m.aggression,
                count_words: m.count_words,
                price_resource: price,
                growth_bp: m.growth_bp,
                growth_unit: m.growth_unit,
                growth_cap: m.growth_cap,
            }),
            fly_cost: def.fly_cost,
            water_walk_cost: def.water_walk_cost,
            sea_points: def.sea_points,
            sea_ground: def.sea_ground,
            calendar: def.calendar,
            months: def
                .months
                .zip(month_monster)
                .map(|(m, monster)| MonthParams {
                    odds: m.odds,
                    creatures: month_creatures,
                    monster,
                    spawn_one_in: m.spawn_one_in,
                    spawn_factor: m.spawn_factor,
                    spawn_aggression: m.spawn_aggression,
                }),
        }),
    }
}

/// Why the months' parameters are invalid, if they are (E08 T83a): odds summing to 0, creatures
/// named twice or none for a creature's month with a weight, an aggression whose low bound is
/// above its high one, or months without the monsters' parameters (their monsters are monsters).
fn check_month_params(def: &MonthParamsDef, monsters: bool) -> Option<&'static str> {
    let distinct = def.creatures.iter().collect::<BTreeSet<_>>().len() == def.creatures.len();
    if def.odds.iter().all(|&w| w == 0) {
        Some("months.odds sum to 0")
    } else if !distinct {
        Some("months.creatures names a creature twice")
    } else if def.odds[1] != 0 && def.creatures.is_empty() {
        Some("months.creatures is empty with a creature's month's weight")
    } else if def.spawn_aggression[0] > def.spawn_aggression[1] {
        Some("months.spawn_aggression's low bound is above its high one")
    } else if !monsters {
        Some("months without the monsters' parameters")
    } else {
        None
    }
}

/// Why the monsters' parameters are invalid, if they are: more aggression rows than dispositions
/// a `u8` names, a row whose low bound is above its high one, count words' thresholds not rising
/// from 2.
fn check_monster_params(def: &MonsterParamsDef) -> Option<&'static str> {
    if def.aggression.len() > 256 {
        Some("monsters.aggression has more rows than dispositions")
    } else if def.aggression.iter().any(|[low, high]| low > high) {
        Some("a monsters.aggression row's low bound is above its high one")
    } else if def
        .count_words
        .iter()
        .try_fold(1, |last, &t| (t > last).then_some(t))
        .is_none()
    {
        Some("monsters.count_words do not rise from 2")
    } else if def.growth_bp > 10_000 {
        Some("monsters.growth_bp is above 10 000")
    } else if def.growth_unit.get() > 256 {
        Some("monsters.growth_unit is above 256")
    } else if def.growth_cap.get() > MAX_MONSTER_COUNT {
        Some("monsters.growth_cap is above MAX_MONSTER_COUNT")
    } else {
        None
    }
}

fn check_namespace(key: &ContentKey, pack: &str) -> Result<(), AdventureCompileError> {
    if key.pack() == pack {
        Ok(())
    } else {
        Err(AdventureCompileError::KeyOutsidePack {
            key: String::from(key.as_str()),
            pack: String::from(pack),
        })
    }
}

fn insert_unique<T>(
    index: &mut BTreeMap<String, Handle<T>>,
    key: &ContentKey,
    handle: Handle<T>,
) -> Result<(), AdventureCompileError> {
    if index.insert(String::from(key.as_str()), handle).is_some() {
        return Err(AdventureCompileError::DuplicateKey(String::from(
            key.as_str(),
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
