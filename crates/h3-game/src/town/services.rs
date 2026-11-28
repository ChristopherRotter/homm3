//! The services the legal listing offers at a town (E09 T59a, the town epic's design note §110,
//! §111, §115): its war machines' purchases (TWN-SMT-001, `town::smith`) and its boat
//! (TWN-SHY-001, `town::boat`), since E09 T59b1 its University's openings (TWN-UNI-001,
//! `town::university`), since E09 T59b2 its Transformer's transformations (TWN-TRF-001,
//! `town::transform`), as `apply` accepts them, asked of the town part at once
//! ([`h3_towns::service_offers`]).

use h3_adventure::HeroOnMap;
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_towns::{BuildingSet, Town, TownArmy, TownCommand};

use super::market::hero_of;
use super::smith::receives;
use super::transform::changing;
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
/// for the hero at a price the treasury holds (`town::university::sells`); since E09 T59b2 the
/// transformations `town::transform::transform` accepts at a standing Transformer, each stack
/// that changes ([`changing`]) of `player`'s visitor's army, then of the upper bar's. `visitor` and
/// `garrison` are the heroes the listing found (the hero on the town's entrance, any player's, and
/// its garrison hero). The cheapest test first: a hero there, the town's spot or a record with a
/// Transformer ([`h3_towns::may_transform`]: its garrison needs no hero), then the town part's
/// offers (its derived roles), then the heroes and the treasury, then the walk over the heroes on
/// the map for the spot (`bench-check`'s `legal-towns-36`: the town part asked of every owned
/// town cost +2.4 % on `master`'s games).
// rule: TWN-SMT-001, TWN-SHY-001, TWN-UNI-001, TWN-TRF-001
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
        transforms: [0; 2],
    };
    let offered = record.map_or(none, |record| {
        offered(state, rules, (town, player, record), heroes).unwrap_or(none)
    });
    Services { town, offered }
}

/// [`listed`]'s answers: the purchases' buildings, the boat, the University's students (the
/// visitor's opening, then the garrison hero's) and the transformations' slots (a mask of each
/// army's, the visitor's, then the upper bar's).
#[derive(Debug, Clone, Copy)]
struct Offered {
    machines: BuildingSet,
    boat: bool,
    students: [Option<HeroId>; 2],
    transforms: [u8; 2],
}

/// [`listed`]'s answers of a town the store holds, none where neither a hero stands there, nor
/// the town has a spot, nor its record a Transformer.
fn offered(
    state: &GameState,
    rules: &GameRuleset,
    (town, player, record): (TownId, PlayerId, &Town),
    (visitor, garrison): (Option<&HeroOnMap>, Option<&HeroOnMap>),
) -> Option<Offered> {
    // Machines and the University need a hero in the town; a boat needs a spot: a town without
    // one stands no Shipyard (TWN-SET-011), and its boat would be refused (`BoatSpotTaken`).
    // Neither: the town part is not asked (most towns of a listing).
    // A transformation needs no hero: the town's garrison's (TWN-TRF-001).
    let (core, part) = (rules.core(), rules.towns());
    let spot = state.adventure().map().town_boat_spot(town);
    if visitor.is_none()
        && garrison.is_none()
        && spot.is_none()
        && !h3_towns::may_transform(record.faction(), part)
    {
        return None;
    }
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
    // The transformations: `player`'s visitor's army, then the upper bar's (TWN-TRF-001).
    let transforms = offers.transformer.map_or([0; 2], |sale| {
        let visiting = visitor.filter(|on| on.owner == player);
        let upper = garrison.map_or_else(|| state.adventure().upper_bar(town), |on| &on.army[..]);
        [
            visiting.map_or(0, |on| changing(&on.army, sale)),
            changing(upper, sale),
        ]
    });
    Some(Offered {
        machines,
        boat,
        students,
        transforms,
    })
}

/// The services [`listed`] lists at a town, as town commands: each war machine's purchase in the
/// record's order, then the boat (E09 T59a), then the University's openings, the visitor's first
/// (E09 T59b1), then the transformations, the visitor's army's in slot order, then the upper
/// bar's (E09 T59b2). One small iterator, so that the listing's chain of a town carries a set, a
/// flag, two heroes and two masks (`bench-check`'s `legal-towns-36`).
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
        if let Some(hero) = offered.students.iter_mut().find_map(Option::take) {
            return Some(TownCommand::University { town, hero });
        }
        // Each changing slot once, the visitor's army first, in slot order.
        let armies = [TownArmy::Visitor, TownArmy::Garrison];
        offered
            .transforms
            .iter_mut()
            .zip(armies)
            .find(|(mask, _)| **mask != 0)
            .map(|(mask, army)| {
                let slot = mask.trailing_zeros();
                *mask &= *mask - 1;
                // A `u8`'s lowest set bit: below 8.
                let slot = u8::try_from(slot).unwrap_or(u8::MAX);
                TownCommand::Transform { town, army, slot }
            })
    }
}
