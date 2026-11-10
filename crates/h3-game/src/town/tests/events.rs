//! Tests of the town events at a player's turn start (E09 T18c, the town epic's design note §30;
//! TWN-EVT-001 to -004): when an event fires (its town's owner, the players it names, its kind, its
//! day), what it gives (the gains, the losses down to 0), what it builds (no hall's test, no mark,
//! the Shipyard's water), what it tells and to whom, and day 1's inside the build.

use alloc::vec;
use alloc::vec::Vec;

use h3_content::ResourceId;
use h3_contracts::{PlayerId, TownId};
use h3_towns::{BuildingsSetup, SlotSet, TownEventSetup, TownFaction};

use super::command::{BLUE, CAPITOL, FORT, RED, SILO, T0, T1, build, building, priced, run, setup};
use super::{CAPITOL_SLOT, GRAIL_SLOT, SHIPYARD_SLOT, SILO_SLOT, faction, town_object};
use crate::{
    Actor, GameCommand, GameEvent, GameRuleset, GameSetup, GameState, TimedEventSetup, Viewer,
    apply, project,
};

const GOLD: ResourceId = ResourceId::from_raw(0);
/// The test pack's Grail building (`b9`).
const GRAIL: usize = 9;
/// A third town, red's or neutral ([`with_third`]).
const T2: TownId = TownId::new(2);

/// An event for `players` (human ones) on `first_day`, every `interval` days after it, giving
/// `gold` (negative: taking) and building the buildings of `slots`.
fn event(
    players: &[PlayerId],
    first_day: u16,
    interval: u16,
    gold: i32,
    slots: &[u8],
) -> TownEventSetup {
    TownEventSetup {
        players: players.to_vec(),
        human: true,
        computer: false,
        first_day,
        interval,
        resources: if gold == 0 {
            vec![]
        } else {
            vec![(GOLD, gold)]
        },
        buildings: slots
            .iter()
            .fold(SlotSet::EMPTY, |set, &slot| set.with(slot)),
        creatures: [0; 7],
    }
}

/// [`setup`] (red's town 0 with its Village Hall alone, blue's town 1) with `events` on red's town.
fn with_events(rules: &GameRuleset, events: Vec<TownEventSetup>) -> GameSetup {
    let mut setup = setup(rules);
    setup.towns[0].events = events;
    setup
}

/// `setup` with a third `t:town` town on (1, 3), owned by `owner` at the start, with `events`.
fn with_third(
    rules: &GameRuleset,
    mut setup: GameSetup,
    owner: Option<u8>,
    events: Vec<TownEventSetup>,
) -> GameSetup {
    let mut town = super::town(TownFaction::Fixed(faction(rules, "t:town")));
    town.events = events;
    setup.towns.push(town);
    setup.map.objects.push(town_object(rules, (1, 3), owner));
    setup
}

/// `player` ends its turn: the events, the next turn start's among them.
fn end(rules: &GameRuleset, game: &mut GameState, player: PlayerId) -> Vec<GameEvent> {
    let mut events = Vec::new();
    apply(
        game,
        rules,
        Actor::Player(player),
        GameCommand::EndTurn,
        &mut events,
    )
    .unwrap();
    events
}

/// Ends the day's turns until red's turn of `day` starts; the events of each end.
fn to_day(rules: &GameRuleset, game: &mut GameState, day: u32) -> Vec<GameEvent> {
    let mut events = Vec::new();
    while game.day() < day {
        events.extend(end(rules, game, RED));
        events.extend(end(rules, game, BLUE));
    }
    events
}

fn fired(events: &[GameEvent]) -> Vec<(PlayerId, TownId, u16)> {
    events
        .iter()
        .filter_map(|e| match *e {
            GameEvent::TownEventFired {
                player,
                town,
                event,
            } => Some((player, town, event)),
            _ => None,
        })
        .collect()
}

fn gold(game: &GameState, player: PlayerId) -> u32 {
    game.treasury(player).unwrap()[usize::from(GOLD.raw())]
}

