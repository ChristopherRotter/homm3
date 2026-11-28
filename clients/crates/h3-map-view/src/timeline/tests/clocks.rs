//! A timed turn's clock (E24 M5 T5 part five, design §5.32): the recorded `seat-clock*` sequences
//! replayed (their timelines and their samples as recorded), then the clock's other ways: a box
//! open at the turn's end, one opened late, the host's expiry, the host's time left, a warning's
//! sound held by the last one's.

use super::seats::{CLOCKS, recorded_samples, samples, seated};
use super::turns::laid;
use super::*;

use h3_ui::message::Style;

use crate::clock::Warning;

#[test]
fn replays_the_recorded_clocks() {
    for (id, ..) in CLOCKS {
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

/// `start`'s loop run to `until` with `actions`.
fn run_to(start: &Start, actions: &BTreeMap<u32, Vec<Action>>, until: u32) -> Loop {
    let mut l = Loop::start(rules(Bugs::Original), start).unwrap();
    for t in 1..=until {
        l.run(t, actions.get(&t).map_or(&[][..], Vec::as_slice))
            .unwrap();
    }
    l
}

/// The loop `l` run on from `from` to `until` with no input.
fn run_on(l: &mut Loop, from: u32, until: u32) {
    for t in from..=until {
        l.run(t, &[]).unwrap();
    }
}

/// End Turn's key down and up at `at` (its question asked: the hero can move).
fn end_turn(at: u32) -> BTreeMap<u32, Vec<Action>> {
    BTreeMap::from([
        (at, vec![Action::Key(Hotkey::EndTurn)]),
        (at + 60, vec![Action::KeyUp(Hotkey::EndTurn)]),
    ])
}

/// `seat-clock`'s start with Move Reminder's question asked.
fn reminded() -> Start {
    let (mut start, ..) = seated("seat-clock");
    start.move_reminder = true;
    start
}

#[test]
fn the_chat_holds_the_last_warnings_and_the_lines_drawn() {
    let (start, actions, ..) = seated("seat-clock");
    let l = run_to(&start, &actions, 58_320);
    assert_eq!(l.timer_at(), Some(0));
    let chat = l.chat();
    assert_eq!(chat.len(), 9);
    assert_eq!(chat[0].warning, Warning::Seconds(9));
    assert_eq!(chat[8].drawn, Some(58_320));
    let s = l.state(58_320);
    assert_eq!(s.drawn.chat.len(), 9);
    assert_eq!(s.warning, None);
}

#[test]
fn a_box_open_at_the_turn_s_end_closes_by_its_style() {
    // End Turn's question at 40 060 (20 s left: no deadline) answered No at 60 001, then the
    // expiry's box in that millisecond; its own close 15 s on, Blue's cover.
    let start = reminded();
    let l = run_to(&start, &end_turn(40_000), 60_000);
    assert!(l.boxed.as_ref().is_some_and(|b| b.told == Told::Question));
    assert_eq!(l.boxed.as_ref().unwrap().deadline, None);
    let mut l = l;
    run_on(&mut l, 60_001, 60_001);
    assert!(l.boxed.as_ref().is_some_and(|b| b.told == Told::Expired));
    assert_eq!(l.boxed.as_ref().unwrap().deadline, Some(75_001));
    run_on(&mut l, 60_002, 75_001);
    assert_eq!(l.state(75_001).shown, Some(1));
}

#[test]
fn a_box_opened_late_closes_itself_15_s_on() {
    // End Turn's question at 50 060 (9 940 ms left): its deadline 65 060, past the turn's end;
    // the expiry's box after it.
    let start = reminded();
    let mut l = run_to(&start, &end_turn(50_000), 65_059);
    assert!(l.boxed.as_ref().is_some_and(|b| b.told == Told::Question));
    assert_eq!(l.boxed.as_ref().unwrap().deadline, Some(65_060));
    run_on(&mut l, 65_060, 65_060);
    assert!(l.boxed.as_ref().is_some_and(|b| b.told == Told::Expired));
}

#[test]
fn the_host_ends_the_turn_and_closes_the_box_open() {
    let mut start = reminded();
    if let Some(clock) = &mut start.clock {
        clock.hosted = true;
    }
    let actions = end_turn(50_000);
    // The loop's own test off: no box past the limit, the question without a deadline.
    let l = run_to(&start, &actions, 61_000);
    assert!(l.boxed.as_ref().is_some_and(|b| b.told == Told::Question));
    assert_eq!(l.boxed.as_ref().unwrap().deadline, None);
    // The host's expiry at 61 000: the question gone, the expiry's box, its 15 s.
    let mut l = run_to(&start, &actions, 60_999);
    l.expire(61_000);
    l.expire(62_000);
    run_on(&mut l, 61_000, 61_000);
    assert!(l.boxed.as_ref().is_some_and(|b| b.told == Told::Expired));
    run_on(&mut l, 61_001, 76_000);
    assert_eq!(l.state(76_000).shown, Some(1));
    // With no box open, at the host's expiry; or known from the start.
    let mut l = run_to(&start, &BTreeMap::new(), 40_000);
    l.expire(40_001);
    run_on(&mut l, 40_001, 40_001);
    assert!(l.boxed.as_ref().is_some_and(|b| b.told == Told::Expired));
    if let Some(clock) = &mut start.clock {
        clock.expiry = Some(40_001);
    }
    let l = run_to(&start, &BTreeMap::new(), 40_001);
    assert!(l.boxed.as_ref().is_some_and(|b| b.told == Told::Expired));
}

#[test]
fn the_host_s_time_left_places_the_turn_s_start() {
    // 40 s left at 0: started 20 s before, the 30 s warning at the first poll past 10 000.
    let (mut start, ..) = seated("seat-clock");
    if let Some(clock) = &mut start.clock {
        clock.left = Some(40_000);
    }
    let l = run_to(&start, &BTreeMap::new(), 10_019);
    assert_eq!(l.state(10_019).warning, Some(19_981));
    // None left: the poll finds it past and gives no more.
    if let Some(clock) = &mut start.clock {
        clock.left = Some(0);
    }
    let l = run_to(&start, &BTreeMap::new(), 200);
    assert_eq!(l.turn_timer.unwrap().next, 0);
    assert!(l.chat().is_empty());
}

#[test]
fn a_warning_while_the_last_one_s_sound_plays_is_silent() {
    let (mut start, ..) = seated("seat-clock");
    if let Some(clock) = &mut start.clock {
        clock.sample = Some(2_000);
    }
    let (_, cues) = play_heard(rules(Bugs::Original), &start, &BTreeMap::new(), 53_000).unwrap();
    let at: Vec<u32> = cues
        .iter()
        .filter(|(_, c)| *c == Cue::Effect("timeover.wav".into()))
        .map(|(t, _)| *t)
        .collect();
    assert_eq!(at, [30_059, 50_039, 52_079]);
}

#[test]
fn a_message_a_walk_kept_finds_the_turn_s_end() {
    // `seat-clock-walk` with a move kept by the walk behind a hotkey's down (taken, not handled:
    // the last step ends it): taken at its end (60 500), it opens the expiry's box.
    let (start, mut actions, ..) = seated("seat-clock-walk");
    actions.insert(
        60_400,
        vec![Action::Key(Hotkey::NextHero), Action::Pointer([710, 310])],
    );
    let l = run_to(&start, &actions, 60_500);
    assert!(l.boxed.as_ref().is_some_and(|b| b.told == Told::Expired));
}

#[test]
fn the_chat_carried_over_keeps_its_lines_drawn() {
    // Play's next view (MCL-Q70): a line drawn before its 0 kept with its time, one not drawn
    // yet stamped at its first draw.
    let (mut start, ..) = seated("seat-clock");
    if let Some(clock) = &mut start.clock {
        clock.chat = vec![
            ChatLine {
                warning: Warning::Minute,
                drawn: Some(-5_000),
            },
            ChatLine {
                warning: Warning::Seconds(9),
                drawn: None,
            },
        ];
    }
    let l = Loop::start(rules(Bugs::Original), &start).unwrap();
    let drawn: Vec<Option<i64>> = l.chat().iter().map(|c| c.drawn).collect();
    assert_eq!(drawn, [Some(-5_000), Some(0)]);
    assert_eq!(l.state(0).chat, Some(vec![-5_000, 0]));
}

#[test]
fn a_box_s_timing_out_takes_its_no_its_cancel_or_its_ok() {
    let open = |style| Open::new((*laid("Q?", style)).clone());
    let timed_out = clock::timed_out;
    assert_eq!(timed_out(&open(Style::OkCancel)), boxes::NO);
    assert_eq!(timed_out(&open(Style::ChooseCancel)), boxes::CANCEL);
    assert_eq!(timed_out(&open(Style::Ok)), boxes::OK);
}

#[test]
fn the_replay_takes_the_host_s_expiry() {
    let (mut start, ..) = seated("seat-clock");
    if let Some(clock) = &mut start.clock {
        clock.hosted = true;
    }
    let mut replay =
        crate::replay::Replay::new(rules(Bugs::Original), &start, BTreeMap::new()).unwrap();
    assert!(replay.at(40_000).unwrap().drawn.message.is_none());
    assert_eq!(replay.timer_at(), Some(0));
    assert_eq!(replay.chat().len(), 1);
    replay.expire(40_001);
    assert!(replay.at(40_001).unwrap().drawn.message.is_some());
}
