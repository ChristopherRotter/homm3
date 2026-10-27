//! Tests of the persistent grid (E08 T5b): reads and writes against a flat model, the one form a
//! leaf is stored in, and the two copy-on-write properties of ADR-0016 point 2.

extern crate std;

use std::format;
use std::vec::Vec;

use h3_kernel::hash::hash_of;
use proptest::prelude::{any, prop_assert, prop_assert_eq, prop_assert_ne, proptest};

use super::*;

fn p(x: u16, y: u16, level: u8) -> Pos {
    Pos::new(x, y, level)
}

#[test]
fn a_new_grid_has_no_tile_set_and_reads_off_the_grid_as_unset() {
    let grid: BitGrid = BitGrid::new(20, 10, 2);
    assert_eq!((grid.width(), grid.height(), grid.levels()), (20, 10, 2));
    assert_eq!(BitGrid::<1>::SIDE, 8);
    assert_eq!(BitGrid::<4>::SIDE, 16);
    assert_eq!(BitGrid::<16>::SIDE, 32);
    assert_eq!(grid.count(), 0);
    assert_eq!(grid.stored_leaves(), 0);
    for pos in [
        p(0, 0, 0),
        p(19, 9, 1),
        p(20, 0, 0),
        p(0, 10, 0),
        p(0, 0, 2),
    ] {
        assert!(!grid.get(pos));
    }
}

#[test]
fn set_and_clear_tiles_across_leaves_and_levels() {
    let mut grid: BitGrid = BitGrid::new(20, 10, 2);
    for pos in [
        p(0, 0, 0),
        p(7, 7, 0),
        p(8, 0, 0),
        p(19, 9, 0),
        p(19, 9, 1),
        p(3, 8, 1),
    ] {
        grid.set(pos, true);
        assert!(grid.get(pos));
    }
    assert_eq!(grid.count(), 6);
    assert!(!grid.get(p(1, 0, 0)));
    // Off the grid nothing happens.
    grid.set(p(20, 0, 0), true);
    grid.set(p(0, 0, 2), true);
    assert_eq!(grid.count(), 6);
    grid.set(p(7, 7, 0), false);
    assert!(!grid.get(p(7, 7, 0)));
    assert_eq!(grid.count(), 5);
}

#[test]
fn a_leaf_is_stored_in_one_form() {
    let mut grid: BitGrid<1> = BitGrid::new(16, 8, 1);
    // An emptied leaf is none.
    grid.set(p(1, 1, 0), true);
    assert_eq!(grid.stored_leaves(), 1);
    grid.set(p(1, 1, 0), false);
    assert_eq!(grid.stored_leaves(), 0);
    // A filled leaf is the shared full leaf, and a fork shares it; clearing a tile copies it.
    for y in 0..8 {
        for x in 0..8 {
            grid.set(p(x, y, 0), true);
        }
    }
    assert!(grid.is_shared_full(p(0, 0, 0)));
    assert!(!grid.is_shared_full(p(8, 0, 0)));
    let fork = grid.clone();
    grid.set(p(2, 3, 0), false);
    assert!(!grid.is_shared_full(p(0, 0, 0)));
    assert!(fork.is_shared_full(p(0, 0, 0)));
    assert_eq!((grid.count(), fork.count()), (63, 64));
}

