//! The load mode's saves (E27 M4 T4b1; `docs/epics/E27-design.md` §3.3, §37, §39): the client's
//! saved games ([`crate::save`]) in its own folder, listed with their map's header, as the
//! original lists its `games` folder's `*.gm?` (`scenario_list_build` 0x582B40; never the
//! original's folder: decision 8).
//!
//! - **The folder** (*provisional*, the task's call): `games` in the working directory, beside the
//!   settings', the keys' and the scores' files ([`GAMES`]; the window's `--games` names another);
//!   a save's file `<name>.GM<n>` as the original's (`game_save_dialog` 0x418160: `.GM` and the
//!   count of humans still playing), so the list's rows show the same names. Up to 5 000 files of
//!   the pattern `*.gm?` (any case), as the original's search, read in the order of their names.
//! - **An entry** ([`entry`]): the map the save's replay names, read from the installation's `Maps`
//!   as a new game's list reads it (`crate::maps::entry`: its header and heroes), the save's file's
//!   name, its difficulty and its file's last-write time ([`Saved`]); for the hot seat's load (E27
//!   M4 T5b1) the colours it holds as human, each colour's choices (the replay's players' faction
//!   and hero, the table's bonus and handicap) and the turn's duration ([`saved`]).
//! - **A save the client cannot list** (the task's call): a file that does not read as a save,
//!   whose replay's header the rules refuse (other packs, profile, rules' hashes or state hash
//!   versions: `ReplayHeader::check`), or whose map is missing or moved (its content hash or
//!   import version): listed as the original lists a broken file (§3.3: a save of another version,
//!   GENRLTXT 431 its name and 505 its description), with no map's columns. The full load (the
//!   commands replayed, `save::load`) is Load's (T4b2).
//! - **Delete** (E27 M4 T4b2, [`delete`]): a listed file of the folder alone, by its plain name
//!   (the original's `DeleteFileA` in `games`, 0x58E08B).

use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use h3_contracts::PlayerId;
use h3_game::{Actor, FactionChoice, GameCommand, GameRuleset, GameState, NoopSink, apply};
use h3_map::Limits;
use h3_sim::adventure::replay::Replay;
use h3_view::shell::scenario::{self, Begun, Bonus, Entry, Heroes, Record, Saved, Seat, UNLIMITED};

use crate::maps::MAPS;

/// The window's saves' folder in the working directory (*provisional*).
pub const GAMES: &str = "games";

/// The most files listed (`scenario_list_build` 0x582B40: 5 000).
pub const MOST: usize = 5_000;

/// What a save the client cannot list shows (GENRLTXT 431 and 505).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Broken {
    /// Its name (GENRLTXT 431).
    pub name: String,
    /// Its description (GENRLTXT 505).
    pub description: String,
}

/// The entries of the saves in `dir` (none without the folder) whose maps `install`'s `Maps`
/// holds, imported under `rules`, in the list's order ([`scenario::sort`]); one the client cannot
/// list shows `broken`.
#[must_use]
pub fn entries(dir: &Path, install: &Path, rules: &GameRuleset, broken: &Broken) -> Vec<Entry> {
    let Ok(read) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<(String, PathBuf)> = read
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .filter_map(|e| Some((e.file_name().to_str()?.to_owned(), e.path())))
        .filter(|(name, _)| listed(name))
        .collect();
    files.sort();
    files.truncate(MOST);
    let mut entries: Vec<Entry> = files
        .into_iter()
        .map(|(name, path)| entry(name, &path, install, rules, broken))
        .collect();
    scenario::sort(&mut entries);
    entries
}

/// Deletes the save `name` of the folder `dir` (E27 M4 T4b2: Delete's OK): a plain file name the
/// list shows ([`listed`]), never a path out of the folder nor a folder.
///
/// # Errors
/// The name is not a listed file's, or the file does not delete.
pub fn delete(dir: &Path, name: &str) -> Result<(), String> {
    let plain = Path::new(name).file_name().is_some_and(|n| n == name);
    if !plain || !listed(name) {
        return Err(format!("{name}: not a save of {}", dir.display()));
    }
    let path = dir.join(name);
    if !path.is_file() {
        return Err(format!("{}: not a file", path.display()));
    }
    fs::remove_file(&path).map_err(|e| format!("{}: {e}", path.display()))
}

