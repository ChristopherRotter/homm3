//! Tests of the teleporters' jumps through the game (E08 T105a1; ADV-TEL-003, ADR-0017 point 4):
//! a monolith's exit is drawn from the acting player's stream, after the whole command is
//! checked; a guarded entrance's jump follows the guard's battle as the held visit (GAM-DEC-007).
//! Since E08 T105b1 an enemy hero on the exit is fought there first, the jump held as a rest
//! after the winner's levels (`Rest::Jump`; GAM-BAT-004, GAM-DEC-007 amended).

use h3_adventure::{AdventureCommandError, AdventureEventKind, ObjectSetup, TileMask};

use super::decisions::{move_hero, play};
use super::mines::{adventure, fight};
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

/// [`game`] with Blue's hero on `at` leading `count` Imps and Red's with `red` Imps at 999
/// experience, one point from its second level.
fn facing(
    rules: &GameRuleset,
    objects: Vec<ObjectSetup>,
    at: Pos,
    (red, count): (u32, u32),
) -> GameState {
    let mut setup = setup();
    setup.map.objects = objects;
    let imps = |count| {
        HeroArmy::Custom(vec![ArmyStack {
            slot: 0,
            creature: IMP,
            count,
        }])
    };
    setup.heroes[0].army = imps(red);
    setup.heroes[0].details.experience = Some(999);
    setup.heroes[1].pos = at;
    setup.heroes[1].army = imps(count);
    GameState::new(rules, &setup, 7).unwrap()
}

/// Whether `events` start a battle at `tile`.
fn starts_at(events: &[GameEvent], tile: Pos) -> bool {
    events
        .iter()
        .any(|e| matches!(e, GameEvent::BattleStarted { tile: at, .. } if *at == tile))
}

// covers: GAM-BAT-004, GAM-DEC-007, ADV-TEL-004, ADV-TEL-007
#[test]
fn a_battle_at_a_monoliths_exit_holds_the_jump_until_the_winners_levels() {
    let rules = rules();
    // Blue's hero, with one Imp, on the one exit: no draw, the battle at the exit.
    let exit = Pos::new(2, 2, 0);
    let objects = vec![
        monolith(&rules, "t:entrance", AT),
        monolith(&rules, "t:exit", exit),
    ];
    let mut state = facing(&rules, objects, exit, (9, 1));
    let events = play(&mut state, &rules, RED, move_hero(0, &[AT])).unwrap();
    assert!(starts_at(&events, exit), "{events:?}");
    assert!(exits(&events).is_empty());
    assert_eq!(state.adventure().hero(HERO).unwrap().pos, AT);
    // The jump is held beside the open battle, in the hash (tag 4, game state hash v27).
    assert_eq!(state.held_rests(), 1);
    let mut bare = state.clone();
    bare.drop_rests();
    assert_ne!(state.state_hash(), bare.state_hash());
    // Won: Blue's hero leaves the map and Red's gains its level, still on the entrance while it
    // is open; the jump once it is answered.
    let events = fight(&mut state, &rules);
    assert!(exits(&events).is_empty());
    assert!(state.adventure().hero(HeroId::new(1)).is_none());
    assert!(state.level_up(RED).is_some());
    assert_eq!(state.adventure().hero(HERO).unwrap().pos, AT);
    let events = win(&mut state, &rules);
    assert_eq!(exits(&events), [exit]);
    assert_eq!(state.adventure().hero(HERO).unwrap().pos, exit);
}

// covers: GAM-BAT-004, GAM-DEC-007
#[test]
fn a_mover_lost_at_a_monoliths_exit_never_jumps() {
    let rules = rules();
    let exit = Pos::new(2, 2, 0);
    let objects = vec![
        monolith(&rules, "t:entrance", AT),
        monolith(&rules, "t:exit", exit),
    ];
    // One Imp against twenty: Red's hero loses and leaves the map; the held jump finds no hero.
    let mut state = facing(&rules, objects, exit, (1, 20));
    let events = play(&mut state, &rules, RED, move_hero(0, &[AT])).unwrap();
    assert!(starts_at(&events, exit));
    let events = fight(&mut state, &rules);
    assert!(exits(&events).is_empty());
    assert!(state.adventure().hero(HERO).is_none());
    assert_eq!(state.adventure().hero(HeroId::new(1)).unwrap().pos, exit);
    assert!(state.battle(RED).is_none() && !state.has_decision(RED));
}
