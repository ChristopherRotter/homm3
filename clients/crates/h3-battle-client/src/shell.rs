//! `h3-view shell` (E27 M2 T3a, T4; CLT-Q103, decision 21: the player's program until E24 M5
//! hosts the shell): the game shell's frame, three ways.
//!
//! - **Headless by words** (T3a): `h3_view::shell::text`, a word a line: the screen on is printed
//!   with the words it takes, each word read ends it and prints the next. The words come from
//!   standard input, or from a script file (`--script`), each echoed after `> `; a word a script's
//!   screen does not take ends the run with exit code 1.
//! - **A recipe's steps replayed** (T4: `--steps <file> --frames <dir>`): a capture recipe's steps
//!   (`clients/capture/shell/<group>.toml`, a step a line) played on the shell's own clock
//!   (`h3_view::shell::replay`), each shot written as `<dir>/<shot>.565` and each clip's changed
//!   frame as `<dir>/<clip>/t<ms>.565` (800 × 600 pixels of 5-6-5, little-endian, as the
//!   harness's dumps), listed in `<dir>/replay.txt` (`shot <name> t<ms>`, `clip <name> t<ms>`, and
//!   `stop <k> <why>` when step `k` is one the viewer cannot take). `h3shell.py drawn` compares
//!   them with the original's shots.
//! - **In a window** (T4: `--window`): the built screens drawn (`h3_view::shell::look`), the
//!   others black until their task, the cursor the shell's arrow; the words still read from
//!   standard input. Closing the window ends it. The main menu takes the mouse and its hotkeys
//!   (T6a: `h3_view::shell::menu`, the key table `h3_view::shell::keys`; since M2's review the
//!   user's file over it as the battle window's, `--keys`, else `keys.toml` here: its `[shell]`),
//!   New Game's and Load Game's menus theirs (T6b1), each press playing its sample
//!   (`button.wav`); the music (`MAINMENU.MP3`, looping) is asked for as each menu opens, as
//!   `main_menu_run` and `game_type_menu_run` ask for it (the track playing goes on). High
//!   Scores' tables are the window's scores file's (E27 M4 T1b: `--scores`, else `scores.toml`
//!   here; none there: `CampHigh.txt`'s defaults), written whole as Reset's OK resets them; the
//!   replay starts from the defaults and writes none.
//!
//! `--no-intro` starts at the main menu (the setting `Show Intro` off); else the start's clips
//! play first (T5a: `h3_view::shell::start`, read from the video archives, `introrim.pcx` from the
//! LODs), from Bink or with `--video-quality low` from their Smacker copies (the setting Bink
//! Video 0, CLT-Q96). The replay draws them as the DLLs copy them and paces a silent Smacker clip
//! as `SmackWait` does (`Bugs::Original`), the window whole and each frame on its time
//! (CLT-Q129 (b)), on the window's own clock. Both give the main menu the inputs of the fade
//! before it (CLT-Q130 (a), `Start::keeps_input`).
//!
//! **The sound** (E27 M2 T5b1): `--effects <0..9>` is the effects setting (the window's default
//! 5, the game's; the replay's 0, the harness's silent runs). The window opens its sound device
//! for the effects or the music (a first run's setting) above 0 (M2's review). With the effects
//! above 0, with a sound device (the window's) or none needed (the replay), the clips play with
//! their sound: a Smacker clip's frames then follow it (`h3_view::shell::start::SmackSound`), and
//! the window plays each clip's track from its start (`HostSound::play_clip`, at
//! `h3_view::clip_sound::volume`) until a skip stops it, or whole once its clip ran to its end
//! (CLT-Q125, decision 34: the original cuts the last frame's sound at the close); the replay
//! lists each start, skip and close in `replay.txt` (`sound <file> t<ms>`, `quiet <file> t<ms>`,
//! `closed <file> t<ms>`), and (T6a) each press's sample as the harness logs the original's
//! (`sample button.wav t<ms> ch<k> <length>ms x1`: `h3_view::shell::samples`).
//!
//! The menus' boxes (T6a, T6b1): Quit's question and the buttons' help, their texts the user's
//! tables' through the key catalogue (`shell.main-menu.*`, `shell.game-type.help.*`), laid out at
//! the start. The stubs (T6b2): New Game's Campaign and Tutorial and the main menu's High Scores
//! and Credits show a notice in the client's own words (`shell.stub.*`, its own pack
//! `clients/text/en.json`) over the menu they were chosen from, in the window; its OK goes back to
//! that menu. The replay stops at them (the original shows its own screens there). The
//! Multiplayer dialog and the hot seat's names (T7): their hints, help and title the user's
//! tables' (`shell.multiplayer.*`, `shell.hot-seat.*`), the network modes' notice and the scenario
//! screen's stub the client's (`shell.stub.network`, `shell.stub.scenario`); the window, the
//! client's profile alone since T7b (decision 39), gives the dialogs' edits the text the keys
//! print while an edit types (SDL's text input, `h3_sdl::Window::typed`: the system's layout),
//! the edits' keys and the hot seat's ring through the key table with Shift as held
//! (`h3_sdl::Window::scans`, `keys::command_for`), no scan code (the original's profile's
//! typing, by the US table, is the replay's alone: M2's review). The replay draws a game type
//! menu's rows its redraw does not show as last shown (CLT-Q118: `h3_view::shell::menu::Stale`,
//! the original's profile).

use std::fs;
use std::io::{self, BufRead};
use std::mem;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::mpsc;
use std::thread;

use h3_assets::wav::Pcm;
use h3_game::GameRuleset;
use h3_render::Bugs;
use h3_render::blit::draw_image;
use h3_render::screen::Screen as Pixels;
use h3_sdl::{Config, CursorImage, Event, Key, KeyScan, Window};
use h3_ui::cursor::Cursor;
use h3_ui::keys::Keys;
use h3_ui::text::Args;
use h3_view::batch::Batch;
use h3_view::compose::FORMAT;
use h3_view::message::{self, MessageBox, Style};
use h3_view::shell::credits::{self, Credits};
use h3_view::shell::keys::{self as shell_keys, Command, GAME_TYPE, MAIN_MENU};
use h3_view::shell::look::{self, CursorLook, ShellLayout};
use h3_view::shell::menu::{self, Arrow, Boxes, Front, Kind, Menu, Menus};
use h3_view::shell::multiplayer::{self, Multiplayer, Texts};
use h3_view::shell::replay::{self, INPUT_STEP, Replay, Stop, Take};
use h3_view::shell::scenario::{self, Begun, Scenario};
use h3_view::shell::scores::{self, Pictures, Scores, ScoresLook, Tab, Tables};
use h3_view::shell::start::{self as clips, Header, Op, Pace, Part, Picture, Source, Start};
use h3_view::shell::{Ended, Game, Screen, Shell, text};

use crate::assets;
use crate::lods::Lods;
use crate::videos::Videos;

/// The original's screen.
const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
/// Where the harness parks the pointer before the game starts (h3shell's `PARK`).
const PARK: [i32; 2] = [400, 300];

/// How the command runs.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Mode {
    /// Words from standard input, or from a script.
    Words(Option<PathBuf>),
    /// A recipe's steps into frames.
    Replay {
        steps: PathBuf,
        frames: PathBuf,
        pointer: [i32; 2],
        input_step: u32,
        /// The original's draws of the scenario screen's background, an opening each (E27 M3
        /// T1a: `--backgrounds`, a recipe's).
        backgrounds: Vec<u8>,
        /// A file naming the maps the scenario list reads, one a line (E27 M3 T1b1: `--maps`, a
        /// recipe's `maps`); none for every map of the installation.
        maps: Option<PathBuf>,
    },
    /// In a window, at a scale (`None`: the largest that fits).
    Window(Option<u32>),
}

/// The command's options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    mode: Mode,
    intro: bool,
    /// The setting Bink Video (Video Quality High: the start's Bink clips).
    bink_video: bool,
    /// The effects setting (0 to 9), when given.
    effects: Option<u8>,
    install: PathBuf,
    /// The window's key file (`--keys`; else `keys.toml` here if there is one: its `[shell]`
    /// table, M2's review of E27, as the battle window's `[battle]`).
    keys: Option<PathBuf>,
    /// The window's scores file (E27 M4 T1b: `--scores`; else `scores.toml` here, beside the
    /// settings' and the keys', *provisional*).
    scores: Option<PathBuf>,
    /// The saves' folder (E27 M4 T4b1: `--games`; else `games` here, *provisional*): the load
    /// mode lists its saves, a replay's the harness's made saves.
    games: Option<PathBuf>,
}

impl Options {
    /// Parses `[--script <file> | --steps <file> --frames <dir> [--pointer <x>,<y>]
    /// [--input-step <ms>] [--backgrounds <n>,…] [--maps <file>] | --window [--scale <n>]
    /// [--keys <file>] [--scores <file>]] [--games <dir>] [--no-intro]
    /// [--video-quality high|low] [--effects <0..9>] [--install <dir>]`; the installation
    /// defaults to `$H3_INSTALL`, then the repository's `original-data/install`.
    ///
    /// # Errors
    /// An unknown argument, a flag without its value, or two modes.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        // An adapter's ambient input, read once here.
        let env = std::env::var_os("H3_INSTALL").map(PathBuf::from);
        Self::parse_with(args, env)
    }

    fn parse_with(args: &[String], env: Option<PathBuf>) -> Result<Self, String> {
        let (mut script, mut steps, mut frames, mut install) = (None, None, None, None);
        let (mut pointer, mut input_step, mut scale) = (None, None, None);
        let (mut intro, mut window, mut bink_video) = (true, false, true);
        let (mut effects, mut keys, mut backgrounds, mut maps) = (None, None, None, None);
        let (mut scores, mut games) = (None, None);
        let mut rest = args.iter();
        while let Some(arg) = rest.next() {
            let mut value = || {
                rest.next()
                    .cloned()
                    .ok_or_else(|| format!("shell: `{arg}` needs a value"))
            };
            match arg.as_str() {
                "--script" => script = Some(PathBuf::from(value()?)),
                "--steps" => steps = Some(PathBuf::from(value()?)),
                "--frames" => frames = Some(PathBuf::from(value()?)),
                "--install" => install = Some(PathBuf::from(value()?)),
                "--pointer" => pointer = Some(point(&value()?)?),
                "--input-step" => input_step = Some(number(&value()?)?),
                "--backgrounds" => backgrounds = Some(draws(&value()?)?),
                "--maps" => maps = Some(PathBuf::from(value()?)),
                "--scale" => scale = Some(number(&value()?)?),
                "--keys" => keys = Some(PathBuf::from(value()?)),
                "--scores" => scores = Some(PathBuf::from(value()?)),
                "--games" => games = Some(PathBuf::from(value()?)),
                "--window" => window = true,
                "--no-intro" => intro = false,
                "--effects" => {
                    let v = value()?;
                    effects = Some(
                        v.parse::<u8>()
                            .ok()
                            .filter(|e| *e <= 9)
                            .ok_or_else(|| format!("shell: effects `{v}` (0 to 9)"))?,
                    );
                }
                "--video-quality" => {
                    bink_video = match value()?.as_str() {
                        "high" => true,
                        "low" => false,
                        other => return Err(format!("shell: video quality `{other}`")),
                    };
                }
                other => return Err(format!("shell: unknown argument `{other}`")),
            }
        }
        let replaying = steps.is_some() || frames.is_some();
        let modes = usize::from(script.is_some()) + usize::from(replaying) + usize::from(window);
        if modes > 1 {
            return Err("shell: one of --script, --steps and --window".into());
        }
        let lone = |given: bool, flag: &str, mode: &str| {
            if given {
                Err(format!("shell: {flag} only with {mode}"))
            } else {
                Ok(())
            }
        };
        let replay_only =
            pointer.is_some() || input_step.is_some() || backgrounds.is_some() || maps.is_some();
        lone(
            !replaying && replay_only,
            "--pointer, --input-step, --backgrounds and --maps",
            "--steps",
        )?;
        lone(!window && scale.is_some(), "--scale", "--window")?;
        lone(!window && keys.is_some(), "--keys", "--window")?;
        lone(!window && scores.is_some(), "--scores", "--window")?;
        let mode = match (steps, frames) {
            (Some(steps), Some(frames)) => Mode::Replay {
                steps,
                frames,
                pointer: pointer.unwrap_or(PARK),
                input_step: input_step.unwrap_or(INPUT_STEP),
                backgrounds: backgrounds.unwrap_or_default(),
                maps,
            },
            (None, None) if window => Mode::Window(scale),
            (None, None) => Mode::Words(script),
            _ => return Err("shell: --steps and --frames go together".into()),
        };
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        Ok(Self {
            mode,
            intro,
            bink_video,
            effects,
            install: install
                .or(env)
                .unwrap_or_else(|| manifest.join("../../../original-data/install")),
            keys,
            scores,
            games,
        })
    }

    /// The saves' folder: `--games`'s, else [`crate::saves::GAMES`] in the working directory.
    fn games(&self) -> PathBuf {
        self.games
            .clone()
            .unwrap_or_else(|| PathBuf::from(crate::saves::GAMES))
    }
}

