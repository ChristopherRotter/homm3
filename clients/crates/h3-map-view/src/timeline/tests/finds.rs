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
        first: vec![(laid("Found.", Style::Ok), Gain::default())],
        again: Some(laid("Again.", Style::Ok)),
        gain: Gain::default(),
        found: None,
        taken: None,
        event: false,
        blocks: None,
        ask: None,
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
    assert!(!l.tell_find(10));
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
        resources: [0, 4, 0, 0, 0, 0, 0],
        skills: [0, 0, 1, 0],
        morale: [1, 2],
        ..Gain::default()
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
    l.gain(Gain::default());
    assert_eq!(*l.drawn(), drawn);
}

/// Part B's sequences (E24 M4 T2a2): a pile, an artifact and an event.
const PICKING: [(&str, &str, &str, &str); 3] = [
    (
        "find-pile",
        include_str!("../../../../../map/situations/seq-find-pile.toml"),
        include_str!("../../../../../map/timelines/find-pile.txt"),
        include_str!("../../../../../map/timelines/find-pile-live.toml"),
    ),
    (
        "find-artifact",
        include_str!("../../../../../map/situations/seq-find-artifact.toml"),
        include_str!("../../../../../map/timelines/find-artifact.txt"),
        include_str!("../../../../../map/timelines/find-artifact-live.toml"),
    ),
    (
        "find-event",
        include_str!("../../../../../map/situations/seq-find-event.toml"),
        include_str!("../../../../../map/timelines/find-event.txt"),
        include_str!("../../../../../map/timelines/find-event-live.toml"),
    ),
];

/// The object a part B sequence takes (the map's fourth: two heroes and a town before it).
const OBJECT: usize = 3;

/// Gold, the resource bar's seventh.
fn gold(amount: i32) -> [i32; 7] {
    [0, 0, 0, 0, 0, 0, amount]
}

/// A part B sequence's start, its find as the binary builds it (the boxes' texts here their own).
fn picking(id: &str) -> (Start, BTreeMap<u32, Vec<Action>>, u32, &'static str) {
    let entry = PICKING.iter().find(|(f, ..)| *f == id).unwrap();
    let (mut start, actions, ms, timeline) = started(entry);
    let found = match id {
        "find-pile" => Found {
            at: ENTRANCE,
            first: Vec::new(),
            again: None,
            gain: Gain {
                resources: gold(70_000),
                ..Gain::default()
            },
            found: Some(FoundStatus {
                resource: 6,
                amount: 70_000,
                text: "Gold.".into(),
            }),
            taken: Some(OBJECT),
            event: false,
            blocks: Some(OBJECT),
            ask: None,
        },
        "find-artifact" => Found {
            at: ENTRANCE,
            first: vec![(laid("Axe.", Style::Ok), Gain::default())],
            again: None,
            gain: Gain {
                skills: [2, 0, 0, 0],
                ..Gain::default()
            },
            found: None,
            taken: Some(OBJECT),
            event: false,
            blocks: Some(OBJECT),
            ask: None,
        },
        _ => Found {
            at: ENTRANCE,
            first: vec![
                (laid("Mana.", Style::Ok), Gain::default()),
                (
                    laid("Gold.", Style::Ok),
                    Gain {
                        resources: gold(500),
                        ..Gain::default()
                    },
                ),
                (laid("Halberdiers.", Style::Ok), Gain::default()),
            ],
            again: None,
            gain: Gain {
                luck: 1,
                mana: 5,
                army: [
                    Some(Stack {
                        creature: 1,
                        count: 5,
                    }),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ],
                ..Gain::default()
            },
            found: None,
            taken: None,
            event: true,
            blocks: None,
            ask: None,
        },
    };
    start.finds = vec![found];
    (start, actions, ms, timeline)
}

