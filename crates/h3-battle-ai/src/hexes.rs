//! The original's hex directions, which the computer's estimates walk (0x523D90, 0x523DF0,
//! 0x445840, 0x448AB0; `docs/formats/installation.md` §7.36, §7.37): six around a hex from top
//! right clockwise (0 top right, 1 right, 2 bottom right, 3 bottom left, 4 left, 5 top left), and
//! eight around a two-hex stack (0 to 2 around its right hex, 3 to 5 around its left hex, 6 and 7
//! the hexes above and below its middle). A stack's facing is its side's: the attacker's faces
//! right (the original's facing 1), the defender's left. The neighbours are the original's table
//! ([`table_neighbor`]): none in a side column.

use h3_battle::{BattleSide, Direction, Field, Hex, Stack};

/// The original's directions 0 to 5 (0x523D90's table, six words per hex).
const DIRECTIONS: [Direction; 6] = [
    Direction::TopRight,
    Direction::Right,
    Direction::BottomRight,
    Direction::BottomLeft,
    Direction::Left,
    Direction::TopLeft,
];

/// A two-hex stack's direction 0 to 7 to its place clockwise around it (0x660878).
const RING_PLACE: [u8; 8] = [0, 1, 2, 4, 5, 6, 7, 3];

/// A place clockwise around a two-hex stack to its direction (0x660898, the inverse).
const RING_DIRECTION: [u8; 8] = [0, 1, 2, 7, 3, 4, 5, 6];

/// A stack as the computer's geometry reads it: its two-hex flag and side (its facing).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Shape {
    /// Whether it covers two hexes.
    pub(crate) two_hex: bool,
    /// Its side (the attacker's faces right).
    pub(crate) side: BattleSide,
}

impl Shape {
    /// The shape of `s`.
    pub(crate) fn of(s: &Stack) -> Self {
        Self {
            two_hex: s.is_two_hex(),
            side: s.side,
        }
    }

    /// Whether it faces right (the original's facing 1).
    pub(crate) fn faces_right(self) -> bool {
        self.side == BattleSide::Attacker
    }

    /// Its other hex at the position `pos` (0x4466A0: the position + 1 facing right, − 1 facing
    /// left; the position itself for a one-hex stack or off the field).
    pub(crate) fn other(self, field: Field, pos: Hex) -> Hex {
        if self.two_hex {
            field
                .neighbor(pos, Field::front_direction(self.side))
                .unwrap_or(pos)
        } else {
            pos
        }
    }
}

/// The neighbour of `hex` in `dir` as the original's neighbour table has it (the battle manager's
/// +0x13468, filled at the battle's setup by 0x4640A0, which 0x523D90, 0x523DF0 and the hex
/// scores read; E23 T7, batch 4b H1): `None` off the field or in a side column, where only the war
/// machines' rear hexes stand (a side column's own hex has its neighbours in the next column).
pub(crate) fn table_neighbor(field: Field, hex: Hex, dir: Direction) -> Option<Hex> {
    field.neighbor(hex, dir).filter(|&n| field.is_standable(n))
}

/// The neighbour of `hex` in the original's direction `d` (0x523D90): 6 and 7 are 5 and 3 for a
/// stack facing right, 0 and 2 for one facing left; `None` off the field or in a side column
/// ([`table_neighbor`]).
pub(crate) fn neighbor(field: Field, shape: Shape, hex: Hex, d: u8) -> Option<Hex> {
    let d = match (d, shape.faces_right()) {
        (6, true) => 5,
        (6, false) => 0,
        (7, true) => 3,
        (7, false) => 2,
        (d, _) => d,
    };
    DIRECTIONS
        .get(usize::from(d))
        .and_then(|&dir| table_neighbor(field, hex, dir))
}

/// The neighbour of a stack of `shape` at the position `pos` in direction `d` (0x523DF0): a
/// two-hex stack's directions 0 to 2 and 6, 7 from its right hex, 3 to 5 from its left hex.
pub(crate) fn stack_neighbor(field: Field, shape: Shape, pos: Hex, d: u8) -> Option<Hex> {
    stack_neighbor_at(field, shape, i32::from(pos.0), d)
}

