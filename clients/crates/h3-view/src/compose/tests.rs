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

/// The compositor with the original's bugs fixed, as the client draws (no draw here differs).
fn compose(list: &[Draw], batch: &Batch, layout: &BattleLayout) -> Result<Screen, ComposeError> {
    super::compose(list, batch, layout, Bugs::Fixed)
}

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

#[test]
fn a_projectile_is_drawn_plain_and_mirrored_toward_the_left() {
    // T4b-1 (§7.9): group 0's frame, its run skipped, reflected across its canvas.
    let p = palette(&[(3, RED), (4, WHITE)]);
    let arrow = build::def(
        0x40,
        (2, 1),
        &p,
        &[(0, &[("f", 0)])],
        &[row(&[3, 4], &[false, false])],
    );
    let mut b = batch();
    b.add("Pr.def", &arrow).unwrap();
    let shot = |x, mirrored| Draw::Projectile {
        def: "PR.DEF".to_string(),
        frame: 0,
        x,
        y: 0,
        mirrored,
    };
    let background = Draw::Background {
        picture: "bg.pcx".to_string(),
        x: 0,
        y: 0,
    };
    let screen = compose(&[background, shot(0, false), shot(2, true)], &b, &small()).unwrap();
    let [bl, r, w] = [BLUE, RED, WHITE].map(|c| FORMAT.pack(c));
    assert_eq!(screen.pixels()[..6], [r, w, w, r, bl, bl]);
    // A frame the DEF lacks.
    let missing = compose(&[shot(0, false)], &batch(), &small()).unwrap_err();
    assert_eq!(missing, ComposeError::Missing("PR.DEF".into()));
}

#[test]
fn a_magic_animation_is_drawn_translucent_or_plain() {
    // E17 M3 T3b (§7.15): group 0's frame over the screen; translucent, its literal pixels half
    // and half with what is under them and its runs skipped; else plain, its runs skipped.
    let p = palette(&[(3, RED), (4, WHITE)]);
    let heal = build::def(
        0x40,
        (2, 1),
        &p,
        &[(0, &[("f", 0)])],
        &[row(&[3, 4], &[false, true])],
    );
    let mut b = batch();
    b.add("Heal.def", &heal).unwrap();
    let magic = |x, mirrored, translucent| Draw::Magic {
        def: "HEAL.DEF".to_string(),
        frame: 0,
        x,
        y: 0,
        mirrored,
        translucent,
    };
    let background = Draw::Background {
        picture: "bg.pcx".to_string(),
        x: 0,
        y: 0,
    };
    let list = [background, magic(0, false, true), magic(2, true, false)];
    let screen = compose(&list, &b, &small()).unwrap();
    let [bl, r] = [BLUE, RED].map(|c| FORMAT.pack(c));
    let half = FORMAT.half_mask();
    let blend = ((r >> 1) & half) + ((bl >> 1) & half);
    assert_eq!(screen.pixels()[..6], [blend, bl, bl, r, bl, bl]);
    // A DEF the batch lacks.
    let missing = compose(&[magic(0, false, true)], &batch(), &small()).unwrap_err();
    assert_eq!(missing, ComposeError::Missing("HEAL.DEF".into()));
}

