//! The map's real-asset gate (E24 M1 T2): every name of the map's asset tables and of the
//! adventure screen's layout is found where the game searches it and decodes, and each DEF's
//! group 0 holds the frames the view draws (`h3_map_view::check`), the terrains', rivers' and
//! roads' as many as the shipped maps' pictures need. It runs only where the user's files are
//! (ADR-0009): a copy of the installation, which it only reads. It reads whole archives, as
//! `h3-view gate` does.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use h3_assets::palette::decode_pal;
use h3_assets::pcx::{Pcx, decode_pcx};
use h3_map::Limits;
use h3_map_view::art::{Layer, MapArt};
use h3_map_view::check::{self, DefSummary};
use h3_map_view::layout::GEM_FRAMES;
use h3_map_view::tables::Tables;

use crate::files;

/// The command's options.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    install: PathBuf,
    map: PathBuf,
    maps: Option<PathBuf>,
}

impl Options {
    /// Parses `[--install <dir>] [--map <dir>] [--maps <dir>]`.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let (mut install, mut map, mut maps) = (None, None, None);
        let mut rest = args.iter();
        while let Some(flag) = rest.next() {
            let slot = match flag.as_str() {
                "--install" => &mut install,
                "--map" => &mut map,
                "--maps" => &mut maps,
                other => return Err(format!("gate: unknown option `{other}`")),
            };
            let value = rest
                .next()
                .ok_or_else(|| format!("gate: `{flag}` needs a path"))?;
            *slot = Some(PathBuf::from(value));
        }
        let install = install.unwrap_or_else(files::default_install);
        Ok(Self {
            maps: maps.or_else(|| Some(install.join("Maps"))),
            install,
            map: map.unwrap_or_else(files::default_map),
        })
    }
}

pub fn run(options: &Options) -> ExitCode {
    match gate(options) {
        Ok(0) => {
            println!(
                "gate: every name found and decoded, every DEF with the frames the view draws"
            );
            ExitCode::SUCCESS
        }
        Ok(n) => {
            eprintln!("gate: {n} problem(s)");
            ExitCode::FAILURE
        }
        Err(message) => {
            eprintln!("gate: {message}");
            ExitCode::FAILURE
        }
    }
}

/// The highest picture the shipped maps give each layer's code, and the map that gives it.
type Highest = BTreeMap<(Layer, u8), (u8, String)>;

fn gate(options: &Options) -> Result<usize, String> {
    let tables = files::load_tables(&options.map)?;
    let layout = files::load_layout(&options.map)?;
    let bytes = files::read_archives(&options.install)?;
    let archives = files::parse_archives(&bytes)?;
    let find = |name: &str| files::find(&archives, name);
    let mut problems = Vec::new();

    let highest = match &options.maps {
        Some(dir) if dir.is_dir() => shipped_maps(dir, &tables, &mut problems)?,
        _ => {
            println!("maps: none (no maps directory)");
            Highest::new()
        }
    };

    println!(
        "\nDEFs (group 0): name, archive, frames held / drawn, encodings, run pixels by index"
    );
    let mut needs: Vec<(String, String, usize)> = Vec::new();
    for need in tables.defs() {
        let maps = need
            .layer
            .and_then(|key| highest.get(&key))
            .map_or(0, |(picture, _)| usize::from(*picture) + 1);
        needs.push((need.what, need.name, need.frames.max(maps)));
    }
    for (i, gem) in layout.view.gems.iter().enumerate() {
        needs.push((format!("gem {i}"), gem.def.clone(), GEM_FRAMES));
    }
    for (what, name, frames) in &needs {
        let Some((archive, data)) = find(name)? else {
            problems.push(format!("{what}: {name} in no archive"));
            continue;
        };
        match check::summarize(&data) {
            Ok(summary) => {
                println!(
                    "  {what:<13} {name:<12} {archive:<13} {}",
                    line(&summary, *frames)
                );
                for p in check::problems(&summary, *frames) {
                    problems.push(format!("{what}: {name}: {p}"));
                }
            }
            Err(e) => problems.push(format!("{what}: {name}: does not decode ({e:?})")),
        }
    }

    println!("\nthe layout's pictures:");
    match find(&layout.picture)? {
        None => problems.push(format!("{}: in no archive", layout.picture)),
        Some((archive, data)) => match decode_pcx(&data) {
            Ok(pcx) => {
                let size = [pcx.width(), pcx.height()];
                let kind = match pcx {
                    Pcx::Indexed(_) => "8-bit",
                    Pcx::Rgb(_) => "24-bit",
                };
                println!(
                    "  {} {archive} {}x{} {kind}",
                    layout.picture, size[0], size[1]
                );
                if size != layout.frame {
                    problems.push(format!(
                        "{}: {size:?}, the frame {:?}",
                        layout.picture, layout.frame
                    ));
                }
                if kind != "8-bit" {
                    problems.push(format!("{}: not 8-bit: no player colours", layout.picture));
                }
            }
            Err(e) => problems.push(format!("{}: does not decode ({e:?})", layout.picture)),
        },
    }
    match find(&layout.player_palette)? {
        None => problems.push(format!("{}: in no archive", layout.player_palette)),
        Some((archive, data)) => match decode_pal(&data) {
            Ok(_) => println!("  {} {archive} decodes", layout.player_palette),
            Err(e) => problems.push(format!(
                "{}: does not decode ({e:?})",
                layout.player_palette
            )),
        },
    }

    if !problems.is_empty() {
        println!("\nproblems:");
        for p in &problems {
            println!("  {p}");
        }
    }
    Ok(problems.len())
}

