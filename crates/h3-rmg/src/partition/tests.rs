//! Step 2's parts on small maps (RMG-ZON-010 to -013): the clip, the two line drawers, the
//! fits test's units (RMG-QUIRK-14), the memory model's choice (RMG-QUIRK-1), the whole frame and
//! the faults. The corpus's step 2 against the phase oracle is `h3-pack-rmg`'s.

use alloc::vec;
use alloc::vec::Vec;

use super::*;
use crate::part::GeneratorTables;

#[derive(Debug, Default)]
struct Log {
    draws: Vec<(u32, u32)>,
    memory: Option<ZoneMemory>,
}

impl Trace for Log {
    fn draw(&mut self, site: u32, value: u32) {
        self.draws.push((site, value));
    }

    fn step_end(&mut self, _step: u8, _state: crate::generate::StepState<'_>) {}

    fn zone_memory(&mut self, _kind: MemoryZone, _index: usize) -> Option<ZoneMemory> {
        self.memory
    }
}

fn tables() -> GeneratorTables {
    GeneratorTables {
        heroes_roe: vec![],
        heroes_expansions: vec![],
        heroes_campaign: vec![],
        map_limits: vec![],
        zone_limits: vec![],
        layout_cos: vec![10_000; 32],
        layout_sin: vec![0; 32],
        terrain_towns: vec![],
        native_terrains: vec![],
        picture_sets: Vec::new(),
        rock_pictures: Vec::new(),
        object_settings: Vec::new(),
    }
}

/// What a level works on.
struct World {
    zones: Vec<Zone>,
    connections: Vec<KeptConnection>,
    gens: Vec<ZoneGenerator>,
    cells: Cells,
    out: Partition,
}

impl World {
    fn new(side: i32) -> Self {
        Self {
            zones: Vec::new(),
            connections: Vec::new(),
            gens: Vec::new(),
            cells: Cells::new(side, 1),
            out: Partition::default(),
        }
    }

    /// A zone of template base `base` and scaled size `size` at `p` on level 0.
    fn zone(&mut self, base: i32, size: i32, p: [i32; 2]) -> usize {
        let id = self.zones.len();
        self.zones.push(water_zone(id, base, ZoneMemory::ZEROED));
        self.gens.push(ZoneGenerator {
            zone: id,
            town_type: -1,
            town_type_2: -1,
            terrain: 0,
            position: [p[0], p[1], 0],
            size,
            bounds: ZoneGenerator::NO_BOUNDS,
        });
        self.out.polygons.push(Vec::new());
        id
    }
}

macro_rules! level {
    ($world:ident, $stream:ident, $input:ident) => {
        Level {
            stream: &mut $stream,
            input: &$input,
            zones: &mut $world.zones,
            connections: &mut $world.connections,
            gens: &mut $world.gens,
            cells: &mut $world.cells,
            out: &mut $world.out,
            z: 0,
            tri: Triangulation::new(),
            fault: None,
        }
    };
}

fn input(tables: &GeneratorTables, water: Water, flags: bool) -> PartitionInput<'_> {
    PartitionInput {
        levels: 1,
        water,
        tables,
        memory_model: flags,
        partition_units: flags,
    }
}

fn marked(c: &Cells, x: i32, y: i32) -> (u8, bool) {
    let cell = c.get(x, y, 0).expect("on the map");
    (cell.zone_byte(), cell.is_land())
}

