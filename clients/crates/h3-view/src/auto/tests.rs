use super::*;

#[test]
fn the_switch_plays_its_turn_from_then_and_the_turns_after_from_their_start() {
    // Turns at 0, 100, 250, 400: switched on at 30 in the opening's.
    let auto = Auto::of(
        &[Input::Move { at: 10, to: [0, 0] }, Input::Auto { at: 30 }],
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
    // On at 0 in the opening's; a press at 120 (turn 1's) stops it at turn 2's start, 250. A
    // press at a turn's very start belongs to that turn: the next one stops.
    let starts = [0, 100, 250, 400];
    let auto = Auto::of(&[Input::Auto { at: 0 }, Input::Stop { at: 120 }], &starts);
    assert!(auto.plays(0, 0) && !auto.from_start(0));
    assert!(auto.plays(1, 100) && auto.plays(1, 249));
    assert!(!auto.played(2) && !auto.played(3));
    let at_start = Auto::of(&[Input::Auto { at: 0 }, Input::Stop { at: 100 }], &starts);
    assert!(at_start.played(1) && !at_start.played(2));
    // A stop while it does not play does nothing; a second switch while it plays neither.
    let idle = Auto::of(
        &[
            Input::Stop { at: 5 },
            Input::Auto { at: 110 },
            Input::Auto { at: 120 },
        ],
        &starts,
    );
    assert!(!idle.played(0));
    assert!(idle.plays(1, 110) && !idle.plays(1, 109) && !idle.from_start(1));
    assert!(idle.from_start(2) && idle.from_start(3));
}

#[test]
fn it_can_be_switched_on_again_and_the_last_turn_takes_the_late_inputs() {
    // Stopped by a press at 10 in the opening's, so turn 1 is a human's again: switched on in it.
    let starts = [0, 100, 250];
    let inputs = [
        Input::Auto { at: 0 },
        Input::Stop { at: 10 },
        Input::Auto { at: 150 },
        Input::Stop { at: 900 },
    ];
    let auto = Auto::of(&inputs, &starts);
    assert!(auto.played(0));
    assert!(!auto.plays(1, 149) && auto.plays(1, 150) && !auto.from_start(1));
    // The last turn takes every input after its start: the stop at 900 stops a turn to come.
    assert!(auto.from_start(2));
    // Two turns starting at the same time: the inputs then are the later one's.
    let same = Auto::of(&[Input::Auto { at: 100 }], &[0, 100, 100]);
    assert!(!same.played(1) && same.plays(2, 100));
    assert_eq!(Auto::default(), Auto::of(&[], &[]));
}
