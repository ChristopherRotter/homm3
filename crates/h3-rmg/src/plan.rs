//! Step 6's obstacle plan (`rmg_paths_and_connections` 0x544920 up to its connections; survey
//! `docs/rules/surveys/E14-generator.md` §2.3, "Step 6's plan as ported (T4c)"): which cells
//! take obstacles (+0x28 bit 26, filled by step 13) and which stay free (bit 27), in the
//! original's order: the lanes (`rmg_free_lanes_draw` 0x543E20), the zone walls
//! (`rmg_zone_borders_close` 0x53F880), the land cells of no zone made obstacles, the islets
//! ([`islets`]), the zones' free cells joined ([`join`]), the water shores sealed ([`seal`]); then
//! the connections ([`connect`], E14 T4d1: by land, by a shipyard, by gates, by monoliths, the
//! guard monsters; the Border Guards T4d2's), which end step 6.
//!
//! A cell with +0x2C bit 0 (a pending Border Guard) keeps its bits 26 and 27 through every mark
//! here but the carve's, which places the guard first (not ported: none is pending before the
//! Border Guards, RMG-STUB-1).

use alloc::collections::VecDeque;
use alloc::vec;
use alloc::vec::Vec;

use crate::cells::{Cells, ENTRANCE, FREE, KEPT, OBSTACLES};
use crate::generate::{GenerateError, Stream, Trace};
use crate::guards::GuardInput;
use crate::layout::{ZoneGenerator, trunc_sqrt};
use crate::objects::Objects;
use crate::painter::{Pictures, ROCK, WATER};
use crate::part::Zone;
use crate::template::KeptConnection;

mod connect;
mod islets;
/// The joining and its cost flood ([`join::cost_flood`]: step 7's junction paths and the
/// connections flood with it).
pub(crate) mod join;
mod seal;

pub use connect::{
    Connected, SITE_GATE_PROTOTYPE, SITE_GATE_SITE, SITE_PASSAGE_CELL, SITE_PASSAGE_GUARD_SIDE,
    SITE_SHIPYARD_PROTOTYPE, SITE_SHIPYARD_SITE, SITE_ZONE_OBJECT_CELL,
};
pub use islets::{SITE_ISLET_CENTRE, SITE_ISLET_HEIGHTS, SITE_ISLET_RADIUS, SITE_ISLET_TERRAIN};

/// The call site of a lane's orientation draw (`rmg_free_lanes_draw`,
/// `rmg_lane_orientation_draw`): `rand() % 4` per level.
pub const SITE_LANE_ORIENTATION: u32 = 0x0054_3EB8;

/// The call site of a lane segment's displacement draw (`rmg_free_lanes_draw`,
/// `rmg_lane_jitter_draw`): `rand() % L` per segment split.
pub const SITE_LANE_JITTER: u32 = 0x0054_403E;

/// What step 6's plan reads besides the stream and the map, and the objects it places.
#[derive(Debug)]
pub(crate) struct PlanInput<'a> {
    /// The kept template's zones and connections (step 2's water zones and connections
    /// appended).
    pub zones: &'a [Zone],
    pub connections: &'a [KeptConnection],
    /// The zone generators.
    pub gens: &'a [ZoneGenerator],
    /// The placed objects and their classes' object settings (the cost floods); the
    /// connections place theirs.
    pub objects: &'a mut Objects,
    /// The generator's +0x14B0 (steps 4 and 5's town entrances): the shipyards' entrances are
    /// appended.
    pub road_targets: &'a mut Vec<[i32; 3]>,
    /// The terrain painter's picture sets (the islets', the shores' seal).
    pub pictures: &'a Pictures,
    /// `quirk.rmg.cosmetic` (RMG-QUIRK-21's painter's half).
    pub cosmetic: bool,
    /// What the guard monsters read.
    pub guards: GuardInput<'a>,
    /// `quirk.rmg.shipyard_sites_repeated` (RMG-QUIRK-19): a shipyard site enters the list once
    /// per land cell it is found from (on), or once (off).
    pub shipyard_sites_repeated: bool,
    /// `quirk.rmg.underground_wide_walled` (RMG-QUIRK-9): on level 1 a zone is walled along a
    /// wide connection too (on), or as on level 0 (off).
    pub underground_wide_walled: bool,
    /// `quirk.rmg.monolith_channels_shared` (RMG-QUIRK-26): a one-way monolith connection is one
    /// channel, its entrances leading to both exits (on), or two where the next is free, each
    /// entrance leading to the other zone's exit (off).
    pub monolith_channels_shared: bool,
}

