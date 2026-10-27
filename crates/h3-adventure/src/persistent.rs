//! Persistent grids (ADR-0016 points 1 and 2, E08 T5b): a per-tile layer that changes (a
//! player's explored tiles, T8) as a wide, shallow structure that a fork shares. The grid holds
//! an `Arc` to a table of leaves, a leaf being a block of `SIDE × SIDE` tiles as a bitset; a fork
//! is one reference increment, and the first write after it copies the table and one leaf
//! (`Arc::make_mut`). A leaf with no tile set is no leaf at all (`None`); every leaf with all its
//! tiles set shares the grid's own full leaf (never a static), so a leaf is stored in one form
//! only.
//!
//! Nothing sees the sharing (ADR-0016 point 2): equality, `Clone`, `Debug` and the canonical hash
//! see content only. This module is the only place pointer identity may appear (`arch.toml`): as
//! a shortcut of equality that cannot change its result (equal pointers, equal content).
//!
//! The leaf size was chosen by measurement against a flat copy (T5b, `examples/grid_bench.rs`,
//! `docs/02-architecture.md` §8; the user, 2026-10-11): 32 × 32 tiles, [`BitGrid`]'s default. A
//! fork costs the same on every map; the first write after it copies the table (50 leaves on a
//! 144 × 144 map with an underground) and one leaf of 128 bytes.

use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use h3_kernel::{CanonicalHash, StateHasher};

use crate::setup::Pos;

/// A grid of one bit per tile over a map's columns, rows and levels, persistent: a fork
/// ([`Clone`]) shares everything, a write copies what it changes. A leaf is `WORDS` words of 64
/// tiles: 1 for 8 × 8 tiles, 4 for 16 × 16, 16 for 32 × 32 (the default), 64 for 64 × 64.
#[derive(Clone)]
pub struct BitGrid<const WORDS: usize = 16> {
    width: u16,
    height: u16,
    levels: u8,
    /// Leaves per row of leaves.
    across: u16,
    /// Rows of leaves per level.
    down: u16,
    /// The leaves, level by level, row by row: `None` when no tile is set.
    table: Arc<Vec<Option<Arc<[u64; WORDS]>>>>,
    /// The leaf with every tile set, shared by every such leaf of the grid and its forks.
    full: Arc<[u64; WORDS]>,
}

impl<const WORDS: usize> BitGrid<WORDS> {
    /// Columns and rows of a leaf: a square of `64 × WORDS` tiles.
    pub const SIDE: u16 = {
        let side = (64 * WORDS).isqrt();
        assert!(
            side * side == 64 * WORDS && side <= 64,
            "a leaf is 1, 4, 16 or 64 words"
        );
        side as u16
    };

    /// A grid with no tile set.
    pub fn new(width: u16, height: u16, levels: u8) -> Self {
        let across = width.div_ceil(Self::SIDE);
        let down = height.div_ceil(Self::SIDE);
        let leaves = usize::from(across) * usize::from(down) * usize::from(levels);
        Self {
            width,
            height,
            levels,
            across,
            down,
            table: Arc::new(vec![None; leaves]),
            full: Arc::new([u64::MAX; WORDS]),
        }
    }

    /// The leaf holding `pos`, the word in it and the bit in the word, if `pos` is on the grid.
    fn locate(&self, pos: Pos) -> Option<(usize, usize, u32)> {
        if pos.x >= self.width || pos.y >= self.height || pos.level >= self.levels {
            return None;
        }
        let side = Self::SIDE;
        let leaf = (usize::from(pos.level) * usize::from(self.down) + usize::from(pos.y / side))
            * usize::from(self.across)
            + usize::from(pos.x / side);
        let bit = usize::from(pos.y % side) * usize::from(side) + usize::from(pos.x % side);
        Some((leaf, bit / 64, (bit % 64) as u32))
    }

    /// Whether the tile at `pos` is set (a tile off the grid is not).
    pub fn get(&self, pos: Pos) -> bool {
        self.locate(pos).is_some_and(|(leaf, word, bit)| {
            self.table[leaf]
                .as_ref()
                .is_some_and(|words| words[word] >> bit & 1 == 1)
        })
    }

