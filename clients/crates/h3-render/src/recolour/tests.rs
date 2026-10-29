use super::*;
use crate::screen::PixelFormat;

const MASKS: [u32; 3] = [0xF800, 0x07E0, 0x001F];

/// FNV-1a over every 16-bit value as `effect` makes it, each value's little-endian bytes.
fn digest(effect: Effect) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in 0..=u16::MAX {
        for b in effect.colour(v, MASKS).to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    }
    h
}

/// White, 0x1234, a grey and pure green.
const SAMPLES: [u16; 4] = [0xFFFF, 0x1234, 0x8410, 0x07E0];

/// `clients/scripts/palette-reference.py`'s digests and samples (IEEE doubles for the x87 at 53
/// bits, float32 for the stores).
const REFERENCE: [(Effect, u64, [u16; 4]); 24] = [
    (
        Effect::Clone,
        0x695c_7e0f_72aa_1a61,
        [0x7BFF, 0x0859, 0x5AF7, 0x001F],
    ),
    (
        Effect::Bloodlust(0),
        0xe27b_9fb2_8146_ae95,
        [0xFFFF, 0x1213, 0x840F, 0x07E0],
    ),
    (
        Effect::Bloodlust(1),
        0x3092_e011_f3c4_3091,
        [0xE79F, 0x0915, 0x8BF1, 0x37E0],
    ),
    (
        Effect::Bloodlust(2),
        0x1a58_32d3_ece5_9b87,
        [0xCF1F, 0x2075, 0x93D1, 0x67E0],
    ),
    (
        Effect::Bloodlust(3),
        0xedae_4beb_5889_19a7,
        [0xBE3F, 0x4876, 0x9BB2, 0x97E0],
    ),
    (
        Effect::Bloodlust(4),
        0xba8f_7cea_c81f_82b6,
        [0xC59F, 0x7077, 0xA391, 0xC7E0],
    ),
    (
        Effect::Bloodlust(5),
        0xcba8_cad3_3b57_20d7,
        [0xD53F, 0x9877, 0xAB91, 0xFFE0],
    ),
    (
        Effect::Bloodlust(6),
        0x1646_a468_f432_1eee,
        [0xE4FF, 0xC076, 0xAB70, 0xFE40],
    ),
    (
        Effect::Bloodlust(7),
        0x5de8_7808_1606_d8ab,
        [0xFCBD, 0xC051, 0xB34F, 0xFCA0],
    ),
    (
        Effect::Bloodlust(8),
        0x233e_20ab_663b_3898,
        [0xFC59, 0xC04C, 0xB32E, 0xFB20],
    ),
    (
        Effect::Bloodlust(9),
        0xe8e1_b7ed_de41_c54b,
        [0xFC34, 0xC846, 0xBB0D, 0xF980],
    ),
    (
        Effect::Bloodlust(10),
        0xdba2_4e98_b642_f3dd,
        [0xFBEF, 0xC841, 0xBAEB, 0xF800],
    ),
    (
        Effect::StoneGlow(0),
        0xee84_99ad_f18b_422a,
        [0xFFFF, 0xA513, 0x840F, 0xFFFE],
    ),
    (
        Effect::StoneGlow(1),
        0x669a_5105_16fc_4144,
        [0xFFFF, 0x94D3, 0x840F, 0xDFFB],
    ),
    (
        Effect::StoneGlow(2),
        0xe251_f34a_b0a1_396c,
        [0xFFFF, 0x8473, 0x840F, 0xC7F8],
    ),
    (
        Effect::StoneGlow(3),
        0x3b3d_347f_7d6e_006a,
        [0xFFFF, 0x7433, 0x840F, 0xAFF5],
    ),
    (
        Effect::StoneGlow(4),
        0x19f8_f881_8171_37e0,
        [0xFFFF, 0x63F3, 0x840F, 0x97F2],
    ),
    (
        Effect::StoneGlow(5),
        0x8bcc_c321_5cc9_8165,
        [0xFFFF, 0x5B93, 0x840F, 0x7FEF],
    ),
    (
        Effect::StoneGlow(6),
        0xf044_7e74_c29b_c455,
        [0xFFFF, 0x4B53, 0x840F, 0x67EC],
    ),
    (
        Effect::StoneGlow(7),
        0x1e1a_e1a0_046b_02d5,
        [0xFFFF, 0x3B13, 0x840F, 0x4FE9],
    ),
    (
        Effect::StoneGlow(8),
        0xe5d5_3a17_9d99_f2c6,
        [0xFFFF, 0x2AB3, 0x840F, 0x37E6],
    ),
    (
        Effect::StoneGlow(9),
        0x5cad_dec3_9d0b_f6e7,
        [0xFFFF, 0x1A73, 0x840F, 0x1FE3],
    ),
    (
        Effect::StoneGlow(10),
        0xe27b_9fb2_8146_ae95,
        [0xFFFF, 0x1213, 0x840F, 0x07E0],
    ),
    (
        Effect::Petrified,
        0x815d_703c_2ffe_d065,
        [0xFFFF, 0xA514, 0x8410, 0xFFFE],
    ),
];