#[test]
fn a_run_sets_or_clears_a_row_across_leaves() {
    let mut grid: BitGrid<1> = BitGrid::new(20, 10, 2);
    // From column 5 to 17 of row 3: three leaves (columns 5–7, 8–15, 16–17).
    grid.set_run(p(5, 3, 1), 13, true);
    assert_eq!(grid.count(), 13);
    assert!((5..18).all(|x| grid.get(p(x, 3, 1))));
    assert!(!grid.get(p(4, 3, 1)) && !grid.get(p(18, 3, 1)) && !grid.get(p(5, 3, 0)));
    grid.set_run(p(15, 9, 0), 40, true);
    assert_eq!(grid.count(), 18);
    // Clearing a run inside one leaf, and runs that change nothing.
    grid.set_run(p(9, 3, 1), 3, false);
    assert_eq!(grid.count(), 15);
    let before = grid.clone();
    grid.set_run(p(9, 3, 1), 3, false);
    grid.set_run(p(5, 3, 1), 4, true);
    grid.set_run(p(0, 0, 0), 0, true);
    grid.set_run(p(20, 0, 0), 5, true);
    grid.set_run(p(0, 10, 0), 5, true);
    grid.set_run(p(0, 0, 2), 5, true);
    assert!(grid.shares_table(&before));
    // A whole row of a 64-tile leaf is one word.
    let mut wide: BitGrid<64> = BitGrid::new(70, 3, 1);
    assert_eq!(BitGrid::<64>::SIDE, 64);
    wide.set_run(p(0, 1, 0), 70, true);
    assert_eq!(wide.count(), 70);
}

#[test]
fn a_fork_shares_until_it_writes_and_a_write_that_changes_nothing_copies_nothing() {
    let mut grid: BitGrid = BitGrid::new(36, 36, 1);
    grid.set(p(5, 5, 0), true);
    let mut fork = grid.clone();
    assert!(fork.shares_table(&grid));
    fork.set(p(5, 5, 0), true);
    fork.set(p(6, 5, 0), false);
    fork.set(p(40, 5, 0), true);
    assert!(fork.shares_table(&grid));
    fork.set(p(6, 5, 0), true);
    assert!(!fork.shares_table(&grid));
    assert!(!grid.get(p(6, 5, 0)));
}

#[test]
fn equality_debug_and_hash_see_content_only() {
    let mut a: BitGrid<1> = BitGrid::new(16, 16, 1);
    a.set(p(9, 9, 0), true);
    // The same content reached another way: set and cleared again, in another order.
    let mut b: BitGrid<1> = BitGrid::new(16, 16, 1);
    b.set(p(0, 0, 0), true);
    b.set(p(9, 9, 0), true);
    b.set(p(0, 0, 0), false);
    assert!(!a.shares_table(&b));
    assert_eq!(a, b);
    assert_eq!(hash_of(&a), hash_of(&b));
    assert_eq!(format!("{a:?}"), format!("{b:?}"));
    // (9, 9) is in leaf 3 (the second of the second row of leaves), bit 9 of its word.
    assert_eq!(
        format!("{a:?}"),
        "BitGrid { width: 16, height: 16, levels: 1, set: 1, leaves: [(3, [512])] }"
    );
    // Other content or other dimensions differ.
    b.set(p(9, 10, 0), true);
    assert_ne!(a, b);
    assert_ne!(hash_of(&a), hash_of(&b));
    let c: BitGrid<1> = BitGrid::new(16, 8, 2);
    assert_ne!(BitGrid::<1>::new(16, 16, 1), c);
    assert_ne!(hash_of(&BitGrid::<1>::new(16, 16, 1)), hash_of(&c));
    // A fork equals its parent.
    assert_eq!(a.clone(), a);
}

/// A flat model of a grid: one bool per tile.
fn model_index(width: u16, height: u16, pos: Pos) -> usize {
    (usize::from(pos.level) * usize::from(height) + usize::from(pos.y)) * usize::from(width)
        + usize::from(pos.x)
}

/// An operation of the properties: set or clear a run of tiles of a row (one tile through `set`;
/// coordinates may fall off the grid).
type Op = (u16, u16, u8, u16, bool);

fn apply_ops<const W: usize>(grid: &mut BitGrid<W>, model: &mut [bool], ops: &[Op]) {
    for &(x, y, level, len, on) in ops {
        if len == 1 {
            grid.set(p(x, y, level), on);
        } else {
            grid.set_run(p(x, y, level), len, on);
        }
        if y < grid.height() && level < grid.levels() {
            for x in x..x.saturating_add(len).min(grid.width()) {
                model[model_index(grid.width(), grid.height(), p(x, y, level))] = on;
            }
        }
    }
}

