//! The computer's retreat test (0x41E570 with 0x41E440 and 0x477A00; E23 T6; BAT-AI-024) on the
//! test ruleset (which prices no surrender: the expected price is 0): its gates in order, the
//! walled-out siege, the predictions with flag 1, the funds, the share of the battle's strength
//! against its threshold; the retreat at the stack's action and the desperate cast at the hero's;
//! with BAT-QUIRK-25 the predictions' planned damage and threat records for the hero's choice
//! (survey slip 26). The price and the walled-out siege themselves: `h3-battle`'s tests.

use std::vec;

use h3_battle::computer::{self, Planned};
use h3_battle::{BattleSetup, BattleState, Command, StackId};
use h3_content_battle::BattleRuleset;
use h3_contracts::{BattleEffect, BattleSide, FortLevel, Moat, RetreatFacts};
use h3_kernel::Rng;
use h3_kernel::arith::{DOUBLE_BITS, X87};

use super::hero::until_side;
use super::{
    ARCHER, BALLISTA, GOG, IMP, RUNNER, TENT, active, at_town, decide, hero, hero_with, on, play,
    ruleset, ruleset_as, setup, spell,
};
use crate::predict::Overwrite;
use crate::retreat::{self, Answer, below, scaled, strengths, threshold};
use crate::spells::SpellContext;

const ATTACKER: BattleSide = BattleSide::Attacker;
const DEFENDER: BattleSide = BattleSide::Defender;

/// A hero worth keeping (5000 experience) with a Tavern town to come back from.
const KEEPER: RetreatFacts = RetreatFacts {
    experience: 5000,
    artifacts: 0,
    tavern: true,
    wanted: false,
};

/// `attacker` led by a hero with `facts` and `funds` against `defender` without one, at
/// `difficulty`.
fn led(
    (attacker, defender): super::Armies<'_>,
    facts: RetreatFacts,
    funds: u32,
    difficulty: u8,
) -> BattleSetup {
    let mut s = on(difficulty, setup(attacker, defender));
    s.heroes[0] = Some(hero(0));
    s.retreat[0] = facts;
    s.funds[0] = funds;
    s
}

/// The test's answer for the side to act, from the stream of `seed`.
fn answer(state: &BattleState, r: &BattleRuleset, seed: u64) -> Answer {
    let side = state.side_to_act().unwrap();
    retreat::test(state, r, side, &mut Rng::from_seed(seed), false)
}

/// One Imp against twenty Gogs on Hard: the Imp moves first, nothing reaches it this round.
fn hopeless(facts: RetreatFacts, funds: u32, difficulty: u8) -> BattleState {
    let s = led((&[(IMP, 1)], &[(GOG, 20)]), facts, funds, difficulty);
    let state = BattleState::new(&ruleset(), &s, 1).unwrap();
    assert_eq!(active(&state), IMP);
    state
}

// covers: BAT-AI-024
#[test]
fn a_hopeless_hero_worth_keeping_retreats() {
    let r = ruleset();
    // The Imp's 100 × 1.2 against the Gogs' 20 × 900 × 1.2 × 1.1: a share of 0.005.
    let state = hopeless(KEEPER, 10_000, 2);
    assert!(answer(&state, &r, 1).retreat);
    // It retreats at the stack's action; the hero has no spell book.
    assert_eq!(decide(&state, &r), Command::Retreat);
    // Artifacts worth 1000 keep a hero without experience too.
    let rich = RetreatFacts {
        experience: 0,
        artifacts: 1000,
        ..KEEPER
    };
    assert!(answer(&hopeless(rich, 10_000, 2), &r, 1).retreat);
    // No: on Easy (no draw), without a Tavern, when the scenario wants the hero, for a hero
    // worth too little (999 and 1999), with less gold than the price and 2500, in Shackles.
    let easy = hopeless(KEEPER, 10_000, 0);
    let mut stream = Rng::from_seed(1);
    assert!(!retreat::test(&easy, &r, ATTACKER, &mut stream, false).retreat);
    assert_eq!(stream, Rng::from_seed(1));
    let poor = RetreatFacts {
        experience: 1999,
        artifacts: 999,
        ..KEEPER
    };
    for facts in [
        RetreatFacts::default(),
        RetreatFacts {
            tavern: false,
            ..KEEPER
        },
        RetreatFacts {
            wanted: true,
            ..KEEPER
        },
        poor,
    ] {
        assert!(
            !answer(&hopeless(facts, 10_000, 2), &r, 1).retreat,
            "{facts:?}"
        );
    }
    assert!(!answer(&hopeless(KEEPER, 2499, 2), &r, 1).retreat);
    assert!(answer(&hopeless(KEEPER, 2500, 2), &r, 1).retreat);
    let mut s = led((&[(IMP, 1)], &[(GOG, 20)]), KEEPER, 10_000, 2);
    s.heroes[0].as_mut().unwrap().effects = vec![BattleEffect::NoRetreat];
    let shackled = BattleState::new(&r, &s, 1).unwrap();
    assert!(!answer(&shackled, &r, 1).retreat);
    // A side without a hero never retreats.
    let state = hopeless(KEEPER, 10_000, 2);
    assert!(!retreat::test(&state, &r, DEFENDER, &mut Rng::from_seed(1), false).retreat);
    // Ten Imps against one: no.
    let s = led((&[(IMP, 10)], &[(IMP, 1)]), KEEPER, 10_000, 2);
    assert!(!answer(&BattleState::new(&r, &s, 1).unwrap(), &r, 1).retreat);
}

