//! The ladders' configuration and their seasons (E16 M4 T1); over loopback, their queues and
//! matches (E16 M4 T2): two logged-in players queue, accept and play their match's game, listed
//! with its ladder and restored after a restart; a decline, an expiry and a departure put the
//! other back in the queue; an anonymous connection, a second queue and a second match on the
//! ladder are refused; a queued connection outlasts the lobby's deadline. A ranked game's seats
//! and forfeits (E16 M4 T3a): no host; a player back within the grace plays on; beyond the grace,
//! or the total in several drops, a forfeit, whose replay verifies; both away, void; the time away
//! across a restart.

use h3_protocol::wire::{LadderInfo, Outcome, Points, Side, UnqueueReason};

use super::*;
use crate::client::Key;
use crate::config::Ladder;
use crate::ladders::points;
use crate::rating::{self, Rating, Score};

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
    // An offer may not be named as a ladder's pool names its scenarios (E16 M4 T2).
    let named = format!(
        "[content]\npacks = \"p\"\n[[offer]]\nid = \"arena.1\"\nscenario = \"s\"\n{ladder}"
    );
    let error = Config::parse(&named, &base).unwrap_err().0;
    assert!(error.contains("names its pool's scenarios so"), "{error}");
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
    rating(&mut ann).await;
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

/// A server with the ladder `arena` whose players may be away `grace_ms` at once and `total_ms`
/// in a game (E16 M4 T3a).
fn graced(grace_ms: u64, total_ms: u64) -> Config {
    let mut config = laddered(60_000);
    config.ladders[0].grace_ms = grace_ms;
    config.ladders[0].grace_total_ms = total_ms;
    config
}

/// Ann and Bob, matched in `arena` and seated in their game: the clients, the game and their seats.
async fn seated_match(running: &Running) -> ([Client; 2], u32, [Seat; 2]) {
    let (mut ann, mut bob) = (
        player(running, "ann", 1).await,
        player(running, "bob", 2).await,
    );
    matched(&mut ann, &mut bob).await;
    let (game, seats) = accepted(&mut ann, &mut bob).await;
    takes(&mut ann, game, seats[0]).await;
    takes(&mut bob, game, seats[1]).await;
    ([ann, bob], game, seats)
}

/// The end of the game `client` hears, whatever comes before it.
async fn ends(client: &mut Client) -> Outcome {
    let end = async {
        loop {
            match client.recv().await {
                Some(ServerMessage::GameEnded { result, .. }) => return result,
                Some(_) => {}
                None => panic!("closed before the end"),
            }
        }
    };
    tokio::time::timeout(GAME_TIME, end).await.unwrap()
}

/// The side a seat plays.
fn side(seat: Seat) -> Side {
    match seat {
        Seat::Defender => Side::Defender,
        Seat::Attacker | Seat::Spectator => Side::Attacker,
    }
}

/// The rating `client` hears after its game's end (E16 M4 T3b): before and after it.
async fn rating(client: &mut Client) -> (Points, Points) {
    match client.recv().await {
        Some(ServerMessage::Rating {
            ladder,
            before,
            after,
            ..
        }) if ladder == "arena" => (before, after),
        other => panic!("{other:?}"),
    }
}

/// What a game that ended with `result` scores for the player of `side`.
fn score(result: Outcome, side: Side) -> Score {
    let won = |winner: Side| {
        if winner == side {
            Score::Win
        } else {
            Score::Loss
        }
    };
    match result {
        Outcome::Victory { side } => won(side),
        Outcome::Retreat { side } | Outcome::Surrender { side } | Outcome::Forfeit { side } => {
            won(side).reversed()
        }
        Outcome::Draw {} => Score::Draw,
        Outcome::NoShow {} | Outcome::Unverified {} => panic!("unrated: {result:?}"),
    }
}

