//! Tests of the turn's start records (E31 T2d) on the town tests' game with values (red's T0 on
//! (1, 1), its entrance, and its hero with 9 imps on (0, 0); blue's T1 on (3, 0) and its hero
//! with 3 imps and a gog on (2, 2)), a second resource (wood), a Marketplace and the market's
//! rates: the whole records of both players, the threat fold's cases, the reserve's and the
//! demand's lists, the buildings the demand reads, the growth from the week's fifth day, the
//! weight factor's cases. The doubles' expectations come from an IEEE double model of the
//! reading (Python, not committed), compared exactly as integers scaled by 2^40.

use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{MarketParamsDef, ObjectKindDef, ObjectTypeDef, Pos, Visit};
use h3_content::{CreatureDef, ResourceDef, TerrainId};
use h3_towns::{
    BuildingDef, BuildingId, BuildingRoles, BuildingsSetup, GarrisonCreature, GarrisonStack,
    SlotSet, TownCommand, TownPack,
};

use super::*;
use crate::tests::key;
use crate::town::tests::{rules_with_adventure, town_pack, towns_core_pack, towns_setup};
use crate::{Actor, GameCommand, GameEvent, GameSetup, apply};

const RED: PlayerId = PlayerId::new(0);

/// The artifacts' mean of a core with no artifact from 7 on (the town fixtures' cores): the FPU's
/// 0 / 0 as an f32 (CPU-STA-008).
const NO_ARTIFACTS: u32 = 0xFFC0_0000;
const BLUE: PlayerId = PlayerId::new(1);
const T0: TownId = TownId::new(0);
const T1: TownId = TownId::new(1);
const RED_HERO: HeroId = HeroId::new(0);
const BLUE_HERO: HeroId = HeroId::new(1);
const IMP: usize = 0;
const GOG: usize = 1;
const CART: usize = 2;
const ELF: usize = 3;
/// The town record's Capitol (`b3`, hall 4) and its map slot.
const CAPITOL: (usize, u8) = (3, 13);
/// The town record's Town Hall (`b1`, hall 2).
const TOWN_HALL: usize = 1;

/// The town tests' rules with values and wood (the imp: Fight Value 100, AI value 70 000, 50
/// gold; the gog: 300, 300, 100 gold and 1 wood; the cart: 0, 10, 10 gold; an elf of no faction:
/// 200, 700, 200 gold and 2 wood), a Marketplace (500 gold, 5 wood) beside the town record's
/// buildings with `edit` made to them, a wood mine and the market's parameters (gold 1, wood 250;
/// the base pack's resource rates).
fn rules_with(edit: impl FnOnce(&mut TownPack)) -> GameRuleset {
    let mut core = towns_core_pack(false);
    core.resources.push(ResourceDef {
        key: key("t:wood"),
        name: "Wood".into(),
    });
    let mut elf = CreatureDef {
        key: key("t:elf"),
        ..core.creatures[IMP].clone()
    };
    elf.info.faction = None;
    core.creatures.push(elf);
    let values = [(100, 70_000), (300, 300), (0, 10), (200, 700)];
    for (creature, (fight, ai)) in core.creatures.iter_mut().zip(values) {
        creature.info.fight_value = fight;
        creature.info.ai_value = ai;
    }
    core.creatures[GOG].info.cost.push((key("t:wood"), 1));
    core.creatures[CART].info.cost = vec![(key("t:gold"), 10)];
    core.creatures[ELF].info.cost = vec![(key("t:gold"), 200), (key("t:wood"), 2)];
    let mut towns = town_pack(false);
    towns.towns[0].buildings.push(BuildingDef {
        key: "trading_post".into(),
        name: "Marketplace".into(),
        slot: None,
        supersedes: vec![],
        forbidden_with: None,
        auto_with: vec![],
        requires: vec![],
        cost: vec![(key("t:wood"), 5), (key("t:gold"), 500)],
        roles: BuildingRoles {
            marketplace: true,
            ..BuildingRoles::default()
        },
        war_machine: None,
    });
    edit(&mut towns);
    rules_with_adventure(core, &towns, |adventure| {
        adventure.object_types.push(ObjectTypeDef {
            key: key("t:wood_mine"),
            name: "Mine".into(),
            visit: Visit::Enter,
            from_above: true,
            kind_mark: false,
            kind: ObjectKindDef::Mine {
                resource: key("t:wood"),
                daily: 2,
                battle_terrain: None,
            },
        });
        adventure.params.as_mut().unwrap().market = Some(MarketParamsDef {
            cost: key("t:gold"),
            values: vec![(key("t:gold"), 1), (key("t:wood"), 250)],
            artifact_values: vec![(key("t:gold"), 1), (key("t:wood"), 125)],
            resources: vec![10, 15, 20, 25, 30, 35, 40, 45, 50, 50],
            artifacts: vec![10; 10],
            creatures: vec![10; 10],
        });
    })
}

