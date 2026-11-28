//! The computer's search (E31 T2i0, AI-Q41; `cross-lane`, the ai lane's): the original's
//! computer pathfinder, `pathfinder_run` 0x56B440 with its relax (`pathfinder_relax` 0x4B1A70),
//! its expansion (`pathfinder_expand` 0x4B2300) and its passage past a stop
//! (`pathfinder_pass_through` 0x56AB40), ported whole in its five modes beside the human reach
//! ([`crate::Reach`]), a loop of its own (a module of its own: the reach loops' codegen, the
//! adventure lane's traps).
//!
//! The search keeps two nodes per tile, as the original does (flags bit 10, [`marks::PLACE`]): a
//! place, where the hero may stop, and a pass node, over which it only goes on. Its order is not
//! the points left but a score S, the node's key plus, once the node lies past the day's points,
//! its danger (the caller's turn buffer, [`SearchAsks::danger`]): the highest S first, then the
//! lowest second cost (`cost2`, the cost with the search's surcharges); a node is replaced only by
//! a higher S, or an equal one with a lower second cost. The cost itself is the movement points
//! the path spends, its steps priced as the original's `step_cost` 0x4B1740 ([`Pricing`]), a day
//! started anew where a node's points no longer pay the step (the node's points, `+0x1C`, become
//! the day's maximum).
//!
//! From mode [`SearchMode::Threat`] the search goes through the teleporters (CPU-PTH-006:
//! `pathfinder_teleport_exits` 0x56A400, `pathfinder_gate_exit` 0x56A730, the start's own at
//! once), and mode [`SearchMode::Plan`] has the hero's jumps (CPU-PTH-007: the Door legs, the Fly
//! stretch, the sea spells, the boat node of Summon Boat and the boat marks; from mode 4 the
//! boarding and landing with the day's rest). Not in the engine: the Castle Gate and Town Portal
//! (E09, `AI-STUB-22`), the Sanctuary and the Quest Guard (E12, `AI-STUB-23`).

use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::{HeroId, PlayerId};
use h3_heroes::{HeroDetails, Movement};
use h3_kernel::handle::Handle;

use crate::movement::{
    DIRECTIONS, Pricing, day_movement, day_points, details_of, hero_movement, sea_day_points,
};
use crate::ruleset::{AdventureRuleset, Effect, MonolithWay, Passage, Visit};
use crate::setup::Pos;
use crate::state::{AdventureState, StaticMap, StaticTile, StepVisit};

use marks::{
    BOAT, CASTLE_GATE, DIRECTION_SHIFT, DOOR, DOOR_OFFSET_SHIFT, FLY, FROM_PLACE, NO_MAGIC, PLACE,
    REACHED, STOP, TOWN_PORTAL, VISIT_HERE, WATER_WALK,
};

mod expand;
mod guards;
mod jumps;
mod teleport;
mod through;

/// The search's mode, the original's sixth argument of `pathfinder_run` 0x56B440, ordered by its
/// number: the original tests "mode 3", "from mode 4" and "below 3" alike.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SearchMode {
    /// 1: from a candidate tile (`ai_hero_destination` 0x42E0B0: bound 500, 59 999 points).
    Local = 1,
    /// 2: a colleague's claims (`ai_hero_reach_claims` 0x42F570, from its kept destination).
    Claims = 2,
    /// 3: an enemy hero's threat (`ai_danger_map` 0x42DE50, `ai_enemy_reach_fold` 0x4280E0):
    /// no fog, its guards stop it ([`SearchAsks::fight_answer`] never asked).
    Threat = 3,
    /// 4: the hero's own plan (`ai_hero_reach_claims` 0x42F570): the guards weighed by the
    /// caller's answer, and the jumps ([`SearchJumps`]).
    Plan = 4,
    /// 5: the walk's replan (`ai_hero_reach_claims` 0x42F570): mode 4 without the jumps.
    Replan = 5,
}

/// How the search treats a tile its player has not explored (AI-Q41; the original's fog test at
/// `pathfinder_fog_test` 0x4B25D4, in every mode but [`SearchMode::Threat`], which has none).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchFog {
    /// The faithful policy, the original's computer test (0x4B25F1..0x4B2630): an unexplored
    /// tile is only flown over (no place) when the step comes from an unexplored tile too and
    /// the player owns a town; else it is a tile like any other.
    Computer,
    /// The fair policy (increment 6), the original's human branch (0x4B25E8): an unexplored tile
    /// is only flown over, never a place.
    Human,
}

/// Mode [`SearchMode::Plan`]'s jumps, which `pathfinder_run` 0x56B440 sets from the hero's spells
/// and masteries (pathfinder `+0x14` to `+0x17` and its levels, 0x56B4FC..0x56B731): the caller
/// computes them (the adventure never reads spell points, ADR-0017 point 10); ignored in every
/// other mode, whose search has none (mode [`SearchMode::Replan`]'s too, R2 §7). The flags give
/// the jumps (CPU-PTH-007) and the no-magic mark ([`marks::NO_MAGIC`], CPU-PTH-002); the levels
/// price the lifted steps and choose a sea spell (0x4B2C31).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SearchJumps {
    /// Summon Boat is useful (`+0x14`: castable, 0x4E5550, and no boat's entrance beside the
    /// hero).
    pub summon_boat: bool,
    /// Dimension Door is castable (`+0x15`).
    pub door: bool,
    /// Fly is castable (`+0x16`).
    pub fly: bool,
    /// Water Walk is castable (`+0x17`).
    pub water_walk: bool,
    /// A Door leg's price: 300, or 200 at Expert (`hero_spell_mastery` 0x4E5080 of spell 8).
    pub door_price: u32,
    /// The fly level the lifted steps are priced at (`+0x1C`), the caller's, as the run sets it:
    /// the school's mastery, lowered to the hero's own level when that is set and lower
    /// (0x56B710); none: no fly price.
    pub fly_level: Option<u8>,
    /// The water-walk level (`+0x18`), the caller's as [`Self::fly_level`] (0x56B722).
    pub water_walk_level: Option<u8>,
}

/// The search's box (pathfinder `+0x28` to `+0x34`, which `pathfinder_relax` 0x4B1A70 tests at
/// 0x4B1AE6..0x4B1B04): the columns from `left` to `right` and the rows from `top` to `bottom`,
/// each right and bottom exclusive, on every level, clipped to the map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchWindow {
    /// The first column.
    pub left: u16,
    /// The first row.
    pub top: u16,
    /// The column after the last.
    pub right: u16,
    /// The row after the last.
    pub bottom: u16,
}