    /// Sets or clears the tile at `pos`; a tile off the grid, or one that already is so, changes
    /// nothing and copies nothing.
    pub fn set(&mut self, pos: Pos, on: bool) {
        if let Some((leaf, word, bit)) = self.locate(pos) {
            self.write(leaf, word, 1 << bit, on);
        }
    }

    /// Sets or clears `len` tiles of a row from `start` rightwards (a reveal sets a disk row by
    /// row): within a leaf a row is one masked word, so each leaf the run crosses is written at
    /// most once. Tiles off the grid are left out; tiles that already are so copy nothing.
    pub fn set_run(&mut self, start: Pos, len: u16, on: bool) {
        if start.y >= self.height || start.level >= self.levels {
            return;
        }
        let side = Self::SIDE;
        let end = start.x.saturating_add(len).min(self.width);
        let row = usize::from(start.level) * usize::from(self.down) + usize::from(start.y / side);
        // A leaf's row lies in one word: `side` divides 64, or is 64.
        let first = usize::from(start.y % side) * usize::from(side);
        let (word, shift) = (first / 64, (first % 64) as u32);
        let mut x = start.x;
        while x < end {
            let column = x / side;
            let to = end.min(column.saturating_add(1).saturating_mul(side));
            let bits = u32::from(to - x);
            let mask = (u64::MAX >> (64 - bits)) << (shift + u32::from(x % side));
            let leaf = row * usize::from(self.across) + usize::from(column);
            self.write(leaf, word, mask, on);
            x = to;
        }
    }

    /// Sets (`on`) or clears the bits of `mask` in word `word` of leaf `leaf`, copying the table
    /// and the leaf only when a bit changes, and keeps the leaf in its one form: an empty leaf is
    /// none, a full one the shared full leaf.
    fn write(&mut self, leaf: usize, word: usize, mask: u64, on: bool) {
        let current = self.table[leaf].as_ref().map_or(0, |words| words[word]);
        let wanted = if on { current | mask } else { current & !mask };
        if wanted == current {
            return;
        }
        let full = &self.full;
        let slot = &mut Arc::make_mut(&mut self.table)[leaf];
        let words = Arc::make_mut(slot.get_or_insert_with(|| Arc::new([0; WORDS])));
        words[word] = wanted;
        if !on && words.iter().all(|&w| w == 0) {
            *slot = None;
        } else if on && words.iter().all(|&w| w == u64::MAX) {
            *slot = Some(Arc::clone(full));
        }
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

    /// How many tiles are set.
    pub fn count(&self) -> u32 {
        self.table
            .iter()
            .flatten()
            .map(|words| words.iter().map(|w| w.count_ones()).sum::<u32>())
            .sum()
    }

    /// Writes the row `y` of level `level` into `out`, the tile of column `x` as bit `x % 64` of
    /// word `x / 64` (the observation's planes, E08 T8b1): one masked word per leaf the row
    /// crosses, whatever the leaf size. The row must be on the grid and `out` hold at least
    /// `width.div_ceil(64)` words; its words past the row's are left as they are. Bits past the
    /// width come out clear: nothing sets a tile off the grid, so no leaf at the edge is full.
    pub(crate) fn read_row(&self, level: u8, y: u16, out: &mut [u64]) {
        let side = Self::SIDE;
        let row = (usize::from(level) * usize::from(self.down) + usize::from(y / side))
            * usize::from(self.across);
        let first = usize::from(y % side) * usize::from(side);
        let (word, shift) = (first / 64, first % 64);
        let mask = u64::MAX >> (64 - u32::from(side));
        out[..usize::from(self.width.div_ceil(64))].fill(0);
        for (column, leaf) in self.table[row..row + usize::from(self.across)]
            .iter()
            .enumerate()
        {
            let bits = leaf
                .as_ref()
                .map_or(0, |words| (words[word] >> shift) & mask);
            // `side` divides 64 or is 64: a leaf's row never straddles two words of `out`.
            let at = column * usize::from(side);
            out[at / 64] |= bits << (at % 64);
        }
    }

    /// The words of the leaf at `index` (zeros for none).
    fn words(&self, index: usize) -> [u64; WORDS] {
        self.table[index].as_deref().copied().unwrap_or([0; WORDS])
    }
}

impl<const WORDS: usize> PartialEq for BitGrid<WORDS> {
    /// Content only: the dimensions and every tile.
    fn eq(&self, other: &Self) -> bool {
        (self.width, self.height, self.levels) == (other.width, other.height, other.levels)
            && (Arc::ptr_eq(&self.table, &other.table)
                || (0..self.table.len()).all(|index| self.words(index) == other.words(index)))
    }
}

impl<const WORDS: usize> Eq for BitGrid<WORDS> {}

impl<const WORDS: usize> fmt::Debug for BitGrid<WORDS> {
    /// Content only: the dimensions, the number of tiles set and the leaves with a tile set (by
    /// index, with their words).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let leaves: Vec<(usize, [u64; WORDS])> = (0..self.table.len())
            .map(|index| (index, self.words(index)))
            .filter(|(_, words)| words.iter().any(|&w| w != 0))
            .collect();
        f.debug_struct("BitGrid")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("levels", &self.levels)
            .field("set", &self.count())
            .field("leaves", &leaves)
            .finish()
    }
}

