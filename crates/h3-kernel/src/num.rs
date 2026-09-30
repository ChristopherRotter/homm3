//! The audited floating-point surface (ADR-0003 §1–4).
//!
//! Logic crates use integers by default. When a rule's original semantics call for IEEE-754
//! arithmetic, they use [`Real`] from this module and nothing else: the float types are forbidden
//! in logic crates outside this file (`arch.toml`), and `clippy::float_arithmetic` is denied
//! everywhere except here.
//!
//! Guarantees:
//! * values are always finite (NaN/±inf are rejected with [`NumError`], never stored);
//! * only exactly-specified IEEE-754 operations are offered (`+ − × ÷`, comparisons, explicit
//!   rounding to integers), which are deterministic on the supported targets
//!   (x86_64, aarch64, wasm32);
//! * no platform `libm`: rounding is implemented with integer conversions, because `core` offers
//!   no `floor`/`trunc`/`sqrt` in `no_std`. `sqrt` and transcendentals will be added as in-repo
//!   software implementations with test vectors when the first rule needs them (YAGNI).
#![allow(clippy::float_arithmetic)]

use core::fmt;

use crate::hash::{CanonicalHash, StateHasher};

/// Errors of the float surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumError {
    /// The value or result is NaN or infinite.
    NonFinite,
    /// Division by zero.
    DivisionByZero,
    /// A conversion to an integer is out of that integer's range.
    OutOfRange,
}

impl fmt::Display for NumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NonFinite => "non-finite floating-point value",
            Self::DivisionByZero => "division by zero",
            Self::OutOfRange => "value out of integer range",
        })
    }
}

/// A finite IEEE-754 binary64 value.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Real(f64);

/// 2^63 as f64 (exactly representable): the exclusive upper bound of `i64`.
const TWO_POW_63: f64 = 9_223_372_036_854_775_808.0;

impl Real {
    /// Zero.
    pub const ZERO: Self = Self(0.0);
    /// One.
    pub const ONE: Self = Self(1.0);

    /// Wraps a finite value.
    pub fn new(value: f64) -> Result<Self, NumError> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(NumError::NonFinite)
        }
    }

    /// Exact conversion from `i32`.
    pub fn from_i32(value: i32) -> Self {
        Self(f64::from(value))
    }

    /// The underlying value.
    pub const fn get(self) -> f64 {
        self.0
    }

    /// `self + rhs`.
    pub fn checked_add(self, rhs: Self) -> Result<Self, NumError> {
        Self::new(self.0 + rhs.0)
    }

    /// `self - rhs`.
    pub fn checked_sub(self, rhs: Self) -> Result<Self, NumError> {
        Self::new(self.0 - rhs.0)
    }

    /// `self * rhs`.
    pub fn checked_mul(self, rhs: Self) -> Result<Self, NumError> {
        Self::new(self.0 * rhs.0)
    }

    /// `self / rhs`.
    pub fn checked_div(self, rhs: Self) -> Result<Self, NumError> {
        if rhs.0 == 0.0 {
            return Err(NumError::DivisionByZero);
        }
        Self::new(self.0 / rhs.0)
    }

    /// Rounds toward zero.
    pub fn trunc_to_i64(self) -> Result<i64, NumError> {
        if self.0 >= TWO_POW_63 || self.0 < -TWO_POW_63 {
            return Err(NumError::OutOfRange);
        }
        // In range, `as` truncates toward zero exactly.
        #[allow(clippy::cast_possible_truncation)]
        let t = self.0 as i64;
        Ok(t)
    }

    /// Rounds toward negative infinity.
    pub fn floor_to_i64(self) -> Result<i64, NumError> {
        let t = self.trunc_to_i64()?;
        // `t` is exactly representable whenever `self` has a fractional part (|self| < 2^52).
        #[allow(clippy::cast_precision_loss)]
        let back = t as f64;
        Ok(if back > self.0 { t - 1 } else { t })
    }

    /// Rounds toward positive infinity.
    pub fn ceil_to_i64(self) -> Result<i64, NumError> {
        let t = self.trunc_to_i64()?;
        #[allow(clippy::cast_precision_loss)]
        let back = t as f64;
        if back < self.0 {
            t.checked_add(1).ok_or(NumError::OutOfRange)
        } else {
            Ok(t)
        }
    }

    /// The IEEE-754 bit pattern (for canonical hashing and diagnostics).
    pub fn to_bits(self) -> u64 {
        // Normalise -0.0 to +0.0 so equal values hash equally.
        if self.0 == 0.0 { 0 } else { self.0.to_bits() }
    }
}

