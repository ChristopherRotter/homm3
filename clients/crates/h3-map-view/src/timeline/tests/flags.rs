//! A flag raised (E24 M4 T2d1, design §5.6): a mine's and a Lighthouse's box, the object the
//! visitor's at its close, the minimap drawn with it at the visit's end and the view at the idle
//! step's draw; a dwelling's flag before its hire, shown after the hire's last box.

use super::*;

use h3_ui::message::Style;

use super::turns::{laid, started_by};

/// The four sequences as `clients/map/to-record.toml` gives them (until they move into
/// `sequences.toml`): their inputs and times.
const SEQUENCES: &str = r#"
[[sequence]]
id = "flag-mine"
situation = "seq-flag-mine"
ms = 1800
hero_speed = 4
seed = 7
why = "a neutral Sawmill without guards taken"
inputs = [
    { at = 100, keydown = "Up" }, { at = 160, keyup = "Up" },
    { at = 1000, keydown = "Return" }, { at = 1060, keyup = "Return" },
]

[[sequence]]
id = "flag-mine-blue"
situation = "seq-flag-mine-blue"
ms = 1800
hero_speed = 4
seed = 7
why = "Blue's Ore Pit without guards taken by Red"
inputs = [
    { at = 100, keydown = "Up" }, { at = 160, keyup = "Up" },
    { at = 1000, keydown = "Return" }, { at = 1060, keyup = "Return" },
]

[[sequence]]
id = "flag-lighthouse"
situation = "seq-flag-lighthouse"
ms = 1800
hero_speed = 4
seed = 7
why = "a neutral Lighthouse taken"
inputs = [
    { at = 100, keydown = "Up" }, { at = 160, keyup = "Up" },
    { at = 1000, keydown = "Return" }, { at = 1060, keyup = "Return" },
]

[[sequence]]
id = "flag-dwelling"
situation = "seq-flag-dwelling"
ms = 2200
hero_speed = 4
seed = 7
why = "a neutral Guardhouse taken, its hire accepted"
inputs = [
    { at = 100, keydown = "Up" }, { at = 160, keyup = "Up" },
    { at = 1000, keydown = "Return" }, { at = 1060, keyup = "Return" },
    { at = 1600, keydown = "Return" }, { at = 1660, keyup = "Return" },
]
"#;

/// The sequences' situations, timelines and kept live parts.
const FLAGGING: [(&str, &str, &str, &str); 4] = [
    (
        "flag-mine",
        include_str!("../../../../../map/situations/seq-flag-mine.toml"),
        include_str!("../../../../../map/timelines/flag-mine.txt"),
        include_str!("../../../../../map/timelines/flag-mine-live.toml"),
    ),
    (
        "flag-mine-blue",
        include_str!("../../../../../map/situations/seq-flag-mine-blue.toml"),
        include_str!("../../../../../map/timelines/flag-mine-blue.txt"),
        include_str!("../../../../../map/timelines/flag-mine-blue-live.toml"),
    ),
    (
        "flag-lighthouse",
        include_str!("../../../../../map/situations/seq-flag-lighthouse.toml"),
        include_str!("../../../../../map/timelines/flag-lighthouse.txt"),
        include_str!("../../../../../map/timelines/flag-lighthouse-live.toml"),
    ),
    (
        "flag-dwelling",
        include_str!("../../../../../map/situations/seq-flag-dwelling.toml"),
        include_str!("../../../../../map/timelines/flag-dwelling.txt"),
        include_str!("../../../../../map/timelines/flag-dwelling-live.toml"),
    ),
];

/// Every object's entrance: above the hero's (5, 5).
const ENTRANCE: Pos = Pos { x: 5, y: 4, z: 0 };

/// The flag raised: the map's fourth object (two heroes and a town before it), Red's.
const FLAG: (usize, u8) = (3, 0);

/// A sequence's find as the binary builds it (the boxes' texts here their own): a mine's or a
/// Lighthouse's box, its flag after it; the Guardhouse's flag, then its hire's question (its OK
/// the Pikemen joining free, told).
fn flagged(id: &str) -> Found {
    let (first, ask) = if id == "flag-dwelling" {
        let pikemen = Stack {
            creature: 0,
            count: 14,
        };
        let question = Question {
            asked: laid("Recruit?", Style::OkCancel),
            chooses: true,
            yes: Some(Answer {
                boxes: vec![(laid("Pikemen join.", Style::Ok), Gain::default())],
                gain: Gain {
                    army: [Some(pikemen), None, None, None, None, None, None],
                    ..Gain::default()
                },
                ..Answer::default()
            }),
            no: Some(Answer::default()),
            attacker: None,
            second: None,
        };
        (Vec::new(), Some(question))
    } else {
        (vec![(laid("Yours now.", Style::Ok), Gain::default())], None)
    };
    Found {
        at: ENTRANCE,
        first,
        again: None,
        gain: Gain::default(),
        found: None,
        taken: None,
        event: false,
        blocks: None,
        ask,
        flag: Some(FLAG),
        fight: None,
    }
}

