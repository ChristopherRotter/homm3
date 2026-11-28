//! The islets on small maps (RMG-CON-007, -008): the distance flood, the height map's records
//! and draws, an islet painted and made obstacles, the zones' loop and the painter's fault. The
//! corpus's islets against the phase oracle are `h3-pack-rmg`'s.

use alloc::vec::Vec;

use super::*;
use crate::cells::Cell;
use crate::layout::ZoneGenerator;
use crate::painter::Pictures;
use crate::painter::tests::tables;
use crate::part::GeneratorTables;
use crate::plan::tests::{Log, drawn, generator, land, no_guards, no_objects, pend, zone};

/// The far costs (+0x1C bits 16-31) of level 0, row by row.
fn far(cells: &Cells) -> Vec<Vec<u32>> {
    let (w, h, _) = cells.size();
    let at = |x, y| cells.get(x, y, 0).map_or(0, Cell::far_cost);
    (0..h).map(|y| (0..w).map(|x| at(x, y)).collect()).collect()
}

/// The flood's steps (+0x28 bits 12-14) of level 0, row by row.
fn steps(cells: &Cells) -> Vec<Vec<u32>> {
    let (w, h, _) = cells.size();
    let at = |x, y| cells.get(x, y, 0).map_or(0, |c| (c.tile_data >> 12) & 7);
    (0..h).map(|y| (0..w).map(|x| at(x, y)).collect()).collect()
}

// covers: RMG-CON-007
/// The flood steps 2 orthogonally and 3 diagonally inside its zone only (the seed of none, a
/// cell of zone 1 left), enters a cell only below its cost, and takes equal costs first listed
/// first: (2, 2) is entered from (2, 1) going S (step 2), listed before (1, 2), whose E step
/// costs the same. An entered cell keeps +0x1C's low half and loses +0x20 bits 24-31.
#[test]
fn the_distance_flood_steps_2_and_3_inside_its_zone_least_first() {
    let mut cells = drawn(&[&[".00", "010", "000"]]);
    for y in 0..3 {
        for x in 0..3 {
            if (x, y) != (1, 1) {
                cells.clamped_mut(x, y, 0).enter_far(32_000, 5);
            }
        }
    }
    distance_flood(&mut cells, [0, 0, 0], 0);
    assert_eq!(far(&cells), [[0, 2, 4], [2, 32_700, 5], [4, 5, 7]]);
    assert_eq!(steps(&cells), [[0, 0, 0], [2, 0, 1], [2, 1, 2]]);
    let corner = cells.get(2, 2, 0).unwrap();
    assert_eq!(corner.movement & 0xFFFF, Cell::RESET.movement & 0xFFFF);
    assert_eq!(corner.zone >> 24, 0);
    assert_eq!(cells.get(1, 1, 0).unwrap().zone >> 24, 0xFF);
}

/// The draws' values in order.
fn values(log: &Log) -> Vec<i32> {
    log.draws.iter().map(|&(_, v)| v as i32).collect()
}

// covers: RMG-CON-008
/// A 2 × 2 box is split at once: four cells of one cell each, no draw, the bottom-right taking
/// the centre a / 2 = 1 ((2 + 2) / 4 + 1 = 2).
#[test]
fn a_small_height_map_is_split_without_a_draw() {
    let mut log = Log::default();
    let mask = heightmap(&mut Stream::new(7, &mut log), 2, 2);
    assert_eq!(mask, [0, 0, 0, 1]);
    assert!(log.draws.is_empty());
}

// covers: RMG-CON-008
/// A 4 × 4 box (a = 3, the centre 1): its quadrants top-left, top-right, bottom-left,
/// bottom-right in turn, each displacing its top, left, bottom, right and centre by
/// `rand() % 3 − 1`, and its four cells (amplitude 2) taking their top-left heights clamped to 0
/// and up: the quadrant's own corner, its top, its left, its centre.
#[test]
fn a_height_map_displaces_each_split_and_keeps_the_top_left_heights() {
    let mut log = Log::default();
    let mask = heightmap(&mut Stream::new(3, &mut log), 4, 4);
    let sites: Vec<u32> = log.draws.iter().map(|&(s, _)| s).collect();
    assert_eq!(sites, [SITE_ISLET_HEIGHTS; 4].concat());
    let v = values(&log);
    let d = |i: usize| (v[i] % 3 - 1).clamp(0, 255) as u8;
    // Per quadrant: its cells (top-left, top-right, bottom-left, bottom-right), its draws from.
    // Every mean of its edges and centre is 0 but (bottom-right) its own corner 1.
    let quadrant = |corner: u8, k: usize| [corner, d(k), d(k + 1), d(k + 4)];
    let [tl, tr, bl, br] = [
        quadrant(0, 0),
        quadrant(0, 5),
        quadrant(0, 10),
        quadrant(1, 15),
    ];
    let expected = [
        tl[0], tl[1], tr[0], tr[1], tl[2], tl[3], tr[2], tr[3], bl[0], bl[1], br[0], br[1], bl[2],
        bl[3], br[2], br[3],
    ];
    assert_eq!(mask, expected);
}

