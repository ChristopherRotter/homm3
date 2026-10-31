//! `h3-map-view window` (E24 M2 T4a, `h3-view window`'s way: E17 design §8.1, ADR-0024): a
//! recorded sequence or a situation in a window at the original's frame times. The map is set up
//! as `sequences` sets it up (`crate::paint`): a sequence's inputs replayed at their milliseconds
//! (its scrolls, edge cursors and walks), a situation standing still. The timeline runs forward as
//! the map's clock needs (`h3_map_view::replay`, `h3_ui::clock`); the frame of each change is
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
//! Every key goes through the table of commands (`h3_map_view::keys` in the common `h3_ui::keys`;
//! *provisional*): the defaults, or a user's file over them (`--keys`, else `keys.toml` in the working directory when there is
//! one); each key it names must be one the adapter knows (`h3_sdl::Key`), Control held with
//! `Ctrl+`. `--check <s>` and `--measure <s>` run unattended for that many seconds (the window's
//! own keys but quit do nothing; the map's input is taken) and report when each frame showed
//! against its time and how long the frames took to make (the view composed, the cursor drawn,
//! the frame widened: E24 M2 T4b3) and how much of them went to the texture; `--check` also
//! compares every frame made with the same frame composed afresh and widened pixel by pixel (E24
//! M3 T2b2: a frame draws, widens and uploads only what changed), reads every frame back before
//! presenting it and compares it with the composed frame at the window's scale (the black around
//! it included), and at the end plays every action (the recorded and the given) headless
//! (`timeline::play`) and compares each change shown with the headless loop's at its time. An
//! adapter: the window is `h3-sdl`'s.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use h3_map_view::damage::{Held, tidy};
use h3_map_view::keys::{Command, Keys};
use h3_map_view::replay::{Change, Replay};
use h3_map_view::sequence::Sequences;
use h3_map_view::sound;
use h3_map_view::timeline::{self, Action, Cursor, Rules, Start, Unplayable};
use h3_render::Bugs;
use h3_sdl::{Config, CursorImage, Event, Key, ReadBack, Window};
use h3_ui::clock::Clock;
use h3_ui::keys::Window as Own;

use crate::audio::{Library, Player};
use crate::files;
use crate::paint::{self, Assets, Painter, Setup, Widened};

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

/// The table of keys: the user's file (`--keys`, else `keys.toml` here if there is one; its `[map]`
/// table) over the defaults, every key it binds one the adapter knows; and where it came from.
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
    let known: Vec<&str> = Key::ALL.iter().map(|k| k.name()).collect();
    keys.check(&known).map_err(|e| format!("{from}: {e}"))?;
    Ok((keys, from))
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

/// The keys held (a key down again before its release repeats), each with the command its down
/// gave last, and either Control key: a key's up releases that command whatever Ctrl did since,
/// as the original matches a key's up by its scan code alone (0x4564D6; E24 M3 T7's review,
/// F4-1). A repeat that gives nothing (Ctrl pressed meanwhile) keeps the command before it.
#[derive(Debug, Default)]
struct HeldKeys {
    keys: BTreeMap<Key, Option<Command>>,
    ctrl: [bool; 2],
}

impl HeldKeys {
    /// A key's down: whether it repeats and the command the table gives it with Ctrl as held;
    /// none for a Control key itself.
    fn down(&mut self, table: &Keys, key: Key) -> Option<(bool, Option<Command>)> {
        let repeat = self.keys.contains_key(&key);
        if matches!(key, Key::LeftCtrl | Key::RightCtrl) {
            self.keys.insert(key, None);
            self.ctrl[usize::from(key == Key::RightCtrl)] = true;
            return None;
        }
        let command = table.command((), self.ctrl.contains(&true), key.name());
        if command.is_some() || !repeat {
            self.keys.insert(key, command);
        }
        Some((repeat, command))
    }

    /// A key's up: the command its down gave (none for a key whose down the window never saw).
    fn up(&mut self, key: Key) -> Option<Command> {
        if matches!(key, Key::LeftCtrl | Key::RightCtrl) {
            self.ctrl[usize::from(key == Key::RightCtrl)] = false;
        }
        self.keys.remove(&key).flatten()
    }
}

