//! The turn order as a query (BAT-TRN-001 to BAT-TRN-003): [`turn_order`] foretells the turns
//! the engine's picks give when every stack takes its turn as it comes.

use std::vec;
use std::vec::Vec;

use h3_content_battle::BattleRuleset;
use h3_contracts::{BattleResult, BattleSetup, BattleSide};

use super::support::*;
use crate::{BattleState, Command, StackId, Status, TacticsPhase, TurnOrder, observe, turn_order};

fn ids(order: &[StackId]) -> Vec<u8> {
    order.iter().map(|id| id.0).collect()
}

/// The order `state` shows its attacker, which every viewer sees alike.
fn told(state: &BattleState, r: &BattleRuleset) -> TurnOrder {
    let order = turn_order(&observe(state, r, BattleSide::Attacker));
    assert_eq!(turn_order(&observe(state, r, BattleSide::Defender)), order);
    order
}

/// Plays `rounds` rounds of defending; at every turn the order told is the rest of the round's
/// turns, and at a round's first turn its next is the next round's.
fn defend_through(state: &mut BattleState, r: &BattleRuleset, rounds: u16) {
    let mut turns: Vec<(u16, u8, TurnOrder)> = Vec::new();
    let end = state.round() + rounds;
    while state.round() < end {
        turns.push((state.round(), state.active().unwrap().0, told(state, r)));
        act(state, r, Command::Defend).unwrap();
    }
    for (i, (round, _, order)) in turns.iter().enumerate() {
        let rest: Vec<u8> = turns[i..]
            .iter()
            .take_while(|(n, _, _)| n == round)
            .map(|(_, id, _)| *id)
            .collect();
        assert_eq!(ids(order.round.as_slice()), rest, "round {round}, turn {i}");
        let next: Vec<u8> = turns
            .iter()
            .filter(|(n, _, _)| *n == round + 1)
            .map(|(_, id, _)| *id)
            .collect();
        if !next.is_empty() {
            assert_eq!(ids(order.next.as_slice()), next, "round {round}, turn {i}");
        }
    }
}

// covers: BAT-TRN-001, BAT-TRN-003
#[test]
fn the_order_foretells_every_turn_when_every_stack_defends() {
    let r = ruleset();
    // Speed ties alternate sides, also across rounds.
    let mut s = battle(
        &r,
        &[(0, "fixed", 1), (1, "fast", 1)],
        &[(0, "fast", 1), (1, "fixed", 1), (2, "slow", 1)],
        1,
    );
    let order = told(&s, &r);
    assert_eq!(ids(order.round.as_slice()), [1, 2, 0, 3, 4]);
    assert_eq!(ids(order.next.as_slice()), [1, 2, 0, 3, 4]);
    defend_through(&mut s, &r, 3);
    // The slow attacker acts last in round 1, so the defender wins the tie in round 2; the
    // army slot breaks a tie within a side.
    let mut t = battle(
        &r,
        &[(0, "fixed", 1), (3, "slow", 1)],
        &[(0, "fixed", 1)],
        1,
    );
    assert_eq!(ids(told(&t, &r).next.as_slice()), [2, 0, 1]);
    defend_through(&mut t, &r, 3);
    let mut u = battle(
        &r,
        &[(5, "fixed", 1), (1, "fixed", 1)],
        &[(0, "slow", 1)],
        1,
    );
    assert_eq!(ids(told(&u, &r).round.as_slice()), [0, 1, 2]);
    defend_through(&mut u, &r, 2);
}

// covers: BAT-TRN-002
#[test]
fn waited_stacks_go_to_the_rounds_end_slowest_first() {
    let r = ruleset();
    // Attacker: 0 = fixed (5), 1 = fast (9). Defender: 2 = slow (2).
    let mut s = battle(&r, &[(0, "fixed", 1), (1, "fast", 1)], &[(0, "slow", 1)], 1);
    assert_eq!(ids(told(&s, &r).round.as_slice()), [1, 0, 2]);
    act(&mut s, &r, Command::Wait).unwrap();
    assert_eq!(ids(told(&s, &r).round.as_slice()), [0, 2, 1]);
    act(&mut s, &r, Command::Wait).unwrap();
    let order = told(&s, &r);
    assert_eq!(ids(order.round.as_slice()), [2, 0, 1]);
    // A wait lasts the round only.
    assert_eq!(ids(order.next.as_slice()), [1, 0, 2]);
    act(&mut s, &r, Command::Defend).unwrap();
    // The wait phase: every stack left is a waited one.
    assert_eq!(ids(told(&s, &r).round.as_slice()), [0, 1]);
    act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(ids(told(&s, &r).round.as_slice()), [1]);
    act(&mut s, &r, Command::Defend).unwrap();
    assert_eq!(s.round(), 2);
    assert_eq!(ids(told(&s, &r).round.as_slice()), [1, 0, 2]);
}

// covers: BAT-MAG-024
#[test]
fn a_stack_that_loses_its_turns_takes_no_place() {
    let r = ruleset();
    let mut s = battle(&r, &[(0, "fixed", 1), (1, "fast", 1)], &[(0, "slow", 1)], 1);
    let fixed = &mut s.stacks.as_mut_slice()[0];
    fixed.spells.status = fixed.spells.status.with(Status::NO_TURNS, true);
    let order = told(&s, &r);
    assert_eq!(ids(order.round.as_slice()), [1, 2]);
    assert_eq!(ids(order.next.as_slice()), [1, 2]);
}

// covers: BAT-TAC-002, BAT-TRN-001
#[test]
fn the_tactics_phase_tells_the_first_round_and_an_ended_battle_none() {
    let r = ruleset();
    let mut h = hero(&r);
    h.tactics = 2;
    let setup = BattleSetup {
        heroes: [Some(h), None],
        ..setup(&r, &[(0, "slow", 5), (1, "fast", 5)], &[(0, "fixed", 5)])
    };
    let mut events = Vec::new();
    let s = BattleState::start(&r, &setup, 1, &mut events).unwrap();
    let mut obs = observe(&s, &r, BattleSide::Attacker);
    assert!(
        obs.tactics
            .is_some_and(|p: TacticsPhase| p.side == BattleSide::Attacker)
    );
    // The placing stack is active, but the first round is still to start.
    assert_eq!(obs.active, Some(StackId(1)));
    let order = turn_order(&obs);
    assert_eq!(ids(order.round.as_slice()), [1, 2, 0]);
    assert_eq!(ids(order.next.as_slice()), [1, 2, 0]);
    obs.result = Some(BattleResult::Victory(BattleSide::Attacker));
    assert_eq!(turn_order(&obs), TurnOrder::default());
    assert_eq!(vec![0u8; 0], ids(TurnOrder::default().round.as_slice()));
}
