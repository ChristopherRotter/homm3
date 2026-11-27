//! A record's visits played (E24 M4 T6, the reviewers' R2-1 to R2-3): each of a record's visits
//! to a tile a find of its own, told at its own walk's end (a Windmill's wood, then empty; Pandora's
//! Box declined from beside, then opened); a visit's own level-ups, any hero's, opened once its
//! experience shows; the record player's answers to a choice (its picture clicked, then Enter), to
//! a question after a find's box (the box's Enter leaves the answer for it) and to a level-up
//! after an army dialog's commands.

use super::*;

use h3_protocol::wire::game::{
    Answer as Given, ArenaChoice, ChestChoice, Consent, SchoolChoice, SkillOffer,
};
use h3_ui::message::Style;

use super::choices::chosen;
use super::director::{at, east_in_order, east_walk, record_start};
use super::turns::laid;
use crate::director::{self, Pace};
use crate::engine::Command;

/// The tile east of the hero the record's walks end on.
const EAST: Pos = Pos { x: 21, y: 17, z: 0 };

/// The object there (its index in the map's order).
const OBJECT: usize = 3;

/// The walk back west from `EAST` to the hero's tile, as a record gives it, and its arrows.
fn west() -> (Vec<Arrow>, Move) {
    let arrows = vec![
        arrow(20, 17, 16),
        arrow(19, 17, 16),
        arrow(18, 17, 16),
        arrow(17, 17, 1),
    ];
    let walk = Move {
        tiles: vec![at(20, 17), at(19, 17), at(18, 17), at(17, 17)],
        points: vec![None, None, None, Some(760)],
        revealed: vec![Vec::new(); 4],
        sea: None,
        battle: None,
        boats: Vec::new(),
        stops: None,
    };
    (arrows, walk)
}

/// A record's find at `EAST` of the walk `walk`: its box `text`, `gain` at its close.
fn told(walk: usize, text: &str, gain: Gain) -> Found {
    Found {
        at: EAST,
        first: vec![(laid(text, Style::Ok), Gain::default())],
        again: None,
        gain,
        found: None,
        taken: None,
        event: false,
        blocks: None,
        ask: None,
        flag: None,
        fight: None,
        walk: Some(walk),
        levels: Vec::new(),
    }
}

/// `n` of wood.
fn wood(n: i32) -> Gain {
    let mut gain = Gain::default();
    gain.resources[0] = n;
    gain
}

/// The loop of `start` with each of `actions` at its millisecond, run to `until`.
fn run(start: &Start, actions: &[(u32, Action)], until: u32) -> Result<Loop, Unplayable> {
    let mut l = Loop::start(rules(Bugs::Fixed), start)?;
    for t in 1..=until {
        let given: Vec<Action> = actions
            .iter()
            .filter(|(at, _)| *at == t)
            .map(|&(_, a)| a)
            .collect();
        l.run(t, &given)?;
    }
    Ok(l)
}

/// The open box's message.
fn shown(l: &Loop) -> Option<Arc<MessageBox>> {
    l.drawn().message.clone()
}

/// Enter pressed at `t` and released 60 ms later.
fn enter(t: u32) -> [(u32, Action); 2] {
    [
        (t, Action::BoxKey(Key::Ok)),
        (t + 60, Action::BoxKeyUp(Key::Ok)),
    ]
}

#[test]
fn each_visit_to_a_tile_shows_its_own_box_and_gain() {
    // A Windmill's wood on the first walk east, nothing on the walk back, empty on the third: the
    // third walk's end the third walk's find, not the first's again (R2-1).
    let mut start = record_start();
    let (back, west_walk) = west();
    start.paths = vec![east_in_order(), back];
    start.moves = Some(vec![east_walk(), west_walk, east_walk()]);
    let first = told(0, "Wood.", wood(3));
    let empty = told(2, "Empty.", Gain::default());
    start.finds = vec![first.clone(), empty.clone()];
    let mut actions = vec![(1, Action::Walk(EAST))];
    actions.extend(enter(100));
    actions.push((300, Action::Walk(at(17, 17))));
    actions.push((500, Action::Walk(EAST)));
    let l = run(&start, &actions, 1).unwrap();
    assert_eq!(shown(&l), Some(Arc::clone(&first.first[0].0)));
    let l = run(&start, &actions, 400).unwrap();
    assert!(!l.boxed() && l.kingdom.resources[0] == 3);
    let l = run(&start, &actions, 500).unwrap();
    assert_eq!(shown(&l), Some(Arc::clone(&empty.first[0].0)));
    actions.extend(enter(600));
    let l = run(&start, &actions, 700).unwrap();
    assert!(!l.boxed() && l.kingdom.resources[0] == 3);
    assert_eq!(l.walks_left(), 0);
    // A recording's find (no walk) is told at every visit: its first boxes, then its `again`.
    let mut recorded = start.clone();
    recorded.finds = vec![Found {
        walk: None,
        again: Some(laid("Again.", Style::Ok)),
        ..first
    }];
    let l = run(&recorded, &actions, 500).unwrap();
    assert_eq!(l.drawn().message, recorded.finds[0].again);
}

