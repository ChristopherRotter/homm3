//! A boat boarded and left (E24 M4 T2d2, design §5.7): the hero dissolved off the land before
//! its step onto an empty boat, the boat lifted as its picture at the step's end; the boat put
//! back on the water before a step onto land, the hero dissolved in at its end.

use super::*;

use crate::live::{Explored, LiveBoat, Moment, Walking};
use crate::sound::Cue;

use super::turns::started_by;

/// The sequences' situations, timelines and kept live parts.
const BOATING: [(&str, &str, &str, &str); 4] = [
    (
        "board",
        include_str!("../../../../../map/situations/seq-board.toml"),
        include_str!("../../../../../map/timelines/board.txt"),
        include_str!("../../../../../map/timelines/board-live.toml"),
    ),
    (
        "board-walk",
        include_str!("../../../../../map/situations/seq-board-walk.toml"),
        include_str!("../../../../../map/timelines/board-walk.txt"),
        include_str!("../../../../../map/timelines/board-walk-live.toml"),
    ),
    (
        "land",
        include_str!("../../../../../map/situations/seq-land.toml"),
        include_str!("../../../../../map/timelines/land.txt"),
        include_str!("../../../../../map/timelines/land-live.toml"),
    ),
    (
        "land-walk",
        include_str!("../../../../../map/situations/seq-land.toml"),
        include_str!("../../../../../map/timelines/land-walk.txt"),
        include_str!("../../../../../map/timelines/land-walk-live.toml"),
    ),
];

/// The map's side.
const SIDE: usize = 36;

/// The inlet the boats lie on and sail: (9, 12) to (12, 12), the rest land (the tiles the
/// sequences step on: the map is Deluge's lake, which the tests do not read).
fn water(x: usize, y: usize) -> bool {
    y == 12 && (9..=12).contains(&x)
}

/// The live part's heroes and boats as the binary's live part holds them.
fn live_part(text: &str) -> Live {
    let table: toml::Table = text.parse().unwrap();
    Live {
        objects: Vec::new(),
        heroes: LiveHero::read_all(&table["heroes"]).unwrap(),
        boats: LiveBoat::read_all(&table["boats"]).unwrap(),
        viewer: 0,
        explored: Explored::all(SIDE, 1, true),
        path: Vec::new(),
        moment: Moment::default(),
        walking: Walking::default(),
        left: Vec::new(),
        kingdom: Kingdom::default(),
        buttons: None,
        minimap: None,
        hover: None,
        hint: None,
        chosen: Chosen::default(),
        pressed: Vec::new(),
        turns: Vec::new(),
        computer: false,
        level_ups: Vec::new(),
        casts: Vec::new(),
        seats: Vec::new(),
        covered: None,
        chat: Vec::new(),
    }
}

/// A boat sequence's start (its boats the live part's, the inlet's water), its actions, its end
/// and its timeline.
fn boating(id: &str) -> (Start, BTreeMap<u32, Vec<Action>>, u32, &'static str) {
    let sequences = Sequences::parse(SEQUENCES).unwrap();
    let entry = BOATING.iter().find(|(b, ..)| *b == id).unwrap();
    let (mut start, actions, ms, timeline) = started_by(sequences.get(id).unwrap(), entry);
    start.boats = Boats::of(&live_part(entry.3));
    start.water = Arc::new(
        (0..SIDE * SIDE)
            .map(|i| water(i % SIDE, i / SIDE))
            .collect(),
    );
    (start, actions, ms, timeline)
}

#[test]
fn replays_the_recorded_boats() {
    for (id, ..) in BOATING {
        let (start, actions, ms, timeline) = boating(id);
        let states = play(rules(Bugs::Original), &start, &actions, ms).unwrap();
        let got = lines(&states, &BTreeMap::new());
        let want = without_screens(timeline);
        for (n, (g, w)) in got.iter().zip(&want).enumerate() {
            assert_eq!(g, w, "{id}: line {n}");
        }
        assert_eq!(got.len(), want.len(), "{id}");
    }
}

/// The loop of a boat sequence run to `until` with its actions, its start changed by `change`.
fn run_with(id: &str, until: u32, change: impl FnOnce(&mut Start)) -> Loop {
    let (mut start, actions, _, _) = boating(id);
    change(&mut start);
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    for t in 1..=until {
        l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
            .unwrap();
    }
    l
}

/// The same, its start as recorded.
fn run_to(id: &str, until: u32) -> Loop {
    run_with(id, until, |_| {})
}

