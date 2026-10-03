//! Tests of war machines (BAT-WAR-001..004).

use std::vec;
use std::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::{BattleResult, BattleSide};

use super::support::*;
use crate::event::StrikeFlags;
use crate::{
    BattleSetup, BattleState, Command, Event, EventKind, SetupError, StackFlags, StackId,
    StackTraits,
};

fn creature(r: &Ruleset, name: &str) -> h3_content::CreatureTypeId {
    r.creature_by_key(&std::format!("t:{name}")).unwrap()
}

fn with_machines(
    r: &Ruleset,
    attacker: &[Spec],
    defender: &[Spec],
    machines: [&[&str]; 2],
) -> BattleSetup {
    BattleSetup {
        war_machines: machines.map(|m| m.iter().map(|k| creature(r, k)).collect()),
        ..setup(r, attacker, defender)
    }
}

fn start(r: &Ruleset, s: &BattleSetup, seed: u64) -> BattleState {
    BattleState::new(r, s, seed).unwrap()
}

fn shots_by(events: &[Event], attacker: u8) -> Vec<(u8, StrikeFlags)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Struck {
                attacker: a,
                target,
                flags,
                ..
            } if a.0 == attacker => Some((target.0, flags)),
            _ => None,
        })
        .collect()
}

fn healed(events: &[Event]) -> Vec<(u8, u8, u32)> {
    events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::Healed {
                healer,
                target,
                hit_points,
            } => Some((healer.0, target.0, hit_points)),
            _ => None,
        })
        .collect()
}

/// Lets every active stack defend until a round has passed; returns all events.
fn defend_round(s: &mut BattleState, r: &Ruleset) -> Vec<Event> {
    let round = s.round();
    let mut events = Vec::new();
    while s.round() == round && !s.is_over() {
        events.extend(act(s, r, Command::Defend).unwrap());
    }
    events
}

// ---------------------------------------------------------------- BAT-WAR-001 placement, traits

// covers: BAT-WAR-001
#[test]
fn war_machines_stand_at_their_places_after_the_army() {
    let r = ruleset();
    let s = start(
        &r,
        &with_machines(
            &r,
            &[(0, "serf", 1)],
            &[(0, "serf", 1)],
            [&["tent", "ballista", "cart"], &["tent", "ballista"]],
        ),
        1,
    );
    let placed: Vec<(&str, u8, (i32, i32))> = s
        .stacks()
        .iter()
        .map(|st| {
            let name = r.creature_key(st.creature).unwrap().name();
            (name, st.slot, field(&r).coords(st.hex))
        })
        .collect();
    assert_eq!(
        placed,
        [
            ("serf", 0, (1, 5)),
            ("ballista", 16, (1, 3)),
            ("cart", 17, (1, 1)),
            ("tent", 18, (1, 9)),
            ("serf", 0, (15, 5)),
            ("ballista", 16, (15, 3)),
            ("tent", 18, (15, 9)),
        ]
    );
    // Two-hex machines reach into the side columns.
    assert_eq!(s.stacks()[1].footprint(field(&r)).tail, Some(at(&r, 0, 3)));
    assert_eq!(s.stacks()[6].footprint(field(&r)).tail, Some(at(&r, 16, 9)));
    // War machines are not affected by morale and not part of the army's alignment.
    assert!(
        s.stacks()[1].has(StackTraits::NO_MORALE) && s.stacks()[1].has(StackTraits::WAR_MACHINE)
    );
    assert_eq!((s.stacks()[0].morale, s.stacks()[1].morale), (1, 0));
}

