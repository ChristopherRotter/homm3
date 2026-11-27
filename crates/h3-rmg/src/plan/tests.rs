//! Step 6's plan on small maps (RMG-CON-001 to -006): the marks, the ray cast, the segments and
//! branches, the lanes' orientations, the zone walls, the cells of no zone and the fault. The
//! corpus's lanes against the phase oracle are `h3-pack-rmg`'s.

use alloc::string::String;
use alloc::vec::Vec;

use super::*;
use crate::objects::Shape;
use crate::painter::tests::tables;
use crate::part::{Connection, Strength, ZoneKind};

#[derive(Debug, Default)]
pub(super) struct Log {
    pub(super) draws: Vec<(u32, u32)>,
}

impl Trace for Log {
    fn draw(&mut self, site: u32, value: u32) {
        self.draws.push((site, value));
    }

    fn step_end(&mut self, _step: u8, _state: crate::generate::StepState<'_>) {}
}

const DIRT: u32 = 0;

/// A map of `side` × `side` on `levels` levels, every cell dirt of no zone, passable (bit 25),
/// without bits 26 and 27.
pub(super) fn land(side: i32, levels: i32) -> Cells {
    let mut cells = Cells::new(side, levels);
    for z in 0..levels {
        for y in 0..side {
            for x in 0..side {
                let c = cells.get_mut(x, y, z).unwrap();
                c.tile = DIRT;
                c.tile_data = FREE;
            }
        }
    }
    cells
}

/// The map of `rows` per level: `.` dirt of no zone, `~` water of no zone, `r` rock of no zone,
/// a digit dirt of that zone, `w` water of zone 0; each passable, without bits 26 and 27.
pub(super) fn drawn(levels: &[&[&str]]) -> Cells {
    let side = levels[0].len() as i32;
    let mut cells = land(side, levels.len() as i32);
    for (z, rows) in levels.iter().enumerate() {
        for (y, row) in rows.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let c = cells.get_mut(x as i32, y as i32, z as i32).unwrap();
                let (tile, zone) = match ch {
                    '.' => (DIRT, 0xFF),
                    '~' => (u32::from(WATER), 0xFF),
                    'r' => (u32::from(ROCK), 0xFF),
                    'w' => (u32::from(WATER), 0),
                    d => (DIRT, d.to_digit(10).unwrap()),
                };
                c.tile = tile;
                c.zone = (c.zone & 0xFF00_FFFF) | (zone << 16);
            }
        }
    }
    cells
}

/// Level `z`'s cells with `bit` as rows of `#`, the others `.`.
pub(super) fn bits(cells: &Cells, z: i32, bit: u32) -> Vec<String> {
    let (w, h, _) = cells.size();
    (0..h)
        .map(|y| {
            (0..w)
                .map(|x| {
                    if cells.get(x, y, z).unwrap().tile_data & bit != 0 {
                        '#'
                    } else {
                        '.'
                    }
                })
                .collect()
        })
        .collect()
}

/// A one-cell object on (x, y, z): blocked (its bit 25 cleared), or an entrance.
fn object(cells: &mut Cells, [x, y, z]: [i32; 3], entrance: bool) {
    let shape = Shape {
        passable: 0,
        entrances: u64::from(entrance),
        has_entrance: entrance,
        entrance: [0, 0],
        width: 1,
        height: 1,
        sea: false,
        class: 0,
        subclass: 0,
    };
    cells.place(0, &shape, [x, y, z]).unwrap();
}

pub(super) fn pend(cells: &mut Cells, x: i32, y: i32, z: i32) {
    cells.get_mut(x, y, z).unwrap().pending_guard = 1;
}

fn tile_data(cells: &Cells, x: i32, y: i32, z: i32) -> u32 {
    cells.get(x, y, z).unwrap().tile_data
}

pub(super) fn zone(id: i32) -> Zone {
    Zone {
        id,
        kind: ZoneKind::Treasure,
        base_size: 10,
        humans: [0, 8],
        players: [0, 8],
        ownership: -1,
        player_towns: [0; 4],
        neutral_towns: [0; 4],
        same_type: false,
        town_types: 0,
        mines: [0; 7],
        mine_density: [0; 7],
        terrain_match: false,
        terrains: 0,
        strength: Strength::None,
        monster_match: false,
        neutral_monsters: false,
        monster_towns: 0,
        treasure: [[0; 3]; 3],
    }
}

