//! A hot seat's hand-over (E24 M5 T5, design §5.31): the recorded `seat-*` sequences replayed
//! (their timelines and their samples as recorded), then the cover's ways one by one.

use super::turns::laid;
use super::*;

use h3_ui::message::Style;

use crate::live::Turn;

/// The recorded hand-overs: their situations, timelines, kept live parts and samples.
const SEATS: [(&str, &str, &str, &str, &str); 9] = [
    (
        "seat-hand-over",
        include_str!("../../../../../map/situations/seq-seat.toml"),
        include_str!("../../../../../map/timelines/seat-hand-over.txt"),
        include_str!("../../../../../map/timelines/seat-hand-over-live.toml"),
        include_str!("../../../../../map/timelines/seat-hand-over-sounds.txt"),
    ),
    (
        "seat-hand-over-late",
        include_str!("../../../../../map/situations/seq-seat.toml"),
        include_str!("../../../../../map/timelines/seat-hand-over-late.txt"),
        include_str!("../../../../../map/timelines/seat-hand-over-late-live.toml"),
        include_str!("../../../../../map/timelines/seat-hand-over-late-sounds.txt"),
    ),
    (
        "seat-new-day",
        include_str!("../../../../../map/situations/seq-seat.toml"),
        include_str!("../../../../../map/timelines/seat-new-day.txt"),
        include_str!("../../../../../map/timelines/seat-new-day-live.toml"),
        include_str!("../../../../../map/timelines/seat-new-day-sounds.txt"),
    ),
    (
        "seat-new-day-quick",
        include_str!("../../../../../map/situations/seq-seat.toml"),
        include_str!("../../../../../map/timelines/seat-new-day-quick.txt"),
        include_str!("../../../../../map/timelines/seat-new-day-quick-live.toml"),
        include_str!("../../../../../map/timelines/seat-new-day-quick-sounds.txt"),
    ),
    (
        "seat-new-day-ai",
        include_str!("../../../../../map/situations/seq-seat-ai.toml"),
        include_str!("../../../../../map/timelines/seat-new-day-ai.txt"),
        include_str!("../../../../../map/timelines/seat-new-day-ai-live.toml"),
        include_str!("../../../../../map/timelines/seat-new-day-ai-sounds.txt"),
    ),
    (
        "seat-new-week",
        include_str!("../../../../../map/situations/seq-seat-week.toml"),
        include_str!("../../../../../map/timelines/seat-new-week.txt"),
        include_str!("../../../../../map/timelines/seat-new-week-live.toml"),
        include_str!("../../../../../map/timelines/seat-new-week-sounds.txt"),
    ),
    (
        "seat-new-month",
        include_str!("../../../../../map/situations/seq-seat-month.toml"),
        include_str!("../../../../../map/timelines/seat-new-month.txt"),
        include_str!("../../../../../map/timelines/seat-new-month-live.toml"),
        include_str!("../../../../../map/timelines/seat-new-month-sounds.txt"),
    ),
    (
        "seat-timed-event",
        include_str!("../../../../../map/situations/seq-seat-event.toml"),
        include_str!("../../../../../map/timelines/seat-timed-event.txt"),
        include_str!("../../../../../map/timelines/seat-timed-event-live.toml"),
        include_str!("../../../../../map/timelines/seat-timed-event-sounds.txt"),
    ),
    (
        "seat-start",
        include_str!("../../../../../map/situations/seq-seat-start.toml"),
        include_str!("../../../../../map/timelines/seat-start.txt"),
        include_str!("../../../../../map/timelines/seat-start-live.toml"),
        include_str!("../../../../../map/timelines/seat-start-sounds.txt"),
    ),
];

/// The samples' lengths on the harness's clock (the recordings').
const LENGTHS: [(&str, u32); 5] = [
    ("NewWeek.wav", 5759),
    ("newweek.wav", 5759),
    ("newday.wav", 3739),
    ("newmonth.wav", 4784),
    ("button.wav", 151),
];

/// The cover's sound's length.
const COVER: u32 = 5759;

