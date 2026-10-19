//! Tests of the pending decisions (E08 T14b; ADR-0017 points 6 and 10): a Cartographer's offer
//! opens as its visitor's owner's decision if its treasury can pay, the answers are then the
//! owner's only legal commands, and an accepted offer is paid and carried out; a border guard's
//! offer (T15a) has no price, and its acceptance checks that it still stands.

use h3_adventure::{
    AdventureCommand, AdventureEvent, AdventureEventKind, ObjectSetup, Offer, OfferKind, TileMask,
    Viewers,
};
use h3_content::ResourceId;

use super::*;

pub(super) const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
pub(super) const GOLD: ResourceId = ResourceId::from_raw(0);
pub(super) const ACCEPT: GameCommand = GameCommand::Answer(Answer::Accept);
pub(super) const DECLINE: GameCommand = GameCommand::Answer(Answer::Decline);

pub(super) fn move_hero(hero: u16, path: &[Pos]) -> GameCommand {
    GameCommand::Adventure(AdventureCommand::MoveHero {
        hero: HeroId::new(hero),
        path: path.to_vec(),
    })
}

/// [`setup`] at `difficulty` (0: 3 000 gold, 1: 2 000) with charts at (2, 0) and (3, 2), entered
/// from the side; red's hero stands at (0, 0), blue's at (2, 2).
fn game(rules: &GameRuleset, difficulty: u8) -> GameState {
    let mut setup = setup();
    setup.difficulty = difficulty;
    for anchor in [Pos::new(2, 0, 0), Pos::new(3, 2, 0)] {
        setup.map.objects.push(ObjectSetup {
            anchor,
            blocked: TileMask::from_bits(1),
            visitable: TileMask::from_bits(1),
            object_type: rules.adventure().object_type_by_key("t:chart"),
            amount: None,
            drawn: TileMask::default(),
            monster: None,
            guards: None,
        });
    }
    GameState::new(rules, &setup, 7).unwrap()
}

/// `command` of `player`, its events; a rejected command must leave the state as it was.
pub(super) fn play(
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
pub(super) fn after_steps(events: Vec<GameEvent>) -> Vec<GameEvent> {
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
        kind: OfferKind::Reveal {
            price: vec![(GOLD, 2500)],
        },
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
    // Blue, acting in the same group, moves on: into the other chart, a decision of its own.
    play(&mut state, &rules, BLUE, move_hero(1, &[Pos::new(3, 2, 0)])).unwrap();
    let (red, blue) = (state.decision(RED).unwrap(), state.decision(BLUE).unwrap());
    assert_eq!(
        (red.anchor, blue.anchor),
        (Pos::new(2, 0, 0), Pos::new(3, 2, 0))
    );
    let seen = observe(&state, &rules, Viewer::Player(BLUE)).decision;
    assert_eq!(seen.as_ref(), Some(blue));
    // Each answers its own; red's answer leaves blue's open.
    play(&mut state, &rules, RED, ACCEPT).unwrap();
    assert!(state.decision(RED).is_none() && state.decision(BLUE).is_some());
    assert_eq!(
        legal_commands(&state, &rules, Actor::Player(BLUE)),
        [ACCEPT, DECLINE]
    );
    assert_eq!(
        play(&mut state, &rules, BLUE, GameCommand::EndTurn),
        Err(GameCommandError::DecisionOpen)
    );
    // Red ends its turn; the group's turn goes on until blue answers and ends its turn.
    play(&mut state, &rules, RED, GameCommand::EndTurn).unwrap();
    play(&mut state, &rules, BLUE, DECLINE).unwrap();
    assert_eq!(state.decision(BLUE), None);
    let events = play(&mut state, &rules, BLUE, GameCommand::EndTurn).unwrap();
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
    assert_eq!(
        GameCommandError::OfferGone.to_string(),
        "the offer no longer stands"
    );
}

/// [`setup`] with a tent at (1, 0) and a guard at (3, 0), both of key 0 and blocking their tile;
/// red's hero stands at (0, 0), blue's at (2, 2).
fn keys_game(rules: &GameRuleset) -> GameState {
    let mut setup = setup();
    for (x, ty) in [(1, "t:tent"), (3, "t:guard")] {
        setup.map.objects.push(ObjectSetup {
            anchor: Pos::new(x, 0, 0),
            blocked: TileMask::from_bits(1),
            visitable: TileMask::from_bits(1),
            object_type: rules.adventure().object_type_by_key(ty),
            amount: None,
            drawn: TileMask::default(),
            monster: None,
            guards: None,
        });
    }
    GameState::new(rules, &setup, 7).unwrap()
}

/// The adventure events among `events`, but the steps.
fn told_beside_steps(events: Vec<GameEvent>) -> Vec<AdventureEventKind> {
    after_steps(events)
        .into_iter()
        .filter_map(|e| match e {
            GameEvent::Adventure(AdventureEvent { kind, .. }) => Some(kind),
            _ => None,
        })
        .collect()
}

// covers: GAM-DEC-001, ADV-VIS-003, ADV-VIS-004
#[test]
fn a_guard_offers_its_key_s_holders_to_pass_without_a_price() {
    let rules = rules();
    let guard = rules.adventure().object_type_by_key("t:guard").unwrap();
    let mut state = keys_game(&rules);
    let red = Actor::Player(RED);
    // Without the key, the guard's visit from (2, 1) does nothing.
    let toward = [Pos::new(1, 1, 0), Pos::new(2, 1, 0), Pos::new(3, 0, 0)];
    let events = play(&mut state, &rules, RED, move_hero(0, &toward)).unwrap();
    assert_eq!(after_steps(events), []);
    assert_eq!(state.decision(RED), None);
    // Red takes the key at the tent: told to red alone, seen in red's observation only.
    let mut state = keys_game(&rules);
    let events = play(&mut state, &rules, RED, move_hero(0, &[Pos::new(1, 0, 0)])).unwrap();
    assert_eq!(
        told_beside_steps(events),
        [AdventureEventKind::KeyGained { key: 0 }]
    );
    assert_eq!(
        observe(&state, &rules, Viewer::Player(RED)).adventure.keys,
        1
    );
    assert_eq!(
        observe(&state, &rules, Viewer::Player(BLUE)).adventure.keys,
        0
    );
    // With it, the guard's visit opens an offer to pass, without a price, at any treasury.
    let toward = [Pos::new(2, 1, 0), Pos::new(3, 0, 0)];
    let events = play(&mut state, &rules, RED, move_hero(0, &toward)).unwrap();
    let opened = GameEvent::OfferOpened {
        player: RED,
        hero: HeroId::new(0),
        object_type: guard,
        anchor: Pos::new(3, 0, 0),
    };
    assert_eq!(after_steps(events), [opened]);
    let offer = Offer {
        hero: HeroId::new(0),
        object_type: guard,
        anchor: Pos::new(3, 0, 0),
        kind: OfferKind::Pass,
    };
    assert_eq!(state.decision(RED), Some(&offer));
    assert_eq!(
        observe(&state, &rules, Viewer::Player(RED)).decision,
        Some(offer)
    );
    assert_eq!(legal_commands(&state, &rules, red), [ACCEPT, DECLINE]);
    // Declined, the guard stays; accepted, it leaves the map for everyone, and nothing is paid.
    // The pending offer to pass is part of the state hash (a fixed test for its kind's arm,
    // which otherwise only the seeded proptests reach: E08 T21b).
    let mut declined = state.clone();
    play(&mut declined, &rules, RED, DECLINE).unwrap();
    assert_ne!(declined.state_hash(), state.state_hash());
    assert!(
        declined
            .adventure()
            .observe(rules.core(), rules.adventure(), RED)
            .objects
            .iter()
            .any(|o| o.object_type == guard)
    );
    let events = play(&mut state, &rules, RED, ACCEPT).unwrap();
    assert_eq!(
        events,
        [
            GameEvent::OfferAnswered {
                player: RED,
                object_type: guard,
                anchor: Pos::new(3, 0, 0),
                accepted: true,
            },
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::ObjectRemoved {
                    object_type: guard,
                    anchor: Pos::new(3, 0, 0),
                },
                seen_by: Viewers::NONE.with(RED).with(BLUE),
            }),
        ]
    );
    assert_eq!(state.treasury(RED), Some(&[2000][..]));
    // Its tile is free: red's hero steps onto it.
    play(&mut state, &rules, RED, move_hero(0, &[Pos::new(3, 0, 0)])).unwrap();
}

