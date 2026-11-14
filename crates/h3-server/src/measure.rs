//! The long journals' measurement (E16 M5 T23; SRV-Q30's major finding's rest): a whole game on a
//! map offer played by the uniformly random legal agent for every seat over many days
//! (`h3_sim::adventure::replay::Replay::record_random`), or by a busy one that makes each of a
//! seat's turns up to a number of map commands with free ones (a seat's bound, `[limits]
//! commands_per_turn`, at its most), its journal written as a game of
//! connections' is (`journal::whole`: every seat taken by its token, the start, each seat's
//! numbered commands, the system's, a checkpoint every `checkpoint_every` commands; untimed, so
//! no `Elapsed` lines), then, at each chosen day, the restart's work on the journal up to that
//! day: its reading (`journal::whole::read`), its re-run as a restart hosts it again
//! (`GameSession::reopen`, the game's build included), and the verification of the adventure
//! replay it holds (`Replay::verify`, a finished game's check once games end, E10); each timed,
//! with the heap each holds and at most, through the caller's [`Meter`] (the binary's counting
//! allocator), and the session's in-memory copy of the game's commands (`GameJournal::commands`).
//! `h3-server-journals` prints them; the design note (`E16-design.md` §15) holds the numbers and
//! the case for state saves.
//!
//! Provisional (T23): the agent, the journal's form, the marks, the tokens (fixed, of a token's
//! length).

// An adapter's measurement: the wall time it prints.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use std::net::{IpAddr, Ipv4Addr};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use std::{fs, io};

use h3_contracts::PlayerId;
use h3_game::{Actor, GameCommand, GameState, Manage, NoopSink, apply};
use h3_kernel::{Rng, StreamId};
use h3_protocol::map::game::manifest;
use h3_protocol::session::game::{Clock, GameEntry, GameOptions, GameSession, Spectators};
use h3_sim::adventure::action::battle_of;
use h3_sim::adventure::replay::{RecordOptions, Replay, ReplayHeader};
use h3_sim::adventure::rollout::choose;
use h3_sim::adventure::{Action, AdventureEnv};

use crate::journal::whole::{self as journal, GAME_JOURNAL_FORMAT_VERSION, GameStart};
use crate::whole::MapOffer;
use crate::{Error, config};

/// The heap as the caller counts it: the binary's counting allocator.
pub trait Meter {
    /// The bytes allocated now.
    fn live(&self) -> usize;
    /// The most bytes allocated at once since the last [`Meter::reset_peak`].
    fn peak(&self) -> usize;
    /// Starts a new peak at the bytes allocated now.
    fn reset_peak(&self);
}

/// What the measurement plays and measures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// The map file.
    pub map: PathBuf,
    /// The packs' directory.
    pub packs: PathBuf,
    /// The game's seed.
    pub seed: u64,
    /// The random agent's seed.
    pub agent_seed: u64,
    /// The game ends when this day's successor starts.
    pub days: u32,
    /// The map commands a seat makes in each of its turns, free ones making up for the agent's
    /// (0: the plain random agent).
    pub per_turn: u32,
    /// The days after which the journal is measured (those past `days` left out).
    pub marks: Vec<u32>,
    /// A state hash in the journal every this many commands (the server's `checkpoint_every`).
    pub checkpoint_every: u32,
    /// A directory for the journals it writes (`<data>/whole/`), emptied first.
    pub data: PathBuf,
}

/// The game played.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Played {
    /// Its players (each a seat).
    pub players: usize,
    /// The day it ended on (the one after its last).
    pub day: u32,
    /// Its accepted commands.
    pub commands: usize,
    /// Of them the system actor's (a neutral battle side's).
    pub system: usize,
    /// Of them battle commands, a seat's or the system's.
    pub battle: usize,
    /// The agent's time to play it (its choices, the env's steps), in seconds.
    pub play_secs: f64,
    /// The game's build alone (`GameState::new`), in seconds.
    pub build_secs: f64,
    /// The heap the built game's state holds before its first command, in bytes.
    pub built_bytes: usize,
}