/// The window's scores file looked for in the working directory (E27 M4 T1b, *provisional*:
/// beside `settings.toml` and `keys.toml`).
const SCORES_FILE: &str = "scores.toml";

/// The scores file's text at `path` (E27 M4 T1b): none when there is none yet (a first run's, or
/// one never reset: the defaults).
fn scores_text(path: &Path) -> Result<Option<String>, String> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

/// The maps a replay's file `path` names, one a line (E27 M3 T1b1: a recipe's `maps`); none for
/// none given.
fn maps_named(path: Option<&Path>) -> Result<Option<Vec<String>>, String> {
    path.map(|p| {
        let text = fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))?;
        Ok(text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(String::from)
            .collect())
    })
    .transpose()
}

fn number(s: &str) -> Result<u32, String> {
    s.parse()
        .map_err(|_| format!("shell: `{s}` is not a number"))
}

/// The scenario screen's background draws, `0` or `1` each, comma-separated.
fn draws(s: &str) -> Result<Vec<u8>, String> {
    s.split(',')
        .map(|n| match n.trim() {
            "0" => Ok(0),
            "1" => Ok(1),
            _ => Err(format!("shell: background `{n}` (0 or 1)")),
        })
        .collect()
}

fn point(s: &str) -> Result<[i32; 2], String> {
    let bad = || format!("shell: `{s}` is not <x>,<y>");
    let (x, y) = s.split_once(',').ok_or_else(bad)?;
    Ok([x.parse().map_err(|_| bad())?, y.parse().map_err(|_| bad())?])
}

/// Runs the shell as the options say.
#[must_use]
pub fn run(options: &Options) -> ExitCode {
    let done = match &options.mode {
        Mode::Words(script) => return words(options.intro, script.as_deref()),
        // A replay's High Scores start from the defaults (as the recipes' runs: no HiScore.dat)
        // and write no file.
        Mode::Replay { maps, .. } => maps_named(maps.as_deref())
            .and_then(|only| {
                let given = (only.as_deref(), options.games());
                Drawer::load(&options.install, Bugs::Original, given, None)
            })
            .and_then(|drawer| replay_steps(options, &drawer)),
        Mode::Window(scale) => {
            let path = options
                .scores
                .clone()
                .unwrap_or_else(|| PathBuf::from(SCORES_FILE));
            scores_text(&path)
                .and_then(|text| {
                    let file = text.as_deref().map(|t| (path.as_path(), t));
                    let given = (None, options.games());
                    Drawer::load(&options.install, Bugs::Fixed, given, file)
                })
                .and_then(|drawer| in_window(options, *scale, &drawer, &path))
        }
    };
    match done {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("h3-view shell: {e}");
            ExitCode::FAILURE
        }
    }
}

/// The words, from standard input or a script (T3a).
fn words(intro: bool, script: Option<&Path>) -> ExitCode {
    let mut shell = Shell::new(intro);
    println!("{}", text::describe(shell.screen()));
    let lines: Vec<String> = match script {
        Some(path) => match fs::read_to_string(path) {
            Ok(script) => script.lines().map(String::from).collect(),
            Err(e) => {
                eprintln!("h3-view shell: {}: {e}", path.display());
                return ExitCode::FAILURE;
            }
        },
        None => {
            for line in io::stdin().lock().lines() {
                let Ok(line) = line else { break };
                println!("{}", text::answer(&mut shell, &line));
                if shell.screen() == Screen::End {
                    break;
                }
            }
            return ExitCode::SUCCESS;
        }
    };
    for line in lines {
        let refused = !line.trim().is_empty() && text::ending(shell.screen(), &line).is_none();
        println!("> {}", line.trim());
        println!("{}", text::answer(&mut shell, &line));
        if refused {
            return ExitCode::FAILURE;
        }
        if shell.screen() == Screen::End {
            break;
        }
    }
    ExitCode::SUCCESS
}

/// `H3Intro`'s frame (`start_clips` 0x4EE7A6 hands it to `video_play_framed`).
const RIM: &str = "introrim.pcx";
/// The buttons' press sample (`button_sample` 0x694DA4, loaded at 0x4EDA51), and its entry in the
/// sound archives (their names without the extension).
const BUTTON: &str = "button.wav";
const BUTTON_ENTRY: &str = "BUTTON";
/// Its volume (`button_press` 0x45688F: 64, on Miles' scale).
const BUTTON_VOLUME: u8 = 64;
/// The menus' music (`main_menu_run` 0x4FB6E0: "MainMenu", `MAINMENU.MP3`).
const MAIN_MUSIC: &str = "MainMenu";

/// The shell's layout, its assets and its cursor: what draws a frame.
struct Drawer {
    layout: ShellLayout,
    batch: Batch,
    /// The arrows, the default cursors' and the adventure map's (CLT-Q137), each drawn and as
    /// the window's hardware cursor.
    cursor: Cursor,
    hardware: CursorImage,
    adventure: Cursor,
    adventure_hardware: CursorImage,
    bugs: Bugs,
    /// `introrim.pcx` drawn on the screen (none when the LODs lack it: `H3Intro` then plays on
    /// what the back buffer held).
    rim: Option<Pixels>,
    videos: Videos,
    /// The menus as they open: their dialogs and their boxes laid out (T6a, T6b1).
    menus: Menus,
    /// The press sample, when the sound archives hold it.
    button: Option<Pcm>,
    /// The game's rules the maps are imported with (E27 M3 T3b1), the begin's setup too (T6).
    rules: GameRuleset,
    /// Play's notices in place of the stub's (T6; E27 M4 T4b2).
    play: PlayNotices,
    /// The saves' folder and what a save the client cannot list shows (E27 M4 T4b1: the window
    /// lists the folder again at each opening of the load mode).
    games: PathBuf,
    broken: crate::saves::Broken,
}

