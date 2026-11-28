//! The joining on small maps (RMG-CON-009 to -011): the cost flood's steps, costs, zeros and
//! entrances, the carve, the seeds, the fallback and RMG-QUIRK-5. The corpus's against the phase
//! oracle are `h3-pack-rmg`'s.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use super::*;
use crate::cells::Cell;
use crate::generate::GenerateError;
use crate::layout::ZoneGenerator;
use crate::painter::Pictures;
use crate::painter::tests::tables;
use crate::part::{OBJECT_TYPES, ObjectSettings, ObjectTemplate};
use crate::plan::tests::{bits, drawn, generator, no_guards, no_objects, pend, zone};
use crate::settings::MapFormat;

/// The unreached cost of a cell as reset (`rmg_map_cell_reset`).
const RESET: u32 = 32_700;

/// Level 0's costs of the cells' own zones' floods (+0x1C bits 0-15), row by row.
fn near(cells: &Cells) -> Vec<Vec<u32>> {
    let (w, h, _) = cells.size();
    let at = |x, y| cells.get(x, y, 0).map_or(0, Cell::near_cost);
    (0..h).map(|y| (0..w).map(|x| at(x, y)).collect()).collect()
}

fn cell(cells: &Cells, x: i32, y: i32) -> Cell {
    *cells.get(x, y, 0).unwrap()
}

/// Marks the cells `#` of `rows` on level 0 with `bit`.
fn mark(cells: &mut Cells, rows: &[&str], bit: u32) {
    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            if ch == '#' {
                cells.get_mut(x as i32, y as i32, 0).unwrap().tile_data |= bit;
            }
        }
    }
}

/// Clears `bit` of the cells `#` of `rows` on level 0.
fn unmark(cells: &mut Cells, rows: &[&str], bit: u32) {
    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            if ch == '#' {
                cells.get_mut(x as i32, y as i32, 0).unwrap().tile_data &= !bit;
            }
        }
    }
}

// covers: RMG-CON-010
/// The flood steps NE to E, +1 inside its zone and +10 onto its water, around rock, cells of no
/// zone and the map's edge, taking equal costs first listed first (the previous cells); into
/// another zone +10 per step in +0x1C's high half, the step back in +0x28 bits 12-14 and the
/// seed's zone in +0x20 bits 24-31, on inside that zone but never back into the seed's.
#[test]
fn the_cost_flood_steps_1_inside_its_zone_and_10_into_another() {
    let mut cells = drawn(&[&["0001", "0r01", ".0w1", "...."]]);
    // The rock of zone 0.
    cells.get_mut(1, 1, 0).unwrap().zone &= 0xFF00_FFFF;
    cost_flood(&mut cells, &no_objects(), [0, 0, 0], false);
    assert_eq!(
        near(&cells),
        [
            [0, 1, 2, RESET],
            [1, RESET, 2, RESET],
            [RESET, 2, 12, RESET],
            [RESET; 4]
        ]
    );
    assert_eq!(cell(&cells, 0, 0).previous, [-1; 3]);
    assert_eq!(cell(&cells, 2, 0).previous, [1, 0, 0]);
    assert_eq!(cell(&cells, 1, 2).previous, [0, 1, 0]);
    assert_eq!(cell(&cells, 2, 2).previous, [1, 2, 0]);
    for (y, step) in [(0, 3), (1, 4), (2, 5)] {
        let c = cell(&cells, 3, y);
        assert_eq!(c.far_cost(), 12);
        assert_eq!((c.tile_data >> 12) & 7, step);
        assert_eq!(c.flooded_by(), 0);
        assert_eq!(c.near_cost(), RESET);
    }
    assert_eq!(cell(&cells, 2, 1).far_cost(), RESET);
    assert_eq!(cell(&cells, 2, 1).flooded_by(), -1);
}

// covers: RMG-CON-010
/// From a cell of cost 0 a kept-free cell (bit 27) is entered at 0, water only with the water
/// flag; a third zone is not entered from the second's cells.
#[test]
fn the_cost_flood_joins_kept_free_cells_at_0() {
    for (water, pool) in [(false, 10), (true, 0)] {
        let mut cells = drawn(&[&["00012", "w0012", ".....", ".....", "....."]]);
        mark(&mut cells, &[".##..", "#...."], KEPT);
        cost_flood(&mut cells, &no_objects(), [0, 0, 0], water);
        assert_eq!(
            near(&cells)[..2],
            [[0, 0, 0, RESET, RESET], [pool, 1, 1, RESET, RESET]]
        );
        assert!(near(&cells)[2..].iter().flatten().all(|&c| c == RESET));
        for y in 0..2 {
            assert_eq!(cell(&cells, 3, y).far_cost(), 10);
            assert_eq!(cell(&cells, 4, y).far_cost(), RESET);
        }
    }
}

