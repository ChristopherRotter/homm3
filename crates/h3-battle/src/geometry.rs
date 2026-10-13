//! Battlefield geometry (BAT-GEO-001, BAT-GEO-002).
//!
//! This module is the geometry seam of the battle context: the rest of the context only uses
//! [`Hex`], [`HexSet`], [`Footprint`] and the [`Field`] API (validity, standability, neighbours,
//! distance, footprints of one- and two-hex stacks, attack positions).
//!
//! Layout: `width × height` hexes in row-major order, `hex = x + width·y`. Rows are horizontal;
//! even rows (0-based) are shifted half a hex to the right. The `side_columns` leftmost and
//! rightmost columns exist (the original keeps heroes and war machines there) but no stack can
//! stand in them.

use h3_content_battle::{FieldParams, HexStep};
use h3_contracts::BattleSide;
use h3_kernel::{CanonicalHash, StateHasher};

/// A hex of the battlefield (row-major index).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Hex(pub u8);

impl Hex {
    /// The raw index.
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

impl CanonicalHash for Hex {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(self.0);
    }
}

/// A set of hexes (one bit per hex index; every `u8` hex fits).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HexSet([u64; 4]);

impl HexSet {
    /// The empty set.
    pub const fn new() -> Self {
        Self([0; 4])
    }

    /// Adds `hex`.
    pub fn insert(&mut self, hex: Hex) {
        self.0[usize::from(hex.0 >> 6)] |= 1 << (hex.0 & 63);
    }

    /// Whether `hex` is in the set.
    pub const fn contains(self, hex: Hex) -> bool {
        self.0[(hex.0 >> 6) as usize] & (1 << (hex.0 & 63)) != 0
    }

    /// Number of hexes in the set.
    pub const fn len(self) -> u32 {
        self.0[0].count_ones()
            + self.0[1].count_ones()
            + self.0[2].count_ones()
            + self.0[3].count_ones()
    }

    /// Whether the set is empty.
    pub const fn is_empty(self) -> bool {
        self.0[0] | self.0[1] | self.0[2] | self.0[3] == 0
    }

    /// The hexes in either set.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self([
            self.0[0] | other.0[0],
            self.0[1] | other.0[1],
            self.0[2] | other.0[2],
            self.0[3] | other.0[3],
        ])
    }

    /// The hexes of this set that are not in `other`.
    #[must_use]
    pub const fn without(self, other: Self) -> Self {
        Self([
            self.0[0] & !other.0[0],
            self.0[1] & !other.0[1],
            self.0[2] & !other.0[2],
            self.0[3] & !other.0[3],
        ])
    }

    /// The hexes in ascending order.
    pub fn iter(self) -> impl Iterator<Item = Hex> {
        (0..=u8::MAX).map(Hex).filter(move |h| self.contains(*h))
    }
}

impl CanonicalHash for HexSet {
    /// Encoding: the four 64-bit words, lowest hexes first.
    fn hash_into(&self, h: &mut StateHasher) {
        for w in self.0 {
            h.write_u64(w);
        }
    }
}

/// The hexes a stack covers: the hex its position names and, for two-hex stacks, the other hex
/// in front of it — right of the position for the attacker, left of it for the defender. A
/// two-hex stack's position is its rear hex, as in the original (stack +0x38, BAT-GEO-002).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Footprint {
    /// The position: a one-hex stack's hex, a two-hex stack's rear hex.
    pub position: Hex,
    /// A two-hex stack's other hex, in front of its position.
    pub other: Option<Hex>,
}

impl Footprint {
    /// The front hex, the one the stack faces the enemy with: a two-hex stack's other hex, a
    /// one-hex stack's position.
    pub const fn front(self) -> Hex {
        match self.other {
            Some(h) => h,
            None => self.position,
        }
    }

    /// A two-hex stack's rear hex (its position); `None` for a one-hex stack.
    pub const fn rear(self) -> Option<Hex> {
        match self.other {
            Some(_) => Some(self.position),
            None => None,
        }
    }

