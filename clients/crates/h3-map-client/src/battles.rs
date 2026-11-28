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
//! - **The battle** is hosted where the map hands it over: a walk's at its end (a plain cut), a
//!   visit's or a guards' answer's where the map's switch shows it whole (E24 M4 T5b2: the fades
//!   are the map's, from the battle's first frame, [`first_frame`], and to its last,
//!   [`Shown::last`]). Its frames are presented in the window as they fall due, its cursor the
//!   window's (drawn or the hardware's, as the map's), its sound on the host's
//!   ([`crate::audio::Player::lend`]), until the result window's OK closes its screen. An
//!   unattended run presses that OK itself (the battle's `Presses::close_result`).
//! - **The keys** are the battle window's (`h3_view::keys`: the user's file's `[battle]` table
//!   over the defaults, as `h3-view window`'s), the combat options' settings a first run's or
//!   `settings.toml`'s here (the file the options dialog writes), as `h3-view window` reads them:
//!   read again before each battle (E24 M4 T6, the reviewer's R5-10: one battle's options the
//!   next's), a failed write of them told as a line.
//!
//! - **Play's battles** (E24 M5 T6a, design §5.33, *provisional*): a battle of the viewer's in
//!   play is hosted live ([`host_game`]): the battle client's element over the session's seat
//!   (`battle::Ground::load_game`: the player's side human, the mouse's, or an unattended run's
//!   presses), loaded with the game's packs (the repository's, as play's rules,
//!   `h3_battle_client::maps::rules`); the host's pass (`window::Session::serve`) before each of
//!   the element's passes, which wait for no more than [`SERVE_MS`] while the seat plays (the
//!   in-process host answers at once); the keys and the settings as a record's.
//!
//! An adapter: it reads files and drives the window.

use std::fs;
use std::path::{Path, PathBuf};

use h3_battle_client::audio::HostSound;
use h3_battle_client::battle::{Ground, Source};
use h3_battle_client::game_seat::GameSeat;
use h3_battle_client::live::Computer;
use h3_battle_client::window::{Flow, Frame, Hosted, Prepared, Presses, Setup, Took};
use h3_render::screen::{Screen, Widening};
use h3_sdl::{Key, Window};
use h3_ui::cursor::Cursor;
use h3_ui::keys::Window as Own;
use h3_view::anim::Speed;
use h3_view::game_record::{self, GameBattle};
use h3_view::keys::Keys;
use h3_view::options::Settings;

/// The combat options' file looked for in the working directory (`h3-view window`'s).
const SETTINGS_FILE: &str = "settings.toml";

/// The longest a live battle's pass waits for input (E24 M5 T6a, *provisional*): the host serves
/// the seat before each pass, so its answers and the other side's turns come within it (the
/// element's own wait for a server's seat, `POLL_MS`, is the same).
pub const SERVE_MS: u32 = 10;