// covers: TWN-EVT-001, TWN-EVT-002, GAM-CAL-004
#[test]
fn a_town_event_fires_at_its_owner_s_turn_start_after_the_map_s_events() {
    let rules = priced();
    let mut setup = with_events(&rules, vec![event(&[RED], 2, 0, 500, &[])]);
    setup.events = vec![TimedEventSetup {
        players: vec![RED],
        human: true,
        computer: false,
        first_day: 2,
        interval: 0,
        resources: vec![(GOLD, 100)],
        message: 0,
    }];
    let mut game = GameState::new(&rules, &setup, 7).unwrap();
    end(&rules, &mut game, RED);
    let before = gold(&game, RED);
    let events = end(&rules, &mut game, BLUE);
    let start = events
        .iter()
        .position(|e| *e == GameEvent::TurnStarted { player: RED })
        .unwrap();
    let told = GameEvent::TownEventFired {
        player: RED,
        town: T0,
        event: 0,
    };
    let gained = |amount| GameEvent::ResourceGained {
        player: RED,
        resource: GOLD,
        amount,
    };
    // The contexts' steps, `TurnStarted`, the map's events, then the town's: its gains first.
    assert_eq!(
        events[start..],
        [
            GameEvent::TurnStarted { player: RED },
            gained(100),
            GameEvent::TimedEventFired {
                player: RED,
                event: 0
            },
            gained(500),
            told,
        ]
    );
    // The day's income (the hall's) came before the turn start.
    let income: u32 = events[..start]
        .iter()
        .filter_map(|e| match *e {
            GameEvent::ResourceGained {
                player: RED,
                amount,
                ..
            } => Some(amount),
            _ => None,
        })
        .sum();
    assert_eq!(gold(&game, RED), before + income + 100 + 500);
    // Its owner's alone.
    assert_eq!(project(&told, Viewer::Player(RED)), Some(told));
    assert_eq!(project(&told, Viewer::Player(BLUE)), None);
    // It fired once: day 3 tells none.
    assert!(fired(&to_day(&rules, &mut game, 3)).is_empty());
}

// covers: TWN-EVT-001
#[test]
fn a_town_event_fires_only_for_its_town_s_owner_named_and_of_its_kind() {
    let rules = priced();
    let mut computer = event(&[RED], 2, 1, 1, &[]);
    (computer.human, computer.computer) = (false, true);
    let setup = with_events(
        &rules,
        vec![
            // Blue's events in red's town: blue never owns it.
            event(&[BLUE], 2, 1, 1, &[]),
            // A computer's: red is human.
            computer,
            // Red's, every second day from day 2: days 2 and 4.
            event(&[RED, BLUE], 2, 2, 1, &[]),
        ],
    );
    // A neutral town's never fire, whoever they name.
    let setup = with_third(&rules, setup, None, vec![event(&[RED, BLUE], 2, 1, 1, &[])]);
    let mut game = GameState::new(&rules, &setup, 7).unwrap();
    let mut seen = Vec::new();
    for day in 2..=5 {
        seen.extend(
            fired(&to_day(&rules, &mut game, day))
                .into_iter()
                .map(|f| (day, f)),
        );
    }
    assert_eq!(seen, [(2, (RED, T0, 2)), (4, (RED, T0, 2))]);
    // The town's owner is the adventure's: a town that changes hands fires for its new owner
    // when the event names it (blue's town 1 given to red here, as a capture would).
    let mut setup = with_events(&rules, vec![]);
    setup.towns[1].events = vec![event(&[RED], 2, 0, 1, &[])];
    setup.map.objects[1].owner = Some(RED);
    let mut game = GameState::new(&rules, &setup, 7).unwrap();
    assert_eq!(fired(&to_day(&rules, &mut game, 2)), [(RED, T1, 0)]);
}

// covers: TWN-EVT-002
#[test]
fn a_town_event_s_losses_stop_at_zero() {
    let rules = priced();
    let setup = with_events(&rules, vec![event(&[RED], 2, 0, -5000, &[])]);
    let mut game = GameState::new(&rules, &setup, 7).unwrap();
    let events = to_day(&rules, &mut game, 2);
    // All red held after the day's income, less than the event's 5 000.
    let at = events
        .iter()
        .position(|e| matches!(e, GameEvent::ResourceLost { player: RED, .. }))
        .unwrap();
    let amount = match events[at] {
        GameEvent::ResourceLost { amount, .. } => amount,
        _ => 0,
    };
    assert!(amount > 0 && amount < 5000, "{amount}");
    assert_eq!(
        events[at + 1],
        GameEvent::TownEventFired {
            player: RED,
            town: T0,
            event: 0
        }
    );
    assert_eq!(gold(&game, RED), 0);
}