/// A 1 × 1 template of class `class` whose one cell is an entrance.
fn entrance(class: u32) -> ObjectTemplate {
    ObjectTemplate {
        picture: String::from("AVXtest.def"),
        passability: (1 << 48) - 2,
        actions: 1,
        editor_terrains: 0x1FF,
        terrains: 0x1FE,
        class,
        subclass: 0,
        group: 1,
        overlay: 0,
        width: 1,
        height: 1,
        colour: (1 << 48) - 1,
    }
}

/// Class 0 visited from beside and not removable, class 1 visited from beside but removable,
/// class 2 entered from above; the others clear.
fn entrance_settings() -> Vec<ObjectSettings> {
    (0..OBJECT_TYPES as u8)
        .map(|class| ObjectSettings {
            not_entered: class < 2,
            from_above: class == 2,
            removable: class == 1,
            class_group: class,
        })
        .collect()
}

/// The objects of classes 0 to 2's entrances placed at `at` on `cells`, in order.
fn entrances(cells: &mut Cells, at: &[([i32; 2], usize)]) -> Objects {
    let lines = [entrance(0), entrance(1), entrance(2)];
    let mut objects = Objects::new(&lines, &entrance_settings(), MapFormat::SoD, 1);
    for &([x, y], class) in at {
        objects.add(cells, class, [x, y, 0]);
    }
    objects
}

// covers: RMG-CON-010
/// An entrance whose class has object settings +0 set and +2 clear is never entered; one with +1
/// clear is not entered by SE, S or SW (the seed's S neighbour is reached around it) and is left
/// only by W, SW, S, SE and E; with +1 set by all eight.
#[test]
fn the_cost_flood_enters_and_leaves_entrances_as_their_settings_allow() {
    for (class, middle) in [(1, 2), (2, 1)] {
        let mut cells = drawn(&[&["000", "000", "000"]]);
        let objects = entrances(&mut cells, &[([1, 1], class), ([2, 2], 0)]);
        // Not kept free: entered at their costs, not at 0.
        unmark(&mut cells, &["...", ".#.", "..#"], KEPT);
        cost_flood(&mut cells, &objects, [1, 0, 0], false);
        assert_eq!(near(&cells), [[1, 0, 1], [1, middle, 1], [2, 2, RESET]]);
    }
    for (class, top) in [(1, 2), (2, 1)] {
        let mut cells = drawn(&[&["000", "000", "000"]]);
        let objects = entrances(&mut cells, &[([1, 1], class)]);
        cost_flood(&mut cells, &objects, [1, 1, 0], false);
        assert_eq!(near(&cells), [[top, top, top], [1, 0, 1], [1, 1, 1]]);
    }
}

// covers: RMG-CON-011
/// A carve walks back from its start while the cost is above 0: each cell kept free, the
/// obstacles of its 3 × 3 in the start's zone cleared but a pending guard's and another zone's;
/// the cell of cost 0 it reaches is left. A start of cost 0 or of 30000 and more carves nothing.
/// With the Border Guard flag (a land connection's) the 3 × 3s keep their obstacles.
#[test]
fn a_carve_walks_back_to_cost_0() {
    let mut cells = drawn(&[&["00001", "00001", "00000", ".....", "....."]]);
    mark(&mut cells, &["#####", "#####", "#####"], OBSTACLES);
    pend(&mut cells, 2, 1, 0);
    for (x, cost) in [(0, 0), (1, 1), (2, 2), (3, 3)] {
        let c = cells.get_mut(x, 0, 0).unwrap();
        c.movement = (c.movement & 0xFFFF_0000) | cost;
        c.previous = [x - 1, 0, 0];
    }
    let marks = |cells: &Cells| -> Vec<u32> { cells.cells().iter().map(|c| c.tile_data).collect() };
    let before = marks(&cells);
    for cost in [0, REACHED] {
        cells.get_mut(3, 0, 0).unwrap().movement = cost;
        carve(&mut cells, [3, 0, 0], false);
        assert_eq!(marks(&cells), before);
    }
    cells.get_mut(3, 0, 0).unwrap().movement = 3;
    let mut flagged = cells.clone();
    carve(&mut flagged, [3, 0, 0], true);
    assert_eq!(
        bits(&flagged, 0, KEPT),
        [".###.", ".....", ".....", ".....", "....."]
    );
    assert_eq!(
        bits(&flagged, 0, OBSTACLES),
        ["#...#", "#####", "#####", ".....", "....."]
    );
    carve(&mut cells, [3, 0, 0], false);
    assert_eq!(
        bits(&cells, 0, KEPT),
        [".###.", ".....", ".....", ".....", "....."]
    );
    assert_eq!(
        bits(&cells, 0, OBSTACLES),
        ["....#", "..#.#", "#####", ".....", "....."]
    );
}

/// The joining of `gens` on `cells` with the zones 0 to 2 and `objects`.
fn join_of(cells: &mut Cells, gens: &[ZoneGenerator], objects: &Objects) -> Result<(), usize> {
    let zones = [zone(0), zone(1), zone(2)];
    let pictures = Pictures::new(&tables());
    let input = PlanInput {
        zones: &zones,
        connections: &[],
        gens,
        objects: &mut objects.clone(),
        road_targets: &mut Vec::new(),
        pictures: &pictures,
        cosmetic: true,
        guards: no_guards(),
        shipyard_sites_repeated: true,
    };
    join(&input, cells)
}

