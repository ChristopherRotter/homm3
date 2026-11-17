//! The finds told (E24 M4 T2a2, design §5.3): a step onto a find's entrance and its box in one
//! millisecond, Enter's OK, the map's idle step at its close; told again, the later box; a turn
//! without a town's two boxes.

use super::*;

use h3_ui::message::{OK, Style, TEXT};

use super::turns::{laid, started};

/// E24 M4 T2a2's finds: their situations, timelines and kept live parts.
const FINDING: [(&str, &str, &str, &str); 7] = [
    (
        "find-windmill",
        include_str!("../../../../../map/situations/seq-find-windmill.toml"),
        include_str!("../../../../../map/timelines/find-windmill.txt"),
        include_str!("../../../../../map/timelines/find-windmill-live.toml"),
    ),
    (
        "find-stone",
        include_str!("../../../../../map/situations/seq-find-stone.toml"),
        include_str!("../../../../../map/timelines/find-stone.txt"),
        include_str!("../../../../../map/timelines/find-stone-live.toml"),
    ),
    (
        "find-axis",
        include_str!("../../../../../map/situations/seq-find-axis.toml"),
        include_str!("../../../../../map/timelines/find-axis.txt"),
        include_str!("../../../../../map/timelines/find-axis-live.toml"),
    ),
    (
        "find-temple",
        include_str!("../../../../../map/situations/seq-find-temple.toml"),
        include_str!("../../../../../map/timelines/find-temple.txt"),
        include_str!("../../../../../map/timelines/find-temple-live.toml"),
    ),
    (
        "find-witch-hut",
        include_str!("../../../../../map/situations/seq-find-witch-hut.toml"),
        include_str!("../../../../../map/timelines/find-witch-hut.txt"),
        include_str!("../../../../../map/timelines/find-witch-hut-live.toml"),
    ),
    (
        "find-shrine",
        include_str!("../../../../../map/situations/seq-find-shrine.toml"),
        include_str!("../../../../../map/timelines/find-shrine.txt"),
        include_str!("../../../../../map/timelines/find-shrine-live.toml"),
    ),
    (
        "find-sign",
        include_str!("../../../../../map/situations/seq-find-sign.toml"),
        include_str!("../../../../../map/timelines/find-sign.txt"),
        include_str!("../../../../../map/timelines/find-sign-live.toml"),
    ),
];

/// Every find's entrance in the sequences: right of the hero's (4, 4).
const ENTRANCE: Pos = Pos { x: 5, y: 4, z: 0 };

/// A find sequence's start: a turn sequence's (the selected hero by the facts), its find at its
/// entrance (the boxes' texts the binary's; here two of their own).
fn finding(id: &str) -> (Start, BTreeMap<u32, Vec<Action>>, u32, &'static str) {
    let entry = FINDING.iter().find(|(f, ..)| *f == id).unwrap();
    let (mut start, actions, ms, timeline) = started(entry);
    start.finds = vec![Found {
        at: ENTRANCE,
        first: laid("Found.", Style::Ok),
        again: laid("Again.", Style::Ok),
        gain: Gain::default(),
    }];
    (start, actions, ms, timeline)
}

#[test]
fn replays_the_recorded_finds() {
    for (id, ..) in FINDING {
        let (start, actions, ms, timeline) = finding(id);
        let states = play(rules(Bugs::Original), &start, &actions, ms).unwrap();
        let got = lines(&states, &BTreeMap::new());
        let want = without_screens(timeline);
        for (n, (g, w)) in got.iter().zip(&want).enumerate() {
            assert_eq!(g, w, "{id}: line {n}");
        }
        assert_eq!(got.len(), want.len(), "{id}");
    }
}

/// The loop of a recorded find run to `until` with its actions.
fn run_to(id: &str, until: u32) -> Loop {
    let (start, actions, _, _) = finding(id);
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    for t in 1..=until {
        l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
            .unwrap();
    }
    l
}

/// The open box's text.
fn text(l: &Loop) -> Option<String> {
    l.drawn()
        .message
        .as_ref()
        .map(|m| m.shown.texts[&TEXT].clone())
}

#[test]
fn a_step_onto_a_find_tells_it_and_its_box_closes_to_the_idle_step() {
    // `find-windmill`: the step at 100 (speed 4: one draw) and the box in that millisecond, the
    // default cursor; the map stands still behind it.
    let l = run_to("find-windmill", 100);
    let s = l.state(100);
    assert_eq!(s.hero.map(|h| h.at), Some(ENTRANCE));
    assert_eq!(text(&l).as_deref(), Some("Found."));
    assert_eq!((s.cursor.kind, s.cursor.frame), (DEFAULT_CURSOR, 0));
    let l = run_to("find-windmill", 999);
    assert_eq!(l.state(999).objects, 0);
    // Enter's press draws OK pressed; its release closes the box, the redraws due since run
    // (two at 1060), the adventure cursor again.
    let l = run_to("find-windmill", 1000);
    assert!(
        l.drawn()
            .message
            .as_ref()
            .is_some_and(|m| m.shown.state(OK).pressed)
    );
    let mut l = run_to("find-windmill", 1060);
    let s = l.state(1060);
    assert!(s.drawn.message.is_none() && l.boxed.is_none());
    assert_eq!((s.objects, s.cursor.kind), (2, CURSOR_TYPE));
    // Back and onto it again: the later box (the find as its visit left it).
    l.run(1061, &[Action::Step(Direction::West)]).unwrap();
    assert!(text(&l).is_none());
    l.run(1062, &[Action::Step(Direction::East)]).unwrap();
    assert_eq!(text(&l).as_deref(), Some("Again."));
}

