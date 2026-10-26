use super::*;

use alloc::string::ToString;
use alloc::vec;

use crate::layout::AdventureLayout;
use crate::sequence::Sequences;
use crate::situation::Situation;
use crate::tables;

/// The committed tables and layout (`clients/map/`).
fn rules(bugs: Bugs) -> Rules {
    let text = include_str!("../../../../map/assets/base.toml");
    let tables = tables::load(&[tables::parse(text).unwrap()], "base").unwrap();
    let layout =
        AdventureLayout::parse(include_str!("../../../../map/layouts/adventure.toml")).unwrap();
    Rules::new(&tables, &layout, bugs)
}

/// The recorded sequences and their situations and timelines (no original file).
const SEQUENCES: &str = include_str!("../../../../map/sequences.toml");
const RECORDED: [(&str, &str, &str); 6] = [
    (
        "idle-cycles",
        include_str!("../../../../map/situations/seq-idle-cycles.toml"),
        include_str!("../../../../map/timelines/idle-cycles.txt"),
    ),
    (
        "idle-objects",
        include_str!("../../../../map/situations/objects-flat.toml"),
        include_str!("../../../../map/timelines/idle-objects.txt"),
    ),
    (
        "scroll-0",
        include_str!("../../../../map/situations/seq-scroll.toml"),
        include_str!("../../../../map/timelines/scroll-0.txt"),
    ),
    (
        "scroll-1",
        include_str!("../../../../map/situations/seq-scroll.toml"),
        include_str!("../../../../map/timelines/scroll-1.txt"),
    ),
    (
        "scroll-2",
        include_str!("../../../../map/situations/seq-scroll.toml"),
        include_str!("../../../../map/timelines/scroll-2.txt"),
    ),
    (
        "scroll-limits",
        include_str!("../../../../map/situations/seq-scroll.toml"),
        include_str!("../../../../map/timelines/scroll-limits.txt"),
    ),
];

/// Where the harness parks the pointer (`h3adv.py`'s `PARK`).
const PARK: [i32; 2] = [700, 300];

fn start(situation: &str, speed: usize) -> Start {
    let s = Situation::parse(situation).unwrap();
    let [x, y, z] = s.view;
    Start {
        view: ViewAt {
            x,
            y,
            z: u8::try_from(z).unwrap(),
        },
        size: i32::from(s.size),
        scroll_speed: speed,
        pointer: PARK,
    }
}

/// A recorded timeline without its `screen` lines (the frames': the binary draws them).
fn without_screens(recorded: &str) -> Vec<String> {
    recorded
        .lines()
        .filter(|l| !l.contains(" screen "))
        .map(ToString::to_string)
        .collect()
}

#[test]
fn replays_the_recorded_timelines() {
    let sequences = Sequences::parse(SEQUENCES).unwrap();
    for (id, situation, recorded) in RECORDED {
        let sequence = sequences.get(id).unwrap();
        assert!(!sequence.walks(), "{id}");
        let states = play(
            rules(Bugs::Original),
            &start(situation, sequence.scroll_speed().unwrap()),
            &sequence.actions().unwrap(),
            sequence.ms,
        )
        .unwrap();
        assert_eq!(states.len(), usize::try_from(sequence.ms).unwrap() + 1);
        let got = lines(&states, &BTreeMap::new());
        let want = without_screens(recorded);
        for (n, (g, w)) in got.iter().zip(&want).enumerate() {
            assert_eq!(g, w, "{id}: line {n}");
        }
        assert_eq!(got.len(), want.len(), "{id}");
    }
}