/// What a search starts from: `pathfinder_run` 0x56B440's arguments and the computer's facts it
/// reads from the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchStart {
    /// The current player, the computer (`current_player_bit` 0x69CCC4): its explored tiles,
    /// whether it owns a town (`+0x3E`), its keys.
    pub player: PlayerId,
    /// The searching hero (argument 1): its prices, its owner's team, its day's maximums. Hidden
    /// for the search (`hero_hide` 0x4D76E0): its own tile is no hero's.
    pub hero: HeroId,
    /// The start tile (argument 2), the hero's or another (a candidate, a kept destination).
    pub from: Pos,
    /// Whether the search starts in a boat (argument 5): the start node's [`marks::BOAT`].
    pub boat: bool,
    /// The bound (argument 4): no node is first reached at a cost above it; 0 none.
    pub bound: u32,
    /// The start's points (argument 7): the start node's points and the day's points the danger
    /// and the diagonal window are measured against (pathfinder `+0x8`).
    pub points: u32,
    /// The mode (argument 6).
    pub mode: SearchMode,
    /// The fog's policy.
    pub fog: SearchFog,
    /// The box; none: the whole map.
    pub window: Option<SearchWindow>,
    /// Mode [`SearchMode::Plan`]'s jumps.
    pub jumps: SearchJumps,
    /// Whether the start tile is read as no entrance for the whole search (E31 T2i, AI-Q42):
    /// `ai_hero_destination` 0x42E0B0 clears its map item's entrance bit for a class visited
    /// from the neighbouring tile before its value field's search (0x42E263, restored at
    /// 0x42E37D). Its spot has no entrance (no guard's tile, no stop) and no closed top (no
    /// up-off-the-entrance test from it); false for every other search.
    pub open: bool,
}

/// The computer's numbers the search reads and `h3-adventure` does not compute.
pub trait SearchAsks {
    /// The turn's danger at `pos` (the tile buffer, pathfinder `+0x6C`, read by
    /// `pathfinder_relax` 0x4B1A70 at 0x4B1BBC..0x4B1C32): a node past the day's points keeps the
    /// lesser of it and its source's, and is refused at or below −500 000 000. 0 everywhere: no
    /// buffer.
    fn danger(&self, _pos: Pos) -> i32 {
        0
    }

    /// `ai_fight_answer` 0x52BD10 for the searching hero at `pos` (a monster's, a garrison's or
    /// another hero's tile), asked in modes [`SearchMode::Plan`] and [`SearchMode::Replan`] only:
    /// at or below −500 000 000 the tile is refused, below 0 its answer lowers the node's key.
    fn fight_answer(&self, pos: Pos) -> i32;

    /// A boat mark on the water tile `pos` with `ai_boat_purchase_cost` 0x431160's change of the
    /// key, if the computer marked one (map item `+0xC` bit 11, `pathfinder_boat_mark_read`
    /// 0x4B2B01; CPU-PTH-007): asked in every mode of a step from a place onto plain water where
    /// no boat is summoned; a mark makes a boat node there.
    fn boat_mark(&self, _pos: Pos) -> Option<i32> {
        None
    }
}

/// The asks of a search that needs none: no danger, no boat marks, every fight answered 0 (mode
/// [`SearchMode::Threat`] never asks one).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoAsks;

impl SearchAsks for NoAsks {
    fn fight_answer(&self, _pos: Pos) -> i32 {
        0
    }
}

/// The marks of a [`SearchNode`], the original's node flags (`+4`) by their bit numbers.
pub mod marks {
    /// Bit 0: the node was reached.
    pub const REACHED: u32 = 1 << 0;
    /// Bit 1: the node's tile is an entrance, a stop: the search goes on from it only where
    /// `pathfinder_pass_through` 0x56AB40 agrees.
    pub const STOP: u32 = 1 << 1;
    /// Bit 2: in a boat.
    pub const BOAT: u32 = 1 << 2;
    /// Bit 3: on a tile without magic (Cursed Ground, an anti-magic garrison's entrance), marked
    /// only when mode [`super::SearchMode::Plan`] has a jump (`pathfinder_no_magic_mark`
    /// 0x4B1DBD): no Door, Fly or Summon Boat from it.
    pub const NO_MAGIC: u32 = 1 << 3;
    /// Bit 4: flying, its steps priced at the fly level.
    pub const FLY: u32 = 1 << 4;
    /// Bit 5: water walking, its steps priced at the water-walk level.
    pub const WATER_WALK: u32 = 1 << 5;
    /// Bit 6: a Town Portal's node (`pathfinder_town_portal_mark` 0x56B363).
    pub const TOWN_PORTAL: u32 = 1 << 6;
    /// Bit 7: a Dimension Door's leg (`pathfinder_door_start` 0x4B2984).
    pub const DOOR: u32 = 1 << 7;
    /// Bit 8: a Castle Gate's node (`pathfinder_castle_gate_mark` 0x56AA7E).
    pub const CASTLE_GATE: u32 = 1 << 8;
    /// Bit 9: a teleporter's exit taken from the start's own tile (`pathfinder_visit_here_mark`
    /// 0x56B9CF).
    pub const VISIT_HERE: u32 = 1 << 9;
    /// Bit 10: a place, where the hero may stop (`pathfinder_step_stop_place` 0x4B2453); a node
    /// without it is a pass node, kept in the tile's second slot.
    pub const PLACE: u32 = 1 << 10;
    /// Bit 11: the node's source is a place (`pathfinder_relax_door_mark` 0x4B1B1F).
    pub const FROM_PLACE: u32 = 1 << 11;
    /// Bits 12 to 15: the direction of the step that entered the node (`DIRECTIONS`' order).
    pub const DIRECTION_SHIFT: u32 = 12;
    /// Bits 16 to 25: a Door leg's offset from its start (0x4B1B21..0x4B1BAF).
    pub const DOOR_OFFSET_SHIFT: u32 = 16;
}

/// A node of the search, the original's 30-byte node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchNode {
    /// Its tile (`+0`).
    pub pos: Pos,
    /// Its [`marks`] (`+4`).
    pub marks: u32,
    /// The tile of the node it was entered from (`+8`); the start's own for the start.
    pub from: Pos,
    /// The guard its path met and weighed last (`+0xC`), none before one.
    pub guard: Option<Pos>,
    /// Its key (`+0x10`): 0, lowered by the fight answers its path weighed.
    pub key: i32,
    /// Its danger (`+0x14`).
    pub danger: i32,
    /// The movement points its path spends (`+0x18`), a day's rest of a boarding included.
    pub cost: u16,
    /// The second cost (`+0x1A`): the cost with the search's surcharges, the order's tie-break.
    pub cost2: u16,
    /// The points the hero has on it (`+0x1C`): the start's, less each step, the day's maximum
    /// where a day started anew.
    pub points: u16,
}

