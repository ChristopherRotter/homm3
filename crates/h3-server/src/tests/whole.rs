//! Whole games over the network (E16 M5 T9a): a map offer in the lobby, a whole game created from
//! it and joined by two clients, its commands answered once, its journal written ahead, the
//! refusals, an abandoned game. Their life (T9b): a restart restores a whole game, its seats
//! re-taken with their numbers kept and a seat's record downloaded; a paused game leaves its
//! thread and a seat's join resumes it, also after a restart; a game whose seats stayed empty
//! pauses at the sweep and goes void; a drain does not wait for a running whole game. The unowned
//! battle sides (T10): the server's computer plays a wandering monster's side, journaled as the
//! system actor's; a restart re-runs its commands, and answers for it where a crash cut them. The
//! queries (T11): a seat asks its hero's moves and moves along a path of the answer, another's
//! question about it is answered none, and no answer enters the record. The end-to-end game
//! (T13): two clients play a golden replay's opening, a battle against the server's computer and
//! the days after it through a restart in the battle, the journal re-running to what they see.

use std::fs;

use h3_contracts::PlayerId;
use h3_protocol::map::game::manifest;
use h3_protocol::session::game::{Allowance, Clock, GameEntry, GameOptions, GameSession};
use h3_protocol::wire::game::{
    AdventureCommand, Computer, GameAnswer, GameCommand, GameEvent, GameQuery, GameRecordHeader,
    MapGameInfo, MapOffer,
};
use h3_protocol::wire::{
    ClientMessage, Command, GoodbyeReason, Phase, Query, Refusal, Seat, ServerMessage,
};

use super::{GAME_TIME, fast, greeted, root};
use crate::client::Client;
use crate::config::{self, Config, Limits};
use crate::journal::whole as journal;
use crate::{Running, start, whole};

/// The duel map of the golden adventure replays (two players), pinned by its hash.
pub(super) fn duel_map() -> config::MapOffer {
    let path = "fixtures/maps/e08-duel-36.h3m";
    let (_, map) = h3_files::adventure::read_map(&root(), path).unwrap();
    config::MapOffer {
        id: String::from("duel-map"),
        file: root().join(path),
        hash: format!("{:#018x}", map.content_hash),
        packs: ["base", "ab", "sod"].map(String::from).to_vec(),
        profile: String::from("base:faithful"),
        difficulty: 1,
        name: None,
        computer: config::Computer::Fair,
        turn_minutes: None,
        level_up_ms: 60_000,
        battle_ms: 180_000,
        battle_increment_ms: 10_000,
        seeds: None,
        test_clock: None,
    }
}

/// The test servers' configuration with the duel map offered.
fn with_map() -> Config {
    Config {
        maps: vec![duel_map()],
        ..super::config(fast())
    }
}

/// The lobby's map offers and whole games.
pub(super) async fn map_listed(running: &Running) -> (Vec<MapOffer>, Vec<MapGameInfo>) {
    let mut client = greeted(running).await;
    client
        .send(&ClientMessage::ListGames { seq: 3 })
        .await
        .unwrap();
    let Some(ServerMessage::Games {
        maps, map_games, ..
    }) = client.recv().await
    else {
        panic!("no list")
    };
    client.close().await;
    (maps, map_games)
}

