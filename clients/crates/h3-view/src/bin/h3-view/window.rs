//! `h3-view window` (E17 M2 T5a, ADR-0024): a battle in a window at the original's frame times,
//! M2's exit. The battle is loaded as `play` loads it ([`crate::battle`]): a scenario fought by
//! the bot or a record read, its commands timed by `h3_view::anim`, every asset of its frames in
//! one checkpoint batch. The window shows the frame of the latest change at or before the
//! battle's clock (`h3_view::clock`), composed and widened ahead of its time, presented when it
//! falls due; past the end the battle idles. Keys: Space pauses, → steps a frame while paused, R
//! restarts, F fullscreen, V vsync, C the hardware or the drawn cursor, M the music, E the
//! creatures' sounds, Esc quits.
//!
//! The sound (T5b, [`crate::audio`]): a battle shown from its start opens as the original's does,
//! its start sound playing over the first frame while the battle's clock waits (at most 10 s),
//! then the music starts, looping and fading in, and the clock with it; the stacks' sounds follow
//! the timeline's cues. `--mute` (or no sound device) plays none and does not wait.
//!
//! `--check <s>` and `--measure <s>` run unattended for that many seconds and report when each
//! frame showed against its time; `--check` also reads every frame back before presenting it and
//! compares it with the composed frame at the window's scale (the black around it included). An
//! adapter: the window is `h3-sdl`'s.

use std::path::PathBuf;
use std::process::ExitCode;

use h3_render::screen::Screen;
use h3_sdl::{Config, CursorImage, Event, Key, ReadBack, Window};
use h3_view::anim::{Sound, Speed};
use h3_view::clock::Clock;
use h3_view::compose::FORMAT;
use h3_view::cursor::{self, Cursor};
use h3_view::scene::Draw;

use crate::assets;
use crate::audio::{Library, Player};
use crate::battle::{Battle, Source};

/// How far past the last change the window looks for the next one before it waits for input
/// only (a battle that no longer changes).
const LOOK_AHEAD_MS: u32 = 600_000;

/// How much of the battle's idling is timed at once past its end.
const IDLE_SPAN_MS: u32 = 60_000;

/// The longest wait for input, so that the clock is read now and then.
const WAIT_MS: u32 = 250;

/// The command's options.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    source: Source,
    scale: Option<u32>,
    vsync: bool,
    drawn_cursor: bool,
    fullscreen: bool,
    from: u32,
    mute: bool,
    /// Unattended for this many seconds; whether each frame is read back.
    run: Option<(u32, bool)>,
}

