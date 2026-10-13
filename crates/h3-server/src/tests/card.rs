//! The server's card over loopback (E16 M6 T2; ADR-0021 point 2): a client's `Describe` after its
//! hello gets the card its server signed, which verifies by the welcome's key (the test client
//! checks every card so); the card names what the configuration and the content say, and counts
//! only where the configuration lets it; it is signed again only once a minute old; `h3-server
//! card` prints the running server's; the card's configuration is bounded.

use h3_protocol::PROTOCOL_VERSION;
use h3_protocol::wire::{CardContent, CardLadder, Offer as OfferInfo, SignedCard};

use super::*;
use crate::card::{verify, wall_ms};
use crate::config::Ladder;

/// A server named and described, at a public address and a LAN one, with the ladder `arena` (the
/// duel), counting or not as `counts`.
fn described(counts: bool) -> Config {
    let mut config = config(Limits::default());
    config.server.public_url = Some(String::from("wss://h3.example.org"));
    config.server.addresses = vec![String::from("ws://192.168.1.5:7878")];
    config.server.name = Some(String::from("Ann's arena"));
    config.server.description = String::from("Ranked duels.\nBe kind.");
    config.server.operator = String::from("ann@example.org");
    config.discovery.counts = counts;
    config.ladders = vec![Ladder {
        id: String::from("arena"),
        name: String::from("Arena"),
        scenarios: vec![root().join("packs/synthetic/scenarios/duel.toml")],
        ..Ladder::default()
    }];
    config
}