impl<const WORDS: usize> CanonicalHash for BitGrid<WORDS> {
    /// The dimensions, then every leaf's words in leaf order (zeros for a leaf with no tile set).
    /// The leaf size is part of the encoding: changing it changes the state hash's version.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u16(self.width);
        h.write_u16(self.height);
        h.write_u8(self.levels);
        for index in 0..self.table.len() {
            for word in self.words(index) {
                h.write_u64(word);
            }
        }
    }
}

/// Entries per leaf of a [`Table`].
const LEAF: usize = 16;
/// Leaves per mid table of a [`Table`].
const MID: usize = 64;

/// A leaf of a [`Table`]: its entries, each value shared.
type Leaf<T> = [Option<Arc<T>>; LEAF];
/// A mid table of a [`Table`]: its leaves.
type Mid<T> = [Option<Arc<Leaf<T>>>; MID];

/// A sparse table of values by index, persistent (E08 T61b; design §10 of
/// `docs/epics/E08-design.md`, decision 133): a top table of mid tables of 64 leaves of 16
/// entries, each value shared, so a fork ([`Clone`]) is a reference count (none for an empty
/// table) and a write copies at most the top table, a mid table and a leaf, pointers only,
/// whatever the number of entries (the armies battles left on objects: T55b's measurement, about
/// 4 200 Ir a write with 41 472 entries against 16 million for one map). A part without an entry
/// is no part at all, so an empty table holds nothing. Nothing sees the sharing: equality,
/// `Debug` and the iteration see the entries in index order.
pub(crate) struct Table<T> {
    top: Option<Arc<Vec<Option<Arc<Mid<T>>>>>>,
    len: usize,
}

impl<T> Clone for Table<T> {
    fn clone(&self) -> Self {
        Self {
            top: self.top.clone(),
            len: self.len,
        }
    }
}

/// The mid table, leaf and entry of an index.
const fn split(index: usize) -> (usize, usize, usize) {
    (index / (MID * LEAF), index / LEAF % MID, index % LEAF)
}

impl<T> Table<T> {
    /// No entry.
    pub(crate) const fn new() -> Self {
        Self { top: None, len: 0 }
    }

    /// How many entries it holds.
    pub(crate) const fn len(&self) -> usize {
        self.len
    }

    /// The entry at `index`, if any.
    pub(crate) fn get(&self, index: usize) -> Option<&T> {
        let (t, m, l) = split(index);
        let mid = self.top.as_ref()?.get(t)?.as_ref()?;
        mid[m].as_ref()?[l].as_deref()
    }

