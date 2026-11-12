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
use super::game_fold::golden;
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

/// Checks what a game's start or restore told each player in `out` against the engine's state
/// `state`: its snapshot, then its turn if it may act. Returns each player's last number and
/// whether its view holds a battle.
fn opened(
    out: &[GameOutbound],
    state: &GameState,
    rules: &GameRuleset,
    name: &str,
) -> (Vec<u32>, Vec<bool>) {
    let players = state.players();
    let (mut next, mut battle) = (vec![0u32; players], vec![false; players]);
    for (index, messages) in told(out, players).into_iter().enumerate() {
        let player = PlayerId::new(u8::try_from(index).unwrap());
        let observation = observe(state, rules, Viewer::Player(player));
        battle[index] = observation.battle.is_some();
        assert_eq!(
            messages[0],
            ServerMessage::GameSnapshot { n: 1, observation }
        );
        let rest: Vec<ServerMessage> = messages[1..].to_vec();
        let expected = turn(state, rules, player).map(|(legal, castable)| {
            vec![ServerMessage::GameTurn {
                n: 2,
                legal,
                castable,
            }]
        });
        assert_eq!(rest, expected.unwrap_or_default(), "{name}");
        next[index] = u32::try_from(messages.len()).unwrap();
    }
    (next, battle)
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
        | ServerMessage::GameLastSnapshot { n, .. }
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
    restores: usize,
    reveals: usize,
    last_snapshots: usize,
}

/// Checks the player's messages `messages` after a command, their numbers from `next`: a
/// `GameAccepted` first if it sent `echo`, then its `GameEvents` (`told`), the last view of each
/// battle of `ended` it owned a side of, its `GameReveal` when
/// its explored tiles grew since `grid`, the grid it was last told (then kept), a battle's
/// snapshot when one opens for it, its turn if it may act. `battle` is whether its view holds a
/// battle.
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
    (battle_held, grid): (&mut bool, &mut BitGrid),
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
    for battle in ended {
        if let Some(side) = battle.side_of(player) {
            let last = map::observe(battle.state(), rules.battle(), side);
            let Some(ServerMessage::GameLastSnapshot { observation, .. }) = rest.first() else {
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
        assert!(!reveal.adventure.tiles.is_empty());
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
        // The start: each player's snapshot, then its turn.
        let (mut next, mut battle) = opened(&out, &shadow, rules, name);
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
                    (&mut battle[player.index()], &mut grids[player.index()]),
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
                opened(&told_again, &shadow, rules, name);
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
    assert_eq!(seen.restores, 72, "{seen:?}");
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

#[test]
fn a_record_keeps_its_newest_lines_within_its_bound() {
    let (rules, setup) = duel();
    let mut out = Vec::new();
    let bounded = GameOptions {
        record_bytes: 1,
        ..options(0)
    };
    let mut session =
        GameSession::start(rules, &setup, 3, manifest(rules, MAP), bounded, &mut out).unwrap();
    let acting = session.state().turn_group()[0];
    session
        .play(acting, 0, GameCommand::EndTurn {}, &mut out)
        .unwrap();
    // Each player's record holds its newest frame alone, numbered as its last.
    let mut sent = vec![Vec::new(); session.state().players()];
    frames(&out, &mut sent);
    for (index, lines) in sent.iter().enumerate() {
        let player = PlayerId::new(u8::try_from(index).unwrap());
        let last = u32::try_from(lines.len()).unwrap();
        let expected = record(&session, player, last, &lines[lines.len() - 1..]);
        assert_eq!(session.record(player), Some(expected));
    }
}