#[test]
fn no_find_no_box() {
    // A step onto a tile with no find, and a loop without finds, open none.
    let (mut start, actions, _, _) = finding("find-stone");
    start.finds[0].at = Pos { x: 9, y: 9, z: 0 };
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    l.run(100, &actions[&100]).unwrap();
    assert!(l.boxed.is_none() && text(&l).is_none());
    start.finds.clear();
    start.hero = None;
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    l.run(100, &actions[&100]).unwrap();
    assert!(l.boxed.is_none());
    assert!(!l.tell_find());
}

/// `turn-no-town`'s loop run to `until`.
fn town_to(until: u32) -> Loop {
    let (start, actions, _, _) = started(&TOWNLESS);
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    for t in 1..=until {
        l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
            .unwrap();
    }
    l
}

/// `turn-no-town`'s entry.
const TOWNLESS: (&str, &str, &str, &str) = (
    "turn-no-town",
    include_str!("../../../../../map/situations/seq-turn-no-town.toml"),
    include_str!("../../../../../map/timelines/turn-no-town.txt"),
    include_str!("../../../../../map/timelines/turn-no-town-live.toml"),
);

#[test]
fn a_turn_without_a_town_warns_then_tells_the_days_left() {
    // End Turn's release at 160: the warning, the days counted from 7.
    let l = town_to(160);
    assert_eq!(
        text(&l).as_deref(),
        Some("Red, you have lost your last town.")
    );
    assert_eq!((l.days, l.state(160).date.day), (7, 1));
    // Enter's release at 1060 closes it: the turn's end counts down, the next day's start tells
    // six days over its whole redraw.
    let mut l = town_to(1060);
    assert_eq!((l.days, l.state(1060).date.day), (6, 2));
    assert_eq!(text(&l).as_deref(), Some("6 days left"));
    let l2 = town_to(2060);
    assert!(l2.boxed.is_none());
    // Without the boxes no test; the days down to their last: the next End Turn refused (the
    // elimination is not built).
    l.days = 1;
    assert_eq!(
        l.town_test(),
        Err(Unplayable::Command(panels::id::END_TURN))
    );
    let mut ended = town_to(160);
    ended.days = 1;
    assert_eq!(
        ended.turn_change(161),
        Err(Unplayable::Command(panels::id::END_TURN))
    );
    l.days = 3;
    assert_eq!(l.town_test(), Ok(None));
    l.town = None;
    assert_eq!(l.town_test(), Ok(None));
}

#[test]
fn the_days_left_then_the_weeks_message_one_after_the_other() {
    // A week's first day without a town: the days left's box, then at its close the week's.
    let (mut start, ..) = started(&TOWNLESS);
    let mut turn = start.turns[0].turn.clone();
    turn.kingdom.date = Date {
        month: 1,
        week: 2,
        day: 1,
    };
    start.turns[0] = NextTurn {
        message: Some(laid("Week of the Rat.", Style::Ok)),
        turn,
        heroes: Vec::new(),
        walks: Vec::new(),
    };
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    l.days = 3;
    l.run(100, &[Action::Key(Hotkey::EndTurn)]).unwrap();
    l.run(160, &[Action::KeyUp(Hotkey::EndTurn)]).unwrap();
    assert_eq!(text(&l).as_deref(), Some("2 days left"));
    l.run(200, &[Action::BoxKey(Key::Ok)]).unwrap();
    l.run(260, &[Action::BoxKeyUp(Key::Ok)]).unwrap();
    assert_eq!(text(&l).as_deref(), Some("Week of the Rat."));
    assert!(l.boxed.as_ref().is_some_and(|b| b.told == Told::Week));
}

#[test]
fn a_finds_gain_shows_at_its_box_close() {
    // `find-windmill`'s Mercury 4, with Spell Power and morale besides: the resource bar and the
    // hero's window as the box closes (1060), not before.
    let (mut start, actions, _, _) = finding("find-windmill");
    start.finds[0].gain = Gain {
        resource: Some((1, 4)),
        skills: [0, 0, 1, 0],
        morale: [1, 2],
    };
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    let hero = |l: &Loop| match &l.drawn().kingdom.status {
        Some(Status::Hero(h)) => (h.skills, h.morale),
        other => panic!("{other:?}"),
    };
    let before = hero(&l);
    for t in 1..=1000 {
        l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
            .unwrap();
    }
    assert_eq!(l.drawn().kingdom.resources[1], 10);
    assert_eq!(hero(&l), before);
    l.run(1060, &actions[&1060]).unwrap();
    assert_eq!(l.drawn().kingdom.resources[1], 14);
    let (skills, morale) = hero(&l);
    assert_eq!((skills[2], morale), (before.0[2] + 1, before.1 + 1));
    let chosen = l.chosen.heroes[0].as_ref().unwrap();
    assert_eq!(chosen.status.skills[2], before.0[2] + 1);
    // On the week's seventh day the Temple's two; morale at most 3, in the window kept as well;
    // a gain of nothing shows none.
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    l.kingdom.date.day = 7;
    l.kingdom.selection.clone_from(&l.kingdom.status);
    l.chosen.heroes.clear();
    l.gain(Gain {
        morale: [1, 5],
        ..Gain::default()
    });
    assert_eq!(hero(&l).1, 3);
    assert!(matches!(&l.kingdom.selection, Some(Status::Hero(h)) if h.morale == 3));
    let drawn = l.drawn().clone();
    l.gain(Gain {
        resource: Some((9, 1)),
        ..Gain::default()
    });
    assert_eq!(*l.drawn(), drawn);
}
