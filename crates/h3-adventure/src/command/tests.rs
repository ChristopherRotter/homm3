//! Tests of `MoveHero` (E08 T7b): a move pays each step from the points left, the path is
//! checked step by step and rejected whole, one event per step. E08 T8a: the fog of war around
//! moves: the reveals at the start and after each step, explored tiles only, the steps' viewers.

extern crate std;

use std::string::{String, ToString};
use std::vec;
use std::vec::Vec;

use h3_content::{ResourceId, Ruleset};
use h3_contracts::{HeroId, PlayerId};

use super::*;
use crate::fog::reveal;
use crate::persistent::BitGrid;
use crate::ruleset::AdventureRuleset;
use crate::ruleset::ObjectTypeId;
use crate::setup::SetupError;
use crate::state::AdventureState;
use crate::testing::{
    CHART, FIRE, GNOLL, GUARD, MAX_STACK, OPEN_CHART, PEASANT, PIKEMAN, TOWER, WOLF, WOOD_PILE,
    draws, hero_at, movement_rules, world, world_state,
};

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);

fn p(x: u16, y: u16) -> Pos {
    Pos::new(x, y, 0)
}

fn move_hero(
    state: &mut AdventureState,
    rules: &(Ruleset, AdventureRuleset),
    player: PlayerId,
    hero: u16,
    path: &[Pos],
) -> Result<Vec<AdventureEvent>, AdventureCommandError> {
    let mut events = Vec::new();
    let command = AdventureCommand::MoveHero {
        hero: HeroId::new(hero),
        path: path.to_vec(),
    };
    let result = state.apply(&rules.0, &rules.1, &[], player, &command, &mut events);
    assert!(
        result.is_ok() || events.is_empty(),
        "a rejected move emitted events"
    );
    result.map(|_| events)
}

fn hero(state: &AdventureState, id: u16) -> (Pos, u32) {
    let hero = state.hero(HeroId::new(id)).unwrap();
    (hero.pos, hero.movement)
}

// covers: ADV-MOV-003, ADV-MOV-005, ADV-MOV-006, ADV-MOV-007
#[test]
fn a_move_pays_each_step_and_emits_one_event_per_step() {
    let rules = movement_rules(false);
    let mut state = world_state(
        &rules,
        &["gggGGg", "gggGGg", "gggggg"],
        vec![hero_at(1, 0, p(0, 0), &[PEASANT])],
    );
    assert_eq!(hero(&state, 1), (p(0, 0), 1300));
    let before = state.state_hash();
    // Grass 100, a grass diagonal 141, grass onto the road 100, a road diagonal 70, off the
    // road onto grass at the grass under the road, 100.
    let path = [p(1, 0), p(2, 1), p(3, 1), p(4, 0), p(5, 0)];
    let events = move_hero(&mut state, &rules, RED, 1, &path).unwrap();
    assert_eq!(
        hero(&state, 1),
        (p(5, 0), 1300 - 100 - 141 - 100 - 70 - 100)
    );
    assert_ne!(state.state_hash(), before);
    // Only red has a hero: only red sees the steps (ADV-FOG-004).
    let red = Viewers::NONE.with(RED);
    let stepped = |from, to| AdventureEvent {
        kind: AdventureEventKind::HeroStepped {
            hero: HeroId::new(1),
            from: Some(from),
            to: Some(to),
            seen_from: red,
            seen_to: red,
        },
        seen_by: red,
    };
    assert_eq!(
        events,
        vec![
            stepped(p(0, 0), p(1, 0)),
            stepped(p(1, 0), p(2, 1)),
            stepped(p(2, 1), p(3, 1)),
            stepped(p(3, 1), p(4, 0)),
            stepped(p(4, 0), p(5, 0)),
        ]
    );
}

// covers: ADV-MOV-006
#[test]
fn the_last_steps_of_a_day() {
    let rules = movement_rules(false);
    let rows = ["gggggggggggggggg", "gggggggggggggggg"];
    let start = || {
        let mut state = world_state(&rules, &rows, vec![hero_at(1, 0, p(0, 0), &[PEASANT])]);
        state.explore_all();
        state
    };
    // Twelve straight steps leave 100 of 1300: a diagonal then costs 100 (the exception).
    let mut state = start();
    let mut path: Vec<Pos> = (1..=12).map(|x| p(x, 0)).collect();
    path.push(p(13, 1));
    move_hero(&mut state, &rules, RED, 1, &path).unwrap();
    assert_eq!(hero(&state, 1), (p(13, 1), 0));
    // Thirteen leave none, and a fourteenth step is refused.
    let mut state = start();
    let path: Vec<Pos> = (1..=14).map(|x| p(x, 0)).collect();
    assert_eq!(
        move_hero(&mut state, &rules, RED, 1, &path),
        Err(AdventureCommandError::TooFewPoints {
            step: 13,
            cost: 100,
            points: 0
        })
    );
    // Eight diagonals leave 172; a straight step would leave 72, fewer than the cheapest step
    // from grass (100): none are left (D-3).
    let mut state = start();
    let mut path: Vec<Pos> = (1..=8).map(|x| p(x, x % 2)).collect();
    path.push(p(9, 0));
    move_hero(&mut state, &rules, RED, 1, &path).unwrap();
    assert_eq!(hero(&state, 1), (p(9, 0), 0));
}

// covers: ADV-MOV-006
#[test]
fn the_points_left_after_a_step_reproduce_batch_2() {
    // E08 batch 2 (experiments/results/E08-batch-2.md, D3a-D3d): after a step the points left
    // drop to 0 only when fewer than the cheapest straight step from the new tile remain (its
    // road's cost, else 100): the reading, observed in SoD; VCMI drops them below the step's own
    // cost, the wiki never. A Peasant has 1300 points; a step costs what the tile left costs.
    let rules = movement_rules(false);
    let walk = |row: &str, steps: u16| {
        let mut state = world_state(&rules, &[row], vec![hero_at(1, 0, p(0, 0), &[PEASANT])]);
        state.explore_all();
        let path: Vec<Pos> = (1..=steps).map(|x| p(x, 0)).collect();
        (
            move_hero(&mut state, &rules, RED, 1, &path),
            hero(&state, 1),
        )
    };
    // D3a: 200 left on sand, onto a cobblestone road: 50 left (not below the road's 50), and a
    // further road step (VCMI: 0 and no step).
    assert_eq!(walk("ggsssssssSSs", 9).1, (p(9, 0), 50));
    assert_eq!(walk("ggsssssssSSs", 10).1, (p(10, 0), 0));
    // D3b: the same onto a dirt road: 50 is below its 75, so none are left and no step follows
    // (the wiki: 50 left).
    assert_eq!(walk("ggsssssssddd", 9).1, (p(9, 0), 0));
    assert_eq!(
        walk("ggsssssssddd", 10).0,
        Err(AdventureCommandError::TooFewPoints {
            step: 9,
            cost: 75,
            points: 0
        })
    );
    // D3c: 100 left on a tile without a road stay (VCMI: 0).
    assert_eq!(walk("sssssssss", 8).1, (p(8, 0), 100));
    // D3d: 50 left there drop to 0 (the wiki: 50).
    assert_eq!(walk("ggssssssss", 9).1, (p(9, 0), 0));
}

// covers: ADV-MOV-004, ADV-MOV-006
#[test]
fn a_native_army_moves_at_the_native_cost() {
    // Gnolls (speed 4: 1500 points) walk swamp at 100 a step.
    let rules = movement_rules(true);
    let mut state = world_state(&rules, &["wwww"], vec![hero_at(1, 0, p(0, 0), &[GNOLL])]);
    move_hero(&mut state, &rules, RED, 1, &[p(1, 0), p(2, 0), p(3, 0)]).unwrap();
    assert_eq!(hero(&state, 1), (p(3, 0), 1200));
}

/// Checks that a rejected move changed nothing.
fn rejected(
    state: &mut AdventureState,
    rules: &(Ruleset, AdventureRuleset),
    player: PlayerId,
    hero: u16,
    path: &[Pos],
) -> AdventureCommandError {
    let before = state.clone();
    let error = move_hero(state, rules, player, hero, path).unwrap_err();
    assert_eq!(*state, before);
    assert_eq!(state.state_hash(), before.state_hash());
    error
}