/// The bits 12 to 15 a node's direction takes.
const DIRECTION_MASK: u32 = 0xF << DIRECTION_SHIFT;

/// The bits 16 to 25 a Door leg's offset takes.
const DOOR_OFFSET_MASK: u32 = 0x3FF << DOOR_OFFSET_SHIFT;

/// The answer and the danger at or below which a tile is refused (0xE2329B00).
const REFUSED: i32 = -500_000_000;

/// The open list's length from which `pathfinder_relax` drops its back entry before an insert
/// (0x4B1DE7).
const OPEN_LIMIT: usize = 500;

/// A node as its cell holds it: a [`SearchNode`] but its tile, the cell's. A cell with no marks
/// ([`Cell::EMPTY`]) is a node not reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cell {
    marks: u32,
    from: Pos,
    guard: Option<Pos>,
    key: i32,
    danger: i32,
    cost: u16,
    cost2: u16,
    points: u16,
}

impl Cell {
    /// The cell of a node not reached.
    const EMPTY: Self = Self {
        marks: 0,
        from: Pos::new(0, 0, 0),
        guard: None,
        key: 0,
        danger: 0,
        cost: 0,
        cost2: 0,
        points: 0,
    };
}

impl SearchNode {
    /// The node of `pos` stored in `cell` ([`Self::store`]).
    #[inline]
    const fn load(pos: Pos, cell: &Cell) -> Self {
        Self {
            pos,
            marks: cell.marks,
            from: cell.from,
            guard: cell.guard,
            key: cell.key,
            danger: cell.danger,
            cost: cell.cost,
            cost2: cell.cost2,
            points: cell.points,
        }
    }

    /// The node as its cell holds it.
    #[inline]
    const fn store(&self) -> Cell {
        Cell {
            marks: self.marks,
            from: self.from,
            guard: self.guard,
            key: self.key,
            danger: self.danger,
            cost: self.cost,
            cost2: self.cost2,
            points: self.points,
        }
    }
}

/// The search's S of a node (the original's order and replacement, 0x4B1CB8..0x4B1CF9,
/// 0x4B1E7A..0x4B1E93): its key, plus its danger once its cost lies past the day's `points`.
const fn score(key: i32, cost: i32, danger: i32, points: i32) -> i32 {
    if cost > points {
        key.wrapping_add(danger)
    } else {
        key
    }
}

/// The search's box and its cells' layout: two slots per tile and level, the place's first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Bounds {
    left: u16,
    top: u16,
    width: u16,
    height: u16,
}

impl Bounds {
    /// The cell of `pos`'s place (`place`) or pass node, if the box holds the tile (the
    /// original's slot: ((level × 2 + kind) × height + row) × width + column).
    fn index(self, pos: Pos, place: bool) -> Option<usize> {
        // A column or row before the box's wraps past its end: the box lies on the map.
        let column = pos.x.wrapping_sub(self.left);
        let row = pos.y.wrapping_sub(self.top);
        if column >= self.width || row >= self.height {
            return None;
        }
        let layer = usize::from(pos.level) * 2 + usize::from(!place);
        Some(
            (layer * usize::from(self.height) + usize::from(row)) * usize::from(self.width)
                + usize::from(column),
        )
    }

    /// The tile of a cell.
    fn pos(self, index: usize) -> Pos {
        let (width, height) = (usize::from(self.width), usize::from(self.height));
        let narrow = |n: usize| u16::try_from(n).unwrap_or(u16::MAX);
        Pos::new(
            self.left + narrow(index % width),
            self.top + narrow(index / width % height),
            u8::try_from(index / width / height / 2).unwrap_or(u8::MAX),
        )
    }
}

/// An entry of the open list (pathfinder `+0x38`): the S and the second cost of the node copied
/// at its insert, and its cell, which the pop reloads.
#[derive(Debug, Clone, Copy)]
struct Entry {
    score: i32,
    cost2: u16,
    cell: u32,
}

/// What a search builds: its box, its cells, the open list, the reached list and the flag of
/// the bound.
struct Frame {
    bounds: Bounds,
    /// The cells of a layer: the box's tiles.
    plane: usize,
    cells: Vec<Cell>,
    open: Vec<Entry>,
    reached: Vec<u32>,
    short: bool,
    /// The step prices [`Search::expand`] keeps between nodes: by the prices' flying and
    /// water-walking bits, the source tile's word and the window, the last straight and
    /// diagonal step's tile word and price.
    priced: ((u32, u64, u32), [(u64, u32); 2]),
}

impl Frame {
    /// The node stored in `cell`.
    #[inline]
    fn load(&self, cell: usize) -> SearchNode {
        SearchNode::load(self.bounds.pos(cell), &self.cells[cell])
    }
}

/// The original's class of an entrance (map item `+0x1E`) as the search reads it (E31 T2i0's
/// mapping, the agent's: another hero first, then an empty boat, then the object visited from
/// the tile by its kind).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    /// 34: another hero, of this owner.
    Hero(PlayerId),
    /// 8: an empty boat.
    Boat,
    /// 98: a town.
    Town,
    /// 33: a garrison, of this owner if it has one (the engine does not tell AB's class 219, a
    /// vertical garrison, from 33: every garrison is 33's).
    Garrison(Option<PlayerId>),
    /// 54: a monster.
    Monster,
    /// 212: a border gate of this key.
    Gate(u8),
    /// 9: a border guard of this key.
    Guard(u8),
    /// 85: a coast bank's (the Shipwreck's).
    Coast,
    /// 44: a one-way monolith's exit, which leads nowhere.
    Exit,
    /// 43, 45, 103 and 111: a teleporter the search goes through.
    Teleporter(Teleporter),
    /// Any other.
    Other,
}

/// A teleporter's entrance as the search goes through it (CPU-PTH-006; the engine's channels,
/// [`crate::state::Channels`], stand for the original's lists by the tile's word and subclass).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Teleporter {
    /// 43: a one-way monolith's entrance of this channel: its exits the channel's exits.
    OneWay(u8),
    /// 45: a two-way monolith of this channel: its exits the channel's other members.
    TwoWay(u8),
    /// 103: a Subterranean Gate: its exit its pair.
    Gate,
    /// 111: a Whirlpool's tile, of this object: its exits the other Whirlpools' tiles.
    Whirlpool(usize),
}

/// An entrance (map item `+0xC` bit 12): its class and whether that class is visited from the
/// neighbouring tile (`object_classes_not_entered` 0x63AA58, the settings' byte `+0`: a hero's
/// and a boat's are; an object's by its type's visit).
#[derive(Debug, Clone, Copy)]
struct Entrance {
    class: Class,
    not_entered: bool,
}

