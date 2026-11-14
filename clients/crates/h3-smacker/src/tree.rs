//! The header's four Huffman trees (FFmpeg 6.1.1's `libavcodec/smacker.c`:
//! `smacker_decode_tree`, `smacker_decode_bigtree`, `smacker_decode_header_tree`,
//! `decode_header_trees`, `last_reset` and `smk_get_code`).
//!
//! Each header tree codes 16-bit values: its leaves hold a low and a high byte, each coded by a
//! byte tree of its own read first. A tree is kept as FFmpeg keeps it, a list of entries in the
//! order read (depth first, the 0 branch first): a node holds [`NODE`] and the count of entries
//! in its 0 branch, so its 0 branch starts at the next entry and its 1 branch that many entries
//! further; a leaf holds its value. Three entries are a cache of the last three values decoded
//! (the "escapes": the leaves whose value equals one of three escape values given in the header,
//! else entries added after the tree), emptied at each frame's start; a code reaching one gives
//! the value cached there. FFmpeg decodes the byte trees through lookup tables built from the
//! leaves' lengths (`ff_vlc_init_from_lengths`); as a tree read depth first is complete, its codes
//! are its paths, so walking it bit by bit reads the same bits and gives the same values.

use alloc::vec;
use alloc::vec::Vec;

use crate::Error;
use crate::bits::Bits;

/// A node's mark (`SMK_NODE`).
pub(crate) const NODE: u32 = 0x8000_0000;
/// The deepest leaf of a byte tree (`SMKTREE_DECODE_MAX_RECURSION`, `3 * SMKTREE_BITS`).
const BYTE_DEPTH: u32 = 27;
/// The deepest entry of a header tree (`SMKTREE_DECODE_BIG_MAX_RECURSION`).
const BIG_DEPTH: u32 = 500;
/// The most leaves a byte tree holds.
const BYTE_LEAVES: usize = 256;

/// The value at `at`, 0 outside the list (never reached: every index walked or cached lies
/// inside its tree by construction).
fn get(values: &[u32], at: usize) -> u32 {
    values.get(at).copied().unwrap_or(0)
}

/// Sets the value at `at`; nothing outside the list (as [`get`]).
fn set(values: &mut [u32], at: usize, value: u32) {
    if let Some(v) = values.get_mut(at) {
        *v = value;
    }
}

/// The entry a walk from the root reaches, reading a bit at each node; `checked`: failing when
/// no bit is left (`smk_get_code`), else reading 0s past the end (the byte trees' `get_vlc2`).
fn walk(values: &[u32], bits: &mut Bits<'_>, checked: bool) -> Result<usize, Error> {
    let mut at = 0usize;
    loop {
        let entry = get(values, at);
        if entry & NODE == 0 {
            return Ok(at);
        }
        if checked && bits.left() < 1 {
            return Err(Error::Invalid("code past the end"));
        }
        if bits.bit() == 1 {
            at = at.saturating_add((entry & !NODE) as usize);
        }
        at = at.saturating_add(1);
    }
}

/// A byte tree: its one value (a tree of one leaf, or none: 0), or its entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ByteTree {
    Value(u32),
    Tree(Vec<u32>),
}

impl ByteTree {
    /// The next value: the one value without reading a bit, else a walk to a leaf, 0s read
    /// past the end (`get_vlc2` over FFmpeg's padding).
    pub(crate) fn read(&self, bits: &mut Bits<'_>) -> u32 {
        match self {
            Self::Value(value) => *value,
            Self::Tree(values) => walk(values, bits, false).map_or(0, |at| get(values, at)),
        }
    }

    /// Reads one (`smacker_decode_tree` from the root): a tree of one leaf gives its value
    /// alone, as FFmpeg keeps `values[i]` and builds no table for it (the sound's trees,
    /// `smka_decode_frame`).
    pub(crate) fn parse(bits: &mut Bits<'_>) -> Result<Self, Error> {
        let (mut values, mut leaves) = (Vec::new(), 0);
        byte_tree(bits, &mut values, &mut leaves, 0)?;
        Ok(if leaves > 1 {
            Self::Tree(values)
        } else {
            Self::Value(get(&values, 0))
        })
    }

    /// Whether it reads bits (FFmpeg's `vlc[i].table` set).
    pub(crate) fn coded(&self) -> bool {
        matches!(self, Self::Tree(_))
    }
}