// covers: ADV-MOV-007, ADV-MAP-001
#[test]
fn a_move_is_rejected_whole_and_changes_nothing() {
    let rules = movement_rules(true);
    let mut state = world_state(
        &rules,
        &["gg#~gg", "gBVTgg", "gggggg"],
        vec![
            hero_at(1, 0, p(0, 0), &[PEASANT]),
            hero_at(2, 0, p(5, 2), &[PEASANT]),
            hero_at(3, 0, p(5, 0), &[PEASANT]),
        ],
    );
    use AdventureCommandError as E;
    // Another player's hero, or none, is never the player's.
    assert_eq!(
        rejected(&mut state, &rules, BLUE, 1, &[p(1, 0)]),
        E::NotYourHero(HeroId::new(1))
    );
    assert_eq!(
        rejected(&mut state, &rules, RED, 9, &[p(1, 0)]),
        E::NotYourHero(HeroId::new(9))
    );
    assert_eq!(rejected(&mut state, &rules, RED, 1, &[]), E::EmptyPath);
    assert_eq!(
        rejected(&mut state, &rules, RED, 1, &[p(1, 0), p(3, 0)]),
        E::NotNeighbour { step: 1 }
    );
    assert_eq!(
        rejected(&mut state, &rules, RED, 1, &[p(1, 0), p(1, 0)]),
        E::NotNeighbour { step: 1 }
    );
    let blocked = |step, error| E::Blocked { step, error };
    // Rock and sea, an object's blocked tile, off the map; past another hero's tile, visited.
    assert_eq!(
        rejected(&mut state, &rules, RED, 1, &[p(1, 0), p(2, 0)]),
        blocked(1, StandError::Terrain)
    );
    assert_eq!(
        rejected(&mut state, &rules, RED, 3, &[p(4, 0), p(3, 0)]),
        blocked(1, StandError::Terrain)
    );
    assert_eq!(
        rejected(&mut state, &rules, RED, 1, &[p(1, 1)]),
        blocked(0, StandError::Blocked)
    );
    assert_eq!(
        rejected(&mut state, &rules, RED, 3, &[p(5, 1), p(5, 2), p(4, 2)]),
        E::PastVisit { step: 2 }
    );
    assert_eq!(
        rejected(&mut state, &rules, RED, 3, &[p(6, 0)]),
        blocked(0, StandError::Outside)
    );
    // An object's visitable tile, and a town's gate (blocked and visitable).
    assert_eq!(
        rejected(&mut state, &rules, RED, 1, &[p(1, 0), p(2, 1)]),
        E::Visit { step: 1 }
    );
    assert_eq!(
        rejected(&mut state, &rules, RED, 3, &[p(4, 1), p(3, 1)]),
        E::Visit { step: 1 }
    );
    // A path longer than the day, refused at its first unpaid step though the steps before
    // it were fine.
    let long: Vec<Pos> = (0..14)
        .map(|i| if i % 2 == 0 { p(0, 1) } else { p(0, 0) })
        .collect();
    assert_eq!(
        rejected(&mut state, &rules, RED, 1, &long[1..]),
        E::NotNeighbour { step: 0 }
    );
    assert_eq!(
        rejected(&mut state, &rules, RED, 1, &long),
        E::TooFewPoints {
            step: 13,
            cost: 100,
            points: 0
        }
    );
}

// covers: ADV-MOV-007
#[test]
fn a_hero_leaves_its_own_tile_and_may_come_back_to_it() {
    let rules = movement_rules(false);
    // A hero on a town's gate leaves it; coming back would be a visit.
    let mut state = world_state(&rules, &["Tgg"], vec![hero_at(1, 0, p(0, 0), &[PEASANT])]);
    assert_eq!(
        move_hero(&mut state, &rules, RED, 1, &[p(1, 0), p(0, 0)]),
        Err(AdventureCommandError::Visit { step: 1 })
    );
    move_hero(&mut state, &rules, RED, 1, &[p(1, 0)]).unwrap();
    assert_eq!(hero(&state, 1).0, p(1, 0));
    // From plain ground it may step out and back: its own tile does not block it.
    move_hero(&mut state, &rules, RED, 1, &[p(2, 0), p(1, 0)]).unwrap();
    assert_eq!(hero(&state, 1), (p(1, 0), 1000));
}

#[test]
fn errors_display() {
    use AdventureCommandError as E;
    let shown = [
        (E::NotYourHero(HeroId::new(3)), "hero 3 is not the player's"),
        (E::EmptyPath, "the path has no step"),
        (
            E::NotNeighbour { step: 2 },
            "step 2 does not go to a neighbouring tile",
        ),
        (
            E::Unexplored { step: 5 },
            "step 5 goes to an unexplored tile",
        ),
        (
            E::Blocked {
                step: 1,
                error: StandError::Occupied,
            },
            "step 1: another hero stands there",
        ),
        (
            E::Visit { step: 0 },
            "step 0 goes to a tile no visit is built for",
        ),
        (
            E::FromAbove { step: 3 },
            "step 3 crosses the top of an object's visitable tile",
        ),
        (E::PastVisit { step: 2 }, "step 2 follows a visit"),
        (
            E::NoStop { step: 1 },
            "step 1 lands where the hero may not stop",
        ),
        (
            E::TooFewPoints {
                step: 4,
                cost: 141,
                points: 100,
            },
            "step 4 costs 141 with 100 points left",
        ),
    ];
    for (error, text) in shown {
        assert_eq!(error.to_string(), text);
    }
}

/// `rows` rows of 16 grass tiles.
fn grass(rows: usize) -> Vec<String> {
    vec!["g".repeat(16); rows]
}

/// A grid of 16 × `height` tiles on `levels` levels with the disks of radius 5 around `centres`.
fn disks(height: u16, levels: u8, centres: &[Pos]) -> BitGrid {
    let mut grid = BitGrid::new(16, height, levels);
    for &centre in centres {
        reveal(&mut grid, centre, 5);
    }
    grid
}

// covers: ADV-FOG-001, ADV-FOG-003
#[test]
fn each_hero_reveals_its_surroundings_to_its_owner_at_the_start() {
    let rules = movement_rules(false);
    let rows = grass(32);
    let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
    let (red_a, red_b, blue) = (p(2, 2), p(13, 2), Pos::new(8, 12, 1));
    let setup = world(
        &rows,
        2,
        vec![
            hero_at(1, 0, red_a, &[PEASANT]),
            hero_at(2, 0, red_b, &[PEASANT]),
            hero_at(3, 1, blue, &[PEASANT]),
        ],
    );
    let state = crate::testing::started(&rules, &setup).unwrap();
    assert_eq!(state.explored(RED), Some(&disks(16, 2, &[red_a, red_b])));
    assert_eq!(state.explored(BLUE), Some(&disks(16, 2, &[blue])));
    // Radius 5: (2, 7) is 5 below the hero, (2, 8) 6.
    assert!(state.is_explored(RED, p(2, 7)) && !state.is_explored(RED, p(2, 8)));
    assert!(!state.is_explored(RED, blue) && !state.is_explored(BLUE, red_a));
    // No other player, no tile off the map.
    assert_eq!(state.explored(PlayerId::new(2)), None);
    assert!(!state.is_explored(PlayerId::new(2), red_a));
    assert!(!state.is_explored(RED, p(16, 2)));
}

// covers: ADV-FOG-003
#[test]
fn each_step_reveals_the_tiles_around_the_tile_it_enters() {
    let rules = movement_rules(false);
    let rows = grass(16);
    let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
    let mut state = world_state(
        &rules,
        &rows,
        vec![
            hero_at(1, 0, p(0, 0), &[PEASANT]),
            hero_at(2, 1, p(15, 15), &[PEASANT]),
        ],
    );
    // (4, 4) would be unexplored (4² + 4² > 30); (3, 4) is not.
    let path = [p(1, 1), p(2, 2), p(3, 3), p(3, 4)];
    move_hero(&mut state, &rules, RED, 1, &path).unwrap();
    let mut centres = vec![p(0, 0)];
    centres.extend(path);
    assert_eq!(state.explored(RED), Some(&disks(16, 1, &centres)));
    // Another player's layer does not change.
    assert_eq!(state.explored(BLUE), Some(&disks(16, 1, &[p(15, 15)])));
}

// covers: ADV-MOV-008
#[test]
fn a_path_goes_over_the_tiles_explored_when_it_is_given() {
    let rules = movement_rules(false);
    let rows = ["gggggggggggggggg", "gggggg#ggggggggg"];
    let mut state = world_state(&rules, &rows, vec![hero_at(1, 0, p(0, 0), &[PEASANT])]);
    let before = state.clone();
    // (5, 0) is explored (5² ≤ 30), (6, 0) not (36 > 30): the first steps would reveal it, but a
    // path is checked against the tiles explored before the move.
    let path: Vec<Pos> = (1..=6).map(|x| p(x, 0)).collect();
    assert_eq!(
        move_hero(&mut state, &rules, RED, 1, &path),
        Err(AdventureCommandError::Unexplored { step: 5 })
    );
    // An unexplored tile is refused before anything about it is asked: the rock at (6, 1) is
    // not reported as rock. A tile off the map is refused as off the map.
    let path: Vec<Pos> = (1..=6).map(|x| p(x, 1)).collect();
    assert_eq!(
        move_hero(&mut state, &rules, RED, 1, &path),
        Err(AdventureCommandError::Unexplored { step: 5 })
    );
    assert!(matches!(
        move_hero(&mut state, &rules, RED, 1, &[p(0, 1), p(0, 2)]),
        Err(AdventureCommandError::Blocked {
            step: 1,
            error: StandError::Outside
        })
    ));
    assert_eq!(state, before);
    // In two moves: the first one's last step reveals (6, 0).
    move_hero(&mut state, &rules, RED, 1, &path_to(5)).unwrap();
    move_hero(&mut state, &rules, RED, 1, &[p(6, 0)]).unwrap();
    assert_eq!(hero(&state, 1), (p(6, 0), 1300 - 600));
}

/// The straight path along row 0 from (1, 0) to (`x`, 0).
fn path_to(x: u16) -> Vec<Pos> {
    (1..=x).map(|x| p(x, 0)).collect()
}

