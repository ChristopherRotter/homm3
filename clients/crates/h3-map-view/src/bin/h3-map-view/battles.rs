//! A record's battles in the map's window (E24 M4 T5b1, *provisional*; `docs/epics/E24-design.md`
//! §5.20): the battle client's element (`h3_battle_client::window::Hosted`, E17 design §13.2)
//! run in the map viewer's window where the map hands a battle over, as the original's combat
//! screen takes over the adventure map's (design §5.19), and the map back where it left off.
//!
//! - **The battles** are the record's as its player saw them (`h3_view::game_record`:
//!   `GameRecord::battles`, each a record of its own, E17 design §13.3), each loaded as a whole
//!   game's battle (`battle::Ground::load_battle`: watched, no side played here; its packs from
//!   the record's packs folder, the archives from the installation). A walk's battle is the first
//!   whose snapshot comes after the `GameEvents` whose `BattleStarted` the walk began
//!   ([`Battles::began`]).
//! - **The switch** is a plain cut (T5b2 draws the original's fades): the battle's frames
//!   presented in the window as they fall due, its cursor the window's (drawn or the hardware's,
//!   as the map's), its sound on the host's ([`crate::audio::Player::lend`]), until the result
//!   window's OK closes its screen. An unattended run presses that OK itself (the battle's
//!   `Presses::close_result`).
//! - **The keys** are the battle window's (`h3_view::keys`: the user's file's `[battle]` table
//!   over the defaults, as `h3-view window`'s), the combat options' settings a first run's or
//!   `settings.toml`'s here (the file the options dialog writes), as `h3-view window` reads them.
//!
//! An adapter: it reads files and drives the window.

use std::fs;
use std::path::{Path, PathBuf};

use h3_battle_client::audio::HostSound;
use h3_battle_client::battle::{Ground, Source};
use h3_battle_client::live::Computer;
use h3_battle_client::window::{Flow, Frame, Hosted, Prepared, Presses, Setup, Took};
use h3_render::screen::Widening;
use h3_sdl::{Key, Window};
use h3_ui::cursor::Cursor;
use h3_ui::keys::Window as Own;
use h3_view::anim::Speed;
use h3_view::game_record::{self, GameBattle};
use h3_view::keys::Keys;
use h3_view::options::Settings;

/// The combat options' file looked for in the working directory (`h3-view window`'s).
const SETTINGS_FILE: &str = "settings.toml";

/// A record's battles and what each is loaded with.
#[derive(Debug)]
pub struct Battles {
    list: Vec<GameBattle>,
    source: Source,
    keys: Keys,
    settings: Settings,
}

impl Battles {
    /// The battles of the whole game's record at `record` (none for a record without any), to be
    /// loaded with the packs of `packs` and the archives of `install`; the battle's keys from the
    /// keys file `keys` (its `[battle]` table; none: the defaults).
    ///
    /// # Errors
    /// The record unreadable or not a whole game's, the keys file or the settings unreadable.
    pub fn read(
        record: &Path,
        packs: &Path,
        install: &Path,
        keys: Option<&Path>,
    ) -> Result<Self, String> {
        let text = fs::read_to_string(record).map_err(|e| format!("{}: {e}", record.display()))?;
        let game = game_record::parse(&text).map_err(|e| format!("{}: {e}", record.display()))?;
        let keys = match keys {
            Some(path) => {
                let text =
                    fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
                h3_ui::keys::Keys::with_file(&text)
                    .map_err(|e| format!("{}: {e}", path.display()))?
            }
            None => Keys::original(),
        };
        let known: Vec<&str> = Key::ALL.iter().map(|k| k.name()).collect();
        keys.check(&known)?;
        let here = Path::new(SETTINGS_FILE);
        let text = here
            .exists()
            .then(|| fs::read_to_string(here))
            .transpose()
            .map_err(|e| format!("{SETTINGS_FILE}: {e}"))?;
        let (settings, _) =
            Settings::read(text.as_deref()).map_err(|e| format!("{SETTINGS_FILE}: {e}"))?;
        let source = Source {
            input: record.to_path_buf(),
            seed: 0,
            speed: Speed::new(settings.speed),
            packs: packs.to_path_buf(),
            install: install.to_path_buf(),
            humans: [false; 2],
            // Unread: a record's battle is watched.
            computer: Computer::Greedy,
            server: None,
            battle: 0,
        };
        Ok(Self {
            list: game.battles(),
            source,
            keys,
            settings,
        })
    }