#[test]
fn replays_the_recorded_pickups() {
    for (id, ..) in PICKING {
        let (start, actions, ms, timeline) = picking(id);
        let states = play(rules(Bugs::Original), &start, &actions, ms).unwrap();
        let got = lines(&states, &BTreeMap::new());
        let want = without_screens(timeline);
        for (n, (g, w)) in got.iter().zip(&want).enumerate() {
            assert_eq!(g, w, "{id}: line {n}");
        }
        assert_eq!(got.len(), want.len(), "{id}");
    }
}

/// A part B sequence's loop run to `until` with its actions and `more`.
fn pick_to(id: &str, until: u32, more: &[(u32, Action)]) -> Loop {
    let (start, actions, _, _) = picking(id);
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    for t in 1..=until {
        let mut given = actions.get(&t).cloned().unwrap_or_default();
        given.extend(more.iter().filter(|(at, _)| *at == t).map(|&(_, a)| a));
        l.run(t, &given).unwrap();
    }
    l
}

/// The status window's kind as drawn.
fn kind(l: &Loop) -> Option<u8> {
    l.drawn().kingdom.status.as_ref().map(Status::kind)
}

#[test]
fn a_pile_from_beside_its_kind_6_and_its_dissolve() {
    // `find-pile`: the step at 100 leaves the hero on (4, 4); the gold on the resource bar, kind
    // 6 asked until 5 100, the pile gone with the dissolve's first step showing it still.
    let l = pick_to("find-pile", 100, &[]);
    let s = l.state(100);
    assert_eq!(s.hero.map(|h| h.at), Some(Pos { x: 4, y: 4, z: 0 }));
    assert_eq!(s.drawn.kingdom.resources[6], 90_000);
    assert_eq!((kind(&l), l.asked, l.end), (Some(6), 6, 5100));
    assert_eq!(*s.drawn.gone, [OBJECT]);
    let dissolve = s.drawn.dissolve.clone().unwrap();
    assert_eq!((dissolve.step, dissolve.rect), (0, [192, 160, 416, 384]));
    assert!(dissolve.before.gone.is_empty() && !s.cursor.shown);
    // A step each 65 ms, the palettes held (their deadline moving on), the map stopped.
    let l = pick_to("find-pile", 555, &[]);
    assert_eq!(l.drawn().dissolve.as_ref().map(|d| d.step), Some(7));
    assert_eq!((l.state(555).cycles, l.state(555).objects), (0, 0));
    let l = pick_to("find-pile", 620, &[]);
    let s = l.state(620);
    assert!(s.drawn.dissolve.is_none() && s.cursor.shown && s.objects == 2);
    assert_eq!(pick_to("find-pile", 800, &[]).state(800).cycles, 1);
    // Kind 6 until its end, the hero's window at the next draw (5 120).
    assert_eq!(kind(&pick_to("find-pile", 5100, &[])), Some(6));
    assert_eq!(kind(&pick_to("find-pile", 5120, &[])), Some(3));
}

#[test]
fn the_actions_given_during_a_dissolve_wait_for_its_end() {
    // A step west given at 300: taken at 620, the hero then on (3, 4); a pointer's move read at
    // once. The pile's tile then free: a step east goes onto it, no second visit.
    let left = Action::Step(Direction::West);
    let l = pick_to(
        "find-pile",
        619,
        &[(300, left), (301, Action::Pointer([650, 300]))],
    );
    assert_eq!(l.state(619).hero.map(|h| h.at.x), Some(4));
    assert_eq!(l.pointer, [650, 300]);
    let mut l = pick_to("find-pile", 620, &[(300, left)]);
    assert_eq!(l.state(620).hero.map(|h| h.at.x), Some(3));
    l.run(700, &[Action::Step(Direction::East)]).unwrap();
    l.run(701, &[Action::Step(Direction::East)]).unwrap();
    assert_eq!(l.state(701).hero.map(|h| h.at.x), Some(5));
    assert!(l.dissolving.is_none() && l.boxed.is_none());
    // Over the status window kind 6 asked gives the selection's at a click.
    let mut l = pick_to("find-pile", 1000, &[]);
    l.status_click(1000);
    assert_eq!((l.asked, kind(&l)), (0, Some(3)));
}

