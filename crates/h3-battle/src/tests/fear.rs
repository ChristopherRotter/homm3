//! Fear (E04b T7c3; BAT-MAG-060, quirk BAT-QUIRK-18), after the original's code
//! (`docs/formats/installation.md` §7.21, "Fear, read for T7c3").

use std::vec;
use std::vec::Vec;

use h3_content::{BattleParamsDef, Ruleset};

use super::spells::spell;
use super::support::*;
use crate::morale::frightened;
use crate::state::StackFlags;
use crate::{BattleState, Command, Event, EventKind, StackId};

/// Puts stack `id` under the other side's control (Hypnotize, BAT-MAG-026).
fn hypnotize(s: &mut BattleState, r: &Ruleset, id: u8) {
    let id_spell = spell(r, "hypnotize");
    let cast = crate::effects::TimedCast {
        id: id_spell,
        spell: r.spell(id_spell).unwrap(),
        mastery: 0,
        rounds: 5,
        own: crate::magic::SpellBonus::default(),
    };
    crate::effects::apply(s, r, cast, StackId(id), &mut Vec::<Event>::new());
}

/// The frozen stacks in `events`.
fn frozen(events: &[Event]) -> Vec<u8> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Frightened { stack } => Some(stack.0),
            _ => None,
        })
        .collect()
}

/// Good morale that always triggers.
fn always(b: &mut BattleParamsDef) {
    b.morale_dice = 255;
    b.good_morale_chances = std::vec![255, 255, 255];
}

// covers: BAT-MAG-060, BAT-MOR-004
#[test]
fn fear_freezes_the_other_sides_stacks_but_not_at_the_battles_first_activation() {
    let r = ruleset();
    // The fast stack (speed 9) acts first: the battle's first activation has no freezing checks.
    let mut s = battle(
        &r,
        &[(0, "fast", 1), (1, "skeleton", 1)],
        &[(0, "azure", 1)],
        1,
    );
    assert_eq!(s.active(), Some(StackId(0)));
    // The azure never frightens itself, a skeleton has no morale and is never frozen.
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!((frozen(&events), s.active()), (vec![], Some(StackId(2))));
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!((frozen(&events), s.active()), (vec![], Some(StackId(1))));
    // In round 2 the fast stack is frozen at its pick: it loses its turn and has acted.
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(frozen(&events), vec![0]);
    assert!(s.stacks()[0].flags.contains(StackFlags::ACTED));
    assert_eq!((s.round(), s.active()), (2, Some(StackId(2))));
}

// covers: BAT-MAG-060
#[test]
fn fear_takes_the_largest_chance_of_the_living_frighteners_outside_the_wait_phase() {
    let r = ruleset();
    let mut spooked = 0;
    for seed in 0..1000 {
        let mut s = battle(
            &r,
            &[(0, "serf", 1)],
            &[(0, "spook", 1), (1, "azure", 1)],
            seed,
        );
        assert!(frightened(&mut s, &r, StackId(0)));
        // Outside the normal phase nothing freezes.
        s.wait_phase = true;
        assert!(!frightened(&mut s, &r, StackId(0)));
        s.wait_phase = false;
        // Without the azure, the spook's 1 %.
        s.stacks.as_mut_slice()[2].count = 0;
        spooked += usize::from(frightened(&mut s, &r, StackId(0)));
        // Without either, none.
        s.stacks.as_mut_slice()[1].count = 0;
        assert!(!frightened(&mut s, &r, StackId(0)));
    }
    assert!((1..30).contains(&spooked), "{spooked}");
}

// covers: BAT-MAG-060
#[test]
fn a_hypnotized_azure_frightens_the_side_that_controls_it() {
    // BAT-QUIRK-18 (the rule): the frightened stack's side is its controller's, the azures count
    // by their real side.
    let r = ruleset();
    let mut s = battle(
        &r,
        &[(0, "serf", 1)],
        &[(0, "azure", 1), (1, "fixed", 1)],
        1,
    );
    hypnotize(&mut s, &r, 1);
    // The attacker controls the azure now, which still frightens the attacker's stacks and spares
    // the defender's.
    assert!(frightened(&mut s, &r, StackId(0)));
    assert!(!frightened(&mut s, &r, StackId(2)));
    // A hypnotized defender stack fights for the attacker: its own side's azure frightens it.
    hypnotize(&mut s, &r, 2);
    assert!(frightened(&mut s, &r, StackId(2)));
}

// covers: BAT-MAG-060, BAT-MOR-003
#[test]
fn fear_can_freeze_a_good_morale_turn_but_not_in_the_wait_phase() {
    let r = ruleset_with("t:faithful", always);
    // Castle pikemen (+1 morale) against a azure.
    let mut s = battle(&r, &[(0, "pikeman_c", 5)], &[(0, "azure", 1)], 1);
    place(&mut s, &r, 0, 2, 5);
    activate(&mut s, 0);
    let events = act(&mut s, &r, Command::Move { to: at(&r, 3, 5) }).unwrap();
    assert!(
        events
            .iter()
            .any(|e| e.kind == EventKind::GoodMorale { stack: StackId(0) })
    );
    assert_eq!((frozen(&events), s.active()), (vec![0], Some(StackId(1))));
    assert!(s.stacks()[0].flags.contains(StackFlags::ACTED));
    // A stack that acts from waiting keeps its good-morale turn.
    let mut s = battle(&r, &[(0, "pikeman_c", 5)], &[(0, "azure", 1)], 1);
    place(&mut s, &r, 0, 2, 5);
    activate(&mut s, 0);
    s.stacks.as_mut_slice()[0]
        .flags
        .set(StackFlags::WAITED, true);
    s.wait_phase = true;
    let events = act(&mut s, &r, Command::Move { to: at(&r, 3, 5) }).unwrap();
    assert_eq!((frozen(&events), s.active()), (vec![], Some(StackId(0))));
}
