//! The peers over loopback (E16 M6 T3, T4; ADR-0021 point 3): a server fetches the cards of the
//! servers its configuration vouches for and relays them with `ListPeers`, its own card naming
//! them; a peer whose key is not the configured one, that names itself at another address, or
//! whose card is forged, expired or another server's is not relayed, and the fetch says why (the
//! log line); a peer that goes down keeps its last good card; a peer behind a TLS proxy is fetched
//! at `wss://`, its certificate checked against the roots the server was given; the peers'
//! configuration is bounded. Beyond the peers (T4): the server relays what its peers relay, to
//! `depth` hops and at most `cards`, each card with the chain that vouches for it, which the test
//! client checks; it never dials a server it only read of; a card a peer relays outside its own
//! vouching, or forged, is dropped, and the fetch says why.
//!
//! The test certificates (P-256, valid until 2126; never use them elsewhere) were made with:
//!
//! ```text
//! openssl ecparam -name prime256v1 -genkey -noout -out root.key
//! openssl req -x509 -new -key root.key -sha256 -days 36500 -subj "/CN=h3-server test root" \
//!     -addext "basicConstraints=critical,CA:TRUE" -addext "keyUsage=critical,keyCertSign,cRLSign"
//! openssl ecparam -name prime256v1 -genkey -noout -out leaf.key
//! openssl req -new -key leaf.key -subj "/CN=localhost" -out leaf.csr
//! openssl x509 -req -in leaf.csr -CA root.pem -CAkey root.key -CAcreateserial -days 36500 \
//!     -sha256 -extfile ext.cnf    # basicConstraints=critical,CA:FALSE, keyUsage=critical,
//!                                 # digitalSignature, extendedKeyUsage=serverAuth,
//!                                 # subjectAltName=DNS:localhost,IP:127.0.0.1
//! openssl pkcs8 -topk8 -nocrypt -in leaf.key -outform DER     # then each as DER, in hex
//! ```

use std::net::SocketAddr;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use h3_protocol::account;
use h3_protocol::card::PAGE;
use h3_protocol::connection::{Connection, Host, Received, Signer};
use h3_protocol::wire::{CardPeer, ListedServer, RelayedCard, ServerCard, SignedCard, WireU64};
use rustls::RootCertStore;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use tokio::net::{TcpListener, TcpStream};

use super::lists::{DAY, list_of, place, signed_by};
use super::*;
use crate::card::{self, wall_ms};
use crate::config::Peer;
use crate::key::ServerKey;
use crate::peers::{self, fetch_now};

/// The test root, DER in hex.
const ROOT: &str = "\
    308201a330820149a0030201020214221abb5b2711940ffb564c4a6250c82148baaf2a300a06082a8648ce3d0403\
    02301e311c301a06035504030c1368332d736572766572207465737420726f6f743020170d323631303133313133\
    3431355a180f32313236303931393131333431355a301e311c301a06035504030c1368332d736572766572207465\
    737420726f6f743059301306072a8648ce3d020106082a8648ce3d030107034200044ff70d52265cf43d4f284864\
    ed1be1ea03f864df0518c3a1d0745c9a9f7a47b635b271fa7015624644243d50bc6609ba4cd8aa254380914fcbe0\
    efefef5cbb49a3633061301d0603551d0e0416041432417f36b7f97207743b2397aeca059c3cf07602301f060355\
    1d2304183016801432417f36b7f97207743b2397aeca059c3cf07602300f0603551d130101ff040530030101ff30\
    0e0603551d0f0101ff040403020106300a06082a8648ce3d0403020348003045022100d344eafe13de97867ceb60\
    a111a8ca76164ea80311ffa62b58dd24e1ddbeb2a00220754b5fcf44dcc5f65a9cfe901962d8d2fb2c21a4c9aa8e\
    aad0cc1ec7b55cc3ab";

