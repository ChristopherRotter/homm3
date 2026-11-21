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
    let saved = auto.saved.unwrap();
    assert_eq!(
        (saved.difficulty, saved.time, saved.duration),
        (3, 1_009_838_340, UNLIMITED)
    );
    // E27 M4 T6 (CLT-Q166 (b)): the zone's offset at the file's own date.
    assert_eq!(saved.offset, h3_sdl::local_offset_at(1_009_838_340));
    assert!(saved.offset.is_some());
    // E27 M4 T5b1: the single player's colour the save's one human.
    assert_eq!(saved.humans.iter().filter(|h| **h).count(), 1);
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
            time: 7,
            offset: h3_sdl::local_offset_at(7),
            ..Saved::default()
        })
    );
    // No folder: nothing, nothing told (E27 M4 T6).
    assert!(entries_of(&install.join("none"), &install, &rules).is_empty());
    assert_eq!(
        listing(&install.join("none"), &install, &rules, &broken()),
        Listing::default()
    );
    // E27 M4 T6 (the review's B5): a folder that does not read (a file in its place) lists
    // nothing and says why.
    let unreadable = listing(&path, &install, &rules, &broken());
    assert!(unreadable.entries.is_empty());
    assert_eq!(unreadable.unread.len(), 1);
    assert!(
        unreadable.unread[0].starts_with(&format!("{}: ", path.display())),
        "{unreadable:?}"
    );
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
    // E27 M4 T6 (the review's B7): why a save is listed broken, told once a listing.
    let told = listing(&games, &install, &rules, &broken()).unread;
    assert_eq!(told.len(), 1, "{told:?}");
    let other = games.join("other.GM1");
    assert!(
        told[0].starts_with(&format!("{}: listed broken: ", other.display())),
        "{told:?}"
    );
    // B3: a file past a save's most is listed broken by its size, not read (a sparse file).
    let huge = games.join("huge.GM1");
    fs::File::create(&huge)
        .unwrap()
        .set_len(crate::save::MOST_BYTES + 1)
        .unwrap();
    let all = listing(&games, &install, &rules, &broken());
    assert_eq!(all.entries.len(), 3);
    assert!(
        all.unread
            .iter()
            .any(|l| l.contains("huge.GM1: ") && l.contains("bytes, past")),
        "{:?}",
        all.unread
    );
    fs::remove_file(&huge).unwrap();
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

#[test]
fn delete_removes_a_listed_file_of_the_folder_alone() {
    // E27 M4 T4b2: Delete's OK.
    let (dir, games) = dirs("delete");
    fs::write(games.join("one.GM1"), "x").unwrap();
    fs::write(dir.join("out.GM1"), "x").unwrap();
    fs::create_dir_all(games.join("folder.GM1")).unwrap();
    fs::write(games.join("notes.txt"), "x").unwrap();
    assert_eq!(delete(&games, "one.GM1"), Ok(()));
    assert!(!games.join("one.GM1").exists());
    // Gone, a path out of the folder, a folder, another pattern: refused, nothing deleted.
    assert!(delete(&games, "one.GM1").is_err());
    assert!(delete(&games, "../out.GM1").is_err());
    assert!(dir.join("out.GM1").exists());
    assert!(delete(&games, "folder.GM1").is_err());
    assert!(games.join("folder.GM1").is_dir());
    assert!(delete(&games, "notes.txt").is_err());
    assert!(delete(&games, "").is_err());
    let _ = fs::remove_dir_all(&dir);
}

