//! Tests of what the game applies of events (E08 T62b2; ADR-0017 point 10): an event's record
//! through `MoveHero`, its firing told to its visitor's owner alone, a guarded event's battle at
//! once and its record after a win; and the hidden-event property (design §12): two games that
//! differ by an event give every player the same observation, legal commands, legal moves and
//! events until it fires.

use std::boxed::Box;

use h3_adventure::{
    AdventureEvent, AdventureEventKind, EventSetup, Guards, Holds, ObjectSetup, RewardRecord,
    TileMask,
};
use proptest::collection::vec;
use proptest::prelude::*;

use super::decisions::{GOLD, RED, move_hero, play};
use super::rewards::{HERO, gold, won};
use super::*;

const BLUE: PlayerId = PlayerId::new(1);

/// An event at `at` (its template blocking its one tile, as the shipped maps' do) giving 300
/// gold, firing for `colours` (and for computers with `computer`), leaving the map after it fires
/// with `remove`, guarded by two imps with `guarded`.
fn event_at(
    rules: &GameRuleset,
    at: Pos,
    (colours, computer, remove): (u8, bool, bool),
    guarded: bool,
) -> ObjectSetup {
    ObjectSetup {
        scenery: Default::default(),
        anchor: at,
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        object_type: rules.adventure().object_type_by_key("t:event"),
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: guarded.then(|| Guards {
            army: vec![ArmyStack {
                slot: 0,
                creature: IMP,
                count: 2,
            }],
            question: false,
        }),
        holds: Holds::Event(Box::new(EventSetup {
            record: RewardRecord {
                resources: vec![(GOLD, 300)],
                ..RewardRecord::default()
            },
            colours,
            computer,
            remove,
        })),
    }
}

/// [`setup`] with red's event at (1, 0), beside red's hero, guarded with `guarded`.
fn evented(rules: &GameRuleset, guarded: bool) -> GameState {
    let mut setup = setup();
    let at = Pos::new(1, 0, 0);
    setup.map.objects = vec![event_at(rules, at, (0b01, false, true), guarded)];
    GameState::new(rules, &setup, 7).unwrap()
}

// covers: ADV-OBJ-007, ADV-MOV-011, ADV-FOG-007, GAM-RES-001
#[test]
fn an_events_record_reaches_the_treasury_and_its_firing_its_owner_alone() {
    let rules = rules();
    let mut state = evented(&rules, false);
    let path = [Pos::new(1, 0, 0), Pos::new(2, 0, 0)];
    let events = play(&mut state, &rules, RED, move_hero(0, &path)).unwrap();
    assert!(events.contains(&GameEvent::ResourceGained {
        player: RED,
        resource: GOLD,
        amount: 300,
    }));
    assert_eq!(gold(&state), 2300);
    assert_eq!(state.adventure().hero(HERO).unwrap().pos, path[0]);
    assert!(state.adventure().is_removed(0));
    let fired = events
        .iter()
        .find(|e| {
            matches!(
                e,
                GameEvent::Adventure(AdventureEvent {
                    kind: AdventureEventKind::EventFired { .. },
                    ..
                })
            )
        })
        .unwrap();
    assert!(project(fired, Viewer::Player(RED)).is_some());
    assert_eq!(project(fired, Viewer::Player(BLUE)), None);
}

// covers: ADV-OBJ-007, ADV-PIK-004
#[test]
fn a_guarded_events_battle_opens_at_once_and_a_win_gives_its_record() {
    let rules = rules();
    let mut state = evented(&rules, true);
    let events = play(&mut state, &rules, RED, move_hero(0, &[Pos::new(1, 0, 0)])).unwrap();
    let started = |e: &GameEvent| matches!(e, GameEvent::BattleStarted { .. });
    assert!(events.iter().any(started));
    assert!(state.battle(RED).is_some() && state.decision(RED).is_none());
    // Won: the record, the event gone.
    let mut state = evented(&rules, true);
    let sides = ([Some(RED), None], [Some(HERO), None], Pos::new(1, 0, 0));
    let mut events = Vec::new();
    state.end_battle(&rules, sides, &won(), &mut events);
    assert_eq!(gold(&state), 2300);
    assert!(state.adventure().is_removed(0));
}

