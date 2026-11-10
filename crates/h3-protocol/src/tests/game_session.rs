//! The whole game's session (E16 M5 T5a): the golden adventure replays run through it, each
//! command applied as its actor's, each player's messages checked against an engine run beside
//! it (the order, the numbers, the events, the battles' snapshots, the turns); a rejected command
//! changes nothing and is told to its player alone.

use std::vec;
use std::vec::Vec;

use h3_contracts::PlayerId;
use h3_game::{Actor, GameRuleset, GameSetup, GameState, Viewer, apply, legal_commands};

use super::game::RULES;
use super::game_fold::golden;
use crate::codec;
use crate::map;
use crate::map::game::{command, events, manifest, observe};
use crate::session::game::{GameOutbound, GameSession};
use crate::wire::game::*;
use crate::wire::{ServerMessage, WireU64};

/// A map's identity for the manifest (the session only carries it).
const MAP: MapIdentity = MapIdentity {
    content_hash: WireU64(7),
    import_version: 1,
};

/// Each player's messages in `out`, decoded, in order.
fn told(out: &[GameOutbound], players: usize) -> Vec<Vec<ServerMessage>> {
    let mut told = vec![Vec::new(); players];
    for o in out {
        told[o.to.index()].push(codec::decode_server(o.frame.as_bytes(), usize::MAX).unwrap());
    }
    told
}

/// A message's number.
fn number(message: &ServerMessage) -> u32 {
    match message {
        ServerMessage::GameSnapshot { n, .. }
        | ServerMessage::GameEvents { n, .. }
        | ServerMessage::GameTurn { n, .. }
        | ServerMessage::GameAccepted { n, .. }
        | ServerMessage::GameRejected { n, .. }
        | ServerMessage::Snapshot { n, .. } => *n,
        other => panic!("not a game's message: {other:?}"),
    }
}

/// The turn `player` gets in `state`: none if it may not act.
fn turn(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
) -> Option<(Vec<GameCommand>, Vec<u16>)> {
    let legal: Vec<GameCommand> = legal_commands(state, rules, Actor::Player(player))
        .iter()
        .map(command)
        .collect();
    let castable = state
        .battle(player)
        .and_then(|battle| {
            let side = battle.side_of(player)?;
            (battle.state().side_to_act() == Some(side))
                .then(|| map::castable(battle.state(), rules.battle(), side))
        })
        .unwrap_or_default();
    (!legal.is_empty()).then_some((legal, castable))
}

/// What a run of the golden replays saw.
#[derive(Debug, Default)]
struct Seen {
    commands: usize,
    system: usize,
    snapshots: usize,
    turns: usize,
    casts: usize,
}

/// Checks the player's messages `messages` after a command, their numbers from `next`: a
/// `GameAccepted` first if it sent `echo`, then its `GameEvents` (`told`), a battle's snapshot
/// when one opens for it, its turn if it may act. `battle` is whether its view holds a battle.
#[allow(
    clippy::too_many_arguments,
    reason = "a test's check of one viewer's messages"
)]
fn check(
    messages: &[ServerMessage],
    next: &mut u32,
    echo: Option<(u32, &GameCommand)>,
    told: Vec<GameEvent>,
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    battle: &mut bool,
    seen: &mut Seen,
) {
    for m in messages {
        *next += 1;
        assert_eq!(number(m), *next, "{m:?}");
    }
    let mut m = messages.iter();
    if let Some((seq, command)) = echo {
        assert_eq!(
            m.next(),
            Some(&ServerMessage::GameAccepted {
                n: number(&messages[0]),
                seq,
                command: command.clone(),
            })
        );
    }
    let started = told
        .iter()
        .any(|e| matches!(e, GameEvent::BattleStarted { .. }));
    if told
        .iter()
        .any(|e| matches!(e, GameEvent::BattleEnded { .. }))
    {
        *battle = false;
    }
    let Some(ServerMessage::GameEvents { events, .. }) = m.next() else {
        panic!("no events: {messages:?}");
    };
    assert_eq!(*events, told);
    let observed = observe(state, rules, Viewer::Player(player)).battle;
    let mut rest: Vec<&ServerMessage> = m.collect();
    if let Some(ServerMessage::Snapshot { observation, .. }) = rest.first() {
        assert_eq!(Some(observation), observed.as_ref());
        assert!(started || !*battle, "a snapshot only when a battle opens");
        *battle = true;
        seen.snapshots += 1;
        rest.remove(0);
    } else {
        assert!(
            !started || observed.is_none(),
            "a battle opened without its snapshot"
        );
    }
    // The view holds a battle exactly when the viewer has one.
    assert_eq!(*battle, observed.is_some());
    match (rest.as_slice(), turn(state, rules, player)) {
        ([], None) => {}
        (
            [
                ServerMessage::GameTurn {
                    legal, castable, ..
                },
            ],
            Some((l, c)),
        ) => {
            assert_eq!((legal, castable), (&l, &c));
            seen.turns += 1;
            seen.casts += usize::from(!c.is_empty());
        }
        (rest, expected) => panic!("the turn: {rest:?} against {expected:?}"),
    }
}

