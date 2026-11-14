//! The whole game's session (E16 M5 T5a, T5b): the golden adventure replays run through it, each
//! command applied as its actor's, each player's messages checked against an engine run beside
//! it (the order, the numbers, the events, a battle's last view, what each newly sees, the
//! battles' snapshots, the turns); a rejected command
//! changes nothing and is told to its player alone. The journal is the replay's commands and
//! checkpoints, its entries restore the game at any point, a journal that is not its re-run
//! restores nothing; each player's record is its header and every frame it was sent.

use std::string::String;
use std::vec;
use std::vec::Vec;

use h3_contracts::PlayerId;
use h3_game::{
    Actor, BitGrid, GameRuleset, GameSetup, GameState, OpenBattle, Sink, Viewer, apply,
    legal_commands,
};

use super::game::RULES;
use super::game_fold::{golden, one};
use crate::RECORD_FORMAT_VERSION;
use crate::codec;
use crate::map;
use crate::map::game::{command, events, manifest, observe};
use crate::session::game::{
    GameEntry, GameJournal, GameOptions, GameOutbound, GameRestoreError, GameSession,
};
use crate::wire::game::*;
use crate::wire::{ServerMessage, WireU64};

/// A map's identity for the manifest (the session only carries it).
const MAP: MapIdentity = MapIdentity {
    content_hash: WireU64(7),
    import_version: 1,
};

/// The options of a test's game: every record unbounded.
const fn options(checkpoint_every: u32) -> GameOptions {
    GameOptions {
        game: 9,
        checkpoint_every,
        record_bytes: usize::MAX,
        clock: crate::session::game::Clock::UNLIMITED,
    }
}

/// The engine's events and the battles a command ended, as they ended (`Sink::battle_ended`).
#[derive(Default)]
struct Kept {
    events: Vec<h3_game::GameEvent>,
    ended: Vec<OpenBattle>,
}

impl Sink for Kept {
    fn emit(&mut self, event: h3_game::GameEvent) {
        self.events.push(event);
    }

    fn battle_ended(&mut self, battle: &OpenBattle) {
        self.ended.push(battle.clone());
    }
}

/// Each player's frames in `out`, in order, appended to `frames`.
fn frames(out: &[GameOutbound], frames: &mut [Vec<String>]) {
    for o in out {
        frames[o.to.index()].push(o.frame.clone());
    }
}

/// The record of a game's `player` that holds `lines` from number `first` on.
fn record(session: &GameSession<'_>, player: PlayerId, first: u32, lines: &[String]) -> String {
    let header = GameRecordHeader {
        format: RECORD_FORMAT_VERSION,
        game: 9,
        player: player.raw(),
        manifest: session.manifest().clone(),
        first,
    };
    let mut record = codec::encode_text(&header);
    for line in lines {
        record.push('\n');
        record.push_str(line);
    }
    record
}

/// Checks what a game's start (`start`) or restore told each player in `out` against the
/// engine's state `state`: its snapshot, at the start the town events that fired at its turn
/// start inside the build in one `GameEvents` if any (E16 M5 T11; a restore tells none), then its
/// turn if it may act. Returns each player's last number, whether its view holds a battle and how
/// many start's events it was told.
fn opened(
    out: &[GameOutbound],
    (state, start): (&GameState, bool),
    rules: &GameRuleset,
    name: &str,
) -> (Vec<u32>, Vec<bool>, usize) {
    let players = state.players();
    let (mut next, mut battle) = (vec![0u32; players], vec![false; players]);
    let mut fired = 0;
    for (index, messages) in told(out, players).into_iter().enumerate() {
        let player = PlayerId::new(u8::try_from(index).unwrap());
        let observation = observe(state, rules, Viewer::Player(player));
        battle[index] = observation.battle.is_some();
        let mut expected = vec![ServerMessage::GameSnapshot { n: 1, observation }];
        // Without a timer, only that the game waits for another's level-up (E16 M5 T16).
        let clock = waiting(state, player);
        if clock != GameClock::default() {
            expected.push(ServerMessage::GameClock { n: 2, clock });
        }
        let at_start: Vec<h3_game::GameEvent> = state
            .town_events_at_start(player)
            .into_iter()
            .filter(|_| start)
            .map(|(town, event)| h3_game::GameEvent::TownEventFired {
                player,
                town,
                event,
            })
            .collect();
        if !at_start.is_empty() {
            fired += at_start.len();
            let events = events(&at_start, Viewer::Player(player));
            let n = u32::try_from(expected.len()).unwrap() + 1;
            expected.push(ServerMessage::GameEvents { n, events });
        }
        if let Some((legal, castable)) = turn(state, rules, player) {
            let n = u32::try_from(expected.len()).unwrap() + 1;
            expected.push(ServerMessage::GameTurn { n, legal, castable });
        }
        assert_eq!(messages, expected, "{name}");
        next[index] = u32::try_from(messages.len()).unwrap();
    }
    (next, battle, fired)
}

