//! Play's entry for the shell (E24 M5 T2b; the user's decision at M5 T0, "the shell on top", the
//! API agreed for the client lane at T2b): the game the shell began ([`Game`]: its map file, the
//! engine's validated setup, the hot seat's names, the turn's duration and the game's seed) played
//! in the shell's window with the shell's sound lent ([`play`]), and how play ended handed back
//! ([`End`]: the main menu, the load menu, the scenario again or the program's end, as E27 M5's
//! system menu will give them).
//!
//! - **What play shows until T3** (*provisional*; the user at T2b): the game started in-process by
//!   `h3-protocol`'s game session ([`start`]: `GameSession::start`, every seat the host's, its
//!   clock unlimited), its first human's record so far (the game's snapshot and turn) set up as
//!   the record player sets up a record's file, the map standing at that start with M2's input
//!   (the window's, `crate::window`): every fact the engine's (the user at M5 T0: no rules table or
//!   map text read in play), the pictures the map file's. T3 gives every human a seat, End Turn
//!   and the hand-over between them.
//! - **Its ends:** the window's quit key (and an unattended run's end) [`End::MainMenu`], the
//!   window closed [`End::Quit`]; [`End::Load`] and [`End::Restart`] wait for E27 M5's system menu.
//! - **The window lent** (800 × 600, the map's frame, at the host's scale): play sets the cursor
//!   and the key repeats it needs and turns text input off; the host sets its own again once play
//!   returned. **The sound lent:** the map's streams added on the host's sound and removed before
//!   play returns, an error's return too; none lent, play is silent.
//!
//! `h3-map-view play` ([`Options`], [`run`]) begins a game on a map file as the shell's Begin
//! would (the import's humans, or those `--humans` names) and plays it in a window and on a sound
//! of its own, until the shell calls [`play`] (the client lane's move, its inbox).

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use h3_battle_client::audio::HostSound;
use h3_contracts::PlayerId;
use h3_game::{GameRuleset, GameSetup};
use h3_protocol::map::game::manifest;
use h3_protocol::session::game::{Clock, CommandBounds, GameOptions, GameSession, Spectators};
use h3_protocol::wire::WireU64;
use h3_protocol::wire::game::MapIdentity;
use h3_render::Bugs;
use h3_sdl::{Config, Window};

use crate::files;
use crate::window::{self, Host, Source};

/// A game the shell began (its `begin::Game`, by the client lane's own type): what play needs of
/// it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    /// The map file the game was begun on (its pictures; its bytes' hash the game's manifest
    /// names).
    pub map: PathBuf,
    /// The engine's setup, validated (`GameSetup::validate`) under the rules play is given.
    pub setup: GameSetup,
    /// Each colour's human's name, red first (the hot seat's: T3's seats, T5's hand-over).
    pub names: [Option<String>; 8],
    /// The turn's duration (`turndur.txt`'s record: the session's clock from T3 and T5).
    pub duration: u8,
    /// The game's seed: the same setup and seed begin the same game.
    pub seed: u64,
}

/// Where play reads its files and how it draws: the installation copy (its archives, sounds and
/// music), the map client's folder (`clients/map`: its tables and layouts, the string catalogue
/// beside it), the user's key file (its `[map]` table over the defaults; none: `keys.toml` in the
/// working directory when there is one) and the original's drawing bugs or the client's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Folders {
    pub install: PathBuf,
    pub map: PathBuf,
    pub keys: Option<PathBuf>,
    pub bugs: Bugs,
}

impl Folders {
    /// The installation copy `install`, the map client's folder beside this package (the build's
    /// path: a Windows build passes its own, the lane's TRAPS), no key file, the client's drawing.
    #[must_use]
    pub fn new(install: PathBuf) -> Self {
        Self {
            install,
            map: files::default_map(),
            keys: None,
            bugs: Bugs::Fixed,
        }
    }
}

/// How play ended: back to the main menu, to the load menu, the scenario begun again, or the
/// program's end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum End {
    MainMenu,
    Load,
    Restart,
    Quit,
}

/// Plays `game` (its setup validated under `rules`) in the host's `window` (opened at the map's
/// frame, 800 × 600) with the host's `sound` lent (taken and given back before the return, the
/// map's streams removed; none: silent), its files from `folders`; how play ended. What play
/// writes on its way (the window's report) goes to standard error.
///
/// # Errors
/// The game does not start, or its map or the client's files do not load: nothing shown then, the
/// sound given back.
pub fn play(
    game: &Game,
    rules: &GameRuleset,
    window: &mut Window,
    sound: &mut Option<HostSound>,
    folders: &Folders,
) -> Result<End, String> {
    let options = window::Options::for_play(game, folders, None, false);
    let (lines, _, end) = played(game, rules, Host::Lent { window, sound }, &options)?;
    for line in lines {
        eprintln!("h3-map-client play: {line}");
    }
    Ok(end)
}

