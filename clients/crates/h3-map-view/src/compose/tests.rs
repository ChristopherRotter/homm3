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
        view: None,
        cycles: vec![],
        flags: "flags.pal".to_string(),
        bugs: Bugs::Fixed,
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
    let gem = Draw::Gem {
        def: "x.def".to_string(),
        frame: 0,
        x: 0,
        y: 0,
    };
    assert_eq!(missing(gem), "not in the batch: x.def");
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

#[test]
fn a_kept_frame_draws_each_scene_as_a_fresh_one() {
    // E24 M3 T2b1: the background drawn once while it stays, the rectangle the last scene's and
    // this scene's other draws cover restored from it, the palettes kept for their steps.
    let b = batch();
    let with = |draws: Vec<Draw>, steps: u32| {
        let mut s = scene(draws);
        s.cycles = vec![Cycle {
            def: "terrain.def".to_string(),
            ranges: vec![[1, 4]],
            steps,
        }];
        s
    };
    let left = piece(Pass::Terrain, "terrain.def", 0, 0, 0, [0, 0, 2, 2]);
    let right = piece(Pass::Terrain, "terrain.def", 1, 2, 0, [2, 0, 4, 2]);
    let gem = Draw::Gem {
        def: "gem.def".to_string(),
        frame: 1,
        x: 2,
        y: 1,
    };
    let scenes = [
        with(vec![frame_of_viewer(0), left.clone()], 1),
        // The piece moved: where it was shows the background again.
        with(vec![frame_of_viewer(0), right.clone()], 1),
        // Its cycles' steps changed: its palette made again.
        with(vec![frame_of_viewer(0), right], 2),
        // Nothing but the background: the last scene's rectangle restored; then nothing to.
        with(vec![frame_of_viewer(0)], 2),
        with(vec![frame_of_viewer(0)], 2),
        // A gem, and a frame's picture after a piece: the whole frame.
        with(vec![frame_of_viewer(0), left, gem, frame_of_viewer(1)], 2),
        // Another viewer: the background drawn again.
        with(
            vec![
                frame_of_viewer(2),
                piece(Pass::Shroud, "shroud.def", 0, 0, 0, ALL),
            ],
            2,
        ),
    ];
    let mut composer = Composer::new([4, 2]).unwrap();
    for s in &scenes {
        let fresh = compose(s, &b, [4, 2]).unwrap();
        assert_eq!(composer.compose(s, &b).unwrap(), &fresh);
        assert_eq!(composer.screen(), &fresh);
    }
    // A scene that fails leaves the next to draw everything again.
    let bad = with(
        vec![frame_of_viewer(2), piece(Pass::Road, "x.def", 0, 0, 0, ALL)],
        2,
    );
    assert_eq!(
        composer.compose(&bad, &b),
        Err(ComposeError::Missing("x.def".to_string()))
    );
    let s = &scenes[1];
    assert_eq!(
        composer.compose(s, &b).unwrap(),
        &compose(s, &b, [4, 2]).unwrap()
    );
    assert_eq!(
        Composer::new([1 << 15, 1]).unwrap_err(),
        ComposeError::Frame([1 << 15, 1])
    );
}

/// A batch for the views below: the test batch, an object of two frames (a literal 9 or 10, a
/// flag's run, a shadow's runs) and the flag palette.
fn view_batch() -> Batch {
    let mut b = batch();
    let mut object = raw_frame(2, 2, &[9, 5, 1, 4]);
    object.encoding = Encoding::PackedRows;
    object.runs = vec![false, true, true, true];
    let mut other = object.clone();
    other.pixels[0] = 10;
    b.add("object.def", &def(&pixels(), &[object, other]))
        .unwrap();
    b.add("flags.pal", &build::pal(&pixels(), &[0; 256], 0, &[]))
        .unwrap();
    b
}