impl Drawer {
    /// The layout (`clients/layouts/shell.toml`) and its assets from `install`'s archives, read
    /// by their tables in one batch (CLT-Q2); High Scores' tables the scores file's (its path
    /// and text, `scores`) over the defaults (E27 M4 T1b), the defaults with none.
    fn load(
        install: &Path,
        bugs: Bugs,
        (maps, games): (Option<&[String]>, PathBuf),
        scores: Option<(&Path, &str)>,
    ) -> Result<Self, String> {
        let path = assets::clients_dir().join("layouts").join("shell.toml");
        let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let layout = look::parse_layout(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        let lods = Lods::open(install)?;
        let tables = assets::load(&assets::default_dir(), assets::ALL)?;
        let mut names = look::names(&layout);
        names.push(String::from(RIM));
        // The heroes' small portraits (E27 M3 T3b1: the advanced options' rows).
        names.extend(portraits(&tables));
        // High Scores' defaults, a scores file's tables (T1b) and the creatures' DEFs the rows of
        // both show (E27 M4 T1a).
        let (scores_tables, creature_defs) = scores_tables(&layout.high_scores, &lods, scores)?;
        names.extend(creature_defs.values().cloned());
        let mut batch = Batch::with_bugs(bugs);
        for (name, bytes) in names.iter().zip(lods.batch(&names)?) {
            batch.add(name, &bytes).map_err(|e| e.to_string())?;
        }
        let pictures = creature_pictures(&batch, creature_defs);
        let (menus, rules, play, broken) = menus(
            &layout,
            (&batch, &tables),
            &lods,
            (install, maps, &games),
            (scores_tables, pictures),
            bugs,
        )?;
        let mut lines = Vec::new();
        let button = crate::audio::clips(install, &[BUTTON_ENTRY.to_owned()], &mut lines)?
            .into_values()
            .next();
        let rim = batch.picture(RIM).and_then(|pcx| {
            let mut pixels = Pixels::new(WIDTH, HEIGHT, FORMAT)?;
            draw_image(&mut pixels, pcx, 0, 0);
            Some(pixels)
        });
        let arrow = |look: &CursorLook| {
            let (def, frame) = (&look.def, look.frame);
            let cursor = batch
                .sprite(def)
                .and_then(|s| Cursor::read(s, frame, (0, 0), FORMAT))
                .ok_or_else(|| format!("{def}: no frame {frame}"))?;
            let image = cursor.image(FORMAT);
            let hardware = CursorImage {
                width: image.width,
                height: image.height,
                hot_x: image.hot.0,
                hot_y: image.hot.1,
                rgba: image.rgba,
            };
            Ok::<_, String>((cursor, hardware))
        };
        let (cursor, hardware) = arrow(&layout.cursor)?;
        let (adventure, adventure_hardware) = arrow(&layout.adventure_cursor)?;
        Ok(Self {
            layout,
            batch,
            cursor,
            hardware,
            adventure,
            adventure_hardware,
            bugs,
            rim,
            videos: Videos::open(install),
            menus,
            button,
            rules,
            play,
            games,
            broken,
        })
    }

    /// The press sample's length in ms (as the harness logs it: its frames at its rate, rounded
    /// up), none without it.
    fn button_ms(&self) -> Option<u32> {
        let pcm = self.button.as_ref()?;
        Some(h3_view::sound::length_ms(pcm.frames() as u64, pcm.rate))
    }

    /// The start's clips' bytes from the video archives (none for a clip no archive holds: the
    /// sequence ends there, as the original's).
    fn clips(&self, bink_video: bool) -> [Option<Vec<u8>>; 3] {
        Part::ALL.map(|part| self.videos.read(part.file(bink_video)).ok())
    }

    /// The frame of `screen` (a menu or the Multiplayer dialog as `front` is), the cursor at
    /// `pointer` if any and shown: a built screen's background copied whole, its dialog over it,
    /// the rows a redraw left as last shown (the original's profile), the box over them;
    /// another's black.
    fn frame(
        &self,
        screen: Screen,
        pointer: Option<[i32; 2]>,
        front: Option<&Front>,
    ) -> Result<Pixels, String> {
        let mut pixels = Pixels::new(WIDTH, HEIGHT, FORMAT).ok_or("no 800 × 600 screen")?;
        let (mut cursor, mut arrow) = (true, &self.cursor);
        if let Some(shown) = look::look(&self.layout, screen, front) {
            let background = self
                .batch
                .picture(&shown.background)
                .ok_or_else(|| format!("{}: not loaded", shown.background))?;
            draw_image(&mut pixels, background, 0, 0);
            for draw in &shown.draws {
                h3_ui::paint::draw(&mut pixels, draw, &self.batch, self.bugs)?;
            }
            for stale in &shown.stale {
                let mut kept = Pixels::new(WIDTH, HEIGHT, FORMAT).ok_or("no 800 × 600 screen")?;
                draw_image(&mut kept, background, 0, 0);
                for draw in &stale.draws {
                    h3_ui::paint::draw(&mut kept, draw, &self.batch, self.bugs)?;
                }
                for &rect in &stale.rects {
                    copy_rect(&mut pixels, &kept, rect);
                }
            }
            for draw in &shown.over {
                h3_ui::paint::draw(&mut pixels, draw, &self.batch, self.bugs)?;
            }
            // Play's start's fade (E27 M3 T6): the screen drawn, its channels shifted down.
            if shown.shift > 0 {
                let from = pixels.clone();
                clips::faded(&from, &mut pixels, shown.shift);
            }
            cursor = shown.cursor;
            if shown.arrow == Arrow::Adventure {
                arrow = &self.adventure;
            }
        }
        if let Some([x, y]) = pointer.filter(|_| cursor) {
            // The original's bug: the mouse's x rounded down to even first (0x50CE04, E17
            // design §9.9: fixed, reproduced for the comparisons only, as the battle's).
            let x = match self.bugs {
                Bugs::Original => x & !1,
                Bugs::Fixed => x,
            };
            arrow.draw(&mut pixels, x, y);
        }
        Ok(pixels)
    }
}

/// `from`'s pixels inside `rect` (left, top, width, height; clipped to the screen) copied onto
/// `to` (both 800 × 600).
fn copy_rect(to: &mut Pixels, from: &Pixels, [x, y, w, h]: [i32; 4]) {
    let clamp = |v: i32, most: u32| usize::try_from(v.clamp(0, most as i32)).unwrap_or(0);
    let (left, right) = (clamp(x, WIDTH), clamp(x.saturating_add(w), WIDTH));
    let (top, bottom) = (clamp(y, HEIGHT), clamp(y.saturating_add(h), HEIGHT));
    let width = WIDTH as usize;
    let source = from.pixels();
    let target = to.pixels_mut();
    for row in top..bottom {
        let span = row * width + left..row * width + right;
        target[span.clone()].copy_from_slice(&source[span]);
    }
}

/// The objects' class of a map's monster (`class_template_last`'s 0x36 at High Scores' callers).
const MONSTER: u32 = 54;

/// High Scores' defaults (E27 M4 T1a: `CampHigh.txt`'s records, the user's tables), the tables of
/// the scores file `file` (its path and text) when there is one (T1b: `Tables::read`, over the
/// defaults), and
/// the DEF of each creature the rows of both show (Reset's OK sets the defaults again): the
/// adventure map's last template of the creature (class 54, its subclass the creature) in
/// `objects.txt`'s order, as `game_init` fills the game's map with them (0x505DA0) and
/// `class_template_last` 0x505EA0 finds them (a creature without one shows none).
fn scores_tables(
    look: &ScoresLook,
    lods: &Lods,
    file: Option<(&Path, &str)>,
) -> Result<(ScoresTables, std::collections::BTreeMap<i32, String>), String> {
    let bytes = lods.lookup("CAMPHIGH.TXT")?;
    let table =
        h3_formats::text::TextTable::parse(&bytes).map_err(|e| format!("CAMPHIGH.TXT: {e:?}"))?;
    let defaults = Tables::defaults(&table.records)
        .ok_or("CAMPHIGH.TXT: not 23 records of the player, the land, the days and the score")?;
    let tables = file
        .map(|(path, text)| Tables::read(text).map_err(|e| format!("{}: {e}", path.display())))
        .transpose()?;
    // Latin-1, one byte a character; the first line the count.
    let objects: String = lods
        .lookup("OBJECTS.TXT")?
        .into_iter()
        .map(char::from)
        .collect();
    let mut last = std::collections::BTreeMap::new();
    for line in objects.lines().skip(1) {
        if let Ok(template) = h3_map::writer::ObjectTemplate::from_objects_txt(line)
            && template.class == MONSTER
        {
            last.insert(template.subclass, template.def);
        }
    }
    let defs = [Some(&defaults), tables.as_ref()]
        .into_iter()
        .flatten()
        .flat_map(|t| {
            Tab::ALL
                .iter()
                .flat_map(move |&tab| t.of(tab).iter().map(move |r| (tab, r.score)))
        })
        .filter_map(|(tab, score)| look.creature(score, tab))
        .filter_map(|c| {
            let def = last.get(&u32::try_from(c).ok()?)?;
            Some((c, def.clone()))
        })
        .collect();
    Ok((ScoresTables { defaults, tables }, defs))
}

/// High Scores' tables as the start reads them (E27 M4 T1b): the defaults, and a scores file's.
struct ScoresTables {
    defaults: Tables,
    tables: Option<Tables>,
}

/// Each creature's picture: its DEF and the frames of its group 0 in `batch` (0 when the batch
/// lacks it).
fn creature_pictures(batch: &Batch, defs: std::collections::BTreeMap<i32, String>) -> Pictures {
    defs.into_iter()
        .map(|(creature, def)| {
            let frames = batch
                .sprite(&def)
                .and_then(|s| s.def.groups.iter().find(|g| g.id == 0))
                .map_or(0, |g| u32::try_from(g.frames.len()).unwrap_or(u32::MAX));
            (creature, scores::Picture { def, frames })
        })
        .collect()
}

/// The menus as they open (T6a, T6b1): the layout's dialogs, Quit's question and the buttons'
/// help laid out in the batch's fonts, their texts the user's tables' (`shell.main-menu.quit`,
/// `shell.main-menu.help.<button>`, `shell.game-type.help.<button>`: Load Game's the same as New
/// Game's), and the stubs' notices (T6b2: `shell.stub.<screen>`, the client's own pack's, style
/// 1), framed in the viewing player's colours; High Scores (E27 M4 T1a) with `scores`' tables and
/// creatures, its headings and Reset's question (T1b) the user's tables' (`shell.high-scores.*`).
fn menus(
    layout: &ShellLayout,
    (batch, tables): (&Batch, &h3_view::tables::Tables),
    lods: &Lods,
    (install, maps, games): (&Path, Option<&[String]>, &Path),
    (scores_tables, pictures): (ScoresTables, Pictures),
    bugs: Bugs,
) -> Result<(Menus, GameRuleset, PlayNotices, crate::saves::Broken), String> {
    let exe = crate::text::executable(install)?;
    let catalogue = crate::log::catalogue(tables, lods, &exe)?;
    // The game's rules (E27 M3 T3b1: the maps' heroes; T3b2: the hero types' classes).
    let rules = crate::maps::rules()?;
    let font = |name: &str| {
        batch
            .font(name)
            .ok_or_else(|| format!("{name}: not loaded"))
    };
    let fonts = (
        font(&layout.message.font)?,
        font(&layout.message.caption_font)?,
    );
    let laid = |key: &str, style| {
        message::layout(
            &layout.message,
            fonts,
            &catalogue.text(key, &Args::new()),
            style,
            None,
            (menu::BOX_PLAYER, bugs),
        )
    };
    let help = MAIN_MENU
        .iter()
        .map(|&(id, name, _)| {
            let key = format!("shell.main-menu.help.{name}");
            (id, laid(&key, Style::Popup))
        })
        .collect();
    let main = Menu::new(
        Kind::Main,
        layout.main_menu.dialog.clone(),
        layout.main_menu.redrawn.clone(),
        Boxes {
            question: Some(laid("shell.main-menu.quit", Style::OkCancel)),
            help,
        },
        PARK,
        bugs,
    );
    let game_help: std::collections::BTreeMap<u16, _> = GAME_TYPE
        .iter()
        .map(|&(choice, name, _)| {
            let key = format!("shell.game-type.help.{name}");
            (choice as u16, laid(&key, Style::Popup))
        })
        .collect();
    let game_type = |game| {
        let look = &layout.game_type;
        Menu::new(
            Kind::GameType(game),
            look.dialog_for(game),
            look.redrawn.clone(),
            Boxes {
                question: None,
                help: game_help.clone(),
            },
            PARK,
            bugs,
        )
    };
    let notices = menu::STUBS
        .iter()
        .map(|&(screen, _, _)| {
            let key = format!("shell.stub.{}", menu::stub_name(screen));
            (screen, laid(&key, Style::Ok))
        })
        .collect();
    // The Multiplayer dialog's and the hot seat's texts (T7).
    let mut hints = std::collections::BTreeMap::new();
    let mut help = std::collections::BTreeMap::new();
    let texted = multiplayer::TEXTED
        .iter()
        .map(|&(id, name)| (id, "multiplayer", name))
        .chain(
            multiplayer::HOT_SEAT_TEXTED
                .iter()
                .map(|&(id, name)| (id, "hot-seat", name)),
        );
    for (id, dialog, name) in texted {
        let hint = catalogue.text(&format!("shell.{dialog}.hint.{name}"), &Args::new());
        hints.insert(id, hint);
        help.insert(
            id,
            laid(&format!("shell.{dialog}.help.{name}"), Style::Popup),
        );
    }
    let texts = Texts {
        hints,
        help,
        network: laid("shell.stub.network", Style::Ok),
        title: catalogue.text("shell.hot-seat.title", &Args::new()),
    };
    // The scenario screen's texts (E27 M3 T1a) and the installation's maps.
    let text = |key: &str| catalogue.text(key, &Args::new());
    let labels = scenario::LABELS
        .iter()
        .map(|&(id, name)| {
            let word = text(&format!("shell.scenario.{name}"));
            let shown = if scenario::COLONED.contains(&id) {
                catalogue.text("shell.scenario.label", &Args::new().text("word", &word))
            } else {
                word
            };
            (id, shown)
        })
        .collect();
    let standard = text("shell.scenario.victory.0");
    let or = text("shell.scenario.or");
    let scenario_texts = scenario::Texts {
        labels,
        unnamed: text("shell.scenario.unnamed"),
        difficulties: ["easy", "normal", "hard", "expert", "impossible"]
            .iter()
            .map(|d| text(&format!("shell.scenario.difficulty.{d}")))
            .collect(),
        ratings: scenario::RATINGS
            .iter()
            .map(|&r| {
                let args = Args::new().number("rating", i64::from(r));
                catalogue.text("shell.scenario.rating", &args)
            })
            .collect(),
        victories: (0..12)
            .map(|k| text(&format!("shell.scenario.victory.{k}")))
            .collect(),
        with_standard: (1..12)
            .map(|k| {
                let goal = text(&format!("shell.scenario.victory.{k}"));
                let args = Args::new()
                    .text("goal", &goal)
                    .text("or", &or)
                    .text("standard", &standard);
                catalogue.text("shell.scenario.with-standard", &args)
            })
            .collect(),
        losses: (0..4)
            .map(|k| text(&format!("shell.scenario.loss.{k}")))
            .collect(),
        help: scenario::HELPED
            .iter()
            .map(|&(id, name)| (id, name))
            // The advanced options' items of every colour (T3c2): one key, eight items.
            .chain(
                scenario::COLOUR_HELPED
                    .iter()
                    .flat_map(|&(id, name)| (0..8u16).map(move |c| (id + c, name))),
            )
            // The random map's tab (T5b).
            .chain(scenario::RANDOM_HELPED.iter().copied())
            .map(|(id, name)| {
                let key = format!("shell.scenario.help.{name}");
                (id, laid(&key, Style::Popup))
            })
            .collect(),
        // The list's columns (T1b1): "%d/%d" by the players and the humans, the sizes' letters.
        players: (0..9i64)
            .flat_map(|p| (0..9i64).map(move |h| (p, h)))
            .map(|(p, h)| {
                let args = Args::new().number("players", p).number("humans", h);
                catalogue.text("shell.scenario.players", &args)
            })
            .collect(),
        sizes: ["small", "medium", "large", "extra-large"]
            .iter()
            .map(|s| text(&format!("shell.scenario.size.{s}")))
            .collect(),
        // The teams' dialog (T1b2): its title and "Team %d" of 1 to 8.
        teams: Some(scenario::Teams {
            look: layout.scenario.teams.clone(),
            title: text("shell.scenario.teams.title"),
            names: (1..=8i64)
                .map(|k| {
                    let args = Args::new().number("team", k);
                    catalogue.text("shell.scenario.teams.team", &args)
                })
                .collect(),
        }),
        // Begin's message over an empty list (T2b: GENRLTXT 530).
        no_maps: Some(laid("shell.scenario.no-maps", Style::Ok)),
        // Begin's messages over a map (T4b: GENRLTXT 531, 83).
        unplaced: Some(laid("shell.scenario.unplaced", Style::Ok)),
        too_few: Some(laid("shell.scenario.too-few", Style::Ok)),
        // The description's split (T2c): the font's metrics alone.
        description_font: Some(metrics(font(&layout.scenario.description.font)?)),
        // The advanced options (T3a).
        advanced: Some(advanced_texts(
            &layout.scenario.advanced,
            (tables, &rules),
            &catalogue,
        )),
        // A random map's name and description (T5a).
        random: [
            text("shell.scenario.random-name"),
            text("shell.scenario.random-description"),
        ],
        // Begin with a random map chosen (T5b): the client's stub (CLT-Q92).
        random_stub: Some(laid("shell.stub.random-map", Style::Ok)),
        // The load mode (E27 M4 T4b1): its title, the autosave's name, a save's date (its
        // placeholders kept for the screen to fill); T4b2: Delete's question (GENRLTXT 688, the
        // file's name the screen's to fill) and the notice with no saves (686).
        load_title: text("shell.scenario.load-title"),
        autosave: text("shell.scenario.autosave"),
        date: catalogue.text(
            "shell.scenario.date",
            &["month", "day", "year", "hour", "minute"]
                .iter()
                .fold(Args::new(), |args, name| {
                    args.text(name, &format!("{{{{{name}}}}}"))
                }),
        ),
        delete: Some(scenario::Asking {
            layout: layout.message.clone(),
            fonts: [metrics(fonts.0), metrics(fonts.1)],
            text: catalogue.text(
                "shell.scenario.delete",
                &Args::new().text("file", scenario::FILE),
            ),
        }),
        no_saves: Some(laid("shell.scenario.no-saves", Style::Ok)),
    };
    // A save the client cannot list (E27 M4 T4b1: GENRLTXT 431, 505).
    let broken = crate::saves::Broken {
        name: text("shell.scenario.broken-name"),
        description: text("shell.scenario.broken"),
    };
    // The load mode's screen (E27 M4 T4b1): the saves of `games`, their dates at the zone's
    // present offset.
    let offset = h3_sdl::local_offset().unwrap_or(0);
    let load = Scenario::new(
        layout.scenario.dialog.clone(),
        (
            layout.scenario.list.clone(),
            layout.scenario.description.clone(),
        ),
        scenario_texts.clone(),
        crate::saves::entries(games, install, &rules, &broken),
        bugs,
    )
    .loading(&layout.scenario.load_begin, offset);
    // Play's notices: the engine's refusal of the begin's setup (T6), a save loaded and a save
    // the load refuses (E27 M4 T4b2).
    let play = PlayNotices {
        refused: laid("shell.stub.play-refused", Style::Ok),
        loaded: laid("shell.stub.loaded", Style::Ok),
        load_refused: laid("shell.stub.load-refused", Style::Ok),
    };
    let menus = Menus {
        main,
        new_game: game_type(Game::New),
        load_game: game_type(Game::Load),
        notices,
        multiplayer: Multiplayer::new(
            layout.multiplayer.clone(),
            layout.hot_seat.clone(),
            texts,
            bugs,
        ),
        scenario: Scenario::new(
            layout.scenario.dialog.clone(),
            (
                layout.scenario.list.clone(),
                layout.scenario.description.clone(),
            ),
            scenario_texts,
            crate::maps::named(crate::maps::entries(install, &rules), maps, &rules),
            bugs,
        )
        .with_random_offers(crate::maps::random_offers(&rules)),
        load,
        // High Scores (E27 M4 T1a).
        scores: Scores::new(
            layout.high_scores.clone(),
            scores::Texts {
                rank: text("shell.high-scores.rank"),
                player: text("shell.high-scores.player"),
                land: text("shell.high-scores.land"),
                days: text("shell.high-scores.days"),
                campaign: text("shell.high-scores.campaign"),
                score: text("shell.high-scores.score"),
            },
            scores_tables.defaults.clone(),
            pictures,
            laid("shell.high-scores.reset", Style::OkCancel),
            bugs,
        )
        .with_tables(scores_tables.tables.unwrap_or(scores_tables.defaults)),
        // The credits (E27 M4 T2): Credits.txt's two records, the text's lines counted in its
        // font.
        credits: Credits::laid(
            layout.credits.clone(),
            credits::Texts {
                text: text("shell.credits.text"),
                closing: text("shell.credits.closing"),
            },
            font(&layout.credits.font)?,
            bugs,
        ),
    };
    Ok((menus, rules, play, broken))
}

/// Play's notices in place of the stub's `shell.stub.play` (CLT-STUB-13; the client's own words):
/// the engine's refusal of the begin's setup (T6), a save loaded, a save the load refuses (E27
/// M4 T4b2, *provisional*: the original's load and its messages are M5's).
#[derive(Debug, Clone)]
struct PlayNotices {
    /// `shell.stub.play-refused`.
    refused: MessageBox,
    /// `shell.stub.loaded`.
    loaded: MessageBox,
    /// `shell.stub.load-refused`.
    load_refused: MessageBox,
}

/// The advanced options' texts (E27 M3 T3a): the catalogue's `shell.advanced.*` and, for the
/// towns' popups, each creature's singular name by its original id (the asset tables' key); for a
/// hero's popup (T3b2) each hero type's class's name (its class in `rules`) and specialty's name.
fn advanced_texts(
    look: &scenario::AdvancedLook,
    (tables, rules): (&h3_view::tables::Tables, &GameRuleset),
    catalogue: &h3_ui::text::Catalogue,
) -> scenario::AdvancedTexts {
    let text = |key: &str| catalogue.text(key, &Args::new());
    let ids: std::collections::BTreeSet<u16> = look
        .town_popup
        .creatures
        .iter()
        .flatten()
        .copied()
        .collect();
    let creature_names = ids
        .into_iter()
        .map(|id| {
            let name = tables
                .creatures
                .iter()
                .find(|(_, c)| c.original == id)
                .map(|(key, _)| catalogue.text(&format!("creature.{key}"), &Args::new().count(1)))
                .unwrap_or_default();
            (id, name)
        })
        .collect();
    let resource = |name: &str| {
        [
            text(&format!("shell.advanced.resource.{name}.caption")),
            text(&format!("shell.advanced.resource.{name}.text")),
        ]
    };
    // A faction's resource (design §3.15): wood and ore, crystal (Rampart), gems (Tower), mercury
    // (Inferno, Conflux), sulfur (Dungeon).
    let resources = [
        "wood-ore", "crystal", "gems", "mercury", "wood-ore", "sulfur", "wood-ore", "wood-ore",
        "mercury",
    ]
    .map(resource)
    .to_vec();
    let popup = |bonus: &str, caption: bool, body: bool| {
        let key = |part: &str| format!("shell.advanced.bonus-popup.{bonus}.{part}");
        [
            text(&key("title")),
            if caption {
                text(&key("caption"))
            } else {
                String::new()
            },
            if body {
                text(&key("text"))
            } else {
                String::new()
            },
        ]
    };
    // The hero types by their original ids (E27 M3 T3b1): their names and small portraits, in
    // type order.
    let mut heroes: Vec<(u16, &str, Option<&String>)> = tables
        .heroes
        .iter()
        .map(|(key, h)| (h.original, key.as_str(), h.small_portrait.as_ref()))
        .collect();
    heroes.sort_unstable_by_key(|h| h.0);
    scenario::AdvancedTexts {
        look: look.clone(),
        hero_names: heroes
            .iter()
            .map(|(_, key, _)| catalogue.text(&format!("hero.{key}"), &Args::new()))
            .collect(),
        portraits: heroes
            .iter()
            .map(|(_, _, p)| p.cloned().unwrap_or_default())
            .collect(),
        random_hero: [
            text("shell.advanced.random-hero.title"),
            text("shell.advanced.random-hero.text"),
        ],
        hero_popup: [
            text("shell.advanced.hero-popup.title"),
            text("shell.advanced.hero-popup.specialty"),
        ],
        hero_line: catalogue.text(
            "shell.advanced.hero-popup.name",
            &Args::new()
                .text("name", &String::from(scenario::HERO_NAME))
                .text("class", &String::from(scenario::HERO_CLASS)),
        ),
        // The hero types' classes as the game's rules give them, in type order (the engine's
        // hero type and class numbers are the original's).
        hero_classes: rules
            .core()
            .hero_types()
            .iter()
            .map(|h| text(&format!("hero-class.{}", h.class.index())))
            .collect(),
        specialties: heroes
            .iter()
            .map(|(_, key, _)| text(&format!("hero-specialty.{key}")))
            .collect(),
        kinds: ["both", "human", "computer"].map(|k| text(&format!("shell.advanced.kind.{k}"))),
        computer: text("shell.advanced.computer"),
        handicaps: [0, 1, 2].map(|h| text(&format!("shell.advanced.handicap.{h}"))),
        random: text("shell.advanced.random"),
        none: text("shell.advanced.none"),
        towns: (0..9)
            .map(|t| text(&format!("shell.advanced.town.{t}")))
            .collect(),
        bonuses: ["artifact", "gold", "resource"]
            .iter()
            .map(|b| text(&format!("shell.advanced.bonus.{b}")))
            .collect(),
        durations: (0..11)
            .map(|d| text(&format!("shell.advanced.duration.{d}")))
            .collect(),
        alignment: text("shell.advanced.alignment"),
        creatures: text("shell.advanced.creatures"),
        creature_names,
        random_town: [
            text("shell.advanced.random-town.title"),
            text("shell.advanced.random-town.text"),
        ],
        bonus_popups: [
            popup("artifact", false, true),
            popup("gold", true, true),
            popup("resource", false, false),
            popup("random", false, true),
        ],
        resources,
    }
}

/// The heroes' small portraits the asset tables name (E27 M3 T3b1), in their keys' order.
fn portraits(tables: &h3_view::tables::Tables) -> Vec<String> {
    tables
        .heroes
        .values()
        .filter_map(|h| h.small_portrait.clone())
        .collect()
}

/// A font's metrics alone (its glyphs' pixels dropped): what a split measures.
fn metrics(font: &h3_assets::fnt::Font) -> h3_assets::fnt::Font {
    let mut metrics = font.clone();
    for glyph in &mut metrics.glyphs {
        glyph.pixels = Vec::new();
    }
    metrics
}

/// A start clip parsed.
enum Parsed<'a> {
    Smacker(h3_smacker::Clip<'a>),
    Bink(h3_bink::Clip<'a>),
}

/// The start's clips parsed (none for one missing or not readable as its format: the original
/// ends the sequence at a clip it cannot open).
fn parse_clips(bytes: &[Option<Vec<u8>>; 3]) -> [Option<Parsed<'_>>; 3] {
    let mut parsed = [None, None, None];
    for (slot, bytes) in parsed.iter_mut().zip(bytes) {
        *slot = bytes.as_deref().and_then(|b| {
            h3_smacker::Clip::parse(b)
                .map(Parsed::Smacker)
                .or_else(|_| h3_bink::Clip::parse(b).map(Parsed::Bink))
                .ok()
        });
    }
    parsed
}

impl Parsed<'_> {
    /// The clip's sound in bytes a second (E27 M2 T5b1), none without a track: a Smacker clip's
    /// first track present, as `SMACKW32.DLL` counts it (16-bit and stereo doubling the rate);
    /// a Bink clip's first.
    fn sound(&self) -> Option<u32> {
        match self {
            Self::Smacker(clip) => clip
                .tracks
                .iter()
                .find(|t| t.rate != 0 && t.flags & 0x40 != 0)
                .map(|t| {
                    t.rate * (1 + u32::from(t.flags >> 5 & 1)) * (1 + u32::from(t.flags >> 4 & 1))
                }),
            Self::Bink(clip) => clip.tracks.first().map(|t| u32::from(t.rate) * 4),
        }
    }

    const fn format(&self) -> h3_view::clip_sound::Format {
        match self {
            Self::Smacker(_) => h3_view::clip_sound::Format::Smacker,
            Self::Bink(_) => h3_view::clip_sound::Format::Bink,
        }
    }

    /// What the start's timing takes of the header (a Smacker clip's ring frame not counted, as
    /// the handle's +0xC).
    fn header(&self) -> Header {
        match self {
            Self::Smacker(clip) => Header {
                size: [clip.width, clip.height],
                frames: u32::try_from(clip.frames() - usize::from(clip.ring())).unwrap_or(0),
                pace: Pace::Smacker(clips::smack_period(clip.rate)),
            },
            Self::Bink(clip) => Header {
                size: [clip.width, clip.height],
                frames: u32::try_from(clip.frames().len()).unwrap_or(0),
                pace: Pace::Bink {
                    num: clip.rate_num,
                    den: clip.rate_den,
                },
            },
        }
    }
}

/// The start from its clips at 0 ms, and its picture (`bugs` the DLLs' copies or whole frames);
/// with `sound` the clips play with their sound (E27 M2 T5b1).
fn start_of<'c, 'a>(
    parsed: &'c [Option<Parsed<'a>>; 3],
    rim: Option<&'c Pixels>,
    bink_video: bool,
    bugs: Bugs,
    sound: bool,
) -> Result<((Start, Vec<Op>), Picture<'c, 'a>), String> {
    let sounds = parsed
        .each_ref()
        .map(|p| p.as_ref().filter(|_| sound).and_then(Parsed::sound));
    let headers = parsed.each_ref().map(|p| p.as_ref().map(Parsed::header));
    // The residues as the DLL reads them for the comparisons (CLT-Q68).
    let reading = match bugs {
        Bugs::Fixed => h3_bink::Reading::Fixed,
        Bugs::Original => h3_bink::Reading::Original,
    };
    let mut sources = [None, None, None];
    for (slot, p) in sources.iter_mut().zip(parsed) {
        *slot = match p {
            Some(Parsed::Smacker(clip)) => {
                h3_smacker::Decoder::new(clip)
                    .ok()
                    .map(|decoder| Source::Smacker {
                        reader: Box::new(clip.reader()),
                        decoder: Box::new(decoder),
                    })
            }
            Some(Parsed::Bink(clip)) => h3_bink::Decoder::new(clip)
                .ok()
                .map(|decoder| decoder.reading(reading))
                .map(|decoder| Source::Bink {
                    clip,
                    decoder: Box::new(decoder),
                    next: 0,
                }),
            None => None,
        }
        .zip(p.as_ref().map(Parsed::header));
    }
    // A clip whose decoder refuses it does not open.
    let headers = {
        let mut h = headers;
        for (header, source) in h.iter_mut().zip(&sources) {
            if source.is_none() {
                *header = None;
            }
        }
        h
    };
    let black = Pixels::new(WIDTH, HEIGHT, FORMAT).ok_or("no 800 × 600 screen")?;
    let picture = Picture::new(black, sources, rim, bink_video, bugs);
    let mut ops = Vec::new();
    let start = Start::with_sound(headers, sounds, bink_video, bugs, 0, &mut ops);
    Ok(((start, ops), picture))
}

