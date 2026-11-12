//! Whole games over the network (E16 M5 T9a): a map offer in the lobby, a whole game created from
//! it and joined by two clients, its commands answered once, its journal written ahead, the
//! refusals, an abandoned game. Their life (T9b): a restart restores a whole game, its seats
//! re-taken with their numbers kept and a seat's record downloaded; a paused game leaves its
//! thread and a seat's join resumes it, also after a restart; a game whose seats stayed empty
//! pauses at the sweep and goes void; a drain does not wait for a running whole game.

use std::fs;

use h3_contracts::PlayerId;
use h3_protocol::map::game::manifest;
use h3_protocol::session::game::{GameEntry, GameOptions, GameSession};
use h3_protocol::wire::game::{GameCommand, GameRecordHeader, MapGameInfo, MapOffer};
use h3_protocol::wire::{ClientMessage, GoodbyeReason, Phase, Refusal, Seat, ServerMessage};

use super::{GAME_TIME, fast, greeted, root};
use crate::client::Client;
use crate::config::{self, Config, Limits};
use crate::journal::whole as journal;
use crate::{Running, start, whole};

/// The duel map of the golden adventure replays (two players), pinned by its hash.
fn duel_map() -> config::MapOffer {
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
async fn map_listed(running: &Running) -> (Vec<MapOffer>, Vec<MapGameInfo>) {
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
async fn next<T>(client: &mut Client, pick: impl Fn(ServerMessage) -> Option<T>) -> T {
    loop {
        let message = client.recv().await.expect("the connection closed");
        if let Some(picked) = pick(message) {
            return picked;
        }
    }
}

/// A seat's `GameJoined` and `GameSnapshot`: the seat's token.
async fn game_joined(client: &mut Client, game: u32, player: u8) -> String {
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
        (m.file.clone(), m.difficulty, m.name.clone()),
        (base.join("m.h3m"), 1, None)
    );
    let refused = |text: &str| Config::parse(&format!("[content]\npacks = \"p\"\n{text}"), &base);
    assert!(refused(&format!("{map}{map}")).is_err(), "a taken id");
    let offer = "[[offer]]\nid = \"m\"\nscenario = \"s.toml\"\n";
    assert!(refused(&format!("{offer}{map}")).is_err(), "an offer's id");
    assert!(
        refused(&map.replace("id = \"m\"", "id = \"\"")).is_err(),
        "an empty id"
    );
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
    };
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
    let mut a = greeted(running).await;
    a.create_map("duel-map", 0).await.unwrap();
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
