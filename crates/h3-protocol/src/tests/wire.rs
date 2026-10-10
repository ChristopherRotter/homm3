//! The wire's format: every message round-trips in JSON and in a format that is not
//! self-describing (`postcard`), the JSON forms are serde's default enum form, 64-bit values are
//! strings, and unknown fields are refused.

use std::string::{String, ToString};
use std::vec;
use std::vec::Vec;

use h3_battle::BattleSide;
use serde::Serialize;
use serde::de::DeserializeOwned;

use super::battle;
use crate::PROTOCOL_VERSION;
use crate::map;
use crate::wire::*;

/// One of every command.
pub(super) fn commands() -> Vec<Command> {
    vec![
        Command::Move { to: 3 },
        Command::Attack {
            target: 9,
            from: 20,
        },
        Command::Shoot { target: 9, at: 21 },
        Command::Wait {},
        Command::Defend {},
        Command::Heal { target: 2 },
        Command::EndTactics {},
        Command::Retreat {},
        Command::Surrender {},
        Command::Cast {
            spell: 4,
            target: Target::Stack { stack: 8 },
            second: Target::Hex { hex: 40 },
        },
        Command::CreatureCast {
            target: Target::None {},
        },
    ]
}

fn client_messages() -> Vec<ClientMessage> {
    let mut out = vec![
        ClientMessage::Hello {
            protocol: PROTOCOL_VERSION,
            client: "test".to_string(),
            name: "Ann".to_string(),
        },
        ClientMessage::Join {
            game: 1,
            seat: Seat::Attacker,
            token: None,
        },
        ClientMessage::Join {
            game: 1,
            seat: Seat::Defender,
            token: Some("00ff".to_string()),
        },
        ClientMessage::Join {
            game: 2,
            seat: Seat::Spectator,
            token: None,
        },
        ClientMessage::Query {
            seq: 8,
            query: Query::CastTargets { spell: 2 },
        },
        ClientMessage::Query {
            seq: 9,
            query: Query::SecondTargets {
                spell: 2,
                target: Target::Stack { stack: 1 },
            },
        },
        ClientMessage::FetchReplay { seq: 10 },
        ClientMessage::FetchRecord { seq: 11 },
        ClientMessage::Leave {},
        ClientMessage::Ping { nonce: 5 },
    ];
    out.extend(
        commands()
            .into_iter()
            .map(|command| ClientMessage::Command { seq: 7, command }),
    );
    out
}

fn server_messages() -> Vec<ServerMessage> {
    let (ruleset, state) = battle();
    let manifest = Manifest {
        protocol: PROTOCOL_VERSION,
        engine: "0.1.0".to_string(),
        packs: vec![PackRef {
            id: "synthetic".to_string(),
            version: "1".to_string(),
        }],
        profile: "synthetic:faithful".to_string(),
        ruleset_hash: WireU64(u64::MAX),
        rng: Algorithm {
            name: "pcg".to_string(),
            version: 1,
        },
        hash: Algorithm {
            name: "fx".to_string(),
            version: 1,
        },
        battle_state_hash_version: 40,
        field: map::field(&ruleset),
        keys: map::key_tables(&ruleset),
    };
    let mut out = vec![
        ServerMessage::Welcome {
            protocol: PROTOCOL_VERSION,
            server: "h3-server 0.1.0".to_string(),
        },
        ServerMessage::Joined {
            game: 1,
            seat: Seat::Attacker,
            token: Some("00ff".to_string()),
            manifest,
        },
        ServerMessage::Snapshot {
            n: 1,
            observation: map::observe(&state, &ruleset, BattleSide::Attacker),
        },
        ServerMessage::YourTurn {
            n: 2,
            legal: commands(),
            castable: vec![1, 4],
        },
        ServerMessage::Accepted {
            n: 3,
            seq: 7,
            command: Command::Wait {},
        },
        ServerMessage::Rejected {
            n: 4,
            seq: 8,
            code: RejectCode::NotYourTurn,
        },
        ServerMessage::QueryResult {
            n: 6,
            seq: 9,
            targets: vec![
                Target::None {},
                Target::Stack { stack: 3 },
                Target::Hex { hex: 7 },
            ],
        },
        ServerMessage::Events {
            n: 7,
            events: vec![
                Event::Struck {
                    attacker: 1,
                    target: 9,
                    damage: WireU64(1 << 60),
                    kills: 3,
                    flags: Strike {
                        lucky: true,
                        ..Strike::default()
                    },
                },
                Event::BattleEnded {
                    result: Outcome::Draw {},
                },
            ],
        },
        ServerMessage::GameEnded {
            n: 8,
            result: Outcome::Victory {
                side: Side::Defender,
            },
        },
        ServerMessage::Replay {
            n: 9,
            seq: 10,
            replay: "[header]".to_string(),
        },
        ServerMessage::Record {
            n: 9,
            seq: 11,
            record: "{}".to_string(),
        },
        ServerMessage::Pong { nonce: 5 },
    ];
    out.extend(
        [
            Refusal::NoSuchGame,
            Refusal::SeatTaken,
            Refusal::BadToken,
            Refusal::AlreadyJoined,
            Refusal::NoSpectators,
        ]
        .map(|reason| ServerMessage::Refused { reason }),
    );
    out.extend(
        [
            GoodbyeReason::Protocol,
            GoodbyeReason::Malformed,
            GoodbyeReason::TooLarge,
            GoodbyeReason::RateLimited,
            GoodbyeReason::IdleTimeout,
            GoodbyeReason::TooSlow,
            GoodbyeReason::Left,
            GoodbyeReason::Replaced,
            GoodbyeReason::Shutdown,
        ]
        .map(|reason| ServerMessage::Goodbye { reason }),
    );
    out
}

