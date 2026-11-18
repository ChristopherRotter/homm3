//! A level-up's choice (E24 M4 T2c, design §5.5): the five recorded sequences replayed headless;
//! the dialog after the Learning Stone's box and after a chest's dissolve, the next level's at the
//! first's close; its offers chosen by a click, a key or at once; its OK by Enter, Escape or a
//! click; what the levels raise shown at the last close, the mouse looked at again; a level whose
//! facts are not given, a hero whose experience is not known.

use super::*;

use h3_ui::message::{OK, Style};
use h3_ui::ui::State as Marked;

use super::choices::chosen;
use super::computers::hover_of;
use super::turns::{laid, started};
use crate::levels::{self, FRAMES, LevelUp, Offer};
use crate::sound::Cue;

/// E24 M4 T2c's sequences: their situations, timelines and kept live parts.
const LEVELLING: [(&str, &str, &str, &str); 5] = [
    (
        "level-up",
        include_str!("../../../../../map/situations/seq-level-up.toml"),
        include_str!("../../../../../map/timelines/level-up.txt"),
        include_str!("../../../../../map/timelines/level-up-live.toml"),
    ),
    (
        "level-up-right",
        include_str!("../../../../../map/situations/seq-level-up.toml"),
        include_str!("../../../../../map/timelines/level-up-right.txt"),
        include_str!("../../../../../map/timelines/level-up-right-live.toml"),
    ),
    (
        "level-up-one",
        include_str!("../../../../../map/situations/seq-level-up-one.toml"),
        include_str!("../../../../../map/timelines/level-up-one.txt"),
        include_str!("../../../../../map/timelines/level-up-one-live.toml"),
    ),
    (
        "level-up-none",
        include_str!("../../../../../map/situations/seq-level-up-none.toml"),
        include_str!("../../../../../map/timelines/level-up-none.txt"),
        include_str!("../../../../../map/timelines/level-up-none-live.toml"),
    ),
    (
        "level-up-twice",
        include_str!("../../../../../map/situations/seq-level-up-twice.toml"),
        include_str!("../../../../../map/timelines/level-up-twice.txt"),
        include_str!("../../../../../map/timelines/level-up-twice-live.toml"),
    ),
];

/// The Learning Stone's entrance, right of the hero's (4, 4); the chest's, above its (5, 5).
const STONE: Pos = Pos { x: 5, y: 4, z: 0 };

/// The chest (the map's fourth object: two heroes and a town before it).
const CHEST: usize = 3;

/// The recorded click on the left offer's picture.
const LEFT: [i32; 2] = [351, 413];

/// The dialogs of a live part's level-ups as the binary lays them out (texts aside).
fn dialogs(live: &str) -> Vec<LevelBox> {
    let table: toml::Table = live.parse().unwrap();
    let layout =
        AdventureLayout::parse(include_str!("../../../../../map/layouts/adventure.toml")).unwrap();
    LevelUp::read_all(&table["level_up"])
        .unwrap()
        .iter()
        .map(|up| boxed(&layout, up))
        .collect()
}

/// One level-up's dialog.
fn boxed(layout: &AdventureLayout, up: &LevelUp) -> LevelBox {
    LevelBox {
        level: up.level,
        dialog: Arc::new(levels::dialog(
            &layout.level_up,
            up,
            BTreeMap::new(),
            "hpl016rn.pcx",
            0,
        )),
        offers: up.offers.clone(),
        primary: up.primary,
    }
}

/// A sequence's start as the binary builds it: the stone's find (its box, its experience) or the
/// chest's choice (its experience at the dissolve's end), the hero's experience the situation's,
/// the level-ups' dialogs the live part's, the cursor after the walk the recording's.
fn levelling(id: &str) -> (Start, BTreeMap<u32, Vec<Action>>, u32, &'static str) {
    let entry = LEVELLING.iter().find(|(f, ..)| *f == id).unwrap();
    let (mut start, actions, ms, timeline) = started(entry);
    start.hover = Some(hover_of(entry.3));
    let sequences = Sequences::parse(SEQUENCES).unwrap();
    start.reached = sequences.get(id).unwrap().reached(timeline).unwrap();
    start.experience = Situation::parse(entry.1).unwrap().heroes[0].experience;
    start.level_ups = dialogs(entry.3);
    start.finds = vec![if id == "level-up-twice" {
        chest(1500)
    } else {
        stone()
    }];
    (start, actions, ms, timeline)
}