/// `game` played as [`play`] plays it, on `host` with the window's `options`: the window's report,
/// its check's verdict and how play ended.
fn played(
    game: &Game,
    rules: &GameRuleset,
    host: Host<'_>,
    options: &window::Options,
) -> Result<(Vec<String>, bool, End), String> {
    let started = start(game, rules)?;
    let source = Source::Text {
        map: &game.map,
        text: &started.record,
        rules,
        label: format!("play's start for colour {}", started.colour),
    };
    window::window(options, source, host)
}

/// A game started in-process: its first human (the setup's first human player: its id and its
/// colour) and that player's record so far, its start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Started {
    pub player: PlayerId,
    pub colour: u8,
    pub record: String,
}

/// `game` started in-process under `rules` (`GameSession::start`, as `h3-server record` starts a
/// replay's: every seat the host's, the clock unlimited, the whole game one record), the map named
/// by its bytes' hash: its first human and that player's record so far.
///
/// # Errors
/// The map file does not read; the engine starts no game of the setup; no player is human.
pub fn start(game: &Game, rules: &GameRuleset) -> Result<Started, String> {
    let file = &game.map;
    let failed = |e: &dyn std::fmt::Display| format!("{}: {e}", file.display());
    let name = file
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| failed(&"not a map file's name"))?;
    let root = file.parent().unwrap_or_else(|| Path::new("."));
    let (_, map) = h3_files::adventure::read_map(root, name).map_err(|e| failed(&e))?;
    let identity = MapIdentity {
        content_hash: WireU64(map.content_hash),
        import_version: map.import_version,
    };
    let options = GameOptions {
        game: 0,
        checkpoint_every: 0,
        record_bytes: usize::MAX,
        clock: Clock::UNLIMITED,
        commands: CommandBounds::NONE,
        spectators: Spectators::NONE,
    };
    let (index, human) = game
        .setup
        .players
        .iter()
        .enumerate()
        .find(|(_, p)| p.human)
        .ok_or_else(|| failed(&"no player is human"))?;
    let player = PlayerId::new(u8::try_from(index).map_err(|e| failed(&e))?);
    // Every frame stays in the session's records; play folds them from T3 on.
    let mut out = Vec::new();
    let session = GameSession::start(
        rules,
        &game.setup,
        game.seed,
        manifest(rules, identity),
        options,
        &mut out,
    )
    .map_err(|e| failed(&format_args!("the game does not start: {e}")))?;
    let record = session
        .record(player)
        .ok_or_else(|| failed(&format_args!("no record of player {}", player.index())))?;
    Ok(Started {
        player,
        colour: human.colour,
        record,
    })
}

/// The command's options.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    map: PathBuf,
    seed: u64,
    humans: Option<Vec<u8>>,
    scale: Option<u32>,
    mute: bool,
    folders: Folders,
    /// Unattended for this many seconds; whether each frame is read back.
    run: Option<(u32, bool)>,
}

impl Options {
    /// Parses `<map.h3m> [--seed <n>] [--humans <c>,...] [--keys <file>] [--scale <n>] [--mute]
    /// [--bugs fixed|original] [--install <dir>] [--map <dir>] [--check <s> | --measure <s>]`.
    ///
    /// # Errors
    /// An unknown option, a bad value, no map file.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let (mut map, mut seed, mut humans, mut scale, mut mute) = (None, 0, None, None, false);
        let (mut keys, mut bugs, mut install, mut folder, mut run) =
            (None, Bugs::Fixed, None, None, None);
        let mut rest = args.iter();
        while let Some(arg) = rest.next() {
            let mut value = || {
                rest.next()
                    .cloned()
                    .ok_or_else(|| format!("play: `{arg}` needs a value"))
            };
            let number = |v: &str| {
                v.parse::<u64>()
                    .map_err(|_| format!("play: bad number `{v}` for `{arg}`"))
            };
            let small = |v: &str| {
                v.parse::<u32>()
                    .map_err(|_| format!("play: bad number `{v}` for `{arg}`"))
            };
            match arg.as_str() {
                "--seed" => seed = number(&value()?)?,
                "--humans" => humans = Some(colours(&value()?)?),
                "--keys" => keys = Some(PathBuf::from(value()?)),
                "--scale" => scale = Some(small(&value()?)?.clamp(1, 16)),
                "--mute" => mute = true,
                "--bugs" => {
                    bugs = match value()?.as_str() {
                        "fixed" => Bugs::Fixed,
                        "original" => Bugs::Original,
                        _ => return Err("play: --bugs takes fixed or original".into()),
                    };
                }
                "--install" => install = Some(PathBuf::from(value()?)),
                "--map" => folder = Some(PathBuf::from(value()?)),
                "--check" => run = Some((small(&value()?)?, true)),
                "--measure" => run = Some((small(&value()?)?, false)),
                flag if flag.starts_with("--") => {
                    return Err(format!("play: unknown option `{flag}`"));
                }
                file if map.is_none() => map = Some(PathBuf::from(file)),
                extra => return Err(format!("play: unexpected `{extra}`")),
            }
        }
        Ok(Self {
            map: map.ok_or("play: needs a map file")?,
            seed,
            humans,
            scale,
            mute,
            folders: Folders {
                install: install.unwrap_or_else(files::default_install),
                map: folder.unwrap_or_else(files::default_map),
                keys,
                bugs,
            },
            run,
        })
    }
}

