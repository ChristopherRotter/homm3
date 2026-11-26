//! Step 2, each level partitioned (`rmg_level_zones_partition` 0x53E050; survey
//! `docs/rules/surveys/E14-generator.md` §2.2 and §3.2.4, RMG-ZON-009 to -014): the level's zone
//! centres and step 2's points (a water zone on level 0, a point without zone on level 1) into
//! the triangulation, each zone's Voronoi cell drawn on the map, each zone's free region filled,
//! then the water zones connected.

use alloc::vec::Vec;

use h3_kernel::num::Real;

use crate::cells::Cells;
use crate::generate::{GenerateError, Stream, Trace};
use crate::layout::{ZoneGenerator, decimal, town_type_drawn, trunc_sqrt};
use crate::part::{Connection, GeneratorTables, Strength, Zone, ZoneKind};
use crate::settings::Water;
use crate::template::KeptConnection;
use crate::triangulation::{Edge, Triangulation, Vertex};

/// The call site of a jagged border's displacement draw (`rmg_jagged_line_draw`,
/// `rmg_border_jitter_draw`).
pub const SITE_BORDER_JITTER: u32 = 0x0053_C0F2;

/// Water's terrain number.
const WATER_TERRAIN: i32 = 8;
/// A water zone's two treasure bands (`rmg_level_zones_partition`): 100 to 1000 at density 5,
/// 2000 to 6000 at density 1.
const WATER_TREASURE: [[i32; 3]; 3] = [[100, 1000, 5], [2000, 6000, 1], [0, 0, 0]];
/// The distances' value for a zone not reached (`rmg_water_zones_connect`).
pub const UNREACHED: i16 = 32_000;

/// Why step 2 stops where the original divides by zero (RMG-QUIRK-3), or reads or writes outside
/// its arrays or never ends (RMG-QUIRK-4); none reached by the corpus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionFault {
    /// A point on or outside the triangulation's frame (-200 to 400).
    Frame,
    /// A walk through the triangulation that does not end.
    Walk,
    /// A flat triangle's circumcentre.
    Flat,
    /// A jagged border's limit of 0 (`rand() % 0`): a zone of scaled size 0.
    Limit,
    /// A straight line through a cell off the map (its unchecked index: another row's or
    /// level's cell, or outside the cells), or a fill's clipped start off it.
    Outside,
}

/// Which of step 2's zones `rmg_zone_generator_init` reads: the temporary zone of a level that
/// tests positions (on the stack) or a water zone (from the heap).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryZone {
    /// The temporary zone (`rmg_level_zones_partition`'s call at 0x53E149).
    Temporary,
    /// A water zone (its call at 0x53E45C).
    Water,
}

/// What a step 2 zone's unwritten bytes hold, in the form fresh starts agree on (RMG-QUIRK-1;
/// `docs/lanes/rmg/ORACLE.md`, "The memory record"): whether each is 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZoneMemory {
    /// +0x41 to +0x49 not 0, bit k for town type k: `rand() %` their count draws the zone's town
    /// type, the r-th of them.
    pub town_types: u16,
    /// +0x40 not 0 (towns of the same type).
    pub same_type: bool,
    /// +0x84 not 0 (the terrain matches the town).
    pub terrain_match: bool,
    /// +0x94 not 0 (the monsters match the town).
    pub monster_match: bool,
}

impl ZoneMemory {
    /// Zeroed bytes (`strict-fixed`): no town type, no draw.
    pub const ZEROED: Self = Self {
        town_types: 0,
        same_type: false,
        terrain_match: false,
        monster_match: false,
    };

    /// The oracle model's temporary zone (RMG-QUIRK-1, provisional until increment 3's
    /// checkpoint): the stack's bytes, 5 to 7 town types set in each of the corpus's 29 measured
    /// (0BB the most common; its town type is never read, a temporary zone being of type 3, so
    /// only the draw counts), +0x40, +0x84 and +0x94 set.
    pub const TEMPORARY: Self = Self {
        town_types: 0x0BB,
        same_type: true,
        terrain_match: true,
        monster_match: true,
    };

    /// The oracle model's water zone (provisional): the heap's bytes, all nine town types set in
    /// 36 of the corpus's 44 (none set in 2), +0x40, +0x84 and +0x94 set in 43.
    pub const WATER: Self = Self {
        town_types: 0x1FF,
        same_type: true,
        terrain_match: true,
        monster_match: true,
    };
}