#[test]
fn the_golden_replays_run_through_the_session() {
    let mut seen = Seen::default();
    golden(|name, rules, setup, replay| {
        let mut out = Vec::new();
        let mut session =
            GameSession::start(rules, setup, replay.seed, manifest(rules, MAP), &mut out).unwrap();
        assert_eq!(*session.manifest(), manifest(rules, MAP));
        let mut shadow = GameState::new(rules, setup, replay.seed).unwrap();
        let players = shadow.players();
        let ids: Vec<PlayerId> = (0..=u8::MAX).take(players).map(PlayerId::new).collect();
        let mut next = vec![0u32; players];
        let mut battle = vec![false; players];
        // The start: each player's snapshot, then its turn.
        for (&player, messages) in ids.iter().zip(told(&out, players)) {
            let observation = observe(&shadow, rules, Viewer::Player(player));
            battle[player.index()] = observation.battle.is_some();
            assert_eq!(
                messages[0],
                ServerMessage::GameSnapshot { n: 1, observation }
            );
            let rest: Vec<ServerMessage> = messages[1..].to_vec();
            let expected = turn(&shadow, rules, player).map(|(legal, castable)| {
                vec![ServerMessage::GameTurn {
                    n: 2,
                    legal,
                    castable,
                }]
            });
            assert_eq!(rest, expected.unwrap_or_default(), "{name}");
            next[player.index()] = u32::try_from(messages.len()).unwrap();
        }
        assert!(session.system_legal().is_empty());
        for (seq, (actor, engine)) in replay.commands.iter().enumerate() {
            let seq = u32::try_from(seq).unwrap();
            let wire = command(engine);
            let mut events_told = Vec::new();
            apply(&mut shadow, rules, *actor, engine.clone(), &mut events_told).unwrap();
            out.clear();
            match *actor {
                Actor::Player(player) => session.play(player, seq, wire.clone(), &mut out),
                Actor::System => {
                    assert!(!session.system_legal().is_empty(), "{name} command {seq}");
                    seen.system += 1;
                    session.system(wire.clone(), &mut out)
                }
            }
            .unwrap();
            assert_eq!(session.state().state_hash(), shadow.state_hash());
            for (&player, messages) in ids.iter().zip(told(&out, players)) {
                let echo = (*actor == Actor::Player(player)).then_some((seq, &wire));
                check(
                    &messages,
                    &mut next[player.index()],
                    echo,
                    events(&events_told, Viewer::Player(player)),
                    &shadow,
                    rules,
                    player,
                    &mut battle[player.index()],
                    &mut seen,
                );
            }
            seen.commands += 1;
        }
        assert_eq!(session.state().state_hash(), replay.final_hash, "{name}");
    });
    // Every kind of message was seen.
    assert!(seen.commands > 14_000, "{seen:?}");
    assert!(seen.system > 0, "{seen:?}");
    assert!(seen.snapshots > 0, "{seen:?}");
    assert!(seen.turns >= seen.commands - seen.system, "{seen:?}");
    assert!(seen.casts > 0, "{seen:?}");
}

