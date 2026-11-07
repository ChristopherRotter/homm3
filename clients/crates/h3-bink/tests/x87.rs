//! `h3_bink::x87` against the hardware's IEEE arithmetic (E17 M6 T6c1): binary64's operations
//! are the x87's at 53-bit precision on values that neither overflow nor underflow (these
//! tests'), its conversion to binary32 the `fst dword`'s, `round_ties_even` `fistp`'s rounding.
//! Floating point lives here, outside the crate's sources (`h3-view`'s purity test).
#![allow(clippy::float_arithmetic, clippy::cast_possible_truncation)]

use h3_bink::x87::X;
use proptest::prelude::*;

fn x(v: f64) -> X {
    X::from_bits64(v.to_bits())
}

fn f(v: X) -> f64 {
    f64::from_bits(v.to_bits64().expect("in binary64's normal range"))
}

/// Random binary64 values in the range the DCT meets, both signs, some of them binary32 values
/// or small integers (whose sums cancel exactly), and zero.
fn value() -> impl Strategy<Value = f64> {
    prop_oneof![
        (any::<u64>(), -60i32..40).prop_map(|(bits, e)| {
            f64::from_bits((bits & 0x800F_FFFF_FFFF_FFFF) | ((1023 + e) as u64) << 52)
        }),
        any::<i16>().prop_map(f64::from),
        (any::<u32>(), -30i32..20).prop_map(|(bits, e)| {
            f64::from(f32::from_bits(
                (bits & 0x807F_FFFF) | ((127 + e) as u32) << 23,
            ))
        }),
        Just(0.0),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20_000))]

    #[test]
    fn the_operations_round_as_binary64(a in value(), b in value()) {
        prop_assert_eq!(f(x(a).add(x(b))), a + b);
        prop_assert_eq!(f(x(a).sub(x(b))), a - b);
        prop_assert_eq!(f(x(a).mul(x(b))), a * b);
        prop_assert_eq!(f(x(a).single()), f64::from(a as f32));
        prop_assert_eq!(x(a).single().to_bits32(), Some((a as f32).to_bits()));
    }

    #[test]
    fn pixels_are_fistp_words_low_bytes(a in value(), k in -40_000i32..40_000) {
        for v in [a, f64::from(k), f64::from(k) + 0.5, f64::from(k) - 0.5, f64::from(k) / 7.0] {
            let r = v.round_ties_even();
            let word = if (-32_768.0..=32_767.0).contains(&r) { r as i16 } else { i16::MIN };
            prop_assert_eq!(x(v).pixel(), word as u8, "{}", v);
        }
    }

    #[test]
    fn integers_ratios_and_binary32_load_exactly(k in any::<i16>(), n in 1i64..1 << 53, d in 1i64..1 << 53, bits in any::<u32>()) {
        prop_assert_eq!(f(X::int(i32::from(k))), f64::from(k));
        // Both exact in binary64, so its division is the quotient rounded once.
        prop_assert_eq!(f(X::ratio(n, d)), n as f64 / d as f64);
        prop_assert_eq!(f(X::ratio(n >> 40, d)), (n >> 40) as f64 / d as f64);
        let e = (bits >> 23) & 0xFF;
        prop_assume!(e != 0 && e != 255);
        prop_assert_eq!(f(X::from_bits32(bits)), f64::from(f32::from_bits(bits)));
    }
}
