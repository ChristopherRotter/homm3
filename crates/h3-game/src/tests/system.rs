//! Tests of the system actor's own commands (E16 M5 T14, ADR-0017 point 2): a player's turn
//! ended (GAM-SYS-001) and its decision answered by its default (GAM-SYS-002) as the player's
//! own commands, each refused unless it applies, none from a player; the battle side's in
//! `battles.rs`, the level-up's hold in `level_ups.rs`.

use super::decisions::{self, BLUE, DECLINE, RED, play};
use super::*;

/// The system actor's `command`, its events; a rejected command must leave the state as it was.
pub(super) fn system(
    state: &mut GameState,
    rules: &GameRuleset,
    command: SystemCommand,
) -> Result<Vec<GameEvent>, GameCommandError> {
    let before = state.clone();
    let mut events = Vec::new();
    let command = GameCommand::System(command);
    let result = apply(state, rules, Actor::System, command, &mut events);
    if result.is_err() {
        assert!(events.is_empty(), "a rejected command emitted events");
        assert_eq!(state, &before, "a rejected command changed the state");
    }
    result.map(|()| events)
}

/// `player`'s `command` and the system's `system_command` from `state` apply alike: the same
/// state and the same events.
fn alike(
    state: &GameState,
    rules: &GameRuleset,
    (player, command): (PlayerId, GameCommand),
    system_command: SystemCommand,
) -> GameState {
    let mut own = state.clone();
    let told = play(&mut own, rules, player, command).unwrap();
    let mut by_system = state.clone();
    assert_eq!(system(&mut by_system, rules, system_command), Ok(told));
    assert_eq!(by_system, own);
    assert_eq!(by_system.state_hash(), own.state_hash());
    by_system
}

// covers: GAM-SYS-001
#[test]
fn the_system_ends_a_players_turn_as_its_own_end_turn() {
    let rules = rules();
    let mut state = decisions::game(&rules, 0);
    let stranger = PlayerId::new(5);
    use GameCommandError as E;
    let end = |player| SystemCommand::EndTurn { player };
    // Refused unless the player's turn is running.
    assert_eq!(
        system(&mut state, &rules, end(BLUE)),
        Err(E::NotYourTurn(BLUE))
    );
    assert_eq!(
        system(&mut state, &rules, end(stranger)),
        Err(E::UnknownPlayer(stranger))
    );
    // Not listed: the host issues them by its clock.
    assert_eq!(legal_commands(&state, &rules, Actor::System), []);
    let state = alike(&state, &rules, (RED, GameCommand::EndTurn), end(RED));
    assert!(state.is_acting(BLUE) && !state.is_acting(RED));
    assert_eq!(end(BLUE).player(), BLUE);
}

// covers: GAM-SYS-001, GAM-SYS-002
#[test]
fn the_systems_answer_is_its_players_default() {
    let rules = rules();
    let mut state = decisions::game(&rules, 0);
    use GameCommandError as E;
    let answer = |player| SystemCommand::Answer { player };
    let stranger = PlayerId::new(5);
    assert_eq!(system(&mut state, &rules, answer(RED)), Err(E::NoDecision));
    assert_eq!(
        system(&mut state, &rules, answer(stranger)),
        Err(E::UnknownPlayer(stranger))
    );
    // Red enters the Cartographer: its offer is open, its turn cannot end before its answer.
    let enter = [Pos::new(1, 0, 0), Pos::new(2, 0, 0)];
    play(&mut state, &rules, RED, decisions::move_hero(0, &enter)).unwrap();
    assert!(state.decision(RED).is_some());
    let end = SystemCommand::EndTurn { player: RED };
    assert_eq!(system(&mut state, &rules, end), Err(E::DecisionOpen));
    assert_eq!(system(&mut state, &rules, answer(BLUE)), Err(E::NoDecision));
    // The default declines it, as red's own `Decline`; then the turn may end.
    let mut state = alike(&state, &rules, (RED, DECLINE), answer(RED));
    assert!(!state.has_decision(RED));
    system(&mut state, &rules, end).unwrap();
}

// covers: GAM-SYS-001, GAM-SYS-002, GAM-SYS-003
#[test]
fn no_player_issues_a_system_command() {
    let rules = rules();
    let mut state = decisions::game(&rules, 0);
    for player in [RED, BLUE] {
        for command in [
            SystemCommand::EndTurn { player: RED },
            SystemCommand::Answer { player: BLUE },
            SystemCommand::Battle {
                player,
                command: h3_battle::Command::Wait,
            },
        ] {
            assert_eq!(
                play(&mut state, &rules, player, GameCommand::System(command)),
                Err(GameCommandError::SystemOnly)
            );
        }
    }
}
