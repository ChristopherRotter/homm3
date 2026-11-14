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
//! - **In a window** (T4: `--window`): the built screens drawn (`h3_view::shell::look`, the main
//!   menu at rest), the others black until their task, the cursor the shell's arrow; the words
//!   still read from standard input. Closing the window ends it.
//!
//! `--no-intro` starts at the main menu (the setting `Show Intro` off).

use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::mpsc;
use std::thread;

use h3_render::Bugs;
use h3_render::blit::draw_image;
use h3_render::screen::Screen as Pixels;
use h3_sdl::{Config, CursorImage, Event, Window};
use h3_ui::cursor::Cursor;
use h3_view::batch::Batch;
use h3_view::compose::FORMAT;
use h3_view::shell::look::{self, ShellLayout};
use h3_view::shell::replay::{self, INPUT_STEP, Replay, Stop, Take};
use h3_view::shell::{Screen, Shell, text};

use crate::assets;
use crate::lods::Lods;

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
    install: PathBuf,
}

impl Options {
    /// Parses `[--script <file> | --steps <file> --frames <dir> [--pointer <x>,<y>]
    /// [--input-step <ms>] | --window [--scale <n>]] [--no-intro] [--install <dir>]`; the
    /// installation defaults to `$H3_INSTALL`, then the repository's `original-data/install`.
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
        let (mut intro, mut window) = (true, false);
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
            .and_then(|drawer| in_window(options.intro, *scale, &drawer)),
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

/// The shell's layout, its assets and its cursor: what draws a frame.
struct Drawer {
    layout: ShellLayout,
    batch: Batch,
    cursor: Cursor,
    hardware: CursorImage,
    bugs: Bugs,
}

impl Drawer {
    /// The layout (`clients/layouts/shell.toml`) and its assets from `install`'s archives, read
    /// by their tables in one batch (CLT-Q2).
    fn load(install: &Path, bugs: Bugs) -> Result<Self, String> {
        let path = assets::clients_dir().join("layouts").join("shell.toml");
        let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let layout = look::parse_layout(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        let lods = Lods::open(install)?;
        let names = look::names(&layout);
        let mut batch = Batch::with_bugs(bugs);
        for (name, bytes) in names.iter().zip(lods.batch(&names)?) {
            batch.add(name, &bytes).map_err(|e| e.to_string())?;
        }
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
        })
    }

    /// The frame of `screen`, the cursor at `pointer` if any: a built screen's background copied
    /// whole, its dialog over it; another's black.
    fn frame(&self, screen: Screen, pointer: Option<[i32; 2]>) -> Result<Pixels, String> {
        let mut pixels = Pixels::new(WIDTH, HEIGHT, FORMAT).ok_or("no 800 × 600 screen")?;
        if let Some(shown) = look::look(&self.layout, screen) {
            let background = self
                .batch
                .picture(&shown.background)
                .ok_or_else(|| format!("{}: not loaded", shown.background))?;
            draw_image(&mut pixels, background, 0, 0);
            for draw in &shown.draws {
                h3_ui::paint::draw(&mut pixels, draw, &self.batch, self.bugs)?;
            }
        }
        if let Some([x, y]) = pointer {
            self.cursor.draw(&mut pixels, x, y);
        }
        Ok(pixels)
    }
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
        Stop::AtRest(screen) => {
            format!(
                "stop {k} at-rest {} ({})",
                text::name(*screen),
                screen.task()
            )
        }
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
    let mut r = Replay::new(Shell::new(options.intro), *pointer, *input_step);
    for (k, step) in parsed.iter().enumerate() {
        let mut take = |r: &Replay, t: Take<'_>| -> Result<(), String> {
            let pixels = drawer.frame(r.screen(), Some(r.pointer()))?;
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
    let listing: String = listing.iter().map(|l| format!("{l}\n")).collect();
    print!("{listing}");
    write(&frames.join("replay.txt"), listing.as_bytes())
}

/// The shell in a window (T4): the built screens drawn, the words from standard input.
fn in_window(intro: bool, scale: Option<u32>, drawer: &Drawer) -> Result<(), String> {
    let mut shell = Shell::new(intro);
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
    let (mut pointer, mut dirty) = (None, true);
    loop {
        for event in win.wait(50) {
            match event {
                Event::Quit => return Ok(()),
                Event::Pointer(at) => {
                    pointer = at.map(|(x, y)| [x, y]);
                    dirty |= drawn;
                }
                Event::Redraw => dirty = true,
                _ => {}
            }
        }
        while let Ok(line) = words.try_recv() {
            println!("{}", text::answer(&mut shell, &line));
            if shell.screen() == Screen::End {
                return Ok(());
            }
            dirty = true;
        }
        if dirty {
            let frame = drawer.frame(shell.screen(), pointer.filter(|_| drawn))?;
            win.present(&frame.xrgb())?;
            dirty = false;
        }
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
        ];
        for args in refused {
            assert!(parse(args).is_err(), "{args:?}");
        }
    }
}
