//! The harmful spells' evaluators (Curse to Age, §7.46; E23 T4c; BAT-AI-016) and the removals
//! (0x439A80, Cure, Anti-Magic), by hand on the test ruleset.

use h3_battle::{BattleSetup, BattleState, Command, StackFlags, StackId, computer};
use h3_content::{Affects, SpellCategory};
use h3_content_battle::{BattleRuleset, SpellPolarity};
use h3_contracts::{BattleEffect, BattleSide};
use h3_kernel::Rng;
use h3_kernel::arith::X87;

use super::{
    ARCHER, EFREET, GOG, IMP, RUNNER, active, cast_at, caster, hero, play, ruleset, ruleset_as,
    same_map, setup, spell,
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
        // By hand: each harmful spell in turn (the spells' order), in the enemy's half, on the
        // copy so far.
        let mut names = ["curse", "slow", "ray"];
        names.sort_by_key(|n| spell(&r, n).index());
        let mut copy = state.clone();
        let mut want = 0;
        for name in names {
            let id = spell(&r, name);
            let e = state
                .effects()
                .iter()
                .find(|e| e.stack == runners && e.spell == id)
                .unwrap();
            if profile == "t:fixed" && name == "ray" {
                continue;
            }
            computer::end_spell(&mut copy, &r, (runners, id), false);
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
        // Anti-Magic at Basic, level v: with the slip it ends every spell but Poison, the side's
        // own Bless too (subtracted in this half), and adds the spell protection up to v;
        // without, the harmful spells below v that are no ability and last their rounds, and the
        // protection below v.
        let am = Candidate::cast(spell(&r, "antimagic"), 1, 6);
        let level = u8::try_from(r.spell(am.spell).unwrap().effect[1]).unwrap();
        let slip = profile == "t:faithful";
        let mut names = ["bless", "curse", "slow", "ray"];
        names.sort_by_key(|n| spell(&r, n).index());
        let mut copy = state.clone();
        let mut want = 0i32;
        for name in names {
            let id = spell(&r, name);
            let sp = r.spell(id).unwrap();
            let e = state
                .effects()
                .iter()
                .find(|e| e.stack == runners && e.spell == id)
                .unwrap();
            let harmful = sp.polarity == SpellPolarity::Negative;
            let ends = slip
                || (e.rounds.is_some()
                    && harmful
                    && sp.category != SpellCategory::Ability
                    && sp.level < level);
            if !ends {
                continue;
            }
            computer::end_spell(&mut copy, &r, (runners, id), name != "ray");
            let rounds = e.rounds.map_or(255, u32::from);
            let removed = Candidate {
                spell: id,
                mastery: e.mastery,
                power: rounds,
                rounds,
                cast: false,
            };
            if harmful {
                want += view.other().on(&copy).value(runners, removed);
            } else {
                want -= view.on(&copy).value(runners, removed);
            }
        }
        let limit = if slip { level } else { level - 1 };
        want += view.spell_protection(runners, (None, limit, 0), 6);
        assert_eq!(view.value(runners, am), want, "{profile}");
    }
}

// covers: BAT-AI-016, BAT-AI-010
#[test]
fn the_context_leaves_its_last_search_to_the_fallbacks_with_the_slip() {
    // The threat records search last the gogs (the defender's last walker: its archers shoot);
    // with the slip (22) a Berserk or Hypnotize value searches after them and the context keeps
    // that map once; without, the records' map stays.
    for (profile, slips) in [("t:faithful", true), ("t:fixed", false)] {
        let r = ruleset_as(profile);
        let state = BattleState::new(&r, &threats(&r), 1).unwrap();
        assert_eq!(computer::last_searched(&state, &r), Some(GOGS));
        let records = computer::map(&state, &r, GOGS, DEFENDER);
        let sc = context(&state, &r);
        assert!(same_map(&sc.cells().unwrap(), &records), "{profile}");
        // Berserk on the side's runners searches their map over the whole field.
        value(&sc, &r, ("berserk", false), StackId(0));
        let berserk = computer::berserk_search(&state, &r, StackId(0)).1.unwrap();
        let want = if slips { &berserk } else { &records };
        assert!(same_map(&sc.cells().unwrap(), want), "{profile}");
        assert!(same_map(&sc.cells().unwrap(), &records), "{profile}");
        // Hypnotize on the enemy runners: their map for the side within the turns (Advanced: 2,
        // at most the class) times their speed.
        value(&sc, &r, ("hypnotize", true), ENEMY_RUNNERS);
        let ctx = computer::context(&state, &r, ATTACKER);
        let class = crate::casts::balance(&state, &r, &ctx, ATTACKER).class;
        let speed = computer::unslowed_speed(&state, ENEMY_RUNNERS);
        let range = speed * u16::from(class.min(2));
        let hypnotize = computer::map_within(&state, &r, (ENEMY_RUNNERS, ATTACKER), range);
        assert!(!same_map(&hypnotize, &records));
        let want = if slips { &hypnotize } else { &records };
        assert!(same_map(&sc.cells().unwrap(), want), "{profile}");
    }
    // Without a defender's walker the attacker's last; with no walker at all, none.
    let r = ruleset();
    let state =
        BattleState::new(&r, &setup(&[(RUNNER, 1), (ARCHER, 1)], &[(ARCHER, 1)]), 1).unwrap();
    assert_eq!(computer::last_searched(&state, &r), Some(StackId(0)));
    let state = BattleState::new(&r, &setup(&[(ARCHER, 1)], &[(ARCHER, 1)]), 1).unwrap();
    assert_eq!(computer::last_searched(&state, &r), None);
    assert!(context(&state, &r).cells().is_none());
}

