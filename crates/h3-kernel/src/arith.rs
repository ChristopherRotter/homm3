//! Checked integer helpers for game formulas (ADR-0003 §6: checked operations with defined
//! behavior wherever overflow is possible).

use core::num::NonZeroU64;

/// `floor(a * b / d)` computed exactly in 128-bit arithmetic. `None` if `d == 0` or the result
/// does not fit in `u64`.
pub fn mul_div_floor(a: u64, b: u64, d: u64) -> Option<u64> {
    let product = u128::from(a) * u128::from(b);
    let q = product.checked_div(u128::from(d))?;
    u64::try_from(q).ok()
}

/// The exact product `a · b` as its `(high, low)` 128-bit halves (schoolbook on 64-bit limbs;
/// no overflow for any inputs).
pub fn mul_wide(a: u128, b: u128) -> (u128, u128) {
    let mask = u128::from(u64::MAX);
    let (a1, a0) = (a >> 64, a & mask);
    let (b1, b0) = (b >> 64, b & mask);
    let (p00, p01, p10, p11) = (a0 * b0, a0 * b1, a1 * b0, a1 * b1);
    // At most 3 · (2^64 − 1).
    let mid = (p00 >> 64) + (p01 & mask) + (p10 & mask);
    let low = (p00 & mask) | (mid << 64);
    // The high half of a product below 2^256 fits.
    let high = p11 + (p01 >> 64) + (p10 >> 64) + (mid >> 64);
    (high, low)
}

/// Compares the fractions `a.0 / a.1` and `b.0 / b.1` (positive denominators) exactly, by
/// their cross products `a.0 · b.1` and `b.0 · a.1` in 256 bits ([`mul_wide`]).
pub fn cmp_fractions(a: (u128, u128), b: (u128, u128)) -> core::cmp::Ordering {
    mul_wide(a.0, b.1).cmp(&mul_wide(b.0, a.1))
}

/// Fraction bits of the numerators [`single_precision`] returns.
pub const SINGLE_FRAC_BITS: u32 = 40;

/// The single-precision value of the rate `num / den` (IEEE 754 binary32, rounded to nearest,
/// ties to even), as an exact numerator over `2^SINGLE_FRAC_BITS`; computed in integers, no
/// floats (ADR-0003). The original game stores its damage rates as such values (quirk Q-0005):
/// `single_precision(5, 100)` is `13421773 · 2^12`, i.e. `13421773 / 2^28`, slightly more than
/// 0.05. Every such value of a `u16` fraction is a multiple of `2^-39`, so the result is exact.
/// `None` if `den == 0`.
pub fn single_precision(num: u16, den: u16) -> Option<u64> {
    if den == 0 {
        return None;
    }
    let (n, d) = (u64::from(num), u64::from(den));
    if n == 0 {
        return Some(0);
    }
    // `e = floor(log2(n / d))`, in `-16..=15`: start from the bit lengths and correct by one.
    let bits = |x: u64| 64 - i64::from(x.leading_zeros());
    let mut e = bits(n) - bits(d);
    let below = if e >= 0 { n < d << e } else { n << -e < d };
    if below {
        e -= 1;
    }
    // 24 significant bits: `m = round(n · 2^k / d)` with `k = 23 − e` in `8..=39`, so `n << k`
    // stays below 2^55 and `m <= 2^24`.
    let k = 23 - e;
    let scaled = n << k;
    // Round to nearest. A tie would need `2^(k+1)` to divide `d`, and then `n >= 2^24`; so
    // ties cannot occur for `u16` fractions and "ties to even" needs no code.
    let (mut m, r) = (scaled / d, scaled % d);
    if 2 * r >= d {
        m += 1;
    }
    Some(m << (i64::from(SINGLE_FRAC_BITS) - k))
}

/// The value `num / 2^SINGLE_FRAC_BITS` rounded to single precision (IEEE 754 binary32, to
/// nearest, ties to even), as a numerator over `2^SINGLE_FRAC_BITS`: what the original's x87 code
/// keeps when it stores a sum or difference of single-precision values (quirk Q-0005). Values
/// below `2^-16` keep all their bits (single precision would keep more than the numerator can
/// hold); no rule distinguishes them from their rounding. Saturates at `u64::MAX`.
pub fn round_single(num: u64) -> u64 {
    // 24 significant bits: drop `shift` low bits of a value with `bits` bits.
    let bits = 64 - num.leading_zeros();
    let shift = bits.saturating_sub(24);
    if shift == 0 {
        return num;
    }
    let (q, rest) = (num >> shift, num & ((1 << shift) - 1));
    let half = 1 << (shift - 1);
    let up = rest > half || (rest == half && q & 1 == 1);
    u64::try_from(u128::from(q + u64::from(up)) << shift).unwrap_or(u64::MAX)
}

