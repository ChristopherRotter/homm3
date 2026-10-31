//! The adventure state (ADR-0016, E08 T5a): the static map, one block behind an `Arc` that every
//! fork shares and nothing writes after setup (since T12b with the objects' random contents drawn
//! when it is built), the small hot parts in plain arrays (the heroes on the map with their
//! armies), each player's explored tiles as a persistent grid (T8a, on T5b's [`BitGrid`]) and the
//! objects that left the map with the tiles they freed (`occupancy.rs`, T12b); what a player sees
//! of it is the observation's (`observe.rs`, T8b1). Nothing sees the sharing: equality, `Clone`, `Debug` and the canonical hash
//! see content only (ADR-0016 point 2).

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use h3_content::{
    ArtifactTypeId, ContentKey, CreatureFlag, CreatureTypeId, FRINGE_SHOWS_OBJECTS, Formation,
    RADIUS_WAITS_FOR_A_STEP, ResourceId, Ruleset, TerrainId,
};
use h3_contracts::{ArmyStack, HeroId, PlayerId};
use h3_heroes::{HeroDetails, TerrainShares};
use h3_kernel::hash::hash_of;
use h3_kernel::{CanonicalHash, Rng, StateHasher};

use crate::ADVENTURE_STATE_HASH_VERSION;
use crate::command::{
    AdventureCommand, AdventureCommandError, AdventureEvent, AdventureEventKind, AdventureOutcome,
    BattleStart, Combatant, Defender, Meeting, Offer, OfferKind, Sink, Viewers,
};
use crate::fog::{on_fringe, reveal};
use crate::movement::{
    DIRECTIONS, Pricing, day_maximum, details_of, direction, hero_movement, hero_radius,
};
use crate::persistent::{BitGrid, Table};
use crate::ruleset::{
    AdventureRuleset, Effect, MAX_GAIN, MAX_MONSTER_COUNT, MAX_PICKUP_AMOUNT, MAX_PLAYERS,
    ObjectKind, ObjectType, ObjectTypeId, Passage, Pickup, RiverId, RoadId, Visit,
};
use crate::setup::{
    AdventureSetup, ArmyError, Bans, EventSetup, Guards, HeroSetup, Holds, MapSetup, MonsterSetup,
    ObjectSetup, Pos, Scenery, Seat, SetupError, StandError, TileMask,
};

/// How far around a battle's tile the battlefield picture's variant counts the scenery, in tiles
/// along each axis (GAM-BAT-011, E08 T54; the original's 0x41ADB0: x − 3 to x + 3, y − 3 to y + 3).
pub const BACKGROUND_RADIUS: u16 = 3;

/// A tile of the static map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaticTile {
    /// The terrain.
    pub terrain: TerrainId,
    /// The road, if any.
    pub road: Option<RoadId>,
    /// The river, if any.
    pub river: Option<RiverId>,
    /// An object's blocked tile (ADV-OBJ-001).
    pub blocked: bool,
    /// An object's visitable tile (ADV-OBJ-001).
    pub visitable: bool,
    /// The tile's top ground takes the native price away (Cursed Ground; ADV-MOV-004, E08 T40):
    /// a step from it, or the drop after a step onto it, prices its terrain for every army.
    pub no_native: bool,
    /// The tile's top ground is a ground of the sea (Favorable Winds; ADV-SEA-004, E08 T72): a
    /// boat's step from it costs the part's `sea_ground` share (T73).
    pub winds: bool,
    /// A hole lies on the tile (E08 T72): no landing onto it (ADV-SEA-003, T73).
    pub hole: bool,
}

impl StaticTile {
    /// What [`StaticMap::at`] gives for a tile off the map: no terrain the rules know, blocked.
    const OUTSIDE: Self = Self {
        terrain: TerrainId::from_raw(u16::MAX),
        road: None,
        river: None,
        blocked: true,
        visitable: false,
        no_native: false,
        winds: false,
        hole: false,
    };
}

mod boats;
mod draws;
mod economy;
mod events;
mod flags;
mod holds;
mod manage;
mod mines;
mod monsters;
mod occupancy;
mod passage;
mod piles;
mod records;
mod rewards;
mod visits;

pub use boats::Boat;
use boats::Boats;
pub use draws::Resolved;
pub use economy::Weekly;
pub(crate) use economy::allied;
use flags::Flags;
pub(crate) use occupancy::{Added, Occupancy};
pub(crate) use passage::{Mode, Node};
pub use visits::Visits;

/// An object of the static map: its position, footprint, type and contents (ADR-0016 point 4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapObject {
    /// The bottom right tile its masks are relative to.
    pub anchor: Pos,
    /// The tiles no hero enters.
    pub blocked: TileMask,
    /// The tiles a hero visits it from.
    pub visitable: TileMask,
    /// Its object type; none for a blocker. A random resource's is the pile type drawn when the
    /// game was built (ADV-PIK-002).
    pub object_type: Option<ObjectTypeId>,
    /// What a pickup gives, drawn when the game was built (ADV-PIK-001, ADV-PIK-003): each
    /// resource with its amount, the type's multiplier applied; since E08 T61b a chest's gold
    /// (ADV-PIK-005) and a find's resources (ADV-PIK-009 to -012: the Wagon's beside its
    /// artifact too); empty for every other kind.
    pub contents: Vec<(ResourceId, u32)>,
    /// The tiles its picture is drawn on beyond its blocked and visitable ones: a special
    /// ground's (ADV-GRD-001, E08 T20a2a); empty for every other object.
    pub drawn: TileMask,
    /// A wandering monster's data with what the build drew (ADV-MON-001 to ADV-MON-004, E08
    /// T20a2a); none for every other object.
    pub monster: Option<Monster>,
    /// A guarded pile's guards as the map gives them, their stacks in slot order (ADV-PIK-004,
    /// E08 T21a); none for every other object. A battle they survive leaves their stacks in the
    /// state, not here.
    pub guards: Option<Guards>,
    /// What it is among a battle's surroundings (GAM-BAT-011, E08 T54).
    pub scenery: Scenery,
    /// What the map put in it (E08 T61a; [`Holds`]).
    pub holds: Holds,
    /// What the build drew for it beside what the map put in it (E08 T61b; [`Resolved`]).
    pub resolved: Resolved,
}

/// A wandering monster of the static map (ADV-MON-001, E08 T20a2a): the map's data with what the
/// build drew. A battle it survives leaves its count in the state, not here (E08 T20b).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Monster {
    /// The creature: the map's, or a random monster's drawn one (ADV-MON-003).
    pub creature: CreatureTypeId,
    /// The count: the map's, or drawn from the creature's adventure-map range (ADV-MON-004).
    pub count: u32,
    /// The aggression, drawn by the disposition (ADV-MON-002).
    pub aggression: i8,
    /// It never flees.
    pub never_flees: bool,
    /// It does not grow.
    pub no_growth: bool,
    /// What the player that beats it or that it joins gains.
    pub treasure: Vec<(ResourceId, u32)>,
    /// The artifact its treasure holds for the hero that beats it or that it joins (ADV-MON-011,
    /// E08 T61a).
    pub artifact: Option<ArtifactTypeId>,
}

impl MapObject {
    /// The event it is, if it is one (ADV-OBJ-007, E08 T62b2: the setup gives every event, and
    /// only an event, [`Holds::Event`]): never visited from its tile, never observed.
    pub fn event(&self) -> Option<&EventSetup> {
        match &self.holds {
            Holds::Event(event) => Some(event),
            _ => None,
        }
    }

    /// `pos` as `(column, row)` left of and above the anchor, if it is on the anchor's level and
    /// in the block of its masks.
    fn offset_of(&self, pos: Pos) -> Option<(u8, u8)> {
        let column = self.anchor.x.checked_sub(pos.x).map(u8::try_from);
        let row = self.anchor.y.checked_sub(pos.y).map(u8::try_from);
        match (column, row) {
            (Some(Ok(column)), Some(Ok(row))) if self.anchor.level == pos.level => {
                Some((column, row))
            }
            _ => None,
        }
    }

    /// Whether the object blocks `pos`, and whether it is visited from `pos`: neither for an
    /// event, which lies on its tile apart from both (ADV-OBJ-007, E08 T62b2; the original's
    /// placement clears both bits of its tile, 0x505C78, whatever its template's masks say).
    pub fn covers(&self, pos: Pos) -> (bool, bool) {
        if self.event().is_some() {
            return (false, false);
        }
        self.offset_of(pos).map_or((false, false), |(column, row)| {
            (
                self.blocked.contains(column, row),
                self.visitable.contains(column, row),
            )
        })
    }

    /// Whether its picture is drawn on `pos` beyond its blocked and visitable tiles: a special
    /// ground lies on `pos` (ADV-GRD-001, E08 T20a2a).
    pub fn lies_on(&self, pos: Pos) -> bool {
        self.offset_of(pos)
            .is_some_and(|(column, row)| self.drawn.contains(column, row))
    }

    /// Its entrance: the first tile of `map` it is visited from, in [`Self::tiles`]' order (E08
    /// T83b1: an ownable object's, whose reveal and boat tile start there); none for an object
    /// visited from no tile of the map.
    pub fn entrance(&self, map: &StaticMap) -> Option<Pos> {
        self.tiles(map).find(|&pos| self.covers(pos).1)
    }

    /// The tiles of `map` it covers: its blocked, visitable and drawn tiles, row by row (a tile
    /// in several masks once).
    pub fn tiles<'a>(&self, map: &'a StaticMap) -> impl Iterator<Item = Pos> + use<'a> {
        let anchor = self.anchor;
        TileMask::from_bits(self.blocked.bits() | self.visitable.bits() | self.drawn.bits())
            .offsets()
            .filter_map(move |(column, row)| map.offset_pos(anchor, column, row))
    }
}

/// What an object lying on a tile marks there ([`StaticMap::mark_grounds`]).
#[derive(Clone, Copy)]
enum Mark {
    /// A ground on top: whether it takes the native price away, whether it is the sea's.
    Top {
        /// [`StaticTile::no_native`].
        no_native: bool,
        /// [`StaticTile::winds`].
        winds: bool,
    },
    /// A hole.
    Hole,
}

/// What never changes after setup: dimensions, terrain, roads and rivers per tile, the objects
/// and the tiles they cover (ADR-0016 point 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticMap {
    width: u16,
    height: u16,
    levels: u8,
    tiles: Vec<StaticTile>,
    objects: Vec<MapObject>,
    /// The objects visited from each visitable tile, by tile index, in map order (E08 T13b):
    /// what a visit's step looks up. Follows from the objects, so the hash leaves it out.
    visits: BTreeMap<usize, Vec<usize>>,
    /// A bit per tile index: whether a typed object is visited from the tile, so a blocker's
    /// visitable tile, the common case, is turned down without a lookup in [`Self::visits`]
    /// (`bench-check`, E08 T13b). Follows from the objects too.
    typed_visits: Vec<u64>,
    /// The events lying on each tile, by tile index, in map order (ADV-OBJ-007, E08 T62b2; design
    /// §12): an event's template's visitable tile, kept apart from the tiles objects are visited
    /// from, so the movement, the reachable tiles, the occupancy and the views see nothing of it
    /// (the original's placement clears the tile's visitable bit, 0x505C78). Follows from the
    /// objects, so the hash leaves it out.
    events: BTreeMap<usize, Vec<usize>>,
    /// A bit per tile index: whether an event lies on the tile, so a step tests it without a
    /// lookup in [`Self::events`] (`bench-check`). Follows from the objects too.
    event_tiles: Vec<u64>,
    /// A bit per tile index: land on the coast (the map's flag, [`crate::TileSetup::coast`]; E08
    /// T19a), which only a battle's ground reads, so the tiles the hot paths copy stay small
    /// (`bench-check`).
    coast: Vec<u64>,
    /// A bit per tile index: a monster's visitable tile lies within one step on its level, so a
    /// monster may guard the tile (ADV-MON-005, E08 T20a2b); a tile without the bit is guarded by
    /// none, which a move's step and the reachable-tiles query test first (`bench-check`).
    /// Follows from the objects too.
    near_monsters: Vec<u64>,
    /// Whether any object is a monster ([`Self::any_monster`]). Follows from the objects too.
    any_monster: bool,
    /// The map's terrain shares (ADV-MAP-002, E08 T29, T38b), counted once from the tiles, the
    /// objects placed on them and the part's costs: they follow from those, so the hash leaves
    /// them out.
    shares: TerrainShares,
    /// The ownable objects' indices, in object order (E08 T83b1; increment 9's design note §6): a
    /// mine, an abandoned mine, a Lighthouse, a Shipyard ([`ObjectKind::ownable`]); the state's
    /// owners are by their place here. Follows from the objects, so the hash leaves it out.
    ownables: Vec<usize>,
    /// The weekly objects' indices, in object order (E08 T83b1): a Windmill, a Water Wheel, a
    /// Mystical Garden ([`ObjectKind::weekly`]); the state's weekly contents are by their place
    /// here. Follows from the objects too.
    weeklies: Vec<usize>,
    hash: u64,
}

