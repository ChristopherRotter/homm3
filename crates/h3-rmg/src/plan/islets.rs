//! Step 6's islets (`rmg_water_zone_islets` 0x53F470 per zone generator; survey
//! `docs/rules/surveys/E14-generator.md` §2.3, "The islets", "The height map as read" and "Step
//! 6's plan as ported (T4c)"): in a water zone, while a cell lies far enough from the zone's edge,
//! a small island of land painted there by the terrain painter and made obstacles.

use alloc::vec;
use alloc::vec::Vec;

use super::{PlanInput, pending};
use crate::cells::{Cells, KEPT, OBSTACLES};
use crate::generate::{Stream, Trace};
use crate::objects::NEIGHBOURS;
use crate::painter::{PaintFault, Painter, WATER};

/// The call site of an islet's centre draw (`rmg_water_zone_islets`, `rmg_islet_centre_draw`):
/// `rand() %` the cells 20 or more from the zone's edge (unsigned), per islet.
pub const SITE_ISLET_CENTRE: u32 = 0x0053_F745;
/// An islet's half-size draw (`rmg_islet_radius_draw`): `rand() % (d / 3 − 5)`, per islet.
pub const SITE_ISLET_RADIUS: u32 = 0x0053_F794;
/// An islet's terrain draw (`rmg_islet_draw`, `rmg_islet_terrain_draw`): `rand() % 6`.
pub const SITE_ISLET_TERRAIN: u32 = 0x0053_EFE4;
/// An islet height map's five displacements per rectangle split (`rmg_islet_heightmap`,
/// `rmg_islet_height_draw_1` to `_5`): the top, left, bottom and right edge midpoints, the
/// centre, `rand() % a` each.
pub const SITE_ISLET_HEIGHTS: [u32; 5] = [
    0x0053_EEFF,
    0x0053_EF0C,
    0x0053_EF19,
    0x0053_EF2E,
    0x0053_EF43,
];

/// The distance a cell must lie from its zone's edge to centre an islet (+0x1C's high half).
const FAR: u32 = 20;

/// `rmg_zone_distance_flood` 0x53F1A0 from `seed` over the cells of zone `zone` (their zone
/// byte; the seed's own unchecked): the seed entered at cost 0, then least cost first (a vector
/// kept sorted from the highest, the last taken; a cell entered again is listed again, and a
/// listed cell expands at its cost when taken) every [`NEIGHBOURS`] step on the map into a cell
/// of the zone, 2 orthogonal and 3 diagonal, enters it when that is below its +0x1C high half
/// ([`crate::cells::Cell::enter_far`]: the cost, the step, +0x20 bits 24-31 cleared). An entry
/// goes before those of its cost: they are taken first.
// rule: RMG-CON-007
pub(crate) fn distance_flood(cells: &mut Cells, [x, y, z]: [i32; 3], zone: i32) {
    cells.clamped_mut(x, y, z).enter_far(0, 0);
    // (cell, cost), the least last.
    let mut listed: Vec<([i32; 2], u32)> = vec![([x, y], 0)];
    while let Some(([x, y], _)) = listed.pop() {
        let cost = cells.clamped(x, y, z).far_cost();
        for (step, [dx, dy]) in NEIGHBOURS.into_iter().enumerate() {
            let (u, v) = (x.wrapping_add(dx), y.wrapping_add(dy));
            let Some(c) = cells.get_mut(u, v, z) else {
                continue;
            };
            let next = cost + if step % 2 == 1 { 3 } else { 2 };
            if c.zone_index() != zone || next >= c.far_cost() {
                continue;
            }
            c.enter_far(next, step as u32);
            let at = listed.partition_point(|&(_, c)| c > next);
            listed.insert(at, ([u, v], next));
        }
    }
}

/// One record of `rmg_islet_heightmap` (`rmg_heightmap_rect`, 0x24 bytes): rows `r0..r1` and
/// columns `c0..c1` of the mask, the heights at its corners top-left, top-right, bottom-left,
/// bottom-right, the amplitude.
#[derive(Debug, Clone, Copy)]
struct Rect {
    rows: [i32; 2],
    columns: [i32; 2],
    corners: [i32; 4],
    amplitude: i32,
}

