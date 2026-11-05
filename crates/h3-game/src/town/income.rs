//! The towns' part of the day's income as the game composes it (TWN-INC-001, TWN-INC-002; E09
//! T8c1, the town epic's design note §16, ADR-0017's amendment): each town a player owns pays it
//! `h3_towns::daily_income` beside its mines and heroes, in the day's advance (E08 T85b's,
//! GAM-RES-004).

use h3_adventure::AdventureState;
use h3_contracts::{PlayerId, TownId};
use h3_towns::{TownRuleset, Towns};

/// Adds the day's income of every town `player` owns (the adventure's owners, by `TownId`) to
/// `income`, its amounts by resource in the core's order, saturating: each town's
/// [`h3_towns::daily_income`] (its hall's gold, a Grail building's, its silo's resources). A
/// neutral town pays nobody; a town captured today pays its new owner from the next day's start,
/// as a mine flagged today; several Capitols each pay theirs (the limit comes with a capture,
/// TWN-CAP-002). A town the store lacks pays nothing (the store holds every town of the map).
// rule: TWN-INC-001, TWN-INC-002
pub(crate) fn add_income(
    adventure: &AdventureState,
    towns: &Towns,
    part: &TownRuleset,
    player: PlayerId,
    income: &mut [u32],
) {
    let owned = adventure
        .map()
        .towns()
        .iter()
        .zip((0..=u16::MAX).map(TownId::new))
        .filter(|&(&index, _)| adventure.owner(index) == Some(player))
        .filter_map(|(_, town)| towns.get(town));
    for town in owned {
        for (resource, amount) in h3_towns::daily_income(town, part) {
            if let Some(entry) = income.get_mut(resource.index()) {
                *entry = entry.saturating_add(amount);
            }
        }
    }
}