/// The colours of `--humans`: 0 (red) to 7, comma-separated.
fn colours(list: &str) -> Result<Vec<u8>, String> {
    list.split(',')
        .map(|c| {
            c.trim()
                .parse::<u8>()
                .ok()
                .filter(|&c| c < 8)
                .ok_or_else(|| format!("play: bad colour `{c}` for `--humans` (0 to 7)"))
        })
        .collect()
}

/// The game a map file begins under `rules`, as the shell's Begin makes it
/// (`h3_battle_client::begin`): the map read and imported, its humans the import's or, given, only
/// those of `humans` (a colour the map gives the computer alone refused), validated by the engine;
/// no names, no turn's limit.
///
/// # Errors
/// The map does not read or import; a colour of `humans` the map gives the computer alone, or none
/// it holds; the engine's refusal of the setup.
pub fn begin(
    map: &Path,
    humans: Option<&[u8]>,
    seed: u64,
    rules: &GameRuleset,
) -> Result<Game, String> {
    let failed = |e: &dyn std::fmt::Display| format!("{}: {e}", map.display());
    let (_, raw) = crate::paint::read_map(map)?;
    let mut setup = h3_map::import(&raw, rules).map_err(|e| failed(&e))?;
    if let Some(humans) = humans {
        for &colour in humans {
            let player = setup.players.iter().find(|p| p.colour == colour);
            if !player.is_some_and(|p| p.human) {
                return Err(failed(&format_args!(
                    "colour {colour}: no player a human may play"
                )));
            }
        }
        for player in &mut setup.players {
            player.human &= humans.contains(&player.colour);
        }
    }
    setup.validate(rules).map_err(|e| failed(&e))?;
    Ok(Game {
        map: map.to_path_buf(),
        setup,
        names: Default::default(),
        duration: 0,
        seed,
    })
}

