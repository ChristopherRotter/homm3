//! Tests of a town's growth (E09 T25a) on the fixture's Keep: level 1 (handles 10 and 11) grows
//! 14 with a horde of 8, level 2 (20) 9, level 3 (30) 7 without a dwelling, level 4 (40) 4.

use super::*;
use crate::fixture::{
    CASTLE, CITADEL, DWELLING_1, DWELLING_2, FORT, GRAIL, HORDE_1, KEEP, UPG_DWELLING_1,
    UPG_HORDE_1, WILD, core, creature, part, part_with_keep, set, town, town_of,
};
use crate::record::Summoned;

/// An owned town's inputs: the Statue's share, the Legion's levels, the map's dwellings.
fn owned(share_bp: u16, legion: [u16; MAX_CREATURE_LEVELS]) -> GrowthInputs {
    GrowthInputs {
        owned: true,
        share_bp,
        legion,
        ..GrowthInputs::NEUTRAL
    }
}

/// The growth at each index of `built`'s Keep town with `inputs`.
fn growths(built: &[usize], inputs: &GrowthInputs) -> [u16; CREATURE_INDEXES] {
    let (part, core, town) = (part(), core(), town(built));
    core::array::from_fn(|index| growth(&town, &part, &core, index, inputs))
}

#[test]
fn the_neutral_inputs_are_the_default() {
    assert_eq!(GrowthInputs::NEUTRAL, GrowthInputs::default());
}

#[test]
fn an_index_s_creature_is_its_level_s_or_its_upgrade() {
    let part = part();
    let keep = part.town(KEEP).unwrap();
    let at = |index| index_creature(keep, index);
    assert_eq!((at(0), at(7)), (Some(creature(10)), Some(creature(11))));
    assert_eq!((at(3), at(10)), (Some(creature(40)), Some(creature(41))));
    // Level 3 has no upgrade; the Keep has four levels; past the fourteen indexes none.
    assert_eq!((at(2), at(9)), (Some(creature(30)), None));
    assert_eq!((at(4), at(13), at(14)), (None, None, None));
}

// covers: TWN-GRO-001
#[test]
fn a_dwelling_grows_its_creature_s_growth_while_its_upgrade_is_not_built() {
    let neutral = GrowthInputs::NEUTRAL;
    assert_eq!(growths(&[], &neutral), [0; CREATURE_INDEXES]);
    // The base dwelling alone: index 0; the upgrade's index only with the upgraded dwelling.
    let mut expected = [0; CREATURE_INDEXES];
    expected[0] = 14;
    assert_eq!(growths(&[DWELLING_1], &neutral), expected);
    expected[1] = 9;
    assert_eq!(growths(&[DWELLING_1, DWELLING_2], &neutral), expected);
    // The upgraded dwelling ever built: the upgrade's index grows, the base's none.
    let upgraded = growths(&[DWELLING_1, UPG_DWELLING_1, DWELLING_2], &neutral);
    assert_eq!((upgraded[0], upgraded[7], upgraded[1]), (0, 14, 9));
    // An upgraded dwelling without its base still grows (a custom set's), at its index alone.
    let alone = growths(&[UPG_DWELLING_1], &neutral);
    assert_eq!((alone[0], alone[7]), (0, 14));
}

#[test]
fn the_castle_adds_its_growth_the_citadel_half() {
    let neutral = GrowthInputs::NEUTRAL;
    let at = |built: &[usize]| {
        let g = growths(built, &neutral);
        (g[0], g[1])
    };
    let dwellings = [DWELLING_1, DWELLING_2];
    assert_eq!(at(&dwellings), (14, 9));
    assert_eq!(at(&[DWELLING_1, DWELLING_2, FORT]), (14, 9));
    // Level 2's 9 ÷ 2 truncated.
    assert_eq!(at(&[DWELLING_1, DWELLING_2, FORT, CITADEL]), (21, 13));
    assert_eq!(
        at(&[DWELLING_1, DWELLING_2, FORT, CITADEL, CASTLE]),
        (28, 18)
    );
}