impl StaticMap {
    /// Builds the static map from its setup, checking the dimensions, the tiles and the objects
    /// (level, type, amount, their number), then draws the objects' random contents from `draws`
    /// under the scenario's `bans`.
    fn new(
        core: &Ruleset,
        part: &AdventureRuleset,
        setup: &MapSetup,
        bans: &Bans,
        draws: &mut Rng,
    ) -> Result<Self, SetupError> {
        let params = part.params();
        let in_bounds = |n: u16| (1..=params.max_map_size()).contains(&n);
        if !(in_bounds(setup.width)
            && in_bounds(setup.height)
            && (1..=params.max_levels()).contains(&setup.levels))
        {
            return Err(SetupError::MapSize {
                width: setup.width,
                height: setup.height,
                levels: setup.levels,
            });
        }
        let expected = usize::from(setup.width)
            .checked_mul(usize::from(setup.height))
            .and_then(|n| n.checked_mul(usize::from(setup.levels)));
        if expected != Some(setup.tiles.len()) {
            return Err(SetupError::TileCount(setup.tiles.len()));
        }
        let mut tiles = Vec::with_capacity(setup.tiles.len());
        let mut coast = vec![0; setup.tiles.len().div_ceil(64)];
        for (index, tile) in setup.tiles.iter().enumerate() {
            if usize::from(tile.terrain.raw()) >= core.terrains().len() {
                return Err(SetupError::UnknownTerrain(index));
            }
            if tile.road.is_some_and(|road| part.road(road).is_none()) {
                return Err(SetupError::UnknownRoad(index));
            }
            if tile.river.is_some_and(|river| part.river(river).is_none()) {
                return Err(SetupError::UnknownRiver(index));
            }
            tiles.push(StaticTile {
                terrain: tile.terrain,
                road: tile.road,
                river: tile.river,
                blocked: false,
                visitable: false,
                no_native: false,
                winds: false,
                hole: false,
            });
            if tile.coast {
                coast[index / 64] |= 1 << (index % 64);
            }
        }
        let mut map = Self {
            width: setup.width,
            height: setup.height,
            levels: setup.levels,
            tiles,
            objects: Vec::with_capacity(setup.objects.len()),
            visits: BTreeMap::new(),
            typed_visits: vec![0; setup.tiles.len().div_ceil(64)],
            events: BTreeMap::new(),
            event_tiles: vec![0; setup.tiles.len().div_ceil(64)],
            coast,
            near_monsters: vec![0; setup.tiles.len().div_ceil(64)],
            any_monster: false,
            shares: TerrainShares::default(),
            ownables: Vec::new(),
            weeklies: Vec::new(),
            hash: 0,
        };
        if u16::try_from(setup.objects.len().div_ceil(Occupancy::ROW)).is_err() {
            return Err(SetupError::Objects(setup.objects.len()));
        }
        // What a player sees and is told names a typed object by its type and anchor, never by
        // its index (ADR-0016 point 6), and an offer names its object so: two typed objects of
        // one type on one anchor would share that name (the review of E08 T15: accepting one
        // guard's passage could remove the other). The 160 shipped maps have none (T15a, T21b) but
        // special grounds, which nothing names (never visited, offered or removed): 5 maps stack
        // two of one type on one anchor (E08 T20a2a), so grounds are left out of the check.
        let mut named = BTreeSet::new();
        for (index, object) in setup.objects.iter().enumerate() {
            if object.anchor.level >= map.levels {
                return Err(SetupError::ObjectLevel(index));
            }
            let ty = object
                .object_type
                .map(|id| {
                    part.object_type(id)
                        .ok_or(SetupError::UnknownObjectType(index))
                })
                .transpose()?;
            // rule: ADV-OBJ-002
            let takes = ty.is_some_and(|ty| ty.takes_amount());
            if object
                .amount
                .is_some_and(|a| !(takes && (1..=MAX_PICKUP_AMOUNT).contains(&a)))
            {
                return Err(SetupError::ObjectAmount(index));
            }
            // E08 T72: nothing names a ground of the sea or a hole either.
            let ground = ty.is_some_and(|ty| {
                matches!(
                    ty.kind,
                    ObjectKind::Ground { .. } | ObjectKind::SeaGround | ObjectKind::Hole
                )
            });
            if let Some(id) = object.object_type
                && !ground
                && !named.insert((id.raw(), object.anchor))
            {
                return Err(SetupError::SharedAnchor(index));
            }
            // rule: ADV-MON-001
            let levels = ty.and_then(|ty| match ty.kind {
                ObjectKind::Monster { levels } => Some(levels),
                _ => None,
            });
            if !check_monster(core, part, levels, object.monster.as_ref()) {
                return Err(SetupError::Monster(index));
            }
            // rule: ADV-PIK-004
            // E08 T61a: an artifact, a scroll, Pandora's Box and an event take guards too.
            let guarded = ty.is_some_and(|ty| {
                matches!(
                    ty.kind,
                    ObjectKind::Pickup(Pickup::Resource { .. } | Pickup::OneOf(_))
                        | ObjectKind::Artifact { .. }
                        | ObjectKind::Scroll
                        | ObjectKind::PandoraBox
                        | ObjectKind::Event
                )
            });
            let guards = object
                .guards
                .as_ref()
                .map(|guards| {
                    check_guards(core, part, guarded, guards).ok_or(SetupError::Guards(index))
                })
                .transpose()?;
            if !holds::check_holds(core, part, ty.map(|ty| &ty.kind), &object.holds) {
                return Err(SetupError::Holds(index));
            }
            let event = ty.is_some_and(|ty| matches!(ty.kind, ObjectKind::Event));
            map.place(object, guards, event);
        }
        map.resolve(core, part, &setup.objects, draws);
        draws::resolve_rewards(core, part, bans, &mut map.objects, draws);
        draws::resolve_mines(part, &mut map.objects, draws);
        map.fix_boat_tiles(part);
        map.list_economy(part);
        map.mark_near_monsters();
        map.mark_grounds(part);
        // After the objects, as the original counts after placing its map's objects.
        map.shares = terrain_shares(part, &map.tiles);
        map.hash = hash_of(&map);
        Ok(map)
    }

    /// Fixes each Shipyard's boat tile (ADV-SHY-001, E08 T83b1; increment 9's design note §6), after
    /// every object is placed: from its entrance, the first of its kind's offsets (column, row) on
    /// the entrance's level that is a tile of the map on water that no object blocks or is visited
    /// from, as the original's `shipyard_boat_tiles` 0x500DE0 tests (map item +0xD 0x01 clear, no
    /// object's entrance; it also takes a boat's entrance, and the state's boats are no objects,
    /// ADV-SEA-001; survey `E08-economy.md` §5.1). None found: the Shipyard keeps
    /// [`Resolved::Nothing`] and never builds. Computed once: an object that leaves an earlier
    /// offset later does not move it.
    // rule: ADV-SHY-001
    fn fix_boat_tiles(&mut self, part: &AdventureRuleset) {
        for index in 0..self.objects.len() {
            let object = &self.objects[index];
            let Some(ObjectKind::Shipyard { offsets, .. }) = object
                .object_type
                .and_then(|id| part.object_type(id))
                .map(|ty| &ty.kind)
            else {
                continue;
            };
            let free = |pos: &Pos| {
                self.tile(*pos).is_some_and(|tile| {
                    matches!(part.passage(tile.terrain), Some(Passage::Water(_)))
                        && !tile.blocked
                        && !tile.visitable
                })
            };
            let tile = object.entrance(self).and_then(|entrance| {
                offsets.iter().find_map(|&(column, row)| {
                    let x = entrance.x.checked_add_signed(i16::from(column));
                    let y = entrance.y.checked_add_signed(i16::from(row));
                    x.zip(y)
                        .map(|(x, y)| Pos::new(x, y, entrance.level))
                        .filter(free)
                })
            });
            if let Some(tile) = tile {
                self.objects[index].resolved = Resolved::BoatTile(tile);
            }
        }
    }

    /// Lists the ownable and the weekly objects ([`Self::ownables`], [`Self::weeklies`]).
    fn list_economy(&mut self, part: &AdventureRuleset) {
        for (index, object) in self.objects.iter().enumerate() {
            let kind = object
                .object_type
                .and_then(|id| part.object_type(id))
                .map(|ty| &ty.kind);
            if kind.is_some_and(ObjectKind::ownable) {
                self.ownables.push(index);
            }
            if kind.is_some_and(ObjectKind::weekly) {
                self.weeklies.push(index);
            }
        }
    }

    /// Marks the tiles within one step of a monster's visitable tile, on its level
    /// ([`Self::near_monsters`]), and whether the map holds a monster at all.
    fn mark_near_monsters(&mut self) {
        let mut near = core::mem::take(&mut self.near_monsters);
        let visitable: Vec<Pos> = self
            .objects
            .iter()
            .filter(|o| o.monster.is_some())
            .flat_map(|o| {
                o.visitable
                    .offsets()
                    .filter_map(|(column, row)| self.offset_pos(o.anchor, column, row))
            })
            .collect();
        for at in &visitable {
            for (dx, dy) in DIRECTIONS.iter().copied().chain([(0, 0)]) {
                let x = at.x.checked_add_signed(i16::from(dx));
                let y = at.y.checked_add_signed(i16::from(dy));
                if let (Some(x), Some(y)) = (x, y)
                    && let Some(index) = self.index(Pos::new(x, y, at.level))
                {
                    near[index / 64] |= 1 << (index % 64);
                }
            }
        }
        self.near_monsters = near;
        self.any_monster = !visitable.is_empty();
    }

    /// Marks the tiles whose top ground takes the native price away ([`StaticTile::no_native`],
    /// E08 T40) or is a ground of the sea ([`StaticTile::winds`], E08 T72): on each tile the last
    /// ground in map order whose picture lies on it decides, a special ground or a ground of the
    /// sea, as for a battle's ground ([`AdventureState::battle_ground`]; the original's
    /// `tile_special_ground` 0x4FCE20, which `step_cost` 0x4B1740 asks for Cursed Ground and
    /// Favorable Winds, 0x4B17B3); and the tiles a hole lies on ([`StaticTile::hole`], E08 T72).
    /// Follows from the objects, so the hash leaves it out.
    // rule: ADV-MOV-004
    fn mark_grounds(&mut self, part: &AdventureRuleset) {
        let map = &*self;
        let marks: Vec<(usize, Mark)> = map
            .objects
            .iter()
            .filter_map(|object| {
                let ty = object.object_type.and_then(|id| part.object_type(id))?;
                let mark = match ty.kind {
                    ObjectKind::Ground { no_native, .. } => Mark::Top {
                        no_native,
                        winds: false,
                    },
                    ObjectKind::SeaGround => Mark::Top {
                        no_native: false,
                        winds: true,
                    },
                    ObjectKind::Hole => Mark::Hole,
                    _ => return None,
                };
                Some((object, mark))
            })
            .flat_map(|(object, mark)| {
                object
                    .drawn
                    .offsets()
                    .filter_map(move |(column, row)| map.offset(object.anchor, column, row))
                    .map(move |index| (index, mark))
            })
            .collect();
        for (index, mark) in marks {
            let tile = &mut self.tiles[index];
            match mark {
                Mark::Top { no_native, winds } => {
                    tile.no_native = no_native;
                    tile.winds = winds;
                }
                Mark::Hole => tile.hole = true,
            }
        }
    }

    /// Whether a monster's visitable tile lies within one step of `pos` on its level (a tile off
    /// the map: no).
    pub(crate) fn near_monster(&self, pos: Pos) -> bool {
        self.any_monster
            && self
                .index(pos)
                .is_some_and(|index| self.near_monsters[index / 64] >> (index % 64) & 1 == 1)
    }

    /// Whether a monster stands on the map (one that left it counts): when none does, no tile is
    /// guarded, which the reachable-tiles query tests once (`bench-check`).
    pub(crate) const fn any_monster(&self) -> bool {
        self.any_monster
    }

    /// Draws the objects' random contents (ADR-0016 point 4, ADR-0017 point 4) in the original's
    /// two walks, each in object order: first each random resource's pile type (ADV-PIK-002: one
    /// of its types with equal chances), then each pile's amount where the map fixes none and each
    /// paired pickup's amount and resource (ADV-PIK-001, ADV-PIK-003: the amount first). Piles
    /// take the map's amount, or the drawn one, times the multiplier. Then the monsters
    /// ([`Self::resolve_monsters`]); the rewards come after both (`draws::resolve_rewards`, E08
    /// T61b).
    // rule: ADV-PIK-001, ADV-PIK-002, ADV-PIK-003
    fn resolve(
        &mut self,
        core: &Ruleset,
        part: &AdventureRuleset,
        setup: &[ObjectSetup],
        draws: &mut Rng,
    ) {
        let kind = |object: &MapObject| {
            object
                .object_type
                .and_then(|id| part.object_type(id))
                .map(|ty| &ty.kind)
        };
        for object in &mut self.objects {
            if let Some(ObjectKind::Pickup(Pickup::OneOf(choices))) = kind(object) {
                object.object_type = pick(choices, draws).copied();
            }
        }
        for (object, placed) in self.objects.iter_mut().zip(setup) {
            object.contents = match kind(object) {
                Some(ObjectKind::Pickup(Pickup::Resource {
                    resource,
                    amount: [low, high],
                    multiplier,
                })) => {
                    let amount = placed
                        .amount
                        .unwrap_or_else(|| draws.range_inclusive(*low, *high));
                    vec![(*resource, amount.saturating_mul(multiplier.get()))]
                }
                Some(ObjectKind::Pickup(Pickup::Paired {
                    amount: [low, high],
                    resources,
                    paired,
                    multiplier,
                })) => {
                    let amount = draws.range_inclusive(*low, *high);
                    pick(resources, draws).map_or_else(Vec::new, |&resource| {
                        vec![
                            (resource, amount),
                            (*paired, amount.saturating_mul(multiplier.get())),
                        ]
                    })
                }
                _ => Vec::new(),
            };
        }
        self.resolve_monsters(core, part, setup, draws);
    }

    /// Draws the monsters' random parts (E08 T20a2a) in three walks, each in object order, after
    /// the pickups' (E08 design §3; the original draws the aggression when it loads the map, the
    /// random creatures and then the counts when a game starts, survey `E08-battles.md` §1.6):
    /// each random monster's creature, with equal chances among its levels' creatures
    /// (ADV-MON-003, [`AdventureRuleset::random_monsters`]); each count of 0 from the creature's
    /// adventure-map range, at least 1 (ADV-MON-004); each aggression between its disposition's
    /// bounds, without a draw when they are equal (ADV-MON-002: the original's compliant and
    /// savage monsters).
    // rule: ADV-MON-002, ADV-MON-003, ADV-MON-004
    fn resolve_monsters(
        &mut self,
        core: &Ruleset,
        part: &AdventureRuleset,
        setup: &[ObjectSetup],
        draws: &mut Rng,
    ) {
        let creatures: Vec<Option<CreatureTypeId>> = self
            .objects
            .iter()
            .zip(setup)
            .map(|(object, placed)| {
                let monster = placed.monster.as_ref()?;
                match object
                    .object_type
                    .and_then(|id| part.object_type(id))
                    .map(|ty| &ty.kind)
                {
                    Some(ObjectKind::Monster {
                        levels: Some(levels),
                    }) => {
                        let pool: Vec<_> = part.random_monsters(core, *levels).collect();
                        pick(&pool, draws).copied()
                    }
                    _ => monster.creature,
                }
            })
            .collect();
        let counts: Vec<u32> = creatures
            .iter()
            .zip(setup)
            .map(|(creature, placed)| match (creature, &placed.monster) {
                (Some(creature), Some(monster)) if monster.count == 0 => {
                    let [low, high] = core
                        .creature(*creature)
                        .map_or([1, 1], |info| info.adventure_count);
                    draws
                        .range_inclusive(u32::from(low), u32::from(high))
                        .max(1)
                }
                (_, monster) => monster.as_ref().map_or(0, |m| m.count),
            })
            .collect();
        let bounds = part.params().monsters();
        for (((object, placed), creature), count) in self
            .objects
            .iter_mut()
            .zip(setup)
            .zip(creatures)
            .zip(counts)
        {
            let (Some(monster), Some(creature)) = (&placed.monster, creature) else {
                continue;
            };
            let [low, high] = bounds
                .and_then(|b| b.aggression(monster.disposition))
                .unwrap_or_default();
            let aggression = if low == high {
                low
            } else {
                let span = u32::from(high.abs_diff(low));
                let offset = u8::try_from(draws.range_inclusive(0, span)).unwrap_or(u8::MAX);
                low.saturating_add_unsigned(offset)
            };
            object.monster = Some(Monster {
                creature,
                count,
                aggression,
                never_flees: monster.never_flees,
                no_growth: monster.no_growth,
                treasure: monster.treasure.clone(),
                artifact: monster.artifact,
            });
        }
    }

    /// Marks the tiles an object covers; tiles of its footprint off the map are ignored
    /// (ADV-OBJ-001). An `event` blocks no tile, and its visitable tiles are the tiles it lies on,
    /// no tile an object is visited from (ADV-OBJ-007, E08 T62b2: the original's placement clears
    /// the blocked and the visitable bit of its tile, 0x505C78; every shipped map's event has a
    /// template blocking its tile).
    // rule: ADV-OBJ-001, ADV-OBJ-007
    fn place(&mut self, object: &ObjectSetup, guards: Option<Guards>, event: bool) {
        for (column, row) in object.blocked.offsets().filter(|_| !event) {
            if let Some(index) = self.offset(object.anchor, column, row) {
                self.tiles[index].blocked = true;
            }
        }
        for (column, row) in object.visitable.offsets() {
            if let Some(index) = self.offset(object.anchor, column, row) {
                if event {
                    self.events
                        .entry(index)
                        .or_default()
                        .push(self.objects.len());
                    self.event_tiles[index / 64] |= 1 << (index % 64);
                    continue;
                }
                self.tiles[index].visitable = true;
                self.visits
                    .entry(index)
                    .or_default()
                    .push(self.objects.len());
                if object.object_type.is_some() {
                    self.typed_visits[index / 64] |= 1 << (index % 64);
                }
            }
        }
        self.objects.push(MapObject {
            anchor: object.anchor,
            blocked: object.blocked,
            visitable: object.visitable,
            object_type: object.object_type,
            contents: Vec::new(),
            drawn: object.drawn,
            monster: None,
            guards,
            scenery: object.scenery,
            holds: object.holds.clone(),
            resolved: Resolved::Nothing,
        });
    }

