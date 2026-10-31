//! A battle stack's palette effects (E17 M5 T5b1, `docs/epics/E17-design.md` §7.26):
//! `draw_battle_stack` 0x43E280 to 0x43E4C7 changes the creature's palette in the screen's format
//! (entries [`FIRST`] to 255) for its sprite alone and puts it back after the draw (0x43E535).
//! The first effect that applies of: Bloodlust's glow (the stack's +0x84 flag 0x20000000),
//! Stone Gaze's glow (0x40000000), the petrified look (Stone Gaze's duration +0x2B0 above 0), a
//! clone's colours (0x800000).
//!
//! Bloodlust's glow and a clone's colours take each entry through HSV and back
//! (`palette16_recolour` 0x522B50, with 0x523190 and 0x5232F0), Stone Gaze's glow too
//! (`palette16_stone_glow` 0x522A10), with single-precision values on the x87 at 53 bits
//! (`docs/formats/installation.md` §7.10): computed here exactly in integers ([`Num`]), each x87
//! result rounded to 53 significant bits and each f32 store to 24, ties to even, as the tint
//! ([`crate::tint`]). The petrified look is integers alone (`palette16_petrified` 0x522D50).
//! `clients/scripts/palette-reference.py` computes them in IEEE doubles; the tests compare their
//! digests over every 16-bit value.

use core::cmp::Ordering;

use crate::palette::ScreenPalette;
use crate::screen::PixelFormat;

/// The first palette entry the effects change (0x522B68: 246 entries from +0x30, the palette's
/// +0x1C plus 10 × 2): the entries below are the drawers' keyed colours.
pub const FIRST: usize = 10;

/// A battle stack's palette effect (`draw_battle_stack` 0x43E280).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Effect {
    /// Bloodlust's glow (0x43E2CC) at a step n of its animation, the stack's +0x450 being
    /// f32(n × 0.1) (`cast_spell` 0x5A12A8): the hue moved toward red (0) by the strength,
    /// saturation and brightness toward 1 by a factor of the strength plus 1.
    Bloodlust(u8),
    /// Stone Gaze's glow (0x43E388) at a step n (as [`Effect::Bloodlust`]): the saturation
    /// times the strength.
    StoneGlow(u8),
    /// The petrified look (0x43E40E): every channel the entry's brightest, a grey.
    Petrified,
    /// A clone's colours (0x43E47C): the hue set to 0.67 (blue), saturation and brightness moved
    /// halfway to 1.
    Clone,
}

impl Effect {
    /// `value` (a 16-bit colour in a format whose channels' masks are `masks`, red, green, blue)
    /// as the effect makes it. A mask of 2^31 or more (no 16-bit format's) scales by 1, not 0.
    #[must_use]
    pub fn colour(self, value: u16, masks: [u32; 3]) -> u16 {
        let scales = masks.map(|m| (0x7FFF_FFFF / m.max(1)).max(1));
        let scaled = [0, 1, 2].map(|c| (u32::from(value) & masks[c]) * scales[c]);
        let out = match self {
            Self::Petrified => {
                // 0x522DC7: the largest of the three, unsigned.
                let largest = scaled.into_iter().max().unwrap_or(0);
                [largest; 3]
            }
            Self::Bloodlust(step) => {
                let glow = strength(step);
                let factor = glow.add(Num::ONE).x87().f32();
                let hsv = hsv(scaled);
                rgb(recolour(hsv, [Num::ZERO, glow, factor, factor]), scaled)
            }
            Self::Clone => {
                let [hue, one, two] =
                    [0x3F2B_851F, 0x3F80_0000, 0x4000_0000].map(Num::from_f32_bits);
                rgb(recolour(hsv(scaled), [hue, one, two, two]), scaled)
            }
            Self::StoneGlow(step) => {
                let [h, s, v] = hsv(scaled);
                rgb([h, toward_one(s, strength(step)), v], scaled)
            }
        };
        let packed = (0..3).fold(0, |packed, c| packed | (out[c] / scales[c]) & masks[c]);
        u16::try_from(packed).unwrap_or(u16::MAX)
    }
}

