//! The game's terrain painter (survey `docs/rules/surveys/E14-generator.md` §2.7.1 to §2.7.3):
//! the game's own code outside the generator's range, which steps 3, 6 and 11 reach through one
//! level of the generator's map (`rmg_map_vtable` 0x6409CC's slots 1 and 4). A painter is made
//! for a terrain and the weight 4 (`terrain_painter_init` 0x5B7250), paints rectangles
//! (`terrain_painter_add_rect` 0x5B7690) and, at its end (`terrain_painter_finish` 0x5B72F0),
//! mends the cells it left pinched and fits every picture of the level to its neighbours. About a
//! third of a generation's draws are its pictures'.
//!
//! The original keeps a 16-bit copy of each cell it reads (`terrain_painter_cell_word`
//! 0x5B48D0); every write goes to both and nothing else writes the map while a painter works, so
//! the port reads the map itself: the copy's 4 bits of terrain and 7 of picture hold every value
//! the sets give (RMG-TER-006).

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use crate::cells::{Cell, Cells};
use crate::generate::{Stream, Trace};
use crate::part::{GeneratorTables, MAX_SHAPE, RockPicture};

/// `terrain_picture_draw` 0x5B3890's flag roll: `rand() % 100` below the share × weight / 8
/// takes a decorated plain picture.
pub const SITE_PICTURE_FLAG: u32 = 0x005B_38B9;
/// `terrain_picture_draw`'s picture: `rand() %` the plain or decorated pictures' count.
pub const SITE_PICTURE: u32 = 0x005B_38D3;
/// `terrain_picture_draw_kept` 0x5B38F0: `rand() %` a transition shape's pictures.
pub const SITE_PICTURE_SHAPE: u32 = 0x005B_3912;
/// `rock_picture_draw` 0x5B3AA0: `rand() %` rock's plain pictures.
pub const SITE_ROCK_PICTURE: u32 = 0x005B_3AB6;
/// `rock_picture_draw_kept` 0x5B3AE0: `rand() %` a rock shape's pictures of two variants.
pub const SITE_ROCK_PICTURE_SHAPE: u32 = 0x005B_3B2D;

/// Dirt: the terrain the lands meet through (`terrain_painter_border_class` 0x5B3E40).
const DIRT: u8 = 0;
/// Sand: borders nothing (`terrain_painter_border_class`).
const SAND: u8 = 1;
/// Water.
pub(crate) const WATER: u8 = 8;
/// Rock.
pub(crate) const ROCK: u8 = 9;

/// The weight every generator's painter is made with.
pub(crate) const WEIGHT: u32 = 4;

/// The ranges of the first kind: per shape a plain and a decorated one.
const RANGES: usize = (MAX_SHAPE as usize + 1) * 2;

/// The neighbours in the painters' order (`neighbour_offsets` 0x6A80A8): N, NE, E, SE, S, SW,
/// W, NW; even indexes orthogonal.
const RING: [[i32; 2]; 8] = [
    [0, -1],
    [1, -1],
    [1, 0],
    [1, 1],
    [0, 1],
    [-1, 1],
    [-1, 0],
    [-1, -1],
];

/// The ring's indexes by name.
const N: usize = 0;
const NE: usize = 1;
const E: usize = 2;
const SE: usize = 3;
const S: usize = 4;
const SW: usize = 5;
const W: usize = 6;
const NW: usize = 7;

/// The neighbour that plays each role of the ring under a mirroring
/// (`terrain_painter_mirror_roles` 0x642C00), by its word's W-E byte × 2 + its N-S byte.
const ROLES: [[usize; 8]; 4] = [
    [0, 1, 2, 3, 4, 5, 6, 7],
    [4, 3, 2, 1, 0, 7, 6, 5],
    [0, 7, 6, 5, 4, 3, 2, 1],
    [4, 5, 6, 7, 0, 1, 2, 3],
];

/// The mirrorings in the order the shape rule tries them (`terrain_painter_mirror_words`
/// 0x6A52B8): none, N-S, W-E, both; a word is (W-E, N-S).
const MIRRORINGS: [[bool; 2]; 4] = [[false, false], [false, true], [true, false], [true, true]];

