//! The mouse over time's tests (E17 M3 T6c-2, design §9.10), on the committed layout and the
//! mouse's test battle: the attacker's stack 0 acting on hex 86, the defender's 1 on 90, the
//! attacker's 2 on 120, the defender's 3 on 100.

use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use h3_protocol::wire::{Command, Observation, Side};
use h3_render::Bugs;

use super::{Input, Shown, Start, Turn, follow};
use crate::battle_log::Logged;
use crate::mouse::tests::{battle, centre, layout};
use crate::mouse::{Facing, Field, Hint, Legal, Spot, slot, slots, spot};

/// Where the harness parks the mouse: in the panel, over nothing that has a hint.
const PARK: [i32; 2] = [618, 593];

/// A hint as the tests word it.
fn name(hint: &Hint, _: &Observation) -> String {
    format!("{hint:?}")
}

fn status(text: &str, forced: bool) -> Logged {
    Logged::Status {
        text: text.into(),
        forced,
    }
}

/// The acting stack 0 may walk to 87 and strike the stack on 90 from 89 or 91.
fn legal() -> Legal {
    Legal::new(&[
        Command::Move { to: 87 },
        Command::Attack {
            target: 1,
            from: 89,
        },
        Command::Attack {
            target: 1,
            from: 91,
        },
    ])
}