/// The test endpoint's certificate (localhost, 127.0.0.1), signed by the test root, DER in hex.
const LEAF: &str = "\
    308201ca3082016fa00302010202142e5277dd47a91acf89b32ff5e126f1722b0e64ef300a06082a8648ce3d0403\
    02301e311c301a06035504030c1368332d736572766572207465737420726f6f743020170d323631303133313133\
    3431355a180f32313236303931393131333431355a30143112301006035504030c096c6f63616c686f7374305930\
    1306072a8648ce3d020106082a8648ce3d03010703420004530540b7444c9f75affa4f7f3316121739bce4dc48e5\
    ecf0c524f7bd7bbb9b5b0649b837757ac612f27622c6b766ef29bfc3968f6740bae24ae2b66a9d3dd6aea3819230\
    818f300c0603551d130101ff04023000300e0603551d0f0101ff04040302078030130603551d25040c300a06082b\
    06010505070301301a0603551d110413301182096c6f63616c686f737487047f000001301d0603551d0e04160414\
    faf04a981174829ba30bb75e3b7bc9d60e5f92d6301f0603551d2304183016801432417f36b7f97207743b2397ae\
    ca059c3cf07602300a06082a8648ce3d0403020349003046022100d5fa6fd3f6be1851bdda2855e11025dc4676c6\
    358c951fae3cadfe1174129cba022100b1d88776228a185ee53156232ceb282ab59c1b8919413df75cfdf3cef8fd\
    4aff";

/// The test endpoint's private key, PKCS#8 DER in hex.
const LEAF_KEY: &str = "\
    308187020100301306072a8648ce3d020106082a8648ce3d030107046d306b020101042004e4a99c2cded1f718d8\
    c0c23fc852e3eb2d0c4f7131e79b3c1e759442168073a14403420004530540b7444c9f75affa4f7f3316121739bc\
    e4dc48e5ecf0c524f7bd7bbb9b5b0649b837757ac612f27622c6b766ef29bfc3968f6740bae24ae2b66a9d3dd6ae";

/// Bytes of `hex`.
fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

/// The roots of a server that trusts the test root only.
fn test_roots() -> RootCertStore {
    let mut roots = RootCertStore::empty();
    roots.add(CertificateDer::from(bytes(ROOT))).unwrap();
    roots
}

/// A fetch's round trips well within this.
const WAIT: Duration = Duration::from_secs(20);

/// A server that fetches its peers `peers` every 200 ms (below the configuration's bounds).
fn fetching(peers: Vec<Peer>) -> Config {
    let mut config = config(fast());
    config.discovery.refresh_ms = 200;
    config.discovery.timeout_ms = 5_000;
    config.peers = peers;
    config
}

/// `running`'s key, as a client's welcome shows it.
async fn key_of(running: &Running) -> String {
    let client = greeted(running).await;
    account::to_hex(&client.server_key().unwrap())
}

/// `running` as a peer at its own address.
async fn peer(running: &Running) -> Peer {
    Peer {
        key: key_of(running).await,
        address: format!("ws://{}", running.address),
    }
}

/// Resolves once `running` did `rounds` rounds of fetches since its start.
async fn rounds(running: &Running, rounds: u64) {
    tokio::time::timeout(WAIT, running.peers.fetched(rounds))
        .await
        .unwrap();
}

/// What `running` relays, each card checked by its client against its own card: the cards, and
/// as they were signed.
async fn relayed(running: &Running) -> Vec<(ServerCard, SignedCard)> {
    let mut client = greeted(running).await;
    let (own, _) = client.describe(1).await.unwrap();
    let relayed = client.relayed(&own).await.unwrap();
    relayed
        .into_iter()
        .map(|(card, e)| (card, e.card))
        .collect()
}

/// `running`'s own card, as signed.
async fn own_card(running: &Running) -> (ServerCard, SignedCard) {
    greeted(running).await.describe(1).await.unwrap()
}

