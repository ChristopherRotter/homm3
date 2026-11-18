//! The questions (E24 M4 T2b2 part A1, design §5.4): a question on a find's entrance or from
//! beside, in the step's millisecond; Escape's Cancel ends the visit, Enter's OK shows the answer's
//! box and, at its close, what the panels show; an answer not built refused.

use super::*;

use h3_ui::message::{Key, NO, OK, Style, TEXT, YES};

use super::turns::{laid, started};

/// E24 M4 T2b1's sequences this part plays: their situations, timelines and kept live parts.
const ASKING: [(&str, &str, &str, &str); 10] = [
    (
        "ask-pile",
        include_str!("../../../../../map/situations/seq-ask-pile.toml"),
        include_str!("../../../../../map/timelines/ask-pile.txt"),
        include_str!("../../../../../map/timelines/ask-pile-live.toml"),
    ),
    (
        "ask-artifact",
        include_str!("../../../../../map/situations/seq-ask-artifact.toml"),
        include_str!("../../../../../map/timelines/ask-artifact.txt"),
        include_str!("../../../../../map/timelines/ask-artifact-live.toml"),
    ),
    (
        "ask-mine",
        include_str!("../../../../../map/situations/seq-ask-mine.toml"),
        include_str!("../../../../../map/timelines/ask-mine.txt"),
        include_str!("../../../../../map/timelines/ask-mine-live.toml"),
    ),
    (
        "ask-bank",
        include_str!("../../../../../map/situations/seq-ask-bank.toml"),
        include_str!("../../../../../map/timelines/ask-bank.txt"),
        include_str!("../../../../../map/timelines/ask-bank-live.toml"),
    ),
    (
        "ask-crypt",
        include_str!("../../../../../map/situations/seq-ask-crypt.toml"),
        include_str!("../../../../../map/timelines/ask-crypt.txt"),
        include_str!("../../../../../map/timelines/ask-crypt-live.toml"),
    ),
    (
        "ask-tree",
        include_str!("../../../../../map/situations/seq-ask-tree.toml"),
        include_str!("../../../../../map/timelines/ask-tree.txt"),
        include_str!("../../../../../map/timelines/ask-tree-live.toml"),
    ),
    (
        "ask-pyramid",
        include_str!("../../../../../map/situations/seq-ask-pyramid.toml"),
        include_str!("../../../../../map/timelines/ask-pyramid.txt"),
        include_str!("../../../../../map/timelines/ask-pyramid-live.toml"),
    ),
    (
        "ask-dwelling-guards",
        include_str!("../../../../../map/situations/seq-ask-dwelling-guards.toml"),
        include_str!("../../../../../map/timelines/ask-dwelling-guards.txt"),
        include_str!("../../../../../map/timelines/ask-dwelling-guards-live.toml"),
    ),
    (
        "ask-tomb",
        include_str!("../../../../../map/situations/seq-ask-tomb.toml"),
        include_str!("../../../../../map/timelines/ask-tomb.txt"),
        include_str!("../../../../../map/timelines/ask-tomb-live.toml"),
    ),
    (
        "ask-hire",
        include_str!("../../../../../map/situations/seq-ask-hire.toml"),
        include_str!("../../../../../map/timelines/ask-hire.txt"),
        include_str!("../../../../../map/timelines/ask-hire-live.toml"),
    ),
];

/// Every question's entrance: above the hero's (5, 5).
const ENTRANCE: Pos = Pos { x: 5, y: 4, z: 0 };

/// The object asked about (the map's fourth: two heroes and a town before it).
const OBJECT: usize = 3;

/// The Pikemen joining at `ask-hire`'s OK (the Guardhouse's week).
const PIKEMEN: Stack = Stack {
    creature: 0,
    count: 14,
};

/// A sequence's question as the binary builds it (the boxes' texts here their own): from beside
/// for a pile and an artifact, the status window chosen first but for the Crypt, the Tree and the
/// Tomb; the Tomb's and the hire's OK shown, the rest not built.
fn question(id: &str) -> Found {
    let yes = match id {
        "ask-tomb" => Some(Answer {
            boxes: vec![(laid("Gloves.", Style::Ok), Gain::default())],
            gain: Gain {
                morale: [-3, -3],
                ..Gain::default()
            },
        }),
        "ask-hire" => Some(Answer {
            boxes: vec![(laid("Pikemen join.", Style::Ok), Gain::default())],
            gain: Gain {
                army: [Some(PIKEMEN), None, None, None, None, None, None],
                ..Gain::default()
            },
        }),
        _ => None,
    };
    let beside = matches!(id, "ask-pile" | "ask-artifact");
    Found {
        at: ENTRANCE,
        first: Vec::new(),
        again: None,
        gain: Gain::default(),
        found: None,
        taken: None,
        event: false,
        blocks: beside.then_some(OBJECT),
        ask: Some(Question {
            asked: laid("Will you?", Style::OkCancel),
            chooses: !matches!(id, "ask-crypt" | "ask-tree" | "ask-tomb"),
            yes,
        }),
    }
}

