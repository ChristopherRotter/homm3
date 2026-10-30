//! `connect` over loopback (E17 M5 T6a): a host of one game over a real WebSocket on threads, as
//! `h3-server` hosts one (`h3-protocol`'s `Connection` for each connection, its `Session` for the
//! game, sans-I/O both), plain or behind TLS with the server's test certificate.

use std::collections::BTreeMap;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use ed25519_dalek::{Signer as _, SigningKey};
use h3_protocol::account::{KEY_BYTES, SIGNATURE_BYTES};
use h3_protocol::codec::DEFAULT_MAX_CLIENT_FRAME;
use h3_protocol::connection::{Connection as Gate, Host, Received, Signer};
use h3_protocol::session::{ConnId, SeatKind, Session};
use h3_protocol::wire::{Outcome, Seat, ServerMessage};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::{RootCertStore, ServerConfig, ServerConnection, StreamOwned};
use tungstenite::Message;

use super::*;
use crate::bot::Greedy;
use crate::scenario::{GAME, battle, config};

/// The server tests' root, endpoint certificate (localhost, 127.0.0.1) and its key, DER in hex
/// (`crates/h3-server/src/tests/peers.rs`, E16 M6 T3: the project's own test fixtures).
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
const LEAF_KEY: &str = "\
    308187020100301306072a8648ce3d020106082a8648ce3d030107046d306b020101042004e4a99c2cded1f718d8\
    c0c23fc852e3eb2d0c4f7131e79b3c1e759442168073a14403420004530540b7444c9f75affa4f7f3316121739bc\
    e4dc48e5ecf0c524f7bd7bbb9b5b0649b837757ac612f27622c6b766ef29bfc3968f6740bae24ae2b66a9d3dd6ae";

fn bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

/// The roots of a client that trusts the test root only.
fn test_roots() -> RootCertStore {
    let mut roots = RootCertStore::empty();
    roots.add(CertificateDer::from(bytes(ROOT))).unwrap();
    roots
}

/// The host's key, which signs its welcomes.
#[derive(Debug)]
struct Key(SigningKey);

impl Signer for Key {
    fn key(&self) -> [u8; KEY_BYTES] {
        self.0.verifying_key().to_bytes()
    }

    fn sign(&self, message: &[u8]) -> [u8; SIGNATURE_BYTES] {
        self.0.sign(message).to_bytes()
    }
}

/// What the host's connections share: the game and each connection's gate and queue.
struct Shared {
    session: Session<'static>,
    gates: BTreeMap<u64, (Gate, Sender<(String, bool)>)>,
}

/// A connection's socket, plain or behind TLS.
trait Stream: Read + Write + Send {}
impl<S: Read + Write + Send> Stream for S {}

/// A host of `duel.toml`'s battle (seed 1) with `seats`, its address (`wss://localhost:…` with
/// `tls`, `ws://127.0.0.1:…` without) and its key.
fn host(seats: [SeatKind; 2], tls: bool) -> (String, [u8; KEY_BYTES]) {
    let packs = repo().join("packs");
    let (ruleset, setup) = battle(&packs.join("synthetic/scenarios/duel.toml"), &packs, 1).unwrap();
    let ruleset = Box::leak(Box::new(ruleset));
    let session = Session::new(ruleset, config(setup, 1), seats).unwrap();
    let shared = Arc::new(Mutex::new(Shared {
        session,
        gates: BTreeMap::new(),
    }));
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let address = if tls {
        format!("wss://localhost:{port}")
    } else {
        format!("ws://127.0.0.1:{port}")
    };
    let key = Arc::new(Key(SigningKey::from_bytes(&[42; 32])));
    let public = key.key();
    let tls = tls.then(|| {
        let provider = Arc::new(rustls_graviola::default_provider());
        let leaf = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(bytes(LEAF_KEY)));
        let config = ServerConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(vec![CertificateDer::from(bytes(LEAF))], leaf)
            .unwrap();
        Arc::new(config)
    });
    let id = address.clone();
    std::thread::Builder::new()
        .spawn(move || {
            for (n, stream) in (1..).zip(listener.incoming()) {
                let (shared, key, id, tls) = (shared.clone(), key.clone(), id.clone(), tls.clone());
                std::thread::Builder::new()
                    .spawn(move || connection(stream.unwrap(), n, &shared, (&key, &id), tls))
                    .unwrap();
            }
        })
        .unwrap();
    (address, public)
}

