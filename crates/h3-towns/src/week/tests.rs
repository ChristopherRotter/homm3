//! Tests of the week's and the month's growth (E09 T25b1) on the fixture's Keep: level 1 (handles
//! 10 and 11) grows 14, level 2 (20) 9, level 3 (30) 7 without a dwelling built here; batch
//! TWN-3's rows 1 and 3 (survey `E09-growth.md` §17) with the fixture's growths, which are the
//! rows' towns'.

use alloc::vec::Vec;

use h3_kernel::hash::hash_of;
use h3_kernel::{Rng, StreamId};

use super::*;
use crate::fixture::{
    DWELLING_1, DWELLING_2, GRAIL, MARSH, UPG_DWELLING_1, WILD, core, core_with, creature,
    empty_part, part, part_with, town, town_of,
};
use crate::record::{Summoned, TownStatic};

/// Both month flags on: the faithful profile's (the original's rules).
const FAITHFUL: [&str; 2] = [
    "quirk.town.creature_month_exact",
    "quirk.town.neutral_plague_full_growth",
];

/// An owned town's inputs with nothing more.
const OWNED: GrowthInputs = GrowthInputs {
    owned: true,
    ..GrowthInputs::NEUTRAL
};

/// A building's week of the Keep's level 1 (base and upgrade), its bonus the base's growth.
const OF_LEVEL_1: WeekKind = WeekKind::Of {
    creatures: [Some(creature(10)), Some(creature(11))],
    bonus: 14,
};

/// `built`'s Keep town with `counts` at the indexes from 0 and a Portal of 3 of handle 50.
fn counted(built: &[usize], counts: &[(usize, u16)]) -> Town {
    counts
        .iter()
        .fold(town(built), |t, &(index, count)| {
            t.with_available(index, count)
        })
        .with_portal(Some(Summoned {
            creature: creature(50),
            count: 3,
        }))
}

/// The counts at `indexes` of `town`.
fn at<const N: usize>(town: &Town, indexes: [usize; N]) -> [u16; N] {
    indexes.map(|index| town.available(index))
}

#[test]
fn a_week_names_its_creatures_and_their_bonus() {
    let part = part();
    assert_eq!(WeekKind::default(), WeekKind::Normal);
    let (normal, griffin) = (WeekKind::Normal, WeekKind::Creature(creature(10)));
    assert!(!normal.names(creature(10)));
    assert!(griffin.names(creature(10)) && !griffin.names(creature(11)));
    assert!(OF_LEVEL_1.names(creature(10)) && OF_LEVEL_1.names(creature(11)));
    assert!(!OF_LEVEL_1.names(creature(20)));
    // A building's week of a level without an upgrade names its base alone.
    let alone = WeekKind::Of {
        creatures: [Some(creature(30)), None],
        bonus: 7,
    };
    assert!(alone.names(creature(30)) && !alone.names(creature(31)));
    // The part's 5 for a creature's week, the building's own bonus, none in a normal week or
    // without the part's parameters.
    assert_eq!(
        [
            normal.bonus(&part),
            griffin.bonus(&part),
            OF_LEVEL_1.bonus(&part)
        ],
        [0, 5, 14]
    );
    assert_eq!(griffin.bonus(&empty_part()), 0);
}

#[test]
fn every_week_s_kind_hashes_apart() {
    let kinds = [
        WeekKind::Normal,
        WeekKind::Creature(creature(10)),
        WeekKind::Creature(creature(11)),
        OF_LEVEL_1,
        WeekKind::Of {
            creatures: [Some(creature(10)), None],
            bonus: 14,
        },
        WeekKind::Of {
            creatures: [Some(creature(10)), Some(creature(11))],
            bonus: 15,
        },
    ];
    for (i, a) in kinds.iter().enumerate() {
        for b in &kinds[i + 1..] {
            assert_ne!(hash_of(a), hash_of(b), "{a:?} and {b:?}");
        }
    }
}

