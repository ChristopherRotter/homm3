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
use crate::army_info::Windows;
use crate::battle_log::Logged;
use crate::mouse::aim::{Aim, Area, Target};
use crate::mouse::tests::{battle, centre, layout};
use crate::mouse::{Facing, Field, Hint, Legal, Previews, Spot, slot, slots, spot};
use crate::shades::Options;

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

/// An action's start that unmarks the mouse's stack (a walk, a strike, a shot, a cast).
const UNMARKS: Start = Start {
    unmarks: true,
    ends_tactics: false,
};

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
fn creature_info_s_level_words_the_hints_written_after_it() {
    // E17 M6 T4b2a (design §12.14): F5 or the options dialog sets the level at its time and
    // writes nothing (oddity 6); a move over the same hex writes nothing either, but a new slot
    // writes the strike from its position again, at the level then; a new hex at the level then.
    let o = battle();
    let mut legal = legal();
    let from = |from| Command::Attack { target: 1, from };
    legal.previews = Previews::new(&[(from(89), [10, 12]), (from(91), [20, 24])]);
    let field = Field {
        observation: &o,
        legal: &legal,
        facing: &Facing::Sides,
        bugs: Bugs::Fixed,
    };
    let turns = [Turn { field, human: true }];
    // Right of hex 90's centre the slot from 91, left of it the one from 89.
    let (a, b) = (
        [centre(90)[0] + 10, centre(90)[1]],
        [centre(90)[0] - 10, centre(90)[1]],
    );
    let level = |at, level| Input::ArmyInfo { at, level };
    let to = |at, to| Input::Move { at, to };
    let inputs = [
        to(0, PARK),
        to(100, a),
        level(110, 1),
        to(120, a),
        to(150, b),
        level(160, 2),
        to(170, b),
        to(200, centre(87)),
        to(220, a),
        level(250, 0),
        to(300, b),
    ];
    assert_eq!(level(7, 1).at(), 7);
    let shown = run(&[], &[], &turns, &inputs);
    let log = [
        (100, status("Attack(1)", true)),
        (150, status("Strike(1, [10, 12])", true)),
        (200, status("Move(0)", false)),
        (220, status("Strike(1, [20, 24])", true)),
        (300, status("Attack(1)", true)),
    ];
    assert_eq!(shown.log, log);
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
    let shown = run(&log, &[UNMARKS, Start::default()], &turns, &inputs);
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
    let shown = run(&log, &[UNMARKS], &turns, &inputs);
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
        mastery: 0,
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
    let shown = run(&log, &[UNMARKS], &turns, &inputs);
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
        [(100, Some(90)), (200, None), (300, Some(90)), (320, None)]
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
    // Dragon's key leaves the pointer until another hex (the original's; CLT-Q27 (1)); a spell
    // the turn does not offer is not aimed.
    let o = battle();
    let mut legal = legal();
    legal.aims = vec![fireball(), arrow(true)];
    let field = Field {
        observation: &o,
        legal: &legal,
        facing: &Facing::Sides,
        bugs: Bugs::Original,
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
    // The aim's start does not keep the hex its cursor takes: a click before a move casts nothing.
    assert_eq!(shown.aimed(15), None);
    // The client's (CLT-Q27): the key's aim given up posts the mouse as the book's does, its
    // cursor at once what the original shows after the next move; the aim's start keeps its hex.
    let field = Field {
        bugs: Bugs::Fixed,
        ..field
    };
    let turns = [Turn { field, human: true }];
    let client = run(&[], &[], &turns, &inputs);
    assert_eq!(texts(&client), texts(&shown));
    assert_eq!(client.look(75, mouse), shown.look(85, mouse));
    assert_eq!(client.look(96, mouse), Some(Look::Combat(1)));
    assert_eq!(client.aimed(15), Some(90));
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
fn a_press_reads_the_move_made_at_its_time() {
    // E17 M5 T8b2 (R4b-1): the window follows a pass's moves before its press, which casts at the
    // hex the cursor took at the press's time: a move off the target in the press's millisecond
    // leaves nothing to cast at (the window gives no `Act`); a move onto it then casts, and the
    // aim ends with the press.
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
    let to = |at, hex: u8| Input::Move {
        at,
        to: centre(hex),
    };
    let moved = vec![
        Input::Move { at: 0, to: PARK },
        Input::Aim {
            at: 0,
            spell: 7,
            creature: false,
        },
        to(10, 90),
        to(20, 91),
    ];
    let shown = run(&[], &[], &turns, &moved);
    assert_eq!(shown.aimed(10), Some(90));
    assert_eq!(shown.aimed(20), None);
    let mut pressed = moved;
    pressed.extend([to(30, 90), Input::Act { at: 30 }]);
    let log = [(30, Logged::Action)];
    let shown = run(&log, &[UNMARKS], &turns, &pressed);
    assert_eq!(
        shown.aimed,
        [(10, Some(90)), (20, None), (30, Some(90)), (30, None)]
    );
    assert_eq!(shown.aimed(30), None);
    assert_eq!(shown.spell.last(), Some(&(30, None)));
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
    let shown = run(&log, &[UNMARKS], &turns, &inputs);
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
    let still = run(&log, &[UNMARKS], &turns, &[]);
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
    let starts = [UNMARKS];
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
    let starts = [UNMARKS];
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

#[test]
fn the_stats_dialog_holds_the_mouse_and_hands_it_back() {
    // E17 M6 T2a2a: a press on a stack to look at (hex 120, the attacker's own) opens it: the
    // combat cursor's pointer, the status line kept, the moves over the dialog not the battle's;
    // as it closes the mouse's hex is forgotten, the log's new entries settle and the mouse is
    // posted where it rests: its hex's hint written (again after the second).
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
            to: centre(120),
        },
        Input::Look { at: 1 },
        Input::Move {
            at: 2,
            to: centre(87),
        },
        Input::Looked { at: 3 },
        Input::Look { at: 4 },
        Input::Looked { at: 5 },
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
    assert_eq!(texts.len(), 3);
    assert_eq!(texts[0].0, 0);
    assert_eq!(texts[1], (3, String::from("Move(0)")));
    assert_eq!(texts[2], (5, String::from("Move(0)")));
    let settles: Vec<u32> = shown
        .log
        .iter()
        .filter(|(_, l)| *l == Logged::Settle)
        .map(|&(t, _)| t)
        .collect();
    assert_eq!(settles, [3, 5]);
    // The pointer as it opens, the walk's cursor at the posted move, the pointer again.
    assert_eq!(shown.frame(1), Some(6));
    assert_eq!(shown.frame(2), Some(6));
    assert_eq!(shown.frame(3), Some(1));
    assert_eq!(shown.frame(4), Some(6));
    assert_eq!(
        (Input::Look { at: 7 }.at(), Input::Looked { at: 8 }.at()),
        (7, 8)
    );
    // On a computer's turn nothing is posted.
    let computers = [Turn {
        field,
        human: false,
    }];
    let shown = run(&[], &[], &computers, &inputs[..4]);
    assert!(shown.log.iter().all(|(_, l)| *l != Logged::Settle));
}

#[test]
fn the_dialogs_close_after_an_action_settles_the_log_and_posts_no_move() {
    // E17 M6 T2a2b1: the stats dialog's cast button's aim gave an action; the dialog's close
    // after it settles the log, and its posted move waits behind the action: no hint written.
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
            to: centre(120),
        },
        Input::Look { at: 1 },
        Input::Move {
            at: 1,
            to: centre(87),
        },
        Input::Act { at: 2 },
        Input::Looked { at: 2 },
    ];
    let log = [(2, Logged::Action)];
    let shown = run(&log, &[Start::default()], &turns, &inputs);
    let at_two: Vec<&Logged> = shown
        .log
        .iter()
        .filter(|(t, _)| *t == 2)
        .map(|(_, l)| l)
        .collect();
    assert!(at_two.contains(&&Logged::Settle));
    assert!(!at_two.iter().any(|l| matches!(l, Logged::Status { .. })));
}

