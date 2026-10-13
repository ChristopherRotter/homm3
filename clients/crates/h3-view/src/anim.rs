//! The scene over time (E17 M2 T4a; `docs/epics/E17-design.md` §7): a record's commands become
//! each stack's animation at the original's timing. [`timeline`] plays the events of each
//! command on the original's clock (the frame deadline on its grid, each group's frame time, the
//! strike's iterations, the shot's frames and projectile, the waits outside the deadline) and
//! keeps every pose a stack takes with the time it is drawn and the time it shows.
//! [`Timeline::at`] is what shows at any time, [`Timeline::lines`] the capture harness's
//! timeline format (design §7.6), which the recorded sequences check.
//!
//! *Provisional* (the M2 review list): the next command's default start, the shooter's frames in
//! the strike loop, the walk's and the flight's pixel offsets, the flight's cycles rounding.
//! Not yet (T4b): fidgets and the idle step, turns, the draw layers of walks and strikes, the
//! projectile's and the spells' sprites, deaths other than by a strike, created stacks.

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt::Write as _;

use h3_protocol::wire::{Event, Observation, Side};

use crate::layout::BattleLayout;
use crate::tables::Groups;

/// Every frame outside a walk's group and the shot's: 100 ms times the factor (§7.1).
const TICK: u32 = 100;
/// The wait before a retaliation or a second strike, outside the deadline (§7.4).
const PAUSE: u32 = 150;
/// A projectile's step, outside the deadline (§7.4).
const STEP: u32 = 33;
/// A projectile's step length in pixels: trunc((d + 20) / 40) steps (§7.4).
const STEP_PIXELS: u64 = 40;
/// Half a hex: a two-hex target is aimed at from its middle (0x43F323).
const HALF_HEX: i32 = 22;
/// tan(25°) × 10^15: a shot is up above 25°, down below −25° (0x469880).
const TAN_25: i64 = 466_307_658_154_999;
/// The scale of [`TAN_25`].
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

/// The poses of a record's stacks over time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Timeline {
    /// The stacks living at the start, by id.
    pub members: Vec<Member>,
    /// Every pose taken, in the order drawn.
    pub keys: Vec<Key>,
    /// The active stack from a time on (its turn started then).
    pub turns: Vec<(u32, u8)>,
    /// When the last frame shows.
    pub end: u32,
}

