//! `sequences`: the recorded sequences replayed without the original (E24 M2 T2; the epic's M2
//! decision). Each sequence of `clients/map/sequences.toml` (or one) is played by the library's
//! timeline (`h3_map_view::timeline`) from its situation's start and its inputs; the screen is
//! drawn at every millisecond it can change (the view at each draw, the cursor over it), and the
//! box it changed in outside the panels gives the timeline's `screen` lines. The timeline is
//! compared with the one recorded (`timelines/<id>.txt`). With `--frames <dir>` (one sequence)
//! the frame at every time the screen changed and at every time `<dir>/frames/` holds one of the
//! original's (`h3map.py frames`' `t<ms>.npz`) is written as `<dir>/ours/t<ms>.png`, the timeline
//! as `<dir>/ours.txt`. The sequence's set-up and drawing are `crate::paint`'s (shared with
//! `window` since M2 T4a): a walk is played with the hero the live part at 0 selects and the paths
//! its recorded timeline shows.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use h3_map_view::sequence::{Sequence, Sequences};
use h3_map_view::timeline::{self, Cursor, Draw, Rules, Unplayable};
use h3_render::Bugs;
use h3_render::screen::Screen;

use crate::files;
use crate::paint::{self, Assets};

/// The screen the timeline's `screen` lines and the frame check look at: outside the panels
/// (`h3map.py`'s `MASKS["panels"]`; the panels are M3's).
pub const COMPARED: [i32; 4] = [0, 0, 608, 552];

/// The command's options.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    only: Option<String>,
    frames: Option<PathBuf>,
    live: Option<PathBuf>,
    bugs: Bugs,
    install: PathBuf,
    map: PathBuf,
}

impl Options {
    /// Parses `[<id>] [--frames <dir>] [--live <live.toml>] [--bugs fixed|original] [--install
    /// <dir>] [--map <dir>]`.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let mut rest = args.iter();
        let (mut only, mut frames, mut live, mut install, mut map) = (None, None, None, None, None);
        let mut bugs = Bugs::Original;
        while let Some(arg) = rest.next() {
            if !arg.starts_with("--") {
                if only.replace(arg.clone()).is_some() {
                    return Err("sequences: one sequence or all".into());
                }
                continue;
            }
            let value = rest
                .next()
                .ok_or_else(|| format!("sequences: `{arg}` needs a value"))?;
            match arg.as_str() {
                "--frames" => frames = Some(PathBuf::from(value)),
                "--live" => live = Some(PathBuf::from(value)),
                "--bugs" => {
                    bugs = match value.as_str() {
                        "fixed" => Bugs::Fixed,
                        "original" => Bugs::Original,
                        _ => return Err("sequences: --bugs takes fixed or original".into()),
                    };
                }
                "--install" => install = Some(PathBuf::from(value)),
                "--map" => map = Some(PathBuf::from(value)),
                other => return Err(format!("sequences: unknown option `{other}`")),
            }
        }
        if only.is_none() && (frames.is_some() || live.is_some()) {
            return Err("sequences: --frames and --live take one sequence".into());
        }
        Ok(Self {
            only,
            frames,
            live,
            bugs,
            install: install.unwrap_or_else(files::default_install),
            map: map.unwrap_or_else(files::default_map),
        })
    }
}

/// What a sequence gave.
#[derive(Debug, PartialEq, Eq)]
enum Verdict {
    /// Its timeline as recorded: its lines.
    Same(usize),
    /// No timeline recorded: its lines.
    New(usize),
    /// The first line that differs (0-based): ours and the recorded one (empty past the end).
    Changed(usize, String, String),
}

pub fn run(options: &Options) -> ExitCode {
    match all(options) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(message) => {
            eprintln!("sequences: {message}");
            ExitCode::FAILURE
        }
    }
}

/// Every sequence asked for; whether each is as recorded.
fn all(options: &Options) -> Result<bool, String> {
    let tables = files::load_tables(&options.map)?;
    let layout = files::load_layout(&options.map)?;
    let path = options.map.join("sequences.toml");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let sequences = Sequences::parse(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let chosen: Vec<&Sequence> = match &options.only {
        Some(id) => vec![
            sequences
                .get(id)
                .ok_or_else(|| format!("no sequence `{id}` in {}", path.display()))?,
        ],
        None => sequences.sequence.iter().collect(),
    };
    let archive_bytes = files::read_archives(&options.install)?;
    let archives = files::parse_archives(&archive_bytes)?;
    let templates = files::templates(&archives)?;
    let assets = Assets {
        tables: &tables,
        layout: &layout,
        archives: &archives,
        templates: &templates,
        bugs: options.bugs,
    };
    let context = Context {
        assets: &assets,
        options,
    };
    let mut same = 0;
    for sequence in &chosen {
        let id = &sequence.id;
        match context.check(sequence) {
            Ok(Verdict::Same(n)) => {
                same += 1;
                println!("{id}: {n} lines as recorded");
            }
            Ok(Verdict::New(n)) => println!("{id}: {n} lines, new (no recorded timeline)"),
            Ok(Verdict::Changed(line, ours, recorded)) => println!(
                "{id}: CHANGED at line {}: ours `{ours}`, recorded `{recorded}`",
                line + 1
            ),
            Err(message) => println!("{id}: FAILED ({message})"),
        }
    }
    let played = chosen.len();
    println!("sequences: {played} played, {same} as recorded");
    Ok(same == played)
}

/// What every sequence shares.
struct Context<'a> {
    assets: &'a Assets<'a>,
    options: &'a Options,
}

