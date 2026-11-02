use super::*;
use alloc::collections::BTreeMap;
use alloc::string::ToString;
use alloc::vec;
use h3_assets::build;
use h3_assets::def::{Def, Encoding, FrameRef, Group};
use h3_assets::fnt::Glyph;
use h3_assets::pcx::decode_pcx;
use h3_render::screen::PixelFormat;

use crate::ui::{Label, Text};

const FORMAT: PixelFormat = PixelFormat::RGB565;
const BLUE: Rgb = Rgb { r: 0, g: 0, b: 255 };
const GREEN: Rgb = Rgb { r: 0, g: 255, b: 0 };
const RED: Rgb = Rgb { r: 255, g: 0, b: 0 };
const WHITE: Rgb = Rgb {
    r: 255,
    g: 255,
    b: 255,
};

#[derive(Default)]
struct Store {
    sprites: BTreeMap<String, Sprite>,
    pictures: BTreeMap<String, Pcx>,
    palettes: BTreeMap<String, Palette>,
    fonts: BTreeMap<String, Font>,
}

impl Assets for Store {
    fn sprite(&self, name: &str) -> Option<&Sprite> {
        self.sprites.get(name)
    }
    fn picture(&self, name: &str) -> Option<&Pcx> {
        self.pictures.get(name)
    }
    fn palette(&self, name: &str) -> Option<&Palette> {
        self.palettes.get(name)
    }
    fn font(&self, name: &str) -> Option<&Font> {
        self.fonts.get(name)
    }
}

fn palette(entries: &[(usize, Rgb)]) -> Palette {
    let mut p = Palette::default();
    for &(i, c) in entries {
        p.0[i] = c;
    }
    p
}

/// A DEF of group 0 with two 2 × 1 frames: [3, 224] and [224, 0], the second's 0 a run.
fn sprite() -> Sprite {
    let frame = |pixels: Vec<u8>, runs: Vec<bool>| Frame {
        encoding: Encoding::Rows,
        full_width: 2,
        full_height: 1,
        left: 0,
        top: 0,
        width: 2,
        height: 1,
        pixels,
        runs,
        short_header: false,
    };
    Sprite {
        def: Def {
            kind: 0x47,
            width: 2,
            height: 1,
            palette: palette(&[(3, RED), (224, WHITE)]),
            groups: vec![Group {
                id: 0,
                frames: vec![
                    FrameRef {
                        name: "a".into(),
                        offset: 1,
                    },
                    FrameRef {
                        name: "b".into(),
                        offset: 2,
                    },
                ],
            }],
        },
        frames: BTreeMap::from([
            (1, frame(vec![3, 224], vec![false, false])),
            (2, frame(vec![224, 0], vec![false, true])),
        ]),
    }
}

/// Glyph `1` inks its one pixel (a 1 × 1 font).
fn font() -> Font {
    let mut glyphs = vec![Glyph::default(); 256];
    glyphs[usize::from(b'1')].width = 1;
    glyphs[usize::from(b'1')].pixels = vec![0xFF];
    Font {
        height: 1,
        y_offset: 0,
        glyphs,
    }
}

fn store() -> Store {
    let mut s = Store::default();
    s.sprites.insert("b.def".into(), sprite());
    // A DEF without a group 0.
    let mut other = sprite();
    other.def.groups[0].id = 1;
    s.sprites.insert("g1.def".into(), other);
    let picture = |pixels: &[u8]| {
        decode_pcx(&build::pcx_indexed(
            2,
            1,
            pixels,
            &palette(&[(1, BLUE), (224, RED)]),
        ))
        .unwrap()
    };
    s.pictures.insert("p.pcx".into(), picture(&[0, 224]));
    s.pictures.insert("q.pcx".into(), picture(&[1, 1]));
    s.pictures.insert(
        "rgb.pcx".into(),
        decode_pcx(&build::pcx_rgb(1, 1, &[GREEN])).unwrap(),
    );
    // The players' palette: player 1's entries 32 to 63, its first green.
    s.palettes
        .insert("players.pal".into(), palette(&[(32, GREEN)]));
    // TextDraw's colour 0 draws entry 9 of `game.pal`, 2 entry 11 (h3_render::text).
    s.palettes
        .insert("game.pal".into(), palette(&[(9, WHITE), (11, BLUE)]));
    s.fonts.insert("f.fnt".into(), font());
    s
}

