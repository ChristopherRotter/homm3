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
use h3_map::writer::ObjectTemplate;
use h3_map_view::art::{CELLS, Layer, MapArt, TemplateArt};
use h3_map_view::check::{self, DefSummary};
use h3_map_view::layout::GEM_FRAMES;
use h3_map_view::objects::{self, Shape};
use h3_map_view::tables::{FACTIONS, Tables};

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

/// The artifacts' class.
const ARTIFACT: u32 = 5;

/// Object pictures by DEF name upper-cased: the name as first met and its template's class and
/// subclass.
type Pictures = BTreeMap<String, (String, u32, u32)>;

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

    // The shipped maps' checks need their directory (E24 M1 T6b, the reviewer's R2-1).
    let (highest, shipped, visits) = match &options.maps {
        Some(dir) if dir.is_dir() => shipped_maps(dir, &tables, &mut problems)?,
        dir => {
            let dir = dir
                .as_deref()
                .map_or_else(String::new, |d| d.display().to_string());
            problems.push(format!("maps: {dir} is no directory"));
            (Highest::new(), Pictures::new(), Vec::new())
        }
    };

    println!(
        "\nDEFs (group 0): name, archive, frames held / drawn, encodings, run pixels by index"
    );
    let mut needs: Vec<(String, String, usize, u32)> = Vec::new();
    for need in tables.defs() {
        let maps = need
            .layer
            .and_then(|key| highest.get(&key))
            .map_or(0, |(picture, _)| usize::from(*picture) + 1);
        needs.push((need.what, need.name, need.frames.max(maps), need.groups));
    }
    for (i, gem) in layout.view.gems.iter().enumerate() {
        needs.push((format!("gem {i}"), gem.def.clone(), GEM_FRAMES, 1));
    }
    for (what, name, frames, groups) in &needs {
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
                for p in check::problems(&summary, *frames, *groups) {
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
    let (pictures, shapes) = object_pictures(&archives, &templates, &shipped, &mut problems)?;
    let mut visits = visits;
    visits.extend(templates.iter().map(|t| TemplateArt {
        def: t.def.clone(),
        passable: t.passable,
        visitable: t.visitable,
        class: t.class,
        subclass: t.subclass,
        priority: t.priority,
    }));
    entrances_outside(&visits, &shapes, &mut problems);
    owners_and_towns(&archives, &tables, &pictures, &mut problems)?;

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

/// Reads every shipped map: the highest picture of each layer's code, the templates' pictures and
/// the templates; a code the tables lack, a map without art, a map the reader refuses and a
/// directory without a map it reads are problems (E24 M1 T6b, the reviewer's R2-1).
fn shipped_maps(
    dir: &Path,
    tables: &Tables,
    problems: &mut Vec<String>,
) -> Result<(Highest, Pictures, Vec<TemplateArt>), String> {
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("h3m")))
        .collect();
    paths.sort();
    let mut highest = Highest::new();
    let mut pictures = Pictures::new();
    let mut templates: BTreeMap<(String, [u8; 6]), TemplateArt> = BTreeMap::new();
    let (mut read, mut refused) = (0, Vec::new());
    // The owned objects without an entrance on the map (MCL-Q4: their flags take tile (0, 0)'s
    // owner in the original).
    let mut no_entrance = Vec::new();
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
        for (index, object) in art.objects.iter().enumerate() {
            let class = art.templates.get(object.template).map(|t| t.class);
            let owned = class.is_some_and(|c| tables.map.owners.classes.contains(&c));
            if owned && objects::entrance(&art, index).is_none() {
                no_entrance.push(format!("{name}: object {index} at {:?}", object.anchor));
            }
        }
        for t in &art.templates {
            pictures
                .entry(t.def.to_ascii_uppercase())
                .or_insert_with(|| (t.def.clone(), t.class, t.subclass));
            templates
                .entry((t.def.to_ascii_uppercase(), t.visitable))
                .or_insert_with(|| t.clone());
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
        problems.push(format!("the reader refuses {r}"));
    }
    if read == 0 {
        problems.push(format!("maps: none read from {}", dir.display()));
    }
    println!(
        "owned objects without an entrance on the map: {}",
        no_entrance.len()
    );
    for o in &no_entrance {
        println!("  {o}");
    }
    println!("the highest picture by layer and code (the map that gives it):");
    for ((layer, code), (picture, map)) in &highest {
        println!("  {layer:?} {code}: {picture} ({map})");
    }
    Ok((highest, pictures, templates.into_values().collect()))
}