/// Carries out the ops `picture` has not yet.
fn catch_up(picture: &mut Picture<'_, '_>, ops: &[Op], applied: &mut usize) -> Result<(), String> {
    for op in ops.get(*applied..).unwrap_or_default() {
        picture
            .apply(*op)
            .map_err(|e| format!("the start's {op:?}: {e:?}"))?;
    }
    *applied = ops.len();
    Ok(())
}

/// The frame's pixels as the harness's dumps hold them: 5-6-5, little-endian, row by row.
fn raw(pixels: &Pixels) -> Vec<u8> {
    pixels
        .pixels()
        .iter()
        .flat_map(|p| p.to_le_bytes())
        .collect()
}

/// Why the replay stopped, a line of `replay.txt`.
fn stopped(k: usize, stop: &Stop) -> String {
    match stop {
        Stop::Harness(verb) => format!("stop {k} harness {verb}"),
        Stop::NoButton(name) => format!("stop {k} no-button {name}"),
        Stop::Unknown(name) => format!("stop {k} unknown {name}"),
        Stop::NotBuilt(screen) => {
            format!(
                "stop {k} not-built {} ({})",
                text::name(*screen),
                screen.task()
            )
        }
        // The network modes' notice (CLT-STUB-6), Begin's with a random map chosen (E27 M3 T5b:
        // the original generates it; CLT-Q92, decision 13), or play's after a begin or a load
        // (CLT-STUB-13).
        Stop::Stub(screen) => {
            let why = match screen {
                Screen::Scenario(..) => "a random map: CLT-Q92, no generator",
                Screen::Play { .. } => "CLT-STUB-13, E24 M5: play",
                _ => "CLT-STUB-6, E28",
            };
            format!(
                "stop {k} stub {} (the client's notice: {why})",
                text::name(*screen)
            )
        }
        Stop::NotShown { screens, screen } => {
            format!(
                "stop {k} not-shown {} on {}",
                screens.join("|"),
                text::name(*screen)
            )
        }
    }
}

