//! `h3-map-view window` (E24 M2 T4a, `h3-view window`'s way: E17 design §8.1, ADR-0024): a
//! recorded sequence or a situation in a window at the original's frame times. The map is set up
//! as `sequences` sets it up (`crate::paint`): a sequence's inputs replayed at their milliseconds
//! (its scrolls, edge cursors and walks), a situation standing still; past a sequence's inputs the
//! map stands still on. The timeline runs forward as the map's clock needs (`h3_map_view::replay`,
//! `h3_map_view::clock`); the frame of each change is composed and widened ahead of its time and
//! presented when it falls due, a change overtaken by a later one passed over. The cursor is the
//! replay's, drawn in the frame at its point (the system's is hidden over the window); the mouse
//! and the map's keys take no effect yet (T4b, the map's live input).
//!
//! Every key goes through the table of commands (`h3_map_view::keys`; *provisional*): the defaults,
//! or a user's file over them (`--keys`, else `keys.toml` in the working directory when there is
//! one); each key it names must be one the adapter knows (`h3_sdl::Key`), Control held with
//! `Ctrl+`. `--check <s>` and `--measure <s>` run unattended for that many seconds (the keys but
//! quit do nothing) and report when each frame showed against its time; `--check` also reads
//! every frame back before presenting it and compares it with the composed frame at the window's
//! scale (the black around it included). An adapter: the window is `h3-sdl`'s.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use h3_map_view::clock::Clock;
use h3_map_view::keys::{Command, Keys};
use h3_map_view::replay::{Change, Replay};
use h3_map_view::sequence::Sequences;
use h3_map_view::timeline::{Rules, Unplayable};
use h3_render::Bugs;
use h3_render::screen::Screen;
use h3_sdl::{Config, Event, Key, ReadBack, Window};

use crate::files;
use crate::paint::{self, Assets, Painter, Setup};

/// How far past a change the window looks for the next one before it waits for input only.
const LOOK_AHEAD_MS: u32 = 600_000;

/// The longest wait for input, so that the clock is read now and then.
const WAIT_MS: u32 = 250;

/// The user's file of keys looked for in the working directory.
const KEYS_FILE: &str = "keys.toml";

/// The command's options.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    id: String,
    keys: Option<PathBuf>,
    scale: Option<u32>,
    vsync: bool,
    fullscreen: bool,
    bugs: Bugs,
    install: PathBuf,
    map: PathBuf,
    /// Unattended for this many seconds; whether each frame is read back.
    run: Option<(u32, bool)>,
}

impl Options {
    /// Parses `<sequence | situation> [--keys <file>] [--scale <n>] [--vsync] [--fullscreen]
    /// [--bugs fixed|original] [--install <dir>] [--map <dir>] [--check <s> | --measure <s>]`.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let (mut id, mut keys, mut scale, mut install, mut map) = (None, None, None, None, None);
        let (mut vsync, mut fullscreen, mut bugs, mut run) = (false, false, Bugs::Fixed, None);
        let mut rest = args.iter();
        while let Some(arg) = rest.next() {
            let mut value = || {
                rest.next()
                    .cloned()
                    .ok_or_else(|| format!("window: `{arg}` needs a value"))
            };
            let number = |v: String| {
                v.parse::<u32>()
                    .map_err(|_| format!("window: bad number `{v}` for `{arg}`"))
            };
            match arg.as_str() {
                "--keys" => keys = Some(PathBuf::from(value()?)),
                "--scale" => scale = Some(number(value()?)?.clamp(1, 16)),
                "--vsync" => vsync = true,
                "--fullscreen" => fullscreen = true,
                "--bugs" => {
                    bugs = match value()?.as_str() {
                        "fixed" => Bugs::Fixed,
                        "original" => Bugs::Original,
                        _ => return Err("window: --bugs takes fixed or original".into()),
                    };
                }
                "--install" => install = Some(PathBuf::from(value()?)),
                "--map" => map = Some(PathBuf::from(value()?)),
                "--check" => run = Some((number(value()?)?, true)),
                "--measure" => run = Some((number(value()?)?, false)),
                flag if flag.starts_with("--") => {
                    return Err(format!("window: unknown option `{flag}`"));
                }
                name if id.is_none() => id = Some(name.to_owned()),
                extra => return Err(format!("window: unexpected `{extra}`")),
            }
        }
        Ok(Self {
            id: id.ok_or("window: needs a sequence or a situation")?,
            keys,
            scale,
            vsync,
            fullscreen,
            bugs,
            install: install.unwrap_or_else(files::default_install),
            map: map.unwrap_or_else(files::default_map),
            run,
        })
    }
}