// covers: RMG-CON-008
/// One row of 5 (a = 2): quadrants of no row are skipped, a record of one row and two cells is
/// split; amplitude 1 (the whole's quadrants' children) splits without a draw: two records draw,
/// ten draws. One column of 3: the quadrants of no column are skipped.
#[test]
fn a_height_maps_empty_quadrants_are_skipped_and_amplitude_1_draws_nothing() {
    let mut log = Log::default();
    let mask = heightmap(&mut Stream::new(1, &mut log), 5, 1);
    assert_eq!(mask, [0; 5]);
    assert_eq!(log.draws.len(), 10);
    let mut log = Log::default();
    let mask = heightmap(&mut Stream::new(1, &mut log), 1, 3);
    assert_eq!(log.draws.len(), 5);
    // The top cell is the whole's top 0; the bottom ones a split's top and centre.
    let v = values(&log);
    let d = |i: usize| (v[i] % 2 - 1).clamp(0, 255) as u8;
    assert_eq!(mask, [0, d(0), d(4)]);
}

/// A `side` × `side` level of water of no zone with the box [x0, x1) × [y0, y1) of zone 0, its
/// zone generator (water), the zones and pictures of [`tables`].
fn sea(side: i32, [x0, y0, x1, y1]: [i32; 4]) -> (Cells, ZoneGenerator) {
    let mut cells = land(side, 1);
    for y in 0..side {
        for x in 0..side {
            let c = cells.clamped_mut(x, y, 0);
            c.tile = u32::from(WATER);
            if (x0..x1).contains(&x) && (y0..y1).contains(&y) {
                c.zone &= 0xFF00_FFFF;
            }
        }
    }
    let g = ZoneGenerator {
        terrain: i32::from(WATER),
        bounds: [x0, y0, x1, y1],
        ..generator(0)
    };
    (cells, g)
}

// covers: RMG-CON-008
/// An islet over a 4 × 4 box of water: the terrain `rand() % 6`, the height map's twenty draws,
/// then the painter's; every cell of the box made land is an obstacle (bit 26 set, 27 cleared)
/// unless it has a pending Border Guard, the box's water is left.
#[test]
fn an_islet_is_painted_and_its_land_made_obstacles() {
    let (mut cells, _) = sea(10, [0, 0, 10, 10]);
    for y in 0..10 {
        for x in 0..10 {
            cells.clamped_mut(x, y, 0).tile_data |= KEPT;
        }
    }
    // The bottom-right quadrant's top-left cell is always land (its height 1).
    pend(&mut cells, 5, 5, 0);
    let zones = [zone(0)];
    let pictures = Pictures::new(&tables());
    let input = PlanInput {
        zones: &zones,
        connections: &[],
        gens: &[],
        objects: &mut no_objects(),
        pictures: &pictures,
        cosmetic: true,
        guards: no_guards(),
    };
    let mut log = Log::default();
    let before = cells.clone();
    let painted = islet(
        &mut Stream::new(2, &mut log),
        &input,
        &mut cells,
        [3, 3, 7, 7],
        0,
    );
    assert_eq!(painted, Ok(()));
    let terrain = log.draws[0].1 % 6;
    assert_eq!(log.draws[0].0, SITE_ISLET_TERRAIN);
    let heights: Vec<u32> = log.draws[1..21].iter().map(|&(s, _)| s).collect();
    assert_eq!(heights, [SITE_ISLET_HEIGHTS; 4].concat());
    assert!(log.draws.len() > 21);
    let (mut land, mut water) = (0, 0);
    for y in 3..7 {
        for x in 3..7 {
            let (now, was) = (cells.get(x, y, 0).unwrap(), before.get(x, y, 0).unwrap());
            if now.land() == u32::from(WATER) {
                water += 1;
                assert_eq!(now.tile_data & (KEPT | OBSTACLES), KEPT, "{x} {y}");
            } else {
                land += 1;
                assert_eq!(now.land(), terrain);
                let marks = if (x, y) == (5, 5) {
                    was.tile_data & (KEPT | OBSTACLES)
                } else {
                    OBSTACLES
                };
                assert_eq!(now.tile_data & (KEPT | OBSTACLES), marks, "{x} {y}");
            }
        }
    }
    assert!(land > 0 && water > 0);
}

