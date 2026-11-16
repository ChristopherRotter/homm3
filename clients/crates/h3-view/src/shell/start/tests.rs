//! The start's tests (E27 M2 T5a): the DLLs' schedules, the sequence, its fades and skips, and
//! the picture's ops on synthetic clips.

use alloc::vec;
use alloc::vec::Vec;

use h3_render::Bugs;
use h3_render::screen::Screen as Pixels;

use super::{
    Codec, FADE_STEP, Header, Op, Pace, Part, Picture, PictureError, SmackSchedule, SmackSound,
    Source, Start, key_skips, place, shown_union, smack_period, smack_wait,
};
use crate::compose::FORMAT;

const SMACK: Pace = Pace::Smacker(6666);

fn black() -> Pixels {
    Pixels::new(800, 600, FORMAT).unwrap()
}
const BINK: Pace = Pace::Bink { num: 15, den: 1 };

fn header(frames: u32, pace: Pace) -> Option<Header> {
    Some(Header {
        size: [400, 300],
        frames,
        pace,
    })
}

/// The times of the readings at which `start` lets each op happen, reading at every `step` ms
/// up to `until`; `inputs` skips at those times.
fn run(start: &mut Start, step: u32, until: u32, inputs: &[u32]) -> Vec<(u32, Op)> {
    let mut seen = Vec::new();
    let mut now = 0;
    while now < until && !start.ended() {
        now += step;
        let mut ops = Vec::new();
        start.read(now, &mut ops);
        if inputs.contains(&now) {
            start.skip(now, &mut ops);
        }
        seen.extend(ops.into_iter().map(|op| (now, op)));
    }
    seen
}

fn frames_at(seen: &[(u32, Op)], part: Part) -> Vec<u32> {
    seen.iter()
        .filter(|(_, op)| matches!(op, Op::Frame(p, _) if *p == part))
        .map(|(t, _)| *t)
        .collect()
}

#[test]
fn the_frame_times_are_the_headers() {
    assert_eq!(smack_period(-6666), 6666);
    assert_eq!(smack_period(100), 10_000);
    assert_eq!(smack_period(0), 10_000);
    assert_eq!(smack_period(i32::MIN), 2_147_483_648);
    assert_eq!(Part::Logo.codec(true), Codec::Smacker);
    assert_eq!(Part::Nwc.codec(true), Codec::Bink);
    assert_eq!(Part::Intro.codec(false), Codec::Smacker);
    assert_eq!(
        Part::ALL.map(|p| p.file(true)),
        ["3DOLOGO.SMK", "NWCLOGO.BIK", "H3INTRO.BIK"]
    );
    assert_eq!(
        Part::ALL.map(|p| p.file(false)),
        ["3DOLOGO.SMK", "NWCLOGO.SMK", "H3INTRO.SMK"]
    );
    assert_eq!(Part::ALL.map(Part::index), [28, 29, 30]);
}

#[test]
fn every_key_but_f4_stops_a_clip() {
    // The window's rule and the replay's (the review of M2, fix 1: the window skipped on F4).
    for (name, skips) in [("F4", false), ("f4", false), ("F3", true), ("Escape", true)] {
        assert_eq!(key_skips(name), skips, "{name}");
    }
}

#[test]
fn smack_wait_lets_a_frame_through_a_period_after_the_last() {
    // The first call sets the due time and waits; at 1 ms steps the frames go 67 ms apart, at
    // 10 ms steps 70 (the recordings' `start-bink` logo and `start-smacker`'s 70 ms).
    for (step, apart) in [(1, 67), (10, 70)] {
        let mut s = SmackSchedule::default();
        let gone: Vec<u32> = (0..=400 / step)
            .map(|k| k * step)
            .filter(|&t| smack_wait(&mut s, t, 6666, Bugs::Original))
            .collect();
        let want: Vec<u32> = (1..=400 / apart).map(|k| k * apart).collect();
        assert_eq!(gone, want, "{step} ms steps");
    }
    // A call between one and two periods late keeps the schedule (the due time + a period):
    // the next frame goes at once.
    let mut s = SmackSchedule::default();
    assert!(!smack_wait(&mut s, 0, 6666, Bugs::Original));
    assert!(smack_wait(&mut s, 150, 6666, Bugs::Original));
    assert!(smack_wait(&mut s, 150, 6666, Bugs::Original));
    assert!(!smack_wait(&mut s, 150, 6666, Bugs::Original));
}

