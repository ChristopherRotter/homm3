//! The Artifact Merchants' stock in the game (TWN-STK-001, -002; E09 T57, the town epic's design
//! note §108, decisions 207, 212 and 214; survey `E09-trades.md` §3.1, §3.2): the game's two
//! draws of the adventure's stock (`h3_adventure::AdventureState::draw_merchants`), each of the
//! town part's `merchants_stock`, whether or not any town has the merchants: the new game's from
//! the setup stream after every other setup draw, each later month's from the towns' stream at the
//! month's start's end. Neither tells anything: the stock is seen only in its window (E09 T58).

use h3_content::ArtifactClass;
use h3_kernel::Rng;

use crate::{GameRuleset, GameState};

/// The new game's draw (TWN-STK-001): the stock drawn from the setup stream `draws` last, after
/// the Portals of Summoning (`super::portal::at_setup`; the original's `new_game_heroes`
/// 0x4BFE70 ends with it), so no earlier setup draw moves.
// rule: TWN-STK-001
pub(crate) fn at_setup(
    adventure: &mut h3_adventure::AdventureState,
    rules: &GameRuleset,
    draws: &mut Rng,
) {
    adventure.draw_merchants(rules.core(), classes(rules), draws);
}

/// A month's draw (TWN-STK-002; GAM-CAL-002): on the first day of every month from the second,
/// after the month's kind, the towns' month and a creature month's adventure step, before the
/// towns' day (the original's `new_month` 0x4C8F40 ends with it), the whole stock drawn anew
/// from the towns' stream (decision 214), sold or not, the old entries' marks kept.
// rule: TWN-STK-002
pub(crate) fn monthly(state: &mut GameState, rules: &GameRuleset) {
    let (adventure, _, draws) = state.town_draws(super::Draws::Towns);
    adventure.draw_merchants(rules.core(), classes(rules), draws);
}

/// The stock's classes: the town part's `merchants_stock`, none without its parameters (a game
/// without towns: an empty stock, nothing drawn).
fn classes(rules: &GameRuleset) -> &[ArtifactClass] {
    rules
        .towns()
        .params()
        .map_or(&[][..], |params| params.merchants_stock.as_slice())
}
