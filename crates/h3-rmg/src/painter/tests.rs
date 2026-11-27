//! The terrain painter on small maps (RMG-TER-005 to -018): the picture sets' ranges, the slots'
//! draws, a split water ring mended, the cosmetic flag's two halves (RMG-QUIRK-21) and the
//! faults. The corpus's step 3 against the phase oracle is `h3-pack-rmg`'s.

use alloc::vec;
use alloc::vec::Vec;

use super::*;
use crate::part::PictureSet;

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

/// Sets of one picture per shape, two plain and two decorated on land; rock's per shape and
/// variants.
pub(crate) fn tables() -> GeneratorTables {
    let land = |share| {
        let mut pictures = vec![(0, false), (0, false), (0, true), (0, true)];
        pictures.extend((2..=28).map(|s| (s, false)));
        PictureSet {
            through_dirt: true,
            land: true,
            share,
            pictures,
        }
    };
    let mut water = land(0);
    water.land = false;
    water.through_dirt = false;
    water.pictures.drain(2..4);
    let mut sets = vec![land(50); 8];
    sets[1].through_dirt = false;
    sets.push(water);
    let mut rock = vec![(0, [false; 2])];
    for shape in 2..=28 {
        for v in [[false, false], [true, false], [false, true], [true, true]] {
            rock.push((shape, v));
        }
    }
    GeneratorTables {
        heroes_roe: Vec::new(),
        heroes_expansions: Vec::new(),
        heroes_campaign: Vec::new(),
        map_limits: Vec::new(),
        zone_limits: Vec::new(),
        layout_cos: Vec::new(),
        layout_sin: Vec::new(),
        terrain_towns: Vec::new(),
        native_terrains: Vec::new(),
        picture_sets: sets,
        rock_pictures: rock,
        object_settings: Vec::new(),
    }
}

/// A `side` × `side` map of one level all of terrain `t`, picture 0.
fn map(side: i32, t: u8) -> Cells {
    let mut cells = Cells::new(side, 1);
    for y in 0..side {
        for x in 0..side {
            let c = cells.clamped_mut(x, y, 0);
            c.tile = u32::from(t);
            c.tile_data = 0;
        }
    }
    cells
}

// covers: RMG-TER-006
/// A range's first is its last run's start, its count all of its pictures; rock's by variants.
#[test]
fn the_ranges_are_built_as_the_original() {
    let set = Set::new(
        true,
        true,
        50,
        false,
        vec![
            (0, false, [false; 2]),
            (2, false, [false; 2]),
            (0, false, [false; 2]),
        ],
    );
    assert_eq!(set.range(0), (2, 2));
    assert_eq!(set.range(4), (1, 1));
    let rock = Set::new(
        false,
        false,
        0,
        true,
        vec![(8, false, [true, false]), (8, false, [false, true])],
    );
    assert_eq!(rock.range((8 * 2 + 1) * 2), (0, 1));
    assert_eq!(rock.range((8 * 2) * 2 + 1), (1, 1));
    assert!(rock.fits(0, 8, Some([true, false])) && !rock.fits(0, 8, Some([false, true])));
    assert!(!rock.flagged(0) && !rock.flagged(9));
}

