//! Step 6's connections (`rmg_zone_connections_make` 0x543240, the plan's seventh part; survey
//! `docs/rules/surveys/E14-generator.md` §2.3, "The connections" and "The guards"): the crossing
//! cells; per land zone generator in order its connections not yet made, by land
//! (`rmg_connection_by_land` 0x541140), else by a shipyard (`rmg_connection_by_shipyard`
//! 0x541AD0), else, the other zone being land, by gates (`rmg_connection_by_gate` 0x542080); then
//! per land zone generator with one left, by a shipyard, else, the other zone being land, by
//! monoliths. The monoliths (`rmg_connection_by_monoliths` 0x542CE0) are E14 T4d1's last part and
//! the Border Guards (`rmg_border_guard_place` 0x540D60) T4d2's (RMG-STUB-1): they place nothing
//! and draw nothing.

use alloc::vec;
use alloc::vec::Vec;

use super::PlanInput;
use super::join::carve;
use crate::cells::{Cells, ENTRANCE, FREE, KEPT, OBSTACLES, STEP};
use crate::generate::{Stream, Trace};
use crate::guards::{Monster, create, guard_value};
use crate::objects::{NEIGHBOURS, ObjectFault, Objects, PlaceZone};
use crate::painter::{ROCK, WATER};
use crate::part::Connection;

/// The call site of a land passage's cell draw (`rmg_connection_by_land`,
/// `rmg_passage_cell_draw`): `rand() %` the least-cost crossings left, per passage.
pub const SITE_PASSAGE_CELL: u32 = 0x0054_146B;

/// The call site of a land passage's guard side draw (`rmg_connection_by_land`,
/// `rmg_passage_guard_side_draw`): `rand() & 1`, the monster at P (0) or Q (1).
pub const SITE_PASSAGE_GUARD_SIDE: u32 = 0x0054_1673;

/// The call site of a shipyard connection's prototype draw (`rmg_connection_by_shipyard`,
/// `rmg_shipyard_prototype_draw`): `rand() %` the shipyard prototypes, before any site is sought.
pub const SITE_SHIPYARD_PROTOTYPE: u32 = 0x0054_1B78;

/// The call site of a shipyard's site draw (`rmg_connection_by_shipyard`,
/// `rmg_shipyard_site_draw`): `rand() %` the sites found (RMG-QUIRK-19: with repeats).
pub const SITE_SHIPYARD_SITE: u32 = 0x0054_1D6C;

/// The call site of a gate connection's prototype draw (`rmg_connection_by_gate`,
/// `rmg_gate_prototype_draw`): `rand() %` the gate prototypes, before any position is sought.
pub const SITE_GATE_PROTOTYPE: u32 = 0x0054_21BC;

/// The call site of a gate pair's position draw (`rmg_connection_by_gate`, `rmg_gate_site_draw`):
/// `rand() %` the positions of greatest distance sum.
pub const SITE_GATE_SITE: u32 = 0x0054_2387;

/// +0x28 bit 24: reached by boat (`rmg_boat_reach_flood` 0x541780).
const BOAT: u32 = 1 << 24;

/// The shipyards' class (87).
const SHIPYARDS: u32 = 87;

/// The subterranean gates' class group (103, the generator's +0x6A8).
const GATES: usize = 103;

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

/// The boat reach ([`boat_reach`]) from a shipyard anchored at (x, y, z): from the first of its
/// [`SIDE_STEPS`] cells on the map that is water, kept free or not (none: nothing).
fn boat_from_yard(cells: &mut Cells, [x, y, z]: [i32; 3]) {
    let water = SIDE_STEPS.into_iter().find(|&[dx, dy]| {
        cells
            .get(x.wrapping_add(dx), y.wrapping_add(dy), z)
            .is_some_and(|c| c.land() == u32::from(WATER))
    });
    if let Some([dx, dy]) = water {
        boat_reach(cells, [x.wrapping_add(dx), y.wrapping_add(dy), z]);
    }
}

/// `rmg_border_guard_place` 0x540D60: `count` guards from `at` eastward (by land 1 at P and Q, a
/// shipyard 3 below its footprint from left of its entrance, gates 1 below each), the tent in
/// zone generator `tent`'s zone, then the caller's marks (by land `rmg_border_guard_ring_mark`
/// 0x540FC0, gates the two cells beside): the guard value 0 once one is placed (a shipyard's or
/// gates' monsters are not placed then), else `value`.
fn border_guard(_at: [i32; 3], _count: i32, _tent: usize, value: i32) -> i32 {
    // stub: RMG-STUB-1 — E14 T4d2: the Border Guard (the next Keymaster colour, its tent at
    // random in the zone, 0x542A8F, the marks); places nothing and draws nothing, as every
    // flagged connection of the corpus whose tent finds no room.
    value
}

