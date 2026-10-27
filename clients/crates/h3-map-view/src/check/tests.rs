use super::*;

use h3_assets::build;
use h3_assets::def::Frame;
use h3_assets::palette::{Palette, Rgb};

/// A 4 × 2 frame of `pixels` in `encoding`, its runs where the encoder would repeat.
fn frame(encoding: Encoding, pixels: [u8; 8]) -> Frame {
    Frame {
        encoding,
        full_width: 4,
        full_height: 2,
        left: 0,
        top: 0,
        width: 4,
        height: 2,
        runs: build::runs_by_repetition(encoding, 4, &pixels),
        pixels: pixels.to_vec(),
        short_header: false,
    }
}

fn def(groups: &[(u32, &[(&str, usize)])], frames: &[Frame]) -> Vec<u8> {
    build::def(
        0x45,
        (4, 2),
        &Palette([Rgb::default(); 256]),
        groups,
        frames,
    )
}

#[test]
fn counts_group_0s_frames_encodings_and_runs() {
    let frames = [
        frame(Encoding::Raw, [9; 8]),
        frame(Encoding::PackedRows, [0, 0, 9, 8, 5, 5, 5, 9]),
        frame(Encoding::Rows, [200, 200, 9, 8, 7, 7, 1, 2]),
    ];
    let bytes = def(
        &[(0, &[("a", 0), ("b", 1), ("c", 2)]), (1, &[("d", 0)])],
        &frames,
    );
    let summary = summarize(&bytes).unwrap();
    assert_eq!(summary.frames, 3);
    assert_eq!(
        summary.encodings.iter().copied().collect::<Vec<_>>(),
        [Encoding::Raw, Encoding::Rows, Encoding::PackedRows]
    );
    // Runs of 0 (2 pixels), 5 (3), and in encoding 1 of 200 and 7 (2 each: 7 and above in 7).
    assert_eq!(summary.run_pixels, [2, 0, 0, 0, 0, 3, 0, 4]);
    assert_eq!(summary.runs(), 9);
    assert_eq!(problems(&summary, 3, 1), Vec::<String>::new());
    assert_eq!(
        problems(&summary, 4, 1),
        ["3 frames in group 0, the view draws 4"]
    );
    // The groups drawn (a hero's facings, E24 M1 T5a): group 1 holds fewer frames than group 0,
    // there is no group 2.
    assert_eq!(summary.groups, [(0, 3), (1, 1)]);
    assert_eq!(
        problems(&summary, 1, 3),
        [
            "1 frames in group 1, fewer than group 0's 3",
            "no group 2, the view draws it"
        ]
    );
    let full = def(&[(0, &[("a", 0)]), (1, &[("b", 1)])], &frames[..2]);
    assert!(problems(&summarize(&full).unwrap(), 1, 2).is_empty());
    // A step's walking groups (E24 M2 T3b): each with the walk's frames, whatever group 0 holds.
    assert_eq!(
        walk_problems(&summary, 1..3, 2),
        [
            "1 frames in group 1, a step draws 2",
            "no group 2, a step draws it"
        ]
    );
    assert!(walk_problems(&summary, 0..1, 3).is_empty());
    assert!(walk_problems(&summary, 0..0, 9).is_empty());
}

#[test]
fn a_def_without_group_0_holds_no_frames() {
    let bytes = def(&[(2, &[("a", 0)])], &[frame(Encoding::Raw, [1; 8])]);
    let summary = summarize(&bytes).unwrap();
    assert_eq!((summary.frames, summary.runs()), (0, 0));
    assert!(summary.encodings.is_empty());
    assert_eq!(
        problems(&summary, 0, 1),
        ["0 frames in group 0, the view draws 1"]
    );
}

#[test]
fn a_broken_def_or_frame_is_an_error() {
    assert!(summarize(&[1, 2, 3]).is_err());
    let mut bytes = def(&[(0, &[("a", 0)])], &[frame(Encoding::Raw, [1; 8])]);
    let len = bytes.len();
    bytes.truncate(len - 4);
    assert!(summarize(&bytes).is_err());
    // A group naming a frame past the file's end.
    let bytes = def(&[(0, &[("a", 5)])], &[frame(Encoding::Raw, [1; 8])]);
    assert!(summarize(&bytes).is_err());
}