/// What stands on a tile besides its objects.
#[derive(Debug, Clone, Copy)]
enum Occupant {
    /// An empty boat.
    Boat,
    /// Another hero, of this owner.
    Hero(PlayerId),
}

/// A tile's terrain as the search reads it: the map item's terrain (`+4`), rock (9) or water (8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ground {
    Land,
    Water,
    Rock,
}

/// A tile as a step reads it.
#[derive(Debug, Clone, Copy)]
struct Tile {
    /// The tile as it stands now.
    live: StaticTile,
    /// The map item's `+0xD` bit 0: an object's blocked tile, or (the agent's mapping) a visitable
    /// tile whose visit the engine has not built.
    blocked: bool,
    /// The entrance, if the tile is one.
    entrance: Option<Entrance>,
}

/// A tile as the search reads it, read once when the search starts ([`Spots`]).
#[derive(Debug, Clone, Copy)]
struct Spot {
    /// The tile as a step reads it.
    tile: Tile,
    /// Its tile as it stands now as one word ([`tile_key`]).
    key: u64,
    /// The cell of its place in the frame, [`u32::MAX`] when the box does not hold it (or the
    /// spot was read where it is asked, outside the region).
    cell: u32,
    /// Its terrain.
    ground: Ground,
    /// Whether it is the entrance of an object not visited from above
    /// ([`AdventureState::closed_above`]).
    closed: bool,
}

/// The spots of the tiles a search reads, each read once when it starts: a tile's read is the
/// same for the whole search, the searching hero hidden and nothing it reads changed by it. They
/// cover its box grown by [`MARGIN`] each way, clipped to the map, on every level; a tile outside
/// (a teleporter's exit, a guard beyond the margin) is read where it is asked.
struct Spots {
    region: Bounds,
    /// The step from a spot to its neighbour's in each of the eight directions.
    steps: [isize; 8],
    spots: Vec<Spot>,
}

/// The tiles the spots cover around the box: a step's neighbour and the guards around it.
const MARGIN: u16 = 2;

/// Every field of `tile` in one word, two tiles alike exactly when their words are (a step's
/// price is the same onto a tile alike, [`Search::expand`]'s memo): the terrain in bits 0 to 15,
/// the road in bits 16 to 32 (whether one, then its index) and the river in bits 33 to 49 as
/// it, the five marks in bits 50 to 54. No tile's word is [`u64::MAX`].
#[inline(always)]
fn tile_key(tile: &StaticTile) -> u64 {
    let StaticTile {
        terrain,
        road,
        river,
        blocked,
        visitable,
        no_native,
        winds,
        hole,
    } = *tile;
    let road = u64::from(road.is_some()) | u64::from(road.map_or(0, Handle::raw)) << 1;
    let river = u64::from(river.is_some()) | u64::from(river.map_or(0, Handle::raw)) << 1;
    u64::from(terrain.raw())
        | road << 16
        | river << 33
        | u64::from(blocked) << 50
        | u64::from(visitable) << 51
        | u64::from(no_native) << 52
        | u64::from(winds) << 53
        | u64::from(hole) << 54
}

impl Spots {
    /// The place of `pos`'s spot, past the last when the region does not hold the tile: (level
    /// × height + row) × width + column.
    fn at(&self, pos: Pos) -> usize {
        let region = self.region;
        let column = pos.x.wrapping_sub(region.left);
        let row = pos.y.wrapping_sub(region.top);
        let outside = (column >= region.width) | (row >= region.height);
        (usize::from(pos.level) * usize::from(region.height) + usize::from(row))
            * usize::from(region.width)
            + usize::from(column)
            + usize::from(outside) * self.spots.len()
    }
}

/// What a search reads: the state, the rules, its start and asks, the hero's prices and
/// maximums, and what stands on the map.
struct Search<'s, A> {
    state: &'s AdventureState,
    part: &'s AdventureRuleset,
    map: &'s StaticMap,
    asks: &'s A,
    start: &'s SearchStart,
    /// The searching hero's owner (hero `+0x22`): the teams of `pathfinder_pass_through`.
    owner: PlayerId,
    /// The hero's own levels (its day's movement: hero `+0x112`, `+0x116`, the artifacts' 3),
    /// which the start node's flying and water walking come from.
    own: Movement,
    /// Whether the hero is in a boat (hero `+0x105` bit 18).
    aboard: bool,
    /// The hero's own tile, which `hero_flight_tile_test` 0x4E5CE0 reads.
    at: Pos,
    /// The step prices by the node's flying and water walking ([`FLY`] bit 0 of the index,
    /// [`WATER_WALK`] bit 1): `step_cost` 0x4B1740 with the fly level `+0x1C` and the water-walk
    /// level `+0x18`, each or neither.
    pricings: [Pricing<'s>; 4],
    /// The day's maximum on land (`+0xC`) and at sea (`+0x10`).
    land: u32,
    sea: u32,
    /// The day's points (`+0x8`) and the bound, as the original's 32-bit values.
    points: i32,
    bound: i32,
    /// Whether the player owns a town (`+0x3E`, its town count, above 0).
    towns: bool,
    /// The jumps the search may make (`+0x14` to `+0x17`, the door's price): the start's in mode
    /// [`SearchMode::Plan`], none in any other.
    cast: SearchJumps,
    /// Whether it has a jump: the no-magic mark.
    jumps: bool,
    /// The fly level (`+0x1C`) and the water-walk level (`+0x18`) the lifted steps are priced at,
    /// which choose a sea spell (0x4B2C31).
    levels: (Option<u8>, Option<u8>),
    /// Whether the hero flies and water-walks on its own (`hero_wears_artifact` 0x4D91F0 of Angel
    /// Wings, 72, and of the Boots of Levitation, 90, at 0x4B2C3F and 0x4B2C85: the agent's
    /// mapping, the hero's own levels without the day's spells, which only those artifacts give
    /// in the original's content): its sea spell takes no surcharge.
    wings: bool,
    boots: bool,
    /// The empty boats and the other heroes on the map, by tile in order.
    occupants: Vec<(Pos, Occupant)>,
    /// The tiles read ahead.
    spots: Spots,
}

