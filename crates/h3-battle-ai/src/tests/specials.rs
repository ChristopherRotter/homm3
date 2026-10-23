//! The computer hero's special spells (0x43ACA0, 0x43B1E0 with 0x43AF50, 0x43B8F0; E23 T5b1;
//! BAT-AI-019) on the test ruleset: the raises' creatures, the three-quarter rule and a raise
//! that lasts only the battle, Sacrifice's victims with survey slip 28, Earthquake's walkers and
//! shooters.

use std::vec;
use std::vec::Vec;

use h3_battle::computer;
use h3_battle::{BattleState, CastTarget, Command, NoopSink, StackId, apply, validate};
use h3_content_battle::BattleRuleset;
use h3_contracts::{BattleHero, BattleSide, FortLevel, Moat};
use h3_kernel::Rng;
use h3_kernel::arith::X87;

use super::hero::{cast, led, until_side, valued};
use super::{
    ARCHER, Armies, GOG, HYDRA, IMP, RUNNER, active, at_town, cast_at, caster, hero_with, play,
    ruleset, ruleset_as, setup, spell,
};
use crate::hero::{self, Hero, Valued};
use crate::spells::{Candidate, SpellContext};

const ATTACKER: BattleSide = BattleSide::Attacker;
const DEFENDER: BattleSide = BattleSide::Defender;

/// A hero with `names` at `mastery`, 999 mana and spell power `power`.
fn with_power(r: &BattleRuleset, names: &[&str], mastery: u8, power: u8) -> BattleHero {
    let mut h = hero_with(r, names, mastery, 999);
    h.power = power;
    h
}

/// `own` led by `hero` against `enemies`, whose hero bolts the attacker's stack `bolted` at its
/// first chance (60 damage); then every stack defends until a stack of the attacker acts.
fn bolted(
    r: &BattleRuleset,
    (own, enemies): Armies<'_>,
    hero: BattleHero,
    bolted: u8,
) -> BattleState {
    let mut s = led(own, enemies, hero);
    s.heroes[1] = Some(caster(r, 1));
    let mut state = BattleState::new(r, &s, 1).unwrap();
    cast_at(&mut state, r, DEFENDER, "bolt", StackId(bolted));
    until_side(&mut state, r, ATTACKER);
    state
}

/// The spell context of `side` in `state`, from a fixed stream.
fn context<'a>(state: &'a BattleState, r: &'a BattleRuleset, side: BattleSide) -> SpellContext<'a> {
    let ctx = computer::context(state, r, side);
    SpellContext::new(state, r, (&ctx, side), &mut Rng::from_seed(5))
}

/// `trunc(k × w)` for stack `t`'s worth with the shot flag of `shot_of`, doubled as `sc` says.
fn raised(
    (state, r): (&BattleState, &BattleRuleset),
    sc: &SpellContext<'_>,
    (t, shot_of): (StackId, StackId),
    k: u64,
) -> i32 {
    let ctx = computer::context(state, r, ATTACKER);
    let shot = computer::shoots(state, r, shot_of);
    let v = computer::ftol(computer::worth(state, r, &ctx, t, shot).times(X87::int(k)));
    if sc.raises_double() { 2 * v } else { v }
}

/// The value of `side`'s hero's cast of `name` in `state` (0x43BB20).
fn valued_for(state: &BattleState, r: &BattleRuleset, name: &str, side: BattleSide) -> Valued {
    let ctx = computer::context(state, r, side);
    let sc = context(state, r, side);
    let h = Hero::new(&sc, (state, r), (state.active().unwrap(), side, ctx));
    let id = spell(r, name);
    let hc = computer::hero_cast(state, r, side, (id, r.spell(id).unwrap()));
    let c = Candidate {
        spell: id,
        mastery: hc.mastery,
        power: u32::from(hc.power),
        rounds: u32::from(hc.rounds),
        cast: true,
    };
    hero::value(&h, (id, r.spell(id).unwrap()), c, &mut Rng::from_seed(5))
}

