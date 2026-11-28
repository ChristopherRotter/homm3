//! Play's entry for the shell (E24 M5 T2b; the user's decision at M5 T0, "the shell on top", the
//! API agreed for the client lane at T2b): the game the shell began ([`Game`]: its map file, the
//! engine's validated setup, the hot seat's names, the turn's duration and the game's seed) played
//! in the shell's window with the shell's sound lent ([`play`]), and how play ended handed back
//! ([`End`]: the main menu, the load menu, the scenario again or the program's end, as E27 M5's
//! system menu will give them).
//!
//! - **What play shows until T3b** (*provisional*; the user at T2b and T3a): the game hosted
//!   in-process with a seat per player (`crate::hot_seat`, E24 M5 T3a: `GameSession::start`, the
//!   turn's duration its clock, the computers' turns ended at once), the record so far of the human
//!   whose turn runs set up as the record player sets up a record's file, the map standing at that
//!   start with M2's input (the window's, `crate::window`): every fact the engine's (the user at
//!   M5 T0: no rules table or map text read in play), the pictures the map file's. T3b gives the
//!   window End Turn through the session and the hand-over between the humans' views.
//! - **Its ends:** the window's quit key (and an unattended run's end) [`End::MainMenu`], the
//!   window closed [`End::Quit`]; [`End::Load`] and [`End::Restart`] wait for E27 M5's system menu.
//! - **The window lent** (800 × 600, the map's frame, at the host's scale): play sets the cursor
//!   and the key repeats it needs and turns text input off; the host sets its own again once play
//!   returned. **The sound lent:** the map's streams added on the host's sound and removed before
//!   play returns, an error's return too; none lent, play is silent.
//!
//! `h3-map-view play` ([`Options`], [`run`]) begins a game on a map file as the shell's Begin
//! would (the import's humans, or those `--humans` names; the turn's duration `--duration`'s) and
//! plays it in a window and on a sound of its own, until the shell calls [`play`] (the client
//! lane's move, its inbox); or, with `--turns`, headless: that many End Turns of the humans in turn
//! through the hosted session (E24 M5 T3a), a line for each. `--records` writes each human's record
//! at the end, as `h3-server record` names them, for the record player (`h3-map-view record`).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use h3_battle_client::audio::HostSound;
use h3_battle_client::game_seat::GameSeat;
use h3_contracts::PlayerId;
use h3_game::{GameRuleset, GameSetup};
use h3_map_view::engine::{GameRecord, Live};
use h3_protocol::wire::game::{
    AdventureCommand, Answer, ArmyStack, Decision, GameAnswer, GameCommand, GameQuery, Moves,
    Pos as WirePos,
};
use h3_render::Bugs;
use h3_sdl::{Config, Window};

use crate::files;
use crate::hot_seat::{self, HotSeat, Left};
use crate::paint::Seated;
use crate::window::{self, Host, Source};
use h3_map_view::clock::{ChatLine, Warning};

/// A game the shell began (its `begin::Game`, by the client lane's own type): what play needs of
/// it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    /// The map file the game was begun on (its pictures; its bytes' hash the game's manifest
    /// names).
    pub map: PathBuf,
    /// The engine's setup, validated (`GameSetup::validate`) under the rules play is given.
    pub setup: GameSetup,
    /// Each colour's human's name, red first (the hot seat's: T3's seats, T5's hand-over).
    pub names: [Option<String>; 8],
    /// The turn's duration (`turndur.txt`'s record, 0 to 10, 10 unlimited: the session's clock,
    /// [`hot_seat::clock`]).
    pub duration: u8,
    /// The game's seed: the same setup and seed begin the same game.
    pub seed: u64,
}

/// Where play reads its files and how it draws: the installation copy (its archives, sounds and
/// music), the map client's folder (`clients/map`: its tables and layouts, the string catalogue
/// beside it), the user's key file (its `[map]` table over the defaults; none: `keys.toml` in the
/// working directory when there is one) and the original's drawing bugs or the client's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Folders {
    pub install: PathBuf,
    pub map: PathBuf,
    pub keys: Option<PathBuf>,
    pub bugs: Bugs,
}

impl Folders {
    /// The installation copy `install`, the map client's folder beside this package (the build's
    /// path: a Windows build passes its own, the lane's TRAPS), no key file, the client's drawing.
    #[must_use]
    pub fn new(install: PathBuf) -> Self {
        Self {
            install,
            map: files::default_map(),
            keys: None,
            bugs: Bugs::Fixed,
        }
    }
}

/// How play ended: back to the main menu, to the load menu, the scenario begun again, or the
/// program's end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum End {
    MainMenu,
    Load,
    Restart,
    Quit,
}

/// Plays `game` (its setup validated under `rules`) in the host's `window` (opened at the map's
/// frame, 800 × 600) with the host's `sound` lent (taken and given back before the return, the
/// map's streams removed; none: silent), its files from `folders`; how play ended. What play
/// writes on its way (the window's report) goes to standard error.
///
/// # Errors
/// The game does not start, or its map or the client's files do not load: nothing shown then, the
/// sound given back.
pub fn play(
    game: &Game,
    rules: &GameRuleset,
    window: &mut Window,
    sound: &mut Option<HostSound>,
    folders: &Folders,
) -> Result<End, String> {
    let options = window::Options::for_play(game, folders, None, false);
    let mut hot = HotSeat::start(game, rules)?;
    let (lines, _, end) = played(game, rules, &mut hot, (window, sound), &options)?;
    for line in lines {
        eprintln!("h3-map-client play: {line}");
    }
    Ok(end)
}