/// The templates with a visitable cell outside their picture's `.msk` (E24 M1 T6b, the
/// reviewer's R1-5): the original looks for an entrance only within the `.msk`'s width and height
/// (object_entrance_tile 0x4FEC30), `objects::entrance` in all 8 × 6 cells, the same while no
/// template has one: each a problem.
fn entrances_outside(
    templates: &[TemplateArt],
    shapes: &BTreeMap<String, Shape>,
    problems: &mut Vec<String>,
) {
    let mut outside = 0;
    for t in templates {
        let Some(shape) = shapes.get(&t.def.to_ascii_uppercase()) else {
            continue;
        };
        let beyond = (0..CELLS.0)
            .flat_map(|dx| (0..CELLS.1).map(move |dy| (dx, dy)))
            .any(|(dx, dy)| (dx >= shape.width || dy >= shape.height) && t.visitable(dx, dy));
        if beyond {
            outside += 1;
            problems.push(format!(
                "template {} (class {}): a visitable cell outside its .msk's {} x {} cells",
                t.def, t.class, shape.width, shape.height
            ));
        }
    }
    println!(
        "the templates (Objects.txt and the shipped maps') with a visitable cell outside their \
         .msk: {outside} of {}",
        templates.len()
    );
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
) -> Result<(Pictures, BTreeMap<String, Shape>), String> {
    let mut pictures = Pictures::new();
    for t in templates {
        pictures
            .entry(t.def.to_ascii_uppercase())
            .or_insert_with(|| (t.def.clone(), t.class, t.subclass));
    }
    for (key, picture) in shipped {
        pictures
            .entry(key.clone())
            .or_insert_with(|| picture.clone());
    }
    let mut shapes = BTreeMap::new();
    let mut lose = 0;
    for (key, (name, ..)) in &pictures {
        let shape = match files::load_shape(archives, name) {
            Ok(shape) => shape,
            Err(e) => {
                problems.push(format!("object picture {e}"));
                continue;
            }
        };
        // D8 checks group 0, the passes draw the first group (0x410EA1): the same group
        // (E24 M1 T6b, the reviewer's R2-9).
        let first = files::find(archives, name)?
            .and_then(|(_, data)| check::summarize(&data).ok())
            .and_then(|summary| summary.groups.first().map(|g| g.0));
        if first != Some(0) {
            problems.push(format!(
                "object picture {name}: its first group is not group 0"
            ));
        }
        match undrawn(archives, name, &shape) {
            Ok(undrawn) if undrawn.any() => {
                lose += 1;
                problems.push(format!(
                    "object picture {name}: {} body and {} shadow pixels outside its .msk's \
                     masks",
                    undrawn.body, undrawn.shadow
                ));
            }
            Ok(_) => {}
            Err(e) => problems.push(format!("object picture {e}")),
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
    for (key, (name, class, _)) in &pictures {
        let Some(&(_, becomes)) = RANDOM.iter().find(|(c, _)| c == class) else {
            continue;
        };
        let Some(random) = shapes.get(key) else {
            continue;
        };
        for (other, (drawn, ..)) in pictures.iter().filter(|(_, (_, c, _))| *c == becomes) {
            let Some(own) = shapes.get(other) else {
                continue;
            };
            if (own.width, own.height, own.colour, own.shadow)
                == (random.width, random.height, random.colour, random.shadow)
            {
                continue;
            }
            // The client draws an artifact's own picture with its own masks on its random
            // template's entries (MCL-Q3's fix): cells outside them it never draws (E24 M1 T6b,
            // the reviewer's R1-7).
            if becomes == ARTIFACT && (own.width > random.width || own.height > random.height) {
                problems.push(format!(
                    "{drawn}: {} x {} cells, the random template {name}'s {} x {}: the client \
                     never draws the cells outside",
                    own.width, own.height, random.width, random.height
                ));
            }
            match undrawn(archives, drawn, random) {
                Ok(undrawn) if undrawn.any() => {
                    cut += 1;
                    println!(
                        "  {drawn} as {name}: {} body and {} shadow pixels undrawn",
                        undrawn.body, undrawn.shadow
                    );
                }
                Ok(_) => {}
                Err(e) => problems.push(format!("random object picture {e}")),
            }
        }
    }
    println!("  {cut} picture(s) lose pixels");
    Ok((pictures, shapes))
}

/// The owned objects' colours (E24 M1 T5b1; design §2.9): the owners' palette decodes, and its
/// entries are listed; the object pictures with flag pixels (runs of index 5) by class, those of a
/// class whose flags the view skips listed (D10); the towns' pictures drawn with each town
/// template's masks (a new game's copy keeps them, 0x4C9990), those that lose pixels listed.
fn owners_and_towns(
    archives: &[h3_formats::lod::LodArchive<'_>],
    tables: &Tables,
    pictures: &Pictures,
    problems: &mut Vec<String>,
) -> Result<(), String> {
    let owners = &tables.map.owners;
    match files::find(archives, &owners.palette)? {
        None => problems.push(format!("{}: in no archive", owners.palette)),
        Some((archive, data)) => match decode_pal(&data) {
            Ok(pal) => {
                let entries: Vec<String> = (0..=8u8)
                    .map(|i| {
                        let c = pal.palette.0[usize::from(owners.neutral + i)];
                        format!("{}:{},{},{}", owners.neutral + i, c.r, c.g, c.b)
                    })
                    .collect();
                println!(
                    "\nthe owners' palette {} ({archive}): {}",
                    owners.palette,
                    entries.join(" ")
                );
            }
            Err(e) => problems.push(format!("{}: does not decode ({e:?})", owners.palette)),
        },
    }
    let mut flagged: BTreeMap<u32, Vec<String>> = BTreeMap::new();
    for (name, class, _) in pictures.values() {
        let Some((_, data)) = files::find(archives, name)? else {
            continue;
        };
        if check::summarize(&data).is_ok_and(|summary| summary.run_pixels[5] > 0) {
            flagged.entry(*class).or_default().push(name.clone());
        }
    }
    println!("the object pictures with flag pixels (runs of 5), by class:");
    for (class, names) in &flagged {
        let skipped = if owners.classes.contains(class) {
            ""
        } else {
            " (skipped: not an owned class)"
        };
        println!("  {class}: {}{skipped}", names.len());
        if !skipped.is_empty() {
            println!("    {}", names.join(" "));
        }
    }
    let towns = &tables.map.towns;
    println!("the towns' pictures drawn with a town template's masks (design §2.9):");
    let mut cut = 0;
    for (name, class, subclass) in pictures.values().filter(|(_, c, _)| *c == 77 || *c == 98) {
        // A name in no archive is a problem, and the gate goes on (E24 M1 T6b, R2-7).
        let template = match files::load_shape(archives, name) {
            Ok(template) => template,
            Err(e) => {
                problems.push(format!("town template {e}"));
                continue;
            }
        };
        let defs = [&towns.village, &towns.fort, &towns.capitol];
        let factions: Vec<usize> = match (*class, usize::try_from(*subclass)) {
            (98, Ok(f)) => vec![f],
            _ => (0..FACTIONS).collect(),
        };
        for drawn in factions
            .iter()
            .flat_map(|&f| defs.iter().filter_map(move |list| list.get(f)))
        {
            match undrawn(archives, drawn, &template) {
                Ok(undrawn) if undrawn.any() => {
                    cut += 1;
                    println!(
                        "  {drawn} as {name}: {} body and {} shadow pixels undrawn",
                        undrawn.body, undrawn.shadow
                    );
                }
                Ok(_) => {}
                Err(e) => problems.push(format!("town picture {e}")),
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
