//! Hit points per creature under Poison and Age (E04b T7a2; BAT-MAG-044), after the original's
//! code (`docs/formats/installation.md` §7.20): a stack's poison factor falls by a tenth at each
//! Poison (the cast and every round start while it lasts), to half at most, and stays down after
//! Poison ends until a Cure; Age halves the hit points while it lasts. The hit points per creature
//! are recomputed from the stack's start value, the factor and Age; the top creature keeps its
//! lost hit points, at least 1 left. Kept beside the stacks (the hot paths never read it).

use h3_content::SINGLE_PRECISION_RATES;
use h3_content_battle::BattleRuleset;
use h3_kernel::arith::{DOUBLE_BITS, SINGLE_BITS, X87};
use h3_kernel::{CanonicalHash, StateHasher};

use crate::state::{BattleState, STACK_CAPACITY, StackId};

/// Most tenths Poison takes (the factor stops at 0.5).
pub const MAX_POISON_TENTHS: u8 = 5;

/// What Poison and Age keep per stack, by stack id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Vitality {
    /// The stack's hit points per creature at its start, recorded when Poison or Age first
    /// changes them (0: never changed, its `max_hp` is the start value).
    pub(crate) base: [u32; STACK_CAPACITY],
    /// The tenths Poison took for good (0 to [`MAX_POISON_TENTHS`]).
    pub(crate) poison: [u8; STACK_CAPACITY],
}

impl Default for Vitality {
    fn default() -> Self {
        Self {
            base: [0; STACK_CAPACITY],
            poison: [0; STACK_CAPACITY],
        }
    }
}

impl CanonicalHash for Vitality {
    /// Per stack: the start value, the tenths (v31).
    fn hash_into(&self, h: &mut StateHasher) {
        for (&b, &p) in self.base.iter().zip(&self.poison) {
            h.write_u32(b);
            h.write_u8(p);
        }
    }
}

/// The poison factor after `tenths` Poisons as the original stores it (an `f32`): from 1, each
/// time `max(factor − f32(0.1), 0.5)` in the x87's precision, stored as an `f32`.
fn single_factor(tenths: u8) -> X87 {
    let tenth = X87::ratio(1, 10, SINGLE_BITS).unwrap_or(X87::ZERO);
    let half = X87::ratio(1, 2, DOUBLE_BITS).unwrap_or(X87::ZERO);
    (0..tenths.min(MAX_POISON_TENTHS))
        .fold(X87::ONE, |f, _| f.minus(tenth).max(half).round(SINGLE_BITS))
}

/// The hit points per creature of a stack that started with `base`, with `tenths` taken by
/// Poison and, if `aged`, halved (BAT-MAG-044): `⌊base × (10 − tenths) / 10 × (½ if aged) +
/// 0.95⌋` with exact decimals or, with quirk Q-0005 (`single`), as the original's x87 code
/// computes it from `f32` values (0x4448F0, 0x447120, 0x444510).
pub(crate) fn hit_points(base: u32, tenths: u8, aged: bool, single: bool) -> u32 {
    let tenths = tenths.min(MAX_POISON_TENTHS);
    let hp = if single {
        let f = single_factor(tenths);
        let mut v = X87::int(u64::from(base)).round(SINGLE_BITS).times(f);
        if aged {
            v = v.times(X87::ratio(1, 2, SINGLE_BITS).unwrap_or(X87::ZERO));
        }
        v.plus(X87::ratio(95, 100, SINGLE_BITS).unwrap_or(X87::ZERO))
            .trunc()
    } else {
        let a = if aged { 2 } else { 1 };
        (100 * u64::from(base) * u64::from(10 - tenths) + 950 * a) / (1000 * a)
    };
    u32::try_from(hp).unwrap_or(u32::MAX).max(1)
}

