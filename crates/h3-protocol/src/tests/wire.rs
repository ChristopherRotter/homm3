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

/// One of every client message (several of some).
pub(super) fn client_messages() -> Vec<ClientMessage> {
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
        ClientMessage::ListGames { seq: 12 },
        ClientMessage::CreateGame {
            seq: 13,
            offer: "duel".to_string(),
            seat: Seat::Defender,
            spectators: None,
        },
        ClientMessage::CreateGame {
            seq: 13,
            offer: "duel".to_string(),
            seat: Seat::Attacker,
            spectators: Some(false),
        },
        ClientMessage::Pause { seq: 14 },
        ClientMessage::SetSpectators {
            seq: 15,
            open: true,
        },
        ClientMessage::ListReplays { seq: 16, from: 50 },
        ClientMessage::FetchArchived {
            seq: 17,
            replay: "duel-game3-00000000000000ff".to_string(),
        },
        ClientMessage::Login {
            seq: 18,
            login: "ann".to_string(),
            key: "11".repeat(32),
            signature: "22".repeat(64),
            display: None,
        },
        ClientMessage::Login {
            seq: 19,
            login: "ann_1".to_string(),
            key: "11".repeat(32),
            signature: "22".repeat(64),
            display: Some("Ann Smith".to_string()),
        },
        ClientMessage::Rekey {
            seq: 20,
            key: "33".repeat(32),
            by_old: "44".repeat(64),
            by_new: "55".repeat(64),
        },
        ClientMessage::Rename {
            seq: 21,
            display: "Ann B".to_string(),
        },
        ClientMessage::SetListed {
            seq: 22,
            listed: false,
        },
        ClientMessage::ListLadders { seq: 23 },
        ClientMessage::Queue {
            seq: 24,
            ladder: "arena".to_string(),
        },
        ClientMessage::Dequeue { seq: 25 },
        ClientMessage::AcceptMatch { seq: 26 },
        ClientMessage::DeclineMatch { seq: 27 },
        // E16 M4 T4.
        ClientMessage::Leaderboard {
            seq: 28,
            ladder: "arena".to_string(),
            season: None,
            page: 0,
        },
        ClientMessage::Leaderboard {
            seq: 29,
            ladder: "arena".to_string(),
            season: Some(2),
            page: 3,
        },
        ClientMessage::Standing {
            seq: 30,
            ladder: "arena".to_string(),
        },
        // E16 M4 T6.
        ClientMessage::ProposePause {
            seq: 31,
            ms: 300_000,
        },
        ClientMessage::AnswerPause {
            seq: 32,
            accept: true,
        },
    ];
    out.extend(
        commands()
            .into_iter()
            .map(|command| ClientMessage::Command { seq: 7, command }),
    );
    out
}

/// A leaderboard's row at `rank` (E16 M4 T4).
fn row(rank: u32, bot: bool) -> LeaderboardRow {
    LeaderboardRow {
        rank,
        display: "Ann B".to_string(),
        rating: Points {
            rating: 1662,
            deviation: 290,
        },
        games: 3,
        wins: 2,
        losses: 0,
        draws: 1,
        bot,
    }
}