/// `game` played as [`play`] plays it, hosted by `hot`, in the host's window and on its sound
/// with the window's `options` (E24 M5 T3b): each human's view in turn, set up from its record
/// since it was last left, until play ends in one of them; the views' reports (each headed by its
/// colour), whether every view's check held, and how play ended.
fn played(
    game: &Game,
    rules: &GameRuleset,
    hot: &mut HotSeat<'_>,
    (window, sound): (&mut Window, &mut Option<HostSound>),
    options: &window::Options,
) -> Result<(Vec<String>, bool, End), String> {
    let (mut lines, mut ok) = (Vec::new(), true);
    // Where each human's view was left, by player.
    let mut left: BTreeMap<usize, Left> = BTreeMap::new();
    // A hot seat begun with more than one human (E24 M5 T5, design §5.31): each human's turn
    // covered, its box naming it; the human whose view came last.
    let humans: Vec<(u8, Option<String>)> = hot
        .seats()
        .iter()
        .filter(|s| s.human)
        .map(|s| (s.colour, s.name.clone()))
        .collect();
    let mut last: Option<PlayerId> = None;
    // An unattended run's seconds count for the whole of play, not for each view.
    let started = window.ticks_ns();
    // A timed game's clock across the views (E24 M5 T5 part five, design §5.32): the turn's
    // limit; a hot seat's held from the game's start until the first cover's start.
    let limit = hot_seat::TURN_MINUTES
        .get(usize::from(game.duration))
        .filter(|&&m| m > 0)
        .map(|&m| u32::from(m) * 60_000);
    let mut kept = Kept::default();
    if humans.len() > 1 {
        kept.hold(started / 1_000_000);
    }
    let mut views = 0_usize;
    loop {
        let Some(player) = hot.turn() else {
            lines.push("no human's turn runs: play ends".into());
            return Ok((lines, ok, End::MainMenu));
        };
        let mut options = options.clone();
        // Each view's cues a file of its own (E24 M5 T6a).
        options.cues = options
            .cues
            .map(|p| PathBuf::from(format!("{}.{views}", p.display())));
        views += 1;
        if let Some((seconds, read)) = options.run {
            let spent = window.ticks_ns().saturating_sub(started) / 1_000_000_000;
            let rest = u64::from(seconds).saturating_sub(spent);
            if rest == 0 {
                lines.push("the unattended run's time is spent: play ends".into());
                return Ok((lines, ok, End::MainMenu));
            }
            options.run = Some((u32::try_from(rest).unwrap_or(seconds), read));
        }
        let colour = hot.seat(player).map_or(0, |s| s.colour);
        let seated = (humans.len() > 1).then(|| Seated {
            names: humans.clone(),
            sample: None,
            timed: game.duration != hot_seat::UNLIMITED,
            computers: computers_between(hot.seats(), last, player),
            framed: last.and_then(|p| hot.seat(p)).map(|s| s.colour),
        });
        last = Some(player);
        let mut viewed = Viewed {
            from: left.remove(&player.index()),
            hot: &mut *hot,
            map: &game.map,
            rules,
            player,
            colour,
            left: None,
            read: false,
            seated,
            limit,
            kept: &mut kept,
        };
        let host = Host::Lent {
            window: &mut *window,
            sound: &mut *sound,
        };
        let (view, view_ok, end) = window::window(&options, Source::Session(&mut viewed), host)?;
        lines.push(format!("colour {colour}'s view:"));
        lines.extend(view.into_iter().map(|line| format!("  {line}")));
        ok &= view_ok;
        match viewed.left.take() {
            Some(at) => {
                left.insert(player.index(), at);
            }
            None => return Ok((lines, ok, end)),
        }
    }
}

/// A human's view in the window hosted by the session (E24 M5 T3b, the window's
/// [`window::Session`]): the seat, where its view was set up from (where it was last left; none:
/// its whole record) and, once the turn passed from it, where it was left.
struct Viewed<'h, 'r> {
    hot: &'h mut HotSeat<'r>,
    map: &'h Path,
    rules: &'h GameRuleset,
    player: PlayerId,
    colour: u8,
    from: Option<Left>,
    left: Option<Left>,
    /// Whether the view read the clock yet.
    read: bool,
    /// The hot seat's hand-over (E24 M5 T5): none with one human.
    seated: Option<Seated>,
    /// A timed game's turn's limit, ms (E24 M5 T5 part five): none unlimited.
    limit: Option<u32>,
    /// What play keeps across the views (the host's clock held, the chat).
    kept: &'h mut Kept,
}

