use super::*;
use alloc::vec;
use alloc::vec::Vec;

use h3_assets::def::Encoding;
use h3_assets::palette::{Palette, Rgb};
use h3_render::screen::PixelFormat;

const FORMAT: PixelFormat = PixelFormat::RGB565;

/// A palette whose entry i is the 565 pixel i (red 0, green i >> 5, blue i & 31).
fn palette() -> ScreenPalette {
    let mut p = Palette([Rgb::default(); 256]);
    for (i, c) in p.0.iter_mut().enumerate() {
        let i = u8::try_from(i).unwrap();
        *c = Rgb {
            r: 0,
            g: (i >> 5) << 2,
            b: (i & 31) << 3,
        };
    }
    ScreenPalette::new(&p, FORMAT)
}

/// A 3 × 2 frame on a 3 × 2 canvas: `pixels` row-major, `runs` marked.
fn frame(pixels: [u8; 6], runs: [bool; 6]) -> Frame {
    Frame {
        encoding: Encoding::PackedRows,
        full_width: 3,
        full_height: 2,
        left: 0,
        top: 0,
        width: 3,
        height: 2,
        pixels: pixels.to_vec(),
        runs: runs.to_vec(),
        short_header: false,
    }
}

fn screen(fill: u16) -> Screen {
    let mut s = Screen::new(5, 4, FORMAT).unwrap();
    s.fill(fill);
    s
}

/// The screen's rows as lists of pixels.
fn rows(s: &Screen) -> Vec<Vec<u16>> {
    s.pixels().chunks(5).map(<[u16]>::to_vec).collect()
}

const ALL: [i32; 4] = [0, 0, 5, 4];

#[test]
fn raw_draws_literal_pixels_and_skips_runs() {
    let f = frame([1, 2, 3, 4, 5, 6], [false, true, false, false, false, true]);
    let mut s = screen(0xFFFF);
    let at = Place {
        x: 1,
        y: 1,
        ..Place::default()
    };
    raw(&mut s, &f, &palette(), at, ALL);
    let w = 0xFFFF;
    assert_eq!(
        rows(&s),
        [
            vec![w, w, w, w, w],
            vec![w, 1, w, 3, w],
            vec![w, 4, 5, w, w],
            vec![w, w, w, w, w],
        ]
    );
}

#[test]
fn raw_mirrors_both_ways_across_the_canvas() {
    let f = frame([1, 2, 3, 4, 5, 6], [false; 6]);
    let draw = |mirror_x, mirror_y| {
        let mut s = screen(0);
        let at = Place {
            x: 0,
            y: 0,
            mirror_x,
            mirror_y,
        };
        raw(&mut s, &f, &palette(), at, ALL);
        rows(&s)
    };
    assert_eq!(draw(false, false)[..2], [[1, 2, 3, 0, 0], [4, 5, 6, 0, 0]]);
    assert_eq!(draw(true, false)[..2], [[3, 2, 1, 0, 0], [6, 5, 4, 0, 0]]);
    assert_eq!(draw(false, true)[..2], [[4, 5, 6, 0, 0], [1, 2, 3, 0, 0]]);
    assert_eq!(draw(true, true)[..2], [[6, 5, 4, 0, 0], [3, 2, 1, 0, 0]]);
}

#[test]
fn raw_clips_to_the_rectangle_and_the_canvas() {
    // A 3 × 2 frame at (1, 0) of a 3 × 2 canvas: its last column lies past the canvas.
    let mut f = frame([1, 2, 3, 4, 5, 6], [false; 6]);
    f.left = 1;
    f.full_width = 3;
    let mut s = screen(0);
    raw(&mut s, &f, &palette(), Place::default(), ALL);
    assert_eq!(rows(&s)[..2], [[0, 1, 2, 0, 0], [0, 4, 5, 0, 0]]);
    // The clip (1, 1)–(3, 2), the frame mirrored: its pixels at x 1, 2 of row 1 only.
    let f = frame([1, 2, 3, 4, 5, 6], [false; 6]);
    let mut s = screen(0);
    let at = Place {
        x: 0,
        y: 0,
        mirror_x: true,
        mirror_y: false,
    };
    raw(&mut s, &f, &palette(), at, [1, 1, 3, 2]);
    assert_eq!(rows(&s)[..2], [[0, 0, 0, 0, 0], [0, 5, 4, 0, 0]]);
    // Off the screen: nothing, and no panic.
    let mut s = screen(0);
    let at = Place {
        x: -10,
        y: 10,
        ..Place::default()
    };
    raw(&mut s, &f, &palette(), at, [-20, -20, 20, 20]);
    assert!(s.pixels().iter().all(|&p| p == 0));
}

#[test]
fn a_frame_across_the_screens_sides_keeps_what_lies_on_it() {
    // The clip wider than the screen: the screen cuts. Row 0 above the screen, column 2 past its
    // right side.
    let f = frame([1, 2, 3, 4, 5, 6], [false; 6]);
    let wide = [-20, -20, 20, 20];
    let mut s = screen(0);
    let at = Place {
        x: 3,
        y: -1,
        ..Place::default()
    };
    raw(&mut s, &f, &palette(), at, wide);
    assert_eq!(rows(&s), [[0, 0, 0, 4, 5], [0; 5], [0; 5], [0; 5]]);
    // Mirrored at x −1: column 2 left of the screen; row 1 below it.
    let mut s = screen(0);
    let at = Place {
        x: -1,
        y: 3,
        mirror_x: true,
        mirror_y: false,
    };
    raw(&mut s, &f, &palette(), at, wide);
    assert_eq!(rows(&s), [[0; 5], [0; 5], [0; 5], [2, 1, 0, 0, 0]]);
    // Its rows on the screen, its columns all past its right side: nothing.
    let mut s = screen(0);
    let at = Place {
        x: 5,
        ..Place::default()
    };
    raw(&mut s, &f, &palette(), at, wide);
    assert!(s.pixels().iter().all(|&p| p == 0));
}

