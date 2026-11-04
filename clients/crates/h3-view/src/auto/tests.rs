use super::*;

/// A switch at `at` in turn `turn`.
const fn switch(at: u32, turn: usize) -> Input {
    Input::Auto { at, turn }
}

/// A stop pressed at `at` in turn `turn`.
const fn stop(at: u32, turn: usize) -> Input {
    Input::Stop { at, turn }
}

#[test]
fn the_switch_plays_its_turn_from_then_and_the_turns_after_from_their_start() {
    // Turns at 0, 100, 250, 400: switched on at 30 in the opening's.
    let auto = Auto::of(
        &[Input::Move { at: 10, to: [0, 0] }, switch(30, 0)],
        &[0, 100, 250, 400],
        &[],
    );
    assert!(!auto.plays(0, 29) && auto.plays(0, 30));
    assert!(auto.played(0) && !auto.from_start(0));
    for (k, start) in [(1, 100), (2, 250), (3, 400)] {
        assert!(auto.plays(k, start) && auto.played(k) && auto.from_start(k));
    }
    // No such turn.
    assert!(!auto.plays(4, 500) && !auto.played(4) && !auto.from_start(4));
}

#[test]
fn a_press_while_it_plays_stops_it_as_the_next_turn_starts() {
    // On at 0 in the opening's; a press in turn 1 stops it at turn 2's start, 250. A press at a
    // turn's very start is that turn's: the next one stops.
    let starts = [0, 100, 250, 400];
    let auto = Auto::of(&[switch(0, 0), stop(120, 1)], &starts, &[]);
    assert!(auto.plays(0, 0) && !auto.from_start(0));
    assert!(auto.plays(1, 100) && auto.plays(1, 249));
    assert!(!auto.played(2) && !auto.played(3));
    let at_start = Auto::of(&[switch(0, 0), stop(100, 1)], &starts, &[]);
    assert!(at_start.played(1) && !at_start.played(2));
    // A stop while it does not play does nothing; a second switch while it plays neither.
    let idle = Auto::of(&[stop(5, 0), switch(110, 1), switch(120, 1)], &starts, &[]);
    assert!(!idle.played(0));
    assert!(idle.plays(1, 110) && !idle.plays(1, 109) && !idle.from_start(1));
    assert!(idle.from_start(2) && idle.from_start(3));
}

#[test]
fn it_can_be_switched_on_again_and_the_last_turn_takes_the_late_inputs() {
    // Stopped by a press in the opening's, so turn 1 is a human's again: switched on in it.
    let starts = [0, 100, 250];
    let inputs = [switch(0, 0), stop(10, 0), switch(150, 1), stop(900, 5)];
    let auto = Auto::of(&inputs, &starts, &[]);
    assert!(auto.played(0));
    assert!(!auto.plays(1, 149) && auto.plays(1, 150) && !auto.from_start(1));
    // The last turn takes every input naming a turn after it: the stop in turn 5 stops a turn to
    // come.
    assert!(auto.from_start(2));
    // An input naming an earlier turn than the one before it is in that one's.
    let late = Auto::of(&[stop(5, 1), switch(120, 0)], &starts, &[]);
    assert!(!late.played(0) && late.plays(1, 120) && late.from_start(2));
    assert_eq!(Auto::default(), Auto::of(&[], &[], &[]));
}

#[test]
fn a_switch_whose_command_takes_no_time_is_in_the_turn_it_ended() {
    // T8b2, R5c-1: a human's tactics phase, its turn from 0; A at 500 gives the bot's
    // `EndTactics`, which draws no frame, so round 1's first turn starts at 500 too. The switch
    // is the phase's turn's: round 1's first turn plays itself from its start (by the time it
    // shares with that start it would be that turn's, which then waited for the human).
    let auto = Auto::of(&[switch(500, 0)], &[0, 500, 900], &[]);
    assert!(auto.played(0) && !auto.plays(0, 499) && !auto.from_start(0));
    assert!(auto.from_start(1) && auto.plays(1, 500) && auto.from_start(2));
    // A press in round 1's first turn (as it starts, the same millisecond) stops it as the next
    // starts.
    let stopped = Auto::of(&[switch(500, 0), stop(500, 1)], &[0, 500, 900], &[]);
    assert!(stopped.from_start(1) && !stopped.played(2));
}

