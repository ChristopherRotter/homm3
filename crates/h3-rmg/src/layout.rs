//! Step 1, the zones laid out (`rmg_zones_lay_out` 0x53BCB0; survey
//! `docs/rules/surveys/E14-generator.md` §2.2 and §3.2.3, RMG-ZON-001 to -008): per template
//! zone a zone generator with its town type, three passes of placements in template units, the
//! layout scaled to the map, then each zone's terrain and second town type.
//!
//! The placements mutate the placed zone generator's position as the original does (each
//! candidate tested is written there first), and the candidates around the placed zone itself
//! (passes 2 and 3 without a connected zone's) start from that written position.

use alloc::vec::Vec;

use h3_kernel::num::Real;

use crate::generate::{GenerateError, Stream, Trace};
use crate::part::{GeneratorTables, LAYOUT_POINTS, TERRAIN_TOWNS, Zone};
use crate::settings::{COLOURS, MapFormat, Water};
use crate::template::KeptConnection;

/// The call site of a zone generator's town type draw (`rmg_zone_generator_init`).
pub const SITE_ZONE_TOWN_TYPE: u32 = 0x0053_2A5B;
/// The call site of a placement's draw among its candidates (`rmg_zone_place`).
pub const SITE_ZONE_POSITION: u32 = 0x0053_BB5F;
/// The call site of a zone's terrain draw (`rmg_zone_terrain_draw`).
pub const SITE_ZONE_TERRAIN: u32 = 0x0053_2B00;
/// The call site of a zone's second town type draw (`rmg_zones_lay_out`).
pub const SITE_ZONE_TERRAIN_TOWN: u32 = 0x0053_BFC0;

/// The terrain numbers step 1 tests: subterranean (allowed only on level 1; the level's land
/// but lava turns to it) and lava.
const SUBTERRANEAN: i32 = 6;
const LAVA: i32 = 7;
/// The town types a start zone on level 1 needs (Inferno, Necropolis, Dungeon).
const UNDERGROUND_TOWNS: core::ops::RangeInclusive<i32> = 3..=5;
/// Conflux, which RoE has not.
const CONFLUX: i32 = 8;

/// A zone generator as step 1 leaves it (`rmg_zone_generator`, `docs/exe/structs.toml`): the
/// words +0x4 to +0x1C the phase oracle's `layout.txt` records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZoneGenerator {
    /// Its template zone's index.
    pub zone: usize,
    /// +0x4: drawn among the zone's town types (-1 none), a start zone with colour towns taking
    /// its colour's town choice.
    pub town_type: i32,
    /// +0x8: +0x4, or one drawn from its terrain's row (-1 none).
    pub town_type_2: i32,
    /// +0xC: dirt 0 to lava 7 (8 water).
    pub terrain: i32,
    /// +0x10 to +0x18: x, y in template units until the scaling, then on the map; the level.
    pub position: [i32; 3],
    /// +0x1C: the base size, then scaled.
    pub size: i32,
}

/// What step 1 reads besides the stream.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LayoutInput<'a> {
    /// The kept template's zones and connections.
    pub zones: &'a [Zone],
    pub connections: &'a [KeptConnection],
    /// Each start position's colour and each colour's town choice.
    pub positions: &'a [Option<u8>; COLOURS],
    pub towns: &'a [Option<u8>; COLOURS],
    /// The map's side and levels, its water.
    pub side: i32,
    pub levels: i32,
    pub water: Water,
    pub format: MapFormat,
    pub tables: &'a GeneratorTables,
    /// `quirk.rmg.terrain_towns_count` (RMG-QUIRK-12): the second town type drawn among all four
    /// entries of the terrain's row.
    pub all_four: bool,
}

/// `trunc(sqrt(n))` as `crt_sqrt` and `crt_ftol` give it for an `i32`: the integer square root
/// (binary64's correctly rounded root never reaches the next integer below 2^31); a negative `n`
/// (a wrapped square) is NaN, whose 64-bit conversion leaves 0 in the low word.
pub(crate) fn trunc_sqrt(n: i32) -> i32 {
    let Ok(n) = u32::try_from(n) else {
        return 0;
    };
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = x / 2 + x % 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x.cast_signed()
}

