use super::*;
use crate::screen::PixelFormat;
use alloc::vec::Vec;

/// A 256 × 256 screen holding every 16-bit pixel once, row-major.
fn every_pixel() -> Screen {
    let mut screen = Screen::new(256, 256, PixelFormat::RGB565).unwrap();
    for p in 0..=u16::MAX {
        screen.set(i64::from(p % 256), i64::from(p / 256), p);
    }
    screen
}

/// FNV-1a over the pixels' little-endian bytes.
fn digest(screen: &Screen) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for p in screen.pixels() {
        for b in p.to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    }
    h
}

/// The f32 bits of the count box's hue for a spells' balance r: (r + 1) × 0.1667 (0x43E70E).
const GOOD: u32 = 0x3EAA_B368;
const MIXED: u32 = 0x3E2A_B368;
const THIRD: u32 = 0x3E63_99E0;
/// 0.75 and 0.8: the count box without spells (0x43E682).
const NONE: Tint = Tint {
    hue: 0x3F40_0000,
    saturation: 0x3F4C_CCCD,
};

#[test]
fn every_pixel_is_tinted_as_the_executable_does() {
    // The digests of all 65 536 pixels tinted by a reference in IEEE doubles (the x87 at 53
    // bits) with f32 stores, and three pixels each: white, a grey of the count box and 0x1234.
    let cases = [
        (NONE, 0x3f4a_1949_2640_ee72, [0x919F, 0x6916, 0x6113]),
        (
            Tint { hue: GOOD, ..NONE },
            0xecfb_5107_e2a9_a6e4,
            [0x37E6, 0x25A4, 0x2504],
        ),
        (
            Tint { hue: MIXED, ..NONE },
            0x9ef3_fbcc_fa28_6bb2,
            [0xF7E6, 0xB5A4, 0x9D03],
        ),
        (
            Tint { hue: 0, ..NONE },
            0x91e6_6eba_0e76_b992,
            [0xF986, 0xB104, 0xA103],
        ),
        (
            Tint { hue: THIRD, ..NONE },
            0xc1a1_3134_904f_2a0a,
            [0xB7E6, 0x85A4, 0x7503],
        ),
        (
            Tint {
                hue: 0x3F00_0000,
                saturation: 0x3E99_999A,
            },
            0x784e_d45a_7c31_39aa,
            [0xAFFF, 0x7DB6, 0x7513],
        ),
        (
            Tint {
                hue: 0x3F7F_BE77,
                saturation: 0x3F80_0000,
            },
            0x27b8_a4ce_e45e_de25,
            [0xF800, 0xB000, 0xA000],
        ),
        // A hue of 1: the switch's default, every pixel as it was.
        (
            Tint {
                hue: 0x3F80_0000,
                saturation: 0x3F00_0000,
            },
            0xba36_b317_0979_df25,
            [0xFFFF, 0xB5B6, 0x1234],
        ),
        // No saturation: grey at the largest channel, which the f32 cuts (0x1234's blue 20 to 19).
        (
            Tint {
                hue: 0x3DCC_CCCD,
                saturation: 0,
            },
            0xee84_99ad_f18b_422a,
            [0xFFFF, 0xB5B6, 0xA513],
        ),
    ];
    for (tint, expected, pixels) in cases {
        let mut screen = every_pixel();
        adjust_hue_saturation(&mut screen, [0, 0], [256, 256], tint);
        assert_eq!(digest(&screen), expected, "{tint:?}");
        let at = |p: u16| {
            screen
                .pixel(i64::from(p % 256), i64::from(p / 256))
                .unwrap()
        };
        assert_eq!([at(0xFFFF), at(0xB5B6), at(0x1234)], pixels, "{tint:?}");
    }
}

#[test]
fn the_rectangle_is_cut_at_the_screen() {
    let mut screen = Screen::new(4, 3, PixelFormat::RGB565).unwrap();
    screen.fill(0xFFFF);
    adjust_hue_saturation(&mut screen, [-1, 1], [3, 9], NONE);
    let rows: Vec<&[u16]> = screen.pixels().chunks(4).collect();
    assert_eq!(rows[0], [0xFFFF; 4]);
    assert_eq!(rows[1], [0x919F, 0x919F, 0xFFFF, 0xFFFF]);
    assert_eq!(rows[2], [0x919F, 0x919F, 0xFFFF, 0xFFFF]);
    // Nothing for an empty or negative size.
    let before = screen.clone();
    adjust_hue_saturation(&mut screen, [0, 0], [0, 3], NONE);
    adjust_hue_saturation(&mut screen, [0, 0], [-2, 3], NONE);
    assert_eq!(screen, before);
}

#[test]
fn hues_and_saturations_outside_the_unit_are_clamped() {
    assert_eq!(unit(0), 0);
    assert_eq!(unit(0x8000_0000 | 0x3F00_0000), 0);
    // A denormal and a value below 2^-48 vanish; 2^-48 itself is the grid's step.
    assert_eq!(unit(0x0000_0001), 0);
    assert_eq!(unit(0x2700_0000), 0);
    assert_eq!(unit(0x2780_0000), 1);
    assert_eq!(unit(0x3F00_0000), ONE / 2);
    assert_eq!(unit(0x3F80_0000), ONE);
    assert_eq!(unit(0x4000_0000), ONE);
    assert_eq!(unit(0x7F80_0000), ONE);
    // A negative hue is 0 (red), a saturation above 1 is 1.
    let mut a = every_pixel();
    let mut b = every_pixel();
    adjust_hue_saturation(
        &mut a,
        [0, 0],
        [256, 256],
        Tint {
            hue: 0xBF00_0000,
            saturation: 0x4000_0000,
        },
    );
    adjust_hue_saturation(
        &mut b,
        [0, 0],
        [256, 256],
        Tint {
            hue: 0,
            saturation: 0x3F80_0000,
        },
    );
    assert_eq!(a, b);
}

#[test]
fn roundings_are_to_even() {
    // To three bits: 1011 and 1101 tie up and down to the even 110; 1111 carries.
    assert_eq!(round_bits(11, 3), 12);
    assert_eq!(round_bits(9, 3), 8);
    assert_eq!(round_bits(13, 3), 12);
    assert_eq!(round_bits(15, 3), 16);
    assert_eq!(round_bits(21, 3), 20);
    assert_eq!(round_bits(23, 3), 24);
    assert_eq!(round_bits(5, 8), 5);
    // To integers: 2.5 to 2, 3.5 to 4, 2.75 to 3.
    assert_eq!(integer(5, 1), 2);
    assert_eq!(integer(7, 1), 4);
    assert_eq!(integer(11, 2), 3);
    assert_eq!(integer(7, 0), 7);
    assert_eq!(integer(u128::from(u32::MAX) + 1, 0), 0);
}
