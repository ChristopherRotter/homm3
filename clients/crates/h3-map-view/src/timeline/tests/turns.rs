//! The turn's end and the next day (E24 M4 T1a, design §5.1): End Turn's question, the turn's
//! change from the live part's `[[turn]]`, the week's and the month's message.

use super::*;

use h3_assets::fnt::{Font, Glyph};
use h3_ui::message::{self, NO, OK, Style, YES};

use crate::live::{ChosenTown, HeroStatus, Turn};

/// E24 M4 T1a's sequences (and T2a2's turn without a town): their situations, timelines and kept
/// live parts.
const TURNING: [(&str, &str, &str, &str); 8] = [
    (
        "turn-day",
        include_str!("../../../../../map/situations/seq-turn.toml"),
        include_str!("../../../../../map/timelines/turn-day.txt"),
        include_str!("../../../../../map/timelines/turn-day-live.toml"),
    ),
    (
        "turn-question",
        include_str!("../../../../../map/situations/seq-turn.toml"),
        include_str!("../../../../../map/timelines/turn-question.txt"),
        include_str!("../../../../../map/timelines/turn-question-live.toml"),
    ),
    (
        "turn-week",
        include_str!("../../../../../map/situations/seq-turn-week.toml"),
        include_str!("../../../../../map/timelines/turn-week.txt"),
        include_str!("../../../../../map/timelines/turn-week-live.toml"),
    ),
    (
        "turn-week-creature",
        include_str!("../../../../../map/situations/seq-turn-week.toml"),
        include_str!("../../../../../map/timelines/turn-week-creature.txt"),
        include_str!("../../../../../map/timelines/turn-week-creature-live.toml"),
    ),
    (
        "turn-month",
        include_str!("../../../../../map/situations/seq-turn-month.toml"),
        include_str!("../../../../../map/timelines/turn-month.txt"),
        include_str!("../../../../../map/timelines/turn-month-live.toml"),
    ),
    (
        "turn-month-creature",
        include_str!("../../../../../map/situations/seq-turn-month.toml"),
        include_str!("../../../../../map/timelines/turn-month-creature.txt"),
        include_str!("../../../../../map/timelines/turn-month-creature-live.toml"),
    ),
    (
        "turn-plague",
        include_str!("../../../../../map/situations/seq-turn-month.toml"),
        include_str!("../../../../../map/timelines/turn-plague.txt"),
        include_str!("../../../../../map/timelines/turn-plague-live.toml"),
    ),
    (
        "turn-no-town",
        include_str!("../../../../../map/situations/seq-turn-no-town.toml"),
        include_str!("../../../../../map/timelines/turn-no-town.txt"),
        include_str!("../../../../../map/timelines/turn-no-town-live.toml"),
    ),
];

/// A box of `style` laid out in a font of 8-pixel glyphs 16 high (no original file).
pub(super) fn laid(text: &str, style: Style) -> Arc<MessageBox> {
    let layout =
        AdventureLayout::parse(include_str!("../../../../../map/layouts/adventure.toml")).unwrap();
    let font = Font {
        height: 16,
        y_offset: 0,
        glyphs: vec![
            Glyph {
                left: 0,
                width: 8,
                right: 0,
                pixels: Vec::new(),
            };
            256
        ],
    };
    Arc::new(message::layout(
        &layout.message,
        &font,
        text,
        style,
        (0, Bugs::Original),
    ))
}

/// A turn's start with its week's box when its date starts a week.
fn next(turn: Turn) -> NextTurn {
    let week = turn::starts_week(turn.kingdom.date);
    NextTurn {
        message: week.then(|| laid("Week of the Rat.", Style::Ok)),
        turn,
        heroes: Vec::new(),
        walks: Vec::new(),
        taken: Vec::new(),
        flags: Vec::new(),
        lost: Vec::new(),
        list: None,
        first: false,
        open: false,
    }
}

/// The boxes of a turn without a town (E24 M4 T2a2): the warning and the days left's, seven.
fn town() -> TownBoxes {
    TownBoxes {
        lost: laid("Red, you have lost your last town.", Style::Ok),
        left: (1..=7)
            .map(|d| laid(&alloc::format!("{d} days left"), Style::Ok))
            .collect(),
    }
}