/// The tile (x, 12).
fn at(x: u8) -> Pos {
    Pos { x, y: 12, z: 0 }
}

/// Red's boat of type 1, as a hero sails it.
const RED_BOAT: Boat = Boat { kind: 1, colour: 0 };

#[test]
fn the_boarding_dissolves_the_hero_off_the_land_first() {
    // `board`: the key's step at 100 lifts the hero, centres the view and shows the whole screen
    // with its own path's arrow on the boat; then the screen as shown dissolves into the view
    // without the hero and without the arrow, the boat on its tile, the palettes cycling on.
    let mut l = run_to("board", 100);
    let drawn = l.drawn().clone();
    let dissolve = drawn.dissolve.clone().unwrap();
    assert_eq!(dissolve.step, 0);
    let arrow = Arrow {
        at: at(9),
        arrow: DESTINATION,
    };
    assert_eq!(dissolve.before.path, [arrow]);
    assert_eq!(dissolve.before.lifted, Some([2, 0]));
    assert!(!dissolve.before.hidden);
    assert!(drawn.hidden && drawn.lifted.is_none() && drawn.path.is_empty());
    assert_eq!(drawn.hero.unwrap().at, at(8));
    assert_eq!(drawn.boats.empty.len(), 1);
    assert_eq!(l.aboard, Some(Aboard::Board(drawn.boats.empty[0])));
    assert!(!l.state(100).cursor.shown);
    // The mouse-over and the sounds read the hero as off its tile.
    assert_eq!(l.standing(), None);
    l.selected_id = Some(48);
    assert!(l.selection().unwrap().lifted);
    let l = run_to("board", 600);
    assert_eq!(l.drawn().dissolve.as_ref().unwrap().step, 7);
    assert_eq!(l.state(600).cycles, 3);
    // The step at 620: nothing lifted, the walking group's frame left at 0.
    let l = run_to("board", 670);
    let s = l.state(670);
    assert_eq!(s.hero.unwrap().at, at(9));
    assert!(s.drawn.hidden && s.drawn.dissolve.is_none());
    assert_eq!((s.walk.group, s.walk.frame), (7, 0));
}

#[test]
fn the_boat_boarded_is_lifted_as_the_heros() {
    // `board` at 770: the boat off its tile, the hero in it in Red's colour, its points 0; the
    // boat lifted in the group of its own facing (down-left: 3), the hero facing right; the key's
    // tail draws the view a frame step on; Move Hero and Next Hero off.
    let l = run_to("board", 770);
    let s = l.state(770);
    assert!(s.drawn.boats.empty.is_empty());
    assert_eq!(s.drawn.boats.sailed[0], Some(RED_BOAT));
    assert_eq!(s.drawn.lifted, Some([3, 0]));
    assert!(!s.drawn.hidden);
    assert_eq!(s.hero.unwrap().facing, Direction::East);
    assert_eq!((s.objects, s.drawn.frames), (2, 3));
    assert_eq!(s.points, Some(0));
    assert!(!l.kingdom.heroes[0].movable);
    assert_eq!(
        s.buttons,
        Buttons {
            move_hero: false,
            next_hero: false,
            left: false,
        }
    );
    assert!(s.cursor.shown && l.aboard.is_none() && l.dissolving.is_none());
    // `board-walk`: the dissolve at the third step's start, the boat lifted at 1270 in its
    // facing's group (right: 2), no key's tail.
    let l = run_to("board-walk", 600);
    assert!(l.drawn().hidden && l.dissolving.is_some() && l.moving.is_none());
    let l = run_to("board-walk", 1270);
    let s = l.state(1270);
    assert_eq!(s.drawn.boats.sailed[0], Some(RED_BOAT));
    assert_eq!(
        (s.drawn.lifted, s.objects, s.drawn.frames),
        (Some([2, 0]), 3, 3)
    );
}

#[test]
fn a_boarding_at_speed_4_ends_its_step_at_the_dissolves_end() {
    // One draw for the step (*provisional*: not recorded): at 620 the hero on the boat's tile,
    // the boat lifted, the mouse shown, the key's release taken.
    let l = run_with("board", 620, |start| start.hero_speed = 4);
    let s = l.state(620);
    assert_eq!(s.hero.unwrap().at, at(9));
    assert_eq!(s.drawn.boats.sailed[0], Some(RED_BOAT));
    assert!(s.cursor.shown && l.moving.is_none() && l.dissolving.is_none());
}