/// Waits until the lobby lists the whole games `until` accepts.
async fn listed_until(running: &Running, until: impl Fn(&[MapGameInfo]) -> bool) {
    for _ in 0..250 {
        if until(&map_listed(running).await.1) {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    panic!("never listed so: {:?}", map_listed(running).await.1);
}

/// The next message of `client` that `pick` takes, those before it skipped.
pub(super) async fn next<T>(client: &mut Client, pick: impl Fn(ServerMessage) -> Option<T>) -> T {
    loop {
        let message = client.recv().await.expect("the connection closed");
        if let Some(picked) = pick(message) {
            return picked;
        }
    }
}

/// A seat's `GameJoined` and `GameSnapshot`: the seat's token.
pub(super) async fn game_joined(client: &mut Client, game: u32, player: u8) -> String {
    let joined = client.recv().await;
    let Some(ServerMessage::GameJoined {
        game: g,
        player: p,
        token: Some(token),
        last: None,
        ..
    }) = joined
    else {
        panic!("{joined:?}")
    };
    assert_eq!((g, p), (game, player));
    assert!(matches!(
        client.recv().await,
        Some(ServerMessage::GameSnapshot { .. })
    ));
    token
}

#[test]
fn a_map_offer_loads_only_the_pinned_map() {
    let packs = root().join("packs");
    let offer = whole::MapOffer::load(&duel_map(), &packs).unwrap();
    let info = &offer.info;
    assert_eq!(
        (info.id.as_str(), info.size, info.players.len()),
        ("duel-map", 36, 2)
    );
    assert!(!info.name.is_empty(), "the map's own name");
    assert!(
        offer.setup.players.iter().all(|p| p.human),
        "every seat a player's"
    );
    let named = config::MapOffer {
        name: Some(String::from("Duel")),
        difficulty: 0,
        ..duel_map()
    };
    let named = whole::MapOffer::load(&named, &packs).unwrap();
    assert_eq!(
        (named.info.name.as_str(), named.info.difficulty),
        ("Duel", 0)
    );
    for (map, why) in [
        (
            config::MapOffer {
                hash: String::from("0x0000000000000001"),
                ..duel_map()
            },
            "content hash",
        ),
        (
            config::MapOffer {
                hash: String::from("5918efa3271a85e0"),
                ..duel_map()
            },
            "16 hex digits",
        ),
        (
            config::MapOffer {
                file: root().join("fixtures/maps/none.h3m"),
                ..duel_map()
            },
            "none.h3m",
        ),
        (
            config::MapOffer {
                packs: vec![String::from("nothing")],
                ..duel_map()
            },
            "duel-map",
        ),
        (
            config::MapOffer {
                difficulty: 9,
                ..duel_map()
            },
            "duel-map",
        ),
    ] {
        let error = whole::MapOffer::load(&map, &packs).unwrap_err();
        assert!(error.0.contains(why), "{why}: {error}");
    }
}

#[test]
fn a_map_offer_is_configured() {
    let base = root();
    let map = "[[map]]\nid = \"m\"\nfile = \"m.h3m\"\nhash = \"0x0\"\npacks = [\"base\"]\nprofile = \"p\"\n";
    let parsed = Config::parse(&format!("[content]\npacks = \"p\"\n{map}"), &base).unwrap();
    assert!(parsed.offers.is_empty(), "a map alone is an offer");
    let m = &parsed.maps[0];
    assert_eq!(
        (m.file.clone(), m.difficulty, m.name.clone(), m.computer),
        (base.join("m.h3m"), 1, None, config::Computer::Fair)
    );
    // E16 M5 T10: the all-seeing computer by name.
    let all = Config::parse(
        &format!("[content]\npacks = \"p\"\n{map}computer = \"all-seeing\"\n"),
        &base,
    );
    assert_eq!(all.unwrap().maps[0].computer, config::Computer::AllSeeing);
    let refused = |text: &str| Config::parse(&format!("[content]\npacks = \"p\"\n{text}"), &base);
    assert!(refused(&format!("{map}{map}")).is_err(), "a taken id");
    let offer = "[[offer]]\nid = \"m\"\nscenario = \"s.toml\"\n";
    assert!(refused(&format!("{offer}{map}")).is_err(), "an offer's id");
    assert!(
        refused(&map.replace("id = \"m\"", "id = \"\"")).is_err(),
        "an empty id"
    );
    assert!(
        refused(&format!("{map}computer = \"human\"\n")).is_err(),
        "no such computer"
    );
    // E16 M5 T17: the clock, untimed turns and the session's other defaults without; a turn one
    // of the original's durations, a battle's allowance not 0.
    assert_eq!(
        m.clock(),
        Clock {
            turn_ms: None,
            level_up_ms: Some(60_000),
            battle: Allowance::DEFAULT,
        }
    );
    let timed = "turn_minutes = 4\nlevel_up_ms = 0\nbattle_ms = 90000\nbattle_increment_ms = 0\n";
    let timed = Config::parse(&format!("[content]\npacks = \"p\"\n{map}{timed}"), &base);
    assert_eq!(
        timed.unwrap().maps[0].clock(),
        Clock {
            turn_ms: Some(240_000),
            level_up_ms: None,
            battle: Allowance {
                base_ms: 90_000,
                increment_ms: 0,
            },
        }
    );
    for minutes in config::TURN_MINUTES {
        let ok = Config::parse(
            &format!("[content]\npacks = \"p\"\n{map}turn_minutes = {minutes}\n"),
            &base,
        );
        assert!(ok.is_ok(), "{minutes} minutes");
    }
    for (text, why) in [
        ("turn_minutes = 3\n", "not the original's"),
        ("turn_minutes = 0\n", "unlimited is no number"),
        ("battle_ms = 0\n", "a side run out at once"),
        ("level_up_ms = -1\n", "no negative time"),
    ] {
        let error = refused(&format!("{map}{text}"));
        assert!(error.is_err(), "{why}");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_whole_game_is_created_joined_and_played_its_journal_ahead() {
    let config = with_map();
    let running = start(&config).await.unwrap();
    let (maps, games) = map_listed(&running).await;
    assert_eq!(maps.len(), 1);
    assert_eq!(
        (maps[0].id.as_str(), maps[0].players.len()),
        ("duel-map", 2)
    );
    assert!(games.is_empty());
    // The creator takes the first player's seat; the game waits for the second.
    let mut a = greeted(&running).await;
    a.create_map("duel-map", 0).await.unwrap();
    let created = a.recv().await;
    let Some(ServerMessage::GameCreated { seq: 0, game }) = created else {
        panic!("{created:?}")
    };
    let t0 = game_joined(&mut a, game, 0).await;
    let waiting = MapGameInfo {
        game,
        offer: String::from("duel-map"),
        phase: Phase::Waiting,
        open: vec![1],
        seated: vec![],
    };
    listed_until(&running, |games| games == [waiting.clone()]).await;
    // The second seat taken: the game starts, the first player acts.
    let mut b = greeted(&running).await;
    let join = |player, token| ClientMessage::GameJoin {
        game,
        player,
        token,
    };
    b.send(&join(1, None)).await.unwrap();
    let t1 = game_joined(&mut b, game, 1).await;
    let turn = |m| matches!(m, ServerMessage::GameTurn { .. }).then_some(());
    next(&mut a, turn).await;
    listed_until(&running, |games| {
        games
            .first()
            .is_some_and(|g| (g.phase, g.open.len()) == (Phase::Running, 0))
    })
    .await;
    // Its command is answered, the other player hears it and acts; a resend applies nothing.
    let end_turn = ClientMessage::GameCommand {
        seq: 1,
        command: GameCommand::EndTurn {},
    };
    let accepted = |m| match m {
        ServerMessage::GameAccepted { seq, .. } => Some(seq),
        _ => None,
    };
    a.send(&end_turn).await.unwrap();
    assert_eq!(next(&mut a, accepted).await, 1);
    next(&mut b, turn).await;
    a.send(&end_turn).await.unwrap();
    assert_eq!(next(&mut a, accepted).await, 1);
    // The journal holds every entry before its frames went: the seats, the start, the command
    // once; it re-runs in-process to the game the players saw.
    let path = journal::dir(&config.server.data).join(format!("{game}.journal"));
    let read = journal::read(&path).unwrap();
    assert_eq!(
        (read.start.offer.as_str(), read.start.game),
        ("duel-map", game)
    );
    assert_eq!(read.start.tokens, [t0.clone(), t1.clone()]);
    let p = PlayerId::new;
    let end = h3_game::GameCommand::EndTurn;
    assert_eq!(
        read.entries,
        [
            GameEntry::Taken(p(0)),
            GameEntry::Taken(p(1)),
            GameEntry::Started,
            GameEntry::Numbered(p(0), 1, end.clone()),
        ]
    );
    let replay = h3_files::adventure::parse_replay(&read.start.replay).unwrap();
    let offer = whole::MapOffer::load(&duel_map(), &config.content.packs).unwrap();
    assert_eq!(replay.map.content_hash, offer.map.content_hash);
    let options = GameOptions {
        game,
        checkpoint_every: config.server.checkpoint_every,
        record_bytes: config.limits.record_bytes,
        clock: offer.clock,
    };
    // The game's clock is in its journal's first line (E16 M5 T17).
    assert_eq!(Clock::from(read.start.clock), offer.clock);
    let tokens = read.start.tokens.clone();
    let mut reopened = GameSession::reopen(
        &offer.rules,
        &offer.setup,
        replay.seed,
        manifest(&offer.rules, offer.info.map),
        options,
        |player| tokens[player.index()].clone(),
        &read.entries,
    )
    .unwrap();
    assert!(reopened.started() && reopened.drain_entries().next().is_none());
    assert_eq!(reopened.journal().commands.len(), 1);
    // A restart restores it: listed running, its seats empty and none open.
    for client in [a, b] {
        client.close().await;
    }
    running.shutdown().await;
    let again = start(&config).await.unwrap();
    let restored = MapGameInfo {
        phase: Phase::Running,
        open: vec![],
        ..waiting
    };
    assert_eq!(map_listed(&again).await.1, [restored]);
    // Its seats re-taken with their tokens, the numbers kept: the first player's resend is answered
    // as the first time, the second player acts.
    let mut b = greeted(&again).await;
    b.send(&join(1, Some(t1.clone()))).await.unwrap();
    let back = |m| match m {
        ServerMessage::GameJoined { last, .. } => Some(last),
        _ => None,
    };
    assert_eq!(next(&mut b, back).await, None);
    next(&mut b, turn).await;
    let mut a = greeted(&again).await;
    a.send(&join(0, Some(t0.clone()))).await.unwrap();
    assert_eq!(next(&mut a, back).await, Some(1));
    a.send(&end_turn).await.unwrap();
    assert_eq!(next(&mut a, accepted).await, 1);
    b.send(&end_turn).await.unwrap();
    assert_eq!(next(&mut b, accepted).await, 1);
    next(&mut a, turn).await;
    // A seat downloads its record: its header names the game and the seat's player.
    a.send(&ClientMessage::FetchRecord { seq: 9 })
        .await
        .unwrap();
    let record = |m| match m {
        ServerMessage::Record { seq: 9, record, .. } => Some(record),
        _ => None,
    };
    let record = next(&mut a, record).await;
    let header: GameRecordHeader = serde_json::from_str(record.lines().next().unwrap()).unwrap();
    assert_eq!((header.game, header.player), (game, 0));
    assert!(record.lines().count() > 1);
    let read = journal::read(&path).unwrap();
    assert_eq!(
        read.entries.last(),
        Some(&GameEntry::Numbered(p(1), 1, end))
    );
    // The ids handed out after it are higher.
    let mut c = greeted(&again).await;
    c.create_map("duel-map", 1).await.unwrap();
    let created = c.recv().await;
    assert!(
        matches!(created, Some(ServerMessage::GameCreated { game: g, .. }) if g > game),
        "{created:?}"
    );
    again.shutdown().await;
}

/// A whole game of the duel map created and joined by two clients (the creator the first
/// player's seat, the host's), up to the first player's turn: both clients, the game and the
/// seats' tokens.
async fn started(running: &Running) -> (Client, Client, u32, [String; 2]) {
    started_on(running, "duel-map").await
}

/// [`started`] on the map offer `offer` of two players (E16 M5 T17).
pub(super) async fn started_on(
    running: &Running,
    offer: &str,
) -> (Client, Client, u32, [String; 2]) {
    let mut a = greeted(running).await;
    a.create_map(offer, 0).await.unwrap();
    let Some(ServerMessage::GameCreated { game, .. }) = a.recv().await else {
        panic!("not created")
    };
    let t0 = game_joined(&mut a, game, 0).await;
    let mut b = greeted(running).await;
    let join = ClientMessage::GameJoin {
        game,
        player: 1,
        token: None,
    };
    b.send(&join).await.unwrap();
    let t1 = game_joined(&mut b, game, 1).await;
    next(&mut a, |m| {
        matches!(m, ServerMessage::GameTurn { .. }).then_some(())
    })
    .await;
    (a, b, game, [t0, t1])
}

/// The answer to `client`'s query `query` numbered `seq`, the messages before it skipped.
async fn asked(client: &mut Client, seq: u32, query: GameQuery) -> GameAnswer {
    client
        .send(&ClientMessage::GameQuery { seq, query })
        .await
        .unwrap();
    next(client, |m| match m {
        ServerMessage::GameAnswer { seq: s, answer, .. } if s == seq => Some(answer),
        _ => None,
    })
    .await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_seat_asks_the_games_rules_and_moves_by_their_answer() {
    let config = with_map();
    let running = start(&config).await.unwrap();
    let (mut a, mut b, _, _) = started(&running).await;
    // The first player's hero: the one whose moves it is answered.
    let mut own = None;
    for hero in 0..16 {
        let answer = asked(&mut a, 100 + u32::from(hero), GameQuery::Moves { hero }).await;
        let GameAnswer::Moves { hero: h, moves } = answer else {
            panic!("{answer:?}")
        };
        assert_eq!(h, hero);
        own = own.or(moves.map(|moves| (hero, moves)));
    }
    let (hero, moves) = own.expect("the first player has a hero that may move");
    // The other player's question about it is none of its, as its battle's targets.
    let theirs = asked(&mut b, 1, GameQuery::Moves { hero }).await;
    assert_eq!(theirs, GameAnswer::Moves { hero, moves: None });
    let battle = GameQuery::Battle {
        query: Query::CreatureCastTargets {},
    };
    let targets = asked(&mut b, 2, battle).await;
    assert_eq!(targets, GameAnswer::Battle { targets: vec![] });
    // The first player moves its hero along the tree's path to a tile it lists.
    let (to, _) = moves.tiles()[0];
    let path = moves.path(to).unwrap();
    let command = GameCommand::Adventure {
        command: AdventureCommand::MoveHero { hero, path },
    };
    a.send(&ClientMessage::GameCommand { seq: 1, command })
        .await
        .unwrap();
    next(&mut a, |m| {
        matches!(m, ServerMessage::GameAccepted { seq: 1, .. }).then_some(())
    })
    .await;
    // Its record holds the command, no answer.
    a.send(&ClientMessage::FetchRecord { seq: 2 })
        .await
        .unwrap();
    let record = next(&mut a, |m| match m {
        ServerMessage::Record { record, .. } => Some(record),
        _ => None,
    })
    .await;
    assert!(record.contains("GameAccepted"));
    assert!(!record.contains("GameAnswer"));
    running.shutdown().await;
}

/// The goodbye a client gets, the messages before it skipped, then the end of the connection.
async fn goodbye_after(client: &mut Client) -> GoodbyeReason {
    let reason = next(client, |m| match m {
        ServerMessage::Goodbye { reason } => Some(reason),
        _ => None,
    })
    .await;
    assert_eq!(client.recv().await, None, "closed after the goodbye");
    reason
}

/// Waits until the lobby lists the whole game `game` in `phase`, or no more (`None`).
async fn map_listed_as(running: &Running, game: u32, phase: Option<Phase>) {
    listed_until(running, |games| {
        games.iter().find(|g| g.game == game).map(|g| g.phase) == phase
    })
    .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_paused_whole_game_leaves_its_thread_and_a_seat_resumes_it_also_after_a_restart() {
    let config = with_map();
    let running = start(&config).await.unwrap();
    let (mut a, mut b, game, [t0, t1]) = started(&running).await;
    // The host pauses it: both seats hear it, the lobby lists it paused, its journal ends so.
    a.send(&ClientMessage::Pause { seq: 1 }).await.unwrap();
    assert_eq!(goodbye_after(&mut a).await, GoodbyeReason::Paused);
    assert_eq!(goodbye_after(&mut b).await, GoodbyeReason::Paused);
    map_listed_as(&running, game, Some(Phase::Paused)).await;
    let path = journal::dir(&config.server.data).join(format!("{game}.journal"));
    let read = journal::read(&path).unwrap();
    assert_eq!(read.entries.last(), Some(&GameEntry::Paused));
    // A join without the seat's token, with another, of a player the map lacks, or a battle's
    // join, is refused as the running game refuses it.
    let mut c = greeted(&running).await;
    let join = |player, token| ClientMessage::GameJoin {
        game,
        player,
        token,
    };
    let battle_join = ClientMessage::Join {
        game,
        seat: Seat::Attacker,
        token: None,
    };
    for (message, reason) in [
        (join(1, None), Refusal::SeatTaken),
        (join(1, Some(String::from("x"))), Refusal::BadToken),
        (join(5, Some(t1.clone())), Refusal::NoSuchSeat),
        (battle_join, Refusal::NoSuchGame),
    ] {
        c.send(&message).await.unwrap();
        assert_eq!(c.recv().await, Some(ServerMessage::Refused { reason }));
    }
    // A seat's join with its token resumes it on a thread of its own.
    c.send(&join(1, Some(t1.clone()))).await.unwrap();
    let joined = |m| matches!(m, ServerMessage::GameJoined { player: 1, .. }).then_some(());
    next(&mut c, joined).await;
    map_listed_as(&running, game, Some(Phase::Running)).await;
    // Paused again by its host, it stays paused over a restart, and a join resumes it there.
    let mut a = greeted(&running).await;
    a.send(&join(0, Some(t0.clone()))).await.unwrap();
    next(&mut a, |m| {
        matches!(m, ServerMessage::GameJoined { player: 0, .. }).then_some(())
    })
    .await;
    a.send(&ClientMessage::Pause { seq: 2 }).await.unwrap();
    assert_eq!(goodbye_after(&mut a).await, GoodbyeReason::Paused);
    assert_eq!(goodbye_after(&mut c).await, GoodbyeReason::Paused);
    map_listed_as(&running, game, Some(Phase::Paused)).await;
    running.shutdown().await;
    let again = start(&config).await.unwrap();
    map_listed_as(&again, game, Some(Phase::Paused)).await;
    let mut a = greeted(&again).await;
    a.send(&join(0, Some(t0))).await.unwrap();
    let last = |m| match m {
        ServerMessage::GameJoined {
            player: 0, last, ..
        } => Some(last),
        _ => None,
    };
    assert_eq!(next(&mut a, last).await, None);
    map_listed_as(&again, game, Some(Phase::Running)).await;
    let read = journal::read(&path).unwrap();
    let pauses = |entry: &GameEntry| matches!(entry, GameEntry::Paused | GameEntry::Resumed);
    let pauses: Vec<_> = read.entries.into_iter().filter(pauses).collect();
    assert_eq!(
        pauses,
        [
            GameEntry::Paused,
            GameEntry::Resumed,
            GameEntry::Paused,
            GameEntry::Resumed,
        ]
    );
    again.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_whole_game_whose_seats_stayed_empty_pauses_then_goes_void() {
    let limits = Limits {
        empty_game_ms: 200,
        paused_game_ms: 400,
        ..fast()
    };
    let config = Config {
        maps: vec![duel_map()],
        ..super::config(limits)
    };
    let running = start(&config).await.unwrap();
    let (a, b, game, _) = started(&running).await;
    // Its players gone, the running game pauses at its deadline, its journal so.
    for client in [a, b] {
        client.close().await;
    }
    map_listed_as(&running, game, Some(Phase::Paused)).await;
    let path = journal::dir(&config.server.data).join(format!("{game}.journal"));
    let read = journal::read(&path).unwrap();
    assert_eq!(read.entries.last(), Some(&GameEntry::Paused));
    // Nobody resumes it: it goes void, its journal moved aside.
    map_listed_as(&running, game, None).await;
    assert!(!path.exists());
    let void: Vec<_> = fs::read_dir(config.server.data.join("void"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        void.iter()
            .any(|name| name.starts_with(&format!("{game}-"))),
        "{void:?}"
    );
    running.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_draining_server_does_not_wait_for_a_running_whole_game() {
    let config = with_map();
    let running = start(&config).await.unwrap();
    let (mut a, mut b, game, _) = started(&running).await;
    let mut c = greeted(&running).await;
    c.create_map("duel-map", 0).await.unwrap();
    let Some(ServerMessage::GameCreated { game: waiting, .. }) = c.recv().await else {
        panic!("not created")
    };
    game_joined(&mut c, waiting, 0).await;
    // The drain ends the waiting game and does not wait for the running one, which plays on.
    running.drain();
    assert_eq!(goodbye_after(&mut c).await, GoodbyeReason::Shutdown);
    tokio::time::timeout(GAME_TIME, running.drained())
        .await
        .unwrap();
    assert_eq!(running.in_play(), 1);
    let end_turn = ClientMessage::GameCommand {
        seq: 1,
        command: GameCommand::EndTurn {},
    };
    a.send(&end_turn).await.unwrap();
    let accepted = |m| matches!(m, ServerMessage::GameAccepted { seq: 1, .. }).then_some(());
    next(&mut a, accepted).await;
    // The stop ends it, its journal kept for the next start.
    running.shutdown().await;
    assert_eq!(goodbye_after(&mut b).await, GoodbyeReason::Shutdown);
    let path = journal::dir(&config.server.data).join(format!("{game}.journal"));
    assert!(path.is_file());
    let waiting_path = journal::dir(&config.server.data).join(format!("{waiting}.journal"));
    assert!(!waiting_path.exists());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn whole_games_refuse_out_of_place_and_go_when_abandoned() {
    let config = with_map();
    let running = start(&config).await.unwrap();
    let mut c = greeted(&running).await;
    let refusal = async |client: &mut Client, message: ClientMessage| {
        client.send(&message).await.unwrap();
        client.recv().await
    };
    let create = |offer: &str, player| ClientMessage::CreateMapGame {
        seq: 1,
        offer: offer.to_owned(),
        player,
    };
    let join = |game, player| ClientMessage::GameJoin {
        game,
        player,
        token: None,
    };
    for (message, reason) in [
        (create("nothing", 0), Refusal::NoSuchOffer),
        (create("pvp", 0), Refusal::NoSuchOffer),
        (create("duel-map", 2), Refusal::NoSuchSeat),
        (join(99, 0), Refusal::NoSuchGame),
    ] {
        assert_eq!(
            refusal(&mut c, message).await,
            Some(ServerMessage::Refused { reason })
        );
    }
    // A created game: its taken seat, a battle's join and a player it does not have refused.
    c.create_map("duel-map", 0).await.unwrap();
    let Some(ServerMessage::GameCreated { game, .. }) = c.recv().await else {
        panic!("not created")
    };
    game_joined(&mut c, game, 0).await;
    let mut d = greeted(&running).await;
    let battle_join = ClientMessage::Join {
        game,
        seat: Seat::Attacker,
        token: None,
    };
    for (message, reason) in [
        (join(game, 0), Refusal::SeatTaken),
        (battle_join, Refusal::NoSuchGame),
        (join(game, 5), Refusal::NoSuchSeat),
    ] {
        assert_eq!(
            refusal(&mut d, message).await,
            Some(ServerMessage::Refused { reason })
        );
    }
    // Its creator leaves before the game starts: it is abandoned, its journal gone.
    let path = journal::dir(&config.server.data).join(format!("{game}.journal"));
    assert!(path.is_file());
    c.send(&ClientMessage::Leave {}).await.unwrap();
    assert_eq!(
        c.recv().await,
        Some(ServerMessage::Goodbye {
            reason: GoodbyeReason::Left
        })
    );
    listed_until(&running, <[MapGameInfo]>::is_empty).await;
    assert!(!path.exists());
    running.shutdown().await;
}

/// The names of the journals in `data`'s void.
fn voided(data: &std::path::Path) -> Vec<String> {
    let names = fs::read_dir(data.join("void")).into_iter().flatten();
    let names = names.map(|e| e.unwrap().file_name().to_string_lossy().into_owned());
    names.collect()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn whole_journals_that_do_not_restore_are_void() {
    let config = with_map();
    let running = start(&config).await.unwrap();
    let (_, _, kept, _) = started(&running).await;
    let (_, _, garbled, _) = started(&running).await;
    running.shutdown().await;
    // A journal with a line that does not read is void; the other is restored.
    let path = |game: u32| journal::dir(&config.server.data).join(format!("{game}.journal"));
    let mut text = fs::read_to_string(path(garbled)).unwrap();
    text.push_str("{\"Nothing\":{}}\n");
    fs::write(path(garbled), text).unwrap();
    let again = start(&config).await.unwrap();
    let listed: Vec<u32> = map_listed(&again).await.1.iter().map(|g| g.game).collect();
    assert_eq!(listed, [kept]);
    assert!(!path(garbled).exists());
    let void = voided(&config.server.data);
    assert!(
        void.iter()
            .any(|name| name.starts_with(&format!("{garbled}-"))),
        "{void:?}"
    );
    again.shutdown().await;
    // A server that no longer offers its map voids it.
    let without = Config {
        server: config.server.clone(),
        ..super::config(fast())
    };
    let last = start(&without).await.unwrap();
    assert!(map_listed(&last).await.1.is_empty());
    assert!(!path(kept).exists());
    let void = voided(&config.server.data);
    assert!(
        void.iter()
            .any(|name| name.starts_with(&format!("{kept}-"))),
        "{void:?}"
    );
    last.shutdown().await;
}

/// The golden replay whose 17th command opens a battle against a wandering monster on the duel
/// map (its 18th the player's, then the system actor's).
pub(super) const MONSTER_REPLAY: &str = "fixtures/replays/adventure/duel-s3-easy-d7.toml";

/// Game `game`'s journal rewritten in `data` with its first line `start` and `entries`.
fn rewrite(data: &std::path::Path, start: &journal::GameStart, entries: &[GameEntry]) {
    let path = journal::dir(data).join(format!("{}.journal", start.game));
    fs::remove_file(&path).unwrap();
    let mut writer = journal::create(data, start).unwrap();
    writer.append_game(entries.iter().cloned()).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_server_plays_the_unowned_sides_and_a_restart_answers_for_them() {
    let config = Config {
        maps: vec![config::MapOffer {
            computer: config::Computer::AllSeeing,
            ..duel_map()
        }],
        ..super::config(fast())
    };
    let data = &config.server.data;
    let running = start(&config).await.unwrap();
    // The offer names its computer.
    let (maps, _) = map_listed(&running).await;
    assert_eq!(maps[0].computer, Computer::AllSeeing);
    let (a, b, game, [t0, t1]) = started(&running).await;
    for client in [a, b] {
        client.close().await;
    }
    running.shutdown().await;
    // Its journal re-seeded as the golden replay's game before its first command (nothing re-runs
    // yet but the seats), so the replay's commands play on it.
    let golden = fs::read_to_string(root().join(MONSTER_REPLAY)).unwrap();
    let golden = h3_files::adventure::parse_replay(&golden).unwrap();
    let path = journal::dir(data).join(format!("{game}.journal"));
    let read = journal::read(&path).unwrap();
    let mut replay = h3_files::adventure::parse_replay(&read.start.replay).unwrap();
    (replay.seed, replay.difficulty) = (golden.seed, golden.difficulty);
    let start_line = journal::GameStart {
        replay: h3_files::adventure::replay_to_toml(&replay).unwrap(),
        ..read.start.clone()
    };
    rewrite(data, &start_line, &read.entries);
    let running = start(&config).await.unwrap();
    let rejoin = |player, token: &String| ClientMessage::GameJoin {
        game,
        player,
        token: Some(token.clone()),
    };
    let mut clients = [greeted(&running).await, greeted(&running).await];
    clients[0].send(&rejoin(0, &t0)).await.unwrap();
    clients[1].send(&rejoin(1, &t1)).await.unwrap();
    let accepted = |seq| {
        move |m| matches!(m, ServerMessage::GameAccepted { seq: s, .. } if s == seq).then_some(())
    };
    // The replay's commands up to the one opening the battle, each from its seat's client.
    let opens = 16;
    for (seq, (actor, command)) in golden.commands[..=opens].iter().enumerate() {
        let h3_game::Actor::Player(player) = actor else {
            panic!("command {seq} is the system actor's");
        };
        let seq = u32::try_from(seq).unwrap();
        let message = ClientMessage::GameCommand {
            seq,
            command: h3_protocol::map::game::command(command),
        };
        let client = &mut clients[player.index()];
        client.send(&message).await.unwrap();
        next(client, accepted(seq)).await;
    }
    let [a, mut b] = clients;
    a.close().await;
    // The fighter plays its side, the server's computer the monster's, until the battle ends.
    let mut rng = h3_kernel::Rng::from_seed(5);
    let mut seq = u32::try_from(opens).unwrap();
    let ended = loop {
        match b.recv().await.expect("the connection closed") {
            ServerMessage::GameTurn { legal, .. } => {
                let battle: Vec<Command> = (legal.iter())
                    .filter_map(|c| match c {
                        GameCommand::Battle { command } => Some(*command),
                        _ => None,
                    })
                    .collect();
                let Some(command) = crate::bench::choose(&battle, &mut rng) else {
                    continue;
                };
                seq += 1;
                let command = GameCommand::Battle { command };
                b.send(&ClientMessage::GameCommand { seq, command })
                    .await
                    .unwrap();
            }
            ServerMessage::GameEvents { events, .. }
                if events
                    .iter()
                    .any(|e| matches!(e, GameEvent::BattleEnded { .. })) =>
            {
                break seq;
            }
            _ => {}
        }
    };
    assert!(ended > u32::try_from(opens).unwrap());
    b.close().await;
    running.shutdown().await;
    // The computer's commands are the system actor's in the journal, beside the fighter's.
    let read = journal::read(&path).unwrap();
    let theirs = |e: &GameEntry| matches!(e, GameEntry::Command(h3_game::Actor::System, _));
    assert!(read.entries.iter().any(theirs), "{:?}", read.entries);
    // A crash cut the journal after the fighter's last command the computer answered: the
    // restart re-runs the computer's earlier commands and answers for it before any join.
    let answered = (read.entries.iter().enumerate())
        .filter(|(i, e)| {
            matches!(e, GameEntry::Numbered(..))
                && (read.entries[i + 1..].iter())
                    .find(|e| !matches!(e, GameEntry::Checkpoint(_)))
                    .is_some_and(theirs)
        })
        .map(|(i, _)| i)
        .next_back()
        .unwrap();
    let cut = &read.entries[..=answered];
    assert!(cut.iter().any(theirs));
    rewrite(data, &read.start, cut);
    // The offer's fair computer now: the computer is the offer's, not the journal's.
    let fair = Config {
        server: config.server.clone(),
        ..with_map()
    };
    let running = start(&fair).await.unwrap();
    let mut b = greeted(&running).await;
    b.send(&rejoin(1, &t1)).await.unwrap();
    next(&mut b, |m| {
        matches!(m, ServerMessage::GameJoined { .. }).then_some(())
    })
    .await;
    let again = journal::read(&path).unwrap().entries;
    assert!(again.starts_with(cut));
    let after = again[cut.len()..]
        .iter()
        .find(|e| !matches!(e, GameEntry::Checkpoint(_)));
    assert!(after.is_some_and(theirs), "{:?}", &again[cut.len()..]);
    b.close().await;
    running.shutdown().await;
}

/// The battle commands among a turn's legal commands.
fn battle_commands(legal: &[GameCommand]) -> Vec<Command> {
    (legal.iter())
        .filter_map(|c| match c {
            GameCommand::Battle { command } => Some(*command),
            _ => None,
        })
        .collect()
}

/// Reads `client`'s messages until its next battle turn: the `GameEvents` it got on the way and
/// that turn's battle commands, none once the battle ended.
async fn battle_turn(client: &mut Client) -> (usize, Vec<Command>) {
    let mut events = 0;
    loop {
        match client.recv().await.expect("the connection closed") {
            ServerMessage::GameTurn { legal, .. } if !battle_commands(&legal).is_empty() => {
                return (events, battle_commands(&legal));
            }
            ServerMessage::GameEvents { events: e, .. } => {
                events += 1;
                if (e.iter()).any(|e| matches!(e, GameEvent::BattleEnded { .. })) {
                    return (events, Vec::new());
                }
            }
            _ => {}
        }
    }
}

/// The fighter's side of a battle, from its turn's battle commands `legal` on: at most `most`
/// commands chosen by `rng` and numbered after `seq`, each followed by the messages up to its next
/// battle turn. The commands it sent, the `GameEvents` it got, and its battle turn's commands at
/// the end, none once the battle ended.
async fn fight(
    client: &mut Client,
    mut legal: Vec<Command>,
    (seq, rng): (&mut u32, &mut h3_kernel::Rng),
    most: usize,
) -> (Vec<GameCommand>, usize, Vec<Command>) {
    let (mut sent, mut events) = (Vec::new(), 0);
    while !legal.is_empty() && sent.len() < most {
        let command = GameCommand::Battle {
            command: crate::bench::choose(&legal, rng).unwrap(),
        };
        *seq += 1;
        let message = ClientMessage::GameCommand {
            seq: *seq,
            command: command.clone(),
        };
        client.send(&message).await.unwrap();
        sent.push(command);
        let (got, next) = battle_turn(client).await;
        (events, legal) = (events + got, next);
    }
    (sent, events, legal)
}

/// What a client reads until its turn: every message up to and with its `GameTurn`, or up to the
/// end of its connection.
async fn read_until_turn(mut client: Client) -> (Client, Vec<ServerMessage>) {
    let mut seen = Vec::new();
    while let Some(message) = client.recv().await {
        let turn = matches!(message, ServerMessage::GameTurn { .. });
        seen.push(message);
        if turn {
            break;
        }
    }
    (client, seen)
}

/// A re-taken seat's `GameJoined` and `GameSnapshot`: its last command's number and its view.
pub(super) async fn rejoined(
    client: &mut Client,
    game: u32,
    (player, token): (u8, &str),
) -> (Option<u32>, h3_protocol::wire::game::GameObservation) {
    let join = ClientMessage::GameJoin {
        game,
        player,
        token: Some(String::from(token)),
    };
    client.send(&join).await.unwrap();
    let joined = client.recv().await;
    let Some(ServerMessage::GameJoined {
        player: p,
        token: Some(_),
        last,
        ..
    }) = joined
    else {
        panic!("{joined:?}")
    };
    assert_eq!(p, player);
    let snapshot = client.recv().await;
    let Some(ServerMessage::GameSnapshot { observation, .. }) = snapshot else {
        panic!("{snapshot:?}")
    };
    (last, observation)
}

/// How many `GameEvents` `messages` hold.
fn events_in(messages: &[ServerMessage]) -> usize {
    let events = |m: &&ServerMessage| matches!(m, ServerMessage::GameEvents { .. });
    messages.iter().filter(events).count()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn two_clients_play_a_whole_game_through_a_restart_and_a_battle_against_the_computer() {
    // The duel map offered with the golden replay's seed and difficulty pinned (the tests' hook in
    // place of the OS), so the replay's opening plays on its game, and its computer's seed pinned,
    // so the battle repeats.
    let golden = fs::read_to_string(root().join(MONSTER_REPLAY)).unwrap();
    let golden = h3_files::adventure::parse_replay(&golden).unwrap();
    let offer = config::MapOffer {
        difficulty: golden.difficulty,
        seeds: Some((golden.seed, 7)),
        ..duel_map()
    };
    let config = Config {
        maps: vec![offer.clone()],
        ..super::config(fast())
    };
    let running = start(&config).await.unwrap();
    // Two clients take the map's two seats from the lobby; the game starts.
    let (mut a, mut b, game, [t0, t1]) = started(&running).await;
    let accepted = |seq| {
        move |m| matches!(m, ServerMessage::GameAccepted { seq: s, .. } if s == seq).then_some(())
    };
    // The replay's opening, each command from its seat's client: three days of both players'
    // moves and turns' ends, a move's end found among its hero's moves before it is sent; the
    // last opens the second player's battle against a wandering monster.
    let opens = 16;
    let mut lasts = [None; 2];
    for (seq, (actor, command)) in golden.commands[..=opens].iter().enumerate() {
        let h3_game::Actor::Player(player) = actor else {
            panic!("command {seq} is the system actor's");
        };
        let seq = u32::try_from(seq).unwrap();
        let client = if player.index() == 0 { &mut a } else { &mut b };
        let command = h3_protocol::map::game::command(command);
        if let GameCommand::Adventure {
            command: AdventureCommand::MoveHero { hero, path },
        } = &command
        {
            let answer = asked(client, 1000 + seq, GameQuery::Moves { hero: *hero }).await;
            let GameAnswer::Moves {
                moves: Some(moves), ..
            } = answer
            else {
                panic!("{answer:?}")
            };
            let end = *path.last().unwrap();
            assert!(moves.points_left(end).is_some(), "move {seq}'s end");
        }
        client
            .send(&ClientMessage::GameCommand { seq, command })
            .await
            .unwrap();
        next(client, accepted(seq)).await;
        lasts[player.index()] = Some(seq);
    }
    // The first player's client reads on meanwhile (its outbound queue is bounded): until the
    // restart closes its connection.
    let reading = tokio::spawn(read_until_turn(a));
    // The fighter plays its side, the server's computer the monster's: two of the fighter's
    // commands, then the server stops with the fighter to act.
    let mut rng = h3_kernel::Rng::from_seed(5);
    let mut seq = u32::try_from(opens).unwrap();
    let (_, legal) = battle_turn(&mut b).await;
    let (sent, _, legal) = fight(&mut b, legal, (&mut seq, &mut rng), 2).await;
    assert_eq!(sent.len(), 2);
    assert!(!legal.is_empty(), "the battle ended before the restart");
    running.shutdown().await;
    assert_eq!(goodbye_after(&mut b).await, GoodbyeReason::Shutdown);
    let (_, seen) = reading.await.unwrap();
    assert_eq!(
        seen.last(),
        Some(&ServerMessage::Goodbye {
            reason: GoodbyeReason::Shutdown
        })
    );
    // The restart restores the game in its battle; the clients re-take their seats with their
    // numbers, the fighter's view holding its battle and its turn's commands as before.
    let running = start(&config).await.unwrap();
    map_listed_as(&running, game, Some(Phase::Running)).await;
    let mut a = greeted(&running).await;
    let (last, view) = rejoined(&mut a, game, (0, &t0)).await;
    assert_eq!((last, view.battle.is_none()), (lasts[0], true));
    let mut b = greeted(&running).await;
    let (last, view) = rejoined(&mut b, game, (1, &t1)).await;
    assert_eq!((last, view.battle.is_some()), (Some(seq), true));
    let (got, again) = battle_turn(&mut b).await;
    assert_eq!((got, &again), (0, &legal));
    // The fighter's last command resent: answered as the first time, nothing applied, nobody told.
    let resent = sent.last().unwrap().clone();
    let message = ClientMessage::GameCommand {
        seq,
        command: resent.clone(),
    };
    b.send(&message).await.unwrap();
    let echo = b.recv().await;
    assert!(
        matches!(&echo, Some(ServerMessage::GameAccepted { seq: s, command, .. }) if *s == seq && *command == resent),
        "{echo:?}"
    );
    // The battle to its end, the first player's client reading on until its turn.
    let reading = tokio::spawn(read_until_turn(a));
    let (sent, mut events, legal) = fight(&mut b, legal, (&mut seq, &mut rng), usize::MAX).await;
    assert!(!sent.is_empty() && legal.is_empty());
    // The fighter's day goes on: it ends its turn, the first player's next day begins.
    let end_turn = GameCommand::EndTurn {};
    loop {
        match b.recv().await.expect("the connection closed") {
            ServerMessage::GameEvents { .. } => events += 1,
            ServerMessage::GameTurn { legal, .. } if legal.contains(&end_turn) => break,
            _ => {}
        }
    }
    seq += 1;
    let message = ClientMessage::GameCommand {
        seq,
        command: end_turn.clone(),
    };
    b.send(&message).await.unwrap();
    next(&mut b, accepted(seq)).await;
    next(&mut b, |m| {
        matches!(m, ServerMessage::GameEvents { .. }).then_some(())
    })
    .await;
    events += 1;
    // Each player got one `GameEvents` per command since the restart, the computer's too, and
    // none for the resend.
    let (mut a, seen) = reading.await.unwrap();
    assert!(matches!(seen.last(), Some(ServerMessage::GameTurn { .. })));
    assert_eq!(events_in(&seen), events);
    // The first player asks its hero's moves on the new day and ends its turn; the fighter acts.
    let first = h3_game::Actor::Player(PlayerId::new(0));
    let hero = (golden.commands[..=opens].iter())
        .filter(|(actor, _)| *actor == first)
        .find_map(|(_, c)| match h3_protocol::map::game::command(c) {
            GameCommand::Adventure {
                command: AdventureCommand::MoveHero { hero, .. },
            } => Some(hero),
            _ => None,
        });
    let hero = hero.unwrap();
    let answer = asked(&mut a, 2000, GameQuery::Moves { hero }).await;
    assert!(
        matches!(answer, GameAnswer::Moves { moves: Some(_), .. }),
        "{answer:?}"
    );
    let a_seq = lasts[0].unwrap() + 1;
    let message = ClientMessage::GameCommand {
        seq: a_seq,
        command: end_turn,
    };
    a.send(&message).await.unwrap();
    next(&mut a, accepted(a_seq)).await;
    next(&mut b, |m| {
        matches!(m, ServerMessage::GameTurn { .. }).then_some(())
    })
    .await;
    // The journal: each seat's commands numbered strictly increasing, the resend not among them,
    // the computer's as the system actor's; it re-runs in-process to the game the clients see.
    let path = journal::dir(&config.server.data).join(format!("{game}.journal"));
    let read = journal::read(&path).unwrap();
    for player in 0..2 {
        let numbers: Vec<u32> = (read.entries.iter())
            .filter_map(|e| match e {
                GameEntry::Numbered(p, n, _) if p.index() == player => Some(*n),
                _ => None,
            })
            .collect();
        assert!(numbers.windows(2).all(|w| w[0] < w[1]), "{numbers:?}");
        let last = if player == 0 { a_seq } else { seq };
        assert_eq!(numbers.last(), Some(&last));
    }
    let theirs = |e: &&GameEntry| matches!(e, GameEntry::Command(h3_game::Actor::System, _));
    assert!(read.entries.iter().filter(theirs).count() > 0);
    let loaded = whole::MapOffer::load(&offer, &config.content.packs).unwrap();
    let replay = h3_files::adventure::parse_replay(&read.start.replay).unwrap();
    assert_eq!(replay.seed, golden.seed);
    let options = GameOptions {
        game,
        checkpoint_every: config.server.checkpoint_every,
        record_bytes: config.limits.record_bytes,
        clock: loaded.clock,
    };
    let tokens = read.start.tokens.clone();
    let reopened = GameSession::reopen(
        &loaded.rules,
        &loaded.setup,
        replay.seed,
        manifest(&loaded.rules, loaded.info.map),
        options,
        |player| tokens[player.index()].clone(),
        &read.entries,
    )
    .unwrap();
    // Each client, back with a fresh connection, is sent the view the re-run game gives its
    // player; its record holds no answer and none of the journal's state hashes.
    let hashes: Vec<u64> = (read.entries.iter())
        .filter_map(|e| match e {
            GameEntry::Checkpoint(hash) => Some(*hash),
            _ => None,
        })
        .collect();
    assert!(!hashes.is_empty());
    for (client, (player, token)) in [(a, (0, &t0)), (b, (1, &t1))] {
        client.close().await;
        let mut client = greeted(&running).await;
        let (_, view) = rejoined(&mut client, game, (player, token)).await;
        let viewer = h3_game::Viewer::Player(PlayerId::new(player));
        let expected = h3_protocol::map::game::observe(reopened.state(), &loaded.rules, viewer);
        assert_eq!(view, expected, "player {player}'s view");
        client
            .send(&ClientMessage::FetchRecord { seq: 9 })
            .await
            .unwrap();
        let record = next(&mut client, |m| match m {
            ServerMessage::Record { record, .. } => Some(record),
            _ => None,
        })
        .await;
        let header: GameRecordHeader =
            serde_json::from_str(record.lines().next().unwrap()).unwrap();
        assert_eq!((header.game, header.player), (game, player));
        assert!(!record.contains("GameAnswer"));
        for hash in &hashes {
            for form in [
                format!("{hash}"),
                format!("{hash:x}"),
                format!("{hash:016x}"),
            ] {
                assert!(
                    !record.contains(&form),
                    "player {player}'s record: a hash {form}"
                );
            }
        }
        client.close().await;
    }
    running.shutdown().await;
}