    /// The index of the tile `column` columns left of and `row` rows above `anchor`, if it is
    /// on the map.
    fn offset(&self, anchor: Pos, column: u8, row: u8) -> Option<usize> {
        self.offset_pos(anchor, column, row)
            .and_then(|pos| self.index(pos))
    }

    /// The tile `column` columns left of and `row` rows above `anchor`, if it is on the map.
    fn offset_pos(&self, anchor: Pos, column: u8, row: u8) -> Option<Pos> {
        let x = anchor.x.checked_sub(u16::from(column))?;
        let y = anchor.y.checked_sub(u16::from(row))?;
        let pos = Pos::new(x, y, anchor.level);
        self.index(pos).map(|_| pos)
    }

    /// The index of a tile in [`Self::tiles`], if it is on the map.
    pub fn index(&self, pos: Pos) -> Option<usize> {
        (pos.x < self.width && pos.y < self.height && pos.level < self.levels).then(|| {
            (usize::from(pos.level) * usize::from(self.height) + usize::from(pos.y))
                * usize::from(self.width)
                + usize::from(pos.x)
        })
    }

    /// Columns.
    pub const fn width(&self) -> u16 {
        self.width
    }

    /// Rows.
    pub const fn height(&self) -> u16 {
        self.height
    }

    /// Levels.
    pub const fn levels(&self) -> u8 {
        self.levels
    }

    /// Every tile, level by level, row by row, column by column.
    pub fn tiles(&self) -> &[StaticTile] {
        &self.tiles
    }

    /// The map's terrain shares the original's choice between two offers weighs Pathfinding and
    /// Navigation by (ADV-MAP-002, HER-LVL-006; E08 T29).
    pub const fn terrain_shares(&self) -> TerrainShares {
        self.shares
    }

    /// Whether the tile at `pos` is land on the coast (the map's flag; E08 T19a); a tile off the
    /// map is not.
    pub fn is_coast(&self, pos: Pos) -> bool {
        self.index(pos)
            .is_some_and(|index| self.coast[index / 64] >> (index % 64) & 1 == 1)
    }

    /// The tile at `pos`, if it is on the map.
    pub fn tile(&self, pos: Pos) -> Option<&StaticTile> {
        self.index(pos).map(|index| &self.tiles[index])
    }

    /// The tile at `pos`, or a blocked tile of no known terrain off the map (for positions the
    /// state already checked).
    pub(crate) fn at(&self, pos: Pos) -> StaticTile {
        self.tile(pos).copied().unwrap_or(StaticTile::OUTSIDE)
    }

    /// The objects, in the setup's order.
    pub fn objects(&self) -> &[MapObject] {
        &self.objects
    }

    /// The ownable objects' indices, in object order (E08 T83b1): the state's owners are by their
    /// place here ([`AdventureState::owner`]).
    pub fn ownables(&self) -> &[usize] {
        &self.ownables
    }

    /// The place of the object at `index` among the ownable ones, if it is one.
    pub fn ownable_index(&self, index: usize) -> Option<usize> {
        self.ownables.binary_search(&index).ok()
    }

    /// The weekly objects' indices, in object order (E08 T83b1): the state's weekly contents are
    /// by their place here ([`AdventureState::weekly`]).
    pub fn weeklies(&self) -> &[usize] {
        &self.weeklies
    }

    /// The place of the object at `index` among the weekly ones, if it is one.
    pub fn weekly_index(&self, index: usize) -> Option<usize> {
        self.weeklies.binary_search(&index).ok()
    }

    /// The objects visited from `pos`, by index in map order, if a typed one is among them (none
    /// for a tile only blockers are visited from, or one off the map). A visit concerns the last
    /// of them still on the map ([`AdventureState::visit_at`], E08 T14a).
    pub fn visited_from(&self, pos: Pos) -> &[usize] {
        self.index(pos)
            .filter(|&index| self.typed_visits[index / 64] >> (index % 64) & 1 == 1)
            .and_then(|index| self.visits.get(&index))
            .map_or(&[], Vec::as_slice)
    }

    /// The events lying on `pos`, by index in map order (ADV-OBJ-007, E08 T62b2; none off the
    /// map): a step onto the tile fires the last of them still on the map
    /// ([`AdventureState::event_at`]).
    pub fn events_at(&self, pos: Pos) -> &[usize] {
        self.index(pos)
            .filter(|&index| self.event_tiles[index / 64] >> (index % 64) & 1 == 1)
            .and_then(|index| self.events.get(&index))
            .map_or(&[], Vec::as_slice)
    }

    /// The canonical hash of the static map, computed once when it is built.
    pub const fn content_hash(&self) -> u64 {
        self.hash
    }
}

impl CanonicalHash for StaticMap {
    /// Width, height, levels; every tile (terrain, road, river; since v8 whether it is on the
    /// coast); every object (anchor, blocked and visitable masks; since v4 its object type and
    /// contents; since v9 its drawn mask and, as 0 for none or 1 and its fields in order, its
    /// monster: creature, count, aggression, never flees, does not grow, the treasure as length
    /// and entries; since v10, as 0 for none or 1 and its fields, its guards: whether they ask,
    /// the stacks as length and slot, creature and count each; since v12 its scenery, 0 other, 1
    /// ownable, 2 a tree, 3 a mountain, 4 dead vegetation; since v13 a monster's treasure artifact
    /// after its treasure, and after the scenery what the object holds, [`Holds`]' encoding; since
    /// v14 after it what the build drew, [`Resolved`]'s). The per-tile object flags follow from
    /// the objects.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u16(self.width);
        h.write_u16(self.height);
        h.write_u8(self.levels);
        h.write_len(self.tiles.len());
        for (index, tile) in self.tiles.iter().enumerate() {
            tile.terrain.hash_into(h);
            tile.road.hash_into(h);
            tile.river.hash_into(h);
            h.write_bool(self.coast[index / 64] >> (index % 64) & 1 == 1);
        }
        h.write_len(self.objects.len());
        for object in &self.objects {
            object.hash_into(h);
        }
    }
}

impl CanonicalHash for MapObject {
    /// Its anchor, blocked and visitable masks, object type and contents, drawn mask, monster,
    /// guards, scenery, what it holds and what the build drew ([`StaticMap`]'s encoding of an
    /// object; since adventure state hash v19, E08 T83b2a, an added object's too).
    fn hash_into(&self, h: &mut StateHasher) {
        self.anchor.hash_into(h);
        h.write_u64(self.blocked.bits());
        h.write_u64(self.visitable.bits());
        self.object_type.hash_into(h);
        h.write_len(self.contents.len());
        for (resource, amount) in &self.contents {
            resource.hash_into(h);
            h.write_u32(*amount);
        }
        h.write_u64(self.drawn.bits());
        match &self.monster {
            None => h.write_u8(0),
            Some(monster) => {
                h.write_u8(1);
                monster.creature.hash_into(h);
                h.write_u32(monster.count);
                h.write_i32(i32::from(monster.aggression));
                h.write_bool(monster.never_flees);
                h.write_bool(monster.no_growth);
                h.write_len(monster.treasure.len());
                for (resource, amount) in &monster.treasure {
                    resource.hash_into(h);
                    h.write_u32(*amount);
                }
                monster.artifact.hash_into(h);
            }
        }
        match &self.guards {
            None => h.write_u8(0),
            Some(guards) => {
                h.write_u8(1);
                h.write_bool(guards.question);
                h.write_len(guards.army.len());
                for stack in &guards.army {
                    h.write_u8(stack.slot);
                    stack.creature.hash_into(h);
                    h.write_u32(stack.count);
                }
            }
        }
        h.write_u8(match self.scenery {
            Scenery::Other => 0,
            Scenery::Ownable => 1,
            Scenery::Tree => 2,
            Scenery::Mountain => 3,
            Scenery::DeadVegetation => 4,
        });
        self.holds.hash_into(h);
        self.resolved.hash_into(h);
    }
}

impl CanonicalHash for Pos {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u16(self.x);
        h.write_u16(self.y);
        h.write_u8(self.level);
    }
}

/// What a player's step onto an object's visitable tile meets (ADV-OBJ-003): the object, by index
/// in the static map, its type, the tile it is visited from, how it is visited, whether the player
/// passes it instead (a border gate whose key it holds, ADV-VIS-005: the step does not visit, the
/// tile is crossed like any other), whether it may be from above (ADV-OBJ-004) and what the visit
/// does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StepVisit<'p> {
    pub(crate) object: usize,
    pub(crate) object_type: ObjectTypeId,
    pub(crate) tile: Pos,
    pub(crate) mode: Visit,
    pub(crate) pass: bool,
    pub(crate) from_above: bool,
    pub(crate) effect: Effect<'p>,
}

/// What a step meets that ends the move (ADV-OBJ-003): an object's visit, or another hero, by
/// index among the heroes on the map, visited from the tile before: another player's in a battle
/// (GAM-BAT-004, E08 T19a), an own one in a meeting (GAM-MET-001, T50); since E08 T73 an empty
/// boat boarded (ADV-SEA-002) and a landing (ADV-SEA-003), each by boat index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Meets<'p> {
    Object(StepVisit<'p>),
    Hero(usize),
    /// The empty boat the hero boards, on the step's tile.
    Board(u32),
    /// The free coastal tile the hero lands on, leaving its boat on the water tile it left.
    Land(u32),
}

impl Meets<'_> {
    /// Whether the hero steps onto the tile: an entered object's, a boat's, a landing's; not a
    /// neighbour-visited object's nor another hero's.
    fn enters(self) -> bool {
        match self {
            Self::Object(visit) => visit.mode == Visit::Enter,
            Self::Hero(_) => false,
            Self::Board(_) | Self::Land(_) => true,
        }
    }
}

/// A hero on the adventure map (ADR-0016 point 3): its id, owner, tile, army and movement points.
/// The hero's details stay in `h3-heroes`, kept by `h3-game`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeroOnMap {
    /// The hero.
    pub id: HeroId,
    /// The player owning it.
    pub owner: PlayerId,
    /// The tile it stands on.
    pub pos: Pos,
    /// Its army, by slot.
    pub army: Vec<ArmyStack>,
    /// Its army's formation in battle (ADR-0016 point 1, E08 T18b).
    pub formation: Formation,
    /// Its tactics flag (ADR-0016 point 3, ADV-ARM-006; E08 T50): on at the start (the
    /// original's +0x48 = 2); off, it denies the tactics phase it would get (GAM-BAT-009).
    pub tactics: bool,
    /// The movement points it has left today.
    pub movement: u32,
    /// The boat it is in, by boat index (ADR-0016 point 3, ADV-SEA-001; E08 T72): none on land.
    pub boat: Option<u32>,
    /// The day's maximum (ADR-0016 point 3, E08 T72): the points its turn started with on its
    /// surface, set by the start of the game and of its turn (ADV-MOV-002) and, from T73, by a
    /// boarding and a landing.
    pub max_movement: u32,
}

impl HeroOnMap {
    /// The hero as a side of a battle on the map.
    pub(crate) fn combatant(&self) -> Combatant {
        Combatant {
            hero: self.id,
            owner: self.owner,
            army: self.army.clone(),
            formation: self.formation,
            tactics: self.tactics,
        }
    }
}

/// The adventure context's state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdventureState {
    map: Arc<StaticMap>,
    players: u8,
    heroes: Vec<HeroOnMap>,
    /// Each player's explored tiles, by player id (ADR-0016 point 5).
    explored: Vec<BitGrid>,
    /// The objects that left the map and the tiles they freed (ADR-0016 point 1); none until an
    /// object leaves, so a fork and a tile's read cost nothing more before (`bench-check`).
    occupancy: Option<Arc<Occupancy>>,
    /// The objects added in play, after the static map's in map order, with the grids of their
    /// tiles (E08 T83b2a; increment 9's design note §6): none until one is added (T85a's creature
    /// month), so a read that asks for them costs one test and a fork nothing more before
    /// (`bench-check`), then one block the forks share.
    added: Option<Arc<Added>>,
    /// The per-player state of object types (ADR-0016 point 1): the Cartographer types bought
    /// (ADV-VIS-001, E08 T14b) and the keys held (ADV-VIS-003, T15a), as the original's flags.
    /// None until a first purchase or key, then one block the forks share (a write copies it), so
    /// a fork costs nothing more before and a reference count after (`bench-check`).
    flags: Option<Arc<Flags>>,
    /// The game's difficulty, from 0 (easy; E08 T20a2a): a static input the monsters' answer
    /// reads (ADV-MON-007, E08 T20b).
    difficulty: u8,
    /// Each player's seat by player id (E08 T62b2): a static input an event's step reads
    /// (ADV-OBJ-007); the entries past the players unused. An array, so a fork copies it.
    seats: [Seat; MAX_PLAYERS as usize],
    /// The armies battles left on objects, by object index (E08 T20a2a): a monster's survivors
    /// (ADV-MON-010, E08 T20b), a guarded object's (E08 T21b). Since E08 T61b a persistent
    /// two-level table (increment 7's design note §10): a fork shares it, a write copies at most
    /// three small tables, so an army left on every object of the XL map costs a write no more than
    /// one does.
    army_left: Table<Vec<ArmyStack>>,
    /// What visits left on the objects (E08 T61b; [`Visits`], increment 7's design note §10): none
    /// until a visit writes it (E08 T62), then one block the forks share, as [`Self::flags`].
    visits: Option<Arc<Visits>>,
    /// The boats (ADR-0016 point 1, ADV-SEA-001; E08 T72): none for a map without boats, so a
    /// fork costs nothing more (`bench-check`).
    boats: Option<Boats>,
    /// The ownable objects' owners by their place among the static map's ownable objects
    /// (ADV-OWN-002, E08 T83b1; increment 9's design note §6; `economy.rs`): none for a map
    /// without ownable objects, so a fork of one costs nothing more (`bench-check`), else one
    /// block the forks share, which a flag copies (T84).
    owners: Option<Arc<[Option<PlayerId>]>>,
    /// The weekly objects' contents and marks by their place among the static map's weekly
    /// objects (ADV-WEK-001 to -003, E08 T83b1): none for a map without weekly objects, else one
    /// block the forks share, which a visit or the week's step copies (T84, T85a).
    weekly: Option<Arc<[Weekly]>>,
    /// Each wandering monster's fraction of a creature (its sixteenths, the monsters' growth unit;
    /// ADV-MON-013, E08 T83b2a; increment 9's design note §6), by object index, none for 0: what
    /// the week's growth leaves below a whole creature (T85a writes it). A persistent table, as
    /// [`Self::army_left`]: empty, a fork copies nothing.
    fractions: Table<u8>,
}

