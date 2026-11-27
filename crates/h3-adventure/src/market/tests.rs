//! Tests of the market's rates (ADV-MKT-002, E08 T132a1): the survey's tables at five
//! Marketplaces (`docs/rules/surveys/E08-trades.md` §4, corrected at T132a1), the counts' rows
//! with the halves that fall on the binary32 side (D-70's 37, 62 and 112), the creature sale and
//! the artifact's price as batch E14 saw them (E14-1, E14-2, E14-4, E14-5), the factors against
//! the executable's tables, and the refusals.

extern crate std;

use std::vec;
use std::vec::Vec;

use proptest::prelude::{prop_assert, proptest};

use super::*;

/// The core's resources in the original's order.
const WOOD: ResourceId = ResourceId::from_raw(0);
const MERCURY: ResourceId = ResourceId::from_raw(1);
const ORE: ResourceId = ResourceId::from_raw(2);
const GEMS: ResourceId = ResourceId::from_raw(5);
const GOLD: ResourceId = ResourceId::from_raw(6);
/// A resource the parameters do not value.
const UNKNOWN: ResourceId = ResourceId::from_raw(7);

fn resource(i: usize) -> ResourceId {
    ResourceId::from_index(i).unwrap()
}

/// The original's rows: the resources', the artifacts' and the creatures' factors in hundredths
/// by count 1 to 10 (`market_resource_rates` 0x678344, `market_artifact_rates` 0x678370,
/// `market_creature_rates` 0x67839C).
const ROWS: [[u16; 3]; 10] = [
    [10, 20, 30],
    [15, 25, 45],
    [20, 30, 50],
    [25, 35, 65],
    [30, 40, 70],
    [35, 45, 85],
    [40, 50, 90],
    [45, 55, 100],
    [50, 60, 100],
    [50, 60, 100],
];

/// The original's parameters (`resource_values` 0x68C482, `resource_values_artifacts`
/// 0x68C492, [`ROWS`]).
fn original() -> MarketParams {
    MarketParams::new(
        GOLD,
        vec![250, 500, 250, 500, 500, 500, 1],
        vec![125, 250, 125, 250, 250, 250, 1],
        &ROWS,
    )
    .unwrap()
}

fn lot(give: u32, take: u32) -> Option<Lot> {
    Some(Lot { give, take })
}

// covers: ADV-MKT-002
#[test]
fn the_trades_at_five_marketplaces_are_the_originals() {
    // Survey §4's table (every map object trades as five), give : take; a precious resource for
    // wood or ore is 2 : 1 (the survey's first table said 1 : 2; `market_resources_proc` read
    // again at T132a1: q = 250 / 150 is at least 1, so round(q) given for one wanted).
    let w = (7, 1);
    let p = (3, 1);
    let (c, d) = ((833, 1), (1667, 1));
    let table: [[(u32, u32); 7]; 7] = [
        [(0, 0), w, (3, 1), w, w, w, (1, 75)],
        [(2, 1), (0, 0), (2, 1), p, p, p, (1, 150)],
        [(3, 1), w, (0, 0), w, w, w, (1, 75)],
        [(2, 1), p, (2, 1), (0, 0), p, p, (1, 150)],
        [(2, 1), p, (2, 1), p, (0, 0), p, (1, 150)],
        [(2, 1), p, (2, 1), p, p, (0, 0), (1, 150)],
        [c, d, c, d, d, d, (0, 0)],
    ];
    let market = original();
    for (g, row) in table.iter().enumerate() {
        for (t, &(give, take)) in row.iter().enumerate() {
            let expected = (g != t).then_some(Lot { give, take });
            assert_eq!(
                market.resource_lot(5, resource(g), resource(t)),
                expected,
                "{g} for {t}"
            );
        }
    }
}