/// `rmg_shipyard_site_fits` 0x541960 for a shipyard anchored at (x, y, z): the row below on the
/// map; the 3 × 2 cells x − 2..x, y..y + 1 not water, without an entrance (bit 22), free (bit 25)
/// and not rock; the first of its [`SIDE_STEPS`] cells (its column on the map) that is water and
/// kept free (bit 27); the cell on the row y on the other side (x + 1 for a side on the left,
/// x − 3 for one on the right) on the map and not water. A block cell is read unchecked
/// ([`ObjectFault::Outside`] off the map: a shipyard narrower than 3 at x < 2, RMG-QUIRK-4).
// rule: RMG-CON-015
fn site_fits(cells: &Cells, [x, y, z]: [i32; 3]) -> Result<bool, ObjectFault> {
    let below = y.wrapping_add(1);
    if below >= cells.size().1 {
        return Ok(false);
    }
    for v in y..=below {
        for u in x.wrapping_sub(2)..=x {
            let c = cells.get(u, v, z).ok_or(ObjectFault::Outside)?;
            let land = c.land();
            if land == u32::from(WATER)
                || c.tile_data & ENTRANCE != 0
                || c.tile_data & FREE == 0
                || land == u32::from(ROCK)
            {
                return Ok(false);
            }
        }
    }
    let water = SIDE_STEPS.into_iter().find(|&[dx, dy]| {
        cells
            .get(x.wrapping_add(dx), y.wrapping_add(dy), z)
            .is_some_and(|c| c.land() == u32::from(WATER) && c.tile_data & KEPT != 0)
    });
    let Some([dx, _]) = water else {
        return Ok(false);
    };
    let u = if dx < 0 {
        x.wrapping_add(1)
    } else {
        x.wrapping_sub(3)
    };
    Ok(cells
        .get(u, y, z)
        .is_some_and(|c| c.land() != u32::from(WATER)))
}