/// The journal up to the end of one day.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mark {
    /// The day whose end it is (the next day's start its last command).
    pub day: u32,
    /// The commands up to it.
    pub commands: usize,
    /// The journal's entries after its first line.
    pub entries: usize,
    /// The journal file's length.
    pub bytes: u64,
    /// The engine's re-run of the commands alone (`h3_game::apply`, no session, no hash), in
    /// seconds: the floor of every re-run.
    pub apply_secs: f64,
    /// The heap the game's state holds at the day's end, in bytes.
    pub state_bytes: usize,
    /// The journal's reading, in seconds.
    pub read_secs: f64,
    /// The most heap the reading took at once, in bytes.
    pub read_peak: usize,
    /// The heap the read journal holds (its entries), in bytes.
    pub read_bytes: usize,
    /// The re-run as a restart hosts the game again (`GameSession::reopen`: the build, every
    /// command through the session, the checkpoints, each player's snapshot into its record), in
    /// seconds.
    pub reopen_secs: f64,
    /// The most heap the re-run took at once beyond the read journal, in bytes.
    pub reopen_peak: usize,
    /// The heap the re-run session holds, in bytes.
    pub session_bytes: usize,
    /// Of it the session's copy of the game's commands (`GameJournal::commands`), in bytes.
    pub copy_bytes: usize,
    /// The verification of the adventure replay up to the day (`Replay::verify`: the build, every
    /// command, the checkpoints compared), in seconds.
    pub verify_secs: f64,
}

/// Where the commands of the day `day` end: the first command applied after it began.
#[derive(Debug, Clone, Copy)]
struct Cut {
    day: u32,
    commands: usize,
    /// The day after them and the state's hash: the prefix replay's end.
    then: u32,
    hash: u64,
    apply_secs: f64,
    state_bytes: usize,
}

/// A seat's token: a fixed one of a token's length (32 hex digits).
fn token(player: PlayerId) -> String {
    format!("{:032x}", u128::from(player.raw()) + 1)
}

/// Plays the game, writes its journals and measures them at each mark.
///
/// # Errors
///
/// The offer's, the agent's, a journal's or a re-run's, as text.
pub fn run(options: &Options, meter: &dyn Meter) -> Result<(Played, Vec<Mark>), Error> {
    let offer = load(options)?;
    let err = |what: &str, e: &dyn std::fmt::Display| Error(format!("{what}: {e}"));
    let clock = Instant::now();
    let replay = if options.per_turn == 0 {
        Replay::record_random(
            &offer.rules,
            Arc::new(offer.setup.clone()),
            offer.map.clone(),
            RecordOptions {
                seed: options.seed,
                agent_seed: options.agent_seed,
                max_days: options.days,
                max_steps: 0,
                checkpoint_every: options.checkpoint_every,
            },
        )
        .map_err(|e| err("the agent's game", &e))?
    } else {
        busy(&offer, options)?
    };
    let play_secs = clock.elapsed().as_secs_f64();
    let players = offer.setup.players.len();
    let (built_bytes, build_secs, hash) = {
        let before = meter.live();
        let clock = Instant::now();
        let state = GameState::new(&offer.rules, &offer.setup, replay.seed)
            .map_err(|e| err("the build", &format_args!("{e:?}")))?;
        let secs = clock.elapsed().as_secs_f64();
        let bytes = meter.live().saturating_sub(before);
        (bytes, secs, state.state_hash())
    };
    let played = Played {
        players,
        day: replay.final_day,
        commands: replay.commands.len(),
        system: (replay.commands.iter())
            .filter(|(actor, _)| *actor == Actor::System)
            .count(),
        battle: (replay.commands.iter())
            .filter(|(_, command)| matches!(command, GameCommand::Battle(_)))
            .count(),
        play_secs,
        build_secs,
        built_bytes,
    };
    let cuts = cuts(&offer, &replay, &options.marks, meter)?;
    let dir = journal::dir(&options.data);
    match fs::remove_dir_all(&dir) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(err("the journals", &e)),
        _ => {}
    }
    let mut marks = Vec::new();
    for cut in cuts {
        marks.push(measure((&offer, &replay, hash), cut, &options.data, meter)?);
    }
    Ok((played, marks))
}

