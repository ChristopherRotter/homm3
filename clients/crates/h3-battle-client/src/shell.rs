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
//!   (T6a: `h3_view::shell::menu`, the key table `h3_view::shell::keys`), New Game's and Load
//!   Game's menus theirs (T6b1), each press playing its sample (`button.wav`); the music
//!   (`MAINMENU.MP3`, looping) is asked for as each menu opens, as `main_menu_run` and
//!   `game_type_menu_run` ask for it (the track playing goes on).
//!
//! `--no-intro` starts at the main menu (the setting `Show Intro` off); else the start's clips
//! play first (T5a: `h3_view::shell::start`, read from the video archives, `introrim.pcx` from the
//! LODs), from Bink or with `--video-quality low` from their Smacker copies (the setting Bink
//! Video 0, CLT-Q96). The replay draws them as the DLLs copy them (`Bugs::Original`), the window
//! whole, on the window's own clock.
//!
//! **The sound** (E27 M2 T5b1): `--effects <0..9>` is the effects setting (the window's default
//! 5, the game's; the replay's 0, the harness's silent runs). Above 0, with a sound device (the
//! window's) or none needed (the replay), the clips play with their sound: a Smacker clip's frames
//! then follow it (`h3_view::shell::start::SmackSound`), and the window plays each clip's track
//! from its start (`HostSound::play_clip`, at `h3_view::clip_sound::volume`) until a skip stops
//! it, or whole once its clip ran to its end (CLT-Q125, decision 34: the original cuts the last
//! frame's sound at the close); the replay lists each start, skip and close in `replay.txt`
//! (`sound <file> t<ms>`, `quiet <file> t<ms>`, `closed <file> t<ms>`), and (T6a) each press's
//! sample as the harness logs the original's (`sample button.wav t<ms> ch<k> <length>ms x1`:
//! `h3_view::shell::samples`).
//!
//! The menus' boxes (T6a, T6b1): Quit's question and the buttons' help, their texts the user's
//! tables' through the key catalogue (`shell.main-menu.*`, `shell.game-type.help.*`), laid out at
//! the start. The stubs (T6b2): New Game's Campaign and Tutorial and the main menu's High Scores
//! and Credits show a notice in the client's own words (`shell.stub.*`, its own pack
//! `clients/text/en.json`) over the menu they were chosen from, in the window; its OK goes back to
//! that menu. The replay stops at them (the original shows its own screens there). The replay
//! draws a game type menu's rows its redraw does not show as last shown (CLT-Q118:
//! `h3_view::shell::menu::Stale`, the original's profile).

use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::mpsc;
use std::thread;

use h3_assets::wav::Pcm;
use h3_render::Bugs;
use h3_render::blit::draw_image;
use h3_render::screen::Screen as Pixels;
use h3_sdl::{Config, CursorImage, Event, Key, Window};
use h3_ui::cursor::Cursor;
use h3_ui::keys::Keys;
use h3_ui::text::Args;
use h3_view::batch::Batch;
use h3_view::compose::FORMAT;
use h3_view::message::{self, Style};
use h3_view::shell::keys::{Command, GAME_TYPE, MAIN_MENU};
use h3_view::shell::look::{self, ShellLayout};
use h3_view::shell::menu::{self, Boxes, Kind, Menu, Menus};
use h3_view::shell::replay::{self, INPUT_STEP, Replay, Stop, Take};
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
}

impl Options {
    /// Parses `[--script <file> | --steps <file> --frames <dir> [--pointer <x>,<y>]
    /// [--input-step <ms>] | --window [--scale <n>]] [--no-intro] [--video-quality high|low]
    /// [--effects <0..9>] [--install <dir>]`; the installation defaults to `$H3_INSTALL`, then the repository's
    /// `original-data/install`.
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
        let mut effects = None;
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
                "--scale" => scale = Some(number(&value()?)?),
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
        lone(
            !replaying && (pointer.is_some() || input_step.is_some()),
            "--pointer and --input-step",
            "--steps",
        )?;
        lone(!window && scale.is_some(), "--scale", "--window")?;
        let mode = match (steps, frames) {
            (Some(steps), Some(frames)) => Mode::Replay {
                steps,
                frames,
                pointer: pointer.unwrap_or(PARK),
                input_step: input_step.unwrap_or(INPUT_STEP),
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
        })
    }
}

fn number(s: &str) -> Result<u32, String> {
    s.parse()
        .map_err(|_| format!("shell: `{s}` is not a number"))
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
        Mode::Replay { .. } => Drawer::load(&options.install, Bugs::Original)
            .and_then(|drawer| replay_steps(options, &drawer)),
        Mode::Window(scale) => Drawer::load(&options.install, Bugs::Fixed)
            .and_then(|drawer| in_window(options, *scale, &drawer)),
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
    cursor: Cursor,
    hardware: CursorImage,
    bugs: Bugs,
    /// `introrim.pcx` drawn on the screen (none when the LODs lack it: `H3Intro` then plays on
    /// what the back buffer held).
    rim: Option<Pixels>,
    videos: Videos,
    /// The menus as they open: their dialogs and their boxes laid out (T6a, T6b1).
    menus: Menus,
    /// The press sample, when the sound archives hold it.
    button: Option<Pcm>,
}