    /// The covered hexes, the front hex first (the order walks over them keep: a fire obstacle
    /// or a mine under the front hex comes first, BAT-MAG-028, BAT-MAG-033).
    pub fn hexes(self) -> impl Iterator<Item = Hex> {
        core::iter::once(self.front()).chain(self.rear())
    }

    /// Whether the footprint covers `hex`.
    pub fn covers(self, hex: Hex) -> bool {
        self.position == hex || self.other == Some(hex)
    }
}

/// Number of canonical attack positions around a target ([`Field::attack_positions`]).
pub const ATTACK_SLOTS: usize = 24;

/// The six neighbour directions, clockwise from the right.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Same row, next column.
    Right,
    /// Row below, towards the right.
    BottomRight,
    /// Row below, towards the left.
    BottomLeft,
    /// Same row, previous column.
    Left,
    /// Row above, towards the left.
    TopLeft,
    /// Row above, towards the right.
    TopRight,
}

impl Direction {
    /// All directions, in a fixed order (also the order of [`Field::neighbors`]).
    pub const ALL: [Self; 6] = [
        Self::Right,
        Self::BottomRight,
        Self::BottomLeft,
        Self::Left,
        Self::TopLeft,
        Self::TopRight,
    ];

    /// The direction of a content step (an obstacle spell's shape, BAT-MAG-027).
    pub const fn from_step(step: HexStep) -> Self {
        match step {
            HexStep::Right => Self::Right,
            HexStep::BottomRight => Self::BottomRight,
            HexStep::BottomLeft => Self::BottomLeft,
            HexStep::Left => Self::Left,
            HexStep::TopLeft => Self::TopLeft,
            HexStep::TopRight => Self::TopRight,
        }
    }

    /// This direction turned `steps` sixths of a full turn clockwise.
    #[must_use]
    pub const fn turned(self, steps: u8) -> Self {
        Self::ALL[(self as usize + steps as usize) % 6]
    }
}

/// The battlefield shape of a ruleset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Field {
    width: u8,
    height: u8,
    side_columns: u8,
}

impl Field {
    /// The field described by compiled parameters (already validated by the ruleset compiler:
    /// at most 255 hexes, at least two standable columns).
    pub const fn new(params: FieldParams) -> Self {
        Self {
            width: params.width,
            height: params.height,
            side_columns: params.side_columns,
        }
    }

    /// First column a stack may stand in (the attacker's starting column).
    pub const fn first_standable_column(self) -> i32 {
        self.side_columns as i32
    }

    /// Last column a stack may stand in (the defender's starting column).
    pub const fn last_standable_column(self) -> i32 {
        self.width as i32 - 1 - self.side_columns as i32
    }

    /// Number of hexes.
    pub const fn hex_count(self) -> usize {
        self.width as usize * self.height as usize
    }

    /// The hex at column `x`, row `y`, if inside the field.
    // rule: BAT-GEO-001
    pub fn hex(self, x: i32, y: i32) -> Option<Hex> {
        if x < 0 || y < 0 || x >= i32::from(self.width) || y >= i32::from(self.height) {
            return None;
        }
        u8::try_from(x + i32::from(self.width) * y).ok().map(Hex)
    }

    /// Column and row of a hex.
    pub const fn coords(self, hex: Hex) -> (i32, i32) {
        let w = self.width as i32;
        let i = hex.0 as i32;
        (i % w, i / w)
    }

    /// Whether `hex` is inside the field.
    pub const fn contains(self, hex: Hex) -> bool {
        hex.index() < self.hex_count()
    }

    /// Whether a stack may stand on `hex` (inside the field, not a side column).
    // rule: BAT-GEO-001
    pub const fn is_standable(self, hex: Hex) -> bool {
        if !self.contains(hex) {
            return false;
        }
        let (x, _) = self.coords(hex);
        x >= self.side_columns as i32 && x < self.width as i32 - self.side_columns as i32
    }

