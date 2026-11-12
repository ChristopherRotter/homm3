//! Tests of a neutral town's garrison week (E09 T25b2) on the fixture's Keep (level 1: handles 10
//! and 11, growth 14, AI value 100 and 110; level 2: 20 and 21, growth 9; level 3: 30 without an
//! upgrade, growth 7) and Marsh (one level, 50 without an upgrade, growth 6); batch TWN-3's row 4
//! (survey `E09-growth.md` §17) for the room and the counts.

extern crate std;

use std::vec;
use std::vec::Vec;

use h3_kernel::StreamId;

use super::*;
use crate::fixture::{
    DWELLING_1, FORT, MARSH, WILD, creature, empty_part, part, part_changed, part_with, town,
    town_of,
};
use crate::ruleset::{GarrisonWeekParams, TownParams, TownRecord};

/// The army's slots and a stack's most creatures (the adventure's 7 and 4 095).
const RULES: (u8, u32) = (7, 4095);

/// The towns' stream of `seed`.
fn draws(seed: u64) -> Rng {
    Rng::for_stream(seed, StreamId::TOWNS)
}

fn stack(slot: u8, raw: u16, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature: creature(raw),
        count,
    }
}

/// Parameters of a sure week: a stack every week (101 in 101), always level 1 (no weights), never
/// the upgrade; then changed by `change`.
fn sure_params(p: &mut TownParams, change: impl FnOnce(&mut GarrisonWeekParams)) {
    p.garrison_week = GarrisonWeekParams {
        odds: [101, 101],
        out_of: 101,
        weights: Vec::new(),
        weeks: 8,
        upgrade: 0,
    };
    change(&mut p.garrison_week);
}

/// The part with a sure week changed by `change`.
fn sure(change: impl FnOnce(&mut GarrisonWeekParams)) -> TownRuleset {
    part_with(|p| sure_params(p, change))
}

/// The part with a sure week and the Keep's record changed by `keep`.
fn sure_keep(keep: impl FnOnce(&mut TownRecord)) -> TownRuleset {
    part_changed(|p| sure_params(p, |_| {}), keep)
}

/// `rng` after a week's draws: the chance, the three sums with `n`, the upgrade's when `upgrade`;
/// with the sum S.
fn after(rng: &Rng, n: u32, upgrade: bool) -> (Rng, u32) {
    let mut reference = rng.clone();
    reference.below(101);
    let sum = (0..3).map(|_| reference.range_inclusive(0, n + 1)).sum();
    if upgrade {
        reference.range_inclusive(1, 100);
    }
    (reference, sum)
}

/// The Keep's garrison week of `town` with `garrison`, `weeks` 3, [`RULES`], in `part`.
fn week(
    part: &TownRuleset,
    town: &Town,
    garrison: &[ArmyStack],
    rng: &mut Rng,
) -> Option<GarrisonWeek> {
    garrison_week(town, part, &crate::fixture::core(), 3, garrison, RULES, rng)
}

#[test]
fn the_level_is_the_first_weight_the_sum_does_not_exceed() {
    // covers: TWN-GRO-006
    let weights = [2, 3, 4, 5, 4, 3];
    // The original's table (survey §4): S ≤ 2 level 1, ≤ 5 level 2, ≤ 9 level 3, ≤ 14 level 4,
    // ≤ 18 level 5, ≤ 21 level 6, else level 7.
    let expected = |s: u32| match s {
        0..=2 => 1,
        3..=5 => 2,
        6..=9 => 3,
        10..=14 => 4,
        15..=18 => 5,
        19..=21 => 6,
        _ => 7,
    };
    for sum in 0..=30 {
        assert_eq!(level(&weights, sum), expected(sum), "sum {sum}");
    }
    // The most S is 3 (n + 1): week 2's draw (n 1) reaches level 3 at most, n 6 level 6; level 7
    // needs n 7 (from week 8).
    assert_eq!(level(&weights, 3 * 2), 3);
    assert_eq!(level(&weights, 3 * 7), 6);
    assert_eq!(level(&weights, 3 * 8), 7);
    // No weights: every sum is level 1.
    assert_eq!(level(&[], 0), 1);
    assert_eq!(level(&[], 9), 1);
}

