//! `h3-map-view window` (E24 M2 T4a, `h3-view window`'s way: E17 design §8.1, ADR-0024): a
//! recorded sequence or a situation in a window at the original's frame times. The map is set up
//! as `sequences` sets it up (`crate::paint`): a sequence's inputs replayed at their milliseconds
//! (its scrolls, edge cursors and walks), a situation standing still. The timeline runs forward as
//! the map's clock needs (`h3_map_view::replay`, `h3_map_view::clock`); the frame of each change is
//! composed and widened ahead of its time and presented when it falls due, a change overtaken by a
//! later one passed over. During a sequence's inputs the cursor is the recording's, drawn in the
//! frame (the system's is hidden over the window).
//!
//! The map's live input (M2 T4b, *provisional*; design §3.10): past a sequence's inputs, and in a
//! situation from the start, the keys and the mouse are the timeline's actions, given at the
//! clock's millisecond (after the change shown: `Replay::give`) — the table's scroll and step
//! keys (a held key's repeats too, as the original's: design §3.4), the pointer's moves (the last
//! of a batch of events; off the window a point off the frame) and the left button's presses. The
//! timeline plays them live (`Start::live`): the pointer over the view shows the arrow, a click it
//! cannot play only moves the pointer (M3's). The cursor then is the hardware one with the
//! timeline's frame (hidden while a walk hides it), or drawn in the frame at the timeline's point
//! (`--drawn-cursor`; not while that point lies off the frame); the cursor command switches.
//!
//! Every key goes through the table of commands (`h3_map_view::keys`; *provisional*): the defaults,
//! or a user's file over them (`--keys`, else `keys.toml` in the working directory when there is
//! one); each key it names must be one the adapter knows (`h3_sdl::Key`), Control held with
//! `Ctrl+`. `--check <s>` and `--measure <s>` run unattended for that many seconds (the window's
//! own keys but quit do nothing; the map's input is taken) and report when each frame showed
//! against its time; `--check` also reads every frame back before presenting it and compares it
//! with the composed frame at the window's scale (the black around it included), and at the end
//! plays every action (the recorded and the given) headless (`timeline::play`) and compares each
//! change shown with the headless loop's at its time. An adapter: the window is `h3-sdl`'s.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use h3_map_view::clock::Clock;
use h3_map_view::keys::{Command, Keys};
use h3_map_view::replay::{Change, Replay};
use h3_map_view::sequence::Sequences;
use h3_map_view::sound;
use h3_map_view::timeline::{self, Action, Cursor, Rules, Start, Unplayable};
use h3_render::Bugs;
use h3_render::screen::Screen;
use h3_sdl::{Config, CursorImage, Event, Key, ReadBack, Window};

use crate::audio::{Library, Player};
use crate::files;
use crate::paint::{self, Assets, Painter, Setup};

/// How far past a change the window looks for the next one before it waits for input only.
const LOOK_AHEAD_MS: u32 = 600_000;

/// The longest wait for input, so that the clock is read now and then.
const WAIT_MS: u32 = 250;

/// The user's file of keys looked for in the working directory.
const KEYS_FILE: &str = "keys.toml";

/// The pointer's point once it left the window: off the frame, where no edge scrolls.
const OFF: [i32; 2] = [-1, -1];

/// The command's options.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    id: String,
    keys: Option<PathBuf>,
    scale: Option<u32>,
    vsync: bool,
    fullscreen: bool,
    drawn_cursor: bool,
    bugs: Bugs,
    install: PathBuf,
    map: PathBuf,
    /// Unattended for this many seconds; whether each frame is read back.
    run: Option<(u32, bool)>,
    /// No sound (E24 M2 T5).
    mute: bool,
    /// Where each cue played is written (the cue's line and the clock's millisecond it played at).
    cues: Option<PathBuf>,
}

impl Options {
    /// Parses `<sequence | situation> [--keys <file>] [--scale <n>] [--vsync] [--fullscreen]
    /// [--drawn-cursor] [--mute] [--cues <file>] [--bugs fixed|original] [--install <dir>]
    /// [--map <dir>] [--check <s> | --measure <s>]`.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let (mut id, mut keys, mut scale, mut install, mut map) = (None, None, None, None, None);
        let (mut vsync, mut fullscreen, mut drawn_cursor) = (false, false, false);
        let (mut mute, mut cues) = (false, None);
        let (mut bugs, mut run) = (Bugs::Fixed, None);
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
                "--drawn-cursor" => drawn_cursor = true,
                "--mute" => mute = true,
                "--cues" => cues = Some(PathBuf::from(value()?)),
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
            drawn_cursor,
            bugs,
            install: install.unwrap_or_else(files::default_install),
            map: map.unwrap_or_else(files::default_map),
            run,
            mute,
            cues,
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

/// How a frame shows the cursor: the recording's in the frame until the map is live, then the
/// hardware cursor (none in the frame) or the drawn one (not while its point lies off the frame).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Look {
    /// The map takes the player's input.
    live: bool,
    /// The hardware cursor chosen.
    hardware: bool,
    /// The frame's size.
    frame: [u32; 2],
}

