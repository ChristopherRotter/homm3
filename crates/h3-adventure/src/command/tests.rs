//! Tests of `MoveHero` (E08 T7b): a move pays each step from the points left, the path is
//! checked step by step and rejected whole, one event per step.

extern crate std;

use std::string::ToString;
use std::vec;
use std::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::{HeroId, PlayerId};

use super::*;
use crate::ruleset::AdventureRuleset;
use crate::state::AdventureState;
use crate::testing::{GNOLL, PEASANT, hero_at, movement_rules, world_state};

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
    let stepped = |from, to| AdventureEvent::HeroStepped {
        hero: HeroId::new(1),
        from,
        to,
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
    let start = || world_state(&rules, &rows, vec![hero_at(1, 0, p(0, 0), &[PEASANT])]);
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