impl<A: SearchAsks> Search<'_, A> {
    /// The terrain of `tile`.
    fn ground(&self, tile: &StaticTile) -> Ground {
        match self.part.passage(tile.terrain) {
            Some(Passage::Land(_)) => Ground::Land,
            Some(Passage::Water(_)) => Ground::Water,
            _ => Ground::Rock,
        }
    }

    /// The terrain of the tile at `pos`.
    fn ground_at(&self, pos: Pos) -> Ground {
        self.spot(pos).ground
    }

    /// The spot of `pos`: read ahead, or read now outside the spots' region.
    fn spot(&self, pos: Pos) -> Spot {
        match self.spots.spots.get(self.spots.at(pos)) {
            Some(&spot) => spot,
            None => self.spot_of(pos, self.occupant(pos)),
        }
    }

    /// The spot of `pos`, a tile the spots' region holds: a node's (the box's), its neighbours'
    /// and the guards' around them ([`MARGIN`]).
    fn near(&self, pos: Pos) -> &Spot {
        &self.spots.spots[self.spots.at(pos)]
    }

    /// The spots of the tiles in `region` on every level, level by level, row by row: each
    /// tile's spot without what stands on it, then each occupied tile's again with its occupant.
    #[inline(never)]
    fn read_ahead(&self, region: Bounds, bounds: Bounds) -> Spots {
        let mut spots = Vec::with_capacity(
            usize::from(self.map.levels()) * usize::from(region.width) * usize::from(region.height),
        );
        let (tiles, width) = (self.map.tiles(), usize::from(self.map.width()));
        let height = usize::from(self.map.height());
        for level in 0..self.map.levels() {
            for y in region.top..region.top + region.height {
                let start = (usize::from(level) * height + usize::from(y)) * width
                    + usize::from(region.left);
                let row = &tiles[start..start + usize::from(region.width)];
                spots.extend(row.iter().zip(region.left..).map(|(&tile, x)| {
                    let pos = Pos::new(x, y, level);
                    let cell = bounds.index(pos, true).map_or(u32::MAX, |cell| cell as u32);
                    Spot {
                        cell,
                        ..self.spot_with(pos, tile, None)
                    }
                }));
            }
        }
        for spot in &mut spots {
            spot.key = tile_key(&spot.tile.live);
        }
        let width = isize::try_from(usize::from(region.width)).unwrap_or(isize::MAX);
        let steps = DIRECTIONS.map(|(dx, dy)| isize::from(dy) * width + isize::from(dx));
        let mut spots = Spots {
            region,
            steps,
            spots,
        };
        for &(pos, _) in &self.occupants {
            let at = spots.at(pos);
            if let Some(spot) = spots.spots.get_mut(at) {
                *spot = Spot {
                    cell: spot.cell,
                    ..self.spot_of(pos, self.occupant(pos))
                };
            }
        }
        if self.start.open {
            // The start's spot as the cleared entrance bit reads (0x42E263): its blocked bit
            // (map item `+0xD` bit 0) kept. A start outside the region reaches nothing.
            let at = spots.at(self.start.from);
            spots.spots.get_mut(at).into_iter().for_each(|spot| {
                spot.tile.entrance = None;
                spot.closed = false;
            });
        }
        spots
    }

    /// The spot of `pos` with `occupant` standing on it: the tile as [`Self::read`] reads it, its
    /// terrain (the static map's), and the closed top, asked only where the static map or the
    /// tile as it stands now is visitable (an object is visited from no other tile).
    fn spot_of(&self, pos: Pos, occupant: Option<Occupant>) -> Spot {
        let spot = self.spot_with(pos, self.map.at(pos), occupant);
        Spot {
            key: tile_key(&spot.tile.live),
            ..spot
        }
    }

    /// [`Self::spot_of`] of `tile`, the static map's tile at `pos`, but its key, 0, inlined for
    /// the tiles read ahead (their keys after).
    #[inline(always)]
    fn spot_with(&self, pos: Pos, tile: StaticTile, occupant: Option<Occupant>) -> Spot {
        let live = self.state.live(pos, tile);
        let entrance = match occupant {
            Some(Occupant::Hero(owner)) => Some(Entrance {
                class: Class::Hero(owner),
                not_entered: true,
            }),
            Some(Occupant::Boat) => Some(Entrance {
                class: Class::Boat,
                not_entered: true,
            }),
            None if live.visitable => self
                .state
                .visit_at(self.part, self.start.player, pos)
                .map(|visit| Self::entrance_of(self.state, &visit)),
            None => None,
        };
        Spot {
            key: 0,
            cell: u32::MAX,
            tile: Tile {
                live,
                blocked: (live.blocked && !live.visitable)
                    || (live.visitable && entrance.is_none()),
                entrance,
            },
            ground: self.ground(&tile),
            closed: (tile.visitable || live.visitable) && self.state.closed_above(self.part, pos),
        }
    }

    /// What stands on `pos` besides its objects.
    fn occupant(&self, pos: Pos) -> Option<Occupant> {
        let at = self.occupants.partition_point(|&(tile, _)| tile < pos);
        self.occupants
            .get(at)
            .filter(|&&(tile, _)| tile == pos)
            .map(|&(_, occupant)| occupant)
    }

    /// The tile at `pos` as a step reads it: another hero is an entrance of class 34, an empty
    /// boat one of class 8, a visitable tile one of its object's class if its visit is built,
    /// else blocked.
    fn read(&self, pos: Pos) -> Tile {
        self.spot(pos).tile
    }

    /// The entrance of an object's visit: its class by its kind, visited from the neighbouring
    /// tile by its type's visit.
    fn entrance_of(state: &AdventureState, visit: &StepVisit<'_>) -> Entrance {
        let class = match visit.effect {
            Effect::Town => Class::Town,
            Effect::Garrison => Class::Garrison(state.owner(visit.object)),
            Effect::Monster => Class::Monster,
            Effect::Gate(key) => Class::Gate(key),
            Effect::Guard(key) => Class::Guard(key),
            Effect::Bank(bank) if bank.coast => Class::Coast,
            Effect::Monolith { channel, way } => match way {
                MonolithWay::Entrance => Class::Teleporter(Teleporter::OneWay(channel)),
                MonolithWay::Exit => Class::Exit,
                MonolithWay::TwoWay => Class::Teleporter(Teleporter::TwoWay(channel)),
            },
            Effect::SubterraneanGate => Class::Teleporter(Teleporter::Gate),
            Effect::Whirlpool(_) => Class::Teleporter(Teleporter::Whirlpool(visit.object)),
            _ => Class::Other,
        };
        Entrance {
            class,
            not_entered: visit.mode == Visit::Neighbour,
        }
    }

    /// Whether the player has explored `pos` (the tile's vision word against
    /// `current_player_bit` 0x69CCC4).
    fn explored(&self, pos: Pos) -> bool {
        self.state.is_explored(self.start.player, pos)
    }

    /// `hero_flight_tile_test` 0x4E5CE0 for the hero's own tile, the hero not in a boat: land
    /// that no object blocks (map item `+0xC` bit 0x40), not the entrance of a class visited from
    /// the neighbouring tile.
    fn stands_free(&self, pos: Pos) -> bool {
        let tile = self.read(pos);
        self.ground(&tile.live) == Ground::Land
            && !tile.blocked
            && !tile.entrance.is_some_and(|entrance| entrance.not_entered)
    }

    /// Whether `pos` is a tile without magic (`tile_special_ground` 0x4FCE20 21 or 33): its top
    /// ground takes the native price away (Cursed Ground, as the engine marks it) or it is an
    /// anti-magic garrison's entrance ([`AdventureState::anti_magic`]).
    fn no_magic(&self, pos: Pos) -> bool {
        self.map.at(pos).no_native || self.state.anti_magic(self.part, pos)
    }

    /// The prices of a node with `marks`' flying and water walking.
    fn pricing(&self, marks: u32) -> &Pricing<'_> {
        &self.pricings[(marks >> 4 & 3) as usize]
    }

    /// The start node and its relax (`pathfinder_run` 0x56B440 at 0x56B7DD..0x56B8F9; the
    /// flags at `pathfinder_start_flags` 0x56B864): on the start tile, in a boat as the start
    /// says; flying and water walking by the hero's own levels unless in a boat; a place, but
    /// for a flyer or a water walker not in a boat whose own tile fails `hero_flight_tile_test`
    /// 0x4E5CE0; its cost and costs 0, its points the start's, its source itself, relaxed as
    /// its own source in direction 2 with no price, key or guard. From mode 3 a start on a
    /// one-way monolith's entrance, a two-way monolith or a Subterranean Gate (not a Whirlpool)
    /// goes through it at once (`pathfinder_visit_here_mark` 0x56B9CF..0x56B9E8): a copy of the
    /// start node with [`VISIT_HERE`] and its second cost 50 more, handed to
    /// `pathfinder_pass_through` 0x56AB40, whose case is the teleporter's (CPU-PTH-006).
    // rule: CPU-PTH-001, CPU-PTH-006
    fn begin(&self, frame: &mut Frame) {
        let (start, own) = (self.start, &self.own);
        let lifted = own.flying.is_some() || own.water_walking.is_some();
        let mut marks = PLACE;
        if start.boat {
            marks |= BOAT;
        } else {
            marks |= (u32::from(own.flying.is_some()) * FLY)
                | (u32::from(own.water_walking.is_some()) * WATER_WALK);
        }
        if !self.aboard && lifted && !self.stands_free(self.at) {
            marks &= !PLACE;
        }
        let mut node = SearchNode {
            pos: start.from,
            marks,
            from: start.from,
            guard: None,
            key: 0,
            danger: 0,
            cost: 0,
            cost2: 0,
            points: start.points as u16,
        };
        let source = node;
        self.relax(frame, &source, &mut node, (2, 0), (0, None, false));
        // The start on a town's entrance from mode 3 (`pathfinder_castle_gate` 0x56A850, at
        // 0x56B94C) and mode 4's Town Portal (`pathfinder_town_portal` 0x56B050, at 0x56BA01).
        // stub: AI-STUB-22
        if start.mode < SearchMode::Threat {
            return;
        }
        let class = self
            .read(start.from)
            .entrance
            .map(|entrance| entrance.class);
        if let Some(Class::Teleporter(
            teleporter @ (Teleporter::OneWay(_) | Teleporter::TwoWay(_) | Teleporter::Gate),
        )) = class
        {
            let mut copy = SearchNode {
                marks: node.marks | VISIT_HERE,
                cost2: node.cost2.wrapping_add(50),
                ..node
            };
            self.teleport(frame, &mut copy, teleporter);
        }
    }

    /// The search's loop (`pathfinder_run` 0x56B440 from 0x56BA14): pops the open list's back
    /// entry, the highest S and the lowest second cost, and reloads its cell; skips it when the
    /// next back entry is the same cell (0x56BAEE..0x56BB59); clears the node's Town Portal,
    /// Castle Gate and visit marks; goes on from a stop only where `pathfinder_pass_through`
    /// 0x56AB40 agrees, from a boat node only on water (0x56BBC1); below mode 4 a node that
    /// neither flies nor is a Door leg, off the start tile, on a tile a monster may guard is
    /// guarded by the first monster `find_guarding_monster` 0x41A460 finds but the one its path
    /// met (0x56BBCE..0x56BC55); expands it at the day's points less its cost. Ends when the
    /// list is empty: the computer's callers name no destination (0x56BC7E).
    // rule: CPU-PTH-001
    fn run(&self, frame: &mut Frame) {
        self.begin(frame);
        if self.start.mode == SearchMode::Threat {
            self.pops::<true>(frame);
        } else {
            self.pops::<false>(frame);
        }
    }

    /// [`Self::run`]'s loop, compiled apart for mode [`SearchMode::Threat`] (`THREAT`: the
    /// mode known, no jumps), the threat fold's at every computer turn start.
    fn pops<const THREAT: bool>(&self, frame: &mut Frame) {
        let mode = self.mode::<THREAT>();
        while let Some(entry) = frame.open.pop() {
            if frame
                .open
                .last()
                .is_some_and(|next| next.cell == entry.cell)
            {
                continue;
            }
            let cell = entry.cell as usize;
            let mut node = frame.load(cell);
            node.marks &= !(TOWN_PORTAL | CASTLE_GATE | VISIT_HERE);
            let left = self.points.wrapping_sub(i32::from(node.cost));
            if node.marks & STOP != 0 && !self.pass_through(frame, cell, &mut node) {
                continue;
            }
            if node.marks & BOAT != 0 && self.near(node.pos).ground != Ground::Water {
                continue;
            }
            let guarded = if node.marks & (FLY | DOOR) == 0
                && mode < SearchMode::Plan
                && self.state.may_guard(node.pos)
                && node.pos != self.start.from
            {
                self.guarding(node.pos, node.guard)
            } else {
                None
            };
            self.expand::<THREAT>(frame, &node, left, guarded);
        }
    }

    /// The search's mode, [`SearchMode::Threat`] when `THREAT`.
    fn mode<const THREAT: bool>(&self) -> SearchMode {
        if THREAT {
            SearchMode::Threat
        } else {
            self.start.mode
        }
    }

    /// `pathfinder_relax` 0x4B1A70: relaxes `node`, a step from `source` in `direction` priced
    /// `price`, with `key`, `guard` and the stop flag `stop`, writing into `node` as the
    /// original does (its callers reuse it). Its cost is the source's plus the price, its second
    /// cost its own surcharges over the source's cost plus the new cost (0x4B1A7E..0x4B1AA9).
    /// Refused outside the box (0x4B1AE6); its source the source's tile and [`FROM_PLACE`] the
    /// source's place (0x4B1B0C), its Door offset cleared, a Door leg's by the leg's rules
    /// (CPU-PTH-007, 0x4B1B21..0x4B1BAF: a landing on an entrance refused, the leg's start as
    /// its source, 9 columns and 8 rows from it at most, its offset kept). Its danger the buffer's at the tile,
    /// past the day's points the lesser of that and the source's, refused at or below −500 000 000
    /// (0x4B1BBC..0x4B1C32). Against the tile's stored node of its kind, if reached: refused
    /// when the stored S is higher, or equal with a second cost no higher (0x4B1CB8..0x4B1CF9).
    /// Past a bound, unless strictly better, refused and the search marked short
    /// (0x4B1CFF..0x4B1D15). [`NO_MAGIC`] the stored node's, else, with a jump in mode 4, the
    /// tile's (`pathfinder_no_magic_mark` 0x4B1DBD). The open list at 500 entries or more first
    /// drops its back entry, the best (0x4B1DE7: a slip the original's list has, ported as
    /// read), then the node goes in at its place, ascending S and, among equal S, descending
    /// second cost, before the entries equal to it (0x4B1E27..0x4B1EC2); a place first reached
    /// joins the reached list (0x4B2166); the node is stored (0x4B22E5).
    // rule: CPU-PTH-002
    #[inline(always)]
    fn relax(
        &self,
        frame: &mut Frame,
        source: &SearchNode,
        node: &mut SearchNode,
        step: (u32, u32),
        weigh: (i32, Option<Pos>, bool),
    ) {
        let place = frame.bounds.index(node.pos, true);
        self.relax_in(frame, source, (node, place), step, weigh);
    }

    /// [`Self::relax`] of `node` whose tile's place has the cell `place`, if the box holds the
    /// tile (its pass node's the cell a layer later).
    #[inline(always)]
    fn relax_in(
        &self,
        frame: &mut Frame,
        source: &SearchNode,
        (node, place): (&mut SearchNode, Option<usize>),
        (direction, price): (u32, u32),
        (key, guard, stop): (i32, Option<Pos>, bool),
    ) {
        let cost = i32::from(source.cost).wrapping_add(price as i32);
        let cost2 = i32::from(node.cost2) - i32::from(node.cost) + cost;
        let Some(place) = place else {
            return;
        };
        let cell = place + usize::from(node.marks & PLACE == 0) * frame.plane;
        // A Door leg takes its source and marks before the leg's rules, any other node once it
        // is kept: a refused node's are read by none (each relax sets them anew).
        let door = node.marks & DOOR != 0;
        if door {
            Self::link(source, node);
            if !jumps::leap(source, node, stop) {
                return;
            }
        }
        let buffer = self.asks.danger(node.pos);
        let danger = if cost > self.points {
            let danger = buffer.min(source.danger);
            if danger <= REFUSED {
                return;
            }
            danger
        } else {
            buffer
        };
        let new = score(key, cost, danger, self.points);
        let stored = SearchNode::load(node.pos, &frame.cells[cell]);
        let reached = stored.marks & REACHED != 0;
        let mut better = false;
        if reached {
            let old = score(
                stored.key,
                i32::from(stored.cost),
                stored.danger,
                self.points,
            );
            if old > new || (old == new && cost2 >= i32::from(stored.cost2)) {
                return;
            }
            better = old < new;
        }
        if cost > self.bound && self.bound > 0 && !better {
            frame.short = true;
            return;
        }
        if !door {
            Self::link(source, node);
        }
        let relaxed = (cost, cost2, danger, new);
        self.insert(
            frame,
            node,
            (cell, stored.marks),
            relaxed,
            (direction, key, guard, stop),
        );
    }

    /// `node`'s source `source`'s tile, its [`FROM_PLACE`] the source's place and its Door offset
    /// cleared (0x4B1B0C).
    #[inline(always)]
    fn link(source: &SearchNode, node: &mut SearchNode) {
        node.from = source.pos;
        node.marks = node.marks & !(FROM_PLACE | DOOR_OFFSET_MASK) | (source.marks & PLACE) << 1;
    }

    /// The rest of [`Self::relax`] for a node it keeps, in `cell` whose stored node has the
    /// marks `stored`, with its cost, second cost, danger and S: out of line, the refusals
    /// before it inlined into each step.
    #[inline(never)]
    fn insert(
        &self,
        frame: &mut Frame,
        node: &mut SearchNode,
        (cell, stored): (usize, u32),
        (cost, cost2, danger, new): (i32, i32, i32, i32),
        (direction, key, guard, stop): (u32, i32, Option<Pos>, bool),
    ) {
        let reached = stored & REACHED != 0;
        if reached {
            node.marks = node.marks & !NO_MAGIC | stored & NO_MAGIC;
        } else {
            node.marks &= !NO_MAGIC;
            if self.jumps && self.no_magic(node.pos) {
                node.marks |= NO_MAGIC;
            }
        }
        if frame.open.len() >= OPEN_LIMIT {
            frame.open.pop();
        }
        let at = frame
            .open
            .partition_point(|e| e.score < new || (e.score == new && cost2 < i32::from(e.cost2)));
        node.marks = (node.marks & !(DIRECTION_MASK | STOP))
            | ((direction & 0xF) << DIRECTION_SHIFT)
            | (u32::from(stop) * STOP)
            | REACHED;
        node.cost = cost as u16;
        node.cost2 = cost2 as u16;
        node.key = key;
        node.danger = danger;
        node.guard = guard;
        let entry = Entry {
            score: score(key, i32::from(node.cost), danger, self.points),
            cost2: node.cost2,
            cell: cell as u32,
        };
        frame.open.insert(at, entry);
        if !reached && node.marks & PLACE != 0 {
            frame.reached.push(entry.cell);
        }
        frame.cells[cell] = node.store();
    }
}