fn screen(width: u32) -> Screen {
    Screen::new(width, 1, FORMAT).unwrap()
}

fn green_of_1() -> Option<PlayerColours> {
    Some(PlayerColours {
        palette: "players.pal".into(),
        player: 1,
    })
}

fn text_item(picture: &str, font: &str, font_palette: &str) -> Text {
    Text {
        id: 1,
        picture: picture.into(),
        at: [0, 0],
        size: [2, 1],
        font: font.into(),
        font_palette: font_palette.into(),
        colour: 0,
        align: 0,
        hint: None,
    }
}

fn label_item(font: &str, font_palette: &str) -> Label {
    Label {
        id: 1,
        at: [0, 0],
        size: [2, 1],
        font: font.into(),
        font_palette: font_palette.into(),
        colour: 0,
        align: 0,
        hint: None,
        repeat: None,
    }
}

#[test]
fn names_list_what_each_draw_needs() {
    let picture = |player| Draw::Picture {
        picture: "p.pcx".into(),
        x: 0,
        y: 0,
        player,
        size: None,
    };
    assert_eq!(names(&picture(None)), ["p.pcx"]);
    assert_eq!(names(&picture(green_of_1())), ["p.pcx", "players.pal"]);
    let button = |player| Draw::Button {
        def: "b.def".into(),
        frame: 0,
        x: 0,
        y: 0,
        mirrored: false,
        player,
    };
    assert_eq!(names(&button(None)), ["b.def"]);
    assert_eq!(names(&button(green_of_1())), ["b.def", "players.pal"]);
    let text = Draw::Text {
        item: text_item("q.pcx", "f.fnt", "game.pal"),
        x: 0,
        y: 0,
        text: String::new(),
    };
    assert_eq!(names(&text), ["q.pcx", "f.fnt", "game.pal"]);
    let label = Draw::Label {
        item: label_item("f.fnt", "game.pal"),
        x: 0,
        y: 0,
        text: String::new(),
        disabled: false,
    };
    assert_eq!(names(&label), ["f.fnt", "game.pal"]);
    assert!(names(&Draw::Darken { rect: [0; 4] }).is_empty());
}

#[test]
fn a_picture_is_keyed_clipped_and_in_a_players_colours() {
    let s = store();
    let picture = |name: &str, x, player, size| Draw::Picture {
        picture: name.into(),
        x,
        y: 0,
        player,
        size,
    };
    let [blue, red, green] = [BLUE, RED, GREEN].map(|c| FORMAT.pack(c));
    let mut out = screen(3);
    out.fill(blue);
    // Index 0 transparent: the blue under it stays.
    draw(&mut out, &picture("p.pcx", 1, None, None), &s, Bugs::Fixed).unwrap();
    assert_eq!(out.pixels(), [blue, blue, red]);
    // The player's colours over 224-255; at most the item's size of it.
    draw(
        &mut out,
        &picture("p.pcx", 0, green_of_1(), None),
        &s,
        Bugs::Fixed,
    )
    .unwrap();
    assert_eq!(out.pixels(), [blue, green, red]);
    let mut cut = screen(3);
    draw(
        &mut cut,
        &picture("q.pcx", 0, None, Some([1, 1])),
        &s,
        Bugs::Fixed,
    )
    .unwrap();
    assert_eq!(cut.pixels(), [blue, 0, 0]);
    // A picture of its own colours takes none.
    let mut rgb = screen(1);
    draw(
        &mut rgb,
        &picture("rgb.pcx", 0, green_of_1(), None),
        &s,
        Bugs::Fixed,
    )
    .unwrap();
    assert_eq!(rgb.pixels(), [green]);
    // What the assets lack.
    let missing = draw(&mut out, &picture("no.pcx", 0, None, None), &s, Bugs::Fixed);
    assert_eq!(missing, Err("no.pcx".to_string()));
    let no_players = Draw::Picture {
        picture: "p.pcx".into(),
        x: 0,
        y: 0,
        player: Some(PlayerColours {
            palette: "no.pal".into(),
            player: 1,
        }),
        size: None,
    };
    assert_eq!(
        draw(&mut out, &no_players, &s, Bugs::Fixed),
        Err("no.pal".to_string())
    );
}