#[test]
fn a_ray_is_written_into_the_screen_in_its_colours() {
    // M3 T1b-1 (§7.12): each pixel its entry's colour scaled into the masks; an entry beyond the
    // colours takes the last; pixels off the battlefield or the screen are left out.
    use crate::anim::Ray;
    use crate::ray::Stroke;
    let stroke = |x, lo, hi| Stroke {
        x,
        y: 0,
        lo,
        hi,
        column: true,
    };
    let ray = Draw::Ray(Ray {
        strokes: vec![stroke(1, -2, 2), stroke(4, 0, 1)],
        colours: vec![[152, 188, 24], [124, 216, 124]],
    });
    let background = Draw::Background {
        picture: "bg.pcx".to_string(),
        x: 0,
        y: 0,
    };
    let screen = compose(&[background, ray], &batch(), &small()).unwrap();
    let bl = FORMAT.pack(BLUE);
    // (1, 0) is entry 2 (the last colour), (1, 1) entry 1; (4, 0) entry 1, (4, 1) entry 0.
    assert_eq!(
        screen.pixels(),
        [
            bl, 0x7EAF, bl, bl, 0x7EAF, bl, bl, 0x7EAF, bl, bl, 0x95C2, bl
        ]
    );
    // Without colours: black (the executable's colour 0).
    let bare = Draw::Ray(Ray {
        strokes: vec![stroke(0, 0, 0)],
        colours: vec![],
    });
    let background = Draw::Background {
        picture: "bg.pcx".to_string(),
        x: 0,
        y: 0,
    };
    let screen = compose(&[background, bare], &batch(), &small()).unwrap();
    assert_eq!(screen.pixel(0, 0), Some(0));
    assert_eq!(screen.pixel(1, 0), Some(bl));
}

#[test]
fn a_hero_and_its_flag_take_the_shadowed_drawer_the_flag_its_players_colours() {
    // §4.5 (E17 M3 T3a): 0x4952B0 without an outline (a run of 5 skipped), the canvas at its
    // point, the right hero mirrored in it; a flag's entries 224-255 its player's 32 of the
    // players' palette, from entry 32 x the player's colour (0x5FFE20).
    let p = palette(&[(4, WHITE), (224, RED)]);
    let flag = build::def(
        0x40,
        (2, 1),
        &p,
        &[(0, &[("f", 0)])],
        &[row(&[224, 4], &[false, false])],
    );
    let hero = build::def(
        0x49,
        (2, 1),
        &p,
        &[(1, &[("f", 0)])],
        &[row(&[4, 5], &[false, true])],
    );
    let players = build::pal(&palette(&[(0, GREEN), (32, YELLOW)]), &[0; 256], 0, &[]);
    let mut b = batch();
    for (name, bytes) in [("Fl.def", flag), ("Ch.def", hero), ("Players.pal", players)] {
        b.add(name, &bytes).unwrap();
    }
    let flag = |player, x, frame| Draw::Flag {
        def: "FL.DEF".to_string(),
        frame,
        x,
        y: 0,
        palette: "PLAYERS.PAL".to_string(),
        player,
    };
    let hero = |x, mirrored| Draw::Hero {
        def: "CH.DEF".to_string(),
        group: 1,
        frame: 0,
        x,
        y: 1,
        mirrored,
    };
    let background = Draw::Background {
        picture: "bg.pcx".to_string(),
        x: 0,
        y: 0,
    };
    let list = [
        background.clone(),
        flag(0, 0, 0),
        flag(1, 2, 0),
        // A colour beyond the palette's eight players keeps the DEF's own.
        flag(9, 4, 0),
        hero(0, false),
        hero(2, true),
    ];
    let screen = compose(&list, &b, &small()).unwrap();
    let [bl, g, r, w, y] = [BLUE, GREEN, RED, WHITE, YELLOW].map(|c| FORMAT.pack(c));
    assert_eq!(screen.pixels(), [g, w, y, w, r, w, w, bl, bl, w, bl, bl]);
    // A players' palette or a frame the batch lacks.
    let no_palette = Draw::Flag {
        def: "FL.DEF".to_string(),
        frame: 0,
        x: 0,
        y: 0,
        palette: "none.pal".to_string(),
        player: 0,
    };
    let missing = |list: &[Draw]| compose(list, &b, &small()).unwrap_err();
    assert_eq!(
        missing(&[no_palette]),
        ComposeError::Missing("none.pal".into())
    );
    assert_eq!(
        missing(&[flag(0, 0, 1)]),
        ComposeError::Missing("FL.DEF group 0 frame 1".into())
    );
    let lost = Draw::Hero {
        def: "CH.DEF".to_string(),
        group: 1,
        frame: 3,
        x: 0,
        y: 1,
        mirrored: false,
    };
    assert_eq!(
        missing(&[lost]),
        ComposeError::Missing("CH.DEF group 1 frame 3".into())
    );
}

