//! Part B's choices (E24 M4 T2b2, design §5.4): a treasure chest's gold or experience (style 7),
//! the Arena's skills and a School of Magic's (style 10). The six replayed headless; Enter before
//! a choice does nothing; a click on a picture chooses it (its frame shown, the other's hidden, the
//! OK enabled); the OK's answer by the picture chosen, the Cancel's (the Arena's the Attack:
//! MCL-Q33; the School's neither); the chest taken; the cursor over the view after the walk the
//! recording's.

use super::*;

use h3_assets::fnt::{Font, Glyph};
use h3_ui::message::{self, CANCEL, CHOICE, OK, Picture, Style};
use h3_ui::ui::State as Marked;

use super::computers::hover_of;
use super::turns::started;

/// E24 M4 T2b1's sequences this part plays: their situations, timelines and kept live parts.
const CHOOSING: [(&str, &str, &str, &str); 6] = [
    (
        "choose-chest",
        include_str!("../../../../../map/situations/seq-choose-chest.toml"),
        include_str!("../../../../../map/timelines/choose-chest.txt"),
        include_str!("../../../../../map/timelines/choose-chest-live.toml"),
    ),
    (
        "choose-chest-gold",
        include_str!("../../../../../map/situations/seq-choose-chest.toml"),
        include_str!("../../../../../map/timelines/choose-chest-gold.txt"),
        include_str!("../../../../../map/timelines/choose-chest-gold-live.toml"),
    ),
    (
        "choose-arena",
        include_str!("../../../../../map/situations/seq-choose-arena.toml"),
        include_str!("../../../../../map/timelines/choose-arena.txt"),
        include_str!("../../../../../map/timelines/choose-arena-live.toml"),
    ),
    (
        "choose-arena-cancel",
        include_str!("../../../../../map/situations/seq-choose-arena.toml"),
        include_str!("../../../../../map/timelines/choose-arena-cancel.txt"),
        include_str!("../../../../../map/timelines/choose-arena-cancel-live.toml"),
    ),
    (
        "choose-school",
        include_str!("../../../../../map/situations/seq-choose-school.toml"),
        include_str!("../../../../../map/timelines/choose-school.txt"),
        include_str!("../../../../../map/timelines/choose-school-live.toml"),
    ),
    (
        "choose-school-leave",
        include_str!("../../../../../map/situations/seq-choose-school.toml"),
        include_str!("../../../../../map/timelines/choose-school-leave.txt"),
        include_str!("../../../../../map/timelines/choose-school-leave-live.toml"),
    ),
];

/// The chest's entrance and the Arena's and the School's: above the hero's (5, 5).
const ENTRANCE: Pos = Pos { x: 5, y: 4, z: 0 };

/// The object visited (the map's fourth: two heroes and a town before it).
const OBJECT: usize = 3;

/// A font whose every glyph advances `advance`, `height` high.
fn font(advance: i32, height: u8) -> Font {
    Font {
        height,
        y_offset: 0,
        glyphs: vec![
            Glyph {
                left: 0,
                width: advance,
                right: 0,
                pixels: Vec::new(),
            };
            256
        ],
    }
}

/// A choice's box of `style` in test fonts, as the recorded one sits (320 × 448 at (240, 76),
/// its two pictures' frames under the recorded clicks: eleven lines of text, two pictures 82 × 93
/// with a caption a line high).
pub(super) fn chosen(style: Style, or: Option<&str>) -> Arc<MessageBox> {
    let layout =
        AdventureLayout::parse(include_str!("../../../../../map/layouts/adventure.toml")).unwrap();
    let text = vec!["x".repeat(30); 11].join("\n");
    let picture = |caption: &str| Picture {
        def: "resour82.def".into(),
        frame: 6,
        size: [82, 93],
        caption: caption.into(),
    };
    Arc::new(message::choice(
        &layout.message,
        (&font(8, 16), &font(5, 10)),
        &text,
        style,
        &[picture("1000"), picture("500")],
        or,
        (0, Bugs::Original),
    ))
}

/// What the panels show: `gold` more, `skills` gained.
fn gain(gold: i32, skills: [i8; 4]) -> Gain {
    let mut gain = Gain {
        skills,
        ..Gain::default()
    };
    gain.resources[6] = gold;
    gain
}

