//! The speed and spell-protection evaluators (0x439550 with the exchange value 0x435DC0, Haste,
//! Prayer; 0x4396E0 with the Protections and Magic Mirror; E23 T4c; BAT-AI-016) by hand on the
//! test ruleset.

use std::vec;

use h3_battle::{BattleSetup, BattleState, Command, StackId, computer};
use h3_content_battle::BattleRuleset;
use h3_contracts::{BattleEffect, BattleHero, BattleSide};
use h3_kernel::Rng;

use super::{
    ARCHER, EFREET, GOG, IMP, MAGE, PEGASUS, RUNNER, TENT, creature, hero, play, ruleset,
    ruleset_as, setup, spell,
};
use crate::spells::{Candidate, SpellContext};

const ATTACKER: BattleSide = BattleSide::Attacker;

fn context<'a>(state: &'a BattleState, r: &'a BattleRuleset) -> SpellContext<'a> {
    let ctx = computer::context(state, r, ATTACKER);
    SpellContext::new(state, r, (&ctx, ATTACKER), &mut Rng::from_seed(0))
}

/// A hero with the bolt in its book at Basic, `mana` spell points and power 5.
fn mage(r: &BattleRuleset, mana: u16) -> BattleHero {
    let mut h = hero(0);
    h.power = 5;
    h.magic.spell_book = Some(vec![spell(r, "bolt")]);
    h.magic.masteries = vec![(spell(r, "bolt"), 1)];
    h.magic.mana = mana;
    h.magic.max_mana = mana;
    h
}

/// Ten imps, ten runners and ten archers against five gogs, four efreet and six archers whose
/// hero casts the bolt with `mana` (the class 7).
fn mixed(r: &BattleRuleset, mana: u16) -> BattleSetup {
    let mut s = setup(
        &[(IMP, 10), (RUNNER, 10), (ARCHER, 10)],
        &[(GOG, 5), (EFREET, 4), (ARCHER, 6)],
    );
    s.heroes[1] = Some(mage(r, mana));
    s
}

const IMPS: StackId = StackId(0);
const RUNNERS: StackId = StackId(1);
const ARCHERS: StackId = StackId(2);

// covers: BAT-AI-016
#[test]
fn haste_values_the_turns_it_wins_and_overtaking_its_enemy() {
    // The imps (speed 5) reach the gogs at a cost of 13: three turns, two at speed 8 (Haste at
    // Basic, 3): (7 − 2 + 1) × 1 000 / 7 − (7 − 3 + 1) × 1 000 / 7 = 857 − 714 = 143. The runners
    // reach them in one turn either way and do not overtake them (speed 3): nothing.
    let r = ruleset();
    let state = BattleState::new(&r, &mixed(&r, 20), 1).unwrap();
    let sc = context(&state, &r);
    let haste = |sc: &SpellContext<'_>, s| sc.value(s, Candidate::cast(spell(&r, "haste"), 0, 6));
    assert_eq!((haste(&sc, IMPS), haste(&sc, RUNNERS)), (143, 0));
    // A stack without a threat record (the First Aid Tent): nothing; nor with no round left.
    let mut s = mixed(&r, 20);
    s.heroes[0] = Some(hero(0));
    s.war_machines[0] = vec![creature(TENT)];
    let state = BattleState::new(&r, &s, 1).unwrap();
    let sc = context(&state, &r);
    let tent = StackId(3);
    assert!(state.stacks()[tent.index()].has(h3_battle::StackTraits::FIRST_AID));
    assert_eq!(haste(&sc, tent), 0);
    assert_eq!(sc.speed_bonus(IMPS, 3, 0), 0);
    // A stack that has acted has a round less: with one round, none.
    let mut state = BattleState::new(&r, &mixed(&r, 20), 1).unwrap();
    play(&mut state, &r, Command::Defend);
    let sc = context(&state, &r);
    assert!(
        state.stacks()[RUNNERS.index()]
            .flags
            .contains(h3_battle::StackFlags::ACTED)
    );
    assert_eq!(sc.speed_bonus(RUNNERS, 3, 1), 0);
    // Runners that overtake efreet (both 20, 23 hasted) value striking first both ways.
    let state = BattleState::new(&r, &setup(&[(RUNNER, 10)], &[(EFREET, 4)]), 1).unwrap();
    let sc = context(&state, &r);
    let (runners, efreet) = (StackId(0), StackId(1));
    let both = sc
        .exchange_value((runners, efreet))
        .wrapping_add(sc.exchange_value((efreet, runners)));
    assert_eq!(haste(&sc, runners), both.max(0));
    // Two turns where the class is 1: beyond the battle's expected length, nothing.
    let state = BattleState::new(&r, &setup(&[(IMP, 10), (RUNNER, 40)], &[(GOG, 1)]), 1).unwrap();
    let sc = context(&state, &r);
    assert_eq!(haste(&sc, StackId(0)), 0);
    // When the predictions beat the enemy: nothing.
    let won = BattleState::new(&r, &setup(&[(RUNNER, 100)], &[(IMP, 1)]), 1).unwrap();
    assert!(context(&won, &r).beaten());
    assert_eq!(haste(&context(&won, &r), StackId(0)), 0);
}

