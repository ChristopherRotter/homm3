//! The ladders' configuration and their seasons (E16 M4 T1); over loopback, their queues and
//! matches (E16 M4 T2): two logged-in players queue, accept and play their match's game, listed
//! with its ladder and restored after a restart; a decline, an expiry and a departure put the
//! other back in the queue; an anonymous connection, a second queue and a second match on the
//! ladder are refused; a queued connection outlasts the lobby's deadline.

use h3_protocol::wire::{LadderInfo, UnqueueReason};

use super::*;
use crate::client::Key;
use crate::config::Ladder;

fn arena(profile: Option<&str>) -> Ladder {
    let scenarios = root().join("packs/synthetic/scenarios");
    Ladder {
        id: String::from("arena"),
        name: String::from("Arena"),
        scenarios: vec![scenarios.join("duel.toml")],
        profile: profile.map(String::from),
        ..Ladder::default()
    }
}

#[test]
fn ladders_are_configured_and_checked() {
    let base = root();
    let offer = "[[offer]]\nid = \"a\"\nscenario = \"s.toml\"\n";
    let ladder = "[[ladder]]\nid = \"arena\"\nname = \"The Arena\"\nscenarios = [\"x.toml\"]\n";
    let parse =
        |text: &str| Config::parse(&format!("[content]\npacks = \"p\"\n{offer}{text}"), &base);
    assert!(parse("").unwrap().ladders.is_empty(), "none by default");
    let parsed = parse(ladder).unwrap();
    let expected = Ladder {
        id: String::from("arena"),
        name: String::from("The Arena"),
        scenarios: vec![base.join("x.toml")],
        ..Ladder::default()
    };
    assert_eq!(parsed.ladders, [expected]);
    let defaults = Ladder::default();
    assert_eq!(
        (
            defaults.accept_ms,
            defaults.grace_ms,
            defaults.grace_total_ms
        ),
        (30_000, 300_000, 600_000),
        "the user's 5 and 10 minutes (SRV-Q19)"
    );
    assert_eq!(
        (defaults.pauses, defaults.pause_ms, defaults.spectators),
        (0, 0, true)
    );
    let refused = |text: &str| parse(text).unwrap_err().0;
    for (text, expected) in [
        (format!("{ladder}{ladder}"), "its id is taken"),
        (ladder.replace("arena", "Arena"), "its id"),
        (ladder.replace("arena", "a/b"), "its id"),
        (ladder.replace("arena", &"a".repeat(33)), "its id"),
        (ladder.replace("The Arena", "A"), "not a display name"),
        (ladder.replace("[\"x.toml\"]", "[]"), "no scenarios"),
        (
            format!("{ladder}accept_ms = 999"),
            "accept_ms = 999: outside",
        ),
        (format!("{ladder}grace_ms = 0"), "grace_ms = 0: outside"),
        (
            format!("{ladder}grace_total_ms = 31536000001"),
            "grace_total_ms",
        ),
        (format!("{ladder}pauses = 101"), "pauses = 101: outside"),
        (format!("{ladder}pauses = 1"), "pauses without pause_ms"),
        (format!("{ladder}queue = 1"), "unknown field"),
    ] {
        let error = refused(&text);
        assert!(error.contains(expected), "{expected}: {error}");
    }
    let paused = parse(&format!("{ladder}pauses = 2\npause_ms = 600000")).unwrap();
    assert_eq!(
        (paused.ladders[0].pauses, paused.ladders[0].pause_ms),
        (2, 600_000)
    );
    // The example's ladder loads.
    let example = Config::load(&base.join("crates/h3-server/server.example.toml")).unwrap();
    let ids: Vec<&str> = example.ladders.iter().map(|l| l.id.as_str()).collect();
    assert_eq!(ids, ["arena"]);
    assert!(example.ladders[0].scenarios.iter().all(|s| s.is_file()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_ladder_starts_a_season_when_its_content_changes() {
    let config = Config {
        ladders: vec![arena(None)],
        ..config(fast())
    };
    let running = start(&config).await.unwrap();
    assert_eq!(
        (running.season("arena"), running.season("other")),
        (Some(1), None)
    );
    assert_eq!(running.standing("arena", 1, "alice"), None);
    running.shutdown().await;
    // The same content: the same season.
    let running = start(&config).await.unwrap();
    assert_eq!(running.season("arena"), Some(1));
    running.shutdown().await;
    // Another profile: the next season.
    let changed = Config {
        ladders: vec![arena(Some("synthetic:strict-fixed"))],
        ..config.clone()
    };
    let running = start(&changed).await.unwrap();
    assert_eq!(running.season("arena"), Some(2));
    running.shutdown().await;
    // A results line naming an account the player log does not know stops the start.
    let log = config.server.data.join("ladders/arena.log");
    let mut text = std::fs::read_to_string(&log).unwrap();
    text.push_str("{\"NoShow\":{\"game\":1,\"replay\":\"r\",\"accounts\":[\"alice\",\"bob\"]}}\n");
    std::fs::write(&log, text).unwrap();
    let error = start(&changed).await.unwrap_err();
    assert!(
        error.0.contains("arena.log: line 3: no such account"),
        "{error}"
    );
    std::fs::remove_dir_all(&config.server.data).unwrap();
}

/// A server with the ladder `arena` (the duel; 1 s to accept a match), automated players' limits
/// and the lobby's deadline `lobby_ms`.
fn laddered(lobby_ms: u64) -> Config {
    Config {
        ladders: vec![Ladder {
            accept_ms: 1_000,
            ..arena(None)
        }],
        ..config(Limits {
            lobby_timeout_ms: lobby_ms,
            ..fast()
        })
    }
}

/// A client logged in as `login` (its display name the same, capitalised), its key of seed `n`.
async fn player(running: &Running, login: &str, n: u8) -> Client {
    let mut client = greeted(running).await;
    let display = format!("{}{}", login[..1].to_uppercase(), &login[1..]);
    let key = Key::from_seed([n; 32]);
    let answer = client.login(1, login, &key, Some(&display)).await.unwrap();
    assert!(
        matches!(answer, Some(ServerMessage::Account { .. })),
        "{answer:?}"
    );
    client
}

/// What `client` hears after it sends `message`.
async fn ask(client: &mut Client, message: ClientMessage) -> Option<ServerMessage> {
    client.send(&message).await.unwrap();
    client.recv().await
}

fn queue(ladder: &str) -> ClientMessage {
    ClientMessage::Queue {
        seq: 2,
        ladder: ladder.to_string(),
    }
}

fn queued() -> Option<ServerMessage> {
    Some(ServerMessage::Queued {
        seq: 2,
        ladder: String::from("arena"),
    })
}

fn refused(reason: Refusal) -> Option<ServerMessage> {
    Some(ServerMessage::Refused { reason })
}

fn found() -> Option<ServerMessage> {
    Some(ServerMessage::MatchFound {
        ladder: String::from("arena"),
        accept_ms: 1_000,
    })
}

/// Both clients queue in `arena` and hear their match.
async fn matched(a: &mut Client, b: &mut Client) {
    for client in [&mut *a, &mut *b] {
        assert_eq!(ask(client, queue("arena")).await, queued());
    }
    assert_eq!((a.recv().await, b.recv().await), (found(), found()));
}

/// Both accept their match: the game and each one's seat.
async fn accepted(a: &mut Client, b: &mut Client) -> (u32, [Seat; 2]) {
    for client in [&mut *a, &mut *b] {
        client
            .send(&ClientMessage::AcceptMatch { seq: 3 })
            .await
            .unwrap();
    }
    let ready = |m: Option<ServerMessage>| match m {
        Some(ServerMessage::MatchReady { ladder, game, seat }) if ladder == "arena" => (game, seat),
        other => panic!("{other:?}"),
    };
    let ((game, a_seat), (other, b_seat)) = (ready(a.recv().await), ready(b.recv().await));
    assert_eq!(game, other);
    assert_ne!(a_seat, b_seat);
    (game, [a_seat, b_seat])
}

/// The client takes its account's seat in `game`.
async fn takes(client: &mut Client, game: u32, seat: Seat) {
    client.join(game, seat, None).await.unwrap();
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn two_players_queue_accept_and_play_their_match() {
    let config = laddered(60_000);
    let running = start(&config).await.unwrap();
    // An anonymous connection never queues.
    let mut anonymous = greeted(&running).await;
    assert_eq!(
        ask(&mut anonymous, queue("arena")).await,
        refused(Refusal::NotLoggedIn)
    );
    let ladders = ask(&mut anonymous, ClientMessage::ListLadders { seq: 4 }).await;
    let arena = LadderInfo {
        id: String::from("arena"),
        name: String::from("Arena"),
        season: 1,
        waiting: 0,
    };
    assert_eq!(
        ladders,
        Some(ServerMessage::Ladders {
            seq: 4,
            ladders: vec![arena]
        })
    );
    let (mut ann, mut bob) = (
        player(&running, "ann", 1).await,
        player(&running, "bob", 2).await,
    );
    assert_eq!(
        ask(&mut ann, queue("pit")).await,
        refused(Refusal::NoSuchLadder)
    );
    assert_eq!(
        ask(&mut ann, ClientMessage::AcceptMatch { seq: 3 }).await,
        refused(Refusal::NoMatch)
    );
    assert_eq!(ask(&mut ann, queue("arena")).await, queued());
    assert_eq!(
        ask(&mut ann, queue("arena")).await,
        refused(Refusal::AlreadyQueued)
    );
    assert_eq!(ask(&mut bob, queue("arena")).await, queued());
    assert_eq!((ann.recv().await, bob.recv().await), (found(), found()));
    let (game, [ann_seat, bob_seat]) = accepted(&mut ann, &mut bob).await;
    // The game is listed with its ladder and its pool's offer, its seats the accounts'.
    let info = listed(&running)
        .await
        .into_iter()
        .find(|g| g.game == game)
        .unwrap();
    assert_eq!(
        (
            info.offer.as_str(),
            info.ladder.as_deref(),
            info.phase,
            info.open.len()
        ),
        ("arena.1", Some("arena"), Phase::Waiting, 0)
    );
    // While it is not over, neither queues on the ladder again.
    assert_eq!(
        ask(&mut ann, queue("arena")).await,
        refused(Refusal::InRankedGame)
    );
    // Another account takes no seat of it.
    let mut cid = player(&running, "cid", 3).await;
    cid.join(game, ann_seat, None).await.unwrap();
    assert_eq!(cid.recv().await, refused(Refusal::SeatTaken));
    takes(&mut ann, game, ann_seat).await;
    takes(&mut bob, game, bob_seat).await;
    let ((mut ann, ann_saw), (_, bob_saw)) = tokio::join!(play(ann, 5), play(bob, 6));
    for saw in [ann_saw, bob_saw] {
        assert!(matches!(saw.last(), Some(ServerMessage::GameEnded { .. })));
    }
    // Over, it keeps no one from the ladder.
    let mut attempts = 0;
    while ask(&mut ann, queue("arena")).await != queued() {
        attempts += 1;
        assert!(attempts < 250, "never queued again");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    running.shutdown().await;
    std::fs::remove_dir_all(&config.server.data).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_decline_an_expiry_or_a_departure_puts_the_other_back() {
    // A short lobby deadline: a queued connection outlasts it.
    let config = laddered(300);
    let running = start(&config).await.unwrap();
    let (mut ann, mut bob) = (
        player(&running, "ann", 1).await,
        player(&running, "bob", 2).await,
    );
    matched(&mut ann, &mut bob).await;
    // Bob declines: he leaves the queue; Ann waits again.
    assert_eq!(
        ask(&mut bob, ClientMessage::DeclineMatch { seq: 5 }).await,
        Some(ServerMessage::Dequeued { seq: 5 })
    );
    assert_eq!(ann.recv().await, Some(ServerMessage::MatchCancelled {}));
    assert_eq!(
        ask(&mut bob, ClientMessage::Dequeue { seq: 6 }).await,
        refused(Refusal::NotQueued)
    );
    // Cid comes, and never answers: at the due time he is out, and Ann, who accepted, waits again.
    let mut cid = player(&running, "cid", 3).await;
    assert_eq!(ask(&mut cid, queue("arena")).await, queued());
    assert_eq!((ann.recv().await, cid.recv().await), (found(), found()));
    ann.send(&ClientMessage::AcceptMatch { seq: 7 })
        .await
        .unwrap();
    let expired = Some(ServerMessage::Unqueued {
        reason: UnqueueReason::Expired,
    });
    assert_eq!(
        (ann.recv().await, cid.recv().await),
        (Some(ServerMessage::MatchCancelled {}), expired)
    );
    // Dee is matched with Ann and goes away: Ann waits again.
    let mut dee = player(&running, "dee", 4).await;
    assert_eq!(ask(&mut dee, queue("arena")).await, queued());
    assert_eq!((ann.recv().await, dee.recv().await), (found(), found()));
    dee.close().await;
    assert_eq!(ann.recv().await, Some(ServerMessage::MatchCancelled {}));
    // Ann has waited far beyond the lobby's deadline, and leaves when she asks.
    assert_eq!(
        ask(&mut ann, ClientMessage::Dequeue { seq: 8 }).await,
        Some(ServerMessage::Dequeued { seq: 8 })
    );
    // Out of the queue and in no game, the lobby's deadline runs again.
    assert_eq!(goodbye(&mut ann).await, GoodbyeReason::IdleTimeout);
    running.shutdown().await;
    std::fs::remove_dir_all(&config.server.data).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_matchs_game_is_restored_with_its_ladder() {
    let config = laddered(60_000);
    let running = start(&config).await.unwrap();
    let (mut ann, mut bob) = (
        player(&running, "ann", 1).await,
        player(&running, "bob", 2).await,
    );
    matched(&mut ann, &mut bob).await;
    let (game, [ann_seat, bob_seat]) = accepted(&mut ann, &mut bob).await;
    takes(&mut ann, game, ann_seat).await;
    takes(&mut bob, game, bob_seat).await;
    // The server stops; it starts again on the journals as they were.
    let restarted = Config {
        server: Server {
            data: crashed(&config.server.data),
            ..config.server.clone()
        },
        ..config.clone()
    };
    let again = start(&restarted).await.unwrap();
    let info = listed(&again)
        .await
        .into_iter()
        .find(|g| g.game == game)
        .unwrap();
    assert_eq!(
        (info.offer.as_str(), info.ladder.as_deref(), info.phase),
        ("arena.1", Some("arena"), Phase::Running)
    );
    // Each account takes its seat again, with no token.
    let mut ann = player(&again, "ann", 1).await;
    takes(&mut ann, game, ann_seat).await;
    again.shutdown().await;
    running.shutdown().await;
    for data in [&config.server.data, &restarted.server.data] {
        std::fs::remove_dir_all(data).unwrap();
    }
}