impl AdventureState {
    /// Builds the state from a setup, checking every invariant commands keep: the map's bounds
    /// and tiles, the objects, the players' bound, and each hero's owner, tile (ADV-MAP-001) and
    /// army. The objects' random contents are drawn from `draws`, the game's setup stream
    /// (ADR-0017 point 4); every draw is valid, so whether a setup is valid does not depend on
    /// them. The heroes are kept by id, their stacks by slot, without movement points, then the
    /// boats (since E08 T72; [`SetupError::Boat`]), each empty; nothing is explored, and every
    /// object is on the map. The game gives the heroes their levels at load,
    /// then starts the game ([`Self::start_game`]: the points and the reveals, ADR-0017 point 1).
    ///
    /// # Errors
    ///
    /// The first [`SetupError`] found, in that order.
    pub fn new(
        core: &Ruleset,
        part: &AdventureRuleset,
        setup: &AdventureSetup,
        draws: &mut Rng,
    ) -> Result<Self, SetupError> {
        let map = StaticMap::new(core, part, &setup.map, &setup.bans, draws)?;
        if !(1..=part.params().max_players()).contains(&setup.players) {
            return Err(SetupError::Players(setup.players));
        }
        if setup.seats.len() != usize::from(setup.players) {
            return Err(SetupError::Seats(setup.seats.len()));
        }
        // rule: ADV-OWN-002
        for (index, object) in setup.map.objects.iter().enumerate() {
            let ownable = object
                .object_type
                .and_then(|id| part.object_type(id))
                .is_some_and(|ty| ty.kind.ownable());
            if object
                .owner
                .is_some_and(|owner| !(ownable && owner.index() < usize::from(setup.players)))
            {
                return Err(SetupError::ObjectOwner(index));
            }
        }
        let mut seats = [Seat::default(); MAX_PLAYERS as usize];
        seats[..setup.seats.len()].copy_from_slice(&setup.seats);
        let owners = economy::owners_at_start(&map, &setup.map.objects);
        let weekly = economy::first_week(part, &map, draws);
        let unexplored = BitGrid::new(map.width(), map.height(), map.levels());
        let mut state = Self {
            map: Arc::new(map),
            players: setup.players,
            heroes: Vec::with_capacity(setup.heroes.len()),
            explored: vec![unexplored; usize::from(setup.players)],
            occupancy: None,
            added: None,
            flags: None,
            difficulty: setup.difficulty,
            seats,
            army_left: Table::new(),
            visits: None,
            boats: None,
            owners,
            weekly,
            fractions: Table::new(),
        };
        for hero in &setup.heroes {
            state.place(core, part, hero)?;
        }
        state.boats = Boats::new(part, &state.map, &setup.map.boats)?;
        Ok(state)
    }

    /// The start of the game (ADR-0017 point 1, E08 T40), after the game gave the heroes their
    /// levels at load: each hero gets the day's movement points (ADV-MOV-002; since E08 T72 on
    /// its surface, [`day_maximum`], which is also its day's maximum) and
    /// reveals the tiles around it to its owner with its radius (ADV-FOG-003), both from its
    /// details in `heroes`, the game's store. The original's order: `placed_hero_setup` 0x4D8B30
    /// sets the points after the levels (survey `E08-progression.md` §3.2), `placed_heroes_setup`
    /// 0x4CAE10 reveals after it (0x4CAF74, §3.1). Since E08 T83b1 every object owned at load then
    /// reveals the tiles within the part's flag radius of its entrance to its owner
    /// (`reveal_owned`, ADV-OWN-002; ADR-0016 point 5). The game calls it once, before day 1's
    /// start-of-day step.
    // rule: ADV-MOV-002, ADV-FOG-001, ADV-FOG-003
    pub fn start_game(
        &mut self,
        core: &Ruleset,
        part: &AdventureRuleset,
        heroes: &[Arc<HeroDetails>],
    ) {
        for hero in &mut self.heroes {
            let details = details_of(heroes, hero.id);
            hero.max_movement = day_maximum(core, part, details, hero);
            hero.movement = hero.max_movement;
        }
        self.reveal_around_heroes(core, part, heroes);
        self.reveal_owned(part);
    }

    /// The adventure's start-of-day step (ADR-0017 point 3, E08 T40), after the calendar and
    /// before the first group's turn, also on day 1: under `strict-fixed` (`ADV-QUIRK-18` off)
    /// every hero on the map reveals the tiles around it to its owner with its radius, so a
    /// radius grown in the day shows by the next day for a hero that does not move. The original
    /// (`faithful`) reveals nothing here: its new day reveals around a player's heroes only after
    /// a Cover of Darkness hid tiles (`new_day` 0x4C7FE0, `cover_of_darkness_hide` 0x4C7BA0; survey
    /// `E08-hero-movement.md` §4.2), which comes with towns (E09). No event, as the start's and the
    /// steps' reveals have none of their own: the owner's observation shows the explored tiles.
    // rule: ADV-FOG-003
    pub fn start_day(
        &mut self,
        core: &Ruleset,
        part: &AdventureRuleset,
        heroes: &[Arc<HeroDetails>],
    ) {
        if !core.quirks().is_enabled(RADIUS_WAITS_FOR_A_STEP) {
            self.reveal_around_heroes(core, part, heroes);
        }
    }

    /// Each hero reveals the tiles around it to its owner with its radius (ADV-FOG-001).
    fn reveal_around_heroes(
        &mut self,
        core: &Ruleset,
        part: &AdventureRuleset,
        heroes: &[Arc<HeroDetails>],
    ) {
        for hero in &self.heroes {
            let radius = hero_radius(core, part, details_of(heroes, hero.id));
            reveal(
                &mut self.explored[usize::from(hero.owner.raw())],
                hero.pos,
                radius,
            );
        }
    }

    /// Checks and places one hero of the setup, without movement points ([`Self::start_game`]
    /// gives them).
    fn place(
        &mut self,
        core: &Ruleset,
        part: &AdventureRuleset,
        hero: &HeroSetup,
    ) -> Result<(), SetupError> {
        let Err(at) = self.heroes.binary_search_by_key(&hero.id, |h| h.id) else {
            return Err(SetupError::DuplicateHero(hero.id));
        };
        if hero.owner.raw() >= self.players {
            return Err(SetupError::UnknownOwner(hero.id));
        }
        self.check_stand(part, hero.pos)
            .map_err(|error| SetupError::HeroTile {
                hero: hero.id,
                error,
            })?;
        let army = check_army(core, part, &hero.army).map_err(|error| SetupError::Army {
            hero: hero.id,
            error,
        })?;
        self.heroes.insert(
            at,
            HeroOnMap {
                id: hero.id,
                owner: hero.owner,
                pos: hero.pos,
                army,
                formation: hero.formation,
                tactics: true,
                movement: 0,
                boat: None,
                max_movement: 0,
            },
        );
        Ok(())
    }

    /// The start of `player`'s turn (ADR-0017 point 3), a step only the game triggers: each of
    /// its heroes gets the day's movement points for its army and its details in `heroes`, the
    /// game's store, on its surface ([`day_maximum`], E08 T72: the land's or, in a boat, the
    /// sea's), which is also its day's maximum; what was left is lost. A gain in the day (a level-up's
    /// Logistics, an artifact put on) shows here first (survey `E08-hero-movement.md` §1.4).
    // rule: ADV-MOV-002
    pub fn start_turn(
        &mut self,
        core: &Ruleset,
        part: &AdventureRuleset,
        heroes: &[Arc<HeroDetails>],
        player: PlayerId,
    ) {
        for hero in self.heroes.iter_mut().filter(|h| h.owner == player) {
            let details = details_of(heroes, hero.id);
            hero.max_movement = day_maximum(core, part, details, hero);
            hero.movement = hero.max_movement;
        }
    }

    /// Whether a hero may stand on `pos` (ADV-MAP-001): a tile of the map whose terrain is
    /// walked on land, which no object blocks unless an object is visited from it, and no other
    /// hero stands on. A visitable tile is allowed even where a mask also blocks it (a hero may
    /// stand on a town's entrance, which the town's masks mark both ways), unless its object is
    /// visited from a neighbouring tile: no move ever enters such a tile (ADV-OBJ-003), so a setup
    /// may not place a hero there either (E08 T14a). A border gate's tile is such a tile: only its
    /// key's holders cross it (ADV-VIS-005, T15b), and no player holds a key at a setup; a hero a
    /// move stops on a gate leaves it as any tile.
    ///
    /// # Errors
    ///
    /// Why it may not, checked in the order of [`StandError`].
    // rule: ADV-MAP-001
    pub fn check_stand(&self, part: &AdventureRuleset, pos: Pos) -> Result<(), StandError> {
        self.check_tile(part, pos, None)?;
        match self.tile_object(part, pos) {
            Some((_, _, ty)) if ty.visit == Visit::Neighbour => Err(StandError::Visited),
            _ => Ok(()),
        }
    }

    /// [`Self::check_stand`] for `mover`, which may stand where it already stands; the tile.
    fn check_tile(
        &self,
        part: &AdventureRuleset,
        pos: Pos,
        mover: Option<HeroId>,
    ) -> Result<StaticTile, StandError> {
        let tile = self.live(pos, *self.map.tile(pos).ok_or(StandError::Outside)?);
        if !enterable_terrain(part, &tile) {
            return Err(StandError::Terrain);
        }
        if tile.blocked && !tile.visitable {
            return Err(StandError::Blocked);
        }
        if self
            .heroes
            .iter()
            .any(|h| h.pos == pos && Some(h.id) != mover)
        {
            return Err(StandError::Occupied);
        }
        Ok(tile)
    }

    /// Applies a command of `player` (ADR-0017 point 1: the game has checked that the player's
    /// turn is running): validated against the state, then applied; the events go to `sink`, and
    /// what the player gains beyond the map is the outcome (ADR-0017 point 10). `heroes` is the
    /// game's store of heroes' details by [`HeroId`] (ADR-0016's E08 amendment, decision 25; E08
    /// T20a2b), which the adventure reads only through `h3-heroes`' derivations: the monsters'
    /// answer reads the visitor's (E08 T20b).
    ///
    /// # Errors
    ///
    /// An [`AdventureCommandError`]; the state is then unchanged and no event was emitted.
    pub fn apply<S: Sink>(
        &mut self,
        core: &Ruleset,
        part: &AdventureRuleset,
        heroes: &[Arc<HeroDetails>],
        player: PlayerId,
        command: &AdventureCommand,
        sink: &mut S,
    ) -> Result<AdventureOutcome, AdventureCommandError> {
        match command {
            AdventureCommand::MoveHero { hero, path } => {
                self.move_hero(core, part, heroes, (player, *hero), path, sink)
            }
        }
    }