// covers: BAT-AI-024
#[test]
fn on_normal_half_of_the_draws_go_on() {
    let r = ruleset();
    let state = hopeless(KEEPER, 10_000, 1);
    let mut both = [false; 2];
    for seed in 0..12 {
        let goes_on = Rng::from_seed(seed).range_inclusive(1, 100) > 50;
        assert_eq!(answer(&state, &r, seed).retreat, goes_on, "{seed}");
        both[usize::from(goes_on)] = true;
    }
    assert_eq!(both, [true, true]);
}

// covers: BAT-AI-024
#[test]
fn the_predictions_leaving_no_creature_retreat_without_gold() {
    let r = ruleset();
    // Ten Runners (speed 20) defend first; in the predictions with flag 1 the Imp's run is
    // followed by theirs, which strike it dead: a retreat, though the funds fall short.
    let s = led((&[(IMP, 1)], &[(RUNNER, 10)]), KEEPER, 0, 2);
    let mut state = BattleState::new(&r, &s, 1).unwrap();
    play(&mut state, &r, Command::Defend);
    assert_eq!(active(&state), IMP);
    assert!(answer(&state, &r, 1).retreat);
    let planned = crate::predict::plan_both(&state, &r, ATTACKER, &mut Rng::from_seed(1), None);
    assert_eq!(planned.left(&state, StackId(0)), 0);
    // The context's predictions plan the Imp's side alone.
    let alone = crate::predict::plan(&state, &r, ATTACKER, &mut Rng::from_seed(1));
    assert!(alone.left(&state, StackId(0)) > 0);
}

// covers: BAT-AI-024
#[test]
fn an_army_walled_out_of_a_siege_retreats() {
    let r = ruleset();
    // Gogs before the closed gate of a town with walls, its Imps inside, nobody to fly, shoot or
    // batter the walls: a retreat before any other term.
    let mut s = at_town(
        &r,
        (&[(GOG, 10)], &[(IMP, 10)]),
        (FortLevel::Fort, Moat::Absent, 0),
    );
    s.heroes[0] = Some(hero(0));
    s.retreat[0] = RetreatFacts {
        experience: 0,
        ..KEEPER
    };
    s.difficulty = 2;
    let mut state = BattleState::new(&r, &s, 1).unwrap();
    until_side(&mut state, &r, ATTACKER);
    assert!(computer::walled_out(&state, &r, ATTACKER));
    assert!(answer(&state, &r, 1).retreat);
    // With archers among them it is not walled out, and the hero is worth too little.
    let mut s = at_town(
        &r,
        (&[(GOG, 10), (ARCHER, 1)], &[(IMP, 10)]),
        (FortLevel::Fort, Moat::Absent, 0),
    );
    s.heroes[0] = Some(hero(0));
    s.retreat[0] = RetreatFacts {
        experience: 0,
        ..KEEPER
    };
    s.difficulty = 2;
    let mut state = BattleState::new(&r, &s, 1).unwrap();
    until_side(&mut state, &r, ATTACKER);
    assert!(!answer(&state, &r, 1).retreat);
}

