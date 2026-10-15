//! `h3-view sequences` (E17 M2 T4a): the recorded sequences (`clients/situations/sequences.toml`)
//! played by the engine in-process and timed by the viewer (`h3_view::anim`), each timeline
//! compared with the one the capture harness recorded in the original (`docs/epics/E17-design.md`
//! §7.6). A sequence's `[[sequence.command]]` entries are the engine's commands for what the
//! harness's actions did there (a seat and a wire command); one without them is skipped. With
//! `--frames` (E17 M2 T4b-1, design §7.9) it also draws the viewer's frame at each time the
//! harness kept a frame of the original (`<dir>/<id>/frames.toml`, `h3drive.py frames`), for the
//! harness's diff. An adapter: it reads files and the archives.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use h3_formats::lod::LodArchive;
use h3_protocol::wire::{Command, Seat, ServerMessage, Side};
use h3_render::Bugs;
use h3_view::anim::{self, Pose, Speed, Timeline};
use h3_view::batch::{self, Batch};
use h3_view::compose::compose;
use h3_view::hero::HeroPose;
use h3_view::layout::BattleLayout;
use h3_view::record::Record;
use h3_view::tables::Tables;
use h3_view::{layout, record, scene};
use serde::Deserialize;

use crate::render::{lookup, read_archives, write};
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

impl Sequence {
    /// When the harness's recording ends: `ms` after its last actions (`pre`, or the last
    /// command's `at` in a battle in progress, M3 T1a).
    fn until(&self) -> u32 {
        let last = self
            .command
            .iter()
            .filter_map(|c| c.at)
            .fold(self.pre, u32::max);
        last + self.ms
    }
}

/// A command of a seat; `at`, when a player gave it (M3 T1a): its time after the opening (the
/// first command's defaults to the sequence's `pre`); a command without one is the computer's,
/// which acts as soon as its turn comes.
#[derive(Debug, Deserialize)]
struct Step {
    seat: Seat,
    command: Command,
    #[serde(default)]
    at: Option<u32>,
}

/// The frames the harness kept of a sequence in the original (`h3drive.py frames`).
#[derive(Debug, Deserialize)]
struct Frames {
    frame: Vec<Kept>,
}

/// One kept frame: its time, the stacks the original shows fidgeting then (the viewer draws
/// none of its own in a check), each `[side, record, group, frame]`, and its heroes (E17 M3 T3a:
/// their defeats and victories come from its generator), each `[side, group, frame]`: the frame
/// check's, not the capture set's, whose heroes are the viewer's own. The flags are always the
/// viewer's.
#[derive(Debug, Deserialize)]
struct Kept {
    t: u32,
    file: String,
    #[serde(default)]
    fidgets: Vec<[u32; 4]>,
    #[serde(default)]
    heroes: Vec<[u32; 3]>,
}

/// The command's options.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    only: Option<String>,
    speed: Speed,
    packs: PathBuf,
    install: PathBuf,
    out: PathBuf,
    frames: Option<PathBuf>,
}

