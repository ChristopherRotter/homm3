//! Step 6's connections on small maps (RMG-CON-013, -014, -020): the crossings, the boat reach,
//! the two passes' bookkeeping, a land passage's candidates, carves and guard, and the
//! shipyards' boat reach of pass 2. The corpus's against the phase oracle are `h3-pack-rmg`'s.

use alloc::vec;
use alloc::vec::Vec;

use super::*;
use crate::cells::OBSTACLES;
use crate::guards::{GuardCreature, GuardInput, SITE_GUARD_CREATURE};
use crate::layout::ZoneGenerator;
use crate::objects::Objects;
use crate::objects::tests::{line, settings};
use crate::painter::Pictures;
use crate::painter::tests::tables;
use crate::part::{ObjectTemplate, Zone};
use crate::plan::tests::{Log, bits, connection, drawn, generator, zone};
use crate::settings::MapFormat;
use crate::template::KeptConnection;

/// What the connections run on.
struct World {
    zones: Vec<Zone>,
    connections: Vec<KeptConnection>,
    gens: Vec<ZoneGenerator>,
    objects: Objects,
    creatures: Vec<GuardCreature>,
    cells: Cells,
    road_targets: Vec<[i32; 3]>,
    /// RMG-QUIRK-19's flag.
    repeated: bool,
    /// RMG-QUIRK-26's flag.
    shared: bool,
}

impl World {
    /// Zones 0 to `n − 1` (neutral monsters allowed), a land zone generator each on level 0,
    /// `connections`, a monster line of creature 0 (its anchor its entrance) and a neutral
    /// creature 0 of AI value 100 and counts 10 to 30, on `cells`.
    fn new(n: usize, connections: Vec<KeptConnection>, cells: Cells) -> Self {
        let monster = ObjectTemplate {
            actions: 1,
            ..line(54, 0)
        };
        let mut creatures = vec![GuardCreature::UNUSED; 145];
        creatures[0] = GuardCreature {
            faction: -1,
            level: 0,
            ai_value: 100,
            counts: [10, 30],
        };
        Self {
            zones: (0..n as i32)
                .map(|k| Zone {
                    neutral_monsters: true,
                    ..zone(k)
                })
                .collect(),
            connections,
            gens: (0..n).map(generator).collect(),
            objects: Objects::new(&[monster, line(5, 0)], &settings(), MapFormat::SoD, n),
            creatures,
            cells,
            road_targets: Vec::new(),
            repeated: true,
            shared: true,
        }
    }