/// The offer of the map, as a server's `[[map]]` loads it: the three packs, the faithful profile,
/// normal difficulty, untimed.
fn load(options: &Options) -> Result<MapOffer, Error> {
    let root = (options.map.parent()).unwrap_or_else(|| Path::new("."));
    let name = (options.map.file_name().and_then(|name| name.to_str()))
        .ok_or_else(|| Error(format!("{}: not a file's name", options.map.display())))?;
    let (_, map) = h3_files::adventure::read_map(root, name).map_err(|e| Error(e.to_string()))?;
    let text = format!(
        "id = \"long\"\nfile = {:?}\nhash = \"{:#018x}\"\npacks = [\"base\", \"ab\", \"sod\"]\n\
         profile = \"base:faithful\"\n",
        options.map.display().to_string(),
        map.content_hash
    );
    let config: config::MapOffer = toml::from_str(&text).map_err(|e| Error(e.to_string()))?;
    MapOffer::load(&config, &options.packs)
}

/// The game played by the busy agent, as [`Replay::record_random`] records the plain one: the
/// random agent's choice for every seat (`h3_sim::adventure::rollout::choose`), but before a
/// seat's end of turn on the map its turn's map commands (neither a battle's nor the end itself)
/// made up to `per_turn` with free management commands, a hero of its own's tactics flag switched
/// (GAM-MAN-001: any number in a turn, each counted by a seat's bound; none for a seat without a
/// hero): the most a seat may add to the journal in a turn under `[limits] commands_per_turn`.
fn busy(offer: &MapOffer, options: &Options) -> Result<Replay, Error> {
    let err = |e: &dyn std::fmt::Display| Error(format!("the busy agent's game: {e}"));
    let setup = Arc::new(offer.setup.clone());
    let mut env = AdventureEnv::new(offer.rules.clone(), setup, options.seed, options.days)
        .map_err(|e| err(&format_args!("{e:?}")))?;
    let mut rng = Rng::for_stream(options.agent_seed, StreamId::AGENT);
    let every = usize::try_from(options.checkpoint_every).unwrap_or(usize::MAX);
    let (mut commands, mut checkpoints) = (Vec::new(), Vec::new());
    let mut made = 0;
    let mut push = |env: &AdventureEnv, actor: Actor, command: GameCommand| {
        commands.push((actor, command));
        if commands.len().checked_rem(every) == Some(0) {
            checkpoints.push(env.state_hash());
        }
        commands.len()
    };
    loop {
        let Some(seat) = env.to_move().next() else {
            break;
        };
        let action = choose(&env, seat, &mut rng);
        let on_map = battle_of(env.state(), seat).is_none();
        if let (Some(player), true, true) = (seat.player(), on_map, action == Action::end_turn()) {
            let own = (env.state().adventure().heroes().iter()).find(|hero| hero.owner == player);
            if let Some((hero, mut on)) = own.map(|hero| (hero.id, hero.tactics)) {
                for _ in made..options.per_turn {
                    on = !on;
                    let command = GameCommand::Manage(Manage::Tactics { hero, on });
                    (env.step_command(seat, command.clone(), &mut NoopSink))
                        .map_err(|e| err(&format_args!("a flag: {e:?}")))?;
                    push(&env, seat.actor(), command);
                }
            }
        }
        let (_, command) = (env.step_decoded(seat, action, &mut NoopSink))
            .map_err(|e| err(&format_args!("a command: {e:?}")))?;
        match command {
            GameCommand::EndTurn => made = 0,
            GameCommand::Battle(_) => {}
            _ => made += 1,
        }
        push(&env, seat.actor(), command);
    }
    Ok(Replay {
        header: ReplayHeader::current(&offer.rules),
        map: offer.map.clone(),
        seed: options.seed,
        difficulty: env.difficulty(),
        commands,
        checkpoint_every: options.checkpoint_every,
        checkpoints,
        final_day: env.day(),
        final_hash: env.state_hash(),
    })
}

