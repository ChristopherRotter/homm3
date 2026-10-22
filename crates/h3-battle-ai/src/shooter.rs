//! The computer's shooter (0x41F060; `docs/formats/installation.md` §7.33, §7.35; BAT-AI-002 to
//! BAT-AI-004): a stack that can shoot now shoots the stack the computer picks for it (0x41EB80,
//! the automatic Ballista's choice, BAT-WAR-008; an area shooter's by its value at a hex,
//! 0x41EDA0), a Cyclops may batter the weakest wall instead (0x41EEA0), and without a target it
//! defends (action 3). The target is `h3-battle`'s [`computer::valued_shot`]; the wall shot, which
//! draws, is the policy's. With BAT-QUIRK-25 off a shooter that casts a fixed spell (the Storm
//! and Ice Elementals) first weighs its cast against the shot's value (survey slip 25,
//! [`casts::before_shot`]).

use alloc::vec::Vec;
use core::cmp::Ordering;

use h3_battle::computer::{self, Context, Shot};
use h3_battle::{BattleSide, BattleState, Command, StackId};
use h3_content_battle::BattleRuleset;
use h3_kernel::Rng;
use h3_kernel::arith::X87;

use crate::casts;

/// The command of the computer's shooter `id` (0x41F060): [`computer::valued_shot`]'s target, the
/// shot aimed where it says; a wall shot when [`wall_shot`] batters one; Defend without a target.
/// With BAT-QUIRK-25 off, a cast of a fixed spell that beats the shot's value instead (survey
/// slip 25, [`casts::before_shot`]; its draws from `rng` after the shot's own).
// rule: BAT-AI-002
pub(crate) fn shoot(
    state: &BattleState,
    ruleset: &BattleRuleset,
    id: StackId,
    rng: &mut Rng,
) -> Command {
    let wall = |ctx: &Context, best: i32| wall_shot(state, ruleset, (ctx, id), best, rng);
    let (shot, value) = computer::valued_shot(state, ruleset, id, wall);
    if let Some(cast) = casts::before_shot(state, ruleset, id, value, rng) {
        return cast;
    }
    match shot {
        Shot::Stack { target, at } => Command::Shoot { target, at },
        Shot::Wall { part } => Command::ShootWall { part },
        Shot::Nothing => Command::Defend,
    }
}

/// The wall the computer's Cyclops `shooter` batters instead of shooting its target of value
/// `best` (0x41EEA0; BAT-AI-004), in its side's context `ctx`: none for the defender or without a
/// standing wall; else when the strength of its side's living stacks for which the computer
/// records no enemy (`B`: war machines but the Ballista, the hypnotized, berserk, blind or stone
/// for more than a round, walkers that reach no enemy) over its side's strength (`S₀`) exceeds
/// three times `best` over the other side's (`S₁`), as the original's doubles compare them, the
/// standing wall with the fewest hit points, one of several such drawn uniformly from `rng`
/// (`random(1, n)`, drawn also for one) in index order.
// rule: BAT-AI-004
fn wall_shot(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (ctx, shooter): (&Context, StackId),
    best: i32,
    rng: &mut Rng,
) -> Option<u8> {
    let side = state.stacks()[shooter.index()].controller();
    if side == BattleSide::Defender {
        return None;
    }
    let walls: Vec<(u8, u8)> = computer::walls(state).collect();
    let least = walls.iter().map(|&(_, hp)| hp).min()?;
    let locked = (0u8..)
        .zip(state.stacks())
        .filter(|(i, s)| {
            s.side == side
                && s.is_alive()
                && computer::records_no_enemy(state, ruleset, StackId(*i))
        })
        .fold(0i32, |sum, (i, _)| {
            sum.wrapping_add(computer::strength(state, ruleset, ctx, StackId(i)))
        });
    let (own, other) = ctx.strengths();
    let three = i64::from(best) * 3;
    if !exceeds(
        quotient(locked.into(), own.into()),
        quotient(three, other.into()),
    ) {
        return None;
    }
    let weakest = || walls.iter().filter(move |&&(_, hp)| hp == least);
    let n = u32::try_from(weakest().count()).unwrap_or(1);
    let k = rng.range_inclusive(1, n);
    weakest()
        .nth(usize::try_from(k - 1).unwrap_or(0))
        .map(|&(part, _)| part)
}

/// A quotient of two integers as the x87 divides them (`fild`, `fdiv`, at 53 bits: 0x41EF8E,
/// 0x41EFA6): its sign (−2 and 2 for the infinities, 0 for zero) and its magnitude rounded as the
/// FPU rounds it; `None` for 0 / 0, a NaN.
fn quotient(n: i64, d: i64) -> Option<(i8, X87)> {
    let sign = |v: i64| if v < 0 { -1 } else { 1 };
    if d == 0 {
        return (n != 0).then(|| (2 * sign(n), X87::ZERO));
    }
    if n == 0 {
        return Some((0, X87::ZERO));
    }
    let magnitude = X87::int(n.unsigned_abs())
        .over(X87::int(d.unsigned_abs()))
        .unwrap_or(X87::ZERO);
    Some((sign(n) * sign(d), magnitude))
}

/// Whether the left quotient is above the right one as 0x41EFA9's `fcompp` decides it: a NaN on
/// either side counts as above (the comparison sets C0 when unordered), an infinity is not above
/// an equal one.
fn exceeds(left: Option<(i8, X87)>, right: Option<(i8, X87)>) -> bool {
    let (Some((ls, lm)), Some((rs, rm))) = (left, right) else {
        return true;
    };
    match ls.cmp(&rs) {
        Ordering::Equal if ls < 0 => lm < rm,
        Ordering::Equal => lm > rm,
        order => order == Ordering::Greater,
    }
}

#[cfg(test)]
mod tests {
    use super::{exceeds, quotient};

    /// `B / S₀ > 3·best / S₁` as the original compares it, for `(B, S₀, best, S₁)`.
    fn batters((locked, own, best, other): (i64, i64, i64, i64)) -> bool {
        exceeds(quotient(locked, own), quotient(3 * best, other))
    }

    // covers: BAT-AI-004
    #[test]
    fn the_wall_test_compares_as_the_fpu() {
        // Finite quotients: a half against nothing, nothing against 0.6.
        assert!(batters((1, 2, 0, 5)));
        assert!(!batters((0, 2, 1, 5)));
        // Equal quotients are not above each other.
        assert!(!batters((3, 5, 1, 5)));
        // 0 / 0 is a NaN, which counts as above on either side.
        assert!(batters((0, 0, 1, 5)));
        assert!(batters((0, 2, 0, 0)));
        // A division by 0 is infinite: above every number, not above itself.
        assert!(batters((1, 0, 1, 5)));
        assert!(!batters((1, 0, 1, 0)));
        assert!(!batters((1, 2, 1, 0)));
        assert!(!batters((-1, 0, 0, 5)));
        // Wrapped strengths below 0: the larger magnitude is the smaller quotient.
        assert!(!batters((-1, 2, 0, 5)));
        assert!(batters((-1, 2, -1, 5)));
        assert!(!batters((-2, 2, -1, 5)));
    }
}