#[test]
fn the_army_info_windows_follow_the_mouse_the_presses_and_the_actions() {
    // E17 M6 T4b2b1 (design §12.15): a new hex hides every window and shows the hero's under
    // it at levels 1 and 2; the same hex changes nothing; a press over the battlefield (`Hide`)
    // hides them, the hex kept; the level changes none (drawn or not); the right press keeps its
    // window at 1 and 2, hides it at 0; an action's start hides them; a human's turn start shows
    // the one under the mouse again, filled by its own battle.
    let (o, legal) = (battle(), legal());
    let mut next = battle();
    next.active = Some(2);
    let none = Legal::default();
    let l = layout();
    let hero = |index: usize| {
        let [left, top, right, bottom] = l.mouse.heroes[index];
        [(left + right) / 2, (top + bottom) / 2]
    };
    let (a, d) = (hero(0), hero(1));
    let to = |at, to| Input::Move { at, to };
    let level = |at, level| Input::ArmyInfo { at, level };
    let inputs = [
        level(0, 1),
        to(0, PARK),
        to(100, a),
        to(110, [a[0] + 1, a[1]]),
        Input::Hide { at: 120, key: true },
        to(130, d),
        level(140, 0),
        Input::HeroInfo {
            at: 150,
            side: Side::Defender,
        },
        level(160, 2),
        to(170, centre(87)),
        to(180, a),
        Input::Hide {
            at: 190,
            key: false,
        },
        Input::HeroInfo {
            at: 200,
            side: Side::Attacker,
        },
        Input::Act { at: 400 },
    ];
    assert_eq!(Input::Hide { at: 7, key: false }.at(), 7);
    assert_eq!(
        Input::HeroInfo {
            at: 8,
            side: Side::Attacker
        }
        .at(),
        8
    );
    let log = [(400, Logged::Action), (500, Logged::Turn(2))];
    let windows = |heroes, level| Windows {
        heroes,
        stack: None,
        level,
    };
    for bugs in [Bugs::Fixed, Bugs::Original] {
        let turn = |observation, legal| Turn {
            field: Field {
                observation,
                legal,
                facing: &Facing::Sides,
                bugs,
            },
            human: true,
        };
        let turns = [turn(&o, &legal), turn(&next, &none)];
        let shown = run(&log, &[UNMARKS], &turns, &inputs);
        let mut army = vec![
            (0, windows([None, None], 1)),
            (100, windows([Some(0), None], 1)),
            (120, windows([None, None], 1)),
            (130, windows([None, Some(0)], 1)),
            (140, windows([None, Some(0)], 0)),
            (150, windows([None, None], 0)),
            (160, windows([None, None], 2)),
            (180, windows([Some(0), None], 2)),
            (190, windows([None, None], 2)),
            (200, windows([Some(0), None], 2)),
            (400, windows([None, None], 2)),
            (500, windows([Some(1), None], 2)),
        ];
        if bugs == Bugs::Original {
            // The original's keeps the windows through the keys C, F, S, R and O (CLT-Q49).
            army.remove(2);
        }
        assert_eq!(shown.army, army, "{bugs:?}");
        assert_eq!(shown.army(115), windows([Some(0), None], 1));
        assert_eq!(shown.army(145).drawn().count(), 0);
        // The mouse still on the attacker's rectangle since 180: the left press hides its window,
        // the right press shows it again, and it stays after the release until the action.
        assert_eq!(shown.army(195).heroes, [None, None]);
        assert_eq!(shown.army(200).heroes, [Some(0), None]);
        assert_eq!(shown.army(399).heroes, [Some(0), None]);
    }
    assert_eq!(Shown::default().army(0), Windows::default());
}

