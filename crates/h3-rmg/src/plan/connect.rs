//! Step 6's connections (`rmg_zone_connections_make` 0x543240, the plan's seventh part; survey
//! `docs/rules/surveys/E14-generator.md` §2.3, "The connections" and "The guards"): the crossing
//! cells; per land zone generator in order its connections not yet made, by land
//! (`rmg_connection_by_land` 0x541140), else by a shipyard, else, the other zone being land, by
//! gates; then per land zone generator with one left, by a shipyard, else, the other zone being
//! land, by monoliths. The shipyards (`rmg_connection_by_shipyard` 0x541AD0), the gates
//! (`rmg_connection_by_gate` 0x542080), the monoliths (`rmg_connection_by_monoliths` 0x542CE0) and
//! the Border Guards (`rmg_border_guard_place` 0x540D60) are E14 T4d2's and T4d3's
//! (RMG-STUB-1): they make nothing and draw nothing.

use alloc::vec;
use alloc::vec::Vec;

use super::PlanInput;
use super::join::carve;
use crate::cells::{Cells, FREE, KEPT, STEP};
use crate::generate::{Stream, Trace};
use crate::guards::{Monster, create, guard_value};
use crate::objects::{NEIGHBOURS, ObjectFault};
use crate::painter::{ROCK, WATER};
use crate::part::Connection;

/// The call site of a land passage's cell draw (`rmg_connection_by_land`,
/// `rmg_passage_cell_draw`): `rand() %` the least-cost crossings left, per passage.
pub const SITE_PASSAGE_CELL: u32 = 0x0054_146B;

/// The call site of a land passage's guard side draw (`rmg_connection_by_land`,
/// `rmg_passage_guard_side_draw`): `rand() & 1`, the monster at P (0) or Q (1).
pub const SITE_PASSAGE_GUARD_SIDE: u32 = 0x0054_1673;

/// +0x28 bit 24: reached by boat (`rmg_boat_reach_flood` 0x541780).
const BOAT: u32 = 1 << 24;

/// The shipyards' class (87).
const SHIPYARDS: u32 = 87;

/// The two-way monoliths' class group (45, the generator's +0x308).
const TWO_WAY: usize = 45;

/// The one-way monolith entrances' class group (43, +0x2E8).
const ONE_WAY: usize = 43;

/// `rmg_side_steps` 0x69CE00 (`rmg_side_steps_init` 0x541910): left and right of a 3-wide
/// footprint, on its row and the one below.
const SIDE_STEPS: [[i32; 2]; 4] = [[-3, 0], [1, 0], [-3, 1], [1, 1]];

/// A land passage's crossings cost at most 100.
const COSTLIEST: u32 = 100;

/// What step 6's connections write besides the cells and the objects.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Connected {
    /// Per zone generator its +0x404: the (x, y) of the passages' ends in its zone, in order
    /// (each land passage's P in its own zone, Q in the other's; step 7's junction paths start
    /// from them).
    pub passages: Vec<Vec<[i32; 2]>>,
    /// The guard monsters placed, in placing order.
    pub monsters: Vec<Monster>,
}

/// An entry of a zone's connection list (+0xC4, 0x1C bytes): the connection and its far zone,
/// both by index.
#[derive(Debug, Clone, Copy)]
struct End {
    connection: usize,
    far: usize,
}

/// Each template zone's list, by its index: per connection touching it, in order (the
/// template's rows, step 2's appended), the connection and its far end (a connection of a zone
/// to itself twice).
fn lists(input: &PlanInput<'_>) -> Vec<Vec<End>> {
    (0..input.zones.len())
        .map(|k| {
            input
                .connections
                .iter()
                .enumerate()
                .flat_map(|(connection, c)| {
                    let [a, b] = c.zones;
                    [
                        (a == k).then_some(End { connection, far: b }),
                        (b == k).then_some(End { connection, far: a }),
                    ]
                })
                .flatten()
                .collect()
        })
        .collect()
}

/// The cell a cost flood entered `p` from: p + `rmg_neighbours`[+0x28 bits 12-14].
fn back_of(cells: &Cells, [x, y, z]: [i32; 3]) -> [i32; 3] {
    let step = (cells.clamped(x, y, z).tile_data & STEP) >> 12;
    let [dx, dy] = NEIGHBOURS[step as usize];
    [x.wrapping_add(dx), y.wrapping_add(dy), z]
}