/// The cell below an entrance `at` kept free and not for obstacles (+0x28 bit 27 set, 26
/// cleared) unless a Border Guard is pending there (+0x2C bit 0). The original writes it
/// unchecked: off the map (a gate line without an entrance, offsets 8, 6) it is a fault
/// ([`ObjectFault::Outside`], RMG-QUIRK-4), nothing written.
fn keep_below(cells: &mut Cells, objects: &mut Objects, [x, y, z]: [i32; 3]) {
    match cells.get_mut(x, y.wrapping_add(1), z) {
        Some(c) => {
            if c.pending_guard & 1 == 0 {
                c.tile_data = (c.tile_data & !OBSTACLES) | KEPT;
            }
        }
        None => objects.fail(ObjectFault::Outside),
    }
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
    /// which is water floods it ([`boat_from_yard`]). The original leaves the row unchecked: it is
    /// on the map, every shipyard being [`Connect::by_shipyard`]'s, whose site test keeps the row
    /// below its anchor on it.
    // rule: RMG-CON-013
    fn yards(&mut self, k: usize) {
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
            boat_from_yard(self.cells, [x, y, z]);
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
        let mut value = self.value(row);
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
                value = border_guard(p, 1, f, value);
                value = border_guard(q, 1, k, value);
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

    /// A guard monster worth `value` at `at` when it holds no object (0x541692 to 0x541746, the
    /// other ways' alike): for the zone generator of its zone byte (unchecked: P's is this zone
    /// generator's, Q's the other's, below a shipyard's or gate's entrance the zone its
    /// `rmg_map_can_place` tested; `owner`, which a byte past them falls back to), created
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

    /// What `rmg_map_can_place` reads of zone generator `g`.
    fn place_zone(&self, g: usize) -> PlaceZone {
        let g = &self.input.gens[g];
        PlaceZone {
            id: self.input.zones[g.zone].id,
            water: g.terrain == i32::from(WATER),
        }
    }

    /// The prototype of class group `class` drawn at `site` (`rand() %` its prototypes,
    /// unsigned); none: the original divides by zero (`fault`, RMG-QUIRK-3).
    fn prototype(&mut self, class: usize, site: u32, fault: ObjectFault) -> Option<usize> {
        let r = self.stream.draw(site) as usize;
        let group = self.input.objects.group(class);
        let found = r.checked_rem(group.len()).map(|i| group[i]);
        if found.is_none() {
            self.input.objects.fail(fault);
        }
        found
    }

    /// The guard value of the connection `row` (0x541F24 to 0x541F97, 0x54250D to 0x542582,
    /// inline: [`guard_value`]'s), 0 when wide.
    fn value(&self, row: &Connection) -> i32 {
        if row.wide {
            0
        } else {
            guard_value(row.value, self.input.guards.strength)
        }
    }

    /// `rmg_connection_by_shipyard` 0x541AD0 between zone generators `k` and `f` for the
    /// connection `row`: refused unless on one level. A shipyard prototype drawn first (0x541B78,
    /// [`ObjectFault::NoShipyard`] without one). Over k's box row by row, its cells of k's zone
    /// the other's flood entered (+0x20 bits 24-31 its id): one reached by boat (+0x28 bit 24)
    /// makes it at once; per land one whose row below is on the map, the anchors x..x + 2 on
    /// its row that `rmg_map_can_place` (in k's zone) and [`site_fits`] allow, a site once per
    /// such cell (RMG-QUIRK-19 on, else once); none: refused. The shipyard created (its use count
    /// raised), its site `rand() %` the sites (0x541D6C, unsigned), placed through the
    /// generator's slot 1; its entrance a road target (+0x14B0); the row below its footprint
    /// (x − width + 1..x, y + 1) kept free unless a Border Guard is pending and each cell into
    /// k's +0x404; the boat reach flooded from its water side ([`boat_from_yard`]). Its guard, on
    /// the row below the anchor: with the Border Guard flag three guards from the entrance's
    /// column less 1, the tent in f's zone; else, g above 0, a guard monster in the entrance's
    /// column ([`Connect::guard`]). Made at this end only.
    // rule: RMG-CON-015, RMG-CON-020
    fn by_shipyard(&mut self, k: usize, f: usize, row: &Connection) -> [bool; 2] {
        let gens = self.input.gens;
        let (g, h) = (&gens[k], &gens[f]);
        let z = g.position[2];
        if z != h.position[2] {
            return [false, false];
        }
        let Some(prototype) = self.prototype(
            SHIPYARDS as usize,
            SITE_SHIPYARD_PROTOTYPE,
            ObjectFault::NoShipyard,
        ) else {
            return [false, false];
        };
        let (a, b) = (self.input.zones[g.zone].id, self.input.zones[h.zone].id);
        let place = self.place_zone(k);
        let height = self.cells.size().1;
        let [x0, y0, x1, y1] = g.bounds;
        let mut sites: Vec<[i32; 3]> = Vec::new();
        for y in y0..y1 {
            for x in x0..x1 {
                // The box is on the map.
                let c = self.cells.clamped(x, y, z);
                if c.zone_index() != a || c.flooded_by() != b {
                    continue;
                }
                if c.tile_data & BOAT != 0 {
                    return [true, false];
                }
                if c.land() == u32::from(WATER) || y.wrapping_add(1) >= height {
                    continue;
                }
                for u in x..=x.wrapping_add(2) {
                    let at = [u, y, z];
                    let fits = self
                        .input
                        .objects
                        .can_place(self.cells, prototype, at, place)
                        && site_fits(self.cells, at).unwrap_or_else(|fault| {
                            self.input.objects.fail(fault);
                            false
                        });
                    if fits && (self.input.shipyard_sites_repeated || !sites.contains(&at)) {
                        sites.push(at);
                    }
                }
            }
        }
        if sites.is_empty() {
            return [false, false];
        }
        let objects = &mut *self.input.objects;
        objects.created(prototype);
        // `rand()` is at most 0x7FFF: an unsigned division.
        let at = sites[self.stream.draw(SITE_SHIPYARD_SITE) as usize % sites.len()];
        objects.add(self.cells, prototype, at);
        let shape = objects.prototypes()[prototype].shape;
        let [x, y, _] = at;
        let entrance = [
            x.wrapping_sub(shape.entrance[0]),
            y.wrapping_sub(shape.entrance[1]),
            z,
        ];
        self.input.road_targets.push(entrance);
        let below = y.wrapping_add(1);
        for u in x.wrapping_sub(shape.width).wrapping_add(1)..=x {
            // On the map: the footprint test keeps x at least the width less 1, the site test
            // the row below on the map.
            let c = self.cells.clamped_mut(u, below, z);
            if c.pending_guard & 1 == 0 {
                c.tile_data = (c.tile_data & !OBSTACLES) | KEPT;
            }
            self.out.passages[k].push([u, below]);
        }
        // Its site's water side is still water: one floods.
        boat_from_yard(self.cells, at);
        // The guards' row is the anchor's below (0x541FAA, 0x541FE6), the entrance's below for an
        // entrance on the anchor's row.
        let mut value = self.value(row);
        let ex = entrance[0];
        if row.border_guard {
            value = border_guard([ex.wrapping_sub(1), below, z], 3, f, value);
        }
        if value > 0 {
            self.guard([ex, below, z], k, value);
        }
        [true, false]
    }

    /// `rmg_connection_by_gate` 0x542080 between zone generators `k` and `f` for the connection
    /// `row`: refused on one level, for k on water, or when their boxes do not overlap. A gate
    /// prototype drawn first (0x5421BC, [`ObjectFault::NoGate`] without one). Over the overlap
    /// row by row, the (x, y) of k's zone on k's level and of f's on f's, whose sum of the two
    /// cells' object distances (+0x20 bits 0-15, §2.1's flood) is at least the greatest kept
    /// (from 0) and where `rmg_map_can_place` allows the prototype on both levels (in k's zone,
    /// then f's); a greater sum empties them; none: refused. One drawn (0x542387, unsigned); a
    /// gate created and placed there on k's level, then one on f's; the entrance (x, y) into
    /// k's +0x404 and f's; the cell below the entrance kept free on each level ([`keep_below`]).
    /// Its guards: with the Border Guard flag one below the entrance on k's level, the tent in
    /// f's zone, then one on f's level, the tent in k's; then, g above 0, a guard monster below
    /// each entrance, k's level first ([`Connect::guard`]). Made at both ends.
    // rule: RMG-CON-016, RMG-CON-020
    fn by_gate(&mut self, k: usize, f: usize, row: &Connection) -> [bool; 2] {
        let gens = self.input.gens;
        let (g, h) = (&gens[k], &gens[f]);
        let (kz, fz) = (g.position[2], h.position[2]);
        if kz == fz || g.terrain == i32::from(WATER) {
            return [false, false];
        }
        let ([ax0, ay0, ax1, ay1], [bx0, by0, bx1, by1]) = (g.bounds, h.bounds);
        let (x0, y0, x1, y1) = (ax0.max(bx0), ay0.max(by0), ax1.min(bx1), ay1.min(by1));
        if x0 >= x1 || y0 >= y1 {
            return [false, false];
        }
        let Some(prototype) = self.prototype(GATES, SITE_GATE_PROTOTYPE, ObjectFault::NoGate)
        else {
            return [false, false];
        };
        let (a, b) = (self.input.zones[g.zone].id, self.input.zones[h.zone].id);
        let (near, far) = (self.place_zone(k), self.place_zone(f));
        let mut best = 0u32;
        let mut sites: Vec<[i32; 2]> = Vec::new();
        for y in y0..y1 {
            for x in x0..x1 {
                // The overlap is on the map.
                let c = self.cells.clamped(x, y, kz);
                if c.zone_index() != a {
                    continue;
                }
                let d = self.cells.clamped(x, y, fz);
                if d.zone_index() != b {
                    continue;
                }
                let sum = (c.zone & 0xFFFF) + (d.zone & 0xFFFF);
                if sum < best {
                    continue;
                }
                let objects = &mut *self.input.objects;
                if !objects.can_place(self.cells, prototype, [x, y, kz], near)
                    || !objects.can_place(self.cells, prototype, [x, y, fz], far)
                {
                    continue;
                }
                if sum > best {
                    sites.clear();
                    best = sum;
                }
                sites.push([x, y]);
            }
        }
        if sites.is_empty() {
            return [false, false];
        }
        // `rand()` is at most 0x7FFF: an unsigned division.
        let [x, y] = sites[self.stream.draw(SITE_GATE_SITE) as usize % sites.len()];
        let objects = &mut *self.input.objects;
        for z in [kz, fz] {
            objects.created(prototype);
            objects.add(self.cells, prototype, [x, y, z]);
        }
        let [dx, dy] = objects.prototypes()[prototype].shape.entrance;
        let (ex, ey) = (x.wrapping_sub(dx), y.wrapping_sub(dy));
        self.out.passages[k].push([ex, ey]);
        self.out.passages[f].push([ex, ey]);
        let mut value = self.value(row);
        for z in [kz, fz] {
            keep_below(self.cells, self.input.objects, [ex, ey, z]);
        }
        let (p, q) = ([ex, ey.wrapping_add(1), kz], [ex, ey.wrapping_add(1), fz]);
        if row.border_guard {
            value = border_guard(p, 1, f, value);
            value = border_guard(q, 1, k, value);
        }
        if value > 0 {
            self.guard(p, k, value);
            self.guard(q, f, value);
        }
        [true, true]
    }

    /// `rmg_connection_by_monoliths` 0x542CE0 for zone generators `k` and `f` with the monolith
    /// index `index`: both ends are made whatever it places.
    fn by_monoliths(&mut self, _k: usize, _f: usize, _row: &Connection, _index: usize) {
        // stub: RMG-STUB-1 — E14 T4d1 (its last part): the index's two-way or one-way
        // prototypes, an entrance at random in each zone (0x542A8F), for one-way an exit in
        // each; their guards. Places nothing, draws nothing.
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
