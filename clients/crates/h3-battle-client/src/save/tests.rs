//! The saved game's tests (E27 M4 T3b): over a committed map of the repository's
//! (`fixtures/maps`) copied into an installation's `Maps` of the test's own, so CI runs them.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use h3_map::writer as w;
use h3_sim::adventure::replay::{RecordOptions, Replay};
use h3_view::shell::scenario::{Begun, Bonus, Record};

use super::*;
use crate::begin;

/// The committed map the saves are made on.
const DUEL: &str = "e08-duel-36.h3m";

const NONE: Record = Record {
    colour: None,
    town: None,
    hero: None,
    bonus: Bonus::Random,
    handicap: 0,
};

/// An installation of the test's own (`tag` names it): its `Maps` holding the committed map
/// [`DUEL`].
fn install(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("h3-view-save-{tag}-{}", std::process::id()));
    let maps = dir.join(crate::maps::MAPS);
    fs::create_dir_all(&maps).unwrap();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/maps");
    fs::copy(fixture.join(DUEL), maps.join(DUEL)).unwrap();
    dir
}

/// The game begun on [`DUEL`]: the import's first human-playable colour a human's (its name with
/// a quote, a backslash and a control character, gold, a mild handicap), the other colour the
/// computer's (the import's human flag cleared: a choice the import alone does not make), at
/// Hard, the duration's record 4.
fn begun(install: &Path, rules: &GameRuleset) -> Game {
    let map = begin::read(&begin::map_path(install, DUEL)).unwrap();
    let import = h3_map::import(&map, rules).unwrap();
    let first = import.players.iter().find(|p| p.human).unwrap().colour;
    assert!(
        import.players.iter().any(|p| p.colour != first && p.human),
        "the duel's two colours may both be human"
    );
    let mut records = [NONE; 8];
    records[usize::from(first)] = Record {
        colour: Some(first),
        bonus: Bonus::Gold,
        handicap: 1,
        ..NONE
    };
    let mut names: [Option<String>; 8] = Default::default();
    names[usize::from(first)] = Some(String::from("Ann \"the\" \\ \u{1} Bold"));
    let begun = Begun {
        file: String::from(DUEL),
        difficulty: 3,
        records,
        names,
        duration: 4,
    };
    let setup = begin::setup(&map, &begun, rules).unwrap();
    begin::validated(&setup, rules).unwrap();
    assert_ne!(setup.players, import.players);
    begin::game(setup, &begun)
}

/// The engine's record of `game` on the installation's map: the random agent's 60 commands for
/// every seat, a checkpoint every 10.
fn record(install: &Path, game: &Game, rules: &GameRuleset) -> Replay {
    let options = RecordOptions {
        seed: 7,
        agent_seed: 3,
        max_days: 0,
        max_steps: 60,
        checkpoint_every: 10,
    };
    let map = map(install, &game.file).unwrap();
    Replay::record_random(rules, Arc::new(game.setup.clone()), map, options).unwrap()
}

/// A save of [`begun`]'s game, its rules and installation.
fn made(tag: &str) -> (Save, GameRuleset, PathBuf, Game) {
    let rules = crate::maps::rules().unwrap();
    let install = install(tag);
    let game = begun(&install, &rules);
    let replay = record(&install, &game, &rules);
    (Save::of("Week one", &game, replay), rules, install, game)
}

