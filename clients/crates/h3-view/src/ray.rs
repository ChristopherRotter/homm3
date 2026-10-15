//! The energy ray (E17 M3 T1b-1; `docs/epics/E17-design.md` §7.12): a shooter with the ray flag
//! (creature flag 0x800: the Arch Magi, the Beholders, the Evil Eyes) draws its shot as a ray
//! instead of a projectile (`shot_animation` 0x43F51D). The ray is `DrawRay` 0x5A5C20 with the
//! creature's arguments: no branches and no randomness, 5 pixels thick at the start and 4 at the
//! target, `trunc(d / 15) + 15` pixel steps per tick (d the shot's distance). [`trace`] follows it
//! tick by tick as the executable does: each tick moves the ray's head that many steps of one
//! pixel along its aim (the f32 head plus the sine and cosine of the f32 aim, at the x87's 53 bits),
//! drawing a cross-section at every pixel it enters (0x5A5440); between ticks it aims again at the
//! target from the pixel it reached and sets its thickness by the way gone (0x5A5260). It ends at
//! the pixel within 2 of the target (city-block distance), or where it moves away after coming
//! within 15.
//!
//! No float arithmetic (ADR-0003 §3): the f32 and 53-bit roundings on integers, the sine, cosine
//! and arc tangent by their series in fixed point to about 2^-58, the x87's own error below
//! 2^-63; they could round differently only where an exact value lies that close to a rounding
//! boundary.

use alloc::vec::Vec;

/// The battlefield's last column and row, which the ray keeps to (0x5A5AA4, 0x5A54E4, 0x5A5504).
const RIGHT: i32 = 799;
const BOTTOM: i32 = 555;
/// The thickness at the start and at the target (`shot_animation`'s arguments, 0x43F5A7).
const START_WIDTH: i32 = 5;
const END_WIDTH: i32 = 4;
/// Within this city-block distance of the target the ray closes in (0x5A5A3A); it ends within
/// [`ARRIVED`] or as it moves away (0x5A5A2B).
const NEAR: i32 = 15;
const ARRIVED: i32 = 2;
/// The pixel steps per tick: trunc(d / 15) + 15 (0x43F578).
const STEP_DIVISOR: u64 = 15;
const STEP_BASE: u64 = 15;
/// More ticks than any ray on the battlefield takes (the executable has no limit).
const MAX_TICKS: usize = 256;

/// The fractional bits of the fixed point the ray's numbers live in: every f32 the ray holds
/// (0 or at least 2^-30 in magnitude, below 2^10) is exact on its grid.
const FRAC: u32 = 60;
/// 1 and π in the fixed point.
const ONE: i128 = 1 << FRAC;
const PI: i128 = 3_622_009_729_038_561_421;

/// One cross-section of the ray at the pixel (`x`, `y`) its head entered: the pixels `lo..=hi`
/// across it, down the column when `column` (a ray mostly sideways, 0x5A5B96), else along the
/// row. A pixel's colour is the ray's entry by its distance from the nearer edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stroke {
    /// The head's column.
    pub x: i32,
    /// The head's row.
    pub y: i32,
    /// The first pixel across, from the head (−(w >> 1) for the thickness w).
    pub lo: i32,
    /// The last pixel across (lo + w − 1).
    pub hi: i32,
    /// Across means down the column.
    pub column: bool,
}

impl Stroke {
    /// The cross-section's pixels on the battlefield (0x5A5571), in the order drawn, each with
    /// its colour entry (0 at the edges, rising inward; 0x5A5599).
    pub fn pixels(self) -> impl Iterator<Item = ([i32; 2], usize)> {
        (self.lo..=self.hi).filter_map(move |k| {
            let at = if self.column {
                [self.x, self.y + k]
            } else {
                [self.x + k, self.y]
            };
            let entry = if k < 0 { k - self.lo } else { self.hi - k };
            let inside = (0..=RIGHT).contains(&at[0]) && (0..=BOTTOM).contains(&at[1]);
            inside.then(|| (at, usize::try_from(entry).unwrap_or(0)))
        })
    }
}