/// A recipe's steps into frames (T4).
fn replay_steps(options: &Options, drawer: &Drawer) -> Result<(), String> {
    let Mode::Replay {
        steps,
        frames,
        pointer,
        input_step,
        backgrounds,
        ..
    } = &options.mode
    else {
        return Err("not a replay".into());
    };
    let text = fs::read_to_string(steps).map_err(|e| format!("{}: {e}", steps.display()))?;
    let parsed = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(replay::parse)
        .collect::<Result<Vec<_>, _>>()?;
    fs::create_dir_all(frames).map_err(|e| format!("{}: {e}", frames.display()))?;
    let write = |path: &Path, bytes: &[u8]| {
        fs::write(path, bytes).map_err(|e| format!("{}: {e}", path.display()))
    };
    let mut listing = Vec::new();
    let mut last: Option<Vec<u16>> = None;
    let bytes = if options.intro {
        drawer.clips(options.bink_video)
    } else {
        [None, None, None]
    };
    let clips = parse_clips(&bytes);
    let sound = options.effects.unwrap_or(0) > 0;
    let (start, mut picture) = start_of(
        &clips,
        drawer.rim.as_ref(),
        options.bink_video,
        drawer.bugs,
        sound,
    )?;
    let mut applied = 0;
    let shell = Shell::new(options.intro);
    let mut r = Replay::new(shell, *pointer, *input_step, Some(start), &drawer.menus)
        .with_backgrounds(backgrounds);
    if sound && let Some(ms) = drawer.button_ms() {
        r = r.with_sound(ms);
    }
    for (k, step) in parsed.iter().enumerate() {
        let mut take = |r: &Replay, t: Take<'_>| -> Result<(), String> {
            catch_up(&mut picture, r.ops(), &mut applied)?;
            let pixels = if r.screen() == Screen::Start {
                picture.screen().clone()
            } else {
                drawer.frame(r.screen(), Some(r.pointer()), r.front())?
            };
            match t {
                Take::Shot(name) => {
                    write(&frames.join(format!("{name}.565")), &raw(&pixels))?;
                    listing.push(format!("shot {name} t{}", r.time()));
                }
                Take::Clip { name, first } => {
                    if first || last.as_deref() != Some(pixels.pixels()) {
                        let dir = frames.join(name);
                        fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
                        write(&dir.join(format!("t{}.565", r.time())), &raw(&pixels))?;
                        listing.push(format!("clip {name} t{}", r.time()));
                        last = Some(pixels.pixels().to_vec());
                    }
                }
            }
            Ok(())
        };
        if let Some(stop) = r.play(step, &mut take)? {
            listing.push(stopped(k, &stop));
            break;
        }
    }
    // The clips' sound (E27 M2 T5b1): when each started, at what volume (T5b2: 0 to 127, the
    // index of `SMACKW32.DLL`'s table or Miles' volume), and when it stopped: a skip (`quiet`) or
    // the close after its last frame (`closed`), as the original's (`h3shell.py`'s voices).
    let effects = options.effects.unwrap_or(0);
    for (op, t) in r.ops().iter().zip(r.op_times()) {
        let line = match *op {
            Op::Sound(part) => {
                let volume = clips[part as usize].as_ref().map_or(0, |clip| {
                    h3_view::clip_sound::volume(clip.format(), effects, drawer.bugs)
                });
                format!("sound {} t{t} v{volume}", part.file(options.bink_video))
            }
            Op::Quiet(part) => format!("quiet {} t{t}", part.file(options.bink_video)),
            Op::Closed(part) => format!("closed {} t{t}", part.file(options.bink_video)),
            _ => continue,
        };
        listing.push(line);
    }
    // The press samples (T6a), as the harness logs the original's.
    for p in r.samples() {
        listing.push(format!(
            "sample {BUTTON} t{} ch{} {}ms x1",
            p.time, p.channel, p.length
        ));
    }
    let listing: String = listing.iter().map(|l| format!("{l}\n")).collect();
    print!("{listing}");
    write(&frames.join("replay.txt"), listing.as_bytes())
}