    /// `MoveHero`: the whole path is checked and priced first (ADV-MOV-003 to ADV-MOV-008, on the
    /// tiles the player has explored when it gives the command), then the hero stands on its last
    /// tile with the points left; each step reveals the tiles around the tile it enters
    /// (ADV-FOG-003) and emits its event, seen by the players that see the hero on either of its
    /// tiles (ADV-FOG-004, ADV-FOG-005). A step onto a tile an object is visited from is the
    /// path's last (ADV-OBJ-003) and visits it ([`Self::visit`]): an entered object's tile (the
    /// Cartographer's, E08 T14a) the hero steps onto; toward a neighbour-visited object's (a
    /// pickup's, T13b) the step is paid as if taken (ADV-MOV-006's rule included) and the hero
    /// stays on the tile before. A border gate whose key the player holds is not visited: the step
    /// crosses it as any tile, and the path may go on or end there (ADV-VIS-005, T15b). A step
    /// onto a tile a monster guards (ADV-MON-005) is the path's last too and visits the guard
    /// after any visit of the tile's own object ([`Self::visit_guard`], ADV-MON-006, E08 T20a2b);
    /// the hero's own tile may be guarded: it leaves it. A step that does not enter (toward a
    /// neighbour-visited object or a hero) tests no guard, even from a guarded tile: the original
    /// tests the hero's tile only after a step that changed it (E08 T20b3a). A step toward another
    /// hero visits it from the tile before as well: another player's starts a battle (GAM-BAT-004),
    /// an own one makes a meeting ([`AdventureOutcome::meeting`], GAM-MET-001, E08 T50), which the
    /// game opens.
    ///
    /// A flying or water-walking hero (E08 T41; ADV-MOV-009, ADV-MOV-010) moves by
    /// [`Self::passage`]'s table, each step from the node the step before left it on (the start a
    /// stop node): a step before the last passes over what the hero may pass, else stops (a free
    /// tile, then the next step goes on from there); the last step stops, visits or is rejected
    /// as [`AdventureCommandError::NoStop`]. A flying hero passes guarded tiles and objects on the
    /// way: only its last step's tile is tested for a guard and only its object visited (the
    /// original's move, 0x48147D to 0x4814E1). Each step, a pass included, reveals and emits its
    /// event.
    ///
    /// Events (E08 T62b2; ADV-MOV-011, ADV-OBJ-007): the path is checked and priced as if no event
    /// lay on the map, so neither its acceptance nor its price tells of one; then the first step
    /// onto a tile the hero stands on (a stop, nothing visited) that fires an event
    /// ([`Self::firing_event`]) ends the move there with the points left after it, the rest of
    /// the path dropped ([`AdventureOutcome::dropped`]), and the event fires
    /// ([`Self::fire_event`]); on a tile a monster guards (a flyer stopped there is at its move's
    /// end for the guard test) the guard visits first and the event waits for the guard's visit
    /// ([`AdventureOutcome::event_after`], E08 T63b, `ADV-QUIRK-29`). The hero's start is no step:
    /// a hero that starts on an event does not fire it.
    ///
    /// Boats (E08 T73; ADV-SEA-002 to -005, increment 8's design note §7 and §8): a step onto an
    /// empty boat's tile boards it ([`Self::board`]), priced by the hero's mode on land, from a
    /// stop node, the walk ending there with no guard test; a hero in a boat sails
    /// ([`Mode::Sail`]): each step is a stop priced as a boat's ([`Pricing::sail`]) and tested for
    /// a guard (a water monster attacks at once) and for an event on water as a walker's, a step
    /// onto a free coastal tile lands ([`Self::land`]), the walk ending there and the tile's guards
    /// visiting after the landing; an object or a hero in a boat is visited from the boat's tile.
    /// The drop after a step is [`Pricing::left_by`]'s; a boarding's and a landing's conversion
    /// follows it (D-41).
    // rule: ADV-MOV-006, ADV-MOV-007, ADV-MOV-009, ADV-MOV-010, ADV-FOG-003
    // rule: ADV-OBJ-003, ADV-MON-006, GAM-MET-001, ADV-MOV-011, ADV-OBJ-007
    // rule: ADV-SEA-002, ADV-SEA-003, ADV-SEA-004, ADV-SEA-005
    fn move_hero<S: Sink>(
        &mut self,
        core: &Ruleset,
        part: &AdventureRuleset,
        heroes: &[Arc<HeroDetails>],
        (player, id): (PlayerId, HeroId),
        path: &[Pos],
        sink: &mut S,
    ) -> Result<AdventureOutcome, AdventureCommandError> {
        let index = self
            .heroes
            .binary_search_by_key(&id, |h| h.id)
            .ok()
            .filter(|&index| self.heroes[index].owner == player)
            .ok_or(AdventureCommandError::NotYourHero(id))?;
        if path.is_empty() {
            return Err(AdventureCommandError::EmptyPath);
        }
        let hero = &self.heroes[index];
        let details = details_of(heroes, id);
        let movement = hero_movement(core, details);
        let mode = Mode::of(&movement, hero.boat);
        let pricing = Pricing::new(core, part, &hero.army, &movement);
        let start = hero.pos;
        let (mut at, mut from, mut points) =
            (start, self.live(start, self.map.at(start)), hero.movement);
        let (mut node, mut visit, mut guard) = (Node::Stop, None, None);
        // The first step that fires an event: the step, the event, the points left after it.
        let mut fired = None;
        // A landing's water tile and direction (ADV-SEA-003).
        let mut landing = (start, 0);
        for (step, &next) in path.iter().enumerate() {
            if visit.is_some() || guard.is_some() {
                return Err(AdventureCommandError::PastVisit { step });
            }
            let direction =
                direction(at, next).ok_or(AdventureCommandError::NotNeighbour { step })?;
            let last = step + 1 == path.len();
            let (to, met, kind) = if mode == Mode::Walk {
                let (to, met) = self.check_step(part, player, id, (at, next), direction, step)?;
                (to, met, Node::Stop)
            } else {
                let entry =
                    self.passage(part, (player, id, mode), (at, node), next, direction, step)?;
                match entry.stop {
                    Some(met) if last || !entry.pass => (entry.tile, met, Node::Stop),
                    _ if !last => (entry.tile, None, Node::Pass),
                    _ => return Err(AdventureCommandError::NoStop { step }),
                }
            };
            let cost = pricing.step_by(mode, (&from, &to), direction % 2 == 1, points);
            let left = points
                .checked_sub(cost)
                .ok_or(AdventureCommandError::TooFewPoints { step, cost, points })?;
            points = pricing.left_by(mode, met, &to, left);
            landing = (at, direction);
            if met.is_none_or(Meets::enters) {
                (at, from, node) = (next, to, kind);
                // A flying hero meets a guard only where its move ends (0x48147D); no guard
                // attacks on a boat's tile (0x480108, ADV-SEA-002).
                if (mode != Mode::Fly || last) && !matches!(met, Some(Meets::Board(_))) {
                    guard = self.guard_at(part, next);
                }
            }
            if fired.is_none() && met.is_none() && kind == Node::Stop {
                fired = self
                    .firing_event(player, next)
                    .map(|event| (step, event, points));
            }
            visit = met;
        }
        let walked = if let Some((step, _, left)) = fired {
            (at, points) = (path[step], left);
            &path[..=step]
        } else {
            let stays = visit.is_some_and(|met| !met.enters());
            &path[..path.len() - usize::from(stays)]
        };
        let hero = &mut self.heroes[index];
        hero.pos = at;
        hero.movement = points;
        let radius = hero_radius(core, part, details);
        let mut from = start;
        for &to in walked {
            reveal(&mut self.explored[usize::from(player.raw())], to, radius);
            let (seen_from, seen_to) = (
                self.viewers(core, player, from),
                self.viewers(core, player, to),
            );
            sink.emit(AdventureEvent {
                kind: AdventureEventKind::HeroStepped {
                    hero: id,
                    from: Some(from),
                    to: Some(to),
                    seen_from,
                    seen_to,
                },
                seen_by: seen_from.union(seen_to),
            });
            from = to;
        }
        let rules = (core, part, heroes);
        if let Some((step, event, _)) = fired {
            // On a guarded tile the guard visits first (a flyer stopped there is at its move's
            // end for the guard test, 0x48147D) and the event waits for the guard's visit
            // (0x48169D, 0x481741; E08 T63b, `ADV-QUIRK-29`).
            let outcome = match self.guard_at(part, at) {
                Some(guard) => AdventureOutcome {
                    event_after: Some((id, at)),
                    ..self.visit_monster(rules, id, guard)
                },
                None => self.fire_event(rules, (player, id), (event, at), sink),
            };
            return Ok(AdventureOutcome {
                dropped: path.len() - step - 1,
                ..outcome
            });
        }
        let visited = matches!(visit, Some(Meets::Object(_)));
        let outcome = match visit {
            None => AdventureOutcome::default(),
            Some(Meets::Object(visit)) => self.visit(rules, (player, id), visit, sink),
            Some(Meets::Board(boat)) => {
                self.board(rules, (player, index), boat, sink);
                AdventureOutcome::default()
            }
            Some(Meets::Land(boat)) => {
                let (water, direction) = landing;
                self.land(rules, (player, index), (boat, water, direction), sink);
                AdventureOutcome::default()
            }
            Some(Meets::Hero(other)) if self.heroes[other].owner == player => AdventureOutcome {
                meeting: Some(Meeting {
                    visitor: id,
                    visited: self.heroes[other].id,
                }),
                ..AdventureOutcome::default()
            },
            Some(Meets::Hero(defender)) => AdventureOutcome {
                battle: Some(self.battle_start(part, index, defender)),
                ..AdventureOutcome::default()
            },
        };
        // The guard visits after the tile's own object (ADV-MON-006): once the object's visit is
        // over, with everything it opened (E08 T63b: the game holds it as a rest of the visit,
        // GAM-DEC-007; before it the guard came after an offer's answer, E08 T20b1, a choice's or
        // a University's, T63a, or at once beside what the visit gave). On a tile without an
        // object the guard's visit is the step's, a landing's after the landing (its tile's
        // guards attack once the hero is on land, `hero_lands_from_boat`'s guard_attack; E08
        // T73). A step that does not enter tests no guard (T20b3a), nor a boarding.
        Ok(match guard {
            None => outcome,
            Some(_) if visited => AdventureOutcome {
                guard_after: Some(id),
                ..outcome
            },
            Some(guard) => self.visit_monster(rules, id, guard),
        })
    }

    /// The monster that guards `pos`, by index in the static map, if one does (ADV-MON-005, E08
    /// T20a2b; survey `E08-battles.md` §2.1, the original's 0x41A460): the 3 × 3 block around
    /// `pos` on its level, clipped to the map, column by column from the left and each column
    /// from the top, leaving out the row above when `pos`'s object is not visited from above
    /// (ADV-OBJ-004); the first tile whose object (the last still on the map visited from it,
    /// ADV-OBJ-003) is a monster, on water if and only if `pos` is. Computed from the static map
    /// and the objects that left it, so a removed monster guards nothing.
    // rule: ADV-MON-005
    pub fn guard_of(&self, part: &AdventureRuleset, pos: Pos) -> Option<usize> {
        self.guard_at(part, pos).map(|(index, _)| index)
    }

    /// [`Self::guard_of`] with the guard's object type.
    pub(crate) fn guard_at(
        &self,
        part: &AdventureRuleset,
        pos: Pos,
    ) -> Option<(usize, ObjectTypeId)> {
        if !self.may_guard(pos) {
            return None;
        }
        let water = |at: Pos| {
            matches!(
                part.passage(self.map.at(at).terrain),
                Some(Passage::Water(_))
            )
        };
        let skip_above = self.closed_above(part, pos);
        let at_water = water(pos);
        for dx in -1i16..=1 {
            for dy in -1i16..=1 {
                if skip_above && dy == -1 {
                    continue;
                }
                let (Some(x), Some(y)) =
                    (pos.x.checked_add_signed(dx), pos.y.checked_add_signed(dy))
                else {
                    continue;
                };
                let tile = Pos::new(x, y, pos.level);
                if self.map.tile(tile).is_none() || water(tile) != at_water {
                    continue;
                }
                if let Some((index, id, ty)) = self.tile_object(part, tile)
                    && matches!(ty.kind, ObjectKind::Monster { .. })
                {
                    return Some((index, id));
                }
            }
        }
        None
    }

    /// The battle the hero at `attacker` starts by visiting the hero at `defender` (indices among
    /// the heroes on the map; GAM-BAT-004, E08 T19a): both sides as they stand, fought at the
    /// defender's tile ([`Self::battle_at`]).
    fn battle_start(
        &self,
        part: &AdventureRuleset,
        attacker: usize,
        defender: usize,
    ) -> BattleStart {
        let defending = &self.heroes[defender];
        self.battle_at(
            part,
            &self.heroes[attacker],
            Defender::Hero(defending.combatant()),
            defending.pos,
        )
    }

    /// The battle `attacker`, a hero on the map, starts against `defender` at `tile` (GAM-BAT-001,
    /// E08 T19a): fought where [`Self::battle_ground`] says; since E08 T54 with the battlefield
    /// picture's variant of the tile's surroundings ([`Self::background`], GAM-BAT-011) and the
    /// special ground of the attacking hero's own tile ([`Self::special_ground`], GAM-BAT-013);
    /// since E08 T74b at sea when either side's hero is in a boat ([`Self::sea_battle_ground`]:
    /// the attacker, or a defending hero; a monster, an event's or a pile's guards never are).
    pub(crate) fn battle_at(
        &self,
        part: &AdventureRuleset,
        attacker: &HeroOnMap,
        defender: Defender,
        tile: Pos,
    ) -> BattleStart {
        let afloat = |hero: Option<&HeroOnMap>| u8::from(hero.is_some_and(|h| h.boat.is_some()));
        let defending = match &defender {
            Defender::Hero(combatant) => self.hero(combatant.hero),
            _ => None,
        };
        let boats = afloat(Some(attacker)) + afloat(defending);
        let (terrain, battlefield) = self.sea_battle_ground(part, tile, boats);
        BattleStart {
            attacker: attacker.combatant(),
            defender,
            tile,
            terrain,
            battlefield,
            background: self.background(tile),
            attacker_ground: self.special_ground(part, attacker.pos),
        }
    }

    /// Where a battle at `pos` is fought (GAM-BAT-001, E08 T19a), in the original's order (survey
    /// `E04c-battlefields.md` §4; the ruleset part's [`crate::BattleTerrains`]): underground on its
    /// terrain, else on a coast tile (the map's flag) on the coast's terrain and battlefield, else
    /// on the tile's terrain without a special battlefield. Without the part's battle terrains,
    /// always the last. Then a special ground on the tile ([`Self::special_ground`]) replaces the
    /// battlefield, the coast's too, and keeps the terrain (ADV-GRD-001, E08 T20a2a).
    // rule: GAM-BAT-001, ADV-GRD-001
    pub fn battle_ground(
        &self,
        part: &AdventureRuleset,
        pos: Pos,
    ) -> (TerrainId, Option<ContentKey>) {
        let (terrain, field) = match part.params().battle_terrains() {
            Some(battle) if pos.level > 0 => (battle.underground, None),
            Some(battle) if self.map.is_coast(pos) => {
                (battle.coast, Some(battle.coast_battlefield.clone()))
            }
            _ => (self.map.at(pos).terrain, None),
        };
        (terrain, self.special_ground(part, pos).or(field))
    }

    /// Where a battle at `pos` is fought when `boats` of its two sides' heroes are in boats
    /// (GAM-BAT-001, E08 T74b; survey `E08-sea.md` §7.1, increment 8's design note §13): with one,
    /// on the part's sea terrain and its deck; with both (2 or more), on the sea terrain and its
    /// two boats. This comes first, before the underground: the original's
    /// `battle_setup_battlefield` (0x4643B0) tests either side's hero for the in-boat bit after a
    /// siege only (water and the ship flag; its picture with both the boats', batch E8's E8-11 and
    /// E8-12). A special ground on the tile then replaces the battlefield and keeps the sea
    /// terrain, as on land (ADV-GRD-001; none lies on water in the shipped maps). With no hero in a
    /// boat, or without the part's battle terrains, [`Self::battle_ground`].
    // rule: GAM-BAT-001
    pub fn sea_battle_ground(
        &self,
        part: &AdventureRuleset,
        pos: Pos,
        boats: u8,
    ) -> (TerrainId, Option<ContentKey>) {
        match part.params().battle_terrains() {
            Some(battle) if boats > 0 => {
                let field = if boats > 1 {
                    &battle.boats
                } else {
                    &battle.deck
                };
                (
                    battle.sea,
                    self.special_ground(part, pos).or(Some(field.clone())),
                )
            }
            _ => self.battle_ground(part, pos),
        }
    }

    /// The special battlefield of the special ground on `pos`, if any (ADV-GRD-001, E08 T20a2a):
    /// the last ground in map order whose picture lies on the tile (the original walks the tile's
    /// draw list from the top, 0x4FCE20). Since E08 T54 also the attacking hero's own tile's
    /// (GAM-BAT-013: the original's computer values a hero's spells at its own tile's mastery,
    /// `tile_spell_field` 0x4FCF40, which names only the grounds that raise masteries; the others
    /// raise none, so the battle values them alike).
    // rule: ADV-GRD-001, GAM-BAT-013
    pub fn special_ground(&self, part: &AdventureRuleset, pos: Pos) -> Option<ContentKey> {
        self.map.objects().iter().rev().find_map(|object| {
            match object.object_type.and_then(|id| part.object_type(id)) {
                Some(ObjectType {
                    kind: ObjectKind::Ground { battlefield, .. },
                    ..
                }) if object.lies_on(pos) => Some(battlefield.clone()),
                _ => None,
            }
        })
    }

    /// The battlefield picture's variant of a battle at `pos` (GAM-BAT-011, E08 T54; the
    /// original's 0x41ADB0): over the tiles within [`BACKGROUND_RADIUS`] of `pos` on its level,
    /// clipped to the map, the scenery each tile records ([`Self::tile_sceneries`]), 0 when dead
    /// vegetation outnumbers both the trees and the mountains, 1 when the mountains outnumber both
    /// the trees and the dead vegetation, else 2 (an empty surrounding too). For viewers: no rule
    /// reads it.
    // rule: GAM-BAT-011
    pub fn background(&self, pos: Pos) -> u8 {
        let (mut trees, mut mountains, mut dead) = (0u8, 0u8, 0u8);
        for scenery in self.tile_sceneries(pos) {
            match scenery {
                Scenery::Tree => trees += 1,
                Scenery::Mountain => mountains += 1,
                Scenery::DeadVegetation => dead += 1,
                Scenery::Other | Scenery::Ownable => {}
            }
        }
        if dead > trees && dead > mountains {
            0
        } else if mountains > trees && mountains > dead {
            1
        } else {
            2
        }
    }