// covers: BAT-AI-016
#[test]
fn the_exchange_gives_the_second_stack_the_first_ones_shot_flag_with_the_slip() {
    // The runners strike the archers, which strike back: with the slip the archers' answer is
    // valued with the runners' shot flag (none), as the rule with their own.
    let values = |profile| {
        let r = ruleset_as(profile);
        let state = BattleState::new(&r, &setup(&[(RUNNER, 10)], &[(ARCHER, 6)]), 1).unwrap();
        let sc = context(&state, &r);
        (
            sc.exchange_value((StackId(0), StackId(1))),
            sc.exchange_value((StackId(1), StackId(0))),
        )
    };
    let (slip, fixed) = (values("t:faithful"), values("t:fixed"));
    assert_ne!(slip.0, fixed.0);
    // The archers first: both shoot with their own flag, the runners' answer melee.
    assert_eq!(slip.1, fixed.1);
}

// covers: BAT-AI-016
#[test]
fn prayer_sums_its_defense_speed_and_attack() {
    let r = ruleset();
    let state = BattleState::new(&r, &mixed(&r, 20), 1).unwrap();
    let sc = context(&state, &r);
    let c = Candidate::cast(spell(&r, "prayer"), 0, 6);
    for s in [IMPS, RUNNERS, ARCHERS] {
        let attack = if s == IMPS {
            0
        } else {
            let e = computer::threat(&state, &r, &computer::context(&state, &r, ATTACKER), s)
                .unwrap()
                .enemy;
            sc.attack_bonus((s, e), 6, 2)
        };
        assert_eq!(
            sc.value(s, c),
            sc.defense_bonus(s, 6, 2) + sc.speed_bonus(s, 2, 6) + attack
        );
    }
}

// covers: BAT-AI-016
#[test]
fn the_protections_value_the_hero_damage_they_take_off() {
    // The enemy hero's bolt (power 5: 60) on the runners (100 hit points): Protection from Fire
    // at Basic keeps 70 %, so 18 of the 60 come off, 1.8 runners at 300: 540 × 6 / 7 = 462; on
    // the imps (40 hit points) 42 kept of 60 leave nothing of the 40 they have; at Advanced (50
    // %) 10, 2.5 imps: 250 × 6 / 7 = 214. Protection from Air takes nothing off a fire bolt.
    let r = ruleset();
    let state = BattleState::new(&r, &mixed(&r, 20), 1).unwrap();
    let sc = context(&state, &r);
    let value = |sc: &SpellContext<'_>, name, mastery, s| {
        sc.value(s, Candidate::cast(spell(&r, name), mastery, 6))
    };
    assert_eq!(value(&sc, "protfire", 0, RUNNERS), 462);
    assert_eq!(value(&sc, "protfire", 0, IMPS), 0);
    assert_eq!(value(&sc, "protfire", 2, IMPS), 214);
    assert_eq!(value(&sc, "protair", 2, RUNNERS), 0);
    // Magic Mirror at Basic (20 %): with the slip it keeps 2 × (50 − 20) = 60 %: 4 of the imps'
    // 40 come off, one imp: 100 × 6 / 7 = 85; as the rule 80 %: nothing.
    assert_eq!(value(&sc, "mirror", 0, IMPS), 85);
    let fixed = ruleset_as("t:fixed");
    let state = BattleState::new(&fixed, &mixed(&fixed, 20), 1).unwrap();
    let sc = context(&state, &fixed);
    let mirror = sc.value(IMPS, Candidate::cast(spell(&fixed, "mirror"), 0, 6));
    assert_eq!(mirror, 0);
    // Without the enemy hero's mana for the bolt (5), while casting is forbidden or without a
    // hero: nothing.
    for (mana, no_casting, with_hero) in [(4, false, true), (20, true, true), (20, false, false)] {
        let mut s = mixed(&r, mana);
        if no_casting {
            s.heroes[1]
                .as_mut()
                .unwrap()
                .effects
                .push(BattleEffect::NoCasting);
        }
        if !with_hero {
            s.heroes[1] = None;
        }
        let state = BattleState::new(&r, &s, 1).unwrap();
        assert_eq!(value(&context(&state, &r), "protfire", 0, RUNNERS), 0);
    }
    // When the predictions beat the enemy: nothing.
    let mut s = setup(&[(RUNNER, 100)], &[(IMP, 1)]);
    s.heroes[1] = Some(mage(&r, 20));
    let won = BattleState::new(&r, &s, 1).unwrap();
    assert_eq!(value(&context(&won, &r), "protfire", 0, StackId(0)), 0);
}

// covers: BAT-AI-016
#[test]
fn the_protections_estimate_costs_with_the_casters_own_army_with_the_slip() {
    // The caster's own pegasi raise its cost to 7 in the slip's estimate (its own army taken as
    // the enemy's), not as the rule: with 6 mana the bolt is paid only without the slip. Its own
    // mages lower it to 3 either way.
    for (profile, own, mana, paid) in [
        ("t:faithful", PEGASUS, 6, false),
        ("t:fixed", PEGASUS, 6, true),
        ("t:faithful", MAGE, 3, true),
        ("t:fixed", MAGE, 3, true),
    ] {
        let r = ruleset_as(profile);
        let mut s = setup(&[(RUNNER, 10)], &[(GOG, 5), (own, 2)]);
        s.heroes[1] = Some(mage(&r, mana));
        let state = BattleState::new(&r, &s, 1).unwrap();
        let v = context(&state, &r).value(StackId(0), Candidate::cast(spell(&r, "protfire"), 0, 6));
        assert_eq!(v > 0, paid, "{profile} {own}");
    }
}
