//! Damage (BAT-DMG-001..003, BAT-DMG-006/008, BAT-RNG-002/003) and the Defend bonus (BAT-TRN-004).
//!
//! A strike's damage is its base damage (BAT-DMG-001) times the sum of its bonuses and the product
//! of its reductions, computed exactly with one final rounding down (BAT-DMG-002):
//! `floor(base · (100 + Σ bonus%) / 100 · Π (1000 − reduction‰) / 1000)`, at least 1.

use h3_content::BattleParams;
use h3_kernel::Rng;
use h3_kernel::arith::mul_div_floor;

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

/// The bonuses (percent, summed) and reductions (per mille, multiplied) of one strike.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Factors {
    bonus_percent: u32,
    reductions: [u16; MAX_REDUCTIONS],
    len: usize,
}

impl Factors {
    /// Adds a bonus in percent of the base damage.
    pub(crate) fn bonus(&mut self, percent: u32) {
        self.bonus_percent = self.bonus_percent.saturating_add(percent);
    }

    /// Adds a reduction in per mille (`0..=1000`). Strikes have fewer than `MAX_REDUCTIONS`
    /// sources of reduction by construction; the call sites are the rules below.
    pub(crate) fn reduce(&mut self, permille: u16) {
        if let Some(slot) = self.reductions.get_mut(self.len) {
            *slot = permille.min(1000);
            self.len += 1;
        }
    }

    /// The damage of `base` with these factors: exact, rounded down once, at least 1
    /// (BAT-DMG-002). Saturates on absurd inputs.
    // rule: BAT-DMG-002
    pub(crate) fn apply(&self, base: u64) -> u64 {
        let mut num = u128::from(base) * (100 + u128::from(self.bonus_percent));
        let mut den: u128 = 100;
        for r in &self.reductions[..self.len] {
            num = num.saturating_mul(1000 - u128::from(*r));
            den *= 1000;
        }
        u64::try_from(num / den).unwrap_or(u64::MAX).max(1)
    }
}

/// The attack/defense part of the factors: +`per_point`% per point of attack over defense
/// (capped) or −`per_point`‰ per point of defense over attack (capped).
// rule: BAT-DMG-002
pub(crate) fn attack_defense(
    factors: &mut Factors,
    attack: u16,
    defense: u16,
    params: &BattleParams,
) {
    if attack > defense {
        let diff = u32::from(attack - defense);
        factors.bonus(
            (diff * u32::from(params.attack_bonus_per_point_percent))
                .min(u32::from(params.attack_bonus_cap_percent)),
        );
    } else {
        let diff = u32::from(defense - attack);
        let reduction = (diff * u32::from(params.defense_reduction_per_point_permille))
            .min(u32::from(params.defense_reduction_cap_permille));
        factors.reduce(u16::try_from(reduction).unwrap_or(1000));
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
