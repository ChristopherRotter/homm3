//! The ladders' configuration and their seasons (E16 M4 T1); over loopback, their queues and
//! matches (E16 M4 T2): two logged-in players queue, accept and play their match's game, listed
//! with its ladder and restored after a restart; a decline, an expiry and a departure put the
//! other back in the queue; an anonymous connection, a second queue and a second match on the
//! ladder are refused; a queued connection outlasts the lobby's deadline. A ranked game's seats
//! and forfeits (E16 M4 T3a): no host; a player back within the grace plays on; beyond the grace,
//! or the total in several drops, a forfeit, whose replay verifies; both away, void; the time away
//! across a restart. The leaderboards (E16 M4 T4): pages by season, one's own row, a display name
//! changed, a new season's ratings carried with their deviations reset (T7). The bots (E16 M4 T5): a lone player plays one after the wait, told so (T5b), rated,
//! flagged; a player who opted out (T5b) waits on. The pause by agreement (E16 M4 T6): declined,
//! agreed and resumed in time, a player not back after its due time forfeits, neither back: void.
//! M4's exit (E16 M4 T7): two players queue, accept and play a rated battle through a restart,
//! their ratings the same after another; a player queues again as soon as its game ended; ids
//! beyond the ladders' logs without `next-game`.

use h3_protocol::wire::{LadderInfo, LeaderboardRow, Outcome, Points, Seated, Side, UnqueueReason};

use super::*;
use crate::client::Key;
use crate::config::{BotKind, Ladder, LadderBot};
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
    // E16 M4 T5: the ladder's bots.
    assert_eq!(
        (defaults.bots.clone(), defaults.bot_after_ms),
        (vec![], 60_000)
    );
    let bot = |login: &str, display: &str| {
        format!("[[ladder.bot]]\nlogin = \"{login}\"\ndisplay = \"{display}\"\n")
    };
    let botted = parse(&format!(
        "{ladder}bot_after_ms = 0\n{}",
        bot("randy", "Randy")
    ));
    let botted = botted.unwrap();
    assert_eq!(
        (
            botted.ladders[0].bots.clone(),
            botted.ladders[0].bot_after_ms
        ),
        (
            vec![LadderBot {
                login: String::from("randy"),
                display: String::from("Randy"),
                kind: BotKind::Random,
            }],
            0
        )
    );
    let pit = ladder.replace("arena", "pit");
    for (text, expected) in [
        (
            format!("{ladder}{}", bot("Randy", "Randy")),
            "bot `Randy`: not a name",
        ),
        (
            format!("{ladder}{}", bot("randy", " R")),
            "bot `randy`: not a name",
        ),
        (
            format!("{ladder}{}{}", bot("randy", "Randy"), bot("randy", "Randy")),
            "bot `randy`: listed twice",
        ),
        (
            format!(
                "{ladder}{}{pit}{}",
                bot("randy", "Randy"),
                bot("randy", "Rand")
            ),
            "ladder `pit`: bot `randy` differs",
        ),
        (
            format!("{ladder}{}kind = \"smart\"\n", bot("randy", "Randy")),
            "unknown variant",
        ),
        (
            format!("{ladder}bot_after_ms = 31536000001"),
            "bot_after_ms",
        ),
    ] {
        let error = refused(&text);
        assert!(error.contains(expected), "{expected}: {error}");
    }
    // The same bot on two ladders, the same account on each.
    let shared = format!(
        "{ladder}{}{pit}{}",
        bot("randy", "Randy"),
        bot("randy", "Randy")
    );
    assert_eq!(parse(&shared).unwrap().ladders.len(), 2);
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
    // Its games keep no leak of hidden obstacles (BAT-QUIRK-4; the user, 2026-10-19).
    assert_eq!(
        example.ladders[0].profile.as_deref(),
        Some("synthetic:strict-fixed")
    );
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

