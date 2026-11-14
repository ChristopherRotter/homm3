//! The sound decoder's tables in integers (E27 M2 T2): the cosines and sines of the FFT's
//! twiddles and the band scales' powers of ten, each correctly rounded to binary32 (ties to
//! even) from a fixed-point value of 124 fractional bits.
//!
//! The DLL computes them on the x87 (`fcos`, `fsin`, the CRT's `_CIpow`) and stores them as
//! binary32; every one of them is the correctly rounded value of its argument (E27 M2 T2: all
//! 256 scales and the twiddles of the three table sizes, `binkprobe sound-tables`, design §8),
//! as the FPU's results carry more bits than binary32's. The series' truncations stay far below
//! the rounding's half unit (2^-120 against 2^-25 of the value); a value that close to a tie
//! would round otherwise, which the check against the DLL's tables rules out for every value
//! the decoder uses. *Provisional*.

use crate::x87::X;

/// The fixed point's fractional bits.
const FRAC: u32 = 124;
/// ln 10 and ln 2 at 2^-124 (truncated).
const LN10: u128 = 0x24D7_6377_6AAA_2B05_BA95_B58A_E0B4_C28A;
const LN2: u128 = 0x0B17_217F_7D1C_F79A_BC9E_3B39_803F_2F6A;
/// 1 at 2^-124.
const ONE: u128 = 1 << FRAC;

/// `(a × b) >> 124`: the product of two fixed-point values, truncated (the result below 2^128).
const fn mul(a: u128, b: u128) -> u128 {
    const M: u128 = u64::MAX as u128;
    let (a1, a0) = (a >> 64, a & M);
    let (b1, b0) = (b >> 64, b & M);
    let p00 = a0 * b0;
    let p01 = a0 * b1;
    let p10 = a1 * b0;
    let p11 = a1 * b1;
    let mid = (p00 >> 64) + (p01 & M) + (p10 & M);
    let high = p11 + (p01 >> 64) + (p10 >> 64) + (mid >> 64);
    let low = (mid << 64) | (p00 & M);
    (high << (128 - FRAC)) | (low >> FRAC)
}

/// `|x|` at 2^-`frac` (truncated; `|x|` below 2^(127 − `frac`)).
fn fixed(x: X, frac: u32) -> u128 {
    let (m, e) = x.parts();
    let shift = i64::from(e) + i64::from(frac);
    let a = u128::from(m.unsigned_abs());
    if shift >= 0 { a << shift } else { a >> -shift }
}

/// A positive fixed-point value times `2^k` rounded to binary32 (ties to even).
fn round32(v: u128, k: i32) -> X {
    if v == 0 {
        return X::ZERO;
    }
    let top = 127 - v.leading_zeros() as i32;
    let shift = top - 23;
    let (mut m, rest, half) = if shift > 0 {
        let s = shift as u32;
        (v >> s, v & ((1 << s) - 1), 1u128 << (s - 1))
    } else {
        (v << -shift, 0, 1)
    };
    if rest > half || (rest == half && m & 1 == 1) {
        m += 1;
    }
    // m below 2^25 (it reached 2^24 only by the rounding): a 24-bit or 25-bit integer exact.
    X::int(m as i32).scaled(shift - FRAC as i32 + k)
}

/// `cos x` rounded to binary32, for `0 ≤ x ≤ π/4`.
pub(crate) fn cos(x: X) -> X {
    let x = fixed(x, FRAC);
    let x2 = mul(x, x);
    let (mut sum, mut term, mut k) = (ONE, ONE, 0u128);
    while term != 0 {
        term = mul(term, x2) / ((k + 1) * (k + 2));
        k += 2;
        sum = if k % 4 == 2 { sum - term } else { sum + term };
    }
    round32(sum, 0)
}

/// `sin x` rounded to binary32, for `0 ≤ x ≤ π/4`.
pub(crate) fn sin(x: X) -> X {
    let x = fixed(x, FRAC);
    let x2 = mul(x, x);
    let (mut sum, mut term, mut k) = (x, x, 1u128);
    while term != 0 {
        term = mul(term, x2) / ((k + 1) * (k + 2));
        k += 2;
        sum = if k % 4 == 3 { sum - term } else { sum + term };
    }
    round32(sum, 0)
}

/// `10^y` rounded to binary32, for `0 ≤ y < 26` (the band scales': `y` at most 255 × 0.0664).
pub(crate) fn pow10(y: X) -> X {
    // y and z = y ln 10 (below 60) at 2^-120: at 2^-124 they would pass 2^128.
    let z = mul(fixed(y, FRAC - 4), LN10);
    let ln2 = LN2 >> 4;
    let k = z / ln2;
    let r = (z - k * ln2) << 4;
    let (mut sum, mut term, mut n) = (ONE, ONE, 0u128);
    while term != 0 {
        n += 1;
        term = mul(term, r) / n;
        sum += term;
    }
    round32(sum, k as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bits(x: X) -> u32 {
        x.to_bits32().unwrap()
    }

    #[test]
    fn the_functions_round_their_values_to_binary32() {
        // cos 0, sin 0, 10^0 and 10^1 exact; π/4's (the binary32 0x3F490FDB) from the DLL's
        // tables.
        assert_eq!(bits(cos(X::ZERO)), 0x3F80_0000);
        assert_eq!(bits(sin(X::ZERO)), 0);
        assert_eq!(bits(pow10(X::ZERO)), 0x3F80_0000);
        assert_eq!(bits(pow10(X::int(1))), 0x4120_0000);
        let quarter = X::from_bits32(0x3F49_0FDB);
        assert_eq!(bits(cos(quarter)), 0x3F35_04F3);
        assert_eq!(bits(sin(quarter)), 0x3F35_04F3);
        // 10^25.
        assert_eq!(bits(pow10(X::int(25))), 0x6904_5951);
        // A value of fewer bits than binary32's, from a fixed point that needs no shift down.
        assert_eq!(bits(round32(1, 124)), 0x3F80_0000);
        assert_eq!(round32(0, 0), X::ZERO);
        // Ties to even: 2^24 + 1 and 2^24 + 3 halfway between two 24-bit values.
        assert_eq!(round32((1 << 24) + 1, 124), X::int(1 << 24));
        assert_eq!(round32((1 << 24) + 3, 124), X::int((1 << 24) + 4));
        assert_eq!(round32((1 << 24) + 2, 124), X::int((1 << 24) + 2));
    }
}
