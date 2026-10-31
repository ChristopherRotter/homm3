//! Per-object state and the live presence (ADR-0016 point 1, E08 T12b; E08 T83b2a, increment 9's
//! design note §6): which objects have left the map, which tiles the static map marks blocked or
//! visitable that no object still on the map covers so any more, and the objects added in play (a
//! creature month's monsters, T85a), after the static map's in map order. Persistent grids
//! ([`BitGrid`]) and one shared list of the added objects, so a fork shares them and a removal
//! copies a leaf or two; the static map keeps every footprint. The state holds one only once an
//! object has left the map or one was added. Objects leave the map by T13's visits (a pickup, a
//! border guard); until then nothing removes one.

use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_kernel::{CanonicalHash, StateHasher};

use super::{AdventureState, MapObject, StaticMap, StaticTile};
use crate::persistent::BitGrid;
use crate::ruleset::{AdventureRuleset, MonsterParams, ObjectKind, Passage};
use crate::setup::Pos;

/// The objects that left the map, the tiles they freed, and the objects added in play.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Occupancy {
    /// The objects that left the map, by index: object `i` is column `i % ROW` of row `i / ROW`;
    /// the added objects' indices follow the static map's (one more row as they need it).
    removed: BitGrid,
    /// The tiles the static map marks blocked that no object still on the map blocks.
    unblocked: BitGrid,
    /// The tiles the static map marks visitable that no object still on the map is visited from.
    unvisitable: BitGrid,
    /// How many objects the static map holds: the first added object's index.
    base: usize,
    /// The objects added in play, in the order they came (index [`Self::base`] + n; E08 T83b2a):
    /// shared by the forks, so a removal's copy of the occupancy copies none of them.
    added: Arc<Vec<MapObject>>,
    /// The tiles an added object blocks or is visited from, left or not: where [`Self::live`]
    /// and [`Self::added_visited_from`] look at the added objects at all.
    added_tiles: BitGrid,
    /// The tiles within one step of an added monster's visitable tile on its level, left or not
    /// (its 3 × 3, as [`StaticMap::near_monster`] marks the placed ones): where a guard's search
    /// looks at the added objects at all.
    added_near: BitGrid,
}

impl Occupancy {
    /// Objects per row of [`Self::removed`]: a leaf's side, so a row of objects is one word.
    pub(crate) const ROW: usize = BitGrid::<16>::SIDE as usize;

    /// Every object on the map, none added. The static map holds at most `ROW × u16::MAX` objects
    /// ([`crate::SetupError::Objects`]).
    pub(crate) fn new(map: &StaticMap) -> Self {
        let base = map.objects().len();
        let grid = || BitGrid::new(map.width(), map.height(), map.levels());
        Self {
            removed: BitGrid::new(BitGrid::<16>::SIDE, Self::rows(base), 1),
            unblocked: grid(),
            unvisitable: grid(),
            base,
            added: Arc::default(),
            added_tiles: grid(),
            added_near: grid(),
        }
    }

    /// The rows of [`Self::removed`] that `objects` objects need.
    fn rows(objects: usize) -> u16 {
        u16::try_from(objects.div_ceil(Self::ROW)).unwrap_or(u16::MAX)
    }

    /// The grid tile standing for object `index`.
    fn slot(index: usize) -> Pos {
        let column = u16::try_from(index % Self::ROW).unwrap_or(u16::MAX);
        Pos::new(
            column,
            u16::try_from(index / Self::ROW).unwrap_or(u16::MAX),
            0,
        )
    }

    /// Whether the object at `index` has left the map.
    pub(crate) fn is_removed(&self, index: usize) -> bool {
        self.removed.get(Self::slot(index))
    }

    /// The objects added in play, by index less [`Self::base`].
    pub(crate) fn added(&self) -> &[MapObject] {
        &self.added
    }

    /// Whether an object was added in play (one that left counts).
    pub(crate) fn any_added(&self) -> bool {
        !self.added.is_empty()
    }

