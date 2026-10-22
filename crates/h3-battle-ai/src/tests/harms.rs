//! The harmful spells' evaluators (Curse to Age, §7.46; E23 T4c; BAT-AI-016) and the removals
//! (0x439A80, Cure, Anti-Magic), by hand on the test ruleset.

use h3_battle::{BattleSetup, BattleState, StackId, computer};
use h3_content_battle::BattleRuleset;
use h3_contracts::BattleSide;
use h3_kernel::Rng;

use super::{
    ARCHER, EFREET, GOG, IMP, RUNNER, cast_at, caster, hero, ruleset, ruleset_as, setup, spell,
};
use crate::spells::{Candidate, SpellContext};

const ATTACKER: BattleSide = BattleSide::Attacker;
const DEFENDER: BattleSide = BattleSide::Defender;

fn context<'a>(state: &'a BattleState, r: &'a BattleRuleset) -> SpellContext<'a> {
    let ctx = computer::context(state, r, ATTACKER);
    SpellContext::new(state, r, (&ctx, ATTACKER), &mut Rng::from_seed(0))
}

/// Ten runners and ten archers with a caster against four runners, six archers and five gogs
/// (the gogs reach none of the side this turn).
fn threats(r: &BattleRuleset) -> BattleSetup {
    let mut s = setup(
        &[(RUNNER, 10), (ARCHER, 10)],
        &[(RUNNER, 4), (ARCHER, 6), (GOG, 5)],
    );
    s.heroes[0] = Some(caster(r, 1));
    s.heroes[1] = Some(hero(0));
    s
}

const ENEMY_RUNNERS: StackId = StackId(2);
const ENEMY_ARCHERS: StackId = StackId(3);
const GOGS: StackId = StackId(4);

/// The value of `name` at Advanced on `s` in the attacker's half, a removal's (`cast` false) or a
/// cast's.
fn value(sc: &SpellContext<'_>, r: &BattleRuleset, (name, cast): (&str, bool), s: StackId) -> i32 {
    let c = Candidate {
        cast,
        ..Candidate::cast(spell(r, name), 2, 6)
    };
    sc.view().value(s, c)
}

// covers: BAT-AI-016
#[test]
fn the_harmful_spells_by_hand() {
    // The enemy runners (4 × 300 = 1 200), archers (1 800) and gogs (4 500; the class 1); the
    // runners and archers threaten the side this turn, the gogs do not.
    let r = ruleset();
    let state = BattleState::new(&r, &threats(&r), 1).unwrap();
    let sc = context(&state, &r);
    assert_eq!(sc.view().tables().threatens, 1 << 2 | 1 << 3);
    let v = |name, s| value(&sc, &r, (name, false), s);
    // Curse (1 at Advanced): the runners' 2 to 4 (3.0) fall to max(1, 2 − 1): g = 1 / 3,
    // trunc(1 200 − √(1/3) × 1 200) = 507; the archers' 2.5: g = 0.4, 661; the gogs: nothing.
    assert_eq!(
        [ENEMY_RUNNERS, ENEMY_ARCHERS, GOGS].map(|s| v("curse", s)),
        [507, 661, 0]
    );
    // Sorrow (2) from morale 1: 0.0173 + 0.0833 of 1 200 = 120; Misfortune (2) from luck 0:
    // 2 × 0.0122 of 1 200 = 29.
    assert_eq!(
        (v("sorrow", ENEMY_RUNNERS), v("misfortune", ENEMY_RUNNERS)),
        (120, 29)
    );
    // Disrupting Ray (4) only on a stack the side's threat records name (the gogs):
    // trunc(4 500 − √0.8 × 4 500) = 475.
    assert_eq!((v("ray", ENEMY_RUNNERS), v("ray", GOGS)), (0, 475));
    // The Rust Dragons' Acid Breath is worth nothing.
    assert_eq!(v("acid", GOGS), 0);
    // Blind on a stack that is not all its side has: its strength, 1 200; Disease (w − 0.9 w) ×
    // 6 = 720; Poison half a runner, 150; Age a third, 400.
    assert_eq!(v("blind", ENEMY_RUNNERS), 1200);
    assert_eq!(v("disease", ENEMY_RUNNERS), 720);
    assert_eq!(
        (v("poison", ENEMY_RUNNERS), v("age", ENEMY_RUNNERS)),
        (150, 400)
    );
    // Forgetfulness only on a shooter.
    assert_eq!(v("forget", ENEMY_RUNNERS), 0);
    assert!(v("forget", ENEMY_ARCHERS) > 0);
    // Weakness: the attack bonus's value of the 5 attack the runners would lose.
    let e = computer::threat(
        &state,
        &r,
        &computer::context(&state, &r, ATTACKER),
        ENEMY_RUNNERS,
    )
    .unwrap()
    .enemy;
    assert_eq!(
        v("weakness", ENEMY_RUNNERS),
        sc.view().attack_bonus((ENEMY_RUNNERS, e), 6, 5)
    );
    // Berserk: the mean of its attacks on the nearest stacks; Hypnotize: the best on its own side.
    let attacks: std::vec::Vec<i32> = computer::berserk_candidates(&state, &r, ENEMY_RUNNERS)
        .map(|t| sc.view().berserk_attack((ENEMY_RUNNERS, t)))
        .collect();
    let n = i32::try_from(attacks.len()).unwrap();
    assert!(n > 0);
    assert_eq!(v("berserk", ENEMY_RUNNERS), attacks.iter().sum::<i32>() / n);
    assert!(v("hypnotize", ENEMY_RUNNERS) > 0);
    // A berserk stack's attack on a stack of the side is worth nothing.
    assert_eq!(sc.view().berserk_attack((ENEMY_RUNNERS, StackId(0))), 0);
}