/// A recorded hand-over's start as the binary's `for_sequence` and `Painter::hot_seat` make it:
/// the live part's kingdom, heroes, selection and turns; each human's turn covered (its seat's
/// selection's window, a computer's colour between it and the human before), its first hero
/// selected and its heroes by the colour; a box of each human's; the cover's sound's length, the
/// clock, and the game's start covered with `start`.
fn seated(
    id: &str,
) -> (
    Start,
    BTreeMap<u32, Vec<Action>>,
    u32,
    &'static str,
    &'static str,
) {
    let &(_, situation, timeline, live, sounds) = SEATS.iter().find(|s| s.0 == id).unwrap();
    let sequences = Sequences::parse(SEQUENCES).unwrap();
    let sequence = sequences.get(id).unwrap();
    let mut start = start(situation, sequence.scroll_speed().unwrap());
    let situated = Situation::parse(situation).unwrap();
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
    if let Selected::Hero(slot) = start.kingdom.selected {
        start.index = Some(index_of(slot));
        start.hero = Some(start.heroes[index_of(slot)]);
    }
    if let Some(Status::Hero(status)) = start.kingdom.status.clone() {
        start.chosen.heroes = vec![Some(ChosenHero {
            hero: index_of(0),
            status,
            path: Vec::new(),
        })];
    }
    start.player = viewer;
    // The mouse-over's facts (the binary's): the viewer's heroes by its list.
    start.hover = Some(computers::hover_of(live));
    start.question = Some(laid("End?", Style::OkCancel));
    start.sound = Some(crate::sound::tests::sound(crate::sound::tests::tiles(
        36,
        |_, _| 2,
        |_, _| None,
    )));
    let names = sequence.names().unwrap().unwrap();
    let mut colours: Vec<u8> = heroes.iter().map(|h| h.owner).collect();
    colours.sort_unstable();
    colours.dedup();
    let humans: Vec<u8> = colours.iter().copied().take(names.len()).collect();
    let seats: &[toml::Value] = table
        .get("seat")
        .and_then(toml::Value::as_array)
        .map_or(&[], Vec::as_slice);
    let mut before = viewer;
    start.turns = table
        .get("turn")
        .and_then(toml::Value::as_array)
        .map_or(&[][..], Vec::as_slice)
        .iter()
        .map(|t| {
            let mut turn = Turn::read(t).unwrap();
            let player = turn.player;
            let seat = seats
                .iter()
                .position(|s| s["at"].as_integer() == Some(i64::from(turn.at)));
            let selection = seat
                .and_then(|k| seats[k].get("selection"))
                .map(|s| Status::read(s).unwrap());
            let computers = (1..8)
                .map(|k| (before + k) % 8)
                .take_while(|&c| c != player)
                .any(|c| colours.contains(&c) && !humans.contains(&c));
            before = player;
            if turn.kingdom.selected == Selected::None {
                turn.kingdom.selected = Selected::Hero(0);
            }
            let list = (player != viewer).then(|| {
                heroes
                    .iter()
                    .enumerate()
                    .filter(|(_, h)| h.owner == player)
                    .map(|(i, _)| Some(i))
                    .collect()
            });
            let week = turn::starts_week(turn.kingdom.date);
            // The situation's timed events for the player that day (the binary's from the map).
            let date = turn.kingdom.date;
            let day = (date.month - 1) * 28 + (date.week - 1) * 7 + date.day;
            let events = situated
                .events
                .iter()
                .filter(|e| e.day == day && e.players.contains(&player))
                .map(|e| (laid(&e.message, Style::Ok), e.resources))
                .collect();
            NextTurn {
                message: week.then(|| laid("Week of the Rat.", Style::Ok)),
                turn,
                heroes: Vec::new(),
                walks: Vec::new(),
                taken: Vec::new(),
                flags: Vec::new(),
                lost: Vec::new(),
                boats: Vec::new(),
                list,
                first: false,
                open: false,
                cover: Some(Cover {
                    seat,
                    computers,
                    selection,
                    events,
                    opening: false,
                }),
            }
        })
        .collect();
    if sequence.start {
        let kingdom = start.kingdom.clone();
        let opening = NextTurn {
            turn: Turn {
                at: 0,
                player: viewer,
                status: kingdom.status.clone().unwrap(),
                kingdom,
                week: crate::live::Calendar::Named(0),
                month: crate::live::Calendar::Named(0),
                heroes: None,
                boats: None,
            },
            message: None,
            heroes: Vec::new(),
            walks: Vec::new(),
            taken: Vec::new(),
            flags: Vec::new(),
            lost: Vec::new(),
            boats: Vec::new(),
            list: None,
            first: true,
            open: false,
            cover: Some(Cover {
                opening: true,
                ..Cover::default()
            }),
        };
        start.turns.insert(0, opening);
    }
    start.seat = Some(HandOver {
        boxes: humans
            .iter()
            .map(|&c| (c, laid("A human's turn.", Style::Ok)))
            .collect(),
        sample: Some(COVER),
        timed: sequence.timed(),
        framed: None,
    });
    (
        start,
        sequence.actions().unwrap(),
        sequence.ms,
        timeline,
        sounds,
    )
}