    /// The scenery each tile within [`BACKGROUND_RADIUS`] of `pos` records (GAM-BAT-011; survey
    /// `E08-objects.md` §2.5, the original's 0x505610 on each tile), in no particular order: of
    /// the objects still on the map, the topmost — taken as the last in map order, as for visits
    /// — visited from the tile, else the topmost ownable one blocking it, else the topmost one
    /// blocking it; a tile without any records [`Scenery::Other`]. Heroes are no objects here, as
    /// they take no tile in the original. One pass over the objects, whose masks reach at most
    /// [`TileMask::COLUMNS`] − 1 columns left of and [`TileMask::ROWS`] − 1 rows above their
    /// anchors.
    fn tile_sceneries(&self, pos: Pos) -> impl Iterator<Item = Scenery> {
        const SIDE: usize = 2 * BACKGROUND_RADIUS as usize + 1;
        // Per tile of the square, the walk that took it (1 visited from, 2 an ownable object
        // blocking it, 3 any object blocking it; 4 none yet) and its object's scenery.
        let mut taken = [[(4u8, Scenery::Other); SIDE]; SIDE];
        let left = pos.x.saturating_sub(BACKGROUND_RADIUS);
        let top = pos.y.saturating_sub(BACKGROUND_RADIUS);
        let right = pos
            .x
            .saturating_add(BACKGROUND_RADIUS)
            .min(self.map.width().saturating_sub(1));
        let bottom = pos
            .y
            .saturating_add(BACKGROUND_RADIUS)
            .min(self.map.height().saturating_sub(1));
        let reach = |low: u16, high: u16, extent: u8, at: u16| {
            at >= low && at <= high.saturating_add(u16::from(extent) - 1)
        };
        for (index, object) in self.map.objects().iter().enumerate().rev() {
            let anchor = object.anchor;
            if anchor.level != pos.level
                || !reach(left, right, TileMask::COLUMNS, anchor.x)
                || !reach(top, bottom, TileMask::ROWS, anchor.y)
                || self.is_removed(index)
            {
                continue;
            }
            for y in top..=bottom {
                for x in left..=right {
                    let tile = Pos::new(x, y, pos.level);
                    let (blocks, visited) = object.covers(tile);
                    let walk = if visited {
                        1
                    } else if blocks && object.scenery == Scenery::Ownable {
                        2
                    } else if blocks {
                        3
                    } else {
                        continue;
                    };
                    let slot = &mut taken[usize::from(y - top)][usize::from(x - left)];
                    // The objects come topmost first: a later one takes a tile only by an
                    // earlier walk.
                    if walk < slot.0 {
                        *slot = (walk, object.scenery);
                    }
                }
            }
        }
        let (columns, rows) = (usize::from(right - left) + 1, usize::from(bottom - top) + 1);
        taken
            .into_iter()
            .take(rows)
            .flat_map(move |row| row.into_iter().take(columns))
            .map(|(_, scenery)| scenery)
    }

    /// Applies a battle's outcome to its heroes on the map (ADR-0017 point 11; GAM-BAT-002,
    /// GAM-BAT-003, E08 T19a), in the order given: a hero with an army keeps it as its army (its
    /// survivors by slot: at least one stack, the caller's check), a hero without one leaves the
    /// map, told to its owner and the players that see its tile (ADV-FOG-004) as
    /// [`AdventureEventKind::HeroRemoved`]; since E08 T74b a hero in a boat takes its boat with it
    /// ([`Self::remove_boat`], ADV-SEA-006), and a winner in a boat stays in it. Infallible: a
    /// hero not on the map is skipped.
    // rule: GAM-BAT-002, GAM-BAT-003, ADV-SEA-006
    pub fn end_battle<S: Sink>(
        &mut self,
        core: &Ruleset,
        heroes: &[(HeroId, Option<&[ArmyStack]>)],
        sink: &mut S,
    ) {
        for &(id, army) in heroes {
            let Ok(index) = self.heroes.binary_search_by_key(&id, |h| h.id) else {
                continue;
            };
            match army {
                Some(army) => self.heroes[index].army = army.to_vec(),
                None => {
                    let hero = self.heroes.remove(index);
                    let seen_by = self.viewers(core, hero.owner, hero.pos);
                    sink.emit(AdventureEvent {
                        kind: AdventureEventKind::HeroRemoved {
                            hero: id,
                            pos: hero.pos,
                        },
                        seen_by,
                    });
                    if let Some(boat) = hero.boat {
                        self.remove_boat((id, boat, hero.pos), seen_by, sink);
                    }
                }
            }
        }
    }

    /// Adds creatures a battle raised to a hero's army (GAM-BAT-006, E08 T19a2b; a hero's
    /// Necromancy after its side won): `count` of `creature`, at most a stack's most (ADV-ARM-007,
    /// E08 T50), join its first stack of that creature that takes them within the most, else a
    /// new stack in its first free slot; if neither is possible (a full army without such a
    /// stack), ⌈2 · that count / 3⌉ of the creature's first upgrade join its first stack of the
    /// upgrade, if it takes them within the most; else none join. Returns the creature and the
    /// count that joined, `None` when none did, `count` was 0 or the hero is not on the map. The
    /// original's `necromancy_add_raised` (0x469F20): `army_add_stack` (0x44ACE0), then once the
    /// upgrade (0x47B1A0) at (2c + 2) / 3; its exception for the four elementals (112 to 115, no
    /// upgrade while a game flag is off) is not modelled, as no hero raises them.
    // rule: GAM-BAT-006, ADV-ARM-007
    pub fn raise(
        &mut self,
        core: &Ruleset,
        part: &AdventureRuleset,
        hero: HeroId,
        (creature, count): (CreatureTypeId, u32),
    ) -> Option<(CreatureTypeId, u32)> {
        let most = part.params().max_stack_count();
        let rules = (part.params().army_slots(), most);
        let army = &mut self.heroes.iter_mut().find(|h| h.id == hero)?.army;
        let count = count.min(most);
        if count == 0 {
            return None;
        }
        if monsters::add_stack(army, rules, creature, count) {
            return Some((creature, count));
        }
        let upgrade = core.upgrades(creature).first().copied()?;
        // ⌈2c / 3⌉ = (2c + 2) / 3, exact in 64 bits and below 2^32 again.
        let fewer = u32::try_from((u64::from(count) * 2).div_ceil(3)).unwrap_or(u32::MAX);
        let stack = army
            .iter_mut()
            .find(|s| s.creature == upgrade && fewer <= most.saturating_sub(s.count))?;
        stack.count += fewer;
        Some((upgrade, fewer))
    }

    /// An object's visit by `player`'s hero `hero`, by what its kind does ([`Effect`]). A pickup
    /// (ADV-PIK-001, ADV-PIK-003): the visitor's owner gains its contents, and it leaves the map
    /// ([`Self::take_object`], ADV-OBJ-005); a guarded pile's guards ask first or fight
    /// ([`Self::visit_pile`], ADV-PIK-004, E08 T21a). A Cartographer (ADV-VIS-001, E08 T14b): an offer of
    /// its reveal for its price, unless the player has bought its type; the game opens it if the
    /// player can pay (ADR-0017 point 10). An observatory (ADV-VIS-002, T15a): the tiles within its
    /// radius of the visited tile, on its level, explored for the player, told to it alone with
    /// the number it had not explored, at every visit. A Keymaster's tent (ADV-VIS-003): its key
    /// for the player, told to it alone, unless it holds it (then nothing, no event). A border
    /// guard (ADV-VIS-004): for a holder of its key an offer to pass, without a price; for another
    /// player nothing (the original's message is not modelled). A border gate (ADV-VIS-005, E08
    /// T15b), visited by a player without its key (its holders cross it, [`Self::visit_at`]):
    /// nothing, as the guard's (the original's message is not modelled). A wandering monster
    /// (ADV-MON-007, E08 T20b1): its answer to the hero ([`Self::visit_monster`]). Since E08
    /// T62a increment 7's pickups, teachers and finds (`rewards.rs`): a treasure chest
    /// ([`Self::visit_chest`]), an artifact or a Spell Scroll ([`Self::visit_artifact`]), a
    /// once-per-hero gift ([`Self::visit_gift`]), a Shrine, a Witch Hut, a Scholar, a University
    /// and a find. Since E08 T74a the sea's finds, which leave the map once searched, and a
    /// message (the Ocean Bottle, ADV-PIK-017; the original's 0x4A9AFA, survey `E08-sea.md`
    /// §8.5): read by the visitor's owner alone ([`AdventureEventKind::MessageRead`]; a computer's
    /// hero as a human's, `ADV-QUIRK-30` not reproduced), then, when it leaves, taken off the map;
    /// nothing given, no question.
    // rule: ADV-PIK-001, ADV-PIK-003, ADV-OBJ-005, ADV-VIS-001, ADV-VIS-002, ADV-VIS-003, ADV-VIS-004
    // rule: ADV-VIS-005, ADV-PIK-004, ADV-PIK-017
    fn visit<S: Sink>(
        &mut self,
        rules: (&Ruleset, &AdventureRuleset, &[Arc<HeroDetails>]),
        (player, hero): (PlayerId, HeroId),
        visit: StepVisit<'_>,
        sink: &mut S,
    ) -> AdventureOutcome {
        let core = rules.0;
        let object = (visit.object, visit.object_type);
        let anchor = self.object_at(visit.object).anchor;
        let offer = |kind| Offer {
            hero,
            object_type: visit.object_type,
            anchor,
            kind,
        };
        let mut outcome = AdventureOutcome::default();
        match visit.effect {
            Effect::Pickup => outcome = self.visit_pile(core, rules.1, hero, object, sink),
            Effect::Chart(price) => {
                outcome.offer = (!self.has_bought(player, visit.object_type)).then(|| {
                    offer(OfferKind::Reveal {
                        price: price.to_vec(),
                    })
                });
            }
            Effect::Reveal(radius) => {
                let grid = &mut self.explored[usize::from(player.raw())];
                let before = grid.count();
                reveal(grid, visit.tile, radius);
                sink.emit(AdventureEvent {
                    kind: AdventureEventKind::TilesRevealed {
                        object_type: visit.object_type,
                        count: grid.count() - before,
                    },
                    seen_by: Viewers::NONE.with(player),
                });
            }
            Effect::Tent(key) => {
                if !self.has_key(player, key) {
                    let flags = self.flags.get_or_insert_with(Arc::default);
                    Arc::make_mut(flags).give_key(player, key);
                    sink.emit(AdventureEvent {
                        kind: AdventureEventKind::KeyGained { key },
                        seen_by: Viewers::NONE.with(player),
                    });
                }
            }
            Effect::Guard(key) => {
                outcome.offer = self.has_key(player, key).then(|| offer(OfferKind::Pass));
            }
            Effect::Gate(_) => {}
            Effect::Monster => outcome = self.visit_monster(rules, hero, object),
            Effect::Chest(chest) => outcome = self.visit_chest(rules, hero, object, chest, sink),
            Effect::Artifact => outcome = self.visit_artifact(rules, hero, object, sink),
            Effect::Gift(gift) => outcome = self.visit_gift(rules, (player, hero), object, gift),
            Effect::Shrine => outcome = self.visit_shrine(rules, (player, hero), visit.object),
            Effect::WitchHut => outcome = self.visit_witch_hut(rules, (player, hero), visit.object),
            Effect::Scholar => outcome = self.visit_scholar(rules, hero, object, sink),
            Effect::University(price) => {
                outcome = self.visit_university((player, hero), object, price);
            }
            Effect::Find(find) => {
                outcome = self.visit_find(rules, (player, hero), object, find, sink);
            }
            Effect::Mine => {
                outcome = self.visit_mine((core, rules.1), (player, hero), object, sink);
            }
            Effect::PandoraBox => outcome = self.ask(hero, object, OfferKind::OpenBox),
            Effect::Pyramid => outcome = self.ask(hero, object, OfferKind::SearchPyramid),
            Effect::Message(leaves) => {
                sink.emit(AdventureEvent {
                    kind: AdventureEventKind::MessageRead {
                        hero,
                        object_type: visit.object_type,
                        anchor,
                    },
                    seen_by: Viewers::NONE.with(player),
                });
                if leaves {
                    self.take_object(core, object, sink);
                }
            }
        }
        outcome
    }

    /// Takes the object at `index` of the static map, of type `object_type`, off the map
    /// ([`Self::remove_object`], ADV-OBJ-005), told to the players that see it there, by its type
    /// and anchor.
    fn take_object<S: Sink>(
        &mut self,
        core: &Ruleset,
        (index, object_type): (usize, ObjectTypeId),
        sink: &mut S,
    ) {
        let object = self.object_at(index);
        sink.emit(AdventureEvent {
            kind: AdventureEventKind::ObjectRemoved {
                object_type,
                anchor: object.anchor,
            },
            seen_by: self.object_viewers(core, object),
        });
        self.remove_object(index);
    }

    /// Whether `offer` still stands (ADR-0017 point 10): its object is still on the map. A border
    /// guard may have left it since the offer opened, passed by another player of the same turn
    /// group (E08 T15a); a Cartographer never leaves it, and nothing of its owner's but its answer
    /// runs while an offer is open (ADR-0017 point 6).
    pub fn offer_stands(&self, offer: &Offer) -> bool {
        self.offer_object(offer).is_some()
    }

    /// The object `offer` names, if it is still on the map: the last in map order still on the
    /// map with the offer's type and anchor (the one its visit concerned: a step visits the last
    /// such object, [`Self::tile_object`]; no shipped map has two typed objects of one type on one
    /// anchor, `crates/h3-map/examples/original_data.rs`).
    fn offer_object(&self, offer: &Offer) -> Option<usize> {
        self.object_named(offer.object_type, offer.anchor)
    }

    /// Accepts `offer`, made to `player` by a visit (ADR-0017 point 10: the game has checked that
    /// it still stands, [`Self::offer_stands`], and paid its price). A Cartographer's
    /// (ADV-VIS-001): every tile of the map, on every level, whose terrain is among the
    /// Cartographer's terrains becomes explored for `player`, told to it alone with the number of
    /// tiles it had not explored, and the player has bought the type, so no later visit of an
    /// object of the type makes it an offer; it was made for a type the player had not bought, and
    /// while it is open nothing of the player's but its answer runs (ADR-0017 point 6) and no other
    /// player's command buys for it. A border guard's (ADV-VIS-004, E08 T15a): the guard leaves the
    /// map for everyone ([`Self::take_object`]). A wandering monster's offer to join, for free or
    /// for its price (ADV-MON-008, E08 T20b1): the group joins the hero's army and the monster
    /// leaves the map; its treasure is the outcome's gains ([`Self::join`]). Its offer to flee:
    /// the hero fights it (ADV-MON-008; [`Self::pursue`]). A guarded pile's question (ADV-PIK-004,
    /// E08 T21a): the hero fights its guards as they stand, or picks it up when none stand any
    /// more ([`Self::fight_or_pick`]; since E08 T62a a guarded artifact's or scroll's too); a
    /// guarded mine's (ADV-MIN-002, E08 T84a): their battle, or the flag when none stand any more
    /// ([`Self::fight_for_mine`]). A gift's price (ADV-TCH-003, E08 T62a): the gift given, the hero marked. A find's question
    /// (ADV-PIK-011): the find searched. Pandora's Box's (ADV-PIK-013, E08 T62b1): its guards'
    /// battle or its record ([`Self::open_box`]). The Pyramid's (ADV-TCH-008, E08 T62b1): a
    /// looted one's luck or its guards' battle ([`Self::search_pyramid`]). Infallible; the outcome
    /// is what the game applies after.
    // rule: ADV-VIS-001, ADV-VIS-004, ADV-MON-008, ADV-PIK-004, ADV-TCH-003, ADV-PIK-011
    // rule: ADV-PIK-013, ADV-TCH-008, ADV-MIN-002
    pub fn accept_offer<S: Sink>(
        &mut self,
        rules: (&Ruleset, &AdventureRuleset, &[Arc<HeroDetails>]),
        player: PlayerId,
        offer: &Offer,
        sink: &mut S,
    ) -> AdventureOutcome {
        let (core, part, _) = rules;
        match offer.kind {
            OfferKind::Reveal { .. } => {
                self.reveal_terrain(part, player, offer.object_type, sink);
                AdventureOutcome::default()
            }
            OfferKind::Pass => {
                if let Some(index) = self.offer_object(offer) {
                    self.take_object(core, (index, offer.object_type), sink);
                }
                AdventureOutcome::default()
            }
            OfferKind::Join | OfferKind::JoinForGold { .. } => {
                self.join(rules, player, offer, sink)
            }
            OfferKind::Pursue => self.pursue(part, offer),
            OfferKind::Fight => self
                .offer_object(offer)
                .map(|index| {
                    let object = (index, offer.object_type);
                    if self.is_mine(part, index) {
                        self.fight_for_mine((core, part), (player, offer.hero), object, sink)
                    } else {
                        self.fight_or_pick(core, part, offer.hero, object, sink)
                    }
                })
                .unwrap_or_default(),
            OfferKind::Gift { .. } => self.accept_gift(rules, offer),
            OfferKind::Search => self.accept_search(rules, player, offer, sink),
            OfferKind::OpenBox => self.open_box(rules, offer, sink),
            OfferKind::SearchPyramid => self.search_pyramid(part, player, offer),
        }
    }