/// The TLS client a server builds for the test root.
fn test_tls() -> Arc<rustls::ClientConfig> {
    peers::tls(test_roots()).unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_server_relays_the_cards_of_the_peers_it_vouches_for() {
    let mut named = config(fast());
    named.server.name = Some(String::from("Bea's"));
    let b = start(&named).await.unwrap();
    let c = start(&config(fast())).await.unwrap();
    let (pb, pc) = (peer(&b).await, peer(&c).await);
    let a = start(&fetching(vec![pb.clone(), pc.clone()]))
        .await
        .unwrap();
    rounds(&a, 1).await;
    // A's card vouches for B and C, in the configuration's order.
    let (own, _) = own_card(&a).await;
    let configured = [&pb, &pc].map(|p| CardPeer {
        key: p.key.clone(),
        address: p.address.clone(),
    });
    assert_eq!(own.peers, configured);
    // Each card as its server signed it (the same within a minute), each checked by the client.
    let relayed = relayed(&a).await;
    let (b_card, b_signed) = own_card(&b).await;
    let (_, c_signed) = own_card(&c).await;
    assert_eq!(relayed[0], (b_card, b_signed));
    assert_eq!(relayed[0].0.name, "Bea's");
    assert_eq!(relayed[1].1, c_signed);
    assert_eq!(relayed.len(), 2);
    // Paged: from the second, the second; past the last, none.
    let mut client = greeted(&a).await;
    let (total, page) = client.list_peers(3, 1).await.unwrap();
    assert_eq!((total, page.len()), (2, 1));
    assert_eq!(page[0].card, c_signed);
    assert_eq!(client.list_peers(4, 2).await.unwrap(), (2, vec![]));
    // B relays nothing: it vouches for no one.
    assert!(relayed_by(&b).await.is_empty());
    for server in [a, b, c] {
        server.shutdown().await;
    }
}

/// What `running` relays, unchecked.
async fn relayed_by(running: &Running) -> Vec<RelayedCard> {
    greeted(running).await.list_peers(1, 0).await.unwrap().1
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_peer_gone_keeps_its_last_good_card() {
    let b = start(&config(fast())).await.unwrap();
    let pb = peer(&b).await;
    let a = start(&fetching(vec![pb.clone()])).await.unwrap();
    rounds(&a, 1).await;
    let before = relayed(&a).await;
    assert_eq!(before.len(), 1);
    b.shutdown().await;
    // Two rounds later, one of them wholly after B stopped: B's fetch fails, its card stays.
    let error = fetch_now(&pb, test_tls()).await.unwrap_err();
    assert!(error.0.contains("connect"), "{error}");
    rounds(&a, a.peers.rounds() + 2).await;
    assert_eq!(relayed(&a).await, before);
    a.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_peer_whose_key_is_another_or_that_names_another_address_is_not_relayed() {
    let b = start(&config(fast())).await.unwrap();
    let mut pb = peer(&b).await;
    pb.key = account::to_hex(&ServerKey::from_seed([9; 32]).key());
    let error = fetch_now(&pb, test_tls()).await.unwrap_err();
    assert!(error.0.contains("not the configured one"), "{error}");
    let a = start(&fetching(vec![pb])).await.unwrap();
    rounds(&a, 2).await;
    assert!(relayed_by(&a).await.is_empty());
    a.shutdown().await;
    b.shutdown().await;
    // A server whose id is another address than the one dialed: its key answers, but elsewhere.
    let mut elsewhere = config(fast());
    elsewhere.server.public_url = Some(String::from("wss://b.example.org"));
    let b = start(&elsewhere).await.unwrap();
    let mut pb = peer(&b).await;
    let error = fetch_now(&pb, test_tls()).await.unwrap_err();
    assert!(
        error.0.contains("names itself \"wss://b.example.org\""),
        "{error}"
    );
    // Dialed at an address it does not know as its own: the same.
    pb.address = format!("ws://localhost:{}", b.address.port());
    assert!(fetch_now(&pb, test_tls()).await.is_err());
    b.shutdown().await;
}

/// A peer that answers each hello with a welcome by the key of `seed`, each `Describe` with
/// `card` as it is, each `ListPeers` with a page of `relays` and `Lists` with none: its address.
pub(super) async fn fake(seed: [u8; 32], card: SignedCard, relays: Vec<RelayedCard>) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        while let Ok((tcp, _)) = listener.accept().await {
            let (card, relays) = (card.clone(), relays.clone());
            tokio::spawn(async move {
                let mut socket = tokio_tungstenite::accept_async(tcp).await.unwrap();
                let key = ServerKey::from_seed(seed);
                let id = format!("ws://{address}");
                let host = Host {
                    program: "fake",
                    id: &id,
                    addresses: &[],
                    signer: &key,
                };
                let mut gate = Connection::new(4096, [3; 32], true);
                while let Some(Ok(Message::Text(text))) = socket.next().await {
                    let answer = match gate.receive(text.as_bytes(), &host) {
                        Received::Reply { frame, .. } => frame,
                        Received::Game(ClientMessage::Describe { seq }) => {
                            let card = card.clone();
                            codec::encode_text(&ServerMessage::Described { seq, card })
                        }
                        Received::Game(ClientMessage::Lists { seq }) => {
                            let lists = vec![];
                            codec::encode_text(&ServerMessage::ListsAre { seq, lists })
                        }
                        Received::Game(ClientMessage::ListPeers { seq, from }) => {
                            let entries = (relays.iter().skip(from as usize).take(PAGE))
                                .cloned()
                                .collect();
                            let total = relays.len() as u32;
                            codec::encode_text(&ServerMessage::Peers {
                                seq,
                                from,
                                total,
                                entries,
                            })
                        }
                        _ => return,
                    };
                    if socket.send(Message::text(answer)).await.is_err() {
                        return;
                    }
                }
            });
        }
    });
    address
}