// covers: BAT-AI-019
#[test]
fn a_raise_is_worth_the_creatures_it_brings_back() {
    // The bolt kills 4 of the 10 gogs (13 hit points each). A raise of 30 hit points (power 0)
    // brings back trunc(30 / 13) = 2 of them, worth trunc(2 × w); the unhurt imps are no target.
    // The imps act first: the gogs can still act, so the raise is not due now.
    let r = ruleset();
    let own = [(GOG, 10), (IMP, 5)];
    let enemies = [(RUNNER, 4), (GOG, 5)];
    let mut state = bolted(&r, (&own, &enemies), with_power(&r, &["raise"], 1, 0), 0);
    let gogs = StackId(0);
    assert_eq!(state.stacks()[0].count, 6);
    assert_eq!(active(&state), IMP);
    let sc = context(&state, &r, ATTACKER);
    assert!(!sc.beaten());
    let want = raised((&state, &r), &sc, (gogs, gogs), 2);
    assert_eq!(
        valued(&state, &r, "raise"),
        Valued {
            value: want,
            target: CastTarget::Stack(gogs),
            second: CastTarget::None,
            now: false,
        }
    );
    assert_eq!(cast(&state, &r), None);
    // At the gogs' own turn it is due, and cast.
    while state.active() != Some(gogs) {
        play(&mut state, &r, Command::Defend);
    }
    assert!(valued(&state, &r, "raise").now);
    let c = cast(&state, &r).unwrap();
    assert_eq!(
        c,
        Command::Cast {
            spell: spell(&r, "raise"),
            target: CastTarget::Stack(gogs),
            second: CastTarget::None,
        }
    );
    assert!(validate(&state, &r, ATTACKER, c).is_ok());
    // A raise of more than the 4 lost (power 5: 280 hit points, 21 gogs) counts only once the
    // losses reach three quarters of the start (7) or the enemy is beaten: nothing here.
    let state = bolted(&r, (&own, &enemies), with_power(&r, &["raise"], 1, 5), 0);
    let v = valued(&state, &r, "raise");
    assert_eq!((v.value, v.target), (0, CastTarget::None));
}

// covers: BAT-AI-019
#[test]
fn with_the_enemy_beaten_a_raise_brings_back_every_lost_creature_unless_it_lasts_the_battle() {
    // The bolt kills 6 of the 30 archers; their shot kills the lone imp: the enemy is beaten, so
    // 28 raised archers count as the 6 lost, due now. A raise that lasts only the battle below
    // Advanced is worth nothing then; at Advanced it counts as the other.
    let r = ruleset();
    let (own, enemies) = ([(ARCHER, 30)], [(IMP, 1)]);
    let archers = StackId(0);
    let state = bolted(
        &r,
        (&own, &enemies),
        hero_with(&r, &["raise", "resurrect"], 1, 999),
        0,
    );
    assert_eq!(state.stacks()[0].count, 24);
    let sc = context(&state, &r, ATTACKER);
    assert!(sc.beaten());
    let want = raised((&state, &r), &sc, (archers, archers), 6);
    assert_eq!(
        valued(&state, &r, "raise"),
        Valued {
            value: want,
            target: CastTarget::Stack(archers),
            second: CastTarget::None,
            now: true,
        }
    );
    assert_eq!(valued(&state, &r, "resurrect").value, 0);
    let state = bolted(
        &r,
        (&own, &enemies),
        hero_with(&r, &["resurrect"], 2, 999),
        0,
    );
    let sc = context(&state, &r, ATTACKER);
    let want = raised((&state, &r), &sc, (archers, archers), 6);
    assert_eq!(valued(&state, &r, "resurrect").value, want);
}