/// [`stack_neighbor`] at any hex index `hex` (0x523DF0 takes a hex number, not a position: the
/// hex scores ask it one hex beside a stack, 0x41FEF8): a two-hex stack's other hex is the index
/// ± 1 by its facing, as the original's arithmetic gives it (beyond a row's end the next row's
/// first hex); `None` for an index off the field.
pub(crate) fn stack_neighbor_at(field: Field, shape: Shape, hex: i32, d: u8) -> Option<Hex> {
    let from = match (shape.two_hex, shape.faces_right()) {
        (true, true) if d <= 2 || d >= 6 => hex + 1,
        (true, false) if d >= 3 => hex - 1,
        _ => hex,
    };
    u8::try_from(from)
        .ok()
        .filter(|&h| usize::from(h) < field.hex_count())
        .and_then(|h| neighbor(field, shape, Hex(h), d))
}

/// How many directions a stack of `shape` has (six, eight for a two-hex one).
pub(crate) const fn directions(shape: Shape) -> u8 {
    if shape.two_hex { 8 } else { 6 }
}

/// The first direction from a stack of `shape` at `pos` whose neighbour is the struck stack's hex
/// `t_hex` or its other hex there (`target`'s shape; 0x445840).
pub(crate) fn direction_to(
    field: Field,
    (shape, pos): (Shape, Hex),
    (target, t_hex): (Shape, Hex),
) -> Option<u8> {
    let t_other = target.other(field, t_hex);
    (0..directions(shape))
        .find(|&d| stack_neighbor(field, shape, pos, d).is_some_and(|n| n == t_hex || n == t_other))
}