    /// The tiles an added object blocks or is visited from ([`Self::added_tiles`]).
    pub(crate) const fn added_tiles(&self) -> &BitGrid {
        &self.added_tiles
    }

    /// Whether an added monster's visitable tile lies within one step of `pos` on its level.
    pub(crate) fn near_added(&self, pos: Pos) -> bool {
        self.added_near.get(pos)
    }

    /// The added objects still on the map with their indices.
    fn added_on_map(&self) -> impl Iterator<Item = (usize, &MapObject)> {
        (self.base..)
            .zip(self.added.iter())
            .filter(|&(index, _)| !self.is_removed(index))
    }

    /// The last added object still on the map that is visited from `pos`, by index (ADV-OBJ-003:
    /// the added objects come after the static map's in map order); none where no added object
    /// covers the tile.
    pub(crate) fn added_visited_from(&self, pos: Pos) -> Option<usize> {
        if !self.added_tiles.get(pos) {
            return None;
        }
        self.added_on_map()
            .filter(|(_, object)| object.covers(pos).1)
            .last()
            .map(|(index, _)| index)
    }

    /// `tile`, the static map's tile at `pos`, blocked and visitable only where an object still
    /// on the map covers it so: the static map's marks less the freed tiles, then an added
    /// object's where one covers the tile. The grids are read only where the static tile is
    /// marked or an added object covers it.
    pub(crate) fn live(&self, pos: Pos, mut tile: StaticTile) -> StaticTile {
        tile.blocked = tile.blocked && !self.unblocked.get(pos);
        tile.visitable = tile.visitable && !self.unvisitable.get(pos);
        if self.added_tiles.get(pos) {
            let (blocked, visitable) = self.added_cover(pos);
            tile.blocked |= blocked;
            tile.visitable |= visitable;
        }
        tile
    }

    /// Whether an added object still on the map blocks `pos`, and whether one is visited from it.
    fn added_cover(&self, pos: Pos) -> (bool, bool) {
        self.added_on_map()
            .map(|(_, object)| object.covers(pos))
            .fold((false, false), |(b, v), (ob, ov)| (b || ob, v || ov))
    }

    /// Adds `object` to the map (E08 T83b2a; increment 9's design note §6: a creature month's
    /// monster, T85a), after every object before it in map order, and returns its index: its
    /// tiles marked in [`Self::added_tiles`], a monster's 3 × 3 around each visitable tile in
    /// [`Self::added_near`]. The caller has checked that it is a monster on land tiles no other
    /// object still on the map covers.
    #[cfg(test)]
    pub(crate) fn add(&mut self, map: &StaticMap, object: MapObject) -> usize {
        let index = self.base + self.added.len();
        if index.is_multiple_of(Self::ROW) {
            let mut removed = BitGrid::new(BitGrid::<16>::SIDE, Self::rows(index + 1), 1);
            for left in (0..index).filter(|&i| self.is_removed(i)) {
                removed.set(Self::slot(left), true);
            }
            self.removed = removed;
        }
        for pos in object.tiles(map) {
            let (blocked, visitable) = object.covers(pos);
            if blocked || visitable {
                self.added_tiles.set(pos, true);
            }
            if visitable && object.monster.is_some() {
                for (dx, dy) in crate::movement::DIRECTIONS.iter().copied().chain([(0, 0)]) {
                    let x = pos.x.checked_add_signed(i16::from(dx));
                    let y = pos.y.checked_add_signed(i16::from(dy));
                    if let (Some(x), Some(y)) = (x, y) {
                        self.added_near.set(Pos::new(x, y, pos.level), true);
                    }
                }
            }
        }
        Arc::make_mut(&mut self.added).push(object);
        index
    }