/// What `player` sees of an untimed game's clock in `state` (E16 M5 T16): whether the game waits
/// for another player's level-up.
fn waiting(state: &GameState, player: PlayerId) -> GameClock {
    GameClock {
        waits: state.waiting_for().is_some_and(|owner| owner != player),
        ..GameClock::default()
    }
}

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
        | ServerMessage::GameReveal { n, .. }
        | ServerMessage::GameTurn { n, .. }
        | ServerMessage::GameAccepted { n, .. }
        | ServerMessage::GameRejected { n, .. }
        | ServerMessage::GameClock { n, .. }
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
    restores: usize,
    reveals: usize,
    last_snapshots: usize,
    start_events: usize,
    clocks: usize,
}

/// Checks the player's messages `messages` after a command, their numbers from `next`: a
/// `GameAccepted` first if it sent `echo`, then its `GameEvents` (`told`), the last snapshot of
/// each battle of `ended` it owned a side of unless they open another battle for it, its
/// `GameReveal` when
/// its explored tiles grew since `grid`, the grid it was last told (then kept), a battle's
/// snapshot when one opens for it, its clock when whether the game waits for another player's
/// level-up changed (`waits`, as last told; E16 M5 T16), its turn if it may act. `battle` is
/// whether its view holds a battle.
#[allow(
    clippy::too_many_arguments,
    reason = "a test's check of one viewer's messages"
)]
fn check(
    messages: &[ServerMessage],
    next: &mut u32,
    echo: Option<(u32, &GameCommand)>,
    (told, ended): (Vec<GameEvent>, &[OpenBattle]),
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
    (battle_held, grid, waits): (&mut bool, &mut BitGrid, &mut GameClock),
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
        *battle_held = false;
    }
    let Some(ServerMessage::GameEvents { events, .. }) = m.next() else {
        panic!("no events: {messages:?}");
    };
    assert_eq!(*events, told);
    let observed = observe(state, rules, Viewer::Player(player)).battle;
    let mut rest: Vec<&ServerMessage> = m.collect();
    for battle in ended.iter().filter(|_| !started) {
        if let Some(side) = battle.side_of(player) {
            let last = map::observe(battle.state(), rules.battle(), side);
            let Some(ServerMessage::Snapshot { observation, .. }) = rest.first() else {
                panic!("no last snapshot: {rest:?}");
            };
            assert_eq!(*observation, last);
            assert!(!*battle_held, "a last snapshot after its battle's end");
            seen.last_snapshots += 1;
            rest.remove(0);
        }
    }
    let now = state.adventure().explored(player).unwrap();
    if let Some(ServerMessage::GameReveal { reveal, .. }) = rest.first() {
        assert_ne!(now, grid, "a reveal only when the explored tiles grew");
        let viewer = Viewer::Player(player);
        assert_eq!(*reveal, map::game::reveal(state, rules, viewer, grid));
        assert!(!reveal.adventure.areas.is_empty());
        seen.reveals += 1;
        rest.remove(0);
    } else {
        assert_eq!(now, grid, "the explored tiles grew without a reveal");
    }
    grid.clone_from(now);
    if let Some(ServerMessage::Snapshot { observation, .. }) = rest.first() {
        assert_eq!(Some(observation), observed.as_ref());
        assert!(
            started || !*battle_held,
            "a snapshot only when a battle opens"
        );
        *battle_held = true;
        seen.snapshots += 1;
        rest.remove(0);
    } else {
        assert!(
            !started || observed.is_none(),
            "a battle opened without its snapshot"
        );
    }
    // The view holds a battle exactly when the viewer has one.
    assert_eq!(*battle_held, observed.is_some());
    let clock = waiting(state, player);
    if let Some(ServerMessage::GameClock { clock: told, .. }) = rest.first() {
        assert_eq!(*told, clock);
        assert_ne!(*waits, clock, "a clock only when it changed");
        seen.clocks += 1;
        rest.remove(0);
    } else {
        assert_eq!(*waits, clock, "the clock changed untold");
    }
    *waits = clock;
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
        let options = options(replay.checkpoint_every);
        let mut session = GameSession::start(
            rules,
            setup,
            replay.seed,
            manifest(rules, MAP),
            options,
            &mut out,
        )
        .unwrap();
        assert_eq!(*session.manifest(), manifest(rules, MAP));
        let mut shadow = GameState::new(rules, setup, replay.seed).unwrap();
        let players = shadow.players();
        let ids: Vec<PlayerId> = (0..=u8::MAX).take(players).map(PlayerId::new).collect();
        let mut sent = vec![Vec::new(); players];
        frames(&out, &mut sent);
        // The start: each player's snapshot, its start's events, then its turn.
        let (mut next, mut battle, fired) = opened(&out, (&shadow, true), rules, name);
        seen.start_events += fired;
        let mut clocks: Vec<GameClock> = ids.iter().map(|&p| waiting(&shadow, p)).collect();
        let mut grids: Vec<BitGrid> = ids
            .iter()
            .map(|&p| shadow.adventure().explored(p).unwrap().clone())
            .collect();
        assert!(session.system_legal().is_empty());
        assert_eq!(
            session.drain_entries().count(),
            0,
            "a start journals nothing"
        );
        let mut entries = Vec::new();
        let middle = replay.commands.len().div_ceil(2);
        for (seq, (actor, engine)) in replay.commands.iter().enumerate() {
            let seq = u32::try_from(seq).unwrap();
            let wire = command(engine);
            let mut kept = Kept::default();
            apply(&mut shadow, rules, *actor, engine.clone(), &mut kept).unwrap();
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
            frames(&out, &mut sent);
            // The journal's entries: the command, then the checkpoint when one is due.
            let drained: Vec<GameEntry> = session.drain_entries().collect();
            let mut expected = vec![GameEntry::Command(*actor, engine.clone())];
            let done = usize::try_from(seq).unwrap() + 1;
            if done % usize::try_from(replay.checkpoint_every).unwrap() == 0 {
                expected.push(GameEntry::Checkpoint(shadow.state_hash()));
            }
            assert_eq!(drained, expected, "{name} command {seq}");
            entries.extend(drained);
            for (&player, messages) in ids.iter().zip(told(&out, players)) {
                let echo = (*actor == Actor::Player(player)).then_some((seq, &wire));
                check(
                    &messages,
                    &mut next[player.index()],
                    echo,
                    (events(&kept.events, Viewer::Player(player)), &kept.ended),
                    &shadow,
                    rules,
                    player,
                    (
                        &mut battle[player.index()],
                        &mut grids[player.index()],
                        &mut clocks[player.index()],
                    ),
                    &mut seen,
                );
            }
            seen.commands += 1;
            // Halfway, the entries so far restore the game as it stands, told afresh.
            if done == middle {
                let mut told_again = Vec::new();
                let restored = GameSession::restore(
                    rules,
                    setup,
                    replay.seed,
                    manifest(rules, MAP),
                    options,
                    &entries,
                    &mut told_again,
                )
                .unwrap();
                assert_eq!(restored.state().state_hash(), shadow.state_hash(), "{name}");
                assert_eq!(*restored.journal(), *session.journal(), "{name}");
                opened(&told_again, (&shadow, false), rules, name);
                let mut again = vec![Vec::new(); players];
                frames(&told_again, &mut again);
                for (&player, lines) in ids.iter().zip(&again) {
                    let expected = record(&restored, player, 1, lines);
                    assert_eq!(restored.record(player), Some(expected));
                }
                seen.restores += 1;
            }
        }
        assert_eq!(session.state().state_hash(), replay.final_hash, "{name}");
        // The journal is the replay's.
        let journal = GameJournal {
            seed: replay.seed,
            commands: replay.commands.clone(),
            checkpoint_every: replay.checkpoint_every,
            checkpoints: replay.checkpoints.clone(),
        };
        assert_eq!(*session.journal(), journal, "{name}");
        // Each player's record: its header, then every frame it was sent.
        for (&player, lines) in ids.iter().zip(&sent) {
            assert_eq!(
                session.record(player),
                Some(record(&session, player, 1, lines))
            );
        }
        let outside = PlayerId::new(u8::try_from(players).unwrap());
        assert_eq!(session.record(outside), None);
    });
    // Every kind of message was seen.
    assert!(seen.commands > 14_000, "{seen:?}");
    assert!(seen.system > 0, "{seen:?}");
    assert!(seen.snapshots > 0, "{seen:?}");
    assert!(seen.turns >= seen.commands - seen.system, "{seen:?}");
    assert!(seen.casts > 0, "{seen:?}");
    assert!(seen.reveals > 1_000, "{seen:?}");
    assert!(seen.last_snapshots > 0, "{seen:?}");
    assert!(seen.start_events > 0, "{seen:?}");
    assert!(seen.clocks > 0, "{seen:?}");
    assert_eq!(seen.restores, 86, "{seen:?}");
}

