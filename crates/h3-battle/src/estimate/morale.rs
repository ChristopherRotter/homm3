//! The computer's value of a morale or luck change (`ai_morale_change_value` 0x435850 with its
//! two callers' factors, `ai_morale_change` 0x435830 and `ai_luck_change` 0x435960;
//! `installation.md` §7.46). The battle AI's evaluators read it (E23 T4c, BAT-AI-016: Mirth, the
//! harms, the removals) and so does the adventure's computer player (E31 T2f: the artifacts'
//! morale and luck, later the objects' and the stacks'). It moved here from `h3-battle-ai` at
//! E31 T2f (AI-Q37) so that both read one implementation.

use h3_kernel::arith::{DOUBLE_BITS, SINGLE_BITS, SignedX87, X87};

#[cfg(test)]
mod tests;

/// The highest and lowest morale or luck the values count (the callers' clamp to ±3).
pub const LIMIT: i32 = 3;

/// The values of a morale point above 0 and of one below it (`ai_morale_factors` 0x63B780,
/// doubles: 0.0173 and −0.0833, the second as its magnitude): [`change_value`]'s `(p, n)`.
pub fn morale_values() -> (X87, X87) {
    let value = |thousandths| X87::ratio(thousandths, 10_000, DOUBLE_BITS).unwrap_or(X87::ZERO);
    (value(173), value(833))
}

/// The values of a luck point above 0 and of one below it (`ai_luck_factors` 0x63B790, singles
/// widened to doubles: 0.0173 and −0.0122, the second as its magnitude).
pub fn luck_values() -> (X87, X87) {
    let value = |thousandths| X87::ratio(thousandths, 10_000, SINGLE_BITS).unwrap_or(X87::ZERO);
    (value(173), value(122))
}

/// `n × f` for an integer `n` and a positive factor `f`, rounded as the FPU does.
fn product(n: i32, f: X87) -> SignedX87 {
    SignedX87::new(n < 0, X87::int(u64::from(n.unsigned_abs())).times(f))
}

/// The value of a change of morale or luck by `c` from `m` (0x435850; BAT-AI-016), with the
/// values `(p, n)` of a point above 0 and the magnitude of one below (each point below 0 costs
/// `n`): the change of `V`, `V(m) = p × m` above 0 and `−n × m` below, the result clamped to ±3;
/// 0 for a rise at 3 or a fall at −3. With `slip` (the battle's BAT-QUIRK-25; the computer's as
/// read) a rise from below 0 is always valued as one that crosses 0 (survey slip 12: the test
/// compares the change with `m`, not with `−m`): from −3 by 1 it gives `−2p + 3n`, not `n`.
// rule: BAT-AI-016
pub fn change_value(m: i32, c: i32, (p, n): (X87, X87), slip: bool) -> SignedX87 {
    if c > 0 {
        if m >= LIMIT {
            return SignedX87::ZERO;
        }
        let c = c.min(LIMIT - m);
        if m >= 0 {
            return product(c, p);
        }
        let stays_below = if slip { c <= m } else { c <= -m };
        if stays_below {
            return product(c, n);
        }
        return product(m + c, p).plus(product(-m, n));
    }
    if m <= -LIMIT {
        return SignedX87::ZERO;
    }
    let c = c.max(-LIMIT - m);
    if m <= 0 {
        return product(c, n);
    }
    if -c <= m {
        return product(c, p);
    }
    product(m, p).negated().plus(product(m + c, n))
}