impl Drawer {
    /// The layout (`clients/layouts/shell.toml`) and its assets from `install`'s archives, read
    /// by their tables in one batch (CLT-Q2).
    fn load(install: &Path, bugs: Bugs) -> Result<Self, String> {
        let path = assets::clients_dir().join("layouts").join("shell.toml");
        let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let layout = look::parse_layout(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        let lods = Lods::open(install)?;
        let mut names = look::names(&layout);
        names.push(String::from(RIM));
        let mut batch = Batch::with_bugs(bugs);
        for (name, bytes) in names.iter().zip(lods.batch(&names)?) {
            batch.add(name, &bytes).map_err(|e| e.to_string())?;
        }
        let menus = menus(&layout, &batch, &lods, install, bugs)?;
        let mut lines = Vec::new();
        let button = crate::audio::clips(install, &[BUTTON_ENTRY.to_owned()], &mut lines)?
            .into_values()
            .next();
        let rim = batch.picture(RIM).and_then(|pcx| {
            let mut pixels = Pixels::new(WIDTH, HEIGHT, FORMAT)?;
            draw_image(&mut pixels, pcx, 0, 0);
            Some(pixels)
        });
        let (def, frame) = (&layout.cursor.def, layout.cursor.frame);
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
        Ok(Self {
            layout,
            batch,
            cursor,
            hardware,
            bugs,
            rim,
            videos: Videos::open(install),
            menus,
            button,
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

    /// The frame of `screen` (a menu as `menu` is), the cursor at `pointer` if any and shown: a
    /// built screen's background copied whole, its dialog over it, the rows a redraw left as last
    /// shown (the original's profile), the box over them; another's black.
    fn frame(
        &self,
        screen: Screen,
        pointer: Option<[i32; 2]>,
        menu: Option<&Menu>,
    ) -> Result<Pixels, String> {
        let mut pixels = Pixels::new(WIDTH, HEIGHT, FORMAT).ok_or("no 800 × 600 screen")?;
        let mut cursor = true;
        if let Some(shown) = look::look(&self.layout, screen, menu) {
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
            cursor = shown.cursor;
        }
        if let Some([x, y]) = pointer.filter(|_| cursor) {
            // The original's bug: the mouse's x rounded down to even first (0x50CE04, E17
            // design §9.9: fixed, reproduced for the comparisons only, as the battle's).
            let x = match self.bugs {
                Bugs::Original => x & !1,
                Bugs::Fixed => x,
            };
            self.cursor.draw(&mut pixels, x, y);
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

/// The menus as they open (T6a, T6b1): the layout's dialogs, Quit's question and the buttons'
/// help laid out in the batch's fonts, their texts the user's tables' (`shell.main-menu.quit`,
/// `shell.main-menu.help.<button>`, `shell.game-type.help.<button>`: Load Game's the same as New
/// Game's), and the stubs' notices (T6b2: `shell.stub.<screen>`, the client's own pack's, style
/// 1), framed in the viewing player's colours.
fn menus(
    layout: &ShellLayout,
    batch: &Batch,
    lods: &Lods,
    install: &Path,
    bugs: Bugs,
) -> Result<Menus, String> {
    let tables = assets::load(&assets::default_dir(), assets::ALL)?;
    let exe = crate::text::executable(install)?;
    let catalogue = crate::log::catalogue(&tables, lods, &exe)?;
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
            let key = format!("shell.stub.{}", text::name(screen));
            (screen, laid(&key, Style::Ok))
        })
        .collect();
    Ok(Menus {
        main,
        new_game: game_type(Game::New),
        load_game: game_type(Game::Load),
        notices,
    })
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
    let start = Start::with_sound(headers, sounds, bink_video, 0, &mut ops);
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
        Stop::Unknown(name) => format!("stop {k} unknown {name}"),
        Stop::NotBuilt(screen) => {
            format!(
                "stop {k} not-built {} ({})",
                text::name(*screen),
                screen.task()
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
    let mut r = Replay::new(shell, *pointer, *input_step, Some(start), &drawer.menus);
    if sound && let Some(ms) = drawer.button_ms() {
        r = r.with_sound(ms);
    }
    for (k, step) in parsed.iter().enumerate() {
        let mut take = |r: &Replay, t: Take<'_>| -> Result<(), String> {
            catch_up(&mut picture, r.ops(), &mut applied)?;
            let pixels = if r.screen() == Screen::Start {
                picture.screen().clone()
            } else {
                drawer.frame(r.screen(), Some(r.pointer()), r.menu())?
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
fn in_window(options: &Options, scale: Option<u32>, drawer: &Drawer) -> Result<(), String> {
    let mut shell = Shell::new(options.intro);
    let bytes = if options.intro {
        drawer.clips(options.bink_video)
    } else {
        [None, None, None]
    };
    let clips = parse_clips(&bytes);
    // The sound (E27 M2 T5b1): the effects above 0 and a device, else the clips silent (the
    // game opens no track without a driver, and its Smacker clips keep their clock); the menus'
    // samples and music with it (T6a).
    let effects = options.effects.unwrap_or(5);
    let mut host = if effects > 0 {
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
    // Each clip's track decoded beside the window (H3Intro's takes seconds), taken at its start.
    let mut samples = [0, 1, 2].map(|k| {
        let format = clips[k].as_ref()?.format();
        let bytes = bytes[k].clone()?;
        host.as_ref()?;
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
    let music_volume = h3_view::options::sound_volume(
        h3_view::sound::MUSIC_VOLUME,
        h3_view::options::Settings::FIRST_RUN.music,
    );
    let mut presses: Vec<usize> = Vec::new();
    let keys = Keys::<Command>::original();
    // The menu on the screen, opened as its screen shows (`on`: the screen it was opened for).
    let mut menu: Option<Menu> = None;
    let ((mut start, mut ops), mut picture) = start_of(
        &clips,
        drawer.rim.as_ref(),
        options.bink_video,
        drawer.bugs,
        host.is_some(),
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
    loop {
        let playing = shell.screen() == Screen::Start;
        for event in win.wait(if playing { 5 } else { 50 }) {
            let mut did = menu::Did::default();
            match event {
                Event::Quit => return Ok(()),
                Event::Pointer(at) => {
                    pointer = at.map(|(x, y)| [x, y]);
                    dirty |= drawn;
                    if let (Some(menu), Some(at)) = (menu.as_mut(), pointer) {
                        did = menu.moved(at);
                    }
                }
                Event::Redraw => dirty = true,
                Event::Key(_) | Event::Click { down: true, .. } if playing => {
                    start.skip(now(&win), &mut ops);
                }
                event => {
                    if let Some(menu) = menu.as_mut() {
                        did = on_menu(menu, &keys, &event, pointer.unwrap_or(PARK));
                    }
                }
            }
            dirty |= did.changed;
            if did.sample
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
                let _ = shell.end(ended);
                println!("{}", text::describe(shell.screen()));
                if shell.screen() == Screen::End {
                    return Ok(());
                }
                on = None;
                menu = None;
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
            println!("{}", text::answer(&mut shell, &line));
            if shell.screen() == Screen::End {
                return Ok(());
            }
            if shell.screen() != before {
                on = None;
                menu = None;
            }
            dirty = true;
        }
        // The help popup hides the cursor (`popup_run_held`): the hardware one too.
        let hidden = menu.as_ref().is_some_and(|m| !m.cursor());
        if !drawn && hidden != cursor_hidden {
            let image = (!hidden).then(|| drawer.hardware.clone());
            win.set_cursor(image)?;
            cursor_hidden = hidden;
        }
        // A screen shown anew (the start's end, a menu's, a word's): its menu opened at the
        // pointer, before it is drawn, its music asked for (the track playing goes on:
        // E24-design §3.11).
        if on != Some(shell.screen()) {
            on = Some(shell.screen());
            menu = drawer.menus.opened(shell.screen(), pointer.unwrap_or(PARK));
            if let (Some(host), true) = (host.as_mut(), menu.is_some()) {
                let _ = host.request(MAIN_MUSIC, false);
            }
            dirty = true;
        }
        if dirty {
            let frame = if shell.screen() == Screen::Start {
                picture.screen().clone()
            } else {
                drawer.frame(shell.screen(), pointer.filter(|_| drawn), menu.as_ref())?
            };
            win.present(&frame.xrgb())?;
            dirty = false;
        }
    }
}

/// A window's event to the menu on the screen: the left button's press and release, the right
/// button's (its help), a hotkey's key down through the shell's key table and its key up.
fn on_menu(menu: &mut Menu, keys: &Keys<Command>, event: &Event, pointer: [i32; 2]) -> menu::Did {
    match event {
        Event::Click {
            down: true,
            at: (x, y),
        } => menu.press([*x, *y]),
        Event::Click { down: false, .. } => menu.release(),
        Event::Key(Key::RightButton) => menu.right(pointer),
        Event::KeyUp(Key::RightButton) => menu.right_release(),
        Event::Key(key) => keys
            .command(menu.context(), false, key.name())
            .map_or_else(menu::Did::default, |command| menu.key(command, key.name())),
        Event::KeyUp(key) => menu.key_up(key.name()),
        _ => menu::Did::default(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{Mode, Options, PARK};

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
                input_step: 100
            }
        );
        assert_eq!(replay.install, PathBuf::from("i"));
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
            &["--video-quality", "medium"],
        ];
        for args in refused {
            assert!(parse(args).is_err(), "{args:?}");
        }
    }
}
