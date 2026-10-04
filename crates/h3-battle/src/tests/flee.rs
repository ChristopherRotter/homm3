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
        assert_eq!(observe(&state, &r, viewer).may_retreat, [true, false]);
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
    assert_eq!(observe(&state, &r, Attacker).may_retreat, [false, false]);
    let free = BattleSetup {
        heroes: [Some(hero(&r)), Some(hero(&r))],
        ..both.clone()
    };
    let free = BattleState::new(&r, &free, 1).unwrap();
    assert_eq!(observe(&free, &r, Defender).may_retreat, [true, true]);
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

/// Batch 1b R4: a hero with 7 Pikemen, a Pixie and a Ballista against a hero with Pikemen; the
/// attacker has `funds`.
fn r4(r: &h3_content::Ruleset, funds: u32) -> BattleState {
    let s = BattleSetup {
        heroes: [Some(hero(r)), Some(hero(r))],
        war_machines: [vec![r.creature_by_key("t:ballista").unwrap()], vec![]],
        funds: [funds, 0],
        ..setup(r, &[(0, "fixed", 7), (1, "flyer", 1)], &[(0, "slow", 10)])
    };
    BattleState::new(r, &s, 1).unwrap()
}

// covers: BAT-FLE-002
#[test]
fn the_surrender_price_is_half_the_cost_of_the_living_stacks_war_machines_included() {
    let r = ruleset();
    let mut state = r4(&r, 1472);
    // The Pixie (speed 7) acts first. ½ × (7 × 60 + 25 + 2500) = 1472.5, rounded down (R4).
    assert_eq!(state.side_to_act(), Some(Attacker));
    assert_eq!(state.surrender_price(&r, Attacker), Some(1472));
    assert!(state.may_surrender(&r, Attacker));
    let own = observe(&state, &r, Attacker).own_hero.unwrap();
    assert_eq!((own.surrender_price, own.funds), (Some(1472), 1472));
    let commands = legal(&state, &r, Attacker);
    assert_eq!(
        commands[commands.len() - 2..],
        [Command::Retreat, Command::Surrender]
    );
    // With the current counts of the living stacks: 6 Pikemen and the Ballista, 1430 (R4).
    let mut later = state.clone();
    later.stacks.as_mut_slice()[0].count = 6;
    later.stacks.as_mut_slice()[1].count = 0;
    later.stacks.as_mut_slice()[1].top_hp = 0;
    assert_eq!(later.surrender_price(&r, Attacker), Some(1430));
    let events = act(&mut state, &r, Command::Surrender).unwrap();
    let result = BattleResult::Surrender(Attacker);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, EventKind::BattleEnded { result });
    assert_eq!(
        (state.result(), state.winner()),
        (Some(result), Some(Defender))
    );
    // The outcome reports what the side pays (BAT-END-003).
    assert_eq!(state.outcome().unwrap().surrender_cost, Some(1472));
    assert!(Command::Surrender.leaves_battle());
    // One gold short: no surrender.
    let mut poor = r4(&r, 1471);
    assert!(!poor.may_surrender(&r, Attacker));
    assert!(!legal(&poor, &r, Attacker).contains(&Command::Surrender));
    let before = poor.clone();
    assert_eq!(
        act(&mut poor, &r, Command::Surrender),
        Err(CommandError::CannotAffordSurrender)
    );
    assert_eq!(poor, before);
}

// covers: BAT-FLE-002
#[test]
fn the_discount_enters_as_a_single_precision_rate_under_q_0005() {
    // Batch 1b R4: 10 Pikemen at 40 % discount cost 179 in the original, 180 exactly.
    for (profile, price) in [("t:faithful", 180), ("t:single", 179)] {
        let r = ruleset_with(profile, |_| {});
        let mut diplomat = hero(&r);
        diplomat.surrender_discount_bp = 4000;
        let s = BattleSetup {
            heroes: [Some(diplomat), Some(hero(&r))],
            ..setup(&r, &[(0, "fixed", 10)], &[(0, "slow", 10)])
        };
        let state = BattleState::new(&r, &s, 1).unwrap();
        assert_eq!(
            state.surrender_price(&r, Attacker),
            Some(price),
            "{profile}"
        );
    }
    // A full discount makes it free.
    let r = ruleset_with("t:single", |_| {});
    let mut free = hero(&r);
    free.surrender_discount_bp = 10_000;
    let s = BattleSetup {
        heroes: [Some(free), Some(hero(&r))],
        ..setup(&r, &[(0, "fixed", 10)], &[(0, "slow", 10)])
    };
    let state = BattleState::new(&r, &s, 1).unwrap();
    assert_eq!(state.surrender_price(&r, Attacker), Some(0));
    assert!(state.may_surrender(&r, Attacker));
}

