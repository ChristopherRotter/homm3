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
    let auto = Auto::of(&[switch(0, 0), stop(120, 1)], &starts);
    assert!(auto.plays(0, 0) && !auto.from_start(0));
    assert!(auto.plays(1, 100) && auto.plays(1, 249));
    assert!(!auto.played(2) && !auto.played(3));
    let at_start = Auto::of(&[switch(0, 0), stop(100, 1)], &starts);
    assert!(at_start.played(1) && !at_start.played(2));
    // A stop while it does not play does nothing; a second switch while it plays neither.
    let idle = Auto::of(&[stop(5, 0), switch(110, 1), switch(120, 1)], &starts);
    assert!(!idle.played(0));
    assert!(idle.plays(1, 110) && !idle.plays(1, 109) && !idle.from_start(1));
    assert!(idle.from_start(2) && idle.from_start(3));
}

#[test]
fn it_can_be_switched_on_again_and_the_last_turn_takes_the_late_inputs() {
    // Stopped by a press in the opening's, so turn 1 is a human's again: switched on in it.
    let starts = [0, 100, 250];
    let inputs = [switch(0, 0), stop(10, 0), switch(150, 1), stop(900, 5)];
    let auto = Auto::of(&inputs, &starts);
    assert!(auto.played(0));
    assert!(!auto.plays(1, 149) && auto.plays(1, 150) && !auto.from_start(1));
    // The last turn takes every input naming a turn after it: the stop in turn 5 stops a turn to
    // come.
    assert!(auto.from_start(2));
    // An input naming an earlier turn than the one before it is in that one's.
    let late = Auto::of(&[stop(5, 1), switch(120, 0)], &starts);
    assert!(!late.played(0) && late.plays(1, 120) && late.from_start(2));
    assert_eq!(Auto::default(), Auto::of(&[], &[]));
}

#[test]
fn a_switch_whose_command_takes_no_time_is_in_the_turn_it_ended() {
    // T8b2, R5c-1: a human's tactics phase, its turn from 0; A at 500 gives the bot's
    // `EndTactics`, which draws no frame, so round 1's first turn starts at 500 too. The switch
    // is the phase's turn's: round 1's first turn plays itself from its start (by the time it
    // shares with that start it would be that turn's, which then waited for the human).
    let auto = Auto::of(&[switch(500, 0)], &[0, 500, 900]);
    assert!(auto.played(0) && !auto.plays(0, 499) && !auto.from_start(0));
    assert!(auto.from_start(1) && auto.plays(1, 500) && auto.from_start(2));
    // A press in round 1's first turn (as it starts, the same millisecond) stops it as the next
    // starts.
    let stopped = Auto::of(&[switch(500, 0), stop(500, 1)], &[0, 500, 900]);
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