/// A game on a fixture map: its rules and setup.
fn duel() -> (&'static GameRuleset, GameSetup) {
    let rules = &*RULES;
    let map = include_bytes!("../../../../fixtures/maps/e08-duel-36.h3m");
    let m = h3_map::read(map, &h3_map::Limits::default()).unwrap();
    (rules, h3_map::import(&m, rules).unwrap())
}

#[test]
fn a_rejected_command_changes_nothing_and_is_told_to_its_player_alone() {
    let (rules, setup) = duel();
    let mut out = Vec::new();
    let mut session = GameSession::start(rules, &setup, 3, manifest(rules, MAP), &mut out).unwrap();
    let players = session.state().players();
    let started = told(&out, players);
    let acting = session.state().turn_group()[0];
    let waiting = PlayerId::new(u8::from(acting.raw() == 0));
    assert!(!session.state().is_acting(waiting));
    let hash = session.state().state_hash();
    let last = |p: PlayerId| u32::try_from(started[p.index()].len()).unwrap();
    // Out of its turn: told to it alone, numbered on.
    out.clear();
    let code = session.play(waiting, 4, GameCommand::EndTurn {}, &mut out);
    assert_eq!(
        code,
        Err(GameReject::NotYourTurn {
            player: waiting.raw()
        })
    );
    assert_eq!(session.state().state_hash(), hash);
    let rejected = |n, seq, code| {
        vec![GameOutbound {
            to: waiting,
            frame: codec::encode_text(&ServerMessage::GameRejected { n, seq, code }),
        }]
    };
    assert_eq!(out, rejected(last(waiting) + 1, 4, code.unwrap_err()));
    // A building beyond any record, refused before the engine.
    out.clear();
    let build = GameCommand::Town {
        command: TownCommand::Build {
            town: 0,
            building: 64,
        },
    };
    let code = session.play(waiting, 5, build, &mut out).unwrap_err();
    assert_eq!(
        code,
        GameReject::Town {
            error: TownReject::UnknownBuilding {}
        }
    );
    assert_eq!(out, rejected(last(waiting) + 2, 5, code));
    // A player outside the game is told nothing; nor is the system actor.
    out.clear();
    let outside = PlayerId::new(200);
    assert_eq!(
        session.play(outside, 6, GameCommand::EndTurn {}, &mut out),
        Err(GameReject::UnknownPlayer { player: 200 })
    );
    assert_eq!(
        session.system(GameCommand::EndTurn {}, &mut out),
        Err(GameReject::NotForSystem {})
    );
    assert!(out.is_empty());
    assert!(session.system_legal().is_empty());
    assert_eq!(session.state().state_hash(), hash);
    // The acting player's command is accepted: its echo first, then every player's events.
    session
        .play(acting, 7, GameCommand::EndTurn {}, &mut out)
        .unwrap();
    assert_ne!(session.state().state_hash(), hash);
    let after = told(&out, players);
    assert_eq!(
        after[acting.index()][0],
        ServerMessage::GameAccepted {
            n: last(acting) + 1,
            seq: 7,
            command: GameCommand::EndTurn {},
        }
    );
    assert!(matches!(
        after[waiting.index()][0],
        ServerMessage::GameEvents { n, .. } if n == last(waiting) + 3
    ));
}

#[test]
fn a_setup_that_starts_no_game_sends_nothing() {
    let (rules, setup) = duel();
    let setup = GameSetup {
        difficulty: 200,
        ..setup
    };
    let mut out = Vec::new();
    let started = GameSession::start(rules, &setup, 3, manifest(rules, MAP), &mut out);
    assert_eq!(
        started.map(|_| ()),
        Err(h3_game::GameSetupError::Difficulty(200))
    );
    assert_eq!(out, Vec::<GameOutbound>::new());
}
