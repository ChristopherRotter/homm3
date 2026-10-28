use super::*;
use alloc::string::ToString;
use alloc::vec;

use h3_assets::build;
use h3_assets::def::Encoding;

use crate::batch::tests::{def, raw_frame};

/// A palette whose entry i is the 565 pixel i (green i >> 5, blue i & 31).
fn pixels() -> Palette {
    let mut p = Palette([Rgb::default(); 256]);
    for (i, c) in p.0.iter_mut().enumerate() {
        let i = u8::try_from(i).unwrap();
        *c = Rgb {
            r: 0,
            g: (i >> 5) << 2,
            b: (i & 31) << 3,
        };
    }
    p
}

/// The players' palette: player p's 32 entries all the 565 pixel 0xF800 + p.
fn players() -> Palette {
    let mut p = Palette([Rgb::default(); 256]);
    for (i, c) in p.0.iter_mut().enumerate() {
        *c = Rgb {
            r: 255,
            g: 0,
            b: u8::try_from(i / 32).unwrap() << 3,
        };
    }
    p
}

fn piece(pass: Pass, def: &str, frame: u32, x: i32, y: i32, clip: [i32; 4]) -> Draw {
    Draw::Piece {
        pass,
        def: def.to_string(),
        group: 0,
        frame,
        x,
        y,
        mirror_x: false,
        mirror_y: false,
        clip,
        flag: None,
    }
}

/// A 4 × 2 batch: the frame picture (indices 1 and 224 alternating), the players' palette, a
/// 2 × 2 terrain of frames 0 (1, 2, 3, 4) and 1 (5, 6, 7, 8), a shroud of runs of 0 and 3, a gem.
fn batch() -> Batch {
    let mut batch = Batch::default();
    batch
        .add(
            "Frame.pcx",
            &build::pcx_indexed(4, 2, &[1, 224, 1, 224, 1, 224, 1, 224], &pixels()),
        )
        .unwrap();
    batch
        .add("Players.pal", &build::pal(&players(), &[0; 256], 0, &[]))
        .unwrap();
    let terrain = [
        raw_frame(2, 2, &[1, 2, 3, 4]),
        raw_frame(2, 2, &[5, 6, 7, 8]),
    ];
    batch.add("terrain.def", &def(&pixels(), &terrain)).unwrap();
    let mut shroud = raw_frame(2, 2, &[9, 0, 3, 3]);
    shroud.encoding = Encoding::PackedRows;
    shroud.runs = vec![false, true, true, true];
    batch.add("shroud.def", &def(&pixels(), &[shroud])).unwrap();
    let mut gem = raw_frame(2, 1, &[0, 11]);
    gem.encoding = Encoding::Rows;
    gem.runs = vec![true, false];
    batch
        .add("gem.def", &def(&pixels(), &[gem.clone(), gem]))
        .unwrap();
    batch
}

fn scene(draws: Vec<Draw>) -> Scene {
    Scene {
        draws,
        cycles: vec![],
        flags: "flags.pal".to_string(),
        bugs: h3_render::Bugs::Fixed,
    }
}

fn frame_of_viewer(viewer: u8) -> Draw {
    Draw::Frame {
        picture: "frame.pcx".to_string(),
        players: "PLAYERS.PAL".to_string(),
        viewer,
    }
}

const ALL: [i32; 4] = [0, 0, 4, 2];

#[test]
fn the_frame_in_the_viewers_colours() {
    let screen = compose(&scene(vec![frame_of_viewer(3)]), &batch(), [4, 2]).unwrap();
    assert_eq!(
        screen.pixels(),
        [1, 0xF803, 1, 0xF803, 1, 0xF803, 1, 0xF803]
    );
    // A 24-bit picture has no palette: drawn as it is.
    let mut b = batch();
    let white = Rgb {
        r: 255,
        g: 255,
        b: 255,
    };
    b.add("frame.pcx", &build::pcx_rgb(1, 1, &[white])).unwrap();
    let screen = compose(&scene(vec![frame_of_viewer(3)]), &b, [1, 1]).unwrap();
    assert_eq!(screen.pixels(), [0xFFFF]);
}

