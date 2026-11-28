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
            pictures: &pictures,
            cosmetic: true,
            guards: GuardInput {
                creatures: &self.creatures,
                format: MapFormat::SoD,
                strength: 3,
                roe_guard_walk: true,
            },
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
/// twice), nothing for a water end; a zone with every entry made skipped. Nothing is drawn.
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
        &[monolith(45), monolith(45), monolith(43)],
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
    assert!(draws.is_empty());
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
/// side cell (its column on the map) that is water floods; another zone's shipyard is passed, one
/// on the last row reads below the map (a fault, the side skipped). Without monolith prototypes
/// the index is taken `mod 0` (a fault).
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
    assert!(draws.is_empty() && out.monsters.is_empty());
    let mut boat = vec!["##......", "##......", "##......", "#......."];
    boat.extend(["........"; 4]);
    assert_eq!(bits(&world.cells, 0, BOAT), boat);
    assert_eq!(world.objects.take_fault(), Some(ObjectFault::ShipyardSide));
    let mut world = World::new(2, vec![connection(0, 1, false)], drawn(&[&["01", "01"]]));
    world.connect(0);
    assert_eq!(world.objects.take_fault(), Some(ObjectFault::NoMonolith));
}
