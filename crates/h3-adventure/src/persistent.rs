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
            let to = end.min((column + 1) * side);
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