// covers: GAM-DEC-001, ADV-VIS-004
#[test]
fn an_offer_another_player_s_acceptance_withdrew_may_only_be_declined() {
    let rules = rules();
    let guard = rules.adventure().object_type_by_key("t:guard").unwrap();
    let mut state = keys_game(&rules);
    state.set_schedule(&rules, vec![vec![RED, BLUE]]);
    // Both take the key and visit the guard in the same group's turn: two offers to pass it.
    play(&mut state, &rules, RED, move_hero(0, &[Pos::new(1, 0, 0)])).unwrap();
    let toward = [Pos::new(2, 1, 0), Pos::new(3, 0, 0)];
    play(&mut state, &rules, RED, move_hero(0, &toward)).unwrap();
    let tent = [Pos::new(1, 1, 0), Pos::new(1, 0, 0)];
    play(&mut state, &rules, BLUE, move_hero(1, &tent)).unwrap();
    let toward = [Pos::new(2, 0, 0), Pos::new(3, 0, 0)];
    play(&mut state, &rules, BLUE, move_hero(1, &toward)).unwrap();
    let blue = Actor::Player(BLUE);
    assert_eq!(legal_commands(&state, &rules, blue), [ACCEPT, DECLINE]);
    // Red passes first: the guard leaves the map, and blue's offer no longer stands.
    play(&mut state, &rules, RED, ACCEPT).unwrap();
    let offer = state.decision(BLUE).unwrap().clone();
    assert_eq!(
        (offer.object_type, offer.kind.clone()),
        (guard, OfferKind::Pass)
    );
    assert!(!state.adventure().offer_stands(&offer));
    assert_eq!(legal_commands(&state, &rules, blue), [DECLINE]);
    assert_eq!(
        play(&mut state, &rules, BLUE, ACCEPT),
        Err(GameCommandError::OfferGone)
    );
    assert_eq!(
        observe(&state, &rules, Viewer::Player(BLUE)).decision,
        Some(offer)
    );
    // Declining closes it; blue then walks over the guard's tile.
    let events = play(&mut state, &rules, BLUE, DECLINE).unwrap();
    assert_eq!(
        events,
        [GameEvent::OfferAnswered {
            player: BLUE,
            object_type: guard,
            anchor: Pos::new(3, 0, 0),
            accepted: false,
        }]
    );
    assert_eq!(legal_commands(&state, &rules, blue), [GameCommand::EndTurn]);
    play(&mut state, &rules, BLUE, move_hero(1, &[Pos::new(3, 0, 0)])).unwrap();
}