    /// How many battles the record holds.
    pub fn len(&self) -> usize {
        self.list.len()
    }

    /// The battle the record's `GameEvents` numbered `n` began (its place, from 0): the first
    /// whose snapshot comes after it.
    pub fn began(&self, n: u32) -> Option<usize> {
        self.list.iter().position(|b| b.at.opened > n)
    }
}

/// How a battle was hosted.
#[derive(Debug, Default)]
pub struct Shown {
    /// The frames presented.
    pub frames: usize,
    /// Its outcome once closed (none: not closed).
    pub outcome: Option<String>,
    /// The window quit during it (Esc over the battle, the window closed).
    pub quit: bool,
}

/// What the window's run asks of a hosted battle.
#[derive(Debug, Clone, Copy)]
pub struct Run {
    /// Unattended: the keys do nothing but quit, the result window's OK pressed.
    pub unattended: bool,
    /// The cursor drawn in the frame, not the hardware's.
    pub drawn_cursor: bool,
    /// The window's clock (ns) at which an unattended run stops.
    pub stop_ns: Option<u64>,
}

/// Runs battle `k` of `battles` in `win` to its closed result window (or the window's quit, or an
/// unattended run's end), its sound on `sound`; the sound handed back (its battle's streams
/// removed) and how it went. A battle that does not load or start is noted in `lines` and skipped.
pub fn host(
    win: &mut Window,
    battles: &Battles,
    k: usize,
    sound: Option<HostSound>,
    run: Run,
    lines: &mut Vec<String>,
) -> Result<(Option<HostSound>, Shown), String> {
    let Some(battle) = battles.list.get(k).cloned() else {
        lines.push(format!("battle {}: not in the record", k + 1));
        return Ok((sound, Shown::default()));
    };
    let ground = match Ground::load_battle(&battles.source, battle) {
        Ok(ground) => ground,
        Err(e) => {
            lines.push(format!("battle {}: {e}; not shown", k + 1));
            return Ok((sound, Shown::default()));
        }
    };
    let prepared = Prepared::new(&ground)?;
    let setup = Setup {
        keys: battles.keys.clone(),
        settings: battles.settings,
        settings_path: PathBuf::from(SETTINGS_FILE),
        from: 0,
        sound,
        install: battles.source.install.clone(),
        seed: battles.source.seed,
        unattended: run.unattended,
        presses: run.unattended.then_some(Presses {
            close_result: true,
            ..Presses::default()
        }),
    };
    let mut hosted = match prepared.start(setup, &|| win.ticks_ns(), lines) {
        Ok(hosted) => hosted,
        Err(not) => {
            lines.push(format!("battle {}: {}; not shown", k + 1, not.error));
            return Ok((not.sound, Shown::default()));
        }
    };
    let shown = pass(win, &mut hosted, run, lines);
    let sound = hosted.take_sound();
    let mut shown = shown?;
    shown.outcome = hosted.outcome().map(|o| format!("{o:?}"));
    Ok((sound, shown))
}