/// The value of the candidate `c` of `name` at `mastery` on `s` in the attacker's half.
fn at(sc: &SpellContext<'_>, r: &BattleRuleset, (name, mastery): (&str, u8), s: StackId) -> i32 {
    sc.view()
        .value(s, Candidate::cast(spell(r, name), mastery, 6))
}

// covers: BAT-AI-016
#[test]
fn the_harmful_spells_value_nothing_where_their_tests_fail() {
    let r = ruleset();
    let state = BattleState::new(&r, &threats(&r), 1).unwrap();
    let sc = context(&state, &r);
    // The gogs reach none of the side: nothing for the spells that ask it.
    for name in ["weakness", "sorrow", "misfortune", "blind", "disease"] {
        assert_eq!(value(&sc, &r, (name, true), GOGS), 0, "{name}");
    }
    // Slow on the archers, which shoot (one turn, no turns lost), and on the gogs, several turns
    // from the side (no stack to strike first).
    assert!(value(&sc, &r, ("slow", true), ENEMY_ARCHERS) >= 0);
    assert!(value(&sc, &r, ("slow", true), GOGS) >= 0);
    // A morale or luck already at −3 falls no further.
    let mut s = threats(&r);
    let mut low = hero(0);
    for e in [
        BattleEffect::Morale {
            amount: -6,
            affects: Affects::Own,
        },
        BattleEffect::Luck {
            amount: -6,
            affects: Affects::Own,
        },
    ] {
        low.effects.push(e);
    }
    s.heroes[1] = Some(low);
    let state = BattleState::new(&r, &s, 1).unwrap();
    let sc = context(&state, &r);
    assert_eq!(state.stacks()[ENEMY_RUNNERS.index()].morale, -3);
    for name in ["sorrow", "misfortune"] {
        assert_eq!(value(&sc, &r, (name, true), ENEMY_RUNNERS), 0, "{name}");
    }
    // Berserk archers shoot at anyone, so they threaten the side, but have no threat record:
    // no Weakness, no Slow; their Berserk value searches nothing.
    let mut state = BattleState::new(&r, &threats(&r), 1).unwrap();
    cast_at(&mut state, &r, ATTACKER, "berserk", ENEMY_ARCHERS);
    let sc = context(&state, &r);
    assert_eq!(value(&sc, &r, ("weakness", true), ENEMY_ARCHERS), 0);
    assert_eq!(value(&sc, &r, ("slow", true), ENEMY_ARCHERS), 0);
    assert!(
        sc.view()
            .berserk(ENEMY_ARCHERS, Candidate::cast(spell(&r, "berserk"), 2, 6))
            >= 0
    );
    // The speed before Slow counts what Haste adds (3 at Basic).
    cast_at(&mut state, &r, ATTACKER, "haste", StackId(0));
    assert_eq!(computer::unslowed_speed(&state, StackId(0)), 23);
    // An enemy that has acted: a round less for Slow; Hypnotize at Basic (one turn) none left.
    let mut state = BattleState::new(&r, &threats(&r), 1).unwrap();
    while !state.stacks()[ENEMY_RUNNERS.index()]
        .flags
        .contains(StackFlags::ACTED)
    {
        let side = state.side_to_act().unwrap();
        h3_battle::apply(
            &mut state,
            &r,
            side,
            Command::Defend,
            &mut h3_battle::NoopSink,
        )
        .unwrap();
    }
    let sc = context(&state, &r);
    let one = Candidate::cast(spell(&r, "hypnotize"), 1, 6);
    assert_eq!(sc.view().value(ENEMY_RUNNERS, one), 0);
    assert!(value(&sc, &r, ("slow", true), ENEMY_RUNNERS) >= 0);
    // The enemy beaten: nothing for the spells that ask it.
    let won = BattleState::new(&r, &setup(&[(RUNNER, 100)], &[(IMP, 1)]), 1).unwrap();
    let sc = context(&won, &r);
    assert!(sc.view().beaten());
    for name in ["berserk", "hypnotize", "poison", "age"] {
        assert_eq!(at(&sc, &r, (name, 2), StackId(1)), 0, "{name}");
    }
    // A strength's loss above 1: negative (√4 × 100 − 100 lost: −100).
    assert_eq!(crate::spells::View::loss(X87::int(100), X87::int(4)), -100);
    // The other half's other half is the side's own.
    let sc = context(&won, &r);
    let back = sc.view().other().other();
    assert_eq!(
        back.value(StackId(1), Candidate::cast(spell(&r, "age"), 2, 6)),
        0
    );
}

