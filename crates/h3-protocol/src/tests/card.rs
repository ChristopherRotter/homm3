//! The server's card (E16 M6 T2; ADR-0021 point 2): signed over its purpose and its text, read
//! back only when its own key's signature verifies, it has not expired and was not issued too far
//! ahead, and within its bounds; the chain of vouching of a relayed card (E16 M6 T3).

use std::string::{String, ToString};
use std::vec;
use std::vec::Vec;

use proptest::prelude::*;

use crate::account::{self, card_message};
use crate::card::{
    CardError, MAX_ADDRESSES, MAX_CARD_BYTES, MAX_DESCRIPTION, MAX_LISTED, MAX_OPERATOR, MAX_PACKS,
    MAX_PEERS, MAX_TEXT, SKEW_MS, chain, fits, is_address, read, sign,
};
use crate::codec;
use crate::connection::Signer;
use crate::wire::{CardContent, CardLadder, CardPeer, PackRef, ServerCard, SignedCard, WireU64};

use super::Fold;

/// When the card was issued; it expires a day later.
const ISSUED: u64 = 1_800_000_000_000;

/// A day in milliseconds.
const DAY: u64 = 86_400_000;

/// A card of [`Fold`]'s key with one of each list.
pub(super) fn card() -> ServerCard {
    ServerCard {
        key: account::to_hex(&Fold.key()),
        name: "Ann's arena".to_string(),
        description: "Ranked duels.\nBe kind.".to_string(),
        addresses: vec![
            "wss://h3.example.org".to_string(),
            "ws://192.168.1.5:7878".to_string(),
        ],
        protocol: crate::PROTOCOL_VERSION,
        engine: "0.1.0".to_string(),
        content: vec![CardContent {
            packs: vec![PackRef {
                id: "synthetic".to_string(),
                version: "1".to_string(),
            }],
            profile: "synthetic:faithful".to_string(),
            ruleset_hash: WireU64(0x0123_4567_89ab_cdef),
            battle_ruleset_hash: WireU64(7),
        }],
        ladders: vec![CardLadder {
            id: "arena".to_string(),
            name: "Arena".to_string(),
            season: 2,
            players: Some(14),
        }],
        players_online: Some(3),
        games_open: Some(1),
        anonymous: true,
        operator: "ann@example.org".to_string(),
        issued: WireU64(ISSUED),
        expires: WireU64(ISSUED + DAY),
        peers: vec![CardPeer {
            key: "11".repeat(32),
            address: "wss://h3.example.net".to_string(),
        }],
    }
}

/// [`Fold`]'s verification: its key, and its signature of the message.
fn folded(key: &[u8; 32], message: &[u8], signature: &[u8; 64]) -> bool {
    *key == Fold.key() && Fold.sign(message) == *signature
}

/// `card` signed by [`Fold`], read at `now`.
fn signed_and_read(card: &ServerCard, now: u64) -> Result<ServerCard, CardError> {
    read(&sign(card, &Fold), now, folded)
}

#[test]
fn a_card_signs_its_purpose_and_its_text() {
    let mut expected: Vec<u8> = b"h3-card-v1".to_vec();
    expected.extend_from_slice(&5u32.to_be_bytes());
    expected.extend_from_slice(b"{\"a\"}");
    assert_eq!(card_message("{\"a\"}"), expected);
}

