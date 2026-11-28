//! The shores' seal on small maps (RMG-CON-012): which water is sealed, its marks, the listing
//! and the painter flushed from one land to the next, a level with nothing to paint, the
//! painter's fault. The corpus's against the phase oracle is `h3-pack-rmg`'s.

use alloc::vec::Vec;

use super::*;
use crate::layout::ZoneGenerator;
use crate::objects::Shape;
use crate::painter::Pictures;
use crate::painter::tests::tables;
use crate::part::GeneratorTables;
use crate::plan::tests::{
    Log, bits, connection, drawn, generator, no_guards, no_objects, pend, zone,
};

/// Grass and snow.
const GRASS: u32 = 2;
const SNOW: u32 = 3;

/// Level 0 of five by five: zone 0's grass, zone 1's water and zone 2's snow, water of no zone
/// below them and a water cell of zone 5 beside land of no zone; every cell kept free. Zone 0's
/// flood entered (2, 0), (2, 1) and (3, 0), zone 2's (2, 2), (4, 1); zone 0's and zone 5's
/// (0, 4). An entrance on (0, 2), a pending guard on (3, 1). Level 1 water of no zone.
fn shores() -> Cells {
    let mut cells = drawn(&[
        &["00111", "00111", "22112", "~~~~~", "5.~~~"],
        &["~~~~~", "~~~~~", "~~~~~", "~~~~~", "~~~~~"],
    ]);
    for (y, row) in ["ggwww", "ggwww", "sswws", "~~~~~", "w.~~~"]
        .iter()
        .enumerate()
    {
        for (x, ch) in row.chars().enumerate() {
            let land = match ch {
                'g' => GRASS,
                's' => SNOW,
                '.' => 0,
                _ => u32::from(WATER),
            };
            let c = cells.get_mut(x as i32, y as i32, 0).unwrap();
            c.tile = land;
            c.tile_data |= KEPT;
        }
    }
    for ([x, y], zone) in [
        ([2, 0], 0),
        ([2, 1], 0),
        ([3, 0], 0),
        ([2, 2], 2),
        ([4, 1], 2),
        ([0, 4], 0),
    ] {
        let c = cells.get_mut(x, y, 0).unwrap();
        c.zone = (c.zone & 0x00FF_FFFF) | (zone << 24);
    }
    let entrance = Shape {
        passable: 0,
        entrances: 1,
        has_entrance: true,
        entrance: [0, 0],
        width: 1,
        height: 1,
        sea: false,
        class: 0,
        subclass: 0,
    };
    cells.place(0, &entrance, [0, 2, 0]).unwrap();
    pend(&mut cells, 3, 1, 0);
    cells
}

/// The seal of `cells` with zones 0 to 2, zone 1 water and connected to zone 2, the pictures of
/// `tables`: the result and the draws.
fn seal_of(cells: &mut Cells, tables: &GeneratorTables) -> (Result<(), PaintFault>, Log) {
    let zones = [zone(0), zone(1), zone(2)];
    let water = ZoneGenerator {
        terrain: i32::from(WATER),
        ..generator(1)
    };
    let gens = [generator(0), water, generator(2)];
    let pictures = Pictures::new(tables);
    let input = PlanInput {
        zones: &zones,
        connections: &[connection(1, 2, false)],
        gens: &gens,
        objects: &mut no_objects(),
        pictures: &pictures,
        cosmetic: true,
        guards: no_guards(),
    };
    let mut log = Log::default();
    let sealed = seal(&mut Stream::new(6, &mut log), &input, cells);
    (sealed, log)
}

// covers: RMG-CON-012, RMG-TER-005
/// A water cell of a zone that another zone's flood entered, with a passable land cell not an
/// obstacle in its 3 × 3, is sealed when its zone has no connection to that zone: (2, 0) by
/// zone 0's grass, then (2, 1) by zone 2's snow (zone 0's grass beside it already obstacles);
/// (3, 0) has no land beside, (2, 2) none left, (4, 1) was entered by zone 2's flood (connected),
/// (0, 4)'s zone byte is past the zone generators. The 3 × 3s become obstacles (a pending guard's
/// cell kept), the 5 × 5s lose bit 27 (an object's and a pending guard's kept); their water is
/// painted grass, then the painter flushed to snow, as listed.
#[test]
fn unconnected_shores_are_sealed_and_painted() {
    let mut cells = shores();
    let before = cells.clone();
    let (sealed, log) = seal_of(&mut cells, &tables());
    assert_eq!(sealed, Ok(()));
    assert_eq!(
        bits(&cells, 0, OBSTACLES),
        [".###.", ".##..", ".###.", ".....", "....."]
    );
    assert_eq!(
        bits(&cells, 0, KEPT),
        [".....", "...#.", "#....", ".....", "#####"]
    );
    assert_eq!(bits(&cells, 1, KEPT), bits(&before, 1, KEPT));
    // The painting as listed: grass over (2, 0), (3, 0), (2, 1), (3, 1), then snow over those
    // and (2, 2), (3, 2).
    let mut replica = before;
    let pictures = Pictures::new(&tables());
    let mut again = Log::default();
    let mut stream = Stream::new(6, &mut again);
    let mut p = Painter::new(&pictures, &mut replica, &mut stream, 0, GRASS as u8, true);
    for [x, y] in [[2, 0], [3, 0], [2, 1], [3, 1]] {
        p.add_rect(x, y, 1, 1);
    }
    p.flush(SNOW as u8);
    for [x, y] in [[2, 0], [3, 0], [2, 1], [3, 1], [2, 2], [3, 2]] {
        p.add_rect(x, y, 1, 1);
    }
    assert_eq!(p.finish(), Ok(()));
    assert_eq!(log.draws, again.draws);
    let tiles = |c: &Cells| -> Vec<u32> { c.cells().iter().map(|c| c.tile).collect() };
    assert_eq!(tiles(&cells), tiles(&replica));
    assert_eq!(cells.get(2, 2, 0).unwrap().land(), SNOW);
}

// covers: RMG-CON-012
/// A painter's fault in a seal (a land without pictures, `rand() % 0` in the original) is
/// returned after the last level.
#[test]
fn a_seals_painter_fault_is_returned() {
    let mut bare = tables();
    bare.picture_sets[GRASS as usize].pictures.clear();
    let (sealed, log) = seal_of(&mut shores(), &bare);
    assert_eq!(
        sealed,
        Err(PaintFault::NoPicture {
            terrain: GRASS as u8,
            shape: 0
        })
    );
    assert!(!log.draws.is_empty());
}