/// A 14 × 12 screen's view at `origin` (the screen point of the map's tile (0, 0); 2 × 2 tiles)
/// and moment `t`, clipped to `clip`, as the scenes draw a view: the terrain (two frames by the
/// tile, cycling by `t`), mirrored pieces, the objects' shadows and bodies (their frame by `t`,
/// owned), a lifted hero blended at a place of the screen's own, the shroud, two gems at the
/// corners.
fn view_at(origin: [i32; 2], t: u32, clip: [i32; 4]) -> Scene {
    let mut draws = vec![frame_of_viewer(0)];
    let [ox, oy] = origin;
    let tiles = |f: &mut dyn FnMut(i32, i32, [i32; 4])| {
        let first = |edge: i32, o: i32| (edge - o).div_euclid(2) - 1;
        for my in first(clip[1], oy)..=first(clip[3], oy) + 1 {
            for mx in first(clip[0], ox)..=first(clip[2], ox) + 1 {
                let (px, py) = (ox + 2 * mx, oy + 2 * my);
                if let Some(rect) = meet([px, py, px + 2, py + 2], clip) {
                    f(mx, my, rect);
                }
            }
        }
    };
    let piece_at = |pass, def: &str, frame, mx: i32, my: i32, rect, mirror: bool| Draw::Piece {
        pass,
        def: def.to_string(),
        group: 0,
        frame,
        x: ox + 2 * mx,
        y: oy + 2 * my,
        mirror_x: mirror,
        mirror_y: mirror && mx % 2 == 0,
        clip: rect,
        flag: (pass == Pass::Body).then_some(64),
    };
    tiles(&mut |mx, my, rect| {
        let frame = u32::try_from((mx * 3 + my * 5).rem_euclid(2)).unwrap();
        let mirror = (mx + my).rem_euclid(3) == 0;
        draws.push(piece_at(
            Pass::Terrain,
            "terrain.def",
            frame,
            mx,
            my,
            rect,
            mirror,
        ));
    });
    let object = |mx: i32, my: i32| mx.rem_euclid(4) == 1 && my.rem_euclid(3) == 0;
    tiles(&mut |mx, my, rect| {
        if object(mx, my) {
            draws.push(piece_at(Pass::Shadow, "object.def", 0, mx, my, rect, false));
        }
    });
    tiles(&mut |mx, my, rect| {
        if object(mx, my) {
            let frame = (t + u32::try_from(mx.rem_euclid(2)).unwrap()) % 2;
            draws.push(piece_at(
                Pass::Body,
                "object.def",
                frame,
                mx,
                my,
                rect,
                false,
            ));
        }
    });
    if let Some(rect) = meet([6, 6, 8, 8], clip) {
        draws.push(piece(Pass::Blend, "shroud.def", 0, 6, 6, rect));
    }
    tiles(&mut |mx, my, rect| {
        if (mx + 2 * my).rem_euclid(7) == 0 {
            draws.push(piece_at(Pass::Shroud, "shroud.def", 0, mx, my, rect, false));
        }
    });
    for (x, y) in [(0, 2), (12, 10)] {
        draws.push(Draw::Gem {
            def: "gem.def".to_string(),
            frame: 1,
            x,
            y,
        });
    }
    let mut s = scene(draws);
    s.view = Some(ViewPlace {
        clip,
        origin,
        tile: 2,
    });
    s.cycles = vec![Cycle {
        def: "TERRAIN.DEF".to_string(),
        ranges: vec![[1, 8]],
        steps: t / 3,
    }];
    s
}

/// Composes `scenes` in turn with one composer: each frame as a fresh one's, every pixel that
/// changed inside the rectangles the composer reports.
fn as_fresh(scenes: &[Scene], b: &Batch, frame: [u32; 2]) {
    let mut composer = Composer::new(frame).unwrap();
    let mut before = Screen::new(frame[0], frame[1], FORMAT).unwrap();
    for (n, s) in scenes.iter().enumerate() {
        let fresh = compose(s, b, frame).unwrap();
        // The scenes given in turn by reference and kept.
        let got = if n % 2 == 0 {
            composer.compose(s, b).unwrap().clone()
        } else {
            composer.compose_owned(s.clone(), b).unwrap().clone()
        };
        assert_eq!(got, fresh, "scene {n}");
        let width = usize::try_from(frame[0]).unwrap();
        for (i, (p, q)) in got.pixels().iter().zip(before.pixels()).enumerate() {
            let (x, y) = (
                i32::try_from(i % width).unwrap(),
                i32::try_from(i / width).unwrap(),
            );
            let inside = composer
                .damage()
                .iter()
                .any(|r| r[0] <= x && x < r[2] && r[1] <= y && y < r[3]);
            assert!(
                p == q || inside,
                "scene {n}: ({x}, {y}) changed outside the damage"
            );
        }
        before = got;
    }
}