fn run(log: &[(u32, Logged)], starts: &[Start], turns: &[Turn<'_>], inputs: &[Input]) -> Shown {
    follow(log, starts, turns, inputs, &layout(), &name)
}

#[test]
fn a_waiting_human_turn_takes_each_move() {
    let (o, legal) = (battle(), legal());
    let field = Field {
        observation: &o,
        legal: &legal,
        facing: &Facing::Sides,
        bugs: Bugs::Fixed,
    };
    let turns = [Turn { field, human: true }];
    let l = layout();
    // Two points of hex 90 in different slots: right of its centre the sword from 91, left of
    // it the one from 89; the same point again changes nothing.
    let (a, b) = (
        [centre(90)[0] + 10, centre(90)[1]],
        [centre(90)[0] - 10, centre(90)[1]],
    );
    let frame = |p: [i32; 2]| slots(&o, &legal, 90)[slot(&l, p, 90)];
    assert_ne!(frame(a), frame(b));
    let inputs = [
        Input::Move { at: 0, to: PARK },
        Input::Move { at: 100, to: a },
        Input::Move { at: 120, to: a },
        Input::Move { at: 150, to: b },
        Input::Move {
            at: 200,
            to: centre(87),
        },
        Input::Move {
            at: 300,
            to: [400, 580],
        },
    ];
    let shown = run(&[], &[], &turns, &inputs);
    let frames = [(0, 6), (100, frame(a)), (150, frame(b)), (200, 1), (300, 6)];
    assert_eq!(shown.frames, frames);
    assert_eq!(shown.marked, [(100, Some(1)), (200, None)]);
    assert_eq!(shown.entered, [(100, 1)]);
    // The mouse resting where it was placed before the battle enters no item: no hint at 0 (E17
    // M4 T3b); the log's item at 300 has none.
    let log = [
        (100, status("Attack(1)", true)),
        (150, status("Attack(1)", true)),
        (200, status("Move(0)", false)),
        (300, status("Empty", false)),
    ];
    assert_eq!(shown.log, log);
    assert_eq!(
        shown.points,
        [
            (0, PARK),
            (100, a),
            (150, b),
            (200, centre(87)),
            (300, [400, 580])
        ]
    );
    assert_eq!(shown.frame(99), Some(6));
    assert_eq!(shown.point(160), Some(b));
    assert_eq!(shown.marked(150), Some(1));
    // Before the mouse is anywhere: nothing.
    assert_eq!(Shown::default().frame(0), None);
    assert_eq!(Shown::default().point(0), None);
    assert_eq!(Shown::default().marked(0), None);
}

#[test]
fn an_item_writes_its_hint_only_as_the_mouse_enters_it() {
    // E17 M4 T3b (`hovered_item` 0x6AAC68, read per frame): the mouse resting where it was placed
    // before the battle enters no item; the queued move into the panel enters its picture (an
    // empty line), the next turn's post there writes nothing; a control's hint as the mouse
    // enters it, the picture's empty line as it comes back.
    let (o, legal) = (battle(), legal());
    let mut next = battle();
    next.active = Some(2);
    let none = Legal::default();
    let turn = |observation, legal| Turn {
        field: Field {
            observation,
            legal,
            facing: &Facing::Sides,
            bugs: Bugs::Fixed,
        },
        human: true,
    };
    let turns = [turn(&o, &legal), turn(&next, &none), turn(&next, &none)];
    let inputs = [
        Input::Move { at: 0, to: PARK },
        Input::Move {
            at: 100,
            to: centre(90),
        },
        Input::Act { at: 100 },
        Input::Move { at: 100, to: PARK },
        Input::Act { at: 500 },
        Input::Move {
            at: 900,
            to: [700, 570],
        },
        Input::Move { at: 950, to: PARK },
        Input::Move { at: 990, to: PARK },
    ];
    let log = [
        (100, Logged::Action),
        (500, Logged::Turn(2)),
        (500, Logged::Action),
        (900, Logged::Turn(2)),
    ];
    let shown = follow(
        &log,
        &[Start { unmarks: true }, Start::default()],
        &turns,
        &inputs,
        &layout(),
        &name,
    );
    assert_eq!(
        shown.log,
        [
            (100, status("Attack(1)", true)),
            (100, Logged::Action),
            (500, Logged::Turn(2)),
            (500, status("Empty", false)),
            (500, Logged::Action),
            (900, Logged::Turn(2)),
            (900, status("Control(\"battle.control.wait\")", false)),
            (950, status("Empty", false)),
        ]
    );
}

#[test]
fn moves_wait_while_an_action_plays() {
    let (o, legal) = (battle(), legal());
    let mut next = battle();
    next.active = Some(2);
    let none = Legal::default();
    let turns = [
        Turn {
            field: Field {
                observation: &o,
                legal: &legal,
                facing: &Facing::Sides,
                bugs: Bugs::Fixed,
            },
            human: true,
        },
        Turn {
            field: Field {
                observation: &next,
                legal: &none,
                facing: &Facing::Sides,
                bugs: Bugs::Fixed,
            },
            human: true,
        },
    ];
    // The harness's click on 90 at 100: the mouse there, the click, the mouse parked again.
    let inputs = [
        Input::Move {
            at: 0,
            to: centre(90),
        },
        Input::Move {
            at: 100,
            to: centre(90),
        },
        Input::Act { at: 100 },
        Input::Move { at: 100, to: PARK },
        // At the turn's time: after the turn starts; over Wait, its hint.
        Input::Move {
            at: 500,
            to: [700, 570],
        },
    ];
    let log = [(100, Logged::Action), (500, Logged::Turn(2))];
    let shown = run(&log, &[Start { unmarks: true }], &turns, &inputs);
    // The same hex again writes nothing; the action shows the pointer and unmarks; the parked
    // mouse waits until the next human turn, which takes it (entering the panel's picture: an
    // empty line), then posts it again (the same item: nothing; E17 M4 T3b).
    assert_eq!(
        shown.log,
        [
            (0, status("Attack(1)", true)),
            (100, Logged::Action),
            (500, Logged::Turn(2)),
            (500, status("Empty", false)),
            (500, status("Control(\"battle.control.wait\")", false)),
        ]
    );
    assert_eq!(shown.marked, [(0, Some(1)), (100, None)]);
    assert_eq!(shown.frames.last(), Some(&(100, 6)));
    assert_eq!(
        shown.points,
        [(0, centre(90)), (100, PARK), (500, [700, 570])]
    );
}

#[test]
fn a_computers_turn_and_a_round_unmark() {
    let (o, legal) = (battle(), legal());
    let mut computer = battle();
    computer.active = Some(3);
    let mut again = battle();
    again.active = Some(0);
    let none = Legal::default();
    let field = |observation, legal| Field {
        observation,
        legal,
        facing: &Facing::Sides,
        bugs: Bugs::Fixed,
    };
    let turns = [
        Turn {
            field: field(&o, &legal),
            human: true,
        },
        Turn {
            field: field(&computer, &none),
            human: false,
        },
        Turn {
            field: field(&again, &legal),
            human: true,
        },
    ];
    let inputs = [
        Input::Move {
            at: 0,
            to: centre(90),
        },
        Input::Move {
            at: 70,
            to: centre(87),
        },
        // A key that gives nothing (no action then).
        Input::Act { at: 75 },
    ];
    let log = [
        (50, Logged::Round),
        (60, Logged::Turn(3)),
        (80, Logged::Action),
        (90, Logged::Turn(0)),
    ];
    let shown = run(&log, &[Start::default()], &turns, &inputs);
    assert_eq!(shown.marked, [(0, Some(1)), (50, None)]);
    // The computer's turn shows the pointer; the mouse moved meanwhile is taken at the next
    // human turn, and posted there again over the same hex: written once.
    let attack = shown.frames[0].1;
    assert_eq!(shown.frames, [(0, attack), (60, 6), (90, 1)]);
    let statuses: Vec<_> = shown
        .log
        .iter()
        .filter(|(_, l)| matches!(l, Logged::Status { .. }))
        .collect();
    assert_eq!(
        statuses,
        [
            &(0, status("Attack(1)", true)),
            &(90, status("Move(0)", false))
        ]
    );
    // A computer's opening shows the pointer at once.
    let opening = [Turn {
        field: field(&computer, &none),
        human: false,
    }];
    assert_eq!(run(&[], &[], &opening, &[]).frames, [(0, 6)]);
}

#[test]
fn a_target_left_over_says_no_shots_left() {
    // The acting stack shoots but has no shots: it strikes the stack on 90 in melee.
    let mut o = battle();
    o.stacks[0].shooter = true;
    o.stacks[0].shots = 0;
    let legal = legal();
    let turns = [Turn {
        field: Field {
            observation: &o,
            legal: &legal,
            facing: &Facing::Sides,
            bugs: Bugs::Fixed,
        },
        human: true,
    }];
    let off = [5, 200];
    assert_eq!(spot(&layout(), off, false), Spot::Nothing);
    let inputs = [
        Input::Move {
            at: 0,
            to: centre(90),
        },
        Input::Move { at: 10, to: off },
        Input::Move {
            at: 20,
            to: centre(88),
        },
        Input::Move { at: 30, to: off },
        // Nothing again: no hex before or now, written again.
        Input::Move {
            at: 35,
            to: [6, 200],
        },
    ];
    let shown = run(&[], &[], &turns, &inputs);
    let texts: Vec<_> = shown
        .log
        .iter()
        .map(|(t, l)| match l {
            Logged::Status { text, .. } => (*t, text.clone()),
            _ => (*t, String::new()),
        })
        .collect();
    // Over nothing the target set over 90 says "No shots left!"; an empty hex clears it.
    assert_eq!(
        texts,
        [
            (0, "Attack(1)".into()),
            (10, "NoShots".into()),
            (20, "Empty".into()),
            (30, "Empty".into()),
            (35, "Empty".into()),
        ]
    );
}

#[test]
fn a_turned_stack_takes_its_other_hexes() {
    // The defender's two-hex stack on 90 faces right: its other hex is 91, not 89.
    let mut o = battle();
    o.stacks[1].two_hex = true;
    let legal = Legal::default();
    let sides = Facing::Sides;
    let turned = Facing::Left(BTreeSet::new());
    let marked = |facing, hex| {
        let field = Field {
            observation: &o,
            legal: &legal,
            facing,
            bugs: Bugs::Fixed,
        };
        crate::mouse::pointer_in(&field, &layout(), centre(hex), &mut None).marked
    };
    assert_eq!(marked(&sides, 89), Some(1));
    assert_eq!(marked(&sides, 91), None);
    assert_eq!(marked(&turned, 91), Some(1));
    assert_eq!(marked(&turned, 89), None);
    assert!(sides.left(&o.stacks[1]));
    assert!(!turned.left(&o.stacks[1]));
    assert!(!sides.left(&o.stacks[0]));
    // No acting stack: nothing.
    let mut idle = battle();
    idle.active = None;
    let field = Field {
        observation: &idle,
        legal: &legal,
        facing: &sides,
        bugs: Bugs::Fixed,
    };
    let p = crate::mouse::pointer_in(&field, &layout(), centre(90), &mut None);
    assert_eq!(p.hint, Hint::Empty);
    assert_eq!(o.stacks[1].side, Side::Defender);
}

#[test]
fn without_turns_the_panel_still_answers() {
    // A record without its battle: the mouse over the panel writes the control's hint (no
    // words without a battle), over the field nothing; resting where it was placed before the
    // battle, nothing (E17 M4 T3b).
    let inputs = [
        Input::Move { at: 0, to: PARK },
        Input::Move {
            at: 1,
            to: [700, 570],
        },
        Input::Move {
            at: 5,
            to: centre(90),
        },
        Input::Act { at: 9 },
    ];
    let shown = run(&[(9, Logged::Action)], &[], &[], &inputs);
    assert_eq!(shown.log, [(1, status("", false)), (9, Logged::Action)]);
    assert_eq!(shown.frames, [(0, 6)]);
    assert_eq!(Input::Act { at: 4 }.at(), 4);
}

#[test]
fn a_two_hex_stack_stays_marked_across_its_hexes() {
    // The defender's two-hex stack on 90 (its other hex 89): from one hex to the other it stays
    // marked and is not entered again. A turn the battle has no entry for, and a line, pass.
    let mut o = battle();
    o.stacks[1].two_hex = true;
    let legal = legal();
    let turns = [Turn {
        field: Field {
            observation: &o,
            legal: &legal,
            facing: &Facing::Sides,
            bugs: Bugs::Fixed,
        },
        human: true,
    }];
    let inputs = [
        Input::Move {
            at: 0,
            to: centre(90),
        },
        Input::Move {
            at: 10,
            to: centre(89),
        },
    ];
    let log = [
        (5, Logged::Wait { stack: 0, count: 1 }),
        (20, Logged::Turn(3)),
    ];
    let shown = run(&log, &[], &turns, &inputs);
    assert_eq!(shown.marked, [(0, Some(1))]);
    assert_eq!(shown.entered, [(0, 1)]);
    assert!(
        shown
            .log
            .contains(&(5, Logged::Wait { stack: 0, count: 1 }))
    );
    assert_eq!(shown.log.last(), Some(&(20, Logged::Turn(3))));
}

#[test]
fn an_aimed_spell_hints_its_cast() {
    // E17 M3 T7b-1: from the book's spell to the click, the cast's hint over the battlefield,
    // not forced (`arrow`'s frames: the log's new lines stay), the marked stack and the cursor
    // left as they were; the click ends the aim.
    let (o, legal) = (battle(), legal());
    let field = Field {
        observation: &o,
        legal: &legal,
        facing: &Facing::Sides,
        bugs: Bugs::Fixed,
    };
    let turns = [Turn { field, human: true }];
    let inputs = [
        Input::Move { at: 0, to: PARK },
        Input::Aim { at: 0, spell: 54 },
        Input::Move {
            at: 0,
            to: centre(90),
        },
        Input::Move {
            at: 0,
            to: centre(87),
        },
        Input::Move {
            at: 0,
            to: [700, 570],
        },
        Input::Move {
            at: 0,
            to: [400, 3],
        },
        Input::Act { at: 0 },
        Input::Move {
            at: 10,
            to: centre(90),
        },
    ];
    let shown = run(&[], &[], &turns, &inputs);
    let texts: Vec<(String, bool)> = shown
        .log
        .iter()
        .filter_map(|(_, l)| match l {
            Logged::Status { text, forced } => Some((text.clone(), *forced)),
            _ => None,
        })
        .collect();
    let s = |t: &str, f| (String::from(t), f);
    assert_eq!(
        texts,
        [
            s("Cast(54, Some(1))", false),
            s("Cast(54, None)", false),
            s("Control(\"battle.control.wait\")", false),
            s("Empty", false),
            s("Attack(1)", true),
        ]
    );
    assert_eq!(shown.marked, [(10, Some(1))]);
    assert_eq!(Input::Aim { at: 3, spell: 1 }.at(), 3);
}

#[test]
fn the_spell_book_empties_the_status_line() {
    // E17 M3 T7c-1: the book over the battle takes the mouse (a spell without a target is cast
    // from it at once); a move back over the same hex writes its hint again.
    let (o, legal) = (battle(), legal());
    let field = Field {
        observation: &o,
        legal: &legal,
        facing: &Facing::Sides,
        bugs: Bugs::Fixed,
    };
    let turns = [Turn { field, human: true }];
    let inputs = [
        Input::Move {
            at: 0,
            to: centre(87),
        },
        Input::Move { at: 1, to: PARK },
        Input::Book { at: 2 },
        Input::Move {
            at: 3,
            to: centre(87),
        },
        Input::Move { at: 4, to: PARK },
    ];
    let shown = run(&[], &[], &turns, &inputs);
    let texts: Vec<(u32, String)> = shown
        .log
        .iter()
        .filter_map(|(t, l)| match l {
            Logged::Status { text, .. } => Some((*t, text.clone())),
            _ => None,
        })
        .collect();
    // The book's items, the hovered one among them, are gone as it closes (E17 M4 T3b): the
    // panel's picture is entered again.
    let empty = String::from("Empty");
    assert_eq!(texts.len(), 5);
    assert_eq!(texts[1], (1, empty.clone()));
    assert_eq!(texts[2], (2, String::new()));
    assert_eq!(texts[3].1, texts[0].1);
    assert_eq!(texts[4], (4, empty));
    assert_eq!(Input::Book { at: 4 }.at(), 4);
}
