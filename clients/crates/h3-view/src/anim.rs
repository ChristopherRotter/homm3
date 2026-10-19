//! The scene over time (E17 M2 T4a; `docs/epics/E17-design.md` §7): a record's commands become
//! each stack's animation at the original's timing. [`timeline`] plays the events of each
//! command on the original's clock (the frame deadline on its grid, each group's frame time, the
//! strike's iterations, the shot's frames and projectile, the waits outside the deadline) and
//! keeps every pose a stack takes with the time it is drawn and the time it shows.
//! [`Timeline::at`] is what shows at any time, [`Timeline::lines`] the capture harness's
//! timeline format (design §7.6), which the recorded sequences check.
//!
//! T4b-1 (design §7.9) added the projectile's flights, the idle steps and the outlined stack, and
//! checked every frame of the recorded sequences against the original's screen (the walk's
//! offsets confirmed; the shooter's strike loop and the flight's points corrected).
//!
//! T4b-2a added the turns, the draw layers and the viewer's own fidgets, T5b the sounds' cues.
//!
//! *Provisional* (the M2 review list): the next command's default start, a command without a
//! time counting as the computer's (the idle step and the outline), the flight's cycles rounding.
//! Confirmed by the recorded `fly` (E17 M3 T1b-2): a flier's cycles over every row, a breath
//! told by its second stack's place. Not yet (E17 M3 T1): the spells' sprites, deaths other than
//! by a strike, created stacks (a strike by a stack the timeline does not know plays its
//! reactions alone).

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt::Write as _;

use h3_protocol::wire::{Event, Observation, ServerMessage, Side, Stack};

use crate::hero::{Action, Happening, Heroes};
use crate::layout::BattleLayout;
use crate::ray::{self, Stroke};
use crate::tables::Groups;

/// Every frame outside a walk's group and the shot's: 100 ms times the factor (§7.1).
const TICK: u32 = 100;
/// The wait before a retaliation or a second strike, outside the deadline (§7.4).
const PAUSE: u32 = 150;
/// A projectile's step, outside the deadline (§7.4).
const STEP: u32 = 33;
/// The wait before a ray, outside the deadline (0x43F52F), and a ray's tick (`DrawRay`'s
/// argument, 0x43F58B; its own clock, not the frame deadline: §7.12).
const RAY_WAIT: u32 = 115;
const RAY_TICK: u32 = 10;
/// The idle step's period, whatever the combat speed (0x4960D0, §7.1).
const IDLE_STEP: u32 = 100;
/// The draw layers (+0x8, §4.6, §7.9): standing (0x43D750), walking up and down (0x43F0B0), a
/// melee's striker (0x441CC1) and a struck stack (the harness's reading of memory), the corpses'
/// pass.
const STANDING: u8 = 4;
const WALK_UP: u8 = 3;
const WALK_DOWN: u8 = 7;
const STRIKER: u8 = 6;
const STRUCK: u8 = 5;
const CORPSE: u8 = 1;
/// Not a pass: a flier between its hexes (E17 M2 T5a, design §7.10). The flight takes it off its
/// hexes (0x468730, from 0x4B4BFE), so no row pass draws it, and draws it by itself at each
/// point over the whole battlefield (0x4B4D60); it stands on the landing hex again before group
/// 21 (0x4B4E24). The scene draws this layer after every row: the recorded `fly`'s cycles across
/// rows match the original's frames (E17 M3 T1b-2).
pub const FLYING: u8 = 8;
/// A projectile's step length in pixels: trunc((d + 20) / 40) steps (§7.4).
const STEP_PIXELS: u64 = 40;
/// Half a hex: a two-hex target is aimed at from its middle (0x43F323).
const HALF_HEX: i32 = 22;
/// tan(25°) × 10^15: a shot is up above 25°, down below −25° (0x469880).
const TAN_25: i64 = 466_307_658_154_999;
/// The scale of [`TAN_25`] and of [`tangent`].
const TAN_SCALE: i64 = 1_000_000_000_000_000;

/// The combat speed (the setting `Combat Speed`, 0 to 2; 0x6987EC) and its factor F (§7.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Speed(u8);

impl Speed {
    /// The setting `setting` (0, 1 or 2; a higher one is 2).
    pub fn new(setting: u8) -> Self {
        Self(setting.min(2))
    }

    /// The setting.
    pub const fn setting(self) -> u8 {
        self.0
    }

    /// F as the f32 the executable multiplies with (0x63CF7C: 1.0, 0.63, 0.40): its mantissa
    /// over a power of two.
    const fn factor(self) -> (u64, u32) {
        match self.0 {
            0 => (1, 0),
            1 => (10_569_646, 24),
            _ => (13_421_773, 25),
        }
    }

    /// trunc(`ms` × F): the f32 operands multiplied exactly, as the x87 does at 53 bits (100 ms
    /// gives 100, 62, 40).
    pub fn scale(self, ms: u32) -> u32 {
        self.ratio(ms, 1)
    }

    /// trunc(`ms` × F / `n`): a walk's frame (§7.3); `n` of 0 counts as 1.
    pub fn ratio(self, ms: u32, n: u32) -> u32 {
        let (mantissa, shift) = self.factor();
        let v = u64::from(ms) * mantissa / (u64::from(n.max(1)) << shift);
        u32::try_from(v).unwrap_or(u32::MAX)
    }
}

/// A creature's animation numbers: its times from the asset tables and its DEF's groups.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Look {
    /// The frames of each DEF group.
    pub frames: BTreeMap<u32, u32>,
    /// `cranim.txt`'s times: fidget interval, walk, attack, flight distance
    /// ([`crate::tables::Creature::times`]).
    pub times: [u32; 4],
    /// The attack's climax frame (0: none).
    pub climax: u32,
    /// Where a projectile starts, up, straight and down (x, y; facing right).
    pub missile_offsets: [[i16; 2]; 3],
    /// The top edge of frame 0 of the standing group in its canvas: a shot aims at the target's
    /// anchor y minus (267 − it) / 2 (0x43F338; the 267 is the canvas's anchor,
    /// [`BattleLayout::stack_canvas`]).
    pub top: i32,
    /// A shooter's projectile (none: no projectile is drawn).
    pub missile: Option<Missile>,
    /// A ray shooter's colours by the distance from the ray's edge (R, G, B; none: it shoots
    /// a projectile; §7.12).
    pub ray: Option<Vec<[u8; 3]>>,
}

/// A shooter's projectile DEF and how its frame follows the shot's angle (`shot_angle_group`
/// 0x469880, `shot_animation` 0x43F5D6; design §7.9).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Missile {
    /// The DEF.
    pub def: String,
    /// Its canvas's width and height (the DEF's): each step's canvas is centred on its point.
    pub size: [i32; 2],
    /// The creature's angle in degrees for each frame of group 0, from high to low: frame f is
    /// drawn when the first f midpoints between consecutive angles are at least the shot's angle
    /// and the next is not (0x469957).
    pub angles: Vec<i16>,
}

/// A projectile in flight (`shot_animation` 0x43F5D6 to 0x43F826, design §7.9): `steps` steps
/// of one frame, the first shown with the shooter's last frame before it, one every `step` ms,
/// each over the screen as it stands (nothing else is drawn meanwhile); erased after the last.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flight {
    /// The DEF.
    pub def: String,
    /// Its frame in group 0.
    pub frame: u32,
    /// Drawn mirrored: the target lies left of the start.
    pub mirrored: bool,
    /// When the first step shows.
    pub from: u32,
    /// Each step's time, trunc(33 F).
    pub step: u32,
    /// The steps, trunc((d + 20) / 40).
    pub steps: u32,
    /// The first step's canvas: the start point less half the DEF's size.
    pub start: [i32; 2],
    /// The move per step: the distance to the target over the steps, truncated.
    pub delta: [i32; 2],
}

/// A ray (`DrawRay` 0x5A5C20, [`crate::ray`], design §7.12): the ticks' strokes, tick k shown
/// k × `step` after `from`; erased as the last shows (`DrawRay` redraws the battle then without
/// waiting for the frame deadline, so the last tick shows for no time).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Beam {
    /// When the ray starts (after the wait of trunc(115 F)).
    pub from: u32,
    /// Each tick's time, trunc(10 F).
    pub step: u32,
    /// Each tick's strokes.
    pub ticks: Vec<Vec<Stroke>>,
    /// The colours by the distance from the edge (R, G, B).
    pub colours: Vec<[u8; 3]>,
}

/// A ray as it shows at a time: the strokes drawn so far, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ray {
    /// The strokes.
    pub strokes: Vec<Stroke>,
    /// The colours by the distance from the edge (R, G, B).
    pub colours: Vec<[u8; 3]>,
}

/// A projectile as it shows at a time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Projectile {
    /// The DEF.
    pub def: String,
    /// Its frame in group 0.
    pub frame: u32,
    /// Its canvas's top-left.
    pub at: [i32; 2],
    /// Drawn mirrored.
    pub mirrored: bool,
}

impl Look {
    /// The frames of `group` (0 if the DEF lacks it).
    fn count(&self, group: u8) -> u32 {
        self.frames.get(&u32::from(group)).copied().unwrap_or(0)
    }
}

/// Where the battle is drawn: the layout, the field's width and the DEF groups.
#[derive(Debug, Clone, Copy)]
pub struct Stage<'a> {
    /// The battle screen's layout.
    pub layout: &'a BattleLayout,
    /// Hexes per row.
    pub width: u8,
    /// The DEF groups.
    pub groups: Groups,
    /// The magic animations the record shows, by number (E17 M3 T3b).
    pub magic: &'a BTreeMap<u8, MagicLook>,
}

/// A magic animation's numbers (`PlayMagicAnimation` 0x496840, design §7.15; E17 M3 T3b): its
/// DEF, the frames of its group 0, its width and height (the DEF's), where it is drawn over a
/// stack ([`crate::tables::Magic::place`]) and whether translucent.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MagicLook {
    /// The DEF.
    pub def: String,
    /// The frames of its group 0.
    pub frames: u32,
    /// The DEF's width and height.
    pub size: [i32; 2],
    /// Where it is drawn over a stack: 0 standing on the hex, 1 centred on the creature, 2 above
    /// it (3, beside it, is not built yet).
    pub place: u8,
    /// Drawn by the translucent drawer.
    pub translucent: bool,
}

/// The magic animation of the First Aid Tent's heal (`first_aid_tent_action` 0x478D14: number
/// 0x4F over the patient, 100 ms a frame, without the patient's hit).
pub const HEAL: u8 = 79;

