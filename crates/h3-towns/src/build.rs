//! Construction's rules (E09 increment 2, design §27): pure functions of a town, the part and
//! plain inputs, beside increment 1's derivations. So far what a build does ([`built_with`], E09
//! T17a); the hall's tests and the town events' buildings come with E09 T17b.

use crate::building::{BuildingId, BuildingSet};
use crate::record::Town;
use crate::ruleset::TownRuleset;
use crate::setup::{closed, upgrade_hordes};

/// The ever-built set of `town` after `building` is built (the original's `town_build` 0x5BE930
/// with the every-built set recomputed, `town_build_effects` 0x5BEDE0 steps 1 and 3; E09
/// construction survey §4, towns survey §16.2): its ever-built set with the building and every
/// building it supersedes, then each building whose non-empty `auto_with` is all ever built, with
/// what it supersedes, to a fixed point (a horde and its level's upgraded dwelling make the
/// upgraded horde, whichever comes first). No other test: the forbidden set, the requirements,
/// the Capitol and the day's mark are the callers' (the hall's tests or none for a town event).
/// `None` when the part has no record for the town's faction or the record lacks the building.
///
/// The original's hall companions (Rampart, Necropolis and Conflux's 23, 27 to 29 with the
/// halls) are display pieces the engine never has (E09 survey §16.3, design §5).
// rule: TWN-BLD-005
pub fn built_with(town: &Town, part: &TownRuleset, building: BuildingId) -> Option<BuildingSet> {
    let record = part.town(town.faction())?;
    record.building(building)?;
    let built = town.built().union(closed(record, building));
    Some(upgrade_hordes(record, built, BuildingSet::EMPTY))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::{
        BROTHERHOOD, CAPITOL, CITY_HALL, DWELLING_1, HORDE_1, MARSH, SHRINE, TAVERN, TOWN_HALL,
        UPG_DWELLING_1, UPG_HORDE_1, VILLAGE_HALL, WILD, part, set, town, town_of,
    };

    fn id(index: usize) -> BuildingId {
        BuildingId::from_index(index).unwrap()
    }

    #[test]
    fn a_build_adds_the_building_and_what_it_supersedes() {
        // covers: TWN-BLD-005
        let part = part();
        let halls = town(&[VILLAGE_HALL]);
        assert_eq!(
            built_with(&halls, &part, id(TOWN_HALL)),
            Some(set(&[VILLAGE_HALL, TOWN_HALL]))
        );
        // A building whose closure skips a step (a custom set's City Hall without the Town Hall):
        // the Capitol brings every hall below it into the ever-built set.
        let city = town(&[CITY_HALL]);
        assert_eq!(
            built_with(&city, &part, id(CAPITOL)),
            Some(set(&[VILLAGE_HALL, TOWN_HALL, CITY_HALL, CAPITOL]))
        );
        // The Brotherhood supersedes the Tavern, which joins the ever-built set unbuilt.
        assert_eq!(
            built_with(&halls, &part, id(BROTHERHOOD)),
            Some(set(&[VILLAGE_HALL, TAVERN, BROTHERHOOD]))
        );
        // Building what stands changes nothing.
        assert_eq!(
            built_with(&halls, &part, id(VILLAGE_HALL)),
            Some(halls.built())
        );
    }

    #[test]
    fn the_automatic_buildings_follow_to_a_fixed_point() {
        // covers: TWN-BLD-005
        let part = part();
        let upgraded = set(&[DWELLING_1, UPG_DWELLING_1, HORDE_1, UPG_HORDE_1, SHRINE]);
        // The horde after the upgraded dwelling, or the upgraded dwelling after the horde: the
        // upgraded horde, then the fixture's shrine (two steps), whichever comes first.
        let dwelling_first = town(&[DWELLING_1, UPG_DWELLING_1]);
        assert_eq!(
            built_with(&dwelling_first, &part, id(HORDE_1)),
            Some(upgraded)
        );
        let horde_first = town(&[DWELLING_1, HORDE_1]);
        assert_eq!(
            built_with(&horde_first, &part, id(UPG_DWELLING_1)),
            Some(upgraded)
        );
        // The horde alone makes nothing automatic.
        assert_eq!(
            built_with(&town(&[DWELLING_1]), &part, id(HORDE_1)),
            Some(set(&[DWELLING_1, HORDE_1]))
        );
        // No forbidden test: a town event's horde in a town that forbids it (and so the upgraded
        // horde and the shrine) still upgrades, as the original's `town_build`.
        let forbidding = Town::new(
            dwelling_first.faction(),
            dwelling_first.built(),
            set(&[HORDE_1, UPG_HORDE_1, SHRINE]),
        );
        assert_eq!(built_with(&forbidding, &part, id(HORDE_1)), Some(upgraded));
    }

    #[test]
    fn a_build_outside_the_record_has_no_outcome() {
        let part = part();
        // A building the Keep's record lacks; a faction without a record; the Marsh's own.
        assert_eq!(built_with(&town(&[VILLAGE_HALL]), &part, id(40)), None);
        assert_eq!(built_with(&town_of(WILD, &[]), &part, id(0)), None);
        assert_eq!(
            built_with(&town_of(MARSH, &[0]), &part, id(1)),
            Some(set(&[0, 1]))
        );
    }
}