#[test]
fn a_signed_card_is_its_text_and_reads_back() {
    let signed = sign(&card(), &Fold);
    assert_eq!(signed.card, codec::encode_text(&card()));
    assert_eq!(
        signed.signature,
        account::to_hex(&Fold.sign(&card_message(&signed.card)))
    );
    let start = std::format!(
        r#"{{"key":"{}","name":"Ann's arena","description":"Ranked duels.\nBe kind.","addresses":["wss://h3.example.org","ws://192.168.1.5:7878"],"protocol":{},"#,
        "5a".repeat(32),
        crate::PROTOCOL_VERSION
    );
    assert!(signed.card.starts_with(&start), "{}", signed.card);
    assert!(
        signed.card.ends_with(&std::format!(
            r#""issued":"{ISSUED}","expires":"{}","peers":[{{"key":"{}","address":"wss://h3.example.net"}}]}}"#,
            ISSUED + DAY,
            "11".repeat(32)
        )),
        "{}",
        signed.card
    );
    assert_eq!(read(&signed, ISSUED, folded), Ok(card()));
    // No counts on the card: `null`, and it reads back so.
    let mut quiet = card();
    quiet.players_online = None;
    quiet.games_open = None;
    quiet.ladders[0].players = None;
    let text = sign(&quiet, &Fold).card;
    assert!(text.contains(r#""players":null"#), "{text}");
    assert!(
        text.contains(r#""players_online":null,"games_open":null"#),
        "{text}"
    );
    assert_eq!(signed_and_read(&quiet, ISSUED), Ok(quiet));
}

#[test]
fn a_tampered_card_does_not_verify() {
    let signed = sign(&card(), &Fold);
    // Another name in the text, its signature kept.
    let renamed = SignedCard {
        card: signed.card.replace("Ann's arena", "Bob's arena"),
        signature: signed.signature.clone(),
    };
    assert_eq!(read(&renamed, ISSUED, folded), Err(CardError::Signature));
    // The same card re-encoded with another spacing: not the text signed.
    let spaced = SignedCard {
        card: signed.card.replacen(',', ", ", 1),
        signature: signed.signature.clone(),
    };
    assert_eq!(read(&spaced, ISSUED, folded), Err(CardError::Signature));
    // Another signature.
    let mut forged = signed.clone();
    forged.signature = "00".repeat(64);
    assert_eq!(read(&forged, ISSUED, folded), Err(CardError::Signature));
    // A signature that is not 64 bytes of hex.
    for bad in ["", "zz", &"00".repeat(63)] {
        let mut odd = signed.clone();
        odd.signature = bad.to_string();
        assert_eq!(
            read(&odd, ISSUED, folded),
            Err(CardError::Malformed),
            "{bad}"
        );
    }
}

#[test]
fn a_card_naming_another_key_does_not_verify_by_it() {
    // Signed by Fold's key, but naming another: its key's signature does not hold.
    let mut foreign = card();
    foreign.key = "22".repeat(32);
    assert_eq!(signed_and_read(&foreign, ISSUED), Err(CardError::Signature));
}

#[test]
fn a_card_holds_from_its_issue_less_the_skew_until_its_expiry() {
    let card = card();
    let expires = ISSUED + DAY;
    assert_eq!(signed_and_read(&card, expires - 1), Ok(card.clone()));
    assert_eq!(signed_and_read(&card, expires), Err(CardError::Expired));
    assert_eq!(signed_and_read(&card, u64::MAX), Err(CardError::Expired));
    // Issued ahead of the reader's clock: within the skew, it holds.
    assert_eq!(signed_and_read(&card, ISSUED - SKEW_MS), Ok(card.clone()));
    assert_eq!(
        signed_and_read(&card, ISSUED - SKEW_MS - 1),
        Err(CardError::Ahead)
    );
    // A card issued at the epoch's end: the reader's clock does not wrap.
    let mut late = card;
    late.issued = WireU64(u64::MAX - 1);
    late.expires = WireU64(u64::MAX);
    assert_eq!(signed_and_read(&late, 0), Err(CardError::Ahead));
}

#[test]
fn a_card_too_large_or_not_a_card_is_refused() {
    let signed = sign(&card(), &Fold);
    let mut large = signed.clone();
    large.card = " ".repeat(MAX_CARD_BYTES + 1);
    assert_eq!(read(&large, ISSUED, folded), Err(CardError::TooLarge));
    for text in [
        "",
        "{}",
        "not a card",
        &signed.card.replacen("{", r#"{"extra":1,"#, 1),
        &signed.card.replace(r#""protocol":"#, r#""protocol":-"#),
    ] {
        let mut bad = signed.clone();
        bad.card = text.to_string();
        assert_eq!(
            read(&bad, ISSUED, folded),
            Err(CardError::Malformed),
            "{text}"
        );
    }
    // At the bound, the text is read (and is no card).
    large.card = " ".repeat(MAX_CARD_BYTES);
    assert_eq!(read(&large, ISSUED, folded), Err(CardError::Malformed));
}

/// A change to the card that breaks one of its bounds.
type Breach = fn(&mut ServerCard);

#[test]
fn a_card_out_of_its_bounds_is_malformed() {
    let long = "x".repeat(MAX_TEXT + 1);
    let address = |n: usize| std::format!("ws://h{n}.example.org");
    let breaches: Vec<(&str, Breach)> = vec![
        ("a key that is not hex", |c| c.key = "5A".repeat(32)),
        ("no name", |c| c.name = String::new()),
        ("a long name", |c| c.name = "x".repeat(MAX_TEXT + 1)),
        ("a name of two lines", |c| c.name = "a\nb".to_string()),
        ("a long description", |c| {
            c.description = "x".repeat(MAX_DESCRIPTION + 1);
        }),
        ("a tab in the description", |c| {
            c.description = "a\tb".to_string();
        }),
        ("no address", |c| c.addresses.clear()),
        ("too many addresses", |c| {
            c.addresses = (0..=MAX_ADDRESSES)
                .map(|n| std::format!("ws://h{n}.example.org"))
                .collect();
        }),
        ("an address of http", |c| {
            c.addresses[1] = "http://h3.example.org".to_string();
        }),
        ("no engine", |c| c.engine = String::new()),
        ("too many rulesets", |c| {
            c.content = vec![c.content[0].clone(); MAX_LISTED + 1];
        }),
        ("too many packs", |c| {
            c.content[0].packs = vec![c.content[0].packs[0].clone(); MAX_PACKS + 1];
        }),
        ("a pack without an id", |c| {
            c.content[0].packs[0].id = String::new();
        }),
        ("a pack without a version", |c| {
            c.content[0].packs[0].version = String::new();
        }),
        ("no profile", |c| c.content[0].profile = String::new()),
        ("too many ladders", |c| {
            c.ladders = vec![c.ladders[0].clone(); MAX_LISTED + 1];
        }),
        ("a ladder without an id", |c| {
            c.ladders[0].id = String::new()
        }),
        ("a ladder without a name", |c| {
            c.ladders[0].name = String::new();
        }),
        ("a long operator contact", |c| {
            c.operator = "x".repeat(MAX_OPERATOR + 1);
        }),
        ("an operator contact of two lines", |c| {
            c.operator = "a\nb".to_string();
        }),
        ("expiring as it is issued", |c| c.expires = c.issued),
        ("too many peers", |c| {
            c.peers = vec![c.peers[0].clone(); MAX_PEERS + 1];
        }),
        ("a peer's key that is not hex", |c| {
            c.peers[0].key = "11".repeat(31);
        }),
        ("a peer's address that is not one", |c| {
            c.peers[0].address = "h3.example.net".to_string();
        }),
    ];
    assert!(fits(&card()));
    for (what, breach) in breaches {
        let mut broken = card();
        breach(&mut broken);
        assert!(!fits(&broken), "{what}");
        assert_eq!(
            signed_and_read(&broken, ISSUED),
            Err(CardError::Malformed),
            "{what}"
        );
    }
    // At their bounds, they fit.
    let mut full = card();
    full.name = long[1..].to_string();
    full.addresses = (0..MAX_ADDRESSES).map(address).collect();
    full.peers = vec![full.peers[0].clone(); MAX_PEERS];
    full.ladders = vec![full.ladders[0].clone(); MAX_LISTED];
    full.content[0].packs = vec![full.content[0].packs[0].clone(); MAX_PACKS];
    full.content = vec![full.content[0].clone(); MAX_LISTED];
    assert!(fits(&full));
}

#[test]
fn an_address_is_a_websocket_url() {
    for good in [
        "ws://a",
        "wss://h3.example.org:443/x",
        &std::format!("ws://{}", "a".repeat(200)),
    ] {
        assert!(is_address(good), "{good}");
    }
    for bad in [
        "",
        "ws://",
        "wss://",
        "http://a",
        "ws://a b",
        "WS://a",
        "ws://é",
        &std::format!("ws://{}", "a".repeat(201)),
    ] {
        assert!(!is_address(bad), "{bad}");
    }
}

/// E16 M6 T3: a chain holds when each card names the next one's key among its peers, whatever
/// the addresses; a single card, and none, are chains.
#[test]
fn a_chain_vouches_card_by_card() {
    let server = |key: &str, peers: &[&str]| ServerCard {
        key: key.repeat(32),
        peers: (peers.iter())
            .map(|peer| CardPeer {
                key: peer.repeat(32),
                address: "ws://elsewhere".to_string(),
            })
            .collect(),
        ..card()
    };
    let (a, b, c) = (
        server("aa", &["bb", "dd"]),
        server("bb", &["cc"]),
        server("cc", &[]),
    );
    assert!(chain(&[&a, &b]));
    assert!(chain(&[&a, &b, &c]));
    assert!(chain(&[&c]));
    assert!(chain(&[]));
    // A card its voucher does not name, a link left out, a chain backwards.
    assert!(!chain(&[&b, &a]));
    assert!(!chain(&[&a, &c]));
    assert!(!chain(&[&c, &b, &a]));
}

proptest! {
    /// Reading a card never panics, whatever its text and signature (E16 M6 T2).
    #[test]
    fn reading_any_card_never_panics(text in ".{0,300}", signature in "[0-9a-f]{0,130}") {
        let signed = SignedCard { card: text, signature };
        prop_assert!(read(&signed, ISSUED, folded).is_err());
    }
}