// covers: BAT-WAR-001
#[test]
fn war_machine_setups_are_validated() {
    let r = ruleset();
    let error = |att: &[Spec], machines: [&[&str]; 2], obstacles: &[(i32, i32)]| {
        let mut s = with_machines(&r, att, &[(0, "serf", 1)], machines);
        s.obstacles = obstacles.iter().map(|&(x, y)| at(&r, x, y).0).collect();
        BattleState::new(&r, &s, 1).unwrap_err()
    };
    let invalid = SetupError::InvalidWarMachine {
        side: BattleSide::Attacker,
    };
    let serf: &[Spec] = &[(0, "serf", 1)];
    assert_eq!(error(serf, [&["serf"], &[]], &[]), invalid);
    assert_eq!(error(serf, [&["gadget"], &[]], &[]), invalid);
    assert_eq!(error(serf, [&["cart", "cart"], &[]], &[]), invalid);
    assert_eq!(
        error(serf, [&[], &["ballista"]], &[(15, 3)]),
        SetupError::NoRoom {
            side: BattleSide::Defender,
            slot: 16
        }
    );
    assert_eq!(
        error(&[(0, "ballista", 1)], [&[], &[]], &[]),
        SetupError::WarMachineInArmy {
            side: BattleSide::Attacker,
            slot: 0
        }
    );
    let mut bad = with_machines(&r, serf, serf, [&[], &[]]);
    bad.war_machines[0].push(h3_content::CreatureTypeId::from_raw(999));
    assert_eq!(BattleState::new(&r, &bad, 1).unwrap_err(), invalid);
    // The compiler keeps the machines' places apart (BAT-WAR-001), but an obstacle of the setup
    // can still stand on one: the Ballista's head hex (1, 3) is hex 52.
    let mut s = with_machines(&r, serf, serf, [&["ballista"], &[]]);
    s.obstacles = vec![52];
    assert_eq!(
        BattleState::new(&r, &s, 1).unwrap_err(),
        SetupError::NoRoom {
            side: BattleSide::Attacker,
            slot: 16
        }
    );
}

// covers: BAT-WAR-001
#[test]
fn war_machines_do_not_retaliate_and_do_not_count_for_the_end_of_the_battle() {
    let r = ruleset();
    // 0 = fixed (attacker), 1 = serf ×1 (defender), 2 = defender's tent.
    let mut s = start(
        &r,
        &with_machines(&r, &[(0, "fixed", 10)], &[(0, "serf", 1)], [&[], &["tent"]]),
        1,
    );
    place(&mut s, &r, 0, 14, 8);
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(2),
            from: at(&r, 14, 8),
        },
    )
    .unwrap();
    // 20 damage + 50 % (attack 10 against defense 0) on 75 HP, no retaliation.
    assert_eq!(shots_by(&events, 2), vec![]);
    assert_eq!(s.stacks()[2].top_hp, 45);
    // Killing the last creature ends the battle although the tent stands.
    activate(&mut s, 0);
    place(&mut s, &r, 0, 14, 5);
    let events = act(
        &mut s,
        &r,
        Command::Attack {
            target: StackId(1),
            from: at(&r, 14, 5),
        },
    )
    .unwrap();
    assert!(events.iter().any(|e| e.kind
        == EventKind::BattleEnded {
            result: BattleResult::Victory(BattleSide::Attacker)
        }));
    let outcome = s.outcome().unwrap();
    assert!(outcome.survivors[1].stacks.is_empty());
    assert_eq!(outcome.war_machines[1], vec![creature(&r, "tent")]);
    assert!(outcome.war_machines[0].is_empty());
}

// covers: BAT-END-002, BAT-WAR-002
#[test]
fn a_shot_that_kills_the_last_creatures_of_both_sides_is_a_draw() {
    let r = ruleset();
    // 0 = serf (attacker), 1 = its firebomb, 2 = serf (defender), side by side.
    let mut s = start(
        &r,
        &with_machines(
            &r,
            &[(0, "serf", 1)],
            &[(0, "serf", 1)],
            [&["firebomb"], &[]],
        ),
        1,
    );
    place(&mut s, &r, 0, 9, 5);
    place(&mut s, &r, 2, 10, 5);
    let mut events = act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(s.active(), Some(StackId(2)));
    events.extend(act(&mut s, &r, Command::Defend).unwrap());
    // The firebomb's shot at the defender's serf also hits the attacker's serf: both die and
    // neither side has a living creature left. (Before BAT-END-002 the battle never ended: the
    // firebomb kept getting turns with nothing to shoot.)
    assert_eq!(
        shots_by(&events, 1),
        vec![
            (2, StrikeFlags::RANGED),
            (0, StrikeFlags::RANGED.with(StrikeFlags::SECONDARY))
        ]
    );
    assert_eq!(
        events.last(),
        Some(&Event::public(EventKind::BattleEnded {
            result: BattleResult::Draw
        }))
    );
    assert!(s.is_over());
    assert_eq!(s.result(), Some(BattleResult::Draw));
    assert_eq!(s.winner(), None);
    assert_eq!(s.active(), None);
    // The result is part of the state hash.
    let mut undecided = s.clone();
    undecided.result = None;
    assert_ne!(s.state_hash(), undecided.state_hash());
    let outcome = s.outcome().unwrap();
    assert_eq!(outcome.result, BattleResult::Draw);
    assert!(outcome.survivors.iter().all(|a| a.stacks.is_empty()));
    assert_eq!(
        outcome.war_machines,
        [vec![creature(&r, "firebomb")], vec![]]
    );
    for side in BattleSide::BOTH {
        assert_eq!(
            crate::validate(&s, &r, side, Command::Defend),
            Err(crate::CommandError::BattleOver)
        );
    }
}

