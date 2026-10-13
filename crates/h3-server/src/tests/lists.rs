//! Lists of servers over loopback (E16 M6 T5; ADR-0021 point 5): the lists in a server's
//! `<data>/lists/` are read at start and at each round; one expired, forged, malformed or of a key
//! an earlier file holds is skipped; the servers the good ones name are fetched and their cards
//! relayed marked with the list's key, which the test client checks against `Lists`; the
//! operator's tools make a key and a signed list that verifies.

use std::fs;
use std::path::{Path, PathBuf};

use h3_protocol::account;
use h3_protocol::connection::Signer as _;
use h3_protocol::list;
use h3_protocol::wire::{ListedServer, ServerList, SignedList, WireU64};

use super::*;
use crate::card::wall_ms;
use crate::key::ServerKey;
use crate::lists::{self, LISTS_DIR};

/// A day in milliseconds.
const DAY: u64 = 86_400_000;

/// A fetch's round trips well within this.
const WAIT: Duration = Duration::from_secs(20);

/// `running`'s key and its address, as a list names it.
async fn listed(running: &Running) -> ListedServer {
    let client = greeted(running).await;
    ListedServer {
        key: account::to_hex(&client.server_key().unwrap()),
        address: format!("ws://{}", running.address),
    }
}

/// The list `name` of the key of the seed `seed`, naming `servers`, from `issued` to `expires`.
fn list_of(
    seed: u8,
    name: &str,
    (issued, expires): (u64, u64),
    servers: &[&ListedServer],
) -> ServerList {
    ServerList {
        key: account::to_hex(&ServerKey::from_seed([seed; 32]).key()),
        name: name.to_owned(),
        issued: WireU64(issued),
        expires: WireU64(expires),
        servers: servers.iter().map(|&s| s.clone()).collect(),
    }
}

/// `list` signed by the key of the seed `seed`.
fn signed_by(list: &ServerList, seed: u8) -> SignedList {
    list::sign(list, &ServerKey::from_seed([seed; 32]))
}

/// Writes `text` as the file `name` in `data`'s lists.
fn place(data: &Path, name: &str, text: &str) {
    let dir = data.join(LISTS_DIR);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(name), text).unwrap();
}

/// What `running` relays, each card checked by the test client against its chain or its list:
/// each card's key and its list's key.
async fn marked(running: &Running) -> Vec<(String, Option<String>)> {
    let mut client = greeted(running).await;
    let (own, _) = client.describe(1).await.unwrap();
    (client.relayed(&own).await.unwrap().into_iter())
        .map(|(card, entry)| {
            assert!(entry.via.is_empty());
            (card.key, entry.list)
        })
        .collect()
}

/// A server that fetches every 200 ms (below the configuration's bounds).
fn fetching() -> Config {
    let mut config = config(fast());
    config.discovery.refresh_ms = 200;
    config.discovery.timeout_ms = 5_000;
    config
}