#[test]
fn a_button_draws_its_frame_or_frame_0_plain_in_a_players_colours() {
    let s = store();
    let button = |def: &str, frame, mirrored, player| Draw::Button {
        def: def.into(),
        frame,
        x: 0,
        y: 0,
        mirrored,
        player,
    };
    let [red, white, green] = [RED, WHITE, GREEN].map(|c| FORMAT.pack(c));
    let mut out = screen(2);
    draw(&mut out, &button("b.def", 0, false, None), &s, Bugs::Fixed).unwrap();
    assert_eq!(out.pixels(), [red, white]);
    // Frame 1's run of 0 is skipped: the red stays; mirrored.
    draw(&mut out, &button("b.def", 1, true, None), &s, Bugs::Fixed).unwrap();
    assert_eq!(out.pixels(), [red, white]);
    let mut fresh = screen(2);
    draw(
        &mut fresh,
        &button("b.def", 1, false, None),
        &s,
        Bugs::Fixed,
    )
    .unwrap();
    assert_eq!(fresh.pixels(), [white, 0]);
    // Past the group's frames: frame 0; in player 1's colours (224 its green).
    draw(
        &mut out,
        &button("b.def", 9, false, green_of_1()),
        &s,
        Bugs::Fixed,
    )
    .unwrap();
    assert_eq!(out.pixels(), [red, green]);
    // What the assets lack: the DEF, group 0, the players' palette.
    let gone = |def: &str, player| {
        draw(
            &mut screen(2),
            &button(def, 0, false, player),
            &s,
            Bugs::Fixed,
        )
    };
    assert_eq!(gone("no.def", None), Err("no.def".to_string()));
    assert_eq!(
        gone("g1.def", None),
        Err("g1.def group 0 frame 0".to_string())
    );
    let no_players = Some(PlayerColours {
        palette: "no.pal".into(),
        player: 0,
    });
    assert_eq!(gone("b.def", no_players), Err("no.pal".to_string()));
}

#[test]
fn darken_halves_a_rectangle() {
    let mut out = screen(2);
    out.fill(FORMAT.pack(WHITE));
    draw(
        &mut out,
        &Draw::Darken { rect: [1, 0, 1, 1] },
        &store(),
        Bugs::Fixed,
    )
    .unwrap();
    assert_eq!(out.pixel(0, 0), Some(FORMAT.pack(WHITE)));
    assert_ne!(out.pixel(1, 0), Some(FORMAT.pack(WHITE)));
}