#[test]
fn a_view_draws_only_what_changed_and_keeps_what_a_move_keeps() {
    // E24 M3 T2b2: the clip below the frame's picture, on black (the background one colour
    // there: the clip's pixels may move).
    let clip = [1, 3, 13, 11];
    let moves = [
        ([0, 0], 0),
        // Idle: the objects' frames and then the terrain's cycle change.
        ([0, 0], 1),
        ([0, 0], 2),
        ([0, 0], 3),
        ([0, 0], 3),
        // Scrolls by a pixel, by more than a tile, both ways; a walk's steps.
        ([1, 0], 3),
        ([3, -1], 4),
        ([3, -1], 4),
        ([-2, 4], 4),
        ([-3, 4], 5),
        ([-3, 3], 5),
        ([-4, 2], 6),
        // A jump: as large as the clip, and anywhere.
        ([10, 2], 6),
        ([-40, 31], 7),
        ([-40, 32], 7),
    ];
    let scenes: Vec<Scene> = moves.iter().map(|&(o, t)| view_at(o, t, clip)).collect();
    as_fresh(&scenes, &view_batch(), [14, 12]);
    // The same over the frame's picture: the background not one colour across the clip, so no
    // pixel moves; a view moved draws again what moved.
    let scenes: Vec<Scene> = moves
        .iter()
        .map(|&(o, t)| view_at(o, t, [0, 0, 14, 12]))
        .collect();
    as_fresh(&scenes, &view_batch(), [14, 12]);
    // A clip past the screen's sides, a gem off it and a piece with nothing to clip: what meets a
    // cell only off the screen draws nothing.
    let scenes: Vec<Scene> = moves
        .iter()
        .map(|&(o, t)| {
            let mut s = view_at(o, t, [-1, 3, 15, 13]);
            s.draws.push(Draw::Gem {
                def: "gem.def".to_string(),
                frame: 0,
                x: 14,
                y: 9,
            });
            s.draws
                .push(piece(Pass::Path, "shroud.def", 0, 4, 4, [4, 4, 4, 6]));
            s
        })
        .collect();
    as_fresh(&scenes, &view_batch(), [14, 12]);
}

#[test]
fn a_drawable_frame_by_its_group_and_index() {
    let b = batch();
    let terrain = b.drawable("Terrain.def").unwrap();
    let (frame, spans) = terrain.frame(0, 1).unwrap();
    assert_eq!(
        (frame.pixels.as_slice(), spans),
        (&[5, 6, 7, 8][..], &draw::Spans::of(frame))
    );
    assert!(terrain.frame(9, 0).is_none());
    assert!(terrain.frame(0, 2).is_none());
    assert!(b.drawable("none.def").is_none());
}

