use super::*;
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use h3_assets::palette::{Palette, Rgb};
use h3_assets::pcx::{IndexedImage, RgbImage};

const F: PixelFormat = PixelFormat::RGB565;
const WHITE: u16 = 0xFFFF;
const OUTLINE: u16 = 0x07E0;

/// Index `i` is the colour `(i, i, i)`, so a drawn index shows as its own grey.
fn palette() -> ScreenPalette {
    let mut p = Palette::default();
    for (i, c) in p.0.iter_mut().enumerate() {
        let v = u8::try_from(i).unwrap();
        *c = Rgb { r: v, g: v, b: v };
    }
    ScreenPalette::new(&p, F)
}

fn grey(i: u8) -> u16 {
    F.pack(Rgb { r: i, g: i, b: i })
}

/// What `drawer` leaves on a white pixel for index `i`, from a run or a literal.
fn result(drawer: Drawer, encoding: Encoding, i: u8, run: bool) -> u16 {
    let mut pixel = WHITE;
    apply(op(drawer, encoding, i, run), &mut pixel, &palette(), F);
    pixel
}

const BODY: u16 = 0x7BEF;
const BORDER: u16 = 22 << 11 | 46 << 5 | 22;

#[test]
fn literal_pixels_are_colours_except_in_the_shadow_pass() {
    for drawer in [
        Drawer::Plain {
            skip_runs: true,
            colour: Some(OUTLINE),
        },
        Drawer::Shadowed {
            colour: Some(OUTLINE),
        },
    ] {
        for encoding in [
            Encoding::Raw,
            Encoding::Rows,
            Encoding::PackedRows,
            Encoding::PackedBlocks,
        ] {
            for i in [0, 1, 4, 5, 6, 7, 8, 200] {
                assert_eq!(
                    result(drawer, encoding, i, false),
                    grey(i),
                    "{drawer:?} {encoding:?} {i}"
                );
            }
        }
    }
    assert_eq!(
        result(Drawer::ShadowPass, Encoding::PackedBlocks, 9, false),
        WHITE
    );
    assert_eq!(
        result(Drawer::ShadowPass, Encoding::PackedBlocks, 1, false),
        WHITE
    );
}

#[test]
fn the_shadowed_drawer_treats_encoding_1_runs_by_index() {
    let none = Drawer::Shadowed { colour: None };
    let outlined = Drawer::Shadowed {
        colour: Some(OUTLINE),
    };
    let expect = |drawer, i| result(drawer, Encoding::Rows, i, true);
    // Without an outline: 1 and 7 a border, 4 and 6 a body, everything else skipped (8 and up too).
    let skipped = [0, 2, 3, 5, 8, 9, 0xFE];
    for i in skipped {
        assert_eq!(expect(none, i), WHITE, "{i}");
    }
    assert_eq!([expect(none, 1), expect(none, 7)], [BORDER; 2]);
    assert_eq!([expect(none, 4), expect(none, 6)], [BODY; 2]);
    // With one: 5, 6 and 7 take it.
    assert_eq!(
        [
            expect(outlined, 5),
            expect(outlined, 6),
            expect(outlined, 7)
        ],
        [OUTLINE; 3]
    );
    assert_eq!([expect(outlined, 1), expect(outlined, 4)], [BORDER, BODY]);
    assert_eq!(expect(outlined, 8), WHITE);
}

#[test]
fn the_plain_drawer_paints_or_skips_encoding_1_runs() {
    let paint = Drawer::Plain {
        skip_runs: false,
        colour: None,
    };
    let skip = Drawer::Plain {
        skip_runs: true,
        colour: None,
    };
    for i in [0, 1, 4, 8, 99] {
        assert_eq!(result(paint, Encoding::Rows, i, true), grey(i));
        assert_eq!(result(skip, Encoding::Rows, i, true), WHITE);
    }
}

