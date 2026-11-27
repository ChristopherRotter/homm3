//! Steps 4 and 5 on small maps (RMG-TWN-001 to -005): the first town nearest the centre, the
//! kinds' order and owners, the least towns and RMG-QUIRK-13, the type redrawn, the further towns
//! farthest from the objects, the densities' loop and the faults. The corpus's steps 4 and 5
//! against the phase oracle are `h3-pack-rmg`'s.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use super::*;
use crate::cells::FREE;
use crate::part::{OBJECT_TYPES, ObjectSettings, ObjectTemplate, Strength, ZoneKind};

/// The masks' 48 bits.
const MASK48: u64 = (1 << 48) - 1;
/// Grass, a land type.
const GRASS: u32 = 2;

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

impl Log {
    fn at(&self, site: u32) -> Vec<u32> {
        self.draws
            .iter()
            .filter(|d| d.0 == site)
            .map(|d| d.1)
            .collect()
    }
}

/// A town line of `subclass`, 1 × 1, its anchor blocked and (with `entrance`) its entrance.
fn town_line(subclass: u32, entrance: bool) -> ObjectTemplate {
    ObjectTemplate {
        picture: String::from("AVCtest.def"),
        passability: MASK48 & !1,
        actions: u64::from(entrance),
        editor_terrains: 0x1FF,
        terrains: 0x1FE,
        class: 98,
        subclass,
        group: 1,
        overlay: 0,
        width: 1,
        height: 1,
        colour: MASK48,
    }
}

fn settings() -> Vec<ObjectSettings> {
    (0..OBJECT_TYPES as u8)
        .map(|class| ObjectSettings {
            not_entered: false,
            from_above: false,
            removable: false,
            class_group: class,
        })
        .collect()
}