#[test]
fn the_landing_puts_the_boat_back_and_dissolves_the_hero_in() {
    // `land`: at 100 the boat back on the water facing the step's way (left), the hero drawn
    // nowhere through the step; at 250 the hero out of the boat, lifted, dissolving in.
    let l = run_to("land", 100);
    let drawn = l.drawn();
    let boat = LiveBoat {
        at: at(9),
        kind: 1,
        facing: Facing::Left,
    };
    assert_eq!(drawn.boats.empty, [boat]);
    assert_eq!(drawn.boats.sailed[0], Some(RED_BOAT));
    assert!(drawn.hidden && drawn.lifted.is_none());
    assert_eq!(l.aboard, Some(Aboard::Land));
    let l = run_to("land", 250);
    let s = l.state(250);
    assert_eq!(s.drawn.boats.sailed[0], None);
    assert_eq!(s.drawn.boats.empty, [boat]);
    assert_eq!(s.drawn.lifted, Some([2, 0]));
    assert!(!s.drawn.hidden && s.drawn.dissolve.is_some());
    assert!(s.drawn.dissolve.unwrap().before.hidden);
    assert_eq!(
        s.buttons,
        Buttons {
            move_hero: true,
            next_hero: false,
            left: false,
        }
    );
    // Its end at 770: the redraws, Move Hero off, the mouse shown.
    let l = run_to("land", 770);
    let s = l.state(770);
    assert!(s.drawn.dissolve.is_none() && s.cursor.shown);
    assert!(!s.buttons.move_hero);
    // `land-walk`: three steps at sea, no landing until the coast.
    let l = run_to("land-walk", 700);
    assert!(l.aboard.is_none() && !l.hidden && l.lifted);
    let l = run_to("land-walk", 750);
    assert!(l.hidden && l.drawn().boats.empty[0].at == at(12));
    let l = run_to("land-walk", 1420);
    assert!(l.dissolving.is_none() && l.lifted && l.sailed().is_none());
}

#[test]
fn the_actions_given_meanwhile_wait_for_the_dissolves() {
    // A click during `land`'s dissolve waits for its end behind the key's release the walk
    // queued; during `board`'s, for the walk's queue at its end.
    let click = Action::Click([304, 280]);
    let mut l = run_to("land", 300);
    l.run(301, &[click]).unwrap();
    assert_eq!(l.dissolving.as_ref().unwrap().queued, [Action::Up, click]);
    let mut l = run_to("board", 300);
    l.run(301, &[click]).unwrap();
    assert_eq!(l.dissolving.as_ref().unwrap().queued, [Action::Up, click]);
    for t in 302..=620 {
        l.run(t, &[]).unwrap();
    }
    assert_eq!(l.moving.as_ref().unwrap().queued, [Action::Up, click]);
}

#[test]
fn an_action_refused_after_a_dissolve_is_the_runs_error() {
    // A click on a tile no path leads to, given during the dissolve: refused in a recording when
    // the walk's end takes it (a boarding's at speed 4 ends with its dissolve; a landing's).
    let click = Action::Click([100, 100]);
    let mut l = run_with("board", 300, |start| start.hero_speed = 4);
    l.run(301, &[click]).unwrap();
    let refused = (302..=620).find_map(|t| l.run(t, &[]).err());
    assert!(
        matches!(refused, Some(Unplayable::NoPath(_))),
        "{refused:?}"
    );
    let mut l = run_to("land", 300);
    l.run(301, &[click]).unwrap();
    let refused = (302..=770).find_map(|t| l.run(t, &[]).err());
    assert!(
        matches!(refused, Some(Unplayable::NoPath(_))),
        "{refused:?}"
    );
}

/// The step sounds and the effects a sequence gives, its start with the map's sound (the inlet's
/// water, sand around it).
fn sounds(id: &str, until: u32) -> Vec<String> {
    let mut l = run_with(id, until, |start| {
        let tiles = crate::sound::tests::tiles(
            36,
            |x, y| {
                let [x, y] = [x, y].map(|v| usize::try_from(v).unwrap());
                if water(x, y) { 8 } else { 1 }
            },
            |_, _| None,
        );
        start.sound = Some(crate::sound::tests::sound(tiles));
    });
    l.take_cues()
        .iter()
        .filter(|(_, c)| matches!(c, Cue::Step(_) | Cue::StepStop | Cue::Effect(_)))
        .map(|(t, c)| crate::sound::line(*t, c))
        .collect()
}