/// A frame of a magic animation drawn over stacks, or (`frame` none) its erasure: drawn at
/// `drawn`, shown from `shown` (E17 M3 T3b).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MagicKey {
    /// Its number.
    pub animation: u8,
    /// When it is drawn.
    pub drawn: u32,
    /// When it shows.
    pub shown: u32,
    /// The frame of group 0, none once erased.
    pub frame: Option<u32>,
    /// The DEF.
    pub def: String,
    /// Drawn by the translucent drawer.
    pub translucent: bool,
    /// The stacks it is drawn over, each with the canvas's top-left corner and whether mirrored.
    pub over: Vec<(u8, [i32; 2], bool)>,
}

/// A magic animation as it shows at a time: its frame over each of its stacks, drawn after the
/// stack's sprite (`draw_battle_stack`'s tail 0x43E795; E17 M3 T3b).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Magic {
    /// The DEF.
    pub def: String,
    /// The frame of group 0.
    pub frame: u32,
    /// Drawn by the translucent drawer.
    pub translucent: bool,
    /// The stacks, each with the canvas's top-left corner and whether mirrored.
    pub over: Vec<(u8, [i32; 2], bool)>,
}

impl Stage<'_> {
    /// The anchor of `hex` (§4.1).
    fn anchor(&self, hex: u8) -> [i32; 2] {
        let width = self.width.max(1);
        self.layout.anchor(hex / width, hex % width)
    }

    /// The row of `hex`.
    fn row(&self, hex: u8) -> u8 {
        hex / self.width.max(1)
    }

    /// The neighbour of `hex` in direction `d` (0 up and right, then clockwise: right, down and
    /// right, down and left, left, up and left; the executable's table at +0x13468): the hex of
    /// the rows around whose anchor lies a column's width, or half of it and a row's height,
    /// that way. None off the field's columns or above it.
    fn neighbor(&self, hex: u8, d: u8) -> Option<u8> {
        let width = self.width.max(1);
        let (row, column) = (hex / width, hex % width);
        let [x, y] = self.anchor(hex);
        let [w, h] = self.layout.hex_step;
        let want = match d {
            0 => [x + w / 2, y - h],
            1 => [x + w, y],
            2 => [x + w / 2, y + h],
            3 => [x - w / 2, y + h],
            4 => [x - w, y],
            _ => [x - w / 2, y - h],
        };
        (row.saturating_sub(1)..=row.saturating_add(1))
            .flat_map(|r| {
                (column.saturating_sub(1)..=column.saturating_add(1)).map(move |c| (r, c))
            })
            .filter(|&(_, c)| c < width)
            .find(|&(r, c)| self.layout.anchor(r, c) == want)
            .and_then(|(r, c)| u8::try_from(u16::from(r) * u16::from(width) + u16::from(c)).ok())
    }
}

/// What a stack shows: a frame of a DEF group on a hex, moved by an offset in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pose {
    /// The DEF group.
    pub group: u32,
    /// The frame in it.
    pub frame: u32,
    /// The stack's position as the executable keeps it (+0x38): during a walk the hex it is
    /// leaving, during a flight its start until it lands.
    pub hex: u8,
    /// Where it is drawn from its hex's anchor (+0x104, +0x100).
    pub offset: [i32; 2],
    /// Drawn mirrored: it faces left (+0x44 = 0; E17 M2 T4b-2a, turns).
    pub mirrored: bool,
    /// Its draw layer (+0x8), the redraw's pass that draws it in its row (§4.6): 4 standing, 3 or
    /// 7 walking up or down, 6 a melee's striker, 5 a struck stack, 1 among the corpses; or
    /// [`FLYING`], over every row.
    pub layer: u8,
    /// Its count box hidden (+0x30, H3API's `skipCountDraw`; E17 M3 T5a, design §4.3): set
    /// through a walk, from before its first turn to its end (`stack_walk` 0x442145, 0x4423D9),
    /// and through a flight, from its cycles to the standing frame after it lands
    /// (`stack_flight_animation` 0x4B4BD8, 0x4B4E73).
    pub boxless: bool,
}

impl Pose {
    /// Whether it holds its hex against another stack's count box (0x43E605 to 0x43E627): it
    /// stands on the hex records, which a death leaves at the strike's end (`stack_death`
    /// 0x444400, from `strike_animation` 0x4692C4; from then among the corpses), and its box is
    /// not hidden.
    #[must_use]
    pub const fn holds_hex(&self) -> bool {
        !self.boxless && self.layer != CORPSE
    }
}

/// One pose of one stack: drawn at `drawn` (the harness reads it from then, design §7.1), shown
/// from `shown`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    /// The stack's id.
    pub stack: u8,
    /// When the frame is drawn.
    pub drawn: u32,
    /// When it shows.
    pub shown: u32,
    /// The pose.
    pub pose: Pose,
}

/// A stack the timeline animates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Member {
    /// Its id.
    pub stack: u8,
    /// Its side.
    pub side: Side,
    /// Its place among its side's stacks (the harness's record index).
    pub record: u8,
    /// Its pose at the start.
    pub start: Pose,
}

/// A command's events and when it was given (ms after the record's first snapshot; `None`:
/// the default, [`timeline`]).
#[derive(Debug, Clone, Copy)]
pub struct Command<'a> {
    /// When the command was given.
    pub at: Option<u32>,
    /// What it did, as the viewer saw it.
    pub events: &'a [Event],
}

/// The commands of a record's messages: each `Events` message one command, without a time (the
/// first given at the snapshot's moment, each later one when the previous one ended, [`timeline`]).
pub fn commands(messages: &[ServerMessage]) -> Vec<Command<'_>> {
    messages
        .iter()
        .filter_map(|m| match m {
            ServerMessage::Events { events, .. } => Some(Command { at: None, events }),
            _ => None,
        })
        .collect()
}

/// A creature's sounds, in the order of a stack's eight (+0x170 to +0x18C, loaded by
/// `stack_load_graphics` 0x43D9F0; E17 M2 T5b, design §8.2). Which file each is, and whether a
/// creature has it, the asset tables say.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Sound {
    /// Moving: started with a move's first cycle, looping until the move ends.
    Move,
    /// A melee strike.
    Attack,
    /// Struck.
    Wince,
    /// A shot.
    Shot,
    /// Dying.
    Death,
    /// Struck while defending.
    Defend,
    /// Before a move (group 20).
    Start,
    /// After a move (group 21).
    End,
    /// The First Aid Tent's heal, cued on its patient: the battle's sound, not the creature's
    /// (the asset tables' `heal_sound`, category 2: `play_sound_file` 0x59A470 from
    /// `first_aid_tent_action` 0x478C46; E17 M3 T3b).
    Heal,
}

/// A sound a stack starts at a time, or (`stop`) the move sound it ends then (E17 M2 T5b; the
/// sites: design §8.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cue {
    /// When: the time of the frame it goes with.
    pub at: u32,
    /// The stack.
    pub stack: u8,
    /// Its sound.
    pub sound: Sound,
    /// Stops the sound instead of starting it.
    pub stop: bool,
}

/// The poses of a record's stacks over time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Timeline {
    /// The stacks living at the start, by id.
    pub members: Vec<Member>,
    /// Every pose taken, in the order drawn.
    pub keys: Vec<Key>,
    /// The active stack from a time on (its turn started then).
    pub turns: Vec<(u32, u8)>,
    /// The projectiles' flights, in order.
    pub flights: Vec<Flight>,
    /// The rays, in order.
    pub beams: Vec<Beam>,
    /// The magic animations' frames and erasures, in order (E17 M3 T3b).
    pub magics: Vec<MagicKey>,
    /// The stack outlined as active from a time on (none: no outline), in order (§7.9).
    pub outlines: Vec<(u32, Option<u8>)>,
    /// When each idle step ran, up to the last command's start (none runs while a command
    /// plays); [`Timeline::idle`] goes on from its end.
    pub steps: Vec<u32>,
    /// The idle steps after the last command.
    pub idle: Idle,
    /// The viewer's own fidgets as they stand after the last command (none: no fidgets, as the
    /// reference checks take the original's).
    pub fidgets: Option<Fidgets>,
    /// The creatures' sounds, in time order.
    pub cues: Vec<Cue>,
    /// Each command's end and what it did that the heroes answer, in order (E17 M3 T3a).
    pub actions: Vec<Action>,
    /// The heroes beside the battlefield (none: none drawn; [`Timeline::with_heroes`]).
    pub heroes: Option<Heroes>,
    /// When the last frame shows.
    pub end: u32,
}

/// The viewer's own fidgets (§7.2, E17 M2 T4b-2a). The executable draws them from its rules'
/// generator, which the engine does not share (ADR-0003): the viewer draws them from a generator
/// of its own (the C library's `rand()` sequence, *provisional*), so they cannot match the
/// original's frame for frame. At the battle's start and after every action each stack's fidget
/// clock is drawn again: its fidget starts at the first idle step after now + 2r, r =
/// random(50, t), t its fidget interval; then one frame of group 1 per idle step, each advance
/// stalled when a draw of 0 to 100 gives less than 8; after its last frame it stands, the next
/// fidget 0.75 t to 1.25 t later (0x4960D0). A stack the action drew stops fidgeting; not read:
/// the blind, petrified and paralysed stacks the executable skips.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fidgets {
    rng: u32,
    group: u8,
    stacks: Vec<Fidget>,
}

/// One stack's fidget: its interval and frames, its clock (+0xFC), the frame it shows (none:
/// standing) and its standing pose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Fidget {
    stack: u8,
    interval: u32,
    frames: u32,
    clock: i64,
    frame: Option<u32>,
    base: Pose,
}

impl Fidgets {
    /// The C library's `rand()`: 15 bits of a linear congruential sequence.
    fn rand(&mut self) -> u32 {
        self.rng = self.rng.wrapping_mul(214_013).wrapping_add(2_531_011);
        (self.rng >> 16) & 0x7FFF
    }

    /// A draw from `low` to `high` (`low` ≤ `high`).
    fn between(&mut self, low: u32, high: u32) -> u32 {
        low + self.rand() % (high - low + 1)
    }