pub(super) fn generator(zone: usize) -> ZoneGenerator {
    ZoneGenerator {
        zone,
        town_type: -1,
        town_type_2: -1,
        terrain: 0,
        position: [0, 0, 0],
        size: 10,
        bounds: ZoneGenerator::NO_BOUNDS,
        town: None,
    }
}

fn connection(a: usize, b: usize, wide: bool) -> KeptConnection {
    KeptConnection {
        zones: [a, b],
        row: Connection {
            zones: [a as i32, b as i32],
            value: 0,
            wide,
            border_guard: false,
            humans: [0, 8],
            players: [0, 8],
        },
        created: false,
    }
}

// covers: RMG-CON-003
/// A free 3 × 3 is clamped to the map and skips a cell with a pending Border Guard.
#[test]
fn a_free_3x3_is_clamped_and_skips_pending_guards() {
    let mut cells = land(4, 1);
    for x in 0..4 {
        cells.get_mut(x, 0, 0).unwrap().tile_data |= OBSTACLES;
    }
    pend(&mut cells, 1, 1, 0);
    mark_free(&mut cells, 0, 0, 0);
    assert_eq!(bits(&cells, 0, KEPT), ["##..", "#...", "....", "...."]);
    assert_eq!(bits(&cells, 0, OBSTACLES), ["..##", "....", "....", "...."]);
}

// covers: RMG-CON-004, RMG-CON-006
/// An obstacle takes bit 26 and loses 27 unless its guard is pending; of its 3 × 3 only the
/// passable land cells that are no entrance and have no pending guard lose bit 27.
#[test]
fn an_obstacle_clears_bit_27_of_its_passable_empty_land_neighbours() {
    for guard in [false, true] {
        let mut cells = drawn(&[&["....", ".r~.", "....", "...."]]);
        for y in 0..4 {
            for x in 0..4 {
                cells.get_mut(x, y, 0).unwrap().tile_data |= KEPT;
            }
        }
        // (0, 0) an entrance, (1, 0) not passable, (2, 0) pending, (1, 1) rock, (2, 1) water.
        cells.get_mut(0, 0, 0).unwrap().tile_data |= ENTRANCE;
        cells.get_mut(1, 0, 0).unwrap().tile_data &= !FREE;
        pend(&mut cells, 2, 0, 0);
        if guard {
            pend(&mut cells, 1, 0, 0);
            cells.get_mut(1, 0, 0).unwrap().tile_data |= FREE;
        }
        mark_obstacle(&mut cells, 1, 0, 0);
        let centre = tile_data(&cells, 1, 0, 0);
        assert_eq!(centre & OBSTACLES != 0, !guard);
        assert_eq!(centre & KEPT == 0, !guard);
        let top = if guard { "####" } else { "#.##" };
        assert_eq!(bits(&cells, 0, KEPT), [top, ".###", "####", "####"]);
    }
}

// covers: RMG-CON-004
/// The ray cast walks past its target until it would leave the interior, along its major axis
/// (|dy| on ties) with diagonal steps, either sign.
#[test]
fn a_ray_runs_to_the_interiors_edge() {
    let cells = land(10, 1);
    assert_eq!(cast(&cells, [2, 5], [6, 5], 0), [8, 5]);
    assert_eq!(cast(&cells, [5, 5], [5, 1], 0), [5, 1]);
    assert_eq!(cast(&cells, [2, 2], [5, 5], 0), [8, 8]);
    assert_eq!(cast(&cells, [6, 2], [2, 4], 0), [1, 5]);
    assert_eq!(cast(&cells, [1, 5], [0, 5], 0), [1, 5]);
}

// covers: RMG-CON-004
/// From its third step the ray stops before a cell whose 3 × 3 holds a free cell (bit 27); one
/// near its first two steps is passed.
#[test]
fn a_ray_stops_before_a_free_3x3_from_its_third_step() {
    let mut cells = land(10, 1);
    cells.get_mut(7, 5, 0).unwrap().tile_data |= KEPT;
    assert_eq!(cast(&cells, [2, 5], [6, 5], 0), [5, 5]);
    let mut cells = land(10, 1);
    cells.get_mut(3, 4, 0).unwrap().tile_data |= KEPT;
    assert_eq!(cast(&cells, [2, 5], [6, 5], 0), [8, 5]);
}

