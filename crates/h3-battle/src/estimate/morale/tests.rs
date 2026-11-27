//! The value of a morale or luck change (moved from `h3-battle-ai`'s value tests at E31 T2f,
//! AI-Q37) and the two callers' factors as the executable holds them.

use super::*;

// covers: BAT-AI-016
#[test]
fn a_morale_or_luck_change_with_and_without_the_slip() {
    let (p, n) = morale_values();
    let micro = |m, c, slip| {
        change_value(m, c, (p, n), slip)
            .times(SignedX87::int(1_000_000))
            .ftol()
    };
    // A rise from −3 by 1: with the slip valued as crossing 0, −2 × 0.0173 + 3 × 0.0833; as the
    // rule, 0.0833.
    assert_eq!(micro(-3, 1, true), 215_300);
    assert_eq!(micro(-3, 1, false), 83_300);
    // A rise that crosses 0 (from −1 by 2): 0.0173 + 0.0833 either way.
    assert_eq!(micro(-1, 2, true), 100_600);
    assert_eq!(micro(-1, 2, false), 100_600);
    // Above 0, cut at 3: from 1 by 5 counts 2; nothing at 3.
    assert_eq!(micro(1, 5, false), 34_600);
    assert_eq!(micro(3, 1, false), 0);
    // Falls: below 0 each point costs 0.0833, cut at −3; above 0 0.0173; across 0 both; nothing
    // at −3; a change of 0 is worth nothing.
    assert_eq!(micro(-1, -1, true), -83_300);
    assert_eq!(micro(-2, -3, true), -83_300);
    assert_eq!(micro(2, -1, true), -17_300);
    assert_eq!(micro(1, -3, true), -183_900);
    assert_eq!(micro(-3, -1, true), 0);
    assert_eq!(change_value(0, 0, (p, n), true), SignedX87::ZERO);
}

// covers: BAT-AI-016
#[test]
fn the_factors_are_the_executables() {
    // 0x63B780: the doubles 0.0173 and −0.0833; 0x63B790: the singles 0x3C8DB8BB and
    // 0xBC47E282, widened.
    let (p, n) = morale_values();
    assert_eq!(p, X87::ratio(173, 10_000, DOUBLE_BITS).unwrap());
    assert_eq!(n, X87::ratio(833, 10_000, DOUBLE_BITS).unwrap());
    let (p, n) = luck_values();
    assert_eq!(p.to_f32_bits(), 0x3C8D_B8BB);
    assert_eq!(n.to_f32_bits(), 0x3C47_E282);
    assert_eq!(p, X87::from_f32_bits(0x3C8D_B8BB));
}
