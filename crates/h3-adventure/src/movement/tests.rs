//! Tests of a day's movement points (E08 T6b), the army's native terrain and the price of a step
//! (T7b), each from its source (survey `E08-adventure-core.md` §1.2–1.3); the hero's details in
//! them since T40 (survey `E08-hero-movement.md` §1, §2, §4).

use h3_content::CreatureTypeId;
use h3_contracts::ArmyStack;

use super::*;
use crate::ruleset::RoadId;
use crate::testing::{
    COBBLE_ROAD, DIRT_ROAD, FLYING, GNOLL, GRASS, LOGISTICS, PATHFINDING, PEASANT, PIKEMAN, ROCK,
    SAND, SCOUTING, SEA, SWAMP, WATER_WALKING, WOLF, army, hero_details, hero_rules,
    movement_rules, rules,
};

const IMP: CreatureTypeId = CreatureTypeId::from_raw(0);
const GOG: CreatureTypeId = CreatureTypeId::from_raw(1);

fn stack(slot: u8, creature: CreatureTypeId) -> ArmyStack {
    ArmyStack {
        slot,
        creature,
        count: 1,
    }
}

// covers: ADV-MOV-001
#[test]
fn the_slowest_creature_picks_the_land_tables_entry() {
    let (core, part) = rules();
    let points = |army: &[ArmyStack]| day_points(&core, &part, None, army);
    // Gogs (speed 2) slow the imps (speed 5) down to the table's third entry.
    assert_eq!(points(&[stack(0, IMP), stack(4, GOG)]), 1430);
    assert_eq!(points(&[stack(1, GOG)]), 1430);
    // Speed 5 is past the table: its last entry.
    assert_eq!(points(&[stack(0, IMP)]), 1500);
    // Neither an empty army nor an unknown creature slows an army down.
    assert_eq!(points(&[]), 1500);
    let ghost = stack(2, CreatureTypeId::from_raw(9));
    assert_eq!(points(&[ghost, stack(0, IMP)]), 1500);
    // A hero without bonuses: the table's entry alone, as without details.
    let plain = hero_details(&[]);
    let army = [stack(0, IMP), stack(4, GOG)];
    assert_eq!(land_table_points(&core, &part, Some(&plain), &army), 1430);
    assert_eq!(day_points(&core, &part, Some(&plain), &army), 1430);
}

// covers: ADV-MOV-001, ADV-FOG-001
#[test]
fn the_heros_details_grow_its_day_points_and_radius() {
    // E08 T40: `h3_heroes::land_points` on the table's entry (HER-ADV-003: Logistics +10 %, 20 %,
    // 30 %, rounded down), `h3_heroes::scouting_radius` on the part's 5 (HER-ADV-006), the
    // Pathfinding level for the price (HER-ADV-005). The creature specialty's speed and the
    // artifacts: `h3-game`'s tests (the store's hero types).
    let (core, part) = hero_rules(true);
    let pikemen = army(&[PIKEMAN]); // speed 4: 1500
    for (level, points, radius) in [(1, 1650, 6), (2, 1800, 7), (3, 1950, 8)] {
        let hero = hero_details(&[(LOGISTICS, level), (SCOUTING, level), (PATHFINDING, level)]);
        assert_eq!(land_table_points(&core, &part, Some(&hero), &pikemen), 1500);
        assert_eq!(day_points(&core, &part, Some(&hero), &pikemen), points);
        assert_eq!(hero_radius(&core, &part, Some(&hero)), radius);
        assert_eq!(hero_movement(&core, Some(&hero)).pathfinding, level);
    }
    assert_eq!(day_points(&core, &part, None, &pikemen), 1500);
    assert_eq!(hero_radius(&core, &part, None), 5);
    assert_eq!(hero_movement(&core, None), Movement::default());
    // The store by hero id: a hero it lacks has no details.
    let store = [Arc::new(hero_details(&[(SCOUTING, 1)]))];
    assert_eq!(details_of(&store, HeroId::new(0)), Some(&*store[0]));
    assert_eq!(details_of(&store, HeroId::new(1)), None);
}

fn tile(terrain: TerrainId, road: Option<RoadId>) -> StaticTile {
    StaticTile {
        terrain,
        road,
        river: None,
        blocked: false,
        visitable: false,
        no_native: false,
        winds: false,
        hole: false,
    }
}

