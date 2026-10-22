//! A removal's copy of a stack (0x439A80 on 0x437A00's lone record; E23 T4d, BAT-AI-016): with
//! BAT-QUIRK-25 (survey slip 24) the evaluators walk the battle's records beside the copy, its
//! record of the stack with every spell among them and none the copy; in both profiles the copy
//! counts its own turns, and Hypnotize's map marks a shooter's whole side.

use std::vec::Vec;

use h3_battle::{BattleSetup, BattleState, StackId, computer};
use h3_content_battle::BattleRuleset;
use h3_contracts::BattleSide;
use h3_kernel::Rng;

use super::{ARCHER, GOG, RUNNER, cast_at, caster, ruleset_as, setup, spell};
use crate::spells::{Candidate, SpellContext};

const ATTACKER: BattleSide = BattleSide::Attacker;
const DEFENDER: BattleSide = BattleSide::Defender;

const RUNNERS: StackId = StackId(0);
const ENEMY_ARCHERS: StackId = StackId(1);

fn context<'a>(state: &'a BattleState, r: &'a BattleRuleset) -> SpellContext<'a> {
    let ctx = computer::context(state, r, ATTACKER);
    SpellContext::new(state, r, (&ctx, ATTACKER), &mut Rng::from_seed(0))
}

/// Ten runners against four runners, six archers and five gogs, both heroes casting every test
/// spell at Expert.
fn armies(r: &BattleRuleset) -> BattleSetup {
    heroes(
        r,
        setup(&[(RUNNER, 10)], &[(RUNNER, 4), (ARCHER, 6), (GOG, 5)]),
    )
}

/// `s` with heroes casting every test spell at Expert on both sides.
fn heroes(r: &BattleRuleset, mut s: BattleSetup) -> BattleSetup {
    s.heroes[0] = Some(caster(r, 3));
    s.heroes[1] = Some(caster(r, 3));
    s
}

/// The candidate a removal values for the spell `name` lasting on stack `s` (its mastery, its
/// rounds left as power and rounds, no cast).
fn removed(state: &BattleState, r: &BattleRuleset, name: &str, s: StackId) -> Candidate {
    let id = spell(r, name);
    let e = state
        .effects()
        .iter()
        .find(|e| e.stack == s && e.spell == id)
        .unwrap();
    let rounds = e.rounds.map_or(255, u32::from);
    Candidate {
        spell: id,
        mastery: e.mastery,
        power: rounds,
        rounds,
        cast: false,
    }
}

/// `state` with the spell `name` ended on stack `s`, as a removal's copy has it.
fn without(state: &BattleState, r: &BattleRuleset, name: &str, s: StackId) -> BattleState {
    let mut copy = state.clone();
    computer::end_spell(&mut copy, r, (s, spell(r, name)), true);
    copy
}

/// The berserk attack's value of the copy (`copy`'s stack `s`) on the battle's record of it, by
/// hand in `side`'s half (0x43A340: 0 for a record of the half's side; else the exchange, both
/// losses counted).
fn on_itself(
    (r, side): (&BattleRuleset, BattleSide),
    battle: &BattleState,
    copy: &BattleState,
    s: StackId,
) -> i32 {
    if battle.stacks()[s.index()].side == side {
        return 0;
    }
    let shot = computer::shoots(copy, r, s);
    let hp = (
        computer::hit_points(copy, s),
        computer::hit_points(battle, s),
    );
    let (hs, ht) = computer::exchange_with_itself((battle, copy), r, side, (s, hp), shot);
    let ctx = computer::context(battle, r, side);
    let desperate = ctx.desperate();
    let value = |state, lost| computer::damage_value(state, r, &ctx, (s, lost), (shot, desperate));
    value(copy, hp.0 - hs).wrapping_add(value(battle, hp.1 - ht))
}