// covers: ADV-FOG-004
#[test]
fn a_step_is_seen_by_its_owner_and_by_the_players_that_explored_its_tiles() {
    let rules = movement_rules(false);
    // Blue's hero at (10, 0) explored (5, 0) to (15, 0).
    let mut state = world_state(
        &rules,
        &["gggggggggggggggg"],
        vec![
            hero_at(1, 0, p(0, 0), &[PEASANT]),
            hero_at(2, 1, p(10, 0), &[PEASANT]),
        ],
    );
    let events = move_hero(&mut state, &rules, RED, 1, &path_to(5)).unwrap();
    let seen: Vec<Viewers> = events.iter().map(|event| event.seen_by).collect();
    let red = Viewers::NONE.with(RED);
    let both = red.with(BLUE);
    // The steps onto (1, 0) to (4, 0) are on tiles blue has not explored; the step onto (5, 0)
    // enters one it has.
    assert_eq!(seen, [red, red, red, red, both]);
    // Leaving a tile blue has explored is seen too, onto one it has not: back from (5, 0) to
    // (4, 0). Then from (4, 0) to (3, 0), neither of which blue has explored: red only.
    let back = move_hero(&mut state, &rules, RED, 1, &[p(4, 0), p(3, 0)]).unwrap();
    let seen: Vec<Viewers> = back.iter().map(|event| event.seen_by).collect();
    assert_eq!(seen, [both, red]);
    assert!(state.is_explored(BLUE, p(5, 0)) && !state.is_explored(BLUE, p(4, 0)));
}

#[test]
fn viewers_hold_the_players_of_a_game() {
    let (red, blue, eighth) = (RED, BLUE, PlayerId::new(7));
    let viewers = Viewers::NONE.with(red).with(eighth);
    assert!(viewers.contains(red) && viewers.contains(eighth) && !viewers.contains(blue));
    assert_eq!(viewers.bits(), 0b1000_0001);
    assert_eq!(Viewers::default(), Viewers::NONE);
    // An id past the eighth player is no game's: it is left out.
    assert_eq!(viewers.with(PlayerId::new(8)), viewers);
    assert!(
        !Viewers::NONE
            .with(PlayerId::new(8))
            .contains(PlayerId::new(8))
    );
}

/// Red's hero at (0, 0) on a row of grass with `row`'s objects, blue's at (8, 0): blue has
/// explored the row from (3, 0), so (2, 0) is on its fringe.
fn visit_world(rules: &(Ruleset, AdventureRuleset), row: &str) -> AdventureState {
    world_state(
        rules,
        &[row, "gggggggggggggggg"],
        vec![
            hero_at(1, 0, p(0, 0), &[PEASANT]),
            hero_at(2, 1, p(8, 0), &[PEASANT]),
        ],
    )
}

// covers: ADV-OBJ-003, ADV-OBJ-005, ADV-PIK-001
#[test]
fn a_step_toward_a_pile_visits_it_from_the_tile_before() {
    for faithful in [true, false] {
        let rules = movement_rules(faithful);
        let mut state = visit_world(&rules, "ggPggggggggggggg");
        let contents = state.map().objects()[0].contents.clone();
        assert_eq!(contents.len(), 1);
        let mut events = Vec::new();
        let command = AdventureCommand::MoveHero {
            hero: HeroId::new(1),
            path: vec![p(1, 0), p(2, 0)],
        };
        let outcome = state
            .apply(&rules.0, &rules.1, &[], RED, &command, &mut events)
            .unwrap();
        // The owner gains the pile; the hero stays on (1, 0) and pays both steps.
        assert_eq!(outcome.gains, contents);
        assert_eq!(hero(&state, 1), (p(1, 0), 1300 - 200));
        assert!(state.is_removed(0));
        // One step, then the removal, seen by red and, on its fringe under ADV-QUIRK-1, blue.
        assert_eq!(events.len(), 2);
        let seen = if faithful {
            Viewers::NONE.with(RED).with(BLUE)
        } else {
            Viewers::NONE.with(RED)
        };
        assert_eq!(
            events[1],
            AdventureEvent {
                kind: AdventureEventKind::ObjectRemoved {
                    object_type: WOOD_PILE,
                    anchor: p(2, 0),
                },
                seen_by: seen,
            }
        );
        // Blue is told the removal alone, as the observation names the object.
        let told = events[1].project(BLUE);
        assert_eq!(told.is_some(), faithful);
        assert_eq!(
            told.map(|e| (e.kind, e.seen_by)),
            faithful.then_some((events[1].kind, Viewers::NONE.with(BLUE)))
        );
        // The pile's tile is free now: the hero walks over it.
        move_hero(&mut state, &rules, RED, 1, &[p(2, 0), p(3, 0)]).unwrap();
        assert_eq!(hero(&state, 1).0, p(3, 0));
    }
}

// covers: ADV-OBJ-003, ADV-PIK-003
#[test]
fn a_visit_from_the_hero_s_own_tile_moves_nothing_and_a_fire_gives_two_resources() {
    let rules = movement_rules(true);
    let mut state = visit_world(&rules, "gFgggggggggggggg");
    let contents = state.map().objects()[0].contents.clone();
    let mut events = Vec::new();
    let command = AdventureCommand::MoveHero {
        hero: HeroId::new(1),
        path: vec![p(1, 0)],
    };
    let outcome = state
        .apply(&rules.0, &rules.1, &[], RED, &command, &mut events)
        .unwrap();
    // The fire's wood, then its gold (ADV-PIK-003's order).
    assert_eq!(outcome.gains, contents);
    let [(_, wood), (_, gold)] = outcome.gains[..] else {
        panic!("two resources");
    };
    assert_eq!(gold, wood * 100);
    assert_eq!(hero(&state, 1), (p(0, 0), 1300 - 100));
    assert_eq!(
        events,
        [AdventureEvent {
            kind: AdventureEventKind::ObjectRemoved {
                object_type: FIRE,
                anchor: p(1, 0),
            },
            seen_by: Viewers::NONE.with(RED),
        }]
    );
    // The visit's step pays as a step does: points below the cheapest next step are lost
    // (ADV-MOV-006, as if the hero stood on the fire's tile).
    let mut state = visit_world(&rules, "gFgggggggggggggg");
    state.set_movement(HeroId::new(1), 150);
    move_hero(&mut state, &rules, RED, 1, &[p(1, 0)]).unwrap();
    assert_eq!(hero(&state, 1), (p(0, 0), 0));
}

// covers: ADV-OBJ-003
#[test]
fn a_visit_ends_the_move_and_a_blocker_s_is_not_built() {
    let rules = movement_rules(true);
    let rejected = |row: &str, path: &[Pos]| {
        let state = visit_world(&rules, row);
        let mut moved = state.clone();
        let error = move_hero(&mut moved, &rules, RED, 1, path).unwrap_err();
        assert_eq!(moved, state);
        error
    };
    assert_eq!(
        rejected("gPgggggggggggggg", &[p(1, 0), p(2, 0)]),
        AdventureCommandError::PastVisit { step: 1 }
    );
    // Two blockers' visitable tiles (every typed kind's visit is built since E08 T15b).
    for row in ["gVgggggggggggggg", "gTgggggggggggggg"] {
        assert_eq!(
            rejected(row, &[p(0, 1), p(1, 0)]),
            AdventureCommandError::Visit { step: 1 }
        );
    }
}

/// Red's hero at `at` on three rows of grass whose middle one holds `chart` at (2, 1), blue's at
/// (12, 2).
fn chart_world(rules: &(Ruleset, AdventureRuleset), chart: char, at: Pos) -> AdventureState {
    let middle = std::format!("gg{chart}ggggggggggggg");
    world_state(
        rules,
        &["gggggggggggggggg", &middle, "gggggggggggggggg"],
        vec![
            hero_at(1, 0, at, &[PEASANT]),
            hero_at(2, 1, p(12, 2), &[PEASANT]),
        ],
    )
}

/// `path` of red's hero 1 is rejected with `error` and changes nothing.
fn refused(
    state: &AdventureState,
    rules: &(Ruleset, AdventureRuleset),
    path: &[Pos],
    error: AdventureCommandError,
) {
    let mut moved = state.clone();
    assert_eq!(move_hero(&mut moved, rules, RED, 1, path), Err(error));
    assert_eq!(&moved, state);
}

// covers: ADV-OBJ-003, ADV-MOV-006
#[test]
fn a_step_onto_a_chart_enters_it_and_ends_the_move() {
    for faithful in [true, false] {
        let rules = movement_rules(faithful);
        let mut state = visit_world(&rules, "ggCggggggggggggg");
        let mut events = Vec::new();
        let command = AdventureCommand::MoveHero {
            hero: HeroId::new(1),
            path: vec![p(1, 0), p(2, 0)],
        };
        let outcome = state
            .apply(&rules.0, &rules.1, &[], RED, &command, &mut events)
            .unwrap();
        // The hero stands on the chart's tile, both steps paid as steps are; the visit gains and
        // removes nothing, and offers the chart's type (E08 T14b).
        assert!(outcome.gains.is_empty());
        assert!(outcome.offer.is_some());
        assert_eq!(hero(&state, 1), (p(2, 0), 1300 - 200));
        assert!(!state.is_removed(0));
        assert_eq!(events.len(), 2);
        assert!(matches!(
            events[1].kind,
            AdventureEventKind::HeroStepped { to: Some(to), .. } if to == p(2, 0)
        ));
        // The step onto the chart reveals around it (ADV-FOG-003).
        assert!(state.is_explored(RED, p(7, 0)));
        // A visit ends the move.
        refused(
            &visit_world(&rules, "ggCggggggggggggg"),
            &rules,
            &[p(1, 0), p(2, 0), p(3, 0)],
            AdventureCommandError::PastVisit { step: 2 },
        );
        // The hero leaves the chart sideways and enters it again; with points below the cheapest
        // next step, those are lost there as anywhere (ADV-MOV-006).
        move_hero(&mut state, &rules, RED, 1, &[p(3, 0)]).unwrap();
        state.set_movement(HeroId::new(1), 150);
        move_hero(&mut state, &rules, RED, 1, &[p(2, 0)]).unwrap();
        assert_eq!(hero(&state, 1), (p(2, 0), 0));
    }
}