// covers: TWN-EVT-003, TWN-EVT-004
#[test]
fn a_town_event_builds_without_the_hall_s_tests_or_the_mark() {
    let rules = priced();
    let mut setup = with_events(
        &rules,
        vec![
            // The creatures alone: it fires, nothing else changes (`TWN-STUB-10`).
            TownEventSetup {
                creatures: [3, 0, 0, 0, 0, 0, 0],
                ..event(&[RED], 2, 0, 0, &[])
            },
            // The Capitol, the Grail's building, the silo forbidden in this town, the Shipyard
            // of a town without water.
            event(
                &[RED],
                2,
                0,
                0,
                &[CAPITOL_SLOT, GRAIL_SLOT, SILO_SLOT, SHIPYARD_SLOT],
            ),
        ],
    );
    setup.towns[0].buildings = BuildingsSetup::Custom {
        built: SlotSet::EMPTY,
        forbidden: SlotSet::EMPTY.with(SILO_SLOT),
    };
    let mut game = GameState::new(&rules, &setup, 7).unwrap();
    let before = game.towns().get(T0).unwrap().clone();
    assert!(before.forbidden().contains(building(SILO)));
    let events = to_day(&rules, &mut game, 2);
    let start = events
        .iter()
        .position(|e| *e == GameEvent::TurnStarted { player: RED })
        .unwrap();
    let ours: Vec<GameEvent> = events[start + 1..]
        .iter()
        .copied()
        .filter(|e| {
            matches!(
                e,
                GameEvent::TownEventFired { .. } | GameEvent::TownBuilt { .. }
            )
        })
        .collect();
    let built = |index| GameEvent::TownBuilt {
        player: RED,
        town: T0,
        building: building(index),
        event: Some(1),
    };
    assert_eq!(
        ours,
        [
            GameEvent::TownEventFired {
                player: RED,
                town: T0,
                event: 0
            },
            GameEvent::TownEventFired {
                player: RED,
                town: T0,
                event: 1
            },
            built(CAPITOL),
            built(GRAIL),
            built(SILO),
        ]
    );
    // Each build tells the views it changed after its `TownBuilt`, its owner's first.
    let after_first = events.iter().position(|e| *e == built(CAPITOL)).unwrap();
    assert!(matches!(
        events[after_first + 1],
        GameEvent::TownChanged {
            player: RED,
            town: T0,
            ..
        }
    ));
    let town = game.towns().get(T0).unwrap();
    for index in [CAPITOL, GRAIL, SILO] {
        assert!(town.built().contains(building(index)), "{index}");
    }
    assert!(!town.built().contains(building(super::SHIPYARD)));
    // No mark: the day's build is still free.
    assert!(!town.built_today());
    run(&rules, &mut game, RED, build(T0, FORT)).unwrap();
}

// covers: TWN-EVT-003
#[test]
fn a_building_two_events_name_the_same_day_is_built_once() {
    let rules = priced();
    let setup = with_events(
        &rules,
        vec![
            event(&[RED], 2, 0, 0, &[SILO_SLOT]),
            event(&[RED], 2, 0, 0, &[SILO_SLOT, CAPITOL_SLOT]),
        ],
    );
    let mut game = GameState::new(&rules, &setup, 7).unwrap();
    let built: Vec<(usize, Option<u16>)> = to_day(&rules, &mut game, 2)
        .iter()
        .filter_map(|e| match *e {
            GameEvent::TownBuilt {
                building, event, ..
            } => Some((building.index(), event)),
            _ => None,
        })
        .collect();
    // The second event's silo stands by its turn: only its Capitol is built and told.
    assert_eq!(built, [(SILO, Some(0)), (CAPITOL, Some(1))]);
}

// covers: TWN-EVT-003
#[test]
fn an_event_s_capitol_stands_beside_its_owner_s_other_one() {
    let rules = priced();
    let mut setup = with_events(&rules, vec![]);
    setup.towns[0].buildings = BuildingsSetup::Custom {
        built: SlotSet::EMPTY.with(CAPITOL_SLOT),
        forbidden: SlotSet::EMPTY,
    };
    // T-8: red's third town builds a second Capitol.
    let setup = with_third(
        &rules,
        setup,
        Some(0),
        vec![event(&[RED], 2, 0, 0, &[CAPITOL_SLOT])],
    );
    let mut game = GameState::new(&rules, &setup, 7).unwrap();
    to_day(&rules, &mut game, 2);
    for town in [T0, T2] {
        assert!(
            game.towns()
                .get(town)
                .unwrap()
                .built()
                .contains(building(CAPITOL)),
            "{town:?}"
        );
    }
}

// covers: TWN-EVT-001, TWN-EVT-002, TWN-EVT-003
#[test]
fn day_1_s_town_events_fire_inside_the_build() {
    let rules = priced();
    let mut setup = with_events(&rules, vec![event(&[RED], 1, 0, 1000, &[SILO_SLOT])]);
    setup.towns[1].events = vec![event(&[BLUE], 1, 0, 300, &[])];
    let mut game = GameState::new(&rules, &setup, 7).unwrap();
    // Red's turn started inside the build: its town's event gave and built there.
    assert_eq!(gold(&game, RED), 3000);
    assert!(
        game.towns()
            .get(T0)
            .unwrap()
            .built()
            .contains(building(SILO))
    );
    assert_eq!(game.town_events_at_start(RED), [(T0, 0)]);
    assert!(game.town_events_at_start(BLUE).is_empty());
    assert!(game.town_events_at_start(PlayerId::new(9)).is_empty());
    // Blue's turn starts with a sink: told, not kept.
    let events = end(&rules, &mut game, RED);
    assert_eq!(fired(&events), [(BLUE, T1, 0)]);
    assert_eq!(gold(&game, BLUE), 2300);
    assert!(game.town_events_at_start(BLUE).is_empty());
    // A fork keeps them; a game whose events fire later has none at the start.
    assert_eq!(game.clone().town_events_at_start(RED), [(T0, 0)]);
    let later = GameState::new(
        &rules,
        &with_events(&rules, vec![event(&[RED], 2, 0, 1, &[])]),
        7,
    )
    .unwrap();
    assert!(later.town_events_at_start(RED).is_empty());
}
