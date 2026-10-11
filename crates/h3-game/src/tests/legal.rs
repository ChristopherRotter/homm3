//! Tests of the legal commands (E08 T8b2a): who has any, their order, and that each applies with
//! the points the reachable-tiles query reports.

use h3_adventure::AdventureCommand;

use super::*;

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);

/// The hero and the path of a listed move.
fn as_move(command: &GameCommand) -> (HeroId, &[Pos]) {
    let GameCommand::Adventure(AdventureCommand::MoveHero { hero, path }) = command else {
        panic!("not a move: {command:?}");
    };
    (*hero, path)
}

/// The tiles of the 4 × 4 map but `skip`, row by row.
fn tiles_but(skip: &[Pos]) -> Vec<Pos> {
    (0..4)
        .flat_map(|y| (0..4).map(move |x| Pos::new(x, y, 0)))
        .filter(|pos| !skip.contains(pos))
        .collect()
}

#[test]
fn the_acting_player_may_end_its_turn_or_move_a_hero_to_each_reachable_tile() {
    let rules = rules();
    let state = GameState::new(&rules, &setup(), 7).unwrap();
    let legal = legal_commands(&state, &rules, Actor::Player(RED));
    assert_eq!(legal[0], GameCommand::EndTurn);
    // Red's hero on (0, 0) reaches every tile but its own, blue's hero's and the rock's, row by
    // row, along the query's paths.
    let (red, rock) = (Pos::new(0, 0, 0), Pos::new(3, 3, 0));
    let reach = state
        .adventure()
        .reach(rules.core(), rules.adventure(), HeroId::new(0))
        .unwrap();
    let moves: Vec<(HeroId, &[Pos])> = legal[1..].iter().map(as_move).collect();
    let to: Vec<Pos> = moves
        .iter()
        .map(|(_, path)| *path.last().unwrap())
        .collect();
    assert_eq!(to, tiles_but(&[red, Pos::new(2, 2, 0), rock]));
    for (command, &(hero, path)) in legal[1..].iter().zip(&moves) {
        assert_eq!(hero, HeroId::new(0));
        let to = *path.last().unwrap();
        assert_eq!(reach.path(to).as_deref(), Some(path));
        // Each applies and leaves the points the query reports.
        let mut moved = state.clone();
        let actor = Actor::Player(RED);
        apply(&mut moved, &rules, actor, command.clone(), &mut NoopSink).unwrap();
        let hero = moved.adventure().hero(hero).unwrap();
        assert_eq!((hero.pos, Some(hero.movement)), (to, reach.points_left(to)));
    }
}

#[test]
fn a_player_whose_turn_is_not_running_has_none() {
    let rules = rules();
    let mut state = GameState::new(&rules, &setup(), 7).unwrap();
    let legal = |state: &GameState, player| legal_commands(state, &rules, Actor::Player(player));
    assert_eq!(legal(&state, BLUE), []);
    assert_eq!(legal(&state, PlayerId::new(4)), []);
    let end = GameCommand::EndTurn;
    apply(&mut state, &rules, Actor::Player(RED), end, &mut NoopSink).unwrap();
    assert_eq!(legal(&state, RED), []);
    // In blue's turn: its own hero's moves only, around red's hero on (0, 0).
    let blue = legal(&state, BLUE);
    assert_eq!(blue[0], GameCommand::EndTurn);
    let moves: Vec<(HeroId, &[Pos])> = blue[1..].iter().map(as_move).collect();
    assert!(moves.iter().all(|&(hero, _)| hero == HeroId::new(1)));
    let to: Vec<Pos> = moves
        .iter()
        .map(|(_, path)| *path.last().unwrap())
        .collect();
    let skip = [Pos::new(0, 0, 0), Pos::new(2, 2, 0), Pos::new(3, 3, 0)];
    assert_eq!(to, tiles_but(&skip));
}