/// A save is written as the engine's replay file and the client's table, read back the same,
/// and loaded into the game it was made of: the setup rebuilt with its choices, the client's
/// table, the state after its commands; the file written whole and read again.
#[test]
fn a_save_writes_reads_and_loads_its_game() {
    let (save, rules, install, game) = made("round");
    assert_eq!(save.replay.map.path, DUEL);
    assert_eq!(save.replay.commands.len(), 60);
    let text = save.write().unwrap();
    // The engine's file whole, then the table.
    let at = table_start(&text).unwrap();
    let replay = h3_files::adventure::replay_to_toml(&save.replay).unwrap();
    assert_eq!(
        text[..at].trim_end(),
        format!("{HEADER}{replay}").trim_end()
    );
    assert!(
        text[at..].starts_with("[save]\nversion = 1\nname = \"Week one\"\nduration = 4\n"),
        "{}",
        &text[at..]
    );
    assert!(
        text.contains(
            "name = \"Ann \\\"the\\\" \\\\ \\u0001 Bold\"\nbonus = \"gold\"\nhandicap = 1\n"
        ),
        "{}",
        &text[at..]
    );
    let read = Save::read(&text).unwrap();
    assert_eq!(read, save);
    // Line ends of another system read the same.
    assert_eq!(Save::read(&text.replace('\n', "\r\n")).unwrap(), save);
    let loaded = load(&read, &install, &rules).unwrap();
    assert_eq!(loaded.game, game);
    assert_eq!(loaded.state.state_hash(), save.replay.final_hash);
    assert_eq!(loaded.state.day(), save.replay.final_day);
    // The file, written whole (no sibling left) and read again.
    let path = install.join("week-one.toml");
    write_file(&path, &save).unwrap();
    assert!(!install.join("week-one.toml.tmp").exists());
    assert_eq!(read_file(&path).unwrap(), save);
    let missing = read_file(&install.join("missing.toml")).unwrap_err();
    assert!(missing.contains("missing.toml"), "{missing}");
    let unwritable = write_file(&install.join("no-folder/save.toml"), &save).unwrap_err();
    assert!(unwritable.contains("no-folder"), "{unwritable}");
    fs::write(&path, "not a save").unwrap();
    let refused = read_file(&path).unwrap_err();
    assert!(
        refused.contains("week-one.toml: no line `[save]`"),
        "{refused}"
    );
    fs::remove_dir_all(install).unwrap();
}

/// The client's table is read strictly: its line, its version, its keys, eight colours, the
/// bonuses' words, the handicaps' and the duration's ranges; the replay's part as the engine's
/// reader takes it.
#[test]
fn a_save_s_table_is_read_strictly() {
    let (save, _, install, _) = made("strict");
    let text = save.write().unwrap();
    let at = table_start(&text).unwrap();
    let refused = |text: &str, part: &str| {
        let e = Save::read(text).unwrap_err();
        assert!(e.contains(part), "{part}: {e}");
    };
    refused(&text[..at], "no line `[save]`");
    refused(
        &text.replace("version = 1\n", "version = 2\n"),
        "version 2: this client",
    );
    refused(&text.replace("version = 1\n", ""), "no `version` number");
    refused(
        &text.replace("version = 1\n", "version = \"1\"\n"),
        "no `version`",
    );
    refused(&format!("{text}level = 3\n"), "level");
    refused(&format!("{text}\n[other]\n"), "other");
    refused(&format!("{text}name = \n"), "[save]");
    refused(
        &text.replace("duration = 4\n", "duration = 11\n"),
        "duration 11",
    );
    refused(
        &text.replace("bonus = \"gold\"", "bonus = \"silver\""),
        "bonus `silver`",
    );
    refused(
        &text.replace("handicap = 1\n", "handicap = 3\n"),
        "handicap 3",
    );
    let seven = text.rfind("\n[[save.colours]]").unwrap();
    refused(&text[..seven], "7 colours, not 8");
    // The replay's part is the engine's file: its keys and words as its reader takes them.
    refused(
        &text.replacen("format = 20", "format = \"20\"", 1),
        "replay format",
    );
    refused(
        &text.replacen("[map]", "[save.colours]\n[map]", 1),
        "replay format",
    );
    // A duration at unlimited's reads.
    let unlimited = text.replace("duration = 4\n", "duration = 10\n");
    assert_eq!(Save::read(&unlimited).unwrap().duration, UNLIMITED);
    fs::remove_dir_all(install).unwrap();
}