/// +0x2C bit 0: a pending Border Guard, whose cell keeps its bits 26 and 27.
const fn pending(guard: u32) -> bool {
    guard & 1 != 0
}

/// The cells of the 3 × 3 around (x, y) clamped to a map of `w` × `h`, row by row.
fn around(w: i32, h: i32, x: i32, y: i32) -> impl Iterator<Item = (i32, i32)> {
    let (x0, x1) = (x.wrapping_sub(1).max(0), x.wrapping_add(2).min(w));
    let (y0, y1) = (y.wrapping_sub(1).max(0), y.wrapping_add(2).min(h));
    (y0..y1).flat_map(move |v| (x0..x1).map(move |u| (u, v)))
}

/// `rmg_map_mark_free_3x3` 0x531AD0 on the cell (x, y, z) of the map: it and the 3 × 3 around it
/// clamped to the map take bit 27 and lose bit 26, each unless its +0x2C bit 0 (the cell itself
/// is written first, then again as one of the 3 × 3).
// rule: RMG-CON-003
fn mark_free(cells: &mut Cells, x: i32, y: i32, z: i32) {
    let (w, h, _) = cells.size();
    for (u, v) in around(w, h, x, y) {
        let c = cells.clamped_mut(u, v, z);
        if !pending(c.pending_guard) {
            c.tile_data = (c.tile_data & !OBSTACLES) | KEPT;
        }
    }
}

/// `rmg_map_mark_obstacle` 0x531BD0 on the cell (x, y, z) of the map: unless its +0x2C bit 0 it
/// takes bit 26 and loses bit 27; in the 3 × 3 around it clamped to the map (itself among them)
/// every cell not an entrance (bit 22), passable (bit 25), neither rock nor water and without
/// +0x2C bit 0 loses bit 27.
// rule: RMG-CON-004, RMG-CON-006
fn mark_obstacle(cells: &mut Cells, x: i32, y: i32, z: i32) {
    let (w, h, _) = cells.size();
    let c = cells.clamped_mut(x, y, z);
    if !pending(c.pending_guard) {
        c.tile_data = (c.tile_data & !KEPT) | OBSTACLES;
    }
    for (u, v) in around(w, h, x, y) {
        let c = cells.clamped_mut(u, v, z);
        let land = c.land();
        if c.tile_data & ENTRANCE == 0
            && c.tile_data & FREE != 0
            && land != u32::from(ROCK)
            && land != u32::from(WATER)
            && !pending(c.pending_guard)
        {
            c.tile_data &= !KEPT;
        }
    }
}

/// `rmg_lane_ray_cast` 0x543C70 on level `z`: a Bresenham walk from `from` toward and past `to`
/// (its major axis the larger of |dx| and |dy|, |dy| on ties; the error from half the major,
/// a diagonal step when it reaches the major; each axis's sign + when its delta is above 0,
/// else −), returning the last point before the walk leaves the map's interior (columns 1 to
/// W − 2, rows 1 to H − 2) or, from its third step, the point before one whose 3 × 3 holds a
/// cell with bit 27. The first step's leaving returns `from`.
// rule: RMG-CON-004
fn cast(cells: &Cells, from: [i32; 2], to: [i32; 2], z: i32) -> [i32; 2] {
    let (w, h, _) = cells.size();
    let dx = to[0].wrapping_sub(from[0]);
    let dy = to[1].wrapping_sub(from[1]);
    let (adx, ady) = (dx.wrapping_abs(), dy.wrapping_abs());
    let sign = |d: i32| if d > 0 { 1 } else { -1 };
    let diagonal = [sign(dx), sign(dy)];
    let (major, minor, straight) = if adx > ady {
        (adx, ady, [diagonal[0], 0])
    } else {
        (ady, adx, [0, diagonal[1]])
    };
    let mut err = major / 2;
    let mut p = from;
    let mut steps = 0u32;
    loop {
        let previous = p;
        err = err.wrapping_add(minor);
        steps = steps.saturating_add(1);
        let step = if err < major {
            straight
        } else {
            err = err.wrapping_sub(major);
            diagonal
        };
        p = [p[0].wrapping_add(step[0]), p[1].wrapping_add(step[1])];
        let inside =
            (1..w.wrapping_sub(1)).contains(&p[0]) && (1..h.wrapping_sub(1)).contains(&p[1]);
        if !inside
            || (steps > 2
                && around(w, h, p[0], p[1])
                    .any(|(u, v)| cells.clamped(u, v, z).tile_data & KEPT != 0))
        {
            return previous;
        }
    }
}