/// [`tile`] under a ground that takes the native price away (Cursed Ground, E08 T40).
fn cursed(terrain: TerrainId, road: Option<RoadId>) -> StaticTile {
    StaticTile {
        no_native: true,
        ..tile(terrain, road)
    }
}

/// [`tile`] blocked by an object, which no object is visited from (an obstacle).
fn obstacle(terrain: TerrainId, road: Option<RoadId>) -> StaticTile {
    StaticTile {
        blocked: true,
        ..tile(terrain, road)
    }
}

/// [`tile`] as an object's visitable tile, blocked too (a pile's).
fn entrance(terrain: TerrainId) -> StaticTile {
    StaticTile {
        blocked: true,
        visitable: true,
        ..tile(terrain, None)
    }
}

/// The pricing of the steps of a hero with `army`, no Pathfinding and these flying and
/// water-walking levels (E08 T41).
fn lifted<'a>(
    core: &Ruleset,
    part: &'a AdventureRuleset,
    army: &[ArmyStack],
    flying: Option<u8>,
    water_walking: Option<u8>,
) -> Pricing<'a> {
    let movement = Movement {
        flying,
        water_walking,
        ..Movement::default()
    };
    Pricing::new(core, part, army, &movement)
}

/// The pricing of a walker's steps with `army` and no Pathfinding.
fn walking<'a>(core: &Ruleset, part: &'a AdventureRuleset, army: &[ArmyStack]) -> Pricing<'a> {
    Pricing::new(core, part, army, &Movement::default())
}

/// The pricing of a walker's steps with `army` and Pathfinding at `level`.
fn pathfinder<'a>(
    core: &Ruleset,
    part: &'a AdventureRuleset,
    army: &[ArmyStack],
    level: u8,
) -> Pricing<'a> {
    let movement = Movement {
        pathfinding: level,
        ..Movement::default()
    };
    Pricing::new(core, part, army, &movement)
}

// covers: ADV-MOV-004
#[test]
fn an_army_is_native_where_every_stack_is() {
    for faithful in [false, true] {
        let (core, _) = movement_rules(faithful);
        let native = |creatures: &[CreatureTypeId]| native_terrain(&core, &army(creatures));
        assert_eq!(native(&[PIKEMAN]), Some(GRASS));
        assert_eq!(native(&[PIKEMAN, PIKEMAN]), Some(GRASS));
        assert_eq!(native(&[GNOLL]), Some(SWAMP));
        // Two native terrains, or none at all.
        assert_eq!(native(&[PIKEMAN, GNOLL]), None);
        assert_eq!(native(&[]), None);
        assert_eq!(native(&[PEASANT]), None);
        assert_eq!(native(&[WOLF]), None);
        // A neutral stack after a faction stack cancels it in both profiles.
        assert_eq!(native(&[PIKEMAN, PEASANT]), None);
        assert_eq!(native(&[PEASANT, PIKEMAN, PEASANT]), None);
        // An unknown creature counts as neutral.
        let ghost = CreatureTypeId::from_raw(9);
        assert_eq!(native(&[PIKEMAN, ghost]), None);
    }
}

// covers: ADV-MOV-004
#[test]
fn leading_neutral_stacks_keep_the_native_terrain_under_the_quirk() {
    // ADV-QUIRK-4 (0x44C590): the slots in order, stacks without a native terrain before the
    // first faction stack skipped.
    let (faithful, _) = movement_rules(true);
    let (strict, _) = movement_rules(false);
    for creatures in [&[PEASANT, PIKEMAN][..], &[PEASANT, WOLF, PIKEMAN, PIKEMAN]] {
        assert_eq!(native_terrain(&faithful, &army(creatures)), Some(GRASS));
        assert_eq!(native_terrain(&strict, &army(creatures)), None);
    }
    assert_eq!(native_terrain(&faithful, &army(&[PEASANT, PEASANT])), None);
}