/// The islets of `gens` on `cells` with the zones 0 and 1 and `tables`' pictures: the result and
/// the draws.
fn islets_of(
    cells: &mut Cells,
    gens: &[ZoneGenerator],
    tables: &GeneratorTables,
) -> (Result<(), PaintFault>, Log) {
    let zones = [zone(0), zone(1)];
    let pictures = Pictures::new(tables);
    let input = PlanInput {
        zones: &zones,
        connections: &[],
        gens,
        objects: &mut no_objects(),
        pictures: &pictures,
        cosmetic: true,
        guards: no_guards(),
    };
    let mut log = Log::default();
    let painted = islets(&mut Stream::new(5, &mut log), &input, cells);
    (painted, log)
}

// covers: RMG-CON-007
/// A water zone of 20 × 20 inside a sea of no zone, after a land zone (skipped): its box's
/// costs reset and flooded from the cells around it (those 0), islets drawn while a cell of
/// columns and rows 3 to W − 5 lies 20 or more from the edge, each a centre, a half-size of 3 to
/// 6, a terrain and its painter's draws, its centre flooded again (0). None is left that far.
#[test]
fn a_water_zones_islets_are_drawn_until_no_cell_is_far() {
    let (mut cells, water) = sea(24, [2, 2, 22, 22]);
    let land = ZoneGenerator {
        zone: 1,
        ..generator(1)
    };
    let (painted, log) = islets_of(&mut cells, &[land, water], &tables());
    assert_eq!(painted, Ok(()));
    let centres: Vec<usize> = (log.draws.iter().enumerate())
        .filter(|(_, d)| d.0 == SITE_ISLET_CENTRE)
        .map(|(i, _)| i)
        .collect();
    assert!(!centres.is_empty());
    for &i in &centres {
        assert_eq!(log.draws[i + 1].0, SITE_ISLET_RADIUS);
        assert_eq!(log.draws[i + 2].0, SITE_ISLET_TERRAIN);
        assert_eq!(log.draws[i + 3].0, SITE_ISLET_HEIGHTS[0]);
    }
    let far = far(&cells);
    assert_eq!(far[1][1], 0);
    assert_eq!(cells.get(1, 1, 0).unwrap().zone >> 24, 0);
    // No cell of columns and rows 3 to 19 is left far; each centre is the zone's only cells at 0
    // (the seeds around it are of no zone).
    assert!((3..20).all(|y| (3..20).all(|x| far[y][x] < FAR)));
    let zeros = far[2..22]
        .iter()
        .flat_map(|row| &row[2..22])
        .filter(|&&d| d == 0);
    assert_eq!(zeros.count(), centres.len());
    assert!(cells.cells().iter().any(|c| c.tile_data & OBSTACLES != 0));
}

// covers: RMG-CON-007, RMG-CON-008
/// A water zone too small for an islet draws nothing; an islet whose terrain has no pictures is
/// the painter's fault (`rand() % 0` in the original), returned after the last zone.
#[test]
fn a_small_water_zone_draws_nothing_and_a_painters_fault_is_returned() {
    let (mut cells, water) = sea(16, [1, 1, 15, 15]);
    let (painted, log) = islets_of(&mut cells, &[water], &tables());
    assert_eq!(painted, Ok(()));
    assert!(log.draws.is_empty());
    let mut bare = tables();
    for set in &mut bare.picture_sets[..8] {
        set.pictures.clear();
    }
    let (mut cells, water) = sea(24, [2, 2, 22, 22]);
    let (painted, log) = islets_of(&mut cells, &[water, water], &bare);
    let terrain = log.draws[2].1 % 6;
    assert_eq!(
        painted,
        Err(PaintFault::NoPicture {
            terrain: terrain as u8,
            shape: 0
        })
    );
    assert!(
        log.draws
            .iter()
            .filter(|d| d.0 == SITE_ISLET_CENTRE)
            .count()
            > 1
    );
}
