//! `h3-view sequences` (E17 M2 T4a): the recorded sequences (`clients/situations/sequences.toml`)
//! played by the engine in-process and timed by the viewer (`h3_view::anim`), each timeline
//! compared with the one the capture harness recorded in the original (`docs/epics/E17-design.md`
//! §7.6). A sequence's `[[sequence.command]]` entries are the engine's commands for what the
//! harness's actions did there (a seat and a wire command); one without them is skipped. An
//! adapter: it reads files and the archives.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use h3_formats::lod::LodArchive;
use h3_protocol::wire::{Command, Seat, ServerMessage};
use h3_view::anim::{self, Speed};
use h3_view::{layout, record};
use serde::Deserialize;

use crate::render::{read_archives, write};
use crate::{assets, motion, scenario};

/// The sequences file.
#[derive(Debug, Deserialize)]
struct File {
    sequence: Vec<Sequence>,
}

/// One recorded sequence (the harness's fields besides these are its own).
#[derive(Debug, Deserialize)]
struct Sequence {
    id: String,
    scenario: String,
    timeline: String,
    /// The milliseconds stepped before the actions, and after them.
    #[serde(default)]
    pre: u32,
    ms: u32,
    #[serde(default)]
    command: Vec<Step>,
}

/// A command of a seat.
#[derive(Debug, Deserialize)]
struct Step {
    seat: Seat,
    command: Command,
}

/// The command's options.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    only: Option<String>,
    speed: Speed,
    packs: PathBuf,
    install: PathBuf,
    out: PathBuf,
}

impl Options {
    /// Parses `[<id>] [--speed <0-2>] [--packs <dir>] [--install <dir>] [--out <dir>]`.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let env = std::env::var_os("H3_INSTALL").map(PathBuf::from);
        let (mut only, mut speed, mut packs, mut install, mut out) = (None, 0, None, None, None);
        let mut rest = args.iter();
        while let Some(arg) = rest.next() {
            let mut value = || {
                rest.next()
                    .cloned()
                    .ok_or_else(|| format!("sequences: `{arg}` needs a value"))
            };
            match arg.as_str() {
                "--speed" => {
                    let v = value()?;
                    speed = v
                        .parse()
                        .ok()
                        .filter(|s| *s <= 2)
                        .ok_or(format!("sequences: bad speed `{v}`"))?;
                }
                "--packs" => packs = Some(PathBuf::from(value()?)),
                "--install" => install = Some(PathBuf::from(value()?)),
                "--out" => out = Some(PathBuf::from(value()?)),
                flag if flag.starts_with("--") => {
                    return Err(format!("sequences: unknown option `{flag}`"));
                }
                id if only.is_none() => only = Some(id.to_owned()),
                extra => return Err(format!("sequences: unexpected `{extra}`")),
            }
        }
        Ok(Self {
            only,
            speed: Speed::new(speed),
            packs: packs.unwrap_or_else(|| manifest.join("../../../packs")),
            install: install
                .or(env)
                .unwrap_or_else(|| manifest.join("../../../original-data/install")),
            out: out.unwrap_or_else(|| manifest.join("../../out/sequences")),
        })
    }
}

/// Runs the command: exits 1 when a timeline differs from its recording.
pub fn run(options: &Options) -> ExitCode {
    match sequences(options) {
        Ok(0) => ExitCode::SUCCESS,
        Ok(_) => ExitCode::from(1),
        Err(message) => {
            eprintln!("sequences: {message}");
            ExitCode::FAILURE
        }
    }
}

/// Plays and compares the sequences; returns how many differ.
fn sequences(options: &Options) -> Result<usize, String> {
    let dir = assets::default_dir().join("../situations");
    let path = dir.join("sequences.toml");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let file: File = toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let layout_path = assets::default_dir().join("../layouts/battle.toml");
    let layout = layout::parse_battle(
        &fs::read_to_string(&layout_path).map_err(|e| format!("{}: {e}", layout_path.display()))?,
    )
    .map_err(|e| format!("{}: {e}", layout_path.display()))?;
    let bytes = read_archives(&options.install)?;
    let archives = bytes
        .iter()
        .map(|(name, b)| {
            LodArchive::parse(b)
                .map(|a| (*name, a))
                .map_err(|e| format!("{name}: {e:?}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut differing = 0;
    for sequence in &file.sequence {
        if options.only.as_ref().is_some_and(|id| *id != sequence.id) {
            continue;
        }
        if sequence.command.is_empty() {
            println!("{}: no engine commands, skipped", sequence.id);
            continue;
        }
        let steps: Vec<(Seat, Command)> = sequence
            .command
            .iter()
            .map(|s| (s.seat, s.command))
            .collect();
        let text = scenario::played(&dir.join(&sequence.scenario), &options.packs, 1, &steps)
            .map_err(|e| format!("{}: {e}", sequence.id))?;
        write(
            &options.out.join(format!("{}.jsonl", sequence.id)),
            text.as_bytes(),
        )?;
        let record = record::parse(&text).map_err(|e| format!("{}: {e}", sequence.id))?;
        let top = record
            .header
            .manifest
            .packs
            .last()
            .map_or("", |p| p.id.as_str());
        let tables = assets::load(&assets::default_dir(), top)?;
        let looks = motion::looks(&record, &tables, &archives)?;
        let stage = motion::stage(&record, &tables, &layout);
        // The harness acts `pre` ms after the opening.
        let commands: Vec<anim::Command<'_>> = record
            .messages
            .iter()
            .filter_map(|m| match m {
                ServerMessage::Events { events, .. } => Some(events),
                _ => None,
            })
            .enumerate()
            .map(|(i, events)| anim::Command {
                at: (i == 0).then_some(sequence.pre),
                events,
            })
            .collect();
        let snapshot = record
            .first_snapshot()
            .ok_or("the record holds no snapshot")?;
        let timeline = anim::timeline(snapshot, &commands, &looks, stage, options.speed)
            .map_err(|e| format!("{}: {e}", sequence.id))?;
        // What the harness saw: its recording ends `ms` after the actions.
        let until = sequence.pre + sequence.ms;
        let ours: String = timeline
            .lines()
            .lines()
            .filter(|l| l.split(' ').next().and_then(|t| t.parse::<u32>().ok()) <= Some(until))
            .map(|l| format!("{l}\n"))
            .collect();
        let ours_path = options.out.join(format!("{}.txt", sequence.id));
        write(&ours_path, ours.as_bytes())?;
        if options.speed.setting() != 0 {
            println!(
                "{}: {} changes at speed {} (recorded at 0 only)",
                sequence.id,
                ours.lines().count(),
                options.speed.setting()
            );
            continue;
        }
        let recorded_path = dir.join(&sequence.timeline);
        let recorded = fs::read_to_string(&recorded_path)
            .map_err(|e| format!("{}: {e}", recorded_path.display()))?;
        let diff = anim::differences(&ours, &recorded);
        if diff.is_empty() {
            println!(
                "{}: {} changes, as recorded",
                sequence.id,
                ours.lines().count()
            );
        } else {
            differing += 1;
            println!(
                "{}: differs from {} ({})",
                sequence.id,
                recorded_path.display(),
                ours_path.display()
            );
            for line in diff.iter().take(40) {
                println!("  {line}");
            }
        }
    }
    Ok(differing)
}