fn rules() -> GameRuleset {
    rules_with(|_| {})
}

/// The town tests' game on `rules` with `edit` made to its setup.
fn game(rules: &GameRuleset, edit: impl FnOnce(&mut GameSetup)) -> GameState {
    let mut setup = towns_setup(rules);
    edit(&mut setup);
    GameState::new(rules, &setup, 3).unwrap()
}

/// Both towns red's.
fn both_red(setup: &mut GameSetup) {
    setup.map.objects[1].owner = Some(RED);
}

/// `command` of `player`, which must apply.
fn run(state: &mut GameState, rules: &GameRuleset, player: PlayerId, command: GameCommand) {
    let mut events: Vec<GameEvent> = Vec::new();
    apply(state, rules, Actor::Player(player), command, &mut events).unwrap();
}

/// A double compared exactly: the value times 2^40, truncated.
fn scaled(value: SignedX87) -> i64 {
    value.times(SignedX87::int(1 << 40)).trunc()
}

/// A hero's values per unit without power (no spell book): `experience` and the spell gains'
/// floors (CPU-STA-007).
fn bookless(experience: u32) -> HeroValues {
    HeroValues {
        experience,
        spells: SpellGains {
            power: 10,
            rounds: 0,
            knowledge: 10,
            doubled: 0,
            refilled: 0,
        },
    }
}

/// The weights of `record` compared exactly ([`scaled`]).
fn weights_of(record: &Record, rules: &GameRuleset) -> Vec<i64> {
    record.weights(rules).into_iter().map(scaled).collect()
}

// covers: CPU-STA-001, CPU-STA-002, CPU-STA-003, CPU-STA-004, CPU-STA-005, CPU-STA-006
#[test]
fn each_players_start_records_its_reserve_weights_experience_and_threats() {
    let rules = rules();
    let game = game(&rules, |_| {});
    // Red: T0 with 15 imps available (one stack: no reserve, the loop's unsigned bound), 2 000
    // gold and 500 a day; the demand the Marketplace's cost plus the imps' (15 × 50); blue's hero
    // takes T0's empty garrison from where it stands. Red's hero: 9 imps worth 630 000, 1 000
    // experience to its next level.
    let red = record(&game, &rules, RED);
    assert_eq!(
        red,
        Record {
            eyes: 0,
            reserve: vec![0, 0],
            means: vec![3000, 0],
            demand: vec![1250, 5],
            markets: 1,
            mean: 250,
            artifacts: NO_ARTIFACTS,
            heroes: vec![(RED_HERO, bookless(1_098_711_040))],
            threats: vec![(BLUE_HERO, 5_000_000)],
            towns: vec![(T0, 1)],
        }
    );
    // Gold: (1 750 × f32(0.1) + 1 250) / 3 000; wood: 5 over means of 0, below 1 / f32(0.1).
    assert_eq!(
        weights_of(&red, &rules),
        [522_268_024_149, 1_374_389_534_720_000]
    );
    // Blue: T1 with 15 imps and 8 gogs (750 gold, 800 gold and 8 wood); its wood factor 13 is
    // capped at 1 / f32(0.1), just below 10, so the mean sums 0 and 2 499.
    let blue = record(&game, &rules, BLUE);
    assert_eq!(
        blue,
        Record {
            eyes: 0,
            reserve: vec![1550, 8],
            means: vec![3000, 0],
            demand: vec![2050, 13],
            markets: 1,
            mean: 499,
            artifacts: NO_ARTIFACTS,
            heroes: vec![(BLUE_HERO, bookless(1_084_898_673))],
            threats: vec![(RED_HERO, 5_000_000)],
            towns: vec![(T1, 1)],
        }
    );
    assert_eq!(
        weights_of(&blue, &rules),
        [786_150_814_378, 2_748_779_028_480_000]
    );
}