// covers: ADV-MKT-002
#[test]
fn every_count_has_its_row_and_the_halves_fall_on_the_binary32_side() {
    let market = original();
    assert_eq!(market.counts(), 10);
    // A wood for gold: 250 × R exactly half a gold off a whole at 2, 4, 6 and 8 Marketplaces;
    // the binary32 of 1 / a decides (D-70's 37, 62 and 112, the town's rows; 88 at 6). Under a
    // 24-bit precision the 37 would be 38 (survey §4).
    let wood_for_gold: Vec<_> = (1..=10)
        .map(|m| market.resource_lot(m, WOOD, GOLD).unwrap().take)
        .collect();
    assert_eq!(wood_for_gold, [25, 37, 50, 62, 75, 88, 100, 112, 125, 125]);
    let gold_for_wood: Vec<_> = (1..=10)
        .map(|m| market.resource_lot(m, GOLD, WOOD).unwrap().give)
        .collect();
    assert_eq!(
        gold_for_wood,
        [2500, 1667, 1250, 1000, 833, 714, 625, 556, 500, 500]
    );
    // Batch E14's E14-5 at a Trading Post: 2 wood for 150 gold, 833 gold for a wood, 3 mercury
    // for a gem.
    assert_eq!(market.resource_lot(5, WOOD, GOLD), lot(1, 75));
    assert_eq!(market.resource_lot(5, GOLD, WOOD), lot(833, 1));
    assert_eq!(market.resource_lot(5, MERCURY, GEMS), lot(3, 1));
    assert_eq!(market.resource_lot(1, WOOD, ORE), lot(10, 1));
}

// covers: ADV-MKT-002
#[test]
fn a_creature_sells_for_its_cost_by_the_creatures_factor() {
    let market = original();
    // E14-1: a Pixie (25 gold) for 17 gold (25 × 0.7 is 17.5, the binary32 of 1 / 17.5 above
    // it; VCMI's ceiling 18, D-71), a Vampire (360) for one wood; E14-2: a Skeleton (60) for 42.
    assert_eq!(market.creature_lot(5, 25, GOLD), lot(1, 17));
    assert_eq!(market.creature_lot(5, 360, WOOD), lot(1, 1));
    assert_eq!(market.creature_lot(5, 60, GOLD), lot(1, 42));
    assert_eq!(market.creature_lot(5, 75, GOLD), lot(1, 52));
    // A cheap creature for a resource: several creatures for one; a dear one: several of the
    // resource for one creature (3 000 gold: 8 wood); a dear one for gold keeps every bit.
    assert_eq!(market.creature_lot(5, 360, GEMS), lot(2, 1));
    assert_eq!(market.creature_lot(5, 1, WOOD), lot(357, 1));
    assert_eq!(market.creature_lot(5, 3_000, WOOD), lot(1, 8));
    assert_eq!(market.creature_lot(5, 3_000, MERCURY), lot(1, 4));
    assert_eq!(market.creature_lot(5, 30_000, GOLD), lot(1, 21_000));
    assert_eq!(market.creature_lot(8, 5_000, GOLD), lot(1, 5_000));
    assert_eq!(market.creature_lot(1, 25, GOLD), lot(1, 8));
}

// covers: ADV-MKT-002
#[test]
fn an_artifact_costs_its_cost_over_the_artifacts_factor() {
    let market = original();
    // At five Marketplaces 2.5 times the cost in gold (E14-4: 5 000 for a treasure of 2 000),
    // cost / 50 of wood or ore, cost / 100 of a precious resource.
    for (cost, prices) in [
        (500, [10, 5, 10, 5, 5, 5, 1_250]),
        (2_000, [40, 20, 40, 20, 20, 20, 5_000]),
        (10_000, [200, 100, 200, 100, 100, 100, 25_000]),
    ] {
        for (r, price) in prices.into_iter().enumerate() {
            assert_eq!(market.artifact_price(5, cost, resource(r)), Some(price));
        }
    }
    assert_eq!(market.artifact_price(1, 2_000, GOLD), Some(10_000));
    assert_eq!(market.artifact_price(10, 2_000, WOOD), Some(27));
}

