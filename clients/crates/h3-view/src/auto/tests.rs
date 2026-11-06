use super::*;
use h3_render::Bugs;

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
    // E17 M6 T5b2b: the switch's turn is played from a switch (its button selected by it), the
    // turns after from their start.
    assert!(auto.switched(0, 30) && !auto.switched(0, 29));
    assert!(!auto.switched(1, 100) && !auto.switched(4, 500));
}

#[test]
fn a_switch_again_after_a_stop_at_a_cast_s_end_plays_from_a_switch() {
    // E17 M6 T5b2b: turn 1 played from its start, stopped at its cast's end (400), switched on
    // again at 500: from 500 a switch's play.
    let starts = [0, 100, 900];
    let casts = [None, cast(1, 100, 400)];
    let inputs = [switch(30, 0), stop(250, 1), switch(500, 1)];
    let auto = Auto::casting(&inputs, &starts, &[], &casts);
    assert!(!auto.switched(1, 300) && !auto.switched(1, 450) && auto.switched(1, 500));
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

#[test]
fn the_computer_casts_for_a_human_s_hero_as_a_turn_starts_under_it_with_spells_set() {
    // E17 M6 T5b1 (design §12.18, 0x422E42): asked as each turn starts with Auto Combat on (its
    // stop read) and Combat Auto Spells set, in the original's whether the stack plays itself or
    // not; in the client's (CLT-Q62) only on a turn it plays.
    let starts = [0, 100, 250, 400];
    let spells = [false, true, false, false, false];
    let both = [true, true, false, false, false];
    // On at 30 in the opening's: not as it started; the turns after it, until the stop in turn 2
    // stops it as turn 3 starts.
    let auto = Auto::of(&[switch(30, 0), stop(260, 2)], &starts, &[]);
    for bugs in [Bugs::Original, Bugs::Fixed] {
        assert!(!auto.asks_cast(0, both, bugs));
        assert!(auto.asks_cast(1, both, bugs) && auto.asks_cast(2, spells, bugs));
        assert!(!auto.asks_cast(1, CREATURES_ONLY, bugs) && !auto.asks_cast(3, both, bugs));
        // The switch's own turn: on from 30.
        assert!(auto.asks_cast_at(0, 30, spells, bugs) && !auto.asks_cast_at(0, 29, spells, bugs));
        assert!(!auto.asks_cast_at(0, 30, CREATURES_ONLY, bugs));
    }
    // Over stacks it does not play (their box off) the flag stays on: asked in the original's,
    // the stop dropped; never in the client's.
    let human = Auto::of(
        &[switch(30, 0), stop(260, 2)],
        &starts,
        &[None, None, None, None],
    );
    let original = Bugs::Original;
    assert!(human.asks_cast(1, spells, original) && human.asks_cast(3, spells, original));
    assert!(!human.asks_cast(9, spells, original));
    assert!(human.asks_cast_at(0, 30, spells, original));
    assert!(!human.asks_cast(1, spells, Bugs::Fixed) && !human.asks_cast(3, spells, Bugs::Fixed));
    assert!(!human.asks_cast_at(0, 30, spells, Bugs::Fixed));
}

#[test]
fn the_boxes_as_a_turn_starts_are_those_of_the_last_close_before_it() {
    // E17 M6 T5b1: a close in turn 2 counts from turn 3; the first boxes before any.
    let (none, all) = ([false; 5], [true; 5]);
    let changes = [
        (0, 0, CREATURES_ONLY),
        (2, 900, all),
        (2, 950, none),
        (4, 2000, all),
    ];
    assert_eq!(boxes_at(CREATURES_ONLY, &[], 7), CREATURES_ONLY);
    assert_eq!(boxes_at(none, &changes, 0), none);
    assert_eq!(boxes_at(none, &changes, 1), CREATURES_ONLY);
    assert_eq!(boxes_at(none, &changes, 2), CREATURES_ONLY);
    assert_eq!(boxes_at(all, &changes, 3), none);
    assert_eq!(boxes_at(none, &changes, 5), all);
}

/// A cast's action `action` from `start` to `end`.
const fn cast(action: usize, start: u32, end: u32) -> Option<Cast> {
    Some(Cast { action, start, end })
}

#[test]
fn a_press_during_the_computer_s_cast_stops_it_at_the_cast_s_end() {
    // E17 M6 T5b2b (CLT-Q61, design §12.20): on at 30 in the opening's; turn 1 (from 100) plays
    // itself from its start and its first action is the hero's cast, 100 to 400. A press at 250
    // in it stops Auto Combat at 400, that turn's: the human's from then, the flag off, its button
    // unselected; the turns after are the human's, no stop waiting.
    let starts = [0, 100, 600, 900];
    let casts = [None, cast(1, 100, 400)];
    let auto = Auto::casting(&[switch(30, 0), stop(250, 1)], &starts, &[], &casts);
    assert!(auto.from_start(1) && auto.plays(1, 399) && !auto.plays(1, 400));
    assert!(auto.on(1, 399) && !auto.on(1, 400) && !auto.on(1, u32::MAX));
    assert!(!auto.played(1) && !auto.stopped(1) && auto.stopped_at(1) == Some(400));
    assert!(!auto.on_at_start(2) && !auto.played(2) && !auto.played(3));
    assert_eq!(auto.handed(&casts), [(1, 400)]);
    assert_eq!((auto.stopped_at(0), auto.stopped_at(9)), (None, None));
    // At the cast's end or after it: the next turn's start, as without a cast; and so without
    // `casts` (`Auto::of`).
    for late in [
        Auto::casting(&[switch(30, 0), stop(400, 1)], &starts, &[], &casts),
        Auto::casting(&[switch(30, 0), stop(450, 1)], &starts, &[], &casts),
        Auto::of(&[switch(30, 0), stop(250, 1)], &starts, &[]),
    ] {
        assert!(late.plays(1, u32::MAX) && late.played(1) && late.stopped(1));
        assert!(late.stopped_at(1).is_none() && !late.played(2));
        assert!(late.handed(&casts).is_empty());
    }
}

#[test]
fn a_cast_auto_combat_did_not_play_reads_no_stop_at_its_end() {
    // E17 M6 T5b2b: the human's own cast at 120 on a turn it does not play, then A at 450: the
    // press at 460 waits for the next turn's start. A stop while it is off does nothing. The
    // switch's own cast (from 450) reads it at its end.
    let starts = [0, 100, 900];
    let casts = [None, cast(0, 120, 420)];
    let own = Auto::casting(&[switch(450, 1), stop(460, 1)], &starts, &[], &casts);
    assert!(own.stopped(1) && own.stopped_at(1).is_none() && own.played(1));
    let off = Auto::casting(&[stop(200, 1)], &starts, &[], &casts);
    assert!(!off.stopped(1) && off.stopped_at(1).is_none());
    let switched = [None, cast(0, 450, 750)];
    let auto = Auto::casting(&[switch(450, 1), stop(460, 1)], &starts, &[], &switched);
    assert!(auto.plays(1, 450) && !auto.plays(1, 750) && auto.stopped_at(1) == Some(750));
    assert!(!auto.from_start(1) && !auto.on(1, 750) && auto.on(1, 749));
    // Switched on again after the stop: from then, and the turns after.
    let again = Auto::casting(
        &[switch(450, 1), stop(460, 1), switch(800, 1)],
        &starts,
        &[],
        &switched,
    );
    assert!(again.plays(1, 700) && !again.plays(1, 799) && again.plays(1, 800));
    assert!(again.played(1) && again.stopped_at(1) == Some(750) && again.from_start(2));
    // A second press after it changes nothing more.
    let twice = Auto::casting(
        &[switch(450, 1), stop(460, 1), stop(700, 1)],
        &starts,
        &[],
        &switched,
    );
    assert!(!twice.stopped(1) && !twice.on_at_start(2));
}

#[test]
fn each_turn_s_cast_is_the_first_of_a_human_s_hero() {
    // E17 M6 T5b2b: the opening's turn's walk, then turn 1's cast by the attacker (a human's)
    // and its second one (another round's, not in a turn here: kept the first), turn 2's by the
    // defender (the computer's), turn 3 without an action; an action past the list counts none.
    let action = |start, end, cast| Action {
        start,
        end,
        cast,
        ..Action::default()
    };
    let actions = [
        action(0, 100, None),
        action(100, 400, Some(Side::Attacker)),
        action(400, 500, Some(Side::Attacker)),
        action(500, 800, Some(Side::Defender)),
    ];
    let log = [
        (0, Logged::Action),
        (100, Logged::Turn(1)),
        (100, Logged::Action),
        (300, Logged::Unmark),
        (400, Logged::Action),
        (500, Logged::Turn(2)),
        (500, Logged::Action),
        (800, Logged::Turn(3)),
        (800, Logged::Action),
    ];
    let human = [true, false];
    assert_eq!(
        casts(&log, &actions, human),
        [None, cast(1, 100, 400), None, None]
    );
    assert_eq!(casts(&log, &actions, [true; 2])[2], cast(3, 500, 800));
    assert_eq!(casts(&[], &actions, human), [None]);
}