/// Runs the command.
pub fn run(options: &Options) -> ExitCode {
    match window(options) {
        Ok((lines, ok)) => {
            for line in lines {
                println!("{line}");
            }
            if ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(message) => {
            eprintln!("window: {message}");
            ExitCode::FAILURE
        }
    }
}

/// The table of keys: the user's file (`--keys`, else `keys.toml` here if there is one) over the
/// defaults, every key it binds one the adapter knows; and where it came from.
fn load_keys(path: Option<&Path>) -> Result<(Keys, String), String> {
    let here = Path::new(KEYS_FILE);
    let path = path.or_else(|| here.exists().then_some(here));
    let (keys, from) = match path {
        Some(path) => {
            let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
            let keys = Keys::with_file(&text).map_err(|e| format!("{}: {e}", path.display()))?;
            (keys, path.display().to_string())
        }
        None => (Keys::original(), "the defaults".to_owned()),
    };
    check_keys(&keys).map_err(|e| format!("{from}: {e}"))?;
    Ok((keys, from))
}

/// Every key the table binds is one of the adapter's, and not Control alone (Ctrl+ holds it).
fn check_keys(keys: &Keys) -> Result<(), String> {
    for (command, chord) in keys.chords() {
        match Key::from_name(&chord.key) {
            Some(Key::LeftCtrl | Key::RightCtrl) => {
                return Err(format!(
                    "`{}`: {chord}: Control is held with a key (`Ctrl+`), not bound alone",
                    command.name()
                ));
            }
            Some(_) => {}
            None => {
                let names: Vec<&str> = Key::ALL
                    .iter()
                    .filter(|k| !matches!(k, Key::LeftCtrl | Key::RightCtrl))
                    .map(|k| k.name())
                    .collect();
                return Err(format!(
                    "`{}`: {chord}: no such key; the keys: {}",
                    command.name(),
                    names.join(", ")
                ));
            }
        }
    }
    Ok(())
}

/// The map `id` names set up: a recorded sequence of `<map>/sequences.toml`, else a situation of
/// `<map>/situations/`; and what it is.
fn set_up<'a>(assets: &'a Assets<'a>, options: &Options) -> Result<(Setup<'a>, String), String> {
    let path = options.map.join("sequences.toml");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let sequences = Sequences::parse(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    if let Some(sequence) = sequences.get(&options.id) {
        let setup = paint::for_sequence(assets, &options.map, &options.install, sequence, None)?;
        let what = format!(
            "the sequence {} ({} inputs to {} ms)",
            sequence.id,
            sequence.inputs.len(),
            sequence.ms
        );
        return Ok((setup, what));
    }
    let situation = options
        .map
        .join("situations")
        .join(format!("{}.toml", options.id));
    if !situation.exists() {
        return Err(format!(
            "`{}` is no sequence of {} and no situation of {}",
            options.id,
            path.display(),
            options.map.join("situations").display()
        ));
    }
    let setup = paint::for_situation(assets, &options.map, &options.install, &options.id)?;
    Ok((setup, format!("the situation {}", options.id)))
}

/// A frame composed ahead of its time: the change it shows, the frame and the frame widened.
struct Frame {
    change: Change,
    screen: Screen,
    xrgb: Vec<u32>,
}

impl Frame {
    fn of(painter: &mut Painter<'_>, change: Change) -> Result<Self, String> {
        let screen = painter.frame(&change.drawn, change.cursor)?;
        let xrgb = screen.xrgb();
        Ok(Self {
            change,
            screen,
            xrgb,
        })
    }

    /// Whether it shows what `change` shows.
    fn shows(&self, change: &Change) -> bool {
        self.change.drawn == change.drawn && self.change.cursor == change.cursor
    }
}

/// A replay's error as a message.
fn unplayable((t, why): (u32, Unplayable)) -> String {
    format!("at {t} ms: {why}")
}

/// What the run measured.
#[derive(Debug, Default)]
struct Report {
    /// Per frame shown at its time: how late, in microseconds.
    late_us: Vec<u64>,
    /// Frames passed over because a later one was due already.
    skipped: usize,
    /// Frames read back, and those that matched.
    read: usize,
    exact: usize,
    /// The first difference found.
    difference: Option<String>,
    /// The last read-back's output, the frame's place in it and its scale.
    geometry: Option<String>,
}

/// Opens the window and plays the map until a quit, a close, or the unattended run's end.
#[allow(clippy::too_many_lines)]
fn window(options: &Options) -> Result<(Vec<String>, bool), String> {
    let mut lines = Vec::new();
    let (keys, keys_from) = load_keys(options.keys.as_deref())?;
    let tables = files::load_tables(&options.map)?;
    let layout = files::load_layout(&options.map)?;
    let archive_bytes = files::read_archives(&options.install)?;
    let archives = files::parse_archives(&archive_bytes)?;
    let templates = files::templates(&archives)?;
    let assets = Assets {
        tables: &tables,
        layout: &layout,
        archives: &archives,
        templates: &templates,
        bugs: options.bugs,
    };
    let (setup, what) = set_up(&assets, options)?;
    let Setup {
        mut painter,
        start,
        actions,
        ..
    } = setup;
    let rules = Rules::new(&tables, &layout, options.bugs);
    let mut replay = Replay::new(rules, &start, actions).map_err(|why| unplayable((0, why)))?;
    let [width, height] = layout.frame;
    let mut win = Window::open(&Config {
        title: format!("h3-map-view — {}", options.id),
        width,
        height,
        scale: options.scale,
        vsync: options.vsync,
        fullscreen: options.fullscreen,
    })?;
    let mut clock = Clock::new(win.ticks_ns(), 0);
    let mut shown: Option<Frame> = None;
    let mut next = Some(Frame::of(&mut painter, replay.at(0).map_err(unplayable)?)?);
    // Either Control key held.
    let mut ctrl = [false, false];
    let mut report = Report::default();
    let started = win.ticks_ns();
    let stop_ns = options
        .run
        .map(|(seconds, _)| u64::from(seconds) * 1_000_000_000);
    let read = options.run.is_some_and(|(_, read)| read);
    'run: loop {
        let now = win.ticks_ns();
        if stop_ns.is_some_and(|s| now.saturating_sub(started) >= s) {
            break;
        }
        let timeout = next
            .as_ref()
            .and_then(|f| clock.until(now, f.change.t))
            .map_or(WAIT_MS, |ns| {
                u32::try_from(ns / 1_000_000)
                    .unwrap_or(WAIT_MS)
                    .min(WAIT_MS)
            });
        let mut redraw = false;
        for event in win.wait(timeout) {
            let now = win.ticks_ns();
            let key = match event {
                Event::Quit => break 'run,
                Event::Redraw => {
                    redraw = true;
                    continue;
                }
                Event::Key(k @ (Key::LeftCtrl | Key::RightCtrl))
                | Event::KeyUp(k @ (Key::LeftCtrl | Key::RightCtrl)) => {
                    ctrl[usize::from(k == Key::RightCtrl)] = matches!(event, Event::Key(_));
                    continue;
                }
                Event::Key(k) => k,
                // The mouse and the keys' releases: the map's live input, T4b.
                Event::KeyUp(_) | Event::Pointer(_) | Event::Click { .. } => continue,
            };
            let command = keys.command(ctrl.contains(&true), key.name());
            match command {
                Some(Command::Quit) => break 'run,
                // Unattended, the other keys do nothing.
                _ if options.run.is_some() => {}
                Some(Command::Pause) => clock.toggle(now),
                Some(Command::Frame) if clock.paused() => {
                    if let Some(f) = &next {
                        clock.seek(now, f.change.t);
                    }
                }
                Some(Command::Restart) => {
                    replay.restart();
                    clock.seek(now, 0);
                    shown = None;
                    next = Some(Frame::of(&mut painter, replay.at(0).map_err(unplayable)?)?);
                }
                Some(Command::Fullscreen) => {
                    let on = !win.fullscreen();
                    win.set_fullscreen(on);
                }
                Some(Command::Vsync) => {
                    let on = !win.vsync();
                    win.set_vsync(on);
                }
                // The map's scroll and the hero's steps: T4b; a frame on while running: nothing.
                Some(Command::Scroll(_) | Command::Hero(_) | Command::Frame) | None => {}
            }
        }
        let ms = clock.ms(win.ticks_ns());
        // A later change may be due already: show the latest instead; if it shows what shows,
        // nothing new is presented (a redraw still is).
        let due = match next.take_if(|f| f.change.t <= ms) {
            None => None,
            Some(d) => {
                let latest = replay.at(ms).map_err(unplayable)?;
                if latest.t <= d.change.t {
                    Some(d)
                } else {
                    report.skipped += 1;
                    match shown.as_mut().filter(|s| s.shows(&latest)) {
                        Some(s) => {
                            s.change.t = latest.t;
                            next = replay
                                .after(latest.t, LOOK_AHEAD_MS)
                                .map_err(unplayable)?
                                .map(|c| Frame::of(&mut painter, c))
                                .transpose()?;
                            None
                        }
                        None => Some(Frame::of(&mut painter, latest)?),
                    }
                }
            }
        };
        if let Some(due) = due {
            let back = show(&mut win, &due, read)?;
            report
                .late_us
                .push(clock.late(win.ticks_ns(), due.change.t) / 1000);
            if let Some(back) = back {
                compare(&back, &due, &mut report);
            }
            next = replay
                .after(due.change.t, LOOK_AHEAD_MS)
                .map_err(unplayable)?
                .map(|c| Frame::of(&mut painter, c))
                .transpose()?;
            shown = Some(due);
        } else if redraw && let Some(f) = &shown {
            show(&mut win, f, false)?;
        }
    }
    let seconds = win.ticks_ns().saturating_sub(started) / 1_000_000;
    lines.push(win.describe());
    drop(win);
    lines.push(format!(
        "{what}, keys: {keys_from}; {} frames shown in {seconds} ms, {} passed over, the map run \
         to {} ms",
        report.late_us.len(),
        report.skipped,
        replay.ran()
    ));
    lines.push(format!(
        "frame after its time (ms): {}",
        summary(&mut report.late_us)
    ));
    let ok = report.difference.is_none();
    if read {
        lines.push(format!(
            "read back: {} of {} frames exact ({}){}",
            report.exact,
            report.read,
            report.geometry.unwrap_or_default(),
            report
                .difference
                .map(|d| format!("; the first difference: {d}"))
                .unwrap_or_default()
        ));
    }
    Ok((lines, ok))
}

