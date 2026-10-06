//! Damage (BAT-DMG-001..003, BAT-DMG-006/008, BAT-RNG-002/003) and the Defend bonus (BAT-TRN-004).
//!
//! A strike's damage is its base damage (BAT-DMG-001) with its bonuses and reductions
//! (BAT-DMG-002). With exact decimal rates it is `floor(base · (1 + Σ bonus) · Π (1 − reduction))`,
//! at least 1, rounded once. With quirk Q-0005 it follows the executable's three truncated stages
//! (`docs/formats/installation.md` §7.10), in its x87 arithmetic (`h3_kernel::arith::X87`): the
//! bonuses as an integer, then the defense and penalty factor, then the protection factor.

use core::num::NonZeroU64;

use h3_content::BattleParams;
use h3_kernel::Rng;
use h3_kernel::arith::{DOUBLE_BITS, SINGLE_BITS, WideUint, X87, mul_div_floor};

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

/// Scale of the exact decimal rates: basis points.
const BP: u64 = 10_000;

/// The same as a divisor.
const BP_DIVISOR: NonZeroU64 = match NonZeroU64::new(BP) {
    Some(d) => d,
    None => NonZeroU64::MIN,
};

/// Everything besides its base that decides a strike's damage (BAT-DMG-002), set by the rules
/// that contribute to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Strike {
    /// Jousting: percent per hex and hexes charged (BAT-DMG-005).
    pub jousting: (u16, u32),
    /// A lucky strike: +100 % (BAT-LCK-002).
    pub lucky: bool,
    /// The striker's attack and the target's defense as they meet (BAT-MAG-011, BAT-DMG-006,
    /// BAT-TRN-004).
    pub attack: i32,
    /// See `attack`.
    pub defense: i32,
    /// Hate against the target, percent (BAT-DMG-004).
    pub hate_percent: u16,
    /// The striker's hero's Offense or Archery, basis points (BAT-HER-004).
    pub hero_bonus_bp: u16,
    /// A death blow: +100 % (BAT-DMG-007).
    pub death_blow: bool,
    /// The Ballista's double damage: +100 % (BAT-WAR-006).
    pub double_damage: bool,
    /// The range or melee penalty, percent (BAT-RNG-002/003); 0 = none.
    pub penalty_percent: u16,
    /// The share of damage the target's Shield or Air Shield lets through, percent
    /// (BAT-MAG-012); 100 = none.
    pub protection_percent: u16,
    /// The target's hero's Armorer, basis points (BAT-HER-004).
    pub hero_reduction_bp: u16,
}

impl Strike {
    /// A strike of `attack` on `defense` with nothing else.
    pub(crate) const fn new(attack: i32, defense: i32) -> Self {
        Self {
            jousting: (0, 0),
            lucky: false,
            attack,
            defense,
            hate_percent: 0,
            hero_bonus_bp: 0,
            death_blow: false,
            double_damage: false,
            penalty_percent: 0,
            protection_percent: 100,
            hero_reduction_bp: 0,
        }
    }

    /// The damage of `base` (BAT-DMG-002): in the executable's stages with quirk Q-0005
    /// (`staged`), else exact and rounded once; at least 1.
    // rule: BAT-DMG-002
    pub(crate) fn damage(&self, base: u64, params: &BattleParams, staged: bool) -> u64 {
        if staged {
            self.staged(base, params)
        } else {
            self.exact(base, params)
        }
        .max(1)
    }

    /// `floor(base · (1 + Σ bonus) · Π (1 − reduction))` with decimal rates, in basis points.
    /// Saturates on absurd inputs.
    fn exact(&self, base: u64, p: &BattleParams) -> u64 {
        let diff = u64::from(self.attack.abs_diff(self.defense));
        let whole =
            u64::from(self.lucky) + u64::from(self.death_blow) + u64::from(self.double_damage);
        let mut bonus = whole * BP
            + u64::from(self.jousting.0) * u64::from(self.jousting.1) * 100
            + u64::from(self.hate_percent) * 100
            + u64::from(self.hero_bonus_bp);
        let mut reductions = [0u64; 4];
        if self.attack > self.defense {
            let per_point = u64::from(p.attack_bonus_per_point_percent) * 100;
            bonus += (diff * per_point).min(u64::from(p.attack_bonus_cap_percent) * 100);
        } else {
            let per_point = u64::from(p.defense_reduction_per_point_permille) * 10;
            reductions[0] =
                (diff * per_point).min(u64::from(p.defense_reduction_cap_permille) * 10);
        }
        reductions[1] = u64::from(self.penalty_percent) * 100;
        reductions[2] = u64::from(100u16.saturating_sub(self.protection_percent)) * 100;
        reductions[3] = u64::from(self.hero_reduction_bp);
        // One rounding: the five factors' product (at most 2^64 · (2^64)^5, so the wide
        // product never saturates) divided by BP^5 at the end.
        let mut n = WideUint::new(base).saturating_mul(BP + bonus);
        for r in reductions {
            n = n.saturating_mul(BP - r.min(BP));
        }
        for _ in 0..=reductions.len() {
            n = n.div_floor(BP_DIVISOR);
        }
        n.to_u64().unwrap_or(u64::MAX)
    }