/// A coordinate held on the frame (0 to `size` - 1).
fn clamp(v: i32, size: u32) -> i32 {
    v.clamp(
        0,
        i32::try_from(size)
            .unwrap_or(i32::MAX)
            .saturating_sub(1)
            .max(0),
    )
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
    /// The cursor drawn in the frame for the timeline's; one drawn nowhere is one hidden cursor,
    /// whatever the timeline's point and frame (M2 T6a, the reviewer's R2-9: a hidden cursor's
    /// moves made frames the same as the one shown).
    fn cursor(self, cursor: Cursor) -> Cursor {
        let [x, y] = cursor.at;
        let on = u32::try_from(x).is_ok_and(|x| x < self.frame[0])
            && u32::try_from(y).is_ok_and(|y| y < self.frame[1]);
        if cursor.shown && (!self.live || (!self.hardware && on)) {
            cursor
        } else {
            Cursor {
                shown: false,
                at: OFF,
                frame: 0,
            }
        }
    }

    /// Whether the hardware cursor shows the timeline's.
    fn hardware(self) -> bool {
        self.live && self.hardware
    }
}

/// A frame composed ahead of its time: the change it shows, the cursor drawn in it and the frame
/// widened.
struct Frame {
    change: Change,
    cursor: Cursor,
    widened: Widened,
}

/// What making frames keeps (E24 M3 T2b1, T2b2): the microseconds each took, the widened frames
/// no longer shown (made again into, not allocated afresh: only what changed since the frame each
/// holds widened), and with `--check` the frames compared with the same frame composed afresh and
/// widened pixel by pixel (`Screen::xrgb`), and those the same.
#[derive(Debug, Default)]
struct Maker {
    made_us: Vec<u64>,
    spare: Vec<Widened>,
    check: bool,
    afresh: [usize; 2],
}

impl Maker {
    /// A frame no longer kept: its vector made into again.
    fn recycle(&mut self, frame: Option<Frame>) {
        self.spare.extend(frame.map(|f| f.widened));
    }
}

impl Frame {
    /// The frame `look` shows `change` in, the microseconds it took to make pushed to the maker's.
    fn of(
        painter: &mut Painter<'_>,
        change: Change,
        look: Look,
        (win, maker): (&Window, &mut Maker),
    ) -> Result<Self, String> {
        let started = win.ticks_ns();
        let cursor = look.cursor(change.cursor);
        let mut widened = maker.spare.pop().unwrap_or_default();
        painter.frame_into(&change.drawn, change.buttons, cursor, &mut widened)?;
        maker
            .made_us
            .push(win.ticks_ns().saturating_sub(started) / 1000);
        if maker.check {
            let screen = painter.fresh(&change.drawn, change.buttons, cursor)?;
            maker.afresh[0] += 1;
            maker.afresh[1] += usize::from(screen.xrgb() == widened.xrgb);
        }
        Ok(Self {
            change,
            cursor,
            widened,
        })
    }

