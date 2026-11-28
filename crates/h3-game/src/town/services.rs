//! The services the legal listing offers at a town (E09 T59a, the town epic's design note §110,
//! §111, §115): its war machines' purchases (TWN-SMT-001, `town::smith`) and its boat
//! (TWN-SHY-001, `town::boat`), since E09 T59b1 its University's openings (TWN-UNI-001,
//! `town::university`), as `apply` accepts them, asked of the town part at once
//! ([`h3_towns::service_offers`]).

use h3_adventure::HeroOnMap;
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_towns::{BuildingSet, Town, TownCommand};

use super::market::hero_of;
use super::smith::receives;
use super::university::sells;
use crate::{GameRuleset, GameState};

/// What [`crate::legal_commands`] lists of the services at `player`'s own `town`, its record
/// `record` (the listing's; none for a town the store lacks, which lists none), in `player`'s turn
/// without a decision (TWN-SMT-001, TWN-SHY-001; E09
/// T59a, design §115): the war machines' purchases `town::smith::buy` accepts (each standing seller
/// of [`h3_towns::service_offers`], in the record's order, whose machine the town's hero, the
/// market window's, would receive and whose price `player`'s treasury holds) and whether
/// `town::boat::build` accepts its boat (a standing Shipyard, the price in the treasury, the spot
/// free); since E09 T59b1 the University's openings `town::university::open` accepts for the
/// town's visitor and its garrison hero, `player`'s, each where its University sells a slot open
/// for the hero at a price the treasury holds (`town::university::sells`). `visitor` and
/// `garrison` are the heroes the listing found (the hero on the town's entrance, any player's, and
/// its garrison hero). The cheapest test first: a hero there or the town's spot, then the town
/// part's offers (its derived roles), then the heroes and the treasury, then the walk over the
/// heroes on the map for the spot (`bench-check`'s `legal-towns-36`: the town part asked of every
/// owned town cost +2.4 % on `master`'s games).
// rule: TWN-SMT-001, TWN-SHY-001, TWN-UNI-001
pub(crate) fn listed(
    state: &GameState,
    rules: &GameRuleset,
    (town, player, record): (TownId, PlayerId, Option<&Town>),
    heroes: (Option<&HeroOnMap>, Option<&HeroOnMap>),
) -> Services {
    let none = Offered {
        machines: BuildingSet::EMPTY,
        boat: false,
        students: [None; 2],
    };
    let offered = record.map_or(none, |record| {
        offered(state, rules, (town, player, record), heroes).unwrap_or(none)
    });
    Services { town, offered }
}

/// [`listed`]'s answers: the purchases' buildings, the boat and the University's students (the
/// visitor's opening, then the garrison hero's).
#[derive(Debug, Clone, Copy)]
struct Offered {
    machines: BuildingSet,
    boat: bool,
    students: [Option<HeroId>; 2],
}

/// [`listed`]'s answers of a town the store holds, none where neither a hero stands there nor
/// the town has a spot.
fn offered(
    state: &GameState,
    rules: &GameRuleset,
    (town, player, record): (TownId, PlayerId, &Town),
    (visitor, garrison): (Option<&HeroOnMap>, Option<&HeroOnMap>),
) -> Option<Offered> {
    // Machines and the University need a hero in the town; a boat needs a spot: a town without
    // one stands no Shipyard (TWN-SET-011), and its boat would be refused (`BoatSpotTaken`).
    // Neither: the town part is not asked (most towns of a listing).
    let spot = state.adventure().map().town_boat_spot(town);
    if visitor.is_none() && garrison.is_none() && spot.is_none() {
        return None;
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
    // The University's students: the town's visitor and garrison hero, each `player`'s.
    let students = offers.university.map_or([None; 2], |sale| {
        [visitor, garrison].map(|on| {
            on.filter(|on| on.owner == player)
                .map(|on| on.id)
                .filter(|&hero| sells(state, rules, (player, hero), sale))
        })
    });
    Some(Offered {
        machines,
        boat,
        students,
    })
}

/// The services [`listed`] lists at a town, as town commands: each war machine's purchase in the
/// record's order, then the boat (E09 T59a), then the University's openings, the visitor's first
/// (E09 T59b1). One small iterator, so that the listing's chain of a town carries a set, a flag
/// and two heroes (`bench-check`'s `legal-towns-36`).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Services {
    town: TownId,
    offered: Offered,
}

impl Iterator for Services {
    type Item = TownCommand;

    fn next(&mut self) -> Option<TownCommand> {
        let (town, offered) = (self.town, &mut self.offered);
        if let Some(building) = offered.machines.iter().next() {
            offered.machines = offered
                .machines
                .difference(BuildingSet::EMPTY.with(building));
            return Some(TownCommand::BuyWarMachine { town, building });
        }
        // The boat once, after the machines.
        if core::mem::take(&mut offered.boat) {
            return Some(TownCommand::BuildBoat { town });
        }
        // Each student once, the visitor first.
        offered
            .students
            .iter_mut()
            .find_map(Option::take)
            .map(|hero| TownCommand::University { town, hero })
    }
}
