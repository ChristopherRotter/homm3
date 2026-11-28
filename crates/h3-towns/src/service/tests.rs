//! Tests of the services' town part (E09 T59a, design §110, §111): the war machines' purchase
//! (TWN-SMT-001) and the boat (TWN-SHY-001) on the fixture's Keep; since E09 T59b1 the Magic
//! University's opening (TWN-UNI-001, design §112); since E09 T59b2 the Skeleton Transformer's
//! table (TWN-TRF-001, design §113).

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
    // covers: TWN-UNI-001, TWN-TRF-001
    let smiths = part_with_keep(|keep| {
        keep.buildings[TAVERN].war_machine = Some(BALLISTA);
        keep.buildings[BROTHERHOOD].war_machine = Some(CART);
        keep.buildings[ESCAPE_TUNNEL].supersedes = set(&[SHIPYARD]);
        keep.buildings[LOOKOUT].roles.university = true;
        keep.buildings[OBELISK].supersedes = set(&[LOOKOUT]);
        keep.buildings[FOUNTAIN].roles.transformer = true;
        keep.buildings[MYSTIC_POND].supersedes = set(&[FOUNTAIN]);
    });
    let parts = [smiths, part(), empty_part()];
    let built: [&[usize]; 10] = [
        &[],
        &[TAVERN],
        &[TAVERN, BROTHERHOOD, SHIPYARD],
        &[SHIPYARD, ESCAPE_TUNNEL],
        &[SHIPYARD, SILO],
        &[BROTHERHOOD],
        &[LOOKOUT, TAVERN],
        &[LOOKOUT, OBELISK],
        &[FOUNTAIN, SILO],
        &[FOUNTAIN, MYSTIC_POND],
    ];
    for part in &parts {
        for faction in [KEEP, MARSH, WILD] {
            for set in built {
                let town = town_of(faction, set);
                let offers = service_offers(&town, part);
                assert_eq!(offers.machines, war_machine_buildings(&town, part));
                assert_eq!(offers.boat, check_boat(&town, part).ok());
                assert_eq!(offers.university, check_university(&town, part).ok());
                assert_eq!(offers.transformer, check_transformer(&town, part).ok());
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

#[test]
fn a_standing_university_teaches_the_parts_skills_at_its_price() {
    // covers: TWN-UNI-001
    let university = part_with_keep(|keep| {
        keep.buildings[LOOKOUT].roles.university = true;
        keep.buildings[OBELISK].supersedes = set(&[LOOKOUT]);
    });
    let sale = UniversitySale {
        skills: &[SkillId::from_raw(0)],
        price: &[(GOLD, 2000)],
    };
    assert_eq!(check_university(&town(&[LOOKOUT]), &university), Ok(sale));
    // Never built; superseded (no original record supersedes its University: the test's); a
    // faction without a record; a record without the role.
    for (town, part) in [
        (town(&[FORT]), &university),
        (town(&[LOOKOUT, OBELISK]), &university),
        (town_of(WILD, &[LOOKOUT]), &university),
        (town(&[LOOKOUT]), &part()),
        (town(&[LOOKOUT]), &empty_part()),
    ] {
        assert_eq!(
            check_university(&town, part),
            Err(TownCommandError::NoUniversity)
        );
    }
}

#[test]
fn a_standing_transformer_makes_the_dragons_its_dragon_and_the_rest_its_skeleton() {
    // covers: TWN-TRF-001
    let (skeleton, bone_dragon) = (creature(50), creature(51));
    let dragons = [creature(3), creature(7)];
    let transformer = part_changed(
        |params| {
            params.transformer_into = skeleton;
            params.transformer_dragons_into = bone_dragon;
            params.transformer_dragons = dragons.to_vec();
        },
        |keep| {
            keep.buildings[FOUNTAIN].roles.transformer = true;
            keep.buildings[MYSTIC_POND].supersedes = set(&[FOUNTAIN]);
        },
    );
    let sale = check_transformer(&town(&[FOUNTAIN]), &transformer).unwrap();
    assert_eq!(
        sale,
        TransformerSale {
            into: skeleton,
            dragons_into: bone_dragon,
            dragons: &dragons,
        }
    );
    // The dragons their dragon, every other creature the skeleton, the undead too; the skeleton
    // its own target.
    for (creature, target) in [
        (dragons[0], bone_dragon),
        (dragons[1], bone_dragon),
        (creature(5), skeleton),
        (bone_dragon, skeleton),
        (skeleton, skeleton),
    ] {
        assert_eq!(sale.target(creature), target, "{creature:?}");
    }
    assert!(may_transform(KEEP, &transformer));
    // Never built; superseded (no original record supersedes its Transformer: the test's); a
    // faction without a record; a record without the role.
    for (town, part) in [
        (town(&[FORT]), &transformer),
        (town(&[FOUNTAIN, MYSTIC_POND]), &transformer),
        (town_of(WILD, &[FOUNTAIN]), &transformer),
        (town(&[FOUNTAIN]), &part()),
        (town(&[FOUNTAIN]), &empty_part()),
    ] {
        assert_eq!(
            check_transformer(&town, part),
            Err(TownCommandError::NoTransformer)
        );
    }
    for (faction, part) in [(WILD, &transformer), (KEEP, &part()), (KEEP, &empty_part())] {
        assert!(!may_transform(faction, part));
    }
}
