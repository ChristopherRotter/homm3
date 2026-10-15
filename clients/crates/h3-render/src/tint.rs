//! The executable's tint of a screen rectangle (`AdjustHueSaturation` 0x44E940; E17 M3 T4a,
//! `docs/epics/E17-design.md` §3.7): each pixel keeps its brightness, its largest channel, and
//! takes the hue and saturation given, by the HSV conversion with single-precision values on the
//! x87 (at 53 bits, §7.10). The count box's inside is tinted so (0x43E6A0, 0x43E742).
//!
//! Per pixel: each channel `c` scaled to `c × K` with `K = 0x7FFF_FFFF / mask` (unsigned, so the
//! largest channel nears 2^31), V the largest of the three as an f32; with H = f32(hue × 6), i its
//! integer part (`crt_ftol`), f its fraction (`fmod`), s the saturation:
//! p = f32((1 − s) V), q = f32((1 − f s) V), t = f32((1 − (1 − f) s) V), the operands f32, each
//! step at 53 bits; then by i the channels (V, t, p), (q, V, p), (p, V, t), (p, q, V), (t, p, V),
//! (V, p, q), each rounded to an integer (ties to even: the 1.5 × 2^52 addition), divided by its
//! `K` and kept to its mask. For i above 5 the pixel stays as it was (the switch's default).
//!
//! Computed exactly in integers, as the client must (no float arithmetic, ADR-0003 §3): the hue
//! and the saturation as fixed-point values to 2^−48, the products and differences exact, each
//! x87 result and each f32 store rounded to its 53 or 24 significant bits, ties to even.
//! Exact for the values the executable passes (in [0, 1], at least 2^−24 or 0). Outside [0, 1]
//! they are clamped: a hue of 1 or more leaves every pixel as the executable does, a negative
//! one or a saturation above 1 (which the executable would turn into other colours; never
//! passed) is taken as 0 and 1.

use crate::screen::Screen;

/// A tint's hue and saturation as the executable passes them: the bits of two f32 values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tint {
    /// The hue, a sixth of a turn per unit of 1/6 (0 red, 1/3 green, 2/3 blue).
    pub hue: u32,
    /// The saturation.
    pub saturation: u32,
}

/// The fractional bits of the fixed point the hue and saturation take.
const FRAC: u32 = 48;
/// 1 in that fixed point.
const ONE: u128 = 1 << FRAC;

/// Tints the `width` × `height` pixels from (`x`, `y`) with `tint` (0x44E940). The executable
/// cuts the rectangle at the screen's right and bottom edges only; here at every edge (a
/// rectangle past the left or top edge would write outside its rows there).
pub fn adjust_hue_saturation(
    screen: &mut Screen,
    [x, y]: [i32; 2],
    [width, height]: [i32; 2],
    tint: Tint,
) {
    let h6 = round_bits(unit(tint.hue) * 6, 24);
    let Ok(sextant @ 0..=5) = u8::try_from(h6 >> FRAC) else {
        return;
    };
    let f = h6 & (ONE - 1);
    let s = unit(tint.saturation);
    let one_less_s = round_bits(ONE - s, 24);
    // 1 − f s and 1 − (1 − f) s at 53 bits, to 2^−96 (both under 1).
    let one_less_fs = round_bits((ONE << FRAC) - round_bits(f * s, 53), 53);
    let one_less_gs = round_bits((ONE << FRAC) - round_bits((ONE - f) * s, 53), 53);
    let masks = screen.format().channel_masks();
    let scales = masks.map(|m| 0x7FFF_FFFF / m.max(1));
    for row in 0..i64::from(height) {
        for column in 0..i64::from(width) {
            let (px, py) = (i64::from(x) + column, i64::from(y) + row);
            let Some(pixel) = screen.pixel(px, py) else {
                continue;
            };
            let scaled = [0, 1, 2].map(|c| (u32::from(pixel) & masks[c]) * scales[c]);
            let v = round_bits(u128::from(scaled.into_iter().max().unwrap_or(0)), 24);
            // p to 2^−48, q and t to 2^−96, v whole.
            let p = integer(round_bits(one_less_s * v, 24), FRAC);
            let q = integer(round_bits(round_bits(one_less_fs * v, 53), 24), 2 * FRAC);
            let t = integer(round_bits(round_bits(one_less_gs * v, 53), 24), 2 * FRAC);
            let v = integer(v, 0);
            let channels = match sextant {
                0 => [v, t, p],
                1 => [q, v, p],
                2 => [p, v, t],
                3 => [p, q, v],
                4 => [t, p, v],
                _ => [v, p, q],
            };
            let out = (0..3).fold(0, |out, c| out | (channels[c] / scales[c]) & masks[c]);
            screen.set(px, py, u16::try_from(out).unwrap_or(u16::MAX));
        }
    }
}

/// An f32's bits as a value in [0, 1] to 2^−48: the bits below dropped, a negative value 0, one
/// above 1 (an infinity or not a number too) 1.
fn unit(bits: u32) -> u128 {
    let exponent = (bits >> 23) & 0xFF;
    let mantissa = u128::from(bits & 0x7F_FFFF | 0x80_0000);
    if bits >> 31 == 1 || exponent == 0 {
        0
    } else if exponent >= 127 {
        ONE
    } else if exponent >= 102 {
        // The value is mantissa × 2^(exponent − 150): exponent − 102 places above 2^−48.
        mantissa << (exponent - 102)
    } else {
        mantissa >> (102 - exponent)
    }
}

/// `v` rounded to `bits` significant bits, ties to even: an x87 result at its precision, or an
/// f32 store.
fn round_bits(v: u128, bits: u32) -> u128 {
    let width = 128 - v.leading_zeros();
    if width <= bits {
        return v;
    }
    let shift = width - bits;
    let (kept, rest) = (v >> shift, v & ((1 << shift) - 1));
    let half = 1 << (shift - 1);
    let up = rest > half || (rest == half && kept & 1 == 1);
    (kept + u128::from(up)) << shift
}

/// A value with `frac` fractional bits rounded to an integer, ties to even (the 1.5 × 2^52
/// addition at 53 bits, its low word taken), as the executable's 32 bits.
fn integer(v: u128, frac: u32) -> u32 {
    let whole = if frac == 0 {
        v
    } else {
        let (kept, rest) = (v >> frac, v & ((1 << frac) - 1));
        let half = 1 << (frac - 1);
        kept + u128::from(rest > half || (rest == half && kept & 1 == 1))
    };
    u32::try_from(whole & u128::from(u32::MAX)).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests;
