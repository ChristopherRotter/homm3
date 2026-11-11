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
//! (their players the battle's human ones, the computer playing the others: E17 M6 T5b1, the
//! engine's original computer on its side's world, or seeing the whole battle with `--sight
//! all-seeing`, *provisional*; under Auto Combat it plays a human's side too, casting for its
//! hero with Combat Auto Spells set, design §12.18). On a human's turn the
//! battle waits at its end, idling, until a left press over the battlefield gives its command
//! (`h3_view::mouse::click`, as the button goes down) or the panel's Wait or Defend is clicked
//! (the mouse's button released over it, or its hotkey released: `mouse::button`; in the tactics
//! phase its panel's Next Creature or Start Combat, E17 M5 T4c2b); input made
//! while the battle takes none (an action playing, the computer's turn) gives nothing. The screen
//! shows the side to act's hidden objects (`anim::Timeline::screen`). Unattended (`--check`,
//! `--measure`), the window presses a human's commands itself: a fixed bot's choice, pressed
//! where the mouse gives it (a hero's cast through its book and aim; the tactics phase ended by
//! Start Combat, E17 M5 T4c2b), so that the battle is the one fought again at once with the same
//! setup (the computer's choices asked again in order, `live::refight`), which the run checks.
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

use h3_battle_ai::Sight;
use h3_protocol::wire::{Command, Target};
use h3_render::Bugs;
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
use crate::battle::{
    Battle, BookDid, CursorLook, Ground, MessageDid, Opened, Press, Source, StatsDid,
};
use crate::boxes::Asked;
use crate::live::Computer;

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
    /// Unattended, the human's answers to Auto Combat's question and offer (E17 M6 T5b2a,
    /// `--auto-question cancel|retreat|surrender`, *provisional*: Cancel by default): the
    /// question's OK, then the offer's.
    question: (bool, bool),
    /// Unattended, the result window's OK pressed once it shows (E17 M7 T1d, T7b's R10-2:
    /// `--close-result`), Eagle Eye's boxes after it too, so the run ends with the battle's
    /// screen; without it the window stays until the run's seconds end.
    close_result: bool,
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
    /// [--leave retreat|surrender] [--auto <turn>] [--auto-question cancel|retreat|surrender]
    /// [--close-result] [--sight fair|all-seeing]`.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let clients = assets::clients_dir();
        let env = std::env::var_os("H3_INSTALL").map(PathBuf::from);
        let (mut input, mut seed, mut speed, mut packs, mut install) = (None, 1, 0, None, None);
        let (mut scale, mut vsync, mut drawn_cursor, mut fullscreen) = (None, false, false, false);
        let (mut from, mut run, mut mute, mut humans) = (0, None, false, [false; 2]);
        let (mut leave, mut keys, mut auto) = (None, None, None);
        let (mut settings, mut speed_set, mut question) = (None, false, (false, false));
        let mut close_result = false;
        // The computer's sight (E17 M6 T5b1, *provisional*): the fair actor's by default.
        let mut sight = None;
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
                "--auto-question" => {
                    question = match value()?.as_str() {
                        "cancel" => (false, false),
                        "retreat" => (true, false),
                        "surrender" => (true, true),
                        v => {
                            return Err(format!(
                                "window: no answer `{v}` (cancel, retreat, surrender)"
                            ));
                        }
                    };
                }
                "--close-result" => close_result = true,
                "--sight" => {
                    sight = Some(match value()?.as_str() {
                        "fair" => Sight::Fair,
                        "all-seeing" => Sight::AllSeeing,
                        v => return Err(format!("window: no sight `{v}` (fair, all-seeing)")),
                    });
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
        if server.is_some() && sight.is_some() {
            return Err("window: a seat on a server plays against the server's computer".into());
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
                computer: Computer::Original(sight.unwrap_or_default()),
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
            question,
            close_result,
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
/// run's, a file named that does not exist yet too (E17 M6 T7b, R4-6); the file the window
/// writes them to (the one named, else `settings.toml` here); where they came from, with a line
/// per value the file's reading left out (R4-5).
fn load_settings(path: Option<&Path>) -> Result<(Settings, PathBuf, String), String> {
    let here = Path::new(SETTINGS_FILE);
    let read = path.or_else(|| here.exists().then_some(here));
    let text = read
        .map(|p| match fs::read_to_string(p) {
            Ok(text) => Ok(Some(text)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("{}: {e}", p.display())),
        })
        .transpose()?
        .flatten();
    let (settings, notes) = Settings::read(text.as_deref()).map_err(|e| {
        let from = read.map_or(String::new(), |p| p.display().to_string());
        format!("{from}: {e}")
    })?;
    let from = match (read, text.is_some()) {
        (Some(p), true) => p.display().to_string(),
        (Some(p), false) => format!("a first run's ({} not there yet)", p.display()),
        (None, _) => "a first run's".to_owned(),
    };
    let from = notes
        .iter()
        .fold(from, |from, note| format!("{from}; {note}"));
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
/// frame widened; `held`, at the end the battle holds at for the computer's cast (E17 M6 T5b2b,
/// [`After::Held`]): the frame shown before, made again when it is due.
#[derive(Debug)]
struct Frame {
    t: u32,
    list: Vec<Draw>,
    screen: Screen,
    xrgb: Vec<u32>,
    held: bool,
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

    /// The first change after `t` (none within [`LOOK_AHEAD_MS`]), and whether it is the end of
    /// the computer's cast for the human's hero the battle holds at (E17 M6 T5b2b, [`After`]).
    fn after(&mut self, battle: &mut Battle<'_>, t: u32) -> Result<Option<(u32, bool)>, String> {
        let limit = t.saturating_add(LOOK_AHEAD_MS);
        loop {
            let i = self.times.partition_point(|&c| c <= t);
            let next = self.times.get(i).copied();
            let grows = (battle.playing(), battle.timeline().end);
            match after(next, t, grows, || battle.cast_end()) {
                After::Grow => {
                    battle.grow_one()?;
                    self.grown(battle);
                }
                After::Held(end) => return Ok(Some((end, true))),
                After::Change(n) => return Ok(Some((n, false))),
                After::Further if self.until >= limit => return Ok(None),
                After::Further => {
                    let more = self.until.saturating_add(1);
                    self.reach(battle, more)?;
                }
            }
        }
    }
}

/// What [`Changes::after`] does with the first change `next` after `t`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum After {
    /// It gives it.
    Change(u32),
    /// The battle plays on first: what comes at or past its timeline's end is the next
    /// command's (E17 M5 T1b).
    Grow,
    /// It gives the timeline's end, the battle held there while the computer's cast for the
    /// human's hero it played last on a turn Auto Combat plays lasts (E17 M6 T5b2b, CLT-Q61,
    /// `Battle::cast_end`): the stack's action is asked only once the clock reaches that end, a
    /// key or a button pressed before the cast's end stopping Auto Combat there (`at < end`, as
    /// the comparisons'), its frame made then.
    Held(u32),
    /// None so far: the times are read further.
    Further,
}

/// [`After`] by the first change `next` after `t`, whether the battle is `playing` with its
/// timeline ending at `end`, and (asked only where it would play on) the end of the cast it
/// holds for (`cast_end`; one passed by `t` holds nothing).
fn after(
    next: Option<u32>,
    t: u32,
    (playing, end): (bool, u32),
    cast_end: impl FnOnce() -> Option<u32>,
) -> After {
    if playing && next.is_none_or(|n| n >= end) {
        let held = cast_end().is_some_and(|e| t < e);
        return if held { After::Held(end) } else { After::Grow };
    }
    next.map_or(After::Further, After::Change)
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
        held: false,
    })
}

