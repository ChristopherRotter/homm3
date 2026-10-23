//! The map's real-asset gate (E24 M1 T2): every name of the map's asset tables and of the
//! adventure screen's layout is found where the game searches it and decodes, and each DEF's
//! group 0 holds the frames the view draws (`h3_map_view::check`), the terrains', rivers' and
//! roads' as many as the shipped maps' pictures need. The objects' pictures (T4b; design §2.6,
//! §2.7): every template's DEF of `Objects.txt` and of the shipped maps has a shape and no pixel
//! its `.msk` leaves undrawn (D8); and, listed, the pixels a random object's picture loses when a
//! new game draws it with its random template's masks. It runs only where the user's files are
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
use h3_map::writer::ObjectTemplate;
use h3_map_view::check::{self, DefSummary};
use h3_map_view::objects::Shape;
use h3_map_view::layout::GEM_FRAMES;
use h3_map_view::tables::Tables;

use crate::files;

/// The random objects a new game draws (resolve_random_objects 0x4C9DD0; design §2.7), by their
/// class, and the class of what each becomes: the random artifacts (65–69) artifacts (5), the
/// random monsters (71–75, 162–164) monsters (54), the random resource (76) a pile (79).
const RANDOM: [(u32, u32); 14] = [
    (65, 5),
    (66, 5),
    (67, 5),
    (68, 5),
    (69, 5),
    (71, 54),
    (72, 54),
    (73, 54),
    (74, 54),
    (75, 54),
    (162, 54),
    (163, 54),
    (164, 54),
    (76, 79),
];

/// Object pictures by DEF name upper-cased: the name as first met and its template's class.
type Pictures = BTreeMap<String, (String, u32)>;

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

    let (highest, shipped) = match &options.maps {
        Some(dir) if dir.is_dir() => shipped_maps(dir, &tables, &mut problems)?,
        _ => {
            println!("maps: none (no maps directory)");
            (Highest::new(), Pictures::new())
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

    let templates = files::templates(&archives)?;
    object_pictures(&archives, &templates, &shipped, &mut problems)?;

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

/// Reads every shipped map: the highest picture of each layer's code and the templates' pictures;
/// a code the tables lack and a map without art are problems; a map the reader refuses is
/// reported and skipped.
fn shipped_maps(
    dir: &Path,
    tables: &Tables,
    problems: &mut Vec<String>,
) -> Result<(Highest, Pictures), String> {
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("h3m")))
        .collect();
    paths.sort();
    let mut highest = Highest::new();
    let mut pictures = Pictures::new();
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
        for t in &art.templates {
            pictures
                .entry(t.def.to_ascii_uppercase())
                .or_insert_with(|| (t.def.clone(), t.class));
        }
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
    Ok((highest, pictures))
}

/// The objects' pictures (E24 M1 T4b): every template's DEF of `Objects.txt` and of the shipped
/// maps needs a shape (a problem else) and no pixel its `.msk` leaves undrawn (D8; a problem
/// else); then, listed, the pictures a random object can become that lose pixels when drawn with
/// a random template's masks, as the copy a new game makes keeps them (design §2.7).
fn object_pictures(
    archives: &[h3_formats::lod::LodArchive<'_>],
    templates: &[ObjectTemplate],
    shipped: &Pictures,
    problems: &mut Vec<String>,
) -> Result<(), String> {
    let mut pictures = Pictures::new();
    for t in templates {
        pictures
            .entry(t.def.to_ascii_uppercase())
            .or_insert_with(|| (t.def.clone(), t.class));
    }
    for (key, picture) in shipped {
        pictures.entry(key.clone()).or_insert_with(|| picture.clone());
    }
    let mut shapes = BTreeMap::new();
    let mut lose = 0;
    for (key, (name, _)) in &pictures {
        let shape = match files::load_shape(archives, name) {
            Ok(shape) => shape,
            Err(e) => {
                problems.push(format!("object picture {e}"));
                continue;
            }
        };
        let undrawn = undrawn(archives, name, &shape)?;
        if undrawn.any() {
            lose += 1;
            problems.push(format!(
                "object picture {name}: {} body and {} shadow pixels outside its .msk's masks",
                undrawn.body, undrawn.shadow
            ));
        }
        shapes.insert(key.clone(), shape);
    }
    println!(
        "\nthe objects' pictures (Objects.txt and the shipped maps' templates): {} with a shape \
         of {}, {lose} with pixels outside their .msk's masks",
        shapes.len(),
        pictures.len()
    );

    println!("the random objects' pictures drawn with a random template's masks (design §2.7):");
    let mut cut = 0;
    for (key, (name, class)) in &pictures {
        let Some(&(_, becomes)) = RANDOM.iter().find(|(c, _)| c == class) else {
            continue;
        };
        let Some(random) = shapes.get(key) else {
            continue;
        };
        for (other, (drawn, _)) in pictures.iter().filter(|(_, (_, c))| *c == becomes) {
            let Some(own) = shapes.get(other) else {
                continue;
            };
            if (own.width, own.height, own.colour, own.shadow)
                == (random.width, random.height, random.colour, random.shadow)
            {
                continue;
            }
            let undrawn = undrawn(archives, drawn, random)?;
            if undrawn.any() {
                cut += 1;
                println!(
                    "  {drawn} as {name}: {} body and {} shadow pixels undrawn",
                    undrawn.body, undrawn.shadow
                );
            }
        }
    }
    println!("  {cut} picture(s) lose pixels");
    Ok(())
}

/// The pixels of the DEF `name` the passes never draw with `shape` (its group 0).
fn undrawn(
    archives: &[h3_formats::lod::LodArchive<'_>],
    name: &str,
    shape: &Shape,
) -> Result<check::Undrawn, String> {
    let (_, data) = files::find(archives, name)?.ok_or_else(|| format!("{name}: in no archive"))?;
    check::undrawn(&data, shape).map_err(|e| format!("{name}: does not decode ({e:?})"))
}