/// A turn sequence's start (as the binary's `for_sequence` makes it): the kingdom, the buttons,
/// the map's heroes and the selected one by the mouse-over's facts, the question and the turns;
/// each kingdom hero's choice its map hero by the facts (the binary's loop finds it through the
/// mouse-over), its window the live part's; since E24 M4 T2a2 the boxes of a turn without a town.
fn turning(id: &str) -> (Start, BTreeMap<u32, Vec<Action>>, u32, &'static str) {
    let entry = TURNING.iter().find(|(t, ..)| *t == id).unwrap();
    started(entry)
}

/// The start of the recorded sequence `(id, situation, timeline, live)` as [`turning`] makes it.
pub(super) fn started(
    entry: &(&str, &str, &'static str, &str),
) -> (Start, BTreeMap<u32, Vec<Action>>, u32, &'static str) {
    let sequences = Sequences::parse(SEQUENCES).unwrap();
    started_by(sequences.get(entry.0).unwrap(), entry)
}

/// [`started`] with the sequence given (E24 M4 T2d1: one not yet in `sequences.toml`).
pub(super) fn started_by(
    sequence: &sequence::Sequence,
    &(id, situation, timeline, live): &(&str, &str, &'static str, &str),
) -> (Start, BTreeMap<u32, Vec<Action>>, u32, &'static str) {
    let mut start = start(situation, sequence.scroll_speed().unwrap());
    (start.kingdom, start.buttons) = live_kingdom(live);
    let (heroes, viewer) = live_heroes(live);
    let table: toml::Table = live.parse().unwrap();
    let objects = table["anchors"].as_array().unwrap().len();
    let facts = hover::Facts::read(&table["hover"], objects, heroes.len(), 36, 1).unwrap();
    let index_of = |slot: usize| {
        facts
            .heroes
            .iter()
            .position(|h| h.0 == facts.hero_list[slot].0)
            .unwrap()
    };
    start.heroes = heroes
        .iter()
        .map(|h| Hero {
            at: h.at,
            facing: Direction::ALL[usize::from(h.facing.number())],
        })
        .collect();
    let Selected::Hero(slot) = start.kingdom.selected else {
        panic!("{id}: no hero selected at 0");
    };
    start.index = Some(index_of(slot));
    start.hero = Some(start.heroes[index_of(slot)]);
    let Some(Status::Hero(status)) = start.kingdom.status.clone() else {
        panic!("{id}: no hero's window at 0");
    };
    start.chosen.heroes = (0..facts.hero_list.len())
        .map(|i| {
            Some(ChosenHero {
                hero: index_of(i),
                status: status.clone(),
                path: Vec::new(),
            })
        })
        .collect();
    start.player = viewer;
    start.move_reminder = sequence.move_reminder().unwrap();
    start.question = Some(laid("End?", Style::OkCancel));
    start.turns = table
        .get("turn")
        .and_then(toml::Value::as_array)
        .map_or(&[][..], Vec::as_slice)
        .iter()
        .map(|t| next(Turn::read(t).unwrap()))
        .collect();
    start.points = sequence.points(timeline).unwrap();
    start.paths = sequence.paths(timeline).unwrap();
    start.hero_speed = sequence.hero_speed().unwrap();
    start.town = Some(town());
    (start, sequence.actions().unwrap(), sequence.ms, timeline)
}

#[test]
fn replays_the_recorded_turns() {
    for (id, ..) in TURNING {
        let (start, actions, ms, timeline) = turning(id);
        let states = play(rules(Bugs::Original), &start, &actions, ms).unwrap();
        let got = lines(&states, &BTreeMap::new());
        let want = without_screens(timeline);
        for (n, (g, w)) in got.iter().zip(&want).enumerate() {
            assert_eq!(g, w, "{id}: line {n}");
        }
        assert_eq!(got.len(), want.len(), "{id}");
    }
}

/// The loop of a recorded turn sequence run to `until` with its actions.
fn run_to(id: &str, until: u32) -> Loop {
    let (start, actions, _, _) = turning(id);
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    for t in 1..=until {
        l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
            .unwrap();
    }
    l
}

#[test]
fn end_turn_without_its_question_starts_the_next_day_at_once() {
    // `turn-day`: E's key up at 160 the turn's change in that millisecond.
    let l = run_to("turn-day", 159);
    assert_eq!(l.drawn().pressed, [12]);
    let l = run_to("turn-day", 160);
    let s = l.state(160);
    assert_eq!(
        (s.date, s.view),
        (
            Date {
                month: 1,
                week: 1,
                day: 2
            },
            ViewAt { x: -5, y: -4, z: 0 }
        )
    );
    // The whole screen drawn: the lists with the next day's resources, the date's window asked
    // for 3 000 ms at its first frame; no box on a week's second day.
    assert_eq!(s.drawn.kingdom.resources[6], 21_200);
    assert_eq!((l.asked, l.end), (1, 3160));
    assert_eq!(
        s.drawn.kingdom.status,
        Some(Status::Date(DateStatus { frame: 0 }))
    );
    assert!(s.drawn.message.is_none() && s.drawn.pressed.is_empty());
    assert!(l.drawn().lifted.is_some());
    // The selected hero's window kept for the chooser (the same hero selected again).
    assert!(matches!(l.kingdom.selection, Some(Status::Hero(_))));
    // No turn left: the next End Turn is not built.
    let mut l = l;
    l.run(161, &[Action::Key(Hotkey::EndTurn)]).unwrap();
    assert_eq!(
        l.run(162, &[Action::KeyUp(Hotkey::EndTurn)]),
        Err(Unplayable::Command(12))
    );
}

#[test]
fn the_question_cancelled_and_answered() {
    // `turn-question`: E's key up opens the question, the default cursor's frame 0 at the mouse.
    let l = run_to("turn-question", 160);
    let s = l.state(160);
    assert!(
        s.drawn.message.is_some() && l.boxed.as_ref().is_some_and(|b| b.told == Told::Question)
    );
    assert_eq!((s.cursor.kind, s.cursor.frame), (DEFAULT_CURSOR, 0));
    // The map stands still behind it: no redraw at 180, the palettes stepping in memory.
    let l = run_to("turn-question", 399);
    assert_eq!((l.state(399).objects, l.state(399).cycles), (0, 1));
    // Escape's key down presses Cancel; its key up closes the box, the idle step's redraws due
    // since run (two at 460), the cursor before it again.
    let l = run_to("turn-question", 400);
    assert!(
        l.drawn()
            .message
            .as_ref()
            .is_some_and(|m| m.shown.state(NO).pressed)
    );
    let l = run_to("turn-question", 460);
    let s = l.state(460);
    assert!(s.drawn.message.is_none() && l.boxed.is_none());
    assert_eq!((s.objects, s.cursor.kind), (2, CURSOR_TYPE));
    // End Turn's button again, then Enter: the next day at 1060, the redraw's deadline (820,
    // passed) at 1240.
    let l = run_to("turn-question", 1060);
    assert_eq!(l.state(1060).date.day, 2);
    assert_eq!((l.frame_at, l.state(1060).objects), (1240, 3));
}

#[test]
fn the_weeks_message_steps_the_date_behind_it() {
    // `turn-week`: the box over the week's picture, which steps every 100 ms from its opening to
    // its last frame (the second week's eight).
    let frame = |l: &Loop| match l.kingdom.status {
        Some(Status::Date(d)) => d.frame,
        _ => panic!("not the date's window"),
    };
    let l = run_to("turn-week", 160);
    assert!(l.boxed.as_ref().is_some_and(|b| b.told != Told::Question));
    assert_eq!(frame(&l), 0);
    assert_eq!(frame(&run_to("turn-week", 259)), 0);
    assert_eq!(frame(&run_to("turn-week", 260)), 1);
    assert_eq!(frame(&run_to("turn-week", 1500)), 7);
    // Enter closes it: the idle step's two redraws.
    let l = run_to("turn-week", 1660);
    assert_eq!((l.boxed.is_none(), l.state(1660).objects), (true, 2));
}

/// A loop on the 36-tile map with Red's two heroes (the first selected, able to move), Move
/// Reminder `asks`, the question and `turns`.
fn ended(asks: bool, turns: Vec<NextTurn>, live: bool) -> Loop {
    let (start, ..) = turning("turn-question");
    let start = Start {
        move_reminder: asks,
        turns,
        live,
        ..start
    };
    Loop::start(rules(Bugs::Original), &start).unwrap()
}

/// The `turn-question` sequence's turn, its date `date`, its selection `selected`.
fn turn_on(date: Date, selected: Selected) -> NextTurn {
    let (start, ..) = turning("turn-question");
    let mut turn = start.turns[0].turn.clone();
    turn.kingdom.date = date;
    turn.kingdom.selected = selected;
    next(turn)
}

const WEEK: Date = Date {
    month: 1,
    week: 2,
    day: 1,
};

#[test]
fn the_boxes_take_the_mouse() {
    // The question's buttons by the mouse: a press presses OK (its sound), a move off lets it up,
    // back on presses it again (its sound again), a release over it ends the turn.
    let mut l = ended(true, vec![turn_on(WEEK, Selected::Hero(0))], false);
    let tiles = crate::sound::tests::tiles(36, |_, _| 2, |_, _| None);
    l.sound = Some(crate::sound::tests::sound(tiles));
    l.run(1, &[Action::Click([711, 372])]).unwrap();
    let [x, y] = l.drawn().message.as_ref().unwrap().dialog.place;
    let (ok, off) = ([x + 60, y + 70], [x + 5, y + 5]);
    l.take_cues();
    // A press off the buttons, the right button and a hotkey: nothing.
    l.run(2, &[Action::Press(off), Action::Release(off)])
        .unwrap();
    l.run(3, &[Action::RightPress, Action::Key(Hotkey::NextHero)])
        .unwrap();
    assert!(l.boxed.is_some() && l.take_cues().is_empty());
    l.run(4, &[Action::Press(ok)]).unwrap();
    assert_eq!(l.take_cues().len(), 1);
    l.run(5, &[Action::Pointer(off)]).unwrap();
    assert!(!l.drawn().message.as_ref().unwrap().shown.state(YES).pressed);
    l.run(6, &[Action::Pointer(ok)]).unwrap();
    assert_eq!(l.take_cues().len(), 1);
    assert!(l.drawn().message.as_ref().unwrap().shown.state(YES).pressed);
    // The week's box opens over the next day with its sound; the box's mouse hold stops the
    // date's steps.
    l.run(7, &[Action::Release(ok)]).unwrap();
    assert!(l.boxed.as_ref().is_some_and(|b| b.told != Told::Question));
    let cues: Vec<String> = l
        .take_cues()
        .iter()
        .map(|(t, c)| crate::sound::line(*t, c))
        .collect();
    assert!(cues.iter().any(|c| c.ends_with("newweek.wav")), "{cues:?}");
    let [bx, by] = l.drawn().message.as_ref().unwrap().dialog.place;
    let alone = [bx + 128, by + 70];
    l.run(8, &[Action::Press(alone)]).unwrap();
    assert!(l.boxed.as_ref().is_some_and(|b| b.open.holds_mouse()));
    for t in 9..=300 {
        l.run(t, &[]).unwrap();
    }
    assert_eq!(
        l.kingdom.status,
        Some(Status::Date(DateStatus { frame: 0 }))
    );
    // The right button's release while the mouse holds OK: its click.
    l.run(301, &[Action::RightRelease]).unwrap();
    assert!(l.boxed.is_none());
    // A click on the question's Cancel: closed, the turn kept.
    let mut l = ended(true, vec![turn_on(WEEK, Selected::Hero(0))], false);
    l.run(
        1,
        &[Action::Key(Hotkey::EndTurn), Action::KeyUp(Hotkey::EndTurn)],
    )
    .unwrap();
    assert!(l.boxed.is_some());
    l.run(2, &[Action::Click([x + 140, y + 70])]).unwrap();
    assert_eq!((l.boxed.is_none(), l.next_turn), (true, 0));
}

#[test]
fn a_box_empties_the_queue_as_it_opens() {
    // The actions after the one that opened the box in its millisecond are dropped: Enter's down
    // given with End Turn's release presses nothing.
    let mut l = ended(true, vec![turn_on(WEEK, Selected::Hero(0))], false);
    l.run(1, &[Action::Key(Hotkey::EndTurn)]).unwrap();
    l.run(
        2,
        &[Action::KeyUp(Hotkey::EndTurn), Action::BoxKey(Key::Ok)],
    )
    .unwrap();
    assert!(l.boxed.as_ref().is_some_and(|b| pressed(&b.open).is_none()));
    // A box's keys with none open, and a key up its box never pressed: nothing.
    let mut l = ended(false, Vec::new(), true);
    l.run(1, &[Action::BoxKey(Key::Ok), Action::BoxKeyUp(Key::Ok)])
        .unwrap();
    assert!(l.boxed.is_none() && l.drawn().message.is_none());
    // Live with no turn left (a window's situation): End Turn does nothing.
    l.run(
        2,
        &[Action::Key(Hotkey::EndTurn), Action::KeyUp(Hotkey::EndTurn)],
    )
    .unwrap();
    assert_eq!(
        l.state(2).date,
        Date {
            month: 1,
            week: 1,
            day: 1
        }
    );
    assert_eq!(
        (key_name(Key::Ok), key_name(Key::Cancel)),
        ("Return", "Escape")
    );
    assert_eq!(l.close_box(3, OK), Ok(()));
    assert_eq!(l.box_act(3, Action::Pointer([1, 1]), true), Ok(()));
}

#[test]
fn the_question_needs_a_hero_to_go_to_and_its_box() {
    // No hero able to move: no question, the turn's change at once.
    let mut l = ended(true, vec![turn_on(WEEK, Selected::Hero(0))], false);
    for h in &mut l.kingdom.heroes {
        h.movable = false;
    }
    l.run(
        1,
        &[Action::Key(Hotkey::EndTurn), Action::KeyUp(Hotkey::EndTurn)],
    )
    .unwrap();
    assert_eq!(l.next_turn, 1);
    // A hero to go to and no question laid out: not built.
    let mut l = ended(true, Vec::new(), false);
    l.question = None;
    l.run(1, &[Action::Key(Hotkey::EndTurn)]).unwrap();
    assert_eq!(
        l.run(2, &[Action::KeyUp(Hotkey::EndTurn)]),
        Err(Unplayable::Command(12))
    );
}

#[test]
fn the_turns_selection_a_hero_a_town_or_none() {
    let day = Date {
        month: 1,
        week: 1,
        day: 2,
    };
    let end = [Action::Key(Hotkey::EndTurn), Action::KeyUp(Hotkey::EndTurn)];
    // Another hero: its choice's window and path, the view centred on it.
    let mut l = ended(false, vec![turn_on(day, Selected::Hero(1))], false);
    l.chosen.heroes = vec![l.chosen.heroes[0].clone(), l.chosen.heroes[0].clone()];
    if let Some(Some(c)) = l.chosen.heroes.get_mut(1) {
        c.hero = 1;
        c.path = vec![arrow(33, 32, 1)];
    }
    l.run(1, &end).unwrap();
    assert_eq!(
        (l.index, l.state(1).view),
        (Some(1), ViewAt { x: 23, y: 24, z: 0 })
    );
    assert_eq!(l.arrows(), [arrow(33, 32, 1)]);
    // A town: its choice's.
    let mut l = ended(false, vec![turn_on(day, Selected::Town(0))], false);
    l.chosen.towns = vec![Some(ChosenTown {
        at: Pos { x: 8, y: 4, z: 0 },
        status: crate::live::TownStatus {
            name: crate::live::TownName::Record(0),
            buildings: 0,
            garrison: false,
            army: [None; 7],
            silo: Vec::new(),
            income: 0,
        },
    })];
    l.run(1, &end).unwrap();
    assert_eq!(
        (l.hero, l.state(1).view),
        (None, ViewAt { x: -1, y: -4, z: 0 })
    );
    assert!(matches!(l.kingdom.selection, Some(Status::Town(_))));
    // Nothing selected.
    let mut l = ended(false, vec![turn_on(day, Selected::None)], false);
    l.run(1, &end).unwrap();
    assert_eq!((l.hero, l.index, l.buttons.move_hero), (None, None, false));
    // A town or a hero whose place the live part does not give: not built, nothing changed.
    for selected in [Selected::Town(0), Selected::Hero(5)] {
        let mut l = ended(false, vec![turn_on(day, selected)], false);
        l.run(1, &[Action::Key(Hotkey::EndTurn)]).unwrap();
        assert_eq!(
            l.run(2, &[Action::KeyUp(Hotkey::EndTurn)]),
            Err(Unplayable::Command(12))
        );
        assert_eq!(l.next_turn, 0);
    }
}

#[test]
fn the_turns_hero_by_the_mouse_overs_facts() {
    // Without its choice the kingdom's hero is the map's of its record index (`hovering`'s facts:
    // the list's second, record 20, the map's second hero).
    let mut l = hovering(false, Bugs::Original);
    l.heroes = vec![l.hero.unwrap(); 2];
    assert_eq!(
        (l.map_hero(0), l.map_hero(1), l.map_hero(2)),
        (Some(0), Some(1), None)
    );
    // The turn's hero by them: its record index the mouse-over's selected one.
    let mut ended = l.clone();
    let day = Date {
        month: 1,
        week: 1,
        day: 2,
    };
    ended.turns = Arc::new(vec![turn_on(day, Selected::Hero(0))]);
    ended
        .run(
            1,
            &[Action::Key(Hotkey::EndTurn), Action::KeyUp(Hotkey::EndTurn)],
        )
        .unwrap();
    assert_eq!((ended.index, ended.selected_id), (Some(0), Some(16)));
    l.heroes.truncate(1);
    assert_eq!(l.map_hero(1), None);
    l.hover = None;
    assert_eq!(l.map_hero(0), None);
}

#[test]
fn the_date_and_the_player_lines() {
    let mut l = ended(false, vec![turn_on(WEEK, Selected::Hero(0))], false);
    let first = l.state(0);
    l.turns = Arc::new(
        l.turns
            .iter()
            .cloned()
            .map(|mut t| {
                t.turn.player = 1;
                t.message = None;
                t
            })
            .collect(),
    );
    l.run(
        1,
        &[Action::Key(Hotkey::EndTurn), Action::KeyUp(Hotkey::EndTurn)],
    )
    .unwrap();
    // A week's start without its box laid out: its sound alone.
    assert!(l.boxed.is_none());
    let lines = lines(&[first, l.state(1)], &BTreeMap::new());
    assert!(lines.contains(&"1 date 1 2 1".to_string()), "{lines:?}");
    assert!(lines.contains(&"1 player 1".to_string()), "{lines:?}");
}

#[test]
fn a_turns_table_missing_or_wrong_is_none() {
    let (_, _, _, live) = TURNING[2];
    let table: toml::Table = live.parse().unwrap();
    let turn = table["turn"].as_array().unwrap()[0]
        .as_table()
        .unwrap()
        .clone();
    assert_eq!(Turn::read(&toml::Value::Integer(1)), None);
    let value = |text: &str| -> toml::Value {
        let t: toml::Table = toml::from_str(&format!("v = {text}")).unwrap();
        t["v"].clone()
    };
    for key in ["at", "player", "kingdom", "status", "week", "month"] {
        let mut bad = turn.clone();
        bad.remove(key);
        assert_eq!(Turn::read(&toml::Value::Table(bad)), None, "without {key}");
    }
    for (key, text) in [
        ("at", "-1"),
        ("player", "-1"),
        ("kingdom", "1"),
        ("status", "1"),
        ("week", "1"),
        ("week", "[0, \"x\"]"),
        ("week", "[\"x\", 0]"),
        ("week", "[1, -1]"),
    ] {
        let mut bad = turn.clone();
        bad.insert(key.into(), value(text));
        assert_eq!(Turn::read(&toml::Value::Table(bad)), None, "{key} = {text}");
    }
}

#[test]
fn the_turns_tables_read() {
    // `[[turn]]` (E24 M4 T1a): none for a kind or a number out of range.
    let (_, _, _, live) = TURNING[2];
    let table: toml::Table = live.parse().unwrap();
    let mut turn = table["turn"].as_array().unwrap()[0].clone();
    let read = Turn::read(&turn).unwrap();
    assert_eq!((read.at, read.player), (160, 0));
    assert_eq!(read.week, crate::live::Calendar::Named(4));
    for (key, value) in [
        ("week", "[3, 0]"),
        ("week", "[0, -1]"),
        ("month", "[0]"),
        ("player", "8"),
    ] {
        let mut bad = turn.as_table().unwrap().clone();
        bad.insert(
            key.into(),
            toml::from_str(&format!("v = {value}"))
                .map(|t: toml::Table| t["v"].clone())
                .unwrap(),
        );
        assert_eq!(Turn::read(&toml::Value::Table(bad)), None, "{key} {value}");
    }
    let table = turn.as_table_mut().unwrap();
    table.insert("week".into(), toml::Value::Array(vec![2.into(), 0.into()]));
    table.insert("month".into(), toml::Value::Array(vec![1.into(), 4.into()]));
    let read = Turn::read(&turn).unwrap();
    assert_eq!(
        (read.week, read.month),
        (
            crate::live::Calendar::Special,
            crate::live::Calendar::Creature(4)
        )
    );
    let _: Option<HeroStatus> = None;
}

#[test]
fn a_record_s_first_turn_played_at_its_start() {
    // E24 M4 T6 (the reviewer's R1-1): a record whose viewer does not act first starts in another
    // player's turn; with nothing of it in sight the viewer's own first turn starts at once, at 0
    // (the date's window asked, its hero chosen), and End Turn then shows the next one.
    let day = |day| Date {
        month: 1,
        week: 1,
        day,
    };
    let mut first = turn_on(day(1), Selected::Hero(0));
    first.first = true;
    let mut l = ended(
        false,
        vec![first, turn_on(day(2), Selected::Hero(0))],
        false,
    );
    assert_eq!((l.next_turn, l.asked, l.state(0).date), (1, 1, day(1)));
    assert!(l.idle() && l.index == Some(0));
    l.run(
        1,
        &[Action::Key(Hotkey::EndTurn), Action::KeyUp(Hotkey::EndTurn)],
    )
    .unwrap();
    assert_eq!((l.next_turn, l.state(1).date), (2, day(2)));
}

#[test]
fn a_day_s_hero_and_choices_by_the_record_s_list() {
    // E24 M4 T6 (the reviewer's R1-4): the turn's list gives each slot's map hero (the record's
    // list at the day's start: here the two heroes' places swapped, a third hired with no map
    // hero), so its selection is the slot's hero by it, not by the start's list; the choices
    // follow it, the record player's slots too.
    let day = Date {
        month: 1,
        week: 1,
        day: 2,
    };
    let mut next = turn_on(day, Selected::Hero(0));
    next.list = Some(vec![Some(1), Some(0), None]);
    let mut l = ended(false, vec![next], false);
    l.heroes = vec![l.hero.unwrap(); 2];
    l.heroes[1].at = Pos { x: 30, y: 30, z: 0 };
    l.chosen.heroes = vec![l.chosen.heroes[0].clone(), l.chosen.heroes[0].clone()];
    if let Some(Some(c)) = l.chosen.heroes.get_mut(1) {
        c.hero = 1;
    }
    assert_eq!((l.slot_of(0), l.slot_of(1)), (Some(0), Some(1)));
    l.run(
        1,
        &[Action::Key(Hotkey::EndTurn), Action::KeyUp(Hotkey::EndTurn)],
    )
    .unwrap();
    assert_eq!(l.index, Some(1));
    assert_eq!(l.hero.map(|h| h.at), Some(Pos { x: 30, y: 30, z: 0 }));
    let heroes: Vec<Option<usize>> = l
        .chosen
        .heroes
        .iter()
        .map(|c| c.as_ref().map(|c| c.hero))
        .collect();
    assert_eq!(heroes, [Some(1), Some(0), None]);
    assert_eq!((l.slot_of(0), l.slot_of(1)), (Some(1), Some(0)));
}
