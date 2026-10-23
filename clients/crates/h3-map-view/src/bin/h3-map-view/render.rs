//! `render`: a still of the adventure screen (E24 M1 T3b1): a map's static art from its `.h3m`,
//! the live part at the map's start (every tile explored but a chosen rectangle, the moment from
//! a time), the objects' shapes and the scene at a view, its assets from the installation copy's
//! archives, composed and written as a PNG. The comparisons with the original (T3b2, T4a, T4b)
//! give the live part read from its memory (`--live`: the palette step, the frame step, the
//! viewer, the objects' phases and what the random ones became).

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use h3_map::Limits;
use h3_map::writer::ObjectTemplate;
use h3_map_view::art::MapArt;
use h3_map_view::batch::{self, Batch};
use h3_map_view::compose::compose;
use h3_map_view::live::{Drawn, Live, Moment};
use h3_map_view::objects;
use h3_map_view::scene::{self, Draw, ViewAt};
use h3_render::Bugs;

use crate::files;

/// The command's options.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    map_file: PathBuf,
    view: ViewAt,
    at: u32,
    cycles: Option<u32>,
    live: Option<PathBuf>,
    viewer: u8,
    hidden: Vec<[i32; 4]>,
    bugs: Bugs,
    install: PathBuf,
    map: PathBuf,
    out: Option<PathBuf>,
    list: bool,
}

/// Parses comma-separated integers, exactly `N` of them (`what` names them in the message).
fn numbers<const N: usize>(value: &str, what: &str) -> Result<[i32; N], String> {
    let parsed: Result<Vec<i32>, _> = value.split(',').map(|v| v.trim().parse()).collect();
    parsed
        .ok()
        .and_then(|v| <[i32; N]>::try_from(v).ok())
        .ok_or_else(|| format!("render: {what} must be {N} integers separated by commas"))
}

impl Options {
    /// Parses `<map.h3m> --view <x>,<y>[,<z>] [--at <ms>] [--cycles <k>] [--live <live.toml>]
    /// [--viewer <0-7>] [--hidden <x0>,<y0>,<x1>,<y1>]... [--bugs fixed|original]
    /// [--install <dir>] [--map <dir>] [--out <file.png>] [--list]`.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let mut rest = args.iter();
        let mut map_file = None;
        let (mut view, mut at, mut cycles, mut viewer, mut hidden) = (None, 0, None, 0, Vec::new());
        let mut live = None;
        let (mut bugs, mut install, mut map, mut out, mut list) =
            (Bugs::Fixed, None, None, None, false);
        while let Some(arg) = rest.next() {
            if arg == "--list" {
                list = true;
                continue;
            }
            if !arg.starts_with("--") {
                if map_file.replace(PathBuf::from(arg)).is_some() {
                    return Err("render: one map only".into());
                }
                continue;
            }
            let value = rest
                .next()
                .ok_or_else(|| format!("render: `{arg}` needs a value"))?;
            match arg.as_str() {
                "--view" => {
                    let v = numbers::<3>(value, "--view")
                        .or_else(|_| numbers::<2>(value, "--view").map(|[x, y]| [x, y, 0]))?;
                    let z = u8::try_from(v[2]).map_err(|_| "render: --view's level is 0 or 1")?;
                    view = Some(ViewAt {
                        x: v[0],
                        y: v[1],
                        z,
                    });
                }
                "--at" => {
                    at = value
                        .parse()
                        .map_err(|_| "render: --at takes milliseconds")?
                }
                "--cycles" => {
                    cycles = Some(
                        value
                            .parse()
                            .map_err(|_| "render: --cycles takes a count of steps")?,
                    );
                }
                "--viewer" => {
                    viewer = value
                        .parse()
                        .ok()
                        .filter(|&c: &u8| c < 8)
                        .ok_or("render: --viewer takes a colour, 0-7")?;
                }
                "--live" => live = Some(PathBuf::from(value)),
                "--hidden" => hidden.push(numbers::<4>(value, "--hidden")?),
                "--bugs" => {
                    bugs = match value.as_str() {
                        "fixed" => Bugs::Fixed,
                        "original" => Bugs::Original,
                        _ => return Err("render: --bugs takes fixed or original".into()),
                    };
                }
                "--install" => install = Some(PathBuf::from(value)),
                "--map" => map = Some(PathBuf::from(value)),
                "--out" => out = Some(PathBuf::from(value)),
                other => return Err(format!("render: unknown option `{other}`")),
            }
        }
        Ok(Self {
            map_file: map_file.ok_or("render: which map?")?,
            view: view.ok_or("render: --view is required")?,
            at,
            cycles,
            live,
            viewer,
            hidden,
            bugs,
            install: install.unwrap_or_else(files::default_install),
            map: map.unwrap_or_else(files::default_map),
            out,
            list,
        })
    }

    /// The PNG's path: `--out`, else `clients/out/map/<stem>-v<x>_<y>_<z>[-<ms>ms].png`.
    fn out(&self) -> PathBuf {
        self.out.clone().unwrap_or_else(|| {
            let stem = self
                .map_file
                .file_stem()
                .map_or_else(|| "map".into(), |s| s.to_string_lossy().into_owned());
            let ViewAt { x, y, z } = self.view;
            let moment = if self.at > 0 {
                format!("-{}ms", self.at)
            } else {
                String::new()
            };
            files::default_map()
                .join("../out/map")
                .join(format!("{stem}-v{x}_{y}_{z}{moment}.png"))
        })
    }
}

