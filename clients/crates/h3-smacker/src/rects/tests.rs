use super::*;
use crate::build::{self, FrameSpec, Header, TreeSpec, Video};
use crate::check;
use crate::tree::TYPES;
use crate::{Clip, Decoder};
use alloc::vec;

/// Every call of one frame's `SmackToBufferRect` on `map`: whether it took one and the last.
fn calls(rects: &mut Rects, map: &[u8], palette: bool) -> Vec<(bool, Rect)> {
    rects.decoded();
    let mut out = Vec::new();
    loop {
        let took = rects.take(map, palette);
        out.push((took, rects.last()));
        if !took {
            return out;
        }
    }
}

/// The iterator's runs of one frame's `map`.
fn runs(rects: &mut Rects, map: &[u8]) -> Vec<Rect> {
    rects.decoded();
    let mut out = Vec::new();
    while rects.span(map) {
        out.push(rects.run);
    }
    // Done stays done.
    assert!(!rects.span(map));
    out
}

#[test]
fn runs_join_while_their_box_stays_within_a_quarter_more_than_their_areas() {
    // 128 × 8: two spans a row. The second row's right span alone: the box over both rows
    // (1 024) is more than 1.25 times their areas (960), so two rectangles; both of the second
    // row's spans: one.
    let mut rects = Rects::new([128, 8], [0, 0]);
    assert_eq!(
        calls(&mut rects, &[1, 1, 0, 1], false),
        [
            (true, [0, 0, 128, 4]),
            (true, [64, 4, 64, 4]),
            (false, [64, 4, 64, 4])
        ]
    );
    assert_eq!(
        calls(&mut rects, &[1, 1, 1, 1], false),
        [(true, [0, 0, 128, 8]), (false, [0, 0, 128, 8])]
    );
}

#[test]
fn a_frame_without_changes_gives_none_and_one_of_its_palette_an_empty_one_and_the_last_run() {
    let mut rects = Rects::new([128, 8], [0, 0]);
    assert_eq!(
        calls(&mut rects, &[1, 0, 0, 0], false),
        [(true, [0, 0, 64, 4]), (false, [0, 0, 64, 4])]
    );
    // Only the palette: an empty rectangle at (0, 0), then the iterator's last run again.
    assert_eq!(
        calls(&mut rects, &[0; 4], true),
        [
            (true, [0, 0, 0, 0]),
            (true, [0, 0, 64, 4]),
            (false, [0, 0, 64, 4])
        ]
    );
    // Nothing: none, the last kept (the game shows it again).
    assert_eq!(calls(&mut rects, &[0; 4], false), [(false, [0, 0, 64, 4])]);
}

#[test]
fn a_run_cut_at_a_row_s_end_goes_on_at_the_next_and_looks_two_spans_ahead() {
    // 96 × 12: spans of 64 run on across the rows (span 1: 64 to 96 of row 0 and 0 to 32 of row
    // 1). Its rest on row 1 looks at spans 2 and 3 as one word: span 3 changed takes span 2 in.
    let mut rects = Rects::new([96, 12], [0, 0]);
    assert_eq!(
        runs(&mut rects, &[0, 1, 0, 1, 0]),
        [[64, 0, 32, 4], [0, 4, 96, 4], [0, 8, 64, 4]]
    );
    // Both unchanged: the rest alone.
    assert_eq!(
        runs(&mut rects, &[0, 1, 0, 0, 0]),
        [[64, 0, 32, 4], [0, 4, 32, 4]]
    );
    assert_eq!(
        calls(&mut rects, &[0, 1, 0, 0, 0], false),
        [
            (true, [64, 0, 32, 4]),
            (true, [0, 4, 32, 4]),
            (false, [0, 4, 32, 4])
        ]
    );
}

#[test]
fn a_clip_narrower_than_a_span_takes_a_run_a_row() {
    // 16 wide: a span is four rows of blocks.
    let mut rects = Rects::new([16, 12], [0, 0]);
    assert_eq!(
        runs(&mut rects, &[1]),
        [[0, 0, 16, 4], [0, 4, 16, 4], [0, 8, 16, 4]]
    );
    // An unchanged span skips its rows.
    let mut rects = Rects::new([16, 32], [0, 0]);
    assert_eq!(
        runs(&mut rects, &[0, 1]),
        [
            [0, 16, 16, 4],
            [0, 20, 16, 4],
            [0, 24, 16, 4],
            [0, 28, 16, 4]
        ]
    );
    let mut rects = Rects::new([16, 4], [0, 0]);
    assert!(runs(&mut rects, &[0]).is_empty());
}

#[test]
fn rectangles_are_aligned_to_8_pixels_of_the_buffer() {
    // At x 83: a left edge 147 moves to 144 (still right of 83); the width 67 rounds up to 72
    // where the clip has room.
    let mut rects = Rects::new([256, 4], [83, 187]);
    assert_eq!(
        calls(&mut rects, &[0, 1, 0, 0], false)[0],
        (true, [144, 187, 72, 4])
    );
    let mut rects = Rects::new([128, 4], [83, 187]);
    assert_eq!(
        calls(&mut rects, &[0, 1], false)[0],
        (true, [144, 187, 67, 4])
    );
    // The clip's own left edge stays.
    let mut rects = Rects::new([128, 4], [83, 187]);
    assert_eq!(
        calls(&mut rects, &[1, 0], false)[0],
        (true, [83, 187, 64, 4])
    );
}