/// A question sequence's start, its actions, its end and its timeline.
fn asking(id: &str) -> (Start, BTreeMap<u32, Vec<Action>>, u32, &'static str) {
    let entry = ASKING.iter().find(|(f, ..)| *f == id).unwrap();
    let (mut start, actions, ms, timeline) = started(entry);
    start.finds = vec![question(id)];
    (start, actions, ms, timeline)
}

#[test]
fn replays_the_recorded_questions() {
    for (id, ..) in ASKING {
        let (start, actions, ms, timeline) = asking(id);
        let states = play(rules(Bugs::Original), &start, &actions, ms).unwrap();
        let got = lines(&states, &BTreeMap::new());
        let want = without_screens(timeline);
        for (n, (g, w)) in got.iter().zip(&want).enumerate() {
            assert_eq!(g, w, "{id}: line {n}");
        }
        assert_eq!(got.len(), want.len(), "{id}");
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

/// The loop of a recorded question run to `until`.
fn ask_to(id: &str, until: u32) -> Loop {
    let (start, actions, _, _) = asking(id);
    run(&start, &actions, until)
}

/// The open box's text.
fn text(l: &Loop) -> Option<String> {
    l.drawn()
        .message
        .as_ref()
        .map(|m| m.shown.texts[&TEXT].clone())
}

/// The selected hero's window's morale and army.
fn hero_window(l: &Loop) -> (i8, [Option<Stack>; 7]) {
    match &l.drawn().kingdom.status {
        Some(Status::Hero(hero)) => (hero.morale, hero.army),
        other => panic!("no hero's window: {other:?}"),
    }
}

#[test]
fn a_question_from_beside_and_its_cancel() {
    // `ask-pile`: the key's step turns the hero up beside the pile, its question in that
    // millisecond (the default cursor); Escape presses the Cancel, its release ends the step and
    // goes to the idle step (two redraws due at 1060), the adventure cursor again.
    let l = ask_to("ask-pile", 100);
    let s = l.state(100);
    assert_eq!(
        s.hero,
        Some(Hero {
            at: Pos { x: 5, y: 5, z: 0 },
            facing: Direction::North
        })
    );
    assert_eq!(text(&l).as_deref(), Some("Will you?"));
    assert_eq!((s.cursor.kind, s.cursor.frame), (DEFAULT_CURSOR, 0));
    assert!(l.cursor().shown);
    let l = ask_to("ask-pile", 1000);
    assert!(
        l.drawn()
            .message
            .as_ref()
            .is_some_and(|m| m.shown.state(NO).pressed)
    );
    let l = ask_to("ask-pile", 1060);
    let s = l.state(1060);
    assert!(l.boxed.is_none() && s.drawn.message.is_none());
    assert_eq!((s.objects, s.cursor.kind), (2, CURSOR_TYPE));
    assert!(l.answered.is_empty());
}

#[test]
fn a_question_on_the_entrance_asked_again() {
    // `ask-mine`: the step onto its entrance, its question in that millisecond.
    let mut l = ask_to("ask-mine", 1060);
    assert_eq!(l.state(1060).hero.map(|h| h.at), Some(ENTRANCE));
    assert!(l.boxed.is_none());
    // Declined, asked again at the next visit (the hero down and up again).
    l.run(1061, &[Action::Step(Direction::South)]).unwrap();
    assert!(text(&l).is_none());
    l.run(1062, &[Action::Step(Direction::North)]).unwrap();
    assert_eq!(text(&l).as_deref(), Some("Will you?"));
}

#[test]
fn an_answer_not_built_is_refused() {
    // `ask-pile`'s OK (the battle): refused in a recording; live it ends the visit as the
    // Cancel does.
    let (mut start, mut actions, _, _) = asking("ask-pile");
    for t in [1000, 1060] {
        let key = |a: &Action| match a {
            Action::BoxKey(_) => Action::BoxKey(Key::Ok),
            _ => Action::BoxKeyUp(Key::Ok),
        };
        let given: Vec<Action> = actions[&t].iter().map(key).collect();
        actions.insert(t, given);
    }
    let mut l = run(&start, &actions, 1059);
    assert_eq!(
        l.run(1060, &actions[&1060]),
        Err(Unplayable::Answer([5, 4, 0]))
    );
    start.live = true;
    let l = run(&start, &actions, 1060);
    assert!(l.boxed.is_none() && l.answered.is_empty());
    assert_eq!(
        Unplayable::Answer([5, 4, 0]).to_string(),
        "the answer of the question at (5, 4, 0) (the engine's)"
    );
}

#[test]
fn an_answers_box_then_its_gain() {
    // `ask-tomb`: Enter's OK at 1060 opens the find's box at once (no idle step between), its
    // close at 1660 shows the morale lost, then the idle step.
    let l = ask_to("ask-tomb", 1000);
    assert!(
        l.drawn()
            .message
            .as_ref()
            .is_some_and(|m| m.shown.state(YES).pressed)
    );
    let morale = hero_window(&l).0;
    let l = ask_to("ask-tomb", 1060);
    let s = l.state(1060);
    assert_eq!(text(&l).as_deref(), Some("Gloves."));
    assert_eq!((s.objects, s.cursor.kind), (0, DEFAULT_CURSOR));
    assert_eq!(hero_window(&l).0, morale);
    assert_eq!(l.answered, vec![0]);
    let l = ask_to("ask-tomb", 1600);
    assert!(
        l.drawn()
            .message
            .as_ref()
            .is_some_and(|m| m.shown.state(OK).pressed)
    );
    let mut l = ask_to("ask-tomb", 1660);
    let s = l.state(1660);
    assert!(l.boxed.is_none());
    assert_eq!((s.objects, s.cursor.kind), (2, CURSOR_TYPE));
    assert_eq!(hero_window(&l).0, (morale - 3).max(-3));
    // Searched, the Tomb tells nothing more (*provisional*).
    l.run(1661, &[Action::Step(Direction::South)]).unwrap();
    l.run(1662, &[Action::Step(Direction::North)]).unwrap();
    assert!(text(&l).is_none() && l.boxed.is_none());
    // `ask-hire`: the Pikemen join the first empty slot at the box's close.
    let before = hero_window(&ask_to("ask-hire", 1600)).1;
    assert!(!before.contains(&Some(PIKEMEN)));
    let army = hero_window(&ask_to("ask-hire", 1660)).1;
    assert_eq!(
        army.iter().flatten().count(),
        before.iter().flatten().count() + 1
    );
    assert!(army.contains(&Some(PIKEMEN)));
}

#[test]
fn a_question_after_the_first_boxes_and_a_visit_from_beside_without_one() {
    // A find's first box, then its question at its close (the status window chosen again).
    let (mut start, actions, _, _) = asking("ask-mine");
    start.finds[0].first = vec![(laid("First.", Style::Ok), Gain::default())];
    let mut l = run(&start, &actions, 100);
    assert_eq!(text(&l).as_deref(), Some("First."));
    l.run(200, &[Action::BoxKey(Key::Ok)]).unwrap();
    l.run(201, &[Action::BoxKeyUp(Key::Ok)]).unwrap();
    assert_eq!(text(&l).as_deref(), Some("Will you?"));
    assert!(matches!(
        l.boxed.as_ref().map(|b| b.told),
        Some(Told::Asked(0))
    ));
    // An answer with no box: its gain at once, then the idle step.
    let (mut start, actions, _, _) = asking("ask-tomb");
    if let Some(question) = &mut start.finds[0].ask
        && let Some(yes) = &mut question.yes
    {
        yes.boxes.clear();
    }
    let mut l = run(&start, &actions, 1060);
    assert!(l.boxed.is_none());
    assert_eq!(l.answered, vec![0]);
    // An answer's box of a find past the finds closes to the idle step.
    assert_eq!(l.answered(1061, 5, 0), Ok(()));
    assert!(l.boxed.is_none());
    // A find blocking its tile with no box, no question and nothing taken: the visit and the
    // step end at once.
    let (mut start, actions, _, _) = asking("ask-pile");
    start.finds[0].ask = None;
    let l = run(&start, &actions, 100);
    assert!(l.boxed.is_none());
    assert_eq!(
        l.state(100).hero.map(|h| h.at),
        Some(Pos { x: 5, y: 5, z: 0 })
    );
    assert!(l.cursor().shown);
}
