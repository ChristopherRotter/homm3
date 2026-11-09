//! Construction's rules (E09 increment 2, design §27): pure functions of a town, the part and
//! plain inputs, beside increment 1's derivations: what a build does ([`built_with`], E09 T17a),
//! the hall's offer and tests ([`buildable`], [`possible`], [`check_build`]) and a town event's
//! buildings ([`event_buildings`]; E09 T17b). The game gives the inputs no town holds: whether the
//! town's owner holds a Capitol in another town (`holds_capitol`, as the capture's limit) and
//! whether the town has water (its boat spot, the adventure's).

use h3_content::ResourceId;

use crate::building::{BuildingId, BuildingSet};
use crate::command::TownCommandError;
use crate::record::{Town, TownEvent};
use crate::ruleset::{Building, TownRecord, TownRuleset};
use crate::setup::{closed, indexed, upgrade_hordes};

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

/// The hall's offer of `town`: every building of its faction's record that [`check_build`]
/// allows, in index order (the original's buildable-now set, `town_buildable_now` 0x5C0F20, as
/// the hall shows it, `hall_dialog_fill` 0x461190; E09 survey `E09-construction.md` §1.2, §1.3,
/// §2.1). `holds_capitol`: the town's owner holds another town with a standing Capitol. Empty
/// while the town was built in today, and for a faction without a record.
///
/// A horde whose level's upgraded dwelling was ever built is offered as its upgraded horde alone
/// (the hall's horde slot shows the upgrade then, at the same cost, and buying either gives the
/// same ever-built set; E09 design §27, the agent's call): one command names one outcome.
// rule: TWN-BLD-002, TWN-BLD-003, TWN-BLD-004
pub fn buildable(town: &Town, part: &TownRuleset, holds_capitol: bool) -> BuildingSet {
    let Some(record) = part.town(town.faction()) else {
        return BuildingSet::EMPTY;
    };
    if town.built_today() {
        return BuildingSet::EMPTY;
    }
    indexed(record)
        .filter(|&(id, building)| refusal(town, record, id, building, holds_capitol).is_none())
        .fold(BuildingSet::EMPTY, |set, (id, _)| set.with(id))
}

/// The buildings `town`'s hall may still build some day, ever built or not (the original's
/// `town_build_possible` 0x5C0E60, survey §2.2: the hall's "not now" against "never", for clients
/// over the owner's view): each building of its faction's record that is not forbidden, not a
/// Capitol while `holds_capitol`, and whose every requirement is ever built or not forbidden (a
/// building one of whose requirements is forbidden and unbuilt is never buildable). It reads
/// neither the day's mark nor whether the building was built; the Grail's building is the
/// original's too (no hall slot shows it); the Shipyard without water is forbidden from the start
/// (TWN-SET-011). Empty for a faction without a record. Holds [`buildable`]'s set.
// rule: TWN-BLD-002
pub fn possible(town: &Town, part: &TownRuleset, holds_capitol: bool) -> BuildingSet {
    let Some(record) = part.town(town.faction()) else {
        return BuildingSet::EMPTY;
    };
    let never = town.forbidden().difference(town.built());
    indexed(record)
        .filter(|&(id, building)| {
            !town.forbidden().contains(id)
                && !(holds_capitol && is_capitol(building))
                && building.requires.difference(never) == building.requires
        })
        .fold(BuildingSet::EMPTY, |set, (id, _)| set.with(id))
}

/// The town's part of a build of `building` in `town` (TWN-BLD-001; E09 design §27): its cost
/// in the core's resource order (`Building.txt`'s, TWN-BLD-006; empty for a free building), or
/// the first refusal in [`TownCommandError`]'s order: [`TownCommandError::UnknownBuilding`] (not
/// of the faction's record), `Built` (ever built), `Forbidden`, `Grail` (never by the hall),
/// `Requirements` (not every requirement ever built, TWN-BLD-007), `Capitol` (a hall-4 building
/// while `holds_capitol`), `OfferedAs` (a horde the hall offers as its upgraded horde,
/// TWN-BLD-003), `BuiltToday` (the day's mark, TWN-BLD-004). `Ok` exactly for the buildings of
/// [`buildable`]. The game checks the town, its owner and the treasury (E09 design §29).
///
/// # Errors
///
/// The first refusal, as above; the state is the caller's and unchanged.
// rule: TWN-BLD-001, TWN-BLD-002, TWN-BLD-003, TWN-BLD-004, TWN-BLD-006
pub fn check_build<'a>(
    town: &Town,
    part: &'a TownRuleset,
    building: BuildingId,
    holds_capitol: bool,
) -> Result<&'a [(ResourceId, u32)], TownCommandError> {
    let record = part
        .town(town.faction())
        .ok_or(TownCommandError::UnknownBuilding)?;
    let def = record
        .building(building)
        .ok_or(TownCommandError::UnknownBuilding)?;
    refusal(town, record, building, def, holds_capitol).map_or(Ok(def.cost.as_slice()), Err)
}

