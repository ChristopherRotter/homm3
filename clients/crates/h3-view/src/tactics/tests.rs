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
    // T8b2, R5c-2: a zone wider than the field (a hostile server's) is the whole field.
    for n in [16, 17, 255] {
        let wide = |side| Tactics { side, columns: n };
        assert_eq!(zone(&wide(Side::Defender)), (0..16).collect::<Vec<_>>());
        assert_eq!(zone(&wide(Side::Attacker)), (1..17).collect::<Vec<_>>());
        let mut o = phase(
            n,
            alloc::vec![stack(0, Side::Defender, 0, hex(5, 15), true)],
        );
        o.tactics = Some(wide(Side::Defender));
        assert_eq!(shade(&o), columns(1, 15));
    }
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
    // The defender's phase: its player's (T8b2, R5c-9).
    o.tactics = Some(Tactics {
        side: Side::Defender,
        columns: 3,
    });
    assert!(told(&o));
    o.sides[1].human = false;
    assert!(!told(&o));
}

#[test]
fn a_sieges_walls_and_closed_gate_block_the_phase() {
    // T8b2, R5c-6 (BAT-SIE-002, BAT-SIE-003, the engine's `siege::blocked_for`).
    use crate::siege::WALL_HEXES;
    use h3_protocol::wire::Siege;
    let wall = |h: u8| WALL_HEXES[usize::from(h / COLUMNS)];
    let closed = Siege {
        faction: 0,
        fort: 1,
        moat: 0,
        gate_open: false,
        walls: WALL_HEXES.to_vec(),
        moat_hexes: Vec::new(),
        parts: Vec::new(),
    };
    let mut o = phase(
        12,
        alloc::vec![stack(0, Side::Attacker, 0, hex(5, 1), false)],
    );
    o.siege = Some(closed.clone());
    // The walls, the closed gate and the drawbridge keep the attacker outside.
    let mut outside: BTreeSet<u8> = columns(1, 12)
        .into_iter()
        .filter(|&h| h < wall(h))
        .collect();
    outside.remove(&BRIDGES[0]);
    assert_eq!(shade(&o), outside);
    // Open, the gate lets it in: the zone but the standing walls.
    o.siege.as_mut().unwrap().gate_open = true;
    let mut open = columns(1, 12);
    open.retain(|&h| h == GATE_HEX || !WALL_HEXES.contains(&h));
    assert_eq!(shade(&o), open);
    // The defender's step opens the closed gate: the zone but the walls.
    let mut d = phase(
        7,
        alloc::vec![stack(0, Side::Defender, 0, hex(5, 14), false)],
    );
    d.tactics = Some(Tactics {
        side: Side::Defender,
        columns: 7,
    });
    d.siege = Some(closed);
    let mut through = columns(9, 15);
    through.retain(|&h| h == GATE_HEX || !WALL_HEXES.contains(&h));
    assert_eq!(shade(&d), through);
    // Not with a stack on a bridge (here a friend on the outer one): only inside the walls.
    d.stacks
        .push(stack(1, Side::Defender, 0, BRIDGES[1], false));
    let inside: BTreeSet<u8> = columns(9, 15)
        .into_iter()
        .filter(|&h| h > wall(h))
        .collect();
    assert_eq!(shade(&d), inside);
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

#[test]
fn a_record_shades_each_human_turn_of_the_phase_while_it_waits() {
    // E17 M5 T4c2b: the opening's turn until the first action, none while an action plays, the
    // next turn's from its start (the battle as it found it: the placement's hex), none once the
    // phase ended.
    let first = stack(0, Side::Attacker, 0, hex(5, 1), false);
    let second = stack(1, Side::Attacker, 0, hex(7, 1), false);
    let opening = phase(3, alloc::vec![first, second]);
    let mut placed = opening.clone();
    placed.stacks[0].hex = hex(5, 3);
    placed.active = Some(1);
    let mut ended = placed.clone();
    ended.tactics = None;
    ended.active = Some(0);
    let turns = [placed.clone(), ended];
    let log = [
        (100, Logged::Action),
        (900, Logged::Turn(1)),
        (1500, Logged::Action),
        (1500, Logged::Turn(0)),
    ];
    let at = |t| shade_at(&opening, &turns, &log, t);
    assert_eq!(at(0), shade(&opening));
    assert_eq!(at(99), shade(&opening));
    assert!(at(100).is_empty() && at(899).is_empty());
    assert_eq!(at(900), shade(&placed));
    assert!(at(900).contains(&hex(5, 1)) && !at(900).contains(&hex(5, 3)));
    assert_eq!(at(1499), shade(&placed));
    // Start Combat at 1 500: round 1's turn starts in the same millisecond, the phase over.
    assert!(at(1500).is_empty() && at(5000).is_empty());
    // A log's lines between them change nothing.
    let lined = [
        (100, Logged::Action),
        (
            50,
            Logged::Defend {
                stack: 0,
                count: 1,
                defense: 1,
            },
        ),
        (900, Logged::Turn(1)),
    ];
    assert_eq!(shade_at(&opening, &turns, &lined, 1000), shade(&placed));
}
