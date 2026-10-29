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
    // past the group's), a darkened rectangle; unknown for a text, a label or a missing asset.
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
    let text = Draw::Text {
        item: text_item("q.pcx", "f.fnt", "game.pal"),
        x: 0,
        y: 0,
        text: "1".into(),
    };
    assert_eq!(reach(&text, &s), None);
    let label = Draw::Label {
        item: label_item("f.fnt", "game.pal"),
        x: 0,
        y: 0,
        text: "1".into(),
        disabled: false,
    };
    assert_eq!(reach(&label, &s), None);
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
