//! A shot's straight line through a siege's walls (`siege_line_clear` 0x467840, from the cursor's
//! wall test `shot_wall_test` 0x467510; E17 M4 T2c-2, design §10.2; the battle crate's
//! `siege::line_blocked`, BAT-SIE-008): on the column and row grid, not the hexes' geometry, from
//! the shooter's column and row towards the target's in 17 × the larger difference steps of 1/17
//! of a cell along the longer axis and the matching fraction along the other, each position a
//! single-precision value truncated to a hex at every step.
//!
//! No float arithmetic (`clippy.toml`): each axis is computed exactly in integers, as numerators
//! over 2^[`FRAC`] ([`Axis`]). The original stores an axis's step as an `f32`: the axis's
//! difference over the longer one's in the FPU's 53 bits, stored, then divided by 17 the same
//! way; each position is the `f32` sum of the last one and the step (exact in 53 bits, rounded
//! once to 24). The binary's tests check every start, difference and length of the field against
//! the engine's x87 arithmetic.

use super::COLUMNS;

/// The fraction bits of an axis's numerators: a step is at least 1/272, a multiple of 2^-32,
/// and a position below 17 keeps 24 significant bits.
pub const FRAC: u32 = 40;
/// The significant bits of an `f32` and of the FPU's arithmetic (the thread's default, 53).
const SINGLE: u32 = 24;
const DOUBLE: u32 = 53;
/// The steps per cell.
const STEPS: u32 = 17;

/// A value `m` × 2^`e`.
type Float = (u128, i32);

/// `n / d` rounded to `bits` significant bits, to nearest, ties to even (`d` > 0; 0 for `n` = 0).
fn divided(n: u128, d: u128, bits: u32) -> Float {
    if n == 0 {
        return (0, 0);
    }
    let len = |x: u128| 128 - i64::from(x.leading_zeros());
    // e = floor(log2(n / d)).
    let mut e = len(n) - len(d);
    let below = if e >= 0 { n < d << e } else { n << -e < d };
    if below {
        e -= 1;
    }
    // m = round(n · 2^k / d), k = bits − 1 − e, so that m has `bits` bits (or 2^bits rounded up).
    let k = i64::from(bits) - 1 - e;
    let (scaled, d) = if k >= 0 { (n << k, d) } else { (n, d << -k) };
    let (mut m, r) = (scaled / d, scaled % d);
    if 2 * r > d || (2 * r == d && m & 1 == 1) {
        m += 1;
    }
    (m, i32::try_from(-k).unwrap_or(0))
}

/// `v` rounded to `bits` significant bits, to nearest, ties to even (a store).
fn rounded((m, e): Float, bits: u32) -> Float {
    let len = 128 - m.leading_zeros();
    if len <= bits {
        return (m, e);
    }
    let k = len - bits;
    let (q, rest) = (m >> k, m & ((1 << k) - 1));
    let half = 1 << (k - 1);
    let up = rest > half || (rest == half && q & 1 == 1);
    (q + u128::from(up), e + i32::try_from(k).unwrap_or(0))
}

/// `v` as a numerator over 2^[`FRAC`] (exact for the steps).
fn fixed((m, e): Float) -> u64 {
    let shift = e + i32::try_from(FRAC).unwrap_or(0);
    let v = match u32::try_from(shift) {
        Ok(up) => m << up,
        Err(_) => m >> shift.unsigned_abs(),
    };
    u64::try_from(v).unwrap_or(u64::MAX)
}

/// The numerator `num` over 2^[`FRAC`] rounded to an `f32`'s 24 significant bits, to nearest,
/// ties to even (the kernel's `arith::round_single`).
fn single(num: u64) -> u64 {
    let (m, e) = rounded((u128::from(num), 0), SINGLE);
    u64::try_from(m << e).unwrap_or(u64::MAX)
}

/// One axis of the line: its position and step, numerators over 2^[`FRAC`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Axis {
    at: u64,
    by: u64,
    back: bool,
}

impl Axis {
    /// The axis starting at `from` towards a difference of `d`, the longer axis's difference
    /// being `long` (0 for a line from a hex to itself: no step).
    #[must_use]
    pub fn new(from: i32, d: i32, long: u32) -> Self {
        let unit = match long {
            0 => (0, 0),
            _ => rounded(
                divided(u128::from(d.unsigned_abs()), u128::from(long), DOUBLE),
                SINGLE,
            ),
        };
        // The step: the stored ratio over 17, in 53 bits, stored.
        let (m, e) = unit;
        let (q, f) = divided(m, u128::from(STEPS), DOUBLE);
        let by = fixed(rounded((q, f + e), SINGLE));
        Self {
            at: u64::from(from.unsigned_abs()) << FRAC,
            by,
            back: d < 0,
        }
    }

