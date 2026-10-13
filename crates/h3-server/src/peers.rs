//! The server's peers (E16 M6 T3, T4; ADR-0021 point 3, `E16-design.md` §12): the servers its
//! operator vouches for (`[[peer]]`), whose cards it fetches and relays with `ListPeers`, and the
//! cards they relay in turn, each with the chain of keys that vouches for it.
//!
//! One task fetches the peers' cards at start and every `[discovery] refresh_ms`, one peer after
//! the other in the configuration's order. A fetch dials the peer's address as a client does
//! (`wss://` through the TLS client, `ws://` plain), says hello, checks the welcome (signed by the
//! configured key, naming the address dialed as the peer's id: the key's holder answered there),
//! and asks `Describe`: the card must hold ([`card::verify`]) and be the configured key's. Below
//! `[discovery] depth` hops it then reads the cards the peer relays (`ListPeers`), at most
//! `[discovery] cards` of them, and keeps those this server may relay on ([`vouched`]): a card
//! whose chain stays within `depth`, that holds, and whose chain holds from the peer's card
//! through the cards its `via` names (which the peer relays too) to it; a card that does not hold
//! or whose chain does not is dropped and logged. A fetch reads at most [`FRAME`] bytes a frame and
//! [`budget`] in all, within `[discovery] timeout_ms`. A peer that fails is logged and keeps its
//! last good card and relays until they expire. The server dials only the addresses its operator
//! configured, never one it read in a card.
//!
//! Lists of servers (E16 M6 T5, [`crate::lists`]): each round first reads the lists in
//! `<data>/lists/`; after the peers it fetches the servers they name as it fetches a peer, without
//! what they relay (depth 1): each server once (the first list's), neither the server itself nor
//! a peer, at most `[discovery] cards` less the peers. Their cards are relayed marked with their
//! list's key, which a client checks against the list (`Lists`, `h3_protocol::list::names`).
//!
//! What `ListPeers` pages ([`Peers::page`]) is rebuilt after each fetch and each round
//! ([`relay`]): the peers' cards in the configuration's order, then the listed servers' in the
//! lists' order, then the cards the peers relay, the nearer hops first, the peers in the
//! configuration's order and each peer's cards in its own; each key once, by its shortest chain;
//! each chain checked again against the cards relayed before it, so that a client finds every card
//! of a chain among them and checks it as the server did (`h3_protocol::card::chain`); at most
//! `[discovery] cards`, never the server's own card. A card is relayed until the first card of its
//! chain (a listed server's: or its list) expires.
//!
//! The TLS client (ADR-0021 point 4): `rustls` with the pure-Rust graviola backend, certificates
//! checked against the operating system's roots, read once at start when a peer is at `wss://`.
//! Graviola runs on x86_64 and aarch64 with their crypto extensions only, and panics elsewhere at
//! its first use: each fetch runs as a task of its own, so such a panic fails the fetch, logged,
//! not the server.
//!
//! Provisional (E16 M6 T3–T5): the frame bounds, the hello's name (the server's id), the log
//! lines, one fetch at a time, a card a peer relays for its list not relayed on, the listed
//! servers' place after the peers' cards and before what the peers relay.

use std::collections::BTreeMap;
use std::iter;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use h3_protocol::account::{self, KEY_BYTES};
use h3_protocol::card::{MAX_CARD_BYTES, PAGE};
use h3_protocol::wire::{RelayedCard, ServerCard, ServerMessage, SignedCard};
use rustls::{ClientConfig, RootCertStore};
use tokio::sync::watch;
use tokio::task::JoinSet;
use tokio::time::{Instant, sleep_until, timeout};

use crate::Error;
use crate::card::{self, wall_ms};
use crate::client::Client;
use crate::config::{Discovery, Peer};
use crate::lists::{self, Held};

/// The largest frame of one card a fetch reads (a `Described`, or a card on a page of `Peers`):
/// its text, escaped in the frame, takes at most twice its bytes; with room for its signature, a
/// `via`, a list's key and the rest of the message.
const CARD_FRAME: usize = 2 * MAX_CARD_BYTES + 1024;

/// The largest frame a fetch reads: a page of `Peers` (E16 M6 T4), into which a `Described` fits.
const FRAME: usize = PAGE * CARD_FRAME + 1024;

/// The bytes a fetch that reads at most `cards` of the cards a peer relays reads in all: its
/// welcome and the peer's card, the pages, with room for pings.
fn budget(cards: usize) -> usize {
    2 * CARD_FRAME + cards.div_ceil(PAGE) * FRAME
}

/// A configured peer.
#[derive(Debug, Clone)]
struct Configured {
    /// Its key.
    key: [u8; KEY_BYTES],
    /// Where its card is fetched.
    address: String,
}

/// A card that held when it was fetched: as its key signed it, and what a chain of vouching reads
/// of it.
#[derive(Debug)]
struct Kept {
    /// As its key signed it.
    signed: SignedCard,
    /// Its key, as the card names it (lowercase hex).
    key: String,
    /// The keys of the servers it vouches for: its card's peers.
    vouches: Vec<String>,
    /// When it expires, in milliseconds since the Unix epoch.
    expires: u64,
}

impl Kept {
    /// `card`, which `signed` holds.
    fn new(card: ServerCard, signed: SignedCard) -> Arc<Self> {
        Arc::new(Self {
            signed,
            key: card.key,
            vouches: card.peers.into_iter().map(|peer| peer.key).collect(),
            expires: card.expires.0,
        })
    }
}