/// Sets stack `s`'s hit points per creature from its start value, its poison and whether it is
/// `aged` (BAT-MAG-044); a stack that neither ever changes keeps them. The top creature keeps
/// the hit points it lost, at least 1 left.
pub(crate) fn materialize(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    s: StackId,
    aged: bool,
) {
    let i = s.index();
    let tenths = state.vitality.poison[i];
    if state.vitality.base[i] == 0 {
        if !aged && tenths == 0 {
            return;
        }
        state.vitality.base[i] = state.at(s).max_hp;
    }
    let single = ruleset.quirks().is_enabled(SINGLE_PRECISION_RATES);
    let hp = hit_points(state.vitality.base[i], tenths, aged, single);
    let stack = state.at_mut(s);
    let lost = stack.max_hp.saturating_sub(stack.top_hp);
    stack.max_hp = hp;
    if stack.is_alive() {
        stack.top_hp = hp.saturating_sub(lost).max(1);
    }
}

/// Poison takes another tenth of stack `s`'s hit points (BAT-MAG-044), up to half; the caller
/// recomputes the stack's values.
pub(crate) fn poison(state: &mut BattleState, s: StackId) {
    let i = s.index();
    if state.vitality.base[i] == 0 {
        state.vitality.base[i] = state.at(s).max_hp;
    }
    let p = &mut state.vitality.poison[i];
    *p = (*p + 1).min(MAX_POISON_TENTHS);
}

/// Gives back what Poison took of stack `s` (a Cure, BAT-MAG-044); the caller recomputes the
/// stack's values.
pub(crate) fn restore(state: &mut BattleState, s: StackId) {
    state.vitality.poison[s.index()] = 0;
}

/// The start value of stack `s`'s hit points per creature (a clone starts from it, as the
/// original sets a new stack up from its creature, BAT-MAG-038).
pub(crate) fn start_hit_points(state: &BattleState, s: StackId) -> u32 {
    match state.vitality.base[s.index()] {
        0 => state.at(s).max_hp,
        base => base,
    }
}

/// Forgets what Poison and Age kept of stack `id` (a slot a created stack takes).
pub(crate) fn reset(state: &mut BattleState, id: StackId) {
    state.vitality.base[id.index()] = 0;
    state.vitality.poison[id.index()] = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_single_precision_factors_follow_the_originals_f32_values() {
        // 1, 0x3F666666, 0x3F4CCCCC, 0x3F333332, 0x3F199998, 0.5: the f32 values the x87 code
        // stores (each step rounds f − 0.1f to the nearest f32).
        let bits = [
            0x3F80_0000,
            0x3F66_6666,
            0x3F4C_CCCC,
            0x3F33_3332,
            0x3F19_9998,
            0x3F00_0000,
        ];
        for (k, b) in (0u8..).zip(bits) {
            assert_eq!(single_factor(k).to_f32_bits(), b, "{k}");
        }
        assert_eq!(single_factor(9), single_factor(5));
    }

    #[test]
    fn hit_points_round_up_from_0_05_and_stop_at_half() {
        // Exact: 25 × 0.9 + 0.95 = 23.45; 25 × 0.5 + 0.95 = 13.45; aged 12.5 + 0.95 = 13.45.
        assert_eq!(hit_points(25, 1, false, false), 23);
        assert_eq!(hit_points(25, 5, false, false), 13);
        assert_eq!(hit_points(25, 7, false, false), 13);
        assert_eq!(hit_points(25, 0, true, false), 13);
        assert_eq!(hit_points(25, 5, true, false), 7);
        // 10 × 0.9 = 9 (+0.95 → 9) both ways; 10 × 0.6: the f32 factor lies below 0.6.
        for single in [false, true] {
            assert_eq!(hit_points(10, 1, false, single), 9);
            assert_eq!(hit_points(10, 4, false, single), 6);
            assert_eq!(hit_points(1, 5, true, single), 1);
            assert_eq!(hit_points(40, 0, false, single), 40);
        }
        // A value at .05 reaches the next whole number with 0.95 only exactly: 9 poisoned once
        // and aged, 9 × 0.9 / 2 = 4.05 → 5; the f32 factor 0.89999998 and 0.95's f32
        // 0.94999999 give 4.9999998 → 4 (Q-0005). Likewise 3 × 0.7 / 2 = 1.05 → 2 or 1.
        assert_eq!(hit_points(9, 1, true, false), 5);
        assert_eq!(hit_points(9, 1, true, true), 4);
        assert_eq!(hit_points(3, 3, true, false), 2);
        assert_eq!(hit_points(3, 3, true, true), 1);
        assert_eq!(hit_points(21, 1, false, true), 19);
    }
}
