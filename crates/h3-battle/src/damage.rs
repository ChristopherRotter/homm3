//! Damage (BAT-DMG-001..003, BAT-DMG-006/008, BAT-RNG-002/003) and the Defend bonus (BAT-TRN-004).
//!
//! A strike's damage is its base damage (BAT-DMG-001) times the sum of its bonuses and the product
//! of its reductions, computed exactly with one final rounding down (BAT-DMG-002):
//! `floor(base · (1 + Σ bonus) · Π (1 − reduction))`, at least 1. The rates are exact decimals or,
//! with quirk Q-0005, their single-precision values (`h3_kernel::arith::single_precision`).

use core::num::NonZeroU64;

use h3_content::BattleParams;
use h3_kernel::Rng;
use h3_kernel::arith::{SINGLE_FRAC_BITS, WideUint, mul_div_floor, single_precision};

use crate::state::{Stack, StackFlags};

/// Base damage of a stack of `count` creatures with damage range `[min, max]`: for up to
/// `samples` creatures every creature rolls and the rolls are summed; for more, `samples` rolls
/// are summed and scaled by `count / samples`, rounded down.
// rule: BAT-DMG-001
pub(crate) fn base_damage(rng: &mut Rng, count: u32, min: u16, max: u16, samples: u8) -> u64 {
    let rolls = count.min(u32::from(samples));
    let mut sum: u64 = 0;
    for _ in 0..rolls {
        sum += u64::from(rng.range_inclusive(u32::from(min), u32::from(max)));
    }
    if count <= u32::from(samples) {
        sum
    } else {
        // `samples >= 1` (compiler-validated); saturate on absurd inputs instead of overflowing.
        mul_div_floor(sum, u64::from(count), u64::from(samples)).unwrap_or(u64::MAX)
    }
}

/// Most reductions one strike can have.
const MAX_REDUCTIONS: usize = 6;

/// Scale of the factors with exact decimal rates: basis points.
const DECIMAL_SCALE: u64 = 10_000;

/// The same as a divisor.
const DECIMAL_DIVISOR: NonZeroU64 = match NonZeroU64::new(DECIMAL_SCALE) {
    Some(d) => d,
    None => NonZeroU64::MIN,
};

/// The unit a rate parameter is given in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Unit {
    /// Hundredths.
    Percent,
    /// Thousandths.
    Permille,
}

impl Unit {
    /// The unit's denominator.
    const fn den(self) -> u16 {
        match self {
            Self::Percent => 100,
            Self::Permille => 1000,
        }
    }
}

/// The bonuses (summed) and reductions (multiplied) of one strike, as numerators over the scale:
/// basis points with exact decimal rates, `2^SINGLE_FRAC_BITS` with the single-precision values
/// of the original (quirk Q-0005, `quirk.battle.single_precision_rates`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Factors {
    single: bool,
    bonus: u64,
    reductions: [u64; MAX_REDUCTIONS],
    len: usize,
}

impl Factors {
    /// No factors yet; `single` selects the single-precision rates (Q-0005).
    pub(crate) const fn new(single: bool) -> Self {
        Self {
            single,
            bonus: 0,
            reductions: [0; MAX_REDUCTIONS],
            len: 0,
        }
    }

    /// The numerator of 1.
    const fn scale(&self) -> u64 {
        if self.single {
            1 << SINGLE_FRAC_BITS
        } else {
            DECIMAL_SCALE
        }
    }

    /// The rate `value` (in `unit`) at the scale: exact, or its single-precision value (Q-0005).
    // rule: BAT-DMG-002
    pub(crate) fn rate(&self, value: u16, unit: Unit) -> u64 {
        if self.single {
            // The denominator is never 0.
            single_precision(value, unit.den()).unwrap_or(0)
        } else {
            u64::from(value) * (DECIMAL_SCALE / u64::from(unit.den()))
        }
    }

    /// Adds a bonus (a [`Factors::rate`], possibly times a count) to the bonus sum.
    pub(crate) fn bonus(&mut self, rate: u64) {
        self.bonus = self.bonus.saturating_add(rate);
    }

    /// Adds a reduction (a [`Factors::rate`], at most 1). Reductions of 0 are left out; strikes
    /// have fewer than `MAX_REDUCTIONS` others by construction (the call sites are the rules
    /// below), extra ones are ignored.
    pub(crate) fn reduce(&mut self, rate: u64) {
        if rate == 0 {
            return;
        }
        let scale = self.scale();
        if let Some(slot) = self.reductions.get_mut(self.len) {
            *slot = rate.min(scale);
            self.len += 1;
        }
    }

    /// The damage of `base` with these factors: `floor(base · (1 + Σ bonus) · Π (1 − reduction))`,
    /// exact, rounded down once, at least 1 (BAT-DMG-002). Saturates on absurd inputs.
    // rule: BAT-DMG-002
    pub(crate) fn apply(&self, base: u64) -> u64 {
        self.product(base).unwrap_or(u64::MAX).max(1)
    }