/// The cells past a corner's arms (`terrain_painter_corner_arm_offsets` 0x6A5260), by the word's
/// N-S byte × 2 + its W-E byte: unmirrored SW and NE.
const ARMS: [[[i32; 2]; 2]; 4] = [
    [[-1, 1], [1, -1]],
    [[1, 1], [-1, -1]],
    [[-1, -1], [1, 1]],
    [[1, -1], [-1, 1]],
];

/// Two steps toward a bordering diagonal (`terrain_painter_corner_far_offsets` 0x6A3D68), by
/// the word's N-S byte × 2 + its W-E byte.
const FAR: [[i32; 2]; 4] = [[2, 2], [-2, 2], [2, -2], [-2, -2]];

/// What the original's painter does not survive (RMG-QUIRK-3, -4): never reached by the
/// corpus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaintFault {
    /// A picture drawn among none: `rand() % 0` (a shape its terrain's set lacks: water's 24,
    /// rock's 16 and 24, RMG-TER-018).
    NoPicture {
        /// The terrain.
        terrain: u8,
        /// The shape.
        shape: u8,
    },
    /// A pinched cell the finish mends nothing at: its loop would never end.
    Stuck,
}

/// One terrain's picture set as `terrain_picture_set_init` 0x5B3780 (the first kind) or
/// `rock_picture_ranges_init` 0x5B3940 (rock's) builds it: its pictures and, per shape and flag
/// (rock: per shape and variants), the first picture of its last run and the count of all.
#[derive(Debug, Clone)]
struct Set {
    through_dirt: bool,
    land: bool,
    share: u32,
    rock: bool,
    /// Per picture its shape, flag and variants.
    pictures: Vec<(u8, bool, [bool; 2])>,
    /// (first, count) by `shape × 2 + flag`, rock's by `(shape × 2 + a) × 2 + b`.
    ranges: Vec<(u32, u32)>,
}

impl Set {
    fn new(
        through_dirt: bool,
        land: bool,
        share: u8,
        rock: bool,
        pictures: Vec<(u8, bool, [bool; 2])>,
    ) -> Self {
        let size = if rock { 2 * RANGES } else { RANGES };
        let mut ranges = alloc::vec![(0u32, 0u32); size];
        let mut previous = None;
        for (i, &(shape, flag, [a, b])) in pictures.iter().enumerate() {
            let key = if rock {
                (usize::from(shape) * 2 + usize::from(a)) * 2 + usize::from(b)
            } else {
                usize::from(shape) * 2 + usize::from(flag)
            };
            let range = &mut ranges[key.min(size - 1)];
            if previous != Some(key) {
                range.0 = i as u32;
            }
            range.1 += 1;
            previous = Some(key);
        }
        Self {
            through_dirt,
            land,
            share: u32::from(share),
            rock,
            pictures,
            ranges,
        }
    }

    fn range(&self, key: usize) -> (u32, u32) {
        self.ranges[key.min(self.ranges.len() - 1)]
    }

    /// Slot 2: the picture is decorated (rock's never).
    fn flagged(&self, picture: u8) -> bool {
        self.pictures.get(usize::from(picture)).is_some_and(|p| p.1)
    }

    /// The picture's shape is `shape` (and, with `variants`, rock's variants are those).
    fn fits(&self, picture: u8, shape: u8, variants: Option<[bool; 2]>) -> bool {
        self.pictures
            .get(usize::from(picture))
            .is_some_and(|p| p.0 == shape && variants.is_none_or(|v| v == p.2))
    }
}

/// The painter's picture sets, dirt to lava, water, then rock.
#[derive(Debug, Clone)]
pub(crate) struct Pictures {
    sets: Vec<Set>,
}

impl Pictures {
    // rule: RMG-TER-006
    pub(crate) fn new(tables: &GeneratorTables) -> Self {
        let mut sets: Vec<Set> = (tables.picture_sets.iter())
            .map(|s| {
                let pictures = s
                    .pictures
                    .iter()
                    .map(|&(shape, flag)| (shape, flag, [false; 2]));
                Set::new(s.through_dirt, s.land, s.share, false, pictures.collect())
            })
            .collect();
        let rock = tables.rock_pictures.iter();
        let rock = rock.map(|&(shape, variants): &RockPicture| (shape, false, variants));
        sets.push(Set::new(false, false, 0, true, rock.collect()));
        Self { sets }
    }

