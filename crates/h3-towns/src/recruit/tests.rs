//! Tests of recruitment at a town (E09 T27) on the fixture's Keep: level 1's creature 10 and its
//! upgrade 11 (the dwelling 17, its upgrade 18, the horde 20 and the upgraded horde 21), level 2's
//! 20 (the dwelling 19), level 3's 30 without an upgrade, the Shrine 22 its Portal; the Marsh's
//! one level, 50.

extern crate std;

use std::string::ToString;
use std::vec;
use std::vec::Vec;

use super::*;
use crate::fixture::{
    DWELLING_1, DWELLING_2, HORDE_1, MARSH, SHRINE, TAVERN, UPG_DWELLING_1, UPG_HORDE_1, WILD,
    creature, empty_part, part, part_with_keep, town, town_of,
};
use crate::record::Summoned;

/// The Keep's building at `index`.
fn b(index: usize) -> BuildingId {
    BuildingId::from_index(index).unwrap()
}

/// `town` with `count` available at each of `indexes`.
fn with_counts(mut town: Town, indexes: &[usize], count: u16) -> Town {
    for &index in indexes {
        town = town.with_available(index, count);
    }
    town
}

/// A Portal holding `count` of creature `raw`.
fn held(raw: u16, count: u16) -> Option<Summoned> {
    Some(Summoned {
        creature: creature(raw),
        count,
    })
}

#[test]
fn a_standing_dwelling_or_horde_sells_its_index() {
    // covers: TWN-REC-001
    let cases = [
        (vec![DWELLING_1], DWELLING_1, 0),
        (vec![DWELLING_1, DWELLING_2], DWELLING_2, 1),
        (vec![DWELLING_1, UPG_DWELLING_1], UPG_DWELLING_1, 7),
        // The horde supersedes its dwelling and sells the dwelling's index.
        (vec![DWELLING_1, HORDE_1], HORDE_1, 0),
        (
            vec![DWELLING_1, HORDE_1, UPG_DWELLING_1, UPG_HORDE_1],
            UPG_HORDE_1,
            7,
        ),
    ];
    for (built, building, index) in cases {
        assert_eq!(
            recruit_source(&town(&built), &part(), b(building)),
            Ok(RecruitSource::Index(index)),
            "{built:?} {building}"
        );
    }
}

#[test]
fn a_building_that_sells_nothing_is_refused() {
    // covers: TWN-REC-001
    let keep = town(&[DWELLING_1, TAVERN, UPG_DWELLING_1, HORDE_1]);
    let refused = [
        (b(TAVERN), TownCommandError::NotRecruiting),
        (b(63), TownCommandError::UnknownBuilding),
        // Never built, or superseded by its upgrade.
        (b(DWELLING_2), TownCommandError::NotStanding),
        (b(DWELLING_1), TownCommandError::NotStanding),
        // A base horde standing beside the upgraded dwelling: its count was moved.
        (b(HORDE_1), TownCommandError::NotStanding),
    ];
    for (building, error) in refused {
        assert_eq!(recruit_source(&keep, &part(), building), Err(error));
    }
    assert_eq!(
        recruit_source(&keep, &part(), b(UPG_DWELLING_1)),
        Ok(RecruitSource::Index(7))
    );
    // A town without a record in the part has no building.
    assert_eq!(
        recruit_source(&keep, &empty_part(), b(DWELLING_1)),
        Err(TownCommandError::UnknownBuilding)
    );
}

#[test]
fn the_recruiting_buildings_are_those_the_source_accepts() {
    // covers: TWN-REC-001, TWN-REC-002
    let towns = [
        town(&[
            DWELLING_1,
            TAVERN,
            UPG_DWELLING_1,
            HORDE_1,
            DWELLING_2,
            SHRINE,
        ]),
        town(&[DWELLING_1, HORDE_1, SHRINE]).with_portal(held(50, 1)),
        town(&[DWELLING_1, HORDE_1, UPG_DWELLING_1, UPG_HORDE_1]),
        town(&[]),
    ];
    for keep in towns {
        let sells: Vec<usize> = recruiting(&keep, &part())
            .iter()
            .map(BuildingId::index)
            .collect();
        let accepted: Vec<usize> = (0..64)
            .filter(|&i| recruit_source(&keep, &part(), b(i)).is_ok())
            .collect();
        assert_eq!(sells, accepted, "{keep:?}");
    }
    assert_eq!(
        recruiting(
            &town(&[DWELLING_1, TAVERN, UPG_DWELLING_1, HORDE_1, DWELLING_2]),
            &part()
        )
        .iter()
        .map(BuildingId::index)
        .collect::<Vec<_>>(),
        vec![UPG_DWELLING_1, DWELLING_2]
    );
    assert!(recruiting(&town(&[DWELLING_1]), &empty_part()).is_empty());
}

#[test]
fn a_dwelling_of_no_level_sells_nothing() {
    // covers: TWN-REC-001
    for level in [0, 8] {
        let part = part_with_keep(|keep| {
            let dwelling = keep.buildings[DWELLING_2].roles.dwelling.as_mut().unwrap();
            dwelling.level = level;
        });
        assert_eq!(
            recruit_source(&town(&[DWELLING_1, DWELLING_2]), &part, b(DWELLING_2)),
            Err(TownCommandError::NotStanding),
            "level {level}"
        );
    }
}