/// The Learning Stone's find: its box, its 1 000 experience.
fn stone() -> Found {
    Found {
        at: STONE,
        first: vec![(laid("You have learned much.", Style::Ok), Gain::default())],
        again: None,
        gain: Gain {
            experience: 1000,
            ..Gain::default()
        },
        found: None,
        taken: None,
        event: false,
        blocks: None,
        ask: None,
    }
}

/// A chest's choice from beside: its gold, or `experience` at the chest's dissolve's end.
fn chest(experience: i32) -> Found {
    let mut gold = Gain::default();
    gold.resources[6] = 2000;
    Found {
        at: STONE,
        first: Vec::new(),
        again: None,
        gain: Gain::default(),
        found: None,
        taken: None,
        event: false,
        blocks: Some(CHEST),
        ask: Some(Question {
            asked: chosen(Style::Choose, Some("or")),
            chooses: true,
            yes: Some(Answer {
                gain: gold,
                taken: Some((CHEST, Fade::Pickup)),
                ..Answer::default()
            }),
            no: Some(Answer::default()),
            attacker: None,
            second: Some(Answer {
                taken: Some((CHEST, Fade::Pickup)),
                after: Gain {
                    experience,
                    ..Gain::default()
                },
                ..Answer::default()
            }),
        }),
    }
}

/// The loop of `start` run to `until` with `actions`.
fn run(start: &Start, actions: &BTreeMap<u32, Vec<Action>>, until: u32) -> Loop {
    let mut l = Loop::start(rules(Bugs::Original), start).unwrap();
    for t in 1..=until {
        l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
            .unwrap();
    }
    l
}

/// The loop of a recorded level-up run to `until`.
fn level_to(id: &str, until: u32) -> Loop {
    let (start, actions, _, _) = levelling(id);
    run(&start, &actions, until)
}

/// The open dialog's level, its OK's state and its frames' (hidden); none without one.
fn open(l: &Loop) -> Option<(u8, Marked, [bool; 2])> {
    let boxed = l.boxed.as_ref()?;
    let Told::Level(leveling) = &boxed.told else {
        return None;
    };
    let shown = &boxed.open.message.shown;
    Some((
        leveling.open.level,
        shown.state(OK),
        FRAMES.map(|id| shown.state(id).hidden),
    ))
}

/// The OK disabled.
fn disabled() -> Marked {
    Marked {
        disabled: true,
        ..Marked::default()
    }
}

/// The selected hero's window: its primary skills and morale.
fn window(l: &Loop) -> ([i8; 4], i8, i8) {
    match &l.drawn().kingdom.status {
        Some(Status::Hero(hero)) => (hero.skills, hero.morale, hero.luck),
        other => panic!("no hero's window: {other:?}"),
    }
}