    /// The neighbour of `hex` in `dir`, if inside the field.
    // rule: BAT-GEO-001
    pub fn neighbor(self, hex: Hex, dir: Direction) -> Option<Hex> {
        let (x, y) = self.coords(hex);
        // Even rows are shifted right: their upper/lower neighbours are at x and x + 1; odd rows'
        // are at x - 1 and x.
        let shift = if y % 2 == 0 { 0 } else { -1 };
        let (nx, ny) = match dir {
            Direction::Right => (x + 1, y),
            Direction::Left => (x - 1, y),
            Direction::TopLeft => (x + shift, y - 1),
            Direction::TopRight => (x + shift + 1, y - 1),
            Direction::BottomLeft => (x + shift, y + 1),
            Direction::BottomRight => (x + shift + 1, y + 1),
        };
        self.hex(nx, ny)
    }

    /// The direction from `from` to its neighbour `to`, if they are neighbours.
    pub fn direction(self, from: Hex, to: Hex) -> Option<Direction> {
        Direction::ALL
            .into_iter()
            .find(|&d| self.neighbor(from, d) == Some(to))
    }

    /// All neighbours of `hex`, in [`Direction::ALL`] order. Computed from one `coords` and
    /// written out rather than mapped over the directions, which keeps this hot path (movement)
    /// independent of the inlining of [`Field::neighbor`] (BACKLOG, codegen swings).
    pub fn neighbors(self, hex: Hex) -> [Option<Hex>; 6] {
        let (x, y) = self.coords(hex);
        // As in `neighbor`.
        let shift = if y % 2 == 0 { 0 } else { -1 };
        [
            self.hex(x + 1, y),
            self.hex(x + shift + 1, y + 1),
            self.hex(x + shift, y + 1),
            self.hex(x - 1, y),
            self.hex(x + shift, y - 1),
            self.hex(x + shift + 1, y - 1),
        ]
    }

    /// Hex distance (number of steps between hexes on an empty field).
    // rule: BAT-GEO-001
    pub const fn distance(self, a: Hex, b: Hex) -> u32 {
        // Axial coordinates with X = x + floor(y / 2): neighbours differ by (±1, 0), (0, ±1),
        // (+1, +1) or (−1, −1).
        let (ax, ay) = self.coords(a);
        let (bx, by) = self.coords(b);
        let dx = (bx + by / 2) - (ax + ay / 2);
        let dy = by - ay;
        if (dx >= 0) == (dy >= 0) {
            let (p, q) = (dx.unsigned_abs(), dy.unsigned_abs());
            if p > q { p } else { q }
        } else {
            dx.unsigned_abs() + dy.unsigned_abs()
        }
    }

    /// Whether two hexes are neighbours.
    pub const fn are_adjacent(self, a: Hex, b: Hex) -> bool {
        self.distance(a, b) == 1
    }

    /// The direction from a two-hex stack's position (its rear hex) to its other hex, towards
    /// the enemy: right for the attacker, left for the defender.
    pub const fn front_direction(side: BattleSide) -> Direction {
        match side {
            BattleSide::Attacker => Direction::Right,
            BattleSide::Defender => Direction::Left,
        }
    }

    /// The footprint of a stack of `side` whose position is `position` (a two-hex stack's rear
    /// hex), or `None` if the position or a two-hex stack's other hex is off the field.
    // rule: BAT-GEO-002
    pub fn footprint(self, position: Hex, two_hex: bool, side: BattleSide) -> Option<Footprint> {
        if !self.contains(position) {
            return None;
        }
        if !two_hex {
            return Some(Footprint {
                position,
                other: None,
            });
        }
        self.neighbor(position, Self::front_direction(side))
            .map(|other| Footprint {
                position,
                other: Some(other),
            })
    }

    /// The footprint of a stack of `side` whose front hex ([`Footprint::front`]) is `front`, or
    /// `None` if a hex of it is off the field.
    pub fn footprint_at_front(
        self,
        front: Hex,
        two_hex: bool,
        side: BattleSide,
    ) -> Option<Footprint> {
        // The position lies behind the front hex: the opposite direction, half a turn.
        let position = if two_hex {
            self.neighbor(front, Self::front_direction(side).turned(3))?
        } else {
            front
        };
        self.footprint(position, two_hex, side)
    }

