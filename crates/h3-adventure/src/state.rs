//! The adventure state (ADR-0016, E08 T5a): the static map, one block behind an `Arc` that every
//! fork shares and nothing writes after setup (since T12b with the objects' random contents drawn
//! when it is built), the small hot parts in plain arrays (the heroes on the map with their
//! armies), each player's explored tiles as a persistent grid (T8a, on T5b's [`BitGrid`]) and the
//! objects that left the map with the tiles they freed (`occupancy.rs`, T12b); what a player sees
//! of it is the observation's (`observe.rs`, T8b1). Nothing sees the sharing: equality, `Clone`, `Debug` and the canonical hash
//! see content only (ADR-0016 point 2).

use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use h3_content::{FRINGE_SHOWS_OBJECTS, ResourceId, Ruleset, TerrainId};
use h3_contracts::{ArmyStack, HeroId, PlayerId};
use h3_kernel::hash::hash_of;
use h3_kernel::{CanonicalHash, Rng, StateHasher};

use crate::ADVENTURE_STATE_HASH_VERSION;
use crate::command::{
    AdventureCommand, AdventureCommandError, AdventureEvent, AdventureEventKind, AdventureOutcome,
    Offer, Sink, Viewers, bit,
};
use crate::fog::{on_fringe, reveal};
use crate::movement::{DIRECTIONS, Pricing, day_points, direction};
use crate::persistent::BitGrid;
use crate::ruleset::{
    AdventureRuleset, MAX_PICKUP_AMOUNT, ObjectKind, ObjectType, ObjectTypeId, Passage, Pickup,
    RiverId, RoadId, Visit,
};
use crate::setup::{
    AdventureSetup, ArmyError, HeroSetup, MapSetup, ObjectSetup, Pos, SetupError, StandError,
    TileMask,
};

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
}

impl StaticTile {
    /// What [`StaticMap::at`] gives for a tile off the map: no terrain the rules know, blocked.
    const OUTSIDE: Self = Self {
        terrain: TerrainId::from_raw(u16::MAX),
        road: None,
        river: None,
        blocked: true,
        visitable: false,
    };
}

mod occupancy;

pub(crate) use occupancy::Occupancy;

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
    /// resource with its amount, the type's multiplier applied; empty for every other kind.
    pub contents: Vec<(ResourceId, u32)>,
}

impl MapObject {
    /// Whether the object blocks `pos`, and whether it is visited from `pos`.
    pub fn covers(&self, pos: Pos) -> (bool, bool) {
        let column = self.anchor.x.checked_sub(pos.x).map(u8::try_from);
        let row = self.anchor.y.checked_sub(pos.y).map(u8::try_from);
        match (column, row) {
            (Some(Ok(column)), Some(Ok(row))) if self.anchor.level == pos.level => (
                self.blocked.contains(column, row),
                self.visitable.contains(column, row),
            ),
            _ => (false, false),
        }
    }

    /// The tiles of `map` it covers: its blocked and its visitable tiles, row by row (a tile both
    /// ways once).
    pub fn tiles<'a>(&self, map: &'a StaticMap) -> impl Iterator<Item = Pos> + use<'a> {
        let anchor = self.anchor;
        TileMask::from_bits(self.blocked.bits() | self.visitable.bits())
            .offsets()
            .filter_map(move |(column, row)| map.offset_pos(anchor, column, row))
    }
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
    hash: u64,
}