/// Red's hero 1 at (1, 1) and blue's hero 2 at (14, 1) on two levels of three rows: a chart at
/// (2, 1) and an open chart at (12, 1) on the first level, grass, sand, sea and rock around.
fn reveal_world(rules: &(Ruleset, AdventureRuleset)) -> AdventureState {
    let rows = [
        "gggggggggggggsss",
        "ggCggggggggg~cgg",
        "gggggggggggg##gg",
        "ssssssssgggggggg",
        "gg~~~~~~~#####gs",
        "gggggggggggggggg",
    ];
    let heroes = vec![
        hero_at(1, 0, p(1, 1), &[PEASANT]),
        hero_at(2, 1, p(14, 1), &[PEASANT]),
    ];
    crate::testing::started(rules, &world(&rows, 2, heroes)).unwrap()
}

/// `player`'s hero `hero` steps onto `to` and back off it to `back`, then onto `to` again: the
/// outcome of the last step.
fn enter(
    state: &mut AdventureState,
    rules: &(Ruleset, AdventureRuleset),
    (player, hero): (PlayerId, u16),
    to: Pos,
) -> AdventureOutcome {
    state.set_movement(HeroId::new(hero), 1000);
    let command = AdventureCommand::MoveHero {
        hero: HeroId::new(hero),
        path: vec![to],
    };
    state
        .apply(&rules.0, &rules.1, &[], player, &command, &mut Vec::new())
        .unwrap()
}

// covers: ADV-VIS-001
#[test]
fn a_chart_offers_its_reveal_once_per_player_and_type() {
    let rules = movement_rules(false);
    let mut state = reveal_world(&rules);
    let gold = ResourceId::from_raw(1);
    // Entering the chart offers its type for its price.
    let before = state.clone();
    let offer = enter(&mut state, &rules, (RED, 1), p(2, 1)).offer.unwrap();
    assert_eq!(
        offer,
        Offer {
            hero: HeroId::new(1),
            object_type: CHART,
            anchor: p(2, 1),
            kind: OfferKind::Reveal {
                price: vec![(gold, 1000)],
            },
        }
    );
    assert_eq!(offer.price(), [(gold, 1000)]);
    assert!(!state.has_bought(RED, CHART));
    // Accepted: every grass tile of both levels is explored for red, told to red alone with the
    // number it had not explored; no other terrain, and nothing for blue.
    let map = state.map().clone();
    let grass = |pos: Pos| map.tile(pos).unwrap().terrain == crate::testing::GRASS;
    let explored = |state: &AdventureState, player| state.explored(player).unwrap().count();
    let (red, blue) = (explored(&state, RED), explored(&state, BLUE));
    let tiles: Vec<Pos> = (0..2u8)
        .flat_map(|level| {
            (0..3u16).flat_map(move |y| (0..16u16).map(move |x| Pos::new(x, y, level)))
        })
        .collect();
    let new = tiles
        .iter()
        .filter(|&&pos| grass(pos) && !state.is_explored(RED, pos))
        .count();
    let hidden: Vec<Pos> = tiles
        .iter()
        .copied()
        .filter(|&pos| !grass(pos) && !state.is_explored(RED, pos))
        .collect();
    assert!(new > 20 && !hidden.is_empty(), "{new} {hidden:?}");
    let mut events = Vec::new();
    state.accept_offer(&rules.0, &rules.1, RED, &offer, &mut events);
    assert_eq!(
        events,
        vec![AdventureEvent {
            kind: AdventureEventKind::TilesRevealed {
                object_type: CHART,
                count: u32::try_from(new).unwrap(),
            },
            seen_by: Viewers::NONE.with(RED),
        }]
    );
    assert_eq!(events[0].project(RED), Some(events[0]));
    assert_eq!(events[0].project(BLUE), None);
    assert_eq!(explored(&state, RED), red + u32::try_from(new).unwrap());
    assert!(
        tiles
            .iter()
            .all(|&pos| !grass(pos) || state.is_explored(RED, pos))
    );
    assert!(hidden.iter().all(|&pos| !state.is_explored(RED, pos)));
    assert_eq!(explored(&state, BLUE), blue);
    assert!(state.has_bought(RED, CHART) && !state.has_bought(BLUE, CHART));
    assert_eq!(state.bought(RED).collect::<Vec<_>>(), [CHART]);
    assert_eq!(state.bought(BLUE).count(), 0);
    // The viewer sees its own purchases only.
    assert_eq!(state.observe(&rules.0, &rules.1, RED).bought, [CHART]);
    assert!(state.observe(&rules.0, &rules.1, BLUE).bought.is_empty());
    assert!(
        state
            .observe(&rules.0, &rules.1, PlayerId::new(2))
            .bought
            .is_empty()
    );
    assert_ne!(state.state_hash(), before.state_hash());
    // Bought: a later visit offers nothing; another type still offers.
    enter(&mut state, &rules, (RED, 1), p(1, 1));
    assert_eq!(
        enter(&mut state, &rules, (RED, 1), p(2, 1)),
        AdventureOutcome::default()
    );
    let open = enter(&mut state, &rules, (BLUE, 2), p(13, 1))
        .offer
        .unwrap();
    assert_eq!((open.object_type, open.anchor), (OPEN_CHART, p(13, 1)));
    // Blue buys the open chart, then the chart; red's purchase stays its own.
    state.accept_offer(&rules.0, &rules.1, BLUE, &open, &mut Vec::new());
    let chart = Offer {
        hero: HeroId::new(2),
        ..offer.clone()
    };
    state.accept_offer(&rules.0, &rules.1, BLUE, &chart, &mut Vec::new());
    assert_eq!(state.bought(BLUE).collect::<Vec<_>>(), [CHART, OPEN_CHART]);
    assert_eq!(state.bought(RED).collect::<Vec<_>>(), [CHART]);
    assert!(!state.has_bought(RED, OPEN_CHART));
    // Equal purchases hash equal, whatever their order.
    let mut other = reveal_world(&rules);
    other.accept_offer(&rules.0, &rules.1, BLUE, &chart, &mut Vec::new());
    other.accept_offer(&rules.0, &rules.1, RED, &offer, &mut Vec::new());
    other.accept_offer(&rules.0, &rules.1, BLUE, &open, &mut Vec::new());
    let mut moved = reveal_world(&rules);
    moved.accept_offer(&rules.0, &rules.1, RED, &offer, &mut Vec::new());
    assert_eq!(other.bought(RED).collect::<Vec<_>>(), [CHART]);
    assert_ne!(other.state_hash(), moved.state_hash());
    // A kind other than a Cartographer reveals nothing by terrain.
    let pile = rules.1.object_type(WOOD_PILE).unwrap();
    assert_eq!(pile.kind.reveal_terrain(), None);
}

// covers: ADV-OBJ-004
#[test]
fn a_chart_is_neither_entered_from_nor_left_toward_the_tiles_above() {
    let rules = movement_rules(true);
    let from = AdventureCommandError::FromAbove { step: 0 };
    let next = AdventureCommandError::FromAbove { step: 1 };
    // From above: down, down-right and down-left onto the chart.
    let above = chart_world(&rules, 'C', p(2, 0));
    refused(&above, &rules, &[p(2, 1)], from);
    refused(&above, &rules, &[p(1, 0), p(2, 1)], next);
    refused(&above, &rules, &[p(3, 0), p(2, 1)], next);
    // From the side and from below it is entered.
    let mut state = above.clone();
    move_hero(&mut state, &rules, RED, 1, &[p(1, 0), p(1, 1), p(2, 1)]).unwrap();
    assert_eq!(hero(&state, 1).0, p(2, 1));
    let mut state = chart_world(&rules, 'C', p(1, 2));
    move_hero(&mut state, &rules, RED, 1, &[p(2, 2), p(2, 1)]).unwrap();
    // Standing on it, the hero leaves it toward none of the three tiles above.
    for up in [p(1, 0), p(2, 0), p(3, 0)] {
        refused(&state, &rules, &[up], from);
    }
    for side in [p(1, 1), p(3, 1), p(1, 2), p(2, 2), p(3, 2)] {
        move_hero(&mut state.clone(), &rules, RED, 1, &[side]).unwrap();
    }
    // A hero placed on it at the start too.
    let placed = chart_world(&rules, 'C', p(2, 1));
    refused(&placed, &rules, &[p(2, 0)], from);
    move_hero(&mut placed.clone(), &rules, RED, 1, &[p(2, 2)]).unwrap();
}

// covers: ADV-OBJ-004
#[test]
fn objects_visited_from_above_are_entered_and_left_across_their_top() {
    let rules = movement_rules(true);
    // An open chart from above, and up off it.
    let mut state = chart_world(&rules, 'c', p(2, 0));
    move_hero(&mut state, &rules, RED, 1, &[p(2, 1)]).unwrap();
    assert_eq!(hero(&state, 1).0, p(2, 1));
    move_hero(&mut state, &rules, RED, 1, &[p(3, 0)]).unwrap();
    // A pile from above, visited from the tile above.
    let mut state = chart_world(&rules, 'P', p(1, 0));
    let outcome = state
        .apply(
            &rules.0,
            &rules.1,
            &[],
            RED,
            &AdventureCommand::MoveHero {
                hero: HeroId::new(1),
                path: vec![p(2, 1)],
            },
            &mut Vec::new(),
        )
        .unwrap();
    assert_eq!(outcome.gains.len(), 1);
    assert_eq!(hero(&state, 1).0, p(1, 0));
}