/// Queue in `ladder`, bots welcome.
fn queue(ladder: &str) -> ClientMessage {
    ClientMessage::Queue {
        seq: 2,
        ladder: ladder.to_string(),
        bots: None,
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

/// A match in `arena`, against a bot or not (E16 M4 T5b).
fn found_against(bot: bool) -> Option<ServerMessage> {
    Some(ServerMessage::MatchFound {
        ladder: String::from("arena"),
        accept_ms: 1_000,
        bot,
    })
}

/// A match in `arena` against a person.
fn found() -> Option<ServerMessage> {
    found_against(false)
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
        bots: vec![],
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
    // The lobby names both seats' accounts, as it did from the game's start (E16 M4 T5).
    let seated = (listed(&running).await.into_iter())
        .find(|g| g.game == game)
        .map(|g| g.seated);
    let named = |seat, display: &str| Seated {
        seat,
        display: display.to_string(),
        bot: false,
    };
    let mut expected = vec![named(ann_seat, "Ann"), named(bob_seat, "Bob")];
    if bob_seat == Seat::Attacker {
        expected.reverse();
    }
    assert_eq!(seated, Some(expected));
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

/// `client` answers its turns until it heard `commands` commands applied, its own or the other
/// seat's (an `Events` each), and stops there: both seats of a game stop after the same command.
async fn plays_until(client: &mut Client, seed: u64, commands: usize) {
    let mut rng = Rng::from_seed(seed);
    let (mut heard, mut seq) = (0, 0);
    while heard < commands {
        match client.recv().await {
            Some(ServerMessage::YourTurn { legal, .. }) => {
                let command = bench::choose(&legal, &mut rng).unwrap();
                seq += 1;
                let command = ClientMessage::Command { seq, command };
                client.send(&command).await.unwrap();
            }
            Some(ServerMessage::Events { .. }) => heard += 1,
            Some(_) => {}
            None => panic!("closed after {heard} commands"),
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn two_players_queue_accept_and_play_a_rated_battle_through_a_restart() {
    // E16 M4 T7: M4's exit criteria, end to end.
    let config = laddered(60_000);
    let running = start(&config).await.unwrap();
    let ([mut ann, mut bob], game, [ann_seat, bob_seat]) = seated_match(&running).await;
    // A command each, and the server stops mid-battle: the duel's stacks start out of each
    // other's reach, and one strike cannot end it.
    tokio::join!(plays_until(&mut ann, 5, 2), plays_until(&mut bob, 6, 2));
    running.shutdown().await;
    // The server starts again: the match is back with its ladder, the players re-take their
    // seats by account and play on to the end.
    let again = start(&config).await.unwrap();
    let info = listed(&again).await.into_iter().find(|g| g.game == game);
    let info = info.map(|g| (g.ladder, g.phase));
    assert_eq!(info, Some((Some(String::from("arena")), Phase::Running)));
    let (mut ann, mut bob) = (
        player(&again, "ann", 1).await,
        player(&again, "bob", 2).await,
    );
    takes(&mut ann, game, ann_seat).await;
    takes(&mut bob, game, bob_seat).await;
    let ((mut ann, ann_saw), (mut bob, bob_saw)) = tokio::join!(play(ann, 7), play(bob, 8));
    let result = end(&ann_saw);
    assert_eq!(result, end(&bob_saw));
    // Rated as Glicko-2 rates the game between new accounts (a game that did not verify is
    // unrated: `score` refuses it).
    let new = Rating::default();
    let (ann_after, bob_after) = rating::game(new, new, score(result, side(ann_seat)));
    assert_eq!(rating(&mut ann).await, (points(new), points(ann_after)));
    assert_eq!(rating(&mut bob).await, (points(new), points(bob_after)));
    let row = |running: &Running, login| running.standing("arena", 1, login).unwrap();
    let rows = [row(&again, "ann"), row(&again, "bob")];
    assert_eq!(
        rows.map(|r| (r.rating, r.games)),
        [(ann_after, 1), (bob_after, 1)]
    );
    // Ann queues again as soon as she heard the end (M4's review: the game left the ranked
    // games before its end was told).
    assert_eq!(ask(&mut ann, queue("arena")).await, queued());
    // Another restart, `next-game` lost: the same ratings, the game no longer listed, and the
    // next game's id still beyond the one the ladder's log names (M4's review).
    again.shutdown().await;
    std::fs::remove_file(config.server.data.join("next-game")).unwrap();
    let third = start(&config).await.unwrap();
    assert_eq!([row(&third, "ann"), row(&third, "bob")], rows);
    listed_as(&third, game, None).await;
    let (_, next, _) = created(&third, "pvp", Seat::Attacker).await;
    assert!(next > game, "{next} after {game}");
    third.shutdown().await;
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_leaderboard_pages_by_season_and_shows_ones_own_row() {
    // E16 M4 T4. Bob forfeits: Ann wins, and both are listed, Ann first.
    let config = graced(1_000, 60_000);
    let running = start(&config).await.unwrap();
    let ([mut ann, bob], _, _) = seated_match(&running).await;
    bob.close().await;
    ends(&mut ann).await;
    rating(&mut ann).await;
    let new = Rating::default();
    let (won, lost) = rating::game(new, new, Score::Win);
    let row = |rank, display: &str, rating, winner: bool| LeaderboardRow {
        rank,
        display: display.to_string(),
        rating: points(rating),
        games: 1,
        wins: u32::from(winner),
        losses: u32::from(!winner),
        draws: 0,
        bot: false,
    };
    let board = |seq, ladder: &str, season, page| ClientMessage::Leaderboard {
        seq,
        ladder: ladder.to_string(),
        season,
        page,
    };
    let page = |seq, season, total, rows| {
        Some(ServerMessage::LeaderboardPage {
            seq,
            ladder: String::from("arena"),
            season,
            total,
            rows,
        })
    };
    // Any connection reads it, logged in or not, a page at a time.
    let mut carol = greeted(&running).await;
    let rows = vec![row(1, "Ann", won, true), row(2, "Bob", lost, false)];
    assert_eq!(
        ask(&mut carol, board(4, "arena", None, 0)).await,
        page(4, 1, 2, rows.clone())
    );
    assert_eq!(
        ask(&mut carol, board(5, "arena", Some(1), 1)).await,
        page(5, 1, 2, vec![])
    );
    assert_eq!(
        ask(&mut carol, board(6, "arena", Some(2), 0)).await,
        refused(Refusal::NoSuchSeason)
    );
    assert_eq!(
        ask(&mut carol, board(7, "pit", None, 0)).await,
        refused(Refusal::NoSuchLadder)
    );
    // One's own row needs one's account; none before one's first rated game.
    let standing = |seq| ClientMessage::Standing {
        seq,
        ladder: "arena".to_string(),
    };
    assert_eq!(
        ask(&mut carol, standing(8)).await,
        refused(Refusal::NotLoggedIn)
    );
    let own = |seq, row| {
        Some(ServerMessage::OwnStanding {
            seq,
            ladder: String::from("arena"),
            season: 1,
            total: 2,
            row,
        })
    };
    let mut carol = player(&running, "carol", 3).await;
    assert_eq!(ask(&mut carol, standing(9)).await, own(9, None));
    let mut bob = player(&running, "bob", 2).await;
    assert_eq!(
        ask(&mut bob, standing(10)).await,
        own(10, Some(rows[1].clone()))
    );
    // A display name changed shows.
    let rename = ClientMessage::Rename {
        seq: 11,
        display: "Ann B".to_string(),
    };
    let renamed = ask(&mut ann, rename).await;
    assert!(
        matches!(renamed, Some(ServerMessage::Account { .. })),
        "{renamed:?}"
    );
    let ann_row = row(1, "Ann B", won, true);
    assert_eq!(
        ask(&mut ann, standing(12)).await,
        own(12, Some(ann_row.clone()))
    );
    running.shutdown().await;
    // A content change starts season 2 with an empty leaderboard; the first season's is kept.
    let mut changed = config.clone();
    changed.ladders[0].profile = Some(String::from("synthetic:strict-fixed"));
    changed.ladders[0].bots = vec![LadderBot {
        login: String::from("randy"),
        display: String::from("Randy"),
        kind: BotKind::Random,
    }];
    changed.ladders[0].bot_after_ms = 0;
    let again = start(&changed).await.unwrap();
    let mut dan = greeted(&again).await;
    assert_eq!(
        ask(&mut dan, board(13, "arena", None, 0)).await,
        page(13, 2, 0, vec![])
    );
    assert_eq!(
        ask(&mut dan, board(14, "arena", Some(1), 0)).await,
        page(14, 1, 2, vec![ann_row, rows[1].clone()])
    );
    // The new season's first game (M4's exit, E16 M4 T7): Ann's rating carried, its deviation
    // reset. She plays the ladder's bot, at once (Bob is too far below her to be paired soon).
    let mut ann = player(&again, "ann", 1).await;
    assert_eq!(ask(&mut ann, queue("arena")).await, queued());
    assert_eq!(ann.recv().await, found_against(true));
    let ready = ask(&mut ann, ClientMessage::AcceptMatch { seq: 3 }).await;
    let Some(ServerMessage::MatchReady { game, seat, .. }) = ready else {
        panic!("{ready:?}")
    };
    takes(&mut ann, game, seat).await;
    let (mut ann, saw) = tokio::time::timeout(GAME_TIME, play(ann, 5)).await.unwrap();
    let carried = won.new_season();
    assert_eq!(points(carried).deviation, 350);
    let score = score(end(&saw), side(seat));
    let (after, _) = rating::game(carried, Rating::default(), score);
    assert_eq!(rating(&mut ann).await, (points(carried), points(after)));
    again.shutdown().await;
    std::fs::remove_dir_all(&config.server.data).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_lone_player_plays_the_ladders_bot_after_the_wait_rated_and_flagged() {
    // E16 M4 T5: Randy, the ladder's bot, plays whoever waited 1.5 s without a match.
    let mut config = laddered(60_000);
    let randy = |display: &str| LadderBot {
        login: String::from("randy"),
        display: display.to_string(),
        kind: BotKind::Random,
    };
    config.ladders[0].bots = vec![randy("Randy")];
    config.ladders[0].bot_after_ms = 1_500;
    let running = start(&config).await.unwrap();
    let mut ann = player(&running, "ann", 1).await;
    let ladders = ask(&mut ann, ClientMessage::ListLadders { seq: 4 }).await;
    let Some(ServerMessage::Ladders { ladders, .. }) = ladders else {
        panic!("{ladders:?}")
    };
    assert_eq!(ladders[0].bots, ["Randy"]);
    // No one logs in as the bot, nor takes its name.
    let mut other = greeted(&running).await;
    let taken = other
        .login(1, "randy", &Key::from_seed([9; 32]), Some("Randy Two"))
        .await
        .unwrap();
    assert_eq!(taken, refused(Refusal::BadLogin));
    let mut other = greeted(&running).await;
    let taken = other
        .login(1, "rand", &Key::from_seed([9; 32]), Some("RANDY"))
        .await
        .unwrap();
    assert_eq!(taken, refused(Refusal::NameTaken));
    // Ann waits alone: after the wait she is matched with Randy, who accepted already, and is
    // told he is a bot (E16 M4 T5b).
    let waited = tokio::time::Instant::now();
    assert_eq!(ask(&mut ann, queue("arena")).await, queued());
    assert_eq!(ann.recv().await, found_against(true));
    assert!(
        waited.elapsed() >= Duration::from_millis(1_500),
        "{:?}",
        waited.elapsed()
    );
    let ready = ask(&mut ann, ClientMessage::AcceptMatch { seq: 3 }).await;
    let Some(ServerMessage::MatchReady { game, seat, .. }) = ready else {
        panic!("{ready:?}")
    };
    takes(&mut ann, game, seat).await;
    // The lobby names the bot's seat, flagged.
    let other_seat = match seat {
        Seat::Attacker => Seat::Defender,
        _ => Seat::Attacker,
    };
    let info = listed(&running).await.into_iter().find(|g| g.game == game);
    let seated = info.map(|g| g.seated);
    assert_eq!(
        seated
            .as_deref()
            .map(|seated| seated.iter().find(|s| s.bot).cloned()),
        Some(Some(Seated {
            seat: other_seat,
            display: String::from("Randy"),
            bot: true,
        }))
    );
    // The server's bot plays its seat to the end; the game is rated for both accounts.
    let (mut ann, saw) = tokio::time::timeout(GAME_TIME, play(ann, 5)).await.unwrap();
    let result = end(&saw);
    let new = Rating::default();
    let (ann_after, randy_after) = rating::game(new, new, score(result, side(seat)));
    assert_eq!(rating(&mut ann).await, (points(new), points(ann_after)));
    let row = |login| {
        running
            .standing("arena", 1, login)
            .map(|s| (s.rating, s.games))
    };
    assert_eq!(
        (row("ann"), row("randy")),
        (Some((ann_after, 1)), Some((randy_after, 1)))
    );
    // Its row on the leaderboard is flagged.
    let board = ClientMessage::Leaderboard {
        seq: 6,
        ladder: String::from("arena"),
        season: None,
        page: 0,
    };
    let page = ask(&mut ann, board.clone()).await;
    let Some(ServerMessage::LeaderboardPage { rows, .. }) = page else {
        panic!("{page:?}")
    };
    let flags: Vec<(&str, bool)> = rows.iter().map(|r| (r.display.as_str(), r.bot)).collect();
    assert!(
        flags.contains(&("Randy", true)) && flags.contains(&("Ann", false)),
        "{flags:?}"
    );
    // E16 M4 T5b: Bob waits for people only. Twice the wait on he has heard nothing, and still
    // waits: he leaves as he asks.
    let mut bob = player(&running, "bob", 2).await;
    let people = ClientMessage::Queue {
        seq: 2,
        ladder: String::from("arena"),
        bots: Some(false),
    };
    assert_eq!(ask(&mut bob, people).await, queued());
    let heard = tokio::time::timeout(Duration::from_millis(3_000), bob.recv()).await;
    assert!(heard.is_err(), "{heard:?}");
    let left = ask(&mut bob, ClientMessage::Dequeue { seq: 7 }).await;
    assert_eq!(left, Some(ServerMessage::Dequeued { seq: 7 }));
    running.shutdown().await;
    // The configuration renames the bot: its account is renamed at the next start, not added.
    let mut renamed = config.clone();
    renamed.ladders[0].bots = vec![randy("Randy Bot")];
    let again = start(&renamed).await.unwrap();
    let mut ann = player(&again, "ann", 1).await;
    let page = ask(&mut ann, board).await;
    let Some(ServerMessage::LeaderboardPage { rows, .. }) = page else {
        panic!("{page:?}")
    };
    assert!(
        rows.iter().any(|r| r.display == "Randy Bot" && r.bot),
        "{rows:?}"
    );
    again.shutdown().await;
    let log = std::fs::read_to_string(config.server.data.join("players.log")).unwrap();
    assert_eq!(
        (
            log.matches("\"Bot\"").count(),
            log.matches("\"Renamed\"").count()
        ),
        (1, 1),
        "{log}"
    );
    // A bot named as a player's account stops the start.
    let mut clash = config.clone();
    clash.ladders[0].bots = vec![LadderBot {
        login: String::from("ann"),
        ..randy("Annie")
    }];
    let error = start(&clash).await.unwrap_err();
    assert!(error.0.contains("bot `ann`: a player's account"), "{error}");
    std::fs::remove_dir_all(&config.server.data).unwrap();
}

/// A server with the ladder `arena` whose players may be away 1 s at once and propose `pauses`
/// pauses of at most `pause_ms` (E16 M4 T6).
fn pausing(pauses: u32, pause_ms: u64) -> Config {
    let mut config = graced(1_000, 60_000);
    config.ladders[0].pauses = pauses;
    config.ladders[0].pause_ms = pause_ms;
    config
}

/// The first message `client` hears that `wanted` takes, whatever comes before it.
async fn heard(client: &mut Client, wanted: impl Fn(&ServerMessage) -> bool) -> ServerMessage {
    let found = async {
        loop {
            match client.recv().await {
                Some(message) if wanted(&message) => return message,
                Some(_) => {}
                None => panic!("closed"),
            }
        }
    };
    tokio::time::timeout(GAME_TIME, found).await.unwrap()
}

/// `seat` proposes a pause of `ms`, and `other` agrees: both hear the answer, then the goodbye.
async fn agreed(proposer: &mut Client, other: &mut Client, ms: u64) {
    let propose = ClientMessage::ProposePause { seq: 20, ms };
    proposer.send(&propose).await.unwrap();
    heard(other, |m| matches!(m, ServerMessage::PauseProposed { .. })).await;
    let accept = ClientMessage::AnswerPause {
        seq: 21,
        accept: true,
    };
    other.send(&accept).await.unwrap();
    for client in [proposer, other] {
        let answered = |m: &ServerMessage| {
            matches!(
                m,
                ServerMessage::PauseAnswered { accept: true, .. } | ServerMessage::Goodbye { .. }
            )
        };
        assert!(matches!(
            heard(client, answered).await,
            ServerMessage::PauseAnswered { accept: true, .. }
        ));
        assert_eq!(
            heard(client, |m| matches!(m, ServerMessage::Goodbye { .. })).await,
            ServerMessage::Goodbye {
                reason: GoodbyeReason::Paused
            }
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_pause_declined_then_agreed_and_resumed_in_time() {
    let config = pausing(1, 5_000);
    let running = start(&config).await.unwrap();
    let ([mut ann, mut bob], game, [ann_seat, bob_seat]) = seated_match(&running).await;
    // Ann proposes, and asks no more until the game moves on; Bob declines.
    let propose = |seq, ms| ClientMessage::ProposePause { seq, ms };
    ann.send(&propose(10, 2_000)).await.unwrap();
    let proposed = heard(&mut bob, |m| {
        matches!(m, ServerMessage::PauseProposed { .. })
    })
    .await;
    assert!(
        matches!(proposed, ServerMessage::PauseProposed { ms: 2_000, seat, .. } if seat == ann_seat),
        "{proposed:?}"
    );
    ann.send(&propose(11, 2_000)).await.unwrap();
    let again = heard(&mut ann, |m| {
        matches!(m, ServerMessage::Rejected { seq: 11, .. })
    })
    .await;
    assert!(
        matches!(
            again,
            ServerMessage::Rejected {
                code: RejectCode::AlreadyAsked,
                ..
            }
        ),
        "{again:?}"
    );
    let decline = ClientMessage::AnswerPause {
        seq: 12,
        accept: false,
    };
    bob.send(&decline).await.unwrap();
    let declined = heard(&mut ann, |m| {
        matches!(m, ServerMessage::PauseAnswered { .. })
    })
    .await;
    assert!(
        matches!(declined, ServerMessage::PauseAnswered { accept: false, seat, .. } if seat == bob_seat),
        "{declined:?}"
    );
    // Bob proposes; Ann agrees: the game pauses, listed so.
    agreed(&mut bob, &mut ann, 3_000).await;
    listed_as(&running, game, Some(Phase::Paused)).await;
    // Both are back in time: the game plays on to its end, no forfeit.
    let mut ann = player(&running, "ann", 1).await;
    takes(&mut ann, game, ann_seat).await;
    let mut bob = player(&running, "bob", 2).await;
    takes(&mut bob, game, bob_seat).await;
    let ((_, ann_saw), (_, bob_saw)) = tokio::join!(play(ann, 5), play(bob, 6));
    for saw in [ann_saw, bob_saw] {
        let result = end(&saw);
        assert!(!matches!(result, Outcome::Forfeit { .. }), "{result:?}");
    }
    running.shutdown().await;
    std::fs::remove_dir_all(&config.server.data).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_player_not_back_after_an_agreed_pause_forfeits() {
    let config = pausing(1, 1_500);
    let running = start(&config).await.unwrap();
    let ([mut ann, mut bob], game, [ann_seat, bob_seat]) = seated_match(&running).await;
    agreed(&mut ann, &mut bob, 1_500).await;
    let paused = tokio::time::Instant::now();
    // Ann is back at once; Bob's absence counts only from the due time, then the grace.
    let mut ann = player(&running, "ann", 1).await;
    takes(&mut ann, game, ann_seat).await;
    let forfeit = Outcome::Forfeit {
        side: side(bob_seat),
    };
    assert_eq!(ends(&mut ann).await, forfeit);
    let waited = paused.elapsed();
    assert!(
        waited >= Duration::from_millis(2_300) && waited < Duration::from_secs(10),
        "the pause, then the grace: {waited:?}"
    );
    running.shutdown().await;
    std::fs::remove_dir_all(&config.server.data).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_paused_game_whose_players_are_not_back_wakes_and_is_void() {
    // Neither comes back: at the due time the game runs on without them, and the grace ends it, a
    // no-show each; the deadline of a paused game (here shorter) does not void it before.
    let mut config = pausing(1, 1_000);
    config.limits.paused_game_ms = 500;
    let running = start(&config).await.unwrap();
    let ([mut ann, mut bob], _, _) = seated_match(&running).await;
    agreed(&mut ann, &mut bob, 1_000).await;
    let no_show = |login| running.standing("arena", 1, login).map(|r| r.no_shows);
    let mut attempts = 0;
    while no_show("ann") != Some(1) {
        attempts += 1;
        assert!(attempts < 500, "never ended");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(no_show("bob"), Some(1));
    let void = config.server.data.join("void");
    assert!(!void.exists() || void.read_dir().unwrap().next().is_none());
    running.shutdown().await;
    std::fs::remove_dir_all(&config.server.data).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn finished_unrated_games_a_start_re_runs_are_rated_by_game_id() {
    // M4's review: games over but unrated (their ladder's log did not write) are rated at the next
    // start one after another by id, as Glicko-2 needs; ids 9 and 10, whose names sort the other
    // way. Each journal is taken as its players sat down and given a forfeit by hand.
    let config = graced(1_000, 60_000);
    std::fs::create_dir_all(&config.server.data).unwrap();
    std::fs::write(config.server.data.join("next-game"), "9\n").unwrap();
    let running = start(&config).await.unwrap();
    let journal = |game| std::fs::read_to_string(journal_file(&config.server.data, game)).unwrap();
    let forfeit = |seat| format!("{{\"Forfeit\":{{\"side\":\"{:?}\"}}}}\n", side(seat));
    let ([ann, bob], first, [_, bob_seat]) = seated_match(&running).await;
    let first_journal = journal(first) + &forfeit(bob_seat);
    // Both leave: void, no rating changes, and Ann may queue again.
    ann.close().await;
    bob.close().await;
    let mut attempts = 0;
    while running.standing("arena", 1, "ann").map(|s| s.no_shows) != Some(1) {
        attempts += 1;
        assert!(attempts < 500, "never ended");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let (mut ann, mut carl) = (
        player(&running, "ann", 1).await,
        player(&running, "carl", 3).await,
    );
    matched(&mut ann, &mut carl).await;
    let (second, [ann_seat, carl_seat]) = accepted(&mut ann, &mut carl).await;
    takes(&mut ann, second, ann_seat).await;
    takes(&mut carl, second, carl_seat).await;
    let second_journal = journal(second) + &forfeit(carl_seat);
    assert_eq!((first, second), (9, 10));
    running.shutdown().await;
    // A start on both journals and the player log.
    let data = config.server.data.with_extension("rerun");
    std::fs::create_dir_all(data.join("games")).unwrap();
    std::fs::write(journal_file(&data, first), first_journal).unwrap();
    std::fs::write(journal_file(&data, second), second_journal).unwrap();
    std::fs::copy(
        config.server.data.join("players.log"),
        data.join("players.log"),
    )
    .unwrap();
    let rerun = Config {
        server: Server {
            data: data.clone(),
            ..config.server.clone()
        },
        ..config.clone()
    };
    let again = start(&rerun).await.unwrap();
    let log = std::fs::read_to_string(data.join("ladders/arena.log")).unwrap();
    let at = |game: u32| {
        log.find(&format!("{{\"Rated\":{{\"game\":{game},"))
            .unwrap()
    };
    assert!(at(first) < at(second), "{log}");
    assert_eq!(again.standing("arena", 1, "ann").map(|s| s.wins), Some(2));
    again.shutdown().await;
    for data in [&config.server.data, &data] {
        std::fs::remove_dir_all(data).unwrap();
    }
}

/// The agreed pause's length in the tests of its hold across a restart (E16 M4 T8).
const HELD_MS: u64 = 8_000;

/// Ann and Bob agree on a pause of [`HELD_MS`], and Ann is back at once; the server stops before
/// its due time and starts again on the journals as they were (E16 M4 T8). The restarted server,
/// both data directories, the game, Bob's seat, when the pause began, and Ann in her seat again.
async fn held_through_a_restart() -> (
    Running,
    [PathBuf; 2],
    u32,
    Seat,
    tokio::time::Instant,
    Client,
) {
    let config = pausing(1, HELD_MS);
    let running = start(&config).await.unwrap();
    let ([mut ann, mut bob], game, [ann_seat, bob_seat]) = seated_match(&running).await;
    agreed(&mut ann, &mut bob, HELD_MS).await;
    let paused = tokio::time::Instant::now();
    let mut ann = player(&running, "ann", 1).await;
    takes(&mut ann, game, ann_seat).await;
    let restarted = Config {
        server: Server {
            data: crashed(&config.server.data),
            ..config.server.clone()
        },
        ..config.clone()
    };
    running.shutdown().await;
    let again = start(&restarted).await.unwrap();
    let mut ann = player(&again, "ann", 1).await;
    takes(&mut ann, game, ann_seat).await;
    let data = [config.server.data, restarted.server.data];
    (again, data, game, bob_seat, paused, ann)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn an_agreed_pause_resumed_early_holds_the_absences_across_a_restart() {
    // SRV-Q22 (b): Bob is back beyond the grace (1 s) from the restart, before the pause's due
    // time: the game plays on to its end, no forfeit.
    let (again, data, game, bob_seat, paused, ann) = held_through_a_restart().await;
    tokio::time::sleep(Duration::from_millis(2_500)).await;
    let mut bob = player(&again, "bob", 2).await;
    takes(&mut bob, game, bob_seat).await;
    assert!(
        paused.elapsed() < Duration::from_millis(HELD_MS),
        "before the due time: {:?}",
        paused.elapsed()
    );
    // A forfeit would have been rated before Bob's join reached the game.
    assert!(again.standing("arena", 1, "bob").is_none(), "no forfeit");
    let played = async { tokio::join!(play(ann, 5), play(bob, 6)) };
    let ((_, ann_saw), (_, bob_saw)) = tokio::time::timeout(GAME_TIME, played).await.unwrap();
    for saw in [ann_saw, bob_saw] {
        let result = end(&saw);
        assert!(!matches!(result, Outcome::Forfeit { .. }), "{result:?}");
    }
    again.shutdown().await;
    for data in data {
        std::fs::remove_dir_all(data).unwrap();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_player_not_back_by_a_held_pauses_due_time_forfeits_after_a_restart() {
    // Bob does not come back: his absence counts from the due time, then the grace forfeits him.
    let (again, data, _, bob_seat, paused, mut ann) = held_through_a_restart().await;
    let forfeit = Outcome::Forfeit {
        side: side(bob_seat),
    };
    assert_eq!(ends(&mut ann).await, forfeit);
    let waited = paused.elapsed();
    assert!(
        waited >= Duration::from_millis(HELD_MS + 800) && waited < Duration::from_secs(20),
        "the pause, then the grace: {waited:?}"
    );
    again.shutdown().await;
    for data in data {
        std::fs::remove_dir_all(data).unwrap();
    }
}