/// Presents `frame`; reads it back first if `read`.
fn show(win: &mut Window, frame: &Frame, read: bool) -> Result<Option<ReadBack>, String> {
    if read {
        win.present_read(&frame.xrgb).map(Some)
    } else {
        win.present(&frame.xrgb).map(|()| None)
    }
}

/// Compares what the window read back with `frame` at its scale, the black around it included.
fn compare(back: &ReadBack, frame: &Frame, report: &mut Report) {
    report.read += 1;
    report.geometry = Some(format!(
        "output {} × {}, the frame at ({}, {}), scale {}",
        back.width, back.height, back.left, back.top, back.scale
    ));
    let (fw, fh) = (frame.screen.width(), frame.screen.height());
    let s = back.scale.max(1);
    let inside = |x: u32, y: u32| {
        let (dx, dy) = (x.checked_sub(back.left)? / s, y.checked_sub(back.top)? / s);
        (dx < fw && dy < fh).then(|| (dy * fw + dx) as usize)
    };
    for y in 0..back.height {
        for x in 0..back.width {
            let got = back.pixels[(y * back.width + x) as usize] & 0x00FF_FFFF;
            let want = inside(x, y).map_or(0, |i| frame.xrgb[i]);
            if got != want {
                report.difference.get_or_insert_with(|| {
                    format!(
                        "{} ms, output ({x}, {y}) of {} × {}, the frame at ({}, {}) at scale \
                         {s}: {got:06X}, not {want:06X}",
                        frame.change.t, back.width, back.height, back.left, back.top
                    )
                });
                return;
            }
        }
    }
    report.exact += 1;
}

