//! End-to-end tests over loopback (E16 M1 T3): headless clients play hosted battles against each
//! other and against the bot, re-take a seat, and send what a hostile client sends; the replay the
//! server writes verifies as `h3 replay verify` does.

use std::path::{Path, PathBuf};
use std::time::Duration;

use h3_kernel::Rng;
use h3_protocol::wire::{
    ClientMessage, Command, GameInfo, GoodbyeReason, Phase, Refusal, RejectCode, Seat,
    ServerMessage,
};
use tokio_tungstenite::tungstenite::Message;

use crate::client::Client;
use crate::config::{BotSeat, Config, Content, Limits, Offer, Server};
use crate::{Running, bench, game, start};

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

/// Two offers of the synthetic scenarios: `pvp` (the `full` battle between players) and `bot` (the
/// `duel` on a field of random obstacles, the server's bot defending); a free loopback port, the
/// data in a fresh directory.
fn config(limits: Limits) -> Config {
    let scenarios = root().join("packs/synthetic/scenarios");
    Config {
        server: Server {
            bind: String::from("127.0.0.1:0"),
            data: std::env::temp_dir().join(format!("h3-server-{:016x}", game::os_u64().unwrap())),
            checkpoint_every: 10,
        },
        content: Content {
            packs: root().join("packs"),
        },
        limits,
        offers: vec![
            Offer {
                id: String::from("pvp"),
                scenario: scenarios.join("full.toml"),
                profile: None,
                bots: vec![],
            },
            Offer {
                id: String::from("bot"),
                scenario: scenarios.join("duel.toml"),
                profile: None,
                bots: vec![BotSeat::Defender],
            },
        ],
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

/// The `Joined` and `Snapshot` after a join: the seat's token.
async fn joined(client: &mut Client) -> String {
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
    token
}

/// A client that created a game of `offer` and took `seat` in it: the game and the seat's token.
async fn created(running: &Running, offer: &str, seat: Seat) -> (Client, u32, String) {
    let mut client = greeted(running).await;
    client.create(offer, seat).await.unwrap();
    let created = client.recv().await;
    let Some(ServerMessage::GameCreated { game, .. }) = created else {
        panic!("{created:?}")
    };
    let token = joined(&mut client).await;
    (client, game, token)
}

/// A client that took `seat` in game `game`, with the seat's token.
async fn seated(running: &Running, game: u32, seat: Seat) -> (Client, String) {
    let mut client = greeted(running).await;
    client.join(game, seat, None).await.unwrap();
    let token = joined(&mut client).await;
    (client, token)
}

/// The games the server lists.
async fn listed(running: &Running) -> Vec<GameInfo> {
    let mut client = greeted(running).await;
    client
        .send(&ClientMessage::ListGames { seq: 3 })
        .await
        .unwrap();
    let Some(ServerMessage::Games {
        seq: 3,
        offers,
        games,
    }) = client.recv().await
    else {
        panic!("no list")
    };
    assert_eq!(offers.len(), 2);
    client.close().await;
    games
}

/// Plays a seat to the end of the game: every `YourTurn` is answered with a strike if there is
/// one, else with a command that stays in the battle; every message after the snapshot.
async fn play(mut client: Client, seed: u64) -> (Client, Vec<ServerMessage>) {
    let mut rng = Rng::from_seed(seed);
    let mut seen = Vec::new();
    let mut seq = 0;
    while let Some(message) = client.recv().await {
        if let ServerMessage::YourTurn { legal, .. } = &message {
            let command = bench::choose(legal, &mut rng).unwrap();
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

/// The replay a seat of game `game` fetches after the game: the file the server wrote, which
/// verifies.
async fn fetched_replay(client: &mut Client, config: &Config, game: u32) {
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
    let path = replay_file(&config.server.data, game);
    assert_eq!(replay, std::fs::read_to_string(&path).unwrap());
    let verified = h3_cli::verify_file(&path, &config.content.packs).unwrap();
    assert!(verified.commands > 0);
    assert!(
        !journal_file(&config.server.data, game).exists(),
        "the replay is the finished journal"
    );
}

/// Game `game`'s replay file.
fn replay_file(data: &Path, game: u32) -> PathBuf {
    let marker = format!("-game{game}-");
    let mut found = std::fs::read_dir(data.join("replays"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.to_string_lossy().contains(&marker));
    let path = found.next().unwrap();
    assert!(found.next().is_none());
    path
}

/// The last message seen: the game's end.
fn end(seen: &[ServerMessage]) -> h3_protocol::wire::Outcome {
    match seen.last() {
        Some(ServerMessage::GameEnded { result, .. }) => *result,
        other => panic!("{other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn two_clients_play_a_hosted_battle_and_its_replay_verifies() {
    let config = config(fast());
    let running = start(&config).await.unwrap();
    let (a, game, _) = created(&running, "pvp", Seat::Attacker).await;
    let (b, _) = seated(&running, game, Seat::Defender).await;
    let (a, b) = (tokio::spawn(play(a, 1)), tokio::spawn(play(b, 2)));
    let (mut a, a_seen) = tokio::time::timeout(GAME_TIME, a).await.unwrap().unwrap();
    let (_, b_seen) = tokio::time::timeout(GAME_TIME, b).await.unwrap().unwrap();
    assert_eq!(end(&a_seen), end(&b_seen));
    fetched_replay(&mut a, &config, game).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_client_plays_the_bot_on_a_field_of_random_obstacles() {
    let config = config(fast());
    let running = start(&config).await.unwrap();
    let mut c = greeted(&running).await;
    c.create("bot", Seat::Defender).await.unwrap();
    assert_eq!(
        c.recv().await,
        Some(ServerMessage::Refused {
            reason: Refusal::SeatTaken
        }),
        "the bot's seat"
    );
    let (a, game, _) = created(&running, "bot", Seat::Attacker).await;
    let (mut a, seen) = tokio::time::timeout(GAME_TIME, play(a, 3)).await.unwrap();
    assert!(matches!(seen.last(), Some(ServerMessage::GameEnded { .. })));
    // The obstacles came from their own seed and travel in the replay's setup.
    fetched_replay(&mut a, &config, game).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn two_games_are_played_at_once() {
    let config = config(fast());
    let running = start(&config).await.unwrap();
    let (a1, g1, _) = created(&running, "pvp", Seat::Attacker).await;
    let (b2, g2, _) = created(&running, "pvp", Seat::Defender).await;
    assert_ne!(g1, g2);
    let (b1, _) = seated(&running, g1, Seat::Defender).await;
    let (a2, _) = seated(&running, g2, Seat::Attacker).await;
    let running_games: Vec<(u32, Phase, usize)> = listed(&running)
        .await
        .into_iter()
        .map(|g| (g.game, g.phase, g.open.len()))
        .collect();
    assert_eq!(
        running_games,
        [(g1, Phase::Running, 0), (g2, Phase::Running, 0)]
    );
    let players = [(a1, 1), (b1, 2), (a2, 3), (b2, 4)].map(|(c, seed)| tokio::spawn(play(c, seed)));
    let mut ends = Vec::new();
    let mut clients = Vec::new();
    for player in players {
        let (client, seen) = tokio::time::timeout(GAME_TIME, player)
            .await
            .unwrap()
            .unwrap();
        ends.push(end(&seen));
        clients.push(client);
    }
    assert_eq!(
        (ends[0], ends[2]),
        (ends[1], ends[3]),
        "each game's two views agree"
    );
    fetched_replay(&mut clients[0], &config, g1).await;
    fetched_replay(&mut clients[2], &config, g2).await;
    // Over, the games are listed until their players leave; then they are gone.
    assert!(
        listed(&running)
            .await
            .iter()
            .all(|g| g.phase == Phase::Over)
    );
    for client in clients {
        client.close().await;
    }
    let mut attempts = 0;
    while !listed(&running).await.is_empty() {
        attempts += 1;
        assert!(attempts < 250, "the games never leave");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_lobby_refuses_with_its_reason_and_frees_a_waiting_game() {
    let limits = Limits {
        games_per_address: 1,
        ..Limits::default()
    };
    let config = config(limits);
    let running = start(&config).await.unwrap();
    let (creator, game, _) = created(&running, "pvp", Seat::Attacker).await;
    assert!(journal_file(&config.server.data, game).exists());
    assert_eq!(
        listed(&running).await,
        [GameInfo {
            game,
            offer: String::from("pvp"),
            phase: Phase::Waiting,
            open: vec![Seat::Defender],
        }]
    );
    let mut c = greeted(&running).await;
    let mut refusal = async |message: ClientMessage| {
        c.send(&message).await.unwrap();
        c.recv().await
    };
    let create = |offer: &str, seat| ClientMessage::CreateGame {
        seq: 1,
        offer: offer.to_owned(),
        seat,
    };
    for (message, reason) in [
        (create("pvp", Seat::Attacker), Refusal::TooManyGames),
        (create("nothing", Seat::Attacker), Refusal::NoSuchOffer),
        (create("pvp", Seat::Spectator), Refusal::NoSpectators),
        (
            ClientMessage::Join {
                game: 99,
                seat: Seat::Attacker,
                token: None,
            },
            Refusal::NoSuchGame,
        ),
    ] {
        assert_eq!(
            refusal(message).await,
            Some(ServerMessage::Refused { reason })
        );
    }
    // A game's message from a connection in no game.
    let wait = ClientMessage::Command {
        seq: 5,
        command: Command::Wait {},
    };
    assert_eq!(
        refusal(wait).await,
        Some(ServerMessage::Rejected {
            n: 0,
            seq: 5,
            code: RejectCode::NotSeated
        })
    );
    // The waiting game goes when its creator does, and frees the address's place.
    creator.close().await;
    let mut attempts = 0;
    while !listed(&running).await.is_empty() {
        attempts += 1;
        assert!(attempts < 250, "the game never leaves");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(
        !journal_file(&config.server.data, game).exists(),
        "a game that never started leaves no journal"
    );
    let (_, next, _) = created(&running, "pvp", Seat::Defender).await;
    assert!(next > game, "ids are not reused");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_dropped_client_retakes_its_seat_with_its_token() {
    let running = start(&config(Limits::default())).await.unwrap();
    let (a, game, a_token) = created(&running, "pvp", Seat::Attacker).await;
    let (_b, _) = seated(&running, game, Seat::Defender).await;
    a.close().await;
    let mut thief = greeted(&running).await;
    thief.join(game, Seat::Attacker, None).await.unwrap();
    assert_eq!(
        thief.recv().await,
        Some(ServerMessage::Refused {
            reason: Refusal::SeatTaken
        })
    );
    let mut back = greeted(&running).await;
    back.join(game, Seat::Attacker, Some(a_token.clone()))
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

/// Answers `turns` of the seat's turns as `play` does, then returns at the next `YourTurn`, which
/// it leaves unanswered: every message.
async fn play_turns(client: &mut Client, seed: u64, turns: usize) -> Vec<ServerMessage> {
    let mut rng = Rng::from_seed(seed);
    let mut seen = Vec::new();
    while let Some(message) = client.recv().await {
        let ended = matches!(message, ServerMessage::GameEnded { .. });
        if let ServerMessage::YourTurn { legal, .. } = &message {
            let answered = seen
                .iter()
                .filter(|m| matches!(m, ServerMessage::YourTurn { .. }))
                .count();
            if answered == turns {
                seen.push(message);
                break;
            }
            let command = bench::choose(legal, &mut rng).unwrap();
            let seq = u32::try_from(answered).unwrap() + 1;
            client
                .send(&ClientMessage::Command { seq, command })
                .await
                .unwrap();
        }
        seen.push(message);
        if ended {
            break;
        }
    }
    seen
}

/// A connection that re-took `seat` of game `game` with `token`: what it was sent after `Joined`
/// (the snapshot, and the turn if it is the seat's), the numbers set to 0.
async fn retaken(
    running: &Running,
    game: u32,
    seat: Seat,
    token: &str,
) -> (Client, Vec<ServerMessage>) {
    let mut client = greeted(running).await;
    client
        .join(game, seat, Some(token.to_owned()))
        .await
        .unwrap();
    assert!(matches!(
        client.recv().await,
        Some(ServerMessage::Joined { .. })
    ));
    let mut sent = Vec::new();
    while let Ok(Some(mut message)) =
        tokio::time::timeout(Duration::from_millis(300), client.recv()).await
    {
        if let ServerMessage::Snapshot { n, .. } | ServerMessage::YourTurn { n, .. } = &mut message
        {
            *n = 0;
        }
        sent.push(message);
    }
    (client, sent)
}

/// A crash at this moment: a new data directory with a copy of the journals `data` holds now (the
/// server syncs every line before it sends what depends on it).
fn crashed(data: &Path) -> PathBuf {
    let copy = data.with_extension("restarted");
    std::fs::create_dir_all(copy.join("games")).unwrap();
    for entry in std::fs::read_dir(data.join("games")).unwrap() {
        let path = entry.unwrap().path();
        std::fs::copy(&path, copy.join("games").join(path.file_name().unwrap())).unwrap();
    }
    copy
}

/// Game `game`'s journal in the data directory `data`.
fn journal_file(data: &Path, game: u32) -> PathBuf {
    data.join("games").join(format!("{game}.journal"))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_game_stopped_mid_battle_resumes_after_a_restart_and_its_replay_verifies() {
    let config = config(fast());
    let running = start(&config).await.unwrap();
    let (mut a, game, token) = created(&running, "bot", Seat::Attacker).await;
    let seen = play_turns(&mut a, 5, 2).await;
    assert!(
        matches!(seen.last(), Some(ServerMessage::YourTurn { .. })),
        "mid-battle: {seen:?}"
    );
    let (_, before) = retaken(&running, game, Seat::Attacker, &token).await;
    // The server stops here; another starts on what its data directory holds.
    let restarted = Config {
        server: Server {
            data: crashed(&config.server.data),
            ..config.server.clone()
        },
        ..config.clone()
    };
    let again = start(&restarted).await.unwrap();
    assert_eq!(
        listed(&again).await,
        [GameInfo {
            game,
            offer: String::from("bot"),
            phase: Phase::Running,
            open: vec![],
        }],
        "the game is back, its seats taken"
    );
    let mut thief = greeted(&again).await;
    thief.join(game, Seat::Attacker, None).await.unwrap();
    assert_eq!(
        thief.recv().await,
        Some(ServerMessage::Refused {
            reason: Refusal::SeatTaken
        })
    );
    // The same battle, the same turn: the player goes on with its token.
    let (mut back, after) = retaken(&again, game, Seat::Attacker, &token).await;
    assert_eq!(after, before);
    let Some(ServerMessage::YourTurn { legal, .. }) = after.last() else {
        panic!("{after:?}")
    };
    let command = bench::choose(legal, &mut Rng::from_seed(6)).unwrap();
    back.send(&ClientMessage::Command { seq: 1, command })
        .await
        .unwrap();
    let (mut back, seen) = tokio::time::timeout(GAME_TIME, play(back, 7))
        .await
        .unwrap();
    assert!(matches!(seen.last(), Some(ServerMessage::GameEnded { .. })));
    // The replay holds the commands from before the restart and after; the journal is gone.
    fetched_replay(&mut back, &restarted, game).await;
    assert!(!journal_file(&restarted.server.data, game).exists());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn journals_that_do_not_restore_their_game_are_void_or_gone() {
    let limits = Limits {
        games_per_address: 8,
        ..fast()
    };
    let config = config(limits);
    let running = start(&config).await.unwrap();
    // Game 1 runs, game 2 waits for its second player, game 3 runs.
    let (mut one, first, _) = created(&running, "bot", Seat::Attacker).await;
    play_turns(&mut one, 1, 1).await;
    let (_two, waiting, _) = created(&running, "pvp", Seat::Attacker).await;
    let (mut three, third, _) = created(&running, "bot", Seat::Attacker).await;
    play_turns(&mut three, 3, 1).await;
    let data = crashed(&config.server.data);
    let rewrite = |game: u32, change: &dyn Fn(&mut crate::journal::Start), to: u32| {
        let text = std::fs::read_to_string(journal_file(&data, game)).unwrap();
        let (first, rest) = text.split_once('\n').unwrap();
        let mut start: crate::journal::Start = serde_json::from_str(first).unwrap();
        change(&mut start);
        start.game = to;
        let line = serde_json::to_string(&start).unwrap();
        std::fs::write(journal_file(&data, to), format!("{line}\n{rest}")).unwrap();
    };
    // Game 3's content changed under it: another ruleset.
    rewrite(
        third,
        &|start| {
            let hash = start
                .replay
                .lines()
                .find(|l| l.starts_with("ruleset_hash"))
                .unwrap();
            start.replay = start
                .replay
                .replace(hash, "ruleset_hash = \"0x0000000000000001\"");
        },
        third,
    );
    // Game 9's offer is gone.
    rewrite(first, &|start| start.offer = String::from("gone"), 9);
    // Game 1's last line was cut short by the crash.
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(journal_file(&data, first))
        .unwrap();
    std::io::Write::write_all(&mut file, b"{\"Command\":{\"comm").unwrap();
    let restarted = Config {
        server: Server {
            data: data.clone(),
            ..config.server.clone()
        },
        ..config.clone()
    };
    let again = start(&restarted).await.unwrap();
    let games: Vec<u32> = listed(&again).await.iter().map(|g| g.game).collect();
    assert_eq!(games, [first], "only the game that re-ran");
    let journals: Vec<_> = std::fs::read_dir(data.join("games")).unwrap().collect();
    assert_eq!(journals.len(), 1, "the waiting game's is gone");
    assert!(!journal_file(&data, waiting).exists());
    let void: Vec<_> = std::fs::read_dir(data.join("void")).unwrap().collect();
    assert_eq!(void.len(), 2, "game 3's and game 9's");
    // Ids go on after the restored game's.
    let (_, next, _) = created(&again, "pvp", Seat::Attacker).await;
    assert_eq!(next, first + 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn malformed_oversized_and_out_of_place_frames_close_the_connection() {
    let running = start(&config(Limits::default())).await.unwrap();
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
    // The server is unharmed: a client still creates a game.
    created(&running, "pvp", Seat::Attacker).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_flooding_client_is_cut_off() {
    let limits = Limits {
        messages_per_second: 5,
        ..Limits::default()
    };
    let running = start(&config(limits)).await.unwrap();
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
    let running = start(&config(limits)).await.unwrap();
    let mut c = greeted(&running).await;
    assert_eq!(goodbye(&mut c).await, GoodbyeReason::IdleTimeout);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn connections_beyond_the_limit_are_refused_until_one_closes() {
    let limits = Limits {
        connections_per_address: 1,
        ..Limits::default()
    };
    let running = start(&config(limits)).await.unwrap();
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

/// A client that stops reading loses its connection and its place, even while it never reads the
/// close (the M1 review: the close frame's write waited forever on a full socket).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_client_that_stops_reading_is_dropped_and_its_place_freed() {
    let limits = Limits {
        connections_per_address: 1,
        idle_timeout_ms: 1_000,
        ..fast()
    };
    let running = start(&config(limits)).await.unwrap();
    let (mut stalled, _, _) = created(&running, "pvp", Seat::Attacker).await;
    // Records, never read, until the socket is full: the server's write stalls, then its reads,
    // and the game drops the connection whose queue fills meanwhile.
    for seq in 1..=20_000 {
        let fetch = ClientMessage::FetchRecord { seq };
        let sent = tokio::time::timeout(Duration::from_secs(1), stalled.send(&fetch)).await;
        if !matches!(sent, Ok(Ok(()))) {
            break;
        }
    }
    let mut attempts = 0;
    loop {
        if let Ok(mut client) = Client::connect(running.address).await
            && matches!(
                client.hello("test").await,
                Ok(Some(ServerMessage::Welcome { .. }))
            )
        {
            break;
        }
        attempts += 1;
        assert!(attempts < 500, "the place is never released");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    drop(stalled);
}

#[test]
fn the_example_configuration_loads() {
    let path = root().join("crates/h3-server/server.example.toml");
    let config = Config::load(&path).unwrap();
    assert_eq!(config.server.bind, "127.0.0.1:7878");
    assert_eq!(config.limits, Limits::default());
    let ids: Vec<&str> = config.offers.iter().map(|o| o.id.as_str()).collect();
    assert_eq!(ids, ["full", "duel-bot"]);
    assert_eq!(config.offers[1].bots, [BotSeat::Defender]);
    for offer in &config.offers {
        assert!(offer.scenario.is_file(), "relative to the file");
        assert!(game::Offer::load(offer, &config.content.packs).is_ok());
    }
}

#[test]
fn configurations_are_checked() {
    let base = root();
    let offer = "[[offer]]\nid = \"a\"\nscenario = \"s.toml\"\n";
    let minimal = Config::parse(&format!("[content]\npacks = \"p\"\n{offer}"), &base).unwrap();
    assert_eq!(minimal.content.packs, base.join("p"));
    assert_eq!(minimal.server.data, base.join("data"));
    assert_eq!(minimal.offers[0].scenario, base.join("s.toml"));
    assert_eq!(
        (
            minimal.server.checkpoint_every,
            minimal.offers[0].bots.len()
        ),
        (25, 0)
    );
    let refused = |text: &str| Config::parse(&format!("[content]\npacks = \"p\"\n{text}"), &base);
    assert!(
        refused(&format!("{offer}[server]\nport = 1")).is_err(),
        "an unknown key"
    );
    assert!(refused("").is_err(), "no offer");
    assert!(refused(&format!("{offer}{offer}")).is_err(), "a taken id");
    assert!(
        refused("[[offer]]\nid = \"\"\nscenario = \"s\"").is_err(),
        "an empty id"
    );
    assert!(
        refused(&format!("{offer}bots = [\"attacker\", \"defender\"]")).is_err(),
        "a hosted game needs a player"
    );
    for limit in [
        "max_frame = 0",
        "max_frame = 2000000",
        "messages_per_second = 0",
        "connections = 0",
        "connections_per_address = 0",
        "idle_timeout_ms = 0",
        "idle_timeout_ms = 9223372036854775807",
        "outbound_queue = 0",
        "outbound_queue = 4000000",
        "games = 0",
        "games = 5000",
        "games_per_address = 0",
    ] {
        let text = format!("{offer}[limits]\n{limit}");
        let error = refused(&text).unwrap_err();
        assert!(error.0.contains("outside"), "{limit}: {error}");
    }
    assert!(Config::load(&base.join("no-such-file.toml")).is_err());
    // An offer whose scenario or profile does not load stops the start.
    let scenarios = base.join("packs/synthetic/scenarios");
    let packs = base.join("packs");
    let load = |scenario: &str, profile: Option<&str>| {
        let offer = Offer {
            id: String::from("x"),
            scenario: scenarios.join(scenario),
            profile: profile.map(String::from),
            bots: vec![],
        };
        game::Offer::load(&offer, &packs)
    };
    assert!(load("full.toml", None).is_ok());
    assert!(load("no-such-scenario.toml", None).is_err());
    assert!(load("full.toml", Some("synthetic:no-such-profile")).is_err());
}

#[test]
fn tokens_are_128_bits_of_os_randomness() {
    let (a, b) = (game::token().unwrap(), game::token().unwrap());
    assert_eq!(a.len(), 32);
    assert!(a.bytes().all(|c| c.is_ascii_hexdigit()));
    assert_ne!(a, b);
}

#[test]
fn the_workload_plays_hosted_battles_to_their_end() {
    let (_, ruleset, setup) = h3_cli::load_scenario(
        &root().join("packs/synthetic/scenarios/full.toml"),
        &root().join("packs"),
        1,
    )
    .unwrap();
    let totals = bench::run(&ruleset, &setup, 1, 2, 5_000).unwrap();
    assert_eq!((totals.games, totals.ended), (2, 2));
    // Two hellos and two joins, then one command per turn; every command is answered.
    assert!(totals.frames_in > 4 && totals.frames_out > totals.frames_in);
    assert!(totals.bytes_out > totals.frames_out);
    assert_eq!(
        bench::run(&ruleset, &setup, 1, 2, 5_000).unwrap(),
        totals,
        "deterministic"
    );
    let cut = bench::run(&ruleset, &setup, 1, 1, 3).unwrap();
    assert_eq!(
        (cut.ended, cut.frames_in),
        (0, 7),
        "the command limit holds"
    );
}

#[test]
fn the_scripted_client_strikes_when_it_can() {
    let mut rng = Rng::from_seed(1);
    let attack = Command::Attack { target: 3, from: 9 };
    let legal = [Command::Defend {}, attack, Command::Retreat {}];
    assert_eq!(bench::choose(&legal, &mut rng), Some(attack));
    let stay = [Command::Retreat {}, Command::Surrender {}, Command::Wait {}];
    assert_eq!(bench::choose(&stay, &mut rng), Some(Command::Wait {}));
    assert_eq!(bench::choose(&[Command::Retreat {}], &mut rng), None);
}