/// `rmg_heightmap_quadrants_push` 0x53E9E0: `rect` cut at its middle row (r0 + r1) / 2 and
/// column into quadrants with the heights at their own corners (the record's, the edge midpoints
/// `edges` left, top, right, bottom, the centre), appended bottom-right, bottom-left, top-right,
/// top-left, a quadrant of no row or no column skipped.
// rule: RMG-CON-008
fn quadrants(stack: &mut Vec<Rect>, rect: Rect, [left, top, right, bottom]: [i32; 4], centre: i32) {
    let [r0, r1] = rect.rows;
    let [c0, c1] = rect.columns;
    let [tl, tr, bl, br] = rect.corners;
    let (rm, cm) = ((r0 + r1) / 2, (c0 + c1) / 2);
    for (rows, columns, corners) in [
        ([rm, r1], [cm, c1], [centre, right, bottom, br]),
        ([rm, r1], [c0, cm], [left, centre, bl, bottom]),
        ([r0, rm], [cm, c1], [top, tr, centre, right]),
        ([r0, rm], [c0, cm], [tl, top, left, centre]),
    ] {
        if rows[0] != rows[1] && columns[0] != columns[1] {
            stack.push(Rect {
                rows,
                columns,
                corners,
                amplitude: rect.amplitude,
            });
        }
    }
}

/// `rmg_islet_heightmap` 0x53ED00: a midpoint-displacement height map of `h` rows by `w`
/// columns, row-major. The whole is split at once (corners and edge midpoints 0, the centre
/// a / 2, its quadrants' amplitude a = (h + w) / 4 + 1, no draw); then last in, first out, a
/// record of one cell writes its top-left height clamped to 0..=255, a larger one its edges'
/// means top, left, bottom, right and the centre's (of the four corners), each displaced while
/// a > 1 by `rand() % a − a / 2` in that order ([`SITE_ISLET_HEIGHTS`]), split with amplitude
/// (a − 1) / 2 + 1. Every record lies inside the mask and every cell gets one of one cell: the
/// original's bounds tests never fail and its uninitialised mask is all written.
// rule: RMG-CON-008
fn heightmap<T: Trace>(stream: &mut Stream<'_, T>, w: i32, h: i32) -> Vec<u8> {
    let mut mask = vec![0u8; (w * h) as usize];
    let a = (h + w) / 4 + 1;
    let whole = Rect {
        rows: [0, h],
        columns: [0, w],
        corners: [0; 4],
        amplitude: a,
    };
    let mut stack = Vec::new();
    quadrants(&mut stack, whole, [0; 4], a / 2);
    while let Some(rect) = stack.pop() {
        let ([r0, r1], [c0, c1]) = (rect.rows, rect.columns);
        let [tl, tr, bl, br] = rect.corners;
        if r1 == r0 + 1 && c1 == c0 + 1 {
            mask[(r0 * w + c0) as usize] = tl.clamp(0, 255) as u8;
            continue;
        }
        let mut heights = [
            (tr + tl) / 2,
            (bl + tl) / 2,
            (br + bl) / 2,
            (br + tr) / 2,
            (br + bl + tr + tl) / 4,
        ];
        let a = rect.amplitude;
        if a > 1 {
            for (height, site) in heights.iter_mut().zip(SITE_ISLET_HEIGHTS) {
                // `rand()` is at most 0x7FFF: the signed remainder is the unsigned one.
                *height += (stream.draw(site) as i32) % a - a / 2;
            }
        }
        let [top, left, bottom, right, centre] = heights;
        let child = Rect {
            amplitude: (a - 1) / 2 + 1,
            ..rect
        };
        quadrants(&mut stack, child, [left, top, right, bottom], centre);
    }
    mask
}