// ---------------------------------------------------------------- BAT-WAR-002 automatic shots

// covers: BAT-WAR-002
#[test]
fn the_ballista_shoots_last_in_the_normal_phase_also_when_blocked() {
    let r = ruleset();
    // 0 = serf, 1 = ballista (attacker); 2 = fixed (defender), standing next to the ballista.
    let mut s = start(
        &r,
        &with_machines(
            &r,
            &[(0, "serf", 1)],
            &[(0, "fixed", 10)],
            [&["ballista"], &[]],
        ),
        1,
    );
    place(&mut s, &r, 2, 2, 3);
    // The serf defends, the defender waits: the ballista acts before the wait phase.
    let events = act(&mut s, &r, Command::Defend).unwrap();
    assert!(shots_by(&events, 1).is_empty());
    let events = act(&mut s, &r, Command::Wait).unwrap();
    assert_eq!(shots_by(&events, 1), vec![(2, StrikeFlags::RANGED)]);
    let started: Vec<u8> = events
        .iter()
        .filter_map(|e| match e.kind {
            EventKind::TurnStarted { stack } => Some(stack.0),
            _ => None,
        })
        .collect();
    assert_eq!(started, vec![1, 2]);
    assert_eq!(s.stacks()[1].shots, 23);
    assert!(s.stacks()[1].flags.contains(StackFlags::ACTED));
}

/// The target the attacker's ballista picks among the defender stacks `defender` (extra: the
/// defender's war machines).
fn ballista_target(defender: &[Spec], machines: &[&str]) -> u8 {
    let r = ruleset();
    let mut s = start(
        &r,
        &with_machines(&r, &[(0, "serf", 1)], defender, [&["ballista"], machines]),
        1,
    );
    let events = defend_round(&mut s, &r);
    shots_by(&events, 1)[0].0
}

// covers: BAT-WAR-002
#[test]
fn the_ballista_prefers_creatures_threats_and_value() {
    // Ids: 0 serf, 1 ballista, then the defender stacks from 2.
    // Creatures before war machines.
    assert_eq!(ballista_target(&[(0, "fixed", 1)], &["ballista"]), 2);
    // A shooter can attack next turn, slow walkers cannot: the archers are shot first.
    assert_eq!(
        ballista_target(&[(0, "valuable", 9), (1, "archer", 1)], &[]),
        3
    );
    // Otherwise the higher value: valuable creatures before worthless ones.
    assert_eq!(
        ballista_target(&[(0, "fixed", 1), (1, "valuable", 1)], &[]),
        3
    );
    // Equal value: the lower id.
    assert_eq!(
        ballista_target(&[(0, "valuable", 1), (1, "valuable", 1)], &[]),
        2
    );
}