#[test]
fn smack_wait_in_the_clients_profile_keeps_each_frame_on_its_time() {
    // CLT-Q129 (b): a frame a period after the last one's due time, at any step: the first
    // reading at or past k × 66.66 ms lets frame k − 1 through.
    for step in [1, 7, 10] {
        let mut s = SmackSchedule::default();
        let last = 400 / step * step;
        let gone: Vec<u32> = (0..=400 / step)
            .map(|k| k * step)
            .filter(|&t| smack_wait(&mut s, t, 6666, Bugs::Fixed))
            .collect();
        let want: Vec<u32> = (1..)
            .map(|k: u32| (6666 * k).div_ceil(100 * step) * step)
            .take_while(|&t| t <= last)
            .collect();
        assert_eq!(gone, want, "{step} ms steps");
    }
    // A reading several periods late lets each frame due through, one a call (the DLL's would
    // date the next from it: one frame, then a period on).
    let mut s = SmackSchedule::default();
    assert!(!smack_wait(&mut s, 0, 6666, Bugs::Fixed));
    assert!((0..3).all(|_| smack_wait(&mut s, 200, 6666, Bugs::Fixed)));
    assert!(!smack_wait(&mut s, 200, 6666, Bugs::Fixed));
    let mut s = SmackSchedule::default();
    assert!(!smack_wait(&mut s, 0, 6666, Bugs::Original));
    assert!(smack_wait(&mut s, 200, 6666, Bugs::Original));
    assert!(!smack_wait(&mut s, 200, 6666, Bugs::Original));
}

#[test]
fn a_silent_smacker_clip_drifts_with_the_step_in_the_originals_profile_alone() {
    // 81 frames at 10 ms steps: the DLL's 70 ms a frame (the last at 5 670 ms, design §11.2's
    // 3DO logo), the client's a frame time each (the last at 81 × 66.66 = 5 399.5: 5 400).
    for (bugs, second, last) in [(Bugs::Original, 140, 5670), (Bugs::Fixed, 140, 5400)] {
        let mut ops = Vec::new();
        let mut start = Start::new([header(81, SMACK), None, None], true, bugs, 0, &mut ops);
        let seen = run(&mut start, 10, 7000, &[]);
        let frames = frames_at(&seen, Part::Logo);
        assert_eq!(frames.len(), 81, "{bugs:?}");
        assert_eq!((frames[1], frames[80]), (second, last), "{bugs:?}");
    }
    // At 1 ms steps the two part from the third frame on (201 ms against 200).
    let third = |bugs| {
        let mut ops = Vec::new();
        let mut start = Start::new([header(81, SMACK), None, None], true, bugs, 0, &mut ops);
        frames_at(&run(&mut start, 1, 300, &[]), Part::Logo)[2]
    };
    assert_eq!((third(Bugs::Original), third(Bugs::Fixed)), (201, 200));
}

/// The readings (every 10 ms) at which `start` keeps an input for the main menu, and the end.
fn kept(mut start: Start, skip: Option<u32>) -> (Vec<u32>, u32) {
    let (mut ops, mut kept, mut now) = (Vec::new(), Vec::new(), 0);
    while !start.ended() {
        assert!(!start.keeps_input() || start.playing().is_none());
        now += 10;
        start.read(now, &mut ops);
        if skip == Some(now) {
            start.skip(now, &mut ops);
        }
        if start.keeps_input() {
            kept.push(now);
        }
    }
    assert!(!start.keeps_input());
    (kept, now)
}

#[test]
fn an_input_waits_for_the_menu_during_a_fade_no_clip_follows() {
    // CLT-Q130 (a): the next clip's loop flushes the input queue; with none the menu takes it.
    // H3Intro's fade alone, its 150 ms before the menu.
    let mut ops = Vec::new();
    let headers = [header(2, SMACK), header(2, BINK), header(2, BINK)];
    let (kept_at, end) = kept(Start::new(headers, true, Bugs::Original, 0, &mut ops), None);
    assert_eq!(kept_at, (end - 150..end).step_by(10).collect::<Vec<_>>());
    // A skipped logo's fade ends the start; so does one before a clip that does not open.
    let start = Start::new(headers, true, Bugs::Fixed, 0, &mut ops);
    let (kept_at, end) = kept(start, Some(50));
    assert_eq!((kept_at.first(), kept_at.len(), end), (Some(&50), 15, 200));
    let start = Start::new([header(2, SMACK), None, None], true, Bugs::Original, 0, &mut ops);
    let (kept_at, end) = kept(start, None);
    assert_eq!(kept_at, (end - 150..end).step_by(10).collect::<Vec<_>>());
}