#[test]
fn runs_of_the_other_encodings() {
    for drawer in [
        Drawer::Plain {
            skip_runs: false,
            colour: Some(OUTLINE),
        },
        Drawer::Shadowed {
            colour: Some(OUTLINE),
        },
    ] {
        // Encoding 3: a run of 5 takes the flag colour; the rest (shadows too) are skipped.
        assert_eq!(result(drawer, Encoding::PackedBlocks, 5, true), OUTLINE);
        for i in [0, 1, 4, 6] {
            assert_eq!(result(drawer, Encoding::PackedBlocks, i, true), WHITE);
        }
        // Encoding 2: every run skipped. Encoding 0 has no runs; marked ones are drawn.
        assert_eq!(result(drawer, Encoding::PackedRows, 5, true), WHITE);
        assert_eq!(result(drawer, Encoding::Raw, 3, true), grey(3));
    }
    let no_flag = Drawer::Shadowed { colour: None };
    assert_eq!(result(no_flag, Encoding::PackedBlocks, 5, true), WHITE);
}

#[test]
fn the_shadow_pass_draws_only_shadows() {
    for encoding in [Encoding::PackedBlocks, Encoding::Rows] {
        assert_eq!(result(Drawer::ShadowPass, encoding, 1, true), BORDER);
        assert_eq!(result(Drawer::ShadowPass, encoding, 4, true), BODY);
        for i in [0, 2, 5, 6, 7, 8] {
            assert_eq!(result(Drawer::ShadowPass, encoding, i, true), WHITE);
        }
    }
}

/// A frame of `width` × `height` at `(left, top)` in a `canvas`, every pixel literal unless `runs`.
fn frame(
    width: u32,
    left: i32,
    top: i32,
    canvas: (u32, u32),
    pixels: Vec<u8>,
    runs: Vec<bool>,
) -> Frame {
    let height = u32::try_from(pixels.len()).unwrap() / width;
    Frame {
        encoding: Encoding::Rows,
        full_width: canvas.0,
        full_height: canvas.1,
        left,
        top,
        width,
        height,
        pixels,
        runs,
        short_header: false,
    }
}

fn screen(width: u32, height: u32) -> Screen {
    let mut s = Screen::new(width, height, F).unwrap();
    s.fill(WHITE);
    s
}

fn row(s: &Screen, y: i64) -> Vec<u16> {
    (0..i64::from(s.width()))
        .map(|x| s.pixel(x, y).unwrap())
        .collect()
}

const SHADOWED: Drawer = Drawer::Shadowed { colour: None };

#[test]
fn a_frame_sits_at_its_offset_in_its_canvas() {
    let mut s = screen(6, 2);
    let f = frame(2, 1, 1, (4, 2), vec![10, 20], vec![]);
    draw_frame(
        &mut s,
        &f,
        &palette(),
        Placement {
            x: 2,
            y: 0,
            mirrored: false,
        },
        SHADOWED,
    );
    assert_eq!(row(&s, 0), [WHITE; 6]);
    assert_eq!(row(&s, 1), [WHITE, WHITE, WHITE, grey(10), grey(20), WHITE]);
}

#[test]
fn a_mirrored_frame_is_reflected_across_its_own_canvas() {
    // Canvas 4 wide, frame at columns 1–2: mirrored, columns 2–1, reversed.
    let mut s = screen(6, 1);
    let f = frame(2, 1, 0, (4, 1), vec![10, 20], vec![]);
    draw_frame(
        &mut s,
        &f,
        &palette(),
        Placement {
            x: 1,
            y: 0,
            mirrored: true,
        },
        SHADOWED,
    );
    assert_eq!(row(&s, 0), [WHITE, WHITE, grey(20), grey(10), WHITE, WHITE]);
    // A frame at the canvas's left edge lands at its right edge.
    let mut s = screen(4, 1);
    let f = frame(1, 0, 0, (4, 1), vec![10], vec![]);
    draw_frame(
        &mut s,
        &f,
        &palette(),
        Placement {
            mirrored: true,
            ..Placement::default()
        },
        SHADOWED,
    );
    assert_eq!(row(&s, 0), [WHITE, WHITE, WHITE, grey(10)]);
}

