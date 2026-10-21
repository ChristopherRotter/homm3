//! The last retaliation's gain and the walker's side's threat records on its turn (0x41F3B0,
//! 0x422B20 with flag 1; BAT-AI-009).

use h3_battle::{BattleState, Command, StackId, computer};
use h3_contracts::BattleSide;
use h3_kernel::arith::X87;

use super::{ARCHER, EFREET, GOG, RUNNER, on, play, ruleset_as, setup};
use crate::gain::{gain, records};

/// The battle of the attacker's efreet `w` and runners `o` against the defender's gogs `e`, on
/// Hard, at the efreet's first turn.
fn efreet_and_runners(profile: &str, (w, o, e): (u32, u32, u32)) -> BattleState {
    let r = ruleset_as(profile);
    let state = BattleState::new(
        &r,
        &on(2, setup(&[(EFREET, w), (RUNNER, o)], &[(GOG, e)])),
        3,
    )
    .unwrap();
    assert_eq!(state.active(), Some(StackId(0)));
    state
}

/// The gain the formula gives for the runners `o` against the gogs `e` after the efreet `w`
/// struck them, from the computer's numbers (0x41F3B0's arithmetic, slip 4 per `slips`).
fn expected(state: &BattleState, profile: &str, slips: bool) -> i32 {
    let r = ruleset_as(profile);
    let (w, o, e) = (StackId(0), StackId(1), StackId(2));
    let ctx = computer::context(state, &r, BattleSide::Attacker);
    let creatures = |id: StackId| {
        computer::hit_points(state, id).div_ceil(u64::from(state.stacks()[id.index()].max_hp))
    };
    let strike = |a: StackId| {
        i32::try_from(computer::damage_estimate(
            state,
            &r,
            (a, e),
            creatures(a),
            false,
            0,
        ))
        .unwrap()
    };
    let rest = i32::try_from(computer::hit_points(state, e)).unwrap() - strike(w);
    let d = strike(o).min(rest);
    let hp = i32::try_from(state.stacks()[2].max_hp).unwrap();
    let worth = computer::worth(state, &r, &ctx, e, false);
    let map = computer::turn_map(state, &r, o, BattleSide::Attacker);
    let cost = map.mark(state.stacks()[2].hex).unwrap();
    let v = computer::strike_value(state, &r, &ctx, (o, e), u8::try_from(cost).unwrap());
    let kills = if ctx.desperate() {
        let top = if slips {
            rest % hp
        } else {
            (hp - rest % hp) % hp
        };
        X87::int(u64::try_from((top + d) / hp).unwrap()).times(worth)
    } else {
        X87::int(u64::try_from(d).unwrap())
            .times(worth)
            .over(X87::int(u64::try_from(hp).unwrap()))
            .unwrap()
    };
    (computer::ftol(kills) - v).max(0)
}

// covers: BAT-AI-009
#[test]
fn the_runners_gain_when_the_efreet_takes_the_gogs_last_retaliation() {
    for profile in ["t:faithful", "t:fixed"] {
        let r = ruleset_as(profile);
        let state = efreet_and_runners(profile, (20, 60, 40));
        let ctx = computer::context(&state, &r, BattleSide::Attacker);
        let side = (StackId(0), BattleSide::Attacker);
        let recs = records(&state, &r, &ctx, side);
        let got = gain(
            &state,
            &r,
            &ctx,
            &recs,
            (StackId(0), BattleSide::Attacker, StackId(2)),
        );
        assert!(!ctx.desperate());
        assert_eq!(got, expected(&state, profile, true), "{profile}");
        assert!(got > 0, "{profile}: {got}");
    }
}

