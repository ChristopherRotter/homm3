//! The object model on small maps (RMG-OBJ-009 to -011): the prototypes by format and class
//! group, the monsters' sort, the shapes, the placing and its lists and dump, the entrances'
//! counts and distance flood, the footprint, outline and ring tests and the faults.

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use super::*;
use crate::cells::Cell;
use crate::generate::{GenerateError, StepState};

/// Grass, a land type.
const GRASS: u32 = 2;

/// The bits of the cells `[c, r]` (c columns left of, r rows above the anchor).
fn bits(cells: &[[i32; 2]]) -> u64 {
    cells.iter().fold(0, |b, &[c, r]| b | 1 << (8 * r + c))
}

/// A line of class `class` and subclass `subclass`, 1 × 1, its anchor blocked, on land.
fn line(class: u32, subclass: u32) -> ObjectTemplate {
    ObjectTemplate {
        picture: String::from("AVXtest.def"),
        passability: MASK48 & !1,
        actions: 0,
        editor_terrains: 0x1FF,
        terrains: 0x1FE,
        class,
        subclass,
        group: 1,
        overlay: 0,
        width: 1,
        height: 1,
        colour: MASK48,
    }
}

/// Every class its own group, every flag clear.
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

/// A shape `width` × `height` with the blocked cells `blocked` and the entrances `entrances`
/// (blocked too), on land.
fn shape(width: i32, height: i32, blocked: &[[i32; 2]], entrances: &[[i32; 2]]) -> Shape {
    let entrances = bits(entrances);
    Shape {
        passable: MASK48 & !bits(blocked) & !entrances,
        entrances,
        has_entrance: entrances != 0,
        entrance: if entrances != 0 {
            let k = entrances.trailing_zeros() as i32;
            [k % 8, k / 8]
        } else {
            NO_ENTRANCE
        },
        width,
        height,
        sea: false,
        class: 5,
        subclass: 0,
    }
}

/// A `side` × `side` map of `levels` levels, every cell grass of zone 0.
fn grass(side: i32, levels: i32) -> Cells {
    let mut cells = Cells::new(side, levels);
    for z in 0..levels {
        for y in 0..side {
            for x in 0..side {
                let c = cells.get_mut(x, y, z).unwrap();
                c.mark(0, true);
                c.tile = (c.tile & !0x3F) | GRASS;
            }
        }
    }
    cells
}

fn cell(cells: &mut Cells, x: i32, y: i32) -> &mut Cell {
    cells.get_mut(x, y, 0).unwrap()
}

/// The cell's land type `land`.
fn set_land(cells: &mut Cells, x: i32, y: i32, land: u32) {
    let c = cell(cells, x, y);
    c.tile = (c.tile & !0x3F) | land;
}

const ZONE: PlaceZone = PlaceZone {
    id: 0,
    water: false,
};

/// The objects of `lines` in SoD with `zones` zone generators.
fn objects(lines: &[ObjectTemplate], zones: usize) -> Objects {
    Objects::new(lines, &settings(), MapFormat::SoD, zones)
}

/// The templates of the prototypes of `group`.
fn templates(o: &Objects, group: usize) -> Vec<usize> {
    o.group(group)
        .iter()
        .map(|&p| o.prototypes()[p].template)
        .collect()
}

