//! A spell's aim's tests (E17 M5 T3b, design §7.18), on the mouse's test battle: the attacker's
//! stack 0 acting on hex 86, the defender's 1 on 90, the attacker's 2 on 120, the defender's 3 on
//! 100.

use alloc::collections::BTreeSet;
use alloc::vec;

use h3_protocol::wire::{Effect, Side};
use h3_render::Bugs;

use super::{Aim, Area, Target, screen_hex};
use crate::mouse::tests::battle;
use crate::mouse::{Facing, Field, Hint, Legal, Spot};

/// Magic Arrow (its original number 15, hostile, level 1) aimed at the defender's stacks.
fn arrow() -> Aim {
    Aim {
        spell: 7,
        original: 15,
        creature: false,
        targets: vec![Target::Stack(1), Target::Stack(3)],
        area: None,
        stacks: true,
        polarity: -1,
        level: 1,
        anti_magic: Some(9),
    }
}

fn with<R>(o: &h3_protocol::wire::Observation, f: impl FnOnce(&Field<'_>) -> R) -> R {
    let legal = Legal::default();
    let field = Field {
        observation: o,
        legal: &legal,
        facing: &Facing::Sides,
        bugs: Bugs::Fixed,
    };
    f(&field)
}

#[test]
fn the_screen_s_numbers_are_the_executable_s() {
    assert_eq!(screen_hex(Spot::Hex(90)), 90);
    assert_eq!(screen_hex(Spot::Hero(Side::Attacker)), 0xFC);
    assert_eq!(screen_hex(Spot::Hero(Side::Defender)), 0xFD);
    assert_eq!(screen_hex(Spot::Keep), 0xFE);
    assert_eq!(screen_hex(Spot::Tower), 0xFF);
    assert_eq!(screen_hex(Spot::Nothing), -1);
}

#[test]
fn the_cursor_takes_the_engine_s_targets_off_the_side_columns() {
    let o = battle();
    with(&o, |field| {
        let a = arrow();
        assert_eq!(a.takes(field, 90), Some(90));
        assert_eq!(a.takes(field, 100), Some(100));
        // Not a target, nobody there, off the field, a hero's rectangle.
        for hex in [86, 91, -1, 187, 0xFC] {
            assert_eq!(a.takes(field, hex), None, "{hex}");
        }
        // A hex target, but none in a side column.
        let mut fire = arrow();
        fire.targets = vec![Target::Hex(0), Target::Hex(16), Target::Hex(40)];
        assert_eq!(fire.takes(field, 40), Some(40));
        assert_eq!(fire.takes(field, 0), None);
        assert_eq!(fire.takes(field, 16), None);
        assert_eq!(fire.target(field, 40), Some(Target::Hex(40)));
        assert_eq!(fire.target(field, 41), None);
    });
}

#[test]
fn the_cast_s_line_names_the_stack_by_the_spell() {
    let mut o = battle();
    // A dead stack on hex 50, raised by Resurrection.
    let mut dead = o.stacks[1].clone();
    dead.id = 4;
    dead.hex = 50;
    dead.alive = false;
    o.stacks.push(dead);
    with(&o, |field| {
        let a = arrow();
        assert_eq!(a.line(field, 90), Hint::Cast(7, Some(1)));
        assert_eq!(a.line(field, 91), Hint::Cast(7, None));
        // Fireball names none, Remove Obstacle has its own line.
        let fireball = Aim {
            original: 21,
            ..arrow()
        };
        assert_eq!(fireball.line(field, 90), Hint::Cast(7, None));
        let remove = Aim {
            original: 64,
            ..arrow()
        };
        assert_eq!(remove.line(field, 90), Hint::RemoveObstacle);
        // Resurrection names the stack it raises, none over a hex target.
        let raise = Aim {
            original: 38,
            targets: vec![Target::Stack(4), Target::Hex(40)],
            ..arrow()
        };
        assert_eq!(raise.line(field, 50), Hint::Cast(7, Some(4)));
        assert_eq!(raise.line(field, 40), Hint::Cast(7, None));
    });
}

#[test]
fn a_refusal_says_why() {
    let mut o = battle();
    // The defender's stack 3 under Anti-Magic at Basic (its value 4: levels up to 3 kept off).
    o.stacks[3].effects = vec![Effect {
        spell: 9,
        rounds: Some(2),
        mastery: 1,
        value: 4,
        attack: 0,
        defense: 0,
        speed: 0,
    }];
    with(&o, |field| {
        // The engine lets the spell be cast at no stack: each is refused.
        let a = Aim {
            targets: vec![],
            ..arrow()
        };
        let side = Side::Attacker;
        // No stack there, off the field, not aimed at a stack.
        assert_eq!(a.refusal(field, 91, side), Hint::SelectTarget);
        assert_eq!(a.refusal(field, -1, side), Hint::SelectTarget);
        assert_eq!(a.refusal(field, 187, side), Hint::SelectTarget);
        let hexes = Aim {
            stacks: false,
            ..a.clone()
        };
        assert_eq!(hexes.refusal(field, 86, side), Hint::SelectTarget);
        // Anti-Magic, a hostile spell at an own stack, a friendly one at an enemy.
        assert_eq!(a.refusal(field, 100, side), Hint::Protects(9, 3));
        let high = Aim {
            level: 4,
            ..a.clone()
        };
        assert_eq!(high.refusal(field, 100, side), Hint::Immune(3, 7));
        let none = Aim {
            anti_magic: None,
            ..a.clone()
        };
        assert_eq!(none.refusal(field, 100, side), Hint::Immune(3, 7));
        assert_eq!(a.refusal(field, 86, side), Hint::HostileOnly(7));
        let bless = Aim {
            polarity: 1,
            ..a.clone()
        };
        assert_eq!(bless.refusal(field, 90, side), Hint::FriendlyOnly(7));
        assert_eq!(bless.refusal(field, 86, side), Hint::Immune(0, 7));
        // E17 M5 T8b2 (R4b-2): a stack the engine lets it be cast at passes every test, its
        // client's own readings (here Anti-Magic on stack 3) too.
        assert_eq!(arrow().refusal(field, 90, side), Hint::SelectTarget);
        assert_eq!(arrow().refusal(field, 100, side), Hint::SelectTarget);
        // Neither: the immunity, a raising spell's "does not work on".
        let neutral = Aim {
            polarity: 0,
            ..a.clone()
        };
        assert_eq!(neutral.refusal(field, 90, side), Hint::Immune(1, 7));
        for original in [38, 39, 40] {
            let raise = Aim {
                original,
                ..neutral.clone()
            };
            assert_eq!(
                raise.refusal(field, 90, side),
                Hint::NotOn(7, 1),
                "{original}"
            );
        }
    });
}

#[test]
fn an_area_marks_its_living_stacks() {
    let mut o = battle();
    // A dead stack on 89 is not marked; one in a side column (hex 85, column 0) neither.
    o.stacks[2].hex = 89;
    o.stacks[2].alive = false;
    let mut edge = o.stacks[1].clone();
    edge.id = 4;
    edge.hex = 85;
    o.stacks.push(edge);
    with(&o, |field| {
        let fireball = Aim {
            original: 21,
            area: Some(Area {
                radius: 1,
                centre: true,
            }),
            ..arrow()
        };
        // Around 87 only the acting stack on 86 (the dead one on 89 is two hexes away).
        assert_eq!(fireball.marks(field, 87), BTreeSet::from([0]));
        assert_eq!(fireball.marks(field, 90), BTreeSet::from([1]));
        assert_eq!(fireball.marks(field, -1), BTreeSet::new());
        assert_eq!(fireball.marks(field, 0xFC), BTreeSet::new());
        // Frost Ring leaves its centre out; Inferno reaches two hexes.
        let ring = Aim {
            area: Some(Area {
                radius: 1,
                centre: false,
            }),
            ..fireball.clone()
        };
        assert_eq!(ring.marks(field, 90), BTreeSet::new());
        let inferno = Aim {
            area: Some(Area {
                radius: 2,
                centre: true,
            }),
            ..fireball.clone()
        };
        assert_eq!(inferno.marks(field, 88), BTreeSet::from([0, 1]));
        // A side column's hex: the area around it, without its own column.
        assert_eq!(fireball.marks(field, 85), BTreeSet::from([0]));
        // No area, no marks.
        assert_eq!(arrow().marks(field, 90), BTreeSet::new());
    });
}

#[test]
fn a_two_hex_stack_is_taken_at_both_hexes_off_the_side_columns() {
    // E17 M5 T8b2 (R4b-7): the defender's stack 1 two hexes wide on 90 (its other hex 89, the
    // defender facing left), its stack 3 on 103 with its other hex 102 in the side column 0.
    let mut o = battle();
    o.stacks[1].two_hex = true;
    o.stacks[3].two_hex = true;
    o.stacks[3].hex = 103;
    with(&o, |field| {
        let a = arrow();
        let side = Side::Attacker;
        for hex in [89, 90] {
            assert_eq!(a.takes(field, i32::from(hex)), Some(hex));
            assert_eq!(a.target(field, hex), Some(Target::Stack(1)));
            assert_eq!(a.line(field, hex), Hint::Cast(7, Some(1)));
        }
        assert_eq!(a.takes(field, 103), Some(103));
        // The side column's hex: refused, "Select Spell Target" (R4b-2: every test passes there),
        // its target and line the stack's still.
        assert_eq!(a.takes(field, 102), None);
        assert_eq!(a.target(field, 102), Some(Target::Stack(3)));
        assert_eq!(a.line(field, 102), Hint::Cast(7, Some(3)));
        assert_eq!(a.refusal(field, 102, side), Hint::SelectTarget);
        // A stack it may not be cast at there gets its reason.
        let only = Aim {
            targets: vec![Target::Stack(1)],
            ..arrow()
        };
        assert_eq!(only.refusal(field, 102, side), Hint::Immune(3, 7));
    });
}

#[test]
fn of_two_corpses_on_a_hex_the_lowest_number_is_raised() {
    // E17 M5 T8b2 (R4b-7): two dead stacks on hex 53 (5 listed first), both raisable: the cast
    // and its line take stack 4; with 5 alone listed, 5.
    let mut o = battle();
    for id in [5, 4] {
        let mut dead = o.stacks[1].clone();
        dead.id = id;
        dead.hex = 53;
        dead.alive = false;
        o.stacks.push(dead);
    }
    with(&o, |field| {
        let raise = Aim {
            original: 38,
            polarity: 1,
            targets: vec![Target::Stack(4), Target::Stack(5)],
            ..arrow()
        };
        assert_eq!(raise.target(field, 53), Some(Target::Stack(4)));
        assert_eq!(raise.takes(field, 53), Some(53));
        assert_eq!(raise.line(field, 53), Hint::Cast(7, Some(4)));
        let five = Aim {
            targets: vec![Target::Stack(5)],
            ..raise.clone()
        };
        assert_eq!(five.target(field, 53), Some(Target::Stack(5)));
        assert_eq!(five.line(field, 53), Hint::Cast(7, Some(5)));
        // No living stack there: the default refusal.
        let none = Aim {
            targets: vec![],
            ..raise
        };
        assert_eq!(none.refusal(field, 53, Side::Attacker), Hint::SelectTarget);
    });
}