// covers: RMG-TER-007, RMG-TER-008, RMG-TER-010
/// Slot 4 keeps a plain picture and draws twice on land (the flag roll at share × weight / 8),
/// once on water and rock; slot 5 keeps a picture of the shape and the word, rock's clears it;
/// the weight halves per decorated neighbour of the terrain.
#[test]
fn the_slots_draw_as_the_original() {
    let tables = tables();
    let pictures = Pictures::new(&tables);
    let mut cells = map(4, 2);
    let mut log = Log::default();
    let mut stream = Stream::new(3, &mut log);
    let mut p = Painter::new(&pictures, &mut cells, &mut stream, 0, 2, true);
    // A plain picture is kept (the transition pass's test), a decorated one is drawn again.
    assert!(p.pictures.set(2).fits(1, 0, None));
    assert!(!p.pictures.set(2).fits(5, 0, None));
    let drawn = p.draw_plain(2, 4);
    assert!(drawn < 4);
    assert_eq!(p.draw_plain(2, 0) / 2, 0, "weight 0: plain only");
    let _ = p.draw_plain(WATER, 4);
    let _ = p.draw_plain(ROCK, 4);
    assert_eq!(p.draw_shape(2, 5, [true, false], 7), (7, [true, false]));
    let (picture, word) = p.draw_shape(2, 6, [true, true], 7);
    assert_eq!((picture, word), (8, [true, true]));
    let (picture, word) = p.draw_shape(ROCK, 3, [true, false], 0);
    assert_eq!(word, [false; 2]);
    assert!(p.pictures.set(ROCK).fits(picture, 3, Some([true, false])));
    assert_eq!(
        p.draw_shape(ROCK, 3, [true, false], picture),
        (picture, [false; 2])
    );
    // Decorated W and N neighbours of grass: 4 halved twice.
    p.write(0, 1, 2, 2, [false; 2]);
    p.write(1, 0, 2, 3, [false; 2]);
    assert_eq!(p.weight_at(1, 1, 2), 1);
    assert_eq!(p.weight_at(1, 1, WATER), 4);
    assert_eq!(p.finish(), Ok(()));
    let sites: Vec<u32> = log.draws.iter().map(|d| d.0).collect();
    assert_eq!(
        sites[..8],
        [
            SITE_PICTURE_FLAG,
            SITE_PICTURE,
            SITE_PICTURE_FLAG,
            SITE_PICTURE,
            SITE_PICTURE,
            SITE_ROCK_PICTURE,
            SITE_PICTURE_SHAPE,
            SITE_ROCK_PICTURE_SHAPE
        ]
    );
}

// covers: RMG-TER-005, RMG-TER-009, RMG-TER-011, RMG-TER-012, RMG-TER-014
/// A water column three cells long in grass: its middle cell is pinched W-E and its ring split;
/// the finish gives it W (no neighbour pinched, no diagonal water), then mends the ring's two
/// one-diagonal gaps, SW first, and leaves the costliest; the last pass fits every picture.
#[test]
fn a_split_water_ring_is_mended() {
    let tables = tables();
    let pictures = Pictures::new(&tables);
    let mut cells = map(7, 2);
    let mut log = Log::default();
    let mut stream = Stream::new(1, &mut log);
    let mut p = Painter::new(&pictures, &mut cells, &mut stream, 0, WATER, true);
    for y in 2..5 {
        p.add_rect(3, y, 1, 1);
    }
    assert!(p.pinched.contains(&(3, 3)));
    assert!(p.pinched_we(3, 3, WATER) && p.ring_split(3, 3));
    assert_eq!(p.finish(), Ok(()));
    // No water cell is left pinched, nor grass beside the water.
    let mut log = Log::default();
    let mut stream = Stream::new(1, &mut log);
    let check = Painter::new(&pictures, &mut cells, &mut stream, 0, WATER, true);
    for y in 0..7 {
        for x in 0..7 {
            if check.terrain_at(x, y) == WATER {
                assert!(!check.pinched(x, y), "({x}, {y})");
            }
        }
    }
    assert!(log.draws.is_empty());
}

// covers: RMG-TER-012
/// On water a changed cell's neighbours of the terrain are tested again: a pinched one enters
/// set 1, one in it no longer pinched leaves it and is queued.
#[test]
fn water_neighbours_enter_and_leave_the_pinched_set() {
    let tables = tables();
    let pictures = Pictures::new(&tables);
    let mut cells = map(7, 2);
    for (x, y) in [(3, 3), (3, 2), (2, 2), (1, 5)] {
        cells.clamped_mut(x, y, 0).tile = u32::from(WATER);
    }
    let mut log = Log::default();
    let mut stream = Stream::new(1, &mut log);
    let mut p = Painter::new(&pictures, &mut cells, &mut stream, 0, WATER, true);
    // (3, 3) pinched W-E; once (2, 3) is water its ring is one arc: out of set 1.
    p.pinched.insert((3, 3));
    p.set_cell(2, 3);
    assert!(!p.pinched.contains(&(3, 3)));
    // (1, 5), pinched W-E, beside the changed (2, 4): into set 1.
    p.set_cell(2, 4);
    assert!(p.pinched.contains(&(5, 1)));
    assert_eq!(p.finish(), Ok(()));
}