#[test]
fn replays_the_recorded_level_ups() {
    for (id, ..) in LEVELLING {
        let (start, actions, ms, timeline) = levelling(id);
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
fn the_stones_level_its_offers_by_a_click_and_enter() {
    let before = window(&level_to("level-up", 100));
    // The stone's box closed by Enter at 1060: the dialog in that millisecond with its sound,
    // the arrow, both frames hidden and the OK disabled; the map stood still (no idle step).
    let mut l = level_to("level-up", 1059);
    let tiles = crate::sound::tests::tiles(36, |_, _| 0, |_, _| None);
    l.sound = Some(crate::sound::tests::sound(tiles));
    let (_, actions, _, _) = levelling("level-up");
    l.run(1060, &actions[&1060]).unwrap();
    assert!(
        l.take_cues()
            .contains(&(1060, Cue::Effect("nwherolv.wav".into())))
    );
    assert_eq!(open(&l), Some((2, disabled(), [true, true])));
    assert_eq!((l.cursor().kind, l.cursor().frame), (CURSOR_TYPE, 0));
    // Enter at 1400 does nothing; the left picture clicked at 1800 shows its frame and enables
    // the OK, the dialog drawn again.
    let l = level_to("level-up", 1460);
    assert_eq!(open(&l), Some((2, disabled(), [true, true])));
    let l = level_to("level-up", 1800);
    assert_eq!(open(&l), Some((2, Marked::default(), [false, true])));
    assert_eq!(
        l.drawn().message.as_ref().unwrap().shown,
        l.boxed.as_ref().unwrap().open.message.shown
    );
    // Enter at 2200 presses the OK, its release at 2260 closes it: Defense +1 shown, the
    // cursor over the view the recording's (the move cursor), the idle step.
    assert!(
        level_to("level-up", 2200)
            .boxed
            .as_ref()
            .unwrap()
            .open
            .message
            .shown
            .state(OK)
            .pressed
    );
    let l = level_to("level-up", 2260);
    assert!(l.boxed.is_none());
    let (skills, ..) = window(&l);
    assert_eq!(
        skills,
        [before.0[0], before.0[1] + 1, before.0[2], before.0[3]]
    );
    assert_eq!((l.cursor().kind, l.cursor().frame), (CURSOR_TYPE, 4));
    assert_eq!(l.state(2260).objects, 2);
    // Visited again later: no level more.
    assert_eq!(l.hero_level, 2);
}

#[test]
fn the_offers_keys_and_the_ok_clicked() {
    // `level-up-right`: 2 at 1400 the right, 1 at 1600 the left, the right clicked at 1800.
    assert_eq!(
        open(&level_to("level-up-right", 1400)),
        Some((2, Marked::default(), [true, false]))
    );
    assert_eq!(
        open(&level_to("level-up-right", 1600)).unwrap().2,
        [false, true]
    );
    assert_eq!(
        open(&level_to("level-up-right", 1800)).unwrap().2,
        [true, false]
    );
    // The OK clicked at 2000: closed, the cursor where the poll last drew it until 2013.
    let l = level_to("level-up-right", 2000);
    assert!(l.boxed.is_none());
    assert_eq!(l.cursor().at, [448, 413]);
    assert_eq!(level_to("level-up-right", 2013).cursor().at, [532, 493]);
}

#[test]
fn one_offer_chosen_at_once_none_closed_by_escape() {
    // One: its frame shown, the OK enabled from the start; Enter closes it.
    let l = level_to("level-up-one", 1060);
    let (level, ok, hidden) = open(&l).unwrap();
    assert_eq!((level, ok, hidden[0]), (2, Marked::default(), false));
    assert!(level_to("level-up-one", 1660).boxed.is_none());
    // None: the key 1 does nothing; Escape presses the OK and its release closes it.
    let mut l = level_to("level-up-none", 1060);
    let shown = l.boxed.as_ref().unwrap().open.message.shown.clone();
    l.run(1061, &[Action::Offer(0), Action::Up]).unwrap();
    assert_eq!(l.boxed.as_ref().unwrap().open.message.shown, shown);
    let l = level_to("level-up-none", 1600);
    assert!(
        l.boxed
            .as_ref()
            .unwrap()
            .open
            .message
            .shown
            .state(OK)
            .pressed
    );
    assert!(level_to("level-up-none", 1660).boxed.is_none());
}

#[test]
fn a_chests_two_levels_after_its_dissolve() {
    let before = window(&level_to("level-up-twice", 100));
    // The experience chosen at 1460: the chest dissolved, then level 2's dialog at 1980.
    let l = level_to("level-up-twice", 1979);
    assert!(l.boxed.is_none() && l.drawn().dissolve.is_some());
    let l = level_to("level-up-twice", 1980);
    assert_eq!(open(&l), Some((2, disabled(), [true, true])));
    assert!(l.cursor().shown);
    assert_eq!(l.drawn().dissolve, None);
    // Its close at 4260 opens level 3's in that millisecond, the window as it was.
    let l = level_to("level-up-twice", 4260);
    assert_eq!(open(&l), Some((3, disabled(), [true, true])));
    assert_eq!(window(&l), before);
    // The last's close at 5260: Defense and Knowledge +1, the Advanced Leadership's morale +1;
    // the step's end (the move cursor), the idle step.
    let l = level_to("level-up-twice", 5260);
    assert!(l.boxed.is_none());
    let (skills, morale, luck) = window(&l);
    assert_eq!(
        skills,
        [before.0[0], before.0[1] + 1, before.0[2], before.0[3] + 1]
    );
    assert_eq!((morale, luck), (before.1 + 1, before.2));
    assert_eq!(l.cursor().frame, 4);
}

/// `level-up-twice` with the chest's experience as its answer's gain (Pandora's Box's way,
/// `grant_rewards`: the levels before the object's removal; this answer takes nothing), level 2
/// a Luck's.
fn answering() -> (Start, BTreeMap<u32, Vec<Action>>) {
    let (mut start, actions, _, _) = levelling("level-up-twice");
    let mut found = chest(0);
    let ask = found.ask.as_mut().unwrap();
    ask.second = Some(Answer {
        gain: Gain {
            experience: 1500,
            ..Gain::default()
        },
        ..Answer::default()
    });
    start.finds = vec![found];
    let layout =
        AdventureLayout::parse(include_str!("../../../../../map/layouts/adventure.toml")).unwrap();
    let luck = LevelUp {
        hero: 16,
        level: 2,
        primary: 0,
        offers: vec![Offer { skill: 9, level: 1 }],
    };
    start.level_ups = vec![boxed(&layout, &luck), start.level_ups[1].clone()];
    (start, actions)
}

#[test]
fn a_lucks_level_and_an_answers_levels() {
    // The levels at the answer's close; a Luck chosen shows a luck more.
    let (start, actions) = answering();
    let mut l = run(&start, &actions, 1460);
    assert_eq!(open(&l).map(|o| o.0), Some(2));
    let before = window(&l);
    // Enter: level 3's, then its left (Advanced Leadership) clicked and Enter.
    l.run(1461, &[Action::BoxKey(Key::Ok), Action::BoxKeyUp(Key::Ok)])
        .unwrap();
    assert_eq!(open(&l).map(|o| o.0), Some(3));
    l.run(1462, &[Action::Click(LEFT)]).unwrap();
    l.run(1463, &[Action::BoxKey(Key::Ok), Action::BoxKeyUp(Key::Ok)])
        .unwrap();
    assert!(l.boxed.is_none());
    let (skills, morale, luck) = window(&l);
    assert_eq!(skills[0], before.0[0] + 1);
    assert_eq!((morale, luck), (before.1 + 1, before.2 + 1));
    // The answer's end: the visit over, the chest left standing (this answer takes nothing).
    assert!(l.drawn().gone.is_empty());
}

#[test]
fn a_level_not_given_or_an_experience_not_known() {
    // Level 2's facts missing: refused in a recording, passed over live (the idle step).
    let (mut start, actions, _, _) = levelling("level-up");
    start.level_ups.clear();
    let mut l = run(&start, &actions, 1059);
    assert_eq!(l.run(1060, &actions[&1060]), Err(Unplayable::LevelUp(2)));
    assert_eq!(
        Unplayable::LevelUp(2).to_string(),
        "the level-up to level 2: its facts not given"
    );
    start.live = true;
    let l = run(&start, &actions, 1060);
    assert!(l.boxed.is_none());
    assert_eq!(l.state(1060).objects, 2);
    // At a chest's dissolve's end and at an answer's end alike.
    let (mut start, actions, _, _) = levelling("level-up-twice");
    start.level_ups.clear();
    let mut l = run(&start, &actions, 1979);
    assert_eq!(l.run(1980, &[]), Err(Unplayable::LevelUp(2)));
    let (mut start, actions) = answering();
    start.level_ups.clear();
    let mut l = run(&start, &actions, 1459);
    assert_eq!(l.run(1460, &actions[&1460]), Err(Unplayable::LevelUp(2)));
    // A primary skill past the four raises none.
    let (mut start, actions, _, _) = levelling("level-up-one");
    let before = window(&run(&start, &actions, 100));
    start.level_ups[0].primary = 4;
    assert_eq!(window(&run(&start, &actions, 1660)), before);
    // No experience known: no level.
    let (mut start, actions, _, _) = levelling("level-up");
    start.experience = None;
    assert!(run(&start, &actions, 1060).boxed.is_none());
    // Another hero's experience than the one known: none counted.
    let (start, actions, _, _) = levelling("level-up");
    let mut l = run(&start, &actions, 1059);
    l.experienced = None;
    l.run(1060, &actions[&1060]).unwrap();
    assert!(l.boxed.is_none());
    assert_eq!(l.experience, Some(0));
}

#[test]
fn the_dialogs_input_beside_its_offers() {
    let mut l = level_to("level-up", 1060);
    let shown = l.boxed.as_ref().unwrap().open.message.shown.clone();
    // A click on the background and a move, a right release, a third offer's key: nothing.
    l.run(
        1061,
        &[Action::Click([400, 120]), Action::Pointer([400, 130])],
    )
    .unwrap();
    l.run(1062, &[Action::RightRelease, Action::Offer(2)])
        .unwrap();
    assert_eq!(l.boxed.as_ref().unwrap().open.message.shown, shown);
    // Escape with no offer chosen: the OK disabled takes no key.
    l.run(1063, &[Action::BoxKey(Key::Cancel)]).unwrap();
    assert!(
        !l.boxed
            .as_ref()
            .unwrap()
            .open
            .message
            .shown
            .state(OK)
            .pressed
    );
    // A plain box takes no offer's key; with nothing open the key does nothing.
    let mut l = level_to("level-up", 500);
    let shown = l.boxed.as_ref().unwrap().open.message.shown.clone();
    l.run(501, &[Action::Offer(0)]).unwrap();
    assert_eq!(l.boxed.as_ref().unwrap().open.message.shown, shown);
    assert!(Action::Offer(1).ends_walk());
    let mut l = level_to("level-up", 50);
    l.run(51, &[Action::Offer(0)]).unwrap();
    assert!(l.boxed.is_none());
}
