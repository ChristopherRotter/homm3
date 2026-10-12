//! Tests of the pending decisions (E08 T14b; ADR-0017 points 6 and 10): a Cartographer's offer
//! opens as its visitor's owner's decision if its treasury can pay, the answers are then the
//! owner's only legal commands, and an accepted offer is paid and carried out.

use h3_adventure::{
    AdventureCommand, AdventureEvent, AdventureEventKind, ObjectSetup, Offer, TileMask, Viewers,
};
use h3_content::ResourceId;

use super::*;

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const GOLD: ResourceId = ResourceId::from_raw(0);
const ACCEPT: GameCommand = GameCommand::Answer(Answer::Accept);
const DECLINE: GameCommand = GameCommand::Answer(Answer::Decline);

fn move_hero(hero: u16, path: &[Pos]) -> GameCommand {
    GameCommand::Adventure(AdventureCommand::MoveHero {
        hero: HeroId::new(hero),
        path: path.to_vec(),
    })
}

/// [`setup`] at `difficulty` (0: 3 000 gold, 1: 2 000) with a chart at (2, 0), entered from the
/// side; red's hero stands at (0, 0), blue's at (2, 2).
fn game(rules: &GameRuleset, difficulty: u8) -> GameState {
    let mut setup = setup();
    setup.difficulty = difficulty;
    setup.map.objects.push(ObjectSetup {
        anchor: Pos::new(2, 0, 0),
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: rules.adventure().object_type_by_key("t:chart"),
        amount: None,
    });
    GameState::new(rules, &setup, 7).unwrap()
}

/// `command` of `player`, its events; a rejected command must leave the state as it was.
fn play(
    state: &mut GameState,
    rules: &GameRuleset,
    player: PlayerId,
    command: GameCommand,
) -> Result<Vec<GameEvent>, GameCommandError> {
    let before = state.clone();
    let mut events = Vec::new();
    let result = apply(state, rules, Actor::Player(player), command, &mut events);
    if result.is_err() {
        assert!(events.is_empty(), "a rejected command emitted events");
        assert_eq!(state, &before, "a rejected command changed the state");
    }
    result.map(|()| events)
}

/// The events after a move's steps.
fn after_steps(events: Vec<GameEvent>) -> Vec<GameEvent> {
    events
        .into_iter()
        .filter(|e| {
            !matches!(
                e,
                GameEvent::Adventure(AdventureEvent {
                    kind: AdventureEventKind::HeroStepped { .. },
                    ..
                })
            )
        })
        .collect()
}