/// Significant bits the original's x87 arithmetic keeps in every operation (quirk Q-0005): the
/// game never sets the precision control, so it runs at the thread default of Windows, 53 bits
/// (double precision; an inference, `docs/formats/installation.md` §7.10: 24 bits is refuted by
/// batch 1a, 64 bits is to be told apart by batch 2b).
pub const X87_PRECISION: u32 = 53;

/// Significant bits of a single-precision (`f32`) value.
pub const SINGLE_BITS: u32 = 24;

/// Significant bits of a double-precision (`f64`) value.
pub const DOUBLE_BITS: u32 = 53;

/// Fraction bits of an [`X87`] value.
const X87_FRAC: u32 = 64;

/// A nonnegative value of the original's x87 arithmetic, computed exactly in integers (ADR-0003:
/// no floats in logic crates): `numerator / 2^64`, below `2^64`. Each operation rounds its exact
/// result to [`X87_PRECISION`] significant bits, to nearest, ties to even, as the FPU does;
/// [`X87::round`] models a store to `f32` or `f64`. Values below `2^-11` lose bits beyond `2^-64`
/// (no rule uses them). Every value has at most 64 significant bits, so products are exact in
/// `u128` before their rounding. Results that would reach `2^64` saturate at [`X87::MAX`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct X87(u128);

impl X87 {
    /// Zero.
    pub const ZERO: Self = Self(0);
    /// One.
    pub const ONE: Self = Self(1 << X87_FRAC);
    /// The largest value, `2^64 − 1` (an integer, so 64 significant bits).
    pub const MAX: Self = Self((u64::MAX as u128) << X87_FRAC);

    /// The integer `n`, exact (as `fild` loads it).
    pub const fn int(n: u64) -> Self {
        Self((n as u128) << X87_FRAC)
    }

    /// The value `num / den` rounded to `bits` significant bits (a constant of the code: 24 for
    /// `f32`, 53 for `f64`). `None` if `den == 0`.
    pub fn ratio(num: u32, den: u32, bits: u32) -> Option<Self> {
        if den == 0 {
            return None;
        }
        if num == 0 {
            return Some(Self::ZERO);
        }
        let (n, d) = (u128::from(num), u128::from(den));
        // `e = floor(log2(n / d))`, in `-32..=31`.
        let len = |x: u128| 128 - i64::from(x.leading_zeros());
        let mut e = len(n) - len(d);
        let below = if e >= 0 { n < d << e } else { n << -e < d };
        if below {
            e -= 1;
        }
        // `m = round(n · 2^k / d)` with `k = bits − 1 − e`, so `m` has `bits` bits (or is
        // `2^bits` after rounding up); `n << k` stays below 2^(32 + 32 + bits), `d << -k` below
        // 2^(64 − bits).
        let k = i64::from(bits) - 1 - e;
        let (scaled, d) = if k >= 0 { (n << k, d) } else { (n, d << -k) };
        let (mut m, r) = (scaled / d, scaled % d);
        if 2 * r > d || (2 * r == d && m & 1 == 1) {
            m += 1;
        }
        // The value is `m · 2^-k`; as a numerator over 2^64.
        let shift = i64::from(X87_FRAC) - k;
        Some(Self(if shift >= 0 { m << shift } else { m >> -shift }))
    }

    /// The value rounded to `bits` significant bits, to nearest, ties to even (a store to `f32`
    /// with [`SINGLE_BITS`] or to `f64` with [`DOUBLE_BITS`]).
    pub fn round(self, bits: u32) -> Self {
        let len = 128 - self.0.leading_zeros();
        if len <= bits {
            return self;
        }
        let k = len - bits;
        let (q, rest) = (self.0 >> k, self.0 & ((1 << k) - 1));
        let half = 1 << (k - 1);
        let up = rest > half || (rest == half && q & 1 == 1);
        // `q + up <= 2^bits`, and the shift back stays below 2^128 unless the value saturates.
        match (q + u128::from(up)).checked_mul(1 << k) {
            Some(v) if v <= Self::MAX.0 => Self(v),
            _ => Self::MAX,
        }
    }

    /// `self · rhs` as the FPU computes it: the exact product rounded to [`X87_PRECISION`] bits.
    pub fn times(self, rhs: Self) -> Self {
        if self.0 == 0 || rhs.0 == 0 {
            return Self::ZERO;
        }
        // Both mantissas have at most 64 significant bits, so their product fits in u128.
        let (ta, tb) = (self.0.trailing_zeros(), rhs.0.trailing_zeros());
        let product = Self((self.0 >> ta) * (rhs.0 >> tb)).round(X87_PRECISION).0;
        // The value is `product · 2^(ta + tb − 128)`; as a numerator over 2^64.
        let shift = i64::from(ta + tb) - i64::from(X87_FRAC);
        if shift < 0 {
            return Self(product >> -shift);
        }
        let shift = shift.unsigned_abs() as u32;
        if product.leading_zeros() < shift {
            return Self::MAX;
        }
        Self((product << shift).min(Self::MAX.0))
    }