// covers: RMG-ZON-012
#[test]
fn the_clip_moves_onto_the_maps_lines_in_order() {
    // Inside: the point itself.
    assert_eq!(clip([3, 3], [50, 50], 10, 10), [3, 3]);
    // Left, onto x = 0.
    assert_eq!(clip([-5, 5], [5, 5], 10, 10), [0, 5]);
    // Top, then right (x1 − 1): two moves.
    assert_eq!(clip([15, -5], [5, 5], 10, 10), [9, 1]);
    // Bottom (y1 − 1).
    assert_eq!(clip([4, 14], [4, 0], 10, 10), [4, 9]);
    // The left move leaves y's range from inside it: a miss gives the point itself.
    assert_eq!(clip([-5, 5], [5, 30], 10, 10), [-5, 5]);
    // A top move leaving x's range: a miss too.
    assert_eq!(clip([5, -5], [30, 5], 10, 10), [5, -5]);
    // A divisor 0 skips the move: the result stays outside.
    assert_eq!(clip([-5, 5], [-5, 8], 10, 10), [-5, 5]);
    // The line, not the segment: q is never tested.
    assert_eq!(clip([-5, 5], [-4, 5], 10, 10), [0, 5]);
}

// covers: RMG-ZON-012
#[test]
fn straight_lines_are_bresenhams_from_the_lower_x() {
    let t = tables();
    let i = input(&t, Water::Normal, true);
    let mut log = Log::default();
    let mut stream = Stream::new(1, &mut log);
    let mut w = World::new(10);
    let mut level = level!(w, stream, i);
    // From (1, 4), the end of lower x, to (5, 2): x-major, the far end the index only.
    level.line([5, 2], [1, 4], 7);
    for p in [[1, 4], [2, 3], [3, 3], [4, 2]] {
        assert_eq!(marked(level.cells, p[0], p[1]), (7, true), "{p:?}");
    }
    assert_eq!(marked(level.cells, 5, 2), (7, false));
    assert_eq!(marked(level.cells, 2, 4), (0xFF, false));
    // y-major, straight down; a point line marks its end only.
    level.line([7, 0], [7, 3], 3);
    assert_eq!(
        [0, 1, 2, 3].map(|y| marked(level.cells, 7, y)),
        [(3, true), (3, true), (3, true), (3, false)]
    );
    level.line([8, 8], [8, 8], 4);
    assert_eq!(marked(level.cells, 8, 8), (4, false));
    assert_eq!(level.fault, None);
    // Off the map the original's index is unchecked: a fault, the cells on the map drawn.
    level.line([-1, 5], [3, 5], 1);
    assert_eq!(level.fault, Some(PartitionFault::Outside));
    assert_eq!(marked(level.cells, 0, 5), (1, true));
}

// covers: RMG-ZON-012
#[test]
fn jagged_lines_split_with_the_borders_draw_and_clamp_their_cells() {
    let t = tables();
    let i = input(&t, Water::Islands, true);
    let mut log = Log::default();
    let mut stream = Stream::new(7, &mut log);
    let mut w = World::new(10);
    let mut level = level!(w, stream, i);
    // From off the map: its first cell clamped to (0, 5); on the islands' level 0 no land bit.
    level.jagged([-3, 5], [9, 5], 2, 4);
    assert_eq!(marked(level.cells, 0, 5), (2, false));
    assert_eq!(level.fault, None);
    // A limit of 0 is `rand() % 0` (RMG-QUIRK-3): a fault; the first one stays.
    level.jagged([0, 0], [9, 9], 2, 0);
    level.fail(PartitionFault::Outside);
    assert_eq!(level.fault, Some(PartitionFault::Limit));
    drop(level);
    assert!(log.draws.iter().all(|&(s, _)| s == SITE_BORDER_JITTER));
    assert!(!log.draws.is_empty());
}

// covers: RMG-ZON-010
#[test]
fn the_fit_compares_template_units_by_the_flag() {
    let t = tables();
    for (units, fits) in [(true, true), (false, false)] {
        let i = input(&t, Water::Normal, units);
        let mut log = Log::default();
        let mut stream = Stream::new(1, &mut log);
        let mut w = World::new(72);
        // Base 10 in template units, 40 on the map; the temporary zone of 5 at 20 cells.
        w.zone(10, 40, [10, 10]);
        let level = level!(w, stream, i);
        // 10 × 20 against 8 × (10 + 5) = 120 (RMG-QUIRK-14), or 8 × (40 + 5) = 360.
        assert_eq!(level.fits([30, 10], 5), fits);
    }
}

