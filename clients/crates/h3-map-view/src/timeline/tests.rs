use super::*;

use alloc::string::ToString;
use alloc::vec;

use crate::layout::AdventureLayout;
use crate::live::{
    Chosen, DateStatus, Facing, Kingdom, KingdomHero, KingdomStatus, LiveHero, Status,
};
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
const RECORDED: [(&str, &str, &str); 15] = [
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
    (
        "status-kingdom-end",
        include_str!("../../../../map/situations/status-kingdom.toml"),
        include_str!("../../../../map/timelines/status-kingdom-end.txt"),
    ),
    (
        "status-date-end",
        include_str!("../../../../map/situations/status-date.toml"),
        include_str!("../../../../map/timelines/status-date-end.txt"),
    ),
];

/// The walks' live parts at 0 (`h3map.py frames`): their selected heroes.
const LIVE: [(&str, &str); 15] = [
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
    (
        "status-kingdom-end",
        include_str!("../../../../map/timelines/status-kingdom-end-live.toml"),
    ),
    (
        "status-date-end",
        include_str!("../../../../map/timelines/status-date-end-live.toml"),
    ),
];

/// A live part's heroes and viewer (`[x, y, level, class, facing, owner, boat, colour]` rows, as
/// the binary's `read_live` takes them).
/// A kept live part's kingdom and buttons (E24 M3 T2a), the status window's and the selection's
/// (T5b1, T5b2).
fn live_kingdom(text: &str) -> (Kingdom, Option<Buttons>) {
    let table: toml::Table = text.parse().unwrap();
    let mut kingdom = Kingdom::read(&table["kingdom"]).unwrap();
    kingdom.status = table.get("status").map(|s| Status::read(s).unwrap());
    kingdom.selection = table.get("selection").map(|s| Status::read(s).unwrap());
    kingdom.realm = table.get("realm").map(|s| match Status::read(s).unwrap() {
        Status::Kingdom(realm) => realm,
        other => panic!("`realm` is {other:?}"),
    });
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
        points: Vec::new(),
        buttons: None,
        asked: None,
        hover: None,
        levels: 1,
        heroes: Vec::new(),
        index: None,
        chosen: crate::live::Chosen::default(),
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
    start.points = sequence.points(timeline).unwrap();
    start.asked = sequence.asked().unwrap();
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
        start.points = sequence.points(recorded).unwrap();
        start.asked = sequence.asked().unwrap();
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
            "walk-sail",
            "hover-map",
            "hover-sail",
            "hover-visits-a",
            "hover-visited-a",
            "hover-visits-b",
            "hover-visited-b",
            "hover-owners",
            "hover-banks",
            "hover-teach",
            "hover-quests",
            "choose-lists",
            "press-buttons",
            "underground",
            "minimap-drag",
            "status-click",
            "walk-queue",
            "walk-stop-key",
            "walk-stop-press",
            "walk-stop-space"
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
        points: Vec::new(),
        buttons: None,
        asked: None,
        hover: None,
        levels: 1,
        heroes: Vec::new(),
        index: None,
        chosen: crate::live::Chosen::default(),
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
        points: Vec::new(),
        buttons: None,
        asked: None,
        hover: None,
        levels: 1,
        heroes: Vec::new(),
        index: None,
        chosen: crate::live::Chosen::default(),
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
    let later = BTreeMap::from([(5, vec![Action::Click([771, 228])])]);
    assert_eq!(
        play(rules(Bugs::Fixed), &parked, &later, 10).map(|s| s.len()),
        Ok(11)
    );
    let over_later = BTreeMap::from([(5, vec![Action::Pointer([300, 300])])]);
    assert_eq!(
        play(rules(Bugs::Fixed), &parked, &over_later, 10),
        Err((5, Unplayable::OverView([300, 300])))
    );
    assert_eq!(
        Unplayable::OverView([1, 2]).to_string(),
        "the pointer over the view at (1, 2) (M3)"
    );
    assert_eq!(
        Unplayable::Panel([700, 2]).to_string(),
        "a click on a panel at (700, 2) not built"
    );
    assert_eq!(
        Unplayable::NoPath([3, 4, 0]).to_string(),
        "no path given from the selected hero to (3, 4, 0)"
    );
    assert_eq!(
        Unplayable::Command(9).to_string(),
        "button 9's command (a dialog or the engine's)"
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
        points: Vec::new(),
        buttons: None,
        asked: None,
        hover: None,
        levels: 1,
        heroes: Vec::new(),
        index: None,
        chosen: crate::live::Chosen::default(),
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

/// E24 M3 T6b3: a button's press, a hotkey's and Space's key downs stop the walk too, taken and
/// not handled: no button pressed, no hero chosen again; a click's up waits behind (its down
/// taken) and does nothing.
#[test]
fn a_press_a_hotkey_or_space_stops_the_paths_walk_too() {
    for stop in [
        Action::Press([700, 400]),
        Action::Key(Hotkey::NextHero),
        Action::Visit,
    ] {
        let mut l = walking(2, vec![east()]);
        l.run(1, &[Action::Click(tile(21, 17))]).unwrap();
        l.run(2, &[Action::Click(tile(21, 17))]).unwrap();
        l.run(3, &[Action::Pointer([700, 400]), stop]).unwrap();
        l.run(4, &[Action::Release([700, 400])]).unwrap();
        for t in 5..=152 {
            l.run(t, &[]).unwrap();
        }
        let s = l.state(152);
        assert_eq!(s.hero.unwrap().at, Pos { x: 18, y: 17, z: 0 }, "{stop:?}");
        assert_eq!(s.arrows.len(), 3, "{stop:?}");
        assert!(s.drawn.pressed.is_empty() && s.cursor.shown, "{stop:?}");
        assert_eq!((s.view.x, s.view.y), (9, 9), "{stop:?}");
    }
    // A click stopping it leaves its up, taken after the walk (nothing held: nothing).
    let mut l = walking(2, vec![east()]);
    l.run(1, &[Action::Click(tile(21, 17))]).unwrap();
    l.run(2, &[Action::Click(tile(21, 17))]).unwrap();
    l.run(3, &[Action::Pointer(PARK), Action::Click(PARK), Action::Up])
        .unwrap();
    for t in 4..=152 {
        l.run(t, &[]).unwrap();
    }
    assert_eq!(l.state(152).hero.unwrap().at, Pos { x: 18, y: 17, z: 0 });
}

/// E24 M3 T6b3, design §4.15: the input ring keeps the last 63 messages, the oldest dropped (a
/// click's down before its up); what counts and what ends a path's walk.
#[test]
fn the_input_ring_keeps_the_last_63_messages() {
    let mut q: Vec<Action> = (0..70)
        .map(|i| {
            if i % 2 == 0 {
                Action::Step(Direction::East)
            } else {
                Action::Up
            }
        })
        .collect();
    keep_ring(&mut q);
    assert_eq!((q.len(), q[0]), (QUEUED, Action::Up));
    let mut q = vec![Action::Click([1, 2])];
    q.extend([Action::Up; 62]);
    keep_ring(&mut q);
    assert_eq!((q.len(), q[0]), (63, Action::Release([1, 2])));
    keep_ring(&mut Vec::new());
    assert_eq!(
        (
            Action::Click([0, 0]).messages(),
            Action::Press([0, 0]).messages()
        ),
        (2, 1)
    );
    for a in [
        Action::Step(Direction::East),
        Action::Scroll(Direction::East),
        Action::Click([0, 0]),
        Action::Press([0, 0]),
        Action::Key(Hotkey::NextHero),
        Action::Visit,
    ] {
        assert!(a.ends_walk(), "{a:?}");
    }
    for a in [
        Action::Pointer([0, 0]),
        Action::Release([0, 0]),
        Action::KeyUp(Hotkey::NextHero),
        Action::Up,
    ] {
        assert!(!a.ends_walk(), "{a:?}");
    }
}

/// A key's step east, then during it 80 key messages (east and west pressed and released by
/// turns): the ring keeps the last 63, whose 31 key downs step after it, west first.
#[test]
fn the_steps_queued_during_a_step_are_the_rings() {
    let mut l = walking(2, Vec::new());
    l.run(1, &[Action::Step(Direction::East)]).unwrap();
    let burst: Vec<Action> = (0..20)
        .flat_map(|_| {
            [
                Action::Step(Direction::East),
                Action::Up,
                Action::Step(Direction::West),
                Action::Up,
            ]
        })
        .collect();
    l.run(2, &burst).unwrap();
    let mut facings = Vec::new();
    for t in 3..=1 + 32 * 150 + 10 {
        l.run(t, &[]).unwrap();
        let hero = l.state(t).hero.unwrap();
        if facings.last() != Some(&(hero.at.x, hero.facing)) && t > 151 {
            facings.push((hero.at.x, hero.facing));
        }
    }
    assert_eq!(facings.len(), 31);
    assert_eq!(facings[0], (17, Direction::West));
    assert_eq!(facings[30], (17, Direction::West));
    assert!(l.state(1 + 32 * 150 + 10).cursor.shown);
}

/// Space (E24 M3 T6b3, design §4.15): a hero standing on its tile (after a scroll) chosen again,
/// the view centred on it and lifted; with a path shown, the path kept (made again from its
/// destination) and move hero by it; a lifted hero, or none selected, nothing.
#[test]
fn space_chooses_a_standing_hero_again() {
    let mut l = walking(2, vec![east()]);
    l.run(1, &[Action::Scroll(Direction::East)]).unwrap();
    assert_eq!(l.state(1).view.x, 10);
    l.run(2, &[Action::Visit, Action::Up]).unwrap();
    let s = l.state(2);
    assert_eq!((s.view.x, s.view.y), (8, 9));
    assert!(s.drawn.lifted.is_some() && !s.buttons.move_hero);
    // Lifted: nothing.
    l.run(3, &[Action::Visit]).unwrap();
    assert_eq!(l.state(3), {
        let mut same = l.state(2);
        same.t = 3;
        same
    });
    // A path shown, the hero put back by a scroll: the path kept, move hero by it.
    l.run(4, &[Action::Click(tile(21, 17))]).unwrap();
    l.run(5, &[Action::Scroll(Direction::West)]).unwrap();
    let s = l.state(5);
    assert!(s.drawn.lifted.is_none() && s.view.x == 6 && s.arrows.len() == 4);
    l.run(6, &[Action::Visit]).unwrap();
    let s = l.state(6);
    assert_eq!((s.view.x, s.arrows.len(), s.drawn.path.len()), (8, 4, 4));
    assert!(s.buttons.move_hero && s.drawn.lifted.is_some());
    // No hero selected.
    let mut l = looped(Bugs::Fixed, 1);
    let before = l.state(0);
    l.run(1, &[Action::Visit]).unwrap();
    assert_eq!(l.state(1).view, before.view);
}

/// Space with the hero on an object's entrance: its visit is the engine's, refused in a
/// recording, nothing live; on a tile the lift leaves plain, nothing.
#[test]
fn space_on_an_entrance_visits_it() {
    let (_, _, hover) = hover::tests::fixture(false);
    let heroes = [[4, 3], [7, 3], [3, 2]];
    let door = (0..8)
        .flat_map(|y| (0..8).map(move |x| [x, y, 0]))
        .find(|&[x, y, z]| {
            !heroes.contains(&[x, y])
                && hover
                    .current([x, y, z], None)
                    .is_some_and(|t| t.entrance && t.class != 3)
        })
        .unwrap();
    let at = Pos {
        x: u8::try_from(door[0]).unwrap(),
        y: u8::try_from(door[1]).unwrap(),
        z: 0,
    };
    for live in [false, true] {
        let mut l = hovering(live, Bugs::Fixed);
        l.run(1, &[Action::Visit]).unwrap();
        l.hero = Some(Hero {
            at,
            facing: Direction::South,
        });
        let got = l.run(2, &[Action::Visit]);
        if live {
            assert_eq!(got, Ok(()));
        } else {
            assert_eq!(got, Err(Unplayable::Visit(door)));
        }
    }
    assert_eq!(
        Unplayable::Visit([1, 2, 0]).to_string(),
        "a visit of the object at (1, 2, 0) (the engine's)"
    );
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
    // The adventure options' button clicked (E24 M3 T6b1): its dialog not built.
    assert_eq!(
        l.run(1, &[Action::Click([700, 300])]),
        Err(Unplayable::Command(9))
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
        points: Vec::new(),
        buttons: None,
        asked: None,
        hover: None,
        levels: 1,
        heroes: Vec::new(),
        index: None,
        chosen: crate::live::Chosen::default(),
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
        // The status bar's hint follows the pointer (E24 M3 T6a1): (700, 2) lies on no item.
        assert_eq!(s.drawn.hint, Hint::Empty, "{hero}");
        let drawn = Draw {
            hint: before.hint.clone(),
            ..s.drawn.clone()
        };
        assert_eq!(drawn, before, "{hero}");
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
        ..KingdomHero::default()
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
        index: None,
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

/// `walking`'s loop with a kingdom of `heroes` heroes (portrait 0, 1 000 points as the list shows
/// them, 10 mana), the last one selected and on the hero's tile, the list's top `top`, and the
/// points memory holds by millisecond (E24 M3 T4b).
fn listed(speed: usize, heroes: usize, top: usize, points: Vec<(u32, i32)>) -> Loop {
    let hero = KingdomHero {
        asleep: false,
        movable: true,
        portrait: 0,
        points: 1000,
        mana: 10,
    };
    let start = Start {
        kingdom: Kingdom {
            heroes: vec![hero; heroes],
            selected: Selected::Hero(heroes - 1),
            tops: [top, 0],
            ..Kingdom::default()
        },
        points,
        live: false,
        hero_speed: speed,
        ..live(PARK, true, Vec::new())
    };
    Loop::start(rules(Bugs::Fixed), &start).unwrap()
}

#[test]
fn the_lists_drawn_again_by_a_lift_each_steps_end_and_a_walk_that_ends_the_points() {
    // E24 M3 T4b (design §4.5): at 0 the list as shown, memory's points the recording's; a key's
    // step lifting the hero draws both lists as memory holds them (the points before the step);
    // the step's end its slot, the points it left shown, the mana bar left as shown; a path's
    // walk whose points end draws both lists.
    let run = |l: &mut Loop, from: u32, to: u32| {
        for t in from..=to {
            l.run(t, &[]).unwrap();
        }
    };
    let shown = |l: &Loop, t: u32| {
        let s = l.state(t);
        let h = s.drawn.kingdom.heroes.last().copied().unwrap();
        (h.points, h.mana, s.points)
    };
    let mut l = listed(1, 1, 0, vec![(0, 2000), (201, 1900), (500, 1759)]);
    assert_eq!(shown(&l, 0), (1000, 10, Some(2000)));
    l.run(1, &[Action::Step(Direction::NorthEast)]).unwrap();
    assert_eq!(shown(&l, 1), (2000, 10, Some(2000)));
    run(&mut l, 2, 200);
    assert_eq!(shown(&l, 200), (2000, 10, Some(2000)));
    // The step's end (its frames at 1, 51, 101 and 151): the points it left, shown.
    l.run(201, &[]).unwrap();
    assert_eq!(shown(&l, 201), (1900, 10, Some(1900)));
    run(&mut l, 202, 300);
    // A second step, the hero lifted: nothing drawn until its end; the mana a spell took (M4) not
    // shown by the slot.
    l.kingdom.heroes[0].mana = 5;
    l.run(301, &[Action::Step(Direction::East)]).unwrap();
    run(&mut l, 302, 500);
    assert_eq!(shown(&l, 500), (1900, 10, Some(1900)));
    l.run(501, &[]).unwrap();
    assert_eq!(shown(&l, 501), (1759, 10, Some(1759)));
    // A scroll puts the hero back; the next step lifts it again: every slot drawn.
    l.run(601, &[Action::Scroll(Direction::South)]).unwrap();
    assert_eq!(shown(&l, 601), (1759, 10, Some(1759)));
    l.run(800, &[Action::Step(Direction::West)]).unwrap();
    assert_eq!(shown(&l, 800), (1759, 5, Some(1759)));
    // The timeline's lines: memory's points at 0 and at each step's end.
    let states = play(
        rules(Bugs::Fixed),
        &Start {
            kingdom: listed(1, 1, 0, Vec::new()).kingdom.clone(),
            points: vec![(0, 2000), (201, 1900)],
            live: false,
            hero_speed: 1,
            ..live(PARK, true, Vec::new())
        },
        &BTreeMap::from([(1, vec![Action::Step(Direction::NorthEast)])]),
        300,
    )
    .unwrap();
    let points: Vec<String> = lines(&states, &BTreeMap::new())
        .into_iter()
        .filter(|l| l.contains(" points "))
        .collect();
    assert_eq!(points, ["0 points 2000", "201 points 1900"]);
    // No hero selected: no points.
    assert_eq!(looped(Bugs::Fixed, 1).state(0).points, None);
}

#[test]
fn a_slot_off_the_list_is_not_drawn_and_a_lift_scrolls_to_it() {
    // E24 M3 T4b: the seventh hero of seven, the list's top 0: its step's end draws no slot; the
    // lift that centres the view scrolls the list to show it (top 2) and draws it.
    let mut l = listed(1, 7, 0, vec![(0, 1000), (201, 900)]);
    l.lifted = true;
    l.run(1, &[Action::Step(Direction::NorthEast)]).unwrap();
    for t in 2..=201 {
        l.run(t, &[]).unwrap();
    }
    let s = l.state(201);
    assert_eq!(s.points, Some(900));
    assert_eq!(
        (s.drawn.kingdom.tops, s.drawn.kingdom.heroes[6].points),
        ([0, 0], 1000)
    );
    l.run(301, &[Action::Scroll(Direction::South)]).unwrap();
    l.run(302, &[Action::Step(Direction::East)]).unwrap();
    let s = l.state(302);
    assert_eq!(
        (s.drawn.kingdom.tops, s.drawn.kingdom.heroes[6].points),
        ([2, 0], 900)
    );
}

#[test]
fn a_walk_that_ends_the_points_draws_both_lists() {
    // E24 M3 T4b (0x4081B9): the path's walk ends on its red arrows, the hero unable to move:
    // every slot drawn as memory holds it, the hero's points 0.
    let short = vec![
        arrow(18, 17, 12),
        arrow(19, 17, 12),
        arrow(20, 17, 37),
        arrow(21, 17, 26),
    ];
    let mut l = listed(4, 1, 0, vec![(0, 450), (2, 0)]);
    l.paths = vec![short];
    l.run(1, &[Action::Click(tile(21, 17))]).unwrap();
    l.run(2, &[Action::Click(tile(21, 17))]).unwrap();
    let s = l.state(2);
    assert_eq!(s.hero.unwrap().at, Pos { x: 19, y: 17, z: 0 });
    let drawn = s.drawn.kingdom.heroes[0];
    assert_eq!((drawn.points, drawn.movable), (0, false));
    assert!(!s.buttons.next_hero);
}

/// The status window drawn at each millisecond of a play.
fn statuses(states: &[State]) -> Vec<Option<Status>> {
    states
        .iter()
        .map(|s| s.drawn.kingdom.status.clone())
        .collect()
}

fn date_frame(status: Option<&Status>) -> Option<u32> {
    match status {
        Some(Status::Date(d)) => Some(d.frame),
        _ => None,
    }
}

#[test]
fn the_status_window_over_time() {
    // The date's asked at 0 for 3 000 ms (E24 M3 T5b2, design §4.9): its frame stepped at each
    // redraw (180 ms apart, at least 100 after the last) to its picture's last (the day's
    // tenth), then the selected hero's window at the redraw at 3 060.
    let (start, actions, ms) = recorded("status-date-end");
    let states = play(rules(Bugs::Original), &start, &actions, ms).unwrap();
    let shown = statuses(&states);
    let frame = |t: usize| date_frame(shown[t].as_ref());
    for (t, want) in [(0, 0), (179, 0), (180, 1), (359, 1), (360, 2), (1620, 9)] {
        assert_eq!(frame(t), Some(want), "{t}");
    }
    assert_eq!(frame(3059), Some(9));
    assert!(matches!(shown[3060], Some(Status::Hero(_))));
    assert_eq!(shown[3060], start.kingdom.selection);
    // The kingdom's asked for 700 ms: the hero's at the redraw at 720.
    let (start, actions, ms) = recorded("status-kingdom-end");
    let shown = statuses(&play(rules(Bugs::Original), &start, &actions, ms).unwrap());
    assert!(matches!(shown[719], Some(Status::Kingdom(_))));
    assert_eq!(shown[720], start.kingdom.selection);
    // Without a selection's window known the kind asked stays at its end; the kingdom's asked
    // without the live part's kingdom shows what it shows.
    let mut unknown = start.clone();
    unknown.kingdom.selection = None;
    let shown = statuses(&play(rules(Bugs::Original), &unknown, &actions, ms).unwrap());
    assert!(matches!(shown[900], Some(Status::Kingdom(_))));
    let (date, _, _) = recorded("status-date-end");
    let realmless = Start {
        asked: Some(Asked { kind: 2, end: 500 }),
        ..date.clone()
    };
    let shown = statuses(&play(rules(Bugs::Original), &realmless, &actions, 400).unwrap());
    assert_eq!(date_frame(shown[0].as_ref()), Some(0));
    assert_eq!(date_frame(shown[400].as_ref()), Some(0));
    // Nothing selected: the end gives the date's window again, built afresh at its first frame.
    let none = Start {
        kingdom: Kingdom {
            selected: Selected::None,
            status: Some(Status::Kingdom(KingdomStatus {
                halls: [1, 0, 0, 0],
                allies: vec![0],
                enemies: vec![1],
            })),
            ..date.kingdom.clone()
        },
        asked: Some(Asked { kind: 2, end: 300 }),
        ..date.clone()
    };
    let shown = statuses(&play(rules(Bugs::Original), &none, &actions, 600).unwrap());
    assert!(matches!(shown[359], Some(Status::Kingdom(_))));
    assert_eq!(date_frame(shown[360].as_ref()), Some(0));
    assert_eq!(date_frame(shown[540].as_ref()), Some(1));
    // A town selected: its window (a hero's window as the selection's with a town selected is
    // left as it is).
    let town = crate::live::TownStatus {
        name: 3,
        buildings: 0,
        garrison: false,
        army: [None; 7],
        silo: Vec::new(),
        income: 500,
    };
    let towned = Start {
        kingdom: Kingdom {
            selected: Selected::Town(0),
            selection: Some(Status::Town(town.clone())),
            ..none.kingdom.clone()
        },
        ..none.clone()
    };
    let shown = statuses(&play(rules(Bugs::Original), &towned, &actions, 400).unwrap());
    assert_eq!(shown[360], Some(Status::Town(town)));
    let mismatched = Start {
        kingdom: Kingdom {
            selected: Selected::Town(0),
            selection: date.kingdom.selection.clone(),
            ..none.kingdom.clone()
        },
        ..none.clone()
    };
    let shown = statuses(&play(rules(Bugs::Original), &mismatched, &actions, 400).unwrap());
    assert!(matches!(shown[400], Some(Status::Kingdom(_))));
    // A still's date's window (no kind asked): asked until no end, its frame stepping on.
    let still = Start {
        asked: None,
        kingdom: Kingdom {
            status: Some(Status::Date(DateStatus { frame: 3 })),
            ..date.kingdom.clone()
        },
        ..date.clone()
    };
    let shown = statuses(&play(rules(Bugs::Original), &still, &actions, 5000).unwrap());
    assert_eq!(date_frame(shown[0].as_ref()), Some(3));
    assert_eq!(date_frame(shown[180].as_ref()), Some(4));
    assert_eq!(date_frame(shown[5000].as_ref()), Some(9));
}

#[test]
fn the_status_window_built_again_by_a_selection_and_a_key_s_step() {
    // The date's window over a walking hero (E24 M3 T5b2): a key's step lifts the hero (its
    // selection centring the view, 0x41800B) and ends (0x4090FE): each builds the date's again
    // at its first frame.
    let (date, _, _) = recorded("status-date-end");
    let (walk, _, _) = recorded("walk-1");
    let start = Start {
        kingdom: Kingdom {
            status: Some(Status::Date(DateStatus { frame: 0 })),
            selection: walk.kingdom.status.clone(),
            date: date.kingdom.date,
            ..walk.kingdom.clone()
        },
        asked: Some(Asked {
            kind: 1,
            end: 10_000,
        }),
        ..walk.clone()
    };
    let actions = BTreeMap::from([(400, vec![Action::Step(Direction::East)])]);
    let states = play(rules(Bugs::Original), &start, &actions, 1500).unwrap();
    let shown = statuses(&states);
    assert_eq!(date_frame(shown[360].as_ref()), Some(2));
    // The step at 400: built again at once, then stepping from there.
    assert_eq!(date_frame(shown[400].as_ref()), Some(0));
    let restarts = (401..=1500)
        .filter(|&t| {
            date_frame(shown[t].as_ref()) == Some(0) && date_frame(shown[t - 1].as_ref()) != Some(0)
        })
        .count();
    assert_eq!(restarts, 1, "the step's end builds it again");
}

#[test]
fn the_day_s_window_plays_its_sound_when_built() {
    // A day's window built at 0 and again: its sound each time; a week's none (E24 M3 T5b2).
    let tiles = crate::sound::tests::tiles(36, |_, _| 2, |_, _| None);
    let (date, actions, _) = recorded("status-date-end");
    let start = Start {
        sound: Some(crate::sound::tests::sound(tiles.clone())),
        ..date.clone()
    };
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    let effects = |l: &mut Loop| -> Vec<(u32, Cue)> {
        l.take_cues()
            .into_iter()
            .filter(|(_, c)| matches!(c, Cue::Effect(_)))
            .collect()
    };
    assert_eq!(effects(&mut l), [(0, Cue::Effect("newday.wav".into()))]);
    for t in 1..=600 {
        l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
            .unwrap();
    }
    assert!(effects(&mut l).is_empty());
    let week = Start {
        kingdom: Kingdom {
            date: crate::live::Date {
                month: 2,
                week: 3,
                day: 1,
            },
            ..start.kingdom.clone()
        },
        ..start.clone()
    };
    let mut l = Loop::start(rules(Bugs::Original), &week).unwrap();
    assert!(effects(&mut l).is_empty());
    // A still's window at a later frame was built before: none.
    let later = Start {
        kingdom: Kingdom {
            status: Some(Status::Date(DateStatus { frame: 2 })),
            ..start.kingdom.clone()
        },
        ..start.clone()
    };
    let mut l = Loop::start(rules(Bugs::Original), &later).unwrap();
    assert!(effects(&mut l).is_empty());
    // Nothing selected and a kind asked ending: the date's built again, with its sound.
    let none = Start {
        kingdom: Kingdom {
            selected: Selected::None,
            status: Some(Status::Kingdom(KingdomStatus {
                halls: [0; 4],
                allies: vec![0],
                enemies: Vec::new(),
            })),
            ..start.kingdom.clone()
        },
        asked: Some(Asked { kind: 2, end: 100 }),
        ..start.clone()
    };
    let mut l = Loop::start(rules(Bugs::Original), &none).unwrap();
    for t in 1..=200 {
        l.run(t, &[]).unwrap();
    }
    assert_eq!(effects(&mut l), [(180, Cue::Effect("newday.wav".into()))]);
}

/// The middle of the tile (x, y) of a view from (0, 0).
fn on(x: i32, y: i32) -> [i32; 2] {
    [x * 32 + 16, y * 32 + 16]
}

/// The mouse-over's loop (E24 M3 T6a1): `hover::tests::fixture`'s 8 × 8 map from (0, 0), Red's
/// hero A at (4, 3) selected with 100 points, the pointer parked.
fn hovering(live: bool, bugs: Bugs) -> Loop {
    let (_, _, hover) = hover::tests::fixture(false);
    let mut kingdom = hover::tests::kingdom();
    kingdom.heroes[0].points = 100;
    let start = Start {
        view: ViewAt { x: 0, y: 0, z: 0 },
        size: 8,
        scroll_speed: 1,
        hero_speed: 2,
        pointer: PARK,
        hero: Some(Hero {
            at: Pos { x: 4, y: 3, z: 0 },
            facing: Direction::South,
        }),
        paths: Vec::new(),
        path: Vec::new(),
        live,
        sound: None,
        kingdom,
        points: Vec::new(),
        buttons: None,
        asked: None,
        hover: Some(hover),
        levels: 1,
        heroes: Vec::new(),
        index: None,
        chosen: crate::live::Chosen::default(),
    };
    Loop::start(rules(bugs), &start).unwrap()
}

#[test]
fn the_mouse_over_the_map_and_the_panels() {
    use crate::hover::frame;
    let mut l = hovering(false, Bugs::Original);
    let r = rules(Bugs::Original);
    // At 0 the parked pointer's item, the adventure options.
    assert_eq!(l.state(0).drawn.hint, Hint::Panel(8));
    let shown = |l: &Loop, t| {
        let s = l.state(t);
        (s.cursor.frame, s.drawn.hint.clone())
    };
    // A guarded tile: the attack, no hint; the monster: the attack a day away, its hint.
    l.run(1, &[Action::Pointer(on(4, 4))]).unwrap();
    assert_eq!(shown(&l, 1), (frame::ATTACK, Hint::Empty));
    l.run(2, &[Action::Pointer(on(5, 4))]).unwrap();
    let monster = Hint::Monster {
        creature: 13,
        count: 7,
    };
    assert_eq!(shown(&l, 2), (frame::ATTACK + frame::DAY, monster.clone()));
    // A tile not explored: the arrow, the hint left.
    l.run(3, &[Action::Pointer(on(0, 6))]).unwrap();
    assert_eq!(shown(&l, 3), (r.arrow, monster.clone()));
    // The parking, its item the one looked at last: the hint left (MCL-Q17).
    l.run(4, &[Action::Pointer(PARK)]).unwrap();
    assert_eq!(shown(&l, 4), (r.arrow, monster));
    // Over the hero, then elsewhere in the same tile: the hero once.
    l.run(5, &[Action::Pointer(on(4, 3))]).unwrap();
    let a = Hint::Hero {
        id: 16,
        class: 2,
        own_name: false,
    };
    assert_eq!(shown(&l, 5), (frame::HERO, a.clone()));
    l.run(6, &[Action::Pointer(PARK)]).unwrap();
    let [x, y] = on(4, 3);
    l.run(7, &[Action::Pointer([x + 3, y + 3])]).unwrap();
    assert_eq!(shown(&l, 7), (r.arrow, a.clone()));
    // A panel's item, then between the panels: the arrow and nothing.
    l.run(8, &[Action::Pointer([695, 212])]).unwrap();
    assert_eq!(shown(&l, 8), (r.arrow, Hint::Panel(2)));
    l.run(9, &[Action::Pointer([604, 180])]).unwrap();
    assert_eq!(shown(&l, 9), (r.arrow, Hint::Empty));
    // The hero list's first slot: hero A.
    l.run(10, &[Action::Pointer([641, 228])]).unwrap();
    assert_eq!(shown(&l, 10).1, a);
}

#[test]
fn the_tile_under_the_mouse_is_the_drawn_ones() {
    // A drawn row's last 8 pixels (MCL-Q18): the client's row, the original's the one below.
    let view = ViewAt { x: 3, y: 4, z: 1 };
    let fixed = rules(Bugs::Fixed);
    let original = rules(Bugs::Original);
    assert_eq!(fixed.column_at([368, 100]), [11, 2]);
    assert_eq!(original.column_at([368, 100]), [11, 3]);
    assert_eq!(fixed.tile_at(view, [368, 100]), [14, 6, 1]);
    assert_eq!(original.tile_at(view, [368, 100]), [14, 7, 1]);
    // The view's first drawn row starts at its origin's y (8).
    assert_eq!(fixed.column_at([0, 8]), [0, 0]);
    assert_eq!(fixed.column_at([31, 39]), [0, 0]);
    assert_eq!(fixed.column_at([32, 40]), [1, 1]);
}

#[test]
fn the_cursor_without_the_selected_heros_facts() {
    use crate::hover::frame;
    // No hero selected: Red's hero B, the no-hero cursor.
    let mut l = hovering(false, Bugs::Fixed);
    l.hero = None;
    l.selected_id = None;
    l.run(1, &[Action::Pointer(on(7, 3))]).unwrap();
    assert_eq!(l.state(1).cursor.frame, frame::HERO);
    // A hero the facts do not hold: none selected either.
    let mut l = hovering(false, Bugs::Fixed);
    l.selected_id = None;
    l.run(1, &[Action::Pointer(on(2, 2))]).unwrap();
    assert_eq!(l.state(1).cursor.frame, frame::TOWN);
}

#[test]
fn a_click_looks_at_its_tile_first() {
    use crate::hover::frame;
    let mut l = hovering(true, Bugs::Fixed);
    // Live, a click on a tile no path leads to only moves the pointer: its tile looked at.
    l.run(1, &[Action::Click(on(5, 4))]).unwrap();
    let s = l.state(1);
    assert_eq!(s.cursor.frame, frame::ATTACK + frame::DAY);
    assert!(matches!(s.drawn.hint, Hint::Monster { .. }));
    // After a step the reach is stale: the arrow over a tile it reached.
    l.run(2, &[Action::Step(Direction::East)]).unwrap();
    for t in 3..600 {
        l.run(t, &[]).unwrap();
    }
    l.run(600, &[Action::Pointer(on(4, 4))]).unwrap();
    assert_eq!(l.state(600).cursor.frame, rules(Bugs::Fixed).arrow);
}

#[test]
fn an_edge_cursor_stays_in_its_zone_outside_the_view() {
    for bugs in [Bugs::Fixed, Bugs::Original] {
        let r = rules(bugs);
        let mut l = looped(bugs, 1);
        l.run(1, &[Action::Pointer([2, 300])]).unwrap();
        for t in 2..=71 {
            l.run(t, &[]).unwrap();
        }
        assert_eq!(l.state(71).cursor.frame, r.edges + 6, "{bugs:?}");
        l.run(72, &[Action::Pointer([3, 301])]).unwrap();
        assert_eq!(l.state(72).cursor.frame, r.edges + 6, "{bugs:?}");
        // Out of the zone outside the view: the arrow.
        l.run(73, &[Action::Pointer([604, 180])]).unwrap();
        assert_eq!(l.state(73).cursor.frame, r.arrow, "{bugs:?}");
    }
    let r = rules(Bugs::Original);
    assert!(r.keeps_edge([785, 300]) && !r.keeps_edge([784, 300]) && !r.keeps_edge([-1, 2]));
    assert!(!r.keeps_edge([2, 600]));
}

/// The recorded sequences of the lists' clicks and the buttons' presses (E24 M3 T6b1): their
/// situation, timeline and live part at 0 (with the `[chosen]` table).
const CHOOSING: [(&str, &str, &str); 2] = [
    (
        "choose-lists",
        include_str!("../../../../map/timelines/choose-lists.txt"),
        include_str!("../../../../map/timelines/choose-lists-live.toml"),
    ),
    (
        "press-buttons",
        include_str!("../../../../map/timelines/press-buttons.txt"),
        include_str!("../../../../map/timelines/press-buttons-live.toml"),
    ),
];

/// A choices' sequence's start (as the binary's `for_sequence` makes it): the kingdom, the
/// buttons, what each choice shows, the map's heroes, the selected one and its path at 0.
fn choosing(id: &str) -> (Start, BTreeMap<u32, Vec<Action>>, u32, &'static str) {
    let sequences = Sequences::parse(SEQUENCES).unwrap();
    let sequence = sequences.get(id).unwrap();
    let (_, timeline, live) = CHOOSING.iter().find(|(c, _, _)| *c == id).unwrap();
    let situation = include_str!("../../../../map/situations/seq-lists.toml");
    let mut start = start(situation, sequence.scroll_speed().unwrap());
    (start.kingdom, start.buttons) = live_kingdom(live);
    let table: toml::Table = live.parse().unwrap();
    start.chosen = Chosen::read(&table["chosen"]).unwrap();
    let (heroes, _) = live_heroes(live);
    start.heroes = heroes
        .iter()
        .map(|h| Hero {
            at: h.at,
            facing: Direction::ALL[usize::from(h.facing.number())],
        })
        .collect();
    let Selected::Hero(i) = start.kingdom.selected else {
        panic!("{id}: no hero selected at 0");
    };
    let index = start.chosen.heroes[i].as_ref().unwrap().hero;
    (start.index, start.hero) = (Some(index), Some(start.heroes[index]));
    start.path = start.chosen.heroes[i].as_ref().unwrap().path.clone();
    start.points = sequence.points(timeline).unwrap();
    start.paths = sequence.paths(timeline).unwrap();
    (start, sequence.actions().unwrap(), sequence.ms, timeline)
}

#[test]
fn replays_the_recorded_choices() {
    for (id, _, _) in CHOOSING {
        let (start, actions, ms, timeline) = choosing(id);
        let states = play(rules(Bugs::Original), &start, &actions, ms).unwrap();
        let got = lines(&states, &BTreeMap::new());
        let want = without_screens(timeline);
        for (n, (g, w)) in got.iter().zip(&want).enumerate() {
            assert_eq!(g, w, "{id}: line {n}");
        }
        assert_eq!(got.len(), want.len(), "{id}");
    }
}

#[test]
fn a_choice_moves_the_selection_and_its_draws() {
    let (start, actions, _, _) = choosing("choose-lists");
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    for t in 1..=200 {
        l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
            .unwrap();
    }
    // The third hero chosen at 200: selected, its index the draw's and the minimap's, the hint
    // hidden by the dialog's redraw (the item looked at last kept).
    let d = l.drawn();
    assert_eq!(l.kingdom.selected, Selected::Hero(2));
    assert_eq!((d.selected, d.minimap.index), (l.index, l.index));
    assert_eq!(d.hint, Hint::Empty);
    assert_eq!(l.last_item, Some(41));
    // A town chosen: no hero selected, none drawn.
    for t in 201..=800 {
        l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
            .unwrap();
    }
    assert_eq!(l.kingdom.selected, Selected::Town(1));
    assert_eq!((l.drawn().selected, l.state(800).hero), (None, None));
    // A button held: only the tick runs, its press drawn; the release over it its command.
    for t in 801..=1100 {
        l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
            .unwrap();
    }
    assert_eq!(l.held, Some((14, true)));
    assert_eq!(l.drawn().pressed, [14]);
    let steps = l.steps;
    for t in 1101..1250 {
        l.run(t, &[]).unwrap();
    }
    assert_eq!(l.steps, steps);
    l.run(1250, &actions[&1250]).unwrap();
    assert_eq!((l.held, l.kingdom.tops[0]), (None, 1));
    assert!(l.drawn().pressed.is_empty());
}

#[test]
fn the_buttons_hotkeys_and_their_skips() {
    let (start, _, _, _) = choosing("choose-lists");
    let r = rules(Bugs::Original);
    let mut l = Loop::start(r.clone(), &start).unwrap();
    // Q without a quest, U on one level, W with the hero awake, M without a green arrow: none.
    l.buttons.move_hero = false;
    for key in [
        Hotkey::Quests,
        Hotkey::Underground,
        Hotkey::Wake,
        Hotkey::MoveHero,
    ] {
        l.run(1, &[Action::Key(key)]).unwrap();
    }
    assert!(l.drawn().pressed.is_empty() && l.keyed.is_empty());
    // Z presses sleep, twice its key down one press; its release is sleep's command (refused in
    // a recording, nothing live); a key up that pressed nothing nothing.
    l.run(2, &[Action::Key(Hotkey::Sleep), Action::Key(Hotkey::Sleep)])
        .unwrap();
    assert_eq!((l.drawn().pressed.as_slice(), l.keyed.len()), (&[6][..], 1));
    assert_eq!(
        l.run(3, &[Action::KeyUp(Hotkey::Sleep)]),
        Err(Unplayable::Command(6))
    );
    assert!(l.drawn().pressed.is_empty());
    l.run(4, &[Action::KeyUp(Hotkey::Kingdom)]).unwrap();
    // The hero asleep: W presses sleep's button, Z none; with a quest Q the quest log.
    let mut asleep = start.clone();
    asleep.kingdom.heroes[0].asleep = true;
    asleep.kingdom.quests = true;
    asleep.live = true;
    let mut l = Loop::start(r.clone(), &asleep).unwrap();
    l.run(1, &[Action::Key(Hotkey::Sleep), Action::Key(Hotkey::Wake)])
        .unwrap();
    l.run(2, &[Action::Key(Hotkey::Quests)]).unwrap();
    assert_eq!(l.drawn().pressed, [5, 6]);
    // Live, a command not built does nothing.
    l.run(3, &[Action::KeyUp(Hotkey::Wake)]).unwrap();
    assert_eq!(l.drawn().pressed, [5]);
    // The quest log taken by the mouse: pressed with a quest, not without.
    l.run(4, &[Action::Press([695, 244])]).unwrap();
    assert_eq!(l.held, Some((5, true)));
    l.run(5, &[Action::Release([695, 244])]).unwrap();
    let mut l = Loop::start(r, &start).unwrap();
    l.run(1, &[Action::Press([695, 244])]).unwrap();
    assert_eq!(l.held, None);
}

#[test]
fn a_hold_and_what_it_drops() {
    let (start, _, _, _) = choosing("choose-lists");
    let r = rules(Bugs::Original);
    let mut l = Loop::start(r.clone(), &start).unwrap();
    // The kingdom overview held: off it released, back on pressed; keys and presses dropped;
    // released off it, no command.
    l.run(1, &[Action::Press([695, 212])]).unwrap();
    l.run(2, &[Action::Pointer([695, 185])]).unwrap();
    assert_eq!((l.held, l.drawn().pressed.len()), (Some((3, false)), 0));
    l.run(3, &[Action::Pointer([695, 212])]).unwrap();
    assert_eq!(l.drawn().pressed, [3]);
    l.run(
        4,
        &[
            Action::Key(Hotkey::Sleep),
            Action::Press([641, 292]),
            Action::Step(Direction::North),
        ],
    )
    .unwrap();
    assert_eq!(
        (l.drawn().pressed.as_slice(), l.kingdom.selected),
        (&[3][..], start.kingdom.selected)
    );
    l.run(5, &[Action::Release([695, 185])]).unwrap();
    assert_eq!((l.held, l.drawn().pressed.len()), (None, 0));
    // Clicked at once on a list's arrow: its command; on the minimap its loop at once over
    // (E24 M3 T6b2), on the status window its click (no kingdom's window known: nothing built),
    // on no item nothing.
    l.run(6, &[Action::Click([641, 380])]).unwrap();
    assert_eq!(l.kingdom.tops[0], 1);
    l.run(7, &[Action::Click([700, 100])]).unwrap();
    assert!(!l.dragging);
    l.run(8, &[Action::Click([700, 480]), Action::Click([700, 2])])
        .unwrap();
    assert_eq!((l.asked, l.status_held), (2, false));
    // Each list's arrows held at its ends: the up one at the top, the down one past the end.
    let mut l = Loop::start(r, &start).unwrap();
    l.scroll_list(false, true);
    l.scroll_list(true, true);
    assert_eq!(l.kingdom.tops, [0, 0]);
    for _ in 0..5 {
        l.scroll_list(false, false);
        l.scroll_list(true, false);
    }
    assert_eq!(l.kingdom.tops, [2, 1]);
}

#[test]
fn the_lists_slots_and_what_they_refuse() {
    let (start, _, _, _) = choosing("choose-lists");
    let r = rules(Bugs::Original);
    // The selected hero's slot: its screen (M4, M5), refused; live nothing.
    let mut l = Loop::start(r.clone(), &start).unwrap();
    assert_eq!(
        l.run(1, &[Action::Press([641, 228])]),
        Err(Unplayable::Panel([641, 228]))
    );
    let mut live = start.clone();
    live.live = true;
    let mut l = Loop::start(r.clone(), &live).unwrap();
    l.run(1, &[Action::Press([641, 228])]).unwrap();
    // A town chosen, then its slot again: its screen.
    l.run(2, &[Action::Press([771, 228])]).unwrap();
    assert_eq!(l.kingdom.selected, Selected::Town(0));
    l.run(3, &[Action::Press([771, 228])]).unwrap();
    assert_eq!(l.kingdom.selected, Selected::Town(0));
    // A hero or town whose choice the live part lacks: refused.
    let mut unknown = start.clone();
    unknown.chosen.heroes[2] = None;
    unknown.chosen.towns[0] = None;
    let mut l = Loop::start(r.clone(), &unknown).unwrap();
    assert_eq!(
        l.run(1, &[Action::Press([641, 292])]),
        Err(Unplayable::Panel([641, 292]))
    );
    assert_eq!(
        l.run(2, &[Action::Press([771, 228])]),
        Err(Unplayable::Panel([771, 228]))
    );
    // A hero whose index names no map's hero: unknown too.
    if let Some(c) = &mut unknown.chosen.heroes[1] {
        c.hero = 99;
    }
    let mut l = Loop::start(r.clone(), &unknown).unwrap();
    assert!(l.run(1, &[Action::Press([641, 260])]).is_err());
    // Two heroes: a selection frame past the list takes the click and does nothing.
    let mut two = start.clone();
    two.kingdom.heroes.truncate(2);
    let mut l = Loop::start(r.clone(), &two).unwrap();
    l.run(1, &[Action::Press([641, 324])]).unwrap();
    assert_eq!(l.kingdom.selected, start.kingdom.selected);
    // Next hero with every hero asleep: none chosen, the path cleared.
    let mut asleep = start.clone();
    for h in &mut asleep.kingdom.heroes {
        h.asleep = true;
    }
    let mut l = Loop::start(r, &asleep).unwrap();
    l.next_hero(1);
    assert_eq!(l.kingdom.selected, start.kingdom.selected);
    assert!(l.path.is_empty());
}

#[test]
fn a_choice_with_the_facts_and_the_sound() {
    // The mouse-over's facts name the hero chosen; its choice unknown for the one selected, so
    // the selected hero's path is not kept.
    let mut l = hovering(true, Bugs::Original);
    l.heroes = vec![
        Hero {
            at: Pos { x: 4, y: 3, z: 0 },
            facing: Direction::South,
        },
        Hero {
            at: Pos { x: 2, y: 5, z: 0 },
            facing: Direction::East,
        },
    ];
    l.index = Some(0);
    let status = crate::live::HeroStatus {
        hero: 1,
        own_name: false,
        skills: [0; 4],
        morale: 0,
        luck: 0,
        army: [None; 7],
    };
    l.chosen.heroes = vec![
        None,
        Some(ChosenHero {
            hero: 1,
            status,
            path: Vec::new(),
        }),
    ];
    let id = l.hover.as_ref().unwrap().facts().hero_list[1].0;
    l.run(1, &[Action::Press([641, 260])]).unwrap();
    assert_eq!((l.selected_id, l.index), (Some(id), Some(1)));
    // A selected hero whose place names no map's hero: its tile is not kept; next hero to a hero
    // past the choices: none chosen.
    l.index = Some(9);
    l.chosen.heroes.truncate(1);
    for h in &mut l.kingdom.heroes {
        h.movable = true;
    }
    l.next_hero(2);
    assert_eq!(l.kingdom.selected, Selected::Hero(1));
    l.kingdom.selected = Selected::Hero(0);
    l.next_hero(3);
    assert_eq!(l.kingdom.selected, Selected::Hero(0));
    // A town chosen and a button pressed with the sound: its cues.
    let (start, _, _, _) = choosing("choose-lists");
    let tiles = crate::sound::tests::tiles(36, |x, _| u8::from(x > 10), |_, _| None);
    let sounded = Start {
        sound: Some(crate::sound::tests::sound(tiles)),
        ..start
    };
    let mut l = Loop::start(rules(Bugs::Original), &sounded).unwrap();
    l.take_cues();
    l.run(1, &[Action::Press([771, 260])]).unwrap();
    assert!(
        l.take_cues()
            .iter()
            .any(|(_, c)| matches!(c, Cue::Music(_)))
    );
    l.run(2, &[Action::Press([695, 212])]).unwrap();
    assert_eq!(l.take_cues(), [(2, Cue::Effect("button.wav".into()))]);
}

/// A start on `seq-underground` (E24 M3 T6b2): a map of two levels, Red's first hero selected,
/// its status window the hero's and the kingdom's window known; the view at (0, 0, 0).
fn underground_start() -> Start {
    let situation = include_str!("../../../../map/situations/seq-underground.toml");
    let mut start = start(situation, 1);
    start.levels = 2;
    let hero = Hero {
        at: Pos { x: 10, y: 12, z: 0 },
        facing: Direction::East,
    };
    (start.hero, start.heroes, start.index) = (Some(hero), vec![hero], Some(0));
    start.kingdom = Kingdom {
        heroes: vec![KingdomHero {
            movable: true,
            portrait: 2,
            points: 1500,
            ..KingdomHero::default()
        }],
        selected: Selected::Hero(0),
        status: Some(Status::Hero(crate::live::HeroStatus {
            hero: 2,
            own_name: false,
            skills: [1, 2, 3, 4],
            morale: 0,
            luck: 0,
            army: [None; 7],
        })),
        realm: Some(KingdomStatus {
            halls: [0, 1, 0, 0],
            allies: vec![0],
            enemies: vec![1],
        }),
        ..Kingdom::default()
    };
    start
}

#[test]
fn the_underground_s_button_and_key() {
    let r = rules(Bugs::Original);
    let start = underground_start();
    let mut l = Loop::start(r.clone(), &start).unwrap();
    l.lifted = true;
    // The button clicked: the other level at the same corner, the hero put back, the whole
    // screen updated (the hint hidden, the status window built again: the same), no sound.
    l.run(1, &[Action::Press([727, 212])]).unwrap();
    assert_eq!(l.drawn().pressed, [4]);
    l.run(2, &[Action::Release([727, 212])]).unwrap();
    assert_eq!(l.view, ViewAt { x: 0, y: 0, z: 1 });
    assert_eq!((l.drawn().view.z, l.drawn().minimap.view.z), (1, 1));
    assert!(!l.lifted && l.drawn().pressed.is_empty());
    assert_eq!(l.drawn().hint, Hint::Empty);
    assert_eq!(l.kingdom.status, start.kingdom.status);
    // Its key: pressed by U, clicked by its release; back to the surface.
    l.run(3, &[Action::Key(Hotkey::Underground)]).unwrap();
    assert_eq!(l.drawn().pressed, [4]);
    l.run(4, &[Action::KeyUp(Hotkey::Underground)]).unwrap();
    assert_eq!(l.view.z, 0);
    // A map of one level: the command does nothing (its button disabled takes nothing).
    let mut one = start.clone();
    one.levels = 1;
    let mut l = Loop::start(r, &one).unwrap();
    l.underground(1);
    assert_eq!(l.view.z, 0);
    l.run(2, &[Action::Key(Hotkey::Underground)]).unwrap();
    assert!(l.drawn().pressed.is_empty());
}

#[test]
fn the_minimap_s_click_and_its_loop() {
    let r = rules(Bugs::Original);
    let start = underground_start();
    let mut l = Loop::start(r.clone(), &start).unwrap();
    l.lifted = true;
    let corner = |l: &Loop| [l.view.x, l.view.y];
    // A press: the tile under it (4 pixels a tile on 36 tiles) less the view's middle, drawn with
    // the minimap and the objects' step when due; the hero put back.
    l.run(200, &[Action::Press([650, 50])]).unwrap();
    assert_eq!(corner(&l), [5 - 9, 6 - 8]);
    assert!(l.dragging && !l.lifted);
    assert_eq!((l.drawn().view, l.drawn().minimap.view), (l.view, l.view));
    let stepped = l.steps;
    // Moves: inside, past the right and bottom edges (held at the edge: the last tile), past the
    // top-left corner (the first).
    l.run(201, &[Action::Pointer([700, 100])]).unwrap();
    assert_eq!(corner(&l), [17 - 9, 18 - 8]);
    l.run(202, &[Action::Pointer([790, 200])]).unwrap();
    assert_eq!(corner(&l), [35 - 9, 35 - 8]);
    l.run(203, &[Action::Pointer([600, 20])]).unwrap();
    assert_eq!(corner(&l), [-9, -8]);
    // While held, nothing else: no idle redraw, keys, steps, scrolls and the edge dropped.
    let drawn = l.drawn().clone();
    l.run(
        600,
        &[
            Action::Key(Hotkey::NextHero),
            Action::Step(Direction::North),
            Action::Scroll(Direction::South),
            Action::Press([727, 212]),
        ],
    )
    .unwrap();
    assert_eq!((l.drawn(), l.steps, l.keyed.len()), (&drawn, stepped, 0));
    // Released where it was: no move, no redraw, then the loop's next turn redraws as due.
    l.run(601, &[Action::Release([600, 20])]).unwrap();
    assert!(!l.dragging);
    assert!(l.steps > stepped);
    // Released at another point: its move drawn first.
    l.run(700, &[Action::Press([650, 50])]).unwrap();
    l.run(701, &[Action::Release([740, 90])]).unwrap();
    assert_eq!((corner(&l), l.dragging), ([27 - 9, 16 - 8], false));
    // A click: pressed and released at once.
    l.run(800, &[Action::Click([773, 169])]).unwrap();
    assert_eq!((corner(&l), l.dragging), ([26, 27], false));
    // On the underground the minimap's level follows the view.
    l.underground(900);
    l.run(901, &[Action::Click([630, 26])]).unwrap();
    assert_eq!(l.drawn().minimap.view, ViewAt { x: -9, y: -8, z: 1 });
    // A map of 108 tiles divides by 1.3333 (not the box's 1.33): its last pixel the last tile.
    let mut big = start.clone();
    big.size = 108;
    let mut l = Loop::start(r.clone(), &big).unwrap();
    l.run(1, &[Action::Press([773, 169])]).unwrap();
    assert_eq!(corner(&l), [107 - 9, 107 - 8]);
    l.run(2, &[Action::Pointer([700, 120])]).unwrap();
    assert_eq!(corner(&l), [52 - 9, 70 - 8]);
    // Without the layout's minimap, no loop.
    let mut bare = r;
    bare.panels = PanelItems::default();
    let mut l = Loop::start(bare, &start).unwrap();
    l.minimap_press(1, [650, 50]);
    assert!(!l.dragging);
    l.dragging = true;
    l.drag(2, Action::Pointer([660, 60]));
    assert_eq!(corner(&l), [0, 0]);
}

#[test]
fn the_status_window_s_click() {
    let r = rules(Bugs::Original);
    let mut start = underground_start();
    start.sound = None;
    let hero = start.kingdom.status.clone();
    let realm = start.kingdom.realm.clone().map(Status::Kingdom);
    let mut l = Loop::start(r.clone(), &start).unwrap();
    // Pressed on the window, released elsewhere: its click, the kingdom's for 3 000 ms.
    l.run(1, &[Action::Press([700, 480])]).unwrap();
    assert!(l.status_held);
    l.run(2, &[Action::Pointer([700, 580])]).unwrap();
    l.run(3, &[Action::Release([700, 580])]).unwrap();
    assert_eq!((l.asked, l.end, l.status_held), (2, 3003, false));
    assert_eq!((&l.kingdom.status, &l.kingdom.selection), (&realm, &hero));
    assert_eq!(l.drawn().kingdom.status, realm);
    // The kingdom's gives the date's (built at its first frame), the date's the hero's.
    l.run(4, &[Action::Click([700, 480])]).unwrap();
    assert_eq!((l.asked, l.end), (1, 3004));
    assert_eq!(
        l.kingdom.status,
        Some(Status::Date(DateStatus { frame: 0 }))
    );
    l.run(5, &[Action::Click([700, 480])]).unwrap();
    assert_eq!((l.asked, &l.kingdom.status), (0, &hero));
    // The selection's while the kingdom's shows: the date's.
    l.kingdom.status.clone_from(&realm);
    l.run(6, &[Action::Click([700, 480])]).unwrap();
    assert_eq!(l.asked, 1);
    // A press the window built again drops: its release does nothing.
    l.run(7, &[Action::Click([700, 480])]).unwrap();
    assert_eq!(l.asked, 0);
    l.run(8, &[Action::Click([700, 480])]).unwrap();
    assert_eq!(l.asked, 2);
    l.run(3000, &[Action::Press([700, 480])]).unwrap();
    for t in 3001..=3200 {
        l.run(t, &[]).unwrap();
    }
    assert_eq!(
        (l.asked, l.status_held, &l.kingdom.status),
        (0, false, &hero)
    );
    l.run(3201, &[Action::Release([700, 480])]).unwrap();
    assert_eq!(l.asked, 0);
    // Outside the window's picture (the area around it), or with no window known: nothing held.
    l.run(3300, &[Action::Press([608, 392])]).unwrap();
    assert!(!l.status_held);
    let mut none = start.clone();
    none.kingdom.status = None;
    let mut l = Loop::start(r, &none).unwrap();
    l.run(1, &[Action::Press([700, 480])]).unwrap();
    assert!(!l.status_held);
}

/// The recorded sequences of the underground, the minimap and the status window (E24 M3 T6b2):
/// their timelines and live parts at 0, on `seq-underground`.
const CLICKING: [(&str, &str, &str); 3] = [
    (
        "underground",
        include_str!("../../../../map/timelines/underground.txt"),
        include_str!("../../../../map/timelines/underground-live.toml"),
    ),
    (
        "minimap-drag",
        include_str!("../../../../map/timelines/minimap-drag.txt"),
        include_str!("../../../../map/timelines/minimap-drag-live.toml"),
    ),
    (
        "status-click",
        include_str!("../../../../map/timelines/status-click.txt"),
        include_str!("../../../../map/timelines/status-click-live.toml"),
    ),
];

#[test]
fn replays_the_recorded_clicks() {
    let situation = include_str!("../../../../map/situations/seq-underground.toml");
    let sequences = Sequences::parse(SEQUENCES).unwrap();
    for (id, timeline, live) in CLICKING {
        let sequence = sequences.get(id).unwrap();
        let mut start = start(situation, sequence.scroll_speed().unwrap());
        start.levels = 2;
        (start.kingdom, start.buttons) = live_kingdom(live);
        assert!(start.kingdom.realm.is_some(), "{id}");
        // The selected hero by the mouse-over's facts, as the binary's `selected_in` finds it.
        let (heroes, _) = live_heroes(live);
        let table: toml::Table = live.parse().unwrap();
        let objects = table["anchors"].as_array().unwrap().len();
        let facts = hover::Facts::read(&table["hover"], objects, heroes.len(), 36, 2).unwrap();
        let Selected::Hero(slot) = start.kingdom.selected else {
            panic!("{id}: no hero selected at 0");
        };
        let selected = facts
            .heroes
            .iter()
            .position(|h| h.0 == facts.hero_list[slot].0)
            .unwrap();
        let hero = Hero {
            at: heroes[selected].at,
            facing: Direction::ALL[usize::from(heroes[selected].facing.number())],
        };
        start.heroes = heroes
            .iter()
            .map(|h| Hero {
                at: h.at,
                facing: Direction::ALL[usize::from(h.facing.number())],
            })
            .collect();
        start.index = start.heroes.iter().position(|h| h.at == hero.at);
        start.hero = Some(hero);
        start.points = sequence.points(timeline).unwrap();
        start.paths = sequence.paths(timeline).unwrap();
        let states = play(
            rules(Bugs::Original),
            &start,
            &sequence.actions().unwrap(),
            sequence.ms,
        )
        .unwrap();
        let got = lines(&states, &BTreeMap::new());
        let want = without_screens(timeline);
        for (n, (g, w)) in got.iter().zip(&want).enumerate() {
            assert_eq!(g, w, "{id}: line {n}");
        }
        assert_eq!(got.len(), want.len(), "{id}");
    }
}

/// E24 M3 T6b3's sequences on `seq-walk`: their timelines and kept live parts.
const WALKING: [(&str, &str, &str); 4] = [
    (
        "walk-queue",
        include_str!("../../../../map/timelines/walk-queue.txt"),
        include_str!("../../../../map/timelines/walk-queue-live.toml"),
    ),
    (
        "walk-stop-key",
        include_str!("../../../../map/timelines/walk-stop-key.txt"),
        include_str!("../../../../map/timelines/walk-stop-key-live.toml"),
    ),
    (
        "walk-stop-press",
        include_str!("../../../../map/timelines/walk-stop-press.txt"),
        include_str!("../../../../map/timelines/walk-stop-press-live.toml"),
    ),
    (
        "walk-stop-space",
        include_str!("../../../../map/timelines/walk-stop-space.txt"),
        include_str!("../../../../map/timelines/walk-stop-space-live.toml"),
    ),
];

#[test]
fn replays_the_recorded_keys_during_walks() {
    let situation = include_str!("../../../../map/situations/seq-walk.toml");
    let sequences = Sequences::parse(SEQUENCES).unwrap();
    for (id, timeline, live) in WALKING {
        let sequence = sequences.get(id).unwrap();
        let mut start = start(situation, sequence.scroll_speed().unwrap());
        (start.kingdom, start.buttons) = live_kingdom(live);
        // The selected hero by the mouse-over's facts, as the binary's `selected_in` finds it.
        let (heroes, _) = live_heroes(live);
        let table: toml::Table = live.parse().unwrap();
        let objects = table["anchors"].as_array().unwrap().len();
        let facts = hover::Facts::read(&table["hover"], objects, heroes.len(), 36, 1).unwrap();
        let Selected::Hero(slot) = start.kingdom.selected else {
            panic!("{id}: no hero selected at 0");
        };
        let selected = facts
            .heroes
            .iter()
            .position(|h| h.0 == facts.hero_list[slot].0)
            .unwrap();
        let hero = Hero {
            at: heroes[selected].at,
            facing: Direction::ALL[usize::from(heroes[selected].facing.number())],
        };
        start.heroes = heroes
            .iter()
            .map(|h| Hero {
                at: h.at,
                facing: Direction::ALL[usize::from(h.facing.number())],
            })
            .collect();
        start.index = start.heroes.iter().position(|h| h.at == hero.at);
        start.hero = Some(hero);
        start.points = sequence.points(timeline).unwrap();
        start.paths = sequence.paths(timeline).unwrap();
        let states = play(
            rules(Bugs::Original),
            &start,
            &sequence.actions().unwrap(),
            sequence.ms,
        )
        .unwrap();
        let got = lines(&states, &BTreeMap::new());
        let want = without_screens(timeline);
        for (n, (g, w)) in got.iter().zip(&want).enumerate() {
            assert_eq!(g, w, "{id}: line {n}");
        }
        assert_eq!(got.len(), want.len(), "{id}");
    }
}
