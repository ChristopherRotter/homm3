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
//! Still to come in E31 T2i0: the teleporters (CPU-PTH-006: the start's channel and
//! `pathfinder_pass_through`'s cases 43, 45, 103 and 111, which answer as its default does until
//! then) and mode [`SearchMode::Plan`]'s jumps (CPU-PTH-007: the Door legs, the Fly stretch, the
//! sea spells, the boat node of Summon Boat and the boat marks, the boarding and landing with the
//! day's rest). Not in the engine: the Castle Gate and Town Portal (E09, `AI-STUB-22`), the
//! Sanctuary and the Quest Guard (E12, `AI-STUB-23`).

use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::{HeroId, PlayerId};
use h3_heroes::{HeroDetails, Movement};

use crate::movement::{Pricing, day_movement, day_points, details_of, sea_day_points};
use crate::ruleset::{AdventureRuleset, Effect, Passage, Visit};
use crate::setup::Pos;
use crate::state::{AdventureState, StaticMap, StaticTile, StepVisit};

use marks::{
    BOAT, CASTLE_GATE, DIRECTION_SHIFT, DOOR, DOOR_OFFSET_SHIFT, FLY, FROM_PLACE, NO_MAGIC, PLACE,
    REACHED, STOP, TOWN_PORTAL, VISIT_HERE, WATER_WALK,
};

mod expand;
mod guards;
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
/// other mode. Since E31 T2i0 only the no-magic mark reads the four flags ([`marks::NO_MAGIC`],
/// CPU-PTH-002) and the levels the lifted steps' prices; the jumps themselves are CPU-PTH-007's.
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
    /// 0x4B2B01; CPU-PTH-007).
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

/// The packed column and row of `pos`, a cell's word.
const fn pack(pos: Pos) -> u32 {
    pos.x as u32 | (pos.y as u32) << 16
}

/// The tile of a packed column and row on `level`.
const fn unpack(word: u32, level: u8) -> Pos {
    Pos::new(word as u16, (word >> 16) as u16, level)
}

impl SearchNode {
    /// The node of `pos` stored in `cell` ([`Self::store`]).
    fn load(pos: Pos, cell: &[u32; 8]) -> Self {
        let levels = cell[2];
        Self {
            pos,
            marks: cell[0],
            from: unpack(cell[1], levels as u8),
            guard: (levels >> 16 == 1).then(|| unpack(cell[3], (levels >> 8) as u8)),
            key: cell[4] as i32,
            danger: cell[5] as i32,
            cost: cell[6] as u16,
            cost2: (cell[6] >> 16) as u16,
            points: cell[7] as u16,
        }
    }

    /// The node as a cell's eight words: the marks, the source's column and row, the levels
    /// (the source's, the guard's, whether there is a guard), the guard's column and row, the
    /// key, the danger, the costs, the points. A zeroed cell is a node not reached.
    fn store(&self) -> [u32; 8] {
        let (guard, level) = self.guard.map_or((0, 0), |guard| {
            (pack(guard), 1 << 16 | u32::from(guard.level) << 8)
        });
        [
            self.marks,
            pack(self.from),
            u32::from(self.from.level) | level,
            guard,
            self.key as u32,
            self.danger as u32,
            u32::from(self.cost) | u32::from(self.cost2) << 16,
            u32::from(self.points),
        ]
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
        let column = pos.x.checked_sub(self.left).filter(|&c| c < self.width)?;
        let row = pos.y.checked_sub(self.top).filter(|&r| r < self.height)?;
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
    cells: Vec<[u32; 8]>,
    open: Vec<Entry>,
    reached: Vec<u32>,
    short: bool,
}

impl Frame {
    /// The node stored in `cell`.
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
    /// Any other.
    Other,
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
    /// Whether mode [`SearchMode::Plan`] has a jump (`+0x14` to `+0x17`): the no-magic mark.
    jumps: bool,
    /// The empty boats and the other heroes on the map, by tile in order.
    occupants: Vec<(Pos, Occupant)>,
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
        self.ground(&self.map.at(pos))
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
        let live = self.state.live(pos, self.map.at(pos));
        let entrance = match self.occupant(pos) {
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
        Tile {
            live,
            blocked: (live.blocked && !live.visitable) || (live.visitable && entrance.is_none()),
            entrance,
        }
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
    /// its own source in direction 2 with no price, key or guard.
    // rule: CPU-PTH-001
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
        // The start on a teleporter from mode 3 (0x56B9CF..0x56B9E8): CPU-PTH-006.
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
            if node.marks & BOAT != 0 && self.ground_at(node.pos) != Ground::Water {
                continue;
            }
            let guarded = if node.marks & (FLY | DOOR) == 0
                && self.start.mode < SearchMode::Plan
                && self.state.may_guard(node.pos)
                && node.pos != self.start.from
            {
                self.guarding(node.pos, node.guard)
            } else {
                None
            };
            self.expand(frame, &node, left, guarded);
        }
    }

    /// `pathfinder_relax` 0x4B1A70: relaxes `node`, a step from `source` in `direction` priced
    /// `price`, with `key`, `guard` and the stop flag `stop`, writing into `node` as the
    /// original does (its callers reuse it). Its cost is the source's plus the price, its second
    /// cost its own surcharges over the source's cost plus the new cost (0x4B1A7E..0x4B1AA9).
    /// Refused outside the box (0x4B1AE6); its source the source's tile and [`FROM_PLACE`] the
    /// source's place (0x4B1B0C), its Door offset cleared. Its danger the buffer's at the tile,
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
    fn relax(
        &self,
        frame: &mut Frame,
        source: &SearchNode,
        node: &mut SearchNode,
        (direction, price): (u32, u32),
        (key, guard, stop): (i32, Option<Pos>, bool),
    ) {
        let cost = i32::from(source.cost).wrapping_add(price as i32);
        let cost2 = i32::from(node.cost2) - i32::from(node.cost) + cost;
        let Some(cell) = frame.bounds.index(node.pos, node.marks & PLACE != 0) else {
            return;
        };
        node.from = source.pos;
        node.marks = node.marks & !(FROM_PLACE | DOOR_OFFSET_MASK) | (source.marks & PLACE) << 1;
        // A Door leg's rules (0x4B1B21..0x4B1BAF) come with mode 4's jumps: CPU-PTH-007.
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
        if reached {
            node.marks = node.marks & !NO_MAGIC | stored.marks & NO_MAGIC;
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
    cells: Vec<[u32; 8]>,
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
            .filter(|cell| cell[0] & REACHED != 0)
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
    /// The computer's search from `start` (see [`ComputerSearch`]; CPU-PTH-001 to -005), its
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
        let jumps = start.jumps;
        let search = Search {
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
            jumps: start.mode == SearchMode::Plan
                && (jumps.summon_boat || jumps.door || jumps.fly || jumps.water_walk),
            occupants,
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
        let size =
            usize::from(map.levels()) * 2 * usize::from(bounds.width) * usize::from(bounds.height);
        let frame = Frame {
            bounds,
            cells: vec![[0; 8]; size],
            open: Vec::new(),
            reached: Vec::new(),
            short: false,
        };
        Some((search, frame))
    }
}

#[cfg(test)]
mod tests;
