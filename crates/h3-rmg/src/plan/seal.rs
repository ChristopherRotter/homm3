//! Sealing water shores (`rmg_water_shores_seal` 0x53FCB0, step 6's sixth part; survey
//! `docs/rules/surveys/E14-generator.md` §2.3, "Sealing water shores" and "Step 6's plan as
//! ported (T4c)"): water a zone's cost flood reached from a zone its water zone has no connection
//! to is made obstacles with its shore and painted to that shore's land, level by level.

use alloc::vec::Vec;

use super::{PlanInput, around, connection_lists, pending};
use crate::cells::{Cells, FREE, KEPT, OBSTACLES};
use crate::generate::{Stream, Trace};
use crate::painter::{PaintFault, Painter, ROCK, WATER};

/// The cells of the 5 × 5 around (x, y) clamped to a map of `w` × `h`, row by row.
fn around_5x5(w: i32, h: i32, x: i32, y: i32) -> impl Iterator<Item = (i32, i32)> {
    let (x0, x1) = (x.wrapping_sub(2).max(0), x.wrapping_add(3).min(w));
    let (y0, y1) = (y.wrapping_sub(2).max(0), y.wrapping_add(3).min(h));
    (y0..y1).flat_map(move |v| (x0..x1).map(move |u| (u, v)))
}

/// `rmg_water_shores_seal` 0x53FCB0, level by level, cell by cell (row, column): a water cell of
/// a zone (its zone byte not negative) that a zone F's cost flood entered (+0x20 bits 24-31 not
/// negative, any zone's), whose 3 × 3 clamped to the map holds a land cell neither water nor rock,
/// passable (bit 25) and without bit 26 (the first row by row gives the land L), is sealed when
/// its zone's list (+0xC4: its zone generator's by the zone byte, RMG-QUIRK-4; a byte past them
/// is left) names F as no connection's far end: every cell of its 3 × 3 takes bit 26 and loses
/// 27 unless +0x2C bit 0, its water cells listed with L in that order; every cell of its 5 × 5
/// clamped to the map holding no object and without +0x2C bit 0 loses bit 27. A seal's marks
/// are read by the later cells' tests. Then, when the level listed a cell, a terrain painter
/// of the first L on the level gives each listed cell a rectangle of one cell in the list's
/// order (a cell listed twice twice), flushed to the next L before a cell of another
/// ([`Painter::flush`]), and is finished. The marks all come before the painter: nothing else
/// writes the map in its life.
///
/// # Errors
///
/// The first terrain painter's fault met (RMG-QUIRK-3, -4), the seal run to its end.
// rule: RMG-CON-012
pub(crate) fn seal<T: Trace>(
    stream: &mut Stream<'_, T>,
    input: &PlanInput<'_>,
    cells: &mut Cells,
) -> Result<(), PaintFault> {
    let lists = connection_lists(input);
    let (w, h, levels) = cells.size();
    let mut fault = None;
    for z in 0..levels {
        // The water cells to paint and their land, in the order listed.
        let mut listed: Vec<([i32; 2], u8)> = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let c = cells.clamped(x, y, z);
                let (own, from) = (c.zone_index(), c.flooded_by());
                if own < 0 || c.land() != u32::from(WATER) || from < 0 {
                    continue;
                }
                let shore = around(w, h, x, y)
                    .map(|(u, v)| cells.clamped(u, v, z))
                    .find(|n| {
                        let land = n.land();
                        land != u32::from(WATER)
                            && land != u32::from(ROCK)
                            && n.tile_data & OBSTACLES == 0
                            && n.tile_data & FREE != 0
                    });
                let Some(land) = shore.map(|n| n.land() as u8) else {
                    continue;
                };
                let unconnected = input
                    .gens
                    .get(own.cast_unsigned() as usize)
                    .and_then(|g| lists.get(g.zone))
                    .is_some_and(|list| list.iter().all(|&(far, _)| far != from));
                if !unconnected {
                    continue;
                }
                for (u, v) in around(w, h, x, y) {
                    let n = cells.clamped_mut(u, v, z);
                    if !pending(n.pending_guard) {
                        n.tile_data = (n.tile_data & !KEPT) | OBSTACLES;
                    }
                    if n.land() == u32::from(WATER) {
                        listed.push(([u, v], land));
                    }
                }
                for (u, v) in around_5x5(w, h, x, y) {
                    let empty = cells.objects_at(u, v, z).is_empty();
                    let n = cells.clamped_mut(u, v, z);
                    if empty && !pending(n.pending_guard) {
                        n.tile_data &= !KEPT;
                    }
                }
            }
        }
        let Some(&(_, first)) = listed.first() else {
            continue;
        };
        let mut painter = Painter::new(input.pictures, cells, stream, z, first, input.cosmetic);
        let mut terrain = first;
        for ([x, y], land) in listed {
            if land != terrain {
                painter.flush(land);
                terrain = land;
            }
            painter.add_rect(x, y, 1, 1);
        }
        fault = fault.or(painter.finish().err());
    }
    fault.map_or(Ok(()), Err)
}

#[cfg(test)]
mod tests;