/// The world of [`chart_world`] with `row` as its middle row, its first object at (2, 1), and
/// a second object of `object_type` over it, later in map order.
fn stacked(
    rules: &(Ruleset, AdventureRuleset),
    row: &str,
    object_type: Option<ObjectTypeId>,
    at: Pos,
) -> Result<AdventureState, SetupError> {
    let mut setup = world(
        &["gggggggggggggggg", row, "gggggggggggggggg"],
        1,
        vec![
            hero_at(1, 0, at, &[PEASANT]),
            hero_at(2, 1, p(12, 2), &[PEASANT]),
        ],
    );
    let mut object = setup.map.objects[0].clone();
    object.object_type = object_type;
    setup.map.objects.push(object);
    crate::testing::started(rules, &setup)
}

// covers: ADV-OBJ-003, ADV-OBJ-004
#[test]
fn a_tile_visited_from_two_objects_visits_the_last_still_on_the_map() {
    let rules = movement_rules(true);
    // A pile, then a chart: the step enters the chart; the pile stays.
    let mut state = stacked(&rules, "ggPggggggggggggg", Some(CHART), p(0, 1)).unwrap();
    move_hero(&mut state, &rules, RED, 1, &[p(1, 1), p(2, 1)]).unwrap();
    assert_eq!(hero(&state, 1).0, p(2, 1));
    assert!(!state.is_removed(0));
    // A chart, then a pile: the step visits the pile from (1, 1); once the pile is gone, the
    // chart is entered.
    let mut state = stacked(&rules, "ggCggggggggggggg", Some(WOOD_PILE), p(0, 1)).unwrap();
    move_hero(&mut state, &rules, RED, 1, &[p(1, 1), p(2, 1)]).unwrap();
    assert_eq!(hero(&state, 1).0, p(1, 1));
    assert!(state.is_removed(1));
    move_hero(&mut state, &rules, RED, 1, &[p(2, 1)]).unwrap();
    assert_eq!(hero(&state, 1).0, p(2, 1));
    // The last object decides the test from above too: a chart over an open chart is not
    // entered from above, an open chart over a chart is.
    let state = stacked(&rules, "ggcggggggggggggg", Some(CHART), p(2, 0)).unwrap();
    refused(
        &state,
        &rules,
        &[p(2, 1)],
        AdventureCommandError::FromAbove { step: 0 },
    );
    let mut state = stacked(&rules, "ggCggggggggggggg", Some(OPEN_CHART), p(2, 0)).unwrap();
    move_hero(&mut state, &rules, RED, 1, &[p(2, 1)]).unwrap();
    // A blocker over a pile: the tile is refused, as a blocker's is, and a hero may stand there.
    let state = stacked(&rules, "ggPggggggggggggg", None, p(0, 1)).unwrap();
    refused(
        &state,
        &rules,
        &[p(1, 1), p(2, 1)],
        AdventureCommandError::Visit { step: 1 },
    );
    assert!(stacked(&rules, "ggPggggggggggggg", None, p(2, 1)).is_ok());
    // And whether a hero may start there: on a pile under a chart, not on a chart under a pile.
    assert!(stacked(&rules, "ggPggggggggggggg", Some(CHART), p(2, 1)).is_ok());
    assert_eq!(
        stacked(&rules, "ggCggggggggggggg", Some(WOOD_PILE), p(2, 1)),
        Err(SetupError::HeroTile {
            hero: HeroId::new(1),
            error: StandError::Visited,
        })
    );
}

// covers: ADV-MAP-001, ADV-OBJ-003
#[test]
fn no_hero_starts_on_the_tile_of_an_object_visited_from_a_neighbour() {
    let rules = movement_rules(true);
    let start = |row: &str| {
        AdventureState::new(
            &rules.0,
            &rules.1,
            &world(&[row], 1, vec![hero_at(1, 0, p(1, 0), &[PEASANT])]),
            &mut draws(),
        )
    };
    // A pile, a fire, a guard, a gate (visited from a neighbour; a gate's key's holders cross
    // it, E08 T15b, but no player holds a key at the start).
    for row in ["gPg", "gFg", "gKg", "gXg"] {
        assert_eq!(
            start(row).map(|_| ()),
            Err(SetupError::HeroTile {
                hero: HeroId::new(1),
                error: StandError::Visited,
            }),
            "{row}"
        );
    }
    // Entered objects (built or not) and a blocker's visitable tiles, like a town's gate.
    for row in ["gCg", "gcg", "gOg", "gVg", "gTg"] {
        assert!(start(row).is_ok(), "{row}");
    }
    // Once the pile is gone, its tile is one to stand on.
    let mut state = visit_world(&rules, "gPgggggggggggggg");
    assert_eq!(
        state.check_stand(&rules.1, p(1, 0)),
        Err(StandError::Visited)
    );
    move_hero(&mut state, &rules, RED, 1, &[p(1, 0)]).unwrap();
    assert_eq!(state.check_stand(&rules.1, p(1, 0)), Ok(()));
}

/// The adventure events of `state`'s red hero 1 moving along `path` with 1 000 points, after its
/// steps; its outcome.
fn visit_by(
    state: &mut AdventureState,
    rules: &(Ruleset, AdventureRuleset),
    (player, hero): (PlayerId, u16),
    path: &[Pos],
) -> (Vec<AdventureEvent>, AdventureOutcome) {
    state.set_movement(HeroId::new(hero), 1000);
    let mut events = Vec::new();
    let command = AdventureCommand::MoveHero {
        hero: HeroId::new(hero),
        path: path.to_vec(),
    };
    let outcome = state
        .apply(&rules.0, &rules.1, &[], player, &command, &mut events)
        .unwrap();
    events.retain(|e| !matches!(e.kind, AdventureEventKind::HeroStepped { .. }));
    (events, outcome)
}

// covers: ADV-VIS-002, ADV-OBJ-003
#[test]
fn an_observatory_reveals_the_disk_around_it_at_every_visit() {
    let rules = movement_rules(false);
    // Two levels of 16 × 16; a tower (radius 8) at (8, 8) on the first; red's hero at (4, 8),
    // blue's at (15, 15) on the second.
    let mut rows = grass(32);
    rows[8].replace_range(8..9, "O");
    let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
    let heroes = vec![
        hero_at(1, 0, p(4, 8), &[PEASANT]),
        hero_at(2, 1, Pos::new(15, 15, 1), &[PEASANT]),
    ];
    let mut state = crate::testing::started(&rules, &world(&rows, 2, heroes)).unwrap();
    let blue = state.explored(BLUE).cloned();
    let path: Vec<Pos> = (5..=8).map(|x| p(x, 8)).collect();
    let mut events = move_hero(&mut state, &rules, RED, 1, &path).unwrap();
    // The hero enters the tower: its steps, then the reveal, told to red alone with the number
    // of tiles the steps had not explored; the radius on the tower's level only.
    let mut centres = vec![p(4, 8)];
    centres.extend(&path);
    let stepped = disks(16, 2, &centres);
    let mut expected = stepped.clone();
    reveal(&mut expected, p(8, 8), 8);
    let reveal_event = events.pop().unwrap();
    assert_eq!(events.len(), path.len());
    assert_eq!(
        reveal_event,
        AdventureEvent {
            kind: AdventureEventKind::TilesRevealed {
                object_type: TOWER,
                count: expected.count() - stepped.count(),
            },
            seen_by: Viewers::NONE.with(RED),
        }
    );
    assert!(expected.count() - stepped.count() > 50);
    assert_eq!(reveal_event.project(BLUE), None);
    assert_eq!(state.explored(RED), Some(&expected));
    assert!(state.is_explored(RED, p(8, 0)) && !state.is_explored(RED, p(0, 0)));
    assert!(!state.is_explored(RED, Pos::new(8, 8, 1)));
    assert_eq!(state.explored(BLUE).cloned(), blue);
    assert_eq!(hero(&state, 1).0, p(8, 8));
    // Every visit reveals again: nothing is left to explore, and red is told so.
    let mut again = state.clone();
    move_hero(&mut again, &rules, RED, 1, &[p(9, 8)]).unwrap();
    let (events, outcome) = visit_by(&mut again, &rules, (RED, 1), &[p(8, 8)]);
    assert_eq!(outcome, AdventureOutcome::default());
    assert_eq!(
        events,
        [AdventureEvent {
            kind: AdventureEventKind::TilesRevealed {
                object_type: TOWER,
                count: 0,
            },
            seen_by: Viewers::NONE.with(RED),
        }]
    );
    assert_eq!(again.explored(RED), Some(&expected));
    // A visit ends the move.
    refused(
        &state,
        &rules,
        &[p(9, 8), p(8, 8), p(7, 8)],
        AdventureCommandError::PastVisit { step: 2 },
    );
}

