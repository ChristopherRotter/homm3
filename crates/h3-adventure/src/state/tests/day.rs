//! Tests of the hero's details in the adventure's day (E08 T40, survey `E08-hero-movement.md`):
//! the start of the game's points and reveals from the store (ADR-0017 point 1), the turn start's
//! points, the start-of-day reveal under `ADV-QUIRK-18`, a move's price by the hero's Pathfinding
//! and its reveals by its radius, and Cursed Ground's tiles.

use std::sync::Arc;

use h3_content::SkillId;
use h3_heroes::HeroDetails;

use super::*;
use crate::command::{AdventureCommand, AdventureCommandError, AdventureEvent};
use crate::fog::reveal;
use crate::persistent::BitGrid;
use crate::ruleset::ObjectTypeId;
use crate::setup::{ObjectSetup, Pos, TileMask};
use crate::testing::{
    BARRENS, GNOLL, HALLOW, LOGISTICS, PATHFINDING, PIKEMAN, SCOUTING, hero_at, hero_details,
    hero_rules, world,
};

type Rules = (Ruleset, AdventureRuleset);

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);

fn p(x: u16, y: u16) -> Pos {
    Pos::new(x, y, 0)
}

/// A store of heroes by id from 0: these details, each in an `Arc`.
fn store(details: Vec<HeroDetails>) -> Vec<Arc<HeroDetails>> {
    details.into_iter().map(Arc::new).collect()
}

/// A 16 × 16 grass map of one level with red's hero 1 (pikemen) at (2, 2) and blue's hero 2 at
/// (12, 12), placed and not yet started.
fn placed(rules: &Rules, rows: &[&str]) -> AdventureState {
    let setup = world(
        rows,
        1,
        vec![
            hero_at(1, 0, p(2, 2), &[PIKEMAN]),
            hero_at(2, 1, p(12, 12), &[PIKEMAN]),
        ],
    );
    AdventureState::new(&rules.0, &rules.1, &setup, &mut draws()).unwrap()
}

fn grass() -> Vec<&'static str> {
    vec!["gggggggggggggggg"; 16]
}

/// A 16 × 16 grid with the disks around these centres of these radii.
fn disks(centres: &[(Pos, u8)]) -> BitGrid {
    let mut grid = BitGrid::new(16, 16, 1);
    for &(centre, radius) in centres {
        reveal(&mut grid, centre, radius);
    }
    grid
}

fn points(state: &AdventureState) -> Vec<u32> {
    state.heroes().iter().map(|h| h.movement).collect()
}

fn walk(
    state: &mut AdventureState,
    rules: &Rules,
    heroes: &[Arc<HeroDetails>],
    path: &[Pos],
) -> Result<Vec<AdventureEvent>, AdventureCommandError> {
    let mut events = Vec::new();
    let command = AdventureCommand::MoveHero {
        hero: HeroId::new(1),
        path: path.to_vec(),
    };
    state
        .apply(&rules.0, &rules.1, heroes, RED, &command, &mut events)
        .map(|_| events)
}

// covers: ADV-MOV-001, ADV-MOV-002, ADV-FOG-001, ADV-FOG-003
#[test]
fn the_start_of_the_game_gives_the_points_and_reveals_from_the_store() {
    let rules = hero_rules(true);
    let mut state = placed(&rules, &grass());
    // Placed: no points, nothing explored (the game gives the levels at load first).
    assert_eq!(points(&state), [0, 0]);
    assert_eq!(state.explored(RED), Some(&BitGrid::new(16, 16, 1)));
    // Red's hero with Basic Logistics and Advanced Scouting, blue's without bonuses; hero 0 is
    // not on the map.
    let heroes = store(vec![
        hero_details(&[(LOGISTICS, 3)]),
        hero_details(&[(LOGISTICS, 1), (SCOUTING, 2)]),
        hero_details(&[]),
    ]);
    state.start_game(&rules.0, &rules.1, &heroes);
    assert_eq!(points(&state), [1650, 1500]);
    assert_eq!(state.explored(RED), Some(&disks(&[(p(2, 2), 7)])));
    assert_eq!(state.explored(BLUE), Some(&disks(&[(p(12, 12), 5)])));
    // Without the store (this crate's tests): the table's entry and the part's radius.
    let mut plain = placed(&rules, &grass());
    plain.start_game(&rules.0, &rules.1, &[]);
    assert_eq!(points(&plain), [1500, 1500]);
    assert_eq!(plain.explored(RED), Some(&disks(&[(p(2, 2), 5)])));
}