/// A land zone generator of zone `k` over `bounds` on level 0.
fn land_zone(k: usize, bounds: [i32; 4]) -> ZoneGenerator {
    ZoneGenerator {
        bounds,
        ..generator(k)
    }
}

// covers: RMG-CON-009, RMG-CON-010, RMG-CON-011
/// The joining resets the floods, then per zone generator: zone 0 seeds at its first empty,
/// kept-free, passable land cell (its water and an entrance passed), joins its kept-free cells at
/// 0, carves from the one left unjoined (kept free back to cost 0, the obstacles beside cleared
/// but a pending guard's) and floods from it again (a cell entered at 1 before stays 1); zone 1,
/// without a kept-free cell, makes its last empty one kept free and seeds there; zone 2, without
/// a cell, seeds nothing.
#[test]
fn each_zones_free_cells_are_joined() {
    let mut cells = drawn(&[&["w00000", "100000", "......", "......", "......", "......"]]);
    unmark(&mut cells, &["......", ".#####"], FREE);
    mark(&mut cells, &["#.#..#", "......"], KEPT);
    mark(&mut cells, &["...##.", "######"], OBSTACLES);
    pend(&mut cells, 4, 1, 0);
    let objects = entrances(&mut cells, &[([1, 0], 1)]);
    // A cell no flood enters: its floods reset.
    let untouched = cells.get_mut(5, 1, 0).unwrap();
    untouched.previous = [5, 5, 5];
    untouched.zone &= 0x00FF_FFFF;
    untouched.tile_data |= STEP;
    let gens = [
        land_zone(0, [0, 0, 6, 2]),
        land_zone(1, [0, 0, 6, 2]),
        generator(2),
    ];
    assert_eq!(join_of(&mut cells, &gens, &objects), Ok(()));
    let unset = 32_000;
    assert_eq!(
        near(&cells)[..2],
        [[10, 0, 0, 1, 0, 0], [0, unset, unset, unset, unset, unset]]
    );
    assert!(near(&cells)[2..].iter().flatten().all(|&c| c == unset));
    assert_eq!(bits(&cells, 0, KEPT)[..2], ["######", "#....."]);
    assert_eq!(bits(&cells, 0, OBSTACLES)[..2], ["......", ".#..#."]);
    // Zone 1's seed was entered by zone 0's flood; zone 1's entered zone 0's cells, on through
    // them (the entrance left eastward).
    let flooded = |x, y| {
        let c = cell(&cells, x, y);
        (c.far_cost(), (c.tile_data >> 12) & 7, c.flooded_by())
    };
    assert_eq!(cell(&cells, 0, 1).previous, [-1; 3]);
    assert_eq!(flooded(0, 1), (10, 7, 0));
    assert_eq!(flooded(0, 0), (10, 2, 1));
    assert_eq!(flooded(5, 0), (50, 4, 1));
    let c = cell(&cells, 5, 1);
    assert_eq!(c.movement, 0x7D00_7D00);
    assert_eq!(c.previous, [-1; 3]);
    assert_eq!((flooded(5, 1).1, flooded(5, 1).2), (0, -1));
}

// covers: RMG-CON-009
/// RMG-QUIRK-5's regression test (exploit-fix): a zone generator without an empty cell of its
/// zone, before any had one, is refused (the original seeds it from the stack's bytes); its
/// fallback seed with a pending guard is not marked; after one with an empty cell, a zone
/// generator without one seeds nothing (the original's is that one's seed: no change).
#[test]
fn a_zone_without_an_empty_cell_is_refused_before_any_had_one() {
    let mut cells = drawn(&[&["00", "11"]]);
    assert_eq!(join_of(&mut cells, &[generator(0)], &no_objects()), Err(0));
    let gens = [land_zone(0, [0, 0, 2, 2]), land_zone(1, [0, 0, 2, 2])];
    let mut full = drawn(&[&["11", "00"]]);
    let objects = entrances(&mut full, &[([0, 1], 2), ([1, 1], 2)]);
    assert_eq!(join_of(&mut full, &gens, &objects), Err(0));
    let objects = entrances(&mut cells, &[([0, 1], 2), ([1, 1], 2)]);
    pend(&mut cells, 1, 0, 0);
    assert_eq!(join_of(&mut cells, &gens, &objects), Ok(()));
    // Zone 0's fallback seed (1, 0) left unmarked; zone 1 flooded from its first entrance.
    assert_eq!(bits(&cells, 0, KEPT), ["..", "##"]);
    assert_eq!(near(&cells), [[1, 0], [0, 0]]);
    let e = GenerateError::NoEmptyCell { step: 6, zone: 3 };
    assert_eq!(
        e.to_string(),
        "step 6: zone generator 3's box holds no empty cell of its zone"
    );
}