#[test]
fn the_clips_play_in_order_each_ending_in_a_fade() {
    let mut ops = Vec::new();
    let headers = [header(3, SMACK), header(2, BINK), header(2, BINK)];
    let mut start = Start::new(headers, true, Bugs::Original, 0, &mut ops);
    assert_eq!(ops, [Op::Black]);
    assert_eq!(start.playing(), Some(Part::Logo));
    assert!(start.bink_video());
    let seen = run(&mut start, 1, 2000, &[]);
    // The logo's frames 67 ms apart; its last at 201 not shown, the fade: 201, 251, 301, black
    // at 351 and NWCLogo's first frame then; Bink's at ⌊1000 p / 15⌋ from it.
    assert_eq!(frames_at(&seen, Part::Logo), [67, 134, 201]);
    assert_eq!(frames_at(&seen, Part::Nwc), [351, 417]);
    let fades: Vec<(u32, Op)> = seen
        .iter()
        .copied()
        .filter(|(_, op)| matches!(op, Op::Fade(_) | Op::Black))
        .collect();
    assert_eq!(
        fades,
        [
            (201, Op::Fade(0)),
            (251, Op::Fade(1)),
            (301, Op::Fade(2)),
            (351, Op::Black),
            (417, Op::Fade(0)),
            (467, Op::Fade(1)),
            (517, Op::Fade(2)),
            (567, Op::Black),
            (633, Op::Fade(0)),
            (683, Op::Fade(1)),
            (733, Op::Fade(2)),
            (783, Op::Black),
        ]
    );
    // H3Intro: its frame, its first frame decoded, the whole screen shown, at the black.
    let at_567: Vec<Op> = seen
        .iter()
        .filter(|(t, _)| *t == 567)
        .map(|(_, op)| *op)
        .collect();
    assert_eq!(
        at_567,
        [
            Op::Black,
            Op::Rim,
            Op::Frame(Part::Intro, 0),
            Op::Show(Part::Intro),
            Op::ShowAll
        ]
    );
    let shows = seen
        .iter()
        .filter(|(_, op)| matches!(op, Op::Show(_)))
        .count();
    assert_eq!(shows, 2 + 1 + 1, "every frame but the last shown");
    assert!(start.ended() && start.playing().is_none());
    // A reading after the end does nothing.
    let mut ops = Vec::new();
    start.read(5000, &mut ops);
    start.skip(5000, &mut ops);
    assert!(ops.is_empty());
}

#[test]
fn a_fade_step_waits_for_its_50_ms_and_one_reading_takes_one_step() {
    let mut ops = Vec::new();
    let mut start = Start::new([header(1, BINK), None, None], true, Bugs::Original, 0, &mut ops);
    assert_eq!(ops, [Op::Black, Op::Frame(Part::Logo, 0), Op::Fade(0)]);
    for (now, want) in [
        (FADE_STEP - 1, &[][..]),
        (200, &[Op::Fade(1)]),
        (200, &[]),
        (249, &[]),
        (250, &[Op::Fade(2)]),
        (300, &[Op::Black]),
    ] {
        let mut ops = Vec::new();
        start.read(now, &mut ops);
        assert_eq!(ops, want, "at {now}");
    }
    // NWCLogo does not open: the sequence ends.
    assert!(start.ended());
}

#[test]
fn a_skip_fades_a_full_screen_clip_and_stops_h3intro_at_once() {
    // The logo skipped at 2000 ms by a reading every 10 ms: its fade, the menu at 2150.
    let headers = [header(100, SMACK), header(100, BINK), header(100, BINK)];
    let mut ops = Vec::new();
    let mut start = Start::new(headers, true, Bugs::Original, 0, &mut ops);
    let seen = run(&mut start, 10, 10_000, &[2000]);
    assert_eq!(seen.last(), Some(&(2150, Op::Black)));
    assert_eq!(seen.iter().filter(|(_, op)| *op == Op::Fade(0)).count(), 1);
    assert!(start.ended());
    // An input during a fade is lost; H3Intro's skip ends the start at once, without a fade.
    let mut ops = Vec::new();
    let mut start = Start::new(
        [header(2, SMACK), header(2, BINK), header(50, BINK)],
        true,
        Bugs::Original,
        0,
        &mut ops,
    );
    let seen = run(&mut start, 10, 10_000, &[150, 1500]);
    assert_eq!(frames_at(&seen, Part::Logo), [70, 140]);
    // H3Intro from 510 (the fades' 150 ms twice, NWCLogo's frames at 290 and 360): its frame 14
    // at 1450, the 15th due at 1510; the skip at 1500 ends it with nothing drawn.
    assert_eq!(frames_at(&seen, Part::Intro).first(), Some(&510));
    assert_eq!(frames_at(&seen, Part::Intro).last(), Some(&1450));
    assert!(seen.iter().all(|(t, _)| *t < 1500));
    assert!(start.ended());
    // A clip that does not open ends the sequence where it would begin.
    let mut ops = Vec::new();
    let start = Start::new([None, header(2, BINK), header(2, BINK)], true, Bugs::Original, 0, &mut ops);
    assert!(start.ended());
    assert_eq!(ops, [Op::Black]);
}