// covers: CPU-STA-004, CPU-STA-005
#[test]
fn the_threat_fold_counts_each_town_an_enemy_would_take_from_where_it_reaches() {
    let rules = rules();
    // Both towns red's: blue's hero would take both, its threat 5 000 000 over two; blue, without
    // a town, counts no threat. Red's two towns: two incomes, both towns' creatures (30 imps, 8
    // gogs), the rate's row of two Marketplaces (f32(0.15), its cap 1 / f32(0.15) for wood).
    let both = game(&rules, both_red);
    let red = record(&both, &rules, RED);
    assert_eq!(red.threats, [(BLUE_HERO, 2_500_000)]);
    assert_eq!(red.towns, [(T0, 1), (T1, 1)]);
    assert_eq!(
        (red.means.clone(), red.demand.clone(), red.markets, red.mean),
        (vec![4000, 0], vec![2800, 13], 2, 333)
    );
    assert_eq!(
        weights_of(&red, &rules),
        [819_136_164_659, 1_832_519_306_808_891]
    );
    let blue = record(&both, &rules, BLUE);
    assert_eq!((blue.threats, blue.towns), (vec![], vec![]));
    // A garrison of 20 gogs holds T0 against blue's hero: T1 alone counts.
    let held = game(&rules, |setup| {
        both_red(setup);
        setup.towns[0].garrison = Some(vec![GarrisonStack {
            slot: 0,
            creature: GarrisonCreature::Creature(CreatureTypeId::from_raw(1)),
            count: 20,
        }]);
    });
    let red = record(&held, &rules, RED);
    assert_eq!(red.threats, [(BLUE_HERO, 2_500_000)]);
    assert_eq!(red.towns, [(T1, 1)]);
    // Walled in by rock on (0, 3), blue's hero reaches no town.
    let walled = game(&rules, |setup| {
        both_red(setup);
        setup.heroes[1].pos = Pos::new(0, 3, 0);
        for index in [8, 9, 13] {
            setup.map.tiles[index].terrain = TerrainId::from_raw(1);
        }
    });
    let red = record(&walled, &rules, RED);
    assert_eq!((red.threats, red.towns), (vec![], vec![]));
}

/// An entry of `creature` with `count` and the value `value`.
fn entry(creature: usize, value: i32, count: i16) -> Entry {
    Entry {
        creature,
        value,
        count,
    }
}

// covers: CPU-STA-002
#[test]
fn a_towns_reserve_is_its_two_highest_stacks_by_their_16_bit_values() {
    let rules = rules();
    // The imp's AI value 70 000 keeps 4 464 in 16 bits: 8 imps wrap to −29 824, 7 do not.
    assert_eq!(reserve_value(70_000, 7), 31_248);
    assert_eq!(reserve_value(70_000, 8), -29_824);
    assert_eq!(reserve_value(300, 3), 900);
    // 8 imps sort lowest, so the gogs and the carts count: 3 × 100 + 50 × 10 gold, 3 wood.
    let entries = vec![
        entry(IMP, reserve_value(70_000, 8), 8),
        entry(GOG, reserve_value(300, 3), 3),
        entry(CART, reserve_value(10, 50), 50),
    ];
    assert_eq!(two_dearest(&rules, entries), [800, 3]);
    // A single stack counts nothing; two count both.
    assert_eq!(two_dearest(&rules, vec![entry(GOG, 900, 3)]), [0, 0]);
    let two = vec![entry(GOG, 900, 3), entry(IMP, 31_248, 7)];
    assert_eq!(two_dearest(&rules, two), [650, 3]);
}

// covers: CPU-STA-003
#[test]
fn the_demand_adds_the_three_dearest_creatures_of_its_list() {
    let rules = rules();
    // Values: 2 imps 140 000, 5 elves 3 500, 50 carts 500, 1 gog 300: the gog is left out.
    let costs = three_dearest(&rules, vec![2, 1, 50, 5]);
    assert_eq!(costs, [2 * 50 + 50 * 10 + 5 * 200, 5 * 2]);
    // With fewer than three, the rest are counts of 0.
    assert_eq!(three_dearest(&rules, vec![0, 4, 0, 0]), [400, 4]);
}

