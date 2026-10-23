//! A siege's shots over time (E17 M4 T2b; `docs/epics/E17-design.md` §10.2): a turret's shot
//! (`turret_shot` 0x465AD0) on the turret's own record, and a wall shot (`wall_shot_animate`
//! 0x445FD0): the shooter's frames, the rock's arc (`catapult_rock_flight` 0x467A00), the
//! landing's explosion with the part's new state at its frame 5, the whole redraw. The original's
//! slips (CLT-Q19 (1) to (5)) under [`Bugs::Original`], fixed in what the client draws.

use alloc::string::String;
use alloc::vec::Vec;

use h3_render::Bugs;

use super::{
    Flight, Key, Live, Player, STANDING, STRUCK, Sound, TAN_25, TAN_SCALE, TICK, isqrt, launch,
};
use crate::battle_log::Logged;
use crate::siege::{GATE_DESTROYED, Walls};

/// What a siege's shots draw besides the creatures (E17 M4 T2b): the turrets' points and the
/// explosions a wall shot lands in.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SiegeStage {
    /// Each turret record's point, its shooter's anchor and its shot's origin (0 the keep, 1 the
    /// lower tower, 2 the upper; the town's `siege_turrets_by_town` 0x63CF88).
    pub turrets: [[i32; 2]; 3],
    /// The explosion a hit lands in (`SGEXPL.DEF`, 0x4462DC).
    pub hit: Burst,
    /// The one a shot of no damage lands in (`CSGRCK.DEF`).
    pub miss: Burst,
}

/// An explosion's DEF: its name, its canvas's width and height, the frames of its group 0.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Burst {
    /// The DEF.
    pub def: String,
    /// Its canvas's width and height (the DEF's +0x30, +0x34).
    pub size: [i32; 2],
    /// The frames of its group 0.
    pub frames: u32,
}

/// The wall-shot targets' points (`wall_shot_targets` 0x63BE60, the same in every town): the
/// upper tower, the upper wall, the mid-upper wall, the gate, the mid-lower wall, the lower
/// wall, the lower tower, the keep.
pub const TARGET_POINTS: [[i32; 2]; 8] = [
    [586, 48],
    [564, 128],
    [520, 212],
    [498, 296],
    [520, 380],
    [586, 506],
    [630, 506],
    [762, 212],
];

/// The wall-shot targets' parts (`wall_shot_targets`' +0x8): target `t`'s part is entry `t`.
const TARGET_PARTS: [usize; 8] = [5, 6, 8, 9, 10, 12, 13, 14];
/// The gate's target.
const GATE_TARGET: usize = 3;
/// The covers of the targets that carry a turret: the upper tower's, the lower tower's, the
/// keep's (`wall_part_damage` 0x465A1F to 0x465A53).
const COVERS: [(usize, usize); 3] = [(0, 17), (6, 16), (7, 15)];
/// A part's state after a hit that leaves it hit points (`wall_part_damage` 0x465A9D).
const HIT_STATE: u8 = 1;
/// The explosion's frame at which the part takes its damage (0x4463B5).
const DAMAGE_FRAME: u32 = 5;
/// The rock's steps: one per 20 pixels of the distance, rounded (0x467AE1).
const ROCK_PIXELS: i64 = 20;

/// The turret record of a turret stack's position (`turret_shot` 0x465B0B): the keep's (254)
/// 0, the lower tower's (251) 1, the upper tower's (255) 2.
#[must_use]
pub const fn turret_record(hex: u8) -> Option<u8> {
    match hex {
        254 => Some(0),
        251 => Some(1),
        255 => Some(2),
        _ => None,
    }
}

/// A turret's shooter as its record holds it (+0x14 to +0x1C): its group, frame and facing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurretPose {
    /// The DEF group.
    pub group: u32,
    /// The frame in it.
    pub frame: u32,
    /// It faces right (+0x14 = 1): drawn unmirrored (§10.2).
    pub right: bool,
}

/// A turret record's pose drawn at `drawn` (memory), shown from `shown`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurretKey {
    /// The record (0 the keep, 1 the lower tower, 2 the upper).
    pub record: u8,
    /// When it is drawn.
    pub drawn: u32,
    /// When it shows ([`u32::MAX`]: never).
    pub shown: u32,
    /// The pose.
    pub pose: TurretPose,
}