    /// Terrain `t`'s set (`terrain_picture_sets` 0x642BD8); past rock, rock's.
    fn set(&self, t: u8) -> &Set {
        &self.sets[usize::from(t).min(self.sets.len() - 1)]
    }
}

/// The cell's terrain as the painter's copy keeps it (+0x24 bits 0-3).
const fn terrain_of(c: &Cell) -> u8 {
    (c.tile & 0xF) as u8
}

/// The cell's picture as the copy keeps it (+0x24 bits 6-12).
const fn picture_of(c: &Cell) -> u8 {
    ((c.tile >> 6) & 0x7F) as u8
}

/// A painter of one terrain on level `z` of the generator's map.
pub(crate) struct Painter<'a, 't, T: Trace> {
    pictures: &'a Pictures,
    cells: &'a mut Cells,
    stream: &'a mut Stream<'t, T>,
    z: i32,
    width: i32,
    height: i32,
    terrain: u8,
    weight: u32,
    /// `quirk.rmg.cosmetic` (RMG-QUIRK-21): the original's else-if per axis in the queue and
    /// group 3's inverted mirroring; off, both neighbours of an axis queued and group 3's 8s
    /// mirrored as matched.
    cosmetic: bool,
    /// Set 1 (+0x14): pinched cells of the terrain, by (y, x).
    pinched: BTreeSet<(i32, i32)>,
    /// Set 2 (+0x24): cells of other terrains to test again, by (y, x).
    queued: BTreeSet<(i32, i32)>,
    fault: Option<PaintFault>,
}

impl<'a, 't, T: Trace> Painter<'a, 't, T> {
    /// `terrain_painter_init(map, terrain, weight)` on level `z`.
    pub(crate) fn new(
        pictures: &'a Pictures,
        cells: &'a mut Cells,
        stream: &'a mut Stream<'t, T>,
        z: i32,
        terrain: u8,
        cosmetic: bool,
    ) -> Self {
        let (width, height, _) = cells.size();
        Self {
            pictures,
            cells,
            stream,
            z,
            width,
            height,
            terrain,
            weight: WEIGHT,
            cosmetic,
            pinched: BTreeSet::new(),
            queued: BTreeSet::new(),
            fault: None,
        }
    }

    fn fail(&mut self, fault: PaintFault) {
        self.fault = self.fault.or(Some(fault));
    }

    fn on_map(&self, x: i32, y: i32) -> bool {
        (0..self.width).contains(&x) && (0..self.height).contains(&y)
    }

    fn cell(&self, x: i32, y: i32) -> &Cell {
        self.cells.clamped(x, y, self.z)
    }

    /// The terrain of the cell at (x, y), clamped to the map on each axis.
    fn terrain_at(&self, x: i32, y: i32) -> u8 {
        terrain_of(self.cell(x, y))
    }

    /// Slot 1, `rmg_map_set_tile` 0x532190: the terrain, the picture and the mirror flags.
    fn write(&mut self, x: i32, y: i32, terrain: u8, picture: u8, word: [bool; 2]) {
        let c = self.cells.clamped_mut(x, y, self.z);
        c.tile = (c.tile & !0x3FFF) | u32::from(terrain & 0x3F) | (u32::from(picture) << 6);
        c.tile_data =
            (c.tile_data & !(3 << 15)) | (u32::from(word[0]) << 15) | (u32::from(word[1]) << 16);
    }

    /// `rand() %` the range's count plus its first (`div`); none is `rand() % 0`.
    fn pick(&mut self, range: (u32, u32), site: u32, terrain: u8, shape: u8) -> u8 {
        let value = self.stream.draw(site);
        if range.1 == 0 {
            self.fail(PaintFault::NoPicture { terrain, shape });
            return 0;
        }
        (value % range.1 + range.0) as u8
    }