/// Reads a byte tree's entries (`smacker_decode_tree`); `leaves` counts its leaves.
fn byte_tree(
    bits: &mut Bits<'_>,
    values: &mut Vec<u32>,
    leaves: &mut usize,
    length: u32,
) -> Result<(), Error> {
    if length > BYTE_DEPTH {
        return Err(Error::Invalid("byte tree too deep"));
    }
    if bits.bit() == 0 {
        if *leaves >= BYTE_LEAVES {
            return Err(Error::Invalid("byte tree too large"));
        }
        if bits.left() < 8 {
            return Err(Error::Invalid("byte tree past the end"));
        }
        values.push(bits.bits(8));
        *leaves += 1;
        return Ok(());
    }
    let node = values.len();
    values.push(NODE);
    byte_tree(bits, values, leaves, length + 1)?;
    let zero = u32::try_from(values.len() - node - 1).unwrap_or(0);
    set(values, node, NODE | zero);
    byte_tree(bits, values, leaves, length + 1)
}

/// A header tree being read (`DBCtx`).
struct Big<'b> {
    bytes: &'b [ByteTree; 2],
    escapes: [u32; 3],
    last: [Option<usize>; 3],
    values: Vec<u32>,
    /// The most entries the header's size allows.
    length: usize,
}

impl Big<'_> {
    /// Reads the entries below a node at `depth`; their count (`smacker_decode_bigtree`).
    fn read(&mut self, bits: &mut Bits<'_>, depth: u32) -> Result<u32, Error> {
        if depth > BIG_DEPTH {
            return Err(Error::Invalid("tree too deep"));
        }
        if self.values.len() >= self.length {
            return Err(Error::Invalid("tree too large"));
        }
        if bits.left() <= 0 {
            return Err(Error::Invalid("tree past the end"));
        }
        if bits.bit() == 0 {
            let low = self.bytes[0].read(bits);
            let high = self.bytes[1].read(bits);
            let mut value = low | (high << 8);
            if let Some(k) = self.escapes.iter().position(|&e| e == value) {
                self.last[k] = Some(self.values.len());
                value = 0;
            }
            self.values.push(value);
            return Ok(1);
        }
        let node = self.values.len();
        self.values.push(NODE);
        let zero = self.read(bits, depth + 1)?;
        set(&mut self.values, node, NODE | zero);
        let one = self.read(bits, depth + 1)?;
        Ok(zero.saturating_add(1).saturating_add(one))
    }
}

/// One of the header's trees: its entries and where its three cached values are.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Tree {
    values: Vec<u32>,
    last: [usize; 3],
}

impl Tree {
    /// A tree the header leaves out: every code gives 0 and reads no bit.
    pub(crate) fn absent() -> Self {
        Self {
            values: vec![0, 0],
            last: [1; 3],
        }
    }

    /// Reads a present tree (`smacker_decode_header_tree`) of the header's `size`.
    fn read(bits: &mut Bits<'_>, size: u32) -> Result<Self, Error> {
        if size >= u32::MAX >> 4 {
            return Err(Error::Invalid("tree size"));
        }
        let mut bytes = [ByteTree::Value(0), ByteTree::Value(0)];
        for tree in &mut bytes {
            if bits.bit() == 0 {
                continue;
            }
            *tree = ByteTree::parse(bits)?;
            bits.bit();
        }
        let escapes = [bits.bits(16), bits.bits(16), bits.bits(16)];
        let mut big = Big {
            bytes: &bytes,
            escapes,
            last: [None; 3],
            values: Vec::new(),
            length: ((size + 3) >> 2) as usize,
        };
        big.read(bits, 0)?;
        bits.bit();
        let mut last = [0; 3];
        for (slot, found) in last.iter_mut().zip(big.last) {
            *slot = found.unwrap_or_else(|| {
                big.values.push(0);
                big.values.len() - 1
            });
        }
        Ok(Self {
            values: big.values,
            last,
        })
    }

    /// Empties the cache (`last_reset`), at each frame's start.
    pub(crate) fn reset(&mut self) {
        for at in self.last {
            set(&mut self.values, at, 0);
        }
    }

