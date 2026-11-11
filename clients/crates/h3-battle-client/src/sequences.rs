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
use h3_protocol::wire::{Command, Seat, Side};
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

/// When a stats dialog's creature first steps after the dialog opened (E17 M6 T2a1: its modal run
/// sets the frame deadline 100 ms after the opening, 0x5F4801).
const STATS_FIRST_STEP: u32 = 100;

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
    /// Recorded with the original playing its sounds (E17 M5 T5b2, the harness's `sound`): the
    /// battle waits for them, so the viewer's timeline does too (the samples' lengths from the
    /// installation's sound archives).
    #[serde(default)]
    sound: bool,
    /// The grid's and the shadows' options it was recorded at (E17 M6 T4c, *provisional*: the
    /// harness writes them into the original's registry; 0 without them).
    #[serde(flatten)]
    shades: Shadows,
    /// The Auto Combat boxes it was recorded at (E17 M6 T5b1, *provisional*).
    #[serde(flatten)]
    auto: AutoBoxes,
}

/// The Auto Combat boxes a sequence was recorded at (E17 M6 T5b1, design §12.18, *provisional*):
/// the harness writes them into the original's registry before the game starts (`h3drive.AUTO`:
/// `Combat Auto Creatures`, `Combat Auto Spells`), each 0 or 1; without them `setup.sh`'s,
/// Creatures 1 and Spells 0 (`Settings::CAPTURE`). The machines' boxes stay clear.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub(crate) struct AutoBoxes {
    /// Combat Auto Creatures (1 without it).
    #[serde(default)]
    auto_creatures: Option<u8>,
    /// Combat Auto Spells (0 without it): the computer's casts for the human's hero, which the
    /// record's commands hold (each without a time, the computer's).
    #[serde(default)]
    auto_spells: u8,
}