    /// Slot 4 of terrain `t`'s set (`terrain_picture_draw` 0x5B3890, `rock_picture_draw`
    /// 0x5B3AA0): a current plain picture kept (the transition pass's test, before it reads the
    /// weight); else, with decorated pictures, `rand() % 100` below share × weight / 8 takes
    /// them, and `rand() %` the pictures' count.
    // rule: RMG-TER-007
    fn draw_plain(&mut self, t: u8, weight: u32) -> u8 {
        let pictures = self.pictures;
        let set = pictures.set(t);
        if set.rock {
            let range = set.range(0);
            return self.pick(range, SITE_ROCK_PICTURE, t, 0);
        }
        let (plain, decorated, share) = (set.range(0), set.range(1), set.share);
        let range = if decorated.1 == 0 {
            plain
        } else if self.stream.draw(SITE_PICTURE_FLAG) % 100 < (share * weight) >> 3 {
            decorated
        } else {
            plain
        };
        self.pick(range, SITE_PICTURE, t, 0)
    }

    /// Slot 5 (`terrain_picture_draw_kept` 0x5B38F0, `rock_picture_draw_kept` 0x5B3AE0): a
    /// current picture of the shape kept (rock: and of the word's variants), else `rand() %` the
    /// shape's plain pictures (rock: its variants'); the first kind keeps the word, rock clears
    /// it.
    // rule: RMG-TER-008
    fn draw_shape(&mut self, t: u8, shape: u8, word: [bool; 2], current: u8) -> (u8, [bool; 2]) {
        let pictures = self.pictures;
        let set = pictures.set(t);
        if set.rock {
            if set.fits(current, shape, Some(word)) {
                return (current, [false; 2]);
            }
            let key = (usize::from(shape) * 2 + usize::from(word[0])) * 2 + usize::from(word[1]);
            let range = set.range(key);
            return (
                self.pick(range, SITE_ROCK_PICTURE_SHAPE, t, shape),
                [false; 2],
            );
        }
        if set.fits(current, shape, None) {
            return (current, word);
        }
        let range = set.range(usize::from(shape) * 2);
        (self.pick(range, SITE_PICTURE_SHAPE, t, shape), word)
    }

    /// `terrain_painter_decor_weight` 0x5B6FD0: the painter's weight halved for each of the W,
    /// N, E, S neighbours on the map of terrain `t` whose picture is decorated.
    // rule: RMG-TER-010
    fn weight_at(&self, x: i32, y: i32, t: u8) -> u32 {
        let set = self.pictures.set(t);
        [[-1, 0], [0, -1], [1, 0], [0, 1]]
            .iter()
            .filter(|[dx, dy]| {
                let (u, v) = (x + dx, y + dy);
                self.on_map(u, v)
                    && self.terrain_at(u, v) == t
                    && set.flagged(picture_of(self.cell(u, v)))
            })
            .fold(self.weight, |w, _| w >> 1)
    }

    /// `terrain_painter_pinched_we` 0x5B6320: inside the map's columns, W and E of other terrains
    /// than `t`.
    // rule: RMG-TER-011
    fn pinched_we(&self, x: i32, y: i32, t: u8) -> bool {
        0 < x
            && x < self.width - 1
            && self.terrain_at(x - 1, y) != t
            && self.terrain_at(x + 1, y) != t
    }

    /// `terrain_painter_pinched_ns` 0x5B6430: the same vertically.
    // rule: RMG-TER-011
    fn pinched_ns(&self, x: i32, y: i32, t: u8) -> bool {
        0 < y
            && y < self.height - 1
            && self.terrain_at(x, y - 1) != t
            && self.terrain_at(x, y + 1) != t
    }

    /// `terrain_painter_ring_same` 0x5B6540: per neighbour in ring order, clamped on each axis,
    /// of the cell's own terrain; a diagonal only with one of its two orthogonals.
    fn ring_same(&self, x: i32, y: i32) -> [bool; 8] {
        let t = self.terrain_at(x, y);
        let mut same = [false; 8];
        for (k, [dx, dy]) in RING.iter().enumerate() {
            same[k] = self.terrain_at(x + dx, y + dy) == t;
        }
        for k in [NE, SE, SW, NW] {
            same[k] = same[k] && (same[k - 1] || same[(k + 1) & 7]);
        }
        same
    }