// covers: BAT-AI-016
#[test]
fn cure_heals_the_lost_hit_points_unless_the_battle_is_won_without_them() {
    // The enemy's hero curses and bolts the side's gogs: their top creature lost hit points.
    // Cure heals x = min(lost, power × p + v) and adds its worth to the removal of the Curse:
    // trunc(x × worth / h + removal), x87.
    for profile in ["t:faithful", "t:fixed"] {
        let r = ruleset_as(profile);
        let mut s = setup(&[(GOG, 10), (IMP, 5)], &[(RUNNER, 4), (GOG, 5)]);
        s.heroes[0] = Some(caster(&r, 1));
        s.heroes[1] = Some(caster(&r, 1));
        let mut state = BattleState::new(&r, &s, 1).unwrap();
        let gogs = StackId(0);
        cast_at(&mut state, &r, DEFENDER, "curse", gogs);
        cast_at(&mut state, &r, DEFENDER, "bolt", gogs);
        let stack = state.stacks()[0];
        let lost = u64::from(stack.max_hp - stack.top_hp);
        assert!(lost > 0, "{profile}");
        let sc = context(&state, &r);
        let view = sc.view();
        let c = Candidate::cast(spell(&r, "cure"), 1, 6);
        let cure = r.spell(c.spell).unwrap();
        let x = lost.min(u64::from(cure.power) * 6 + u64::from(cure.effect[1]));
        let shot = computer::shoots(&state, &r, gogs);
        let ctx = computer::context(&state, &r, ATTACKER);
        let worth = computer::worth(&state, &r, &ctx, gogs, shot);
        let healed = X87::int(x)
            .times(worth)
            .over(X87::int(u64::from(stack.max_hp)))
            .unwrap();
        // The Curse's removal by hand, in the enemy's half on a copy without it.
        let curse = spell(&r, "curse");
        let e = state
            .effects()
            .iter()
            .find(|e| e.stack == gogs && e.spell == curse)
            .unwrap();
        let mut copy = state.clone();
        computer::end_spell(&mut copy, &r, (gogs, curse), false);
        let rounds = e.rounds.map_or(255, u32::from);
        let removed = Candidate {
            spell: curse,
            mastery: e.mastery,
            power: rounds,
            rounds,
            cast: false,
        };
        let removal = view.other().on(&copy).value(gogs, removed);
        assert!(removal >= 0, "{profile}");
        let want = computer::ftol(healed.plus(X87::int(u64::try_from(removal).unwrap())));
        assert_eq!(view.value(gogs, c), want, "{profile}");
    }
    // The enemy beaten and the runners' loss short of a creature: the removal alone (none here).
    // The archer shoots at the runners on its turn.
    let r = ruleset();
    let mut state = BattleState::new(&r, &setup(&[(RUNNER, 100)], &[(ARCHER, 1)]), 1).unwrap();
    play(&mut state, &r, Command::Defend);
    let shot = super::decide(&state, &r);
    play(&mut state, &r, shot);
    while active(&state) != RUNNER {
        play(&mut state, &r, Command::Defend);
    }
    let stack = state.stacks()[0];
    assert!(stack.top_hp < stack.max_hp);
    let sc = context(&state, &r);
    assert!(sc.view().beaten());
    assert_eq!(at(&sc, &r, ("cure", 1), StackId(0)), 0);
}
