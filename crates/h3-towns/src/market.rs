//! The market at a town (TWN-MKT-001, -002; E09 T58a1, the town epic's design note §103, §104;
//! survey `E09-trades.md` §2): the trader's count of Marketplaces and what a town's market window
//! offers. The window is E08's market decision, which the game opens with the town as its place
//! (`h3_game`'s `town/market.rs`, ADR-0028 point 6); its rates, checks and the Artifact Merchants'
//! stock are the adventure's.

use h3_contracts::TownId;

use crate::building::BuildingSet;
use crate::command::TownCommandError;
use crate::record::Town;
use crate::ruleset::TownRuleset;
use crate::store::Towns;

/// What a town's market window offers beside its trades and its transfer, which it offers
/// whenever it opens (TWN-MKT-002; design §104, survey §2.2): the screens a hero needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TownMarket {
    /// The Artifact Merchants' purchase: a building of role `artifact_merchants` stands and the
    /// window has a hero.
    pub merchants: bool,
    /// The Freelancer's Guild's creature sale: a building of role `freelancers` stands and the
    /// window has a hero.
    pub freelancers: bool,
}

/// The number of `towns` with a building of role `marketplace` ever built (TWN-MKT-001; design
/// §103, survey §2.1, D-70, TD-29: the original's `player_marketplace_count` over the trader's
/// towns): the game passes the trader's own towns, whoever owns the town traded at, and clamps the
/// count to the market parameters' rows (at most their counts, at least 1: E09 decision 203,
/// `TWN-QUIRK-17`). A town the store does not have counts none.
// rule: TWN-MKT-001
pub fn marketplaces(
    store: &Towns,
    part: &TownRuleset,
    towns: impl IntoIterator<Item = TownId>,
) -> usize {
    // E09 T58a2: the record's derived roles, no walk over a town's buildings (the listing's
    // count, `bench-check`'s `legal-towns-36`).
    towns
        .into_iter()
        .filter_map(|town| store.get(town))
        .filter(|town| {
            part.town_with_market_roles(town.faction())
                .is_some_and(|(_, roles)| town.built().intersects(roles.marketplace))
        })
        .count()
}