impl Timeline {
    /// Each stack's pose shown at `t`.
    pub fn at(&self, t: u32) -> BTreeMap<u8, Pose> {
        let mut poses: BTreeMap<u8, Pose> =
            self.members.iter().map(|m| (m.stack, m.start)).collect();
        for key in self.keys.iter().filter(|k| k.shown <= t) {
            poses.insert(key.stack, key.pose);
        }
        poses
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
    /// at 0; a change at a time is the last pose drawn then.
    pub fn lines(&self) -> String {
        let mut now: BTreeMap<u8, Pose> = self.members.iter().map(|m| (m.stack, m.start)).collect();
        let mut printed: BTreeMap<u8, (u32, u32, u8)> = BTreeMap::new();
        let mut out = String::new();
        let mut times: Vec<u32> = self.keys.iter().map(|k| k.drawn).collect();
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
    hex: u8,
    count: u32,
    flies: bool,
    two_hex: bool,
    defending: bool,
    look: Look,
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
/// opening); a later one without a time when the previous one's last frame showed, one tick
/// later if it drew any (*provisional*: in the melee sequence the computer acts 100 ms after the
/// Angels' last frame shows; after Wait or Defend the next stack acts at once, §7.4).
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
    let standing = u32::from(stage.groups.standing);
    let mut members = Vec::new();
    let mut stacks = BTreeMap::new();
    let mut records = [0u8; 2];
    for s in snapshot.stacks.iter().filter(|s| s.alive) {
        let look = looks.get(&s.creature).ok_or(AnimError::Look(s.creature))?;
        let side = usize::from(s.side == Side::Defender);
        members.push(Member {
            stack: s.id,
            side: s.side,
            record: records[side],
            start: still(standing, s.hex),
        });
        records[side] += 1;
        stacks.insert(
            s.id,
            Live {
                side: s.side,
                hex: s.hex,
                count: s.count,
                flies: s.flies,
                two_hex: s.two_hex,
                defending: s.defending,
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
        turns: Vec::new(),
    };
    let mut drew = false;
    for (i, command) in commands.iter().enumerate() {
        let default = match (i, drew) {
            (0, _) => 0,
            (_, true) => player.clock.now + grid,
            (_, false) => player.clock.now,
        };
        player.clock.start(command.at.unwrap_or(default));
        let before = player.keys.len();
        player.command(command.events);
        drew = player.keys.len() > before;
    }
    Ok(Timeline {
        members,
        keys: player.keys,
        turns: player.turns,
        end: player.clock.now,
    })
}

/// A still pose: frame 0 of `group` on `hex`.
const fn still(group: u32, hex: u8) -> Pose {
    Pose {
        group,
        frame: 0,
        hex,
        offset: [0, 0],
    }
}

/// A frame of `group` on `hex`.
fn pose(group: u8, frame: u32, hex: u8) -> Pose {
    Pose {
        group: u32::from(group),
        frame,
        hex,
        offset: [0, 0],
    }
}

/// `part` of `whole` pixels, `of` the way (`of` > 0).
fn share(whole: i32, part: u32, of: u32) -> i32 {
    let v = i64::from(whole) * i64::from(part) / i64::from(of.max(1));
    i32::try_from(v).unwrap_or(0)
}

/// A struck stack and how it reacts: its reaction's group and frames, and whether it died.
#[derive(Debug, Clone, Copy)]
struct Reaction {
    stack: u8,
    group: u8,
    frames: u32,
    died: bool,
    hex: u8,
}

/// Plays the commands.
#[derive(Debug)]
struct Player<'a> {
    stage: Stage<'a>,
    speed: Speed,
    clock: Clock,
    stacks: BTreeMap<u8, Live>,
    keys: Vec<Key>,
    turns: Vec<(u32, u8)>,
}

impl Player<'_> {
    /// One frame: `poses` drawn now, shown at the deadline; the deadline then moves on by `time`.
    fn frame(&mut self, poses: &[(u8, Pose)], time: u32) {
        let drawn = self.clock.now;
        let shown = self.clock.show(time);
        self.keys.extend(poses.iter().map(|&(stack, pose)| Key {
            stack,
            drawn,
            shown,
            pose,
        }));
    }

    /// The standing pose of `stack` on `hex`.
    fn standing(&self, hex: u8) -> Pose {
        still(u32::from(self.stage.groups.standing), hex)
    }

    /// One command's events, in order.
    fn command(&mut self, events: &[Event]) {
        let mut i = 0;
        // A melee strike of this command played: the next one waits (§7.4).
        let mut struck = false;
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
                    self.moved(stack, &path);
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
                        self.shot(attacker, &hit);
                    } else {
                        if struck {
                            self.clock.wait(self.speed.scale(PAUSE));
                        }
                        struck = true;
                        self.strike(attacker, &hit);
                    }
                }
                Event::Struck { target, kills, .. } => {
                    self.reactions(&[(target, kills)]);
                }
                Event::Defended { stack } => self.defends(stack, true),
                Event::TurnStarted { stack } => {
                    self.defends(stack, false);
                    self.turns.push((self.clock.now, stack));
                }
                _ => {}
            }
        }
    }

    /// Sets whether `stack` defends.
    fn defends(&mut self, stack: u8, defending: bool) {
        self.stacks
            .entry(stack)
            .and_modify(|s| s.defending = defending);
    }

    /// A move along `path` (its start first): a walk hex by hex, or a flight (§7.3).
    fn moved(&mut self, stack: u8, path: &[u8]) {
        let Some(live) = self.stacks.get(&stack).cloned() else {
            return;
        };
        let g = self.stage.groups;
        let tick = self.speed.scale(TICK);
        let n = live.look.count(g.moving);
        let step = self.speed.ratio(live.look.times[1], n);
        let start = path.first().copied().unwrap_or(live.hex);
        let end = path.last().copied().unwrap_or(live.hex);
        for f in 0..live.look.count(g.move_start) {
            self.frame(&[(stack, pose(g.move_start, f, start))], tick);
        }
        if live.flies {
            // A flight (0x4B4A40): k cycles of the moving group along the straight line.
            let [ax, ay] = self.stage.anchor(start);
            let [bx, by] = self.stage.anchor(end);
            let (dx, dy) = (bx - ax, by - ay);
            let d = isqrt(i64::from(dx).pow(2) + i64::from(dy).pow(2));
            let distance = u64::from(live.look.times[3]);
            let cycles = if distance == 0 {
                1
            } else {
                (d + distance / 2) / distance
            };
            let total = u32::try_from(cycles).unwrap_or(0) * n;
            for j in 0..total {
                let mut p = pose(g.moving, j % n, start);
                p.offset = [share(dx, j, total), share(dy, j, total)];
                self.frame(&[(stack, p)], step);
            }
        } else {
            // A walk (`walk_step` 0x43F0B0): one cycle per hex, the stack drawn along the step.
            for pair in path.windows(2) {
                let [ax, ay] = self.stage.anchor(pair[0]);
                let [bx, by] = self.stage.anchor(pair[1]);
                for f in 0..n {
                    let mut p = pose(g.moving, f, pair[0]);
                    p.offset = [share(bx - ax, f, n), share(by - ay, f, n)];
                    self.frame(&[(stack, p)], step);
                }
            }
        }
        for f in 0..live.look.count(g.move_end) {
            self.frame(&[(stack, pose(g.move_end, f, end))], tick);
        }
        let standing = self.standing(end);
        self.frame(&[(stack, standing)], tick);
        self.stacks.entry(stack).and_modify(|s| s.hex = end);
    }

    /// The struck stacks' reactions; their kills taken.
    fn reactions(&mut self, hit: &[(u8, u32)]) -> Vec<Reaction> {
        let g = self.stage.groups;
        let mut out = Vec::new();
        for &(stack, kills) in hit {
            let Some(live) = self.stacks.get_mut(&stack) else {
                continue;
            };
            live.count = live.count.saturating_sub(kills);
            let died = live.count == 0;
            let group = if died {
                g.death
            } else if live.defending {
                g.defend
            } else {
                g.hit
            };
            out.push(Reaction {
                stack,
                group,
                frames: live.look.count(group),
                died,
                hex: live.hex,
            });
        }
        out
    }

    /// A melee strike of `attacker` on the stacks `hit` (§7.4).
    fn strike(&mut self, attacker: u8, hit: &[(u8, u32)]) {
        let reactions = self.reactions(hit);
        let Some(live) = self.stacks.get(&attacker).cloned() else {
            return;
        };
        let target_row = reactions
            .first()
            .map_or(self.stage.row(live.hex), |r| self.stage.row(r.hex));
        let direction = match target_row.cmp(&self.stage.row(live.hex)) {
            Ordering::Less => 0,
            Ordering::Equal => 1,
            Ordering::Greater => 2,
        };
        let group = self.stage.groups.strike[direction];
        let frames = live.look.count(group);
        self.iterate(attacker, live.hex, (group, 0, frames), &reactions);
    }

    /// A shot of `shooter` at the stacks `hit` (`shot_animation` 0x43F2C0, §7.4).
    fn shot(&mut self, shooter: u8, hit: &[(u8, u32)]) {
        let Some(live) = self.stacks.get(&shooter).cloned() else {
            return;
        };
        let target = hit.first().and_then(|(t, _)| self.stacks.get(t)).cloned();
        let (direction, steps) = target.map_or((1, 0), |t| self.aim(&live, &t));
        let group = self.stage.groups.shoot[direction];
        let n = live.look.count(group);
        let climax = if live.look.climax == 0 || live.look.climax > n {
            n
        } else {
            live.look.climax
        };
        let time = self.speed.scale(live.look.times[2] / climax.max(1));
        for f in 0..climax {
            self.frame(&[(shooter, pose(group, f, live.hex))], time);
        }
        self.clock.wait(steps * self.speed.scale(STEP));
        let reactions = self.reactions(hit);
        // The shooter plays its group out from the climax frame (*provisional*: what the shot
        // sequence fits; the loop's first iterations show at once, the deadline being behind).
        let from = climax.saturating_sub(1);
        self.iterate(shooter, live.hex, (group, from, n - from), &reactions);
    }

    /// The shot's group (0 up, 1 straight, 2 down) and its projectile's steps (0x469880).
    fn aim(&self, shooter: &Live, target: &Live) -> (usize, u32) {
        let [sx, sy] = self.stage.anchor(shooter.hex);
        let right = shooter.side == Side::Attacker;
        let start = |i: usize| {
            let [x, y] = shooter.look.missile_offsets[i];
            let x = i32::from(x);
            [if right { sx + x } else { sx - x }, sy + i32::from(y)]
        };
        let [tx, ty] = self.stage.anchor(target.hex);
        let tx = match (target.two_hex, target.side) {
            (false, _) => tx,
            (true, Side::Attacker) => tx + HALF_HEX,
            (true, Side::Defender) => tx - HALF_HEX,
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
        (direction, u32::try_from(steps).unwrap_or(0))
    }

    /// The strike's iterations (`strike_animation` 0x468990): the striker plays `frames` of
    /// `group` from frame `from`, each reaction ends with the loop; then everyone stands, a dead
    /// stack on its death's last frame.
    fn iterate(
        &mut self,
        striker: u8,
        hex: u8,
        (group, from, frames): (u8, u32, u32),
        reactions: &[Reaction],
    ) {
        let longest = reactions.iter().map(|r| r.frames).max().unwrap_or(0);
        let loops = (frames + longest).saturating_sub(1).max(1);
        let tick = self.speed.scale(TICK);
        for i in 0..loops {
            let mut poses = Vec::with_capacity(reactions.len() + 1);
            let striking = if i < frames {
                pose(group, from + i, hex)
            } else {
                self.standing(hex)
            };
            poses.push((striker, striking));
            for r in reactions {
                // It reacts once the iterations left are no more than its frames.
                let k = i64::from(i) - (i64::from(loops) - 1 - i64::from(r.frames));
                let frame = if (0..i64::from(r.frames)).contains(&k) {
                    u32::try_from(k).ok()
                } else if r.died && k >= 0 {
                    r.frames.checked_sub(1)
                } else {
                    None
                };
                let p = frame.map_or_else(|| self.standing(r.hex), |f| pose(r.group, f, r.hex));
                poses.push((r.stack, p));
            }
            self.frame(&poses, tick);
        }
        for r in reactions.iter().filter(|r| r.died) {
            self.stacks.remove(&r.stack);
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
mod tests;