/// The shell in a window (T4): the built screens drawn, the words from standard input; the
/// start's clips on the window's clock (T5a), a key or a left press skipping them.
fn in_window(
    options: &Options,
    scale: Option<u32>,
    drawer: &Drawer,
    scores: &Path,
) -> Result<(), String> {
    let mut shell = Shell::new(options.intro);
    // The menus as they open, the scenario screen's kept map carried from one to the next.
    let mut menus = drawer.menus.clone();
    let bytes = if options.intro {
        drawer.clips(options.bink_video)
    } else {
        [None, None, None]
    };
    let clips = parse_clips(&bytes);
    // The sound (E27 M2 T5b1): the clips and the presses' samples with the effects above 0 and
    // a device, else silent (the game opens no track without a driver, and its Smacker clips
    // keep their clock); the menus' music (T6a) at the music setting (a first run's: no client
    // settings yet) with the device, opened for the effects or the music above 0 (M2's review of
    // E27: the music needed the effects before).
    let effects = options.effects.unwrap_or(5);
    let music_setting = h3_view::options::Settings::FIRST_RUN.music;
    let mut host = if effects > 0 || music_setting > 0 {
        match crate::audio::HostSound::open(drawer.bugs) {
            Ok(host) => Some(host),
            Err(e) => {
                eprintln!("h3-view shell: no sound ({e})");
                None
            }
        }
    } else {
        None
    };
    // The clips and the presses heard: the effects above 0 and the device.
    let effects_heard = effects > 0 && host.is_some();
    // Each clip's track decoded beside the window (H3Intro's takes seconds), taken at its start.
    let mut samples = [0, 1, 2].map(|k| {
        let format = clips[k].as_ref()?.format();
        let bytes = bytes[k].clone()?;
        effects_heard.then_some(())?;
        thread::Builder::new()
            .name("h3-view shell's clip sound".to_owned())
            .spawn(move || {
                let samples = match format {
                    h3_view::clip_sound::Format::Smacker => {
                        h3_view::clip_sound::ClipSound::smacker(&bytes)
                            .ok()
                            .and_then(|tracks| tracks.into_iter().next())
                    }
                    h3_view::clip_sound::Format::Bink => {
                        h3_view::clip_sound::ClipSound::bink(&bytes).ok().flatten()
                    }
                };
                samples.map(|s| (s, format))
            })
            .ok()
    });
    let mut streams: [Option<usize>; 3] = [None; 3];
    // The menus' music decoded beside the window (T6a), added once decoded (the music waits for
    // it); the presses' samples on streams of their own, each removed once played.
    let install = options.install.clone();
    let mut music = host.as_ref().and_then(|_| {
        thread::Builder::new()
            .name("h3-view shell's music".to_owned())
            .spawn(move || crate::audio::mp3(&install, "MAINMENU", &mut Vec::new()))
            .ok()
    });
    let music_volume = h3_view::options::sound_volume(h3_view::sound::MUSIC_VOLUME, music_setting);
    let mut presses: Vec<usize> = Vec::new();
    // The user's key file over the defaults, every key one the adapter knows (M2's review of
    // E27: the battle window's `load_keys`, the shell's `[shell]` table).
    let (keys, keys_from) = crate::window::load_keys::<Command>(options.keys.as_deref())?;
    eprintln!("h3-view shell: keys: {keys_from}");
    // What takes the inputs on the screen, opened as its screen shows (`on`: the screen it was
    // opened for; `previous`: what took them before, the Multiplayer dialog's menu).
    let mut front: Option<Front> = None;
    let mut previous: Option<Front> = None;
    let ((mut start, mut ops), mut picture) = start_of(
        &clips,
        drawer.rim.as_ref(),
        options.bink_video,
        drawer.bugs,
        effects_heard,
    )?;
    let mut applied = 0;
    let mut heard = 0;
    if start.ended() {
        let _ = shell.end(Ended::Start);
    }
    let mut on: Option<Screen> = None;
    println!("{}", text::describe(shell.screen()));
    let (sender, words) = mpsc::channel();
    // Standard input's reader, beside the window's loop (as the connection's I/O thread, `net`).
    thread::Builder::new()
        .name("h3-view shell's words".to_owned())
        .spawn(move || {
            for line in io::stdin().lock().lines() {
                let Ok(line) = line else { break };
                if sender.send(line).is_err() {
                    break;
                }
            }
        })
        .map_err(|e| format!("the words' thread: {e}"))?;
    let mut win = Window::open(&Config {
        title: String::from("h3-view shell"),
        width: WIDTH,
        height: HEIGHT,
        scale,
        vsync: true,
        fullscreen: false,
    })?;
    let drawn = win.set_cursor(Some(drawer.hardware.clone())).is_err();
    // The start's clock: the window's, from its opening (the start read at 0 ms there).
    let began = win.ticks_ns();
    let now = |win: &Window| {
        u32::try_from(win.ticks_ns().saturating_sub(began) / 1_000_000).unwrap_or(u32::MAX)
    };
    let (mut pointer, mut dirty, mut cursor_hidden) = (None, true, false);
    let mut cursor_arrow = Arrow::Default;
    // The inputs given during the start's last fade, the main menu's first (CLT-Q130 (a)).
    let mut kept: Vec<Given> = Vec::new();
    // The begin's choices (T6) until play's start, and the game it began, kept for E24 M5.
    let mut begun: Option<Begun> = None;
    let mut game: Option<crate::begin::Game> = None;
    // The save the load mode's Load named, and the game it loaded, kept for E24 M5 (E27 M4 T4b2).
    let mut to_load: Option<PathBuf> = None;
    let mut loaded: Option<crate::save::Loaded> = None;
    loop {
        let playing = shell.screen() == Screen::Start;
        // Play's fade (T6) steps every 50 ms, the credits every 100 (E27 M4 T2): the window's
        // wait as the start's.
        let fading = matches!(&front, Some(Front::Play(p)) if !p.noticing())
            || matches!(&front, Some(Front::Credits(_)));
        // The edits' typing (T7b): text input while one has the focus.
        win.set_text_input(front.as_ref().is_some_and(Front::typing));
        // A held key's repeats on the scenario screen (E27 M4 T4b1: the original queues every key
        // down, `input_key_message` 0x4EC0E0, a repeat's too; *provisional*: the other screens
        // take none, design §16.4).
        win.set_key_repeats(matches!(front, Some(Front::Scenario(_))));
        let events = win.wait(if playing || fading { 5 } else { 50 });
        let mut inputs = if playing {
            Vec::new()
        } else {
            mem::take(&mut kept)
        };
        inputs.extend(merged(events, win.scans(), win.typed()));
        for given in inputs {
            let mut did = menu::Did::default();
            match given {
                Given::Typed(c) => {
                    if let Some(front) = front.as_mut() {
                        did = front.typed(c);
                    }
                }
                Given::Event(Event::Quit, _) => return Ok(()),
                Given::Event(Event::Pointer(at), _) => {
                    pointer = at.map(|(x, y)| [x, y]);
                    dirty |= drawn;
                    if let (Some(front), Some(at)) = (front.as_mut(), pointer) {
                        did = front.moved(at);
                    }
                }
                Given::Event(Event::Redraw, _) => dirty = true,
                // During the start's last fade an input waits for the main menu, which opens
                // at its end (`Start::keeps_input`).
                Given::Event(event, scan) if playing && start.keeps_input() => {
                    kept.push(Given::Event(event, scan));
                }
                // A key down or a press goes to the clip, which only some stop (F4 does not).
                Given::Event(event @ (Event::Key(_) | Event::Click { down: true, .. }), _)
                    if playing =>
                {
                    if skips_start(&event) {
                        start.skip(now(&win), &mut ops);
                    }
                }
                Given::Event(event, scan) => {
                    if let Some(front) = front.as_mut() {
                        // The scenario list's double press is timed on the window's clock (T1b1);
                        // the credits step and end by it (E27 M4 T2).
                        let ticked = front.clock(now(&win));
                        did = if ticked.ended.is_some() {
                            ticked
                        } else {
                            let given = (&event, scan);
                            let did = on_front(front, &keys, given, pointer.unwrap_or(PARK));
                            menu::Did {
                                changed: did.changed || ticked.changed,
                                ..did
                            }
                        };
                    }
                }
            }
            dirty |= did.changed;
            // High Scores' Reset (E27 M4 T1b): the scores file written whole as its OK resets
            // the tables (the original's HiScore.dat; a failure told, *provisional*: the
            // original's `file_error_box`).
            if let Some(Front::Scores(s)) = front.as_mut()
                && let Some(text) = s.take_save()
                && let Err(e) = crate::window::write_whole(scores, &text)
            {
                eprintln!("h3-view shell: scores: {}: {e}", scores.display());
            }
            // The load mode's Delete (E27 M4 T4b2): its OK deletes the file from the client's own
            // folder (the original does not check its `DeleteFileA`: a failure told).
            if let Some(Front::Scenario(s)) = front.as_mut()
                && let Some(file) = s.take_deleted()
                && let Err(e) = crate::saves::delete(&drawer.games, &file)
            {
                eprintln!("h3-view shell: delete: {e}");
            }
            if did.sample
                && effects_heard
                && let (Some(host), Some(pcm)) = (host.as_mut(), drawer.button.as_ref())
            {
                let volume = h3_view::options::sound_volume(BUTTON_VOLUME, effects);
                let sound = h3_view::clip_sound::ClipSound {
                    rate: pcm.rate,
                    channels: pcm.channels,
                    samples: pcm.samples.clone(),
                };
                presses.extend(host.play_clip(&sound, volume).ok());
            }
            if let Some(ended) = did.ended {
                // The begin's choices (T6), or the load's save (E27 M4 T4b2), for play's start.
                if let (Ended::Scenario(true), Some(Front::Scenario(s))) = (ended, front.as_ref()) {
                    begun = s.begun();
                    to_load = s.loaded().map(|file| drawer.games.join(file));
                }
                // The game kept for E24 M5 left as play's stub ends.
                if let (Ended::Play(_), Some(game)) = (ended, game.take()) {
                    eprintln!("h3-view shell: {} left (E24 M5 hosts it)", game.file);
                }
                if let (Ended::Play(_), Some(loaded)) = (ended, loaded.take()) {
                    eprintln!("h3-view shell: {} left (E24 M5 hosts it)", loaded.game.file);
                }
                let _ = shell.end(ended);
                println!("{}", text::describe(shell.screen()));
                if shell.screen() == Screen::End {
                    return Ok(());
                }
                on = None;
                previous = front.take();
                dirty = true;
            }
        }
        if let Some(host) = host.as_mut() {
            if music.as_ref().is_some_and(thread::JoinHandle::is_finished) {
                let decoded = music.take().and_then(|t| t.join().ok());
                let _ = host.add_track(MAIN_MUSIC, decoded.and_then(Result::ok).flatten());
            }
            host.service(music_volume);
            presses.retain(|&stream| {
                let playing = host.clip_playing(stream);
                if !playing {
                    host.stop_clip(stream);
                }
                playing
            });
        }
        if shell.screen() == Screen::Start {
            start.read(now(&win), &mut ops);
        }
        // The clips' sound started and stopped as the ops say.
        if let Some(host) = host.as_mut() {
            for op in ops.get(heard..).unwrap_or_default() {
                match *op {
                    Op::Sound(part) => {
                        let decoded = samples[part as usize].take().and_then(|t| t.join().ok());
                        if let Some(Some((sound, format))) = decoded {
                            let volume = h3_view::clip_sound::volume(format, effects, drawer.bugs);
                            streams[part as usize] = host.play_clip(&sound, volume).ok();
                        }
                    }
                    // A skip stops the sound; the close after the last frame cuts it in the
                    // original's only (CLT-Q125, decision 34: the track plays out).
                    Op::Quiet(part) | Op::Closed(part)
                        if matches!(op, Op::Quiet(_)) || drawer.bugs == Bugs::Original =>
                    {
                        if let Some(stream) = streams[part as usize].take() {
                            host.stop_clip(stream);
                        }
                    }
                    _ => {}
                }
            }
        }
        heard = ops.len();
        if shell.screen() == Screen::Start {
            dirty |= applied < ops.len();
            catch_up(&mut picture, &ops, &mut applied)?;
            if start.ended() {
                let _ = shell.end(Ended::Start);
                println!("{}", text::describe(shell.screen()));
                dirty = true;
            }
        }
        while let Ok(line) = words.try_recv() {
            let before = shell.screen();
            if let Some(refusal) = refused_word(before, &line) {
                eprintln!("h3-view shell: {refusal}");
                println!("{}", text::describe(before));
                continue;
            }
            println!("{}", text::answer(&mut shell, &line));
            if shell.screen() == Screen::End {
                return Ok(());
            }
            if shell.screen() != before {
                on = None;
                previous = front.take();
            }
            dirty = true;
        }
        // The help popup hides the cursor (`popup_run_held`): the hardware one too; play's
        // loading screen and fade too (T6).
        let hidden = front.as_ref().is_some_and(|f| match f {
            Front::Menu(m) => !m.cursor(),
            Front::Multiplayer(m) => !m.cursor(),
            Front::Scenario(s) => !s.cursor(),
            Front::Play(p) => !p.noticing() || !p.menu().cursor(),
            Front::Scores(_) | Front::Credits(_) => false,
        });
        // Which arrow (CLT-Q137): the adventure map's after the scenario screen's busy cursor.
        let arrow = front.as_ref().map(Front::hovers).unwrap_or_default().arrow;
        if !drawn && (hidden != cursor_hidden || arrow != cursor_arrow) {
            let image = (!hidden).then(|| match arrow {
                Arrow::Default => drawer.hardware.clone(),
                Arrow::Adventure => drawer.adventure_hardware.clone(),
            });
            win.set_cursor(image)?;
            cursor_hidden = hidden;
            cursor_arrow = arrow;
        }
        // A screen shown anew (the start's end, a menu's, a word's): its menu opened at the
        // pointer, before it is drawn, its music asked for (the track playing goes on:
        // E24-design §3.11).
        if on != Some(shell.screen()) {
            on = Some(shell.screen());
            // The load mode's saves listed anew at each opening (E27 M4 T4b1: the original reads
            // its folder at each one).
            if shell.screen() == Screen::Scenario(Game::Load, h3_view::shell::Kind::Single) {
                let install = options.install.as_path();
                let saves =
                    crate::saves::entries(&drawer.games, install, &drawer.rules, &drawer.broken);
                menus.load.set_entries(saves);
            }
            front = menus.front(shell.screen(), previous.take(), pointer.unwrap_or(PARK));
            // The scenario screen's background, the client's own draw (E27 M3 T1a, a look):
            // `random_between(1, 100) < 51` gives `gamselb1.pcx`.
            if let Some(Front::Scenario(s)) = front.as_mut() {
                **s = s.as_ref().clone().with_background(background_draw());
            }
            // Play's start (T6): the loading screen's bar stepped as the begin's work goes, the
            // setup kept for E24 M5 or its refusal the notice's, then the bar full and the fade.
            if matches!(front, Some(Front::Play(_))) {
                let screen = shell.screen();
                let mut show = |front: &mut Option<Front>, step: bool| {
                    if let (Some(Front::Play(p)), true) = (front.as_mut(), step) {
                        p.step();
                    }
                    if let Ok(frame) = drawer.frame(screen, None, front.as_ref()) {
                        let _ = win.present(&frame.xrgb());
                    }
                };
                show(&mut front, false);
                let at = (options.install.as_path(), &drawer.rules);
                let outcome = begun
                    .take()
                    .map(|b| begun_game(&b, at, &mut || show(&mut front, true)));
                match outcome {
                    Some(Ok(begun)) => {
                        eprintln!(
                            "h3-view shell: {} begun, {} players: the setup kept for E24 M5",
                            begun.file,
                            begun.setup.players.len()
                        );
                        game = Some(begun);
                    }
                    Some(Err(e)) => {
                        eprintln!("h3-view shell: the game's setup refused: {e}");
                        if let Some(Front::Play(p)) = front.as_mut() {
                            p.notice(drawer.play.refused.clone());
                        }
                    }
                    None => {}
                }
                // A load (E27 M4 T4b2): the save read and its game loaded by the engine
                // (`save::load`), kept for E24 M5; the client's own notice either way.
                let outcome = to_load
                    .take()
                    .map(|path| loaded_game(&path, at, &mut || show(&mut front, true)));
                let notice = match outcome {
                    Some(Ok(game)) => {
                        eprintln!(
                            "h3-view shell: {} loaded, day {}: the game kept for E24 M5",
                            game.game.file,
                            game.state.day()
                        );
                        loaded = Some(game);
                        Some(&drawer.play.loaded)
                    }
                    Some(Err(e)) => {
                        eprintln!("h3-view shell: the save refused: {e}");
                        Some(&drawer.play.load_refused)
                    }
                    None => None,
                };
                if let (Some(Front::Play(p)), Some(notice)) = (front.as_mut(), notice) {
                    p.notice(notice.clone());
                }
                if let Some(Front::Play(p)) = front.as_mut() {
                    p.loaded(now(&win));
                }
            }
            if let Some(front) = front.as_mut() {
                front.clock(now(&win));
            }
            if let (Some(host), true) = (host.as_mut(), front.is_some()) {
                let _ = host.request(MAIN_MUSIC, false);
            }
            dirty = true;
        }
        // Play's fade (T6), High Scores' creatures (E27 M4 T1a) and the credits (T2) on the
        // window's clock; the credits' end the main menu, opened at the next turn.
        if let Some(f @ (Front::Play(_) | Front::Scores(_) | Front::Credits(_))) = front.as_mut() {
            let ticked = f.clock(now(&win));
            dirty |= ticked.changed;
            if let Some(ended) = ticked.ended {
                let _ = shell.end(ended);
                println!("{}", text::describe(shell.screen()));
                on = None;
                previous = front.take();
                continue;
            }
        }
        if dirty {
            let frame = if shell.screen() == Screen::Start {
                picture.screen().clone()
            } else {
                drawer.frame(shell.screen(), pointer.filter(|_| drawn), front.as_ref())?
            };
            win.present(&frame.xrgb())?;
            dirty = false;
        }
    }
}

