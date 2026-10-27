//! `h3-view play` (E17 M2 T4b-2a): a whole battle, frame by frame, headless (M2's headless
//! exit). A scenario is fought to its end in-process by the bot in both seats ([`crate::bot`]),
//! as it is played since E17 M5 T1b (each command when the frames reach the timeline's end,
//! `crate::live`), or a record file is read; its commands are played at the original's timing
//! (`h3_view::anim`) and every distinct frame is drawn. An adapter: it reads files and the
//! archives, and writes the frames.

use std::fmt::Write as _;
use std::path::PathBuf;
use std::process::ExitCode;

use h3_view::anim::{Playhead, Speed, Timeline};
use h3_view::scene::Moment;

use crate::assets;
use crate::battle::{Battle, Ground, Source};
use crate::gate::timed;
use crate::render::write;

/// The command's options.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    source: Source,
    out: Option<PathBuf>,
    png: bool,
    from: u32,
    to: Option<u32>,
    moments: bool,
}

impl Options {
    /// Parses `<scenario.toml | record.jsonl> [--seed <n>] [--speed <0-2>] [--packs <dir>]
    /// [--install <dir>] [--out <dir>] [--png] [--from <ms>] [--to <ms>]`.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let clients = assets::clients_dir();
        let env = std::env::var_os("H3_INSTALL").map(PathBuf::from);
        let (mut input, mut seed, mut speed, mut packs, mut install) = (None, 1, 0, None, None);
        let (mut out, mut png, mut from, mut to) = (None, false, 0, None);
        let mut moments = false;
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
                "--moments" => moments = true,
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
            source: Source {
                input: input.ok_or("play: needs a scenario or a record file")?,
                seed,
                speed: Speed::new(speed),
                packs: packs.unwrap_or_else(|| clients.join("../packs")),
                install: install
                    .or(env)
                    .unwrap_or_else(|| clients.join("../original-data/install")),
                humans: [false; 2],
            },
            out,
            png,
            from,
            to,
            moments,
        })
    }

    /// The output directory: `clients/out/play/<stem>[-s<seed>]` by default.
    fn dir(&self) -> PathBuf {
        self.out.clone().unwrap_or_else(|| {
            let stem = self.source.stem();
            let name = if self.source.scenario() {
                format!("{stem}-s{}", self.source.seed)
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
    let ground = Ground::load(&options.source)?;
    let mut battle = Battle::new(&ground)?;
    // A scenario is fought as its frames are drawn; past the last command the battle idles: the
    // outline pulses and the stacks fidget.
    let (frames, listed) = timed(|| battle.frames(options.to));
    let frames = frames?;
    if let Some(text) = battle.record_text() {
        let path = dir.join("record.jsonl");
        write(&path, text?.as_bytes())?;
        lines.push(format!("record: {}", path.display()));
    }
    lines.push(format!(
        "{} frames' moments and draw lists in {} ms",
        frames.len(),
        listed.as_millis()
    ));
    if options.moments {
        let times: Vec<u32> = frames.iter().map(|(t, ..)| *t).collect();
        lines.extend(moments(battle.timeline(), &times));
    }
    let mut log = String::new();
    for (t, moment, _) in &frames {
        let _ = writeln!(log, "{}", describe(*t, moment));
    }
    write(&dir.join("frames.txt"), log.as_bytes())?;
    // The creatures' sounds as the timeline starts and stops them (T5b), with their entries.
    let sounds = battle.stack_sounds();
    let named = battle.battle_sounds();
    let mut cues = String::new();
    for c in &battle.timeline().cues {
        // The heal's sound is the battle's (E17 M3 T3b), as the casts' are (T7b-1).
        let name = match named.get(&c.sound) {
            Some(name) => Some(name.as_str()),
            None => sounds.get(&c.stack).and_then(|s| s.name(c.sound)),
        };
        let name = name.unwrap_or("-");
        let what = if c.stop { "stop" } else { "start" };
        let _ = writeln!(cues, "{} {} {:?} {what} {name}", c.at, c.stack, c.sound);
    }
    write(&dir.join("cues.txt"), cues.as_bytes())?;
    battle.load_batch(frames.iter().map(|(_, _, list)| list.as_slice()), &[])?;
    let (drawn, took) = timed(|| {
        let mut drawn = 0;
        for (t, _, list) in frames.iter().filter(|(t, ..)| *t >= options.from) {
            let screen = battle.compose(*t, list)?;
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
    lines.push(format!(
        "{} commands, {} ms at speed {}; {} frames ({drawn} drawn{} in {} ms), \
         their moments in {}, the sounds' cues in cues.txt",
        battle.commands(),
        battle.timeline().end,
        options.source.speed.setting(),
        frames.len(),
        if options.png { " as PNGs" } else { "" },
        took.as_millis(),
        dir.join("frames.txt").display()
    ));
    Ok(lines)
}

/// The cost of a frame's moment at `times` (E17 M5 T1a), a line for each quarter of them: the
/// playhead's, read forward through all of them, and the reference's ([`Moment::at`], each read
/// from the battle's start), in ns a frame.
fn moments(timeline: &Timeline, times: &[u32]) -> Vec<String> {
    let mut head = Playhead::new(timeline);
    let quarter = times.len().div_ceil(4).max(1);
    times
        .chunks(quarter)
        .map(|part| {
            let ((), ours) = timed(|| {
                for &t in part {
                    head.seek(timeline, t);
                    std::hint::black_box(head.moment(timeline));
                }
            });
            let ((), reference) = timed(|| {
                for &t in part {
                    std::hint::black_box(Moment::at(timeline, t));
                }
            });
            let per = |d: std::time::Duration| d.as_nanos() / part.len().max(1) as u128;
            format!(
                "moments {} to {} ms ({} frames): the playhead {} ns a frame, Moment::at {} ns",
                part.first().copied().unwrap_or(0),
                part.last().copied().unwrap_or(0),
                part.len(),
                per(ours),
                per(reference)
            )
        })
        .collect()
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
            "--moments",
        ]))
        .unwrap();
        assert_eq!(o.source.seed, 7);
        assert_eq!(o.source.speed, Speed::new(2));
        assert!(o.png && o.source.scenario());
        assert_eq!((o.from, o.to), (100, Some(900)));
        assert!(o.moments);
        assert_eq!(o.dir(), PathBuf::from("x"));
        let r = Options::parse(&args(&["a/b.jsonl"])).unwrap();
        assert!(!r.source.scenario());
        assert!(r.dir().ends_with("out/play/b"));
        assert!(Options::parse(&args(&[])).is_err());
        assert!(Options::parse(&args(&["a", "--seed", "x"])).is_err());
        assert!(Options::parse(&args(&["a", "--bogus"])).is_err());
        assert!(Options::parse(&args(&["a", "b"])).is_err());
        assert!(Options::parse(&args(&["a", "--to"])).is_err());
    }
}