// covers: RMG-OBJ-009
/// Each line whose class the format allows becomes a prototype of its class group in the table's
/// order: RoE below 165, AB below 222, SoD below 232, and before SoD the monoliths and gates (43
/// to 45) only of subclasses 0 to 2 (compared signed, as `c_atoi`'s); a class past the settings
/// files as itself.
#[test]
fn the_prototypes_are_filed_by_the_format_and_the_class_group() {
    let lines = [
        line(5, 0),            // 0: every format
        line(165, 0),          // 1: AB and SoD
        line(222, 0),          // 2: SoD
        line(232, 0),          // 3: none
        line(43, 3),           // 4: SoD
        line(43, 2),           // 5: every format
        line(44, 0x8000_0000), // 6: every format (subclass negative)
        line(45, 7),           // 7: SoD
        line(u32::MAX, 0),     // 8: none (class negative)
        line(219, 1),          // 9: AB and SoD, filed with 33
        line(5, 4),            // 10: every format
    ];
    let mut s = settings();
    s[219].class_group = 33;
    // A class group past the vectors files nothing.
    s[5].class_group = 240;
    let filed = |format| {
        let o = Objects::new(&lines, &s, format, 0);
        (0..OBJECT_TYPES)
            .map(|g| (g, templates(&o, g)))
            .filter(|(_, t)| !t.is_empty())
            .collect::<Vec<_>>()
    };
    assert_eq!(filed(MapFormat::RoE), [(43, vec![5]), (44, vec![6])]);
    assert_eq!(
        filed(MapFormat::AB),
        [(33, vec![9]), (43, vec![5]), (44, vec![6]), (165, vec![1])]
    );
    assert_eq!(
        filed(MapFormat::SoD),
        [
            (33, vec![9]),
            (43, vec![4, 5]),
            (44, vec![6]),
            (45, vec![7]),
            (165, vec![1]),
            (222, vec![2])
        ]
    );
    // Without settings a class files as itself; past the groups there are none.
    let o = Objects::new(&lines, &[], MapFormat::SoD, 0);
    assert_eq!(templates(&o, 219), [9]);
    assert!(o.group(OBJECT_TYPES).is_empty());
    let found: Vec<usize> = o.with_subclass(5, 4).collect();
    assert_eq!(found, [o.group(5)[1]]);
    let p = &o.prototypes()[o.group(5)[0]];
    assert_eq!((p.template, p.used, p.outline.len()), (0, 0, 0));
    assert_eq!(p.shape, Shape::of(&lines[0]));
}

// covers: RMG-OBJ-009
/// The monsters' prototypes (54) are sorted by subclass with the exchange sort: a later smaller
/// one is exchanged into each place, so equal subclasses may change order (2a 2b 1 gives 1 2b
/// 2a) or keep it (3 1a 2 1b gives 1a 1b 2 3); another class keeps the table's order.
#[test]
fn the_monsters_are_sorted_by_the_exchange_sort() {
    let o = objects(
        &[
            line(54, 2),
            line(54, 2),
            line(54, 1),
            line(17, 2),
            line(17, 1),
        ],
        0,
    );
    assert_eq!(templates(&o, 54), [2, 1, 0]);
    assert_eq!(templates(&o, 17), [3, 4]);
    let o = objects(&[line(54, 3), line(54, 1), line(54, 2), line(54, 1)], 0);
    assert_eq!(templates(&o, 54), [1, 3, 2, 0]);
}

// covers: RMG-OBJ-009
/// The shape: passable is the text or no body pixel, the entrances the visitable cells not
/// passable, the first entrance the lowest bit (rows up, each leftward; none: (8, 6)); a sea
/// object is group 0 with water in the fourth column; a cell past bit 47 is a range error.
#[test]
fn the_shape_takes_the_msk_and_the_first_entrance() {
    let mut t = line(5, 0);
    t.passability = MASK48 & !bits(&[[0, 0], [1, 0], [2, 1], [3, 1]]);
    // (1, 0) has no body pixel: passable whatever the text.
    t.colour = MASK48 & !bits(&[[1, 0]]);
    t.actions = bits(&[[1, 0], [3, 1], [2, 1], [5, 5]]);
    t.width = 4;
    t.height = 2;
    let s = Shape::of(&t);
    assert_eq!(s.passable, MASK48 & !bits(&[[0, 0], [2, 1], [3, 1]]));
    assert_eq!(s.entrances, bits(&[[2, 1], [3, 1]]));
    assert!(s.has_entrance);
    assert_eq!(s.entrance, [2, 1]);
    assert_eq!((s.width, s.height, s.class, s.subclass), (4, 2, 5, 0));
    assert!(!s.sea);
    assert_eq!(s.cell(0, 0), Some((false, false)));
    assert_eq!(s.cell(3, 1), Some((false, true)));
    assert_eq!(s.cell(7, 5), Some((true, false)));
    assert_eq!(s.cell(8, 5), None);
    assert_eq!(s.cell(-1, 0), None);
    let s = Shape::of(&line(5, 0));
    assert_eq!(
        (s.has_entrance, s.entrance, s.entrances),
        (false, [8, 6], 0)
    );
    let mut sea = line(8, 0);
    sea.group = 0;
    sea.terrains = 1;
    assert!(Shape::of(&sea).sea);
    sea.group = 1;
    assert!(!Shape::of(&sea).sea);
    sea.group = 0;
    sea.terrains = 0x1FE;
    assert!(!Shape::of(&sea).sea);
}

