//! Per-object state and the occupancy layer (ADR-0016 point 1, E08 T12b): which objects have left
//! the map, and which tiles the static map marks blocked or visitable that no object still on the
//! map covers so any more. Persistent grids ([`BitGrid`]), so a fork shares them and a removal
//! copies a leaf or two; the static map keeps every footprint. The state holds one only once an
//! object has left the map. Objects leave the map by T13's
//! visits (a pickup, a border guard); until then nothing removes one.

use h3_kernel::{CanonicalHash, StateHasher};

use super::{StaticMap, StaticTile};
use crate::persistent::BitGrid;
use crate::setup::Pos;

/// The objects that left the map and the tiles they freed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Occupancy {
    /// The objects that left the map, by index: object `i` is column `i % ROW` of row `i / ROW`.
    removed: BitGrid,
    /// The tiles the static map marks blocked that no object still on the map blocks.
    unblocked: BitGrid,
    /// The tiles the static map marks visitable that no object still on the map is visited from.
    unvisitable: BitGrid,
}

impl Occupancy {
    /// Objects per row of [`Self::removed`]: a leaf's side, so a row of objects is one word.
    pub(crate) const ROW: usize = BitGrid::<16>::SIDE as usize;

    /// Every object on the map. The static map holds at most `ROW × u16::MAX` objects
    /// ([`crate::SetupError::Objects`]).
    pub(crate) fn new(map: &StaticMap) -> Self {
        let rows = u16::try_from(map.objects().len().div_ceil(Self::ROW)).unwrap_or(u16::MAX);
        Self {
            removed: BitGrid::new(BitGrid::<16>::SIDE, rows, 1),
            unblocked: BitGrid::new(map.width(), map.height(), map.levels()),
            unvisitable: BitGrid::new(map.width(), map.height(), map.levels()),
        }
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

    /// `tile`, the static map's tile at `pos`, blocked and visitable only where an object still
    /// on the map covers it so. The grids are read only where the static tile is marked.
    pub(crate) fn live(&self, pos: Pos, mut tile: StaticTile) -> StaticTile {
        tile.blocked = tile.blocked && !self.unblocked.get(pos);
        tile.visitable = tile.visitable && !self.unvisitable.get(pos);
        tile
    }

    /// Takes the object at `index`, one of the map's still on it, off the map (ADR-0016 point 1):
    /// each of its tiles that no other object still on the map blocks becomes unblocked, and each
    /// that no other object is visited from becomes unvisitable.
    pub(crate) fn remove(&mut self, map: &StaticMap, index: usize) {
        self.removed.set(Self::slot(index), true);
        let object = &map.objects()[index];
        for pos in object.tiles(map) {
            let (blocked, visitable) = object.covers(pos);
            let others = map
                .objects()
                .iter()
                .enumerate()
                .filter(|&(other, _)| !self.is_removed(other))
                .map(|(_, other)| other.covers(pos))
                .fold((false, false), |(b, v), (ob, ov)| (b || ob, v || ov));
            if blocked && !others.0 {
                self.unblocked.set(pos, true);
            }
            if visitable && !others.1 {
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
