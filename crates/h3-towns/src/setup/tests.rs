extern crate std;

use std::string::ToString;
use std::vec;
use std::vec::Vec;

use h3_kernel::StreamId;

use super::*;
use crate::fixture::*;
use crate::ruleset::GarrisonRow;

/// The setup stream of `seed`.
fn draws(seed: u64) -> Rng {
    Rng::for_stream(seed, StreamId::SETUP)
}

/// The Keep's buildings at the start from `setup`, owned when `owned`, in `part`.
fn keep_town(part: &TownRuleset, setup: BuildingsSetup, owned: bool, rng: &mut Rng) -> Town {
    let record = part.town(KEEP).unwrap();
    town_buildings_setup(record, part.params().unwrap(), &setup, owned, rng)
}

/// A custom set of the Keep's buildings at `built` and `forbidden`.
fn custom(built: &[usize], forbidden: &[usize]) -> BuildingsSetup {
    BuildingsSetup::Custom {
        built: slots(built),
        forbidden: slots(forbidden),
    }
}

/// The Keep's garrison at the start from `garrison`, owned when `owned`, in `part`.
fn keep_garrison(
    part: &TownRuleset,
    garrison: Option<&[GarrisonStack]>,
    owned: bool,
    rng: &mut Rng,
) -> Vec<ArmyStack> {
    let record = part.town(KEEP).unwrap();
    town_garrison(record, part.params().unwrap(), garrison, owned, rng)
}

fn stack(slot: u8, raw: u16, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature: creature(raw),
        count,
    }
}

#[test]
fn a_slot_set_holds_the_slots_below_41() {
    let set = SlotSet::EMPTY.with(0).with(40).with(41).with(255);
    assert_eq!(set.bits(), 1 | 1 << 40);
    assert!(set.contains(0) && set.contains(40) && !set.contains(1));
    assert!(!set.contains(41) && !set.contains(255));
    assert_eq!(SlotSet::from_bits(u64::MAX).bits(), (1 << 41) - 1);
    assert_eq!(SlotSet::from_bits(1 << 41 | 1 << 3), SlotSet::EMPTY.with(3));
    assert_eq!(SlotSet::default(), SlotSet::EMPTY);
}

#[test]
fn a_slot_names_the_factions_building_at_it_or_nothing() {
    // covers: TWN-SET-003
    let part = part();
    let keep = part.town(KEEP).unwrap();
    let at = slots(&[DWELLING_1, TAVERN]).with(NO_BUILDING_SLOT);
    assert_eq!(at.buildings(keep), set(&[TAVERN, DWELLING_1]));
    assert!(SlotSet::EMPTY.buildings(keep).is_empty());
    // The Marsh's buildings have no slot: no slot names one.
    let marsh = part.town(MARSH).unwrap();
    assert!(SlotSet::from_bits(u64::MAX).buildings(marsh).is_empty());
}

#[test]
fn a_custom_set_builds_its_slots_with_what_they_supersede_and_the_village_hall() {
    // covers: TWN-SET-003
    let part = part();
    let mut rng = draws(1);
    let before = rng.clone();
    let town = keep_town(&part, custom(&[CAPITOL, CASTLE], &[]), true, &mut rng);
    let halls = [VILLAGE_HALL, TOWN_HALL, CITY_HALL, CAPITOL];
    assert_eq!(town.faction(), KEEP);
    assert_eq!(
        town.built(),
        set(&halls).union(set(&[FORT, CITADEL, CASTLE]))
    );
    assert!(town.forbidden().is_empty());
    // The Village Hall without its slot; a slot without a building builds nothing.
    let tavern = BuildingsSetup::Custom {
        built: slots(&[TAVERN]).with(NO_BUILDING_SLOT),
        forbidden: SlotSet::EMPTY.with(NO_BUILDING_SLOT),
    };
    let town = keep_town(&part, tavern, false, &mut rng);
    assert_eq!(town.built(), set(&[VILLAGE_HALL, TAVERN]));
    assert!(town.forbidden().is_empty());
    // A custom set draws nothing.
    assert_eq!(rng, before);
}