#[test]
fn the_stacks_windows_follow_the_mouse_by_the_level() {
    // E17 M6 T4b2b2 (design §12.15): a new hex over a stack shows its window by its owner's side,
    // kind 1 at level 1 and kind 2 at 2, filled by the turn's battle; the same hex changes
    // nothing, nor the other hex of a two-hex stack (oddity 10: hidden and shown again, the same
    // window); a level changed keeps the kind (drawn at 0 no more); a press's hide, an action's
    // start hide it; a human's turn start shows the one under the mouse again.
    use crate::army_info::{Kind, StackShown};
    let mut o = battle();
    o.stacks[3].two_hex = true;
    let mut next = o.clone();
    next.active = Some(2);
    let (legal, none) = (legal(), Legal::default());
    let to = |at, to| Input::Move { at, to };
    let level = |at, level| Input::ArmyInfo { at, level };
    let inputs = [
        level(0, 1),
        to(0, PARK),
        to(100, centre(86)),
        to(110, [centre(86)[0] + 1, centre(86)[1]]),
        to(120, centre(90)),
        level(130, 2),
        to(140, centre(100)),
        to(150, centre(99)),
        Input::Hide {
            at: 160,
            key: false,
        },
        to(170, centre(87)),
        to(180, centre(120)),
        level(190, 0),
        to(200, centre(90)),
        level(210, 1),
        to(220, centre(86)),
        Input::Act { at: 400 },
    ];
    let log = [(400, Logged::Action), (500, Logged::Turn(2))];
    let windows = |stack: Option<(Side, Kind, u8, usize)>, level| Windows {
        heroes: [None, None],
        stack: stack.map(|(side, kind, stack, turn)| StackShown {
            side,
            kind,
            stack,
            turn,
        }),
        level,
    };
    let (a, d) = (Side::Attacker, Side::Defender);
    for bugs in [Bugs::Fixed, Bugs::Original] {
        let turn = |observation, legal| Turn {
            field: Field {
                observation,
                legal,
                facing: &Facing::Sides,
                bugs,
            },
            human: true,
        };
        let turns = [turn(&o, &legal), turn(&next, &none)];
        let shown = run(&log, &[UNMARKS], &turns, &inputs);
        let army = vec![
            (0, windows(None, 1)),
            (100, windows(Some((a, Kind::All, 0, 0)), 1)),
            (120, windows(Some((d, Kind::All, 1, 0)), 1)),
            (130, windows(Some((d, Kind::All, 1, 0)), 2)),
            (140, windows(Some((d, Kind::Spells, 3, 0)), 2)),
            (160, windows(None, 2)),
            (180, windows(Some((a, Kind::Spells, 2, 0)), 2)),
            (190, windows(Some((a, Kind::Spells, 2, 0)), 0)),
            (200, windows(None, 0)),
            (210, windows(None, 1)),
            (220, windows(Some((a, Kind::All, 0, 0)), 1)),
            (400, windows(None, 1)),
            (500, windows(Some((a, Kind::All, 0, 1)), 1)),
        ];
        assert_eq!(shown.army, army, "{bugs:?}");
        assert_eq!(shown.army(195).drawn().count(), 0);
        assert_eq!(shown.army(135).drawn().count(), 1);
    }
}