// covers: RMG-OBJ-009
/// Placing: an entrance gets bit 22 and, unless a pending guard, bit 26 cleared and 27 set, its
/// bit 25 kept; a blocked cell loses bit 25; both list the object; passable cells and cells off
/// the map are left alone; the type is counted, the entrance's zone too (not -1); the use count
/// is the create methods'.
#[test]
fn placing_marks_the_footprint_and_lists_the_object() {
    let mut t = line(5, 0);
    // Row 0: the anchor an entrance, (1, 0) blocked, (2, 0) passable; row 1: (0, 1) blocked.
    t.passability = MASK48 & !bits(&[[0, 0], [1, 0], [0, 1]]);
    t.actions = bits(&[[0, 0]]);
    t.width = 3;
    t.height = 2;
    let mut o = objects(&[t.clone(), t], 2);
    let mut cells = grass(4, 1);
    // Row 3 to be filled with obstacles, not kept free.
    for x in 0..4 {
        let c = cell(&mut cells, x, 3);
        c.tile_data = (c.tile_data | OBSTACLES) & !KEPT;
    }
    cell(&mut cells, 3, 3).pending_guard = 1;
    let p = o.group(5)[0];
    o.created(p);
    o.created(p);
    assert_eq!(o.prototypes()[p].used, 2);
    let at = |cells: &Cells, x, y| *cells.get(x, y, 0).unwrap();
    let marks = ENTRANCE | FREE | OBSTACLES | KEPT;
    assert_eq!(o.add(&mut cells, p, [2, 3, 0]), 0);
    assert_eq!(at(&cells, 2, 3).tile_data & marks, ENTRANCE | FREE | KEPT);
    // The second's blocked cell (1, 0) is the first's entrance: it loses bit 25.
    assert_eq!(o.add(&mut cells, p, [3, 3, 0]), 1);
    assert_eq!(at(&cells, 2, 3).tile_data & marks, ENTRANCE | KEPT);
    // The third's footprint runs off the map's top and left edges.
    assert_eq!(o.add(&mut cells, p, [1, 0, 0]), 2);
    // The pending guard keeps 26 and 27.
    assert_eq!(
        at(&cells, 3, 3).tile_data & marks,
        ENTRANCE | FREE | OBSTACLES
    );
    assert_eq!(at(&cells, 1, 3).tile_data & (ENTRANCE | FREE), 0);
    assert_eq!(cells.objects_at(2, 3, 0), [0, 1]);
    assert_eq!(cells.objects_at(3, 3, 0), [1]);
    assert_eq!(cells.objects_at(1, 3, 0), [0]);
    assert_eq!(cells.objects_at(2, 2, 0), [0]);
    assert!(cells.objects_at(0, 3, 0).is_empty());
    assert_eq!(at(&cells, 0, 3).tile_data & FREE, FREE);
    assert!(cells.objects_at(9, 9, 0).is_empty());
    assert_eq!(cells.objects_at(1, 0, 0), [2]);
    assert_eq!(cells.objects_at(0, 0, 0), [2]);
    assert_eq!(o.count_by_type()[5], 3);
    assert_eq!(o.zone_count(0)[5], 3);
    assert!(o.zone_count(2).is_empty());
    assert_eq!(
        o.placed()[1],
        Object {
            prototype: p,
            template: 0,
            class: 5,
            subclass: 0,
            position: [3, 3, 0],
        }
    );
    assert_eq!(o.take_fault(), None);
    // An entrance outside every zone is not counted.
    let mut cells = Cells::new(4, 1);
    o.add(&mut cells, p, [2, 2, 0]);
    assert_eq!((o.count_by_type()[5], o.zone_count(0)[5]), (4, 3));
}