#[test]
fn reach_is_the_rectangle_each_drawer_can_change() {
    // E24 M3 T4c: a picture's at most its item's size of it, a button's frame's canvas (frame 0
    // past the group's), a darkened rectangle; T5a: a text's and a label's box widened by the
    // font's overhangs; unknown for a missing asset.
    let s = store();
    let picture = |name: &str, size| Draw::Picture {
        picture: name.into(),
        x: 1,
        y: 2,
        player: None,
        size,
    };
    assert_eq!(reach(&picture("p.pcx", None), &s), Some([1, 2, 3, 3]));
    assert_eq!(
        reach(&picture("p.pcx", Some([1, 5])), &s),
        Some([1, 2, 2, 3])
    );
    assert_eq!(reach(&picture("no.pcx", None), &s), None);
    let button = |def: &str, frame| Draw::Button {
        def: def.into(),
        frame,
        x: -1,
        y: 4,
        mirrored: true,
        player: None,
    };
    assert_eq!(reach(&button("b.def", 1), &s), Some([-1, 4, 1, 5]));
    assert_eq!(reach(&button("b.def", 9), &s), Some([-1, 4, 1, 5]));
    assert_eq!(reach(&button("g1.def", 0), &s), None);
    assert_eq!(reach(&button("no.def", 0), &s), None);
    let dark = Draw::Darken { rect: [3, 1, 2, 4] };
    assert_eq!(reach(&dark, &s), Some([3, 1, 5, 5]));
    let text = |picture: &str, font: &str, palette: &str| Draw::Text {
        item: text_item(picture, font, palette),
        x: 3,
        y: 1,
        text: "1".into(),
    };
    assert_eq!(
        reach(&text("q.pcx", "f.fnt", "game.pal"), &s),
        Some([3, 1, 5, 2])
    );
    assert_eq!(reach(&text("no.pcx", "f.fnt", "game.pal"), &s), None);
    assert_eq!(reach(&text("q.pcx", "no.fnt", "game.pal"), &s), None);
    assert_eq!(reach(&text("q.pcx", "f.fnt", "no.pal"), &s), None);
    let label = |font: &str, palette: &str| Draw::Label {
        item: label_item(font, palette),
        x: -2,
        y: 4,
        text: "1".into(),
        disabled: false,
    };
    assert_eq!(reach(&label("f.fnt", "game.pal"), &s), Some([-2, 4, 0, 5]));
    assert_eq!(reach(&label("no.fnt", "game.pal"), &s), None);
    assert_eq!(reach(&label("f.fnt", "no.pal"), &s), None);
}

#[test]
fn every_pixel_a_drawer_changes_lies_inside_its_reach() {
    // T7b2 (F6-3): the partial redraw is exact only if `draw` changes nothing outside `reach`
    // (design §4.6): each kind drawn on a screen of a colour no draw uses, every changed pixel
    // inside its rectangle. The texts take glyphs with negative spaces first, inside and last,
    // lines wider than their box, every alignment and both profiles; a mirrored button's frame
    // fills only part of its canvas.
    let mut s = store();
    let mut f = font();
    f.height = 2;
    let mut glyph = |c: u8, left, width, right| {
        let g = &mut f.glyphs[usize::from(c)];
        (g.left, g.width, g.right) = (left, width, right);
        g.pixels = vec![0xFF; usize::try_from(width * 2).unwrap()];
    };
    glyph(b'a', -2, 2, 0);
    glyph(b'b', 0, 3, -2);
    glyph(b'c', 1, 1, 1);
    glyph(b'd', -1, 2, -1);
    glyph(b' ', 0, 0, 2);
    s.fonts.insert("o.fnt".into(), f);
    let mut part = sprite();
    let frame = part.frames.get_mut(&1).unwrap();
    (frame.full_width, frame.left) = (5, 1);
    s.sprites.insert("m.def".into(), part);
    s.sprites.insert("c.def".into(), creature());
    let sentinel = FORMAT.pack(Rgb {
        r: 200,
        g: 100,
        b: 48,
    });
    let check = |d: &Draw| -> usize {
        let mut out = Screen::new(40, 12, FORMAT).unwrap();
        out.fill(sentinel);
        let [l, t, r, b] = reach(d, &s).unwrap();
        let mut changed = 0;
        for bugs in [Bugs::Original, Bugs::Fixed] {
            draw(&mut out, d, &s, bugs).unwrap();
            for y in 0..12 {
                for x in 0..40 {
                    if out.pixel(x, y) != Some(sentinel) {
                        changed += 1;
                        let inside = (l..r).contains(&(x as i32)) && (t..b).contains(&(y as i32));
                        assert!(
                            inside,
                            "{d:?} changed ({x}, {y}) outside {:?}",
                            [l, t, r, b]
                        );
                    }
                }
            }
        }
        changed
    };
    for text in [
        "abc",
        "cab",
        "bcd",
        "dddd",
        "ab ab ab",
        "cccccccccc",
        "a",
        "b",
    ] {
        for align in [0, 1, 2, 3, 1 | 4, 2 | 8] {
            let mut item = label_item("o.fnt", "game.pal");
            (item.size, item.align) = ([6, 4], align);
            let label = Draw::Label {
                item: item.clone(),
                x: 10,
                y: 3,
                text: text.into(),
                disabled: false,
            };
            assert!(check(&label) > 0, "{text} {align}");
            let mut item = text_item("q.pcx", "o.fnt", "game.pal");
            (item.size, item.align) = ([6, 4], align);
            let text = Draw::Text {
                item,
                x: 10,
                y: 3,
                text: text.into(),
            };
            assert!(check(&text) > 0);
        }
    }
    for mirrored in [false, true] {
        let button = Draw::Button {
            def: "m.def".into(),
            frame: 0,
            x: 7,
            y: 2,
            mirrored,
            player: None,
        };
        assert!(check(&button) > 0);
    }
    for size in [None, Some([1, 1])] {
        let picture = Draw::Picture {
            picture: "q.pcx".into(),
            x: 3,
            y: 5,
            player: None,
            size,
        };
        assert!(check(&picture) > 0);
    }
    assert!(
        check(&Draw::Darken {
            rect: [38, 10, 5, 5]
        }) > 0
    );
    // E17 M6 T2a1: a creature cut to boxes narrower and wider than its frame.
    for size in [[1, 2], [6, 2], [20, 3]] {
        let creature = Draw::Creature {
            def: "c.def".into(),
            group: 2,
            frame: 0,
            x: 4,
            y: 6,
            size,
        };
        assert!(check(&creature) > 0, "{size:?}");
    }
}

