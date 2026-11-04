//! A town's hall and its day's income (E09 increment 1's design note §6, §16): the gold of its
//! highest standing hall and of a Grail ever built, the resources of a standing Resource Silo.

use alloc::vec::Vec;

use h3_content::ResourceId;

use crate::derive::Roles;
use crate::record::Town;
use crate::ruleset::TownRuleset;

/// The town's hall level, 1 (the Village Hall) to 4 (the Capitol): its highest standing `hall`,
/// 1 without one (the original pays the Village Hall's gold without testing for it, E09 survey
/// §11.1).
// rule: TWN-INC-001
pub fn hall_level(town: &Town, part: &TownRuleset) -> u8 {
    level(&Roles::of(town, part))
}

fn level(roles: &Roles<'_>) -> u8 {
    roles
        .standing_roles()
        .filter_map(|(_, r)| r.hall)
        .fold(1, u8::max)
}

/// The town's day's income, which the day's advance pays its owner (design §16, E09 T8c): the
/// part's `hall_gold` of its [`hall_level`] and `grail_gold` with a building of role `grail` ever
/// built, in the part's `gold`; the record's `silo` resources with a standing building of role
/// `silo` (the original's table gives no faction gold, a part may: summed). Each resource once,
/// in the core's resource order, none 0, the sums saturating. Nothing without the parameters (a
/// part without towns).
// rule: TWN-INC-001, TWN-INC-002
pub fn daily_income(town: &Town, part: &TownRuleset) -> Vec<(ResourceId, u32)> {
    let roles = Roles::of(town, part);
    let gold = part.params().map(|p| {
        let hall = p
            .hall_gold
            .get(usize::from(level(&roles)).saturating_sub(1))
            .copied()
            .unwrap_or(0);
        let grail = roles.built_roles().any(|(_, r)| r.grail);
        (
            p.gold,
            hall.saturating_add(if grail { p.grail_gold } else { 0 }),
        )
    });
    let silo = roles
        .record()
        .filter(|_| roles.standing_roles().any(|(_, r)| r.silo))
        .map_or(&[][..], |r| r.silo.as_slice());
    gold.into_iter()
        .chain(silo.iter().copied())
        .filter(|&(_, amount)| amount > 0)
        .fold(Vec::new(), |mut income, (resource, amount)| {
            match income.iter_mut().find(|(r, _)| *r == resource) {
                Some((_, sum)) => *sum = u32::saturating_add(*sum, amount),
                None => {
                    let at = income.partition_point(|&(r, _)| r < resource);
                    income.insert(at, (resource, amount));
                }
            }
            income
        })
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

    #[test]
    fn a_town_pays_its_hall_and_its_grail_in_gold() {
        // covers: TWN-INC-001
        let part = part();
        let income = |built: &[usize]| daily_income(&town(built), &part);
        assert_eq!(income(&[VILLAGE_HALL]), [(GOLD, 500)]);
        assert_eq!(income(&[VILLAGE_HALL, TOWN_HALL]), [(GOLD, 1000)]);
        assert_eq!(
            income(&[VILLAGE_HALL, TOWN_HALL, CITY_HALL]),
            [(GOLD, 2000)]
        );
        let capitol = [VILLAGE_HALL, TOWN_HALL, CITY_HALL, CAPITOL];
        assert_eq!(income(&capitol), [(GOLD, 4000)]);
        assert_eq!(income(&[CAPITOL, GRAIL]), [(GOLD, 9000)]);
        // A town without a hall pays the Village Hall's; a faction without a record too.
        assert_eq!(income(&[GRAIL]), [(GOLD, 5500)]);
        assert_eq!(
            daily_income(&town_of(WILD, &[CAPITOL]), &part),
            [(GOLD, 500)]
        );
        // The part without towns pays nothing.
        assert!(daily_income(&town(&capitol), &empty_part()).is_empty());
    }

    #[test]
    fn a_standing_silo_pays_its_resources_beside_the_gold() {
        // covers: TWN-INC-002
        let part = part();
        let keep = daily_income(&town(&[VILLAGE_HALL, SILO]), &part);
        assert_eq!(keep, [(WOOD, 1), (ORE, 1), (GOLD, 500)]);
        // A silo that pays gold adds it to the hall's.
        let marsh = daily_income(&town_of(MARSH, &[0, 1]), &part);
        assert_eq!(marsh, [(GOLD, 750)]);
        assert_eq!(daily_income(&town_of(MARSH, &[0]), &part), [(GOLD, 500)]);
    }

    #[test]
    fn no_resource_is_paid_as_0_and_the_sums_saturate() {
        // covers: TWN-INC-001, TWN-INC-002
        let none = part_with(|p| p.hall_gold = [0; 4]);
        assert_eq!(
            daily_income(&town(&[VILLAGE_HALL, SILO]), &none),
            [(WOOD, 1), (ORE, 1)]
        );
        let most = part_with(|p| {
            p.hall_gold = [u32::MAX; 4];
            p.grail_gold = 1;
        });
        assert_eq!(daily_income(&town(&[GRAIL]), &most), [(GOLD, u32::MAX)]);
        assert_eq!(
            daily_income(&town_of(MARSH, &[0, 1]), &most),
            [(GOLD, u32::MAX)]
        );
    }
}
