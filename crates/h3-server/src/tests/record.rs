//! `h3-server record` (E16 M5 T6): every golden adventure replay runs through the whole game's
//! session to its checkpoints and final hash, and each player's record is its header and the
//! messages it was sent: one `GameEvents` per command whatever the player may see, its own
//! commands echoed, no state hash anywhere (what each message holds is projected for its viewer
//! by construction, gated in `h3-protocol`'s `game_session` test). A replay that is not its re-run
//! records nothing; the files are written per player.

use std::fs;
use std::path::{Path, PathBuf};

use h3_contracts::PlayerId;
use h3_game::Actor;
use h3_protocol::RECORD_FORMAT_VERSION;
use h3_protocol::codec;
use h3_protocol::wire::game::{GameRecordHeader, MapIdentity};
use h3_protocol::wire::{ServerMessage, WireU64};
use h3_sim::adventure::replay::Replay;

use crate::game;
use crate::record::{RecordOptions, Recorded, record, record_replay};

/// The workspace's root.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The golden adventure replays, by name.
fn golden() -> Vec<PathBuf> {
    let dir = root().join("fixtures/replays/adventure");
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|e| e == "toml"))
        .collect();
    files.sort();
    files
}

/// The smallest golden replay (8 commands, two players, a checkpoint).
fn small() -> PathBuf {
    root().join("fixtures/replays/adventure/army-dialog-s1.toml")
}

fn recorded(file: &Path, game: u32) -> Result<Recorded, String> {
    record_replay(file, &root().join("packs"), &root(), game).map_err(|e| e.0)
}