// covers: BAT-AI-024
#[test]
fn the_strengths_weigh_the_stacks_yet_to_act_and_the_town() {
    let r = ruleset();
    // Ten Gogs (900) and ten Imps (100), none acted: 10 800 and 1200; the other side's × 1.1.
    let s = on(2, setup(&[(GOG, 10)], &[(IMP, 10)]));
    let mut state = BattleState::new(&r, &s, 1).unwrap();
    assert_eq!(strengths(&state, &r, ATTACKER), [10_800, 1320]);
    assert_eq!(strengths(&state, &r, DEFENDER), [11_880, 1200]);
    // The Imps acted: 1000, × 1.1.
    play(&mut state, &r, Command::Defend);
    assert_eq!(strengths(&state, &r, ATTACKER), [10_800, 1100]);
    // At a town the defender's sum is 1.1 times, its enemy's again 1.1 times.
    let s = at_town(
        &r,
        (&[(GOG, 10)], &[(IMP, 10)]),
        (FortLevel::Fort, Moat::Absent, 0),
    );
    let state = BattleState::new(&r, &s, 1).unwrap();
    assert_eq!(strengths(&state, &r, ATTACKER), [10_800, 1452]);
    assert_eq!(strengths(&state, &r, DEFENDER), [11_880, 1320]);
    // `_ftol` of the products: 1.2 times 5 is 6, of −5 −6.
    let more = X87::ratio(12, 10, DOUBLE_BITS).unwrap();
    assert_eq!((scaled(5, more), scaled(-5, more)), (6, -6));
}

// covers: BAT-AI-024
#[test]
fn the_threshold_steps_through_f32() {
    use BattleSide::{Attacker, Defender};
    let f = |bits| X87::from_f32_bits(bits);
    // The expected `f32`s, each step rounded as the original stores it.
    for (difficulty, side, artifacts, experience, bits) in [
        (1, Attacker, 0, 0, 0x3D61_47AD),
        (1, Defender, 0, 0, 0x3DEB_851E),
        (1, Attacker, 1, 0, 0x3DC2_8F5A),
        (2, Attacker, 5001, 0, 0x3DF5_C28E),
        (3, Attacker, 10_001, 0, 0x3E14_7AE1),
        (1, Attacker, 0, 200_000, 0x3DAE_147A),
        (0, Attacker, 0, 0, 0x3D23_D709),
        // At most 0.16.
        (4, Defender, 1, 0, 0x3E23_D70A),
        (4, Defender, 0, 200_000, 0x3E23_D70A),
    ] {
        let x = threshold(difficulty, side, artifacts, experience);
        assert_eq!(x, f(bits), "{difficulty} {side:?} {artifacts} {experience}");
    }
}

// covers: BAT-AI-024
#[test]
fn the_share_is_an_f32_quotient() {
    let x = X87::from_f32_bits(0x3E23_D70A);
    assert!(below(120, 23_760, x));
    assert!(!below(1, 1, x));
    // 16 of 100 lies on 0.16f's other side: 0.16f is a hair below 0.16.
    assert!(!below(16, 84, x));
    assert!(below(15, 85, x));
    // 0 / 0 is unordered, 0 / t is 0, a negative quotient (32-bit sums wrapped) lies below, two
    // negatives give a positive one; `own / 0` is an infinity of `own`'s sign.
    assert!(below(0, 0, x) && below(0, 5, x) && below(5, -10, x));
    assert!(!below(-5, 3, x));
    assert!(below(-5, 5, x) && !below(5, -5, x));
}

// covers: BAT-AI-024, BAT-AI-017
#[test]
fn the_hero_casts_its_damage_spell_in_retreat_and_else_the_side_retreats() {
    let r = ruleset();
    let mut s = led((&[(IMP, 1)], &[(GOG, 20)]), KEEPER, 10_000, 2);
    s.heroes[0] = Some(hero_with(&r, &["bolt"], 1, 50));
    let state = BattleState::new(&r, &s, 1).unwrap();
    assert!(matches!(
        decide(&state, &r),
        Command::Cast { spell: s, .. } if s == spell(&r, "bolt")
    ));
    // A hero without a damage spell casts nothing in retreat: the side retreats.
    let mut s = led((&[(IMP, 1)], &[(GOG, 20)]), KEEPER, 10_000, 2);
    s.heroes[0] = Some(hero_with(&r, &["dispel"], 1, 50));
    let state = BattleState::new(&r, &s, 1).unwrap();
    assert_eq!(decide(&state, &r), Command::Retreat);
}