/// A record's battles and what each is loaded with.
#[derive(Debug)]
pub struct Battles {
    list: Vec<GameBattle>,
    source: Source,
    keys: Keys,
    /// The combat options' settings as last read, and their file.
    settings: Settings,
    settings_path: PathBuf,
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
        let mut battles = Self::of(record, packs, install, keys)?;
        battles.list = game.battles();
        Ok(battles)
    }

    /// Play's battles (E24 M5 T6a, design §5.33): none of a record's, each hosted live from the
    /// session's seat ([`host_game`]) with the packs of `packs` and the archives of `install`; the
    /// keys and the settings as [`Battles::read`]'s.
    ///
    /// # Errors
    /// The keys file or the settings unreadable.
    pub fn live(packs: &Path, install: &Path, keys: Option<&Path>) -> Result<Self, String> {
        // No record: the seat's battle is the session's (`Ground::load_game` reads no input).
        Self::of(Path::new(""), packs, install, keys)
    }

    /// The battles of `record` (none listed yet) loaded with `packs` and `install`, the keys of
    /// the file `keys` and the settings read.
    fn of(
        record: &Path,
        packs: &Path,
        install: &Path,
        keys: Option<&Path>,
    ) -> Result<Self, String> {
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
        let settings_path = PathBuf::from(SETTINGS_FILE);
        let settings = read_settings(&settings_path)?;
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
            list: Vec::new(),
            source,
            keys,
            settings,
            settings_path,
        })
    }

    /// The combat options' settings read again (E24 M4 T6, R5-10: the options dialog of the
    /// battle before may have written them); an unreadable file noted in `lines`, the settings
    /// read last kept.
    fn settings_now(&mut self, lines: &mut Vec<String>) -> Settings {
        match read_settings(&self.settings_path) {
            Ok(settings) => self.settings = settings,
            Err(e) => lines.push(format!("{e}; the settings read before kept")),
        }
        self.source.speed = Speed::new(self.settings.speed);
        self.settings
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

/// The combat options' settings of the file at `path` (a first run's where there is none).
fn read_settings(path: &Path) -> Result<Settings, String> {
    let shown = path.display();
    let text = path
        .exists()
        .then(|| fs::read_to_string(path))
        .transpose()
        .map_err(|e| format!("{shown}: {e}"))?;
    let (settings, _) = Settings::read(text.as_deref()).map_err(|e| format!("{shown}: {e}"))?;
    Ok(settings)
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
    /// Its last frame shown (E24 M4 T5b2: the switch's fade out fades it).
    pub last: Option<Screen>,
    /// Where the mouse stood over the battle's frame at its end (E24 M4 T6, R5-5: the map's
    /// mouse then; none outside the window).
    pub pointer: Option<(i32, i32)>,
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
/// unattended run's end), its sound on `sound`, the combat options' settings read again first;
/// the sound handed back (its battle's streams removed) and how it went. A battle that does not
/// load or start is noted in `lines` and skipped, its sound handed back; a failed write of the
/// settings is noted there too.
pub fn host(
    win: &mut Window,
    battles: &mut Battles,
    k: usize,
    sound: Option<HostSound>,
    run: Run,
    lines: &mut Vec<String>,
) -> Result<(Option<HostSound>, Shown), String> {
    let Some(battle) = battles.list.get(k).cloned() else {
        lines.push(format!("battle {}: not in the record", k + 1));
        return Ok((sound, Shown::default()));
    };
    let settings = battles.settings_now(lines);
    let ground = match Ground::load_battle(&battles.source, battle) {
        Ok(ground) => ground,
        Err(e) => {
            lines.push(format!("battle {}: {e}; not shown", k + 1));
            return Ok((sound, Shown::default()));
        }
    };
    let prepared = match Prepared::new(&ground) {
        Ok(prepared) => prepared,
        Err(e) => {
            lines.push(format!("battle {}: {e}; not shown", k + 1));
            return Ok((sound, Shown::default()));
        }
    };
    let setup = Setup {
        keys: battles.keys.clone(),
        settings,
        settings_path: battles.settings_path.clone(),
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
    let shown = pass(win, &mut hosted, run, lines, None);
    let sound = hosted.take_sound();
    lines.extend(hosted.take_unsaved());
    let mut shown = shown?;
    shown.outcome = hosted.outcome().map(|o| format!("{o:?}"));
    shown.last = hosted.shown().map(|f| f.screen.clone());
    shown.pointer = hosted.pointer();
    Ok((sound, shown))
}

/// Runs the viewer's battle in play (E24 M5 T6a, design §5.33) in `win` from the session's `seat`
/// to its closed result window (or the window's quit, or an unattended run's end), as [`host`]
/// runs a record's: `serve` (the session's host pass, `window::Session::serve`) before each of
/// its passes, the engine's refusals of the seat's commands noted in `lines`; the sound handed
/// back and how it went.
///
/// # Errors
/// The battle does not load or start (the engine's battle stays open: play cannot go on), a
/// frame does not show, the session's.
pub fn host_game(
    win: &mut Window,
    battles: &mut Battles,
    seat: GameSeat,
    serve: &mut dyn FnMut(&mut GameSeat) -> Result<Vec<String>, String>,
    (sound, run): (Option<HostSound>, Run),
    lines: &mut Vec<String>,
) -> Result<(Option<HostSound>, Shown), String> {
    let settings = battles.settings_now(lines);
    let ground = Ground::load_game(&battles.source, seat)
        .map_err(|e| format!("the battle does not load: {e}"))?;
    let prepared = Prepared::new(&ground).map_err(|e| format!("the battle does not load: {e}"))?;
    let setup = Setup {
        keys: battles.keys.clone(),
        settings,
        settings_path: battles.settings_path.clone(),
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
    let mut hosted = prepared
        .start(setup, &|| win.ticks_ns(), lines)
        .map_err(|not| format!("the battle does not start: {}", not.error))?;
    let mut refused = Vec::new();
    let mut served = || -> Result<(), String> {
        if let Some(seat) = ground.game_seat() {
            let why = serve(&mut seat.borrow_mut())?;
            refused.extend(
                why.into_iter()
                    .map(|w| format!("battle: the engine refused {w}")),
            );
        }
        Ok(())
    };
    let shown = pass(win, &mut hosted, run, lines, Some(&mut served));
    lines.append(&mut refused);
    let sound = hosted.take_sound();
    lines.extend(hosted.take_unsaved());
    let mut shown = shown?;
    shown.outcome = hosted.outcome().map(|o| format!("{o:?}"));
    shown.last = hosted.shown().map(|f| f.screen.clone());
    shown.pointer = hosted.pointer();
    Ok((sound, shown))
}

/// The host's passes (E17 design §13.2's order, `h3-view window`'s loop) until the battle's
/// screen closes, the window quits or an unattended run ends; since E24 M5 T6a a live battle's
/// host pass `serve` before each (its wait no longer than [`SERVE_MS`]).
fn pass(
    win: &mut Window,
    hosted: &mut Hosted<'_>,
    run: Run,
    lines: &mut Vec<String>,
    mut serve: Option<&mut dyn FnMut() -> Result<(), String>>,
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
    // A focus lost before the battle (E24 M4 T6, R5-5): its effects held from their start (the
    // shared music knows it: the map told the host's sound).
    let mut focused = win.focused();
    if !focused {
        hosted.focus(false);
    }
    loop {
        let now = win.ticks_ns();
        if run.stop_ns.is_some_and(|s| now >= s) {
            return Ok(shown);
        }
        let mut timeout = hosted.timeout(now);
        if let Some(serve) = serve.as_mut() {
            serve()?;
            timeout = timeout.min(SERVE_MS);
        }
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

/// Battle `k` of `battles`' first frame as its element draws it at its start (E24 M4 T5b2,
/// design §5.21, *provisional*: a switch's fade in): started unattended, with no sound, on a
/// stopped clock, the settings read at the window's start. None where it does not load or start
/// (noted in `lines`).
pub fn first_frame(
    battles: &Battles,
    k: usize,
    lines: &mut Vec<String>,
) -> Result<Option<Screen>, String> {
    let Some(battle) = battles.list.get(k).cloned() else {
        return Ok(None);
    };
    let name = format!("battle {}", k + 1);
    match Ground::load_battle(&battles.source, battle) {
        Ok(ground) => start_frame(battles, &ground, &name, lines),
        Err(e) => {
            lines.push(format!("{name}: {e}; its first frame not drawn"));
            Ok(None)
        }
    }
}

/// The first frame of the viewer's battle in play from the session's `seat` (E24 M5 T6a, design
/// §5.33: a switch's fade in), as [`first_frame`] draws a record's; none where it does not load or
/// start (noted in `lines`).
///
/// # Errors
/// The frame does not draw.
pub fn first_game_frame(
    battles: &Battles,
    seat: GameSeat,
    lines: &mut Vec<String>,
) -> Result<Option<Screen>, String> {
    let name = "the battle";
    match Ground::load_game(&battles.source, seat) {
        Ok(ground) => start_frame(battles, &ground, name, lines),
        Err(e) => {
            lines.push(format!("{name}: {e}; its first frame not drawn"));
            Ok(None)
        }
    }
}

/// The first frame of the battle `ground` holds (its `name` in `lines`), as its element draws it
/// at its start: unattended, no sound, a stopped clock, the settings read at the window's start.
fn start_frame(
    battles: &Battles,
    ground: &Ground,
    name: &str,
    lines: &mut Vec<String>,
) -> Result<Option<Screen>, String> {
    let prepared = match Prepared::new(ground) {
        Ok(prepared) => prepared,
        Err(e) => {
            lines.push(format!("{name}: {e}; its first frame not drawn"));
            return Ok(None);
        }
    };
    let setup = Setup {
        keys: battles.keys.clone(),
        settings: battles.settings,
        settings_path: battles.settings_path.clone(),
        from: 0,
        sound: None,
        install: battles.source.install.clone(),
        seed: battles.source.seed,
        unattended: true,
        presses: Some(Presses {
            close_result: true,
            ..Presses::default()
        }),
    };
    let mut hosted = match prepared.start(setup, &|| 0, lines) {
        Ok(hosted) => hosted,
        Err(not) => {
            lines.push(format!("{name}: {}; its first frame not drawn", not.error));
            return Ok(None);
        }
    };
    Ok(hosted.due(0)?.map(|frame| frame.screen))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_settings_read_again_before_a_battle() {
        // E24 M4 T6 (R5-10): a battle's options dialog writes the file; the next battle reads it,
        // the speed too; an unreadable file keeps the settings read before, noted.
        let dir = std::env::temp_dir().join(format!("h3-map-view-settings-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(SETTINGS_FILE);
        let _ = fs::remove_file(&path);
        let first = read_settings(&path).unwrap();
        assert_eq!(first, Settings::FIRST_RUN);
        let mut battles = Battles {
            list: Vec::new(),
            source: Source {
                input: dir.join("r.jsonl"),
                seed: 0,
                speed: Speed::new(first.speed),
                packs: dir.clone(),
                install: dir.clone(),
                humans: [false; 2],
                computer: Computer::Greedy,
                server: None,
                battle: 0,
            },
            keys: Keys::original(),
            settings: first,
            settings_path: path.clone(),
        };
        let faster = Settings {
            speed: (first.speed + 1) % 3,
            ..first
        };
        fs::write(&path, faster.write()).unwrap();
        let mut lines = Vec::new();
        assert_eq!(battles.settings_now(&mut lines), faster);
        assert_eq!(battles.source.speed, Speed::new(faster.speed));
        fs::write(&path, "battle = 3").unwrap();
        assert_eq!(battles.settings_now(&mut lines), faster);
        assert_eq!(lines.len(), 1);
        let _ = fs::remove_dir_all(&dir);
    }
}
