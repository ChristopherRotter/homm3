//! The facts a siege takes from its town (E09 increment 1's design note §6, §15; ADR-0023 points
//! 2 and 3): the fort level, the battle's [`Siege`], the natural obstacles and the defending
//! hero's bonuses. The game puts them into the battle's setup (E09 T8b); the battle part's siege
//! record for the faction gives the walls', the moat's and the turrets' details.

use alloc::vec::Vec;

use h3_contracts::{FortLevel, Moat, Siege, TownBonus, TownBonuses};

use crate::derive::Roles;
use crate::record::Town;
use crate::ruleset::{DefenderBonus, TownRuleset};

/// The town's fort level: its highest standing `fort` (1 the Fort to 3 the Castle; the original
/// tests the Fort, the Citadel and the Castle in that order and keeps the last found, E09 survey
/// §9.4), [`FortLevel::None`] without one.
// rule: TWN-SIE-003
pub fn fort_level(town: &Town, part: &TownRuleset) -> FortLevel {
    fort(&Roles::of(town, part))
}

/// [`fort_level`] of a town already read with its record (E09 T26a1: the growth's one pass).
pub(crate) fn fort(roles: &Roles<'_>) -> FortLevel {
    match roles
        .standing_roles()
        .filter_map(|(_, r)| r.fort)
        .max()
        .unwrap_or(0)
    {
        0 => FortLevel::None,
        1 => FortLevel::Fort,
        2 => FortLevel::Citadel,
        _ => FortLevel::Castle,
    }
}

/// The town's facts for a battle at it (ADR-0023 point 2, as amended by E09 and CLT-Q40; survey
/// §9.4, §16.4):
///
/// - `faction` the town's, `fort` its [`fort_level`];
/// - `moat` its record's from a Citadel, else [`Moat::Absent`];
/// - `structures` its standing buildings not forbidden, less its hall-1 building (no faction's
///   list holds the Village Hall, so the original never counts it: E09 decision 28);
/// - `escape_tunnel` a building of role `escape_tunnel` ever built;
/// - `bonuses` one [`TownBonus`] per building ever built with a morale or luck change, in index
///   order: its `morale` while it stands (the Brotherhood of the Sword supersedes the Tavern, so
///   a Castle's town gives +2, not +3), its `luck` once ever built (the Fountain of Fortune), as
///   the original reads its two sets; at most [`h3_contracts::MAX_TOWN_BONUSES`] (the part's
///   compiler refuses a record with more such buildings).
// rule: TWN-SIE-003
pub fn siege(town: &Town, part: &TownRuleset) -> Siege {
    let roles = Roles::of(town, part);
    let fort = fort(&roles);
    let structures = roles
        .roles(roles.standing().difference(town.forbidden()))
        .filter(|(_, r)| r.hall != Some(1))
        .count();
    Siege {
        faction: town.faction(),
        fort,
        moat: roles
            .record()
            .filter(|_| fort >= FortLevel::Citadel)
            .map_or(Moat::Absent, |r| r.moat),
        // At most 64 buildings, which fits.
        structures: u8::try_from(structures).unwrap_or(u8::MAX),
        escape_tunnel: roles.built_roles().any(|(_, r)| r.escape_tunnel),
        bonuses: bonuses(&roles),
    }
}

/// The town's morale and luck bonuses (BAT-SIE-025's town side): see [`siege`].
fn bonuses(roles: &Roles<'_>) -> TownBonuses {
    let standing = roles.standing();
    let bonuses: Vec<TownBonus> = roles
        .built_roles()
        .map(|(id, r)| TownBonus {
            building: id.raw(),
            morale: if standing.contains(id) { r.morale } else { 0 },
            luck: r.luck,
        })
        .filter(|b| b.morale != 0 || b.luck != 0)
        .collect();
    // At most `MAX_TOWN_BONUSES`: the part's compiler's check.
    TownBonuses::new(&bonuses).unwrap_or(TownBonuses::NONE)
}

