//! `h3-view render` (E17 M1 T7): a battle still. A record's first snapshot (or a scenario file's,
//! through [`crate::scenario`]) and the asset tables give the draw list (`h3_view::scene`); its
//! assets are loaded as one checkpoint batch (ADR-0020 point 7: the four archives read whole until
//! CLT-Q2's table-only reader lands, then the batch's entries by name); the compositor draws the
//! frame; the frame is written as a PNG.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use h3_formats::lod::LodArchive;
use h3_protocol::wire::Seat;
use h3_render::Bugs;
use h3_view::anim::{self, Speed};
use h3_view::batch::{self, Batch};
use h3_view::compose::compose;
use h3_view::scene::Panels;
use h3_view::siege::Walls;
use h3_view::{layout, record, scene};

use crate::gate::{find_ci, timed};
use crate::{assets, motion};

/// The archives, the base ones first: a DEF is searched in the sprite archives, anything else in
/// the bitmap ones (`docs/epics/E17-design.md` §3.1).
const SPRITES: [&str; 2] = ["H3sprite.lod", "H3ab_spr.lod"];
const BITMAPS: [&str; 2] = ["H3bitmap.lod", "H3ab_bmp.lod"];

/// The command's options.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    input: PathBuf,
    seed: u64,
    packs: PathBuf,
    install: PathBuf,
    out: Option<PathBuf>,
    list: bool,
    at: Option<u32>,
    speed: u8,
    frames: Option<PathBuf>,
    /// Whose record a scenario plays into (E17 M4 T1b: a side's shows its hidden objects).
    seat: Seat,
    /// A siege's wall states in place of the record's (E17 M4 T1b, *provisional*).
    walls: Option<Walls>,
}

impl Options {
    /// Parses `<scenario.toml | record.jsonl> [--seed <n>] [--packs <dir>] [--install <dir>]
    /// [--out <file.png>] [--list] [--at <ms> | --frames <dir>] [--speed <0-2>] [--seat
    /// <attacker | defender>] [--walls <the 18 parts' picture states>,<the gate's>]`; the
    /// installation defaults as the gate's, the record a spectator's, the walls the record's.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let env = std::env::var_os("H3_INSTALL").map(PathBuf::from);
        Self::parse_with(args, env)
    }

    fn parse_with(args: &[String], env: Option<PathBuf>) -> Result<Self, String> {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let (mut input, mut seed, mut packs, mut install, mut out, mut list) =
            (None, 1, None, None, None, false);
        let (mut at, mut speed, mut frames) = (None, 0, None);
        let (mut seat, mut walls) = (Seat::Spectator, None);
        let mut rest = args.iter();
        while let Some(arg) = rest.next() {
            let mut value = || {
                rest.next()
                    .cloned()
                    .ok_or_else(|| format!("render: `{arg}` needs a value"))
            };
            match arg.as_str() {
                "--seed" => {
                    let v = value()?;
                    seed = v.parse().map_err(|_| format!("render: bad seed `{v}`"))?;
                }
                "--packs" => packs = Some(PathBuf::from(value()?)),
                "--install" => install = Some(PathBuf::from(value()?)),
                "--out" => out = Some(PathBuf::from(value()?)),
                "--list" => list = true,
                "--at" => {
                    let v = value()?;
                    at = Some(v.parse().map_err(|_| format!("render: bad time `{v}`"))?);
                }
                "--frames" => frames = Some(PathBuf::from(value()?)),
                "--walls" => walls = Some(parse_walls(&value()?)?),
                "--seat" => {
                    seat = match value()?.as_str() {
                        "attacker" => Seat::Attacker,
                        "defender" => Seat::Defender,
                        other => return Err(format!("render: bad seat `{other}`")),
                    };
                }
                "--speed" => {
                    let v = value()?;
                    speed = v
                        .parse()
                        .ok()
                        .filter(|s| *s <= 2)
                        .ok_or(format!("render: bad speed `{v}`"))?;
                }
                flag if flag.starts_with("--") => {
                    return Err(format!("render: unknown option `{flag}`"));
                }
                path if input.is_none() => input = Some(PathBuf::from(path)),
                extra => return Err(format!("render: unexpected `{extra}`")),
            }
        }
        if at.is_some() && frames.is_some() {
            return Err("render: --at or --frames, not both".into());
        }
        let options = Self {
            input: input.ok_or("render: which scenario or record?")?,
            seed,
            packs: packs.unwrap_or_else(|| manifest.join("../../../packs")),
            install: install
                .or(env)
                .unwrap_or_else(|| manifest.join("../../../original-data/install")),
            out,
            list,
            at,
            speed,
            frames,
            seat,
            walls,
        };
        // A record keeps the seat it was recorded from (E17 M4 T3a, R2-4).
        if options.seat != Seat::Spectator && !options.scenario() {
            return Err("render: --seat runs a scenario; a record keeps its own seat".into());
        }
        Ok(options)
    }

    /// Whether the input is a scenario file (else a record).
    fn scenario(&self) -> bool {
        self.input
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("toml"))
    }

    /// The PNG's path: `--out`, or `clients/out/render/<stem>[-s<seed>].png`.
    fn png(&self) -> PathBuf {
        self.out.clone().unwrap_or_else(|| {
            let stem = self
                .input
                .file_stem()
                .map_or_else(|| "still".into(), |s| s.to_string_lossy());
            let at = self.at.map_or_else(String::new, |t| format!("-{t}ms"));
            let name = if self.scenario() {
                format!("{stem}-s{}{at}.png", self.seed)
            } else {
                format!("{stem}{at}.png")
            };
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../out/render")
                .join(name)
        })
    }
}