#[test]
fn pixels_past_the_frames_runs_are_literal() {
    // Runs for the first four pixels only: the last two literal; none at all (encoding 0).
    let mut f = frame([5, 2, 5, 7, 5, 6], [false; 6]);
    f.runs = vec![true, false, true, true];
    let flag = Some(0x1234);
    let draw = |f: &Frame, mirror_x| {
        let mut s = screen(0);
        let at = Place {
            mirror_x,
            ..Place::default()
        };
        flagged(&mut s, f, &palette(), at, ALL, flag);
        rows(&s)
    };
    assert_eq!(
        draw(&f, false)[..2],
        [[0x1234, 2, 0x1234, 0, 0], [0, 5, 6, 0, 0]]
    );
    assert_eq!(
        draw(&f, true)[..2],
        [[0x1234, 2, 0x1234, 0, 0], [6, 5, 0, 0, 0]]
    );
    f.runs.clear();
    assert_eq!(draw(&f, false)[..2], [[5, 2, 5, 0, 0], [7, 5, 6, 0, 0]]);
    assert_eq!(draw(&f, true)[..2], [[5, 2, 5, 0, 0], [6, 5, 7, 0, 0]]);
}

#[test]
fn shadows_darken_under_runs_of_one_to_four() {
    let white = FORMAT.pack(Rgb {
        r: 255,
        g: 255,
        b: 255,
    });
    // Runs of 0 to 4 and a literal 1: borders under 1 and 2, bodies under 3 and 4.
    let f = frame([0, 1, 2, 3, 4, 1], [true, true, true, true, true, false]);
    let mut s = screen(white);
    shadows(&mut s, &f, Place::default(), ALL);
    let (border, body) = (FORMAT.shadow_border(white), FORMAT.shadow_body(white));
    assert_eq!(
        rows(&s)[..2],
        [
            [white, border, border, white, white],
            [body, body, white, white, white]
        ]
    );
    // Clipped, mirrored top to bottom: the body of row 1 lands on row 0, only x 0–1.
    let mut s = screen(white);
    let at = Place {
        mirror_y: true,
        ..Place::default()
    };
    shadows(&mut s, &f, at, [0, 0, 2, 1]);
    assert_eq!(rows(&s)[0], [body, body, white, white, white]);
    assert_eq!(rows(&s)[1], [white; 5]);
}

#[test]
fn flagged_draws_literal_pixels_and_flag_runs_in_the_colour() {
    // Literal 7, runs of 0, 1, 4 and 5, literal 5.
    let f = frame([7, 0, 1, 4, 5, 5], [false, true, true, true, true, false]);
    let w = 0xFFFF;
    let mut s = screen(w);
    flagged(&mut s, &f, &palette(), Place::default(), ALL, Some(0xF800));
    assert_eq!(rows(&s)[..2], [[7, w, w, w, w], [w, 0xF800, 5, w, w]]);
    // Without a colour the flag's run is skipped too; clipped to the first column.
    let mut s = screen(w);
    flagged(&mut s, &f, &palette(), Place::default(), [0, 0, 1, 4], None);
    assert_eq!(rows(&s)[..2], [[7, w, w, w, w], [w, w, w, w, w]]);
}

#[test]
fn the_shadow_pass_darkens_under_runs_of_one_and_four_only() {
    let white = FORMAT.pack(Rgb {
        r: 255,
        g: 255,
        b: 255,
    });
    // Runs of 1, 2, 3, 4 and 5, and a literal 1.
    let f = frame([1, 2, 3, 4, 5, 1], [true, true, true, true, true, false]);
    let mut s = screen(white);
    shadow_pass(&mut s, &f, Place::default(), ALL);
    let (border, body) = (FORMAT.shadow_border(white), FORMAT.shadow_body(white));
    assert_eq!(
        rows(&s)[..2],
        [
            [border, white, white, white, white],
            [body, white, white, white, white]
        ]
    );
}

#[test]
fn blend_halves_literal_pixels_with_the_screen_and_skips_runs() {
    // Pixel 0x001F (blue 31) over white 0xFFFF: each channel's halves added under the half mask
    // 0x7BEF; a run (here of 5, a flag's index) leaves the screen.
    let f = frame(
        [31, 5, 0, 31, 31, 31],
        [false, true, false, false, false, false],
    );
    let mut s = screen(0xFFFF);
    blend(&mut s, &f, &palette(), Place::default(), [0, 0, 3, 1]);
    let half = FORMAT.half_mask();
    let mixed = ((31 >> 1) & half) + ((0xFFFF >> 1) & half);
    let black = (0xFFFF >> 1) & half;
    let w = 0xFFFF;
    assert_eq!(
        rows(&s),
        [
            vec![mixed, w, black, w, w],
            vec![w, w, w, w, w],
            vec![w, w, w, w, w],
            vec![w, w, w, w, w],
        ]
    );
    // Over the same picture drawn plain: each channel's lowest bit lost.
    let mut s = screen(0);
    raw(&mut s, &f, &palette(), Place::default(), ALL);
    blend(&mut s, &f, &palette(), Place::default(), ALL);
    assert_eq!(s.pixel(0, 1), Some(30));
}