#[test]
fn the_tactics_phase_s_end_hides_no_window() {
    // E17 M6 T4b2b1 (design §12.15): Start Combat is no action (0x4751CC, §7.24): the window
    // shown stays through the phase's end and a computer's turn start; the next action's start
    // hides it. In an action's place the same log hides it at once.
    let (o, legal) = (battle(), legal());
    let none = Legal::default();
    let turn = |legal, human| Turn {
        field: Field {
            observation: &o,
            legal,
            facing: &Facing::Sides,
            bugs: Bugs::Fixed,
        },
        human,
    };
    let turns = [turn(&legal, true), turn(&none, false)];
    let [left, top, right, bottom] = layout().mouse.heroes[0];
    let a = [(left + right) / 2, (top + bottom) / 2];
    let inputs = [
        Input::ArmyInfo { at: 0, level: 1 },
        Input::Move { at: 10, to: a },
        Input::Act { at: 100 },
    ];
    let log = [
        (100, Logged::Action),
        (100, Logged::Turn(2)),
        (900, Logged::Action),
    ];
    let ended = Start {
        ends_tactics: true,
        ..Start::default()
    };
    let shown = run(&log, &[ended, Start::default()], &turns, &inputs);
    let heroes = |shown: &Shown, t| shown.army(t).heroes;
    assert_eq!(heroes(&shown, 10), [Some(0), None]);
    assert_eq!(heroes(&shown, 899), [Some(0), None]);
    assert_eq!(heroes(&shown, 900), [None, None]);
    let action = run(&log, &[Start::default(); 2], &turns, &inputs);
    assert_eq!(heroes(&action, 100), [None, None]);
}