/// A card a peer relays that the server may relay on, and the keys from the peer's own peer down
/// to its voucher (its `via`, as the peer relayed it).
type Relay = (Arc<Kept>, Vec<String>);

/// What the last good fetch of a peer got.
#[derive(Debug)]
struct Fetched {
    /// The peer's card.
    card: Arc<Kept>,
    /// The cards it relays that the server may relay on, in its order ([`vouched`]).
    relays: Vec<Relay>,
}

/// A server a list names that the server fetches (E16 M6 T5).
#[derive(Debug)]
struct Listed {
    /// The list's key, which marks the server's card.
    list: String,
    /// When the list expires.
    until: u64,
    /// The server, as the list names it.
    server: Configured,
    /// Its last good card.
    card: Option<Arc<Kept>>,
}

/// A card the server relays.
#[derive(Debug, Clone)]
struct Relayed {
    /// The card.
    kept: Arc<Kept>,
    /// The keys from the server's peer down to the card's voucher (`RelayedCard::via`): none for a
    /// peer's own card and a listed server's.
    via: Vec<String>,
    /// The key of the list that names it, for a listed server's card (`RelayedCard::list`).
    list: Option<String>,
    /// When the first card of its chain (or its list) expires: it is relayed until then.
    until: u64,
}

/// The peers' last good fetches, the lists and their servers', and the cards relayed of them.
#[derive(Debug)]
struct State {
    /// Each peer's last good fetch, in the configuration's order.
    fetched: Vec<Option<Fetched>>,
    /// The lists that held when last read (E16 M6 T5).
    lists: Vec<Held>,
    /// The servers they name that the server fetches, in the lists' order.
    listed: Vec<Listed>,
    /// The cards relayed, in their order ([`relay`]).
    relayed: Vec<Relayed>,
}

/// The peers, their last good cards and the cards relayed of them.
#[derive(Debug)]
pub(crate) struct Peers {
    /// The server's own key (lowercase hex), whose card it never relays.
    own: String,
    /// The peers, in the configuration's order.
    configured: Vec<Configured>,
    /// How many hops of vouching the cards relayed reach (`[discovery] depth`).
    depth: usize,
    /// The most cards relayed (`[discovery] cards`).
    cards: usize,
    /// The fetches and the cards relayed.
    state: Mutex<State>,
    /// The rounds of fetches done.
    rounds: watch::Sender<u64>,
}

impl Peers {
    /// The peers `peers` (`[[peer]]`) of the server of key `own`, none fetched yet, relaying as
    /// `discovery` bounds it (`depth`, `cards`); why not, if a key is not 32 bytes of hex or is
    /// `own`.
    pub(crate) fn new(
        peers: &[Peer],
        discovery: &Discovery,
        own: [u8; KEY_BYTES],
    ) -> Result<Self, Error> {
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
        let state = State {
            fetched: iter::repeat_with(|| None).take(configured.len()).collect(),
            lists: Vec::new(),
            listed: Vec::new(),
            relayed: Vec::new(),
        };
        Ok(Self {
            own: account::to_hex(&own),
            configured,
            depth: discovery.depth,
            cards: discovery.cards,
            state: Mutex::new(state),
            rounds: watch::Sender::new(0),
        })
    }

    /// The fetches and the cards relayed, locked.
    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The answer to `ListPeers { seq, from }` at `now` (milliseconds since the Unix epoch): the
    /// cards relayed whose chains hold then, from the `from`th, at most a page of them.
    pub(crate) fn page(&self, seq: u32, from: u32, now: u64) -> ServerMessage {
        let state = self.state();
        let held: Vec<&Relayed> = (state.relayed.iter())
            .filter(|relayed| relayed.until > now)
            .collect();
        let entries = (held.iter())
            .skip(usize::try_from(from).unwrap_or(usize::MAX))
            .take(PAGE)
            .map(|relayed| RelayedCard {
                card: relayed.kept.signed.clone(),
                via: relayed.via.clone(),
                list: relayed.list.clone(),
            })
            .collect();
        ServerMessage::Peers {
            seq,
            from,
            total: u32::try_from(held.len()).unwrap_or(u32::MAX),
            entries,
        }
    }

    /// The answer to `Lists { seq }` at `now` (E16 M6 T5): the lists that hold then, as signed.
    pub(crate) fn lists(&self, seq: u32, now: u64) -> ServerMessage {
        let state = self.state();
        let lists = (state.lists.iter())
            .filter(|held| held.list.expires.0 > now)
            .map(|held| held.signed.clone())
            .collect();
        ServerMessage::ListsAre { seq, lists }
    }

    /// The cards relayed rebuilt at `now`.
    fn rebuild(&self, state: &mut State, now: u64) {
        let reach = (self.depth, self.cards);
        state.relayed = relay(&state.fetched, &state.listed, &self.own, reach, now);
    }

    /// The `i`th peer's new fetch, at `now`.
    fn keep(&self, i: usize, fetched: Fetched, now: u64) {
        let mut state = self.state();
        if let Some(slot) = state.fetched.get_mut(i) {
            *slot = Some(fetched);
        }
        self.rebuild(&mut state, now);
    }