impl CanonicalHash for Real {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u64(self.to_bits());
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::hash::hash_of;
    use std::string::ToString;

    fn r(v: f64) -> Real {
        Real::new(v).unwrap()
    }

    #[test]
    fn rejects_non_finite() {
        assert_eq!(Real::new(f64::NAN), Err(NumError::NonFinite));
        assert_eq!(Real::new(f64::INFINITY), Err(NumError::NonFinite));
        assert_eq!(r(f64::MAX).checked_mul(r(2.0)), Err(NumError::NonFinite));
        assert_eq!(
            r(1.0).checked_div(Real::ZERO),
            Err(NumError::DivisionByZero)
        );
        assert_eq!(
            NumError::NonFinite.to_string(),
            "non-finite floating-point value"
        );
        assert_eq!(NumError::DivisionByZero.to_string(), "division by zero");
        assert_eq!(
            NumError::OutOfRange.to_string(),
            "value out of integer range"
        );
    }

    #[test]
    fn exact_ieee_operations() {
        // Test vectors: results are exactly specified by IEEE-754 round-to-nearest-even.
        assert_eq!(
            r(0.1).checked_add(r(0.2)).unwrap().get().to_bits(),
            0x3fd3_3333_3333_3334
        );
        assert_eq!(
            r(1.0).checked_sub(r(0.9)).unwrap().get().to_bits(),
            0x3fb9_9999_9999_9998
        );
        assert_eq!(
            r(1.1).checked_mul(r(3.0)).unwrap().get().to_bits(),
            0x400a_6666_6666_6667
        );
        assert_eq!(
            Real::ONE.checked_div(r(3.0)).unwrap().get().to_bits(),
            0x3fd5_5555_5555_5555
        );
        assert_eq!(Real::from_i32(-7).get(), -7.0);
    }

    #[test]
    fn rounding_to_integers() {
        assert_eq!(r(2.7).trunc_to_i64(), Ok(2));
        assert_eq!(r(-2.7).trunc_to_i64(), Ok(-2));
        assert_eq!(r(2.7).floor_to_i64(), Ok(2));
        assert_eq!(r(-2.7).floor_to_i64(), Ok(-3));
        assert_eq!(r(-3.0).floor_to_i64(), Ok(-3));
        assert_eq!(r(2.1).ceil_to_i64(), Ok(3));
        assert_eq!(r(-2.1).ceil_to_i64(), Ok(-2));
        assert_eq!(r(4.0).ceil_to_i64(), Ok(4));
        // 0.7 * 3 is 2.0999999999999996 in binary64 and 1.1 * 3 is 3.3000000000000003: the
        // kind of representation error the original's float rules can expose.
        let x = r(0.7).checked_mul(r(3.0)).unwrap();
        assert_eq!(x.floor_to_i64(), Ok(2));
        let y = r(1.1).checked_mul(r(3.0)).unwrap();
        assert_eq!(y.ceil_to_i64(), Ok(4));
        assert_eq!(r(TWO_POW_63).trunc_to_i64(), Err(NumError::OutOfRange));
        assert_eq!(r(-TWO_POW_63).trunc_to_i64(), Ok(i64::MIN));
        assert_eq!(r(-1e19).floor_to_i64(), Err(NumError::OutOfRange));
        assert_eq!(r(1e19).ceil_to_i64(), Err(NumError::OutOfRange));
        // Largest double below 2^63 is an integer; ceil must not overflow.
        assert_eq!(
            r(9_223_372_036_854_774_784.0).ceil_to_i64(),
            Ok(9_223_372_036_854_774_784)
        );
    }

    #[test]
    fn hashing_normalises_signed_zero() {
        assert_eq!(hash_of(&r(0.0)), hash_of(&r(-0.0)));
        assert_ne!(hash_of(&r(1.0)), hash_of(&r(-1.0)));
        assert_eq!(r(1.5).to_bits(), 1.5f64.to_bits());
    }
}