#[test]
fn an_artifacts_box_then_its_dissolve_and_its_skills() {
    // `find-artifact`: the box at 100 over the mouse shown; its close at 1060 the dissolve's
    // first step, the mouse hidden; at its end (1580) the Attack +2 in the hero's window.
    let attack = |l: &Loop| match &l.drawn().kingdom.status {
        Some(Status::Hero(h)) => h.skills[0],
        other => panic!("{other:?}"),
    };
    let l = pick_to("find-artifact", 100, &[]);
    assert_eq!(text(&l).as_deref(), Some("Axe."));
    assert!(l.state(100).cursor.shown);
    let before = attack(&l);
    let l = pick_to("find-artifact", 1060, &[]);
    assert!(text(&l).is_none() && !l.state(1060).cursor.shown);
    assert_eq!(l.drawn().dissolve.as_ref().map(|d| d.step), Some(0));
    assert_eq!(attack(&l), before);
    let l = pick_to("find-artifact", 1580, &[]);
    assert_eq!(attack(&l), before + 2);
    assert!(l.drawn().dissolve.is_none() && *l.drawn().gone == [OBJECT]);
}

#[test]
fn an_events_boxes_one_after_another() {
    // `find-event`: three boxes; the gold as the second opens (1060); the slot's points, the
    // luck, the spell points and the creatures after the last (2260).
    let points = |l: &Loop| l.drawn().kingdom.heroes[0].points;
    let hero = |l: &Loop| match &l.drawn().kingdom.status {
        Some(Status::Hero(h)) => (h.luck, h.army[1]),
        other => panic!("{other:?}"),
    };
    let l = pick_to("find-event", 100, &[]);
    assert_eq!(text(&l).as_deref(), Some("Mana."));
    assert_eq!(points(&l), 1700);
    let l = pick_to("find-event", 1060, &[]);
    assert_eq!(text(&l).as_deref(), Some("Gold."));
    assert_eq!(l.drawn().kingdom.resources[6], 20_500);
    let l = pick_to("find-event", 1660, &[]);
    assert_eq!(text(&l).as_deref(), Some("Halberdiers."));
    let mana = l.drawn().kingdom.heroes[0].mana;
    let l = pick_to("find-event", 2260, &[]);
    assert!(text(&l).is_none());
    assert_eq!(points(&l), 1600);
    assert_eq!(l.drawn().kingdom.heroes[0].mana, mana + 5);
    assert_eq!(
        hero(&l),
        (
            1,
            Some(Stack {
                creature: 1,
                count: 5
            })
        )
    );
}

#[test]
fn an_event_without_a_town_tests_the_scenario() {
    // With heroes and no town the scenario's test after the boxes warns (its box closing to the
    // idle step).
    let (mut start, actions, _, _) = picking("find-event");
    start.kingdom.towns.clear();
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    for t in 1..=2260 {
        l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
            .unwrap();
    }
    assert_eq!(
        text(&l).as_deref(),
        Some("Red, you have lost your last town.")
    );
    assert_eq!(l.days, 7);
    l.run(2300, &[Action::BoxKey(Key::Ok)]).unwrap();
    l.run(2360, &[Action::BoxKeyUp(Key::Ok)]).unwrap();
    assert!(l.boxed.is_none());
}

