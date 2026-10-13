use super::*;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use h3_assets::build;
use h3_assets::def::Encoding;
use h3_assets::palette::{Palette, Rgb};
use h3_render::screen::MAX_SIDE;

use crate::scene::tests::layout;

const BLUE: Rgb = Rgb { r: 0, g: 0, b: 255 };
const GREEN: Rgb = Rgb { r: 0, g: 255, b: 0 };
const RED: Rgb = Rgb { r: 255, g: 0, b: 0 };
const WHITE: Rgb = Rgb {
    r: 255,
    g: 255,
    b: 255,
};
const YELLOW: Rgb = Rgb {
    r: 255,
    g: 255,
    b: 0,
};

fn palette(entries: &[(usize, Rgb)]) -> Palette {
    let mut p = Palette::default();
    for &(i, c) in entries {
        p.0[i] = c;
    }
    p
}

/// A one-row frame of encoding 1: `pixels`, each a run where `runs` says so.
fn row(pixels: &[u8], runs: &[bool]) -> Frame {
    let width = u32::try_from(pixels.len()).unwrap();
    Frame {
        encoding: Encoding::Rows,
        full_width: width,
        full_height: 1,
        left: 0,
        top: 0,
        width,
        height: 1,
        pixels: pixels.to_vec(),
        runs: runs.to_vec(),
        short_header: false,
    }
}

/// A DEF two pixels wide whose group 2 holds `frame`.
fn def(frame: Frame) -> Vec<u8> {
    let p = palette(&[(3, RED), (4, WHITE)]);
    build::def(0x42, (2, 1), &p, &[(2, &[("f", 0)])], &[frame])
}

/// A 6 x 2 frame; a creature's canvas at its anchor minus (1, 0).
fn small() -> BattleLayout {
    BattleLayout {
        frame: [6, 2],
        stack_canvas: [1, 0],
        ..layout()
    }
}

fn batch() -> Batch {
    let mut b = Batch::default();
    let background = build::pcx_indexed(6, 2, &[1; 12], &palette(&[(1, BLUE)]));
    let absolute = build::pcx_indexed(2, 1, &[0, 2], &palette(&[(0, RED), (2, GREEN)]));
    let pal = build::pal(&palette(&[(0x60, YELLOW)]), &[0; 256], 0, &[]);
    for (name, bytes) in [
        ("Bg.pcx", background),
        ("Abs.pcx", absolute),
        ("game.pal", pal),
        // The obstacle's run (index 1, a shadow) is skipped.
        ("Ob.def", def(row(&[3, 1], &[false, true]))),
        // The creature's run of 5 takes the outline's colour, or is skipped without one.
        ("Cr.def", def(row(&[4, 5], &[false, true]))),
    ] {
        b.add(name, &bytes).unwrap();
    }
    b
}

fn creature(anchor: [i32; 2], mirrored: bool, outline: Option<u8>) -> Draw {
    creature_of("CR.DEF", anchor, mirrored, outline)
}

fn creature_of(def: &str, anchor: [i32; 2], mirrored: bool, outline: Option<u8>) -> Draw {
    Draw::Creature {
        def: def.to_string(),
        group: 2,
        frame: 0,
        anchor,
        mirrored,
        outline,
    }
}

#[test]
fn each_draw_takes_its_drawer() {
    let list = [
        Draw::Background {
            picture: "bg.pcx".to_string(),
            x: 0,
            y: 0,
        },
        Draw::AbsoluteObstacle {
            picture: "abs.pcx".to_string(),
            x: 0,
            y: 0,
        },
        Draw::Obstacle {
            def: "ob.def".to_string(),
            group: 2,
            frame: 0,
            x: 2,
            y: 0,
        },
        creature([1, 1], false, Some(0x60)),
        // Mirrored across the DEF's width: its left edge at 4 - 2 + 1.
        creature([4, 1], true, None),
    ];
    let screen = compose(&list, &batch(), &small()).unwrap();
    let [b, g, r, w, y] = [BLUE, GREEN, RED, WHITE, YELLOW].map(|c| FORMAT.pack(c));
    assert_eq!(screen.pixels(), [b, g, r, b, b, b, w, y, b, b, w, b]);
}

#[test]
fn a_stack_is_placed_by_facing_with_the_native_offsets() {
    // §4.2: the canvas at the anchor minus (196, 267); mirrored, its left edge at the anchor's x
    // minus 254 for a DEF 450 wide, so canvas column 0 lands at x + 195.
    let layout = BattleLayout {
        frame: [460, 8],
        stack_canvas: [196, 267],
        ..layout()
    };
    let one = Frame {
        encoding: Encoding::Rows,
        full_width: 450,
        full_height: 1,
        left: 0,
        top: 0,
        width: 1,
        height: 1,
        pixels: vec![4],
        runs: vec![false],
        short_header: false,
    };
    let p = palette(&[(4, WHITE)]);
    let mut batch = Batch::default();
    let wide = build::def(0x42, (450, 1), &p, &[(2, &[("f", 0)])], &[one]);
    batch.add("Wide.def", &wide).unwrap();
    let white = FORMAT.pack(WHITE);
    let drawn = |anchor: [i32; 2], mirrored: bool| {
        let list = [creature_of("WIDE.DEF", anchor, mirrored, None)];
        let screen = compose(&list, &batch, &layout).unwrap();
        let at: Vec<usize> = (0..screen.pixels().len())
            .filter(|&i| screen.pixels()[i] == white)
            .collect();
        at
    };
    assert_eq!(drawn([200, 270], false), [3 * 460 + 4]);
    assert_eq!(drawn([200, 270], true), [3 * 460 + 395]);
    // Far off the screen: the placement saturates, nothing is drawn, nothing overflows.
    assert!(drawn([i32::MIN, i32::MIN], true).is_empty());
}

#[test]
fn what_the_batch_lacks_is_an_error() {
    let batch = batch();
    let missing = |list: &[Draw]| compose(list, &batch, &small()).unwrap_err();
    let picture = |name: &str| Draw::Background {
        picture: name.to_string(),
        x: 0,
        y: 0,
    };
    let absolute = Draw::AbsoluteObstacle {
        picture: "no.pcx".to_string(),
        x: 0,
        y: 0,
    };
    let obstacle = |def: &str, frame| Draw::Obstacle {
        def: def.to_string(),
        group: 2,
        frame,
        x: 0,
        y: 0,
    };
    let gone = |what: &str| ComposeError::Missing(what.to_string());
    assert_eq!(missing(&[picture("no.pcx")]), gone("no.pcx"));
    assert_eq!(missing(&[absolute]), gone("no.pcx"));
    assert_eq!(missing(&[obstacle("no.def", 0)]), gone("no.def"));
    let lost = creature_of("no.def", [1, 1], false, None);
    assert_eq!(missing(&[lost]), gone("no.def"));
    assert_eq!(
        missing(&[obstacle("Ob.def", 1)]),
        gone("Ob.def group 2 frame 1")
    );
    let mut no_palette = small();
    no_palette.outline_palette = "other.pal".to_string();
    assert_eq!(
        compose(&[creature([1, 1], false, Some(1))], &batch, &no_palette),
        Err(gone("other.pal"))
    );
    let mut huge = small();
    huge.frame = [MAX_SIDE + 1, 1];
    let error = compose(&[], &batch, &huge).unwrap_err();
    assert_eq!(error, ComposeError::Frame([MAX_SIDE + 1, 1]));
    assert_eq!(
        ComposeError::Frame([3, 4]).to_string(),
        "a 3 x 4 frame is too large"
    );
    assert_eq!(gone("x").to_string(), "not in the batch: x");
}
