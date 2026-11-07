//! The arithmetic of `BINKW32.DLL`'s inverse DCT (E17 M6 T6c1, CLT-Q30), in integers: the x87
//! FPU as the game runs it (control word 0x27F: 53-bit precision, round to nearest, ties to
//! even), so that the DCT blocks' pixels are the DLL's on every platform, without floating point
//! (the client's rule, `clippy::float_arithmetic`).
//!
//! The DLL keeps some intermediates on the x87 stack, where each operation rounds to the
//! precision control's 53 bits ([`X::add`], [`X::sub`], [`X::mul`]: binary64's significand; the
//! stack's wider exponent never matters here, below), and stores others as binary32
//! ([`X::single`]: 24 bits); a row's pixels leave by `fistp word` ([`X::pixel`]). A value is
//! `m × 2^e` with `m` normalized to 53 bits (`2^52 ≤ |m| < 2^53`) or zero, so the operations
//! round exactly as the FPU does: a sum is formed exactly in 128 bits (or, when one operand lies
//! more than 60 binary places below the other, rounds to the larger, as it lies under a quarter
//! of the larger's last place), a product of two 53-bit significands is exact in 106 bits.
//!
//! Exponents never leave the binary32 range on any input the bundles allow: a coefficient is a
//! 16-bit integer, a dequantizer under 2^4 and over 2^-7, a pass multiplies by at most 2.62 and
//! adds eight terms, so a nonzero value lies between about 2^-160 (a difference of two multiples
//! of the inputs' smallest place) and 2^40. So there is no overflow, underflow or denormal to
//! model; zero's sign is not kept (it never reaches a pixel: `fistp` of either zero is 0, and a
//! zero only ever adds or multiplies). *Provisional*.
//!
//! Public only for the integration test `tests/x87.rs`, which checks it against the hardware's
//! IEEE arithmetic (the crate's own sources hold no floating point; `h3-view`'s purity test).

/// A value of the DLL's x87 arithmetic (see the module's doc).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct X {
    /// The significand: 0, or `2^52 ≤ |m| < 2^53`.
    m: i64,
    /// The exponent of `m`'s last place.
    e: i32,
}

/// The precision control's significand bits (0x27F).
const P53: u32 = 53;
/// binary32's.
const P24: u32 = 24;

/// `v × 2^e` rounded to `p` significant bits, ties to even, normalized to 53 bits.
const fn round(v: i128, e: i32, p: u32) -> X {
    if v == 0 {
        return X::ZERO;
    }
    let negative = v < 0;
    let mut a = v.unsigned_abs();
    let bits = 128 - a.leading_zeros();
    let mut e = e;
    if bits > p {
        let shift = bits - p;
        let rest = a & ((1u128 << shift) - 1);
        let half = 1u128 << (shift - 1);
        a >>= shift;
        e += shift as i32;
        if rest > half || (rest == half && a & 1 == 1) {
            a += 1;
            if a == 1u128 << p {
                a >>= 1;
                e += 1;
            }
        }
    }
    // Normalized to 53 bits (a `p`-bit result is exact there).
    let now = 128 - a.leading_zeros();
    let up = P53 - now;
    let m = (a << up) as i64;
    X {
        m: if negative { -m } else { m },
        e: e - up as i32,
    }
}

impl X {
    /// Zero.
    pub const ZERO: Self = Self { m: 0, e: 0 };

    /// `fild`: an integer, exact.
    pub const fn int(v: i32) -> Self {
        round(v as i128, 0, P53)
    }

    /// A binary32 value from its bits (a finite normal one, or zero: the tables').
    pub const fn from_bits32(bits: u32) -> Self {
        let exponent = ((bits >> 23) & 0xFF) as i32;
        let fraction = (bits & 0x7F_FFFF) as i128;
        if exponent == 0 {
            return Self::ZERO;
        }
        let m = fraction | 1 << 23;
        let m = if bits >> 31 == 1 { -m } else { m };
        round(m, exponent - 150, P53)
    }