// covers: BAT-AI-024
#[test]
fn the_ballista_retreats_the_tent_heals() {
    let r = ruleset();
    // Fifty Imps with the hero's Ballista and Tent against a Runner and two hundred Gogs: a share
    // of 0.03. The Runner wounds the Imps, so that the commanded Tent has a patient and takes its
    // turns (BAT-WAR-007); the Ballista's turn asks the retreat test, the Tent's does not.
    let mut s = led(
        (&[(IMP, 50)], &[(RUNNER, 1), (GOG, 200)]),
        KEEPER,
        10_000,
        2,
    );
    s.war_machines[0] = vec![super::creature(BALLISTA), super::creature(TENT)];
    let mut state = BattleState::new(&r, &s, 4).unwrap();
    let mut seen = [false; 2];
    let mut legal = h3_battle::LegalCommands::new();
    for _ in 0..12 {
        let Some(side) = state.side_to_act() else {
            break;
        };
        let command = match (side, active(&state)) {
            (DEFENDER, RUNNER) => {
                h3_battle::legal_commands(&state, &r, side, &mut legal);
                legal
                    .iter()
                    .copied()
                    .find(|c| matches!(c, Command::Attack { target, .. } if *target == StackId(0)))
                    .unwrap_or(Command::Defend)
            }
            (_, BALLISTA) => {
                assert_eq!(decide(&state, &r), Command::Retreat);
                seen[0] = true;
                Command::Defend
            }
            (_, TENT) => {
                let heal = decide(&state, &r);
                assert!(matches!(heal, Command::Heal { .. }), "{heal:?}");
                seen[1] = true;
                heal
            }
            _ => Command::Defend,
        };
        play(&mut state, &r, command);
    }
    assert_eq!(seen, [true, true]);
}

// covers: BAT-AI-024
#[test]
fn with_the_slip_the_choice_reads_what_the_predictions_left() {
    let r = ruleset_as("t:faithful");
    // The Runners' (stack 1) strike plans the Imp (stack 0) dead; each walker's records of its
    // side clear its own.
    let s = led((&[(IMP, 1)], &[(RUNNER, 10)]), KEEPER, 0, 2);
    let mut state = BattleState::new(&r, &s, 1).unwrap();
    play(&mut state, &r, Command::Defend);
    let (imp, runners) = (StackId(0), StackId(1));
    let ctx = computer::context(&state, &r, ATTACKER);
    let mut sc = SpellContext::new(&state, &r, (&ctx, ATTACKER), &mut Rng::from_seed(1));
    assert!(!sc.at_risk());
    assert_eq!(sc.threat_enemy(runners), Some(imp));
    let answer = retreat::test(&state, &r, ATTACKER, &mut Rng::from_seed(1), true);
    let predicted = answer.predicted.unwrap();
    assert!(matches!(
        predicted.overwrites,
        [
            Some(Overwrite::Walker { walker: w0, .. }),
            Some(Overwrite::Walker { walker: w1, .. })
        ] if w0 == imp && w1 == runners
    ));
    sc.after_retreat(&predicted);
    assert!(sc.at_risk());
    assert_eq!(sc.threat_enemy(runners), None);
    // Teleport's refill brings the context's records back; the planned damage stays.
    sc.refill();
    assert_eq!(sc.threat_enemy(runners), Some(imp));
    assert!(sc.at_risk());
    // Without the slip the test keeps nothing.
    let fixed = ruleset_as("t:fixed");
    let a = retreat::test(&state, &fixed, ATTACKER, &mut Rng::from_seed(1), false);
    assert!(a.retreat && a.predicted.is_none());
}

// covers: BAT-AI-024
#[test]
fn a_predicted_shot_leaves_the_other_sides_records() {
    let r = ruleset_as("t:faithful");
    // The Imp walks, then the defender's Archer shoots: the attacker's records are the shot's, in
    // the Archer's context with the damage planned before it.
    let s = led((&[(IMP, 1)], &[(ARCHER, 5)]), KEEPER, 0, 2);
    let state = BattleState::new(&r, &s, 1).unwrap();
    assert_eq!(active(&state), IMP);
    let answer = retreat::test(&state, &r, ATTACKER, &mut Rng::from_seed(1), true);
    let predicted = answer.predicted.unwrap();
    let Some(Overwrite::Shot { planned }) = predicted.overwrites[0] else {
        panic!("{:?}", predicted.overwrites);
    };
    assert!(predicted.overwrites[1].is_none());
    let ctx = computer::context(&state, &r, ATTACKER);
    let mut sc = SpellContext::new(&state, &r, (&ctx, ATTACKER), &mut Rng::from_seed(1));
    sc.after_retreat(&predicted);
    let shot = computer::context(&state, &r, DEFENDER).predicting(&planned);
    let want = computer::threat(&state, &r, &shot, StackId(0)).map(|t| t.enemy);
    assert_eq!(sc.threat_enemy(StackId(0)), want);
    assert_eq!(want, Some(StackId(1)));
    // A predicting context passes over a stack without hit points left, and its enemies so.
    let mut dead = Planned::default();
    dead.add(&state, StackId(1), u64::MAX);
    let spent = computer::context(&state, &r, ATTACKER).predicting(&dead);
    assert_eq!(computer::threat(&state, &r, &spent, StackId(0)), None);
    assert_eq!(computer::threat(&state, &r, &spent, StackId(1)), None);
}
