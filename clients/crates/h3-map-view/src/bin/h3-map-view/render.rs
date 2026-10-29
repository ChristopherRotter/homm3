//! `render`: a still of the adventure screen (E24 M1 T3b1): a map's static art from its `.h3m`,
//! the live part at the map's start (every tile explored but a chosen rectangle, the moment from
//! a time), the objects' shapes and the scene at a view, its assets from the installation copy's
//! archives, composed and written as a PNG. The comparisons with the original (T3b2 to T5b2)
//! give the live part read from its memory (`--live`: the palette step, the frame step, the
//! viewer, the objects' phases and what the random ones became, the heroes and boats, the owners,
//! the explored tiles and the path's arrows).

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use h3_map::Limits;
use h3_map::model::Pos;
use h3_map::writer::ObjectTemplate;
use h3_map_view::art::{MapArt, TemplateArt};
use h3_map_view::batch::{self, Batch};
use h3_map_view::compose::compose;
use h3_map_view::live::{
    Arrow, Boat, Buttons, Drawn, Explored, Facing, Kingdom, Live, LiveBoat, LiveHero, Moment,
    TOWN_CLASS,
};
use h3_map_view::objects;
use h3_map_view::scene::{self, Draw, ViewAt};
use h3_map_view::tables::PATH_FRAMES;
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
    let mut art = MapArt::from_raw(&raw).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut live = Live::at_start(&raw, &art, options.viewer, &tables.map.towns);
    live.moment = Moment::at(options.at, tables.map.cycle_ms, tables.map.frame_ms);
    if let Some(cycles) = options.cycles {
        live.moment.cycles = cycles;
        live.moment.river_cycles = cycles;
    }
    let archive_bytes = files::read_archives(&options.install)?;
    let archives = files::parse_archives(&archive_bytes)?;
    let mut pointer = None;
    if let Some(path) = &options.live {
        pointer = read_live(
            path,
            options.view,
            &mut art,
            &mut live,
            &files::templates(&archives)?,
        )?;
    }
    hide(&art, &mut live, options.view.z, &options.hidden);
    let mut shapes = files::load_shapes(&archives, &objects::pictures(&art, &live))?;
    let heroes: Vec<String> = tables
        .map
        .heroes
        .defs()
        .into_iter()
        .map(|(d, _)| d)
        .collect();
    files::load_frames(&archives, &heroes, &mut shapes)?;
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
    let mut screen = compose(&scene, &batch, layout.frame).map_err(|e| e.to_string())?;
    if let Some([x, y]) = pointer {
        // The parked mouse's cursor (E24 M3 T2a): the arrow, as the mouse manager draws it over
        // all (0x50D8B0) at the point less its hot spot.
        let art = &layout.cursor;
        let name = art.def.to_ascii_uppercase();
        let (_, data) = files::find(&archives, &name)?.ok_or(format!("{name}: in no archive"))?;
        batch.add(&name, &data).map_err(|e| e.to_string())?;
        let [hx, hy] = usize::try_from(art.arrow)
            .ok()
            .and_then(|f| art.hot.get(f).copied())
            .unwrap_or_default();
        let hot = (
            u32::try_from(hx).unwrap_or(0),
            u32::try_from(hy).unwrap_or(0),
        );
        let cursor = batch
            .sprite(&name)
            .and_then(|s| h3_ui::cursor::Cursor::read(s, art.arrow, hot, screen.format()))
            .ok_or_else(|| format!("{}: no frame {}", art.def, art.arrow))?;
        cursor.draw(&mut screen, x, y);
    }
    let out = options.out();
    if let Some(dir) = out.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    fs::write(&out, h3_render::png::encode(&screen))
        .map_err(|e| format!("{}: {e}", out.display()))?;
    Ok(out)
}

