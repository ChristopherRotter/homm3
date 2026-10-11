//! The adventure state (ADR-0016, E08 T5a): the static map, one block behind an `Arc` that every
//! fork shares and nothing writes after setup, the small hot parts in plain arrays (the heroes on
//! the map with their armies), and each player's explored tiles as a persistent grid (T8a, on
//! T5b's [`BitGrid`]); what a player sees of it is the observation's (`observe.rs`, T8b1). Nothing sees the sharing: equality, `Clone`, `Debug` and the canonical hash
//! see content only (ADR-0016 point 2).

use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use h3_content::{FRINGE_SHOWS_OBJECTS, Ruleset, TerrainId};
use h3_contracts::{ArmyStack, HeroId, PlayerId};
use h3_kernel::hash::hash_of;
use h3_kernel::{CanonicalHash, StateHasher};

use crate::ADVENTURE_STATE_HASH_VERSION;
use crate::command::{
    AdventureCommand, AdventureCommandError, AdventureEvent, AdventureEventKind, Sink, Viewers,
};
use crate::fog::{on_fringe, reveal};
use crate::movement::{Pricing, day_points, direction};
use crate::persistent::BitGrid;
use crate::ruleset::{AdventureRuleset, Passage, RiverId, RoadId};
use crate::setup::{
    AdventureSetup, ArmyError, HeroSetup, MapSetup, ObjectKind, ObjectSetup, Pos, SetupError,
    StandError, TileMask,
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

/// An object of the static map: its position, footprint and kind (ADR-0016 point 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapObject {
    /// The bottom right tile its masks are relative to.
    pub anchor: Pos,
    /// The tiles no hero enters.
    pub blocked: TileMask,
    /// The tiles a hero visits it from.
    pub visitable: TileMask,
    /// Its behaviour.
    pub kind: ObjectKind,
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
    hash: u64,
}

impl StaticMap {
    /// Builds the static map from its setup, checking the dimensions, the tiles and the objects'
    /// levels.
    fn new(core: &Ruleset, part: &AdventureRuleset, setup: &MapSetup) -> Result<Self, SetupError> {
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
            hash: 0,
        };
        for (index, object) in setup.objects.iter().enumerate() {
            if object.anchor.level >= map.levels {
                return Err(SetupError::ObjectLevel(index));
            }
            map.place(object);
        }
        map.hash = hash_of(&map);
        Ok(map)
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
            }
        }
        self.objects.push(MapObject {
            anchor: object.anchor,
            blocked: object.blocked,
            visitable: object.visitable,
            kind: object.kind,
        });
    }

    /// The index of the tile `column` columns left of and `row` rows above `anchor`, if it is
    /// on the map.
    fn offset(&self, anchor: Pos, column: u8, row: u8) -> Option<usize> {
        let x = anchor.x.checked_sub(u16::from(column))?;
        let y = anchor.y.checked_sub(u16::from(row))?;
        self.index(Pos::new(x, y, anchor.level))
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

    /// The canonical hash of the static map, computed once when it is built.
    pub const fn content_hash(&self) -> u64 {
        self.hash
    }
}

impl CanonicalHash for StaticMap {
    /// Width, height, levels; every tile (terrain, road, river); every object (anchor, blocked
    /// and visitable masks, kind). The per-tile object flags follow from the objects.
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
            h.write_u8(match object.kind {
                ObjectKind::Blocker => 0,
            });
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
}