/// The engine's re-run of the commands alone: where each mark's day ends, the state's hash and
/// heap there, the time to it.
fn cuts(
    offer: &MapOffer,
    replay: &Replay,
    marks: &[u32],
    meter: &dyn Meter,
) -> Result<Vec<Cut>, Error> {
    let before = meter.live();
    let clock = Instant::now();
    let mut state = GameState::new(&offer.rules, &offer.setup, replay.seed)
        .map_err(|e| Error(format!("the build: {e:?}")))?;
    let mut marks = marks.iter().copied().peekable();
    let mut cuts = Vec::new();
    let mut cut = |state: &GameState, commands: usize, cuts: &mut Vec<Cut>| {
        while let Some(day) = marks.next_if(|day| state.day() > *day) {
            cuts.push(Cut {
                day,
                commands,
                then: state.day(),
                hash: state.state_hash(),
                apply_secs: clock.elapsed().as_secs_f64(),
                state_bytes: meter.live().saturating_sub(before),
            });
        }
    };
    for (at, (actor, command)) in replay.commands.iter().enumerate() {
        cut(&state, at, &mut cuts);
        apply(
            &mut state,
            &offer.rules,
            *actor,
            command.clone(),
            &mut NoopSink,
        )
        .map_err(|e| Error(format!("command {at}: {e}")))?;
    }
    cut(&state, replay.commands.len(), &mut cuts);
    Ok(cuts)
}

/// The journal of a game of connections up to `commands`: every seat taken by its token in
/// player order (the first the host's), the start, each command as its seat's numbered one or the
/// system's, a checkpoint every `checkpoint_every` commands, as `GameSession` journals them.
fn entries(replay: &Replay, players: usize, commands: usize) -> Vec<GameEntry> {
    let ids = (0..players).map(|i| PlayerId::new(u8::try_from(i).unwrap_or(u8::MAX)));
    let mut entries: Vec<GameEntry> = ids.map(GameEntry::Taken).collect();
    entries.push(GameEntry::Started);
    let mut seqs = vec![0u32; players];
    let every = usize::try_from(replay.checkpoint_every).unwrap_or(usize::MAX);
    let mut checkpoints = replay.checkpoints.iter().copied();
    for (at, (actor, command)) in replay.commands[..commands].iter().enumerate() {
        entries.push(match actor {
            Actor::Player(player) => {
                let seq = &mut seqs[player.index()];
                *seq += 1;
                GameEntry::Numbered(*player, *seq, command.clone())
            }
            Actor::System => GameEntry::Command(Actor::System, command.clone()),
        });
        if (at + 1).checked_rem(every) == Some(0)
            && let Some(hash) = checkpoints.next()
        {
            entries.push(GameEntry::Checkpoint(hash));
        }
    }
    entries
}

/// The replay's first `commands` commands and their checkpoints, ending on `day` with `hash`.
fn prefix(replay: &Replay, (commands, day, hash): (usize, u32, u64)) -> Replay {
    let every = usize::try_from(replay.checkpoint_every).unwrap_or(usize::MAX);
    let checkpoints = commands.checked_div(every).unwrap_or(0);
    Replay {
        header: replay.header.clone(),
        map: replay.map.clone(),
        seed: replay.seed,
        difficulty: replay.difficulty,
        commands: replay.commands[..commands].to_vec(),
        checkpoint_every: replay.checkpoint_every,
        checkpoints: replay.checkpoints[..checkpoints].to_vec(),
        final_day: day,
        final_hash: hash,
    }
}

