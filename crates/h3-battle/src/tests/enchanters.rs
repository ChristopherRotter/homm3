//! The periodic caster, the Enchanter (E04b T7b4; BAT-MAG-053), after the original's code
//! (`docs/formats/installation.md` §7.21, "The Enchanter, re-read for T7b4").

use std::vec;
use std::vec::Vec;

use h3_content::{Ruleset, SpellId};
use h3_contracts::{BattleSetup, BattleSide};

use super::spells::spell;
use super::support::*;
use crate::state::StackTraits;
use crate::{BattleState, Command, Event, EventKind, NoopSink, Stack, StackId, Status, observe};

/// The periodic casts in `events`, as `(stack, spell)`.
fn periodic(events: &[Event]) -> Vec<(u8, SpellId)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::PeriodicCast { stack, spell } => Some((stack.0, spell)),
            _ => None,
        })
        .collect()
}

/// The stacks `events` apply an effect to, with the effect's rounds.
fn applied(events: &[Event]) -> Vec<(u8, Option<u16>)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::EffectApplied { target, rounds, .. } => Some((target.0, rounds)),
            _ => None,
        })
        .collect()
}

/// Starts a battle of `setup`, returning the state and the events of its start.
fn start(r: &Ruleset, setup: &BattleSetup, seed: u64) -> (BattleState, Vec<Event>) {
    let mut events = Vec::new();
    let s = BattleState::start(r, setup, seed, &mut events).unwrap();
    (s, events)
}

/// Lets every stack defend for `turns` turns, returning each periodic cast with its round.
fn defend(s: &mut BattleState, r: &Ruleset, turns: usize) -> Vec<(u16, u8, SpellId)> {
    let mut out = Vec::new();
    for _ in 0..turns {
        let events = act(s, r, Command::Defend).unwrap();
        out.extend(
            periodic(&events)
                .into_iter()
                .map(|(stack, spell)| (s.round(), stack, spell)),
        );
    }
    out
}

/// The test Enchanter's spells that count when its side's first stack is a full-health
/// shooter and the enemy's first a melee stack: Haste, Slow, Stone Skin, Bless and Weakness
/// (not Air Shield: no enemy shoots; not Bloodlust: a shooter; not Cure: nobody wounded).
fn counted(r: &Ruleset) -> [SpellId; 5] {
    ["haste", "sloth", "skin", "bless", "weaken"].map(|n| spell(r, n))
}

// covers: BAT-MAG-053
#[test]
fn every_battle_starts_with_the_enchanters_ready() {
    // BAT-QUIRK-14: the original keeps the count of the battle before; here every battle starts
    // with 255, so the side's Enchanters cast at their first activation in round 1.
    let r = ruleset();
    let setup = setup(
        &r,
        &[(0, "chanter", 5), (1, "fixed", 3)],
        &[(0, "fixed", 4)],
    );
    let mut events = Vec::new();
    let fresh = BattleState::start(&r, &setup, 3, &mut NoopSink).unwrap();
    assert_eq!(fresh.periodic_rounds(BattleSide::Defender), u8::MAX);
    let (s, start_events) = start(&r, &setup, 3);
    events.extend(start_events);
    assert!(s.stacks()[0].has(StackTraits::PERIODIC_CASTER));
    assert!(!s.stacks()[1].has(StackTraits::PERIODIC_CASTER));
    let casts = periodic(&events);
    assert_eq!(casts.len(), 1);
    assert_eq!(casts[0].0, 0);
    assert!(counted(&r).contains(&casts[0].1), "{casts:?}");
    // Its side's count is back to 0, the other side's stays; the Enchanter still has its turn
    // and no cast was used.
    assert_eq!(s.round(), 1);
    assert_eq!(s.periodic_rounds(BattleSide::Attacker), 0);
    assert_eq!(s.periodic_rounds(BattleSide::Defender), u8::MAX);
    assert_eq!(s.active(), Some(StackId(0)));
    assert_eq!(s.casts_left(StackId(0)), 0);
    // The observation shows both counts (public).
    let obs = observe(&s, &r, BattleSide::Defender);
    assert_eq!(obs.periodic_rounds, [0, u8::MAX]);
}

// covers: BAT-MAG-053
#[test]
fn the_cast_is_the_expert_mass_version_for_three_rounds() {
    let r = ruleset();
    let setup = setup(
        &r,
        &[(0, "chanter", 5), (1, "fixed", 3)],
        &[(0, "fixed", 4)],
    );
    // Over several seeds the draw reaches positive and negative spells; each reaches every stack
    // of the side it aims at, for the caster's power in rounds (Expert's mass version, power 3).
    let mut drawn = Vec::new();
    for seed in 1..=12 {
        let (_, events) = start(&r, &setup, seed);
        let [(_, cast)] = periodic(&events)[..] else {
            panic!("one cast: {events:?}");
        };
        let negative = cast == spell(&r, "sloth") || cast == spell(&r, "weaken");
        let expected = if negative {
            vec![(2, Some(3))]
        } else {
            vec![(0, Some(3)), (1, Some(3))]
        };
        assert_eq!(applied(&events), expected, "{cast:?}");
        drawn.push(negative);
    }
    assert!(drawn.contains(&true) && drawn.contains(&false));
}