/// The directions a multi-headed striker of `shape` at `pos` hits when it strikes the stack at
/// `t_hex` (0x448AB0), as a mask of bits: every one, or for the Cerberi (`three_headed`) the
/// direction that touches it ([`direction_to`]) and the two beside it.
pub(crate) fn pattern(
    field: Field,
    (shape, pos, three_headed): (Shape, Hex, bool),
    (target, t_hex): (Shape, Hex),
) -> u8 {
    if !three_headed {
        return if shape.two_hex { 0xFF } else { 0x3F };
    }
    // The striker stands next to the stack it strikes (the plan's hexes, §7.37): a direction
    // touches it, then the one counterclockwise and the one clockwise of it.
    direction_to(field, (shape, pos), (target, t_hex)).map_or(0, |d| {
        let (back, on) = if shape.two_hex {
            let place = RING_PLACE[usize::from(d)];
            (
                RING_DIRECTION[usize::from((place + 7) % 8)],
                RING_DIRECTION[usize::from((place + 1) % 8)],
            )
        } else {
            ((d + 5) % 6, (d + 1) % 6)
        };
        1 << d | 1 << back | 1 << on
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::vec::Vec;

    fn shape(two_hex: bool, side: BattleSide) -> Shape {
        Shape { two_hex, side }
    }

    #[test]
    fn the_directions_go_clockwise_from_top_right() {
        let field = Field::new(h3_content_battle::FieldParams {
            width: 17,
            height: 11,
            side_columns: 1,
        });
        let one = shape(false, BattleSide::Attacker);
        // Hex 5 on row 0: right 6, bottom right 23 and bottom left 22 (row 1 is shifted left).
        let around: Vec<Option<Hex>> = (0..6).map(|d| neighbor(field, one, Hex(5), d)).collect();
        assert_eq!(
            around,
            [
                None,
                Some(Hex(6)),
                Some(Hex(23)),
                Some(Hex(22)),
                Some(Hex(4)),
                None
            ]
        );
        // A two-hex attacker at 39 covers 39 and 40: its right side from 40, its left from 39,
        // above and below its middle 23 and 57.
        let two = shape(true, BattleSide::Attacker);
        let ring: Vec<Option<Hex>> = (0..8)
            .map(|d| stack_neighbor(field, two, Hex(39), d))
            .collect();
        let want = [24, 41, 58, 56, 38, 22, 23, 57].map(|h| Some(Hex(h)));
        assert_eq!(ring, want);
        // The defender's at 40 covers 40 and 39: the same ring.
        let defender = shape(true, BattleSide::Defender);
        let ring: Vec<Option<Hex>> = (0..8)
            .map(|d| stack_neighbor(field, defender, Hex(40), d))
            .collect();
        assert_eq!(ring, want);
        assert_eq!(neighbor(field, defender, Hex(23), 6), Some(Hex(6)));
        assert_eq!(neighbor(field, defender, Hex(23), 7), Some(Hex(40)));
        assert_eq!(neighbor(field, one, Hex(23), 8), None);
    }

    #[test]
    fn the_table_has_no_neighbour_in_a_side_column() {
        let field = Field::new(h3_content_battle::FieldParams {
            width: 17,
            height: 11,
            side_columns: 1,
        });
        let one = shape(false, BattleSide::Defender);
        let around = |hex: u8| -> Vec<Option<Hex>> {
            (0..6).map(|d| neighbor(field, one, Hex(hex), d)).collect()
        };
        // Hex 15 (row 0, column 15): two neighbours, 32 below and 14 (batch 4b H1, read from the
        // original's table in a running battle); 16 and 33 lie in the side column.
        assert_eq!(
            around(15),
            [None, None, None, Some(Hex(32)), Some(Hex(14)), None]
        );
        // Hex 9: four.
        assert_eq!(
            around(9),
            [None, Some(Hex(10)), Some(Hex(27)), Some(Hex(26)), Some(Hex(8)), None]
        );
        // A side column's own hexes have their neighbours in the next column only: 34 (row 2,
        // shifted right) three, 17 (row 1) one.
        assert_eq!(
            around(34),
            [Some(Hex(18)), Some(Hex(35)), Some(Hex(52)), None, None, None]
        );
        assert_eq!(around(17), [None, Some(Hex(18)), None, None, None, None]);
        // The defender's two-hex war machine at 169 (its rear in the side column, its front 168):
        // its ring has no hex in column 16.
        let machine = shape(true, BattleSide::Defender);
        let ring: Vec<Option<Hex>> = (0..8)
            .map(|d| stack_neighbor(field, machine, Hex(169), d))
            .collect();
        assert!(
            ring.iter()
                .flatten()
                .all(|h| field.coords(*h).0 != 16 && field.contains(*h))
        );
        assert_eq!(table_neighbor(field, Hex(168), Direction::Right), None);
        assert_eq!(
            table_neighbor(field, Hex(169), Direction::Left),
            Some(Hex(168))
        );
    }

    #[test]
    fn a_cerberus_hits_three_directions_a_hydra_all() {
        let field = Field::new(h3_content_battle::FieldParams {
            width: 17,
            height: 11,
            side_columns: 1,
        });
        let one = shape(false, BattleSide::Attacker);
        let two = shape(true, BattleSide::Attacker);
        let target = shape(false, BattleSide::Defender);
        // A one-hex striker at 39 strikes 40 (direction 1): directions 0, 1 and 2.
        assert_eq!(
            direction_to(field, (one, Hex(39)), (target, Hex(40))),
            Some(1)
        );
        assert_eq!(
            pattern(field, (one, Hex(39), true), (target, Hex(40))),
            0b111
        );
        assert_eq!(
            pattern(field, (one, Hex(39), false), (target, Hex(40))),
            0x3F
        );
        // Struck at direction 0, the one counterclockwise is 5.
        assert_eq!(
            pattern(field, (one, Hex(39), true), (target, Hex(23))),
            0b10_0011
        );
        // A two-hex Cerberus at 39 striking 23 above its middle (direction 6): its places 7, 6 and
        // 0, the directions 6, 5 and 0.
        assert_eq!(
            direction_to(field, (two, Hex(39)), (target, Hex(23))),
            Some(6)
        );
        assert_eq!(
            pattern(field, (two, Hex(39), true), (target, Hex(23))),
            0b110_0001
        );
        // Striking 57 below its middle (direction 7, place 3): directions 2, 7 and 3.
        assert_eq!(
            pattern(field, (two, Hex(39), true), (target, Hex(57))),
            0b1000_1100
        );
        assert_eq!(
            pattern(field, (two, Hex(39), false), (target, Hex(57))),
            0xFF
        );
        // A two-hex target's other hex counts: a defender at 42 covers 41 beside the striker.
        let wide = shape(true, BattleSide::Defender);
        assert_eq!(
            direction_to(field, (two, Hex(39)), (wide, Hex(42))),
            Some(1)
        );
        // Nothing touches it: no direction, an empty pattern.
        assert_eq!(
            direction_to(field, (one, Hex(39)), (target, Hex(100))),
            None
        );
        assert_eq!(pattern(field, (one, Hex(39), true), (target, Hex(100))), 0);
    }
}