/// Median, 95th percentile and largest of microseconds, in milliseconds.
fn summary(us: &mut [u64]) -> String {
    if us.is_empty() {
        return "none".into();
    }
    us.sort_unstable();
    let at = |q: usize| us[(us.len() - 1) * q / 100];
    let ms = |v: u64| format!("{}.{}", v / 1000, v % 1000 / 100);
    format!(
        "median {}, p95 {}, max {} ({} frames)",
        ms(at(50)),
        ms(at(95)),
        ms(at(100)),
        us.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use h3_map_view::keys::Chord;
    use h3_map_view::timeline::Direction;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn options_are_parsed() {
        let o = Options::parse(&args(&[
            "walk-1",
            "--keys",
            "mine.toml",
            "--scale",
            "0",
            "--vsync",
            "--fullscreen",
            "--bugs",
            "original",
            "--map",
            "m",
            "--install",
            "i",
            "--check",
            "20",
        ]))
        .unwrap();
        assert_eq!(o.id, "walk-1");
        assert_eq!(o.keys, Some(PathBuf::from("mine.toml")));
        assert!(o.vsync && o.fullscreen);
        assert_eq!(
            (o.scale, o.bugs, o.run),
            (Some(1), Bugs::Original, Some((20, true)))
        );
        assert_eq!((o.map, o.install), (PathBuf::from("m"), PathBuf::from("i")));
        let m = Options::parse(&args(&["boats", "--measure", "5"])).unwrap();
        assert_eq!(
            (m.scale, m.bugs, m.run),
            (None, Bugs::Fixed, Some((5, false)))
        );
        for bad in [
            &[][..],
            &["a", "--scale", "x"],
            &["a", "--bogus"],
            &["a", "b"],
            &["a", "--check"],
            &["a", "--bugs", "some"],
        ] {
            assert!(Options::parse(&args(bad)).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn the_keys_bound_are_the_adapters() {
        assert_eq!(check_keys(&Keys::original()), Ok(()));
        let keys = Keys::with_file("pause = [\"Space\"]\nquit = [\"Ctrl+F12\"]").unwrap();
        assert_eq!(check_keys(&keys), Ok(()));
        assert_eq!(keys.command(true, "F12"), Some(Command::Quit));
        let unknown = Keys::with_file("restart = [\"Backspace\"]").unwrap();
        let message = check_keys(&unknown).unwrap_err();
        assert!(
            message.starts_with("`restart`: Backspace: no such key; the keys: Escape, Space,"),
            "{message}"
        );
        assert!(!message.contains("LeftCtrl"), "{message}");
        let alone = Keys::with_file("hero-north = [\"LeftCtrl\"]").unwrap();
        assert_eq!(
            check_keys(&alone),
            Err(
                "`hero-north`: LeftCtrl: Control is held with a key (`Ctrl+`), not bound alone"
                    .to_owned()
            )
        );
        assert_eq!(
            Chord::parse("Ctrl+Up").map(|c| Keys::original().command(c.ctrl, &c.key)),
            Ok(Some(Command::Scroll(Direction::North)))
        );
    }

    #[test]
    fn delays_are_summarised() {
        assert_eq!(summary(&mut []), "none");
        let mut us: Vec<u64> = (1..=100).map(|v| v * 100).collect();
        assert_eq!(
            summary(&mut us),
            "median 5.0, p95 9.5, max 10.0 (100 frames)"
        );
    }
}