// covers: ADV-MOV-005
#[test]
fn a_diagonal_costs_the_straight_cost_times_root_two_rounded_down() {
    // The wiki's and VCMI's diagonal costs for the terrains and roads.
    for (straight, diagonal) in [
        (100, 141),
        (125, 176),
        (150, 212),
        (175, 247),
        (75, 106),
        (65, 91),
        (50, 70),
    ] {
        assert_eq!(diagonal_cost(straight), diagonal);
    }
    assert_eq!(diagonal_cost(0), 0);
    // Costs are at most u16::MAX.
    assert_eq!(diagonal_cost(u32::MAX), diagonal_cost(u32::from(u16::MAX)));
    assert_eq!(diagonal_cost(u32::from(u16::MAX)), 92_680);
    // The fixed-point product is the integer root of 2 · straight² for every cost (E08 T13b).
    for straight in 0..=u64::from(u16::MAX) {
        let exact = u32::try_from((2 * straight * straight).isqrt()).unwrap();
        assert_eq!(diagonal_cost(u32::try_from(straight).unwrap()), exact);
    }
}

// covers: ADV-MOV-007
#[test]
fn a_step_goes_to_one_of_the_eight_tiles_around_on_the_same_level() {
    let at = Pos::new(5, 5, 0);
    let around = [
        (5, 4),
        (6, 4),
        (6, 5),
        (6, 6),
        (5, 6),
        (4, 6),
        (4, 5),
        (4, 4),
    ];
    for (dir, (x, y)) in around.into_iter().enumerate() {
        assert_eq!(direction(at, Pos::new(x, y, 0)), Some(dir));
    }
    assert_eq!(direction(at, at), None);
    assert_eq!(direction(at, Pos::new(7, 5, 0)), None);
    assert_eq!(direction(at, Pos::new(5, 4, 1)), None);
    assert_eq!(direction(Pos::new(0, 0, 0), Pos::new(u16::MAX, 0, 0)), None);
}

// covers: ADV-MOV-003
#[test]
fn only_the_source_tile_prices_a_straight_step() {
    let (core, part) = movement_rules(false);
    let pricing = walking(&core, &part, &army(&[PEASANT]));
    let price = |from: StaticTile, to: StaticTile| pricing.step(&from, &to, false, 1500);
    let (grass, sand, swamp) = (tile(GRASS, None), tile(SAND, None), tile(SWAMP, None));
    assert_eq!(price(grass, grass), 100);
    assert_eq!(price(sand, grass), 150);
    assert_eq!(price(grass, swamp), 100);
    assert_eq!(price(swamp, sand), 175);
    // Along a road (both tiles have one) the source's road prices it.
    let (dirt, cobble) = (tile(GRASS, Some(DIRT_ROAD)), tile(SAND, Some(COBBLE_ROAD)));
    assert_eq!(price(dirt, cobble), 75);
    assert_eq!(price(cobble, dirt), 50);
    // Off a road: the terrain under the road (D-2, the code reading and VCMI; not the wiki's
    // destination terrain), 100 from a road on grass onto swamp.
    assert_eq!(price(dirt, swamp), 100);
    assert_eq!(price(cobble, grass), 150);
    // Onto a road: the source terrain.
    assert_eq!(price(swamp, cobble), 175);
    // Water prices by its own cost (by boat, after increment 1); a tile no hero stands on as the
    // dearest cost.
    assert_eq!(price(tile(SEA, None), grass), 100);
    assert_eq!(price(tile(ROCK, None), grass), u32::from(u16::MAX));
    assert_eq!(
        price(tile(TerrainId::from_raw(99), None), grass),
        u32::from(u16::MAX)
    );
}

// covers: ADV-MOV-004
#[test]
fn a_native_army_steps_from_its_terrain_at_the_native_cost() {
    let (core, part) = movement_rules(false);
    let swamp = tile(SWAMP, None);
    let grass = tile(GRASS, None);
    let gnolls = walking(&core, &part, &army(&[GNOLL, GNOLL]));
    assert_eq!(gnolls.step(&swamp, &grass, false, 1500), 100);
    assert_eq!(gnolls.step(&swamp, &grass, true, 1500), 141);
    // Grass is not theirs: its own cost, which is 100 too; pikemen pay swamp's.
    assert_eq!(gnolls.step(&grass, &swamp, false, 1500), 100);
    let pikemen = walking(&core, &part, &army(&[PIKEMAN]));
    assert_eq!(pikemen.step(&swamp, &grass, false, 1500), 175);
    // A road still prices a road step.
    let road = tile(SWAMP, Some(DIRT_ROAD));
    assert_eq!(gnolls.step(&road, &road, false, 1500), 75);
}

