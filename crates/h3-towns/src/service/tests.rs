//! Tests of the services' town part (E09 T59a, design §110, §111): the war machines' purchase
//! (TWN-SMT-001) and the boat (TWN-SHY-001) on the fixture's Keep.

use super::*;
use crate::fixture::*;

/// The Keep with its Tavern selling the Ballista and its Brotherhood of the Sword, which
/// supersedes the Tavern, selling the cart (the original's Stronghold sells at its Blacksmith and
/// its Ballista Yard, neither superseding the other: the supersede is the test's).
fn smith_part() -> TownRuleset {
    part_with_keep(|keep| {
        keep.buildings[TAVERN].war_machine = Some(BALLISTA);
        keep.buildings[BROTHERHOOD].war_machine = Some(CART);
    })
}

fn id(index: usize) -> BuildingId {
    BuildingId::from_index(index).unwrap()
}

#[test]
fn a_standing_smith_sells_its_war_machine_at_its_creatures_cost() {
    // covers: TWN-SMT-001
    let (core, part) = (smith_core(), smith_part());
    let tavern = town(&[TAVERN]);
    let ballista = WarMachineSale {
        artifact: BALLISTA,
        price: &[(WOOD, 2), (GOLD, 2500)],
    };
    assert_eq!(
        check_war_machine(&tavern, &part, &core, id(TAVERN)),
        Ok(ballista)
    );
    // The Brotherhood over it: the Tavern no longer stands, the Brotherhood sells the cart.
    let both = town(&[TAVERN, BROTHERHOOD]);
    assert_eq!(
        check_war_machine(&both, &part, &core, id(TAVERN)),
        Err(TownCommandError::NotStanding)
    );
    let cart = WarMachineSale {
        artifact: CART,
        price: &[(GOLD, 1000)],
    };
    assert_eq!(
        check_war_machine(&both, &part, &core, id(BROTHERHOOD)),
        Ok(cart)
    );
    // Never built.
    assert_eq!(
        check_war_machine(&tavern, &part, &core, id(BROTHERHOOD)),
        Err(TownCommandError::NotStanding)
    );
    // A building that sells none, standing or not.
    for built in [&[SILO][..], &[]] {
        assert_eq!(
            check_war_machine(&town(built), &part, &core, id(SILO)),
            Err(TownCommandError::NoWarMachine)
        );
    }
    // An artifact that is no war machine, or one the core lacks (a part of another core).
    for artifact in [RING, ArtifactTypeId::from_raw(9)] {
        let other = part_with_keep(|keep| keep.buildings[TAVERN].war_machine = Some(artifact));
        assert_eq!(
            check_war_machine(&tavern, &other, &core, id(TAVERN)),
            Err(TownCommandError::NoWarMachine)
        );
    }
    // A building the record lacks; a faction without a record.
    assert_eq!(
        check_war_machine(&tavern, &part, &core, id(40)),
        Err(TownCommandError::UnknownBuilding)
    );
    assert_eq!(
        check_war_machine(&town_of(WILD, &[TAVERN]), &part, &core, id(TAVERN)),
        Err(TownCommandError::UnknownBuilding)
    );
}

#[test]
fn the_listing_reads_the_standing_smiths_in_the_records_order() {
    // covers: TWN-SMT-001
    let part = smith_part();
    assert_eq!(
        war_machine_buildings(&town(&[TAVERN]), &part),
        set(&[TAVERN])
    );
    assert_eq!(
        war_machine_buildings(&town(&[TAVERN, BROTHERHOOD]), &part),
        set(&[BROTHERHOOD])
    );
    // Both standing where neither supersedes the other (the original's Stronghold).
    let both = part_with_keep(|keep| {
        keep.buildings[TAVERN].war_machine = Some(BALLISTA);
        keep.buildings[SILO].war_machine = Some(CART);
    });
    let built = town(&[SILO, TAVERN]);
    assert_eq!(war_machine_buildings(&built, &both), set(&[TAVERN, SILO]));
    let core = smith_core();
    for building in war_machine_buildings(&built, &both).iter() {
        assert!(check_war_machine(&built, &both, &core, building).is_ok());
    }
    assert_eq!(
        war_machine_buildings(&town(&[SILO]), &part),
        BuildingSet::EMPTY
    );
    assert_eq!(
        war_machine_buildings(&town_of(WILD, &[TAVERN]), &part),
        BuildingSet::EMPTY
    );
}

#[test]
fn the_listings_offers_are_the_two_questions_answers() {
    // covers: TWN-SMT-001, TWN-SHY-001
    let smiths = part_with_keep(|keep| {
        keep.buildings[TAVERN].war_machine = Some(BALLISTA);
        keep.buildings[BROTHERHOOD].war_machine = Some(CART);
        keep.buildings[ESCAPE_TUNNEL].supersedes = set(&[SHIPYARD]);
    });
    let parts = [smiths, part(), empty_part()];
    let built: [&[usize]; 6] = [
        &[],
        &[TAVERN],
        &[TAVERN, BROTHERHOOD, SHIPYARD],
        &[SHIPYARD, ESCAPE_TUNNEL],
        &[SHIPYARD, SILO],
        &[BROTHERHOOD],
    ];
    for part in &parts {
        for faction in [KEEP, MARSH, WILD] {
            for set in built {
                let town = town_of(faction, set);
                let offers = service_offers(&town, part);
                assert_eq!(offers.machines, war_machine_buildings(&town, part));
                assert_eq!(offers.boat, check_boat(&town, part).ok());
            }
        }
    }
}

#[test]
fn a_standing_shipyard_builds_the_parts_boat_for_its_price() {
    // covers: TWN-SHY-001
    let part = part();
    let sale = BoatSale {
        price: &[(WOOD, 10), (GOLD, 1000)],
        boat_type: 1,
    };
    assert_eq!(check_boat(&town(&[SHIPYARD]), &part), Ok(sale));
    assert_eq!(
        check_boat(&town(&[FORT]), &part),
        Err(TownCommandError::NotStanding)
    );
    assert_eq!(
        check_boat(&town_of(WILD, &[SHIPYARD]), &part),
        Err(TownCommandError::NotStanding)
    );
    assert_eq!(
        check_boat(&town(&[SHIPYARD]), &empty_part()),
        Err(TownCommandError::NotStanding)
    );
    // Superseded (no original record supersedes its Shipyard: the test's).
    let over = part_with_keep(|keep| keep.buildings[ESCAPE_TUNNEL].supersedes = set(&[SHIPYARD]));
    assert_eq!(
        check_boat(&town(&[SHIPYARD, ESCAPE_TUNNEL]), &over),
        Err(TownCommandError::NotStanding)
    );
    assert_eq!(check_boat(&town(&[SHIPYARD]), &over), Ok(sale));
}