#[test]
fn a_texts_reach_takes_the_fonts_widest_overhangs() {
    // E24 M3 T5a: a negative left space may put a glyph's pixels left of the box, a negative
    // right one past its right edge; positive spaces widen nothing; the rows stay the box's.
    let mut f = font();
    assert_eq!(text_reach(&f, 5, 6, [10, 3]), [5, 6, 15, 9]);
    f.glyphs[usize::from(b'a')].left = -1;
    f.glyphs[usize::from(b'b')].left = -2;
    f.glyphs[usize::from(b'b')].right = 4;
    f.glyphs[usize::from(b'c')].right = -3;
    assert_eq!(text_reach(&f, 5, 6, [10, 3]), [3, 6, 18, 9]);
    assert_eq!(
        text_reach(&f, i32::MIN + 1, i32::MAX, [1, 1]),
        [i32::MIN, i32::MAX, i32::MIN + 5, i32::MAX]
    );
    assert_eq!(
        text_reach(&f, i32::MAX - 1, 0, [5, 1]),
        [i32::MAX - 3, 0, i32::MAX, 1]
    );
    let empty = Font {
        height: 1,
        y_offset: 0,
        glyphs: Vec::new(),
    };
    assert_eq!(text_reach(&empty, 0, 0, [2, 2]), [0, 0, 2, 2]);
}

#[test]
fn a_text_is_its_picture_opaque_then_the_text() {
    let s = store();
    let text = |item, text: &str| Draw::Text {
        item,
        x: 0,
        y: 0,
        text: text.into(),
    };
    let [blue, white] = [BLUE, WHITE].map(|c| FORMAT.pack(c));
    let mut out = screen(2);
    draw(
        &mut out,
        &text(text_item("q.pcx", "f.fnt", "game.pal"), "1"),
        &s,
        Bugs::Fixed,
    )
    .unwrap();
    assert_eq!(out.pixels(), [white, blue]);
    let failing = |item| draw(&mut screen(2), &text(item, ""), &s, Bugs::Fixed);
    assert_eq!(
        failing(text_item("no.pcx", "f.fnt", "game.pal")),
        Err("no.pcx".to_string())
    );
    assert_eq!(
        failing(text_item("q.pcx", "no.fnt", "game.pal")),
        Err("no.fnt".to_string())
    );
    assert_eq!(
        failing(text_item("q.pcx", "f.fnt", "no.pal")),
        Err("no.pal".to_string())
    );
}