// covers: ADV-MOV-005, ADV-MOV-006
#[test]
fn the_diagonal_exception_charges_the_straight_cost_up_to_its_bound() {
    // Grass, no road, no native army: the check is 100 in both profiles. With 100 to 141
    // points (the bound is ⌊100 × √2⌋, inclusive: 0x4B1740's x87 compare) the diagonal costs
    // 100; below, or above, 141.
    for faithful in [false, true] {
        let (core, part) = movement_rules(faithful);
        let pricing = walking(&core, &part, &army(&[PEASANT]));
        let grass = tile(GRASS, None);
        let price = |points| pricing.step(&grass, &grass, true, points);
        assert_eq!(price(99), 141);
        assert_eq!(price(100), 100);
        assert_eq!(price(141), 100);
        assert_eq!(price(142), 141);
        assert_eq!(price(1500), 141);
        // A straight step never takes the exception.
        assert_eq!(pricing.step(&grass, &grass, false, 120), 100);
    }
}

// covers: ADV-MOV-006
#[test]
fn the_exceptions_check_ignores_roads_under_adv_quirk_2() {
    // A diagonal cobblestone-road step over grass: straight 50, diagonal 70. Clean, the check is
    // the step's own 50 (exception from 50 to 70 points); the original checks grass's 100
    // (exception from 100 to 141).
    let road = tile(GRASS, Some(COBBLE_ROAD));
    let (core, part) = movement_rules(false);
    let strict = walking(&core, &part, &army(&[PEASANT]));
    let (core, part) = movement_rules(true);
    let faithful = walking(&core, &part, &army(&[PEASANT]));
    for (points, clean, original) in [
        (60, 50, 70),
        (70, 50, 70),
        (71, 70, 70),
        (120, 70, 50),
        (141, 70, 50),
        (142, 70, 70),
    ] {
        assert_eq!(strict.step(&road, &road, true, points), clean, "{points}");
        assert_eq!(
            faithful.step(&road, &road, true, points),
            original,
            "{points}"
        );
    }
}

// covers: ADV-MOV-006
#[test]
fn the_exceptions_check_ignores_the_native_cost_under_adv_quirk_3() {
    // Gnolls on swamp: a straight step costs the native 100, a diagonal 141. Clean, the check is
    // 100 (exception from 100 to 141 points); the original checks swamp's 175, so with 100 to
    // 174 points the diagonal costs 141, and from 175 to 247 only 100.
    let swamp = tile(SWAMP, None);
    let (core, part) = movement_rules(false);
    let strict = walking(&core, &part, &army(&[GNOLL]));
    let (core, part) = movement_rules(true);
    let faithful = walking(&core, &part, &army(&[GNOLL]));
    for (points, clean, original) in [
        (120, 100, 141),
        (141, 100, 141),
        (160, 141, 141),
        (175, 141, 100),
        (247, 141, 100),
        (248, 141, 141),
    ] {
        assert_eq!(strict.step(&swamp, &swamp, true, points), clean, "{points}");
        assert_eq!(
            faithful.step(&swamp, &swamp, true, points),
            original,
            "{points}"
        );
    }
}

// covers: ADV-MOV-006
#[test]
fn points_below_the_cheapest_next_step_are_lost() {
    // D-3 (0x4B1880 after each step): the cheapest straight step from the new tile is its road's
    // cost, else the native cost, whatever its terrain.
    let (core, part) = movement_rules(true);
    let pricing = walking(&core, &part, &army(&[PEASANT]));
    let swamp = tile(SWAMP, None);
    assert_eq!(pricing.left_after(&swamp, 99), 0);
    assert_eq!(pricing.left_after(&swamp, 100), 100);
    let cobble = tile(SAND, Some(COBBLE_ROAD));
    assert_eq!(pricing.left_after(&cobble, 49), 0);
    assert_eq!(pricing.left_after(&cobble, 50), 50);
    let dirt = tile(GRASS, Some(DIRT_ROAD));
    assert_eq!(pricing.left_after(&dirt, 74), 0);
    // The example of ADV-Q3: sand onto a cobblestone road with 200 points leaves 50, which pay
    // one more road step (VCMI would leave 0).
    let sand = tile(SAND, None);
    let cost = pricing.step(&sand, &cobble, false, 200);
    assert_eq!(pricing.left_after(&cobble, 200 - cost), 50);
}