/// The client queues in `arena` as soon as its game, over, lets it.
async fn queues_again(client: &mut Client) {
    let mut attempts = 0;
    while ask(client, queue("arena")).await != queued() {
        attempts += 1;
        assert!(attempts < 250, "never queued again");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_player_back_within_the_grace_retakes_the_seat_and_plays_on() {
    let config = laddered(60_000);
    let running = start(&config).await.unwrap();
    let ([mut ann, bob], game, [ann_seat, _]) = seated_match(&running).await;
    // A ranked game has no host: neither player pauses it alone (Ann's turn, if it is hers, comes
    // again when she takes her seat again).
    ann.send(&ClientMessage::Pause { seq: 7 }).await.unwrap();
    loop {
        match ann.recv().await {
            Some(ServerMessage::Rejected { seq: 7, code, .. }) => {
                assert_eq!(code, RejectCode::NotHost);
                break;
            }
            Some(_) => {}
            None => panic!("closed"),
        }
    }
    // Ann drops and comes back on a new connection, as her account: the game goes on.
    ann.close().await;
    let mut ann = player(&running, "ann", 1).await;
    takes(&mut ann, game, ann_seat).await;
    let ((_, ann_saw), (_, bob_saw)) = tokio::join!(play(ann, 5), play(bob, 6));
    for saw in [ann_saw, bob_saw] {
        let result = end(&saw);
        assert!(
            !matches!(result, Outcome::Forfeit { .. } | Outcome::NoShow {}),
            "{result:?}"
        );
    }
    running.shutdown().await;
    std::fs::remove_dir_all(&config.server.data).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_player_away_beyond_the_grace_forfeits_and_the_replay_holds_the_game_so_far() {
    let config = graced(1_000, 60_000);
    let running = start(&config).await.unwrap();
    let ([mut ann, bob], game, [_, bob_seat]) = seated_match(&running).await;
    bob.close().await;
    let forfeit = Outcome::Forfeit {
        side: side(bob_seat),
    };
    assert_eq!(ends(&mut ann).await, forfeit);
    // Rated (E16 M4 T3b): a win for Ann, as Glicko-2 rates one game between new accounts.
    let new = Rating::default();
    let (won, lost) = rating::game(new, new, Score::Win);
    assert_eq!(rating(&mut ann).await, (points(new), points(won)));
    let standing = |login| running.standing("arena", 1, login).map(|s| s.rating);
    assert_eq!((standing("ann"), standing("bob")), (Some(won), Some(lost)));
    // The replay holds the battle as it stood, undecided, and verifies.
    ann.send(&ClientMessage::FetchReplay { seq: 900 })
        .await
        .unwrap();
    let Some(ServerMessage::Replay { replay, .. }) = ann.recv().await else {
        panic!("no replay")
    };
    let path = replay_file(&config.server.data, game);
    assert_eq!(replay, std::fs::read_to_string(&path).unwrap());
    assert!(replay.contains("winner = \"none\""), "{replay}");
    h3_cli::verify_file(&path, &config.content.packs).unwrap();
    // Bob takes his seat in the game over; nothing more happens in it.
    let mut bob = player(&running, "bob", 2).await;
    takes(&mut bob, game, bob_seat).await;
    let more = tokio::time::timeout(Duration::from_millis(300), bob.recv()).await;
    assert!(more.is_err(), "{more:?}");
    // Over, it keeps neither from the ladder.
    queues_again(&mut ann).await;
    queues_again(&mut bob).await;
    running.shutdown().await;
    std::fs::remove_dir_all(&config.server.data).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_player_away_beyond_the_total_in_several_drops_forfeits() {
    // Each drop is shorter than the grace (4 s); the three add up beyond the total (2.5 s).
    let config = graced(4_000, 2_500);
    let running = start(&config).await.unwrap();
    let ([ann, mut bob], game, [ann_seat, _]) = seated_match(&running).await;
    let mut ann = Some(ann);
    for _ in 0..2 {
        ann.take().unwrap().close().await;
        tokio::time::sleep(Duration::from_millis(900)).await;
        let mut back = player(&running, "ann", 1).await;
        takes(&mut back, game, ann_seat).await;
        ann = Some(back);
    }
    let dropped = tokio::time::Instant::now();
    ann.take().unwrap().close().await;
    let forfeit = Outcome::Forfeit {
        side: side(ann_seat),
    };
    assert_eq!(ends(&mut bob).await, forfeit);
    assert!(
        dropped.elapsed() < Duration::from_millis(4_000),
        "the total, not the grace: {:?}",
        dropped.elapsed()
    );
    running.shutdown().await;
    std::fs::remove_dir_all(&config.server.data).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn both_players_away_the_game_is_void() {
    let config = graced(1_000, 60_000);
    let running = start(&config).await.unwrap();
    let (mut ann, mut bob) = (
        player(&running, "ann", 1).await,
        player(&running, "bob", 2).await,
    );
    matched(&mut ann, &mut bob).await;
    let (game, _) = accepted(&mut ann, &mut bob).await;
    // Neither comes to the game, counted from its creation; its spectator hears it end.
    let mut watcher = greeted(&running).await;
    watcher.join(game, Seat::Spectator, None).await.unwrap();
    assert!(matches!(
        watcher.recv().await,
        Some(ServerMessage::Joined { .. })
    ));
    assert_eq!(ends(&mut watcher).await, Outcome::NoShow {});
    // Unrated, a no-show each in the ladder's table (E16 M4 T3b).
    for login in ["ann", "bob"] {
        let row = running.standing("arena", 1, login).unwrap();
        assert_eq!(
            (row.rating, row.games, row.no_shows),
            (Rating::default(), 0, 1)
        );
    }
    queues_again(&mut ann).await;
    queues_again(&mut bob).await;
    running.shutdown().await;
    std::fs::remove_dir_all(&config.server.data).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_players_time_away_holds_across_a_restart() {
    // Bob is away 3 s of his 4 s before the restart; after it, the rest of the total forfeits him,
    // long before the grace (10 s) or a total counted afresh.
    let config = graced(10_000, 4_000);
    let running = start(&config).await.unwrap();
    let ([_ann, bob], game, [ann_seat, bob_seat]) = seated_match(&running).await;
    bob.close().await;
    tokio::time::sleep(Duration::from_millis(3_000)).await;
    let mut bob = player(&running, "bob", 2).await;
    takes(&mut bob, game, bob_seat).await;
    // The server stops; it starts again on the journals as they were.
    let restarted = Config {
        server: Server {
            data: crashed(&config.server.data),
            ..config.server.clone()
        },
        ..config.clone()
    };
    running.shutdown().await;
    let again = start(&restarted).await.unwrap();
    let begun = tokio::time::Instant::now();
    let mut ann = player(&again, "ann", 1).await;
    takes(&mut ann, game, ann_seat).await;
    // The restored match keeps its players from another on the ladder.
    let mut bob = player(&again, "bob", 2).await;
    assert_eq!(
        ask(&mut bob, queue("arena")).await,
        refused(Refusal::InRankedGame)
    );
    let forfeit = Outcome::Forfeit {
        side: side(bob_seat),
    };
    assert_eq!(ends(&mut ann).await, forfeit);
    assert!(
        begun.elapsed() < Duration::from_millis(3_500),
        "the total held: {:?}",
        begun.elapsed()
    );
    again.shutdown().await;
    for data in [&config.server.data, &restarted.server.data] {
        std::fs::remove_dir_all(data).unwrap();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_played_game_is_rated_and_the_ratings_hold_across_a_restart() {
    let config = laddered(60_000);
    let running = start(&config).await.unwrap();
    let ([ann, bob], first, [ann_seat, bob_seat]) = seated_match(&running).await;
    let ((mut ann, ann_saw), (mut bob, bob_saw)) = tokio::join!(play(ann, 5), play(bob, 6));
    let result = end(&ann_saw);
    assert_eq!(result, end(&bob_saw));
    // Each hears its rating after the end, as Glicko-2 rates the game between new accounts.
    let new = Rating::default();
    let (ann_after, bob_after) = rating::game(new, new, score(result, side(ann_seat)));
    assert_eq!(
        score(result, side(bob_seat)),
        score(result, side(ann_seat)).reversed()
    );
    assert_eq!(rating(&mut ann).await, (points(new), points(ann_after)));
    assert_eq!(rating(&mut bob).await, (points(new), points(bob_after)));
    let row = |running: &Running, login| running.standing("arena", 1, login).unwrap();
    let rows = [row(&running, "ann"), row(&running, "bob")];
    assert_eq!(
        rows.map(|r| (r.rating, r.games)),
        [(ann_after, 1), (bob_after, 1)]
    );
    // The server stops and starts again: the ratings read back as they were written, and no game
    // id is handed out twice.
    running.shutdown().await;
    let again = start(&config).await.unwrap();
    assert_eq!([row(&again, "ann"), row(&again, "bob")], rows);
    let (_, next, _) = created(&again, "pvp", Seat::Attacker).await;
    assert!(next > first, "{next} after {first}");
    let log = std::fs::read_to_string(config.server.data.join("ladders/arena.log")).unwrap();
    assert_eq!(log.lines().count(), 2, "the season and the game: {log}");
    assert!(log.contains(&format!("\"game\":{first},")), "{log}");
    again.shutdown().await;
    std::fs::remove_dir_all(&config.server.data).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_ranked_game_that_does_not_verify_is_void_unrated_and_its_players_told() {
    let config = laddered(60_000);
    let running = start(&config).await.unwrap();
    let ([ann, bob], game, _) = seated_match(&running).await;
    // The journal's seed changes under the game: its re-run cannot end as the game did.
    let seed = |line: &str| {
        let at = line.find("seed = \\\"")? + 9;
        let end = at + line[at..].find('\\')? - 1;
        let digit = if &line[end..=end] == "0" { "1" } else { "0" };
        Some(format!("{}{digit}{}", &line[..end], &line[end + 1..]))
    };
    flip(&config.server.data, game, &seed);
    let ((mut ann, ann_saw), (_, bob_saw)) = tokio::join!(play(ann, 5), play(bob, 6));
    for saw in [&ann_saw, &bob_saw] {
        assert_eq!(end(saw), Outcome::Unverified {});
    }
    // No rating follows, no row, the replay withheld.
    ann.send(&ClientMessage::FetchReplay { seq: 900 })
        .await
        .unwrap();
    assert!(
        matches!(
            ann.recv().await,
            Some(ServerMessage::Rejected {
                seq: 900,
                code: RejectCode::Unverified,
                ..
            })
        ),
        "no rating"
    );
    assert_eq!(running.standing("arena", 1, "ann"), None);
    assert!(config.server.data.join("void").read_dir().unwrap().count() > 0);
    // Over, it keeps neither from the ladder.
    queues_again(&mut ann).await;
    running.shutdown().await;
    std::fs::remove_dir_all(&config.server.data).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_seat_whose_client_falls_silent_is_noticed_by_its_pings() {
    // SRV-Q20: Bob's client stops reading (a network gone, no close). His connection answers no
    // ping and closes within the ping timeout, and the grace runs from there: he forfeits in
    // seconds, where the idle timeout (5 minutes) would have held his seat.
    // A test client answers pings only while it reads: the timeout outlasts the matcher's second,
    // in which one of the two waits unread.
    let mut config = graced(1_000, 60_000);
    config.limits.ping_every_ms = 500;
    config.limits.ping_timeout_ms = 2_500;
    let running = start(&config).await.unwrap();
    let ([mut ann, mut bob], _, [_, bob_seat]) = seated_match(&running).await;
    let silent = tokio::time::Instant::now();
    let forfeit = Outcome::Forfeit {
        side: side(bob_seat),
    };
    // Ann reads on, and her client answers the pings meanwhile.
    assert_eq!(ends(&mut ann).await, forfeit);
    assert!(
        silent.elapsed() < Duration::from_secs(10),
        "{:?}",
        silent.elapsed()
    );
    rating(&mut ann).await;
    // Bob's connection is gone: what his client did not read ends with the goodbye, or with the
    // reset his late pong draws from the closed socket (which drops the unread rest).
    let rest = async {
        while let Some(message) = bob.recv().await {
            if let ServerMessage::Goodbye { reason } = message {
                assert_eq!(reason, GoodbyeReason::IdleTimeout);
            }
        }
    };
    tokio::time::timeout(Duration::from_secs(5), rest)
        .await
        .unwrap();
    running.shutdown().await;
    std::fs::remove_dir_all(&config.server.data).unwrap();
}
