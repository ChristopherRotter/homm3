//! Lists of servers (E16 M6 T5; ADR-0021 point 5): signed over their purpose and their text, read
//! back only when their own key's signature verifies, they have not expired and were not issued
//! too far ahead, and within their bounds; a card relayed for a list holds for it when the list
//! names the card's key at one of its addresses.

use std::string::{String, ToString};
use std::vec;
use std::vec::Vec;

use crate::account::{self, list_message};
use crate::card::{CardError, MAX_TEXT, SKEW_MS};
use crate::codec;
use crate::connection::Signer;
use crate::list::{MAX_LIST_BYTES, MAX_SERVERS, fits, names, read, sign};
use crate::wire::{ListedServer, ServerList, SignedList, WireU64};

use super::Fold;

/// When the list was issued; it expires a day later.
const ISSUED: u64 = 1_800_000_000_000;

/// A day in milliseconds.
const DAY: u64 = 86_400_000;

/// A list of [`Fold`]'s key naming the card of `super::card` and another server.
pub(super) fn list() -> ServerList {
    ServerList {
        key: account::to_hex(&Fold.key()),
        name: "Community".to_string(),
        issued: WireU64(ISSUED),
        expires: WireU64(ISSUED + DAY),
        servers: vec![
            ListedServer {
                key: "5a".repeat(32),
                address: "wss://h3.example.org".to_string(),
            },
            ListedServer {
                key: "33".repeat(32),
                address: "ws://192.168.1.6:7878".to_string(),
            },
        ],
    }
}

/// [`Fold`]'s verification: its key, and its signature of the message.
fn folded(key: &[u8; 32], message: &[u8], signature: &[u8; 64]) -> bool {
    *key == Fold.key() && Fold.sign(message) == *signature
}

/// `list` signed by [`Fold`], read at `now`.
fn signed_and_read(list: &ServerList, now: u64) -> Result<ServerList, CardError> {
    read(&sign(list, &Fold), now, folded)
}

#[test]
fn a_list_signs_its_purpose_and_its_text() {
    let mut expected: Vec<u8> = b"h3-list-v1".to_vec();
    expected.extend_from_slice(&5u32.to_be_bytes());
    expected.extend_from_slice(b"{\"a\"}");
    assert_eq!(list_message("{\"a\"}"), expected);
    // Not a card's: a card's signature is good for no list.
    assert_ne!(list_message("{}"), account::card_message("{}"));
}

#[test]
fn a_signed_list_is_its_text_and_reads_back() {
    let signed = sign(&list(), &Fold);
    assert_eq!(signed.list, codec::encode_text(&list()));
    assert_eq!(
        signed.signature,
        account::to_hex(&Fold.sign(&list_message(&signed.list)))
    );
    let expected = std::format!(
        r#"{{"key":"{}","name":"Community","issued":"{ISSUED}","expires":"{}","servers":[{{"key":"{}","address":"wss://h3.example.org"}},{{"key":"{}","address":"ws://192.168.1.6:7878"}}]}}"#,
        "5a".repeat(32),
        ISSUED + DAY,
        "5a".repeat(32),
        "33".repeat(32)
    );
    assert_eq!(signed.list, expected);
    assert_eq!(read(&signed, ISSUED, folded), Ok(list()));
    // A list of no server holds too.
    let mut empty = list();
    empty.servers.clear();
    assert_eq!(signed_and_read(&empty, ISSUED), Ok(empty));
}

#[test]
fn a_tampered_or_foreign_list_does_not_verify() {
    let signed = sign(&list(), &Fold);
    let renamed = SignedList {
        list: signed.list.replace("Community", "Communitz"),
        signature: signed.signature.clone(),
    };
    assert_eq!(read(&renamed, ISSUED, folded), Err(CardError::Signature));
    let mut forged = signed.clone();
    forged.signature = "00".repeat(64);
    assert_eq!(read(&forged, ISSUED, folded), Err(CardError::Signature));
    for bad in ["", "zz", &"00".repeat(63)] {
        let mut odd = signed.clone();
        odd.signature = bad.to_string();
        assert_eq!(read(&odd, ISSUED, folded), Err(CardError::Malformed));
    }
    // Signed by Fold's key, but naming another.
    let mut foreign = list();
    foreign.key = "22".repeat(32);
    assert_eq!(signed_and_read(&foreign, ISSUED), Err(CardError::Signature));
    // A card's text signed as a card does not read as a list.
    let card = crate::card::sign(&super::card::card(), &Fold);
    let as_list = SignedList {
        list: card.card,
        signature: card.signature,
    };
    assert_eq!(read(&as_list, ISSUED, folded), Err(CardError::Malformed));
}