/// What play keeps from one human's view for the next (E24 M5 T5 part five, design §5.32,
/// *provisional*): the host's clock held while no human's turn runs on the screen (from the turn's
/// passing, by End Turn or by its time, to the next view's start of its timer after its cover:
/// the original starts the turn's timer there, `turn_timer_start` 0x4C78F0, while the session's
/// starts at the turn's passing: MCL-Q76) and while a human's battle runs (E24 M5 T6a, design
/// §5.33, MCL-Q75 (b): from its hand-over, `window::Session::battle`, to its return; the
/// session's clock times a battle against a monster on the turn's timer, the original's pauses
/// it for every battle and gives the time back), and the screen chat's lines as a view left
/// them, each by the host's time of its first draw (MCL-Q70).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Kept {
    /// The host's time a hold began, while it holds.
    since: Option<u64>,
    /// The host's time held so far.
    held: u64,
    /// The chat's lines, oldest first.
    chat: Vec<(Warning, Option<u64>)>,
}

impl Kept {
    /// The session's clock at the host's `now` (ms): the host's less the time held.
    fn clock(&self, now: u64) -> u64 {
        let holding = self.since.map_or(0, |since| now.saturating_sub(since));
        now.saturating_sub(self.held).saturating_sub(holding)
    }

    /// The session's clock held from the host's `now` (a hold under way kept).
    fn hold(&mut self, now: u64) {
        self.since.get_or_insert(now);
    }

    /// The session's clock running again from the host's `now`.
    fn release(&mut self, now: u64) {
        if let Some(since) = self.since.take() {
            self.held = self.held.saturating_add(now.saturating_sub(since));
        }
    }
}

/// Whether a computer's seat plays between the human `last` (none: the game's start) and
/// `player` in the seats' order (E24 M5 T5, design §5.31: its turn, ended at once by the host,
/// `MCL-STUB-5`, leaves the cover's four buttons disabled).
fn computers_between(seats: &[hot_seat::Seat], last: Option<PlayerId>, player: PlayerId) -> bool {
    let to = player.index();
    let between = |i: usize| match last.map(PlayerId::index) {
        None => i < to,
        Some(from) if from < to => from < i && i < to,
        Some(from) => i > from || i < to,
    };
    seats
        .iter()
        .enumerate()
        .any(|(i, s)| !s.human && between(i))
}

impl window::Session for Viewed<'_, '_> {
    fn map(&self) -> &Path {
        self.map
    }

    fn rules(&self) -> &GameRuleset {
        self.rules
    }

    fn colour(&self) -> u8 {
        self.colour
    }

    fn record(&self) -> Result<(GameRecord, Live), String> {
        self.hot.since(self.player, self.from.as_ref())
    }

    fn end_turn(&mut self, now: u64) -> Result<bool, String> {
        // The clock read first (the session's rule): a turn whose time ran out is not ended twice.
        self.hot.elapse(self.kept.clock(now))?;
        if self.hot.turn() == Some(self.player) {
            self.hot.end_turn()?;
        }
        Ok(self.runs(now))
    }

    fn tick(&mut self, now: u64) -> Result<bool, String> {
        let at = self.kept.clock(now);
        if !self.read || self.hot.deadline().is_some_and(|d| d <= at) {
            self.read = true;
            self.hot.elapse(at)?;
        }
        Ok(self.runs(now))
    }

    fn leave(&mut self, facings: Vec<(u16, u8)>) {
        self.left = self.hot.leave(self.player, facings);
    }

    fn moves(&self, hero: u16) -> Option<Moves> {
        match self.hot.query(self.player, GameQuery::Moves { hero })? {
            GameAnswer::Moves { moves, .. } => moves,
            _ => None,
        }
    }

    fn walk(&mut self, hero: u16, path: Vec<WirePos>) -> Result<Option<String>, String> {
        let command = AdventureCommand::MoveHero { hero, path };
        let refused = self.hot.play(GameCommand::Adventure { command })?;
        Ok(refused.map(|why| format!("{why:?}")))
    }

    fn decision(&self) -> Option<Decision> {
        self.hot.decision(self.player)
    }

    fn answer(&mut self, answer: Answer) -> Result<Option<String>, String> {
        let refused = self.hot.play(GameCommand::Answer { answer })?;
        Ok(refused.map(|why| format!("{why:?}")))
    }

    fn revisit(&mut self, hero: u16) -> Result<Option<String>, String> {
        let command = AdventureCommand::Revisit { hero };
        let refused = self.hot.play(GameCommand::Adventure { command })?;
        Ok(refused.map(|why| format!("{why:?}")))
    }

    fn seated(&self) -> Option<&Seated> {
        self.seated.as_ref()
    }

    fn clock(&self) -> Option<(u32, Option<u32>)> {
        let left = self
            .hot
            .seat(self.player)
            .and_then(|s| s.clock)
            .and_then(|c| c.turn)
            .map(|timer| timer.left_ms);
        self.limit.map(|limit| (limit, left))
    }

    fn chat(&mut self, now: u64) -> Vec<ChatLine> {
        let base = i64::try_from(now).unwrap_or(i64::MAX);
        std::mem::take(&mut self.kept.chat)
            .into_iter()
            .map(|(warning, drawn)| ChatLine {
                warning,
                drawn: drawn.map(|d| i64::try_from(d).unwrap_or(i64::MAX).saturating_sub(base)),
            })
            .collect()
    }