    /// Whether it shows what `change` shows as `look` draws it.
    fn shows(&self, change: &Change, look: Look) -> bool {
        self.change
            .shows(self.cursor, change, look.cursor(change.cursor))
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
    /// Times the frame due was passed over because a later change was due already: one each
    /// time, however many changes lay between the two (T7b2, F6-5).
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
    let mut tables = files::load_tables(&options.map)?;
    let layout = files::load_layout(&options.map)?;
    let archive_bytes = files::read_archives(&options.install)?;
    let archives = files::parse_archives(&archive_bytes)?;
    tables.texts = crate::texts::catalogue(&options.install, &archives)?;
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
            .and_then(|library| Ok((library, Player::open(options.bugs)?)))
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
    let read = options.run.is_some_and(|(_, read)| read);
    let mut maker = Maker {
        check: read,
        ..Maker::default()
    };
    let mut texture = Texture {
        size: layout.frame,
        ..Texture::default()
    };
    let mut next = Some(Frame::of(
        &mut painter,
        replay.at(0).map_err(unplayable)?,
        look,
        (&win, &mut maker),
    )?);
    let mut held = HeldKeys::default();
    // The mouse's last point, and the changes shown (the headless check's).
    let mut mouse: Option<[i32; 2]> = None;
    // Whether the window had the focus at the last look (the edge scroll's `app_active`).
    let mut focused = true;
    let mut log: Vec<Change> = Vec::new();
    // The actions given: scrolls, steps, the pointer's moves, the mouse's button, hotkeys.
    let mut given = [0_usize; 5];
    let mut given_at: BTreeSet<u32> = BTreeSet::new();
    let mut renew = look.live;
    let mut report = Report::default();
    let started = win.ticks_ns();
    let stop_ns = options
        .run
        .map(|(seconds, _)| u64::from(seconds) * 1_000_000_000);
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
                    // A point on the window but off the frame (fullscreen's or a large window's
                    // bars) is the frame's nearest: the original's screen is the display, the
                    // mouse held at its sides (M2 T6a, the reviewer's R2-8).
                    let at = p.map_or(OFF, |(x, y)| {
                        [clamp(x, look.frame[0]), clamp(y, look.frame[1])]
                    });
                    (mouse, pointer) = (Some(at), Some(at));
                    continue;
                }
                Event::Click { down, at: (x, y) } => {
                    // The button's own move is the press's or the release's (E24 M3 T6b1: a
                    // panel's button held down until the release); off the frame its nearest, as
                    // a move's (F4-7).
                    let (x, y) = (clamp(x, look.frame[0]), clamp(y, look.frame[1]));
                    (mouse, pointer) = (Some([x, y]), None);
                    input.push(if down {
                        Action::Press([x, y])
                    } else {
                        Action::Release([x, y])
                    });
                    continue;
                }
                // The right button, the mouse's on the map before any key table's binding (E24 M3
                // T7's review, F4-5): its down and up messages.
                Event::Key(Key::RightButton) | Event::KeyUp(Key::RightButton) => {
                    input.extend(pointer.take().map(Action::Pointer));
                    input.push(if matches!(event, Event::Key(_)) {
                        Action::RightPress
                    } else {
                        Action::RightRelease
                    });
                    continue;
                }
                Event::KeyUp(k) => {
                    // A panel's hotkey released clicks its button (E24 M3 T6b1).
                    if let Some(action) = held.up(k).and_then(Command::release) {
                        input.extend(pointer.take().map(Action::Pointer));
                        input.push(action);
                    }
                    continue;
                }
                Event::Key(k) => k,
            };
            let Some((repeat, command)) = held.down(&keys, key) else {
                continue;
            };
            // The map's keys, repeats too (a direction key's as a repeat: MCL-Q25); the window's
            // own once a press.
            if let Some(action) = command.and_then(Command::action) {
                input.extend(pointer.take().map(Action::Pointer));
                input.push(match action {
                    Action::Step(d) if repeat => Action::Repeat(d),
                    action => action,
                });
                continue;
            }
            match command {
                _ if repeat => {}
                Some(Command::Window(Own::Quit)) => break 'run,
                // Unattended, the window's other keys do nothing.
                _ if unattended => {}
                Some(Command::Window(Own::Pause)) => {
                    clock.toggle(now);
                    if let Some((_, player)) = &mut audio {
                        player.pause(clock.paused());
                    }
                }
                Some(Command::Window(Own::Music)) => {
                    if let Some((_, player)) = &mut audio {
                        player.music_on = !player.music_on;
                    }
                }
                Some(Command::Window(Own::Effects)) => {
                    if let Some((_, player)) = &mut audio {
                        let on = !player.effects_on;
                        player.set_effects(on);
                    }
                }
                Some(Command::Window(Own::Frame)) if clock.paused() => {
                    if let Some(f) = &next {
                        clock.seek(now, f.change.t);
                    }
                }
                Some(Command::Window(Own::Restart)) => {
                    replay = new_replay()?;
                    if let Some((_, player)) = &mut audio {
                        player.reset();
                    }
                    clock.seek(now, 0);
                    look.live = live_after.is_none();
                    (shown, renew) = (None, true);
                    log.clear();
                    input.clear();
                    // A situation is live again at once: the mouse's point given anew (M2 T6a, the
                    // reviewer's R2-10).
                    pointer = if look.live { mouse } else { None };
                }
                Some(Command::Window(Own::Cursor)) => {
                    look.hardware = !look.hardware;
                    renew = true;
                }
                Some(Command::Window(Own::Fullscreen)) => {
                    let on = !win.fullscreen();
                    win.set_fullscreen(on);
                }
                Some(Command::Window(Own::Vsync)) => {
                    let on = !win.vsync();
                    win.set_vsync(on);
                }
                // A frame on while running: nothing.
                Some(
                    Command::Scroll(_)
                    | Command::Hero(_)
                    | Command::Panel(_)
                    | Command::Visit
                    | Command::Window(Own::Frame),
                )
                | None => {}
            }
        }
        // The original scrolls at the edges only while it is the active application (`app_active`
        // 0x6783D0, WM_ACTIVATEAPP; design §3.4): at the focus's loss the pointer is given off the
        // frame, the moves without it dropped, with it again the mouse's point (M2 T6a, the
        // reviewer's R2-7). The sound follows it (E24 M3 T6b3, design §4.15): held and faded out
        // without it. Each change of the batch in order, a loss and a regain both (E24 M3 T7's
        // review, F4-8).
        let mut regained = false;
        for &has_focus in win.focus_changes() {
            if has_focus == focused {
                continue;
            }
            if has_focus {
                regained = true;
            } else {
                input.push(Action::Pointer(OFF));
            }
            if let Some((library, player)) = &mut audio {
                player.focus(library, has_focus, &mut lines);
                lines.push(format!(
                    "{} the focus: the sound {}",
                    if has_focus { "regained" } else { "lost" },
                    if has_focus { "resumed" } else { "held" }
                ));
            }
            focused = has_focus;
        }
        if !focused {
            pointer = None;
        } else if regained {
            pointer = pointer.or(mouse);
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
                    Action::Step(_) | Action::Repeat(_) => 1,
                    Action::Pointer(_) => 2,
                    Action::Click(_)
                    | Action::Press(_)
                    | Action::Release(_)
                    | Action::RightPress
                    | Action::RightRelease => 3,
                    Action::Key(_) | Action::KeyUp(_) | Action::Visit | Action::Up => 4,
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
                    let again =
                        Frame::of(&mut painter, f.change.clone(), look, (&win, &mut maker))?;
                    maker.recycle(Some(f));
                    if let Some(back) = texture.show(&mut win, &painter, &again, read)? {
                        compare(&back, &again, look.frame, &mut report);
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
                (old, c) => {
                    maker.recycle(old);
                    c.map(|c| Frame::of(&mut painter, c, look, (&win, &mut maker)))
                        .transpose()?
                }
            };
        }
        // A redraw asked for (the window's contents may be lost): whatever this turn presents is
        // uploaded whole, and the frame shown is presented again when nothing new is (E24 M3 T7's
        // review, F2-5: a frame due in the same turn took the redraw's place, in part or not at
        // all).
        if redraw {
            texture.held = None;
        }
        let mut presented = false;
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
                Some(s) if s.shows(&change, look) => {
                    s.change = change;
                    maker.recycle(Some(d));
                }
                _ => {
                    let due = if d.change == change && d.shows(&change, look) {
                        d
                    } else {
                        maker.recycle(Some(d));
                        Frame::of(&mut painter, change, look, (&win, &mut maker))?
                    };
                    let back = texture.show(&mut win, &painter, &due, read)?;
                    presented = true;
                    let late = clock.late(win.ticks_ns(), due.change.t) / 1000;
                    report.late_us.push(late);
                    if given_at.contains(&due.change.t) {
                        report.input_late_us.push(late);
                    }
                    if let Some(back) = back {
                        compare(&back, &due, look.frame, &mut report);
                    }
                    let old = shown.replace(due);
                    maker.recycle(old);
                }
            }
            let t = shown.as_ref().map_or(0, |f| f.change.t);
            next = replay
                .after(t, LOOK_AHEAD_MS)
                .map_err(unplayable)?
                .map(|c| Frame::of(&mut painter, c, look, (&win, &mut maker)))
                .transpose()?;
        }
        if redraw
            && !presented
            && let Some(f) = &shown
        {
            texture.show(&mut win, &painter, f, false)?;
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
    let [scrolls, steps, moves, clicks, hotkeys] = given;
    let view = shown
        .as_ref()
        .map(|f| f.change.drawn.view)
        .unwrap_or(start.view);
    lines.push(format!(
        "{what}, keys: {keys_from}; {} frames shown in {seconds} ms, {} times a later one shown instead, the map run \
         to {} ms; given: {scrolls} scrolls, {steps} steps, {moves} moves, {clicks} presses and \
         releases, {hotkeys} hotkeys; the view at ({}, {}, {})",
        report.late_us.len(),
        report.skipped,
        replay.ran(),
        view.x,
        view.y,
        view.z
    ));
    lines.push(format!(
        "frame after its time (ms): {}; at an input's time: {}; a frame made in: {}; {}",
        summary(&mut report.late_us),
        summary(&mut report.input_late_us),
        summary(&mut maker.made_us),
        texture.describe()
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
        let [checked, same] = maker.afresh;
        ok &= checked == same;
        lines.push(format!(
            "afresh: {same} of {checked} frames made as the same frame composed afresh and \
             widened pixel by pixel"
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

/// What the window's texture holds (E24 M3 T2b2, *provisional*): the frame last uploaded, so
/// that the next uploads only what differs from it; the frames uploaded, those uploaded whole, and
/// the pixels uploaded.
#[derive(Debug, Default)]
struct Texture {
    size: [u32; 2],
    held: Option<Held>,
    rects: Vec<[i32; 4]>,
    uploads: [u64; 3],
}

impl Texture {
    /// Presents `frame`, uploading only what differs from the frame the texture holds when the
    /// painter knows it (the whole frame otherwise); reads it back first if `read`.
    fn show(
        &mut self,
        win: &mut Window,
        painter: &Painter<'_>,
        frame: &Frame,
        read: bool,
    ) -> Result<Option<ReadBack>, String> {
        self.rects.clear();
        let size = self.size;
        let part = match (self.held, frame.widened.held) {
            (Some(a), Some(b)) if painter.changed(a, b, &mut self.rects) => {
                tidy(&mut self.rects, size, MOST_RECTS);
                Some(
                    self.rects
                        .iter()
                        .map(|r| r.map(|v| u32::try_from(v).unwrap_or(0)))
                        .collect::<Vec<_>>(),
                )
            }
            _ => None,
        };
        let pixels = part
            .as_ref()
            .map_or(u64::from(size[0]) * u64::from(size[1]), |rects| {
                rects
                    .iter()
                    .map(|r| u64::from(r[2] - r[0]) * u64::from(r[3] - r[1]))
                    .sum()
            });
        self.uploads[0] += 1;
        self.uploads[1] += u64::from(part.is_none());
        self.uploads[2] += pixels;
        let xrgb = &frame.widened.xrgb;
        let back = if read {
            win.present_read_part(xrgb, part.as_deref()).map(Some)
        } else {
            win.present_part(xrgb, part.as_deref()).map(|()| None)
        };
        // A frame not shown leaves the texture unknown.
        self.held = frame.widened.held.filter(|_| back.is_ok());
        back
    }

    /// What was uploaded, for the report.
    fn describe(&self) -> String {
        let [frames, whole, pixels] = self.uploads;
        let size = self.size;
        let all = frames * u64::from(size[0]) * u64::from(size[1]);
        let tenths = (pixels * 1000).checked_div(all).unwrap_or(0);
        format!(
            "uploaded: {frames} frames, {whole} whole; {}.{} % of their pixels",
            tenths / 10,
            tenths % 10
        )
    }
}

/// Past this many rectangles a frame's upload is the smallest rectangle holding them.
const MOST_RECTS: usize = 64;

/// Compares what the window read back with `frame` (of `size`) at its scale, the black around it
/// included.
fn compare(back: &ReadBack, frame: &Frame, size: [u32; 2], report: &mut Report) {
    report.read += 1;
    report.geometry = Some(format!(
        "output {} × {}, the frame at ({}, {}), scale {}",
        back.width, back.height, back.left, back.top, back.scale
    ));
    let [fw, fh] = size;
    let s = back.scale.max(1);
    let inside = |x: u32, y: u32| {
        let (dx, dy) = (x.checked_sub(back.left)? / s, y.checked_sub(back.top)? / s);
        (dx < fw && dy < fh).then(|| (dy * fw + dx) as usize)
    };
    for y in 0..back.height {
        for x in 0..back.width {
            let got = back.pixels[(y * back.width + x) as usize] & 0x00FF_FFFF;
            let want = inside(x, y).map_or(0, |i| frame.widened.xrgb[i]);
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

    use h3_map_view::timeline::Direction;
    use h3_ui::keys::Chord;

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
        let known: Vec<&str> = Key::ALL.iter().map(|k| k.name()).collect();
        assert_eq!(Keys::original().check(&known), Ok(()));
        let keys = Keys::with_file("[map]\npause = [\"Space\"]\nvisit = []\nquit = [\"Ctrl+F12\"]")
            .unwrap();
        assert_eq!(keys.check(&known), Ok(()));
        assert_eq!(
            keys.command((), true, "F12"),
            Some(Command::Window(Own::Quit))
        );
        let unknown = Keys::with_file("[map]\nrestart = [\"Backspace\"]").unwrap();
        let message = unknown.check(&known).unwrap_err();
        assert!(
            message.starts_with("`restart`: Backspace: no such key; the keys: Escape, Space,"),
            "{message}"
        );
        assert!(!message.contains("LeftCtrl"), "{message}");
        assert_eq!(
            Chord::parse("Ctrl+Up").map(|c| Keys::original().command((), c.ctrl, &c.key)),
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
        // Drawn nowhere: one hidden cursor, whatever its point and frame (R2-9).
        let hidden = Cursor {
            shown: false,
            at: OFF,
            frame: 0,
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
        assert_eq!(
            look.cursor(Cursor {
                shown: false,
                ..cursor
            }),
            hidden
        );
        look.hardware = true;
        assert_eq!(
            look.cursor(Cursor {
                at: [99, 99],
                ..cursor
            }),
            hidden
        );
    }

    #[test]
    fn a_keys_up_releases_the_command_its_down_gave() {
        // H down (next hero), Ctrl down, H up: next hero's release, not Ctrl+H's nothing (E24
        // M3 T7's review, F4-1); Ctrl alone and an up never seen down give nothing.
        let table = Keys::original();
        let mut held = HeldKeys::default();
        let h = Key::from_name("H").unwrap();
        let next = Some(Command::Panel(timeline::Hotkey::NextHero));
        assert_eq!(held.down(&table, h), Some((false, next)));
        assert_eq!(held.down(&table, Key::LeftCtrl), None);
        // Its repeat with Ctrl held gives nothing and keeps next hero's.
        assert_eq!(held.down(&table, h), Some((true, None)));
        assert_eq!(held.up(h), next);
        assert_eq!(held.up(Key::LeftCtrl), None);
        assert_eq!(held.up(h), None);
        // Ctrl+H down, Ctrl up, H's repeat: next hero's, released at H's up.
        held.down(&table, Key::RightCtrl);
        assert_eq!(held.down(&table, h), Some((false, None)));
        held.up(Key::RightCtrl);
        assert_eq!(held.down(&table, h), Some((true, next)));
        assert_eq!(held.up(h), next);
    }

    #[test]
    fn a_point_off_the_frame_is_its_nearest() {
        assert_eq!(
            [
                clamp(-240, 800),
                clamp(5, 800),
                clamp(900, 800),
                clamp(700, 600)
            ],
            [0, 5, 799, 599]
        );
        assert_eq!(clamp(3, 0), 0);
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
