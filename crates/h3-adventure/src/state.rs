//! The adventure state (ADR-0016, E08 T5a): the static map, one block behind an `Arc` that every
//! fork shares and nothing writes after setup, and the small hot parts in plain arrays (the heroes
//! on the map with their armies). The persistent grids for per-tile layers that change (fog of
//! war) come with T5b. Nothing sees the sharing: equality, `Clone`, `Debug` and the canonical
//! hash see content only (ADR-0016 point 2).

use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use h3_content::{Ruleset, TerrainId};
use h3_contracts::{ArmyStack, HeroId, PlayerId};
use h3_kernel::hash::hash_of;
use h3_kernel::{CanonicalHash, StateHasher};

use crate::ADVENTURE_STATE_HASH_VERSION;
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
    /// anchors.
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
            if map.index(object.anchor).is_none() {
                return Err(SetupError::ObjectOutside(index));
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

/// A hero on the adventure map (ADR-0016 point 3): its id, owner, tile and army. The hero's
/// details stay in `h3-heroes`, kept by `h3-game`.
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
}

/// The adventure context's state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdventureState {
    map: Arc<StaticMap>,
    players: u8,
    heroes: Vec<HeroOnMap>,
}

impl AdventureState {
    /// Builds the state from a setup, checking every invariant commands keep: the map's bounds
    /// and tiles, the players' bound, and each hero's owner, tile (ADV-MAP-001) and army. The
    /// heroes are kept by id, their stacks by slot.
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
        let mut state = Self {
            map: Arc::new(map),
            players: setup.players,
            heroes: Vec::with_capacity(setup.heroes.len()),
        };
        for hero in &setup.heroes {
            state.place(core, part, hero)?;
        }
        Ok(state)
    }

    /// Checks and places one hero of the setup.
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
            },
        );
        Ok(())
    }

    /// Whether a hero may stand on `pos` (ADV-MAP-001): a tile of the map whose terrain is
    /// walked on land, which no object blocks and no other hero stands on. An object's
    /// visitable tile is allowed (a hero may stand on a town's entrance).
    ///
    /// # Errors
    ///
    /// Why it may not, checked in the order of [`StandError`].
    // rule: ADV-MAP-001
    pub fn check_stand(&self, part: &AdventureRuleset, pos: Pos) -> Result<(), StandError> {
        let tile = self.map.tile(pos).ok_or(StandError::Outside)?;
        if !matches!(part.passage(tile.terrain), Some(Passage::Land(_))) {
            return Err(StandError::Terrain);
        }
        if tile.blocked {
            return Err(StandError::Blocked);
        }
        if self.heroes.iter().any(|h| h.pos == pos) {
            return Err(StandError::Occupied);
        }
        Ok(())
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
    /// Encoding v1 (E08 T5a): version; the static map's content hash; the number of players;
    /// the heroes by id (id, owner, tile, the army's stacks by slot: slot, creature, count).
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
mod tests;
