//! Step 3, the terrain painted (`rmg_terrain_paint` 0x53E6A0; survey
//! `docs/rules/surveys/E14-generator.md` §2.2, §2.7.5): the zones' boxes and centroids, with
//! islands each level-0 zone's coast and inside, then the game's terrain painter
//! ([`crate::painter`]): level 1 rock, level 0 water, each zone's land cells its terrain.

use alloc::vec::Vec;

use crate::cells::Cells;
use crate::generate::{GenerateError, Stream, Trace};
use crate::layout::{ZoneGenerator, trunc_sqrt};
use crate::painter::{PaintFault, Painter, Pictures, ROCK, WATER};
use crate::part::{GeneratorTables, Zone};
use crate::settings::Water;

/// `rmg_coast_line_draw` 0x53CD30's jitter: `rand() % min(size / 2, L / 2)`, less half.
pub const SITE_COAST_JITTER: u32 = 0x0053_CE1D;

/// +0x28 bit 28, the land bit.
const LAND: u32 = 1 << 28;

/// What step 3 meets that the original does not survive (RMG-QUIRK-3, -4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainFault {
    /// An island's coast jittered by `rand() % 0`: a zone of scaled size below 2.
    CoastLimit,
    /// An island without a polygon: the original reads its first corner from nowhere.
    NoPolygon,
    /// The terrain painter's.
    Paint(PaintFault),
}

/// What step 3 reads besides the stream and the map.
pub(crate) struct TerrainInput<'a> {
    pub water: Water,
    /// `quirk.rmg.cosmetic` (RMG-QUIRK-21's painter's half).
    pub cosmetic: bool,
    pub tables: &'a GeneratorTables,
    pub zones: &'a [Zone],
    /// Step 2's polygons, per zone generator.
    pub polygons: &'a [Vec<[i32; 2]>],
}

/// The zone byte of a cell as the signed index it is.
fn zone_index(zone: u32) -> i32 {
    i32::from((zone >> 16) as u8 as i8)
}

/// `rmg_zone_bounds_compute` 0x53BBB0: every cell of a zone (level by level, row by row) widens
/// its zone generator's box.
// rule: RMG-TER-001
fn bounds(gens: &mut [ZoneGenerator], cells: &Cells) {
    let (w, h, levels) = cells.size();
    for z in 0..levels {
        for y in 0..h {
            for x in 0..w {
                let index = cells.get(x, y, z).map_or(-1, |c| zone_index(c.zone));
                if let Some(g) = usize::try_from(index).ok().and_then(|i| gens.get_mut(i)) {
                    let b = &mut g.bounds;
                    *b = [b[0].min(x), b[1].min(y), b[2].max(x + 1), b[3].max(y + 1)];
                }
            }
        }
    }
}

/// `rmg_zone_centre_to_centroid` 0x53D0D0: the zone's centre moved to the truncated mean of its
/// box's cells holding its id, kept without any.
// rule: RMG-TER-001
fn centroid(g: &mut ZoneGenerator, id: i32, cells: &Cells) {
    let [x0, y0, x1, y1] = g.bounds;
    let z = g.position[2];
    let (mut sx, mut sy, mut n) = (0i32, 0i32, 0i32);
    for y in y0..y1 {
        for x in x0..x1 {
            if cells.get(x, y, z).is_some_and(|c| zone_index(c.zone) == id) {
                sx = sx.wrapping_add(x);
                sy = sy.wrapping_add(y);
                n += 1;
            }
        }
    }
    if n != 0 {
        g.position[0] = sx / n;
        g.position[1] = sy / n;
    }
}

/// Step 3's islands and their faults.
struct Islands<'a, 't, T: Trace> {
    stream: &'a mut Stream<'t, T>,
    cells: &'a mut Cells,
    fault: Option<TerrainFault>,
}

impl<T: Trace> Islands<'_, '_, T> {
    fn fail(&mut self, fault: TerrainFault) {
        self.fault = self.fault.or(Some(fault));
    }

    /// `rmg_coast_line_draw` 0x53CD30: `rmg_jagged_line_draw`'s walk from a to b (survey §2.2)
    /// with the jitter `rand() % k − k / 2`, k = min(limit, L / 2); a cell reached (clamped to
    /// the map) gets the land bit when its zone byte is `id`.
    // rule: RMG-TER-002
    fn coast(&mut self, a: [i32; 2], b: [i32; 2], id: i32, z: i32, limit: i32) {
        let (w, h, _) = self.cells.size();
        let mut current = a;
        let mut stack = alloc::vec![b];
        while let Some(end) = stack.pop() {
            let half = |u: i32, v: i32| u.wrapping_add(v).wrapping_add(1) / 2;
            let mut mid = [half(end[0], current[0]), half(end[1], current[1])];
            if mid == current || mid == end {
                let x = current[0].max(0).min(w - 1);
                let y = current[1].max(0).min(h - 1);
                let c = self.cells.clamped_mut(x, y, z);
                if zone_index(c.zone) == id {
                    c.tile_data |= LAND;
                }
                current = end;
                continue;
            }
            let dy = end[1].wrapping_sub(current[1]);
            let dx = end[0].wrapping_sub(current[0]);
            let l = trunc_sqrt(dy.wrapping_mul(dy).wrapping_add(dx.wrapping_mul(dx)));
            if l > 1 {
                let k = limit.min(l / 2);
                if k == 0 {
                    self.fail(TerrainFault::CoastLimit);
                    return;
                }
                // `rand()` is at most 0x7FFF; `idiv`'s remainder has the dividend's sign.
                let r = (self.stream.draw(SITE_COAST_JITTER) as i32) % k;
                let shift = r - k / 2;
                mid[0] = mid[0].wrapping_add(shift.wrapping_mul(dy.wrapping_neg()) / l);
                mid[1] = mid[1].wrapping_add(dx.wrapping_mul(shift) / l);
            }
            stack.push(end);
            stack.push(mid);
        }
    }