fn lane<'s, 't, 'c>(
    stream: &'s mut Stream<'t, Log>,
    cells: &'c mut Cells,
) -> Lane<'s, 't, 'c, Log> {
    Lane {
        stream,
        cells,
        z: 0,
        segments: Vec::new(),
        branches: VecDeque::new(),
    }
}

// covers: RMG-CON-003
/// A segment whose midpoint is an end marks its first end's 3 × 3 free when that end lies on
/// the map; one of length 1 whose midpoint is a third cell splits without a draw.
#[test]
fn a_segment_at_its_end_marks_its_first_end_free() {
    let mut log = Log::default();
    let mut stream = Stream::new(1, &mut log);
    let mut cells = land(6, 1);
    let mut l = lane(&mut stream, &mut cells);
    l.split([3, 3], [3, 3]);
    l.split([-1, 0], [0, 0]);
    l.split([1, 0], [0, 1]);
    assert_eq!(l.segments, [[[0, 1], [1, 1]], [[1, 1], [1, 0]]]);
    assert!(l.branches.is_empty());
    assert_eq!(
        bits(&cells, 0, KEPT),
        ["......", "......", "..###.", "..###.", "..###.", "......"]
    );
    assert!(log.draws.is_empty());
}

// covers: RMG-CON-003
/// A segment of length L above 1 moves its midpoint along the normal by (rand() % L − L / 2) / L
/// of it; from 8 on with its midpoint on the map it queues the two branches, ± the normal.
#[test]
fn a_long_segment_is_displaced_and_branches() {
    let mut log = Log::default();
    let mut stream = Stream::new(7, &mut log);
    let mut cells = land(40, 1);
    let mut l = lane(&mut stream, &mut cells);
    l.split([2, 2], [6, 2]);
    l.split([0, 20], [16, 20]);
    l.split([-30, -30], [-14, -30]);
    let segments = l.segments.clone();
    let branches: Vec<_> = l.branches.iter().copied().collect();
    let r: Vec<i32> = log.draws.iter().map(|&(_, v)| v as i32).collect();
    assert!(log.draws.iter().all(|&(site, _)| site == SITE_LANE_JITTER));
    assert_eq!(r.len(), 3);
    let short = [4, 2 + (r[0] % 4 - 2)];
    let long = [8, 20 + (r[1] % 16 - 8)];
    let off = [-21, -29 + (r[2] % 16 - 8)];
    assert_eq!(
        segments,
        [
            [[6, 2], short],
            [short, [2, 2]],
            [[16, 20], long],
            [long, [0, 20]],
            [[-14, -30], off],
            [off, [-30, -30]],
        ]
    );
    assert_eq!(
        branches,
        [
            [long, [long[0], long[1] + 16]],
            [long, [long[0], long[1] - 16]]
        ]
    );
}

// covers: RMG-CON-003, RMG-CON-004
/// With the stack empty the first branch is cast; its segment is drawn when it reaches 5 cells
/// or more (its end then marked free), not when shorter.
#[test]
fn a_branch_is_drawn_from_five_cells() {
    let mut log = Log::default();
    let mut stream = Stream::new(3, &mut log);
    let mut cells = land(12, 1);
    let mut l = lane(&mut stream, &mut cells);
    l.branches.push_back([[5, 5], [5, 9]]);
    l.branches.push_back([[1, 1], [0, 1]]);
    l.draw();
    assert!(l.segments.is_empty() && l.branches.is_empty());
    // The first cast ends at (5, 10), 5 cells on: its end's 3 × 3 is free; the second at its
    // start: (1, 1)'s is not.
    for (x, y) in around(12, 12, 5, 10).chain([(1, 1)]) {
        assert_eq!(tile_data(&cells, x, y, 0) & KEPT != 0, y > 1, "{x} {y}");
    }
    assert!(!log.draws.is_empty());
}

// covers: RMG-CON-002
/// A lane's four lines: the diagonal, the vertical middle, the anti-diagonal, the horizontal
/// middle (W / 2 and H / 2 truncating).
#[test]
fn a_lane_takes_one_of_four_lines() {
    assert_eq!(line(0, 9, 7), [[0, 0], [8, 6]]);
    assert_eq!(line(1, 9, 7), [[4, 0], [4, 6]]);
    assert_eq!(line(2, 9, 7), [[8, 0], [0, 6]]);
    assert_eq!(line(3, 9, 7), [[0, 3], [8, 3]]);
}