fn matches<const W: usize>(grid: &BitGrid<W>, model: &[bool]) -> bool {
    let (width, height) = (grid.width(), grid.height());
    (0..grid.levels()).all(|level| {
        (0..height).all(|y| {
            (0..width).all(|x| {
                grid.get(p(x, y, level)) == model[model_index(width, height, p(x, y, level))]
            })
        })
    })
}

fn ops() -> impl proptest::strategy::Strategy<Value = Vec<Op>> {
    let len = proptest::prop_oneof![3 => proptest::strategy::Just(1u16), 1 => 0u16..80];
    proptest::collection::vec((0u16..90, 0u16..45, 0u8..3, len, any::<bool>()), 0..120)
}

proptest! {
    /// ADR-0016 point 2, the first property: a write through a fork never reaches its parent or
    /// a sibling; each matches its own flat model.
    #[test]
    fn a_write_through_a_fork_reaches_nothing_else(
        width in 1u16..81,
        height in 1u16..41,
        levels in 1u8..3,
        base in ops(),
        first in ops(),
        second in ops(),
    ) {
        let size = usize::from(width) * usize::from(height) * usize::from(levels);
        let mut grid: BitGrid = BitGrid::new(width, height, levels);
        let mut model = std::vec![false; size];
        apply_ops(&mut grid, &mut model, &base);
        let (mut a, mut model_a) = (grid.clone(), model.clone());
        let (mut b, mut model_b) = (grid.clone(), model.clone());
        apply_ops(&mut a, &mut model_a, &first);
        apply_ops(&mut b, &mut model_b, &second);
        prop_assert!(matches(&grid, &model));
        prop_assert!(matches(&a, &model_a));
        prop_assert!(matches(&b, &model_b));
        let count = |m: &[bool]| u32::try_from(m.iter().filter(|&&on| on).count()).unwrap();
        prop_assert_eq!((grid.count(), a.count(), b.count()), (count(&model), count(&model_a), count(&model_b)));
    }

    /// ADR-0016 point 2, the second property: grids compare and hash equal exactly when their
    /// content is equal, whatever they share, for each leaf size.
    #[test]
    fn equal_content_compares_and_hashes_equal_whatever_the_sharing(
        width in 1u16..41,
        height in 1u16..41,
        base in ops(),
        first in ops(),
        second in ops(),
    ) {
        fn check<const W: usize>(width: u16, height: u16, base: &[Op], first: &[Op], second: &[Op]) -> Result<(), proptest::test_runner::TestCaseError> {
            let size = usize::from(width) * usize::from(height) * 2;
            let mut grid: BitGrid<W> = BitGrid::new(width, height, 2);
            let mut model = std::vec![false; size];
            apply_ops(&mut grid, &mut model, base);
            // `a` forks and writes; `b` builds from nothing, all of base and first in one go.
            let (mut a, mut model_a) = (grid.clone(), model.clone());
            apply_ops(&mut a, &mut model_a, first);
            let mut b: BitGrid<W> = BitGrid::new(width, height, 2);
            let mut model_b = std::vec![false; size];
            let all: Vec<Op> = base.iter().chain(first).copied().collect();
            apply_ops(&mut b, &mut model_b, &all);
            prop_assert_eq!(&a, &b);
            prop_assert_eq!(hash_of(&a), hash_of(&b));
            prop_assert_eq!(format!("{a:?}"), format!("{b:?}"));
            // A sibling: equal exactly when its model is.
            let (mut c, mut model_c) = (grid.clone(), model);
            apply_ops(&mut c, &mut model_c, second);
            prop_assert_eq!(a == c, model_a == model_c);
            if model_a != model_c {
                prop_assert_ne!(hash_of(&a), hash_of(&c));
            }
            Ok(())
        }
        check::<1>(width, height, &base, &first, &second)?;
        check::<4>(width, height, &base, &first, &second)?;
        check::<16>(width, height, &base, &first, &second)?;
        check::<64>(width, height, &base, &first, &second)?;
    }
}