// covers: TWN-GRO-003
#[test]
fn a_week_adds_each_growing_index_s_growth_halved_without_an_owner() {
    let (part, core) = (part(), core());
    let before = counted(&[DWELLING_1, DWELLING_2], &[(0, 3), (1, 2), (2, 5)]);
    let normal = WeekKind::Normal;
    let owned = week_growth(&before, &part, &core, &normal, &OWNED);
    // Index 2 (level 3, no dwelling) grows nothing and keeps its count.
    assert_eq!(at(&owned, [0, 1, 2]), [17, 11, 5]);
    assert_eq!(owned.portal(), before.portal());
    let neutral = week_growth(&before, &part, &core, &normal, &GrowthInputs::NEUTRAL);
    // 14 ÷ 2, 9 ÷ 2 truncated.
    assert_eq!(at(&neutral, [0, 1, 2]), [10, 6, 5]);
    // The inputs reach the growth: the Statue's half on the owned town's 14 and 9.
    let statue = GrowthInputs {
        share_bp: 5_000,
        ..OWNED
    };
    let more = week_growth(&before, &part, &core, &normal, &statue);
    assert_eq!(at(&more, [0, 1]), [24, 15]);
}

// covers: TWN-GRO-003
#[test]
fn a_week_s_creature_grows_its_bonus_before_the_halving() {
    let (part, core) = (part(), core());
    let base = counted(&[DWELLING_1, DWELLING_2], &[]);
    let grow = |town: &Town, kind: &WeekKind, inputs: &GrowthInputs| {
        week_growth(town, &part, &core, kind, inputs)
    };
    let griffin = WeekKind::Creature(creature(10));
    assert_eq!(at(&grow(&base, &griffin, &OWNED), [0, 1]), [19, 9]);
    // (14 + 5) ÷ 2, not 14 ÷ 2 + 5.
    assert_eq!(
        at(&grow(&base, &griffin, &GrowthInputs::NEUTRAL), [0, 1]),
        [9, 4]
    );
    // A creature's week names its creature alone: the upgrade's misses a base dwelling, the
    // base's an upgraded one.
    let royal = WeekKind::Creature(creature(11));
    assert_eq!(at(&grow(&base, &royal, &OWNED), [0]), [14]);
    let upgraded = counted(&[DWELLING_1, UPG_DWELLING_1], &[]);
    assert_eq!(at(&grow(&upgraded, &griffin, &OWNED), [0, 7]), [0, 14]);
    assert_eq!(at(&grow(&upgraded, &royal, &OWNED), [0, 7]), [0, 19]);
    // A building's week gives either form of its level its own bonus.
    assert_eq!(at(&grow(&base, &OF_LEVEL_1, &OWNED), [0, 1]), [28, 9]);
    assert_eq!(at(&grow(&upgraded, &OF_LEVEL_1, &OWNED), [7]), [28]);
    assert_eq!(
        at(&grow(&upgraded, &OF_LEVEL_1, &GrowthInputs::NEUTRAL), [7]),
        [14]
    );
}

// covers: TWN-GRO-003
#[test]
fn a_week_s_count_is_held_at_the_bound_and_a_town_without_a_record_grows_nothing() {
    let (part, core) = (part(), core());
    let full = counted(&[DWELLING_1], &[(0, MAX_AVAILABLE - 3)]);
    let grown = week_growth(&full, &part, &core, &WeekKind::Normal, &OWNED);
    assert_eq!(at(&grown, [0]), [MAX_AVAILABLE]);
    let wild = town_of(WILD, &[]).with_available(0, 4);
    assert_eq!(
        week_growth(&wild, &part, &core, &WeekKind::Normal, &OWNED),
        wild
    );
}

