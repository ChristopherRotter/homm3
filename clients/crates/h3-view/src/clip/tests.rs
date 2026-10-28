use super::*;

const RATE: [u32; 2] = [15, 1];
const ORIGINAL: Bugs = Bugs::Original;

fn timing(frames: usize) -> Timing {
    Timing { frames, rate: RATE }
}

#[test]
fn each_fate_has_its_clip_as_its_tune() {
    let names = [
        (Tune::Win, "Win3.bik", None),
        (Tune::LoseCastle, "LoseCstl.bik", Some("LoseCslp.bik")),
        (Tune::Retreat, "rtstart.bik", Some("rtloop.bik")),
        (Tune::Surrender, "Surrender.bik", None),
        (Tune::DefendCastle, "Defendall.bik", Some("defendloop.bik")),
        (Tune::Lose, "lbstart.bik", Some("lbloop.bik")),
    ];
    for (tune, start, looped) in names {
        assert_eq!(of(tune), &Clip { start, looped }, "{tune:?}");
    }
}

#[test]
fn the_first_frame_stays_three_frames_and_the_rest_follow_at_the_rate() {
    // The retreat's clips, 40 frames each, as captured (100 ms: frame 0; 200 ms: 1; 300 ms: 2;
    // 2 700 ms: 38) and as binkprobe measured the DLL (two redraws, then ticks).
    let (start, looped) = (timing(40), Some(timing(40)));
    let at = |since| shown(since, start, looped, ORIGINAL);
    assert_eq!(at(0), Some((Part::Start, 0)));
    assert_eq!(at(199), Some((Part::Start, 0)));
    assert_eq!(at(200), Some((Part::Start, 1)));
    assert_eq!(at(265), Some((Part::Start, 1)));
    assert_eq!(at(266), Some((Part::Start, 2)));
    assert_eq!(at(300), Some((Part::Start, 2)));
    assert_eq!(at(2700), Some((Part::Start, 38)));
    // The start's last frame (39) never shows: the loop's first at once (2 733 ms), then on.
    assert_eq!(at(2732), Some((Part::Start, 38)));
    assert_eq!(at(2733), Some((Part::Loop, 0)));
    assert_eq!(at(2798), Some((Part::Loop, 0)));
    assert_eq!(at(2799), Some((Part::Loop, 1)));
    // The loop circles: its frame 40 is its frame 0 again (2 733 + 2 666 ms).
    assert_eq!(at(5398), Some((Part::Loop, 39)));
    assert_eq!(at(5399), Some((Part::Loop, 0)));
    assert_eq!(at(u32::MAX).map(|(part, _)| part), Some(Part::Loop));
}

#[test]
fn a_start_without_a_loop_circles_its_last_frame_shown() {
    // `Win3`, 73 frames: its frame 72 from 4 933 ms (75 × 66.7 − 66.7), its frame 0 again from
    // 5 000 ms.
    let start = timing(73);
    assert_eq!(shown(4932, start, None, ORIGINAL), Some((Part::Start, 71)));
    assert_eq!(shown(4933, start, None, ORIGINAL), Some((Part::Start, 72)));
    assert_eq!(shown(4999, start, None, ORIGINAL), Some((Part::Start, 72)));
    assert_eq!(shown(5000, start, None, ORIGINAL), Some((Part::Start, 0)));
    assert_eq!(shown(5066, start, None, ORIGINAL), Some((Part::Start, 1)));
}

#[test]
fn a_one_frame_start_hands_over_at_its_tick() {
    let (start, looped) = (timing(1), Some(timing(3)));
    assert_eq!(shown(132, start, looped, ORIGINAL), Some((Part::Start, 0)));
    assert_eq!(shown(133, start, looped, ORIGINAL), Some((Part::Loop, 0)));
    assert_eq!(shown(199, start, looped, ORIGINAL), Some((Part::Loop, 1)));
}

#[test]
fn empty_clips_and_a_rate_of_zero_show_nothing_or_stand_still() {
    assert_eq!(shown(0, timing(0), None, ORIGINAL), None);
    assert_eq!(shown(5000, timing(2), Some(timing(0)), ORIGINAL), None);
    let still = Timing {
        frames: 5,
        rate: [0, 1],
    };
    assert_eq!(
        shown(100_000, still, Some(timing(5)), ORIGINAL),
        Some((Part::Start, 0))
    );
    let halted = Timing {
        frames: 5,
        rate: [15, 0],
    };
    assert_eq!(
        shown(100_000, halted, None, ORIGINAL),
        Some((Part::Start, 0))
    );
    // Another rate: 10 a second (frame 1 from 300 ms).
    let slow = Timing {
        frames: 5,
        rate: [10, 1],
    };
    assert_eq!(shown(299, slow, None, ORIGINAL), Some((Part::Start, 0)));
    assert_eq!(shown(300, slow, None, ORIGINAL), Some((Part::Start, 1)));
}