/// `palette`'s entries from [`FIRST`] on, in `format`, as `effect` makes them.
pub fn apply(palette: &mut ScreenPalette, format: PixelFormat, effect: Effect) {
    let masks = format.channel_masks();
    for entry in palette.0.iter_mut().skip(FIRST) {
        *entry = effect.colour(*entry, masks);
    }
}

/// A glow's strength at `step`: f32(step × 0.1), the product at 53 bits (0x5A12B9).
fn strength(step: u8) -> Num {
    let tenth = Num::from_f64_bits(0x3FB9_9999_9999_999A);
    Num::int(u64::from(step)).mul(tenth).x87().f32()
}

/// 0x523190: the scaled red, green and blue as hue, saturation and value, f32 values.
fn hsv([r, g, b]: [u32; 3]) -> [Num; 3] {
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let top = Num::int(u64::from(max)).f32();
    let v = top.div(Num::int(1 << 31)).x87().f32();
    if max == min {
        // 0x523216: no hue; the saturation 0 (max − min over max, or nothing over nothing).
        return [Num::ZERO, Num::ZERO, v];
    }
    let span = Num::int(u64::from(max - min)).f32();
    let s = span.div(top).x87().f32();
    let distance = |c: u32| Num::int(u64::from(max - c)).f32().div(span).x87().f32();
    let [dr, dg, db] = [distance(r), distance(g), distance(b)];
    let six = Num::int(6);
    // 0x52327B: by the channel that is largest, red first, then green; f32(1/3), f32(2/3).
    let (x, y, base) = if r == max {
        (db, dg, Num::ZERO)
    } else if g == max {
        (dr, db, Num::from_f32_bits(0x3EAA_AAAB))
    } else {
        (dg, dr, Num::from_f32_bits(0x3F2A_AAAB))
    };
    let mut h = x.sub(y).x87().div(six).x87().add(base).x87().f32();
    if h.cmp(Num::ZERO) == Ordering::Less {
        h = h.add(Num::ONE).x87().f32();
    }
    [h, s, v]
}

/// 0x5232F0: hue, saturation and value as the scaled red, green and blue, each rounded to an
/// integer, ties to even (the 1.5 × 2^52 addition, its low word); a sextant above 5 leaves the
/// channels `scaled` as they were (the switch's default).
fn rgb([h, s, v]: [Num; 3], scaled: [u32; 3]) -> [u32; 3] {
    let top = v.mul(Num::int(1 << 31));
    if s.cmp(Num::ZERO) == Ordering::Equal {
        return [top.integer(); 3];
    }
    let h6 = h.mul(Num::int(6)).x87().f32();
    let f = h6.fraction();
    let one = Num::ONE;
    let p = one.sub(s).x87().mul(top).x87().f32().integer();
    let q = one.sub(f.mul(s).x87()).x87().mul(top).x87().f32();
    let t = one
        .sub(one.sub(f).x87().mul(s).x87())
        .x87()
        .mul(top)
        .x87()
        .f32();
    let (q, t, v) = (q.integer(), t.integer(), top.integer());
    match h6.whole() {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        5 => [v, p, q],
        _ => scaled,
    }
}

/// 0x522BD8 to 0x522CD5: the hue moved toward `hue` by `weight` the shorter way round, the
/// saturation and the value by their factors ([`toward_one`]); a negative argument leaves its
/// part as it was.
fn recolour(
    [mut h, mut s, mut v]: [Num; 3],
    [hue, weight, saturation, value]: [Num; 4],
) -> [Num; 3] {
    let negative = |x: Num| x.cmp(Num::ZERO) == Ordering::Less;
    if !negative(weight) {
        let d = hue.sub(h).x87().f32();
        h = d.mul(weight).x87().add(h).x87().f32();
        if d.abs().cmp(Num::HALF) == Ordering::Greater {
            h = if d.cmp(Num::ZERO) == Ordering::Greater {
                Num::ONE.sub(weight).x87().add(h).x87().f32()
            } else {
                h.add(weight).x87().f32()
            };
            if h.cmp(Num::ONE) != Ordering::Less {
                h = h.sub(Num::ONE).x87().f32();
            }
        }
    }
    if !negative(saturation) {
        s = toward_one(s, saturation);
    }
    if !negative(value) {
        v = toward_one(v, value);
    }
    [h, s, v]
}