    /// The next value (`smk_get_code`); the cache keeps the last three values that differ from
    /// the one before.
    pub(crate) fn code(&mut self, bits: &mut Bits<'_>) -> Result<u32, Error> {
        let at = walk(&self.values, bits, true)?;
        let value = get(&self.values, at);
        let [a, b, c] = self.last;
        let (first, second) = (get(&self.values, a), get(&self.values, b));
        if value != first {
            set(&mut self.values, c, second);
            set(&mut self.values, b, first);
            set(&mut self.values, a, value);
        }
        Ok(value)
    }

    /// A walk from the root to a leaf, each node's branch chosen by `choose` (for the writer:
    /// the bits a code of the leaf reached is written with).
    pub(crate) fn path(&self, mut choose: impl FnMut() -> bool) -> Vec<bool> {
        let (mut at, mut path) = (0usize, Vec::new());
        loop {
            let entry = get(&self.values, at);
            if entry & NODE == 0 {
                return path;
            }
            let one = choose();
            path.push(one);
            if one {
                at = at.saturating_add((entry & !NODE) as usize);
            }
            at = at.saturating_add(1);
        }
    }

    /// The tree's entries and its cache's places (for the tests).
    #[cfg(test)]
    pub(crate) fn parts(&self) -> (&[u32], [usize; 3]) {
        (&self.values, self.last)
    }
}

/// The header's trees, in the file's order (`decode_header_trees`); indexed by [`MAP`],
/// [`COLORS`], [`FULL`] and [`TYPES`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Trees {
    /// The mono blocks' maps.
    pub(crate) map: Tree,
    /// The mono blocks' two colours.
    pub(crate) colors: Tree,
    /// The full blocks' pixel pairs.
    pub(crate) full: Tree,
    /// The blocks' types, runs and fill colours.
    pub(crate) types: Tree,
}

impl Trees {
    /// Reads the trees of the header's `sizes` (map, colours, full, types) from `bytes`.
    pub(crate) fn read(sizes: [u32; 4], bytes: &[u8]) -> Result<Self, Error> {
        let mut bits = Bits::new(bytes);
        let mut absent = 0;
        let mut trees = sizes.map(|_| Tree::absent());
        for (tree, size) in trees.iter_mut().zip(sizes) {
            if bits.bit() == 0 {
                absent += 1;
            } else {
                *tree = Tree::read(&mut bits, size)?;
            }
        }
        if absent == 4 || bits.left() < 0 {
            return Err(Error::Invalid("header trees"));
        }
        let [map, colors, full, types] = trees;
        Ok(Self {
            map,
            colors,
            full,
            types,
        })
    }

    /// Empties every tree's cache.
    pub(crate) fn reset(&mut self) {
        for tree in [
            &mut self.map,
            &mut self.colors,
            &mut self.full,
            &mut self.types,
        ] {
            tree.reset();
        }
    }

    /// Every tree absent (the writer's stand-in where its own trees fail to read: never).
    pub(crate) fn absent() -> Self {
        Self {
            map: Tree::absent(),
            colors: Tree::absent(),
            full: Tree::absent(),
            types: Tree::absent(),
        }
    }

    /// The tree `index` (one of [`MAP`] to [`TYPES`]; the types' for any other).
    pub(crate) fn get_mut(&mut self, index: usize) -> &mut Tree {
        match index {
            MAP => &mut self.map,
            COLORS => &mut self.colors,
            FULL => &mut self.full,
            _ => &mut self.types,
        }
    }
}