    /// The lists `lists`, read at `now` (E16 M6 T5), and the servers the next fetches reach of
    /// them: each server once (the first list's), neither the server itself nor a peer, at most
    /// `cards` less the peers, in the lists' order. A server named again at the same address keeps
    /// its last good card.
    fn list(&self, lists: Vec<Held>, now: u64) {
        let mut state = self.state();
        let mut before = std::mem::take(&mut state.listed);
        let room = self.cards.saturating_sub(self.configured.len());
        let mut listed: Vec<Listed> = Vec::new();
        for held in &lists {
            let servers = (held.list.servers.iter())
                .filter_map(|server| Some((account::from_hex(&server.key)?, &server.address)));
            for (key, address) in servers {
                let peer = (self.configured.iter()).any(|peer| peer.key == key);
                let taken = listed.iter().any(|l| l.server.key == key);
                if peer || taken || account::to_hex(&key) == self.own || listed.len() >= room {
                    continue;
                }
                let card = (before.iter_mut())
                    .find(|l| l.server.key == key && l.server.address == *address)
                    .and_then(|l| l.card.take());
                listed.push(Listed {
                    list: held.list.key.clone(),
                    until: held.list.expires.0,
                    server: Configured {
                        key,
                        address: address.clone(),
                    },
                    card,
                });
            }
        }
        state.listed = listed;
        state.lists = lists;
        self.rebuild(&mut state, now);
    }

    /// The servers the lists name that the next fetches reach, in order.
    fn listed(&self) -> Vec<Configured> {
        (self.state().listed.iter())
            .map(|listed| listed.server.clone())
            .collect()
    }

    /// The new card of `server`, the `j`th listed server, at `now`.
    fn keep_listed(&self, j: usize, server: &Configured, card: Arc<Kept>, now: u64) {
        let mut state = self.state();
        let slot = state.listed.get_mut(j);
        if let Some(listed) = slot.filter(|l| l.server.key == server.key) {
            listed.card = Some(card);
        }
        self.rebuild(&mut state, now);
    }

    /// Drops the cards expired at `now`.
    fn prune(&self, now: u64) {
        let mut state = self.state();
        for slot in state.fetched.iter_mut() {
            if slot
                .as_ref()
                .is_some_and(|fetched| fetched.card.expires <= now)
            {
                *slot = None;
            }
            if let Some(fetched) = slot {
                fetched.relays.retain(|(kept, _)| kept.expires > now);
            }
        }
        for listed in state.listed.iter_mut() {
            if listed.card.as_ref().is_some_and(|card| card.expires <= now) {
                listed.card = None;
            }
        }
        self.rebuild(&mut state, now);
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

/// A card offered to [`relay`]: the card, the keys of its chain's links, and its list's key and
/// expiry for a listed server's.
type Offered<'a> = (&'a Arc<Kept>, Vec<&'a String>, Option<(&'a String, u64)>);

/// The cards the server of key `own` relays at `now` of its peers' fetches `fetched` (in the
/// configuration's order) and of the servers its lists name, `listed`: the peers' own cards, the
/// listed servers' (marked with their list's key), then the cards the peers relay, those with a
/// `via` of no key first, then of one, and so on below `depth` hops, the peers in order and each
/// peer's cards in its own; each key once (by its shortest chain), `own` never; at most `cards`. A
/// card the peer `P` relays with `via: [v…]` is relayed with `via: [P, v…]` if the chain holds
/// through the cards relayed before it: `P`'s card (which the server vouches for), each card of
/// `v…` vouching for the next and the last (or `P`'s, without `v…`) for the card. It is relayed
/// until the first card of that chain expires; a listed server's, until it or its list does.
fn relay(
    fetched: &[Option<Fetched>],
    listed: &[Listed],
    own: &str,
    (depth, cards): (usize, usize),
    now: u64,
) -> Vec<Relayed> {
    let peers: Vec<&Fetched> = fetched.iter().flatten().collect();
    // Each card offered: the peers' own cards first, then the listed servers'.
    let mut offered: Vec<Offered<'_>> = (peers.iter())
        .map(|peer| (&peer.card, Vec::new(), None))
        .collect();
    for listed in listed {
        if let Some(card) = &listed.card {
            offered.push((card, Vec::new(), Some((&listed.list, listed.until))));
        }
    }
    for links in 0..depth.saturating_sub(1) {
        for peer in &peers {
            let relays = (peer.relays.iter()).filter(|(_, via)| via.len() == links);
            for (kept, via) in relays {
                let links = iter::once(&peer.card.key).chain(via).collect();
                offered.push((kept, links, None));
            }
        }
    }
    let mut relayed = Vec::new();
    // Each key relayed: its card, and until when it is relayed.
    let mut at: BTreeMap<&str, (&Kept, u64)> = BTreeMap::new();
    for (kept, links, list) in offered {
        if relayed.len() >= cards {
            break;
        }
        if kept.expires <= now || kept.key == own || at.contains_key(kept.key.as_str()) {
            continue;
        }
        let Some(until) = through(&at, &links, kept) else {
            continue;
        };
        let until = list.map_or(until, |(_, expires)| until.min(expires));
        if until <= now {
            continue;
        }
        at.insert(&kept.key, (kept, until));
        relayed.push(Relayed {
            kept: Arc::clone(kept),
            via: links.into_iter().cloned().collect(),
            list: list.map(|(key, _)| key.clone()),
            until,
        });
    }
    relayed
}