#[test]
fn an_owned_town_adds_the_statue_s_share_and_the_legion() {
    let castle = [DWELLING_1, DWELLING_2, FORT, CITADEL, CASTLE];
    // The Statue's half of the base and the castle's, the Legion's level 2 piece.
    let mut legion = [0; MAX_CREATURE_LEVELS];
    legion[1] = 5;
    let g = growths(&castle, &owned(5000, legion));
    assert_eq!((g[0], g[1]), (28 + 14, 18 + 9 + 5));
    // Truncated: a third of level 2's 18 is 5.
    let g = growths(&castle, &owned(3334, [0; MAX_CREATURE_LEVELS]));
    assert_eq!((g[0], g[1]), (28 + 9, 18 + 6));
    // A neutral town reads neither.
    let neutral = GrowthInputs {
        owned: false,
        ..owned(5000, legion)
    };
    let g = growths(&castle, &neutral);
    assert_eq!((g[0], g[1]), (28, 18));
    // A share past the whole (no content gives one) is no overflow.
    let g = growths(&castle, &owned(u16::MAX, [0; MAX_CREATURE_LEVELS]));
    assert_eq!(u32::from(g[0]), 28 + 28 * 65_535 / 10_000);
}

#[test]
fn a_horde_adds_its_creature_s_horde_growth_at_its_index() {
    let neutral = GrowthInputs::NEUTRAL;
    let g = growths(&[DWELLING_1, HORDE_1], &neutral);
    assert_eq!((g[0], g[7]), (14 + 8, 0));
    // The upgraded horde grows at the upgrade's index; the base horde there adds nothing.
    let g = growths(
        &[DWELLING_1, UPG_DWELLING_1, HORDE_1, UPG_HORDE_1],
        &neutral,
    );
    assert_eq!((g[0], g[7]), (0, 14 + 8));
    let g = growths(&[DWELLING_1, UPG_DWELLING_1, HORDE_1], &neutral);
    assert_eq!((g[0], g[7]), (0, 14));
}

#[test]
fn the_map_s_dwellings_add_at_a_growing_index_only() {
    let mut inputs = GrowthInputs::NEUTRAL;
    inputs.dwellings[0] = 2;
    inputs.dwellings[7] = 2;
    inputs.dwellings[2] = 3;
    // Index 7 without the upgraded dwelling and level 3 without a dwelling grow nothing.
    let g = growths(&[DWELLING_1], &inputs);
    assert_eq!((g[0], g[7], g[2]), (16, 0, 0));
}

#[test]
fn the_grail_adds_half_of_the_sum() {
    // Batch TWN-3's row 2 (survey `E09-growth.md` §17): the Castle, the Grail, the Statue
    // elsewhere, Legs of Legion on the visitor and two flagged dwellings at index 0 give
    // (14 + 14 + 14 + 2) × 1.5 = 66 and (9 + 9 + 9 + 5) × 1.5 = 48.
    let built = [DWELLING_1, DWELLING_2, FORT, CITADEL, CASTLE, GRAIL];
    let mut legion = [0; MAX_CREATURE_LEVELS];
    legion[1] = 5;
    let mut inputs = owned(5000, legion);
    inputs.dwellings[0] = 2;
    let g = growths(&built, &inputs);
    assert_eq!((g[0], g[1]), (66, 48));
    // Truncated: level 1's 15 + 7.
    let g = growths(&[DWELLING_1, GRAIL, HORDE_1], &GrowthInputs::NEUTRAL);
    assert_eq!(g[0], 22 + 11);
    let g = growths(&[DWELLING_2, GRAIL], &GrowthInputs::NEUTRAL);
    assert_eq!(g[1], 13);
}

// TWN-QUIRK-8: a growth saturates (T-10 fixed).
#[test]
fn a_growth_is_held_at_the_bound() {
    let mut inputs = GrowthInputs::NEUTRAL;
    inputs.dwellings[0] = 40_000;
    assert_eq!(growths(&[DWELLING_1], &inputs)[0], MAX_AVAILABLE);
    assert_eq!(growths(&[DWELLING_1, GRAIL], &inputs)[0], MAX_AVAILABLE);
}