    /// `terrain_painter_ring_split` 0x5B6810: the same-terrain neighbours in two arcs or more.
    // rule: RMG-TER-011
    fn ring_split(&self, x: i32, y: i32) -> bool {
        let same = self.ring_same(x, y);
        (0..8).filter(|&k| same[k] && !same[(k + 7) & 7]).count() >= 2
    }

    /// Pinched against its own terrain: W-E, N-S or, for a set without byte +5 (water, rock),
    /// its ring split.
    fn pinched(&self, x: i32, y: i32) -> bool {
        let t = self.terrain_at(x, y);
        self.pinched_we(x, y, t)
            || self.pinched_ns(x, y, t)
            || (!self.pictures.set(t).land && self.ring_split(x, y))
    }

    /// `terrain_painter_queue_neighbours` 0x5B50F0: into set 2 the first of N, S on the map of
    /// another terrain, the first of W, E (RMG-QUIRK-21 off: each of the four), and each
    /// diagonal of another terrain whose set has no byte +5.
    // rule: RMG-TER-013
    fn queue(&mut self, x: i32, y: i32) {
        let t = self.terrain;
        let foreign = |p: &Self, u: i32, v: i32| p.on_map(u, v) && p.terrain_at(u, v) != t;
        for pair in [[[0, -1], [0, 1]], [[-1, 0], [1, 0]]] {
            let mut found = pair
                .into_iter()
                .filter(|[dx, dy]| foreign(self, x + dx, y + dy));
            let first = found.next();
            let second = if self.cosmetic { None } else { found.next() };
            for [dx, dy] in first.into_iter().chain(second) {
                self.queued.insert((y + dy, x + dx));
            }
        }
        for [dx, dy] in [[-1, -1], [1, -1], [-1, 1], [1, 1]] {
            let (u, v) = (x + dx, y + dy);
            if foreign(self, u, v) && !self.pictures.set(self.terrain_at(u, v)).land {
                self.queued.insert((v, u));
            }
        }
    }

    /// `terrain_painter_set_cell` 0x5B4B20: the cell drawn a plain picture and made the
    /// painter's terrain, out of set 2; on land its N and S neighbours in set 1 no longer pinched
    /// W-E and its W and E ones no longer pinched N-S leave set 1 and are queued; on water and
    /// rock each of its 8 neighbours of the terrain is tested again (leaving or entering set 1);
    /// then the cell itself into set 1 when pinched, else queued.
    // rule: RMG-TER-012
    fn set_cell(&mut self, x: i32, y: i32) {
        let t = self.terrain;
        let weight = self.weight_at(x, y, t);
        let picture = self.draw_plain(t, weight);
        self.write(x, y, t, picture, [false; 2]);
        self.queued.remove(&(y, x));
        if self.pictures.set(t).land {
            for ([dx, dy], we) in [
                ([0, -1], true),
                ([0, 1], true),
                ([-1, 0], false),
                ([1, 0], false),
            ] {
                let (u, v) = (x + dx, y + dy);
                if self.on_map(u, v) && self.pinched.contains(&(v, u)) {
                    let other = self.terrain_at(u, v);
                    let still = if we {
                        self.pinched_we(u, v, other)
                    } else {
                        self.pinched_ns(u, v, other)
                    };
                    if !still {
                        self.pinched.remove(&(v, u));
                        self.queue(u, v);
                    }
                }
            }
        } else {
            for [dx, dy] in RING {
                let (u, v) = (x + dx, y + dy);
                if !self.on_map(u, v) || self.terrain_at(u, v) != t {
                    continue;
                }
                let pinched = self.pinched(u, v);
                if self.pinched.contains(&(v, u)) {
                    if !pinched {
                        self.pinched.remove(&(v, u));
                        self.queue(u, v);
                    }
                } else if pinched {
                    self.pinched.insert((v, u));
                }
            }
        }
        if self.pinched(x, y) {
            self.pinched.insert((y, x));
        } else {
            self.queue(x, y);
        }
    }

