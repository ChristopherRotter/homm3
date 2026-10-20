//! End-to-end tests over loopback (E16 M1 T3): headless clients play hosted battles against each
//! other and against the bot, re-take a seat, and send what a hostile client sends; the replay the
//! server writes verifies as `h3 replay verify` does.

use std::path::{Path, PathBuf};
use std::time::Duration;

use h3_kernel::Rng;
use h3_protocol::codec;
use h3_protocol::session::ConnId;
use h3_protocol::wire::{
    ClientMessage, Command, GameInfo, GoodbyeReason, Phase, Query, RecordHeader, Refusal,
    RejectCode, ReplayInfo, Seat, ServerMessage,
};
use tokio_tungstenite::tungstenite::Message;

use crate::client::Client;
use crate::config::{BotSeat, Config, Content, Discovery, Limits, Offer, Server};
use crate::{Running, bench, game, registry, start};

mod accounts;
mod bounds;
mod card;
mod identity;
mod ladders;
mod lists;
mod peers;
mod proxy;
mod seats;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Limits for automated players, who answer every turn at once (a person sends far fewer than the
/// default 20 messages per second).
fn fast() -> Limits {
    Limits {
        messages_per_second: 100_000,
        messages_per_address: 100_000,
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
            shadow_instance: false,
            public_url: None,
            addresses: vec![],
            anonymous: true,
            trusted_proxies: vec![],
            name: None,
            description: String::new(),
            operator: String::new(),
        },
        content: Content {
            packs: root().join("packs"),
        },
        limits,
        discovery: Discovery::default(),
        peers: vec![],
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
        ladders: vec![],
    }
}