// covers: ADV-MOV-002, ADV-FOG-003
#[test]
fn a_gain_in_the_day_shows_at_the_next_turn_start_and_step() {
    for faithful in [true, false] {
        let rules = hero_rules(faithful);
        let mut state = placed(&rules, &grass());
        let before = store(vec![
            hero_details(&[]),
            hero_details(&[]),
            hero_details(&[]),
        ]);
        state.start_game(&rules.0, &rules.1, &before);
        // The hero learns Expert Logistics and Basic Scouting in the day: neither its points left
        // nor the explored tiles change (survey §1.4).
        let after = store(vec![
            hero_details(&[]),
            hero_details(&[(LOGISTICS, 3), (SCOUTING, 1)]),
            hero_details(&[]),
        ]);
        let unchanged = state.clone();
        state.start_turn(&rules.0, &rules.1, &after, BLUE);
        assert_eq!(state, unchanged, "blue's turn: red's hero keeps its points");
        // The day's start: strict-fixed (ADV-QUIRK-18 off) reveals around every hero with its
        // radius now; faithful waits for the hero's next step.
        state.start_day(&rules.0, &rules.1, &after);
        let grown = disks(&[(p(2, 2), 6)]);
        if faithful {
            assert_eq!(state, unchanged);
        } else {
            assert_eq!(state.explored(RED), Some(&grown));
            assert_eq!(points(&state), [1500, 1500]);
        }
        // Red's turn: the grown points.
        state.start_turn(&rules.0, &rules.1, &after, RED);
        assert_eq!(points(&state), [1950, 1500]);
        // A step reveals with the grown radius in both profiles.
        walk(&mut state, &rules, &after, &[p(3, 2)]).unwrap();
        let start = if faithful { 5 } else { 6 };
        assert_eq!(
            state.explored(RED),
            Some(&disks(&[(p(2, 2), start), (p(3, 2), 6)]))
        );
    }
}

// covers: ADV-MOV-003, ADV-MOV-008
#[test]
fn a_move_and_the_reach_price_the_steps_by_the_heros_pathfinding() {
    // Sand (150; 125, 100, 100 by Pathfinding level): five steps east from (2, 2), the shortest
    // path there.
    let rules = hero_rules(false);
    let rows = vec!["ssssssssssssssss"; 16];
    let path: Vec<Pos> = (3..=7).map(|x| p(x, 2)).collect();
    for (level, cost) in [(0, 150), (1, 125), (2, 100), (3, 100)] {
        let mut state = placed(&rules, &rows);
        let skills: &[(SkillId, u8)] = if level == 0 {
            &[]
        } else {
            &[(PATHFINDING, level)]
        };
        let heroes = store(vec![hero_details(&[]), hero_details(skills)]);
        state.start_game(&rules.0, &rules.1, &heroes);
        let reach = state
            .reach(&rules.0, &rules.1, &heroes, HeroId::new(1))
            .unwrap();
        assert_eq!(reach.points_left(p(7, 2)), Some(1500 - 5 * cost), "{level}");
        walk(&mut state, &rules, &heroes, &path).unwrap();
        assert_eq!(
            state.hero(HeroId::new(1)).unwrap().movement,
            1500 - 5 * cost
        );
    }
}

/// A ground of `ty` lying on the 3 × 2 tiles left of and above `anchor` (its picture only).
fn ground(ty: ObjectTypeId, anchor: Pos) -> ObjectSetup {
    ObjectSetup {
        anchor,
        blocked: TileMask::default(),
        visitable: TileMask::default(),
        object_type: Some(ty),
        amount: None,
        drawn: TileMask::from_bits(0x707),
        monster: None,
        guards: None,
    }
}

// covers: ADV-MOV-004, ADV-MOV-006
#[test]
fn cursed_ground_takes_the_native_price_away_on_its_tiles() {
    // The test rules' barrens take the native price away (as Cursed Ground), the hallow does not.
    // A barrens on columns 2 to 4 of rows 1 and 2, a hallow over its right part (columns 4 to 6):
    // on each tile the last ground in map order decides (0x4FCE20, as the battle's ground).
    let rules = hero_rules(true);
    let mut setup = world(
        &["wwwwwwwwwwwwwwww"; 16],
        1,
        vec![hero_at(1, 0, p(2, 2), &[GNOLL])],
    );
    setup.map.objects.push(ground(BARRENS, p(4, 2)));
    setup.map.objects.push(ground(HALLOW, p(6, 2)));
    let mut state = AdventureState::new(&rules.0, &rules.1, &setup, &mut draws()).unwrap();
    let cursed: Vec<u16> = (0..8)
        .filter(|&x| state.map().tile(p(x, 2)).unwrap().no_native)
        .collect();
    assert_eq!(cursed, [2, 3]);
    assert!(!state.map().tile(p(2, 3)).unwrap().no_native);
    // The bit follows from the objects: the hash leaves it out.
    let mut plain = setup.clone();
    plain.map.objects.clear();
    let plain = AdventureState::new(&rules.0, &rules.1, &plain, &mut draws()).unwrap();
    assert!(plain.map().tiles().iter().all(|t| !t.no_native));
    // Gnolls (native to swamp) step east from (2, 2): 175 from each cursed tile, then 100.
    state.start_game(&rules.0, &rules.1, &[]);
    let path: Vec<Pos> = (3..=6).map(|x| p(x, 2)).collect();
    walk(&mut state, &rules, &[], &path).unwrap();
    assert_eq!(
        state.hero(HeroId::new(1)).unwrap().movement,
        1500 - 175 - 175 - 100 - 100
    );
    // The drop after a step onto a cursed tile prices its terrain: 174 points left on (3, 2)
    // fall to 0.
    let mut tired = AdventureState::new(&rules.0, &rules.1, &setup, &mut draws()).unwrap();
    tired.start_game(&rules.0, &rules.1, &[]);
    tired.set_movement(HeroId::new(1), 175 + 174);
    walk(&mut tired, &rules, &[], &[p(3, 2)]).unwrap();
    assert_eq!(tired.hero(HeroId::new(1)).unwrap().movement, 0);
}