/// The one-shot samples of `cues` as `<ms> <length> <loops>`, each millisecond's in order.
fn samples(cues: &[(u32, Cue)]) -> Vec<String> {
    let mut out: Vec<(u32, u32)> = cues
        .iter()
        .filter_map(|(t, cue)| match cue {
            Cue::Effect(name) | Cue::Cover(name) => {
                let length = LENGTHS.iter().find(|(n, _)| n == name).unwrap().1;
                Some((*t, length))
            }
            _ => None,
        })
        .collect();
    out.sort_unstable();
    out.iter()
        .map(|(t, l)| alloc::format!("{t} {l} 1"))
        .collect()
}

/// A recording's samples without their channels, each millisecond's in order (a game's start's
/// cover at −1 the viewer's 0, its Begin's click before it left out).
fn recorded_samples(text: &str) -> Vec<String> {
    let mut out: Vec<(u32, u32)> = text
        .lines()
        .filter_map(|line| {
            let n: Vec<i64> = line.split(' ').map(|f| f.parse().unwrap()).collect();
            let t = match n[0] {
                -1 => 0,
                t => u32::try_from(t).ok()?,
            };
            Some((t, u32::try_from(n[2]).unwrap()))
        })
        .collect();
    out.sort_unstable();
    out.iter()
        .map(|(t, l)| alloc::format!("{t} {l} 1"))
        .collect()
}

#[test]
fn replays_the_recorded_hand_overs() {
    for (id, ..) in SEATS {
        let (start, actions, ms, timeline, sounds) = seated(id);
        let (states, cues) = play_heard(rules(Bugs::Original), &start, &actions, ms).unwrap();
        let got = lines(&states, &BTreeMap::new());
        let want = without_screens(timeline);
        for (n, (g, w)) in got.iter().zip(&want).enumerate() {
            assert_eq!(g, w, "{id}: line {n}");
        }
        assert_eq!(got.len(), want.len(), "{id}");
        assert_eq!(samples(&cues), recorded_samples(sounds), "{id}");
    }
}

/// `id`'s loop run to `until` with its actions.
fn seat_run_to(start: &Start, id: &str, until: u32) -> Loop {
    let (_, actions, ..) = seated(id);
    let mut l = Loop::start(rules(Bugs::Original), start).unwrap();
    for t in 1..=until {
        l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
            .unwrap();
    }
    l
}

#[test]
fn the_cover_shows_the_new_colour_over_the_map_s_corner() {
    // `seat-hand-over` at 160: Blue's cover (the view from (0, 0), the view's corner kept, the
    // frame's picture Red's, the date and the quest log Blue's, the bar blank), its box, the
    // arrow; Blue's kingdom with no selection; Red's clock stopped.
    let (start, ..) = seated("seat-hand-over");
    let l = seat_run_to(&start, "seat-hand-over", 160);
    let s = l.state(160);
    let cover = s.drawn.cover.unwrap();
    assert_eq!(
        (cover.frame, cover.resources, cover.started),
        (0, None, false)
    );
    assert_eq!(s.drawn.view, ViewAt { x: 0, y: 0, z: 0 });
    assert_eq!(s.view, ViewAt { x: 14, y: 0, z: 0 });
    assert_eq!(s.drawn.seat, Some(0));
    assert_eq!(
        (s.player, s.shown, s.hero, s.points),
        (1, Some(1), None, None)
    );
    assert_eq!((s.cursor.kind, s.cursor.frame), (DEFAULT_CURSOR, 0));
    assert!(s.drawn.message.is_some() && s.drawn.hint == Hint::Empty);
    // The OK at 660 waits 4 000 ms (NewWeek.wav's end at 5 919 is later); nothing changes.
    let l = seat_run_to(&start, "seat-hand-over", 4659);
    assert!(l.state(4659).drawn.cover.is_some() && l.handing.is_some());
    let l = seat_run_to(&start, "seat-hand-over", 4660);
    let s = l.state(4660);
    assert!(s.drawn.cover.is_none() && l.handing.is_none());
    assert_eq!(s.hero.map(|h| h.at), Some(Pos { x: 30, y: 30, z: 0 }));
    assert_eq!(l.framed, 1);
}

