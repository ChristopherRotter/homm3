//! A town's hall and its day's income (E09 increment 1's design note §6, §16): the gold of its
//! highest standing hall and of a Grail ever built, the resources of a standing Resource Silo.

use crate::derive::Roles;
use crate::record::Town;
use crate::ruleset::TownRuleset;

/// The town's hall level, 1 (the Village Hall) to 4 (the Capitol): its highest standing `hall`,
/// 1 without one (the original pays the Village Hall's gold without testing for it, E09 survey
/// §11.1).
// rule: TWN-INC-001
pub fn hall_level(town: &Town, part: &TownRuleset) -> u8 {
    Roles::of(town, part)
        .standing_roles()
        .filter_map(|(_, r)| r.hall)
        .fold(1, u8::max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::*;

    #[test]
    fn the_hall_level_is_the_highest_standing_hall() {
        // covers: TWN-INC-001
        let part = part();
        let level = |built: &[usize]| hall_level(&town(built), &part);
        assert_eq!(level(&[VILLAGE_HALL]), 1);
        assert_eq!(level(&[VILLAGE_HALL, TOWN_HALL, CITY_HALL]), 3);
        assert_eq!(level(&[VILLAGE_HALL, TOWN_HALL, CITY_HALL, CAPITOL]), 4);
        // A custom set's Town Hall alone; no hall at all pays as the Village Hall.
        assert_eq!(level(&[TOWN_HALL]), 2);
        assert_eq!(level(&[]), 1);
        assert_eq!(hall_level(&town_of(WILD, &[CAPITOL]), &part), 1);
    }
}
