//! The spell context's predictions (0x422A40, the side's run 0x422880, the melee prediction
//! 0x4227A0, the predicted strike 0x4224E0; E23 T4b2; BAT-AI-015): the damage the side's stacks
//! that act before the other side's next one plan, against the computer's own estimates, and "the
//! enemy beaten". The Ogre Mages' and the Faerie Dragons' choices with them: the harvested
//! `harvested_computer_casts.rs`.

use std::vec;

use h3_battle::computer::{self, Planned};
use h3_battle::{BattleSetup, BattleState, Command, Field, Hex, StackId};
use h3_content_battle::BattleRuleset;
use h3_contracts::BattleSide;
use h3_kernel::Rng;

use super::{
    ARCHER, CRUSADER, DRAGON, EFREET, GOG, GOLEM, HYDRA, IMP, RUNNER, SULTAN, TENT, active,
    creature, hero, on, play, ruleset, setup,
};
use crate::predict;
use crate::spells::{SpellContext, beaten};

const ATTACKER: BattleSide = BattleSide::Attacker;

/// The damage the attacker's run plans in `state`, its draws from a fixed stream.
fn plan(state: &BattleState, r: &BattleRuleset) -> Planned {
    predict::plan(state, r, ATTACKER, &mut Rng::from_seed(3))
}

/// The computer's damage of `count` creatures of `a` striking `t`, no run-up, capped.
fn hit(state: &BattleState, r: &BattleRuleset, (a, t): (StackId, StackId), count: u64) -> u64 {
    computer::damage_estimate(state, r, (a, t), count, false, 0)
}

/// The creatures `hp` hit points of stack `id` give, rounded up.
fn creatures(state: &BattleState, id: StackId, hp: u64) -> u64 {
    hp.div_ceil(u64::from(state.stacks()[id.index()].max_hp))
}

/// The state after the defender's stacks (the faster) moved to `hexes` in turn and the rest
/// defended until the attacker's first stack is active.
fn met(r: &BattleRuleset, setup: &BattleSetup, hexes: &[u8], first: usize) -> BattleState {
    let mut state = BattleState::new(r, setup, 16).unwrap();
    for &to in hexes {
        play(&mut state, r, Command::Move { to: Hex(to) });
    }
    while active(&state) != first {
        play(&mut state, r, Command::Defend);
    }
    state
}

// covers: BAT-AI-015
#[test]
fn a_walkers_move_and_attack_plans_its_strike_and_the_retaliation() {
    // Ten runners (speed 20) strike the five gogs within their turn: 31 of the gogs' 65 hit
    // points; the 34 left give three gogs to strike back, on Normal, or on Easy for a human side.
    // On Easy a computer side predicts no retaliation.
    let r = ruleset();
    for (difficulty, human, retaliates) in [(1, false, true), (0, false, false), (0, true, true)] {
        let mut setup = on(difficulty, setup(&[(RUNNER, 10)], &[(GOG, 5)]));
        setup.human[0] = human;
        let state = BattleState::new(&r, &setup, 2).unwrap();
        assert_eq!(active(&state), RUNNER);
        let (runners, gogs) = (StackId(0), StackId(1));
        let p = plan(&state, &r);
        let strike = hit(&state, &r, (runners, gogs), 10);
        assert_eq!((strike, p.on(gogs)), (31, 31), "{difficulty} {human}");
        let back = hit(
            &state,
            &r,
            (gogs, runners),
            creatures(&state, gogs, 65 - 31),
        );
        let want = if retaliates { back } else { 0 };
        assert!(back > 0);
        assert_eq!(p.on(runners), want, "{difficulty} {human}");
        assert_eq!(p.left(&state, runners), 100 - want);
    }
}

// covers: BAT-AI-015
#[test]
fn the_run_ends_at_the_other_sides_next_stack() {
    let r = ruleset();
    // The runners and the golems (speed 20) act before the gogs: both strike them, and the gogs,
    // their retaliation never used up in a prediction, strike back at each. The imps (speed 5)
    // are too far away for a move and attack: they plan nothing.
    let state = BattleState::new(
        &r,
        &setup(&[(RUNNER, 10), (GOLEM, 10), (IMP, 5)], &[(GOG, 5)]),
        2,
    )
    .unwrap();
    let (runners, golems, gogs) = (StackId(0), StackId(1), StackId(3));
    let p = plan(&state, &r);
    let first = hit(&state, &r, (runners, gogs), 10);
    let second = hit(&state, &r, (golems, gogs), 10);
    assert_eq!(p.on(gogs), first + second);
    assert!(p.on(gogs) < 65);
    let left = 65 - first;
    assert_eq!(
        p.on(runners),
        hit(&state, &r, (gogs, runners), creatures(&state, gogs, left))
    );
    let left = 65 - first - second;
    assert_eq!(
        p.on(golems),
        hit(&state, &r, (gogs, golems), creatures(&state, gogs, left))
    );
    assert_eq!(p.on(StackId(2)), 0);
    // An efreet of the other side at the same speed acts right after the runners (the order
    // alternates the sides among equal keys): the run ends there, the golems plan nothing.
    let state = BattleState::new(
        &r,
        &setup(&[(RUNNER, 10), (GOLEM, 10)], &[(EFREET, 1), (GOG, 5)]),
        2,
    )
    .unwrap();
    let p = plan(&state, &r);
    let golems = StackId(1);
    assert!(p.on(StackId(2)) + p.on(StackId(3)) > 0);
    assert_eq!(p.on(golems), 0);
}