/// What step 2 reads besides the stream, the template and the map.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PartitionInput<'a> {
    pub levels: i32,
    pub water: Water,
    pub tables: &'a GeneratorTables,
    /// `quirk.rmg.zone_memory_model` (RMG-QUIRK-1): the unwritten bytes as the oracle model
    /// holds them, else zeroed.
    pub memory_model: bool,
    /// `quirk.rmg.partition_units` (RMG-QUIRK-14): the other zones' base sizes in template units
    /// against a map distance, else their scaled sizes.
    pub partition_units: bool,
}

/// What step 2 leaves beside the cells, the template and the zone generators.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Partition {
    /// Per zone generator its Voronoi cell's points on the map (+0x3F4, `rmg_zone_borders_draw`).
    pub polygons: Vec<Vec<[i32; 2]>>,
    /// Per zone generator, per zone below the last level's count, the connections from that
    /// zone (+0x3E4; [`UNREACHED`] none).
    pub distances: Vec<Vec<i16>>,
}

/// One level's work: the stream, the input, the template's zones and connections, the zone
/// generators, the map, the output.
struct Level<'s, 'a, 't, T: Trace> {
    stream: &'s mut Stream<'t, T>,
    input: &'s PartitionInput<'a>,
    zones: &'s mut Vec<Zone>,
    connections: &'s mut Vec<KeptConnection>,
    gens: &'s mut Vec<ZoneGenerator>,
    cells: &'s mut Cells,
    out: &'s mut Partition,
    z: i32,
    tri: Triangulation,
}

/// `rmg_segment_clip` 0x53CAC0: the first point of the line from p toward q in
/// [0, W) × [0, H): p inside gives p; else p moved onto the left, top, right (W − 1) and bottom
/// (H − 1) lines in that order, each when beyond it and its divisor not 0 (32-bit products,
/// divisions toward zero), a move that leaves the other axis' range from inside it a miss (p);
/// the result may still lie outside.
// rule: RMG-ZON-012
pub(crate) fn clip(p: [i32; 2], q: [i32; 2], w: i32, h: i32) -> [i32; 2] {
    let inside = |s: [i32; 2]| (0..w).contains(&s[0]) && (0..h).contains(&s[1]);
    if inside(p) {
        return p;
    }
    let d = [q[0].wrapping_sub(p[0]), q[1].wrapping_sub(p[1])];
    let mut s = p;
    // (axis, beyond, the line): left, top, right, bottom.
    for (axis, line) in [
        (0, 0),
        (1, 0),
        (0, w.wrapping_sub(1)),
        (1, h.wrapping_sub(1)),
    ] {
        let beyond = if line == 0 {
            s[axis] < 0
        } else {
            s[axis] > line
        };
        if !beyond || d[axis] == 0 {
            continue;
        }
        let k = line.wrapping_sub(s[axis]);
        let other = 1 - axis;
        s[axis] = s[axis].wrapping_add(d[axis].wrapping_mul(k).wrapping_div(d[axis]));
        s[other] = s[other].wrapping_add(d[other].wrapping_mul(k).wrapping_div(d[axis]));
        let size = if other == 0 { w } else { h };
        if (p[other] >= 0 && s[other] < 0) || (p[other] < size && s[other] >= size) {
            return p;
        }
    }
    s
}

