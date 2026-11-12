//! The whole game's frames and records on the XL fixtures (E16 M5 T7a1): each of the eight
//! 144-tile fixture maps played 28 days at random through the session, every frame within its
//! bound and every record within its own. The bounds, provisional (part 2's network seats size the
//! server's queue and records by them): a reveal at most the snapshot it brings its viewer's view
//! to (its tiles packed as the snapshot's: E16 M5 T7a1 measured an unpacked whole map's reveal 60
//! times its snapshot), a snapshot within [`SNAPSHOT`], every other frame within [`FRAME`]; a
//! record begins with a snapshot and holds at most [`RECORD`] bytes.

use std::format;
use std::sync::Arc;
use std::vec::Vec;

use h3_contracts::PlayerId;
use h3_game::{Actor, NoopSink, Viewer};
use h3_kernel::rng::{Rng, StreamId};
use h3_sim::adventure::AdventureEnv;
use h3_sim::adventure::rollout::choose;

use super::game::RULES;
use crate::codec;
use crate::map::game::{command, manifest, observe};
use crate::session::game::{GameOptions, GameOutbound, GameSession};
use crate::wire::game::MapIdentity;
use crate::wire::{ServerMessage, WireU64};

/// The most bytes of a snapshot on the fixtures: the largest measured is 431 KB, a fresh one of a
/// player who saw a whole map's 2 523 objects (`e08-xl-144`); a starting one 150 KB.
const SNAPSHOT: usize = 512 << 10;

/// The most bytes of any other frame on the fixtures: the largest measured is 22 KB, a command's
/// events (`e08-xl-144`).
const FRAME: usize = 64 << 10;

/// The players' records' bound: about four of the largest snapshots, so that one with its stretch
/// of half the bound and a command's messages fits.
const RECORD: usize = 2 << 20;

/// The fixtures, with their seeds.
const MAPS: [(&str, &[u8], u64); 8] = [
    (
        "boxes",
        include_bytes!("../../../../fixtures/maps/e08-boxes-144.h3m"),
        1,
    ),
    (
        "dwellings",
        include_bytes!("../../../../fixtures/maps/e08-dwellings-144.h3m"),
        2,
    ),
    (
        "economy",
        include_bytes!("../../../../fixtures/maps/e08-economy-144.h3m"),
        3,
    ),
    (
        "gates",
        include_bytes!("../../../../fixtures/maps/e08-gates-144.h3m"),
        4,
    ),
    (
        "sea",
        include_bytes!("../../../../fixtures/maps/e08-sea-144.h3m"),
        5,
    ),
    (
        "stones",
        include_bytes!("../../../../fixtures/maps/e08-stones-144.h3m"),
        6,
    ),
    (
        "teleporters",
        include_bytes!("../../../../fixtures/maps/e08-teleporters-144.h3m"),
        7,
    ),
    (
        "xl",
        include_bytes!("../../../../fixtures/maps/e08-xl-144.h3m"),
        // A whole map's reveal.
        5,
    ),
];

/// The largest of each kind of frame met.
#[derive(Debug, Default)]
struct Largest {
    snapshot: usize,
    reveal: usize,
    other: usize,
    record: usize,
    reveals: usize,
}

/// Checks the frames a call sent: a reveal against the snapshot its viewer's view now is, the
/// others against their bounds.
fn check(session: &GameSession<'_>, out: &[GameOutbound], at: &str, l: &mut Largest) {
    for GameOutbound { to, frame } in out {
        let len = frame.len();
        match codec::decode_server(frame.as_bytes(), usize::MAX).unwrap() {
            ServerMessage::GameSnapshot { .. } => {
                assert!(len <= SNAPSHOT, "{at}: a snapshot of {len} bytes");
                l.snapshot = l.snapshot.max(len);
            }
            ServerMessage::GameReveal { n, .. } => {
                let observation = observe(session.state(), &RULES, Viewer::Player(*to));
                let snapshot = codec::encode_text(&ServerMessage::GameSnapshot { n, observation });
                assert!(
                    len <= snapshot.len(),
                    "{at}: a reveal of {len} bytes, its snapshot {}",
                    snapshot.len()
                );
                l.reveal = l.reveal.max(len);
                l.reveals += 1;
            }
            _ => {
                assert!(len <= FRAME, "{at}: a frame of {len} bytes: {frame:.200}");
                l.other = l.other.max(len);
            }
        }
    }
}

/// Checks every player's record: it begins with a snapshot and keeps within [`RECORD`].
fn records(session: &GameSession<'_>, at: &str, l: &mut Largest) {
    for p in (0..=u8::MAX).take(session.state().players()) {
        let record = session.record(PlayerId::new(p)).unwrap();
        let mut lines = record.lines().skip(1);
        let first = lines.next().unwrap();
        assert!(first.starts_with(r#"{"GameSnapshot":"#), "{at}");
        let bytes = first.len() + 1 + lines.map(|line| line.len() + 1).sum::<usize>();
        assert!(bytes <= RECORD, "{at}: player {p}'s record {bytes} bytes");
        l.record = l.record.max(bytes);
    }
}

#[test]
fn the_games_frames_and_records_keep_their_bounds_on_the_xl_fixtures() {
    let rules = &*RULES;
    let identity = MapIdentity {
        content_hash: WireU64(1),
        import_version: 1,
    };
    let options = GameOptions {
        game: 1,
        checkpoint_every: 0,
        record_bytes: RECORD,
    };
    let mut l = Largest::default();
    for (name, map, seed) in MAPS {
        let m = h3_map::read(map, &h3_map::Limits::default()).unwrap();
        let setup = h3_map::import(&m, rules).unwrap();
        let mut out = Vec::new();
        let start = manifest(rules, identity);
        let mut session =
            GameSession::start(rules, &setup, seed, start, options, &mut out).unwrap();
        check(&session, &out, name, &mut l);
        let mut env = AdventureEnv::new(rules.clone(), Arc::new(setup), seed, 28).unwrap();
        let mut rng = Rng::for_stream(seed, StreamId::AGENT);
        let mut seq = 0;
        loop {
            let Some(seat) = env.to_move().next() else {
                break;
            };
            let action = choose(&env, seat, &mut rng);
            let (_, played) = env.step_decoded(seat, action, &mut NoopSink).unwrap();
            out.clear();
            match seat.actor() {
                Actor::Player(player) => session.play(player, seq, command(&played), &mut out),
                Actor::System => session.system(command(&played), &mut out),
            }
            .unwrap();
            let at = format!("{name} command {seq}");
            check(&session, &out, &at, &mut l);
            if seq % 100 == 0 {
                records(&session, &at, &mut l);
            }
            seq += 1;
        }
        assert_eq!(session.state().state_hash(), env.state_hash(), "{name}");
        assert!(seq > 100, "{name}: {seq} commands");
        records(&session, name, &mut l);
    }
    // What the fixtures reach: a whole map's reveal, records cut at their snapshots.
    assert!(l.reveal > 256 << 10, "{l:?}");
    assert!(l.reveals > 500, "{l:?}");
    assert!(l.record > RECORD / 2, "{l:?}");
}
