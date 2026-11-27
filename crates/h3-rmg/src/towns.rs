//! Steps 4 and 5, the towns (`rmg_zone_first_town` 0x544A50 and `rmg_zone_towns_place`
//! 0x544AE0 per zone generator; survey `docs/rules/surveys/E14-generator.md` §2.2, "Steps 4 and
//! 5, the towns" and "Steps 4 and 5 as ported (T4b)", §3.2.6, RMG-TWN-001 to -005): each zone's
//! first town nearest its centre (`rmg_town_place_central` 0x545250), then its least towns and
//! the towns its densities ask for, each farthest from the objects placed
//! (`rmg_town_place_far` 0x544D90).
//!
//! A town is of the type-98 prototype at the town type's index in its class group's vector (the
//! generator's +0x658: SoD's `objects.txt` files the nine towns' lines in subtype order, so the
//! index is the subtype), owned by the zone's colour (the generator's +0xEE4 by the zone's
//! position, Ownership less 1; -1 neutral) or neutral, with a castle or not.

use alloc::vec::Vec;

use crate::cells::{Cells, KEPT, OBSTACLES};
use crate::generate::{GenerateError, Stream, Trace};
use crate::layout::{ZoneGenerator, trunc_sqrt};
use crate::objects::{ObjectFault, Objects, PlaceZone};
use crate::part::Zone;
use crate::settings::{COLOURS, MapFormat};

/// The call site of the first town's cell draw (`rmg_town_place_central`,
/// `rmg_town_central_cell_draw`).
pub const SITE_TOWN_CENTRAL_CELL: u32 = 0x0054_5492;
/// The call site of a further town's type draw among the zone's town types
/// (`rmg_town_place_far`, `rmg_town_type_zone_draw`).
pub const SITE_TOWN_TYPE_ZONE: u32 = 0x0054_4DEC;
/// The call site of a further town's type draw among all (`rmg_town_place_far`,
/// `rmg_town_type_any_draw`).
pub const SITE_TOWN_TYPE_ANY: u32 = 0x0054_4E22;
/// The call site of a further town's cell draw (`rmg_town_place_far`, `rmg_town_far_cell_draw`).
pub const SITE_TOWN_FAR_CELL: u32 = 0x0054_5174;

/// The towns' class (98).
const TOWNS: usize = 98;
/// Water's terrain number.
const WATER_TERRAIN: i32 = 8;
/// The first town's least squared distance to the centre before any cell (32000).
const NEAREST: i32 = 32_000;
/// The densities' spacing numerator: trunc(sqrt(82944 / S)).
const SPACING: i32 = 82_944;

/// A town placed (`rmg_town_object`, 0x28 bytes: `rmg_object`'s base, then +0x1C to +0x24).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Town {
    /// Its object, by its index among the placed objects.
    pub object: usize,
    /// The zone generator it was placed for.
    pub zone: usize,
    /// +0x1C: the generator's object serial (+0xF44) when it was created.
    pub serial: i32,
    /// +0x20: its owner's colour, -1 neutral.
    pub owner: i32,
    /// +0x24: a castle (a fort), not a village.
    pub castle: bool,
}

/// A kind of town a zone asks for: of its colour or neutral, a castle or not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Kind {
    colour: bool,
    castle: bool,
}

/// The kinds in the steps' order: colour castles, colour towns, neutral castles, neutral towns.
const KINDS: [Kind; 4] = [
    Kind {
        colour: true,
        castle: true,
    },
    Kind {
        colour: true,
        castle: false,
    },
    Kind {
        colour: false,
        castle: true,
    },
    Kind {
        colour: false,
        castle: false,
    },
];

/// A zone's least count of each kind (+0x24, +0x20, +0x34, +0x30).
fn least(zone: &Zone) -> [i32; 4] {
    [
        zone.player_towns[1],
        zone.player_towns[0],
        zone.neutral_towns[1],
        zone.neutral_towns[0],
    ]
}

/// A zone's density of each kind (+0x2C, +0x28, +0x3C, +0x38).
fn density(zone: &Zone) -> [i32; 4] {
    [
        zone.player_towns[3],
        zone.player_towns[2],
        zone.neutral_towns[3],
        zone.neutral_towns[2],
    ]
}

/// What steps 4 and 5 read besides the stream and what they write.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TownInput<'a> {
    /// The kept template's zones (step 2's water zones appended).
    pub zones: &'a [Zone],
    /// Each start position's colour.
    pub positions: &'a [Option<u8>; COLOURS],
    pub format: MapFormat,
    /// `quirk.rmg.least_town_skipped` (RMG-QUIRK-13): step 5 skips the first of the first kind
    /// asked for whether step 4 placed it or not.
    pub least_town_skipped: bool,
}