pub fn run(options: &Options) -> ExitCode {
    match render(options) {
        Ok(lines) => {
            for line in lines {
                println!("{line}");
            }
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("render: {message}");
            ExitCode::FAILURE
        }
    }
}

pub fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    fs::write(path, bytes).map_err(|e| format!("{}: {e}", path.display()))
}

/// The archives of `install`, read whole (CLT-Q2), each with its name.
pub fn read_archives(install: &Path) -> Result<Vec<(&'static str, Vec<u8>)>, String> {
    let data = find_ci(install, "Data").map_err(|e| format!("{}: {e}", install.display()))?;
    SPRITES
        .iter()
        .chain(&BITMAPS)
        .map(|&name| {
            let path =
                find_ci(&data, name).map_err(|e| format!("{}/{name}: {e}", data.display()))?;
            let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            Ok((name, bytes))
        })
        .collect()
}

/// The entry `name` from the first archive of its search list that holds it.
pub fn lookup(archives: &[(&str, LodArchive<'_>)], name: &str) -> Result<Vec<u8>, String> {
    let list: &[&str] = if name.to_ascii_uppercase().ends_with(".DEF") {
        &SPRITES
    } else {
        &BITMAPS
    };
    for (archive_name, archive) in archives.iter().filter(|(n, _)| list.contains(n)) {
        if let Some(bytes) = archive
            .read_by_name(name)
            .map_err(|e| format!("{archive_name}:{name}: {e:?}"))?
        {
            return Ok(bytes);
        }
    }
    Err(format!("{name}: in none of {}", list.join(", ")))
}

/// A siege's wall states as `--walls` gives them (E17 M4 T1b): the 18 parts' picture states,
/// then the gate's, comma-separated.
fn parse_walls(text: &str) -> Result<Walls, String> {
    let bad = || format!("render: bad walls `{text}` (the 18 parts' states, then the gate's)");
    let numbers = text
        .split(',')
        .map(|n| n.trim().parse::<u8>().map_err(|_| bad()))
        .collect::<Result<Vec<_>, _>>()?;
    let (gate, parts) = numbers.split_last().ok_or_else(bad)?;
    Ok(Walls {
        parts: parts.try_into().map_err(|_| bad())?,
        gate: *gate,
    })
}

/// Renders the still; the lines to print: the PNG, the record written (from a scenario), the
/// counts and the times, and with `--list` the draw list.
fn render(options: &Options) -> Result<Vec<String>, String> {
    let mut lines = Vec::new();
    let text = if options.scenario() {
        let text =
            crate::scenario::record_of(&options.input, &options.packs, options.seed, options.seat)?;
        let path = options.png().with_extension("jsonl");
        write(&path, text.as_bytes())?;
        lines.push(format!("record: {}", path.display()));
        text
    } else {
        fs::read_to_string(&options.input)
            .map_err(|e| format!("{}: {e}", options.input.display()))?
    };
    let record = record::parse(&text).map_err(|e| e.to_string())?;
    let observation = record
        .first_snapshot()
        .ok_or("the record holds no snapshot")?;
    let manifest = &record.header.manifest;
    // The tables of the game's topmost pack, as the ruleset names its packs.
    let top = manifest.packs.last().map_or("", |p| p.id.as_str());
    let tables = assets::load(&assets::default_dir(), top)?;
    let layout_path = assets::default_dir().join("../layouts/battle.toml");
    let layout = layout::parse_battle(
        &fs::read_to_string(&layout_path).map_err(|e| format!("{}: {e}", layout_path.display()))?,
    )
    .map_err(|e| format!("{}: {e}", layout_path.display()))?;
    // The checkpoint: the archives (whole, CLT-Q2), then the batch.
    let (bytes, read) = timed(|| read_archives(&options.install));
    let bytes = bytes?;
    // The executable's own strings: read only when a line needs them (none in an installation
    // without it, as the synthetic one of the tests).
    let exe = crate::text::executable(&options.install).unwrap_or_default();
    let archives = bytes
        .iter()
        .map(|(name, b)| {
            LodArchive::parse(b)
                .map(|a| (*name, a))
                .map_err(|e| format!("{name}: {e:?}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if let Some(dir) = &options.frames {
        // The capture set's frames by time (E17 M3 T1b-2, `h3drive.py set`): the record's
        // commands at the original's timing without fidgets of the viewer's own, each frame with
        // the fidgets the original showed then, as the frame check draws a sequence's.
        let looks = motion::looks(&record, &tables, &archives)?;
        let magic = motion::magic(&record, &tables, &archives, false)?;
        let siege = motion::siege(&record, &tables, &archives)?;
        let casts = motion::casts(&record, &tables, &archives, &options.packs, false)?;
        let stage = motion::stage(&record, &tables, &layout, (&magic, &casts, siege.as_ref()));
        let commands = anim::commands(&record.messages);
        let timeline = anim::timeline(
            observation,
            &commands,
            &looks,
            stage,
            Speed::new(options.speed),
        )
        .map_err(|e| e.to_string())?
        .with_heroes(motion::heroes(&record, &tables, &archives, None)?);
        let panels = crate::log::panels(
            &record,
            (&timeline.log, &timeline.controllers),
            &tables,
            &layout,
            (&archives, &exe),
            Bugs::Original,
        )?;
        // The acting stack's legal commands at the opening, for the mouse (E17 M3 T6c-1): the
        // engine's, from the scenario; a record names none.
        let legal = if options.scenario() {
            crate::scenario::opening_legal(&options.input, &options.packs, options.seed)?
        } else {
            Vec::new()
        };
        // A situation's actions (E17 M5 T3a: the spell book's stills) played at its opening,
        // after the tactics phase's message it opens with (E17 M5 T4c2a).
        let crate::sequences::Situation {
            mouse,
            actions,
            army_info,
            shades,
        } = crate::sequences::situation_inputs(dir)?;
        let told = h3_view::tactics::told(observation);
        // The status line at Creature Info's levels 1 and 2 words a strike's or a shot's damage
        // (E17 M6 T4b2b2, the epic's review item 42): the engine's previews of the opening's
        // attacks, as the window's (`scenario::previews`); a record names none.
        let scenario = (options.scenario()).then_some((
            options.input.as_path(),
            options.packs.as_path(),
            options.seed,
        ));
        let still_legal = crate::pointer::still_legal(
            &record.header.manifest,
            &tables,
            observation,
            (&legal, scenario),
        )?;
        // At Creature Info's level 1 or 2 the mouse at rest shows a window too (E17 M6 T4b2b1), and
        // with the cursor shadow on shades its hex, the grid and the movement shadow drawn with
        // their options (E17 M6 T4c).
        let plain = shades == h3_view::shades::Options::default();
        let book = if actions.is_empty() && !told && army_info == 0 && plain {
            None
        } else {
            let ruleset = crate::scenario::ruleset(&options.packs, manifest)?;
            let towns = crate::scenario::towns(&options.packs, manifest)?;
            let catalogue = crate::log::catalogue(&tables, &archives, &exe)?;
            let hero = || {
                crate::spellbook::takes(observation)
                    .then(|| {
                        crate::spellbook::hero(
                            observation,
                            (manifest, &ruleset),
                            (&tables, &catalogue),
                            &layout.book,
                        )
                    })
                    .flatten()
            };
            // The message boxes (E17 M5 T4a): laid out in the layout's fonts, in the viewing
            // player's colours; the side's ways to leave; its hero's spells' refusals (E17 M6
            // T5a).
            let writer =
                crate::log::Writer::new(&tables, &layout, (&archives, &exe), Bugs::Original)?;
            let viewing = viewing_player(observation);
            let tell = |text: &str, style, picture: Option<&h3_view::message::Picture>| {
                writer.message(&layout, text, style, picture, viewing)
            };
            let leave =
                crate::boxes::Leave::of(observation, &legal, &manifest.keys.hero_types, &catalogue);
            let spells: Vec<(u16, u32)> = observation
                .own_hero
                .iter()
                .flat_map(|h| &h.spell_book)
                .filter_map(|b| {
                    let entry = motion::spell_entry_in(&manifest.keys.spells, &tables, b.spell)?;
                    Some((b.spell, u32::from(entry.original)))
                })
                .collect();
            let refusals = if options.scenario() {
                crate::scenario::opening_refusals(
                    &options.input,
                    &options.packs,
                    options.seed,
                    &spells,
                    (manifest, &tables, &catalogue),
                )?
            } else {
                Vec::new()
            };
            let refusal = |spell: u16| {
                (refusals.iter())
                    .find(|(handle, _)| *handle == spell)
                    .map(|(_, text)| text.clone())
            };
            let def = layout.message.spell_pictures.clone();
            let size = lookup(&archives, &def)
                .ok()
                .and_then(|b| h3_assets::def::parse_def(&b).ok())
                .map_or([0, 0], |d| {
                    [d.width, d.height].map(|v| i32::try_from(v).unwrap_or(i32::MAX))
                });
            // The result window a box's OK ends the battle with (E17 M5 T4b1): the engine's
            // outcome of the opening's command.
            let result = |command| {
                if !options.scenario() {
                    return None;
                }
                let (ruleset, outcome) = crate::scenario::outcome(
                    &options.input,
                    &options.packs,
                    options.seed,
                    &[command],
                )
                .ok()?;
                let sources = crate::results::Sources {
                    manifest,
                    tables: &tables,
                    ruleset: Some(&ruleset),
                    catalogue: &catalogue,
                    layout: &layout,
                    // The comparisons draw the original's (CLT-Q35).
                    bugs: Bugs::Original,
                };
                let outcome = outcome?;
                let (window, tune) =
                    crate::results::window(&sources, observation, &outcome, viewing);
                // Eagle Eye's message after it (E17 M6 T5c), laid out as the window.
                let told = crate::results::eagle_eye(&sources, observation, &outcome, (&def, size));
                let after = told
                    .iter()
                    .filter_map(|t| writer.pictures(&layout, &t.text, &t.pictures, viewing))
                    .collect();
                Some((window, tune, after))
            };
            // A press on a stack to look at opens its stats dialog (E17 M6 T2a1): the stack
            // under the battle's cursor of kind 5, its creature's DEF's groups for its steps.
            let looking = crate::pointer::legal(manifest, &tables, observation, (&legal, &[]));
            let stats_sources = crate::stats::Sources {
                manifest,
                ruleset: &ruleset,
                tables: &tables,
                catalogue: &catalogue,
                towns: Some(&towns),
            };
            let seed = u32::try_from(options.seed & u64::from(u32::MAX)).unwrap_or(1);
            // Morale's and luck's descriptions' pictures' sizes (E17 M6 T2a2b2).
            let described =
                [&layout.stats.morale_described, &layout.stats.luck_described].map(|d| {
                    lookup(&archives, d)
                        .ok()
                        .and_then(|b| h3_assets::def::parse_def(&b).ok())
                        .map_or([0, 0], |d| {
                            [d.width, d.height].map(|v| i32::try_from(v).unwrap_or(i32::MAX))
                        })
                });
            // The stack's dialog with the mouse at `point`: the press's stack there, or the
            // acting stack's by the key T (E17 M6 T2a2a; none in the tactics phase), or `held`
            // the stack there the right button shows while held (E17 M6 T2b1).
            let dialog = |(id, held): (Option<u8>, bool),
                          point: [i32; 2],
                          memory: &mut h3_view::stats::Memory| {
                let field = h3_view::mouse::Field {
                    observation,
                    legal: &looking,
                    facing: &h3_view::mouse::Facing::Sides,
                    bugs: Bugs::Original,
                };
                let id = match id {
                    Some(id) => id,
                    None if held => h3_view::mouse::held_at(&field, &layout, point)?,
                    None => h3_view::mouse::looked_at(&field, &layout, point)?,
                };
                let stack = observation.stacks.iter().find(|s| s.id == id)?;
                let key = manifest.keys.creatures.get(usize::from(stack.creature))?;
                let def = &tables.creatures.get(key)?.def;
                let frames = lookup(&archives, def)
                    .ok()
                    .and_then(|b| h3_assets::def::parse_def(&b).ok())
                    .map(|d| {
                        d.groups
                            .iter()
                            .map(|g| (g.id, u32::try_from(g.frames.len()).unwrap_or(0)))
                            .collect()
                    })
                    .unwrap_or_default();
                let anchor = layout.anchor(stack.hex / 17, stack.hex % 17);
                crate::stats::open(
                    &stats_sources,
                    observation,
                    (id, point, held),
                    (&layout.stats, anchor, described),
                    (frames, viewing, seed),
                    memory,
                )
            };
            let look = |point: [i32; 2], memory: &mut h3_view::stats::Memory| {
                dialog((None, false), point, memory)
            };
            let acting = |point: [i32; 2], memory: &mut h3_view::stats::Memory| {
                let id = observation
                    .active
                    .filter(|_| observation.tactics.is_none())?;
                dialog((Some(id), false), point, memory)
            };
            let held = |point: [i32; 2], memory: &mut h3_view::stats::Memory| {
                dialog((None, true), point, memory)
            };
            // A turret's info where a press shows it (E17 M6 T2b2), the left button's or the
            // right's: the keep's and towers' lines the record tells.
            let turret = |point: [i32; 2], right: bool| {
                let field = h3_view::mouse::Field {
                    observation,
                    legal: &looking,
                    facing: &h3_view::mouse::Facing::Sides,
                    bugs: Bugs::Original,
                };
                if !h3_view::mouse::turret_at(&field, &layout, point, right) {
                    return None;
                }
                let part = |faction: u16, part: usize| {
                    let key = manifest.keys.factions.get(usize::from(faction))?;
                    Some(format!("wall.{key}.{part}"))
                };
                h3_view::turret::text(observation, &catalogue, &part)
            };
            // A panel control's help where the right button shows it (E17 M6 T3a): the panel
            // the opening shows, the tactics phase's own in the phase.
            let help = |point: [i32; 2]| {
                let key = h3_view::panel::help_at(layout.panel_in(observation), point)?;
                Some(catalogue.text(key, &h3_ui::text::Args::new()))
            };
            // A side's hero's info window as the opening fills it (E17 M6 T3b1), where the right
            // button shows it and, at Creature Info's levels 1 and 2, the mouse's move (T4b2b1).
            let hero_window = |side: h3_protocol::wire::Side| {
                let portrait = |hero_type: u16| {
                    crate::results::portrait(&manifest.keys.hero_types, &tables, hero_type)
                };
                let field = crate::stats::field(manifest, observation);
                let info = h3_view::hero_info::Info::of(observation, side, field, &portrait)?;
                Some(h3_view::hero_info::window(
                    &layout.hero_info,
                    &catalogue,
                    side,
                    &info,
                ))
            };
            let hero_info =
                |point: [i32; 2]| hero_window(h3_view::hero_info::at(&layout, observation, point)?);
            // The army info windows as the opening fills them (E17 M6 T4b2b2: a stack's from the
            // stats dialog's numbers, in the original's profile: a Ballista's Dmg its own range,
            // CLT-Q48).
            let army_window = |drawn: h3_view::army_info::Drawn| match drawn {
                h3_view::army_info::Drawn::Hero(side, _) => hero_window(side),
                h3_view::army_info::Drawn::Stack(shown) => {
                    let info =
                        crate::stats::army_info(&stats_sources, observation, shown.stack, viewing)?;
                    Some(h3_view::army_info::stack_window(
                        &layout.army_info,
                        &catalogue,
                        (shown.side, shown.kind),
                        &info,
                        Bugs::Original,
                    ))
                }
            };
            // The combat options dialog the key O or a click on the panel's Options opens while
            // the opening takes a command (E17 M6 T4a): the capture harness's settings at
            // Creature Info's level then (E17 M6 T4b2b2), no audio device (`setup.sh`), its
            // background in the acting side's player's colours.
            let acting_player = observation
                .active
                .and_then(|id| observation.stacks.iter().find(|s| s.id == id))
                .map(|s| usize::from(s.controller == h3_protocol::wire::Side::Defender))
                .map_or(viewing, |i| {
                    observation.sides[i]
                        .colour
                        .unwrap_or_else(|| u8::try_from(i).unwrap_or(0))
                });
            let options =
                |point: Option<[i32; 2]>, (level, shades): (u8, h3_view::shades::Options)| {
                    let on_button = point.is_none_or(|p| {
                        layout.panel_in(observation).button_at(p) == Some(h3_view::panel::OPTIONS)
                    });
                    (on_button && !legal.is_empty()).then(|| {
                        // The grid's and the shadows' boxes as the still's options stand (E17 M6 T4c).
                        let mut settings = h3_view::options::Settings {
                            army_info: level,
                            ..h3_view::options::Settings::CAPTURE
                        };
                        shades.set_in(&mut settings);
                        h3_view::options::Open::new(
                            &layout.options,
                            &catalogue,
                            settings,
                            (acting_player, false, Bugs::Original),
                        )
                    })
                };
            let boxes = crate::spellbook::Boxes {
                tell: &tell,
                leave,
                refusal: &refusal,
                pictures: (def.clone(), size),
                result: &result,
                opening: told
                    .then(|| catalogue.text("battle.tactics.phase", &h3_ui::text::Args::new())),
                look: &look,
                acting: &acting,
                held: &held,
                turret: &turret,
                help: &help,
                hero_info: &hero_info,
                options: &options,
                army: Some(crate::spellbook::Army {
                    layout: &layout,
                    observation,
                    legal: &still_legal,
                    level: army_info,
                    window: &army_window,
                    shades,
                }),
            };
            Some(crate::spellbook::still(
                &actions,
                mouse.unwrap_or(crate::pointer::PARK),
                hero,
                (&layout.book, &catalogue),
                Some(&boxes),
            )?)
        };
        let interface = crate::sequences::Interface {
            panels: &panels,
            legal: &still_legal,
            shown: None,
            walls: options.walls,
            book: book.as_ref(),
            result: None,
            videos: Some(&crate::videos::Videos::open(&options.install)),
            asked: &[],
            closes: &[],
            eagle_eye: &[],
        };
        let drawn = crate::sequences::draw_frames(
            dir,
            &record,
            &timeline,
            &tables,
            &layout,
            (&archives, &exe),
            &interface,
        )?;
        lines.push(format!("{drawn} frames drawn in {}", dir.display()));
        return Ok(lines);
    }
    // The panel at the opening, or at `--at` as the record's turns and lines left it (E17 M3 T6b).
    let mut panel = Panels::new(observation, &[], Vec::new()).at(observation, &[], 0, &layout);
    let list = match options.at {
        None => {
            let moment = scene::Moment {
                active: observation.active,
                walls: options.walls,
                ..scene::Moment::default()
            };
            scene::battle_at(observation, &moment, manifest, &tables, &layout)
        }
        Some(t) => {
            // The record's commands at the original's timing (E17 M2 T4a, `h3_view::anim`).
            let looks = motion::looks(&record, &tables, &archives)?;
            let magic = motion::magic(&record, &tables, &archives, false)?;
            let siege = motion::siege(&record, &tables, &archives)?;
            let casts = motion::casts(&record, &tables, &archives, &options.packs, false)?;
            let stage = motion::stage(&record, &tables, &layout, (&magic, &casts, siege.as_ref()));
            let commands = anim::commands(&record.messages);
            // With the viewer's own fidgets, from the seed (T4b-2a), and its heroes' (E17 M3
            // T3a).
            let seed = u32::try_from(options.seed & u64::from(u32::MAX)).unwrap_or(1);
            let timeline = anim::fidgeting(
                observation,
                &commands,
                &looks,
                stage,
                Speed::new(options.speed),
                seed,
            )
            .map_err(|e| e.to_string())?
            .with_heroes(motion::heroes(&record, &tables, &archives, Some(seed))?);
            lines.push(format!("the timeline ends at {} ms", timeline.end));
            let mut moment = scene::Moment::at(&timeline, t);
            moment.walls = options.walls.or(moment.walls);
            // The mouse parked as the harness parks it (E17 M3 T6c-2): resting there as the battle
            // opens it enters no item and writes no hint (E17 M4 T3b), which needs no words (an
            // installation without text tables).
            let catalogue = crate::log::catalogue(&tables, &archives, &exe).unwrap_or_default();
            let parked = [h3_view::mouse::track::Input::Move {
                at: 0,
                to: crate::pointer::PARK,
            }];
            let shown = crate::pointer::shown(
                &record,
                &timeline,
                &[],
                &parked,
                (&layout, &catalogue),
                (
                    &tables,
                    Bugs::Fixed,
                    h3_view::options::Settings::CAPTURE.auto,
                ),
            );
            let panels = crate::log::panels(
                &record,
                (&shown.log, &timeline.controllers),
                &tables,
                &layout,
                (&archives, &exe),
                Bugs::Fixed,
            )?;
            panel = panels.at(observation, &timeline.turns, t, &layout);
            scene::battle_at(observation, &moment, manifest, &tables, &layout)
        }
    }
    .map_err(|e| e.to_string())?;
    // The panel's draws after the battlefield (E17 M3 T4b-2b-1, T6a, T6b).
    let mut list = list;
    list.extend(panel);
    let names = batch::names(&list, &layout);
    let (batch, loaded) = timed(|| {
        let mut batch = Batch::default();
        for name in &names {
            batch
                .add(name, &lookup(&archives, name)?)
                .map_err(|e| e.to_string())?;
        }
        Ok::<_, String>(batch)
    });
    let batch = batch?;
    let screen = compose(&list, &batch, &layout, Bugs::Fixed).map_err(|e| e.to_string())?;
    let png = options.png();
    write(&png, &h3_render::png::encode(&screen))?;
    lines.insert(0, png.display().to_string());
    lines.push(format!(
        "{} draws; batch of {} assets; archives read in {} ms, batch decoded in {} ms",
        list.len(),
        names.len(),
        read.as_millis(),
        loaded.as_millis()
    ));
    if options.list {
        lines.extend(list.iter().map(|d| format!("{d:?}")));
    }
    Ok(lines)
}

/// The colour whose view the screen shows in `observation`'s battle (`viewing_player` 0x4CEA20;
/// E17 M5 T4a, as the window's battle takes it): the attacker's when human, else the defender's
/// when human, else red (the record names none: red for the attacker, blue for the defender).
pub fn viewing_player(observation: &h3_protocol::wire::Observation) -> u8 {
    let colour = |index: usize| {
        observation.sides[index]
            .colour
            .unwrap_or_else(|| u8::try_from(index).unwrap_or(0))
    };
    match (observation.sides[0].human, observation.sides[1].human) {
        (true, _) => colour(0),
        (false, true) => colour(1),
        (false, false) => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use h3_assets::build;
    use h3_assets::palette::Palette;
    use h3_protocol::codec;
    use h3_protocol::wire::{
        Algorithm, Field, KeyTables, Manifest, PackRef, RecordHeader, Seat, WireU64,
    };

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn options_are_parsed() {
        let o = Options::parse_with(&args(&["a/duel.toml"]), None).unwrap();
        assert_eq!((o.seed, o.list), (1, false));
        assert!(o.install.ends_with("original-data/install"));
        assert!(o.packs.ends_with("packs"));
        assert!(o.png().ends_with("out/render/duel-s1.png"));
        let o = Options::parse_with(
            &args(&[
                "--seed", "7", "r.JSONL", "--list", "--packs", "/p", "--out", "/o.png",
            ]),
            Some("/env".into()),
        )
        .unwrap();
        assert_eq!((o.seed, o.list, o.scenario()), (7, true, false));
        assert_eq!(o.png(), PathBuf::from("/o.png"));
        assert_eq!(
            (o.packs, o.install),
            (PathBuf::from("/p"), PathBuf::from("/env"))
        );
        let o = Options::parse_with(&args(&["r.jsonl", "--install", "/i"]), None).unwrap();
        assert!(o.png().ends_with("out/render/r.png"));
        assert_eq!(o.install, PathBuf::from("/i"));
        let o =
            Options::parse_with(&args(&["a.toml", "--at", "250", "--speed", "2"]), None).unwrap();
        assert_eq!((o.at, o.speed), (Some(250), 2));
        assert!(o.png().ends_with("out/render/a-s1-250ms.png"));
        let o = Options::parse_with(&args(&["a.toml", "--frames", "/f"]), None).unwrap();
        assert_eq!((o.at, o.frames), (None, Some(PathBuf::from("/f"))));
        assert_eq!((o.seat, o.walls), (Seat::Spectator, None));
        let states = "0,0,0,0,0,2,1,0,1,2,1,0,1,2,2,1,1,1,3";
        let o = Options::parse_with(&args(&["a.toml", "--walls", states]), None).unwrap();
        let walls = o.walls.unwrap();
        assert_eq!((walls.parts[6], walls.parts[17], walls.gate), (1, 1, 3));
        for (name, seat) in [("attacker", Seat::Attacker), ("defender", Seat::Defender)] {
            let o = Options::parse_with(&args(&["a.toml", "--seat", name]), None).unwrap();
            assert_eq!(o.seat, seat);
        }
        for bad in [
            &["a.toml", "--seat", "spectator"][..],
            &["r.jsonl", "--seat", "defender"],
            &["a.toml", "--walls", "1,2,3"],
            &["a.toml", "--walls", ""],
            &["a.toml", "--walls", "x"],
            &["a.toml", "--at", "1", "--frames", "/f"],
            &["--seed"],
            &["--seed", "x", "a.toml"],
            &["--at", "soon", "a.toml"],
            &["--speed", "3", "a.toml"],
            &["--what"],
            &["a.toml", "b.toml"],
            &[],
        ] {
            assert!(Options::parse_with(&args(bad), None).is_err(), "{bad:?}");
        }
    }

    /// A record of an empty battle of the `base` packs: it draws nothing, so its batch is only the
    /// outline palette.
    fn empty_record() -> String {
        use h3_protocol::wire::ServerMessage;
        let header = RecordHeader {
            format: h3_protocol::RECORD_FORMAT_VERSION,
            game: 1,
            seat: Seat::Spectator,
            manifest: Manifest {
                protocol: h3_protocol::PROTOCOL_VERSION,
                engine: "test".into(),
                packs: vec![PackRef {
                    id: "base".into(),
                    version: "1".into(),
                }],
                profile: "base:faithful".into(),
                ruleset_hash: WireU64(0),
                battle_ruleset_hash: WireU64(0),
                rng: Algorithm {
                    name: "r".into(),
                    version: 1,
                },
                hash: Algorithm {
                    name: "h".into(),
                    version: 1,
                },
                battle_state_hash_version: 1,
                field: Field {
                    width: 17,
                    height: 11,
                },
                keys: KeyTables::default(),
            },
            first: 1,
        };
        let text = crate::scenario::record_of(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../packs/synthetic/scenarios/duel.toml"),
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../packs"),
            1,
            Seat::Spectator,
        )
        .unwrap();
        let mut snapshot = record::parse(&text).unwrap().messages.remove(0);
        if let ServerMessage::Snapshot { observation, .. } = &mut snapshot {
            observation.stacks.clear();
            observation.removable_obstacles.clear();
            observation.absolute_obstacle = None;
            observation.terrain = None;
        }
        [codec::encode_text(&header), codec::encode_text(&snapshot)].join("\n")
    }

    #[test]
    fn a_record_renders_from_a_synthetic_installation() {
        use h3_formats::lod::build as lod;
        let root = std::env::temp_dir().join(format!("h3-view-render-{}", std::process::id()));
        let data = root.join("install/Data");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&data).unwrap();
        let pal = build::pal(&Palette::default(), &[0; 256], 0, &[]);
        // The panel's log item: its picture and font (E17 M3 T4b-2b-1).
        let picture = build::pcx_indexed(1, 1, &[0], &Palette::default());
        let font = build::fnt(&h3_assets::fnt::Font {
            height: 1,
            y_offset: 0,
            glyphs: Vec::new(),
        });
        // The panel's picture and buttons (E17 M3 T6a): one-pixel pictures and DEFs.
        let frame = h3_assets::def::Frame {
            encoding: h3_assets::def::Encoding::Raw,
            full_width: 1,
            full_height: 1,
            left: 0,
            top: 0,
            width: 1,
            height: 1,
            pixels: vec![1],
            runs: vec![false],
            short_header: false,
        };
        let button = build::def(
            0x47,
            (1, 1),
            &Palette::default(),
            &[(0, &[("a", 0)])],
            &[frame],
        );
        let buttons = [
            "ICM001.DEF",
            "ICM002.DEF",
            "ICM003.DEF",
            "ICM004.DEF",
            "ICM005.DEF",
            "ICM006.DEF",
            "ICM007.DEF",
            "COMSLIDE.DEF",
        ]
        .map(|name| (name, &button[..], true));
        for (name, items) in [
            ("H3sprite.lod", buttons.to_vec()),
            ("H3ab_spr.lod", vec![]),
            (
                "H3bitmap.lod",
                vec![
                    ("CROLLOVR.PCX", &picture[..], true),
                    ("SMALFONT.FNT", &font[..], true),
                    ("CBAR.PCX", &picture[..], true),
                    ("PLAYERS.PAL", &pal[..], true),
                ],
            ),
            ("h3ab_bmp.lod", vec![("GAME.PAL", &pal[..], true)]),
        ] {
            fs::write(data.join(name), lod(0x200, &items)).unwrap();
        }
        let input = root.join("empty.jsonl");
        fs::write(&input, empty_record()).unwrap();
        let options = Options {
            out: Some(root.join("out/still.png")),
            list: true,
            ..Options::parse_with(&args(&["x.jsonl"]), Some(root.join("install"))).unwrap()
        };
        let options = Options { input, ..options };
        let lines = render(&options).unwrap();
        assert_eq!(lines[0], root.join("out/still.png").display().to_string());
        assert!(
            lines[1].starts_with("11 draws; batch of 13 assets"),
            "{lines:?}"
        );
        let png = fs::read(root.join("out/still.png")).unwrap();
        assert_eq!(png[16..24], [0, 0, 3, 32, 0, 0, 2, 88]); // 800 x 600
        // At a time: the record's (no) commands played first.
        let later = Options {
            at: Some(500),
            ..Options { ..options }
        };
        let lines = render(&later).unwrap();
        assert!(
            lines.contains(&"the timeline ends at 0 ms".to_owned()),
            "{lines:?}"
        );
        let options = Options { at: None, ..later };
        // A record without a snapshot, and an archive without the palette.
        let header = empty_record().lines().next().unwrap().to_owned();
        fs::write(root.join("bare.jsonl"), &header).unwrap();
        let bare = Options {
            input: root.join("bare.jsonl"),
            ..Options::parse_with(&args(&["x"]), Some(root.join("install"))).unwrap()
        };
        assert_eq!(render(&bare).unwrap_err(), "the record holds no snapshot");
        fs::write(data.join("h3ab_bmp.lod"), lod(0x200, &[])).unwrap();
        assert!(
            render(&options)
                .unwrap_err()
                .contains("GAME.PAL: in none of")
        );
        fs::remove_dir_all(&root).unwrap();
    }
}