/// Whether a file of the name `name` is listed (`*.gm?`, any case: the original's pattern of a
/// single load, `scenario_file_patterns` 0x683458).
#[must_use]
pub fn listed(name: &str) -> bool {
    Path::new(name)
        .extension()
        .and_then(|x| x.to_str())
        .is_some_and(|x| x.len() == 3 && x.is_char_boundary(2) && x[..2].eq_ignore_ascii_case("gm"))
}

/// The entry of the save file `file` at `path`: its map's, the save's difficulty and the file's
/// time; `broken`'s for a save the client cannot list.
#[must_use]
pub fn entry(
    file: String,
    path: &Path,
    install: &Path,
    rules: &GameRuleset,
    broken: &Broken,
) -> Entry {
    let time = modified(path);
    match map_of(path, install, rules) {
        Ok((map, saved)) => Entry {
            file,
            saved: Some(Saved { time, ..saved }),
            ..map
        },
        Err(_) => Entry {
            file,
            format: None,
            name: broken.name.clone(),
            description: broken.description.clone(),
            size: 0,
            difficulty: 0,
            victory: None,
            standard: false,
            loss: None,
            seats: [Seat::default(); 8],
            teams: [0, 1, 2, 3, 4, 5, 6, 7],
            heroes: Heroes::default(),
            saved: Some(Saved {
                time,
                ..Saved::default()
            }),
        },
    }
}

/// The file's last-write time in seconds from 1970 UTC (0 when the system cannot tell).
fn modified(path: &Path) -> i64 {
    let Ok(at) = fs::metadata(path).and_then(|m| m.modified()) else {
        return 0;
    };
    match at.duration_since(UNIX_EPOCH) {
        Ok(after) => i64::try_from(after.as_secs()).unwrap_or(i64::MAX),
        Err(before) => i64::try_from(before.duration().as_secs()).map_or(i64::MIN, |s| -s),
    }
}

/// The save at `path`'s map's entry (as a new game's list reads it) and what the save shows of
/// itself ([`saved`], its time the caller's), or why it cannot be listed.
fn map_of(path: &Path, install: &Path, rules: &GameRuleset) -> Result<(Entry, Saved), String> {
    let save = crate::save::read_file(path)?;
    let replay = &save.replay;
    replay.header.check(rules).map_err(|e| e.to_string())?;
    let (bytes, map) = h3_files::adventure::read_map(&install.join(MAPS), &replay.map.path)
        .map_err(|e| e.to_string())?;
    if map.content_hash != replay.map.content_hash
        || map.import_version != replay.map.import_version
    {
        return Err(format!("{}: not the save's map", replay.map.path));
    }
    let raw = h3_map::read(&bytes, &Limits::default()).map_err(|e| e.to_string())?;
    let entry = crate::maps::entry(replay.map.path.clone(), &raw, rules);
    Ok((entry, saved(&save)))
}

/// What `save` shows of itself on the scenario screen (E27 M4 T5b1; design §41.1), its file's
/// time 0: its difficulty, the colours its replay's players hold as human, each colour's town
/// (a fixed faction's choice, else Random) and first hero (the pick, else Random), bonus and
/// handicap (its table's), and the turn's duration.
#[must_use]
pub fn saved(save: &crate::save::Save) -> Saved {
    let mut saved = Saved {
        difficulty: save.replay.difficulty,
        duration: save.duration,
        ..Saved::default()
    };
    for (record, (&bonus, &handicap)) in saved
        .records
        .iter_mut()
        .zip(save.bonuses.iter().zip(&save.handicaps))
    {
        record.bonus = bonus;
        record.handicap = handicap;
    }
    for player in &save.replay.players {
        let c = usize::from(player.colour);
        let (Some(human), Some(record)) = (saved.humans.get_mut(c), saved.records.get_mut(c))
        else {
            continue;
        };
        *human = player.human;
        record.town = match player.faction {
            Some(FactionChoice::Fixed(f)) => u8::try_from(f.index()).ok(),
            Some(FactionChoice::Random) | None => None,
        };
        record.hero = player.hero.and_then(|h| u8::try_from(h.index()).ok());
    }
    saved
}

/// The game's seed of a made save ([`made`]).
pub const MADE_SEED: u64 = 1;

/// A save named `name` of a single game begun on the installation's map `map` at `difficulty`,
/// the turns ended until day `day` begins (E27 M4 T4b1: the harness's made saves, `h3-view save`,
/// matching the original's the maker recipes make, design §38.2): the single player's colour the
/// map's (`Entry::local`, as the screen's Begin with the advanced options untouched: Random
/// town, hero and bonus, no handicap, unlimited turns), every other colour the computer's; each
/// player's End Turn in turn, its replay checkpointed every 10 commands.
///
/// # Errors
/// The map's read, the begin's or the engine's refusal.
pub fn made(
    install: &Path,
    rules: &GameRuleset,
    map: &str,
    at: (u8, u32),
    name: &str,
) -> Result<crate::save::Save, String> {
    made_with(install, rules, map, at, &Seated::default(), name)
}