/// `x` times `factor` up to 1, else moved toward 1: 1 − (1 − x) / `factor` (0x522C71).
fn toward_one(x: Num, factor: Num) -> Num {
    if factor.cmp(Num::ONE) == Ordering::Greater {
        Num::ONE
            .sub(Num::ONE.sub(x).x87().div(factor).x87())
            .x87()
            .f32()
    } else {
        x.mul(factor).x87().f32()
    }
}

/// A binary number, exact: (−1)^`neg` × `m` × 2^`e`; `sticky`: bits below `m`'s last were cut
/// (none was zero), so its magnitude lies strictly between `m` and `m` + 1 times 2^`e`. An
/// operation's result is exact or sticky; [`Num::x87`] and [`Num::f32`] round it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Num {
    neg: bool,
    m: u128,
    e: i32,
    sticky: bool,
}

/// Where [`Num::add`] puts both operands' leading bit: a rounded operand has at most 64
/// significant bits, so its bits below 57 are zeros and a shift right of up to 57 loses nothing.
const LEAD: u32 = 120;

impl Num {
    const ZERO: Self = Self::int(0);
    const ONE: Self = Self::int(1);
    const HALF: Self = Self {
        neg: false,
        m: 1,
        e: -1,
        sticky: false,
    };

    /// An unsigned integer.
    const fn int(v: u64) -> Self {
        Self {
            neg: false,
            m: v as u128,
            e: 0,
            sticky: false,
        }
    }

    /// An f32's bits (a subnormal as such; an infinity or not a number as a large value).
    fn from_f32_bits(bits: u32) -> Self {
        let exponent = i32::try_from((bits >> 23) & 0xFF).unwrap_or(0);
        let fraction = u128::from(bits & 0x7F_FFFF);
        let (m, e) = if exponent == 0 {
            (fraction, -149)
        } else {
            (fraction | 1 << 23, exponent - 150)
        };
        Self {
            neg: bits >> 31 == 1,
            m,
            e,
            sticky: false,
        }
    }

    /// An f64's bits (normal: the executable's constants).
    fn from_f64_bits(bits: u64) -> Self {
        let exponent = i32::try_from((bits >> 52) & 0x7FF).unwrap_or(0);
        Self {
            neg: bits >> 63 == 1,
            m: u128::from(bits & 0xF_FFFF_FFFF_FFFF) | 1 << 52,
            e: exponent - 1075,
            sticky: false,
        }
    }

    /// Rounded to `bits` significant bits, ties to even.
    fn round(self, bits: u32) -> Self {
        let width = 128 - self.m.leading_zeros();
        let shift = width.saturating_sub(bits);
        if shift == 0 {
            // Every operation that cuts bits leaves more than 64 (the quotient, an aligned sum).
            return Self {
                sticky: false,
                ..self
            };
        }
        let (kept, rest) = (self.m >> shift, self.m & ((1 << shift) - 1));
        let half = 1 << (shift - 1);
        let up = rest > half || (rest == half && (self.sticky || kept & 1 == 1));
        Self {
            neg: self.neg,
            m: kept + u128::from(up),
            e: self.e + i32::try_from(shift).unwrap_or(0),
            sticky: false,
        }
    }

    /// An x87 result at 53 bits.
    fn x87(self) -> Self {
        self.round(53)
    }

    /// An f32 store.
    fn f32(self) -> Self {
        self.round(24)
    }

    /// The product, exact (two rounded operands: at most 128 bits).
    fn mul(self, other: Self) -> Self {
        Self {
            neg: self.neg != other.neg,
            m: self.m * other.m,
            e: self.e + other.e,
            sticky: false,
        }
    }