#[test]
fn a_custom_sets_forbidden_buildings_close_over_forbidden_with_and_are_not_built() {
    // covers: TWN-SET-003
    let part = part();
    let mut rng = draws(2);
    // The horde forbids its upgrade, which forbids the shrine: two steps.
    let town = keep_town(&part, custom(&[], &[HORDE_1]), true, &mut rng);
    assert_eq!(town.forbidden(), set(&[HORDE_1, UPG_HORDE_1, SHRINE]));
    assert_eq!(town.built(), set(&[VILLAGE_HALL]));
    // A forbidden building is not built, neither as a slot's nor as one a slot's supersedes.
    let setup = custom(&[HORDE_1, UPG_DWELLING_1, CASTLE], &[CITADEL, HORDE_1]);
    let town = keep_town(&part, setup, true, &mut rng);
    let built = [VILLAGE_HALL, FORT, CASTLE, DWELLING_1, UPG_DWELLING_1];
    assert_eq!(town.built(), set(&built));
    let forbidden = [CITADEL, HORDE_1, UPG_HORDE_1, SHRINE];
    assert_eq!(town.forbidden(), set(&forbidden));
    // The Village Hall is built even when forbidden (the original builds it first).
    let town = keep_town(&part, custom(&[], &[VILLAGE_HALL]), true, &mut rng);
    assert_eq!(town.built(), set(&[VILLAGE_HALL]));
    assert_eq!(town.forbidden(), set(&[VILLAGE_HALL]));
}

#[test]
fn twn_quirk_5_a_custom_sets_horde_never_leaves_its_dwelling_standing() {
    // covers: TWN-SET-003
    // TWN-QUIRK-5's regression test: the original's custom set builds the horde before the
    // dwelling its supersede set brings in, and both stand; here the dwelling is ever built but
    // the horde supersedes it, so the horde stands alone.
    let part = part();
    let keep = part.town(KEEP).unwrap();
    let town = keep_town(&part, custom(&[HORDE_1], &[]), true, &mut draws(3));
    assert_eq!(town.built(), set(&[VILLAGE_HALL, HORDE_1, DWELLING_1]));
    assert!(
        town.built()
            .contains(BuildingId::from_index(DWELLING_1).unwrap())
    );
    assert_eq!(town.standing(keep), set(&[VILLAGE_HALL, HORDE_1]));
}

#[test]
fn a_horde_and_its_levels_upgraded_dwelling_stand_as_the_upgraded_horde_alone() {
    // covers: TWN-SET-003
    let part = part();
    let keep = part.town(KEEP).unwrap();
    let town = keep_town(
        &part,
        custom(&[HORDE_1, UPG_DWELLING_1], &[]),
        true,
        &mut draws(4),
    );
    let level_1 = [HORDE_1, DWELLING_1, UPG_DWELLING_1, UPG_HORDE_1];
    // The upgraded horde, then the shrine it brings (the fixture's second step).
    assert_eq!(
        town.built(),
        set(&level_1).union(set(&[VILLAGE_HALL, SHRINE]))
    );
    assert_eq!(
        town.standing(keep),
        set(&[VILLAGE_HALL, UPG_HORDE_1, SHRINE])
    );
    // A building forbidden is never built by the upgrade: the shrine at a slot of its own.
    let mut record = keep.clone();
    record.buildings[SHRINE].slot = Some(39);
    let setup = BuildingsSetup::Custom {
        built: slots(&[HORDE_1, UPG_DWELLING_1]),
        forbidden: SlotSet::EMPTY.with(39),
    };
    let params = part.params().unwrap();
    let town = town_buildings_setup(&record, params, &setup, true, &mut draws(4));
    assert_eq!(town.built(), set(&level_1).union(set(&[VILLAGE_HALL])));
    assert_eq!(town.forbidden(), set(&[SHRINE]));
    // The horde alone, or the upgraded dwelling alone, upgrades nothing.
    let town = keep_town(&part, custom(&[UPG_DWELLING_1], &[]), true, &mut draws(4));
    assert_eq!(
        town.built(),
        set(&[VILLAGE_HALL, DWELLING_1, UPG_DWELLING_1])
    );
}

