//! Tests of `MoveHero` (E08 T7b): a move pays each step from the points left, the path is
//! checked step by step and rejected whole, one event per step. E08 T8a: the fog of war around
//! moves: the reveals at the start and after each step, explored tiles only, the steps' viewers.

extern crate std;

use std::string::{String, ToString};
use std::vec;
use std::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::{HeroId, PlayerId};

use super::*;
use crate::fog::reveal;
use crate::persistent::BitGrid;
use crate::ruleset::AdventureRuleset;
use crate::state::AdventureState;
use crate::testing::{GNOLL, PEASANT, hero_at, movement_rules, world, world_state};

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
    let result = state.apply(&rules.0, &rules.1, player, &command, &mut events);
    assert!(
        result.is_ok() || events.is_empty(),
        "a rejected move emitted events"
    );
    result.map(|()| events)
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
    let stepped = |from, to| AdventureEvent {
        kind: AdventureEventKind::HeroStepped {
            hero: HeroId::new(1),
            from,
            to,
        },
        seen_by: Viewers::NONE.with(RED),
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
            hero_at(2, 1, p(5, 2), &[PEASANT]),
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
    // Rock and sea, an object's blocked tile, another hero, off the map.
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
        rejected(&mut state, &rules, RED, 3, &[p(5, 1), p(5, 2)]),
        blocked(1, StandError::Occupied)
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
            "step 0 enters a tile an object is visited from",
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
    let state = AdventureState::new(&rules.0, &rules.1, &setup).unwrap();
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
    // Leaving a tile blue has explored is seen too, onto a tile it has not explored as well.
    move_hero(
        &mut state,
        &rules,
        BLUE,
        2,
        &[p(9, 0), p(8, 0), p(7, 0), p(6, 0)],
    )
    .unwrap();
    let back = move_hero(&mut state, &rules, RED, 1, &[p(4, 0)]).unwrap();
    assert_eq!(back[0].seen_by, both);
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
