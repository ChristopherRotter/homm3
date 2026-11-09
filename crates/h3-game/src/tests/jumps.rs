//! Tests of the teleporters' jumps through the game (E08 T105a1; ADV-TEL-003, ADR-0017 point 4):
//! a monolith's exit is drawn from the acting player's stream, after the whole command is
//! checked; a guarded entrance's jump follows the guard's battle as the held visit (GAM-DEC-007).

use h3_adventure::{AdventureCommandError, AdventureEventKind, ObjectSetup, TileMask};

use super::decisions::{move_hero, play};
use super::mines::adventure;
use super::rewards::{AT, HERO, guard, starts_battle, win};
use super::*;

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);

/// A monolith of type `k` on `anchor`, visited from it.
fn monolith(rules: &GameRuleset, k: &str, anchor: Pos) -> ObjectSetup {
    ObjectSetup {
        scenery: Default::default(),
        anchor,
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: rules.adventure().object_type_by_key(k),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        holds: Default::default(),
        owner: None,
    }
}

/// [`setup`] with `objects`, seed 7.
fn game(rules: &GameRuleset, objects: Vec<ObjectSetup>) -> GameState {
    let mut setup = setup();
    setup.map.objects = objects;
    GameState::new(rules, &setup, 7).unwrap()
}

/// The tiles red's hero jumped to among `events`.
fn exits(events: &[GameEvent]) -> Vec<Pos> {
    adventure(events)
        .iter()
        .filter_map(|e| match e.kind {
            AdventureEventKind::HeroTeleported { to, .. } => to,
            _ => None,
        })
        .collect()
}

// covers: ADV-TEL-003
#[test]
fn a_monoliths_exit_is_drawn_from_the_acting_players_stream() {
    let rules = rules();
    // A one-way entrance on (1, 0) and three exits in the walk's order (level, row, column); the
    // 4 × 4 map is explored by both heroes.
    let free = [Pos::new(3, 0, 0), Pos::new(3, 1, 0), Pos::new(0, 3, 0)];
    let mut objects = vec![monolith(&rules, "t:entrance", AT)];
    objects.extend(free.map(|exit| monolith(&rules, "t:exit", exit)));
    let mut state = game(&rules, objects);
    let blue = state.stream(BLUE);
    // A rejected move leaves the stream where it was.
    let before = state.stream(RED);
    assert_eq!(
        play(
            &mut state,
            &rules,
            RED,
            move_hero(0, &[AT, Pos::new(2, 0, 0)])
        ),
        Err(GameCommandError::Adventure(
            AdventureCommandError::PastVisit { step: 1 }
        ))
    );
    assert_eq!(state.stream(RED), before);
    // The step onto the entrance: one draw among the three exits from red's stream.
    let mut expected = before;
    let place = usize::try_from(expected.below(3)).unwrap();
    let events = play(&mut state, &rules, RED, move_hero(0, &[AT])).unwrap();
    assert_eq!(exits(&events), [free[place]]);
    assert_eq!(state.adventure().hero(HERO).unwrap().pos, free[place]);
    assert_eq!(state.stream(RED), expected);
    assert_eq!(state.stream(BLUE), blue);
}

// covers: ADV-MON-006, GAM-DEC-007, ADV-TEL-004
#[test]
fn a_guarded_entrances_jump_follows_its_guards_battle() {
    let rules = rules();
    // The monster on (2, 0) guards the entrance on (1, 0); one exit, no draw.
    let exit = Pos::new(0, 3, 0);
    let objects = vec![
        monolith(&rules, "t:entrance", AT),
        monolith(&rules, "t:exit", exit),
        guard(&rules),
    ];
    let mut state = game(&rules, objects);
    let events = play(&mut state, &rules, RED, move_hero(0, &[AT])).unwrap();
    assert!(starts_battle(&events), "{events:?}");
    assert!(exits(&events).is_empty());
    assert_eq!(state.adventure().hero(HERO).unwrap().pos, AT);
    // Won: the jump after the battle's end and the winner's levels.
    let events = win(&mut state, &rules);
    assert_eq!(exits(&events), [exit]);
    assert_eq!(state.adventure().hero(HERO).unwrap().pos, exit);
}