#[test]
fn the_clips_take_their_places() {
    assert_eq!(
        place(Part::Logo, Codec::Smacker, [640, 480]),
        [80, 60, 640, 480]
    );
    assert_eq!(place(Part::Nwc, Codec::Bink, [400, 300]), [0, 0, 400, 300]);
    assert_eq!(
        place(Part::Nwc, Codec::Smacker, [800, 600]),
        [0, 0, 800, 600]
    );
    assert_eq!(
        place(Part::Intro, Codec::Bink, [640, 360]),
        [80, 187, 640, 360]
    );
    assert_eq!(
        place(Part::Intro, Codec::Smacker, [640, 360]),
        [80, 187, 640, 360]
    );
}

#[test]
fn the_picture_carries_out_the_ops() {
    let rim = {
        let mut rim = Pixels::new(800, 600, FORMAT).unwrap();
        rim.fill(0x1234);
        rim
    };
    let bink = h3_bink::build::sample(1);
    let clip = h3_bink::Clip::parse(&bink).unwrap();
    let decoder = h3_bink::Decoder::new(&clip).unwrap();
    let size = [clip.width, clip.height];
    let sources = [
        None,
        Some((
            Source::Bink {
                clip: &clip,
                decoder: alloc::boxed::Box::new(decoder.clone()),
                next: 0,
            },
            Header {
                size,
                frames: 3,
                pace: BINK,
            },
        )),
        Some((
            Source::Bink {
                clip: &clip,
                decoder: alloc::boxed::Box::new(decoder),
                next: 0,
            },
            Header {
                size,
                frames: 3,
                pace: BINK,
            },
        )),
    ];
    let mut picture = Picture::new(black(), sources, Some(&rim), true, Bugs::Fixed);
    assert!(picture.screen().pixels().iter().all(|&p| p == 0));
    picture.apply(Op::Rim).unwrap();
    // The sound's ops leave the buffers (E27 M2 T5b1).
    let (back, screen) = (picture.back().clone(), picture.screen().clone());
    picture.apply(Op::Sound(Part::Logo)).unwrap();
    picture.apply(Op::Quiet(Part::Logo)).unwrap();
    picture.apply(Op::Closed(Part::Logo)).unwrap();
    assert_eq!((picture.back(), picture.screen()), (&back, &screen));
    assert!(picture.back().pixels().iter().all(|&p| p == 0x1234));
    assert!(picture.screen().pixels().iter().all(|&p| p == 0));
    picture.apply(Op::Frame(Part::Intro, 0)).unwrap();
    let back = picture.back().pixels().to_vec();
    // H3Intro's place: (80, 187); outside it the rim.
    let at = |x: usize, y: usize| back[y * 800 + x];
    let first = crate::clip::rgb565(h3_bink::check::frames(&bink).unwrap().first().unwrap());
    assert_eq!(at(80, 187), first[0]);
    assert_eq!(at(79, 187), 0x1234);
    picture.apply(Op::ShowAll).unwrap();
    assert_eq!(picture.screen().pixels(), &back[..]);
    // The fade's steps halve each channel; black fills both.
    picture.apply(Op::Fade(1)).unwrap();
    let half = picture.screen().pixel(0, 0).unwrap();
    assert_eq!(
        half,
        ((0x1234 & 0xF800) >> 1 & 0xF800)
            | ((0x1234 & 0x07E0) >> 1 & 0x07E0)
            | ((0x1234 & 0x1F) >> 1)
    );
    picture.apply(Op::Black).unwrap();
    assert!(
        picture
            .back()
            .pixels()
            .iter()
            .chain(picture.screen().pixels())
            .all(|&p| p == 0)
    );
    // NWCLogo's Bink copy at the corner, doubled onto the screen.
    picture.apply(Op::Frame(Part::Nwc, 0)).unwrap();
    picture.apply(Op::Show(Part::Nwc)).unwrap();
    let screen = picture.screen();
    // The sample's 256 pixels across over the screen's 800: a step of 20 971 / 65 536.
    assert_eq!(size[0], 256);
    assert_eq!(screen.pixel(0, 0), Some(first[0]));
    assert_eq!(screen.pixel(3, 1), Some(first[0]));
    assert_eq!(screen.pixel(4, 0), Some(first[1]));
    assert_eq!(screen.pixel(799, 0), Some(first[255]));
    // The fix's fade (CLT-Q123): from the screen as shown, the stretched picture; its second
    // step from the same picture though the screen changed.
    let shown = screen.clone();
    picture.apply(Op::Fade(0)).unwrap();
    assert_eq!(picture.screen(), &shown);
    picture.apply(Op::Fade(1)).unwrap();
    let halve =
        |p: u16| ((p & 0xF800) >> 1 & 0xF800) | ((p & 0x07E0) >> 1 & 0x07E0) | ((p & 0x1F) >> 1);
    assert_eq!(
        picture.screen().pixel(799, 599),
        shown.pixel(799, 599).map(halve)
    );
    picture.apply(Op::Black).unwrap();
    // Out of order, or a clip without its source: refused.
    assert_eq!(
        picture.apply(Op::Frame(Part::Nwc, 2)),
        Err(PictureError::Order(Part::Nwc, 2))
    );
    assert_eq!(
        picture.apply(Op::Frame(Part::Logo, 0)),
        Err(PictureError::Order(Part::Logo, 0))
    );
}