// covers: RMG-OBJ-009
/// The dump: per cell its objects' count and the first three's indices (-1 none, or not among
/// the placed objects given), the objects' six words, the places past the third.
#[test]
fn the_dump_holds_the_objects_and_the_further_places() {
    let mut o = objects(&[line(9, 9), line(5, 7)], 1);
    let mut cells = grass(2, 1);
    let p = o.group(5)[0];
    for _ in 0..5 {
        o.add(&mut cells, p, [1, 0, 0]);
    }
    let placed = o.placed();
    let state = StepState {
        cells: &cells,
        zones: &[],
        objects: placed,
    };
    let dump = state.dump(4);
    assert_eq!(dump, cells.dump(4, placed));
    let word =
        |dump: &[u8], k: usize| u32::from_le_bytes(dump[4 * k + 8..4 * k + 12].try_into().unwrap());
    assert_eq!(word(&dump, 1), 4);
    // Cell 1 (x 1): five objects, 0 to 2 first.
    assert_eq!(
        (17..21).map(|k| word(&dump, k)).collect::<Vec<_>>(),
        [5, 0, 1, 2]
    );
    // Cell 0: none.
    assert_eq!(
        (5..9).map(|k| word(&dump, k)).collect::<Vec<_>>(),
        [0, u32::MAX, u32::MAX, u32::MAX]
    );
    let objects = 5 + 12 * 4;
    assert_eq!(word(&dump, objects), 5);
    assert_eq!(
        (objects + 1..objects + 7)
            .map(|k| word(&dump, k))
            .collect::<Vec<_>>(),
        [1, 5, 7, 1, 0, 0]
    );
    let further = objects + 1 + 6 * 5;
    assert_eq!(
        (further..further + 7)
            .map(|k| word(&dump, k))
            .collect::<Vec<_>>(),
        [2, 1, 3, 3, 1, 4, 4]
    );
    assert_eq!(dump.len(), 4 * (further + 7) + 8);
    // Objects not among those given are -1.
    let short = cells.dump(4, &placed[..2]);
    assert_eq!(word(&short, 20), u32::MAX);
    assert_eq!(word(&short, objects), 2);
    let further = objects + 1 + 6 * 2;
    assert_eq!(
        (further..further + 7)
            .map(|k| word(&short, k))
            .collect::<Vec<_>>(),
        [2, 1, 3, u32::MAX, 1, 4, u32::MAX]
    );
}

/// A 1 × 1 object with its anchor the entrance, of class 5.
fn post() -> Objects {
    let mut t = line(5, 0);
    t.actions = 1;
    objects(&[t], 4)
}

fn distances(cells: &Cells, z: i32) -> Vec<Vec<u32>> {
    let (w, h, _) = cells.size();
    (0..h)
        .map(|y| {
            (0..w)
                .map(|x| cells.get(x, y, z).unwrap().zone & 0xFFFF)
                .collect()
        })
        .collect()
}

// covers: RMG-OBJ-010
/// The entrance's flood: the least distance on its level, 2 a side and 3 a diagonal step, over
/// every cell; kept where not below (a lower value stops it there: the cells beyond are reached
/// around it); a later entrance lowers only what it reaches closer; the other level untouched.
#[test]
fn an_entrance_floods_the_distances_on_its_level() {
    let mut o = post();
    let mut cells = grass(5, 2);
    let p = o.group(5)[0];
    // A distance of 0 at (2, 1) before: the flood never passes it.
    let c = cell(&mut cells, 2, 1);
    c.zone &= 0xFFFF_0000;
    o.add(&mut cells, p, [2, 2, 0]);
    let mut want: Vec<Vec<u32>> = (0..5)
        .map(|y: i32| {
            (0..5)
                .map(|x: i32| {
                    let (a, b) = ((x - 2).unsigned_abs(), (y - 2).unsigned_abs());
                    2 * a.max(b) + a.min(b)
                })
                .collect()
        })
        .collect();
    want[1][2] = 0;
    want[0][2] = 6;
    assert_eq!(distances(&cells, 0), want);
    assert!(distances(&cells, 1).iter().flatten().all(|&d| d == 32_700));
    // The zone bytes are kept.
    assert!(cells.cells().iter().all(|c| c.zone_index() == 0));
    o.add(&mut cells, p, [0, 4, 0]);
    let d = distances(&cells, 0);
    assert_eq!(
        (d[4][0], d[3][0], d[4][1], d[3][1], d[2][2], d[0][0]),
        (0, 2, 2, 3, 0, 6)
    );
    assert_eq!((d[4][4], d[4][2]), (6, 4));
    assert_eq!(o.zone_count(0)[5], 2);
}

// covers: RMG-OBJ-010
/// The flood's queue: a cell goes before the first of a distance at most its own, so taken from
/// the end the nearest come first and equal ones first in, first out.
#[test]
fn the_floods_queue_takes_the_nearest_first_in_first_out() {
    let mut queue = vec![([0, 0], 5), ([1, 0], 3), ([2, 0], 2)];
    enqueue(&mut queue, [3, 0], 3);
    enqueue(&mut queue, [4, 0], 6);
    enqueue(&mut queue, [5, 0], 1);
    let order: Vec<i32> = core::iter::from_fn(|| queue.pop().map(|(c, _)| c[0])).collect();
    assert_eq!(order, [5, 2, 1, 3, 0, 4]);
}