impl Options {
    /// Parses `[<id>] [--speed <0-2>] [--packs <dir>] [--install <dir>] [--out <dir>]
    /// [--frames <dir>]`.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let env = std::env::var_os("H3_INSTALL").map(PathBuf::from);
        let (mut only, mut speed, mut packs, mut install, mut out) = (None, 0, None, None, None);
        let mut frames = None;
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
                "--frames" => frames = Some(PathBuf::from(value()?)),
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
            frames,
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
        let magic = motion::magic(&record, &tables, &archives)?;
        let stage = motion::stage(&record, &tables, &layout, &magic);
        // The harness acts `pre` ms after the opening, and then at each command's `at`.
        let commands: Vec<anim::Command<'_>> = record
            .messages
            .iter()
            .filter_map(|m| match m {
                ServerMessage::Events { events, .. } => Some(events),
                _ => None,
            })
            .enumerate()
            .map(|(i, events)| anim::Command {
                at: sequence
                    .command
                    .get(i)
                    .and_then(|s| s.at)
                    .or((i == 0).then_some(sequence.pre)),
                events,
            })
            .collect();
        let snapshot = record
            .first_snapshot()
            .ok_or("the record holds no snapshot")?;
        let timeline = anim::timeline(snapshot, &commands, &looks, stage, options.speed)
            .map_err(|e| format!("{}: {e}", sequence.id))?
            .with_heroes(motion::heroes(&record, &tables, &archives, None)?);
        // What the harness saw: its recording ends `ms` after the actions.
        let until = sequence.until();
        let ours: String = timeline
            .lines()
            .lines()
            .filter(|l| l.split(' ').next().and_then(|t| t.parse::<u32>().ok()) <= Some(until))
            .map(|l| format!("{l}\n"))
            .collect();
        let ours_path = options.out.join(format!("{}.txt", sequence.id));
        write(&ours_path, ours.as_bytes())?;
        if let Some(dir) = &options.frames {
            let here = dir.join(&sequence.id);
            let drawn = draw_frames(&here, &record, &timeline, &tables, &layout, &archives)?;
            println!(
                "{}: {drawn} frames drawn in {}",
                sequence.id,
                here.display()
            );
        }
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

/// Draws the viewer's frame at each time in `<here>/frames.toml` (the stacks the original shows
/// fidgeting then in its poses) into `<here>/ours/<file>.png`, and what it drew into
/// `<here>/ours.txt`; returns how many.
pub(crate) fn draw_frames(
    here: &Path,
    record: &Record,
    timeline: &Timeline,
    tables: &Tables,
    layout: &BattleLayout,
    archives: &[(&str, LodArchive<'_>)],
) -> Result<usize, String> {
    let path = here.join("frames.toml");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let kept: Frames = toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let snapshot = record
        .first_snapshot()
        .ok_or("the record holds no snapshot")?;
    let manifest = &record.header.manifest;
    let mut lists = Vec::new();
    let mut log = String::new();
    for frame in &kept.frame {
        let mut moment = scene::Moment::at(timeline, frame.t);
        for &[side, index, group, f] in &frame.fidgets {
            let side = if side == 0 {
                Side::Attacker
            } else {
                Side::Defender
            };
            let member = timeline
                .members
                .iter()
                .find(|m| m.side == side && u32::from(m.record) == index);
            if let Some(m) = member {
                let now = moment.poses.get(&m.stack).copied().unwrap_or(m.start);
                let pose = Pose {
                    group,
                    frame: f,
                    offset: [0, 0],
                    ..now
                };
                moment.poses.insert(m.stack, pose);
            }
        }
        for &[side, group, f] in &frame.heroes {
            if let Some(hero) = moment
                .heroes
                .get_mut(side as usize)
                .and_then(Option::as_mut)
            {
                *hero = HeroPose {
                    group,
                    frame: f,
                    ..*hero
                };
            }
        }
        log.push_str(&describe(frame.t, &moment));
        let list = scene::battle_at(snapshot, &moment, manifest, tables, layout)
            .map_err(|e| e.to_string())?;
        lists.push((frame, list));
    }
    let mut names = std::collections::BTreeSet::new();
    for (_, list) in &lists {
        names.extend(batch::names(list, layout));
    }
    let mut batch = Batch::default();
    for name in &names {
        batch
            .add(name, &lookup(archives, name)?)
            .map_err(|e| e.to_string())?;
    }
    for (frame, list) in &lists {
        // Compared with the original's captures: its drawing bugs reproduced (E17 M3 T4a).
        let screen = compose(list, &batch, layout, Bugs::Original).map_err(|e| e.to_string())?;
        let png = here
            .join("ours")
            .join(Path::new(&frame.file).with_extension("png"));
        write(&png, &h3_render::png::encode(&screen))?;
    }
    write(&here.join("ours.txt"), log.as_bytes())?;
    Ok(lists.len())
}

/// A moment as one line: `<t> pulse <n> outline <id|-> [projectile <def> f<frame> <x>,<y>]`, then
/// each stack's `<id>:g<group>f<frame>h<hex>[m][+<x>,<y>]`, `m` when it faces left (the harness
/// places a count box by the facing, M3 T1a).
pub(crate) fn describe(t: u32, moment: &scene::Moment) -> String {
    use std::fmt::Write as _;
    let mut line = format!("{t} pulse {} outline ", moment.pulse);
    match moment.active {
        Some(id) => line.push_str(&id.to_string()),
        None => line.push('-'),
    }
    if let Some(p) = &moment.projectile {
        let _ = write!(
            line,
            " projectile {} f{} {},{}",
            p.def, p.frame, p.at[0], p.at[1]
        );
    }
    // The heroes (E17 M3 T3a), in a shape the harness's pose pattern skips.
    for (side, hero) in moment.heroes.iter().enumerate() {
        if let Some(h) = hero {
            let _ = write!(line, " hero{side} g{}f{} flag{}", h.group, h.frame, h.flag);
        }
    }
    for (id, p) in &moment.poses {
        let _ = write!(line, " {id}:g{}f{}h{}", p.group, p.frame, p.hex);
        if p.mirrored {
            line.push('m');
        }
        if p.offset != [0, 0] {
            let _ = write!(line, "+{},{}", p.offset[0], p.offset[1]);
        }
    }
    line.push('\n');
    line
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
            "melee", "--speed", "1", "--out", "/o", "--packs", "/p",
        ]))
        .unwrap();
        assert_eq!(o.only.as_deref(), Some("melee"));
        assert_eq!(o.speed, Speed::new(1));
        assert_eq!((o.out, o.packs), (PathBuf::from("/o"), PathBuf::from("/p")));
        let o = Options::parse(&args(&["--install", "/i", "--frames", "/f"])).unwrap();
        assert_eq!((o.only, o.install), (None, PathBuf::from("/i")));
        assert_eq!(o.frames, Some(PathBuf::from("/f")));
        assert!(o.out.ends_with("out/sequences"));
        for bad in [
            &["--speed"][..],
            &["--speed", "4"],
            &["--what"],
            &["a", "b"],
        ] {
            assert!(Options::parse(&args(bad)).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn the_sequences_file_names_engine_commands() {
        let path = assets::default_dir().join("../situations/sequences.toml");
        let file: File = toml::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        let melee = file.sequence.iter().find(|s| s.id == "melee").unwrap();
        assert_eq!(melee.command.len(), 2);
        assert_eq!(melee.command[1].seat, Seat::Defender);
        assert_eq!(
            melee.command[1].command,
            Command::Attack {
                target: 0,
                from: 45
            }
        );
        assert_eq!((melee.pre, melee.ms), (0, 6000));
        assert_eq!((melee.until(), melee.command[0].at), (6000, None));
    }

    #[test]
    fn a_battle_in_progress_counts_from_its_last_actions() {
        let s: Sequence = toml::from_str(
            r#"
            id = "x"
            scenario = "x.toml"
            timeline = "x.txt"
            pre = 100
            ms = 500
            actions = ["hex:3", "turn", "key:w"]
            command = [
                { seat = "Attacker", command = { Wait = {} } },
                { seat = "Defender", at = 4500, command = { Defend = {} } },
                { seat = "Defender", at = 2000, command = { Defend = {} } },
            ]
            "#,
        )
        .unwrap();
        assert_eq!(s.until(), 5000);
        assert_eq!((s.command[0].at, s.command[2].at), (None, Some(2000)));
    }
}