/// A game on a fixture map: its rules and setup.
pub(super) fn duel() -> (&'static GameRuleset, GameSetup) {
    let rules = &*RULES;
    let map = include_bytes!("../../../../fixtures/maps/e08-duel-36.h3m");
    let m = h3_map::read(map, &h3_map::Limits::default()).unwrap();
    (rules, h3_map::import(&m, rules).unwrap())
}

#[test]
fn a_rejected_command_changes_nothing_and_is_told_to_its_player_alone() {
    let (rules, setup) = duel();
    let mut out = Vec::new();
    let mut session =
        GameSession::start(rules, &setup, 3, manifest(rules, MAP), options(0), &mut out).unwrap();
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
    // E16 M5 T14 (v56): a seat's system command, the system actor's alone.
    out.clear();
    let end_acting = GameCommand::System {
        command: SystemCommand::EndTurn {
            player: acting.raw(),
        },
    };
    let code = session.play(waiting, 6, end_acting, &mut out).unwrap_err();
    assert_eq!(code, GameReject::SystemOnly {});
    assert_eq!(out, rejected(last(waiting) + 3, 6, code));
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
    let end_waiting = GameCommand::System {
        command: SystemCommand::EndTurn {
            player: waiting.raw(),
        },
    };
    assert_eq!(
        session.system(end_waiting, &mut out),
        Err(GameReject::NotYourTurn {
            player: waiting.raw()
        })
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
        ServerMessage::GameEvents { n, .. } if n == last(waiting) + 4
    ));
}