/// A sequence's choice as the binary builds it: the chest's from beside, its gold or nothing the
/// panels show, taken either way; the Arena's +2 Attack (its Cancel's too, marked) or Defense; the
/// School's +1 Spell Power or Knowledge for 1 000 gold, its Cancel nothing.
fn choice(id: &str) -> Found {
    let answer = |gain: Gain, taken: bool| Answer {
        gain,
        taken: taken.then_some((OBJECT, Fade::Pickup)),
        ..Answer::default()
    };
    let chest = id.starts_with("choose-chest");
    let (yes, second, no) = if chest {
        (
            answer(gain(1000, [0; 4]), true),
            answer(Gain::default(), true),
            Answer::default(),
        )
    } else if id.starts_with("choose-arena") {
        (
            answer(gain(0, [2, 0, 0, 0]), false),
            answer(gain(0, [0, 2, 0, 0]), false),
            Answer {
                marks: true,
                ..answer(gain(0, [2, 0, 0, 0]), false)
            },
        )
    } else {
        (
            answer(gain(-1000, [0, 0, 1, 0]), false),
            answer(gain(-1000, [0, 0, 0, 1]), false),
            Answer::default(),
        )
    };
    let style = if chest {
        Style::Choose
    } else {
        Style::ChooseCancel
    };
    Found {
        at: ENTRANCE,
        first: Vec::new(),
        again: None,
        gain: Gain::default(),
        found: None,
        taken: None,
        event: false,
        blocks: chest.then_some(OBJECT),
        flag: None,
        ask: Some(Question {
            asked: chosen(style, chest.then_some("or")),
            chooses: true,
            yes: Some(yes),
            no: Some(no),
            attacker: None,
            second: Some(second),
        }),
        fight: None,
    }
}