#[test]
fn the_cells_comparisons() {
    let gem = Draw::Gem {
        def: "gem.def".to_string(),
        frame: 0,
        x: 0,
        y: 0,
    };
    let terrain = piece(Pass::Terrain, "terrain.def", 0, 0, 0, ALL);
    // A piece is no gem; a gem moved is the gem moved.
    assert!(!same_moved(&terrain, &gem, [0, 0]));
    let moved = Draw::Gem {
        def: "gem.def".to_string(),
        frame: 0,
        x: 2,
        y: -1,
    };
    assert!(same_moved(&moved, &gem, [2, -1]));
    // A frame's picture or a dialog's item counts as changed whatever cycles.
    assert!(uses(&frame_of_viewer(0), &[], [&[], &[]]));
    assert!(!uses(&gem, &["terrain.def".to_string()], [&[], &[]]));
    assert!(uses(&terrain, &["TERRAIN.DEF".to_string()], [&[], &[]]));
    // The minimap's terrain counts only where a cycle (the last scene's or this one's) rotates
    // its entries (8 and 9 here): not 10 to 20, nor another DEF's.
    use Colour::Open;
    let map = minimap([Open(0); 4]);
    let terrain_def = ["terrain.def".to_string()];
    let cycle = |def: &str, ranges: Vec<[u8; 2]>| Cycle {
        def: def.to_string(),
        ranges,
        steps: 1,
    };
    let past = [
        cycle("TERRAIN.DEF", vec![[10, 20]]),
        cycle("other.def", vec![[1, 9]]),
    ];
    assert!(!uses(&map, &terrain_def, [&past, &[]]));
    assert!(uses(
        &map,
        &terrain_def,
        [&[], &[cycle("terrain.def", vec![[1, 8]])]]
    ));
    assert!(!uses(
        &map,
        &[],
        [&[], &[cycle("terrain.def", vec![[1, 8]])]]
    ));
    // It stays where it is when the view moves.
    assert!(same_moved(&map, &map, [2, -1]));
    assert!(stays_put(&map) && !stays_put(&gem));
    // A gem the batch lacks does not draw.
    let b = batch();
    let lacking = scene(vec![]);
    let mut kept = BTreeMap::new();
    let mut s = Screen::new(2, 2, FORMAT).unwrap();
    let mut pen = Pen::new(&lacking, &b, &mut kept);
    let none = Draw::Gem {
        def: "none.def".to_string(),
        frame: 0,
        x: 0,
        y: 0,
    };
    assert_eq!(
        pen.draw(&mut s, &none, None),
        Err(ComposeError::Missing("none.def".to_string()))
    );
    // The minimap and its box cut to a rectangle they do not meet, or off the screen, draw
    // nothing.
    let b = minimap_batch();
    let mut pen = Pen::new(&lacking, &b, &mut kept);
    let mut s = Screen::new(14, 12, FORMAT).unwrap();
    let radar = radar_at([11, 1], [0, 0]);
    pen.draw(&mut s, &radar, Some([0, 0, 2, 2])).unwrap();
    pen.draw(&mut s, &map, Some([20, 20, 22, 22])).unwrap();
    assert!(s.pixels().iter().all(|&p| p == 0));
    // Whatever they take missing from the batch: the box's DEF, a blocked tile's terrain, the
    // flag palette.
    let mut pen = Pen::new(&lacking, &b, &mut kept);
    let mut nameless = radar_at([11, 1], [0, 0]);
    if let Draw::Radar { def, .. } = &mut nameless {
        *def = "none.def".to_string();
    }
    assert_eq!(
        pen.draw(&mut s, &nameless, None),
        Err(ComposeError::Missing("none.def".to_string()))
    );
    let blocked = minimap([Colour::Blocked(3); 4]);
    assert_eq!(
        pen.draw(&mut s, &blocked, None),
        Err(ComposeError::Missing(String::new()))
    );
    let owned = minimap([Colour::Owner(64); 4]);
    let plain = batch();
    let mut pen = Pen::new(&lacking, &plain, &mut kept);
    assert_eq!(
        pen.draw(&mut s, &owned, None),
        Err(ComposeError::Missing("flags.pal".to_string()))
    );
    // A clip off the screen is one colour; a screen of two colours across it is not.
    let mut s = Screen::new(2, 2, FORMAT).unwrap();
    assert!(one_colour(&s, [5, 5, 9, 9]));
    assert!(one_colour(&s, ALL));
    s.set(1, 1, 7);
    assert!(!one_colour(&s, ALL));
}

#[test]
fn a_view_draws_everything_again_when_the_rest_changes() {
    let clip = [1, 3, 13, 11];
    let b = view_batch();
    let mut flags = view_at([1, 1], 0, clip);
    flags.flags = "other.pal".to_string();
    let mut b2 = b.clone();
    b2.add("other.pal", &build::pal(&players(), &[0; 256], 0, &[]))
        .unwrap();
    let mut picture = view_at([1, 1], 0, clip);
    picture.draws.push(frame_of_viewer(1));
    let mut placeless = view_at([1, 1], 1, clip);
    placeless.view = None;
    let mut other_clip = view_at([2, 1], 1, clip);
    other_clip.view = Some(ViewPlace {
        clip: [1, 3, 13, 10],
        origin: [2, 1],
        tile: 2,
    });
    let mut other_viewer = view_at([2, 1], 1, clip);
    other_viewer.draws[0] = frame_of_viewer(2);
    let scenes = [
        view_at([0, 0], 0, clip),
        // The flag palette another: every piece again.
        flags.clone(),
        view_at([1, 1], 0, clip),
        // A frame's picture after the view: the whole frame.
        picture,
        view_at([1, 1], 0, clip),
        // A scene without the view's place: its cells at (0, 0), nothing moves.
        placeless,
        view_at([1, 1], 1, clip),
        // Another clip: nothing moves.
        other_clip,
        // Another viewer: the background drawn again.
        other_viewer,
        view_at([2, 1], 1, clip),
        // Nothing but the background: what the last scene drew restored.
        scene(vec![frame_of_viewer(0)]),
        view_at([2, 1], 1, clip),
    ];
    as_fresh(&scenes, &b2, [14, 12]);
    // Nothing over the background, then another flag palette: nothing to restore.
    let mut other_flags = scene(vec![frame_of_viewer(0)]);
    other_flags.flags = "other.pal".to_string();
    as_fresh(
        &[scene(vec![frame_of_viewer(0)]), other_flags],
        &b2,
        [14, 12],
    );
    // A view moved whose clip lies off the screen: nothing moves.
    let off = [20, 3, 30, 11];
    as_fresh(
        &[view_at([0, 0], 0, off), view_at([1, 0], 0, off)],
        &b2,
        [14, 12],
    );
    // A scene that fails leaves the next to draw everything again: by reference, kept, after
    // another flag palette.
    let mut composer = Composer::new([14, 12]).unwrap();
    let mut bad = view_at([1, 0], 0, clip);
    bad.draws.push(piece(Pass::Road, "x.def", 0, 0, 0, clip));
    assert!(composer.compose_owned(bad.clone(), &b2).is_err());
    composer.compose(&view_at([0, 0], 0, clip), &b2).unwrap();
    bad.flags = "other.pal".to_string();
    assert!(composer.compose(&bad, &b2).is_err());
    composer.compose(&view_at([0, 0], 0, clip), &b2).unwrap();
    let mut bad = view_at([1, 0], 0, clip);
    bad.draws.push(piece(Pass::Road, "x.def", 0, 0, 0, clip));
    assert_eq!(
        composer.compose(&bad, &b2),
        Err(ComposeError::Missing("x.def".to_string()))
    );
    assert_eq!(composer.damage(), [[0, 0, 14, 12]]);
    let next = view_at([1, 0], 1, clip);
    assert_eq!(
        composer.compose(&next, &b2).unwrap(),
        &compose(&next, &b2, [14, 12]).unwrap()
    );
    assert_eq!(composer.damage(), [[0, 0, 14, 12]]);
}