/// E16 M5 T14 (v56): the system's end of a player's turn is told as the player's own, but for
/// its echo, and journaled as the system actor's.
#[test]
fn the_systems_end_of_a_turn_is_told_as_the_players_own() {
    let (rules, setup) = duel();
    let start = |out: &mut Vec<GameOutbound>| {
        GameSession::start(rules, &setup, 3, manifest(rules, MAP), options(0), out).unwrap()
    };
    let (mut own, mut by_system) = (start(&mut Vec::new()), start(&mut Vec::new()));
    let players = own.state().players();
    let acting = own.state().turn_group()[0];
    let (mut ended, mut timed_out) = (Vec::new(), Vec::new());
    own.play(acting, 1, GameCommand::EndTurn {}, &mut ended)
        .unwrap();
    let end = SystemCommand::EndTurn {
        player: acting.raw(),
    };
    let command = GameCommand::System { command: end };
    by_system.system(command.clone(), &mut timed_out).unwrap();
    assert_eq!(by_system.state().state_hash(), own.state().state_hash());
    let (mut ended, timed_out) = (told(&ended, players), told(&timed_out, players));
    assert!(matches!(
        ended[acting.index()].remove(0),
        ServerMessage::GameAccepted { .. }
    ));
    let renumbered = |messages: &[ServerMessage]| -> Vec<ServerMessage> {
        messages
            .iter()
            .map(|m| {
                let mut m = m.clone();
                if let ServerMessage::GameEvents { n, .. } | ServerMessage::GameTurn { n, .. } =
                    &mut m
                {
                    *n = 0;
                }
                m
            })
            .collect()
    };
    for player in 0..players {
        assert_eq!(renumbered(&ended[player]), renumbered(&timed_out[player]));
    }
    let entries: Vec<GameEntry> = by_system.drain_entries().collect();
    let engine = map::game::command_to_engine(command).unwrap();
    assert!(entries.contains(&GameEntry::Command(Actor::System, engine)));
}

