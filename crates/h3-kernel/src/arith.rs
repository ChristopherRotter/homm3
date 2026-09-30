//! Checked integer helpers for game formulas (ADR-0003 §6: checked operations with defined
//! behavior wherever overflow is possible).

/// `floor(a * b / d)` computed exactly in 128-bit arithmetic. `None` if `d == 0` or the result
/// does not fit in `u64`.
pub fn mul_div_floor(a: u64, b: u64, d: u64) -> Option<u64> {
    let product = u128::from(a) * u128::from(b);
    let q = product.checked_div(u128::from(d))?;
    u64::try_from(q).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_floor_division() {
        assert_eq!(mul_div_floor(7, 3, 2), Some(10));
        assert_eq!(mul_div_floor(u64::MAX, u64::MAX, u64::MAX), Some(u64::MAX));
        assert_eq!(mul_div_floor(u64::MAX, 2, 1), None);
        assert_eq!(mul_div_floor(1, 1, 0), None);
        assert_eq!(mul_div_floor(0, 5, 3), Some(0));
    }
}
