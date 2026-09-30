//! Battlefield geometry (BAT-GEO-001).
//!
//! This module is the geometry seam of the battle context: the rest of the context only uses
//! [`Hex`] and the [`Field`] API (validity, standability, neighbours, distance). E03 can replace
//! the internals (obstacles, two-hex creatures, siege walls) without touching callers.
//!
//! Layout: `width × height` hexes in row-major order, `hex = x + width·y`. Rows are horizontal;
//! even rows (0-based) are shifted half a hex to the right. The `side_columns` leftmost and
//! rightmost columns exist (the original keeps heroes and war machines there) but no stack can
//! stand in them.

use h3_content::FieldParams;
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

    /// All neighbours of `hex`, in [`Direction::ALL`] order.
    pub fn neighbors(self, hex: Hex) -> [Option<Hex>; 6] {
        Direction::ALL.map(|d| self.neighbor(hex, d))
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