/// Step 1's point `trunc(f64(r) × t + f64(o))` (binary64, `crt_ftol` truncating; the low word).
fn point(r: i32, t: Real, o: i32) -> i32 {
    Real::from_i32(r)
        .checked_mul(t)
        .and_then(|v| v.checked_add(Real::from_i32(o)))
        .and_then(Real::trunc_to_i64)
        .map_or(0, |v| v as i32)
}

/// A table entry in ten-thousandths as the double nearest it (exact: a correctly rounded
/// quotient of two exact integers).
pub(crate) fn decimal(ten_thousandths: i32) -> Real {
    Real::from_i32(ten_thousandths)
        .checked_div(Real::from_i32(10_000))
        .unwrap_or(Real::ZERO)
}

/// The layout under way: the zone generators, the levels, the tables as doubles.
struct Layout<'a> {
    input: &'a LayoutInput<'a>,
    /// Per zone, the other end of each of its connections in the template's order (each
    /// connection appended to its first zone's list, then to its second's).
    links: Vec<Vec<usize>>,
    cos: [Real; LAYOUT_POINTS],
    sin: [Real; LAYOUT_POINTS],
    gens: Vec<ZoneGenerator>,
}

impl Layout<'_> {
    fn base(&self, g: usize) -> i32 {
        self.input.zones[self.gens[g].zone].base_size
    }

    /// `rmg_zone_position_fits` 0x53AD60 for `me` at its position, against the first `created`
    /// zone generators: a start zone on level 1 needs an underground town; every other zone on
    /// the level `10 × trunc(dist) ≥ 8 × (the two bases)`.
    // rule: RMG-ZON-004
    fn fits(&self, me: usize, created: usize) -> bool {
        let g = &self.gens[me];
        let kind = self.input.zones[g.zone].kind.value();
        if (kind == 0 || kind == 1)
            && g.position[2] == 1
            && !UNDERGROUND_TOWNS.contains(&g.town_type)
        {
            return false;
        }
        let base = self.base(me);
        self.gens[..created].iter().enumerate().all(|(k, o)| {
            if o.position[2] != g.position[2] || o.zone == g.zone {
                return true;
            }
            let dx = o.position[0].wrapping_sub(g.position[0]);
            let dy = o.position[1].wrapping_sub(g.position[1]);
            let t = trunc_sqrt(dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy)));
            t.wrapping_mul(10) >= self.base(k).wrapping_add(base).wrapping_mul(8)
        })
    }

    /// `rmg_zone_candidates_around` 0x53AE80: the 32 points at the bases' sum around `o`'s
    /// position on its level, and with two levels its position on the other level and 32 points
    /// there at the larger base, each kept when `me` fits there.
    // rule: RMG-ZON-003
    fn around(&mut self, o: usize, me: usize, created: usize, out: &mut Vec<[i32; 3]>) {
        let [ox, oy, oz] = self.gens[o].position;
        let (bo, bm) = (self.base(o), self.base(me));
        let mut test = |layout: &mut Self, p: [i32; 3]| {
            layout.gens[me].position = p;
            if layout.fits(me, created) {
                out.push(p);
            }
        };
        let r = bo.wrapping_add(bm);
        for k in 0..LAYOUT_POINTS {
            let p = [point(r, self.cos[k], ox), point(r, self.sin[k], oy), oz];
            test(self, p);
        }
        if self.input.levels == 1 {
            return;
        }
        let other = 1i32.wrapping_sub(oz);
        test(self, [ox, oy, other]);
        let r = bo.max(bm);
        for k in 0..LAYOUT_POINTS {
            let p = [point(r, self.cos[k], ox), point(r, self.sin[k], oy), other];
            test(self, p);
        }
    }

    /// `rmg_zones_touch` 0x532BD0: on one level `11 s ≥ 10 t`; across `s ≥ t` and `s − t` above
    /// half the smaller base (s the bases' sum, t the truncated distance).
    fn touch(&self, a: usize, b: usize) -> bool {
        let (p, q) = (self.gens[a].position, self.gens[b].position);
        let dx = p[0].wrapping_sub(q[0]);
        let dy = p[1].wrapping_sub(q[1]);
        let t = trunc_sqrt(dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy)));
        let (ba, bb) = (self.base(a), self.base(b));
        let s = ba.wrapping_add(bb);
        if p[2] == q[2] {
            s.wrapping_mul(11) >= t.wrapping_mul(10)
        } else {
            s >= t && s.wrapping_sub(t) > ba.min(bb) / 2
        }
    }

    /// The connected zones among the first `created` that `me` touches at `p` (written there).
    fn touching(&mut self, me: usize, created: usize, p: [i32; 3]) -> usize {
        self.gens[me].position = p;
        let zone = self.gens[me].zone;
        self.links[zone]
            .iter()
            .filter(|&&other| other < created && self.touch(other, me))
            .count()
    }

    /// `rmg_zone_candidates_filter` 0x53B2F0: with two levels and a level holding no other zone,
    /// only that level's candidates when one stands at an index above 0 (RMG-QUIRK-3's survey
    /// note: index 0 alone filters nothing); then those touching the most connected zones; then
    /// those giving the least `max(d, the box's height, width)`, the box over the other zones
    /// and the candidate's zone (from 0).
    // rule: RMG-ZON-005
    fn filter(&mut self, me: usize, created: usize, cands: &mut Vec<[i32; 3]>, d: i32) {
        if self.input.levels > 1 {
            let mut occupied = [false; 2];
            for (k, o) in self.gens[..created].iter().enumerate() {
                if k != me {
                    occupied[usize::from(o.position[2] == 1)] = true;
                }
            }
            let free = |p: &[i32; 3]| !occupied[usize::from(p[2] == 1)];
            if !(occupied[0] && occupied[1]) && cands.iter().rposition(free).is_some_and(|i| i > 0)
            {
                cands.retain(free);
            }
        }
        let most = cands
            .clone()
            .into_iter()
            .map(|p| self.touching(me, created, p))
            .max()
            .unwrap_or(0);
        for i in (0..cands.len()).rev() {
            if self.touching(me, created, cands[i]) < most {
                cands.remove(i);
            }
        }
        // The box over the others: least y, x, greatest y + 1, x + 1 (from 0).
        let mut others = [0i32; 4];
        for (k, o) in self.gens[..created].iter().enumerate() {
            if k == me {
                continue;
            }
            let b = self.base(k);
            let [x, y, _] = o.position;
            others[0] = others[0].min(y.wrapping_sub(b));
            others[1] = others[1].min(x.wrapping_sub(b));
            others[2] = others[2].max(y.wrapping_add(b).wrapping_add(1));
            others[3] = others[3].max(x.wrapping_add(b).wrapping_add(1));
        }
        let b = self.base(me);
        let value = |p: &[i32; 3]| {
            let [x, y, _] = *p;
            let height = others[2]
                .max(y.wrapping_add(b).wrapping_add(1))
                .wrapping_sub(others[0].min(y.wrapping_sub(b)));
            let width = others[3]
                .max(x.wrapping_add(b).wrapping_add(1))
                .wrapping_sub(others[1].min(x.wrapping_sub(b)));
            d.max(height).max(width)
        };
        let best = cands.iter().map(value).fold(32_000, i32::min);
        cands.retain(|p| value(p) <= best);
    }

    /// `rmg_zone_place` 0x53B970: one placement of `me` against the first `created` zone
    /// generators (none: (0,0,0), with two levels also (0,0,1) if it fits, unfiltered); else the
    /// candidates around its connected created zones (none: around every created zone),
    /// filtered; then `rand() %` their number.
    // rule: RMG-ZON-002, RMG-ZON-003
    fn place<T: Trace>(
        &mut self,
        stream: &mut Stream<'_, T>,
        me: usize,
        created: usize,
        d: i32,
    ) -> Result<(), GenerateError> {
        let mut cands = Vec::new();
        if created == 0 {
            self.gens[me].position = [0, 0, 0];
            cands.push([0, 0, 0]);
            if self.input.levels > 1 {
                self.gens[me].position = [0, 0, 1];
                if self.fits(me, created) {
                    cands.push([0, 0, 1]);
                }
            }
        } else {
            let zone = self.gens[me].zone;
            for k in 0..self.links[zone].len() {
                let other = self.links[zone][k];
                if other < created {
                    self.around(other, me, created, &mut cands);
                }
            }
            if cands.is_empty() {
                for o in 0..created {
                    self.around(o, me, created, &mut cands);
                }
            }
            self.filter(me, created, &mut cands, d);
        }
        if cands.is_empty() {
            return Err(GenerateError::NoPosition {
                zone: self.gens[me].zone,
            });
        }
        // `rand()` is at most 0x7FFF: an unsigned division.
        let r = stream.draw(SITE_ZONE_POSITION) as usize % cands.len();
        self.gens[me].position = cands[r];
        Ok(())
    }
}