#[test]
fn a_crafted_box_or_colour_at_the_integers_end_draws_without_overflowing() {
    // T7b2 (F1-1): the release build checks overflow; a layout's text box or label colour at
    // the end of its type is drawn as far as the screen goes, never a panic.
    let s = store();
    let mut item = text_item("q.pcx", "f.fnt", "game.pal");
    item.size = [i32::MAX, i32::MAX];
    let text = Draw::Text {
        item,
        x: 1,
        y: 1,
        text: "1".into(),
    };
    let _ = draw(&mut screen(2), &text, &s, Bugs::Fixed);
    let mut item = label_item("f.fnt", "game.pal");
    item.colour = u32::MAX;
    let label = Draw::Label {
        item,
        x: 0,
        y: 0,
        text: "1".into(),
        disabled: true,
    };
    let _ = draw(&mut screen(2), &label, &s, Bugs::Fixed);
}

#[test]
fn a_label_takes_its_colour_two_more_when_disabled() {
    let s = store();
    let label = |disabled| Draw::Label {
        item: label_item("f.fnt", "game.pal"),
        x: 0,
        y: 0,
        text: "1".into(),
        disabled,
    };
    let mut out = screen(2);
    draw(&mut out, &label(false), &s, Bugs::Fixed).unwrap();
    assert_eq!(out.pixels(), [FORMAT.pack(WHITE), 0]);
    draw(&mut out, &label(true), &s, Bugs::Fixed).unwrap();
    assert_eq!(out.pixels(), [FORMAT.pack(BLUE), 0]);
    let no_font = Draw::Label {
        item: label_item("no.fnt", "game.pal"),
        x: 0,
        y: 0,
        text: "1".into(),
        disabled: false,
    };
    assert_eq!(
        draw(&mut out, &no_font, &s, Bugs::Fixed),
        Err("no.fnt".to_string())
    );
}

/// A creature DEF: group 2's frame 0, 2 × 1 at column 3 of a 9 × 280 canvas, row 274 (one row
/// above the row the item's bottom takes), [3, 224]; group 1's frame 0 the same at row 270.
fn creature() -> Sprite {
    let frame = |top: i32| Frame {
        encoding: Encoding::Rows,
        full_width: 9,
        full_height: 280,
        left: 3,
        top,
        width: 2,
        height: 1,
        pixels: vec![3, 224],
        runs: vec![false, false],
        short_header: false,
    };
    let group = |id: u32, offset: u32| Group {
        id,
        frames: vec![FrameRef {
            name: "f".into(),
            offset,
        }],
    };
    Sprite {
        def: Def {
            kind: 0x42,
            width: 9,
            height: 280,
            palette: palette(&[(3, RED), (224, WHITE)]),
            groups: vec![group(1, 2), group(2, 1)],
        },
        frames: BTreeMap::from([(1, frame(274)), (2, frame(270))]),
    }
}

