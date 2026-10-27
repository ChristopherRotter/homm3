//! `sequences`: the recorded sequences replayed without the original (E24 M2 T2; the epic's M2
//! decision). Each sequence of `clients/map/sequences.toml` (or one) is played by the library's
//! timeline (`h3_map_view::timeline`) from its situation's start and its inputs; the screen is
//! drawn at every millisecond it can change (the view at each draw, the cursor over it), and the
//! box it changed in outside the panels gives the timeline's `screen` lines. The timeline is
//! compared with the one recorded (`timelines/<id>.txt`). With `--frames <dir>` (one sequence)
//! the frame at every time the screen changed and at every time `<dir>/frames/` holds one of the
//! original's (`h3map.py frames`' `t<ms>.npz`) is written as `<dir>/ours/t<ms>.png`, the timeline
//! as `<dir>/ours.txt`. A walk (M2 T3a) is played with the hero the live part at 0 selects and
//! the paths its recorded timeline shows: its lines but the `screen` ones are compared, its
//! frames are M2 T3b's (the walk drawn).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use h3_formats::lod::LodArchive;
use h3_map::Limits;
use h3_map::writer::ObjectTemplate;
use h3_map_view::art::MapArt;
use h3_map_view::batch::{self, Batch};
use h3_map_view::compose::{FORMAT, compose};
use h3_map_view::cursor::Cursor as Sprite;
use h3_map_view::layout::AdventureLayout;
use h3_map_view::live::{Live, Moment};
use h3_map_view::objects::{self, Shapes};
use h3_map_view::scene::{self, ViewAt};
use h3_map_view::sequence::{self, Sequence, Sequences};
use h3_map_view::situation::Situation;
use h3_map_view::tables::Tables;
use h3_map_view::timeline::{self, Cursor, Draw, Rules, Start, State, Unplayable};
use h3_render::Bugs;
use h3_render::screen::Screen;

use crate::{files, render, situation};

/// Where the harness parks the pointer (`h3adv.py`'s `PARK`, over the right panel): a sequence's
/// start.
pub const PARK: [i32; 2] = [700, 300];

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
    /// A walk whose lines are as recorded but its screens, which are M2 T3b's: its lines.
    Walk(usize),
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

/// Every sequence asked for; whether each is as recorded (a walk counts as such).
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
    let context = Context {
        tables: &tables,
        layout: &layout,
        archives: &archives,
        templates: &templates,
        options,
    };
    let (mut same, mut walks) = (0, 0);
    for sequence in &chosen {
        let id = &sequence.id;
        match context.check(sequence) {
            Ok(Verdict::Same(n)) => {
                same += 1;
                println!("{id}: {n} lines as recorded");
            }
            Ok(Verdict::Walk(n)) => {
                same += 1;
                walks += 1;
                println!("{id}: {n} lines as recorded but the screens (a walk's: M2 T3b)");
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
    println!(
        "sequences: {played} played, {same} as recorded; {walks} walks' screens left to M2 T3b"
    );
    Ok(same == played)
}

/// What every sequence shares.
struct Context<'a> {
    tables: &'a Tables,
    layout: &'a AdventureLayout,
    archives: &'a [LodArchive<'a>],
    templates: &'a [ObjectTemplate],
    options: &'a Options,
}