/// A DEF's line: frames held and drawn, encodings, run pixels by index (only those with any).
fn line(summary: &DefSummary, frames: usize) -> String {
    let encodings: Vec<String> = summary
        .encodings
        .iter()
        .map(|e| e.number().to_string())
        .collect();
    let runs: Vec<String> = summary
        .run_pixels
        .iter()
        .enumerate()
        .filter(|(_, n)| **n > 0)
        .map(|(i, n)| format!("{i}:{n}"))
        .collect();
    format!(
        "{:>3} / {frames:<3} enc {:<5} runs {}",
        summary.frames,
        encodings.join(","),
        if runs.is_empty() {
            "none".to_string()
        } else {
            runs.join(" ")
        }
    )
}

/// Reads every shipped map: the highest picture of each layer's code, a code the tables lack and
/// a map without art are problems; a map the reader refuses is reported and skipped.
fn shipped_maps(
    dir: &Path,
    tables: &Tables,
    problems: &mut Vec<String>,
) -> Result<Highest, String> {
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("h3m")))
        .collect();
    paths.sort();
    let mut highest = Highest::new();
    let (mut read, mut refused) = (0, Vec::new());
    for path in &paths {
        let name = path
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        let file = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let raw = match h3_map::read(&file, &Limits::default()) {
            Ok(raw) => raw,
            Err(e) => {
                refused.push(format!("{name} ({e})"));
                continue;
            }
        };
        let art = match MapArt::from_raw(&raw) {
            Ok(art) => art,
            Err(e) => {
                problems.push(format!("map {name}: no art: {e}"));
                continue;
            }
        };
        read += 1;
        for layer in [Layer::Terrain, Layer::River, Layer::Road] {
            for (code, picture) in art.pictures(layer) {
                if !tables.layer(layer).contains_key(&code) {
                    problems.push(format!(
                        "map {name}: {layer:?} code {code} not in the tables"
                    ));
                }
                let slot = highest
                    .entry((layer, code))
                    .or_insert((picture, name.clone()));
                if picture > slot.0 {
                    *slot = (picture, name.clone());
                }
            }
        }
    }
    println!(
        "maps: {read} of {} read from {}",
        paths.len(),
        dir.display()
    );
    for r in &refused {
        println!("  the reader refuses {r}");
    }
    println!("the highest picture by layer and code (the map that gives it):");
    for ((layer, code), (picture, map)) in &highest {
        println!("  {layer:?} {code}: {picture} ({map})");
    }
    Ok(highest)
}