// covers: RMG-CON-002, RMG-CON-003, RMG-CON-004
/// The lanes draw per level one orientation, then their displacements; every cell ends either
/// free (bit 27) or an obstacle (bit 26) or neither, never both, and each level has both.
#[test]
fn the_lanes_draw_one_orientation_per_level() {
    let mut log = Log::default();
    let mut cells = land(16, 2);
    lanes(&mut Stream::new(0, &mut log), &mut cells);
    let orientations: Vec<usize> = log
        .draws
        .iter()
        .enumerate()
        .filter(|&(_, &(site, _))| site == SITE_LANE_ORIENTATION)
        .map(|(k, _)| k)
        .collect();
    assert_eq!(orientations[0], 0);
    assert_eq!(orientations.len(), 2);
    assert!(
        log.draws
            .iter()
            .all(|&(site, _)| site == SITE_LANE_ORIENTATION || site == SITE_LANE_JITTER)
    );
    for z in 0..2 {
        let free = bits(&cells, z, KEPT).concat();
        let obstacles = bits(&cells, z, OBSTACLES).concat();
        assert!(free.contains('#') && obstacles.contains('#'));
        assert!(
            free.chars()
                .zip(obstacles.chars())
                .all(|(f, o)| f == '.' || o == '.')
        );
    }
}

// covers: RMG-CON-002, RMG-CON-004
/// The lanes' marks: an object's cell kept free, an empty cell an obstacle, a pending guard's
/// cell untouched; then water and rock kept free (but a pending guard's), the obstacles marked
/// (an obstacle's passable land neighbour loses bit 27).
#[test]
fn the_lanes_free_objects_water_and_rock_and_skip_pending_guards() {
    let mut rows = ["........."; 9];
    rows[1] = ".~.r.r...";
    let mut cells = drawn(&[&rows]);
    object(&mut cells, [1, 0, 0], false);
    object(&mut cells, [2, 0, 0], false);
    pend(&mut cells, 2, 0, 0);
    pend(&mut cells, 4, 0, 0);
    pend(&mut cells, 5, 1, 0);
    let before = [2, 4].map(|x| tile_data(&cells, x, 0, 0));
    let rock = tile_data(&cells, 5, 1, 0);
    let mut log = Log::default();
    lanes(&mut Stream::new(5, &mut log), &mut cells);
    assert_eq!(tile_data(&cells, 1, 0, 0) & (KEPT | OBSTACLES), KEPT);
    assert_eq!([2, 4].map(|x| tile_data(&cells, x, 0, 0)), before);
    assert_eq!(tile_data(&cells, 1, 1, 0) & (KEPT | OBSTACLES), KEPT);
    assert_eq!(tile_data(&cells, 3, 1, 0) & (KEPT | OBSTACLES), KEPT);
    assert_eq!(tile_data(&cells, 5, 1, 0), rock);
    // Every cell is either free or an obstacle; an obstacle has no bit 27.
    for t in cells.cells().iter().map(|c| c.tile_data) {
        assert_ne!(t & (KEPT | OBSTACLES), KEPT | OBSTACLES);
    }
}

fn walls_of(cells: &mut Cells, zones: usize, connections: &[KeptConnection]) {
    let zones: Vec<Zone> = (0..zones as i32).map(zone).collect();
    let gens: Vec<ZoneGenerator> = (0..zones.len()).map(generator).collect();
    walls(
        &PlanInput {
            zones: &zones,
            connections,
            gens: &gens,
            pictures: &Pictures::new(&tables()),
            cosmetic: true,
        },
        cells,
    );
}

// covers: RMG-CON-005
/// The walls: a zone cell is walled beside water of no zone, a zone it has no connection to or
/// one whose connection is not wide, and on level 1 beside any other zone; not beside land of
/// no zone or along a wide connection on level 0. Water of a zone and cells of no zone are not
/// walled.
#[test]
fn zones_are_walled_except_along_wide_connections_on_level_0() {
    let mut cells = drawn(&[
        &["0011", "0011", "2200", "22~."],
        &["0011", "0011", "0000", "0000"],
    ]);
    let connections = [connection(0, 1, true), connection(2, 0, false)];
    walls_of(&mut cells, 3, &connections);
    assert_eq!(bits(&cells, 0, OBSTACLES), ["....", "###.", "####", ".#.."]);
    assert_eq!(bits(&cells, 1, OBSTACLES), [".##.", ".###", ".###", "...."]);
    let mut cells = drawn(&[&["w0..", "0w0.", "....", "...."]]);
    walls_of(&mut cells, 1, &[]);
    assert_eq!(bits(&cells, 0, OBSTACLES), ["....", "....", "....", "...."]);
}