impl StaticMap {
    /// Builds the static map from its setup, checking the dimensions, the tiles and the objects
    /// (level, type, amount, their number), then draws the objects' random contents from `draws`.
    fn new(
        core: &Ruleset,
        part: &AdventureRuleset,
        setup: &MapSetup,
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
            });
        }
        let mut map = Self {
            width: setup.width,
            height: setup.height,
            levels: setup.levels,
            tiles,
            objects: Vec::with_capacity(setup.objects.len()),
            visits: BTreeMap::new(),
            typed_visits: vec![0; setup.tiles.len().div_ceil(64)],
            hash: 0,
        };
        if u16::try_from(setup.objects.len().div_ceil(Occupancy::ROW)).is_err() {
            return Err(SetupError::Objects(setup.objects.len()));
        }
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
            map.place(object);
        }
        map.resolve(part, &setup.objects, draws);
        map.hash = hash_of(&map);
        Ok(map)
    }

    /// Draws the objects' random contents (ADR-0016 point 4, ADR-0017 point 4) in the original's
    /// two walks, each in object order: first each random resource's pile type (ADV-PIK-002: one
    /// of its types with equal chances), then each pile's amount where the map fixes none and each
    /// paired pickup's amount and resource (ADV-PIK-001, ADV-PIK-003: the amount first). Piles
    /// take the map's amount, or the drawn one, times the multiplier.
    // rule: ADV-PIK-001, ADV-PIK-002, ADV-PIK-003
    fn resolve(&mut self, part: &AdventureRuleset, setup: &[ObjectSetup], draws: &mut Rng) {
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
    }

    /// Marks the tiles an object covers; tiles of its footprint off the map are ignored
    /// (ADV-OBJ-001).
    // rule: ADV-OBJ-001
    fn place(&mut self, object: &ObjectSetup) {
        for (column, row) in object.blocked.offsets() {
            if let Some(index) = self.offset(object.anchor, column, row) {
                self.tiles[index].blocked = true;
            }
        }
        for (column, row) in object.visitable.offsets() {
            if let Some(index) = self.offset(object.anchor, column, row) {
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

    /// The objects visited from `pos`, by index in map order, if a typed one is among them (none
    /// for a tile only blockers are visited from, or one off the map). A visit concerns the last
    /// of them still on the map ([`AdventureState::visit_at`], E08 T14a).
    pub fn visited_from(&self, pos: Pos) -> &[usize] {
        self.index(pos)
            .filter(|&index| self.typed_visits[index / 64] >> (index % 64) & 1 == 1)
            .and_then(|index| self.visits.get(&index))
            .map_or(&[], Vec::as_slice)
    }

    /// The canonical hash of the static map, computed once when it is built.
    pub const fn content_hash(&self) -> u64 {
        self.hash
    }
}

impl CanonicalHash for StaticMap {
    /// Width, height, levels; every tile (terrain, road, river); every object (anchor, blocked
    /// and visitable masks; since v4 its object type and contents). The per-tile object flags
    /// follow from the objects.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u16(self.width);
        h.write_u16(self.height);
        h.write_u8(self.levels);
        h.write_len(self.tiles.len());
        for tile in &self.tiles {
            tile.terrain.hash_into(h);
            tile.road.hash_into(h);
            tile.river.hash_into(h);
        }
        h.write_len(self.objects.len());
        for object in &self.objects {
            object.anchor.hash_into(h);
            h.write_u64(object.blocked.bits());
            h.write_u64(object.visitable.bits());
            object.object_type.hash_into(h);
            h.write_len(object.contents.len());
            for (resource, amount) in &object.contents {
                resource.hash_into(h);
                h.write_u32(*amount);
            }
        }
    }
}

impl CanonicalHash for Pos {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u16(self.x);
        h.write_u16(self.y);
        h.write_u8(self.level);
    }
}

/// What a step visits (ADV-OBJ-003): the object, by index in the static map, its type, how it
/// is visited and whether it may be from above (ADV-OBJ-004).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StepVisit {
    pub(crate) object: usize,
    pub(crate) object_type: ObjectTypeId,
    pub(crate) mode: Visit,
    pub(crate) from_above: bool,
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
    /// The movement points it has left today.
    pub movement: u32,
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
    /// The Cartographer types bought (ADV-VIS-001, E08 T14b), in type order, each with a bit per
    /// player that bought it (bit `n` for the player with id `n`): the per-player state of object
    /// types as one small plain array (ADR-0016 point 1), laid out as the original's flags, a byte
    /// of player bits per kind (game +0x4E372). None until a first purchase, then shared by the
    /// forks (a purchase builds it anew), so a fork costs nothing more before and a reference
    /// count after (`bench-check`).
    bought: Option<Arc<[(ObjectTypeId, u8)]>>,
}