#[test]
fn a_setup_that_starts_no_game_sends_nothing() {
    let (rules, setup) = duel();
    let setup = GameSetup {
        difficulty: 200,
        ..setup
    };
    let mut out = Vec::new();
    let started = GameSession::start(rules, &setup, 3, manifest(rules, MAP), options(0), &mut out);
    assert_eq!(
        started.map(|_| ()),
        Err(h3_game::GameSetupError::Difficulty(200))
    );
    let restored = GameSession::restore(
        rules,
        &setup,
        3,
        manifest(rules, MAP),
        options(0),
        &[],
        &mut out,
    );
    assert_eq!(
        restored.map(|_| ()),
        Err(GameRestoreError::Setup(
            h3_game::GameSetupError::Difficulty(200)
        ))
    );
    assert_eq!(out, Vec::<GameOutbound>::new());
}

/// A duel of `commands` accepted commands, a checkpoint after each: its session and journal.
fn journaled(
    rules: &'static GameRuleset,
    setup: &GameSetup,
    commands: usize,
) -> (GameSession<'static>, Vec<GameEntry>) {
    let mut out = Vec::new();
    let mut session =
        GameSession::start(rules, setup, 3, manifest(rules, MAP), options(1), &mut out).unwrap();
    let mut entries = Vec::new();
    for seq in 0..commands {
        let (player, legal) = (0..session.state().players())
            .map(|p| PlayerId::new(u8::try_from(p).unwrap()))
            .find_map(|p| {
                let legal = legal_commands(session.state(), rules, Actor::Player(p));
                (!legal.is_empty()).then_some((p, legal))
            })
            .unwrap();
        let end = legal
            .iter()
            .find(|c| matches!(c, h3_game::GameCommand::EndTurn))
            .unwrap_or(&legal[0]);
        let seq = u32::try_from(seq).unwrap();
        session.play(player, seq, command(end), &mut out).unwrap();
        entries.extend(session.drain_entries());
    }
    (session, entries)
}

/// The restore of `entries` on the duel: its session, or why not.
fn restored(
    rules: &'static GameRuleset,
    setup: &GameSetup,
    entries: &[GameEntry],
) -> Result<GameSession<'static>, GameRestoreError> {
    GameSession::restore(
        rules,
        setup,
        3,
        manifest(rules, MAP),
        options(1),
        entries,
        &mut Vec::new(),
    )
}