// covers: TWN-GRO-004
#[test]
fn a_creature_s_month_doubles_its_creature_s_counts_after_the_week() {
    // Batch TWN-3's row 1 with the fixture's level 1: after the week K1 (upgraded) and K2 (base)
    // each hold 47; the month of the base creature doubles K2's alone (survey §17: 47 and 94).
    let (part, core) = (part(), core_with(&FAITHFUL));
    let k1 = counted(
        &[DWELLING_1, UPG_DWELLING_1, DWELLING_2],
        &[(7, 47), (1, 5)],
    );
    let k2 = counted(&[DWELLING_1, DWELLING_2], &[(0, 47), (1, 5)]);
    let griffin = MonthGrowth::Creature(creature(10));
    let normal = WeekKind::Normal;
    let after = |town: &Town| month(town, &part, &core, griffin, &normal, &OWNED);
    assert_eq!(at(&after(&k1), [0, 7, 1]), [0, 47, 5]);
    assert_eq!(at(&after(&k2), [0, 7, 1]), [94, 0, 5]);
    assert_eq!(after(&k2).portal(), k2.portal());
    // The upgrade's month doubles the upgraded dwelling's count alone; a neutral town's too.
    let royal = MonthGrowth::Creature(creature(11));
    let neutral = GrowthInputs::NEUTRAL;
    assert_eq!(
        at(&month(&k1, &part, &core, royal, &normal, &neutral), [7]),
        [94]
    );
    assert_eq!(
        at(&month(&k2, &part, &core, royal, &normal, &neutral), [0]),
        [47]
    );
    // Doubled, held at the bound.
    let full = counted(&[DWELLING_1], &[(0, 20_000)]);
    assert_eq!(
        at(&month(&full, &part, &core, griffin, &normal, &OWNED), [0]),
        [MAX_AVAILABLE]
    );
}

// covers: TWN-GRO-004
#[test]
fn twn_quirk_10_off_a_creature_s_month_doubles_either_form_of_its_level() {
    // The flags off: VCMI's level rule doubles K1's upgraded count too (row 1's refuted 94).
    let (part, core) = (part(), core());
    let k1 = counted(
        &[DWELLING_1, UPG_DWELLING_1, DWELLING_2],
        &[(7, 47), (1, 5)],
    );
    let k2 = counted(&[DWELLING_1, DWELLING_2], &[(0, 47), (1, 5)]);
    let normal = WeekKind::Normal;
    for creature in [creature(10), creature(11)] {
        let kind = MonthGrowth::Creature(creature);
        assert_eq!(
            at(&month(&k1, &part, &core, kind, &normal, &OWNED), [7, 1]),
            [94, 5]
        );
        assert_eq!(
            at(&month(&k2, &part, &core, kind, &normal, &OWNED), [0, 1]),
            [94, 5]
        );
    }
}

// covers: TWN-GRO-004
#[test]
fn a_building_s_week_and_a_normal_month_double_nothing() {
    let (part, core) = (part(), core_with(&FAITHFUL));
    let k2 = counted(&[DWELLING_1, DWELLING_2], &[(0, 47)]);
    let imp = MonthGrowth::Creature(creature(10));
    assert_eq!(month(&k2, &part, &core, imp, &OF_LEVEL_1, &OWNED), k2);
    let normal = MonthGrowth::default();
    assert_eq!(normal, MonthGrowth::Normal);
    assert_eq!(
        month(&k2, &part, &core, normal, &WeekKind::Normal, &OWNED),
        k2
    );
    // A town without a record keeps its counts in any month.
    let wild = town_of(WILD, &[]).with_available(0, 4);
    for kind in [imp, MonthGrowth::Plague] {
        assert_eq!(
            month(&wild, &part, &core, kind, &WeekKind::Normal, &OWNED),
            wild
        );
    }
}

