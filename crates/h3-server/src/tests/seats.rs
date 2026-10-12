//! Seats bound to accounts over loopback (E16 M3 T3): a seat a logged-in connection takes is its
//! account's, re-taken by a new connection logged in as it without a token, after a drop, to
//! resume a paused game and after a restart; another account, an anonymous connection and the
//! seat's token are refused; the lobby names the seats' accounts; a replay is listed only if no
//! seated account opted out; a re-key closes the account's other connections.

use h3_protocol::wire::Seated;

use super::*;
use crate::client::Key;

/// A key of a fixed seed.
fn key(n: u8) -> Key {
    Key::from_seed([n; 32])
}

/// A client logged in as `login` with `key`, registered as `display` if the account is new.
async fn logged_in(running: &Running, login: &str, key: &Key, display: &str) -> Client {
    let mut client = greeted(running).await;
    let answer = client.login(1, login, key, Some(display)).await.unwrap();
    assert!(
        matches!(answer, Some(ServerMessage::Account { .. })),
        "{answer:?}"
    );
    client
}

/// The `Joined` and `Snapshot` after a join to an account's seat: no token.
async fn joined_as_account(client: &mut Client) {
    let joined = client.recv().await;
    assert!(
        matches!(joined, Some(ServerMessage::Joined { token: None, .. })),
        "{joined:?}"
    );
    assert!(matches!(
        client.recv().await,
        Some(ServerMessage::Snapshot { .. })
    ));
}

/// A logged-in client that created a game of `offer` and took `seat`: the game.
async fn created_as_account(client: &mut Client, offer: &str, seat: Seat) -> u32 {
    client.create(offer, seat).await.unwrap();
    let created = client.recv().await;
    let Some(ServerMessage::GameCreated { game, .. }) = created else {
        panic!("{created:?}")
    };
    joined_as_account(client).await;
    game
}

/// What `client` hears first when it joins `seat` of `game` with `token`.
async fn join_answer(
    client: &mut Client,
    game: u32,
    seat: Seat,
    token: Option<&str>,
) -> Option<ServerMessage> {
    client
        .join(game, seat, token.map(str::to_owned))
        .await
        .unwrap();
    client.recv().await
}

/// The goodbye a client gets after whatever its game still sent it.
async fn farewell(client: &mut Client) -> GoodbyeReason {
    loop {
        match client.recv().await {
            Some(ServerMessage::Goodbye { reason }) => return reason,
            Some(_) => {}
            None => panic!("closed without a goodbye"),
        }
    }
}

/// The attacker's token the journal of game `game` holds in its first line: the operator's copy.
fn journaled_token(data: &Path, game: u32) -> String {
    let text = std::fs::read_to_string(journal_file(data, game)).unwrap();
    let start: serde_json::Value = serde_json::from_str(text.lines().next().unwrap()).unwrap();
    start["tokens"][0].as_str().unwrap().to_owned()
}

/// Game `game` as the lobby lists it: its phase and the seats it names.
async fn listing(running: &Running, game: u32) -> (Phase, Vec<Seated>) {
    let games = listed(running).await;
    let info = games.into_iter().find(|g| g.game == game).unwrap();
    (info.phase, info.seated)
}

