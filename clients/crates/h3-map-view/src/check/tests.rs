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
    assert_eq!(problems(&summary, 3), Vec::<String>::new());
    assert_eq!(
        problems(&summary, 4),
        ["3 frames in group 0, the view draws 4"]
    );
}

#[test]
fn a_def_without_group_0_holds_no_frames() {
    let bytes = def(&[(2, &[("a", 0)])], &[frame(Encoding::Raw, [1; 8])]);
    let summary = summarize(&bytes).unwrap();
    assert_eq!((summary.frames, summary.runs()), (0, 0));
    assert!(summary.encodings.is_empty());
    assert_eq!(
        problems(&summary, 0),
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