    /// `self + rhs` as the FPU computes it, rounded to [`X87_PRECISION`] bits.
    pub fn plus(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0).min(Self::MAX.0)).round(X87_PRECISION)
    }

    /// `self / rhs` as the FPU computes it (`fdiv`): the exact quotient rounded to
    /// [`X87_PRECISION`] bits, to nearest, ties to even (E08 T19a2b: Necromancy's count). `None`
    /// if `rhs` is 0; a quotient of `2^64` or more saturates at [`X87::MAX`], one below `2^-64`
    /// is 0.
    pub fn over(self, rhs: Self) -> Option<Self> {
        if rhs.0 == 0 {
            return None;
        }
        if self.0 == 0 {
            return Some(Self::ZERO);
        }
        // `self / rhs = (a · 2^ta) / (b · 2^tb)` with odd mantissas of at most 64 bits (every
        // value has at most 64 significant bits).
        let (ta, tb) = (self.0.trailing_zeros(), rhs.0.trailing_zeros());
        let (a, b) = (self.0 >> ta, rhs.0 >> tb);
        // `e = floor(log2(a / b))`, in `-64..=63`.
        let len = |x: u128| 128 - i64::from(x.leading_zeros());
        let mut e = len(a) - len(b);
        let below = if e >= 0 { a < b << e } else { a << -e < b };
        if below {
            e -= 1;
        }
        // `m = round(a · 2^k / b)` with `k = PRECISION − 1 − e`, so `m` has `PRECISION` bits
        // (or is `2^PRECISION` after rounding up); `a << k` stays below 2^(PRECISION + 1 +
        // len(b)) ≤ 2^118, `b << -k` below 2^(64 + 11).
        let k = i64::from(X87_PRECISION) - 1 - e;
        let (num, den) = if k >= 0 { (a << k, b) } else { (a, b << -k) };
        let (mut m, r) = (num / den, num % den);
        if 2 * r > den || (2 * r == den && m & 1 == 1) {
            m += 1;
        }
        // The value is `m · 2^(ta − tb − k)`; as a numerator over 2^64.
        let shift = i64::from(ta) - i64::from(tb) - k + i64::from(X87_FRAC);
        Some(if shift < 0 {
            Self(m >> shift.unsigned_abs().min(127))
        } else if shift >= 128 || m.leading_zeros() < shift.unsigned_abs() as u32 {
            Self::MAX
        } else {
            Self((m << shift).min(Self::MAX.0))
        })
    }

    /// The square root as the FPU's `fsqrt` gives it (the runtime's `sqrt`, E04c T7d: the
    /// computer's worth of a creature): the exact root rounded to [`X87_PRECISION`] bits, to
    /// nearest (an inexact root is never halfway, so ties need no rule).
    pub fn sqrt(self) -> Self {
        // `sqrt(N / 2^64) = sqrt(N · 4^s) / 2^(32 + s)`: the scaled integer root gets 55 to 64
        // bits, at least two below the 53 kept (`len + 2s` is 110, 111 or `len` itself).
        let len = 128 - self.0.leading_zeros();
        let s = (2 * (X87_PRECISION + 2)).saturating_sub(len).div_ceil(2);
        let n = self.0 << (2 * s);
        let r = n.isqrt();
        let k = (128 - r.leading_zeros()).saturating_sub(X87_PRECISION);
        let (q, rest) = (r >> k, r & ((1 << k) - 1));
        let half = 1 << k >> 1;
        // An inexact root lies strictly between `r` and `r + 1`: above half when `rest` is.
        let up = rest > half || (rest == half && (r * r != n || q & 1 == 1));
        let m = (q + u128::from(up)) << k;
        // The value is `m / 2^(32 + s)`, `s` at most 55; below `2^32`, so no saturation.
        if s <= 32 {
            Self(m << (32 - s))
        } else {
            Self(m >> (s - 32))
        }
    }

    /// `self − rhs` as the FPU computes it, rounded to [`X87_PRECISION`] bits; 0 if `rhs` is
    /// larger (no rule subtracts more than it has).
    pub fn minus(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0)).round(X87_PRECISION)
    }

    /// The value of the IEEE 754 binary32 bit pattern `bits` of a nonnegative `f32` (a value the
    /// original stores), exact; a negative, infinite or NaN pattern counts as 0, a value of
    /// `2^64` or more saturates at [`X87::MAX`], a subnormal one is 0.
    pub fn from_f32_bits(bits: u32) -> Self {
        let exponent = (bits >> 23) & 0xff;
        if bits >> 31 != 0 || exponent == 0 || exponent == 0xff {
            return Self::ZERO;
        }
        // `value = m · 2^(exponent − 150)`, a numerator over 2^64: `m · 2^(exponent − 86)`;
        // from exponent 191 on the value is at least 2^64.
        let mantissa = u128::from((bits & 0x7f_ffff) | 0x80_0000);
        if exponent >= 191 {
            Self::MAX
        } else if exponent >= 86 {
            Self(mantissa << (exponent - 86))
        } else {
            Self(mantissa >> (86 - exponent))
        }
    }

    /// The IEEE 754 binary32 bit pattern of the value rounded to single precision (to nearest,
    /// ties to even, [`X87::round`]); 0 for 0.
    pub fn to_f32_bits(self) -> u32 {
        let v = self.round(SINGLE_BITS).0;
        if v == 0 {
            return 0;
        }
        // The leading bit is at `len − 1`, the value's exponent `len − 65`, biased `len + 62`.
        let len = 128 - v.leading_zeros();
        let mantissa = if len >= 24 {
            v >> (len - 24)
        } else {
            v << (24 - len)
        };
        ((len + 62) << 23) | (mantissa as u32 & 0x7f_ffff)
    }

    /// The value `num / 2^SINGLE_FRAC_BITS` (a numerator of [`single_precision`]'s fixed
    /// point), exact.
    pub const fn fixed(num: u64) -> Self {
        Self((num as u128) << (X87_FRAC - SINGLE_FRAC_BITS))
    }

    /// The value as a numerator over `2^SINGLE_FRAC_BITS`, rounded down (exact for a
    /// single-precision value of at least `2^-16`; saturating).
    pub fn to_fixed(self) -> u64 {
        u64::try_from(self.0 >> (X87_FRAC - SINGLE_FRAC_BITS)).unwrap_or(u64::MAX)
    }

    /// The value rounded toward zero (the runtime's `_ftol`).
    pub fn trunc(self) -> u64 {
        (self.0 >> X87_FRAC) as u64
    }

    /// The numerator over `2^64` (tests compare it with IEEE doubles in `num`).
    #[cfg(test)]
    pub(crate) const fn raw(self) -> u128 {
        self.0
    }
}

