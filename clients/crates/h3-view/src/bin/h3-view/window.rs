//! `h3-view window` (E17 M2 T5a, ADR-0024): a battle in a window at the original's frame times,
//! M2's exit. The battle is loaded as `play` loads it ([`crate::battle`]): a scenario fought by
//! the bot as it plays (E17 M5 T1b: each command when the window reaches the timeline's end) or a
//! record read, its commands timed by `h3_view::anim`, the assets of its frames in one checkpoint
//! batch as they are first drawn. The window shows the frame of the latest change at or before the
//! battle's clock (`h3_ui::clock`), composed and widened ahead of its time, presented when it
//! falls due; past the end the battle idles. Every key goes through the battle's table of
//! commands (E24 M3 T1b, `h3_view::keys`, *provisional*; the user's file `--keys`, else
//! `keys.toml` here, its `[battle]` table over the defaults): the window's own are the map
//! window's (F2 the music, F3 the creatures' sounds, Ctrl+F5 pause, Ctrl+F6 a frame on while
//! paused, Ctrl+F7 restart, Ctrl+F8 vsync, F9 the hardware or the drawn cursor, F11 fullscreen,
//! Esc quit; until T1b P, →, Home, M, E, V, H and F11; the four with Ctrl since E17 M6 T4b2a, the
//! user, 2026-11-04), the battle's the original's hotkeys by what is open over it: nothing (the
//! log's arrows, the Faerie Dragon's F, the panel's letters, F5 Creature Info's level since
//! T4b2a), the spell book, a
//! message box, an aim or the result window (each its own and the window's that work whatever is
//! open, Esc its close, Cancel, give-up or OK). Unattended, the keys do nothing but quit.
//!
//! The mouse (T6c-2, `crate::pointer`): the battle takes it as the original does
//! (`h3_view::mouse::track`): its cursor (the combat cursor's frame, the hardware cursor or drawn),
//! the stack it marks and those it enters fidgeting, its hints in the panel's log. The panel's
//! buttons show their click frame while the mouse holds them or their hotkeys are down (the
//! table's: S, R, O, A, C, W, D and Space; in the tactics phase its panel's Space and Enter
//! first, E17 M5 T4c2b); a replay takes no command, but the log's scroll buttons and the arrow
//! keys scroll it.
//!
//! Hot seat (E17 M5 T2b): `--human attacker|defender|both` names the sides a human plays here
//! (their players the battle's human ones, the bot playing the others). On a human's turn the
//! battle waits at its end, idling, until a left press over the battlefield gives its command
//! (`h3_view::mouse::click`, as the button goes down) or the panel's Wait or Defend is clicked
//! (the mouse's button released over it, or its hotkey released: `mouse::button`; in the tactics
//! phase its panel's Next Creature or Start Combat, E17 M5 T4c2b); input made
//! while the battle takes none (an action playing, the computer's turn) gives nothing. The screen
//! shows the side to act's hidden objects (`anim::Timeline::screen`). Unattended (`--check`,
//! `--measure`), the window presses a human's commands itself: the bot's choice, pressed where
//! the mouse gives it (a hero's cast through its book and aim; the tactics phase ended by Start
//! Combat, E17 M5 T4c2b), so that the battle is the bot's at once with the same setup, which the
//! run checks.
//!
//! The sound (T5b, [`crate::audio`]): a battle shown from its start opens as the original's does,
//! its start sound playing over the first frame while the battle's clock waits (at most 10 s),
//! then the music starts, looping and fading in, and the clock with it; the stacks' sounds follow
//! the timeline's cues. When the battle is decided (T8), every effect ends and its end's tune
//! replaces the music, once. `--mute` (or no sound device) plays none and does not wait.
//!
//! The battle's end (E17 M5 T4b1, [`crate::results`]): a battle fought here opens its result
//! window as it is decided, over the whole screen darkened, the battle standing behind it; its
//! OK (a click, Enter or Escape) closes the window's battle, as the original's leaves the battle's
//! screen.
//!
//! `--check <s>` and `--measure <s>` run unattended for that many seconds and report when each
//! frame showed against its time; `--check` also reads every frame back before presenting it and
//! compares it with the composed frame at the window's scale (the black around it included). An
//! adapter: the window is `h3-sdl`'s.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use h3_protocol::wire::{Command, Target};
use h3_render::screen::Screen;
use h3_sdl::{Config, CursorImage, Event, Key, ReadBack, Window};
use h3_ui::clock::Clock;
use h3_ui::cursor::Cursor;
use h3_ui::keys::Window as Own;
use h3_ui::ui::Item;
use h3_view::anim::Speed;
use h3_view::compose::FORMAT;
use h3_view::keys::{Command as Do, Context, Keys};
use h3_view::mouse::track::Input;
use h3_view::options::Settings;
use h3_view::scene::Draw;
use h3_view::shades::{Options as Shades, Toggle};
use h3_view::sound::Tune;

use crate::assets;
use crate::audio::{Library, Player};
use crate::battle::{Battle, BookDid, CursorLook, Ground, MessageDid, Press, Source, StatsDid};

/// How far past the last change the window looks for the next one before it waits for input
/// only (a battle that no longer changes).
const LOOK_AHEAD_MS: u32 = 600_000;

/// How much of the battle's idling is timed at once past its end.
const IDLE_SPAN_MS: u32 = 60_000;

/// The longest wait for input, so that the clock is read now and then.
const WAIT_MS: u32 = 250;

/// The longest wait for input while a seat on a server is played (E17 M5 T6b1, *provisional*):
/// what the server sent is taken at least this often.
const POLL_MS: u32 = 10;

/// How long after a human's turn started an unattended run presses its command (E17 M5 T2b):
/// a few idle steps show first.
const PRESS_AFTER_MS: u32 = 400;

/// The most pages and tabs an unattended run turns the spell book through to find a spell (E17
/// M5 T3b).
const BOOK_TURNS: usize = 64;

/// The user's file of keys looked for in the working directory (its `[battle]` table; E24 M3 T1b).
const KEYS_FILE: &str = "keys.toml";
/// The user's settings file in the working directory, beside the keys' (E17 M6 T4a,
/// *provisional*: the combat options dialog's settings, `h3_view::options::Settings`).
const SETTINGS_FILE: &str = "settings.toml";

/// The tunes a battle may end with (E17 M5 T1b: decoded before a battle fought as it is played
/// ends).
const TUNES: [Tune; 6] = [
    Tune::Win,
    Tune::Lose,
    Tune::DefendCastle,
    Tune::Retreat,
    Tune::Surrender,
    Tune::LoseCastle,
];

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
    /// Unattended, the human side leaves at its first turn it may by this panel button
    /// (Surrender or Retreat, through its question: E17 M5 T4a).
    leave: Option<u16>,
    /// Unattended, the human side switches Auto Combat on at its turn of this number (from 1;
    /// E17 M5 T4c1) and stops it by a press two commands later.
    auto: Option<usize>,
    /// The user's file of keys (E24 M3 T1b).
    keys: Option<PathBuf>,
    /// The user's settings file (E17 M6 T4a; none: `settings.toml` here).
    settings: Option<PathBuf>,
    /// `--speed` was given: it wins over the settings file's combat speed for this run.
    speed_set: bool,
}

/// The sides `name` names (`attacker`, `defender`, `both`).
fn sides(name: &str) -> Option<[bool; 2]> {
    match name {
        "attacker" => Some([true, false]),
        "defender" => Some([false, true]),
        "both" => Some([true, true]),
        _ => None,
    }
}

impl Options {
    /// Parses `(<scenario.toml | record.jsonl> | --server <ws://… | wss://…> (--game <id> |
    /// --create <offer>) --seat attacker|defender [--token <t>] [--name <display>]
    /// [--pins <file>] [--drop <n>] [--login <login> [--register <display>] [--accounts <file>]])
    /// [--seed <n>] [--speed <0-2>] [--packs <dir>]
    /// [--install <dir>] [--human attacker|defender|both] [--keys <file>] [--scale <n>] [--vsync]
    /// [--drawn-cursor] [--fullscreen] [--from <ms>] [--mute] [--check <s> | --measure <s>]
    /// [--leave retreat|surrender] [--auto <turn>]`.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let clients = assets::clients_dir();
        let env = std::env::var_os("H3_INSTALL").map(PathBuf::from);
        let (mut input, mut seed, mut speed, mut packs, mut install) = (None, 1, 0, None, None);
        let (mut scale, mut vsync, mut drawn_cursor, mut fullscreen) = (None, false, false, false);
        let (mut from, mut run, mut mute, mut humans) = (0, None, false, [false; 2]);
        let (mut leave, mut keys, mut auto) = (None, None, None);
        let (mut settings, mut speed_set) = (None, false);
        // A seat on a server (E17 M5 T6b1): `connect`'s options.
        let mut server: Vec<String> = Vec::new();
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
                "--speed" => {
                    speed = number(value()?)?.min(2);
                    speed_set = true;
                }
                "--packs" => packs = Some(PathBuf::from(value()?)),
                "--install" => install = Some(PathBuf::from(value()?)),
                "--settings" => settings = Some(PathBuf::from(value()?)),
                "--human" => {
                    let v = value()?;
                    humans = sides(&v).ok_or_else(|| format!("window: no sides `{v}`"))?;
                }
                "--keys" => keys = Some(PathBuf::from(value()?)),
                "--scale" => scale = Some(number(value()?)?.clamp(1, 16)),
                "--vsync" => vsync = true,
                "--drawn-cursor" => drawn_cursor = true,
                "--fullscreen" => fullscreen = true,
                "--from" => from = number(value()?)?,
                "--mute" => mute = true,
                "--check" => run = Some((number(value()?)?, true)),
                "--measure" => run = Some((number(value()?)?, false)),
                "--leave" => {
                    leave = Some(match value()?.as_str() {
                        "retreat" => h3_view::panel::RETREAT,
                        "surrender" => h3_view::panel::SURRENDER,
                        v => return Err(format!("window: no way to leave `{v}`")),
                    });
                }
                "--auto" => {
                    let turn = usize::try_from(number(value()?)?).unwrap_or(usize::MAX);
                    auto = Some(turn.max(1));
                }
                "--server" | "--game" | "--create" | "--seat" | "--token" | "--name" | "--pins"
                | "--drop" | "--login" | "--register" | "--accounts" => {
                    let v = value()?;
                    if arg == "--server" {
                        server.insert(0, v);
                    } else {
                        server.extend([arg.clone(), v]);
                    }
                }
                flag if flag.starts_with("--") => {
                    return Err(format!("window: unknown option `{flag}`"));
                }
                path if input.is_none() => input = Some(PathBuf::from(path)),
                extra => return Err(format!("window: unexpected `{extra}`")),
            }
        }
        let server = (!server.is_empty())
            .then(|| online_server(&server))
            .transpose()?;
        if server.is_some() && (input.is_some() || humans.contains(&true)) {
            return Err("window: a seat on a server, or a scenario or record, not both".into());
        }
        Ok(Self {
            source: Source {
                input: match (input, &server) {
                    (Some(input), _) => input,
                    (None, Some(_)) => PathBuf::new(),
                    (None, None) => return Err("window: needs a scenario or a record file".into()),
                },
                seed,
                speed: Speed::new(u8::try_from(speed).unwrap_or(2)),
                packs: packs.unwrap_or_else(|| clients.join("../packs")),
                install: install
                    .or(env)
                    .unwrap_or_else(|| clients.join("../original-data/install")),
                humans,
                server,
            },
            scale,
            vsync,
            drawn_cursor,
            fullscreen,
            from,
            mute,
            run,
            leave,
            auto,
            keys,
            settings,
            speed_set,
        })
    }
}

