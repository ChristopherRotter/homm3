//! Tests of leaving a battle (BAT-FLE-001).

use std::vec;

use h3_contracts::{BattleEffect, BattleResult, BattleSide};

use super::support::*;
use crate::{
    BattleSetup, BattleState, Command, CommandError, EventKind, LegalCommands, legal_commands,
    observe,
};

use BattleSide::{Attacker, Defender};

fn legal(state: &BattleState, r: &h3_content::Ruleset, side: BattleSide) -> vec::Vec<Command> {
    let mut out = LegalCommands::new();
    legal_commands(state, r, side, &mut out);
    out.as_slice().to_vec()
}

// covers: BAT-FLE-001
#[test]
fn a_side_with_a_hero_retreats_on_its_turn() {
    let r = ruleset();
    let s = BattleSetup {
        heroes: [Some(hero(&r)), None],
        ..setup(&r, &[(0, "fixed", 5)], &[(0, "fast", 5)])
    };
    let mut state = BattleState::new(&r, &s, 1).unwrap();
    assert!(state.may_retreat(Attacker) && !state.may_retreat(Defender));
    for viewer in BattleSide::BOTH {
        assert_eq!(observe(&state, viewer).may_retreat, [true, false]);
    }
    // The defender (speed 9) acts first and has no hero; the attacker cannot act out of turn.
    assert_eq!(state.side_to_act(), Some(Defender));
    assert!(!legal(&state, &r, Defender).contains(&Command::Retreat));
    let before = state.clone();
    assert_eq!(
        act(&mut state, &r, Command::Retreat),
        Err(CommandError::CannotRetreat)
    );
    let mut copy = state.clone();
    assert_eq!(
        crate::apply(
            &mut copy,
            &r,
            Attacker,
            Command::Retreat,
            &mut crate::NoopSink
        ),
        Err(CommandError::NotYourTurn)
    );
    assert_eq!((&state, &copy), (&before, &before));
    act(&mut state, &r, Command::Defend).unwrap();
    // On its own turn the attacker may retreat; Retreat comes last among its commands.
    assert_eq!(legal(&state, &r, Attacker).last(), Some(&Command::Retreat));
    let events = act(&mut state, &r, Command::Retreat).unwrap();
    let result = BattleResult::Retreat(Attacker);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, EventKind::BattleEnded { result });
    assert_eq!(state.result(), Some(result));
    assert_eq!(state.winner(), Some(Defender));
    assert_eq!((state.active(), state.side_to_act()), (None, None));
    assert_eq!(state.outcome().unwrap().result, result);
    assert_eq!(
        act(&mut state, &r, Command::Retreat),
        Err(CommandError::BattleOver)
    );
    assert!(Command::Retreat.leaves_battle() && !Command::Wait.leaves_battle());
    // Every result has its own hash code.
    let mut hashes: vec::Vec<u64> = [
        BattleResult::Victory(Attacker),
        BattleResult::Victory(Defender),
        BattleResult::Draw,
        BattleResult::Retreat(Attacker),
        BattleResult::Retreat(Defender),
        BattleResult::Surrender(Attacker),
        BattleResult::Surrender(Defender),
    ]
    .into_iter()
    .map(|result| {
        state.result = Some(result);
        state.state_hash()
    })
    .collect();
    hashes.sort_unstable();
    hashes.dedup();
    assert_eq!(hashes.len(), 7);
}

// covers: BAT-FLE-001
#[test]
fn shackles_of_war_stop_every_retreat_also_against_monsters() {
    let r = ruleset();
    let mut shackled = hero(&r);
    shackled.effects = vec![BattleEffect::NoRetreat];
    // Batch 1b B6: the wearer cannot retreat from a monster.
    let alone = BattleSetup {
        heroes: [Some(shackled.clone()), None],
        ..setup(&r, &[(0, "fast", 5)], &[(0, "fixed", 5)])
    };
    let mut state = BattleState::new(&r, &alone, 1).unwrap();
    assert_eq!(state.side_to_act(), Some(Attacker));
    assert!(!state.may_retreat(Attacker));
    assert!(!legal(&state, &r, Attacker).contains(&Command::Retreat));
    assert_eq!(
        act(&mut state, &r, Command::Retreat),
        Err(CommandError::CannotRetreat)
    );
    // Worn by the enemy's hero, they stop the other hero too.
    let both = BattleSetup {
        heroes: [Some(hero(&r)), Some(shackled)],
        ..setup(&r, &[(0, "fast", 5)], &[(0, "fixed", 5)])
    };
    let state = BattleState::new(&r, &both, 1).unwrap();
    assert_eq!(observe(&state, Attacker).may_retreat, [false, false]);
    let free = BattleSetup {
        heroes: [Some(hero(&r)), Some(hero(&r))],
        ..both.clone()
    };
    let free = BattleState::new(&r, &free, 1).unwrap();
    assert_eq!(observe(&free, Defender).may_retreat, [true, true]);
    assert_ne!(state.state_hash(), free.state_hash());
}

// covers: BAT-FLE-001
#[test]
fn a_side_may_retreat_in_its_tactics_phase() {
    let r = ruleset();
    let mut tactician = hero(&r);
    tactician.tactics = 2;
    let s = BattleSetup {
        heroes: [None, Some(tactician)],
        ..setup(&r, &[(0, "fixed", 5)], &[(0, "fixed", 5)])
    };
    let mut state = BattleState::new(&r, &s, 1).unwrap();
    assert!(state.tactics().is_some());
    assert_eq!(legal(&state, &r, Defender).last(), Some(&Command::Retreat));
    act(&mut state, &r, Command::Retreat).unwrap();
    assert_eq!(state.result(), Some(BattleResult::Retreat(Defender)));
    assert_eq!(state.winner(), Some(Attacker));
    assert_eq!((state.tactics(), state.round()), (None, 0));
}