/// A client that said hello (at the loopback address for a server bound to an unspecified one).
async fn greeted(running: &Running) -> Client {
    let mut client = Client::connect(crate::reachable(running.address))
        .await
        .unwrap();
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
async fn spectators_watch_a_hosted_game_within_its_limit() {
    let limits = Limits {
        spectators_per_game: 1,
        ..fast()
    };
    let running = start(&config(limits)).await.unwrap();
    let (a, game, _) = created(&running, "bot", Seat::Attacker).await;
    let mut watcher = greeted(&running).await;
    watcher.join(game, Seat::Spectator, None).await.unwrap();
    assert!(matches!(
        watcher.recv().await,
        Some(ServerMessage::Joined {
            seat: Seat::Spectator,
            token: None,
            ..
        })
    ));
    let Some(ServerMessage::Snapshot { observation, .. }) = watcher.recv().await else {
        panic!("no snapshot")
    };
    assert!(observation.spectator && observation.own_hero.is_none());
    // One spectator is the limit here.
    let mut late = greeted(&running).await;
    late.join(game, Seat::Spectator, None).await.unwrap();
    assert_eq!(
        late.recv().await,
        Some(ServerMessage::Refused {
            reason: Refusal::NoSpectators
        })
    );
    // The spectator sees the game to its end, never a turn, and gets no replay.
    let (_, seen) = tokio::time::timeout(GAME_TIME, play(a, 3)).await.unwrap();
    let ended = end(&seen);
    let mut watched = Vec::new();
    while let Some(message) = tokio::time::timeout(GAME_TIME, watcher.recv())
        .await
        .unwrap()
    {
        let over = matches!(message, ServerMessage::GameEnded { .. });
        watched.push(message);
        if over {
            break;
        }
    }
    assert_eq!(end(&watched), ended);
    assert!(watched.iter().all(|m| matches!(
        m,
        ServerMessage::Events { .. } | ServerMessage::GameEnded { .. }
    )));
    watcher
        .send(&ClientMessage::FetchReplay { seq: 9 })
        .await
        .unwrap();
    assert!(matches!(
        watcher.recv().await,
        Some(ServerMessage::Rejected {
            seq: 9,
            code: RejectCode::NotSeated,
            ..
        })
    ));
}

/// A client that created a game of `offer` closed to spectators and took `seat` in it.
async fn created_closed(running: &Running, offer: &str, seat: Seat) -> (Client, u32) {
    let mut client = greeted(running).await;
    let create = ClientMessage::CreateGame {
        seq: 1,
        offer: offer.to_owned(),
        seat,
        spectators: Some(false),
    };
    client.send(&create).await.unwrap();
    let created = client.recv().await;
    let Some(ServerMessage::GameCreated { game, .. }) = created else {
        panic!("{created:?}")
    };
    joined(&mut client).await;
    (client, game)
}

/// Waits until the lobby lists game `game` as taking spectators or not.
async fn listed_watchable(running: &Running, game: u32, spectators: bool) {
    let mut attempts = 0;
    while !listed(running)
        .await
        .iter()
        .any(|g| g.game == game && g.spectators == spectators)
    {
        attempts += 1;
        assert!(attempts < 250, "game {game} is never listed so");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_host_opens_and_closes_its_game_to_spectators_before_it_starts() {
    let config = config(fast());
    let running = start(&config).await.unwrap();
    let (mut host, game) = created_closed(&running, "pvp", Seat::Attacker).await;
    listed_watchable(&running, game, false).await;
    let refused = Some(ServerMessage::Refused {
        reason: Refusal::NoSpectators,
    });
    let mut watcher = greeted(&running).await;
    watcher.join(game, Seat::Spectator, None).await.unwrap();
    assert_eq!(watcher.recv().await, refused);
    // Opened, the game takes the spectator; closed again, its watch ends.
    let set = |seq, open| ClientMessage::SetSpectators { seq, open };
    host.send(&set(2, true)).await.unwrap();
    assert!(matches!(
        host.recv().await,
        Some(ServerMessage::SpectatorsSet {
            seq: 2,
            open: true,
            ..
        })
    ));
    listed_watchable(&running, game, true).await;
    watcher.join(game, Seat::Spectator, None).await.unwrap();
    assert!(matches!(
        watcher.recv().await,
        Some(ServerMessage::Joined { .. })
    ));
    host.send(&set(3, false)).await.unwrap();
    assert!(matches!(
        host.recv().await,
        Some(ServerMessage::SpectatorsSet { open: false, .. })
    ));
    assert!(matches!(
        watcher.recv().await,
        Some(ServerMessage::Snapshot { .. })
    ));
    assert_eq!(goodbye(&mut watcher).await, GoodbyeReason::SpectatorsClosed);
    // Opened again, then the second player starts the game: the setting is fixed.
    host.send(&set(4, true)).await.unwrap();
    let (_defender, _) = seated(&running, game, Seat::Defender).await;
    host.send(&set(5, false)).await.unwrap();
    loop {
        match host.recv().await {
            Some(ServerMessage::Rejected { seq: 5, code, .. }) => {
                break assert_eq!(code, RejectCode::AlreadyStarted);
            }
            Some(_) => {}
            None => panic!("no answer"),
        }
    }
    // A game against the bot, created closed, starts at once.
    let (_against_bot, bot_game) = created_closed(&running, "bot", Seat::Attacker).await;
    // After a restart each game takes spectators as it started: its journal holds the setting.
    let restarted = Config {
        server: Server {
            data: crashed(&config.server.data),
            ..config.server.clone()
        },
        ..config.clone()
    };
    let again = start(&restarted).await.unwrap();
    let watchable: Vec<_> = listed(&again)
        .await
        .iter()
        .map(|g| (g.game, g.spectators))
        .collect();
    assert_eq!(watchable, [(game, true), (bot_game, false)]);
    let mut late = greeted(&again).await;
    late.join(bot_game, Seat::Spectator, None).await.unwrap();
    assert_eq!(late.recv().await, refused);
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
            spectators: true,
            seated: vec![],
            ladder: None,
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
        spectators: None,
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

/// A crash at this moment: a new data directory with a copy of the journals, the player log and the
/// server's key `data` holds now (the server syncs every line before it sends what depends on it).
fn crashed(data: &Path) -> PathBuf {
    let copy = data.with_extension("restarted");
    std::fs::create_dir_all(copy.join("games")).unwrap();
    for entry in std::fs::read_dir(data.join("games")).unwrap() {
        let path = entry.unwrap().path();
        std::fs::copy(&path, copy.join("games").join(path.file_name().unwrap())).unwrap();
    }
    for file in ["players.log", crate::key::KEY_FILE] {
        if data.join(file).exists() {
            std::fs::copy(data.join(file), copy.join(file)).unwrap();
        }
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
            spectators: true,
            seated: vec![],
            ladder: None,
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
    // Ids go on after every one handed out before, the void journals' too (E16 M4 T3b: unique for
    // the server's life).
    let (_, next, _) = created(&again, "pvp", Seat::Attacker).await;
    assert_eq!(next, 10);
}

/// A change of a journal's line: the line it becomes, or `None` to keep it.
type Change = dyn Fn(&str) -> Option<String>;

/// Rewrites the first line of game `game`'s journal that `change` changes, in place and at the same
/// length, as a fault on the disk would while the server appends to it.
fn flip(data: &Path, game: u32, change: &Change) {
    use std::io::{Seek, SeekFrom, Write};
    let path = journal_file(data, game);
    let text = std::fs::read_to_string(&path).unwrap();
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        if let Some(changed) = change(line) {
            assert_eq!(changed.len(), line.len());
            let mut file = std::fs::OpenOptions::new().write(true).open(&path).unwrap();
            file.seek(SeekFrom::Start(offset)).unwrap();
            file.write_all(changed.as_bytes()).unwrap();
            return;
        }
        offset += line.len() as u64;
    }
    panic!("nothing to change in {text}");
}

/// Answers the `YourTurn` that `seen` ends with, then plays the seat to the end of the game.
async fn play_on(mut client: Client, seen: &[ServerMessage], seed: u64) -> Client {
    let Some(ServerMessage::YourTurn { legal, .. }) = seen.last() else {
        panic!("{seen:?}")
    };
    let command = bench::choose(legal, &mut Rng::from_seed(seed)).unwrap();
    client
        .send(&ClientMessage::Command { seq: 100, command })
        .await
        .unwrap();
    let (client, seen) = tokio::time::timeout(GAME_TIME, play(client, seed))
        .await
        .unwrap();
    assert!(matches!(seen.last(), Some(ServerMessage::GameEnded { .. })));
    client
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_game_whose_journal_changed_is_void_and_its_replay_withheld() {
    let mut config = config(fast());
    config.server.checkpoint_every = 2;
    let running = start(&config).await.unwrap();
    let data = &config.server.data;
    // A flipped hash, and a command given to the other side.
    let checkpoint = |line: &str| {
        let at = line.find("\"hash\":\"0x")? + 10;
        let digit = if &line[at..=at] == "0" { "1" } else { "0" };
        Some(format!("{}{digit}{}", &line[..at], &line[at + 1..]))
    };
    let command = |line: &str| {
        let side = line.find("{\"Command\":{\"command\":\"")? + 23;
        let other = if &line[side..=side] == "A" { "D" } else { "A" };
        Some(format!("{}{other}{}", &line[..side], &line[side + 1..]))
    };
    let flips: [(&Change, &str); 2] = [
        (&checkpoint, "# void: state hash after 2 commands: expected"),
        (&command, "# void: command 0 rejected"),
    ];
    for (change, diagnosis) in flips {
        let (mut a, game, _) = created(&running, "bot", Seat::Attacker).await;
        let seen = play_turns(&mut a, 5, 2).await;
        flip(data, game, change);
        let mut a = play_on(a, &seen, 7).await;
        a.send(&ClientMessage::FetchReplay { seq: 900 })
            .await
            .unwrap();
        assert!(matches!(
            a.recv().await,
            Some(ServerMessage::Rejected {
                seq: 900,
                code: RejectCode::Unverified,
                ..
            })
        ));
        // No replay; the journal and the replay as played are in the void directory.
        let marker = format!("-game{game}-");
        let named = |dir: &str| -> Vec<PathBuf> {
            std::fs::read_dir(data.join(dir))
                .into_iter()
                .flatten()
                .map(|entry| entry.unwrap().path())
                .filter(|path| {
                    let name = path.file_name().unwrap().to_string_lossy();
                    name.contains(&marker) || name.starts_with(&format!("{game}-"))
                })
                .collect()
        };
        assert_eq!(named("replays"), Vec::<PathBuf>::new());
        assert!(!journal_file(data, game).exists());
        let void = named("void");
        assert_eq!(void.len(), 2, "{void:?}");
        let played = void
            .iter()
            .find(|p| p.extension().unwrap() == "toml")
            .unwrap();
        let text = std::fs::read_to_string(played).unwrap();
        assert!(text.starts_with(diagnosis), "{}", &text[..80]);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_paused_game_resumes_with_a_seats_token_also_after_a_restart() {
    let config = config(fast());
    let running = start(&config).await.unwrap();
    let (mut a, game, token) = created(&running, "bot", Seat::Attacker).await;
    let seen = play_turns(&mut a, 5, 2).await;
    assert!(matches!(seen.last(), Some(ServerMessage::YourTurn { .. })));
    let (mut host, before) = retaken(&running, game, Seat::Attacker, &token).await;
    assert_eq!(goodbye(&mut a).await, GoodbyeReason::Replaced);
    // The host pauses: its connection closes, the game is listed as paused, its journal stays.
    host.send(&ClientMessage::Pause { seq: 1 }).await.unwrap();
    assert_eq!(goodbye(&mut host).await, GoodbyeReason::Paused);
    let paused = GameInfo {
        game,
        offer: String::from("bot"),
        phase: Phase::Paused,
        open: vec![],
        spectators: false,
        seated: vec![],
        ladder: None,
    };
    let mut attempts = 0;
    while listed(&running).await != [paused.clone()] {
        attempts += 1;
        assert!(attempts < 250, "the game never pauses");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let journal = std::fs::read_to_string(journal_file(&config.server.data, game)).unwrap();
    assert!(journal.ends_with("{\"Paused\":{}}\n"));
    // Only a seat's token resumes it.
    let mut c = greeted(&running).await;
    for (seat, token, reason) in [
        (Seat::Attacker, None, Refusal::SeatTaken),
        (Seat::Attacker, Some("0f"), Refusal::BadToken),
        (Seat::Defender, Some(token.as_str()), Refusal::SeatTaken),
        (Seat::Spectator, None, Refusal::NoSpectators),
    ] {
        c.join(game, seat, token.map(String::from)).await.unwrap();
        assert_eq!(c.recv().await, Some(ServerMessage::Refused { reason }));
    }
    assert_eq!(listed(&running).await, std::slice::from_ref(&paused));
    // Another server starts on what the data directory holds: the game is still paused there.
    let restarted = Config {
        server: Server {
            data: crashed(&config.server.data),
            ..config.server.clone()
        },
        ..config.clone()
    };
    let again = start(&restarted).await.unwrap();
    assert_eq!(listed(&again).await, [paused]);
    // On both, the seat's token resumes it where it paused.
    let (_, here) = retaken(&running, game, Seat::Attacker, &token).await;
    assert_eq!(here, before);
    let (back, after) = retaken(&again, game, Seat::Attacker, &token).await;
    assert_eq!(after, before);
    assert_eq!(listed(&again).await[0].phase, Phase::Running);
    let mut back = play_on(back, &after, 7).await;
    fetched_replay(&mut back, &restarted, game).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_draining_server_refuses_new_games_and_stops_once_none_runs() {
    let config = config(fast());
    let running = start(&config).await.unwrap();
    let (mut waiting, waiting_game, _) = created(&running, "pvp", Seat::Attacker).await;
    let (a, _, _) = created(&running, "bot", Seat::Attacker).await;
    assert_eq!(running.in_play(), 2);
    running.drain();
    // The waiting game ends, its journal with it; no game is created any more.
    assert_eq!(goodbye(&mut waiting).await, GoodbyeReason::Shutdown);
    let mut late = greeted(&running).await;
    late.create("pvp", Seat::Attacker).await.unwrap();
    assert_eq!(
        late.recv().await,
        Some(ServerMessage::Refused {
            reason: Refusal::Draining
        })
    );
    assert!(!journal_file(&config.server.data, waiting_game).exists());
    // The running game plays on; once it is over the server is drained.
    let undrained = tokio::time::timeout(Duration::from_millis(100), running.drained()).await;
    assert!(undrained.is_err(), "a game runs");
    let (mut a, _) = tokio::time::timeout(GAME_TIME, play(a, 3)).await.unwrap();
    tokio::time::timeout(GAME_TIME, running.drained())
        .await
        .unwrap();
    // Stopped, the connections hear it and no new one is taken.
    let address = running.address;
    running.shutdown().await;
    assert_eq!(goodbye(&mut a).await, GoodbyeReason::Shutdown);
    assert_eq!(goodbye(&mut late).await, GoodbyeReason::Shutdown);
    assert!(Client::connect(address).await.is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_stopped_server_restores_its_games_at_its_next_start() {
    let config = config(fast());
    let running = start(&config).await.unwrap();
    let (mut a, game, token) = created(&running, "bot", Seat::Attacker).await;
    let seen = play_turns(&mut a, 5, 2).await;
    assert!(matches!(seen.last(), Some(ServerMessage::YourTurn { .. })));
    running.shutdown().await;
    assert_eq!(goodbye(&mut a).await, GoodbyeReason::Shutdown);
    assert!(journal_file(&config.server.data, game).exists());
    // The next start on the same data directory has the game back, at the same turn.
    let again = start(&config).await.unwrap();
    assert_eq!(listed(&again).await[0].phase, Phase::Running);
    let (mut back, after) = retaken(&again, game, Seat::Attacker, &token).await;
    let Some(ServerMessage::YourTurn { legal, .. }) = after.last() else {
        panic!("{after:?}")
    };
    let command = bench::choose(legal, &mut Rng::from_seed(6)).unwrap();
    back.send(&ClientMessage::Command { seq: 1, command })
        .await
        .unwrap();
    let (_, seen) = tokio::time::timeout(GAME_TIME, play(back, 7))
        .await
        .unwrap();
    assert!(matches!(seen.last(), Some(ServerMessage::GameEnded { .. })));
}

/// The archive's page from its `from`-th newest replay: how many it holds, and the page.
async fn archive_page(running: &Running, from: u32) -> (u32, Vec<ReplayInfo>) {
    let mut client = greeted(running).await;
    let list = ClientMessage::ListReplays { seq: 5, from };
    client.send(&list).await.unwrap();
    let Some(ServerMessage::Replays {
        seq: 5,
        total,
        replays,
    }) = client.recv().await
    else {
        panic!("no page")
    };
    client.close().await;
    (total, replays)
}

/// What a client in no game gets for `FetchArchived { replay: id }`.
async fn fetched_archived(running: &Running, id: &str) -> Option<ServerMessage> {
    let mut client = greeted(running).await;
    let fetch = ClientMessage::FetchArchived {
        seq: 6,
        replay: id.to_owned(),
    };
    client.send(&fetch).await.unwrap();
    client.recv().await
}

/// E16 M2 T5d (SRV-Q15): a finished game's replay is public: any client lists the archive and
/// downloads a replay from it without joining a game; a game its host closed to spectators is
/// archived unlisted; beyond `replays_kept` the oldest go; a restart reads the archive back.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn finished_games_are_archived_for_anyone_to_download() {
    let limits = Limits {
        replays_kept: 2,
        ..fast()
    };
    let config = config(limits);
    let running = start(&config).await.unwrap();
    let mut games = Vec::new();
    for (seed, open) in [(1, true), (2, false), (3, true), (4, true)] {
        let (client, game) = if open {
            let (client, game, _) = created(&running, "bot", Seat::Attacker).await;
            (client, game)
        } else {
            created_closed(&running, "bot", Seat::Attacker).await
        };
        let (client, _) = tokio::time::timeout(GAME_TIME, play(client, seed))
            .await
            .unwrap();
        client.close().await;
        games.push(game);
    }
    // The replay joins the archive just after the end is told.
    let mut attempts = 0;
    let (total, replays) = loop {
        let (total, replays) = archive_page(&running, 0).await;
        if replays.first().is_some_and(|r| r.game == games[3]) {
            break (total, replays);
        }
        attempts += 1;
        assert!(attempts < 250, "the last game is never archived");
        tokio::time::sleep(Duration::from_millis(20)).await;
    };
    // The two newest open games, newest first; the oldest went, file and all.
    assert_eq!(total, 2);
    let listed: Vec<u32> = replays.iter().map(|r| r.game).collect();
    assert_eq!(listed, [games[3], games[2]]);
    assert!(replays.iter().all(|r| r.offer == "bot"));
    let replays_dir = config.server.data.join("replays");
    let file = |id: &str| replays_dir.join(format!("{id}.toml"));
    assert!(
        !replay_file_exists(&replays_dir, games[0]),
        "the oldest went"
    );
    assert!(
        replay_file_exists(&replays_dir.join("closed"), games[1]),
        "the closed game's replay is kept apart, unlisted"
    );
    // A page further on; past the end, none.
    assert_eq!(archive_page(&running, 1).await.1, replays[1..]);
    assert_eq!(archive_page(&running, 5).await, (2, vec![]));
    // Anyone downloads a listed replay, which verifies.
    let newest = &replays[0].id;
    let Some(ServerMessage::Replay {
        n: 0,
        seq: 6,
        replay,
    }) = fetched_archived(&running, newest).await
    else {
        panic!("no replay")
    };
    assert_eq!(replay, std::fs::read_to_string(file(newest)).unwrap());
    assert!(h3_cli::verify_file(&file(newest), &config.content.packs).is_ok());
    let no_such = Some(ServerMessage::Refused {
        reason: Refusal::NoSuchReplay,
    });
    for id in ["closed", "../journal", "bot-game1-0", ""] {
        assert_eq!(fetched_archived(&running, id).await, no_such, "{id}");
    }
    // A restart reads the archive back, in the same order.
    running.shutdown().await;
    let again = start(&config).await.unwrap();
    assert_eq!(archive_page(&again, 0).await, (total, replays));
}

/// Whether `dir` holds a replay of game `game`.
fn replay_file_exists(dir: &Path, game: u32) -> bool {
    let marker = format!("-game{game}-");
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .any(|entry| entry.is_ok_and(|e| e.path().to_string_lossy().contains(&marker)))
}

/// Waits until the lobby lists game `game` in `phase`, or no more (`None`).
async fn listed_as(running: &Running, game: u32, phase: Option<Phase>) {
    let mut attempts = 0;
    while listed(running)
        .await
        .iter()
        .find(|g| g.game == game)
        .map(|g| g.phase)
        != phase
    {
        attempts += 1;
        assert!(attempts < 250, "game {game} is never {phase:?}");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn an_abandoned_game_pauses_then_goes_void_at_its_deadlines() {
    let limits = Limits {
        empty_game_ms: 200,
        paused_game_ms: 400,
        ..fast()
    };
    let config = config(limits);
    let running = start(&config).await.unwrap();
    let (a, game, _) = created(&running, "bot", Seat::Attacker).await;
    let (mut watcher, _) = (greeted(&running).await, ());
    watcher.join(game, Seat::Spectator, None).await.unwrap();
    // Its player gone, the running game pauses at its deadline: the spectator hears it.
    a.close().await;
    listed_as(&running, game, Some(Phase::Paused)).await;
    assert!(matches!(
        watcher.recv().await,
        Some(ServerMessage::Joined { .. })
    ));
    assert!(matches!(
        watcher.recv().await,
        Some(ServerMessage::Snapshot { .. })
    ));
    assert_eq!(goodbye(&mut watcher).await, GoodbyeReason::Paused);
    // Paused, nobody resumes it: it goes void, its journal moved aside.
    listed_as(&running, game, None).await;
    assert!(!journal_file(&config.server.data, game).exists());
    let void: Vec<_> = std::fs::read_dir(config.server.data.join("void"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        void.iter()
            .any(|name| name.starts_with(&format!("{game}-"))),
        "{void:?}"
    );
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
        nonce: "00".repeat(32),
        address: String::new(),
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

/// Pongs, until the goodbye: its reason and how many pongs came before it.
async fn pongs_until_goodbye(client: &mut Client) -> (usize, GoodbyeReason) {
    let mut pongs = 0;
    loop {
        match client.recv().await {
            Some(ServerMessage::Pong { .. }) => pongs += 1,
            Some(ServerMessage::Goodbye { reason }) => return (pongs, reason),
            other => panic!("{other:?}"),
        }
    }
}

/// E16 M2 T5b: WebSocket pings count in a connection's rate, and an address's connections share
/// one rate besides their own.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pings_count_in_the_rate_and_an_address_shares_one() {
    let limits = Limits {
        messages_per_second: 5,
        ..Limits::default()
    };
    let running = start(&config(limits)).await.unwrap();
    let mut c = greeted(&running).await;
    for _ in 0..20 {
        if c.send_raw(Message::Ping(vec![1].into())).await.is_err() {
            break;
        }
    }
    assert_eq!(goodbye(&mut c).await, GoodbyeReason::RateLimited);
    let limits = Limits {
        messages_per_second: 100,
        messages_per_address: 6,
        ..Limits::default()
    };
    let running = start(&config(limits)).await.unwrap();
    let (mut calm, mut flood) = (greeted(&running).await, greeted(&running).await);
    for nonce in 0..20 {
        if flood.send(&ClientMessage::Ping { nonce }).await.is_err() {
            break;
        }
    }
    let (pongs, reason) = pongs_until_goodbye(&mut flood).await;
    assert_eq!(reason, GoodbyeReason::RateLimited);
    assert!(
        pongs < 20,
        "within its own rate, beyond its address's: {pongs}"
    );
    // The other connection of the address goes on in the next second.
    tokio::time::sleep(Duration::from_millis(1_100)).await;
    calm.send(&ClientMessage::Ping { nonce: 7 }).await.unwrap();
    assert_eq!(calm.recv().await, Some(ServerMessage::Pong { nonce: 7 }));
}

/// E16 M2 T5b: a connection that says no hello, or that is in no game, holds its place only for a
/// moment; one in a game is not timed.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_connection_without_a_hello_or_a_game_is_cut_off() {
    let limits = Limits {
        hello_timeout_ms: 200,
        lobby_timeout_ms: 400,
        ..Limits::default()
    };
    let running = start(&config(limits)).await.unwrap();
    let mut silent = Client::connect(running.address).await.unwrap();
    assert_eq!(goodbye(&mut silent).await, GoodbyeReason::IdleTimeout);
    let (mut seated, _, _) = created(&running, "pvp", Seat::Attacker).await;
    let mut browsing = greeted(&running).await;
    let mut lists = 0;
    let reason = loop {
        browsing
            .send(&ClientMessage::ListGames { seq: 1 })
            .await
            .unwrap();
        match browsing.recv().await {
            Some(ServerMessage::Games { .. }) => lists += 1,
            Some(ServerMessage::Goodbye { reason }) => break reason,
            other => panic!("{other:?}"),
        }
        assert!(lists < 100, "never cut off");
        tokio::time::sleep(Duration::from_millis(20)).await;
    };
    assert_eq!(reason, GoodbyeReason::IdleTimeout, "messages do not help");
    // The creator, in its game all along, still is.
    seated
        .send(&ClientMessage::ListGames { seq: 2 })
        .await
        .unwrap();
    assert!(matches!(
        seated.recv().await,
        Some(ServerMessage::Games { seq: 2, .. })
    ));
}

/// E16 M2 T5b (the M2 T1 inbox): a connection whose join a game refused is in that game no more:
/// it does not keep a waiting game listed after its creator left, and the lobby's deadline holds
/// for it. A join it sent before it heard the refusal still counts.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_refused_join_leaves_the_game() {
    let limits = Limits {
        lobby_timeout_ms: 500,
        ..fast()
    };
    let running = start(&config(limits)).await.unwrap();
    let (creator, game, _) = created(&running, "pvp", Seat::Attacker).await;
    let mut late = greeted(&running).await;
    late.join(game, Seat::Attacker, None).await.unwrap();
    assert_eq!(
        late.recv().await,
        Some(ServerMessage::Refused {
            reason: Refusal::SeatTaken
        })
    );
    creator.close().await;
    let mut attempts = 0;
    while !listed(&running).await.is_empty() {
        attempts += 1;
        assert!(attempts < 250, "the refused connection keeps the game");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(goodbye(&mut late).await, GoodbyeReason::IdleTimeout);
    // Two joins at once: the first refused, the second takes the seat it asked for.
    let (creator, game, _) = created(&running, "pvp", Seat::Attacker).await;
    let mut eager = greeted(&running).await;
    eager.join(game, Seat::Attacker, None).await.unwrap();
    eager.join(game, Seat::Defender, None).await.unwrap();
    assert!(matches!(
        eager.recv().await,
        Some(ServerMessage::Refused {
            reason: Refusal::SeatTaken
        })
    ));
    joined(&mut eager).await;
    tokio::time::sleep(Duration::from_millis(700)).await;
    assert!(
        listed(&running)
            .await
            .iter()
            .any(|g| g.game == game && g.phase == Phase::Running),
        "both seats held"
    );
    eager
        .send(&ClientMessage::ListGames { seq: 4 })
        .await
        .unwrap();
    let after = loop {
        match eager.recv().await {
            Some(ServerMessage::Games { seq, .. }) => break seq,
            Some(ServerMessage::Goodbye { reason }) => panic!("{reason:?}"),
            Some(_) => {}
            None => panic!("closed"),
        }
    };
    assert_eq!(after, 4, "in its game past the lobby's deadline");
    drop(creator);
}

/// E16 M2 T5e (the reviewer's finding): joins a game refuses do not restart the lobby's deadline,
/// so a connection that only ever asks for a taken seat holds its place no longer than one that
/// says nothing.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn refused_joins_do_not_keep_a_connection_in_the_lobby() {
    let limits = Limits {
        lobby_timeout_ms: 500,
        ..fast()
    };
    let running = start(&config(limits)).await.unwrap();
    let (_creator, game, _) = created(&running, "pvp", Seat::Attacker).await;
    let mut late = greeted(&running).await;
    let mut refusals = 0;
    let reason = loop {
        late.join(game, Seat::Attacker, None).await.unwrap();
        match late.recv().await {
            Some(ServerMessage::Refused {
                reason: Refusal::SeatTaken,
            }) => refusals += 1,
            Some(ServerMessage::Goodbye { reason }) => break reason,
            other => panic!("{other:?}"),
        }
        assert!(refusals < 30, "never cut off");
        tokio::time::sleep(Duration::from_millis(100)).await;
    };
    assert_eq!(reason, GoodbyeReason::IdleTimeout);
}

/// E16 M2 T5e (the reviewer's finding): a created game's first message is its creator's join,
/// queued as the registry creates it, so a stranger's join that reaches the new game at once does
/// not take the creator's seat.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_created_games_creator_joins_before_anyone_else() {
    let running = start(&config(fast())).await.unwrap();
    // The creator's side of a creation, as its task does it, but nobody reads its queue yet.
    let (to, mut from_game) = game::outbox(16, 1 << 20);
    let asked = ("pvp", Seat::Attacker, true);
    let creator = (ConnId(u64::MAX), to, None);
    let local = [127, 0, 0, 1].into();
    let (game, _) = registry::create(&running.registry, local, asked, creator).unwrap();
    let mut stranger = greeted(&running).await;
    stranger.join(game, Seat::Attacker, None).await.unwrap();
    assert_eq!(
        stranger.recv().await,
        Some(ServerMessage::Refused {
            reason: Refusal::SeatTaken
        })
    );
    let first = from_game.recv().await.unwrap();
    let first = codec::decode_server(first.text.as_bytes(), usize::MAX).unwrap();
    assert!(matches!(first, ServerMessage::Joined { .. }), "{first:?}");
}

/// E16 M2 T5c: a seated client's record, fetched from the server, keeps its latest messages within
/// `[limits] record_bytes`, however much the client asks.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_record_stays_within_its_bound() {
    let bound = 4096;
    let limits = Limits {
        record_bytes: bound,
        ..fast()
    };
    let running = start(&config(limits)).await.unwrap();
    let (mut client, _, _) = created(&running, "pvp", Seat::Attacker).await;
    for seq in 1..=100 {
        let query = Query::CastTargets { spell: 0 };
        client
            .send(&ClientMessage::Query { seq, query })
            .await
            .unwrap();
    }
    client
        .send(&ClientMessage::FetchRecord { seq: 101 })
        .await
        .unwrap();
    let (n, record) = loop {
        match client.recv().await {
            Some(ServerMessage::QueryResult { .. }) => {}
            Some(ServerMessage::Record { n, record, .. }) => break (n, record),
            other => panic!("{other:?}"),
        }
    };
    let mut lines = record.lines();
    let header: RecordHeader = serde_json::from_str(lines.next().unwrap()).unwrap();
    let lines: Vec<&str> = lines.collect();
    assert!(
        n > 100 && header.first > 1,
        "the oldest went: {n}, {}",
        header.first
    );
    assert_eq!(header.first as usize + lines.len() - 1, n as usize);
    let held: usize = lines.iter().map(|l| l.len() + 1).sum();
    assert!(held <= bound, "{held}");
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
        "messages_per_address = 0",
        "connections = 0",
        "connections_per_address = 0",
        "idle_timeout_ms = 0",
        "idle_timeout_ms = 9223372036854775807",
        "ping_every_ms = 86400001",
        "ping_timeout_ms = 0",
        "hello_timeout_ms = 0",
        "lobby_timeout_ms = 86400001",
        "outbound_queue = 0",
        "outbound_queue = 4000000",
        "outbound_bytes = 0",
        "record_bytes = 0",
        "games = 0",
        "games = 5000",
        "games_per_address = 0",
        "spectators_per_game = 1025",
        "empty_game_ms = 31536000001",
        "paused_game_ms = 31536000001",
        "failed_logins_per_address = 0",
        "account_changes_per_address = 0",
        "commands_per_game = 0",
        "commands_per_game = 1000001",
    ] {
        let text = format!("{offer}[limits]\n{limit}");
        let error = refused(&text).unwrap_err();
        assert!(error.0.contains("outside"), "{limit}: {error}");
    }
    assert!(
        refused(&format!("{offer}[limits]\nreplays_kept = 0"))
            .unwrap_err()
            .0
            .contains("outside")
    );
    let doubled = refused(&format!("{offer}[limits]\nrecord_bytes = 1048576")).unwrap_err();
    assert!(doubled.0.contains("fit twice"), "{doubled}");
    // A replay of the longest game fits in the outbound queue (E16 M3 T4a).
    let long = refused(&format!("{offer}[limits]\ncommands_per_game = 20000")).unwrap_err();
    assert!(long.0.contains("its replay"), "{long}");
    // A live client says nothing between two pings (SRV-Q20); without pings, no timeout applies.
    let pings = refused(&format!("{offer}[limits]\nping_every_ms = 30000")).unwrap_err();
    assert!(pings.0.contains("longer than ping_every_ms"), "{pings}");
    let off = format!("{offer}[limits]\nping_every_ms = 0\nping_timeout_ms = 100");
    assert!(refused(&off).is_ok());
    // The server's id is a WebSocket address (E16 M3 T2a).
    for url in [
        "",
        "ws://",
        "http://h3.example",
        "wss://h3 example",
        "h3.example",
    ] {
        let error = refused(&format!("{offer}[server]\npublic_url = \"{url}\"")).unwrap_err();
        assert!(error.0.contains("public_url"), "{url}: {error}");
        // And so is each of its other addresses (E16 M6 T1).
        let other = format!("{offer}[server]\naddresses = [\"ws://192.168.1.5:7878\", \"{url}\"]");
        let error = refused(&other).unwrap_err();
        assert!(error.0.contains("server.addresses"), "{url}: {error}");
    }
    let public = refused(&format!(
        "{offer}[server]\npublic_url = \"wss://h3.example:443/x\"\nanonymous = false"
    ))
    .unwrap();
    assert_eq!(
        (public.server.public_url.as_deref(), public.server.anonymous),
        (Some("wss://h3.example:443/x"), false)
    );
    assert_eq!(
        (minimal.server.public_url, minimal.server.anonymous),
        (None, true)
    );
    assert!(minimal.server.addresses.is_empty());
    let watched = refused(&format!("{offer}[limits]\nspectators_per_game = 0")).unwrap();
    assert_eq!(watched.limits.spectators_per_game, 0, "no spectators");
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