    /// A Cartographer's reveal for `player` and its purchase ([`Self::accept_offer`]).
    fn reveal_terrain<S: Sink>(
        &mut self,
        part: &AdventureRuleset,
        player: PlayerId,
        object_type: ObjectTypeId,
        sink: &mut S,
    ) {
        let terrains = part
            .object_type(object_type)
            .and_then(|ty| ty.kind.reveal_terrain())
            .map_or(&[][..], |(terrains, _)| terrains);
        let map = &self.map;
        let (width, height) = (usize::from(map.width()), usize::from(map.height()));
        let at = |n: usize| u16::try_from(n).unwrap_or(u16::MAX);
        let reveals = |tile: &StaticTile| terrains.contains(&tile.terrain);
        let grid = &mut self.explored[usize::from(player.raw())];
        let before = grid.count();
        for (row, tiles) in map.tiles().chunks_exact(width).enumerate() {
            let level = u8::try_from(row / height).unwrap_or(u8::MAX);
            let mut x = 0;
            // Row by row, a run of tiles of the terrains at a time.
            for run in tiles.chunk_by(|a, b| reveals(a) == reveals(b)) {
                if reveals(&run[0]) {
                    grid.set_run(
                        Pos::new(at(x), at(row % height), level),
                        at(run.len()),
                        true,
                    );
                }
                x += run.len();
            }
        }
        sink.emit(AdventureEvent {
            kind: AdventureEventKind::TilesRevealed {
                object_type,
                count: grid.count() - before,
            },
            seen_by: Viewers::NONE.with(player),
        });
        let flags = self.flags.get_or_insert_with(Arc::default);
        Arc::make_mut(flags).buy(player, object_type);
    }

    /// Whether `player` has bought the Cartographer type `object_type` (ADV-VIS-001).
    pub fn has_bought(&self, player: PlayerId, object_type: ObjectTypeId) -> bool {
        self.flags
            .as_ref()
            .is_some_and(|flags| flags.has_bought(player, object_type))
    }

    /// The Cartographer types `player` has bought, in type order (ADV-VIS-001).
    pub fn bought(&self, player: PlayerId) -> impl Iterator<Item = ObjectTypeId> + '_ {
        self.flags
            .iter()
            .flat_map(move |flags| flags.bought(player))
    }

    /// Whether `player` holds the key of `colour` (ADV-VIS-003, E08 T15a).
    pub fn has_key(&self, player: PlayerId, colour: u8) -> bool {
        self.flags
            .as_ref()
            .is_some_and(|flags| flags.has_key(player, colour))
    }

    /// The keys `player` holds, bit `c` for the colour `c` (ADV-VIS-003, E08 T15a).
    pub fn keys(&self, player: PlayerId) -> u8 {
        self.flags.as_ref().map_or(0, |flags| flags.keys(player))
    }

    /// Who sees `object` on the map, as the observation shows it (ADV-FOG-004, ADV-FOG-005):
    /// every player that has explored a tile it covers and, under `ADV-QUIRK-1` or for a monster
    /// (E08 decision 40: it guards the explored tiles beside it), every player with such a tile on
    /// its fringe.
    fn object_viewers(&self, core: &Ruleset, object: &MapObject) -> Viewers {
        let fringe = core.quirks().is_enabled(FRINGE_SHOWS_OBJECTS) || object.monster.is_some();
        self.explored
            .iter()
            .zip((0..self.players).map(PlayerId::new))
            .filter(|(grid, _)| {
                object
                    .tiles(&self.map)
                    .any(|pos| grid.get(pos) || (fringe && on_fringe(grid, pos)))
            })
            .fold(Viewers::NONE, |viewers, (_, player)| viewers.with(player))
    }

    /// The object a visit at `pos` concerns, with its type: the last object in map order still on
    /// the map that is visited from `pos`, if it has a type (E08 T14a; ADV-OBJ-003), an object
    /// added in play coming after the static map's (E08 T83b2a: looked up only where one covers
    /// the tile). The original
    /// keeps one object per tile (the map tile's class, 0x4FCBD0), the topmost one drawn there that
    /// is visited from it (0x505610), the objects placed in map order when the map loads
    /// (0x4FD690, 0x505B20); the step, the visit and the pathfinder's test from above all read
    /// that one, so a single order serves both visit modes here. VCMI visits the last of the tile
    /// when it is entered (`visitObjectOnTile`), the first when it is not (`blockingVisit`).
    fn tile_object<'p>(
        &self,
        part: &'p AdventureRuleset,
        pos: Pos,
    ) -> Option<(usize, ObjectTypeId, &'p ObjectType)> {
        let index = self.added_visited_from(pos).or_else(|| {
            self.map
                .visited_from(pos)
                .iter()
                .rev()
                .copied()
                .find(|&index| !self.is_removed(index))
        })?;
        let id = self.object_at(index).object_type?;
        part.object_type(id).map(|ty| (index, id, ty))
    }

    /// What a step of `player`'s hero onto `pos` meets, if the visit of its object is built
    /// ([`ObjectKind::effect`]): the object of [`Self::tile_object`], how it is visited, whether
    /// the player passes it instead and what the visit does. The visit mode is the type's, but per
    /// player: a border gate whose key `player` holds is passed, not visited (ADV-VIS-005, E08
    /// T15b; the original's step, 0x480EE2, and its pathfinder, 0x56ABB1, test the player's key
    /// bit). `MoveHero` and the reachable-tiles query both ask here. The caller has checked that
    /// `pos` is visitable as it stands now.
    // rule: ADV-VIS-005
    #[inline]
    pub(crate) fn visit_at<'p>(
        &self,
        part: &'p AdventureRuleset,
        player: PlayerId,
        pos: Pos,
    ) -> Option<StepVisit<'p>> {
        let (object, object_type, ty) = self.tile_object(part, pos)?;
        ty.kind.effect().map(|effect| StepVisit {
            object,
            object_type,
            tile: pos,
            mode: ty.visit,
            pass: matches!(effect, Effect::Gate(key) if self.has_key(player, key)),
            from_above: ty.from_above,
            effect,
        })
    }

    /// Whether `pos` is the visitable tile of an object not visited from above (ADV-OBJ-004): a
    /// walking hero neither steps onto it from, nor leaves it toward, the three tiles above it.
    // rule: ADV-OBJ-004
    pub(crate) fn closed_above(&self, part: &AdventureRuleset, pos: Pos) -> bool {
        self.tile_object(part, pos)
            .is_some_and(|(_, _, ty)| !ty.from_above)
    }

    /// Who sees a hero of `owner` on `pos`, a tile of the map (ADV-FOG-004, ADV-FOG-005, as the
    /// observation, ADV-FOG-006): `owner`, every player that has explored the tile and, under
    /// `ADV-QUIRK-1`, every player with it on its fringe (heroes show there, the user,
    /// 2026-10-11).
    // rule: ADV-FOG-004, ADV-FOG-005
    fn viewers(&self, core: &Ruleset, owner: PlayerId, pos: Pos) -> Viewers {
        let fringe = core.quirks().is_enabled(FRINGE_SHOWS_OBJECTS);
        self.explored
            .iter()
            .zip((0..self.players).map(PlayerId::new))
            .filter(|(grid, player)| {
                *player == owner || grid.get(pos) || (fringe && on_fringe(grid, pos))
            })
            .fold(Viewers::NONE, |viewers, (_, player)| viewers.with(player))
    }

    /// Whether `player`'s hero `mover` may step from `from` onto `pos`, in `direction`: a tile the
    /// player has explored (ADV-MOV-008; a tile off the map is not asked about, the map's size
    /// being public), then one it may stand on (ADV-MOV-007), then not onto or off an object's
    /// tile across its top (ADV-OBJ-004); the tile, and the object the step visits if an object
    /// is visited from it (ADR-0016 point 4: a pickup's visit since E08 T13b, the Cartographer's
    /// since T14a), none for a border gate the player passes (ADV-VIS-005, T15b). The explored test
    /// comes first, so a rejection tells nothing about a hidden tile. Since E08 T73 a step onto an
    /// empty boat's tile on water boards it ([`Meets::Board`], ADV-SEA-002: the walk ends there,
    /// no guard attacks on it), and a hero in a boat is visited from the shore as any hero.
    // rule: ADV-MOV-007, ADV-MOV-008, ADV-OBJ-004, ADV-SEA-002
    fn check_step<'p>(
        &self,
        part: &'p AdventureRuleset,
        player: PlayerId,
        mover: HeroId,
        (from, pos): (Pos, Pos),
        direction: usize,
        step: usize,
    ) -> Result<(StaticTile, Option<Meets<'p>>), AdventureCommandError> {
        if self.map.tile(pos).is_some() && !self.is_explored(player, pos) {
            return Err(AdventureCommandError::Unexplored { step });
        }
        // A step down enters `pos` from above; a step up leaves `from` across its top
        // (ADV-OBJ-004). The tile's object decides, a hero standing on it or not: the original's
        // test from above reads the object under a hero (0x4FCBD0; batch E4's E4-10, E08 T33).
        let across = match DIRECTIONS[direction].1 {
            1 => Some(pos),
            -1 => Some(from),
            _ => None,
        };
        let closed = || across.is_some_and(|at| self.closed_above(part, at));
        // ADR-0016 point 4: another hero is visited from the neighbouring tile: another player's
        // in a battle at once (E08 T19a), an own one in a meeting (T50); not across the top of an
        // object's tile it stands on or the visitor stands on.
        // rule: GAM-BAT-004, GAM-MET-001
        if let Some(other) = self
            .heroes
            .iter()
            .position(|h| h.pos == pos && h.id != mover)
        {
            if closed() {
                return Err(AdventureCommandError::FromAbove { step });
            }
            return Ok((self.live(pos, self.map.at(pos)), Some(Meets::Hero(other))));
        }
        let (tile, boat) = match self.check_tile(part, pos, Some(mover)) {
            Ok(tile) => (tile, None),
            // Water: an empty boat's tile only (0x481D0D; class 85's, the Shipwreck's, is an
            // untyped blocker's here, ADV-STUB-12).
            Err(error @ StandError::Terrain) => {
                let boat = self
                    .empty_boat(pos)
                    .ok_or(AdventureCommandError::Blocked { step, error })?;
                (self.live(pos, self.map.at(pos)), Some(boat))
            }
            Err(error) => return Err(AdventureCommandError::Blocked { step, error }),
        };
        if closed() {
            return Err(AdventureCommandError::FromAbove { step });
        }
        if let Some(boat) = boat {
            return Ok((tile, Some(Meets::Board(boat))));
        }
        if !tile.visitable {
            return Ok((tile, None));
        }
        self.visit_at(part, player, pos)
            .map(|object| (tile, (!object.pass).then_some(Meets::Object(object))))
            .ok_or(AdventureCommandError::Visit { step })
    }

    /// The static map.
    pub fn map(&self) -> &StaticMap {
        &self.map
    }

    /// How many players the game has.
    pub const fn players(&self) -> u8 {
        self.players
    }

    /// The heroes on the map, by id.
    pub fn heroes(&self) -> &[HeroOnMap] {
        &self.heroes
    }

    /// The tiles `player` has explored, if it is a player of the game (ADR-0016 point 5).
    pub fn explored(&self, player: PlayerId) -> Option<&BitGrid> {
        self.explored.get(usize::from(player.raw()))
    }

    /// The tile at `pos` as it stands now (ADR-0016 point 1): `tile`, the static map's tile there,
    /// blocked or visitable only where an object still on the map covers it so, an object added
    /// in play's included (E08 T83b2a).
    #[inline]
    pub(crate) fn live(&self, pos: Pos, tile: StaticTile) -> StaticTile {
        let tile = self.live_placed(pos, tile);
        match &self.added {
            Some(added) if added.tiles.get(pos) => self.added_live(pos, tile),
            _ => tile,
        }
    }

    /// [`Self::live`] but for the objects added in play: the static map's objects still on it
    /// only, for a search that knows none was added (E08 T83b2a, `bench-check`).
    #[inline]
    pub(crate) fn live_placed(&self, pos: Pos, tile: StaticTile) -> StaticTile {
        match &self.occupancy {
            Some(occupancy) => occupancy.live(pos, tile),
            None => tile,
        }
    }

    /// Whether the object at `index`, of the static map or added in play, has left the map (no
    /// object has before increment 2's visits).
    pub fn is_removed(&self, index: usize) -> bool {
        self.occupancy
            .as_ref()
            .is_some_and(|occupancy| occupancy.is_removed(index))
    }

    /// Takes the object at `index`, of the static map or added in play, off the map (ADR-0016
    /// point 1): the tiles it covered that no other object still on the map covers so become
    /// free; the static map keeps its footprint. A monster's fraction goes with it (E08 T83b2a,
    /// *agent's call*: the table holds only monsters on the map).
    pub(crate) fn remove_object(&mut self, index: usize) {
        if self.object(index).is_none() || self.is_removed(index) {
            return;
        }
        self.fractions.set(index, None);
        let map = &self.map;
        let added = self.added.as_deref().map_or(&[][..], |a| &a.objects);
        let occupancy = self.occupancy.get_or_insert_with(|| {
            Arc::new(Occupancy::new(map, map.objects().len() + added.len()))
        });
        Arc::make_mut(occupancy).remove(map, added, index);
    }

    /// The object at `index`: the static map's, or one added in play after them (E08 T83b2a;
    /// increment 9's design note §6), on the map or not; none past the last.
    pub fn object(&self, index: usize) -> Option<&MapObject> {
        let statics = self.map.objects();
        statics
            .get(index)
            .or_else(|| self.added_objects().get(index - statics.len()))
    }

    /// [`Self::object`] for an index the caller found among the objects (a visit's, a guard's).
    pub(crate) fn object_at(&self, index: usize) -> &MapObject {
        let statics = self.map.objects();
        statics
            .get(index)
            .unwrap_or_else(|| &self.added_objects()[index - statics.len()])
    }

    /// The objects added in play, in the order they came, each at index
    /// `map().objects().len() + n` (E08 T83b2a: none until T85a's creature month; tests add
    /// them), on the map or not.
    pub fn added_objects(&self) -> &[MapObject] {
        self.added.as_deref().map_or(&[], |added| &added.objects)
    }

    /// Whether a monster's visitable tile, of the static map or added in play, lies within one
    /// step of `pos` on its level (ADV-MON-005: a tile without one is guarded by none).
    #[inline]
    pub(crate) fn may_guard(&self, pos: Pos) -> bool {
        self.map.near_monster(pos)
            || self
                .added
                .as_deref()
                .is_some_and(|added| added.near.get(pos))
    }

    /// The tiles an object added in play blocks or is visited from, if one was added (E08
    /// T83b2a): where a search that reads the static map's marks first must also ask
    /// [`Self::live`].
    pub(crate) fn added_tiles(&self) -> Option<&BitGrid> {
        self.added.as_deref().map(|added| &added.tiles)
    }

    /// A wandering monster's fraction of a creature at object index `index`, in the monsters'
    /// growth unit (ADV-MON-013, E08 T83b2a): 0 when it has none.
    pub fn fraction(&self, index: usize) -> u8 {
        self.fractions.get(index).copied().unwrap_or(0)
    }

    /// Whether `player` has explored the tile at `pos` (a tile off the map, or a player not in
    /// the game, has none).
    pub fn is_explored(&self, player: PlayerId, pos: Pos) -> bool {
        self.explored(player).is_some_and(|grid| grid.get(pos))
    }

    /// The game's difficulty, from 0 (easy; E08 T20a2a).
    pub const fn difficulty(&self) -> u8 {
        self.difficulty
    }

    /// The army a battle left on the object at `index`, if one did (E08 T20a2a; written from
    /// E08 T20b).
    pub fn army_left(&self, index: usize) -> Option<&[ArmyStack]> {
        self.army_left.get(index).map(Vec::as_slice)
    }

    /// What visits left on the objects (E08 T61b, increment 7's design note §10): none until a
    /// visit writes it.
    pub fn visits(&self) -> Option<&Visits> {
        self.visits.as_deref()
    }

    /// The count of the monster at object index `index` as it stands (E08 T20a2a): the creatures
    /// a battle left on it, else the static map's count; none for an object that is no monster.
    pub fn monster_count(&self, index: usize) -> Option<u32> {
        let monster = self.object(index)?.monster.as_ref()?;
        Some(self.count_of(index, monster))
    }

    /// The count of `monster`, the object at `index`, as it stands ([`Self::monster_count`]).
    fn count_of(&self, index: usize, monster: &Monster) -> u32 {
        self.army_left(index).map_or(monster.count, |army| {
            army.iter().fold(0, |sum, s| sum.saturating_add(s.count))
        })
    }

    /// The hero with this id, if it is on the map.
    pub fn hero(&self, id: HeroId) -> Option<&HeroOnMap> {
        self.heroes
            .binary_search_by_key(&id, |h| h.id)
            .ok()
            .map(|index| &self.heroes[index])
    }

    /// The canonical state hash (ADR-0003 point 12, ADR-0016 point 2): content only, whatever
    /// the sharing.
    pub fn state_hash(&self) -> u64 {
        hash_of(self)
    }
}

