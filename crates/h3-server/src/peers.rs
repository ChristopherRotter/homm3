//! The server's peers (E16 M6 T3; ADR-0021 point 3, `E16-design.md` §12): the servers its operator
//! vouches for (`[[peer]]`), whose cards it fetches and relays with `ListPeers`.
//!
//! One task fetches the peers' cards at start and every `[discovery] refresh_ms`, one peer after
//! the other in the configuration's order. A fetch dials the peer's address as a client does
//! (`wss://` through the TLS client, `ws://` plain), says hello, checks the welcome (signed by the
//! configured key, naming the address dialed as the peer's id: the key's holder answered there),
//! and asks `Describe`: the card must hold ([`card::verify`]) and be the configured key's. It reads
//! at most [`FRAME`] bytes a frame and [`FETCH`] in all, within `[discovery] timeout_ms`. A peer
//! that fails is logged and keeps its last good card until that card expires. The server dials
//! only the addresses its operator configured, never one it read in a card.
//!
//! The TLS client (ADR-0021 point 4): `rustls` with the pure-Rust graviola backend, certificates
//! checked against the operating system's roots, read once at start when a peer is at `wss://`.
//! Graviola runs on x86_64 and aarch64 with their crypto extensions only, and panics elsewhere at
//! its first use: each fetch runs as a task of its own, so such a panic fails the fetch, logged,
//! not the server.
//!
//! `ListPeers` pages the cards that hold, the peers' in the configuration's order (`Peers`).
//!
//! Provisional (E16 M6 T3): the frame bounds, the hello's name (the server's id), the log lines,
//! one fetch at a time.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use h3_protocol::account::{self, KEY_BYTES};
use h3_protocol::card::{MAX_CARD_BYTES, PAGE};
use h3_protocol::wire::{RelayedCard, ServerMessage, SignedCard};
use rustls::{ClientConfig, RootCertStore};
use tokio::sync::watch;
use tokio::task::JoinSet;
use tokio::time::{Instant, sleep_until, timeout};

use crate::Error;
use crate::card::wall_ms;
use crate::client::Client;
use crate::config::{Discovery, Peer};

/// The largest frame a fetch reads: a `Described` whose card holds (its text, escaped in the
/// frame, takes at most twice its bytes), with room for the rest of the message.
pub(crate) const FRAME: usize = 2 * MAX_CARD_BYTES + 1024;

/// The bytes a fetch reads in all: its welcome and its card, with room for pings.
pub(crate) const FETCH: usize = 2 * FRAME;

/// A configured peer.
#[derive(Debug, Clone)]
struct Configured {
    /// Its key.
    key: [u8; KEY_BYTES],
    /// Where its card is fetched.
    address: String,
}

/// A peer's last good card.
#[derive(Debug, Clone)]
struct Kept {
    /// As its key signed it.
    signed: SignedCard,
    /// When it expires, in milliseconds since the Unix epoch.
    expires: u64,
}

/// The peers and their last good cards.
#[derive(Debug)]
pub(crate) struct Peers {
    /// The peers, in the configuration's order.
    configured: Vec<Configured>,
    /// Each one's last good card, in the same order.
    cards: Mutex<Vec<Option<Kept>>>,
    /// The rounds of fetches done.
    rounds: watch::Sender<u64>,
}

impl Peers {
    /// The peers `peers` (`[[peer]]`), none fetched yet; why not, if a key is not 32 bytes of hex
    /// or is `own`, the server's own.
    pub(crate) fn new(peers: &[Peer], own: [u8; KEY_BYTES]) -> Result<Self, Error> {
        let configured = (peers.iter().enumerate())
            .map(|(i, peer)| match account::from_hex(&peer.key) {
                Some(key) if key != own => Ok(Configured {
                    key,
                    address: peer.address.clone(),
                }),
                Some(_) => Err(Error(format!("peer {i}: the server's own key"))),
                None => Err(Error(format!("peer {i}: not a key: {:?}", peer.key))),
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            cards: Mutex::new(vec![None; configured.len()]),
            configured,
            rounds: watch::Sender::new(0),
        })
    }

    /// The last good cards, locked.
    fn cards(&self) -> MutexGuard<'_, Vec<Option<Kept>>> {
        self.cards.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The answer to `ListPeers { seq, from }` at `now` (milliseconds since the Unix epoch): the
    /// cards that hold then, from the `from`th, at most a page of them.
    pub(crate) fn page(&self, seq: u32, from: u32, now: u64) -> ServerMessage {
        let cards = self.cards();
        let held: Vec<&Kept> = (cards.iter().flatten())
            .filter(|kept| kept.expires > now)
            .collect();
        let entries = (held.iter())
            .skip(usize::try_from(from).unwrap_or(usize::MAX))
            .take(PAGE)
            .map(|kept| RelayedCard {
                card: kept.signed.clone(),
                via: Vec::new(),
                list: None,
            })
            .collect();
        ServerMessage::Peers {
            seq,
            from,
            total: u32::try_from(held.len()).unwrap_or(u32::MAX),
            entries,
        }
    }