/// One connection numbered `n`, until either side closes it.
fn connection(
    stream: TcpStream,
    n: u64,
    shared: &Mutex<Shared>,
    (key, id): (&Key, &str),
    tls: Option<Arc<ServerConfig>>,
) {
    let handle = stream.try_clone().unwrap();
    let stream: Box<dyn Stream> = match tls {
        Some(config) => Box::new(StreamOwned::new(
            ServerConnection::new(config).unwrap(),
            stream,
        )),
        None => Box::new(stream),
    };
    let Ok(mut socket) = tungstenite::accept(stream) else {
        return;
    };
    handle
        .set_read_timeout(Some(Duration::from_millis(2)))
        .unwrap();
    let (queue, queued) = mpsc::channel();
    let gate = Gate::new(DEFAULT_MAX_CLIENT_FRAME, [n as u8; 32], true);
    lock(shared).gates.insert(n, (gate, queue));
    let host = Host {
        program: "loopback host",
        id,
        addresses: &[],
        signer: key,
    };
    loop {
        while let Ok((frame, close)) = queued.try_recv() {
            if socket.send(Message::text(frame)).is_err() {
                return gone(shared, n);
            }
            if close {
                let _ = socket.close(None);
                let _ = socket.flush();
                return gone(shared, n);
            }
        }
        match socket.read() {
            Ok(Message::Text(text)) => receive(&mut lock(shared), n, text.as_bytes(), &host),
            Ok(_) => {}
            Err(tungstenite::Error::Io(e))
                if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
            Err(_) => return gone(shared, n),
        }
    }
}

fn lock(shared: &Mutex<Shared>) -> std::sync::MutexGuard<'_, Shared> {
    shared.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The client's `frame` on connection `n`, as the server's host takes it.
fn receive(shared: &mut Shared, n: u64, frame: &[u8], host: &Host<'_>) {
    let Shared { session, gates } = shared;
    let Some((gate, _)) = gates.get_mut(&n) else {
        return;
    };
    let mut out = Vec::new();
    match gate.receive(frame, host) {
        Received::Reply { frame, close } => out.push((n, frame, close)),
        Received::Game(message) => {
            let mut sent = Vec::new();
            session.handle(ConnId(n), message, &mut sent);
            out.extend(sent.into_iter().map(|o| (o.to.0, o.frame, o.close)));
        }
        Received::Account(_) => {}
    }
    for (to, frame, close) in out {
        if let Some((_, queue)) = gates.get(&to) {
            let _ = queue.send((frame, close));
        }
    }
}

