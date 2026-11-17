//! A town's Tavern as a hire's place (TWN-TAV-003; E09 T38a1, the town epic's design note §64):
//! the town part's half of `TownCommand::Hire`. The game checks the rest (the town's owner, the
//! player's slot and its hero, the adventure's room and entrance, the treasury), pays the price
//! and places the hero (ADR-0028 point 6, as a build's).

use h3_content::ResourceId;

use crate::command::TownCommandError;
use crate::record::Town;
use crate::ruleset::TownRuleset;
use crate::view::has_tavern;

/// The town part's test of a hire at `town` (TWN-TAV-003; `town_tavern_click` 0x5D82B0: the town
/// screen's Tavern opens the dialog): a standing building of role `tavern` ([`has_tavern`]: the
/// Tavern, or Castle's Brotherhood of the Sword over it), else [`TownCommandError::NoTavern`];
/// then the hire's price, the part's (`TownParams::hire_price`, one for every hire and edition;
/// none for a part without parameters, which has no town record to stand a Tavern in).
///
/// # Errors
///
/// [`TownCommandError::NoTavern`]; the state is the caller's and unchanged.
// rule: TWN-TAV-003
pub fn check_hire<'a>(
    town: &Town,
    part: &'a TownRuleset,
) -> Result<&'a [(ResourceId, u32)], TownCommandError> {
    if !has_tavern(town, part) {
        return Err(TownCommandError::NoTavern);
    }
    Ok(part
        .params()
        .map_or(&[], |params| params.hire_price.as_slice()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::*;

    #[test]
    fn a_hire_needs_a_standing_tavern_and_costs_the_parts_price() {
        // covers: TWN-TAV-003
        let part = part();
        let price: &[(ResourceId, u32)] = &[(GOLD, 2500)];
        assert_eq!(check_hire(&town(&[TAVERN]), &part), Ok(price));
        // The Brotherhood over it stands in its place.
        assert_eq!(check_hire(&town(&[TAVERN, BROTHERHOOD]), &part), Ok(price));
        assert_eq!(
            check_hire(&town(&[]), &part),
            Err(TownCommandError::NoTavern)
        );
    }
}