    fn keep_chat(&mut self, lines: Vec<ChatLine>, ms: u32, now: u64) {
        let base = i64::try_from(now).unwrap_or(i64::MAX) - i64::from(ms);
        self.kept.chat = lines
            .into_iter()
            .map(|l| {
                let drawn = l.drawn.map(|d| u64::try_from(base + d).unwrap_or(0));
                (l.warning, drawn)
            })
            .collect();
    }

    fn started(&mut self, now: u64) {
        self.kept.release(now);
    }

    fn battle(&mut self, now: u64) -> Option<GameSeat> {
        if self.hot.fighting() != Some(self.player) {
            return None;
        }
        let seat = self.hot.battle()?;
        // MCL-Q75 (b): the battle pauses the turn's timer (E24 M5 T6a, design §5.33).
        self.kept.hold(now);
        Some(seat)
    }

    fn army(&self, hero: u16) -> Option<Vec<ArmyStack>> {
        let view = self.hot.seat(self.player)?.view.as_ref()?;
        let hero = view.adventure.heroes.iter().find(|h| h.id == hero)?;
        hero.own.as_ref().map(|own| own.army.clone())
    }

    fn opening(&mut self) -> Option<GameSeat> {
        (self.hot.fighting() == Some(self.player))
            .then(|| self.hot.battle())
            .flatten()
    }

    fn serve(&mut self, seat: &mut GameSeat) -> Result<Vec<String>, String> {
        let refused = self.hot.serve(seat)?;
        Ok(refused.iter().map(|why| format!("{why:?}")).collect())
    }

    fn returned(&mut self, now: u64) -> Result<bool, String> {
        self.kept.release(now);
        self.hot.returned();
        Ok(self.runs(now))
    }
}

impl Viewed<'_, '_> {
    /// Whether the viewer's turn still runs at the host's `now`; once it passed to another human
    /// of a hot seat (End Turn or its time), the session's clock held until that human's view
    /// starts its timer (E24 M5 T5 part five).
    fn runs(&mut self, now: u64) -> bool {
        let runs = self.hot.turn() == Some(self.player);
        if !runs && self.seated.is_some() {
            self.kept.hold(now);
        }
        runs
    }
}

/// The command's options.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    map: PathBuf,
    seed: u64,
    humans: Option<Vec<u8>>,
    /// The turn's duration (`turndur.txt`'s record).
    duration: u8,
    scale: Option<u32>,
    mute: bool,
    folders: Folders,
    /// Unattended for this many seconds; whether each frame is read back.
    run: Option<(u32, bool)>,
    /// Headless: this many End Turns of the humans in turn, no window.
    turns: Option<u32>,
    /// Where each human's record is written at the end.
    records: Option<PathBuf>,
    /// Where each view's cues are written (E24 M5 T6a: the window's `--cues`, the view's number
    /// in play appended, `<file>.0`, `<file>.1`, …).
    cues: Option<PathBuf>,
}

impl Options {
    /// Parses `<map.h3m> [--seed <n>] [--humans <c>,...] [--duration <record>] [--keys <file>]
    /// [--scale <n>] [--mute] [--bugs fixed|original] [--install <dir>] [--map <dir>] [--check <s>
    /// | --measure <s> | --turns <n>] [--records <dir>] [--cues <file>]`.
    ///
    /// # Errors
    /// An unknown option, a bad value, no map file.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let (mut map, mut seed, mut humans, mut scale, mut mute) = (None, 0, None, None, false);
        let (mut keys, mut bugs, mut install, mut folder, mut run) =
            (None, Bugs::Fixed, None, None, None);
        let (mut duration, mut turns, mut records) = (hot_seat::UNLIMITED, None, None);
        let mut cues = None;
        let mut rest = args.iter();
        while let Some(arg) = rest.next() {
            let mut value = || {
                rest.next()
                    .cloned()
                    .ok_or_else(|| format!("play: `{arg}` needs a value"))
            };
            let number = |v: &str| {
                v.parse::<u64>()
                    .map_err(|_| format!("play: bad number `{v}` for `{arg}`"))
            };
            let small = |v: &str| {
                v.parse::<u32>()
                    .map_err(|_| format!("play: bad number `{v}` for `{arg}`"))
            };
            match arg.as_str() {
                "--seed" => seed = number(&value()?)?,
                "--humans" => humans = Some(colours(&value()?)?),
                "--duration" => {
                    let v = value()?;
                    duration = v
                        .parse::<u8>()
                        .ok()
                        .filter(|&d| d <= hot_seat::UNLIMITED)
                        .ok_or_else(|| format!("play: bad duration `{v}` (0 to 10)"))?;
                }
                "--turns" => turns = Some(small(&value()?)?),
                "--records" => records = Some(PathBuf::from(value()?)),
                "--cues" => cues = Some(PathBuf::from(value()?)),
                "--keys" => keys = Some(PathBuf::from(value()?)),
                "--scale" => scale = Some(small(&value()?)?.clamp(1, 16)),
                "--mute" => mute = true,
                "--bugs" => {
                    bugs = match value()?.as_str() {
                        "fixed" => Bugs::Fixed,
                        "original" => Bugs::Original,
                        _ => return Err("play: --bugs takes fixed or original".into()),
                    };
                }
                "--install" => install = Some(PathBuf::from(value()?)),
                "--map" => folder = Some(PathBuf::from(value()?)),
                "--check" => run = Some((small(&value()?)?, true)),
                "--measure" => run = Some((small(&value()?)?, false)),
                flag if flag.starts_with("--") => {
                    return Err(format!("play: unknown option `{flag}`"));
                }
                file if map.is_none() => map = Some(PathBuf::from(file)),
                extra => return Err(format!("play: unexpected `{extra}`")),
            }
        }
        if turns.is_some() && run.is_some() {
            return Err("play: --turns runs no window (no --check or --measure)".into());
        }
        Ok(Self {
            map: map.ok_or("play: needs a map file")?,
            seed,
            humans,
            duration,
            scale,
            mute,
            folders: Folders {
                install: install.unwrap_or_else(files::default_install),
                map: folder.unwrap_or_else(files::default_map),
                keys,
                bugs,
            },
            run,
            turns,
            records,
            cues,
        })
    }
}