/// The trees' indices in the header's order.
pub(crate) const MAP: usize = 0;
/// See [`MAP`].
pub(crate) const COLORS: usize = 1;
/// See [`MAP`].
pub(crate) const FULL: usize = 2;
/// See [`MAP`].
pub(crate) const TYPES: usize = 3;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::{BitWriter, Shape, TreeSpec};

    fn bytes_of(spec: &TreeSpec) -> Vec<u8> {
        let mut w = BitWriter::new();
        spec.write(&mut w);
        w.bytes()
    }

    fn tree(spec: &TreeSpec) -> Tree {
        Tree::read(&mut Bits::new(&bytes_of(spec)), spec.size()).unwrap()
    }

    fn read(spec: &TreeSpec, size: u32) -> Result<Tree, Error> {
        Tree::read(&mut Bits::new(&bytes_of(spec)), size)
    }

    fn bits(path: &[bool]) -> Vec<u8> {
        let mut w = BitWriter::new();
        for &bit in path {
            w.flag(bit);
        }
        w.bytes()
    }

    /// A chain of `depth` nodes, each with a leaf on its 1 branch.
    fn chain(depth: u32) -> Shape {
        (0..depth).fold(Shape::Leaf(1), |deeper, _| {
            Shape::node(deeper, Shape::Leaf(2))
        })
    }

    #[test]
    fn a_tree_is_kept_as_ffmpeg_keeps_it() {
        let t = tree(&TreeSpec::of(&[0x0102, 0x0304, 0x0506]));
        // The 0 branch's count in each node; the cache's three entries after the tree.
        let (values, last) = t.parts();
        assert_eq!(
            values,
            [NODE | 1, 0x0102, NODE | 1, 0x0304, 0x0506, 0, 0, 0]
        );
        assert_eq!(last, [5, 6, 7]);
        let mut t = t;
        for (path, value) in [
            (&[false][..], 0x0102),
            (&[true, false], 0x0304),
            (&[true, true], 0x0506),
        ] {
            assert_eq!(t.code(&mut Bits::new(&bits(path))), Ok(value));
        }
        // Each code needs its bits: none left fails.
        assert_eq!(
            t.code(&mut Bits::new(&[])),
            Err(Error::Invalid("code past the end"))
        );
        assert_eq!(t.path(|| true), [true, true]);
        assert_eq!(t.path(|| false), [false]);
    }

    #[test]
    fn the_escapes_leaves_give_the_last_three_values() {
        let spec = TreeSpec::of(&[5, 7, 0xFFFD, 0xFFFE]);
        let mut t = tree(&spec);
        let (values, last) = t.parts();
        // Node, 5, 7 under it; node, the two escape leaves (now 0) under it; one entry added.
        assert_eq!(values, [NODE | 3, NODE | 1, 5, 7, NODE | 1, 0, 0, 0]);
        assert_eq!(last, [5, 6, 7]);
        t.reset();
        let mut code = |path: &[bool]| t.code(&mut Bits::new(&bits(path))).unwrap();
        let (five, seven, first, second) =
            ([false, false], [false, true], [true, false], [true, true]);
        assert_eq!(code(&first), 0);
        assert_eq!(code(&five), 5);
        assert_eq!(code(&first), 5);
        assert_eq!(code(&seven), 7);
        assert_eq!(code(&second), 5);
        // 5 differed from the first (7): the cache moved (5, 7, 5).
        assert_eq!(code(&second), 7);
        assert_eq!(code(&first), 7);
        t.reset();
        assert_eq!(t.parts().0, [NODE | 3, NODE | 1, 5, 7, NODE | 1, 0, 0, 0]);
    }

    #[test]
    fn byte_trees_may_be_left_out_or_one_leaf() {
        let spec = TreeSpec {
            low: Some(Shape::Leaf(0x42)),
            high: None,
            escapes: [1, 2, 3],
            tree: Shape::node(Shape::Leaf(0x42), Shape::Leaf(0x42)),
        };
        let mut t = tree(&spec);
        assert_eq!(t.parts().0, [NODE | 1, 0x42, 0x42, 0, 0, 0]);
        t.reset();
        assert_eq!(t.code(&mut Bits::new(&bits(&[true]))), Ok(0x42));
    }

    #[test]
    fn trees_past_ffmpegs_limits_fail() {
        let of = TreeSpec::of(&[1, 2, 3]);
        assert_eq!(read(&of, u32::MAX >> 4), Err(Error::Invalid("tree size")));
        assert!(read(&of, (u32::MAX >> 4) - 1).is_ok());
        // Room for two entries of five.
        assert_eq!(read(&of, 8), Err(Error::Invalid("tree too large")));
        let byte_tree = |shape: Shape| TreeSpec {
            low: Some(shape),
            ..TreeSpec::of(&[1])
        };
        let deep = byte_tree(chain(BYTE_DEPTH + 1));
        assert_eq!(read(&deep, 100), Err(Error::Invalid("byte tree too deep")));
        assert!(read(&byte_tree(chain(BYTE_DEPTH)), 100).is_ok());
        let wide: Vec<u32> = (0..=256).collect();
        let large = byte_tree(Shape::balanced(&wide));
        assert_eq!(
            read(&large, 100),
            Err(Error::Invalid("byte tree too large"))
        );
        assert!(read(&byte_tree(Shape::balanced(&wide[..256])), 100).is_ok());
        let deep = TreeSpec {
            tree: chain(BIG_DEPTH + 1),
            ..TreeSpec::of(&[1])
        };
        assert_eq!(read(&deep, 10_000), Err(Error::Invalid("tree too deep")));
        let deep = TreeSpec {
            tree: chain(BIG_DEPTH),
            ..TreeSpec::of(&[1])
        };
        assert!(read(&deep, 10_000).is_ok());
    }

    #[test]
    fn trees_past_their_bytes_fail() {
        // A byte tree's leaf without its 8 bits.
        let mut bits = Bits::new(&[0b0000_0001]);
        assert_eq!(
            Tree::read(&mut bits, 100),
            Err(Error::Invalid("byte tree past the end"))
        );
        // The values' tree with no bit left.
        let bytes = bytes_of(&TreeSpec::of(&[1, 2]));
        assert_eq!(
            Tree::read(&mut Bits::new(&bytes[..7]), 100),
            Err(Error::Invalid("tree past the end"))
        );
    }

    #[test]
    fn header_trees_read_in_order() {
        let specs = [
            None,
            Some(TreeSpec::of(&[1, 2])),
            None,
            Some(TreeSpec::of(&[0x0300])),
        ];
        let header = crate::build::Header::new(4, 4, specs.clone());
        let mut trees = Trees::read(header.tree_sizes(), &header.trees()).unwrap();
        assert_eq!(trees.map, Tree::absent());
        assert_eq!(trees.colors, tree(specs[1].as_ref().unwrap()));
        assert_eq!(trees.full, Tree::absent());
        assert_eq!(trees.types, tree(specs[3].as_ref().unwrap()));
        assert_eq!(*trees.get_mut(MAP), Tree::absent());
        assert_eq!(trees.get_mut(COLORS).parts().0.len(), 6);
        assert_eq!(*trees.get_mut(FULL), Tree::absent());
        assert_eq!(trees.get_mut(TYPES).parts().0, [0x0300, 0, 0, 0]);
        // An absent tree's code reads no bit and gives 0.
        let mut empty = Bits::new(&[]);
        assert_eq!(trees.map.code(&mut empty), Ok(0));
        assert_eq!(empty.left(), 0);
        trees.reset();
        let absent = Trees::absent();
        assert_eq!((absent.map, absent.types), (Tree::absent(), Tree::absent()));
        // All four left out, or bits read past the end.
        assert_eq!(
            Trees::read([12; 4], &[0]),
            Err(Error::Invalid("header trees"))
        );
        assert_eq!(
            Trees::read(header.tree_sizes(), &[0b1000]),
            Err(Error::Invalid("tree past the end"))
        );
    }

    #[test]
    fn the_last_leafs_bits_may_run_past_the_end() {
        // The types' tree alone, its one leaf's low byte coded by a tree of three leaves: 3 + 1 +
        // 31 + 11 + 48 bits, then the leaf's bit and the first of its code's two, the twelfth
        // byte's last two; the code's second bit and the closing bit are read past the end.
        let spec = TreeSpec {
            low: Some(Shape::balanced(&[7, 8, 9])),
            high: Some(Shape::Leaf(1)),
            escapes: [0; 3],
            tree: Shape::Leaf(0x0109),
        };
        let header = crate::build::Header::new(4, 4, [None, None, None, Some(spec)]);
        let bytes = header.trees();
        let cut = &bytes[..12];
        assert_eq!(
            Trees::read(header.tree_sizes(), cut),
            Err(Error::Invalid("header trees"))
        );
        assert!(Trees::read(header.tree_sizes(), &bytes).is_ok());
    }

    #[test]
    fn reads_and_writes_outside_a_list_do_nothing() {
        let mut values = [1, 2];
        set(&mut values, 2, 9);
        assert_eq!((values, get(&values, 2)), ([1, 2], 0));
        assert_eq!(walk(&[NODE | 5], &mut Bits::new(&[1]), false), Ok(6));
        assert_eq!(ByteTree::Value(3).read(&mut Bits::new(&[])), 3);
    }
}