/// The walls' picture states from `drawn` on in memory, shown from `shown` (E17 M4 T2b: a wall
/// shot's damage at its explosion's frame 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WallKey {
    /// When memory holds them.
    pub drawn: u32,
    /// When the whole screen shows them.
    pub shown: u32,
    /// The states.
    pub walls: Walls,
}

/// A frame of a wall shot's explosion shown from `shown`, or (`frame` none) the explosion gone
/// (§10.2, step 5): `blast` numbers the explosion, `walls` the states its redraw draws.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlastKey {
    /// The explosion's number in the timeline.
    pub blast: usize,
    /// When it shows.
    pub shown: u32,
    /// The frame of group 0, none once gone.
    pub frame: Option<u32>,
    /// The DEF.
    pub def: String,
    /// The canvas's top-left.
    pub at: [i32; 2],
    /// The rectangle shown (the canvas clipped to the battlefield; left, top, right, bottom,
    /// exclusive).
    pub rect: [i32; 4],
    /// The walls' states the frame's redraw draws.
    pub walls: Walls,
    /// Each frame drawn on the background restored (the slip fixed, CLT-Q19 (4)); else over the
    /// frames before it.
    pub restored: bool,
}

/// An explosion as it shows at a time: in its rectangle the frames drawn so far, each over a
/// redraw of the battle with its walls' states (the original's, unrestored), or the last alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blast {
    /// The DEF.
    pub def: String,
    /// The canvas's top-left.
    pub at: [i32; 2],
    /// The rectangle shown.
    pub rect: [i32; 4],
    /// The frames in the order drawn, each with the walls' states its redraw drew.
    pub frames: Vec<(u32, Walls)>,
}

/// What a wall shot leaves the walls: the picture states and each target's hit points.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Siege {
    pub(super) walls: Walls,
    pub(super) hit_points: [u8; 8],
}

impl Siege {
    /// `damage` on target `t` (`wall_part_damage` 0x465990): its hit points less it, at least 0;
    /// its part's state 1, or 0 once destroyed, a destroyed tower's or keep's cover 0 too and the
    /// gate's state destroyed. False for no such target.
    pub(super) fn damage(&mut self, t: usize, damage: u8) -> bool {
        let (Some(hp), Some(&part)) = (self.hit_points.get_mut(t), TARGET_PARTS.get(t)) else {
            return false;
        };
        *hp = hp.saturating_sub(damage);
        let destroyed = *hp == 0;
        self.walls.parts[part] = if destroyed { 0 } else { HIT_STATE };
        if destroyed {
            if let Some(&(_, cover)) = COVERS.iter().find(|c| c.0 == t) {
                self.walls.parts[cover] = 0;
            }
            if t == GATE_TARGET {
                self.walls.gate = GATE_DESTROYED;
            }
        }
        true
    }
}