#[test]
fn the_list_holds_four_and_is_taken_first_in_first_out() {
    // Five runs too far apart to join.
    let mut rects = Rects::new([576, 4], [0, 0]);
    let got = calls(&mut rects, &[1, 0, 1, 0, 1, 0, 1, 0, 1], false);
    let want: Vec<_> = (0..5).map(|k| (true, [128 * k, 0, 64, 4])).collect();
    assert_eq!(got[..5], want[..]);
    assert_eq!(got[5], (false, [512, 0, 64, 4]));
}

/// The list's join on `list` (the last the new one unless `new` says otherwise).
fn joined(list: &[Rect], new: usize) -> Vec<Rect> {
    let mut rects = Rects::new([256, 16], [0, 0]);
    rects.list[..list.len()].copy_from_slice(list);
    rects.count = list.len();
    rects.join(new);
    rects.list[..rects.count].to_vec()
}

#[test]
fn a_new_rectangle_joins_one_it_shares_a_whole_edge_with() {
    let (a, below, beside) = ([0, 0, 64, 4], [0, 4, 64, 8], [64, 0, 128, 4]);
    assert_eq!(joined(&[a, below], 1), [[0, 0, 64, 8]]);
    assert_eq!(joined(&[below, a], 1), [[0, 0, 64, 8]]);
    assert_eq!(joined(&[a, beside], 1), [[0, 0, 128, 4]]);
    assert_eq!(joined(&[beside, a], 1), [[0, 0, 128, 4]]);
    // Same columns or rows, apart: none.
    assert_eq!(joined(&[a, [0, 8, 64, 12]], 1), [a, [0, 8, 64, 12]]);
    assert_eq!(joined(&[a, [128, 0, 192, 4]], 1), [a, [128, 0, 192, 4]]);
    assert_eq!(joined(&[a, [8, 4, 64, 8]], 1), [a, [8, 4, 64, 8]]);
}

#[test]
fn a_joined_rectangle_joins_on_by_its_old_index() {
    // The new one joins the first, the first then the third, which moves down: the index then
    // names the slot past the list (its copy), which joins nothing.
    assert_eq!(
        joined(&[[0, 0, 64, 4], [128, 0, 192, 4], [64, 0, 128, 4]], 2),
        [[0, 0, 192, 4]]
    );
    // That copy joining the second: the DLL's copy would run backwards and fault; it stops.
    assert_eq!(
        joined(
            &[
                [0, 0, 64, 4],
                [0, 4, 192, 8],
                [128, 0, 192, 4],
                [64, 0, 128, 4]
            ],
            3
        ),
        [[0, 0, 192, 8]]
    );
}

/// A clip of 64 × 8 (a span a row): frames of fills and skips (`h3_smacker`'s `build`).
fn clip(frames: &[&[u32]]) -> Vec<u8> {
    let fill = 3 | 15 << 2 | 0x22 << 8;
    let skip = 2 | 15 << 2;
    let h = Header::new(64, 8, [None, None, None, Some(TreeSpec::of(&[fill, skip]))]);
    let specs: Vec<_> = frames
        .iter()
        .map(|runs| {
            let mut video = Video::new(&h);
            for &fill_it in *runs {
                video.code(TYPES, if fill_it == 1 { fill } else { skip });
            }
            FrameSpec::video(video.bytes())
        })
        .chain([FrameSpec::video(vec![])])
        .collect();
    build::file(&h, &specs)
}

#[test]
fn the_game_s_reads_of_decoded_frames() {
    // The first row filled, then nothing: the game reads the last one again.
    let bytes = clip(&[&[1, 0], &[0, 0]]);
    let parsed = Clip::parse(&bytes).unwrap();
    let mut decoder = Decoder::new(&parsed).unwrap();
    let mut rects = Rects::new([64, 8], [0, 0]);
    let mut reader = parsed.reader();
    for _ in 0..2 {
        let packet = reader.next().unwrap().unwrap();
        rects.decoded();
        let frame = decoder.decode(&packet).unwrap();
        assert_eq!(rects.shown(frame), [[0, 0, 64, 4]]);
    }
    // The check's lines (the frame without video left out).
    assert_eq!(
        check::rects(&bytes, [0, 0]).unwrap(),
        [
            vec![(true, [0, 0, 64, 4]), (false, [0, 0, 64, 4])],
            vec![(false, [0, 0, 64, 4])]
        ]
    );
    assert_eq!(check::rects(b"no", [0, 0]), Err(crate::Error::Magic));
    let wide = Header::new(1000, 8, [None, None, None, Some(TreeSpec::of(&[2]))]);
    let bytes = build::file(&wide, &[FrameSpec::video(vec![0])]);
    assert_eq!(check::rects(&bytes, [0, 0]), Err(crate::Error::Size));
}