/// The computer's search from a start ([`AdventureState::computer_search`]): every node it
/// reached, with its marks, its source, its guard, its key, its danger, its costs and its points,
/// in the original's two slots per tile, and the order in which the places were first reached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputerSearch {
    bounds: Bounds,
    cells: Vec<Cell>,
    reached: Vec<u32>,
    short: bool,
}

impl ComputerSearch {
    /// The place of `pos` (the slot of [`marks::PLACE`]), if the search reached it.
    pub fn node(&self, pos: Pos) -> Option<SearchNode> {
        self.slot(pos, true)
    }

    /// The pass node of `pos` (the slot without [`marks::PLACE`]), if the search reached it.
    pub fn pass_node(&self, pos: Pos) -> Option<SearchNode> {
        self.slot(pos, false)
    }

    /// The node of `pos` in its slot of `place`, if reached.
    fn slot(&self, pos: Pos, place: bool) -> Option<SearchNode> {
        let cell = self.bounds.index(pos, place)?;
        self.cells
            .get(cell)
            .filter(|cell| cell.marks & REACHED != 0)
            .map(|cell| SearchNode::load(pos, cell))
    }

    /// The places in the order the search first reached them (pathfinder `+0x58`), each with
    /// its final values.
    pub fn reached(&self) -> impl DoubleEndedIterator<Item = SearchNode> + '_ {
        self.reached.iter().map(|&cell| {
            let cell = cell as usize;
            SearchNode::load(self.bounds.pos(cell), &self.cells[cell])
        })
    }

    /// Whether the bound refused a node (pathfinder `+0x20`): the search was cut short.
    pub const fn short(&self) -> bool {
        self.short
    }
}