#[test]
fn a_smacker_frame_is_copied_whole_or_as_the_dll_copies_it() {
    use h3_smacker::check::{Format, original_16};
    for seed in 1..=6 {
        let bytes = h3_smacker::build::sample(seed);
        let clip = h3_smacker::Clip::parse(&bytes).unwrap();
        let decoded = h3_smacker::check::frames(&bytes).unwrap();
        let (w, h) = (clip.width as usize, clip.height as usize);
        let frames = u32::try_from(decoded.len()).unwrap();
        for bugs in [Bugs::Fixed, Bugs::Original] {
            let source = Source::Smacker {
                reader: alloc::boxed::Box::new(clip.reader()),
                decoder: alloc::boxed::Box::new(h3_smacker::Decoder::new(&clip).unwrap()),
            };
            let header = Header {
                size: [clip.width, clip.height],
                frames,
                pace: SMACK,
            };
            let sources = [Some((source, header)), None, None];
            let mut picture = Picture::new(black(), sources, None, false, bugs);
            for n in 0..frames {
                picture.apply(Op::Frame(Part::Logo, n)).unwrap();
            }
            picture.apply(Op::Show(Part::Logo)).unwrap();
            let [x, y, _, _] = place(Part::Logo, Codec::Smacker, header.size).map(i64::from);
            // The original's screen shows only the DLL's rectangles (the next test): its copy
            // is the back buffer's.
            let drawn = match bugs {
                Bugs::Fixed => picture.screen(),
                Bugs::Original => picture.back(),
            };
            let shown: Vec<u16> = (0..h)
                .flat_map(|r| (0..w).map(move |c| (c, r)))
                .map(|(c, r)| drawn.pixel(x + c as i64, y + r as i64).unwrap())
                .collect();
            let want: Vec<u16> = match bugs {
                // Every pixel through the last frame's palette.
                Bugs::Fixed => {
                    let last = decoded.last().unwrap();
                    let p = last.palette();
                    last.pixels()
                        .iter()
                        .map(|&i| {
                            let at = 3 * usize::from(i);
                            let [r, g, b] = [0, 1, 2].map(|c| u16::from(p[at + c]));
                            (r >> 3) << 11 | (g >> 2) << 5 | b >> 3
                        })
                        .collect()
                }
                // The DLL's copy (`original_16`): the last frame's words.
                Bugs::Original => {
                    let all = original_16(&decoded, Format::Rgb565);
                    let each = 2 * w * h + 768;
                    let last = &all[each * (decoded.len() - 1)..][..2 * w * h];
                    last.chunks(2)
                        .map(|b| u16::from_le_bytes([b[0], b[1]]))
                        .collect()
                }
            };
            assert_eq!(shown, want, "seed {seed}, {bugs:?}");
            // Outside the clip the back buffer's black.
            assert_eq!(picture.screen().pixel(x - 1, y), Some(0));
        }
    }
}

#[test]
fn the_union_of_the_rectangles_is_the_game_s() {
    assert_eq!(shown_union(&[]), None);
    assert_eq!(shown_union(&[[16, 0, 32, 16]]), Some([16, 0, 32, 16]));
    // One below and further out: the union grows.
    assert_eq!(
        shown_union(&[[16, 0, 32, 16], [0, 16, 64, 16]]),
        Some([0, 0, 64, 32])
    );
    // One below and further left that does not reach as far right: the union's right edge
    // comes in by as much (the game keeps the width while it moves the left edge).
    assert_eq!(
        shown_union(&[[64, 0, 64, 16], [0, 16, 16, 16]]),
        Some([0, 0, 64, 32])
    );
    // One inside: nothing changes.
    assert_eq!(
        shown_union(&[[0, 0, 64, 32], [16, 0, 16, 16]]),
        Some([0, 0, 64, 32])
    );
}

