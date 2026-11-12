//! The whole game's queries (E16 M5 T11): a hero's moves on the wire are its reach (every tile's
//! points and path, over every move of the golden replays, flying and water-walking heroes' too),
//! a client's reads of the tree are the engine's and refuse a malformed one; a town's hall is
//! its owner's alone; a battle's targets are the battle's own query's for the side the player
//! owns. Over the connections: the answer takes no number and leaves the record untouched, a
//! query outside a running game is rejected as a command; the game's start tells each player the
//! town events that fired at its turn start inside the build, once.

use std::vec;
use std::vec::Vec;

use h3_adventure::{AdventureCommand as EngineAdventure, Reach};
use h3_contracts::PlayerId;
use h3_game::{Actor, GameCommand as EngineCommand, GameState, NoopSink, apply, legal_moves};

use super::game_fold::{golden, one};
use super::game_seats::{MAP, OPTIONS, end_turn, join, send, to, token};
use super::game_session::duel;
use super::wire::moves as tree;
use crate::map;
use crate::map::game::{answer, events, manifest, moves, pos};
use crate::session::ConnId;
use crate::session::game::{GameOutbound, GameSession};
use crate::wire::game::{GameAnswer, GameEvent, GameQuery, GameReject, Moves, Pos, TownOffer};
use crate::wire::{ClientMessage, GoodbyeReason, Query, ServerMessage, WireU64};

/// A tile of the test tree's level.
const fn at(x: u16, y: u16) -> Pos {
    Pos { x, y, level: 0 }
}

/// A tile's sort key: row by row.
const fn key(p: &Pos) -> (u8, u16, u16) {
    (p.level, p.y, p.x)
}

/// What the golden replays' moves held.
#[derive(Debug, Default)]
struct Seen {
    reaches: usize,
    tiles: usize,
    /// Reaches whose tree has a node no path ends on: a flying or water-walking hero's pass.
    passing: usize,
}

/// Checks the wire's `tree` against `reach`: its tile and points, every tile's points and path
/// (each node's path built once), and a client's reads on a sample of them.
fn same(reach: &Reach, tree: &Moves, name: &str, seen: &mut Seen) {
    let origin = pos(reach.origin());
    assert_eq!(tree.origin, origin, "{name}");
    assert_eq!(
        Some(tree.points),
        reach.points_left(reach.origin()),
        "{name}"
    );
    let nodes = tree.nodes().unwrap();
    let mut paths: Vec<Vec<Pos>> = vec![Vec::new()];
    for (k, &step) in tree.steps.iter().enumerate() {
        let mut path = paths[usize::try_from(step >> 3).unwrap()].clone();
        path.push(nodes[k + 1]);
        paths.push(path);
    }
    let mut wire: Vec<(Pos, u32, Vec<Pos>)> = (1..nodes.len())
        .filter(|&k| tree.left[k - 1] > 0)
        .map(|k| (nodes[k], tree.left[k - 1] - 1, paths[k].clone()))
        .collect();
    let mut engine: Vec<(Pos, u32, Vec<Pos>)> = reach
        .tiles()
        .map(|(tile, points)| {
            let path = reach.path(tile).unwrap().into_iter().map(pos).collect();
            (pos(tile), points, path)
        })
        .collect();
    wire.sort_by_key(|(tile, _, _)| key(tile));
    engine.sort_by_key(|(tile, _, _)| key(tile));
    assert_eq!(wire, engine, "{name}");
    let mut listed = tree.tiles();
    listed.sort_by_key(|(tile, _)| key(tile));
    let points: Vec<(Pos, u32)> = engine.iter().map(|(t, p, _)| (*t, *p)).collect();
    assert_eq!(listed, points, "{name}");
    for (tile, points, path) in engine.iter().step_by(23) {
        assert_eq!(tree.points_left(*tile), Some(*points), "{name}");
        assert_eq!(tree.path(*tile).as_ref(), Some(path), "{name}");
    }
    assert_eq!(tree.path(origin), Some(Vec::new()));
    seen.reaches += 1;
    seen.tiles += engine.len();
    seen.passing += usize::from(nodes.len() > engine.len() + 1);
}

