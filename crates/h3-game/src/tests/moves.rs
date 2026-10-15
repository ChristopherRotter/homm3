//! Tests of the adventure's commands through the game (E08 T7b): the game checks the turn, the
//! adventure the rest; its events become game events.

use h3_adventure::{
    AdventureCommand, AdventureCommandError, AdventureEvent, AdventureEventKind, MAX_GAIN,
    ObjectSetup, TileMask, Viewers,
};
use h3_content::ResourceId;

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

// covers: GAM-RES-001, ADV-PIK-001
#[test]
fn a_visits_gains_go_to_its_owner_s_treasury_and_are_told_to_it_alone() {
    let rules = rules();
    let mut setup = setup();
    // A gold pile at (2, 0), next to red's hero at (1, 0) after one step.
    setup.map.objects.push(ObjectSetup {
        anchor: Pos::new(2, 0, 0),
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: rules.adventure().object_type_by_key("t:pile"),
        amount: Some(7),
        drawn: TileMask::default(),
        monster: None,
        guards: None,
    });
    let mut state = GameState::new(&rules, &setup, 7).unwrap();
    let gold = ResourceId::from_raw(0);
    let path = [Pos::new(1, 0, 0), Pos::new(2, 0, 0)];
    let events = play(&mut state, &rules, RED, move_hero(0, &path)).unwrap();
    let gained = GameEvent::ResourceGained {
        player: RED,
        resource: gold,
        amount: 700,
    };
    assert_eq!(events.last(), Some(&gained));
    assert_eq!(state.treasury(RED), Some(&[2000 + 700][..]));
    assert_eq!(state.treasury(BLUE), Some(&[2000][..]));
    assert_eq!(project(&gained, Viewer::Player(RED)), Some(gained));
    assert_eq!(project(&gained, Viewer::Player(BLUE)), None);
    assert_eq!(
        observe(&state, &rules, Viewer::Player(RED)).treasury,
        [2700]
    );
}

// covers: GAM-RES-001
#[test]
fn a_gain_saturates_at_the_cap() {
    let gold = ResourceId::from_raw(0);
    let mut treasuries = Treasuries::new(2, &[MAX_GAIN - 5]);
    assert_eq!(treasuries.gain(BLUE, gold, 3), 3);
    assert_eq!(treasuries.gain(BLUE, gold, u32::MAX), 2);
    assert_eq!(treasuries.of(BLUE), Some(&[MAX_GAIN][..]));
    assert_eq!(treasuries.of(RED), Some(&[MAX_GAIN - 5][..]));
    // A resource the ruleset does not have, or a player without a treasury, gains nothing.
    assert_eq!(treasuries.gain(RED, ResourceId::from_raw(1), 9), 0);
    assert_eq!(treasuries.gain(PlayerId::new(2), gold, 9), 0);
    assert_eq!(treasuries.of(PlayerId::new(2)), None);
    assert_eq!(treasuries.of(RED), Some(&[MAX_GAIN - 5][..]));
}