/// The ray from `from` (the shooter's missile point) to `to` (the target's point): each tick's
/// strokes, in the order drawn. Each tick shows when its strokes are drawn; the strokes stay
/// until the ray ends.
pub fn trace(from: [i32; 2], to: [i32; 2]) -> Vec<Vec<Stroke>> {
    // `shot_animation`'s distance, between the points as given (0x43F4AA).
    let steps = distance(from, to) / STEP_DIVISOR + STEP_BASE;
    let clamp = |[x, y]: [i32; 2]| [x.clamp(0, RIGHT), y.clamp(0, BOTTOM)];
    let (from, to) = (clamp(from), clamp(to));
    let mut head = Head::new(from, to);
    let mut ticks = Vec::new();
    while ticks.len() < MAX_TICKS {
        let (strokes, done) = head.advance(steps);
        ticks.push(strokes);
        if done {
            break;
        }
        head.aim();
    }
    ticks
}

/// The ray's state (its record, 0x5A5A90).
#[derive(Debug, Clone)]
struct Head {
    to: [i32; 2],
    /// The head as f32s (+0x24, +0x28) and the pixel it is on (+0x2C, +0x30).
    f: [i128; 2],
    at: [i32; 2],
    /// The distance from the start to the target (+0x64).
    length: u64,
    /// Cross-sections down the column (+0x34).
    column: bool,
    /// The thickness (+0x14).
    width: i32,
    /// The aim's sine and cosine, of the f32 angle (+0x38).
    step: [i128; 2],
    /// The least city-block distance to the target once within [`NEAR`] (+0x44, +0x48).
    near: Option<i32>,
}

impl Head {
    fn new(from: [i32; 2], to: [i32; 2]) -> Self {
        let [dx, dy] = [to[0] - from[0], to[1] - from[1]];
        let mut head = Self {
            to,
            f: [i128::from(from[0]) << FRAC, i128::from(from[1]) << FRAC],
            at: from,
            length: distance(from, to),
            column: dx.abs() > dy.abs(),
            width: START_WIDTH,
            step: [0, ONE],
            near: None,
        };
        head.aim();
        head
    }

    /// Aims at the target from the pixel reached and sets the thickness by the way gone
    /// (0x5A5260): the start's thickness plus trunc((end − start ± 1) × p), at least 1, p the
    /// f32 share of the start's distance gone (0 beyond it).
    fn aim(&mut self) {
        let left = distance(self.at, self.to);
        let gone = if left > self.length {
            0
        } else {
            f32_ratio(
                i128::from(self.length - left),
                i128::from(self.length.max(1)),
            )
        };
        self.width = width(START_WIDTH, END_WIDTH, gone);
        // atan2(dx, dy): the angle from the y axis (0x5A5368), stored as an f32.
        let [dx, dy] = [self.to[0] - self.at[0], self.to[1] - self.at[1]];
        let angle = round(atan2(dx, dy), 24);
        self.step = sin_cos(angle);
    }

    /// One tick (0x5A5440): `steps` steps of one pixel; a cross-section at every pixel entered;
    /// done when the head arrives. The strokes and whether it is done.
    fn advance(&mut self, steps: u64) -> (Vec<Stroke>, bool) {
        let lo = -(self.width >> 1);
        let hi = lo + self.width - 1;
        let mut strokes = Vec::new();
        let mut last = self.at;
        for _ in 0..steps {
            for axis in 0..2 {
                let (f, step) = match axis {
                    0 => (self.f[0], self.step[0]),
                    _ => (self.f[1], self.step[1]),
                };
                let (end, mut v) = match axis {
                    0 => (RIGHT, round(round(f + step, 53), 24)),
                    _ => (BOTTOM, round(round(f + step, 53), 24)),
                };
                let mut pixel = trunc(v);
                if pixel < 0 {
                    (pixel, v) = (0, 0);
                } else if pixel > end {
                    (pixel, v) = (end, i128::from(end) << FRAC);
                }
                match axis {
                    0 => (self.f[0], self.at[0]) = (v, pixel),
                    _ => (self.f[1], self.at[1]) = (v, pixel),
                }
            }
            if self.at == last {
                continue;
            }
            last = self.at;
            strokes.push(Stroke {
                x: self.at[0],
                y: self.at[1],
                lo,
                hi,
                column: self.column,
            });
            let m = (self.to[0] - self.at[0]).abs() + (self.to[1] - self.at[1]).abs();
            match self.near {
                Some(best) if m > best + 1 || m <= ARRIVED => return (strokes, true),
                Some(best) if m < best => self.near = Some(m),
                Some(_) => {}
                None if m < NEAR => self.near = Some(m),
                None => {}
            }
        }
        (strokes, false)
    }
}