// covers: BAT-AI-009
#[test]
fn the_desperate_kills_with_and_without_the_slip() {
    // One efreet and one runner against 300 gogs: desperate (the side's strength doubled is
    // below the other's, above Easy). The gogs' top creature after the efreet's strike keeps a
    // part of its 13 hit points, which the slip adds where its lost ones belong.
    let (faithful, fixed) = ("t:faithful", "t:fixed");
    let state = efreet_and_runners(faithful, (1, 1, 300));
    let r = ruleset_as(faithful);
    let ctx = computer::context(&state, &r, BattleSide::Attacker);
    assert!(ctx.desperate());
    let at = |profile: &str| {
        let r = ruleset_as(profile);
        let ctx = computer::context(&state, &r, BattleSide::Attacker);
        let recs = records(&state, &r, &ctx, (StackId(0), BattleSide::Attacker));
        gain(
            &state,
            &r,
            &ctx,
            &recs,
            (StackId(0), BattleSide::Attacker, StackId(2)),
        )
    };
    assert_eq!(at(faithful), expected(&state, faithful, true));
    assert_eq!(at(fixed), expected(&state, fixed, false));
    // The slip counts one gog more (its worth, 900): 970 against 70.
    assert_eq!((at(faithful), at(fixed)), (970, 70));
}

// covers: BAT-AI-009
#[test]
fn no_gain_without_a_last_retaliation_or_a_stack_that_strikes_after_the_enemy() {
    let profile = "t:faithful";
    let r = ruleset_as(profile);
    // The efreet strikes the gogs and spends their retaliation: at the runners' turn none is left.
    let mut state = efreet_and_runners(profile, (20, 60, 40));
    let gogs = StackId(2);
    let from = h3_battle::Field::new(r.battle().field)
        .attack_positions(
            false,
            BattleSide::Attacker,
            state.stacks()[2].footprint(h3_battle::Field::new(r.battle().field)),
        )
        .into_iter()
        .flatten()
        .find(|&h| {
            h3_battle::validate(
                &state,
                &r,
                BattleSide::Attacker,
                Command::Attack {
                    target: gogs,
                    from: h,
                },
            )
            .is_ok()
        });
    let from = from.unwrap();
    play(&mut state, &r, Command::Attack { target: gogs, from });
    assert_eq!(state.active(), Some(StackId(1)));
    assert_eq!(state.stacks()[2].retaliations_left, 0);
    let ctx = computer::context(&state, &r, BattleSide::Attacker);
    let recs = records(&state, &r, &ctx, (StackId(1), BattleSide::Attacker));
    assert_eq!(
        gain(
            &state,
            &r,
            &ctx,
            &recs,
            (StackId(1), BattleSide::Attacker, gogs)
        ),
        0
    );
    // A second efreet stack acts after the defender's runners (the same speed, the sides
    // alternating, the walker's first): it records none of them, though it reaches them, so the
    // runners' last retaliation gains the walker nothing.
    let state = BattleState::new(
        &r,
        &on(2, setup(&[(EFREET, 20), (EFREET, 30)], &[(RUNNER, 40)])),
        3,
    )
    .unwrap();
    assert_eq!(state.active(), Some(StackId(0)));
    let ctx = computer::context(&state, &r, BattleSide::Attacker);
    let recs = records(&state, &r, &ctx, (StackId(0), BattleSide::Attacker));
    assert!(
        computer::turn_map(&state, &r, StackId(1), BattleSide::Attacker)
            .mark(state.stacks()[2].hex)
            .is_some()
    );
    assert_eq!(
        gain(
            &state,
            &r,
            &ctx,
            &recs,
            (StackId(0), BattleSide::Attacker, StackId(2))
        ),
        0
    );
    // A shooter of the side records nothing either (it shoots): the archers beside the efreet.
    let state = BattleState::new(
        &r,
        &on(2, setup(&[(EFREET, 5), (ARCHER, 10)], &[(GOG, 30)])),
        3,
    )
    .unwrap();
    let ctx = computer::context(&state, &r, BattleSide::Attacker);
    let recs = records(&state, &r, &ctx, (StackId(0), BattleSide::Attacker));
    assert_eq!(
        gain(
            &state,
            &r,
            &ctx,
            &recs,
            (StackId(0), BattleSide::Attacker, StackId(2))
        ),
        0
    );
}