proptest! {
    /// A row reads as its tiles (E08 T8b1, the observation's planes): bit `x % 64` of word
    /// `x / 64` is the tile of column `x`, bits past the width are clear, and the words after the
    /// row's are left as they were, for each leaf size.
    #[test]
    fn a_row_reads_as_its_tiles(
        width in 1u16..200,
        height in 1u16..41,
        base in ops(),
    ) {
        fn check<const W: usize>(width: u16, height: u16, base: &[Op]) -> Result<(), proptest::test_runner::TestCaseError> {
            let mut grid: BitGrid<W> = BitGrid::new(width, height, 2);
            let mut model = std::vec![false; usize::from(width) * usize::from(height) * 2];
            let wide: Vec<Op> = base.iter().map(|&(x, y, level, len, on)| (x * 2, y, level, len * 2, on)).collect();
            apply_ops(&mut grid, &mut model, &wide);
            let stride = usize::from(width.div_ceil(64));
            for level in 0..2 {
                for y in 0..height {
                    let mut out = std::vec![u64::MAX; stride + 1];
                    grid.read_row(level, y, &mut out);
                    prop_assert_eq!(out[stride], u64::MAX);
                    for x in 0..u16::try_from(stride * 64).unwrap() {
                        let bit = out[usize::from(x / 64)] >> (x % 64) & 1 == 1;
                        let tile = x < width && model[model_index(width, height, p(x, y, level))];
                        prop_assert_eq!(bit, tile);
                    }
                }
            }
            Ok(())
        }
        check::<1>(width, height, &base)?;
        check::<4>(width, height, &base)?;
        check::<16>(width, height, &base)?;
        check::<64>(width, height, &base)?;
    }
}

#[test]
fn a_run_on_the_widest_grid_does_not_overflow() {
    // The last leaf boundary of a 65 520-tile row is 65 536, past `u16`: it saturates.
    let mut grid: BitGrid = BitGrid::new(65_520, 1, 1);
    grid.set_run(p(65_510, 0, 0), 20, true);
    assert_eq!(grid.count(), 10);
    assert!(grid.get(p(65_519, 0, 0)));
}

/// E08 T61b: the persistent table's entries by index, across leaves and mid tables.
#[test]
fn a_table_sets_reads_and_removes_entries_by_index() {
    let mut table: Table<u32> = Table::new();
    assert_eq!(
        (table.len(), table.get(0), table.stored()),
        (0, None, (0, 0))
    );
    // Entries in one leaf, in two leaves of one mid table, and in a later mid table.
    for (index, value) in [(3, 30), (15, 150), (16, 160), (5000, 50)] {
        table.set(index, Some(value));
    }
    assert_eq!(table.len(), 4);
    assert_eq!(
        (table.get(3), table.get(15), table.get(16), table.get(5000)),
        (Some(&30), Some(&150), Some(&160), Some(&50))
    );
    assert_eq!(
        (table.get(4), table.get(4999), table.get(1 << 40)),
        (None, None, None)
    );
    assert_eq!(table.stored(), (2, 3));
    let entries: Vec<(usize, u32)> = table.iter().map(|(i, &v)| (i, v)).collect();
    assert_eq!(entries, [(3, 30), (15, 150), (16, 160), (5000, 50)]);
    // A value is replaced in place.
    table.set(3, Some(31));
    assert_eq!((table.len(), table.get(3)), (4, Some(&31)));
    // A part without an entry is no part: the leaf, then the mid table, then the top go.
    table.set(16, None);
    assert_eq!((table.len(), table.stored()), (3, (2, 2)));
    table.set(5000, None);
    assert_eq!((table.len(), table.stored()), (2, (1, 1)));
    table.set(3, None);
    table.set(15, None);
    assert_eq!((table.len(), table.stored()), (0, (0, 0)));
    assert!(table.shares_top(&Table::new()));
    assert_eq!(table, Table::new());
}