/// The thickness at the share `gone` of the way (an f32 in the fixed point, 0 to 1): `start` plus
/// trunc((`end` − `start` ± 1) × gone), the ± away from 0, at least 1 (0x5A52EC).
fn width(start: i32, end: i32, gone: i128) -> i32 {
    let change = match end - start {
        0 => 0,
        d if d > 0 => d + 1,
        d => d - 1,
    };
    let width = i128::from(start) + i128::from(change) * gone / ONE;
    i32::try_from(width).unwrap_or(1).max(1)
}

/// trunc(√(dx² + dy²)): `crt_sqrt` truncated (exact for these integers).
fn distance(a: [i32; 2], b: [i32; 2]) -> u64 {
    let [dx, dy] = [i64::from(b[0] - a[0]), i64::from(b[1] - a[1])];
    u64::try_from(dx * dx + dy * dy).unwrap_or(0).isqrt()
}

/// A fixed-point value's truncation toward zero (`crt_ftol`).
fn trunc(v: i128) -> i32 {
    i32::try_from(v / ONE).unwrap_or(0)
}

/// `v` rounded to `bits` significant bits, ties to even: an x87 result at precision `bits`.
fn round(v: i128, bits: u32) -> i128 {
    let m = v.unsigned_abs();
    let width = 128 - m.leading_zeros();
    if width <= bits {
        return v;
    }
    let shift = width - bits;
    let (kept, rest) = (m >> shift, m & ((1 << shift) - 1));
    let half = 1 << (shift - 1);
    let up = rest > half || (rest == half && kept & 1 == 1);
    let r = i128::try_from((kept + u128::from(up)) << shift).unwrap_or(i128::MAX);
    if v < 0 { -r } else { r }
}

/// `num` / `den` (both ≥ 0, `den` > 0) as the nearest f32, ties to even (the 53-bit quotient
/// rounds the same for these integers: no ratio of them lies within 2^-53 of an f32 midpoint
/// without being one). The quotient carries a sticky bit below its guard bits: an inexact one
/// lies a little above its grid value, never on a midpoint.
fn f32_ratio(num: i128, den: i128) -> i128 {
    /// Guard bits below the fixed point's grid.
    const GUARD: u32 = 8;
    let scaled = num << (FRAC + GUARD);
    let q = (scaled / den) << 1 | i128::from(scaled % den != 0);
    round(q, 24) >> (GUARD + 1)
}

/// sin(`x`) and cos(`x`) (|x| ≤ π + 1) by their series.
fn sin_cos(x: i128) -> [i128; 2] {
    let (mut sin, mut cos, mut term) = (0, 0, ONE);
    for n in 0..64 {
        match n % 4 {
            0 => cos += term,
            1 => sin += term,
            2 => cos -= term,
            _ => sin -= term,
        }
        term = term * x / ONE / (n + 1);
        if term == 0 {
            break;
        }
    }
    [sin, cos]
}

/// atan2(`y`, `x`) of two integers, in the fixed point (the x87's `fpatan`; atan2(0, 0) = 0).
fn atan2(y: i32, x: i32) -> i128 {
    let (a, b) = (i128::from(y.unsigned_abs()), i128::from(x.unsigned_abs()));
    let angle = if a == 0 && b == 0 {
        0
    } else if a <= b {
        atan_ratio(a, b)
    } else {
        PI / 2 - atan_ratio(b, a)
    };
    let angle = if x < 0 { PI - angle } else { angle };
    if y < 0 { -angle } else { angle }
}

/// atan(`p` / `q`) for 0 ≤ p ≤ q, q > 0: the argument halved twice
/// (atan z = 2 atan(z / (1 + √(1 + z²)))), then the series.
fn atan_ratio(p: i128, q: i128) -> i128 {
    let halve = |z: i128| z * ONE / (ONE + isqrt(ONE * ONE + z * z));
    let z = halve(halve((p << FRAC) / q));
    let z2 = z * z / ONE;
    let (mut sum, mut power) = (0, z);
    for n in 0..64 {
        let term = power / (2 * n + 1);
        if term == 0 {
            break;
        }
        sum += if n % 2 == 0 { term } else { -term };
        power = power * z2 / ONE;
    }
    4 * sum
}

/// ⌊√`n`⌋ of a non-negative fixed-point square.
fn isqrt(n: i128) -> i128 {
    i128::try_from(n.unsigned_abs().isqrt()).unwrap_or(0)
}

#[cfg(test)]
mod tests;