#[test]
fn no_step_costs_less_than_the_cheapest_straight_step() {
    let (core, part) = movement_rules(true);
    assert_eq!(walking(&core, &part, &army(&[PEASANT])).cheapest(), 50);
    // The other test rules' cheapest step is their road's.
    let (core, part) = rules();
    assert_eq!(walking(&core, &part, &[]).cheapest(), 75);
}

// covers: ADV-MOV-003
#[test]
fn pathfinding_lowers_the_rough_terrains_walking_price() {
    // E08 T40: the part's costs by Pathfinding level (the original's `step_cost_table` 0x63E510:
    // sand 150, 125, 100, 100; swamp 175, 150, 125, 100), from the source's terrain; roads, the
    // native cost and grass unchanged.
    let (core, part) = movement_rules(false);
    let (grass, sand, swamp) = (tile(GRASS, None), tile(SAND, None), tile(SWAMP, None));
    let road = tile(SWAMP, Some(DIRT_ROAD));
    for (level, from_sand, from_swamp) in
        [(0, 150, 175), (1, 125, 150), (2, 100, 125), (3, 100, 100)]
    {
        let pricing = pathfinder(&core, &part, &army(&[PEASANT]), level);
        assert_eq!(
            pricing.step(&sand, &grass, false, 1500),
            from_sand,
            "{level}"
        );
        assert_eq!(
            pricing.step(&swamp, &grass, false, 1500),
            from_swamp,
            "{level}"
        );
        assert_eq!(pricing.step(&grass, &swamp, false, 1500), 100);
        assert_eq!(pricing.step(&road, &road, false, 1500), 75);
        assert_eq!(pricing.cheapest(), 50);
        // Water's price at every level.
        assert_eq!(pricing.step(&tile(SEA, None), &grass, false, 1500), 100);
        // Gnolls keep their native 100 on swamp.
        let gnolls = pathfinder(&core, &part, &army(&[GNOLL]), level);
        assert_eq!(gnolls.step(&swamp, &grass, false, 1500), 100);
    }
    // ADV-QUIRK-3's check is the source terrain's price at the hero's level (0x4B1740): gnolls on
    // swamp with Basic Pathfinding check 150, so from 150 to 212 points a diagonal costs 100.
    let (core, part) = movement_rules(true);
    let gnolls = pathfinder(&core, &part, &army(&[GNOLL]), 1);
    assert_eq!(gnolls.step(&swamp, &swamp, true, 149), 141);
    assert_eq!(gnolls.step(&swamp, &swamp, true, 150), 100);
    assert_eq!(gnolls.step(&swamp, &swamp, true, 212), 100);
    assert_eq!(gnolls.step(&swamp, &swamp, true, 213), 141);
}

// covers: ADV-MOV-003, ADV-MOV-004
#[test]
fn a_terrain_walker_counts_its_terrain_as_another_for_every_price() {
    // E08 T40 (Nomads' sand as dirt, 0x4B1740): the test rules' wolf walks sand as swamp.
    for faithful in [false, true] {
        let (core, part) = hero_rules(faithful);
        let (sand, grass) = (tile(SAND, None), tile(GRASS, None));
        // Without a wolf, sand's 150; with one, swamp's 175 (125 at Advanced Pathfinding).
        let gnolls = walking(&core, &part, &army(&[GNOLL]));
        assert_eq!(gnolls.step(&sand, &grass, false, 1500), 150);
        let wolves = walking(&core, &part, &army(&[WOLF]));
        assert_eq!(wolves.step(&sand, &grass, false, 1500), 175);
        assert_eq!(
            pathfinder(&core, &part, &army(&[WOLF]), 2).step(&sand, &grass, false, 1500),
            125
        );
        // Only the source's terrain: from grass onto sand grass's 100.
        assert_eq!(wolves.step(&grass, &sand, false, 1500), 100);
        // The native test sees sand as swamp: wolves leading gnolls are native to swamp under
        // ADV-QUIRK-4 (faithful), so sand costs the native 100; strict, no native terrain.
        let pack = walking(&core, &part, &army(&[WOLF, GNOLL]));
        let native = if faithful { 100 } else { 175 };
        assert_eq!(pack.step(&sand, &grass, false, 1500), native);
        // The diagonal exception's check under ADV-QUIRK-3: swamp's 175 from sand, not sand's.
        if faithful {
            assert_eq!(pack.step(&sand, &sand, true, 175), 100);
            assert_eq!(pack.step(&sand, &sand, true, 174), 141);
        }
        // The drop prices the tile against itself after the walker too (under Cursed Ground).
        assert_eq!(wolves.left_after(&cursed(SAND, None), 174), 0);
        assert_eq!(wolves.left_after(&cursed(SAND, None), 175), 175);
        assert_eq!(wolves.cheapest(), 50);
    }
}

