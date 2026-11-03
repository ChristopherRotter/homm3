//! What a capture reads and writes of a town (E09 increment 1's design note §6, §11): the reveal
//! for the new owner and the Capitol's limit. The capture itself (the owner, the garrison, the
//! tiles revealed) is the adventure's, the game composes the two (E09 T8a).

use crate::building::{BuildingId, BuildingSet};
use crate::derive::Roles;
use crate::record::Town;
use crate::ruleset::{Reveal, TownRuleset};

/// The reveal for the town's owner at the start and at a capture: the part's `reveal_radius`, or
/// the largest `reveal` of a building ever built when larger, [`Reveal::All`] over any radius; a
/// radius of 0 without parameters (E09 survey §8.3 step 8: 5, a Tower's Lookout Tower 20, its
/// Grail the whole map).
// rule: TWN-CAP-001
pub fn reveal(town: &Town, part: &TownRuleset) -> Reveal {
    let radius = part.params().map_or(0, |p| p.reveal_radius);
    Roles::of(town, part)
        .built_roles()
        .filter_map(|(_, r)| r.reveal)
        .fold(Reveal::Radius(radius), larger)
}

/// The larger of two reveals.
fn larger(a: Reveal, b: Reveal) -> Reveal {
    match (a, b) {
        (Reveal::Radius(a), Reveal::Radius(b)) => Reveal::Radius(a.max(b)),
        _ => Reveal::All,
    }
}

/// The town's standing hall-4 building (the Capitol), if any.
pub(crate) fn standing_capitol(town: &Town, part: &TownRuleset) -> Option<BuildingId> {
    capitol_of(&Roles::of(town, part))
}

/// The standing hall-4 building of a town read with its record.
fn capitol_of(roles: &Roles<'_>) -> Option<BuildingId> {
    roles
        .standing_roles()
        .find_map(|(id, r)| (r.hall == Some(4)).then_some(id))
}

/// The town's ever-built set after the Capitol's limit at a capture: with a standing hall-4
/// building and `holds_another` (the new owner holds another town with one), that building out
/// and the record's `capitol_demoted_to` in, so that the hall-3 building stands; else unchanged.
/// The caller passes `false` for a neutral owner (E09 survey §8.4: `capitol_limit` 0x5BEC60; the
/// start never applies it, §11.3).
// rule: TWN-CAP-002
pub fn capitol_limit(town: &Town, part: &TownRuleset, holds_another: bool) -> BuildingSet {
    let roles = Roles::of(town, part);
    let built = roles.built();
    roles
        .record()
        .zip(capitol_of(&roles).filter(|_| holds_another))
        .map_or(built, |(record, capitol)| {
            built
                .difference(BuildingSet::EMPTY.with(capitol))
                .with(record.capitol_demoted_to)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::*;

    const HALLS: [usize; 4] = [VILLAGE_HALL, TOWN_HALL, CITY_HALL, CAPITOL];

    #[test]
    fn the_reveal_is_the_radius_or_a_larger_building_s() {
        // covers: TWN-CAP-001
        let part = part();
        let reveal_of = |built: &[usize]| reveal(&town(built), &part);
        assert_eq!(reveal_of(&[VILLAGE_HALL]), Reveal::Radius(5));
        assert_eq!(reveal_of(&[VILLAGE_HALL, LOOKOUT]), Reveal::Radius(20));
        // The Grail's whole map wins over any radius, before or after it in index order.
        assert_eq!(reveal_of(&[GRAIL]), Reveal::All);
        assert_eq!(reveal_of(&[GRAIL, LOOKOUT]), Reveal::All);
        // A part's radius above the building's stays; a faction without a record reveals the
        // radius; without parameters, nothing around the town.
        let wide = part_with_radius(30);
        assert_eq!(reveal(&town(&[LOOKOUT]), &wide), Reveal::Radius(30));
        assert_eq!(reveal(&town_of(WILD, &[LOOKOUT]), &part), Reveal::Radius(5));
        assert_eq!(reveal(&town(&[LOOKOUT]), &empty_part()), Reveal::Radius(0));
    }

    #[test]
    fn a_standing_capitol_is_the_hall_4_building() {
        let part = part();
        let capitol = BuildingId::from_index(CAPITOL);
        assert_eq!(standing_capitol(&town(&HALLS), &part), capitol);
        assert_eq!(standing_capitol(&town(&[CAPITOL]), &part), capitol);
        assert_eq!(standing_capitol(&town(&HALLS[..3]), &part), None);
        assert_eq!(standing_capitol(&town_of(WILD, &HALLS), &part), None);
        assert_eq!(standing_capitol(&town(&HALLS), &empty_part()), None);
    }

    #[test]
    fn a_captured_capitol_becomes_a_city_hall_when_the_captor_holds_another() {
        // covers: TWN-CAP-002
        let part = part();
        let limited = capitol_limit(
            &town(&[FORT, VILLAGE_HALL, TOWN_HALL, CITY_HALL, CAPITOL]),
            &part,
            true,
        );
        assert_eq!(limited, set(&[FORT, VILLAGE_HALL, TOWN_HALL, CITY_HALL]));
        let demoted = Town::new(KEEP, limited, BuildingSet::EMPTY);
        assert_eq!(crate::hall_level(&demoted, &part), 3);
        // A custom set's Capitol alone: the City Hall enters and stands.
        assert_eq!(
            capitol_limit(&town(&[CAPITOL]), &part, true),
            set(&[CITY_HALL])
        );
        // Unchanged: the captor holds no other (or is neutral), no Capitol stands, no record.
        let unchanged = |town: Town, part: &TownRuleset, holds_another: bool| {
            assert_eq!(capitol_limit(&town, part, holds_another), town.built());
        };
        unchanged(town(&HALLS), &part, false);
        unchanged(town(&HALLS[..3]), &part, true);
        unchanged(town_of(WILD, &HALLS), &part, true);
        unchanged(town(&HALLS), &empty_part(), true);
    }
}
