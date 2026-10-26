//! A checkpoint's batch (ADR-0020 point 7): every asset the coming screen needs, decoded once
//! before it is shown, so that nothing loads while it is. The adapter reads the bytes from the
//! archives (whole archives until CLT-Q2's table-only reader lands) and adds them here by name;
//! [`names`] says which a draw list needs.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use h3_assets::def::{Def, Frame, decode_frame, parse_def};
use h3_assets::fnt::{Font, decode_fnt};
use h3_assets::palette::{Palette, decode_pal};
use h3_assets::pcx::{Pcx, decode_pcx};

use crate::layout::BattleLayout;
use crate::scene::Draw;

/// A DEF with every frame decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sprite {
    /// The DEF's header, palette and groups.
    pub def: Def,
    /// Its frames, by their offset in the DEF.
    pub frames: BTreeMap<u32, Frame>,
}

impl Sprite {
    /// Frame `index` of group `group` (by the group's id).
    pub fn frame(&self, group: u32, index: u32) -> Option<&Frame> {
        let group = self.def.groups.iter().find(|g| g.id == group)?;
        let at = group
            .frames
            .get(usize::try_from(index).unwrap_or(usize::MAX))?;
        self.frames.get(&at.offset)
    }
}

/// The decoded assets, by upper-cased name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Batch {
    sprites: BTreeMap<String, Sprite>,
    pictures: BTreeMap<String, Pcx>,
    palettes: BTreeMap<String, Palette>,
    fonts: BTreeMap<String, Font>,
}

/// Why an asset is not in the batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchError {
    /// The name's extension is not one a batch holds (`.DEF`, `.PCX`, `.PAL`, `.FNT`).
    Kind(String),
    /// The bytes do not decode: the name and the decoder's error.
    Decode(String, String),
}

impl fmt::Display for BatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Kind(name) => write!(f, "{name}: not a DEF, PCX, PAL or FNT"),
            Self::Decode(name, why) => write!(f, "{name}: {why}"),
        }
    }
}

impl Batch {
    /// Decodes `bytes` as the asset `name` (by its extension) and keeps it.
    ///
    /// # Errors
    /// An extension the batch does not hold, bytes that do not decode.
    pub fn add(&mut self, name: &str, bytes: &[u8]) -> Result<(), BatchError> {
        let upper = name.to_ascii_uppercase();
        let failed = |e: h3_assets::Error| BatchError::Decode(upper.clone(), format!("{e:?}"));
        if upper.ends_with(".DEF") {
            let def = parse_def(bytes).map_err(failed)?;
            let frames = def
                .offsets()
                .into_iter()
                .map(|at| decode_frame(bytes, at).map(|f| (at, f)))
                .collect::<Result<_, _>>()
                .map_err(failed)?;
            self.sprites.insert(upper, Sprite { def, frames });
        } else if upper.ends_with(".PCX") {
            let picture = decode_pcx(bytes).map_err(failed)?;
            self.pictures.insert(upper, picture);
        } else if upper.ends_with(".PAL") {
            let palette = decode_pal(bytes).map_err(failed)?.palette;
            self.palettes.insert(upper, palette);
        } else if upper.ends_with(".FNT") {
            let font = decode_fnt(bytes).map_err(failed)?;
            self.fonts.insert(upper, font);
        } else {
            return Err(BatchError::Kind(upper));
        }
        Ok(())
    }

    /// The DEF `name`, any case.
    pub fn sprite(&self, name: &str) -> Option<&Sprite> {
        self.sprites.get(&name.to_ascii_uppercase())
    }

    /// The picture `name`, any case.
    pub fn picture(&self, name: &str) -> Option<&Pcx> {
        self.pictures.get(&name.to_ascii_uppercase())
    }

    /// The palette `name`, any case.
    pub fn palette(&self, name: &str) -> Option<&Palette> {
        self.palettes.get(&name.to_ascii_uppercase())
    }

    /// The font `name`, any case (E17 M3 T4a).
    pub fn font(&self, name: &str) -> Option<&Font> {
        self.fonts.get(&name.to_ascii_uppercase())
    }
}