// covers: ADV-VIS-003
#[test]
fn a_tent_gives_its_key_to_its_visitor_s_owner_once() {
    let rules = movement_rules(true);
    let mut state = visit_world(&rules, "ggYggggggggggggg");
    let before = state.clone();
    assert_eq!((state.keys(RED), state.has_key(RED, 0)), (0, false));
    let (events, outcome) = visit_by(&mut state, &rules, (RED, 1), &[p(1, 0), p(2, 0)]);
    // Red's hero enters the tent: red holds key 0, told to red alone.
    let gained = AdventureEvent {
        kind: AdventureEventKind::KeyGained { key: 0 },
        seen_by: Viewers::NONE.with(RED),
    };
    assert_eq!(
        (events, outcome),
        (vec![gained], AdventureOutcome::default())
    );
    assert_eq!(gained.project(RED), Some(gained));
    assert_eq!(gained.project(BLUE), None);
    assert_eq!(hero(&state, 1).0, p(2, 0));
    assert!(state.has_key(RED, 0) && !state.has_key(BLUE, 0));
    assert!(!state.has_key(RED, 1) && !state.has_key(RED, 8));
    assert_eq!((state.keys(RED), state.keys(BLUE)), (1, 0));
    assert!(!state.is_removed(0));
    assert_ne!(state.state_hash(), before.state_hash());
    // The viewer sees its own keys only.
    assert_eq!(state.observe(&rules.0, &rules.1, RED).keys, 1);
    assert_eq!(state.observe(&rules.0, &rules.1, BLUE).keys, 0);
    assert_eq!(state.observe(&rules.0, &rules.1, PlayerId::new(2)).keys, 0);
    // Held: a later visit gives nothing and tells nothing.
    move_hero(&mut state, &rules, RED, 1, &[p(3, 0)]).unwrap();
    let held = state.clone();
    let (events, _) = visit_by(&mut state, &rules, (RED, 1), &[p(2, 0)]);
    assert!(events.is_empty());
    assert_eq!(state.keys(RED), held.keys(RED));
    // Blue's hero enters it too: blue's own key, red's unchanged.
    move_hero(&mut state, &rules, RED, 1, &[p(3, 1)]).unwrap();
    let (events, _) = visit_by(&mut state, &rules, (BLUE, 2), &[p(7, 0), p(6, 0), p(5, 0)]);
    assert!(events.is_empty());
    let (events, _) = visit_by(&mut state, &rules, (BLUE, 2), &[p(4, 0), p(3, 0), p(2, 0)]);
    assert_eq!(
        events,
        [AdventureEvent {
            kind: AdventureEventKind::KeyGained { key: 0 },
            seen_by: Viewers::NONE.with(BLUE),
        }]
    );
    assert_eq!((state.keys(RED), state.keys(BLUE)), (1, 1));
    // Equal keys hash equal, whatever the order they came in.
    let go = |state: &mut AdventureState, (player, hero): (PlayerId, u16), path: &[Pos]| {
        visit_by(state, &rules, (player, hero), path);
    };
    let (red_in, red_out) = ([p(1, 0), p(2, 0)], [p(3, 1)]);
    let blue_in = [p(7, 0), p(6, 0), p(5, 0), p(4, 0), p(3, 0), p(2, 0)];
    let blue_out = [p(1, 1)];
    let mut red_first = visit_world(&rules, "ggYggggggggggggg");
    red_first.explore_all();
    let mut blue_first = red_first.clone();
    go(&mut red_first, (RED, 1), &red_in);
    go(&mut red_first, (RED, 1), &red_out);
    go(&mut red_first, (BLUE, 2), &blue_in);
    go(&mut red_first, (BLUE, 2), &blue_out);
    go(&mut blue_first, (BLUE, 2), &blue_in);
    go(&mut blue_first, (BLUE, 2), &blue_out);
    go(&mut blue_first, (RED, 1), &red_in);
    go(&mut blue_first, (RED, 1), &red_out);
    for state in [&mut red_first, &mut blue_first] {
        state.set_movement(HeroId::new(1), 1000);
        state.set_movement(HeroId::new(2), 1000);
    }
    assert_eq!((red_first.keys(RED), red_first.keys(BLUE)), (1, 1));
    assert_eq!(red_first, blue_first);
    assert_eq!(red_first.state_hash(), blue_first.state_hash());
}

/// Red's hero 1 at (0, 0) and blue's 2 at (12, 1) on two rows of grass: a guard of key 0 at
/// (3, 0) and a tent of key 0 at (7, 0).
fn guard_world(rules: &(Ruleset, AdventureRuleset)) -> AdventureState {
    world_state(
        rules,
        &["gggKgggYgggggggg", "gggggggggggggggg"],
        vec![
            hero_at(1, 0, p(0, 0), &[PEASANT]),
            hero_at(2, 1, p(12, 1), &[PEASANT]),
        ],
    )
}

// covers: ADV-VIS-004, ADV-OBJ-003, ADV-OBJ-005
#[test]
fn a_border_guard_offers_its_key_s_holders_to_pass() {
    let rules = movement_rules(false);
    let mut state = guard_world(&rules);
    // Without the key: the hero visits the guard from (2, 0), pays the step and stays; nothing
    // else happens, and nothing is told.
    let (events, outcome) = visit_by(&mut state, &rules, (RED, 1), &[p(1, 0), p(2, 0), p(3, 0)]);
    assert_eq!((events, outcome), (vec![], AdventureOutcome::default()));
    assert_eq!(hero(&state, 1), (p(2, 0), 1000 - 300));
    refused(
        &guard_world(&rules),
        &rules,
        &[p(1, 0), p(2, 0), p(3, 0), p(4, 0)],
        AdventureCommandError::PastVisit { step: 3 },
    );
    // With the key: an offer to pass, without a price.
    let around = [p(3, 1), p(4, 1), p(5, 1), p(6, 1), p(7, 0)];
    visit_by(&mut state, &rules, (RED, 1), &around);
    assert!(state.has_key(RED, 0));
    let back = [p(6, 1), p(5, 1), p(4, 1), p(3, 0)];
    let (events, outcome) = visit_by(&mut state, &rules, (RED, 1), &back);
    let offer = Offer {
        hero: HeroId::new(1),
        object_type: GUARD,
        anchor: p(3, 0),
        kind: OfferKind::Pass,
    };
    assert!(events.is_empty() && outcome.gains.is_empty());
    assert_eq!(outcome.offer.as_ref(), Some(&offer));
    assert_eq!(offer.price(), []);
    assert_eq!(hero(&state, 1).0, p(4, 1));
    assert!(state.offer_stands(&offer));
    // Accepted: the guard leaves the map for everyone, told to the players that see it (red
    // alone: blue's hero, nine columns off, has not explored its tile), and its tile is free.
    let mut events = Vec::new();
    state.accept_offer(&rules.0, &rules.1, RED, &offer, &mut events);
    assert_eq!(
        events,
        [AdventureEvent {
            kind: AdventureEventKind::ObjectRemoved {
                object_type: GUARD,
                anchor: p(3, 0),
            },
            seen_by: Viewers::NONE.with(RED),
        }]
    );
    assert!(state.is_removed(0) && !state.offer_stands(&offer));
    assert!(
        state
            .observe(&rules.0, &rules.1, RED)
            .objects
            .iter()
            .all(|o| o.object_type != GUARD)
    );
    // Another player passes too: blue walks over the guard's tile.
    state.explore_all();
    let blue = [
        p(11, 1),
        p(10, 1),
        p(9, 1),
        p(8, 1),
        p(7, 1),
        p(6, 1),
        p(5, 1),
        p(4, 0),
        p(3, 0),
    ];
    let mut events = Vec::new();
    state.set_movement(HeroId::new(2), 1300);
    let command = AdventureCommand::MoveHero {
        hero: HeroId::new(2),
        path: blue.to_vec(),
    };
    state
        .apply(&rules.0, &rules.1, &[], BLUE, &command, &mut events)
        .unwrap();
    assert_eq!(hero(&state, 2).0, p(3, 0));
    // An offer that no longer stands carries nothing out (the game rejects its acceptance
    // first, ADR-0017 point 10).
    let gone = state.clone();
    let mut events = Vec::new();
    state.accept_offer(&rules.0, &rules.1, RED, &offer, &mut events);
    assert!(events.is_empty());
    assert_eq!(state, gone);
    // Declining needs nothing of the context: the guard stays until a key holder accepts.
    let mut other = guard_world(&rules);
    other.explore_all();
    let tent = [
        p(1, 1),
        p(2, 1),
        p(3, 1),
        p(4, 1),
        p(5, 1),
        p(6, 1),
        p(7, 0),
    ];
    visit_by(&mut other, &rules, (RED, 1), &tent);
    let (_, outcome) = visit_by(&mut other, &rules, (RED, 1), &back);
    assert!(outcome.offer.is_some() && !other.is_removed(0));
}

/// Red's hero 1 at (0, 1) and blue's 2 at (12, 2) on three rows of grass: a gate of key 0 at
/// (3, 1) and a tent of key 0 at (7, 1).
fn gate_world(rules: &(Ruleset, AdventureRuleset)) -> AdventureState {
    world_state(
        rules,
        &["gggggggggggggggg", "gggXgggYgggggggg", "gggggggggggggggg"],
        vec![
            hero_at(1, 0, p(0, 1), &[PEASANT]),
            hero_at(2, 1, p(12, 2), &[PEASANT]),
        ],
    )
}

