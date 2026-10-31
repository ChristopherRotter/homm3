//! Per-object state and the live presence (ADR-0016 point 1, E08 T12b; E08 T83b2a, increment 9's
//! design note §6): which objects have left the map, which tiles the static map marks blocked or
//! visitable that no object still on the map covers so any more ([`Occupancy`]), and the objects
//! added in play (a creature month's monsters, T85a), after the static map's in map order
//! ([`Added`]). Persistent grids ([`BitGrid`]), so a fork shares them and a removal copies a leaf
//! or two; the static map keeps every footprint. The state holds an occupancy only once an
//! object has left the map, and the added objects only once one was added, so a map without
//! either pays one test where it reads them (`bench-check`). Objects leave the map by T13's
//! visits (a pickup, a border guard); until then nothing removes one.

use alloc::vec::Vec;

use h3_kernel::{CanonicalHash, StateHasher};

use super::{AdventureState, MapObject, StaticMap, StaticTile};
use crate::persistent::BitGrid;
use crate::ruleset::{AdventureRuleset, MonsterParams, ObjectKind, Passage};
use crate::setup::Pos;

/// The objects that left the map and the tiles they freed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Occupancy {
    /// The objects that left the map, by index: object `i` is column `i % ROW` of row `i / ROW`;
    /// the added objects' indices follow the static map's (E08 T83b2a: as many rows as the
    /// objects need, added ones included).
    removed: BitGrid,
    /// The tiles the static map marks blocked that no object still on the map blocks.
    unblocked: BitGrid,
    /// The tiles the static map marks visitable that no object still on the map is visited from.
    unvisitable: BitGrid,
}

impl Occupancy {
    /// Objects per row of [`Self::removed`]: a leaf's side, so a row of objects is one word.
    pub(crate) const ROW: usize = BitGrid::<16>::SIDE as usize;