/// The note a seat on a server shows when the content check finds the game's content not this
/// client's own (E17 M5 T6b1, *provisional*: the window's title): none when it is the same.
const fn content_note(content: h3_view::remote::Content) -> Option<&'static str> {
    match content {
        h3_view::remote::Content::Same => None,
        h3_view::remote::Content::Different => {
            Some("the server's rules are not this client's packs: no previews")
        }
        // The battle does not load without them (`Ground::load`).
        h3_view::remote::Content::Absent => Some("the game's packs are not here"),
    }
}

/// The seat on a server `args` name (`--server`'s address first, then `connect`'s other options;
/// E17 M5 T6b1): an attacker's or a defender's seat.
fn online_server(args: &[String]) -> Result<crate::online::Server, String> {
    let options =
        crate::connect::Options::parse(args).map_err(|e| e.replacen("connect", "window", 1))?;
    if options.seat == h3_protocol::wire::Seat::Spectator {
        return Err("window: a seat on a server is an attacker's or a defender's".into());
    }
    Ok(crate::online::Server {
        address: options.address,
        game: options.game,
        seat: options.seat,
        token: options.token,
        name: options.name,
        pins: options.pins,
        drop: options.drop,
        account: options.account,
    })
}

/// The table of keys: the user's file (`--keys`, else `keys.toml` here if there is one; its
/// `[battle]` table) over the defaults, every key it binds one the adapter knows; and where it
/// came from (E24 M3 T1b, as the map window's).
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

/// The combat options' settings (E17 M6 T4a): the user's file (`--settings`, else `settings.toml`
/// here if there is one; its `[battle]` table, `h3_view::options::Settings::read`), else a first
/// run's; and the file the dialog writes them to as it closes after a click (the one named, else
/// `settings.toml` here).
fn load_settings(path: Option<&Path>) -> Result<(Settings, PathBuf, String), String> {
    let here = Path::new(SETTINGS_FILE);
    let read = path.or_else(|| here.exists().then_some(here));
    let text = read
        .map(|p| fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display())))
        .transpose()?;
    let settings = Settings::read(text.as_deref()).map_err(|e| {
        let from = read.map_or(String::new(), |p| p.display().to_string());
        format!("{from}: {e}")
    })?;
    let from = read.map_or_else(|| "a first run's".to_owned(), |p| p.display().to_string());
    Ok((settings, path.unwrap_or(here).to_path_buf(), from))
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

/// The times at which the battle may change, timed as far as needed: a battle still played is
/// played on before a change at or past its timeline's end is given (E17 M5 T1b: what comes then
/// is the next command's).
#[derive(Debug)]
struct Changes {
    times: Vec<u32>,
    until: u32,
}

impl Changes {
    fn new(battle: &Battle<'_>) -> Self {
        let until = battle.timeline().end;
        Self {
            times: battle.timeline().changes(until),
            until,
        }
    }

    /// The times again after the battle grew.
    fn grown(&mut self, battle: &Battle<'_>) {
        self.until = self.until.max(battle.timeline().end);
        self.times = battle.timeline().changes(self.until);
    }

    /// Times at least up to `t`, the battle played on while its timeline ends by then.
    fn reach(&mut self, battle: &mut Battle<'_>, t: u32) -> Result<(), String> {
        if battle.grow(t)? {
            self.grown(battle);
        }
        while self.until < t {
            self.until = self.until.saturating_add(IDLE_SPAN_MS);
            self.times = battle.timeline().changes(self.until);
        }
        Ok(())
    }

    /// The latest change at or before `t`.
    fn latest(&mut self, battle: &mut Battle<'_>, t: u32) -> Result<u32, String> {
        self.reach(battle, t)?;
        let i = self.times.partition_point(|&c| c <= t);
        Ok(self.times.get(i.wrapping_sub(1)).copied().unwrap_or(0))
    }

    /// The first change after `t` (none within [`LOOK_AHEAD_MS`]).
    fn after(&mut self, battle: &mut Battle<'_>, t: u32) -> Result<Option<u32>, String> {
        let limit = t.saturating_add(LOOK_AHEAD_MS);
        loop {
            let i = self.times.partition_point(|&c| c <= t);
            let next = self.times.get(i).copied();
            if battle.playing() && next.is_none_or(|n| n >= battle.timeline().end) {
                battle.grow_one()?;
                self.grown(battle);
                continue;
            }
            if next.is_some() {
                return Ok(next);
            }
            if self.until >= limit {
                return Ok(None);
            }
            let more = self.until.saturating_add(1);
            self.reach(battle, more)?;
        }
    }
}