/// The crossing cells (`rmg_zone_connections_make` 0x543286 to 0x5433AE), level by level, row by
/// row: a cell a cost flood entered from another zone (+0x20 bits 24-31 not negative), neither
/// water nor rock, passable (bit 25), whose cell back ([`back_of`]; on the map: the flood's
/// own step) is not water and of another zone byte.
// rule: RMG-CON-013
fn crossings(cells: &Cells) -> Vec<[i32; 3]> {
    let (w, h, levels) = cells.size();
    let mut out = Vec::new();
    for z in 0..levels {
        for y in 0..h {
            for x in 0..w {
                let c = cells.clamped(x, y, z);
                let land = c.land();
                if c.flooded_by() < 0
                    || land == u32::from(WATER)
                    || c.tile_data & FREE == 0
                    || land == u32::from(ROCK)
                {
                    continue;
                }
                let [u, v, _] = back_of(cells, [x, y, z]);
                let q = cells.clamped(u, v, z);
                if q.land() != u32::from(WATER) && (q.zone ^ c.zone) & 0x00FF_0000 != 0 {
                    out.push([x, y, z]);
                }
            }
        }
    }
    out
}

/// Every cell of level `z` loses +0x28 bit 24 (the boat reach).
fn clear_boat(cells: &mut Cells, z: i32) {
    let (w, h, _) = cells.size();
    for y in 0..h {
        for x in 0..w {
            cells.clamped_mut(x, y, z).tile_data &= !BOAT;
        }
    }
}

/// `rmg_boat_reach_flood` 0x541780 from the cell (x, y, z) on the map: it takes +0x28 bit 24;
/// from a cell taken (the last pushed first) its four orthogonal neighbours on the map
/// (`rmg_neighbours`' even ones: E, S, W, N) without bit 24 that are kept free (bit 27) or not
/// water take it, the water ones pushed.
// rule: RMG-CON-013
fn boat_reach(cells: &mut Cells, [x, y, z]: [i32; 3]) {
    cells.clamped_mut(x, y, z).tile_data |= BOAT;
    let mut stack = vec![[x, y]];
    while let Some([x, y]) = stack.pop() {
        for [dx, dy] in NEIGHBOURS.into_iter().step_by(2) {
            let Some(c) = cells.get_mut(x.wrapping_add(dx), y.wrapping_add(dy), z) else {
                continue;
            };
            let water = c.land() == u32::from(WATER);
            if c.tile_data & BOAT != 0 || (c.tile_data & KEPT == 0 && water) {
                continue;
            }
            c.tile_data |= BOAT;
            if water {
                stack.push([x.wrapping_add(dx), y.wrapping_add(dy)]);
            }
        }
    }
}

/// `rmg_border_guard_place` 0x540D60 at `at`, its tent in zone generator `tent`'s zone, then
/// `rmg_border_guard_ring_mark` 0x540FC0: the guard value 0 once one is placed, else `value`.
fn border_guard(_at: [i32; 3], _tent: usize, value: i32) -> i32 {
    // stub: RMG-STUB-1 — E14 T4d2: the Border Guard (the next Keymaster colour, its tent at
    // random in the zone, 0x542A8F, its ring marked); places nothing and draws nothing, as every
    // flagged connection of the corpus whose tent finds no room.
    value
}

/// The connections' work.
struct Connect<'r, 'a, 't, T: Trace> {
    stream: &'r mut Stream<'t, T>,
    input: &'r mut PlanInput<'a>,
    cells: &'r mut Cells,
    /// Each template zone's list and each entry's +0xA (made).
    lists: Vec<Vec<End>>,
    made: Vec<Vec<bool>>,
    crossings: Vec<[i32; 3]>,
    out: Connected,
}

impl<'r, 'a, 't, T: Trace> Connect<'r, 'a, 't, T> {
    /// The work over `cells`: each template zone's list, its entries' made flags from the
    /// connections' `created` (both ends alike), the crossing cells ([`crossings`]), a passage
    /// list per zone generator.
    fn new(
        stream: &'r mut Stream<'t, T>,
        input: &'r mut PlanInput<'a>,
        cells: &'r mut Cells,
    ) -> Self {
        let lists = lists(input);
        let made = lists
            .iter()
            .map(|l| {
                l.iter()
                    .map(|e| input.connections[e.connection].created)
                    .collect()
            })
            .collect();
        let gens = input.gens.len();
        Self {
            stream,
            input,
            crossings: crossings(cells),
            cells,
            lists,
            made,
            out: Connected {
                passages: vec![Vec::new(); gens],
                monsters: Vec::new(),
            },
        }
    }