/// Limbs of [`WideUint`].
const WIDE_LIMBS: usize = 8;

/// An unsigned integer of 512 bits for exact products of up to eight `u64` factors (damage
/// formulas with several rates). Only what those need: add, multiply by a `u64`, divide by a
/// `u64` or a power of two, rounding down.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WideUint {
    /// Little-endian limbs.
    limbs: [u64; WIDE_LIMBS],
    /// Limbs in use (the others are 0).
    len: usize,
}

impl WideUint {
    /// The value `value`.
    pub const fn new(value: u64) -> Self {
        let mut limbs = [0; WIDE_LIMBS];
        limbs[0] = value;
        Self {
            limbs,
            len: if value == 0 { 0 } else { 1 },
        }
    }

    /// The largest value, `2^512 − 1`.
    pub const MAX: Self = Self {
        limbs: [u64::MAX; WIDE_LIMBS],
        len: WIDE_LIMBS,
    };

    /// `self · factor`, saturating at [`WideUint::MAX`] (eight `u64` factors always fit).
    pub fn saturating_mul(self, factor: u64) -> Self {
        let mut out = Self::new(0);
        let mut carry: u128 = 0;
        for (o, &l) in out.limbs.iter_mut().zip(&self.limbs[..self.len]) {
            let p = u128::from(l) * u128::from(factor) + carry;
            *o = p as u64;
            carry = p >> 64;
        }
        out.len = self.len;
        if carry != 0 {
            match out.limbs.get_mut(self.len) {
                Some(top) => *top = carry as u64,
                None => return Self::MAX,
            }
            out.len += 1;
        }
        out.trim();
        out
    }

    /// `self + rhs`, saturating at [`WideUint::MAX`].
    pub fn saturating_add(self, rhs: Self) -> Self {
        let mut out = Self::new(0);
        let mut carry = false;
        for ((o, &a), &b) in out.limbs.iter_mut().zip(&self.limbs).zip(&rhs.limbs) {
            let (sum, c1) = a.overflowing_add(b);
            let (sum, c2) = sum.overflowing_add(u64::from(carry));
            *o = sum;
            carry = c1 || c2;
        }
        if carry {
            return Self::MAX;
        }
        out.len = WIDE_LIMBS;
        out.trim();
        out
    }

    /// `floor(self / divisor)`.
    pub fn div_floor(self, divisor: NonZeroU64) -> Self {
        let d = u128::from(divisor.get());
        let mut out = self;
        let mut rem: u128 = 0;
        for l in out.limbs[..self.len].iter_mut().rev() {
            let cur = (rem << 64) | u128::from(*l);
            *l = (cur / d) as u64;
            rem = cur % d;
        }
        out.trim();
        out
    }

    /// `floor(self / 2^bits)`.
    pub fn shr_floor(self, bits: u32) -> Self {
        let (skip, shift) = ((bits / 64) as usize, bits % 64);
        let mut out = Self::new(0);
        if skip >= self.len {
            return out;
        }
        let src = &self.limbs[skip..self.len];
        for (i, o) in out.limbs.iter_mut().take(src.len()).enumerate() {
            let hi = src.get(i + 1).copied().unwrap_or(0);
            *o = if shift == 0 {
                src[i]
            } else {
                (src[i] >> shift) | (hi << (64 - shift))
            };
        }
        out.len = src.len();
        out.trim();
        out
    }