#[test]
fn the_walks_are_m2_t3s() {
    let sequences = Sequences::parse(SEQUENCES).unwrap();
    let walks: Vec<&str> = sequences
        .sequence
        .iter()
        .filter(|s| s.walks())
        .map(|s| s.id.as_str())
        .collect();
    assert_eq!(
        walks,
        [
            "walk-1",
            "walk-2",
            "walk-3",
            "walk-4",
            "walk-follow",
            "walk-points",
            "walk-sail"
        ]
    );
    let walk = sequences.get("walk-2").unwrap();
    let situation = include_str!("../../../../map/situations/seq-walk.toml");
    let first = *walk.actions().unwrap().keys().next().unwrap();
    assert_eq!(
        play(
            rules(Bugs::Original),
            &start(situation, 1),
            &walk.actions().unwrap(),
            walk.ms
        ),
        Err((first, Unplayable::Walk))
    );
}

/// A loop over a 36-tile map at (9, 10), the pointer parked.
fn looped(bugs: Bugs, speed: usize) -> Loop {
    let start = Start {
        view: ViewAt { x: 9, y: 10, z: 0 },
        size: 36,
        scroll_speed: speed,
        pointer: PARK,
    };
    Loop::start(rules(bugs), &start).unwrap()
}

#[test]
fn the_redraws_and_the_palettes_on_their_grids() {
    let mut l = looped(Bugs::Fixed, 1);
    let s = l.state(0);
    assert_eq!(
        (s.objects, s.cycles, s.drawn.frames, s.drawn.cycles),
        (0, 0, 0, 0)
    );
    for t in 1..=400 {
        l.run(t, &[]).unwrap();
        let s = l.state(t);
        // The redraws at 180 and 360 draw the steps +0x100 held (1, 2) and step it after.
        let drawn = match t {
            0..180 => 0,
            180..360 => 1,
            _ => 2,
        };
        assert_eq!(s.drawn.frames, drawn, "{t}");
        assert_eq!(s.objects, drawn, "{t}");
        // The palettes step at 200 and 400; the screen shows a step at the next redraw.
        assert_eq!(s.cycles, t / 200, "{t}");
        assert_eq!(s.drawn.cycles, u32::from(t >= 360), "{t}");
    }
}

#[test]
fn a_late_redraw_steps_the_objects_for_each_step_missed() {
    // The window's clock may jump (T4): at 400 the redraw due at 180 draws the step it held, then
    // steps twice (the deadline moved by the lateness, 220, then by 180: 580); one palette step.
    let mut l = looped(Bugs::Fixed, 1);
    l.run(400, &[]).unwrap();
    let s = l.state(400);
    assert_eq!(
        (s.drawn.frames, s.objects, s.cycles, s.drawn.cycles),
        (1, 2, 1, 1)
    );
    l.run(579, &[]).unwrap();
    assert_eq!(l.state(579).objects, 2);
    l.run(580, &[]).unwrap();
    assert_eq!(l.state(580).objects, 3);
}

#[test]
fn a_key_scrolls_by_the_speeds_step_and_stops_at_the_limits() {
    for (speed, step) in [(0, 1), (1, 2), (2, 3)] {
        let mut l = looped(Bugs::Fixed, speed);
        l.run(1, &[Action::Scroll(Direction::NorthWest)]).unwrap();
        let s = l.state(1);
        assert_eq!((s.view.x, s.view.y), (9 - step, 10 - step));
        // Drawn at once, at the objects' step +0x100 holds, without stepping it (not due).
        assert_eq!((s.drawn.view, s.drawn.frames, s.objects), (s.view, 1, 0));
        // The cursor stays the arrow: a key's step shows no edge cursor.
        assert_eq!(s.cursor.frame, 0);
    }
    let mut l = looped(Bugs::Fixed, 2);
    for t in 1..=20 {
        l.run(t, &[Action::Scroll(Direction::SouthEast)]).unwrap();
    }
    let s = l.state(20);
    assert_eq!((s.view.x, s.view.y), (26, 27));
    for t in 21..=40 {
        l.run(t, &[Action::Scroll(Direction::NorthWest)]).unwrap();
    }
    let s = l.state(40);
    assert_eq!((s.view.x, s.view.y), (-9, -8));
    // At the limit nothing is drawn: the last draw stays the one that moved.
    l.run(41, &[Action::Scroll(Direction::North)]).unwrap();
    assert_eq!(l.state(41).drawn, s.drawn);
}