/// A 2 × 1 shape: its body in the anchor's cell, its shadow in both.
fn shape_2x1() -> Shape {
    Shape {
        width: 2,
        height: 1,
        colour: [0, 0, 0, 0, 0, 0x80],
        shadow: [0, 0, 0, 0, 0, 0xC0],
        frames: 1,
    }
}

/// A one-row frame of `pixels` (their runs given) at (left, top).
fn row(left: i32, top: i32, pixels: &[(u8, bool)]) -> Frame {
    Frame {
        encoding: Encoding::PackedBlocks,
        full_width: 64,
        full_height: 32,
        left,
        top,
        width: u32::try_from(pixels.len()).unwrap(),
        height: 1,
        pixels: pixels.iter().map(|p| p.0).collect(),
        runs: pixels.iter().map(|p| p.1).collect(),
        short_header: false,
    }
}

#[test]
fn a_body_pixel_outside_the_colour_mask_is_never_drawn() {
    let shape = shape_2x1();
    // At x 30 and 31 the cell left of the anchor (shadow only), at 32 and 33 the anchor's.
    let frame = row(30, 0, &[(9, false), (5, true), (9, false), (5, true)]);
    assert_eq!(
        undrawn_frame(&frame, &shape),
        Undrawn { body: 2, shadow: 0 }
    );
    // Shadows in both cells, other runs drawn by neither pass.
    let frame = row(
        30,
        0,
        &[(1, true), (4, true), (0, true), (2, true), (4, true)],
    );
    assert_eq!(undrawn_frame(&frame, &shape), Undrawn::default());
    assert!(!Undrawn::default().any());
}

#[test]
fn a_shadow_pixel_outside_the_shadow_mask_is_never_drawn() {
    let mut shape = shape_2x1();
    shape.shadow = [0, 0, 0, 0, 0, 0x80];
    let frame = row(31, 0, &[(1, true), (4, true), (4, true)]);
    let undrawn = undrawn_frame(&frame, &shape);
    assert_eq!(undrawn, Undrawn { body: 0, shadow: 1 });
    assert!(undrawn.any());
    assert!(Undrawn { body: 1, shadow: 0 }.any());
}

#[test]
fn pixels_outside_the_footprints_squares_are_never_drawn() {
    let shape = shape_2x1();
    let pixel = [(9, false)];
    // Right of the anchor's square, below it, left of the footprint, above it.
    for (left, top) in [(64, 0), (0, 32), (-1, 0), (0, -1)] {
        assert_eq!(
            undrawn_frame(&row(left, top, &pixel), &shape),
            Undrawn { body: 1, shadow: 0 },
            "{left},{top}"
        );
    }
    // Rows wrap at the frame's width.
    let mut two_rows = row(32, 0, &[(9, false), (9, false)]);
    two_rows.width = 1;
    two_rows.height = 2;
    assert_eq!(undrawn_frame(&two_rows, &shape), Undrawn::default());
    two_rows.top = 31;
    assert_eq!(
        undrawn_frame(&two_rows, &shape),
        Undrawn { body: 1, shadow: 0 }
    );
}

#[test]
fn a_defs_undrawn_pixels_are_its_group_0s() {
    let shape = shape_2x1();
    // 64 × 2 canvases: the left 32 columns are the cell without a body.
    let mut left = [0u8; 128];
    left[..32].fill(9);
    left[64..96].fill(9);
    let both = [9u8; 128];
    let frames = [frame_64([9; 128]), frame_64(left), frame_64(both)];
    let bytes = build::def(
        0x43,
        (64, 2),
        &Palette([Rgb::default(); 256]),
        &[(1, &[("c", 0)]), (0, &[("a", 1), ("b", 2)])],
        &frames,
    );
    assert_eq!(
        undrawn(&bytes, &shape).unwrap(),
        Undrawn {
            body: 64 + 64,
            shadow: 0
        }
    );
    assert!(undrawn(&[1, 2, 3], &shape).is_err());
    let truncated = &bytes[..bytes.len() - 4];
    assert!(undrawn(truncated, &shape).is_err());
}

/// A 64 × 2 frame in encoding 3 (literal pixels: no two neighbours of a run's index repeat).
fn frame_64(pixels: [u8; 128]) -> Frame {
    Frame {
        encoding: Encoding::PackedBlocks,
        full_width: 64,
        full_height: 2,
        left: 0,
        top: 0,
        width: 64,
        height: 2,
        runs: build::runs_by_repetition(Encoding::PackedBlocks, 64, &pixels),
        pixels: pixels.to_vec(),
        short_header: false,
    }
}