impl Player<'_> {
    /// A turret's frame `pose` of `record` drawn now, shown at the deadline (a redraw of its
    /// rectangle through the deadline, `time` on), or never (`shown` false: its rectangle no
    /// longer dirty, CLT-Q19 (5)).
    fn turret_frame(&mut self, record: u8, pose: TurretPose, time: u32, shown: bool) {
        let drawn = self.clock.now;
        let at = self.clock.show(time);
        self.flush(drawn, at);
        self.turret_keys.push(TurretKey {
            record,
            drawn,
            shown: if shown { at } else { u32::MAX },
            pose,
        });
    }

    /// A turret's shot at the stacks `hit` (`turret_shot` 0x465AD0, §10.2): `turret` its stack on
    /// `record`, `line` the damage's line. Its facing by the target's side of its point, the
    /// group and the start by the shot's angle from there; the creature's shot sound by its file
    /// (category 2); the frames to the climax through the deadline; the arrow on its own clock;
    /// the line, the reactions without a striker; the rest of the group drawn, shown only with
    /// the slip fixed; then it stands facing left in a whole redraw shown at once.
    pub(super) fn turret_shot(
        &mut self,
        (turret, record): (u8, u8),
        hit: &[(u8, u32)],
        line: Logged,
    ) {
        let point = self
            .stage
            .siege
            .map_or([0, 0], |s| s.turrets[usize::from(record)]);
        let live = self.live(turret);
        let target = hit.first().and_then(|(t, _)| self.live(*t));
        let (Some(live), Some(target)) = (live, target) else {
            self.log.push((self.clock.now, line));
            self.react_alone(hit, [STANDING, STRUCK]);
            return;
        };
        let to = self.centre(&target);
        let right = to[0] >= point[0];
        let aim = self.aim_from(point, right, &live.look, &target);
        let group = self.stage.groups.shoot[aim.direction];
        let n = live.look.count(group);
        let climax = live.look.climax_in(n);
        let time = self.speed.scale(live.look.times[2] / climax.max(1));
        self.cue(turret, Sound::TurretShot, false);
        let pose = |frame| TurretPose {
            group: u32::from(group),
            frame,
            right,
        };
        for f in 0..climax {
            self.turret_frame(record, pose(f), time, true);
        }
        self.fly(&live.look, &aim);
        // The line before the reactions (0x465D9E), which play without a striker (0x465DB2).
        self.log.push((self.clock.now, line));
        self.react_alone(hit, [STANDING, STRUCK]);
        let shown = self.stage.bugs == Bugs::Fixed;
        for f in climax..n {
            self.turret_frame(record, pose(f), time, shown);
        }
        // Facing left, standing, in a whole redraw shown at once (0x465E74 to 0x465E89).
        let now = self.clock.now;
        self.flush(now, now);
        self.turret_keys.push(TurretKey {
            record,
            drawn: now,
            shown: now,
            pose: TurretPose {
                group: u32::from(self.stage.groups.standing),
                frame: 0,
                right: false,
            },
        });
    }

    /// A wall shot of `shooter` (the Catapult, a Cyclops) that hit target `t` for `damage`
    /// (`wall_shot_animate` 0x445FD0, §10.2): the group by the angle to the target's point (the
    /// original's unnegated, CLT-Q19 (1); a shot's with the slip fixed), its shot sound and frames
    /// to the climax through the deadline; it stands in memory, the screen keeping its last frame
    /// until the end (CLT-Q19 (2)); the rock's arc on its own clock; the landing's sound and the
    /// explosion's frames through the deadline, each over a redraw without the background
    /// (CLT-Q19 (4)), the part's damage at frame 5; a whole redraw shown at once.
    pub(super) fn wall_shot(&mut self, shooter: u8, t: usize, damage: u8) {
        let Some(live) = self.live(shooter) else {
            self.damage_wall(t, damage, None);
            return;
        };
        let fixed = self.stage.bugs == Bugs::Fixed;
        let to = TARGET_POINTS.get(t).copied().unwrap_or_default();
        let anchor = self.stage.anchor(live.position(live.hex));
        let start = |i: usize| launch(anchor, live.right, &live.look, i);
        let [x, y] = start(1);
        let direction = wall_group(i64::from(to[0] - x), i64::from(to[1] - y), fixed);
        let from = start(direction);
        let group = self.stage.groups.shoot[direction];
        let n = live.look.count(group);
        let climax = live.look.climax_in(n);
        let time = self.speed.scale(live.look.times[2] / climax.max(1));
        self.cue(shooter, Sound::Shot, false);
        for f in 0..climax {
            self.frame(&[(shooter, Self::posed(&live, group, f, live.hex))], time);
        }
        // Standing in memory, not redrawn (0x4462A3): the screen keeps the last frame until the
        // end's redraw (with the slip fixed, it stands from the flight on).
        let standing = self.standing(&live, live.hex);
        let drawn = self.clock.now;
        let mut stand = Key {
            stack: shooter,
            drawn,
            shown: drawn,
            pose: standing,
        };
        if fixed {
            self.keys.push(stand);
        }
        self.rock(&live, from, to);
        let burst = self
            .stage
            .siege
            .map(|s| {
                if damage > 0 {
                    s.hit.clone()
                } else {
                    s.miss.clone()
                }
            })
            .unwrap_or_default();
        let sound = if damage > 0 {
            Sound::WallHit
        } else {
            Sound::WallMiss
        };
        self.cue(shooter, sound, false);
        let [fw, fh] = self.stage.layout.mouse.field;
        let [w, h] = burst.size;
        let at = [to[0] - w / 2, to[1] - h / 2];
        let rect = [
            at[0].max(0),
            at[1].max(0),
            (at[0] + w).min(fw),
            (at[1] + h).min(fh),
        ];
        let blast = self.blast_count;
        self.blast_count += 1;
        let tick = self.speed.scale(TICK);
        // The walls' key the damage adds, if any.
        let key = self.wall_keys.len();
        let mut damaged = false;
        for k in 0..burst.frames {
            if k == DAMAGE_FRAME {
                let drawn = self.clock.now;
                self.damage_wall(t, damage, Some(drawn));
                damaged = true;
            }
            let shown = self.clock.show(tick);
            // With the slip fixed the new states show with frame 5 everywhere; the original's
            // show them in the explosion's rectangle only, elsewhere with the end's redraw.
            let five = k == DAMAGE_FRAME && fixed;
            if let Some(w) = self.wall_keys.get_mut(key).filter(|_| five) {
                w.shown = shown;
            }
            let walls = self.siege.map(|s| s.walls).unwrap_or_default();
            self.blasts.push(BlastKey {
                blast,
                shown,
                frame: Some(k),
                def: burst.def.clone(),
                at,
                rect,
                walls,
                restored: fixed,
            });
        }
        // An explosion of fewer frames (none without the siege's stage) damages at its end.
        if !damaged {
            self.damage_wall(t, damage, None);
        }
        // The end: a whole redraw shown at once (0x4464B1).
        let now = self.clock.now;
        if !fixed {
            stand.shown = now;
            self.keys.push(stand);
            if let Some(w) = self.wall_keys.get_mut(key) {
                w.shown = now;
            }
        }
        self.flush(now, now);
        let walls = self.siege.map(|s| s.walls).unwrap_or_default();
        self.blasts.push(BlastKey {
            blast,
            shown: now,
            frame: None,
            def: burst.def,
            at,
            rect,
            walls,
            restored: fixed,
        });
    }

    /// The damage of a wall shot or Earthquake on target `t`, drawn into memory at `at` (none: no
    /// time of its own, at once): the walls' new states from then on.
    pub(super) fn damage_wall(&mut self, t: usize, damage: u8, at: Option<u32>) {
        let Some(siege) = self.siege.as_mut() else {
            return;
        };
        if damage == 0 || !siege.damage(t, damage) {
            return;
        }
        let walls = siege.walls;
        let now = at.unwrap_or(self.clock.now);
        self.wall_keys.push(WallKey {
            drawn: now,
            shown: now,
            walls,
        });
    }

    /// The rock's arc from `from` to `to` (`catapult_rock_flight` 0x467A00): d = ⌊√(Δx² + Δy²)⌋,
    /// n = ⌊(d + 10) / 20⌋ steps of trunc(100 F) ms on its own clock; step i at x0 + trunc(i Δx /
    /// n) and trunc(y0 + (Δy − (n − i) a) i / n) with a = 2 |Δx| / n, in f64 (the x87 at 53
    /// bits), (x0, y0) the canvas centred on the start; group 0's frames cycling, unmirrored.
    fn rock(&mut self, live: &Live, from: [i32; 2], to: [i32; 2]) {
        let Some(missile) = live.look.missile.as_ref() else {
            return;
        };
        let (dx, dy) = (i64::from(to[0] - from[0]), i64::from(to[1] - from[1]));
        let d = i64::try_from(isqrt(dx * dx + dy * dy)).unwrap_or(0);
        let n = (d + ROCK_PIXELS / 2) / ROCK_PIXELS;
        let step = self.speed.scale(TICK);
        let [w, h] = missile.size;
        let [x0, y0] = [from[0] - w / 2, from[1] - h / 2];
        let path: Vec<[i32; 2]> = (0..n)
            .map(|i| {
                if i == 0 {
                    return [x0, y0];
                }
                let x = x0 + i32::try_from(i * dx / n).unwrap_or(0);
                [x, arc_y(y0, dy, dx.abs(), (i, n))]
            })
            .collect();
        let steps = u32::try_from(n).unwrap_or(0);
        self.flights.push(Flight {
            def: missile.def.clone(),
            frame: 0,
            cycle: missile.frames,
            mirrored: false,
            from: self.clock.now,
            step,
            steps,
            start: [x0, y0],
            delta: [0, 0],
            path,
        });
        self.clock.wait(steps * step);
    }
}

