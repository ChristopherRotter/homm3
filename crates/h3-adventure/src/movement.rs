//! Movement (E08): a day's movement points (T6b); the step costs, `MoveHero` and the
//! reachable-tiles query come with T7.

use h3_content::Ruleset;
use h3_contracts::ArmyStack;

use crate::ruleset::AdventureRuleset;

/// A day's movement points on land for an army (ADV-MOV-001): the land table's entry for the
/// lowest speed among its creatures (the original, 0x4E4990, starts from speed 20 and takes the
/// slowest of the seven slots; the table's last entry stands for every faster speed). An army
/// without stacks, or a creature the core lacks, counts as fastest; a validated army has neither.
/// Bonuses (Logistics, artifacts, a specialty's speed) come with the adventure profile.
// rule: ADV-MOV-001
pub fn day_points(core: &Ruleset, part: &AdventureRuleset, army: &[ArmyStack]) -> u32 {
    let slowest = army
        .iter()
        .map(|stack| core.creature(stack.creature).map_or(u8::MAX, |c| c.speed))
        .min()
        .unwrap_or(u8::MAX);
    part.params().land_points(slowest)
}

#[cfg(test)]
mod tests;