#[test]
fn an_owned_standard_town_without_a_fort_gets_a_tavern_and_draws_nothing() {
    // covers: TWN-SET-003
    let part = part();
    let mut rng = draws(5);
    let before = rng.clone();
    let town = keep_town(
        &part,
        BuildingsSetup::Standard { fort: false },
        true,
        &mut rng,
    );
    // The Tavern, not the Brotherhood that supersedes it; no dwelling, no draw.
    assert_eq!(town.built(), set(&[VILLAGE_HALL, TAVERN]));
    assert!(town.forbidden().is_empty());
    assert_eq!(rng, before);
}

#[test]
fn an_owned_standard_town_with_a_fort_draws_its_level_2_dwelling() {
    // covers: TWN-SET-003
    let fort = BuildingsSetup::Standard { fort: true };
    let base = [VILLAGE_HALL, FORT, TAVERN, DWELLING_1];
    for (chance, built) in [(100, set(&base).union(set(&[DWELLING_2]))), (0, set(&base))] {
        let part = part_with(|p| p.level_2_dwelling_chance = chance);
        let mut rng = draws(6);
        let mut reference = rng.clone();
        reference.range_inclusive(1, 100);
        assert_eq!(keep_town(&part, fort, true, &mut rng).built(), built);
        // One draw either way.
        assert_eq!(rng, reference);
    }
    // At the part's 30 in 100, the dwelling exactly when the draw is at most 30.
    let part = part();
    let two = BuildingId::from_index(DWELLING_2).unwrap();
    let outcomes: Vec<bool> = (0..20)
        .map(|seed| {
            let mut rng = draws(seed);
            let roll = rng.clone().range_inclusive(1, 100);
            let built = keep_town(&part, fort, true, &mut rng).built().contains(two);
            assert_eq!(built, roll <= 30);
            built
        })
        .collect();
    assert!(outcomes.contains(&true) && outcomes.contains(&false));
}

#[test]
fn a_neutral_standard_town_gets_both_dwellings_fort_or_not_without_a_draw() {
    // covers: TWN-SET-003
    let part = part_with(|p| p.level_2_dwelling_chance = 0);
    let mut rng = draws(7);
    let before = rng.clone();
    let without = keep_town(
        &part,
        BuildingsSetup::Standard { fort: false },
        false,
        &mut rng,
    );
    assert_eq!(
        without.built(),
        set(&[VILLAGE_HALL, DWELLING_1, DWELLING_2])
    );
    let with = keep_town(
        &part,
        BuildingsSetup::Standard { fort: true },
        false,
        &mut rng,
    );
    assert_eq!(
        with.built(),
        set(&[VILLAGE_HALL, FORT, DWELLING_1, DWELLING_2])
    );
    assert!(with.forbidden().is_empty());
    assert_eq!(rng, before);
}

#[test]
fn a_standard_set_builds_only_what_the_record_has() {
    // covers: TWN-SET-003
    // The Marsh has its Village Hall and no Fort, Tavern or dwelling: an owned town with the
    // map's Fort still draws for the level-2 dwelling, the original's path.
    let part = part();
    let marsh = part.town(MARSH).unwrap();
    let params = part.params().unwrap();
    let mut rng = draws(8);
    let mut reference = rng.clone();
    reference.range_inclusive(1, 100);
    let fort = BuildingsSetup::Standard { fort: true };
    let town = town_buildings_setup(marsh, params, &fort, true, &mut rng);
    assert_eq!((town.faction(), town.built()), (MARSH, set(&[0])));
    assert_eq!(rng, reference);
    let neutral = town_buildings_setup(marsh, params, &fort, false, &mut rng);
    assert_eq!(neutral.built(), set(&[0]));
    // A record without a Village Hall builds none.
    let mut record = marsh.clone();
    record.buildings[0].roles.hall = None;
    let town = town_buildings_setup(&record, params, &fort, false, &mut rng);
    assert!(town.built().is_empty());
}