/// The frame of the change at `t`.
fn frame_at(battle: &mut Battle<'_>, t: u32) -> Result<Frame, String> {
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
    battle: &mut Battle<'_>,
    changes: &mut Changes,
    shown: &Frame,
) -> Result<Option<Frame>, String> {
    // Behind the open spell book, a message box or the combat options dialog the battle stands
    // still (E17 M5 T3a, T4a; M6 T4a):
    // only an input changes it; the result window's clip plays on, a frame at each of its
    // decodes (E17 M5 T4b2b).
    if battle.book_open() || battle.message_open() || battle.options_open() {
        return match battle.clip_times(shown.t) {
            Some((_, Some(next))) => frame_at(battle, next).map(Some),
            _ => Ok(None),
        };
    }
    // Behind a stack's stats dialog too; its creature steps each 100 ms (E17 M6 T2a2a), none
    // while its right button's popup is held (E17 M6 T2a2b1: only an input changes it then).
    if battle.stats_open() {
        return match battle.stats_step_after(shown.t) {
            Some(next) => frame_at(battle, next).map(Some),
            None => Ok(None),
        };
    }
    let mut t = shown.t;
    while let Some(next) = changes.after(battle, t)? {
        // Past the end of a still battle only the idle steps come, each drawing the same.
        if !battle.playing() && next > battle.timeline().end && battle.timeline().still() {
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
    let (keys, keys_from) = load_keys(options.keys.as_deref())?;
    // The combat options' settings (E17 M6 T4a): the file's combat speed unless `--speed` gave
    // one.
    let (settings, settings_path, settings_from) = load_settings(options.settings.as_deref())?;
    let source = &Source {
        speed: if options.speed_set {
            options.source.speed
        } else {
            Speed::new(settings.speed)
        },
        ..options.source.clone()
    };
    let ground = Ground::load(source)?;
    let mut battle = Battle::new(&ground)?;
    // The checkpoint batch: the cursor now, each frame's names as it is first drawn.
    // The spell book's default cursor too (E17 M5 T3a).
    let combat = battle.layout().mouse.cursor.clone();
    let book_cursor = battle.layout().book.cursor.clone();
    // The spell cursor too (E17 M5 T3b).
    let spell_cursor = battle.layout().mouse.spell_cursor.clone();
    battle.load_batch(std::iter::empty(), &[&combat, &book_cursor, &spell_cursor])?;
    // The combat cursor's frames as the mouse manager draws them (E17 M3 T6c-2), made as shown.
    let mut cursors = Cursors::default();
    let mut frame_now = battle.cursor(options.from);
    let mut cursor_visible = true;
    let (_, hardware) = cursors.get(&battle, &frame_now)?;
    let [width, height] = battle.frame_size();
    // A seat on a server: the content check's note in the title (E17 M5 T6b1).
    let note = ground.content().and_then(content_note);
    if let Some(note) = note {
        lines.push(format!("note: {note}"));
    }
    let mut win = Window::open(&Config {
        title: note.map_or_else(
            || format!("h3-view — {}", source.stem()),
            |note| format!("h3-view — {} — {note}", source.stem()),
        ),
        width,
        height,
        scale: options.scale,
        vsync: options.vsync,
        fullscreen: options.fullscreen,
    })?;
    let mut drawn_cursor = options.drawn_cursor;
    if !drawn_cursor && let Err(e) = win.set_cursor(Some(hardware)) {
        lines.push(format!("hardware cursor: {e}; drawn instead"));
        drawn_cursor = true;
    }

    // The sound: decoded now, the device opened; none with --mute or without a device.
    let seed = u32::try_from(source.seed & u64::from(u32::MAX)).unwrap_or(1);
    let opening = h3_view::sound::opening(seed);
    let library = if options.mute {
        None
    } else {
        // The stacks' sounds and the battle's own its cues name (the heal's, E17 M3 T3b; the
        // casts', T7b-1).
        let sounds = (battle.stack_sounds(), battle.battle_sounds());
        Some(Library::load(
            &source.install,
            (opening, &battle.layout().options.sound),
            sounds,
            &TUNES,
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
    // With its sound playing, the battle waits for its samples as the original does on a
    // computer with sound (E17 M5 T5b2, design §8.3): made again with their lengths.
    if let (Some(_), Some(lib)) = (player.as_ref(), library.as_ref()) {
        ground.play_sounds(lib.lengths());
        battle = Battle::new(&ground)?;
        battle.load_batch(std::iter::empty(), &[&combat, &book_cursor, &spell_cursor])?;
    }
    // The settings the combat options dialog shows, against the audio device opened or not (its
    // no-sound box; E17 M6 T4a); its volumes on the sound.
    battle.settings = settings;
    battle.audio = player.is_some();
    if let Some(p) = player.as_mut() {
        p.set_volumes(settings.music, settings.effects);
    }
    let unsaved = std::cell::RefCell::new(Vec::new());
    let files = SettingsFile {
        path: &settings_path,
        library: library.as_ref(),
        lines: &unsaved,
    };
    // A seat on a server starts where its record stands (E17 M5 T6b2, *provisional*): what came
    // before the window joined is not played again.
    let start = if ground.online().is_some() {
        options.from.max(battle.timeline().end)
    } else {
        options.from
    };
    let mut cue = battle.timeline().cue_from(start);
    // Whether the battle's end's tune started (E17 M3 T8).
    let mut ended = false;

    let mut changes = Changes::new(&battle);
    let mut clock = Clock::new(win.ticks_ns(), start);
    // The clock is held under a notice's box (E17 M5 T5a).
    let mut notice_held = false;
    // The opening: the clock waits while the start sound plays (from the battle's start only).
    let mut opening_until = None;
    if let (Some(p), Some(lib)) = (player.as_mut(), library.as_ref()) {
        let now = win.ticks_ns();
        let wait = if start == 0 { p.start(lib) } else { 0 };
        if wait > 0 {
            clock.toggle(now);
            opening_until = Some(now + u64::from(wait) * 1_000_000);
        } else {
            p.start_music(now);
        }
    }
    let mut shown: Option<Frame> = None;
    let from = changes.latest(&mut battle, start)?;
    let mut next = Some(frame_at(&mut battle, from)?);
    let mut pointer: Option<(i32, i32)> = None;
    // The mouse's moves, the log's scrolls and the button held (E17 M3 T6c-2). Creature Info's
    // level from the settings at the opening (E17 M6 T4b2a), kept by every Restart, and the level
    // the hints read now: a change (F5, the options dialog) is an input at its time.
    let mut inputs: Vec<Input> = vec![
        Input::ArmyInfo {
            at: 0,
            level: battle.settings.army_info,
        },
        // The grid's and the shadows' options from the settings at the opening (E17 M6 T4c), the
        // background composed with them.
        Input::Shades {
            at: 0,
            options: Shades::of(&battle.settings),
            keyed: false,
        },
    ];
    let mut army_info = battle.settings.army_info;
    // The options dialog open as the last pass ended, and the spell aimed then with whether its
    // aim turned the movement shadow off (E17 M6 T4c, design §12.16).
    let mut options_open = false;
    let mut aimed: Option<u16> = None;
    let mut aim_shades = false;
    let mut scrolls: Vec<(u32, i64)> = Vec::new();
    let mut held: Option<Held> = None;
    // A battle's own key that stopped Auto Combat, and the turn the stop comes at (E17 M5 T4c1;
    // its index, T8b2 R5c-4): it acts on that turn if a human's (0x474625).
    let mut stopped_by: Option<(Do, usize)> = None;
    // Either Control key held (E24 M3 T1b: a chord's Ctrl+).
    let mut ctrl = [false; 2];
    let mut report = Report::default();
    // An unattended run presses a human's commands itself (E17 M5 T2b).
    let plays = source.humans.contains(&true) || ground.content().is_some();
    let mut presser = (options.run.is_some() && plays).then(|| Presser {
        points: crate::pointer::press_points(battle.layout()),
        leave: options.leave,
        auto: options.auto,
        ..Presser::default()
    });
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
            })
            .min(if battle.online() { POLL_MS } else { WAIT_MS });
        let mut redraw = false;
        let mut mouse_changed = false;
        // The mouse followed in the pass already (an aim's press, E17 M5 T8b2: R4b-1), whether
        // what it shows changed then.
        let mut followed = false;
        let mut grew = false;
        // The spell book opened, closed or changed (E17 M5 T3a).
        let mut book_changed = false;
        // The box open as the pass starts, to tell one that opens in it (E17 M5 T8b2).
        let box_before = box_shown(&battle);
        // The result window opens at the battle's end (E17 M5 T4b1).
        let ms = clock.ms(win.ticks_ns());
        if battle.open_result(ms) {
            lines.push(format!("the result window opened at {ms} ms"));
            book_changed = true;
        }
        // A notice opens at its time, the clock held there until its box closes (E17 M5 T5a).
        if let Some(t) = battle.open_notice(ms) {
            lines.push(format!("a message box opened at {t} ms"));
            let now = win.ticks_ns();
            clock.seek(now, t);
            if !clock.paused() {
                clock.toggle(now);
                notice_held = true;
            }
            book_changed = true;
        }
        if notice_held && !battle.message_open() {
            clock.toggle(win.ticks_ns());
            notice_held = false;
        }
        // A seat on a server: what the server sent since (E17 M5 T6b1), never waited on.
        let ticks = win.ticks_ns();
        grew |= battle.poll(clock.ms(ticks), ticks / 1_000_000)?;
        // A restarted server's record opened afresh (E17 M5 T6b2): the battle and the window's
        // clock, inputs and presses start again from its new opening, a new timeline.
        if battle.opened_afresh() {
            battle = battle.afresh()?;
            battle.load_batch(std::iter::empty(), &[&combat, &book_cursor, &spell_cursor])?;
            lines.push(format!(
                "a new timeline at {} ms of the one before",
                clock.ms(win.ticks_ns())
            ));
            let start = battle.timeline().end;
            clock = Clock::new(win.ticks_ns(), start);
            (notice_held, ended, held, stopped_by) = (false, false, None, None);
            inputs.clear();
            scrolls.clear();
            changes = Changes::new(&battle);
            cue = battle.timeline().cue_from(start);
            let from = changes.latest(&mut battle, start)?;
            next = Some(frame_at(&mut battle, from)?);
            shown = None;
            if let Some(p) = presser.as_mut() {
                *p = Presser {
                    points: core::mem::take(&mut p.points),
                    leave: options.leave,
                    auto: options.auto,
                    ..Presser::default()
                };
            }
            continue 'run;
        }
        if let Some(p) = presser.as_mut() {
            let ticks = win.ticks_ns();
            grew |= p.press(&mut battle, &mut inputs, clock.ms(ticks), ticks / 1_000_000)?;
        }
        // The key that stopped Auto Combat acts on the turn it stops at, a human's: there the
        // battle hands it to its key handler (0x474625; E17 M5 T4c1); a computer's drops it. A
        // human's turn Auto Combat does not play reads no stop (E17 M6 T4b1): its start drops
        // the key, as with Auto Combat off (CLT-Q45 (2): the original's acts there).
        if let Some((command, turn)) = stopped_by
            && let Some(&(start, _)) = battle.timeline().turns.get(turn - 1)
        {
            let last = battle.timeline().turns.len();
            let at_start = || battle.auto_at_start(turn);
            let carry = carry((battle.waiting(), last, turn), at_start);
            if carry != Carry::Wait {
                stopped_by = None;
            }
            if carry == Carry::Act {
                match command {
                    Do::LogUp | Do::LogDown => {
                        scrolls.push((start, if command == Do::LogUp { -1 } else { 1 }));
                    }
                    // F5 both stopped Auto Combat and cycles the level there (E17 M6 T4b2a); F6 to
                    // F8 toggle their options there (E17 M6 T4c).
                    Do::ArmyInfo => {
                        let at = clock.ms(win.ticks_ns()).max(start);
                        army_info = cycle_army_info(&mut battle, &mut inputs, at, &files);
                    }
                    Do::Shades(toggle) => {
                        let at = clock.ms(win.ticks_ns()).max(start);
                        toggle_shades(&mut battle, &mut inputs, (at, toggle), &files);
                    }
                    _ => {
                        let at = clock.ms(win.ticks_ns()).max(start);
                        // F over the battlefield hides the army info windows first (CLT-Q49,
                        // E17 M6 T4b2b1).
                        if let Some(hide) = key_hides(&battle, at, pointer) {
                            inputs.push(hide);
                        }
                        if let Some(spell) = battle.dragon_aim(at) {
                            inputs.push(Input::Aim {
                                at,
                                spell,
                                creature: true,
                            });
                        }
                    }
                }
                mouse_changed = true;
            }
        }
        // The input queue is emptied as a box opens (the modal loop 0x602520; design §7.19): the
        // presses and key-downs that come with the input that opened it, or after the box opened
        // in this pass, are dropped; the key-ups (no key stays held), the moves and Quit go on.
        let mut last_box = box_before;
        let mut emptied = false;
        for event in win.wait(timeout) {
            let shown_box = box_shown(&battle);
            emptied |= shown_box.is_some() && shown_box != last_box;
            last_box = shown_box;
            if emptied && matches!(event, Event::Key(_) | Event::Click { down: true, .. }) {
                continue;
            }
            let now = win.ticks_ns();
            let at = clock.ms(now);
            match event {
                Event::Quit => break 'run,
                // Control held with a key makes another chord (E24 M3 T1b). Itself a key down, it
                // stops Auto Combat as any key does (T8b2, R5c-5; design §7.22).
                Event::Key(k @ (Key::LeftCtrl | Key::RightCtrl)) => {
                    ctrl[usize::from(k == Key::RightCtrl)] = true;
                    let context = Context::of(
                        battle.result_open(),
                        battle.message_open(),
                        (battle.stats_open(), battle.options_open()),
                        battle.book_open(),
                        battle.aiming(),
                    );
                    if options.run.is_none()
                        && context == Context::Battle
                        && battle.stop_auto(at).is_some()
                    {
                        mouse_changed = true;
                    }
                }
                Event::KeyUp(k @ (Key::LeftCtrl | Key::RightCtrl)) => {
                    ctrl[usize::from(k == Key::RightCtrl)] = false;
                }
                // Unattended, the keys do nothing but quit, whatever is open (the mouse still
                // moves the battle's; a human's commands are the presser's).
                Event::Key(key) if options.run.is_some() => {
                    let command = keys.command(Context::Battle, ctrl.contains(&true), key.name());
                    if command == Some(Do::Window(Own::Quit)) {
                        break 'run;
                    }
                }
                Event::KeyUp(_) if options.run.is_some() => {}
                // A turret's info held by the right button over the battle (E17 M6 T2b2, design
                // §12.6), a control's help over the panel (T3a, §12.7) or a hero's info window
                // (T3b1, §12.8; `popup_run_held` 0x602A40): its ends (the right button's release, a left press or release)
                // close it; every other input is dropped, the moves too (the battle takes none
                // meanwhile, nor after: nothing is posted, 0x476BBF).
                Event::KeyUp(Key::RightButton) | Event::Click { .. } if battle.popup_held() => {
                    book_changed |= battle.popup_end();
                }
                Event::Key(_) | Event::KeyUp(_) if battle.popup_held() => {}
                Event::Pointer(p) if battle.popup_held() => pointer = p,
                // The stats dialog's right button's description (E17 M6 T2a2b1, design §12.3),
                // held while the button is: the popup's loop (0x602A40) takes only its ends (the
                // right button's release, a left press or release) and drops every other input
                // (a key's release lets the OK it holds go in the client's profile, CLT-Q39); the
                // battle keeps where the mouse is.
                // The right button's stats dialog over the battle (E17 M6 T2b1) the same: its
                // end closes it, the battle taking the mouse again (`Input::Looked`).
                Event::KeyUp(Key::RightButton) if battle.stats_popup() => {
                    let did = battle.stats_held(at, true, None, false);
                    book_changed |= stats_did(&mut battle, &mut inputs, at, did);
                }
                Event::Click { down, .. } if battle.stats_popup() => {
                    let did = battle.stats_held(at, true, None, !down);
                    book_changed |= stats_did(&mut battle, &mut inputs, at, did);
                }
                Event::KeyUp(key) if battle.stats_popup() => {
                    let did = battle.stats_held(at, false, Some(key.name()), false);
                    book_changed |= stats_did(&mut battle, &mut inputs, at, did);
                }
                Event::Key(_) if battle.stats_popup() => {}
                Event::Pointer(p) if battle.stats_popup() => {
                    pointer = p;
                    if let Some((x, y)) = p {
                        inputs.push(Input::Move { at, to: [x, y] });
                        mouse_changed = true;
                    }
                }
                Event::Key(Key::RightButton) if battle.stats_open() => {
                    let point = pointer.map_or(crate::pointer::PARK, |(x, y)| [x, y]);
                    book_changed |= battle.stats_right(at, point);
                }
                // The combat options dialog's help (E17 M6 T4a, design §12.12): the right button
                // over a control shows it while held, as the panel's controls' help.
                Event::Key(Key::RightButton) if battle.options_open() && !battle.message_open() => {
                    let point = pointer.map_or(crate::pointer::PARK, |(x, y)| [x, y]);
                    book_changed |= battle.options_right(point);
                }
                // What a key does where the battle takes it now (E24 M3 T1b, `h3_view::keys`): a
                // dialog or an aim over the battle takes its own keys and the window's that work
                // whatever is open; its Esc is its close, Cancel or give-up, never the window's
                // quit.
                Event::Key(key) => {
                    let context = Context::of(
                        battle.result_open(),
                        battle.message_open(),
                        (battle.stats_open(), battle.options_open()),
                        battle.book_open(),
                        battle.aiming(),
                    );
                    let ctrl_held = ctrl.contains(&true);
                    // The tactics phase's panel's keys first, then the battle's (E17 M5 T4c2b).
                    let command = (context == Context::Battle && battle.in_tactics(at))
                        .then(|| keys.command(Context::Tactics, ctrl_held, key.name()))
                        .flatten()
                        .or_else(|| keys.command(context, ctrl_held, key.name()));
                    // A key while Auto Combat plays stops it as the next turn starts and does
                    // nothing else (E17 M5 T4c1, 0x4741F9); the window's own keys stay its own.
                    if context == Context::Battle
                        && !matches!(command, Some(Do::Window(_)))
                        && let Some(turn) = battle.stop_auto(at)
                    {
                        stopped_by = command.filter(|&c| carried(c)).map(|c| (c, turn));
                        mouse_changed = true;
                        continue;
                    }
                    // The right button over the battlefield (E17 M6 T2b1, design §12.5;
                    // `battle_right_click` 0x4769C0): the stack on the hex there shows its stats
                    // dialog while the button is held, the combat cursor's pointer set first;
                    // over the panel a control's help (T3a, `battle_control_help` 0x472A50).
                    if command.is_none()
                        && matches!(key, Key::RightButton)
                        && context == Context::Battle
                    {
                        let point = pointer.map_or(crate::pointer::PARK, |(x, y)| [x, y]);
                        if battle.look_held(at, point) {
                            inputs.push(Input::Look { at });
                            book_changed = true;
                        } else if let Some(side) = battle.hero_info(at, point) {
                            // Else a hero's info window over its rectangle (T3b1, design §12.8),
                            // held as the stats dialog's popup; at Creature Info's levels 1 and 2
                            // it stays shown after the release (E17 M6 T4b2b1, §12.15).
                            inputs.push(Input::HeroInfo { at, side });
                            (book_changed, mouse_changed) = (true, true);
                        } else {
                            // Else a turret's info held (E17 M6 T2b2, design §12.6), or over the
                            // panel a control's help (T3a, design §12.7; `battle_buttons`
                            // 0x474DE4): each held as the stats dialog's popup.
                            book_changed |= battle.turret_info(at, point, true)
                                || battle.control_help(at, point);
                        }
                        continue;
                    }
                    let Some(command) = command else {
                        continue;
                    };
                    match command {
                        Do::Window(own) => {
                            // A key that moves the battle ends the opening first: the clock
                            // runs, the music starts (pause does nothing more).
                            if matches!(own, Own::Pause | Own::Frame | Own::Restart)
                                && opening_until.take().is_some()
                            {
                                clock.toggle(now);
                                if let Some(p) = player.as_mut() {
                                    p.start_music(now);
                                }
                                if own == Own::Pause {
                                    continue;
                                }
                            }
                            match own {
                                Own::Quit => break 'run,
                                Own::Pause => {
                                    clock.toggle(now);
                                    if let Some(p) = player.as_mut() {
                                        p.pause(clock.paused(), now);
                                    }
                                }
                                Own::Frame if clock.paused() => {
                                    if let Some(f) = &next {
                                        // The sounds of the frames stepped over are dropped; the
                                        // next frame's, cued as this one shows, play when the
                                        // battle runs again.
                                        clock.seek(now, f.t);
                                        cue = battle.timeline().cue_from(f.t);
                                        if let Some(p) = player.as_mut() {
                                            p.silence();
                                        }
                                    }
                                }
                                // A frame on while running: nothing.
                                Own::Frame => {}
                                Own::Restart => {
                                    clock.seek(now, options.from);
                                    // The inputs and scrolls after it are gone (design
                                    // §9.10; E17 M3 T9b, the reviewer: they played again at
                                    // their times); Auto Combat's switches and stops stay the
                                    // battle's, and a key that stopped it still acts on the
                                    // turn it stops at (T8b2, R5c-4).
                                    inputs.retain(|i| i.at() <= options.from);
                                    // Creature Info's level stays the settings' (E17 M6 T4b2a),
                                    // the grid's and the shadows' options too (E17 M6 T4c).
                                    inputs.push(Input::ArmyInfo {
                                        at: options.from,
                                        level: army_info,
                                    });
                                    inputs.push(Input::Shades {
                                        at: options.from,
                                        options: Shades::of(&battle.settings),
                                        keyed: false,
                                    });
                                    scrolls.retain(|&(t, _)| t <= options.from);
                                    mouse_changed = true;
                                    shown = None;
                                    let from = changes.latest(&mut battle, options.from)?;
                                    next = Some(frame_at(&mut battle, from)?);
                                    cue = battle.timeline().cue_from(options.from);
                                    if let Some(p) = player.as_mut() {
                                        p.silence();
                                        // The end's tune gives way to the battle's music again.
                                        if ended {
                                            p.start_music(now);
                                        }
                                    }
                                    ended = false;
                                }
                                Own::Music => {
                                    if let Some(p) = player.as_mut() {
                                        p.music_on = !p.music_on;
                                    }
                                }
                                Own::Effects => {
                                    if let Some(p) = player.as_mut() {
                                        p.effects_on = !p.effects_on;
                                        if !p.effects_on {
                                            p.silence();
                                        }
                                    }
                                }
                                Own::Fullscreen => {
                                    let on = !win.fullscreen();
                                    win.set_fullscreen(on);
                                }
                                Own::Vsync => {
                                    let on = !win.vsync();
                                    win.set_vsync(on);
                                }
                                Own::Cursor => {
                                    drawn_cursor = !drawn_cursor;
                                    let image = if drawn_cursor {
                                        None
                                    } else {
                                        Some(cursors.get(&battle, &frame_now)?.1)
                                    };
                                    if win.set_cursor(image).is_err() {
                                        drawn_cursor = true;
                                    }
                                    redraw = true;
                                }
                            }
                        }
                        // A message box over the battle (and the book; E17 M5 T4a): its button
                        // down, its key's release clicking it.
                        Do::Message(k) => {
                            let did = battle.message_key(k, key.name());
                            grew |=
                                message_did(&mut battle, &mut inputs, at, did, &mut book_changed)?;
                        }
                        // The result window's OK (E17 M5 T4b1): down, its key's release closing
                        // the battle's screen.
                        Do::ResultOk => {
                            let did = battle.message_key(h3_view::message::Key::Ok, key.name());
                            grew |=
                                message_did(&mut battle, &mut inputs, at, did, &mut book_changed)?;
                        }
                        // The combat options dialog's OK down, its key's release closing it (E17
                        // M6 T4a).
                        Do::OptionsOk => {
                            let did = battle.options_key(key.name());
                            book_changed |= options_did(&mut battle, &mut player, did, &files);
                        }
                        // A stack's stats dialog's OK down, its key's release closing it (E17
                        // M6 T2a2a).
                        Do::StatsOk => {
                            book_changed |= battle.stats_key(key.name()) != StatsDid::Nothing;
                        }
                        // The acting stack's stats dialog (E17 M6 T2a2a), the mouse where it is;
                        // the army info windows hidden first (0x475D1E; E17 M6 T4b2b1).
                        Do::Stats => {
                            let point = pointer.map_or(crate::pointer::PARK, |(x, y)| [x, y]);
                            if battle.look(at, point, true) {
                                inputs.push(Input::Hide { at, key: false });
                                inputs.push(Input::Look { at });
                                book_changed = true;
                            }
                        }
                        // The spell book open over the battle (E17 M5 T3a).
                        Do::Book(k) => {
                            let did = battle.book_key(k);
                            grew |= book_did(&mut battle, &mut inputs, at, did, &mut book_changed)?;
                        }
                        // A spell aimed given up (E17 M5 T3b; the aim's loop 0x5A3070).
                        Do::AimCancel => {
                            battle.cancel_aim();
                            inputs.push(Input::Cancel { at });
                            // The stats dialog's close after its cast button's aim (E17 M6
                            // T2a2b1).
                            if battle.looked_after_aim() {
                                inputs.push(Input::Looked { at });
                            }
                            mouse_changed = true;
                        }
                        // Creature Info's level (E17 M6 T4b2a, design §12.14; `battle_buttons`'
                        // case 63, no test of the turn but only where the battle takes its keys):
                        // cycled 0, 1, 2, no hint written (the line stays until the mouse takes
                        // another hex), the frame drawn again and the settings written at once.
                        Do::ArmyInfo if !battle.takes(at) => {}
                        Do::ArmyInfo => {
                            army_info = cycle_army_info(&mut battle, &mut inputs, at, &files);
                            book_changed = true;
                        }
                        // The grid and the shadows (E17 M6 T4c, design §12.16; `battle_buttons`'
                        // F6 to F8, no test of the turn but only where the battle takes its
                        // keys): toggled, the frame drawn again (on a human's turn the mouse's
                        // hex shaded again), the settings written at once.
                        Do::Shades(_) if !battle.takes(at) => {}
                        Do::Shades(toggle) => {
                            toggle_shades(&mut battle, &mut inputs, (at, toggle), &files);
                            (book_changed, mouse_changed) = (true, true);
                        }
                        // One of the battle's keys made while a battle fought here takes no
                        // input is dropped (design §9.11; E17 M5 T2b).
                        Do::LogUp | Do::LogDown if !battle.takes(at) => {}
                        Do::LogUp | Do::LogDown => {
                            scrolls.push((at, if command == Do::LogUp { -1 } else { 1 }));
                            mouse_changed = true;
                        }
                        // The Faerie Dragon's key aims its drawn spell (E17 M5 T3b); with the mouse
                        // over the battlefield the client's profile hides the army info windows
                        // first (CLT-Q49, E17 M6 T4b2b1).
                        Do::DragonSpell => {
                            if let Some(hide) = key_hides(&battle, at, pointer) {
                                inputs.push(hide);
                                mouse_changed = true;
                            }
                            if let Some(spell) = battle.dragon_aim(at) {
                                inputs.push(Input::Aim {
                                    at,
                                    spell,
                                    creature: true,
                                });
                                mouse_changed = true;
                            }
                        }
                        // A panel button's hotkey holds it down; its release clicks (below).
                        Do::Button(id) => {
                            if battle.takes(at) {
                                held = Some(Held {
                                    id,
                                    key: Some(key),
                                    over: true,
                                });
                                mouse_changed = true;
                            }
                        }
                    }
                }
                Event::KeyUp(key) if battle.message_open() => {
                    let did = battle.message_key_up(key.name());
                    grew |= message_did(&mut battle, &mut inputs, at, did, &mut book_changed)?;
                }
                Event::KeyUp(key) if battle.stats_open() => {
                    let did = battle.stats_key_up(key.name());
                    let changed = stats_did(&mut battle, &mut inputs, at, did);
                    (book_changed, mouse_changed) =
                        (book_changed | changed, mouse_changed | changed);
                }
                Event::KeyUp(key) if battle.options_open() => {
                    let did = battle.options_key_up(key.name());
                    let changed = options_did(&mut battle, &mut player, did, &files);
                    (book_changed, mouse_changed) =
                        (book_changed | changed, mouse_changed | changed);
                }
                Event::KeyUp(_) if battle.book_open() || battle.aiming() => {}
                Event::KeyUp(key) => {
                    if let Some(h) = held.take_if(|h| h.key == Some(key)) {
                        // The keys C, S, R and O over the battlefield hide the army info windows
                        // first in the client's profile (CLT-Q49, E17 M6 T4b2b1).
                        let keyed = [
                            h3_view::panel::SPELL_BOOK,
                            h3_view::panel::SURRENDER,
                            h3_view::panel::RETREAT,
                            h3_view::panel::OPTIONS,
                        ];
                        if keyed.contains(&h.id)
                            && let Some(hide) = key_hides(&battle, at, pointer)
                        {
                            inputs.push(hide);
                        }
                        // A hotkey's release clicks its button (0x45659F).
                        if let Some(command) = battle.button(at, h.id) {
                            grew |= give(&mut battle, &mut inputs, at, command)?;
                        }
                        // A switches Auto Combat on (E17 M5 T4c1).
                        if h.id == h3_view::panel::AUTO_COMBAT {
                            grew |= battle.switch_auto(at)?;
                        }
                        // C opens the spell book (E17 M5 T3a).
                        if h.id == h3_view::panel::SPELL_BOOK && battle.open_book(at) {
                            inputs.push(Input::Book { at });
                            book_changed = true;
                        }
                        // O opens the combat options dialog (E17 M6 T4a).
                        book_changed |= h.id == h3_view::panel::OPTIONS && battle.open_options(at);
                        // S and R ask first (E17 M5 T4a).
                        book_changed |= battle.ask_leave(at, h.id);
                        mouse_changed = true;
                    }
                }
                Event::Pointer(p) if battle.message_open() => {
                    pointer = p;
                    redraw |= drawn_cursor;
                    if let Some((x, y)) = p {
                        book_changed |= battle.message_moved([x, y]);
                    }
                }
                Event::Click {
                    down: true,
                    at: (x, y),
                } if battle.message_open() => {
                    let did = battle.message_press([x, y]);
                    grew |= message_did(&mut battle, &mut inputs, at, did, &mut book_changed)?;
                }
                Event::Click { down: false, .. } if battle.message_open() => {
                    let did = battle.message_release();
                    grew |= message_did(&mut battle, &mut inputs, at, did, &mut book_changed)?;
                }
                // A stack's stats dialog over the battle (E17 M6 T2a2a): its moves, its OK.
                Event::Pointer(p) if battle.stats_open() => {
                    pointer = p;
                    redraw |= drawn_cursor;
                    if let Some((x, y)) = p {
                        book_changed |= battle.stats_moved([x, y]);
                        // The battle keeps where the mouse is, to take it as the dialog closes.
                        inputs.push(Input::Move { at, to: [x, y] });
                        mouse_changed = true;
                    }
                }
                Event::Click {
                    down: true,
                    at: (x, y),
                } if battle.stats_open() => {
                    book_changed |= battle.stats_press([x, y]) != StatsDid::Nothing;
                }
                Event::Click { down: false, .. } if battle.stats_open() => {
                    let did = battle.stats_release();
                    let changed = stats_did(&mut battle, &mut inputs, at, did);
                    (book_changed, mouse_changed) =
                        (book_changed | changed, mouse_changed | changed);
                }
                // The combat options dialog over the battle (E17 M6 T4a): its moves (the button
                // held follows; the battle takes none), its presses and releases.
                Event::Pointer(p) if battle.options_open() => {
                    pointer = p;
                    redraw |= drawn_cursor;
                    if let Some((x, y)) = p {
                        let did = battle.options_moved([x, y]);
                        book_changed |= options_did(&mut battle, &mut player, did, &files);
                    }
                }
                Event::Click {
                    down: true,
                    at: (x, y),
                } if battle.options_open() => {
                    let did = battle.options_press([x, y]);
                    book_changed |= options_did(&mut battle, &mut player, did, &files);
                }
                Event::Click { down: false, .. } if battle.options_open() => {
                    let did = battle.options_release();
                    let changed = options_did(&mut battle, &mut player, did, &files);
                    (book_changed, mouse_changed) =
                        (book_changed | changed, mouse_changed | changed);
                }
                Event::Pointer(p) if battle.book_open() => {
                    pointer = p;
                    redraw |= drawn_cursor;
                    if let Some((x, y)) = p {
                        book_changed |= battle.book_hover([x, y]);
                        // The battle keeps where the mouse is, to aim from there (E17 M5 T3b).
                        inputs.push(Input::Move { at, to: [x, y] });
                        mouse_changed = true;
                    }
                }
                Event::Click {
                    down: true,
                    at: (x, y),
                } if battle.book_open() => {
                    let did = battle.book_press([x, y]);
                    grew |= book_did(&mut battle, &mut inputs, at, did, &mut book_changed)?;
                }
                Event::Click { down: false, .. } if battle.book_open() => {}
                Event::Click { down: true, .. } if battle.aiming() => {
                    // The press casts at the hex the cursor took as it went down: the pass's
                    // moves before it followed first (E17 M5 T8b2, R4b-1: `wait` gives every
                    // pending event at once, a move and the press among them).
                    followed |= battle.mouse(&inputs, &scrolls)?;
                    if let Some(command) = battle.aim_press(at) {
                        grew |= give(&mut battle, &mut inputs, at, command)?;
                        // The stats dialog's close after its cast button's aim, the action
                        // given (E17 M6 T2a2b1: its posted move waits behind the action).
                        if battle.looked_after_aim() {
                            inputs.push(Input::Looked { at });
                        }
                    }
                    mouse_changed = true;
                }
                Event::Click { down: false, .. } if battle.aiming() => {}
                Event::Pointer(p) => {
                    pointer = p;
                    redraw |= drawn_cursor;
                    if let Some((x, y)) = p {
                        // The options dialog's arrow goes (E17 M6 T4a).
                        battle.mouse_moved();
                        // The battle takes the move now (inputs after a seek back are gone).
                        inputs.retain(|i| i.at() <= at);
                        inputs.push(Input::Move { at, to: [x, y] });
                        // The mouse's button follows it on or off the button it holds.
                        if let Some(h) = held.as_mut().filter(|h| h.key.is_none()) {
                            h.over = battle.panel_at(at).button_at([x, y]) == Some(h.id);
                        }
                        mouse_changed = true;
                    }
                }
                // A press while Auto Combat plays stops it as the next turn starts and does
                // nothing else (E17 M5 T4c1).
                Event::Click { down: true, .. } if battle.stop_auto(at).is_some() => {
                    mouse_changed = true;
                }
                // A press made while a battle fought here takes no input is dropped, its release
                // then doing nothing (design §9.11; E17 M5 T2b).
                Event::Click { down: true, .. } if !battle.takes(at) => {}
                Event::Click {
                    down: true,
                    at: (x, y),
                } => {
                    if let Some(id) = battle.panel_at(at).button_at([x, y]) {
                        held = Some(Held {
                            id,
                            key: None,
                            over: true,
                        });
                        mouse_changed = true;
                        continue;
                    }
                    // Any left press over the battlefield hides the army info windows first, the
                    // mouse's hex kept (0x4752BB; E17 M6 T4b2b1, design §12.15).
                    if h3_view::army_info::on_field(battle.layout(), [x, y]) {
                        inputs.push(Input::Hide { at, key: false });
                        mouse_changed = true;
                    }
                    if let Some(command) = battle.press(at, [x, y]) {
                        // A press over the battlefield acts as the button goes down (design
                        // §9.11; E17 M5 T2b).
                        grew |= give(&mut battle, &mut inputs, at, command)?;
                    } else if battle.look(at, [x, y], false) {
                        // A stack to look at: its stats dialog (E17 M6 T2a2a).
                        inputs.push(Input::Look { at });
                        book_changed = true;
                    } else if battle.turret_info(at, [x, y], false) {
                        // A turret: its info in a box, the combat cursor's pointer set first
                        // (0x476E22; E17 M6 T2b2).
                        inputs.push(Input::Look { at });
                        book_changed = true;
                    }
                }
                Event::Click { down: false, .. } => {
                    if let Some(h) = held.take_if(|h| h.key.is_none()) {
                        // A disabled button takes none (0x4562DD; E17 M3 T9b, the reviewer).
                        if let Some(by) = scroll_by(h.id).filter(|_| h.over && battle.scrolls(at)) {
                            scrolls.push((at, by));
                        }
                        // Wait and Defend give theirs as the button is released over them.
                        if let Some(command) = battle.button(at, h.id).filter(|_| h.over) {
                            grew |= give(&mut battle, &mut inputs, at, command)?;
                        }
                        // Auto Combat switches on (E17 M5 T4c1).
                        if h.over && h.id == h3_view::panel::AUTO_COMBAT {
                            grew |= battle.switch_auto(at)?;
                        }
                        // The spell book opens as its button is released over it (E17 M5 T3a).
                        if h.over && h.id == h3_view::panel::SPELL_BOOK && battle.open_book(at) {
                            inputs.push(Input::Book { at });
                            book_changed = true;
                        }
                        // The combat options dialog opens as its button is released over it (E17
                        // M6 T4a).
                        book_changed |=
                            h.over && h.id == h3_view::panel::OPTIONS && battle.open_options(at);
                        // Surrender and Retreat ask first (E17 M5 T4a).
                        book_changed |= h.over && battle.ask_leave(at, h.id);
                        mouse_changed = true;
                    }
                }
                Event::Redraw => redraw = true,
            }
        }
        // The result window's OK closes the battle's screen (E17 M5 T4b1).
        if battle.result_closed() {
            break;
        }
        // The options dialog's close changed the Auto Combat boxes (E17 M6 T4b1): the turns to
        // come, and under Auto Combat the turn waiting, play by them.
        grew |= battle.boxes_closed(clock.ms(win.ticks_ns()))?;
        // The options dialog's close composes the background again with its options, without the
        // cursor shadow's call (E17 M6 T4c, design §12.16: oddity 2, CLT-Q52).
        if options_open && !battle.options_open() {
            inputs.push(Input::Shades {
                at: clock.ms(win.ticks_ns()),
                options: Shades::of(&battle.settings),
                keyed: false,
            });
            mouse_changed = true;
        }
        options_open = battle.options_open();
        // An aim with both shadows on turns the movement shadow off as it starts, saved so, and on
        // again as it ends (E17 M6 T4c, design §12.16: `battle_set_shades` 0x59EFB8, 0x59F077;
        // oddity 8), as the mouse's follow does (`mouse::track`).
        let aim = battle.aim_original();
        if aimed.is_none()
            && let Some(set) = aim.and_then(|o| Shades::of(&battle.settings).aiming(o))
        {
            set.set_in(&mut battle.settings);
            save_settings(&battle.settings, &files);
            aim_shades = true;
        }
        if aim.is_none() && core::mem::take(&mut aim_shades) {
            battle.settings.movement_shade = true;
            save_settings(&battle.settings, &files);
        }
        aimed = aim;
        // The options dialog set Creature Info's level (E17 M6 T4b2a): the hints read it from now.
        if battle.settings.army_info != army_info {
            army_info = battle.settings.army_info;
            inputs.push(Input::ArmyInfo {
                at: clock.ms(win.ticks_ns()),
                level: army_info,
            });
            mouse_changed = true;
        }
        // The mouse, a button held, the log scrolled (E17 M3 T6c-2), a command given (E17 M5
        // T2b): the battle follows them and the frame now is made again.
        if mouse_changed || grew || book_changed {
            let pressed = held.filter(|h| h.over).map(|h| h.id);
            let changed = battle.mouse(&inputs, &scrolls)? || followed || battle.pressed != pressed;
            battle.pressed = pressed;
            if changed || grew || book_changed {
                let ms = clock.ms(win.ticks_ns());
                changes = Changes::new(&battle);
                let latest = changes.latest(&mut battle, ms)?;
                // The result window's clip goes on from its frame now (E17 M5 T4b2b).
                let latest = battle
                    .clip_times(ms)
                    .map_or(latest, |(began, _)| latest.max(began));
                next = Some(frame_at(&mut battle, latest)?);
                shown = None;
            }
        }
        // The cursor's frame now, none while a ray hides it: the hardware cursor made again when
        // it changes.
        let ms = clock.ms(win.ticks_ns());
        let frame = battle.cursor(ms);
        // The stats dialog's popup hides it too (`mouse_hide` 0x602A6C; E17 M6 T2a2b1), and a
        // turret's info held (T2b2).
        let visible =
            !battle.timeline().cursor_hidden(ms) && !battle.stats_popup() && !battle.popup_held();
        if (&frame, visible) != (&frame_now, cursor_visible) {
            let image = if visible && !drawn_cursor {
                Some(cursors.get(&battle, &frame)?.1)
            } else {
                None
            };
            (frame_now, cursor_visible) = (frame, visible);
            if drawn_cursor {
                redraw = true;
            } else {
                if win.set_cursor(image).is_err() {
                    drawn_cursor = true;
                    redraw = true;
                }
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
                while let Some(c) = battle
                    .timeline()
                    .cues
                    .get(cue)
                    .filter(|c| c.at <= battle_ms)
                {
                    p.cue(lib, c);
                    cue += 1;
                }
                if let Some((_, tune)) = battle
                    .end_tune()
                    .filter(|(at, _)| !ended && *at <= battle_ms)
                {
                    p.end(lib, tune);
                    ended = true;
                }
            }
            p.service(lib, now);
        }
        // A later change may be due already: show the latest instead; if it draws what shows,
        // nothing new is presented (a redraw still is).
        let due = match next.take_if(|f| f.t <= battle_ms) {
            None => None,
            Some(d) => {
                let latest = changes.latest(&mut battle, battle_ms)?;
                if latest <= d.t {
                    Some(d)
                } else {
                    report.skipped += 1;
                    let late = frame_at(&mut battle, latest)?;
                    match shown.as_mut().filter(|s| s.list == late.list) {
                        Some(s) => {
                            s.t = latest;
                            next = next_frame(&mut battle, &mut changes, s)?;
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
                (drawn_cursor && cursor_visible)
                    .then_some(pointer)
                    .flatten(),
                &cursors.get(&battle, &frame_now)?.0,
                read,
            )?;
            report
                .late_us
                .push(clock.late(win.ticks_ns(), due.t) / 1000);
            if let Some((back, presented)) = back {
                compare(&back, &due, &presented, &mut report);
            }
            next = next_frame(&mut battle, &mut changes, &due)?;
            shown = Some(due);
        } else if redraw && let Some(f) = &shown {
            show(
                &mut win,
                f,
                (drawn_cursor && cursor_visible)
                    .then_some(pointer)
                    .flatten(),
                &cursors.get(&battle, &frame_now)?.0,
                false,
            )?;
        }
    }
    let seconds = win.ticks_ns().saturating_sub(started) / 1_000_000;
    lines.push(format!("keys: {keys_from}"));
    lines.push(format!("settings: {settings_from}"));
    lines.extend(unsaved.take());
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
        "{} commands, {} ms at speed {}; {} shown in {} ms, {} passed over",
        battle.commands(),
        battle.timeline().end,
        source.speed.setting(),
        report.late_us.len(),
        seconds,
        report.skipped,
    ));
    lines.push(format!(
        "frame after its time (ms): {}",
        summary(&mut report.late_us)
    ));
    let mut ok = report.difference.is_none();
    if let Some(p) = &presser {
        lines.push(format!(
            "pressed: {} commands on the battlefield, {} on the panel's buttons, {} given without \
             a press",
            p.at, p.buttons, p.given
        ));
        let yes = |b: bool| if b { "yes" } else { "no" };
        if let Some(turn) = p.auto {
            // E17 M5 T4c1: switched on at the human's turn asked, stopped two commands later.
            let at = |c: Option<usize>| c.map_or_else(|| "never".to_owned(), |c| c.to_string());
            lines.push(format!(
                "auto combat at the human's turn {turn}: switched on after {} commands, stopped \
                 after {}",
                at(p.switched.map(|(on, _)| on)),
                at(p.switched.and_then(|(_, off)| off)),
            ));
            ok &= p.switched.is_some();
        }
        if let Some(left) = p.left {
            // Left through its question (E17 M5 T4a): the bot's battle up to the question, and
            // the battle over by the leaving.
            let before = p.before_leaving == Some(true);
            let over = !battle.playing();
            lines.push(format!(
                "left by {left:?} after {} commands: the battle before it the bot's at once: {}, \
                 over: {}",
                battle.commands().saturating_sub(1),
                yes(before),
                yes(over)
            ));
            ok &= before && over;
        } else if let Some(online) = ground.online() {
            // A seat on a server (E17 M5 T6b1): what the connection did, whether the game ended.
            let online = online.borrow();
            lines.extend(online.lines.iter().cloned());
            lines.push(format!(
                "re-joined {} times; the game over: {}",
                online.rejoined,
                yes(online.over())
            ));
        } else {
            let same = battle.as_the_bot_fights().unwrap_or(Ok(false))?;
            lines.push(format!(
                "the battle so far is the bot's at once: {}",
                yes(same)
            ));
            ok &= same;
        }
    }
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

/// What the spell book `did` at `at` (E17 M5 T3a): a change to draw, the cast it closed on
/// given, or the aim it closed on started (E17 M5 T3b); whether the battle took a command.
fn book_did(
    battle: &mut Battle<'_>,
    inputs: &mut Vec<Input>,
    at: u32,
    did: BookDid,
    changed: &mut bool,
) -> Result<bool, String> {
    *changed |= did != BookDid::Nothing;
    match did {
        BookDid::Nothing => Ok(false),
        BookDid::Changed => {
            // Closed without a spell, the battle takes the mouse again.
            if !battle.book_open() {
                inputs.push(Input::Cancel { at });
            }
            Ok(false)
        }
        BookDid::Cast(command) => give(battle, inputs, at, command),
        BookDid::Aim(spell) => {
            inputs.push(Input::Aim {
                at,
                spell,
                creature: false,
            });
            Ok(false)
        }
    }
}

/// The open box as the window tells one from another (E17 M5 T8b2): its buttons' places, none
/// while no box is open. A box that gives way to another (the surrender refused) has others.
/// What a stats dialog's input did (E17 M6 T2a2a): its close hands the mouse back to the battle
/// (`Input::Looked`); its cast button's (E17 M6 T2a2b1) starts the acting Faerie Dragon's aim
/// first, the close coming as the aim ends (at once when no aim starts). Whether the screen
/// changed.
fn stats_did(battle: &mut Battle<'_>, inputs: &mut Vec<Input>, at: u32, did: StatsDid) -> bool {
    match did {
        StatsDid::Closed => inputs.push(Input::Looked { at }),
        StatsDid::Cast => match battle.look_cast(at) {
            Some(spell) => inputs.push(Input::Aim {
                at,
                spell,
                creature: true,
            }),
            None => inputs.push(Input::Looked { at }),
        },
        StatsDid::Nothing | StatsDid::Changed => {}
    }
    did != StatsDid::Nothing
}

fn box_shown(battle: &Battle<'_>) -> Option<[Option<[i32; 2]>; 3]> {
    use h3_view::message::{NO, OK, YES};
    // The combat options dialog's opening flushes the input queue too (E17 M6 T4a).
    let options = battle
        .options_open()
        .then_some([None, None, Some([-1, -1])]);
    battle
        .message_open()
        .then(|| [OK, YES, NO].map(|id| battle.message_button(id)))
        .or(options)
}

/// What the combat options dialog `did` (E17 M6 T4a, design §12.12): its press samples played
/// (the speeds' twice, oddity 1), its volumes on the sound at once; at its close after a click its
/// settings written to the settings file (`files`: the file and the report's lines). Whether it
/// changed (draw again).
fn options_did(
    battle: &mut Battle<'_>,
    player: &mut Option<Player>,
    did: crate::battle::OptionsDid,
    files: &SettingsFile<'_>,
) -> bool {
    if let (Some(p), Some(lib)) = (player.as_mut(), files.library) {
        if did.button_sound {
            p.click(lib, h3_view::sound::FILE_VOLUME);
        }
        if did.box_sound {
            p.click(lib, h3_view::sound::EFFECT_VOLUME);
        }
        p.set_volumes(battle.settings.music, battle.settings.effects);
    }
    if let Some(settings) = battle.options_saved() {
        save_settings(&settings, files);
    }
    did != crate::battle::OptionsDid::default()
}

/// What becomes of the key that stopped Auto Combat as the stop's turn `turn` comes ([`carried`];
/// E17 M5 T4c1, M6 T4b1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Carry {
    /// It acts on that turn, a human's waiting for its command (0x474625).
    Act,
    /// It is lost: the turn played itself (a computer's), or Auto Combat was still on at the
    /// human's turn's start (`at_start`: a turn it does not play reads no stop, the key dropped
    /// as with Auto Combat off, CLT-Q45 (2)).
    Drop,
    /// The turn has not come or not waited yet.
    Wait,
}

/// [`Carry`] by whether the battle waits for a human's command, the turns started `last` and the
/// stop's `turn`, and (asked only for a human's waiting turn) whether Auto Combat was on at its
/// start.
fn carry((waiting, last, turn): (bool, usize, usize), at_start: impl FnOnce() -> bool) -> Carry {
    if waiting && last == turn {
        if at_start() { Carry::Drop } else { Carry::Act }
    } else if last > turn {
        Carry::Drop
    } else {
        Carry::Wait
    }
}

/// The battle's keys that act on the turn Auto Combat's stop comes at when they stopped it, a
/// human's (0x474625, `battle_buttons`' key switch; E17 M5 T4c1, design §7.22): the log's arrows,
/// the Faerie Dragon's F and, since E17 M6 T4b2a, F5 (case 63, Creature Info's level), since
/// E17 M6 T4c F6 to F8 (the grid and the shadows).
const fn carried(command: Do) -> bool {
    matches!(
        command,
        Do::LogUp | Do::LogDown | Do::DragonSpell | Do::ArmyInfo | Do::Shades(_)
    )
}

/// Creature Info's level cycled by the key F5 at `at` (E17 M6 T4b2a, design §12.14;
/// `battle_buttons`' case 63): an input from then (the hints written after it read it, none is
/// written again) and the settings file written at once (`files`). The level now.
fn cycle_army_info(
    battle: &mut Battle<'_>,
    inputs: &mut Vec<Input>,
    at: u32,
    files: &SettingsFile<'_>,
) -> u8 {
    let level = battle.settings.cycle_army_info();
    inputs.push(Input::ArmyInfo { at, level });
    save_settings(&battle.settings, files);
    level
}

/// The grid, the cursor shadow or the movement shadow toggled by its key at `at` (E17 M6 T4c,
/// design §12.16; `battle_buttons`' F6 to F8, `battle_set_shades`' effects): an input from then
/// (the mouse's follow takes the shade off first, composes the background again and on a human's
/// turn shades the mouse's hex again) and the settings file written at once (`files`).
fn toggle_shades(
    battle: &mut Battle<'_>,
    inputs: &mut Vec<Input>,
    (at, toggle): (u32, Toggle),
    files: &SettingsFile<'_>,
) {
    let options = Shades::of(&battle.settings).toggled(toggle);
    options.set_in(&mut battle.settings);
    inputs.push(Input::Shades {
        at,
        options,
        keyed: true,
    });
    save_settings(&battle.settings, files);
}

/// Writes `settings` to the settings file (E17 M6 T4a: the options dialog's close after a click;
/// T4b2a: the key F5), a failure in the report's lines.
fn save_settings(settings: &Settings, files: &SettingsFile<'_>) {
    if let Err(e) = fs::write(files.path, settings.write()) {
        files
            .lines
            .borrow_mut()
            .push(format!("settings: {}: {e}", files.path.display()));
    }
}

/// Where the combat options dialog's settings go (E17 M6 T4a): the file, the sound's library for
/// its press sample, the report's lines for a failed write.
struct SettingsFile<'a> {
    path: &'a Path,
    library: Option<&'a Library>,
    lines: &'a std::cell::RefCell<Vec<String>>,
}

/// The hide one of the keys C, F, S, R and O gives at `at` with the mouse at `pointer` (E17 M6
/// T4b2b1, CLT-Q49 fixed in the client's profile, the window's; `h3_view::army_info::Hover::key`
/// for a still): over the battlefield while the battle takes the key, as a left press there
/// hides the army info windows (*provisional*: whatever the key then does).
fn key_hides(battle: &Battle<'_>, at: u32, pointer: Option<(i32, i32)>) -> Option<Input> {
    let (x, y) = pointer?;
    (battle.takes(at) && h3_view::army_info::on_field(battle.layout(), [x, y]))
        .then_some(Input::Hide { at, key: true })
}

/// What a message box `did` at `at` (E17 M5 T4a): a change to draw, or the command its OK gave
/// (Retreat, Surrender); whether the battle took a command.
fn message_did(
    battle: &mut Battle<'_>,
    inputs: &mut Vec<Input>,
    at: u32,
    did: MessageDid,
    changed: &mut bool,
) -> Result<bool, String> {
    *changed |= did != MessageDid::Nothing;
    match did {
        MessageDid::Give(command) => give(battle, inputs, at, command),
        // A turret's info closed (E17 M6 T2b2): the battle takes the mouse again (0x476E34).
        MessageDid::Looked => {
            inputs.push(Input::Looked { at });
            Ok(false)
        }
        MessageDid::Nothing | MessageDid::Changed | MessageDid::Ended => Ok(false),
    }
}

/// Gives a human's `command` at `at` (E17 M5 T2b): the mouse's inputs after it gone, its
/// action's start taken; whether the battle took it.
fn give(
    battle: &mut Battle<'_>,
    inputs: &mut Vec<Input>,
    at: u32,
    command: Command,
) -> Result<bool, String> {
    inputs.retain(|i| i.at() <= at);
    inputs.push(Input::Act { at });
    battle.command(at, command)
}

/// An unattended run's presses for the humans (E17 M5 T2b): where a press gives each command
/// on the battlefield, and the commands pressed there, on the panel's buttons and given without
/// a press.
#[derive(Debug, Default)]
struct Presser {
    points: Vec<([i32; 2], Vec<[i32; 2]>)>,
    at: usize,
    buttons: usize,
    given: usize,
    /// The panel button the human side leaves by (E17 M5 T4a: `--leave`), until it left or was
    /// told it cannot pay.
    leave: Option<u16>,
    /// When the open message box showed, in the window's milliseconds (`win.ticks_ns`): a
    /// notice's box holds the battle's clock (E17 M5 T8b2, R5a-1).
    asked: Option<u64>,
    /// Whether the open box is the leaving's (its question, or the refusal after it), not a
    /// notice's.
    asking: bool,
    /// Whether the battle was the bot's at once when the side asked to leave, and the command
    /// it left by.
    before_leaving: Option<bool>,
    left: Option<Command>,
    /// The human turn Auto Combat is switched on at (E17 M5 T4c1: `--auto`), the human turns
    /// met so far, and the commands played when it was switched on and when it was stopped.
    auto: Option<usize>,
    waited: usize,
    met: Option<usize>,
    switched: Option<(usize, Option<usize>)>,
}

impl Presser {
    /// At `now`, [`PRESS_AFTER_MS`] after a human's turn started, presses the bot's choice: the
    /// mouse moved where it gives it, then its press; whether a command was given. `wall` is the
    /// window's milliseconds, which an open box's OK is timed by.
    fn press(
        &mut self,
        battle: &mut Battle<'_>,
        inputs: &mut Vec<Input>,
        now: u32,
        wall: u64,
    ) -> Result<bool, String> {
        // The result window stays until the run's end (E17 M5 T4b1: its frames read back too).
        if battle.result_open() {
            return Ok(false);
        }
        if battle.message_open() {
            return self.answer(battle, inputs, now, wall);
        }
        // Auto Combat stopped by a press two commands after it was switched on (E17 M5 T4c1).
        if let Some((on, None)) = self.switched
            && battle.commands() >= on + 2
            && battle.stop_auto(now).is_some()
        {
            self.switched = Some((on, Some(battle.commands())));
            return Ok(true);
        }
        // A seat on a server waits for the other seat's turns too, taking no press then (E17 M5
        // T6b1).
        if !battle.waiting()
            || now < battle.timeline().end.saturating_add(PRESS_AFTER_MS)
            || !battle.takes(now)
        {
            return Ok(false);
        }
        if self.met != Some(battle.commands()) {
            self.met = Some(battle.commands());
            self.waited += 1;
        }
        // Auto Combat's button clicked at the turn `--auto` names (E17 M5 T4c1).
        if self.auto == Some(self.waited) && self.switched.is_none() {
            let commands = battle.commands();
            if let Some(to) = button_centre(battle.panel_at(now), h3_view::panel::AUTO_COMBAT) {
                inputs.retain(|i| i.at() <= now);
                inputs.push(Input::Move { at: now, to });
            }
            if battle.switch_auto(now)? {
                self.switched = Some((commands, None));
                self.buttons += 1;
                return Ok(true);
            }
        }
        // Leaving: the panel's button clicked, its question shown (E17 M5 T4a).
        if let Some(id) = self.leave {
            if let Some(to) = button_centre(battle.panel_at(now), id) {
                inputs.retain(|i| i.at() <= now);
                inputs.push(Input::Move { at: now, to });
            }
            let before = battle.as_the_bot_fights();
            if battle.ask_leave(now, id) {
                (self.asked, self.asking) = (Some(wall), true);
                self.before_leaving = Some(before.unwrap_or(Ok(false))?);
                return Ok(true);
            }
        }
        let wanted = battle.choice().ok_or("no command for the turn waiting")?;
        inputs.retain(|i| i.at() <= now);
        // A hero's cast through its book and the aim, a Faerie Dragon's by its key (E17 M5 T3b).
        if matches!(wanted, Command::Cast { .. } | Command::CreatureCast { .. }) {
            return self.cast(battle, inputs, now, wanted);
        }
        let to = match battle.press_for(now, wanted, &self.points) {
            Some(Press::At(point)) => {
                self.at += 1;
                Some(point)
            }
            Some(Press::Button(id)) => {
                self.buttons += 1;
                button_centre(battle.panel_at(now), id)
            }
            None => {
                self.given += 1;
                None
            }
        };
        if let Some(to) = to {
            inputs.push(Input::Move { at: now, to });
        }
        give(battle, inputs, now, wanted)
    }
}

impl Presser {
    /// The open message box's OK clicked [`PRESS_AFTER_MS`] after it showed (E17 M5 T4a), timed
    /// by the window's clock `wall` (a notice's box holds the battle's at `now`): the command it
    /// gives, or the box it gives way to (a surrender the side cannot pay, which ends the
    /// leaving); whether a command was given or the screen changed.
    fn answer(
        &mut self,
        battle: &mut Battle<'_>,
        inputs: &mut Vec<Input>,
        now: u32,
        wall: u64,
    ) -> Result<bool, String> {
        let asked = *self.asked.get_or_insert(wall);
        if wall < asked.saturating_add(u64::from(PRESS_AFTER_MS)) {
            return Ok(false);
        }
        let ok = battle
            .message_button(h3_view::message::YES)
            .or_else(|| battle.message_button(h3_view::message::OK))
            .ok_or("the message box has no OK")?;
        let _ = battle.message_press(ok);
        let did = battle.message_release();
        self.asked = battle.message_open().then_some(wall);
        // A notice's box closed leaves the leaving to come (E17 M5 T8b2).
        if self.asking && !battle.message_open() && did != MessageDid::Give(Command::Retreat {}) {
            self.leave = None;
        }
        self.asking &= battle.message_open();
        match did {
            MessageDid::Give(command) => {
                self.left = Some(command);
                give(battle, inputs, now, command)
            }
            // The presser never presses a turret (E17 M6 T2b2): its box closed, the battle takes
            // the mouse again all the same.
            MessageDid::Looked => {
                inputs.push(Input::Looked { at: now });
                Ok(true)
            }
            MessageDid::Changed | MessageDid::Nothing | MessageDid::Ended => Ok(true),
        }
    }

    /// Presses the bot's cast `wanted` at `now` (E17 M5 T3b): a hero's opens its book, turns to
    /// the spell (the next pages, then the tabs in turn) and presses its place, then aims it at
    /// the target's hex if it takes one; a Faerie Dragon's aims by its key F. A press that would
    /// give another command (two stacks on the hex) gives the bot's without one.
    fn cast(
        &mut self,
        battle: &mut Battle<'_>,
        inputs: &mut Vec<Input>,
        now: u32,
        wanted: Command,
    ) -> Result<bool, String> {
        let target = match wanted {
            Command::Cast { spell, target, .. } => {
                if !battle.open_book(now) {
                    return Err("the spell book does not open for a cast".to_owned());
                }
                inputs.push(Input::Book { at: now });
                let mut place = battle.book_place_of(spell);
                for _ in 0..BOOK_TURNS {
                    if place.is_some() {
                        break;
                    }
                    let key = if battle.book_next() {
                        h3_view::book::Key::Next
                    } else {
                        h3_view::book::Key::TabDown
                    };
                    battle.book_key(key);
                    place = battle.book_place_of(spell);
                }
                let place = place.ok_or("the spell is on no page of the book")?;
                inputs.push(Input::Move { at: now, to: place });
                let mut changed = false;
                match battle.book_press(place) {
                    BookDid::Aim(spell) => inputs.push(Input::Aim {
                        at: now,
                        spell,
                        creature: false,
                    }),
                    did => {
                        self.at += 1;
                        return book_did(battle, inputs, now, did, &mut changed);
                    }
                }
                target
            }
            Command::CreatureCast { target } => {
                // The key does nothing once the side's hero cast this round (the original's
                // cursor casts then, which the client does not build): given without a press.
                let Some(spell) = battle.dragon_aim(now) else {
                    self.given += 1;
                    return give(battle, inputs, now, wanted);
                };
                inputs.push(Input::Aim {
                    at: now,
                    spell,
                    creature: true,
                });
                target
            }
            _ => return Ok(false),
        };
        let hex = match target {
            Target::Stack { stack } => battle.stack_hex(stack),
            Target::Hex { hex } => Some(hex),
            Target::None {} => None,
        }
        .ok_or("a cast aimed at nothing")?;
        let to = self
            .points
            .get(usize::from(hex))
            .map(|(centre, _)| *centre)
            .ok_or("a hex off the field")?;
        inputs.push(Input::Move { at: now, to });
        battle.mouse(inputs, &[])?;
        match battle.aim_press(now).filter(|c| *c == wanted) {
            Some(command) => {
                self.at += 1;
                give(battle, inputs, now, command)
            }
            None => {
                battle.cancel_aim();
                inputs.push(Input::Cancel { at: now });
                self.given += 1;
                give(battle, inputs, now, wanted)
            }
        }
    }
}

/// The centre of the panel's button `id`.
fn button_centre(panel: &h3_ui::ui::Dialog, id: u16) -> Option<[i32; 2]> {
    panel.items.iter().find_map(|item| match item {
        Item::Button(b) if b.id == id => Some([
            panel.place[0] + b.at[0] + b.size[0] / 2,
            panel.place[1] + b.at[1] + b.size[1] / 2,
        ]),
        _ => None,
    })
}

/// A panel button held: by the mouse (`key` none; `over` while the mouse stays on it) or by a
/// hotkey.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Held {
    id: u16,
    key: Option<Key>,
    over: bool,
}

/// How far the log's scroll button `id` scrolls it (`battle_log_up` 0x472DF0, `battle_log_down`
/// 0x472E40: the panel's items 2006 and 2007).
fn scroll_by(id: u16) -> Option<i64> {
    match id {
        2006 => Some(-1),
        2007 => Some(1),
        _ => None,
    }
}

/// The combat cursor's frames made so far: each drawn and as the hardware cursor's image.
#[derive(Default)]
struct Cursors {
    made: std::collections::BTreeMap<CursorLook, (Cursor, CursorImage)>,
}

impl Cursors {
    /// The cursor `look` (the combat cursor's frame, or the spell book's default cursor's: E17
    /// M5 T3a) with its hot spot.
    fn get(
        &mut self,
        battle: &Battle<'_>,
        look: &CursorLook,
    ) -> Result<(Cursor, CursorImage), String> {
        if let Some(made) = self.made.get(look) {
            return Ok(made.clone());
        }
        let [x, y] = look.hot;
        let hot = (u32::try_from(x).unwrap_or(0), u32::try_from(y).unwrap_or(0));
        let (def, frame) = (&look.def, look.frame);
        let cursor = battle
            .sprite(def)
            .and_then(|s| Cursor::read(s, frame, hot, FORMAT))
            .ok_or_else(|| format!("{def}: no frame {frame}"))?;
        let image = cursor.image(FORMAT);
        let hardware = CursorImage {
            width: image.width,
            height: image.height,
            hot_x: image.hot.0,
            hot_y: image.hot.1,
            rgba: image.rgba,
        };
        self.made
            .insert(look.clone(), (cursor.clone(), hardware.clone()));
        Ok((cursor, hardware))
    }
}

/// Presents `frame`, with the drawn cursor at `pointer` if any; reads it back first if `read`,
/// with the pixels presented (the drawn cursor's among them: E17 M3 T9b, the reviewer).
fn show(
    win: &mut Window,
    frame: &Frame,
    pointer: Option<(i32, i32)>,
    cursor: &Cursor,
    read: bool,
) -> Result<Option<(ReadBack, Vec<u32>)>, String> {
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
        win.present_read(pixels)
            .map(|back| Some((back, pixels.clone())))
    } else {
        win.present(pixels).map(|()| None)
    }
}