    /// Every one of `objects` objects on the map (the static map's and those added in play). The
    /// static map holds at most `ROW × u16::MAX` objects ([`crate::SetupError::Objects`]).
    pub(crate) fn new(map: &StaticMap, objects: usize) -> Self {
        let grid = || BitGrid::new(map.width(), map.height(), map.levels());
        Self {
            removed: BitGrid::new(BitGrid::<16>::SIDE, Self::rows(objects), 1),
            unblocked: grid(),
            unvisitable: grid(),
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

    /// Room in [`Self::removed`] for `objects` objects (E08 T83b2a: an added object's index), a
    /// row more when they need it, every removal kept: the rows follow from the objects, so the
    /// order of removals and additions does not change the grid.
    #[cfg(test)]
    pub(crate) fn make_room(&mut self, objects: usize) {
        let rows = Self::rows(objects);
        if rows > self.removed.height() {
            let mut removed = BitGrid::new(BitGrid::<16>::SIDE, rows, 1);
            for left in (0..objects).filter(|&i| self.is_removed(i)) {
                removed.set(Self::slot(left), true);
            }
            self.removed = removed;
        }
    }

    /// `tile`, the static map's tile at `pos`, blocked and visitable only where an object still
    /// on the map covers it so. The grids are read only where the static tile is marked.
    pub(crate) fn live(&self, pos: Pos, mut tile: StaticTile) -> StaticTile {
        tile.blocked = tile.blocked && !self.unblocked.get(pos);
        tile.visitable = tile.visitable && !self.unvisitable.get(pos);
        tile
    }

    /// Takes the object at `index`, one of the map's or of `added` (the objects added in play)
    /// still on it, off the map (ADR-0016 point 1): each of its tiles that no other object still
    /// on the map blocks becomes unblocked, and each that no other object is visited from
    /// becomes unvisitable.
    pub(crate) fn remove(&mut self, map: &StaticMap, added: &[MapObject], index: usize) {
        self.removed.set(Self::slot(index), true);
        let statics = map.objects();
        let object = statics
            .get(index)
            .unwrap_or_else(|| &added[index - statics.len()]);
        let freed: Vec<(Pos, bool, bool)> = object
            .tiles(map)
            .map(|pos| {
                let (blocked, visitable) = object.covers(pos);
                let others = statics
                    .iter()
                    .chain(added)
                    .enumerate()
                    .filter(|&(other, _)| !self.is_removed(other))
                    .map(|(_, other)| other.covers(pos))
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

impl CanonicalHash for Occupancy {
    /// The removed objects ([`BitGrid`]'s encoding); the freed tiles follow from them and the
    /// static map.
    fn hash_into(&self, h: &mut StateHasher) {
        self.removed.hash_into(h);
    }
}

/// The objects added in play (E08 T83b2a; increment 9's design note §6): their list, after the
/// static map's in map order (index `map.objects().len() + n`), and the grids where the reads
/// look at them at all. One block the forks share, as the boats' records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Added {
    /// The objects, in the order they came, on the map or not (a removal is the occupancy's).
    pub(crate) objects: Vec<MapObject>,
    /// The tiles an added object blocks or is visited from, left or not: where
    /// [`AdventureState::live`] and the visit's lookup read the added objects.
    pub(crate) tiles: BitGrid,
    /// The tiles within one step of an added monster's visitable tile on its level, left or not
    /// (its 3 × 3, as [`StaticMap::near_monster`] marks the placed ones): where a guard's search
    /// reads them.
    pub(crate) near: BitGrid,
}

impl Added {
    /// No object added yet, on `map`'s tiles.
    #[cfg(test)]
    pub(crate) fn new(map: &StaticMap) -> Self {
        Self {
            objects: Vec::new(),
            tiles: BitGrid::new(map.width(), map.height(), map.levels()),
            near: BitGrid::new(map.width(), map.height(), map.levels()),
        }
    }

    /// Adds `object`, its tiles marked in [`Self::tiles`] and a monster's 3 × 3 around each
    /// visitable tile in [`Self::near`]. The caller has checked that it is a monster on land
    /// tiles no other object still on the map covers.
    #[cfg(test)]
    pub(crate) fn push(&mut self, map: &StaticMap, object: MapObject) {
        let anchor = object.anchor;
        let on_map = |mask: crate::setup::TileMask| {
            mask.offsets()
                .filter_map(move |(column, row)| map.offset_pos(anchor, column, row))
        };
        let covered =
            crate::setup::TileMask::from_bits(object.blocked.bits() | object.visitable.bits());
        for pos in on_map(covered) {
            self.tiles.set(pos, true);
        }
        for pos in on_map(object.visitable) {
            if object.monster.is_some() {
                for (dx, dy) in crate::movement::DIRECTIONS.iter().copied().chain([(0, 0)]) {
                    let x = pos.x.checked_add_signed(i16::from(dx));
                    let y = pos.y.checked_add_signed(i16::from(dy));
                    if let (Some(x), Some(y)) = (x, y) {
                        self.near.set(Pos::new(x, y, pos.level), true);
                    }
                }
            }
        }
        self.objects.push(object);
    }
}

impl CanonicalHash for Added {
    /// Their number, then each one's encoding ([`MapObject`]'s, as the static map's); the grids
    /// follow from them and the static map.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_len(self.objects.len());
        for object in &self.objects {
            object.hash_into(h);
        }
    }
}

impl AdventureState {
    /// The objects added in play still on the map, with their indices.
    pub(crate) fn added_on_map(&self) -> impl Iterator<Item = (usize, &MapObject)> {
        (self.map.objects().len()..)
            .zip(self.added_objects())
            .filter(|&(index, _)| !self.is_removed(index))
    }

    /// The last object added in play still on the map that is visited from `pos`, by index
    /// (ADV-OBJ-003: they come after the static map's in map order); none where none covers the
    /// tile, which costs one test without any added object (`bench-check`).
    #[inline]
    pub(crate) fn added_visited_from(&self, pos: Pos) -> Option<usize> {
        self.added
            .as_deref()
            .filter(|added| added.tiles.get(pos))
            .and_then(|_| self.added_visited(pos))
    }

    /// [`Self::added_visited_from`] where an added object covers `pos`, out of the hot paths.
    #[cold]
    #[inline(never)]
    fn added_visited(&self, pos: Pos) -> Option<usize> {
        self.added_on_map()
            .filter(|(_, object)| object.covers(pos).1)
            .last()
            .map(|(index, _)| index)
    }

    /// `tile` blocked and visitable where an object added in play still on the map covers `pos`
    /// so, besides what it is already ([`Self::live`]'s second half, out of the hot paths).
    #[cold]
    #[inline(never)]
    pub(crate) fn added_live(&self, pos: Pos, mut tile: StaticTile) -> StaticTile {
        let (blocked, visitable) = self
            .added_on_map()
            .map(|(_, object)| object.covers(pos))
            .fold((false, false), |(b, v), (ob, ov)| (b || ob, v || ov));
        tile.blocked |= blocked;
        tile.visitable |= visitable;
        tile
    }

    /// The monsters' fractions' and the objects added in play's invariants (E08 T83b2a;
    /// increment 9's design note §6), which every command keeps: a fraction only on a wandering
    /// monster still on the map, above 0 and below the monsters' growth unit; each added object
    /// still on the map a typed monster of a monster kind covering at least one tile, each a
    /// land tile marked in the added objects' grid (a visitable one in the grid near them too)
    /// that no other object still on the map blocks or is visited from.
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
                .added
                .as_deref()
                .is_none_or(|added| self.check_added_objects(part, added))
    }

    /// [`Self::check_added`]'s added objects, those still on the map.
    fn check_added_objects(&self, part: &AdventureRuleset, added: &Added) -> bool {
        let map = &*self.map;
        let land = |pos: Pos| matches!(part.passage(map.at(pos).terrain), Some(Passage::Land(_)));
        self.added_on_map().all(|(index, object)| {
            let monster = object.monster.is_some()
                && object
                    .object_type
                    .and_then(|id| part.object_type(id))
                    .is_some_and(|ty| matches!(ty.kind, ObjectKind::Monster { .. }));
            let free = |pos: Pos| {
                map.objects()
                    .iter()
                    .chain(&added.objects)
                    .enumerate()
                    .filter(|&(other, _)| other != index && !self.is_removed(other))
                    .all(|(_, other)| other.covers(pos) == (false, false))
            };
            let marked =
                |pos: Pos| added.tiles.get(pos) && (!object.covers(pos).1 || added.near.get(pos));
            monster
                && object.tiles(map).next().is_some()
                && object
                    .tiles(map)
                    .all(|pos| land(pos) && marked(pos) && free(pos))
        })
    }
}