#[test]
fn pixels_outside_the_canvas_or_the_screen_are_clipped() {
    // A 3 × 2 frame at (-1, 1) in a 2 × 2 canvas keeps its pixel at canvas (0, 1) and (1, 1) only.
    let mut s = screen(3, 3);
    let f = frame(3, -1, 1, (2, 2), vec![1, 2, 3, 4, 5, 6], vec![false; 6]);
    let plain = Drawer::Plain {
        skip_runs: false,
        colour: None,
    };
    draw_frame(&mut s, &f, &palette(), Placement::default(), plain);
    assert_eq!(row(&s, 0), [WHITE; 3]);
    assert_eq!(row(&s, 1), [grey(2), grey(3), WHITE]);
    assert_eq!(row(&s, 2), [WHITE; 3]);
    // Partly or wholly off the screen on every side: exactly the visible pixels are drawn (no
    // row wraps into its neighbour), nothing breaks.
    let cases: [((i32, i32), &[usize]); 6] = [
        ((-2, 0), &[]),
        ((-1, 1), &[3, 6]),
        ((2, 0), &[2, 5]),
        ((0, -3), &[]),
        ((0, 2), &[6, 7]),
        ((i32::MIN, i32::MAX), &[]),
    ];
    for ((x, y), expected) in cases {
        let mut s = screen(3, 3);
        let f = frame(2, 0, 0, (2, 2), vec![9; 4], vec![]);
        draw_frame(
            &mut s,
            &f,
            &palette(),
            Placement {
                x,
                y,
                mirrored: false,
            },
            plain,
        );
        let drawn: Vec<usize> = (0..9).filter(|&i| s.pixels()[i] != WHITE).collect();
        assert_eq!(drawn, expected, "{x} {y}");
    }
}

#[test]
fn shadows_compound_where_they_overlap() {
    // Two shadow bodies over one pixel quarter it; a border over that darkens it once more.
    let mut s = screen(2, 1);
    let body = frame(2, 0, 0, (2, 1), vec![4, 4], vec![true; 2]);
    let border = frame(1, 1, 0, (2, 1), vec![1], vec![true]);
    let p = palette();
    draw_frame(&mut s, &body, &p, Placement::default(), SHADOWED);
    draw_frame(&mut s, &body, &p, Placement::default(), SHADOWED);
    draw_frame(&mut s, &border, &p, Placement::default(), SHADOWED);
    let quarter = 7 << 11 | 15 << 5 | 7;
    // 7 -> 1 + 3, 15 -> 3 + 7.
    assert_eq!(s.pixels(), [quarter, 4 << 11 | 10 << 5 | 4]);
}

#[test]
fn a_frame_of_width_0_draws_nothing() {
    let mut s = screen(1, 1);
    let f = Frame {
        width: 0,
        height: 0,
        ..frame(1, 0, 0, (1, 1), vec![], vec![])
    };
    draw_frame(&mut s, &f, &palette(), Placement::default(), SHADOWED);
    assert_eq!(s.pixels(), [WHITE]);
}

#[test]
fn images_are_drawn_opaque_and_clipped() {
    let mut pal = Palette::default();
    pal.0[3] = Rgb {
        r: 0xFF,
        g: 0,
        b: 0,
    };
    let indexed = Pcx::Indexed(Box::new(IndexedImage {
        width: 2,
        height: 1,
        pixels: vec![3, 0],
        palette: pal,
    }));
    let mut s = screen(3, 1);
    draw_image(&mut s, &indexed, 1, 0);
    assert_eq!(s.pixels(), [WHITE, 0xF800, 0]);
    let rgb = Pcx::Rgb(RgbImage {
        width: 1,
        height: 2,
        pixels: vec![
            Rgb {
                r: 0,
                g: 0,
                b: 0xFF,
            },
            Rgb {
                r: 0,
                g: 0xFF,
                b: 0,
            },
        ],
    });
    draw_image(&mut s, &rgb, 0, -1);
    assert_eq!(s.pixels(), [0x07E0, 0xF800, 0]);
    // Keyed: an 8-bit image's index 0 leaves the screen as it was; a 24-bit one is opaque.
    let mut s = screen(3, 1);
    draw_image_keyed(&mut s, &indexed, 1, 0);
    assert_eq!(s.pixels(), [WHITE, 0xF800, WHITE]);
    draw_image_keyed(&mut s, &rgb, 2, -1);
    assert_eq!(s.pixels(), [WHITE, 0xF800, 0x07E0]);
    // Through another palette (E17 M3 T6a: a picture in a player's colours): its index 3 green,
    // index 0 still transparent; a 24-bit image as it is.
    let mut green = Palette::default();
    green.0[3] = Rgb {
        r: 0,
        g: 0xFF,
        b: 0,
    };
    let mut s = screen(3, 1);
    draw_image_keyed_in(&mut s, &indexed, &green, 1, 0);
    assert_eq!(s.pixels(), [WHITE, 0x07E0, WHITE]);
    draw_image_keyed_in(&mut s, &rgb, &green, 0, 0);
    assert_eq!(s.pixels(), [0x001F, 0x07E0, WHITE]);
}

