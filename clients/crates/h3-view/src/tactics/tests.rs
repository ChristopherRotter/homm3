use super::*;
use alloc::vec::Vec;

use crate::record::tests::observation;
use crate::scene::tests::stack;

/// Hex `column` of `row`.
const fn hex(row: u8, column: u8) -> u8 {
    row * COLUMNS + column
}

/// Every hex of the columns `from..=to`.
fn columns(from: u8, to: u8) -> BTreeSet<u8> {
    (0..11)
        .flat_map(|row| (from..=to).map(move |column| hex(row, column)))
        .collect()
}

/// A battle in the attacker's tactics phase of `columns` columns, `stacks` on the field, the
/// first of them active.
fn phase(columns: u8, stacks: Vec<Stack>) -> Observation {
    let mut o = observation();
    o.sides[0].human = true;
    o.sides[1].human = true;
    o.tactics = Some(Tactics {
        side: Side::Attacker,
        columns,
    });
    o.active = stacks.first().map(|s| s.id);
    o.stacks = stacks;
    o
}

#[test]
fn the_zone_is_the_sides_columns_at_its_edge() {
    let attacker = Tactics {
        side: Side::Attacker,
        columns: 3,
    };
    let defender = Tactics {
        side: Side::Defender,
        columns: 3,
    };
    let zone = |t: &Tactics| {
        (0..COLUMNS)
            .filter(|&c| in_zone(t, hex(4, c)))
            .collect::<Vec<_>>()
    };
    assert_eq!(zone(&attacker), [1, 2, 3]);
    assert_eq!(zone(&defender), [13, 14, 15]);
    // BAT-TAC-001's widest zone: seven columns.
    let widest = Tactics {
        side: Side::Defender,
        columns: 7,
    };
    assert_eq!(zone(&widest), [9, 10, 11, 12, 13, 14, 15]);
}

#[test]
fn nothing_is_shaded_outside_the_phase_or_without_its_stack() {
    let walker = stack(0, Side::Attacker, 0, hex(5, 1), false);
    let mut o = phase(3, alloc::vec![walker.clone()]);
    o.tactics = None;
    assert!(shade(&o).is_empty());
    let mut o = phase(3, alloc::vec![walker]);
    o.active = None;
    assert!(shade(&o).is_empty());
    o.active = Some(0);
    o.stacks[0].alive = false;
    assert!(shade(&o).is_empty());
    // A stack the computer plays (0x4939B0).
    o.stacks[0].alive = true;
    assert!(!shade(&o).is_empty());
    o.sides[0].human = false;
    assert!(shade(&o).is_empty());
}

#[test]
fn a_humans_phase_opens_with_its_message() {
    let mut o = phase(3, Vec::new());
    assert!(told(&o));
    o.sides[0].human = false;
    assert!(!told(&o));
    o.tactics = None;
    o.sides[0].human = true;
    assert!(!told(&o));
}

#[test]
fn a_walker_shades_its_zone_and_the_enemies_it_reaches() {
    let walker = stack(0, Side::Attacker, 0, hex(5, 1), false);
    let friend = stack(1, Side::Attacker, 0, hex(2, 2), false);
    let enemy = stack(2, Side::Defender, 0, hex(5, 15), true);
    let mut o = phase(3, alloc::vec![walker, friend, enemy]);
    o.obstacles = alloc::vec![hex(8, 3)];
    let mut wanted = columns(1, 3);
    // Neither the friend's hex nor the obstacle's; both of the two-hex enemy's (0x4938F9).
    wanted.remove(&hex(2, 2));
    wanted.remove(&hex(8, 3));
    wanted.extend([hex(5, 15), hex(5, 14)]);
    assert_eq!(shade(&o), wanted);
}

#[test]
fn a_walled_off_walker_reaches_only_its_side_of_the_wall() {
    let walker = stack(0, Side::Attacker, 0, hex(5, 1), false);
    let enemy = stack(1, Side::Defender, 0, hex(5, 15), false);
    let mut o = phase(3, alloc::vec![walker, enemy]);
    // Column 2 blocked from top to bottom: column 1 is cut off from the rest.
    o.obstacles = (0..11).map(|row| hex(row, 2)).collect();
    assert_eq!(shade(&o), columns(1, 1));
    // A flyer passes over the wall (0x4B39C4): the zone's free hexes and the enemy.
    o.stacks[0].flies = true;
    let mut wanted = columns(1, 3);
    wanted.retain(|h| h % COLUMNS != 2);
    wanted.insert(hex(5, 15));
    assert_eq!(shade(&o), wanted);
}

#[test]
fn a_two_hex_stack_needs_its_front_hex_in_the_zone() {
    let champion = stack(0, Side::Attacker, 0, hex(5, 1), true);
    let mut o = phase(3, alloc::vec![champion]);
    // Row 8's column 2 blocked: in that row the stack fits nowhere with its front in the zone,
    // so its columns 1 and 3 stay unshaded (column 3 only ever as a front hex).
    o.obstacles = alloc::vec![hex(8, 2)];
    let mut wanted = columns(1, 3);
    wanted.retain(|&h| h / COLUMNS != 8);
    assert_eq!(shade(&o), wanted);
    // An enemy at the field's edge: no position next to it holds the stack with its front, but
    // its hex is the front hex a position's neighbour would take (0x4B38C4).
    let goblin = stack(1, Side::Defender, 0, hex(8, 15), false);
    let mut edge = phase(3, alloc::vec![o.stacks[0].clone(), goblin]);
    edge.obstacles = alloc::vec![hex(7, 15), hex(9, 15)];
    assert!(shade(&edge).contains(&hex(8, 15)));
    // Walled in on its left too, it is out of reach.
    edge.obstacles.push(hex(8, 14));
    assert!(!shade(&edge).contains(&hex(8, 15)));
    // On the defender's side the front hex is to the left.
    let mut defender = phase(
        3,
        alloc::vec![stack(0, Side::Defender, 0, hex(5, 15), true)],
    );
    defender.tactics = Some(Tactics {
        side: Side::Defender,
        columns: 3,
    });
    assert_eq!(shade(&defender), columns(13, 15));
}