#[test]
fn a_journal_that_is_not_its_rerun_restores_nothing() {
    let (rules, setup) = duel();
    let (session, entries) = journaled(rules, &setup, 6);
    assert_eq!(entries.len(), 12, "a checkpoint after each command");
    let back = restored(rules, &setup, &entries).unwrap();
    assert_eq!(back.state().state_hash(), session.state().state_hash());
    let restore = |entries: &[GameEntry]| restored(rules, &setup, entries).err();
    // A flipped hash.
    let mut flipped = entries.clone();
    flipped[3] = GameEntry::Checkpoint(1);
    assert_eq!(
        restore(&flipped),
        Some(GameRestoreError::Diverged { at: 3 })
    );
    // A checkpoint out of place, or one where none is due.
    let mut early = entries.clone();
    early.swap(1, 2);
    assert_eq!(restore(&early), Some(GameRestoreError::Diverged { at: 1 }));
    let mut extra = entries.clone();
    extra.push(GameEntry::Checkpoint(0));
    assert_eq!(
        restore(&extra),
        Some(GameRestoreError::Diverged { at: entries.len() })
    );
    // A command the engine refuses: its player's, out of its turn.
    let GameEntry::Command(Actor::Player(acting), _) = entries[2] else {
        panic!("{entries:?}");
    };
    let waiting = PlayerId::new(u8::from(acting.raw() == 0));
    let mut wrong = entries.clone();
    wrong[2] = GameEntry::Command(Actor::Player(waiting), h3_game::GameCommand::EndTurn);
    assert_eq!(
        restore(&wrong),
        Some(GameRestoreError::Rejected {
            at: 2,
            error: h3_game::GameCommandError::NotYourTurn(waiting),
        })
    );
}

#[test]
fn a_journal_cut_short_is_completed() {
    let (rules, setup) = duel();
    let (session, entries) = journaled(rules, &setup, 4);
    // A crash between the last command and its checkpoint: the restore makes the checkpoint.
    let cut = &entries[..entries.len() - 1];
    let mut back = restored(rules, &setup, cut).unwrap();
    assert_eq!(
        back.drain_entries().collect::<Vec<_>>(),
        [entries[entries.len() - 1].clone()]
    );
    assert_eq!(*back.journal(), *session.journal());
    // The restored game goes on, journaling on.
    let mut out = Vec::new();
    let acting = back.state().turn_group()[0];
    back.play(acting, 0, GameCommand::EndTurn {}, &mut out)
        .unwrap();
    let after: Vec<GameEntry> = back.drain_entries().collect();
    assert!(matches!(
        after.as_slice(),
        [GameEntry::Command(..), GameEntry::Checkpoint(_)]
    ));
    assert_eq!(back.journal().commands.len(), 5);
}

