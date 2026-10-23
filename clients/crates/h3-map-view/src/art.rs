//! The static art of a map (E24 M1 T2, *provisional*): what the view draws that the map file
//! fixes, read from the raw model ([`h3_map::RawMap`]; the import keeps none of it, E24 T0's
//! finding). Each tile's terrain, river and road with their pictures and mirrors
//! (`docs/epics/E24-design.md` §1.3), each template's DEF, masks and flatness (§2.1), each
//! object's anchor and template, by its index in the map's object list (the index the original's
//! draw lists hold, §2.1; the live part's objects follow it, [`crate::live`]).
//!
//! [`MapArt::from_raw`] checks only what the raw model leaves open: the tile count, the objects'
//! templates and the DEFs' names. A code or picture the asset tables or the DEFs lack is the
//! scene's to refuse, since only the tables and the loaded DEFs know them (E24 design §1.3: no
//! pass of the original checks a picture against its DEF's frames).

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use h3_map::RawMap;
use h3_map::model::{self, Pos};

/// The art of one map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapArt {
    /// The side of a level, in tiles.
    pub size: usize,
    /// The levels (1, or 2 with the underground).
    pub levels: usize,
    /// `size × size × levels` tiles; tile (x, y, z) at `(z × size + y) × size + x`.
    pub tiles: Vec<TileArt>,
    /// The templates, in the map's order.
    pub templates: Vec<TemplateArt>,
    /// The objects, in the map's order: an object's index is its place here.
    pub objects: Vec<ObjectArt>,
}

/// One tile's pictures (the `.h3m` tile record, as the original keeps it in map_item +0x4 to
/// +0xC; E24 design §1.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileArt {
    /// The terrain: its code (0 dirt … 9 rock), picture and mirrors (flags bits 0, 1).
    pub terrain: Piece,
    /// The river, if any: code 1 clear … 4 lava, picture, mirrors (flags bits 2, 3).
    pub river: Option<Piece>,
    /// The road, if any: code 1 dirt … 3 cobblestone, picture, mirrors (flags bits 4, 5).
    pub road: Option<Piece>,
    /// The tile record's flags as stored (the mirrors above, bit 6 the coast; E08 T19a).
    pub flags: u8,
}

/// The water's terrain code.
pub const WATER: u8 = 8;
/// The water's pictures below this one are its shores (map_tiles_load 0x4FE220).
pub const SHORE_PICTURES: u8 = 20;
/// The tile record's coast bit (on land, the map file's own).
pub const COAST_BIT: u8 = 0x40;

impl TileArt {
    /// The coast (map_item +0xD bit 0x02, map_tiles_load 0x4FE220; design §2.8): water of a
    /// shore picture, or land whose record has the coast bit. A boat's waves are not drawn there
    /// (0x40FE30, 0x410760).
    #[must_use]
    pub fn coast(&self) -> bool {
        if self.terrain.code == WATER {
            self.terrain.picture < SHORE_PICTURES
        } else {
            self.flags & COAST_BIT != 0
        }
    }
}

/// One layer of a tile: group 0's frame `picture` of the DEF the asset tables give the code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    /// The `.h3m`'s code of the terrain, river or road.
    pub code: u8,
    /// The frame.
    pub picture: u8,
    /// Mirrored left to right.
    pub mirror_x: bool,
    /// Mirrored top to bottom.
    pub mirror_y: bool,
}

/// A template as the view draws it (E24 design §2.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateArt {
    /// Its DEF's name, as the map gives it (the archives look names up ignoring case).
    pub def: String,
    /// The passability mask as stored (a set bit passable; [`TemplateArt::passable`]).
    pub passable: [u8; 6],
    /// The visitability mask as stored (a set bit visitable).
    pub visitable: [u8; 6],
    /// The object class (34 hero, 8 boat, 98 town, …).
    pub class: u32,
    /// The subclass (a hero's class, a creature, a faction, …).
    pub subclass: u32,
    /// The template's priority byte: not 0 makes it flat (inferred, design §2.1: the original's
    /// template +0x40, which the special grounds set; T4 confirms).
    pub priority: u8,
}

/// The cells a template's masks cover: 8 columns, 6 rows.
pub const CELLS: (u8, u8) = (8, 6);

impl TemplateArt {
    /// Whether the cell `dx` columns left of and `dy` rows above the anchor is passable (cells
    /// outside the masks' 8 × 6 are).
    #[must_use]
    pub fn passable(&self, dx: u8, dy: u8) -> bool {
        mask_bit(&self.passable, dx, dy).unwrap_or(true)
    }

    /// Whether the cell is visitable (cells outside the masks are not).
    #[must_use]
    pub fn visitable(&self, dx: u8, dy: u8) -> bool {
        mask_bit(&self.visitable, dx, dy).unwrap_or(false)
    }

    /// Flat: drawn in the flat objects' pass, every cell at layer 0 (design §2.1, §2.2).
    #[must_use]
    pub fn flat(&self) -> bool {
        self.priority != 0
    }
}