/// A count box's layout (M3 T4a): a 4 x 3 picture at (2, 1) of an 8 x 4 frame, its middle row's
/// inner two pixels tinted, the count in a 4 x 3 text box, centred.
fn boxed() -> (BattleLayout, Batch) {
    let mut layout = small();
    layout.frame = [8, 4];
    layout.count_box.picture = "Box.pcx".to_string();
    layout.count_box.font = "Digits.fnt".to_string();
    layout.count_box.font_palette = "text.pal".to_string();
    layout.count_box.tint_rect = [[1, 1], [2, 1]];
    layout.count_box.text_size = [4, 3];
    let mut b = batch();
    let pixels = [0, 1, 1, 1, 1, 2, 2, 1, 1, 1, 1, 1];
    let picture = build::pcx_indexed(4, 3, &pixels, &palette(&[(1, BLUE), (2, WHITE)]));
    b.add("Box.pcx", &picture).unwrap();
    // The text's colour (entry 13) and the shadow's (32).
    let text = build::pal(&palette(&[(13, WHITE), (32, GREEN)]), &[0; 256], 0, &[]);
    b.add("text.pal", &text).unwrap();
    // `1` inks its bottom row, `2` shades it; each one pixel wide.
    let mut glyphs = vec![h3_assets::fnt::Glyph::default(); 256];
    glyphs[usize::from(b'1')].width = 1;
    glyphs[usize::from(b'1')].pixels = vec![0, 0, 0xFF];
    glyphs[usize::from(b'2')].width = 1;
    glyphs[usize::from(b'2')].pixels = vec![0, 0, 1];
    let font = h3_assets::fnt::Font {
        height: 3,
        y_offset: 0,
        glyphs,
    };
    b.add("Digits.fnt", &build::fnt(&font)).unwrap();
    (layout, b)
}

#[test]
fn a_count_box_is_its_picture_tinted_with_the_count_over_it() {
    let (layout, batch) = boxed();
    let count = |tint| Draw::CountBox {
        x: 2,
        y: 1,
        count: 12,
        tint,
    };
    // The count box's own tint: white becomes 0x919F (h3_render::tint's tests).
    let screen = compose(&[count([0x3F40_0000, 0x3F4C_CCCD])], &batch, &layout).unwrap();
    let [b, w, g] = [BLUE, WHITE, GREEN].map(|c| FORMAT.pack(c));
    let tinted = 0x919F;
    assert_eq!(
        screen.pixels(),
        [
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, b, b, b, 0, 0],
            [0, 0, b, tinted, tinted, b, 0, 0],
            [0, 0, b, w, g, b, 0, 0],
        ]
        .concat()
    );
    // A hue of 1 leaves the picture as it is.
    let plain = compose(&[count([0x3F80_0000, 0])], &batch, &layout).unwrap();
    assert_eq!(plain.pixel(3, 2), Some(w));
    // What the batch lacks: the font, its palette, the picture.
    let mut no_font = layout.clone();
    no_font.count_box.font = "no.fnt".to_string();
    let gone = |what: &str| Err(ComposeError::Missing(what.to_string()));
    let list = [count([0, 0])];
    assert_eq!(compose(&list, &batch, &no_font), gone("no.fnt"));
    let mut no_palette = layout.clone();
    no_palette.count_box.font_palette = "no.pal".to_string();
    assert_eq!(compose(&list, &batch, &no_palette), gone("no.pal"));
    let mut no_picture = layout;
    no_picture.count_box.picture = "no.pcx".to_string();
    assert_eq!(compose(&list, &batch, &no_picture), gone("no.pcx"));
}
