//! Tests of the reveal's shape (ADV-FOG-002, E08 T8a): the disk of the original's and VCMI's test,
//! clipped to the map and its level, against a tile-by-tile model.

extern crate std;

use std::vec::Vec;

use proptest::prelude::{prop_assert_eq, proptest};

use super::*;

fn p(x: u16, y: u16, level: u8) -> Pos {
    Pos::new(x, y, level)
}

/// The tiles of `grid` that are set, level by level, row by row.
fn set_tiles(grid: &BitGrid) -> Vec<Pos> {
    let mut tiles = Vec::new();
    for level in 0..grid.levels() {
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                if grid.get(p(x, y, level)) {
                    tiles.push(p(x, y, level));
                }
            }
        }
    }
    tiles
}

/// The tiles the reveal should set, tile by tile: on the map, on `center`'s level, with
/// dx² + dy² ≤ r² + r.
fn model(grid: &BitGrid, center: Pos, radius: u8) -> Vec<Pos> {
    let r = i64::from(radius);
    let mut tiles = Vec::new();
    for y in 0..grid.height() {
        for x in 0..grid.width() {
            let dx = i64::from(x) - i64::from(center.x);
            let dy = i64::from(y) - i64::from(center.y);
            if center.level < grid.levels() && dx * dx + dy * dy <= r * r + r {
                tiles.push(p(x, y, center.level));
            }
        }
    }
    tiles
}

// covers: ADV-FOG-002
#[test]
fn radius_five_reveals_ninety_seven_tiles_in_rows_of_five_to_eleven() {
    let mut grid: BitGrid = BitGrid::new(20, 20, 2);
    reveal(&mut grid, p(10, 10, 1), 5);
    assert_eq!(grid.count(), 97);
    // Row widths from the top: |dx| ≤ 2, 3, 4, 5, 5, 5, 5, 5, 4, 3, 2.
    let widths: Vec<usize> = (5..=15)
        .map(|y| (0..20).filter(|&x| grid.get(p(x, y, 1))).count())
        .collect();
    assert_eq!(widths, [5, 7, 9, 11, 11, 11, 11, 11, 9, 7, 5]);
    // The corners of the bounding square stay hidden: 3² + 5² = 34 > 30, 4² + 4² = 32 > 30.
    assert!(grid.get(p(12, 5, 1)) && !grid.get(p(13, 5, 1)));
    assert!(grid.get(p(13, 6, 1)) && !grid.get(p(14, 6, 1)));
    assert!(!grid.get(p(14, 14, 1)));
    // Nothing on the other level.
    assert!(set_tiles(&grid).iter().all(|pos| pos.level == 1));
}

// covers: ADV-FOG-002
#[test]
fn the_reveal_is_clipped_to_the_map() {
    // Radius 0: the centre only.
    let mut grid: BitGrid = BitGrid::new(8, 8, 1);
    reveal(&mut grid, p(3, 4, 0), 0);
    assert_eq!(set_tiles(&grid), [p(3, 4, 0)]);
    // A corner: a quarter of the disk.
    let mut grid: BitGrid = BitGrid::new(8, 8, 1);
    reveal(&mut grid, p(0, 0, 0), 5);
    assert_eq!(set_tiles(&grid), model(&grid, p(0, 0, 0), 5));
    // Rows of |dx| ≤ 5, 5, 5, 4, 3, 2 from the corner.
    assert_eq!(grid.count(), 6 + 6 + 6 + 5 + 4 + 3);
    // A centre past the edges, a level not on the map: only what lies on the map. From (9, 8)
    // with radius 3: (7, 7) at 2² + 1², (6, 7) at 3² + 1², (7, 6) at 2² + 2²; (6, 6) at
    // 3² + 2² = 13 > 12.
    let mut grid: BitGrid = BitGrid::new(8, 8, 1);
    reveal(&mut grid, p(9, 8, 0), 3);
    assert_eq!(set_tiles(&grid), [p(7, 6, 0), p(6, 7, 0), p(7, 7, 0)]);
    reveal(&mut grid, p(u16::MAX, u16::MAX, 0), u8::MAX);
    reveal(&mut grid, p(3, 3, 1), 5);
    assert_eq!(grid.count(), 3);
    // The widest radius over the whole map.
    let mut grid: BitGrid = BitGrid::new(40, 40, 1);
    reveal(&mut grid, p(20, 20, 0), u8::MAX);
    assert_eq!(grid.count(), 40 * 40);
}

proptest! {
    // covers: ADV-FOG-002
    #[test]
    fn the_reveal_sets_the_disk_and_nothing_else(
        width in 1u16..80,
        height in 1u16..80,
        levels in 1u8..3,
        x in 0u16..100,
        y in 0u16..100,
        level in 0u8..3,
        radius in 0u8..12,
    ) {
        let mut grid: BitGrid = BitGrid::new(width, height, levels);
        reveal(&mut grid, p(x, y, level), radius);
        prop_assert_eq!(set_tiles(&grid), model(&grid, p(x, y, level), radius));
    }
}