#[test]
fn a_custom_garrison_is_the_maps_without_a_draw() {
    // covers: TWN-SET-004
    let part = part();
    let random = |slot, level, upgraded, count| GarrisonStack {
        slot,
        creature: GarrisonCreature::Random { level, upgraded },
        count,
    };
    let garrison = [
        GarrisonStack {
            slot: 0,
            creature: GarrisonCreature::Creature(creature(99)),
            count: 5,
        },
        random(1, 1, true, 3),
        // Level 3 has no upgrade: its base.
        random(2, 3, true, 2),
        random(3, 2, false, 4),
        // A level the Keep lacks, level 0, a count of 0: each slot empty.
        random(4, 7, false, 1),
        random(5, 0, false, 1),
        GarrisonStack {
            slot: 6,
            creature: GarrisonCreature::Creature(creature(98)),
            count: 0,
        },
    ];
    let mut rng = draws(9);
    let before = rng.clone();
    let army = keep_garrison(&part, Some(&garrison), false, &mut rng);
    let expected = [
        stack(0, 99, 5),
        stack(1, 11, 3),
        stack(2, 30, 2),
        stack(3, 20, 4),
    ];
    assert_eq!(army, expected);
    assert_eq!(
        keep_garrison(&part, Some(&garrison), true, &mut rng),
        expected
    );
    assert_eq!(rng, before);
}

#[test]
fn a_custom_empty_garrison_stays_empty_and_an_owned_towns_default_one_too() {
    // covers: TWN-SET-004
    let part = part_with(|p| p.neutral_garrison[0].chance = 100);
    let mut rng = draws(10);
    let before = rng.clone();
    // TD-5: a neutral town's explicitly empty garrison is not drawn.
    assert!(keep_garrison(&part, Some(&[]), false, &mut rng).is_empty());
    assert!(keep_garrison(&part, None, true, &mut rng).is_empty());
    assert_eq!(rng, before);
}

#[test]
fn a_neutral_towns_default_garrison_is_drawn_level_by_level() {
    // covers: TWN-SET-004
    let part = part_with(|p| {
        p.neutral_garrison = [(100, [8, 15]), (0, [5, 7]), (100, [3, 5]), (100, [1, 3])]
            .map(|(chance, count)| GarrisonRow { chance, count });
    });
    let mut rng = draws(11);
    let mut reference = rng.clone();
    let mut roll = |lo, hi| reference.range_inclusive(lo, hi);
    roll(1, 100);
    let first = roll(8, 15);
    roll(1, 100);
    roll(1, 100);
    let third = roll(3, 5);
    roll(1, 100);
    let fourth = roll(1, 3);
    let army = keep_garrison(&part, None, false, &mut rng);
    // Level 2 missed: levels 3 and 4 take the next free slots.
    assert_eq!(
        army,
        [
            stack(0, 10, first),
            stack(1, 30, third),
            stack(2, 40, fourth)
        ]
    );
    assert_eq!(rng, reference);
    assert!((8..=15).contains(&first) && (3..=5).contains(&third) && (1..=3).contains(&fourth));
    // Every chance 0: four draws, no stack.
    let none = part_with(|p| p.neutral_garrison.iter_mut().for_each(|row| row.chance = 0));
    let mut rng = draws(11);
    let mut reference = rng.clone();
    (0..4).for_each(|_| {
        reference.range_inclusive(1, 100);
    });
    assert!(keep_garrison(&none, None, false, &mut rng).is_empty());
    assert_eq!(rng, reference);
}

#[test]
fn a_neutral_towns_drawn_garrison_has_only_the_records_levels() {
    // covers: TWN-SET-004
    let part = part_with(|p| {
        p.neutral_garrison
            .iter_mut()
            .for_each(|row| row.chance = 100)
    });
    let marsh = part.town(MARSH).unwrap();
    let mut rng = draws(12);
    let mut reference = rng.clone();
    reference.range_inclusive(1, 100);
    let count = reference.range_inclusive(1, 3);
    let army = town_garrison(marsh, part.params().unwrap(), None, false, &mut rng);
    assert_eq!(army, [stack(0, 50, count)]);
    assert_eq!(rng, reference);
}