/// Compares what the window read back with the pixels `presented` of `frame` at its scale, the
/// black around them included.
fn compare(back: &ReadBack, frame: &Frame, presented: &[u32], report: &mut Report) {
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
            let want = inside(x, y).map_or(0, |i| presented[i]);
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
            "--keys",
            "mine.toml",
            "--auto",
            "0",
        ]))
        .unwrap();
        assert_eq!(o.keys, Some(PathBuf::from("mine.toml")));
        assert_eq!(o.auto, Some(1));
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
    fn the_keys_bound_are_the_adapters() {
        let known: Vec<&str> = Key::ALL.iter().map(|k| k.name()).collect();
        assert_eq!(Keys::original().check(&known), Ok(()));
        // No file here: the defaults.
        let (keys, from) = load_keys(None).unwrap();
        assert_eq!((keys, from.as_str()), (Keys::original(), "the defaults"));
        assert!(load_keys(Some(Path::new("no/such/keys.toml"))).is_err());
    }

    #[test]
    fn the_settings_come_from_the_users_file_or_a_first_run() {
        // E17 M6 T4a: no file here, a first run's, written to settings.toml here; a file named
        // is read and written back; a missing or a bad one refused.
        let (settings, to, from) = load_settings(None).unwrap();
        assert_eq!(settings, Settings::FIRST_RUN);
        assert_eq!(
            (to.as_path(), from.as_str()),
            (Path::new(SETTINGS_FILE), "a first run's")
        );
        assert!(load_settings(Some(Path::new("no/such/settings.toml"))).is_err());
        let dir = std::env::temp_dir().join(format!("h3-view-settings-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.toml");
        let set = Settings {
            speed: 2,
            ..Settings::FIRST_RUN
        };
        fs::write(&path, set.write()).unwrap();
        let (read, to, _) = load_settings(Some(&path)).unwrap();
        assert_eq!((read, to), (set, path.clone()));
        fs::write(&path, "[battle]\nmusic = 1\n").unwrap();
        assert!(load_settings(Some(&path)).unwrap_err().contains("music"));
        fs::remove_dir_all(&dir).unwrap();
        // --settings and an explicit --speed.
        let args: Vec<String> = ["a.toml", "--settings", "s.toml", "--speed", "1"]
            .map(String::from)
            .into();
        let o = Options::parse(&args).unwrap();
        assert_eq!(o.settings.as_deref(), Some(Path::new("s.toml")));
        assert!(o.speed_set);
        let o = Options::parse(&["a.toml".to_owned()]).unwrap();
        assert!(!o.speed_set && o.settings.is_none());
    }

    #[test]
    fn the_keys_a_stop_carries_are_the_battle_s_own_handler_s() {
        // E17 M6 T4b2a (the reviewer): F5 stops Auto Combat and cycles the level on the human's
        // turn the stop comes at, as the arrows and F; a button's hotkey or the stats' T not.
        for c in [Do::LogUp, Do::LogDown, Do::DragonSpell, Do::ArmyInfo] {
            assert!(carried(c), "{c:?}");
        }
        for c in [Do::Stats, Do::Button(h3_view::panel::WAIT), Do::StatsOk] {
            assert!(!carried(c), "{c:?}");
        }
    }

    #[test]
    fn the_stop_s_key_acts_on_a_human_s_turn_and_is_lost_on_a_computer_s() {
        // E17 M5 T4c1, M6 T4b1 (the reviewer of T4b2a): the human's turn the stop comes at
        // waiting, the key acts unless Auto Combat was still on at its start; a turn after it
        // already, lost; before, it waits.
        assert_eq!(carry((true, 3, 3), || false), Carry::Act);
        assert_eq!(carry((true, 3, 3), || true), Carry::Drop);
        assert_eq!(carry((false, 4, 3), || unreachable!()), Carry::Drop);
        assert_eq!(carry((true, 4, 3), || unreachable!()), Carry::Drop);
        assert_eq!(carry((false, 3, 3), || unreachable!()), Carry::Wait);
        assert_eq!(carry((true, 2, 3), || unreachable!()), Carry::Wait);
    }

    #[test]
    fn the_key_f5_s_level_is_written_at_once() {
        // E17 M6 T4b2a: F5 cycles the level and writes the file, which the next start reads; a
        // file that cannot be written is a line of the report.
        let dir = std::env::temp_dir().join(format!("h3-view-army-info-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.toml");
        let lines = std::cell::RefCell::new(Vec::new());
        let files = SettingsFile {
            path: &path,
            library: None,
            lines: &lines,
        };
        let mut settings = Settings::FIRST_RUN;
        assert_eq!(settings.cycle_army_info(), 1);
        save_settings(&settings, &files);
        assert_eq!(load_settings(Some(&path)).unwrap().0.army_info, 1);
        let unwritable = SettingsFile {
            path: &dir,
            ..files
        };
        save_settings(&settings, &unwritable);
        assert_eq!(lines.borrow().len(), 1);
        assert!(lines.borrow()[0].starts_with("settings: "));
        fs::remove_dir_all(&dir).unwrap();
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
