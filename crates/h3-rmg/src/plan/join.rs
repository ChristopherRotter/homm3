//! Joining each zone's free cells (`rmg_zone_paths_connect` 0x5405D0, step 6's fifth part and
//! step 9, also `rmg_monolith_border_guard` 0x542B00's; survey `docs/rules/surveys/E14-generator.md`
//! §2.3, "Joining each zone's free cells" and "Step 6's plan as ported (T4c)"): every cell's
//! floods reset; per zone generator a seed, its cost flood ([`cost_flood`], also step 7's and
//! the connections' input), then a path carved ([`carve`]) from every free cell the flood left
//! unjoined and a flood from there.

use alloc::vec;
use alloc::vec::Vec;

use super::{PlanInput, around, pending};
use crate::cells::{Cells, ENTRANCE, FREE, KEPT, OBSTACLES, STEP};
use crate::objects::{NEIGHBOURS, Objects};
use crate::painter::{ROCK, WATER};

/// A carve's start must cost less (+0x1C bits 0-15): a cell its zone's flood never reached is
/// left (32000).
const REACHED: u32 = 30_000;

/// `rmg_zone_cost_flood` 0x531460 on the map from `seed` (`water`: the zone generator's terrain
/// is water), least cost first as `rmg_zone_distance_flood` (a vector kept sorted from the
/// highest, a new entry before its equals, the last taken and expanded at the cell's current
/// cost: in its zone's half when of the seed's zone, else the other). The seed, of zone Z (its
/// zone byte, unchecked), takes +0x1C bits 0-15 0 and +0x10 to +0x18 -1. A taken cell steps
/// [`NEIGHBOURS`] from the last to the first (NE, N, NW, W, SW, S, SE, E), only the first five
/// counted down from W (W, SW, S, SE, E) when it is an entrance (bit 22) whose first object's
/// class has object settings byte +1 clear, onto a cell of the level on the map of a zone (its
/// zone byte not negative), passable (bit 25), not rock, and when an entrance not one whose first
/// object's settings have +0 set and +2 clear, nor one with +1 clear entered by SE, S or SW. Into
/// another zone than Z, from a cell of Z or of that zone: cost + 10 when below its +0x1C bits
/// 16-31 ([`crate::cells::Cell::enter_from`]: the cost, the step back (k + 4) mod 8, Z in +0x20
/// bits 24-31). Into Z, from a cell of Z: cost + 1 (+ 10 onto water) when below its bits 0-15,
/// then 0 from a cell of cost 0 onto a kept-free cell (bit 27) not water unless `water`; the
/// cost written and the cell taken from in +0x10 to +0x18.
// rule: RMG-CON-010
pub(crate) fn cost_flood(cells: &mut Cells, objects: &Objects, [x, y, z]: [i32; 3], water: bool) {
    let seed = cells.clamped_mut(x, y, z);
    let zone = seed.zone_index();
    seed.movement &= 0xFFFF_0000;
    seed.previous = [-1; 3];
    // (cell, cost), the least last.
    let mut listed: Vec<([i32; 2], u32)> = vec![([x, y], 0)];
    while let Some(([x, y], _)) = listed.pop() {
        let c = *cells.clamped(x, y, z);
        let own = c.zone_index();
        let cost = if own == zone {
            c.near_cost()
        } else {
            c.far_cost()
        };
        let settings = objects.first_settings(cells, [x, y, z]);
        let ways = if c.tile_data & ENTRANCE != 0 && !settings.is_some_and(|s| s.from_above) {
            5
        } else {
            8
        };
        for k in (0..ways).rev() {
            let [dx, dy] = NEIGHBOURS[k];
            let (u, v) = (x.wrapping_add(dx), y.wrapping_add(dy));
            let Some(&n) = cells.get(u, v, z) else {
                continue;
            };
            let other = n.zone_index();
            if other < 0 || n.tile_data & FREE == 0 || n.land() == u32::from(ROCK) {
                continue;
            }
            if n.tile_data & ENTRANCE != 0 {
                let settings = objects.first_settings(cells, [u, v, z]);
                if settings.is_some_and(|s| s.not_entered && !s.removable)
                    || (!settings.is_some_and(|s| s.from_above) && (1..4).contains(&k))
                {
                    continue;
                }
            }
            let next = if other != zone {
                let next = cost + 10;
                if (own != zone && own != other) || n.far_cost() <= next {
                    continue;
                }
                cells
                    .clamped_mut(u, v, z)
                    .enter_from(next, (k as u32 + 4) & 7, zone);
                next
            } else {
                let onto = n.land();
                let mut next = cost + if onto == u32::from(WATER) { 10 } else { 1 };
                if own != zone || n.near_cost() <= next {
                    continue;
                }
                if cost == 0 && n.tile_data & KEPT != 0 && (onto != u32::from(WATER) || water) {
                    next = 0;
                }
                let m = cells.clamped_mut(u, v, z);
                m.movement = (m.movement & 0xFFFF_0000) | next;
                m.previous = [x, y, z];
                next
            };
            let at = listed.partition_point(|&(_, c)| c > next);
            listed.insert(at, ([u, v], next));
        }
    }
}