#[test]
fn an_input_is_in_the_turn_started_last_by_its_time() {
    let turns = [(100, 2), (250, 0), (250, 3)];
    assert_eq!(turn_at(&turns, 0), 0);
    assert_eq!(turn_at(&turns, 99), 0);
    assert_eq!(turn_at(&turns, 100), 1);
    // Two turns starting at the same time: the later.
    assert_eq!(turn_at(&turns, 250), 3);
    assert_eq!(turn_at(&turns, 9_999), 3);
    assert_eq!(turn_at(&[], 50), 0);
}

/// The harness's boxes (`Settings::CAPTURE`): Creatures alone.
const CREATURES_ONLY: [bool; 5] = [true, false, false, false, false];

#[test]
fn each_stack_plays_itself_by_its_own_box() {
    // E17 M6 T4b1 (design §12.13, `stack_automatic`'s case table 0x474D64): the Catapult (145) by
    // Combat Catapult, the Ballista (146) and an arrow tower (149) by Combat Ballista, the First
    // Aid Tent (147) by Combat First Aid Tent, any other (the Ammo Cart 148 too, an unknown one)
    // by Combat Auto Creatures; Combat Auto Spells is no stack's.
    let cases = [
        (None, CREATURES),
        (Some(13), CREATURES),
        (Some(148), CREATURES),
        (Some(145), CATAPULT_BOX),
        (Some(146), BALLISTA_BOX),
        (Some(149), BALLISTA_BOX),
        (Some(147), TENT_BOX),
    ];
    for (creature, own) in cases {
        for set in 0..5 {
            let mut boxes = [false; 5];
            boxes[set] = true;
            assert_eq!(
                plays_itself(creature, boxes),
                set == own,
                "{creature:?} {set}"
            );
            let mut all_but = [true; 5];
            all_but[set] = false;
            assert_eq!(
                plays_itself(creature, all_but),
                set != own,
                "{creature:?} {set}"
            );
        }
        assert!(!plays_itself(creature, [false, true, false, false, false]));
    }
    // A machine without its hero's skill (the engine plays it) and the computer's: whatever the
    // boxes; a commanded Ballista (its hero's Artillery) by its box.
    assert!(Acting::Itself.plays_itself([false; 5]));
    assert!(!Acting::Human(Some(146)).plays_itself(CREATURES_ONLY));
    assert!(Acting::Human(Some(146)).plays_itself([false, false, false, true, false]));
    assert!(Acting::Human(Some(13)).plays_itself(CREATURES_ONLY));
}

#[test]
fn the_boxes_say_from_when_each_turn_plays_itself() {
    // The opening's a creature, then the computer's turn, a commanded Ballista, a Catapult, the
    // Ballista again; Creatures alone set as the battle begins.
    let stacks = [
        Acting::Human(Some(13)),
        Acting::Itself,
        Acting::Human(Some(146)),
        Acting::Human(Some(145)),
        Acting::Human(Some(146)),
    ];
    assert_eq!(
        itself(&stacks, CREATURES_ONLY, &[]),
        [Some(0), Some(0), None, None, None]
    );
    // Combat Ballista set by a close at 260 in the Ballista's turn: from then, and the turns
    // after; Creatures cleared at 300 in turn 3 (the Catapult's, which its box leaves the
    // human's): the Ballista's next turn still plays itself.
    let ballista = [true, false, false, true, false];
    let changes = [
        (2, 260, ballista),
        (3, 300, [false, false, false, true, false]),
    ];
    assert_eq!(
        itself(&stacks, CREATURES_ONLY, &changes),
        [Some(0), Some(0), Some(260), None, Some(0)]
    );
    // Set then cleared in the same turn: never; cleared then set: from the set.
    let flip = [(2, 260, ballista), (2, 270, CREATURES_ONLY)];
    assert_eq!(itself(&stacks, CREATURES_ONLY, &flip)[2], None);
    let flop = [(0, 5, [false; 5]), (0, 9, CREATURES_ONLY)];
    assert_eq!(
        itself(&stacks, CREATURES_ONLY, &flop)[..2],
        [Some(9), Some(0)]
    );
    // Set twice in the same turn: from the first.
    let twice = [(2, 260, ballista), (2, 280, [true; 5])];
    assert_eq!(itself(&stacks, CREATURES_ONLY, &twice)[2], Some(260));
    let unset = [
        (0, 5, [false; 5]),
        (0, 9, CREATURES_ONLY),
        (0, 12, [false; 5]),
    ];
    assert_eq!(
        itself(&stacks, CREATURES_ONLY, &unset)[..2],
        [None, Some(0)]
    );
}