#[test]
fn a_town_without_a_record_or_a_creature_the_core_lacks_grows_nothing() {
    let (part, core) = (part(), core());
    let inputs = GrowthInputs::NEUTRAL;
    let wild = town_of(WILD, &[DWELLING_1]);
    assert_eq!(growth(&wild, &part, &core, 0, &inputs), 0);
    let keep = town(&[DWELLING_1]);
    assert_eq!(growth(&keep, &part, &core, 14, &inputs), 0);
    let unknown = part_with_keep(|k| k.creatures[0].base = creature(99));
    assert_eq!(growth(&keep, &unknown, &core, 0, &inputs), 0);
}

/// `town` with the first creatures of the Keep's buildings `new`.
fn first(town: Town, new: &[usize]) -> Town {
    let (part, core) = (part(), core());
    first_creatures(town, part.town(KEEP).unwrap(), &core, set(new))
}

// covers: TWN-GRO-005
#[test]
fn a_built_dwelling_gives_its_first_creatures() {
    let built = first(town(&[]), &[DWELLING_1, DWELLING_2]);
    assert_eq!((built.available(0), built.available(1)), (14, 9));
    // A base dwelling overwrites what its index held; a horde and other buildings add nothing.
    let held = town(&[]).with_available(0, 3).with_available(1, 5);
    let again = first(held, &[DWELLING_1, HORDE_1, GRAIL, FORT]);
    assert_eq!((again.available(0), again.available(1)), (14, 5));
    // The upgraded dwelling takes its base index's creatures, which it leaves empty.
    let upgraded = first(again, &[UPG_DWELLING_1]);
    assert_eq!((upgraded.available(0), upgraded.available(7)), (0, 14));
    // Both at once: the base first, then the upgrade.
    let both = first(town(&[]), &[UPG_DWELLING_1, DWELLING_1]);
    assert_eq!((both.available(0), both.available(7)), (0, 14));
    // Nothing new: nothing changes, the Portal and the rest kept.
    let imp = Summoned {
        creature: creature(4),
        count: 2,
    };
    let kept = upgraded.with_portal(Some(imp));
    assert_eq!(first(kept.clone(), &[]), kept);
    // A creature the core lacks gives none.
    let (core, unknown) = (
        core(),
        part_with_keep(|k| k.creatures[0].base = creature(99)),
    );
    let record = unknown.town(KEEP).unwrap();
    let none = first_creatures(
        town(&[]).with_available(0, 4),
        record,
        &core,
        set(&[DWELLING_1]),
    );
    assert_eq!(none.available(0), 0);
}

// covers: TWN-EVT-004
#[test]
fn an_event_s_creatures_go_to_the_upgraded_dwelling_else_the_dwelling_else_nowhere() {
    let part = part();
    let creatures = [3, 4, 5, 6, 0, 0, 9];
    let counts = |built: &[usize], before: &[(usize, u16)]| {
        let town = before.iter().fold(town(built), |t, &(index, count)| {
            t.with_available(index, count)
        });
        *event_creatures(&town, &part, &creatures).available_all()
    };
    // No dwelling: every level's lost.
    assert_eq!(counts(&[], &[]), [0; CREATURE_INDEXES]);
    // Level 1's dwelling and level 2's: added to the counts there (no week's growth); level 3
    // without a dwelling, level 4 without its, level 7 the Keep lacks: lost.
    let mut expected = [0; CREATURE_INDEXES];
    expected[0] = 10;
    expected[1] = 4;
    assert_eq!(counts(&[DWELLING_1, DWELLING_2], &[(0, 7)]), expected);
    // The upgraded dwelling ever built: level 1's go to its index, the base's count kept.
    let mut expected = [0; CREATURE_INDEXES];
    expected[0] = 2;
    expected[7] = 4;
    assert_eq!(
        counts(&[DWELLING_1, UPG_DWELLING_1], &[(0, 2), (7, 1)]),
        expected
    );
    // Held at the bound (`TWN-QUIRK-8`).
    let held = counts(&[DWELLING_1], &[(0, MAX_AVAILABLE - 1)]);
    assert_eq!(held[0], MAX_AVAILABLE);
    // A faction without a record keeps its counts.
    let wild = town_of(WILD, &[DWELLING_1]).with_available(0, 5);
    assert_eq!(event_creatures(&wild, &part, &creatures), wild);
}