fn round_trips<T: Serialize + DeserializeOwned + PartialEq + core::fmt::Debug>(message: &T) {
    let json = serde_json::to_vec(message).unwrap();
    assert_eq!(&serde_json::from_slice::<T>(&json).unwrap(), message);
    let binary = postcard::to_allocvec(message).unwrap();
    assert_eq!(&postcard::from_bytes::<T>(&binary).unwrap(), message);
}

#[test]
fn every_message_round_trips_in_json_and_in_postcard() {
    for m in client_messages() {
        round_trips(&m);
    }
    for m in server_messages() {
        round_trips(&m);
    }
}

#[test]
fn the_json_is_serdes_default_enum_form() {
    let text = |m: &ClientMessage| String::from_utf8(serde_json::to_vec(m).unwrap()).unwrap();
    assert_eq!(
        text(&ClientMessage::Command {
            seq: 7,
            command: Command::Move { to: 3 }
        }),
        r#"{"Command":{"seq":7,"command":{"Move":{"to":3}}}}"#
    );
    assert_eq!(
        text(&ClientMessage::Join {
            game: 1,
            seat: Seat::Defender,
            token: None
        }),
        r#"{"Join":{"game":1,"seat":"Defender","token":null}}"#
    );
    assert_eq!(text(&ClientMessage::Leave {}), r#"{"Leave":{}}"#);
}

#[test]
fn a_64_bit_value_is_a_decimal_string() {
    let damage = Event::Reflected {
        stack: 1,
        damage: WireU64(u64::MAX),
        kills: 0,
    };
    let json = String::from_utf8(serde_json::to_vec(&damage).unwrap()).unwrap();
    assert_eq!(
        json,
        r#"{"Reflected":{"stack":1,"damage":"18446744073709551615","kills":0}}"#
    );
    assert_eq!(WireU64(42).to_string(), "42");
    // A number, a word or a value beyond 64 bits is refused.
    for bad in [r#"42"#, r#""forty""#, r#""18446744073709551616""#] {
        assert!(serde_json::from_str::<WireU64>(bad).is_err(), "{bad}");
    }
}

#[test]
fn unknown_fields_are_refused() {
    // A command names no side: the actor is the seat (ADR-0018 point 6).
    let with_side = r#"{"Command":{"seq":7,"command":{"Move":{"to":3,"side":"Attacker"}}}}"#;
    assert!(serde_json::from_str::<ClientMessage>(with_side).is_err());
    let extra = r#"{"Ping":{"nonce":1,"admin":true}}"#;
    assert!(serde_json::from_str::<ClientMessage>(extra).is_err());
    let unknown = r#"{"Shutdown":{}}"#;
    assert!(serde_json::from_str::<ClientMessage>(unknown).is_err());
}