// covers: BAT-MAG-053
#[test]
fn a_side_casts_once_in_three_round_starts() {
    let r = ruleset();
    // An Enchanter on each side, and a stack of each that waits its turn.
    let setup = setup(
        &r,
        &[(0, "chanter", 5), (1, "fixed", 3)],
        &[(0, "fixed", 4), (1, "chanter", 2)],
    );
    let (mut s, events) = start(&r, &setup, 5);
    assert_eq!(periodic(&events).len(), 1);
    // The defender's Enchanter casts at its first activation too; then each side once in three
    // round starts: rounds 4 and 7 (its count 3 at their starts).
    let casts = defend(&mut s, &r, 4 * 7 - 1);
    let rounds = |stack| {
        casts
            .iter()
            .filter(|&&(_, s, _)| s == stack)
            .map(|&(round, ..)| round)
            .collect::<Vec<_>>()
    };
    assert_eq!(rounds(0), [4, 7]);
    assert_eq!(rounds(3), [1, 4, 7]);
    assert_eq!(s.round(), 7);
    assert_eq!(s.periodic_rounds(BattleSide::Attacker), 0);
}

// covers: BAT-MAG-053
#[test]
fn nothing_counts_nothing_is_cast_and_the_count_stays() {
    let r = ruleset();
    // The first stack is a full-health shooter: neither Bloodlust nor Cure counts.
    let setup = setup(
        &r,
        &[(0, "archer", 5), (1, "luster", 3)],
        &[(0, "fixed", 4)],
    );
    let r_first = ruleset_with("t:first_stack", |_| {});
    let (mut s, events) = start(&r_first, &setup, 2);
    assert!(periodic(&events).is_empty());
    assert_eq!(s.periodic_rounds(BattleSide::Attacker), u8::MAX);
    // Wounded, the first stack makes Cure count: the next activation casts it.
    s.stacks.as_mut_slice()[0].top_hp -= 4;
    let casts = defend(&mut s, &r_first, 3);
    assert_eq!(casts, [(2, 1, spell(&r, "cure"))]);
    assert_eq!(s.stacks()[0].top_hp, s.stacks()[0].max_hp);
    assert_eq!(s.periodic_rounds(BattleSide::Attacker), 0);
}

// covers: BAT-MAG-053
#[test]
fn only_the_first_stack_decides_with_quirk_bat_quirk_13() {
    // Survey S-12: the original tests a spell on the first stack of the side it aims at.
    let setup = setup(
        &ruleset(),
        &[(0, "archer", 5), (1, "luster", 3), (2, "fixed", 2)],
        &[(0, "fixed", 4)],
    );
    let first = ruleset_with("t:first_stack", |_| {});
    let (_, events) = start(&first, &setup, 2);
    assert!(periodic(&events).is_empty());
    // Fixed (strict): Bloodlust counts on the melee stacks, and its mass version reaches every
    // own stack that is no war machine.
    let r = ruleset();
    let (_, events) = start(&r, &setup, 2);
    assert_eq!(periodic(&events), [(1, spell(&r, "lust"))]);
    assert_eq!(applied(&events), [(0, Some(3)), (1, Some(3)), (2, Some(3))]);
}

// covers: BAT-MAG-053
#[test]
fn a_dead_first_stack_accepts_nothing_with_quirk_bat_quirk_13() {
    let setup = setup(
        &ruleset(),
        &[(0, "fixed", 5), (1, "chanter", 3)],
        &[(0, "fixed", 4), (1, "fixed", 4)],
    );
    let kill = |s: &mut BattleState, id: usize| {
        let stack: &mut Stack = &mut s.stacks.as_mut_slice()[id];
        stack.count = 0;
        stack.top_hp = 0;
    };
    let draws = |r: &Ruleset, dead: usize| {
        (1..=12)
            .flat_map(|seed| {
                let mut s = BattleState::new(r, &setup, seed).unwrap();
                kill(&mut s, dead);
                // The Enchanter acts first: start its turn again.
                s.casts.periodic = [u8::MAX; 2];
                let mut events = Vec::new();
                crate::casters::periodic(&mut s, r, StackId(1), &mut events);
                periodic(&events)
            })
            .map(|(_, spell)| spell)
            .collect::<Vec<_>>()
    };
    let first = ruleset_with("t:first_stack", |_| {});
    let negative = [spell(&first, "sloth"), spell(&first, "weaken")];
    // Its own first stack dead: only the spells aimed at the enemy count.
    let own_dead = draws(&first, 0);
    assert_eq!(own_dead.len(), 12);
    assert!(
        own_dead.iter().all(|s| negative.contains(s)),
        "{own_dead:?}"
    );
    // The enemy's first stack dead: only the spells aimed at its own side.
    let enemy_dead = draws(&first, 2);
    assert_eq!(enemy_dead.len(), 12);
    assert!(enemy_dead.iter().all(|s| !negative.contains(s)));
    // Fixed: any living stack of the side counts.
    let r = ruleset();
    let fixed = draws(&r, 0);
    assert!(fixed.iter().any(|s| !negative.contains(s)));
}

