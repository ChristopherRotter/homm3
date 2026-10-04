//! Leaving a battle: the price of a surrender (BAT-FLE-002). Whether a side may retreat at all
//! is the heroes' business (`hero::may_retreat`, BAT-FLE-001).

use h3_content::{FULL_BP, Ruleset, SINGLE_PRECISION_RATES};
use h3_contracts::BattleSide;
use h3_kernel::arith::{SINGLE_FRAC_BITS, single_precision};

use crate::state::BattleState;

/// The price of `side`'s surrender, if it may surrender at all: it may retreat (BAT-FLE-001),
/// the enemy is led by a hero and the ruleset prices surrenders. The price is the ruleset's
/// `cost_percent` of the summed cost, in its resource, of the side's living stacks with their
/// current counts, war machines included (batch 1b R4), times one minus the hero's discount,
/// rounded down once; with quirk Q-0005 the discount enters as its single-precision value
/// (batch 1b R4: 179 for 300 at 40 %, not 180). Computed exactly; saturated at `u32::MAX`.
// rule: BAT-FLE-002
pub(crate) fn surrender_price(
    state: &BattleState,
    ruleset: &Ruleset,
    side: BattleSide,
) -> Option<u32> {
    let params = ruleset
        .battle()
        .surrender
        .filter(|_| state.may_retreat(side) && state.has_hero(side.opponent()))?;
    let unit_cost = |creature| {
        ruleset.creature_info(creature).map_or(0, |info| {
            info.cost
                .iter()
                .filter(|(resource, _)| *resource == params.resource)
                .map(|&(_, amount)| u128::from(amount))
                .sum::<u128>()
        })
    };
    // At most 38 stacks of u32 creatures at u32 costs: below 2^70.
    let cost: u128 = state
        .stacks
        .iter()
        .filter(|s| s.side == side && s.is_alive())
        .map(|s| u128::from(s.count) * unit_cost(s.creature))
        .sum();
    let discount = state.heroes[side.index()].map_or(0, |h| h.surrender_discount_bp);
    // The share kept after the discount, over `scale` (BAT-HER-001 checks discount <= 10 000).
    let (kept, scale) = if ruleset.quirks().is_enabled(SINGLE_PRECISION_RATES) {
        let one = 1u128 << SINGLE_FRAC_BITS;
        let rate = single_precision(discount, FULL_BP).map_or(0, u128::from);
        (one.saturating_sub(rate), one)
    } else {
        (
            u128::from(FULL_BP.saturating_sub(discount)),
            u128::from(FULL_BP),
        )
    };
    // Below 2^70 * 2^16 * 2^40 = 2^126: no overflow.
    let price = cost * u128::from(params.cost_percent) * kept / (100 * scale);
    Some(u32::try_from(price).unwrap_or(u32::MAX))
}