/// One of every server message (several of some).
pub(super) fn server_messages() -> Vec<ServerMessage> {
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
            id: "wss://h3.example.org".to_string(),
            challenge: "07".repeat(32),
        },
        ServerMessage::Account {
            seq: 18,
            display: "Ann Smith".to_string(),
            listed: true,
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
        ServerMessage::Games {
            seq: 12,
            offers: vec![map::offer("duel", "duel", &ruleset, vec![Seat::Defender])],
            games: vec![
                GameInfo {
                    game: 3,
                    offer: "duel".to_string(),
                    phase: Phase::Waiting,
                    open: vec![Seat::Attacker],
                    spectators: true,
                    seated: vec![
                        Seated {
                            seat: Seat::Defender,
                            display: "Alice".to_string(),
                            bot: false,
                        },
                        // E16 M4 T5: a bot account's seat.
                        Seated {
                            seat: Seat::Attacker,
                            display: "Randy".to_string(),
                            bot: true,
                        },
                    ],
                    ladder: None,
                },
                GameInfo {
                    game: 4,
                    offer: "duel".to_string(),
                    phase: Phase::Paused,
                    open: vec![],
                    spectators: false,
                    seated: vec![],
                    ladder: Some("arena".to_string()),
                },
            ],
        },
        ServerMessage::GameCreated { seq: 13, game: 3 },
        ServerMessage::PauseAsked {
            n: 10,
            seat: Seat::Defender,
        },
        ServerMessage::Rejected {
            n: 11,
            seq: 10,
            code: RejectCode::Unverified,
        },
        ServerMessage::SpectatorsSet {
            n: 12,
            seq: 15,
            open: false,
        },
        ServerMessage::Replays {
            seq: 16,
            total: 51,
            replays: vec![ReplayInfo {
                id: "duel-game3-00000000000000ff".to_string(),
                offer: "duel".to_string(),
                game: 3,
            }],
        },
        ServerMessage::Refused {
            reason: Refusal::NoSuchReplay,
        },
    ];
    // E16 M4 T3a: a ranked game's ends by absence.
    out.extend(
        [
            Outcome::Forfeit {
                side: Side::Attacker,
            },
            Outcome::NoShow {},
            // E16 M4 T3b.
            Outcome::Unverified {},
        ]
        .map(|result| ServerMessage::GameEnded { n: 14, result }),
    );
    // E16 M4 T3b: a seat's rating after its ranked game.
    out.push(ServerMessage::Rating {
        n: 15,
        ladder: "arena".to_string(),
        before: Points {
            rating: 1500,
            deviation: 350,
        },
        after: Points {
            rating: -7,
            deviation: 0,
        },
    });
    // E16 M4 T6: a pause proposed and answered.
    out.extend([
        ServerMessage::PauseProposed {
            n: 16,
            seat: Seat::Attacker,
            ms: 300_000,
        },
        ServerMessage::PauseAnswered {
            n: 17,
            seat: Seat::Defender,
            accept: false,
        },
    ]);
    // E16 M4 T4: a leaderboard's page, one's own row.
    out.extend([
        ServerMessage::LeaderboardPage {
            seq: 28,
            ladder: "arena".to_string(),
            season: 1,
            total: 2,
            rows: vec![row(1, false), row(2, true)],
        },
        ServerMessage::LeaderboardPage {
            seq: 29,
            ladder: "arena".to_string(),
            season: 2,
            total: 0,
            rows: vec![],
        },
        ServerMessage::OwnStanding {
            seq: 30,
            ladder: "arena".to_string(),
            season: 1,
            total: 2,
            row: Some(row(2, true)),
        },
        ServerMessage::OwnStanding {
            seq: 31,
            ladder: "arena".to_string(),
            season: 1,
            total: 2,
            row: None,
        },
    ]);
    out.extend(
        [
            RejectCode::NotHost,
            RejectCode::AlreadyStarted,
            // E16 M4 T6.
            RejectCode::NotRanked,
            RejectCode::AlreadyAsked,
            RejectCode::NoPausesLeft,
            RejectCode::BadPauseLength,
            RejectCode::NoProposal,
        ]
        .map(|code| ServerMessage::Rejected {
            n: 13,
            seq: 15,
            code,
        }),
    );
    out.extend(
        [
            Refusal::NoSuchGame,
            Refusal::SeatTaken,
            Refusal::BadToken,
            Refusal::AlreadyJoined,
            Refusal::NoSpectators,
            Refusal::NoSuchOffer,
            Refusal::TooManyGames,
            Refusal::Draining,
            Refusal::BadLogin,
            Refusal::BadName,
            Refusal::NameTaken,
            Refusal::NotLoggedIn,
            Refusal::AlreadyLoggedIn,
            Refusal::NoSuchLadder,
            Refusal::AlreadyQueued,
            Refusal::InRankedGame,
            Refusal::NotQueued,
            Refusal::NoMatch,
            Refusal::NoSuchSeason,
        ]
        .map(|reason| ServerMessage::Refused { reason }),
    );
    out.extend([
        ServerMessage::Ladders {
            seq: 23,
            ladders: vec![LadderInfo {
                id: "arena".to_string(),
                name: "The Arena".to_string(),
                season: 2,
                waiting: 3,
                bots: vec!["Randy".to_string()],
            }],
        },
        ServerMessage::Queued {
            seq: 24,
            ladder: "arena".to_string(),
        },
        ServerMessage::Dequeued { seq: 25 },
        ServerMessage::MatchFound {
            ladder: "arena".to_string(),
            accept_ms: 30_000,
        },
        ServerMessage::MatchCancelled {},
        ServerMessage::MatchReady {
            ladder: "arena".to_string(),
            game: 9,
            seat: Seat::Defender,
        },
    ]);
    out.extend(
        [
            UnqueueReason::Expired,
            UnqueueReason::Draining,
            UnqueueReason::Full,
        ]
        .map(|reason| ServerMessage::Unqueued { reason }),
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
            GoodbyeReason::Paused,
            GoodbyeReason::SpectatorsClosed,
            GoodbyeReason::Rekeyed,
            GoodbyeReason::CommandLimit,
            GoodbyeReason::Abandoned,
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
    assert_eq!(
        text(&ClientMessage::Pause { seq: 4 }),
        r#"{"Pause":{"seq":4}}"#
    );
    assert_eq!(
        text(&ClientMessage::CreateGame {
            seq: 2,
            offer: "duel".to_string(),
            seat: Seat::Attacker,
            spectators: Some(false),
        }),
        r#"{"CreateGame":{"seq":2,"offer":"duel","seat":"Attacker","spectators":false}}"#
    );
    // A creation that does not say takes spectators (E16 M2 T4c).
    assert_eq!(
        serde_json::from_str::<ClientMessage>(
            r#"{"CreateGame":{"seq":2,"offer":"duel","seat":"Attacker"}}"#
        )
        .unwrap(),
        ClientMessage::CreateGame {
            seq: 2,
            offer: "duel".to_string(),
            seat: Seat::Attacker,
            spectators: None,
        }
    );
    assert_eq!(
        text(&ClientMessage::SetSpectators {
            seq: 3,
            open: false
        }),
        r#"{"SetSpectators":{"seq":3,"open":false}}"#
    );
    let info = GameInfo {
        game: 3,
        offer: "duel".to_string(),
        phase: Phase::Running,
        open: vec![],
        spectators: true,
        seated: vec![Seated {
            seat: Seat::Attacker,
            display: "Alice".to_string(),
            bot: false,
        }],
        ladder: Some("arena".to_string()),
    };
    assert_eq!(
        serde_json::to_string(&info).unwrap(),
        r#"{"game":3,"offer":"duel","phase":"Running","open":[],"spectators":true,"seated":[{"seat":"Attacker","display":"Alice","bot":false}],"ladder":"arena"}"#
    );
    // E16 M4 T2: the ladders' messages.
    assert_eq!(
        text(&ClientMessage::Queue {
            seq: 4,
            ladder: "arena".to_string()
        }),
        r#"{"Queue":{"seq":4,"ladder":"arena"}}"#
    );
    let server = |m: &ServerMessage| String::from_utf8(serde_json::to_vec(m).unwrap()).unwrap();
    assert_eq!(
        server(&ServerMessage::MatchFound {
            ladder: "arena".to_string(),
            accept_ms: 30_000
        }),
        r#"{"MatchFound":{"ladder":"arena","accept_ms":30000}}"#
    );
    assert_eq!(
        server(&ServerMessage::Unqueued {
            reason: UnqueueReason::Expired
        }),
        r#"{"Unqueued":{"reason":"Expired"}}"#
    );
    // E16 M4 T3a: a forfeit and a no-show.
    assert_eq!(
        server(&ServerMessage::GameEnded {
            n: 3,
            result: Outcome::Forfeit {
                side: Side::Defender
            }
        }),
        r#"{"GameEnded":{"n":3,"result":{"Forfeit":{"side":"Defender"}}}}"#
    );
    assert_eq!(
        server(&ServerMessage::GameEnded {
            n: 4,
            result: Outcome::NoShow {}
        }),
        r#"{"GameEnded":{"n":4,"result":{"NoShow":{}}}}"#
    );
    // E16 M4 T3b: a void ranked game, a rating.
    assert_eq!(
        server(&ServerMessage::GameEnded {
            n: 5,
            result: Outcome::Unverified {}
        }),
        r#"{"GameEnded":{"n":5,"result":{"Unverified":{}}}}"#
    );
    assert_eq!(
        server(&ServerMessage::Rating {
            n: 6,
            ladder: "arena".to_string(),
            before: Points {
                rating: 1500,
                deviation: 350
            },
            after: Points {
                rating: 1662,
                deviation: 290
            }
        }),
        r#"{"Rating":{"n":6,"ladder":"arena","before":{"rating":1500,"deviation":350},"after":{"rating":1662,"deviation":290}}}"#
    );
    // E16 M4 T6: a proposal's length is a plain number (at most a year: far below 2^53).
    assert_eq!(
        text(&ClientMessage::ProposePause {
            seq: 1,
            ms: 600_000
        }),
        r#"{"ProposePause":{"seq":1,"ms":600000}}"#
    );
    // E16 M4 T4: a leaderboard's page; the season left out is the current one.
    assert_eq!(
        serde_json::from_str::<ClientMessage>(
            r#"{"Leaderboard":{"seq":1,"ladder":"arena","page":0}}"#
        )
        .unwrap(),
        ClientMessage::Leaderboard {
            seq: 1,
            ladder: "arena".to_string(),
            season: None,
            page: 0
        }
    );
    assert_eq!(
        server(&ServerMessage::LeaderboardPage {
            seq: 1,
            ladder: "arena".to_string(),
            season: 2,
            total: 1,
            rows: vec![row(1, false)]
        }),
        r#"{"LeaderboardPage":{"seq":1,"ladder":"arena","season":2,"total":1,"rows":[{"rank":1,"display":"Ann B","rating":{"rating":1662,"deviation":290},"games":3,"wins":2,"losses":0,"draws":1,"bot":false}]}}"#
    );
    // E16 M3 T1: a login, keys and signatures as hex.
    assert_eq!(
        text(&ClientMessage::Login {
            seq: 1,
            login: "ann".to_string(),
            key: "ab".to_string(),
            signature: "cd".to_string(),
            display: None,
        }),
        r#"{"Login":{"seq":1,"login":"ann","key":"ab","signature":"cd","display":null}}"#
    );
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
