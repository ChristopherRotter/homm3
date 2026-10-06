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

/// Limbs of [`WideUint`].
const WIDE_LIMBS: usize = 8;

/// An unsigned integer of 512 bits for exact products of up to eight `u64` factors (damage
/// formulas with several rates). Only what those need: multiply by a `u64`, divide by a `u64`
/// or a power of two, rounding down.
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
