//! Damage (BAT-DMG-001..003, BAT-DMG-006/008, BAT-RNG-002/003) and the Defend bonus (BAT-TRN-004).
//!
//! A strike's damage is its base damage (BAT-DMG-001) with its bonuses and reductions
//! (BAT-DMG-002). With exact decimal rates it is `floor(base · (1 + Σ bonus) · Π (1 − reduction))`,
//! at least 1, rounded once. With quirk Q-0005 it follows the executable's three truncated stages
//! (`docs/formats/installation.md` §7.10), in its x87 arithmetic (`h3_kernel::arith::X87`): the
//! bonuses as an integer, then the defense and penalty factor, then the protection factor.

use core::num::NonZeroU64;

use h3_content_battle::BattleParams;
use h3_contracts::Rate;
use h3_kernel::Rng;
use h3_kernel::arith::{
    DOUBLE_BITS, SINGLE_BITS, SINGLE_FRAC_BITS, WideUint, X87, mul_div_floor, single_precision,
};

use crate::magic::{Specialist, hero_rate};
use crate::state::{Stack, StackFlags, StackTraits};

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
    /// The striker's hero's Offense or Archery (BAT-HER-004).
    pub hero_bonus: Rate,
    /// The striker's hero's specialty in the striker's Bless: its bonus of the base joins the
    /// first stage (BAT-MAG-020).
    pub blessing: Option<Specialist>,
    /// A death blow: +100 % (BAT-DMG-007).
    pub death_blow: bool,
    /// The Ballista's double damage: +100 % (BAT-WAR-006).
    pub double_damage: bool,
    /// The range or melee penalty, percent (BAT-RNG-002/003); 0 = none.
    pub penalty_percent: u16,
    /// The striker's penalty against the target's creature, percent (BAT-DMG-010: the Psychic
    /// and Magic Elementals' half; the original's stage 2); 0 = none.
    pub creature_penalty_percent: u8,
    /// The share of its damage a retaliation of a stack that was blind when attacked deals,
    /// percent (BAT-MAG-024; the original's stage 2); 100 = none.
    pub retaliation_percent: u16,
    /// The share of damage the target's Shield or Air Shield lets through, percent
    /// (BAT-MAG-012); 100 = none.
    pub protection_percent: u16,
    /// The target's hero's Armorer (BAT-HER-004).
    pub hero_reduction: Rate,
    /// The target is petrified: half the damage, in the original's stage 3 (BAT-MAG-043).
    pub petrified: bool,
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
            hero_bonus: Rate::bp(0),
            blessing: None,
            death_blow: false,
            double_damage: false,
            penalty_percent: 0,
            creature_penalty_percent: 0,
            retaliation_percent: 100,
            protection_percent: 100,
            hero_reduction: Rate::bp(0),
            petrified: false,
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

    /// The first stage of the damage of `base`, before the defense, the penalties and the
    /// protections: the executable's stage 1 with quirk Q-0005 (`staged`), else `floor(base ·
    /// (1 + Σ bonus))` with decimal rates, plus a Bless specialist's bonus (Fire Shield's base,
    /// BAT-MAG-018).
    pub(crate) fn stage_one(&self, base: u64, params: &BattleParams, staged: bool) -> u64 {
        if staged {
            self.staged_one(base, params)
        } else {
            let bonus = self.exact_bonus(params);
            mul_div_floor(base, BP + bonus, BP)
                .unwrap_or(u64::MAX)
                .saturating_add_signed(self.blessed(base))
        }
    }

    /// A Bless specialist's bonus of `base` (BAT-MAG-020), or 0; never below `−base`.
    fn blessed(&self, base: u64) -> i64 {
        self.blessing.map_or(0, |b| {
            b.bonus(base).max(-i64::try_from(base).unwrap_or(i64::MAX))
        })
    }

    /// The bonus sum of the exact formula in basis points: the whole bonuses, jousting, hate,
    /// the hero's bonus and the attack advantage (capped).
    fn exact_bonus(&self, p: &BattleParams) -> u64 {
        let whole =
            u64::from(self.lucky) + u64::from(self.death_blow) + u64::from(self.double_damage);
        let mut bonus = whole * BP
            + u64::from(self.jousting.0) * u64::from(self.jousting.1) * 100
            + u64::from(self.hate_percent) * 100
            + u64::from(self.hero_bonus.bp);
        if self.attack > self.defense {
            let diff = u64::from(self.attack.abs_diff(self.defense));
            let per_point = u64::from(p.attack_bonus_per_point_percent) * 100;
            bonus += (diff * per_point).min(u64::from(p.attack_bonus_cap_percent) * 100);
        }
        bonus
    }

    /// `floor((base · (1 + Σ bonus) + a Bless specialist's bonus) · Π (1 − reduction))` with
    /// decimal rates, in basis points. Saturates on absurd inputs.
    fn exact(&self, base: u64, p: &BattleParams) -> u64 {
        let diff = u64::from(self.attack.abs_diff(self.defense));
        let bonus = self.exact_bonus(p);
        let mut reductions = [0u64; 7];
        if self.attack <= self.defense {
            let per_point = u64::from(p.defense_reduction_per_point_permille) * 10;
            reductions[0] =
                (diff * per_point).min(u64::from(p.defense_reduction_cap_permille) * 10);
        }
        reductions[1] = u64::from(self.penalty_percent) * 100;
        reductions[2] = u64::from(100u16.saturating_sub(self.protection_percent)) * 100;
        reductions[3] = u64::from(self.hero_reduction.bp);
        reductions[4] = u64::from(100u16.saturating_sub(self.retaliation_percent)) * 100;
        reductions[5] = u64::from(self.petrified) * BP / 2;
        reductions[6] = u64::from(self.creature_penalty_percent) * 100;
        // One rounding: the eight factors' product (at most 2^78 · (2^14)^7, so the wide
        // product never saturates) divided by BP^8 at the end. The specialist's bonus is a
        // whole amount (at least −base), so the first stage is `(base + extra) · BP + base ·
        // bonus` over BP.
        let extra = self.blessed(base);
        let whole = base.saturating_add_signed(extra);
        let mut n = WideUint::new(whole)
            .saturating_mul(BP)
            .saturating_add(WideUint::new(base).saturating_mul(bonus));
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
        let d1 = self.staged_one(base, p);
        // 2. Times the defense and penalty factor.
        let mut f = X87::ONE;
        if self.defense > self.attack {
            let diff = X87::int(u64::from(self.attack.abs_diff(self.defense)));
            let cap = p.defense_reduction_cap_permille.min(1000);
            f = X87::ONE
                .minus(diff.times(double(p.defense_reduction_per_point_permille, 1000)))
                .max(double(1000 - cap, 1000));
        }
        // The elementals' half right after the defense factor (`installation.md` §7.21).
        if self.creature_penalty_percent > 0 {
            let kept = 100u16.saturating_sub(self.creature_penalty_percent.into());
            f = f.times(double(kept, 100));
        }
        if self.penalty_percent > 0 {
            f = f.times(double(100u16.saturating_sub(self.penalty_percent), 100));
        }
        // Blind's retaliation factor, stored as an `f32` (`installation.md` §7.14).
        if self.retaliation_percent < 100 {
            f = f.times(single(self.retaliation_percent, 100));
        }
        let d2 = stored(d1, DOUBLE_BITS).times(f).trunc();
        // 3. Times the protection factor.
        let mut g = X87::ONE;
        if self.protection_percent < 100 {
            g = single(self.protection_percent, 100);
        }
        // Stone Gaze's half (`installation.md` §7.10).
        if self.petrified {
            g = g.times(double(1, 2));
        }
        let reduction = hero_rate(true, self.hero_reduction);
        if reduction > 0 {
            let kept = X87::ONE.minus(X87::fixed(reduction).min(X87::ONE));
            g = kept.times(g);
        }
        stored(d2, DOUBLE_BITS).times(g).trunc()
    }

    /// The executable's stage 1: the bonuses as an integer, each fractional one truncated as it
    /// is added, plus the base.
    fn staged_one(&self, base: u64, p: &BattleParams) -> u64 {
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
        let bonus = hero_rate(true, self.hero_bonus);
        if bonus > 0 {
            sum = X87::fixed(bonus)
                .times(stored(base, SINGLE_BITS))
                .plus(stored(sum, SINGLE_BITS))
                .trunc();
        }
        // A Bless specialist's bonus right after the hero's (`installation.md` §7.12).
        sum = sum.saturating_add_signed(self.blessed(base));
        let whole = u64::from(self.death_blow) + u64::from(self.double_damage);
        base.saturating_add(sum)
            .saturating_add(base.saturating_mul(whole))
    }
}