    /// The zone generator of template zone `zone` (+0x10E4 by its id, unchecked: the ids are
    /// the indices); none past them, the connection skipped.
    fn gen_of(&self, zone: usize) -> Option<usize> {
        usize::try_from(self.input.zones[zone].id)
            .ok()
            .filter(|&g| g < self.input.gens.len())
    }

    /// The entry the other end marks: the first of zone generator `f`'s zone's list whose far
    /// zone's id is `k` (not the same connection's when two join the same zones); none: the
    /// original writes through a null pointer (never: the lists are symmetric), nothing marked.
    fn back(&self, f: usize, k: usize) -> Option<(usize, usize)> {
        let zone = self.input.gens[f].zone;
        self.lists[zone]
            .iter()
            .position(|e| usize::try_from(self.input.zones[e.far].id) == Ok(k))
            .map(|i| (zone, i))
    }

    /// Marks the entry `i` of zone `own`'s list and the other end's `back` by `ends`.
    fn mark(&mut self, own: usize, i: usize, back: Option<(usize, usize)>, ends: [bool; 2]) {
        self.made[own][i] |= ends[0];
        if let Some((zone, j)) = back {
            self.made[zone][j] |= ends[1];
        }
    }

    /// Pass 1 (0x5433D6 to 0x5435B7): per land zone generator k in order, the boat reach cleared
    /// on its level, per entry of its zone's list not made: by land (both ends made), else by a
    /// shipyard (this end), else, the other zone generator's terrain not water, by gates (both).
    // rule: RMG-CON-013
    fn pass_1(&mut self) {
        let (gens, connections) = (self.input.gens, self.input.connections);
        for (k, g) in gens.iter().enumerate() {
            if g.terrain == i32::from(WATER) {
                continue;
            }
            clear_boat(self.cells, g.position[2]);
            let own = g.zone;
            for i in 0..self.lists[own].len() {
                let End { connection, far } = self.lists[own][i];
                if self.made[own][i] {
                    continue;
                }
                let Some(f) = self.gen_of(far) else {
                    continue;
                };
                let back = self.back(f, k);
                let row = &connections[connection].row;
                let mut ends = self.by_land(k, f, row);
                if !ends[0] {
                    ends = self.by_shipyard(k, f, row);
                }
                if !ends[0] && gens[f].terrain != i32::from(WATER) {
                    ends = self.by_gate(k, f, row);
                }
                self.mark(own, i, back, ends);
            }
        }
    }

    /// Pass 2 (0x5435BC to 0x543987): per land zone generator k with an entry not made, the
    /// boat reach cleared on its level and flooded from its shipyards ([`Connect::yards`]); per
    /// entry not made from the first: by a shipyard (this end), else, the other's terrain not
    /// water, by monoliths, both ends made whatever was placed, the monolith index i becoming
    /// (i + 1) mod (the two-way and one-way entrance prototypes' count).
    // rule: RMG-CON-013
    fn pass_2(&mut self) {
        let (gens, connections) = (self.input.gens, self.input.connections);
        let mut monolith = 0usize;
        for (k, g) in gens.iter().enumerate() {
            let own = g.zone;
            if g.terrain == i32::from(WATER) {
                continue;
            }
            let Some(first) = self.made[own].iter().position(|&m| !m) else {
                continue;
            };
            clear_boat(self.cells, g.position[2]);
            self.yards(k);
            for i in first..self.lists[own].len() {
                let End { connection, far } = self.lists[own][i];
                if self.made[own][i] {
                    continue;
                }
                let Some(f) = self.gen_of(far) else {
                    continue;
                };
                let back = self.back(f, k);
                let row = &connections[connection].row;
                let mut ends = self.by_shipyard(k, f, row);
                if !ends[0] && gens[f].terrain != i32::from(WATER) {
                    self.by_monoliths(k, f, row, monolith);
                    ends = [true, true];
                    let objects = &mut *self.input.objects;
                    let n = objects.group(TWO_WAY).len() + objects.group(ONE_WAY).len();
                    match monolith.wrapping_add(1).checked_rem(n) {
                        Some(next) => monolith = next,
                        None => objects.fail(ObjectFault::NoMonolith),
                    }
                }
                self.mark(own, i, back, ends);
            }
        }
    }

