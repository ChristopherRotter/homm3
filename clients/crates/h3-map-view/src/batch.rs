//! The assets a scene needs, decoded once before it is drawn (E24 M1 T3b1, *provisional*; as
//! `h3-view`'s checkpoint batch, ADR-0020 point 7, which is the client lane's: M3's common crate
//! may merge the two). The adapter reads the bytes from the archives and adds them by name;
//! [`names`] says which a scene needs.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use core::fmt;

use h3_assets::def::{Def, Frame, decode_frame, parse_def};
use h3_assets::palette::{Palette, decode_pal};
use h3_assets::pcx::{Pcx, decode_pcx};

use crate::scene::{Draw, Scene};

/// A DEF with every frame decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sprite {
    /// The DEF's header, palette and groups.
    pub def: Def,
    /// Its frames, by their offset in the DEF.
    pub frames: BTreeMap<u32, Frame>,
}

impl Sprite {
    /// Frame `index` of the group of id `group`.
    #[must_use]
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
        self.sprites.get(&name.to_ascii_uppercase())
    }

    /// The picture `name`.
    #[must_use]
    pub fn picture(&self, name: &str) -> Option<&Pcx> {
        self.pictures.get(&name.to_ascii_uppercase())
    }

    /// The palette `name`.
    #[must_use]
    pub fn palette(&self, name: &str) -> Option<&Palette> {
        self.palettes.get(&name.to_ascii_uppercase())
    }
}

/// Every asset `scene` draws, upper-cased.
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
            Draw::Piece { def, .. } | Draw::Gem { def, .. } => {
                names.insert(def.to_ascii_uppercase());
            }
        }
    }
    names
}

#[cfg(test)]
pub(crate) mod tests;