/// Bit `7 − dx` of byte `5 − dy` (`docs/formats/h3m.md`, "Template masks").
pub(crate) fn mask_bit(mask: &[u8; 6], dx: u8, dy: u8) -> Option<bool> {
    let row = (CELLS.1 - 1).checked_sub(dy);
    let bit = (CELLS.0 - 1).checked_sub(dx);
    row.zip(bit)
        .and_then(|(r, b)| mask.get(usize::from(r)).map(|byte| byte >> b & 1 != 0))
}

/// An object as the view draws it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectArt {
    /// Its anchor: the template's bottom-right corner (it may lie past the map's edges).
    pub anchor: Pos,
    /// Its template, an index into [`MapArt::templates`].
    pub template: usize,
}

/// Why a raw map has no art.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtError {
    /// The terrain holds a number of tiles other than its size and levels give.
    Tiles {
        /// The count the size and levels give.
        expected: usize,
        /// The count held.
        found: usize,
    },
    /// A template's DEF name is empty or not printable ASCII.
    Name(usize),
    /// An object names a template the map does not have.
    Template(usize),
}

impl fmt::Display for ArtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tiles { expected, found } => {
                write!(f, "the terrain has {found} tiles, not {expected}")
            }
            Self::Name(t) => write!(f, "template {t}: its DEF's name is not printable ASCII"),
            Self::Template(o) => write!(f, "object {o}: no such template"),
        }
    }
}

impl MapArt {
    /// The art of a raw map.
    ///
    /// # Errors
    /// [`ArtError`]: the tiles do not fill the levels, a template's name is not a name, or an
    /// object's template is missing.
    pub fn from_raw(raw: &RawMap) -> Result<Self, ArtError> {
        let terrain = &raw.terrain;
        let expected = terrain
            .size
            .saturating_mul(terrain.size)
            .saturating_mul(terrain.levels);
        if terrain.tiles.len() != expected {
            return Err(ArtError::Tiles {
                expected,
                found: terrain.tiles.len(),
            });
        }
        let mut templates = Vec::with_capacity(raw.templates.len());
        for (i, t) in raw.templates.iter().enumerate() {
            let name = &t.def.0;
            if name.is_empty() || !name.iter().all(u8::is_ascii_graphic) {
                return Err(ArtError::Name(i));
            }
            templates.push(TemplateArt {
                def: name.iter().map(|&b| char::from(b)).collect(),
                passable: t.passable,
                visitable: t.visitable,
                class: t.class,
                subclass: t.subclass,
                priority: t.priority,
            });
        }
        let mut objects = Vec::with_capacity(raw.objects.len());
        for (i, o) in raw.objects.iter().enumerate() {
            let template = usize::try_from(o.template)
                .ok()
                .filter(|&t| t < templates.len())
                .ok_or(ArtError::Template(i))?;
            objects.push(ObjectArt {
                anchor: o.pos,
                template,
            });
        }
        Ok(Self {
            size: terrain.size,
            levels: terrain.levels,
            tiles: terrain.tiles.iter().map(tile).collect(),
            templates,
            objects,
        })
    }

    /// The tile at (x, y, z), if it is on the map.
    #[must_use]
    pub fn tile(&self, x: i32, y: i32, z: u8) -> Option<&TileArt> {
        let (x, y) = (usize::try_from(x).ok()?, usize::try_from(y).ok()?);
        let z = usize::from(z);
        if x >= self.size || y >= self.size || z >= self.levels {
            return None;
        }
        self.tiles.get((z * self.size + y) * self.size + x)
    }

    /// The highest picture any tile gives each code of a layer, by code ascending (what the
    /// layer's DEFs must hold: the real-asset gate checks the shipped maps against them).
    #[must_use]
    pub fn pictures(&self, layer: Layer) -> Vec<(u8, u8)> {
        let mut highest: [Option<u8>; 256] = [None; 256];
        for t in &self.tiles {
            let piece = match layer {
                Layer::Terrain => Some(t.terrain),
                Layer::River => t.river,
                Layer::Road => t.road,
            };
            if let Some(p) = piece {
                let slot = &mut highest[usize::from(p.code)];
                *slot = Some(slot.map_or(p.picture, |h| h.max(p.picture)));
            }
        }
        (0..=u8::MAX)
            .zip(highest)
            .filter_map(|(code, h)| h.map(|h| (code, h)))
            .collect()
    }
}

/// A tile's three layers (E24 design §1.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Layer {
    /// The terrain.
    Terrain,
    /// The rivers.
    River,
    /// The roads.
    Road,
}

/// A raw tile's pieces: code 0 is no river or road.
fn tile(t: &model::Tile) -> TileArt {
    let piece = |code, picture, shift: u8| Piece {
        code,
        picture,
        mirror_x: t.flags & (1 << shift) != 0,
        mirror_y: t.flags & (2 << shift) != 0,
    };
    TileArt {
        terrain: piece(t.terrain, t.view, 0),
        river: (t.river != 0).then(|| piece(t.river, t.river_dir, 2)),
        road: (t.road != 0).then(|| piece(t.road, t.road_dir, 4)),
        flags: t.flags,
    }
}

#[cfg(test)]
pub(crate) mod tests;