    /// Pass 2's boat reach (0x5436DD to 0x543807): per placed shipyard (class 87) whose anchor's
    /// zone byte is `k`, the first of its [`SIDE_STEPS`] cells whose column is on the map and
    /// which is water floods it ([`boat_reach`]); a side below the map's last row
    /// ([`ObjectFault::ShipyardSide`]) is skipped.
    // rule: RMG-CON-013
    fn yards(&mut self, k: usize) {
        let (w, _, _) = self.cells.size();
        let yards: Vec<[i32; 3]> = self
            .input
            .objects
            .placed()
            .iter()
            .filter(|o| o.class == SHIPYARDS)
            .map(|o| o.position)
            .collect();
        for [x, y, z] in yards {
            if usize::try_from(self.cells.clamped(x, y, z).zone_index()) != Ok(k) {
                continue;
            }
            for [dx, dy] in SIDE_STEPS {
                let (u, v) = (x.wrapping_add(dx), y.wrapping_add(dy));
                if !(0..w).contains(&u) {
                    continue;
                }
                match self.cells.get(u, v, z) {
                    None => self.input.objects.fail(ObjectFault::ShipyardSide),
                    Some(c) if c.land() == u32::from(WATER) => {
                        boat_reach(self.cells, [u, v, z]);
                        break;
                    }
                    Some(_) => {}
                }
            }
        }
    }

    /// `rmg_connection_by_land` 0x541140 between zone generators `k` and `f` for the connection
    /// `row`: refused unless on one level and both land. The crossings P of k's zone (its zone
    /// byte k's zone's id) the other's flood entered (+0x20 bits 24-31 its id), P and its cell
    /// back Q ([`back_of`]) holding no object, counted (n); the candidates those of least
    /// entering cost (+0x1C bits 16-31, at most 100; a lower one empties them), in order; none:
    /// refused. The guard value g: 0 when wide, else [`guard_value`]. A least cost of 1 with g 0
    /// and no Border Guard: made, nothing carved (never: an entering cost is at least 10). Else
    /// min(candidates, (n + 39) / 40) passages, each: P `rand() %` the candidates left (0x54146B,
    /// unsigned), the paths carved from P and from Q ([`carve`] with the Border Guard flag), P into
    /// k's +0x404 and Q into f's, P removed from the candidates; with the flag a Border Guard at
    /// P (its tent in f's zone), then at Q (in k's), each placed making g 0 for this passage and
    /// the later ones; then, g above 0, `rand() & 1` (0x541673) puts a guard monster
    /// ([`Connect::guard`]) at P (0) or Q (1). Made.
    // rule: RMG-CON-014, RMG-CON-020
    fn by_land(&mut self, k: usize, f: usize, row: &Connection) -> [bool; 2] {
        let gens = self.input.gens;
        let (g, h) = (&gens[k], &gens[f]);
        if g.position[2] != h.position[2]
            || g.terrain == i32::from(WATER)
            || h.terrain == i32::from(WATER)
        {
            return [false, false];
        }
        let (a, b) = (self.input.zones[g.zone].id, self.input.zones[h.zone].id);
        let mut least = COSTLIEST;
        let mut crossed = 0i32;
        let mut candidates: Vec<[i32; 3]> = Vec::new();
        for &[x, y, z] in &self.crossings {
            let c = self.cells.clamped(x, y, z);
            if c.zone_index() != a
                || c.flooded_by() != b
                || !self.cells.objects_at(x, y, z).is_empty()
            {
                continue;
            }
            let [u, v, _] = back_of(self.cells, [x, y, z]);
            if !self.cells.objects_at(u, v, z).is_empty() {
                continue;
            }
            crossed = crossed.wrapping_add(1);
            let cost = c.far_cost();
            if cost > least {
                continue;
            }
            if cost < least {
                candidates.clear();
                least = cost;
            }
            candidates.push([x, y, z]);
        }
        if candidates.is_empty() {
            return [false, false];
        }
        let mut value = if row.wide {
            0
        } else {
            guard_value(row.value, self.input.guards.strength)
        };
        if least == 1 && value == 0 && !row.border_guard {
            return [true, true];
        }
        let passages = candidates
            .len()
            .min(usize::try_from(crossed.wrapping_add(39) / 40).unwrap_or(0));
        for _ in 0..passages {
            // At least one candidate is left: one is removed per passage, at most their count.
            let p =
                candidates.remove(self.stream.draw(SITE_PASSAGE_CELL) as usize % candidates.len());
            let q = back_of(self.cells, p);
            carve(self.cells, p, row.border_guard);
            self.out.passages[k].push([p[0], p[1]]);
            carve(self.cells, q, row.border_guard);
            self.out.passages[f].push([q[0], q[1]]);
            if row.border_guard {
                value = border_guard(p, f, value);
                value = border_guard(q, k, value);
            }
            if value > 0 {
                if self.stream.draw(SITE_PASSAGE_GUARD_SIDE) & 1 == 0 {
                    self.guard(p, k, value);
                } else {
                    self.guard(q, f, value);
                }
            }
        }
        [true, true]
    }