// covers: BAT-AI-015
#[test]
fn a_shooters_shot_plans_its_damage() {
    // Ten archers (speed 4) act before the gogs (3) and shoot them, by their whole count; the
    // other side's archers, also shooters, at one turn.
    let r = ruleset();
    for defender in [GOG, ARCHER] {
        let state = BattleState::new(&r, &setup(&[(ARCHER, 10)], &[(defender, 5)]), 2).unwrap();
        assert_eq!(active(&state), ARCHER);
        let (archers, target) = (StackId(0), StackId(1));
        let p = plan(&state, &r);
        let shot = computer::damage_estimate(&state, &r, (archers, target), 10, true, 0);
        assert!(shot > 0);
        assert_eq!(p.on(target), shot);
        assert_eq!(p.on(archers), 0);
    }
}

// covers: BAT-AI-015
#[test]
fn a_stack_predicted_dead_is_passed_over_and_no_target() {
    // A hundred runners kill the lone imp; the imp (speed 5) is passed over, and the archers
    // (speed 4) after it have no target left: nothing more is planned. The enemy is beaten: the
    // attack bonus is worth nothing.
    let r = ruleset();
    let state =
        BattleState::new(&r, &setup(&[(RUNNER, 100), (ARCHER, 10)], &[(IMP, 1)]), 2).unwrap();
    let (runners, archers, imp) = (StackId(0), StackId(1), StackId(2));
    let p = plan(&state, &r);
    assert_eq!((p.on(imp), p.left(&state, imp)), (4, 0));
    assert_eq!((p.on(runners), p.on(archers)), (0, 0));
    let ctx = computer::context(&state, &r, ATTACKER);
    let sc = SpellContext::new(&state, &r, (&ctx, ATTACKER), &mut Rng::from_seed(3));
    assert!(sc.beaten());
    assert_eq!(sc.attack_bonus((runners, imp), 6, 6), 0);
    // Without the archers too: the runners' turn alone, and a second imp left alive.
    let state = BattleState::new(&r, &setup(&[(RUNNER, 100)], &[(IMP, 1), (IMP, 100)]), 2).unwrap();
    let ctx = computer::context(&state, &r, ATTACKER);
    let sc = SpellContext::new(&state, &r, (&ctx, ATTACKER), &mut Rng::from_seed(3));
    assert!(!sc.beaten());
}

// covers: BAT-AI-015
#[test]
fn the_enemy_beaten_leaves_out_the_war_machines() {
    let r = ruleset();
    let state = BattleState::new(
        &r,
        &BattleSetup {
            heroes: [None, Some(hero(0))],
            war_machines: [vec![], vec![creature(TENT)]],
            ..setup(&[(RUNNER, 10)], &[(IMP, 3)])
        },
        2,
    )
    .unwrap();
    let imps = StackId(1);
    assert_eq!(state.stacks()[2].creature, creature(TENT));
    let mut p = Planned::default();
    assert!(!beaten(&state, &p, ATTACKER));
    p.add(&state, imps, 11);
    assert_eq!((p.on(imps), p.left(&state, imps)), (11, 1));
    assert!(!beaten(&state, &p, ATTACKER));
    // Planned damage stops at the hit points.
    p.add(&state, imps, 11);
    assert_eq!((p.on(imps), p.left(&state, imps)), (12, 0));
    assert!(beaten(&state, &p, ATTACKER));
    assert!(!beaten(&state, &p, BattleSide::Defender));
}

// covers: BAT-AI-015
#[test]
fn a_creature_that_strikes_twice_strikes_again_while_its_target_lives() {
    // Twenty runners come next to the ten crusaders (speed 6): their strike, the runners'
    // retaliation, then the second strike by the crusaders' hit points left.
    let r = ruleset();
    let s = on(1, setup(&[(CRUSADER, 10)], &[(RUNNER, 20)]));
    let state = met(&r, &s, &[2], CRUSADER);
    let (crusaders, runners) = (StackId(0), StackId(1));
    let p = plan(&state, &r);
    let first = hit(&state, &r, (crusaders, runners), 10);
    assert!(first < 200);
    let back = hit(
        &state,
        &r,
        (runners, crusaders),
        creatures(&state, runners, 200 - first),
    );
    let again = hit(
        &state,
        &r,
        (crusaders, runners),
        creatures(&state, crusaders, 350 - back),
    );
    assert_eq!(p.on(crusaders), back);
    assert_eq!(p.on(runners), (first + again).min(200));
    // A lone runner dies to the first strike: no retaliation, no second strike.
    let s = on(1, setup(&[(CRUSADER, 10)], &[(RUNNER, 1)]));
    let state = met(&r, &s, &[2], CRUSADER);
    let p = plan(&state, &r);
    assert_eq!((p.on(crusaders), p.on(runners)), (0, 10));
}