// covers: RMG-OBJ-009
/// The faults of placing: a footprint on a level the map has not, past the masks' bits, an
/// entrance off the map, a zone byte past the zone generators: each kept until taken, the first
/// one only.
#[test]
fn placing_faults_are_kept_until_taken() {
    let mut o = post();
    let p = o.group(5)[0];
    let mut cells = grass(3, 1);
    o.add(&mut cells, p, [1, 1, 1]);
    assert_eq!(o.take_fault(), Some(ObjectFault::Outside));
    assert_eq!(o.take_fault(), None);
    // Past the masks: a footprint 9 wide and 6 high reaches bit 48 at (8, 5).
    let mut wide = line(5, 0);
    wide.width = 9;
    wide.height = 6;
    let mut w = objects(&[wide], 1);
    let mut big = grass(10, 1);
    let q = w.group(5)[0];
    w.add(&mut big, q, [9, 9, 0]);
    assert_eq!(w.take_fault(), Some(ObjectFault::MaskBit));
    // An entrance off the map (the anchor off it too).
    o.add(&mut cells, p, [5, 1, 0]);
    assert_eq!(o.take_fault(), Some(ObjectFault::Outside));
    // A zone byte past the four zone generators; then the first fault kept.
    cell(&mut cells, 0, 0).mark(4, true);
    o.add(&mut cells, p, [0, 0, 0]);
    o.add(&mut cells, p, [1, 1, 1]);
    assert_eq!(o.take_fault(), Some(ObjectFault::Zone));
    assert_eq!(
        GenerateError::Objects {
            step: 4,
            fault: ObjectFault::Zone
        }
        .to_string(),
        "step 4, an object: Zone"
    );
}

// covers: RMG-OBJ-011
/// The footprint test: off the map by any of its four bounds; an entrance needs a free cell, not
/// rock, without an entrance, of the zone and with the flag without bit 26; a blocked cell the
/// same, without the flag's test, and on water exactly for a sea object; passable cells are not
/// tested; a cell past the masks' bits and a level the map has not are faults.
#[test]
fn the_footprint_needs_free_cells_of_the_zone() {
    // 2 × 2: the anchor an entrance, (1, 0) blocked, (0, 1) passable, (1, 1) blocked.
    let s = shape(2, 2, &[[1, 0], [1, 1]], &[[0, 0]]);
    let test = |cells: &Cells, at: [i32; 3], flag: bool| footprint_blocked(cells, &s, at, 0, flag);
    let cells = grass(4, 1);
    for at in [[0, 1, 0], [4, 1, 0], [1, 0, 0], [1, 4, 0]] {
        assert_eq!(test(&cells, at, false), Ok(true), "{at:?}");
    }
    assert_eq!(test(&cells, [1, 1, 0], true), Ok(false));
    assert_eq!(test(&cells, [3, 3, 0], false), Ok(false));
    let marks: [fn(&mut Cell); 4] = [
        |c| c.tile_data &= !FREE,
        |c| c.tile = (c.tile & !0x3F) | 9,
        |c| c.tile_data |= ENTRANCE,
        |c| c.zone = (c.zone & 0xFF00_FFFF) | 0x1_0000,
    ];
    // Each refuses the entrance and the blocked cells; a passable cell takes them.
    for mark in marks {
        for (x, y, blocked) in [(1, 1, true), (0, 1, true), (0, 0, true), (1, 0, false)] {
            let mut cells = grass(4, 1);
            mark(cell(&mut cells, x, y));
            assert_eq!(test(&cells, [1, 1, 0], false), Ok(blocked), "({x}, {y})");
        }
    }
    // Bit 26 refuses the entrance with the flag only, never a blocked cell.
    let mut cells = grass(4, 1);
    cell(&mut cells, 1, 1).tile_data |= OBSTACLES;
    cell(&mut cells, 0, 1).tile_data |= OBSTACLES;
    assert_eq!(test(&cells, [1, 1, 0], false), Ok(false));
    assert_eq!(test(&cells, [1, 1, 0], true), Ok(true));
    // Water: a blocked cell on it refuses unless the shape is a sea object's, which refuses land.
    let mut cells = grass(4, 1);
    cell(&mut cells, 0, 1).tile = 0x548;
    assert_eq!(test(&cells, [1, 1, 0], false), Ok(true));
    let sea = Shape { sea: true, ..s };
    assert_eq!(
        footprint_blocked(&cells, &sea, [1, 1, 0], 0, false),
        Ok(true)
    );
    for x in 0..2 {
        for y in 0..2 {
            cell(&mut cells, x, y).tile = 0x548;
        }
    }
    assert_eq!(
        footprint_blocked(&cells, &sea, [1, 1, 0], 0, false),
        Ok(false)
    );
    assert_eq!(test(&cells, [1, 1, 0], false), Ok(true));
    // The faults.
    assert_eq!(test(&cells, [1, 1, 1], false), Err(ObjectFault::Outside));
    let wide = shape(9, 6, &[[0, 0]], &[]);
    assert_eq!(
        footprint_blocked(&grass(10, 1), &wide, [9, 9, 0], 0, false),
        Err(ObjectFault::MaskBit)
    );
}