/// E27 M4 T5b1: a hot seat's made save holds its players as humans with their names and choices
/// and the turn's duration; its entry shows them (the hot seat's load's rows).
#[test]
fn a_hot_seat_save_holds_its_players_choices_and_duration() {
    let rules = crate::maps::rules().unwrap();
    let (install, games) = dirs("hot-seat");
    let seated = Seated {
        players: vec![
            Player::parse("0:A:random:random:gold").unwrap(),
            Player::parse("1:B:random:random:resource:1").unwrap(),
        ],
        duration: 7,
    };
    let save = made_with(&install, &rules, DUEL, (4, 1), &seated, "Pair").unwrap();
    assert_eq!(save.duration, 7);
    assert_eq!(save.names[0].as_deref(), Some("A"));
    assert_eq!(save.names[1].as_deref(), Some("B"));
    assert_eq!(save.bonuses[..2], [Bonus::Gold, Bonus::Resource]);
    // E27 M4 T6 (the review's C9): B's handicap written into the save.
    assert_eq!(save.handicaps[..2], [0, 1]);
    let loaded = crate::save::load(&save, &install, &rules).unwrap();
    assert_eq!(
        loaded.game.setup.players.iter().filter(|p| p.human).count(),
        2
    );
    // E27 M4 T6 (the review's item 38): the game's draws at the save, as the loaded state tells
    // them (the duel's heroes placed, no towns: none drawn).
    assert_eq!(
        save.drawn,
        crate::save::drawn(&loaded.game.setup, &loaded.state)
    );
    // E27 M4 T6 (CLT-Q170 (a)): a single load plays it on alone, the colour whose turn it is
    // (red's, the first day's first) the only human; the hot seat's load hands its names on.
    assert_eq!(loaded.humans(None), Some(crate::save::Humans::Alone(0)));
    let names = save.names.clone();
    assert_eq!(
        loaded.humans(Some(names.clone())),
        Some(crate::save::Humans::Named(names))
    );
    let path = games.join("Pair.GM2");
    crate::save::write_file(&path, &save).unwrap();
    let entries = entries(&games, &install, &rules, &broken());
    let listed = entries[0].saved.unwrap();
    assert_eq!(listed.difficulty, 4);
    assert_eq!(listed.duration, 7);
    assert_eq!(listed.humans[..3], [true, true, false]);
    assert_eq!(listed.records[0].bonus, Bonus::Gold);
    assert_eq!(listed.records[1].bonus, Bonus::Resource);
    assert!(listed.records.iter().all(|r| r.colour.is_none()));
    assert_eq!(listed.records.map(|r| r.handicap)[..3], [0, 1, 0]);
    // The Random choices shown as the game drew them (the original's load shows the game's).
    for c in 0..2 {
        let (record, drawn) = (listed.records[c], save.drawn[c]);
        assert_eq!((record.town, record.hero), (drawn.town, drawn.hero), "{c}");
    }
    // The table's choices and the replay's are what `saved` reads.
    let mut fixed = save.clone();
    fixed.replay.players[0].faction = Some(FactionChoice::Fixed(
        h3_content::FactionId::from_index(3).unwrap(),
    ));
    fixed.replay.players[0].hero = h3_content::HeroTypeId::from_index(50);
    fixed.replay.players[1].faction = Some(FactionChoice::Random);
    fixed.replay.players[1].colour = 9;
    fixed.handicaps[0] = 2;
    let shown = saved(&fixed);
    assert_eq!(
        (shown.records[0].town, shown.records[0].hero),
        (Some(3), Some(50))
    );
    // Without the draws (a save that does not tell): Random.
    let mut untold = save.clone().with_drawn(Default::default());
    untold.replay.players[0].faction = None;
    let shown_untold = saved(&untold);
    assert_eq!(
        (shown_untold.records[0].town, shown_untold.records[0].hero),
        (None, None)
    );
    assert_eq!(shown.records[0].handicap, 2);
    // A player of a colour past the eight is passed over.
    assert!(!shown.humans[1]);
    // Two players of one colour are refused.
    let twice = Seated {
        players: vec![
            Player::parse("0:A:random:random:gold").unwrap(),
            Player::parse("0:B:random:random:gold").unwrap(),
        ],
        duration: UNLIMITED,
    };
    assert!(made_with(&install, &rules, DUEL, (0, 1), &twice, "x").is_err());
    let _ = fs::remove_dir_all(&install);
}