#[test]
fn a_bink_frame_shows_its_rectangles_union_as_the_game_takes_it() {
    use h3_bink::build::{self, Block, Header as BinkHeader};
    // 64 × 32 (4 × 2 cells): a key frame, a frame changing nothing, then one changing the
    // cells (3, 0) and (0, 1): their union as the game takes it is (0, 0, 16, 32), so cell
    // (3, 0) reaches the back buffer but not the screen.
    let fills = |n, c| vec![Block::Fill(c); n];
    let key = build::frame(64, 32, [&fills(32, 40), &fills(8, 90), &fills(8, 160)]);
    let still = build::frame(
        64,
        32,
        [
            &vec![Block::Skip; 32],
            &vec![Block::Skip; 8],
            &vec![Block::Skip; 8],
        ],
    );
    let mut luma = vec![Block::Skip; 32];
    luma[6] = Block::Fill(200);
    luma[16] = Block::Fill(200);
    let moved = build::frame(
        64,
        32,
        [&luma, &vec![Block::Skip; 8], &vec![Block::Skip; 8]],
    );
    let bytes = build::file(&BinkHeader::new(64, 32), &[key, still, moved]);
    let clip = h3_bink::Clip::parse(&bytes).unwrap();
    let decoder = h3_bink::Decoder::new(&clip).unwrap();
    let header = Header {
        size: [64, 32],
        frames: 3,
        pace: BINK,
    };
    let source = Source::Bink {
        clip: &clip,
        decoder: alloc::boxed::Box::new(decoder),
        next: 0,
    };
    let mut picture = Picture::new(
        black(),
        [None, None, Some((source, header))],
        None,
        true,
        Bugs::Original,
    );
    let (x, y) = (80, 187);
    // Before a frame: no rectangles, nothing shown; no rim: nothing copied.
    picture.apply(Op::Rim).unwrap();
    picture.apply(Op::Show(Part::Intro)).unwrap();
    assert!(picture.screen().pixels().iter().all(|&p| p == 0));
    assert!(picture.back().pixels().iter().all(|&p| p == 0));
    for n in 0..3 {
        picture.apply(Op::Frame(Part::Intro, n)).unwrap();
        picture.apply(Op::Show(Part::Intro)).unwrap();
        let (back, screen) = (picture.back(), picture.screen());
        let same = |px: i64, py: i64| back.pixel(x + px, y + py) == screen.pixel(x + px, y + py);
        // Frames 0 and 1 (nothing marked: the first rectangle kept) show the whole picture.
        assert!(same(63, 0) && same(0, 31), "frame {n}");
        if n == 2 {
            assert!(same(0, 16), "cell (0, 1) shown");
            assert!(!same(48, 0), "cell (3, 0) not shown");
        }
    }
    // Past the clip's last frame: the decoder's refusal.
    assert!(matches!(
        picture.apply(Op::Frame(Part::Intro, 3)),
        Err(PictureError::Bink(_))
    ));
    // The original's fade takes the back buffer, what the screen did not show included.
    picture.apply(Op::Fade(0)).unwrap();
    assert_eq!(picture.screen(), picture.back());
}

#[test]
fn the_original_shows_a_smacker_frame_s_rectangles_from_smackw32() {
    use h3_smacker::build::{self, FrameSpec, Header as SmackHeader, TREES, TreeSpec, Video};
    // 64 × 8 (a span a row): the first frame fills both rows, the second only the first.
    let (top, bottom) = (3 | 15 << 2 | 0x40 << 8, 3 | 15 << 2 | 0x80 << 8);
    let skip = 2 | 15 << 2;
    let h = SmackHeader::new(
        64,
        8,
        [None, None, None, Some(TreeSpec::of(&[top, bottom, skip]))],
    );
    let frame = |runs: [u32; 2]| {
        let mut video = Video::new(&h);
        for value in runs {
            video.code(TREES[3], value);
        }
        FrameSpec::video(video.bytes())
    };
    // Colours 0x40 and 0x80 white and red.
    let colours = [
        build::Op::Skip(64),
        build::Op::New([63, 63, 63]),
        build::Op::Skip(63),
        build::Op::New([63, 0, 0]),
    ];
    let first = FrameSpec {
        palette: Some(build::palette(&colours)),
        ..frame([top, bottom])
    };
    let bytes = build::file(&h, &[first, frame([bottom, skip])]);
    let clip = h3_smacker::Clip::parse(&bytes).unwrap();
    let header = Header {
        size: [64, 8],
        frames: 2,
        pace: SMACK,
    };
    let [x, y, _, _] = place(Part::Logo, Codec::Smacker, header.size).map(i64::from);
    for bugs in [Bugs::Fixed, Bugs::Original] {
        let source = Source::Smacker {
            reader: alloc::boxed::Box::new(clip.reader()),
            decoder: alloc::boxed::Box::new(h3_smacker::Decoder::new(&clip).unwrap()),
        };
        let mut picture = Picture::new(
            black(),
            [Some((source, header)), None, None],
            None,
            false,
            bugs,
        );
        // The first frame decoded and not shown, the second shown.
        picture.apply(Op::Frame(Part::Logo, 0)).unwrap();
        picture.apply(Op::Frame(Part::Logo, 1)).unwrap();
        picture.apply(Op::Show(Part::Logo)).unwrap();
        let row = |r: i64| picture.screen().pixel(x, y + r).unwrap();
        assert_ne!(row(0), 0, "{bugs:?}");
        // The second frame's rectangle is its first row: the original leaves the first
        // frame's second row off the screen.
        match bugs {
            Bugs::Fixed => assert_eq!(picture.screen(), picture.back()),
            Bugs::Original => {
                assert_eq!(row(4), 0);
                assert_ne!(picture.back().pixel(x, y + 4), Some(0));
            }
        }
    }
}