/// A fresh directory of its own.
fn scratch(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("h3-record-{name}-{:016x}", game::os_u64().unwrap()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// A message's number.
fn number(line: &str) -> u64 {
    let value: serde_json::Value = serde_json::from_str(line).unwrap();
    let (_, body) = value.as_object().unwrap().iter().next().unwrap();
    body["n"].as_u64().unwrap()
}

/// Checks player `k`'s record of `replay`, recorded as game `game`.
fn check(name: &str, replay: &Replay, game: u32, k: usize, text: &str) {
    let mut lines = text.lines();
    let header: GameRecordHeader =
        codec::decode(lines.next().unwrap().as_bytes(), usize::MAX).unwrap();
    assert_eq!(header.format, RECORD_FORMAT_VERSION, "{name}");
    assert_eq!(header.game, game);
    assert_eq!(usize::from(header.player), k, "{name}");
    let map = MapIdentity {
        content_hash: WireU64(replay.map.content_hash),
        import_version: replay.map.import_version,
    };
    assert_eq!(header.manifest.map, map, "{name}");
    assert_eq!(header.first, 1, "{name}: every message kept");
    let (mut events, mut seqs) = (0, Vec::new());
    for (i, line) in lines.enumerate() {
        let message = codec::decode_server(line.as_bytes(), usize::MAX).unwrap();
        assert_eq!(number(line), i as u64 + 1, "{name} player {k}: numbered on");
        match message {
            ServerMessage::GameSnapshot { .. } => assert_eq!(i, 0, "{name}: the snapshot first"),
            ServerMessage::GameEvents { .. } => events += 1,
            ServerMessage::GameAccepted { seq, .. } => seqs.push(seq),
            ServerMessage::GameTurn { .. } | ServerMessage::Snapshot { .. } => {}
            other => panic!("{name} player {k}: {other:?}"),
        }
    }
    // Every command tells every player its events, also none: how many a player gets does not
    // depend on what it may see.
    assert_eq!(events, replay.commands.len(), "{name} player {k}");
    // Its own commands, echoed and numbered by its own counter.
    let own = replay
        .commands
        .iter()
        .filter(|(actor, _)| *actor == Actor::Player(PlayerId::new(header.player)))
        .count();
    let expected: Vec<u32> = (1..).take(own).collect();
    assert_eq!(seqs, expected, "{name} player {k}");
    // No state hash: they hold the whole state, what the player may not see too.
    for hash in replay.checkpoints.iter().chain([&replay.final_hash]) {
        for form in [
            format!("{hash}"),
            format!("{hash:x}"),
            format!("{hash:016x}"),
        ] {
            assert!(
                !text.contains(&form),
                "{name} player {k}: a state hash {form}"
            );
        }
    }
}

#[test]
fn the_golden_replays_record_through_the_session() {
    let files = golden();
    assert_eq!(files.len(), 66, "the golden adventure replays");
    let threads = std::thread::available_parallelism().map_or(1, usize::from);
    let chunk = files.len().div_ceil(threads);
    let players: usize = std::thread::scope(|scope| {
        let runs: Vec<_> = files
            .chunks(chunk)
            .map(|files| {
                scope.spawn(move || {
                    let mut players = 0;
                    for file in files {
                        let name = file.file_stem().unwrap().to_string_lossy().into_owned();
                        let text = fs::read_to_string(file).unwrap();
                        let replay = h3_files::adventure::parse_replay(&text).unwrap();
                        let recorded = recorded(file, 7).unwrap_or_else(|e| panic!("{e}"));
                        assert_eq!(recorded.map, replay.map.path, "{name}");
                        assert_eq!(recorded.commands, replay.commands.len(), "{name}");
                        assert_eq!(recorded.day, replay.final_day, "{name}");
                        assert_eq!(recorded.final_hash, replay.final_hash, "{name}");
                        assert!(recorded.records.len() >= 2, "{name}");
                        for (k, text) in recorded.records.iter().enumerate() {
                            check(&name, &replay, 7, k, text);
                        }
                        players += recorded.records.len();
                    }
                    players
                })
            })
            .collect();
        runs.into_iter().map(|run| run.join().unwrap()).sum()
    });
    assert!(players >= 110, "{players} records");
}

#[test]
fn a_record_is_the_same_on_every_run() {
    let first = recorded(&small(), 3).unwrap();
    assert_eq!(recorded(&small(), 3).unwrap(), first);
    assert_eq!(first.records.len(), 2);
    assert_ne!(first.records[0], first.records[1], "each player's own");
    // The game's id is the header's alone.
    let other = recorded(&small(), 4).unwrap();
    for (a, b) in first.records.iter().zip(&other.records) {
        assert_ne!(a, b);
        assert_eq!(a.split_once('\n').unwrap().1, b.split_once('\n').unwrap().1);
    }
}

#[test]
fn a_replay_that_is_not_its_rerun_records_nothing() {
    let dir = scratch("broken");
    let text = fs::read_to_string(small()).unwrap();
    let broken = |name: &str, from: &str, to: &str| {
        assert!(text.contains(from), "{name}: `{from}`");
        let file = dir.join(format!("{name}.toml"));
        fs::write(&file, text.replacen(from, to, 1)).unwrap();
        recorded(&file, 0).unwrap_err()
    };
    let cases = [
        (
            "checkpoint",
            "0x9d60dde7cc4d591f\"]",
            "0x9d60dde7cc4d5910\"]",
            "checkpoint 0 after command 4: the replay's 0x9d60dde7cc4d5910, the session's 0x9d60dde7cc4d591f",
        ),
        (
            "final-hash",
            "final_hash = \"0x033379ef1388bfae\"",
            "final_hash = \"0x033379ef1388bfa0\"",
            "state hash after 8 commands: expected 0x033379ef1388bfa0, got 0x033379ef1388bfae",
        ),
        ("day", "day = 2", "day = 3", "final day: expected 3, got 2"),
        (
            "map",
            "hash = \"0x40320993e64e5b23\"",
            "hash = \"0x40320993e64e5b20\"",
            "replay header mismatch: map.hash",
        ),
        (
            "import",
            "import_version = 21",
            "import_version = 20",
            "replay header mismatch: map.import_version",
        ),
        (
            "rules",
            "ruleset_hash = \"0xe5ac723212bdce02\"",
            "ruleset_hash = \"0xe5ac723212bdce00\"",
            "replay header mismatch: ruleset_hash",
        ),
        (
            "checkpoints",
            "checkpoints = [\"0x9d60dde7cc4d591f\"]",
            "checkpoints = []",
            "checkpoint count does not match the command count",
        ),
        (
            "rejected",
            "\"0 move 0 2,5,0\"",
            "\"1 move 0 2,5,0\"",
            "command 0 rejected: ",
        ),
    ];
    for (name, from, to, error) in cases {
        let got = broken(name, from, to);
        assert!(got.contains(error), "{name}: {got}");
        assert!(got.starts_with(&dir.display().to_string()), "{name}: {got}");
    }
    let missing = recorded(&dir.join("missing.toml"), 0).unwrap_err();
    assert!(missing.contains("missing.toml"), "{missing}");
    fs::write(dir.join("text.toml"), "not a replay").unwrap();
    assert!(recorded(&dir.join("text.toml"), 0).is_err());
    let packs = record_replay(&small(), &dir, &root(), 0).unwrap_err().0;
    assert!(packs.contains("army-dialog-s1.toml"), "{packs}");
    // A map below another root is not found.
    let elsewhere = record_replay(&small(), &root().join("packs"), &dir, 0)
        .unwrap_err()
        .0;
    assert!(elsewhere.contains("e08-army-12.h3m"), "{elsewhere}");
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn the_options_parse() {
    let words = |text: &str| -> Vec<String> { text.split(' ').map(str::to_owned).collect() };
    let options = RecordOptions::parse(&words("a.toml")).unwrap();
    assert_eq!(
        options,
        RecordOptions {
            replays: vec![PathBuf::from("a.toml")],
            packs_dir: PathBuf::from("packs"),
            root: PathBuf::from("."),
            out: PathBuf::from("."),
            game: 0,
        }
    );
    let options = RecordOptions::parse(&words(
        "--packs-dir p --root r a.toml --out o --game 12 b.toml",
    ))
    .unwrap();
    assert_eq!(
        options,
        RecordOptions {
            replays: vec![PathBuf::from("a.toml"), PathBuf::from("b.toml")],
            packs_dir: PathBuf::from("p"),
            root: PathBuf::from("r"),
            out: PathBuf::from("o"),
            game: 12,
        }
    );
    for (args, error) in [
        ("", "no replay given"),
        ("--out", "`--out` needs a value"),
        ("--game x a.toml", "bad game id `x`"),
        ("--seed 1 a.toml", "unknown option `--seed`"),
    ] {
        let args = if args.is_empty() {
            Vec::new()
        } else {
            words(args)
        };
        assert_eq!(RecordOptions::parse(&args).unwrap_err().0, error);
    }
}

#[test]
fn each_players_record_is_written_to_its_file() {
    let out = scratch("out");
    let options = RecordOptions {
        replays: vec![small()],
        packs_dir: root().join("packs"),
        root: root(),
        out: out.clone(),
        game: 5,
    };
    let lines = record(&options).unwrap();
    let expected = recorded(&small(), 5).unwrap();
    let files: Vec<PathBuf> = (0..2)
        .map(|k| out.join(format!("army-dialog-s1.player-{k}.jsonl")))
        .collect();
    for (file, text) in files.iter().zip(&expected.records) {
        assert_eq!(fs::read_to_string(file).unwrap(), *text);
    }
    assert_eq!(lines.len(), 1);
    assert!(
        lines[0].ends_with(&format!(
            ": fixtures/maps/e08-army-12.h3m (8 commands, day 2, final hash 0x033379ef1388bfae): {}, {}",
            files[0].display(),
            files[1].display()
        )),
        "{}",
        lines[0]
    );
    // A directory that is not there writes nothing.
    let options = RecordOptions {
        out: out.join("missing"),
        ..options
    };
    assert!(record(&options).unwrap_err().0.contains("missing"));
    fs::remove_dir_all(&out).unwrap();
}