#[test]
fn a_creature_is_centred_by_group_2_and_cut_to_its_box() {
    // E17 M6 T2a1 (0x4EAB40 with the flags 0x12): the canvas at w / 2 − fw / 2 − m across
    // (6 / 2 − 2 / 2 − 3 = −1) and h − 275 down; its frame drawn by the shadowed drawer, cut to
    // the box.
    let mut s = store();
    s.sprites.insert("c.def".into(), creature());
    let draw_at = |group: u32, size: [i32; 2]| Draw::Creature {
        def: "c.def".into(),
        group,
        frame: 0,
        x: 1,
        y: 0,
        size,
    };
    assert_eq!(names(&draw_at(2, [6, 2])), ["c.def"]);
    let (_, _, at) = creature_frame(&s, "c.def", (2, 0), [1, 0], [6, 2]).unwrap();
    assert_eq!((at.x, at.y), (0, -273));
    let mut out = Screen::new(8, 2, FORMAT).unwrap();
    draw(&mut out, &draw_at(2, [6, 2]), &s, Bugs::Fixed).unwrap();
    let red = FORMAT.pack(RED);
    let white = FORMAT.pack(WHITE);
    assert_eq!(out.pixels()[..8], [0; 8]);
    assert_eq!(out.pixels()[8..], [0, 0, 0, red, white, 0, 0, 0]);
    assert_eq!(reach(&draw_at(2, [6, 2]), &s), Some([1, 0, 7, 2]));
    // Group 1's frame lies four rows higher, outside the two-row box: nothing drawn.
    let mut out = Screen::new(8, 2, FORMAT).unwrap();
    draw(&mut out, &draw_at(1, [6, 2]), &s, Bugs::Fixed).unwrap();
    assert_eq!(out.pixels(), [0; 16]);
    // A narrower box moves the canvas (3 / 2 − 1 − 3: at −2, the frame at columns 1 and 2) and
    // one of width 1 (0 − 1 − 3: at −3, the frame at 0 and 1) cuts the frame's first pixel.
    let mut out = Screen::new(8, 2, FORMAT).unwrap();
    draw(&mut out, &draw_at(2, [3, 2]), &s, Bugs::Fixed).unwrap();
    assert_eq!(out.pixels()[8..], [0, red, white, 0, 0, 0, 0, 0]);
    let mut out = Screen::new(8, 2, FORMAT).unwrap();
    draw(&mut out, &draw_at(2, [1, 2]), &s, Bugs::Fixed).unwrap();
    assert_eq!(out.pixels()[8..], [0, white, 0, 0, 0, 0, 0, 0]);
    // A missing DEF, group 2 or frame.
    let missing = Draw::Creature {
        def: "no.def".into(),
        group: 2,
        frame: 0,
        x: 1,
        y: 0,
        size: [6, 2],
    };
    assert_eq!(
        draw(&mut out, &missing, &s, Bugs::Fixed),
        Err("no.def".to_string())
    );
    assert_eq!(reach(&missing, &s), None);
    assert!(draw(&mut out, &draw_at(5, [6, 2]), &s, Bugs::Fixed).is_err());
    let mut lone = creature();
    lone.def.groups.retain(|g| g.id != 2);
    s.sprites.insert("c.def".into(), lone);
    assert!(draw(&mut out, &draw_at(1, [6, 2]), &s, Bugs::Fixed).is_err());
}

#[test]
fn a_shared_frame_is_the_other_defs_in_the_items_colours() {
    // E17 M6 T2a1, CLT-Q36: frame 1 of `b.def` (its pixels [224, 0-run]) in `p.def`'s colours
    // (224 green): the run skipped, as a button's; a frame past the group draws its first.
    let mut s = store();
    let mut colours = sprite();
    colours.def.palette = palette(&[(3, BLUE), (224, GREEN)]);
    s.sprites.insert("p.def".into(), colours);
    let shared = |frame| Draw::Shared {
        def: "p.def".into(),
        frames: "b.def".into(),
        frame,
        x: 0,
        y: 0,
    };
    assert_eq!(names(&shared(0)), ["p.def", "b.def"]);
    let mut out = screen(2);
    draw(&mut out, &shared(1), &s, Bugs::Fixed).unwrap();
    assert_eq!(out.pixels(), [FORMAT.pack(GREEN), 0]);
    let mut out = screen(2);
    draw(&mut out, &shared(9), &s, Bugs::Fixed).unwrap();
    assert_eq!(out.pixels(), [FORMAT.pack(BLUE), FORMAT.pack(GREEN)]);
    assert_eq!(reach(&shared(1), &s), Some([0, 0, 2, 1]));
    // A missing DEF of either kind.
    let lacking = Draw::Shared {
        def: "no.def".into(),
        frames: "b.def".into(),
        frame: 0,
        x: 0,
        y: 0,
    };
    assert_eq!(
        draw(&mut out, &lacking, &s, Bugs::Fixed),
        Err("no.def".to_string())
    );
    assert_eq!(reach(&lacking, &s), None);
    let no_frames = Draw::Shared {
        def: "p.def".into(),
        frames: "no.def".into(),
        frame: 0,
        x: 0,
        y: 0,
    };
    assert!(draw(&mut out, &no_frames, &s, Bugs::Fixed).is_err());
    assert_eq!(reach(&no_frames, &s), None);
}