    /// `terrain_painter_add_rect` 0x5B7690 (`terrain_painter_paint_rect` 0x5B4960): row by
    /// row, a cell of another terrain made the painter's, one of it drawn a new plain picture
    /// (its mirror flags cleared).
    // rule: RMG-TER-009
    pub(crate) fn add_rect(&mut self, x0: i32, y0: i32, w: i32, h: i32) {
        let t = self.terrain;
        for y in y0..y0 + h {
            for x in x0..x0 + w {
                if self.terrain_at(x, y) == t {
                    let weight = self.weight_at(x, y, t);
                    let picture = self.draw_plain(t, weight);
                    self.write(x, y, t, picture, [false; 2]);
                } else {
                    self.set_cell(x, y);
                }
            }
        }
    }

    /// `terrain_painter_resolve_pinch` 0x5B5440 for set 1's smallest cell: a N-S pinch mended
    /// by N or S, then a W-E one by W or E, then (water, rock) a split ring's gaps but the
    /// costliest, cheapest first. The number of cells made the terrain.
    // rule: RMG-TER-014
    fn resolve(&mut self, x: i32, y: i32) -> usize {
        let t = self.terrain;
        let mut changed = 0;
        for (along_x, first, second) in [(true, N, S), (false, W, E)] {
            let pinched = if along_x {
                self.pinched_ns(x, y, t)
            } else {
                self.pinched_we(x, y, t)
            };
            if !pinched {
                continue;
            }
            let at = |k: usize| (x + RING[k][0], y + RING[k][1]);
            let (a, b) = (at(first), at(second));
            // The diagonals beside the first neighbour and beside the second.
            let (near, far) = if along_x {
                ([NW, NE], [SW, SE])
            } else {
                ([NW, SW], [NE, SE])
            };
            let edge = if along_x {
                x == 0 || x == self.width - 1
            } else {
                y == 0 || y == self.height - 1
            };
            let of_t = |p: &Self, ks: [usize; 2]| {
                ks.iter().any(|&k| {
                    let (u, v) = at(k);
                    p.terrain_at(u, v) == t
                })
            };
            let pick = if self.pinched(a.0, a.1) {
                a
            } else if self.pinched(b.0, b.1) {
                b
            } else if edge || of_t(self, near) || !of_t(self, far) {
                a
            } else {
                b
            };
            self.set_cell(pick.0, pick.1);
            changed += 1;
        }
        if !self.pictures.set(t).land && self.ring_split(x, y) {
            changed += self.mend_ring(x, y);
        }
        changed
    }

    /// A split ring's gaps (runs of neighbours not flagged by `ring_same`, read once, listed in
    /// ring order from after the first flagged one; 2 an orthogonal, 1 a diagonal): while more
    /// than one is left, the cheapest's cells on the map made the terrain in ring order.
    fn mend_ring(&mut self, x: i32, y: i32) -> usize {
        let same = self.ring_same(x, y);
        let start = same.iter().position(|&s| s).unwrap_or(0);
        let mut gaps: Vec<(u32, Vec<usize>)> = Vec::new();
        let mut open = false;
        for step in 1..=8 {
            let k = (start + step) & 7;
            if same[k] {
                open = false;
                continue;
            }
            if !open {
                gaps.push((0, Vec::new()));
                open = true;
            }
            let last = gaps.len() - 1;
            gaps[last].0 += if k % 2 == 0 { 2 } else { 1 };
            gaps[last].1.push(k);
        }
        let mut changed = 0;
        while gaps.len() > 1 {
            let mut cheapest = 0;
            for (i, gap) in gaps.iter().enumerate() {
                if gap.0 < gaps[cheapest].0 {
                    cheapest = i;
                }
            }
            let (_, cells) = gaps.remove(cheapest);
            for k in cells {
                let (u, v) = (x + RING[k][0], y + RING[k][1]);
                if self.on_map(u, v) {
                    self.set_cell(u, v);
                    changed += 1;
                }
            }
        }
        changed
    }