/// A 2 × 2-tile minimap at (9, 0) of a 14 × 12 screen (columns of 2 pixels, rows of 1 then 2):
/// the terrain's open and blocked colours (entries 8 and 9, which its cycle rotates), an owner's,
/// black.
fn minimap(colours: [Colour; 4]) -> Draw {
    Draw::Minimap(Tiles {
        look: alloc::sync::Arc::new(crate::minimap::Look {
            at: [9, 0],
            size: [4, 3],
            side: 2,
            pattern: Some(crate::minimap::Pattern {
                columns: vec![2],
                rows: vec![1, 2],
            }),
            terrains: vec!["terrain.def".to_string()],
            entries: [8, 9],
        }),
        colours: alloc::sync::Arc::new(colours.to_vec()),
        heroes: vec![],
    })
}

/// The minimap's box at `at`, from its canvas's `cut`, inside the minimap's area.
fn radar_at(at: [i32; 2], cut: [i32; 2]) -> Draw {
    Draw::Radar {
        def: "radar.def".to_string(),
        frame: 0,
        at,
        cut,
        area: [9, 0, 13, 3],
    }
}

/// The view's batch with a 3 × 2 box of literal pixels (index 12) round a run of 0.
fn minimap_batch() -> Batch {
    let mut b = view_batch();
    let mut radar = raw_frame(3, 2, &[12, 0, 12, 12, 12, 12]);
    radar.encoding = Encoding::Rows;
    radar.runs = vec![false, true, false, false, false, false];
    b.add("radar.def", &def(&pixels(), &[radar])).unwrap();
    b
}

