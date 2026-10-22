//! The spell context's threat tables (0x43C040 with its seed 0x43BF20) and the defenses'
//! evaluators (0x4387C0, 0x438910, Shield, Air Shield, Stone Skin, Fire Shield) with
//! Counterstrike and Frenzy (E23 T4c; BAT-AI-016), by hand on the test ruleset.

use h3_battle::{BattleSetup, BattleState, StackId, computer};
use h3_content_battle::BattleRuleset;
use h3_contracts::BattleSide;
use h3_kernel::Rng;
use h3_kernel::arith::X87;

use super::{
    ARCHER, CRUSADER, EFREET, GIANT, GOG, GRIFFIN, IMP, MARKSMAN, ROYAL, RUNNER, SULTAN, hero,
    ruleset, ruleset_as, setup, spell,
};
use crate::spells::tables::Entry;
use crate::spells::{Candidate, SpellContext};

const ATTACKER: BattleSide = BattleSide::Attacker;

/// The attacker's spell context in `state`.
fn context<'a>(state: &'a BattleState, r: &'a BattleRuleset) -> SpellContext<'a> {
    let ctx = computer::context(state, r, ATTACKER);
    SpellContext::new(state, r, (&ctx, ATTACKER), &mut Rng::from_seed(0))
}

/// The attacker's `own` with a hero against `enemies`.
fn battle(own: &[(usize, u32)], enemies: &[(usize, u32)]) -> BattleSetup {
    let mut setup = setup(own, enemies);
    setup.heroes[0] = Some(hero(0));
    setup
}

/// Ten runners, ten archers and four crusaders against four runners, six archers, five gogs and
/// three runners: the enemy runners reach every stack of the side within a turn, the archers
/// shoot them, the gogs reach none.
fn melee_and_shots() -> BattleSetup {
    battle(
        &[(RUNNER, 10), (ARCHER, 10), (CRUSADER, 4)],
        &[(RUNNER, 4), (ARCHER, 6), (GOG, 5), (RUNNER, 3)],
    )
}

/// `enemy`'s strike (or `shot`) by its whole count on `s`, uncapped.
fn strike(state: &BattleState, r: &BattleRuleset, (enemy, s): (u8, u8), shot: bool) -> u64 {
    let count = u64::from(state.stacks()[usize::from(enemy)].count);
    computer::uncapped_damage(state, r, (StackId(enemy), StackId(s)), count, shot, 0)
}

