//! Tests of the revisit through the game (E08 T105b2; ADV-TEL-008, increment 11's design note
//! §13): listed after `EndTurn` (and the Shipyards' boats) for each hero on an entered object whose
//! visit is built; the game's checks first (the turn, an open decision); the visit's decisions
//! opened as a step's would be; no step and no points spent.

use h3_adventure::AdventureCommandError;

use super::decisions::{DECLINE, after_steps, game, move_hero, play, revisit};
use super::*;

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);

/// Red's hero's tile, points and day's maximum.
fn red_hero(state: &GameState) -> (Pos, u32, u32) {
    let hero = state.adventure().hero(HeroId::new(0)).unwrap();
    (hero.pos, hero.movement, hero.max_movement)
}

// covers: ADV-TEL-008, GAM-DEC-001, ADV-VIS-001
#[test]
fn a_revisit_is_listed_on_an_entered_object_and_opens_its_offer_again() {
    let rules = rules();
    let chart = rules.adventure().object_type_by_key("t:chart").unwrap();
    let red = Actor::Player(RED);
    // 3 000 gold: the Cartographer's offer opens at the step and at the revisit.
    let mut state = game(&rules, 0);
    assert_eq!(legal_commands(&state, &rules, red), [GameCommand::EndTurn]);
    assert_eq!(
        play(&mut state, &rules, RED, revisit(0)),
        Err(GameCommandError::Adventure(
            AdventureCommandError::NothingToRevisit
        ))
    );
    let enter = [Pos::new(1, 0, 0), Pos::new(2, 0, 0)];
    play(&mut state, &rules, RED, move_hero(0, &enter)).unwrap();
    assert!(state.decision(RED).is_some());
    // The offer open: the game refuses the revisit first (GAM-DEC-001).
    assert_eq!(
        play(&mut state, &rules, RED, revisit(0)),
        Err(GameCommandError::DecisionOpen)
    );
    play(&mut state, &rules, RED, DECLINE).unwrap();
    assert_eq!(
        legal_commands(&state, &rules, red),
        [GameCommand::EndTurn, revisit(0)]
    );
    assert_eq!(legal_commands(&state, &rules, Actor::Player(BLUE)), []);
    assert_eq!(
        play(&mut state, &rules, BLUE, revisit(1)),
        Err(GameCommandError::NotYourTurn(BLUE))
    );
    // The revisit: the offer again, without a step, the points, the day's maximum and red's
    // stream as they were (the visit draws nothing).
    let (before, stream) = (red_hero(&state), state.stream(RED));
    let events = play(&mut state, &rules, RED, revisit(0)).unwrap();
    assert_eq!(after_steps(events.clone()), events);
    let opened = GameEvent::OfferOpened {
        player: RED,
        hero: HeroId::new(0),
        object_type: chart,
        anchor: Pos::new(2, 0, 0),
    };
    assert_eq!(events.last(), Some(&opened));
    assert_eq!((red_hero(&state), state.stream(RED)), (before, stream));
    assert!(state.decision(RED).is_some());
    assert_eq!(legal_commands(&state, &rules, red).len(), 2);
}
