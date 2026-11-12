//! Whole games over the network (E16 M5 T9a): a map offer in the lobby, a whole game created from
//! it and joined by two clients, its commands answered once, its journal written ahead and kept as
//! it is over a restart (T9b restores it), the refusals, an abandoned game.

use std::fs;

use h3_contracts::PlayerId;
use h3_protocol::map::game::manifest;
use h3_protocol::session::game::{GameEntry, GameOptions, GameSession};
use h3_protocol::wire::game::{GameCommand, MapGameInfo, MapOffer};
use h3_protocol::wire::{ClientMessage, GoodbyeReason, Phase, Refusal, Seat, ServerMessage};

use super::{fast, greeted, root};
use crate::client::Client;
use crate::config::{self, Config};
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
    assert_eq!(read.start.tokens, [t0.clone(), t1]);
    let p = PlayerId::new;
    let end = h3_game::GameCommand::EndTurn;
    assert_eq!(
        read.entries,
        [
            GameEntry::Taken(p(0)),
            GameEntry::Taken(p(1)),
            GameEntry::Started,
            GameEntry::Numbered(p(0), 1, end),
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
    // A restart keeps the whole game's journal as it is, and lists no game of it until T9b.
    let before = fs::read(&path).unwrap();
    for client in [a, b] {
        client.close().await;
    }
    running.shutdown().await;
    let again = start(&config).await.unwrap();
    assert!(map_listed(&again).await.1.is_empty());
    assert_eq!(fs::read(&path).unwrap(), before);
    let mut c = greeted(&again).await;
    c.create_map("duel-map", 1).await.unwrap();
    let created = c.recv().await;
    assert!(
        matches!(created, Some(ServerMessage::GameCreated { game: g, .. }) if g > game),
        "{created:?}"
    );
    again.shutdown().await;
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
