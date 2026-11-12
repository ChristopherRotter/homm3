//! The whole game's messages mutated (E16 M5 T7a2; the decoder's mutation tests of M2 T5d,
//! `mutation.rs`, for the game). A client's side: a frame of a player's record of a golden replay
//! (its header, its snapshot, its events, reveals, turns and battle snapshots) or of a game event
//! of every kind, mutated in its bytes or its values, decodes or not without a panic, round-trips
//! when it decodes, and folds onto the view as a client keeps it (a snapshot replaces the view,
//! `GameObservation::fold`, `GameObservation::reveal`) with the rest of the record after it,
//! without a panic. The server's side: a wire command of every kind or of the replay, mutated,
//! played into the session at a point of the replay by a player of the game, the replay's next
//! actor, the system actor or a player outside the game, never panics; a rejection leaves the
//! state and the journal unchanged and is told to its player alone (to nobody for the system and
//! a player outside the game), an accepted command is journaled so that a restore re-runs it to
//! the same state.
//!
//! A longer run: `PROPTEST_CASES=10000 cargo test -p h3-protocol mutat`.

use std::string::String;
use std::sync::LazyLock;
use std::vec;
use std::vec::Vec;

use h3_contracts::PlayerId;
use h3_game::{Actor, GameRuleset, GameSetup, Viewer};
use h3_sim::adventure::replay::Replay;
use proptest::prelude::*;

use super::game_command::every_command;
use super::game_event::every_event;
use super::game_fold::one;
use super::mutation::{Mutation, NUMBERS, STRINGS};
use crate::codec::{decode, decode_server, encode, encode_text};
use crate::map::game::{command, events, manifest};
use crate::session::game::{GameEntry, GameOptions, GameOutbound, GameSession};
use crate::wire::game::{GameCommand, GameObservation, GameRecordHeader, MapIdentity};
use crate::wire::{ServerMessage, WireU64};

/// The replay: battles, towns, sieges and reveals for both players.
static REPLAY: LazyLock<(GameRuleset, GameSetup, Replay)> = LazyLock::new(|| one("towns-s173-d21"));

/// The game's options: every record unbounded, no checkpoint.
const OPTIONS: GameOptions = GameOptions {
    game: 2,
    checkpoint_every: 0,
    record_bytes: usize::MAX,
};

/// The map's identity for the manifest.
const MAP: MapIdentity = MapIdentity {
    content_hash: WireU64(5),
    import_version: 1,
};

/// Player 0's record of the replay played through the session: its header, then its messages.
static RECORD: LazyLock<(String, Vec<ServerMessage>)> = LazyLock::new(|| {
    let (rules, setup, replay) = &*REPLAY;
    let mut out = Vec::new();
    let mut session = GameSession::start(
        rules,
        setup,
        replay.seed,
        manifest(rules, MAP),
        OPTIONS,
        &mut out,
    )
    .unwrap();
    for (seq, (actor, engine)) in replay.commands.iter().enumerate() {
        let seq = u32::try_from(seq).unwrap();
        match *actor {
            Actor::Player(player) => session.play(player, seq, command(engine), &mut out),
            Actor::System => session.system(command(engine), &mut out),
        }
        .unwrap();
    }
    let record = session.record(PlayerId::new(0)).unwrap();
    let mut lines = record.lines();
    let header = String::from(lines.next().unwrap());
    let messages = lines
        .map(|line| decode_server(line.as_bytes(), usize::MAX).unwrap())
        .collect();
    (header, messages)
});

/// The frames a mutation starts from: the record's header and messages, then an event of every
/// kind (player 0's view of each), each as a `GameEvents` of its own.
static FRAMES: LazyLock<Vec<Vec<u8>>> = LazyLock::new(|| {
    let (header, messages) = &*RECORD;
    let every = events(&every_event(), Viewer::Player(PlayerId::new(0)));
    let extra = every.into_iter().map(|event| {
        encode(&ServerMessage::GameEvents {
            n: 9,
            events: vec![event],
        })
    });
    let record = messages.iter().map(encode);
    [header.clone().into_bytes()]
        .into_iter()
        .chain(record)
        .chain(extra)
        .collect()
});

/// The commands a mutation starts from: one of every kind, then the replay's, as their JSON.
static COMMANDS: LazyLock<Vec<Vec<u8>>> = LazyLock::new(|| {
    let (_, _, replay) = &*REPLAY;
    let every = every_command().into_iter();
    let played = replay.commands.iter().map(|(_, engine)| engine.clone());
    every.chain(played).map(|c| encode(&command(&c))).collect()
});