// covers: GAM-DEC-001, ADV-VIS-001
#[test]
fn an_open_offer_leaves_its_owner_its_answers_only() {
    let rules = rules();
    let chart = rules.adventure().object_type_by_key("t:chart").unwrap();
    let mut state = game(&rules, 0);
    let (red, blue) = (Actor::Player(RED), Actor::Player(BLUE));
    let enter = [Pos::new(1, 0, 0), Pos::new(2, 0, 0)];
    let events = play(&mut state, &rules, RED, move_hero(0, &enter)).unwrap();
    let opened = GameEvent::OfferOpened {
        player: RED,
        hero: HeroId::new(0),
        object_type: chart,
        anchor: Pos::new(2, 0, 0),
    };
    assert_eq!(after_steps(events), [opened]);
    assert_eq!(project(&opened, Viewer::Player(RED)), Some(opened));
    assert_eq!(project(&opened, Viewer::Player(BLUE)), None);
    let offer = Offer {
        hero: HeroId::new(0),
        object_type: chart,
        anchor: Pos::new(2, 0, 0),
        price: vec![(GOLD, 2500)],
    };
    assert_eq!(state.decision(RED), Some(&offer));
    assert_eq!(state.decision(BLUE), None);
    assert_eq!(
        observe(&state, &rules, Viewer::Player(RED)).decision,
        Some(offer)
    );
    assert_eq!(observe(&state, &rules, Viewer::Player(BLUE)).decision, None);
    // Its answers only: no move, no end of the turn.
    assert_eq!(legal_commands(&state, &rules, red), [ACCEPT, DECLINE]);
    assert_eq!(legal_moves(&state, &rules, red, HeroId::new(0)), None);
    assert_eq!(legal_commands(&state, &rules, blue), []);
    let open = Err(GameCommandError::DecisionOpen);
    assert_eq!(play(&mut state, &rules, RED, GameCommand::EndTurn), open);
    assert_eq!(
        play(&mut state, &rules, RED, move_hero(0, &[Pos::new(1, 0, 0)])),
        open
    );
    // Nobody else answers it.
    assert_eq!(
        play(&mut state, &rules, BLUE, ACCEPT),
        Err(GameCommandError::NoDecision)
    );
    let stranger = PlayerId::new(4);
    assert_eq!(
        play(&mut state, &rules, stranger, DECLINE),
        Err(GameCommandError::UnknownPlayer(stranger))
    );
    // Declined: the decision closes and nothing else changes.
    let open_hash = state.state_hash();
    let events = play(&mut state, &rules, RED, DECLINE).unwrap();
    let declined = GameEvent::OfferAnswered {
        player: RED,
        object_type: chart,
        anchor: Pos::new(2, 0, 0),
        accepted: false,
    };
    assert_eq!(events, [declined]);
    assert_eq!(project(&declined, Viewer::Player(BLUE)), None);
    assert_eq!(state.decision(RED), None);
    assert_ne!(state.state_hash(), open_hash);
    assert_eq!(state.treasury(RED), Some(&[3000][..]));
    assert!(!state.adventure().has_bought(RED, chart));
    assert_eq!(legal_commands(&state, &rules, red), [GameCommand::EndTurn]);
    assert_eq!(
        play(&mut state, &rules, RED, DECLINE),
        Err(GameCommandError::NoDecision)
    );
    // Asked again on the next visit; accepted, the price is paid, then the chart reveals.
    play(&mut state, &rules, RED, move_hero(0, &[Pos::new(1, 0, 0)])).unwrap();
    play(&mut state, &rules, RED, move_hero(0, &[Pos::new(2, 0, 0)])).unwrap();
    let events = play(&mut state, &rules, RED, ACCEPT).unwrap();
    let paid = GameEvent::ResourcePaid {
        player: RED,
        resource: GOLD,
        amount: 2500,
    };
    assert_eq!(
        events,
        [
            GameEvent::OfferAnswered {
                player: RED,
                object_type: chart,
                anchor: Pos::new(2, 0, 0),
                accepted: true,
            },
            paid,
            // Red's hero explored the whole 4 × 4 map at the start.
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::TilesRevealed {
                    object_type: chart,
                    count: 0,
                },
                seen_by: Viewers::NONE.with(RED),
            }),
        ]
    );
    assert_eq!(project(&paid, Viewer::Player(RED)), Some(paid));
    assert_eq!(project(&paid, Viewer::Player(BLUE)), None);
    assert_eq!(state.treasury(RED), Some(&[500][..]));
    assert_eq!(state.treasury(BLUE), Some(&[3000][..]));
    assert!(state.adventure().has_bought(RED, chart));
    let view = observe(&state, &rules, Viewer::Player(RED));
    assert_eq!((view.decision, view.adventure.bought), (None, vec![chart]));
    // Bought: the next visit offers nothing.
    play(&mut state, &rules, RED, move_hero(0, &[Pos::new(1, 0, 0)])).unwrap();
    let events = play(&mut state, &rules, RED, move_hero(0, &[Pos::new(2, 0, 0)])).unwrap();
    assert_eq!(after_steps(events), []);
    play(&mut state, &rules, RED, GameCommand::EndTurn).unwrap();
}