/// The game `begun` begins (T6; `crate::begin`): its map read from `install`'s copy, its setup
/// made and validated under `rules`, each a step of the loading screen's bar (`step`: the
/// client's steps).
///
/// # Errors
/// The map does not read or import, or the engine refuses the setup.
fn begun_game(
    begun: &Begun,
    (install, rules): (&Path, &GameRuleset),
    step: &mut dyn FnMut(),
) -> Result<crate::begin::Game, String> {
    let map = crate::begin::read(&crate::begin::map_path(install, &begun.file))?;
    step();
    let setup = crate::begin::setup(&map, begun, rules)?;
    step();
    crate::begin::validated(&setup, rules)?;
    step();
    Ok(crate::begin::game(setup, begun))
}

/// The game the save at `path` holds (E27 M4 T4b2; `crate::save`): the file read, then loaded by
/// the engine over `install`'s map under `rules` (`save::load`: its commands replayed), each a
/// step of the loading screen's bar (`step`).
///
/// # Errors
/// The file does not read as a save, or the load refuses it.
fn loaded_game(
    path: &Path,
    (install, rules): (&Path, &GameRuleset),
    step: &mut dyn FnMut(),
) -> Result<crate::save::Loaded, String> {
    let save = crate::save::read_file(path)?;
    step();
    let loaded = crate::save::load(&save, install, rules)?;
    step();
    Ok(loaded)
}

/// Why the window refuses the standard input's word `line` on `screen` (E27 M3 T7, the review's
/// C6), none where the shell's words take it: `begin` makes no game's setup (the scenario
/// screen's Begin does, its choices with it), so the screen stays.
fn refused_word(screen: Screen, line: &str) -> Option<&'static str> {
    matches!(text::ending(screen, line), Some(Ended::Scenario(true)))
        .then_some("begin refused: a game begins from the scenario screen's Begin")
}

/// The scenario screen's background drawn as the original draws it (`random_between(1, 100) <
/// 51`: 1), from the system's randomness (none: 0).
fn background_draw() -> u8 {
    getrandom::u32().map_or(0, |v| u8::from(v % 100 + 1 < 51))
}

/// What the window gave in one wait, in order (T7b): its events, each keyboard key down with its
/// Shift, and the characters its text input typed between them.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Given {
    /// An event and, for a keyboard key down, its Shift.
    Event(Event, Option<KeyScan>),
    /// A character typed.
    Typed(char),
}

/// Whether a window's event stops a start's clip: a key down but F4 (`h3_view`'s rule,
/// [`clips::key_skips`], the replay's too), a left press, a right press (the adapter's
/// [`Key::RightButton`]).
fn skips_start(event: &Event) -> bool {
    match event {
        Event::Key(key) => clips::key_skips(key.name()),
        Event::Click { down, .. } => *down,
        _ => false,
    }
}

/// A wait's `events` with their `scans` and the characters `typed` among them (each with the
/// number of events before it, `h3_sdl::Window::typed`), in order.
fn merged(events: Vec<Event>, scans: &[Option<KeyScan>], typed: &[(usize, char)]) -> Vec<Given> {
    let mut given = Vec::with_capacity(events.len() + typed.len());
    let mut typed = typed.iter().copied().peekable();
    for (i, (event, &scan)) in events.into_iter().zip(scans).enumerate() {
        while let Some((_, c)) = typed.next_if(|&(at, _)| at <= i) {
            given.push(Given::Typed(c));
        }
        given.push(Given::Event(event, scan));
    }
    given.extend(typed.map(|(_, c)| Given::Typed(c)));
    given
}