/// Until when the chain to `card` through the cards of `links`, relayed already (`at`), holds:
/// from the server (which vouches for its peers, and the first link is one) through each link's
/// card vouching for the next to the last's vouching for `card`. `None` if a link is not relayed
/// or does not vouch for the next.
fn through(at: &BTreeMap<&str, (&Kept, u64)>, links: &[&String], card: &Kept) -> Option<u64> {
    let mut until = card.expires;
    let mut voucher: Option<&Kept> = None;
    for &link in links {
        let &(kept, relayed) = at.get(link.as_str())?;
        if voucher.is_some_and(|voucher| !voucher.vouches.contains(link)) {
            return None;
        }
        voucher = Some(kept);
        until = until.min(relayed);
    }
    (voucher.is_none_or(|voucher| voucher.vouches.contains(&card.key))).then_some(until)
}

/// The cards the peer of card `peer` relays (`relayed`) that the server may relay on at `now`,
/// each with its `via`, and why each other card within `depth` was dropped: a card relayed for a
/// list (E16 M6 T5) is not; one whose `via` holds `depth − 1` keys or more is beyond `depth` (the
/// server's chain to it adds the peer); a card within `depth` must hold ([`card::verify`]), and
/// the chain from `peer` through the cards its `via` names, which the peer relays too, to it must
/// hold (`h3_protocol::card::chain`, as a client checks it).
fn vouched(
    peer: &ServerCard,
    relayed: Vec<RelayedCard>,
    depth: usize,
    now: u64,
) -> (Vec<Relay>, Vec<String>) {
    let mut dropped = Vec::new();
    let within =
        (relayed.into_iter()).filter(|entry| entry.list.is_none() && entry.via.len() + 1 < depth);
    let read: Vec<(ServerCard, RelayedCard)> = within
        .filter_map(|entry| match card::verify(&entry.card, now) {
            Ok(card) => Some((card, entry)),
            Err(why) => {
                dropped.push(format!(
                    "a card it relays is dropped: it does not hold ({why:?})"
                ));
                None
            }
        })
        .collect();
    // The cards by key, the first of a key.
    let mut by_key: BTreeMap<&str, &ServerCard> = BTreeMap::new();
    for (card, _) in &read {
        by_key.entry(card.key.as_str()).or_insert(card);
    }
    let holds: Vec<bool> = (read.iter())
        .map(|(card, entry)| {
            let links: Option<Vec<&ServerCard>> = (entry.via.iter())
                .map(|key| by_key.get(key.as_str()).copied())
                .collect();
            links.is_some_and(|links| {
                let chain: Vec<&ServerCard> = (iter::once(peer).chain(links))
                    .chain(iter::once(card))
                    .collect();
                h3_protocol::card::chain(&chain)
            })
        })
        .collect();
    let mut relays = Vec::new();
    for ((card, entry), holds) in read.into_iter().zip(holds) {
        if holds {
            relays.push((Kept::new(card, entry.card), entry.via));
        } else {
            dropped.push(format!(
                "a card it relays is dropped: the chain to {} does not hold",
                card.key
            ));
        }
    }
    (relays, dropped)
}

/// Fetches the peers' cards at start and every `discovery.refresh_ms` (from one round's start to
/// the next's), each within `discovery.timeout_ms`, until the task is aborted; `own` is the
/// server's id, its hello's name. Each round first reads the lists in the directory `dir`
/// (`<data>/lists/`, E16 M6 T5) and fetches the servers they name after the peers. A fetch in
/// flight ends with the task.
pub(crate) async fn fetch_every(
    peers: Arc<Peers>,
    tls: Arc<ClientConfig>,
    discovery: Discovery,
    own: String,
    dir: PathBuf,
) {
    let every = Duration::from_millis(discovery.refresh_ms);
    let limit = Duration::from_millis(discovery.timeout_ms);
    let reach = (peers.depth, peers.cards);
    loop {
        let started = Instant::now();
        peers.list(lists::read(&dir, wall_ms()), wall_ms());
        for (i, peer) in peers.configured.iter().enumerate() {
            let what = format!("peer {}", peer.address);
            if let Some(fetched) = fetch_logged(peer, &what, (&tls, &own), reach, limit).await {
                peers.keep(i, fetched, wall_ms());
            }
        }
        // The listed servers' own cards only (depth 1).
        for (j, server) in peers.listed().iter().enumerate() {
            let what = format!("listed server {}", server.address);
            if let Some(fetched) = fetch_logged(server, &what, (&tls, &own), (1, 0), limit).await {
                peers.keep_listed(j, server, fetched.card, wall_ms());
            }
        }
        peers.prune(wall_ms());
        peers.rounds.send_modify(|done| *done += 1);
        sleep_until(started + every).await;
    }
}

/// A fetch of `to` (named `what` in the log) through the TLS client and as the server's id, `(tls,
/// own)`, reaching `reach` within `limit`, as a task of its own, which the set aborts when the
/// caller's is: what it got, each card dropped logged; `None` once its failure is logged.
async fn fetch_logged(
    to: &Configured,
    what: &str,
    (tls, own): (&Arc<ClientConfig>, &str),
    reach: (usize, usize),
    limit: Duration,
) -> Option<Fetched> {
    let mut fetching = JoinSet::new();
    let (target, tls, own) = (to.clone(), Arc::clone(tls), own.to_owned());
    fetching.spawn(async move { timeout(limit, fetch(&target, tls, &own, reach)).await });
    let why = match fetching.join_next().await? {
        Ok(Ok(Ok((fetched, dropped)))) => {
            for why in &dropped {
                eprintln!("h3-server: {what}: {why}");
            }
            return Some(fetched);
        }
        Ok(Ok(Err(why))) => why.0,
        Ok(Err(_)) => format!("no card within {limit:?}"),
        // A panic: graviola's on a processor without the extensions it needs, likely.
        Err(e) => format!(
            "the fetch failed: {e} (the TLS client needs x86_64 or aarch64 with their crypto \
             extensions)"
        ),
    };
    eprintln!("h3-server: {what}: {why}; what it last gave is relayed until it expires");
    None
}