/// The batch a draw list needs: every name in it (a flag's palette too, a picture's players'
/// palette, a text's picture, font and font palette), the layout's count box's picture, font and
/// font palette when it draws a count box, and the outline palette, upper-cased.
pub fn names(list: &[Draw], layout: &BattleLayout) -> BTreeSet<String> {
    let boxes = &layout.count_box;
    let named = list.iter().flat_map(|d| match d {
        Draw::Background { picture, .. }
        | Draw::AbsoluteObstacle { picture, .. }
        | Draw::Wall { picture, .. } => vec![picture],
        Draw::Obstacle { def, .. }
        | Draw::Projectile { def, .. }
        | Draw::Explosion { def, .. }
        | Draw::Creature { def, .. }
        | Draw::Shooter { def, .. }
        | Draw::Hero { def, .. }
        | Draw::Magic { def, .. }
        | Draw::Tiled { def, .. } => vec![def],
        Draw::Flag { def, palette, .. } => vec![def, palette],
        Draw::CountBox { .. } => vec![&boxes.picture, &boxes.font, &boxes.font_palette],
        Draw::Picture {
            picture, player, ..
        } => {
            let mut names = vec![picture];
            names.extend(player.as_ref().map(|p| &p.palette));
            names
        }
        Draw::Button { def, .. } | Draw::Cursor { def, .. } => vec![def],
        Draw::Text { item, .. } => vec![&item.picture, &item.font, &item.font_palette],
        Draw::Ray(_) | Draw::Within { .. } | Draw::Shaken { .. } => Vec::new(),
    });
    let mut all: BTreeSet<String> = named
        .chain(core::iter::once(&layout.outline_palette))
        .map(|n| n.to_ascii_uppercase())
        .collect();
    // A list drawn within a rectangle names its own (E17 M4 T2b), a shaken one too (T2c-2).
    for d in list {
        if let Draw::Within { list, .. } | Draw::Shaken { list, .. } = d {
            all.extend(names(list, layout));
        }
    }
    all
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;
    use h3_assets::build;
    use h3_assets::def::Encoding;

    /// A one-pixel frame of index `i`.
    pub(crate) fn pixel(i: u8) -> Frame {
        Frame {
            encoding: Encoding::Raw,
            full_width: 1,
            full_height: 1,
            left: 0,
            top: 0,
            width: 1,
            height: 1,
            pixels: vec![i],
            runs: vec![false],
            short_header: false,
        }
    }

    #[test]
    fn a_draw_list_names_its_batch() {
        let layout = crate::scene::tests::layout();
        let picture = |picture: &str| Draw::Background {
            picture: picture.to_string(),
            x: 0,
            y: 0,
        };
        let list = [
            picture("bg.pcx"),
            Draw::AbsoluteObstacle {
                picture: "Abs.pcx".to_string(),
                x: 0,
                y: 0,
            },
            Draw::Obstacle {
                def: "ob.def".to_string(),
                group: 0,
                frame: 0,
                x: 0,
                y: 0,
            },
            Draw::Creature {
                def: "CR.DEF".to_string(),
                group: 2,
                frame: 0,
                anchor: [0, 0],
                mirrored: false,
                outline: None,
            },
            picture("BG.pcx"),
            Draw::Projectile {
                def: "PlcBowX.def".to_string(),
                frame: 0,
                x: 0,
                y: 0,
                mirrored: false,
            },
            // A ray names nothing (M3 T1b-1).
            Draw::Ray(crate::anim::Ray {
                strokes: vec![],
                colours: vec![],
            }),
            // A hero its DEF, a flag its DEF and its players' palette (M3 T3a).
            Draw::Hero {
                def: "Ch00.def".to_string(),
                group: 0,
                frame: 0,
                x: 0,
                y: 0,
                mirrored: true,
            },
            Draw::Flag {
                def: "CmFlagL.def".to_string(),
                frame: 0,
                x: 0,
                y: 0,
                palette: "Players.pal".to_string(),
                player: 0,
            },
            // A magic animation its DEF (M3 T3b).
            Draw::Magic {
                def: "sp12_.def".to_string(),
                frame: 0,
                x: 0,
                y: 0,
                mirrored: false,
                translucent: true,
            },
            // A tiled animation its DEF (M3 T7c-2).
            Draw::Tiled {
                def: "C06spF0.def".to_string(),
                frame: 0,
                area: [800, 556],
            },
            // A count box the layout's picture, font and its palette (M3 T4a).
            Draw::CountBox {
                stack: 0,
                x: 0,
                y: 0,
                count: 1,
                tint: [0, 0],
            },
            // A dialog's text item its picture, font and its palette (M3 T4b-2b-1, T6a).
            Draw::Text {
                item: layout.log().unwrap().clone(),
                x: 0,
                y: 0,
                text: "x".to_string(),
            },
            // A picture its own name and the players' palette; a button its DEF (M3 T6a).
            Draw::Picture {
                picture: "cbar.pcx".to_string(),
                x: 0,
                y: 0,
                player: Some(crate::scene::PlayerColours {
                    palette: "Players.pal".to_string(),
                    player: 0,
                }),
            },
            Draw::Picture {
                picture: "other.pcx".to_string(),
                x: 0,
                y: 0,
                player: None,
            },
            Draw::Button {
                def: "icm001.def".to_string(),
                frame: 0,
                x: 0,
                y: 0,
            },
            Draw::Cursor {
                def: "CrCombat.def".to_string(),
                frame: 6,
                x: 0,
                y: 0,
                hot: [0, 0],
            },
            // A siege's part its picture, a shooter its DEF (M4 T1b).
            Draw::Wall {
                picture: "SgCsTw21.pcx".to_string(),
                x: 0,
                y: 0,
                clip: None,
            },
            Draw::Shooter {
                def: "clcbow.def".to_string(),
                group: 2,
                frame: 0,
                anchor: [0, 0],
                mirrored: true,
                outline: None,
                rows: 232,
            },
            // A list within a rectangle its own names (M4 T2b: a wall shot's explosion).
            Draw::Within {
                rect: [0, 0, 1, 1],
                list: vec![Draw::Explosion {
                    def: "SgExpl.def".to_string(),
                    frame: 0,
                    x: 0,
                    y: 0,
                    rows: 1,
                }],
            },
            // A shaken list too (M4 T2c-2: Earthquake).
            Draw::Shaken {
                offsets: vec![[1, 1]],
                rect: [0, 0, 1, 1],
                list: vec![Draw::Background {
                    picture: "Shaken.pcx".to_string(),
                    x: 0,
                    y: 0,
                }],
            },
        ];
        let names: Vec<String> = names(&list, &layout).into_iter().collect();
        assert_eq!(
            names,
            [
                "ABS.PCX",
                "BG.PCX",
                "C06SPF0.DEF",
                "CBAR.PCX",
                "CH00.DEF",
                "CLCBOW.DEF",
                "CMFLAGL.DEF",
                "CMNUMWIN.PCX",
                "CR.DEF",
                "CRCOMBAT.DEF",
                "CROLLOVR.PCX",
                "GAME.PAL",
                "ICM001.DEF",
                "OB.DEF",
                "OTHER.PCX",
                "PLAYERS.PAL",
                "PLCBOWX.DEF",
                "SGCSTW21.PCX",
                "SGEXPL.DEF",
                "SHAKEN.PCX",
                "SMALFONT.FNT",
                "SP12_.DEF",
                "TINY.FNT"
            ]
        );
    }

    #[test]
    fn assets_decode_by_their_extension() {
        let def = build::def(
            0x42,
            (1, 1),
            &Palette::default(),
            &[(2, &[("a", 0)])],
            &[pixel(3)],
        );
        let pcx = build::pcx_indexed(1, 1, &[0], &Palette::default());
        let pal = build::pal(&Palette::default(), &[0; 256], 0, &[]);
        let mut batch = Batch::default();
        batch.add("s.def", &def).unwrap();
        batch.add("B.pcx", &pcx).unwrap();
        batch.add("game.pal", &pal).unwrap();
        let font = Font {
            height: 1,
            y_offset: 0,
            glyphs: vec![h3_assets::fnt::Glyph::default(); 256],
        };
        batch.add("tiny.fnt", &build::fnt(&font)).unwrap();
        assert_eq!(batch.font("TINY.fnt"), Some(&font));
        let sprite = batch.sprite("S.DEF").unwrap();
        assert_eq!(sprite.frame(2, 0), Some(&pixel(3)));
        assert_eq!(sprite.frame(2, 1), None);
        assert_eq!(sprite.frame(9, 0), None);
        assert!(batch.picture("b.PCX").is_some());
        assert_eq!(batch.palette("GAME.PAL"), Some(&Palette::default()));
        assert_eq!(batch.sprite("t.def"), None);
        assert_eq!(
            batch.add("x.txt", b""),
            Err(BatchError::Kind("X.TXT".to_string()))
        );
        for name in ["Y.DEF", "Y.PCX", "Y.PAL", "Y.FNT"] {
            let error = batch.add(name, b"bad").unwrap_err().to_string();
            assert!(error.starts_with(&format!("{name}: ")), "{error}");
        }
        // A header that parses with a frame that does not decode (its encoding field).
        let at = usize::try_from(parse_def(&def).unwrap().groups[0].frames[0].offset).unwrap();
        let mut bad = def.clone();
        bad[at + 4..at + 8].copy_from_slice(&9u32.to_le_bytes());
        assert!(parse_def(&bad).is_ok());
        let error = batch.add("z.def", &bad).unwrap_err().to_string();
        assert!(error.starts_with("Z.DEF: "), "{error}");
        assert_eq!(
            BatchError::Kind("A".to_string()).to_string(),
            "A: not a DEF, PCX, PAL or FNT"
        );
        assert_eq!(
            BatchError::Decode("A".to_string(), "b".to_string()).to_string(),
            "A: b"
        );
    }
}
