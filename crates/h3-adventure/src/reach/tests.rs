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
use crate::command::{AdventureCommand, AdventureCommandError, AdventureEvent, AdventureEventKind};
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
) -> Result<Vec<AdventureEvent>, AdventureCommandError> {
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
    result.map(|_| events)
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
    state.explore_all();
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

// covers: ADV-MOV-006
#[test]
fn more_points_on_a_tile_can_leave_fewer_after_the_next_diagonal() {
    // Strict-fixed: the clean diagonal exception alone. Peasants and gnolls have no native
    // terrain; 598 points on the grass at (4, 1).
    let rules = movement_rules(false);
    let mut state = world_state(
        &rules,
        &["GdwSS", "sgdgg", "GWGsS", "#wws#", "#Wsws"],
        vec![hero_at(1, 0, p(4, 1), &[PEASANT, GNOLL])],
    );
    state.set_movement(HeroId::new(1), 598);
    let query = reach(&state, &rules, 1);
    // The query keeps the most points on the swamp at (2, 3): 257, over grass and the grass road
    // (100 + 141 + 100). From there the diagonal onto the road at (1, 4) costs the full 247 (257 is
    // above ⌊175 × √2⌋ = 247): 10 left, below the road's 50, so none.
    assert_eq!(query.points_left(p(2, 3)), Some(257));
    assert_eq!(
        query.path(p(1, 4)),
        Some(vec![p(3, 1), p(2, 2), p(2, 3), p(1, 4)])
    );
    assert_eq!(query.points_left(p(1, 4)), Some(0));
    // Over the sand (141 + 212) the swamp is reached with 245, inside the exception's band (175 to
    // 247): the same diagonal costs 175 and leaves 70, one more road step.
    let mut moved = state.clone();
    walk(&mut moved, &rules, 1, vec![p(3, 2), p(2, 3), p(1, 4)]).unwrap();
    assert_eq!(moved.hero(HeroId::new(1)).unwrap().movement, 70);
    // With 50 points the window holds only the tiles around the hero: (4, 4) is in its columns
    // but below its rows.
    state.set_movement(HeroId::new(1), 50);
    assert_eq!(reach(&state, &rules, 1).points_left(p(4, 4)), None);
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
    let state =
        AdventureState::new(&rules.0, &rules.1, &setup, &mut crate::testing::draws()).unwrap();
    let reach = reach(&state, &rules, 1);
    assert_eq!(reach.points_left(Pos::new(1, 1, 1)), Some(1159));
    assert_eq!(reach.points_left(p(1, 1)), None);
    assert_eq!(reach.points_left(p(1, 0)), None);
}

/// The tiles of the property's maps, weighted toward walkable ones.
const TILES: &[char] = &[
    'g', 'g', 'g', 'g', 's', 's', 'w', 'w', 'G', 'G', 'S', 'W', 'd', '~', '#', 'B', 'V', 'T', 'P',
    'C', 'c', 'O', 'Y', 'K', 'X',
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

// covers: ADV-MOV-008
#[test]
fn the_query_keeps_to_the_explored_tiles() {
    let rules = movement_rules(false);
    let mut state = world_state(
        &rules,
        &["gggggggggggggggg"],
        vec![hero_at(1, 0, p(0, 0), &[PEASANT])],
    );
    // 1300 points pay thirteen steps, but only (1, 0) to (5, 0) are explored.
    let tiles: Vec<Pos> = reach(&state, &rules, 1)
        .tiles()
        .map(|(pos, _)| pos)
        .collect();
    assert_eq!(tiles, (1..=5).map(|x| p(x, 0)).collect::<Vec<_>>());
    assert_eq!(reach(&state, &rules, 1).points_left(p(6, 0)), None);
    // Standing on (5, 0), the hero has explored up to (10, 0).
    walk(&mut state, &rules, 1, (1..=5).map(|x| p(x, 0)).collect()).unwrap();
    let reach = reach(&state, &rules, 1);
    assert_eq!(reach.points_left(p(10, 0)), Some(300));
    assert_eq!(reach.points_left(p(11, 0)), None);
}

proptest! {
    /// On random maps, armies and profiles, from the hero's tile and after up to three moves
    /// along reported paths: every reported path is accepted and ends with the reported points,
    /// on its last tile (a chart's, a tower's or a tent's entered, E08 T14a, T15a; a gate's
    /// crossed or stopped on by a holder of its key, T15b) or, for a pile's, a guard's or (without
    /// the key) a gate's visit, on the tile before, one event per step and at most one for the
    /// visit; the tile of an accepted one-step move is reported with at least the points it
    /// leaves; a rejected move changes nothing; the query is deterministic. The hero's owner
    /// holds the gates' key from the start in half the cases, and takes it at a tent in others.
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
        key in any::<bool>(),
    ) {
        let rules = movement_rules(faithful);
        let (hx, hy) = (start.0 % width, start.1 % height);
        let mut grid: Vec<Vec<char>> = (0..height)
            .map(|y| (0..width).map(|x| TILES[cells[y * 8 + x]]).collect())
            .collect();
        // No hero stands on a pile's, a guard's or a gate's tile (ADV-MAP-001); one may on a
        // chart's, a tower's or a tent's (E08 T14a).
        let standable = |c: char| !matches!(c, '~' | '#' | 'B' | 'P' | 'K' | 'X');
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
        if key {
            state.give_key(RED, 0);
        }
        for round in 0..=picks.len() {
            let reach = reach(&state, &rules, 1);
            prop_assert_eq!(&reach, &state.reach(&rules.0, &rules.1, HeroId::new(1)).unwrap());
            let origin = state.hero(HeroId::new(1)).unwrap().pos;
            for (pos, left) in reach.tiles() {
                let path = reach.path(pos).unwrap();
                let mut moved = state.clone();
                let events = walk(&mut moved, &rules, 1, path.clone()).unwrap();
                let steps = events
                    .iter()
                    .filter(|e| matches!(e.kind, AdventureEventKind::HeroStepped { .. }))
                    .count();
                // A visit from the tile before steps one tile less; a gate the owner holds the
                // key of is no visit.
                let visit = state
                    .visit_at(&rules.1, RED, pos)
                    .is_some_and(|v| !v.pass && v.mode == crate::ruleset::Visit::Neighbour);
                prop_assert_eq!(steps + usize::from(visit), path.len());
                prop_assert!(events.len() <= steps + 1);
                let at = match path.len().checked_sub(2) {
                    _ if !visit => pos,
                    Some(before) => path[before],
                    None => origin,
                };
                let hero = moved.hero(HeroId::new(1)).unwrap();
                prop_assert_eq!((hero.pos, hero.movement), (at, left));
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

// covers: ADV-MOV-007, ADV-OBJ-003
#[test]
fn the_tiles_a_removed_object_frees_are_reached_and_walked() {
    // A wall of a blocker, a pile and a town-like gate at column 1; the hero at (0, 0).
    let rules = movement_rules(false);
    let mut state = world_state(
        &rules,
        &["gBg", "gPg", "gTg"],
        vec![hero_at(1, 0, p(0, 0), &[PEASANT])],
    );
    let walled = reach(&state, &rules, 1);
    // The blocker and the untyped gate stop the search; the pile is a destination, the visit's
    // (E08 T13b), a diagonal step paid from (0, 0), where the hero stays; nothing past it.
    assert_eq!(walled.points_left(p(1, 0)), None);
    assert_eq!(walled.points_left(p(1, 2)), None);
    assert_eq!(walled.points_left(p(1, 1)), Some(1300 - 141));
    assert_eq!(walled.path(p(1, 1)), Some(vec![p(1, 1)]));
    assert_eq!(walled.points_left(p(2, 1)), None);
    assert_eq!(
        walk(&mut state.clone(), &rules, 1, vec![p(1, 1), p(2, 1)]),
        Err(AdventureCommandError::PastVisit { step: 1 })
    );
    // The pile (object 1 in map order) leaves the map: its tile is reached and walked through.
    state.remove_object(1);
    let open = reach(&state, &rules, 1);
    assert!(open.points_left(p(1, 1)).is_some());
    assert!(open.points_left(p(2, 1)).is_some());
    assert_eq!(open.points_left(p(1, 0)), None);
    walk(&mut state, &rules, 1, vec![p(1, 1), p(2, 1)]).unwrap();
}

/// Red's hero at `at` on three rows of grass whose middle one holds `chart` at (2, 1).
fn chart_world(rules: &(Ruleset, AdventureRuleset), chart: char, at: Pos) -> AdventureState {
    let middle = std::format!("gg{chart}ggggggggggggg");
    world_state(
        rules,
        &["gggggggggggggggg", &middle, "gggggggggggggggg"],
        vec![hero_at(1, 0, at, &[PEASANT])],
    )
}

// covers: ADV-OBJ-003
#[test]
fn a_chart_is_a_destination_with_the_points_left_on_it() {
    let rules = movement_rules(false);
    let state = world_state(
        &rules,
        &["ggCggggg"],
        vec![hero_at(1, 0, p(0, 0), &[PEASANT])],
    );
    let reach = reach(&state, &rules, 1);
    assert_eq!(reach.points_left(p(2, 0)), Some(1300 - 200));
    assert_eq!(reach.path(p(2, 0)), Some(vec![p(1, 0), p(2, 0)]));
    // Nothing past it: the path ends at the visit.
    assert_eq!(reach.points_left(p(3, 0)), None);
    let mut moved = state.clone();
    walk(&mut moved, &rules, 1, reach.path(p(2, 0)).unwrap()).unwrap();
    assert_eq!(moved.hero(HeroId::new(1)).unwrap().movement, 1100);
}

// covers: ADV-OBJ-004
#[test]
fn the_query_keeps_off_a_chart_s_top() {
    let rules = movement_rules(false);
    // From above, the chart is reached around a corner: a diagonal and a step along the row.
    let reach_above = reach(&chart_world(&rules, 'C', p(2, 0)), &rules, 1);
    assert_eq!(reach_above.points_left(p(2, 1)), Some(1300 - 141 - 100));
    let path = reach_above.path(p(2, 1)).unwrap();
    assert_eq!((path.len(), path[0].y), (2, 1));
    // An open chart, straight down.
    let open = reach(&chart_world(&rules, 'c', p(2, 0)), &rules, 1);
    assert_eq!(open.path(p(2, 1)), Some(vec![p(2, 1)]));
    // Standing on a chart, the hero reaches the three tiles above only around a corner; on an
    // open chart, straight up.
    let on = reach(&chart_world(&rules, 'C', p(2, 1)), &rules, 1);
    for up in [p(1, 0), p(2, 0), p(3, 0)] {
        assert_eq!(on.path(up).map(|path| path.len()), Some(2), "{up:?}");
    }
    assert_eq!(on.points_left(p(2, 0)), Some(1300 - 100 - 141));
    assert_eq!(on.path(p(2, 2)), Some(vec![p(2, 2)]));
    let on_open = reach(&chart_world(&rules, 'c', p(2, 1)), &rules, 1);
    assert_eq!(on_open.path(p(2, 0)), Some(vec![p(2, 0)]));
}

// covers: ADV-VIS-005, ADV-OBJ-003
#[test]
fn a_gate_is_crossed_by_its_key_s_holders_only() {
    // A wall of rock at column 1 with a gate of key 0 in it; the hero at (0, 1).
    let rules = movement_rules(false);
    let mut state = world_state(
        &rules,
        &["g#ggg", "gXggg", "g#ggg"],
        vec![hero_at(1, 0, p(0, 1), &[PEASANT])],
    );
    state.explore_all();
    // Without the key the gate is a destination, the visit's from (0, 1): nothing past it.
    let shut = reach(&state, &rules, 1);
    assert_eq!(shut.points_left(p(1, 1)), Some(1300 - 100));
    assert_eq!(shut.path(p(1, 1)), Some(vec![p(1, 1)]));
    assert!((2..5).all(|x| shut.points_left(p(x, 1)).is_none()));
    // With it the gate's tile is one like any other: on the way, and an end too.
    state.give_key(RED, 0);
    let open = reach(&state, &rules, 1);
    assert_eq!(open.points_left(p(1, 1)), Some(1300 - 100));
    assert_eq!(open.points_left(p(4, 1)), Some(1300 - 400));
    let path = open.path(p(3, 0)).unwrap();
    assert_eq!((path.len(), path[0]), (3, p(1, 1)));
    for to in [p(1, 1), p(4, 1)] {
        let mut moved = state.clone();
        walk(&mut moved, &rules, 1, open.path(to).unwrap()).unwrap();
        let hero = moved.hero(HeroId::new(1)).unwrap();
        assert_eq!((hero.pos, Some(hero.movement)), (to, open.points_left(to)));
    }
    // A key of another colour opens nothing.
    let mut other = world_state(
        &rules,
        &["g#ggg", "gXggg", "g#ggg"],
        vec![hero_at(1, 0, p(0, 1), &[PEASANT])],
    );
    other.explore_all();
    other.give_key(RED, 1);
    assert_eq!(reach(&other, &rules, 1), shut);
}