impl AutoBoxes {
    /// The boxes (`what` names the sequence in an error).
    pub(crate) fn boxes(self, what: &str) -> Result<[bool; 5], String> {
        let flag = |name: &str, v: u8| match v {
            0 | 1 => Ok(v == 1),
            _ => Err(format!("{what}: {name} is 0 or 1")),
        };
        let mut boxes = h3_view::options::Settings::CAPTURE.auto;
        if let Some(v) = self.auto_creatures {
            boxes[h3_view::auto::CREATURES] = flag("auto_creatures", v)?;
        }
        boxes[h3_view::auto::SPELLS] = flag("auto_spells", self.auto_spells)?;
        Ok(boxes)
    }
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
/// which acts as soon as its turn comes. E17 M6 T5b2a (design §12.19, *provisional*): with
/// `question`, the computer's command for a human's stack under Auto Combat that came after its
/// question (and the offer), the battle standing still behind the boxes from the action's
/// millisecond; its `at` the answer that gave it.
#[derive(Debug, Deserialize)]
struct Step {
    seat: Seat,
    command: Command,
    #[serde(default)]
    at: Option<u32>,
    #[serde(default)]
    question: bool,
}

/// The commands that came after Auto Combat's question (E17 M6 T5b2a): each by its index with
/// its answer's time, which a step marked `question` must give.
fn holds(steps: &[Step]) -> Result<Vec<(usize, u32)>, String> {
    (steps.iter().enumerate())
        .filter(|(_, s)| s.question)
        .map(|(i, s)| {
            s.at.map(|at| (i, at))
                .ok_or_else(|| format!("command {}: a question's command needs its `at`", i + 1))
        })
        .collect()
}

/// Where Auto Combat's questions were open (E17 M6 T5b2a): each held command's from the
/// action's millisecond (the command before's end in `timeline`, `pre` for the first; E17 M6
/// T5b2b, recorded: the switch's when it came later, on a human's turn waiting,
/// `auto-retreat-human`) to its answer; `actions` the sequence's.
fn asked_spans(
    holds: &[(usize, u32)],
    timeline: &Timeline,
    (pre, actions): (u32, &[String]),
) -> Vec<(u32, u32)> {
    holds
        .iter()
        .map(|&(i, until)| {
            let from = i
                .checked_sub(1)
                .and_then(|k| timeline.actions.get(k))
                .map_or(pre, |a| a.end);
            (from.max(switched(actions, pre, until)), until)
        })
        .collect()
}

/// The time of the last `key:a` of `actions` at or before `until`, the switch of Auto Combat on
/// that the question came after (E17 M6 T5b2b), by the times their `at:` and `step:` reach from
/// `pre`; `pre` without one. (A `turn:` waits for a turn's start, which a box's opening takes
/// anyway: [`asked_spans`].) *Provisional* (E17 M6 T7b, R10-3): any `key:a` counts, a switch off
/// or a stop too, and one pressed while the box is open (which the box takes) moves its span
/// and its hold to that key: *not modelled*, a sequence presses none of them before its answer
/// (`pointer::answering` reads such a key as a stop too).
fn switched(actions: &[String], pre: u32, until: u32) -> u32 {
    let (mut t, mut last) = (pre, pre);
    for a in actions {
        let (kind, arg) = a.split_once(':').unwrap_or((a, ""));
        match kind {
            "at" => t = arg.parse().unwrap_or(t),
            "step" => t = t.saturating_add(arg.parse().unwrap_or(0)),
            "key" if pointer::auto_combat(arg) && t <= until => last = t,
            _ => {}
        }
    }
    last
}

/// Whether the video archives `videos` hold the result window's clip of `tune` (E17 M5 T4b2b).
fn held(videos: &crate::videos::Videos, tune: h3_view::sound::Tune) -> bool {
    let clip = h3_view::clip::of(tune);
    std::iter::once(clip.start)
        .chain(clip.looped)
        .all(|name| videos.has(name))
}

/// The times the result window and then each box of Eagle Eye's message after it closed (E17 M6
/// T5c): the `answers` from the battle's `end` on, but those that answered one of Auto Combat's
/// questions open over `spans` (a leaving given at the end's millisecond); a `key:Escape` closes
/// the window (its OK's key) but no box of Eagle Eye's (style 1's OK takes Enter alone,
/// `message_box_build` 0x4F6990; the reviewer).
fn closes(answers: &[(u32, bool)], spans: &[(u32, u32)], end: Option<u32>) -> Vec<u32> {
    let Some(end) = end else {
        return Vec::new();
    };
    let mut out: Vec<u32> = Vec::new();
    for &(t, escape) in answers {
        let asked = spans
            .iter()
            .any(|&(from, until)| (from..=until).contains(&t));
        if t >= end && !asked && !(escape && !out.is_empty()) {
            out.push(t);
        }
    }
    out
}

/// The boxes Auto Combat's questions showed (E17 M6 T5b2a): over each span of `spans`, the
/// question until the first of `answers` in it, then, after a second one, the offer until it;
/// each `[from, until)` with the index of its held command and whether it is the offer.
fn asked_boxes(
    spans: &[(u32, u32)],
    holds: &[(usize, u32)],
    answers: &[u32],
) -> Result<Vec<(u32, u32, usize, bool)>, String> {
    let mut out = Vec::new();
    for (&(from, until), &(i, _)) in spans.iter().zip(holds) {
        let mut given = answers.iter().filter(|&&t| (from..=until).contains(&t));
        let first = *given
            .next()
            .ok_or_else(|| format!("command {}: its question has no answer", i + 1))?;
        out.push((from, first, i, false));
        if let Some(&second) = given.next() {
            out.push((first, second, i, true));
        }
    }
    Ok(out)
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
    /// Creature Info's level a situation runs at (E17 M6 T4b2b1, *provisional*: the harness
    /// writes it into the original's registry as `Combat Army Info Level` before the run; 0, the
    /// harness's `Settings::CAPTURE`, without it).
    #[serde(default)]
    army_info: u8,
    /// The grid's and the shadows' options a situation runs at (E17 M6 T4c, *provisional*: the
    /// harness writes them into the original's registry as `Show Combat Grid`, `Show Combat Mouse
    /// Hex` and `Combat Shade Level` before the run; 0, the harness's, without them).
    #[serde(flatten)]
    shades: Shadows,
}

/// The grid's and the shadows' options a situation or a sequence runs at (E17 M6 T4c): each 0 or
/// 1, 0 without it.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub(crate) struct Shadows {
    /// Show Combat Grid.
    #[serde(default)]
    grid: u8,
    /// Show Combat Mouse Hex.
    #[serde(default)]
    cursor_shadow: u8,
    /// Combat Shade Level.
    #[serde(default)]
    movement_shadow: u8,
}

impl Shadows {
    /// The options (`what` names the file in an error).
    pub(crate) fn options(self, what: &str) -> Result<h3_view::shades::Options, String> {
        let flag = |name: &str, v: u8| match v {
            0 | 1 => Ok(v == 1),
            _ => Err(format!("{what}: {name} is 0 or 1")),
        };
        Ok(h3_view::shades::Options {
            grid: flag("grid", self.grid)?,
            cursor: flag("cursor_shadow", self.cursor_shadow)?,
            movement: flag("movement_shadow", self.movement_shadow)?,
        })
    }
}

/// A situation's inputs as the harness kept them (E17 M5 T3a; M6 T4b2b1).
pub(crate) struct Situation {
    /// Where the harness left the mouse at the opening.
    pub mouse: Option<[i32; 2]>,
    /// The actions after the mouse's move.
    pub actions: Vec<String>,
    /// Creature Info's level it runs at (0 to 2).
    pub army_info: u8,
    /// The grid's and the shadows' options it runs at (E17 M6 T4c).
    pub shades: h3_view::shades::Options,
}