/// A wall shot's group (0 up, 1 straight, 2 down; 0x4460B5) by the angle from its start to the
/// target, `dx` and `dy` the target less the start (screen y down): the original's atan(Δy /
/// |Δx|), Δy not negated (CLT-Q19 (1)), with the slip fixed a shot's, atan(−Δy / |Δx|); above
/// 25° up, above −25° straight. Straight above or below both take 90° for Δy ≤ 0, else −90°.
pub(super) fn wall_group(dx: i64, dy: i64, fixed: bool) -> usize {
    let dx = dx.abs();
    if dx == 0 {
        return if dy <= 0 { 0 } else { 2 };
    }
    let up = if fixed { -dy } else { dy };
    if up * TAN_SCALE > dx * TAN_25 {
        0
    } else if up * TAN_SCALE > -dx * TAN_25 {
        1
    } else {
        2
    }
}

/// The rock's y at step `i` of `n` (`catapult_rock_flight` 0x467BBE to 0x467C0D): trunc(y0 +
/// (Δy − (n − i) a) i / n), a = 2 |Δx| / n, each operation an f64's (53 bits, rounded to
/// nearest), the sum truncated toward zero.
fn arc_y(y0: i32, dy: i64, adx: i64, (i, n): (i64, i64)) -> i32 {
    let a = F64::int(2 * adx).over(n);
    let p = a.times(n - i);
    let q = F64::int(dy).minus(p);
    let r = q.times(i);
    let s = r.over(n);
    let y = s.minus(F64::int(-i64::from(y0)));
    i32::try_from(y.trunc()).unwrap_or(0)
}