impl Look {
    /// The cursor drawn in the frame for the timeline's.
    fn cursor(self, cursor: Cursor) -> Cursor {
        let [x, y] = cursor.at;
        let on = u32::try_from(x).is_ok_and(|x| x < self.frame[0])
            && u32::try_from(y).is_ok_and(|y| y < self.frame[1]);
        Cursor {
            shown: cursor.shown && (!self.live || (!self.hardware && on)),
            ..cursor
        }
    }

    /// Whether the hardware cursor shows the timeline's.
    fn hardware(self) -> bool {
        self.live && self.hardware
    }
}

/// A frame composed ahead of its time: the change it shows, the cursor drawn in it, the frame and
/// the frame widened.
struct Frame {
    change: Change,
    cursor: Cursor,
    screen: Screen,
    xrgb: Vec<u32>,
}

impl Frame {
    fn of(painter: &mut Painter<'_>, change: Change, look: Look) -> Result<Self, String> {
        let cursor = look.cursor(change.cursor);
        let screen = painter.frame(&change.drawn, cursor)?;
        let xrgb = screen.xrgb();
        Ok(Self {
            change,
            cursor,
            screen,
            xrgb,
        })
    }

    /// Whether it shows what `change` shows as `look` draws it.
    fn shows(&self, change: &Change, look: Look) -> bool {
        self.change.drawn == change.drawn && self.cursor == look.cursor(change.cursor)
    }
}