    /// Takes the object at `index`, one of the map's or the added ones still on it, off the map
    /// (ADR-0016 point 1): each of its tiles that no other object still on the map blocks
    /// becomes unblocked, and each that no other object is visited from becomes unvisitable. An
    /// added object's tiles stay in its grids: [`Self::live`] reads the objects still on the map
    /// there.
    pub(crate) fn remove(&mut self, map: &StaticMap, index: usize) {
        self.removed.set(Self::slot(index), true);
        let object = map
            .objects()
            .get(index)
            .unwrap_or_else(|| &self.added[index - self.base]);
        let freed: Vec<(Pos, bool, bool)> = object
            .tiles(map)
            .map(|pos| {
                let (blocked, visitable) = object.covers(pos);
                let statics = map
                    .objects()
                    .iter()
                    .enumerate()
                    .filter(|&(other, _)| !self.is_removed(other))
                    .map(|(_, other)| other.covers(pos));
                let added = self.added_on_map().map(|(_, other)| other.covers(pos));
                let others = statics
                    .chain(added)
                    .fold((false, false), |(b, v), (ob, ov)| (b || ob, v || ov));
                (pos, blocked && !others.0, visitable && !others.1)
            })
            .collect();
        for (pos, unblocked, unvisitable) in freed {
            if unblocked {
                self.unblocked.set(pos, true);
            }
            if unvisitable {
                self.unvisitable.set(pos, true);
            }
        }
    }
}

impl Occupancy {
    /// The added objects' invariants ([`AdventureState::check_added`]): each still on the map a
    /// typed monster covering at least one tile, each a land tile of `map` marked in
    /// [`Self::added_tiles`] (a visitable one in [`Self::added_near`] too) that no other object
    /// still on the map blocks or is visited from.
    fn check_added(&self, part: &AdventureRuleset, map: &StaticMap) -> bool {
        let land = |pos: Pos| matches!(part.passage(map.at(pos).terrain), Some(Passage::Land(_)));
        self.added_on_map().all(|(index, object)| {
            let monster = object.monster.is_some()
                && object
                    .object_type
                    .and_then(|id| part.object_type(id))
                    .is_some_and(|ty| matches!(ty.kind, ObjectKind::Monster { .. }));
            let free = |pos: Pos| {
                let statics = map
                    .objects()
                    .iter()
                    .enumerate()
                    .filter(|&(other, _)| !self.is_removed(other));
                let added = self.added_on_map().filter(|&(other, _)| other != index);
                statics
                    .chain(added)
                    .all(|(_, other)| other.covers(pos) == (false, false))
            };
            let marked = |pos: Pos| {
                self.added_tiles.get(pos) && (!object.covers(pos).1 || self.added_near.get(pos))
            };
            monster
                && object.tiles(map).next().is_some()
                && object
                    .tiles(map)
                    .all(|pos| land(pos) && marked(pos) && free(pos))
        })
    }
}

impl AdventureState {
    /// The monsters' fractions' and the objects added in play's invariants (E08 T83b2a;
    /// increment 9's design note §6), which every command keeps: a fraction only on a wandering
    /// monster still on the map, above 0 and below the monsters' growth unit; each added object
    /// still on the map a typed monster on land tiles that no other object still on the map
    /// covers ([`Occupancy::check_added`]).
    pub fn check_added(&self, part: &AdventureRuleset) -> bool {
        let unit = part
            .params()
            .monsters()
            .map_or(0, MonsterParams::growth_unit);
        let fractions = self.fractions.iter().all(|(index, &fraction)| {
            fraction > 0
                && u16::from(fraction) < unit
                && !self.is_removed(index)
                && self
                    .object(index)
                    .is_some_and(|object| object.monster.is_some())
        });
        fractions
            && self
                .occupancy
                .as_deref()
                .is_none_or(|occupancy| occupancy.check_added(part, &self.map))
    }
}

impl CanonicalHash for Occupancy {
    /// The removed objects ([`BitGrid`]'s encoding); since adventure state hash v19 (E08 T83b2a)
    /// the added objects as their number and each one's encoding ([`MapObject`]'s, as the static
    /// map's). The freed tiles and the grids follow from them and the static map.
    fn hash_into(&self, h: &mut StateHasher) {
        self.removed.hash_into(h);
        h.write_len(self.added.len());
        for object in self.added.iter() {
            object.hash_into(h);
        }
    }
}