/// A card of the key of `seed`, issued at `issued` and expiring at `expires`.
pub(super) fn card_of(seed: [u8; 32], (issued, expires): (u64, u64)) -> ServerCard {
    let mut card = card::tests::template(&ServerKey::from_seed(seed));
    card.issued = WireU64(issued);
    card.expires = WireU64(expires);
    card
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_forged_expired_or_foreign_card_is_not_relayed() {
    let (seed, other) = ([7; 32], [8; 32]);
    let key = ServerKey::from_seed(seed);
    let now = wall_ms();
    let good = card_of(seed, (now, now + DAY));
    let signed =
        |card: &ServerCard, by: [u8; 32]| h3_protocol::card::sign(card, &ServerKey::from_seed(by));
    let mut tampered = signed(&good, seed);
    tampered.card = tampered.card.replace("\"Test\"", "\"Tost\"");
    let cases = [
        // Signed by another key than the one it names.
        (signed(&good, other), "Signature"),
        (tampered, "Signature"),
        (
            signed(&card_of(seed, (now - 2 * DAY, now - DAY)), seed),
            "Expired",
        ),
        (
            signed(&card_of(seed, (now + DAY, now + 2 * DAY)), seed),
            "Ahead",
        ),
        // Good, but another server's: not the welcome's key.
        (
            signed(&card_of(other, (now, now + DAY)), other),
            "another card",
        ),
    ];
    let mut peers = Vec::new();
    for (card, why) in cases {
        let address = fake(seed, card, vec![]).await;
        let fake = Peer {
            key: account::to_hex(&key.key()),
            address: format!("ws://{address}"),
        };
        let error = fetch_now(&fake, test_tls()).await.unwrap_err();
        assert!(error.0.contains(why), "{why}: {error}");
        peers.push(fake);
    }
    // A fake peer whose card is good is relayed: the fakes differ in their cards alone.
    let address = fake(seed, signed(&good, seed), vec![]).await;
    let honest = Peer {
        key: account::to_hex(&key.key()),
        address: format!("ws://{address}"),
    };
    assert!(fetch_now(&honest, test_tls()).await.is_ok());
    let a = start(&fetching(peers)).await.unwrap();
    rounds(&a, 2).await;
    assert!(relayed_by(&a).await.is_empty());
    a.shutdown().await;
}

/// What `running` relays, each card checked by its client with its chain: each card's key and
/// its `via`.
async fn chains(running: &Running) -> Vec<(String, Vec<String>)> {
    let mut client = greeted(running).await;
    let (own, _) = client.describe(1).await.unwrap();
    (client.relayed(&own).await.unwrap().into_iter())
        .map(|(card, entry)| (card.key, entry.via))
        .collect()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_server_relays_what_its_peers_relay_to_depth_each_card_with_its_chain() {
    // A → B → C → D: each vouches for the next.
    let d = start(&fetching(vec![])).await.unwrap();
    let c = start(&fetching(vec![peer(&d).await])).await.unwrap();
    rounds(&c, 1).await;
    let b = start(&fetching(vec![peer(&c).await])).await.unwrap();
    rounds(&b, 1).await;
    let (kb, kc, kd) = (key_of(&b).await, key_of(&c).await, key_of(&d).await);
    // B relays C, and D through C (at the default depth, 2).
    let through_c = (kd.clone(), vec![kc.clone()]);
    assert_eq!(chains(&b).await, [(kc.clone(), vec![]), through_c]);
    // C and D go down: A learns of them only through B, and dials B alone.
    let pb = peer(&b).await;
    c.shutdown().await;
    d.shutdown().await;
    let a = start(&fetching(vec![pb.clone()])).await.unwrap();
    rounds(&a, 1).await;
    let (to_b, through_b) = ((kb.clone(), vec![]), (kc.clone(), vec![kb.clone()]));
    assert_eq!(chains(&a).await, [to_b.clone(), through_b.clone()]);
    // Three hops: D too, by the chain A → B → C → D, which the client checked card by card.
    let mut deeper = fetching(vec![pb.clone()]);
    deeper.discovery.depth = 3;
    let a3 = start(&deeper).await.unwrap();
    rounds(&a3, 1).await;
    let through_b_c = (kd, vec![kb, kc]);
    assert_eq!(chains(&a3).await, [to_b.clone(), through_b, through_b_c]);
    // At most `cards`, the nearer first.
    let mut bounded = fetching(vec![pb]);
    (bounded.discovery.depth, bounded.discovery.cards) = (3, 1);
    let a1 = start(&bounded).await.unwrap();
    rounds(&a1, 1).await;
    assert_eq!(chains(&a1).await, [to_b]);
    for server in [a, a3, a1, b] {
        server.shutdown().await;
    }
}

/// The key of the seed `n`…`n`, in hex.
fn key_hex(n: u8) -> String {
    account::to_hex(&ServerKey::from_seed([n; 32]).key())
}

/// A card of the key of the seed `n`…`n` that holds for a day, vouching for those of `vouches`, as
/// that key signed it.
fn vouching(n: u8, vouches: &[u8]) -> SignedCard {
    let now = wall_ms();
    let mut card = card_of([n; 32], (now, now + 86_400_000));
    card.peers = (vouches.iter())
        .map(|&v| CardPeer {
            key: key_hex(v),
            address: format!("ws://s{v}"),
        })
        .collect();
    h3_protocol::card::sign(&card, &ServerKey::from_seed([n; 32]))
}

/// `card` relayed with the keys of the seeds `via`.
fn relayed_as(card: SignedCard, via: &[u8]) -> RelayedCard {
    RelayedCard {
        card,
        via: via.iter().map(|&v| key_hex(v)).collect(),
        list: None,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_card_a_peer_relays_outside_its_vouching_or_forged_is_dropped() {
    // B (seed 1) vouches for C (3), which vouches for D (4); no one vouches for X (5) or Y (6).
    let honest = vec![
        relayed_as(vouching(3, &[4]), &[]),
        relayed_as(vouching(4, &[]), &[3]),
    ];
    let mut forged = vouching(3, &[4]);
    forged.card = forged.card.replace("\"Test\"", "\"Tost\"");
    let mut relays = honest.clone();
    relays.extend([
        relayed_as(vouching(5, &[]), &[]),
        relayed_as(vouching(6, &[]), &[3]),
        relayed_as(forged, &[]),
    ]);
    let pb = Peer {
        key: key_hex(1),
        address: format!("ws://{}", fake([1; 32], vouching(1, &[3]), relays).await),
    };
    // The fetch keeps C and D, and says why it drops the others.
    let (_, dropped) = fetch_now(&pb, test_tls()).await.unwrap();
    let chain = |n: u8| {
        let key = key_hex(n);
        format!("a card it relays is dropped: the chain to {key} does not hold")
    };
    let forged = String::from("a card it relays is dropped: it does not hold (Signature)");
    assert_eq!(dropped, [forged, chain(5), chain(6)]);
    let mut deeper = fetching(vec![pb]);
    deeper.discovery.depth = 3;
    let a = start(&deeper).await.unwrap();
    rounds(&a, 1).await;
    let b = (key_hex(1), vec![]);
    let c = (key_hex(3), vec![key_hex(1)]);
    let d = (key_hex(4), vec![key_hex(1), key_hex(3)]);
    assert_eq!(chains(&a).await, [b, c, d]);
    a.shutdown().await;
}

/// A client that said hello to the server at `address`, and its card.
async fn greeted_at(address: SocketAddr) -> (Client, ServerCard) {
    let mut client = Client::connect(address).await.unwrap();
    client.hello("test").await.unwrap();
    let (card, _) = client.describe(1).await.unwrap();
    (client, card)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_client_checks_each_chain_a_server_relays() {
    let honest = vec![
        relayed_as(vouching(3, &[4]), &[]),
        relayed_as(vouching(4, &[]), &[3]),
    ];
    let address = fake([1; 32], vouching(1, &[3]), honest.clone()).await;
    let (mut client, own) = greeted_at(address).await;
    let checked = client.relayed(&own).await.unwrap();
    assert_eq!(checked.len(), 2);
    // Each fails: a card its server does not vouch for; one whose voucher does not; one through a
    // card the server does not relay.
    for (bad, failing) in [
        (relayed_as(vouching(5, &[]), &[]), 5),
        (relayed_as(vouching(6, &[]), &[3]), 6),
        (relayed_as(vouching(4, &[]), &[7]), 4),
    ] {
        let mut relays = honest.clone();
        relays.push(bad);
        let address = fake([1; 32], vouching(1, &[3]), relays).await;
        let (mut client, own) = greeted_at(address).await;
        let error = client.relayed(&own).await.unwrap_err();
        let expected = format!("its chain does not vouch for: {}", key_hex(failing));
        assert!(error.0.contains(&expected), "{error}");
    }
    // A card marked with a list the server does not give (E16 M6 T5): its list does not name it.
    let mut listed = relayed_as(vouching(5, &[]), &[]);
    listed.list = Some(key_hex(9));
    let address = fake([1; 32], vouching(1, &[3]), vec![listed]).await;
    let (mut client, own) = greeted_at(address).await;
    let error = client.relayed(&own).await.unwrap_err();
    let expected = format!("its list does not name: {}", key_hex(5));
    assert!(error.0.contains(&expected), "{error}");
}

/// A TLS proxy in front of `to` with the test certificate, as an operator's reverse proxy is: its
/// address.
async fn proxy(listener: TcpListener, to: SocketAddr) {
    let provider = Arc::new(rustls_graviola::default_provider());
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(bytes(LEAF_KEY)));
    let config = rustls::ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![CertificateDer::from(bytes(LEAF))], key)
        .unwrap();
    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(config));
    tokio::spawn(async move {
        while let Ok((tcp, _)) = listener.accept().await {
            let acceptor = acceptor.clone();
            tokio::spawn(async move {
                let (Ok(mut tls), Ok(mut plain)) =
                    (acceptor.accept(tcp).await, TcpStream::connect(to).await)
                else {
                    return;
                };
                let _ = tokio::io::copy_bidirectional(&mut tls, &mut plain).await;
            });
        }
    });
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_peer_behind_a_tls_proxy_is_fetched_at_wss() {
    // B behind the proxy, whose address is B's public one.
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let public = format!("wss://localhost:{}", listener.local_addr().unwrap().port());
    let mut behind = config(fast());
    behind.server.public_url = Some(public.clone());
    let b = start(&behind).await.unwrap();
    proxy(listener, b.address).await;
    let pb = Peer {
        key: key_of(&b).await,
        address: public,
    };
    // Trusting the test root, A fetches B's card through the proxy.
    let a = crate::launch(&fetching(vec![pb.clone()]), test_roots)
        .await
        .unwrap();
    rounds(&a, 1).await;
    let peered = relayed(&a).await;
    assert_eq!(peered.len(), 1);
    assert_eq!(peered[0].0.addresses[0], pb.address);
    a.shutdown().await;
    // A server without peers whose list names B at `wss://` reads the roots for it alike (E16 M6's
    // review: they were read only for a peer at `wss://`).
    let listing = fetching(vec![]);
    let now = wall_ms();
    let lb = ListedServer {
        key: pb.key.clone(),
        address: pb.address.clone(),
    };
    let list = signed_by(&list_of(1, "Community", (now, now + DAY), &[&lb]), 1);
    place(&listing.server.data, "a.json", &codec::encode_text(&list));
    let c = crate::launch(&listing, test_roots).await.unwrap();
    rounds(&c, 1).await;
    let listed = relayed(&c).await;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].0.addresses[0], pb.address);
    c.shutdown().await;
    // The operating system's roots do not know the test root: the certificate is refused.
    let error = fetch_now(&pb, peers::tls(peers::os_roots()).unwrap())
        .await
        .unwrap_err();
    assert!(error.0.contains("certificate"), "{error}");
    // Nor does a plain `ws://` dial reach a TLS endpoint.
    let mut plain = pb.clone();
    plain.address = pb.address.replace("wss://", "ws://");
    assert!(fetch_now(&plain, test_tls()).await.is_err());
    b.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_peer_with_the_servers_own_key_stops_the_start() {
    let mut config = fetching(vec![]);
    let b = start(&config).await.unwrap();
    let own = key_of(&b).await;
    b.shutdown().await;
    config.peers = vec![Peer {
        key: own,
        address: String::from("ws://127.0.0.1:1"),
    }];
    let error = start(&config).await.unwrap_err();
    assert!(error.0.contains("peer 0: the server's own key"), "{error}");
}

#[test]
fn the_peers_configuration_is_bounded() {
    let base = root();
    let parse = |peers: &str, discovery: &str| {
        let text = format!(
            "[discovery]\n{discovery}\n[content]\npacks = \"p\"\n\
             [[offer]]\nid = \"a\"\nscenario = \"s.toml\"\n{peers}"
        );
        Config::parse(&text, &base)
    };
    let key = |seed: u8| account::to_hex(&ServerKey::from_seed([seed; 32]).key());
    let entry =
        |key: &str, address: &str| format!("[[peer]]\nkey = \"{key}\"\naddress = \"{address}\"\n");
    let defaults = parse("", "").unwrap();
    assert!(defaults.peers.is_empty());
    let discovery = |d: &Config| {
        let d = d.discovery;
        (d.refresh_ms, d.timeout_ms, d.depth, d.cards)
    };
    assert_eq!(discovery(&defaults), (3_600_000, 10_000, 2, 200));
    let two = entry(&key(1), "wss://h3.example.net") + &entry(&key(2), "ws://192.168.1.6:7878");
    let set = parse(
        &two,
        "refresh_ms = 60000\ntimeout_ms = 1000\ndepth = 4\ncards = 1000",
    )
    .unwrap();
    assert_eq!(set.peers.len(), 2);
    assert_eq!(set.peers[1].address, "ws://192.168.1.6:7878");
    assert_eq!(discovery(&set), (60_000, 1_000, 4, 1000));
    let least = discovery(&parse("", "depth = 1\ncards = 1").unwrap());
    assert_eq!((least.2, least.3), (1, 1));
    let many: String = (0..=64u8)
        .map(|n| entry(&key(n), &format!("ws://h{n}")))
        .collect();
    for (peers, discovery, error) in [
        (entry("00".repeat(32).as_str(), "ws://a"), "", "peer 0: key"),
        (entry(&key(1).to_uppercase(), "ws://a"), "", "peer 0: key"),
        (entry(&key(1)[2..], "ws://a"), "", "peer 0: key"),
        (entry(&key(1), "https://a"), "", "peer 0: address"),
        (
            entry(&key(1), "ws://a") + &entry(&key(1), "ws://b"),
            "",
            "peer 1",
        ),
        (
            entry(&key(1), "ws://a") + &entry(&key(2), "ws://a"),
            "",
            "peer 1",
        ),
        (many, "", "at most 64 [[peer]]"),
        (String::new(), "refresh_ms = 59999", "discovery.refresh_ms"),
        (
            String::new(),
            "refresh_ms = 86400001",
            "discovery.refresh_ms",
        ),
        (String::new(), "timeout_ms = 999", "discovery.timeout_ms"),
        (String::new(), "timeout_ms = 120001", "discovery.timeout_ms"),
        (
            String::new(),
            "depth = 0",
            "discovery.depth = 0: from 1 to 4",
        ),
        (String::new(), "depth = 5", "discovery.depth"),
        (
            String::new(),
            "cards = 0",
            "discovery.cards = 0: from 1 to 1000",
        ),
        (String::new(), "cards = 1001", "discovery.cards"),
        (entry(&key(1), "ws://a") + "port = 1\n", "", "unknown field"),
    ] {
        let parsed = parse(&peers, discovery).unwrap_err();
        assert!(parsed.0.contains(error), "{peers} {discovery}: {parsed}");
    }
}

#[test]
fn the_operating_systems_roots_are_read() {
    // Whatever this machine has: reading them never fails, and the client is built on them.
    let roots = peers::os_roots();
    assert!(peers::tls(roots).is_ok());
}