// covers: BAT-FLE-002
#[test]
fn surrender_needs_an_enemy_hero_no_shackles_and_a_priced_ruleset() {
    let r = ruleset();
    let rich = [u32::MAX, u32::MAX];
    let led = |heroes| BattleSetup {
        heroes,
        funds: rich,
        ..setup(&r, &[(0, "fast", 10)], &[(0, "fixed", 10)])
    };
    let mut shackled = hero(&r);
    shackled.effects = vec![BattleEffect::NoRetreat];
    for heroes in [
        [Some(hero(&r)), None],
        [None, Some(hero(&r))],
        [Some(hero(&r)), Some(shackled)],
    ] {
        let mut state = BattleState::new(&r, &led(heroes.clone()), 1).unwrap();
        assert_eq!(state.surrender_price(&r, Attacker), None, "{heroes:?}");
        // Both sides see that neither may surrender (public, ADR-0013 point 9).
        for viewer in BattleSide::BOTH {
            let o = observe(&state, &r, viewer);
            assert_eq!(o.surrender_possible, [false, false]);
            assert_eq!(o.own_hero.and_then(|h| h.surrender_price), None);
        }
        assert_eq!(
            act(&mut state, &r, Command::Surrender),
            Err(CommandError::CannotSurrender)
        );
    }
    // A ruleset that prices no surrenders.
    let unpriced = ruleset_with("t:faithful", |b| b.surrender = None);
    let state = BattleState::new(&unpriced, &led([Some(hero(&r)), Some(hero(&r))]), 1).unwrap();
    assert_eq!(state.surrender_price(&unpriced, Attacker), None);
    assert!(!state.surrender_possible(&unpriced, Attacker));
    assert_eq!(
        observe(&state, &unpriced, Defender).surrender_possible,
        [false, false]
    );
    assert!(state.surrender_possible(&r, Attacker));
    // In its tactics phase a side may surrender too.
    let mut tactician = hero(&r);
    tactician.tactics = 1;
    let mut state = BattleState::new(&r, &led([Some(tactician), Some(hero(&r))]), 1).unwrap();
    assert!(state.tactics().is_some());
    assert_eq!(
        legal(&state, &r, Attacker).last(),
        Some(&Command::Surrender)
    );
    act(&mut state, &r, Command::Surrender).unwrap();
    assert_eq!(state.result(), Some(BattleResult::Surrender(Attacker)));
}

// covers: BAT-END-003
#[test]
fn the_outcome_reports_every_stack_and_the_heroes() {
    use h3_contracts::{HeroOutcome, StackOrigin, StackRecord};
    let r = ruleset();
    let mut state = r4(&r, 0);
    assert_eq!(state.outcome(), None);
    // Batch 1b R4 later: 6 Pikemen left, the Pixie dead; then the attacker retreats.
    state.stacks.as_mut_slice()[0].count = 6;
    state.stacks.as_mut_slice()[1].count = 0;
    state.stacks.as_mut_slice()[1].top_hp = 0;
    act(&mut state, &r, Command::Retreat).unwrap();
    let o = state.outcome().unwrap();
    let id = |key: &str| r.creature_by_key(key).unwrap();
    let record = |origin, key, start, end| StackRecord {
        origin,
        creature: id(key),
        start,
        end,
    };
    assert_eq!(
        o.stacks,
        [
            vec![
                record(StackOrigin::Army(0), "t:fixed", 7, 6),
                record(StackOrigin::Army(1), "t:flyer", 1, 0),
                record(StackOrigin::WarMachine, "t:ballista", 1, 1),
            ],
            vec![record(StackOrigin::Army(0), "t:slow", 10, 10)],
        ]
    );
    // Survivors and war machines are derived from the records.
    let survivors: vec::Vec<_> = o.survivors[0]
        .stacks
        .iter()
        .map(|s| (s.slot, s.count))
        .collect();
    assert_eq!(survivors, [(0, 6)]);
    assert_eq!(o.war_machines, [vec![id("t:ballista")], vec![]]);
    let hero = HeroOutcome {
        mana: 15,
        spells_cast: vec![],
    };
    assert_eq!(o.heroes, [Some(hero.clone()), Some(hero)]);
    assert_eq!(
        (o.result, o.rounds, o.surrender_cost),
        (BattleResult::Retreat(Attacker), 1, None)
    );
}