// covers: ADV-VIS-005, ADV-OBJ-003, ADV-MOV-006
#[test]
fn a_border_gate_lets_its_key_s_holders_cross_it() {
    let rules = movement_rules(false);
    let mut state = gate_world(&rules);
    // Without the key: the hero visits the gate from (2, 1), pays the step and stays; nothing
    // else happens, and nothing is told.
    let (events, outcome) = visit_by(&mut state, &rules, (RED, 1), &[p(1, 1), p(2, 1), p(3, 1)]);
    assert_eq!((events, outcome), (vec![], AdventureOutcome::default()));
    assert_eq!(hero(&state, 1), (p(2, 1), 1000 - 300));
    refused(
        &gate_world(&rules),
        &rules,
        &[p(1, 1), p(2, 1), p(3, 1), p(4, 1)],
        AdventureCommandError::PastVisit { step: 3 },
    );
    // The visit's step pays as a step does: points below the cheapest next step are lost.
    let mut poor = state.clone();
    poor.set_movement(HeroId::new(1), 150);
    move_hero(&mut poor, &rules, RED, 1, &[p(3, 1)]).unwrap();
    assert_eq!(hero(&poor, 1), (p(2, 1), 0));
    // With the key (red's hero takes it at the tent), the gate's tile is crossed as any tile:
    // a step event per tile, nothing else, and the move goes on.
    visit_by(
        &mut state,
        &rules,
        (RED, 1),
        &[p(3, 2), p(4, 2), p(5, 2), p(6, 2), p(7, 1)],
    );
    assert!(state.has_key(RED, 0));
    let back = [p(6, 1), p(5, 1), p(4, 1), p(3, 1), p(2, 1), p(1, 1)];
    state.set_movement(HeroId::new(1), 1000);
    let events = move_hero(&mut state, &rules, RED, 1, &back).unwrap();
    assert_eq!(events.len(), back.len());
    assert!(
        events
            .iter()
            .all(|e| matches!(e.kind, AdventureEventKind::HeroStepped { .. }))
    );
    assert_eq!(hero(&state, 1), (p(1, 1), 1000 - 600));
    // A move may end on it, and the hero leaves it as any tile, upward too (a gate is visited
    // from above).
    let (events, outcome) = visit_by(&mut state, &rules, (RED, 1), &[p(2, 1), p(3, 1)]);
    assert_eq!((events, outcome), (vec![], AdventureOutcome::default()));
    assert_eq!(hero(&state, 1).0, p(3, 1));
    let mut up = state.clone();
    move_hero(&mut up, &rules, RED, 1, &[p(4, 0)]).unwrap();
    assert_eq!(hero(&up, 1).0, p(4, 0));
    // Blue holds no key: its step onto the gate is a visit of red's hero while it stands there,
    // a battle (E08 T19a), then a visit from (4, 2) that does nothing.
    state.explore_all();
    let walk: Vec<Pos> = (4..12).rev().map(|x| p(x, 2)).collect();
    visit_by(&mut state, &rules, (BLUE, 2), &walk);
    let before = state.clone();
    let (events, outcome) = visit_by(&mut state, &rules, (BLUE, 2), &[p(3, 1)]);
    assert!(events.is_empty());
    let battle = outcome.battle.unwrap();
    assert_eq!(
        (battle.attacker.hero, defender_hero(&battle), battle.tile),
        (HeroId::new(2), HeroId::new(1), p(3, 1))
    );
    assert_eq!(hero(&state, 2).0, p(4, 2));
    state = before;
    move_hero(&mut state, &rules, RED, 1, &[p(3, 0)]).unwrap();
    let (events, outcome) = visit_by(&mut state, &rules, (BLUE, 2), &[p(3, 1)]);
    assert_eq!((events, outcome), (vec![], AdventureOutcome::default()));
    assert_eq!(hero(&state, 2), (p(4, 2), 1000 - 141));
    // The gate never leaves the map.
    assert!(!state.is_removed(0));
}

/// The world of the hero visits (E08 T19a): red's hero 1 at (0, 0) and hero 3 at (1, 2); blue's
/// hero 2 at (3, 0), a sand tile on the coast, and hero 4 at (1, 1), on a Cartographer's tile (not
/// visited from above).
fn heroes_world(rules: &(Ruleset, AdventureRuleset)) -> AdventureState {
    let mut setup = world(
        &["gggs", "gCgg", "gggg"],
        1,
        vec![
            hero_at(1, 0, p(0, 0), &[PEASANT, GNOLL]),
            hero_at(2, 1, p(3, 0), &[WOLF]),
            hero_at(3, 0, p(1, 2), &[PEASANT]),
            hero_at(4, 1, p(1, 1), &[PEASANT]),
        ],
    );
    setup.map.tiles[3].coast = true;
    setup.heroes[1].formation = Formation::Tight;
    crate::testing::started(rules, &setup).unwrap()
}

// covers: GAM-BAT-004, GAM-BAT-001
#[test]
fn a_step_toward_another_players_hero_visits_it_and_starts_a_battle() {
    let rules = movement_rules(false);
    let mut state = heroes_world(&rules);
    let mut events = Vec::new();
    let command = AdventureCommand::MoveHero {
        hero: HeroId::new(1),
        path: vec![p(1, 0), p(2, 0), p(3, 0)],
    };
    // The defender's tactics flag off goes with its side (GAM-BAT-009, E08 T51a).
    state.set_tactics(BLUE, HeroId::new(2), false).unwrap();
    let outcome = state
        .apply(&rules.0, &rules.1, &[], RED, &command, &mut events)
        .unwrap();
    // Two steps, then the visit from (2, 0) at the step's price: 1300 - 3 × 100.
    assert_eq!(events.len(), 2);
    assert_eq!(hero(&state, 1), (p(2, 0), 1000));
    let side = |id: u16, owner, formation, tactics| {
        let hero = state.hero(HeroId::new(id)).unwrap();
        Combatant {
            hero: hero.id,
            owner,
            army: hero.army.clone(),
            formation,
            tactics,
        }
    };
    assert_eq!(
        outcome,
        AdventureOutcome {
            battle: Some(BattleStart {
                attacker: side(1, RED, Formation::Loose, true),
                defender: Defender::Hero(side(2, BLUE, Formation::Tight, false)),
                tile: p(3, 0),
                terrain: crate::testing::SAND,
                battlefield: Some(ContentKey::parse("t:shore").unwrap()),
                background: 2,
                attacker_ground: None,
            }),
            ..AdventureOutcome::default()
        }
    );
    // The query lists the hero's tile with the points the visit leaves, its own hero's too (a
    // meeting, E08 T50), whatever object lies under them.
    let reach = state
        .reach(&rules.0, &rules.1, &[], HeroId::new(3))
        .unwrap();
    assert_eq!(reach.points_left(p(1, 1)), Some(1200));
    assert_eq!(reach.path(p(1, 1)), Some(vec![p(1, 1)]));
    let reach = state
        .reach(&rules.0, &rules.1, &[], HeroId::new(1))
        .unwrap();
    assert_eq!(reach.points_left(p(3, 0)), Some(900));
    assert_eq!(reach.points_left(p(1, 2)), Some(1000 - 100 - 141));
}

// covers: GAM-BAT-004, GAM-MET-001, ADV-OBJ-003, ADV-OBJ-004
#[test]
fn a_hero_visit_ends_the_move_and_an_own_hero_is_met() {
    let rules = movement_rules(false);
    let mut state = heroes_world(&rules);
    use AdventureCommandError as E;
    assert_eq!(
        rejected(
            &mut state,
            &rules,
            RED,
            1,
            &[p(1, 0), p(2, 0), p(3, 0), p(3, 1)]
        ),
        E::PastVisit { step: 3 }
    );
    // A step toward an own hero meets it from the tile before at the step's price (E08 T50);
    // nothing goes on past it.
    assert_eq!(
        rejected(&mut state, &rules, RED, 1, &[p(0, 1), p(1, 2), p(2, 2)]),
        E::PastVisit { step: 2 }
    );
    let mut met = state.clone();
    let mut events = Vec::new();
    let path = vec![p(0, 1), p(1, 2)];
    let command = AdventureCommand::MoveHero {
        hero: HeroId::new(1),
        path,
    };
    let outcome = met
        .apply(&rules.0, &rules.1, &[], RED, &command, &mut events)
        .unwrap();
    let meeting = Meeting {
        visitor: HeroId::new(1),
        visited: HeroId::new(3),
    };
    assert_eq!(
        outcome,
        AdventureOutcome {
            meeting: Some(meeting),
            ..AdventureOutcome::default()
        }
    );
    assert_eq!(events.len(), 1);
    let points = state.hero(HeroId::new(1)).unwrap().movement;
    // A straight step and a diagonal one (141), the meeting's paid as if taken.
    assert_eq!(hero(&met, 1), (p(0, 1), points - 100 - 141));
    assert_eq!(met.hero(HeroId::new(3)), state.hero(HeroId::new(3)));
    // Blue's hero 4 stands on the Cartographer's tile, not visited from above: no step down onto
    // it from the three tiles above, the visit a hero's or not (ADV-OBJ-004; batch E4's E4-10:
    // the original finds no path and starts no battle, E08 T33). Straight down, down-right,
    // down-left.
    for path in [
        &[p(1, 0), p(1, 1)][..],
        &[p(1, 1)],
        &[p(1, 0), p(2, 0), p(1, 1)],
    ] {
        assert_eq!(
            rejected(&mut state, &rules, RED, 1, path),
            E::FromAbove {
                step: path.len() - 1
            }
        );
    }
    // The query's path to it goes round: from (0, 1), 1300 - 100 - 100.
    let reach = state
        .reach(&rules.0, &rules.1, &[], HeroId::new(1))
        .unwrap();
    assert_eq!(reach.points_left(p(1, 1)), Some(1100));
    assert_eq!(reach.path(p(1, 1)), Some(vec![p(0, 1), p(1, 1)]));
    // Red's hero 1 above it: hero 4 leaves its tile toward none of the three tiles above, a
    // hero's visit included (ADV-OBJ-004).
    let mut events = Vec::new();
    let step = AdventureCommand::MoveHero {
        hero: HeroId::new(1),
        path: vec![p(1, 0)],
    };
    state
        .apply(&rules.0, &rules.1, &[], RED, &step, &mut events)
        .unwrap();
    assert_eq!(hero(&state, 1), (p(1, 0), 1200));
    assert_eq!(
        rejected(&mut state, &rules, BLUE, 4, &[p(1, 0)]),
        E::FromAbove { step: 0 }
    );
    // The query's path to red's hero goes round: from (0, 1), 1300 - 100 - 141.
    let reach = state
        .reach(&rules.0, &rules.1, &[], HeroId::new(4))
        .unwrap();
    assert_eq!(reach.points_left(p(1, 0)), Some(1059));
    assert_eq!(reach.path(p(1, 0)), Some(vec![p(0, 1), p(1, 0)]));
    // From the side the visit is the hero's: a battle on the chart's tile.
    let side = AdventureCommand::MoveHero {
        hero: HeroId::new(1),
        path: vec![p(0, 1), p(1, 1)],
    };
    let outcome = state
        .apply(&rules.0, &rules.1, &[], RED, &side, &mut events)
        .unwrap();
    let battle = outcome.battle.unwrap();
    assert_eq!(
        (
            defender_hero(&battle),
            battle.tile,
            battle.terrain,
            battle.battlefield
        ),
        (HeroId::new(4), p(1, 1), crate::testing::GRASS, None)
    );
}