#[test]
fn the_portal_sells_only_what_it_holds() {
    // covers: TWN-REC-002
    let empty = town(&[DWELLING_1, SHRINE]);
    assert_eq!(
        recruit_source(&empty, &part(), b(SHRINE)),
        Err(TownCommandError::NothingSummoned)
    );
    let full = empty.with_portal(held(50, 6));
    assert_eq!(
        recruit_source(&full, &part(), b(SHRINE)),
        Ok(RecruitSource::Portal)
    );
    assert_eq!(
        recruit_source(&town(&[DWELLING_1]), &part(), b(SHRINE)),
        Err(TownCommandError::NotStanding)
    );
    // Even an empty Portal holding a count of 0 is a source: the dialog opens.
    let zero = empty.with_portal(held(50, 0));
    assert_eq!(
        recruit_source(&zero, &part(), b(SHRINE)),
        Ok(RecruitSource::Portal)
    );
}

#[test]
fn an_upgraded_index_sells_both_creatures_from_one_count() {
    // covers: TWN-REC-001
    let keep = with_counts(town(&[DWELLING_1, UPG_DWELLING_1]), &[0, 7], 5);
    let keep = keep.with_available(7, 12);
    assert_eq!(
        recruit_stock(&keep, &part(), RecruitSource::Index(7)),
        vec![(creature(11), 12), (creature(10), 12)]
    );
    assert_eq!(
        recruit_stock(&keep, &part(), RecruitSource::Index(0)),
        vec![(creature(10), 5)]
    );
    assert_eq!(
        recruit_stock(&keep, &part(), RecruitSource::Index(1)),
        vec![(creature(20), 0)]
    );
    // Level 3 has no upgrade; the Keep has four levels; past the fourteen indexes nothing.
    for index in [9, 6, 13, 14, 20] {
        assert_eq!(
            recruit_stock(&keep, &part(), RecruitSource::Index(index)),
            vec![],
            "index {index}"
        );
    }
    // The Marsh's one level; the Wild has no record.
    let marsh = town_of(MARSH, &[]).with_available(0, 3);
    assert_eq!(
        recruit_stock(&marsh, &part(), RecruitSource::Index(0)),
        vec![(creature(50), 3)]
    );
    let wild = town_of(WILD, &[]).with_available(0, 3);
    assert_eq!(
        recruit_stock(&wild, &part(), RecruitSource::Index(0)),
        vec![]
    );
}

#[test]
fn the_portals_stock_is_its_creature() {
    // covers: TWN-REC-002
    let keep = town(&[SHRINE]);
    assert_eq!(recruit_stock(&keep, &part(), RecruitSource::Portal), vec![]);
    assert_eq!(
        recruit_stock(
            &keep.with_portal(held(20, 9)),
            &part(),
            RecruitSource::Portal
        ),
        vec![(creature(20), 9)]
    );
}

#[test]
fn a_purchase_is_of_a_creature_sold_within_its_count() {
    // covers: TWN-REC-001
    let keep = town(&[DWELLING_1, UPG_DWELLING_1]).with_available(7, 4);
    let at = RecruitSource::Index(7);
    for raw in [10, 11] {
        for amount in 1..=4 {
            assert_eq!(
                check_recruit(&keep, &part(), at, (creature(raw), amount)),
                Ok(())
            );
        }
    }
    assert_eq!(
        check_recruit(&keep, &part(), at, (creature(20), 1)),
        Err(RecruitError::NotOffered(creature(20)))
    );
    for amount in [0, 5, u32::MAX] {
        assert_eq!(
            check_recruit(&keep, &part(), at, (creature(11), amount)),
            Err(RecruitError::Amount(amount))
        );
    }
    let portal = town(&[SHRINE]).with_portal(held(50, 2));
    assert_eq!(
        check_recruit(&portal, &part(), RecruitSource::Portal, (creature(50), 2)),
        Ok(())
    );
    assert_eq!(
        check_recruit(&portal, &part(), RecruitSource::Portal, (creature(50), 3)),
        Err(RecruitError::Amount(3))
    );
}

#[test]
fn a_purchase_lowers_the_one_count_and_never_below_zero() {
    // covers: TWN-REC-001
    let keep = with_counts(town(&[DWELLING_1, UPG_DWELLING_1]), &[0, 7, 1], 10);
    let bought = recruit(&keep, RecruitSource::Index(7), 3);
    assert_eq!(bought, keep.with_available(7, 7));
    // Either creature of the index is bought from the one count; the rest of the town is kept.
    assert_eq!(
        recruit_stock(&bought, &part(), RecruitSource::Index(7)),
        vec![(creature(11), 7), (creature(10), 7)]
    );
    assert_eq!(bought.available(0), 10);
    assert_eq!(
        recruit(&keep, RecruitSource::Index(1), 70_000).available(1),
        0
    );
    assert_eq!(recruit(&keep, RecruitSource::Index(1), 11).available(1), 0);
}

#[test]
fn a_portals_purchase_keeps_its_creature() {
    // covers: TWN-REC-002
    let keep = town(&[SHRINE]).with_portal(held(50, 6));
    assert_eq!(
        recruit(&keep, RecruitSource::Portal, 6).portal(),
        held(50, 0)
    );
    assert_eq!(
        recruit(&keep, RecruitSource::Portal, 2).portal(),
        held(50, 4)
    );
    let empty = town(&[SHRINE]);
    assert_eq!(recruit(&empty, RecruitSource::Portal, 1), empty);
}

#[test]
fn every_refusal_has_a_message() {
    assert_eq!(
        RecruitError::NotOffered(creature(20)).to_string(),
        "creature 20 is not the source's"
    );
    assert_eq!(
        RecruitError::Amount(7).to_string(),
        "7 is not an amount the source holds"
    );
}