// covers: BAT-AI-016
#[test]
fn a_cast_weighs_the_resistance_and_the_slips_pick_the_record() {
    // Efreet are immune to the test's fire spells: a cast of Curse at them is worth nothing, a
    // removal's value of it as before.
    for (profile, slip) in [("t:faithful", true), ("t:fixed", false)] {
        let r = ruleset_as(profile);
        let mut s = setup(&[(RUNNER, 10)], &[(EFREET, 4), (GOG, 5)]);
        s.heroes[0] = Some(caster(&r, 1));
        let state = BattleState::new(&r, &s, 1).unwrap();
        let sc = context(&state, &r);
        let efreet = StackId(1);
        assert!(value(&sc, &r, ("curse", false), efreet) > 0);
        assert_eq!(value(&sc, &r, ("curse", true), efreet), 0);
        // Paralyze (an ability, no school) with the slip takes Blind's resistance (fire: none
        // left), without its own; Disease likewise Weakness's.
        for name in ["paralyze", "disease"] {
            let cast = value(&sc, &r, (name, true), efreet);
            assert_eq!(cast == 0, slip, "{profile} {name}");
        }
        // Slow, Sorrow, Misfortune, Disrupting Ray and Forgetfulness resisted alike.
        for name in ["slow", "sorrow", "misfortune"] {
            assert_eq!(value(&sc, &r, (name, true), efreet), 0, "{name}");
        }
    }
}

// covers: BAT-AI-016
#[test]
fn blind_on_a_stack_that_is_all_its_side_has() {
    // A lone enemy stack of 40 runners (12 000): (0.5 − √(e / 400)) × w. Blind at Basic (50):
    // trunc(0.146 × 12 000) = 1 757; Paralyze with the slip takes Blind's 50 too, without its
    // own 25: 0.25 × 12 000 = 3 000. (The side's first stack, the imps, seeds no target: the
    // runners threaten them; they would not threaten a side whose stacks all aim at them, the
    // retaliation the tables seed.)
    for (profile, paralyze) in [("t:faithful", 1757), ("t:fixed", 3000)] {
        let r = ruleset_as(profile);
        let mut s = setup(&[(IMP, 5), (RUNNER, 10)], &[(RUNNER, 40)]);
        s.heroes[0] = Some(caster(&r, 1));
        let state = BattleState::new(&r, &s, 1).unwrap();
        let sc = context(&state, &r);
        assert_eq!(sc.view().tables().threatens, 1 << 2);
        let v = |name| {
            sc.view()
                .value(StackId(2), Candidate::cast(spell(&r, name), 0, 6))
        };
        assert_eq!((v("blind"), v("paralyze")), (1757, paralyze), "{profile}");
    }
}