impl Options {
    /// Parses `<scenario.toml | record.jsonl> [--seed <n>] [--speed <0-2>] [--packs <dir>]
    /// [--install <dir>] [--scale <n>] [--vsync] [--drawn-cursor] [--fullscreen] [--from <ms>]
    /// [--mute] [--check <s> | --measure <s>]`.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let clients = assets::clients_dir();
        let env = std::env::var_os("H3_INSTALL").map(PathBuf::from);
        let (mut input, mut seed, mut speed, mut packs, mut install) = (None, 1, 0, None, None);
        let (mut scale, mut vsync, mut drawn_cursor, mut fullscreen) = (None, false, false, false);
        let (mut from, mut run, mut mute) = (0, None, false);
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
                "--seed" => {
                    let v = value()?;
                    seed = v
                        .parse::<u64>()
                        .map_err(|_| format!("window: bad number `{v}` for `{arg}`"))?;
                }
                "--speed" => speed = number(value()?)?.min(2),
                "--packs" => packs = Some(PathBuf::from(value()?)),
                "--install" => install = Some(PathBuf::from(value()?)),
                "--scale" => scale = Some(number(value()?)?.clamp(1, 16)),
                "--vsync" => vsync = true,
                "--drawn-cursor" => drawn_cursor = true,
                "--fullscreen" => fullscreen = true,
                "--from" => from = number(value()?)?,
                "--mute" => mute = true,
                "--check" => run = Some((number(value()?)?, true)),
                "--measure" => run = Some((number(value()?)?, false)),
                flag if flag.starts_with("--") => {
                    return Err(format!("window: unknown option `{flag}`"));
                }
                path if input.is_none() => input = Some(PathBuf::from(path)),
                extra => return Err(format!("window: unexpected `{extra}`")),
            }
        }
        Ok(Self {
            source: Source {
                input: input.ok_or("window: needs a scenario or a record file")?,
                seed,
                speed: Speed::new(u8::try_from(speed).unwrap_or(2)),
                packs: packs.unwrap_or_else(|| clients.join("../packs")),
                install: install
                    .or(env)
                    .unwrap_or_else(|| clients.join("../original-data/install")),
            },
            scale,
            vsync,
            drawn_cursor,
            fullscreen,
            from,
            mute,
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

/// A frame composed ahead of its time: the change it shows, its draw list, the frame and the
/// frame widened.
#[derive(Debug)]
struct Frame {
    t: u32,
    list: Vec<Draw>,
    screen: Screen,
    xrgb: Vec<u32>,
}

/// The times at which the battle may change, timed as far as needed.
#[derive(Debug)]
struct Changes {
    times: Vec<u32>,
    until: u32,
}

impl Changes {
    fn new(battle: &Battle) -> Self {
        let until = battle.timeline.end;
        Self {
            times: battle.timeline.changes(until),
            until,
        }
    }

    /// Times at least up to `t`.
    fn reach(&mut self, battle: &Battle, t: u32) {
        while self.until < t {
            self.until = self.until.saturating_add(IDLE_SPAN_MS);
            self.times = battle.timeline.changes(self.until);
        }
    }

    /// The latest change at or before `t`.
    fn latest(&mut self, battle: &Battle, t: u32) -> u32 {
        self.reach(battle, t);
        let i = self.times.partition_point(|&c| c <= t);
        self.times.get(i.wrapping_sub(1)).copied().unwrap_or(0)
    }

    /// The first change after `t` (none within [`LOOK_AHEAD_MS`]).
    fn after(&mut self, battle: &Battle, t: u32) -> Option<u32> {
        let limit = t.saturating_add(LOOK_AHEAD_MS);
        loop {
            let i = self.times.partition_point(|&c| c <= t);
            if let Some(&next) = self.times.get(i) {
                return Some(next);
            }
            if self.until >= limit {
                return None;
            }
            let more = self.until.saturating_add(1);
            self.reach(battle, more);
        }
    }
}

/// The frame of the change at `t`.
fn frame_at(battle: &Battle, t: u32) -> Result<Frame, String> {
    let list = battle.list_at(t)?;
    let screen = battle.compose(t, &list)?;
    let xrgb = screen.xrgb();
    Ok(Frame {
        t,
        list,
        screen,
        xrgb,
    })
}

/// The next frame after `shown`: the first later change that draws something else.
fn next_frame(
    battle: &Battle,
    changes: &mut Changes,
    shown: &Frame,
) -> Result<Option<Frame>, String> {
    let mut t = shown.t;
    while let Some(next) = changes.after(battle, t) {
        // Past the end of a still battle only the idle steps come, each drawing the same.
        if next > battle.timeline.end && battle.timeline.still() {
            return Ok(None);
        }
        let list = battle.list_at(next)?;
        if list != shown.list {
            let screen = battle.compose(next, &list)?;
            let xrgb = screen.xrgb();
            return Ok(Some(Frame {
                t: next,
                list,
                screen,
                xrgb,
            }));
        }
        t = next;
    }
    Ok(None)
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

/// Opens the window and plays the battle until Esc, a close, or the unattended run's end.
fn window(options: &Options) -> Result<(Vec<String>, bool), String> {
    let mut lines = Vec::new();
    let source = &options.source;
    let mut battle = Battle::load(source, None, &mut lines)?;
    // The checkpoint batch: every frame's names to the end, and the cursor.
    let frames = battle.frames(battle.timeline.end)?;
    battle.load_batch(
        frames.iter().map(|(_, _, l)| l.as_slice()),
        &[cursor::DEFAULT],
    )?;
    let count = frames.len();
    drop(frames);
    let cursor = battle
        .sprite(cursor::DEFAULT)
        .and_then(|s| Cursor::new(s, 0, (0, 0), FORMAT))
        .ok_or_else(|| format!("{}: no first frame", cursor::DEFAULT))?;
    let image = cursor.image(FORMAT);
    let hardware = CursorImage {
        width: image.width,
        height: image.height,
        hot_x: image.hot.0,
        hot_y: image.hot.1,
        rgba: image.rgba,
    };
    let [width, height] = battle.frame_size();
    let mut win = Window::open(&Config {
        title: format!("h3-view — {}", source.stem()),
        width,
        height,
        scale: options.scale,
        vsync: options.vsync,
        fullscreen: options.fullscreen,
    })?;
    let mut drawn_cursor = options.drawn_cursor;
    if !drawn_cursor && let Err(e) = win.set_cursor(Some(hardware.clone())) {
        lines.push(format!("hardware cursor: {e}; drawn instead"));
        drawn_cursor = true;
    }

    // The sound: decoded now, the device opened; none with --mute or without a device.
    let seed = u32::try_from(source.seed & u64::from(u32::MAX)).unwrap_or(1);
    let opening = h3_view::sound::opening(seed);
    let library = if options.mute {
        None
    } else {
        let stacks = battle.stack_sounds();
        // The tent's heal sound, if the battle heals (E17 M3 T3b).
        let heals = battle.timeline.cues.iter().any(|c| c.sound == Sound::Heal);
        let heal = heals.then_some(battle.tables.battle.heal_sound.as_str());
        Some(Library::load(
            &source.install,
            opening,
            stacks,
            heal,
            &mut lines,
        )?)
    };
    let mut player = match library.as_ref().map(Player::open) {
        Some(Ok(p)) => Some(p),
        Some(Err(e)) => {
            lines.push(format!("audio: {e}; no sound"));
            None
        }
        None => None,
    };
    let mut cue = battle.timeline.cue_from(options.from);

    let mut changes = Changes::new(&battle);
    let mut clock = Clock::new(win.ticks_ns(), options.from);
    // The opening: the clock waits while the start sound plays (from the battle's start only).
    let mut opening_until = None;
    if let (Some(p), Some(lib)) = (player.as_mut(), library.as_ref()) {
        let now = win.ticks_ns();
        let wait = if options.from == 0 { p.start(lib) } else { 0 };
        if wait > 0 {
            clock.toggle(now);
            opening_until = Some(now + u64::from(wait) * 1_000_000);
        } else {
            p.start_music(now);
        }
    }
    let mut shown: Option<Frame> = None;
    let mut next = Some(frame_at(&battle, changes.latest(&battle, options.from))?);
    let mut pointer: Option<(i32, i32)> = None;
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
            .and_then(|f| clock.until(now, f.t))
            .or_else(|| opening_until.map(|end| end.saturating_sub(now)))
            .map_or(WAIT_MS, |ns| {
                u32::try_from(ns / 1_000_000)
                    .unwrap_or(WAIT_MS)
                    .min(WAIT_MS)
            });
        let mut redraw = false;
        for event in win.wait(timeout) {
            let now = win.ticks_ns();
            // A key that moves the battle ends the opening first: the clock runs, the music starts.
            if matches!(event, Event::Key(Key::Space | Key::Right | Key::R))
                && options.run.is_none()
                && opening_until.take().is_some()
            {
                clock.toggle(now);
                if let Some(p) = player.as_mut() {
                    p.start_music(now);
                }
                if event == Event::Key(Key::Space) {
                    continue;
                }
            }
            match event {
                Event::Quit | Event::Key(Key::Escape) => break 'run,
                _ if options.run.is_some() => {}
                Event::Key(Key::Space) => {
                    clock.toggle(now);
                    if let Some(p) = player.as_mut() {
                        p.pause(clock.paused(), now);
                    }
                }
                Event::Key(Key::Right) if clock.paused() => {
                    if let Some(f) = &next {
                        // The sounds of the frames stepped over are dropped; the next frame's,
                        // cued as this one shows, play when the battle runs again.
                        clock.seek(now, f.t);
                        cue = battle.timeline.cue_from(f.t);
                        if let Some(p) = player.as_mut() {
                            p.silence();
                        }
                    }
                }
                Event::Key(Key::Right) => {}
                Event::Key(Key::R) => {
                    clock.seek(now, options.from);
                    shown = None;
                    next = Some(frame_at(&battle, changes.latest(&battle, options.from))?);
                    cue = battle.timeline.cue_from(options.from);
                    if let Some(p) = player.as_mut() {
                        p.silence();
                    }
                }
                Event::Key(Key::M) => {
                    if let Some(p) = player.as_mut() {
                        p.music_on = !p.music_on;
                    }
                }
                Event::Key(Key::E) => {
                    if let Some(p) = player.as_mut() {
                        p.effects_on = !p.effects_on;
                        if !p.effects_on {
                            p.silence();
                        }
                    }
                }
                Event::Key(Key::F) => {
                    let on = !win.fullscreen();
                    win.set_fullscreen(on);
                }
                Event::Key(Key::V) => {
                    let on = !win.vsync();
                    win.set_vsync(on);
                }
                Event::Key(Key::C) => {
                    drawn_cursor = !drawn_cursor;
                    let image = (!drawn_cursor).then(|| hardware.clone());
                    if win.set_cursor(image).is_err() {
                        drawn_cursor = true;
                    }
                    redraw = true;
                }
                Event::Pointer(p) => {
                    pointer = p;
                    redraw |= drawn_cursor;
                }
                Event::Redraw => redraw = true,
            }
        }
        // A hardware cursor SDL could not remake (a new scale) is drawn instead.
        if !drawn_cursor && !win.hardware_cursor() {
            lines.push("hardware cursor lost; drawn instead".to_owned());
            win.set_cursor(None)?;
            drawn_cursor = true;
            redraw = true;
        }
        let now = win.ticks_ns();
        if opening_until.is_some_and(|end| now >= end) {
            opening_until = None;
            clock.toggle(now);
            if let Some(p) = player.as_mut() {
                p.start_music(now);
            }
        }
        let battle_ms = clock.ms(now);
        if let (Some(p), Some(lib)) = (player.as_mut(), library.as_ref()) {
            if !clock.paused() {
                while let Some(c) = battle.timeline.cues.get(cue).filter(|c| c.at <= battle_ms) {
                    p.cue(lib, c);
                    cue += 1;
                }
            }
            p.service(lib, now);
        }
        // A later change may be due already: show the latest instead; if it draws what shows,
        // nothing new is presented (a redraw still is).
        let due = match next.take_if(|f| f.t <= battle_ms) {
            None => None,
            Some(d) => {
                let latest = changes.latest(&battle, battle_ms);
                if latest <= d.t {
                    Some(d)
                } else {
                    report.skipped += 1;
                    let late = frame_at(&battle, latest)?;
                    match shown.as_mut().filter(|s| s.list == late.list) {
                        Some(s) => {
                            s.t = latest;
                            next = next_frame(&battle, &mut changes, s)?;
                            None
                        }
                        None => Some(late),
                    }
                }
            }
        };
        if let Some(due) = due {
            let back = show(
                &mut win,
                &due,
                drawn_cursor.then_some(pointer).flatten(),
                &cursor,
                read,
            )?;
            report
                .late_us
                .push(clock.late(win.ticks_ns(), due.t) / 1000);
            if let Some(back) = back {
                compare(&back, &due, &mut report);
            }
            next = next_frame(&battle, &mut changes, &due)?;
            shown = Some(due);
        } else if redraw && let Some(f) = &shown {
            show(
                &mut win,
                f,
                drawn_cursor.then_some(pointer).flatten(),
                &cursor,
                false,
            )?;
        }
    }
    let seconds = win.ticks_ns().saturating_sub(started) / 1_000_000;
    lines.push(win.describe());
    if let Some(lib) = &library {
        lines.push(lib.describe());
    }
    if let Some(p) = &player {
        lines.push(p.describe());
    }
    drop(player);
    drop(win);
    lines.push(format!(
        "{} commands, {} ms at speed {}; {count} frames to the end; {} shown in {} ms, {} \
         passed over",
        battle.commands,
        battle.timeline.end,
        source.speed.setting(),
        report.late_us.len(),
        seconds,
        report.skipped,
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

/// Presents `frame`, with the drawn cursor at `pointer` if any; reads it back first if `read`.
fn show(
    win: &mut Window,
    frame: &Frame,
    pointer: Option<(i32, i32)>,
    cursor: &Cursor,
    read: bool,
) -> Result<Option<ReadBack>, String> {
    let with_cursor;
    let pixels = match pointer {
        Some((x, y)) => {
            let mut screen = frame.screen.clone();
            cursor.draw(&mut screen, x, y);
            with_cursor = screen.xrgb();
            &with_cursor
        }
        None => &frame.xrgb,
    };
    if read {
        win.present_read(pixels).map(Some)
    } else {
        win.present(pixels).map(|()| None)
    }
}

/// Compares what the window read back with the frame at its scale, the black around it included.
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
                        frame.t, back.width, back.height, back.left, back.top
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

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn options_are_parsed() {
        let o = Options::parse(&args(&[
            "scenarios/battle.toml",
            "--seed",
            "7",
            "--speed",
            "5",
            "--scale",
            "0",
            "--vsync",
            "--drawn-cursor",
            "--fullscreen",
            "--from",
            "100",
            "--mute",
            "--check",
            "20",
        ]))
        .unwrap();
        assert_eq!(o.source.seed, 7);
        assert_eq!(o.source.speed, Speed::new(2));
        assert!(o.source.scenario() && o.vsync && o.drawn_cursor && o.fullscreen && o.mute);
        assert_eq!((o.scale, o.from, o.run), (Some(1), 100, Some((20, true))));
        let m = Options::parse(&args(&["a/b.jsonl", "--measure", "5"])).unwrap();
        assert!(!m.source.scenario());
        assert_eq!((m.scale, m.run), (None, Some((5, false))));
        assert!(Options::parse(&args(&[])).is_err());
        assert!(Options::parse(&args(&["a", "--seed", "x"])).is_err());
        assert!(Options::parse(&args(&["a", "--bogus"])).is_err());
        assert!(Options::parse(&args(&["a", "b"])).is_err());
        assert!(Options::parse(&args(&["a", "--check"])).is_err());
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