#[test]
fn over_a_stack_it_does_not_play_the_turn_is_the_human_s_and_the_flag_stays_on() {
    // E17 M6 T4b1: switched on at 30 over a creature; the computer's turn at 100; a commanded
    // Ballista's at 250 (its box off): the human's, Auto Combat still on; a creature's at 400 plays
    // itself from its start.
    let starts = [0, 100, 250, 400];
    let itself = [Some(0), Some(0), None, Some(0)];
    let auto = Auto::of(&[switch(30, 0)], &starts, &itself);
    assert!(auto.plays(0, 30) && auto.from_start(1));
    assert!(!auto.played(2) && !auto.from_start(2));
    assert!(auto.on_at_start(2) && auto.on(2, 250) && auto.on(2, u32::MAX));
    assert!(auto.from_start(3));
    // Its click there switches it off (a toggle): the turns after are the human's; again, on,
    // still the human's turn (its stack does not play itself), the next plays itself.
    let off = Auto::of(&[switch(30, 0), switch(300, 2)], &starts, &itself);
    assert!(off.on(2, 299) && !off.on(2, 300) && !off.played(2));
    assert!(!off.on_at_start(3) && !off.played(3));
    let again = Auto::of(
        &[switch(30, 0), switch(300, 2), switch(320, 2)],
        &starts,
        &itself,
    );
    assert!(!again.on(2, 319) && again.on(2, 320) && !again.played(2));
    assert!(again.from_start(3));
    // Switched on over a stack that does not play itself: on, the turn the human's.
    let late = Auto::of(&[switch(260, 2)], &starts, &itself);
    assert!(!late.played(0) && !late.on(2, 259) && late.on(2, 260) && !late.played(2));
    assert!(late.from_start(3));
}

#[test]
fn a_press_before_a_turn_it_does_not_play_is_gone_and_the_flag_stays_on() {
    // CLT-Q45 (2): a press in the computer's turn (120) before the commanded Ballista's (250,
    // the human's): that turn's start reads no stop (0x4741F0), the press is dropped (the
    // original's acts there), Auto Combat stays on and the creature's turn at 400 plays itself.
    let starts = [0, 100, 250, 400];
    let itself = [Some(0), Some(0), None, Some(0)];
    let auto = Auto::of(&[switch(30, 0), stop(120, 1)], &starts, &itself);
    assert!(auto.stopped(1) && !auto.stopped(2));
    assert!(auto.on_at_start(2) && !auto.played(2) && auto.from_start(3));
    // Before a turn it plays, the same press stops it there.
    let plays = Auto::of(&[switch(30, 0), stop(120, 1)], &starts, &[Some(0); 4]);
    assert!(!plays.on_at_start(2) && !plays.played(2) && !plays.played(3));
    // A press after the human's command on a turn it does not play (while on) stops it as the
    // next turn it plays starts.
    let after = Auto::of(&[switch(30, 0), stop(270, 2)], &starts, &itself);
    assert!(after.stopped(2) && !after.on_at_start(3) && !after.played(3));
    // Switched off there first: no stop.
    let off = Auto::of(
        &[switch(30, 0), switch(260, 2), stop(270, 2)],
        &starts,
        &itself,
    );
    assert!(!off.stopped(2) && !off.on_at_start(3));
}

#[test]
fn a_box_set_under_auto_combat_hands_the_turn_under_way_over() {
    // The original's oddity 4 (0x4744C0): on over the Ballista's turn (250, its box off), the
    // dialog's close at 270 sets it: the computer plays the turn from then.
    let starts = [0, 100, 250, 400];
    let boxed = [Some(0), Some(0), Some(270), Some(0)];
    let auto = Auto::of(&[switch(30, 0)], &starts, &boxed);
    assert!(!auto.plays(2, 269) && auto.plays(2, 270) && !auto.from_start(2));
    // Switched off before the close: the box set changes nothing until it is switched on again,
    // from then.
    let off = Auto::of(&[switch(30, 0), switch(260, 2)], &starts, &boxed);
    assert!(!off.played(2) && !off.on_at_start(3));
    let on = Auto::of(
        &[switch(30, 0), switch(260, 2), switch(290, 2)],
        &starts,
        &boxed,
    );
    assert!(!on.plays(2, 289) && on.plays(2, 290) && on.from_start(3));
    // A switch after the close while on: the turn was handed over at the close.
    let handed = Auto::of(&[switch(30, 0), switch(280, 2)], &starts, &boxed);
    assert!(handed.plays(2, 270) && handed.on(2, u32::MAX));
}
