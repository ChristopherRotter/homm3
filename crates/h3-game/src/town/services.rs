//! The services the legal listing offers at a town (E09 T59a, the town epic's design note §110,
//! §111, §115): its war machines' purchases (TWN-SMT-001, `town::smith`) and its boat
//! (TWN-SHY-001, `town::boat`), as `apply` accepts them, asked of the town part at once
//! ([`h3_towns::service_offers`]).

use h3_adventure::HeroOnMap;
use h3_contracts::{PlayerId, TownId};
use h3_towns::{BuildingSet, Town, TownCommand};

use super::market::hero_of;
use super::smith::receives;
use crate::{GameRuleset, GameState};

/// What [`crate::legal_commands`] lists of the services at `player`'s own `town`, its record
/// `record` (the listing's; none for a town the store lacks, which lists none), in `player`'s turn
/// without a decision (TWN-SMT-001, TWN-SHY-001; E09
/// T59a, design §115): the war machines' purchases `town::smith::buy` accepts (each standing seller
/// of [`h3_towns::service_offers`], in the record's order, whose machine the town's hero, the
/// market window's, would receive and whose price `player`'s treasury holds) and whether
/// `town::boat::build` accepts its boat (a standing Shipyard, the price in the treasury, the spot
/// free). `visitor` and `garrison` are the heroes the listing found (the hero on the town's
/// entrance, any player's, and its garrison hero). The cheapest test first: a hero there or the
/// town's spot, then the town part's offers (its derived roles), then the heroes and the
/// treasury, then the walk over the heroes on the map for the spot (`bench-check`'s
/// `legal-towns-36`: the town part asked of every owned town cost +2.4 % on `master`'s games).
// rule: TWN-SMT-001, TWN-SHY-001
pub(crate) fn listed(
    state: &GameState,
    rules: &GameRuleset,
    (town, player, record): (TownId, PlayerId, Option<&Town>),
    heroes: (Option<&HeroOnMap>, Option<&HeroOnMap>),
) -> Services {
    let (machines, boat) = record.map_or((BuildingSet::EMPTY, false), |record| {
        offered(state, rules, (town, player, record), heroes)
    });
    Services {
        town,
        machines,
        boat,
    }
}

/// [`listed`]'s two answers: the purchases' buildings and the boat.
fn offered(
    state: &GameState,
    rules: &GameRuleset,
    (town, player, record): (TownId, PlayerId, &Town),
    (visitor, garrison): (Option<&HeroOnMap>, Option<&HeroOnMap>),
) -> (BuildingSet, bool) {
    // Machines need a hero in the town; a boat needs a spot: a town without one stands no
    // Shipyard (TWN-SET-011), and its boat would be refused (`BoatSpotTaken`). Neither: the town
    // part is not asked (most towns of a listing).
    let spot = state.adventure().map().town_boat_spot(town);
    if visitor.is_none() && garrison.is_none() && spot.is_none() {
        return (BuildingSet::EMPTY, false);
    }
    let (core, part) = (rules.core(), rules.towns());
    let offers = h3_towns::service_offers(record, part);
    let machines = if offers.machines.is_empty() {
        BuildingSet::EMPTY
    } else {
        let heroes = [visitor, garrison].map(|on| on.map(|on| (on.id, on.owner)));
        let buyer = hero_of(rules, player, heroes).and_then(|hero| state.hero(hero));
        buyer.map_or(BuildingSet::EMPTY, |details| {
            offers
                .machines
                .iter()
                .filter(|&building| {
                    h3_towns::check_war_machine(record, part, core, building).is_ok_and(|sale| {
                        state.can_pay(player, sale.price) && receives(details, core, sale.artifact)
                    })
                })
                .fold(BuildingSet::EMPTY, BuildingSet::with)
        })
    };
    let boat = offers.boat.is_some_and(|sale| {
        state.can_pay(player, sale.price) && state.adventure().free_town_boat_spot(town).is_some()
    });
    (machines, boat)
}

/// The services [`listed`] lists at a town, as town commands: each war machine's purchase in the
/// record's order, then the boat (E09 T59a). One small iterator, so that the listing's chain of a
/// town carries two sets and a flag (`bench-check`'s `legal-towns-36`).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Services {
    town: TownId,
    machines: BuildingSet,
    boat: bool,
}

impl Iterator for Services {
    type Item = TownCommand;

    fn next(&mut self) -> Option<TownCommand> {
        let town = self.town;
        if let Some(building) = self.machines.iter().next() {
            self.machines = self.machines.difference(BuildingSet::EMPTY.with(building));
            return Some(TownCommand::BuyWarMachine { town, building });
        }
        // The boat once, after the machines.
        core::mem::take(&mut self.boat).then_some(TownCommand::BuildBoat { town })
    }
}