/// A hot seat's player of a made save (E27 M4 T5b1: the hot-seat makers' `viewer` tables, design
/// §41.2): its colour, its name and its choices as the advanced options' arrows chose them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    /// Its colour (0 red to 7 pink).
    pub colour: u8,
    /// Its name.
    pub name: String,
    /// Its town's faction, none for Random (or the map's own town).
    pub town: Option<u8>,
    /// Its first hero's type, none for Random.
    pub hero: Option<u8>,
    /// Its starting bonus.
    pub bonus: Bonus,
}

impl Player {
    /// Parses `<colour>:<name>:<town>:<hero>:<bonus>`: the colour 0 to 7, a name without a colon,
    /// the town's faction and the hero's type by their indexes or `random`, the bonus `artifact`,
    /// `gold`, `resource` or `random`.
    ///
    /// # Errors
    /// Not five fields, or a field out of its range.
    pub fn parse(text: &str) -> Result<Self, String> {
        let bad = || format!("save: player `{text}` (<colour>:<name>:<town>:<hero>:<bonus>)");
        let fields: Vec<&str> = text.split(':').collect();
        let &[colour, name, town, hero, bonus] = fields.as_slice() else {
            return Err(bad());
        };
        let index = |field: &str| -> Result<Option<u8>, String> {
            match field {
                "random" => Ok(None),
                n => n.parse().map(Some).map_err(|_| bad()),
            }
        };
        Ok(Self {
            colour: colour.parse().ok().filter(|c| *c < 8).ok_or_else(bad)?,
            name: name.to_owned(),
            town: index(town)?,
            hero: index(hero)?,
            bonus: crate::save::bonus(bonus).ok_or_else(bad)?,
        })
    }
}

/// Who plays a made save and its turn's duration (E27 M4 T5b1): no players, the single player's
/// game of [`made`]; else the hot seat's, each player a human of its colour, every other colour
/// the computer's (Random choices).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seated {
    /// The hot seat's players, none for a single game.
    pub players: Vec<Player>,
    /// The turn's duration (`turndur.txt`'s record, at most unlimited's).
    pub duration: u8,
}

impl Default for Seated {
    /// A single game of unlimited turns.
    fn default() -> Self {
        Self {
            players: Vec::new(),
            duration: UNLIMITED,
        }
    }
}

/// [`made`] played by `seated` (E27 M4 T5b1): a single game's, or the hot seat's players with
/// their names and choices; the turn's duration `seated`'s.
///
/// # Errors
/// As [`made`]'s; two players of one colour.
pub fn made_with(
    install: &Path,
    rules: &GameRuleset,
    map: &str,
    (difficulty, day): (u8, u32),
    seated: &Seated,
    name: &str,
) -> Result<crate::save::Save, String> {
    let raw = crate::begin::read(&crate::begin::map_path(install, map))?;
    let (records, names) = if seated.players.is_empty() {
        let local = crate::maps::entry(map.to_owned(), &raw, rules).local();
        let records = core::array::from_fn(|c| Record {
            colour: local.filter(|&l| l == c).and_then(|l| u8::try_from(l).ok()),
            town: None,
            hero: None,
            bonus: Bonus::Random,
            handicap: 0,
        });
        (records, Default::default())
    } else {
        let mut records = [Record {
            colour: None,
            town: None,
            hero: None,
            bonus: Bonus::Random,
            handicap: 0,
        }; 8];
        let mut names: [Option<String>; 8] = Default::default();
        for player in &seated.players {
            let c = usize::from(player.colour & 7);
            if names[c].is_some() {
                return Err(format!("{map}: two players of colour {c}"));
            }
            records[c] = Record {
                colour: Some(player.colour & 7),
                town: player.town,
                hero: player.hero,
                bonus: player.bonus,
                handicap: 0,
            };
            names[c] = Some(player.name.clone());
        }
        (records, names)
    };
    let begun = Begun {
        file: map.to_owned(),
        difficulty,
        records,
        names,
        duration: seated.duration.min(UNLIMITED),
    };
    let setup = crate::begin::setup(&raw, &begun, rules)?;
    crate::begin::validated(&setup, rules)?;
    let game = crate::begin::game(setup, &begun);
    let mut state = GameState::new(rules, &game.setup, MADE_SEED).map_err(|e| e.to_string())?;
    let mut commands = Vec::new();
    while state.day() < day {
        let actor = (0..=u8::MAX)
            .take(state.players())
            .map(PlayerId::new)
            .find(|&p| state.is_acting(p))
            .map(Actor::Player)
            .ok_or_else(|| format!("{map}: no player's turn on day {}", state.day()))?;
        apply(
            &mut state,
            rules,
            actor,
            GameCommand::EndTurn,
            &mut NoopSink,
        )
        .map_err(|e| format!("{map}: End Turn: {e:?}"))?;
        commands.push((actor, GameCommand::EndTurn));
    }
    let map_ref = crate::save::map(install, map)?;
    let replay = Replay::record_commands(rules, (&game.setup, map_ref), MADE_SEED, commands, 10)
        .map_err(|e| format!("{map}: {e:?}"))?;
    Ok(crate::save::Save::of(name, &game, replay))
}