impl Context<'_> {
    /// One sequence: played, drawn, its timeline against the recorded one.
    fn check(&self, sequence: &Sequence) -> Result<Verdict, String> {
        let walks = sequence.walks();
        if walks && self.options.frames.is_some() {
            return Err("a walk's frames are M2 T3b's (the walk drawn)".into());
        }
        let map = &self.options.map;
        let path = map
            .join("situations")
            .join(format!("{}.toml", sequence.situation));
        let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let situation = Situation::parse(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        let [x, y, z] = situation.view;
        let view = ViewAt {
            x,
            y,
            z: u8::try_from(z).map_err(|_| format!("{}: the view's level", path.display()))?,
        };
        let recorded_path = map.join("timelines").join(format!("{}.txt", sequence.id));
        let recorded = fs::read_to_string(&recorded_path).ok();
        let bytes = situation::map_bytes(&situation, &self.options.install, self.templates)?;
        let raw = h3_map::parse(&bytes, &Limits::default()).map_err(|e| e.to_string())?;
        let live_path = self.options.live.clone().unwrap_or_else(|| {
            map.join("timelines")
                .join(format!("{}-live.toml", sequence.id))
        });
        let mut painter = Painter::new(self, &raw, &live_path, view)?;
        // A walk's hero is the live part's selected one, its paths the ones the recording shows.
        let (hero, paths) = if walks {
            let live = &painter.live;
            let hero = sequence::selected(&live.heroes, live.viewer)
                .map_err(|e| format!("{}: {e}", live_path.display()))?;
            let paths = sequence.paths(recorded.as_deref().unwrap_or_default())?;
            (Some(hero), paths)
        } else {
            (None, Vec::new())
        };
        let start = Start {
            view,
            size: i32::from(situation.size),
            scroll_speed: sequence.scroll_speed()?,
            hero_speed: sequence.hero_speed()?,
            pointer: PARK,
            hero,
            paths,
        };
        let rules = Rules::new(self.tables, self.layout, self.options.bugs);
        let states = timeline::play(rules, &start, &sequence.actions()?, sequence.ms)
            .map_err(|(t, why): (u32, Unplayable)| format!("at {t} ms: {why}"))?;
        if walks {
            let lines = timeline::lines(&states, &BTreeMap::new());
            let Some(recorded) = recorded else {
                return Ok(Verdict::New(lines.len()));
            };
            let theirs: Vec<&str> = recorded
                .lines()
                .filter(|l| !l.contains(" screen "))
                .collect();
            return Ok(match compare(&lines, &theirs.join("\n")) {
                Verdict::Same(n) => Verdict::Walk(n),
                other => other,
            });
        }
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
                current = Some(painter.frame(s)?);
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

/// The sequence's drawing: the map's art, its live part at 0, the shapes and the assets decoded
/// as the draws need them; the last view drawn kept.
struct Painter<'a> {
    context: &'a Context<'a>,
    art: MapArt,
    live: Live,
    /// The live part's moment at 0: the draws' steps count from it.
    base: Moment,
    shapes: Shapes,
    batch: Batch,
    added: BTreeSet<String>,
    view: Option<(Draw, Screen)>,
    cursors: BTreeMap<u32, Sprite>,
}

impl<'a> Painter<'a> {
    fn new(
        context: &'a Context<'a>,
        raw: &h3_map::RawMap,
        live_path: &PathBuf,
        view: ViewAt,
    ) -> Result<Self, String> {
        let tables = context.tables;
        let mut art = MapArt::from_raw(raw).map_err(|e| e.to_string())?;
        let mut live = Live::at_start(raw, &art, 0, &tables.map.towns);
        render::read_live(live_path, view, &mut art, &mut live, context.templates)?;
        let archives = context.archives;
        let mut shapes = files::load_shapes(archives, &objects::pictures(&art, &live))?;
        let heroes: Vec<String> = tables
            .map
            .heroes
            .defs()
            .into_iter()
            .map(|(d, _)| d)
            .collect();
        files::load_frames(archives, &heroes, &mut shapes)?;
        let base = live.moment;
        let mut painter = Self {
            context,
            art,
            live,
            base,
            shapes,
            batch: Batch::default(),
            added: BTreeSet::new(),
            view: None,
            cursors: BTreeMap::new(),
        };
        painter.add([context.layout.cursor.def.to_ascii_uppercase()])?;
        Ok(painter)
    }

    /// Decodes the named assets not decoded yet.
    fn add(&mut self, names: impl IntoIterator<Item = String>) -> Result<(), String> {
        for name in names {
            if self.added.contains(&name) {
                continue;
            }
            let (_, data) = files::find(self.context.archives, &name)?
                .ok_or(format!("{name}: in no archive"))?;
            self.batch.add(&name, &data).map_err(|e| e.to_string())?;
            self.added.insert(name);
        }
        Ok(())
    }

    /// The view as a draw shows it (the last one kept).
    fn view(&mut self, draw: &Draw) -> Result<Screen, String> {
        if let Some((d, screen)) = &self.view
            && d == draw
        {
            return Ok(screen.clone());
        }
        let base = self.base;
        self.live.moment = Moment {
            cycles: base.cycles.wrapping_add(draw.cycles),
            river_cycles: base.river_cycles.wrapping_add(draw.cycles),
            frames: base.frames.wrapping_add(draw.frames),
        };
        let context = self.context;
        let scene = scene::scene(
            &self.art,
            &self.live,
            context.tables,
            context.layout,
            &self.shapes,
            draw.view,
            context.options.bugs,
        )
        .map_err(|e| e.to_string())?;
        self.add(batch::names(&scene))?;
        let screen =
            compose(&scene, &self.batch, context.layout.frame).map_err(|e| e.to_string())?;
        self.view = Some((draw.clone(), screen.clone()));
        Ok(screen)
    }

    /// The screen a state shows: its view, the cursor over it.
    fn frame(&mut self, s: &State) -> Result<Screen, String> {
        let mut screen = self.view(&s.drawn)?;
        let Cursor {
            frame,
            at: [x, y],
            shown,
        } = s.cursor;
        if !shown {
            return Ok(screen);
        }
        let art = &self.context.layout.cursor;
        if !self.cursors.contains_key(&frame) {
            let [hx, hy] = usize::try_from(frame)
                .ok()
                .and_then(|f| art.hot.get(f).copied())
                .unwrap_or_default();
            let hot = (
                u32::try_from(hx).unwrap_or(0),
                u32::try_from(hy).unwrap_or(0),
            );
            let sprite = self
                .batch
                .sprite(&art.def.to_ascii_uppercase())
                .and_then(|s| Sprite::read(s, frame, hot, FORMAT))
                .ok_or_else(|| format!("{}: no frame {frame}", art.def))?;
            self.cursors.insert(frame, sprite);
        }
        if let Some(cursor) = self.cursors.get(&frame) {
            cursor.draw(&mut screen, x, y);
        }
        Ok(screen)
    }
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