// covers: BAT-AI-019
#[test]
fn sacrifice_weighs_each_victim_against_the_creatures_it_raises() {
    // The bolt kills 6 of the 8 archers (three quarters). Sacrificing the 5 imps raises
    // (3 + 4 + 5) × 5 / 10 = 6 archers, worth trunc(6 × w) less the imps' strength; the hydra
    // would raise 18, counted as the 6 lost, worth less than its own strength: never the victim.
    // With BAT-QUIRK-25 the archers' worth takes the imps' shot flag (survey slip 28): offense
    // halved, as for a shooter that cannot shoot; fixed, their own.
    let mut values: Vec<i32> = vec![];
    for profile in ["t:faithful", "t:fixed"] {
        let r = ruleset_as(profile);
        let state = bolted(
            &r,
            (&[(ARCHER, 8), (IMP, 5), (HYDRA, 1)], &[(GOG, 30)]),
            hero_with(&r, &["sacrifice"], 1, 999),
            0,
        );
        let (archers, imps, hydra) = (StackId(0), StackId(1), StackId(2));
        assert_eq!(state.stacks()[0].count, 2, "{profile}");
        assert_eq!(active(&state), IMP, "{profile}");
        let sc = context(&state, &r, ATTACKER);
        let ctx = computer::context(&state, &r, ATTACKER);
        let strength = |s| computer::strength(&state, &r, &ctx, s);
        let shot_of = |v| if profile == "t:faithful" { v } else { archers };
        let undoubled = |v| {
            let w = computer::worth(&state, &r, &ctx, archers, computer::shoots(&state, &r, v));
            computer::ftol(w.times(X87::int(6))) - strength(v)
        };
        assert!(undoubled(hydra) <= 0, "{profile}");
        let want = raised((&state, &r), &sc, (archers, shot_of(imps)), 6)
            - if sc.raises_double() { 2 } else { 1 } * strength(imps);
        assert!(want > 0, "{profile}");
        let v = valued(&state, &r, "sacrifice");
        assert_eq!(
            v,
            Valued {
                value: want,
                target: CastTarget::Stack(archers),
                second: CastTarget::Stack(imps),
                now: false,
            },
            "{profile}"
        );
        let c = Command::Cast {
            spell: spell(&r, "sacrifice"),
            target: v.target,
            second: v.second,
        };
        assert!(validate(&state, &r, ATTACKER, c).is_ok(), "{profile}");
        values.push(v.value);
    }
    assert!(values[0] < values[1], "{values:?}");
}

/// A battle at the test town's Fort: `own` led by a hero with Earthquake at `mastery` against
/// `enemies`, until an attacker's stack acts.
fn at_fort(r: &BattleRuleset, (own, enemies): Armies<'_>, mastery: u8) -> BattleState {
    let mut s = at_town(r, (own, enemies), (FortLevel::Fort, Moat::Absent, 0));
    s.heroes[0] = Some(hero_with(r, &["quake"], mastery, 999));
    s.heroes[1] = Some(hero_with(r, &["quake"], mastery, 999));
    let mut state = BattleState::new(r, &s, 1).unwrap();
    until_side(&mut state, r, ATTACKER);
    state
}

// covers: BAT-AI-019
#[test]
fn earthquake_values_the_walkers_the_walls_keep_out_and_the_shots_they_halve() {
    // The gogs reach no imp behind the closed walls: no threat enemy, their whole strength while
    // no wall-shot target is down. The archers shoot at an imp inside through the walls:
    // trunc(s × h / 3W), W = 8 × 2 hit points, h = min(W, 2). One-hex shooters: both profiles
    // alike (slip 29 moves only a two-hex shooter's hex).
    for profile in ["t:faithful", "t:fixed"] {
        let r = ruleset_as(profile);
        let state = at_fort(&r, (&[(ARCHER, 10), (GOG, 10)], &[(IMP, 10)]), 1);
        let (archers, gogs) = (StackId(0), StackId(1));
        assert_eq!(state.active(), Some(archers), "{profile}");
        let sc = context(&state, &r, ATTACKER);
        let ctx = computer::context(&state, &r, ATTACKER);
        assert_eq!(sc.threat_enemy(gogs), None, "{profile}");
        let e = sc.threat_enemy(archers).unwrap();
        assert!(computer::walls_halve_from_position(&state, archers, e));
        assert!(computer::walls_halve(&state, archers, e));
        let strength = |s| computer::strength(&state, &r, &ctx, s);
        let want = strength(gogs) + strength(archers) * 2 / 48;
        assert_eq!(
            valued(&state, &r, "quake"),
            Valued {
                value: want,
                target: CastTarget::None,
                second: CastTarget::None,
                now: true,
            },
            "{profile}"
        );
        assert_eq!(
            cast(&state, &r).map(|c| matches!(c, Command::Cast { .. })),
            Some(true)
        );
    }
}