    /// The executable's three stages (0x443F40), each truncated, in x87 arithmetic at
    /// `X87_PRECISION` bits: the rates of the code are doubles, the hero's and the spells'
    /// stored factors single precision (quirk Q-0005).
    fn staged(&self, base: u64, p: &BattleParams) -> u64 {
        // The code's constants and stored values: doubles and `f32`s (`d > 0` throughout).
        let double = |n: u16, d: u32| X87::ratio(u32::from(n), d, DOUBLE_BITS).unwrap_or(X87::ZERO);
        let single = |n: u16, d: u32| X87::ratio(u32::from(n), d, SINGLE_BITS).unwrap_or(X87::ZERO);
        let stored = |v: u64, bits: u32| X87::int(v).round(bits);
        let b = stored(base, DOUBLE_BITS);
        // 1. The bonuses as an integer, each fractional one truncated as it is added.
        let mut sum = b
            .times(stored(u64::from(self.jousting.1), DOUBLE_BITS))
            .times(double(self.jousting.0, 100))
            .trunc();
        if self.lucky {
            sum = sum.saturating_add(base);
        }
        if self.attack >= self.defense {
            let diff = X87::int(u64::from(self.attack.abs_diff(self.defense)));
            let rate = diff
                .times(double(p.attack_bonus_per_point_percent, 100))
                .min(double(p.attack_bonus_cap_percent, 100));
            sum = b.times(rate).plus(stored(sum, DOUBLE_BITS)).trunc();
        }
        sum = sum.saturating_add(mul_div_floor(base, self.hate_percent.into(), 100).unwrap_or(0));
        if self.hero_bonus_bp > 0 {
            sum = single(self.hero_bonus_bp, 10_000)
                .times(stored(base, SINGLE_BITS))
                .plus(stored(sum, SINGLE_BITS))
                .trunc();
        }
        let whole = u64::from(self.death_blow) + u64::from(self.double_damage);
        let d1 = base
            .saturating_add(sum)
            .saturating_add(base.saturating_mul(whole));
        // 2. Times the defense and penalty factor.
        let mut f = X87::ONE;
        if self.defense > self.attack {
            let diff = X87::int(u64::from(self.attack.abs_diff(self.defense)));
            let cap = p.defense_reduction_cap_permille.min(1000);
            f = X87::ONE
                .minus(diff.times(double(p.defense_reduction_per_point_permille, 1000)))
                .max(double(1000 - cap, 1000));
        }
        if self.penalty_percent > 0 {
            f = f.times(double(100u16.saturating_sub(self.penalty_percent), 100));
        }
        let d2 = stored(d1, DOUBLE_BITS).times(f).trunc();
        // 3. Times the protection factor.
        let mut g = X87::ONE;
        if self.protection_percent < 100 {
            g = single(self.protection_percent, 100);
        }
        if self.hero_reduction_bp > 0 {
            let kept = X87::ONE.minus(single(self.hero_reduction_bp.min(10_000), 10_000));
            g = kept.times(g);
        }
        stored(d2, DOUBLE_BITS).times(g).trunc()
    }
}

/// Defense used when `stack` is hit: its defense now (BAT-MAG-011) plus, while defending,
/// `defend_bonus_percent` of it rounded down (of 0 if it is below 0) but at least
/// `defend_bonus_min`.
// rule: BAT-TRN-004
pub(crate) fn effective_defense(stack: &Stack, params: &BattleParams) -> i32 {
    let defense = stack.defense_now();
    if !stack.flags.contains(StackFlags::DEFENDING) {
        return defense;
    }
    let pct = defense.max(0) * i32::from(params.defend_bonus_percent) / 100;
    defense.saturating_add(pct.max(i32::from(params.defend_bonus_min)))
}

/// The defense a creature that ignores `percent` % of it sees: `defense · (100 − percent) / 100`
/// rounded toward 0 (Q19; rounded down for a defense of at least 0).
// rule: BAT-DMG-006
pub(crate) fn reduced_defense(defense: i32, percent: u8) -> i32 {
    // At most |defense| · 100: no overflow for a defense of an `i16` sum.
    defense.saturating_mul(i32::from(100 - percent.min(100))) / 100
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