/// What a fetch of `peer` gets, and why each card it relays and the server does not relay on was
/// dropped ([`vouched`]): dials its address through `tls`, says hello as `own`, checks that the
/// welcome is signed by the peer's key and names the address dialed as its id, asks `Describe` and
/// checks that the card holds and is the key's ([`Client::describe`]); below `depth`, reads at most
/// `cards` of the cards it relays ([`listing`]).
async fn fetch(
    peer: &Configured,
    tls: Arc<ClientConfig>,
    own: &str,
    (depth, cards): (usize, usize),
) -> Result<(Fetched, Vec<String>), Error> {
    let reads = if depth > 1 { cards } else { 0 };
    let mut client = Client::dial(&peer.address, tls, (FRAME, budget(reads))).await?;
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
    let listed = listing(&mut client, reads).await?;
    client.close().await;
    let (relays, dropped) = vouched(&card, listed, depth, wall_ms());
    let card = Kept::new(card, signed);
    Ok((Fetched { card, relays }, dropped))
}

/// At most `cards` of the cards the server of `client` relays, page by page, as they came.
async fn listing(client: &mut Client, cards: usize) -> Result<Vec<RelayedCard>, Error> {
    let mut listed = Vec::new();
    while listed.len() < cards {
        let from = u32::try_from(listed.len()).unwrap_or(u32::MAX);
        let (total, page) = client.list_peers(from, from).await?;
        let last = page.is_empty();
        listed.extend(page);
        if last || listed.len() >= usize::try_from(total).unwrap_or(usize::MAX) {
            break;
        }
    }
    listed.truncate(cards);
    Ok(listed)
}