/// The colours of `--humans`: 0 (red) to 7, comma-separated.
fn colours(list: &str) -> Result<Vec<u8>, String> {
    list.split(',')
        .map(|c| {
            c.trim()
                .parse::<u8>()
                .ok()
                .filter(|&c| c < 8)
                .ok_or_else(|| format!("play: bad colour `{c}` for `--humans` (0 to 7)"))
        })
        .collect()
}

/// The game a map file begins under `rules`, as the shell's Begin makes it
/// (`h3_battle_client::begin`): the map read and imported, its humans the import's or, given, only
/// those of `humans` (a colour the map gives the computer alone refused), validated by the engine;
/// no names, the turn's duration `duration` (`turndur.txt`'s record).
///
/// # Errors
/// The map does not read or import; a colour of `humans` the map gives the computer alone, or none
/// it holds; the engine's refusal of the setup.
pub fn begin(
    map: &Path,
    humans: Option<&[u8]>,
    (seed, duration): (u64, u8),
    rules: &GameRuleset,
) -> Result<Game, String> {
    let failed = |e: &dyn std::fmt::Display| format!("{}: {e}", map.display());
    let (_, raw) = crate::paint::read_map(map)?;
    let mut setup = h3_map::import(&raw, rules).map_err(|e| failed(&e))?;
    if let Some(humans) = humans {
        for &colour in humans {
            let player = setup.players.iter().find(|p| p.colour == colour);
            if !player.is_some_and(|p| p.human) {
                return Err(failed(&format_args!(
                    "colour {colour}: no player a human may play"
                )));
            }
        }
        for player in &mut setup.players {
            player.human &= humans.contains(&player.colour);
        }
    }
    setup.validate(rules).map_err(|e| failed(&e))?;
    Ok(Game {
        map: map.to_path_buf(),
        setup,
        names: Default::default(),
        duration,
        seed,
    })
}

