//! Damage (BAT-DMG-001..003) and the Defend bonus (BAT-TRN-004).

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

/// Applies the attack/defense difference: +`per_point`% per point of attack over defense (capped),
/// or −`per_point`‰ per point of defense over attack (capped); result rounded down, at least 1.
// rule: BAT-DMG-002
pub(crate) fn modified_damage(base: u64, attack: u16, defense: u16, params: &BattleParams) -> u64 {
    let damage = if attack > defense {
        let diff = u64::from(attack - defense);
        let bonus = (diff * u64::from(params.attack_bonus_per_point_percent))
            .min(u64::from(params.attack_bonus_cap_percent));
        mul_div_floor(base, 100 + bonus, 100)
    } else {
        let diff = u64::from(defense - attack);
        let reduction = (diff * u64::from(params.defense_reduction_per_point_permille))
            .min(u64::from(params.defense_reduction_cap_permille));
        mul_div_floor(base, 1000 - reduction, 1000)
    };
    damage.unwrap_or(u64::MAX).max(1)
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

/// One strike of `attacker` on `target` (BAT-DMG-001..003). Returns `(damage, kills)`.
pub(crate) fn strike(
    rng: &mut Rng,
    attacker: &Stack,
    target: &mut Stack,
    params: &BattleParams,
) -> (u64, u32) {
    let base = base_damage(
        rng,
        attacker.count,
        attacker.damage_min,
        attacker.damage_max,
        params.damage_roll_samples,
    );
    let damage = modified_damage(
        base,
        attacker.attack,
        effective_defense(target, params),
        params,
    );
    let kills = apply_damage(target, damage);
    (damage, kills)
}