/// A flag sequence's start, its actions, its end and its timeline.
fn flagging(id: &str) -> (Start, BTreeMap<u32, Vec<Action>>, u32, &'static str) {
    let sequences = Sequences::parse(SEQUENCES).unwrap();
    let entry = FLAGGING.iter().find(|(f, ..)| *f == id).unwrap();
    let (mut start, actions, ms, timeline) = started_by(sequences.get(id).unwrap(), entry);
    start.finds = vec![flagged(id)];
    (start, actions, ms, timeline)
}

#[test]
fn replays_the_recorded_flags() {
    for (id, ..) in FLAGGING {
        let (start, actions, ms, timeline) = flagging(id);
        let states = play(rules(Bugs::Original), &start, &actions, ms).unwrap();
        let got = lines(&states, &BTreeMap::new());
        let want = without_screens(timeline);
        for (n, (g, w)) in got.iter().zip(&want).enumerate() {
            assert_eq!(g, w, "{id}: line {n}");
        }
        assert_eq!(got.len(), want.len(), "{id}");
    }
}

/// The loop of a flag sequence run to `until` with its actions.
fn run_to(id: &str, until: u32) -> Loop {
    let (start, actions, _, _) = flagging(id);
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    for t in 1..=until {
        l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
            .unwrap();
    }
    l
}

/// The owners changed in memory, as the view last drew them and as the minimap did.
fn owners(l: &Loop) -> [Vec<(usize, u8)>; 3] {
    let drawn = l.drawn();
    [
        l.owners.to_vec(),
        drawn.owners.to_vec(),
        drawn.minimap.owners.to_vec(),
    ]
}

#[test]
fn a_mines_flag_after_its_box_on_the_minimap_and_the_view_at_its_close() {
    // `flag-mine`: the box at 100, nothing changed while it is open; Enter's release closes it,
    // the flag raised, the minimap drawn with it and the idle step's redraws (two at 1060) with
    // it; told once (back and onto it again: nothing).
    let l = run_to("flag-mine", 1000);
    assert!(l.drawn().message.is_some());
    assert_eq!(owners(&l), [vec![], vec![], vec![]]);
    let mut l = run_to("flag-mine", 1060);
    let s = l.state(1060);
    assert!(s.drawn.message.is_none() && s.objects == 2);
    assert_eq!(owners(&l), [vec![FLAG], vec![FLAG], vec![FLAG]]);
    l.run(1061, &[Action::Step(Direction::South)]).unwrap();
    l.run(1062, &[Action::Step(Direction::North)]).unwrap();
    assert!(l.drawn().message.is_none() && l.boxed.is_none());
    assert_eq!(l.owners.to_vec(), [FLAG]);
}

#[test]
fn a_dwellings_flag_before_its_hire_shown_at_the_visits_end() {
    // `flag-dwelling`: the flag raised at 100 before the question, drawn nowhere until the free
    // join's box closes at 1660: the minimap, then the idle step's view.
    let l = run_to("flag-dwelling", 100);
    assert!(l.drawn().message.is_some());
    assert_eq!(owners(&l), [vec![FLAG], vec![], vec![]]);
    let l = run_to("flag-dwelling", 1060);
    assert!(l.drawn().message.is_some());
    assert_eq!(owners(&l), [vec![FLAG], vec![], vec![]]);
    let l = run_to("flag-dwelling", 1660);
    assert!(l.drawn().message.is_none());
    assert_eq!(owners(&l), [vec![FLAG], vec![FLAG], vec![FLAG]]);
}

#[test]
fn a_flag_raised_again_or_by_another_colour() {
    // A flag already raised changes nothing; the same object's another colour replaces it, in
    // its place; none raises nothing.
    let mut l = run_to("flag-mine", 1);
    l.raise(Some((5, 1)));
    l.raise(Some(FLAG));
    let before = Arc::clone(&l.owners);
    l.raise(Some(FLAG));
    assert!(Arc::ptr_eq(&before, &l.owners));
    l.raise(Some((5, 2)));
    l.raise(None);
    assert_eq!(l.owners.to_vec(), [(5, 2), FLAG]);
}
