//! Tests of the Artifact Merchants' stock in the game (TWN-STK-001, -002; E09 T57, the town epic's
//! design note §108): the new game's draw from the setup stream after every other setup draw, and
//! each later month's on its first day from the towns' stream, every entry anew with the old ones
//! marked.

use alloc::collections::BTreeSet;
use alloc::vec;
use alloc::vec::Vec;

use h3_content::{ArtifactClass, ArtifactDef, ArtifactTypeId};
use h3_kernel::Rng;

use super::growth::{rules_with, to_day};
use super::towns_setup;
use crate::tests::key;
use crate::town::Draws;
use crate::{GameRuleset, GameState};

/// The growth tests' rules (normal months) with the minors `t:m0` to `t:m2` and the major
/// `t:j0` after the core's artifacts (all treasures), the merchants' stock `classes`.
fn rules_of(classes: Vec<ArtifactClass>) -> GameRuleset {
    rules_with(
        [1, 0, 0],
        |core| {
            for (k, class) in [
                ("t:m0", ArtifactClass::Minor),
                ("t:m1", ArtifactClass::Minor),
                ("t:m2", ArtifactClass::Minor),
                ("t:j0", ArtifactClass::Major),
            ] {
                core.artifacts.push(ArtifactDef {
                    key: key(k),
                    name: "Artifact".into(),
                    class,
                    cost: vec![],
                    slot: Some(key("t:ring_slot")),
                    war_machine: None,
                    needs_skill: None,
                    effects: vec![],
                    parts: vec![],
                });
            }
        },
        |towns| towns.params.as_mut().unwrap().merchants_stock = classes,
    )
}

/// The artifacts of the keys `keys`.
fn artifacts<const N: usize>(rules: &GameRuleset, keys: [&str; N]) -> [ArtifactTypeId; N] {
    keys.map(|k| rules.core().artifact_by_key(k).unwrap())
}

/// The towns' game of [`towns_setup`] under `rules`, built from `seed`.
fn built(rules: &GameRuleset, seed: u64) -> GameState {
    GameState::new(rules, &towns_setup(rules), seed).unwrap()
}

/// The towns' stream as it stands.
fn towns_stream(game: &mut GameState) -> Rng {
    game.town_draws(Draws::Towns).2.clone()
}

// covers: TWN-STK-001
#[test]
fn the_new_games_stock_is_the_setup_streams_last_draws() {
    let rules = rules_of(vec![
        ArtifactClass::Minor,
        ArtifactClass::Minor,
        ArtifactClass::Major,
        ArtifactClass::Treasure,
    ]);
    let without = rules_of(vec![]);
    let minors = artifacts(&rules, ["t:m0", "t:m1", "t:m2"]);
    let [major] = artifacts(&rules, ["t:j0"]);
    let mut stocks = BTreeSet::new();
    for seed in 0..8 {
        let mut game = built(&rules, seed);
        let stock = game.adventure().merchants_stock().to_vec();
        assert_eq!(stock, built(&rules, seed).adventure().merchants_stock());
        let [Some(a), Some(b), Some(j), Some(t)] = stock[..] else {
            panic!("four entries, each an artifact: {stock:?}");
        };
        assert!(a != b && minors.contains(&a) && minors.contains(&b));
        assert_eq!(j, major);
        assert_eq!(
            rules.core().artifact(t).unwrap().class,
            ArtifactClass::Treasure
        );
        stocks.insert(stock);
        // The game without a stock is the same game but for it: no other draw moved, and the
        // towns' stream is untouched.
        let mut other = built(&without, seed);
        assert!(other.adventure().merchants_stock().is_empty());
        assert_eq!(game.towns(), other.towns());
        assert_eq!(game.heroes(), other.heroes());
        assert_eq!(game.adventure().heroes(), other.adventure().heroes());
        assert_eq!(towns_stream(&mut game), towns_stream(&mut other));
        assert_ne!(game.state_hash(), other.state_hash());
    }
    assert!(stocks.len() > 1, "the stock varies with the seed");
}

// covers: TWN-STK-002, GAM-CAL-002
#[test]
fn each_months_first_day_draws_the_stock_anew_from_the_towns_stream() {
    let rules = rules_of(vec![ArtifactClass::Minor]);
    let without = rules_of(vec![]);
    let minors = artifacts(&rules, ["t:m0", "t:m1", "t:m2"]);
    for seed in 0..4 {
        let mut game = built(&rules, seed);
        let mut other = built(&without, seed);
        let mut months = Vec::new();
        for (first, last) in [(1, 28), (29, 56), (57, 57)] {
            to_day(&rules, &mut game, first);
            let stock = game.adventure().merchants_stock().to_vec();
            // Not before the month's first day: the stock stays all month.
            to_day(&rules, &mut game, last);
            assert_eq!(game.adventure().merchants_stock(), stock);
            months.push(stock[0].unwrap());
        }
        // One minor an entry: the unsold minors stay marked, so the three months give the three.
        assert_eq!(BTreeSet::from_iter(months), BTreeSet::from(minors));
        // The months' draws are the towns' stream's: one draw more than the game without a stock.
        to_day(&without, &mut other, 57);
        assert_ne!(towns_stream(&mut game), towns_stream(&mut other));
    }
    // Before day 29 the two games' towns' streams agree.
    let (mut game, mut other) = (built(&rules, 0), built(&without, 0));
    to_day(&rules, &mut game, 28);
    to_day(&without, &mut other, 28);
    assert_eq!(towns_stream(&mut game), towns_stream(&mut other));
}
