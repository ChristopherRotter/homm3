//! The load mode's saves (E27 M4 T4b1): made over a committed map of the repository's
//! (`fixtures/maps`, as the save's tests), listed from a folder of the test's own, so CI runs
//! them.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

use h3_view::shell::scenario::Format;

use super::*;

/// The committed map the saves are made on.
const DUEL: &str = "e08-duel-36.h3m";

/// An installation of the test's own (`tag` names it), its `Maps` holding [`DUEL`], and an empty
/// saves' folder.
fn dirs(tag: &str) -> (PathBuf, PathBuf) {
    let dir = std::env::temp_dir().join(format!("h3-view-saves-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let maps = dir.join(MAPS);
    fs::create_dir_all(&maps).unwrap();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/maps");
    fs::copy(fixture.join(DUEL), maps.join(DUEL)).unwrap();
    let games = dir.join(GAMES);
    fs::create_dir_all(&games).unwrap();
    (dir, games)
}

/// `path` last written at `time` (seconds from 1970 UTC).
fn touch(path: &Path, time: u64) {
    let file = fs::File::options().write(true).open(path).unwrap();
    file.set_modified(UNIX_EPOCH + Duration::from_secs(time))
        .unwrap();
}

fn broken() -> Broken {
    Broken {
        name: String::from("Broken"),
        description: String::from("Save it again."),
    }
}

/// A save made on the second day ends every player's first turn; it is listed with its map's
/// header, its difficulty and its file's time, beside a file that is not a save (broken) and one
/// of another pattern (left out), in the list's order.
#[test]
fn a_made_save_is_listed_with_its_map_its_difficulty_and_its_time() {
    let rules = crate::maps::rules().unwrap();
    let (install, games) = dirs("listed");
    let save = made(&install, &rules, DUEL, (3, 2), "AUTOSAVE").unwrap();
    assert_eq!(save.name, "AUTOSAVE");
    assert_eq!(save.replay.difficulty, 3);
    assert_eq!(save.replay.final_day, 2);
    assert!(!save.replay.commands.is_empty());
    assert!(
        save.replay
            .commands
            .iter()
            .all(|(_, c)| *c == GameCommand::EndTurn)
    );
    // It loads into its game, on its second day.
    let loaded = crate::save::load(&save, &install, &rules).unwrap();
    assert_eq!(loaded.state.day(), 2);
    assert_eq!(loaded.game.file, DUEL);
    // The single player's colour a human's, the other the computer's.
    let humans = loaded.game.setup.players.iter().filter(|p| p.human).count();
    assert_eq!(humans, 1);
    let path = games.join("AUTOSAVE.GM1");
    crate::save::write_file(&path, &save).unwrap();
    touch(&path, 1_009_838_340);
    fs::write(games.join("junk.gm2"), "not a save").unwrap();
    touch(&games.join("junk.gm2"), 7);
    fs::write(games.join("notes.txt"), "not listed").unwrap();
    fs::create_dir(games.join("dir.gm1")).unwrap();
    let entries = entries(&games, &install, &rules, &broken());
    let files: Vec<&str> = entries.iter().map(|e| e.file.as_str()).collect();
    // Key 2: a broken file's no format before the duel's SoD.
    assert_eq!(files, ["junk.gm2", "AUTOSAVE.GM1"]);
    let [junk, auto] = [&entries[0], &entries[1]];
    assert_eq!(
        auto.saved,
        Some(Saved {
            difficulty: 3,
            time: 1_009_838_340
        })
    );
    let map = crate::maps::entry(
        DUEL.to_owned(),
        &crate::begin::read(&crate::begin::map_path(&install, DUEL)).unwrap(),
        &rules,
    );
    assert_eq!(auto.name, map.name);
    assert_eq!(auto.size, map.size);
    assert_eq!(auto.format, Some(Format::SoD));
    assert_eq!(auto.seats, map.seats);
    assert_eq!(junk.name, "Broken");
    assert_eq!(junk.description, "Save it again.");
    assert_eq!(junk.format, None);
    assert_eq!(
        junk.saved,
        Some(Saved {
            difficulty: 0,
            time: 7
        })
    );
    // No folder: nothing.
    assert!(entries_of(&install.join("none"), &install, &rules).is_empty());
}

fn entries_of(dir: &Path, install: &Path, rules: &GameRuleset) -> Vec<Entry> {
    entries(dir, install, rules, &broken())
}

/// A save whose map is gone, or of other rules, is listed as broken.
#[test]
fn a_save_without_its_map_or_of_other_rules_is_broken() {
    let rules = crate::maps::rules().unwrap();
    let (install, games) = dirs("broken");
    let mut save = made(&install, &rules, DUEL, (0, 1), "small").unwrap();
    assert_eq!(save.replay.commands, []);
    crate::save::write_file(&games.join("small.GM1"), &save).unwrap();
    save.replay.header.profile = String::from("other:profile");
    crate::save::write_file(&games.join("other.GM1"), &save).unwrap();
    let names =
        |entries: &[Entry]| -> Vec<String> { entries.iter().map(|e| e.name.clone()).collect() };
    let listed = entries_of(&games, &install, &rules);
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].file, "other.GM1");
    assert_eq!(listed[0].name, "Broken");
    assert_ne!(listed[1].name, "Broken");
    fs::remove_file(install.join(MAPS).join(DUEL)).unwrap();
    assert_eq!(
        names(&entries_of(&games, &install, &rules)),
        ["Broken", "Broken"]
    );
    // A map that does not read: the made save refused.
    assert!(made(&install, &rules, DUEL, (0, 1), "x").is_err());
}

#[test]
fn the_pattern_is_any_gm_and_one_character() {
    for name in ["a.gm1", "A.GM2", "x.Gmz", "AUTOSAVE.GM1"] {
        assert!(listed(name), "{name}");
    }
    for name in ["a.gm", "a.gm12", "a.h3m", "gm1", "a.cgm", "a.g\u{e9}"] {
        assert!(!listed(name), "{name}");
    }
}
