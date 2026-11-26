//! The generator's map (`rmg_map`, survey `docs/rules/surveys/E14-generator.md` §2.1;
//! `docs/exe/structs.toml`'s `rmg_map_cell`): its cells as the original's records hold them,
//! level by level, row by row, so that the steps port their bit operations as read and the
//! comparison seam (ADR-0029 point 8) turns them into the phase oracle's dump format 1
//! (`clients/capture/xdd.c` section 10).

use alloc::vec::Vec;

/// +0x28 bit 28, the land bit.
const LAND: u32 = 1 << 28;

/// One cell of the generator's map: its words +0x10 to +0x2C (`rmg_map_cell`); the objects on it
/// (+0x0) come with the steps that place them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    /// +0x10 to +0x18: the cell a zone flood came from.
    pub previous: [i32; 3],
    /// +0x1C: the floods' costs (bits 0-15 its own zone's, 16-31 a neighbouring zone's).
    pub movement: u32,
    /// +0x20: bits 0-15 the distance to the nearest entrance, 16-23 the zone index (-1 none),
    /// 24-31 the zone whose flood entered it.
    pub zone: u32,
    /// +0x24: the land, river and road types and pictures.
    pub tile: u32,
    /// +0x28: the pictures' flags and the generator's marks.
    pub tile_data: u32,
    /// +0x2C: a pending Border Guard and its colour.
    pub pending_guard: u32,
}

impl Cell {
    /// The zone byte, +0x20 bits 16-23 (0xFF none).
    pub const fn zone_byte(&self) -> u8 {
        (self.zone >> 16) as u8
    }

    /// +0x28 bit 28: the cell is land of its zone (step 2's fill and lines; step 3 paints it).
    pub const fn is_land(&self) -> bool {
        self.tile_data & LAND != 0
    }

    /// Step 2's marks: the zone byte `index` and, with `land`, +0x28 bit 28.
    pub(crate) const fn mark(&mut self, index: u8, land: bool) {
        self.zone = (self.zone & 0xFF00_FFFF) | ((index as u32) << 16);
        if land {
            self.tile_data |= LAND;
        }
    }

    /// A cell as `rmg_map_cell_init` 0x530E90 leaves it: `rmg_map_cell_reset` 0x530F10 on the
    /// heap's bytes, which a fresh start's first map holds as 0 (the phase oracle's dumps;
    /// ORACLE.md, "Cell bits from memory"): +0x10 -1, +0x14 and +0x18 kept, both costs 32700, the
    /// distance 32700 and both zone bytes -1, land 8 with picture 21 (bits 30-31 kept), bits 25
    /// and 27 set (bit 24 kept), bit 0 of +0x2C cleared.
    // rule: RMG-SET-010
    pub const RESET: Self = Self {
        previous: [-1, 0, 0],
        movement: 0x7FBC_7FBC,
        zone: 0xFFFF_7FBC,
        tile: 0x548,
        tile_data: 0x0A00_0000,
        pending_guard: 0,
    };
}

/// The generator's map: `width` × `height` cells per level, level by level, row by row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cells {
    width: i32,
    height: i32,
    levels: i32,
    cells: Vec<Cell>,
}

impl Cells {
    /// The map of `side` × `side` cells and `levels` levels as `rmg_map_init` 0x530FB0 makes it,
    /// every cell [`Cell::RESET`].
    pub fn new(side: i32, levels: i32) -> Self {
        let count = usize::try_from(side.saturating_mul(side).saturating_mul(levels)).unwrap_or(0);
        Self {
            width: side,
            height: side,
            levels,
            cells: alloc::vec![Cell::RESET; count],
        }
    }

    /// The width, the height and the levels.
    pub const fn size(&self) -> (i32, i32, i32) {
        (self.width, self.height, self.levels)
    }

    /// The cells in the map's order.
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    /// The cell at (x, y) on level z, none outside the map.
    pub fn get(&self, x: i32, y: i32, z: i32) -> Option<&Cell> {
        self.index(x, y, z).map(|i| &self.cells[i])
    }

    pub(crate) fn get_mut(&mut self, x: i32, y: i32, z: i32) -> Option<&mut Cell> {
        self.index(x, y, z).map(|i| &mut self.cells[i])
    }

    /// The cell at (x, y, z) clamped to the map (a map has cells: its side is the size's).
    pub(crate) fn clamped_mut(&mut self, x: i32, y: i32, z: i32) -> &mut Cell {
        let clamp = |v: i32, n: i32| v.min(n - 1).max(0);
        let (x, y, z) = (
            clamp(x, self.width),
            clamp(y, self.height),
            clamp(z, self.levels),
        );
        let i = ((self.height * z + y) * self.width + x).cast_unsigned() as usize;
        let last = self.cells.len().saturating_sub(1);
        &mut self.cells[i.min(last)]
    }

    /// `((height × z + y) × width + x)`, the original's index, for a cell on the map.
    fn index(&self, x: i32, y: i32, z: i32) -> Option<usize> {
        if !(0..self.width).contains(&x)
            || !(0..self.height).contains(&y)
            || !(0..self.levels).contains(&z)
        {
            return None;
        }
        usize::try_from((self.height * z + y) * self.width + x).ok()
    }

    /// The phase oracle's dump of the cells after `step` (format 1, `xdd.c` section 10):
    /// "H3RMGCEL", then little-endian 32-bit words: the format 1, the step, the width, height
    /// and levels; per cell 12 words (the objects on it and the first three's indices, -1 none,
    /// then +0x10 to +0x2C); the objects' count and six words each; the further places' count and
    /// three words each. No step places an object yet: none, none further.
    pub fn dump(&self, step: u8) -> Vec<u8> {
        let mut out = Vec::with_capacity(36 + 48 * self.cells.len());
        out.extend_from_slice(b"H3RMGCEL");
        let mut word = |w: u32| out.extend_from_slice(&w.to_le_bytes());
        for w in [1, i32::from(step), self.width, self.height, self.levels] {
            word(w.cast_unsigned());
        }
        for c in &self.cells {
            for w in [0, u32::MAX, u32::MAX, u32::MAX] {
                word(w);
            }
            for w in c.previous {
                word(w.cast_unsigned());
            }
            for w in [c.movement, c.zone, c.tile, c.tile_data, c.pending_guard] {
                word(w);
            }
        }
        word(0);
        word(0);
        out
    }
}