/// A rate of the code's constants: `n / d` as a double (`d > 0` throughout).
fn double(n: u16, d: u32) -> X87 {
    X87::ratio(u32::from(n), d, DOUBLE_BITS).unwrap_or(X87::ZERO)
}

/// A stored factor: `n / d` as an `f32`.
fn single(n: u16, d: u32) -> X87 {
    X87::ratio(u32::from(n), d, SINGLE_BITS).unwrap_or(X87::ZERO)
}

/// An integer stored at `bits` of precision.
fn stored(v: u64, bits: u32) -> X87 {
    X87::int(v).round(bits)
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

/// The defense a creature that ignores `percent` % of it sees: with quirk Q-0005 (`staged`) the
/// executable's `trunc(f32(defense) − f32(defense) × f32(percent / 100))`, the defense left
/// computed from single-precision values and then rounded toward 0 (0x4425C8 to 0x4425F5: the
/// share taken is never rounded on its own; 7 against a Behemoth leaves 4, and 10 leaves 5,
/// as the stored 0.4 lies above 0.4); else `defense · (100 − percent) / 100` rounded toward 0
/// (Q19; 7 leaves 4, 10 leaves 6).
// rule: BAT-DMG-006
pub(crate) fn reduced_defense(defense: i32, percent: u8, staged: bool) -> i32 {
    let percent = percent.min(100);
    if staged {
        // The magnitude apart from the sign: the rounding is toward 0 either way.
        let d = stored(u64::from(defense.unsigned_abs()), SINGLE_BITS);
        let left = d.minus(d.times(single(u16::from(percent), 100))).trunc();
        // At most |defense|.
        defense.signum() * i32::try_from(left).unwrap_or(0)
    } else {
        // At most |defense| · 100: no overflow for a defense of an `i16` sum.
        defense.saturating_mul(i32::from(100 - percent)) / 100
    }
}

/// The attack of a frenzied stack (BAT-MAG-019): `attack + defense × percent / 100` rounded
/// toward 0, with quirk Q-0005 (`staged`) the share `percent / 100` single precision as the
/// executable stores it (`installation.md` §7.11; exact for attacks and defenses below 2^20).
pub(crate) fn frenzied(attack: i32, defense: i32, percent: u16, staged: bool) -> i32 {
    let (num, scale) = if staged {
        (
            i128::from(single_precision(percent, 100).unwrap_or(0)),
            1i128 << SINGLE_FRAC_BITS,
        )
    } else {
        (i128::from(percent), 100)
    };
    let total = i128::from(attack) * scale + i128::from(defense) * num;
    i32::try_from(total / scale).unwrap_or(i32::MAX)
}

/// Heals `stack` by up to `hp` hit points, resurrecting creatures up to its count at the start of
/// the battle (a dead stack too: BAT-MAG-035); returns the hit points regained and the creatures
/// brought back.
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
/// creatures killed. Any damage kills every creature of a clone (BAT-MAG-038).
// rule: BAT-DMG-003
pub(crate) fn apply_damage(stack: &mut Stack, damage: u64) -> u32 {
    let total = stack.total_hp();
    let before = stack.count;
    // rule: BAT-MAG-038
    if damage >= total || stack.has(StackTraits::CLONE) {
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