// covers: TWN-GRO-004
#[test]
fn the_plague_takes_the_growth_then_halves() {
    // Batch TWN-3's row 3 (survey §17) with the fixture's 14 and 9: O1 (owned) 106 and 61 before
    // the week, 120 and 70 after it, 53 and 30 on the Plague's day; N1 (neutral) 42 and 25, 49
    // and 29 after its halved week, 17 and 10 (the whole growth taken off).
    let (part, core) = (part(), core_with(&FAITHFUL));
    let built = [DWELLING_1, DWELLING_2];
    let plague = |before: &[(usize, u16)], inputs: &GrowthInputs| {
        let week = week_growth(
            &counted(&built, before),
            &part,
            &core,
            &WeekKind::Normal,
            inputs,
        );
        let after = month(
            &week,
            &part,
            &core,
            MonthGrowth::Plague,
            &WeekKind::Normal,
            inputs,
        );
        assert_eq!(after.portal(), week.portal());
        (at(&week, [0, 1]), at(&after, [0, 1]))
    };
    assert_eq!(plague(&[(0, 106), (1, 61)], &OWNED), ([120, 70], [53, 30]));
    let neutral = GrowthInputs::NEUTRAL;
    assert_eq!(plague(&[(0, 42), (1, 25)], &neutral), ([49, 29], [17, 10]));
    // At least 0; an index that grows nothing keeps its count.
    let low = counted(&built, &[(0, 10), (2, 9)]);
    let after = month(
        &low,
        &part,
        &core,
        MonthGrowth::Plague,
        &WeekKind::Normal,
        &OWNED,
    );
    assert_eq!(at(&after, [0, 2]), [0, 9]);
}

// covers: TWN-GRO-004
#[test]
fn twn_quirk_9_off_every_town_keeps_half_its_count_before_the_week() {
    // The flags off: N1 loses what its week added, 42 ÷ 2 and 25 ÷ 2 (row 3's refuted reading is
    // VCMI's halving after the week; this is the fix's); an owned town as before.
    let (part, core) = (part(), core());
    let built = [DWELLING_1, DWELLING_2];
    let plague = |before: &[(usize, u16)], inputs: &GrowthInputs| {
        let week = week_growth(
            &counted(&built, before),
            &part,
            &core,
            &WeekKind::Normal,
            inputs,
        );
        at(
            &month(
                &week,
                &part,
                &core,
                MonthGrowth::Plague,
                &WeekKind::Normal,
                inputs,
            ),
            [0, 1],
        )
    };
    assert_eq!(
        plague(&[(0, 42), (1, 25)], &GrowthInputs::NEUTRAL),
        [21, 12]
    );
    assert_eq!(plague(&[(0, 106), (1, 61)], &OWNED), [53, 30]);
}

/// The towns' stream of `seed`, as the game's (E09 T26a2).
fn stream(seed: u64) -> Rng {
    Rng::for_stream(seed, StreamId::TOWNS)
}

/// A store of `towns` without their static parts.
fn store(towns: &[Town]) -> Towns {
    Towns::new(
        towns
            .iter()
            .map(|t| (t.clone(), TownStatic::default()))
            .collect(),
    )
    .unwrap()
}

#[test]
fn a_week_s_creatures_are_every_record_s_levels_and_on_roe_the_roe_records_alone() {
    let part = part();
    let all: Vec<u16> = week_creatures(&part, false)
        .iter()
        .map(|c| c.raw())
        .collect();
    // The Keep's levels, base then upgrade (level 3 has none), then the Marsh's (roe false).
    assert_eq!(all, [10, 11, 20, 21, 30, 40, 41, 50]);
    let roe: Vec<u16> = week_creatures(&part, true)
        .iter()
        .map(|c| c.raw())
        .collect();
    assert_eq!(roe, [10, 11, 20, 21, 30, 40, 41]);
    assert!(week_creatures(&empty_part(), false).is_empty());
}

#[test]
fn a_standing_week_of_building_makes_its_level_s_week() {
    let (core, part) = (core(), part());
    // The fixture's Grail is level 1's: the Keep's 10 and 11, the bonus 10's growth (14).
    assert_eq!(
        building_week(&town(&[GRAIL]), &part, &core),
        Some(OF_LEVEL_1)
    );
    assert_eq!(building_week(&town(&[DWELLING_1]), &part, &core), None);
    // The Marsh has none; a faction without a record none either.
    assert_eq!(building_week(&town_of(MARSH, &[0, 1]), &part, &core), None);
    assert_eq!(building_week(&town_of(WILD, &[GRAIL]), &part, &core), None);
}