/// The card a fetch gets of `peer` through `tls` now, and why each card it relays and the server
/// would not relay on was dropped, at the deepest `depth`; or why the fetch failed (the tests: what
/// a round logs).
#[cfg(test)]
pub(crate) async fn fetch_now(
    peer: &Peer,
    tls: Arc<ClientConfig>,
) -> Result<(SignedCard, Vec<String>), Error> {
    let discovery = Discovery {
        depth: 4,
        ..Discovery::default()
    };
    let peers = Peers::new(std::slice::from_ref(peer), &discovery, [0; KEY_BYTES])?;
    let configured = (peers.configured.first()).ok_or_else(|| Error(String::from("no peer")))?;
    let reach = (discovery.depth, discovery.cards);
    let (fetched, dropped) = fetch(configured, tls, "ws://test.invalid", reach).await?;
    Ok((fetched.card.signed.clone(), dropped))
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
    use h3_protocol::connection::Signer as _;
    use h3_protocol::wire::{CardPeer, WireU64};

    use super::*;
    use crate::key::ServerKey;

    /// The key `n`…`n` in hex.
    fn key(n: u8) -> String {
        format!("{n:02x}").repeat(32)
    }

    /// A card of the key `n`…`n` vouching for `vouches`, expiring at `expires`.
    fn kept(n: u8, vouches: &[u8], expires: u64) -> Arc<Kept> {
        Arc::new(Kept {
            signed: SignedCard {
                card: format!("card {n}"),
                signature: String::new(),
            },
            key: key(n),
            vouches: vouches.iter().map(|&v| key(v)).collect(),
            expires,
        })
    }

    /// A fetch of the card `card` relaying `relays`, each with its `via`.
    fn fetched(card: Arc<Kept>, relays: &[(Arc<Kept>, &[u8])]) -> Option<Fetched> {
        let relays = (relays.iter())
            .map(|(kept, via)| (Arc::clone(kept), via.iter().map(|&v| key(v)).collect()))
            .collect();
        Some(Fetched { card, relays })
    }

    /// What `relay` relays at `now` of `fetched` for the server of key 00…00, as `(card, via)`.
    fn relayed(fetched: &[Option<Fetched>], reach: (usize, usize), now: u64) -> Vec<String> {
        (relay(fetched, &[], &key(0), reach, now).iter())
            .map(|relayed| {
                let via: Vec<String> = (relayed.via.iter()).map(|v| v[..2].to_owned()).collect();
                format!("{} [{}]", relayed.kept.signed.card, via.join(" "))
            })
            .collect()
    }

    /// The peers 1 to `n`, the `i`th with a card that expires at 1000 + `i`.
    fn peers(n: u8) -> Peers {
        let configured: Vec<Peer> = (1..=n)
            .map(|i| Peer {
                key: key(i),
                address: format!("ws://p{i}"),
            })
            .collect();
        let peers = Peers::new(&configured, &Discovery::default(), [0; KEY_BYTES]).unwrap();
        for i in 0..n {
            let card = kept(i + 1, &[], 1000 + u64::from(i));
            peers.keep(usize::from(i), fetched(card, &[]).unwrap(), 0);
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
        assert!(entries.iter().all(|e| e.list.is_none()));
        (total, entries.into_iter().map(|e| e.card.card).collect())
    }

    #[test]
    fn the_cards_are_relayed_in_pages_in_the_configurations_order() {
        let peers = peers(60);
        let (total, first) = page(&peers, 0, 0);
        assert_eq!((total, first.len()), (60, PAGE));
        assert_eq!(
            (first[0].as_str(), first[49].as_str()),
            ("card 1", "card 50")
        );
        assert_eq!(
            page(&peers, 50, 0).1,
            (51..=60).map(|i| format!("card {i}")).collect::<Vec<_>>()
        );
        assert_eq!(page(&peers, 60, 0), (60, vec![]));
        assert_eq!(page(&peers, u32::MAX, 0), (60, vec![]));
        // A peer without a card yet is left out.
        let some = Peer {
            key: "aa".repeat(32),
            address: String::from("ws://a"),
        };
        let some = Peers::new(&[some], &Discovery::default(), [0; KEY_BYTES]).unwrap();
        assert_eq!(page(&some, 0, 0), (0, vec![]));
    }

    #[test]
    fn a_card_is_relayed_until_it_expires() {
        let peers = peers(60);
        // Expired at 1030: the cards of 1 to 31.
        let (total, first) = page(&peers, 0, 1030);
        assert_eq!((total, first[0].as_str()), (29, "card 32"));
        peers.prune(1030);
        assert_eq!(page(&peers, 0, 0).0, 29);
        assert!(peers.state().fetched.iter().take(31).all(Option::is_none));
    }

    #[test]
    fn a_peer_is_another_servers_key() {
        let peer = |key: &str| Peer {
            key: key.to_owned(),
            address: String::from("ws://a"),
        };
        let own = [0xaa; KEY_BYTES];
        let discovery = Discovery::default();
        let both = [peer(&"bb".repeat(32)), peer(&"aa".repeat(32))];
        let error = Peers::new(&both, &discovery, own).unwrap_err();
        assert!(error.0.contains("peer 1: the server's own key"), "{error}");
        let error = Peers::new(&[peer("bb")], &discovery, own).unwrap_err();
        assert!(error.0.contains("peer 0: not a key"), "{error}");
    }

    /// Peers 1 and 2; 1 vouches for 3, which vouches for 5, which vouches for 7; 2 vouches for 4,
    /// which vouches for 6. Each relays what it vouches for, and what those relay.
    fn web() -> Vec<Option<Fetched>> {
        let (c3, c5, c7) = (kept(3, &[5], 9), kept(5, &[7], 9), kept(7, &[], 9));
        let (c4, c6) = (kept(4, &[6], 9), kept(6, &[], 9));
        vec![
            fetched(kept(1, &[3], 9), &[(c3, &[]), (c5, &[3]), (c7, &[3, 5])]),
            fetched(kept(2, &[4], 9), &[(c4, &[]), (c6, &[4])]),
        ]
    }

    #[test]
    fn the_cards_relayed_reach_depth_hops_the_nearer_first() {
        let web = web();
        assert_eq!(relayed(&web, (1, 200), 0), ["card 1 []", "card 2 []"]);
        assert_eq!(
            relayed(&web, (2, 200), 0),
            ["card 1 []", "card 2 []", "card 3 [01]", "card 4 [02]"]
        );
        assert_eq!(
            relayed(&web, (3, 200), 0)[4..],
            ["card 5 [01 03]", "card 6 [02 04]"]
        );
        assert_eq!(relayed(&web, (4, 200), 0)[6..], ["card 7 [01 03 05]"]);
        // At most `cards`, the nearer first.
        assert_eq!(
            relayed(&web, (4, 3), 0),
            ["card 1 []", "card 2 []", "card 3 [01]"]
        );
        assert_eq!(relayed(&web, (4, 1), 0), ["card 1 []"]);
    }

    #[test]
    fn a_key_is_relayed_once_by_its_shortest_chain_and_the_servers_own_never() {
        let (c2, c3) = (kept(2, &[3], 9), kept(3, &[], 9));
        let own = kept(0, &[], 9);
        let web = vec![
            // 1 relays 2, a peer itself; the server's own card; 3 through 2 and directly.
            fetched(
                kept(1, &[0, 2, 3], 9),
                &[
                    (c2, &[]),
                    (own, &[]),
                    (Arc::clone(&c3), &[2]),
                    (Arc::clone(&c3), &[]),
                ],
            ),
            fetched(kept(2, &[3], 9), &[(c3, &[])]),
        ];
        assert_eq!(
            relayed(&web, (3, 200), 0),
            ["card 1 []", "card 2 []", "card 3 [01]"]
        );
    }

    #[test]
    fn a_chain_is_checked_against_the_cards_relayed() {
        // 3 is a peer, whose own card vouches for no one; 1 relays an older card of 3 that vouches
        // for 5, and 5 through it. The client would check 5's chain with 3's card relayed: dropped.
        let (old3, c5) = (kept(3, &[5], 9), kept(5, &[], 9));
        let web = vec![
            fetched(kept(1, &[3], 9), &[(old3, &[]), (Arc::clone(&c5), &[3])]),
            fetched(kept(3, &[], 9), &[]),
        ];
        assert_eq!(relayed(&web, (3, 200), 0), ["card 1 []", "card 3 []"]);
        // A card its peer relays, but whose chain does not hold, or through a card not relayed.
        let web = vec![fetched(
            kept(1, &[], 9),
            &[(Arc::clone(&c5), &[]), (Arc::clone(&c5), &[6])],
        )];
        assert_eq!(relayed(&web, (3, 200), 0), ["card 1 []"]);
    }

    #[test]
    fn a_card_is_relayed_until_the_first_card_of_its_chain_expires() {
        let (c3, c5) = (kept(3, &[5], 3000), kept(5, &[], 5000));
        let web = vec![fetched(kept(1, &[3], 2000), &[(c3, &[]), (c5, &[3])])];
        let until: Vec<u64> = (relay(&web, &[], &key(0), (3, 200), 0).iter())
            .map(|relayed| relayed.until)
            .collect();
        assert_eq!(until, [2000, 2000, 2000]);
        // An expired card is not offered, nor what it vouches for.
        let web = vec![fetched(
            kept(1, &[3], 9000),
            &[(kept(3, &[5], 3000), &[]), (kept(5, &[], 5000), &[3])],
        )];
        assert_eq!(relayed(&web, (3, 200), 3000), ["card 1 []"]);
        // Pruned: the fetches lose what expired.
        let peers = Peers::new(
            &[Peer {
                key: key(1),
                address: String::from("ws://p1"),
            }],
            &Discovery::default(),
            [0; KEY_BYTES],
        )
        .unwrap();
        let mut web = web;
        peers.keep(0, web.remove(0).unwrap(), 0);
        assert_eq!(page(&peers, 0, 0).0, 2);
        peers.prune(3000);
        let state = peers.state();
        let fetched = state.fetched[0].as_ref().unwrap();
        assert_eq!(fetched.relays.len(), 1);
        assert_eq!(state.relayed.len(), 1);
    }

    /// A list of the key `list`…`list` naming the servers of the keys `servers`, expiring at
    /// `expires`, as read (its signature is not checked here).
    fn held(list: u8, servers: &[u8], expires: u64) -> Held {
        let list = h3_protocol::wire::ServerList {
            key: key(list),
            name: format!("list {list}"),
            issued: WireU64(0),
            expires: WireU64(expires),
            servers: (servers.iter())
                .map(|&n| h3_protocol::wire::ListedServer {
                    key: key(n),
                    address: format!("ws://s{n}"),
                })
                .collect(),
        };
        let signed = h3_protocol::wire::SignedList {
            list: format!("list {}", list.name),
            signature: String::new(),
        };
        Held { signed, list }
    }

    /// The server `n` named by the list `list`…`list`, expiring at `until`, with the card `card`.
    fn listed(list: u8, n: u8, until: u64, card: Option<Arc<Kept>>) -> Listed {
        let server = Configured {
            key: [n; KEY_BYTES],
            address: format!("ws://s{n}"),
        };
        Listed {
            list: key(list),
            until,
            server,
            card,
        }
    }

    #[test]
    fn a_listed_servers_card_comes_after_the_peers_marked_with_its_list() {
        // The peer 1 vouches for 3 and relays it; the list 9 names 2 and 3.
        let web = vec![fetched(kept(1, &[3], 9000), &[(kept(3, &[], 9000), &[])])];
        let named = [
            listed(9, 2, 5000, Some(kept(2, &[], 9000))),
            listed(9, 3, 5000, Some(kept(3, &[], 9000))),
            listed(9, 4, 5000, None),
        ];
        let shown = |now: u64| -> Vec<(String, usize, Option<String>, u64)> {
            (relay(&web, &named, &key(0), (2, 200), now).iter())
                .map(|r| {
                    let list = r.list.as_ref().map(|l| l[..2].to_owned());
                    (r.kept.signed.card.clone(), r.via.len(), list, r.until)
                })
                .collect()
        };
        let nine = Some(String::from("09"));
        assert_eq!(
            shown(0),
            [
                (String::from("card 1"), 0, None, 9000),
                (String::from("card 2"), 0, nine.clone(), 5000),
                (String::from("card 3"), 0, nine, 5000),
            ]
        );
        // The list expired: its servers' cards go, and 3 is relayed through its peer again.
        let after: Vec<(String, usize)> = (shown(5000).into_iter())
            .map(|(card, via, list, _)| {
                assert!(list.is_none());
                (card, via)
            })
            .collect();
        assert_eq!(
            after,
            [(String::from("card 1"), 0), (String::from("card 3"), 1)]
        );
    }

    #[test]
    fn the_lists_servers_are_each_fetched_once_neither_the_server_nor_a_peer() {
        // The peer 1; at most 4 cards, so 3 listed servers.
        let discovery = Discovery {
            cards: 4,
            ..Discovery::default()
        };
        let peer = Peer {
            key: key(1),
            address: String::from("ws://p1"),
        };
        let peers = Peers::new(&[peer], &discovery, [0; KEY_BYTES]).unwrap();
        // The server itself (0), the peer (1), 2 and 3; then 2 again, 4 and 5.
        let lists = vec![held(8, &[0, 1, 2, 3], 9000), held(9, &[2, 4, 5], 9000)];
        peers.list(lists.clone(), 0);
        let fetched: Vec<String> = (peers.listed().iter())
            .map(|server| server.address.clone())
            .collect();
        assert_eq!(fetched, ["ws://s2", "ws://s3", "ws://s4"]);
        assert_eq!(peers.state().listed[2].list, key(9));
        // A card fetched; the next round's lists keep it for the server at the same address.
        let s2 = peers.listed().remove(0);
        peers.keep_listed(0, &s2, kept(2, &[], 9000), 0);
        let ServerMessage::Peers { entries, .. } = peers.page(1, 0, 0) else {
            panic!("not a page")
        };
        let entries: Vec<(String, Option<String>)> = (entries.into_iter())
            .map(|e| (e.card.card, e.list))
            .collect();
        assert_eq!(entries, [(String::from("card 2"), Some(key(8)))]);
        peers.list(lists, 0);
        assert!(peers.state().listed[0].card.is_some());
        // Another address for it: a new server to fetch, no card yet.
        let mut moved = held(8, &[2], 9000);
        moved.list.servers[0].address = String::from("ws://elsewhere");
        peers.list(vec![moved], 0);
        assert!(peers.state().listed[0].card.is_none());
        // A card kept for a server no longer there in that place is not kept.
        peers.keep_listed(1, &s2, kept(2, &[], 9000), 0);
        assert!(peers.state().listed.iter().all(|l| l.card.is_none()));
    }

    #[test]
    fn lists_gives_the_lists_that_hold() {
        let peers = Peers::new(&[], &Discovery::default(), [0; KEY_BYTES]).unwrap();
        peers.list(vec![held(8, &[], 1000), held(9, &[], 3000)], 0);
        let given = |now: u64| {
            let ServerMessage::ListsAre { seq: 4, lists } = peers.lists(4, now) else {
                panic!("not lists")
            };
            lists.into_iter().map(|l| l.list).collect::<Vec<_>>()
        };
        assert_eq!(given(0), ["list list 8", "list list 9"]);
        assert_eq!(given(1000), ["list list 9"]);
        assert!(given(3000).is_empty());
        // Pruned: a listed server's card expired is dropped.
        peers.list(vec![held(9, &[2], 9000)], 0);
        let s2 = peers.listed().remove(0);
        peers.keep_listed(0, &s2, kept(2, &[], 2000), 0);
        peers.prune(2000);
        assert!(peers.state().listed[0].card.is_none());
    }

    /// A card of the key of seed `n`…`n` vouching for those of `vouches`, as signed by that key.
    fn signed(n: u8, vouches: &[u8]) -> (ServerCard, SignedCard) {
        let key = ServerKey::from_seed([n; 32]);
        let mut card = card::tests::template(&key);
        card.issued = WireU64(1000);
        card.expires = WireU64(9000);
        card.peers = (vouches.iter())
            .map(|&v| CardPeer {
                key: account::to_hex(&ServerKey::from_seed([v; 32]).key()),
                address: format!("ws://s{v}"),
            })
            .collect();
        let signed = h3_protocol::card::sign(&card, &key);
        (card, signed)
    }

    /// `card` relayed with the keys of the seeds `via`.
    fn entry(card: &(ServerCard, SignedCard), via: &[u8]) -> RelayedCard {
        RelayedCard {
            card: card.1.clone(),
            via: (via.iter())
                .map(|&v| account::to_hex(&ServerKey::from_seed([v; 32]).key()))
                .collect(),
            list: None,
        }
    }

    #[test]
    fn a_card_a_peer_relays_is_kept_if_it_holds_and_its_chain_does() {
        // The peer 1 vouches for 3, which vouches for 4; 1 vouches neither for 5 nor 6.
        let (peer, c3, c4) = (signed(1, &[3]), signed(3, &[4]), signed(4, &[]));
        let (c5, c6) = (signed(5, &[]), signed(6, &[]));
        // 3's card, signed by another key.
        let mut forged = entry(&signed(3, &[]), &[]);
        forged.card.signature = signed(7, &[]).1.signature;
        let mut listed = entry(&c3, &[]);
        listed.list = Some(c3.0.key.clone());
        let relayed = vec![
            entry(&c3, &[]),
            entry(&c4, &[3]),
            entry(&c5, &[]),
            entry(&c6, &[3]),
            entry(&c4, &[8]),
            forged,
            listed,
        ];
        let kept = |depth: usize, now: u64| {
            let (relays, dropped) = vouched(&peer.0, relayed.clone(), depth, now);
            let relays: Vec<(String, usize)> = (relays.iter())
                .map(|(kept, via)| (kept.key.clone(), via.len()))
                .collect();
            (relays, dropped)
        };
        let (relays, dropped) = kept(3, 2000);
        assert_eq!(relays, [(c3.0.key.clone(), 0), (c4.0.key.clone(), 1)]);
        let chain = |card: &(ServerCard, SignedCard)| {
            format!(
                "a card it relays is dropped: the chain to {} does not hold",
                card.0.key
            )
        };
        assert_eq!(
            dropped,
            [
                String::from("a card it relays is dropped: it does not hold (Signature)"),
                chain(&c5),
                chain(&c6),
                chain(&c4),
            ]
        );
        // Within depth 2, only the cards of the peer's own peers; what lies beyond is not read.
        let (relays, dropped) = kept(2, 2000);
        assert_eq!(relays, [(c3.0.key.clone(), 0)]);
        assert_eq!(dropped.len(), 2);
        // At depth 1, none; expired, none holds.
        assert_eq!(kept(1, 2000), (vec![], vec![]));
        let (relays, dropped) = kept(3, 9000);
        assert!(relays.is_empty());
        let expired = dropped.iter().filter(|why| why.contains("(Expired)"));
        assert_eq!(expired.count(), 5, "{dropped:?}");
    }
}