/// An f64 as the x87 holds it at 53 bits: `m` × 2^`e`, |m| < 2^53 (2^53 after a round up), each
/// operation exact before it is rounded to 53 significant bits, ties to even. The rock's numbers
/// (|x| < 2^20, integers' quotients) stay far inside an f64's range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct F64 {
    m: i128,
    e: i32,
}

/// An f64's significant bits.
const F64_BITS: u32 = 53;

impl F64 {
    /// An integer, exact.
    fn int(v: i64) -> Self {
        Self::round(i128::from(v), 0, false)
    }

    /// The exact value `m` × 2^`e` (and below it a remainder, `sticky`) rounded.
    fn round(m: i128, e: i32, sticky: bool) -> Self {
        let negative = m < 0;
        let magnitude = m.unsigned_abs();
        let bits = 128 - magnitude.leading_zeros();
        let shift = bits.saturating_sub(F64_BITS);
        let (mut kept, rest) = (magnitude >> shift, magnitude & ((1u128 << shift) - 1));
        if shift > 0 {
            let half = 1u128 << (shift - 1);
            if rest > half || (rest == half && (sticky || kept & 1 == 1)) {
                kept += 1;
            }
        }
        let kept = i128::try_from(kept).unwrap_or(0);
        Self {
            m: if negative { -kept } else { kept },
            e: e + i32::try_from(shift).unwrap_or(0),
        }
    }

    /// Its product with an integer, rounded.
    fn times(self, k: i64) -> Self {
        Self::round(self.m * i128::from(k), self.e, false)
    }

    /// Its quotient by an integer (`k` > 0), rounded: 64 more bits of the quotient and a sticky
    /// remainder.
    fn over(self, k: i64) -> Self {
        let k = i128::from(k.max(1));
        let scaled = self.m << 64;
        Self::round(scaled / k, self.e - 64, scaled % k != 0)
    }

    /// The difference, rounded: both at the smaller exponent, exact.
    fn minus(self, o: Self) -> Self {
        let e = self.e.min(o.e);
        let at = |v: Self| v.m << u32::try_from(v.e - e).unwrap_or(0);
        Self::round(at(self) - at(o), e, false)
    }

    /// Truncated toward zero (`crt_ftol`). Its exponent is never above 0 here: a value of
    /// 2^53 or more does not occur.
    fn trunc(self) -> i128 {
        let down = u32::try_from(-i64::from(self.e)).unwrap_or(0);
        let magnitude = i128::try_from(self.m.unsigned_abs() >> down).unwrap_or(0);
        if self.m < 0 { -magnitude } else { magnitude }
    }
}

/// A turret's pose in the timeline's lines: `turret <k> g<group> f<frame> d<facing>`.
#[must_use]
pub(super) fn turret_line(t: u32, record: u8, pose: TurretPose) -> String {
    alloc::format!(
        "{t} turret {record} g{} f{} d{}\n",
        pose.group,
        pose.frame,
        u8::from(pose.right)
    )
}

/// The walls' states in the timeline's lines: `walls <18 states> gate <state>`.
#[must_use]
pub(super) fn walls_line(t: u32, walls: &Walls) -> String {
    let parts: Vec<String> = walls.parts.iter().map(|p| alloc::format!("{p}")).collect();
    alloc::format!("{t} walls {} gate {}\n", parts.join(","), walls.gate)
}