#[test]
fn a_week_s_draw_is_a_creature_s_one_time_in_four_never_on_a_month_s_first() {
    let (core, part) = (core(), part());
    let towns = store(&[town(&[DWELLING_1])]);
    let mut creatures = 0;
    let mut named = [false; 8];
    for seed in 0..400 {
        let mut draws = stream(seed);
        let kind = draw_week(&towns, &part, &core, false, false, &mut draws);
        match kind {
            WeekKind::Normal => {
                // One draw: the chance's, failed.
                let mut once = stream(seed);
                assert!(once.below(4) > 0);
                assert_eq!(draws, once, "seed {seed}");
            }
            WeekKind::Creature(c) => {
                creatures += 1;
                let at = week_creatures(&part, false)
                    .iter()
                    .position(|&w| w == c)
                    .unwrap();
                named[at] = true;
                let mut twice = stream(seed);
                assert_eq!(twice.below(4), 0);
                assert_eq!(usize::try_from(twice.below(8)).unwrap(), at);
                assert_eq!(draws, twice, "seed {seed}");
            }
            WeekKind::Of { .. } => panic!("no week_of building stands"),
        }
        // A month's first week draws nothing.
        let mut first = stream(seed);
        assert_eq!(
            draw_week(&towns, &part, &core, false, true, &mut first),
            WeekKind::Normal
        );
        assert_eq!(first, stream(seed));
    }
    assert!(
        (70..=130).contains(&creatures),
        "{creatures} creature weeks in 400"
    );
    assert_eq!(named, [true; 8], "every eligible creature comes");
}

#[test]
fn on_roe_a_week_never_names_a_record_without_roe() {
    let core = core();
    // Every week a creature's.
    let part = part_with(|p| p.week.creature_chance = [1, 1]);
    let towns = store(&[town(&[])]);
    let (mut marsh_roe, mut marsh) = (false, false);
    for seed in 0..200 {
        let roe = draw_week(&towns, &part, &core, true, false, &mut stream(seed));
        let any = draw_week(&towns, &part, &core, false, false, &mut stream(seed));
        assert!(matches!(roe, WeekKind::Creature(_)) && matches!(any, WeekKind::Creature(_)));
        marsh_roe |= roe == WeekKind::Creature(creature(50));
        marsh |= any == WeekKind::Creature(creature(50));
    }
    assert!(!marsh_roe && marsh);
    // A part without records or parameters draws nothing.
    let mut draws = stream(1);
    let kind = draw_week(&store(&[]), &empty_part(), &core, false, false, &mut draws);
    assert_eq!((kind, draws), (WeekKind::Normal, stream(1)));
    // On roe with no roe record: the chance is drawn, no creature.
    let marsh_only = part_with_keep_roe_off();
    let mut draws = stream(1);
    let kind = draw_week(&towns, &marsh_only, &core, true, false, &mut draws);
    let mut once = stream(1);
    once.below(1);
    assert_eq!((kind, draws), (WeekKind::Normal, once));
}

/// The part whose Keep is not a Restoration of Erathia record either, every week a creature's.
fn part_with_keep_roe_off() -> TownRuleset {
    crate::fixture::part_changed(|p| p.week.creature_chance = [1, 1], |k| k.roe = false)
}

#[test]
fn the_first_week_of_building_overwrites_the_draw_every_week() {
    let (core, part) = (core(), part());
    // A town without one first, then a Grail town of any owner: its week, the month's first too.
    let towns = store(&[town(&[DWELLING_1]), town(&[GRAIL]), town(&[])]);
    for seed in 0..40 {
        for month_first in [false, true] {
            let mut draws = stream(seed);
            let kind = draw_week(&towns, &part, &core, false, month_first, &mut draws);
            assert_eq!(kind, OF_LEVEL_1);
            // The draw is made as without the building.
            let mut without = stream(seed);
            draw_week(
                &store(&[town(&[])]),
                &part,
                &core,
                false,
                month_first,
                &mut without,
            );
            assert_eq!(draws, without);
        }
    }
}