#[test]
fn a_taken_find_is_no_walks_and_an_army_has_room() {
    // A walk's end on a pile's tile tells nothing (only a key's step visits it from beside); a
    // box of a find past the finds closes to the idle step; no dissolve, its turn over.
    let (start, ..) = picking("find-pile");
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    l.hero = Some(Hero {
        at: ENTRANCE,
        facing: Direction::East,
    });
    assert!(!l.tell_find(10));
    assert_eq!(l.found_closed(10, 5, 0), Ok(()));
    assert!(l.dissolve_step(10));
    // A gain of spell points with the hero not on the lists as drawn: memory's alone.
    let hero = l.selected_hero().unwrap();
    let mana = l.kingdom.heroes[hero].mana;
    Arc::make_mut(&mut l.drawn.kingdom).heroes.clear();
    l.gain(Gain {
        mana: 2,
        ..Gain::default()
    });
    assert_eq!(l.kingdom.heroes[hero].mana, mana + 2);
    // Kind 6 asked without its facts builds nothing.
    let shown = l.drawn().kingdom.status.clone();
    (l.asked, l.end) = (6, 100);
    l.choose(10, true);
    assert_eq!(l.drawn().kingdom.status, shown);
    // The pickup's sound as the object is taken (the viewer's first).
    let tiles = crate::sound::tests::tiles(36, |_, _| 0, |_, _| None);
    l.sound = Some(crate::sound::tests::sound(tiles));
    l.take(20, OBJECT, Gain::default());
    assert!(
        l.take_cues()
            .contains(&(20, Cue::Effect("pickup01.wav".into())))
    );
    // An event told, or one with no box: nothing again.
    let (mut start, ..) = picking("find-event");
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    l.hero = Some(Hero {
        at: ENTRANCE,
        facing: Direction::East,
    });
    l.told.push(0);
    assert!(!l.tell_find(10));
    start.finds[0].first.clear();
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    l.hero = Some(Hero {
        at: ENTRANCE,
        facing: Direction::East,
    });
    assert!(!l.tell_find(10));
    // The mouse's point of the actions that give one.
    for (action, point) in [
        (Action::Pointer([1, 2]), Some([1, 2])),
        (Action::Click([3, 4]), Some([3, 4])),
        (Action::Press([5, 6]), Some([5, 6])),
        (Action::Release([7, 8]), Some([7, 8])),
        (Action::Visit, None),
    ] {
        assert_eq!(action.point(), point);
    }
    // The creatures join their kind's slot, else the first empty one; none with no room.
    let griffins = Stack {
        creature: 4,
        count: 2,
    };
    let mut army = [Some(griffins); 7];
    joined(&mut army, griffins);
    assert_eq!(army[0].map(|s| s.count), Some(4));
    joined(
        &mut army,
        Stack {
            creature: 5,
            count: 1,
        },
    );
    assert!(army.iter().flatten().all(|s| s.creature == 4));
    army[3] = None;
    joined(
        &mut army,
        Stack {
            creature: 5,
            count: 1,
        },
    );
    assert_eq!(army[3].map(|s| s.creature), Some(5));
}

#[test]
fn a_pickup_lifted_already_without_kind_6_and_the_refusals_after() {
    // The hero lifted before the step (no lift again); a taken find with no box and no kind 6
    // only dissolves.
    let (mut start, ..) = picking("find-pile");
    start.finds[0].found = None;
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    l.lifted = true;
    l.run(100, &[Action::Step(Direction::East)]).unwrap();
    assert_eq!(
        (l.asked, l.drawn().dissolve.as_ref().map(|d| d.step)),
        (0, Some(0))
    );
    // An action the loop cannot play, given during the dissolve, refused at its end.
    let (start, ..) = picking("find-pile");
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    l.run(100, &[Action::Step(Direction::East)]).unwrap();
    l.run(300, &[Action::Click([300, 300])]).unwrap();
    assert!(l.run(620, &[]).is_err());
    // The scenario's test after an event refused with the days down to their last.
    let (mut start, actions, _, _) = picking("find-event");
    start.kingdom.towns.clear();
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    l.days = 1;
    for t in 1..2260 {
        l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
            .unwrap();
    }
    assert!(l.run(2260, &actions[&2260]).is_err());
}