#[test]
fn the_minimap_its_tiles_rectangles_and_its_box_cut_to_its_area() {
    use Colour::{Black, Blocked, Open, Owner};
    let b = minimap_batch();
    let s = scene(vec![
        minimap([Open(0), Blocked(0), Owner(64), Black]),
        radar_at([11, 1], [1, 0]),
    ]);
    let screen = compose(&s, &b, [14, 12]).unwrap();
    let px = |x, y| screen.pixel(x, y).unwrap();
    // An owner's colour looked up once a draw, however many tiles take it.
    let owners = compose(
        &scene(vec![minimap([Owner(64), Owner(64), Owner(65), Owner(64)])]),
        &b,
        [14, 12],
    )
    .unwrap();
    assert_eq!(
        [
            owners.pixel(9, 0),
            owners.pixel(11, 0),
            owners.pixel(9, 1),
            owners.pixel(11, 1)
        ],
        [Some(64), Some(64), Some(65), Some(64)]
    );
    // The terrain's entries 8 and 9, the flag palette's 64 (its 565 pixel 64), black.
    assert_eq!([px(9, 0), px(10, 0), px(11, 0), px(12, 0)], [8, 8, 9, 9]);
    assert_eq!([px(9, 1), px(10, 2), px(11, 1), px(12, 2)], [64, 64, 0, 0]);
    // The box from its canvas's column 1, as wide as the DEF less it and less what lies past the
    // area's right edge (13), the cut taken once more (the original's 13 − 11 − 1): one column,
    // its row 0's run skipped.
    assert_eq!([px(11, 1), px(12, 1)], [0, 0]);
    assert_eq!([px(11, 2), px(12, 2)], [12, 0]);
    assert_eq!(rect_of(&s.draws[1], &b, [0, 0, 14, 12]), Ok([11, 1, 12, 3]));
    // A box past the area's bottom keeps the rows above it; a tile past the screen nothing.
    assert_eq!(
        rect_of(&radar_at([9, 2], [0, 0]), &b, [0, 0, 14, 12]),
        Ok([9, 2, 12, 3])
    );
    let mut wide = minimap([Open(0); 4]);
    if let Draw::Minimap(tiles) = &mut wide {
        let mut look = (*tiles.look).clone();
        look.at = [12, 10];
        tiles.look = alloc::sync::Arc::new(look);
    }
    let screen = compose(&scene(vec![wide]), &b, [14, 12]).unwrap();
    assert_eq!(
        [screen.pixel(13, 10), screen.pixel(13, 11)],
        [Some(8), Some(8)]
    );
    // A tile of a code the tables lack, a missing box: not in the batch.
    let mut odd = minimap([Open(3), Black, Black, Black]);
    assert_eq!(
        compose(&scene(vec![odd.clone()]), &b, [14, 12]),
        Err(ComposeError::Missing(String::new()))
    );
    if let Draw::Minimap(tiles) = &mut odd {
        tiles.colours = alloc::sync::Arc::new(vec![Owner(0)]);
    }
    // An owner's entry of colour 0 is black; a tile without a colour too.
    let screen = compose(&scene(vec![odd]), &b, [14, 12]).unwrap();
    assert_eq!(
        [screen.pixel(9, 0), screen.pixel(11, 0)],
        [Some(0), Some(0)]
    );
    assert!(
        compose(
            &scene(vec![radar_at([9, 0], [0, 0])]),
            &view_batch(),
            [14, 12]
        )
        .is_err()
    );
}

#[test]
fn a_kept_frame_draws_the_minimap_again_only_when_it_changes() {
    use Colour::{Blocked, Open, Owner};
    // The view of the test above with the minimap after it: still, the terrain's cycle stepping
    // (its entries 8 and 9 rotate), a tile changing, the box moving, the view scrolling and
    // jumping.
    let clip = [1, 3, 13, 11];
    let steps = [
        ([0, 0], 0, [Open(0), Open(0), Blocked(0), Owner(64)], [9, 0]),
        ([0, 0], 1, [Open(0), Open(0), Blocked(0), Owner(64)], [9, 0]),
        ([0, 0], 3, [Open(0), Open(0), Blocked(0), Owner(64)], [9, 0]),
        (
            [0, 0],
            3,
            [Owner(65), Open(0), Blocked(0), Owner(64)],
            [9, 0],
        ),
        (
            [0, 0],
            3,
            [Owner(65), Open(0), Blocked(0), Owner(64)],
            [10, 1],
        ),
        (
            [1, 0],
            4,
            [Owner(65), Open(0), Blocked(0), Owner(64)],
            [11, 1],
        ),
        (
            [1, 0],
            4,
            [Owner(65), Open(0), Blocked(0), Owner(64)],
            [11, 1],
        ),
        (
            [-40, 31],
            4,
            [Owner(65), Open(0), Blocked(0), Owner(64)],
            [9, 0],
        ),
    ];
    let scenes: Vec<Scene> = steps
        .iter()
        .map(|&(origin, t, colours, at)| {
            let mut s = view_at(origin, t, clip);
            s.draws.push(minimap(colours));
            s.draws.push(radar_at(at, [0, 0]));
            s
        })
        .collect();
    as_fresh(&scenes, &minimap_batch(), [14, 12]);
    // A minimap whose entries no cycle rotates (10 and 11), two rows above the clip: a scroll
    // and a palette step leave its pixels, its box's too.
    let quiet = |origin: [i32; 2], t: u32| {
        let mut s = view_at(origin, t, clip);
        let mut map = minimap([Owner(64), Open(0), Blocked(0), Open(0)]);
        if let Draw::Minimap(tiles) = &mut map {
            let mut look = (*tiles.look).clone();
            look.size = [4, 2];
            look.entries = [10, 11];
            if let Some(p) = look.pattern.as_mut() {
                p.rows = vec![1];
            }
            tiles.look = alloc::sync::Arc::new(look);
        }
        s.draws.push(map);
        s.draws.push(Draw::Radar {
            def: "radar.def".to_string(),
            frame: 0,
            at: [9, 0],
            cut: [0, 0],
            area: [9, 0, 13, 2],
        });
        s
    };
    let b = minimap_batch();
    as_fresh(
        &[quiet([0, 0], 0), quiet([1, 0], 3), quiet([1, 1], 6)],
        &b,
        [14, 12],
    );
    let mut composer = Composer::new([14, 12]).unwrap();
    composer.compose(&quiet([0, 0], 0), &b).unwrap();
    composer.compose(&quiet([1, 0], 3), &b).unwrap();
    assert!(!composer.damage().is_empty());
    assert!(
        composer
            .damage()
            .iter()
            .all(|&r| meet(r, [9, 0, 13, 2]).is_none())
    );
    // Two scenes alike change nothing.
    let mut composer = Composer::new([14, 12]).unwrap();
    composer.compose(&scenes[1], &minimap_batch()).unwrap();
    composer.compose(&scenes[1], &minimap_batch()).unwrap();
    assert!(composer.damage().is_empty());
}