// covers: BAT-AI-016
#[test]
fn no_record_of_the_battle_is_the_copy() {
    // The enemy rays and slows the side's runners. On the copy the enemy's records still have
    // the runners as their threat enemy only where the copy is the stack itself (without the
    // slip): Disrupting Ray's value (it stays on the copy) and Slow's strikes first need it.
    for (profile, slip) in [("t:faithful", true), ("t:fixed", false)] {
        let r = ruleset_as(profile);
        let mut state = BattleState::new(&r, &armies(&r), 1).unwrap();
        for name in ["ray", "slow"] {
            cast_at(&mut state, &r, DEFENDER, name, RUNNERS);
        }
        let sc = context(&state, &r);
        let enemy = sc.view().other();
        let ray = removed(&state, &r, "ray", RUNNERS);
        let lone = enemy.on_copy(&state).value(RUNNERS, ray);
        let itself = enemy.on(&state).value(RUNNERS, ray);
        assert!(itself > 0, "{profile}");
        assert_eq!(lone, 0, "{profile}");
        let slow = removed(&state, &r, "slow", RUNNERS);
        let copy = without(&state, &r, "slow", RUNNERS);
        let lone = enemy.on_copy(&copy).value(RUNNERS, slow);
        let itself = enemy.on(&copy).value(RUNNERS, slow);
        assert!(lone < itself, "{profile}: {lone} {itself}");
        // Cure values the removal so with the slip, the stack itself without.
        let cure = sc.value(RUNNERS, Candidate::cast(spell(&r, "cure"), 1, 6));
        let mut names = ["ray", "slow"];
        names.sort_by_key(|n| spell(&r, n).index());
        let mut copy = state.clone();
        let mut want = 0;
        for name in names {
            let c = removed(&state, &r, name, RUNNERS);
            if name == "ray" && !slip {
                continue;
            }
            computer::end_spell(&mut copy, &r, (RUNNERS, c.spell), name != "ray");
            want += if slip {
                enemy.on_copy(&copy).value(RUNNERS, c)
            } else {
                enemy.on(&copy).value(RUNNERS, c)
            };
        }
        assert_eq!(cure, want, "{profile}");
    }
}

// covers: BAT-AI-016
#[test]
fn berserk_and_hypnotize_meet_the_stack_itself_on_the_copy() {
    // The enemy berserks (then hypnotizes) the side's runners; Cure's removal values the ended
    // spell in the enemy's half on the copy. With the slip the battle's record of the runners is
    // one of the copy's targets: Berserk's nearest (0x445490), one of the side Hypnotize walks.
    let r = ruleset_as("t:faithful");
    let mut state = BattleState::new(&r, &armies(&r), 1).unwrap();
    cast_at(&mut state, &r, DEFENDER, "berserk", RUNNERS);
    let sc = context(&state, &r);
    let enemy = sc.view().other();
    let c = removed(&state, &r, "berserk", RUNNERS);
    let copy = without(&state, &r, "berserk", RUNNERS);
    let view = enemy.on_copy(&copy);
    let (targets, _) = computer::berserk_search_of_copy(&copy, &r, RUNNERS);
    let (mut sum, mut n) = (0i32, 0u32);
    for t in targets {
        sum = sum.wrapping_add(if t == RUNNERS {
            on_itself((&r, DEFENDER), &state, &copy, RUNNERS)
        } else {
            view.berserk_attack((RUNNERS, t))
        });
        n += 1;
    }
    assert!(n > 0);
    let mean = i32::from_ne_bytes((u32::from_ne_bytes(sum.to_ne_bytes()) / n).to_ne_bytes());
    assert_eq!(view.value(RUNNERS, c), mean);
    assert_ne!(mean, enemy.on(&copy).value(RUNNERS, c));
    // Hypnotize: the copy's walk is the stack's with the record itself added (its position is
    // the map's start).
    let mut state = BattleState::new(&r, &armies(&r), 1).unwrap();
    cast_at(&mut state, &r, DEFENDER, "hypnotize", RUNNERS);
    let sc = context(&state, &r);
    let enemy = sc.view().other();
    let c = removed(&state, &r, "hypnotize", RUNNERS);
    let copy = without(&state, &r, "hypnotize", RUNNERS);
    let itself = on_itself((&r, DEFENDER), &state, &copy, RUNNERS);
    let walk = enemy.on(&copy).value(RUNNERS, c);
    assert!(itself > walk);
    assert_eq!(enemy.on_copy(&copy).value(RUNNERS, c), walk.max(itself));
}

