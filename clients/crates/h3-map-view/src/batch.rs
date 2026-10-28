//! The assets a scene needs, decoded once before it is drawn (E24 M1 T3b1, *provisional*; as
//! `h3-view`'s checkpoint batch, ADR-0020 point 7, which is the client lane's; the decoded DEF,
//! [`Sprite`], is the common crate's since E24 M3 T1a). The adapter reads the bytes from the archives and adds them by name;
//! [`names`] says which a scene needs.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use core::fmt;

use h3_assets::def::{decode_frame, parse_def};
use h3_assets::palette::{Palette, decode_pal};
use h3_assets::pcx::{Pcx, decode_pcx};

use h3_ui::sprite::Sprite;

use crate::scene::{Draw, Scene};

/// The decoded assets, by upper-cased name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Batch {
    sprites: BTreeMap<String, Sprite>,
    pictures: BTreeMap<String, Pcx>,
    palettes: BTreeMap<String, Palette>,
}

/// Why an asset is not in the batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchError {
    /// The name's extension is not one a batch holds (`.DEF`, `.PCX`, `.PAL`).
    Kind(String),
    /// The bytes do not decode: the name and the decoder's error.
    Decode(String, String),
}

impl fmt::Display for BatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Kind(name) => write!(f, "{name}: not a DEF, PCX or PAL"),
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
            self.sprites.insert(upper, Sprite { def, frames });
        } else if upper.ends_with(".PCX") {
            let picture = decode_pcx(bytes).map_err(failed)?;
            self.pictures.insert(upper, picture);
        } else if upper.ends_with(".PAL") {
            let palette = decode_pal(bytes).map_err(failed)?.palette;
            self.palettes.insert(upper, palette);
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
        }
    }
    names
}

#[cfg(test)]
pub(crate) mod tests;