/// The next frame after `shown`: the first later change that draws something else.
fn next_frame(
    battle: &mut Battle<'_>,
    changes: &mut Changes,
    (shown, holding): (&Frame, bool),
) -> Result<Option<Frame>, String> {
    // While the mouse holds a panel button the battle stands still too (E17 M7 T2, CLT-Q72 (a),
    // *provisional*): the button's own loop runs no idle step of the battle (`def_button_message`
    // 0x45641C, as read for the adventure map's, E24 design §4.13); only an input changes it.
    if holding {
        return Ok(None);
    }
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
    while let Some((next, held)) = changes.after(battle, t)? {
        // The end the battle holds at for the computer's cast (E17 M6 T5b2b): a frame to wake
        // at, made then.
        if held {
            return Ok(Some(Frame {
                t: next,
                list: shown.list.clone(),
                screen: shown.screen.clone(),
                xrgb: shown.xrgb.clone(),
                held,
            }));
        }
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
                held,
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
    // The options dialog open as the last pass ended (E17 M6 T4c, design §12.16).
    let mut options_open = false;
    let mut scrolls: Vec<(u32, i64)> = Vec::new();
    let mut held: Option<Held> = None;
    // A battle's own key that stopped Auto Combat, and the turn the stop comes at (E17 M5 T4c1;
    // its index, T8b2 R5c-4): it acts on that turn if a human's (0x474625).
    let mut stopped_by: Option<(Do, usize)> = None;
    // Either Control key held (E24 M3 T1b: a chord's Ctrl+), and whether it was pressed alone
    // (E17 M6 T7b, R5-1).
    let mut ctrl = Control::default();
    let mut report = Report::default();
    // An unattended run presses a human's commands itself (E17 M5 T2b).
    let plays = source.humans.contains(&true) || ground.content().is_some();
    let mut presser = (options.run.is_some() && plays).then(|| Presser {
        points: crate::pointer::press_points(battle.layout()),
        leave: options.leave,
        auto: options.auto,
        question: options.question,
        close_result: options.close_result,
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
                    question: options.question,
                    close_result: options.close_result,
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
        // the key, as with Auto Combat off (CLT-Q45 (2): the original's acts there). E17 M6
        // T5b2b (CLT-Q61): a stop read at the end of the computer's cast for the hero is that
        // turn's, the key acting there (*provisional*: read, not recorded).
        let cast_stop = stopped_by.and_then(|(_, turn)| battle.auto_stopped_at(turn));
        if let Some((command, turn)) = stopped_by
            && let Some((start, read)) = stop_start(&battle.timeline().turns, turn, cast_stop)
        {
            let last = battle.timeline().turns.len();
            let at_start = || read && battle.auto_at_start(turn);
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
                    // T opens the acting stack's stats dialog there (E17 M6 T7b, R5's question:
                    // the stop's key reaches the same key switch, 0x475D07).
                    Do::Stats => {
                        let at = clock.ms(win.ticks_ns()).max(start);
                        book_changed |= acting_stats(&mut battle, &mut inputs, at, pointer);
                    }
                    _ => {
                        let at = clock.ms(win.ticks_ns()).max(start);
                        // F over the battlefield hides the army info windows first (CLT-Q49,
                        // E17 M6 T4b2b1).
                        if let Some(hide) =
                            key_hides(battle.takes(at), battle.layout(), (at, pointer))
                        {
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
        let mut flush = Flush {
            last: box_before,
            emptied: false,
        };
        for event in win.wait(timeout) {
            // The options dialog's close by the event before counts before this one (E17 M6
            // T7b, R5-3): an enemy pressed or the dialog opened again in the same pass finds the
            // boxes changed, and under Auto Combat the turn handed over.
            grew |= battle.boxes_closed(clock.ms(win.ticks_ns()))?;
            // The modifier's state is no input to a box: a Control key's down the queue drops
            // still holds it (E17 M6 T7b, R5-6).
            ctrl.saw(&event);
            if flush.drops(box_shown(&battle), &event) {
                continue;
            }
            let now = win.ticks_ns();
            let at = clock.ms(now);
            // A key's release a dialog, a popup, the spell book or an aim takes (the branches
            // below) lets go of the panel button that key holds, without its click, in the
            // client's profile (CLT-Q71 (b); E17 M7 T1b): it is not left drawn pressed after.
            if let Event::KeyUp(key) = event
                && (battle.message_open()
                    || battle.stats_open()
                    || battle.options_open()
                    || battle.book_open()
                    || battle.aiming())
                && release_taken(&mut held, key, Bugs::Fixed)
            {
                mouse_changed = true;
            }
            match event {
                Event::Quit => break 'run,
                // Control held with a key makes another chord (E24 M3 T1b). Itself a key down, it
                // stops Auto Combat as any key does (T8b2, R5c-5; design §7.22), but only pressed
                // alone, at its release: a chord of the window's own (Ctrl+F5 to Ctrl+F8) stays
                // the window's (E17 M6 T7b, R5-1, *provisional*: a stop by Control alone comes
                // at its release, not as it goes down).
                Event::Key(Key::LeftCtrl | Key::RightCtrl) => ctrl.pressed(),
                Event::KeyUp(Key::LeftCtrl | Key::RightCtrl) => {
                    let context = Context::of(
                        battle.result_open(),
                        battle.message_open(),
                        (battle.stats_open(), battle.options_open()),
                        battle.book_open(),
                        battle.aiming(),
                    );
                    if ctrl.released()
                        && options.run.is_none()
                        && context == Context::Battle
                        && battle.stop_auto(at).is_some()
                    {
                        mouse_changed = true;
                    }
                }
                // Unattended, the keys do nothing but quit, whatever is open (the mouse still
                // moves the battle's; a human's commands are the presser's).
                Event::Key(key) if options.run.is_some() => {
                    let command = keys.command(Context::Battle, ctrl.held(), key.name());
                    if command == Some(Do::Window(Own::Quit)) {
                        break 'run;
                    }
                }
                Event::KeyUp(_) if options.run.is_some() => {}
                // The right button's release ends the own loop of the panel button the mouse
                // holds as the left's does, clicking it over it (E17 M7 T2, CLT-Q72 (a);
                // `def_button_message` 0x456338).
                Event::KeyUp(Key::RightButton) if mouse_holds(held) => {
                    if let Some(h) = held.take() {
                        let lists = (&mut inputs, &mut scrolls);
                        let (g, b) = let_go_held(&mut battle, lists, at, h)?;
                        (grew, book_changed, mouse_changed) = (grew | g, book_changed | b, true);
                    }
                }
                // A turret's info held by the right button over the battle (E17 M6 T2b2, design
                // §12.6), a control's help over the panel (T3a, §12.7) or a hero's info window
                // (T3b1, §12.8; `popup_run_held` 0x602A40): its ends (the right button's release, a left press or release)
                // close it; every other input is dropped, the moves too (the battle takes none
                // meanwhile, nor after: nothing is posted, 0x476BBF).
                Event::KeyUp(Key::RightButton) | Event::Click { down: true, .. }
                    if battle.popup_held() =>
                {
                    book_changed |= battle.popup_end();
                }
                // Over the combat options dialog the release a popup takes lets go of what the
                // mouse or that key holds there without its click (CLT-Q39's fix in the client's
                // profile; E17 M6 T7b, R4-2).
                Event::Click { down: false, .. } if battle.popup_held() => {
                    book_changed |= battle.options_let_go(None);
                    book_changed |= battle.popup_end();
                }
                Event::KeyUp(key) if battle.popup_held() => {
                    book_changed |= battle.options_let_go(Some(key.name()));
                }
                Event::Key(_) if battle.popup_held() => {}
                Event::Pointer(p) if battle.popup_held() => pointer = p,
                // The stats dialog's right button's description (E17 M6 T2a2b1, design §12.3),
                // held while the button is: the popup's loop (0x602A40) takes only its ends (the
                // right button's release, a left press or release) and drops every other input
                // (a key's release lets the OK it holds go in the client's profile, CLT-Q39); the
                // battle keeps where the mouse is.
                // The right button's stats dialog over the battle (E17 M6 T2b1) the same: its
                // end closes it, the battle taking the mouse again (`Input::Looked`).
                Event::KeyUp(Key::RightButton) | Event::Click { .. } if battle.stats_popup() => {
                    let did = battle.stats_held(at, true, None);
                    book_changed |= stats_did(&mut battle, &mut inputs, at, did);
                }
                Event::KeyUp(key) if battle.stats_popup() => {
                    let did = battle.stats_held(at, false, Some(key.name()));
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
                    let ctrl_held = ctrl.held();
                    // The tactics phase's panel's keys first, then the battle's (E17 M5 T4c2b).
                    let command = (context == Context::Battle && battle.in_tactics(at))
                        .then(|| keys.command(Context::Tactics, ctrl_held, key.name()))
                        .flatten()
                        .or_else(|| keys.command(context, ctrl_held, key.name()));
                    // While the mouse holds a panel button its own loop runs (E17 M7 T2, CLT-Q72
                    // (a); `def_button_message` 0x456338 to 0x45641C): every key and the right
                    // press are dropped, no popup opens; the viewer's own keys stay its own.
                    if mouse_holds(held) && !matches!(command, Some(Do::Window(_))) {
                        continue;
                    }
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
                            grew |= message_did(
                                &mut battle,
                                &mut inputs,
                                (at, pointer),
                                did,
                                &mut book_changed,
                            )?;
                        }
                        // The result window's OK (E17 M5 T4b1): down, its key's release closing
                        // the battle's screen.
                        Do::ResultOk => {
                            let did = battle.message_key(h3_view::message::Key::Ok, key.name());
                            grew |= message_did(
                                &mut battle,
                                &mut inputs,
                                (at, pointer),
                                did,
                                &mut book_changed,
                            )?;
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
                        // The acting stack's stats dialog (E17 M6 T2a2a).
                        Do::Stats => {
                            book_changed |= acting_stats(&mut battle, &mut inputs, at, pointer);
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
                        // F6 to F8, no test of the turn: on every turn, CLT-Q75 (a), E17 M7
                        // T1a): toggled, the frame drawn again (on a human's turn the mouse's
                        // hex shaded again, on another the acting stack's reach drawn until its
                        // action), the settings written at once.
                        Do::Shades(_) if !battle.takes_keys(at) => {}
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
                            if let Some(hide) =
                                key_hides(battle.takes(at), battle.layout(), (at, pointer))
                            {
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
                // The right button's release ends the hold of the box's button the mouse holds as
                // the left's does (its own loop; E17 M7 T2, CLT-Q72 (a)); nothing else.
                Event::KeyUp(Key::RightButton) if battle.message_open() => {
                    let did = battle.message_release();
                    grew |= message_did(
                        &mut battle,
                        &mut inputs,
                        (at, pointer),
                        did,
                        &mut book_changed,
                    )?;
                }
                Event::KeyUp(key) if battle.message_open() => {
                    let did = battle.message_key_up(key.name());
                    grew |= message_did(
                        &mut battle,
                        &mut inputs,
                        (at, pointer),
                        did,
                        &mut book_changed,
                    )?;
                }
                // The right button's release ends the hold of the OK or the cast button the mouse
                // holds as the left's does (E17 M7 T2, CLT-Q72 (a)); nothing else.
                Event::KeyUp(Key::RightButton) if battle.stats_open() => {
                    let did = battle.stats_release(at);
                    let changed = stats_did(&mut battle, &mut inputs, at, did);
                    (book_changed, mouse_changed) =
                        (book_changed | changed, mouse_changed | changed);
                }
                Event::KeyUp(key) if battle.stats_open() => {
                    let did = battle.stats_key_up(key.name());
                    let changed = stats_did(&mut battle, &mut inputs, at, did);
                    (book_changed, mouse_changed) =
                        (book_changed | changed, mouse_changed | changed);
                }
                // The right button's release ends a DEF button's hold by the mouse as the left's
                // does (E17 M7 T2, CLT-Q72 (a)); a bar or a box held, nothing.
                Event::KeyUp(Key::RightButton) if battle.options_open() => {
                    let did = battle.options_right_release();
                    let changed = options_did(&mut battle, &mut player, did, &files);
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
                        if HIDING_HOTKEYS.contains(&h.id)
                            && let Some(hide) =
                                key_hides(battle.takes(at), battle.layout(), (at, pointer))
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
                        // C opens the spell book (E17 M5 T3a), or its refusal's box (E17 M6 T5a).
                        if h.id == h3_view::panel::SPELL_BOOK {
                            book_changed |= opened(&mut inputs, at, battle.open_book(at));
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
                    grew |= message_did(
                        &mut battle,
                        &mut inputs,
                        (at, pointer),
                        did,
                        &mut book_changed,
                    )?;
                }
                Event::Click { down: false, .. } if battle.message_open() => {
                    let did = battle.message_release();
                    grew |= message_did(
                        &mut battle,
                        &mut inputs,
                        (at, pointer),
                        did,
                        &mut book_changed,
                    )?;
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
                    book_changed |= battle.stats_press(at, [x, y]) != StatsDid::Nothing;
                }
                Event::Click { down: false, .. } if battle.stats_open() => {
                    let did = battle.stats_release(at);
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
                        if let Some(h) = held.as_mut().filter(|h| h.key.is_none()) {
                            // The mouse's button follows it on or off the button it holds, whose
                            // own loop takes the move: the battle takes none until the release
                            // (E17 M7 T2, CLT-Q72 (a), *provisional*: its status line and cursor
                            // as the press left them, as read for the adventure map's).
                            h.over = battle.panel_at(at).button_at([x, y]) == Some(h.id);
                        } else {
                            // The battle takes the move now (inputs after a seek back are gone).
                            inputs.retain(|i| i.at() <= at);
                            inputs.push(Input::Move { at, to: [x, y] });
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
                    // An enabled panel button goes down and runs its own loop while the mouse
                    // holds it (E17 M7 T2, CLT-Q72 (a)); a disabled one takes no press
                    // (`def_button_message` 0x4562DD), its help the right button's still.
                    if let Some(id) = battle.panel_at(at).button_at([x, y]) {
                        if battle.enabled(at, id) {
                            held = Some(Held {
                                id,
                                key: None,
                                over: true,
                            });
                            mouse_changed = true;
                        }
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
                        let lists = (&mut inputs, &mut scrolls);
                        let (g, b) = let_go_held(&mut battle, lists, at, h)?;
                        (grew, book_changed, mouse_changed) = (grew | g, book_changed | b, true);
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
        // come, and under Auto Combat the turn waiting, play by them (a close by the pass's last
        // event; one before it counted before the next event, above).
        grew |= battle.boxes_closed(clock.ms(win.ticks_ns()))?;
        // The options dialog's close composes the background again with its options, without the
        // cursor shadow's call (E17 M6 T4c, design §12.16: oddity 2, CLT-Q52).
        let options_closed = options_open && !battle.options_open();
        if options_closed {
            inputs.push(Input::Shades {
                at: clock.ms(win.ticks_ns()),
                options: Shades::of(&battle.settings),
                keyed: false,
            });
            mouse_changed = true;
        }
        options_open = battle.options_open();
        // An aim with both shadows on turns the movement shadow off as it starts and on again as
        // it ends in the mouse's follow alone (`mouse::track`, `Options::aiming`; E17 M6 T4c,
        // design §12.16): the settings and their file keep the player's (E17 M6 T7b, R7-2:
        // *provisional*, the original's saves the switch each time, oddity 8, which a window
        // closed inside the aim would leave in the file). The block that copied it never ran: it
        // read the turn's aims, whose original numbers only `pointer::legal` names.
        // The options dialog set Creature Info's level (E17 M6 T4b2a): the hints read it from now.
        if battle.settings.army_info != army_info {
            army_info = battle.settings.army_info;
            inputs.push(Input::ArmyInfo {
                at: clock.ms(win.ticks_ns()),
                level: army_info,
            });
            mouse_changed = true;
        }
        // Then the mouse posted where it rests (CLT-Q43 (3), the client's profile; E17 M7 T1a):
        // the cursor, the line (at the level just set) and the shade those of the hex under it at
        // once, where the original's leave the dialog's arrow and the old hex's until the mouse
        // moves (oddity 10, the stills').
        if options_closed {
            post_mouse(&mut inputs, clock.ms(win.ticks_ns()), pointer);
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
        // A box of Eagle Eye's message opened (E17 M6 T5c): its pickup sound starts, category 3
        // as good luck's, on no stack.
        let pickup = battle.take_pickup();
        if let (Some(p), Some(lib)) = (player.as_mut(), library.as_ref()) {
            if let Some(n) = pickup {
                let cue = h3_view::anim::Cue {
                    at: battle_ms,
                    stack: 0,
                    sound: h3_view::anim::Sound::Pickup(n),
                    stop: false,
                    creature: None,
                };
                p.cue(lib, &cue);
            }
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
                if latest <= d.t && !d.held {
                    Some(d)
                } else {
                    // A later change due already; or the cast's end held (E17 M6 T5b2b): the
                    // battle played on now, the clock there, its frame made now.
                    report.skipped += usize::from(!d.held);
                    let late = frame_at(&mut battle, latest)?;
                    match shown.as_mut().filter(|s| s.list == late.list) {
                        Some(s) => {
                            s.t = latest;
                            next = next_frame(&mut battle, &mut changes, (s, mouse_holds(held)))?;
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
            next = next_frame(&mut battle, &mut changes, (&due, mouse_holds(held)))?;
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

/// What the spell book button's press at `at` led to (E17 M5 T3a): the book opened (the battle
/// takes no move until it closes), or a box tells why not (E17 M6 T5a, design §12.17: the mouse
/// posted after its OK, [`post_mouse`]). Whether the screen changed.
fn opened(inputs: &mut Vec<Input>, at: u32, opened: Opened) -> bool {
    match opened {
        Opened::No => false,
        Opened::Book => {
            inputs.push(Input::Book { at });
            true
        }
        Opened::Refused => true,
    }
}

/// A modal box or dialog over the battle closed at `at`, the mouse at `pointer`: the battle saw no
/// move while it was open; now the mouse's hex is forgotten and the mouse posted where it rests,
/// so the cursor and the hint are those of the hex under it. A refusal's box of the spell book
/// button or of the aim's entry (E17 M6 T5a, design §12.17: the button's path, 0x474F72, as
/// after a book closed without a spell); the combat options dialog in the client's profile
/// (CLT-Q43 (3), E17 M7 T1a).
fn post_mouse(inputs: &mut Vec<Input>, at: u32, pointer: Option<(i32, i32)>) {
    if let Some((x, y)) = pointer {
        inputs.push(Input::Move { at, to: [x, y] });
    }
    inputs.push(Input::Cancel { at });
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
        // A refusal's box over the battle (E17 M6 T5a): the mouse posted after its OK.
        BookDid::Refused => Ok(false),
    }
}

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

/// The open box as the window tells one from another (E17 M5 T8b2): its buttons' places, none
/// while no box is open. A box that gives way to another (the surrender refused) has others.
fn box_shown(battle: &Battle<'_>) -> BoxShown {
    use h3_view::message::{NO, OK, YES};
    // What the box asks matters here only as held or not.
    let asked = if battle.popup_held() {
        Asked::Held
    } else {
        Asked::Told
    };
    shown_box(
        battle
            .message_open()
            .then(|| (asked, [OK, YES, NO].map(|id| battle.message_button(id)))),
        battle.options_open(),
        battle.boxes_opened(),
    )
}

/// The box shown as the window tells one from another ([`box_shown`]).
type BoxShown = Option<([Option<[i32; 2]>; 3], u32)>;

/// The input queue emptied as a box opens in a pass of the window's loop (the modal loop
/// 0x602520; design §7.19): the box shown before the pass's next input, and whether one opened.
struct Flush {
    last: BoxShown,
    emptied: bool,
}

/// The Control keys (E24 M3 T1b: a chord's Ctrl+; E17 M6 T7b, R5-1 and R5-6): which are held,
/// whatever the input queue drops (a modifier's state is no input to a box), and whether one went
/// down, as the battle read it, with no other key down or press since (Control pressed alone,
/// which stops Auto Combat at its release).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Control {
    held: [bool; 2],
    alone: bool,
}

impl Control {
    /// `event` as it comes, before the queue may drop it: a Control key held or let go, any other
    /// key down or press ends Control alone.
    fn saw(&mut self, event: &Event) {
        match *event {
            Event::Key(k @ (Key::LeftCtrl | Key::RightCtrl)) => {
                self.held[usize::from(k == Key::RightCtrl)] = true;
            }
            Event::KeyUp(k @ (Key::LeftCtrl | Key::RightCtrl)) => {
                self.held[usize::from(k == Key::RightCtrl)] = false;
            }
            Event::Key(_) | Event::Click { down: true, .. } => self.alone = false,
            _ => {}
        }
    }

    /// Either Control key held.
    fn held(self) -> bool {
        self.held.contains(&true)
    }

    /// A Control key's down the battle read: Control alone from it.
    fn pressed(&mut self) {
        self.alone = true;
    }

    /// A Control key's release the battle read, after [`Self::saw`]: whether it ends Control
    /// pressed alone (neither held now).
    fn released(&mut self) -> bool {
        let alone = self.alone && !self.held();
        self.alone &= self.held();
        alone
    }
}

impl Flush {
    /// Whether `event` is dropped, the box `shown` as it comes: a press or a key down from the
    /// opening of a box in the pass on (the key-ups, the moves, the releases and Quit go on).
    fn drops(&mut self, shown: BoxShown, event: &Event) -> bool {
        self.emptied |= shown.is_some() && shown != self.last;
        self.last = shown;
        self.emptied && matches!(event, Event::Key(_) | Event::Click { down: true, .. })
    }
}

/// [`box_shown`] of the `message` box open (what it asks, its buttons' places), the options
/// dialog open or not (`options`) and the boxes `opened` so far. A held popup is not a box here:
/// what is under it is shown (E17 M6 T7b, R3-1: [`Asked::empties_input`]).
fn shown_box(
    message: Option<(Asked, [Option<[i32; 2]>; 3])>,
    options: bool,
    opened: u32,
) -> BoxShown {
    // The combat options dialog's opening flushes the input queue too (E17 M6 T4a).
    let options = options.then_some([None, None, Some([-1, -1])]);
    // Each opening a box of its own, though alike (E17 M6 T5c, the reviewer: Eagle Eye's).
    message
        .filter(|(asked, _)| asked.empties_input())
        .map(|(_, buttons)| buttons)
        .or(options)
        .map(|buttons| (buttons, opened))
}

/// What the combat options dialog `did` (E17 M6 T4a, design §12.12): its press samples played
/// (a speed's once, CLT-Q74), its volumes on the sound at once; at its close after a click its
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

/// When the key that stopped Auto Combat acts on the stop's turn `turn` (E17 M5 T4c1; E17 M6
/// T5b2b, CLT-Q61): at the end of the computer's cast its stop was read at (`cast_stop`,
/// `Battle::auto_stopped_at`: Auto Combat off there, no start to drop it), else at the turn's
/// start in `turns` once it started; with whether Auto Combat's state at that start may drop it
/// ([`carry`]'s `at_start`). None: the turn has not started.
fn stop_start(turns: &[(u32, u8)], turn: usize, cast_stop: Option<u32>) -> Option<(u32, bool)> {
    match cast_stop {
        Some(end) => Some((end, false)),
        None => (turn.checked_sub(1))
            .and_then(|i| turns.get(i))
            .map(|&(t, _)| (t, true)),
    }
}

/// The battle's keys that act on the turn Auto Combat's stop comes at when they stopped it, a
/// human's (0x474625, `battle_buttons`' key switch; E17 M5 T4c1, design §7.22): the log's arrows,
/// the Faerie Dragon's F and, since E17 M6 T4b2a, F5 (case 63, Creature Info's level), since
/// E17 M6 T4c F6 to F8 (the grid and the shadows), since E17 M6 T7b T (the switch's index 0,
/// 0x475D07: the acting stack's stats dialog; R5's question, read by the fixer of R5).
const fn carried(command: Do) -> bool {
    matches!(
        command,
        Do::LogUp | Do::LogDown | Do::DragonSpell | Do::ArmyInfo | Do::Shades(_) | Do::Stats
    )
}

/// The key T at `at` (E17 M6 T2a2a; `battle_buttons` 0x475D07): the acting stack's stats dialog,
/// the mouse at `pointer` (the park without one); the army info windows hidden first (0x475D1E;
/// E17 M6 T4b2b1). Whether it opened.
fn acting_stats(
    battle: &mut Battle<'_>,
    inputs: &mut Vec<Input>,
    at: u32,
    pointer: Option<(i32, i32)>,
) -> bool {
    let point = pointer.map_or(crate::pointer::PARK, |(x, y)| [x, y]);
    let opened = battle.look(at, point, true);
    if opened {
        inputs.push(Input::Hide { at, key: false });
        inputs.push(Input::Look { at });
    }
    opened
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
/// T4b2a: the key F5; T4c: F6 to F8), a failure in the report's lines.
fn save_settings(settings: &Settings, files: &SettingsFile<'_>) {
    if let Err(e) = write_whole(files.path, &settings.write()) {
        files
            .lines
            .borrow_mut()
            .push(format!("settings: {}: {e}", files.path.display()));
    }
}

/// Writes `text` to `path` whole or not at all (E17 M6 T7b, R4-5): into a sibling file first
/// (its name and `.tmp`), renamed over `path`, so that a kill, a crash or a full disk in between
/// leaves the old file; the sibling removed again when either step fails.
fn write_whole(path: &Path, text: &str) -> std::io::Result<()> {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".tmp");
    let sibling = path.with_file_name(name);
    fs::write(&sibling, text)
        .and_then(|()| fs::rename(&sibling, path))
        .inspect_err(|_| {
            let _ = fs::remove_file(&sibling);
        })
}

/// Where the combat options dialog's settings go (E17 M6 T4a): the file, the sound's library for
/// its press sample, the report's lines for a failed write.
struct SettingsFile<'a> {
    path: &'a Path,
    library: Option<&'a Library>,
    lines: &'a std::cell::RefCell<Vec<String>>,
}

/// The panel's buttons whose hotkey's release hides the army info windows first ([`key_hides`];
/// CLT-Q49 fixed in the client's profile, E17 M6 T4b2b1): C, S, R and O (F's is the dragon's
/// aim, not a button's). A, D and W hide nothing first: their actions' start does.
const HIDING_HOTKEYS: [u16; 4] = [
    h3_view::panel::SPELL_BOOK,
    h3_view::panel::SURRENDER,
    h3_view::panel::RETREAT,
    h3_view::panel::OPTIONS,
];

/// The hide one of the keys C, F, S, R and O gives at `at` with the mouse at `pointer` (E17 M6
/// T4b2b1, CLT-Q49 fixed in the client's profile, the window's; `h3_view::army_info::Hover::key`
/// for a still): over the battlefield of `layout` while the battle `takes` the key, as a left
/// press there hides the army info windows (*provisional*: whatever the key then does).
fn key_hides(
    takes: bool,
    layout: &h3_view::layout::BattleLayout,
    (at, pointer): (u32, Option<(i32, i32)>),
) -> Option<Input> {
    let (x, y) = pointer?;
    (takes && h3_view::army_info::on_field(layout, [x, y])).then_some(Input::Hide { at, key: true })
}

/// What a message box `did` at `at`, the mouse at `pointer` (E17 M5 T4a): a change to draw, or
/// the command its OK gave (Retreat, Surrender); whether the battle took a command.
fn message_did(
    battle: &mut Battle<'_>,
    inputs: &mut Vec<Input>,
    (at, pointer): (u32, Option<(i32, i32)>),
    did: MessageDid,
    changed: &mut bool,
) -> Result<bool, String> {
    *changed |= did != MessageDid::Nothing;
    // The result window closed into Eagle Eye's boxes (E17 M6 T5c, the reviewer): its clip
    // stands at the close's millisecond, not at the first box's draw.
    if did == MessageDid::Changed {
        battle.window_closed_at(at);
    }
    match did {
        MessageDid::Give(command) => give(battle, inputs, at, command),
        // Auto Combat's question or offer answered (E17 M6 T5b2a): the computer's next.
        MessageDid::Answer(ok) => battle.answer(at, ok),
        // A turret's info closed (E17 M6 T2b2): the battle takes the mouse again (0x476E34).
        MessageDid::Looked => {
            inputs.push(Input::Looked { at });
            Ok(false)
        }
        // A refusal's box closed (E17 M6 T5a): the mouse posted where it rests.
        MessageDid::Refused => {
            post_mouse(inputs, at, pointer);
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
    /// The answers to Auto Combat's question and offer (E17 M6 T5b2a: `--auto-question`).
    question: (bool, bool),
    /// The result window's OK pressed as a box's (E17 M7 T1d: `--close-result`).
    close_result: bool,
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
        // The result window stays until the run's end (E17 M5 T4b1: its frames read back too),
        // unless `--close-result` presses its OK as a box's (E17 M7 T1d), which ends the run.
        if battle.result_open() && !self.close_result {
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
        // Auto Combat's question and offer by `--auto-question` (E17 M6 T5b2a), Cancel or OK.
        let answer = match battle.message_asked() {
            Some(Asked::Question) => Some(self.question.0),
            Some(Asked::Offer) => Some(self.question.1),
            _ => None,
        };
        let ok = match answer {
            Some(false) => battle.message_button(h3_view::message::NO),
            _ => None,
        }
        .or_else(|| battle.message_button(h3_view::message::YES))
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
            MessageDid::Answer(ok) => battle.answer(now, ok),
            // The presser never presses a turret (E17 M6 T2b2): its box closed, the battle takes
            // the mouse again all the same.
            MessageDid::Looked => {
                inputs.push(Input::Looked { at: now });
                Ok(true)
            }
            // A refusal's box (E17 M6 T5a; the bot's casts are legal, so none comes): the mouse
            // posted where the battle last saw it.
            MessageDid::Refused => {
                post_mouse(inputs, now, None);
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
                if battle.open_book(now) != Opened::Book {
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

/// The release of `key` taken by a dialog, a popup, the spell book or an aim over the battle
/// (CLT-Q71, answered (b) at E17 M6's closure, the original's for the comparisons only): the
/// client's profile lets go of the panel button `held` that key holds, without its click, as
/// CLT-Q39's `let_go` does for the stats dialog's OK; the executable's leaves it held, drawn
/// pressed until another hotkey or press replaces it (its hotkey's key down returns at once,
/// 0x4565E7, so the release never reaches the panel; recorded at E17 M7 T1c, `stats-held`: the
/// DEF button keeps its pressed frame after the stats dialog's close). Whether it let one go.
fn release_taken(held: &mut Option<Held>, key: Key, bugs: Bugs) -> bool {
    bugs == Bugs::Fixed && held.take_if(|h| h.key == Some(key)).is_some()
}

/// A panel button held: by the mouse (`key` none; `over` while the mouse stays on it; only an
/// enabled one, [`Battle::enabled`]) or by a hotkey.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Held {
    id: u16,
    key: Option<Key>,
    over: bool,
}

/// Whether the mouse holds a panel button down (E17 M7 T2, CLT-Q72 (a)): its own loop runs
/// (`def_button_message` 0x45641C) until the left button's release or the right's, every key
/// (but the viewer's own) and the right press dropped, the moves only its own, the battle
/// standing still meanwhile. A button its hotkey holds runs none (0x4565E7 returns at once).
fn mouse_holds(held: Option<Held>) -> bool {
    held.is_some_and(|h| h.key.is_none())
}

/// The panel button `h` the mouse held let go at `at` (E17 M3 T6c-2), by the left button's
/// release or the right's (its own loop's ends, `def_button_message` 0x456451 and 0x456338; E17
/// M7 T2, CLT-Q72 (a)): released over it, a log's scroll button scrolls it (`lists`' scrolls),
/// Wait and Defend give theirs, Auto Combat switches on (E17 M5 T4c1), the spell book opens or
/// its refusal's box (E17 M5 T3a, M6 T5a), the combat options dialog opens (E17 M6 T4a),
/// Surrender and Retreat ask first (E17 M5 T4a); released off it, nothing. Whether the battle
/// grew, and whether a dialog opened.
fn let_go_held(
    battle: &mut Battle<'_>,
    (inputs, scrolls): (&mut Vec<Input>, &mut Vec<(u32, i64)>),
    at: u32,
    h: Held,
) -> Result<(bool, bool), String> {
    let (mut grew, mut opened_one) = (false, false);
    if !h.over {
        return Ok((grew, opened_one));
    }
    if let Some(by) = scroll_by(h.id).filter(|_| battle.scrolls(at)) {
        scrolls.push((at, by));
    }
    if let Some(command) = battle.button(at, h.id) {
        grew |= give(battle, inputs, at, command)?;
    }
    if h.id == h3_view::panel::AUTO_COMBAT {
        grew |= battle.switch_auto(at)?;
    }
    if h.id == h3_view::panel::SPELL_BOOK {
        opened_one |= opened(inputs, at, battle.open_book(at));
    }
    opened_one |= h.id == h3_view::panel::OPTIONS && battle.open_options(at);
    opened_one |= battle.ask_leave(at, h.id);
    Ok((grew, opened_one))
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
    fn a_release_a_dialog_takes_lets_go_of_its_key_s_button_in_the_client_s_profile() {
        // CLT-Q71 (b): D held (Defend drawn pressed), a press on a stack opens its stats dialog,
        // which takes D's release: the client's profile lets Defend go without its click, the
        // original's leaves it held (recorded at M7 T1c, `stats-held`).
        let by_d = Held {
            id: h3_view::panel::DEFEND,
            key: Some(Key::D),
            over: true,
        };
        let mut held = Some(by_d);
        assert!(!release_taken(&mut held, Key::D, Bugs::Original));
        assert_eq!(held, Some(by_d));
        // Another key's release, or a button the mouse holds: nothing let go.
        assert!(!release_taken(&mut held, Key::W, Bugs::Fixed));
        assert_eq!(held, Some(by_d));
        let by_mouse = Held { key: None, ..by_d };
        let mut mouse = Some(by_mouse);
        assert!(!release_taken(&mut mouse, Key::D, Bugs::Fixed));
        assert_eq!(mouse, Some(by_mouse));
        assert!(release_taken(&mut held, Key::D, Bugs::Fixed));
        assert_eq!(held, None);
        assert!(!release_taken(&mut held, Key::D, Bugs::Fixed));
    }

    #[test]
    fn a_panel_button_the_mouse_holds_runs_its_own_loop_to_either_release() {
        // E17 M7 T2 (CLT-Q72 (a)): only the mouse's hold runs the loop (a hotkey's returns at
        // once); let go off the button nothing happens, over it Defend's command is given, and
        // only once: the release that ends the loop, left or right, takes the hold.
        let by_mouse = Held {
            id: h3_view::panel::DEFEND,
            key: None,
            over: true,
        };
        assert!(mouse_holds(Some(by_mouse)));
        assert!(!mouse_holds(Some(Held {
            key: Some(Key::D),
            ..by_mouse
        })));
        assert!(!mouse_holds(None));
        let Some(source) = crate::battle::tests::staged("seq-auto-retreat.toml", [true, false])
        else {
            return;
        };
        let ground = Ground::load(&source).unwrap();
        let mut battle = Battle::new(&ground).unwrap();
        battle.grow(0).unwrap();
        let at = battle.timeline().end;
        assert!(battle.waiting() && battle.enabled(at, by_mouse.id));
        let (mut inputs, mut scrolls) = (Vec::new(), Vec::new());
        let off = Held {
            over: false,
            ..by_mouse
        };
        let lists = (&mut inputs, &mut scrolls);
        assert_eq!(let_go_held(&mut battle, lists, at, off), Ok((false, false)));
        assert!(battle.waiting() && inputs.is_empty());
        let lists = (&mut inputs, &mut scrolls);
        assert_eq!(
            let_go_held(&mut battle, lists, at, by_mouse),
            Ok((true, false))
        );
        assert_eq!(inputs, [Input::Act { at }]);
        assert!(scrolls.is_empty());
    }

    #[test]
    fn a_refusal_s_box_posts_the_mouse_after_its_ok() {
        // E17 M6 T5a (the review, design §12.17): the box is modal, so the button's path posts
        // the mouse (0x474F72) after its OK, where the mouse then rests; nothing as it opens.
        let mut inputs = Vec::new();
        assert!(opened(&mut inputs, 10, Opened::Refused));
        assert!(inputs.is_empty());
        assert!(!opened(&mut inputs, 10, Opened::No));
        assert!(opened(&mut inputs, 10, Opened::Book));
        assert_eq!(inputs, [Input::Book { at: 10 }]);
        inputs.clear();
        post_mouse(&mut inputs, 900, Some((320, 240)));
        let moved = Input::Move {
            at: 900,
            to: [320, 240],
        };
        assert_eq!(inputs, [moved, Input::Cancel { at: 900 }]);
        inputs.clear();
        post_mouse(&mut inputs, 900, None);
        assert_eq!(inputs, [Input::Cancel { at: 900 }]);
    }

    #[test]
    fn the_keys_c_f_s_r_and_o_hide_the_army_info_windows_over_the_battlefield_only() {
        // E17 M6 T7b (R6-3, CLT-Q49's fix): the panel's buttons whose default hotkeys are C, S,
        // R and O hide first; A, D, W, Space and the tactics phase's not.
        let hiding: Vec<u16> = (h3_view::keys::BUTTONS.iter())
            .filter(|(_, _, keys)| keys.iter().any(|k| ["C", "S", "R", "O"].contains(k)))
            .map(|&(id, _, _)| id)
            .collect();
        let mut listed = HIDING_HOTKEYS.to_vec();
        listed.sort_unstable();
        let mut hiding = hiding;
        hiding.sort_unstable();
        assert_eq!(listed, hiding);
        for id in [
            h3_view::panel::AUTO_COMBAT,
            h3_view::panel::DEFEND,
            h3_view::panel::WAIT,
        ] {
            assert!(!HIDING_HOTKEYS.contains(&id));
        }
        // Over the battlefield while the battle takes the key: the hide, a key's; off the
        // battlefield (the panel), not taking it, or the mouse out of the window: none.
        let layout =
            h3_view::layout::parse_battle(include_str!("../../../layouts/battle.toml")).unwrap();
        let hide = Some(Input::Hide { at: 7, key: true });
        assert_eq!(key_hides(true, &layout, (7, Some((400, 300)))), hide);
        assert_eq!(key_hides(true, &layout, (7, Some((400, 590)))), None);
        assert_eq!(key_hides(false, &layout, (7, Some((400, 300)))), None);
        assert_eq!(key_hides(true, &layout, (7, None)), None);
    }

    #[test]
    fn control_alone_stops_auto_combat_at_its_release_and_a_chord_never() {
        // E17 M6 T7b (R5-1, R5-6): Control's state as each event comes (`saw`), the battle
        // reading what the queue does not drop (`pressed`, `released`).
        let read = |ctrl: &mut Control, event: Event| {
            ctrl.saw(&event);
            match event {
                Event::Key(Key::LeftCtrl | Key::RightCtrl) => {
                    ctrl.pressed();
                    false
                }
                Event::KeyUp(Key::LeftCtrl | Key::RightCtrl) => ctrl.released(),
                _ => false,
            }
        };
        // Alone: the stop at its release, once.
        let mut ctrl = Control::default();
        assert!(!read(&mut ctrl, Event::Key(Key::LeftCtrl)) && ctrl.held());
        assert!(read(&mut ctrl, Event::KeyUp(Key::LeftCtrl)) && !ctrl.held());
        assert!(!read(&mut ctrl, Event::KeyUp(Key::LeftCtrl)));
        // Ctrl+F5 (the window's pause), and Control with a press: no stop.
        for other in [
            Event::Key(Key::F5),
            Event::Click {
                down: true,
                at: (1, 1),
            },
        ] {
            let mut ctrl = Control::default();
            read(&mut ctrl, Event::Key(Key::RightCtrl));
            read(&mut ctrl, other);
            assert!(ctrl.held());
            read(&mut ctrl, Event::KeyUp(Key::F5));
            assert!(!read(&mut ctrl, Event::KeyUp(Key::RightCtrl)) && !ctrl.held());
        }
        // Both held: alone until the last goes up; a move or another key's release ends nothing.
        let mut ctrl = Control::default();
        read(&mut ctrl, Event::Key(Key::LeftCtrl));
        read(&mut ctrl, Event::Key(Key::RightCtrl));
        read(&mut ctrl, Event::Pointer(Some((3, 3))));
        read(&mut ctrl, Event::KeyUp(Key::F5));
        assert!(!read(&mut ctrl, Event::KeyUp(Key::LeftCtrl)) && ctrl.held());
        assert!(read(&mut ctrl, Event::KeyUp(Key::RightCtrl)));
        // A down the queue drops (a box opened in the pass) still holds Control, so a chord after
        // it is Ctrl's, and is no stop at its release.
        let mut ctrl = Control::default();
        ctrl.saw(&Event::Key(Key::LeftCtrl));
        assert!(ctrl.held());
        assert!(!read(&mut ctrl, Event::KeyUp(Key::LeftCtrl)));
    }

    #[test]
    fn a_held_popup_empties_no_input() {
        // E17 M6 T7b (R3-1): `popup_run_held` 0x602A40 flushes nothing, so a press queued behind
        // a quick right click reaches the popup (its end) or the battle after it. A box does.
        let (none, ok) = ([None; 3], [Some([10, 20]), None, None]);
        let held = shown_box(Some((Asked::Held, none)), false, 3);
        assert_eq!(held, None);
        assert!(!Asked::Held.empties_input() && Asked::Turret.empties_input());
        let press = Event::Click {
            down: true,
            at: (5, 5),
        };
        // Over the battle: the right press opens the popup, its release closes it, the press
        // after it goes on.
        let mut flush = Flush {
            last: None,
            emptied: false,
        };
        assert!(!flush.drops(None, &Event::Key(Key::RightButton)));
        assert!(!flush.drops(held, &Event::KeyUp(Key::RightButton)));
        assert!(!flush.drops(None, &press));
        // Over the options dialog: its help held, then closed, shows the dialog alike.
        let options = shown_box(None, true, 3);
        assert_eq!(shown_box(Some((Asked::Held, none)), true, 3), options);
        let mut flush = Flush {
            last: options,
            emptied: false,
        };
        assert!(!flush.drops(options, &Event::Key(Key::RightButton)));
        assert!(!flush.drops(options, &press));
        // A box opening in the pass drops the presses and key downs after it, not the key ups.
        let told = shown_box(Some((Asked::Told, ok)), false, 4);
        assert_eq!(told, Some((ok, 4)));
        let mut flush = Flush {
            last: None,
            emptied: false,
        };
        assert!(!flush.drops(None, &press));
        assert!(flush.drops(told, &press));
        assert!(flush.drops(None, &Event::Key(Key::RightButton)));
        assert!(!flush.drops(None, &Event::KeyUp(Key::RightButton)));
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
            "--sight",
            "all-seeing",
        ]))
        .unwrap();
        // E17 M6 T5b1: the computer's sight, the fair actor's without `--sight`.
        assert_eq!(o.source.computer, Computer::Original(Sight::AllSeeing));
        assert_eq!(o.keys, Some(PathBuf::from("mine.toml")));
        assert_eq!(o.auto, Some(1));
        assert_eq!(o.source.seed, 7);
        assert_eq!(o.source.speed, Speed::new(2));
        assert!(o.source.scenario() && o.vsync && o.drawn_cursor && o.fullscreen && o.mute);
        assert_eq!((o.scale, o.from, o.run), (Some(1), 100, Some((20, true))));
        let m = Options::parse(&args(&["a/b.jsonl", "--measure", "5"])).unwrap();
        assert_eq!(m.source.computer, Computer::Original(Sight::Fair));
        assert!(Options::parse(&args(&["a", "--sight", "blind"])).is_err());
        // E17 M6 T7b (R9-3): the unattended answers to Auto Combat's question and offer, Cancel
        // without `--auto-question`.
        assert_eq!(m.question, (false, false));
        let answered =
            |v: &str| Options::parse(&args(&["a", "--auto-question", v])).map(|o| o.question);
        assert_eq!(answered("cancel"), Ok((false, false)));
        assert_eq!(answered("retreat"), Ok((true, false)));
        assert_eq!(answered("surrender"), Ok((true, true)));
        assert!(answered("x").is_err());
        // E17 M7 T1d: the result window left open without `--close-result`.
        assert!(!m.close_result);
        assert!(
            Options::parse(&args(&["a", "--close-result"]))
                .unwrap()
                .close_result
        );
        // A seat on a server plays against the server's computer.
        let seat = [
            "--server",
            "ws://127.0.0.1:1",
            "--game",
            "1",
            "--seat",
            "attacker",
        ];
        assert!(Options::parse(&args(&seat)).is_ok());
        let sighted = [&seat[..], &["--sight", "fair"]].concat();
        assert!(Options::parse(&args(&sighted)).is_err());
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
        // is read and written back; a bad one refused. E17 M6 T7b (R4-6): one named that is not
        // there yet a first run's, to be written there.
        let (settings, to, from) = load_settings(None).unwrap();
        assert_eq!(settings, Settings::FIRST_RUN);
        assert_eq!(
            (to.as_path(), from.as_str()),
            (Path::new(SETTINGS_FILE), "a first run's")
        );
        let missing = Path::new("no/such/settings.toml");
        let (settings, to, from) = load_settings(Some(missing)).unwrap();
        assert_eq!((settings, to.as_path()), (Settings::FIRST_RUN, missing));
        assert!(from.contains("not there yet"), "{from}");
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
        // A name it does not know is told and left out (R4-5); text that is not TOML refused,
        // a directory too.
        fs::write(&path, "[battle]\nmusic = 1\n").unwrap();
        let (read, _, from) = load_settings(Some(&path)).unwrap();
        assert_eq!(read, Settings::FIRST_RUN);
        assert!(
            from.ends_with("; battle: no setting `music`, left out"),
            "{from}"
        );
        fs::write(&path, "[battle\n").unwrap();
        assert!(load_settings(Some(&path)).is_err());
        assert!(load_settings(Some(&dir)).is_err());
        // Written whole: through a sibling renamed over it, none left (R4-5).
        write_whole(&path, "[battle]\nmusic_volume = 2\n").unwrap();
        assert_eq!(load_settings(Some(&path)).unwrap().0.music, 2);
        assert!(!dir.join("settings.toml.tmp").exists());
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
        // turn the stop comes at, as the arrows and F; a button's hotkey not. Since E17 M6 T7b
        // (R5's question, 0x475D07) the stats' T too: the stop's key reaches the same switch.
        for c in [
            Do::LogUp,
            Do::LogDown,
            Do::DragonSpell,
            Do::ArmyInfo,
            Do::Stats,
        ] {
            assert!(carried(c), "{c:?}");
        }
        for c in [Do::Button(h3_view::panel::WAIT), Do::StatsOk] {
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
    fn the_stop_s_key_acts_at_the_cast_s_end_or_at_the_turn_s_start() {
        // E17 M6 T5b2b (the reviewer, CLT-Q61): a stop read at the computer's cast's end (that
        // turn's, Auto Combat off there) acts at that end, no start to drop it; else at the
        // stop's turn's start once it started, Auto Combat's state then asked.
        let turns = [(300, 0), (900, 1)];
        assert_eq!(stop_start(&turns, 1, Some(600)), Some((600, false)));
        assert_eq!(stop_start(&turns, 2, None), Some((900, true)));
        assert_eq!(stop_start(&turns, 1, None), Some((300, true)));
        assert_eq!(stop_start(&turns, 3, None), None);
        assert_eq!(stop_start(&turns, 0, None), None);
    }

    #[test]
    fn the_battle_holds_at_the_computer_s_cast_s_end_until_the_clock_reaches_it() {
        // E17 M6 T5b2b (the reviewer, CLT-Q61): the cast (to 900) the battle's last action on a
        // turn Auto Combat plays: the change at its end is given held, the battle not played on,
        // from any time before it; from its end the next command comes first. Else a change
        // before the end is given, one at or past it plays on, none reads further.
        let held = || Some(900);
        assert_eq!(after(Some(900), 850, (true, 900), held), After::Held(900));
        assert_eq!(after(None, 100, (true, 900), held), After::Held(900));
        assert_eq!(after(Some(1000), 900, (true, 900), held), After::Grow);
        assert_eq!(after(Some(900), 850, (true, 900), || None), After::Grow);
        // A timeline ending after the cast: held to its end, unless the cast's end passed.
        assert_eq!(
            after(Some(1000), 850, (true, 1000), held),
            After::Held(1000)
        );
        assert_eq!(after(Some(1000), 950, (true, 1000), held), After::Grow);
        assert_eq!(
            after(Some(400), 300, (true, 900), || unreachable!()),
            After::Change(400)
        );
        assert_eq!(
            after(Some(950), 900, (false, 900), || unreachable!()),
            After::Change(950)
        );
        assert_eq!(
            after(None, 900, (false, 900), || unreachable!()),
            After::Further
        );
    }

    #[test]
    fn the_window_holds_at_the_computer_s_cast_and_its_stop_acts_at_the_cast_s_end() {
        // E17 M7 T1d (T7b's R10-2, CLT-Q61, `auto-cast-stop`): the loop's pieces over a battle.
        // A on the Champions' turn, Combat Auto Spells set: `Changes::after` walks the changes,
        // the battle played on, until the computer's Bless for the Marksmen holds it at the
        // cast's end, given held from any time before it with nothing played on; a press during
        // the cast stops Auto Combat there, and the loop's stop start (`cast_stop`) is that end,
        // no turn's start to drop the key.
        let Some(source) = crate::battle::tests::staged("seq-auto-cast.toml", [true, false]) else {
            return;
        };
        let ground = Ground::load(&source).unwrap();
        let mut battle = Battle::new(&ground).unwrap();
        battle.grow(0).unwrap();
        assert!(battle.switch_auto(battle.timeline().end).unwrap());
        let mut changes = Changes::new(&battle);
        let (mut t, mut walked) = (0, 0);
        let end = loop {
            match changes.after(&mut battle, t).unwrap() {
                Some((next, true)) => break next,
                Some((next, false)) if walked < 100_000 => (t, walked) = (next, walked + 1),
                other => panic!("no held cast's end after {t} ({walked} changes): {other:?}"),
            }
        };
        assert_eq!(battle.cast_end(), Some(end));
        let commands = battle.commands();
        assert_eq!(
            changes.after(&mut battle, end - 1).unwrap(),
            Some((end, true))
        );
        assert_eq!(battle.commands(), commands);
        let k = battle.stop_auto(end - 1).unwrap();
        battle.grow(end).unwrap();
        let cast_stop = battle.auto_stopped_at(k);
        assert_eq!(
            stop_start(&battle.timeline().turns, k, cast_stop),
            Some((end, false))
        );
    }

    #[test]
    fn an_unattended_run_closes_the_result_window_only_when_told() {
        // E17 M7 T1d (T7b's R10-2): the presser leaves the result window open, or with
        // `--close-result` presses its OK as a box's once it has shown for PRESS_AFTER_MS of
        // the window's clock, which closes the battle's screen.
        let Some(source) = crate::battle::tests::staged("seq-auto-retreat.toml", [true, false])
        else {
            return;
        };
        let ground = Ground::load(&source).unwrap();
        let ok = crate::spellbook::tests::one_button(h3_view::message::OK);
        for close in [false, true] {
            let mut battle = Battle::new(&ground).unwrap();
            crate::battle::tests::open_result(&mut battle, ok.clone());
            let mut presser = Presser {
                close_result: close,
                ..Presser::default()
            };
            let mut inputs = Vec::new();
            let wall = u64::from(PRESS_AFTER_MS);
            assert!(!presser.press(&mut battle, &mut inputs, 0, 0).unwrap());
            assert!(
                !presser
                    .press(&mut battle, &mut inputs, 0, wall - 1)
                    .unwrap()
            );
            let pressed = presser.press(&mut battle, &mut inputs, 0, wall).unwrap();
            assert_eq!((pressed, battle.result_closed()), (close, close));
            assert_eq!(battle.result_open(), !close);
        }
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
        // The sibling of a file that cannot be renamed over is gone again (R4-5).
        let mut sibling = dir.file_name().unwrap().to_os_string();
        sibling.push(".tmp");
        assert!(!dir.with_file_name(sibling).exists());
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