    /// Smallest distance between a hex of `a` and a hex of `b`.
    pub fn footprint_distance(self, a: Footprint, b: Footprint) -> u32 {
        a.hexes()
            .flat_map(|x| b.hexes().map(move |y| (x, y)))
            .map(|(x, y)| self.distance(x, y))
            .min()
            .unwrap_or(u32::MAX)
    }

    /// Whether `a` and `b` touch: they do not overlap and some hexes are neighbours
    /// (BAT-ATK-001).
    pub fn touches(self, a: Footprint, b: Footprint) -> bool {
        self.footprint_distance(a, b) == 1
    }

    /// The canonical attack positions of an attacker (`two_hex`, `side`) against the footprint
    /// `target`: positions from which the attacker's footprint touches the target. Slot
    /// `k = (i · 6 + d) · 2 + v`: `i` is the target hex (0 its front hex, 1 a two-hex target's
    /// rear hex), `d` the direction ([`Direction::ALL`]) from it to the contact hex `n`, `v = 0`
    /// the attacker's front hex on `n`, `v = 1` its rear hex on `n` (two-hex attackers only). A
    /// slot is `None` where the position is off the field, overlaps the target, or repeats an
    /// earlier slot, so every position appears once. Occupancy and reach are not considered.
    pub fn attack_positions(
        self,
        two_hex: bool,
        side: BattleSide,
        target: Footprint,
    ) -> [Option<Hex>; ATTACK_SLOTS] {
        let mut out = [None; ATTACK_SLOTS];
        let target_hexes = [Some(target.front()), target.rear()];
        // Positions found so far (every position is kept in its first slot only).
        let mut seen = [Hex(0); ATTACK_SLOTS];
        let mut seen_len = 0;
        for (k, slot) in out.iter_mut().enumerate() {
            let (i, d, v) = (k / 12, (k / 2) % 6, k % 2);
            if v == 1 && !two_hex {
                continue;
            }
            let Some(contact) = target_hexes[i].and_then(|t| self.neighbor(t, Direction::ALL[d]))
            else {
                continue;
            };
            // The contact hex neighbours the target, so the footprint touches it unless they
            // overlap (for `v = 1` the footprint's rear hex, its position, is the contact hex).
            let footprint = if v == 0 {
                self.footprint_at_front(contact, two_hex, side)
            } else {
                self.footprint(contact, two_hex, side)
            };
            let position = footprint
                .filter(|fp| !fp.hexes().any(|h| target.covers(h)))
                .map(|fp| fp.position);
            if let Some(p) = position.filter(|p| !seen[..seen_len].contains(p)) {
                seen[seen_len] = p;
                seen_len += 1;
                *slot = Some(p);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIELD: Field = Field::new(FieldParams {
        width: 17,
        height: 11,
        side_columns: 1,
    });

    fn at(x: i32, y: i32) -> Hex {
        FIELD.hex(x, y).unwrap()
    }

    // covers: BAT-GEO-001
    #[test]
    fn layout_and_standability() {
        assert_eq!(FIELD.hex_count(), 187);
        assert_eq!(at(3, 2), Hex(37));
        assert_eq!(FIELD.coords(Hex(37)), (3, 2));
        assert_eq!(FIELD.hex(17, 0), None);
        assert_eq!(FIELD.hex(0, 11), None);
        assert_eq!(FIELD.hex(-1, 0), None);
        assert_eq!(FIELD.hex(0, -1), None);
        assert!(FIELD.contains(Hex(186)));
        assert!(!FIELD.contains(Hex(187)));
        assert!(!FIELD.is_standable(Hex(200)));
        assert!(!FIELD.is_standable(at(0, 5)));
        assert!(!FIELD.is_standable(at(16, 5)));
        assert!(FIELD.is_standable(at(1, 5)));
        assert!(FIELD.is_standable(at(15, 5)));
        assert_eq!(Hex(9).index(), 9);
        assert_eq!(FIELD.first_standable_column(), 1);
        assert_eq!(FIELD.last_standable_column(), 15);
    }

    // covers: BAT-GEO-001
    #[test]
    fn neighbours_on_even_and_odd_rows() {
        // Even row: upper/lower neighbours at x and x + 1.
        assert_eq!(
            FIELD.neighbors(at(5, 4)),
            [
                Some(at(6, 4)),
                Some(at(6, 5)),
                Some(at(5, 5)),
                Some(at(4, 4)),
                Some(at(5, 3)),
                Some(at(6, 3))
            ]
        );
        // Odd row: upper/lower neighbours at x - 1 and x.
        assert_eq!(
            FIELD.neighbors(at(5, 3)),
            [
                Some(at(6, 3)),
                Some(at(5, 4)),
                Some(at(4, 4)),
                Some(at(4, 3)),
                Some(at(4, 2)),
                Some(at(5, 2))
            ]
        );
        // Corners lose neighbours.
        assert_eq!(
            FIELD.neighbors(at(0, 0)),
            [
                Some(at(1, 0)),
                Some(at(1, 1)),
                Some(at(0, 1)),
                None,
                None,
                None
            ]
        );
        assert_eq!(FIELD.neighbor(at(16, 10), Direction::Right), None);
        // `neighbors` agrees with `neighbor` everywhere.
        for i in 0..FIELD.hex_count() {
            let hex = Hex(u8::try_from(i).unwrap());
            assert_eq!(
                FIELD.neighbors(hex),
                Direction::ALL.map(|d| FIELD.neighbor(hex, d))
            );
        }
    }

    // covers: BAT-GEO-001
    #[test]
    fn distances() {
        let origin = at(5, 4);
        for n in FIELD.neighbors(origin).into_iter().flatten() {
            assert_eq!(FIELD.distance(origin, n), 1);
            assert!(FIELD.are_adjacent(n, origin));
        }
        assert_eq!(FIELD.distance(origin, origin), 0);
        assert_eq!(FIELD.distance(at(1, 0), at(15, 0)), 14);
        assert_eq!(FIELD.distance(at(1, 0), at(1, 10)), 10);
        // Moving down-right diagonally changes x every other row.
        assert_eq!(FIELD.distance(at(1, 0), at(6, 10)), 10);
        assert_eq!(FIELD.distance(at(1, 0), at(15, 10)), 19);
        assert_eq!(FIELD.distance(at(15, 10), at(1, 0)), 19);
        assert_eq!(FIELD.distance(at(15, 0), at(1, 10)), 14 + 10 - 5);
        assert!(!FIELD.are_adjacent(at(1, 0), at(3, 0)));
    }

    // covers: BAT-GEO-001
    #[test]
    fn distance_matches_breadth_first_search() {
        // On an empty field, distance equals the BFS step count for every pair from a few origins.
        for origin in [at(0, 0), at(8, 5), at(16, 10), at(3, 7)] {
            let mut dist = [u32::MAX; 187];
            let mut queue = [Hex(0); 187];
            let (mut head, mut tail) = (0, 1);
            queue[0] = origin;
            dist[origin.index()] = 0;
            while head < tail {
                let h = queue[head];
                head += 1;
                for n in FIELD.neighbors(h).into_iter().flatten() {
                    if dist[n.index()] == u32::MAX {
                        dist[n.index()] = dist[h.index()] + 1;
                        queue[tail] = n;
                        tail += 1;
                    }
                }
            }
            for i in 0..187u8 {
                assert_eq!(
                    FIELD.distance(origin, Hex(i)),
                    dist[usize::from(i)],
                    "{origin:?} -> {i}"
                );
            }
        }
    }

    #[test]
    fn hash_encoding() {
        let mut a = StateHasher::new();
        Hex(7).hash_into(&mut a);
        let mut b = StateHasher::new();
        b.write_u8(7);
        assert_eq!(a.finish(), b.finish());
    }
}