#[test]
fn a_tables_fork_shares_until_it_writes_and_a_write_copies_only_its_path() {
    let mut table: Table<Vec<u8>> = Table::new();
    table.set(1, Some(std::vec![1]));
    table.set(2000, Some(std::vec![2]));
    let mut fork = table.clone();
    assert!(fork.shares_top(&table));
    // Removing what is not there copies nothing.
    fork.set(7, None);
    assert!(fork.shares_top(&table));
    // A write copies the top, one mid table and one leaf; every value stays shared, the
    // untouched entries' leaves too.
    fork.set(3, Some(std::vec![3]));
    assert!(!fork.shares_top(&table));
    assert!(fork.shares_value(&table, 1) && fork.shares_value(&table, 2000));
    assert_eq!((table.len(), table.get(3)), (2, None));
    assert_eq!((fork.len(), fork.get(3)), (3, Some(&std::vec![3])));
    assert!(!fork.shares_value(&table, 3));
}

#[test]
fn a_tables_equality_and_debug_see_content_only() {
    let mut a: Table<u8> = Table::new();
    a.set(40, Some(4));
    a.set(2, Some(2));
    let mut b: Table<u8> = Table::new();
    b.set(2, Some(2));
    b.set(9000, Some(9));
    b.set(40, Some(4));
    assert_ne!(a, b);
    b.set(9000, None);
    assert_eq!(a, b);
    assert!(!a.shares_top(&b));
    assert_eq!(format!("{a:?}"), "{2: 2, 40: 4}");
    let mut c = a.clone();
    c.set(40, Some(5));
    assert_ne!(a, c);
    let mut d = a.clone();
    d.set(41, Some(4));
    d.set(40, None);
    assert_ne!(a, d);
}

proptest! {
    /// The table against a model: a write through a fork reaches nothing else, and each reads,
    /// counts and iterates as its own ordered map.
    #[test]
    fn a_tables_writes_match_an_ordered_map(
        base in proptest::collection::vec((0usize..5000, proptest::option::of(any::<u8>())), 0..60),
        first in proptest::collection::vec((0usize..5000, proptest::option::of(any::<u8>())), 0..60),
    ) {
        let apply = |table: &mut Table<u8>, model: &mut std::collections::BTreeMap<usize, u8>, ops: &[(usize, Option<u8>)]| {
            for &(index, value) in ops {
                table.set(index, value);
                match value {
                    Some(v) => model.insert(index, v),
                    None => model.remove(&index),
                };
            }
        };
        let mut table = Table::new();
        let mut model = std::collections::BTreeMap::new();
        apply(&mut table, &mut model, &base);
        let (mut fork, mut fork_model) = (table.clone(), model.clone());
        apply(&mut fork, &mut fork_model, &first);
        for (table, model) in [(&table, &model), (&fork, &fork_model)] {
            prop_assert_eq!(table.len(), model.len());
            let entries: Vec<(usize, u8)> = table.iter().map(|(i, &v)| (i, v)).collect();
            let expected: Vec<(usize, u8)> = model.iter().map(|(&i, &v)| (i, v)).collect();
            prop_assert_eq!(entries, expected);
            for index in 0..5000 {
                prop_assert_eq!(table.get(index), model.get(&index));
            }
        }
        prop_assert_eq!(table == fork, model == fork_model);
        if table.len() == 0 {
            prop_assert_eq!(table.stored(), (0, 0));
        }
        prop_assert_ne!(fork.len() + 1, 0);
    }
}