#[test]
fn a_box_declined_from_beside_then_opened() {
    // Pandora's Box on the walk east's last tile, visited from beside: declined, then the one
    // step onto it opens it (R2-1: before, the first visit's question again, its OK not built).
    let mut start = record_start();
    start.paths = vec![east_in_order(), vec![arrow(21, 17, 1)]];
    let onto = Move {
        tiles: vec![EAST],
        points: vec![Some(1000)],
        revealed: vec![Vec::new()],
        sea: None,
        battle: None,
        boats: Vec::new(),
        stops: None,
    };
    start.moves = Some(vec![east_walk(), onto]);
    let asked = laid("Open it?", Style::OkCancel);
    let question = |yes: Option<Answer>, no: Option<Answer>| Question {
        hosted: None,
        asked: Arc::clone(&asked),
        chooses: false,
        yes,
        no,
        attacker: None,
        second: None,
    };
    let mut gold = Gain::default();
    gold.resources[6] = 1500;
    let opened = Answer {
        gain: gold,
        taken: Some((OBJECT, Fade::Pickup)),
        ..Answer::default()
    };
    let find = |walk, ask| Found {
        first: Vec::new(),
        blocks: Some(OBJECT),
        ask: Some(ask),
        ..told(walk, "", Gain::default())
    };
    start.finds = vec![
        find(0, question(None, Some(Answer::default()))),
        find(1, question(Some(opened), None)),
    ];
    let mut actions = vec![
        (1, Action::Walk(EAST)),
        (100, Action::BoxKey(Key::Cancel)),
        (160, Action::BoxKeyUp(Key::Cancel)),
        (300, Action::Walk(EAST)),
    ];
    let l = run(&start, &actions, 1).unwrap();
    assert_eq!(shown(&l), Some(Arc::clone(&asked)));
    assert_eq!(l.hero.map(|h| h.at), Some(at(20, 17)));
    let l = run(&start, &actions, 300).unwrap();
    assert_eq!(shown(&l), Some(Arc::clone(&asked)));
    actions.extend(enter(400));
    let l = run(&start, &actions, 1500).unwrap();
    assert!(l.drawn().gone.contains(&OBJECT));
    assert_eq!(l.kingdom.resources[6], 1500);
    assert_eq!(l.walks_left(), 0);
}

#[test]
fn a_pile_on_a_walk_s_last_node_is_taken_from_beside() {
    // A record's pile on the walk east's last tile: the hero stops beside it, the pile taken
    // (its dissolve), what it gives shown, the walk's end after the dissolve.
    let mut start = record_start();
    let mut gold = Gain::default();
    gold.resources[6] = 500;
    start.finds = vec![Found {
        first: Vec::new(),
        taken: Some(OBJECT),
        blocks: Some(OBJECT),
        ..told(0, "", gold)
    }];
    let l = run(&start, &[(1, Action::Walk(EAST))], 1).unwrap();
    assert!(l.dissolving.is_some());
    assert_eq!(l.hero.map(|h| h.at), Some(at(20, 17)));
    let l = run(&start, &[(1, Action::Walk(EAST))], 700).unwrap();
    assert!(l.dissolving.is_none() && l.idle());
    assert!(l.drawn().gone.contains(&OBJECT));
    assert_eq!(l.kingdom.resources[6], 500);
}