#[test]
fn a_week_grows_a_stack_with_the_odds_of_its_fort() {
    // covers: TWN-GRO-006
    let bare = town(&[DWELLING_1]);
    let fort = town(&[FORT, DWELLING_1]);
    // Never without a fort, always with one, and the reverse.
    for (odds, grows) in [([0, 101], [false, true]), ([101, 0], [true, false])] {
        let part = sure(|g| g.odds = odds);
        for (town, grows) in [(&bare, grows[0]), (&fort, grows[1])] {
            let mut rng = draws(1);
            let result = week(&part, town, &[], &mut rng);
            assert_eq!(result.is_some(), grows);
            if !grows {
                // One draw, nothing else.
                let mut reference = draws(1);
                reference.below(101);
                assert_eq!(rng, reference);
            }
        }
    }
    // The original's 40 and 80 in 101 (an empty garrison always has room; weeks 0 give levels 1
    // and 2): a stack exactly when the chance draw is below them.
    let part = part();
    let mut outcomes = Vec::new();
    for seed in 0..40 {
        for (town, odds) in [(&bare, 40), (&fort, 80)] {
            let roll = draws(seed).below(101);
            let mut rng = draws(seed);
            let grown = garrison_week(
                town,
                &part,
                &crate::fixture::core(),
                0,
                &[],
                RULES,
                &mut rng,
            );
            assert_eq!(grown.is_some(), roll < odds, "seed {seed}");
            outcomes.push((odds, grown.is_some()));
        }
    }
    for odds in [40, 80] {
        assert!(outcomes.contains(&(odds, true)) && outcomes.contains(&(odds, false)));
    }
}

#[test]
fn a_new_stack_takes_the_first_empty_slot_out_of_the_level_s_counts() {
    // covers: TWN-GRO-006
    let part = sure(|_| {});
    let counted = town(&[DWELLING_1])
        .with_available(0, 20)
        .with_available(7, 10)
        .with_available(1, 9);
    // Slots 0 and 2 taken, given out of order: the Pikemen-like level 1 (14) into slot 1.
    let garrison = [stack(2, 30, 5), stack(0, 20, 3)];
    let mut rng = draws(2);
    let (reference, _) = after(&rng, 3, true);
    let grown = week(&part, &counted, &garrison, &mut rng).unwrap();
    assert_eq!(
        grown.stacks,
        [stack(0, 20, 3), stack(1, 10, 14), stack(2, 30, 5)]
    );
    // Both of the level's counts lose the 14 (at least 0); level 2's is untouched.
    let counts: Vec<u16> = [0, 7, 1].iter().map(|&i| grown.town.available(i)).collect();
    assert_eq!(counts, [6, 0, 9]);
    assert_eq!(grown.town.built(), counted.built());
    // The chance, the three sums, the base creature's upgrade draw.
    assert_eq!(rng, reference);
}

#[test]
fn a_garrison_holding_the_upgrade_grows_the_upgrade_with_no_upgrade_draw() {
    // covers: TWN-GRO-006
    let part = sure(|g| g.upgrade = 100);
    let counted = town(&[DWELLING_1])
        .with_available(0, 30)
        .with_available(7, 15);
    let garrison = [stack(0, 11, 2), stack(1, 10, 4)];
    let mut rng = draws(3);
    let (reference, _) = after(&rng, 3, false);
    let grown = week(&part, &counted, &garrison, &mut rng).unwrap();
    // The upgrade's 14 merge into its stack; the base stack stays the base's.
    assert_eq!(grown.stacks, [stack(0, 11, 16), stack(1, 10, 4)]);
    assert_eq!([grown.town.available(0), grown.town.available(7)], [16, 1]);
    assert_eq!(rng, reference);
}