#[test]
fn the_stats_dialog_s_close_and_the_book_s_cancel_show_the_window_again() {
    // E17 M6 T4b2b1 (design §12.15, "Every show"): the key T hides the windows before the
    // stats dialog; its close forgets the mouse's hex and posts the move: the window under the
    // mouse comes back; so does a book closed without a spell (0x47519A).
    let (o, legal) = (battle(), legal());
    let field = Field {
        observation: &o,
        legal: &legal,
        facing: &Facing::Sides,
        bugs: Bugs::Fixed,
    };
    let turns = [Turn { field, human: true }];
    let [left, top, right, bottom] = layout().mouse.heroes[0];
    let a = [(left + right) / 2, (top + bottom) / 2];
    let inputs = [
        Input::ArmyInfo { at: 0, level: 1 },
        Input::Move { at: 10, to: a },
        Input::Hide { at: 20, key: false },
        Input::Look { at: 20 },
        Input::Looked { at: 30 },
        Input::Hide { at: 40, key: true },
        Input::Book { at: 40 },
        Input::Cancel { at: 50 },
    ];
    let shown = run(&[], &[], &turns, &inputs);
    let shown_at = |t| shown.army(t).heroes;
    assert_eq!(shown_at(10), [Some(0), None]);
    assert_eq!(shown_at(20), [None, None]);
    assert_eq!(shown_at(30), [Some(0), None]);
    assert_eq!(shown_at(40), [None, None]);
    assert_eq!(shown_at(50), [Some(0), None]);
}

/// The cursor shadow's hexes at `t` (E17 M6 T4c).
fn cursor_at(shown: &Shown, t: u32) -> Vec<u8> {
    shown
        .shades(t)
        .map_or_else(Vec::new, |d| d.overlays.cursor.clone())
}

/// The movement shade at `t` outside the tactics phase (E17 M6 T4c).
fn shade_at(shown: &Shown, t: u32) -> BTreeSet<u8> {
    shown
        .shades(t)
        .map_or_else(BTreeSet::new, |d| d.shade.clone())
}

/// The options with the grid by `grid`, the cursor shadow on, the movement shadow by `movement`.
const fn shadows(grid: bool, movement: bool) -> Options {
    Options {
        grid,
        cursor: true,
        movement,
    }
}

fn shades(at: u32, options: Options, keyed: bool) -> Input {
    Input::Shades { at, options, keyed }
}