#[test]
fn a_heros_moves_on_the_wire_are_its_reach_before_every_move_of_the_golden_replays() {
    let mut seen = Seen::default();
    golden(|name, rules, setup, replay| {
        let mut state = GameState::new(rules, setup, replay.seed).unwrap();
        for (actor, command) in &replay.commands {
            if let EngineCommand::Adventure(EngineAdventure::MoveHero { hero, .. }) = command {
                let reach = legal_moves(&state, rules, *actor, *hero).unwrap();
                let tree = moves(&reach);
                same(&reach, &tree, name, &mut seen);
                // The player's question is answered with the tree.
                let Actor::Player(player) = *actor else {
                    panic!("{name}: the system moves no hero");
                };
                let query = GameQuery::Moves { hero: hero.raw() };
                let expected = GameAnswer::Moves {
                    hero: hero.raw(),
                    moves: Some(tree),
                };
                assert_eq!(answer(&state, rules, player, query), expected);
            }
            apply(&mut state, rules, *actor, command.clone(), &mut NoopSink).unwrap();
        }
    });
    assert!(seen.reaches > 1_000, "{seen:?}");
    assert!(seen.tiles > 100_000, "{seen:?}");
    assert!(seen.passing > 0, "{seen:?}");
}

#[test]
fn a_client_reads_the_tree_and_refuses_a_malformed_one() {
    let tree = tree();
    assert_eq!(
        tree.nodes(),
        Some(vec![at(5, 5), at(6, 5), at(7, 6), at(5, 4), at(6, 4)])
    );
    assert_eq!(
        tree.tiles(),
        [(at(6, 5), 1400), (at(7, 6), 1200), (at(6, 4), 1300)]
    );
    assert_eq!(tree.points_left(at(5, 5)), Some(1500));
    assert_eq!(tree.points_left(at(7, 6)), Some(1200));
    // A tile the path passes, and one off the tree, are not reachable.
    assert_eq!(tree.points_left(at(5, 4)), None);
    assert_eq!(tree.points_left(at(9, 9)), None);
    assert_eq!(tree.path(at(5, 5)), Some(Vec::new()));
    assert_eq!(tree.path(at(7, 6)), Some(vec![at(6, 5), at(7, 6)]));
    assert_eq!(tree.path(at(6, 4)), Some(vec![at(5, 4), at(6, 4)]));
    assert_eq!(tree.path(at(5, 4)), None);
    assert_eq!(tree.path(at(9, 9)), None);
    // Malformed: the lists' lengths differ, a parent not earlier, a step off the map's range in
    // either direction.
    let short = Moves {
        left: vec![1],
        ..tree.clone()
    };
    let later = Moves {
        steps: vec![2, 2 * 8],
        left: vec![1, 1],
        ..tree.clone()
    };
    let west = Moves {
        origin: at(0, 5),
        steps: vec![6],
        left: vec![1],
        ..tree.clone()
    };
    let north = Moves {
        origin: at(5, 0),
        steps: vec![0],
        left: vec![1],
        ..tree.clone()
    };
    for malformed in [short, later, west, north] {
        assert_eq!(malformed.nodes(), None, "{malformed:?}");
        assert!(malformed.tiles().is_empty(), "{malformed:?}");
        assert_eq!(malformed.path(at(6, 5)), None, "{malformed:?}");
        assert_eq!(malformed.points_left(at(6, 5)), None, "{malformed:?}");
        // The hero's own tile is its own.
        let origin = malformed.origin;
        assert_eq!(malformed.points_left(origin), Some(1500), "{malformed:?}");
    }
}

/// The rules, the setup and the seed of the golden replay `builds-events-s1`: red's town events
/// fire on day 1, inside the build.
fn builds() -> (h3_game::GameRuleset, h3_game::GameSetup, u64) {
    let (rules, setup, replay) = one("builds-events-s1");
    (rules, setup, replay.seed)
}