/// A change to a frame anywhere in it: the decoder's mutations (`mutation.rs`) over a game's
/// longer frames, a value among all of a frame's.
fn arb_mutation() -> impl Strategy<Value = Mutation> {
    let anywhere = 0usize..1 << 20;
    let which = 0usize..1 << 16;
    prop_oneof![
        (anywhere.clone(), 1u8..=255).prop_map(|(at, mask)| Mutation::Flip { at, mask }),
        (anywhere.clone(), 1usize..16).prop_map(|(at, len)| Mutation::Cut { at, len }),
        (
            anywhere.clone(),
            proptest::collection::vec(any::<u8>(), 1..8)
        )
            .prop_map(|(at, bytes)| Mutation::Insert { at, bytes }),
        (anywhere.clone(), 1usize..32).prop_map(|(at, len)| Mutation::Repeat { at, len }),
        (0usize..1024, anywhere.clone(), anywhere).prop_map(|(other, at, from)| Mutation::Splice {
            other,
            at,
            from
        }),
        (which.clone(), 0..NUMBERS.len()).prop_map(|(which, to)| Mutation::Number { which, to }),
        (which, 0..STRINGS.len()).prop_map(|(which, to)| Mutation::Text { which, to }),
    ]
}

/// A value replaced: most such frames still parse.
fn arb_value_mutation() -> impl Strategy<Value = Mutation> {
    prop_oneof![
        3 => (0usize..1 << 10, 0..NUMBERS.len()).prop_map(|(which, to)| Mutation::Number { which, to }),
        1 => (0usize..1 << 10, 0..STRINGS.len()).prop_map(|(which, to)| Mutation::Text { which, to }),
    ]
}

/// What a client does with a message of its game: a snapshot replaces its view, the events and a
/// reveal fold into it; the rest leaves it.
fn client(view: &mut Option<GameObservation>, message: &ServerMessage) {
    match (message, view.as_mut()) {
        (ServerMessage::GameSnapshot { observation, .. }, _) => *view = Some(observation.clone()),
        (ServerMessage::GameEvents { events, .. }, Some(view)) => {
            for event in events {
                view.fold(event);
            }
        }
        (ServerMessage::GameReveal { reveal, .. }, Some(view)) => view.reveal(reveal),
        _ => {}
    }
}

/// The frame `sample` mutated: decoded (as a record's header for the header) without a panic,
/// round-tripping when it decodes, and folded by a client in place of the frame it was, the rest
/// of the record after it. Whether it reached the fold.
fn check_frame(sample: usize, mutations: &[Mutation]) -> Result<bool, TestCaseError> {
    let (_, messages) = &*RECORD;
    let sample = sample % FRAMES.len();
    let mut frame = FRAMES[sample].clone();
    for mutation in mutations {
        frame = mutation.apply(&frame);
    }
    if let Ok(header) = decode::<GameRecordHeader>(&frame, usize::MAX) {
        prop_assert_eq!(decode(&encode(&header), usize::MAX), Ok(header));
    }
    let Ok(message) = decode_server(&frame, usize::MAX) else {
        return Ok(false);
    };
    prop_assert_eq!(
        decode_server(&encode(&message), usize::MAX),
        Ok(message.clone())
    );
    // A record's message is folded in its place; a hand-built event after the snapshot.
    let at = match sample {
        0 => return Ok(false),
        k if k <= messages.len() => k - 1,
        _ => 1,
    };
    let mut view = None;
    for m in &messages[..at] {
        client(&mut view, m);
    }
    client(&mut view, &message);
    for m in &messages[at.min(messages.len())..] {
        client(&mut view, m);
    }
    Ok(true)
}

/// The replay's first `point` commands as a journal's entries.
fn entries(point: usize) -> Vec<GameEntry> {
    let (_, _, replay) = &*REPLAY;
    (replay.commands[..point].iter())
        .map(|(actor, engine)| GameEntry::Command(*actor, engine.clone()))
        .collect()
}