pub fn run(options: &Options) -> ExitCode {
    match render(options) {
        Ok(path) => {
            println!("render: wrote {}", path.display());
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("render: {message}");
            ExitCode::FAILURE
        }
    }
}

fn render(options: &Options) -> Result<PathBuf, String> {
    let tables = files::load_tables(&options.map)?;
    let layout = files::load_layout(&options.map)?;
    let path = &options.map_file;
    let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    // A shipped map is gzip-compressed, the writer's (a situation's) is not.
    let raw = if bytes.starts_with(&[0x1F, 0x8B]) {
        h3_map::read(&bytes, &Limits::default())
    } else {
        h3_map::parse(&bytes, &Limits::default())
    }
    .map_err(|e| format!("{}: {e}", path.display()))?;
    let art = MapArt::from_raw(&raw).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut live = Live::at_start(&raw, &art, options.viewer);
    live.moment = Moment::at(options.at, tables.map.cycle_ms, tables.map.frame_ms);
    if let Some(cycles) = options.cycles {
        live.moment.cycles = cycles;
    }
    let archive_bytes = files::read_archives(&options.install)?;
    let archives = files::parse_archives(&archive_bytes)?;
    if let Some(path) = &options.live {
        read_live(path, &art, &mut live, &files::templates(&archives)?)?;
    }
    for &[x0, y0, x1, y1] in &options.hidden {
        for y in y0.min(y1)..=y0.max(y1) {
            for x in x0.min(x1)..=x0.max(x1) {
                live.explored.set(x, y, options.view.z, false);
            }
        }
    }
    let shapes = files::load_shapes(&archives, &objects::pictures(&art, &live))?;
    let scene = scene::scene(
        &art,
        &live,
        &tables,
        &layout,
        &shapes,
        options.view,
        options.bugs,
    )
    .map_err(|e| e.to_string())?;
    if options.list {
        print_list(&scene);
    }
    let mut batch = Batch::default();
    for name in batch::names(&scene) {
        let (_, data) = files::find(&archives, &name)?.ok_or(format!("{name}: in no archive"))?;
        batch.add(&name, &data).map_err(|e| e.to_string())?;
    }
    let screen = compose(&scene, &batch, layout.frame).map_err(|e| e.to_string())?;
    let out = options.out();
    if let Some(dir) = out.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    fs::write(&out, h3_render::png::encode(&screen))
        .map_err(|e| format!("{}: {e}", out.display()))?;
    Ok(out)
}