/// Whether a frame is a whole game's snapshot.
fn snapshot(frame: &str) -> bool {
    frame.starts_with(r#"{"GameSnapshot":"#)
}

#[test]
fn a_record_is_cut_at_its_snapshots_within_its_bound() {
    const BOUND: usize = 48 << 10;
    let (rules, setup, replay) = one("towns-s173-d21");
    let bounded = GameOptions {
        record_bytes: BOUND,
        ..options(0)
    };
    let mut out = Vec::new();
    let mut session = GameSession::start(
        &rules,
        &setup,
        replay.seed,
        manifest(&rules, MAP),
        bounded,
        &mut out,
    )
    .unwrap();
    let players = session.state().players();
    let ids: Vec<PlayerId> = (0..=u8::MAX).take(players).map(PlayerId::new).collect();
    let mut sent = vec![Vec::new(); players];
    frames(&out, &mut sent);
    // The bytes each player was sent since its last snapshot: the start's after its own.
    let mut since: Vec<usize> = (sent.iter())
        .map(|lines| {
            let after = lines.iter().rev().take_while(|l| !snapshot(l));
            after.map(|l| l.len() + 1).sum()
        })
        .collect();
    let (mut fresh, mut in_battle) = (0, 0);
    for (seq, (actor, engine)) in replay.commands.iter().enumerate() {
        let seq = u32::try_from(seq).unwrap();
        out.clear();
        match *actor {
            Actor::Player(player) => session.play(player, seq, command(engine), &mut out),
            Actor::System => session.system(command(engine), &mut out),
        }
        .unwrap();
        let mut now = vec![Vec::new(); players];
        frames(&out, &mut now);
        for (&player, frames) in ids.iter().zip(&now) {
            let i = player.index();
            // The turns come after the session's look at its bound.
            let turns = frames.iter().filter(|f| f.starts_with(r#"{"GameTurn":"#));
            let turns: usize = turns.map(|f| f.len() + 1).sum();
            for (k, frame) in frames.iter().enumerate() {
                if !snapshot(frame) {
                    since[i] += frame.len() + 1;
                    continue;
                }
                // A fresh snapshot only past half the bound, as observed (inside a battle too,
                // E16 M5 T12: it holds the battle's view), after the command's other messages
                // but the turn.
                assert!(since[i] > BOUND / 2, "{seq}");
                let observation = observe(session.state(), &rules, Viewer::Player(player));
                in_battle += usize::from(observation.battle.is_some());
                let message = ServerMessage::GameSnapshot {
                    n: number(&codec::decode_server(frame.as_bytes(), usize::MAX).unwrap()),
                    observation,
                };
                assert_eq!(*frame, codec::encode_text(&message), "{seq}");
                assert!(
                    frames[k + 1..]
                        .iter()
                        .all(|f| f.starts_with(r#"{"GameTurn":"#))
                );
                since[i] = 0;
                fresh += 1;
            }
            // Never past half the bound without one but for the turns (E16 M5 T12: a battle
            // held none back before).
            assert!(since[i] - turns <= BOUND / 2, "{seq}");
            sent[i].extend(frames.iter().cloned());
            // The record: from a snapshot on, every frame since; within the bound unless it
            // begins with the newest snapshot, the earlier one beyond it.
            let lines = &sent[i];
            let begins = |at: usize| lines[at..].iter().map(|l| l.len() + 1).sum::<usize>();
            let snapshots: Vec<usize> = (0..lines.len()).filter(|&k| snapshot(&lines[k])).collect();
            let start = snapshots
                .iter()
                .copied()
                .find(|&k| begins(k) <= BOUND)
                .unwrap_or(*snapshots.last().unwrap());
            if let Some(&earlier) = snapshots.iter().rev().find(|&&k| k < start) {
                assert!(begins(earlier) > BOUND);
            }
            let first = u32::try_from(start).unwrap() + 1;
            let expected = record(&session, player, first, &lines[start..]);
            assert_eq!(session.record(player), Some(expected), "{seq}");
        }
    }
    assert!(fresh > 10, "{fresh}");
    assert!(in_battle > 0, "{in_battle}");
}

#[test]
fn a_players_own_rejections_keep_its_record_within_its_bound() {
    // E16 M5 T7b's review: rejections fill a record as the events do.
    let (rules, setup) = duel();
    let mut out = Vec::new();
    let session =
        GameSession::start(rules, &setup, 3, manifest(rules, MAP), options(0), &mut out).unwrap();
    let acting = session.state().turn_group()[0];
    let waiting = PlayerId::new(u8::from(acting.raw() == 0));
    let snapshot = out.iter().find(|o| o.to == waiting).unwrap().frame.len();
    let bound = 2 * snapshot + (4 << 10);
    let bounded = GameOptions {
        record_bytes: bound,
        ..options(0)
    };
    out.clear();
    let mut session =
        GameSession::start(rules, &setup, 3, manifest(rules, MAP), bounded, &mut out).unwrap();
    let (mut fresh, hash) = (0, session.state().state_hash());
    for seq in 0..2_000 {
        out.clear();
        let refused = session.play(waiting, seq, GameCommand::EndTurn {}, &mut out);
        assert!(refused.is_err());
        // Its rejection, then a fresh snapshot past half the bound, to it alone.
        assert!(out.iter().all(|o| o.to == waiting));
        assert!(out[0].frame.starts_with(r#"{"GameRejected":"#));
        fresh += out.len() - 1;
        let record = session.record(waiting).unwrap();
        let mut lines = record.lines().skip(1);
        assert!(lines.next().unwrap().starts_with(r#"{"GameSnapshot":"#));
        let bytes: usize = record.lines().skip(1).map(|l| l.len() + 1).sum();
        assert!(bytes <= bound, "{seq}: {bytes} of {bound}");
    }
    assert!(fresh >= 2, "{fresh}");
    assert_eq!(session.state().state_hash(), hash);
}

#[test]
fn a_players_own_rejections_inside_a_battle_keep_its_record_within_its_bound() {
    // E16 M5 T12 (T7b's second finding): a fresh snapshot inside a battle too, holding it.
    let (rules, setup, replay) = one("duel-s1-d28");
    let start = |bound: usize| {
        let options = GameOptions {
            record_bytes: bound,
            ..options(0)
        };
        let mut out = Vec::new();
        let manifest = manifest(&rules, MAP);
        GameSession::start(&rules, &setup, replay.seed, manifest, options, &mut out).unwrap()
    };
    // The replay up to its first command that opens a battle for a player.
    let to_battle = |session: &mut GameSession<'_>| {
        for (seq, (actor, engine)) in replay.commands.iter().enumerate() {
            let mut out = Vec::new();
            let seq = u32::try_from(seq).unwrap();
            match *actor {
                Actor::Player(player) => session.play(player, seq, command(engine), &mut out),
                Actor::System => session.system(command(engine), &mut out),
            }
            .unwrap();
            let players = (0..=u8::MAX).take(session.state().players());
            if let Some(player) = players
                .map(PlayerId::new)
                .find(|&p| session.state().battle(p).is_some())
            {
                return player;
            }
        }
        panic!("no battle");
    };
    let mut probe = start(usize::MAX);
    let fighting = to_battle(&mut probe);
    let observation = observe(probe.state(), &rules, Viewer::Player(fighting));
    assert!(observation.battle.is_some());
    let message = ServerMessage::GameSnapshot { n: 1, observation };
    let bound = 2 * codec::encode_text(&message).len() + (4 << 10);
    let mut session = start(bound);
    assert_eq!(to_battle(&mut session), fighting);
    let (mut fresh, hash) = (0, session.state().state_hash());
    for seq in 0..2_000 {
        let mut out = Vec::new();
        let refused = session.play(fighting, 1 << 20 | seq, GameCommand::EndTurn {}, &mut out);
        assert!(refused.is_err());
        assert!(out.iter().all(|o| o.to == fighting));
        assert!(out[0].frame.starts_with(r#"{"GameRejected":"#));
        for o in &out[1..] {
            // The fresh snapshot holds the battle: the battle's own `Snapshot` is not resent.
            let m = codec::decode_server(o.frame.as_bytes(), usize::MAX).unwrap();
            let ServerMessage::GameSnapshot { observation, .. } = m else {
                panic!("{m:?}");
            };
            assert!(observation.battle.is_some());
            fresh += 1;
        }
        let record = session.record(fighting).unwrap();
        let mut lines = record.lines().skip(1);
        assert!(lines.next().unwrap().starts_with(r#"{"GameSnapshot":"#));
        let bytes: usize = record.lines().skip(1).map(|l| l.len() + 1).sum();
        assert!(bytes <= bound, "{seq}: {bytes} of {bound}");
    }
    assert!(fresh >= 2, "{fresh}");
    assert_eq!(session.state().state_hash(), hash);
}