impl CanonicalHash for AdventureState {
    /// Encoding v19 (E08 T83b2a; v1 was T5a's, v2 T6b's, v3 T8a's, v4 T12's, v5 T14b's, v6
    /// T15a's, v7 T18b's, v8 T19a's, v9 T20a2a's, v10 T21a's, v11 T50's, v14 T61b's, v15 T62b2's,
    /// v16 T72's, v18 T83b1's; v12, v13 and v17 changed the static map's):
    /// version; the static map's content hash (since v4 with the objects' types and contents); the
    /// number of players; the heroes by id (id, owner, tile, the army's stacks by slot: slot,
    /// creature, count; since v7 the formation, 0 loose, 1 tight; since v11 the tactics flag;
    /// since v2 the movement points left; since v16 the day's maximum and the boat, 0 for none,
    /// else 1 and its index); since v3 each player's explored tiles, by player id
    /// ([`BitGrid`]'s encoding); since v4 the objects that left the map (0 for none, else 1 and
    /// [`Occupancy`]'s encoding; there is an occupancy only once an object has left); since v19
    /// the objects added in play (0 for none, else 1 and [`Added`]'s encoding); since v5 the
    /// Cartographer types bought, as length and entries in type order (the type, then its byte of
    /// player bits); since v6 then the keys, a byte of player bits per colour (none bought and no
    /// key before the first purchase or key); since v9 the difficulty, (since v15) each player's
    /// seat by player id (its colour, whether it counts as human), then the armies battles
    /// left on objects as their number and, by object index, the index and the army's stacks as
    /// length and entries (slot, creature, count); since v14 the visits' block (0 for none, else
    /// 1 and [`Visits`]' encoding; there is one only once a visit wrote it); since v16 the boats
    /// (their number, 0 for a map without boats, then each: [`Boat`]'s fields in order); since
    /// v18 the owners and the weekly contents (`economy.rs`' `hash_economy`); since v19 the
    /// monsters' fractions as their number and, by object index, the index and the fraction.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u32(ADVENTURE_STATE_HASH_VERSION);
        h.write_u64(self.map.content_hash());
        h.write_u8(self.players);
        h.write_len(self.heroes.len());
        for hero in &self.heroes {
            h.write_u16(hero.id.raw());
            h.write_u8(hero.owner.raw());
            hero.pos.hash_into(h);
            h.write_len(hero.army.len());
            for stack in &hero.army {
                h.write_u8(stack.slot);
                stack.creature.hash_into(h);
                h.write_u32(stack.count);
            }
            h.write_bool(hero.formation == Formation::Tight);
            h.write_bool(hero.tactics);
            h.write_u32(hero.movement);
            h.write_u32(hero.max_movement);
            match hero.boat {
                None => h.write_u8(0),
                Some(boat) => {
                    h.write_u8(1);
                    h.write_u32(boat);
                }
            }
        }
        h.write_len(self.explored.len());
        for grid in &self.explored {
            grid.hash_into(h);
        }
        match &self.occupancy {
            None => h.write_u8(0),
            Some(occupancy) => {
                h.write_u8(1);
                occupancy.hash_into(h);
            }
        }
        match &self.added {
            None => h.write_u8(0),
            Some(added) => {
                h.write_u8(1);
                added.hash_into(h);
            }
        }
        self.flags.as_deref().unwrap_or(&Flags::NONE).hash_into(h);
        h.write_u8(self.difficulty);
        for seat in &self.seats[..usize::from(self.players)] {
            h.write_u8(seat.colour);
            h.write_bool(seat.human);
        }
        h.write_len(self.army_left.len());
        for (index, army) in self.army_left.iter() {
            h.write_len(index);
            h.write_len(army.len());
            for stack in army {
                h.write_u8(stack.slot);
                stack.creature.hash_into(h);
                h.write_u32(stack.count);
            }
        }
        match &self.visits {
            None => h.write_u8(0),
            Some(visits) => {
                h.write_u8(1);
                visits.hash_into(h);
            }
        }
        match &self.boats {
            None => h.write_len(0),
            Some(boats) => boats.hash_into(h),
        }
        self.hash_economy(h);
        h.write_len(self.fractions.len());
        for (index, &fraction) in self.fractions.iter() {
            h.write_len(index);
            h.write_u8(fraction);
        }
    }
}

/// Checks an army against the ruleset (one or more stacks of known creatures with at least one
/// creature each and at most a stack's most (ADV-ARM-007, E08 T50), in distinct slots below the
/// army slots) and returns its stacks by slot.
// rule: ADV-ARM-007
fn check_army(
    core: &Ruleset,
    part: &AdventureRuleset,
    army: &[ArmyStack],
) -> Result<Vec<ArmyStack>, ArmyError> {
    if army.is_empty() {
        return Err(ArmyError::Empty);
    }
    let mut slots: Vec<Option<ArmyStack>> = vec![None; usize::from(part.params().army_slots())];
    for stack in army {
        let slot = slots
            .get_mut(usize::from(stack.slot))
            .ok_or(ArmyError::SlotOutOfRange(stack.slot))?;
        if slot.is_some() {
            return Err(ArmyError::DuplicateSlot(stack.slot));
        }
        if stack.count == 0 {
            return Err(ArmyError::EmptyStack(stack.slot));
        }
        if stack.count > part.params().max_stack_count() {
            return Err(ArmyError::TooManyCreatures(stack.slot));
        }
        if core.creature(stack.creature).is_none() {
            return Err(ArmyError::UnknownCreature(stack.slot));
        }
        *slot = Some(*stack);
    }
    Ok(slots.into_iter().flatten().collect())
}

#[cfg(test)]
impl AdventureState {
    /// Sets a hero's movement points (tests that need a given number of points).
    pub(crate) fn set_movement(&mut self, hero: HeroId, points: u32) {
        self.heroes
            .iter_mut()
            .filter(|h| h.id == hero)
            .for_each(|h| h.movement = points);
    }

    /// Writes an army on the object at `index` (tests of a mine's guards, E08 T83b1, until T84's
    /// battles and Leave Guards write them).
    pub(crate) fn set_army_left(&mut self, index: usize, army: Vec<ArmyStack>) {
        self.army_left.set(index, Some(army));
    }

    /// Writes the weekly contents at weekly place `place` (tests of the weekly invariants and
    /// hash, E08 T83b1, until T84's visits and T85a's week write them).
    pub(crate) fn set_weekly(&mut self, place: usize, weekly: Weekly) {
        self.weekly = self.weekly.as_deref().map(|block| {
            let mut entries = block.to_vec();
            entries[place] = weekly;
            entries.into()
        });
    }

    /// Adds `object` to the map after every object before it (E08 T83b2a: tests of the objects
    /// added in play, until T85a's creature month adds them) and returns its index.
    pub(crate) fn add_object(&mut self, object: MapObject) -> usize {
        let index = self.map.objects().len() + self.added_objects().len();
        if let Some(occupancy) = &mut self.occupancy {
            Arc::make_mut(occupancy).make_room(index + 1);
        }
        let map = &self.map;
        let added = self.added.get_or_insert_with(|| Arc::new(Added::new(map)));
        Arc::make_mut(added).push(map, object);
        index
    }

    /// Writes the fraction of the monster at `index` (tests of the fractions' invariants and
    /// hash, E08 T83b2a, until T85a's week writes them); 0 removes it.
    pub(crate) fn set_fraction(&mut self, index: usize, fraction: u8) {
        self.fractions
            .set(index, (fraction > 0).then_some(fraction));
    }

    /// Gives `player` the key of `colour` (tests of the border gates, E08 T15b).
    pub(crate) fn give_key(&mut self, player: PlayerId, colour: u8) {
        Arc::make_mut(self.flags.get_or_insert_with(Arc::default)).give_key(player, colour);
    }

    /// Explores every tile for every player (tests of movement that walk past the tiles the
    /// heroes revealed at the start).
    pub(crate) fn explore_all(&mut self) {
        let (width, height, levels) = (self.map.width(), self.map.height(), self.map.levels());
        for grid in &mut self.explored {
            for level in 0..levels {
                for y in 0..height {
                    grid.set_run(Pos::new(0, y, level), width, true);
                }
            }
        }
    }
}

/// Whether a hero walks onto a tile's terrain: land (ADV-MAP-001, ADV-MOV-007; water needs a boat,
/// rock is never entered).
pub(crate) fn enterable_terrain(part: &AdventureRuleset, tile: &StaticTile) -> bool {
    matches!(part.passage(tile.terrain), Some(Passage::Land(_)))
}

/// The map's terrain shares (ADV-MAP-002, E08 T29; the original's `ai_map_terrain_shares`
/// 0x52B9C0, survey `E08-progression.md` §10.3): over the tiles of every level that are not
/// impassable and that no object blocks (a tile an object is visited from counts: the original's
/// map item bit +0xC 0x40, set on every tile but rock by `map_tiles_load` 0x4FE220 and cleared by
/// `tile_refresh_object` 0x505610 where an object blocks it; not the coast, as T29 had it: E08
/// T38b, decision 85), water included, how many there are, the water tiles among them, and per
/// Pathfinding level 0 to 2 the tiles whose straight step costs more than a step on native
/// terrain ([`AdventureRuleset::step_cost`] without a road; the original's 100). Known terrains
/// only: the static map checks them first.
fn terrain_shares(part: &AdventureRuleset, tiles: &[StaticTile]) -> TerrainShares {
    // rule: ADV-MAP-002
    let native = part.params().native_cost();
    let mut shares = TerrainShares::default();
    let counted = tiles
        .iter()
        .filter(|tile| !tile.blocked || tile.visitable)
        .filter_map(|tile| part.passage(tile.terrain).map(|p| (tile.terrain, p)))
        .filter(|&(_, passage)| passage != Passage::Impassable);
    for (terrain, passage) in counted {
        shares.tiles = shares.tiles.saturating_add(1);
        let water = u32::from(matches!(passage, Passage::Water(_)));
        shares.water = shares.water.saturating_add(water);
        for (level, rough) in (0..).zip(&mut shares.rough) {
            let costly = part
                .step_cost(terrain, level)
                .is_some_and(|cost| cost > native);
            *rough = rough.saturating_add(u32::from(costly));
        }
    }
    shares
}

#[cfg(test)]
mod tests;

/// Whether an object's monster data fits its type (ADV-MON-001): `levels` is the type's monster
/// levels (`Some(None)` a fixed monster, `Some(Some(_))` a random one), none for an object that
/// is no monster, which has no data. A fixed monster names a creature of the core that is no war
/// machine, a random one none; the count is at most [`MAX_MONSTER_COUNT`]; the monsters'
/// parameters know the disposition; the treasure names known resources once each, each amount
/// from 1 to [`MAX_GAIN`], and an artifact a hero may hold (E08 T61a).
fn check_monster(
    core: &Ruleset,
    part: &AdventureRuleset,
    levels: Option<Option<[u8; 2]>>,
    monster: Option<&MonsterSetup>,
) -> bool {
    let (levels, monster) = match (levels, monster) {
        (None, None) => return true,
        (Some(levels), Some(monster)) => (levels, monster),
        _ => return false,
    };
    let creature = match (levels, monster.creature) {
        (None, Some(creature)) => core
            .creature(creature)
            .is_some_and(|c| !c.flags.contains(CreatureFlag::WarMachine)),
        (Some(_), None) => true,
        _ => false,
    };
    let resources = monster
        .treasure
        .iter()
        .map(|(resource, _)| resource)
        .collect::<BTreeSet<_>>();
    creature
        && monster.count <= MAX_MONSTER_COUNT
        && part
            .params()
            .monsters()
            .and_then(|m| m.aggression(monster.disposition))
            .is_some()
        && resources.len() == monster.treasure.len()
        && monster.treasure.iter().all(|&(resource, amount)| {
            usize::from(resource.raw()) < core.resources().len() && (1..=MAX_GAIN).contains(&amount)
        })
        && holds::check_treasure_artifact(core, monster.artifact)
}

/// A guarded object's guards checked (ADV-PIK-004, E08 T21a): on an object that takes them
/// (`guarded`: a pile; since E08 T61a an artifact, a scroll, Pandora's Box, an event), their
/// stacks valid as a hero's army ([`check_army`]) with no war machine; the stacks in slot order.
fn check_guards(
    core: &Ruleset,
    part: &AdventureRuleset,
    guarded: bool,
    guards: &Guards,
) -> Option<Guards> {
    let army = check_army(core, part, &guards.army).ok()?;
    let machine = army.iter().any(|stack| {
        core.creature(stack.creature)
            .is_some_and(|c| c.flags.contains(CreatureFlag::WarMachine))
    });
    (guarded && !machine).then_some(Guards {
        army,
        question: guards.question,
    })
}

/// One of `choices` with equal chances (`random_between(0, n − 1)` in the original); none for no
/// choices, which the compiler rules out.
fn pick<'a, T>(choices: &'a [T], draws: &mut Rng) -> Option<&'a T> {
    let n = u32::try_from(choices.len()).unwrap_or(u32::MAX);
    choices.get(usize::try_from(draws.below(n)).unwrap_or(usize::MAX))
}