#[test]
fn a_towns_hall_is_its_owners_alone() {
    let (rules, setup, seed) = builds();
    let mut out = Vec::new();
    let session = GameSession::start(
        &rules,
        &setup,
        seed,
        manifest(&rules, MAP),
        OPTIONS,
        &mut out,
    )
    .unwrap();
    let state = session.state();
    let towns = state.adventure().map().towns().len();
    assert!(towns > 1);
    let mut owned = 0;
    for player in [PlayerId::new(0), PlayerId::new(1)] {
        for town in (0..=u16::MAX).take(towns + 1) {
            let query = GameQuery::TownOffer { town };
            let offer = state.town_offer(&rules, player, h3_contracts::TownId::new(town));
            let expected = offer.map(|o| TownOffer {
                possible: WireU64(o.possible.bits()),
                buildable: WireU64(o.buildable.bits()),
                payable: WireU64(o.payable.bits()),
            });
            owned += usize::from(expected.is_some());
            assert_eq!(
                session.query(player, query),
                Ok(GameAnswer::TownOffer {
                    town,
                    offer: expected
                })
            );
        }
    }
    // Each player's own towns; another's and one beyond the map's are none of its.
    assert!(owned >= 2, "{owned}");
    let outside = PlayerId::new(9);
    assert_eq!(
        session.query(outside, GameQuery::TownOffer { town: 0 }),
        Ok(GameAnswer::TownOffer {
            town: 0,
            offer: None
        })
    );
    assert_eq!(
        session.query(outside, GameQuery::Moves { hero: 0 }),
        Ok(GameAnswer::Moves {
            hero: 0,
            moves: None
        })
    );
}

#[test]
fn a_battles_targets_are_its_own_querys_for_the_players_side() {
    let (mut checked, mut aimed) = (0, 0);
    golden(|_, rules, setup, replay| {
        if checked > 200 {
            return;
        }
        let mut state = GameState::new(rules, setup, replay.seed).unwrap();
        for (actor, command) in &replay.commands {
            for player in (0..=u8::MAX).take(state.players()).map(PlayerId::new) {
                let queries = [Query::CreatureCastTargets {}].into_iter();
                let Some((battle, side)) = state
                    .battle(player)
                    .and_then(|battle| battle.side_of(player).map(|side| (battle, side)))
                else {
                    let query = GameQuery::Battle {
                        query: Query::CastTargets { spell: 0 },
                    };
                    let none = GameAnswer::Battle { targets: vec![] };
                    assert_eq!(answer(&state, rules, player, query), none);
                    continue;
                };
                let spells = map::castable(battle.state(), rules.battle(), side);
                let casts = spells.iter().map(|&spell| Query::CastTargets { spell });
                for query in queries.chain(casts) {
                    let targets = map::query(battle.state(), rules.battle(), side, query);
                    aimed += usize::from(!targets.is_empty());
                    let expected = GameAnswer::Battle { targets };
                    let asked = GameQuery::Battle { query };
                    assert_eq!(answer(&state, rules, player, asked), expected);
                    checked += 1;
                }
            }
            apply(&mut state, rules, *actor, command.clone(), &mut NoopSink).unwrap();
        }
    });
    assert!(checked > 200, "{checked}");
    assert!(aimed > 0, "{aimed}");
}

/// Connections of a test.
const A: ConnId = ConnId(1);
const B: ConnId = ConnId(2);
const C: ConnId = ConnId(3);

/// A numbered message's number.
fn number(message: &ServerMessage) -> u32 {
    match message {
        ServerMessage::GameSnapshot { n, .. }
        | ServerMessage::GameEvents { n, .. }
        | ServerMessage::GameTurn { n, .. }
        | ServerMessage::GameRejected { n, .. } => *n,
        _ => 0,
    }
}

/// A query numbered `seq`.
const fn ask(seq: u32, query: GameQuery) -> ClientMessage {
    ClientMessage::GameQuery { seq, query }
}