#[test]
fn the_pieces_through_the_raw_drawer_and_the_shroud_twice() {
    let list = vec![
        frame_of_viewer(0),
        piece(Pass::Terrain, "TERRAIN.def", 1, 0, 0, [0, 0, 2, 2]),
        piece(Pass::Edge, "terrain.def", 0, 2, 0, [2, 0, 3, 2]),
        piece(Pass::Shroud, "shroud.def", 0, 0, 0, ALL),
    ];
    let screen = compose(&scene(list), &batch(), [4, 2]).unwrap();
    let f = FORMAT;
    // Frame 1 at (0, 0); frame 0 at (2, 0) clipped to its first column; then the shroud: its
    // literal 9 over (0, 0), its run of 0 left, its runs of 3 halve the row below.
    assert_eq!(
        screen.pixels(),
        [
            9,
            6,
            1,
            0xF800,
            f.shadow_body(7),
            f.shadow_body(8),
            3,
            0xF800
        ]
    );
}

#[test]
fn the_paths_shadow_alone_then_its_pixels_alone() {
    // E24 M1 T5b2: pass 6 draws an arrow's runs as shadows only, pass 8 its pixels only.
    let terrain = piece(Pass::Terrain, "terrain.def", 1, 0, 0, [0, 0, 2, 2]);
    let shadow = piece(Pass::PathShadow, "shroud.def", 0, 0, 0, ALL);
    let screen = compose(&scene(vec![terrain.clone(), shadow]), &batch(), [2, 2]).unwrap();
    let f = FORMAT;
    // The literal 9 and the run of 0 leave frame 1's pixels; the runs of 3 halve the row below.
    assert_eq!(screen.pixels(), [5, 6, f.shadow_body(7), f.shadow_body(8)]);
    let arrow = piece(Pass::Path, "shroud.def", 0, 0, 0, ALL);
    let screen = compose(&scene(vec![terrain, arrow]), &batch(), [2, 2]).unwrap();
    // The literal 9 alone.
    assert_eq!(screen.pixels(), [9, 6, 7, 8]);
}

#[test]
fn the_lifted_hero_once_more_blended() {
    // E24 M2 T3b: pass 9 blends its literal pixels with the screen and skips its runs.
    let terrain = piece(Pass::Terrain, "terrain.def", 1, 0, 0, [0, 0, 2, 2]);
    let whole = piece(Pass::Blend, "shroud.def", 0, 0, 0, ALL);
    let screen = compose(&scene(vec![terrain, whole]), &batch(), [2, 2]).unwrap();
    let half = FORMAT.half_mask();
    assert_eq!(
        screen.pixels(),
        [((9 >> 1) & half) + ((5 >> 1) & half), 6, 7, 8]
    );
}

#[test]
fn the_objects_through_the_flagged_drawer_and_the_shadow_pass() {
    // An object's 2 × 2 frame: a literal 9, a flag's run of 5, a shadow's runs of 1 and 4.
    let mut b = batch();
    let mut object = raw_frame(2, 2, &[9, 5, 1, 4]);
    object.encoding = Encoding::PackedRows;
    object.runs = vec![false, true, true, true];
    b.add("object.def", &def(&pixels(), &[object])).unwrap();
    let list = vec![
        piece(Pass::Terrain, "terrain.def", 0, 0, 0, [0, 0, 2, 2]),
        piece(Pass::Flat, "object.def", 0, 2, 0, [2, 0, 4, 2]),
        piece(Pass::Shadow, "object.def", 0, 0, 0, [0, 0, 2, 2]),
        piece(Pass::Body, "object.def", 0, 0, 0, [0, 0, 2, 2]),
    ];
    let screen = compose(&scene(list), &b, [4, 2]).unwrap();
    let f = FORMAT;
    // The flat piece at (2, 0): its literal 9 only (the flag skipped, the shadow's runs too);
    // the shadow darkens the terrain's 3 and 4, the body then puts its 9 over the terrain's 1.
    assert_eq!(
        screen.pixels(),
        [9, 2, 9, 0, f.shadow_border(3), f.shadow_body(4), 0, 0]
    );
}