/// Resolves once `running` did `rounds` rounds of fetches since its start.
async fn rounds(running: &Running, rounds: u64) {
    tokio::time::timeout(WAIT, running.peers.fetched(rounds))
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_lists_servers_are_relayed_marked_with_its_key_and_bad_lists_skipped() {
    let b = start(&config(fast())).await.unwrap();
    let d = start(&config(fast())).await.unwrap();
    let (lb, ld) = (listed(&b).await, listed(&d).await);
    let now = wall_ms();
    let day = (now, now + DAY);
    let a = fetching();
    let data = a.server.data.clone();
    // The good list (seed 1) names B; every other list names D.
    let good = signed_by(&list_of(1, "Community", day, &[&lb]), 1);
    place(&data, "a.json", &codec::encode_text(&good));
    let expired = list_of(2, "Old", (now - 2 * DAY, now - DAY), &[&ld]);
    place(
        &data,
        "b.json",
        &codec::encode_text(&signed_by(&expired, 2)),
    );
    // Naming the key of seed 3, signed by seed 4's.
    let forged = signed_by(&list_of(3, "Forged", day, &[&ld]), 4);
    place(&data, "c.json", &codec::encode_text(&forged));
    place(&data, "d.json", "not a signed list");
    // The good list's key again, in a later file.
    let again = signed_by(&list_of(1, "Again", day, &[&ld]), 1);
    place(&data, "e.json", &codec::encode_text(&again));
    // Not a list's file name.
    let other = signed_by(&list_of(5, "Other", day, &[&ld]), 5);
    place(&data, "f.txt", &codec::encode_text(&other));
    let a = start(&a).await.unwrap();
    rounds(&a, 1).await;
    // Lists gives the good list alone, as signed; B is relayed marked with its key, D is not.
    let mut client = greeted(&a).await;
    let lists = client.lists(2).await.unwrap();
    assert_eq!(lists.len(), 1);
    assert_eq!(lists[0].1, good);
    assert_eq!(
        marked(&a).await,
        [(lb.key.clone(), Some(lists[0].0.key.clone()))]
    );
    // The files of its key removed: from the next round on, neither the list nor B.
    for file in ["a.json", "e.json"] {
        fs::remove_file(data.join(LISTS_DIR).join(file)).unwrap();
    }
    rounds(&a, a.peers.rounds() + 2).await;
    assert!(client.lists(3).await.unwrap().is_empty());
    assert!(marked(&a).await.is_empty());
    for server in [a, b, d] {
        server.shutdown().await;
    }
}

/// A fresh directory for the tools' files.
fn workdir(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("h3-lists-{name}-{:016x}", game::os_u64().unwrap()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn the_tools_make_a_key_and_a_list_that_verifies() {
    let dir = workdir("tools");
    let key = dir.join("list.key");
    // The key: written once, the same public key after.
    let public = lists::list_key(&key).unwrap();
    assert_eq!(lists::list_key(&key).unwrap(), public);
    let toml = format!(
        "name = \"Community\"\nvalid_days = 30\n\n[[server]]\nkey = \"{}\"\naddress = \"wss://h3.example.org\"\n\n[[server]]\nkey = \"{}\"\naddress = \"ws://192.168.1.6:7878\"\n",
        "11".repeat(32),
        "22".repeat(32)
    );
    let list_file = dir.join("list.toml");
    fs::write(&list_file, &toml).unwrap();
    let before = wall_ms();
    let text = lists::sign_list(&key, &list_file).unwrap();
    let signed: SignedList = serde_json::from_str(&text).unwrap();
    let read = lists::verify(&signed, wall_ms()).unwrap();
    assert_eq!(read.key, public);
    assert_eq!(read.name, "Community");
    assert_eq!(read.servers.len(), 2);
    assert_eq!(read.servers[1].address, "ws://192.168.1.6:7878");
    assert!(read.issued.0 >= before && read.expires.0 == read.issued.0 + 30 * DAY);
    // Placed in a server's lists, it reads as it was made.
    let data = dir.join("data");
    place(&data, "community.json", &text);
    let held = lists::read(&data.join(LISTS_DIR), wall_ms());
    assert_eq!(held.len(), 1);
    assert_eq!(held[0].signed, signed);
    // The tools refuse: a missing key file, lists out of their bounds, a TOML that is no list.
    let missing = lists::sign_list(&dir.join("none.key"), &list_file).unwrap_err();
    assert!(missing.0.contains("none.key"), "{missing}");
    for (bad, why) in [
        (
            toml.replace("valid_days = 30", "valid_days = 0"),
            "valid_days = 0: from 1 to 366",
        ),
        (
            toml.replace("valid_days = 30", "valid_days = 367"),
            "valid_days",
        ),
        (
            toml.replace("name = \"Community\"", "name = \"\""),
            "a list names itself",
        ),
        (
            toml.replace(&"22".repeat(32), &"11".repeat(32)),
            "no key twice",
        ),
        (
            toml.replace("wss://h3.example.org", "https://h3.example.org"),
            "a list names itself",
        ),
        (toml.replace("valid_days", "days"), "unknown field"),
    ] {
        fs::write(&list_file, &bad).unwrap();
        let error = lists::sign_list(&key, &list_file).unwrap_err();
        assert!(error.0.contains(why), "{why}: {error}");
    }
    // A key file that is not a key is never replaced.
    fs::write(&key, "not a key").unwrap();
    assert!(lists::list_key(&key).unwrap_err().0.contains("not a key"));
    assert_eq!(fs::read_to_string(&key).unwrap(), "not a key");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn the_lists_files_are_bounded_and_read_in_order() {
    let dir = workdir("bounds");
    let data = dir.join("data");
    let now = wall_ms();
    let day = (now, now + DAY);
    // No directory: no list.
    assert!(lists::read(&data.join(LISTS_DIR), now).is_empty());
    // Seventeen good lists of seventeen keys: the first sixteen by name are read.
    for n in 0..17u8 {
        let signed = signed_by(&list_of(n + 1, &format!("L{n}"), day, &[]), n + 1);
        place(&data, &format!("{n:02}.json"), &codec::encode_text(&signed));
    }
    let held = lists::read(&data.join(LISTS_DIR), now);
    assert_eq!(held.len(), list::MAX_LISTS);
    assert_eq!(
        (held[0].list.name.as_str(), held[15].list.name.as_str()),
        ("L0", "L15")
    );
    // A file larger than a list may be: not read.
    let large = " ".repeat(2 * list::MAX_LIST_BYTES + 1025);
    place(&data, "00.json", &large);
    let held = lists::read(&data.join(LISTS_DIR), now);
    assert_eq!(held[0].list.name, "L1");
    // A directory where a list's file should be: skipped.
    fs::remove_file(data.join(LISTS_DIR).join("00.json")).unwrap();
    fs::create_dir(data.join(LISTS_DIR).join("00.json")).unwrap();
    assert_eq!(lists::read(&data.join(LISTS_DIR), now)[0].list.name, "L1");
    fs::remove_dir_all(&dir).unwrap();
}