#[test]
fn a_base_creature_s_week_may_turn_every_stack_of_it_into_the_upgrade() {
    // covers: TWN-GRO-006
    let bare = town(&[DWELLING_1]);
    // Two stacks of the base (a custom garrison): the 14 merge into the first, then both turn.
    let garrison = [stack(0, 10, 3), stack(4, 10, 4), stack(5, 20, 1)];
    for (chance, raw) in [(100, 11), (0, 10)] {
        let part = sure(|g| g.upgrade = chance);
        let mut rng = draws(4);
        let (reference, _) = after(&rng, 3, true);
        let grown = week(&part, &bare, &garrison, &mut rng).unwrap();
        assert_eq!(
            grown.stacks,
            [stack(0, raw, 17), stack(4, raw, 4), stack(5, 20, 1)]
        );
        assert_eq!(rng, reference);
    }
    // The original's 5 in 100: the upgrade exactly when the draw after the sums is at most 5.
    let part = sure(|g| g.upgrade = 5);
    let mut outcomes = Vec::new();
    for seed in 0..200 {
        let (mut probe, _) = after(&draws(seed), 3, false);
        let roll = probe.range_inclusive(1, 100);
        let grown = week(&part, &bare, &[], &mut draws(seed)).unwrap();
        let upgraded = grown.stacks == [stack(0, 11, 14)];
        assert_eq!(upgraded, roll <= 5, "seed {seed}");
        outcomes.push(upgraded);
    }
    assert!(outcomes.contains(&true) && outcomes.contains(&false));
    // A level without an upgrade (the Marsh's): the draw is made, the stack stays.
    let part = sure(|g| g.upgrade = 100);
    let mut rng = draws(5);
    let (reference, _) = after(&rng, 3, true);
    let grown = week(&part, &town_of(MARSH, &[]), &[], &mut rng).unwrap();
    assert_eq!(grown.stacks, [stack(0, 50, 6)]);
    assert_eq!(rng, reference);
}

#[test]
fn a_full_garrison_clears_its_weakest_stack_below_the_new_one() {
    // covers: TWN-GRO-006
    let part = sure(|_| {});
    let bare = town(&[DWELLING_1]).with_available(0, 14);
    // The new stack's value 14 × 100 = 1 400; slots 1 and 2 the least (1 200), equal: the first
    // in slot order goes (batch TWN-3 row 4 cleared the least first; the tie not reached there).
    let full = [
        stack(6, 30, 6),
        stack(5, 41, 4),
        stack(4, 21, 10),
        stack(3, 50, 5),
        stack(2, 40, 3),
        stack(1, 30, 4),
        stack(0, 20, 8),
    ];
    let mut rng = draws(6);
    let (reference, _) = after(&rng, 3, true);
    let grown = week(&part, &bare, &full, &mut rng).unwrap();
    assert_eq!(
        grown.stacks,
        [
            stack(0, 20, 8),
            stack(1, 10, 14),
            stack(2, 40, 3),
            stack(3, 50, 5),
            stack(4, 21, 10),
            stack(5, 41, 4),
            stack(6, 30, 6),
        ]
    );
    assert_eq!(grown.town.available(0), 0);
    assert_eq!(rng, reference);
    // Every stack worth the new one or more: nothing at all, no draw after the sums (row 4's N1).
    let strong: Vec<ArmyStack> = (0..7).map(|slot| stack(slot, 20, 7)).collect();
    let mut rng = draws(6);
    let (reference, _) = after(&rng, 3, false);
    assert_eq!(week(&part, &bare, &strong, &mut rng), None);
    assert_eq!(rng, reference);
    // A full garrison with a stack of the creature merges, clearing nothing.
    let mut held = strong.clone();
    held[3] = stack(3, 10, 5);
    let grown = week(&part, &bare, &held, &mut draws(6)).unwrap();
    assert_eq!(grown.stacks[3], stack(3, 10, 19));
    assert_eq!(grown.stacks.len(), 7);
}