    /// A guard monster worth `value` at `at` when it holds no object (0x541692 to 0x541746): for
    /// the zone generator of its zone byte (unchecked: P's is this zone generator's, Q's the
    /// other's, `owner`, which a byte past them falls back to), created
    /// ([`crate::guards::create`]) and placed through the generator's slot 1.
    fn guard(&mut self, at: [i32; 3], owner: usize, value: i32) {
        let [x, y, z] = at;
        let (gens, zones, guards) = (self.input.gens, self.input.zones, self.input.guards);
        let zone = self.cells.clamped(x, y, z).zone_index();
        let g = usize::try_from(zone)
            .ok()
            .and_then(|i| gens.get(i))
            .unwrap_or(&gens[owner]);
        if !self.cells.objects_at(x, y, z).is_empty() {
            return;
        }
        let objects = &mut *self.input.objects;
        if let Some(m) = create(
            self.stream,
            &guards,
            objects,
            value,
            &zones[g.zone],
            g.town_type,
        ) {
            let object = objects.add(self.cells, m.prototype, at);
            self.out.monsters.push(Monster {
                object,
                serial: m.serial,
                count: m.count,
            });
        }
    }

    /// `rmg_connection_by_shipyard` 0x541AD0 for zone generators `k` and `f`: which ends it made
    /// (this one only).
    fn by_shipyard(&mut self, _k: usize, _f: usize, _row: &Connection) -> [bool; 2] {
        // stub: RMG-STUB-1 — E14 T4d2: a shipyard prototype drawn (0x541B78) on one level, its
        // site (0x541D6C), the shipyard placed, a road target, the boat reach flooded; its
        // guards. Makes nothing, draws nothing.
        [false, false]
    }

    /// `rmg_connection_by_gate` 0x542080 for zone generators `k` and `f` (another level, their
    /// boxes overlapping): which ends it made (both).
    fn by_gate(&mut self, _k: usize, _f: usize, _row: &Connection) -> [bool; 2] {
        // stub: RMG-STUB-1 — E14 T4d2: a gate prototype drawn (0x5421BC), the best position of
        // the overlap (0x542387), a gate on each level; their guards. Makes nothing, draws
        // nothing.
        [false, false]
    }

    /// `rmg_connection_by_monoliths` 0x542CE0 for zone generators `k` and `f` with the monolith
    /// index `index`: both ends are made whatever it places.
    fn by_monoliths(&mut self, _k: usize, _f: usize, _row: &Connection, _index: usize) {
        // stub: RMG-STUB-1 — E14 T4d3: the index's two-way or one-way prototypes, an entrance at
        // random in each zone (0x542A8F), for one-way an exit in each; their guards. Places
        // nothing, draws nothing.
    }
}

/// `rmg_zone_connections_make` 0x543240: the crossing cells ([`crossings`]), then pass 1 and
/// pass 2 ([`Connect::pass_1`], [`Connect::pass_2`]) over each template zone's list.
// rule: RMG-CON-013
pub(crate) fn connect<T: Trace>(
    stream: &mut Stream<'_, T>,
    input: &mut PlanInput<'_>,
    cells: &mut Cells,
) -> Connected {
    let mut work = Connect::new(stream, input, cells);
    work.pass_1();
    work.pass_2();
    work.out
}

#[cfg(test)]
mod tests;