    /// A binary64 value from its bits (a finite normal one, or zero: the tables').
    pub const fn from_bits64(bits: u64) -> Self {
        let exponent = ((bits >> 52) & 0x7FF) as i32;
        let fraction = (bits & 0xF_FFFF_FFFF_FFFF) as i128;
        if exponent == 0 {
            return Self::ZERO;
        }
        let m = fraction | 1 << 52;
        let m = if bits >> 63 == 1 { -m } else { m };
        round(m, exponent - 1075, P53)
    }

    /// `n / d` (positive integers) rounded to 53 bits: the tables' `fdiv`.
    pub const fn ratio(n: i64, d: i64) -> Self {
        let wide = (n as i128) << 64;
        let sticky = (wide % d as i128 != 0) as i128;
        round((wide / d as i128) << 1 | sticky, -65, P53)
    }

    /// `self × 2^k`, exact.
    pub const fn scaled(self, k: i32) -> Self {
        Self {
            m: self.m,
            e: if self.m == 0 { 0 } else { self.e + k },
        }
    }

    /// `fadd` on the stack: `self + other` rounded to 53 bits.
    pub const fn add(self, other: Self) -> Self {
        if self.m == 0 {
            return other;
        }
        if other.m == 0 {
            return self;
        }
        let (hi, lo) = if self.e >= other.e {
            (self, other)
        } else {
            (other, self)
        };
        let d = (hi.e - lo.e) as u32;
        if d > 60 {
            // |lo| < 2^(lo.e + 53) ≤ 2^(hi.e − 8): under a quarter of hi's last place.
            return hi;
        }
        round(((hi.m as i128) << d) + lo.m as i128, lo.e, P53)
    }

    /// `fsub` on the stack: `self − other` rounded to 53 bits.
    pub const fn sub(self, other: Self) -> Self {
        self.add(Self {
            m: -other.m,
            e: other.e,
        })
    }

    /// `fmul` on the stack: `self × other` rounded to 53 bits.
    pub const fn mul(self, other: Self) -> Self {
        round(self.m as i128 * other.m as i128, self.e + other.e, P53)
    }

    /// `fst dword`: rounded to binary32's 24 bits.
    pub const fn single(self) -> Self {
        round(self.m as i128, self.e, P24)
    }

    /// `fistp word` (rounded to an integer, ties to even; outside −32 768 to 32 767 the
    /// integer indefinite 0x8000), then the word's low byte: the pixel the DLL stores.
    pub const fn pixel(self) -> u8 {
        if self.m == 0 || self.e < -54 {
            // |v| < 2^(53 + e) ≤ 1/4.
            return 0;
        }
        if self.e >= 0 {
            // |v| ≥ 2^52.
            return 0;
        }
        let shift = (-self.e) as u32;
        let a = self.m.unsigned_abs();
        let mut q = a >> shift;
        let rest = a & ((1 << shift) - 1);
        let half = 1 << (shift - 1);
        if rest > half || (rest == half && q & 1 == 1) {
            q += 1;
        }
        let v = if self.m < 0 { -(q as i64) } else { q as i64 };
        if v < -32_768 || v > 32_767 {
            0
        } else {
            v as u8
        }
    }

    /// The value's binary32 bits when it is one (a 24-bit significand in binary32's normal
    /// range, or zero), else none.
    #[must_use]
    pub const fn to_bits32(self) -> Option<u32> {
        if self.m == 0 {
            return Some(0);
        }
        let a = self.m.unsigned_abs();
        let exponent = self.e + 29 + 150;
        if a & ((1 << 29) - 1) != 0 || exponent < 1 || exponent > 254 {
            return None;
        }
        let sign = if self.m < 0 { 1 << 31 } else { 0 };
        Some(sign | (exponent as u32) << 23 | ((a >> 29) as u32 & 0x7F_FFFF))
    }

    /// The value's binary64 bits when it lies in binary64's normal range (or is zero), else none.
    #[must_use]
    pub const fn to_bits64(self) -> Option<u64> {
        if self.m == 0 {
            return Some(0);
        }
        let a = self.m.unsigned_abs();
        let exponent = self.e + 1075;
        if exponent < 1 || exponent > 2046 {
            return None;
        }
        let sign = if self.m < 0 { 1 << 63 } else { 0 };
        Some(sign | (exponent as u64) << 52 | (a & 0xF_FFFF_FFFF_FFFF))
    }
}