/// What steps 4 and 5 write besides the cells and the objects: the towns, the road targets.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Placed {
    /// The towns in placing order.
    pub towns: Vec<Town>,
    /// The generator's +0x14B0: each town's entrance x, y, z, in placing order (step 6 appends
    /// the shipyards'; step 14's roads lead to them).
    pub road_targets: Vec<[i32; 3]>,
}

/// The steps' work.
struct Placing<'s, 'a, 't, T: Trace> {
    stream: &'s mut Stream<'t, T>,
    input: &'s TownInput<'a>,
    gens: &'s mut [ZoneGenerator],
    cells: &'s mut Cells,
    objects: &'s mut Objects,
    out: &'s mut Placed,
}

impl<T: Trace> Placing<'_, '_, '_, T> {
    fn zone(&self, g: usize) -> &Zone {
        &self.input.zones[self.gens[g].zone]
    }

    /// What `rmg_map_can_place` reads of zone generator `g`.
    fn place_zone(&self, g: usize) -> PlaceZone {
        PlaceZone {
            id: self.zone(g).id,
            water: self.gens[g].terrain == WATER_TERRAIN,
        }
    }

    /// The owner of a town of `kind` in zone generator `g`: for a colour kind the colour of the
    /// zone's position (+0xEE4 + 4p, p its Ownership less 1; -1 for none and for p = -1, whose
    /// +0xEE0 `rmg_generate_map` clears to -1 and nothing writes), else -1. A position past 7 or
    /// below -1 reads outside the colours ([`ObjectFault::Colour`]; -1).
    fn owner(&mut self, g: usize, kind: Kind) -> i32 {
        if !kind.colour {
            return -1;
        }
        let position = self.zone(g).ownership.wrapping_sub(1);
        if position == -1 {
            return -1;
        }
        match usize::try_from(position)
            .ok()
            .and_then(|p| self.input.positions.get(p))
        {
            Some(colour) => colour.map_or(-1, i32::from),
            None => {
                self.objects.fail(ObjectFault::Colour);
                -1
            }
        }
    }

    /// The town prototype of `town_type` (≥ 0): the vector's entry at that index
    /// ([`ObjectFault::Prototype`] past its end).
    fn prototype(&mut self, town_type: i32) -> Option<usize> {
        let found = usize::try_from(town_type)
            .ok()
            .and_then(|t| self.objects.group(TOWNS).get(t).copied());
        if found.is_none() {
            self.objects.fail(ObjectFault::Prototype);
        }
        found
    }

    /// A town of `prototype` created (its serial +0xF44 taken, its use count raised) and put at
    /// `at` through the generator's slot 1 ([`Objects::add`]), after the draw among `cells` at
    /// `site`; its entrance (`at` less the template's entrance offsets) into the road targets,
    /// the cell below it marked kept free and not for obstacles (+0x28 bit 27 set and 26
    /// cleared) unless a Border Guard is pending there (+0x2C bit 0). Returns the entrance.
    // rule: RMG-TWN-002, RMG-TWN-005
    fn put(
        &mut self,
        g: usize,
        prototype: usize,
        cells: &[[i32; 3]],
        site: u32,
        owner: i32,
        castle: bool,
    ) -> [i32; 3] {
        let serial = self.objects.take_serial();
        self.objects.created(prototype);
        // `rand()` is at most 0x7FFF: an unsigned division.
        let at = cells[self.stream.draw(site) as usize % cells.len()];
        let object = self.objects.add(self.cells, prototype, at);
        let [dx, dy] = self.objects.prototypes()[prototype].shape.entrance;
        let entrance = [at[0].wrapping_sub(dx), at[1].wrapping_sub(dy), at[2]];
        self.out.road_targets.push(entrance);
        // The original writes it unchecked: a town line without an entrance (offsets 8, 6)
        // reaches off the map (RMG-QUIRK-4).
        match self
            .cells
            .get_mut(entrance[0], entrance[1].wrapping_add(1), entrance[2])
        {
            Some(c) => {
                if c.pending_guard & 1 == 0 {
                    c.tile_data = (c.tile_data & !OBSTACLES) | KEPT;
                }
            }
            None => self.objects.fail(ObjectFault::Outside),
        }
        self.out.towns.push(Town {
            object,
            zone: g,
            serial,
            owner,
            castle,
        });
        entrance
    }

    /// `rmg_town_place_central` 0x545250: a zone's first town of `town_type` (-1 refused):
    /// among the cells of the zone generator's box of its zone (the signed zone byte its id), row
    /// by row, at a squared distance from its centre of at most the least so far (from 32000)
    /// and where `rmg_map_can_place` allows the prototype, those of the least distance;
    /// `rand() %` them (0x545492); the town put there, its entrance the zone generator's +0x30,
    /// +0x3C set. None: false, no draw.
    // rule: RMG-TWN-002
    fn central(&mut self, g: usize, town_type: i32, owner: i32, castle: bool) -> bool {
        if town_type == -1 {
            return false;
        }
        let Some(prototype) = self.prototype(town_type) else {
            return false;
        };
        let place = self.place_zone(g);
        let zg = self.gens[g];
        let [cx, cy, z] = zg.position;
        let [x0, y0, x1, y1] = zg.bounds;
        let mut best = NEAREST;
        let mut cells = Vec::new();
        for y in y0..y1 {
            for x in x0..x1 {
                if !self
                    .cells
                    .get(x, y, z)
                    .is_some_and(|c| c.zone_index() == place.id)
                {
                    continue;
                }
                let (dx, dy) = (x.wrapping_sub(cx), y.wrapping_sub(cy));
                let d = dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy));
                if d > best
                    || !self
                        .objects
                        .can_place(self.cells, prototype, [x, y, z], place)
                {
                    continue;
                }
                if d < best {
                    best = d;
                    cells.clear();
                }
                cells.push([x, y, z]);
            }
        }
        if cells.is_empty() {
            return false;
        }
        let entrance = self.put(g, prototype, &cells, SITE_TOWN_CENTRAL_CELL, owner, castle);
        self.gens[g].town = Some(entrance);
        true
    }

    /// `rmg_town_place_far` 0x544D90: one more town in zone generator `g`. Its type is redrawn
    /// when the owner is -1 and the zone has not its same-type byte (+0x40), or when `town_type`
    /// is -1: `rand() %` the zone's set town types, the r-th (0x544DEC), none set `rand() % 8`
    /// in RoE, 9 otherwise (0x544E22). A zone without a town yet takes
    /// [`Placing::central`]; else over the positions from the box's least corner plus the
    /// footprint's width and height to its far edges, row by row: the entrance cell (the
    /// position less the entrance offsets) of the zone, its object distance (+0x20 bits 0-15) at
    /// least the greatest kept so far (from `spacing`), `rmg_map_can_place` true and every cell
    /// of the 3 × 3 around the entrance on the map of the zone; those of the greatest distance,
    /// `rand() %` them (0x545174), the town put there. None: false.
    // rule: RMG-TWN-005
    fn far(&mut self, g: usize, town_type: i32, owner: i32, castle: bool, spacing: i32) -> bool {
        let mut town_type = town_type;
        if (owner == -1 && !self.zone(g).same_type) || town_type == -1 {
            let types = self.zone(g).town_types;
            let set: Vec<i32> = (0..9).filter(|&k| types >> k & 1 != 0).collect();
            town_type = if set.is_empty() {
                let all = if self.input.format.version() >= 1 {
                    9
                } else {
                    8
                };
                (self.stream.draw(SITE_TOWN_TYPE_ANY) % all) as i32
            } else {
                set[self.stream.draw(SITE_TOWN_TYPE_ZONE) as usize % set.len()]
            };
        }
        if self.gens[g].town.is_none() {
            return self.central(g, town_type, owner, castle);
        }
        let Some(prototype) = self.prototype(town_type) else {
            return false;
        };
        let shape = self.objects.prototypes()[prototype].shape;
        let place = self.place_zone(g);
        let zg = self.gens[g];
        let z = zg.position[2];
        let [x0, y0, x1, y1] = zg.bounds;
        let (w, h, _) = self.cells.size();
        let mut best = spacing;
        let mut cells = Vec::new();
        for y in y0.wrapping_add(shape.height)..y1 {
            for x in x0.wrapping_add(shape.width)..x1 {
                let (ex, ey) = (
                    x.wrapping_sub(shape.entrance[0]),
                    y.wrapping_sub(shape.entrance[1]),
                );
                // Unchecked in the original: off the map only for a line without an entrance.
                let Some(c) = self.cells.get(ex, ey, z) else {
                    self.objects.fail(ObjectFault::Outside);
                    continue;
                };
                let d = (c.zone & 0xFFFF) as i32;
                if c.zone_index() != place.id
                    || d < best
                    || !self
                        .objects
                        .can_place(self.cells, prototype, [x, y, z], place)
                {
                    continue;
                }
                let around = (ey.wrapping_sub(1).max(0)..ey.wrapping_add(2).min(h)).all(|v| {
                    (ex.wrapping_sub(1).max(0)..ex.wrapping_add(2).min(w)).all(|u| {
                        self.cells.get(u, v, z).is_some_and(|c| {
                            let i = c.zone_index();
                            i >= 0 && i == place.id
                        })
                    })
                });
                if !around {
                    continue;
                }
                if d > best {
                    best = d;
                    cells.clear();
                }
                cells.push([x, y, z]);
            }
        }
        if cells.is_empty() {
            return false;
        }
        self.put(g, prototype, &cells, SITE_TOWN_FAR_CELL, owner, castle);
        true
    }

    /// `rmg_zone_first_town` 0x544A50 for zone generator `g`: the kinds in order, each only when
    /// its least count is above 0, with the zone generator's town type (+0x4), until one is
    /// placed.
    // rule: RMG-TWN-001
    fn first(&mut self, g: usize) {
        let least = least(self.zone(g));
        let town_type = self.gens[g].town_type;
        for (kind, count) in KINDS.into_iter().zip(least) {
            if count <= 0 {
                continue;
            }
            let owner = self.owner(g, kind);
            if self.central(g, town_type, owner, kind.castle) {
                return;
            }
        }
    }

    /// `rmg_zone_towns_place` 0x544AE0 for zone generator `g`: per kind in order with a least
    /// count above 0, that many towns at spacing 0, the first of the first such kind skipped as
    /// step 4's (RMG-QUIRK-13: whether step 4 placed it or not with `least_town_skipped`, else
    /// only when it did); then the densities d above 0, their sum S (none: done) and product P
    /// (32-bit), the spacing r = trunc(sqrt(82944 / S)), per such kind its step P / d and key
    /// step × its least count; repeatedly the kind not exhausted with the least key (the first
    /// on ties) adds its step to its key and places one more town at spacing r, exhausted at
    /// its first failure, until every kind is.
    // rule: RMG-TWN-003, RMG-TWN-004
    fn further(&mut self, g: usize) {
        let zone = self.zone(g);
        let (least, density) = (least(zone), density(zone));
        let town_type = self.gens[g].town_type;
        let mut skip = self.input.least_town_skipped || self.gens[g].town.is_some();
        for (kind, count) in KINDS.into_iter().zip(least) {
            if count <= 0 {
                continue;
            }
            for _ in i32::from(skip)..count {
                let owner = self.owner(g, kind);
                self.far(g, town_type, owner, kind.castle, 0);
            }
            skip = false;
        }
        let mut exhausted = [true; 4];
        let (mut sum, mut product) = (0i32, 1i32);
        for (k, &d) in density.iter().enumerate() {
            if d > 0 {
                sum = sum.wrapping_add(d);
                product = d.wrapping_mul(product);
                exhausted[k] = false;
            }
        }
        if sum == 0 {
            return;
        }
        let spacing = trunc_sqrt(SPACING.wrapping_div(sum));
        let mut step = [0i32; 4];
        let mut key = least;
        for k in 0..4 {
            if !exhausted[k] {
                step[k] = product.wrapping_div(density[k]);
                key[k] = step[k].wrapping_mul(least[k]);
            }
        }
        // Each town placed takes a free cell (its blocked cells, or its entrance's bit 22) that
        // no later town may take, so a kind fails within the map's cells.
        while let Some(k) = (0..4)
            .filter(|&k| !exhausted[k])
            .reduce(|a, b| if key[b] < key[a] { b } else { a })
        {
            key[k] = key[k].wrapping_add(step[k]);
            let owner = self.owner(g, KINDS[k]);
            if !self.far(g, town_type, owner, KINDS[k].castle, spacing) {
                exhausted[k] = true;
            }
        }
    }
}

/// Step 4 (`rmg_zone_first_town` 0x544A50 per zone generator in order) or step 5
/// (`rmg_zone_towns_place` 0x544AE0 likewise).
///
/// # Errors
///
/// [`GenerateError::Objects`]: the object functions' first fault of the step.
pub(crate) fn towns<T: Trace>(
    stream: &mut Stream<'_, T>,
    input: &TownInput<'_>,
    step: u8,
    gens: &mut [ZoneGenerator],
    cells: &mut Cells,
    objects: &mut Objects,
    out: &mut Placed,
) -> Result<(), GenerateError> {
    let mut placing = Placing {
        stream,
        input,
        gens,
        cells,
        objects,
        out,
    };
    for g in 0..placing.gens.len() {
        if step == 4 {
            placing.first(g);
        } else {
            placing.further(g);
        }
    }
    objects_fault(placing.objects, step)
}

/// A step's end: the object functions' first fault as its error.
fn objects_fault(objects: &mut Objects, step: u8) -> Result<(), GenerateError> {
    objects
        .take_fault()
        .map_or(Ok(()), |fault| Err(GenerateError::Objects { step, fault }))
}

#[cfg(test)]
mod tests;