/// The original's live part as the harness read it (`h3map.py still`'s `live.toml`): `cycles`,
/// the palette step; `frames`, the objects' frame step the screen shows; `viewer`, the viewing
/// player's colour; `phases`, every object's animation phase in the map's order; `anchors`, every
/// object's anchor in that order, which must be the map's (so the phases are the objects' own);
/// `defs` and `classes`, every object's template in the original's memory, its DEF and its class
/// and subclass, in that order: one that differs from the map's is what a random object became
/// (E24 M1 T4b, design §2.7), its own picture the last of the game's `templates` of that class and
/// subclass (MCL-Q3). Each key is optional; other keys are left.
fn read_live(
    path: &PathBuf,
    art: &MapArt,
    live: &mut Live,
    templates: &[ObjectTemplate],
) -> Result<(), String> {
    let failed = |why: &str| format!("{}: {why}", path.display());
    let text = fs::read_to_string(path).map_err(|e| failed(&e.to_string()))?;
    let table: toml::Table = text
        .parse()
        .map_err(|e: toml::de::Error| failed(&e.to_string()))?;
    let number = |key: &str| -> Result<Option<u32>, String> {
        table.get(key).map_or(Ok(None), |v| {
            v.as_integer()
                .and_then(|n| u32::try_from(n).ok())
                .map(Some)
                .ok_or_else(|| failed(&format!("`{key}` is not a count")))
        })
    };
    if let Some(cycles) = number("cycles")? {
        live.moment.cycles = cycles;
    }
    if let Some(frames) = number("frames")? {
        live.moment.frames = frames;
    }
    if let Some(viewer) = number("viewer")? {
        live.viewer = u8::try_from(viewer)
            .ok()
            .filter(|&c| c < 8)
            .ok_or_else(|| failed("`viewer` is not a colour, 0-7"))?;
    }
    if let Some(anchors) = table.get("anchors") {
        let ours: Vec<toml::Value> = art
            .objects
            .iter()
            .map(|o| {
                let a = o.anchor;
                toml::Value::Array([a.x, a.y, a.z].map(|v| i64::from(v).into()).to_vec())
            })
            .collect();
        if anchors.as_array() != Some(&ours) {
            return Err(failed(
                "the original's objects are not the map's (`anchors`)",
            ));
        }
    }
    if let Some(phases) = table.get("phases") {
        let phases: Option<Vec<u8>> = phases.as_array().and_then(|a| {
            a.iter()
                .map(|v| v.as_integer().and_then(|n| u8::try_from(n).ok()))
                .collect()
        });
        let phases = phases.ok_or_else(|| failed("`phases` is not a list of bytes"))?;
        if phases.len() != live.objects.len() {
            return Err(failed(&format!(
                "{} phases for {} objects",
                phases.len(),
                live.objects.len()
            )));
        }
        for (object, phase) in live.objects.iter_mut().zip(phases) {
            object.phase = phase;
        }
    }
    if let (Some(defs), Some(classes)) = (table.get("defs"), table.get("classes")) {
        let defs: Option<Vec<&str>> = defs
            .as_array()
            .and_then(|a| a.iter().map(toml::Value::as_str).collect());
        let classes: Option<Vec<[u32; 2]>> = classes.as_array().and_then(|a| {
            a.iter()
                .map(|v| {
                    let pair: Option<Vec<u32>> = v.as_array()?.iter().map(count).collect();
                    <[u32; 2]>::try_from(pair?).ok()
                })
                .collect()
        });
        let (defs, classes) = defs
            .zip(classes)
            .filter(|(d, c)| d.len() == art.objects.len() && c.len() == art.objects.len())
            .ok_or_else(|| failed("`defs` and `classes` are not one template per object"))?;
        for (index, (def, [class, subclass])) in defs.into_iter().zip(classes).enumerate() {
            let Some(template) = objects::template(art, index) else {
                continue;
            };
            let same = template.def.eq_ignore_ascii_case(def)
                && (template.class, template.subclass) == (class, subclass);
            if !same {
                let own = templates
                    .iter()
                    .rev()
                    .find(|t| (t.class, t.subclass) == (class, subclass));
                live.objects[index].drawn = Some(Drawn {
                    def: def.to_string(),
                    own: own.map(|t| t.def.clone()),
                    class,
                    subclass,
                });
            }
        }
    }
    Ok(())
}

/// A TOML integer as a `u32`.
fn count(value: &toml::Value) -> Option<u32> {
    value.as_integer().and_then(|n| u32::try_from(n).ok())
}

/// The draw list, one line per draw, and the palette cycles.
fn print_list(scene: &scene::Scene) {
    for draw in &scene.draws {
        match draw {
            Draw::Frame {
                picture,
                players,
                viewer,
            } => println!("frame {picture} in {players} colour {viewer}"),
            Draw::Piece {
                pass,
                def,
                frame,
                x,
                y,
                mirror_x,
                mirror_y,
                clip,
            } => {
                let mirrors = match (mirror_x, mirror_y) {
                    (false, false) => "",
                    (true, false) => " mirrored",
                    (false, true) => " flipped",
                    (true, true) => " mirrored flipped",
                };
                println!(
                    "{pass:?} {def} {frame} at {x},{y}{mirrors} clip {},{}-{},{}",
                    clip[0], clip[1], clip[2], clip[3]
                );
            }
            Draw::Gem { def, frame, x, y } => println!("gem {def} {frame} at {x},{y}"),
        }
    }
    for cycle in &scene.cycles {
        println!("cycle {} {:?} by {}", cycle.def, cycle.ranges, cycle.steps);
    }
}