#[cfg(test)]
mod tests {
    //! Edges of the rounding, in integers (the hardware's binary64 as the reference:
    //! `tests/x87.rs`).
    use super::*;

    /// 2^53.
    fn big() -> X {
        X::int(1 << 26).mul(X::int(1 << 27))
    }

    #[test]
    fn sums_round_to_even() {
        // A tie at 53 bits: 2^53 + 1 rounds down to 2^53, 2^53 + 3 up to 2^53 + 4.
        assert_eq!(big().add(X::int(1)), big());
        assert_eq!(big().add(X::int(3)), big().add(X::int(4)));
        assert_ne!(big().add(X::int(4)), big());
        // A carry out of the significand: 2^53 − 1 plus a half place.
        let ones = big().sub(X::int(1));
        assert_eq!(ones.add(X::int(1).scaled(-1)), big());
        // One operand far below the other leaves it, either order and sign.
        let tiny = X::int(1).scaled(-70);
        assert_eq!(X::int(1).add(tiny), X::int(1));
        assert_eq!(tiny.sub(X::int(1)), X::int(-1));
        // x − x is zero; zero's identities.
        assert_eq!(X::int(5).sub(X::int(5)), X::ZERO);
        assert_eq!(X::ZERO.add(X::int(2)), X::int(2));
        assert_eq!(X::ZERO.mul(X::int(2)), X::ZERO);
        assert_eq!(X::ZERO.single(), X::ZERO);
        assert_eq!(X::ZERO.scaled(3), X::ZERO);
        assert_eq!(X::from_bits32(0), X::ZERO);
        assert_eq!(X::from_bits64(0), X::ZERO);
        // binary32's rounding: ties to even at 24 bits; a ratio's sticky bit.
        assert_eq!(X::int(16_777_217).single(), X::int(16_777_216));
        assert_eq!(X::int(16_777_219).single(), X::int(16_777_220));
        assert_eq!(X::ratio(6, 3), X::int(2));
        assert_eq!(X::ratio(1, 3).mul(X::int(3)), X::int(1));
    }

    #[test]
    fn pixels_are_fistp_words_low_bytes() {
        let half = |k: i32| X::int(k).scaled(-1);
        assert_eq!(half(5).pixel(), 2);
        assert_eq!(half(7).pixel(), 4);
        assert_eq!(half(-1).pixel(), 0);
        assert_eq!(X::int(1).scaled(-2).pixel(), 0);
        assert_eq!(X::int(1).scaled(-60).pixel(), 0);
        assert_eq!(X::int(300).pixel(), 44);
        assert_eq!(X::int(-1).pixel(), 255);
        assert_eq!(X::ratio(327_674, 10).pixel(), 0xFF);
        assert_eq!(
            half(65_535).pixel(),
            0,
            "32 767.5 rounds to 32 768: indefinite"
        );
        assert_eq!(half(-65_533).pixel(), 2, "−32 766.5 rounds to −32 766");
        assert_eq!(X::int(-32_769).pixel(), 0);
        assert_eq!(big().pixel(), 0);
    }

    #[test]
    fn bits_in_and_out() {
        let one = 0x3F80_0000;
        assert_eq!(X::from_bits32(one), X::int(1));
        assert_eq!(X::from_bits32(one | 1 << 31), X::int(-1));
        assert_eq!(X::int(-1).to_bits32(), Some(one | 1 << 31));
        assert_eq!(X::ZERO.to_bits32(), Some(0));
        assert_eq!(X::int(16_777_217).to_bits32(), None, "25 bits");
        assert_eq!(X::int(1).scaled(-130).to_bits32(), None);
        assert_eq!(X::int(1).scaled(130).to_bits32(), None);
        let two = 0x4000_0000_0000_0000;
        assert_eq!(X::from_bits64(two), X::int(2));
        assert_eq!(X::from_bits64(two | 1 << 63), X::int(-2));
        assert_eq!(X::int(-2).to_bits64(), Some(two | 1 << 63));
        assert_eq!(X::ZERO.to_bits64(), Some(0));
        assert_eq!(X::int(1).scaled(-1100).to_bits64(), None);
        assert_eq!(X::int(1).scaled(1100).to_bits64(), None);
    }
}