/// One level's lane: the stack of segments (p, q) and the queue of branches (from, to).
struct Lane<'s, 't, 'c, T: Trace> {
    stream: &'s mut Stream<'t, T>,
    cells: &'c mut Cells,
    z: i32,
    segments: Vec<[[i32; 2]; 2]>,
    branches: VecDeque<[[i32; 2]; 2]>,
}

impl<T: Trace> Lane<'_, '_, '_, T> {
    fn on_map(&self, [x, y]: [i32; 2]) -> bool {
        let (w, h, _) = self.cells.size();
        (0..w).contains(&x) && (0..h).contains(&y)
    }

    /// A segment from p to q: its midpoint `((px + qx + 1) / 2, (py + qy + 1) / 2)` (truncating);
    /// one of its ends: p's 3 × 3 made free when p lies on the map. Else, with the normal
    /// n = (−(qy − py), qx − px) and L = trunc(|n|), when L > 1 the midpoint moves by
    /// (s × nx / L, s × ny / L), s = `rand() % L` − L / 2; the halves (q, mid) and (mid, p) are
    /// pushed, (mid, p) on top; when L ≥ 8 and the midpoint lies on the map, the branches
    /// (mid, mid + n) and (mid, mid − n) are queued.
    // rule: RMG-CON-003
    fn split(&mut self, p: [i32; 2], q: [i32; 2]) {
        let half = |a: i32, b: i32| a.wrapping_add(b).wrapping_add(1) / 2;
        let mut mid = [half(q[0], p[0]), half(q[1], p[1])];
        if mid == p || mid == q {
            if self.on_map(p) {
                mark_free(self.cells, p[0], p[1], self.z);
            }
            return;
        }
        let n = [
            q[1].wrapping_sub(p[1]).wrapping_neg(),
            q[0].wrapping_sub(p[0]),
        ];
        let l = trunc_sqrt(
            n[0].wrapping_mul(n[0])
                .wrapping_add(n[1].wrapping_mul(n[1])),
        );
        if l > 1 {
            // `rand()` is at most 0x7FFF; `idiv` truncates toward zero.
            let r = (self.stream.draw(SITE_LANE_JITTER) as i32).wrapping_rem(l);
            let s = r.wrapping_sub(l / 2);
            mid[0] = mid[0].wrapping_add(s.wrapping_mul(n[0]) / l);
            mid[1] = mid[1].wrapping_add(n[1].wrapping_mul(s) / l);
        }
        self.segments.push([q, mid]);
        self.segments.push([mid, p]);
        if l >= 8 && self.on_map(mid) {
            let plus = [mid[0].wrapping_add(n[0]), mid[1].wrapping_add(n[1])];
            let minus = [mid[0].wrapping_sub(n[0]), mid[1].wrapping_sub(n[1])];
            self.branches.push_back([mid, plus]);
            self.branches.push_back([mid, minus]);
        }
    }

    /// The segments from the stack until it is empty, then the first branch queued cast
    /// ([`cast`]) and its segment (end, from) pushed when the end lies 5 cells or more from it
    /// (squared distance 25), until both are empty.
    // rule: RMG-CON-003, RMG-CON-004
    fn draw(&mut self) {
        loop {
            if let Some([p, q]) = self.segments.pop() {
                self.split(p, q);
            } else if let Some([from, to]) = self.branches.pop_front() {
                let end = cast(self.cells, from, to, self.z);
                let d = [end[0].wrapping_sub(from[0]), end[1].wrapping_sub(from[1])];
                if d[0]
                    .wrapping_mul(d[0])
                    .wrapping_add(d[1].wrapping_mul(d[1]))
                    >= 25
                {
                    self.segments.push([end, from]);
                }
            } else {
                return;
            }
        }
    }
}

/// A lane's line (p, q) on a map of `w` × `h` by its orientation `o` (0x543EB8's draw % 4):
/// 0 the diagonal (0, 0)–(W − 1, H − 1), 1 the vertical (W / 2, 0)–(W / 2, H − 1), 2 the
/// anti-diagonal (W − 1, 0)–(0, H − 1), 3 the horizontal (0, H / 2)–(W − 1, H / 2).
// rule: RMG-CON-002
fn line(o: u32, w: i32, h: i32) -> [[i32; 2]; 2] {
    let (right, bottom) = (w.wrapping_sub(1), h.wrapping_sub(1));
    match o {
        0 => [[0, 0], [right, bottom]],
        1 => [[w / 2, 0], [w / 2, bottom]],
        2 => [[right, 0], [0, bottom]],
        _ => [[0, h / 2], [right, h / 2]],
    }
}