/// What `town`'s market window offers, `hero` whether the window has a hero (the game's call:
/// the visitor, with `TWN-QUIRK-15`'s flag off the garrison hero too; ADR-0028 point 6)
/// (TWN-MKT-002; design §104, survey §2.2): the window opens where a building of role
/// `marketplace` stands, with or without a hero, or where one of role `artifact_merchants` or
/// `freelancers` stands and the window has a hero; it offers the trades and the transfer whenever
/// it opens, the purchase and the creature sale with a hero where their building stands.
///
/// # Errors
///
/// [`TownCommandError::NoMarket`] where none of the three stands; [`TownCommandError::NoVisitor`]
/// where only the merchants or the guild stand and the window has no hero (the original's
/// GENRLTXT 274, batch TWN-6's TWN-6-5).
// rule: TWN-MKT-002
pub fn check_market(
    town: &Town,
    part: &TownRuleset,
    hero: bool,
) -> Result<TownMarket, TownCommandError> {
    // E09 T58a2: the record's derived roles, the three read standing through the built buildings
    // that can supersede one (the listing asks it of every owned town: `bench-check`'s
    // `legal-towns-36`, as `guild_offers`).
    let Some((record, roles)) = part.town_with_market_roles(town.faction()) else {
        return Err(TownCommandError::NoMarket);
    };
    let built = town.built();
    let standing = built.intersection(roles.any());
    if standing.is_empty() {
        return Err(TownCommandError::NoMarket);
    }
    let superseded = built
        .intersection(roles.superseding)
        .iter()
        .filter_map(|id| record.building(id))
        .fold(BuildingSet::EMPTY, |set, b| set.union(b.supersedes));
    let standing = standing.difference(superseded);
    let [marketplace, merchants, freelancers] =
        [roles.marketplace, roles.merchants, roles.freelancers].map(|set| standing.intersects(set));
    if !(marketplace || merchants || freelancers) {
        return Err(TownCommandError::NoMarket);
    }
    if !(marketplace || hero) {
        return Err(TownCommandError::NoVisitor);
    }
    Ok(TownMarket {
        merchants: merchants && hero,
        freelancers: freelancers && hero,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::*;
    use crate::record::TownStatic;

    /// The Keep with its Mystic Pond a Marketplace, its Lookout Tower the Artifact Merchants and
    /// its Fountain of Fortune the Freelancer's Guild (the fixture's buildings, any three).
    fn trades_part() -> TownRuleset {
        part_with_keep(|keep| {
            keep.buildings[MYSTIC_POND].roles.marketplace = true;
            keep.buildings[LOOKOUT].roles.artifact_merchants = true;
            keep.buildings[FOUNTAIN].roles.freelancers = true;
        })
    }

    #[test]
    fn the_count_is_the_given_towns_with_a_marketplace_ever_built() {
        // covers: TWN-MKT-001
        let trades = trades_part();
        let store = Towns::new(alloc::vec![
            (town(&[MYSTIC_POND]), TownStatic::default()),
            (town(&[LOOKOUT, FOUNTAIN]), TownStatic::default()),
            (town(&[MYSTIC_POND, TAVERN]), TownStatic::default()),
            (town_of(WILD, &[MYSTIC_POND]), TownStatic::default()),
        ])
        .unwrap();
        let count =
            |towns: &[u16]| marketplaces(&store, &trades, towns.iter().copied().map(TownId::new));
        assert_eq!(count(&[0, 1, 2, 3]), 2);
        assert_eq!(count(&[1]), 0);
        assert_eq!(count(&[2, 0]), 2);
        // Ever built: a superseded Marketplace still counts (the fixture's Fountain supersedes the
        // Pond).
        let superseded = Towns::new(alloc::vec![(
            town(&[MYSTIC_POND, FOUNTAIN]),
            TownStatic::default()
        )])
        .unwrap();
        assert_eq!(marketplaces(&superseded, &trades, [TownId::new(0)]), 1);
        // A town the store lacks counts none; the part without records none either.
        assert_eq!(count(&[9]), 0);
        assert_eq!(marketplaces(&store, &part(), [TownId::new(0)]), 0);
    }

    #[test]
    fn the_window_opens_at_a_marketplace_or_at_a_trade_with_a_hero() {
        // covers: TWN-MKT-002
        let trades = trades_part();
        let window = |built: &[usize], hero| check_market(&town(built), &trades, hero);
        let offers = |merchants, freelancers| {
            Ok(TownMarket {
                merchants,
                freelancers,
            })
        };
        assert_eq!(window(&[MYSTIC_POND], false), offers(false, false));
        assert_eq!(window(&[MYSTIC_POND], true), offers(false, false));
        assert_eq!(window(&[MYSTIC_POND, LOOKOUT], false), offers(false, false));
        assert_eq!(window(&[MYSTIC_POND, LOOKOUT], true), offers(true, false));
        assert_eq!(window(&[LOOKOUT], true), offers(true, false));
        assert_eq!(window(&[FOUNTAIN], true), offers(false, true));
        assert_eq!(window(&[LOOKOUT, FOUNTAIN], true), offers(true, true));
        assert_eq!(window(&[LOOKOUT], false), Err(TownCommandError::NoVisitor));
        assert_eq!(window(&[FOUNTAIN], false), Err(TownCommandError::NoVisitor));
        assert_eq!(window(&[TAVERN], true), Err(TownCommandError::NoMarket));
        // A superseded Marketplace does not stand (the fixture's Fountain supersedes the Pond).
        assert_eq!(window(&[MYSTIC_POND, FOUNTAIN], true), offers(false, true));
        assert_eq!(
            window(&[MYSTIC_POND, FOUNTAIN], false),
            Err(TownCommandError::NoVisitor)
        );
        // The Pond the only market building, superseded: none stands.
        let pond = part_with_keep(|keep| keep.buildings[MYSTIC_POND].roles.marketplace = true);
        let superseded = town(&[MYSTIC_POND, FOUNTAIN]);
        assert_eq!(
            check_market(&superseded, &pond, true),
            Err(TownCommandError::NoMarket)
        );
        // A faction without a record has no market.
        let wild = town_of(WILD, &[MYSTIC_POND]);
        assert_eq!(
            check_market(&wild, &trades, true),
            Err(TownCommandError::NoMarket)
        );
        assert_eq!(
            check_market(&town(&[MYSTIC_POND]), &part(), true),
            Err(TownCommandError::NoMarket)
        );
    }
}