/// Runs the command.
pub fn run(options: &Options) -> ExitCode {
    match play_command(options) {
        Ok((lines, ok, end)) => {
            for line in lines {
                println!("{line}");
            }
            println!("play ended: {end:?}");
            if ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(message) => {
            eprintln!("play: {message}");
            ExitCode::FAILURE
        }
    }
}

/// The command: the game begun and hosted; played headless (`--turns`) or in a window and on a
/// sound of its own lent to play as the shell lends its own; each human's record written where
/// `--records` says.
fn play_command(options: &Options) -> Result<(Vec<String>, bool, End), String> {
    let rules = h3_battle_client::maps::rules()?;
    let game = begin(
        &options.map,
        options.humans.as_deref(),
        (options.seed, options.duration),
        &rules,
    )?;
    let mut hot = HotSeat::start(&game, &rules)?;
    let (mut lines, ok, end) = match options.turns {
        Some(turns) => (headless(&mut hot, turns)?, true, End::MainMenu),
        None => in_window(options, &game, &rules, &mut hot)?,
    };
    if let Some(dir) = &options.records {
        lines.extend(write_records(&hot, &options.map, dir)?);
    }
    Ok((lines, ok, end))
}

/// `game`, hosted by `hot`, played in a window and on a sound of the command's own.
fn in_window(
    options: &Options,
    game: &Game,
    rules: &GameRuleset,
    hot: &mut HotSeat<'_>,
) -> Result<(Vec<String>, bool, End), String> {
    let [width, height] = files::load_layout(&options.folders.map)?.frame;
    let mut win = Window::open(&Config {
        title: format!("h3-map-view play — {}", options.map.display()),
        width,
        height,
        scale: options.scale,
        vsync: false,
        fullscreen: false,
    })?;
    let mut sound = if options.mute {
        None
    } else {
        HostSound::open(options.folders.bugs)
            .map_err(|e| eprintln!("play: no sound ({e})"))
            .ok()
    };
    let mut window_options =
        window::Options::for_play(game, &options.folders, options.run, options.mute);
    window_options.cues.clone_from(&options.cues);
    let (mut lines, ok, end) = played(game, rules, hot, (&mut win, &mut sound), &window_options)?;
    // The sound handed back (the shell's to use again), as play promises.
    lines.push(format!(
        "the sound handed back: {}",
        sound.as_ref().map_or("none", |_| "yes")
    ));
    Ok((lines, ok, end))
}

/// `turns` End Turns of the humans in turn through `hot`, headless (E24 M5 T3a): a line for the
/// start and one for each.
///
/// # Errors
/// [`HotSeat::end_turn`]'s.
pub fn headless(hot: &mut HotSeat<'_>, turns: u32) -> Result<Vec<String>, String> {
    let mut lines = vec![format!("start: {}", in_turn(hot))];
    for k in 1..=turns {
        let colour = hot.turn().and_then(|p| hot.seat(p)).map(|s| s.colour);
        hot.end_turn()?;
        let colour = colour.map_or_else(|| "none".into(), |c| c.to_string());
        lines.push(format!(
            "End Turn {k}: colour {colour} ended its turn; {}",
            in_turn(hot)
        ));
    }
    Ok(lines)
}

/// Whose turn runs in `hot`, and its day, as its seat's view has it.
fn in_turn(hot: &HotSeat<'_>) -> String {
    let Some(seat) = hot.turn().and_then(|p| hot.seat(p)) else {
        return "no human's turn runs".into();
    };
    let day = seat.view.as_ref().map_or(0, |v| v.day);
    let name = seat
        .name
        .as_deref()
        .map_or_else(String::new, |n| format!(" ({n})"));
    format!("colour {}{name}'s turn, day {day}", seat.colour)
}

/// Each human's record so far in `hot` written into `dir` as `h3-server record` names a game's on
/// the map file `map` (`<stem>.player-<k>.jsonl`): a line for each.
///
/// # Errors
/// The folder or a file cannot be written.
pub fn write_records(hot: &HotSeat<'_>, map: &Path, dir: &Path) -> Result<Vec<String>, String> {
    fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let stem = map
        .file_stem()
        .map_or_else(|| "game".into(), |s| s.to_string_lossy().into_owned());
    let mut lines = Vec::new();
    for seat in hot.seats().iter().filter(|s| s.human) {
        let k = seat.player.index();
        let text = hot
            .record(seat.player)
            .ok_or_else(|| format!("no record of player {k}"))?;
        let file = dir.join(format!("{stem}.player-{k}.jsonl"));
        fs::write(&file, text).map_err(|e| format!("{}: {e}", file.display()))?;
        lines.push(format!(
            "the record of colour {}: {}",
            seat.colour,
            file.display()
        ));
    }
    Ok(lines)
}

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::BTreeMap;

    use h3_map_view::engine::parse;
    use h3_map_view::situation::Situation;

    use super::*;

    /// A writer's map of the situation `text` in a file of its own (`name`), removed when dropped.
    pub(crate) struct MapFile(pub(crate) PathBuf);

    impl MapFile {
        pub(crate) fn new(name: &str, text: &str) -> Self {
            Self::with(name, text, &[])
        }

        /// The map of the situation `text` whose given objects' templates are among
        /// `templates` (`Objects.txt`'s lines: [`template`]).
        pub(crate) fn with(name: &str, text: &str, templates: &[&str]) -> Self {
            let templates: Vec<_> = templates.iter().map(|line| template(line)).collect();
            let (map, extras) = Situation::parse(text)
                .unwrap()
                .build(&BTreeMap::new(), &templates)
                .unwrap();
            // Compressed, as the game keeps its maps (the record player reads them so).
            let bytes = h3_map::writer::gzip(&h3_map::writer::write_with(&map, &extras).unwrap());
            let dir =
                std::env::temp_dir().join(format!("h3-map-client-play-{}", std::process::id()));
            fs::create_dir_all(&dir).unwrap();
            let path = dir.join(format!("{name}.h3m"));
            fs::write(&path, bytes).unwrap();
            Self(path)
        }
    }

    impl Drop for MapFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    /// An object's template from its line of `Objects.txt` (written here: no installation read).
    pub(crate) fn template(line: &str) -> h3_map::writer::ObjectTemplate {
        h3_map::writer::ObjectTemplate::from_objects_txt(line).unwrap()
    }

    /// Red's hero and Blue's on a small map.
    pub(crate) const TWO: &str = "size = 36\nterrain = \"grass\"\n\
        heroes = [{ at = [3, 3] }, { at = [30, 30], owner = 1 }]\nview = [0, 0, 0]\n";

    /// A wandering Peasants' template (class 54, creature 139): its anchor's tile blocked and
    /// visited, as `Objects.txt`'s monsters (`h3_map`'s tests' Gnolls).
    pub(crate) const PEASANTS: &str = "AVWpeas.def \
        011111111111111111111111111111111111111111111111 \
        100000000000000000000000000000000000000000000000 111111111 000000001 54 139 2 0";

    /// E24 M5 T6a's battle (as `clients/map/situations/seq-battle-won.toml`): Red's hero on
    /// (5, 5) with the map's `army` ([creature, count] by slot), a second hero of Red's far off
    /// (Red keeps one when the first is lost), `count` savage Peasants that never flee on (5, 4)
    /// (a step north), Blue's hero far off.
    pub(crate) fn peasants(army: &str, count: u16) -> String {
        format!(
            "size = 36\nterrain = \"grass\"\n\
             heroes = [{{ at = [5, 5], class = 2, map_army = {army} }}, \
             {{ at = [30, 5], class = 5 }}, {{ at = [32, 32], owner = 1 }}]\n\
             view = [0, 0, 0]\n\
             objects = [{{ def = \"AVWpeas.def\", at = [5, 4], \
             monster = {{ count = {count}, disposition = 4, never_flees = true }} }}]\n"
        )
    }

    #[test]
    fn a_map_begins_with_the_import_s_humans_or_those_named() {
        let rules = h3_battle_client::maps::rules().unwrap();
        let file = MapFile::new("humans", TWO);
        let humans = |game: &Game| -> Vec<u8> {
            game.setup
                .players
                .iter()
                .filter(|p| p.human)
                .map(|p| p.colour)
                .collect()
        };
        let unlimited = (5, hot_seat::UNLIMITED);
        let game = begin(&file.0, None, unlimited, &rules).unwrap();
        assert_eq!(humans(&game), [0, 1]);
        assert_eq!((game.seed, game.duration), unlimited);
        assert_eq!(game.names, <[Option<String>; 8]>::default());
        let game = begin(&file.0, Some(&[1]), (5, 3), &rules).unwrap();
        assert_eq!((humans(&game), game.duration), (vec![1], 3));
        let e = begin(&file.0, Some(&[2]), unlimited, &rules).unwrap_err();
        assert!(e.ends_with("colour 2: no player a human may play"), "{e}");
        let e = begin(&file.0.with_extension("none"), None, unlimited, &rules).unwrap_err();
        assert!(e.contains("none"), "{e}");
    }

    #[test]
    fn play_starts_with_the_record_of_the_human_in_turn() {
        let rules = h3_battle_client::maps::rules().unwrap();
        let file = MapFile::new("start", TWO);
        let bytes = fs::read(&file.0).unwrap();
        for (humans, id) in [(None, 0), (Some(&[1][..]), 1)] {
            let game = begin(&file.0, humans, (9, hot_seat::UNLIMITED), &rules).unwrap();
            let hot = HotSeat::start(&game, &rules).unwrap();
            let player = hot.turn().unwrap();
            assert_eq!(hot.seat(player).unwrap().colour, id);
            // Its whole record, as the record player reads it (its map the file's by its hash),
            // its start the human's view.
            let (record, live) = hot.since(player, None).unwrap();
            // Its facts: no facing yet, the client's picks seeded (E24 M5 T4b).
            assert_eq!((live.facings.len(), live.picks.is_some()), (0, true));
            assert_eq!(
                Some(&record),
                parse(&hot.record(player).unwrap()).ok().as_ref()
            );
            let loaded = h3_map_view::engine::live(record.clone(), &bytes, &rules, live).unwrap();
            assert_eq!(loaded.first().unwrap().viewer, id);
            // The same setup and seed begin the same game.
            let again = HotSeat::start(&game, &rules).unwrap();
            assert_eq!(again.since(player, None).unwrap().0, record);
        }
    }

    #[test]
    fn headless_turns_pass_between_the_humans_and_their_records_are_written() {
        let rules = h3_battle_client::maps::rules().unwrap();
        let file = MapFile::new("headless", TWO);
        let mut game = begin(&file.0, None, (3, hot_seat::UNLIMITED), &rules).unwrap();
        game.names[1] = Some("Blue's".into());
        let mut hot = HotSeat::start(&game, &rules).unwrap();
        let lines = headless(&mut hot, 3).unwrap();
        assert_eq!(
            lines,
            [
                "start: colour 0's turn, day 1",
                "End Turn 1: colour 0 ended its turn; colour 1 (Blue's)'s turn, day 1",
                "End Turn 2: colour 1 ended its turn; colour 0's turn, day 2",
                "End Turn 3: colour 0 ended its turn; colour 1 (Blue's)'s turn, day 2",
            ]
        );
        let dir = file.0.with_extension("records");
        let written = write_records(&hot, &file.0, &dir).unwrap();
        assert_eq!(written.len(), 2);
        let bytes = fs::read(&file.0).unwrap();
        for k in 0..2_u8 {
            let path = dir.join(format!("headless.player-{k}.jsonl"));
            let text = fs::read_to_string(&path).unwrap();
            assert_eq!(Some(text.clone()), hot.record(PlayerId::new(k)));
            let loaded = h3_map_view::engine::load(&text, &bytes, &rules).unwrap();
            assert_eq!(loaded.first().unwrap().viewer, k);
        }
        fs::remove_dir_all(&dir).unwrap();
    }

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|&a| a.to_owned()).collect()
    }

    #[test]
    fn the_command_takes_a_map_and_its_options() {
        let o = Options::parse(&args(&[
            "m.h3m",
            "--seed",
            "7",
            "--humans",
            "0, 1",
            "--duration",
            "4",
            "--scale",
            "2",
            "--mute",
            "--bugs",
            "original",
            "--install",
            "i",
            "--map",
            "d",
            "--keys",
            "k.toml",
            "--check",
            "5",
        ]))
        .unwrap();
        assert_eq!(o.map, PathBuf::from("m.h3m"));
        assert_eq!(
            (o.seed, o.humans.as_deref(), o.duration, o.scale),
            (7, Some(&[0, 1][..]), 4, Some(2))
        );
        assert!(o.mute);
        assert_eq!(o.run, Some((5, true)));
        assert_eq!(
            o.folders,
            Folders {
                install: PathBuf::from("i"),
                map: PathBuf::from("d"),
                keys: Some(PathBuf::from("k.toml")),
                bugs: Bugs::Original,
            }
        );
        assert_eq!((o.turns, o.records), (None, None));
        let o = Options::parse(&args(&["m.h3m", "--measure", "3"])).unwrap();
        assert_eq!((o.seed, o.humans, o.run), (0, None, Some((3, false))));
        assert_eq!(o.duration, hot_seat::UNLIMITED);
        let o = Options::parse(&args(&["m.h3m", "--turns", "6", "--records", "r"])).unwrap();
        assert_eq!((o.turns, o.records), (Some(6), Some(PathBuf::from("r"))));
        let o = Options::parse(&args(&["m.h3m", "--cues", "c.txt"])).unwrap();
        assert_eq!(o.cues, Some(PathBuf::from("c.txt")));
    }

    #[test]
    fn a_computer_s_seat_between_two_humans() {
        // E24 M5 T5 (design §5.31): a computer's seat between the human before and this one in
        // the seats' order, round from the last; before the first human at the game's start.
        let seat =
            |index: u8, human: bool| hot_seat::Seat::new(PlayerId::new(index), index, human, None);
        let seats = [seat(0, true), seat(1, true), seat(2, false)];
        let p = PlayerId::new;
        assert!(!computers_between(&seats, Some(p(0)), p(1)));
        assert!(computers_between(&seats, Some(p(1)), p(0)));
        assert!(!computers_between(&seats, None, p(0)));
        let first = [seat(0, false), seat(1, true)];
        assert!(computers_between(&first, None, p(1)));
        // A lone human's next turn after the computer's.
        assert!(computers_between(&first, Some(p(1)), p(1)));
    }

    #[test]
    fn a_battle_pauses_the_turn_s_timer_and_gives_its_time_back() {
        // E24 M5 T6a (MCL-Q75 (b), design §5.33): a minute's turn, read first at the host's 1 s;
        // Red's walk onto the Peasants at 10 s begins a battle, handed over then and played
        // through the view's session (the clock read meanwhile) until its return at 50 s: the
        // 40 s are given back, Red's turn ends at the host's 101 s (the session's 61 s), not 61 s.
        use crate::hot_seat::tests::{fight, hero, strike};
        use crate::window::Session as _;
        let rules = h3_battle_client::maps::rules().unwrap();
        let file = MapFile::with("timed", &peasants("[[2, 5]]", 15), &[PEASANTS]);
        let game = begin(&file.0, None, (4, 0), &rules).unwrap();
        let mut hot = HotSeat::start(&game, &rules).unwrap();
        let red = PlayerId::new(0);
        let (ours, at) = hero(&hot, red);
        let mut kept = Kept::default();
        let mut viewed = Viewed {
            hot: &mut hot,
            map: &game.map,
            rules: &rules,
            player: red,
            colour: 0,
            from: None,
            left: None,
            read: false,
            seated: None,
            limit: Some(60_000),
            kept: &mut kept,
        };
        assert!(viewed.tick(1_000).unwrap());
        assert!(viewed.battle(5_000).is_none());
        let north = WirePos { y: at.y - 1, ..at };
        assert_eq!(viewed.walk(ours, vec![north]).unwrap(), None);
        let mut seat = viewed.battle(10_000).unwrap();
        let mut now = 10_000;
        fight(
            &mut seat,
            |s| {
                now += 20;
                assert!(viewed.tick(now).unwrap());
                let refused = viewed.serve(s).unwrap();
                assert!(refused.is_empty(), "{refused:?}");
                Vec::new()
            },
            strike,
        );
        assert!(now < 50_000, "{now}");
        assert!(viewed.returned(50_000).unwrap());
        assert_eq!(viewed.kept.clock(50_000), 10_000);
        assert!(viewed.tick(61_000).unwrap());
        assert!(viewed.tick(100_990).unwrap());
        assert!(!viewed.tick(101_000).unwrap());
        assert_eq!(viewed.hot.turn(), Some(PlayerId::new(1)));
        // Without a battle held, there is none to serve or to hand over.
        assert!(viewed.battle(101_000).is_none());
        assert!(viewed.serve(&mut seat).is_err());
    }

    #[test]
    fn the_command_refuses_what_it_does_not_know() {
        for (list, why) in [
            (&[][..], "needs a map file"),
            (&["m", "--humans", "8"][..], "bad colour `8`"),
            (&["m", "--humans", "red"][..], "bad colour `red`"),
            (&["m", "--seed"][..], "needs a value"),
            (&["m", "--seed", "x"][..], "bad number `x`"),
            (&["m", "--bugs", "some"][..], "fixed or original"),
            (&["m", "--duration", "11"][..], "bad duration `11`"),
            (&["m", "--turns", "2", "--check", "1"][..], "runs no window"),
            (&["m", "--fast"][..], "unknown option `--fast`"),
            (&["m", "n"][..], "unexpected `n`"),
        ] {
            let e = Options::parse(&args(list)).unwrap_err();
            assert!(e.contains(why), "{list:?}: {e}");
        }
    }
}