#[test]
fn input_during_the_wait_is_taken_at_the_start() {
    // The pointer's moves and a key given while the start waits for the cover's sound: the point
    // read at once, the messages kept and taken in the start's millisecond.
    let (start, ..) = seated("seat-hand-over");
    let mut l = seat_run_to(&start, "seat-hand-over", 1000);
    l.run(
        1001,
        &[Action::Pointer([650, 250]), Action::Key(Hotkey::EndTurn)],
    )
    .unwrap();
    assert_eq!(l.pointer, [650, 250]);
    assert_eq!(l.handing.as_ref().map(|h| h.queued.len()), Some(2));
    for t in 1002..=4660 {
        l.run(t, &[]).unwrap();
    }
    assert!(l.handing.is_none());
    assert_eq!(l.drawn().pressed, [panels::id::END_TURN]);
}

#[test]
fn a_cover_without_its_box_starts_at_once() {
    // No box laid out for the colour (*provisional*): the start at the loop's next turn.
    let (mut start, ..) = seated("seat-hand-over");
    if let Some(seat) = &mut start.seat {
        seat.boxes.clear();
    }
    let l = seat_run_to(&start, "seat-hand-over", 160);
    assert!(l.state(160).drawn.message.is_none());
    assert_eq!(l.handing.as_ref().and_then(|h| h.until), Some(160));
    let l = seat_run_to(&start, "seat-hand-over", 161);
    assert!(l.handing.is_none() && l.state(161).drawn.cover.is_none());
}

#[test]
fn a_timed_event_s_box_over_the_cover_then_the_start() {
    // A timed event for Blue (MCL-Q71): its box after the wait, the resources added and drawn in
    // the bar over the cover, the minimap over the shield; its close starts the turn with them.
    let (mut start, ..) = seated("seat-hand-over");
    let event = laid("An event.", Style::Ok);
    if let Some(cover) = &mut start.turns[0].cover {
        cover.events = vec![(event, [0, 0, 0, 0, 0, 0, 500])];
    }
    let mut l = seat_run_to(&start, "seat-hand-over", 4660);
    let s = l.state(4660);
    let cover = s.drawn.cover.unwrap();
    assert!(cover.started && s.drawn.message.is_some());
    assert_eq!(cover.resources.map(|r| r[6]), Some(20_500));
    assert_eq!(s.timer, Some(false));
    l.run(4700, &[Action::BoxKey(Key::Ok)]).unwrap();
    l.run(4760, &[Action::BoxKeyUp(Key::Ok)]).unwrap();
    let s = l.state(4760);
    assert!(s.drawn.cover.is_none() && s.drawn.message.is_none());
    assert_eq!(s.drawn.kingdom.resources[6], 20_500);
    // A close with no hand-over under way goes to the idle step.
    assert_eq!(l.hand_events(4761), Ok(()));
}

#[test]
fn a_timed_turn_s_clock_stops_at_the_hand_over_and_starts_after_it() {
    let (mut start, ..) = seated("seat-hand-over");
    if let Some(seat) = &mut start.seat {
        seat.timed = true;
    }
    let l = seat_run_to(&start, "seat-hand-over", 159);
    assert_eq!(l.state(159).timer, Some(true));
    let l = seat_run_to(&start, "seat-hand-over", 160);
    assert_eq!(l.state(160).timer, Some(false));
    let l = seat_run_to(&start, "seat-hand-over", 4660);
    let states: Vec<State> = [159, 160, 4660]
        .iter()
        .map(|&t| {
            let mut s = l.state(t);
            s.timer = Some(t != 160);
            s
        })
        .collect();
    assert_eq!(
        lines(&states, &BTreeMap::new())
            .iter()
            .filter(|l| l.contains("timer"))
            .collect::<Vec<_>>(),
        ["160 timer 0 0", "4660 timer 1 0"]
    );
}