#[test]
fn the_cursor_shadow_follows_the_moves_the_keys_and_the_actions() {
    // E17 M6 T4c (design §12.16): the hex under each move the battle takes, the same hex once;
    // F6 takes it off, composes the background again and posts the mouse, which shades it again;
    // an action's start takes it and the movement shade off, the next human turn's posted move
    // and overlay bring them back.
    let mut o = battle();
    o.sides[0].human = true;
    let legal = legal();
    let field = Field {
        observation: &o,
        legal: &legal,
        facing: &Facing::Sides,
        bugs: Bugs::Fixed,
    };
    let turns = [Turn { field, human: true }, Turn { field, human: true }];
    let to = |at, hex: u8| Input::Move {
        at,
        to: centre(hex),
    };
    let log = [(30, Logged::Action), (40, Logged::Turn(0))];
    let inputs = [
        shades(
            0,
            Options {
                grid: true,
                ..shadows(true, true)
            },
            false,
        ),
        to(5, 90),
        Input::Move {
            at: 6,
            to: [centre(90)[0] + 3, centre(90)[1]],
        },
        Input::Move { at: 7, to: PARK },
        to(8, 91),
        shades(10, shadows(false, true), true),
        Input::Act { at: 30 },
    ];
    let shown = run(&log, &[UNMARKS], &turns, &inputs);
    let reach = BTreeSet::from([86, 87, 90]);
    assert_eq!(crate::shades::reach(&field), reach);
    let opening = shown.shades(0).unwrap();
    assert!(opening.overlays.grid && opening.overlays.cursor.is_empty());
    assert_eq!(opening.shade, reach);
    assert_eq!(
        [5, 6, 7, 8].map(|t| cursor_at(&shown, t)),
        [vec![90], vec![90], vec![], vec![91]]
    );
    assert!(!shown.shades(10).unwrap().overlays.grid);
    assert_eq!(cursor_at(&shown, 10), [91]);
    assert!(shown.log.contains(&(10, Logged::Settle)));
    assert!(cursor_at(&shown, 30).is_empty() && shade_at(&shown, 30).is_empty());
    assert_eq!(
        (cursor_at(&shown, 40), shade_at(&shown, 40)),
        (vec![91], reach)
    );
    // On the computer's turn the keys leave the combat cursor's pointer.
    let computer = [Turn {
        field,
        human: false,
    }];
    let shown = run(
        &[],
        &[],
        &computer,
        &[shades(5, shadows(true, false), true)],
    );
    assert_eq!(shown.frames, [(0, 6)]);
    assert!(shown.shades(5).unwrap().overlays.grid);
}

#[test]
fn an_area_aim_shades_its_area_the_movement_shadow_off_through_it() {
    // E17 M6 T4c (design §12.16): with both shadows on the aim turns the movement shadow off as it
    // starts and on as it ends, its area shaded at the start and each other hex; given up, the
    // book's posted move shades the mouse's hex.
    for bugs in [Bugs::Original, Bugs::Fixed] {
        let mut o = battle();
        o.sides[0].human = true;
        let mut legal = legal();
        legal.aims = vec![fireball()];
        let field = Field {
            observation: &o,
            legal: &legal,
            facing: &Facing::Sides,
            bugs,
        };
        let turns = [Turn { field, human: true }];
        let to = |at, hex: u8| Input::Move {
            at,
            to: centre(hex),
        };
        let aim = Input::Aim {
            at: 10,
            spell: 8,
            creature: false,
        };
        let inputs = [
            shades(0, shadows(false, true), false),
            to(5, 90),
            aim,
            to(20, 87),
            Input::Cancel { at: 30 },
        ];
        let shown = run(&[], &[], &turns, &inputs);
        let area = |hex| crate::shades::area_shade(&fireball(), &field, hex);
        assert!(shade_at(&shown, 10).is_empty(), "{bugs:?}");
        assert_eq!(cursor_at(&shown, 10), area(90), "{bugs:?}");
        assert_eq!(cursor_at(&shown, 20), area(87), "{bugs:?}");
        assert_eq!(cursor_at(&shown, 30), [87], "{bugs:?}");
        assert!(!shade_at(&shown, 30).is_empty(), "{bugs:?}");
        // With the movement shadow off the given-up aim's shade stays in the original's profile
        // (oddity 3, the same key); the client's takes it off (CLT-Q53).
        let off = [
            shades(0, shadows(false, false), false),
            to(5, 90),
            aim,
            to(20, 87),
            Input::Cancel { at: 30 },
        ];
        let shown = run(&[], &[], &turns, &off);
        let left = if bugs == Bugs::Original {
            area(87)
        } else {
            vec![87]
        };
        assert_eq!(cursor_at(&shown, 30), left, "{bugs:?}");
    }
}