    /// Sets the entry at `index` to `value`, or removes it for none; removing an entry that is
    /// not there copies nothing.
    pub(crate) fn set(&mut self, index: usize, value: Option<T>) {
        if value.is_none() && self.get(index).is_none() {
            return;
        }
        let (t, m, l) = split(index);
        let top = Arc::make_mut(self.top.get_or_insert_with(|| Arc::new(Vec::new())));
        if top.len() <= t {
            top.resize(t + 1, None);
        }
        let mid =
            Arc::make_mut(top[t].get_or_insert_with(|| Arc::new(core::array::from_fn(|_| None))));
        let leaf =
            Arc::make_mut(mid[m].get_or_insert_with(|| Arc::new(core::array::from_fn(|_| None))));
        let had = leaf[l].is_some();
        let has = value.is_some();
        leaf[l] = value.map(Arc::new);
        self.len = self.len + usize::from(has) - usize::from(had);
        if leaf.iter().all(Option::is_none) {
            mid[m] = None;
            if mid.iter().all(Option::is_none) {
                top[t] = None;
                while top.last().is_some_and(Option::is_none) {
                    top.pop();
                }
            }
        }
        if self.len == 0 {
            self.top = None;
        }
    }

    /// The entries in index order.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        self.top
            .iter()
            .flat_map(|top| top.iter().enumerate())
            .filter_map(|(t, mid)| Some((t, mid.as_deref()?)))
            .flat_map(|(t, mid)| {
                mid.iter()
                    .enumerate()
                    .filter_map(move |(m, leaf)| Some((t * MID + m, leaf.as_deref()?)))
            })
            .flat_map(|(at, leaf)| {
                leaf.iter()
                    .enumerate()
                    .filter_map(move |(l, value)| Some((at * LEAF + l, value.as_deref()?)))
            })
    }
}

impl<T: PartialEq> PartialEq for Table<T> {
    /// Content only: the entries in index order.
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.iter().eq(other.iter())
    }
}

impl<T: Eq> Eq for Table<T> {}

impl<T: fmt::Debug> fmt::Debug for Table<T> {
    /// Content only: the entries by index.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

#[cfg(test)]
impl<T> Table<T> {
    /// Whether two tables share their top table (tests of the sharing; pointer identity stays
    /// here); two empty tables do.
    pub(crate) fn shares_top(&self, other: &Self) -> bool {
        match (&self.top, &other.top) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (a, b) => a.is_none() && b.is_none(),
        }
    }

    /// Whether the entries at `index` of the two tables are one shared value.
    pub(crate) fn shares_value(&self, other: &Self, index: usize) -> bool {
        let value = |table: &Self| {
            let (t, m, l) = split(index);
            let mid = table.top.as_ref()?.get(t)?.as_ref()?;
            mid[m].as_ref()?[l].clone()
        };
        value(self)
            .zip(value(other))
            .is_some_and(|(a, b)| Arc::ptr_eq(&a, &b))
    }

    /// How many mid tables and leaves are stored.
    pub(crate) fn stored(&self) -> (usize, usize) {
        let mids: Vec<&Mid<T>> = self
            .top
            .iter()
            .flat_map(|top| top.iter().flatten())
            .map(|m| &**m)
            .collect();
        let leaves = mids.iter().map(|mid| mid.iter().flatten().count()).sum();
        (mids.len(), leaves)
    }
}

#[cfg(test)]
impl<const WORDS: usize> BitGrid<WORDS> {
    /// Whether two grids share their table (tests of the sharing; pointer identity stays here).
    pub(crate) fn shares_table(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.table, &other.table)
    }

    /// Whether the leaf holding `pos` is the shared full leaf.
    pub(crate) fn is_shared_full(&self, pos: Pos) -> bool {
        self.locate(pos)
            .and_then(|(leaf, _, _)| self.table[leaf].as_ref())
            .is_some_and(|words| Arc::ptr_eq(words, &self.full))
    }

    /// How many leaves are stored (not none).
    pub(crate) fn stored_leaves(&self) -> usize {
        self.table.iter().flatten().count()
    }
}

#[cfg(test)]
mod tests;
