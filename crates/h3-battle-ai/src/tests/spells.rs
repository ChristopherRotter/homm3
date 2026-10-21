//! The spell context (0x4369C0; E23 T4b1; BAT-AI-014): the attack bonus's value (0x437800) by
//! hand, the threat records of the war machines and of a walker that reaches no enemy. The Ogre
//! Mages' choices: the harvested `harvested_computer_casts.rs`.

use std::vec;

use h3_battle::{BattleSetup, BattleState, Command, StackId, computer};
use h3_contracts::{BattleSide, FortLevel, Moat};
use h3_kernel::Rng;

use super::{BALLISTA, GOG, IMP, TENT, active, at_town, creature, hero, play, ruleset, setup};
use crate::spells::SpellContext;

// covers: BAT-AI-014
#[test]
fn the_attack_bonus_lasts_six_rounds_of_an_even_battle() {
    // Ten imps against ten (class 7). A base of 100 by the imps (attack 2) on the other imps
    // (defense 3) does 97, with 6 attack more 125: g = 125 / 97. Their strike, 14, rises to 18
    // (below the 40 hit points), so the bonus counts: trunc((√g − 1) × 1 000 × 6 / 7) = 115.
    let r = ruleset();
    let mut state = BattleState::new(&r, &setup(&[(IMP, 10)], &[(IMP, 10)]), 1).unwrap();
    let side = BattleSide::Attacker;
    let ctx = computer::context(&state, &r, side);
    let (imps, enemy) = (StackId(0), StackId(1));
    let base = |s: &BattleState, b| computer::base_damage(s, &r, (imps, enemy), (100, false), b);
    assert_eq!((base(&state, 0), base(&state, 6)), (97, 125));
    assert_eq!(
        computer::damage_estimate(&state, &r, (imps, enemy), 10, false, 0),
        14
    );
    assert_eq!(computer::strength(&state, &r, &ctx, imps), 1000);
    let sc = SpellContext::new(&state, &r, (&ctx, side), &mut Rng::from_seed(0));
    assert_eq!(sc.attack_bonus((imps, enemy), 6, 6), 115);
    // Seven rounds count it whole; no bonus, nothing.
    assert_eq!(sc.attack_bonus((imps, enemy), 7, 6), 135);
    assert_eq!(sc.attack_bonus((imps, enemy), 6, 0), 0);
    // Once they have acted (defending: their strength 1 024) a round less: 5 / 7, 98.
    play(&mut state, &r, Command::Defend);
    assert_eq!(computer::strength(&state, &r, &ctx, imps), 1024);
    let sc = SpellContext::new(&state, &r, (&ctx, side), &mut Rng::from_seed(0));
    assert_eq!(sc.attack_bonus((imps, enemy), 6, 6), 98);
}

// covers: BAT-AI-014
#[test]
fn the_threat_records_of_the_war_machines_and_behind_the_walls() {
    let r = ruleset();
    // The Ballista shoots: every enemy at one turn. The First Aid Tent records none.
    let state = BattleState::new(
        &r,
        &BattleSetup {
            heroes: [Some(hero(0)), None],
            war_machines: [vec![creature(BALLISTA), creature(TENT)], vec![]],
            ..setup(&[(IMP, 10)], &[(GOG, 5)])
        },
        1,
    )
    .unwrap();
    let ctx = computer::context(&state, &r, BattleSide::Attacker);
    let of = |handle: usize| {
        let i = state
            .stacks()
            .iter()
            .position(|s| s.creature.index() == handle)
            .unwrap();
        StackId(u8::try_from(i).unwrap())
    };
    let ballista = computer::threat(&state, &r, &ctx, of(BALLISTA)).unwrap();
    assert_eq!((ballista.enemy, ballista.turns), (of(GOG), 1));
    assert_eq!(computer::threat(&state, &r, &ctx, of(TENT)), None);
    // At a town with a fort the imps reach no defender behind the closed gate.
    let state = BattleState::new(
        &r,
        &at_town(
            &r,
            (&[(IMP, 10)], &[(GOG, 5)]),
            (FortLevel::Fort, Moat::Absent, 0),
        ),
        1,
    )
    .unwrap();
    assert_eq!(active(&state), IMP);
    let ctx = computer::context(&state, &r, BattleSide::Attacker);
    assert_eq!(computer::threat(&state, &r, &ctx, StackId(0)), None);
}