/// The host's passes (E17 design §13.2's order, `h3-view window`'s loop) until the battle's
/// screen closes, the window quits or an unattended run ends.
fn pass(
    win: &mut Window,
    hosted: &mut Hosted<'_>,
    run: Run,
    lines: &mut Vec<String>,
) -> Result<Shown, String> {
    let mut shown = Shown::default();
    let mut drawn_cursor = run.drawn_cursor;
    let mut frame_now = hosted.cursor_at(hosted.ms(win.ticks_ns())).0;
    let mut cursor_visible = true;
    if !drawn_cursor {
        let image = hosted.cursor_image(&frame_now)?.1;
        if win.set_cursor(Some(image)).is_err() {
            drawn_cursor = true;
        }
    }
    let mut focused = win.focused();
    loop {
        let now = win.ticks_ns();
        if run.stop_ns.is_some_and(|s| now >= s) {
            return Ok(shown);
        }
        let timeout = hosted.timeout(now);
        let mut redraw = false;
        if hosted.before(&|| win.ticks_ns(), lines)? == Flow::Again {
            continue;
        }
        let events = win.wait(timeout);
        for &has_focus in win.focus_changes() {
            if has_focus != focused {
                hosted.focus(has_focus);
                focused = has_focus;
            }
        }
        for event in events {
            match hosted.event(event, &|| win.ticks_ns())? {
                Took::Quit => {
                    shown.quit = true;
                    return Ok(shown);
                }
                Took::Redraw => redraw = true,
                Took::Window(Own::Fullscreen) => {
                    let on = !win.fullscreen();
                    win.set_fullscreen(on);
                }
                Took::Window(Own::Vsync) => {
                    let on = !win.vsync();
                    win.set_vsync(on);
                }
                Took::Window(Own::Cursor) => {
                    drawn_cursor = !drawn_cursor;
                    let image = if drawn_cursor {
                        None
                    } else {
                        Some(hosted.cursor_image(&frame_now)?.1)
                    };
                    if win.set_cursor(image).is_err() {
                        drawn_cursor = true;
                    }
                    redraw = true;
                }
                Took::Window(_) | Took::Nothing => {}
            }
            if hosted.take_moved() {
                redraw |= drawn_cursor;
            }
        }
        // The result window's OK closed the battle's screen: the map takes over.
        if hosted.closed() {
            return Ok(shown);
        }
        hosted.after(&|| win.ticks_ns())?;
        let (frame, visible) = hosted.cursor_at(hosted.ms(win.ticks_ns()));
        if (&frame, visible) != (&frame_now, cursor_visible) {
            let image = if visible && !drawn_cursor {
                Some(hosted.cursor_image(&frame)?.1)
            } else {
                None
            };
            (frame_now, cursor_visible) = (frame, visible);
            if drawn_cursor {
                redraw = true;
            } else if win.set_cursor(image).is_err() {
                drawn_cursor = true;
                redraw = true;
            }
        }
        if !drawn_cursor && !win.hardware_cursor() {
            lines.push("battle: hardware cursor lost; drawn instead".to_owned());
            win.set_cursor(None)?;
            drawn_cursor = true;
            redraw = true;
        }
        let battle_ms = hosted.sound(win.ticks_ns());
        let due = hosted.due(battle_ms)?;
        let pointer = (drawn_cursor && cursor_visible)
            .then_some(hosted.pointer())
            .flatten();
        if let Some(due) = due {
            let cursor = hosted.cursor_image(&frame_now)?.0;
            present(win, (&due, hosted.battle().widening()), pointer, &cursor)?;
            shown.frames += 1;
            hosted.showed(due)?;
        } else if redraw && hosted.shown().is_some() {
            let cursor = hosted.cursor_image(&frame_now)?.0;
            if let Some(f) = hosted.shown() {
                present(win, (f, hosted.battle().widening()), pointer, &cursor)?;
            }
        }
    }
}

/// Presents the battle's `frame`, with the drawn cursor at `pointer` if any (only its rectangle
/// widened again, by `widening`: `h3-view window`'s way).
fn present(
    win: &mut Window,
    (frame, widening): (&Frame, &Widening),
    pointer: Option<(i32, i32)>,
    cursor: &Cursor,
) -> Result<(), String> {
    match pointer {
        Some((x, y)) => {
            let mut screen = frame.screen.clone();
            cursor.draw(&mut screen, x, y);
            let mut widened = frame.xrgb.clone();
            widening.widen_rect(&screen, cursor.rect(x, y), &mut widened);
            win.present(&widened)
        }
        None => win.present(&frame.xrgb),
    }
}