// covers: BAT-AI-015
#[test]
fn fire_shield_reflects_onto_the_predicted_striker() {
    // Thirty runners strike five sultans with their own Fire Shield (20 %): the reflection of the
    // strike, then the sultans' retaliation, both planned on the runners.
    let r = ruleset();
    let state = BattleState::new(&r, &setup(&[(RUNNER, 30)], &[(SULTAN, 5)]), 2).unwrap();
    assert_eq!(active(&state), RUNNER);
    let (runners, sultans) = (StackId(0), StackId(1));
    let p = plan(&state, &r);
    let strike = hit(&state, &r, (runners, sultans), 30);
    let reflected = computer::fire_shield_estimate(&state, &r, (runners, sultans), strike, 450);
    assert!(reflected > 0);
    let back = hit(
        &state,
        &r,
        (sultans, runners),
        creatures(&state, sultans, 450 - strike),
    );
    assert_eq!(p.on(sultans), strike);
    assert!(reflected + back < 300);
    assert_eq!(p.on(runners), reflected + back);
}

// covers: BAT-AI-015
#[test]
fn a_multi_headed_strike_hits_each_enemy_beside_it_once() {
    // A runner and an efreet beside the hydra (two-hex, at 1 and 2), the efreet below both of its
    // hexes: the hydra's strike hits each once, by its one creature, whichever it attacks, and
    // the one attacked strikes back.
    let r = ruleset();
    let s = on(
        1,
        setup(&[(HYDRA, 1)], &[(RUNNER, 10), (EFREET, 10), (IMP, 10)]),
    );
    let field = Field::new(r.battle().field);
    let around = |h: u8| field.neighbors(Hex(h)).into_iter().flatten();
    let below = around(1)
        .find(|&n| n.0 > 16 && around(2).any(|m| m == n))
        .unwrap();
    let state = met(&r, &s, &[3, below.0], HYDRA);
    let (hydra, runners, efreets) = (StackId(0), StackId(1), StackId(2));
    let p = plan(&state, &r);
    for x in [runners, efreets] {
        assert_eq!(p.on(x), hit(&state, &r, (hydra, x), 1), "{x:?}");
    }
    let back = |x: StackId| {
        let left = 100 - hit(&state, &r, (hydra, x), 1);
        hit(&state, &r, (x, hydra), creatures(&state, x, left))
    };
    assert!([back(runners), back(efreets)].contains(&p.on(hydra)));
    assert_eq!(p.on(StackId(3)), 0);
    // A hundred runners kill the defender's hydra: its retaliation strikes no one.
    let state = BattleState::new(&r, &setup(&[(RUNNER, 100)], &[(HYDRA, 1)]), 2).unwrap();
    let p = plan(&state, &r);
    assert_eq!((p.on(StackId(0)), p.on(StackId(1))), (0, 175));
}

// covers: BAT-AI-015
#[test]
fn the_breath_strikes_the_stack_behind() {
    // Fifty runners next to the dragon (at 1) and a golem behind them: the dragon strikes the
    // runners from where it stands, and its breath kills the golem.
    let r = ruleset();
    let s = on(1, setup(&[(DRAGON, 1)], &[(RUNNER, 50), (GOLEM, 1)]));
    let state = met(&r, &s, &[2, 3], DRAGON);
    let (dragon, runners, golem) = (StackId(0), StackId(1), StackId(2));
    let p = plan(&state, &r);
    assert_eq!(p.on(runners), hit(&state, &r, (dragon, runners), 1));
    assert_eq!(p.on(golem), 10);
}

// covers: BAT-AI-015
#[test]
fn a_losing_attack_is_predicted_after_all_and_far_walks_plan_nothing() {
    // One imp against fifty gogs, its side stronger at range (the archers): its attack loses, so
    // it would fall back; a prediction attacks after all, but the gogs are more than a turn away,
    // so it walks, which plans nothing. Nor do the archers' shots miss the gogs.
    let r = ruleset();
    let state = BattleState::new(
        &r,
        &on(2, setup(&[(IMP, 1), (ARCHER, 30)], &[(GOG, 50)])),
        2,
    )
    .unwrap();
    assert_eq!(active(&state), IMP);
    let p = plan(&state, &r);
    assert_eq!(p.on(StackId(0)), 0);
    let shot = computer::damage_estimate(&state, &r, (StackId(1), StackId(2)), 30, true, 0);
    assert_eq!(p.on(StackId(2)), shot);
}
