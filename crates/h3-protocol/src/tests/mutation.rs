//! The decoder under mutated frames (E16 M2 T5d, SRV-Q14: mutation tests on the stable toolchain
//! instead of a coverage-guided fuzzer for now). Every valid message of the wire tests, encoded,
//! is mutated a few times (a flipped byte, a cut, an insertion, a repeat, a splice with another
//! message, a number or a string replaced by an extreme one); the decoders never panic, a frame
//! that still decodes encodes back to the same message, and a client message the hello gate lets
//! through is handled by a running game's session without a panic.
//!
//! A longer run: `PROPTEST_CASES=100000 cargo test -p h3-protocol mutation`.

use std::string::{String, ToString};
use std::sync::LazyLock;
use std::vec::Vec;

use h3_battle::BattleSetup;
use h3_content_battle::BattleRuleset;
use proptest::prelude::*;

use super::full;
use super::wire::{client_messages, server_messages};
use crate::PROTOCOL_VERSION;
use crate::codec::{DEFAULT_MAX_CLIENT_FRAME, decode_client, decode_server, encode};
use crate::connection::{Connection, Host, Received};
use crate::session::{ConnId, GameConfig, SeatKind, Session};
use crate::wire::{ClientMessage, Seat};

/// Every sample message of the wire tests, encoded: the client's, then the server's.
static FRAMES: LazyLock<Vec<Vec<u8>>> = LazyLock::new(|| {
    let client = CLIENT.iter().cloned();
    let server = server_messages().into_iter().map(|m| encode(&m));
    client.chain(server).collect()
});

/// The client's sample messages, encoded.
static CLIENT: LazyLock<Vec<Vec<u8>>> =
    LazyLock::new(|| client_messages().into_iter().map(|m| encode(&m)).collect());

/// The `full` scenario, once.
static FULL: LazyLock<(BattleRuleset, BattleSetup)> = LazyLock::new(full);

/// Numbers a mutation puts in place of one in the frame: plausible ones (an id or a hex out of
/// range) and extreme ones.
const NUMBERS: [&str; 17] = [
    "1",
    "2",
    "7",
    "42",
    "186",
    "255",
    "256",
    "65535",
    "0",
    "-1",
    "-0",
    "1.5",
    "1e999",
    "4294967296",
    "18446744073709551616",
    "00",
    "99999999999999999999999999999999999999",
];

/// Strings a mutation puts in place of one in the frame (their JSON text, without the quotes).
const STRINGS: [&str; 7] = [
    "",
    "\\u0000",
    "\\ud800",
    "\\\"",
    "é漢🂡",
    "Hello",
    "999999999999999999999",
];

/// One change to a frame; positions are taken modulo the frame's length.
#[derive(Debug, Clone)]
enum Mutation {
    /// The byte at `at` XOR `mask` (not 0).
    Flip { at: usize, mask: u8 },
    /// `len` bytes from `at` removed.
    Cut { at: usize, len: usize },
    /// `bytes` inserted at `at`.
    Insert { at: usize, bytes: Vec<u8> },
    /// The `len` bytes from `at` repeated in place.
    Repeat { at: usize, len: usize },
    /// The frame up to `at`, then another sample frame's from `from`.
    Splice {
        other: usize,
        at: usize,
        from: usize,
    },
    /// The `which`-th run of digits replaced by `NUMBERS[to]`.
    Number { which: usize, to: usize },
    /// The `which`-th string's content replaced by `STRINGS[to]`.
    Text { which: usize, to: usize },
}

/// A mutation of a value only, which keeps the frame's shape: most such frames still parse.
fn arb_value_mutation() -> impl Strategy<Value = Mutation> {
    prop_oneof![
        (0usize..32, 0..NUMBERS.len()).prop_map(|(which, to)| Mutation::Number { which, to }),
        (0usize..32, 0..STRINGS.len()).prop_map(|(which, to)| Mutation::Text { which, to }),
    ]
}