/// A treasure zone of id 0 asking for `player` (least towns, castles, densities) and `neutral`.
fn zone(ownership: i32, player: [i32; 4], neutral: [i32; 4], types: u16, same: bool) -> Zone {
    Zone {
        id: 0,
        kind: ZoneKind::Treasure,
        base_size: 10,
        humans: [0, 8],
        players: [0, 8],
        ownership,
        player_towns: player,
        neutral_towns: neutral,
        same_type: same,
        town_types: types,
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

/// What the steps work on: a `side` × `side` level of grass of zone 0, one zone generator of
/// `town_type` centred at `centre` with the whole map as its box, the towns of `lines`.
struct World {
    zones: Vec<Zone>,
    gens: Vec<ZoneGenerator>,
    cells: Cells,
    objects: Objects,
    placed: Placed,
    positions: [Option<u8>; COLOURS],
    format: MapFormat,
    skipped: bool,
    log: Log,
}

impl World {
    fn new(side: i32, z: Zone, town_type: i32, centre: [i32; 2], lines: &[ObjectTemplate]) -> Self {
        let mut cells = Cells::new(side, 1);
        for y in 0..side {
            for x in 0..side {
                let c = cells.get_mut(x, y, 0).unwrap();
                c.mark(0, true);
                c.tile = (c.tile & !0x3F) | GRASS;
            }
        }
        let gens = vec![ZoneGenerator {
            zone: 0,
            town_type,
            town_type_2: town_type,
            terrain: GRASS as i32,
            position: [centre[0], centre[1], 0],
            size: side,
            bounds: [0, 0, side, side],
            town: None,
        }];
        let mut positions = [None; COLOURS];
        positions[0] = Some(3);
        Self {
            zones: vec![z],
            gens,
            cells,
            objects: Objects::new(lines, &settings(), MapFormat::SoD, 1),
            placed: Placed::default(),
            positions,
            format: MapFormat::SoD,
            skipped: true,
            log: Log::default(),
        }
    }

    /// Step `step` from `seed`.
    fn run(&mut self, step: u8, seed: u32) -> Result<(), GenerateError> {
        let input = TownInput {
            zones: &self.zones,
            positions: &self.positions,
            format: self.format,
            least_town_skipped: self.skipped,
        };
        let mut stream = Stream::new(seed, &mut self.log);
        towns(
            &mut stream,
            &input,
            step,
            &mut self.gens,
            &mut self.cells,
            &mut self.objects,
            &mut self.placed,
        )
    }

    fn cell(&mut self, x: i32, y: i32) -> &mut crate::cells::Cell {
        self.cells.get_mut(x, y, 0).unwrap()
    }

    /// The placed towns' (x, y), owners, castles and subclasses.
    fn towns(&self) -> Vec<([i32; 2], i32, bool, u32)> {
        self.placed
            .towns
            .iter()
            .map(|t| {
                let o = self.objects.placed()[t.object];
                (
                    [o.position[0], o.position[1]],
                    t.owner,
                    t.castle,
                    o.subclass,
                )
            })
            .collect()
    }
}

/// The nine towns' lines, subclasses 0 to 8.
fn nine() -> Vec<ObjectTemplate> {
    (0..9).map(|t| town_line(t, true)).collect()
}

// covers: RMG-TWN-001, RMG-TWN-002
/// Step 4 places a zone's first town, of the first kind asked for (a colour castle before a
/// colour town) and that one only: of its town type's prototype (the vector's index), at the
/// fitting cells nearest the centre, the r-th row by row; its entrance in the zone generator and
/// the road targets, the cell below it kept free and not for obstacles, the serial taken.
#[test]
fn the_first_town_goes_to_the_nearest_fitting_cells() {
    let mut w = World::new(
        12,
        zone(1, [1, 1, 0, 0], [0; 4], 0, false),
        2,
        [5, 5],
        &nine(),
    );
    for y in 0..12 {
        for x in 0..12 {
            w.cell(x, y).tile_data = (w.cell(x, y).tile_data | OBSTACLES) & !KEPT;
        }
    }
    // The centre does not fit; (5, 4) neither (the cell below its entrance is not free).
    w.cell(5, 5).tile_data &= !FREE;
    w.run(4, 7).unwrap();
    let drawn = w.log.at(SITE_TOWN_CENTRAL_CELL);
    assert_eq!(w.log.draws.len(), 1);
    let nearest = [[4, 5], [6, 5], [5, 6]];
    let at = nearest[drawn[0] as usize % 3];
    assert_eq!(w.towns(), [(at, 3, true, 2)]);
    assert_eq!(w.gens[0].town, Some([at[0], at[1], 0]));
    assert_eq!(w.placed.road_targets, [[at[0], at[1], 0]]);
    assert_eq!(w.placed.towns[0].serial, 1);
    assert_eq!(w.placed.towns[0].zone, 0);
    assert_eq!(w.objects.serial(), 2);
    assert_eq!(w.objects.prototypes()[2].used, 1);
    let below = w.cells.get(at[0], at[1] + 1, 0).unwrap().tile_data;
    assert_eq!(below & (KEPT | OBSTACLES), KEPT);
    // A Border Guard pending below the entrance keeps the cell's marks.
    let mut w = World::new(
        12,
        zone(1, [0, 1, 0, 0], [0; 4], 0, false),
        2,
        [5, 5],
        &nine(),
    );
    for y in 0..12 {
        for x in 0..12 {
            w.cell(x, y).pending_guard = 1;
            w.cell(x, y).tile_data |= OBSTACLES;
        }
    }
    w.run(4, 7).unwrap();
    assert_eq!(w.towns(), [([5, 5], 3, true, 2)]);
    assert_ne!(w.cells.get(5, 6, 0).unwrap().tile_data & OBSTACLES, 0);
}

// covers: RMG-TWN-001, RMG-TWN-002
/// A first town is refused, without a draw, for town type -1, without a cell of the zone in its
/// box, and beyond the squared distance 32000 from its centre.
#[test]
fn a_first_town_is_refused_without_its_type_its_cells_or_nearness() {
    let asked = zone(1, [1, 1, 1, 1], [0; 4], 0, false);
    let mut none = World::new(12, asked.clone(), -1, [5, 5], &nine());
    none.run(4, 1).unwrap();
    let mut other = World::new(12, asked.clone(), 2, [5, 5], &nine());
    for y in 0..12 {
        for x in 0..12 {
            other.cell(x, y).mark(1, true);
        }
    }
    other.run(4, 1).unwrap();
    // 179² + 0² = 32041 is past 32000.
    let mut far = World::new(12, asked, 2, [190, 0], &nine());
    far.run(4, 1).unwrap();
    for w in [none, other, far] {
        assert!(w.placed.towns.is_empty() && w.log.draws.is_empty());
        assert_eq!(w.gens[0].town, None);
    }
}

// covers: RMG-TWN-001, RMG-TWN-002
/// The kinds' owners: a colour kind takes the colour of the zone's position (Ownership less 1),
/// -1 when the position has none or is -1 (Ownership 0: the colours' slot before position 0); a
/// neutral kind -1. Step 4 skips the kinds not asked for.
#[test]
fn a_towns_owner_is_its_zones_colour_or_neutral() {
    for (ownership, player, neutral, owner, castle) in [
        (1, [1, 0, 0, 0], [0; 4], 3, false),
        (2, [1, 0, 0, 0], [0; 4], -1, false),
        (0, [0, 1, 0, 0], [0; 4], -1, true),
        (1, [0; 4], [0, 1, 0, 0], -1, true),
        (1, [0; 4], [1, 0, 0, 0], -1, false),
    ] {
        let mut w = World::new(
            12,
            zone(ownership, player, neutral, 0, false),
            4,
            [5, 5],
            &nine(),
        );
        w.run(4, 3).unwrap();
        assert_eq!(
            w.towns(),
            [([5, 5], owner, castle, 4)],
            "{ownership} {player:?}"
        );
    }
}

// covers: RMG-TWN-002
/// RMG-QUIRK-4's town cases are typed errors: a position past 7 (the colours read outside), a
/// town type past the type-98 vector, a town line without an entrance (the cell below its
/// offsets 8, 6 off the map, then step 5's entrance cells off it).
#[test]
fn reads_outside_the_originals_arrays_are_refused() {
    let fault = |step, fault| Err(GenerateError::Objects { step, fault });
    let mut w = World::new(
        12,
        zone(10, [1, 0, 0, 0], [0; 4], 0, false),
        4,
        [5, 5],
        &nine(),
    );
    assert_eq!(w.run(4, 1), fault(4, ObjectFault::Colour));
    let lines = [town_line(0, true)];
    let mut w = World::new(
        12,
        zone(1, [1, 0, 0, 0], [0; 4], 0, false),
        5,
        [5, 5],
        &lines,
    );
    assert_eq!(w.run(4, 1), fault(4, ObjectFault::Prototype));
    assert!(w.placed.towns.is_empty());
    let lines = [town_line(0, false)];
    let mut w = World::new(
        12,
        zone(1, [1, 0, 0, 0], [2, 0, 0, 0], 0, true),
        0,
        [5, 5],
        &lines,
    );
    assert_eq!(w.run(4, 1), fault(4, ObjectFault::Outside));
    assert_eq!(w.gens[0].town, Some([-3, -1, 0]));
    assert_eq!(w.run(5, 1), fault(5, ObjectFault::Outside));
    // A far town of a type without its line places nothing.
    let lines = [town_line(0, true)];
    let mut w = World::new(
        12,
        zone(1, [0; 4], [3, 0, 0, 0], 0x002, false),
        0,
        [5, 5],
        &lines,
    );
    w.run(4, 1).unwrap();
    assert_eq!(w.run(5, 1), fault(5, ObjectFault::Prototype));
    assert_eq!(w.placed.towns.len(), 1);
}

// covers: RMG-TWN-003, RMG-TWN-005
/// RMG-QUIRK-13: step 5 skips the first of the first kind asked for as step 4's. A zone without
/// a town type gets none in step 4 (type -1 refused); with the flag step 5 skips its one least
/// town all the same, without it places that town: its type redrawn among all (no type set:
/// `rand() % 9`, RoE's 8), at the cell nearest the centre (no town yet).
#[test]
fn step_5_skips_the_least_town_step_4_never_placed_by_the_flag() {
    for (skipped, format, all) in [
        (true, MapFormat::SoD, 9),
        (false, MapFormat::SoD, 9),
        (false, MapFormat::RoE, 8),
    ] {
        for seed in 0..6 {
            let mut w = World::new(
                12,
                zone(1, [0; 4], [1, 0, 0, 0], 0, false),
                -1,
                [5, 5],
                &nine(),
            );
            w.skipped = skipped;
            w.format = format;
            w.run(4, seed).unwrap();
            assert!(w.placed.towns.is_empty() && w.log.draws.is_empty());
            w.run(5, seed).unwrap();
            if skipped {
                assert!(w.placed.towns.is_empty() && w.log.draws.is_empty());
                continue;
            }
            let drawn = w.log.at(SITE_TOWN_TYPE_ANY);
            assert_eq!(w.log.at(SITE_TOWN_CENTRAL_CELL).len(), 1);
            assert_eq!(w.towns(), [([5, 5], -1, false, drawn[0] % all)], "{seed}");
            // The zone generator's town type stays -1.
            assert_eq!(w.gens[0].town_type, -1);
        }
    }
}

/// The cells where a 1 × 1 town of `prototype` would go as a further town: of the zone, at the
/// greatest object distance among those that fit with the 3 × 3 around them of the zone.
fn farthest(w: &mut World, prototype: usize) -> Vec<[i32; 2]> {
    let place = PlaceZone {
        id: 0,
        water: false,
    };
    let mut kept: Vec<([i32; 2], u32)> = Vec::new();
    let (side, _, _) = w.cells.size();
    for y in 1..side {
        for x in 1..side {
            let around = (y - 1..(y + 2).min(side)).all(|v| {
                (x - 1..(x + 2).min(side)).all(|u| w.cells.get(u, v, 0).unwrap().zone_index() == 0)
            });
            let c = *w.cells.get(x, y, 0).unwrap();
            if c.zone_index() == 0
                && around
                && w.objects.can_place(&w.cells, prototype, [x, y, 0], place)
            {
                kept.push(([x, y], c.zone & 0xFFFF));
            }
        }
    }
    let most = kept.iter().map(|k| k.1).max().unwrap();
    kept.into_iter()
        .filter(|k| k.1 == most)
        .map(|k| k.0)
        .collect()
}

// covers: RMG-TWN-003, RMG-TWN-005
/// Step 5's least towns: after step 4's, each further one with its type redrawn among the zone's
/// set types (a neutral town without the same-type byte: `rand() %` their count, the r-th), at the
/// cells of the greatest object distance whose 3 × 3 around the entrance is of the zone, the r-th.
#[test]
fn further_towns_go_farthest_from_the_objects() {
    let mut w = World::new(
        12,
        zone(1, [0; 4], [3, 0, 0, 0], 0x024, false),
        2,
        [5, 5],
        &nine(),
    );
    // The far corner's cells are another zone's: the cells beside them are refused.
    w.cell(11, 11).mark(1, true);
    w.run(4, 5).unwrap();
    assert_eq!(w.towns(), [([5, 5], -1, false, 2)]);
    let (cells, objects) = (w.cells.clone(), w.objects.clone());
    w.run(5, 5).unwrap();
    let types = w.log.at(SITE_TOWN_TYPE_ZONE);
    let spots = w.log.at(SITE_TOWN_FAR_CELL);
    assert_eq!((types.len(), spots.len()), (2, 2));
    let towns = w.towns();
    assert_eq!(towns.len(), 3);
    let second = [2, 5][types[0] as usize % 2];
    assert_eq!(towns[1].3, second);
    // The first further town among the farthest as step 4 left the map.
    let mut before = World {
        cells,
        objects,
        ..World::new(12, zone(1, [0; 4], [0; 4], 0, false), 2, [5, 5], &nine())
    };
    let prototype = before.objects.group(98)[second as usize];
    let spots_then = farthest(&mut before, prototype);
    assert!(!spots_then.contains(&[10, 10]) && !spots_then.contains(&[11, 10]));
    assert_eq!(towns[1].0, spots_then[spots[0] as usize % spots_then.len()]);
    assert!(towns.iter().all(|t| t.1 == -1 && !t.2));
}

// covers: RMG-TWN-005
/// A further town keeps its type for a colour kind, and for a neutral one with the zone's
/// same-type byte; a colour kind of town type -1 draws it among the zone's.
#[test]
fn a_further_towns_type_is_redrawn_only_when_neutral_or_none() {
    for (player, neutral, same, town_type, redrawn) in [
        ([2, 0, 0, 0], [0; 4], false, 6, false),
        ([0; 4], [2, 0, 0, 0], true, 6, false),
        ([2, 0, 0, 0], [0; 4], true, -1, true),
    ] {
        let mut w = World::new(
            12,
            zone(1, player, neutral, 0x100, same),
            town_type,
            [5, 5],
            &nine(),
        );
        w.run(4, 2).unwrap();
        w.run(5, 2).unwrap();
        assert_eq!(w.log.at(SITE_TOWN_TYPE_ZONE).len(), usize::from(redrawn));
        let towns = w.towns();
        let want = if redrawn { 8 } else { 6 };
        assert_eq!(
            towns.len(),
            2 - usize::from(redrawn),
            "{player:?} {neutral:?}"
        );
        assert!(towns.iter().all(|t| t.3 == want));
    }
}

// covers: RMG-TWN-004, RMG-TWN-005
/// The densities: S = 82944 gives the spacing trunc(sqrt(1)) = 1; the two kinds of equal step
/// take turns, the first on ties (the neutral castle, then the neutral town), the first town of
/// a zone without one nearest the centre, until each has failed once; a zone without densities
/// places nothing.
#[test]
fn the_densities_take_turns_by_their_keys_until_each_fails() {
    let mut w = World::new(
        9,
        zone(1, [0; 4], [0, 0, 41_472, 41_472], 0, true),
        3,
        [4, 4],
        &nine(),
    );
    w.run(4, 9).unwrap();
    assert!(w.placed.towns.is_empty());
    w.run(5, 9).unwrap();
    let towns = w.towns();
    assert!(towns.len() > 3, "{towns:?}");
    assert_eq!(towns[0], ([4, 4], -1, true, 3));
    assert!(!towns[1].2 && towns[2].2);
    assert_eq!(w.log.at(SITE_TOWN_CENTRAL_CELL).len(), 1);
    assert_eq!(w.log.at(SITE_TOWN_FAR_CELL).len(), towns.len() - 1);
    // Each further town stands at an object distance of at least 1: never on an entrance.
    let entrances: Vec<[i32; 3]> = w.placed.road_targets.clone();
    assert_eq!(entrances.len(), towns.len());
    let mut sorted = entrances.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), entrances.len());
    // Without densities, nothing.
    let mut none = World::new(
        9,
        zone(1, [0; 4], [0, 0, 0, -3], 0, true),
        3,
        [4, 4],
        &nine(),
    );
    none.run(5, 9).unwrap();
    assert!(none.placed.towns.is_empty() && none.log.draws.is_empty());
}