// covers: BAT-AI-019
#[test]
fn earthquake_is_worth_nothing_to_the_defender_with_the_enemy_beaten_or_without_walls() {
    let r = ruleset();
    // The defender's hero.
    let mut state = at_fort(&r, (&[(ARCHER, 10), (GOG, 10)], &[(IMP, 10)]), 1);
    until_side(&mut state, &r, DEFENDER);
    assert_eq!(valued_for(&state, &r, "quake", DEFENDER).value, 0);
    // The archers' shot kills the lone imp.
    let state = at_fort(&r, (&[(ARCHER, 30)], &[(IMP, 1)]), 1);
    assert!(context(&state, &r, ATTACKER).beaten());
    assert_eq!(valued(&state, &r, "quake").value, 0);
    // A town without a fort, the open field.
    let mut s = at_town(
        &r,
        (&[(ARCHER, 10)], &[(IMP, 10)]),
        (FortLevel::None, Moat::Absent, 0),
    );
    s.heroes[0] = Some(hero_with(&r, &["quake"], 1, 999));
    let mut state = BattleState::new(&r, &s, 1).unwrap();
    until_side(&mut state, &r, ATTACKER);
    assert_eq!(valued(&state, &r, "quake").value, 0);
    let mut s = setup(&[(ARCHER, 10)], &[(IMP, 10)]);
    s.heroes[0] = Some(hero_with(&r, &["quake"], 1, 999));
    let mut state = BattleState::new(&r, &s, 1).unwrap();
    until_side(&mut state, &r, ATTACKER);
    assert_eq!(valued(&state, &r, "quake").value, 0);
}

// covers: BAT-AI-019
#[test]
fn once_a_wall_shot_target_is_down_the_walkers_count_nothing() {
    // An Expert Earthquake's 9 hits on 8 targets of 2 hit points bring one down at least. A round
    // later the gogs add nothing; the archers' share follows the walls left.
    let r = ruleset();
    let mut state = at_fort(&r, (&[(ARCHER, 10), (GOG, 10)], &[(IMP, 10)]), 3);
    let quake = Command::Cast {
        spell: spell(&r, "quake"),
        target: CastTarget::None,
        second: CastTarget::None,
    };
    apply(&mut state, &r, ATTACKER, quake, &mut NoopSink).unwrap();
    let round = state.round();
    while state.round() == round {
        play(&mut state, &r, Command::Defend);
    }
    until_side(&mut state, &r, ATTACKER);
    let siege = computer::siege(&state).unwrap();
    let parts = &siege.parts[..usize::from(siege.targets)];
    assert!(parts.contains(&0), "{parts:?}");
    let walls: i32 = parts.iter().map(|&p| i32::from(p)).sum();
    let sc = context(&state, &r, ATTACKER);
    let ctx = computer::context(&state, &r, ATTACKER);
    let archers = StackId(0);
    let share = sc.threat_enemy(archers).map_or(0, |e| {
        if computer::walls_halve_from_position(&state, archers, e) {
            computer::strength(&state, &r, &ctx, archers) * walls.min(9) / (3 * walls)
        } else {
            0
        }
    });
    assert_eq!(valued(&state, &r, "quake").value, share);
}