// covers: RMG-OBJ-011
/// The outline: the ring clockwise around the blocked group the anchor's row's first blocked
/// cell belongs to, from the cell below it, passable footprint cells on it included; none when
/// the anchor's row is passable (or the footprint has no column); a cell past the masks' bits a
/// fault.
#[test]
fn the_outline_rings_the_blocked_group_clockwise() {
    let one = shape(1, 1, &[[0, 0]], &[]);
    assert_eq!(
        outline(&one),
        Ok(vec![
            [0, 1],
            [-1, 1],
            [-1, 0],
            [-1, -1],
            [0, -1],
            [1, -1],
            [1, 0],
            [1, 1]
        ])
    );
    // 3 × 2: (2, 0) and (0, 1) passable.
    let l = shape(3, 2, &[[1, 0], [1, 1], [2, 1]], &[[0, 0]]);
    assert_eq!(
        outline(&l),
        Ok(vec![
            [0, 1],
            [-1, 1],
            [-2, 1],
            [-2, 0],
            [-3, 0],
            [-3, -1],
            [-3, -2],
            [-2, -2],
            [-1, -2],
            [0, -2],
            [0, -1],
            [1, -1],
            [1, 0],
            [1, 1]
        ])
    );
    // 3 × 1, the middle passable: the ring goes round the anchor alone, through the middle.
    let split = shape(3, 1, &[[0, 0], [2, 0]], &[]);
    assert_eq!(
        outline(&split).unwrap(),
        [
            [0, 1],
            [-1, 1],
            [-1, 0],
            [-1, -1],
            [0, -1],
            [1, -1],
            [1, 0],
            [1, 1]
        ]
    );
    // The anchor passable: from the cell below (1, 0), the anchor on the ring.
    let left = shape(2, 1, &[[1, 0]], &[]);
    assert_eq!(
        outline(&left).unwrap(),
        [
            [-1, 1],
            [-2, 1],
            [-2, 0],
            [-2, -1],
            [-1, -1],
            [0, -1],
            [0, 0],
            [0, 1]
        ]
    );
    assert_eq!(outline(&shape(2, 2, &[[0, 1]], &[])), Ok(vec![]));
    assert_eq!(outline(&shape(0, 1, &[], &[])), Ok(vec![]));
    // Past the bits: on the anchor's row of a footprint 50 wide, and on the walk round 9 × 6.
    assert_eq!(outline(&shape(50, 1, &[], &[])), Err(ObjectFault::MaskBit));
    let all: Vec<[i32; 2]> = (0..6).flat_map(|r| (0..8).map(move |c| [c, r])).collect();
    assert_eq!(outline(&shape(9, 6, &all, &[])), Err(ObjectFault::MaskBit));
}

/// The ring of `one` around (2, 2) on a 5 × 5 map of grass of zone 0.
const RING: [[i32; 2]; 8] = [
    [0, 1],
    [-1, 1],
    [-1, 0],
    [-1, -1],
    [0, -1],
    [1, -1],
    [1, 0],
    [1, 1],
];