#[test]
fn a_query_takes_no_number_and_is_rejected_outside_a_running_game() {
    let (rules, setup) = duel();
    let mut s = GameSession::new(rules, &setup, 1, manifest(rules, MAP), OPTIONS, token).unwrap();
    let red = PlayerId::new(0);
    let hero = (s.state().adventure().heroes().iter())
        .find(|h| h.owner == red)
        .unwrap()
        .id
        .raw();
    let moves_of = GameQuery::Moves { hero };
    // Without a seat: rejected, numbered 0.
    let out = send(&mut s, C, ask(1, moves_of));
    assert_eq!(
        to(&out, C),
        [ServerMessage::GameRejected {
            n: 0,
            seq: 1,
            code: GameReject::NotSeated {},
        }]
    );
    // Before the start: rejected as a command, numbered and recorded.
    send(&mut s, A, join(0, None));
    let out = send(&mut s, A, ask(2, moves_of));
    assert_eq!(
        to(&out, A),
        [ServerMessage::GameRejected {
            n: 2,
            seq: 2,
            code: GameReject::NotStarted {},
        }]
    );
    assert_eq!(s.query(red, moves_of), Err(GameReject::NotStarted {}));
    // Once started: answered with the number of the player's last message, unrecorded.
    let out = send(&mut s, B, join(1, None));
    let last = to(&out, A).iter().map(number).max().unwrap_or(2);
    let record = s.record(red);
    let out = send(&mut s, A, ask(3, moves_of));
    let expected = answer(s.state(), rules, red, moves_of);
    assert!(matches!(
        &expected,
        GameAnswer::Moves { moves: Some(m), .. } if !m.steps.is_empty()
    ));
    assert_eq!(
        to(&out, A),
        [ServerMessage::GameAnswer {
            n: last,
            seq: 3,
            answer: expected.clone(),
        }]
    );
    assert!(to(&out, B).is_empty());
    assert_eq!(s.record(red), record);
    assert_eq!(s.query(red, moves_of), Ok(expected));
    // Blue's question about red's hero is none of its.
    let blue = s.record(PlayerId::new(1)).unwrap();
    let out = send(&mut s, B, ask(1, moves_of));
    assert_eq!(
        to(&out, B),
        [ServerMessage::GameAnswer {
            n: u32::try_from(blue.lines().count() - 1).unwrap(),
            seq: 1,
            answer: GameAnswer::Moves { hero, moves: None },
        }]
    );
    // Its turn not running, red's hero has no moves after its end.
    send(&mut s, A, end_turn(4));
    assert_eq!(
        s.query(red, moves_of),
        Ok(GameAnswer::Moves { hero, moves: None })
    );
    // Paused: the host's pause drops the holders; the game rejects every query.
    let out = send(&mut s, A, ClientMessage::Pause { seq: 5 });
    assert!(to(&out, A).contains(&ServerMessage::Goodbye {
        reason: GoodbyeReason::Paused,
    }));
    assert!(s.paused());
    assert_eq!(s.query(red, moves_of), Err(GameReject::NotStarted {}));
}

#[test]
fn the_start_tells_each_player_its_town_events_of_the_build_once() {
    let (rules, setup, seed) = builds();
    let red = PlayerId::new(0);
    let fired: Vec<h3_game::GameEvent> = {
        let state = GameState::new(&rules, &setup, seed).unwrap();
        assert!(!state.town_events_at_start(red).is_empty());
        assert!(state.town_events_at_start(PlayerId::new(1)).is_empty());
        (state.town_events_at_start(red).into_iter())
            .map(|(town, event)| h3_game::GameEvent::TownEventFired {
                player: red,
                town,
                event,
            })
            .collect()
    };
    let told = events(&fired, h3_game::Viewer::Player(red));
    assert!(
        told.iter()
            .all(|e| matches!(e, GameEvent::TownEventFired { player: 0, .. }))
    );
    let start = ServerMessage::GameEvents { n: 2, events: told };
    // In-process: red's snapshot, its events, its turn; blue's snapshot alone.
    let mut out = Vec::new();
    let manifest = manifest(&rules, MAP);
    GameSession::start(&rules, &setup, seed, manifest.clone(), OPTIONS, &mut out).unwrap();
    let decode = |out: &[GameOutbound], p: u8| -> Vec<ServerMessage> {
        (out.iter().filter(|o| o.to == PlayerId::new(p)))
            .map(|o| crate::codec::decode_server(o.frame.as_bytes(), usize::MAX).unwrap())
            .collect()
    };
    let reds = decode(&out, 0);
    assert_eq!(reds.len(), 3);
    assert_eq!(reds[1], start);
    assert!(matches!(reds[2], ServerMessage::GameTurn { n: 3, .. }));
    assert!((decode(&out, 1).iter()).all(|m| !matches!(m, ServerMessage::GameEvents { .. })));
    // Over the connections: at the last seat's join, before the turns, once; a re-taken seat
    // never hears them again.
    let mut s = GameSession::new(&rules, &setup, seed, manifest, OPTIONS, token).unwrap();
    let first = send(&mut s, A, join(0, None));
    assert!(
        to(&first, A)
            .iter()
            .all(|m| !matches!(m, ServerMessage::GameEvents { .. }))
    );
    let out = send(&mut s, B, join(1, None));
    let reds = to(&out, A);
    assert_eq!(reds[0], start);
    assert!(matches!(reds[1], ServerMessage::GameTurn { n: 3, .. }));
    s.disconnected(A);
    let again = send(&mut s, A, join(0, Some(&token(red))));
    assert!(
        to(&again, A)
            .iter()
            .all(|m| !matches!(m, ServerMessage::GameEvents { .. }))
    );
}