#[test]
fn a_hot_seat_player_parses_from_its_five_fields() {
    let p = Player::parse("5:C:2:32:artifact").unwrap();
    assert_eq!(
        p,
        Player {
            colour: 5,
            name: String::from("C"),
            town: Some(2),
            hero: Some(32),
            bonus: Bonus::Artifact,
            handicap: 0,
        }
    );
    // E27 M4 T6 (the review's C9): a sixth field, the handicap (0 to 2).
    assert_eq!(Player::parse("5:C:2:32:artifact:2").unwrap().handicap, 2);
    assert_eq!(Player::parse("5:C:2:32:artifact:0").unwrap(), p);
    let random = Player::parse("7:D:random:random:random").unwrap();
    assert_eq!(
        (random.town, random.hero, random.bonus),
        (None, None, Bonus::Random)
    );
    for bad in [
        "8:A:random:random:gold",
        "x:A:random:random:gold",
        "0:A:random:random",
        "0:A:random:random:gold:x",
        "0:A:random:random:gold:3",
        "0:A:random:random:gold:1:x",
        "0:A:t:random:gold",
        "0:A:random:300:gold",
        "0:A:random:random:coins",
    ] {
        assert!(Player::parse(bad).is_err(), "{bad}");
    }
}

#[test]
fn the_save_command_takes_a_hot_seat_s_players_and_duration() {
    let args = |a: &[&str]| -> Vec<String> { a.iter().map(|s| (*s).to_owned()).collect() };
    let base = [
        "--map",
        DUEL,
        "--difficulty",
        "2",
        "--name",
        "n",
        "--out",
        "o",
        "--install",
        "i",
    ];
    let single = Options::parse(&args(&base)).unwrap();
    assert_eq!(single.seated, Seated::default());
    let mut more = base.to_vec();
    more.extend([
        "--player",
        "0:A:random:50:gold",
        "--player",
        "3:B:3:51:artifact",
    ]);
    more.extend(["--duration", "7"]);
    let hot = Options::parse(&args(&more)).unwrap();
    assert_eq!(hot.seated.players.len(), 2);
    assert_eq!(hot.seated.players[1].town, Some(3));
    assert_eq!(hot.seated.duration, 7);
    for bad in [
        &["--duration", "11"][..],
        &["--duration", "x"],
        &["--player", "9:A:1:1:gold"],
    ] {
        let mut wrong = base.to_vec();
        wrong.extend_from_slice(bad);
        assert!(Options::parse(&args(&wrong)).is_err(), "{bad:?}");
    }
}

/// E27 M4 T6 (the review's item 38): a made save holds each colour's town and first hero as the
/// game drew them: over `e09-towns-36.h3m` red's Castle and its random hero's drawn type (hero 3,
/// the first random hero, takes the hero choice), blue's Stronghold and no hero (none random,
/// none generated); the save's file keeps them.
#[test]
fn a_made_save_holds_the_game_s_draws() {
    let rules = crate::maps::rules().unwrap();
    let (install, games) = dirs("drawn");
    let towns = "e09-towns-36.h3m";
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/maps");
    fs::copy(fixture.join(towns), install.join(MAPS).join(towns)).unwrap();
    let save = made(&install, &rules, towns, (0, 1), "Towns").unwrap();
    let loaded = crate::save::load(&save, &install, &rules).unwrap();
    let hero3 = loaded
        .state
        .hero(h3_contracts::HeroId::new(3))
        .and_then(|d| u8::try_from(d.hero_type.index()).ok());
    assert!(hero3.is_some());
    assert_eq!(
        save.drawn[..2],
        [
            crate::save::Drawn {
                town: Some(0),
                hero: hero3,
            },
            crate::save::Drawn {
                town: Some(6),
                hero: None,
            },
        ]
    );
    assert!(
        save.drawn[2..]
            .iter()
            .all(|d| *d == crate::save::Drawn::default())
    );
    let path = games.join("Towns.GM1");
    crate::save::write_file(&path, &save).unwrap();
    assert_eq!(crate::save::read_file(&path).unwrap(), save);
    let listed = entries(&games, &install, &rules, &broken())[0]
        .saved
        .unwrap();
    assert_eq!(
        (listed.records[0].town, listed.records[0].hero),
        (Some(0), hero3)
    );
    let _ = fs::remove_dir_all(&install);
}