// covers: ADV-VIS-001
#[test]
fn an_offer_its_owner_cannot_pay_does_not_open() {
    let rules = rules();
    let chart = rules.adventure().object_type_by_key("t:chart").unwrap();
    let mut state = game(&rules, 1);
    let enter = [Pos::new(1, 0, 0), Pos::new(2, 0, 0)];
    let events = play(&mut state, &rules, RED, move_hero(0, &enter)).unwrap();
    let refused = GameEvent::OfferRefused {
        player: RED,
        hero: HeroId::new(0),
        object_type: chart,
        anchor: Pos::new(2, 0, 0),
    };
    assert_eq!(after_steps(events), [refused]);
    assert_eq!(project(&refused, Viewer::Player(RED)), Some(refused));
    assert_eq!(project(&refused, Viewer::Player(BLUE)), None);
    assert_eq!(state.decision(RED), None);
    assert_eq!(state.treasury(RED), Some(&[2000][..]));
    assert_eq!(
        legal_commands(&state, &rules, Actor::Player(RED)),
        [GameCommand::EndTurn]
    );
    assert_eq!(
        play(&mut state, &rules, RED, ACCEPT),
        Err(GameCommandError::NoDecision)
    );
}

// covers: GAM-DEC-001
#[test]
fn a_decision_blocks_no_one_else() {
    let rules = rules();
    let mut state = game(&rules, 0);
    state.set_schedule(&rules, vec![vec![RED, BLUE]]);
    let enter = [Pos::new(1, 0, 0), Pos::new(2, 0, 0)];
    play(&mut state, &rules, RED, move_hero(0, &enter)).unwrap();
    // Blue, acting in the same group, moves and ends its turn; red's turn goes on until it
    // answers, and the group's with it.
    play(&mut state, &rules, BLUE, move_hero(1, &[Pos::new(2, 1, 0)])).unwrap();
    play(&mut state, &rules, BLUE, GameCommand::EndTurn).unwrap();
    assert!(state.is_acting(RED) && state.decision(RED).is_some());
    assert_eq!(
        legal_commands(&state, &rules, Actor::Player(RED)),
        [ACCEPT, DECLINE]
    );
    play(&mut state, &rules, RED, ACCEPT).unwrap();
    let events = play(&mut state, &rules, RED, GameCommand::EndTurn).unwrap();
    assert!(events.contains(&GameEvent::DayStarted { day: 2 }));
}

// covers: GAM-RES-001
#[test]
fn a_treasury_pays_what_it_holds() {
    let mut treasuries = Treasuries::new(2, &[3000]);
    assert!(treasuries.can_pay(RED, &[(GOLD, 3000)]));
    assert!(treasuries.can_pay(RED, &[]));
    assert!(!treasuries.can_pay(RED, &[(GOLD, 3001)]));
    // A resource the ruleset does not have, or a player without a treasury, pays nothing.
    assert!(!treasuries.can_pay(RED, &[(ResourceId::from_raw(1), 1)]));
    assert!(!treasuries.can_pay(PlayerId::new(2), &[(GOLD, 1)]));
    assert_eq!(treasuries.pay(PlayerId::new(2), GOLD, 1), 0);
    assert_eq!(treasuries.pay(RED, GOLD, 1000), 1000);
    assert_eq!(treasuries.of(RED), Some(&[2000][..]));
    assert_eq!(treasuries.of(BLUE), Some(&[3000][..]));
    // Never below 0.
    assert_eq!(treasuries.pay(RED, GOLD, 5000), 2000);
    assert_eq!(treasuries.of(RED), Some(&[0][..]));
}

#[test]
fn the_decision_errors_read_well() {
    assert_eq!(
        GameCommandError::DecisionOpen.to_string(),
        "an open decision waits for its answer"
    );
    assert_eq!(
        GameCommandError::NoDecision.to_string(),
        "no decision is open"
    );
}