// covers: ADV-MKT-002, TWN-MKT-005
#[test]
fn an_artifact_sells_for_its_cost_by_the_artifacts_factor_over_the_trades_value() {
    let market = original();
    // Batch TWN-6 (survey `E09-trades.md` §14): TWN-6-3, Titan's Gladius (10 000) at eight
    // Marketplaces for 5 500 gold; TWN-6-4, Ring of the Magi (its parts' 3 000) at one for 600.
    assert_eq!(market.artifact_sale(8, 10_000, GOLD), Some(5_500));
    assert_eq!(market.artifact_sale(1, 3_000, GOLD), Some(600));
    assert_eq!(market.artifact_sale(10, 10_000, GOLD), Some(6_000));
    // The trades' values, not the purchase's (`resource_values` 0x68C482): 600 / 250 wood, 3 000
    // / 500 gems; a half rounds up (375 / 250), what is below it down (374.8 / 250).
    assert_eq!(market.artifact_sale(1, 3_000, WOOD), Some(2));
    assert_eq!(market.artifact_sale(10, 5_000, GEMS), Some(6));
    assert_eq!(market.artifact_sale(1, 1_875, WOOD), Some(2));
    assert_eq!(market.artifact_sale(1, 1_874, WOOD), Some(1));
    // At least 1: an artifact of no cost, a cheap one for a precious resource.
    assert_eq!(market.artifact_sale(5, 0, GOLD), Some(1));
    assert_eq!(market.artifact_sale(1, 100, GEMS), Some(1));
    // A count outside the parameters; a resource they do not value; a price past `u32`.
    assert_eq!(market.artifact_sale(0, 2_000, GOLD), None);
    assert_eq!(market.artifact_sale(11, 2_000, GOLD), None);
    assert_eq!(market.artifact_sale(5, 2_000, UNKNOWN), None);
    let dear = MarketParams::new(WOOD, vec![1], vec![1], &[[1, 65_535, 1]]).unwrap();
    assert_eq!(dear.artifact_sale(1, 1, WOOD), Some(655));
    assert_eq!(dear.artifact_sale(1, u32::MAX, WOOD), None);
}

// covers: ADV-MKT-002
#[test]
fn the_factors_are_the_executables_binary32() {
    // The tables' bit patterns at counts 1 to 10 (scripts/exe.py data 0x678348, 0x678374,
    // 0x6783A0 --type u32).
    let resources = [
        1_036_831_949,
        1_041_865_114,
        1_045_220_557,
        1_048_576_000,
        1_050_253_722,
        1_051_931_443,
        1_053_609_165,
        1_055_286_886,
        1_056_964_608,
        1_056_964_608,
    ];
    let artifacts = [
        1_045_220_557,
        1_048_576_000,
        1_050_253_722,
        1_051_931_443,
        1_053_609_165,
        1_055_286_886,
        1_056_964_608,
        1_057_803_469,
        1_058_642_330,
        1_058_642_330,
    ];
    let creatures = [
        1_050_253_722,
        1_055_286_886,
        1_056_964_608,
        1_059_481_190,
        1_060_320_051,
        1_062_836_634,
        1_063_675_494,
        1_065_353_216,
        1_065_353_216,
        1_065_353_216,
    ];
    let market = original();
    for m in 1..=10u8 {
        let i = usize::from(m - 1);
        assert_eq!(
            market.factor_bits(m),
            Some([resources[i], artifacts[i], creatures[i]])
        );
    }
    assert_eq!(market.factor_bits(0), None);
    assert_eq!(market.factor_bits(11), None);
    assert_eq!(market.value(MERCURY), Some(500));
    assert_eq!(market.artifact_value(MERCURY), Some(250));
    assert_eq!(
        (market.value(UNKNOWN), market.artifact_value(UNKNOWN)),
        (None, None)
    );
}

// covers: ADV-MKT-002
#[test]
fn what_has_no_rate_is_refused() {
    let market = original();
    // A resource for itself; a count outside the rows; a resource the parameters do not value.
    assert_eq!(market.resource_lot(5, WOOD, WOOD), None);
    assert_eq!(market.resource_lot(0, WOOD, GOLD), None);
    assert_eq!(market.resource_lot(11, WOOD, GOLD), None);
    assert_eq!(market.resource_lot(5, UNKNOWN, GOLD), None);
    assert_eq!(market.resource_lot(5, WOOD, UNKNOWN), None);
    // A creature of no gold cost; a count outside; an unknown resource; a lot past `u32` (a
    // creature of cost `u32::MAX` for gold at a factor of 1: 2^32 gold).
    assert_eq!(market.creature_lot(5, 0, GOLD), None);
    assert_eq!(market.creature_lot(0, 25, GOLD), None);
    assert_eq!(market.creature_lot(5, 25, UNKNOWN), None);
    assert_eq!(market.creature_lot(8, u32::MAX, GOLD), None);
    // An artifact of no cost; a price that rounds to 0; a price past `u32`; a count outside; an
    // unknown resource.
    assert_eq!(market.artifact_price(5, 0, GOLD), None);
    assert_eq!(market.artifact_price(1, 12, WOOD), None);
    assert_eq!(market.artifact_price(1, 13, WOOD), Some(1));
    assert_eq!(market.artifact_price(1, u32::MAX, GOLD), None);
    assert_eq!(market.artifact_price(11, 2_000, GOLD), None);
    assert_eq!(market.artifact_price(5, 2_000, UNKNOWN), None);
}

