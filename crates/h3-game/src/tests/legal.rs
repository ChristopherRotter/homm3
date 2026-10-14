//! Tests of the legal commands and moves (E08 T8b2a): who has any, that the moves are the
//! reachable-tiles query of the actor's own heroes, and that each move applies with the points
//! the query reports.

use h3_adventure::AdventureCommand;

use super::*;

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);

/// The tiles of the 4 × 4 map but `skip`, row by row.
fn tiles_but(skip: &[Pos]) -> Vec<Pos> {
    (0..4)
        .flat_map(|y| (0..4).map(move |x| Pos::new(x, y, 0)))
        .filter(|pos| !skip.contains(pos))
        .collect()
}

#[test]
fn the_acting_player_may_end_its_turn_or_move_its_hero_to_each_reachable_tile() {
    let rules = rules();
    let state = GameState::new(&rules, &setup(), 7).unwrap();
    let actor = Actor::Player(RED);
    assert_eq!(
        legal_commands(&state, &rules, actor),
        [GameCommand::EndTurn]
    );
    // Red's hero on (0, 0) reaches every tile but its own and the rock's; blue's hero's (2, 2) a
    // visit, a battle (E08 T19a).
    let hero = HeroId::new(0);
    let moves = legal_moves(&state, &rules, actor, hero).unwrap();
    let query = state
        .adventure()
        .reach(rules.core(), rules.adventure(), hero);
    assert_eq!(Some(&moves), query.as_ref());
    let to: Vec<Pos> = moves.tiles().map(|(pos, _)| pos).collect();
    let skip = [Pos::new(0, 0, 0), Pos::new(3, 3, 0)];
    assert_eq!(to, tiles_but(&skip));
    // Each path applies and leaves the points the query reports, on the tile before blue's hero
    // for its visit.
    let blue = Pos::new(2, 2, 0);
    for (to, left) in moves.tiles() {
        let path = moves.path(to).unwrap();
        let command = GameCommand::Adventure(AdventureCommand::MoveHero {
            hero,
            path: path.clone(),
        });
        let mut moved = state.clone();
        apply(&mut moved, &rules, actor, command, &mut NoopSink).unwrap();
        let at = if to == blue { path[path.len() - 2] } else { to };
        let hero = moved.adventure().hero(hero).unwrap();
        assert_eq!((hero.pos, hero.movement), (at, left));
        assert_eq!(moved.battle(RED).is_some(), to == blue);
    }
}

#[test]
fn a_player_has_nothing_outside_its_turn_nor_another_players_hero() {
    let rules = rules();
    let mut state = GameState::new(&rules, &setup(), 7).unwrap();
    let (red, blue) = (Actor::Player(RED), Actor::Player(BLUE));
    let (red_hero, blue_hero) = (HeroId::new(0), HeroId::new(1));
    assert_eq!(legal_commands(&state, &rules, blue), []);
    assert_eq!(
        legal_commands(&state, &rules, Actor::Player(PlayerId::new(4))),
        []
    );
    assert_eq!(legal_moves(&state, &rules, blue, blue_hero), None);
    assert_eq!(legal_moves(&state, &rules, red, blue_hero), None);
    assert_eq!(legal_moves(&state, &rules, red, HeroId::new(9)), None);
    apply(&mut state, &rules, red, GameCommand::EndTurn, &mut NoopSink).unwrap();
    assert_eq!(legal_commands(&state, &rules, red), []);
    assert_eq!(legal_moves(&state, &rules, red, red_hero), None);
    // In blue's turn: its own hero's moves, red's hero's tile (0, 0) a visit.
    assert_eq!(legal_commands(&state, &rules, blue), [GameCommand::EndTurn]);
    let moves = legal_moves(&state, &rules, blue, blue_hero).unwrap();
    let to: Vec<Pos> = moves.tiles().map(|(pos, _)| pos).collect();
    let skip = [Pos::new(2, 2, 0), Pos::new(3, 3, 0)];
    assert_eq!(to, tiles_but(&skip));
}
