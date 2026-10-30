//! The mouse over time's tests (E17 M3 T6c-2, design §9.10), on the committed layout and the
//! mouse's test battle: the attacker's stack 0 acting on hex 86, the defender's 1 on 90, the
//! attacker's 2 on 120, the defender's 3 on 100.

use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use h3_protocol::wire::{Command, Observation, Side};
use h3_render::Bugs;

use super::{Input, Lens, Look, Shown, Start, Turn, follow, follow_on, spell_frame};
use crate::battle_log::Logged;
use crate::mouse::aim::{Aim, Area, Target};
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

/// The mouse followed through the battle, checked gone on from the marks of shorter follows too.
fn run(log: &[(u32, Logged)], starts: &[Start], turns: &[Turn<'_>], inputs: &[Input]) -> Shown {
    let whole = follow(log, starts, turns, inputs, &layout(), &name);
    resumes((log, starts, turns), inputs, &whole);
    whole
}

/// E17 M5 T8a (R3-1): a follow of the battle before each turn's start (its log cut there, its
/// turns with it) or whole, with each number of the inputs, gone on from its mark to the whole
/// battle and inputs, shows what the whole follow shows; where it goes on, the whole's lists end
/// with what it adds.
fn resumes(
    (log, starts, turns): (&[(u32, Logged)], &[Start], &[Turn<'_>]),
    inputs: &[Input],
    whole: &Shown,
) {
    let l = layout();
    let is_turn = |(_, e): &(u32, Logged)| matches!(e, Logged::Turn(_));
    let cuts = (0..log.len()).filter(|&i| is_turn(&log[i]));
    for cut in cuts.chain([log.len()]) {
        let started = 1 + log[..cut].iter().filter(|e| is_turn(e)).count();
        let some = &turns[..started.min(turns.len())];
        for n in 0..=inputs.len() {
            let (from, mut shown, mark) =
                follow_on(None, (&log[..cut], starts, some), &inputs[..n], &l, &name);
            assert_eq!(from, Lens::default());
            let (from, tail, _) = follow_on(mark, (log, starts, turns), inputs, &l, &name);
            assert!(whole.ends_with(from, &tail), "log cut at {cut}, {n} inputs");
            shown.splice(from, tail);
            assert_eq!(shown, *whole, "log cut at {cut}, {n} inputs");
        }
    }
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
    let shown = run(
        &log,
        &[Start { unmarks: true }, Start::default()],
        &turns,
        &inputs,
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

/// Magic Arrow (its handle 7, its original number 15) aimed at the defender's stack 1, or a
/// Faerie Dragon's (handle 9) with `creature`.
fn arrow(creature: bool) -> Aim {
    Aim {
        spell: if creature { 9 } else { 7 },
        original: 15,
        creature,
        targets: vec![Target::Stack(1)],
        area: None,
        stacks: true,
        polarity: -1,
        level: 1,
        anti_magic: None,
    }
}

/// Fireball (its handle 8, its original number 21) aimed at hexes 87 and 90.
fn fireball() -> Aim {
    Aim {
        spell: 8,
        original: 21,
        targets: vec![Target::Hex(87), Target::Hex(90)],
        area: Some(Area {
            radius: 1,
            centre: true,
        }),
        stacks: false,
        polarity: 0,
        ..arrow(false)
    }
}

fn texts(shown: &Shown) -> Vec<String> {
    shown
        .log
        .iter()
        .filter_map(|(_, l)| match l {
            Logged::Status { text, forced } => {
                assert!(!forced || text.starts_with("Attack"), "{text}");
                Some(text.clone())
            }
            _ => None,
        })
        .collect()
}

#[test]
fn a_hero_s_aim_follows_the_mouse() {
    // E17 M5 T3b (design §7.18): the book takes the moves; the aim's cursor at the mouse as it
    // starts, its hex not kept (a click casts nothing); over a target the spell cursor, the
    // cast's line and the stack marked; elsewhere frame 0 and the reason, unmarked; the cursor
    // again only over another hex than the one it took; a click casts at it.
    let o = battle();
    let mut legal = legal();
    legal.aims = vec![arrow(false)];
    let field = Field {
        observation: &o,
        legal: &legal,
        facing: &Facing::Sides,
        bugs: Bugs::Fixed,
    };
    let turns = [Turn { field, human: true }];
    let over = |hex: u8, dx: i32| [centre(hex)[0] + dx, centre(hex)[1]];
    let inputs = [
        Input::Move { at: 0, to: PARK },
        Input::Book { at: 0 },
        Input::Move {
            at: 0,
            to: over(87, 0),
        },
        Input::Aim {
            at: 0,
            spell: 7,
            creature: false,
        },
        Input::Act { at: 0 },
        Input::Move {
            at: 100,
            to: over(90, 0),
        },
        Input::Move {
            at: 150,
            to: over(90, 5),
        },
        Input::Move {
            at: 200,
            to: over(86, 0),
        },
        Input::Move { at: 250, to: PARK },
        Input::Move {
            at: 260,
            to: over(86, 0),
        },
        Input::Move {
            at: 300,
            to: over(90, 0),
        },
        Input::Act { at: 320 },
    ];
    let log = [(320, Logged::Action)];
    let shown = run(&log, &[Start { unmarks: true }], &turns, &inputs);
    assert_eq!(
        texts(&shown),
        [
            "",
            "SelectTarget",
            "Cast(7, Some(1))",
            "HostileOnly(7)",
            "HostileOnly(7)",
            "Cast(7, Some(1))",
        ]
    );
    assert_eq!(
        shown.spell,
        [(100, Some(100)), (200, None), (300, Some(300)), (320, None)]
    );
    assert_eq!(shown.frames, [(0, 6), (0, 0), (200, 0), (320, 6)]);
    assert_eq!(
        shown.marked,
        [(100, Some(1)), (200, None), (300, Some(1)), (320, None)]
    );
    assert_eq!(shown.entered, [(100, 1), (300, 1)]);
    assert_eq!(
        shown.aimed,
        [
            (100, Some(90)),
            (200, None),
            (260, None),
            (300, Some(90)),
            (320, None)
        ]
    );
    let mouse = &layout().mouse;
    assert_eq!(shown.look(150, mouse), Some(Look::Spell(0)));
    assert_eq!(shown.look(250, mouse), Some(Look::Combat(0)));
    assert_eq!(shown.look(319, mouse), Some(Look::Spell(0)));
    assert_eq!(shown.look(400, mouse), Some(Look::Combat(6)));
    assert_eq!(shown.aimed(150), Some(90));
    assert_eq!(shown.aimed(400), None);
    assert_eq!(Input::Cancel { at: 3 }.at(), 3);
}

#[test]
fn the_spell_cursor_steps_and_starts_again() {
    // `mouse_poll` steps its frame at each 100 ms of the clock; another hex's cursor draws its
    // frame 0 again unless the frame shown is the one it asks (the spell's number + 1, 16).
    let o = battle();
    let mut legal = legal();
    let mut aim = arrow(false);
    aim.targets = vec![Target::Stack(1), Target::Stack(3)];
    legal.aims = vec![aim];
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
            to: centre(90),
        },
        Input::Aim {
            at: 0,
            spell: 7,
            creature: false,
        },
        Input::Move {
            at: 1650,
            to: centre(100),
        },
        Input::Move {
            at: 1700,
            to: centre(90),
        },
    ];
    let shown = run(&[], &[], &turns, &inputs);
    assert_eq!(shown.spell, [(0, Some(0)), (1700, Some(1700))]);
    let mouse = &layout().mouse;
    assert_eq!(shown.look(99, mouse), Some(Look::Spell(0)));
    assert_eq!(shown.look(100, mouse), Some(Look::Spell(1)));
    assert_eq!(shown.look(1699, mouse), Some(Look::Spell(16)));
    assert_eq!(shown.look(2050, mouse), Some(Look::Spell(3)));
    assert_eq!(shown.look(3700, mouse), Some(Look::Spell(0)));
    assert_eq!(spell_frame(0, 5, &layout().mouse), 0);
}

#[test]
fn an_area_s_stacks_are_marked_and_an_aim_given_up() {
    // An area spell marks the stacks of its area, over a hex it does not take too; given up, the
    // marks go, the hero's book posts the mouse again (its hint where it rests), the Faerie
    // Dragon's key leaves the pointer until another hex; a spell the turn does not offer is not
    // aimed.
    let o = battle();
    let mut legal = legal();
    legal.aims = vec![fireball(), arrow(true)];
    let field = Field {
        observation: &o,
        legal: &legal,
        facing: &Facing::Sides,
        bugs: Bugs::Fixed,
    };
    let turns = [Turn { field, human: true }];
    let aim = |at, spell, creature| Input::Aim {
        at,
        spell,
        creature,
    };
    let to = |at, hex: u8| Input::Move {
        at,
        to: centre(hex),
    };
    let inputs = [
        to(0, 90),
        aim(10, 8, false),
        to(20, 87),
        to(30, 91),
        Input::Cancel { at: 40 },
        aim(50, 9, true),
        to(60, 90),
        Input::Cancel { at: 70 },
        Input::Move {
            at: 80,
            to: [centre(90)[0] + 3, centre(90)[1]],
        },
        aim(90, 99, false),
        to(95, 87),
    ];
    let shown = run(&[], &[], &turns, &inputs);
    assert_eq!(
        shown.area,
        [(10, vec![1]), (20, vec![0]), (30, vec![1]), (40, vec![]),]
    );
    assert_eq!(shown.area(25), [0]);
    assert_eq!(shown.area(45), [] as [u8; 0]);
    assert_eq!(
        texts(&shown),
        [
            "Attack(1)",
            "Cast(8, None)",
            "Cast(8, None)",
            "SelectTarget",
            "Empty",
            "SelectTarget",
            "Cast(9, Some(1))",
            "Attack(1)",
            "Move(0)",
        ]
    );
    // The marked stack stays through the area's aim; the Faerie Dragon's marks its target.
    assert_eq!(
        shown.marked,
        [(0, Some(1)), (30, None), (60, Some(1)), (95, None)]
    );
    // The pointer after the key's aim until the mouse moves (the battle's hex is 91 still).
    let mouse = &layout().mouse;
    assert_eq!(shown.look(75, mouse), Some(Look::Combat(6)));
    assert_eq!(shown.look(96, mouse), Some(Look::Combat(1)));
}

#[test]
fn an_aim_at_an_empty_hex_marks_nothing() {
    // A cancel before the mouse is anywhere posts nothing; a hex target the aim takes with no
    // stack on it unmarks the mouse's stack and enters none.
    let o = battle();
    let mut legal = legal();
    legal.aims = vec![Aim {
        targets: vec![Target::Hex(87)],
        stacks: false,
        ..arrow(false)
    }];
    let field = Field {
        observation: &o,
        legal: &legal,
        facing: &Facing::Sides,
        bugs: Bugs::Fixed,
    };
    let turns = [Turn { field, human: true }];
    let inputs = [
        Input::Cancel { at: 0 },
        Input::Move {
            at: 1,
            to: centre(90),
        },
        Input::Aim {
            at: 2,
            spell: 7,
            creature: false,
        },
        Input::Move {
            at: 3,
            to: centre(87),
        },
    ];
    let shown = run(&[], &[], &turns, &inputs);
    assert_eq!(shown.marked, [(1, Some(1)), (2, None)]);
    assert_eq!(shown.entered, [(1, 1)]);
    assert_eq!(shown.aimed, [(3, Some(87))]);
    assert_eq!(shown.spell, [(3, Some(3))]);
}

#[test]
fn the_spell_book_empties_the_status_line() {
    // E17 M3 T7c-1: the book over the battle takes the mouse (a spell without a target is cast
    // from it at once); E17 M5 T3b: it takes the moves until it closes, the battle then posted
    // the mouse where it rests (its hint written again).
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
        Input::Cancel { at: 4 },
        Input::Move { at: 5, to: PARK },
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
    assert_eq!(texts[3], (4, texts[0].1.clone()));
    assert_eq!(texts[4], (5, empty));
    assert_eq!(Input::Book { at: 4 }.at(), 4);
}

#[test]
fn moves_made_while_an_action_plays_are_dropped_as_the_next_turn_starts() {
    // E17 M5 T2a (`input_flush` 0x4EC570 from `battle_controls_state` 0x478301, design §9.11):
    // the move over the attacker's stack 2 made during the action neither marks nor enters it nor
    // writes its hint; the next human turn takes only the mouse where it rests.
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
    let turns = [turn(&o, &legal), turn(&next, &none)];
    let inputs = [
        Input::Move {
            at: 0,
            to: centre(87),
        },
        // Elsewhere on the same hex: the same walk, nothing written again.
        Input::Move {
            at: 50,
            to: [centre(87)[0] + 3, centre(87)[1]],
        },
        Input::Act { at: 100 },
        Input::Move {
            at: 200,
            to: centre(120),
        },
        Input::Move {
            at: 300,
            to: centre(88),
        },
    ];
    let log = [(100, Logged::Action), (500, Logged::Turn(2))];
    let shown = run(&log, &[Start { unmarks: true }], &turns, &inputs);
    assert_eq!(shown.marked, []);
    assert_eq!(shown.entered, []);
    assert_eq!(
        shown.log,
        [
            (0, status("Move(0)", false)),
            (100, Logged::Action),
            (500, Logged::Turn(2)),
            (500, status("Empty", false)),
        ]
    );
    assert_eq!(shown.point(400), Some(centre(88)));
    // A human turn starting with the mouse nowhere takes nothing (the action showed the pointer).
    let still = run(&log, &[Start { unmarks: true }], &turns, &[]);
    assert_eq!(still.log, log);
    assert_eq!(still.frames, [(100, 6)]);
}

#[test]
fn auto_combat_s_switch_and_stop_move_nothing_themselves() {
    // E17 M5 T4c1: the switch shows with the computer's action it starts, the stop with the turn
    // it stops at (`Turn::human`): followed with them or without, the mouse shows the same.
    let (o, legal) = (battle(), legal());
    let turn = Turn {
        field: Field {
            observation: &o,
            legal: &legal,
            facing: &Facing::Sides,
            bugs: Bugs::Fixed,
        },
        human: true,
    };
    let moved = Input::Move {
        at: 0,
        to: centre(87),
    };
    let log = [(100, Logged::Action)];
    let starts = [Start { unmarks: true }];
    let with = run(
        &log,
        &starts,
        &[turn],
        &[
            moved,
            Input::Auto { at: 100, turn: 0 },
            Input::Stop { at: 150, turn: 0 },
        ],
    );
    assert_eq!(with, run(&log, &starts, &[turn], &[moved]));
    assert_eq!(
        [
            Input::Auto { at: 5, turn: 0 }.at(),
            Input::Stop { at: 6, turn: 0 }.at()
        ],
        [5, 6]
    );
}

#[test]
fn the_client_forgets_the_hovered_item_over_the_battlefield() {
    // CLT-Q24, answered (a) (E17 M5 T2a): back on the panel's picture from the battlefield, the
    // client's profile writes its empty line again; the original's keeps the battlefield's hint.
    let (o, legal) = (battle(), legal());
    let inputs = [
        Input::Move { at: 0, to: PARK },
        Input::Move {
            at: 100,
            to: [700, 570],
        },
        Input::Move { at: 200, to: PARK },
        Input::Move {
            at: 300,
            to: centre(87),
        },
        Input::Move { at: 400, to: PARK },
    ];
    let shown = |bugs| {
        let field = Field {
            observation: &o,
            legal: &legal,
            facing: &Facing::Sides,
            bugs,
        };
        run(&[], &[], &[Turn { field, human: true }], &inputs).log
    };
    let common = [
        (100, status("Control(\"battle.control.wait\")", false)),
        (200, status("Empty", false)),
        (300, status("Move(0)", false)),
    ];
    assert_eq!(shown(Bugs::Original), common);
    let mut fixed = common.to_vec();
    fixed.push((400, status("Empty", false)));
    assert_eq!(shown(Bugs::Fixed), fixed);
}

#[test]
fn a_mark_holds_only_where_the_follow_went_the_same() {
    // E17 M5 T8a (R3-1): the mark before the second turn's start (an input after it waits);
    // a follow goes on from it while the log has its entry there, the inputs before it are the
    // same and the next comes no earlier than the round's start taken before the turn's. Else
    // it follows from the opening.
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
    let turns = [turn(&o, &legal), turn(&next, &none)];
    let starts = [Start { unmarks: true }];
    let log = [
        (100, Logged::Action),
        (300, Logged::Round),
        (500, Logged::Turn(2)),
    ];
    let mut inputs = vec![
        Input::Move {
            at: 50,
            to: centre(90),
        },
        Input::Act { at: 100 },
        Input::Move {
            at: 600,
            to: centre(87),
        },
    ];
    let l = layout();
    let follow = |log: &[(u32, Logged)], inputs: &[Input], mark| {
        follow_on(mark, (log, &starts, &turns), inputs, &l, &name)
    };
    let (_, whole, mark) = follow(&log, &inputs, None);
    let mark = mark.unwrap();
    assert_eq!(mark.turn(), 1);
    assert!(!whole.ends_with(Lens::default(), &Shown::default()));
    // A move at the round's start goes on from the mark, taken before the turn's start.
    inputs.insert(2, Input::Move { at: 300, to: PARK });
    let (from, tail, again) = follow(&log, &inputs, Some(mark.clone()));
    assert_ne!(from, Lens::default());
    let mut shown = whole.clone();
    shown.splice(from, tail);
    assert_eq!(shown, follow(&log, &inputs, None).1);
    assert_eq!(again.unwrap().turn(), 1);
    // Refused: a move before the round's start, another first move, another turn's start.
    let mut earlier = inputs.clone();
    earlier[2] = Input::Move { at: 299, to: PARK };
    let mut other = inputs.clone();
    other[0] = Input::Move { at: 50, to: PARK };
    let mut moved = log.clone();
    moved[2].0 = 400;
    for (log, inputs) in [(&log, &earlier), (&log, &other), (&moved, &inputs)] {
        let (from, tail, _) = follow(log, inputs, Some(mark.clone()));
        assert_eq!(from, Lens::default());
        assert_eq!(tail, follow(log, inputs, None).1);
    }
}