#[test]
fn the_game_s_start_covered_keeps_its_selection() {
    // A first turn made of the live part (the opening, as `start = true`): covered at 0 with the
    // live part's kingdom and hero kept, its OK and the start without a wait (no sound).
    let (mut start, ..) = seated("seat-hand-over");
    let mut kingdom = start.kingdom.clone();
    kingdom.status = Some(Status::Date(DateStatus { frame: 0 }));
    start.kingdom.status = kingdom.status.clone();
    let opening = NextTurn {
        turn: Turn {
            at: 0,
            player: 0,
            kingdom,
            status: Status::Date(DateStatus { frame: 0 }),
            week: crate::live::Calendar::Named(0),
            month: crate::live::Calendar::Named(0),
            heroes: None,
            boats: None,
        },
        message: None,
        heroes: Vec::new(),
        walks: Vec::new(),
        taken: Vec::new(),
        flags: Vec::new(),
        lost: Vec::new(),
        boats: Vec::new(),
        list: None,
        first: true,
        open: false,
        cover: Some(Cover {
            opening: true,
            ..Cover::default()
        }),
    };
    start.turns.insert(0, opening);
    if let Some(seat) = &mut start.seat {
        seat.sample = None;
        seat.framed = Some(5);
    }
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    let s = l.state(0);
    // The frame's picture in the colours the human before's view drew it (play's).
    assert_eq!(s.drawn.cover.map(|c| c.frame), Some(5));
    assert!(s.drawn.message.is_some());
    assert_eq!((s.player, s.shown), (0, Some(0)));
    assert!(s.hero.is_some() && s.points.is_some());
    assert_eq!(s.drawn.seat, None);
    l.run(1, &[Action::BoxKey(Key::Ok)]).unwrap();
    l.run(2, &[Action::BoxKeyUp(Key::Ok)]).unwrap();
    let s = l.state(2);
    assert!(s.drawn.cover.is_none() && s.drawn.message.is_none());
    assert_eq!((s.cursor.kind, s.hero), (CURSOR_TYPE, start.hero));
}

#[test]
fn the_hand_over_s_steps_without_a_hand_over_under_way() {
    // Each step called with none under way (an OK, a wait's end, an event's close): nothing
    // but the idle step; a timed event without the cover drawn opens its box over the map.
    let (start, ..) = seated("seat-hand-over");
    let mut l = Loop::start(rules(Bugs::Original), &start).unwrap();
    assert_eq!(l.covered_ok(5), Ok(()));
    assert!(l.handing.is_none() && l.covered.is_none() && !l.timer);
    let (_, actions, ..) = seated("seat-hand-over");
    for t in 1..=160 {
        l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
            .unwrap();
    }
    l.covered = None;
    if let Some(handing) = &mut l.handing {
        handing
            .events
            .push_back((laid("An event.", Style::Ok), [0; 7]));
    }
    l.boxed = None;
    assert_eq!(l.hand_events(161), Ok(()));
    assert!(l.drawn().message.is_some() && l.drawn().cover.is_none());
}

#[test]
fn an_action_the_loop_cannot_play_given_during_the_wait_fails_at_the_start() {
    // A pointer over the view without the mouse-over's facts (a recording's refusal), given
    // while the start waits: refused in the start's millisecond, at the wait's end or at an
    // event's close; `play_heard` names its millisecond, 0 for a start the loop refuses.
    let (mut start, mut actions, ..) = seated("seat-hand-over");
    start.hover = None;
    actions.insert(1000, vec![Action::Pointer([100, 100])]);
    let failed = play_heard(rules(Bugs::Original), &start, &actions, 8000);
    assert_eq!(failed.err(), Some((4660, Unplayable::OverView([100, 100]))));
    let mut evented = start.clone();
    if let Some(cover) = &mut evented.turns[0].cover {
        cover.events = vec![(laid("An event.", Style::Ok), [0; 7])];
    }
    actions.insert(5000, vec![Action::BoxKey(Key::Ok)]);
    actions.insert(5060, vec![Action::BoxKeyUp(Key::Ok)]);
    let failed = play_heard(rules(Bugs::Original), &evented, &actions, 8000);
    assert_eq!(failed.err(), Some((5060, Unplayable::OverView([100, 100]))));
    let mut over = start;
    over.pointer = [100, 100];
    let failed = play_heard(rules(Bugs::Original), &over, &BTreeMap::new(), 10);
    assert_eq!(failed.err(), Some((0, Unplayable::OverView([100, 100]))));
}