/// The view's clip on the 14 × 12 screen of the panels' tests.
const CLIP: [i32; 4] = [1, 3, 13, 11];

/// A label of `font.fnt` in a 2 × 1 box.
fn label(text: &str) -> h3_ui::ui::Draw {
    h3_ui::ui::Draw::Label {
        item: h3_ui::ui::Label {
            id: 1,
            at: [0, 0],
            size: [2, 1],
            font: "font.fnt".to_string(),
            font_palette: "players.pal".to_string(),
            colour: 0,
            align: 0,
            hint: None,
            repeat: None,
        },
        x: 12,
        y: 0,
        text: text.to_string(),
        disabled: false,
    }
}

/// `view_at`'s scene at `origin` and `t` with a dialog's items after the frame's picture: a button
/// (`terrain.def`'s frame `f`, 2 × 2) at (10, 0), a rectangle darkened over the left gem (0, 2),
/// the frame's picture as a 2 × 2 item at (5, 0) when `picture`, a label of `text` at (12, 0).
fn panel(origin: [i32; 2], t: u32, f: u32, dark: [i32; 4], picture: bool, text: &str) -> Scene {
    let mut items = vec![
        h3_ui::ui::Draw::Button {
            def: "terrain.def".to_string(),
            frame: f,
            x: 10,
            y: 0,
            mirrored: false,
            player: None,
        },
        h3_ui::ui::Draw::Darken { rect: dark },
    ];
    if picture {
        items.push(h3_ui::ui::Draw::Picture {
            picture: "frame.pcx".to_string(),
            x: 5,
            y: 0,
            player: None,
            size: Some([2, 2]),
        });
    }
    items.push(label(text));
    let mut s = view_at(origin, t, CLIP);
    s.draws.splice(1..1, items.into_iter().map(Draw::Dialog));
    s
}

/// The view's batch with the label's font (glyph `1` inks its one pixel).
fn panel_batch() -> Batch {
    let mut b = view_batch();
    let mut glyphs = vec![h3_assets::fnt::Glyph::default(); 256];
    glyphs[usize::from(b'1')].width = 1;
    glyphs[usize::from(b'1')].pixels = vec![0xFF];
    let font = h3_assets::fnt::Font {
        height: 1,
        y_offset: 0,
        glyphs,
    };
    b.add("font.fnt", &build::fnt(&font)).unwrap();
    b.add("other.pal", &build::pal(&players(), &[0; 256], 0, &[]))
        .unwrap();
    b
}

const DARK: [i32; 4] = [0, 1, 3, 2];