#[test]
fn a_list_holds_from_its_issue_less_the_skew_until_its_expiry() {
    let list = list();
    let expires = ISSUED + DAY;
    assert_eq!(signed_and_read(&list, expires - 1), Ok(list.clone()));
    assert_eq!(signed_and_read(&list, expires), Err(CardError::Expired));
    assert_eq!(signed_and_read(&list, ISSUED - SKEW_MS), Ok(list.clone()));
    assert_eq!(
        signed_and_read(&list, ISSUED - SKEW_MS - 1),
        Err(CardError::Ahead)
    );
}

#[test]
fn a_list_too_large_or_not_a_list_is_refused() {
    let signed = sign(&list(), &Fold);
    let mut large = signed.clone();
    large.list = " ".repeat(MAX_LIST_BYTES + 1);
    assert_eq!(read(&large, ISSUED, folded), Err(CardError::TooLarge));
    large.list = " ".repeat(MAX_LIST_BYTES);
    assert_eq!(read(&large, ISSUED, folded), Err(CardError::Malformed));
    for text in [
        "",
        "{}",
        "not a list",
        &signed.list.replacen("{", r#"{"extra":1,"#, 1),
    ] {
        let mut bad = signed.clone();
        bad.list = text.to_string();
        assert_eq!(read(&bad, ISSUED, folded), Err(CardError::Malformed));
    }
}

/// A change to the list that breaks one of its bounds.
type Breach = fn(&mut ServerList);

#[test]
fn a_list_out_of_its_bounds_is_malformed() {
    let server = |n: usize| ListedServer {
        key: std::format!("{:064x}", n + 1),
        address: std::format!("ws://h{n}.example.org"),
    };
    let breaches: Vec<(&str, Breach)> = vec![
        ("a key that is not hex", |l| l.key = "5A".repeat(32)),
        ("no name", |l| l.name = String::new()),
        ("a long name", |l| l.name = "x".repeat(MAX_TEXT + 1)),
        ("a name of two lines", |l| l.name = "a\nb".to_string()),
        ("expiring when issued", |l| l.expires = l.issued),
        ("a server's key", |l| l.servers[1].key = "33".repeat(31)),
        ("a server's address", |l| {
            l.servers[1].address = "https://h3.example.org".to_string();
        }),
        ("a server twice", |l| {
            l.servers[1].key = l.servers[0].key.clone();
        }),
    ];
    for (what, breach) in breaches {
        let mut bad = list();
        breach(&mut bad);
        assert!(!fits(&bad), "{what}");
        assert_eq!(
            signed_and_read(&bad, ISSUED),
            Err(CardError::Malformed),
            "{what}"
        );
    }
    // At most MAX_SERVERS servers.
    let mut most = list();
    most.servers = (0..MAX_SERVERS).map(server).collect();
    assert!(fits(&most));
    most.servers.push(server(MAX_SERVERS));
    assert!(!fits(&most));
    // A name at its bound.
    let mut named = list();
    named.name = "x".repeat(MAX_TEXT);
    assert!(fits(&named));
}

#[test]
fn a_list_names_a_card_by_its_key_at_one_of_its_addresses() {
    let card = super::card::card();
    let list = list();
    assert!(names(&list, &card));
    // The card's other address names it too.
    let mut lan = list.clone();
    lan.servers[0].address = "ws://192.168.1.5:7878".to_string();
    assert!(names(&lan, &card));
    // Its key at an address that is not the card's; another key at the card's address.
    let mut elsewhere = list.clone();
    elsewhere.servers[0].address = "wss://h3.example.net".to_string();
    assert!(!names(&elsewhere, &card));
    let mut other = list;
    other.servers[0].key = "44".repeat(32);
    assert!(!names(&other, &card));
}