impl<T: Trace> Level<'_, '_, '_, T> {
    fn side(&self) -> i32 {
        self.cells.size().0
    }

    /// Step 2's land bit: not on the islands' level 0.
    fn land(&self) -> bool {
        self.z == 1 || self.input.water != Water::Islands
    }

    fn fault(&self, fault: PartitionFault) -> GenerateError {
        GenerateError::Partition {
            level: self.z,
            fault,
        }
    }

    fn tri<R>(&self, r: Result<R, PartitionFault>) -> Result<R, GenerateError> {
        r.map_err(|f| self.fault(f))
    }

    fn memory(&mut self, kind: MemoryZone, index: usize) -> ZoneMemory {
        if !self.input.memory_model {
            return ZoneMemory::ZEROED;
        }
        self.stream.zone_memory(kind, index).unwrap_or(match kind {
            MemoryZone::Temporary => ZoneMemory::TEMPORARY,
            MemoryZone::Water => ZoneMemory::WATER,
        })
    }

    /// `rmg_zone_position_fits` 0x53AD60 for the temporary zone of base `s` at `p` on this
    /// level (type 3, id -1: no start zone's test, no zone of its id): every zone generator on
    /// the level `10 × trunc(dist) ≥ 8 × (its base + s)`, its base its template zone's
    /// (RMG-QUIRK-14, template units for step 1's zones) or, without the flag, its scaled size.
    // rule: RMG-ZON-010
    fn fits(&self, p: [i32; 2], s: i32) -> bool {
        self.gens.iter().all(|o| {
            if o.position[2] != self.z {
                return true;
            }
            let base = if self.input.partition_units {
                self.zones[o.zone].base_size
            } else {
                o.size
            };
            let dx = o.position[0].wrapping_sub(p[0]);
            let dy = o.position[1].wrapping_sub(p[1]);
            let t = trunc_sqrt(dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy)));
            t.wrapping_mul(10) >= base.wrapping_add(s).wrapping_mul(8)
        })
    }

    /// The points around the level's zones present at the start (8 directions each) and, on
    /// level 0, a water zone at each one kept; each into the triangulation.
    // rule: RMG-ZON-010, RMG-ZON-011
    fn points(&mut self, n: usize, waters: &mut usize) -> Result<(), GenerateError> {
        // The temporary zone's constructor draws first (its town type is never read).
        let temporary = self.memory(MemoryZone::Temporary, *waters);
        *waters += 1;
        town_type_drawn(self.stream, temporary.town_types);
        let tables = self.input.tables;
        let table = |v: &Vec<i32>, k: usize| decimal(v.get(k).copied().unwrap_or(0));
        let side = Real::from_i32(self.side());
        for k in 0..n {
            let g = self.gens[k];
            if g.position[2] != self.z {
                continue;
            }
            let [x, y, _] = g.position;
            let s = g.size;
            for d in 0..8 {
                let t = 4 * d;
                let at = |c: Real, o: i32| {
                    c.checked_add(c)
                        .and_then(|v| v.checked_add(Real::from_i32(o)))
                        .and_then(Real::trunc_to_i64)
                        .map_or(0, |v| v as i32)
                };
                let fs = Real::from_i32(s);
                let c = fs
                    .checked_mul(table(&tables.layout_cos, t))
                    .unwrap_or(Real::ZERO);
                let sn = fs
                    .checked_mul(table(&tables.layout_sin, t))
                    .unwrap_or(Real::ZERO);
                let p = [at(c, x), at(sn, y)];
                // Off the map, refused when its half-way point is off too (its f64 compares).
                let off = |v: i32, half: Real| {
                    let fv = Real::from_i32(v);
                    (v < 0 && fv < half)
                        || (v >= self.side() && side.checked_add(half).is_ok_and(|e| e <= fv))
                };
                if off(p[0], c) || off(p[1], sn) || !self.fits(p, s) {
                    continue;
                }
                let data = if self.z == 0 {
                    let memory = self.memory(MemoryZone::Water, *waters);
                    *waters += 1;
                    let id = self.zones.len();
                    self.zones.push(water_zone(id, s, memory));
                    let town_type = town_type_drawn(self.stream, memory.town_types);
                    self.gens.push(ZoneGenerator {
                        zone: id,
                        town_type,
                        // +0x8 is not written (RMG-QUIRK-1: the heap's).
                        town_type_2: -1,
                        terrain: WATER_TERRAIN,
                        position: [p[0], p[1], 0],
                        size: s,
                    });
                    Some(self.gens.len() - 1)
                } else {
                    None
                };
                let r = self.tri.insert(Vertex {
                    x: p[0],
                    y: p[1],
                    data,
                });
                self.tri(r)?;
            }
        }
        Ok(())
    }

    /// The cell a straight line writes at (x, y). The original indexes the cells unchecked:
    /// off the map another row's or level's cell, or memory past them (RMG-QUIRK-4: refused;
    /// never reached by the corpus).
    fn line_cell(&mut self, x: i32, y: i32) -> Result<&mut crate::cells::Cell, GenerateError> {
        let fault = self.fault(PartitionFault::Outside);
        let z = self.z;
        self.cells.get_mut(x, y, z).ok_or(fault)
    }

    /// `rmg_line_draw` 0x53C220: a Bresenham line from its end of lower x (the first on ties),
    /// each cell marked with `index` and the land bit, the far end with the index only.
    // rule: RMG-ZON-012
    fn line(&mut self, a: [i32; 2], b: [i32; 2], index: u8) -> Result<(), GenerateError> {
        let (mut p, e) = if a[0] > b[0] { (b, a) } else { (a, b) };
        let dx = e[0].wrapping_sub(p[0]);
        let dy = e[1].wrapping_sub(p[1]);
        let ady = dy.wrapping_abs();
        let sy = if dy > 0 { 1 } else { -1 };
        // (major, minor, the normal step, the overflow step).
        let (major, minor, step) = if dx > ady {
            (dx, ady, [1, 0])
        } else {
            (ady, dx, [0, sy])
        };
        let land = self.land();
        let mut err = major / 2;
        while p != e {
            self.line_cell(p[0], p[1])?.mark(index, land);
            err = err.wrapping_add(minor);
            if err < major {
                p = [p[0].wrapping_add(step[0]), p[1].wrapping_add(step[1])];
            } else {
                err = err.wrapping_sub(major);
                p = [p[0].wrapping_add(1), p[1].wrapping_add(sy)];
            }
        }
        self.line_cell(p[0], p[1])?.mark(index, false);
        Ok(())
    }

    /// `rmg_jagged_line_draw` 0x53BFF0: a stack of ends from b; per popped end its midpoint
    /// `(a + e + 1) / 2`; at an end the current point is marked (clamped to the map) and the walk
    /// moves on to the popped end; else the midpoint moves along the normal by
    /// `(rand() % k − k / 2) × normal / L` (L the truncated length above 1, k = min(limit, L))
    /// and the end and the midpoint are pushed. The last end is not marked.
    // rule: RMG-ZON-012
    fn jagged(
        &mut self,
        a: [i32; 2],
        b: [i32; 2],
        index: u8,
        limit: i32,
    ) -> Result<(), GenerateError> {
        let (w, h, _) = self.cells.size();
        let land = self.land();
        let mut current = a;
        let mut stack = alloc::vec![b];
        while let Some(end) = stack.pop() {
            let half = |u: i32, v: i32| u.wrapping_add(v).wrapping_add(1) / 2;
            let mut mid = [half(end[0], current[0]), half(end[1], current[1])];
            if mid == current || mid == end {
                let x = current[0].max(0).min(w.wrapping_sub(1));
                let y = current[1].max(0).min(h.wrapping_sub(1));
                if let Some(c) = self.cells.get_mut(x, y, self.z) {
                    c.mark(index, land);
                }
                current = end;
                continue;
            }
            let dy = end[1].wrapping_sub(current[1]);
            let dx = end[0].wrapping_sub(current[0]);
            let l = trunc_sqrt(dy.wrapping_mul(dy).wrapping_add(dx.wrapping_mul(dx)));
            if l > 1 {
                let k = limit.min(l);
                if k == 0 {
                    return Err(self.fault(PartitionFault::Limit));
                }
                // `rand()` is at most 0x7FFF; `idiv`'s remainder has the dividend's sign.
                let r = (self.stream.draw(SITE_BORDER_JITTER) as i32).wrapping_rem(k);
                let shift = r.wrapping_sub(k / 2);
                mid[0] = mid[0].wrapping_add(shift.wrapping_mul(dy.wrapping_neg()) / l);
                mid[1] = mid[1].wrapping_add(dx.wrapping_mul(shift) / l);
            }
            stack.push(end);
            stack.push(mid);
        }
        Ok(())
    }

    /// The zone byte of zone generator `g`'s zone (its template zone's id).
    fn id(&self, g: usize) -> i32 {
        self.zones[self.gens[g].zone].id
    }

    /// `rmg_zone_borders_draw` 0x53C390 for the record `r` at a zone's centre: the record's zone
    /// G (not the caller's), the sides of its Voronoi cell in Onext order from the first whose
    /// clip from its start lies on the map and differs from its clip from its end; each side's
    /// clipped start appended to G's polygon and the side drawn in G's id when its neighbour is
    /// none or of a higher id (jagged with the flag, its limit the smaller size, G's without a
    /// neighbour); where the cell leaves the map, straight lines along the map's edge corner by
    /// corner to where a later side re-enters (at once when the two share x or y); no side on the
    /// map, the whole frame.
    // rule: RMG-ZON-012
    fn borders(&mut self, r: Edge, jagged: bool) -> Result<(), GenerateError> {
        // The record's data: a vertex without a zone has no cell to draw (none as read).
        let Some(g) = self.tri.origin(r).data else {
            return Ok(());
        };
        let id = self.id(g);
        let index = id as u8;
        let (w, h, _) = self.cells.size();
        let inside = |p: [i32; 2]| (0..w).contains(&p[0]) && (0..h).contains(&p[1]);
        let centre = |t: &Triangulation, e: Edge| t.centre(e);
        let side = |t: &Triangulation, e: Edge| {
            let n = t.onext(e);
            (
                clip(centre(t, e), centre(t, n), w, h),
                clip(centre(t, n), centre(t, e), w, h),
            )
        };
        let mut first = None;
        let mut e = r;
        loop {
            let (a, b) = side(&self.tri, e);
            if inside(a) && a != b {
                first = Some(e);
                break;
            }
            e = self.tri.onext(e);
            if e == r {
                break;
            }
        }
        let Some(f) = first else {
            let corners = [[w - 1, h - 1], [w - 1, 0], [0, 0], [0, h - 1]];
            for k in 0..4 {
                self.line(corners[k], corners[(k + 1) % 4], index)?;
            }
            self.out.polygons[g].extend_from_slice(&corners);
            return Ok(());
        };
        let size = self.gens[g].size;
        let mut e = f;
        loop {
            let n = self.tri.onext(e);
            let neighbour = self.tri.destination(n).data;
            let (a, b) = side(&self.tri, e);
            self.out.polygons[g].push(a);
            if neighbour.is_none_or(|o| self.id(o) > id) {
                let limit = neighbour.map_or(size, |o| self.gens[o].size.min(size));
                if jagged {
                    self.jagged(a, b, index, limit)?;
                } else {
                    self.line(a, b, index)?;
                }
            }
            let mut next = n;
            if b != centre(&self.tri, n) {
                // The cell leaves the map at b: the next side re-entering it.
                let mut q = b;
                let mut t = next;
                let c = loop {
                    let u = self.tri.onext(t);
                    let c = clip(centre(&self.tri, t), centre(&self.tri, u), w, h);
                    if inside(c) {
                        break c;
                    }
                    t = u;
                };
                next = t;
                // At most four corners: c lies on the map's edge.
                for _ in 0..4 {
                    if q[0] == c[0] || q[1] == c[1] {
                        break;
                    }
                    let corner = if q[0] == 0 && q[1] != 0 {
                        [0, 0]
                    } else if q[1] == 0 && q[0] != w - 1 {
                        [w - 1, 0]
                    } else if q[0] == w - 1 && q[1] != h - 1 {
                        [w - 1, h - 1]
                    } else {
                        [0, h - 1]
                    };
                    self.line(q, corner, index)?;
                    self.out.polygons[g].push(q);
                    q = corner;
                }
                self.line(q, c, index)?;
                self.out.polygons[g].push(q);
            }
            e = next;
            if e == f {
                return Ok(());
            }
        }
    }

    /// `rmg_zone_flood_fill` 0x53D380 for zone generator `g` and the record `r` at its centre:
    /// from the centre on the map, else the clip from it toward the destination in r's ring
    /// (r.Onext first, r last) deepest in [1, W − 2] × [1, H − 2] (the first on ties; none, no
    /// fill), the 4-connected free region (zone byte 0xFF) of that cell, or of its left
    /// neighbour when it is not free: zone byte g's id, the land bit unless on the islands'
    /// level 0.
    ///
    /// # Errors
    ///
    /// [`PartitionFault::Outside`]: a clipped start off the map.
    // rule: RMG-ZON-013
    fn fill(&mut self, g: usize, r: Edge) -> Result<(), GenerateError> {
        let (w, h, _) = self.cells.size();
        let [x, y, _] = self.gens[g].position;
        let start = if (0..w).contains(&x) && (0..h).contains(&y) {
            Some([x, y])
        } else {
            // The ring's destinations themselves scored, the first deepest one clipped toward.
            let mut best = (0, None);
            let mut e = r;
            loop {
                e = self.tri.onext(e);
                let d = self.tri.destination(e);
                if (1..w - 1).contains(&d.x) && (1..h - 1).contains(&d.y) {
                    let depth = d.x.min(w - 1 - d.x).min(d.y).min(h - 1 - d.y);
                    if depth > best.0 {
                        best = (depth, Some([d.x, d.y]));
                    }
                }
                if e == r {
                    break;
                }
            }
            best.1.map(|v| clip([x, y], v, w, h))
        };
        let Some(mut start) = start else {
            return Ok(());
        };
        // The original uses the clip's result untested (RMG-QUIRK-4): off the level it would
        // read and write outside it.
        if !((0..w).contains(&start[0]) && (0..h).contains(&start[1])) {
            return Err(self.fault(PartitionFault::Outside));
        }
        let z = self.z;
        let free =
            |c: &Cells, x: i32, y: i32| c.get(x, y, z).is_some_and(|c| c.zone_byte() == 0xFF);
        if !free(self.cells, start[0], start[1]) {
            start[0] -= 1;
            if !free(self.cells, start[0], start[1]) {
                return Ok(());
            }
        }
        let index = self.id(g) as u8;
        let land = self.land();
        let mut seeds = alloc::vec![start];
        while let Some([sx, sy]) = seeds.pop() {
            if !free(self.cells, sx, sy) {
                continue;
            }
            let mut x0 = sx;
            while free(self.cells, x0 - 1, sy) {
                x0 -= 1;
            }
            let mut x = x0;
            let mut above = false;
            let mut below = false;
            while free(self.cells, x, sy) {
                if let Some(c) = self.cells.get_mut(x, sy, z) {
                    c.mark(index, land);
                }
                for (row, open) in [(sy - 1, &mut above), (sy + 1, &mut below)] {
                    let f = free(self.cells, x, row);
                    if f && !*open {
                        seeds.push([x, row]);
                    }
                    *open = f;
                }
                x += 1;
            }
        }
        Ok(())
    }

    /// The first record of `g`'s ring (from its centre's record's Onext) whose destination is
    /// zone generator `o`'s vertex.
    fn ring_to(&self, r: Edge, o: usize) -> Option<Edge> {
        let mut e = r;
        loop {
            e = self.tri.onext(e);
            if self.tri.destination(e).data == Some(o) {
                return Some(e);
            }
            if e == r {
                return None;
            }
        }
    }

    /// A connection both ways, `a`'s first.
    fn connect(&mut self, a: usize, b: usize, created: bool) {
        let zones = [self.gens[a].zone, self.gens[b].zone];
        self.connections.push(KeptConnection {
            zones,
            row: Connection {
                zones: zones.map(|z| self.zones[z].id),
                value: 0,
                wide: true,
                border_guard: false,
                humans: [0, 0],
                players: [0, 0],
            },
            created,
        });
    }

    /// Per zone generator its hop counts from each of the first `n` zones over the template's
    /// connections ([`UNREACHED`] none): `rmg_zone_graph_distances` 0x53D8E0's values, which
    /// are the counts whatever its order.
    fn distances(&mut self, n: usize) {
        // The original takes a connection's zone generator by its zone's id: one index.
        let count = self.gens.len();
        let mut links = alloc::vec![Vec::new(); count];
        for c in self.connections.iter() {
            let [a, b] = c.zones;
            if a < count && b < count {
                links[a].push(b);
                links[b].push(a);
            }
        }
        let mut out = alloc::vec![alloc::vec![UNREACHED; n]; count];
        for column in 0..n.min(count) {
            let mut d = alloc::vec![UNREACHED; count];
            d[column] = 0;
            let mut queue = alloc::collections::VecDeque::from([column]);
            while let Some(k) = queue.pop_front() {
                let next = d[k].wrapping_add(1);
                for &j in &links[k] {
                    if d[j] > next {
                        d[j] = next;
                        queue.push_back(j);
                    }
                }
            }
            for (row, v) in out.iter_mut().zip(d) {
                row[column] = v;
            }
        }
        self.out.distances = out;
    }

    /// `rmg_water_zones_connect` 0x53DAD0 with `n` the zone generators before this level's step:
    /// the new zones that are triangulation neighbours connected when their side's clip lies on
    /// the map (value 0, wide, created); the distances; then each new zone to each zone below
    /// `n` on the level among its neighbours, in order, unless that brings the zone nearer to
    /// another (wide, not created: step 6 makes the way).
    // rule: RMG-ZON-014
    fn connect_water(&mut self, n: usize) -> Result<(), GenerateError> {
        let (w, h, _) = self.cells.size();
        let count = self.gens.len();
        let z = self.z;
        for i in n..count {
            let [x, y, _] = self.gens[i].position;
            let r = self.tri.find([x, y]);
            let r = self.tri(r)?;
            for j in i + 1..count {
                if self.gens[j].position[2] != z {
                    continue;
                }
                let Some(e) = self.ring_to(r, j) else {
                    continue;
                };
                let p = clip(self.tri.centre(e), self.tri.centre(self.tri.oprev(e)), w, h);
                if (0..w).contains(&p[0]) && (0..h).contains(&p[1]) {
                    self.connect(i, j, true);
                }
            }
        }
        self.distances(n);
        for wz in n..count {
            let [x, y, _] = self.gens[wz].position;
            let r = self.tri.find([x, y]);
            let r = self.tri(r)?;
            for zz in 0..n {
                if self.gens[zz].position[2] != z || self.ring_to(r, zz).is_none() {
                    continue;
                }
                let zid = self.id(zz);
                let nearer = (0..n).any(|c| {
                    usize::try_from(zid) != Ok(c)
                        && self.out.distances[zz][c] > self.out.distances[wz][c].wrapping_add(1)
                });
                if nearer {
                    continue;
                }
                self.connect(wz, zz, false);
                self.distances(n);
            }
        }
        Ok(())
    }
}