/// A window's event to what takes the inputs on the screen: the left button's press and release,
/// the right button's (wherever the mouse is: at `pointer`), a key's down (its command with Shift
/// as held, T7b; no scan code: the window runs the client's profile, whose edits type the text
/// input, M2's review) and up through the key table.
fn on_front(
    front: &mut Front,
    keys: &Keys<Command>,
    (event, scan): (&Event, Option<KeyScan>),
    pointer: [i32; 2],
) -> menu::Did {
    match event {
        Event::Click {
            down: true,
            at: (x, y),
        } => front.press([*x, *y]),
        Event::Click { down: false, .. } => front.release(),
        Event::Key(Key::RightButton) => front.right(pointer),
        Event::KeyUp(Key::RightButton) => front.right_release(),
        Event::Key(key) => {
            let shift = scan.is_some_and(|s| s.shift);
            let command = shell_keys::command_for(keys, &front.contexts(), shift, key.name());
            front.key(None, command, key.name())
        }
        Event::KeyUp(key) => front.key_up(key.name()),
        _ => menu::Did::default(),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::{
        Given, Mode, Options, PARK, loaded_game, merged, refused_word, scores_text, skips_start,
    };
    use h3_sdl::{Event, Key, KeyScan};
    use h3_ui::keys::Keys;
    use h3_view::shell::keys::Command;

    #[test]
    fn the_shell_s_keys_are_the_adapter_s() {
        // E27 M2 T7b: the edits' keys and the ring among them (Tab, BackSpace, Delete, Shift+Tab).
        let known: Vec<&str> = Key::ALL.iter().map(|k| k.name()).collect();
        assert_eq!(Keys::<Command>::original().check(&known), Ok(()));
        // M2's review of E27: the window's table as the battle window's, the defaults without a
        // file here; a file named that is not there refused.
        let (keys, from) = crate::window::load_keys::<Command>(None).unwrap();
        assert_eq!((keys, from.as_str()), (Keys::original(), "the defaults"));
        assert!(crate::window::load_keys::<Command>(Some(Path::new("no/such/keys.toml"))).is_err());
    }

    /// E27 M3 T7 (the review's C6): the window's typed `begin` makes no setup, so it is refused on
    /// the scenario screen; its other words, and `begin` where no screen takes it, are the
    /// shell's.
    #[test]
    fn a_typed_begin_is_refused_on_the_scenario_screen() {
        use h3_view::shell::{Game, Kind, Screen};
        let scenario = Screen::Scenario(Game::New, Kind::Single);
        assert!(refused_word(scenario, " begin ").is_some_and(|r| r.contains("Begin")));
        assert_eq!(refused_word(scenario, "back"), None);
        assert_eq!(refused_word(Screen::MainMenu, "begin"), None);
    }

    #[test]
    fn every_key_but_f4_and_a_press_stop_the_start_as_in_the_replay() {
        // The review of M2 (fix 1): F4 skipped the start in the window, not in the original or
        // the replay.
        for &key in Key::ALL.iter() {
            assert_eq!(skips_start(&Event::Key(key)), key != Key::F4, "{key:?}");
        }
        assert!(skips_start(&Event::Key(Key::RightButton)));
        // E27 M3 T1a (M2's leftover): a key `h3_sdl::Key` has no variant for (a digit, B,
        // Insert) reaches the window as `Key::Other` (`Window::events`), which skips too.
        assert!(skips_start(&Event::Key(Key::Other)));
        assert!(skips_start(&Event::Click {
            down: true,
            at: (1, 2)
        }));
        for event in [
            Event::Click {
                down: false,
                at: (1, 2),
            },
            Event::KeyUp(Key::Escape),
            Event::Pointer(Some((1, 2))),
            Event::Redraw,
        ] {
            assert!(!skips_start(&event), "{event:?}");
        }
    }

    #[test]
    fn the_typed_characters_come_between_the_events() {
        let scan = Some(KeyScan { shift: false });
        let events = vec![Event::Key(Key::A), Event::KeyUp(Key::A), Event::Redraw];
        let scans = [scan, None, None];
        assert_eq!(
            merged(events, &scans, &[(0, 'x'), (1, 'a'), (3, 'b'), (3, 'c')]),
            [
                Given::Typed('x'),
                Given::Event(Event::Key(Key::A), scan),
                Given::Typed('a'),
                Given::Event(Event::KeyUp(Key::A), None),
                Given::Event(Event::Redraw, None),
                Given::Typed('b'),
                Given::Typed('c'),
            ]
        );
        assert_eq!(merged(Vec::new(), &[], &[(0, 'é')]), [Given::Typed('é')]);
    }

    #[test]
    fn the_scores_file_is_read_when_there_is_one() {
        // E27 M4 T1b: none there yet is a first run's (the defaults); a file's text as it is;
        // one that cannot be read, an error naming it.
        let dir = std::env::temp_dir().join(format!("h3-view-scores-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("scores.toml");
        let _ = fs::remove_file(&path);
        assert_eq!(scores_text(&path), Ok(None));
        fs::write(&path, "version = 1\n").unwrap();
        assert_eq!(scores_text(&path), Ok(Some(String::from("version = 1\n"))));
        let refused = scores_text(&dir).unwrap_err();
        assert!(refused.starts_with(&dir.display().to_string()), "{refused}");
        fs::remove_dir_all(&dir).unwrap();
    }

    fn parse(args: &[&str]) -> Result<Options, String> {
        let args: Vec<String> = args.iter().map(|a| String::from(*a)).collect();
        Options::parse_with(&args, Some(PathBuf::from("env")))
    }

    #[test]
    fn the_options_are_a_mode_the_intro_and_the_installation() {
        let none = parse(&[]).unwrap();
        assert_eq!((none.mode, none.intro), (Mode::Words(None), true));
        assert_eq!(none.install, PathBuf::from("env"));
        let both = parse(&["--no-intro", "--script", "menus.txt"]).unwrap();
        assert_eq!(both.mode, Mode::Words(Some(PathBuf::from("menus.txt"))));
        assert!(!both.intro);
        let replay = parse(&["--steps", "s", "--frames", "f", "--install", "i"]).unwrap();
        assert_eq!(
            replay.mode,
            Mode::Replay {
                steps: "s".into(),
                frames: "f".into(),
                pointer: PARK,
                input_step: 100,
                backgrounds: Vec::new(),
                maps: None,
            }
        );
        assert_eq!(replay.install, PathBuf::from("i"));
        // E27 M3 T1a: the scenario screen's background draws, a replay's alone.
        let drawn = parse(&["--steps", "s", "--frames", "f", "--backgrounds", "0,0,1"]);
        assert!(matches!(
            drawn.unwrap().mode,
            Mode::Replay { backgrounds, .. } if backgrounds == [0, 0, 1]
        ));
        assert!(parse(&["--steps", "s", "--frames", "f", "--backgrounds", "2"]).is_err());
        assert!(parse(&["--window", "--backgrounds", "1"]).is_err());
        // E27 M3 T1b1: the maps a recipe gives, a replay's alone.
        let named = parse(&["--steps", "s", "--frames", "f", "--maps", "m.txt"]);
        assert!(matches!(
            named.unwrap().mode,
            Mode::Replay { maps: Some(m), .. } if m == Path::new("m.txt")
        ));
        assert!(parse(&["--window", "--maps", "m.txt"]).is_err());
        let placed = parse(&[
            "--steps",
            "s",
            "--frames",
            "f",
            "--pointer",
            "1,2",
            "--input-step",
            "50",
        ]);
        assert!(matches!(
            placed.unwrap().mode,
            Mode::Replay {
                pointer: [1, 2],
                input_step: 50,
                ..
            }
        ));
        assert_eq!(
            parse(&["--window", "--scale", "2"]).unwrap().mode,
            Mode::Window(Some(2))
        );
        // M2's review of E27: the window's key file.
        assert_eq!(none.keys, None);
        assert_eq!(
            parse(&["--window", "--keys", "k.toml"]).unwrap().keys,
            Some(PathBuf::from("k.toml"))
        );
        // E27 M4 T1b: the window's scores file.
        assert_eq!(none.scores, None);
        assert_eq!(
            parse(&["--window", "--scores", "s.toml"]).unwrap().scores,
            Some(PathBuf::from("s.toml"))
        );
        assert!(none.bink_video);
        assert!(!parse(&["--video-quality", "low"]).unwrap().bink_video);
        assert!(parse(&["--video-quality", "high"]).unwrap().bink_video);
        // E27 M2 T5b1: the effects setting, 0 to 9.
        assert_eq!(none.effects, None);
        assert_eq!(parse(&["--effects", "5"]).unwrap().effects, Some(5));
        assert!(parse(&["--effects", "10"]).is_err());
        assert!(parse(&["--effects", "x"]).is_err());
        let refused = [
            &["--script"][..],
            &["--intro"],
            &["--steps", "s"],
            &["--frames", "f"],
            &["--window", "--script", "x"],
            &["--steps", "s", "--frames", "f", "--window"],
            &["--pointer", "1,2"],
            &["--steps", "s", "--frames", "f", "--pointer", "1"],
            &["--steps", "s", "--frames", "f", "--pointer", "x,2"],
            &["--input-step", "x"],
            &["--scale", "2"],
            &["--keys", "k.toml"],
            &["--window", "--keys"],
            &["--scores", "s.toml"],
            &["--window", "--scores"],
            &["--video-quality", "medium"],
        ];
        for args in refused {
            assert!(parse(args).is_err(), "{args:?}");
        }
    }

    #[test]
    fn a_load_reads_the_save_and_loads_its_game_stepping_the_bar() {
        // E27 M4 T4b2: Load's end, over a save made on a committed map (CI runs it).
        let rules = crate::maps::rules().unwrap();
        let dir = std::env::temp_dir().join(format!("h3-view-load-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let maps = dir.join(crate::maps::MAPS);
        fs::create_dir_all(&maps).unwrap();
        let duel = "e08-duel-36.h3m";
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/maps");
        fs::copy(fixture.join(duel), maps.join(duel)).unwrap();
        let save = crate::saves::made(&dir, &rules, duel, (1, 2), "two").unwrap();
        let path = dir.join("two.GM1");
        crate::save::write_file(&path, &save).unwrap();
        let mut steps = 0;
        let loaded = loaded_game(&path, (&dir, &rules), &mut || steps += 1).unwrap();
        assert_eq!(
            (loaded.game.file.as_str(), loaded.state.day(), steps),
            (duel, 2, 2)
        );
        // Not a save, or a save whose map is gone: refused, the bar stepped as far as it went.
        fs::write(dir.join("bad.GM1"), "not a save").unwrap();
        let mut steps = 0;
        assert!(loaded_game(&dir.join("bad.GM1"), (&dir, &rules), &mut || steps += 1).is_err());
        assert_eq!(steps, 0);
        fs::remove_file(maps.join(duel)).unwrap();
        let mut steps = 0;
        assert!(loaded_game(&path, (&dir, &rules), &mut || steps += 1).is_err());
        assert_eq!(steps, 1);
        let _ = fs::remove_dir_all(&dir);
    }
}