/// `rmg_free_lanes_draw` 0x543E20: every cell holding an object takes bit 27 and loses 26, every
/// other cell the reverse (each unless its +0x2C bit 0); per level `rand() % 4` (0x543EB8) its
/// lane's [`line`], drawn as a segment ([`Lane`]); then cell by cell (level, row,
/// column) water and rock take bit 27 and lose 26 (unless +0x2C bit 0), and a cell with bit 26
/// is marked an obstacle (`rmg_map_mark_obstacle`).
// rule: RMG-CON-002, RMG-CON-004
fn lanes<T: Trace>(stream: &mut Stream<'_, T>, cells: &mut Cells) {
    let (w, h, levels) = cells.size();
    for z in 0..levels {
        for y in 0..h {
            for x in 0..w {
                let held = !cells.objects_at(x, y, z).is_empty();
                let c = cells.clamped_mut(x, y, z);
                if !pending(c.pending_guard) {
                    c.tile_data = if held {
                        (c.tile_data & !OBSTACLES) | KEPT
                    } else {
                        (c.tile_data & !KEPT) | OBSTACLES
                    };
                }
            }
        }
    }
    for z in 0..levels {
        // `rand()` is at most 0x7FFF: the signed remainder is the unsigned one.
        let line = line(stream.draw(SITE_LANE_ORIENTATION) % 4, w, h);
        Lane {
            stream: &mut *stream,
            cells: &mut *cells,
            z,
            segments: vec![line],
            branches: VecDeque::new(),
        }
        .draw();
    }
    for z in 0..levels {
        for y in 0..h {
            for x in 0..w {
                let c = cells.clamped_mut(x, y, z);
                let land = c.land();
                if (land == u32::from(WATER) || land == u32::from(ROCK))
                    && !pending(c.pending_guard)
                {
                    c.tile_data = (c.tile_data & !OBSTACLES) | KEPT;
                }
                if c.tile_data & OBSTACLES != 0 {
                    mark_obstacle(cells, x, y, z);
                }
            }
        }
    }
}

/// Each template zone's list (+0xC4), by its index: per connection touching it, in order (the
/// template's rows, step 2's appended), its far end's id and its wide flag (a connection of a
/// zone to itself twice).
fn connection_lists(input: &PlanInput<'_>) -> Vec<Vec<(i32, bool)>> {
    let id = |k: usize| input.zones.get(k).map_or(-1, |zone| zone.id);
    (0..input.zones.len())
        .map(|k| {
            input
                .connections
                .iter()
                .flat_map(|c| {
                    let [a, b] = c.zones;
                    [
                        (a == k).then(|| (id(b), c.row.wide)),
                        (b == k).then(|| (id(a), c.row.wide)),
                    ]
                })
                .flatten()
                .collect()
        })
        .collect()
}

/// `rmg_zone_borders_close` 0x53F880, cell by cell (level, row, column): a cell of a zone (its
/// zone byte not negative) and not water is walled when its 3 × 3 clamped to the map holds a
/// water cell of no zone, or a cell of another zone whose id is not the far end of any of the
/// cell's zone's connections, or is that of the first such connection (in the zone's list: the
/// template's row order, step 2's appended) when it is not wide or, RMG-QUIRK-9 on
/// (`underground_wide_walled`), the level is 1. A walled cell takes bit 26 and loses 27 (unless
/// +0x2C bit 0; an object on it or not), and every cell of its 3 × 3 without an object and
/// without +0x2C bit 0 loses bit 27. The original's second
/// pass over [y, y + 1) × [x, x + 1) marks the cell alone again when it holds no object: no
/// change, not ported. The original takes the zone generator by the zone byte unchecked
/// (+0x10E4, RMG-QUIRK-4): a byte past the zone generators, which step 2 never writes (its
/// bytes are zone generators' ids), leaves the cell unwalled.
// rule: RMG-CON-005
fn walls(input: &PlanInput<'_>, cells: &mut Cells) {
    let lists = connection_lists(input);
    let (w, h, levels) = cells.size();
    for z in 0..levels {
        for y in 0..h {
            for x in 0..w {
                let c = cells.clamped(x, y, z);
                let own = c.zone_index();
                if own < 0 || c.land() == u32::from(WATER) {
                    continue;
                }
                let list = input
                    .gens
                    .get(own.cast_unsigned() as usize)
                    .and_then(|g| lists.get(g.zone));
                let Some(list) = list else {
                    continue;
                };
                let walled =
                    around(w, h, x, y).any(|(u, v)| {
                        let n = cells.clamped(u, v, z);
                        let other = n.zone_index();
                        if other < 0 {
                            return n.land() == u32::from(WATER);
                        }
                        other != own
                            && list.iter().find(|&&(far, _)| far == other).is_none_or(
                                |&(_, wide)| !wide || (z == 1 && input.underground_wide_walled),
                            )
                    });
                if !walled {
                    continue;
                }
                let c = cells.clamped_mut(x, y, z);
                if !pending(c.pending_guard) {
                    c.tile_data = (c.tile_data & !KEPT) | OBSTACLES;
                }
                for (u, v) in around(w, h, x, y) {
                    let empty = cells.objects_at(u, v, z).is_empty();
                    let n = cells.clamped_mut(u, v, z);
                    if empty && !pending(n.pending_guard) {
                        n.tile_data &= !KEPT;
                    }
                }
            }
        }
    }
}