    /// After an action at `now` (and at the start): the stacks still living (`base`: their
    /// standing poses, by id) keep their fidgets, those `drawn` stop; each clock drawn again.
    fn reset(&mut self, now: u32, base: &BTreeMap<u8, Pose>, drawn: &[u8]) {
        self.stacks.retain(|f| base.contains_key(&f.stack));
        for i in 0..self.stacks.len() {
            let f = self.stacks[i];
            let mut next = Fidget {
                base: base.get(&f.stack).copied().unwrap_or(f.base),
                frame: f.frame.filter(|_| !drawn.contains(&f.stack)),
                ..f
            };
            if f.interval > 51 {
                let r = self.between(50, f.interval);
                next.clock = i64::from(now) + 2 * i64::from(r) - i64::from(f.interval);
            }
            self.stacks[i] = next;
        }
    }

    /// The idle step at `now`: the poses that change.
    fn step(&mut self, now: u32) -> Vec<(u8, Pose)> {
        let mut out = Vec::new();
        for i in 0..self.stacks.len() {
            let mut f = self.stacks[i];
            match f.frame {
                Some(k) => {
                    if self.between(0, 100) < 8 {
                        continue;
                    }
                    if k + 1 < f.frames {
                        f.frame = Some(k + 1);
                    } else {
                        f.frame = None;
                        let r = i64::from(self.between(0, f.interval));
                        f.clock = (4 * i64::from(now) + 2 * r - i64::from(f.interval)) / 4;
                    }
                }
                None if i64::from(now) - f.clock > i64::from(f.interval) => f.frame = Some(0),
                None => continue,
            }
            let pose = f.frame.map_or(f.base, |frame| Pose {
                group: u32::from(self.group),
                frame,
                ..f.base
            });
            out.push((f.stack, pose));
            self.stacks[i] = f;
        }
        out
    }
}

/// The battle loop's idle steps (0x4960D0, §7.1, §7.9): while the loop ticks (from `from`), a
/// step runs whenever the time reached `next`, which then moves on to the old one plus
/// max(the time since it, 100 ms): on the grid while idle, twice at once after an animation
/// held them up by 100 ms or more.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Idle {
    /// The loop ticks from then.
    pub from: u32,
    /// The next step's time (0x6989B8).
    pub next: u32,
}

impl Idle {
    /// Runs the steps up to `until` (included), each time given to `step`.
    pub(crate) fn run(&mut self, until: u32, mut step: impl FnMut(u32)) {
        loop {
            let now = self.next.max(self.from);
            if now > until {
                return;
            }
            step(now);
            self.next += (now - self.next).max(IDLE_STEP);
        }
    }

    /// One step at `now` if it is due (the computer acts on its turn's first tick); whether
    /// one ran.
    fn once(&mut self, now: u32, steps: &mut Vec<u32>) -> bool {
        let due = self.next <= now;
        if due {
            steps.push(now);
            self.next += (now - self.next).max(IDLE_STEP);
        }
        due
    }
}

impl Timeline {
    /// The index of the first cue at or after `t`: a player that seeks to `t` goes on from there.
    #[must_use]
    pub fn cue_from(&self, t: u32) -> usize {
        self.cues.partition_point(|c| c.at < t)
    }

    /// Each stack's pose shown at `t`.
    pub fn at(&self, t: u32) -> BTreeMap<u8, Pose> {
        let mut poses: BTreeMap<u8, Pose> =
            self.members.iter().map(|m| (m.stack, m.start)).collect();
        for key in self.keys.iter().filter(|k| k.shown <= t) {
            poses.insert(key.stack, key.pose);
        }
        // After the last command the idle steps go on, and the fidgets with them.
        if let Some(fidgets) = &self.fidgets {
            let (mut idle, mut fidgets) = (self.idle, fidgets.clone());
            idle.run(t, |now| poses.extend(fidgets.step(now)));
        }
        poses
    }

    /// The projectile showing at `t`, if one flies then.
    pub fn projectile(&self, t: u32) -> Option<Projectile> {
        self.flights.iter().find_map(|f| {
            let k = t.checked_sub(f.from)? / f.step.max(1);
            (k < f.steps).then(|| {
                let k = i32::try_from(k).unwrap_or(0);
                Projectile {
                    def: f.def.clone(),
                    frame: f.frame,
                    at: [f.start[0] + k * f.delta[0], f.start[1] + k * f.delta[1]],
                    mirrored: f.mirrored,
                }
            })
        })
    }

    /// The ray showing at `t`, if one is drawn then: the strokes of the ticks shown by then.
    pub fn ray(&self, t: u32) -> Option<Ray> {
        self.beams.iter().find_map(|b| {
            let k = usize::try_from(t.checked_sub(b.from)? / b.step.max(1)).unwrap_or(usize::MAX);
            (k >= 1 && k < b.ticks.len()).then(|| Ray {
                strokes: b.ticks.iter().take(k).flatten().copied().collect(),
                colours: b.colours.clone(),
            })
        })
    }

    /// The magic animation showing at `t`, if one is drawn then (E17 M3 T3b).
    pub fn magic(&self, t: u32) -> Option<Magic> {
        let key = self.magics.iter().rev().find(|k| k.shown <= t)?;
        key.frame.map(|frame| Magic {
            def: key.def.clone(),
            frame,
            translucent: key.translucent,
            over: key.over.clone(),
        })
    }

    /// The stack outlined as active at `t` (§7.9): the snapshot's active stack until the first
    /// command's first frame shows; none while a command plays; a turn's stack from its start
    /// when a player acts next, none when the computer does (*provisional*).
    pub fn outlined(&self, t: u32) -> Option<u8> {
        self.outlines
            .iter()
            .rev()
            .find(|(at, _)| *at <= t)
            .and_then(|(_, stack)| *stack)
    }

    /// Whether nothing shown changes after the last frame: no stack outlined then, none left
    /// that fidgets (a window need not look further; E17 M2 T5c, the reviewer).
    pub fn still(&self) -> bool {
        self.outlined(self.end).is_none()
            && self.fidgets.as_ref().is_none_or(|f| f.stacks.is_empty())
            && self.heroes.as_ref().is_none_or(|h| !h.any())
    }

    /// The timeline with its battle's heroes drawn (E17 M3 T3a).
    #[must_use]
    pub fn with_heroes(self, heroes: Heroes) -> Self {
        Self {
            heroes: Some(heroes),
            ..self
        }
    }

    /// The idle steps taken by `t` (the outline's pulse, §4.2): the opening's at 0, then as
    /// [`Idle`] says.
    pub fn pulse(&self, t: u32) -> u32 {
        let mut count = self.steps.iter().filter(|s| **s <= t).count();
        let mut idle = self.idle;
        idle.run(t, |_| count += 1);
        u32::try_from(count).unwrap_or(u32::MAX)
    }

    /// Every time up to `until` at which what shows may change (E17 M2 T4b-2a, `h3-view
    /// play`): 0, each pose's, each projectile step's and its erasure's, each ray tick's and its
    /// erasure's, each magic animation's frame and erasure, each outline's and each idle step's
    /// (the outline's colour), and the last frame's; in order, each once.
    pub fn changes(&self, until: u32) -> Vec<u32> {
        let mut times = vec![0, self.end];
        times.extend(self.keys.iter().map(|k| k.shown));
        for f in &self.flights {
            times.extend((0..=f.steps).map(|k| f.from + k * f.step));
        }
        for b in &self.beams {
            let ticks = u32::try_from(b.ticks.len()).unwrap_or(0);
            times.extend((1..=ticks).map(|k| b.from + k * b.step));
        }
        times.extend(self.magics.iter().map(|k| k.shown));
        times.extend(self.outlines.iter().map(|(t, _)| *t));
        times.extend(&self.steps);
        let mut idle = self.idle;
        idle.run(until, |t| times.push(t));
        times.retain(|t| *t <= until);
        times.sort_unstable();
        times.dedup();
        times
    }

    /// The active stack at `t` (its turn's start), if a turn started by then.
    pub fn active(&self, t: u32) -> Option<u8> {
        self.turns
            .iter()
            .rev()
            .find(|(at, _)| *at <= t)
            .map(|(_, stack)| *stack)
    }

    /// The timeline in the capture harness's format (design §7.6): one line per change of a
    /// stack's group, frame or hex, `<ms> <side>.<record> g<group> f<frame> h<hex>`, every stack
    /// at 0; a change at a time is the last pose drawn then. After a time's stacks, a change of
    /// the magic animation drawn (E17 M3 T3b): `<ms> magic <number> f<frame> <side>.<record>…`
    /// over its stacks, `<ms> magic -` once erased.
    pub fn lines(&self) -> String {
        let mut now: BTreeMap<u8, Pose> = self.members.iter().map(|m| (m.stack, m.start)).collect();
        let mut printed: BTreeMap<u8, (u32, u32, u8)> = BTreeMap::new();
        let mut out = String::new();
        let mut times: Vec<u32> = self.keys.iter().map(|k| k.drawn).collect();
        times.extend(self.magics.iter().map(|k| k.drawn));
        times.push(0);
        times.sort_unstable();
        times.dedup();
        for t in times {
            for key in self.keys.iter().filter(|k| k.drawn == t) {
                now.insert(key.stack, key.pose);
            }
            for m in &self.members {
                let p = now.get(&m.stack).copied().unwrap_or(m.start);
                let seen = (p.group, p.frame, p.hex);
                if printed.insert(m.stack, seen) != Some(seen) {
                    let side = u8::from(m.side == Side::Defender);
                    let _ = writeln!(
                        out,
                        "{t} {side}.{} g{} f{} h{}",
                        m.record, p.group, p.frame, p.hex
                    );
                }
            }
            // Each key drawn differs from the one before it (a frame on, or the erasure).
            if let Some(key) = self.magics.iter().rev().find(|k| k.drawn == t) {
                let _ = match key.frame {
                    Some(frame) => {
                        let over = key.over.iter().filter_map(|(stack, ..)| {
                            self.members.iter().find(|m| m.stack == *stack)
                        });
                        let records: Vec<String> = over
                            .map(|m| format!("{}.{}", u8::from(m.side == Side::Defender), m.record))
                            .collect();
                        let records = records.join(" ");
                        writeln!(out, "{t} magic {} f{frame} {records}", key.animation)
                    }
                    None => writeln!(out, "{t} magic -"),
                };
            }
        }
        out
    }
}

/// The original's clock: the frame deadline 0x698998 on its grid (§7.1).
#[derive(Debug, Clone, Copy)]
struct Clock {
    now: u32,
    deadline: u32,
    grid: u32,
}

impl Clock {
    /// A command at `at`: the clock moves there; the battle's tick (0x474040) has kept the
    /// deadline on its grid, so it is the grid's next point after it.
    fn start(&mut self, at: u32) {
        self.now = self.now.max(at);
        if self.deadline <= self.now {
            let behind = self.now - self.deadline;
            self.deadline += (behind / self.grid + 1) * self.grid;
        }
    }

