//! End-to-end tests over loopback (E16 M1 T3): headless clients play hosted battles against each
//! other and against the bot, re-take a seat, and send what a hostile client sends; the replay the
//! server writes verifies as `h3 replay verify` does.

use std::path::PathBuf;
use std::time::Duration;

use h3_kernel::Rng;
use h3_protocol::wire::{ClientMessage, Command, GoodbyeReason, Refusal, Seat, ServerMessage};
use tokio_tungstenite::tungstenite::Message;

use crate::client::Client;
use crate::config::{BotSeat, Config, Limits};
use crate::{Running, game, start};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Limits for automated players, who answer every turn at once (a person sends far fewer than the
/// default 20 messages per second).
fn fast() -> Limits {
    Limits {
        messages_per_second: 100_000,
        ..Limits::default()
    }
}

/// A game played by test clients ends well within this.
const GAME_TIME: Duration = Duration::from_secs(60);

/// The synthetic `scenario` on a free loopback port, its data in a fresh directory.
fn config(scenario: &str, bots: Vec<BotSeat>, limits: Limits) -> Config {
    Config {
        bind: String::from("127.0.0.1:0"),
        packs: root().join("packs"),
        scenario: root().join("packs/synthetic/scenarios").join(scenario),
        profile: None,
        bots,
        data: std::env::temp_dir().join(format!("h3-server-{:016x}", game::os_u64().unwrap())),
        checkpoint_every: 10,
        limits,
    }
}

/// A client that said hello.
async fn greeted(running: &Running) -> Client {
    let mut client = Client::connect(running.address).await.unwrap();
    let welcome = client.hello("test").await.unwrap();
    assert!(
        matches!(welcome, Some(ServerMessage::Welcome { .. })),
        "{welcome:?}"
    );
    client
}

/// A client that took `seat`, with the seat's token and its snapshot.
async fn seated(running: &Running, seat: Seat) -> (Client, String) {
    let mut client = greeted(running).await;
    client.join(seat, None).await.unwrap();
    let joined = client.recv().await;
    let Some(ServerMessage::Joined {
        token: Some(token), ..
    }) = joined
    else {
        panic!("{joined:?}")
    };
    assert!(matches!(
        client.recv().await,
        Some(ServerMessage::Snapshot { .. })
    ));
    (client, token)
}

/// Plays a seat to the end of the game: every `YourTurn` is answered with a strike if there is
/// one, else with a command that stays in the battle; every message after the snapshot.
async fn play(mut client: Client, seed: u64) -> (Client, Vec<ServerMessage>) {
    let mut rng = Rng::from_seed(seed);
    let mut seen = Vec::new();
    let mut seq = 0;
    while let Some(message) = client.recv().await {
        if let ServerMessage::YourTurn { legal, .. } = &message {
            let strikes: Vec<Command> = legal
                .iter()
                .copied()
                .filter(|c| matches!(c, Command::Attack { .. } | Command::Shoot { .. }))
                .collect();
            let choices: Vec<Command> = if strikes.is_empty() {
                legal
                    .iter()
                    .copied()
                    .filter(|c| !matches!(c, Command::Retreat {} | Command::Surrender {}))
                    .collect()
            } else {
                strikes
            };
            let command = choices[rng.below(u32::try_from(choices.len()).unwrap()) as usize];
            seq += 1;
            client
                .send(&ClientMessage::Command { seq, command })
                .await
                .unwrap();
        }
        let ended = matches!(message, ServerMessage::GameEnded { .. });
        seen.push(message);
        if ended {
            break;
        }
    }
    (client, seen)
}

/// The goodbye a client gets, then the end of the connection.
async fn goodbye(client: &mut Client) -> GoodbyeReason {
    let Some(ServerMessage::Goodbye { reason }) = client.recv().await else {
        panic!("no goodbye")
    };
    assert_eq!(client.recv().await, None, "closed after the goodbye");
    reason
}

/// The replay a seat fetches after the game: the file the server wrote, which verifies.
async fn fetched_replay(client: &mut Client, running: &Running, packs: &std::path::Path) {
    client
        .send(&ClientMessage::FetchReplay { seq: 900 })
        .await
        .unwrap();
    let Some(ServerMessage::Replay {
        seq: 900, replay, ..
    }) = client.recv().await
    else {
        panic!("no replay")
    };
    let file = std::fs::read_to_string(&running.replay_path).unwrap();
    assert_eq!(replay, file);
    let verified = h3_cli::verify_file(&running.replay_path, packs).unwrap();
    assert!(verified.commands > 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn two_clients_play_a_hosted_battle_and_its_replay_verifies() {
    let config = config("full.toml", vec![], fast());
    let running = start(&config).await.unwrap();
    let (a, a_token) = seated(&running, Seat::Attacker).await;
    let (b, b_token) = seated(&running, Seat::Defender).await;
    assert_eq!(running.tokens, [Some(a_token), Some(b_token)]);
    let (a, b) = (tokio::spawn(play(a, 1)), tokio::spawn(play(b, 2)));
    let (mut a, a_seen) = tokio::time::timeout(GAME_TIME, a).await.unwrap().unwrap();
    let (_, b_seen) = tokio::time::timeout(GAME_TIME, b).await.unwrap().unwrap();
    let end = |seen: &[ServerMessage]| match seen.last() {
        Some(ServerMessage::GameEnded { result, .. }) => *result,
        other => panic!("{other:?}"),
    };
    assert_eq!(end(&a_seen), end(&b_seen));
    fetched_replay(&mut a, &running, &config.packs).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_client_plays_the_bot_on_a_field_of_random_obstacles() {
    let config = config("duel.toml", vec![BotSeat::Defender], fast());
    let running = start(&config).await.unwrap();
    assert!(running.tokens[1].is_none(), "the bot's seat has no token");
    let mut c = greeted(&running).await;
    c.join(Seat::Defender, None).await.unwrap();
    assert_eq!(
        c.recv().await,
        Some(ServerMessage::Refused {
            reason: Refusal::SeatTaken
        })
    );
    let (a, _) = seated(&running, Seat::Attacker).await;
    let (mut a, seen) = tokio::time::timeout(GAME_TIME, play(a, 3)).await.unwrap();
    assert!(matches!(seen.last(), Some(ServerMessage::GameEnded { .. })));
    // The obstacles came from their own seed and travel in the replay's setup.
    fetched_replay(&mut a, &running, &config.packs).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_dropped_client_retakes_its_seat_with_its_token() {
    let config = config("full.toml", vec![], Limits::default());
    let running = start(&config).await.unwrap();
    let (a, a_token) = seated(&running, Seat::Attacker).await;
    let (_b, _) = seated(&running, Seat::Defender).await;
    a.close().await;
    let mut thief = greeted(&running).await;
    thief.join(Seat::Attacker, None).await.unwrap();
    assert_eq!(
        thief.recv().await,
        Some(ServerMessage::Refused {
            reason: Refusal::SeatTaken
        })
    );
    let mut back = greeted(&running).await;
    back.join(Seat::Attacker, Some(a_token.clone()))
        .await
        .unwrap();
    assert!(matches!(
        back.recv().await,
        Some(ServerMessage::Joined { token: Some(t), .. }) if t == a_token
    ));
    assert!(matches!(
        back.recv().await,
        Some(ServerMessage::Snapshot { n, .. }) if n > 1
    ));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn malformed_oversized_and_out_of_place_frames_close_the_connection() {
    let config = config("full.toml", vec![], Limits::default());
    let running = start(&config).await.unwrap();
    // Before the hello, only the hello.
    let mut c = Client::connect(running.address).await.unwrap();
    c.send(&ClientMessage::Ping { nonce: 1 }).await.unwrap();
    assert_eq!(goodbye(&mut c).await, GoodbyeReason::Protocol);
    let mut c = Client::connect(running.address).await.unwrap();
    c.send(&ClientMessage::Hello {
        protocol: h3_protocol::PROTOCOL_VERSION + 1,
        client: String::new(),
        name: String::new(),
    })
    .await
    .unwrap();
    assert_eq!(goodbye(&mut c).await, GoodbyeReason::Protocol);
    // After it: text that is no message, a binary message, a message over the frame limit.
    let cases = [
        (Message::text("{\"Ping\":"), GoodbyeReason::Malformed),
        (Message::binary(vec![1, 2, 3]), GoodbyeReason::Malformed),
        (Message::text("x".repeat(5000)), GoodbyeReason::TooLarge),
    ];
    for (message, reason) in cases {
        let mut c = greeted(&running).await;
        c.send_raw(message).await.unwrap();
        assert_eq!(goodbye(&mut c).await, reason);
    }
    // Far over it, the WebSocket drops the connection unread.
    let mut c = greeted(&running).await;
    let _ = c.send_raw(Message::text("x".repeat(1 << 17))).await;
    assert_eq!(c.recv().await, None);
    // The game is unharmed: a client still takes a seat.
    seated(&running, Seat::Attacker).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_flooding_client_is_cut_off() {
    let limits = Limits {
        messages_per_second: 5,
        ..Limits::default()
    };
    let config = config("full.toml", vec![], limits);
    let running = start(&config).await.unwrap();
    let mut c = greeted(&running).await;
    for nonce in 0..20 {
        if c.send(&ClientMessage::Ping { nonce }).await.is_err() {
            break;
        }
    }
    let mut pongs = 0;
    loop {
        match c.recv().await {
            Some(ServerMessage::Pong { .. }) => pongs += 1,
            Some(ServerMessage::Goodbye { reason }) => {
                assert_eq!(reason, GoodbyeReason::RateLimited);
                break;
            }
            other => panic!("{other:?}"),
        }
    }
    assert!(pongs <= 4, "the hello counts too: {pongs}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn an_idle_client_is_cut_off() {
    let limits = Limits {
        idle_timeout_ms: 200,
        ..Limits::default()
    };
    let config = config("full.toml", vec![], limits);
    let running = start(&config).await.unwrap();
    let mut c = greeted(&running).await;
    assert_eq!(goodbye(&mut c).await, GoodbyeReason::IdleTimeout);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn connections_beyond_the_limit_are_refused_until_one_closes() {
    let limits = Limits {
        connections_per_address: 1,
        ..Limits::default()
    };
    let config = config("full.toml", vec![], limits);
    let running = start(&config).await.unwrap();
    let first = greeted(&running).await;
    assert!(Client::connect(running.address).await.is_err());
    first.close().await;
    let mut attempts = 0;
    while Client::connect(running.address).await.is_err() {
        attempts += 1;
        assert!(attempts < 250, "the place is never released");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[test]
fn the_example_configuration_loads() {
    let path = root().join("crates/h3-server/server.example.toml");
    let config = Config::load(&path).unwrap();
    assert_eq!(config.bind, "127.0.0.1:7878");
    assert_eq!(config.bots, [BotSeat::Defender]);
    assert_eq!(config.limits, Limits::default());
    assert!(config.scenario.is_file(), "relative to the file");
    assert!(game::Prepared::new(&config).is_ok());
}

#[test]
fn configurations_are_checked() {
    let base = root();
    let minimal = Config::parse("packs = \"p\"\nscenario = \"s.toml\"", &base).unwrap();
    assert_eq!(minimal.packs, base.join("p"));
    assert_eq!(minimal.data, base.join("data"));
    assert_eq!((minimal.checkpoint_every, minimal.bots.len()), (25, 0));
    assert!(Config::parse("packs = \"p\"\nscenario = \"s\"\nport = 1", &base).is_err());
    assert!(Config::load(&base.join("no-such-file.toml")).is_err());
    let both = config(
        "full.toml",
        vec![BotSeat::Attacker, BotSeat::Defender],
        Limits::default(),
    );
    assert!(
        game::Prepared::new(&both).is_err(),
        "a hosted game needs a player"
    );
    let missing = config("no-such-scenario.toml", vec![], Limits::default());
    assert!(game::Prepared::new(&missing).is_err());
    let profile = Config {
        profile: Some(String::from("synthetic:no-such-profile")),
        ..config("full.toml", vec![], Limits::default())
    };
    assert!(game::Prepared::new(&profile).is_err());
}

#[test]
fn tokens_are_128_bits_of_os_randomness() {
    let (a, b) = (game::token().unwrap(), game::token().unwrap());
    assert_eq!(a.len(), 32);
    assert!(a.bytes().all(|c| c.is_ascii_hexdigit()));
    assert_ne!(a, b);
}