    /// `f` on the connections' work drawn from `seed`: its result and the draws.
    fn run<R>(
        &mut self,
        seed: u32,
        f: impl FnOnce(&mut Connect<'_, '_, '_, Log>) -> R,
    ) -> (R, Vec<(u32, u32)>) {
        let pictures = Pictures::new(&tables());
        let mut input = PlanInput {
            zones: &self.zones,
            connections: &self.connections,
            gens: &self.gens,
            objects: &mut self.objects,
            road_targets: &mut self.road_targets,
            pictures: &pictures,
            cosmetic: true,
            guards: GuardInput {
                creatures: &self.creatures,
                format: MapFormat::SoD,
                strength: 3,
                roe_guard_walk: true,
            },
            shipyard_sites_repeated: self.repeated,
            underground_wide_walled: true,
            monolith_channels_shared: self.shared,
        };
        let mut log = Log::default();
        let r = f(&mut Connect::new(
            &mut Stream::new(seed, &mut log),
            &mut input,
            &mut self.cells,
        ));
        (r, log.draws)
    }

    /// The whole connections drawn from `seed`.
    fn connect(&mut self, seed: u32) -> (Connected, Vec<(u32, u32)>) {
        self.run(seed, |c| {
            c.pass_1();
            c.pass_2();
            core::mem::take(&mut c.out)
        })
    }

    /// A monster of the first line on (x, y, 0): its cell holds an object and stays passable.
    fn occupy(&mut self, x: i32, y: i32) {
        self.objects.add(&mut self.cells, 0, [x, y, 0]);
    }
}

/// A shipyard line: three blocked cells on its anchor's row, the middle one its entrance.
fn shipyard() -> ObjectTemplate {
    ObjectTemplate {
        passability: line(87, 0).passability & !0b110,
        actions: 0b010,
        width: 3,
        ..line(87, 0)
    }
}

/// The first connection by land between zone generators 0 and 1.
fn land(c: &mut Connect<'_, '_, '_, Log>) -> [bool; 2] {
    let connections = c.input.connections;
    c.by_land(0, 1, &connections[0].row)
}

/// (x, y, 0) entered by zone `zone`'s flood at `cost` from its neighbour `step`.
fn cross(cells: &mut Cells, [x, y]: [i32; 2], cost: u32, step: u32, zone: i32) {
    cells.get_mut(x, y, 0).unwrap().enter_from(cost, step, zone);
}

/// (x, y, 0)'s own zone's cost `cost`, entered from `from`.
fn near(cells: &mut Cells, [x, y]: [i32; 2], cost: u32, from: [i32; 2]) {
    let c = cells.get_mut(x, y, 0).unwrap();
    c.movement = (c.movement & 0xFFFF_0000) | cost;
    c.previous = [from[0], from[1], 0];
}

/// Every cell of level 0 with `bit`.
fn mark_all(cells: &mut Cells, bit: u32) {
    let (w, h, _) = cells.size();
    for y in 0..h {
        for x in 0..w {
            cells.get_mut(x, y, 0).unwrap().tile_data |= bit;
        }
    }
}

/// A connection with value `value`, wide or not, a Border Guard or not.
fn row(a: usize, b: usize, value: i32, wide: bool, border_guard: bool) -> KeptConnection {
    let mut c = connection(a, b, wide);
    c.row.value = value;
    c.row.border_guard = border_guard;
    c
}

/// Zones 0 (columns 0 to 2) and 1 (3 to 5) on 6 × 6, the cells of column 2 in rows 0 to 2
/// entered by zone 1's flood from the east (costs 12, 11, 11), those of column 3 by zone 0's
/// from the west; rows 1 and 2 walk back to cost 0 in one step on both sides; every cell an
/// obstacle.
fn two_zones() -> Cells {
    let mut cells = drawn(&[&["000111"; 6]]);
    mark_all(&mut cells, OBSTACLES);
    for (y, cost) in [(0, 12), (1, 11), (2, 11)] {
        cross(&mut cells, [2, y], cost, 0, 1);
        cross(&mut cells, [3, y], cost, 4, 0);
    }
    for y in [1, 2] {
        near(&mut cells, [2, y], 1, [1, y]);
        near(&mut cells, [1, y], 0, [-1, -1]);
        near(&mut cells, [3, y], 1, [4, y]);
        near(&mut cells, [4, y], 0, [-1, -1]);
    }
    cells
}

// covers: RMG-CON-013
/// The crossings: a cell a flood entered from another zone, neither water nor rock, passable,
/// whose cell back is not water and of another zone; level by level, row by row.
#[test]
fn the_crossings_are_cells_entered_from_another_zones_land() {
    let mut cells = drawn(&[&["0011", "0011", "0~1r", "00w1"]]);
    // (1, 0) and (1, 1) entered from (2, ·) of zone 1; (2, 1) from (1, 1) of zone 0.
    cross(&mut cells, [1, 0], 11, 0, 1);
    cross(&mut cells, [1, 1], 11, 0, 1);
    cross(&mut cells, [2, 1], 11, 4, 0);
    // Not passable; water; rock; back in its own zone; back on water (of zone 0).
    cross(&mut cells, [0, 0], 11, 0, 1);
    cells.get_mut(0, 0, 0).unwrap().tile_data &= !FREE;
    cross(&mut cells, [1, 2], 11, 0, 1);
    cross(&mut cells, [3, 2], 11, 4, 0);
    cross(&mut cells, [3, 1], 11, 4, 1);
    cross(&mut cells, [1, 3], 11, 0, 1);
    assert_eq!(crossings(&cells), [[1, 0, 0], [1, 1, 0], [2, 1, 0]]);
}

// covers: RMG-CON-013
/// The boat reach: from the seed over the four orthogonal steps, kept-free water entered,
/// land and kept-free cells marked, water not kept free left; a cell marked is not entered
/// again.
#[test]
fn the_boat_reach_floods_kept_free_water_and_marks_its_shore() {
    let mut cells = drawn(&[&["~~.~", "~.~~", "~~~~", "...."]]);
    for [x, y] in [[0, 0], [1, 0], [0, 1], [0, 2], [1, 2], [2, 2]] {
        cells.get_mut(x, y, 0).unwrap().tile_data |= KEPT;
    }
    boat_reach(&mut cells, [0, 0, 0]);
    assert_eq!(bits(&cells, 0, BOAT), ["###.", "##..", "###.", "###."]);
}

// covers: RMG-CON-014, RMG-CON-018, RMG-CON-020
/// A land connection: its candidates the crossings of least cost (11 of 12, 11, 11), one passage
/// for 3 crossings, P drawn among them and Q its cell back, both carved back to cost 0 with
/// their 3 × 3s in their zones cleared, P into zone 0's passages and Q into zone 1's; a value of
/// 3000 guards it with 2000 at normal strength, the side drawn puts the monster at P (0) or Q
/// (1). Both ends are made: zone 1 makes none, pass 2 nothing.
#[test]
fn a_land_connection_carves_a_passage_and_guards_it() {
    let mut sides = [false; 2];
    for seed in 0..8 {
        let mut world = World::new(2, vec![row(0, 1, 3_000, false, false)], two_zones());
        let (out, draws) = world.connect(seed);
        let sites: Vec<u32> = draws.iter().map(|d| d.0).collect();
        assert_eq!(
            sites,
            [
                SITE_PASSAGE_CELL,
                SITE_PASSAGE_GUARD_SIDE,
                SITE_GUARD_CREATURE,
                crate::guards::SITE_GUARD_COUNT_PLUS,
                crate::guards::SITE_GUARD_COUNT_MINUS
            ]
        );
        let y = 1 + draws[0].1 as i32 % 2;
        assert_eq!(out.passages, [vec![[2, y]], vec![[3, y]]]);
        let side = draws[1].1 & 1;
        sides[side as usize] = true;
        let at = [2 + side as i32, y, 0];
        let count = 20 + draws[3].1 as i32 % 6 - draws[4].1 as i32 % 6;
        assert_eq!(
            out.monsters,
            [Monster {
                object: 0,
                serial: 1,
                count
            }]
        );
        assert_eq!(world.objects.placed()[0].position, at);
        assert_eq!(world.cells.objects_at(at[0], at[1], 0), [0]);
        let mut kept = vec![alloc::string::String::from("......"); 6];
        kept[y as usize] = alloc::string::String::from("..##..");
        assert_eq!(bits(&world.cells, 0, KEPT), kept);
        let cleared = |r: i32| {
            if (y - 1..=y + 1).contains(&r) {
                "#....#"
            } else {
                "######"
            }
        };
        assert_eq!(
            bits(&world.cells, 0, OBSTACLES),
            (0..6).map(cleared).collect::<Vec<_>>()
        );
    }
    assert_eq!(sides, [true, true]);
}

// covers: RMG-CON-014, RMG-CON-018
/// A wide connection is not guarded (no side drawn); with a Border Guard (none placed: E14
/// T4d2) the carves keep their 3 × 3s and a guard monster still stands; a value under the
/// guard's least stands none.
#[test]
fn a_wide_or_flagged_connection_carves_as_its_flags_say() {
    let mut world = World::new(2, vec![row(0, 1, 3_000, true, false)], two_zones());
    let (out, draws) = world.connect(1);
    assert_eq!(draws.len(), 1);
    assert!(out.monsters.is_empty());
    let mut world = World::new(2, vec![row(0, 1, 1_500, false, false)], two_zones());
    assert_eq!(world.connect(1).1.len(), 1);
    let mut world = World::new(2, vec![row(0, 1, 3_000, false, true)], two_zones());
    let (out, draws) = world.connect(1);
    assert_eq!(draws.len(), 5);
    assert_eq!(out.monsters.len(), 1);
    let y = 1 + draws[0].1 as i32 % 2;
    let mut obstacles = vec![alloc::string::String::from("######"); 6];
    obstacles[y as usize] = alloc::string::String::from("##..##");
    assert_eq!(bits(&world.cells, 0, OBSTACLES), obstacles);
}

// covers: RMG-CON-014
/// The candidates: crossings of this zone the other's flood entered with P and Q empty, counted
/// at any cost; costs above 100 or the least so far left out, a lower one emptying them. A least
/// cost of 1 with no guard and no Border Guard makes the connection without a passage, a guard
/// value or a Border Guard carves it; 41 crossings make two passages.
#[test]
fn a_land_passage_takes_the_least_cost_crossings() {
    let mut cells = drawn(&[&["0001111"; 7]]);
    for (y, cost) in [
        (0, 101),
        (1, 20),
        (2, 30),
        (3, 15),
        (4, 15),
        (5, 15),
        (6, 15),
    ] {
        cross(&mut cells, [2, y], cost, 0, 1);
    }
    // Zone 2's flood entered (2, 6); zone 0's entered (3, 3).
    cross(&mut cells, [2, 6], 15, 0, 2);
    cross(&mut cells, [3, 3], 15, 4, 0);
    let mut world = World::new(3, vec![row(0, 1, 0, true, false)], cells);
    world.occupy(2, 4);
    world.occupy(3, 5);
    let ((ends, crossed), draws) = world.run(4, |c| (land(c), c.crossings.len()));
    assert_eq!((ends, crossed), ([true, true], 8));
    assert_eq!(draws.len(), 1);
    assert_eq!(
        world
            .run(4, |c| core::mem::take(&mut c.out))
            .0
            .passages
            .len(),
        3
    );
    let (out, _) = world.connect(4);
    assert_eq!(out.passages[0], [[2, 3]]);
    // A least cost of 1: made at once without a guard; carved with one or a Border Guard.
    for (value, flag, draws) in [(0, false, 0), (3_000, false, 5), (0, true, 1)] {
        let mut cells = drawn(&[&["01", "01"]]);
        cross(&mut cells, [0, 0], 1, 0, 1);
        let mut world = World::new(2, vec![row(0, 1, value, value == 0, flag)], cells);
        let (out, log) = world.connect(2);
        assert_eq!(
            (log.len(), out.passages[0].len()),
            (draws, usize::from(draws > 0))
        );
    }
    // 41 crossings: two passages, the second among the 40 left.
    let row41 = alloc::format!("01{}", ".".repeat(39));
    let rows = vec![row41.as_str(); 41];
    let mut cells = drawn(&[rows.as_slice()]);
    for y in 0..41 {
        cross(&mut cells, [0, y], 10, 0, 1);
    }
    let mut world = World::new(2, vec![row(0, 1, 0, true, false)], cells);
    let (out, draws) = world.connect(3);
    let first = draws[0].1 as i32 % 41;
    let mut left: Vec<i32> = (0..41).filter(|&y| y != first).collect();
    let second = left.remove(draws[1].1 as usize % 40);
    assert_eq!(out.passages[0], [[0, first], [0, second]]);
    assert_eq!(out.passages[1], [[1, first], [1, second]]);
}

// covers: RMG-CON-014
/// Refused by land: zone generators on two levels, one of water, or no crossing between them.
/// The guard: none on a cell holding an object, the zone generator by the cell's zone byte (one
/// past them its owner's), none when no creature fits.
#[test]
fn a_land_connection_is_refused_off_one_level_and_the_guard_needs_room() {
    let mut world = World::new(2, vec![row(0, 1, 3_000, false, false)], two_zones());
    world.gens[1].position[2] = 1;
    assert_eq!(world.run(0, land).0, [false; 2]);
    world.gens[1].position[2] = 0;
    world.gens[1].terrain = i32::from(WATER);
    assert_eq!(world.run(0, land).0, [false; 2]);
    world.gens[1].terrain = 0;
    world.gens[0].terrain = i32::from(WATER);
    assert_eq!(world.run(0, land).0, [false; 2]);
    let mut world = World::new(
        2,
        vec![row(0, 1, 3_000, false, false)],
        drawn(&[&["01", "01"]]),
    );
    assert_eq!(world.run(0, land).0, [false; 2]);
    // The guard.
    let mut world = World::new(2, Vec::new(), drawn(&[&["0145", "....", "....", "...."]]));
    world.occupy(0, 0);
    let (_, draws) = world.run(0, |c| c.guard([0, 0, 0], 0, 2_000));
    assert!(draws.is_empty());
    let (_, draws) = world.run(0, |c| c.guard([2, 0, 0], 1, 2_000));
    assert_eq!(draws.len(), 3);
    assert_eq!(world.objects.placed()[1].position, [2, 0, 0]);
    let (_, draws) = world.run(0, |c| c.guard([3, 0, 0], 1, 50_000));
    assert!(draws.is_empty());
}

// covers: RMG-CON-013
/// The passes: pass 1 skips water zone generators and the entries made (both ends of a
/// connection created), a zone whose id is past the zone generators (its entries left) and an
/// end whose other has no entry back (nothing marked there); pass 2 per zone with an entry
/// left from the first: monoliths for a land end (both ends made, a zone's connection to itself
/// twice), nothing for a water end; a zone with every entry made skipped. Every way tried on one
/// level but by land draws a shipyard prototype and finds no site (11 draws).
#[test]
fn the_passes_make_each_connection_once_from_either_end() {
    let connections = vec![
        connection(0, 1, false),
        connection(0, 0, false),
        connection(0, 2, false),
        connection(0, 3, false),
        KeptConnection {
            created: true,
            ..connection(1, 0, false)
        },
    ];
    let mut world = World::new(4, connections, drawn(&[&["0123", "....", "....", "...."]]));
    world.zones[3].id = 9;
    world.gens[2].terrain = i32::from(WATER);
    let monolith = |class| ObjectTemplate {
        actions: 1,
        ..line(class, 0)
    };
    world.objects = Objects::new(
        &[
            monolith(45),
            monolith(45),
            monolith(43),
            monolith(44),
            shipyard(),
        ],
        &settings(),
        MapFormat::SoD,
        4,
    );
    let ((first, second), draws) = world.run(0, |c| {
        c.pass_1();
        let first = c.made.clone();
        c.pass_2();
        (first, c.made.clone())
    });
    assert_eq!(draws.len(), 11);
    assert!(draws.iter().all(|d| d.0 == SITE_SHIPYARD_PROTOTYPE));
    assert_eq!(
        first,
        [
            vec![false, false, false, false, false, true],
            vec![false, true],
            vec![false],
            vec![false]
        ]
    );
    assert_eq!(
        second,
        [
            vec![true, true, true, false, false, true],
            vec![true, true],
            vec![false],
            vec![true]
        ]
    );
    assert_eq!(world.objects.take_fault(), None);
}

// covers: RMG-CON-013
/// Pass 2's boat reach: the level's marks cleared, then from each shipyard of the zone the first
/// side cell on the map that is water floods; another zone's shipyard is passed, one without
/// water beside floods nothing. The shipyard tried in each pass draws its prototype and finds no
/// site. Without monolith prototypes the index is taken `mod 0` (a fault; the zones on two
/// levels, nothing drawn).
#[test]
fn pass_2_floods_the_boat_reach_from_the_zones_shipyards() {
    let mut cells = drawn(&[&[
        "~0000001", "~0000000", "~0000000", "00000000", "00000000", "00000000", "00000000",
        "00000000",
    ]]);
    for y in 0..3 {
        cells.get_mut(0, y, 0).unwrap().tile_data |= KEPT;
    }
    cells.get_mut(7, 3, 0).unwrap().tile_data |= BOAT;
    let mut world = World::new(2, vec![connection(0, 1, false)], cells);
    world.objects = Objects::new(&[line(87, 0)], &settings(), MapFormat::SoD, 2);
    for at in [[4, 1, 0], [2, 2, 0], [3, 0, 0], [6, 7, 0], [7, 0, 0]] {
        world.objects.add(&mut world.cells, 0, at);
    }
    let (out, draws) = world.connect(0);
    assert!(out.monsters.is_empty());
    assert_eq!(draws.len(), 3);
    assert!(draws.iter().all(|d| d.0 == SITE_SHIPYARD_PROTOTYPE));
    let mut boat = vec!["##......", "##......", "##......", "#......."];
    boat.extend(["........"; 4]);
    assert_eq!(bits(&world.cells, 0, BOAT), boat);
    assert_eq!(world.objects.take_fault(), Some(ObjectFault::NoMonolith));
    let mut world = World::new(
        2,
        vec![connection(0, 1, false)],
        drawn(&[&["01", "01"], &["01", "01"]]),
    );
    world.gens[1].position[2] = 1;
    let (_, draws) = world.connect(0);
    assert!(draws.is_empty());
    assert_eq!(world.objects.take_fault(), Some(ObjectFault::NoMonolith));
}

/// A gate line: one blocked cell, its anchor, the entrance.
fn gate() -> ObjectTemplate {
    ObjectTemplate {
        actions: 1,
        ..line(103, 0)
    }
}

/// The objects of a world of `n` zones: its monster (0), a shipyard (1) and a gate (2).
fn ways(n: usize) -> Objects {
    let monster = ObjectTemplate {
        actions: 1,
        ..line(54, 0)
    };
    Objects::new(
        &[monster, shipyard(), gate()],
        &settings(),
        MapFormat::SoD,
        n,
    )
}

/// The cells (x, y, 0) made water and kept free.
fn sea(cells: &mut Cells, at: &[[i32; 2]]) {
    for &[x, y] in at {
        let c = cells.get_mut(x, y, 0).unwrap();
        c.tile = u32::from(WATER);
        c.tile_data |= KEPT;
    }
}

/// The first connection by a shipyard (gates) between zone generators 0 and 1.
fn yard(c: &mut Connect<'_, '_, '_, Log>) -> [bool; 2] {
    let connections = c.input.connections;
    c.by_shipyard(0, 1, &connections[0].row)
}

fn gates(c: &mut Connect<'_, '_, '_, Log>) -> [bool; 2] {
    let connections = c.input.connections;
    c.by_gate(0, 1, &connections[0].row)
}

// covers: RMG-CON-015
/// A shipyard's site: the row below on the map; the 3 × 2 block land, without an entrance, free,
/// not rock; the first of the four sides that is water and kept free (a water side not kept
/// passed); the cell beyond on the anchor's row on the map and not water. A block cell off the
/// map is read unchecked (a fault).
#[test]
fn a_shipyard_site_is_a_land_block_beside_kept_water() {
    let mut cells = drawn(&[&[
        "~0000~..", "~0000~..", "00000000", "00000000", "........", "........", "........",
        "........",
    ]]);
    for [x, y] in [[0, 0], [0, 1], [5, 1]] {
        cells.get_mut(x, y, 0).unwrap().tile_data |= KEPT;
    }
    let fits = |cells: &Cells, x, y| site_fits(cells, [x, y, 0]);
    // Water on the left, (4, 0) beyond; on the right below, (1, 0) beyond.
    assert_eq!(fits(&cells, 3, 0), Ok(true));
    assert_eq!(fits(&cells, 4, 0), Ok(true));
    // No water beside; water in the block; the row below off the map; the block off the map.
    assert_eq!(fits(&cells, 4, 2), Ok(false));
    assert_eq!(fits(&cells, 2, 1), Ok(false));
    assert_eq!(fits(&cells, 3, 7), Ok(false));
    assert_eq!(fits(&cells, 1, 2), Err(ObjectFault::Outside));
    // A block cell an entrance, not free, rock.
    for (bit, rock) in [(ENTRANCE, false), (0, false), (0, true)] {
        let mut cells = cells.clone();
        let c = cells.get_mut(2, 1, 0).unwrap();
        c.tile_data = (c.tile_data | bit) & if bit == 0 && !rock { !FREE } else { !0 };
        if rock {
            c.tile = u32::from(ROCK);
        }
        assert_eq!(fits(&cells, 3, 0), Ok(false));
    }
    // Water beyond; beyond off the map.
    cells.get_mut(1, 0, 0).unwrap().tile = u32::from(WATER);
    assert_eq!(fits(&cells, 4, 0), Ok(false));
    sea(&mut cells, &[[3, 2]]);
    assert_eq!(fits(&cells, 2, 2), Ok(false));
}

// covers: RMG-CON-015
/// A shipyard is refused off one level (nothing drawn); its prototype is drawn first, `mod 0`
/// without one (a fault); then over the zone's box only cells the other's flood entered count: a
/// water cell and one on the last row are passed, a land cell where none fits gives no site
/// (refused); a cell already reached by boat makes it at once, at this end. A block read off the
/// map (a shipyard narrower than 3) is a fault.
#[test]
fn a_shipyard_is_refused_without_a_site_and_made_by_boat() {
    let rows: &[&str] = &["0w11", "0011", "0011", "0011"];
    let mut world = World::new(2, vec![connection(0, 1, false)], drawn(&[rows, rows]));
    world.gens[0].bounds = [0, 0, 4, 4];
    world.gens[1].position[2] = 1;
    assert_eq!(world.run(0, yard), ([false, false], vec![]));
    world.gens[1].position[2] = 0;
    let (made, draws) = world.run(0, yard);
    assert_eq!(made, [false, false]);
    assert_eq!(draws.len(), 1);
    assert_eq!(draws[0].0, SITE_SHIPYARD_PROTOTYPE);
    assert_eq!(world.objects.take_fault(), Some(ObjectFault::NoShipyard));
    world.objects = ways(2);
    for at in [[1, 0], [1, 3], [0, 1]] {
        cross(&mut world.cells, at, 11, 4, 1);
    }
    let (made, draws) = world.run(0, yard);
    assert_eq!((made, draws.len()), ([false, false], 1));
    cross(&mut world.cells, [0, 2], 11, 4, 1);
    world.cells.get_mut(0, 2, 0).unwrap().tile_data |= BOAT;
    let (made, draws) = world.run(0, yard);
    assert_eq!((made, draws.len()), ([true, false], 1));
    assert!(world.objects.placed().is_empty());
    assert_eq!(world.objects.take_fault(), None);
    let mut world = World::new(2, vec![connection(0, 1, false)], drawn(&[&["0000"; 4]]));
    world.gens[0].bounds = [0, 0, 4, 4];
    world.objects = Objects::new(&[line(87, 0)], &settings(), MapFormat::SoD, 2);
    cross(&mut world.cells, [0, 0], 11, 4, 1);
    assert_eq!(world.run(0, yard).0, [false, false]);
    assert_eq!(world.objects.take_fault(), Some(ObjectFault::Outside));
}

/// Zone 1 in columns 0 and 1 of rows 0 to 2, zone 0 beside it, on 8 × 8.
const SHORE: [&str; 8] = [
    "11000000", "11000000", "11000000", "........", "........", "........", "........", "........",
];

// covers: RMG-CON-015, RMG-CON-018, RMG-CON-020
/// A shipyard on the shore of zone 1's water: the sites of the cells zone 1's flood entered,
/// (4, 0) found from (2, 0) and from (3, 0), (4, 1) from (2, 1): one drawn among three (each
/// once with RMG-QUIRK-19 off: among two). Placed there, its entrance a road target, the row below
/// kept free (but a pending Border Guard's cell) and into zone 0's passages, the boat reach
/// flooded from the water on its left; worth 2000, the guard monster below its entrance (the
/// Border Guard placing none, E14 T4d2); wide, none. Made at this end.
#[test]
fn a_shipyard_takes_a_shore_site_and_guards_its_entrance() {
    let mut differ = false;
    for seed in 0..8 {
        let mut ys = [0; 2];
        for (i, repeated) in [true, false].into_iter().enumerate() {
            let mut cells = drawn(&[&SHORE]);
            sea(
                &mut cells,
                &[[0, 0], [1, 0], [0, 1], [1, 1], [0, 2], [1, 2]],
            );
            for at in [[2, 0], [3, 0], [2, 1]] {
                cross(&mut cells, at, 11, 4, 1);
            }
            cells.get_mut(2, 1, 0).unwrap().pending_guard = 1;
            let mut world = World::new(2, vec![row(0, 1, 3_000, false, i == 0)], cells);
            world.gens[0].bounds = [2, 0, 8, 3];
            world.gens[1].terrain = i32::from(WATER);
            world.objects = ways(2);
            world.repeated = repeated;
            let ((made, out), draws) = world.run(seed, |c| (yard(c), core::mem::take(&mut c.out)));
            assert_eq!(made, [true, false]);
            let sites: Vec<u32> = draws.iter().map(|d| d.0).collect();
            assert_eq!(
                sites,
                [
                    SITE_SHIPYARD_PROTOTYPE,
                    SITE_SHIPYARD_SITE,
                    SITE_GUARD_CREATURE,
                    crate::guards::SITE_GUARD_COUNT_PLUS,
                    crate::guards::SITE_GUARD_COUNT_MINUS
                ]
            );
            let r = draws[1].1 as usize;
            let y = if repeated {
                [0, 0, 1][r % 3]
            } else {
                [0, 1][r % 2]
            };
            ys[i] = y;
            let placed = world.objects.placed();
            assert_eq!((placed[0].class, placed[0].position), (87, [4, y, 0]));
            assert_eq!(placed[1].position, [3, y + 1, 0]);
            assert_eq!(out.monsters.len(), 1);
            assert_eq!(world.road_targets, [[3, y, 0]]);
            assert_eq!(
                out.passages,
                [vec![[2, y + 1], [3, y + 1], [4, y + 1]], vec![]]
            );
            let mut kept = vec![alloc::string::String::from("##......"); 3];
            kept.resize(8, "........".into());
            kept[y as usize].replace_range(3..4, "#");
            let below = if y == 0 { "##.##..." } else { "#####..." };
            kept[y as usize + 1] = below.into();
            assert_eq!(bits(&world.cells, 0, KEPT), kept);
            let mut boat = vec!["###....."; 3];
            boat.extend(["##......", "........", "........", "........", "........"]);
            assert_eq!(bits(&world.cells, 0, BOAT), boat);
            assert_eq!(world.objects.take_fault(), None);
        }
        differ |= ys[0] != ys[1];
    }
    assert!(differ);
    let mut cells = drawn(&[&SHORE]);
    sea(
        &mut cells,
        &[[0, 0], [1, 0], [0, 1], [1, 1], [0, 2], [1, 2]],
    );
    cross(&mut cells, [2, 0], 11, 4, 1);
    let mut world = World::new(2, vec![row(0, 1, 3_000, true, false)], cells);
    world.gens[0].bounds = [2, 0, 8, 3];
    world.objects = ways(2);
    let (made, draws) = world.run(0, yard);
    assert_eq!((made, draws.len()), ([true, false], 2));
    assert_eq!(world.objects.placed().len(), 1);
}

// covers: RMG-CON-016
/// Gates are refused on one level, from water or for boxes apart (nothing drawn); their
/// prototype is drawn first, `mod 0` without one (a fault); no position fitting on both levels:
/// refused.
#[test]
fn gates_are_refused_without_two_levels_or_a_common_position() {
    let rows: &[&str] = &["000", "000", "000"];
    let mut world = World::new(2, vec![connection(0, 1, false)], drawn(&[rows, rows]));
    world.gens[0].bounds = [0, 0, 3, 3];
    world.gens[1].bounds = [0, 0, 3, 3];
    assert_eq!(world.run(0, gates), ([false, false], vec![]));
    world.gens[1].position[2] = 1;
    world.gens[0].terrain = i32::from(WATER);
    assert_eq!(world.run(0, gates), ([false, false], vec![]));
    world.gens[0].terrain = 0;
    for bounds in [[3, 0, 5, 3], [0, 3, 3, 5]] {
        world.gens[1].bounds = bounds;
        assert_eq!(world.run(0, gates), ([false, false], vec![]));
    }
    world.gens[1].bounds = [0, 0, 3, 3];
    let (made, draws) = world.run(0, gates);
    assert_eq!((made, draws.len()), ([false, false], 1));
    assert_eq!(draws[0].0, SITE_GATE_PROTOTYPE);
    assert_eq!(world.objects.take_fault(), Some(ObjectFault::NoGate));
    world.objects = ways(2);
    let (made, draws) = world.run(0, gates);
    assert_eq!((made, draws.len()), ([false, false], 1));
    assert_eq!(world.objects.take_fault(), None);
}

/// Zone 0 on level 0 and zone 1 on level 1 of 5 × 5, both boxes overlapping in x 1..4; every
/// object distance 0 but (2, 1) and (3, 2), summing 7, and (1, 3) and (2, 3), summing 20 and 60,
/// where no gate fits (not free on level 1, rock on level 0); (3, 0) on level 0 and (1, 2) on
/// level 1 of zone 2.
fn two_levels() -> Cells {
    let mut cells = drawn(&[&["00000"; 5], &["11111"; 5]]);
    for z in 0..2 {
        for y in 0..5 {
            for x in 0..5 {
                cells.get_mut(x, y, z).unwrap().zone &= 0xFFFF_0000;
            }
        }
    }
    for (x, y, z, d) in [
        (2, 1, 0, 3),
        (2, 1, 1, 4),
        (3, 2, 0, 5),
        (3, 2, 1, 2),
        (1, 3, 0, 20),
        (2, 3, 0, 30),
        (2, 3, 1, 30),
    ] {
        cells.get_mut(x, y, z).unwrap().zone |= d;
    }
    cells.get_mut(1, 3, 1).unwrap().tile_data &= !FREE;
    cells.get_mut(2, 3, 0).unwrap().tile = u32::from(ROCK);
    for (x, y, z) in [(3, 0, 0), (1, 2, 1)] {
        let c = cells.get_mut(x, y, z).unwrap();
        c.zone = (c.zone & 0xFF00_FFFF) | (2 << 16);
    }
    cells.get_mut(3, 3, 1).unwrap().pending_guard = 1;
    cells
}

// covers: RMG-CON-016, RMG-CON-018, RMG-CON-020
/// Gates: the positions of the overlap in both zones fitting on both levels of greatest distance
/// sum, (2, 1) and (3, 2) (ties kept, a greater sum emptying them); one drawn, a gate there on
/// each level, its entrance into both zones' passages, the cell below it kept free on each level
/// (but a pending Border Guard's), worth 2000 a guard monster below each (the Border Guard placing
/// none, E14 T4d2), zone 0's first. Made at both ends. A gate line without an entrance writes
/// below its offsets off the map (a fault).
#[test]
fn gates_take_the_farthest_common_position_and_guard_both_levels() {
    let mut picked = [false; 2];
    for seed in 0..8 {
        let mut world = World::new(
            2,
            vec![row(0, 1, 3_000, false, seed % 2 == 0)],
            two_levels(),
        );
        world.gens[0].bounds = [0, 0, 4, 5];
        world.gens[1].bounds = [1, 0, 5, 5];
        world.gens[1].position[2] = 1;
        world.objects = ways(3);
        let ((made, out), draws) = world.run(seed, |c| (gates(c), core::mem::take(&mut c.out)));
        assert_eq!(made, [true, true]);
        let sites: Vec<u32> = draws.iter().map(|d| d.0).collect();
        let monster = [
            SITE_GUARD_CREATURE,
            crate::guards::SITE_GUARD_COUNT_PLUS,
            crate::guards::SITE_GUARD_COUNT_MINUS,
        ];
        let mut expected = vec![SITE_GATE_PROTOTYPE, SITE_GATE_SITE];
        expected.extend(monster);
        expected.extend(monster);
        assert_eq!(sites, expected);
        let i = draws[1].1 as usize % 2;
        picked[i] = true;
        let [x, y] = [[2, 1], [3, 2]][i];
        let placed = world.objects.placed();
        let at: Vec<[i32; 3]> = placed.iter().map(|o| o.position).collect();
        assert_eq!(at, [[x, y, 0], [x, y, 1], [x, y + 1, 0], [x, y + 1, 1]]);
        assert_eq!((placed[0].class, placed[1].class), (103, 103));
        assert_eq!(out.passages, [vec![[x, y]], vec![[x, y]]]);
        assert_eq!(out.monsters.len(), 2);
        for z in 0..2 {
            let kept = world.cells.get(x, y + 1, z).unwrap().tile_data & KEPT != 0;
            assert_eq!(kept, (x, z) != (3, 1));
        }
        assert_eq!(world.objects.take_fault(), None);
    }
    assert_eq!(picked, [true, true]);
    let mut world = World::new(2, vec![connection(0, 1, false)], two_levels());
    world.gens[0].bounds = [0, 0, 4, 5];
    world.gens[1].bounds = [1, 0, 5, 5];
    world.gens[1].position[2] = 1;
    world.objects = Objects::new(&[line(103, 0)], &settings(), MapFormat::SoD, 3);
    let ((made, out), _) = world.run(0, |c| (gates(c), core::mem::take(&mut c.out)));
    assert_eq!(made, [true, true]);
    assert_eq!(out.monsters, []);
    assert_eq!(world.objects.take_fault(), Some(ObjectFault::Outside));
}

/// A monolith line of class `class` and subclass `subclass`, `width` × `height`, its anchor
/// blocked and its entrance.
fn monolith(class: u32, subclass: u32, width: u8, height: u8) -> ObjectTemplate {
    ObjectTemplate {
        actions: 1,
        width,
        height,
        ..line(class, subclass)
    }
}

/// The objects of a world of `n` zones: its monster (0), then `lines`.
fn with(n: usize, lines: &[ObjectTemplate]) -> Objects {
    let monster = ObjectTemplate {
        actions: 1,
        ..line(54, 0)
    };
    let mut all = vec![monster];
    all.extend_from_slice(lines);
    Objects::new(&all, &settings(), MapFormat::SoD, n)
}

/// The first connection by monoliths between zone generators 0 and 1 with the index `index`.
fn monoliths(c: &mut Connect<'_, '_, '_, Log>, index: usize) -> usize {
    let connections = c.input.connections;
    c.by_monoliths(0, 1, &connections[0].row, index)
}

/// Each placed object of the classes 43 to 45 as (class, subclass, its anchor's zone byte).
fn placed_monoliths(world: &World) -> Vec<(u32, u32, i32)> {
    world
        .objects
        .placed()
        .iter()
        .filter(|o| (43..=45).contains(&o.class))
        .map(|o| {
            let [x, y, z] = o.position;
            let zone = world.cells.get(x, y, z).unwrap().zone_index();
            (o.class, o.subclass, zone)
        })
        .collect()
}

// covers: RMG-CON-017
/// An object placed at random in a zone: the anchors of its zone generator's box (columns 1 to
/// 3) from its least x + width − 1 (a 2 × 2: from column 2, so not (1, 1), where it would fit)
/// and y + height − 1, of its zone (not (3, 2), zone 1's) and fitting (not (3, 1), its entrance's
/// cell below zone 1's, nor row 3, the map's last): (2, 1) and (2, 2). One drawn (0x542A8F), the
/// object created and placed. A box too narrow for the footprint draws nothing.
#[test]
fn an_object_is_placed_at_random_where_it_fits_in_its_zone() {
    let map = || drawn(&[&["1000", "1000", "1001", "1000"]]);
    let mut picked = [false; 2];
    for seed in 0..8 {
        let mut world = World::new(2, Vec::new(), map());
        world.objects = with(2, &[monolith(45, 0, 2, 2)]);
        world.gens[0].bounds = [1, 0, 4, 4];
        let (found, draws) = world.run(seed, |c| c.place_in_zone(1, 0));
        assert_eq!(draws.len(), 1);
        assert_eq!(draws[0].0, SITE_ZONE_OBJECT_CELL);
        let i = draws[0].1 as usize % 2;
        picked[i] = true;
        let at = [[2, 1, 0], [2, 2, 0]][i];
        assert_eq!(found, Some((0, at)));
        assert_eq!(world.objects.placed()[0].position, at);
        assert_eq!(world.objects.prototypes()[1].used, 1);
    }
    assert_eq!(picked, [true, true]);
    let mut world = World::new(2, Vec::new(), map());
    world.objects = with(2, &[monolith(45, 0, 2, 2)]);
    world.gens[1].bounds = [0, 0, 1, 4];
    let (found, draws) = world.run(0, |c| c.place_in_zone(1, 1));
    assert_eq!((found, draws), (None, Vec::new()));
    assert_eq!(world.objects.prototypes()[1].used, 0);
}

/// Zones 0 (columns 0 to 2) and 1 (3 to 5) on 6 × 6, each zone generator's box its columns.
fn two_boxes(connection: KeptConnection) -> World {
    let mut world = World::new(2, vec![connection], drawn(&[&["000111"; 6]]));
    world.gens[0].bounds = [0, 0, 3, 6];
    world.gens[1].bounds = [3, 0, 6, 6];
    world
}

// covers: RMG-CON-017, RMG-CON-018, RMG-CON-020
/// Two-way monoliths: index 0 below the two-way prototypes' count takes the first for both ends,
/// index 1 the second; an entrance in zone 0, worth 2000 a guard monster below it (the Border
/// Guard placing none, E14 T4d2), then one in zone 1 with its own; both listed in +0x14D0 and
/// each anchor into its zone generator's passages; one index taken. Unguarded without a guard
/// value; a zone without room gets none, the other its own.
#[test]
fn two_way_monoliths_join_the_zones_and_guard_their_entrances() {
    let lines = [monolith(45, 0, 1, 1), monolith(45, 1, 1, 1)];
    for (index, flag) in [(0, false), (1, true)] {
        let mut world = two_boxes(row(0, 1, 3_000, false, flag));
        world.objects = with(2, &lines);
        let ((used, out), draws) = world.run(u32::from(flag), |c| {
            (monoliths(c, index), core::mem::take(&mut c.out))
        });
        assert_eq!(used, 1);
        let monster = [
            SITE_GUARD_CREATURE,
            crate::guards::SITE_GUARD_COUNT_PLUS,
            crate::guards::SITE_GUARD_COUNT_MINUS,
        ];
        let mut expected = vec![SITE_ZONE_OBJECT_CELL];
        expected.extend(monster);
        expected.push(SITE_ZONE_OBJECT_CELL);
        expected.extend(monster);
        assert_eq!(draws.iter().map(|d| d.0).collect::<Vec<_>>(), expected);
        let placed = world.objects.placed();
        assert_eq!(out.two_way, [0, 2]);
        assert_eq!(out.one_way, []);
        let [a, b] = [placed[0].position, placed[2].position];
        assert_eq!(
            (placed[0].subclass, placed[2].subclass),
            (index as u32, index as u32)
        );
        assert_eq!(placed[1].position, [a[0], a[1] + 1, 0]);
        assert_eq!(placed[3].position, [b[0], b[1] + 1, 0]);
        assert_eq!(out.passages, [vec![[a[0], a[1]]], vec![[b[0], b[1]]]]);
        assert_eq!(out.monsters.len(), 2);
        assert_eq!(placed_monoliths(&world)[1].2, 1);
        assert_eq!(world.objects.prototypes()[1 + index].used, 2);
    }
    let mut world = two_boxes(connection(0, 1, false));
    world.objects = with(2, &lines);
    world.gens[1].bounds = ZoneGenerator::NO_BOUNDS;
    let ((used, out), draws) = world.run(0, |c| (monoliths(c, 0), core::mem::take(&mut c.out)));
    assert_eq!(used, 1);
    assert_eq!(draws.len(), 1);
    assert_eq!(out.two_way, [0]);
    assert_eq!(out.monsters, []);
    assert!(out.passages[1].is_empty());
    assert_eq!(world.objects.take_fault(), None);
}

/// One two-way line, then the one-way entrances and exits of subclasses 0 and 1, all 1 × 1.
fn channels() -> [ObjectTemplate; 5] {
    [
        monolith(45, 0, 1, 1),
        monolith(43, 0, 1, 1),
        monolith(43, 1, 1, 1),
        monolith(44, 0, 1, 1),
        monolith(44, 1, 1, 1),
    ]
}

// covers: RMG-CON-017
/// One-way monoliths (RMG-QUIRK-26), index i past the n45 = 1 two-way prototype: channel j =
/// i − 1 for an entrance in each zone, then an exit in zone 0 and one in zone 1, all four in
/// +0x14C0, each into its zone's passages. On, one channel: each entrance leads to both exits,
/// its own zone's among them; one index taken. Off, channel 0 with a next one free: zone 1's
/// entrance and zone 0's exit take channel 1, so each entrance's only exit is in the other zone;
/// two indices taken. At the last channel (j = 1) off keeps one.
#[test]
fn one_way_monoliths_share_a_channel_or_take_two_by_the_flag() {
    for (shared, index, used, subclasses) in [
        (true, 1, 1, [0, 0, 0, 0]),
        (false, 1, 2, [0, 1, 1, 0]),
        (false, 2, 1, [1, 1, 1, 1]),
    ] {
        let mut world = two_boxes(connection(0, 1, false));
        world.objects = with(2, &channels());
        world.shared = shared;
        let ((taken, out), draws) =
            world.run(0, |c| (monoliths(c, index), core::mem::take(&mut c.out)));
        assert_eq!(taken, used);
        assert_eq!(draws.len(), 4);
        assert!(draws.iter().all(|d| d.0 == SITE_ZONE_OBJECT_CELL));
        assert_eq!(out.one_way, [0, 1, 2, 3]);
        assert_eq!(out.two_way, []);
        let found = placed_monoliths(&world);
        let expected: Vec<(u32, u32, i32)> = [43, 43, 44, 44]
            .into_iter()
            .zip(subclasses)
            .zip([0, 1, 0, 1])
            .map(|((class, subclass), zone)| (class, subclass, zone))
            .collect();
        assert_eq!(found, expected);
        assert_eq!(
            out.passages.iter().map(Vec::len).collect::<Vec<_>>(),
            [2, 2]
        );
        // Where each entrance leads: its channel's exits' zones.
        for &(_, channel, from) in &found[..2] {
            let to: Vec<i32> = found[2..]
                .iter()
                .filter(|e| e.1 == channel)
                .map(|e| e.2)
                .collect();
            if used == 1 {
                assert_eq!(to, [0, 1]);
            } else {
                assert_eq!(to, [1 - from]);
            }
        }
    }
}

// covers: RMG-CON-013, RMG-CON-017
/// Pass 2's monolith index by RMG-QUIRK-26's flag: three one-way channels and no two-way, zone
/// 0's two connections by monoliths (each way tried before draws a shipyard prototype and finds
/// no site). On: channels 0 then 1. Off: the first takes channels 0 and 1 (the index moves by
/// 2), the second the last, 2, alone.
#[test]
fn pass_2_moves_the_monolith_index_by_the_channels_taken() {
    for (shared, first, second) in [(true, [0, 0, 0, 0], [1; 4]), (false, [0, 1, 1, 0], [2; 4])] {
        let connections = vec![connection(0, 1, false), connection(0, 2, false)];
        let mut world = World::new(3, connections, drawn(&[&["000111222"; 9]]));
        for (k, g) in world.gens.iter_mut().enumerate() {
            let x = 3 * k as i32;
            g.bounds = [x, 0, x + 3, 9];
        }
        let mut lines = vec![shipyard()];
        lines.extend((0..3).map(|s| monolith(43, s, 1, 1)));
        lines.extend((0..3).map(|s| monolith(44, s, 1, 1)));
        world.objects = with(3, &lines);
        world.shared = shared;
        let (out, draws) = world.connect(0);
        let sites: Vec<u32> = draws.iter().map(|d| d.0).collect();
        let mut expected = vec![SITE_SHIPYARD_PROTOTYPE; 4];
        for _ in 0..2 {
            expected.push(SITE_SHIPYARD_PROTOTYPE);
            expected.extend([SITE_ZONE_OBJECT_CELL; 4]);
        }
        assert_eq!(sites, expected);
        let channels: Vec<u32> = placed_monoliths(&world).iter().map(|m| m.1).collect();
        assert_eq!(channels[..4], first);
        assert_eq!(channels[4..], second);
        assert_eq!(out.one_way.len(), 8);
        assert_eq!(world.objects.take_fault(), None);
    }
}

// covers: RMG-CON-017
/// The faults: without any monolith prototype the original reads class 43's empty vector, with
/// fewer exits than entrances past the exits' vector; nothing placed, one index taken.
#[test]
fn monoliths_without_their_lines_are_faults() {
    for (lines, fault) in [
        (Vec::new(), ObjectFault::NoMonolith),
        (vec![monolith(43, 0, 1, 1)], ObjectFault::Prototype),
    ] {
        let mut world = two_boxes(connection(0, 1, false));
        world.objects = with(2, &lines);
        let ((used, out), draws) = world.run(0, |c| (monoliths(c, 0), core::mem::take(&mut c.out)));
        assert_eq!(used, 1);
        assert_eq!(draws, []);
        assert_eq!(
            out,
            Connected {
                passages: vec![Vec::new(); 2],
                ..Connected::default()
            }
        );
        assert_eq!(world.objects.take_fault(), Some(fault));
    }
}