    /// The rounded-down product, `None` if it exceeds `u64` (at most eight factors of `u64`, so
    /// the wide product itself never saturates).
    fn product(&self, base: u64) -> Option<u64> {
        let scale = self.scale();
        let reductions = &self.reductions[..self.len];
        let mut n = WideUint::new(base).saturating_mul(scale.saturating_add(self.bonus));
        for r in reductions {
            n = n.saturating_mul(scale - r);
        }
        let factors = u32::try_from(reductions.len() + 1).unwrap_or(u32::MAX);
        if self.single {
            n = n.shr_floor(SINGLE_FRAC_BITS.saturating_mul(factors));
        } else {
            for _ in 0..factors {
                n = n.div_floor(DECIMAL_DIVISOR);
            }
        }
        n.to_u64()
    }
}

/// The attack/defense part of the factors: a bonus of `per_point` % per point of attack over
/// defense (capped) or a reduction of `per_point` ‰ per point of defense over attack (capped).
// rule: BAT-DMG-002
pub(crate) fn attack_defense(
    factors: &mut Factors,
    attack: u16,
    defense: u16,
    params: &BattleParams,
) {
    if attack > defense {
        let diff = u64::from(attack - defense);
        let per_point = factors.rate(params.attack_bonus_per_point_percent, Unit::Percent);
        let cap = factors.rate(params.attack_bonus_cap_percent, Unit::Percent);
        factors.bonus(diff.saturating_mul(per_point).min(cap));
    } else {
        let diff = u64::from(defense - attack);
        let per_point = factors.rate(params.defense_reduction_per_point_permille, Unit::Permille);
        let cap = factors.rate(params.defense_reduction_cap_permille, Unit::Permille);
        factors.reduce(diff.saturating_mul(per_point).min(cap));
    }
}

/// Defense used when `stack` is hit: its defense plus, while defending, `defend_bonus_percent`
/// of it rounded down but at least `defend_bonus_min`.
// rule: BAT-TRN-004
pub(crate) fn effective_defense(stack: &Stack, params: &BattleParams) -> u16 {
    if !stack.flags.contains(StackFlags::DEFENDING) {
        return stack.defense;
    }
    let pct = u32::from(stack.defense) * u32::from(params.defend_bonus_percent) / 100;
    let bonus = pct.max(u32::from(params.defend_bonus_min));
    u16::try_from(u32::from(stack.defense) + bonus).unwrap_or(u16::MAX)
}

/// The defense a creature that ignores `percent` % of it sees: `floor(defense · (100 − percent)
/// / 100)` (Q19).
// rule: BAT-DMG-006
pub(crate) fn reduced_defense(defense: u16, percent: u8) -> u16 {
    let kept = u32::from(defense) * u32::from(100 - percent.min(100)) / 100;
    // `kept <= defense`.
    u16::try_from(kept).unwrap_or(defense)
}

/// Heals the living `stack` by up to `hp` hit points, resurrecting creatures up to its count at
/// the start of the battle; returns the hit points regained and the creatures brought back.
// rule: BAT-DMG-008
pub(crate) fn resurrect(stack: &mut Stack, hp: u64) -> (u64, u32) {
    let max_hp = u64::from(stack.max_hp);
    let total = stack.total_hp();
    let healed = hp.min((u64::from(stack.base_count) * max_hp).saturating_sub(total));
    if healed == 0 {
        return (0, 0);
    }
    let before = stack.count;
    let new_total = total + healed;
    let count = new_total.div_ceil(max_hp);
    // `0 < count <= base_count`, so the conversions cannot fail.
    stack.count = u32::try_from(count).unwrap_or(stack.base_count);
    stack.top_hp = u32::try_from(new_total - (count - 1) * max_hp).unwrap_or(stack.max_hp);
    (healed, stack.count - before)
}

/// Deals `damage` to `stack`: the top creature takes damage first; returns the number of
/// creatures killed.
// rule: BAT-DMG-003
pub(crate) fn apply_damage(stack: &mut Stack, damage: u64) -> u32 {
    let total = stack.total_hp();
    let before = stack.count;
    if damage >= total {
        stack.count = 0;
        stack.top_hp = 0;
        return before;
    }
    let remaining = total - damage;
    let max_hp = u64::from(stack.max_hp);
    let count = remaining.div_ceil(max_hp);
    // `count <= before` because `remaining < total`, so the conversion cannot fail.
    stack.count = u32::try_from(count).unwrap_or(before);
    stack.top_hp = u32::try_from(remaining - (count - 1) * max_hp).unwrap_or(stack.max_hp);
    before - stack.count
}