    /// One step: the new position's column or row (truncated, as the original's `_ftol`; a
    /// position below 0 stays 0).
    pub fn advance(&mut self) -> i32 {
        self.at = single(if self.back {
            self.at.saturating_sub(self.by)
        } else {
            self.at + self.by
        });
        self.column()
    }

    /// The position's column or row.
    #[must_use]
    pub fn column(self) -> i32 {
        i32::try_from(self.at >> FRAC).unwrap_or(i32::MAX)
    }

    /// The position, a numerator over 2^[`FRAC`].
    #[must_use]
    pub const fn position(self) -> u64 {
        self.at
    }
}

/// Whether the straight line from hex `from` to hex `to` meets a hex that `blocks` (each hex
/// tested once where the line enters it).
#[must_use]
pub fn blocked(from: u8, to: u8, blocks: &dyn Fn(u8) -> bool) -> bool {
    let c = i32::from(COLUMNS);
    let (fx, fy) = (i32::from(from) % c, i32::from(from) / c);
    let (tx, ty) = (i32::from(to) % c, i32::from(to) / c);
    let (dx, dy) = (tx - fx, ty - fy);
    let long = dx.unsigned_abs().max(dy.unsigned_abs());
    let (mut x, mut y) = (Axis::new(fx, dx, long), Axis::new(fy, dy, long));
    let mut last = (-1, -1);
    for _ in 0..STEPS * long {
        let at = (x.advance(), y.advance());
        if at == last {
            continue;
        }
        last = at;
        // The positions stay between the two hexes' columns and rows.
        let hex = u8::try_from(at.1 * c + at.0).unwrap_or(u8::MAX);
        if blocks(hex) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_axis_steps_a_seventeenth_of_its_share() {
        // Along the longer axis a step is 1/17 of a cell: f32(1/17) = 0x3D70F0F1.
        let mut a = Axis::new(3, 5, 5);
        let one = Axis::new(0, 1, 1);
        assert_eq!(one.by, 0x00F0_F0F1 << 12);
        assert_eq!(a.column(), 3);
        for _ in 0..17 {
            a.advance();
        }
        assert_eq!(a.column(), 4);
        // Backwards a position below 0 stays 0.
        let mut b = Axis::new(0, -1, 1);
        b.advance();
        assert_eq!((b.position(), b.column()), (0, 0));
        // A line from a hex to itself takes no step.
        assert_eq!(Axis::new(7, 0, 0).column(), 7);
        // Rounding: a value of more than 24 bits keeps 24, ties to even.
        assert_eq!(single((1 << 30) + 1), 1 << 30);
        assert_eq!(single((1 << 30) + (1 << 6)), (1 << 30));
        assert_eq!(single((1 << 30) + (3 << 6)), (1 << 30) + (4 << 6));
        assert_eq!(fixed((1, -50)), 0);
        assert_eq!(divided(0, 3, SINGLE), (0, 0));
        assert_eq!(divided(1 << 100, 1, SINGLE), (1 << 23, 77));
        // 2/3 in 2 bits rounds up to 3/4; a tie to the even: 3/2 in 1 bit up to 2, 5/2 in 2
        // bits down to 2.
        assert_eq!(divided(2, 3, 2), (3, -2));
        assert_eq!((divided(3, 2, 1), divided(5, 2, 2)), ((2, 0), (2, 0)));
    }

    #[test]
    fn a_line_finds_what_blocks_it() {
        // Along row 5 from hex 88 to 98 the line enters every hex between, the gate's 96 among
        // them; from 98 to 88 too.
        let gate = |h: u8| h == 96;
        assert!(blocked(88, 98, &gate));
        assert!(blocked(98, 88, &gate));
        assert!(!blocked(88, 95, &gate));
        // From a hex to itself nothing.
        assert!(!blocked(96, 96, &gate));
        // Across rows: from 35 (row 2) to 100 (row 5) the line passes 62's row-3 wall hex or
        // its neighbours; the walls of rows 3 and 4 block it.
        let walls = |h: u8| [62, 78].contains(&h);
        assert!(blocked(35, 100, &walls));
        assert!(!blocked(35, 100, &gate));
    }
}
