use super::*;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use h3_assets::build;
use h3_assets::def::{Encoding, Frame};
use h3_assets::palette::Rgb;

use crate::scene::Pass;

pub(crate) fn raw_frame(width: u32, height: u32, pixels: &[u8]) -> Frame {
    Frame {
        encoding: Encoding::Raw,
        full_width: width,
        full_height: height,
        left: 0,
        top: 0,
        width,
        height,
        runs: vec![false; pixels.len()],
        pixels: pixels.to_vec(),
        short_header: false,
    }
}

/// A DEF of `frames` in group 0 (named `f0`, `f1`, …), its canvas the first frame's.
pub(crate) fn def(palette: &Palette, frames: &[Frame]) -> Vec<u8> {
    let names: Vec<String> = (0..frames.len()).map(|i| alloc::format!("f{i}")).collect();
    let refs: Vec<(&str, usize)> = names.iter().map(String::as_str).zip(0..).collect();
    let size = frames
        .first()
        .map_or((0, 0), |f| (f.full_width, f.full_height));
    build::def(0x45, size, palette, &[(0, &refs)], frames)
}

fn grey() -> Palette {
    Palette([Rgb { r: 9, g: 9, b: 9 }; 256])
}

#[test]
fn holds_sprites_pictures_and_palettes_by_any_case() {
    let mut batch = Batch::default();
    let frames = [raw_frame(2, 1, &[1, 2]), raw_frame(2, 1, &[3, 4])];
    batch.add("Tile.def", &def(&grey(), &frames)).unwrap();
    batch
        .add("frame.PCX", &build::pcx_indexed(1, 1, &[7], &grey()))
        .unwrap();
    batch
        .add("players.pal", &build::pal(&grey(), &[0; 256], 0, &[]))
        .unwrap();
    let sprite = batch.sprite("TILE.DEF").unwrap();
    assert_eq!(sprite.frame(0, 1).unwrap().pixels, [3, 4]);
    assert_eq!(sprite.frame(0, 2), None);
    assert_eq!(sprite.frame(1, 0), None);
    assert!(batch.picture("FRAME.pcx").is_some());
    assert_eq!(batch.palette("Players.Pal"), Some(&grey()));
    assert!(batch.sprite("frame.pcx").is_none());
}

#[test]
fn refuses_what_it_does_not_hold_or_cannot_decode() {
    let mut batch = Batch::default();
    assert_eq!(
        batch.add("sound.wav", &[]),
        Err(BatchError::Kind("SOUND.WAV".to_string()))
    );
    for name in ["a.def", "a.pcx", "a.pal", "a.fnt"] {
        assert!(matches!(
            batch.add(name, &[1, 2, 3]),
            Err(BatchError::Decode(n, _)) if n == name.to_ascii_uppercase()
        ));
    }
    // A DEF whose header reads but whose frame does not: its offset lies past the end.
    let mut bytes = def(&grey(), &[raw_frame(2, 1, &[1, 2])]);
    bytes.truncate(bytes.len() - 4);
    assert!(matches!(
        batch.add("cut.def", &bytes),
        Err(BatchError::Decode(..))
    ));
    assert_eq!(
        BatchError::Kind("X".to_string()).to_string(),
        "X: not a DEF, PCX, PAL or FNT"
    );
    assert_eq!(
        BatchError::Decode("X".to_string(), "why".to_string()).to_string(),
        "X: why"
    );
}

#[test]
fn the_names_a_scene_draws() {
    let scene = Scene {
        draws: vec![
            Draw::Frame {
                picture: "AdvMap.pcx".to_string(),
                players: "Players.pal".to_string(),
                viewer: 0,
            },
            Draw::Piece {
                pass: Pass::Terrain,
                def: "grastl.def".to_string(),
                group: 0,
                frame: 0,
                x: 0,
                y: 0,
                mirror_x: false,
                mirror_y: false,
                clip: [0; 4],
                flag: None,
            },
            Draw::Piece {
                pass: Pass::Body,
                def: "avmgems0.def".to_string(),
                group: 0,
                frame: 0,
                x: 0,
                y: 0,
                mirror_x: false,
                mirror_y: false,
                clip: [0; 4],
                flag: Some(63),
            },
            Draw::Gem {
                def: "agemul.def".to_string(),
                frame: 0,
                x: 0,
                y: 0,
            },
        ],
        cycles: vec![],
        flags: "game.pal".to_string(),
        bugs: h3_render::Bugs::Fixed,
    };
    // The flag palette with them, since a piece takes an entry of it.
    let got: Vec<String> = names(&scene).into_iter().collect();
    assert_eq!(
        got,
        [
            "ADVMAP.PCX",
            "AGEMUL.DEF",
            "AVMGEMS0.DEF",
            "GAME.PAL",
            "GRASTL.DEF",
            "PLAYERS.PAL"
        ]
    );
}

#[test]
fn holds_a_font_and_gives_the_dialogs_drawers_its_assets() {
    // E24 M3 T2a: the dialog's texts' fonts; the batch is `h3_ui::paint`'s assets.
    use h3_ui::paint::Assets;
    let font = Font {
        height: 1,
        y_offset: 0,
        glyphs: vec![h3_assets::fnt::Glyph::default(); 256],
    };
    let mut batch = Batch::default();
    batch.add("smalfont.fnt", &build::fnt(&font)).unwrap();
    batch
        .add("Players.pal", &build::pal(&grey(), &[0; 256], 0, &[]))
        .unwrap();
    assert_eq!(Assets::font(&batch, "SMALFONT.FNT"), Some(&font));
    assert_eq!(Assets::palette(&batch, "players.pal"), Some(&grey()));
    assert!(Assets::picture(&batch, "a.pcx").is_none());
    assert!(Assets::sprite(&batch, "a.def").is_none());
    let button = h3_ui::ui::Draw::Button {
        def: "iam002.def".to_string(),
        frame: 0,
        x: 0,
        y: 0,
        mirrored: false,
        player: Some(h3_ui::ui::PlayerColours {
            palette: "Players.pal".to_string(),
            player: 1,
        }),
    };
    let names = names(&Scene {
        draws: vec![
            Draw::Dialog(h3_ui::ui::Draw::Darken { rect: [0; 4] }),
            Draw::Dialog(button),
        ],
        cycles: vec![],
        flags: "game.pal".to_string(),
        bugs: h3_render::Bugs::Fixed,
    });
    let names: Vec<String> = names.into_iter().collect();
    assert_eq!(names, ["IAM002.DEF", "PLAYERS.PAL"]);
}