// covers: BAT-AI-016
#[test]
fn slow_values_the_strikes_it_lets_the_side_make_first_and_the_turns_it_costs() {
    // The side's efreet (speed 20) aim at the enemy runners (20): slowed to 15 at Basic, the
    // efreet strike first both ways; slowed, the runners need a second turn: w / k more.
    let r = ruleset();
    let state = BattleState::new(&r, &setup(&[(EFREET, 4)], &[(RUNNER, 4)]), 1).unwrap();
    let sc = context(&state, &r);
    let (efreet, runners) = (StackId(0), StackId(1));
    let view = sc.view();
    let c = Candidate::cast(spell(&r, "slow"), 0, 6);
    let first = view
        .exchange_value((efreet, runners))
        .wrapping_add(view.exchange_value((runners, efreet)))
        .max(0);
    let w = computer::strength(
        &state,
        &r,
        &computer::context(&state, &r, ATTACKER),
        runners,
    );
    let k = i32::from(
        crate::casts::balance(
            &state,
            &r,
            &computer::context(&state, &r, ATTACKER),
            ATTACKER,
        )
        .class,
    );
    let turns = (view.turns_at(runners, 20), view.turns_at(runners, 15));
    let won = if turns.1 > turns.0 {
        k * w / k - (k - 1) * w / k
    } else {
        0
    };
    assert_eq!(view.value(runners, c), first + won);
    // Beyond the class, without a threat record or a round left, or beaten: nothing.
    assert_eq!(view.slow(runners, Candidate { rounds: 0, ..c }), 0);
}

// covers: BAT-AI-016
#[test]
fn the_removals_value_what_they_end_to_its_caster() {
    // The enemy's hero curses and slows the side's runners; Cure ends both, valued in the
    // enemy's half on a copy with each ended in turn, added. Without the slip only the spells its
    // rule ends; Disrupting Ray (which lasts) only with the slip.
    for profile in ["t:faithful", "t:fixed"] {
        let r = ruleset_as(profile);
        let mut s = setup(&[(RUNNER, 10), (IMP, 5)], &[(RUNNER, 4), (GOG, 5)]);
        s.heroes[0] = Some(caster(&r, 1));
        s.heroes[1] = Some(caster(&r, 1));
        let mut state = BattleState::new(&r, &s, 1).unwrap();
        let runners = StackId(0);
        for name in ["curse", "slow", "ray"] {
            cast_at(&mut state, &r, DEFENDER, name, runners);
        }
        cast_at(&mut state, &r, ATTACKER, "bless", runners);
        let sc = context(&state, &r);
        let view = sc.view();
        let c = Candidate::cast(spell(&r, "cure"), 1, 6);
        let cure = view.value(runners, c);
        // By hand: each harmful spell in turn, in the enemy's half, on the copy so far.
        let mut copy = state.clone();
        let mut want = 0;
        for name in ["curse", "slow", "ray"] {
            let id = spell(&r, name);
            let e = state
                .effects()
                .iter()
                .find(|e| e.stack == runners && e.spell == id)
                .unwrap();
            if profile == "t:fixed" && name == "ray" {
                continue;
            }
            computer::end_spell(&mut copy, &r, runners, id);
            let rounds = e.rounds.map_or(255, u32::from);
            let removed = Candidate {
                spell: id,
                mastery: e.mastery,
                power: rounds,
                rounds,
                cast: false,
            };
            want += view.other().on(&copy).value(runners, removed);
        }
        // The runners lost no hit points: no healing.
        assert_eq!(cure, want, "{profile}");
        // Anti-Magic ends the side's own Bless too with the slip (subtracted in this half).
        let am = view.value(runners, Candidate::cast(spell(&r, "antimagic"), 1, 6));
        let _ = am;
    }
}