#[test]
fn pixels_are_the_dlls() {
    // `binkprobe`'s run of BINKW32.DLL over every (Y, U, V) (surface type 3), some of its pixels:
    // grey, black, white, saturated and clamped at both ends.
    let dll: [(u8, u8, u8, u16); 13] = [
        (128, 128, 128, 0x8410),
        (16, 128, 128, 0x0000),
        (17, 128, 128, 0x0000),
        (234, 128, 128, 0xFFFF),
        (255, 128, 128, 0xFFFF),
        (60, 200, 90, 0x0034),
        (200, 30, 210, 0xFFE7),
        (13, 113, 139, 0x1040),
        (120, 24, 205, 0xFD60),
        (234, 219, 87, 0xAE3F),
        (90, 140, 160, 0x920D),
        (0, 0, 255, 0xF9A0),
        (255, 255, 0, 0x171F),
    ];
    for (y, u, v, expected) in dll {
        assert_eq!(pixel(y, u, v), expected, "({y}, {u}, {v})");
    }
}

#[test]
fn a_frame_takes_its_blocks_chroma_and_decodes_as_a_clip() {
    let video = Video::decode(&h3_bink::build::sample(3)).unwrap();
    assert_eq!((video.width, video.height, video.rate), (256, 48, [15, 1]));
    assert_eq!(video.frames.len(), 8);
    assert!(video.frames.iter().all(|f| f.len() == 256 * 48));
    assert_eq!(
        video.timing(),
        Timing {
            frames: 8,
            rate: RATE
        }
    );
    // Each pixel from its own luma and its 2 × 2 block's chroma.
    let bytes = h3_bink::build::sample(3);
    let clip = BinkClip::parse(&bytes).unwrap();
    let mut decoder = Decoder::new(&clip).unwrap();
    let frame = decoder.decode(clip.video(0).unwrap()).unwrap();
    let [y, u, v] = frame.planes();
    for (row, col) in [(0, 0), (1, 1), (47, 255), (20, 101)] {
        let expected = pixel(
            y.row(row)[col],
            u.row(row / 2)[col / 2],
            v.row(row / 2)[col / 2],
        );
        assert_eq!(video.frames[0][row * 256 + col], expected);
    }
    assert!(Video::decode(b"BIKb").is_err());
    assert_eq!(Video::default().timing().frames, 0);
}

#[test]
fn decodes_are_the_times_to_draw_at() {
    let (start, looped) = (timing(40), Some(timing(40)));
    // The two redraws at the opening, the tick's re-decode of frame 0 at 133, frame 1 at 200.
    assert_eq!(decodes(0, start, looped, ORIGINAL), (0, Some(133)));
    assert_eq!(decodes(150, start, looped, ORIGINAL), (133, Some(200)));
    assert_eq!(decodes(200, start, looped, ORIGINAL), (200, Some(266)));
    // The hand-over at 2 733 ms, the loop's frame 1 at 2 799.
    assert_eq!(decodes(2700, start, looped, ORIGINAL), (2666, Some(2733)));
    assert_eq!(decodes(2733, start, looped, ORIGINAL), (2733, Some(2799)));
    assert_eq!(decodes(2800, start, looped, ORIGINAL), (2799, Some(2866)));
    // Without a loop the start's decodes go on; a rate of 0 never decodes again.
    assert_eq!(
        decodes(5000, timing(73), None, ORIGINAL),
        (5000, Some(5066))
    );
    let still = Timing {
        frames: 5,
        rate: [0, 1],
    };
    assert_eq!(decodes(9000, still, None, ORIGINAL), (0, None));
    // Past what a u32 holds the next is none.
    assert_eq!(decodes(u32::MAX, timing(73), None, ORIGINAL).1, None);
    let slow = Timing {
        frames: 2,
        rate: [1, 100_000],
    };
    // One frame in 100 000 s: the loop's from 300 000 000 ms, its 39th after it.
    assert_eq!(
        decodes(u32::MAX, slow, Some(slow), ORIGINAL),
        (4_200_000_000, None)
    );
}

#[test]
fn the_client_fixes_the_slips_frame_0_one_period_the_last_shown() {
    // CLT-Q31: every frame its period from the opening, the start's last before the loop's first.
    let (start, looped) = (timing(40), Some(timing(40)));
    let at = |since| shown(since, start, looped, Bugs::Fixed);
    assert_eq!(at(0), Some((Part::Start, 0)));
    assert_eq!(at(65), Some((Part::Start, 0)));
    assert_eq!(at(66), Some((Part::Start, 1)));
    assert_eq!(at(2599), Some((Part::Start, 38)));
    assert_eq!(at(2600), Some((Part::Start, 39)));
    assert_eq!(at(2666), Some((Part::Loop, 0)));
    assert_eq!(at(2733), Some((Part::Loop, 1)));
    assert_eq!(
        shown(4866, timing(73), None, Bugs::Fixed),
        Some((Part::Start, 0))
    );
    assert_eq!(decodes(0, start, looped, Bugs::Fixed), (0, Some(66)));
    assert_eq!(
        decodes(2650, start, looped, Bugs::Fixed),
        (2600, Some(2666))
    );
    assert_eq!(
        decodes(2700, start, looped, Bugs::Fixed),
        (2666, Some(2732))
    );
}
