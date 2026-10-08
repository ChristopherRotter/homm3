//! Creatures that cast as their stack's action (E04b T7b1; BAT-MAG-048), after the original's
//! code (`docs/formats/installation.md` §7.21).

use std::vec;
use std::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::BattleSide;

use super::spells::{arena, mage, spell, stack};
use super::support::*;
use crate::state::StackFlags;
use crate::{
    BattleState, CastTarget, CastTargets, Command, CommandError, Event, EventKind, Hex, StackId,
    Status, creature_cast_spell, creature_cast_targets, observe, stack_spell,
};

/// The stacks the active stack of `side` may cast at now, by id.
fn targets(s: &BattleState, r: &Ruleset, side: BattleSide) -> Vec<u8> {
    let mut out = CastTargets::new();
    creature_cast_targets(s, r, side, &mut out);
    out.iter()
        .map(|t| match *t {
            CastTarget::Stack(id) => id.0,
            other => panic!("not a stack: {other:?}"),
        })
        .collect()
}

/// The command of a stack's cast at stack `id`.
fn at_stack(id: u8) -> Command {
    Command::CreatureCast { target: stack(id) }
}

/// The effects `events` apply, as `(target, spell, rounds)`.
fn applied(events: &[Event]) -> Vec<(u8, h3_content::SpellId, Option<u16>)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::EffectApplied {
                target,
                spell,
                rounds,
            } => Some((target.0, spell, rounds)),
            _ => None,
        })
        .collect()
}

// covers: BAT-MAG-048
#[test]
fn a_caster_casts_its_spell_at_another_stack_of_its_side_and_ends_its_turn() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("conjurer", 5, 2, 3), ("fixed", 3, 2, 7)],
        &[("fixed", 3, 13, 5)],
        1,
    );
    let haste = spell(&r, "haste");
    assert_eq!(stack_spell(&s, &r, StackId(0)), Some(haste));
    assert_eq!(stack_spell(&s, &r, StackId(1)), None);
    assert_eq!(stack_spell(&s, &r, StackId(9)), None);
    assert_eq!(
        creature_cast_spell(&s, &r, BattleSide::Attacker),
        Some(haste)
    );
    assert_eq!(creature_cast_spell(&s, &r, BattleSide::Defender), None);
    // Not itself (the original's cursor), not the enemy (a positive spell).
    assert_eq!(targets(&s, &r, BattleSide::Attacker), vec![1]);
    assert!(targets(&s, &r, BattleSide::Defender).is_empty());
    assert_eq!((s.casts_left(StackId(0)), s.casts_left(StackId(1))), (2, 0));
    let speed = s.stacks()[1].speed_now();
    let events = act(&mut s, &r, at_stack(1)).unwrap();
    assert_eq!(
        events[0].kind,
        EventKind::StackCast {
            stack: StackId(0),
            spell: haste,
            target: stack(1),
        }
    );
    // Advanced: +5 speed (Haste's effect [3, 3, 5, 5]); power 6: six rounds; no hero needed.
    assert_eq!(applied(&events), vec![(1, haste, Some(6))]);
    assert_eq!(s.stacks()[1].speed_now(), speed + 5);
    // One cast used; the turn ends as after an action.
    assert_eq!(s.casts_left(StackId(0)), 1);
    assert!(s.stacks()[0].flags.contains(StackFlags::ACTED));
    assert_ne!(s.active(), Some(StackId(0)));
    assert_eq!(observe(&s, &r, BattleSide::Defender).casts_left[0], 1);
    assert!(events.iter().any(|e| e.kind
        == EventKind::TurnStarted {
            stack: s.active().unwrap()
        }));
}

// covers: BAT-MAG-048
#[test]
fn a_cast_needs_a_caster_with_casts_and_a_target_of_its_spell() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[
            ("conjurer", 5, 2, 3),
            ("fixed", 3, 2, 7),
            ("fixed", 3, 2, 9),
        ],
        &[("fixed", 3, 13, 5)],
        1,
    );
    let before = s.clone();
    for (target, error) in [
        (stack(0), CommandError::InvalidSpellTarget),
        (stack(3), CommandError::InvalidSpellTarget),
        (stack(40), CommandError::InvalidSpellTarget),
        (CastTarget::None, CommandError::InvalidSpellTarget),
        (CastTarget::Hex(Hex(30)), CommandError::InvalidSpellTarget),
    ] {
        let command = Command::CreatureCast { target };
        assert_eq!(act(&mut s, &r, command), Err(error), "{target:?}");
        assert_eq!(
            crate::apply(
                &mut s,
                &r,
                BattleSide::Defender,
                command,
                &mut Vec::<Event>::new()
            ),
            Err(CommandError::NotYourTurn)
        );
    }
    assert_eq!(s, before);
    // A dead stack is no target.
    s.stacks.as_mut_slice()[2].count = 0;
    assert_eq!(targets(&s, &r, BattleSide::Attacker), vec![1]);
    assert_eq!(
        act(&mut s, &r, at_stack(2)),
        Err(CommandError::InvalidSpellTarget)
    );
    // Without casts left it may not cast.
    s.casts.left[0] = 0;
    assert_eq!(creature_cast_spell(&s, &r, BattleSide::Attacker), None);
    assert!(targets(&s, &r, BattleSide::Attacker).is_empty());
    assert_eq!(act(&mut s, &r, at_stack(1)), Err(CommandError::CannotCast));
    // Nor may a stack that is no caster.
    activate(&mut s, 1);
    assert_eq!(act(&mut s, &r, at_stack(0)), Err(CommandError::CannotCast));
    assert!(targets(&s, &r, BattleSide::Attacker).is_empty());
}