#[test]
fn killfade_at_each_dissolve_and_the_step_sound_of_the_tile_stepped_on() {
    // `board`: killfade at the boarding's dissolve; at the step's end the water's step sound
    // started and stopped in the same millisecond.
    assert_eq!(
        sounds("board", 1000),
        [
            "100 effect killfade.wav",
            "770 step horse08.wav",
            "770 step-stop"
        ]
    );
    // `land-walk`: the walk's water from 300, stopped by the landing's start at 750; the sand's at
    // the step's end, stopped, then killfade.
    assert_eq!(
        sounds("land-walk", 1500),
        [
            "300 step horse08.wav",
            "750 step-stop",
            "900 step horse01.wav",
            "900 step-stop",
            "900 effect killfade.wav"
        ]
    );
}

#[test]
fn a_boat_by_a_hero_in_a_boat_or_water_unknown_is_neither_boarded_nor_left() {
    // A hero in a boat does not board another; with no water known nothing is land.
    let mut l = run_to("land", 1);
    l.boats.empty.push(LiveBoat {
        at: at(10),
        kind: 0,
        facing: Facing::Up,
    });
    assert_eq!(l.boat_on(at(10)), None);
    l.water = Arc::default();
    assert!(l.water(at(8)));
    assert_eq!(l.landing(at(8), Direction::West), None);
    // No hero selected: none sails, none is set sailing, none moored.
    l.index = None;
    assert_eq!(l.sailed(), None);
    let boats = l.boats.clone();
    l.sail(Some(RED_BOAT));
    assert_eq!(l.boats, boats);
    l.kingdom.selected = Selected::None;
    let kingdom = l.kingdom.clone();
    l.moored();
    assert_eq!(l.kingdom, kingdom);
    // A hero on land lands nowhere; a tile past the water given is water.
    let l = run_to("board", 1);
    assert_eq!(l.landing(at(9), Direction::East), None);
    assert!(l.water(Pos { x: 40, y: 40, z: 1 }));
    assert_eq!(l.boat_on(at(9)).map(|b| b.facing), Some(Facing::DownLeft));
}

/// A record's one-step walk onto `x` with its boat's fact `sea` (E24 M4 T4b2).
fn on_record(x: u8, sea: Option<Sea>) -> impl FnOnce(&mut Start) {
    move |start: &mut Start| {
        start.moves = Some(vec![Move {
            tiles: vec![at(x)],
            points: vec![Some(0)],
            revealed: vec![Vec::new()],
            sea,
            battle: None,
            boats: Vec::new(),
            stops: None,
        }]);
    }
}

#[test]
fn a_record_boards_and_lands_as_it_says() {
    // `board` on a record: its boarding takes the empty boat on the tile, of the record's type.
    let l = run_with("board", 100, on_record(9, Some(Sea::Board(2))));
    let Some(Aboard::Board(boat)) = l.aboard else {
        panic!("boarding: {:?}", l.aboard);
    };
    assert_eq!(
        (boat.at, boat.kind, boat.facing),
        (at(9), 2, Facing::DownLeft)
    );
    // A record that says none: the boat's tile stepped onto as any.
    let l = run_with("board", 100, on_record(9, None));
    assert!(l.aboard.is_none() && !l.hidden && l.dissolving.is_none());
    // A boarding with no empty boat on the tile: the step is off the record.
    let (mut start, ..) = boating("board");
    on_record(9, Some(Sea::Board(1)))(&mut start);
    start.boats.empty.clear();
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    for t in 1..100 {
        l.run(t, &[]).unwrap();
    }
    let step = [Action::Step(Direction::East)];
    assert_eq!(l.run(100, &step), Err(Unplayable::Moved([9, 12, 0])));
    // `land` on a record: the boat left on its water tile facing the record's way.
    let left = LiveBoat {
        at: at(9),
        kind: 0,
        facing: Facing::DownLeft,
    };
    let l = run_with("land", 100, on_record(8, Some(Sea::Land(left))));
    assert_eq!(l.aboard, Some(Aboard::Land));
    assert_eq!(l.drawn().boats.empty, [left]);
    // One that says none: no landing.
    let l = run_with("land", 100, on_record(8, None));
    assert!(l.aboard.is_none() && l.drawn().boats.empty.is_empty());
}