    /// A frame drawn now waits for the deadline (`wait_until` 0x4F82F0), which then moves on by
    /// max(`time`, the lateness); returns when it shows.
    fn show(&mut self, time: u32) -> u32 {
        let shown = self.now.max(self.deadline);
        let late = shown - self.deadline;
        self.deadline += time.max(late);
        self.now = shown;
        shown
    }

    /// A wait outside the deadline.
    fn wait(&mut self, ms: u32) {
        self.now += ms;
    }
}

/// A stack as the timeline follows it.
#[derive(Debug, Clone)]
struct Live {
    side: Side,
    /// The engine's position (a two-hex stack's rear hex by its side, BAT-GEO-002).
    hex: u8,
    count: u32,
    flies: bool,
    two_hex: bool,
    defending: bool,
    /// A war machine (E17 M3 T3b).
    war_machine: bool,
    /// It faces right (+0x44 = 1); the setup faces the attacker right (0x43D846).
    right: bool,
    /// Its count box hidden: a walk or a flight under way ([`Pose::boxless`]).
    boxless: bool,
    look: Look,
}

impl Live {
    /// It faces the way its side does at the setup.
    fn at_home(&self) -> bool {
        self.right == (self.side == Side::Attacker)
    }

    /// The executable's position (+0x38) for the engine's `hex`: a two-hex stack turned away
    /// from its side's facing stands on its other hex, + 1 for the attacker, − 1 for the
    /// defender (`stack_turn_around` 0x446720).
    fn position(&self, hex: u8) -> u8 {
        match (self.two_hex && !self.at_home(), self.side) {
            (false, _) => hex,
            (true, Side::Attacker) => hex.saturating_add(1),
            (true, Side::Defender) => hex.saturating_sub(1),
        }
    }

    /// The hexes it covers on the executable's field: its position, and a two-hex stack's other
    /// hex ahead of it (0x4466A0).
    fn covers(&self, hex: u8) -> [u8; 2] {
        let at = self.position(hex);
        match (self.two_hex, self.right) {
            (false, _) => [at, at],
            (true, true) => [at, at.saturating_add(1)],
            (true, false) => [at, at.saturating_sub(1)],
        }
    }
}

/// Why a record has no timeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnimError {
    /// A living stack's creature (its handle) has no [`Look`].
    Look(u16),
}

impl core::fmt::Display for AnimError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Look(handle) => write!(f, "no animation numbers for creature handle {handle}"),
        }
    }
}