    /// The value, if it fits in a `u64`.
    pub fn to_u64(self) -> Option<u64> {
        if self.len <= 1 {
            Some(self.limbs[0])
        } else {
            None
        }
    }

    /// Drops leading zero limbs from `len`.
    fn trim(&mut self) {
        while self.len > 0 && self.limbs[self.len - 1] == 0 {
            self.len -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn exact_floor_division() {
        assert_eq!(mul_div_floor(7, 3, 2), Some(10));
        assert_eq!(mul_div_floor(u64::MAX, u64::MAX, u64::MAX), Some(u64::MAX));
        assert_eq!(mul_div_floor(u64::MAX, 2, 1), None);
        assert_eq!(mul_div_floor(1, 1, 0), None);
        assert_eq!(mul_div_floor(0, 5, 3), Some(0));
    }

    #[test]
    fn wide_products_and_fraction_comparisons() {
        assert_eq!(mul_wide(0, u128::MAX), (0, 0));
        assert_eq!(mul_wide(6, 7), (0, 42));
        assert_eq!(mul_wide(1 << 64, 1 << 64), (1, 0));
        // (2^128 − 1)^2 = 2^256 − 2^129 + 1.
        assert_eq!(mul_wide(u128::MAX, u128::MAX), (u128::MAX - 1, 1));
        assert_eq!(cmp_fractions((1, 3), (1, 2)), core::cmp::Ordering::Less);
        assert_eq!(cmp_fractions((2, 4), (1, 2)), core::cmp::Ordering::Equal);
        // Cross products beyond u128: 2^127 / 2^100 = 2^27 > (2^127 − 1) / 2^100.
        assert_eq!(
            cmp_fractions((1 << 127, 1 << 100), ((1 << 127) - 1, 1 << 100)),
            core::cmp::Ordering::Greater
        );
    }

    #[test]
    fn rounding_to_single_precision() {
        let one = 1u64 << SINGLE_FRAC_BITS;
        let rate = |n| single_precision(n, 100).unwrap();
        // Exact values stay; 1 − 0.1 rounds down to 0.89999998 (the original: 10 % hero
        // resistance leaves 89 % to be affected), 1 − 0.05 to 0.94999999.
        assert_eq!(round_single(one), one);
        assert_eq!(round_single(rate(80)), rate(80));
        assert_eq!(round_single(one - rate(10)), 15_099_494 << 16);
        assert!(round_single(one - rate(10)) * 100 < 90 * one);
        assert!(round_single(one - rate(5)) * 100 < 95 * one);
        assert_eq!(round_single(rate(80) - rate(5)), 3 * one / 4);
        // Ties go to the even neighbour; small values keep their bits.
        assert_eq!(round_single((1 << 30) + (1 << 6)), 1 << 30);
        assert_eq!(round_single((1 << 30) + (3 << 6)), (1 << 30) + (1 << 8));
        assert_eq!(round_single((1 << 30) + (1 << 6) + 1), (1 << 30) + (1 << 7));
        assert_eq!(round_single(12_345), 12_345);
        assert_eq!(round_single(0), 0);
        assert_eq!(round_single(u64::MAX), u64::MAX);
    }

    #[test]
    fn single_precision_values() {
        // The rates batch 1a observed (Q-0005): 0.05 = 13421773 / 2^28, 0.025 = 13421773 / 2^29.
        assert_eq!(single_precision(5, 100), Some(13_421_773 << 12));
        assert_eq!(single_precision(25, 1000), Some(13_421_773 << 11));
        assert_eq!(single_precision(500, 10_000), Some(13_421_773 << 12));
        // 0.1 and 0.7 round up and down; binary fractions and integers are exact.
        assert_eq!(single_precision(1000, 10_000), Some(13_421_773 << 13));
        assert_eq!(single_precision(700, 1000), Some(11_744_051 << 16));
        assert_eq!(single_precision(1, 2), Some(1 << 39));
        assert_eq!(single_precision(300, 100), Some(3 << 40));
        assert_eq!(single_precision(1, 1), Some(1 << 40));
        // The extremes: 65535 is exact; 1/65535 just above 2^-16 needs the 2^-39 step.
        assert_eq!(
            single_precision(u16::MAX, 1),
            Some(u64::from(u16::MAX) << 40)
        );
        assert_eq!(single_precision(1, u16::MAX), Some(8_388_736 << 1));
        assert_eq!(single_precision(0, 7), Some(0));
        assert_eq!(single_precision(1, 0), None);
    }

    #[test]
    fn x87_constants() {
        let d = |n, den| X87::ratio(n, den, DOUBLE_BITS).unwrap().0;
        // The code's doubles: 0.05 and 0.025 slightly above the decimals, 0.3 below.
        assert_eq!(d(5, 100), 0x0ccc_cccc_cccc_cd00);
        assert_eq!(d(25, 1000), 0x0666_6666_6666_6680);
        assert_eq!(d(3, 10), 0x4ccc_cccc_cccc_cc00);
        assert_eq!(d(3, 1), 3 << 64);
        assert_eq!(X87::ratio(1, 1, DOUBLE_BITS), Some(X87::ONE));
        // f32(0.85) is above 0.85; a ratio above 2^24 rounds to 24 bits (here up to 2^32).
        assert_eq!(
            X87::ratio(85, 100, SINGLE_BITS).unwrap().0,
            0xd999_9a00_0000_0000
        );
        assert_eq!(
            X87::ratio(u32::MAX, 1, SINGLE_BITS),
            Some(X87::int(1 << 32))
        );
        // A tie of the ratio goes to the even neighbour: (2^24 + 1) / 2^24 at 24 bits is 1.
        assert_eq!(
            X87::ratio((1 << 24) + 1, 1 << 24, SINGLE_BITS),
            Some(X87::ONE)
        );
        assert_eq!(
            X87::ratio((1 << 24) + 3, 1 << 24, SINGLE_BITS).unwrap().0,
            (1 << 64) + (1 << 42)
        );
        // Below 2^-11 the value keeps only the bits above 2^-64: 2^-31 is exact still.
        assert_eq!(X87::ratio(1, 1 << 31, DOUBLE_BITS), Some(X87(1 << 33)));
        assert_eq!(X87::ratio(0, 7, DOUBLE_BITS), Some(X87::ZERO));
        assert_eq!(X87::ratio(1, 0, DOUBLE_BITS), None);
        // The fixed point of `single_precision`, both ways: f32(0.05) exactly.
        let f = single_precision(500, 10_000).unwrap();
        assert_eq!(X87::fixed(f), X87::ratio(500, 10_000, SINGLE_BITS).unwrap());
        assert_eq!(X87::fixed(f).to_fixed(), f);
        assert_eq!(X87::MAX.to_fixed(), u64::MAX);
        // IEEE binary32 bit patterns, both ways: f32(0.05), 1, 3.6, 2^-41 and 2^-42 (below 2^-41
        // only the bits above 2^-64 are kept), 2^-64, 2^70 (saturates), 0 and the patterns
        // counted as 0.
        let f05 = X87::ratio(500, 10_000, SINGLE_BITS).unwrap();
        assert_eq!(X87::from_f32_bits(0x3d4c_cccd), f05);
        assert_eq!(f05.to_f32_bits(), 0x3d4c_cccd);
        assert_eq!(X87::from_f32_bits(0x3f80_0000), X87::ONE);
        assert_eq!(X87::ONE.to_f32_bits(), 0x3f80_0000);
        assert_eq!(
            X87::ratio(36, 10, SINGLE_BITS).unwrap().to_f32_bits(),
            0x4066_6666
        );
        assert_eq!(X87::from_f32_bits(0x2b00_0000), X87(1 << 23));
        assert_eq!(X87::from_f32_bits(0x2a80_0000), X87(1 << 22));
        assert_eq!(X87(1).to_f32_bits(), 0x1f80_0000);
        assert_eq!(X87::from_f32_bits(0x6280_0000), X87::MAX);
        assert_eq!(X87::from_f32_bits(0x5f7f_ffff).to_f32_bits(), 0x5f7f_ffff);
        assert_eq!(X87::ZERO.to_f32_bits(), 0);
        for bits in [0, 0x8000_0001, 0x7f80_0000, 0x7fc0_0000, 0x0000_0001] {
            assert_eq!(X87::from_f32_bits(bits), X87::ZERO);
        }
    }

    #[test]
    fn x87_operations() {
        let d = |n, den| X87::ratio(n, den, DOUBLE_BITS).unwrap();
        // 1 − 24 · 0.025 is just below 0.4: 50 base becomes 19 (batch 1a R2).
        let f24 = X87::ONE.minus(X87::int(24).times(d(25, 1000)));
        assert_eq!(f24.0, 0x6666_6666_6666_6000);
        assert_eq!(X87::int(50).times(f24).trunc(), 19);
        // 1 − 0.025 is just below 0.975; at 53 bits 40 · it rounds back up to 39.
        let f1 = X87::ONE.minus(d(25, 1000));
        assert_eq!(f1.0, 0xf999_9999_9999_9800);
        assert_eq!(X87::int(40).times(f1).trunc(), 39);
        // 3 · 0.05 rounds above 0.15.
        assert_eq!(X87::int(3).times(d(5, 100)).0, 0x2666_6666_6666_6800);
        // Sums round to 53 bits; differences stop at 0.
        let tiny = X87(1);
        assert_eq!(X87::int(1 << 20).plus(tiny), X87::int(1 << 20));
        assert_eq!(X87::ONE.plus(X87::ONE), X87::int(2));
        assert_eq!(X87::ONE.minus(X87::int(2)), X87::ZERO);
        assert_eq!(X87::ZERO.times(X87::ONE), X87::ZERO);
        assert_eq!(X87::ONE.times(X87::ZERO), X87::ZERO);
        // Saturation at 2^64 − 1.
        assert_eq!(X87::MAX.times(X87::int(2)), X87::MAX);
        assert_eq!(X87::MAX.plus(X87::MAX), X87::MAX);
        assert_eq!(X87::MAX.trunc(), u64::MAX);
        assert_eq!(X87(u128::MAX).round(DOUBLE_BITS), X87::MAX);
        // A product far below 2^-64 vanishes.
        assert_eq!(X87(1).times(X87(1)), X87::ZERO);
        // Quotients (E08 T19a2b): 1/3 rounds to 53 bits; 2/3 and 1/10 too; 6/3 is exact.
        assert_eq!(X87::ONE.over(X87::int(3)), Some(d(1, 3)));
        assert_eq!(X87::int(2).over(X87::int(3)), Some(d(2, 3)));
        assert_eq!(X87::ONE.over(X87::int(10)), Some(d(1, 10)));
        assert_eq!(X87::int(6).over(X87::int(3)), Some(X87::int(2)));
        assert_eq!(X87::ZERO.over(X87::int(3)), Some(X87::ZERO));
        assert_eq!(X87::ONE.over(X87::ZERO), None);
        // A quotient just below an integer that rounds up to it at 53 bits truncates to it: 3
        // over (1 + 2^-60) is 3 − 3 · 2^-60 + …, within half a unit of 3 at 53 bits.
        assert_eq!(
            X87::int(3).over(X87((1 << 64) + (1 << 4))).unwrap().trunc(),
            3
        );
        // Saturation at 2^64 − 1, and quotients below 2^-64.
        assert_eq!(X87::MAX.over(X87(1)), Some(X87::MAX));
        assert_eq!(X87::int(1 << 40).over(X87(1 << 20)), Some(X87::MAX));
        assert_eq!(X87(1).over(X87::MAX), Some(X87::ZERO));
        // A quotient halfway between two 53-bit values goes to the even one: (2^53 + 1) / 2 down
        // to 2^52, (2^53 + 3) / 2 up to 2^52 + 2.
        let half = |n: u64| X87::int(n).over(X87::int(2));
        assert_eq!(half((1 << 53) + 1), Some(X87::int(1 << 52)));
        assert_eq!(half((1 << 53) + 3), Some(X87::int((1 << 52) + 2)));
    }

    #[test]
    fn x87_square_roots() {
        // Exact roots stay exact; √2 is the double 0x3FF6_A09E_667F_3BCD; 0 stays 0.
        assert_eq!(X87::int(4).sqrt(), X87::int(2));
        assert_eq!(X87::int(1 << 62).sqrt(), X87::int(1 << 31));
        assert_eq!(X87::int(2).sqrt().0, 0x16_A09E_667F_3BCD << 12);
        assert_eq!(X87::ZERO.sqrt(), X87::ZERO);
        // Small values: 2^-60 has the root 2^-30; the double 0.05's root rounds to 53 bits.
        assert_eq!(X87(16).sqrt(), X87(1 << 34));
        let root = X87::ratio(5, 100, DOUBLE_BITS).unwrap().sqrt();
        assert_eq!(root.round(DOUBLE_BITS), root);
        // 0.2236067977499789… = 0x3FCC_9F25_C5BF_EDD9 as a double.
        assert_eq!(root.0, 0x1C_9F25_C5BF_EDD9 << 9);
        // The largest value: √(2^64 − 1) rounds to 2^32 at 53 bits.
        assert_eq!(X87::MAX.sqrt(), X87::int(1 << 32));
    }

    #[test]
    fn x87_rounding_ties_go_to_even() {
        let one = X87::ONE.0;
        // 1 + 2^-24 is halfway between 1 and 1 + 2^-23 at 24 bits: even, so 1; 1 + 3 · 2^-24 is
        // halfway between 1 + 2^-23 and 1 + 2^-22: the even one is 1 + 2^-22.
        assert_eq!(X87(one + (1 << 40)).round(SINGLE_BITS), X87::ONE);
        assert_eq!(X87(one + (3 << 40)).round(SINGLE_BITS).0, one + (1 << 42));
        assert_eq!(
            X87(one + (1 << 40) + 1).round(SINGLE_BITS).0,
            one + (1 << 41)
        );
        assert_eq!(X87::int(3).round(SINGLE_BITS), X87::int(3));
    }

    proptest! {
        // Below 2^128 the wide product is the plain one; shifted by 64 bits it moves across
        // the halves.
        #[test]
        fn wide_products_agree_with_u128(a: u64, b: u64) {
            let (a, b) = (u128::from(a), u128::from(b));
            prop_assert_eq!(mul_wide(a, b), (0, a * b));
            prop_assert_eq!(mul_wide(a << 64, b), ((a * b) >> 64, (a * b) << 64));
        }
    }

    proptest! {
        // The result is the nearest 24-bit value: within half a unit of the last place.
        #[test]
        fn single_precision_is_nearest(num in 1u16.., den in 1u16..) {
            let v = single_precision(num, den).unwrap();
            let lead = 63 - v.leading_zeros();
            // 24 significant bits at most (2^24 itself rounds up to the next power).
            prop_assert!(v.trailing_zeros() + 23 >= lead);
            let ulp = 1u128 << (lead - 23);
            let exact_scaled = u128::from(num) << SINGLE_FRAC_BITS; // / den
            let diff = (u128::from(v) * u128::from(den)).abs_diff(exact_scaled);
            prop_assert!(2 * diff <= ulp * u128::from(den));
        }

        // The root has at most 53 significant bits and is within half a unit of the last place
        // of the exact one: (r ∓ ulp / 2)² brackets the value, compared exactly in 256 bits.
        #[test]
        fn x87_sqrt_is_nearest(n in 1u128 << 53..u128::from(u64::MAX) << 64) {
            let r = X87(n).sqrt().0;
            let lead = 127 - r.leading_zeros();
            prop_assert!(r.trailing_zeros() + 52 >= lead);
            // In units of 2^-65: the root r · 2 ± ulp, squared, against n · 2^66.
            let ulp = 1u128 << (lead - 52);
            let (lo, hi) = (2 * r - ulp, 2 * r + ulp);
            let target = (n >> 62, n << 66);
            prop_assert!(mul_wide(lo, lo) <= target);
            prop_assert!(target <= mul_wide(hi, hi));
        }

        #[test]
        fn f32_bits_round_trip(bits in 0x2b00_0000u32..0x5f80_0000) {
            // Exact from 2^-41 (exponent 86) up to below 2^64.
            prop_assert_eq!(X87::from_f32_bits(bits).to_f32_bits(), bits);
        }

        #[test]
        fn wide_matches_u128(a in proptest::num::u64::ANY, b in proptest::num::u64::ANY,
                             d in 1u64.., s in 0u32..140) {
            let p = u128::from(a) * u128::from(b);
            let w = WideUint::new(a).saturating_mul(b);
            let d = NonZeroU64::new(d).unwrap();
            let lo = |w: WideUint| u128::from(w.limbs[0]) | (u128::from(w.limbs[1]) << 64);
            prop_assert_eq!(lo(w), p);
            prop_assert_eq!(lo(w.div_floor(d)), p / u128::from(d.get()));
            prop_assert_eq!(lo(w.shr_floor(s)), p.checked_shr(s).unwrap_or(0));
            let sum = WideUint::new(a).saturating_add(WideUint::new(b));
            prop_assert_eq!(lo(sum), u128::from(a) + u128::from(b));
        }
    }

    #[test]
    fn wide_products() {
        let max = WideUint::new(u64::MAX);
        let mut w = max;
        for _ in 0..7 {
            w = w.saturating_mul(u64::MAX);
        }
        // (2^64 − 1)^8 needs all 512 bits; one more factor saturates, a factor 1 does not.
        assert_eq!(w.len, 8);
        assert_eq!(w.saturating_mul(u64::MAX), WideUint::MAX);
        assert_eq!(w.saturating_mul(1), w);
        assert_eq!(w.to_u64(), None);
        // Dividing by the same factors gives the start back, shifting everything gives 0.
        let mut back = w;
        for _ in 0..7 {
            back = back.div_floor(NonZeroU64::MAX);
        }
        assert_eq!(back, max);
        assert_eq!(back.to_u64(), Some(u64::MAX));
        assert_eq!(w.shr_floor(512), WideUint::new(0));
        // Its top limb is 2^64 − 8 ((2^64 − 1)^8 = 2^512 − 8 · 2^448 + …).
        assert_eq!(w.shr_floor(448), WideUint::new(u64::MAX - 7));
        assert_eq!(w.shr_floor(449), WideUint::new((u64::MAX - 7) >> 1));
        assert_eq!(WideUint::new(0).saturating_mul(5), WideUint::new(0));
        // Sums carry into the next limb and saturate at the top.
        let two64 = WideUint::new(u64::MAX).saturating_add(WideUint::new(1));
        assert_eq!(two64.shr_floor(64), WideUint::new(1));
        assert_eq!(two64.to_u64(), None);
        assert_eq!(w.saturating_add(WideUint::new(0)), w);
        assert_eq!(
            WideUint::MAX.saturating_add(WideUint::new(1)),
            WideUint::MAX
        );
        assert_eq!(WideUint::new(0).to_u64(), Some(0));
        // 2^128 · 3 shifted by 128 is 3 (whole-limb shift).
        let big = WideUint::new(1 << 32)
            .saturating_mul(1 << 32)
            .saturating_mul(1 << 32)
            .saturating_mul(1 << 32)
            .saturating_mul(3);
        assert_eq!(big.shr_floor(128).to_u64(), Some(3));
    }
}