// covers: ADV-MOV-004, ADV-MOV-006
#[test]
fn cursed_ground_takes_the_native_price_away() {
    // E08 T40, D-23 (0x4B1740 tests the source tile's special ground 21; batch E5's E5-6: Gnolls
    // 175 from swamp under it): from a cursed tile every army pays the terrain's walking price,
    // by its Pathfinding level; a road still prices a road step; the destination does not count.
    for faithful in [false, true] {
        let (core, part) = movement_rules(faithful);
        let gnolls = walking(&core, &part, &army(&[GNOLL]));
        let (swamp, grass) = (cursed(SWAMP, None), tile(GRASS, None));
        assert_eq!(gnolls.step(&swamp, &grass, false, 1500), 175);
        assert_eq!(gnolls.step(&tile(SWAMP, None), &swamp, false, 1500), 100);
        assert_eq!(
            pathfinder(&core, &part, &army(&[GNOLL]), 1).step(&swamp, &grass, false, 1500),
            150
        );
        let road = cursed(SWAMP, Some(DIRT_ROAD));
        assert_eq!(gnolls.step(&road, &road, false, 1500), 75);
        // A diagonal: 247 from 248 points on; the exception's check is 175 in both profiles.
        assert_eq!(gnolls.step(&swamp, &swamp, true, 248), 247);
        assert_eq!(gnolls.step(&swamp, &swamp, true, 175), 175);
        // The drop after a step onto a cursed tile: the terrain's price (0x4B1880), not 100; a
        // road's still its own.
        assert_eq!(gnolls.left_after(&swamp, 174), 0);
        assert_eq!(gnolls.left_after(&swamp, 175), 175);
        assert_eq!(gnolls.left_after(&tile(SWAMP, None), 100), 100);
        assert_eq!(gnolls.left_after(&road, 75), 75);
        assert_eq!(
            gnolls.left_after(&cursed(TerrainId::from_raw(99), None), 9999),
            0
        );
    }
}

// covers: ADV-MOV-009
#[test]
fn a_flight_costs_the_lesser_from_free_land_and_the_fly_price_elsewhere() {
    // E08 T41 (0x4B1740 point 5 as corrected at ADV-Q22, in every profile; batch E5's E5-4: an
    // Angel Wings hero's inland road steps cost 50, from grass and from swamp 100): from a land
    // tile no object blocks, an object's visitable tile included, the lesser of the fly price at
    // the hero's level (140, 140, 120, 100) and the walking price; from a blocked tile or water
    // the fly price, a road under an obstacle included.
    for faithful in [false, true] {
        let (core, part) = movement_rules(faithful);
        let peasants = army(&[PEASANT]);
        let (grass, swamp, sea) = (tile(GRASS, None), tile(SWAMP, None), tile(SEA, None));
        let road = tile(GRASS, Some(COBBLE_ROAD));
        for (level, fly) in [(0, 140), (1, 140), (2, 120), (3, 100)] {
            let flyer = lifted(&core, &part, &peasants, Some(level), None);
            assert_eq!(flyer.step(&grass, &grass, false, 1500), 100);
            assert_eq!(flyer.step(&swamp, &grass, false, 1500), fly);
            assert_eq!(flyer.step(&road, &road, false, 1500), 50);
            assert_eq!(flyer.step(&entrance(SWAMP), &grass, false, 1500), fly);
            assert_eq!(flyer.step(&obstacle(GRASS, None), &grass, false, 1500), fly);
            let under = obstacle(GRASS, Some(COBBLE_ROAD));
            assert_eq!(flyer.step(&under, &road, false, 1500), fly);
            assert_eq!(flyer.step(&sea, &grass, false, 1500), fly);
            assert_eq!(flyer.step(&sea, &sea, true, 1500), diagonal_cost(fly));
            assert_eq!(flyer.cheapest(), 50);
            // The native cost and Cursed Ground before the lesser: gnolls' swamp 100.
            let gnolls = lifted(&core, &part, &army(&[GNOLL]), Some(level), None);
            assert_eq!(gnolls.step(&swamp, &grass, false, 1500), 100);
            assert_eq!(gnolls.step(&cursed(SWAMP, None), &grass, false, 1500), fly);
        }
    }
}

