//! Tests of the adventure's commands through the game (E08 T7b): the game checks the turn, the
//! adventure the rest; its events become game events.

use h3_adventure::{
    AdventureCommand, AdventureCommandError, AdventureEvent, AdventureEventKind, Viewers,
};

use super::*;

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);

fn move_hero(hero: u16, path: &[Pos]) -> GameCommand {
    GameCommand::Adventure(AdventureCommand::MoveHero {
        hero: HeroId::new(hero),
        path: path.to_vec(),
    })
}

fn play(
    state: &mut GameState,
    rules: &GameRuleset,
    player: PlayerId,
    command: GameCommand,
) -> Result<Vec<GameEvent>, GameCommandError> {
    let mut events = Vec::new();
    let result = apply(state, rules, Actor::Player(player), command, &mut events);
    assert!(
        result.is_ok() || events.is_empty(),
        "a rejected command emitted events"
    );
    result.map(|()| events)
}

// covers: ADV-MOV-007
#[test]
fn a_move_goes_to_the_adventure_in_the_players_turn() {
    let rules = rules();
    let mut state = GameState::new(&rules, &setup(), 7).unwrap();
    let (from, to) = (Pos::new(0, 0, 0), Pos::new(1, 0, 0));
    let events = play(&mut state, &rules, RED, move_hero(0, &[to])).unwrap();
    // The map is 4 × 4: both heroes' scouting explored all of it at the start.
    let both = Viewers::NONE.with(RED).with(BLUE);
    assert_eq!(
        events,
        vec![GameEvent::Adventure(AdventureEvent {
            kind: AdventureEventKind::HeroStepped {
                hero: HeroId::new(0),
                from: Some(from),
                to: Some(to),
                seen_from: both,
                seen_to: both,
            },
            seen_by: both,
        })]
    );
    assert_eq!(state.adventure().hero(HeroId::new(0)).unwrap().pos, to);
}

#[test]
fn a_move_outside_the_turn_or_against_the_rules_changes_nothing() {
    let rules = rules();
    let mut state = GameState::new(&rules, &setup(), 7).unwrap();
    let before = state.clone();
    let step = [Pos::new(2, 1, 0)];
    assert_eq!(
        play(&mut state, &rules, BLUE, move_hero(1, &step)),
        Err(GameCommandError::NotYourTurn(BLUE))
    );
    assert_eq!(
        play(&mut state, &rules, PlayerId::new(4), move_hero(1, &step)),
        Err(GameCommandError::UnknownPlayer(PlayerId::new(4)))
    );
    // In red's turn, blue's hero is not red's to move.
    let error = play(&mut state, &rules, RED, move_hero(1, &step)).unwrap_err();
    assert_eq!(
        error,
        GameCommandError::Adventure(AdventureCommandError::NotYourHero(HeroId::new(1)))
    );
    assert_eq!(error.to_string(), "hero 1 is not the player's");
    assert_eq!(state, before);
}
