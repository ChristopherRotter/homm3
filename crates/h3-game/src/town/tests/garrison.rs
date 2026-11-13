//! Tests of a neutral town's garrison week in the game (E09 T26b, the town epic's design note §44;
//! TWN-GRO-006): on a week's first day each neutral town draws from the towns' stream after the
//! week's growth, its garrison and counts written and told to the viewers whose view of it moves;
//! an owned town's garrison never grows.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use h3_contracts::ArmyStack;
use h3_kernel::{Rng, StreamId};
use h3_towns::{BuildingDef, BuildingId, BuildingRoles, GrowthInputs};

use super::command::RED;
use super::growth::{T0, T3, built_seeded, rules_on, setup, to_day_told};
use crate::{GameEvent, GameRuleset};

/// A building `key` of the test town's record on map slot `slot`, with `roles`, free and without
/// requirements.
pub(super) fn building(key: &str, slot: Option<u8>, roles: BuildingRoles) -> BuildingDef {
    BuildingDef {
        key: String::from(key),
        name: "Building".into(),
        slot,
        supersedes: vec![],
        forbidden_with: None,
        auto_with: vec![],
        requires: vec![],
        cost: vec![],
        roles,
    }
}

/// The growth tests' rules with the neutral garrisons' week at `odds` (without, with a fort
/// level) in 101 and a Tavern, building 13 (no map slot).
fn rules_garrison(odds: [u16; 2]) -> GameRuleset {
    rules_on([1, 0, 0], |towns| {
        towns.params.as_mut().unwrap().garrison_week.odds = odds;
        let tavern = BuildingRoles {
            tavern: true,
            ..BuildingRoles::default()
        };
        towns.towns[0].buildings.push(building("b13", None, tavern));
    })
}

// covers: TWN-GRO-006, TWN-VEW-002
#[test]
fn a_neutral_town_s_garrison_grows_each_week_from_the_towns_stream_and_is_told() {
    // Every week grows (101 in 101), but the level drawn from week 1's sum may be the record's
    // third, which it lacks (nothing then).
    let rules = rules_garrison([101, 101]);
    let (core, part) = (rules.core(), rules.towns());
    let params = rules.adventure().params();
    let army = (params.army_slots(), params.max_stack_count());
    let (mut grew, mut seen) = (0, 0);
    // T3's garrison empty for some seeds (a new stack shows at level 1), its setup's for others.
    for seed in 0..16 {
        let mut setup = setup(&rules);
        if seed % 2 == 0 {
            setup.towns[3].garrison = Some(Vec::new());
        }
        let mut game = built_seeded(&rules, &setup, seed);
        // Red's Tavern in T0: red sees the neutral T3's stacks (level 1).
        let tavern = BuildingId::from_index(13).unwrap();
        game.towns_mut().build(T0, tavern, part, core, false);
        let map = game.adventure().map();
        let (neutral, owned) = (map.town_object(T3).unwrap(), map.town_object(T0).unwrap());
        let garrison = game.adventure().town_garrison(neutral).unwrap().to_vec();
        let kept = game.adventure().town_garrison(owned).unwrap().to_vec();
        // The towns' stream on day 8: the week's kind, then T3's week after its growth.
        let mut stream = Rng::for_stream(seed, StreamId::TOWNS);
        let kind = h3_towns::draw_week(game.towns(), part, core, false, false, &mut stream);
        let before = game.towns().get(T3).unwrap();
        let grown = h3_towns::week_growth(before, part, core, &kind, &GrowthInputs::NEUTRAL);
        let week = h3_towns::garrison_week(&grown, part, core, 1, &garrison, army, &mut stream);
        let events = to_day_told(&rules, &mut game, 8);
        let started = events
            .iter()
            .position(|e| matches!(e, GameEvent::WeekStarted { .. }))
            .unwrap();
        let told: Vec<_> = events[started..]
            .iter()
            .filter_map(|e| match *e {
                GameEvent::TownStackSeen {
                    player,
                    town,
                    stack,
                    ..
                } if player == RED && town == T3 => Some(stack.creature),
                _ => None,
            })
            .collect();
        let now = game.adventure().town_garrison(neutral).unwrap();
        if let Some(week) = week {
            grew += 1;
            assert_eq!(now, &week.stacks[..], "seed {seed}");
            assert_eq!(game.towns().get(T3), Some(&week.town), "seed {seed}");
            // TWN-VEW-002: red's Tavern shows the stacks' creatures in slot order (level 1, no
            // count): told where they changed (a new stack, not one merged into its creature's).
            let creatures = |stacks: &[ArmyStack]| stacks.iter().map(|s| s.creature).collect();
            let after: Vec<_> = creatures(&week.stacks);
            if after == creatures(&garrison) {
                assert!(told.is_empty(), "seed {seed}");
            } else {
                seen += 1;
                assert_eq!(told, after, "seed {seed}");
            }
        } else {
            assert_eq!(now, &garrison[..], "seed {seed}");
            assert_eq!(game.towns().get(T3), Some(&grown), "seed {seed}");
            assert!(told.is_empty(), "seed {seed}");
        }
        // An owned town's garrison never grows.
        assert_eq!(game.adventure().town_garrison(owned).unwrap(), &kept[..]);
    }
    assert!(grew > 0 && seen > 0, "{grew} of 16 grew, {seen} seen");
}

// covers: TWN-GRO-006
#[test]
fn a_neutral_town_whose_week_fails_its_odds_keeps_its_garrison() {
    // The odds' draw alone (0 in 101): T3's garrison as it was, its counts its week's growth.
    let rules = rules_garrison([0, 0]);
    let mut game = built_seeded(&rules, &setup(&rules), 7);
    let neutral = game.adventure().map().town_object(T3).unwrap();
    let garrison = game.adventure().town_garrison(neutral).unwrap().to_vec();
    let _ = to_day_told(&rules, &mut game, 8);
    assert_eq!(
        game.adventure().town_garrison(neutral).unwrap(),
        &garrison[..]
    );
    assert_eq!(game.towns().get(T3).unwrap().available(0), 15 + 15 / 2);
}