fn arb_mutation() -> impl Strategy<Value = Mutation> {
    let anywhere = 0usize..4096;
    prop_oneof![
        (anywhere.clone(), 1u8..=255).prop_map(|(at, mask)| Mutation::Flip { at, mask }),
        (anywhere.clone(), 1usize..16).prop_map(|(at, len)| Mutation::Cut { at, len }),
        (
            anywhere.clone(),
            proptest::collection::vec(any::<u8>(), 1..8)
        )
            .prop_map(|(at, bytes)| Mutation::Insert { at, bytes }),
        (anywhere.clone(), 1usize..32).prop_map(|(at, len)| Mutation::Repeat { at, len }),
        (0usize..1024, anywhere.clone(), anywhere)
            .prop_map(|(other, at, from)| { Mutation::Splice { other, at, from } }),
        (0usize..32, 0..NUMBERS.len()).prop_map(|(which, to)| Mutation::Number { which, to }),
        (0usize..32, 0..STRINGS.len()).prop_map(|(which, to)| Mutation::Text { which, to }),
    ]
}

/// The byte ranges of the frame's runs of digits, or of its strings' contents (`strings`).
fn runs(frame: &[u8], strings: bool) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < frame.len() {
        if strings && frame[i] == b'"' {
            let start = i + 1;
            let mut end = start;
            while end < frame.len() && frame[end] != b'"' {
                end += if frame[end] == b'\\' { 2 } else { 1 };
            }
            let end = end.min(frame.len());
            out.push((start, end));
            i = end + 1;
        } else if !strings && frame[i].is_ascii_digit() {
            let start = i;
            while i < frame.len() && frame[i].is_ascii_digit() {
                i += 1;
            }
            out.push((start, i));
        } else {
            i += 1;
        }
    }
    out
}

impl Mutation {
    /// The frame changed.
    fn apply(&self, frame: &[u8]) -> Vec<u8> {
        let at = |i: usize| if frame.is_empty() { 0 } else { i % frame.len() };
        let mut out = frame.to_vec();
        match self {
            Self::Flip { at: i, mask } => {
                if let Some(b) = out.get_mut(at(*i)) {
                    *b ^= mask;
                }
            }
            Self::Cut { at: i, len } => {
                let start = at(*i);
                let end = (start + len).min(out.len());
                out.drain(start..end);
            }
            Self::Insert { at: i, bytes } => {
                let start = at(*i);
                out.splice(start..start, bytes.iter().copied());
            }
            Self::Repeat { at: i, len } => {
                let start = at(*i);
                let end = (start + len).min(out.len());
                let copy = out[start..end].to_vec();
                out.splice(end..end, copy);
            }
            Self::Splice { other, at: i, from } => {
                let other = &FRAMES[other % FRAMES.len()];
                let from = if other.is_empty() {
                    0
                } else {
                    from % other.len()
                };
                out.truncate(at(*i));
                out.extend_from_slice(&other[from..]);
            }
            Self::Number { which, to } | Self::Text { which, to } => {
                let strings = matches!(self, Self::Text { .. });
                let found = runs(frame, strings);
                if let Some(&(start, end)) = found.get(which % found.len().max(1)) {
                    let with = if strings { STRINGS[*to] } else { NUMBERS[*to] };
                    out.splice(start..end, with.bytes());
                }
            }
        }
        out
    }
}

/// A running game of the `full` scenario, both seats held (connections 1 and 2).
fn running() -> Session<'static> {
    let (ruleset, setup) = &*FULL;
    let config = GameConfig {
        game: 1,
        setup: setup.clone(),
        seed: 3,
        checkpoint_every: 10,
        shadow: false,
        spectators: 2,
        watchable: true,
        record_bytes: 1 << 16,
        ranked: None,
    };
    let seats = ["a", "b"].map(|token| SeatKind::Player {
        token: token.to_string(),
    });
    let mut session = Session::new(ruleset, config, seats).unwrap();
    let mut out = Vec::new();
    for (conn, seat) in [(1, Seat::Attacker), (2, Seat::Defender)] {
        let join = ClientMessage::Join {
            game: 1,
            seat,
            token: None,
        };
        session.handle(ConnId(conn), join, &mut out);
    }
    session
}