/// Sets the hardware cursor to the timeline's (`None`: the system's hidden over the window, as the
/// cursor a walk hides), unless `set` holds it already; whether SDL made it (if not, the system's
/// is hidden and `set` is `None`).
fn hardware(
    win: &mut Window,
    painter: &mut Painter<'_>,
    set: &mut Option<(u32, bool)>,
    cursor: Option<Cursor>,
) -> Result<bool, String> {
    let want = cursor.map(|c| (c.frame, c.shown));
    if *set == want {
        return Ok(true);
    }
    let image = match cursor {
        Some(c) if c.shown => {
            let image = painter.cursor_image(c.frame)?;
            Some(CursorImage {
                width: image.width,
                height: image.height,
                hot_x: image.hot.0,
                hot_y: image.hot.1,
                rgba: image.rgba,
            })
        }
        _ => None,
    };
    let made = win.set_cursor(image).is_ok();
    *set = if made { want } else { None };
    Ok(made)
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
    /// The same for the frames at a millisecond the player's input was given at (composed once it
    /// came, not ahead).
    input_late_us: Vec<u64>,
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

/// The changes shown checked against a headless play of the same start and actions: how many are
/// the headless loop's at their times, and the first that is not.
fn headless(
    rules: &Rules,
    start: &Start,
    actions: &BTreeMap<u32, Vec<Action>>,
    shown: &[Change],
) -> Result<(usize, Option<String>), String> {
    let until = shown.iter().map(|c| c.t).max().unwrap_or(0);
    let states = timeline::play(rules.clone(), start, actions, until).map_err(unplayable)?;
    let mut same = 0;
    let mut first = None;
    for c in shown {
        let s = usize::try_from(c.t).ok().and_then(|t| states.get(t));
        if s.is_some_and(|s| s.drawn == c.drawn && s.cursor == c.cursor) {
            same += 1;
        } else {
            first.get_or_insert_with(|| {
                format!(
                    "{} ms: shown {:?} {:?}, headless {:?}",
                    c.t,
                    c.drawn,
                    c.cursor,
                    s.map(|s| (&s.drawn, s.cursor))
                )
            });
        }
    }
    Ok((same, first))
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
        mut start,
        actions,
        ..
    } = setup;
    start.live = true;
    let rules = Rules::new(&tables, &layout, options.bugs);
    let new_replay =
        || Replay::new(rules.clone(), &start, actions.clone()).map_err(|why| unplayable((0, why)));
    let mut replay = new_replay()?;
    // The sound (E24 M2 T5): the WAVs decoded now, the device opened; none muted, without the
    // tables' sounds, or without a device (noted).
    let mut audio = match (&tables.sound, options.mute) {
        (Some(rules), false) => match Library::load(&options.install, rules, &mut lines)
            .and_then(|library| Ok((library, Player::open()?)))
        {
            Ok(audio) => Some(audio),
            Err(e) => {
                lines.push(format!("sound: {e}; none played"));
                None
            }
        },
        _ => None,
    };
    let mut cue_lines: Vec<String> = Vec::new();
    // The cues at 0 taken now, their music decoded before the window opens; played at its first
    // turn.
    let mut first_cues = Vec::new();
    if let Some((library, _)) = &mut audio {
        first_cues = replay.take_cues(0).map_err(unplayable)?;
        library.prepare(&first_cues, &mut lines);
    }
    // The map is live after a sequence's last input (from the start without one).
    let live_after = actions.keys().next_back().copied();
    let [width, height] = layout.frame;
    let mut win = Window::open(&Config {
        title: format!("h3-map-view — {}", options.id),
        width,
        height,
        scale: options.scale,
        vsync: options.vsync,
        fullscreen: options.fullscreen,
    })?;
    win.set_key_repeats(true);
    let mut look = Look {
        live: live_after.is_none(),
        hardware: !options.drawn_cursor,
        frame: layout.frame,
    };
    // The hardware cursor as set (`None`: the system's hidden, as `Window::open` leaves it).
    let mut hw: Option<(u32, bool)> = None;
    let mut clock = Clock::new(win.ticks_ns(), 0);
    let mut shown: Option<Frame> = None;
    let mut next = Some(Frame::of(
        &mut painter,
        replay.at(0).map_err(unplayable)?,
        look,
    )?);
    // Either Control key held; the keys held (a key down again before its release repeats).
    let mut ctrl = [false, false];
    let mut held: BTreeSet<Key> = BTreeSet::new();
    // The mouse's last point, and the changes shown (the headless check's).
    let mut mouse: Option<[i32; 2]> = None;
    let mut log: Vec<Change> = Vec::new();
    // The actions given: scrolls, steps, the pointer's moves, clicks.
    let mut given = [0_usize; 4];
    let mut given_at: BTreeSet<u32> = BTreeSet::new();
    let mut renew = look.live;
    let mut report = Report::default();
    let started = win.ticks_ns();
    let stop_ns = options
        .run
        .map(|(seconds, _)| u64::from(seconds) * 1_000_000_000);
    let read = options.run.is_some_and(|(_, read)| read);
    let unattended = options.run.is_some();
    'run: loop {
        let now = win.ticks_ns();
        if stop_ns.is_some_and(|s| now.saturating_sub(started) >= s) {
            break;
        }
        let mut input: Vec<Action> = Vec::new();
        if !look.live && live_after.is_some_and(|t| clock.ms(now) > t) {
            look.live = true;
            renew = true;
            input.extend(mouse.map(Action::Pointer));
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
        // The pointer's last move not yet given (a batch's moves are one).
        let mut pointer: Option<[i32; 2]> = None;
        for event in win.wait(timeout) {
            let now = win.ticks_ns();
            let key = match event {
                Event::Quit => break 'run,
                Event::Redraw => {
                    redraw = true;
                    continue;
                }
                Event::Pointer(p) => {
                    let at = p.map_or(OFF, |(x, y)| [x, y]);
                    (mouse, pointer) = (Some(at), Some(at));
                    continue;
                }
                Event::Click {
                    down: true,
                    at: (x, y),
                } => {
                    // The click's own move is the click's.
                    (mouse, pointer) = (Some([x, y]), None);
                    input.push(Action::Click([x, y]));
                    continue;
                }
                Event::Click { down: false, .. } => continue,
                Event::KeyUp(k) => {
                    held.remove(&k);
                    if matches!(k, Key::LeftCtrl | Key::RightCtrl) {
                        ctrl[usize::from(k == Key::RightCtrl)] = false;
                    }
                    continue;
                }
                Event::Key(k) => k,
            };
            let repeat = !held.insert(key);
            if matches!(key, Key::LeftCtrl | Key::RightCtrl) {
                ctrl[usize::from(key == Key::RightCtrl)] = true;
                continue;
            }
            let command = keys.command(ctrl.contains(&true), key.name());
            // The map's keys, repeats too; the window's own once a press.
            if let Some(action) = command.and_then(Command::action) {
                input.extend(pointer.take().map(Action::Pointer));
                input.push(action);
                continue;
            }
            match command {
                _ if repeat => {}
                Some(Command::Quit) => break 'run,
                // Unattended, the window's other keys do nothing.
                _ if unattended => {}
                Some(Command::Pause) => {
                    clock.toggle(now);
                    if let Some((_, player)) = &mut audio {
                        player.pause(clock.paused());
                    }
                }
                Some(Command::Music) => {
                    if let Some((_, player)) = &mut audio {
                        player.music_on = !player.music_on;
                    }
                }
                Some(Command::Effects) => {
                    if let Some((_, player)) = &mut audio {
                        let on = !player.effects_on;
                        player.set_effects(on);
                    }
                }
                Some(Command::Frame) if clock.paused() => {
                    if let Some(f) = &next {
                        clock.seek(now, f.change.t);
                    }
                }
                Some(Command::Restart) => {
                    replay = new_replay()?;
                    if let Some((_, player)) = &mut audio {
                        player.reset();
                    }
                    clock.seek(now, 0);
                    look.live = live_after.is_none();
                    (shown, renew) = (None, true);
                    log.clear();
                    input.clear();
                    pointer = None;
                }
                Some(Command::Cursor) => {
                    look.hardware = !look.hardware;
                    renew = true;
                }
                Some(Command::Fullscreen) => {
                    let on = !win.fullscreen();
                    win.set_fullscreen(on);
                }
                Some(Command::Vsync) => {
                    let on = !win.vsync();
                    win.set_vsync(on);
                }
                // A frame on while running: nothing.
                Some(Command::Scroll(_) | Command::Hero(_) | Command::Frame) | None => {}
            }
        }
        input.extend(pointer.map(Action::Pointer));
        let ms = clock.ms(win.ticks_ns());
        // The player's input, after the change shown.
        if look.live && !input.is_empty() {
            let t = ms.max(shown.as_ref().map_or(0, |f| f.change.t).saturating_add(1));
            given_at.insert(t);
            for action in input {
                given[match action {
                    Action::Scroll(_) => 0,
                    Action::Step(_) => 1,
                    Action::Pointer(_) => 2,
                    Action::Click(_) => 3,
                }] += 1;
                replay.give(t, action);
            }
            renew = true;
        }
        // The sound's cues up to the clock, then the streams kept going.
        if let Some((library, player)) = &mut audio {
            let mut cues = std::mem::take(&mut first_cues);
            cues.extend(replay.take_cues(ms).map_err(unplayable)?);
            for (t, cue) in cues {
                player.cue(library, &cue, &mut lines);
                if options.cues.is_some() {
                    cue_lines.push(format!("{ms} {}", sound::line(t, &cue)));
                }
            }
            player.service(library, &mut lines);
        }
        // A hardware cursor SDL could not remake (a new scale) is drawn instead.
        if look.hardware() && hw.is_some_and(|(_, on)| on) && !win.hardware_cursor() {
            lines.push("hardware cursor lost; drawn instead".to_owned());
            look.hardware = false;
            renew = true;
        }
        if std::mem::take(&mut renew) {
            // The cursor as the look has it, the frame shown again if it changes, the next made
            // again.
            let current = shown.as_ref().map(|f| f.change.cursor);
            if !hardware(
                &mut win,
                &mut painter,
                &mut hw,
                current.filter(|_| look.hardware()),
            )? {
                lines.push("hardware cursor: SDL could not make it; drawn instead".to_owned());
                look.hardware = false;
            }
            if let Some(f) = shown.take() {
                let again = if f.shows(&f.change, look) {
                    f
                } else {
                    let again = Frame::of(&mut painter, f.change, look)?;
                    if let Some(back) = show(&mut win, &again, read)? {
                        compare(&back, &again, &mut report);
                    }
                    again
                };
                shown = Some(again);
            }
            let change = match &shown {
                Some(f) => replay.after(f.change.t, LOOK_AHEAD_MS),
                None => replay.at(0).map(Some),
            }
            .map_err(unplayable)?;
            next = match (next.take(), change) {
                (Some(n), Some(c)) if n.change == c && n.shows(&c, look) => Some(n),
                (_, c) => c.map(|c| Frame::of(&mut painter, c, look)).transpose()?,
            };
        }
        // A later change may be due already: the latest is shown instead; if it shows what shows,
        // nothing new is presented (a redraw still is).
        if let Some(d) = next.take_if(|f| f.change.t <= ms) {
            let latest = replay.at(ms).map_err(unplayable)?;
            let change = if latest.t <= d.change.t {
                d.change.clone()
            } else {
                report.skipped += 1;
                latest
            };
            if look.hardware() && !hardware(&mut win, &mut painter, &mut hw, Some(change.cursor))? {
                lines.push("hardware cursor: SDL could not make it; drawn instead".to_owned());
                look.hardware = false;
            }
            if read {
                log.push(change.clone());
            }
            match &mut shown {
                Some(s) if s.shows(&change, look) => s.change = change,
                _ => {
                    let due = if d.change == change && d.shows(&change, look) {
                        d
                    } else {
                        Frame::of(&mut painter, change, look)?
                    };
                    let back = show(&mut win, &due, read)?;
                    let late = clock.late(win.ticks_ns(), due.change.t) / 1000;
                    report.late_us.push(late);
                    if given_at.contains(&due.change.t) {
                        report.input_late_us.push(late);
                    }
                    if let Some(back) = back {
                        compare(&back, &due, &mut report);
                    }
                    shown = Some(due);
                }
            }
            let t = shown.as_ref().map_or(0, |f| f.change.t);
            next = replay
                .after(t, LOOK_AHEAD_MS)
                .map_err(unplayable)?
                .map(|c| Frame::of(&mut painter, c, look))
                .transpose()?;
        } else if redraw && let Some(f) = &shown {
            show(&mut win, f, false)?;
        }
    }
    let seconds = win.ticks_ns().saturating_sub(started) / 1_000_000;
    lines.push(win.describe());
    if let Some((library, player)) = &audio {
        lines.push(format!(
            "{}; {}; {} cues played",
            library.describe(),
            player.describe(),
            player.played
        ));
    }
    drop(audio);
    if let Some(path) = &options.cues {
        let mut text = cue_lines.join("\n");
        text.push('\n');
        fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    drop(win);
    let [scrolls, steps, moves, clicks] = given;
    let view = shown
        .as_ref()
        .map(|f| f.change.drawn.view)
        .unwrap_or(start.view);
    lines.push(format!(
        "{what}, keys: {keys_from}; {} frames shown in {seconds} ms, {} passed over, the map run \
         to {} ms; given: {scrolls} scrolls, {steps} steps, {moves} moves, {clicks} clicks; the \
         view at ({}, {}, {})",
        report.late_us.len(),
        report.skipped,
        replay.ran(),
        view.x,
        view.y,
        view.z
    ));
    lines.push(format!(
        "frame after its time (ms): {}; at an input's time: {}",
        summary(&mut report.late_us),
        summary(&mut report.input_late_us)
    ));
    let mut ok = report.difference.is_none();
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
        let (same, first) = headless(&rules, &start, replay.actions(), &log)?;
        ok &= first.is_none();
        lines.push(format!(
            "headless: {same} of {} changes shown as the headless loop's{}",
            log.len(),
            first
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
            "--drawn-cursor",
            "--mute",
            "--cues",
            "c.txt",
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
        assert!(o.vsync && o.fullscreen && o.drawn_cursor && o.mute);
        assert_eq!(o.cues, Some(PathBuf::from("c.txt")));
        assert_eq!(
            (o.scale, o.bugs, o.run),
            (Some(1), Bugs::Original, Some((20, true)))
        );
        assert_eq!((o.map, o.install), (PathBuf::from("m"), PathBuf::from("i")));
        let m = Options::parse(&args(&["boats", "--measure", "5"])).unwrap();
        assert!(!m.drawn_cursor && !m.mute && m.cues.is_none());
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
    fn the_cursor_in_the_frame_by_the_look() {
        let cursor = Cursor {
            frame: 33,
            at: [10, 20],
            shown: true,
        };
        let hidden = Cursor {
            shown: false,
            ..cursor
        };
        let mut look = Look {
            live: false,
            hardware: true,
            frame: [800, 600],
        };
        // The recording's, as it is.
        assert_eq!(look.cursor(cursor), cursor);
        assert!(look.cursor(Cursor { at: OFF, ..cursor }).shown);
        assert!(!look.hardware());
        // Live: the hardware one, none in the frame; or drawn while its point is on the frame.
        look.live = true;
        assert!(look.hardware());
        assert_eq!(look.cursor(cursor), hidden);
        look.hardware = false;
        assert_eq!(look.cursor(cursor), cursor);
        for at in [OFF, [800, 5], [5, 600], [-1, 5]] {
            assert!(!look.cursor(Cursor { at, ..cursor }).shown, "{at:?}");
        }
        assert_eq!(look.cursor(hidden), hidden);
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