// covers: RMG-TER-014
/// A ring's cheapest gap off the map's edge: its cells off the map are skipped (the top row's NE
/// clamps to E), the rest made the terrain; the costliest gap stays.
#[test]
fn a_gap_off_the_map_is_skipped() {
    let tables = tables();
    let pictures = Pictures::new(&tables);
    let mut cells = map(7, 2);
    for (x, y) in [(3, 0), (3, 1), (4, 1)] {
        cells.clamped_mut(x, y, 0).tile = u32::from(WATER);
    }
    let mut log = Log::default();
    let mut stream = Stream::new(1, &mut log);
    let mut p = Painter::new(&pictures, &mut cells, &mut stream, 0, WATER, true);
    assert!(p.ring_split(3, 0));
    assert_eq!(p.mend_ring(3, 0), 1);
    assert_eq!(p.terrain_at(4, 0), WATER);
    assert_eq!(p.terrain_at(2, 0), 2);
    assert_eq!(p.finish(), Ok(()));
}

// covers: RMG-TER-013, RMG-TER-016
/// RMG-QUIRK-21's painter's half: the original queues the first foreign neighbour per axis and
/// gives group 3's sand corner shape 8 its mirroring inverted; off, both and as matched.
#[test]
fn the_cosmetic_flag_queues_both_and_keeps_group_3s_mirroring() {
    let tables = tables();
    let pictures = Pictures::new(&tables);
    for (cosmetic, queued) in [(true, 2), (false, 4)] {
        let mut cells = map(5, WATER);
        cells.clamped_mut(2, 2, 0).tile = 2;
        let mut log = Log::default();
        let mut stream = Stream::new(1, &mut log);
        let mut p = Painter::new(&pictures, &mut cells, &mut stream, 0, 2, cosmetic);
        p.queue(2, 2);
        let orthogonal = [(1, 2), (3, 2), (2, 1), (2, 3)];
        let n = orthogonal.iter().filter(|q| p.queued.contains(q)).count();
        assert_eq!(n, queued, "{cosmetic}");
        // Water's diagonals are queued too.
        assert!(p.queued.contains(&(1, 1)) && p.queued.contains(&(3, 3)));
    }
    let mut classes = [0u8; 8];
    classes[E] = 2;
    classes[S] = 1;
    classes[SW] = 2;
    assert_eq!(shape(&classes, true), (8, [true, true]));
    assert_eq!(shape(&classes, false), (8, [false, false]));
    assert_eq!(shape(&[0; 8], true), (0, [false; 2]));
    // The search skipped for every class 0 is the table's own answer: each condition asks for
    // a class 1 or 2 of at least one role, so none holds among a cell's own terrain; and every
    // other mix of classes 0 to 2 meets one.
    let mut conditions = GROUPS.iter().flat_map(|group| group.iter());
    assert!(conditions.clone().count() > 0);
    assert!(conditions.all(|(all, _, _)| !all.is_empty() && all.iter().all(|&(_, c)| c > 0)));
    for n in 1..3u32.pow(8) {
        let mut classes = [0u8; 8];
        let mut m = n;
        for c in &mut classes {
            *c = (m % 3) as u8;
            m /= 3;
        }
        for inverting in [true, false] {
            assert_ne!(shape(&classes, inverting), (0, [false; 2]), "{classes:?}");
        }
    }
}