// covers: RMG-OBJ-011
/// The ring is clear when its open cells form one arc or the whole ring: closed are cells off the
/// map, not free, rock, with an entrance, without bit 27 under the kept flag, of the other water
/// or zone; without the object flag an entrance refuses at once; an empty ring and a level the
/// map has not are faults.
#[test]
fn the_ring_is_clear_in_one_arc_or_whole() {
    let clear = |cells: &Cells, object: bool, kept: bool| {
        outline_clear(cells, &RING, [2, 2, 0], object, ZONE, kept)
    };
    let mut cells = grass(5, 1);
    assert_eq!(clear(&cells, false, false), Ok(true));
    assert_eq!(clear(&cells, false, true), Ok(true));
    // Under the kept flag every cell without bit 27 is closed: all closed.
    let mut loose = grass(5, 1);
    for k in 0..25 {
        cell(&mut loose, k % 5, k / 5).tile_data &= !KEPT;
    }
    assert_eq!(clear(&loose, false, true), Ok(false));
    // One arc closed, the first cell among them: still one arc of open cells.
    cell(&mut cells, 2, 3).tile_data &= !FREE;
    set_land(&mut cells, 1, 3, 9);
    assert_eq!(clear(&cells, false, false), Ok(true));
    // A second closed arc breaks it twice.
    cell(&mut cells, 2, 1).zone |= 0x00FF_0000;
    assert_eq!(clear(&cells, false, false), Ok(false));
    // An arc of open cells under the kept flag, the first cell closed.
    for [dx, dy] in &RING[2..5] {
        cell(&mut loose, 2 + dx, 2 + dy).tile_data |= KEPT;
    }
    assert_eq!(clear(&loose, false, true), Ok(true));
    // Water against a land zone, and an entrance: closed with the object flag, else refused.
    let mut cells = grass(5, 1);
    cell(&mut cells, 3, 2).tile = 0x548;
    assert_eq!(clear(&cells, false, false), Ok(true));
    cell(&mut cells, 1, 1).tile_data |= ENTRANCE;
    assert_eq!(clear(&cells, true, false), Ok(false));
    let mut cells = grass(5, 1);
    cell(&mut cells, 1, 1).tile_data |= ENTRANCE;
    assert_eq!(clear(&cells, true, false), Ok(true));
    assert_eq!(clear(&cells, false, false), Ok(false));
    // Off the map: the ring round a corner is one arc.
    let cells = grass(5, 1);
    assert_eq!(
        outline_clear(&cells, &RING, [0, 0, 0], false, ZONE, false),
        Ok(true)
    );
    assert_eq!(
        outline_clear(&cells, &[], [2, 2, 0], false, ZONE, false),
        Err(ObjectFault::EmptyOutline)
    );
    assert_eq!(
        outline_clear(&cells, &RING, [2, 2, 1], false, ZONE, false),
        Err(ObjectFault::Outside)
    );
}

/// A 1 × 2 object of `class`: its anchor an entrance, the cell above blocked.
fn gate(class: u32) -> ObjectTemplate {
    let mut t = line(class, 0);
    t.passability = MASK48 & !bits(&[[0, 0], [0, 1]]);
    t.actions = 1;
    t.height = 2;
    t
}

/// A 1 × 1 object of `class` whose anchor is an entrance.
fn door(class: u32) -> ObjectTemplate {
    let mut t = line(class, 0);
    t.actions = 1;
    t
}

/// The settings with class 17's +1 and +2 set.
fn gate_settings() -> Vec<ObjectSettings> {
    let mut s = settings();
    s[17].from_above = true;
    s[17].removable = true;
    s
}