#[test]
fn the_fix_shows_a_bink_clip_s_whole_rectangle() {
    use h3_bink::build::{self, Block, Header as BinkHeader};
    let fills = |n, c| vec![Block::Fill(c); n];
    let key = build::frame(64, 32, [&fills(32, 40), &fills(8, 90), &fills(8, 160)]);
    let mut luma = vec![Block::Skip; 32];
    luma[6] = Block::Fill(200);
    luma[16] = Block::Fill(200);
    let moved = build::frame(
        64,
        32,
        [&luma, &vec![Block::Skip; 8], &vec![Block::Skip; 8]],
    );
    let bytes = build::file(&BinkHeader::new(64, 32), &[key, moved]);
    let clip = h3_bink::Clip::parse(&bytes).unwrap();
    let source = Source::Bink {
        clip: &clip,
        decoder: alloc::boxed::Box::new(h3_bink::Decoder::new(&clip).unwrap()),
        next: 0,
    };
    let header = Header {
        size: [64, 32],
        frames: 2,
        pace: BINK,
    };
    let mut picture = Picture::new(
        black(),
        [None, None, Some((source, header))],
        None,
        true,
        Bugs::Fixed,
    );
    for n in 0..2 {
        picture.apply(Op::Frame(Part::Intro, n)).unwrap();
        picture.apply(Op::Show(Part::Intro)).unwrap();
    }
    assert_eq!(picture.screen(), picture.back());
}

#[test]
fn a_smacker_frame_past_the_last_or_cut_short_is_refused() {
    let bytes = h3_smacker::build::sample(2);
    let cut = &bytes[..bytes.len() - 8];
    for (bytes, past) in [(&bytes[..], true), (cut, false)] {
        let clip = h3_smacker::Clip::parse(bytes).unwrap();
        let frames = u32::try_from(clip.frames()).unwrap();
        let source = Source::Smacker {
            reader: alloc::boxed::Box::new(clip.reader()),
            decoder: alloc::boxed::Box::new(h3_smacker::Decoder::new(&clip).unwrap()),
        };
        let header = Header {
            size: [clip.width, clip.height],
            frames,
            pace: SMACK,
        };
        let mut picture = Picture::new(
            black(),
            [Some((source, header)), None, None],
            None,
            false,
            Bugs::Fixed,
        );
        for n in 0..frames - 1 {
            picture.apply(Op::Frame(Part::Logo, n)).unwrap();
        }
        let last = picture.apply(Op::Frame(Part::Logo, frames - 1));
        if past {
            last.unwrap();
            assert_eq!(
                picture.apply(Op::Frame(Part::Logo, frames)),
                Err(PictureError::Order(Part::Logo, frames))
            );
        } else {
            assert!(matches!(last, Err(PictureError::Smacker(_))), "{last:?}");
        }
    }
}

/// The start's clips' sound: 22 050 Hz, 16-bit, stereo.
const TRACK: u32 = 88_200;

fn sound_at(seen: &[(u32, Op)], op: Op) -> Vec<u32> {
    seen.iter()
        .filter(|(_, o)| *o == op)
        .map(|(t, _)| *t)
        .collect()
}

#[test]
fn with_its_sound_a_smacker_clip_follows_the_bytes_played() {
    // 3DOLOGO.SMK as `smackprobe wait` played it (E27 M2 T5b1): frame 0 at the open, the Play 10
    // ms later, frame k once k frames' sound (5 879.4 bytes each) less 8 bytes have played.
    for (step, firsts, last) in [
        (1, [0, 77, 144, 210, 277], 5343),
        (10, [0, 80, 150, 210, 280], 5350),
        (7, [0, 84, 154, 217, 287], 5348),
    ] {
        let mut ops = Vec::new();
        let headers = [header(81, SMACK), None, None];
        let mut start = Start::with_sound(headers, [Some(TRACK), None, None], true, Bugs::Original, 0, &mut ops);
        let mut seen: Vec<(u32, Op)> = ops.into_iter().map(|op| (0, op)).collect();
        seen.extend(run(&mut start, step, 6000, &[]));
        let frames = frames_at(&seen, Part::Logo);
        assert_eq!(frames.len(), 81);
        assert_eq!(frames[..5], firsts, "{step}");
        assert_eq!(frames[80], last, "{step}");
        assert_eq!(
            sound_at(&seen, Op::Sound(Part::Logo)),
            [if step == 7 { 14 } else { 10 }]
        );
        // The close (the original's sound cut, CLT-Q125), before the fade.
        let closed = seen
            .iter()
            .position(|(_, op)| *op == Op::Closed(Part::Logo))
            .unwrap();
        assert_eq!(seen[closed], (last, Op::Closed(Part::Logo)));
        assert_eq!(seen[closed + 1].1, Op::Fade(0));
        assert!(sound_at(&seen, Op::Quiet(Part::Logo)).is_empty());
    }
    // Silent, the same clip keeps SmackWait's clock.
    let mut ops = Vec::new();
    let mut start = Start::with_sound(
        [header(81, SMACK), None, None],
        [None; 3],
        true,
        Bugs::Original,
        0,
        &mut ops,
    );
    let seen = run(&mut start, 1, 6000, &[]);
    assert_eq!(frames_at(&seen, Part::Logo)[..2], [67, 134]);
    assert!(sound_at(&seen, Op::Sound(Part::Logo)).is_empty());
    assert!(sound_at(&seen, Op::Quiet(Part::Logo)).is_empty());
}

