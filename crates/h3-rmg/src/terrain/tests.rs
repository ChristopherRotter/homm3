//! Step 3's parts on small maps (RMG-TER-001 to -003): the boxes and centroids, an island's
//! coast and inside, the levels painted and the faults. The corpus's step 3 against the phase
//! oracle is `h3-pack-rmg`'s.

use alloc::vec;
use alloc::vec::Vec;

use super::*;
use crate::part::{Strength, ZoneKind};

#[derive(Debug, Default)]
struct Log {
    draws: Vec<(u32, u32)>,
}

impl Trace for Log {
    fn draw(&mut self, site: u32, value: u32) {
        self.draws.push((site, value));
    }

    fn step_end(&mut self, _step: u8, _state: crate::generate::StepState<'_>) {}
}

fn zone(id: i32) -> Zone {
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

fn generator(zone: usize, terrain: i32, position: [i32; 3], size: i32) -> ZoneGenerator {
    ZoneGenerator {
        zone,
        town_type: -1,
        town_type_2: -1,
        terrain,
        position,
        size,
        bounds: ZoneGenerator::NO_BOUNDS,
    }
}

/// A map of `levels` levels, the cells of `x0..x1` × `y0..y1` on level 0 of zone 0 (with the land
/// bit when `land`), the rest of none.
fn cells(side: i32, levels: i32, [x0, y0, x1, y1]: [i32; 4], land: bool) -> Cells {
    let mut cells = Cells::new(side, levels);
    for y in y0..y1 {
        for x in x0..x1 {
            cells.clamped_mut(x, y, 0).mark(0, land);
        }
    }
    cells
}

fn run(
    water: Water,
    tables: &GeneratorTables,
    polygons: &[Vec<[i32; 2]>],
    gens: &mut [ZoneGenerator],
    cells: &mut Cells,
) -> (Result<(), GenerateError>, Vec<(u32, u32)>) {
    let zones = [zone(0), zone(1)];
    let mut log = Log::default();
    let mut stream = Stream::new(5, &mut log);
    let input = TerrainInput {
        water,
        cosmetic: true,
        tables,
        zones: &zones,
        polygons,
    };
    let result = paint(&mut stream, &input, gens, cells);
    (result, log.draws)
}

// covers: RMG-TER-001, RMG-TER-003
/// The boxes from the zone's cells, the centre moved to their centroid; a zone without cells
/// keeps its box and centre; level 1 painted rock, level 0 water, the zone's land cells its
/// terrain (each cell one rectangle).
#[test]
fn the_boxes_centroids_and_levels_are_painted() {
    let tables = crate::painter::tests::tables();
    let mut cells = cells(12, 2, [2, 3, 7, 6], true);
    let mut gens = [
        generator(0, 2, [0, 0, 0], 10),
        generator(1, 3, [5, 5, 1], 10),
    ];
    let (result, draws) = run(Water::Normal, &tables, &[], &mut gens, &mut cells);
    assert_eq!(result, Ok(()));
    assert_eq!(gens[0].bounds, [2, 3, 7, 6]);
    assert_eq!(gens[0].position, [4, 4, 0]);
    assert_eq!(gens[1].bounds, ZoneGenerator::NO_BOUNDS);
    assert_eq!(gens[1].position, [5, 5, 1]);
    let terrain = |x, y, z| cells.get(x, y, z).map(|c| c.tile & 0x3F);
    assert_eq!(terrain(4, 4, 0), Some(2));
    assert_eq!(terrain(0, 0, 0), Some(u32::from(WATER)));
    assert_eq!(terrain(5, 5, 1), Some(u32::from(ROCK)));
    // 144 rock pictures, 144 water pictures, then two draws per land cell.
    assert_eq!(
        draws[..144]
            .iter()
            .filter(|d| d.0 == crate::painter::SITE_ROCK_PICTURE)
            .count(),
        144
    );
    assert_eq!(
        draws[144..288]
            .iter()
            .filter(|d| d.0 == crate::painter::SITE_PICTURE)
            .count(),
        144
    );
    assert_eq!(draws[288].0, crate::painter::SITE_PICTURE_FLAG);
}

// covers: RMG-TER-002
/// With islands a level-0 zone's polygon is shrunk toward its centre, its coast jittered
/// (0x53CE1D) and its inside filled: the centre and the coast land, the map's corners not.
#[test]
fn an_island_is_shrunk_jagged_and_filled() {
    let tables = crate::painter::tests::tables();
    let mut cells = cells(36, 1, [0, 0, 36, 36], false);
    let mut gens = [generator(0, 2, [17, 17, 0], 12)];
    let square = vec![[0, 0], [35, 0], [35, 35], [0, 35]];
    let (result, draws) = run(Water::Islands, &tables, &[square], &mut gens, &mut cells);
    assert_eq!(result, Ok(()));
    assert!(draws.iter().any(|d| d.0 == SITE_COAST_JITTER));
    let land = |x, y| cells.get(x, y, 0).is_some_and(|c| c.is_land());
    assert!(land(17, 17) && land(17, 10));
    assert!(!land(0, 0) && !land(35, 35));
    assert_eq!(cells.get(17, 17, 0).map(|c| c.tile & 0x3F), Some(2));
    assert_eq!(
        cells.get(0, 0, 0).map(|c| c.tile & 0x3F),
        Some(u32::from(WATER))
    );
}

// covers: RMG-TER-002
/// RMG-QUIRK-3's and -4's step 3 cases are typed errors: a coast of limit 0 (`rand() % 0`), an
/// island without a polygon.
#[test]
fn step_3s_faults_are_refused() {
    let tables = crate::painter::tests::tables();
    let square = vec![[0, 0], [35, 0], [35, 35], [0, 35]];
    for (size, polygons, fault) in [
        (1, vec![square], TerrainFault::CoastLimit),
        (12, Vec::new(), TerrainFault::NoPolygon),
    ] {
        let mut cells = cells(36, 1, [0, 0, 36, 36], false);
        let mut gens = [generator(0, 2, [17, 17, 0], size)];
        let (result, _) = run(Water::Islands, &tables, &polygons, &mut gens, &mut cells);
        assert_eq!(result, Err(GenerateError::Terrain(fault)));
    }
    // A shape the zone's terrain lacks: the painter's.
    let mut tables = tables;
    tables.picture_sets[2].pictures.truncate(4);
    let mut cells = cells(12, 1, [2, 3, 7, 6], true);
    let mut gens = [generator(0, 2, [4, 4, 0], 10)];
    let (result, _) = run(Water::None, &tables, &[], &mut gens, &mut cells);
    let fault = PaintFault::NoPicture {
        terrain: 2,
        shape: 8,
    };
    assert_eq!(
        result,
        Err(GenerateError::Terrain(TerrainFault::Paint(fault)))
    );
}
