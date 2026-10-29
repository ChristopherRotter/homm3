//! `h3-view sequences` (E17 M2 T4a): the recorded sequences (`clients/situations/sequences.toml`)
//! played by the engine in-process and timed by the viewer (`h3_view::anim`), each timeline
//! compared with the one the capture harness recorded in the original (`docs/epics/E17-design.md`
//! §7.6). A sequence's `[[sequence.command]]` entries are the engine's commands for what the
//! harness's actions did there (a seat and a wire command); one without them is skipped. With
//! `--frames` (E17 M2 T4b-1, design §7.9) it also draws the viewer's frame at each time the
//! harness kept a frame of the original (`<dir>/<id>/frames.toml`, `h3drive.py frames`), for the
//! harness's diff. An adapter: it reads files and the archives.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use h3_formats::lod::LodArchive;
use h3_protocol::wire::{Command, Seat, ServerMessage, Side};
use h3_render::Bugs;
use h3_view::anim::{self, Playhead, Pose, Speed, Timeline};
use h3_view::batch::{self, Batch};
use h3_view::compose::compose;
use h3_view::hero::HeroPose;
use h3_view::layout::BattleLayout;
use h3_view::mouse::track::{Input, Shown};
use h3_view::mouse::{self, Legal};
use h3_view::record::Record;
use h3_view::scene::Panels;
use h3_view::siege::Walls;
use h3_view::tables::Tables;
use h3_view::{layout, record, scene};
use serde::Deserialize;

use crate::render::{lookup, read_archives, write};
use crate::{assets, log, motion, pointer, scenario};

/// The sequences file.
#[derive(Debug, Deserialize)]
struct File {
    sequence: Vec<Sequence>,
}

/// One recorded sequence (the harness's fields besides these are its own).
#[derive(Debug, Deserialize)]
struct Sequence {
    id: String,
    scenario: String,
    timeline: String,
    /// The milliseconds stepped before the actions, and after them.
    #[serde(default)]
    pre: u32,
    ms: u32,
    #[serde(default)]
    command: Vec<Step>,
    /// The harness's actions in the original (E17 M3 T6c-2: the mouse's inputs,
    /// `crate::pointer::inputs`).
    #[serde(default)]
    actions: Vec<String>,
    /// The timeline holds the heroes' changes (E17 M3 T7a-2a; compared since T7b-1).
    #[serde(default)]
    heroes: bool,
    /// The engine's seed (E17 M3 T7b-1: a roll the original made, a resistance, the engine's own
    /// generator makes the same way with it; default 1).
    #[serde(default = "first_seed")]
    seed: u64,
}

/// The engine's seed unless a sequence names one.
const fn first_seed() -> u64 {
    1
}

impl Sequence {
    /// When the harness's recording ends: `ms` after its last actions (`pre`, or the last
    /// command's `at` in a battle in progress, M3 T1a; or the last `at:<ms>` action's time, E17
    /// M5 T4c1: a key pressed while the computer plays).
    fn until(&self) -> u32 {
        let at = self
            .actions
            .iter()
            .filter_map(|a| a.strip_prefix("at:")?.parse::<u32>().ok());
        let last = self
            .command
            .iter()
            .filter_map(|c| c.at)
            .chain(at)
            .fold(self.pre, u32::max);
        last + self.ms
    }
}

/// A command of a seat; `at`, when a player gave it (M3 T1a): its time after the opening (the
/// first command's defaults to the sequence's `pre`); a command without one is the computer's,
/// which acts as soon as its turn comes.
#[derive(Debug, Deserialize)]
struct Step {
    seat: Seat,
    command: Command,
    #[serde(default)]
    at: Option<u32>,
}

/// The frames the harness kept of a sequence in the original (`h3drive.py frames`), or of a
/// situation (`h3drive.py set`).
#[derive(Debug, Deserialize)]
struct Frames {
    frame: Vec<Kept>,
    /// Where the harness left the mouse (E17 M3 T6c-1: a situation's, drawn with its cursor, hint
    /// and marked stack; none for a sequence, whose mouse moves: `Interface::shown`).
    #[serde(default)]
    mouse: Option<[i32; 2]>,
    /// The draws the original's Chain Lightning rays turned by, in order (E17 M3 T7c-3:
    /// `xdd.dll`'s log; the frame check's timeline draws the same rays).
    #[serde(default)]
    rays: Vec<i32>,
    /// A situation's actions after the mouse's move (E17 M5 T3a: the spell book's stills;
    /// `crate::spellbook::still`).
    #[serde(default)]
    actions: Vec<String>,
}