/// The poses of the battle `snapshot` shows when `commands` follow, at combat speed `speed`;
/// `looks` holds each creature's numbers by its handle.
///
/// The first command is given at 0, the snapshot's moment (the harness clicks at the frozen
/// opening); a later one without a time as soon as the previous one's last frame showed
/// (*provisional*: in the melee sequence the computer acts as the flight's second standing frame
/// shows, 0x4B4A15; after Wait or Defend the next stack acts at once, §7.4).
///
/// # Errors
/// A living stack whose creature has no [`Look`].
pub fn timeline(
    snapshot: &Observation,
    commands: &[Command<'_>],
    looks: &BTreeMap<u16, Look>,
    stage: Stage<'_>,
    speed: Speed,
) -> Result<Timeline, AnimError> {
    played(snapshot, commands, looks, stage, speed, None)
}

/// As [`timeline`], the stacks fidgeting while the battle idles, from the viewer's own generator
/// seeded with `seed` ([`Fidgets`]; E17 M2 T4b-2a).
///
/// # Errors
/// As [`timeline`].
pub fn fidgeting(
    snapshot: &Observation,
    commands: &[Command<'_>],
    looks: &BTreeMap<u16, Look>,
    stage: Stage<'_>,
    speed: Speed,
    seed: u32,
) -> Result<Timeline, AnimError> {
    played(snapshot, commands, looks, stage, speed, Some(seed))
}

/// [`timeline`] or [`fidgeting`].
fn played(
    snapshot: &Observation,
    commands: &[Command<'_>],
    looks: &BTreeMap<u16, Look>,
    stage: Stage<'_>,
    speed: Speed,
    seed: Option<u32>,
) -> Result<Timeline, AnimError> {
    let standing = u32::from(stage.groups.standing);
    let mut members = Vec::new();
    // The stacks whose DEF has a fidget (group 1).
    let mut fidgeters = Vec::new();
    let mut stacks = BTreeMap::new();
    let mut records = [0u8; 2];
    // The harness's records follow the original's setup: each side's army, then its war
    // machines in `battle_war_machine_stacks`' order (E17 M3 T3b).
    let mut living: Vec<&Stack> = snapshot.stacks.iter().filter(|s| s.alive).collect();
    living.sort_by_key(|s| {
        (
            s.side == Side::Defender,
            s.war_machine,
            setup_order(s.slot),
            s.id,
        )
    });
    for s in living {
        let look = looks.get(&s.creature).ok_or(AnimError::Look(s.creature))?;
        let side = usize::from(s.side == Side::Defender);
        let start = Pose {
            mirrored: s.side == Side::Defender,
            ..still(standing, s.hex)
        };
        members.push(Member {
            stack: s.id,
            side: s.side,
            record: records[side],
            start,
        });
        records[side] += 1;
        let frames = look.count(stage.groups.mouse_over);
        if frames > 0 {
            fidgeters.push(Fidget {
                stack: s.id,
                interval: look.times[0],
                frames,
                clock: 0,
                frame: None,
                base: start,
            });
        }
        stacks.insert(
            s.id,
            Live {
                side: s.side,
                hex: s.hex,
                count: s.count,
                flies: s.flies,
                two_hex: s.two_hex,
                defending: s.defending,
                war_machine: s.war_machine,
                right: s.side == Side::Attacker,
                boxless: false,
                look: look.clone(),
            },
        );
    }
    let grid = speed.scale(TICK).max(1);
    let mut player = Player {
        stage,
        speed,
        clock: Clock {
            now: 0,
            deadline: 0,
            grid,
        },
        stacks,
        keys: Vec::new(),
        pending: Vec::new(),
        turns: Vec::new(),
        flights: Vec::new(),
        beams: Vec::new(),
        magics: Vec::new(),
        outlined: Vec::new(),
        machine: None,
        idle: Idle::default(),
        steps: Vec::new(),
        cues: Vec::new(),
        happenings: Vec::new(),
    };
    let mut actions = Vec::new();
    let mut fidgets = seed.map(|rng| Fidgets {
        rng,
        group: stage.groups.mouse_over,
        stacks: fidgeters,
    });
    if let Some(f) = fidgets.as_mut() {
        f.reset(0, &player.bases(), &[]);
    }
    let mut outlines = vec![(0, snapshot.active)];
    let mut waiting = snapshot.active;
    for (i, command) in commands.iter().enumerate() {
        let default = if i == 0 { 0 } else { player.clock.now };
        player.clock.start(command.at.unwrap_or(default));
        // A command without a time after the first is the computer's: the loop did not tick
        // since the last one ended, but once when it acts, which outlines its stack until its
        // first frame shows (*provisional*, §7.9).
        let ran = player.steps.len();
        let now = player.clock.now;
        if i > 0 && command.at.is_none() {
            if player.idle.once(now, &mut player.steps) {
                outlines.push((now, waiting));
            }
        } else {
            let Player { idle, steps, .. } = &mut player;
            idle.run(now, |t| steps.push(t));
        }
        // The last command's layer changes show with the battle's next redraw: the first idle
        // step since, if one ran, before any fidget (*provisional*, as at the end; E17 M2 T5c).
        if let Some(&first) = player.steps.get(ran) {
            player.flush(first, first);
        }
        if let Some(f) = fidgets.as_mut() {
            let steps = player.steps[ran..].to_vec();
            for now in steps {
                player
                    .keys
                    .extend(f.step(now).into_iter().map(|(stack, pose)| Key {
                        stack,
                        drawn: now,
                        shown: now,
                        pose,
                    }));
            }
        }
        let (before, turns) = (player.keys.len(), player.turns.len());
        player.command(command.events);
        // The outline goes with the command's first frame shown; a turn started brings its
        // stack's, unless the computer acts next.
        if let Some(first) = player.keys.get(before) {
            outlines.push((first.shown, None));
        }
        outlines.append(&mut player.outlined);
        if let Some(&(at, stack)) = player.turns.get(turns..).and_then(<[_]>::last) {
            let computer = commands.get(i + 1).is_some_and(|c| c.at.is_none());
            outlines.push((at, (!computer).then_some(stack)));
            waiting = Some(stack);
        }
        player.idle.from = player.clock.now;
        actions.push(Action {
            end: player.clock.now,
            happenings: core::mem::take(&mut player.happenings),
        });
        if let Some(f) = fidgets.as_mut() {
            let drawn: Vec<u8> = player.keys[before..].iter().map(|k| k.stack).collect();
            f.reset(player.clock.now, &player.bases(), &drawn);
        }
    }
    // The last layer changes show with the battle's next redraw (*provisional*).
    let now = player.clock.now;
    player.flush(now, now);
    Ok(Timeline {
        members,
        keys: player.keys,
        turns: player.turns,
        flights: player.flights,
        beams: player.beams,
        magics: player.magics,
        outlines,
        steps: player.steps,
        idle: player.idle,
        fidgets,
        cues: player.cues,
        actions,
        heroes: None,
        end: player.clock.now,
    })
}

/// Where a war machine of army slot `slot` comes in the original's setup (`battle_war_machine_
/// stacks` 0x463600: the Catapult, the Ballista, the First Aid Tent, the Ammo Cart, then the
/// turrets; the engine's slots are 16 + its kind, BAT-WAR-001: the Ballista 16, the Ammo Cart 17,
/// the tent 18, the Catapult 19, the turrets 20 to 22); any other slot after them, by itself.
fn setup_order(slot: u8) -> (usize, u8) {
    const ORDER: [u8; 4] = [19, 16, 18, 17];
    let at = ORDER.iter().position(|&s| s == slot);
    (at.unwrap_or(ORDER.len()), slot)
}

/// A still pose: frame 0 of `group` on `hex`, facing right, standing's layer.
const fn still(group: u32, hex: u8) -> Pose {
    Pose {
        group,
        frame: 0,
        hex,
        offset: [0, 0],
        mirrored: false,
        layer: STANDING,
        boxless: false,
    }
}

/// A frame of `group` on `hex`, facing right, standing's layer.
#[cfg(test)]
fn pose(group: u8, frame: u32, hex: u8) -> Pose {
    Pose {
        frame,
        ..still(u32::from(group), hex)
    }
}

/// `part` of `whole` pixels, `of` the way (`of` > 0).
fn share(whole: i32, part: u32, of: u32) -> i32 {
    let v = i64::from(whole) * i64::from(part) / i64::from(of.max(1));
    i32::try_from(v).unwrap_or(0)
}

/// A struck stack and how it reacts: its reaction's group and frames, whether it died, and its
/// standing pose (its position and facing).
#[derive(Debug, Clone, Copy)]
struct Reaction {
    stack: u8,
    group: u8,
    frames: u32,
    died: bool,
    sound: Sound,
    base: Pose,
}

/// A shot's aim (0x469880): its group, its projectile's steps, start and target, and the slope
/// its angle comes from.
#[derive(Debug, Clone, Copy)]
struct Aim {
    /// 0 up, 1 straight, 2 down.
    direction: usize,
    steps: u32,
    /// The start point (the chosen group's missile offset) and the target point.
    from: [i32; 2],
    to: [i32; 2],
    /// |dx| and dy from the straight group's start point to the target: the angle is
    /// atan(−dy / |dx|), and ±90° the other way round for dx = 0.
    slope: [i64; 2],
}

impl Default for Aim {
    /// No target: the straight group, no projectile.
    fn default() -> Self {
        Self {
            direction: 1,
            steps: 0,
            from: [0, 0],
            to: [0, 0],
            slope: [0, 0],
        }
    }
}

impl Aim {
    /// The projectile's frame (0x469957): the count of leading midpoints between consecutive
    /// angles that are at least the shot's angle. The original compares the angle as an f32
    /// with the midpoint; comparing tangents is the same but within an f32's rounding of a
    /// midpoint (*provisional*).
    fn frame(&self, missile: &Missile) -> u32 {
        let [dx, dy] = self.slope;
        let at_least = |pair: &[i16]| {
            let midpoint = tangent(i32::from(pair[0]) + i32::from(pair[1]));
            match (midpoint, dx) {
                (i64::MAX, _) => true,
                // Straight below the angle is +90°, above −90° (the sign inverted, 0x4698EA).
                (_, 0) => dy <= 0,
                _ => {
                    i128::from(midpoint) * i128::from(dx) >= -i128::from(dy) * i128::from(TAN_SCALE)
                }
            }
        };
        let leading = missile
            .angles
            .windows(2)
            .take_while(|p| at_least(p))
            .count();
        u32::try_from(leading).unwrap_or(0)
    }
}

/// Plays the commands.
#[derive(Debug)]
struct Player<'a> {
    stage: Stage<'a>,
    speed: Speed,
    clock: Clock,
    stacks: BTreeMap<u8, Live>,
    keys: Vec<Key>,
    /// Poses that change without a frame of their own (a layer set or restored): they show with
    /// the next frame drawn, as the executable draws them at its next redraw.
    pending: Vec<(u8, Pose)>,
    turns: Vec<(u32, u8)>,
    flights: Vec<Flight>,
    beams: Vec<Beam>,
    /// The magic animations' frames (E17 M3 T3b).
    magics: Vec<MagicKey>,
    /// The outline's changes inside the command so far: a war machine's turn that plays out
    /// in it (E17 M3 T3b).
    outlined: Vec<(u32, Option<u8>)>,
    /// The war machine outlined at its turn's idle step until its action's first frame shows.
    machine: Option<u8>,
    /// The battle loop's idle steps: their clock and when each ran (§7.1).
    idle: Idle,
    steps: Vec<u32>,
    cues: Vec<Cue>,
    /// What the command so far did that the heroes answer (E17 M3 T3a).
    happenings: Vec<Happening>,
}

/// A command's melee (`move_and_attack` 0x478900, `melee_attack` 0x441CB0): its striker, its
/// target, how each faced before it (the striker before its move) and whether the target has
/// turned back, its strikes over.
#[derive(Debug, Clone, Copy)]
struct Melee {
    attacker: u8,
    target: u8,
    attacker_right: bool,
    target_right: bool,
    over: bool,
}

impl Player<'_> {
    /// One frame: `poses` drawn now, shown at the deadline; the deadline then moves on by `time`.
    /// The pending poses of the other stacks show with it.
    fn frame(&mut self, poses: &[(u8, Pose)], time: u32) {
        let drawn = self.clock.now;
        let shown = self.clock.show(time);
        if self.machine.take().is_some() {
            self.outlined.push((shown, None));
        }
        self.pending
            .retain(|(s, _)| poses.iter().all(|(p, _)| p != s));
        self.flush(drawn, shown);
        self.keys.extend(poses.iter().map(|&(stack, pose)| Key {
            stack,
            drawn,
            shown,
            pose,
        }));
    }

    /// A frame drawn and shown now, the frame deadline left as it is (`battle_redraw` without
    /// its sixth argument): the pending poses show.
    fn redraw(&mut self) {
        let now = self.clock.now;
        self.flush(now, now);
    }

    /// `stack`'s `sound` starts (or, `stop`, ends) with the next frame drawn.
    fn cue(&mut self, stack: u8, sound: Sound, stop: bool) {
        self.cues.push(Cue {
            at: self.clock.now,
            stack,
            sound,
            stop,
        });
    }

    /// The pending poses, drawn at `drawn` and shown at `shown`.
    fn flush(&mut self, drawn: u32, shown: u32) {
        let pending = core::mem::take(&mut self.pending);
        self.keys
            .extend(pending.into_iter().map(|(stack, pose)| Key {
                stack,
                drawn,
                shown,
                pose,
            }));
    }

    /// Frame `frame` of `group` of `live` on the engine's `hex`: at its position, facing its way,
    /// in standing's layer.
    fn posed(live: &Live, group: u8, frame: u32, hex: u8) -> Pose {
        Pose {
            group: u32::from(group),
            frame,
            hex: live.position(hex),
            offset: [0, 0],
            mirrored: !live.right,
            layer: STANDING,
            boxless: live.boxless,
        }
    }

    /// `live` standing on the engine's `hex`.
    fn standing(&self, live: &Live, hex: u8) -> Pose {
        Self::posed(live, self.stage.groups.standing, 0, hex)
    }

    /// Every living stack's standing pose, by id.
    fn bases(&self) -> BTreeMap<u8, Pose> {
        self.stacks
            .iter()
            .map(|(&id, live)| (id, self.standing(live, live.hex)))
            .collect()
    }

    /// A living stack as the timeline follows it now.
    fn live(&self, stack: u8) -> Option<Live> {
        self.stacks.get(&stack).cloned()
    }

    /// One command's events, in order.
    fn command(&mut self, events: &[Event]) {
        let mut i = 0;
        // A melee strike of this command played: the next one waits (§7.4).
        let mut struck = false;
        let mut melee: Option<Melee> = None;
        // The striker's facing before the move that brought it to its target.
        let mut before_move: Option<(u8, bool)> = None;
        // A shooter and its facing before its shots.
        let mut shooter: Option<(u8, bool)> = None;
        while let Some(event) = events.get(i) {
            i += 1;
            match *event {
                Event::Moved { stack, from, to } => {
                    let mut path = vec![from, to];
                    // A walk comes hex by hex (CLT-Q1): the run of this stack's moves.
                    while let Some(&Event::Moved {
                        stack: s,
                        from: f,
                        to: t,
                    }) = events.get(i)
                    {
                        if s != stack || path.last() != Some(&f) {
                            break;
                        }
                        path.push(t);
                        i += 1;
                    }
                    // A melee's target turns back before its striker returns (the Harpies,
                    // 0x4789F8).
                    self.melee_over(melee.as_mut());
                    // A move that brings its stack to strike does not turn back (0x478971: the
                    // attack moves with the flag clear); a plain move does (0x479106).
                    let strikes = events[i..].iter().any(|e| {
                        matches!(*e, Event::Struck { attacker, flags, .. }
                            if attacker == stack && !flags.ranged && !flags.retaliation)
                    });
                    if strikes && melee.is_none() {
                        before_move = self.live(stack).map(|l| (stack, l.right));
                    }
                    self.moved(stack, &path, !strikes);
                }
                Event::Struck {
                    attacker,
                    target,
                    kills,
                    flags,
                    ..
                } if !flags.secondary => {
                    let mut hit = vec![(target, kills)];
                    while let Some(&Event::Struck {
                        attacker: a,
                        target: t,
                        kills: k,
                        flags: f,
                        ..
                    }) = events.get(i)
                    {
                        if a != attacker || !f.secondary {
                            break;
                        }
                        hit.push((t, k));
                        i += 1;
                    }
                    if flags.ranged {
                        if shooter.is_none() {
                            shooter = self.live(attacker).map(|l| (attacker, l.right));
                            self.face_shot(attacker, target);
                        }
                        self.shot(attacker, &hit);
                    } else {
                        if struck {
                            self.clock.wait(self.speed.scale(PAUSE));
                        }
                        let layer = if struck { STRUCK } else { STRIKER };
                        struck = true;
                        if melee.is_none() && !flags.retaliation {
                            melee = self.melee(attacker, target, before_move);
                        }
                        self.strike(attacker, &hit, layer);
                    }
                }
                Event::Struck { target, kills, .. } => {
                    self.reactions(&[(target, kills)]);
                }
                Event::Defended { stack } => self.defends(stack, true),
                Event::Healed { healer, target, .. } => self.heal(healer, target),
                Event::RoundStarted { .. } => self.happenings.push(Happening::Round),
                Event::TurnStarted { stack } => {
                    // The attack ends before the next turn starts (E17 M2 T5c, the reviewer).
                    self.attack_over(&mut melee, &mut shooter);
                    self.defends(stack, false);
                    self.turns.push((self.clock.now, stack));
                    // A war machine's turn the command plays out (another turn follows in it):
                    // the battle loop ticks once as it acts, as for the computer's, which
                    // outlines it until its action's first frame shows (E17 M3 T3b, `machines`).
                    let later = events[i..]
                        .iter()
                        .any(|e| matches!(e, Event::TurnStarted { .. }));
                    let machine = self.stacks.get(&stack).is_some_and(|l| l.war_machine);
                    let now = self.clock.now;
                    if later && machine && self.idle.once(now, &mut self.steps) {
                        self.outlined.push((now, Some(stack)));
                        self.machine = Some(stack);
                    }
                }
                _ => {}
            }
        }
        self.attack_over(&mut melee, &mut shooter);
    }

    /// The First Aid Tent heals `target` (`first_aid_tent_action` 0x478B90, design §7.15; E17
    /// M3 T3b): the tent does not move and stays the active stack, outlined from its turn's idle
    /// step (or, commanded, the heal's first frame) to the heal's end; the heal's sound starts, then magic animation
    /// [`HEAL`] over the patient, a frame per redraw of trunc(100 F) on the deadline
    /// (`PlayMagicAnimation` 0x496840 without the hit), and a last redraw that erases it. The
    /// battle then waits for the sound to end (`wait_for_sample` 0x59A4C0), which it has at
    /// speed 0 (`Regener`, 1 881 ms, against the animation's 2 100); not at the faster speeds
    /// with the sound on (*provisional*: not built, the captures play without sound).
    fn heal(&mut self, healer: u8, target: u8) {
        self.cue(target, Sound::Heal, false);
        let Some(look) = self.stage.magic.get(&HEAL).cloned() else {
            return;
        };
        let over: Vec<(u8, [i32; 2], bool)> = self
            .live(target)
            .map(|live| (target, self.magic_at(&live, &look), !live.right))
            .into_iter()
            .collect();
        // The tent stays the active stack: outlined through the heal.
        let ticked = self.machine.take() == Some(healer);
        let time = self.speed.scale(TICK);
        for frame in (0..look.frames).map(Some).chain([None]) {
            let drawn = self.clock.now;
            let shown = self.clock.show(time);
            self.flush(drawn, shown);
            if frame == Some(0) && !ticked {
                self.outlined.push((shown, Some(healer)));
            }
            self.magics.push(MagicKey {
                animation: HEAL,
                drawn,
                shown,
                frame,
                def: look.def.clone(),
                translucent: look.translucent,
                over: over.clone(),
            });
        }
        self.outlined.push((self.clock.now, None));
    }

    /// The canvas's top-left corner of magic animation `look` over `live` (`draw_battle_stack`
    /// 0x43E7FD to 0x43E98C): centred on its position's anchor (on a two-hex stack's middle, 22
    /// pixels toward its front), standing on the anchor (place 0), centred on the creature, whose
    /// height is 267 less its standing frame's top (+0x16C, 0x43DD62; place 1), or above it
    /// (place 2); the halves truncated toward zero. Place 3 (beside it) and the higher ones are
    /// not built: they stand on the anchor (*provisional*).
    fn magic_at(&self, live: &Live, look: &MagicLook) -> [i32; 2] {
        let [x, y] = self.stage.anchor(live.position(live.hex));
        let middle = if !live.two_hex {
            0
        } else if live.right {
            HALF_HEX
        } else {
            -HALF_HEX
        };
        let [w, h] = look.size;
        let height = self.stage.layout.stack_canvas[1] - live.look.top;
        let top = match look.place {
            1 => y - h / 2 - height / 2,
            2 => y - h - height,
            _ => y - h,
        };
        [x + middle - w / 2, top]
    }

    /// An attack is over: a melee's target turns back, then its striker at the attack's end
    /// (0x478A57), a shooter after its shots (0x4402F2), each only alive and turned.
    fn attack_over(&mut self, melee: &mut Option<Melee>, shooter: &mut Option<(u8, bool)>) {
        self.melee_over(melee.as_mut());
        let back = [
            melee.take().map(|m| (m.attacker, m.attacker_right)),
            shooter.take(),
        ];
        for (stack, right) in back.into_iter().flatten() {
            if let Some(live) = self.live(stack).filter(|l| l.right != right) {
                self.turn(stack, live);
            }
        }
    }

    /// A melee starts (`strike_action` 0x445C6A, `melee_attack` 0x441CC1, 0x441D79): the
    /// striker turns to face its target if it is not on its front side, takes the striker's
    /// layer, and the target turns to face it likewise.
    fn melee(&mut self, attacker: u8, target: u8, before: Option<(u8, bool)>) -> Option<Melee> {
        let (Some(mut striker), Some(struck)) = (self.live(attacker), self.live(target)) else {
            return None;
        };
        let attacker_right = before
            .filter(|(s, _)| *s == attacker)
            .map_or(striker.right, |(_, r)| r);
        let target_right = struck.right;
        if !self.faces(&striker, &struck) {
            striker = self.turn(attacker, striker);
        }
        let pose = Pose {
            layer: STRIKER,
            ..self.standing(&striker, striker.hex)
        };
        self.pending.push((attacker, pose));
        if !self.faces(&struck, &striker) {
            self.turn(target, struck);
        }
        Some(Melee {
            attacker,
            target,
            attacker_right,
            target_right,
            over: false,
        })
    }

    /// A melee's strikes are over: its target turns back if it lives and turned (0x441F36).
    fn melee_over(&mut self, melee: Option<&mut Melee>) {
        let Some(m) = melee.filter(|m| !m.over) else {
            return;
        };
        m.over = true;
        if let Some(live) = self.live(m.target).filter(|l| l.right != m.target_right) {
            self.turn(m.target, live);
        }
    }

    /// Whether `stack` has `other` on its front side, so a strike between them needs no turn
    /// (0x445BCE, 0x4458B0 through 0x523DF0): one of the three hexes ahead of a one-hex stack,
    /// any neighbour of a two-hex stack's front hex.
    fn faces(&self, s: &Live, o: &Live) -> bool {
        let theirs = o.covers(o.hex);
        let front = s.covers(s.hex)[1];
        let ahead: &[u8] = match (s.two_hex, s.right) {
            (true, _) => &[0, 1, 2, 3, 4, 5],
            (false, true) => &[0, 1, 2],
            (false, false) => &[3, 4, 5],
        };
        ahead
            .iter()
            .filter_map(|d| self.stage.neighbor(front, *d))
            .any(|h| theirs.contains(&h))
    }

    /// A shooter turns to face its target (`shot_action` 0x440160): the target's position right
    /// of the shooter's other hex (by their anchors' x) while it faces left, or left of it while
    /// it faces right.
    fn face_shot(&mut self, shooter: u8, target: u8) {
        let (Some(s), Some(t)) = (self.live(shooter), self.live(target)) else {
            return;
        };
        let [own, _] = self.stage.anchor(s.covers(s.hex)[1]);
        let [theirs, _] = self.stage.anchor(t.position(t.hex));
        if (theirs > own && !s.right) || (theirs < own && s.right) {
            self.turn(shooter, s);
        }
    }

    /// `stack` turns around, animated (`stack_turn_around` 0x446720 with its flag): facing right
    /// group 7, the flip, group 10; facing left group 9, the flip, group 8; each all its frames
    /// at trunc(100 F), then one standing; a two-hex stack's position moves onto its other hex.
    fn turn(&mut self, stack: u8, mut live: Live) -> Live {
        let g = self.stage.groups;
        let [before, after] = if live.right {
            g.turn_left
        } else {
            g.turn_right
        };
        let tick = self.speed.scale(TICK);
        for f in 0..live.look.count(before) {
            self.frame(&[(stack, Self::posed(&live, before, f, live.hex))], tick);
        }
        live.right = !live.right;
        for f in 0..live.look.count(after) {
            self.frame(&[(stack, Self::posed(&live, after, f, live.hex))], tick);
        }
        let standing = self.standing(&live, live.hex);
        self.frame(&[(stack, standing)], tick);
        self.stacks.insert(stack, live.clone());
        live
    }

    /// Sets whether `stack` defends.
    fn defends(&mut self, stack: u8, defending: bool) {
        self.stacks
            .entry(stack)
            .and_modify(|s| s.defending = defending);
    }

    /// A move along `path` (its start first): a walk hex by hex, or a flight (§7.3); a plain move
    /// turns back at its end to the facing it started with (`turn_back`).
    fn moved(&mut self, stack: u8, path: &[u8], turn_back: bool) {
        let Some(mut live) = self.live(stack) else {
            return;
        };
        let home = live.right;
        let g = self.stage.groups;
        let tick = self.speed.scale(TICK);
        let n = live.look.count(g.moving);
        let step = self.speed.ratio(live.look.times[1], n);
        let start = path.first().copied().unwrap_or(live.hex);
        let end = path.last().copied().unwrap_or(live.hex);
        if live.flies {
            // A flight (0x4B4A40) turns first when it goes the other way by columns.
            let width = self.stage.width.max(1);
            let (from, to) = (start % width, end % width);
            if (to < from && live.right) || (to > from && !live.right) {
                live = self.turn(stack, live);
            }
            // Its sounds (0x4B4BD4): the start's with group 20, the move's (looping) after it, the
            // end's with group 21, when the move's stops (0x4B4E3B).
            self.cue(stack, Sound::Start, false);
            for f in 0..live.look.count(g.move_start) {
                self.frame(&[(stack, Self::posed(&live, g.move_start, f, start))], tick);
            }
            self.cue(stack, Sound::Move, false);
            // Its count box hidden from the cycles on (0x4B4BD8).
            live.boxless = true;
            // k cycles of the moving group along the straight line, the point moved on before
            // each frame by (Δ / k) / n, every value an f32 (0x4B4C7B to 0x4B4CEC), drawn at its
            // truncation (§7.9).
            let [ax, ay] = self.stage.anchor(start);
            let [bx, by] = self.stage.anchor(end);
            let (dx, dy) = (bx - ax, by - ay);
            let d = isqrt(i64::from(dx).pow(2) + i64::from(dy).pow(2));
            let distance = u64::from(live.look.times[3]);
            // k = (d + D / 2) / D; one without a flight distance.
            let cycles = (d + distance / 2).checked_div(distance).unwrap_or(1);
            let k = i128::from(cycles).max(1);
            let total = u32::try_from(cycles).unwrap_or(0) * n;
            let per_frame = |delta: i32| {
                let per_cycle = f32_round(i128::from(delta), k);
                f32_round(i128::from(per_cycle), i128::from(n.max(1)) << F32_FRAC)
            };
            let step_by = [per_frame(dx), per_frame(dy)];
            let mut at = [i64::from(ax) << F32_FRAC, i64::from(ay) << F32_FRAC];
            for j in 0..total {
                for (a, s) in at.iter_mut().zip(step_by) {
                    *a = f32_round(i128::from(*a) + i128::from(s), 1 << F32_FRAC);
                }
                let mut p = Self::posed(&live, g.moving, j % n, start);
                p.offset = [f32_trunc(at[0]) - ax, f32_trunc(at[1]) - ay];
                p.layer = FLYING;
                self.frame(&[(stack, p)], step);
            }
            // The flight ends with group 21 and a frame standing (0x4B4E59, 0x4B4E66); then
            // `stack_fly` turns a plain move back (0x4B49F5) and draws one more frame standing
            // (0x4B4A15).
            self.cue(stack, Sound::End, false);
            self.cue(stack, Sound::Move, true);
            for f in 0..live.look.count(g.move_end) {
                self.frame(&[(stack, Self::posed(&live, g.move_end, f, end))], tick);
            }
            live.hex = end;
            self.frame(&[(stack, self.standing(&live, end))], tick);
            // Shown again after that frame (0x4B4E73), with no redraw of its own: the next frame
            // draws it.
            live.boxless = false;
            self.stacks.insert(stack, live.clone());
            if turn_back && live.right != home {
                live = self.turn(stack, live);
            }
            let standing = self.standing(&live, end);
            self.frame(&[(stack, standing)], tick);
            return;
        }
        // A walk (`walk_step` 0x43F0B0): before each step the stack faces it, the first step's
        // turn animated, a later one at once (0x43F0E5); its layer by the step's direction
        // (0x43F15E: up 3, down 7); one cycle per hex, the stack drawn along the step. Its count
        // box is hidden from before the first turn (0x442145).
        live.boxless = true;
        for (i, pair) in path.windows(2).enumerate() {
            let [ax, ay] = self.stage.anchor(pair[0]);
            let [bx, by] = self.stage.anchor(pair[1]);
            if (bx > ax) != live.right {
                if i == 0 {
                    live = self.turn(stack, live);
                } else {
                    live.right = !live.right;
                }
            }
            let layer = match by.cmp(&ay) {
                Ordering::Less => WALK_UP,
                Ordering::Greater => WALK_DOWN,
                Ordering::Equal => STANDING,
            };
            if i == 0 {
                // The first step's sounds (0x43F19B): the start's with group 20, the move's
                // (looping) with the first cycle.
                self.cue(stack, Sound::Start, false);
                for f in 0..live.look.count(g.move_start) {
                    let p = Pose {
                        layer,
                        ..Self::posed(&live, g.move_start, f, start)
                    };
                    self.frame(&[(stack, p)], tick);
                }
                self.cue(stack, Sound::Move, false);
            }
            for f in 0..n {
                let mut p = Self::posed(&live, g.moving, f, pair[0]);
                p.offset = [share(bx - ax, f, n), share(by - ay, f, n)];
                p.layer = layer;
                self.frame(&[(stack, p)], step);
            }
        }
        // The last step (0x43F267): the end's sound with group 21, the move's stopped.
        self.cue(stack, Sound::End, false);
        self.cue(stack, Sound::Move, true);
        for f in 0..live.look.count(g.move_end) {
            self.frame(&[(stack, Self::posed(&live, g.move_end, f, end))], tick);
        }
        let standing = self.standing(&live, end);
        self.frame(&[(stack, standing)], tick);
        live.hex = end;
        self.stacks.insert(stack, live.clone());
        // `stack_walk` 0x4423A2: a plain move turns back at its end.
        if turn_back && live.right != home {
            live = self.turn(stack, live);
        }
        // Then its box shows again with a redraw of the whole battle, shown at once (0x4423D9,
        // `battle_redraw` 0x4423E7).
        live.boxless = false;
        self.stacks.insert(stack, live.clone());
        self.pending.push((stack, self.standing(&live, end)));
        self.redraw();
    }

    /// The struck stacks' reactions; their kills taken.
    fn reactions(&mut self, hit: &[(u8, u32)]) -> Vec<Reaction> {
        let g = self.stage.groups;
        let mut out = Vec::new();
        for &(stack, kills) in hit {
            let Some(live) = self.stacks.get_mut(&stack) else {
                continue;
            };
            let living = live.count > 0;
            live.count = live.count.saturating_sub(kills);
            let died = live.count == 0;
            let (group, sound) = if died {
                (g.death, Sound::Death)
            } else if live.defending {
                (g.defend, Sound::Defend)
            } else {
                (g.hit, Sound::Wince)
            };
            let frames = live.look.count(group);
            let live = live.clone();
            if died && living {
                self.happenings.push(Happening::Died(live.side));
            }
            out.push(Reaction {
                stack,
                group,
                frames,
                died,
                sound,
                base: self.standing(&live, live.hex),
            });
        }
        out
    }

    /// A melee strike of `attacker` on the stacks `hit` (§7.4), the striker in `layer`.
    fn strike(&mut self, attacker: u8, hit: &[(u8, u32)], layer: u8) {
        let Some(live) = self.live(attacker) else {
            self.react_alone(hit, [layer, STRUCK]);
            return;
        };
        let breath = self.breath(&live, hit);
        let reactions = self.reactions(hit);
        let own = self.stage.row(live.position(live.hex));
        let target_row = reactions
            .first()
            .map_or(own, |r| self.stage.row(r.base.hex));
        let direction = match target_row.cmp(&own) {
            Ordering::Less => 0,
            Ordering::Equal => 1,
            Ordering::Greater => 2,
        };
        // A breath on two stacks takes groups 17 to 19 if the DEF has them, else the strike's
        // (`melee_strike` 0x441ACA to 0x441B66).
        let g = self.stage.groups;
        let group = Some(g.breath[direction])
            .filter(|&b| breath && live.look.count(b) > 0)
            .unwrap_or(g.strike[direction]);
        let frames = live.look.count(group);
        self.iterate(
            Some((attacker, &live)),
            (group, 0, frames, frames),
            &reactions,
            [layer, STRUCK],
        );
    }

    /// Whether `live`'s strike on `hit` is a breath: a second stack struck stands beyond its
    /// neighbours. `melee_strike` strikes the stack behind its target only for a creature with
    /// flag 0x8 (0x44171B; its second stack [ebp − 8], 0x441A35); a multi-headed striker's
    /// stacks all stand beside it (the record does not name the ability; the recorded `fly`'s
    /// breath confirms it, E17 M3 T1b-2).
    fn breath(&self, live: &Live, hit: &[(u8, u32)]) -> bool {
        let near: Vec<u8> = live
            .covers(live.hex)
            .iter()
            .flat_map(|&h| (0..6).filter_map(move |d| self.stage.neighbor(h, d)))
            .collect();
        hit.iter()
            .skip(1)
            .filter_map(|(t, _)| self.stacks.get(t))
            .any(|t| t.covers(t.hex).iter().all(|h| !near.contains(h)))
    }

    /// The stacks `hit` by a striker or shooter the timeline does not know (a stack created in
    /// the battle, E17 M3 T1): their reactions alone, as after a projectile lands, their kills
    /// taken (E17 M2 T5c, the reviewer).
    fn react_alone(&mut self, hit: &[(u8, u32)], layers: [u8; 2]) {
        let reactions = self.reactions(hit);
        self.iterate(None, (0, 0, 0, 2), &reactions, layers);
    }

    /// A shot of `shooter` at the stacks `hit` (`shot_animation` 0x43F2C0, §7.4).
    fn shot(&mut self, shooter: u8, hit: &[(u8, u32)]) {
        let Some(live) = self.live(shooter) else {
            self.react_alone(hit, [STANDING, STANDING]);
            return;
        };
        let target = hit.first().and_then(|(t, _)| self.live(*t));
        let aimed = target.is_some();
        let aim = target.map_or_else(Aim::default, |t| self.aim(&live, &t));
        let group = self.stage.groups.shoot[aim.direction];
        let n = live.look.count(group);
        let climax = if live.look.climax == 0 || live.look.climax > n {
            n
        } else {
            live.look.climax
        };
        let time = self.speed.scale(live.look.times[2] / climax.max(1));
        // The shot's sound with its first frame (0x43F424).
        self.cue(shooter, Sound::Shot, false);
        for f in 0..climax {
            self.frame(&[(shooter, Self::posed(&live, group, f, live.hex))], time);
        }
        if let Some(colours) = live.look.ray.clone().filter(|_| aimed) {
            self.beam((shooter, &live), &aim, colours, (group, climax, n));
            let reactions = self.reactions(hit);
            // After the ray the shooter's group is over: it stands as the loop starts
            // (*provisional*, the recorded `shots`).
            self.iterate(
                Some((shooter, &live)),
                (group, n, 0, 2),
                &reactions,
                [STANDING, STANDING],
            );
            return;
        }
        let step = self.speed.scale(STEP);
        if let Some(missile) = live.look.missile.as_ref().filter(|_| aim.steps > 0) {
            // 0x43F5D6: the canvas centred on the start, moved by the distance over the steps
            // (each truncated toward zero), mirrored when the target lies left of the start.
            let [w, h] = missile.size;
            let steps = i32::try_from(aim.steps).unwrap_or(i32::MAX);
            let [dx, dy] = [aim.to[0] - aim.from[0], aim.to[1] - aim.from[1]];
            self.flights.push(Flight {
                def: missile.def.clone(),
                frame: aim.frame(missile),
                mirrored: aim.to[0] < aim.from[0],
                from: self.clock.now,
                step,
                steps: aim.steps,
                start: [aim.from[0] - w / 2, aim.from[1] - h / 2],
                delta: [dx / steps, dy / steps],
            });
        }
        self.clock.wait(aim.steps * step);
        let reactions = self.reactions(hit);
        // A shot marks no striker (`melee_strike` does, 0x441A87; `stack_shot` does not), so
        // `strike_animation`'s loop counts the reactions alone: the longest starts as the
        // projectile lands, whatever is left of the shot group (E17 M2 T5b, design §7.4). The
        // shooter, still in its shot group (0x468A55), shows its frames from the climax on beside
        // it, one per iteration, its frame advancing before each draw, then stands, the loop
        // going on until it has (the Cyclops, sequence `shots`). The reactions' part counts two
        // striker frames, so it has one iteration more than the longest reaction's frames
        // (L = R + 1: the Marksmen's recorded sequence, §7.9; the reading's L = R,
        // *provisional*); its first iterations show at once, the deadline being behind. A shot's
        // layers stay (not read).
        self.iterate(
            Some((shooter, &live)),
            (group, climax, n - climax, 2),
            &reactions,
            [STANDING, STANDING],
        );
    }

    /// A ray shooter's shot after its climax frames (0x43F51D, `DrawRay` 0x5A5C20, design
    /// §7.12): the wait of trunc(115 F); the ray's ticks on its own clock, each trunc(10 F);
    /// the battle redrawn without the ray at once, the frame deadline left as it is; then, if the
    /// creature has a climax frame (+0x150), the rest of its shot group from the frame shown on,
    /// each trunc(A / n) on the deadline, the last drawn but not shown (0x5A6136 to 0x5A61C9).
    fn beam(
        &mut self,
        (shooter, live): (u8, &Live),
        aim: &Aim,
        colours: Vec<[u8; 3]>,
        (group, climax, n): (u8, u32, u32),
    ) {
        self.clock.wait(self.speed.scale(RAY_WAIT));
        let ticks = ray::trace(aim.from, aim.to);
        let step = self.speed.scale(RAY_TICK);
        let count = u32::try_from(ticks.len()).unwrap_or(0);
        self.beams.push(Beam {
            from: self.clock.now,
            step,
            ticks,
            colours,
        });
        self.clock.wait(count * step);
        self.redraw();
        if live.look.climax != 0 {
            let time = self.speed.scale(live.look.times[2] / n.max(1));
            for f in climax.saturating_sub(1)..n {
                let pose = Self::posed(live, group, f, live.hex);
                if f + 1 < n {
                    self.frame(&[(shooter, pose)], time);
                } else {
                    // Drawn, never shown: the timeline has it as drawn (the original's memory).
                    let drawn = self.clock.now;
                    self.clock.show(time);
                    self.keys.push(Key {
                        stack: shooter,
                        drawn,
                        shown: u32::MAX,
                        pose,
                    });
                }
            }
        }
    }

    /// The shot's group (0 up, 1 straight, 2 down), its start and target points and its
    /// projectile's steps (0x469880, 0x43F4A7).
    fn aim(&self, shooter: &Live, target: &Live) -> Aim {
        let [sx, sy] = self.stage.anchor(shooter.position(shooter.hex));
        let start = |i: usize| {
            let [x, y] = shooter.look.missile_offsets[i];
            let x = i32::from(x);
            [
                if shooter.right { sx + x } else { sx - x },
                sy + i32::from(y),
            ]
        };
        // A two-hex target is aimed at between its hexes, by its facing (0x43F323).
        let [tx, ty] = self.stage.anchor(target.position(target.hex));
        let tx = match (target.two_hex, target.right) {
            (false, _) => tx,
            (true, true) => tx + HALF_HEX,
            (true, false) => tx - HALF_HEX,
        };
        let ty = ty - (self.stage.layout.stack_canvas[1] - target.look.top) / 2;
        let [x, y] = start(1);
        let (dx, dy) = (i64::from(tx - x).abs(), i64::from(ty - y));
        // atan(−dy / |dx|) in degrees; straight up or down the sign is inverted (0x469880).
        let direction = if dx == 0 {
            if dy > 0 { 0 } else { 2 }
        } else if -dy * TAN_SCALE > dx * TAN_25 {
            0
        } else if -dy * TAN_SCALE > -dx * TAN_25 {
            1
        } else {
            2
        };
        let [x, y] = start(direction);
        let d = isqrt(i64::from(tx - x).pow(2) + i64::from(ty - y).pow(2));
        let steps = (d + STEP_PIXELS / 2) / STEP_PIXELS;
        Aim {
            direction,
            steps: u32::try_from(steps).unwrap_or(0),
            from: [x, y],
            to: [tx, ty],
            slope: [dx, dy],
        }
    }

    /// The strike's iterations (`strike_animation` 0x468990): the striker (its id and how the
    /// timeline follows it; none: one it does not know) plays `frames` of `group` from frame
    /// `from` and counts `counted` for the loop's length, each reaction ends with the loop; the
    /// striker and the struck stacks drawn in `layers`. Then everyone stands in standing's
    /// layer, a dead stack on its death's last frame among the corpses, shown with the next
    /// frame (*provisional*: the executable moves it to the corpses' list when its death ends).
    fn iterate(
        &mut self,
        striker: Option<(u8, &Live)>,
        (group, from, frames, counted): (u8, u32, u32, u32),
        reactions: &[Reaction],
        [striking, struck]: [u8; 2],
    ) {
        // L = A + R − 1 (with a magic animation, at least its frames: not yet); at least A + 1,
        // so a striker without a reaction to wait for still stands at the end (the original
        // always has one: a target's reaction has six frames or more). The reactions end with
        // those; a shooter plays the rest of its group out beyond them and stands after its last
        // frame (the Cyclops' 10 frames after a 6-frame hit, sequence `shots`, E17 M3 T1b-1;
        // *provisional*: not read).
        let longest = reactions.iter().map(|r| r.frames).max().unwrap_or(0);
        let reacting = (counted + longest).saturating_sub(1).max(counted + 1);
        let loops = reacting.max(frames + 1);
        let tick = self.speed.scale(TICK);
        let striker = striker.map(|(id, live)| (id, live, self.standing(live, live.hex)));
        for i in 0..loops {
            let mut poses = Vec::with_capacity(reactions.len() + 1);
            if let Some((id, live, standing)) = striker {
                // Each stack's sound as its group starts (0x468F00 to 0x468F8D): the striker's
                // attack (a shooter goes on with its group: none), each reaction's by its group.
                if i == 0 && from == 0 {
                    self.cue(id, Sound::Attack, false);
                }
                let pose = if i < frames {
                    Self::posed(live, group, from + i, live.hex)
                } else {
                    standing
                };
                poses.push((
                    id,
                    Pose {
                        layer: striking,
                        ..pose
                    },
                ));
            }
            for r in reactions {
                if i == (reacting - 1).saturating_sub(r.frames) {
                    self.cue(r.stack, r.sound, false);
                }
            }
            for r in reactions {
                // It reacts once the iterations left are no more than its frames.
                let k = i64::from(i) - (i64::from(reacting) - 1 - i64::from(r.frames));
                let frame = if (0..i64::from(r.frames)).contains(&k) {
                    u32::try_from(k).ok()
                } else if r.died && k >= 0 {
                    r.frames.checked_sub(1)
                } else {
                    None
                };
                let pose = frame.map_or(r.base, |f| Pose {
                    group: u32::from(r.group),
                    frame: f,
                    ..r.base
                });
                poses.push((
                    r.stack,
                    Pose {
                        layer: struck,
                        ..pose
                    },
                ));
            }
            self.frame(&poses, tick);
        }
        if let Some((id, _, standing)) = striker {
            self.pending.push((id, standing));
        }
        for r in reactions {
            let pose = match (r.died, r.frames.checked_sub(1)) {
                (true, Some(last)) => Pose {
                    group: u32::from(r.group),
                    frame: last,
                    layer: CORPSE,
                    ..r.base
                },
                (true, None) => Pose {
                    layer: CORPSE,
                    ..r.base
                },
                (false, _) => r.base,
            };
            self.pending.push((r.stack, pose));
            if r.died {
                self.stacks.remove(&r.stack);
            }
        }
    }
}

/// The fractional bits of the fixed-point integers that hold f32 values exactly here: every f32
/// the flights compute (|x| < 2^23, at least 2^-16 unless 0) is a multiple of 2^-40.
const F32_FRAC: u32 = 40;

/// `num` / `den` (`den` > 0) rounded to the nearest f32, ties to even, as a fixed-point integer
/// (the value × 2^[`F32_FRAC`]): what the x87 stores with `fstp dword` after an operation on
/// f32 operands (the 53-bit intermediate rounds the same, §7.9). A value too small for the
/// fixed point is cut to its grid.
fn f32_round(num: i128, den: i128) -> i64 {
    /// Guard bits below the fixed point's grid.
    const GUARD: u32 = 8;
    let negative = (num < 0) != (den < 0);
    let (num, den) = (num.unsigned_abs(), den.unsigned_abs().max(1));
    let scaled = num << (F32_FRAC + GUARD);
    let (q, sticky) = (scaled / den, !scaled.is_multiple_of(den));
    let bits = 128 - q.leading_zeros();
    let shift = bits.saturating_sub(24).max(GUARD);
    let (kept, rest) = (q >> shift, q & ((1 << shift) - 1));
    let half = 1 << (shift - 1);
    let up = rest > half || (rest == half && (sticky || kept & 1 == 1));
    let magnitude = (kept + u128::from(up)) << (shift - GUARD);
    let magnitude = i64::try_from(magnitude).unwrap_or(i64::MAX);
    if negative { -magnitude } else { magnitude }
}

/// A fixed-point f32's truncation toward zero (`crt_ftol`).
fn f32_trunc(value: i64) -> i32 {
    i32::try_from(value / (1 << F32_FRAC)).unwrap_or(0)
}

/// tan(`half_degrees` / 2 degrees) × [`TAN_SCALE`], truncated: `i64::MAX` from 90° on,
/// `i64::MIN` from −90° down. Sine and cosine by their series in fixed point (no platform
/// transcendental, ADR-0003 §3), exact to about 10^-14.
fn tangent(half_degrees: i32) -> i64 {
    /// 1 and π, times 10^18.
    const ONE: i128 = 1_000_000_000_000_000_000;
    const PI: i128 = 3_141_592_653_589_793_238;
    match half_degrees {
        180.. => i64::MAX,
        ..=-180 => i64::MIN,
        h => {
            let x = i128::from(h) * PI / 360;
            let (mut sin, mut cos, mut term) = (0, 0, ONE);
            for n in 0..40 {
                match n % 4 {
                    0 => cos += term,
                    1 => sin += term,
                    2 => cos -= term,
                    _ => sin -= term,
                }
                term = term * x / ONE / (n + 1);
            }
            i64::try_from(sin * i128::from(TAN_SCALE) / cos).unwrap_or(0)
        }
    }
}

/// ⌊√`n`⌋ (`n` ≥ 0): the executable's `crt_sqrt` truncated, which the steps' and cycles'
/// rounding needs only to the integer.
fn isqrt(n: i64) -> u64 {
    u64::try_from(n).unwrap_or(0).isqrt()
}

/// A timeline's lines that differ from `expected`'s (the harness's format): each differing
/// line with `-` for the expected and `+` for ours.
pub fn differences(ours: &str, expected: &str) -> Vec<String> {
    let mut out = Vec::new();
    let (a, b): (Vec<&str>, Vec<&str>) = (expected.lines().collect(), ours.lines().collect());
    for i in 0..a.len().max(b.len()) {
        match (a.get(i), b.get(i)) {
            (Some(x), Some(y)) if x == y => {}
            (x, y) => {
                out.extend(x.map(|x| format!("- {x}")));
                out.extend(y.map(|y| format!("+ {y}")));
            }
        }
    }
    out
}

#[cfg(test)]
pub(crate) mod tests;