    /// `terrain_painter_finish` 0x5B72F0: until both sets are empty, set 1's smallest cell
    /// resolved while set 1 holds one, then set 2's smallest taken out and, pinched against its
    /// own terrain, made the painter's, while set 2 holds one; then the last pass. The first
    /// fault met in the painter's life.
    // rule: RMG-TER-005
    pub(crate) fn finish(mut self) -> Result<(), PaintFault> {
        self.settle();
        self.fault.map_or(Ok(()), Err)
    }

    /// `terrain_painter_flush(terrain, 4)` 0x5B7520: the finish's loop and last pass, the painter
    /// kept (its sets drained; the original's cell words kept, a set-2 cell's loaded before its
    /// set's byte +5 is read, which the finish's word reads load too: the same); then its
    /// terrain is `terrain` and its weight 4, and it paints on.
    // rule: RMG-TER-005
    pub(crate) fn flush(&mut self, terrain: u8) {
        self.settle();
        self.terrain = terrain;
        self.weight = WEIGHT;
    }

    /// The finish's loop and last pass.
    fn settle(&mut self) {
        loop {
            while let Some(&(y, x)) = self.pinched.first() {
                if self.resolve(x, y) == 0 {
                    self.fail(PaintFault::Stuck);
                    self.pinched.remove(&(y, x));
                }
            }
            while let Some((y, x)) = self.queued.pop_first() {
                if self.pinched(x, y) {
                    self.set_cell(x, y);
                }
            }
            if self.pinched.is_empty() {
                break;
            }
        }
        self.transition_pass();
    }

    /// `terrain_painter_border_class` 0x5B3E40: how a cell of terrain `t` meets a neighbour of
    /// `n`: 0 the same or `t` sand; between two terrains that meet through dirt, 1 unless `t`
    /// is dirt (0); else 2.
    fn class(&self, t: u8, n: u8) -> u8 {
        if t == n || t == SAND {
            0
        } else if self.pictures.set(t).through_dirt && self.pictures.set(n).through_dirt {
            u8::from(t != DIRT)
        } else {
            2
        }
    }

    /// `terrain_painter_transition_pass` 0x5B5A70: every cell of the level row by row, its
    /// neighbours' classes (clamped on each axis), its shape and mirroring, the refinements, its
    /// picture (a shape: slot 5; none: slot 4 at its terrain's weight), written.
    // rule: RMG-TER-015, RMG-TER-016, RMG-TER-017, RMG-TER-018
    fn transition_pass(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let t = self.terrain_at(x, y);
                let mut classes = [0u8; 8];
                for (k, [dx, dy]) in RING.iter().enumerate() {
                    classes[k] = self.class(t, self.terrain_at(x + dx, y + dy));
                }
                let (shape, word) = shape(&classes, self.cosmetic);
                let shape = self.refine(x, y, t, shape, word);
                let current = picture_of(self.cell(x, y));
                let (picture, word) = if shape == 0 {
                    // Slot 4 keeps a current plain picture; only a drawn one reads the weight.
                    let picture = if self.pictures.set(t).fits(current, 0, None) {
                        current
                    } else {
                        let weight = self.weight_at(x, y, t);
                        self.draw_plain(t, weight)
                    };
                    (picture, [false; 2])
                } else {
                    self.draw_shape(t, shape, word, current)
                };
                self.write(x, y, t, picture, word);
            }
        }
    }

    /// `terrain_painter_corner_arm_same` 0x5B6BA0 and `terrain_painter_corner_far_differs`
    /// 0x5B6E00: 2 → 6 and 8 → 12 when a cell past the corner's arms has the cell's terrain; 5
    /// → 7 and 11 → 13 when the cell two columns or two rows toward the diagonal has another.
    // rule: RMG-TER-017
    fn refine(&self, x: i32, y: i32, t: u8, shape: u8, word: [bool; 2]) -> u8 {
        let index = usize::from(word[1]) * 2 + usize::from(word[0]);
        match shape {
            2 | 8 => {
                let same = ARMS[index]
                    .iter()
                    .any(|[dx, dy]| self.terrain_at(x + dx, y + dy) == t);
                if same { shape + 4 } else { shape }
            }
            5 | 11 => {
                let [dx, dy] = FAR[index];
                let differs = self.terrain_at(x + dx, y) != t || self.terrain_at(x, y + dy) != t;
                if differs { shape + 2 } else { shape }
            }
            _ => shape,
        }
    }
}