/// Connection `n` closed.
fn gone(shared: &Mutex<Shared>, n: u64) {
    let mut shared = lock(shared);
    shared.gates.remove(&n);
    shared.session.disconnected(ConnId(n));
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// A fresh directory for one test's files.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("h3-view-connect-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The options of a session at `address` as `seat` by its token, its pins in `dir`.
fn options(address: &str, seat: Seat, dir: &Path) -> Options {
    let token = match seat {
        Seat::Attacker => Some("attacker".to_owned()),
        Seat::Defender => Some("defender".to_owned()),
        Seat::Spectator => None,
    };
    Options {
        address: address.to_owned(),
        game: Game::Join(GAME),
        seat,
        token,
        name: "tester".to_owned(),
        pins: dir.join("pins.txt"),
        packs: repo().join("packs"),
        out: Some(dir.join("record.jsonl")),
        drop: None,
    }
}

fn player(token: &str) -> SeatKind {
    SeatKind::Player {
        token: token.to_owned(),
    }
}

/// The record's messages are numbered from 1, one after another, and it holds the game's end once
/// (a re-join after it adds the re-join's snapshot); that end.
fn whole(record: &Record) -> Outcome {
    for (n, message) in (record.header.first..).zip(&record.messages) {
        assert_eq!(remote::number(message), Some(n));
    }
    assert_eq!(record.header.first, 1);
    let ends: Vec<Outcome> = (record.messages.iter())
        .filter_map(|m| match m {
            ServerMessage::GameEnded { result, .. } => Some(*result),
            _ => None,
        })
        .collect();
    assert_eq!(ends.len(), 1, "{:?}", record.messages.last());
    ends[0]
}

#[test]
fn a_battle_is_played_to_its_end_over_loopback_and_the_key_is_pinned() {
    let (address, key) = host([player("attacker"), SeatKind::Bot(Box::new(Greedy))], false);
    let dir = scratch("end");
    let done = session(&options(&address, Seat::Attacker, &dir), test_roots).unwrap();
    assert_eq!(done.content, Content::Same);
    assert_eq!(done.rejoined, 0);
    whole(&done.record);
    assert_eq!(done.record.header.seat, Seat::Attacker);
    let pins = Pins::parse(&std::fs::read_to_string(dir.join("pins.txt")).unwrap()).unwrap();
    assert_eq!(pins.key(&address), Some(key));
    assert!(
        done.lines
            .iter()
            .any(|l| l.starts_with("pinned the server's key"))
    );
    // The record's text reads back as the viewer's file.
    let text = text(&done.record);
    assert_eq!(h3_view::record::parse(&text).unwrap(), done.record);
}

#[test]
fn two_seats_play_over_tls_and_one_re_joins_after_a_drop() {
    let (address, _) = host([player("attacker"), player("defender")], true);
    let (attacker, defender) = (scratch("tls-a"), scratch("tls-d"));
    let mut dropping = options(&address, Seat::Defender, &defender);
    dropping.drop = Some(1);
    let other = std::thread::Builder::new()
        .spawn(move || session(&dropping, test_roots))
        .unwrap();
    let a = session(&options(&address, Seat::Attacker, &attacker), test_roots).unwrap();
    let d = other.join().unwrap().unwrap();
    assert_eq!((a.rejoined, d.rejoined), (0, 1));
    assert_eq!((a.content, d.content), (Content::Same, Content::Same));
    // The battle waits for the defender's turn: it re-joins mid-battle, its record from the
    // server holding what came before the drop and the re-join's snapshot, and plays on.
    assert_eq!(whole(&a.record), whole(&d.record));
    let snapshots = (d.record.messages.iter())
        .filter(|m| matches!(m, ServerMessage::Snapshot { .. }))
        .count();
    assert_eq!(snapshots, 2);
    assert!(
        d.lines
            .iter()
            .any(|l| l == "dropped the connection after 1 commands")
    );
    let played = d.lines.last().unwrap();
    assert!(!played.ends_with("after 1 commands sent here"), "{played}");
    // The second connection's welcome held the key the first pinned.
    let pinned = d.lines.iter().filter(|l| l.starts_with("pinned")).count();
    assert_eq!(pinned, 1);
}

#[test]
fn a_changed_key_a_wrong_token_and_a_bad_address_end_the_session() {
    let (address, _) = host([player("attacker"), player("defender")], false);
    let dir = scratch("faults");
    let mut pins = Pins::default();
    pins.pin(&address, [3; KEY_BYTES]);
    std::fs::write(dir.join("pins.txt"), pins.text()).unwrap();
    let changed = session(&options(&address, Seat::Attacker, &dir), test_roots).unwrap_err();
    assert!(changed.starts_with("THE SERVER'S KEY CHANGED"), "{changed}");
    std::fs::remove_file(dir.join("pins.txt")).unwrap();
    let mut wrong = options(&address, Seat::Attacker, &dir);
    wrong.token = Some("defender".to_owned());
    assert_eq!(
        session(&wrong, test_roots).unwrap_err(),
        "refused: BadToken"
    );
    let mut http = options("http://127.0.0.1:1", Seat::Attacker, &dir);
    http.address = "http://127.0.0.1:1".to_owned();
    let bad = session(&http, test_roots).unwrap_err();
    assert!(bad.ends_with("not a ws:// or wss:// address"), "{bad}");
    std::fs::write(dir.join("pins.txt"), "not a pin").unwrap();
    let pins = session(&options(&address, Seat::Attacker, &dir), test_roots).unwrap_err();
    assert!(pins.ends_with("pins.txt: line 1"), "{pins}");
}

#[test]
fn without_the_packs_the_content_is_absent() {
    let (address, _) = host([player("attacker"), SeatKind::Bot(Box::new(Greedy))], false);
    let dir = scratch("absent");
    let mut options = options(&address, Seat::Attacker, &dir);
    options.packs = dir.clone();
    let done = session(&options, test_roots).unwrap();
    assert_eq!(done.content, Content::Absent);
}

#[test]
fn the_plain_player_strikes_first_and_walks_toward_the_enemy() {
    let walks = [
        Command::Wait {},
        Command::Defend {},
        Command::Move { to: 18 },
        Command::Move { to: 20 },
        Command::Move { to: 36 },
        Command::Retreat {},
    ];
    assert_eq!(
        plain(&walks, Seat::Attacker, 17),
        Some(Command::Move { to: 20 })
    );
    assert_eq!(
        plain(&walks, Seat::Defender, 17),
        Some(Command::Move { to: 18 })
    );
    let strikes = [
        Command::Move { to: 3 },
        Command::Attack { target: 4, from: 3 },
        Command::Shoot { target: 4, at: 4 },
    ];
    assert_eq!(
        plain(&strikes, Seat::Attacker, 17),
        Some(Command::Shoot { target: 4, at: 4 })
    );
    assert_eq!(
        plain(&strikes[..2], Seat::Attacker, 17),
        Some(Command::Attack { target: 4, from: 3 })
    );
    let tactics = [Command::Move { to: 3 }, Command::EndTactics {}];
    assert_eq!(
        plain(&tactics, Seat::Defender, 17),
        Some(Command::EndTactics {})
    );
    assert_eq!(
        plain(&walks[..2], Seat::Attacker, 17),
        Some(Command::Defend {})
    );
    assert_eq!(
        plain(&walks[..1], Seat::Attacker, 17),
        Some(Command::Wait {})
    );
    assert_eq!(plain(&walks[5..], Seat::Attacker, 17), None);
}

#[test]
fn the_options_name_the_address_the_game_and_the_seat() {
    let args = |s: &str| s.split(' ').map(str::to_owned).collect::<Vec<_>>();
    let parsed = Options::parse(&args(
        "wss://h3.example --game 7 --seat defender --token t --name Ann --pins p --packs k \
         --out o --drop 2",
    ))
    .unwrap();
    assert_eq!(
        parsed,
        Options {
            address: "wss://h3.example".to_owned(),
            game: Game::Join(7),
            seat: Seat::Defender,
            token: Some("t".to_owned()),
            name: "Ann".to_owned(),
            pins: PathBuf::from("p"),
            packs: PathBuf::from("k"),
            out: Some(PathBuf::from("o")),
            drop: Some(2),
        }
    );
    let spectator = Options::parse(&args("ws://h --game 1 --seat spectator")).unwrap();
    assert_eq!((spectator.token.as_deref(), spectator.drop), (None, None));
    assert_eq!(spectator.pins, PathBuf::from("pins.txt"));
    assert!(
        spectator
            .record_path(1)
            .ends_with("out/connect/g1-spectator.jsonl")
    );
    let create = Options::parse(&args("ws://h --create duel --seat attacker")).unwrap();
    assert_eq!(create.game, Game::Create("duel".to_owned()));
    for (bad, why) in [
        ("--game 1 --seat attacker", "needs the server's address"),
        ("ws://h --seat attacker", "needs --game or --create"),
        ("ws://h --game 1 --create duel", "not both"),
        ("ws://h --game 1", "needs --seat"),
        ("ws://h --game x", "bad number"),
        ("ws://h --seat king", "unknown seat"),
        ("ws://h --game", "needs a value"),
        ("ws://h --what", "unknown option"),
        ("ws://h ws://i", "unexpected"),
    ] {
        let error = Options::parse(&args(bad)).unwrap_err();
        assert!(error.contains(why), "{bad}: {error}");
    }
}