// covers: ADV-MOV-010
#[test]
fn a_water_walk_costs_the_water_walk_price_from_water() {
    // E08 T41 (0x4B1740 point 4; batch E5's E5-5 with Boots of Levitation: across a lake 450,
    // the sand's 150 onto the water and 100 a step after; back 491 with a diagonal): from water
    // the water-walk price at the hero's level (140, 140, 120, 100), from land the walking price.
    let (core, part) = movement_rules(true);
    let peasants = army(&[PEASANT]);
    let (sand, sea) = (tile(SAND, None), tile(SEA, None));
    let boots = lifted(&core, &part, &peasants, None, Some(3));
    let across = [sand, sea, sea, sea, sand];
    let price = |path: &[StaticTile], last_diagonal: bool| -> u32 {
        path.windows(2)
            .enumerate()
            .map(|(i, w)| boots.step(&w[0], &w[1], last_diagonal && i == 3, 2000))
            .sum()
    };
    assert_eq!(price(&across, false), 450);
    assert_eq!(price(&across, true), 491);
    for (level, water_walk) in [(0, 140), (1, 140), (2, 120), (3, 100)] {
        let walker = lifted(&core, &part, &peasants, None, Some(level));
        assert_eq!(walker.step(&sea, &sand, false, 1500), water_walk);
        assert_eq!(walker.step(&sand, &sea, false, 1500), 150);
        assert_eq!(walker.cheapest(), 50);
        // A hero that also flies pays the fly price from water.
        let both = lifted(&core, &part, &peasants, Some(2), Some(level));
        assert_eq!(both.step(&sea, &sea, false, 1500), 120);
    }
}

// covers: ADV-MOV-006, ADV-MOV-009, ADV-MOV-010
#[test]
fn the_diagonal_exceptions_check_is_the_walking_one_under_the_quirks() {
    // E08 T41 (0x4B1740 point 6): under ADV-QUIRK-2 and -3 the check C is the source terrain's
    // walking price (water's 100), never the fly or water-walk price; clean, the step's own
    // straight price, whatever priced it. A flight at level 1 from swamp: straight 140, diagonal
    // 197; C is swamp's 175 in the original, the 140 clean.
    let swamp = tile(SWAMP, None);
    let (core, part) = movement_rules(false);
    let strict = lifted(&core, &part, &army(&[PEASANT]), Some(1), None);
    let (core, part) = movement_rules(true);
    let faithful = lifted(&core, &part, &army(&[PEASANT]), Some(1), None);
    for (points, clean, original) in [
        (139, 197, 197),
        (140, 140, 197),
        (174, 140, 197),
        (175, 140, 140),
        (197, 140, 140),
        (198, 197, 140),
        (247, 197, 140),
        (248, 197, 197),
    ] {
        assert_eq!(strict.step(&swamp, &swamp, true, points), clean, "{points}");
        assert_eq!(
            faithful.step(&swamp, &swamp, true, points),
            original,
            "{points}"
        );
    }
    // At level 3, from swamp with 175 to 247 points: 100 in the original, 141 clean.
    let (core, part) = movement_rules(false);
    let strict = lifted(&core, &part, &army(&[PEASANT]), Some(3), None);
    let (core, part) = movement_rules(true);
    let faithful = lifted(&core, &part, &army(&[PEASANT]), Some(3), None);
    assert_eq!(strict.step(&swamp, &swamp, true, 200), 141);
    assert_eq!(faithful.step(&swamp, &swamp, true, 200), 100);
    // A water walk at level 2 from water: straight 120, diagonal 169; C 100 in the original
    // (water's walking price), 120 clean.
    let sea = tile(SEA, None);
    let (core, part) = movement_rules(false);
    let strict = lifted(&core, &part, &army(&[PEASANT]), None, Some(2));
    let (core, part) = movement_rules(true);
    let faithful = lifted(&core, &part, &army(&[PEASANT]), None, Some(2));
    for (points, clean, original) in [(110, 169, 120), (120, 120, 120), (150, 120, 169)] {
        assert_eq!(strict.step(&sea, &sea, true, points), clean, "{points}");
        assert_eq!(
            faithful.step(&sea, &sea, true, points),
            original,
            "{points}"
        );
    }
}