// covers: BAT-MAG-048
#[test]
fn a_hypnotized_caster_casts_for_its_controller() {
    let r = ruleset();
    let mut s = arena(
        &r,
        [None, None],
        &[("fixed", 3, 2, 3), ("fixed", 3, 2, 7)],
        &[("conjurer", 5, 13, 5), ("fixed", 3, 13, 7)],
        1,
    );
    let st = &mut s.stacks.as_mut_slice()[2];
    st.spells.status = st.spells.status.with(Status::HYPNOTIZED, true);
    activate(&mut s, 2);
    // The attacker commands it, and its spell reaches the attacker's stacks (the original's
    // side of the caster, +0x288).
    assert_eq!(s.side_to_act(), Some(BattleSide::Attacker));
    assert!(targets(&s, &r, BattleSide::Defender).is_empty());
    assert_eq!(targets(&s, &r, BattleSide::Attacker), vec![0, 1]);
    let events = act(&mut s, &r, at_stack(0)).unwrap();
    assert_eq!(applied(&events), vec![(0, spell(&r, "haste"), Some(6))]);
}

// covers: BAT-MAG-048
#[test]
fn the_stack_a_creature_casts_at_never_rolls_for_resistance() {
    let r = ruleset();
    let weaken = spell(&r, "weaken");
    for seed in 0..40 {
        let mut s = arena(
            &r,
            [None, None],
            &[("warlock", 5, 2, 3)],
            &[("dwarf", 5, 13, 5)],
            seed,
        );
        // Basic: −3 attack; power 4: four rounds; the dwarves' 20 % never helps them.
        let attack = s.stacks()[1].attack_now(false);
        let events = act(&mut s, &r, at_stack(1)).unwrap();
        assert_eq!(applied(&events), vec![(1, weaken, Some(4))], "seed {seed}");
        assert!(
            !events
                .iter()
                .any(|e| matches!(e.kind, EventKind::SpellResisted { .. }))
        );
        assert_eq!(s.stacks()[1].attack_now(false), attack - 3);
    }
}

// covers: BAT-MAG-048, BAT-MOR-003
#[test]
fn good_morale_may_follow_a_cast() {
    // Good morale that always triggers (BAT-MOR-003), as after a move, an attack or a shot (the
    // original's 0x464920).
    let r = ruleset_with("t:faithful", |b| {
        b.morale_dice = 255;
        b.good_morale_chances = vec![255, 255, 255];
    });
    let mut s = arena(
        &r,
        [None, None],
        &[("conjurer", 5, 2, 3), ("fixed", 3, 2, 7)],
        &[("fixed", 3, 13, 5)],
        1,
    );
    s.stacks.as_mut_slice()[0].morale = 1;
    let events = act(&mut s, &r, at_stack(1)).unwrap();
    assert!(
        events
            .iter()
            .any(|e| e.kind == EventKind::GoodMorale { stack: StackId(0) })
    );
    assert_eq!(s.active(), Some(StackId(0)));
    // Its second cast in a row, its last.
    act(&mut s, &r, at_stack(1)).unwrap();
    assert_eq!(s.casts_left(StackId(0)), 0);
    assert_ne!(s.active(), Some(StackId(0)));
}

// covers: BAT-MAG-048, BAT-MAG-038
#[test]
fn a_created_stack_has_its_creatures_casts() {
    let r = ruleset();
    let mut hero = mage(&r, &["copy"]);
    hero.magic.schools[0] = 1;
    hero.magic.mana = 100;
    let mut s = arena(
        &r,
        [Some(hero), None],
        &[("conjurer", 5, 5, 5), ("fixed", 3, 2, 9)],
        &[("fixed", 3, 13, 5)],
        1,
    );
    act(&mut s, &r, at_stack(1)).unwrap();
    activate(&mut s, 1);
    let events = act(
        &mut s,
        &r,
        Command::Cast {
            spell: spell(&r, "copy"),
            target: stack(0),
            second: CastTarget::None,
        },
    )
    .unwrap();
    assert!(events.iter().any(|e| matches!(
        e.kind,
        EventKind::StackCreated {
            stack: StackId(3),
            ..
        }
    )));
    assert_eq!((s.casts_left(StackId(0)), s.casts_left(StackId(3))), (1, 2));
}