/// The command `sample` mutated, played at the replay's command `point` by `who` (0 and 1: a
/// player, 2: the replay's next actor, 3: the system actor, 4: a player outside the game).
/// Whether it was accepted; none when it did not decode.
fn check_command(
    point: usize,
    sample: usize,
    who: u8,
    mutations: &[Mutation],
) -> Result<Option<bool>, TestCaseError> {
    let (rules, setup, replay) = &*REPLAY;
    let point = point % (replay.commands.len() + 1);
    let mut json = COMMANDS[sample % COMMANDS.len()].clone();
    for mutation in mutations {
        json = mutation.apply(&json);
    }
    let Ok(wire) = serde_json::from_slice::<GameCommand>(&json) else {
        return Ok(None);
    };
    prop_assert_eq!(
        &serde_json::from_slice::<GameCommand>(&encode(&wire)).unwrap(),
        &wire
    );
    let journal = entries(point);
    let restore = |entries: &[GameEntry]| {
        let start = manifest(rules, MAP);
        let mut out = Vec::new();
        GameSession::restore(rules, setup, replay.seed, start, OPTIONS, entries, &mut out).unwrap()
    };
    let mut session = restore(&journal);
    let next = replay
        .commands
        .get(point)
        .map_or(Actor::System, |(a, _)| *a);
    let actor = match who {
        0 | 1 => Actor::Player(PlayerId::new(who)),
        2 => next,
        3 => Actor::System,
        _ => Actor::Player(PlayerId::new(7)),
    };
    let hash = session.state().state_hash();
    let mut out = Vec::new();
    let played = match actor {
        Actor::Player(player) => session.play(player, 11, wire.clone(), &mut out),
        Actor::System => session.system(wire.clone(), &mut out),
    };
    let drained: Vec<GameEntry> = session.drain_entries().collect();
    let accepted = played.is_ok();
    match played {
        Err(code) => {
            prop_assert_eq!(session.state().state_hash(), hash);
            prop_assert_eq!(session.journal().commands.len(), point);
            prop_assert!(drained.is_empty());
            let told: Vec<GameOutbound> = match actor {
                Actor::Player(player) if player.index() < session.state().players() => {
                    let n = match &session
                        .record(player)
                        .unwrap()
                        .lines()
                        .last()
                        .map(|l| decode_server(l.as_bytes(), usize::MAX).unwrap())
                    {
                        Some(ServerMessage::GameRejected { n, .. }) => *n,
                        other => return Err(TestCaseError::fail(std::format!("{other:?}"))),
                    };
                    let frame = encode_text(&ServerMessage::GameRejected { n, seq: 11, code });
                    vec![GameOutbound { to: player, frame }]
                }
                _ => Vec::new(),
            };
            prop_assert_eq!(out, told);
        }
        Ok(()) => {
            // Journaled as the engine's command it was mapped to: a restore re-runs it.
            prop_assert_eq!(drained.len(), 1);
            let mut journal = journal;
            journal.extend(drained);
            prop_assert_eq!(
                restore(&journal).state().state_hash(),
                session.state().state_hash()
            );
        }
    }
    Ok(Some(accepted))
}

proptest! {
    #[test]
    fn mutated_game_frames_decode_and_fold_without_a_panic(
        sample in 0usize..1 << 16,
        mutations in proptest::collection::vec(arb_mutation(), 1..4),
    ) {
        check_frame(sample, &mutations)?;
    }

    /// Values replaced in a game's frame: most still parse, so the fold sees them.
    #[test]
    fn game_frames_with_extreme_values_fold_without_a_panic(
        sample in 0usize..1 << 16,
        mutations in proptest::collection::vec(arb_value_mutation(), 1..4),
    ) {
        check_frame(sample, &mutations)?;
    }

    #[test]
    fn mutated_game_commands_are_played_or_rejected_without_a_trace(
        point in 0usize..1 << 16,
        sample in 0usize..1 << 16,
        who in 0u8..5,
        mutations in proptest::collection::vec(arb_value_mutation(), 0..4),
    ) {
        check_command(point, sample, who, &mutations)?;
    }
}

#[test]
fn the_samples_hold_every_kind_of_a_games_frame() {
    let (_, messages) = &*RECORD;
    let kinds: std::collections::BTreeSet<String> = messages
        .iter()
        .map(|m| {
            let text = encode_text(m);
            String::from(text.split('"').nth(1).unwrap())
        })
        .collect();
    let all = [
        "GameSnapshot",
        "GameEvents",
        "GameReveal",
        "GameTurn",
        "GameAccepted",
        "Snapshot",
    ];
    for kind in all {
        assert!(kinds.contains(kind), "{kind}: {kinds:?}");
    }
    // The hand-built events and commands of every kind join them.
    assert!(FRAMES.len() > messages.len() + 30);
    assert!(COMMANDS.len() > every_command().len() + 200);
}

#[test]
fn the_mutations_reach_the_fold_and_the_session() {
    // A number replaced in every fifth frame: a plausible one still decodes, and folds.
    let (mut folded, mut tried) = (0, 0);
    for sample in (1..FRAMES.len()).step_by(5) {
        for to in [0, 3, 9, 13] {
            let mutation = Mutation::Number { which: sample, to };
            folded += usize::from(check_frame(sample, &[mutation]).unwrap());
            tried += 1;
        }
    }
    assert!(folded * 3 > tried, "{folded} of {tried}");
    // The replay's own command by its actor is accepted; with a number replaced, some are and
    // some are not; by a player outside the game, none is.
    let (_, _, replay) = &*REPLAY;
    let own = every_command().len();
    let (mut accepted, mut rejected) = (0, 0);
    for point in (0..replay.commands.len()).step_by(10) {
        assert_eq!(
            check_command(point, own + point, 2, &[]).unwrap(),
            Some(true)
        );
        for to in [0, 1, 3, 9] {
            let mutation = Mutation::Number { which: point, to };
            match check_command(point, own + point, 2, &[mutation]).unwrap() {
                Some(true) => accepted += 1,
                Some(false) => rejected += 1,
                None => {}
            }
        }
        assert_eq!(
            check_command(point, own + point, 4, &[]).unwrap(),
            Some(false)
        );
    }
    assert!(
        accepted > 0 && rejected > 0,
        "{accepted} accepted, {rejected} rejected"
    );
}