/// The server's offers as its lobby lists them.
async fn offers(client: &mut Client) -> Vec<OfferInfo> {
    client
        .send(&ClientMessage::ListGames { seq: 2 })
        .await
        .unwrap();
    let Some(ServerMessage::Games { offers, .. }) = client.recv().await else {
        panic!("no list")
    };
    offers
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_client_gets_the_servers_signed_card_after_its_hello() {
    let config = described(true);
    let running = start(&config).await.unwrap();
    let mut client = greeted(&running).await;
    let key = client.server_key().unwrap();
    let (card, signed) = client.describe(1).await.unwrap();
    assert_eq!(card.key, h3_protocol::account::to_hex(&key));
    assert_eq!(card.name, "Ann's arena");
    assert_eq!(card.description, "Ranked duels.\nBe kind.");
    assert_eq!(card.operator, "ann@example.org");
    assert_eq!(
        card.addresses,
        ["wss://h3.example.org", "ws://192.168.1.5:7878"]
    );
    assert_eq!(card.protocol, PROTOCOL_VERSION);
    assert_eq!(card.engine, env!("CARGO_PKG_VERSION"));
    assert!(card.anonymous);
    assert!(card.peers.is_empty());
    // Each ruleset of the offers and the ladder's pool once: the duel's twice in the configuration.
    let listed = offers(&mut client).await;
    let rulesets: Vec<CardContent> = (listed.iter())
        .map(|o| CardContent {
            packs: o.packs.clone(),
            profile: o.profile.clone(),
            ruleset_hash: o.ruleset_hash,
            battle_ruleset_hash: o.battle_ruleset_hash,
        })
        .collect();
    assert_eq!(card.content, crate::card::distinct(rulesets.clone()));
    assert!(rulesets.iter().all(|r| card.content.contains(r)));
    assert_eq!(
        card.ladders,
        [CardLadder {
            id: String::from("arena"),
            name: String::from("Arena"),
            season: 1,
            players: Some(0),
        }]
    );
    // The asker's connection is open; no game is.
    assert_eq!((card.players_online, card.games_open), (Some(1), Some(0)));
    // Issued now, it holds a day.
    let now = wall_ms();
    assert!(card.issued.0 <= now && now - card.issued.0 < 60_000);
    assert_eq!(card.expires.0 - card.issued.0, 86_400_000);
    // Within the minute, every client gets the same card, counts and all, though a game opened.
    let (mut creator, _, _) = created(&running, "pvp", Seat::Attacker).await;
    let (_, again) = creator.describe(7).await.unwrap();
    assert_eq!(again, signed);
    let (_, again) = client.describe(8).await.unwrap();
    assert_eq!(again, signed);
    running.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_server_that_keeps_its_counts_off_its_card_names_none() {
    let mut config = described(false);
    config.server.anonymous = false;
    config.server.name = None;
    config.discovery.card_ms = 600_000;
    let running = start(&config).await.unwrap();
    let mut client = greeted(&running).await;
    let (card, _) = client.describe(1).await.unwrap();
    assert_eq!((card.players_online, card.games_open), (None, None));
    assert_eq!(card.ladders[0].players, None);
    assert!(!card.anonymous);
    // Named by its id without a name.
    assert_eq!(card.name, "wss://h3.example.org");
    assert_eq!(card.expires.0 - card.issued.0, 600_000);
    running.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_card_asked_before_the_hello_closes_the_connection() {
    let running = start(&config(Limits::default())).await.unwrap();
    let mut client = Client::connect(running.address).await.unwrap();
    client
        .send(&ClientMessage::Describe { seq: 1 })
        .await
        .unwrap();
    assert_eq!(goodbye(&mut client).await, GoodbyeReason::Protocol);
    running.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_operators_card_is_the_running_servers() {
    let mut config = described(true);
    config.server.bind = String::from("0.0.0.0:0");
    // No server ran on the data directory: it has no key, and no card.
    let error = crate::card(&config).await.unwrap_err();
    assert!(error.0.contains("server.key"), "{error}");
    let running = start(&config).await.unwrap();
    // Asked at the address it listens on, unspecified: the loopback address.
    let mut asked = config.clone();
    asked.server.bind = format!("0.0.0.0:{}", running.address.port());
    let text = crate::card(&asked).await.unwrap();
    let signed: SignedCard = serde_json::from_str(&text).unwrap();
    let card = verify(&signed, wall_ms()).unwrap();
    let mut client = greeted(&running).await;
    let key = client.server_key().unwrap();
    assert_eq!(card.key, h3_protocol::account::to_hex(&key));
    assert_eq!(client.describe(1).await.unwrap().1, signed);
    // Another data directory's key: not this server's card.
    let mut other = asked.clone();
    other.server.data = config.server.data.with_extension("other");
    std::fs::create_dir_all(&other.server.data).unwrap();
    crate::key::ServerKey::open(&other.server.data).unwrap();
    let error = crate::card(&other).await.unwrap_err();
    assert!(error.0.contains("the server's key changed"), "{error}");
    running.shutdown().await;
    let error = crate::card(&asked).await.unwrap_err();
    assert!(error.0.contains("connect"), "{error}");
}

#[test]
fn the_cards_configuration_is_bounded() {
    let base = root();
    let parse = |server: &str, discovery: &str| {
        let text = format!(
            "[server]\n{server}\n[discovery]\n{discovery}\n[content]\npacks = \"p\"\n\
             [[offer]]\nid = \"a\"\nscenario = \"s.toml\"\n"
        );
        Config::parse(&text, &base)
    };
    let defaults = parse("", "").unwrap();
    assert_eq!(defaults.server.name, None);
    assert_eq!(
        (defaults.server.description, defaults.server.operator),
        (String::new(), String::new())
    );
    assert_eq!(defaults.discovery, Discovery::default());
    assert_eq!(
        (defaults.discovery.card_ms, defaults.discovery.counts),
        (86_400_000, true)
    );
    let set = parse(
        "name = \"Ann's arena\"\ndescription = \"Duels.\\nBe kind.\"\noperator = \"ann\"",
        "card_ms = 600000\ncounts = false",
    )
    .unwrap();
    assert_eq!(set.server.name.as_deref(), Some("Ann's arena"));
    assert_eq!(set.server.description, "Duels.\nBe kind.");
    assert_eq!(
        (set.discovery.card_ms, set.discovery.counts),
        (600_000, false)
    );
    let long = "x".repeat(65);
    let addresses = (0..8)
        .map(|n| format!("\"ws://h{n}.example.org\""))
        .collect::<Vec<_>>()
        .join(", ");
    for (server, discovery, error) in [
        ("name = \"\"", "", "server.name"),
        (&format!("name = \"{long}\""), "", "server.name"),
        ("name = \"a\\nb\"", "", "server.name"),
        (
            &format!("description = \"{}\"", "x".repeat(1025)),
            "",
            "server.description",
        ),
        ("description = \"a\\tb\"", "", "server.description"),
        (
            &format!("operator = \"{}\"", "x".repeat(257)),
            "",
            "server.operator",
        ),
        ("operator = \"a\\nb\"", "", "server.operator"),
        (
            &format!("addresses = [{addresses}]"),
            "",
            "server.addresses",
        ),
        ("", "card_ms = 599999", "discovery.card_ms"),
        ("", "card_ms = 2592000001", "discovery.card_ms"),
        ("", "cards = 1", "unknown field"),
    ] {
        let parsed = parse(server, discovery).unwrap_err();
        assert!(parsed.0.contains(error), "{server} {discovery}: {parsed}");
    }
}