#[test]
fn a_record_s_boat_told_after_its_start_is_drawn_and_boarded() {
    // E24 M4 T6 (the reviewer's R3-3): `board` on a record whose start shows no boat, the walk
    // telling the boat (a reveal's or a built one): on the map at the walk's first step, before
    // its boarding there, which takes it. Without it the step is off the record.
    let boat = LiveBoat {
        at: at(9),
        kind: 1,
        facing: Facing::DownLeft,
    };
    let told = |boats: Vec<LiveBoat>| {
        move |start: &mut Start| {
            on_record(9, Some(Sea::Board(2)))(start);
            start.boats.empty.clear();
            if let Some(walk) = start.moves.as_mut().and_then(|m| m.first_mut()) {
                walk.boats = boats;
            }
        }
    };
    let l = run_with("board", 99, told(vec![boat]));
    assert!(l.boats.empty.is_empty());
    let l = run_with("board", 100, told(vec![boat]));
    let Some(Aboard::Board(boarded)) = l.aboard else {
        panic!("boarding: {:?}", l.aboard);
    };
    assert_eq!((boarded.at, boarded.kind), (at(9), 2));
    // The told boat on its tile while the boarding's dissolve runs, taken off at its end.
    assert_eq!(l.boats.empty, [boat]);
    let l = run_with("board", 770, told(vec![boat]));
    assert!(l.drawn().boats.empty.is_empty());
    assert_eq!(l.drawn().boats.sailed[0].map(|b| b.kind), Some(2));
    // One told on a tile with an empty boat replaces it.
    let mut l = run_with("board", 1, |_| {});
    let other = LiveBoat {
        at: at(30),
        kind: 0,
        facing: Facing::Right,
    };
    let before = l.boats.empty.clone();
    l.put_boats(&[LiveBoat { kind: 2, ..boat }, other]);
    assert_eq!(l.boats.empty.len(), before.len() + 1);
    assert!(l.boats.empty.contains(&LiveBoat { kind: 2, ..boat }));
    assert!(!l.boats.empty.contains(&boat) && l.boats.empty.contains(&other));
    // None told: off the record.
    let (mut start, ..) = boating("board");
    told(Vec::new())(&mut start);
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    for t in 1..100 {
        l.run(t, &[]).unwrap();
    }
    let step = [Action::Step(Direction::East)];
    assert_eq!(l.run(100, &step), Err(Unplayable::Moved([9, 12, 0])));
}

#[test]
fn a_record_s_landing_with_no_facing_is_refused() {
    // E24 M4 T6 (the reviewer's R3-6): a `Landed` with a facing past 7 has no boat to leave: the
    // step refused, where it was played as a plain step with the hero left in its boat.
    let (mut start, actions, ..) = boating("land");
    on_record(8, Some(Sea::Unfaced(9)))(&mut start);
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    let given = |t: u32| actions.get(&t).map_or(&[][..], Vec::as_slice);
    for t in 1..100 {
        l.run(t, given(t)).unwrap();
    }
    assert_eq!(l.run(100, given(100)), Err(Unplayable::Landing([8, 12, 0])));
    assert_eq!(
        Unplayable::Landing([8, 12, 0]).to_string(),
        "a landing onto (8, 12, 0) whose boat's facing the record gives past 7"
    );
}

#[test]
fn a_dissolve_shows_the_date_window_its_draw_stepped() {
    // `land` with the date's window asked: the landing's dissolve at 250 shows the window the
    // view's draw stepped at once, outside the middle (E24 M4 T4b2, `game-boats` at 4250).
    let (start, actions, _, _) = boating("land");
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    let run = |l: &mut Loop, from: u32, to: u32| {
        for t in from..=to {
            l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
                .unwrap();
        }
    };
    run(&mut l, 1, 99);
    l.asked = 1;
    l.end = 10_000;
    l.kingdom.status = Some(Status::Date(DateStatus { frame: 0 }));
    // Built at the step's start (100) and stepped at 200; its last step put back to 140, so that
    // the landing's draw at 250 steps it.
    run(&mut l, 100, 249);
    l.stepped = 140;
    let shown = l.drawn().kingdom.status.clone();
    run(&mut l, 250, 250);
    let s = l.state(250);
    let before = s.drawn.dissolve.unwrap().before;
    let Some(Status::Date(date)) = &s.drawn.kingdom.status else {
        panic!("the date's window: {:?}", s.drawn.kingdom.status);
    };
    assert_eq!(*date, DateStatus { frame: 2 });
    assert_ne!(shown, s.drawn.kingdom.status);
    assert_eq!(before.kingdom.status, s.drawn.kingdom.status);
}