// covers: BAT-AI-016
#[test]
fn the_tables_hold_the_strikes_each_stack_meets_this_turn() {
    for slip in [true, false] {
        let r = if slip {
            ruleset()
        } else {
            ruleset_as("t:fixed")
        };
        let state = BattleState::new(&r, &melee_and_shots(), 1).unwrap();
        let (gogs, enemy_runners, more_runners, enemy_archers) = (5, 3, 6, 4);
        // The runners' threat record: the gogs within a turn (the higher value at one turn).
        let ctx = computer::context(&state, &r, ATTACKER);
        let own = computer::threat(&state, &r, &ctx, StackId(0)).unwrap();
        assert_eq!((own.enemy, own.turns), (StackId(gogs), 1));
        // The enemy runners reach all three, the gogs none: their one-turn masks.
        let enemy_ctx = computer::context(&state, &r, BattleSide::Defender);
        let mask = |i| {
            computer::threat(&state, &r, &enemy_ctx, StackId(i))
                .unwrap()
                .mask
        };
        assert_eq!((mask(enemy_runners), mask(gogs)), (0b111, 0));
        let sc = context(&state, &r);
        let t = sc.view().tables();
        let seed = strike(&state, &r, (gogs, 0), false);
        assert_eq!(seed, 11);
        for s in 0..3u8 {
            let melee = [enemy_runners, more_runners].map(|e| strike(&state, &r, (e, s), false));
            // The runners' own seed; with the slip every stack's (the archers, which shoot,
            // and the crusaders, three turns from the gogs, have none of their own).
            let seeded = slip || s == 0;
            let mut want = Entry::default();
            if seeded {
                want = Entry {
                    stack: Some(StackId(gogs)),
                    damage: seed,
                    count: 1,
                    sum: seed,
                };
            }
            for (e, d) in [enemy_runners, more_runners].into_iter().zip(melee) {
                want.count += 1;
                want.sum += d;
                if d > want.damage {
                    (want.stack, want.damage) = (Some(StackId(e)), d);
                }
            }
            assert_eq!(t.melee[usize::from(s)], want, "{slip} {s}");
            let shot = strike(&state, &r, (enemy_archers, s), true);
            let shots = Entry {
                stack: Some(StackId(enemy_archers)),
                damage: shot,
                count: 1,
                sum: shot,
            };
            assert_eq!(t.shots[usize::from(s)], shots);
            assert_eq!(t.worse[usize::from(s)], want);
        }
        assert_eq!(t.threatens, 1 << 3 | 1 << 4 | 1 << 6);
        assert_eq!(t.melee_threatens, 1 << 3 | 1 << 6);
    }
    // The seeded enemy is passed over even after a stronger strike took the entry (the fill reads
    // the seed once, 0x43C0EE): the runners' record takes the four runners, worth more to strike
    // than the forty, which come first and strike harder.
    let r = ruleset();
    let state = BattleState::new(
        &r,
        &battle(&[(RUNNER, 10)], &[(RUNNER, 40), (RUNNER, 4)]),
        1,
    )
    .unwrap();
    let ctx = computer::context(&state, &r, ATTACKER);
    let own = computer::threat(&state, &r, &ctx, StackId(0)).unwrap();
    assert_eq!((own.enemy, own.turns), (StackId(2), 1));
    let (seed, many) = (
        strike(&state, &r, (2, 0), false),
        strike(&state, &r, (1, 0), false),
    );
    assert!(many > seed);
    let t = context(&state, &r).view().tables().clone();
    let want = Entry {
        stack: Some(StackId(1)),
        damage: many,
        count: 2,
        sum: seed + many,
    };
    assert_eq!(t.melee[0], want);
    // Without a melee threat the worse entry is the shots'.
    let r = ruleset_as("t:fixed");
    let state =
        BattleState::new(&r, &battle(&[(ARCHER, 10)], &[(ARCHER, 6), (GOG, 1)]), 1).unwrap();
    let t = context(&state, &r).view().tables().clone();
    assert_eq!(t.melee[0], Entry::default());
    assert_eq!(t.worse[0], t.shots[0]);
    assert_eq!(t.worse[0].stack, Some(StackId(1)));
}

// covers: BAT-AI-016
#[test]
fn the_shields_and_stone_skin_by_the_protection_they_give() {
    // The runners meet 11 (the gogs' seed), 12 and 9 in melee (32) and the archers' 7: Shield
    // keeps 85 % of the melee, trunc(27.2) + 7 = 34 of 39: f = 39 / 34; the strongest (12)
    // falls to 10; trunc((√(39/34) − 1) × 3 000) = 213 (the class 4, six rounds count whole).
    // Air Shield keeps 75 % of the shots: 32 + 5 of 39, the archers' 7 to 6: 80.
    let r = ruleset();
    let state = BattleState::new(&r, &melee_and_shots(), 1).unwrap();
    let sc = context(&state, &r);
    let value = |name, mastery, s| sc.value(s, Candidate::cast(spell(&r, name), mastery, 6));
    assert_eq!(value("shield", 0, StackId(0)), 213);
    assert_eq!(value("airshield", 0, StackId(0)), 80);
    // Stone Skin: the defense bonus's factor against the worse threat, then the same value.
    let worse = sc.view().tables().worse[0].stack.unwrap();
    let base =
        |b| computer::base_damage_on_defense(&state, &r, (worse, StackId(0)), (100, false), b);
    let f = X87::int(base(0)).over(X87::int(base(3))).unwrap().round(53);
    assert_eq!(
        value("stoneskin", 0, StackId(0)),
        sc.view().protection((StackId(0), worse), 6, f)
    );
    assert!(value("stoneskin", 0, StackId(0)) > 0);
    // Without a threat of the kind: nothing (no enemy shoots, the archers meet no melee).
    let r = ruleset_as("t:fixed");
    let state = BattleState::new(&r, &battle(&[(ARCHER, 10)], &[(GOG, 1)]), 1).unwrap();
    let sc = context(&state, &r);
    for name in ["shield", "airshield", "stoneskin"] {
        assert_eq!(
            sc.value(StackId(0), Candidate::cast(spell(&r, name), 0, 6)),
            0
        );
    }
    // Nothing kept (the original's infinite factor): the archer's shot of 1 on the giant, 75 %
    // of it truncated to 0.
    let state = BattleState::new(&r, &battle(&[(GIANT, 1)], &[(ARCHER, 1)]), 1).unwrap();
    let sc = context(&state, &r);
    let shots = sc.view().tables().shots[0];
    assert_eq!((shots.stack, shots.sum), (Some(StackId(1)), 1));
    assert_eq!(
        sc.value(StackId(0), Candidate::cast(spell(&r, "airshield"), 0, 6)),
        0
    );
}