/// The first of the hall's refusals of `building` (its `id` in `record`, the town's faction's) in
/// `town`, if any (E09 design §27's order: the hall's texts', the day's mark last).
fn refusal(
    town: &Town,
    record: &TownRecord,
    id: BuildingId,
    building: &Building,
    holds_capitol: bool,
) -> Option<TownCommandError> {
    let built = town.built();
    if built.contains(id) {
        Some(TownCommandError::Built)
    } else if town.forbidden().contains(id) {
        Some(TownCommandError::Forbidden)
    } else if building.roles.grail {
        Some(TownCommandError::Grail)
    } else if !building.requires.difference(built).is_empty() {
        Some(TownCommandError::Requirements)
    } else if holds_capitol && is_capitol(building) {
        Some(TownCommandError::Capitol)
    } else if let Some(upgrade) = offered_as(record, built, id) {
        Some(TownCommandError::OfferedAs(upgrade))
    } else if town.built_today() {
        Some(TownCommandError::BuiltToday)
    } else {
        None
    }
}

/// Whether `building` is a Capitol, a hall-4 building (one a player, the original's
/// `player_has_capitol` 0x4B9F40, survey §2.4).
fn is_capitol(building: &Building) -> bool {
    building.roles.hall == Some(4)
}

/// The upgraded horde the hall offers in the place of `horde` with the ever-built set `built`:
/// the record's first building, in index order, that a build of `horde` would make automatic and
/// that supersedes it (`horde` in its `auto_with`, the rest of its `auto_with` ever built, itself
/// not), so that building it leaves the same ever-built set (TWN-BLD-003; survey §1.2, §6: the
/// horde's slot shows the upgrade once the level's upgraded dwelling stands). A building that is
/// automatic with `horde` but does not supersede it is no such upgrade.
fn offered_as(record: &TownRecord, built: BuildingSet, horde: BuildingId) -> Option<BuildingId> {
    indexed(record)
        .find(|&(id, upgrade)| {
            upgrade.auto_with.contains(horde)
                && upgrade.supersedes.contains(horde)
                && upgrade.auto_with.difference(built.with(horde)).is_empty()
                && !built.contains(id)
        })
        .map(|(id, _)| id)
}