#[test]
fn a_targeted_aim_s_shade_is_frozen_in_the_original_s_profile_alone() {
    // CLT-Q54 (1): the original's keeps the hex shaded as the aim began through it; the client's
    // aims start with the cursor's shade off.
    for bugs in [Bugs::Original, Bugs::Fixed] {
        let o = battle();
        let mut legal = legal();
        legal.aims = vec![arrow(false)];
        let field = Field {
            observation: &o,
            legal: &legal,
            facing: &Facing::Sides,
            bugs,
        };
        let turns = [Turn { field, human: true }];
        let inputs = [
            shades(0, shadows(false, false), false),
            Input::Move {
                at: 5,
                to: centre(91),
            },
            Input::Aim {
                at: 10,
                spell: 7,
                creature: false,
            },
            Input::Move {
                at: 20,
                to: centre(90),
            },
        ];
        let shown = run(&[], &[], &turns, &inputs);
        let frozen = if bugs == Bugs::Original {
            vec![91]
        } else {
            vec![]
        };
        assert_eq!(
            [10, 20].map(|t| cursor_at(&shown, t)),
            [frozen.clone(), frozen]
        );
    }
}

/// Fire Wall (its handle 11, its original number 13) at Advanced, aimed at hexes 87 and 104, or
/// Force Field (handle 12, number 12).
fn wall(fire: bool) -> Aim {
    Aim {
        spell: if fire { 11 } else { 12 },
        original: if fire { 13 } else { 12 },
        targets: vec![Target::Hex(87), Target::Hex(104)],
        stacks: false,
        polarity: 0,
        mastery: 2,
        ..arrow(false)
    }
}

#[test]
fn the_wall_s_aim_keeps_its_last_hex_in_the_original_s_profile_alone() {
    // E17 M6 T4c (design §12.16): Fire Wall shades its hexes, Force Field none (CLT-Q54 (2),
    // copied). The original's wall aim calls no cursor as it starts and keeps the hex the wall's
    // aim before took: a click before a move casts there (CLT-Q56); the client's takes the hex
    // under the mouse.
    for bugs in [Bugs::Original, Bugs::Fixed] {
        let o = battle();
        let mut legal = legal();
        legal.aims = vec![wall(true), wall(false)];
        let field = Field {
            observation: &o,
            legal: &legal,
            facing: &Facing::Sides,
            bugs,
        };
        let turns = [Turn { field, human: true }];
        let to = |at, hex: u8| Input::Move {
            at,
            to: centre(hex),
        };
        let aim = |at, spell| Input::Aim {
            at,
            spell,
            creature: false,
        };
        let inputs = [
            shades(0, shadows(false, false), false),
            to(5, 90),
            aim(10, 11),
            to(20, 87),
            Input::Act { at: 30 },
            to(35, 104),
            aim(40, 11),
            Input::Cancel { at: 45 },
            aim(50, 12),
            to(60, 87),
        ];
        let shown = run(&[], &[], &turns, &inputs);
        let started = if bugs == Bugs::Original {
            vec![90]
        } else {
            vec![]
        };
        assert_eq!(cursor_at(&shown, 10), started, "{bugs:?}");
        assert_eq!(cursor_at(&shown, 20), [87, 70, 53], "{bugs:?}");
        let aimed = if bugs == Bugs::Original { 87 } else { 104 };
        assert_eq!(shown.aimed(40), Some(aimed), "{bugs:?}");
        // Force Field's aim shades nothing: the mouse's hex as the aim began stays (the
        // original's) or none (the client's).
        let kept = cursor_at(&shown, 50);
        assert_eq!(cursor_at(&shown, 60), kept, "{bugs:?}");
    }
}