#[test]
fn the_whole_town_draws_its_garrison_then_its_buildings() {
    // covers: TWN-SET-003, TWN-SET-004
    let part = part();
    let event = TownEventSetup {
        players: vec![PlayerId::new(1)],
        human: true,
        computer: false,
        first_day: 3,
        interval: 7,
        resources: vec![(GOLD, 500), (WOOD, -2)],
        buildings: slots(&[TAVERN, HORDE_1]).with(NO_BUILDING_SLOT),
        creatures: [1, 0, 0, 0, 0, 0, 2],
    };
    let mut setup = TownSetup {
        faction: TownFaction::Random { alignment: Some(2) },
        buildings: BuildingsSetup::Standard { fort: true },
        garrison: None,
        name: Some("Keepstead".into()),
        events: vec![event.clone(), TownEventSetup::default()],
    };
    let record = part.town(KEEP).unwrap();
    let params = part.params().unwrap();
    for owned in [false, true] {
        let mut rng = draws(13);
        let mut reference = rng.clone();
        let garrison = town_garrison(record, params, None, owned, &mut reference);
        let town = town_buildings_setup(record, params, &setup.buildings, owned, &mut reference);
        let built = town_setup(&setup, KEEP, owned, &part, &mut rng).unwrap();
        assert_eq!((&built.town, &built.garrison), (&town, &garrison));
        assert_eq!(rng, reference);
        assert_eq!(built.town_static.name.as_deref(), Some("Keepstead"));
        let events = &built.town_static.events;
        assert_eq!(events.len(), 2);
        assert_eq!(events[1], TownEvent::default());
        let first = &events[0];
        assert_eq!(first.buildings, set(&[TAVERN, HORDE_1]));
        assert_eq!(
            (&first.players, first.human, first.computer),
            (&event.players, true, false)
        );
        assert_eq!((first.first_day, first.interval), (3, 7));
        assert_eq!(
            (&first.resources, first.creatures),
            (&event.resources, event.creatures)
        );
    }
    // A custom garrison and set, a fixed faction, no name.
    setup.faction = TownFaction::Fixed(KEEP);
    setup.garrison = Some(vec![GarrisonStack {
        slot: 4,
        creature: GarrisonCreature::Random {
            level: 4,
            upgraded: true,
        },
        count: 9,
    }]);
    setup.buildings = custom(&[TAVERN], &[]);
    setup.name = None;
    let built = town_setup(&setup, KEEP, false, &part, &mut draws(14)).unwrap();
    assert_eq!(built.garrison, [stack(4, 41, 9)]);
    assert_eq!(built.town.built(), set(&[VILLAGE_HALL, TAVERN]));
    assert!(built.town_static.name.is_none());
}

#[test]
fn a_faction_without_a_town_record_or_parameters_cannot_be_set_up() {
    let setup = TownSetup {
        faction: TownFaction::Fixed(WILD),
        buildings: BuildingsSetup::Standard { fort: true },
        garrison: None,
        name: None,
        events: vec![],
    };
    let mut rng = draws(15);
    let before = rng.clone();
    let error = town_setup(&setup, WILD, false, &part(), &mut rng).unwrap_err();
    assert_eq!(error, TownSetupError::NoTownRecord(WILD));
    assert_eq!(
        error.to_string(),
        "faction 2 has no town record in the town part"
    );
    // The empty part, and a part of records without parameters (only built so in tests).
    let empty = town_setup(&setup, KEEP, false, &empty_part(), &mut rng);
    assert_eq!(empty, Err(TownSetupError::NoTownRecord(KEEP)));
    let unparametrised = TownRuleset::new(&core(), None, part().towns().to_vec());
    let none = town_setup(&setup, KEEP, false, &unparametrised, &mut rng);
    assert_eq!(none, Err(TownSetupError::NoTownRecord(KEEP)));
    assert_eq!(rng, before);
}
