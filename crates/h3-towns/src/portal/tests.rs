//! Tests of the Portal of Summoning's draw (E09 T25b2) on the fixture's Keep (its Shrine is the
//! Portal; level 1 handles 10 and 11 grow 14, level 2's 20 grows 9, the Marsh's 50 grows 6);
//! batch TWN-3's row 5 (survey `E09-growth.md` §17): a dwelling, then its creature, the count set.

extern crate std;

use std::vec::Vec;

use h3_kernel::StreamId;

use super::*;
use crate::fixture::{
    CASTLE, DWELLING_1, GRAIL, HORDE_1, SHRINE, core, creature, empty_part, part, town,
};

/// The towns' stream of `seed`.
fn draws(seed: u64) -> Rng {
    Rng::for_stream(seed, StreamId::TOWNS)
}

/// What a Portal holds.
fn summoned(raw: u16, count: u16) -> Option<Summoned> {
    Some(Summoned {
        creature: creature(raw),
        count,
    })
}

/// A Keep town with the Portal and `built` ever built, its Portal holding 3 of the Marsh's 50.
fn portal_town(built: &[usize]) -> Town {
    let built: Vec<usize> = built.iter().copied().chain([SHRINE]).collect();
    town(&built).with_portal(summoned(50, 3))
}

#[test]
fn a_town_without_a_standing_portal_draws_nothing() {
    // covers: TWN-GRO-007
    let offer: [&[CreatureTypeId]; 1] = [&[creature(10)]];
    let held = town(&[DWELLING_1]).with_portal(summoned(50, 3));
    for part in [part(), empty_part()] {
        let mut rng = draws(1);
        assert_eq!(summon(&held, &part, &core(), &offer, &mut rng), held);
        assert_eq!(rng, draws(1));
    }
}

#[test]
fn a_portal_without_dwellings_is_emptied_with_no_draw() {
    // covers: TWN-GRO-007
    let mut rng = draws(2);
    let emptied = summon(&portal_town(&[]), &part(), &core(), &[], &mut rng);
    assert_eq!(emptied.portal(), None);
    assert_eq!(emptied.built(), portal_town(&[]).built());
    assert_eq!(rng, draws(2));
}

#[test]
fn a_portal_draws_a_dwelling_then_one_of_its_creatures_its_count_set_to_the_growth() {
    // covers: TWN-GRO-007
    // A castle, a horde and a Grail standing: none touches the Portal's count (TD-16).
    let held = portal_town(&[CASTLE, GRAIL, DWELLING_1, HORDE_1]);
    let two: &[CreatureTypeId] = &[creature(10), creature(11)];
    let one: &[CreatureTypeId] = &[creature(20)];
    let offer = [two, one];
    let mut seen = Vec::new();
    for seed in 0..30 {
        let mut reference = draws(seed);
        let dwelling = offer[usize::try_from(reference.below(2)).unwrap()];
        let pick = dwelling
            [usize::try_from(reference.below(u32::try_from(dwelling.len()).unwrap())).unwrap()];
        let mut rng = draws(seed);
        let drawn = summon(&held, &part(), &core(), &offer, &mut rng);
        let growth = if pick == creature(20) { 9 } else { 14 };
        // Set, not added to the 3 held before; the rest of the town kept.
        assert_eq!(
            drawn,
            held.with_portal(Some(Summoned {
                creature: pick,
                count: growth
            }))
        );
        assert_eq!(rng, reference);
        seen.push(pick);
    }
    for raw in [10, 11, 20] {
        assert!(seen.contains(&creature(raw)), "creature {raw} never drawn");
    }
}

#[test]
fn a_drawn_dwelling_without_creatures_keeps_the_portal() {
    // covers: TWN-GRO-007
    let held = portal_town(&[]);
    let offer: [&[CreatureTypeId]; 1] = [&[]];
    let mut rng = draws(3);
    let mut reference = draws(3);
    reference.below(1);
    assert_eq!(summon(&held, &part(), &core(), &offer, &mut rng), held);
    assert_eq!(rng, reference);
    // An empty Portal stays empty; a creature the core lacks counts 0.
    let empty = held.with_portal(None);
    assert_eq!(
        summon(&empty, &part(), &core(), &offer, &mut draws(3)),
        empty
    );
    let unknown: [&[CreatureTypeId]; 1] = [&[creature(999)]];
    assert_eq!(
        summon(&empty, &part(), &core(), &unknown, &mut draws(3)).portal(),
        summoned(999, 0)
    );
}