/// `rmg_islet_draw` 0x53EFA0 over the box [x0, x1) × [y0, y1) of level `z`: a terrain
/// `rand() % 6` (dirt to rough, [`SITE_ISLET_TERRAIN`]), the box's [`heightmap`], a painter of
/// the terrain given each cell of a height above 0 row by row (one cell a rectangle) and
/// finished; then every cell of the box not water, unless its +0x2C bit 0, takes bit 26 and
/// loses 27 (an islet is obstacles only; a cell the painter made land outside the box is left).
// rule: RMG-CON-008
fn islet<T: Trace>(
    stream: &mut Stream<'_, T>,
    input: &PlanInput<'_>,
    cells: &mut Cells,
    [x0, y0, x1, y1]: [i32; 4],
    z: i32,
) -> Result<(), PaintFault> {
    let (w, h) = (x1 - x0, y1 - y0);
    // `rand()` is at most 0x7FFF: the signed remainder is the unsigned one.
    let terrain = (stream.draw(SITE_ISLET_TERRAIN) % 6) as u8;
    let mask = heightmap(stream, w, h);
    let mut painter = Painter::new(input.pictures, cells, stream, z, terrain, input.cosmetic);
    for y in y0..y1 {
        for x in x0..x1 {
            if mask[((y - y0) * w + x - x0) as usize] > 0 {
                painter.add_rect(x, y, 1, 1);
            }
        }
    }
    let painted = painter.finish();
    for y in y0..y1 {
        for x in x0..x1 {
            let c = cells.clamped_mut(x, y, z);
            if c.land() != u32::from(WATER) && !pending(c.pending_guard) {
                c.tile_data = (c.tile_data & !KEPT) | OBSTACLES;
            }
        }
    }
    painted
}

/// `rmg_water_zone_islets` 0x53F470 for every zone generator of water (terrain 8), in order,
/// with its zone's id and box [x0, x1) × [y0, y1) on its level: the box's cells entered at cost
/// 32000 ([`crate::cells::Cell::enter_far`], step 0); every cell of the box widened by one and
/// clamped to the map whose zone byte is not the id a seed of [`distance_flood`], row by row;
/// then, while the box's cells in columns 3..W − 4 and rows 3..H − 4 (row by row) hold some at
/// [`FAR`] or more, one drawn ([`SITE_ISLET_CENTRE`]) at (x, y) with its distance d, a half-size
/// r = 3 + `rand() % (d / 3 − 5)` at most 6 ([`SITE_ISLET_RADIUS`]; d ≥ 20 makes the modulus
/// 1 or more), its [`islet`] over [x − r, x + r) × [y − r, y + r) clamped to the map, and a
/// flood from (x, y). The candidates are any cells of the box, the zone's or not (RMG-QUIRK
/// candidate: harmless, the others are seeds at 0). The first painter fault met is returned
/// after the last zone.
// rule: RMG-CON-007
pub(crate) fn islets<T: Trace>(
    stream: &mut Stream<'_, T>,
    input: &PlanInput<'_>,
    cells: &mut Cells,
) -> Result<(), PaintFault> {
    let (w, h, _) = cells.size();
    let mut fault = None;
    for g in input.gens {
        if g.terrain != i32::from(WATER) {
            continue;
        }
        let id = input.zones.get(g.zone).map_or(-1, |zone| zone.id);
        let [x0, y0, x1, y1] = g.bounds;
        let z = g.position[2];
        for y in y0..y1 {
            for x in x0..x1 {
                cells.clamped_mut(x, y, z).enter_far(32_000, 0);
            }
        }
        for y in y0.wrapping_sub(1).max(0)..h.min(y1.wrapping_add(1)) {
            for x in x0.wrapping_sub(1).max(0)..w.min(x1.wrapping_add(1)) {
                if cells.clamped(x, y, z).zone_index() != id {
                    distance_flood(cells, [x, y, z], id);
                }
            }
        }
        let (columns, rows) = (x0.max(3)..(w - 4).min(x1), y0.max(3)..(h - 4).min(y1));
        loop {
            let far: Vec<[i32; 2]> = rows
                .clone()
                .flat_map(|y| columns.clone().map(move |x| [x, y]))
                .filter(|&[x, y]| cells.clamped(x, y, z).far_cost() >= FAR)
                .collect();
            if far.is_empty() {
                break;
            }
            let [x, y] = far[(stream.draw(SITE_ISLET_CENTRE) % far.len() as u32) as usize];
            let d = cells.clamped(x, y, z).far_cost();
            // `rand()` is at most 0x7FFF: the signed remainder is the unsigned one.
            let r = (stream.draw(SITE_ISLET_RADIUS) % (d / 3 - 5) + 3).min(6) as i32;
            let rect = [(x - r).max(0), (y - r).max(0), w.min(x + r), h.min(y + r)];
            let painted = islet(stream, input, cells, rect, z);
            fault = fault.or(painted.err());
            distance_flood(cells, [x, y, z], id);
        }
    }
    fault.map_or(Ok(()), Err)
}

#[cfg(test)]
mod tests;