/// `h3-view save`'s options (E27 M4 T4b1: the harness's made saves, `h3shell.py drawn`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    map: String,
    difficulty: u8,
    day: u32,
    seated: Seated,
    name: String,
    out: PathBuf,
    install: PathBuf,
}

impl Options {
    /// Parses `--map <file> --difficulty <0..4> [--day <n>] [--player <p>]... [--duration <0..10>]
    /// --name <name> --out <file> [--install <dir>]`: the day 1 by default; each `--player` a hot
    /// seat's ([`Player::parse`], E27 M4 T5b1; none: a single game), the duration unlimited (10)
    /// by default; the installation `$H3_INSTALL`, then the repository's `original-data/install`.
    ///
    /// # Errors
    /// An unknown argument, a flag without its value, a value out of its range, or a flag missing.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let env = std::env::var_os("H3_INSTALL").map(PathBuf::from);
        let (mut map, mut difficulty, mut day, mut name, mut out, mut install) =
            (None, None, 1, None, None, None);
        let mut seated = Seated::default();
        let mut rest = args.iter();
        while let Some(arg) = rest.next() {
            let mut value = || {
                rest.next()
                    .cloned()
                    .ok_or_else(|| format!("save: `{arg}` needs a value"))
            };
            match arg.as_str() {
                "--map" => map = Some(value()?),
                "--difficulty" => {
                    let v = value()?;
                    difficulty = Some(
                        v.parse::<u8>()
                            .ok()
                            .filter(|d| *d <= 4)
                            .ok_or_else(|| format!("save: difficulty `{v}` (0 to 4)"))?,
                    );
                }
                "--day" => {
                    let v = value()?;
                    day = v
                        .parse::<u32>()
                        .ok()
                        .filter(|d| *d >= 1)
                        .ok_or_else(|| format!("save: day `{v}` (1 on)"))?;
                }
                "--player" => seated.players.push(Player::parse(&value()?)?),
                "--duration" => {
                    let v = value()?;
                    seated.duration = v
                        .parse::<u8>()
                        .ok()
                        .filter(|d| *d <= UNLIMITED)
                        .ok_or_else(|| format!("save: duration `{v}` (0 to {UNLIMITED})"))?;
                }
                "--name" => name = Some(value()?),
                "--out" => out = Some(PathBuf::from(value()?)),
                "--install" => install = Some(PathBuf::from(value()?)),
                other => return Err(format!("save: unknown argument `{other}`")),
            }
        }
        let missing = |flag: &str| format!("save: {flag} is needed");
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        Ok(Self {
            map: map.ok_or_else(|| missing("--map"))?,
            difficulty: difficulty.ok_or_else(|| missing("--difficulty"))?,
            day,
            seated,
            name: name.ok_or_else(|| missing("--name"))?,
            out: out.ok_or_else(|| missing("--out"))?,
            install: install
                .or(env)
                .unwrap_or_else(|| manifest.join("../../../original-data/install")),
        })
    }
}

/// Writes the save the options name ([`made`]).
#[must_use]
pub fn run(options: &Options) -> std::process::ExitCode {
    let done = crate::maps::rules().and_then(|rules| {
        let at = (options.difficulty, options.day);
        let (map, seated) = (&options.map, &options.seated);
        let save = made_with(&options.install, &rules, map, at, seated, &options.name)?;
        crate::save::write_file(&options.out, &save)
    });
    match done {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("h3-view save: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests;