impl Context<'_> {
    /// One sequence: played, drawn, its timeline against the recorded one.
    fn check(&self, sequence: &Sequence) -> Result<Verdict, String> {
        let options = self.options;
        let setup = paint::for_sequence(
            self.assets,
            &options.map,
            &options.install,
            sequence,
            options.live.as_deref(),
        )?;
        let paint::Setup {
            mut painter,
            start,
            actions,
            recorded,
        } = setup;
        let assets = self.assets;
        let rules = Rules::new(assets.tables, assets.layout, assets.bugs);
        let states = timeline::play(rules, &start, &actions, sequence.ms)
            .map_err(|(t, why): (u32, Unplayable)| format!("at {t} ms: {why}"))?;
        let wanted = match &self.options.frames {
            Some(dir) => original_times(dir)?,
            None => BTreeSet::new(),
        };
        let ours = self.options.frames.as_ref().map(|d| d.join("ours"));
        if let Some(dir) = &ours {
            fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        let mut screens = BTreeMap::new();
        let (mut kept, mut current): (Option<Screen>, Option<Screen>) = (None, None);
        let mut shown: Option<(Draw, Cursor)> = None;
        for s in &states {
            if shown.as_ref() != Some(&(s.drawn.clone(), s.cursor)) {
                shown = Some((s.drawn.clone(), s.cursor));
                current = Some(painter.frame(&s.drawn, s.cursor)?);
            }
            let Some(frame) = &current else { continue };
            let new = match kept.as_ref().map(|k| timeline::changed(k, frame, COMPARED)) {
                None => true,
                Some(Some(b)) => {
                    screens.insert(s.t, b);
                    true
                }
                Some(None) => false,
            };
            if new {
                kept.clone_from(&current);
            }
            if let Some(dir) = &ours
                && (new || wanted.contains(&s.t))
            {
                let png = dir.join(format!("t{:05}.png", s.t));
                fs::write(&png, h3_render::png::encode(frame))
                    .map_err(|e| format!("{}: {e}", png.display()))?;
            }
        }
        let lines = timeline::lines(&states, &screens);
        if let Some(dir) = &self.options.frames {
            let out = dir.join("ours.txt");
            fs::write(&out, lines.join("\n") + "\n")
                .map_err(|e| format!("{}: {e}", out.display()))?;
        }
        let Some(recorded) = recorded else {
            return Ok(Verdict::New(lines.len()));
        };
        Ok(compare(&lines, &recorded))
    }
}

/// Our lines against the recorded ones.
fn compare(lines: &[String], recorded: &str) -> Verdict {
    let theirs: Vec<&str> = recorded.lines().collect();
    for n in 0..lines.len().max(theirs.len()) {
        let ours = lines.get(n).map_or("", String::as_str);
        let recorded = theirs.get(n).copied().unwrap_or("");
        if ours != recorded {
            return Verdict::Changed(n, ours.to_string(), recorded.to_string());
        }
    }
    Verdict::Same(lines.len())
}

/// The times of the original's frames in `<dir>/frames/` (`t<ms>.npz`); none if it has none.
fn original_times(dir: &Path) -> Result<BTreeSet<u32>, String> {
    let frames = dir.join("frames");
    let Ok(entries) = fs::read_dir(&frames) else {
        return Ok(BTreeSet::new());
    };
    let mut times = BTreeSet::new();
    for entry in entries {
        let name = entry.map_err(|e| e.to_string())?.file_name();
        let name = name.to_string_lossy();
        if let Some(t) = name
            .strip_prefix('t')
            .and_then(|n| n.strip_suffix(".npz"))
            .and_then(|n| n.parse().ok())
        {
            times.insert(t);
        }
    }
    Ok(times)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(text: &str) -> Vec<String> {
        text.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn parses_its_options() {
        let o = Options::parse(&args("scroll-0 --frames out/x --bugs fixed")).unwrap();
        assert_eq!(o.only.as_deref(), Some("scroll-0"));
        assert_eq!(o.frames, Some(PathBuf::from("out/x")));
        assert_eq!(o.bugs, Bugs::Fixed);
        assert_eq!(Options::parse(&[]).unwrap().bugs, Bugs::Original);
        for (bad, message) in [
            ("a b", "sequences: one sequence or all"),
            (
                "--frames out/x",
                "sequences: --frames and --live take one sequence",
            ),
            ("a --bugs some", "sequences: --bugs takes fixed or original"),
            ("a --frames", "sequences: `--frames` needs a value"),
            ("a --scale 2", "sequences: unknown option `--scale`"),
        ] {
            assert_eq!(
                Options::parse(&args(bad)),
                Err(message.to_string()),
                "{bad}"
            );
        }
    }

    #[test]
    fn compares_line_by_line() {
        let ours = ["0 view 1 2 0".to_string(), "5 objects 1".to_string()];
        assert_eq!(
            compare(&ours, "0 view 1 2 0\n5 objects 1\n"),
            Verdict::Same(2)
        );
        assert_eq!(
            compare(&ours, "0 view 1 2 0\n5 objects 2\n"),
            Verdict::Changed(1, "5 objects 1".into(), "5 objects 2".into())
        );
        assert_eq!(
            compare(&ours, "0 view 1 2 0\n"),
            Verdict::Changed(1, "5 objects 1".into(), String::new())
        );
    }
}