/// `rmg_path_carve` 0x5408E0 from the cell `from` with its flag clear (the joining's): when its
/// cost (+0x1C bits 0-15) is 1 to 29999, back along the cells it was entered from (+0x10 to
/// +0x18) while their cost is above 0, each cell kept free (bit 27 set, 26 cleared) and the cells
/// of the start's zone in its 3 × 3 clamped to the map without +0x2C bit 0 losing bit 26. The
/// walk ends: every cost on it is below the one before, down to a cell of cost 0.
// rule: RMG-CON-011
fn carve(cells: &mut Cells, [x, y, z]: [i32; 3]) {
    let (w, h, _) = cells.size();
    let start = cells.clamped(x, y, z);
    let zone = start.zone_index();
    if !(1..REACHED).contains(&start.near_cost()) {
        return;
    }
    let mut at = [x, y, z];
    loop {
        let c = cells.clamped_mut(at[0], at[1], at[2]);
        // stub: RMG-STUB-1 — a cell with +0x2C bit 0 gets its pending Border Guard here
        // (`rmg_prototype_draw` 0x546154 of the colour, placed by the generator's slot 1, +0x2C
        // bits 0-4 cleared; E14 T4d): none is pending before step 6's connections.
        c.tile_data = (c.tile_data & !OBSTACLES) | KEPT;
        let previous = c.previous;
        for (u, v) in around(w, h, at[0], at[1]) {
            let n = cells.clamped_mut(u, v, at[2]);
            if n.zone_index() == zone && !pending(n.pending_guard) {
                n.tile_data &= !OBSTACLES;
            }
        }
        at = previous;
        if cells.clamped(at[0], at[1], at[2]).near_cost() == 0 {
            return;
        }
    }
}

/// `rmg_zone_paths_connect` 0x5405D0: every cell's floods reset (+0x1C 32000 in both halves,
/// +0x10 to +0x18 -1, +0x20 bits 24-31 -1, +0x28 bits 12-14 0); then per zone generator k in
/// order, on its level over its box row by row: the first cell of zone byte k (water only for a
/// water zone generator) holding no object that is kept free (bit 27), passable (bit 25) and not
/// rock seeds its [`cost_flood`]; with none, the last such empty cell tried is kept free (unless
/// +0x2C bit 0) and seeds it; with no empty cell at all (a water zone of step 2 without a cell:
/// `l1-4p-c`'s ninth), the original's seed is the one an earlier zone generator took, whose mark
/// and flood change nothing, so none is made (RMG-QUIRK-5, as the user decided: RMG-Q3). Then every
/// cell of the box of zone byte k, kept free, passable, neither rock nor water, whose cost is not
/// 0 (row by row, as the floods leave them) gets a [`carve`] and a flood from it.
///
/// # Errors
///
/// RMG-QUIRK-5 (exploit-fix): the index of a zone generator without an empty cell of its zone
/// before any zone generator had one, which the original seeds from the stack's bytes.
// rule: RMG-CON-009
pub(crate) fn join(input: &PlanInput<'_>, cells: &mut Cells) -> Result<(), usize> {
    let (w, h, levels) = cells.size();
    for z in 0..levels {
        for y in 0..h {
            for x in 0..w {
                let c = cells.clamped_mut(x, y, z);
                c.movement = 0x7D00_7D00;
                c.previous = [-1; 3];
                c.zone |= 0xFF00_0000;
                c.tile_data &= !STEP;
            }
        }
    }
    // A zone generator before has tried an empty cell of its zone.
    let mut seeded = false;
    for (k, g) in input.gens.iter().enumerate() {
        let id = k as i32;
        let water = g.terrain == i32::from(WATER);
        let [x0, y0, x1, y1] = g.bounds;
        let z = g.position[2];
        // The last empty cell of the zone tried, and whether it seeds as it is.
        let mut tried = None;
        'rows: for y in y0..y1 {
            for x in x0..x1 {
                let c = cells.clamped(x, y, z);
                if c.zone_index() != id
                    || (c.land() == u32::from(WATER) && !water)
                    || !cells.objects_at(x, y, z).is_empty()
                {
                    continue;
                }
                let seeds = c.tile_data & KEPT != 0
                    && c.tile_data & FREE != 0
                    && c.land() != u32::from(ROCK);
                tried = Some(([x, y], seeds));
                if seeds {
                    break 'rows;
                }
            }
        }
        // RMG-QUIRK-5: without an empty cell the original keeps the coordinates an earlier zone
        // tried last, that zone's seed (cost 0, its own zone's), whose mark and flood change
        // nothing once its zone is joined; with none before they are the stack's bytes.
        match tried {
            Some(([x, y], seeds)) => {
                if !seeds {
                    let c = cells.clamped_mut(x, y, z);
                    if !pending(c.pending_guard) {
                        c.tile_data = (c.tile_data & !OBSTACLES) | KEPT;
                    }
                }
                cost_flood(cells, input.objects, [x, y, z], water);
                seeded = true;
            }
            None if seeded => {}
            None => return Err(k),
        }
        for y in y0..y1 {
            for x in x0..x1 {
                let c = cells.clamped(x, y, z);
                let land = c.land();
                if c.zone_index() == id
                    && c.tile_data & KEPT != 0
                    && c.tile_data & FREE != 0
                    && land != u32::from(ROCK)
                    && c.near_cost() != 0
                    && land != u32::from(WATER)
                {
                    carve(cells, [x, y, z]);
                    cost_flood(cells, input.objects, [x, y, z], water);
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