#[test]
fn an_owned_objects_flag_pixels_in_its_entry_of_the_flag_palette() {
    // The object's frame as above; the flag palette's entry 64 is the 565 pixel 64, entry 0 is 0.
    let mut b = batch();
    let mut object = raw_frame(2, 2, &[9, 5, 1, 4]);
    object.encoding = Encoding::PackedRows;
    object.runs = vec![false, true, true, true];
    b.add("object.def", &def(&pixels(), &[object])).unwrap();
    b.add("flags.pal", &build::pal(&pixels(), &[0; 256], 0, &[]))
        .unwrap();
    let flagged = |flag: Option<u8>, pass: Pass| with_flag(pass, 0, flag);
    let list = vec![
        piece(Pass::Terrain, "terrain.def", 0, 0, 0, [0, 0, 2, 2]),
        piece(Pass::Terrain, "terrain.def", 0, 2, 0, [2, 0, 4, 2]),
        flagged(Some(64), Pass::Body),
        with_flag(Pass::Flat, 2, Some(64)),
    ];
    let screen = compose(&scene(list), &b, [4, 2]).unwrap();
    // The flag's run takes the entry's colour in both passes; the shadow's runs are skipped.
    assert_eq!(screen.pixels(), [9, 64, 9, 64, 3, 4, 3, 4]);
    // An entry whose colour is 0 is none: the run is skipped (0x47D0A0).
    let list = vec![
        piece(Pass::Terrain, "terrain.def", 0, 0, 0, [0, 0, 2, 2]),
        flagged(Some(0), Pass::Body),
    ];
    let screen = compose(&scene(list), &b, [4, 2]).unwrap();
    assert_eq!(&screen.pixels()[..2], [9, 2]);
    // Without the flag palette in the batch, a piece with an entry does not compose.
    assert_eq!(
        compose(
            &scene(vec![flagged(Some(64), Pass::Flat)]),
            &batch_with_object(),
            [4, 2]
        )
        .unwrap_err()
        .to_string(),
        "not in the batch: flags.pal"
    );
}

/// The object's piece of `pass` at (`x`, 0), clipped to its 2 × 2, with the flag entry `flag`.
fn with_flag(pass: Pass, x: i32, flag: Option<u8>) -> Draw {
    Draw::Piece {
        pass,
        def: "object.def".to_string(),
        group: 0,
        frame: 0,
        x,
        y: 0,
        mirror_x: false,
        mirror_y: false,
        clip: [x, 0, x + 2, 2],
        flag,
    }
}

/// The test batch with the object of the flag test and no flag palette.
fn batch_with_object() -> Batch {
    let mut b = batch();
    let mut object = raw_frame(2, 2, &[9, 5, 1, 4]);
    object.encoding = Encoding::PackedRows;
    object.runs = vec![false, true, true, true];
    b.add("object.def", &def(&pixels(), &[object])).unwrap();
    b
}

#[test]
fn the_gems_skip_their_runs() {
    let list = vec![
        frame_of_viewer(0),
        Draw::Gem {
            def: "gem.def".to_string(),
            frame: 1,
            x: 2,
            y: 1,
        },
    ];
    let screen = compose(&scene(list), &batch(), [4, 2]).unwrap();
    assert_eq!(screen.pixels(), [1, 0xF800, 1, 0xF800, 1, 0xF800, 1, 11]);
}