/// The decoders and the hello gate on `frame`, and a running game on what the gate lets through:
/// no panic, and a frame that decodes encodes back to the same message.
fn check(frame: &[u8]) -> Result<(), TestCaseError> {
    if let Ok(message) = decode_client(frame, usize::MAX) {
        prop_assert_eq!(decode_client(&encode(&message), usize::MAX), Ok(message));
    }
    if let Ok(message) = decode_server(frame, usize::MAX) {
        prop_assert_eq!(decode_server(&encode(&message), usize::MAX), Ok(message));
    }
    // Through the hello gate into a running game, as the host passes it.
    let mut gate = Connection::new(
        DEFAULT_MAX_CLIENT_FRAME,
        [7; 32],
        frame.len().is_multiple_of(2),
    );
    let hello = ClientMessage::Hello {
        protocol: PROTOCOL_VERSION,
        client: String::new(),
        name: String::new(),
    };
    let host = Host {
        program: "s",
        id: "ws://s",
    };
    gate.receive(&encode(&hello), &host);
    // A third of the frames reach a logged-in gate (E16 M3 T1), the account's requests too.
    if frame.len().is_multiple_of(3) {
        gate.logged_in(String::from("ann"));
    }
    if let Received::Game(message) = gate.receive(frame, &host) {
        let mut session = running();
        let mut out = Vec::new();
        session.handle(ConnId(1), message, &mut out);
    }
    Ok(())
}

proptest! {
    #[test]
    fn mutated_frames_never_panic_and_what_decodes_round_trips(
        sample in 0usize..1024,
        mutations in proptest::collection::vec(arb_mutation(), 1..4),
    ) {
        let mut frame = FRAMES[sample % FRAMES.len()].clone();
        for mutation in &mutations {
            frame = mutation.apply(&frame);
        }
        check(&frame)?;
    }

    /// Values replaced in a client's frame: most still parse, so the session sees them.
    #[test]
    fn client_frames_with_extreme_values_never_panic(
        sample in 0usize..1024,
        mutations in proptest::collection::vec(arb_value_mutation(), 1..4),
    ) {
        let mut frame = CLIENT[sample % CLIENT.len()].clone();
        for mutation in &mutations {
            frame = mutation.apply(&frame);
        }
        check(&frame)?;
    }
}

#[test]
fn every_mutation_changes_a_frame_in_its_way() {
    let frame = br#"{"Ping":{"nonce":95}}"#;
    let apply = |m: Mutation| String::from_utf8_lossy(&m.apply(frame)).into_owned();
    assert_eq!(
        apply(Mutation::Flip { at: 2, mask: 0x20 }),
        r#"{"ping":{"nonce":95}}"#
    );
    assert_eq!(apply(Mutation::Cut { at: 1, len: 7 }), r#"{{"nonce":95}}"#);
    assert_eq!(
        apply(Mutation::Insert {
            at: 0,
            bytes: b"[".to_vec()
        }),
        r#"[{"Ping":{"nonce":95}}"#
    );
    assert_eq!(
        apply(Mutation::Repeat { at: 0, len: 1 }),
        r#"{{"Ping":{"nonce":95}}"#
    );
    assert_eq!(
        apply(Mutation::Number { which: 0, to: 9 }),
        r#"{"Ping":{"nonce":-1}}"#
    );
    assert_eq!(
        apply(Mutation::Text { which: 1, to: 0 }),
        r#"{"Ping":{"":95}}"#
    );
    let spliced = apply(Mutation::Splice {
        other: 0,
        at: 3,
        from: 0,
    });
    assert!(spliced.starts_with(r#"{"P{"#), "{spliced}");
}