/// `rmg_zone_generator_init` 0x5329E0's town type: `rand() %` the set town types, the r-th set
/// one (none set: -1, no draw).
// rule: RMG-ZON-001
pub(crate) fn town_type_drawn<T: Trace>(stream: &mut Stream<'_, T>, mask: u16) -> i32 {
    let set: Vec<i32> = (0..9).filter(|&k| mask >> k & 1 != 0).collect();
    if set.is_empty() {
        return -1;
    }
    set[stream.draw(SITE_ZONE_TOWN_TYPE) as usize % set.len()]
}

/// `rmg_zone_terrain_draw` 0x532AB0: the town's native terrain when the zone's terrain matches
/// it and it has one; else `rand() %` the allowed terrains (subterranean only on level 1; none:
/// dirt); on level 1 every terrain but lava turns to subterranean.
// rule: RMG-ZON-007
fn terrain_drawn<T: Trace>(
    stream: &mut Stream<'_, T>,
    zone: &Zone,
    g: &ZoneGenerator,
    native: &[i32],
) -> i32 {
    let level = g.position[2];
    let terrain = match usize::try_from(g.town_type) {
        Ok(town) if zone.terrain_match => native.get(town).copied().unwrap_or(0),
        _ => {
            let allowed: Vec<i32> = (0..8)
                .filter(|&k| zone.terrains >> k & 1 != 0 && (k != SUBTERRANEAN || level == 1))
                .collect();
            if allowed.is_empty() {
                0
            } else {
                allowed[stream.draw(SITE_ZONE_TERRAIN) as usize % allowed.len()]
            }
        }
    };
    if level == 1 && terrain != LAVA {
        SUBTERRANEAN
    } else {
        terrain
    }
}