    /// The `i`th peer's new card.
    fn keep(&self, i: usize, kept: Kept) {
        if let Some(slot) = self.cards().get_mut(i) {
            *slot = Some(kept);
        }
    }

    /// Drops the cards expired at `now`.
    fn prune(&self, now: u64) {
        for slot in self.cards().iter_mut() {
            if slot.as_ref().is_some_and(|kept| kept.expires <= now) {
                *slot = None;
            }
        }
    }

    /// Resolves once `rounds` rounds of fetches are done.
    #[cfg(test)]
    pub(crate) async fn fetched(&self, rounds: u64) {
        let mut done = self.rounds.subscribe();
        let _ = done.wait_for(|done| *done >= rounds).await;
    }

    /// The rounds of fetches done.
    #[cfg(test)]
    pub(crate) fn rounds(&self) -> u64 {
        *self.rounds.borrow()
    }
}

/// Fetches the peers' cards at start and every `discovery.refresh_ms` (from one round's start to
/// the next's), each within `discovery.timeout_ms`, until the task is aborted; `own` is the
/// server's id, its hello's name. A fetch in flight ends with the task.
pub(crate) async fn fetch_every(
    peers: Arc<Peers>,
    tls: Arc<ClientConfig>,
    discovery: Discovery,
    own: String,
) {
    let every = Duration::from_millis(discovery.refresh_ms);
    let limit = Duration::from_millis(discovery.timeout_ms);
    loop {
        let started = Instant::now();
        for (i, peer) in peers.configured.iter().enumerate() {
            // A task of its own, which the set aborts when this one is.
            let mut fetching = JoinSet::new();
            let (to, tls, own) = (peer.clone(), Arc::clone(&tls), own.clone());
            fetching.spawn(async move { timeout(limit, fetch(&to, tls, &own)).await });
            match fetching.join_next().await {
                Some(Ok(Ok(Ok(kept)))) => peers.keep(i, kept),
                Some(Ok(Ok(Err(why)))) => failed(peer, &why.0),
                Some(Ok(Err(_))) => failed(peer, &format!("no card within {limit:?}")),
                // A panic: graviola's on a processor without the extensions it needs, likely.
                Some(Err(e)) => failed(
                    peer,
                    &format!(
                        "the fetch failed: {e} (the TLS client needs x86_64 or aarch64 with their \
                         crypto extensions)"
                    ),
                ),
                // The set held the fetch.
                None => {}
            }
        }
        peers.prune(wall_ms());
        peers.rounds.send_modify(|done| *done += 1);
        sleep_until(started + every).await;
    }
}

/// Logs a fetch of `peer`'s card that failed, and why.
fn failed(peer: &Configured, why: &str) {
    eprintln!(
        "h3-server: peer {}: {why}; its last good card is relayed until it expires",
        peer.address
    );
}

/// `peer`'s card: dials its address through `tls`, says hello as `own`, checks that the welcome
/// is signed by the peer's key and names the address dialed as its id, asks `Describe` and checks
/// that the card holds and is the key's ([`Client::describe`]).
async fn fetch(peer: &Configured, tls: Arc<ClientConfig>, own: &str) -> Result<Kept, Error> {
    let mut client = Client::dial(&peer.address, tls, (FRAME, FETCH)).await?;
    let welcome = client.hello(own).await?;
    let shown = match &welcome {
        Some(ServerMessage::Welcome { .. }) => None,
        Some(ServerMessage::Goodbye { reason }) => Some(format!("goodbye {reason:?}")),
        Some(_) => Some(String::from("another message")),
        None => Some(String::from("the connection closed")),
    };
    if let Some(shown) = shown {
        return Err(Error(format!("no welcome: {shown}")));
    }
    if client.server_key() != Some(peer.key) {
        let key = client.server_key().map(|key| account::to_hex(&key));
        return Err(Error(format!(
            "its key is {}, not the configured one",
            key.unwrap_or_default()
        )));
    }
    // The key's holder answered at the address dialed (ADR-0021 point 1).
    let id = client
        .welcome()
        .map(|(id, _)| id.to_owned())
        .unwrap_or_default();
    if id != peer.address {
        let id: String = id.chars().take(100).collect();
        return Err(Error(format!(
            "it names itself {id:?}, not the address dialed"
        )));
    }
    let (card, signed) = client.describe(1).await?;
    client.close().await;
    Ok(Kept {
        signed,
        expires: card.expires.0,
    })
}