/// The load refuses a save whose replay does not fit: other rules, a map missing, outside `Maps`,
/// changed or of another import, a choice the map or the engine refuses, a name for a
/// computer's colour, a command the engine rejects, a hash, the checkpoints or the day.
#[test]
fn a_load_refuses_a_save_that_does_not_fit() {
    let (save, rules, install, _) = made("refused");
    let refused = |edit: &dyn Fn(&mut Save), part: &str| {
        let mut save = save.clone();
        edit(&mut save);
        let e = load(&save, &install, &rules).unwrap_err();
        assert!(e.contains(part), "{part}: {e}");
    };
    refused(&|s| s.replay.header.ruleset_hash ^= 1, "ruleset_hash");
    refused(
        &|s| s.replay.map.path = String::from("missing.h3m"),
        "missing.h3m",
    );
    refused(
        &|s| s.replay.map.path = format!("../{DUEL}"),
        "must be relative",
    );
    refused(&|s| s.replay.map.content_hash ^= 1, "map.hash");
    refused(&|s| s.replay.map.import_version += 1, "map.import_version");
    let other = save
        .replay
        .players
        .iter()
        .find(|p| !p.human)
        .unwrap()
        .colour;
    refused(
        &|s| s.replay.players.push(s.replay.players[0]),
        "mismatch: players",
    );
    refused(
        &|s| s.names[usize::from(other)] = Some(String::from("B")),
        &format!("names colour {other}"),
    );
    refused(&|s| s.replay.difficulty = 9, "setup:");
    refused(
        &|s| {
            let (_, command) = s.replay.commands[0].clone();
            let wrong = save
                .replay
                .commands
                .iter()
                .map(|(actor, _)| *actor)
                .find(|actor| *actor != s.replay.commands[0].0)
                .unwrap();
            s.replay.commands[0] = (wrong, command);
        },
        "command 0 rejected",
    );
    refused(&|s| s.replay.checkpoints[2] ^= 1, "after 30 commands");
    refused(
        &|s| {
            s.replay.checkpoints.pop();
        },
        "checkpoint count",
    );
    refused(&|s| s.replay.final_hash ^= 1, "after 60 commands");
    refused(&|s| s.replay.final_day += 1, "final day");
    // The map's file changed under the save.
    let path = install.join(crate::maps::MAPS).join(DUEL);
    fs::write(&path, b"another map").unwrap();
    refused(&|_| {}, "map.hash");
    // A map whose second colour the computer's alone: a save making it human is refused before
    // its commands run (as the begin refuses the record, E27 M3 T7).
    let mut players: [Option<w::Player>; w::PLAYERS] = Default::default();
    players[0] = Some(w::Player {
        human: true,
        computer: true,
        towns: 0x1FF,
    });
    players[1] = Some(w::Player {
        human: false,
        computer: true,
        towns: 0x1FF,
    });
    let alone = w::Map {
        name: String::from("Alone"),
        description: String::new(),
        size: 36,
        terrain: w::Terrain::Grass,
        patches: Vec::new(),
        roads: Vec::new(),
        players,
        objects: Vec::new(),
        underground: None,
        level_limit: 0,
        banned_skills: Vec::new(),
        banned_artifacts: Vec::new(),
        events: Vec::new(),
        teams: None,
    };
    let file = "alone.h3m";
    fs::write(
        install.join(crate::maps::MAPS).join(file),
        w::gzip(&w::write(&alone).unwrap()),
    )
    .unwrap();
    let map = map(&install, file).unwrap();
    let import = h3_map::import(
        &begin::read(&begin::map_path(&install, file)).unwrap(),
        &rules,
    )
    .unwrap();
    assert!(import.players.iter().any(|p| p.colour == 1 && !p.human));
    refused(
        &|s| {
            s.replay.map = map.clone();
            s.replay.players = vec![h3_sim::adventure::replay::PlayerChoices {
                colour: 1,
                human: true,
                faction: None,
                hero: None,
            }];
            s.names = Default::default();
        },
        "makes colour 1 human, which the map gives the computer alone",
    );
    fs::remove_dir_all(install).unwrap();
}