/// The second town type (+0x8): the town type when set; else from the terrain's row of
/// `rmg_terrain_towns`, `rand() %` all four entries with `all_four` (RMG-QUIRK-12: the -1 ending
/// the row and the Castle padding drawn), else among the row's town types before its -1
/// (Conflux not in RoE); none: -1.
// rule: RMG-ZON-008
fn second_town_type<T: Trace>(
    stream: &mut Stream<'_, T>,
    g: &ZoneGenerator,
    row: [i32; TERRAIN_TOWNS],
    all_four: bool,
    format: MapFormat,
) -> i32 {
    if g.town_type != -1 {
        return g.town_type;
    }
    let candidates: Vec<i32> = if all_four {
        row.to_vec()
    } else {
        row.iter()
            .copied()
            .take_while(|&t| t != -1)
            .filter(|&t| t != CONFLUX || format != MapFormat::RoE)
            .collect()
    };
    if candidates.is_empty() {
        return -1;
    }
    candidates[stream.draw(SITE_ZONE_TERRAIN_TOWN) as usize % candidates.len()]
}

/// Step 1 (`rmg_zones_lay_out` 0x53BCB0): the zone generators laid out.
///
/// # Errors
///
/// RMG-QUIRK-3's step 1 cases, which the original divides by zero at: a placement without a
/// candidate ([`GenerateError::NoPosition`]), a layout spanning nothing
/// ([`GenerateError::LayoutSpan`]).
// rule: RMG-ZON-001, RMG-ZON-002, RMG-ZON-006
pub(crate) fn lay_out<T: Trace>(
    stream: &mut Stream<'_, T>,
    input: &LayoutInput<'_>,
) -> Result<Vec<ZoneGenerator>, GenerateError> {
    let tables = input.tables;
    let mut links = alloc::vec![Vec::new(); input.zones.len()];
    for c in input.connections {
        links[c.zones[0]].push(c.zones[1]);
        links[c.zones[1]].push(c.zones[0]);
    }
    let mut layout = Layout {
        input,
        links,
        cos: core::array::from_fn(|k| decimal(tables.layout_cos.get(k).copied().unwrap_or(0))),
        sin: core::array::from_fn(|k| decimal(tables.layout_sin.get(k).copied().unwrap_or(0))),
        gens: Vec::with_capacity(input.zones.len()),
    };
    // d: the least base (from 32000) times the map's smaller side, over 5, 6 or 7 by the water.
    let least = input
        .zones
        .iter()
        .map(|z| z.base_size)
        .fold(32_000, i32::min);
    let d = least.wrapping_mul(input.side)
        / match input.water {
            Water::None => 5,
            Water::Islands => 7,
            Water::Normal | Water::Random => 6,
        };
    // Pass 1 makes each zone generator before its placement (its town type drawn first), passes 2
    // and 3 place every one again in order.
    let n = input.zones.len();
    for k in 0..3 * n {
        let me = k % n;
        if k < n {
            let zone = &input.zones[me];
            let mut town_type = town_type_drawn(stream, zone.town_types);
            let choice = (zone.player_towns[0].wrapping_add(zone.player_towns[1]) > 0)
                .then(|| usize::try_from(zone.ownership.wrapping_sub(1)).ok())
                .flatten()
                .and_then(|position| input.positions.get(position).copied().flatten())
                .and_then(|colour| input.towns.get(usize::from(colour)).copied().flatten());
            if let Some(town) = choice {
                town_type = i32::from(town);
            }
            layout.gens.push(ZoneGenerator {
                zone: me,
                town_type,
                town_type_2: -1,
                terrain: 0,
                position: [0, 0, 0],
                size: zone.base_size,
            });
        }
        layout.place(stream, me, if k < n { me } else { n }, d)?;
    }
    // The scaling (RMG-ZON-006): the box over every zone from 0, its larger side the span.
    let mut lo = [0i32; 2];
    let mut hi = [0i32; 2];
    for k in 0..n {
        let b = layout.base(k);
        for axis in 0..2 {
            let v = layout.gens[k].position[axis];
            lo[axis] = lo[axis].min(v.wrapping_sub(b));
            hi[axis] = hi[axis].max(v.wrapping_add(b).wrapping_add(1));
        }
    }
    let span = hi[0].wrapping_sub(lo[0]).max(hi[1].wrapping_sub(lo[1]));
    let origin = [0, 1].map(|axis| lo[axis].wrapping_sub(span).wrapping_add(hi[axis]) / 2);
    let scaled = |v: i32| v.wrapping_mul(input.side).checked_div(span);
    for k in 0..n {
        let g = layout.gens[k];
        let zone = &input.zones[g.zone];
        let (Some(x), Some(y), Some(size)) = (
            scaled(g.position[0].wrapping_sub(origin[0])),
            scaled(g.position[1].wrapping_sub(origin[1])),
            scaled(zone.base_size),
        ) else {
            return Err(GenerateError::LayoutSpan);
        };
        let mut g = ZoneGenerator {
            position: [x, y, g.position[2]],
            size,
            ..g
        };
        g.terrain = terrain_drawn(stream, zone, &g, &tables.native_terrains);
        let row = usize::try_from(g.terrain)
            .ok()
            .and_then(|t| tables.terrain_towns.get(t).copied())
            .unwrap_or([-1; TERRAIN_TOWNS]);
        g.town_type_2 = second_town_type(stream, &g, row, input.all_four, input.format);
        layout.gens[k] = g;
    }
    Ok(layout.gens)
}