/// A situation's mouse, actions and Creature Info's level as the harness kept them in `here`'s
/// `frames.toml` (E17 M5 T3a; the level E17 M6 T4b2b1).
pub(crate) fn situation_inputs(here: &Path) -> Result<Situation, String> {
    let path = here.join("frames.toml");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let kept: Frames = toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    if kept.army_info > 2 {
        return Err(format!("{}: army_info is 0, 1 or 2", path.display()));
    }
    let shades = kept.shades.options(&path.display().to_string())?;
    Ok(Situation {
        mouse: kept.mouse,
        actions: kept.actions,
        army_info: kept.army_info,
        shades,
    })
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
    /// Auto Combat's questions and offers a sequence's battle showed (E17 M6 T5b2a,
    /// *provisional*): each drawn over the battle from its first time to before its second, the
    /// box's cursor at the mouse.
    pub asked: &'a [(u32, u32, h3_view::message::MessageBox)],
    /// The times a sequence's actions closed the result window and then each box of Eagle Eye's
    /// message after it (E17 M6 T5c, design §12.21), in order: from the first on the boxes of
    /// `eagle_eye` follow one another over the screen darkened, the default cursor's frame 0 at
    /// the mouse; after the last the darkened screen alone (*provisional*: the original's
    /// adventure map is the map client's).
    pub closes: &'a [u32],
    /// The boxes of Eagle Eye's message after the result window (E17 M6 T5c).
    pub eagle_eye: &'a [h3_view::message::MessageBox],
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
        let mut casts = motion::casts(
            &record,
            &tables,
            &archives,
            &options.packs,
            (false, Bugs::Original),
        )?;
        // The frame check draws Chain Lightning's rays from the original's draws (E17 M3 T7c-3).
        if let Some(dir) = &options.frames {
            casts.rays = kept_rays(&dir.join(&sequence.id))?;
        }
        // The original's drawing bugs, as its captures show them (E17 M3 T7d, CLT-Q8).
        let lengths = if sequence.sound {
            let sounds = (
                crate::battle::stack_sounds(&record, &tables),
                crate::battle::battle_sounds(&record, &tables),
            );
            let mut lines = Vec::new();
            // A sound the archives lack is waited for as the original would: not at all.
            Some(crate::audio::lengths(&options.install, sounds, &mut lines)?)
        } else {
            None
        };
        let stage = anim::Stage {
            bugs: Bugs::Original,
            lengths: lengths.as_ref(),
            ..motion::stage(&record, &tables, &layout, (&magic, &casts, siege.as_ref()))
        };
        // The harness acts `pre` ms after the opening, and then at each command's `at`; a
        // command after Auto Combat's question at its answer, the battle held until then (E17
        // M6 T5b2a).
        let holds = holds(&sequence.command).map_err(|e| format!("{}: {e}", sequence.id))?;
        // Each command's events, the battle's start's left out (protocol v38: the snapshot shows
        // it already).
        let commands: Vec<anim::Command<'_>> = record
            .commands()
            .enumerate()
            .map(|(i, events)| anim::Command {
                at: match sequence.command.get(i) {
                    Some(step) if step.question => None,
                    step => step.and_then(|s| s.at).or((i == 0).then_some(sequence.pre)),
                },
                events,
            })
            .collect();
        let snapshot = record
            .first_snapshot()
            .ok_or("the record holds no snapshot")?;
        // Each held from its box's opening: the first command's at `pre` (the switch's), a
        // later one's at the end of the command before (0: at least there) or the switch's
        // after it (E17 M6 T5b2b, recorded: `auto-retreat-human`'s A on the Dwarves' turn).
        let held: Vec<(usize, u32, u32)> = (holds.iter())
            .map(|&(i, until)| {
                let from = if i == 0 { sequence.pre } else { 0 };
                let switch = switched(&sequence.actions, sequence.pre, until);
                (i, from.max(switch), until)
            })
            .collect();
        let mut timeline = anim::held(snapshot, (&commands, &held), &looks, stage, options.speed)
            .map_err(|e| format!("{}: {e}", sequence.id))?
            .with_heroes(motion::heroes(&record, &tables, &archives, None)?);
        let spans = asked_spans(&holds, &timeline, (sequence.pre, &sequence.actions));

        // The mouse as the harness moved it (E17 M3 T6c-2): its cursor, the stack it marks and
        // enters, the status line's hints, each turn's battle from the engine.
        let scenario_path = dir.join(&sequence.scenario);
        // The heroes' casts in order, which the `cast:` actions aim (E17 M3 T7b-1).
        let spells: Vec<u16> = record
            .events()
            .filter_map(|e| match *e {
                h3_protocol::wire::Event::SpellCast { spell, .. } => Some(spell),
                _ => None,
            })
            .collect();
        // A spell by its number in the original (an `aim:` action's, E17 M5 T3b).
        let keys = &record.header.manifest.keys.spells;
        let handle =
            |original: u16| motion::spell_numbered_in(keys, &tables, original).map(|(h, _)| h);
        // The Auto Combat boxes it was recorded at (E17 M6 T5b1).
        let boxes = sequence.auto.boxes(&sequence.id)?;
        // Which turns play themselves under Auto Combat by them (E17 M6 T5b2b).
        let plays = pointer::plays_itself(&record, &timeline, (&tables, boxes));
        // From the battle's end on a box is open too: the result window, then Eagle Eye's message
        // (E17 M6 T5c), the window closed by a `key:Return`, a `key:Escape` or a `click:`, a box
        // by a `key:Return` or a `click:` (a click trusted to be on its OK, *provisional*; [`closes`]),
        // which then gives nothing to the battle.
        let open: Vec<(u32, u32)> = (spans.iter().copied())
            .chain(timeline.ended.map(|(end, _)| (end, u32::MAX)))
            .collect();
        let (inputs, answers) = pointer::answering(
            &sequence.actions,
            sequence.pre,
            &layout,
            (&timeline, &spells, &handle),
            (
                h3_view::tactics::told(snapshot),
                sequence.shades.options(&sequence.id)?,
                (boxes[h3_view::auto::CREATURES], &plays),
            ),
            &open,
        )
        .map_err(|e| format!("{}: {e}", sequence.id))?;
        let closes = closes(&answers, &spans, timeline.ended.map(|(end, _)| end));
        let answers: Vec<u32> = answers.iter().map(|&(t, _)| t).collect();
        let asked =
            asked_boxes(&spans, &holds, &answers).map_err(|e| format!("{}: {e}", sequence.id))?;
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
            (&tables, Bugs::Original, boxes),
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
                ruleset: Some(&ruleset),
                catalogue: &catalogue,
                layout: &layout,
                // The comparisons draw the original's (CLT-Q35).
                bugs: Bugs::Original,
            };
            let result = outcome.as_ref().map(|o| {
                crate::results::window(
                    &sources,
                    snapshot,
                    o,
                    crate::render::viewing_player(snapshot),
                )
            });
            // Auto Combat as the actions switched it (E17 M5 T4c1), the stacks playing
            // themselves by the harness's boxes and the original's button (E17 M6 T4b1), the
            // sequence's own (E17 M6 T5b1).
            let itself = pointer::captured(&record, &timeline, &turned, (&tables, boxes));
            let panels = log::panels(
                &record,
                (&shown.log, &timeline.controllers),
                &tables,
                &layout,
                (&archives, &exe),
                Bugs::Original,
            )?
            .with_auto(
                snapshot,
                pointer::auto(&inputs, (&timeline, pointer::human_sides(&record)), &itself),
                Bugs::Original,
            );
            // Auto Combat's questions and offers (E17 M6 T5b2a): the side's own view before the
            // held command (its hero's name, the price), laid out in the original's fonts.
            let writer = log::Writer::new(&tables, &layout, (&archives, &exe), Bugs::Original)?;
            let gold = layout.message.resources.clone();
            let gold_size = lookup(&archives, &gold)
                .ok()
                .and_then(|b| h3_assets::def::parse_def(&b).ok())
                .map_or([0, 0], |d| {
                    [d.width, d.height].map(|v| i32::try_from(v).unwrap_or(i32::MAX))
                });
            let mut questions = Vec::new();
            for &(from, until, i, offer) in &asked {
                let own = scenario::own_view(&scenario_path, &options.packs, sequence.seed, {
                    played.get(..i).unwrap_or_default()
                })
                .map_err(|e| format!("{}: {e}", sequence.id))?
                .ok_or_else(|| format!("{}: no question once the battle is over", sequence.id))?;
                let question = match own.own_hero.as_ref().and_then(|h| h.surrender_price) {
                    Some(price) if offer => crate::live::Question::Offer { price },
                    _ => crate::live::Question::Retreat,
                };
                let keys = &record.header.manifest.keys.hero_types;
                let (text, picture, _) =
                    crate::boxes::question(question, &own, keys, &catalogue, (&gold, gold_size));
                let player = crate::render::viewing_player(snapshot);
                let style = h3_view::message::Style::OkCancel;
                if let Some(message) =
                    writer.message(&layout, &text, style, picture.as_ref(), player)
                {
                    questions.push((from, until, message));
                }
            }
            // Eagle Eye's message after the result window (E17 M6 T5c), laid out as the window.
            let spell_pictures = layout.message.spell_pictures.clone();
            let spells_size = lookup(&archives, &spell_pictures)
                .ok()
                .and_then(|b| h3_assets::def::parse_def(&b).ok())
                .map_or([0, 0], |d| {
                    [d.width, d.height].map(|v| i32::try_from(v).unwrap_or(i32::MAX))
                });
            let eagle_eye: Vec<h3_view::message::MessageBox> = outcome
                .as_ref()
                .map(|o| {
                    let pictures = (spell_pictures.as_str(), spells_size);
                    crate::results::eagle_eye(&sources, snapshot, o, pictures)
                })
                .unwrap_or_default()
                .iter()
                .filter_map(|t| {
                    let player = crate::render::viewing_player(snapshot);
                    writer.pictures(&layout, &t.text, &t.pictures, player)
                })
                .collect();
            let interface = Interface {
                panels: &panels,
                legal: &Legal::default(),
                shown: Some(&shown),
                walls: None,
                book: None,
                result: result.as_ref(),
                videos: Some(&videos),
                asked: &questions,
                closes: &closes,
                eagle_eye: &eagle_eye,
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

/// Where a still's battle's mouse rests and its pointer there over `snapshot` with `legal`: the
/// situation's mouse where the harness left it, at the opening (E17 M3 T6c-1; `kept`), or where
/// the actions of `book` moved it over the battle (E17 M5 T4c2a), a stats dialog's press where it
/// was made (E17 M6 T2a1); none while the tactics phase's message left the box's cursor
/// (CLT-Q32). Its status line at the level the battle last wrote it at (E17 M6 T4b2b2: a strike's
/// or a shot's damage at levels 1 and 2, the epic's review item 42; a sequence's 0). A sequence's
/// mouse goes over time (T6c-2: `Interface::shown`).
pub(crate) fn resting(
    snapshot: &h3_protocol::wire::Observation,
    (legal, book): (&Legal, Option<&crate::spellbook::Still>),
    layout: &BattleLayout,
    kept: Option<[i32; 2]>,
) -> Option<([i32; 2], mouse::Pointer)> {
    let arrow = book.is_some_and(|b| b.arrow);
    let rest = book
        .and_then(|b| b.stats.as_ref().map(|s| s.looked).or(b.over))
        .or(kept);
    let level = book.map_or(0, |b| b.line_level);
    rest.filter(|_| !arrow).map(|point| {
        let pointer = mouse::pointer(snapshot, legal, layout, (point, Bugs::Original, level));
        (point, pointer)
    })
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
    let pointer = resting(
        snapshot,
        (interface.legal, interface.book),
        layout,
        kept.mouse,
    );
    let status = match &pointer {
        Some((_, p)) => {
            let catalogue = log::catalogue(tables, archives, exe)?;
            Some(pointer::hint_text(record, &catalogue, &p.hint, snapshot))
        }
        None => None,
    };
    // The frames' times are read forward (E17 M5 T1a; one earlier than the last starts over).
    let mut head = Playhead::new(timeline);
    // The battle behind the result window, its moment before the end read once (E17 M5 T8b2,
    // R5b-9), as the window's (`h3_view::result::Behind`).
    let mut behind = h3_view::result::Behind::default();
    for frame in &kept.frame {
        // While the spell book is open the battle behind it stands as it was when the book opened
        // (E17 M5 T3a: the dialog's modal loop runs no idle step; the original's frames are the
        // same 100 ms and a second after); while the mouse holds a panel button too (E17 M7 T2:
        // the button's own loop runs none either).
        let at = interface
            .book
            .filter(|b| b.stands())
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
            behind.show(&mut moment, end, |t| scene::Moment::at(timeline, t));
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
        // The movement shadow outside the tactics phase, the grid and the cursor shadow (E17 M6
        // T4c, design §12.16): a sequence's as its mouse's follow has them, a situation's as its
        // actions left them over the opening's turn; behind the tactics phase's message the grid
        // alone.
        let shades = match (interface.shown, interface.book) {
            (Some(shown), _) => shown.shades(frame.t).cloned(),
            (None, Some(still)) if opening => Some(h3_view::shades::Drawn {
                shade: std::collections::BTreeSet::new(),
                overlays: scene::Overlays {
                    grid: still.cells.options().grid,
                    ..scene::Overlays::default()
                },
            }),
            (None, Some(still)) => {
                let field = mouse::Field {
                    observation: snapshot,
                    legal: interface.legal,
                    facing: &mouse::Facing::Sides,
                    bugs: Bugs::Original,
                };
                let reach = if snapshot.tactics.is_none() {
                    h3_view::shades::reach(&field)
                } else {
                    std::collections::BTreeSet::new()
                };
                Some(still.cells.drawn(&reach))
            }
            (None, None) => None,
        };
        if let Some(shades) = shades {
            moment.shade.extend(shades.shade);
            moment.overlays = shades.overlays;
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

        let mut list =
            scene::battle_as(snapshot, &moment, manifest, tables, layout, Bugs::Original)
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
            list.extend(panels.before(snapshot, layout));
        } else {
            // A situation's panel button its hotkey holds (E17 M7 T1c), drawn pressed.
            let pressed = interface.book.and_then(|b| b.pressed);
            list.extend(panels.at_pressed(snapshot, &timeline.turns, at, layout, text, pressed));
        }
        // The army info windows a situation's battle mouse left shown (E17 M6 T4b2b1, design
        // §12.15): the battle's redraw ends with them, under every dialog.
        for window in interface.book.map_or(&[][..], |b| &b.army) {
            let draws = window.dialog.draws(&window.shown, &layout.player_palette);
            list.extend(draws.into_iter().map(scene::Draw::Dialog));
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
        // The combat options dialog the actions left open (E17 M6 T4a), with its shadow, under
        // its boxes.
        let options = interface.book.and_then(|b| b.options.as_ref());
        if let Some(open) = options {
            let window = &open.window;
            list.extend(
                window
                    .dialog
                    .draws(&window.shown, &layout.player_palette)
                    .into_iter()
                    .map(scene::Draw::Dialog),
            );
        }
        // Its arrow, the cursor type 0 frame 0 (crdeflt.def's), while it is open or after its key
        // closed it, the mouse not moved since (oddity 10).
        let options_arrow = options.is_some() || interface.book.is_some_and(|b| b.options_arrow);
        // A message box the actions left open (E17 M5 T4a), over the book and the battle, the
        // default cursor's frame 0 at the mouse (0x4F74AC).
        let message = interface
            .book
            .and_then(|b| Some((b.message.as_ref()?, b.mouse)));
        // The result window (E17 M5 T4b1): a still's at once, a sequence's from its battle's end
        // until the actions close it (E17 M6 T5c); over the whole screen darkened, the combat
        // cursor's pointer at the mouse. With it the time it opened, for its clip (E17 M5 T4b2b).
        let closed = interface.closes.partition_point(|&c| c <= frame.t);
        let point = || {
            interface
                .shown
                .and_then(|shown| frame.cursor.or_else(|| shown.point(frame.t)))
                .or_else(|| pointer.as_ref().map(|(p, _)| *p))
        };
        let ended = interface
            .result
            .zip(timeline.ended.filter(|&(t, _)| frame.t >= t));
        let result = match interface
            .book
            .and_then(|b| Some((b.result.as_ref()?, b.mouse, b.at)))
        {
            Some((window, mouse, at)) => Some((window, Some(mouse), at)),
            None => ended
                .filter(|_| closed == 0)
                .map(|(window, (end, _))| (window, point(), end)),
        };
        // Eagle Eye's message after the window closed (E17 M6 T5c, design §12.21): over the
        // screen still darkened, a still's box as its actions left it, a sequence's by the
        // actions' closes; the default cursor's frame 0 at the mouse (`message_box_build`'s).
        // Under a box the window stays as it last showed (recorded, `eagle-eye`: nothing redraws
        // the screen before the box), its OK up, its clip's frame of its close's millisecond
        // (`eagle-eye-one`: a decode in that millisecond came first; a still's at once: its
        // first).
        let darkened = match interface.book.filter(|b| b.darkened) {
            Some(b) => Some(
                (b.message.as_ref()).map(|m| (m, Some(b.mouse), b.window.as_ref().map(|w| (w, 0)))),
            ),
            None => ended.filter(|_| closed > 0).map(|(window, (end, _))| {
                let first = interface.closes.first().copied().unwrap_or(end);
                let since = first.saturating_sub(end);
                (interface.eagle_eye.get(closed - 1)).map(|m| (m, point(), Some((window, since))))
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
            if interface.videos.is_some_and(|v| held(v, *tune)) {
                let since = frame.t.saturating_sub(opened);
                list.push(h3_view::result::clip(&layout.result, *tune, since));
            }
            if let Some(point) = mouse {
                list.push(mouse::cursor(layout, h3_view::result::CURSOR_FRAME, point));
            }
        } else if let Some(shown) = darkened {
            list.push(h3_view::result::backdrop(layout.frame));
            if let Some((message, point, behind)) = shown {
                if let Some(((window, tune), since)) = behind {
                    let window = crate::results::released(window);
                    list.extend(
                        (window.dialog.draws(&window.shown, &layout.player_palette))
                            .into_iter()
                            .map(scene::Draw::Dialog),
                    );
                    if interface.videos.is_some_and(|v| held(v, *tune)) {
                        list.push(h3_view::result::clip(&layout.result, *tune, since));
                    }
                }
                list.extend(
                    message
                        .dialog
                        .draws(&message.shown, &layout.player_palette)
                        .into_iter()
                        .map(scene::Draw::Dialog),
                );
                list.extend(point.map(|[x, y]| scene::Draw::Cursor {
                    def: layout.message.cursor.clone(),
                    frame: 0,
                    x,
                    y,
                    hot: layout.book.hot.first().copied().unwrap_or_default(),
                }));
            }
        } else if let Some((_, _, question)) =
            (interface.asked.iter()).find(|(from, until, _)| (*from..*until).contains(&frame.t))
        {
            // Auto Combat's question or offer (E17 M6 T5b2a): the box's cursor at the mouse.
            list.extend(
                question
                    .dialog
                    .draws(&question.shown, &layout.player_palette)
                    .into_iter()
                    .map(scene::Draw::Dialog),
            );
            let point = interface
                .shown
                .and_then(|shown| frame.cursor.or_else(|| shown.point(frame.t)));
            list.extend(point.map(|[x, y]| scene::Draw::Cursor {
                def: layout.message.cursor.clone(),
                frame: 0,
                x,
                y,
                hot: layout.book.hot.first().copied().unwrap_or_default(),
            }));
        } else if let Some((message, mouse)) = message {
            list.extend(
                message
                    .dialog
                    .draws(&message.shown, &layout.player_palette)
                    .into_iter()
                    .map(scene::Draw::Dialog),
            );
            // The right button's popup held hides it (E17 M6 T2b2).
            let held = interface.book.is_some_and(|b| b.held);
            list.extend((!held).then(|| scene::Draw::Cursor {
                def: layout.message.cursor.clone(),
                frame: 0,
                x: mouse[0],
                y: mouse[1],
                hot: layout.book.hot.first().copied().unwrap_or_default(),
            }));
        } else if let Some((open, mouse, opened)) = interface
            .book
            .and_then(|b| Some((b.stats.as_ref()?, b.mouse, b.at)))
        {
            // A stack's stats dialog (E17 M6 T2a1, design §12.1): its creature stepped since it
            // opened; the cursor its moves set (T2a2a), until then the combat cursor's pointer
            // the press set, the panel's status line keeping the press's hint.
            let window = open.at(STATS_FIRST_STEP, frame.t.saturating_sub(opened));
            list.extend(
                window
                    .dialog
                    .draws(&window.shown, &layout.player_palette)
                    .into_iter()
                    .map(scene::Draw::Dialog),
            );
            // The right button's description held over it, the cursor hidden (E17 M6 T2a2b1);
            // the right button's dialog over the battle held, the cursor hidden (E17 M6 T2b1).
            match &open.modal.popup {
                Some(popup) => list.extend(
                    popup
                        .dialog
                        .draws(&popup.shown, &layout.player_palette)
                        .into_iter()
                        .map(scene::Draw::Dialog),
                ),
                None if open.held => {}
                None => list.push(crate::stats::cursor(layout, &open.modal, mouse)),
            }
        } else if interface.book.is_some_and(|b| b.held) {
            // The right button held over a hero whose window shows nothing (CLT-Q50, the
            // original's: E17 M6 T4b2b1): no cursor (`popup_run_held`).
        } else if let Some(still) = interface.book.filter(|_| options_arrow) {
            list.push(scene::Draw::Cursor {
                def: layout.message.cursor.clone(),
                frame: 0,
                x: still.mouse[0],
                y: still.mouse[1],
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
    // The original's clips' reading too (E17 M6 T6c1, CLT-Q68).
    let mut batch = Batch::with_bugs(Bugs::Original);
    for name in &names {
        // A clip from the video archives (E17 M5 T4b2b), every other asset from the LODs.
        let clip = name.ends_with(".BIK");
        let bytes = match interface.videos {
            Some(videos) if clip => videos.read(name)?,
            _ => lookup(archives, name)?,
        };
        // A clip that does not decode fails the comparison (E17 M5 T8b2, R2b's leftover): the
        // original showed it, so a bare slot would only differ in pixels without saying why
        // (the window leaves the slot bare, T8b1).
        batch.add(name, &bytes).map_err(|e| {
            let e = crate::connect::printable(&e.to_string());
            if clip {
                format!("the result window's clip, which a comparison needs: {e}")
            } else {
                e
            }
        })?;
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
pub fn describe(t: u32, moment: &scene::Moment) -> String {
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
        // E17 M6 T5b1: every entry's Auto Combat boxes good; the two recorded with Spells set,
        // `auto-cast-human` with Creatures cleared.
        for s in &file.sequence {
            assert!(s.auto.boxes(&s.id).is_ok(), "{}", s.id);
        }
        let boxes = |id: &str| {
            let s = file.sequence.iter().find(|s| s.id == id).unwrap();
            s.auto.boxes(id).unwrap()
        };
        assert_eq!(boxes("auto-cast"), [true, true, false, false, false]);
        assert_eq!(boxes("auto-cast-human"), [false, true, false, false, false]);
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
        // E17 M6 T5b1: the harness's boxes without its own, Creatures alone.
        let capture = h3_view::options::Settings::CAPTURE.auto;
        assert_eq!(s.auto.boxes("x"), Ok(capture));
    }

    #[test]
    fn the_sequences_to_record_parse_with_their_boxes() {
        // E17 M6 T5b1: the entries prepared for a recording (`to-record.toml`, read by the
        // harness only until each moves into `sequences.toml`) parse as sequences, their boxes
        // and scenarios good.
        #[derive(Deserialize)]
        struct Pending {
            #[serde(default)]
            sequence: Vec<Sequence>,
        }
        let dir = assets::default_dir().join("../situations");
        let Ok(text) = fs::read_to_string(dir.join("to-record.toml")) else {
            return;
        };
        let pending: Pending = toml::from_str(&text).unwrap();
        for s in &pending.sequence {
            assert!(s.auto.boxes(&s.id).is_ok(), "{}", s.id);
            assert!(dir.join(&s.scenario).exists(), "{}", s.id);
        }
    }

    #[test]
    fn a_command_after_auto_combat_s_question_holds_the_battle_until_its_answer() {
        // E17 M6 T5b2a: a step marked `question` needs its `at`; its box opens at the end of
        // the command before (`pre` for the first) and closes at the first answer in its span,
        // the offer after it until a second one.
        let file: File = toml::from_str(
            r#"[[sequence]]
            id = "x"
            scenario = "x.toml"
            timeline = "x.txt"
            ms = 1
            command = [
                { seat = "Attacker", command = { Defend = {} } },
                { seat = "Defender", command = { Defend = {} } },
                { seat = "Attacker", command = { Retreat = {} }, at = 900, question = true },
            ]"#,
        )
        .unwrap();
        let steps = &file.sequence[0].command;
        let holds = holds(steps).unwrap();
        assert_eq!(holds, [(2, 900)]);
        let missing: File = toml::from_str(
            r#"[[sequence]]
            id = "x"
            scenario = "x.toml"
            timeline = "x.txt"
            ms = 1
            command = [{ seat = "Attacker", command = { Defend = {} }, question = true }]"#,
        )
        .unwrap();
        assert!(holds_of(&missing).is_err());
        let mut timeline = Timeline::default();
        let action = |start, end| h3_view::hero::Action {
            start,
            end,
            ..h3_view::hero::Action::default()
        };
        timeline.actions = vec![action(0, 300), action(300, 450)];
        let spans = asked_spans(&holds, &timeline, (7, &[]));
        assert_eq!(spans, [(450, 900)]);
        assert_eq!(asked_spans(&[(0, 50)], &timeline, (7, &[])), [(7, 50)]);
        // E17 M6 T5b2b: a switch after the command before's end opens the box then; one before
        // it, or after the answer, does not.
        let actions = |a: &[&str]| a.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        let later = actions(&["turn:0.0", "at:600", "key:a", "at:900", "key:Escape"]);
        assert_eq!(asked_spans(&holds, &timeline, (7, &later)), [(600, 900)]);
        let earlier = actions(&["step:93", "key:A", "at:1000", "key:a"]);
        assert_eq!(asked_spans(&holds, &timeline, (7, &earlier)), [(450, 900)]);
        assert_eq!(switched(&earlier, 7, 900), 100);
        // E17 M6 T7b (R10-3): the last `key:a` at or before the answer, also one pressed while
        // the box is open (from 450 here): the span then starts at it (*not modelled*: a
        // sequence presses none there).
        let inside = actions(&["at:100", "key:a", "at:750", "key:a", "at:900", "key:Escape"]);
        assert_eq!(asked_spans(&holds, &timeline, (7, &inside)), [(750, 900)]);
        assert_eq!(switched(&actions(&["key:x:y", "at:z"]), 7, 900), 7);
        assert_eq!(
            asked_boxes(&spans, &holds, &[100, 600, 900]).unwrap(),
            [(450, 600, 2, false), (600, 900, 2, true)]
        );
        assert_eq!(
            asked_boxes(&spans, &holds, &[900]).unwrap(),
            [(450, 900, 2, false)]
        );
        assert!(asked_boxes(&spans, &holds, &[100]).is_err());
        // E17 M6 T5c: the answers from the battle's end on close the result window and then
        // Eagle Eye's boxes, but a question's answer given at the end's millisecond; Escape
        // closes the window, not a box after it (the reviewer).
        let plain = |ts: &[u32]| ts.iter().map(|&t| (t, false)).collect::<Vec<_>>();
        assert_eq!(
            closes(&plain(&[100, 900, 1200, 1500]), &spans, Some(900)),
            [1200, 1500]
        );
        assert_eq!(
            closes(
                &[(1200, true), (1300, true), (1500, false)],
                &spans,
                Some(900)
            ),
            [1200, 1500]
        );
        assert!(closes(&plain(&[100, 1200]), &spans, None).is_empty());
    }

    fn holds_of(file: &File) -> Result<Vec<(usize, u32)>, String> {
        holds(&file.sequence[0].command)
    }

    #[test]
    fn a_sequence_names_the_auto_combat_boxes_it_was_recorded_at() {
        // E17 M6 T5b1 (design §12.18): Creatures cleared, Spells set; a value but 0 or 1 refused.
        let entry = |boxes: &str| {
            let text =
                format!("id = \"x\"\nscenario = \"x.toml\"\ntimeline = \"x.txt\"\nms = 1\n{boxes}");
            toml::from_str::<Sequence>(&text).unwrap()
        };
        let s = entry("auto_creatures = 0\nauto_spells = 1");
        assert_eq!(s.auto.boxes("x"), Ok([false, true, false, false, false]));
        let s = entry("auto_spells = 1");
        assert_eq!(s.auto.boxes("x"), Ok([true, true, false, false, false]));
        assert!(entry("auto_spells = 2").auto.boxes("x").is_err());
        assert!(entry("auto_creatures = 3").auto.boxes("x").is_err());
    }
}
