//! Tests of the reachable-tiles query (E08 T7b): the points and paths it reports, the tiles it
//! leaves out, and a property: every path it reports is one `MoveHero` accepts, ending with the
//! points it reports, and every tile one step away that `MoveHero` accepts is among its tiles
//! (not the other way round: a tile next to the hero may be reachable only around a corner, from
//! a swamp road over the next road tile, say).

extern crate std;

use std::string::String;
use std::vec;
use std::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::{HeroId, PlayerId};
use proptest::prelude::{any, prop_assert, prop_assert_eq, proptest};

use super::*;
use crate::command::{AdventureCommand, AdventureEvent};
use crate::movement::DIRECTIONS;
use crate::testing::{GNOLL, PEASANT, PIKEMAN, WOLF, hero_at, movement_rules, world, world_state};

const RED: PlayerId = PlayerId::new(0);

fn p(x: u16, y: u16) -> Pos {
    Pos::new(x, y, 0)
}

fn reach(state: &AdventureState, rules: &(Ruleset, AdventureRuleset), hero: u16) -> Reach {
    state.reach(&rules.0, &rules.1, HeroId::new(hero)).unwrap()
}

/// Moves hero `hero` of red along `path`; the events, or the error.
fn walk(
    state: &mut AdventureState,
    rules: &(Ruleset, AdventureRuleset),
    hero: u16,
    path: Vec<Pos>,
) -> Result<Vec<AdventureEvent>, crate::AdventureCommandError> {
    let mut events = Vec::new();
    let command = AdventureCommand::MoveHero {
        hero: HeroId::new(hero),
        path,
    };
    let result = state.apply(&rules.0, &rules.1, RED, &command, &mut events);
    assert!(
        result.is_ok() || events.is_empty(),
        "a rejected move emitted events"
    );
    result.map(|()| events)
}

// covers: ADV-MOV-003, ADV-MOV-005, ADV-MOV-007
#[test]
fn the_query_reports_each_tile_with_its_points_and_path() {
    let rules = movement_rules(false);
    let state = world_state(
        &rules,
        &["gggGGg", "gggGGg", "gggggg"],
        vec![hero_at(1, 0, p(0, 0), &[PEASANT])],
    );
    let reach = reach(&state, &rules, 1);
    assert_eq!(reach.origin(), p(0, 0));
    assert_eq!(reach.points_left(p(0, 0)), Some(1300));
    assert_eq!(reach.path(p(0, 0)), Some(vec![]));
    assert_eq!(reach.points_left(p(1, 0)), Some(1200));
    assert_eq!(reach.points_left(p(1, 1)), Some(1159));
    assert_eq!(reach.path(p(3, 0)), Some(vec![p(1, 0), p(2, 0), p(3, 0)]));
    assert_eq!(reach.points_left(p(3, 0)), Some(1000));
    // Along the road: onto it at grass's 100, then 50 a step.
    assert_eq!(reach.points_left(p(4, 0)), Some(950));
    // Every tile but the hero's own, row by row.
    let tiles: Vec<Pos> = reach.tiles().map(|(pos, _)| pos).collect();
    assert_eq!(tiles.len(), 17);
    assert_eq!(tiles[0], p(1, 0));
    assert_eq!(tiles[16], p(5, 2));
    assert_eq!(state.reach(&rules.0, &rules.1, HeroId::new(9)), None);
}

// covers: ADV-MOV-007
#[test]
fn the_query_leaves_out_what_a_step_may_not_enter() {
    let rules = movement_rules(true);
    let state = world_state(
        &rules,
        &["g#~g", "BVTg", "gggg"],
        vec![
            hero_at(1, 0, p(0, 0), &[PEASANT]),
            hero_at(2, 1, p(1, 2), &[PEASANT]),
        ],
    );
    let reach = reach(&state, &rules, 1);
    // Rock, sea, an object's blocked and visitable tiles, a town's gate, another hero.
    for pos in [p(1, 0), p(2, 0), p(0, 1), p(1, 1), p(2, 1), p(1, 2)] {
        assert_eq!(reach.points_left(pos), None, "{pos:?}");
        assert_eq!(reach.path(pos), None);
    }
    // Hemmed in: the hero reaches nothing.
    assert_eq!(reach.tiles().count(), 0);
}

// covers: ADV-MOV-006
#[test]
fn a_better_path_found_later_replaces_the_first() {
    // From swamp (175 a step, 247 a diagonal): (2, 0) is first reached over the swamp at
    // (1, 0) with 1300 − 175 − 175 = 950 points, then from the road at (1, 1) by a road diagonal
    // with 1300 − 247 − 70 = 983.
    let rules = movement_rules(false);
    let state = world_state(
        &rules,
        &["wwG", "gGg"],
        vec![hero_at(1, 0, p(0, 0), &[PEASANT])],
    );
    let reach = reach(&state, &rules, 1);
    assert_eq!(reach.points_left(p(2, 0)), Some(983));
    assert_eq!(reach.path(p(2, 0)), Some(vec![p(1, 1), p(2, 0)]));
}

