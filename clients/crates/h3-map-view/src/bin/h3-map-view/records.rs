//! `h3-map-view record` (E24 M4 T4c, design §5.17): a whole game's record played without a
//! recording, headless, to its end: its start, the viewer's commands as the map's input
//! (`h3_map_view::director`), its other players' turns in sight as a computer's; every screen it
//! shows drawn, its timeline written where asked.

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use h3_map_view::timeline::{self, Draw, Rules};
use h3_render::Bugs;

use crate::paint::{self, Assets, Record};
use crate::{files, window};

/// The command's options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    record: PathBuf,
    record_map: PathBuf,
    packs: PathBuf,
    install: PathBuf,
    map: PathBuf,
    bugs: Bugs,
    timeline: Option<PathBuf>,
}

impl Options {
    /// Parses `<record.jsonl> --record-map <map.h3m> [--packs <dir>] [--timeline <file>]
    /// [--bugs fixed|original] [--install <dir>] [--map <dir>]`.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let (mut record, mut record_map, mut packs, mut timeline) = (None, None, None, None);
        let (mut install, mut map, mut bugs) = (None, None, Bugs::Fixed);
        let mut rest = args.iter();
        while let Some(arg) = rest.next() {
            let mut value = || {
                rest.next()
                    .cloned()
                    .ok_or_else(|| format!("record: `{arg}` needs a value"))
            };
            match arg.as_str() {
                "--record-map" => record_map = Some(PathBuf::from(value()?)),
                "--packs" => packs = Some(PathBuf::from(value()?)),
                "--timeline" => timeline = Some(PathBuf::from(value()?)),
                "--install" => install = Some(PathBuf::from(value()?)),
                "--map" => map = Some(PathBuf::from(value()?)),
                "--bugs" => {
                    bugs = match value()?.as_str() {
                        "fixed" => Bugs::Fixed,
                        "original" => Bugs::Original,
                        _ => return Err("record: --bugs takes fixed or original".into()),
                    };
                }
                flag if flag.starts_with("--") => {
                    return Err(format!("record: unknown option `{flag}`"));
                }
                name if record.is_none() => record = Some(PathBuf::from(name)),
                extra => return Err(format!("record: unexpected `{extra}`")),
            }
        }
        Ok(Self {
            record: record.ok_or("record: needs a record")?,
            record_map: record_map.ok_or("record: needs --record-map")?,
            packs: packs.unwrap_or_else(files::default_packs),
            install: install.unwrap_or_else(files::default_install),
            map: map.unwrap_or_else(files::default_map),
            bugs,
            timeline,
        })
    }
}

/// Runs the command.
pub fn run(options: &Options) -> ExitCode {
    match play(options) {
        Ok(line) => {
            println!("{line}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("record: {message}");
            ExitCode::FAILURE
        }
    }
}

/// The record played to its end, each screen drawn; the line that says so.
fn play(options: &Options) -> Result<String, String> {
    let mut tables = files::load_tables(&options.map)?;
    let layout = files::load_layout(&options.map)?;
    let archive_bytes = files::read_archives(&options.install)?;
    let archives = files::parse_archives(&archive_bytes)?;
    tables.texts = crate::texts::catalogue(&options.install, &options.map, &archives)?;
    let templates = files::templates(&archives)?;
    let assets = Assets {
        tables: &tables,
        layout: &layout,
        archives: &archives,
        templates: &templates,
        bugs: options.bugs,
    };
    let record = Record {
        path: &options.record,
        map: &options.record_map,
        packs: &options.packs,
    };
    let (setup, end) = paint::for_record(&assets, record, window::RECORD_LIMIT)?;
    let paint::Setup {
        mut painter,
        start,
        actions,
        ..
    } = setup;
    let rules = Rules::new(&tables, &layout, options.bugs);
    let states = timeline::play(rules, &start, &actions, end)
        .map_err(|(t, why)| format!("at {t} ms: {why}"))?;
    // Every screen the record shows drawn (a draw the painter cannot make fails here).
    let mut shown: Option<&Draw> = None;
    let mut screens = 0;
    for s in &states {
        if shown != Some(&s.drawn) {
            painter.screen(&s.drawn, s.buttons, s.cursor)?;
            shown = Some(&s.drawn);
            screens += 1;
        }
    }
    if let Some(path) = &options.timeline {
        let lines = timeline::lines(&states, &Default::default());
        fs::write(path, lines.join("\n") + "\n").map_err(|e| format!("{}: {e}", path.display()))?;
    }
    let inputs: usize = actions.values().map(Vec::len).sum();
    Ok(format!(
        "{}: viewed to its end at {end} ms ({inputs} inputs, {screens} screens drawn)",
        options.record.display()
    ))
}