/// Runs the command.
pub fn run(options: &Options) -> ExitCode {
    match play_command(options) {
        Ok((lines, ok, end)) => {
            for line in lines {
                println!("{line}");
            }
            println!("play ended: {end:?}");
            if ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(message) => {
            eprintln!("play: {message}");
            ExitCode::FAILURE
        }
    }
}

/// The command: the game begun, a window and a sound of its own lent to play as the shell lends
/// its own.
fn play_command(options: &Options) -> Result<(Vec<String>, bool, End), String> {
    let rules = h3_battle_client::maps::rules()?;
    let game = begin(
        &options.map,
        options.humans.as_deref(),
        options.seed,
        &rules,
    )?;
    let [width, height] = files::load_layout(&options.folders.map)?.frame;
    let mut win = Window::open(&Config {
        title: format!("h3-map-view play — {}", options.map.display()),
        width,
        height,
        scale: options.scale,
        vsync: false,
        fullscreen: false,
    })?;
    let mut sound = if options.mute {
        None
    } else {
        HostSound::open(options.folders.bugs)
            .map_err(|e| eprintln!("play: no sound ({e})"))
            .ok()
    };
    let window_options =
        window::Options::for_play(&game, &options.folders, options.run, options.mute);
    let host = Host::Lent {
        window: &mut win,
        sound: &mut sound,
    };
    let (mut lines, ok, end) = played(&game, &rules, host, &window_options)?;
    // The sound handed back (the shell's to use again), as play promises.
    lines.push(format!(
        "the sound handed back: {}",
        sound.as_ref().map_or("none", |_| "yes")
    ));
    Ok((lines, ok, end))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;

    use h3_map_view::situation::Situation;

    use super::*;

    /// A writer's map of the situation `text` in a file of its own (`name`), removed when dropped.
    struct MapFile(PathBuf);

    impl MapFile {
        fn new(name: &str, text: &str) -> Self {
            let (map, extras) = Situation::parse(text)
                .unwrap()
                .build(&BTreeMap::new(), &[])
                .unwrap();
            // Compressed, as the game keeps its maps (the record player reads them so).
            let bytes = h3_map::writer::gzip(&h3_map::writer::write_with(&map, &extras).unwrap());
            let dir =
                std::env::temp_dir().join(format!("h3-map-client-play-{}", std::process::id()));
            fs::create_dir_all(&dir).unwrap();
            let path = dir.join(format!("{name}.h3m"));
            fs::write(&path, bytes).unwrap();
            Self(path)
        }
    }

    impl Drop for MapFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    /// Red's hero and Blue's on a small map.
    const TWO: &str = "size = 36\nterrain = \"grass\"\n\
        heroes = [{ at = [3, 3] }, { at = [30, 30], owner = 1 }]\nview = [0, 0, 0]\n";

    #[test]
    fn a_map_begins_with_the_import_s_humans_or_those_named() {
        let rules = h3_battle_client::maps::rules().unwrap();
        let file = MapFile::new("humans", TWO);
        let humans = |game: &Game| -> Vec<u8> {
            game.setup
                .players
                .iter()
                .filter(|p| p.human)
                .map(|p| p.colour)
                .collect()
        };
        let game = begin(&file.0, None, 5, &rules).unwrap();
        assert_eq!(humans(&game), [0, 1]);
        assert_eq!((game.seed, game.duration), (5, 0));
        assert_eq!(game.names, <[Option<String>; 8]>::default());
        let game = begin(&file.0, Some(&[1]), 5, &rules).unwrap();
        assert_eq!(humans(&game), [1]);
        let e = begin(&file.0, Some(&[2]), 5, &rules).unwrap_err();
        assert!(e.ends_with("colour 2: no player a human may play"), "{e}");
        let e = begin(&file.0.with_extension("none"), None, 5, &rules).unwrap_err();
        assert!(e.contains("none"), "{e}");
    }

    #[test]
    fn play_starts_with_the_first_human_s_record() {
        let rules = h3_battle_client::maps::rules().unwrap();
        let file = MapFile::new("start", TWO);
        let bytes = fs::read(&file.0).unwrap();
        for (humans, id) in [(None, 0), (Some(&[1][..]), 1)] {
            let game = begin(&file.0, humans, 9, &rules).unwrap();
            let started = start(&game, &rules).unwrap();
            assert_eq!((started.player, started.colour), (PlayerId::new(id), id));
            // The record the record player reads (its map the file's by its hash), its start the
            // first human's view.
            let loaded = h3_map_view::engine::load(&started.record, &bytes, &rules).unwrap();
            let first = loaded.first().unwrap();
            assert_eq!(first.viewer, id);
            // The same setup and seed begin the same game.
            assert_eq!(start(&game, &rules).unwrap(), started);
        }
        // No human: no start.
        let mut game = begin(&file.0, None, 9, &rules).unwrap();
        for player in &mut game.setup.players {
            player.human = false;
        }
        let e = start(&game, &rules).unwrap_err();
        assert!(e.ends_with("no player is human"), "{e}");
    }

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|&a| a.to_owned()).collect()
    }

    #[test]
    fn the_command_takes_a_map_and_its_options() {
        let o = Options::parse(&args(&[
            "m.h3m",
            "--seed",
            "7",
            "--humans",
            "0, 1",
            "--scale",
            "2",
            "--mute",
            "--bugs",
            "original",
            "--install",
            "i",
            "--map",
            "d",
            "--keys",
            "k.toml",
            "--check",
            "5",
        ]))
        .unwrap();
        assert_eq!(o.map, PathBuf::from("m.h3m"));
        assert_eq!(
            (o.seed, o.humans.as_deref(), o.scale),
            (7, Some(&[0, 1][..]), Some(2))
        );
        assert!(o.mute);
        assert_eq!(o.run, Some((5, true)));
        assert_eq!(
            o.folders,
            Folders {
                install: PathBuf::from("i"),
                map: PathBuf::from("d"),
                keys: Some(PathBuf::from("k.toml")),
                bugs: Bugs::Original,
            }
        );
        let o = Options::parse(&args(&["m.h3m", "--measure", "3"])).unwrap();
        assert_eq!((o.seed, o.humans, o.run), (0, None, Some((3, false))));
    }

    #[test]
    fn the_command_refuses_what_it_does_not_know() {
        for (list, why) in [
            (&[][..], "needs a map file"),
            (&["m", "--humans", "8"][..], "bad colour `8`"),
            (&["m", "--humans", "red"][..], "bad colour `red`"),
            (&["m", "--seed"][..], "needs a value"),
            (&["m", "--seed", "x"][..], "bad number `x`"),
            (&["m", "--bugs", "some"][..], "fixed or original"),
            (&["m", "--fast"][..], "unknown option `--fast`"),
            (&["m", "n"][..], "unexpected `n`"),
        ] {
            let e = Options::parse(&args(list)).unwrap_err();
            assert!(e.contains(why), "{list:?}: {e}");
        }
    }
}