// covers: BAT-MAG-053
#[test]
fn a_hypnotized_enchanter_casts_for_its_controller() {
    // The acting side is the controller's (0x465330): its count and its side's stacks.
    let r = ruleset();
    let setup = setup(
        &r,
        &[(0, "fixed", 5)],
        &[(0, "fixed", 4), (1, "chanter", 3)],
    );
    let mut s = BattleState::new(&r, &setup, 4).unwrap();
    // Hypnotized (the status alone decides the controller).
    let enchanter = &mut s.stacks.as_mut_slice()[2];
    enchanter.spells.status = enchanter.spells.status.with(Status::HYPNOTIZED, true);
    s.casts.periodic = [u8::MAX, 0];
    let mut events = Vec::new();
    crate::casters::periodic(&mut s, &r, StackId(2), &mut events);
    let [(_, cast)] = periodic(&events)[..] else {
        panic!("one cast: {events:?}");
    };
    assert_eq!(s.periodic_rounds(BattleSide::Attacker), 0);
    assert_eq!(s.periodic_rounds(BattleSide::Defender), 0);
    // A positive spell reaches the controller's stack, a negative one its owner's side, itself
    // included.
    let negative = cast == spell(&r, "sloth") || cast == spell(&r, "weaken");
    let targets: Vec<u8> = applied(&events).into_iter().map(|(t, _)| t).collect();
    assert_eq!(targets, if negative { vec![1, 2] } else { vec![0] });
}

// covers: BAT-MAG-053
#[test]
fn a_berserk_enchanter_casts_before_its_automatic_turn() {
    // The original activates a berserk stack (its action comes after): its cast comes first.
    let r = ruleset();
    let setup = setup(
        &r,
        &[(0, "fixed", 5), (1, "chanter", 3)],
        &[(0, "fixed", 4)],
    );
    let (mut s, events) = start(&r, &setup, 6);
    assert_eq!(periodic(&events).len(), 1);
    let berserk = spell(&r, "berserk");
    let cast = crate::effects::TimedCast {
        id: berserk,
        spell: r.spell(berserk).unwrap(),
        mastery: 0,
        rounds: 5,
        own: crate::magic::SpellBonus::default(),
    };
    crate::effects::apply(&mut s, &r, cast, StackId(1), &mut Vec::<Event>::new());
    assert!(s.stacks()[1].spells.status.contains(Status::BERSERK));
    s.casts.periodic[0] = u8::MAX;
    // The Enchanter defends; the next round's activation casts, then the berserk turn follows.
    let mut events = Vec::new();
    for _ in 0..3 {
        events.extend(act(&mut s, &r, Command::Defend).unwrap());
    }
    let cast = events
        .iter()
        .position(|e| matches!(e.kind, EventKind::PeriodicCast { stack, .. } if stack.0 == 1))
        .unwrap();
    let turn = events
        .iter()
        .rposition(|e| matches!(e.kind, EventKind::TurnStarted { stack } if stack.0 == 1))
        .unwrap();
    assert!(turn < cast);
    // Its automatic action (a move or a strike) follows the cast.
    let action = events
        .iter()
        .position(|e| {
            matches!(e.kind, EventKind::Moved { stack, .. } if stack.0 == 1)
                || matches!(e.kind, EventKind::Struck { attacker, .. } if attacker.0 == 1)
        })
        .unwrap();
    assert!(cast < action);
    assert_eq!(s.round(), 2);
    assert_ne!(s.active(), Some(StackId(1)));
}

// covers: BAT-MAG-053
#[test]
fn an_enchanter_that_lost_its_turns_casts_nothing() {
    // Blinded during its own action, it gets no activation at a good-morale turn either (the
    // original's pick passes it over).
    let r = ruleset();
    let setup = setup(&r, &[(0, "chanter", 5)], &[(0, "fixed", 4)]);
    let mut s = BattleState::new(&r, &setup, 1).unwrap();
    let blind = spell(&r, "blind");
    let cast = crate::effects::TimedCast {
        id: blind,
        spell: r.spell(blind).unwrap(),
        mastery: 0,
        rounds: 5,
        own: crate::magic::SpellBonus::default(),
    };
    crate::effects::apply(&mut s, &r, cast, StackId(0), &mut Vec::<Event>::new());
    s.casts.periodic = [u8::MAX; 2];
    let mut events = Vec::new();
    crate::casters::periodic(&mut s, &r, StackId(0), &mut events);
    assert!(periodic(&events).is_empty());
    assert_eq!(s.periodic_rounds(BattleSide::Attacker), u8::MAX);
}
