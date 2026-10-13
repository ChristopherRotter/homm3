//! `h3-view play` (E17 M2 T4b-2a): a whole battle, frame by frame, headless (M2's headless
//! exit). A scenario is fought to its end in-process by the bot in both seats ([`crate::bot`]),
//! or a record file is read; its commands are played at the original's timing
//! (`h3_view::anim`) and every distinct frame is drawn. An adapter: it reads files and the
//! archives, and writes the frames.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use h3_formats::lod::LodArchive;
use h3_view::anim::{self, Speed};
use h3_view::batch::{self, Batch};
use h3_view::compose::compose;
use h3_view::record;
use h3_view::scene::{self, Moment};
use h3_view::{layout, tables::Tables};

use crate::gate::timed;
use crate::render::{lookup, read_archives, write};
use crate::{assets, motion};

/// The command's options.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    input: PathBuf,
    seed: u64,
    speed: Speed,
    packs: PathBuf,
    install: PathBuf,
    out: Option<PathBuf>,
    png: bool,
    from: u32,
    to: Option<u32>,
}

impl Options {
    /// Parses `<scenario.toml | record.jsonl> [--seed <n>] [--speed <0-2>] [--packs <dir>]
    /// [--install <dir>] [--out <dir>] [--png] [--from <ms>] [--to <ms>]`.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let env = std::env::var_os("H3_INSTALL").map(PathBuf::from);
        let (mut input, mut seed, mut speed, mut packs, mut install) = (None, 1, 0, None, None);
        let (mut out, mut png, mut from, mut to) = (None, false, 0, None);
        let mut rest = args.iter();
        while let Some(arg) = rest.next() {
            let mut value = || {
                rest.next()
                    .cloned()
                    .ok_or_else(|| format!("play: `{arg}` needs a value"))
            };
            let number = |v: String| {
                v.parse::<u64>()
                    .map_err(|_| format!("play: bad number `{v}` for `{arg}`"))
            };
            match arg.as_str() {
                "--seed" => seed = number(value()?)?,
                "--speed" => {
                    speed = number(value()?)?.min(2).try_into().unwrap_or(2);
                }
                "--packs" => packs = Some(PathBuf::from(value()?)),
                "--install" => install = Some(PathBuf::from(value()?)),
                "--out" => out = Some(PathBuf::from(value()?)),
                "--png" => png = true,
                "--from" => from = u32::try_from(number(value()?)?).unwrap_or(u32::MAX),
                "--to" => to = Some(u32::try_from(number(value()?)?).unwrap_or(u32::MAX)),
                flag if flag.starts_with("--") => {
                    return Err(format!("play: unknown option `{flag}`"));
                }
                path if input.is_none() => input = Some(PathBuf::from(path)),
                extra => return Err(format!("play: unexpected `{extra}`")),
            }
        }
        Ok(Self {
            input: input.ok_or("play: needs a scenario or a record file")?,
            seed,
            speed: Speed::new(speed),
            packs: packs.unwrap_or_else(|| manifest.join("../../../packs")),
            install: install
                .or(env)
                .unwrap_or_else(|| manifest.join("../../../original-data/install")),
            out,
            png,
            from,
            to,
        })
    }

    /// Whether the input is a scenario (anything but a `.jsonl` record).
    fn scenario(&self) -> bool {
        self.input.extension().is_none_or(|e| e != "jsonl")
    }

    /// The output directory: `clients/out/play/<stem>[-s<seed>]` by default.
    fn dir(&self) -> PathBuf {
        self.out.clone().unwrap_or_else(|| {
            let stem = self
                .input
                .file_stem()
                .map_or_else(|| "battle".into(), |s| s.to_string_lossy().into_owned());
            let name = if self.scenario() {
                format!("{stem}-s{}", self.seed)
            } else {
                stem
            };
            assets::default_dir().join("../out/play").join(name)
        })
    }
}

/// Runs the command.
pub fn run(options: &Options) -> ExitCode {
    match play(options) {
        Ok(lines) => {
            for line in lines {
                println!("{line}");
            }
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("play: {message}");
            ExitCode::FAILURE
        }
    }
}