// covers: RMG-CON-005
/// A walled cell takes bit 26 holding an object, not with a pending guard; the cells of its
/// 3 × 3 lose bit 27 unless they hold an object or a pending guard; a zone of a connection to
/// itself is walled from the others.
#[test]
fn a_walled_cells_3x3_loses_bit_27_but_objects_and_pending_guards() {
    let mut cells = drawn(&[&["01..", "0...", "....", "...."]]);
    for y in 0..4 {
        for x in 0..4 {
            cells.get_mut(x, y, 0).unwrap().tile_data |= KEPT;
        }
    }
    object(&mut cells, [0, 0, 0], false);
    object(&mut cells, [1, 1, 0], false);
    pend(&mut cells, 0, 1, 0);
    pend(&mut cells, 2, 1, 0);
    let pending = tile_data(&cells, 0, 1, 0);
    walls_of(&mut cells, 2, &[connection(0, 0, true)]);
    assert_eq!(bits(&cells, 0, OBSTACLES), ["##..", "....", "....", "...."]);
    assert_eq!(tile_data(&cells, 0, 1, 0), pending);
    assert_eq!(bits(&cells, 0, KEPT), ["...#", "####", "..##", "####"]);
}

// covers: RMG-CON-001, RMG-CON-005
/// A zone byte past the zone generators (the original's unchecked read, RMG-QUIRK-4; step 2
/// writes none) leaves its cell unwalled; the plan runs the lanes, the walls and the cells of no
/// zone in turn.
#[test]
fn a_zone_byte_past_the_generators_is_left_unwalled() {
    let mut cells = drawn(&[&["05", "~0"]]);
    walls_of(&mut cells, 1, &[]);
    assert_eq!(bits(&cells, 0, OBSTACLES), ["#.", ".#"]);
    let mut cells = drawn(&[&["0~.~", "....", "....", "...."]]);
    let zones = [zone(0)];
    let gens = [generator(0)];
    let input = PlanInput {
        zones: &zones,
        connections: &[],
        gens: &gens,
        pictures: &Pictures::new(&tables()),
        cosmetic: true,
    };
    let mut log = Log::default();
    assert_eq!(
        plan(&mut Stream::new(0, &mut log), &input, &mut cells),
        Ok(())
    );
    assert_eq!(log.draws[0].0, SITE_LANE_ORIENTATION);
    // Water is kept free by the lanes; zone 0's cell beside water of no zone is walled, and that
    // water loses bit 27 with the wall's 3 × 3.
    assert_eq!(tile_data(&cells, 3, 0, 0) & (KEPT | OBSTACLES), KEPT);
    assert_eq!(tile_data(&cells, 0, 0, 0) & (KEPT | OBSTACLES), OBSTACLES);
    assert_eq!(tile_data(&cells, 1, 0, 0) & (KEPT | OBSTACLES), 0);
}

// covers: RMG-CON-006
/// The cells of no zone: a passable empty land cell that is not yet an obstacle becomes one;
/// an obstacle already, one not passable, rock, an entrance, one holding an object, a zone's
/// and water are left.
#[test]
fn passable_empty_land_of_no_zone_becomes_an_obstacle() {
    let mut cells = drawn(&[&[".....", ".r.~.", "..0..", ".....", "....."]]);
    for y in 0..5 {
        for x in 0..5 {
            cells.get_mut(x, y, 0).unwrap().tile_data |= KEPT;
        }
    }
    // (0, 0) an obstacle already; (2, 0) not passable; (4, 0) an entrance; (0, 2) holds an
    // object (its bit 25 set again).
    cells.get_mut(0, 0, 0).unwrap().tile_data |= OBSTACLES;
    cells.get_mut(2, 0, 0).unwrap().tile_data &= !FREE;
    cells.get_mut(4, 0, 0).unwrap().tile_data |= ENTRANCE;
    object(&mut cells, [0, 2, 0], false);
    cells.get_mut(0, 2, 0).unwrap().tile_data |= FREE;
    no_zone(&mut cells);
    assert_eq!(
        bits(&cells, 0, OBSTACLES),
        ["##.#.", "#.#.#", ".#.##", "#####", "#####"]
    );
}