impl AdventureState {
    /// The computer's search from `start` (see [`ComputerSearch`]; CPU-PTH-001 to -007), its
    /// hero's prices by its details in the game's store `heroes` (its Pathfinding, its army's
    /// native terrain and terrain walkers, its fly and water-walk levels: the hero's own, in
    /// mode [`SearchMode::Plan`] the jumps'), its maximums by its army, week marks and its
    /// owner's Lighthouses (`hero_max_land_movement` 0x4E4990 for land and sea, as the reach's),
    /// the computer's numbers by `asks`; none when the hero is not on the map. The search reads
    /// the state as it stands, every object still on the map, the explored tiles of the start's
    /// player.
    // rule: CPU-PTH-001
    pub fn computer_search<A: SearchAsks>(
        &self,
        core: &Ruleset,
        part: &AdventureRuleset,
        heroes: &[Arc<HeroDetails>],
        start: &SearchStart,
        asks: &A,
    ) -> Option<ComputerSearch> {
        let (search, mut frame) = self.prepare(core, part, heroes, start, asks)?;
        search.run(&mut frame);
        Some(ComputerSearch {
            bounds: frame.bounds,
            cells: frame.cells,
            reached: frame.reached,
            short: frame.short,
        })
    }

    /// The set-up of [`Self::computer_search`] (`pathfinder_run` 0x56B440 to 0x56B7A1): what
    /// the search reads, and its frame with every cell of the box zeroed (`pathfinder_reset`
    /// 0x4B1530: a node not reached has no marks), if the hero is on the map.
    fn prepare<'s, A: SearchAsks>(
        &'s self,
        core: &Ruleset,
        part: &'s AdventureRuleset,
        heroes: &'s [Arc<HeroDetails>],
        start: &'s SearchStart,
        asks: &'s A,
    ) -> Option<(Search<'s, A>, Frame)> {
        let hero = self.hero(start.hero)?;
        let map = self.map();
        let details = details_of(heroes, hero.id);
        let spells = self.day_spells(hero.id);
        let own = day_movement(core, details, spells);
        let (fly, walk) = if start.mode == SearchMode::Plan {
            (start.jumps.fly_level, start.jumps.water_walk_level)
        } else {
            (own.flying, own.water_walking)
        };
        let pricing = |lifts: u8| {
            let movement = Movement {
                flying: fly.filter(|_| lifts & 1 != 0),
                water_walking: walk.filter(|_| lifts & 2 != 0),
                ..own
            };
            Pricing::new(core, part, &hero.army, (&movement, spells))
        };
        let mut occupants: Vec<(Pos, Occupant)> = Vec::new();
        let boats = self
            .boats()
            .filter(|(_, boat)| !boat.aboard && !boat.gone)
            .map(|(_, boat)| (boat.tile, Occupant::Boat));
        let others = self
            .heroes()
            .iter()
            .filter(|other| other.id != hero.id)
            .map(|other| (other.pos, Occupant::Hero(other.owner)));
        for (pos, occupant) in boats.chain(others) {
            let at = occupants.partition_point(|&(tile, _)| tile < pos);
            occupants.insert(at, (pos, occupant));
        }
        let cast = if start.mode == SearchMode::Plan {
            start.jumps
        } else {
            SearchJumps::default()
        };
        let worn = hero_movement(core, details);
        let mut search = Search {
            state: self,
            part,
            map,
            asks,
            start,
            owner: hero.owner,
            own,
            aboard: hero.boat.is_some(),
            at: hero.pos,
            pricings: [pricing(0), pricing(1), pricing(2), pricing(3)],
            land: day_points(
                core,
                part,
                details,
                &hero.army,
                self.visit_marks(hero.id).week_land_points(part),
            ),
            sea: sea_day_points(
                core,
                part,
                details,
                self.lighthouse_points(part, hero.owner),
            ),
            points: start.points as i32,
            bound: start.bound as i32,
            towns: map
                .towns()
                .iter()
                .any(|&town| self.owner(town) == Some(start.player)),
            cast,
            jumps: cast.summon_boat || cast.door || cast.fly || cast.water_walk,
            levels: (fly, walk),
            wings: worn.flying.is_some(),
            boots: worn.water_walking.is_some(),
            occupants,
            spots: Spots {
                region: Bounds {
                    left: 0,
                    top: 0,
                    width: 0,
                    height: 0,
                },
                steps: [0; 8],
                spots: Vec::new(),
            },
        };
        let (width, height) = (map.width(), map.height());
        let window = start.window.unwrap_or(SearchWindow {
            left: 0,
            top: 0,
            right: width,
            bottom: height,
        });
        let (left, top) = (window.left.min(width), window.top.min(height));
        let bounds = Bounds {
            left,
            top,
            width: window.right.clamp(left, width) - left,
            height: window.bottom.clamp(top, height) - top,
        };
        let (left, top) = (left.saturating_sub(MARGIN), top.saturating_sub(MARGIN));
        let right = (bounds.left + bounds.width)
            .saturating_add(MARGIN)
            .min(width);
        let bottom = (bounds.top + bounds.height)
            .saturating_add(MARGIN)
            .min(height);
        let region = Bounds {
            left,
            top,
            width: right - left,
            height: bottom - top,
        };
        search.spots = search.read_ahead(region, bounds);
        let size =
            usize::from(map.levels()) * 2 * usize::from(bounds.width) * usize::from(bounds.height);
        let frame = Frame {
            bounds,
            plane: usize::from(bounds.width) * usize::from(bounds.height),
            cells: vec![Cell::EMPTY; size],
            open: Vec::new(),
            reached: Vec::new(),
            short: false,
            priced: ((0, u64::MAX, 0), [(u64::MAX, 0); 2]),
        };
        Some((search, frame))
    }
}

#[cfg(test)]
mod tests;
