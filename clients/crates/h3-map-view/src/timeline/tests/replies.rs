//! Part A2's answers (E24 M4 T2b2, design §5.4): a monster's join and its flight, a Border Guard
//! passed, a Cartographer's maps, Pandora's Box. The six replayed headless; the monster drawn as
//! its attack from the visit's start; the object taken with its sound and dissolved; the next
//! question in the Cancel's millisecond; the terrains revealed; a Cancel not built refused.

use super::*;

use h3_ui::message::{Key, Style, TEXT};

use super::turns::{laid, started};
use crate::live::Attacking;

/// E24 M4 T2b1's sequences this part plays: their situations, timelines and kept live parts.
const REPLYING: [(&str, &str, &str, &str); 6] = [
    (
        "ask-join",
        include_str!("../../../../../map/situations/seq-ask-join.toml"),
        include_str!("../../../../../map/timelines/ask-join.txt"),
        include_str!("../../../../../map/timelines/ask-join-live.toml"),
    ),
    (
        "ask-join-no",
        include_str!("../../../../../map/situations/seq-ask-join-no.toml"),
        include_str!("../../../../../map/timelines/ask-join-no.txt"),
        include_str!("../../../../../map/timelines/ask-join-no-live.toml"),
    ),
    (
        "ask-join-gold",
        include_str!("../../../../../map/situations/seq-ask-join-gold.toml"),
        include_str!("../../../../../map/timelines/ask-join-gold.txt"),
        include_str!("../../../../../map/timelines/ask-join-gold-live.toml"),
    ),
    (
        "ask-guard",
        include_str!("../../../../../map/situations/seq-ask-guard.toml"),
        include_str!("../../../../../map/timelines/ask-guard.txt"),
        include_str!("../../../../../map/timelines/ask-guard-live.toml"),
    ),
    (
        "ask-cartographer",
        include_str!("../../../../../map/situations/seq-ask-cartographer.toml"),
        include_str!("../../../../../map/timelines/ask-cartographer.txt"),
        include_str!("../../../../../map/timelines/ask-cartographer-live.toml"),
    ),
    (
        "ask-pandora",
        include_str!("../../../../../map/situations/seq-ask-pandora.toml"),
        include_str!("../../../../../map/timelines/ask-pandora.txt"),
        include_str!("../../../../../map/timelines/ask-pandora-live.toml"),
    ),
];

/// Every question's entrance: above the hero's (5, 5).
const ENTRANCE: Pos = Pos { x: 5, y: 4, z: 0 };

/// The object asked about (the map's fourth: two heroes and a town before it).
const OBJECT: usize = 3;

/// The ten Peasants of `ask-join` and `ask-join-no`, the five Pikemen of `ask-join-gold`.
const PEASANTS: Stack = Stack {
    creature: 139,
    count: 10,
};
const PIKEMEN: Stack = Stack {
    creature: 0,
    count: 5,
};

/// A land Cartographer's terrains.
const LAND: u32 = 0xBF;

/// A monster's join: `stack` joining for `gold`, the status window built again, the monster
/// faded out.
fn joined(stack: Stack, gold: i32) -> Answer {
    let mut gain = Gain::default();
    gain.army[0] = Some(stack);
    gain.resources[6] = -gold;
    Answer {
        gain,
        chooses: true,
        taken: Some((OBJECT, Fade::Killfade)),
        fight: None,
        flag: None,
        ..Answer::default()
    }
}

/// A monster's flight's question: its OK the battle (not built), its Cancel the monster faded
/// out.
fn flight(attacker: u32) -> Question {
    Question {
        asked: laid("Flee?", Style::OkCancel),
        chooses: false,
        yes: None,
        no: Some(Answer {
            taken: Some((OBJECT, Fade::Killfade)),
            fight: None,
            flag: None,
            ..Answer::default()
        }),
        attacker: Some(attacker),
        second: None,
    }
}