// covers: ADV-MOV-006, ADV-MOV-009, ADV-MOV-010
#[test]
fn the_drop_forgets_flying_and_water_walking_under_adv_quirk_17() {
    // E08 T41, A-12 (`cheapest_step_from` 0x4B1880 reads the hero's spell levels, not the
    // artifacts'): clean, the cheapest step from the new tile is priced with the hero's flying
    // and water walking as a step from it; in the original it is the walking one. On Cursed
    // Ground's swamp an Expert flyer's next step costs 100, so clean it keeps 100 to 174 points
    // the original drops.
    let (core, part) = movement_rules(false);
    let strict = lifted(&core, &part, &army(&[PEASANT]), Some(3), Some(2));
    let (core, part) = movement_rules(true);
    let faithful = lifted(&core, &part, &army(&[PEASANT]), Some(3), Some(2));
    let swamp = cursed(SWAMP, None);
    assert_eq!(strict.left_after(&swamp, 99), 0);
    assert_eq!(strict.left_after(&swamp, 100), 100);
    assert_eq!(faithful.left_after(&swamp, 174), 0);
    assert_eq!(faithful.left_after(&swamp, 175), 175);
    // The step itself costs the fly price in both.
    let grass = tile(GRASS, None);
    assert_eq!(strict.step(&swamp, &grass, false, 1500), 100);
    assert_eq!(faithful.step(&swamp, &grass, false, 1500), 100);
    // Over water and an obstacle: the fly price clean (a level 1 flight: 140), the native 100 in
    // the original; a road's cost under the lesser.
    let (core, part) = movement_rules(false);
    let strict = lifted(&core, &part, &army(&[PEASANT]), Some(1), None);
    let (core, part) = movement_rules(true);
    let faithful = lifted(&core, &part, &army(&[PEASANT]), Some(1), None);
    for over in [tile(SEA, None), obstacle(GRASS, None)] {
        assert_eq!(strict.left_after(&over, 139), 0);
        assert_eq!(strict.left_after(&over, 140), 140);
        assert_eq!(faithful.left_after(&over, 100), 100);
    }
    assert_eq!(strict.left_after(&tile(GRASS, Some(COBBLE_ROAD)), 50), 50);
    // A water walk at level 2 on water: 120 clean, 100 in the original.
    let (core, part) = movement_rules(false);
    let strict = lifted(&core, &part, &army(&[PEASANT]), None, Some(2));
    assert_eq!(strict.left_after(&tile(SEA, None), 119), 0);
    assert_eq!(strict.left_after(&tile(GRASS, None), 100), 100);
}

// covers: ADV-MOV-009, ADV-MOV-010
#[test]
fn the_heros_flying_and_water_walking_levels_come_from_its_details() {
    // HER-ADV-005 through `hero_movement` (the test rules' invented skills): the levels price the
    // steps as above.
    let (core, part) = hero_rules(true);
    let hero = hero_details(&[(FLYING, 2), (WATER_WALKING, 1)]);
    let movement = hero_movement(&core, Some(&hero));
    assert_eq!(
        (movement.flying, movement.water_walking),
        (Some(2), Some(1))
    );
    let pricing = Pricing::new(&core, &part, &army(&[PEASANT]), &movement);
    assert_eq!(
        pricing.step(&tile(SWAMP, None), &tile(GRASS, None), false, 1500),
        120
    );
}
