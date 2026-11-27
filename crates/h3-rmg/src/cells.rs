//! The generator's map (`rmg_map`, survey `docs/rules/surveys/E14-generator.md` §2.1;
//! `docs/exe/structs.toml`'s `rmg_map_cell`): its cells as the original's records hold them,
//! level by level, row by row, so that the steps port their bit operations as read and the
//! comparison seam (ADR-0029 point 8) turns them into the phase oracle's dump format 1
//! (`clients/capture/xdd.c` section 10), each cell's objects with it ([`Cells::objects_at`]).

use alloc::vec::Vec;

use crate::objects::{Object, ObjectFault, Shape};

/// +0x28 bit 28, the land bit.
const LAND: u32 = 1 << 28;
/// +0x28 bit 22: an object's entrance.
pub(crate) const ENTRANCE: u32 = 1 << 22;
/// +0x28 bit 25: free (cleared under an object's blocked cells).
pub(crate) const FREE: u32 = 1 << 25;
/// +0x28 bit 26: to be filled with obstacles.
pub(crate) const OBSTACLES: u32 = 1 << 26;
/// +0x28 bit 27: kept free.
pub(crate) const KEPT: u32 = 1 << 27;

/// One cell of the generator's map: its words +0x10 to +0x2C (`rmg_map_cell`); the objects on it
/// (+0x0) are the map's lists ([`Cells::objects_at`]).
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

    /// The land type, +0x24 bits 0-5 (8 water, 9 rock).
    pub const fn land(&self) -> u32 {
        self.tile & 0x3F
    }

    /// The zone byte as the signed index the placing tests compare (-1 none).
    pub(crate) const fn zone_index(&self) -> i32 {
        self.zone_byte() as i8 as i32
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

/// The generator's map: `width` × `height` cells per level, level by level, row by row, and per
/// cell the objects on it (`rmg_map_cell` +0x0: their indices among the generator's placed
/// objects, in the order placed).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cells {
    width: i32,
    height: i32,
    levels: i32,
    cells: Vec<Cell>,
    objects: Vec<Vec<u32>>,
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
            objects: alloc::vec![Vec::new(); count],
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

    /// The objects on the cell at (x, y, z), by their index among the placed objects, in the
    /// order placed; none outside the map.
    pub fn objects_at(&self, x: i32, y: i32, z: i32) -> &[u32] {
        self.index(x, y, z).map_or(&[], |i| &self.objects[i])
    }

    /// `rmg_map_place_object` 0x531EA0 for object `object` (its index among the placed objects)
    /// of `shape` at (x, y, z): per footprint cell (x − c, y − r), rows r and columns c from 0,
    /// those on the map (the level unchecked), the masks' bit 8r + c ([`Shape::cell`]): an
    /// entrance gets bit 22 and, unless +0x2C bit 0, bit 26 cleared and 27 set, its bit 25 kept;
    /// any other cell not passable loses bit 25; both get the object appended to their list;
    /// passable cells are left alone.
    ///
    /// # Errors
    ///
    /// [`ObjectFault::MaskBit`] past the masks' bits, [`ObjectFault::Outside`] on a level the
    /// map has not (the original stops there, its cells written so far kept).
    // rule: RMG-OBJ-009
    pub(crate) fn place(
        &mut self,
        object: u32,
        shape: &Shape,
        [x, y, z]: [i32; 3],
    ) -> Result<(), ObjectFault> {
        for r in 0..shape.height {
            let v = y.wrapping_sub(r);
            if !(0..self.height).contains(&v) {
                continue;
            }
            for c in 0..shape.width {
                let u = x.wrapping_sub(c);
                if !(0..self.width).contains(&u) {
                    continue;
                }
                let (passable, entrance) = shape.cell(c, r).ok_or(ObjectFault::MaskBit)?;
                let i = self.index(u, v, z).ok_or(ObjectFault::Outside)?;
                let cell = &mut self.cells[i];
                if entrance {
                    cell.tile_data |= ENTRANCE;
                    if cell.pending_guard & 1 == 0 {
                        cell.tile_data = (cell.tile_data & !OBSTACLES) | KEPT;
                    }
                } else if passable {
                    continue;
                } else {
                    cell.tile_data &= !FREE;
                }
                self.objects[i].push(object);
            }
        }
        Ok(())
    }

    /// The cell at (x, y, z) clamped to the map on each axis.
    pub(crate) fn clamped(&self, x: i32, y: i32, z: i32) -> &Cell {
        &self.cells[self.clamped_index(x, y, z)]
    }

    /// The cell at (x, y, z) clamped to the map (a map has cells: its side is the size's).
    pub(crate) fn clamped_mut(&mut self, x: i32, y: i32, z: i32) -> &mut Cell {
        let i = self.clamped_index(x, y, z);
        &mut self.cells[i]
    }

    fn clamped_index(&self, x: i32, y: i32, z: i32) -> usize {
        let clamp = |v: i32, n: i32| v.min(n - 1).max(0);
        let (x, y, z) = (
            clamp(x, self.width),
            clamp(y, self.height),
            clamp(z, self.levels),
        );
        let i = ((self.height * z + y) * self.width + x).cast_unsigned() as usize;
        i.min(self.cells.len().saturating_sub(1))
    }

    /// `((height × z + y) × width + x)`, the original's index, for a cell on the map.
    pub(crate) fn index(&self, x: i32, y: i32, z: i32) -> Option<usize> {
        if !(0..self.width).contains(&x)
            || !(0..self.height).contains(&y)
            || !(0..self.levels).contains(&z)
        {
            return None;
        }
        usize::try_from((self.height * z + y) * self.width + x).ok()
    }

    /// The phase oracle's dump of the cells after `step` with the generator's placed objects
    /// `placed` (format 1, `xdd.c` section 10): "H3RMGCEL", then little-endian 32-bit words: the
    /// format 1, the step, the width, height and levels; per cell 12 words (the objects on it and
    /// the first three's indices among `placed`, -1 none or not there, then +0x10 to +0x2C); the
    /// objects' count and six words each (its template's index in the part's object table, the
    /// template's class and subclass, x, y, z); the count of the places past a cell's third and
    /// three words each (the cell's index, the place from 3, the object's index), cell by cell.
    pub fn dump(&self, step: u8, placed: &[Object]) -> Vec<u8> {
        let mut out = Vec::with_capacity(40 + 48 * self.cells.len() + 24 * placed.len());
        let word = |out: &mut Vec<u8>, w: u32| out.extend_from_slice(&w.to_le_bytes());
        out.extend_from_slice(b"H3RMGCEL");
        for w in [1, i32::from(step), self.width, self.height, self.levels] {
            word(&mut out, w.cast_unsigned());
        }
        // An index among `placed`, -1 when it is not there.
        let found = |&i: &u32| placed.get(i as usize).map_or(u32::MAX, |_| i);
        let mut further = Vec::new();
        for (k, (c, on)) in self.cells.iter().zip(&self.objects).enumerate() {
            word(&mut out, on.len() as u32);
            for j in 0..3 {
                word(&mut out, on.get(j).map_or(u32::MAX, found));
            }
            for w in c.previous {
                word(&mut out, w.cast_unsigned());
            }
            for w in [c.movement, c.zone, c.tile, c.tile_data, c.pending_guard] {
                word(&mut out, w);
            }
            for (j, i) in on.iter().enumerate().skip(3) {
                further.push([k as u32, j as u32, found(i)]);
            }
        }
        word(&mut out, placed.len() as u32);
        for o in placed {
            for w in [o.template as u32, o.class, o.subclass] {
                word(&mut out, w);
            }
            for w in o.position {
                word(&mut out, w.cast_unsigned());
            }
        }
        word(&mut out, further.len() as u32);
        for w in further.into_iter().flatten() {
            word(&mut out, w);
        }
        out
    }
}
