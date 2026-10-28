//! The assets a scene needs, decoded once before it is drawn (E24 M1 T3b1, *provisional*; as
//! `h3-view`'s checkpoint batch, ADR-0020 point 7, which is the client lane's; the decoded DEF,
//! [`Sprite`], is the common crate's since E24 M3 T1a). The adapter reads the bytes from the
//! archives and adds them by name; [`names`] says which a scene needs.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use core::fmt;

use h3_assets::def::{decode_frame, parse_def};
use h3_assets::fnt::{Font, decode_fnt};
use h3_assets::palette::{Palette, decode_pal};
use h3_assets::pcx::{Pcx, decode_pcx};

use h3_ui::paint::Assets;
use h3_ui::sprite::Sprite;

use crate::draw::{FrameSpans, Spans};
use crate::scene::{Draw, Scene};

/// The decoded assets, by upper-cased name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Batch {
    sprites: BTreeMap<String, Sprite>,
    /// Each DEF's frames' spans, by the frames' offsets (E24 M3 T2b2).
    spans: BTreeMap<String, BTreeMap<u32, Spans>>,
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
            let mut frames = BTreeMap::new();
            for offset in def.offsets() {
                frames.insert(offset, decode_frame(bytes, offset).map_err(failed)?);
            }
            let spans = frames.iter().map(|(&o, f)| (o, Spans::of(f))).collect();
            self.spans.insert(upper.clone(), spans);
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

    /// The DEF `name`.
    #[must_use]
    pub fn sprite(&self, name: &str) -> Option<&Sprite> {
        upper(&self.sprites, name)
    }

    /// The DEF `name` as the view's drawers take it: its frames with their spans.
    #[must_use]
    pub fn drawable(&self, name: &str) -> Option<Drawable<'_>> {
        Some(Drawable {
            sprite: upper(&self.sprites, name)?,
            spans: upper(&self.spans, name)?,
        })
    }

    /// The picture `name`.
    #[must_use]
    pub fn picture(&self, name: &str) -> Option<&Pcx> {
        upper(&self.pictures, name)
    }

    /// The palette `name`.
    #[must_use]
    pub fn palette(&self, name: &str) -> Option<&Palette> {
        upper(&self.palettes, name)
    }

    /// The font `name` (E24 M3 T2a: the dialog's texts).
    #[must_use]
    pub fn font(&self, name: &str) -> Option<&Font> {
        upper(&self.fonts, name)
    }
}

/// A DEF as the view's drawers take it (E24 M3 T2b2): its frames and their spans.
#[derive(Debug, Clone, Copy)]
pub struct Drawable<'b> {
    /// The DEF.
    pub sprite: &'b Sprite,
    spans: &'b BTreeMap<u32, Spans>,
}

impl<'b> Drawable<'b> {
    /// Frame `index` of the group of id `group`, with its spans.
    #[must_use]
    pub fn frame(&self, group: u32, index: u32) -> Option<FrameSpans<'b>> {
        let group = self.sprite.def.groups.iter().find(|g| g.id == group)?;
        let at = group
            .frames
            .get(usize::try_from(index).unwrap_or(usize::MAX))?;
        Some((
            self.sprite.frames.get(&at.offset)?,
            self.spans.get(&at.offset)?,
        ))
    }
}

impl Assets for Batch {
    fn sprite(&self, name: &str) -> Option<&Sprite> {
        Batch::sprite(self, name)
    }
    fn picture(&self, name: &str) -> Option<&Pcx> {
        Batch::picture(self, name)
    }
    fn palette(&self, name: &str) -> Option<&Palette> {
        Batch::palette(self, name)
    }
    fn font(&self, name: &str) -> Option<&Font> {
        Batch::font(self, name)
    }
}

/// `name`'s entry of a map keyed by upper-cased names, a name with no lower-case letter looked up
/// as it is (a scene's thousands of lookups a view: E24 M2 T4b3).
fn upper<'m, T>(map: &'m BTreeMap<String, T>, name: &str) -> Option<&'m T> {
    if name.bytes().any(|b| b.is_ascii_lowercase()) {
        map.get(&name.to_ascii_uppercase())
    } else {
        map.get(name)
    }
}

/// Every asset `scene` draws, upper-cased (the flag palette when a piece takes an entry of it).
#[must_use]
pub fn names(scene: &Scene) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for draw in &scene.draws {
        match draw {
            Draw::Frame {
                picture, players, ..
            } => {
                names.insert(picture.to_ascii_uppercase());
                names.insert(players.to_ascii_uppercase());
            }
            Draw::Piece { def, flag, .. } => {
                names.insert(def.to_ascii_uppercase());
                if flag.is_some() {
                    names.insert(scene.flags.to_ascii_uppercase());
                }
            }
            Draw::Gem { def, .. } => {
                names.insert(def.to_ascii_uppercase());
            }
            Draw::Dialog(draw) => {
                names.extend(
                    h3_ui::paint::names(draw)
                        .into_iter()
                        .map(str::to_ascii_uppercase),
                );
            }
        }
    }
    names
}

/// Every asset `scene` draws as the scene spells it, once each (E24 M3 T2b1: no upper-cased copy
/// a piece; a viewer keeps those it decoded by this spelling and decodes only the new ones).
#[must_use]
pub fn spelled(scene: &Scene) -> BTreeSet<&str> {
    let mut names = BTreeSet::new();
    each_spelled(scene, |name| {
        names.insert(name);
    });
    names
}

/// Hands `visit` every asset `scene` draws as the scene spells it, in order, as often as drawn.
fn each_spelled<'s>(scene: &'s Scene, mut visit: impl FnMut(&'s str)) {
    for draw in &scene.draws {
        match draw {
            Draw::Frame {
                picture, players, ..
            } => {
                visit(picture.as_str());
                visit(players.as_str());
            }
            Draw::Piece { def, flag, .. } => {
                visit(def.as_str());
                if flag.is_some() {
                    visit(scene.flags.as_str());
                }
            }
            Draw::Gem { def, .. } => {
                visit(def.as_str());
            }
            Draw::Dialog(draw) => h3_ui::paint::names(draw).into_iter().for_each(&mut visit),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests;