impl AdventureState {
    /// Builds the state from a setup, checking every invariant commands keep: the map's bounds
    /// and tiles, the players' bound, and each hero's owner, tile (ADV-MAP-001) and army. The
    /// heroes are kept by id, their stacks by slot. Each hero reveals the tiles around it to its
    /// owner (ADV-FOG-003); nothing else is explored.
    ///
    /// # Errors
    ///
    /// The first [`SetupError`] found, in that order.
    pub fn new(
        core: &Ruleset,
        part: &AdventureRuleset,
        setup: &AdventureSetup,
    ) -> Result<Self, SetupError> {
        let map = StaticMap::new(core, part, &setup.map)?;
        if !(1..=part.params().max_players()).contains(&setup.players) {
            return Err(SetupError::Players(setup.players));
        }
        let unexplored = BitGrid::new(map.width(), map.height(), map.levels());
        let mut state = Self {
            map: Arc::new(map),
            players: setup.players,
            heroes: Vec::with_capacity(setup.heroes.len()),
            explored: vec![unexplored; usize::from(setup.players)],
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
    /// stand on a town's entrance, which the town's masks mark both ways).
    ///
    /// # Errors
    ///
    /// Why it may not, checked in the order of [`StandError`].
    // rule: ADV-MAP-001
    pub fn check_stand(&self, part: &AdventureRuleset, pos: Pos) -> Result<(), StandError> {
        self.check_tile(part, pos, None).map(|_| ())
    }

    /// [`Self::check_stand`] for `mover`, which may stand where it already stands; the tile.
    fn check_tile(
        &self,
        part: &AdventureRuleset,
        pos: Pos,
        mover: Option<HeroId>,
    ) -> Result<StaticTile, StandError> {
        let tile = *self.map.tile(pos).ok_or(StandError::Outside)?;
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
    /// turn is running): validated against the state, then applied; the events go to `sink`.
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
    ) -> Result<(), AdventureCommandError> {
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
    /// tiles (ADV-FOG-004, ADV-FOG-005).
    // rule: ADV-MOV-006, ADV-MOV-007, ADV-FOG-003
    fn move_hero<S: Sink>(
        &mut self,
        core: &Ruleset,
        part: &AdventureRuleset,
        player: PlayerId,
        id: HeroId,
        path: &[Pos],
        sink: &mut S,
    ) -> Result<(), AdventureCommandError> {
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
        for (step, &next) in path.iter().enumerate() {
            let diagonal =
                direction(at, next).ok_or(AdventureCommandError::NotNeighbour { step })? % 2 == 1;
            let to = self.check_step(part, player, id, next, step)?;
            let cost = pricing.step(&from, &to, diagonal, points);
            let left = points
                .checked_sub(cost)
                .ok_or(AdventureCommandError::TooFewPoints { step, cost, points })?;
            points = pricing.left_after(&to, left);
            (at, from) = (next, to);
        }
        let hero = &mut self.heroes[index];
        hero.pos = at;
        hero.movement = points;
        let radius = part.params().scouting_radius();
        let mut from = start;
        for &to in path {
            reveal(&mut self.explored[usize::from(player.raw())], to, radius);
            sink.emit(AdventureEvent {
                kind: AdventureEventKind::HeroStepped { hero: id, from, to },
                seen_by: self.viewers(core, player, &[from, to]),
            });
            from = to;
        }
        Ok(())
    }

    /// Who sees a hero do something on `tiles`, tiles of the map (ADV-FOG-004, ADV-FOG-005):
    /// `owner`, every player that has explored one of the tiles and, under `ADV-QUIRK-1`, every
    /// player with one of them on its fringe, where it sees heroes too (the user, 2026-10-11).
    // rule: ADV-FOG-004, ADV-FOG-005
    fn viewers(&self, core: &Ruleset, owner: PlayerId, tiles: &[Pos]) -> Viewers {
        let fringe = core.quirks().is_enabled(FRINGE_SHOWS_OBJECTS);
        self.explored
            .iter()
            .zip((0..self.players).map(PlayerId::new))
            .filter(|(grid, player)| {
                *player == owner
                    || tiles
                        .iter()
                        .any(|&pos| grid.get(pos) || (fringe && on_fringe(grid, pos)))
            })
            .fold(Viewers::NONE, |viewers, (_, player)| viewers.with(player))
    }

    /// Whether `player`'s hero `mover` may step onto `pos`: a tile the player has explored
    /// (ADV-MOV-008; a tile off the map is not asked about, the map's size being public), then
    /// one it may stand on, which in increment 1 no object is visited from (ADV-MOV-007,
    /// ADR-0016 point 4); the tile. The explored test comes first, so a rejection tells nothing
    /// about a hidden tile.
    // rule: ADV-MOV-007, ADV-MOV-008
    fn check_step(
        &self,
        part: &AdventureRuleset,
        player: PlayerId,
        mover: HeroId,
        pos: Pos,
        step: usize,
    ) -> Result<StaticTile, AdventureCommandError> {
        if self.map.tile(pos).is_some() && !self.is_explored(player, pos) {
            return Err(AdventureCommandError::Unexplored { step });
        }
        let tile = self
            .check_tile(part, pos, Some(mover))
            .map_err(|error| AdventureCommandError::Blocked { step, error })?;
        if tile.visitable {
            return Err(AdventureCommandError::Visit { step });
        }
        Ok(tile)
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
    /// Encoding v3 (E08 T8a; v1 was T5a's, v2 T6b's): version; the static map's content hash;
    /// the number of players; the heroes by id (id, owner, tile, the army's stacks by slot: slot,
    /// creature, count; since v2 the movement points left); since v3 each player's explored
    /// tiles, by player id ([`BitGrid`]'s encoding).
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