// covers: GAM-BAT-001
#[test]
fn a_battle_is_fought_on_the_ground_its_tile_gives() {
    let rules = movement_rules(false);
    let mut setup = world(
        &["gs", "gg", "gg", "gg"],
        2,
        vec![hero_at(1, 0, p(0, 0), &[PEASANT])],
    );
    setup.map.tiles[1].coast = true;
    setup.map.tiles[5].coast = true;
    let state = crate::testing::started(&rules, &setup).unwrap();
    let shore = Some(ContentKey::parse("t:shore").unwrap());
    use crate::testing::{SAND, SWAMP};
    // The tile's terrain, the coast's on the coast, the underground's below (coast or not).
    assert_eq!(
        state.battle_ground(&rules.1, p(0, 0)),
        (crate::testing::GRASS, None)
    );
    assert_eq!(state.battle_ground(&rules.1, p(1, 0)), (SAND, shore));
    for x in 0..2 {
        assert_eq!(
            state.battle_ground(&rules.1, Pos::new(x, 0, 1)),
            (SWAMP, None)
        );
    }
    // Without the part's battle terrains, always the tile's.
    let plain = crate::testing::rules();
    let imp = CreatureTypeId::from_raw(0);
    let mut setup = world(&["gg"], 1, vec![hero_at(1, 0, p(0, 0), &[imp])]);
    setup.map.tiles[1].coast = true;
    let state = AdventureState::new(&plain.0, &plain.1, &setup, &mut draws()).unwrap();
    assert_eq!(
        state.battle_ground(&plain.1, p(1, 0)),
        (crate::testing::GRASS, None)
    );
}

// covers: GAM-BAT-006, ADV-ARM-007
#[test]
fn raised_creatures_join_their_stack_a_free_slot_or_the_upgrades_stack() {
    // The peasant upgrades to the pikeman; armies have seven slots.
    let rules = crate::testing::movement_rules_edited(true, 16, |p| {
        p.creatures[2].info.upgrades = vec![ContentKey::parse("t:pikeman").unwrap()];
    });
    let mut state = heroes_world(&rules);
    let red = HeroId::new(1);
    let army = |state: &AdventureState| state.hero(red).unwrap().army.clone();
    let raise = |state: &mut AdventureState, creature, count| {
        state.raise(&rules.0, &rules.1, red, (creature, count))
    };
    // Its stack of the creature (slot 0: one peasant) grows.
    assert_eq!(raise(&mut state, PEASANT, 5), Some((PEASANT, 5)));
    assert_eq!(army(&state)[0].count, 6);
    // A new creature takes the first free slot, in slot order.
    let mut gap = state.clone();
    gap.end_battle(
        &rules.0,
        &[(
            red,
            Some(
                &[
                    ArmyStack {
                        slot: 0,
                        creature: GNOLL,
                        count: 2,
                    },
                    ArmyStack {
                        slot: 3,
                        creature: GNOLL,
                        count: 1,
                    },
                ][..],
            ),
        )],
        &mut Vec::new(),
    );
    assert_eq!(raise(&mut gap, WOLF, 4), Some((WOLF, 4)));
    let slots: Vec<(u8, CreatureTypeId)> =
        army(&gap).iter().map(|s| (s.slot, s.creature)).collect();
    assert_eq!(slots, [(0, GNOLL), (1, WOLF), (3, GNOLL)]);
    // The first stack of the creature that takes them within a stack's most does (slot 3's 1
    // here, not slot 0's 2); when none can, the first free slot (E08 decision 115).
    for _ in 0..2 {
        assert_eq!(
            raise(&mut gap, GNOLL, MAX_STACK - 1),
            Some((GNOLL, MAX_STACK - 1))
        );
    }
    let stacks: Vec<(u8, CreatureTypeId, u32)> = army(&gap)
        .iter()
        .map(|s| (s.slot, s.creature, s.count))
        .collect();
    assert_eq!(
        stacks,
        [
            (0, GNOLL, 2),
            (1, WOLF, 4),
            (2, GNOLL, MAX_STACK - 1),
            (3, GNOLL, MAX_STACK)
        ]
    );
    // A full army without the creature: two thirds of it, rounded up, as its upgrade, joining
    // the upgrade's stack; without an upgrade's stack or an upgrade, none.
    let full = |first: CreatureTypeId| {
        let mut state = state.clone();
        let mut stacks = crate::testing::army(&[first; 7]);
        stacks.iter_mut().skip(1).for_each(|s| s.creature = GNOLL);
        state.end_battle(&rules.0, &[(red, Some(&stacks[..]))], &mut Vec::new());
        state
    };
    let mut state = full(PIKEMAN);
    // At most a stack's most are raised (ADV-ARM-007): u32::MAX as MAX_STACK, 2 730 upgrades.
    for (count, joined) in [(1, 1), (2, 2), (3, 2), (4, 3), (u32::MAX, 2_730)] {
        let before = army(&state)[0].count;
        assert_eq!(raise(&mut state, PEASANT, count), Some((PIKEMAN, joined)));
        assert_eq!(army(&state)[0].count, before + joined);
    }
    // An upgrade's stack that cannot take them within the most takes none.
    let before = state.clone();
    assert_eq!(raise(&mut state, PEASANT, MAX_STACK), None);
    assert_eq!(state, before);
    let mut state = full(GNOLL);
    let before = state.clone();
    assert_eq!(raise(&mut state, PEASANT, 3), None);
    assert_eq!(raise(&mut state, WOLF, 3), None);
    assert_eq!(state, before);
    // Nothing for 0 creatures or a hero not on the map.
    assert_eq!(raise(&mut state, GNOLL, 0), None);
    assert_eq!(
        state.raise(&rules.0, &rules.1, HeroId::new(9), (GNOLL, 1)),
        None
    );
    assert_eq!(state, before);
}

// covers: GAM-BAT-002, GAM-BAT-003
#[test]
fn a_battles_end_sets_the_armies_and_takes_heroes_off_the_map() {
    let rules = movement_rules(false);
    let mut state = heroes_world(&rules);
    let before = state.clone();
    let survivors = crate::testing::army(&[GNOLL]);
    let mut events = Vec::new();
    state.end_battle(
        &rules.0,
        &[
            (HeroId::new(9), None),
            (HeroId::new(1), Some(&survivors[..])),
            (HeroId::new(2), None),
        ],
        &mut events,
    );
    assert_eq!(state.hero(HeroId::new(1)).unwrap().army, survivors);
    assert_eq!(state.hero(HeroId::new(2)), None);
    assert_eq!(state.heroes().len(), 3);
    // Both players see blue's hero's tile.
    assert_eq!(
        events,
        vec![AdventureEvent {
            kind: AdventureEventKind::HeroRemoved {
                hero: HeroId::new(2),
                pos: p(3, 0),
            },
            seen_by: Viewers::NONE.with(RED).with(BLUE),
        }]
    );
    assert_eq!(
        events[0].project(RED).unwrap().seen_by,
        Viewers::NONE.with(RED)
    );
    // A hero not on the map is skipped.
    let mut again = before.clone();
    again.end_battle(&rules.0, &[(HeroId::new(9), None)], &mut events);
    assert_eq!(again, before);
}

/// The hero of a battle's defending side; panics for a monster's or guards'.
fn defender_hero(battle: &BattleStart) -> HeroId {
    match &battle.defender {
        Defender::Hero(side) => side.hero,
        Defender::Monster { .. } | Defender::Guards { .. } => panic!("no hero defends"),
    }
}