// covers: RMG-ZON-010, RMG-ZON-011
#[test]
fn the_unwritten_bytes_follow_the_flag_and_the_trace() {
    let t = tables();
    let measured = ZoneMemory {
        town_types: 0x080,
        ..ZoneMemory::ZEROED
    };
    for (flag, given, kind, expected) in [
        (false, Some(measured), MemoryZone::Water, ZoneMemory::ZEROED),
        (true, Some(measured), MemoryZone::Water, measured),
        (true, None, MemoryZone::Water, ZoneMemory::WATER),
        (true, None, MemoryZone::Temporary, ZoneMemory::TEMPORARY),
    ] {
        let i = input(&t, Water::Normal, flag);
        let mut log = Log {
            memory: given,
            ..Log::default()
        };
        let mut stream = Stream::new(1, &mut log);
        let mut w = World::new(36);
        let mut level = level!(w, stream, i);
        assert_eq!(level.memory(kind, 0), expected);
    }
}

// covers: RMG-ZON-012
#[test]
fn a_cell_with_no_side_on_the_map_draws_the_whole_frame() {
    let t = tables();
    let i = input(&t, Water::Normal, true);
    let mut log = Log::default();
    let mut stream = Stream::new(1, &mut log);
    // A map of one cell: every side's two clips are that cell.
    let mut w = World::new(1);
    let g = w.zone(5, 5, [0, 0]);
    let mut level = level!(w, stream, i);
    let v = |x, y, data| Vertex { x, y, data };
    assert!(level.tri.insert(v(0, 0, Some(g))));
    level.tri.finish();
    let r = level.tri.find([0, 0]);
    level.borders(r, g, true);
    assert_eq!(level.out.polygons[g], [[0, 0]; 4]);
    assert_eq!(marked(level.cells, 0, 0), (0, false));
    drop(level);
    assert!(log.draws.is_empty());
}

// covers: RMG-ZON-013
#[test]
fn a_fills_start_clipped_off_the_map_is_a_fault() {
    let t = tables();
    let i = input(&t, Water::Normal, true);
    let mut log = Log::default();
    let mut stream = Stream::new(1, &mut log);
    let mut w = World::new(10);
    let g = w.zone(5, 5, [100_000, 5]);
    let o = w.zone(5, 5, [5, 5]);
    let mut level = level!(w, stream, i);
    assert!(level.tri.insert(Vertex {
        x: 5,
        y: 5,
        data: Some(o)
    }));
    // The ring around (5, 5) holds no vertex inside [1, 8]: no start, no fill.
    let r = level.tri.find([5, 5]);
    level.fill(g, r);
    assert_eq!(level.fault, None);
    assert_eq!(marked(level.cells, 5, 5), (0xFF, false));
    // From a corner's record (5, 5) is the deepest destination; the clip from 100 000 toward
    // it overflows its 32-bit products and leaves the map.
    let corner = level.tri.find([-200, -200]);
    level.fill(g, corner);
    assert_eq!(level.fault, Some(PartitionFault::Outside));
}

// covers: RMG-ZON-014
#[test]
fn water_zones_whose_side_lies_off_the_map_are_not_connected() {
    let t = tables();
    let i = input(&t, Water::Normal, true);
    let mut log = Log::default();
    let mut stream = Stream::new(1, &mut log);
    let mut w = World::new(10);
    // Two new zones above the map: their side is the line y = -40, whose clip (dy 0) stays
    // off it.
    let a = w.zone(5, 5, [-50, -50]);
    let b = w.zone(5, 5, [-50, -30]);
    let mut level = level!(w, stream, i);
    for (g, y) in [(a, -50), (b, -30)] {
        assert!(level.tri.insert(Vertex {
            x: -50,
            y,
            data: Some(g)
        }));
    }
    level.tri.finish();
    level.connect_water(0);
    assert!(level.connections.is_empty());
    assert_eq!(level.out.distances, [Vec::<i16>::new(), Vec::new()]);
}