/// A condition of the shape rule: the classes its roles must have, the shape, whether the
/// mirroring is given inverted.
type Rule = (&'static [(usize, u8)], u8, bool);

/// `terrain_painter_transition_shape` 0x5B3E80: the ten groups in order, each tried under the
/// mirrorings none, N-S, W-E, both, its conditions in order; the first that holds gives the
/// shape and the mirroring (with `inverting`, RMG-QUIRK-21, group 3's 8s with both bytes
/// inverted), none shape 0.
// rule: RMG-TER-016
fn shape(classes: &[u8; 8], inverting: bool) -> (u8, [bool; 2]) {
    // A cell among its own terrain (every class 0, most cells of a level) meets no condition,
    // each asking for a class 1 or 2 (a unit test holds the table to it): no shape, at once.
    if *classes == [0; 8] {
        return (0, [false; 2]);
    }
    for group in GROUPS {
        for word in MIRRORINGS {
            let roles = &ROLES[usize::from(word[0]) * 2 + usize::from(word[1])];
            for &(all, shape, inverted) in group {
                if all.iter().all(|&(role, c)| classes[roles[role]] == c) {
                    let word = if inverted && inverting {
                        [!word[0], !word[1]]
                    } else {
                        word
                    };
                    return (shape, word);
                }
            }
        }
    }
    (0, [false; 2])
}

/// The shape rule's groups (survey §2.7.3), each condition's roles and classes.
const GROUPS: [&[Rule]; 10] = [
    &[
        (&[(E, 1), (S, 1), (NE, 2), (SW, 2)], 28, false),
        (&[(E, 1), (S, 1), (SE, 2)], 27, false),
    ],
    &[
        (&[(N, 1), (W, 1), (SE, 1)], 23, false),
        (&[(N, 1), (W, 1), (SE, 2)], 25, false),
        (&[(N, 2), (W, 2), (SE, 2)], 24, false),
        (&[(N, 2), (W, 2), (SE, 1)], 26, false),
    ],
    &[
        (&[(E, 2), (S, 1), (SW, 2)], 8, true),
        (&[(E, 2), (S, 1)], 21, false),
        (&[(E, 1), (S, 2), (NE, 2)], 8, true),
        (&[(E, 1), (S, 2)], 22, false),
    ],
    &[
        (&[(E, 1), (S, 1), (SW, 2)], 17, false),
        (&[(E, 1), (S, 1), (NE, 2)], 18, false),
    ],
    &[(&[(N, 1), (W, 1)], 2, false), (&[(N, 2), (W, 2)], 8, false)],
    &[
        (&[(E, 1), (SW, 2)], 17, false),
        (&[(S, 1), (NE, 2)], 18, false),
        (&[(E, 2), (SW, 1)], 21, false),
        (&[(S, 2), (NE, 1)], 22, false),
        (&[(W, 1), (NE, 1)], 2, false),
        (&[(N, 1), (SW, 1)], 2, false),
        (&[(W, 2), (NE, 2)], 8, false),
        (&[(N, 2), (SW, 2)], 8, false),
    ],
    &[
        (&[(E, 1), (SE, 2)], 19, false),
        (&[(S, 1), (SE, 2)], 20, false),
    ],
    &[
        (&[(N, 1)], 4, false),
        (&[(N, 2)], 10, false),
        (&[(W, 1)], 3, false),
        (&[(W, 2)], 9, false),
    ],
    &[
        (&[(NW, 1), (SE, 1)], 14, false),
        (&[(NW, 1), (SE, 2)], 15, false),
        (&[(NW, 2), (SE, 2)], 16, false),
    ],
    &[(&[(SE, 1)], 5, false), (&[(SE, 2)], 11, false)],
];

#[cfg(test)]
pub(crate) mod tests;