/// Writes the journal up to `cut`, then reads it, re-runs it and verifies its replay.
/// `hash` is the game's state's before its first command.
fn measure(
    (offer, replay, hash): (&MapOffer, &Replay, u64),
    cut: Cut,
    data: &Path,
    meter: &dyn Meter,
) -> Result<Mark, Error> {
    let err =
        |what: &str, e: &dyn std::fmt::Display| Error(format!("day {}: {what}: {e}", cut.day));
    let players = offer.setup.players.len();
    let game = cut.day;
    let start = GameStart {
        format: GAME_JOURNAL_FORMAT_VERSION,
        game,
        offer: offer.id.clone(),
        origin: IpAddr::V4(Ipv4Addr::LOCALHOST),
        tokens: (0..players)
            .map(|i| token(PlayerId::new(u8::try_from(i).unwrap_or(u8::MAX))))
            .collect(),
        replay: h3_files::adventure::replay_to_toml(&prefix(replay, (0, 1, hash)))
            .map_err(|e| err("the start's replay", &e))?,
        clock: Clock::UNLIMITED.into(),
    };
    let written = entries(replay, players, cut.commands);
    let count = written.len();
    let mut writer = journal::create(data, &start).map_err(|e| err("the journal", &e))?;
    writer
        .append_game(written.into_iter())
        .map_err(|e| err("the journal", &e))?;
    drop(writer);
    let path = journal::dir(data).join(format!("{game}.journal"));
    let bytes = fs::metadata(&path)
        .map_err(|e| err("the journal", &e))?
        .len();
    // The reading.
    meter.reset_peak();
    let before = meter.live();
    let clock = Instant::now();
    let read = journal::read(&path).map_err(|e| err("the reading", &e))?;
    let read_secs = clock.elapsed().as_secs_f64();
    let read_peak = meter.peak().saturating_sub(before);
    let read_bytes = meter.live().saturating_sub(before);
    // The re-run, as `whole::Prepared::resume` hosts it.
    let options = GameOptions {
        game,
        checkpoint_every: replay.checkpoint_every,
        record_bytes: config::Limits::default().record_bytes,
        clock: Clock::UNLIMITED,
        commands: config::Limits::default().commands(),
        spectators: Spectators::NONE,
    };
    let tokens = read.start.tokens.clone();
    meter.reset_peak();
    let before = meter.live();
    let clock = Instant::now();
    let session = GameSession::reopen(
        &offer.rules,
        &offer.setup,
        replay.seed,
        manifest(&offer.rules, offer.info.map),
        options,
        |player| tokens[player.index()].clone(),
        &read.entries,
    )
    .map_err(|e| err("the re-run", &format_args!("{e:?}")))?;
    let reopen_secs = clock.elapsed().as_secs_f64();
    let reopen_peak = meter.peak().saturating_sub(before);
    let session_bytes = meter.live().saturating_sub(before);
    let copy_bytes = {
        let before = meter.live();
        let copy = session.journal().commands.clone();
        let bytes = meter.live().saturating_sub(before);
        drop(copy);
        bytes
    };
    drop(session);
    drop(read);
    // The replay's verification up to the day.
    let prefix = prefix(replay, (cut.commands, cut.then, cut.hash));
    let clock = Instant::now();
    prefix
        .verify(&offer.rules, &offer.map, &offer.setup)
        .map_err(|e| err("the verification", &e))?;
    let verify_secs = clock.elapsed().as_secs_f64();
    Ok(Mark {
        day: cut.day,
        commands: cut.commands,
        entries: count,
        bytes,
        apply_secs: cut.apply_secs,
        state_bytes: cut.state_bytes,
        read_secs,
        read_peak,
        read_bytes,
        reopen_secs,
        reopen_peak,
        session_bytes,
        copy_bytes,
        verify_secs,
    })
}