// covers: RMG-OBJ-011
/// `can_place`: the footprint, then the ring (computed once, its object flag from the class's
/// settings +1 and +2), then the cell below the entrance: on the map's rows, free, not rock, of
/// the zone (not -1), without an entrance unless the first object on it is removable, water as
/// the zone; without an entrance the ring decides.
#[test]
fn can_place_tests_the_footprint_the_ring_and_below_the_entrance() {
    let lines = [line(5, 0), gate(17), door(17), door(5), gate(5)];
    let mut o = Objects::new(&lines, &gate_settings(), MapFormat::SoD, 2);
    let (post, door5, gate5) = (o.group(5)[0], o.group(5)[1], o.group(5)[2]);
    let (gate, door17) = (o.group(17)[0], o.group(17)[1]);
    let mut cells = grass(6, 1);
    assert!(o.prototypes()[gate].outline.is_empty());
    assert!(o.can_place(&cells, gate, [2, 2, 0], ZONE));
    assert_eq!(o.prototypes()[gate].outline.len(), 10);
    assert!(o.can_place(&cells, gate, [2, 2, 0], ZONE));
    assert!(o.can_place(&cells, post, [2, 2, 0], ZONE));
    // The footprint and the ring refuse.
    assert!(!o.can_place(&cells, gate, [2, 0, 0], ZONE));
    cell(&mut cells, 1, 2).tile_data &= !FREE;
    cell(&mut cells, 3, 2).tile_data &= !FREE;
    assert!(!o.can_place(&cells, post, [2, 2, 0], ZONE));
    // Below the entrance: off the map's rows, not free, rock, of no zone, of another (each
    // closes one cell of the ring, which stays one arc).
    let cells = grass(6, 1);
    assert!(!o.can_place(&cells, gate, [2, 5, 0], ZONE));
    let below: [fn(&mut Cell); 4] = [
        |c| c.tile_data &= !FREE,
        |c| c.tile = (c.tile & !0x3F) | 9,
        |c| c.zone |= 0x00FF_0000,
        |c| c.zone = (c.zone & 0xFF00_FFFF) | 0x1_0000,
    ];
    for mark in below {
        let mut cells = grass(6, 1);
        mark(cell(&mut cells, 2, 3));
        assert!(!o.can_place(&cells, gate, [2, 2, 0], ZONE));
    }
    // A zone of id -1: the footprint and the ring of its cells pass, below the entrance not.
    let mut none = grass(6, 1);
    for c in 0..36 {
        cell(&mut none, c % 6, c / 6).zone |= 0x00FF_0000;
    }
    let minus = PlaceZone {
        id: -1,
        water: false,
    };
    assert!(!o.can_place(&none, gate, [2, 2, 0], minus));
    assert!(o.can_place(&none, post, [2, 2, 0], minus));
    // Water below the entrance: refused for a land zone, taken for a water zone (whose ring is
    // that one cell open).
    let mut wet = grass(6, 1);
    set_land(&mut wet, 2, 3, 8);
    assert!(!o.can_place(&wet, gate, [2, 2, 0], ZONE));
    let water = PlaceZone { id: 0, water: true };
    assert!(o.can_place(&wet, gate, [2, 2, 0], water));
    // An object's entrance below: a removable one's passes, another's not; on the ring it is
    // closed under the object flag (class 17's), refused at once without (class 5's).
    let mut cells = grass(6, 1);
    o.add(&mut cells, door17, [2, 3, 0]);
    assert!(o.can_place(&cells, gate, [2, 2, 0], ZONE));
    assert!(!o.can_place(&cells, gate5, [2, 2, 0], ZONE));
    let mut cells = grass(6, 1);
    o.add(&mut cells, door5, [2, 3, 0]);
    assert!(!o.can_place(&cells, gate, [2, 2, 0], ZONE));
    assert_eq!(o.take_fault(), None);
}

// covers: RMG-OBJ-011
/// `can_place`'s faults refuse and are kept: an empty outline, the cell below an entrance off
/// the map, an entrance there without an object listed.
#[test]
fn can_place_faults_refuse() {
    let mut open = line(5, 0);
    open.passability = MASK48;
    let mut far = line(5, 0);
    far.passability = MASK48 & !bits(&[[0, 0], [3, 0]]);
    far.actions = bits(&[[3, 0]]);
    let mut o = Objects::new(&[open, far, gate(17)], &gate_settings(), MapFormat::SoD, 1);
    let (open, far, gate) = (o.group(5)[0], o.group(5)[1], o.group(17)[0]);
    let cells = grass(6, 1);
    assert!(!o.can_place(&cells, open, [2, 2, 0], ZONE));
    assert_eq!(o.take_fault(), Some(ObjectFault::EmptyOutline));
    // A level the map has not.
    assert!(!o.can_place(&cells, gate, [2, 2, 1], ZONE));
    assert_eq!(o.take_fault(), Some(ObjectFault::Outside));
    // The entrance three columns left of a 1 × 1 footprint at x 2: below it off the map.
    assert!(!o.can_place(&cells, far, [2, 2, 0], ZONE));
    assert_eq!(o.take_fault(), Some(ObjectFault::Outside));
    let mut cells = grass(6, 1);
    cell(&mut cells, 2, 3).tile_data |= ENTRANCE;
    assert!(!o.can_place(&cells, gate, [2, 2, 0], ZONE));
    assert_eq!(o.take_fault(), Some(ObjectFault::NoObject));
    assert_eq!(o.take_fault(), None);
}

#[test]
fn the_neighbours_are_the_originals() {
    assert_eq!(NEIGHBOURS[0], [1, 0]);
    assert_eq!(NEIGHBOURS[6], [0, -1]);
    assert!(
        NEIGHBOURS
            .iter()
            .enumerate()
            .all(|(i, [dx, dy])| (dx.abs() + dy.abs() == 2) == (i % 2 == 1))
    );
}
