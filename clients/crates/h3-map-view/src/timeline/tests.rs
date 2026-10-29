use super::*;

use alloc::string::ToString;
use alloc::vec;

use crate::layout::AdventureLayout;
use crate::live::{Facing, Kingdom, KingdomHero, LiveHero};
use crate::sequence::{self, Sequences};
use crate::situation::Situation;
use crate::tables;

/// The committed tables and layout (`clients/map/`).
pub(crate) fn rules(bugs: Bugs) -> Rules {
    let text = include_str!("../../../../map/assets/base.toml");
    let tables = tables::load(&[tables::parse(text).unwrap()], "base").unwrap();
    let layout =
        AdventureLayout::parse(include_str!("../../../../map/layouts/adventure.toml")).unwrap();
    Rules::new(&tables, &layout, bugs)
}

/// The recorded sequences and their situations and timelines (no original file).
const SEQUENCES: &str = include_str!("../../../../map/sequences.toml");
const RECORDED: [(&str, &str, &str); 13] = [
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
    (
        "walk-1",
        include_str!("../../../../map/situations/seq-walk.toml"),
        include_str!("../../../../map/timelines/walk-1.txt"),
    ),
    (
        "walk-2",
        include_str!("../../../../map/situations/seq-walk.toml"),
        include_str!("../../../../map/timelines/walk-2.txt"),
    ),
    (
        "walk-3",
        include_str!("../../../../map/situations/seq-walk.toml"),
        include_str!("../../../../map/timelines/walk-3.txt"),
    ),
    (
        "walk-4",
        include_str!("../../../../map/situations/seq-walk.toml"),
        include_str!("../../../../map/timelines/walk-4.txt"),
    ),
    (
        "walk-follow",
        include_str!("../../../../map/situations/seq-walk-follow.toml"),
        include_str!("../../../../map/timelines/walk-follow.txt"),
    ),
    (
        "walk-points",
        include_str!("../../../../map/situations/seq-walk.toml"),
        include_str!("../../../../map/timelines/walk-points.txt"),
    ),
    (
        "walk-sail",
        include_str!("../../../../map/situations/seq-sail.toml"),
        include_str!("../../../../map/timelines/walk-sail.txt"),
    ),
];

/// The walks' live parts at 0 (`h3map.py frames`): their selected heroes.
const LIVE: [(&str, &str); 13] = [
    (
        "idle-cycles",
        include_str!("../../../../map/timelines/idle-cycles-live.toml"),
    ),
    (
        "idle-objects",
        include_str!("../../../../map/timelines/idle-objects-live.toml"),
    ),
    (
        "scroll-0",
        include_str!("../../../../map/timelines/scroll-0-live.toml"),
    ),
    (
        "scroll-1",
        include_str!("../../../../map/timelines/scroll-1-live.toml"),
    ),
    (
        "scroll-2",
        include_str!("../../../../map/timelines/scroll-2-live.toml"),
    ),
    (
        "scroll-limits",
        include_str!("../../../../map/timelines/scroll-limits-live.toml"),
    ),
    (
        "walk-1",
        include_str!("../../../../map/timelines/walk-1-live.toml"),
    ),
    (
        "walk-2",
        include_str!("../../../../map/timelines/walk-2-live.toml"),
    ),
    (
        "walk-3",
        include_str!("../../../../map/timelines/walk-3-live.toml"),
    ),
    (
        "walk-4",
        include_str!("../../../../map/timelines/walk-4-live.toml"),
    ),
    (
        "walk-follow",
        include_str!("../../../../map/timelines/walk-follow-live.toml"),
    ),
    (
        "walk-points",
        include_str!("../../../../map/timelines/walk-points-live.toml"),
    ),
    (
        "walk-sail",
        include_str!("../../../../map/timelines/walk-sail-live.toml"),
    ),
];

/// A live part's heroes and viewer (`[x, y, level, class, facing, owner, boat, colour]` rows, as
/// the binary's `read_live` takes them).
/// A kept live part's kingdom and buttons (E24 M3 T2a).
fn live_kingdom(text: &str) -> (Kingdom, Option<Buttons>) {
    let table: toml::Table = text.parse().unwrap();
    let kingdom = Kingdom::read(&table["kingdom"]).unwrap();
    let flags: Vec<i64> = table["buttons"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_integer().unwrap())
        .collect();
    let buttons = Buttons {
        move_hero: flags[0] == 1,
        next_hero: flags[1] == 1,
    };
    (kingdom, Some(buttons))
}