#[test]
fn a_changed_dialog_item_draws_its_rectangle_alone() {
    // E24 M3 T4c: the background drawn again only where its changed items reach (as they were and
    // as they are), the view's cells there again; everything again when an item's reach is
    // unknown (a label), meets the view's clip, or a dialog's item lies over the view.
    let b = panel_batch();
    let mut flags = panel([1, 0], 1, 1, DARK, true, "1");
    flags.flags = "other.pal".to_string();
    let mut over = panel([1, 0], 1, 1, DARK, true, "1");
    over.draws
        .push(Draw::Dialog(h3_ui::ui::Draw::Darken { rect: [4, 4, 2, 2] }));
    let mut relabelled = panel([1, 0], 2, 0, DARK, true, "1");
    relabelled.draws[3] = Draw::Dialog(label("1"));
    let scenes = [
        panel([0, 0], 0, 0, DARK, true, "1"),
        // The button's frame, with a scroll.
        panel([1, 0], 1, 1, DARK, true, "1"),
        // The darkened rectangle narrower: its old and new reach joined.
        panel([1, 0], 1, 1, [0, 1, 2, 2], true, "1"),
        // An item fewer, then the button and an item more: those between the lists' common start
        // and end.
        panel([1, 0], 2, 1, [0, 1, 2, 2], false, "1"),
        panel([1, 0], 2, 0, DARK, true, "1"),
        // A darkened rectangle reaching the clip, then nothing changed.
        panel([1, 0], 2, 0, [0, 1, 3, 3], true, "1"),
        panel([1, 0], 2, 0, [0, 1, 3, 3], true, "1"),
        // The label's text: its reach unknown; the picture a label (the picture's reach known,
        // the label's not); an item fewer, a label among those between the lists' start and end.
        panel([1, 0], 2, 0, DARK, true, "11"),
        relabelled,
        panel([1, 0], 2, 0, DARK, false, "1"),
        // Another flag palette: what the view drew and the items' reach restored, the view
        // drawn whole.
        flags,
        panel([1, 0], 1, 0, DARK, true, "1"),
        // The button's frame and a dialog's item over the view.
        over,
        panel([1, 0], 1, 0, DARK, true, "1"),
    ];
    as_fresh(&scenes, &b, [14, 12]);
    // The button's frame alone: its 2 × 2 rectangle (the view's cells are its tiles), nothing
    // else.
    let mut composer = Composer::new([14, 12]).unwrap();
    composer
        .compose(&panel([0, 0], 0, 0, DARK, true, "1"), &b)
        .unwrap();
    composer
        .compose(&panel([0, 0], 0, 1, DARK, true, "1"), &b)
        .unwrap();
    assert_eq!(composer.damage(), [[10, 0, 12, 2]]);
    // The darkened rectangle over the gem: the gem drawn again over it.
    composer
        .compose(&panel([0, 0], 0, 1, [0, 1, 2, 2], true, "1"), &b)
        .unwrap();
    assert_eq!(composer.damage(), [[0, 1, 4, 4]]);
}

#[test]
fn a_changed_dialog_item_draws_what_the_batch_has() {
    // E24 M3 T4c: the background's draws drawn again where an item changed fail as a fresh
    // scene's would (a batch without the frame's picture, without the label's font); the next
    // scene draws everything again.
    let b = panel_batch();
    let mut lacking = Batch::default();
    let terrain = [
        raw_frame(2, 2, &[1, 2, 3, 4]),
        raw_frame(2, 2, &[5, 6, 7, 8]),
    ];
    lacking
        .add("terrain.def", &def(&pixels(), &terrain))
        .unwrap();
    let mut gem = raw_frame(2, 1, &[0, 11]);
    gem.encoding = Encoding::Rows;
    gem.runs = vec![true, false];
    lacking
        .add("gem.def", &def(&pixels(), &[gem.clone(), gem]))
        .unwrap();
    let first = panel([0, 0], 0, 0, DARK, false, "1");
    let next = panel([0, 0], 0, 1, DARK, false, "1");
    let mut composer = Composer::new([14, 12]).unwrap();
    composer.compose(&first, &b).unwrap();
    assert_eq!(
        composer.compose(&next, &lacking),
        Err(ComposeError::Missing("frame.pcx".to_string()))
    );
    assert_eq!(
        composer.compose(&next, &b).unwrap(),
        &compose(&next, &b, [14, 12]).unwrap()
    );
    lacking
        .add(
            "Frame.pcx",
            &build::pcx_indexed(4, 2, &[1, 224, 1, 224, 1, 224, 1, 224], &pixels()),
        )
        .unwrap();
    lacking
        .add("Players.pal", &build::pal(&players(), &[0; 256], 0, &[]))
        .unwrap();
    assert_eq!(
        composer.compose(&first, &lacking),
        Err(ComposeError::Missing("font.fnt".to_string()))
    );
    assert_eq!(
        composer.compose(&first, &b).unwrap(),
        &compose(&first, &b, [14, 12]).unwrap()
    );
}

#[test]
fn overlapping_areas_are_joined() {
    assert_eq!(joined(vec![]), Vec::<[i32; 4]>::new());
    assert_eq!(
        joined(vec![[0, 0, 2, 2], [5, 5, 6, 6], [1, 1, 3, 3], [2, 0, 6, 1]]),
        [[5, 5, 6, 6], [0, 0, 6, 3]]
    );
    // Touching is not overlapping.
    assert_eq!(
        joined(vec![[0, 0, 2, 2], [2, 0, 4, 2]]),
        [[2, 0, 4, 2], [0, 0, 2, 2]]
    );
}