#[test]
fn a_scroll_at_the_redraws_time_takes_its_place() {
    // The pointer at the left edge from 40: steps at 110, 180 (the redraw's time: the edge's
    // step comes first and draws the step +0x100 held, then steps it), 250.
    let mut l = looped(Bugs::Fixed, 0);
    for t in 1..=250 {
        let given: &[Action] = if t == 40 {
            &[Action::Pointer([2, 300])]
        } else {
            &[]
        };
        l.run(t, given).unwrap();
        let s = l.state(t);
        if t == 180 {
            assert_eq!((s.view.x, s.drawn.frames, s.objects), (7, 1, 1));
        }
    }
    let s = l.state(250);
    assert_eq!((s.view.x, s.cursor.frame), (6, 38));
}

#[test]
fn the_edge_zones() {
    let fixed = rules(Bugs::Fixed);
    let original = rules(Bugs::Original);
    let at = |r: &Rules, p| r.edge_at(p);
    // The client's: 16 pixels on every side.
    assert_eq!(at(&fixed, [15, 300]), Some(Direction::West));
    assert_eq!(at(&fixed, [16, 300]), None);
    assert_eq!(at(&fixed, [784, 300]), Some(Direction::East));
    assert_eq!(at(&fixed, [783, 300]), None);
    assert_eq!(at(&fixed, [400, 15]), Some(Direction::North));
    assert_eq!(at(&fixed, [400, 584]), Some(Direction::South));
    assert_eq!(at(&fixed, [400, 583]), None);
    // The original's (MCL-Q6): x rounded down to even, then x > 784 and y > 584.
    assert_eq!(at(&original, [15, 300]), Some(Direction::West));
    assert_eq!(at(&original, [785, 300]), None);
    assert_eq!(at(&original, [786, 300]), Some(Direction::East));
    assert_eq!(at(&original, [400, 584]), None);
    assert_eq!(at(&original, [400, 585]), Some(Direction::South));
    // The corners, off the frame.
    for (p, d) in [
        ([0, 0], Direction::NorthWest),
        ([799, 0], Direction::NorthEast),
        ([799, 599], Direction::SouthEast),
        ([0, 599], Direction::SouthWest),
        ([400, 0], Direction::North),
        ([799, 300], Direction::East),
    ] {
        assert_eq!(
            (at(&fixed, p), at(&original, p)),
            (Some(d), Some(d)),
            "{p:?}"
        );
    }
    for p in [[-1, 300], [800, 300], [400, -1], [400, 600], [400, 300]] {
        assert_eq!((at(&fixed, p), at(&original, p)), (None, None), "{p:?}");
    }
    // The cursor's point: the original's x even.
    assert_eq!(fixed.drawn_at([701, 3]), [701, 3]);
    assert_eq!(original.drawn_at([701, 3]), [700, 3]);
}

#[test]
fn the_cursor_follows_the_poll() {
    let mut l = looped(Bugs::Original, 1);
    for t in 1..=40 {
        let given: &[Action] = if t == 10 {
            &[Action::Pointer([3, 300])]
        } else {
            &[]
        };
        l.run(t, given).unwrap();
        let s = l.state(t);
        let at = if t < 33 { PARK } else { [2, 300] };
        assert_eq!(s.cursor, Cursor { frame: 0, at }, "{t}");
    }
    // The first edge step 70 ms after the pointer's arrival, its cursor at once.
    for t in 41..=80 {
        l.run(t, &[]).unwrap();
    }
    let s = l.state(80);
    assert_eq!((s.view.x, s.cursor.frame), (7, 38));
    // Off the zones the arrow; within one an edge cursor kept.
    l.run(81, &[Action::Pointer([2, 310])]).unwrap();
    assert_eq!(l.state(81).cursor.frame, 38);
    l.run(82, &[Action::Pointer([700, 300])]).unwrap();
    assert_eq!(l.state(82).cursor.frame, 0);
}