/// A sequence's start (its mouse-over, the cursor's frames after the walk recorded), its actions,
/// its end and its timeline.
fn choosing(id: &str) -> (Start, BTreeMap<u32, Vec<Action>>, u32, &'static str) {
    let entry = CHOOSING.iter().find(|(f, ..)| *f == id).unwrap();
    let (mut start, actions, ms, timeline) = started(entry);
    start.finds = vec![choice(id)];
    start.hover = Some(hover_of(entry.3));
    let sequences = Sequences::parse(SEQUENCES).unwrap();
    start.reached = sequences.get(id).unwrap().reached(timeline).unwrap();
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

/// The loop of a recorded choice run to `until`.
fn choose_to(id: &str, until: u32) -> Loop {
    let (start, actions, _, _) = choosing(id);
    run(&start, &actions, until)
}

/// The open box's OK's state and the frames' (hidden), the picture chosen.
fn marks(l: &Loop) -> (Marked, [bool; 2], Option<u16>) {
    let boxed = l.boxed.as_ref().unwrap();
    let shown = &boxed.open.message.shown;
    let hidden = [CHOICE, CHOICE + 1].map(|id| shown.state(id).hidden);
    (shown.state(OK), hidden, boxed.open.chosen())
}

/// The selected hero's window's primary skills.
fn skills(l: &Loop) -> [i8; 4] {
    match &l.drawn().kingdom.status {
        Some(Status::Hero(hero)) => hero.skills,
        other => panic!("no hero's window: {other:?}"),
    }
}

#[test]
fn replays_the_recorded_choices() {
    for (id, ..) in CHOOSING {
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
fn a_chests_choice_its_ok_disabled_until_a_picture_is_chosen() {
    let disabled = Marked {
        disabled: true,
        ..Marked::default()
    };
    // `choose-chest`: the box from beside, both frames hidden, its OK disabled; Enter at 600
    // does nothing.
    let l = choose_to("choose-chest", 100);
    assert_eq!(marks(&l), (disabled, [true, true], None));
    assert_eq!(
        l.state(100).hero.map(|h| h.at),
        Some(Pos { x: 5, y: 5, z: 0 })
    );
    let l = choose_to("choose-chest", 660);
    assert_eq!(marks(&l), (disabled, [true, true], None));
    // The gold's picture clicked at 1000: its frame shown, the OK enabled; the experience's at
    // 1400: the gold's frame hidden again.
    let l = choose_to("choose-chest", 1000);
    assert_eq!(marks(&l), (Marked::default(), [false, true], Some(CHOICE)));
    assert_eq!(
        l.drawn().message.as_ref().unwrap().shown,
        l.boxed.as_ref().unwrap().open.message.shown
    );
    let l = choose_to("choose-chest", 1400);
    assert_eq!(
        marks(&l),
        (Marked::default(), [true, false], Some(CHOICE + 1))
    );
    // Enter at 1800 presses the OK, its release at 1860 gives the experience (nothing the panels
    // show), the chest taken and dissolved; marked.
    let before = choose_to("choose-chest", 1859);
    let gold = before.drawn().kingdom.resources[6];
    let l = choose_to("choose-chest", 1860);
    assert!(l.boxed.is_none());
    assert_eq!(l.answered, vec![0]);
    assert_eq!(*l.drawn().gone, vec![OBJECT]);
    assert!(l.drawn().dissolve.is_some());
    assert_eq!(l.drawn().kingdom.resources[6], gold);
    // `choose-chest-gold`: the gold given at the close, then the dissolve.
    let l = choose_to("choose-chest-gold", 1460);
    assert_eq!(l.drawn().kingdom.resources[6], gold + 1000);
    assert_eq!(*l.drawn().gone, vec![OBJECT]);
}

#[test]
fn the_arenas_and_the_schools_choices_and_their_cancels() {
    let attack = skills(&choose_to("choose-arena", 100));
    // `choose-arena`: the Defense chosen, +2 at the OK's release; marked.
    let l = choose_to("choose-arena", 1460);
    assert_eq!(skills(&l), [attack[0], attack[1] + 2, attack[2], attack[3]]);
    assert_eq!(l.answered, vec![0]);
    // `choose-arena-cancel`: Escape presses the Cancel (the OK still disabled); its release gives
    // the Attack (MCL-Q33), marked.
    let l = choose_to("choose-arena-cancel", 1000);
    let open = &l.boxed.as_ref().unwrap().open;
    assert!(open.message.shown.state(CANCEL).pressed);
    let l = choose_to("choose-arena-cancel", 1060);
    assert_eq!(skills(&l), [attack[0] + 2, attack[1], attack[2], attack[3]]);
    assert_eq!(l.answered, vec![0]);
    // `choose-school`: the Knowledge chosen, +1 and 1 000 gold paid; marked.
    let before = choose_to("choose-school", 1459);
    let gold = before.drawn().kingdom.resources[6];
    let l = choose_to("choose-school", 1460);
    assert_eq!(skills(&l), [attack[0], attack[1], attack[2], attack[3] + 1]);
    assert_eq!(l.drawn().kingdom.resources[6], gold - 1000);
    assert_eq!(l.answered, vec![0]);
    // `choose-school-leave`: neither, unpaid, unmarked: asked again at the next visit.
    let mut l = choose_to("choose-school-leave", 1060);
    assert!(l.boxed.is_none() && l.answered.is_empty());
    assert_eq!(l.drawn().kingdom.resources[6], gold);
    l.run(1061, &[Action::Step(Direction::South)]).unwrap();
    l.run(1062, &[Action::Step(Direction::North)]).unwrap();
    assert!(l.boxed.is_some());
    // The Spell Power by a plain OK (no picture chosen: the first's answer; *provisional*).
    let (start, mut actions, _, _) = choosing("choose-school");
    actions.remove(&1000);
    let mut l = run(&start, &actions, 1000);
    let boxed = l.boxed.as_mut().unwrap();
    boxed.open.message.shown.states.remove(&OK);
    for t in 1001..=1460 {
        l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
            .unwrap();
    }
    assert_eq!(skills(&l), [attack[0], attack[1], attack[2] + 1, attack[3]]);
}

#[test]
fn the_cursor_over_the_view_after_the_walk_is_the_recordings() {
    // `choose-arena`: the box's close at 1460 looks at the mouse over the view again (moved there
    // by the click), the reach after the walk the recording's: the move cursor.
    let l = choose_to("choose-arena", 1460);
    assert_eq!((l.cursor().kind, l.cursor().frame), (CURSOR_TYPE, 4));
    // None recorded at that millisecond: the arrow (M3's: the reach after a walk is the
    // engine's).
    let (mut start, actions, _, _) = choosing("choose-arena");
    start.reached.clear();
    let mut l = run(&start, &actions, 1460);
    assert_eq!(l.cursor().frame, hover::frame::ARROW);
    // Over the hero's own tile (5, 4) its cursor, which needs no reach.
    l.run(1461, &[Action::Pointer([304, 272])]).unwrap();
    assert_eq!(l.cursor().frame, hover::frame::HERO);
}

#[test]
fn a_box_a_walk_s_end_opens_drops_no_later_input() {
    // E24 M4 T4b1: at hero speed 2 the walk onto the Arena ends in the loop's turn (no action
    // then), its box opening there; the click on a picture and the pointer's move off the map at
    // 1000 are both taken (the queue emptied when the box opened, not at the next actions).
    let (mut start, mut actions, _, _) = choosing("choose-arena");
    start.hero_speed = 2;
    actions
        .get_mut(&1000)
        .unwrap()
        .push(Action::Pointer([700, 300]));
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    for t in 1..=1000 {
        l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
            .unwrap();
    }
    assert!(l.boxed.is_some());
    assert_eq!(l.pointer, [700, 300]);
}