/// The card a fetch gets of `peer` through `tls` now, or why not (the tests: what a round logs).
#[cfg(test)]
pub(crate) async fn fetch_now(peer: &Peer, tls: Arc<ClientConfig>) -> Result<SignedCard, Error> {
    let peers = Peers::new(std::slice::from_ref(peer), [0; KEY_BYTES])?;
    let configured = (peers.configured.first()).ok_or_else(|| Error(String::from("no peer")))?;
    let kept = fetch(configured, tls, "ws://test.invalid").await?;
    Ok(kept.signed)
}

/// The TLS client's configuration (ADR-0021 point 4): graviola's cryptography, the certificates
/// checked against `roots`.
pub(crate) fn tls(roots: RootCertStore) -> Result<Arc<ClientConfig>, Error> {
    let provider = Arc::new(rustls_graviola::default_provider());
    let config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|e| Error(format!("the TLS client: {e}")))?
        .with_root_certificates(roots)
        .with_no_client_auth();
    Ok(Arc::new(config))
}

/// The operating system's root certificates (`rustls-native-certs`; those the environment's
/// `SSL_CERT_FILE` or `SSL_CERT_DIR` name instead, when set), which the TLS client checks a
/// `wss://` peer's certificate against. What it could not read is logged; without any root, no
/// `wss://` fetch succeeds.
pub(crate) fn os_roots() -> RootCertStore {
    let found = rustls_native_certs::load_native_certs();
    for error in &found.errors {
        eprintln!("h3-server: the operating system's root certificates: {error}");
    }
    let mut roots = RootCertStore::empty();
    let (added, ignored) = roots.add_parsable_certificates(found.certs);
    if added == 0 {
        eprintln!(
            "h3-server: no root certificate from the operating system: no wss:// peer's card \
             can be fetched"
        );
    } else if ignored > 0 {
        eprintln!("h3-server: {ignored} of the operating system's root certificates ignored");
    }
    roots
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Peers of the keys 01…01 to `n`…`n`, the `i`th with a card that expires at 1000 + `i`.
    fn kept(n: u8) -> Peers {
        let configured: Vec<Peer> = (1..=n)
            .map(|i| Peer {
                key: format!("{i:02x}").repeat(32),
                address: format!("ws://p{i}"),
            })
            .collect();
        let peers = Peers::new(&configured, [0; KEY_BYTES]).unwrap();
        for i in 0..usize::from(n) {
            let signed = SignedCard {
                card: format!("card {i}"),
                signature: String::new(),
            };
            let expires = 1000 + u64::try_from(i).unwrap();
            peers.keep(i, Kept { signed, expires });
        }
        peers
    }

    /// The cards a page relays.
    fn page(peers: &Peers, from: u32, now: u64) -> (u32, Vec<String>) {
        let ServerMessage::Peers {
            seq: 7,
            from: answered,
            total,
            entries,
        } = peers.page(7, from, now)
        else {
            panic!("not a page")
        };
        assert_eq!(answered, from);
        assert!(entries.iter().all(|e| e.via.is_empty() && e.list.is_none()));
        (total, entries.into_iter().map(|e| e.card.card).collect())
    }

    #[test]
    fn the_cards_are_relayed_in_pages_in_the_configurations_order() {
        let peers = kept(60);
        let (total, first) = page(&peers, 0, 0);
        assert_eq!((total, first.len()), (60, PAGE));
        assert_eq!(
            (first[0].as_str(), first[49].as_str()),
            ("card 0", "card 49")
        );
        assert_eq!(
            page(&peers, 50, 0).1,
            (50..60).map(|i| format!("card {i}")).collect::<Vec<_>>()
        );
        assert_eq!(page(&peers, 60, 0), (60, vec![]));
        assert_eq!(page(&peers, u32::MAX, 0), (60, vec![]));
        // A peer without a card yet is left out.
        let some = Peers::new(
            &[Peer {
                key: "aa".repeat(32),
                address: String::from("ws://a"),
            }],
            [0; KEY_BYTES],
        )
        .unwrap();
        assert_eq!(page(&some, 0, 0), (0, vec![]));
    }

    #[test]
    fn a_card_is_relayed_until_it_expires() {
        let peers = kept(60);
        // Expired at 1030: the cards of 0 to 30.
        let (total, first) = page(&peers, 0, 1030);
        assert_eq!((total, first[0].as_str()), (29, "card 31"));
        peers.prune(1030);
        assert_eq!(page(&peers, 0, 0).0, 29);
        assert!(peers.cards().iter().take(31).all(Option::is_none));
    }

    #[test]
    fn a_peer_is_another_servers_key() {
        let peer = |key: &str| Peer {
            key: key.to_owned(),
            address: String::from("ws://a"),
        };
        let own = [0xaa; KEY_BYTES];
        let error = Peers::new(&[peer(&"bb".repeat(32)), peer(&"aa".repeat(32))], own).unwrap_err();
        assert!(error.0.contains("peer 1: the server's own key"), "{error}");
        let error = Peers::new(&[peer("bb")], own).unwrap_err();
        assert!(error.0.contains("peer 0: not a key"), "{error}");
    }
}