/// Whether a siege of the town draws the battlefield's natural obstacles too: its record's
/// `siege_obstacles`, from a Citadel (the base pack's Stronghold; the E04c survey's S-29; the
/// game draws them on the field's terrain, design §15).
// rule: TWN-SIE-005
pub fn siege_obstacles(town: &Town, part: &TownRuleset) -> bool {
    let roles = Roles::of(town, part);
    fort(&roles) >= FortLevel::Citadel && roles.record().is_some_and(|r| r.siege_obstacles)
}

/// The defending hero's bonuses in a siege of the town (ADR-0023 point 3): the `defender` role's
/// attack, defense, power and mana, each summed over the buildings ever built, saturating (the
/// Grails, the Fortress' Blood Obelisk and Glyphs of Fear, the Inferno's Brimstone Stormclouds;
/// survey §9.4). The game raises the defending hero by them (design §15).
// rule: TWN-SIE-004
pub fn defender_bonus(town: &Town, part: &TownRuleset) -> DefenderBonus {
    Roles::of(town, part)
        .built_roles()
        .fold(DefenderBonus::default(), |sum, (_, r)| DefenderBonus {
            attack: sum.attack.saturating_add(r.defender.attack),
            defense: sum.defense.saturating_add(r.defender.defense),
            power: sum.power.saturating_add(r.defender.power),
            mana: sum.mana.saturating_add(r.defender.mana),
        })
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::vec;
    use std::vec::Vec;

    use super::*;
    use crate::building::BuildingId;
    use crate::fixture::*;

    fn bonus(building: usize, morale: i8, luck: i8) -> TownBonus {
        TownBonus {
            building: BuildingId::from_index(building).unwrap().raw(),
            morale,
            luck,
        }
    }

    #[test]
    fn the_fort_level_is_the_highest_standing_fort() {
        // covers: TWN-SIE-003
        let part = part();
        let level = |built: &[usize]| fort_level(&town(built), &part);
        assert_eq!(level(&[VILLAGE_HALL]), FortLevel::None);
        assert_eq!(level(&[FORT]), FortLevel::Fort);
        assert_eq!(level(&[FORT, CITADEL]), FortLevel::Citadel);
        assert_eq!(level(&[FORT, CITADEL, CASTLE]), FortLevel::Castle);
        // A custom set's Castle alone.
        assert_eq!(level(&[CASTLE]), FortLevel::Castle);
        assert_eq!(
            fort_level(&town_of(WILD, &[CASTLE]), &part),
            FortLevel::None
        );
    }

    #[test]
    fn the_moat_comes_with_a_citadel() {
        // covers: TWN-SIE-003
        let part = part();
        let moat = |t: &Town| siege(t, &part).moat;
        assert_eq!(moat(&town(&[FORT])), Moat::Absent);
        assert_eq!(moat(&town(&[FORT, CITADEL])), Moat::Damaging);
        assert_eq!(moat(&town(&[FORT, CITADEL, CASTLE])), Moat::Damaging);
        assert_eq!(moat(&town_of(MARSH, &[0, 2])), Moat::Harmless);
        assert_eq!(moat(&town_of(WILD, &[CASTLE])), Moat::Absent);
    }

    #[test]
    fn the_structures_are_the_standing_buildings_but_the_village_hall() {
        // covers: TWN-SIE-003
        let part = part();
        let structures = |t: &Town| siege(t, &part).structures;
        // The standard set of an owned town with a Fort: the Fort, the Tavern, the dwelling.
        assert_eq!(
            structures(&town(&[VILLAGE_HALL, FORT, TAVERN, DWELLING_1])),
            3
        );
        // A neutral town's: the Fort and both dwellings.
        let neutral = [VILLAGE_HALL, FORT, DWELLING_1, DWELLING_2];
        assert_eq!(structures(&town(&neutral)), 3);
        // A chain counts once while its top stands; a Town Hall counts.
        let built = [
            VILLAGE_HALL,
            TOWN_HALL,
            FORT,
            CITADEL,
            DWELLING_1,
            UPG_DWELLING_1,
        ];
        assert_eq!(structures(&town(&built)), 3);
        // A forbidden building does not count, nor a building the record lacks.
        let forbidden = Town::new(KEEP, set(&[FORT, TAVERN, 63]), set(&[TAVERN]));
        assert_eq!(structures(&forbidden), 1);
        assert_eq!(structures(&town(&[])), 0);
        assert_eq!(structures(&town_of(WILD, &[FORT])), 0);
    }

    #[test]
    fn the_escape_tunnel_is_read_over_the_buildings_ever_built() {
        // covers: TWN-SIE-003
        let part = part();
        assert!(siege(&town(&[ESCAPE_TUNNEL]), &part).escape_tunnel);
        assert!(!siege(&town(&[FORT, TAVERN]), &part).escape_tunnel);
        let s = siege(&town(&[VILLAGE_HALL, FORT]), &part);
        assert_eq!((s.faction, s.fort), (KEEP, FortLevel::Fort));
    }

    #[test]
    fn the_bonuses_name_their_buildings() {
        // covers: TWN-SIE-003
        let part = part();
        let bonuses = |built: &[usize]| {
            let s = siege(&town(built), &part);
            (
                s.bonuses.iter().copied().collect::<Vec<_>>(),
                s.morale(),
                s.luck(),
            )
        };
        assert_eq!(bonuses(&[VILLAGE_HALL]), (Vec::new(), 0, 0));
        assert_eq!(bonuses(&[TAVERN]), (vec![bonus(TAVERN, 1, 0)], 1, 0));
        // The Brotherhood supersedes the Tavern: +2, not +3.
        assert_eq!(
            bonuses(&[TAVERN, BROTHERHOOD]),
            (vec![bonus(BROTHERHOOD, 2, 0)], 2, 0)
        );
        // Luck is read over the buildings ever built: the superseded pond's too.
        assert_eq!(
            bonuses(&[TAVERN, FOUNTAIN, MYSTIC_POND]),
            (
                vec![
                    bonus(TAVERN, 1, 0),
                    bonus(FOUNTAIN, 0, 2),
                    bonus(MYSTIC_POND, 0, 1)
                ],
                1,
                3
            )
        );
        assert!(
            siege(&town_of(WILD, &[TAVERN]), &part)
                .bonuses
                .as_slice()
                .is_empty()
        );
    }

    #[test]
    fn the_natural_obstacles_come_with_a_citadel_and_the_record_s_flag() {
        // covers: TWN-SIE-005
        let part = part();
        assert!(!siege_obstacles(&town(&[FORT]), &part));
        assert!(siege_obstacles(&town(&[FORT, CITADEL]), &part));
        assert!(siege_obstacles(&town(&[CASTLE]), &part));
        assert!(!siege_obstacles(&town_of(MARSH, &[0, 2]), &part));
        assert!(!siege_obstacles(&town_of(WILD, &[CASTLE]), &part));
    }

    #[test]
    fn the_defender_s_bonuses_sum_the_buildings_ever_built() {
        // covers: TWN-SIE-004
        let part = part();
        let sum = |built: &[usize]| defender_bonus(&town(built), &part);
        assert_eq!(sum(&[VILLAGE_HALL, FORT]), DefenderBonus::default());
        let expected = DefenderBonus {
            attack: 12,
            defense: 12,
            power: 0,
            mana: 150,
        };
        assert_eq!(sum(&[GRAIL, OBELISK, GLYPHS]), expected);
        assert_eq!(
            defender_bonus(&town_of(WILD, &[GRAIL]), &part),
            DefenderBonus::default()
        );
    }
}