#[test]
fn a_clipped_image_draws_only_inside_its_rectangle() {
    // E17 M4 T1b: a siege's wall strips and the moat over a stack.
    let indexed = Pcx::Indexed(Box::new(IndexedImage {
        width: 3,
        height: 2,
        pixels: vec![5, 0, 7, 8, 9, 10],
        palette: palette_of_greys(),
    }));
    let mut s = screen(4, 2);
    draw_image_keyed_clipped(&mut s, &indexed, 1, 0, [2, 1, 4, 2]);
    assert_eq!(row(&s, 0), [WHITE; 4]);
    assert_eq!(row(&s, 1), [WHITE, WHITE, grey(9), grey(10)]);
    // Index 0 stays transparent inside it; the screen still clips.
    let mut s = screen(4, 2);
    draw_image_keyed_clipped(&mut s, &indexed, 1, 0, [i32::MIN, 0, i32::MAX, 1]);
    assert_eq!(row(&s, 0), [WHITE, grey(5), WHITE, grey(7)]);
    assert_eq!(row(&s, 1), [WHITE; 4]);
}

#[test]
fn a_frame_cut_to_its_canvas_s_top_rows() {
    // E17 M4 T1b: a turret's shooter, its canvas's top 232 rows (0x495090).
    let f = frame(1, 0, 0, (1, 3), vec![10, 20, 30], vec![]);
    let at = Placement {
        x: 0,
        y: 0,
        mirrored: false,
    };
    let mut s = screen(1, 3);
    draw_frame_rows(&mut s, &f, &palette(), at, SHADOWED, 2);
    assert_eq!(s.pixels(), [grey(10), grey(20), WHITE]);
}

/// [`palette`]'s greys as an image's palette.
fn palette_of_greys() -> Palette {
    let mut p = Palette::default();
    for (i, c) in p.0.iter_mut().enumerate() {
        let v = u8::try_from(i).unwrap();
        *c = Rgb { r: v, g: v, b: v };
    }
    p
}

#[test]
fn the_translucent_drawer_blends_literal_pixels_and_skips_runs() {
    // E17 M3 T3b (0x47EF60): half the frame's colour plus half the screen's, per channel.
    let half = F.half_mask();
    for i in [0, 4, 200, 255] {
        let blended = ((grey(i) >> 1) & half) + ((WHITE >> 1) & half);
        assert_eq!(
            result(Drawer::Translucent, Encoding::Rows, i, false),
            blended
        );
        // Every run is skipped, the shadows' included.
        assert_eq!(result(Drawer::Translucent, Encoding::Rows, i, true), WHITE);
    }
    // White over white stays a step under white: each half loses its lowest bit.
    assert_eq!(
        result(Drawer::Translucent, Encoding::Rows, 255, false),
        0xF7DE
    );
    // The other encodings as the plain drawer without a flag colour.
    assert_eq!(result(Drawer::Translucent, Encoding::Raw, 5, true), grey(5));
    assert_eq!(
        result(Drawer::Translucent, Encoding::PackedRows, 5, true),
        WHITE
    );
    assert_eq!(
        result(Drawer::Translucent, Encoding::PackedBlocks, 5, true),
        WHITE
    );
    assert_eq!(
        result(Drawer::Translucent, Encoding::PackedBlocks, 9, false),
        grey(9)
    );
}