#[test]
fn the_dll_s_position_is_carried_on_by_the_clock_between_the_service_s_runs() {
    let mut s = SmackSound::new(TRACK, 6666);
    assert_eq!(s.played(), None);
    assert_eq!(s.heard_by(50), 0);
    // The open's reading: the service runs, nothing to play; the frame fills the ring.
    assert_eq!(s.wait(0), (true, false));
    s.frame(0);
    // Throttled until 10 ms after its run; then the Play.
    assert_eq!(s.wait(9), (false, false));
    assert_eq!(s.wait(10), (false, true));
    assert_eq!(s.played(), Some(10));
    // At 20 the service reads 880 bytes (whole samples); at 25 the position is carried on from
    // them by the clock.
    assert_eq!(s.position(20), (880, false));
    assert_eq!(s.position(25), (880 + 441, false));
    // Never back: at 29 the service has not run again (1 321 + 352), at 30 it reads 1 764.
    assert_eq!(s.position(29), (1673, false));
    assert_eq!(s.position(30), (1764, false));
    assert_eq!(s.heard_by(76), 5820);
    assert_eq!(s.wait(76), (false, false));
    assert_eq!(s.wait(77), (true, false));
    // Around the buffer (45 056 bytes): the bytes keep counting.
    let mut t = 77;
    while t < 2000 {
        t += 10;
        let _ = s.wait(t);
    }
    assert_eq!(s.heard, s.heard_by(t));
    assert!(s.heard > 45_056 * 3);
    assert_eq!(s.size, 45_056);
}

#[test]
fn a_skip_stops_the_sound_and_a_bink_clip_sounds_from_its_first_frame() {
    let headers = [header(81, SMACK), header(30, BINK), None];
    let sounds = [Some(TRACK), Some(TRACK), None];
    // A skip of the 3DO logo after its Play: the sound stops, then the fade.
    let mut ops = Vec::new();
    let mut start = Start::with_sound(headers, sounds, true, Bugs::Original, 0, &mut ops);
    let seen = run(&mut start, 10, 3000, &[500]);
    let quiet = seen
        .iter()
        .position(|(_, op)| *op == Op::Quiet(Part::Logo))
        .unwrap();
    assert_eq!(seen[quiet].0, 500);
    assert_eq!(seen[quiet + 1].1, Op::Fade(0));
    // Skipped at the open's reading, before its Play: nothing sounded, nothing to stop.
    let mut ops = Vec::new();
    let mut start = Start::with_sound(headers, sounds, true, Bugs::Original, 0, &mut ops);
    start.skip(0, &mut ops);
    assert!(!ops.contains(&Op::Quiet(Part::Logo)));
    // Run through: NWCLogo's sound with its first frame, stopped at its last, its frames by its
    // clock as without sound.
    let mut ops = Vec::new();
    let mut start = Start::with_sound(headers, sounds, true, Bugs::Original, 0, &mut ops);
    let seen = run(&mut start, 1, 20_000, &[]);
    let first = frames_at(&seen, Part::Nwc);
    assert_eq!(sound_at(&seen, Op::Sound(Part::Nwc)), [first[0]]);
    assert_eq!(sound_at(&seen, Op::Closed(Part::Nwc)), [first[29]]);
    assert_eq!(first[1] - first[0], 66);
    // H3Intro is skipped at once: its sound stops with it.
    let headers = [header(1, SMACK), header(1, BINK), header(100, BINK)];
    let mut ops = Vec::new();
    let mut start = Start::with_sound(headers, [None, None, Some(TRACK)], true, Bugs::Original, 0, &mut ops);
    let mut seen = run(&mut start, 10, 900, &[]);
    assert!(seen.iter().any(|(_, op)| *op == Op::Sound(Part::Intro)));
    let mut ops = Vec::new();
    start.skip(1000, &mut ops);
    seen.extend(ops.into_iter().map(|op| (1000, op)));
    assert_eq!(seen.last(), Some(&(1000, Op::Quiet(Part::Intro))));
    assert!(start.ended());
}