/// A level-up's dialog of `level` with no offers (the record's facts laid out).
fn level(level: u8) -> LevelBox {
    LevelBox {
        hosted: false,
        level,
        dialog: laid(&format!("Level {level}."), Style::Ok),
        offers: Vec::new(),
        primary: 1,
        chosen: None,
    }
}

#[test]
fn a_record_s_visit_opens_its_own_level_ups() {
    // A Learning Stone's experience on a record: the record's two level-ups after its box, though
    // the start knows no experience (R2-3: the thresholds not followed), then the visit's end.
    let mut start = record_start();
    assert_eq!(start.experience, None);
    let mut stone = told(
        0,
        "Learned.",
        Gain {
            experience: 2000,
            ..Gain::default()
        },
    );
    stone.levels = vec![level(2), level(3)];
    start.finds = vec![stone];
    let mut actions = vec![(1, Action::Walk(EAST))];
    actions.extend(enter(100));
    let l = run(&start, &actions, 160).unwrap();
    assert_eq!(shown(&l), Some(level(2).dialog));
    assert!(l.asks());
    actions.extend(enter(300));
    let l = run(&start, &actions, 360).unwrap();
    assert_eq!(shown(&l), Some(level(3).dialog));
    actions.extend(enter(500));
    let l = run(&start, &actions, 560).unwrap();
    assert!(!l.boxed() && l.idle());
    // A find whose gain has no experience leaves them waiting: none opens.
    let mut start = record_start();
    let mut plain = told(0, "Nothing.", Gain::default());
    plain.levels = vec![level(2)];
    start.finds = vec![plain];
    let l = run(&start, &actions[..3], 160).unwrap();
    assert!(!l.boxed() && l.idle());
}

/// A choice's find at `EAST` on the walk east: its box of `style`, the first picture's answer
/// `gold` more, the second's `wood` more, its Cancel nothing.
fn choice(style: Style) -> Found {
    let mut gold = Gain::default();
    gold.resources[6] = 1000;
    Found {
        first: Vec::new(),
        ask: Some(Question {
            hosted: None,
            asked: chosen(style, None),
            chooses: false,
            yes: Some(Answer {
                gain: gold,
                ..Answer::default()
            }),
            no: Some(Answer::default()),
            attacker: None,
            second: Some(Answer {
                gain: wood(5),
                ..Answer::default()
            }),
        }),
        ..told(0, "", Gain::default())
    }
}

/// The record's walk east and `answer` played by the record player on `start`: its input and
/// the map after it.
fn directed(start: &Start, answer: Given) -> (Vec<Action>, Loop) {
    let walk = Command::Walk { hero: 4, to: EAST };
    let commands = [walk, Command::Answer(answer)];
    let d = director::direct(
        rules(Bugs::Fixed),
        start,
        &commands,
        &[(4, 0)],
        Pace::default(),
        5000,
    )
    .unwrap_or_else(|(t, why)| panic!("{answer:?}: at {t}: {why}"));
    let given = d.actions.values().flatten().copied().collect();
    let pairs: Vec<(u32, Action)> = d
        .actions
        .iter()
        .flat_map(|(&t, a)| a.iter().map(move |&a| (t, a)))
        .collect();
    let l = run(start, &pairs, d.end).unwrap();
    (given, l)
}