/// A water zone of step 2 (`rmg_level_zones_partition`): type 3, base `s`, Ownership 0, its two
/// treasure bands, no towns, mines, terrains or monsters; the bytes nobody writes as `memory`
/// gives them (RMG-QUIRK-1; its player limits are not read after the template is kept).
// rule: RMG-ZON-011
fn water_zone(id: usize, s: i32, memory: ZoneMemory) -> Zone {
    Zone {
        id: i32::try_from(id).unwrap_or(i32::MAX),
        kind: ZoneKind::Junction,
        base_size: s,
        humans: [0, 0],
        players: [0, 0],
        ownership: -1,
        player_towns: [0; 4],
        neutral_towns: [0; 4],
        same_type: memory.same_type,
        town_types: memory.town_types,
        mines: [0; 7],
        mine_density: [0; 7],
        terrain_match: memory.terrain_match,
        terrains: 0,
        strength: Strength::None,
        monster_match: memory.monster_match,
        neutral_monsters: false,
        monster_towns: 0,
        treasure: WATER_TREASURE,
    }
}

/// Step 2 (`rmg_level_zones_partition` 0x53E050 per level): per level the zone centres into a
/// triangulation; on level 1, and on level 0 with water, the points around the zones present at
/// the start, a water zone at each on level 0; the borders, the fills, the water connections.
///
/// # Errors
///
/// [`GenerateError::Partition`]: RMG-QUIRK-3's and -4's step 2 cases.
// rule: RMG-ZON-009, RMG-ZON-010, RMG-ZON-012, RMG-ZON-013
pub(crate) fn partition<T: Trace>(
    stream: &mut Stream<'_, T>,
    input: &PartitionInput<'_>,
    zones: &mut Vec<Zone>,
    connections: &mut Vec<KeptConnection>,
    gens: &mut Vec<ZoneGenerator>,
    cells: &mut Cells,
) -> Result<Partition, GenerateError> {
    let mut out = Partition::default();
    let mut waters = 0;
    for z in 0..input.levels {
        let mut level = Level {
            stream: &mut *stream,
            input,
            zones: &mut *zones,
            connections: &mut *connections,
            gens: &mut *gens,
            cells: &mut *cells,
            out: &mut out,
            z,
            tri: Triangulation::new(),
        };
        for k in 0..level.gens.len() {
            let [x, y, gz] = level.gens[k].position;
            if gz == z {
                let r = level.tri.insert(Vertex {
                    x,
                    y,
                    data: Some(k),
                });
                level.tri(r)?;
            }
        }
        let n = level.gens.len();
        if z == 1 || input.water != Water::None {
            level.points(n, &mut waters)?;
        }
        let r = level.tri.finish();
        level.tri(r)?;
        level.out.polygons.resize(level.gens.len(), Vec::new());
        for k in 0..level.gens.len() {
            let [x, y, gz] = level.gens[k].position;
            if gz != z {
                continue;
            }
            let r = level.tri.find([x, y]);
            let r = level.tri(r)?;
            let jagged = k < n && (input.water != Water::Islands || z == 1);
            level.borders(r, jagged)?;
        }
        for k in 0..level.gens.len() {
            let [x, y, gz] = level.gens[k].position;
            if gz != z {
                continue;
            }
            let r = level.tri.find([x, y]);
            let r = level.tri(r)?;
            level.fill(k, r)?;
        }
        level.connect_water(n)?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests;