/// The tiles of the rectangles `hidden` (`[x0, y0, x1, y1]`, corners included) on level `z` left
/// unexplored; each rectangle clamped to the map (E24 M1 T6b, the reviewer's R2-13: a huge one
/// took long).
pub fn hide(art: &MapArt, live: &mut Live, z: u8, hidden: &[[i32; 4]]) {
    let last = i32::try_from(art.size)
        .unwrap_or(i32::MAX)
        .saturating_sub(1);
    let on_map = |v: i32| v.clamp(0, last);
    for &[x0, y0, x1, y1] in hidden {
        for y in on_map(y0.min(y1))..=on_map(y0.max(y1)) {
            for x in on_map(x0.min(x1))..=on_map(x0.max(x1)) {
                live.explored.set(x, y, z, false);
            }
        }
    }
}

/// The artifacts' class.
const ARTIFACT_CLASS: u32 = 5;

/// The random dwellings' classes (216 any, 217 of a level, 218 of a faction).
const RANDOM_DWELLINGS: [u32; 3] = [216, 217, 218];

/// The original's live part as the harness read it (`h3map.py still`'s `live.toml`): `cycles`,
/// the palette step, and `river_cycles` the rivers' when it differs (E24 M1 T6a: one fewer when
/// the original's load ran a tick between the terrains' DEFs and the rivers', design §1.6;
/// otherwise `cycles`); `frames`, the objects' frame step the screen shows; `viewer`, the viewing
/// player's colour; `phases`, every object's animation phase in the map's order; `anchors`, every
/// object's anchor in that order, which must be the map's (so the phases are the objects' own; a
/// random object's may differ, T5);
/// `defs` and `classes`, every object's template in the original's memory, its DEF and its class
/// and subclass, in that order: one that differs from the map's is what a random object became
/// (E24 M1 T4b, design §2.7), its own picture the last of the game's `templates` of that class and
/// subclass (MCL-Q3); `heroes` and `boats` (E24 M1 T5a), the heroes and empty boats the view
/// draws, each `[x, y, level, class, facing, owner, boat type, boat colour]` (the boat's -1 -1 on
/// land) and `[x, y, level, type, facing]`; `explored` (E24 M1 T5b2), the viewer's explored
/// tiles, a string per row of every level in turn (level by level, rows top to bottom), `1` a tile
/// explored and `0` not; `path` (T5b2), the path shown, each arrow `[x, y, level, arrow]`; `view`,
/// the view's top-left and level the original showed, which must be `--view` (E24 M1 T6b, the
/// reviewer's R2-3); `kingdom` (E24 M3 T2a, design §4.1), what the panels show of the viewer's
/// kingdom: `{ heroes = [[asleep, movable], ...], towns = <n>, selected = [<0 none, 1 a hero, 2 a
/// town>, <its place in its list>], tops = [<heroes>, <towns>], quests = <0 or 1> }`; `buttons`
/// (T2a), move hero's and next hero's states as the original last set them, `[move, next]`, 1
/// enabled; `pointer`
/// (T2a), where the parked mouse is, returned. Each key is optional (`defs` and `classes`
/// together); other keys are left. The file's `cycles`, `frames` and `viewer` override
/// `--cycles`, `--at` and `--viewer`: the comparisons draw the original's.
pub fn read_live(
    path: &PathBuf,
    view: ViewAt,
    art: &mut MapArt,
    live: &mut Live,
    templates: &[ObjectTemplate],
) -> Result<Option<[i32; 2]>, String> {
    let mut resolved = vec![false; art.objects.len()];
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
    if let Some(theirs) = table.get("view") {
        let ours = [i64::from(view.x), i64::from(view.y), i64::from(view.z)];
        let theirs: Option<Vec<i64>> = theirs
            .as_array()
            .and_then(|a| a.iter().map(toml::Value::as_integer).collect());
        if theirs.as_deref() != Some(&ours[..]) {
            return Err(failed(&format!(
                "the original showed the view at {theirs:?}, not --view {ours:?}"
            )));
        }
    }
    if let Some(cycles) = number("cycles")? {
        live.moment.cycles = cycles;
        live.moment.river_cycles = cycles;
    }
    if let Some(cycles) = number("river_cycles")? {
        live.moment.river_cycles = cycles;
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
    let templates_read = match (table.get("defs"), table.get("classes")) {
        (Some(defs), Some(classes)) => Some((defs, classes)),
        (None, None) => None,
        _ => return Err(failed("`defs` and `classes` go together")),
    };
    if let Some((defs, classes)) = templates_read {
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
            let Some(template) = objects::template(art, index).cloned() else {
                continue;
            };
            // A town's picture by its buildings: the start's (`Live::at_start`) must be the
            // original's (E24 M1 T5b1, design §2.9).
            if template.class == TOWN_CLASS {
                let ours = live.objects[index]
                    .drawn
                    .as_ref()
                    .map_or(template.def.as_str(), |d| d.def.as_str());
                if !ours.eq_ignore_ascii_case(def) {
                    return Err(failed(&format!(
                        "object {index}: the original drew the town as {def}, its buildings give {ours}"
                    )));
                }
            }
            let same = template.def.eq_ignore_ascii_case(def)
                && (template.class, template.subclass) == (class, subclass);
            let own = templates
                .iter()
                .rev()
                .find(|t| (t.class, t.subclass) == (class, subclass) && !same);
            if own.is_some() && RANDOM_DWELLINGS.contains(&template.class) {
                // A random dwelling became a dwelling when the map loaded, before its objects
                // were placed (0x502B60, design §2.9): its template is the game's (the last of its
                // class and subclass, 0x505F20), the anchor the original's (`anchors`).
                let dwelling = templates
                    .iter()
                    .rev()
                    .find(|o| {
                        (o.class, o.subclass) == (class, subclass)
                            && o.def.eq_ignore_ascii_case(def)
                    })
                    .ok_or_else(|| {
                        failed(&format!(
                            "object {index}: no template of the game's for the {def} the original \
                             drew"
                        ))
                    })?;
                art.templates.push(TemplateArt {
                    def: dwelling.def.clone(),
                    passable: dwelling.passable,
                    visitable: dwelling.visitable,
                    class,
                    subclass,
                    priority: dwelling.priority,
                });
                art.objects[index].template = art.templates.len() - 1;
                resolved[index] = true;
                continue;
            }
            if !same {
                // Only an artifact's copy names a wrong picture (MCL-Q3); a town's is named by its
                // buildings, and a class the game loads as another (object_class_values
                // 0x63A6E4: the vertical garrison 219 as 33, the decorations 165-205) keeps its
                // DEF.
                let own = own.filter(|_| class == ARTIFACT_CLASS);
                live.objects[index].drawn = Some(Drawn {
                    def: def.to_string(),
                    own: own.map(|t| t.def.clone()),
                    class,
                    subclass,
                });
            }
        }
    }
    if let Some(owners) = table.get("owners") {
        // Every object's owner as the view passes find it (`-1` none; E24 M1 T5b1).
        let owners: Option<Vec<i64>> = owners
            .as_array()
            .and_then(|a| a.iter().map(toml::Value::as_integer).collect());
        let owners = owners
            .filter(|o| o.len() == live.objects.len())
            .ok_or_else(|| failed("`owners` is not one owner per object"))?;
        // -1 none, 0-7 a colour, anything else refused (E24 M1 T6b, the reviewer's R2-4).
        for (index, (object, owner)) in live.objects.iter_mut().zip(owners).enumerate() {
            object.owner = match owner {
                -1 => None,
                _ => Some(byte(owner).filter(|&c| c < 8).ok_or_else(|| {
                    failed(&format!(
                        "`owners`: object {index}'s {owner} is not -1 or 0-7"
                    ))
                })?),
            };
        }
    }
    if let Some(heroes) = table.get("heroes") {
        live.heroes = rows::<8>(heroes)
            .and_then(|rows| rows.into_iter().map(hero).collect())
            .ok_or_else(|| {
                failed(
                    "`heroes` is not a list of [x, y, level, class, facing, owner, boat, colour]",
                )
            })?;
    }
    if let Some(boats) = table.get("boats") {
        live.boats = rows::<5>(boats)
            .and_then(|rows| rows.into_iter().map(boat).collect())
            .ok_or_else(|| failed("`boats` is not a list of [x, y, level, type, facing]"))?;
    }
    if let Some(rows) = table.get("explored") {
        live.explored = explored(rows, art.size, art.levels).ok_or_else(|| {
            failed(&format!(
                "`explored` is not {} rows of {} tiles of 0 and 1",
                art.size * art.levels,
                art.size
            ))
        })?;
    }
    if let Some(path) = table.get("path") {
        live.path = rows::<4>(path)
            .and_then(|rows| rows.into_iter().map(arrow).collect())
            .ok_or_else(|| failed("`path` is not a list of [x, y, level, arrow 1-50]"))?;
    }
    if let Some(anchors) = table.get("anchors") {
        // A random dwelling stands where its dwelling's template put it when the map loaded
        // (0x502B60, design §2.9: its anchor moved right past the bottom row's passable cells
        // before its entrance, on its row and level: E24 M1 T6b, the reviewer's R2-5); every
        // other object must be where the map has it.
        let theirs: Option<Vec<[i64; 3]>> =
            rows::<3>(anchors).filter(|a| a.len() == art.objects.len());
        let theirs = theirs.ok_or_else(|| failed("`anchors` is not one anchor per object"))?;
        for (index, (object, [x, y, z])) in art.objects.iter_mut().zip(theirs).enumerate() {
            let a = object.anchor;
            if [a.x, a.y, a.z].map(i64::from) == [x, y, z] {
                continue;
            }
            let at = (byte(x), byte(y), byte(z));
            match at {
                (Some(x), Some(y), Some(z))
                    if resolved[index] && x > a.x && (y, z) == (a.y, a.z) =>
                {
                    object.anchor = Pos { x, y, z };
                }
                _ => {
                    return Err(failed(&format!(
                        "the original's objects are not the map's (`anchors`: object {index})"
                    )));
                }
            }
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
    if let Some(value) = table.get("buttons") {
        // Move hero's and next hero's states as the original last set them (E24 M3 T2a).
        let flags: Option<Vec<bool>> = value.as_array().and_then(|a| {
            a.iter()
                .map(|v| match v.as_integer() {
                    Some(0) => Some(false),
                    Some(1) => Some(true),
                    _ => None,
                })
                .collect()
        });
        live.buttons = match flags.as_deref() {
            Some(&[move_hero, next_hero]) => Some(Buttons {
                move_hero,
                next_hero,
            }),
            _ => return Err(failed("`buttons` is not [move, next], each 0 or 1")),
        };
    }
    if let Some(value) = table.get("kingdom") {
        live.kingdom = Kingdom::read(value).ok_or_else(|| {
            failed(
                "`kingdom` is not { heroes = [[asleep, movable], ...], towns, selected = [kind, \
                 index], tops = [heroes, towns], quests }",
            )
        })?;
    }
    table.get("pointer").map_or(Ok(None), |value| {
        let point: Option<Vec<i32>> = value.as_array().and_then(|a| {
            a.iter()
                .map(|v| v.as_integer().and_then(|n| i32::try_from(n).ok()))
                .collect()
        });
        point
            .and_then(|p| <[i32; 2]>::try_from(p).ok())
            .map(Some)
            .ok_or_else(|| failed("`pointer` is not [x, y]"))
    })
}

/// A TOML list of lists of `N` integers.
fn rows<const N: usize>(value: &toml::Value) -> Option<Vec<[i64; N]>> {
    value
        .as_array()?
        .iter()
        .map(|row| {
            let row: Option<Vec<i64>> = row
                .as_array()?
                .iter()
                .map(toml::Value::as_integer)
                .collect();
            <[i64; N]>::try_from(row?).ok()
        })
        .collect()
}

/// A byte of a `live.toml` row.
fn byte(n: i64) -> Option<u8> {
    u8::try_from(n).ok()
}

/// A hero of `live.toml`'s `heroes` (its class below 18, a colour 0-7, a boat's type 0-2 or -1).
fn hero([x, y, z, class, facing, owner, kind, colour]: [i64; 8]) -> Option<LiveHero> {
    let boat = match (kind, colour) {
        (-1, -1) => None,
        _ => Some(Boat {
            kind: byte(kind).filter(|&k| k < 3)?,
            colour: byte(colour).filter(|&c| c < 8)?,
        }),
    };
    Some(LiveHero {
        at: Pos {
            x: byte(x)?,
            y: byte(y)?,
            z: byte(z)?,
        },
        class: byte(class).filter(|&c| c < 18)?,
        owner: byte(owner).filter(|&c| c < 8)?,
        facing: Facing::from_number(byte(facing)?)?,
        boat,
    })
}

/// `live.toml`'s `explored`: `levels` × `size` rows of `size` tiles each.
fn explored(value: &toml::Value, size: usize, levels: usize) -> Option<Explored> {
    let rows: Vec<&str> = value
        .as_array()?
        .iter()
        .map(toml::Value::as_str)
        .collect::<Option<_>>()?;
    if rows.len() != size * levels || rows.iter().any(|r| r.len() != size) {
        return None;
    }
    let mut explored = Explored::all(size, levels, false);
    for (index, row) in rows.into_iter().enumerate() {
        let (z, y) = (
            u8::try_from(index / size).ok()?,
            i32::try_from(index % size).ok()?,
        );
        for (x, tile) in row.bytes().enumerate() {
            let seen = match tile {
                b'0' => false,
                b'1' => true,
                _ => return None,
            };
            explored.set(i32::try_from(x).ok()?, y, z, seen);
        }
    }
    Some(explored)
}

/// An arrow of `live.toml`'s `path` (1 to 50: the path DEF's frames).
fn arrow([x, y, z, arrow]: [i64; 4]) -> Option<Arrow> {
    Some(Arrow {
        at: Pos {
            x: byte(x)?,
            y: byte(y)?,
            z: byte(z)?,
        },
        arrow: u16::try_from(arrow)
            .ok()
            .filter(|&a| a > 0 && usize::from(a) <= PATH_FRAMES)?,
    })
}

/// A boat of `live.toml`'s `boats`.
fn boat([x, y, z, kind, facing]: [i64; 5]) -> Option<LiveBoat> {
    Some(LiveBoat {
        at: Pos {
            x: byte(x)?,
            y: byte(y)?,
            z: byte(z)?,
        },
        kind: byte(kind).filter(|&k| k < 3)?,
        facing: Facing::from_number(byte(facing)?)?,
    })
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
                group,
                frame,
                x,
                y,
                mirror_x,
                mirror_y,
                clip,
                flag,
            } => {
                let mirrors = match (mirror_x, mirror_y) {
                    (false, false) => "",
                    (true, false) => " mirrored",
                    (false, true) => " flipped",
                    (true, true) => " mirrored flipped",
                };
                let flag = flag.map_or_else(String::new, |e| format!(" flag {e}"));
                println!(
                    "{pass:?} {def} {group}/{frame} at {x},{y}{mirrors} clip {},{}-{},{}{flag}",
                    clip[0], clip[1], clip[2], clip[3]
                );
            }
            Draw::Gem { def, frame, x, y } => println!("gem {def} {frame} at {x},{y}"),
            Draw::Dialog(item) => println!("dialog {item:?}"),
            Draw::Minimap(tiles) => {
                let look = &tiles.look;
                println!(
                    "minimap {} tiles at {},{} size {}x{}",
                    tiles.colours.len(),
                    look.at[0],
                    look.at[1],
                    look.size[0],
                    look.size[1]
                );
            }
            Draw::Radar {
                def,
                frame,
                at,
                cut,
                area,
            } => println!(
                "radar {def} {frame} at {},{} cut {},{} in {},{}-{},{}",
                at[0], at[1], cut[0], cut[1], area[0], area[1], area[2], area[3]
            ),
        }
    }
    for cycle in &scene.cycles {
        println!("cycle {} {:?} by {}", cycle.def, cycle.ranges, cycle.steps);
    }
}

#[cfg(test)]
mod tests;