    /// The quotient to at least 64 bits, the rest sticky: the dividend's leading bit moved to the
    /// top (a rounded divisor, not zero, of at most 64 bits).
    fn div(self, other: Self) -> Self {
        let neg = self.neg != other.neg;
        if self.m == 0 {
            return Self { neg, ..Self::ZERO };
        }
        let up = self.m.leading_zeros();
        let n = self.m << up;
        let d = other.m.max(1);
        Self {
            neg,
            m: n / d,
            e: self.e - i32::try_from(up).unwrap_or(0) - other.e,
            sticky: !n.is_multiple_of(d),
        }
    }

    /// The sum, its smaller operand's bits beyond the larger's [`LEAD`] + 7 sticky (two rounded
    /// operands).
    fn add(self, other: Self) -> Self {
        if other.m == 0 {
            return self;
        }
        if self.m == 0 {
            return other;
        }
        let lead = |n: Self| {
            let up = LEAD + 1 - (128 - n.m.leading_zeros());
            (n.m << up, n.e - i32::try_from(up).unwrap_or(0))
        };
        let ((am, ae), (bm, be)) = (lead(self), lead(other));
        // The larger in magnitude first: a higher exponent is larger (both lead at the same bit).
        let ((big, bm_big, e), (small, sm, se)) = if (ae, am) >= (be, bm) {
            ((self, am, ae), (other, bm, be))
        } else {
            ((other, bm, be), (self, am, ae))
        };
        let shift = u32::try_from(e - se).unwrap_or(u32::MAX);
        let (aligned, sticky) = if shift > LEAD {
            (0, true)
        } else {
            (sm >> shift, sm & ((1 << shift) - 1) != 0)
        };
        let m = if big.neg == small.neg {
            bm_big + aligned
        } else {
            // The cut bits taken from the difference: one less, and the rest sticky.
            bm_big - aligned - u128::from(sticky)
        };
        let neg = big.neg && m != 0;
        Self { neg, m, e, sticky }
    }

    /// The difference ([`Num::add`]).
    fn sub(self, other: Self) -> Self {
        self.add(Self {
            neg: !other.neg,
            ..other
        })
    }

    /// The magnitude (`crt_fabs`).
    fn abs(self) -> Self {
        Self { neg: false, ..self }
    }

    /// How `self` compares with `other` (`fcomp`): by the sign of their difference.
    fn cmp(self, other: Self) -> Ordering {
        let d = self.sub(other);
        match (d.m == 0 && !d.sticky, d.neg) {
            (true, _) => Ordering::Equal,
            (false, true) => Ordering::Less,
            (false, false) => Ordering::Greater,
        }
    }

    /// The integer part, toward zero (`crt_ftol`, of a value under 2^63).
    fn whole(self) -> i64 {
        let magnitude = match u32::try_from(self.e) {
            Ok(up) => self.m << up.min(127),
            Err(_) => self.m.checked_shr(self.e.unsigned_abs()).unwrap_or(0),
        };
        let magnitude = i64::try_from(magnitude).unwrap_or(i64::MAX);
        if self.neg { -magnitude } else { magnitude }
    }

    /// What lies past the integer part (`crt_fmod`(x, 1), exact).
    fn fraction(self) -> Self {
        let Ok(down) = u32::try_from(-self.e) else {
            return Self::ZERO;
        };
        let mask = 1u128.checked_shl(down).map_or(u128::MAX, |b| b - 1);
        Self {
            m: self.m & mask,
            ..self
        }
    }

    /// The nearest integer, ties to even, as the executable's 32 bits (0x523397: the value plus
    /// 1.5 × 2^52 at 53 bits, its low word), of a rounded value.
    fn integer(self) -> u32 {
        let whole = match u32::try_from(-self.e) {
            Err(_) | Ok(0) => self.m << u32::try_from(self.e).unwrap_or(0).min(127),
            Ok(down) if down > 127 => 0,
            Ok(down) => {
                let (kept, rest) = (self.m >> down, self.m & ((1 << down) - 1));
                let half = 1 << (down - 1);
                kept + u128::from(rest > half || (rest == half && kept & 1 == 1))
            }
        };
        let low = u32::try_from(whole & u128::from(u32::MAX)).unwrap_or(0);
        if self.neg { low.wrapping_neg() } else { low }
    }
}

#[cfg(test)]
mod tests;