    /// `rmg_island_shape` 0x53D1C0: each corner of the zone's polygon moved toward its centre by
    /// min(L / 2, max(L / 4, 4)) of the distance L; the coast drawn from each moved corner, last
    /// first, to the one after it (limit the zone's size / 2); then `rmg_island_fill`.
    // rule: RMG-TER-002
    fn shape(&mut self, g: &ZoneGenerator, id: i32, polygon: &[[i32; 2]]) {
        let [cx, cy, z] = g.position;
        let moved = |p: [i32; 2]| {
            let (dx, dy) = (cx.wrapping_sub(p[0]), cy.wrapping_sub(p[1]));
            let l = trunc_sqrt(dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy)));
            if l <= 0 {
                return p;
            }
            let m = (l / 2).min((l / 4).max(4));
            [
                p[0].wrapping_add(m.wrapping_mul(dx) / l),
                p[1].wrapping_add(m.wrapping_mul(dy) / l),
            ]
        };
        let Some(&first) = polygon.first() else {
            self.fail(TerrainFault::NoPolygon);
            return;
        };
        let mut next = moved(first);
        for &corner in polygon.iter().rev() {
            let q = moved(corner);
            self.coast(q, next, id, z, g.size / 2);
            next = q;
        }
        self.fill(g, id);
    }

    /// `rmg_island_fill` 0x53CF50: from the zone's centre, a stack of cells; each popped cell's
    /// E, S, W and N neighbours on the map of the zone's id without the land bit get it and are
    /// pushed.
    // rule: RMG-TER-002
    fn fill(&mut self, g: &ZoneGenerator, id: i32) {
        let [cx, cy, z] = g.position;
        let mut stack = alloc::vec![[cx, cy]];
        while let Some([x, y]) = stack.pop() {
            for [dx, dy] in [[1, 0], [0, 1], [-1, 0], [0, -1]] {
                let (u, v) = (x + dx, y + dy);
                if let Some(c) = self.cells.get_mut(u, v, z)
                    && c.tile_data & LAND == 0
                    && zone_index(c.zone) == id
                {
                    c.tile_data |= LAND;
                    stack.push([u, v]);
                }
            }
        }
    }
}

/// Step 3 (`rmg_terrain_paint` 0x53E6A0): the boxes; per zone its centroid and, with islands
/// on level 0, its island; with two levels level 1 painted rock (one rectangle); level 0 water;
/// per zone not of water a painter of its terrain over its box's cells of its index with the
/// land bit, one cell a rectangle, row by row.
///
/// # Errors
///
/// [`GenerateError::Terrain`]: RMG-QUIRK-3's and -4's step 3 cases, the first met.
// rule: RMG-TER-003
pub(crate) fn paint<T: Trace>(
    stream: &mut Stream<'_, T>,
    input: &TerrainInput<'_>,
    gens: &mut [ZoneGenerator],
    cells: &mut Cells,
) -> Result<(), GenerateError> {
    bounds(gens, cells);
    let mut islands = Islands {
        stream: &mut *stream,
        cells: &mut *cells,
        fault: None,
    };
    for (i, g) in gens.iter_mut().enumerate() {
        let id = input.zones.get(g.zone).map_or(-1, |z| z.id);
        centroid(g, id, islands.cells);
        if input.water == Water::Islands && g.position[2] == 0 {
            let polygon = input.polygons.get(i).map_or(&[][..], Vec::as_slice);
            islands.shape(g, id, polygon);
        }
    }
    let mut fault = islands.fault;
    let pictures = Pictures::new(input.tables);
    let (w, h, levels) = cells.size();
    let mut levels_painted = alloc::vec![(0, WATER)];
    if levels > 1 {
        levels_painted.insert(0, (1, ROCK));
    }
    for (z, t) in levels_painted {
        let mut painter = Painter::new(&pictures, cells, stream, z, t, input.cosmetic);
        painter.add_rect(0, 0, w, h);
        fault = fault.or(painter.finish().err().map(TerrainFault::Paint));
    }
    for (i, g) in gens.iter().enumerate() {
        if g.terrain == i32::from(WATER) {
            continue;
        }
        let [x0, y0, x1, y1] = g.bounds;
        let z = g.position[2];
        // Painting changes neither a cell's zone byte nor its land bit.
        let land: Vec<[i32; 2]> = (y0..y1)
            .flat_map(|y| (x0..x1).map(move |x| [x, y]))
            .filter(|&[x, y]| {
                cells.get(x, y, z).is_some_and(|c| {
                    i64::from(zone_index(c.zone)) == i as i64 && c.tile_data & LAND != 0
                })
            })
            .collect();
        let mut painter =
            Painter::new(&pictures, cells, stream, z, g.terrain as u8, input.cosmetic);
        for [x, y] in land {
            painter.add_rect(x, y, 1, 1);
        }
        fault = fault.or(painter.finish().err().map(TerrainFault::Paint));
    }
    fault.map_or(Ok(()), |f| Err(GenerateError::Terrain(f)))
}

#[cfg(test)]
mod tests;