impl AdventureState {
    /// Builds the state from a setup, checking every invariant commands keep: the map's bounds
    /// and tiles, the objects, the players' bound, and each hero's owner, tile (ADV-MAP-001) and
    /// army. The objects' random contents are drawn from `draws`, the game's setup stream
    /// (ADR-0017 point 4); every draw is valid, so whether a setup is valid does not depend on
    /// them. The heroes are kept by id, their stacks by slot. Each hero reveals the tiles around
    /// it to its owner (ADV-FOG-003); nothing else is explored, and every object is on the map.
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
        let map = StaticMap::new(core, part, &setup.map, draws)?;
        if !(1..=part.params().max_players()).contains(&setup.players) {
            return Err(SetupError::Players(setup.players));
        }
        let unexplored = BitGrid::new(map.width(), map.height(), map.levels());
        let mut state = Self {
            map: Arc::new(map),
            players: setup.players,
            heroes: Vec::with_capacity(setup.heroes.len()),
            explored: vec![unexplored; usize::from(setup.players)],
            occupancy: None,
            bought: None,
        };
        for hero in &setup.heroes {
            state.place(core, part, hero)?;
        }
        state.reveal_at_start(part);
        Ok(state)
    }

    /// The start of the game: each hero reveals the tiles around it to its owner (ADV-FOG-003;
    /// the original when it sets a placed hero up, 0x486590, and again around everything a
    /// player owns at each new day, 0x4C7C50, which adds nothing while no rule hides a tile).
    /// Increment 1 has no owned objects: a game opens with only its heroes' surroundings
    /// explored.
    // rule: ADV-FOG-001, ADV-FOG-003
    fn reveal_at_start(&mut self, part: &AdventureRuleset) {
        let radius = part.params().scouting_radius();
        for hero in &self.heroes {
            reveal(
                &mut self.explored[usize::from(hero.owner.raw())],
                hero.pos,
                radius,
            );
        }
    }

    /// Checks and places one hero of the setup, with the day's movement points for its army
    /// (the original computes them when it sets a placed hero up, 0x486590; its owner's turn
    /// sets them again, [`Self::start_turn`]).
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
        let movement = day_points(core, part, &army);
        self.heroes.insert(
            at,
            HeroOnMap {
                id: hero.id,
                owner: hero.owner,
                pos: hero.pos,
                army,
                movement,
            },
        );
        Ok(())
    }

    /// The start of `player`'s turn (ADR-0017 point 3), a step only the game triggers: each of
    /// its heroes gets the day's movement points for its army; what was left is lost.
    // rule: ADV-MOV-002
    pub fn start_turn(&mut self, core: &Ruleset, part: &AdventureRuleset, player: PlayerId) {
        for hero in self.heroes.iter_mut().filter(|h| h.owner == player) {
            hero.movement = day_points(core, part, &hero.army);
        }
    }

    /// Whether a hero may stand on `pos` (ADV-MAP-001): a tile of the map whose terrain is
    /// walked on land, which no object blocks unless an object is visited from it, and no other
    /// hero stands on. A visitable tile is allowed even where a mask also blocks it (a hero may
    /// stand on a town's entrance, which the town's masks mark both ways), unless its object is
    /// visited from a neighbouring tile: no move ever enters such a tile (ADV-OBJ-003), so a setup
    /// may not place a hero there either (E08 T14a).
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
    /// what the player gains beyond the map is the outcome (ADR-0017 point 10).
    ///
    /// # Errors
    ///
    /// An [`AdventureCommandError`]; the state is then unchanged and no event was emitted.
    pub fn apply<S: Sink>(
        &mut self,
        core: &Ruleset,
        part: &AdventureRuleset,
        player: PlayerId,
        command: &AdventureCommand,
        sink: &mut S,
    ) -> Result<AdventureOutcome, AdventureCommandError> {
        match command {
            AdventureCommand::MoveHero { hero, path } => {
                self.move_hero(core, part, player, *hero, path, sink)
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
    /// stays on the tile before.
    // rule: ADV-MOV-006, ADV-MOV-007, ADV-FOG-003, ADV-OBJ-003
    fn move_hero<S: Sink>(
        &mut self,
        core: &Ruleset,
        part: &AdventureRuleset,
        player: PlayerId,
        id: HeroId,
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
        let pricing = Pricing::new(core, part, &hero.army);
        let start = hero.pos;
        let (mut at, mut from, mut points) = (start, self.map.at(start), hero.movement);
        let mut visit = None;
        for (step, &next) in path.iter().enumerate() {
            if visit.is_some() {
                return Err(AdventureCommandError::PastVisit { step });
            }
            let direction =
                direction(at, next).ok_or(AdventureCommandError::NotNeighbour { step })?;
            let (to, visited) = self.check_step(part, player, id, (at, next), direction, step)?;
            let cost = pricing.step(&from, &to, direction % 2 == 1, points);
            let left = points
                .checked_sub(cost)
                .ok_or(AdventureCommandError::TooFewPoints { step, cost, points })?;
            points = pricing.left_after(&to, left);
            if visited.is_none_or(|v| v.mode == Visit::Enter) {
                (at, from) = (next, to);
            }
            visit = visited;
        }
        let stays = visit.is_some_and(|v| v.mode == Visit::Neighbour);
        let walked = &path[..path.len() - usize::from(stays)];
        let hero = &mut self.heroes[index];
        hero.pos = at;
        hero.movement = points;
        let radius = part.params().scouting_radius();
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
        Ok(visit.map_or_else(AdventureOutcome::default, |visit| {
            self.visit(core, part, (player, id), visit, sink)
        }))
    }

    /// An object's visit by `player`'s hero `hero`, by the object's kind. A pickup (ADV-PIK-001,
    /// ADV-PIK-003): the visitor's owner gains its contents, and it leaves the map
    /// ([`Self::remove_object`], ADV-OBJ-005), told to the players that see it there. A
    /// Cartographer, the other kind whose visit is built (ADV-VIS-001, E08 T14b): an offer of its
    /// reveal for its price, unless the player has bought its type; the game opens it if the
    /// player can pay (ADR-0017 point 10).
    // rule: ADV-PIK-001, ADV-PIK-003, ADV-OBJ-005, ADV-VIS-001
    fn visit<S: Sink>(
        &mut self,
        core: &Ruleset,
        part: &AdventureRuleset,
        (player, hero): (PlayerId, HeroId),
        visit: StepVisit,
        sink: &mut S,
    ) -> AdventureOutcome {
        let kind = part.object_type(visit.object_type).map(|ty| &ty.kind);
        let object = &self.map.objects()[visit.object];
        if let Some(ObjectKind::Pickup(_)) = kind {
            let gains = object.contents.clone();
            sink.emit(AdventureEvent {
                kind: AdventureEventKind::ObjectRemoved {
                    object_type: visit.object_type,
                    anchor: object.anchor,
                },
                seen_by: self.object_viewers(core, object),
            });
            self.remove_object(visit.object);
            return AdventureOutcome { gains, offer: None };
        }
        let offer = kind
            .and_then(ObjectKind::reveal_terrain)
            .filter(|_| !self.has_bought(player, visit.object_type))
            .map(|(_, price)| Offer {
                hero,
                object_type: visit.object_type,
                anchor: object.anchor,
                price: price.to_vec(),
            });
        AdventureOutcome {
            gains: Vec::new(),
            offer,
        }
    }

    /// Accepts `offer`, made to `player` by a visit (ADR-0017 point 10: the game has paid its
    /// price): every tile of the map, on every level, whose terrain is among the Cartographer's
    /// terrains becomes explored for `player`, told to it alone with the number of tiles it had
    /// not explored, and the player has bought the type, so no later visit of an object of the
    /// type makes it an offer (ADV-VIS-001). Infallible, and with no check that the offer still
    /// stands: it was made for a type the player had not bought, and while it is open nothing of
    /// the player's but its answer runs (ADR-0017 point 6) and no other player's command buys for
    /// it.
    // rule: ADV-VIS-001
    pub fn accept_offer<S: Sink>(
        &mut self,
        part: &AdventureRuleset,
        player: PlayerId,
        offer: &Offer,
        sink: &mut S,
    ) {
        let terrains = part
            .object_type(offer.object_type)
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
                object_type: offer.object_type,
                count: grid.count() - before,
            },
            seen_by: Viewers::NONE.with(player),
        });
        let player = bit(player);
        let mut bought = self.bought.as_deref().unwrap_or_default().to_vec();
        match bought.binary_search_by_key(&offer.object_type, |&(ty, _)| ty) {
            Ok(at) => bought[at].1 |= player,
            Err(at) => bought.insert(at, (offer.object_type, player)),
        }
        self.bought = Some(bought.into());
    }

    /// Whether `player` has bought the Cartographer type `object_type` (ADV-VIS-001).
    pub fn has_bought(&self, player: PlayerId, object_type: ObjectTypeId) -> bool {
        let bought = self.bought.as_deref().unwrap_or_default();
        bought
            .binary_search_by_key(&object_type, |&(ty, _)| ty)
            .is_ok_and(|at| bought[at].1 & bit(player) != 0)
    }

    /// The Cartographer types `player` has bought, in type order (ADV-VIS-001).
    pub fn bought(&self, player: PlayerId) -> impl Iterator<Item = ObjectTypeId> + '_ {
        let player = bit(player);
        self.bought
            .as_deref()
            .unwrap_or_default()
            .iter()
            .filter(move |&&(_, players)| players & player != 0)
            .map(|&(ty, _)| ty)
    }

    /// Who sees `object` on the map, as the observation shows it (ADV-FOG-004, ADV-FOG-005):
    /// every player that has explored a tile it covers and, under `ADV-QUIRK-1`, every player
    /// with such a tile on its fringe.
    fn object_viewers(&self, core: &Ruleset, object: &MapObject) -> Viewers {
        let fringe = core.quirks().is_enabled(FRINGE_SHOWS_OBJECTS);
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
    /// the map that is visited from `pos`, if it has a type (E08 T14a; ADV-OBJ-003). The original
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
        let index = self
            .map
            .visited_from(pos)
            .iter()
            .rev()
            .copied()
            .find(|&index| !self.is_removed(index))?;
        let id = self.map.objects()[index].object_type?;
        part.object_type(id).map(|ty| (index, id, ty))
    }

    /// What a step onto `pos` visits, if its visit is built ([`ObjectKind::visit_built`]): the
    /// object of [`Self::tile_object`] and how it is visited. The caller has checked that `pos`
    /// is visitable as it stands now.
    pub(crate) fn visit_at(&self, part: &AdventureRuleset, pos: Pos) -> Option<StepVisit> {
        self.tile_object(part, pos)
            .filter(|(_, _, ty)| ty.kind.visit_built())
            .map(|(object, object_type, ty)| StepVisit {
                object,
                object_type,
                mode: ty.visit,
                from_above: ty.from_above,
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
    /// since T14a). The explored test comes first, so a rejection tells nothing about a hidden
    /// tile.
    // rule: ADV-MOV-007, ADV-MOV-008, ADV-OBJ-004
    fn check_step(
        &self,
        part: &AdventureRuleset,
        player: PlayerId,
        mover: HeroId,
        (from, pos): (Pos, Pos),
        direction: usize,
        step: usize,
    ) -> Result<(StaticTile, Option<StepVisit>), AdventureCommandError> {
        if self.map.tile(pos).is_some() && !self.is_explored(player, pos) {
            return Err(AdventureCommandError::Unexplored { step });
        }
        let tile = self
            .check_tile(part, pos, Some(mover))
            .map_err(|error| AdventureCommandError::Blocked { step, error })?;
        // A step down enters `pos` from above; a step up leaves `from` across its top.
        let across = match DIRECTIONS[direction].1 {
            1 => Some(pos),
            -1 => Some(from),
            _ => None,
        };
        if across.is_some_and(|at| self.closed_above(part, at)) {
            return Err(AdventureCommandError::FromAbove { step });
        }
        if !tile.visitable {
            return Ok((tile, None));
        }
        self.visit_at(part, pos)
            .map(|object| (tile, Some(object)))
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
    /// blocked or visitable only where an object still on the map covers it so.
    #[inline]
    pub(crate) fn live(&self, pos: Pos, tile: StaticTile) -> StaticTile {
        match &self.occupancy {
            Some(occupancy) => occupancy.live(pos, tile),
            None => tile,
        }
    }

    /// Whether the object at `index` of the static map has left the map (no object has before
    /// increment 2's visits).
    pub fn is_removed(&self, index: usize) -> bool {
        self.occupancy
            .as_ref()
            .is_some_and(|occupancy| occupancy.is_removed(index))
    }

    /// Takes the object at `index` of the static map off the map (ADR-0016 point 1): the tiles it
    /// covered that no other object still on the map covers so become free; the static map keeps
    /// its footprint.
    pub(crate) fn remove_object(&mut self, index: usize) {
        if index >= self.map.objects().len() || self.is_removed(index) {
            return;
        }
        let map = &self.map;
        let occupancy = self
            .occupancy
            .get_or_insert_with(|| Arc::new(Occupancy::new(map)));
        Arc::make_mut(occupancy).remove(map, index);
    }

    /// Whether `player` has explored the tile at `pos` (a tile off the map, or a player not in
    /// the game, has none).
    pub fn is_explored(&self, player: PlayerId, pos: Pos) -> bool {
        self.explored(player).is_some_and(|grid| grid.get(pos))
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
    /// Encoding v5 (E08 T14b; v1 was T5a's, v2 T6b's, v3 T8a's, v4 T12's): version; the static
    /// map's content hash (since v4 with the objects' types and contents); the number of players;
    /// the heroes by id (id, owner, tile, the army's stacks by slot: slot, creature, count; since
    /// v2 the movement points left); since v3 each player's explored tiles, by player id
    /// ([`BitGrid`]'s encoding); since v4 the objects that left the map (0 for none, else 1 and
    /// [`Occupancy`]'s encoding; there is an occupancy only once an object has left); since v5 the
    /// Cartographer types bought, as length and entries in type order (the type, then its byte of
    /// player bits).
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
            h.write_u32(hero.movement);
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
        let bought = self.bought.as_deref().unwrap_or_default();
        h.write_len(bought.len());
        for (object_type, players) in bought {
            object_type.hash_into(h);
            h.write_u8(*players);
        }
    }
}

/// Checks an army against the ruleset (one or more stacks of known creatures with at least one
/// creature each, in distinct slots below the army slots) and returns its stacks by slot.
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

#[cfg(test)]
mod tests;

/// One of `choices` with equal chances (`random_between(0, n − 1)` in the original); none for no
/// choices, which the compiler rules out.
fn pick<'a, T>(choices: &'a [T], draws: &mut Rng) -> Option<&'a T> {
    let n = u32::try_from(choices.len()).unwrap_or(u32::MAX);
    choices.get(usize::try_from(draws.below(n)).unwrap_or(usize::MAX))
}