/// Fights or reads the battle, draws its frames; returns what to print.
fn play(options: &Options) -> Result<Vec<String>, String> {
    let dir = options.dir();
    let mut lines = Vec::new();
    let text = if options.scenario() {
        let (text, took) =
            timed(|| crate::scenario::fought(&options.input, &options.packs, options.seed));
        let text = text?;
        let path = dir.join("record.jsonl");
        write(&path, text.as_bytes())?;
        lines.push(format!(
            "record: {} (fought in {} ms)",
            path.display(),
            took.as_millis()
        ));
        text
    } else {
        fs::read_to_string(&options.input)
            .map_err(|e| format!("{}: {e}", options.input.display()))?
    };
    let record = record::parse(&text).map_err(|e| e.to_string())?;
    let snapshot = record
        .first_snapshot()
        .ok_or("the record holds no snapshot")?;
    let manifest = &record.header.manifest;
    let top = manifest.packs.last().map_or("", |p| p.id.as_str());
    let tables: Tables = assets::load(&assets::default_dir(), top)?;
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
    let looks = motion::looks(&record, &tables, &archives)?;
    let stage = motion::stage(&record, &tables, &layout);
    let commands = anim::commands(&record.messages);
    let timeline = anim::timeline(snapshot, &commands, &looks, stage, options.speed)
        .map_err(|e| e.to_string())?;
    let until = options.to.unwrap_or(timeline.end).min(timeline.end);
    // Every moment the battle may change, drawn; a moment that draws what the one before drew is
    // not a new frame.
    let mut frames: Vec<(u32, Vec<scene::Draw>)> = Vec::new();
    let mut log = String::new();
    for t in timeline.changes(until) {
        let moment = Moment::at(&timeline, t);
        let list = scene::battle_at(snapshot, &moment, manifest, &tables, &layout)
            .map_err(|e| format!("{t} ms: {e}"))?;
        if frames.last().is_some_and(|(_, last)| *last == list) {
            continue;
        }
        let _ = writeln!(log, "{}", describe(t, &moment));
        frames.push((t, list));
    }
    write(&dir.join("frames.txt"), log.as_bytes())?;
    let mut names = BTreeSet::new();
    for (_, list) in &frames {
        names.extend(batch::names(list, &layout));
    }
    let mut batch = Batch::default();
    for name in &names {
        batch
            .add(name, &lookup(&archives, name)?)
            .map_err(|e| e.to_string())?;
    }
    let (drawn, took) = timed(|| {
        let mut drawn = 0;
        for (t, list) in frames.iter().filter(|(t, _)| *t >= options.from) {
            let screen = compose(list, &batch, &layout).map_err(|e| format!("{t} ms: {e}"))?;
            if options.png {
                write(
                    &dir.join(format!("f{t:06}.png")),
                    &h3_render::png::encode(&screen),
                )?;
            }
            drawn += 1;
        }
        Ok::<_, String>(drawn)
    });
    let drawn = drawn?;
    let commands_played = commands.len();
    lines.push(format!(
        "{commands_played} commands, {} ms at speed {}; {} frames ({drawn} drawn{} in {} ms), \
         their moments in {}",
        timeline.end,
        options.speed.setting(),
        frames.len(),
        if options.png { " as PNGs" } else { "" },
        took.as_millis(),
        dir.join("frames.txt").display()
    ));
    Ok(lines)
}

/// A moment as one line, as `sequences --frames` writes them.
fn describe(t: u32, moment: &Moment) -> String {
    crate::sequences::describe(t, moment).trim_end().to_owned()
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
            "--png",
            "--from",
            "100",
            "--to",
            "900",
            "--out",
            "x",
        ]))
        .unwrap();
        assert_eq!(o.seed, 7);
        assert_eq!(o.speed, Speed::new(2));
        assert!(o.png && o.scenario());
        assert_eq!((o.from, o.to), (100, Some(900)));
        assert_eq!(o.dir(), PathBuf::from("x"));
        let r = Options::parse(&args(&["a/b.jsonl"])).unwrap();
        assert!(!r.scenario());
        assert!(r.dir().ends_with("out/play/b"));
        assert!(Options::parse(&args(&[])).is_err());
        assert!(Options::parse(&args(&["a", "--seed", "x"])).is_err());
        assert!(Options::parse(&args(&["a", "--bogus"])).is_err());
        assert!(Options::parse(&args(&["a", "b"])).is_err());
        assert!(Options::parse(&args(&["a", "--to"])).is_err());
    }
}