// covers: CPU-STA-003
#[test]
fn the_demand_reads_what_a_town_could_build_now_its_days_mark_aside_and_no_held_capitol() {
    let rules = rules_with(|towns| {
        let buildings = &mut towns.towns[0].buildings;
        buildings[TOWN_HALL].cost = vec![(key("t:gold"), 1500)];
        buildings[CAPITOL.0].cost = vec![(key("t:gold"), 10_000)];
    });
    let mut state = game(&rules, |_| {});
    // The Capitol's 10 000 with the imps' 750.
    assert_eq!(record(&state, &rules, RED).demand, [10_750, 5]);
    // Red holding a Capitol in T1: T0's Capitol is not buildable, the Town Hall's 1 500 the most.
    let held = game(&rules, |setup| {
        both_red(setup);
        setup.towns[1].buildings = BuildingsSetup::Custom {
            built: SlotSet::EMPTY.with(CAPITOL.1),
            forbidden: SlotSet::EMPTY,
        };
    });
    let demand = record(&held, &rules, RED).demand;
    assert_eq!(demand[0], 1500 + 750);
    // The Town Hall built today: the hall offers nothing, the demand still reads the Capitol.
    let building = BuildingId::from_index(TOWN_HALL).unwrap();
    let build = GameCommand::Town(TownCommand::Build { town: T0, building });
    run(&mut state, &rules, RED, build);
    assert!(
        state
            .town_offer(&rules, RED, T0)
            .unwrap()
            .buildable
            .is_empty()
    );
    assert_eq!(record(&state, &rules, RED).demand, [10_750, 5]);
}

// covers: CPU-STA-003
#[test]
fn from_the_weeks_fifth_day_the_demand_counts_the_next_weeks_growth() {
    let rules = rules();
    let mut state = game(&rules, |_| {});
    let mut demands = Vec::new();
    while state.day() < 6 {
        demands.push((
            state.weekday(&rules).get(),
            record(&state, &rules, RED).demand,
        ));
        run(&mut state, &rules, RED, GameCommand::EndTurn);
        run(&mut state, &rules, BLUE, GameCommand::EndTurn);
    }
    // 15 imps available, from day 5 their dwelling's 15 more: 750 then 1 500 gold.
    let imps = |gold| vec![500 + gold, 5];
    assert_eq!(
        demands,
        [
            (1, imps(750)),
            (2, imps(750)),
            (3, imps(750)),
            (4, imps(750)),
            (5, imps(1500)),
        ]
    );
}

// covers: CPU-STA-003
#[test]
fn a_weights_factor_follows_the_demand_against_the_means() {
    let rate = SignedX87::from_f32_bits(0x3DCC_CCCD);
    let cases = [
        // No demand: the rate.
        ((3000, 0), 109_951_164_416),
        // At most the means: the surplus at the rate plus the demand, over the means.
        ((3000, 1250), 522_268_024_149),
        ((10, 7), 802_643_488_768),
        ((100, -7), 40_681_931_980),
        // More: the demand, over the means above 1, at most 1 / the rate.
        ((0, 5), 5_497_558_138_880),
        ((-5, 3), 3_298_534_883_328),
        ((1, 13), 10_995_116_113_920),
        ((2, 13), 7_146_825_580_544),
        ((4, 100), 10_995_116_113_920),
    ];
    for ((means, demand), expected) in cases {
        assert_eq!(
            scaled(factor(means, demand, rate)),
            expected,
            "{means}, {demand}"
        );
    }
    // Without a rate (no market's parameters): no cap, and a quotient by 0 is 0.
    assert_eq!(scaled(factor(0, 5, SignedX87::ZERO)), 5 << 40);
    assert_eq!(factor(0, -5, SignedX87::ZERO), SignedX87::ZERO);
    let towns = crate::town::tests::towns_rules();
    let record = Record {
        means: vec![3000],
        demand: vec![100],
        markets: 1,
        ..Record::default()
    };
    assert_eq!(record.weights(&towns), [SignedX87::ZERO]);
}

#[test]
fn greater_compares_signs_then_magnitudes() {
    let (one, two) = (SignedX87::ONE, SignedX87::int(2));
    assert!(greater(one, two.negated()));
    assert!(!greater(one.negated(), two));
    assert!(greater(two, one) && !greater(one, two) && !greater(one, one));
    assert!(greater(one.negated(), two.negated()) && !greater(two.negated(), one.negated()));
}