#[test]
fn the_room_is_the_army_s_slots_and_a_stack_s_most() {
    // covers: TWN-GRO-006
    let part = sure(|_| {});
    let bare = town(&[DWELLING_1]);
    let core = crate::fixture::core();
    let grow = |garrison: &[ArmyStack], rules| {
        garrison_week(&bare, &part, &core, 3, garrison, rules, &mut draws(7))
    };
    // Two slots, both taken: the weakest (200) makes room though slots 2 to 6 are not the army's.
    let two = [stack(0, 20, 1), stack(1, 30, 9)];
    assert_eq!(
        grow(&two, (2, 4095)).unwrap().stacks,
        [stack(0, 10, 14), stack(1, 30, 9)]
    );
    // A stack that cannot take the 14 within the most: the next free slot.
    let one = [stack(0, 10, 10)];
    assert_eq!(
        grow(&one, (7, 20)).unwrap().stacks,
        [stack(0, 10, 10), stack(1, 10, 14)]
    );
    // A growth above a stack's most: nothing, no draw after the sums.
    let mut rng = draws(7);
    let (reference, _) = after(&rng, 3, false);
    assert_eq!(
        garrison_week(&bare, &part, &core, 3, &two, (2, 10), &mut rng),
        None
    );
    assert_eq!(rng, reference);
}

#[test]
fn weeks_past_the_parameters_count_as_their_most() {
    // covers: TWN-GRO-006
    let part = sure(|g| g.weights = vec![2, 3, 4, 5, 4, 3]);
    let bare = town(&[DWELLING_1]);
    let core = crate::fixture::core();
    for seed in 0..20 {
        let grow =
            |weeks, rng: &mut Rng| garrison_week(&bare, &part, &core, weeks, &[], RULES, rng);
        let (mut eight, mut fifty) = (draws(seed), draws(seed));
        assert_eq!(grow(8, &mut eight), grow(50, &mut fifty));
        assert_eq!(eight, fifty);
        // Week 2 (one week passed): S at most 6, the levels 1 to 3 the Keep has.
        let mut rng = draws(seed);
        let (_, sum) = after(&rng, 1, false);
        let grown = grow(1, &mut rng).unwrap();
        let expected = [10, 20, 30][level(&[2, 3, 4, 5, 4, 3], sum) - 1];
        assert_eq!(grown.stacks[0].creature, creature(expected));
    }
}

#[test]
fn a_town_without_its_rules_or_a_creature_grows_nothing() {
    // covers: TWN-GRO-006
    let core = crate::fixture::core();
    let grow = |part: &TownRuleset, town: &Town, rng: &mut Rng| {
        garrison_week(town, part, &core, 3, &[], RULES, rng)
    };
    // No parameters, no record: nothing drawn.
    for (part, town) in [
        (empty_part(), town(&[])),
        (sure(|_| {}), town_of(WILD, &[])),
    ] {
        let mut rng = draws(8);
        assert_eq!(grow(&part, &town, &mut rng), None);
        assert_eq!(rng, draws(8));
    }
    // A level the record lacks (the Marsh's level 2, any sum above 0 with the weight 0): nothing
    // after the sums.
    let part = sure(|g| g.weights = vec![0]);
    let marsh = town_of(MARSH, &[]);
    let mut outcomes = Vec::new();
    for seed in 0..40 {
        let mut rng = draws(seed);
        let (reference, sum) = after(&rng, 3, false);
        let grown = grow(&part, &marsh, &mut rng);
        assert_eq!(grown.is_some(), sum == 0, "seed {seed}");
        if grown.is_none() {
            assert_eq!(rng, reference);
        }
        outcomes.push(sum == 0);
    }
    assert!(outcomes.contains(&true) && outcomes.contains(&false));
    // A level whose creature grows none (handle 5): nothing after the sums.
    let part = sure_keep(|k| k.creatures[0].base = creature(5));
    let mut rng = draws(9);
    let (reference, _) = after(&rng, 3, false);
    assert_eq!(grow(&part, &town(&[DWELLING_1]), &mut rng), None);
    assert_eq!(rng, reference);
}