/// A situation's mouse and actions as the harness kept them in `here`'s `frames.toml` (E17 M5
/// T3a).
pub(crate) fn situation_inputs(here: &Path) -> Result<(Option<[i32; 2]>, Vec<String>), String> {
    let path = here.join("frames.toml");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let kept: Frames = toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok((kept.mouse, kept.actions))
}

/// What the frames draw below and over the battlefield: the panel over the battle and the mouse:
/// over time (a sequence, E17 M3 T6c-2), or at the opening where the frames name it, asking the
/// acting stack's legal commands then (a situation, T6c-1).
pub(crate) struct Interface<'a> {
    /// The panel.
    pub panels: &'a Panels,
    /// The legal commands at the opening.
    pub legal: &'a Legal,
    /// The mouse over time.
    pub shown: Option<&'a Shown>,
    /// A siege's wall states in place of the record's (E17 M4 T1b: the harness writes them
    /// into the original before its first frame; *provisional*).
    pub walls: Option<Walls>,
    /// What a situation's actions left of the spell book (E17 M5 T3a): drawn over the panel, its
    /// cursor over all at the mouse they left.
    pub book: Option<&'a crate::spellbook::Still>,
    /// The result window a sequence's battle ends with (E17 M5 T4b1): drawn from the timeline's
    /// end on; with its tune (E17 M5 T4b2b: its clip's).
    pub result: Option<&'a (h3_view::message::MessageBox, h3_view::sound::Tune)>,
    /// The video archives the result window's clip comes from (E17 M5 T4b2b; none: no clip).
    pub videos: Option<&'a crate::videos::Videos>,
}

/// One kept frame: its time, the stacks the original shows fidgeting then (the viewer draws
/// none of its own in a check), each `[side, record, group, frame]`, and its heroes (E17 M3 T3a:
/// their defeats and victories come from its generator), each `[side, group, frame]`: the frame
/// check's, not the capture set's, whose heroes are the viewer's own. The flags are always the
/// viewer's.
#[derive(Debug, Deserialize)]
struct Kept {
    t: u32,
    file: String,
    #[serde(default)]
    fidgets: Vec<[u32; 4]>,
    #[serde(default)]
    heroes: Vec<[u32; 3]>,
    /// The heroes of the screen it replaced within its millisecond (E17 M3 T7c-3): a cast's
    /// first frame is in memory a deadline before the screen shows it, the screen still showing
    /// these.
    #[serde(default)]
    before: Vec<[u32; 3]>,
    /// The point the original's cursor is drawn at (its memory: it reads the mouse on a 33 ms
    /// grid of its own, E17 M3 T6c-2, design §9.10); none: the mouse's.
    #[serde(default)]
    cursor: Option<[i32; 2]>,
}

/// The command's options.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    only: Option<String>,
    speed: Speed,
    packs: PathBuf,
    install: PathBuf,
    out: PathBuf,
    frames: Option<PathBuf>,
}

impl Options {
    /// Parses `[<id>] [--speed <0-2>] [--packs <dir>] [--install <dir>] [--out <dir>]
    /// [--frames <dir>]`.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let env = std::env::var_os("H3_INSTALL").map(PathBuf::from);
        let (mut only, mut speed, mut packs, mut install, mut out) = (None, 0, None, None, None);
        let mut frames = None;
        let mut rest = args.iter();
        while let Some(arg) = rest.next() {
            let mut value = || {
                rest.next()
                    .cloned()
                    .ok_or_else(|| format!("sequences: `{arg}` needs a value"))
            };
            match arg.as_str() {
                "--speed" => {
                    let v = value()?;
                    speed = v
                        .parse()
                        .ok()
                        .filter(|s| *s <= 2)
                        .ok_or(format!("sequences: bad speed `{v}`"))?;
                }
                "--packs" => packs = Some(PathBuf::from(value()?)),
                "--install" => install = Some(PathBuf::from(value()?)),
                "--out" => out = Some(PathBuf::from(value()?)),
                "--frames" => frames = Some(PathBuf::from(value()?)),
                flag if flag.starts_with("--") => {
                    return Err(format!("sequences: unknown option `{flag}`"));
                }
                id if only.is_none() => only = Some(id.to_owned()),
                extra => return Err(format!("sequences: unexpected `{extra}`")),
            }
        }
        Ok(Self {
            only,
            speed: Speed::new(speed),
            packs: packs.unwrap_or_else(|| manifest.join("../../../packs")),
            install: install
                .or(env)
                .unwrap_or_else(|| manifest.join("../../../original-data/install")),
            out: out.unwrap_or_else(|| manifest.join("../../out/sequences")),
            frames,
        })
    }
}