fn live_heroes(text: &str) -> (Vec<LiveHero>, u8) {
    let table: toml::Table = text.parse().unwrap();
    let number = |v: &toml::Value| v.as_integer().unwrap();
    let heroes = table["heroes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let row: Vec<i64> = row.as_array().unwrap().iter().map(number).collect();
            let byte = |i: usize| u8::try_from(row[i]).unwrap();
            LiveHero {
                at: Pos {
                    x: byte(0),
                    y: byte(1),
                    z: byte(2),
                },
                class: byte(3),
                facing: Facing::from_number(byte(4)).unwrap(),
                owner: byte(5),
                boat: None,
            }
        })
        .collect();
    (heroes, u8::try_from(number(&table["viewer"])).unwrap())
}

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
        hero_speed: 2,
        pointer: PARK,
        hero: None,
        paths: Vec::new(),
        path: Vec::new(),
        live: false,
        sound: None,
        kingdom: Kingdom::default(),
        buttons: None,
    }
}

/// A recorded sequence's start, its actions and its end, as `replays_the_recorded_timelines` plays
/// it (the replay's tests, E24 M2 T4a).
pub(crate) fn recorded(id: &str) -> (Start, BTreeMap<u32, Vec<Action>>, u32) {
    let sequences = Sequences::parse(SEQUENCES).unwrap();
    let sequence = sequences.get(id).unwrap();
    let (_, situation, timeline) = RECORDED.iter().find(|(r, _, _)| *r == id).unwrap();
    let mut start = start(situation, sequence.scroll_speed().unwrap());
    start.hero_speed = sequence.hero_speed().unwrap();
    let (_, live) = LIVE.iter().find(|(l, _)| *l == id).unwrap();
    (start.kingdom, start.buttons) = live_kingdom(live);
    if sequence.walks() {
        let (heroes, viewer) = live_heroes(live);
        start.hero = Some(sequence::selected(&heroes, viewer).unwrap());
        start.paths = sequence.paths(timeline).unwrap();
    }
    (start, sequence.actions().unwrap(), sequence.ms)
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
        let mut start = start(situation, sequence.scroll_speed().unwrap());
        start.hero_speed = sequence.hero_speed().unwrap();
        let (_, live) = LIVE.iter().find(|(l, _)| *l == id).unwrap();
        (start.kingdom, start.buttons) = live_kingdom(live);
        if sequence.walks() {
            let (heroes, viewer) = live_heroes(live);
            start.hero = Some(sequence::selected(&heroes, viewer).unwrap());
            start.paths = sequence.paths(recorded).unwrap();
        }
        let states = play(
            rules(Bugs::Original),
            &start,
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
fn the_walks_frames_are_m2_t3bs() {
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
}

/// A loop over a 36-tile map at (9, 10), the pointer parked.
fn looped(bugs: Bugs, speed: usize) -> Loop {
    let start = Start {
        view: ViewAt { x: 9, y: 10, z: 0 },
        size: 36,
        scroll_speed: speed,
        hero_speed: 2,
        pointer: PARK,
        hero: None,
        paths: Vec::new(),
        path: Vec::new(),
        live: false,
        sound: None,
        kingdom: Kingdom::default(),
        buttons: None,
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
fn a_late_redraw_draws_again_for_each_step_missed() {
    // The window's clock may jump (T4), and a walk keeps the idle step from running: at 400 the
    // redraw due at 180 draws the step it held and steps once, the deadline moved by the lateness
    // (220) to the clock, so the loop's next turn redraws at once and steps again (the deadline
    // then by 180: 580); the screen shows the second draw (M2 T3b: the walks' ends). One palette
    // step.
    let mut l = looped(Bugs::Fixed, 1);
    l.run(400, &[]).unwrap();
    let s = l.state(400);
    assert_eq!(
        (s.drawn.frames, s.objects, s.cycles, s.drawn.cycles),
        (2, 2, 1, 1)
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
        assert_eq!(
            s.cursor,
            Cursor {
                frame: 0,
                at,
                shown: true
            },
            "{t}"
        );
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
        l.run(3, &[Action::Pointer([300, 300])]),
        Err(Unplayable::OverView([300, 300]))
    );
    let over = Start {
        view: ViewAt { x: 0, y: 0, z: 0 },
        size: 36,
        scroll_speed: 1,
        hero_speed: 2,
        pointer: [300, 300],
        hero: None,
        paths: Vec::new(),
        path: Vec::new(),
        live: false,
        sound: None,
        kingdom: Kingdom::default(),
        buttons: None,
    };
    assert_eq!(
        Loop::start(rules(Bugs::Fixed), &over),
        Err(Unplayable::OverView([300, 300]))
    );
    assert_eq!(
        play(rules(Bugs::Fixed), &over, &BTreeMap::new(), 10),
        Err((0, Unplayable::OverView([300, 300])))
    );
    let mut parked = over.clone();
    parked.pointer = PARK;
    let later = BTreeMap::from([(5, vec![Action::Click([700, 2])])]);
    assert_eq!(
        play(rules(Bugs::Fixed), &parked, &later, 10),
        Err((5, Unplayable::Panel([700, 2])))
    );
    assert_eq!(
        Unplayable::OverView([1, 2]).to_string(),
        "the pointer over the view at (1, 2) (M3)"
    );
    assert_eq!(
        Unplayable::Panel([700, 2]).to_string(),
        "a click off the view at (700, 2) (M3)"
    );
    assert_eq!(
        Unplayable::NoPath([3, 4, 0]).to_string(),
        "no path given from the selected hero to (3, 4, 0)"
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
            "0 buttons 0 0",
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
    assert_eq!(changed(&before, &after, &[]), None);
    after.set(2, 3, 1);
    after.set(6, 1, 1);
    after.set(9, 7, 1);
    assert_eq!(changed(&before, &after, &[]), Some([2, 1, 10, 8]));
    // Inside the masked rectangles nothing counts (E24 M3 T2a).
    assert_eq!(
        changed(&before, &after, &[[9, 7, 10, 8]]),
        Some([2, 1, 7, 4])
    );
    let all = [[0, 0, 7, 8], [7, 0, 10, 8]];
    assert_eq!(changed(&before, &after, &all), None);
    // Frames of other sizes differ in all of the later one.
    let other = Screen::new(10, 9, format).unwrap();
    assert_eq!(changed(&before, &other, &[]), Some([0, 0, 10, 9]));
}

#[test]
fn a_speed_out_of_range_is_the_originals_default() {
    // `settings_check` 0x50B260 (M2 T6a, the reviewer's R1-7): a scroll speed out of range is
    // speed 1's step (2 tiles), a hero speed out of range speed 2's frames.
    let mut l = looped(Bugs::Fixed, 7);
    l.run(1, &[Action::Scroll(Direction::South)]).unwrap();
    assert_eq!(l.state(1).view, ViewAt { x: 9, y: 12, z: 0 });
    for speed in [0, 5] {
        let mut l = walking(speed, Vec::new());
        l.run(1, &[Action::Step(Direction::East)]).unwrap();
        // The first of speed 2's frames: a tile back less its 10 pixels.
        assert_eq!(l.state(1).walk.offset, [22, 0], "{speed}");
    }
}

/// A loop over a 36-tile map, its selected hero at (17, 17) facing east and the view centred on
/// it, the pointer parked, at a hero speed with these paths.
fn walking(speed: usize, paths: Vec<Vec<Arrow>>) -> Loop {
    let start = Start {
        view: ViewAt { x: 8, y: 9, z: 0 },
        size: 36,
        scroll_speed: 1,
        hero_speed: speed,
        pointer: PARK,
        hero: Some(Hero {
            at: Pos { x: 17, y: 17, z: 0 },
            facing: Direction::East,
        }),
        paths,
        path: Vec::new(),
        live: false,
        sound: None,
        kingdom: Kingdom::default(),
        buttons: None,
    };
    Loop::start(rules(Bugs::Fixed), &start).unwrap()
}

fn arrow(x: u8, y: u8, arrow: u16) -> Arrow {
    Arrow {
        at: Pos { x, y, z: 0 },
        arrow,
    }
}

/// The middle of tile (x, y) while the view's corner is (8, 9).
fn tile(x: i32, y: i32) -> [i32; 2] {
    [(x - 8) * 32 + 16, (y - 9) * 32 + 16]
}

/// Four tiles east of the hero, all green, the last the green cross.
fn east() -> Vec<Arrow> {
    vec![
        arrow(21, 17, 1),
        arrow(18, 17, 12),
        arrow(20, 17, 12),
        arrow(19, 17, 12),
    ]
}

#[test]
fn a_step_needs_a_hero_and_the_map() {
    let mut l = looped(Bugs::Fixed, 1);
    let before = l.state(0);
    l.run(1, &[Action::Step(Direction::North)]).unwrap();
    assert_eq!((l.state(1).hero, l.state(1).drawn), (None, before.drawn));
    for (x, y, direction) in [
        (0, 5, Direction::West),
        (5, 0, Direction::North),
        (35, 5, Direction::East),
    ] {
        let mut l = walking(2, Vec::new());
        l.hero = Some(Hero {
            at: Pos { x, y, z: 0 },
            facing: Direction::East,
        });
        l.run(1, &[Action::Step(direction)]).unwrap();
        let s = l.state(1);
        assert_eq!(s.hero.unwrap().at, Pos { x, y, z: 0 }, "{direction:?}");
        assert!(s.cursor.shown && s.drawn.lifted.is_none());
    }
}

#[test]
fn a_step_and_its_frames() {
    // Speed 1: four frames 50 ms apart sliding the view by 8 pixels, the cursor hidden.
    let mut l = walking(1, Vec::new());
    l.run(1, &[Action::Step(Direction::NorthEast)]).unwrap();
    let s = l.state(1);
    assert_eq!(
        (s.view, s.hero.unwrap().at, s.walk.offset, s.walk.frame),
        (
            ViewAt { x: 9, y: 8, z: 0 },
            Pos { x: 18, y: 16, z: 0 },
            [24, -24],
            1
        )
    );
    assert_eq!(s.drawn.lifted, Some([6, 0]));
    assert!(!s.cursor.shown);
    for t in 2..=200 {
        l.run(t, &[]).unwrap();
    }
    assert_eq!(l.state(200).walk.offset, [0, 0]);
    assert_eq!(l.state(200).walk.group, 6);
    // At 201 the step's end: the standing group by the facing, the cursor shown, the redraw due
    // since 180 drawing the hero standing.
    l.run(201, &[]).unwrap();
    let s = l.state(201);
    assert_eq!((s.walk.group, s.walk.frame), (1, 0));
    assert!(s.cursor.shown);
    assert_eq!(s.drawn.lifted, Some([1, 0]));
    assert_eq!(s.objects, 1);
}

#[test]
fn a_walks_sounds() {
    // The walk east at speed 3 (two frames a step), over grass but dirt from x 19, LoopAren at
    // (16, 17), a tile west of the hero.
    let tiles = crate::sound::tests::tiles(
        36,
        |x, _| if x >= 19 { 0 } else { 2 },
        |x, y| ((x, y) == (16, 17)).then_some(2),
    );
    let mut l = walking(3, vec![east()]);
    l.sound = Some(crate::sound::tests::sound(tiles));
    let (middle, standing) = (l.middle(), l.standing());
    if let Some(sound) = &mut l.sound {
        sound.start(0, middle, standing);
    }
    let dest = tile(21, 17);
    l.run(1, &[Action::Click(dest)]).unwrap();
    l.run(2, &[Action::Click(dest)]).unwrap();
    for t in 3..=402 {
        l.run(t, &[]).unwrap();
    }
    let cues: Vec<String> = l
        .take_cues()
        .iter()
        .map(|(t, c)| crate::sound::line(*t, c))
        .collect();
    // The map's start; the walk's start (its sound, then the lift restarting the loops); each
    // step's end the loop's volume by its distance, gone at 4; onto dirt (a step's end at frame
    // 4) the music at once and the step sound at the frame's wrap (352); the last step's end.
    assert_eq!(
        cues,
        [
            "0 music Grass",
            "0 loop 0 LoopAren.wav 28",
            "0 loop-volume 0 28",
            "2 step horse02.wav",
            "2 loop-stop 0",
            "2 loop 0 LoopAren.wav 28",
            "2 loop-volume 0 28",
            "102 loop-volume 0 20",
            "202 music Dirt",
            "202 loop-volume 0 10",
            "302 loop-stop 0",
            "352 step-stop",
            "352 step horse00.wav",
            "402 step-stop",
        ]
    );
    assert!(l.take_cues().is_empty());
    // Key steps west after a scroll (the lift centres the view on the hero's dirt: no change):
    // no step sound, but onto grass the new terrain's, started and stopped at the step's end
    // (after the ambient sounds); LoopAren back within 3 tiles.
    l.run(403, &[Action::Scroll(Direction::West)]).unwrap();
    for t in [404, 505, 606] {
        l.run(t, &[Action::Step(Direction::West)]).unwrap();
        for t in t + 1..t + 101 {
            l.run(t, &[]).unwrap();
        }
    }
    let cues: Vec<String> = l
        .take_cues()
        .iter()
        .map(|(t, c)| crate::sound::line(*t, c))
        .collect();
    assert_eq!(l.state(706).hero.unwrap().at, Pos { x: 18, y: 17, z: 0 });
    assert_eq!(
        cues,
        [
            "605 loop 0 LoopAren.wav 10",
            "605 loop-volume 0 10",
            "706 music Grass",
            "706 step horse02.wav",
            "706 loop-volume 0 20",
            "706 step-stop"
        ]
    );
}

#[test]
fn a_path_shown_then_walked() {
    let mut l = walking(3, vec![east()]);
    let dest = tile(21, 17);
    l.run(1, &[Action::Click(dest)]).unwrap();
    let s = l.state(1);
    assert_eq!(s.arrows.len(), 4);
    assert_eq!(s.drawn.path, s.arrows);
    // Memory's order: by row, then column.
    assert_eq!(s.arrows[0].at.x, 18);
    l.run(2, &[Action::Click(dest)]).unwrap();
    assert_eq!(l.state(2).arrows.len(), 3);
    for t in 3..=402 {
        l.run(t, &[]).unwrap();
    }
    let s = l.state(402);
    assert_eq!(s.hero.unwrap().at, Pos { x: 21, y: 17, z: 0 });
    assert!(s.arrows.is_empty() && s.cursor.shown);
    // The frame carried from step to step: eight frames drawn, 0 at the end.
    assert_eq!((s.walk.group, s.walk.frame), (2, 0));
}

#[test]
fn a_click_a_key_or_a_scroll_stops_the_paths_walk_after_its_step() {
    for stop in [
        Action::Click(tile(21, 17)),
        Action::Step(Direction::South),
        Action::Scroll(Direction::South),
    ] {
        let mut l = walking(2, vec![east()]);
        l.run(1, &[Action::Click(tile(21, 17))]).unwrap();
        l.run(2, &[Action::Click(tile(21, 17))]).unwrap();
        l.run(3, &[Action::Pointer([700, 400]), stop]).unwrap();
        for t in 4..152 {
            l.run(t, &[]).unwrap();
        }
        assert!(!l.state(151).cursor.shown);
        l.run(152, &[]).unwrap();
        // Ended after its first step (0x407FA7; a scroll's key-down too, M2 T6a, the reviewer's
        // R1-1): the pointer's message before the stop dropped, the stop's own taken (no scroll,
        // no step); the hero standing (`hero_walk_end` 0x408002) when the rest of the path is
        // shown again and drawn (R1-4); the cursor where the mouse is now.
        let s = l.state(152);
        assert_eq!(s.hero.unwrap().at, Pos { x: 18, y: 17, z: 0 }, "{stop:?}");
        assert_eq!((s.arrows.len(), s.drawn.path.len()), (3, 3));
        assert_eq!((s.walk.group, s.view.x, s.view.y), (2, 9, 9));
        assert_eq!(s.drawn.lifted, Some([2, 0]), "{stop:?}");
        let now = if let Action::Click(at) = stop {
            at
        } else {
            [700, 400]
        };
        assert_eq!(s.cursor.at, now, "{stop:?}");
        assert!(s.cursor.shown);
    }
}

#[test]
fn the_messages_behind_the_one_that_stops_a_paths_walk_wait_for_its_end() {
    // Two keys during the first step at speed 2 (M2 T6a, the reviewer's R1-2): the first stops
    // the walk after that step, the second, left in the queue, steps the hero south after it.
    let mut l = walking(2, vec![east()]);
    l.run(1, &[Action::Click(tile(21, 17))]).unwrap();
    l.run(2, &[Action::Click(tile(21, 17))]).unwrap();
    l.run(3, &[Action::Step(Direction::South)]).unwrap();
    l.run(4, &[Action::Step(Direction::South)]).unwrap();
    for t in 5..=152 {
        l.run(t, &[]).unwrap();
    }
    let s = l.state(152);
    assert_eq!(s.hero.unwrap().at, Pos { x: 18, y: 18, z: 0 });
    assert!(s.arrows.is_empty() && !s.cursor.shown);
    for t in 153..=302 {
        l.run(t, &[]).unwrap();
    }
    let s = l.state(302);
    assert_eq!(s.hero.unwrap().at, Pos { x: 18, y: 18, z: 0 });
    assert!(s.cursor.shown);
}

#[test]
fn a_paths_walk_empties_the_input_queue_at_its_start() {
    // During a key's step north (1 to 151) a click showing the path, one on its destination and a
    // key: at the step's end the second click walks the path (the view then (8, 8)) and empties
    // the queue (0x407E87; M2 T6a, the reviewer's R1-5): the key is gone, the path walked whole.
    let mut l = walking(2, vec![east()]);
    let dest = [(21 - 8) * 32 + 16, (17 - 8) * 32 + 16];
    l.run(1, &[Action::Step(Direction::North)]).unwrap();
    l.run(2, &[Action::Click(dest), Action::Click(dest)])
        .unwrap();
    l.run(3, &[Action::Step(Direction::South)]).unwrap();
    for t in 4..=1000 {
        l.run(t, &[]).unwrap();
    }
    let s = l.state(1000);
    assert_eq!(
        s.hero.unwrap(),
        Hero {
            at: Pos { x: 21, y: 17, z: 0 },
            facing: Direction::East
        }
    );
    assert!(s.arrows.is_empty() && s.cursor.shown);
}

#[test]
fn a_message_taken_after_a_walk_reads_the_mouse_now() {
    // The pointer through the east zone and back over the panel during a key's step (M2 T6a, the
    // reviewer's R1-3): at the step's end the edge scroll reads the mouse's point now, the
    // panel's, and scrolls nothing.
    let mut l = walking(2, Vec::new());
    l.run(1, &[Action::Step(Direction::North)]).unwrap();
    l.run(2, &[Action::Pointer([790, 300])]).unwrap();
    l.run(3, &[Action::Pointer(PARK)]).unwrap();
    for t in 4..=151 {
        l.run(t, &[]).unwrap();
    }
    let s = l.state(151);
    assert_eq!(
        (s.hero.unwrap().at, s.view),
        (Pos { x: 17, y: 16, z: 0 }, ViewAt { x: 8, y: 8, z: 0 })
    );
    assert!(s.cursor.shown);
}

#[test]
fn a_red_first_arrow_walks_nowhere() {
    let red = vec![arrow(18, 17, 37), arrow(19, 17, 26)];
    let mut l = walking(2, vec![red]);
    l.run(1, &[Action::Click(tile(19, 17))]).unwrap();
    l.run(2, &[Action::Click(tile(19, 17))]).unwrap();
    let s = l.state(2);
    assert_eq!(s.hero.unwrap().at, Pos { x: 17, y: 17, z: 0 });
    assert_eq!((s.arrows.len(), s.drawn.lifted), (2, Some([2, 0])));
    assert!(s.cursor.shown);
}

#[test]
fn actions_during_a_keys_step_wait_for_its_end() {
    // A click showing a path and a key stepping again, given during a step at speed 1 (1 to 201):
    // at its end the click shows the path from the new tile (its point read with the view as it
    // stands then: (8, 8)), the key clears it (the objects stepped once, the redraw due since
    // 180) and steps again.
    let up = vec![arrow(18, 16, 12), arrow(19, 16, 1)];
    let mut l = walking(1, vec![up]);
    l.run(1, &[Action::Step(Direction::North)]).unwrap();
    l.run(
        2,
        &[Action::Click([(19 - 8) * 32 + 16, (16 - 8) * 32 + 16])],
    )
    .unwrap();
    l.run(3, &[Action::Step(Direction::East), Action::Pointer(PARK)])
        .unwrap();
    for t in 4..=201 {
        l.run(t, &[]).unwrap();
    }
    let s = l.state(201);
    assert_eq!(s.hero.unwrap().at, Pos { x: 18, y: 16, z: 0 });
    assert!(s.arrows.is_empty() && !s.cursor.shown);
    assert_eq!((s.objects, s.drawn.frames), (1, 2));
    for t in 202..=401 {
        l.run(t, &[]).unwrap();
    }
    assert!(l.state(401).cursor.shown);
    // A key's step on a path shown before the redraw's time: cleared, no objects' step.
    let mut l = walking(2, vec![east()]);
    l.run(1, &[Action::Click(tile(21, 17))]).unwrap();
    l.run(2, &[Action::Step(Direction::South)]).unwrap();
    let s = l.state(2);
    assert!(s.arrows.is_empty());
    assert_eq!(s.objects, 0);
    // A pointer over the view given during a step: unplayable at its end.
    let mut l = walking(2, Vec::new());
    l.run(1, &[Action::Step(Direction::North)]).unwrap();
    l.run(2, &[Action::Pointer([300, 300])]).unwrap();
    for t in 3..151 {
        l.run(t, &[]).unwrap();
    }
    assert_eq!(l.run(151, &[]), Err(Unplayable::OverView([300, 300])));
}

#[test]
fn clicks_it_cannot_play() {
    let mut l = walking(2, vec![east()]);
    assert_eq!(
        l.run(1, &[Action::Click([700, 300])]),
        Err(Unplayable::Panel([700, 300]))
    );
    assert_eq!(
        l.run(2, &[Action::Click(tile(21, 19))]),
        Err(Unplayable::NoPath([21, 19, 0]))
    );
    let mut l = looped(Bugs::Fixed, 1);
    assert_eq!(
        l.run(1, &[Action::Click([16, 16])]),
        Err(Unplayable::NoPath([9, 10, 0]))
    );
}

#[test]
fn a_paths_order() {
    let from = Pos { x: 0, y: 0, z: 0 };
    let path = chain(from, &[arrow(2, 1, 1), arrow(1, 0, 12)]).unwrap();
    assert_eq!(
        path,
        [
            (Direction::East, arrow(1, 0, 12)),
            (Direction::SouthEast, arrow(2, 1, 1))
        ]
    );
    // Two neighbours, none, or one on another level: no order.
    assert_eq!(chain(from, &[arrow(1, 0, 12), arrow(0, 1, 12)]), None);
    assert_eq!(chain(from, &[arrow(5, 5, 1)]), None);
    let below = Arrow {
        at: Pos { x: 1, y: 0, z: 1 },
        arrow: 1,
    };
    assert_eq!(chain(from, &[below]), None);
    assert_eq!(Direction::between(from, Pos { x: 2, y: 0, z: 0 }), None);
}

/// A live start (the window's, E24 M2 T4b): `walking`'s map, hero and paths, the pointer at a
/// point, a path shown at 0.
fn live(pointer: [i32; 2], hero: bool, path: Vec<Arrow>) -> Start {
    Start {
        view: ViewAt { x: 8, y: 9, z: 0 },
        size: 36,
        scroll_speed: 1,
        hero_speed: 2,
        pointer,
        hero: hero.then_some(Hero {
            at: Pos { x: 17, y: 17, z: 0 },
            facing: Direction::East,
        }),
        paths: vec![east()],
        path,
        live: true,
        sound: None,
        kingdom: Kingdom::default(),
        buttons: None,
    }
}

#[test]
fn live_the_pointer_over_the_view_shows_the_arrow() {
    let r = rules(Bugs::Fixed);
    let mut l = Loop::start(r.clone(), &live([300, 300], false, Vec::new())).unwrap();
    assert_eq!(l.state(0).cursor.frame, r.arrow);
    // Over the view in the west zone: the edge scroll, its cursor, 70 ms after the idle message
    // before the move.
    l.run(1, &[Action::Pointer([10, 300])]).unwrap();
    for t in 2..71 {
        l.run(t, &[]).unwrap();
    }
    assert_eq!(l.state(70).view.x, 8);
    l.run(71, &[]).unwrap();
    let s = l.state(71);
    assert_eq!((s.view.x, s.cursor.frame), (6, r.edges + 6));
    // Out of the zone over the view: the arrow, no scroll.
    l.run(72, &[Action::Pointer([300, 300])]).unwrap();
    for t in 73..200 {
        l.run(t, &[]).unwrap();
    }
    let s = l.state(199);
    assert_eq!((s.view.x, s.cursor.frame), (6, r.arrow));
    // Not live, the same start is refused.
    let mut strict = live([300, 300], false, Vec::new());
    strict.live = false;
    assert_eq!(
        Loop::start(r, &strict),
        Err(Unplayable::OverView([300, 300]))
    );
}

#[test]
fn live_a_click_the_loop_cannot_play_only_moves_the_pointer() {
    for hero in [true, false] {
        let mut l = Loop::start(rules(Bugs::Fixed), &live(PARK, hero, Vec::new())).unwrap();
        let before = l.state(0).drawn;
        l.run(1, &[Action::Click([700, 2])]).unwrap();
        l.run(2, &[Action::Click(tile(21, 19))]).unwrap();
        let s = l.state(2);
        assert_eq!(s.drawn, before, "{hero}");
        assert!(s.arrows.is_empty() && s.cursor.shown, "{hero}");
        // The path's destination: shown with a hero, nothing without.
        l.run(3, &[Action::Click(tile(21, 17))]).unwrap();
        assert_eq!(l.state(3).arrows.len(), if hero { 4 } else { 0 }, "{hero}");
    }
}

#[test]
fn a_path_shown_at_0() {
    let r = rules(Bugs::Fixed);
    let mut l = Loop::start(r.clone(), &live(PARK, true, east())).unwrap();
    let s = l.state(0);
    assert_eq!((s.arrows.len(), s.drawn.path.len()), (4, 4));
    assert_eq!(s.drawn.path, s.arrows);
    // Its destination clicked: the walk at once.
    l.run(1, &[Action::Click(tile(21, 17))]).unwrap();
    let s = l.state(1);
    assert!(!s.cursor.shown);
    assert_eq!(s.hero.unwrap().at, Pos { x: 18, y: 17, z: 0 });
    // A path that does not lead from the hero, or no hero: none shown.
    let mut away = live(PARK, true, vec![arrow(5, 5, 1)]);
    assert!(
        Loop::start(r.clone(), &away)
            .unwrap()
            .state(0)
            .arrows
            .is_empty()
    );
    away.hero = None;
    away.path = east();
    assert!(Loop::start(r, &away).unwrap().state(0).arrows.is_empty());
}

#[test]
fn move_hero_and_next_hero_through_a_walk() {
    // E24 M3 T2a (design §4.1), as the recordings show them: a path whose last two steps the
    // points do not pay, and one all green.
    let short = vec![
        arrow(18, 17, 12),
        arrow(19, 17, 12),
        arrow(20, 17, 37),
        arrow(21, 17, 26),
    ];
    let awake = KingdomHero {
        asleep: false,
        movable: true,
    };
    let start = |heroes: Vec<KingdomHero>, path: &Vec<Arrow>| {
        let mut s = Start {
            kingdom: Kingdom {
                heroes,
                selected: Selected::Hero(0),
                ..Kingdom::default()
            },
            ..live(PARK, true, Vec::new())
        };
        s.live = false;
        s.paths = vec![path.clone()];
        s.hero_speed = 4;
        Loop::start(rules(Bugs::Fixed), &s).unwrap()
    };
    let buttons = |move_hero, next_hero| Buttons {
        move_hero,
        next_hero,
    };
    let walk = |l: &mut Loop| {
        l.run(1, &[Action::Click(tile(21, 17))]).unwrap();
        assert!(l.state(1).buttons.move_hero);
        l.run(2, &[Action::Click(tile(21, 17))]).unwrap();
        for t in 3..=400 {
            l.run(t, &[]).unwrap();
        }
    };
    let mut l = start(vec![awake], &short);
    assert_eq!(l.state(0).buttons, buttons(false, true));
    // The walk ends on the red arrows: the hero cannot move on, next hero off with no other hero.
    walk(&mut l);
    let s = l.state(400);
    assert_eq!(s.hero.unwrap().at, Pos { x: 19, y: 17, z: 0 });
    assert_eq!(s.buttons, buttons(false, false));
    // Another hero awake and able to move keeps next hero on.
    let mut l = start(vec![awake, awake], &short);
    walk(&mut l);
    assert_eq!(l.state(400).buttons, buttons(false, true));
    // On its destination the path is gone: move hero off.
    let mut l = start(vec![awake], &east());
    walk(&mut l);
    assert_eq!(l.state(400).buttons, buttons(false, true));
    // A key's step: move hero on when it lifts the hero, off once the hero is lifted.
    let mut l = start(vec![awake], &short);
    l.run(1, &[Action::Step(Direction::North)]).unwrap();
    assert!(l.state(1).buttons.move_hero);
    for t in 2..=300 {
        l.run(t, &[]).unwrap();
    }
    l.run(301, &[Action::Step(Direction::North)]).unwrap();
    assert!(!l.state(301).buttons.move_hero);
    // The start's buttons as given.
    let mut given = live(PARK, true, Vec::new());
    given.buttons = Some(buttons(true, false));
    let l = Loop::start(rules(Bugs::Fixed), &given).unwrap();
    assert_eq!(l.state(0).buttons, buttons(true, false));
}

#[test]
fn the_minimap_drawn_again_by_a_scroll_a_lift_and_each_steps_end() {
    // E24 M3 T3b (design §4.4): at 0 the still's; a key's step lifting the hero draws it with the
    // view centred on the hero's tile before the step; the step's frames leave it, its end draws
    // it again (the guard's test); a scroll draws it at the new corner, the hero put back.
    let minimap = |x: i32, y: i32, hx: u8, hy: u8| Minimap {
        view: ViewAt { x, y, z: 0 },
        hero: Some(Pos { x: hx, y: hy, z: 0 }),
    };
    let run = |l: &mut Loop, from: u32, to: u32| {
        for t in from..=to {
            l.run(t, &[]).unwrap();
        }
    };
    let mut l = walking(1, vec![east()]);
    assert_eq!(l.state(0).drawn.minimap, minimap(8, 9, 17, 17));
    l.run(1, &[Action::Step(Direction::NorthEast)]).unwrap();
    let s = l.state(1);
    assert_eq!(s.view, ViewAt { x: 9, y: 8, z: 0 });
    assert_eq!(s.drawn.minimap, minimap(8, 9, 17, 17));
    run(&mut l, 2, 200);
    assert_eq!(l.state(200).drawn.minimap, minimap(8, 9, 17, 17));
    // The step's end (its frames at 1, 51, 101 and 151): the new corner and tile.
    l.run(201, &[]).unwrap();
    assert_eq!(l.state(201).drawn.minimap, minimap(9, 8, 18, 16));
    run(&mut l, 202, 300);
    // A second step: the hero lifted already, the minimap as it was until the step's end.
    l.run(301, &[Action::Step(Direction::East)]).unwrap();
    assert_eq!(l.state(301).drawn.minimap, minimap(9, 8, 18, 16));
    run(&mut l, 302, 600);
    assert_eq!(l.state(600).hero.unwrap().at, Pos { x: 19, y: 16, z: 0 });
    assert_eq!(l.state(600).drawn.minimap, minimap(10, 8, 19, 16));
    // A scroll: the new corner, the hero on its tile.
    l.run(601, &[Action::Scroll(Direction::South)]).unwrap();
    let s = l.state(601);
    assert_eq!(s.drawn.minimap, minimap(s.view.x, s.view.y, 19, 16));
    assert_ne!(s.view, ViewAt { x: 10, y: 8, z: 0 });
    // A scroll that cannot move draws nothing.
    let before = l.state(601).drawn;
    l.view.y = 27;
    l.run(602, &[Action::Scroll(Direction::South)]).unwrap();
    assert_eq!(l.state(602).drawn, before);
    // The next step lifts the hero again: the view centred on its tile, the minimap drawn there
    // before the step moves the hero on.
    l.run(800, &[Action::Step(Direction::West)]).unwrap();
    let s = l.state(800);
    assert_eq!(s.hero.unwrap().at, Pos { x: 18, y: 16, z: 0 });
    assert_eq!(s.drawn.minimap, minimap(10, 8, 19, 16));
    // A path's walk lifts the hero without drawing it; each step's end draws it (speed 4: one
    // draw a step, the walk over at once).
    let mut l = walking(4, vec![east()]);
    l.run(1, &[Action::Click(tile(21, 17))]).unwrap();
    assert_eq!(l.state(1).drawn.minimap, minimap(8, 9, 17, 17));
    l.run(2, &[Action::Click(tile(21, 17))]).unwrap();
    let s = l.state(2);
    assert_eq!(s.hero.unwrap().at, Pos { x: 21, y: 17, z: 0 });
    assert_eq!(s.drawn.minimap, minimap(12, 9, 21, 17));
}