/// What `player` may know and do in `state`: its observation, its legal commands and each own
/// hero's reachable tiles with their paths.
type Seen = (
    GameObservation,
    Vec<GameCommand>,
    Vec<Vec<(Pos, u32, Option<Vec<Pos>>)>>,
);

fn seen(state: &GameState, rules: &GameRuleset, player: PlayerId) -> Seen {
    let actor = Actor::Player(player);
    let moves = state
        .adventure()
        .heroes()
        .iter()
        .filter(|h| h.owner == player)
        .filter_map(|h| legal_moves(state, rules, actor, h.id))
        .map(|reach| {
            reach
                .tiles()
                .map(|(pos, left)| (pos, left, reach.path(pos)))
                .collect()
        })
        .collect();
    (
        observe(state, rules, Viewer::Player(player)),
        legal_commands(state, rules, actor),
        moves,
    )
}

/// The commands the acting player may give in `state` (with `seen`'s moves), and that player.
fn options(state: &GameState, rules: &GameRuleset) -> Option<(PlayerId, Vec<GameCommand>)> {
    let player = [RED, BLUE]
        .into_iter()
        .find(|&p| !legal_commands(state, rules, Actor::Player(p)).is_empty())?;
    let actor = Actor::Player(player);
    let mut commands = legal_commands(state, rules, actor);
    for hero in state
        .adventure()
        .heroes()
        .iter()
        .filter(|h| h.owner == player)
    {
        if let Some(reach) = legal_moves(state, rules, actor, hero.id) {
            for (pos, _) in reach.tiles() {
                if let Some(path) = reach.path(pos).filter(|p| !p.is_empty()) {
                    commands.push(GameCommand::Adventure(
                        h3_adventure::AdventureCommand::MoveHero {
                            hero: hero.id,
                            path,
                        },
                    ));
                }
            }
        }
    }
    Some((player, commands))
}

/// Whether `events` tell of an event's firing.
fn fires(events: &[GameEvent]) -> bool {
    events.iter().any(|e| {
        matches!(
            e,
            GameEvent::Adventure(AdventureEvent {
                kind: AdventureEventKind::EventFired { .. },
                ..
            })
        )
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// An event anywhere a hero may stand, of any colours and flags, guarded or not, for a blue
    /// that counts as human or not: until it fires, every player's observation, legal commands,
    /// legal moves and told events are those of the game without it (design §12; ADV-FOG-007,
    /// ADV-MOV-011).
    // covers: ADV-FOG-007, ADV-MOV-011, ADV-OBJ-007
    #[test]
    fn a_hidden_event_changes_nothing_a_player_sees_until_it_fires(
        tile in 0u16..15,
        flags in (any::<u8>(), any::<bool>(), any::<bool>()),
        guarded in any::<bool>(),
        human in any::<bool>(),
        seed in 0u64..64,
        choices in vec(any::<u16>(), 1..48),
    ) {
        let rules = rules();
        let at = Pos::new(tile % 4, tile / 4, 0);
        prop_assume!(at != Pos::new(0, 0, 0) && at != Pos::new(2, 2, 0));
        let mut setup = setup();
        setup.players[1].human = human;
        let mut plain = GameState::new(&rules, &setup, seed).unwrap();
        setup.map.objects.push(event_at(&rules, at, flags, guarded));
        let mut hidden = GameState::new(&rules, &setup, seed).unwrap();
        for choice in choices {
            for player in [RED, BLUE] {
                prop_assert_eq!(seen(&plain, &rules, player), seen(&hidden, &rules, player));
            }
            let Some((player, commands)) = options(&plain, &rules) else {
                break;
            };
            let command = commands[usize::from(choice) % commands.len()].clone();
            let mut told = (Vec::new(), Vec::new());
            let actor = Actor::Player(player);
            let a = apply(&mut plain, &rules, actor, command.clone(), &mut told.0);
            let b = apply(&mut hidden, &rules, actor, command, &mut told.1);
            if fires(&told.1) {
                break;
            }
            prop_assert_eq!(a, b);
            for viewer in [RED, BLUE].map(Viewer::Player) {
                let projected = |events: &[GameEvent]| -> Vec<GameEvent> {
                    events.iter().filter_map(|e| project(e, viewer)).collect()
                };
                prop_assert_eq!(projected(&told.0), projected(&told.1));
            }
        }
    }
}