/// Waits until the lobby lists game `game` as `expected` (a game tells the lobby after it sent the
/// frames of the same step).
async fn listed_with(running: &Running, game: u32, expected: (Phase, Vec<Seated>)) {
    let mut attempts = 0;
    while listing(running, game).await != expected {
        attempts += 1;
        assert!(
            attempts < 250,
            "game {game} is never listed as {expected:?}"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

const SEAT_TAKEN: Option<ServerMessage> = Some(ServerMessage::Refused {
    reason: Refusal::SeatTaken,
});

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn an_accounts_seat_is_retaken_by_the_account_alone_also_paused_and_after_a_restart() {
    let config = config(fast());
    let running = start(&config).await.unwrap();
    let alice = key(1);
    let mut first = logged_in(&running, "alice", &alice, "Alice").await;
    let game = created_as_account(&mut first, "bot", Seat::Attacker).await;
    let seen = play_turns(&mut first, 5, 2).await;
    assert!(matches!(seen.last(), Some(ServerMessage::YourTurn { .. })));
    let alices = vec![Seated {
        seat: Seat::Attacker,
        display: String::from("Alice"),
    }];
    assert_eq!(
        listing(&running, game).await,
        (Phase::Running, alices.clone())
    );
    // After a drop, neither another account, nor an anonymous connection, nor the seat's token
    // (the journal's copy) takes the account's seat.
    first.close().await;
    let token = journaled_token(&config.server.data, game);
    let mut bob = logged_in(&running, "bob", &key(2), "Bob").await;
    assert_eq!(
        join_answer(&mut bob, game, Seat::Attacker, None).await,
        SEAT_TAKEN
    );
    let mut stranger = greeted(&running).await;
    let with_token = join_answer(&mut stranger, game, Seat::Attacker, Some(&token)).await;
    assert_eq!(with_token, SEAT_TAKEN);
    // A new connection logged in as the account re-takes it without a token; another replaces it.
    let mut second = logged_in(&running, "alice", &alice, "Alice").await;
    second.join(game, Seat::Attacker, None).await.unwrap();
    joined_as_account(&mut second).await;
    let mut third = logged_in(&running, "alice", &alice, "Alice").await;
    third.join(game, Seat::Attacker, None).await.unwrap();
    joined_as_account(&mut third).await;
    assert_eq!(farewell(&mut second).await, GoodbyeReason::Replaced);
    // Paused by its host, the game resumes for the account alone.
    third.send(&ClientMessage::Pause { seq: 9 }).await.unwrap();
    assert_eq!(farewell(&mut third).await, GoodbyeReason::Paused);
    listed_as(&running, game, Some(Phase::Paused)).await;
    assert_eq!(
        listing(&running, game).await,
        (Phase::Paused, alices.clone())
    );
    assert_eq!(
        join_answer(&mut bob, game, Seat::Attacker, None).await,
        SEAT_TAKEN
    );
    let mut stranger = greeted(&running).await;
    let with_token = join_answer(&mut stranger, game, Seat::Attacker, Some(&token)).await;
    assert_eq!(with_token, SEAT_TAKEN);
    let mut fourth = logged_in(&running, "alice", &alice, "Alice").await;
    fourth.join(game, Seat::Attacker, None).await.unwrap();
    joined_as_account(&mut fourth).await;
    listed_as(&running, game, Some(Phase::Running)).await;
    // A crash: the next start binds the seat again, from the journal and the player log.
    let restarted = Config {
        server: Server {
            data: crashed(&config.server.data),
            ..config.server.clone()
        },
        ..config.clone()
    };
    let again = start(&restarted).await.unwrap();
    assert_eq!(listing(&again, game).await, (Phase::Running, alices));
    let mut stranger = greeted(&again).await;
    let with_token = join_answer(&mut stranger, game, Seat::Attacker, Some(&token)).await;
    assert_eq!(with_token, SEAT_TAKEN);
    let mut back = logged_in(&again, "alice", &alice, "Alice").await;
    back.join(game, Seat::Attacker, None).await.unwrap();
    joined_as_account(&mut back).await;
    let (mut back, seen) = tokio::time::timeout(GAME_TIME, play(back, 7))
        .await
        .unwrap();
    assert!(matches!(seen.last(), Some(ServerMessage::GameEnded { .. })));
    fetched_replay(&mut back, &restarted, game).await;
    assert_eq!(archive_page(&again, 0).await.0, 1, "listed: Alice lets it");
    again.shutdown().await;
}

/// Plays a bot game to its end as the logged-in `client`'s attacker; the replay fetched, so the
/// archive holds it.
async fn played(client: Client, seed: u64) -> Client {
    let (mut client, seen) = tokio::time::timeout(GAME_TIME, play(client, seed))
        .await
        .unwrap();
    assert!(matches!(seen.last(), Some(ServerMessage::GameEnded { .. })));
    client
        .send(&ClientMessage::FetchReplay { seq: 7 })
        .await
        .unwrap();
    assert!(matches!(
        client.recv().await,
        Some(ServerMessage::Replay { seq: 7, .. })
    ));
    client
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_lobby_names_the_accounts_and_their_preference_lists_the_replay() {
    let running = start(&config(fast())).await.unwrap();
    // The lobby names a seat's account, not an anonymous player's.
    let mut alice = logged_in(&running, "alice", &key(1), "Alice").await;
    let game = created_as_account(&mut alice, "pvp", Seat::Attacker).await;
    let named = vec![Seated {
        seat: Seat::Attacker,
        display: String::from("Alice"),
    }];
    listed_with(&running, game, (Phase::Waiting, named.clone())).await;
    let (anonymous, _) = seated(&running, game, Seat::Defender).await;
    listed_with(&running, game, (Phase::Running, named)).await;
    anonymous.close().await;
    alice.close().await;
    // A seated account that opted out keeps its game's replay unlisted.
    let mut alice = logged_in(&running, "alice", &key(1), "Alice").await;
    let opted_out = alice.set_listed(2, false).await.unwrap();
    assert!(matches!(
        opted_out,
        Some(ServerMessage::Account { listed: false, .. })
    ));
    created_as_account(&mut alice, "bot", Seat::Attacker).await;
    let mut alice = played(alice, 3).await;
    assert_eq!(archive_page(&running, 0).await.0, 0);
    // Its preference changed, its next game's replay is listed.
    let opted_in = alice.set_listed(3, true).await.unwrap();
    assert!(matches!(
        opted_in,
        Some(ServerMessage::Account { listed: true, .. })
    ));
    let mut alice = logged_in(&running, "alice", &key(1), "Alice").await;
    created_as_account(&mut alice, "bot", Seat::Attacker).await;
    played(alice, 4).await;
    assert_eq!(archive_page(&running, 0).await.0, 1);
    running.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_rekey_closes_the_accounts_other_connections() {
    let running = start(&config(fast())).await.unwrap();
    let (old, new) = (key(1), key(3));
    let mut here = logged_in(&running, "alice", &old, "Alice").await;
    let mut there = logged_in(&running, "alice", &old, "Alice").await;
    let game = created_as_account(&mut there, "bot", Seat::Attacker).await;
    let mut bob = logged_in(&running, "bob", &key(2), "Bob").await;
    let rekeyed = here.rekey(2, "alice", &old, &new).await.unwrap();
    assert!(matches!(
        rekeyed,
        Some(ServerMessage::Account { seq: 2, .. })
    ));
    assert_eq!(farewell(&mut there).await, GoodbyeReason::Rekeyed);
    // The re-keying connection stays, the seat is the account's still; other accounts stay.
    here.join(game, Seat::Attacker, None).await.unwrap();
    joined_as_account(&mut here).await;
    bob.send(&ClientMessage::Ping { nonce: 4 }).await.unwrap();
    assert_eq!(bob.recv().await, Some(ServerMessage::Pong { nonce: 4 }));
    // A re-key to the key the account has closes nothing.
    let mut later = logged_in(&running, "alice", &new, "Alice").await;
    let mut fresh = logged_in(&running, "alice", &new, "Alice").await;
    let same = fresh.rekey(2, "alice", &new, &new).await.unwrap();
    assert!(matches!(same, Some(ServerMessage::Account { seq: 2, .. })));
    later.send(&ClientMessage::Ping { nonce: 5 }).await.unwrap();
    assert_eq!(later.recv().await, Some(ServerMessage::Pong { nonce: 5 }));
    running.shutdown().await;
}