#[test]
fn every_value_is_recoloured_as_the_executable_does() {
    for (effect, expected, samples) in REFERENCE {
        let got = SAMPLES.map(|v| effect.colour(v, MASKS));
        assert_eq!(got, samples, "{effect:?}");
        assert_eq!(digest(effect), expected, "{effect:?}");
    }
}

#[test]
fn the_effects_spare_the_keyed_entries() {
    let mut palette = ScreenPalette([0xFFFF; 256]);
    apply(&mut palette, PixelFormat::RGB565, Effect::Clone);
    assert!(palette.0[..FIRST].iter().all(|&e| e == 0xFFFF));
    assert!(palette.0[FIRST..].iter().all(|&e| e == 0x7BFF));
}

#[test]
fn a_strength_above_one_moves_toward_one() {
    // Steps above 10 the executable never passes (palette-reference.py's functions at step 20):
    // the factor above 1 moves the saturation toward 1 (0x522AB0); white, its blue the largest
    // scaled channel, is a faint cyan (0x523190), and a blue of 1 loses its only level to the
    // f32 store of its scaled value.
    let values = [0x1234, 0xFFFF, 0x0001];
    assert_eq!(
        values.map(|v| Effect::StoneGlow(20).colour(v, MASKS)),
        [0x09F3, 0x7FBF, 0]
    );
    let red = values.map(|v| Effect::Bloodlust(20).colour(v, MASKS));
    assert_eq!(red, [0x06EA, 0x57FD, 0x0540]);
}

#[test]
fn the_numbers_round_add_and_compare_exactly() {
    let one = Num::ONE;
    // A term far below the other only makes the sum sticky: 1 + 2^−200 rounds to 1, 1 − 2^−200
    // to 1 at 53 bits but is below 1.
    let tiny = Num {
        e: -200,
        ..Num::ONE
    };
    assert_eq!(one.add(tiny).x87().cmp(one), Ordering::Equal);
    assert!(one.add(tiny).sticky);
    let below = one.sub(tiny);
    assert_eq!(below.cmp(one), Ordering::Less);
    assert_eq!(below.x87().cmp(one), Ordering::Equal);
    // A term 60 places below: cut bits with a shift the aligned sum keeps.
    let far = Num { e: -60, ..one };
    assert_eq!(one.sub(far).cmp(one), Ordering::Less);
    assert_eq!(far.sub(one).cmp(Num::ZERO), Ordering::Less);
    assert_eq!(Num::ZERO.add(far), far);
    // A half rounds to even, more than a half (sticky) up.
    let three_halves = Num { m: 3, e: -1, ..one };
    assert_eq!(three_halves.integer(), 2);
    assert_eq!(Num { m: 5, e: -1, ..one }.integer(), 2);
    assert_eq!(Num { m: 7, e: -2, ..one }.integer(), 2);
    assert_eq!(Num { e: 3, ..one }.integer(), 8);
    assert_eq!(Num { e: -200, ..one }.integer(), 0);
    assert_eq!(
        Num {
            neg: true,
            ..three_halves
        }
        .integer(),
        u32::MAX - 1
    );
    let cut = Num {
        m: 0b1001,
        e: 0,
        neg: false,
        sticky: true,
    };
    assert_eq!(cut.round(3).m, 0b101);
    // The whole part and the fraction.
    assert_eq!(three_halves.whole(), 1);
    assert_eq!(
        Num {
            neg: true,
            ..three_halves
        }
        .whole(),
        -1
    );
    assert_eq!(
        Num {
            e: 2,
            ..three_halves
        }
        .whole(),
        12
    );
    assert_eq!(Num { e: -200, ..one }.whole(), 0);
    assert_eq!(three_halves.fraction(), Num { m: 1, e: -1, ..one });
    assert_eq!(Num { e: 1, ..one }.fraction(), Num::ZERO);
    assert_eq!(one.fraction().m, 0);
    assert_eq!(Num { e: -200, ..one }.fraction().m, 1);
    // A subnormal f32, a negative one, and nothing divided.
    assert_eq!(Num::from_f32_bits(1), Num { e: -149, ..one });
    assert!(Num::from_f32_bits(0xBF80_0000).neg);
    assert_eq!(Num::ZERO.div(Num::int(6)).m, 0);
}

#[test]
fn a_hue_past_one_wraps_and_a_sixth_sextant_keeps_the_colour() {
    // 0x522C42: a hue moved the shorter way round past 1 has 1 taken off (0.9 toward 0.1 with
    // the weight 1 comes to 0.1); a negative factor leaves its part (0x522C61). 0x523387: a hue
    // whose six times rounds to 6 writes nothing, so the scaled channels come back.
    let f = Num::from_f32_bits;
    let [point_nine, point_one, minus] = [0x3F66_6666, 0x3DCC_CCCD, 0xBF80_0000].map(f);
    let half = Num::HALF;
    let [h, s, v] = recolour(
        [point_nine, half, half],
        [point_one, Num::ONE, minus, minus],
    );
    let [low, high] = [0x3DB8_51EC, 0x3DE1_47AE].map(f);
    assert_eq!(
        (h.cmp(low), h.cmp(high)),
        (Ordering::Greater, Ordering::Less)
    );
    assert_eq!((s, v), (half, half));
    assert_eq!(rgb([Num::ONE, half, half], [1, 2, 3]), [1, 2, 3]);
}