// covers: ADV-MOV-006
#[test]
fn the_query_searches_only_as_far_as_the_points_go() {
    let rules = movement_rules(false);
    let rows = ["gggggggggggggggg", "gggggggggggggggg"];
    let mut state = world_state(
        &rules,
        &rows,
        vec![
            hero_at(1, 0, p(0, 0), &[PEASANT]),
            hero_at(2, 0, p(0, 1), &[PEASANT]),
        ],
    );
    // Twelve steps leave 100: two tiles at most (the cheapest step costs 50).
    walk(&mut state, &rules, 1, (1..=12).map(|x| p(x, 0)).collect()).unwrap();
    let reach = reach(&state, &rules, 1);
    assert_eq!(reach.points_left(p(12, 0)), Some(100));
    assert_eq!(reach.points_left(p(0, 0)), None);
    assert_eq!(reach.path(p(0, 0)), None);
    // One step on grass, or a diagonal by the exception, onto the five tiles around (the map's
    // top row is row 0): each leaves nothing.
    let tiles: Vec<(Pos, u32)> = reach.tiles().collect();
    assert_eq!(tiles.len(), 5);
    assert!(tiles.iter().all(|&(_, left)| left == 0));
    assert_eq!(reach.points_left(p(14, 0)), None);
}

#[test]
fn the_query_stays_on_the_heros_level() {
    let rules = movement_rules(false);
    let setup = world(
        &["ggg", "ggg", "ggg", "ggg"],
        2,
        vec![
            hero_at(1, 0, Pos::new(0, 0, 1), &[PEASANT]),
            hero_at(2, 1, p(1, 1), &[PEASANT]),
        ],
    );
    let state = AdventureState::new(&rules.0, &rules.1, &setup).unwrap();
    let reach = reach(&state, &rules, 1);
    assert_eq!(reach.points_left(Pos::new(1, 1, 1)), Some(1159));
    assert_eq!(reach.points_left(p(1, 1)), None);
    assert_eq!(reach.points_left(p(1, 0)), None);
}

/// The tiles of the property's maps, weighted toward walkable ones.
const TILES: &[char] = &[
    'g', 'g', 'g', 'g', 's', 's', 'w', 'w', 'G', 'G', 'S', 'W', 'd', '~', '#', 'B', 'V', 'T',
];
const CREATURES: [h3_content::CreatureTypeId; 4] = [PIKEMAN, GNOLL, PEASANT, WOLF];

/// Every tile one step from `at` on the map.
fn around(state: &AdventureState, at: Pos) -> Vec<Pos> {
    DIRECTIONS
        .iter()
        .filter_map(|&(dx, dy)| {
            let x = at.x.checked_add_signed(i16::from(dx))?;
            let y = at.y.checked_add_signed(i16::from(dy))?;
            Some(Pos::new(x, y, at.level)).filter(|&pos| state.map().tile(pos).is_some())
        })
        .collect()
}

proptest! {
    /// On random maps, armies and profiles, from the hero's tile and after up to three moves
    /// along reported paths: every reported path is accepted and ends with the reported points;
    /// the tile of an accepted one-step move is reported with at least the points it leaves; a
    /// rejected move changes nothing; the query is deterministic.
    #[test]
    fn the_query_and_move_hero_agree(
        cells in proptest::collection::vec(0usize..TILES.len(), 64),
        width in 3usize..=8,
        height in 3usize..=8,
        start in (0usize..8, 0usize..8),
        others in proptest::collection::vec((0usize..8, 0usize..8), 0..3),
        creatures in proptest::collection::vec(0usize..4, 1..4),
        faithful in any::<bool>(),
        picks in proptest::collection::vec(0usize..1000, 0..4),
    ) {
        let rules = movement_rules(faithful);
        let (hx, hy) = (start.0 % width, start.1 % height);
        let mut grid: Vec<Vec<char>> = (0..height)
            .map(|y| (0..width).map(|x| TILES[cells[y * 8 + x]]).collect())
            .collect();
        let standable = |c: char| !matches!(c, '~' | '#' | 'B');
        if !standable(grid[hy][hx]) {
            grid[hy][hx] = 'g';
        }
        let army: Vec<_> = creatures.iter().map(|&c| CREATURES[c]).collect();
        let mut heroes = vec![hero_at(1, 0, p(hx as u16, hy as u16), &army)];
        for (id, &(x, y)) in (2..).zip(&others) {
            let (x, y) = (x % width, y % height);
            let taken = heroes.iter().any(|h| h.pos == p(x as u16, y as u16));
            if standable(grid[y][x]) && !taken {
                heroes.push(hero_at(id, (id % 2) as u8, p(x as u16, y as u16), &[PEASANT]));
            }
        }
        let rows: Vec<String> = grid.iter().map(|row| row.iter().collect()).collect();
        let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
        let mut state = world_state(&rules, &rows, heroes);
        for round in 0..=picks.len() {
            let reach = reach(&state, &rules, 1);
            prop_assert_eq!(&reach, &state.reach(&rules.0, &rules.1, HeroId::new(1)).unwrap());
            let origin = state.hero(HeroId::new(1)).unwrap().pos;
            for (pos, left) in reach.tiles() {
                let path = reach.path(pos).unwrap();
                let mut moved = state.clone();
                let events = walk(&mut moved, &rules, 1, path.clone());
                prop_assert_eq!(events.map(|e| e.len()), Ok(path.len()));
                let hero = moved.hero(HeroId::new(1)).unwrap();
                prop_assert_eq!((hero.pos, hero.movement), (pos, left));
            }
            for next in around(&state, origin) {
                let mut moved = state.clone();
                match walk(&mut moved, &rules, 1, vec![next]) {
                    Ok(_) => {
                        let left = moved.hero(HeroId::new(1)).unwrap().movement;
                        prop_assert!(reach.points_left(next).is_some_and(|best| best >= left));
                    }
                    Err(_) => prop_assert_eq!(&moved, &state),
                }
            }
            let tiles: Vec<(Pos, u32)> = reach.tiles().collect();
            let Some(&pick) = picks.get(round) else { break };
            if tiles.is_empty() {
                break;
            }
            let (pos, _) = tiles[pick % tiles.len()];
            walk(&mut state, &rules, 1, reach.path(pos).unwrap()).unwrap();
        }
    }
}