// covers: ADV-MKT-002
#[test]
fn the_parameters_refuse_what_no_rate_can_use() {
    let rows = [[30, 40, 70]];
    let new = |values: Vec<u16>, artifact_values: Vec<u16>, rows: &[[u16; 3]]| {
        MarketParams::new(WOOD, values, artifact_values, rows)
    };
    assert_eq!(new(vec![], vec![], &rows), Err(MarketParamsError::Values));
    assert_eq!(
        new(vec![250, 1], vec![125], &rows),
        Err(MarketParamsError::Values)
    );
    assert_eq!(
        new(vec![250, 1], vec![0, 1], &rows),
        Err(MarketParamsError::ZeroValue)
    );
    assert_eq!(new(vec![1], vec![1], &[]), Err(MarketParamsError::Rows));
    assert_eq!(
        new(vec![1], vec![1], &[[30, 40, 70]; 256]),
        Err(MarketParamsError::Rows)
    );
    for zero in [[0, 40, 70], [30, 0, 70], [30, 40, 0]] {
        assert_eq!(
            new(vec![1], vec![1], &[[30, 40, 70], zero]),
            Err(MarketParamsError::ZeroFactor)
        );
    }
    let most = new(vec![1], vec![1], &[[30, 40, 70]; 255]).unwrap();
    assert_eq!(most.counts(), 255);
    // E08 T133b (ADV-Q60 (a)): the cost resource is one the tables value.
    assert_eq!(
        MarketParams::new(ORE, vec![250, 1], vec![125, 1], &rows),
        Err(MarketParamsError::Cost)
    );
    assert_eq!((original().cost(), most.cost()), (GOLD, WOOD));
}

// covers: ADV-MKT-002
#[test]
fn the_cheapest_lot_is_the_least_any_trade_takes_and_its_most_over_the_counts() {
    // E09 T58a2: the legal listing's tables, each the lots' own.
    let params = original();
    let resources = || (0..7).map(resource);
    for give in resources() {
        let mut most = 0;
        for count in 1..=params.counts() {
            let least = resources()
                .filter_map(|take| params.resource_lot(count, give, take))
                .map(|lot| lot.give.max(1))
                .min();
            assert_eq!(
                params.cheapest_lot(count, give),
                least,
                "{give:?} at {count}"
            );
            most = most.max(least.unwrap());
        }
        assert_eq!(params.cheapest_at_every_count(give), Some(most));
    }
    // Gold buys a wood at one Marketplace for its 2 500 (250 over 0.1), at ten for 500.
    assert_eq!(params.cheapest_lot(1, GOLD), Some(2500));
    assert_eq!(params.cheapest_lot(10, GOLD), Some(500));
    assert_eq!(params.cheapest_at_every_count(GOLD), Some(2500));
    // None outside the counts, for a resource not valued, or with no other resource.
    assert_eq!(params.cheapest_lot(0, GOLD), None);
    assert_eq!(params.cheapest_lot(11, GOLD), None);
    assert_eq!(params.cheapest_lot(1, UNKNOWN), None);
    assert_eq!(params.cheapest_at_every_count(UNKNOWN), None);
    let alone = MarketParams::new(WOOD, vec![250], vec![125], &[[10, 20, 30]]).unwrap();
    assert_eq!(alone.cheapest_lot(1, WOOD), None);
    assert_eq!(alone.cheapest_at_every_count(WOOD), None);
}

proptest! {
    // covers: ADV-MKT-002
    #[test]
    fn every_lot_gives_and_takes_at_least_one(
        count in 1u8..=10,
        cost in 1u32..=200_000,
        given in 0usize..7,
        wanted in 0usize..7,
    ) {
        let market = original();
        let (given, wanted) = (resource(given), resource(wanted));
        if let Some(lot) = market.resource_lot(count, given, wanted) {
            prop_assert!(lot.give >= 1 && lot.take >= 1 && (lot.give == 1 || lot.take == 1));
        } else {
            prop_assert!(given == wanted);
        }
        let lot = market.creature_lot(count, cost, wanted).unwrap();
        prop_assert!(lot.give >= 1 && lot.take >= 1 && (lot.give == 1 || lot.take == 1));
        if let Some(price) = market.artifact_price(count, cost, wanted) {
            prop_assert!(price >= 1);
        }
    }
}