// covers: BAT-AI-016
#[test]
fn a_protection_is_worth_nothing_without_a_threat_it_lowers() {
    let r = ruleset();
    let state = BattleState::new(&r, &melee_and_shots(), 1).unwrap();
    let sc = context(&state, &r);
    let (runners, enemy) = (StackId(0), StackId(3));
    // A factor of 1 lowers nothing.
    assert_eq!(sc.view().protection((runners, enemy), 6, X87::ONE), 0);
    // A strike above the stack's hit points is cut to them: forty runners' strike on one imp (4
    // hit points) divided by itself is 1, so f = 4 / 1; divided by more, 0, and the factor
    // 4 / 0: nothing (the original's infinite factor).
    let few = BattleState::new(&r, &battle(&[(IMP, 1)], &[(RUNNER, 40)]), 1).unwrap();
    let sc = context(&few, &r);
    let d = strike(&few, &r, (1, 0), false);
    assert!(d > computer::hit_points(&few, StackId(0)));
    let (imp, runners) = (StackId(0), StackId(1));
    assert_eq!(sc.view().protection((imp, runners), 6, X87::int(d + 1)), 0);
    // trunc((√4 − 1) × 100) = 100: the imp's strength 100, a one-sided battle (the class 1).
    assert_eq!(sc.view().protection((imp, runners), 6, X87::int(d)), 100);
    // The sums times the class short of a creature: the imps' 1 a strike on a giant (150).
    let weak = BattleState::new(&r, &battle(&[(GIANT, 1)], &[(IMP, 1), (IMP, 1)]), 1).unwrap();
    let sc = context(&weak, &r);
    assert_eq!(
        sc.view()
            .protection((StackId(0), StackId(1)), 6, X87::int(2)),
        0
    );
    // The enemy beaten: a hundred runners kill the one imp before it acts; the runners lose no
    // creature to the damage planned on them.
    let won = BattleState::new(&r, &battle(&[(RUNNER, 100)], &[(IMP, 1)]), 1).unwrap();
    let sc = context(&won, &r);
    assert!(sc.view().beaten());
    assert_eq!(
        sc.view()
            .protection((StackId(0), StackId(1)), 6, X87::int(2)),
        0
    );
}

// covers: BAT-AI-016
#[test]
fn fire_shield_reflects_each_melee_threat_as_the_strongest() {
    // The runners' three melee threats, each reflected as the enemy runners' 12: trunc(12 × 20 /
    // 100) = 2; their strike 30 (the enemy runners' 40 hit points): g = (3 × 2 + 30) / 30, 36:
    // trunc((√1.2 − 1) × 3 000) = 286.
    let r = ruleset();
    let state = BattleState::new(&r, &melee_and_shots(), 1).unwrap();
    let sc = context(&state, &r);
    let fire = |sc: &SpellContext<'_>, mastery, s| {
        sc.value(s, Candidate::cast(spell(&r, "fireshield"), mastery, 6))
    };
    assert_eq!(fire(&sc, 0, StackId(0)), 286);
    // A sultan's own shield takes 20 off: nothing left at Basic, 5 at Advanced.
    let sultans = BattleState::new(&r, &battle(&[(SULTAN, 2)], &[(RUNNER, 40)]), 1).unwrap();
    let sc = context(&sultans, &r);
    assert_eq!(fire(&sc, 0, StackId(0)), 0);
    assert!(fire(&sc, 2, StackId(0)) > 0);
    // Against a fire-immune threat, or none: nothing.
    let efreet = BattleState::new(&r, &battle(&[(RUNNER, 10)], &[(EFREET, 4)]), 1).unwrap();
    assert_eq!(fire(&context(&efreet, &r), 0, StackId(0)), 0);
    let far = BattleState::new(&r, &battle(&[(IMP, 10)], &[(GOG, 4)]), 1).unwrap();
    assert_eq!(fire(&context(&far, &r), 0, StackId(0)), 0);
    // When the predictions beat the enemy: nothing.
    let won = BattleState::new(&r, &battle(&[(RUNNER, 100)], &[(IMP, 1)]), 1).unwrap();
    assert_eq!(fire(&context(&won, &r), 0, StackId(0)), 0);
}