// covers: BAT-AI-016
#[test]
fn a_copy_counts_its_turns_at_its_own_speed() {
    // Slowed at Expert the runners need two turns to their threat enemy; the context's record
    // keeps those. On a copy without Slow (a removal's) Bloodlust counts one turn at the copy's
    // speed (0x448BD0 with 0x448CD0 as it values), in both profiles.
    for profile in ["t:faithful", "t:fixed"] {
        let r = ruleset_as(profile);
        let mut state = BattleState::new(&r, &armies(&r), 1).unwrap();
        cast_at(&mut state, &r, DEFENDER, "slow", RUNNERS);
        let ctx = computer::context(&state, &r, ATTACKER);
        let slowed = computer::threat(&state, &r, &ctx, RUNNERS).unwrap();
        assert!(slowed.turns > 1, "{profile}");
        let sc = context(&state, &r);
        let bloodlust = Candidate::cast(spell(&r, "bloodlust"), 2, 6);
        assert_eq!(sc.view().value(RUNNERS, bloodlust), 0, "{profile}");
        let copy = without(&state, &r, "slow", RUNNERS);
        assert!(
            sc.view().on(&copy).value(RUNNERS, bloodlust) > 0,
            "{profile}"
        );
        assert!(
            sc.view().on_copy(&copy).value(RUNNERS, bloodlust) > 0,
            "{profile}"
        );
    }
}

// covers: BAT-AI-016
#[test]
fn hypnotize_takes_a_shooters_whole_side() {
    // The enemy archers can shoot now: the map (0x4B2DA0) marks every other stack of their side
    // (0x4B2F18), also those their walk does not reach (a turn at Basic).
    for profile in ["t:faithful", "t:fixed"] {
        let r = ruleset_as(profile);
        // The archers lead a column of seven: the last stands ten rows away.
        let column = setup(
            &[(RUNNER, 10)],
            &[
                (ARCHER, 6),
                (RUNNER, 4),
                (GOG, 5),
                (GOG, 1),
                (GOG, 1),
                (GOG, 1),
                (RUNNER, 1),
            ],
        );
        let state = BattleState::new(&r, &heroes(&r, column), 1).unwrap();
        assert!(computer::shoots(&state, &r, ENEMY_ARCHERS));
        let sc = context(&state, &r);
        let view = sc.view();
        // At Basic the walk's range is a turn (within the class) times the speed.
        let c = Candidate::cast(spell(&r, "hypnotize"), 1, 6);
        let range = computer::unslowed_speed(&state, ENEMY_ARCHERS);
        let walk = computer::map_within(&state, &r, (ENEMY_ARCHERS, ATTACKER), range);
        let others: Vec<StackId> = (0u8..)
            .zip(state.stacks())
            .filter(|(i, s)| s.side == DEFENDER && StackId(*i) != ENEMY_ARCHERS)
            .map(|(i, _)| StackId(i))
            .collect();
        assert!(
            others
                .iter()
                .any(|t| walk.mark(state.stacks()[t.index()].hex).is_none())
        );
        let best = others
            .iter()
            .map(|&t| view.berserk_attack((ENEMY_ARCHERS, t)))
            .fold(0, i32::max);
        assert_eq!(view.value(ENEMY_ARCHERS, c), best, "{profile}");
    }
}