/// Runs the command: exits 1 when a timeline differs from its recording.
pub fn run(options: &Options) -> ExitCode {
    match sequences(options) {
        Ok(0) => ExitCode::SUCCESS,
        Ok(_) => ExitCode::from(1),
        Err(message) => {
            eprintln!("sequences: {message}");
            ExitCode::FAILURE
        }
    }
}

/// Plays and compares the sequences; returns how many differ.
fn sequences(options: &Options) -> Result<usize, String> {
    let dir = assets::default_dir().join("../situations");
    let path = dir.join("sequences.toml");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let file: File = toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let layout_path = assets::default_dir().join("../layouts/battle.toml");
    let layout = layout::parse_battle(
        &fs::read_to_string(&layout_path).map_err(|e| format!("{}: {e}", layout_path.display()))?,
    )
    .map_err(|e| format!("{}: {e}", layout_path.display()))?;
    let bytes = read_archives(&options.install)?;
    let videos = crate::videos::Videos::open(&options.install);
    // The executable's own strings: read only when a line needs them.
    let exe = crate::text::executable(&options.install).unwrap_or_default();
    let archives = bytes
        .iter()
        .map(|(name, b)| {
            LodArchive::parse(b)
                .map(|a| (*name, a))
                .map_err(|e| format!("{name}: {e:?}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut differing = 0;
    for sequence in &file.sequence {
        if options.only.as_ref().is_some_and(|id| *id != sequence.id) {
            continue;
        }
        let steps: Vec<(Seat, Command)> = sequence
            .command
            .iter()
            .map(|s| (s.seat, s.command))
            .collect();
        let text = scenario::played(
            &dir.join(&sequence.scenario),
            &options.packs,
            sequence.seed,
            &steps,
        )
        .map_err(|e| format!("{}: {e}", sequence.id))?;
        write(
            &options.out.join(format!("{}.jsonl", sequence.id)),
            text.as_bytes(),
        )?;
        let record = record::parse(&text).map_err(|e| format!("{}: {e}", sequence.id))?;
        let top = record
            .header
            .manifest
            .packs
            .last()
            .map_or("", |p| p.id.as_str());
        let tables = assets::load(&assets::default_dir(), top)?;
        let looks = motion::looks(&record, &tables, &archives)?;
        let magic = motion::magic(&record, &tables, &archives, false)?;
        let siege = motion::siege(&record, &tables, &archives)?;
        let mut casts = motion::casts(&record, &tables, &archives, &options.packs, false)?;
        // The frame check draws Chain Lightning's rays from the original's draws (E17 M3 T7c-3).
        if let Some(dir) = &options.frames {
            casts.rays = kept_rays(&dir.join(&sequence.id))?;
        }
        // The original's drawing bugs, as its captures show them (E17 M3 T7d, CLT-Q8).
        let stage = anim::Stage {
            bugs: Bugs::Original,
            ..motion::stage(&record, &tables, &layout, (&magic, &casts, siege.as_ref()))
        };
        // The harness acts `pre` ms after the opening, and then at each command's `at`.
        let commands: Vec<anim::Command<'_>> = record
            .messages
            .iter()
            .filter_map(|m| match m {
                ServerMessage::Events { events, .. } => Some(events),
                _ => None,
            })
            .enumerate()
            .map(|(i, events)| anim::Command {
                at: sequence
                    .command
                    .get(i)
                    .and_then(|s| s.at)
                    .or((i == 0).then_some(sequence.pre)),
                events,
            })
            .collect();
        let snapshot = record
            .first_snapshot()
            .ok_or("the record holds no snapshot")?;
        let mut timeline = anim::timeline(snapshot, &commands, &looks, stage, options.speed)
            .map_err(|e| format!("{}: {e}", sequence.id))?
            .with_heroes(motion::heroes(&record, &tables, &archives, None)?);
        // The mouse as the harness moved it (E17 M3 T6c-2): its cursor, the stack it marks and
        // enters, the status line's hints, each turn's battle from the engine.
        let scenario_path = dir.join(&sequence.scenario);
        // The heroes' casts in order, which the `cast:` actions aim (E17 M3 T7b-1).
        let spells: Vec<u16> = record
            .messages
            .iter()
            .filter_map(|m| match m {
                ServerMessage::Events { events, .. } => Some(events),
                _ => None,
            })
            .flatten()
            .filter_map(|e| match *e {
                h3_protocol::wire::Event::SpellCast { spell, .. } => Some(spell),
                _ => None,
            })
            .collect();
        // A spell by its number in the original (an `aim:` action's, E17 M5 T3b).
        let keys = &record.header.manifest.keys.spells;
        let handle =
            |original: u16| motion::spell_numbered_in(keys, &tables, original).map(|(h, _)| h);
        let inputs = pointer::inputs(
            &sequence.actions,
            sequence.pre,
            &layout,
            (&timeline, &spells, &handle),
            h3_view::tactics::told(snapshot),
        )
        .map_err(|e| format!("{}: {e}", sequence.id))?;
        // The spells the actions aim, each turn's aims of them (E17 M5 T3b).
        let mut aimed: Vec<u16> = inputs
            .iter()
            .filter_map(|i| match *i {
                Input::Aim { spell, .. } => Some(spell),
                _ => None,
            })
            .collect();
        aimed.sort_unstable();
        aimed.dedup();
        let turned = scenario::turned(
            &scenario_path,
            &options.packs,
            sequence.seed,
            &steps,
            &aimed,
        )
        .map_err(|e| format!("{}: {e}", sequence.id))?;
        let catalogue = log::catalogue(&tables, &archives, &exe)?;
        let shown = pointer::shown(
            &record,
            &timeline,
            &turned,
            &inputs,
            (&layout, &catalogue),
            (&tables, Bugs::Original),
        );

        pointer::entered(
            &mut timeline,
            &shown,
            &record,
            &looks,
            tables.battle.groups.mouse_over,
        );
        // What the harness saw: its recording ends `ms` after the actions.
        let until = sequence.until();
        // The timeline read forward as the window reads it (E17 M5 T1a).
        if let Some(t) = playhead_differs(&timeline, until) {
            differing += 1;
            println!(
                "{}: the playhead's moment at {t} ms is not the timeline's",
                sequence.id
            );
        }
        let lines = if sequence.heroes {
            timeline.lines_with_heroes(until)
        } else {
            timeline.lines()
        };
        let ours: String = lines
            .lines()
            .filter(|l| l.split(' ').next().and_then(|t| t.parse::<u32>().ok()) <= Some(until))
            .map(|l| format!("{l}\n"))
            .collect();
        let ours_path = options.out.join(format!("{}.txt", sequence.id));
        write(&ours_path, ours.as_bytes())?;
        if let Some(dir) = &options.frames {
            let here = dir.join(&sequence.id);
            // The result window the battle ends with (E17 M5 T4b1): the engine's outcome after
            // the sequence's commands.
            let played: Vec<Command> = steps.iter().map(|&(_, c)| c).collect();
            let (ruleset, outcome) =
                scenario::outcome(&scenario_path, &options.packs, sequence.seed, &played)
                    .map_err(|e| format!("{}: {e}", sequence.id))?;
            let sources = crate::results::Sources {
                manifest: &record.header.manifest,
                tables: &tables,
                ruleset: &ruleset,
                catalogue: &catalogue,
                layout: &layout,
            };
            let result = outcome.and_then(|o| {
                crate::results::window(
                    &sources,
                    snapshot,
                    &o,
                    crate::render::viewing_player(snapshot),
                )
            });
            // Auto Combat as the actions switched it (E17 M5 T4c1).
            let panels = log::panels(
                &record,
                (&shown.log, &timeline.controllers),
                &tables,
                &layout,
                (&archives, &exe),
                Bugs::Original,
            )?
            .with_auto(snapshot, pointer::auto(&inputs, &timeline));
            let interface = Interface {
                panels: &panels,
                legal: &Legal::default(),
                shown: Some(&shown),
                walls: None,
                book: None,
                result: result.as_ref(),
                videos: Some(&videos),
            };
            let drawn = draw_frames(
                &here,
                &record,
                &timeline,
                &tables,
                &layout,
                (&archives, &exe),
                &interface,
            )?;
            println!(
                "{}: {drawn} frames drawn in {}",
                sequence.id,
                here.display()
            );
        }
        if options.speed.setting() != 0 {
            println!(
                "{}: {} changes at speed {} (recorded at 0 only)",
                sequence.id,
                ours.lines().count(),
                options.speed.setting()
            );
            continue;
        }
        let recorded_path = dir.join(&sequence.timeline);
        let recorded = fs::read_to_string(&recorded_path)
            .map_err(|e| format!("{}: {e}", recorded_path.display()))?;
        // The original's own fidgets are not compared (the viewer draws none of its own in a
        // check, design §7.2), but those of the stacks the mouse entered (E17 M3 T6c-2).
        let entered: std::collections::BTreeSet<String> = shown
            .entered
            .iter()
            .filter_map(|(_, id)| timeline.member(*id))
            .map(|m| format!("{}.{}", u8::from(m.side == Side::Defender), m.record))
            .collect();
        // The screens' boxes (`screens`, E17 M3 T7a-2b) are the frame check's, not the timeline's.
        let recorded: String = recorded
            .lines()
            .filter(|l| l.split(' ').nth(1) != Some("screen"))
            .map(|l| format!("{l}\n"))
            .collect();
        let recorded = anim::unfidgeted(&recorded, &entered);
        // Nor the heroes' rolled reactions to a death (E17 M3 T7c-3).
        let g = tables.battle.hero_groups;
        let recorded = anim::unrolled(&recorded, g.standing, [g.defeat, g.victory]);
        let diff = anim::differences(&ours, &recorded);
        if diff.is_empty() {
            println!(
                "{}: {} changes, as recorded",
                sequence.id,
                ours.lines().count()
            );
        } else {
            differing += 1;
            println!(
                "{}: differs from {} ({})",
                sequence.id,
                recorded_path.display(),
                ours_path.display()
            );
            for line in diff.iter().take(40) {
                println!("  {line}");
            }
        }
    }
    Ok(differing)
}

/// The first time up to `until` at which `timeline`'s playhead, read forward from 0 through its
/// changes, shows another moment than [`scene::Moment::at`] (E17 M5 T1a), if one does.
fn playhead_differs(timeline: &Timeline, until: u32) -> Option<u32> {
    let mut head = Playhead::new(timeline);
    timeline.changes(until).into_iter().find(|&t| {
        head.seek(timeline, t);
        head.moment(timeline) != scene::Moment::at(timeline, t)
    })
}

/// The draws the original's Chain Lightning rays turned by in `<here>/frames.toml` (E17 M3
/// T7c-3; none without the file).
fn kept_rays(here: &Path) -> Result<Vec<i32>, String> {
    let path = here.join("frames.toml");
    let Ok(text) = fs::read_to_string(&path) else {
        return Ok(Vec::new());
    };
    let kept: Frames = toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(kept.rays)
}

/// Draws the viewer's frame at each time in `<here>/frames.toml` (the stacks the original shows
/// fidgeting then in its poses) into `<here>/ours/<file>.png`, what it drew into
/// `<here>/ours.txt` and the text its log's item showed into `<here>/ours-log.json` (by the
/// frame's file name, E17 M3 T6b: the harness compares the item where the original's shows the
/// same); returns how many. The panel is `panels`' at each time.
pub(crate) fn draw_frames(
    here: &Path,
    record: &Record,
    timeline: &Timeline,
    tables: &Tables,
    layout: &BattleLayout,
    (archives, exe): (&[(&str, LodArchive<'_>)], &[u8]),
    interface: &Interface<'_>,
) -> Result<usize, String> {
    let panels = interface.panels;
    let path = here.join("frames.toml");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let kept: Frames = toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let snapshot = record
        .first_snapshot()
        .ok_or("the record holds no snapshot")?;
    let manifest = &record.header.manifest;
    let mut lists = Vec::new();
    let mut drawn = String::new();
    let mut texts = serde_json::Map::new();
    // A situation's mouse where the harness left it, at the opening (E17 M3 T6c-1), or where its
    // actions moved it over the battle (E17 M5 T4c2a), none while the tactics phase's message
    // left the box's cursor (CLT-Q32); a sequence's over time (T6c-2: `interface.shown`).
    let arrow = interface.book.is_some_and(|b| b.arrow);
    let rest = interface.book.and_then(|b| b.over).or(kept.mouse);
    let pointer = rest.filter(|_| !arrow).map(|point| {
        (
            point,
            mouse::pointer(snapshot, interface.legal, layout, (point, Bugs::Original)),
        )
    });
    let status = match &pointer {
        Some((_, p)) => {
            let catalogue = log::catalogue(tables, archives, exe)?;
            Some(pointer::hint_text(record, &catalogue, &p.hint, snapshot))
        }
        None => None,
    };
    // The frames' times are read forward (E17 M5 T1a; one earlier than the last starts over).
    let mut head = Playhead::new(timeline);
    for frame in &kept.frame {
        // While the spell book is open the battle behind it stands as it was when the book opened
        // (E17 M5 T3a: the dialog's modal loop runs no idle step; the original's frames are the
        // same 100 ms and a second after).
        let at = interface
            .book
            .filter(|b| b.dialog())
            .map_or(frame.t, |b| b.at.min(frame.t));
        // Behind the result window the battle stands as it ended, its flags as last shown (E17 M5
        // T4b1, `h3_view::result::behind`).
        let ended = match (interface.result, timeline.ended) {
            (Some(_), Some((end, _))) if frame.t >= end => Some(end),
            _ => None,
        };
        let at = ended.map_or(at, |end| at.min(end));
        head.seek(timeline, at);
        let mut moment = head.moment(timeline);
        if let Some(end) = ended {
            let before = scene::Moment::at(timeline, end.saturating_sub(1));
            h3_view::result::behind(&mut moment, &before);
        }
        moment.walls = interface.walls.or(moment.walls);
        moment.marked = match interface.shown {
            Some(shown) => shown.marked(frame.t),
            None => pointer.as_ref().and_then(|(_, p)| p.marked),
        };
        // The stacks an area spell's aim marks (E17 M5 T3b).
        if let Some(shown) = interface.shown {
            moment.area = shown.area(frame.t).to_vec();
        }
        // The tactics phase's message the battle opened with (E17 M5 T4c2a): behind it the
        // battle before its first turn, no stack outlined or marked; once it closed, a
        // situation's opening turn shaded, a sequence's human turns in the phase while they wait
        // (E17 M5 T4c2b).
        let opening = interface.book.is_some_and(|b| b.opening);
        if opening {
            moment.before_first_turn();
        } else if interface.shown.is_none() {
            moment.shade = h3_view::tactics::shade(snapshot);
        } else {
            moment.shade =
                h3_view::tactics::shade_at(snapshot, panels.turns(), &timeline.log, frame.t);
        }
        for &[side, index, group, f] in &frame.fidgets {
            let side = if side == 0 {
                Side::Attacker
            } else {
                Side::Defender
            };
            let member = u8::try_from(index)
                .ok()
                .and_then(|record| timeline.member_at(side, record));
            // A stack the mouse entered fidgets as the viewer's timeline has it (E17 M3 T6c-2:
            // the original's memory runs ahead of its screen there).
            let entered = |m: &&anim::Member| {
                interface
                    .shown
                    .is_some_and(|s| s.entered.iter().any(|&(_, id)| id == m.stack))
            };
            if let Some(m) = member.filter(|m| !entered(m)) {
                let now = moment.poses.get(&m.stack).copied().unwrap_or(m.start);
                let pose = Pose {
                    group,
                    frame: f,
                    offset: [0, 0],
                    ..now
                };
                moment.poses.insert(m.stack, pose);
            }
        }
        let casting = u32::from(tables.battle.hero_groups.cast);
        for &[side, group, f] in &frame.heroes {
            let before = frame
                .before
                .iter()
                .find(|b| b[0] == side)
                .map(|b| (b[1], b[2]));
            if let Some(hero) = moment
                .heroes
                .get_mut(side as usize)
                .and_then(Option::as_mut)
            {
                // A cast's frames as the viewer's timeline shows them (E17 M3 T7b-1): the
                // original's memory holds each a deadline before its screen shows it, so as a
                // cast starts the screen still shows the frame it replaced (E17 M3 T7c-3,
                // `raise`: a rolled defeat's).
                let (group, f) = match before {
                    Some(b) if group == casting && hero.group != casting && b.0 != casting => b,
                    _ if group == casting || hero.group == casting => continue,
                    _ => (group, f),
                };
                *hero = HeroPose {
                    group,
                    frame: f,
                    ..*hero
                };
            }
        }

        let mut list = scene::battle_at(snapshot, &moment, manifest, tables, layout)
            .map_err(|e| e.to_string())?;
        // The moment, then the count boxes drawn (` box<x>,<y>n<count>@<side>.<record>`, their
        // top-left corners: a sequence that does not compare them masks them where they show,
        // E17 M3 T5a; one that does masks those whose count is not the original's, T5b).
        let mut line = describe(frame.t, &moment);
        line.pop();
        for d in &list {
            if let scene::Draw::CountBox {
                stack, x, y, count, ..
            } = d
            {
                let _ = write!(line, " box{x},{y}n{count}");
                if let Some(m) = timeline.member(*stack) {
                    let side = u8::from(m.side == Side::Defender);
                    let _ = write!(line, "@{side}.{}", m.record);
                }
            }
        }
        line.push('\n');
        drawn.push_str(&line);
        // The panel as its last turn's start set it, its log's item as the battle's lines and
        // turns left it (E17 M3 T6b), or the status line's hint where the mouse is (T6c-1:
        // *provisional*, the situations' battles write no lines; T6c-2 times it).
        let text = status.as_deref().unwrap_or_else(|| panels.text(at));
        if opening {
            list.extend(Panels::before(snapshot, layout));
        } else {
            list.extend(panels.at_showing(snapshot, &timeline.turns, at, layout, text));
        }
        // Earthquake's shake (E17 M4 T2c-2): the screen as it was, shifted; the cursor hidden.
        if let Some(steps) = moment.shake {
            list = vec![scene::shaken(list, steps, layout)];
        }
        // The spell book a situation's actions opened (E17 M5 T3a), in its hero's player's
        // colours (the record names none: red for the attacker, blue for the defender, CLT-Q6).
        let book = interface
            .book
            .and_then(|b| Some((b.open.as_ref()?, b.mouse)));
        if let Some((open, _)) = book {
            let index = usize::from(snapshot.viewer == Side::Defender);
            let player = snapshot.sides[index]
                .colour
                .unwrap_or_else(|| u8::try_from(index).unwrap_or(0));
            list.extend(open.draws(&layout.book, &layout.player_palette, player));
        }
        // A message box the actions left open (E17 M5 T4a), over the book and the battle, the
        // default cursor's frame 0 at the mouse (0x4F74AC).
        let message = interface
            .book
            .and_then(|b| Some((b.message.as_ref()?, b.mouse)));
        // The result window (E17 M5 T4b1): a still's at once, a sequence's from its battle's end;
        // over the whole screen darkened, the combat cursor's pointer at the mouse.
        // With it the time it opened, for its clip (E17 M5 T4b2b).
        let result = match interface
            .book
            .and_then(|b| Some((b.result.as_ref()?, b.mouse, b.at)))
        {
            Some((window, mouse, at)) => Some((window, Some(mouse), at)),
            None => interface
                .result
                .zip(timeline.ended.filter(|&(t, _)| frame.t >= t))
                .map(|(window, (end, _))| {
                    let point = interface
                        .shown
                        .and_then(|shown| frame.cursor.or_else(|| shown.point(frame.t)))
                        .or_else(|| pointer.as_ref().map(|(p, _)| *p));
                    (window, point, end)
                }),
        };
        if let Some(((window, tune), mouse, opened)) = result {
            list.push(h3_view::result::backdrop(layout.frame));
            list.extend(
                window
                    .dialog
                    .draws(&window.shown, &layout.player_palette)
                    .into_iter()
                    .map(scene::Draw::Dialog),
            );
            // Its clip over its slot, when the video archives hold it (E17 M5 T4b2b).
            let clip = h3_view::clip::of(*tune);
            let held = |videos: &crate::videos::Videos| {
                std::iter::once(clip.start)
                    .chain(clip.looped)
                    .all(|name| videos.has(name))
            };
            if interface.videos.is_some_and(held) {
                let since = frame.t.saturating_sub(opened);
                list.push(h3_view::result::clip(&layout.result, *tune, since));
            }
            if let Some(point) = mouse {
                list.push(mouse::cursor(layout, h3_view::result::CURSOR_FRAME, point));
            }
        } else if let Some((message, mouse)) = message {
            list.extend(
                message
                    .dialog
                    .draws(&message.shown, &layout.player_palette)
                    .into_iter()
                    .map(scene::Draw::Dialog),
            );
            list.push(scene::Draw::Cursor {
                def: layout.message.cursor.clone(),
                frame: 0,
                x: mouse[0],
                y: mouse[1],
                hot: layout.book.hot.first().copied().unwrap_or_default(),
            });
        } else if let Some((open, mouse)) = book {
            // The cursor the book set, else the battle's where the actions left the mouse.
            match open.cursor_draw(&layout.book, mouse) {
                Some(cursor) => list.push(cursor),
                None => {
                    if let Some((_, p)) = &pointer {
                        list.push(p.cursor(layout, mouse));
                    }
                }
            }
        } else if let Some(shown) = interface.shown {
            // The cursor's frame now, drawn where the original's is (its memory), else at the
            // mouse; none while a ray hides it.
            let point = frame.cursor.or_else(|| shown.point(frame.t));
            let visible = !timeline.cursor_hidden(frame.t);
            let look = shown.look(frame.t, &layout.mouse);
            if let (Some(look), Some(point), true) = (look, point, visible) {
                list.push(mouse::look_cursor(layout, look, point));
            }
        } else if let Some(still) = interface.book.filter(|b| b.arrow) {
            // The pointer the turn's start set where the mouse rests (0x478321; CLT-Q32).
            list.push(mouse::cursor(layout, layout.mouse.panel_frame, still.mouse));
        } else if let Some((point, p)) = &pointer {
            list.push(p.cursor(layout, *point));
        }
        let stem = Path::new(&frame.file).with_extension("");
        texts.insert(
            stem.to_string_lossy().into_owned(),
            serde_json::Value::String(text.to_owned()),
        );
        lists.push((frame, list));
    }
    let mut names = std::collections::BTreeSet::new();
    for (_, list) in &lists {
        names.extend(batch::names(list, layout));
    }
    let mut batch = Batch::default();
    for name in &names {
        // A clip from the video archives (E17 M5 T4b2b), every other asset from the LODs.
        let bytes = match interface.videos {
            Some(videos) if name.ends_with(".BIK") => videos.read(name)?,
            _ => lookup(archives, name)?,
        };
        batch.add(name, &bytes).map_err(|e| e.to_string())?;
    }
    for (frame, list) in &lists {
        // Compared with the original's captures: its drawing bugs reproduced (E17 M3 T4a).
        let screen = compose(list, &batch, layout, Bugs::Original).map_err(|e| e.to_string())?;
        let png = here
            .join("ours")
            .join(Path::new(&frame.file).with_extension("png"));
        write(&png, &h3_render::png::encode(&screen))?;
    }
    write(&here.join("ours.txt"), drawn.as_bytes())?;
    let json = serde_json::to_string(&texts).map_err(|e| e.to_string())?;
    write(&here.join("ours-log.json"), json.as_bytes())?;
    Ok(lists.len())
}

/// A moment as one line: `<t> pulse <n> outline <id|-> [projectile <def> f<frame> <x>,<y>]`, then
/// each stack's `<id>:g<group>f<frame>h<hex>[m][+<x>,<y>]`, `m` when it faces left (the harness
/// places a count box by the facing, M3 T1a).
pub(crate) fn describe(t: u32, moment: &scene::Moment) -> String {
    let mut line = format!("{t} pulse {} outline ", moment.pulse);
    match moment.active {
        Some(id) => line.push_str(&id.to_string()),
        None => line.push('-'),
    }
    if let Some(p) = &moment.projectile {
        let _ = write!(
            line,
            " projectile {} f{} {},{}",
            p.def, p.frame, p.at[0], p.at[1]
        );
    }
    // The heroes (E17 M3 T3a), in a shape the harness's pose pattern skips.
    for (side, hero) in moment.heroes.iter().enumerate() {
        if let Some(h) = hero {
            let _ = write!(line, " hero{side} g{}f{} flag{}", h.group, h.frame, h.flag);
        }
    }
    for (id, p) in &moment.poses {
        let _ = write!(line, " {id}:g{}f{}h{}", p.group, p.frame, p.hex);
        if p.mirrored {
            line.push('m');
        }
        if p.offset != [0, 0] {
            let _ = write!(line, "+{},{}", p.offset[0], p.offset[1]);
        }
    }
    line.push('\n');
    line
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
            "melee", "--speed", "1", "--out", "/o", "--packs", "/p",
        ]))
        .unwrap();
        assert_eq!(o.only.as_deref(), Some("melee"));
        assert_eq!(o.speed, Speed::new(1));
        assert_eq!((o.out, o.packs), (PathBuf::from("/o"), PathBuf::from("/p")));
        let o = Options::parse(&args(&["--install", "/i", "--frames", "/f"])).unwrap();
        assert_eq!((o.only, o.install), (None, PathBuf::from("/i")));
        assert_eq!(o.frames, Some(PathBuf::from("/f")));
        assert!(o.out.ends_with("out/sequences"));
        for bad in [
            &["--speed"][..],
            &["--speed", "4"],
            &["--what"],
            &["a", "b"],
        ] {
            assert!(Options::parse(&args(bad)).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn the_sequences_file_names_engine_commands() {
        let path = assets::default_dir().join("../situations/sequences.toml");
        let file: File = toml::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        let melee = file.sequence.iter().find(|s| s.id == "melee").unwrap();
        assert_eq!(melee.command.len(), 2);
        assert_eq!(melee.command[1].seat, Seat::Defender);
        assert_eq!(
            melee.command[1].command,
            Command::Attack {
                target: 0,
                from: 45
            }
        );
        assert_eq!((melee.pre, melee.ms), (0, 7000));
        assert_eq!((melee.until(), melee.command[0].at), (7000, None));
    }

    #[test]
    fn a_battle_in_progress_counts_from_its_last_actions() {
        let s: Sequence = toml::from_str(
            r#"
            id = "x"
            scenario = "x.toml"
            timeline = "x.txt"
            pre = 100
            ms = 500
            actions = ["hex:3", "turn", "key:w"]
            command = [
                { seat = "Attacker", command = { Wait = {} } },
                { seat = "Defender", at = 4500, command = { Defend = {} } },
                { seat = "Defender", at = 2000, command = { Defend = {} } },
            ]
            "#,
        )
        .unwrap();
        assert_eq!(s.until(), 5000);
        assert_eq!((s.command[0].at, s.command[2].at), (None, Some(2000)));
    }
}