/// `rmg_paths_and_connections`' third part, cell by cell (level, row, column): a cell without
/// bit 26, passable (bit 25), not rock, not an entrance (bit 22), holding no object, of no zone
/// (its zone byte negative) and not water is marked an obstacle (`rmg_map_mark_obstacle`).
// rule: RMG-CON-006
fn no_zone(cells: &mut Cells) {
    let (w, h, levels) = cells.size();
    for z in 0..levels {
        for y in 0..h {
            for x in 0..w {
                let c = cells.clamped(x, y, z);
                let land = c.land();
                if c.tile_data & OBSTACLES == 0
                    && c.tile_data & FREE != 0
                    && land != u32::from(ROCK)
                    && c.tile_data & ENTRANCE == 0
                    && cells.objects_at(x, y, z).is_empty()
                    && c.zone_index() < 0
                    && land != u32::from(WATER)
                {
                    mark_obstacle(cells, x, y, z);
                }
            }
        }
    }
}

/// Step 6 (`rmg_paths_and_connections` 0x544920), in its order: the lanes, the zone walls, the
/// land cells of no zone made obstacles, the islets, the zones' free cells joined, the water
/// shores sealed; then the connections, which end step 6.
///
/// # Errors
///
/// [`GenerateError::Paint`]: the first terrain painter's fault met (RMG-QUIRK-3, -4): an islet's
/// ends the plan after the joining, a seal's at its end; [`GenerateError::NoEmptyCell`]: a zone
/// generator whose box holds no empty cell of its zone before any had one (RMG-QUIRK-5), which
/// ends it at the joining (after an islet's fault, that one); [`GenerateError::Objects`]: the
/// connections' first object fault ([`Objects::take_fault`]), at their end.
// rule: RMG-CON-001
pub(crate) fn plan<T: Trace>(
    stream: &mut Stream<'_, T>,
    input: &mut PlanInput<'_>,
    cells: &mut Cells,
) -> Result<Connected, GenerateError> {
    let paint = |fault| GenerateError::Paint { step: 6, fault };
    // (1) The lanes (RMG-CON-002 to -004).
    lanes(stream, cells);
    // (2) The zone walls (RMG-CON-005).
    walls(input, cells);
    // (3) The land cells of no zone (RMG-CON-006).
    no_zone(cells);
    // (4) The islets (RMG-CON-007, -008): `rmg_water_zone_islets` 0x53F470 per zone generator,
    // in order.
    let painted = islets::islets(stream, input, cells).map_err(paint);
    // (5) The free cells joined (RMG-CON-009 to -011): `rmg_zone_paths_connect` 0x5405D0.
    let joined =
        join::join(input, cells).map_err(|zone| GenerateError::NoEmptyCell { step: 6, zone });
    painted.and(joined)?;
    // (6) The water shores sealed (RMG-CON-012): `rmg_water_shores_seal` 0x53FCB0; (7) after
    // them the connections (RMG-CON-013 on): `rmg_zone_connections_make` 0x543240.
    seal::seal(stream, input, cells)
        .map_err(paint)
        .and_then(|()| {
            let connected = connect::connect(stream, input, cells);
            match input.objects.take_fault() {
                Some(fault) => Err(GenerateError::Objects { step: 6, fault }),
                None => Ok(connected),
            }
        })
}

#[cfg(test)]
mod tests;