/// A sequence's question as the binary builds it (the boxes' texts here their own).
fn question(id: &str) -> Found {
    let cancel = Some(Answer::default());
    let (yes, no, attacker) = match id {
        "ask-join" | "ask-join-no" => (
            joined(PEASANTS, 0),
            Some(Answer {
                then: Some(Box::new(flight(278))),
                fight: None,
                flag: None,
                ..Answer::default()
            }),
            Some(278),
        ),
        "ask-join-gold" => (
            joined(PIKEMEN, 300),
            Some(Answer {
                then: Some(Box::new(flight(0))),
                fight: None,
                flag: None,
                ..Answer::default()
            }),
            Some(0),
        ),
        "ask-guard" => (
            Answer {
                taken: Some((OBJECT, Fade::Pickup)),
                fight: None,
                flag: None,
                ..Answer::default()
            },
            cancel,
            None,
        ),
        "ask-cartographer" => (
            Answer {
                gain: Gain {
                    resources: [0, 0, 0, 0, 0, 0, -1000],
                    ..Gain::default()
                },
                reveal: LAND,
                fight: None,
                flag: None,
                ..Answer::default()
            },
            cancel,
            None,
        ),
        _ => (
            Answer {
                boxes: vec![(
                    laid("Treasure.", Style::Ok),
                    Gain {
                        resources: [0, 0, 0, 0, 0, 0, 1500],
                        ..Gain::default()
                    },
                )],
                taken: Some((OBJECT, Fade::Pickup)),
                fight: None,
                flag: None,
                ..Answer::default()
            },
            cancel,
            None,
        ),
    };
    Found {
        at: ENTRANCE,
        first: Vec::new(),
        again: None,
        gain: Gain::default(),
        found: None,
        taken: None,
        event: false,
        blocks: (id != "ask-cartographer").then_some(OBJECT),
        flag: None,
        ask: Some(Question {
            asked: laid("Will you?", Style::OkCancel),
            chooses: false,
            yes: Some(yes),
            no,
            attacker,
            second: None,
        }),
        fight: None,
        walk: None,
        levels: Vec::new(),
    }
}