#[test]
fn the_record_player_clicks_a_choice_s_picture_then_enter() {
    // A chest's gold or experience, the Arena's Attack or Defense, a School's first or second
    // skill: the picture its answer chose clicked at its middle, the pointer back, then Enter
    // (R2-2: Enter alone does nothing, the OK disabled until a picture is chosen); a School left
    // its Cancel.
    let mut start = record_start();
    start.finds = vec![choice(Style::Choose)];
    let mut boxed = Loop::start(rules(Bugs::Fixed), &start).unwrap();
    boxed.run(1, &[Action::Walk(EAST)]).unwrap();
    let picture = |n| boxed.choice_at(n).unwrap();
    assert_ne!(picture(0), picture(1));
    assert_eq!(boxed.choice_at(2), None);
    assert_eq!(boxed.choice_at(usize::from(u16::MAX)), None);
    assert_eq!(boxed.choice_at(usize::MAX), None);
    let idle = Loop::start(rules(Bugs::Fixed), &start).unwrap();
    assert_eq!(idle.choice_at(0), None);
    for (answer, style, n, gold, wood) in [
        (
            Given::Chest {
                choice: ChestChoice::Gold,
            },
            Style::Choose,
            Some(0),
            1000,
            0,
        ),
        (
            Given::Chest {
                choice: ChestChoice::Experience,
            },
            Style::Choose,
            Some(1),
            0,
            5,
        ),
        (
            Given::Arena {
                choice: ArenaChoice::Attack,
            },
            Style::ChooseCancel,
            Some(0),
            1000,
            0,
        ),
        (
            Given::Arena {
                choice: ArenaChoice::Defense,
            },
            Style::ChooseCancel,
            Some(1),
            0,
            5,
        ),
        (
            Given::School {
                choice: SchoolChoice::First,
            },
            Style::ChooseCancel,
            Some(0),
            1000,
            0,
        ),
        (
            Given::School {
                choice: SchoolChoice::Second,
            },
            Style::ChooseCancel,
            Some(1),
            0,
            5,
        ),
        (
            Given::School {
                choice: SchoolChoice::Leave,
            },
            Style::ChooseCancel,
            None,
            0,
            0,
        ),
    ] {
        let mut start = record_start();
        start.finds = vec![choice(style)];
        let (given, l) = directed(&start, answer);
        let clicked = given.iter().find_map(|a| match a {
            Action::Click(point) => Some(*point),
            _ => None,
        });
        assert_eq!(clicked, n.map(picture), "{answer:?}");
        assert!(!l.boxed(), "{answer:?}");
        assert_eq!(
            (l.kingdom.resources[6], l.kingdom.resources[0]),
            (gold, wood),
            "{answer:?}"
        );
        assert_eq!(l.pointer, start.pointer, "{answer:?}");
    }
}

#[test]
fn the_record_player_keeps_an_answer_for_its_question() {
    // Pandora's Box's map text before its question (R2-2, R2-4): the text's box takes Enter, the
    // record's decline is its question's Cancel.
    let mut start = record_start();
    let question = Question {
        hosted: None,
        asked: laid("Open it?", Style::OkCancel),
        chooses: false,
        yes: None,
        no: Some(Answer::default()),
        attacker: None,
        second: None,
    };
    start.finds = vec![Found {
        ask: Some(question),
        ..told(0, "A box.", Gain::default())
    }];
    let decline = Given::Offer {
        consent: Consent::Decline,
    };
    let (given, l) = directed(&start, decline);
    let keys: Vec<Action> = given
        .into_iter()
        .filter(|a| matches!(a, Action::BoxKey(_)))
        .collect();
    assert_eq!(keys, [Action::BoxKey(Key::Ok), Action::BoxKey(Key::Cancel)]);
    assert!(!l.boxed() && l.idle());
}

#[test]
fn the_record_player_passes_an_army_dialog_s_commands_to_a_level_up() {
    // A reward's level-up after the army dialog of its creatures with no room (R2-4: the dialog
    // not shown): the dialog's commands and its close passed over, the level-up's offer given in
    // its dialog.
    let mut start = record_start();
    let mut reward = told(
        0,
        "Rewards.",
        Gain {
            experience: 1000,
            ..Gain::default()
        },
    );
    reward.levels = vec![level(2)];
    start.finds = vec![reward];
    let walk = Command::Walk { hero: 4, to: EAST };
    let commands = [
        walk,
        Command::Other,
        Command::Answer(Given::Close {}),
        Command::Answer(Given::LevelUp {
            offer: SkillOffer::First,
        }),
    ];
    let d = director::direct(
        rules(Bugs::Fixed),
        &start,
        &commands,
        &[(4, 0)],
        Pace::default(),
        5000,
    )
    .unwrap();
    let given: Vec<Action> = d.actions.values().flatten().copied().collect();
    let offered = given.iter().position(|a| *a == Action::Offer(0)).unwrap();
    let entered = given
        .iter()
        .position(|a| *a == Action::BoxKey(Key::Ok))
        .unwrap();
    // The reward's box closed by Enter first, the offer in the level-up's dialog after it.
    assert!(entered < offered);
    assert_eq!(given.iter().filter(|a| **a == Action::Offer(0)).count(), 1);
}