// covers: BAT-WAR-002
#[test]
fn the_ballista_can_win_the_battle_and_stops_without_shots() {
    let r = ruleset();
    let mut s = start(
        &r,
        &with_machines(
            &r,
            &[(0, "serf", 1)],
            &[(0, "slow", 1)],
            [&["ballista"], &[]],
        ),
        1,
    );
    // 1–2 damage per shot over the long distance: the slow creature falls within five rounds.
    let events: Vec<Event> = (0..5).flat_map(|_| defend_round(&mut s, &r)).collect();
    assert!(events.iter().any(|e| e.kind
        == EventKind::BattleEnded {
            result: BattleResult::Victory(BattleSide::Attacker)
        }));
    assert_eq!(s.active(), None);
    let mut s = start(
        &r,
        &with_machines(
            &r,
            &[(0, "serf", 1)],
            &[(0, "tank", 1)],
            [&["ballista"], &[]],
        ),
        1,
    );
    s.stacks.as_mut_slice()[1].shots = 0;
    assert!(shots_by(&defend_round(&mut s, &r), 1).is_empty());
}

// ---------------------------------------------------------------- BAT-WAR-003 first aid

// covers: BAT-WAR-003
#[test]
fn the_tent_heals_a_hurt_allied_stack_by_one_to_twenty_five() {
    let r = ruleset();
    let mut amounts = std::collections::BTreeSet::new();
    for seed in 0..300u64 {
        // 0 = tank (attacker, 100 HP each), 1 = attacker's tent, 2 = attacker's ballista.
        let mut s = start(
            &r,
            &with_machines(
                &r,
                &[(0, "tank", 3)],
                &[(0, "tank", 1)],
                [&["tent", "ballista"], &[]],
            ),
            seed,
        );
        s.stacks.as_mut_slice()[0].top_hp = 50;
        s.stacks.as_mut_slice()[2].top_hp = 10;
        let events = defend_round(&mut s, &r);
        let heals = healed(&events);
        assert_eq!(heals.len(), 1);
        let (healer, target, hp) = heals[0];
        // Never the hurt ballista; the top creature only.
        assert_eq!((healer, target), (2, 0));
        assert!((1..=25).contains(&hp));
        assert_eq!(s.stacks()[0].top_hp, 50 + hp);
        assert_eq!(s.stacks()[0].count, 3);
        amounts.insert(hp);
    }
    assert_eq!(amounts.len(), 25, "every amount occurs");
}

// covers: BAT-WAR-003
#[test]
fn the_tent_heals_at_most_the_missing_hit_points_and_waits_without_patients() {
    let r = ruleset();
    let mut s = start(
        &r,
        &with_machines(&r, &[(0, "tank", 1)], &[(0, "tank", 1)], [&["tent"], &[]]),
        3,
    );
    s.stacks.as_mut_slice()[0].top_hp = 99;
    let events = defend_round(&mut s, &r);
    assert_eq!(healed(&events), vec![(1, 0, 1)]);
    assert!(healed(&defend_round(&mut s, &r)).is_empty());
}

// ---------------------------------------------------------------- BAT-WAR-004 ammo cart

// covers: BAT-WAR-004
#[test]
fn the_ammo_cart_keeps_shots_full_until_it_is_destroyed_and_never_acts() {
    let r = ruleset();
    // 0 = archer, 1 = cart (attacker); 2 = brute (defender).
    let mut s = start(
        &r,
        &with_machines(&r, &[(0, "archer", 5)], &[(0, "tank", 5)], [&["cart"], &[]]),
        1,
    );
    let shoot = |s: &mut BattleState| {
        activate(s, 0);
        let at = s.stacks()[2].hex;
        act(
            s,
            &r,
            Command::Shoot {
                target: StackId(2),
                at,
            },
        )
        .unwrap()
    };
    let events = shoot(&mut s);
    assert_eq!(s.stacks()[0].shots, 12);
    assert!(
        !events
            .iter()
            .any(|e| e.kind == EventKind::TurnStarted { stack: StackId(1) })
    );
    let events = defend_round(&mut s, &r);
    assert!(
        !events
            .iter()
            .any(|e| e.kind == EventKind::TurnStarted { stack: StackId(1) })
    );
    // Without the cart, shots are used up.
    s.stacks.as_mut_slice()[1].count = 0;
    s.stacks.as_mut_slice()[1].top_hp = 0;
    shoot(&mut s);
    assert_eq!(s.stacks()[0].shots, 11);
}