/// A sequence's start, its actions, its end and its timeline.
fn replying(id: &str) -> (Start, BTreeMap<u32, Vec<Action>>, u32, &'static str) {
    let entry = REPLYING.iter().find(|(f, ..)| *f == id).unwrap();
    let (mut start, actions, ms, timeline) = started(entry);
    start.finds = vec![question(id)];
    (start, actions, ms, timeline)
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

/// The loop of a recorded sequence run to `until`.
fn reply_to(id: &str, until: u32) -> Loop {
    let (start, actions, _, _) = replying(id);
    run(&start, &actions, until)
}

/// The open box's text.
fn text(l: &Loop) -> Option<String> {
    l.drawn()
        .message
        .as_ref()
        .map(|m| m.shown.texts[&TEXT].clone())
}

/// The selected hero's window's army.
fn army(l: &Loop) -> [Option<Stack>; 7] {
    match &l.drawn().kingdom.status {
        Some(Status::Hero(hero)) => hero.army,
        other => panic!("no hero's window: {other:?}"),
    }
}

#[test]
fn replays_the_recorded_answers() {
    for (id, ..) in REPLYING {
        let (start, actions, ms, timeline) = replying(id);
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
fn a_monsters_join() {
    // `ask-join`: from the visit's start the Peasants drawn as their attack (the hero not left of
    // them: frame 2 × 139), the question over them.
    let l = reply_to("ask-join", 100);
    let attacking = Some(Attacking {
        object: OBJECT,
        frame: 278,
    });
    assert_eq!(l.drawn().attacking, attacking);
    assert_eq!(text(&l).as_deref(), Some("Will you?"));
    // Enter's OK: the army joined and the status window built at once, the monster taken off the
    // map with `killfade` and dissolved, its picture before the dissolve the attack's.
    let mut l = reply_to("ask-join", 1059);
    let before = army(&l);
    let tiles = crate::sound::tests::tiles(36, |_, _| 0, |_, _| None);
    l.sound = Some(crate::sound::tests::sound(tiles));
    let (_, actions, _, _) = replying("ask-join");
    l.run(1060, &actions[&1060]).unwrap();
    assert!(
        l.take_cues()
            .contains(&(1060, Cue::Effect("killfade.wav".into())))
    );
    assert!(l.boxed.is_none());
    assert_eq!(l.answered, vec![0]);
    assert!(army(&l).contains(&Some(PEASANTS)) && !before.contains(&Some(PEASANTS)));
    assert_eq!(*l.drawn().gone, vec![OBJECT]);
    assert_eq!(l.drawn().attacking, None);
    let dissolve = l.drawn().dissolve.clone().unwrap();
    assert_eq!((dissolve.step, dissolve.before.attacking), (0, attacking));
    // `ask-join-gold`: the price paid as it joins.
    let gold = reply_to("ask-join-gold", 1000).kingdom.resources[6];
    let l = reply_to("ask-join-gold", 1060);
    assert_eq!(l.kingdom.resources[6], gold - 300);
    assert!(army(&l).contains(&Some(PIKEMEN)));
}

#[test]
fn a_monster_left_of_the_hero_faces_it() {
    // A monster up and right of the hero: frame 2 × its creature + 1.
    let (mut start, ..) = replying("ask-join");
    start.finds[0].at = Pos { x: 6, y: 4, z: 0 };
    let mut l = run(&start, &BTreeMap::new(), 99);
    l.run(100, &[Action::Step(Direction::NorthEast)]).unwrap();
    assert_eq!(
        l.drawn().attacking,
        Some(Attacking {
            object: OBJECT,
            frame: 279,
        })
    );
    // A blocking find's question without an attacker draws none.
    let (start, actions, _, _) = replying("ask-guard");
    assert_eq!(run(&start, &actions, 100).drawn().attacking, None);
}

#[test]
fn a_declined_offer_then_the_flight() {
    // `ask-join-no`: Escape's Cancel opens the flight's question in its millisecond (the
    // monster's attack still drawn); its Cancel fades the monster out.
    let l = reply_to("ask-join-no", 1060);
    assert_eq!(text(&l).as_deref(), Some("Flee?"));
    assert!(l.answered.is_empty() && l.drawn().attacking.is_some());
    assert!(matches!(
        l.boxed.as_ref().map(|b| &b.told),
        Some(Told::Asked(0, _))
    ));
    let l = reply_to("ask-join-no", 1660);
    assert!(l.boxed.is_none() && l.dissolving.is_some());
    assert_eq!(*l.drawn().gone, vec![OBJECT]);
    assert!(!army(&l).contains(&Some(PEASANTS)));
    // The flight's OK (the battle) is refused; live the visit ends, the attack's picture gone.
    let (mut start, mut actions, _, _) = replying("ask-join-no");
    for t in [1600, 1660] {
        let key = |a: &Action| match a {
            Action::BoxKey(_) => Action::BoxKey(Key::Ok),
            _ => Action::BoxKeyUp(Key::Ok),
        };
        let given: Vec<Action> = actions[&t].iter().map(key).collect();
        actions.insert(t, given);
    }
    let mut l = run(&start, &actions, 1659);
    assert_eq!(
        l.run(1660, &actions[&1660]),
        Err(Unplayable::Answer([5, 4, 0]))
    );
    start.live = true;
    let l = run(&start, &actions, 1660);
    assert!(l.boxed.is_none() && l.dissolving.is_none());
    assert_eq!(l.drawn().attacking, None);
    // A Cancel not built (the insult and the battle) is refused too.
    let (mut start, actions, _, _) = replying("ask-join-no");
    if let Some(question) = &mut start.finds[0].ask {
        question.no = None;
    }
    let mut l = run(&start, &actions, 1059);
    assert_eq!(
        l.run(1060, &actions[&1060]),
        Err(Unplayable::Answer([5, 4, 0]))
    );
}

#[test]
fn a_guard_passed_and_pandoras_box() {
    // `ask-guard`: the guard taken with a pickup's sound and dissolved.
    let l = reply_to("ask-guard", 1060);
    assert!(l.dissolving.is_some());
    assert_eq!(*l.drawn().gone, vec![OBJECT]);
    // `ask-pandora`: its box with the gold shown as it opens, the Box taken at its close.
    let gold = reply_to("ask-pandora", 1000).kingdom.resources[6];
    let l = reply_to("ask-pandora", 1060);
    assert_eq!(text(&l).as_deref(), Some("Treasure."));
    assert_eq!(l.kingdom.resources[6], gold + 1500);
    assert!(l.drawn().gone.is_empty());
    let l = reply_to("ask-pandora", 1660);
    assert!(l.boxed.is_none() && l.dissolving.is_some());
    assert_eq!(*l.drawn().gone, vec![OBJECT]);
}

#[test]
fn a_cartographers_maps() {
    // `ask-cartographer`: on its entrance; Enter's OK pays, reveals the land, draws the view and
    // the minimap again and goes to the idle step.
    let l = reply_to("ask-cartographer", 100);
    assert_eq!(l.state(100).hero.map(|h| h.at), Some(ENTRANCE));
    assert_eq!(l.drawn().revealed, 0);
    let gold = l.kingdom.resources[6];
    let l = reply_to("ask-cartographer", 1060);
    let s = l.state(1060);
    assert!(l.boxed.is_none() && l.dissolving.is_none());
    assert_eq!(s.drawn.revealed, LAND);
    assert_eq!(l.kingdom.resources[6], gold - 1000);
    assert_eq!((s.objects, s.cursor.kind), (2, CURSOR_TYPE));
    assert_eq!(l.drawn().minimap.view, l.drawn().view);
}