// covers: BAT-AI-016
#[test]
fn counterstrike_values_the_retaliations_the_threats_would_use() {
    // The runners' three melee threats against their one retaliation: one more used at Basic.
    // The enemy runners (12) leave 88 hit points: 9 creatures strike back for 27; base = 30 +
    // 27 = 57, g = 84 / 57, the strike 30 rises to 44, cut to the enemy's 40 (g = 4 / 3):
    // trunc((√(4/3) − 1) × 3 000) = 464.
    let r = ruleset();
    let state = BattleState::new(&r, &melee_and_shots(), 1).unwrap();
    let sc = context(&state, &r);
    let counter =
        |sc: &SpellContext<'_>, s| sc.value(s, Candidate::cast(spell(&r, "counterstrike"), 0, 6));
    assert_eq!(counter(&sc, StackId(0)), 464);
    // The crusaders strike twice: one strike back more in the base.
    assert!(counter(&sc, StackId(2)) > 0);
    // A threat for each retaliation: nothing more to use (griffins retaliate twice).
    let griffins =
        BattleState::new(&r, &battle(&[(GRIFFIN, 5)], &[(RUNNER, 4), (RUNNER, 3)]), 1).unwrap();
    assert_eq!(context(&griffins, &r).view().tables().melee[0].count, 2);
    assert_eq!(counter(&context(&griffins, &r), StackId(0)), 0);
    // Unlimited retaliations: nothing.
    let royal =
        BattleState::new(&r, &battle(&[(ROYAL, 5)], &[(RUNNER, 4), (RUNNER, 3)]), 1).unwrap();
    assert_eq!(counter(&context(&royal, &r), StackId(0)), 0);
    // The strongest threat's strike kills the stack: nothing.
    let killed =
        BattleState::new(&r, &battle(&[(IMP, 1)], &[(RUNNER, 40), (RUNNER, 3)]), 1).unwrap();
    assert_eq!(context(&killed, &r).view().tables().melee[0].count, 2);
    assert_eq!(counter(&context(&killed, &r), StackId(0)), 0);
    // When the predictions beat the enemy: nothing.
    let won = BattleState::new(&r, &battle(&[(RUNNER, 100)], &[(IMP, 1)]), 1).unwrap();
    assert_eq!(counter(&context(&won, &r), StackId(0)), 0);
}

// covers: BAT-AI-016
#[test]
fn frenzy_as_a_second_strike_with_the_slip_or_as_its_rule() {
    // With the slip the runners' strike on the gogs is valued doubled: the gogs die in the
    // exchange and do not strike back, g = 2, trunc((√2 − 1) × 3 000) = 1 242.
    let r = ruleset();
    let state = BattleState::new(&r, &melee_and_shots(), 1).unwrap();
    let frenzy = |sc: &SpellContext<'_>, mastery, s| {
        sc.value(s, Candidate::cast(spell(&r, "frenzy"), mastery, 6))
    };
    assert_eq!(frenzy(&context(&state, &r), 0, StackId(0)), 1242);
    // A shooter that shoots twice counts its second shot halved: still the same strike.
    let marksmen = BattleState::new(&r, &battle(&[(MARKSMAN, 10)], &[(GIANT, 2)]), 1).unwrap();
    assert!(frenzy(&context(&marksmen, &r), 0, StackId(0)) > 0);
    // A stack the exchange kills: nothing.
    let killed = BattleState::new(&r, &battle(&[(IMP, 1)], &[(RUNNER, 40)]), 1).unwrap();
    assert_eq!(frenzy(&context(&killed, &r), 0, StackId(0)), 0);
    // As the rule: the attack of its defense times the value less the defense it loses.
    let fixed = ruleset_as("t:fixed");
    let state = BattleState::new(&fixed, &melee_and_shots(), 1).unwrap();
    let sc = context(&state, &fixed);
    let defense = u16::try_from(computer::defense(&state, &fixed, StackId(0))).unwrap();
    let c = Candidate::cast(spell(&fixed, "frenzy"), 3, 6);
    let e = StackId(5);
    assert_eq!(
        sc.value(StackId(0), c),
        sc.view().attack_bonus((StackId(0), e), 6, defense * 2)
            - sc.view().defense_bonus(StackId(0), 6, defense)
    );
    // Beyond a turn: nothing.
    let far = BattleState::new(&r, &battle(&[(IMP, 10)], &[(GOG, 4)]), 1).unwrap();
    assert_eq!(frenzy(&context(&far, &r), 0, StackId(0)), 0);
}
