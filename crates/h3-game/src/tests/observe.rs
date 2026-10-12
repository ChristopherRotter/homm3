//! Tests of the game's observation and the events' projection (E08 T8b1): the public facts, the
//! adventure map as the viewer sees it, nothing of the seed, and which events a viewer is told.

use h3_adventure::{AdventureEvent, AdventureEventKind, Viewers};

use super::*;

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);

#[test]
fn a_player_observes_the_public_facts_and_the_map_as_it_sees_it() {
    let rules = rules();
    let mut state = GameState::new(&rules, &setup(), 7).unwrap();
    let seen = observe(&state, &rules, Viewer::Player(BLUE));
    assert_eq!(seen.viewer, BLUE);
    assert_eq!(seen.day, 1);
    assert_eq!(
        seen.players,
        [
            PlayerView {
                colour: 0,
                acting: true,
            },
            PlayerView {
                colour: 1,
                acting: false,
            },
        ]
    );
    assert_eq!(seen.turn_group, [RED]);
    // Its own treasury (GAM-RES-001), never another player's: the observation has no field for
    // one.
    assert_eq!(seen.treasury, [2000]);
    assert_eq!(
        seen.adventure,
        state.adventure().observe(rules.core(), BLUE)
    );
    // After red's turn: blue acts.
    apply(
        &mut state,
        &rules,
        Actor::Player(RED),
        GameCommand::EndTurn,
        &mut NoopSink,
    )
    .unwrap();
    let seen = observe(&state, &rules, Viewer::Player(RED));
    assert_eq!(seen.turn_group, [BLUE]);
    assert_eq!(
        seen.players.iter().map(|p| p.acting).collect::<Vec<_>>(),
        [false, true]
    );
    // A viewer that is not a player sees the public facts and no tile.
    let stranger = observe(&state, &rules, Viewer::Player(PlayerId::new(5)));
    assert_eq!((stranger.day, stranger.players.len()), (1, 2));
    assert!(stranger.adventure.heroes.is_empty());
    assert!(stranger.treasury.is_empty());
}

#[test]
fn an_observation_does_not_depend_on_the_seed_where_the_viewer_cannot_see_it() {
    // The seed draws blue's starting army, which red never sees (nor blue's points, which the
    // army sets); blue sees its own.
    let rules = rules();
    let red = |seed| {
        observe(
            &GameState::new(&rules, &setup(), seed).unwrap(),
            &rules,
            Viewer::Player(RED),
        )
    };
    let blue = |seed| {
        observe(
            &GameState::new(&rules, &setup(), seed).unwrap(),
            &rules,
            Viewer::Player(BLUE),
        )
    };
    assert!((1..40).all(|seed| red(seed) == red(0)));
    assert!((1..40).any(|seed| blue(seed) != blue(0)));
}

#[test]
fn the_games_events_are_public_and_a_step_shows_to_its_viewers() {
    let public = [
        GameEvent::TurnEnded { player: RED },
        GameEvent::DayStarted { day: 2 },
        GameEvent::TurnStarted { player: BLUE },
    ];
    for event in public {
        assert_eq!(project(&event, Viewer::Player(BLUE)), Some(event));
    }
    // Red sees the hero on both tiles, blue on the tile it left only.
    let red = Viewers::NONE.with(RED);
    let both = red.with(BLUE);
    let step = AdventureEvent {
        kind: AdventureEventKind::HeroStepped {
            hero: HeroId::new(0),
            from: Some(Pos::new(0, 0, 0)),
            to: Some(Pos::new(1, 0, 0)),
            seen_from: both,
            seen_to: red,
        },
        seen_by: both,
    };
    assert_eq!(
        project(&GameEvent::Adventure(step), Viewer::Player(BLUE)),
        Some(GameEvent::Adventure(AdventureEvent {
            kind: AdventureEventKind::HeroStepped {
                hero: HeroId::new(0),
                from: Some(Pos::new(0, 0, 0)),
                to: None,
                seen_from: Viewers::NONE.with(BLUE),
                seen_to: Viewers::NONE,
            },
            seen_by: Viewers::NONE.with(BLUE),
        }))
    );
    let red_only = AdventureEvent {
        kind: AdventureEventKind::HeroStepped {
            hero: HeroId::new(0),
            from: Some(Pos::new(0, 0, 0)),
            to: Some(Pos::new(1, 0, 0)),
            seen_from: red,
            seen_to: red,
        },
        seen_by: red,
    };
    assert_eq!(
        project(&GameEvent::Adventure(red_only), Viewer::Player(BLUE)),
        None
    );
}