#[test]
fn a_cycling_palette_rotates_its_ranges() {
    let mut s = scene(vec![piece(Pass::Terrain, "terrain.def", 0, 0, 0, ALL)]);
    // Entries 1–4 rotated by 1 (5 steps of a 4-entry range): 1 takes 4's colour, 2 takes 1's.
    s.cycles = vec![Cycle {
        def: "Terrain.DEF".to_string(),
        ranges: vec![[1, 4]],
        steps: 5,
    }];
    let screen = compose(&s, &batch(), [2, 2]).unwrap();
    assert_eq!(screen.pixels(), [4, 1, 2, 3]);
    // Another DEF's cycles leave it.
    s.cycles[0].def = "water.def".to_string();
    let screen = compose(&s, &batch(), [2, 2]).unwrap();
    assert_eq!(screen.pixels(), [1, 2, 3, 4]);
}

#[test]
fn rotates_a_range_up() {
    let mut p = pixels();
    rotate(&mut p, 10, 12, 1);
    let at = |i: usize| FORMAT.pack(p.0[i]);
    assert_eq!([at(9), at(10), at(11), at(12), at(13)], [9, 12, 10, 11, 13]);
    // An empty range, or one backwards, is left.
    let before = pixels();
    let mut p = pixels();
    rotate(&mut p, 12, 10, 3);
    rotate(&mut p, 12, 11, 3);
    assert_eq!(p, before);
}

#[test]
fn refuses_a_frame_too_large_or_a_missing_asset() {
    let b = batch();
    assert_eq!(
        compose(&scene(vec![]), &b, [1 << 15, 1]).unwrap_err(),
        ComposeError::Frame([1 << 15, 1])
    );
    let missing = |draw: Draw| {
        compose(&scene(vec![draw]), &b, [4, 2])
            .unwrap_err()
            .to_string()
    };
    let frame = |picture: &str, players: &str| Draw::Frame {
        picture: picture.to_string(),
        players: players.to_string(),
        viewer: 0,
    };
    assert_eq!(
        missing(frame("x.pcx", "players.pal")),
        "not in the batch: x.pcx"
    );
    assert_eq!(
        missing(frame("frame.pcx", "x.pal")),
        "not in the batch: x.pal"
    );
    assert_eq!(
        missing(piece(Pass::Road, "x.def", 0, 0, 0, ALL)),
        "not in the batch: x.def"
    );
    assert_eq!(
        missing(piece(Pass::River, "terrain.def", 2, 0, 0, ALL)),
        "not in the batch: terrain.def group 0 frame 2"
    );
    let gem = Draw::Gem {
        def: "gem.def".to_string(),
        frame: 9,
        x: 0,
        y: 0,
    };
    assert_eq!(missing(gem), "not in the batch: gem.def group 0 frame 9");
    assert_eq!(
        ComposeError::Frame([5, 6]).to_string(),
        "a 5 x 6 frame is too large"
    );
}

#[test]
fn the_dialogs_items_by_their_own_drawers() {
    // E24 M3 T2a: a dialog's picture over the frame (index 0 transparent, `h3_ui::paint`); an
    // item whose asset the batch lacks is missing.
    let picture = |name: &str| {
        Draw::Dialog(h3_ui::ui::Draw::Picture {
            picture: name.to_string(),
            x: 0,
            y: 0,
            player: None,
            size: Some([1, 1]),
        })
    };
    let list = vec![frame_of_viewer(3), picture("frame.pcx")];
    let screen = compose(&scene(list), &batch(), [4, 2]).unwrap();
    // The frame's picture drawn keyed: its first pixel (index 1) over the frame's own.
    assert_eq!(screen.pixel(0, 0), Some(1));
    assert_eq!(screen.pixel(1, 0), Some(0xF803));
    let list = vec![frame_of_viewer(3), picture("none.pcx")];
    assert_eq!(
        compose(&scene(list), &batch(), [4, 2]),
        Err(ComposeError::Missing("none.pcx".to_string()))
    );
}