#[test]
fn what_it_cannot_play_yet() {
    let mut l = looped(Bugs::Fixed, 1);
    assert_eq!(
        l.run(1, &[Action::Step(Direction::North)]),
        Err(Unplayable::Walk)
    );
    assert_eq!(
        l.run(2, &[Action::Click([300, 300])]),
        Err(Unplayable::Walk)
    );
    assert_eq!(
        l.run(3, &[Action::Pointer([300, 300])]),
        Err(Unplayable::OverView([300, 300]))
    );
    let over = Start {
        view: ViewAt { x: 0, y: 0, z: 0 },
        size: 36,
        scroll_speed: 1,
        pointer: [300, 300],
    };
    assert_eq!(
        Loop::start(rules(Bugs::Fixed), &over),
        Err(Unplayable::OverView([300, 300]))
    );
    assert_eq!(
        play(rules(Bugs::Fixed), &over, &BTreeMap::new(), 10),
        Err((0, Unplayable::OverView([300, 300])))
    );
    assert_eq!(Unplayable::Walk.to_string(), "a hero's walk (M2 T3)");
    assert_eq!(
        Unplayable::OverView([1, 2]).to_string(),
        "the pointer over the view at (1, 2) (M3)"
    );
}

#[test]
fn the_directions() {
    let numbers: Vec<u32> = Direction::ALL.iter().map(|d| d.number()).collect();
    assert_eq!(numbers, [0, 1, 2, 3, 4, 5, 6, 7]);
    let sum = Direction::ALL
        .iter()
        .fold([0, 0], |[x, y], d| [x + d.delta()[0], y + d.delta()[1]]);
    assert_eq!(sum, [0, 0]);
    assert_eq!(Direction::NorthEast.delta(), [1, -1]);
}

#[test]
fn the_lines_with_the_screens() {
    let mut l = looped(Bugs::Fixed, 1);
    let mut states = vec![l.state(0)];
    l.run(1, &[Action::Scroll(Direction::East)]).unwrap();
    states.push(l.state(1));
    let screens = BTreeMap::from([(0, [0, 0, 1, 1]), (1, [8, 8, 600, 552])]);
    assert_eq!(
        lines(&states, &screens),
        [
            "0 view 9 10 0",
            "0 objects 0",
            "0 cycles 0 0",
            "0 cursor 1 0 700 300",
            "1 view 11 10 0",
            "1 screen 8,8,600,552",
        ]
    );
}

#[test]
fn the_box_two_frames_differ_in() {
    use h3_render::screen::PixelFormat;
    let format = PixelFormat::RGB565;
    let before = Screen::new(10, 8, format).unwrap();
    let mut after = before.clone();
    assert_eq!(changed(&before, &after, [0, 0, 10, 8]), None);
    after.set(2, 3, 1);
    after.set(6, 1, 1);
    after.set(9, 7, 1);
    assert_eq!(changed(&before, &after, [0, 0, 10, 8]), Some([2, 1, 10, 8]));
    // Outside the area nothing counts.
    assert_eq!(changed(&before, &after, [0, 0, 9, 7]), Some([2, 1, 7, 4]));
    assert_eq!(changed(&before, &after, [7, 0, 9, 7]), None);
    let other = Screen::new(10, 9, format).unwrap();
    assert_eq!(changed(&before, &other, [0, 0, 4, 4]), Some([0, 0, 4, 4]));
}

#[test]
fn a_speed_without_a_step_scrolls_a_tile() {
    let mut l = looped(Bugs::Fixed, 7);
    l.run(1, &[Action::Scroll(Direction::South)]).unwrap();
    assert_eq!(l.state(1).view, ViewAt { x: 9, y: 11, z: 0 });
}