/// The buildings a town event of `town` builds, in index order (the original's id order;
/// `town_events_apply` 0x5BFEB0, survey §8.3; E09 decision 44): `event`'s buildings of the town's
/// faction's record less those ever built, less what another of its buildings supersedes (an
/// event with the Fort and the Castle builds the Castle), less a building of role `shipyard`
/// without `water` (the town's boat spot, the game's input). No other test: not the requirements,
/// the forbidden set, the Capitol (T-8: an event's Capitol beside its owner's other one stands),
/// the Grail, the cost nor the day's mark; the game builds each with [`crate::Towns::build`]
/// without the mark. Empty for a faction without a record. The map's bits past its 41 slots name
/// no building (TWN-QUIRK-7: the original's loop reads 44, past its table).
// rule: TWN-EVT-003
pub fn event_buildings(
    town: &Town,
    part: &TownRuleset,
    event: &TownEvent,
    water: bool,
) -> BuildingSet {
    let Some(record) = part.town(town.faction()) else {
        return BuildingSet::EMPTY;
    };
    let own = || {
        event
            .buildings
            .iter()
            .filter_map(|id| record.building(id).map(|building| (id, building)))
    };
    let superseded = own().fold(BuildingSet::EMPTY, |set, (_, building)| {
        set.union(building.supersedes)
    });
    own()
        .filter(|&(id, building)| {
            !town.built().contains(id)
                && !superseded.contains(id)
                && (water || !building.roles.shipyard)
        })
        .fold(BuildingSet::EMPTY, |set, (id, _)| set.with(id))
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::vec::Vec;

    use super::*;
    use crate::building::MAX_BUILDINGS;
    use crate::fixture::*;
    use crate::setup::SlotSet;

    fn id(index: usize) -> BuildingId {
        BuildingId::from_index(index).unwrap()
    }

    /// `town` built in today.
    fn marked(town: &Town) -> Town {
        town.rebuilt(town.built(), true)
    }

    /// A Keep town with `built` ever built and `forbidden` forbidden.
    fn keep_town(built: &[usize], forbidden: &[usize]) -> Town {
        Town::new(KEEP, set(built), set(forbidden))
    }

    /// The Keep's buildings the hall offers a town with the Village Hall, the Fort and the
    /// Tavern: what requires the two or nothing, less what stands and the Grail.
    const FORT_AND_TAVERN_OFFER: [usize; 11] = [
        TOWN_HALL,
        CITADEL,
        BROTHERHOOD,
        SILO,
        LOOKOUT,
        MYSTIC_POND,
        ESCAPE_TUNNEL,
        OBELISK,
        GLYPHS,
        DWELLING_1,
        SHIPYARD,
    ];

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

    #[test]
    fn the_hall_offers_what_the_ever_built_set_allows() {
        // covers: TWN-BLD-002
        let part = part();
        // At the start: what requires nothing, less the Village Hall that stands and the Grail.
        assert_eq!(
            buildable(&town(&[VILLAGE_HALL]), &part, false),
            set(&[FORT, TAVERN, MYSTIC_POND, SHIPYARD])
        );
        assert_eq!(
            buildable(&town(&[VILLAGE_HALL, FORT, TAVERN]), &part, false),
            set(&FORT_AND_TAVERN_OFFER)
        );
        // A requirement holds while it is ever built: a Castle's town keeps its Fort for the
        // Lookout and the dwellings.
        let castle = buildable(&town(&[VILLAGE_HALL, FORT, CITADEL, CASTLE]), &part, false);
        assert!(castle.contains(id(LOOKOUT)) && castle.contains(id(DWELLING_1)));
        assert!(!castle.contains(id(CASTLE)) && !castle.contains(id(FORT)));
        // The forbidden are never offered.
        let forbidding = keep_town(&[VILLAGE_HALL, FORT, TAVERN], &[TOWN_HALL, SHIPYARD]);
        assert_eq!(
            buildable(&forbidding, &part, false),
            set(&FORT_AND_TAVERN_OFFER).difference(set(&[TOWN_HALL, SHIPYARD]))
        );
        // The Marsh's free buildings require nothing; a faction without a record has no hall.
        assert_eq!(buildable(&town_of(MARSH, &[0]), &part, false), set(&[1, 2]));
        assert!(buildable(&town_of(WILD, &[]), &part, false).is_empty());
        assert_eq!(
            check_build(&town_of(WILD, &[]), &part, id(0), false),
            Err(TownCommandError::UnknownBuilding)
        );
    }

    #[test]
    fn the_hall_offers_a_horde_as_its_upgraded_horde() {
        // covers: TWN-BLD-003
        let part = part();
        // The level's dwelling alone: the horde; its upgrade needs the upgraded dwelling.
        let dwelling = town(&[VILLAGE_HALL, FORT, DWELLING_1]);
        let offer = buildable(&dwelling, &part, false);
        assert!(offer.contains(id(HORDE_1)) && !offer.contains(id(UPG_HORDE_1)));
        assert_eq!(
            check_build(&dwelling, &part, id(UPG_HORDE_1), false),
            Err(TownCommandError::Requirements)
        );
        // The upgraded dwelling ever built: the horde's slot offers the upgraded horde alone, at
        // its own cost, and either build leaves the same ever-built set.
        let upgraded = town(&[VILLAGE_HALL, FORT, DWELLING_1, UPG_DWELLING_1]);
        let offer = buildable(&upgraded, &part, false);
        assert!(!offer.contains(id(HORDE_1)) && offer.contains(id(UPG_HORDE_1)));
        assert_eq!(
            check_build(&upgraded, &part, id(HORDE_1), false),
            Err(TownCommandError::OfferedAs(id(UPG_HORDE_1)))
        );
        assert_eq!(
            check_build(&upgraded, &part, id(UPG_HORDE_1), false),
            Ok(&[(GOLD, 1000)][..])
        );
        assert_eq!(
            built_with(&upgraded, &part, id(HORDE_1)),
            built_with(&upgraded, &part, id(UPG_HORDE_1))
        );
        // The shrine is automatic with the upgraded horde but does not supersede it: no upgrade
        // of it (the upgraded horde is offered as itself above), and offered once a custom set
        // left it unbuilt beside the upgraded horde.
        let without_shrine = town(&[
            VILLAGE_HALL,
            FORT,
            DWELLING_1,
            UPG_DWELLING_1,
            HORDE_1,
            UPG_HORDE_1,
        ]);
        assert!(buildable(&without_shrine, &part, false).contains(id(SHRINE)));
        // A forbidden upgraded horde: the horde is still offered as it, which is forbidden, so
        // neither is (a horde's build would make the forbidden upgrade).
        let forbidding = keep_town(
            &[VILLAGE_HALL, FORT, DWELLING_1, UPG_DWELLING_1],
            &[UPG_HORDE_1, SHRINE],
        );
        let offer = buildable(&forbidding, &part, false);
        assert!(!offer.contains(id(HORDE_1)) && !offer.contains(id(UPG_HORDE_1)));
        assert_eq!(
            check_build(&forbidding, &part, id(HORDE_1), false),
            Err(TownCommandError::OfferedAs(id(UPG_HORDE_1)))
        );
        assert_eq!(
            check_build(&forbidding, &part, id(UPG_HORDE_1), false),
            Err(TownCommandError::Forbidden)
        );
        // An upgraded horde a custom set built without its horde: the horde is no longer
        // offered as it (it stands), so the horde is offered as itself.
        let custom = town(&[VILLAGE_HALL, FORT, DWELLING_1, UPG_DWELLING_1, UPG_HORDE_1]);
        assert_eq!(
            check_build(&custom, &part, id(HORDE_1), false),
            Ok(&[(GOLD, 1000)][..])
        );
    }

    #[test]
    fn a_capitol_is_offered_only_to_a_player_without_one() {
        // covers: TWN-BLD-002
        let part = part();
        let ready = town(&[
            VILLAGE_HALL,
            TOWN_HALL,
            CITY_HALL,
            TAVERN,
            FORT,
            CITADEL,
            CASTLE,
        ]);
        assert!(buildable(&ready, &part, false).contains(id(CAPITOL)));
        assert_eq!(
            check_build(&ready, &part, id(CAPITOL), false),
            Ok(&[(GOLD, 10_000)][..])
        );
        // The owner holds another town with a standing Capitol: never, but the rest as before.
        assert!(!buildable(&ready, &part, true).contains(id(CAPITOL)));
        assert_eq!(
            buildable(&ready, &part, true),
            buildable(&ready, &part, false).difference(set(&[CAPITOL]))
        );
        assert_eq!(
            check_build(&ready, &part, id(CAPITOL), true),
            Err(TownCommandError::Capitol)
        );
        assert!(!possible(&ready, &part, true).contains(id(CAPITOL)));
        assert!(possible(&ready, &part, false).contains(id(CAPITOL)));
        // Short of its requirements the Capitol is refused for them first.
        assert_eq!(
            check_build(&town(&[VILLAGE_HALL]), &part, id(CAPITOL), true),
            Err(TownCommandError::Requirements)
        );
    }

    #[test]
    fn the_hall_never_builds_the_grail() {
        // covers: TWN-BLD-002
        let part = part();
        for built in [&[VILLAGE_HALL][..], &FORT_AND_TAVERN_OFFER] {
            let town = town(built);
            assert!(!buildable(&town, &part, false).contains(id(GRAIL)));
            assert_eq!(
                check_build(&town, &part, id(GRAIL), false),
                Err(TownCommandError::Grail)
            );
            // As the original's `town_build_possible`, which no hall slot asks of it.
            assert!(possible(&town, &part, false).contains(id(GRAIL)));
        }
        let forbidden = keep_town(&[VILLAGE_HALL], &[GRAIL]);
        assert_eq!(
            check_build(&forbidden, &part, id(GRAIL), false),
            Err(TownCommandError::Forbidden)
        );
        assert!(!possible(&forbidden, &part, false).contains(id(GRAIL)));
    }

    #[test]
    fn a_town_built_in_today_offers_nothing() {
        // covers: TWN-BLD-004
        let part = part();
        let today = marked(&town(&[VILLAGE_HALL, FORT, TAVERN]));
        assert!(buildable(&today, &part, false).is_empty());
        assert_eq!(
            check_build(&today, &part, id(TAVERN), false),
            Err(TownCommandError::Built)
        );
        assert_eq!(
            check_build(&today, &part, id(CITADEL), false),
            Err(TownCommandError::BuiltToday)
        );
        // Not now, but still possible.
        assert_eq!(
            possible(&today, &part, false),
            possible(&town(&[VILLAGE_HALL, FORT, TAVERN]), &part, false)
        );
    }

    #[test]
    fn every_refusal_comes_in_its_order() {
        // covers: TWN-BLD-001, TWN-BLD-002, TWN-BLD-003, TWN-BLD-004
        // The Keep's horde made a Grail and a Capitol, so that one building meets every refusal;
        // each step takes the first away and the next shows.
        let part = part_with_keep(|keep| {
            keep.buildings[HORDE_1].roles.grail = true;
            keep.buildings[HORDE_1].roles.hall = Some(4);
        });
        let horde = id(HORDE_1);
        let check = |town: &Town, holds: bool| check_build(town, &part, horde, holds);
        let past = id(part.town(KEEP).unwrap().buildings.len());
        let all = marked(&keep_town(&[VILLAGE_HALL, HORDE_1], &[HORDE_1]));
        assert_eq!(
            check_build(&all, &part, past, true),
            Err(TownCommandError::UnknownBuilding)
        );
        assert_eq!(check(&all, true), Err(TownCommandError::Built));
        let unbuilt = marked(&keep_town(&[VILLAGE_HALL], &[HORDE_1]));
        assert_eq!(check(&unbuilt, true), Err(TownCommandError::Forbidden));
        let allowed = marked(&town(&[VILLAGE_HALL]));
        assert_eq!(check(&allowed, true), Err(TownCommandError::Grail));
        let part = part_with_keep(|keep| keep.buildings[HORDE_1].roles.hall = Some(4));
        let check = |town: &Town, holds: bool| check_build(town, &part, horde, holds);
        assert_eq!(check(&allowed, true), Err(TownCommandError::Requirements));
        let required = marked(&town(&[VILLAGE_HALL, FORT, DWELLING_1, UPG_DWELLING_1]));
        assert_eq!(check(&required, true), Err(TownCommandError::Capitol));
        assert_eq!(
            check(&required, false),
            Err(TownCommandError::OfferedAs(id(UPG_HORDE_1)))
        );
        let horde_alone = marked(&town(&[VILLAGE_HALL, FORT, DWELLING_1]));
        assert_eq!(
            check(&horde_alone, false),
            Err(TownCommandError::BuiltToday)
        );
        let free = town(&[VILLAGE_HALL, FORT, DWELLING_1]);
        assert_eq!(check(&free, false), Ok(&[(GOLD, 1000)][..]));
        // A free building's price is empty.
        let part = part_with_keep(|keep| keep.buildings[TAVERN].cost.clear());
        assert_eq!(
            check_build(&town(&[VILLAGE_HALL]), &part, id(TAVERN), false),
            Ok(&[][..])
        );
    }

    #[test]
    fn the_offer_is_what_check_build_allows_and_possible_holds_it() {
        // covers: TWN-BLD-002, TWN-BLD-003
        let part = part();
        let towns = [
            town(&[VILLAGE_HALL]),
            town(&[VILLAGE_HALL, FORT, TAVERN]),
            town(&[VILLAGE_HALL, FORT, DWELLING_1, UPG_DWELLING_1]),
            keep_town(&[VILLAGE_HALL, FORT, TAVERN], &[CITADEL, DWELLING_1]),
            marked(&town(&[VILLAGE_HALL, FORT])),
            town_of(MARSH, &[0, 2]),
        ];
        for town in &towns {
            for holds in [false, true] {
                let allowed = (0..MAX_BUILDINGS)
                    .map(id)
                    .filter(|&b| check_build(town, &part, b, holds).is_ok())
                    .fold(BuildingSet::EMPTY, BuildingSet::with);
                let offer = buildable(town, &part, holds);
                assert_eq!(offer, allowed, "{town:?}");
                assert_eq!(
                    offer.difference(possible(town, &part, holds)),
                    BuildingSet::EMPTY
                );
            }
        }
    }

    #[test]
    fn possible_tells_not_now_from_never() {
        // covers: TWN-BLD-002
        let part = part();
        let start = town(&[VILLAGE_HALL]);
        let ever = possible(&start, &part, false);
        // What stands and what waits for its requirements are possible; so is everything here.
        assert!(ever.contains(id(VILLAGE_HALL)) && ever.contains(id(CITADEL)));
        assert_eq!(ever, set(&(0..=SHIPYARD).collect::<Vec<_>>()));
        // A forbidden Fort: the Fort and everything that requires it, never.
        let no_fort = keep_town(&[VILLAGE_HALL], &[FORT]);
        let ever = possible(&no_fort, &part, false);
        for never in [FORT, CITADEL, CASTLE, CAPITOL, DWELLING_1, HORDE_1, LOOKOUT] {
            assert!(!ever.contains(id(never)), "{never}");
        }
        assert!(ever.contains(id(TOWN_HALL)) && ever.contains(id(SHIPYARD)));
        // A forbidden requirement ever built still counts (a custom set's).
        let built_fort = keep_town(&[VILLAGE_HALL, FORT], &[FORT]);
        assert!(possible(&built_fort, &part, false).contains(id(CITADEL)));
        assert!(possible(&town_of(WILD, &[]), &part, false).is_empty());
    }

    #[test]
    fn an_event_builds_without_the_halls_tests() {
        // covers: TWN-EVT-003
        let part = part();
        let event = |buildings: &[usize]| TownEvent {
            buildings: set(buildings),
            ..TownEvent::default()
        };
        // Less what is ever built and what another of its buildings supersedes; no requirement
        // (the Capitol, the Castle), forbidden (the Lookout), Grail or day's-mark test.
        let built_in = Town::new(KEEP, set(&[VILLAGE_HALL, TAVERN]), set(&[LOOKOUT]));
        let built_in = marked(&built_in);
        let wide = event(&[FORT, CASTLE, CAPITOL, GRAIL, TAVERN, LOOKOUT, CITY_HALL]);
        assert_eq!(
            event_buildings(&built_in, &part, &wide, true),
            set(&[CAPITOL, CASTLE, GRAIL, LOOKOUT])
        );
        // The Shipyard with water only.
        let shipyard = event(&[FORT, SHIPYARD]);
        let start = town(&[VILLAGE_HALL]);
        assert_eq!(
            event_buildings(&start, &part, &shipyard, true),
            set(&[FORT, SHIPYARD])
        );
        assert_eq!(
            event_buildings(&start, &part, &shipyard, false),
            set(&[FORT])
        );
        // A building past the faction's record, and a faction without a record: nothing.
        assert!(event_buildings(&start, &part, &event(&[40]), true).is_empty());
        assert!(event_buildings(&town_of(WILD, &[]), &part, &wide, true).is_empty());
        // Built in the order given, the event's set equals one build after another.
        let mut town = start.clone();
        for building in event_buildings(&start, &part, &wide, true).iter() {
            town = town.rebuilt(built_with(&town, &part, building).unwrap(), false);
        }
        assert!(!town.built_today());
        assert!(town.built().contains(id(CAPITOL)) && town.built().contains(id(FORT)));
    }

    #[test]
    fn twn_quirk_7_an_events_bits_past_the_maps_slots_build_nothing() {
        // covers: TWN-EVT-003
        // The original's loop reads 44 bits through a table of 41 a faction, so its bits 41 to
        // 43 name the next faction's buildings (past the table for the Conflux); the engine maps
        // the 41 slots only.
        let part = part();
        let record = part.town(KEEP).unwrap();
        let slots = SlotSet::from_bits(0b111 << 41 | 1 << 5);
        assert_eq!(slots.bits(), 1 << 5);
        let event = TownEvent {
            buildings: slots.buildings(record),
            ..TownEvent::default()
        };
        assert_eq!(
            event_buildings(&town(&[VILLAGE_HALL]), &part, &event, true),
            set(&[TAVERN])
        );
    }
}