// covers: RMG-TER-015, RMG-TER-017
/// The classes: sand borders nothing, dirt only what does not meet through dirt, the lands take
/// dirt borders from each other; the refinements: a corner's arm cell of the terrain gives 6,
/// a far cell of another 7.
#[test]
fn the_classes_and_refinements_are_the_originals() {
    let tables = tables();
    let pictures = Pictures::new(&tables);
    let mut cells = map(6, 2);
    let mut log = Log::default();
    let mut stream = Stream::new(1, &mut log);
    let p = Painter::new(&pictures, &mut cells, &mut stream, 0, 2, true);
    assert_eq!(p.class(SAND, WATER), 0);
    assert_eq!(p.class(DIRT, 2), 0);
    assert_eq!(p.class(DIRT, WATER), 2);
    assert_eq!(p.class(2, DIRT), 1);
    assert_eq!(p.class(2, SAND), 2);
    assert_eq!(p.class(2, 2), 0);
    assert_eq!(p.refine(2, 2, 2, 2, [false; 2]), 6);
    assert_eq!(p.refine(2, 2, 2, 8, [true, true]), 12);
    assert_eq!(p.refine(2, 2, 2, 5, [false; 2]), 5);
    assert_eq!(p.refine(2, 2, WATER, 11, [false, true]), 13);
    assert_eq!(p.refine(2, 2, WATER, 2, [false; 2]), 2);
    assert_eq!(p.refine(2, 2, 2, 9, [false; 2]), 9);
}

// covers: RMG-TER-018
/// A shape its set lacks draws among none (RMG-QUIRK-3): a typed error.
#[test]
fn a_missing_shape_is_refused() {
    let mut tables = tables();
    tables.picture_sets[2].pictures.truncate(4);
    let pictures = Pictures::new(&tables);
    let mut cells = map(5, WATER);
    let mut log = Log::default();
    let mut stream = Stream::new(1, &mut log);
    let mut p = Painter::new(&pictures, &mut cells, &mut stream, 0, 2, true);
    p.add_rect(1, 1, 2, 2);
    assert_eq!(
        p.finish(),
        Err(PaintFault::NoPicture {
            terrain: 2,
            shape: 8
        })
    );
}

// covers: RMG-TER-005
/// A cell in set 1 the resolution changes nothing at would loop for ever (RMG-QUIRK-4): a typed
/// error.
#[test]
fn a_pinched_cell_that_cannot_be_mended_is_refused() {
    let tables = tables();
    let pictures = Pictures::new(&tables);
    let mut cells = map(5, 2);
    let mut log = Log::default();
    let mut stream = Stream::new(1, &mut log);
    let mut p = Painter::new(&pictures, &mut cells, &mut stream, 0, 2, true);
    p.pinched.insert((2, 2));
    assert_eq!(p.finish(), Err(PaintFault::Stuck));
}

// covers: RMG-TER-005
/// A flush finishes the painter's terrain (its sets drained, the last pass run) and goes on
/// with another at weight 4: the cells and draws of a painter finished and a new one made; a
/// fault met before it is the finish's.
#[test]
fn a_flush_finishes_one_terrain_and_paints_on_with_another() {
    let tables = tables();
    let pictures = Pictures::new(&tables);
    let run = |flush: bool| {
        let mut cells = map(8, WATER);
        let mut log = Log::default();
        let mut stream = Stream::new(4, &mut log);
        let mut p = Painter::new(&pictures, &mut cells, &mut stream, 0, 2, true);
        p.add_rect(1, 1, 3, 2);
        p.weight = 1;
        if flush {
            p.flush(3);
            assert_eq!((p.terrain, p.weight), (3, WEIGHT));
        } else {
            assert_eq!(p.finish(), Ok(()));
            p = Painter::new(&pictures, &mut cells, &mut stream, 0, 3, true);
        }
        p.add_rect(3, 4, 3, 2);
        assert_eq!(p.finish(), Ok(()));
        (cells, log.draws)
    };
    let (cells, draws) = run(true);
    assert_eq!((&cells, &draws), (&run(false).0, &run(false).1));
    assert!(cells.cells().iter().any(|c| terrain_of(c) == 2));
    assert!(cells.cells().iter().any(|c| terrain_of(c) == 3));
    let mut bare = tables.clone();
    bare.picture_sets[2].pictures.clear();
    let pictures = Pictures::new(&bare);
    let mut cells = map(8, WATER);
    let mut log = Log::default();
    let mut stream = Stream::new(4, &mut log);
    let mut p = Painter::new(&pictures, &mut cells, &mut stream, 0, 2, true);
    p.add_rect(1, 1, 3, 2);
    p.flush(3);
    p.add_rect(3, 4, 3, 2);
    assert_eq!(
        p.finish(),
        Err(PaintFault::NoPicture {
            terrain: 2,
            shape: 0
        })
    );
}
