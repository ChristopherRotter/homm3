use super::*;

use alloc::vec;

use crate::timeline::DESTINATION;

fn wire(x: u16, y: u16) -> WirePos {
    WirePos { x, y, level: 0 }
}

fn at(x: u8, y: u8) -> Pos {
    Pos { x, y, z: 0 }
}

/// The walk's arrows' table: each pair of steps its own frame.
fn table() -> Vec<u8> {
    (0..64).collect()
}

/// A tree from (5, 5) with 1 000 points: east 900 left, east again 800, south-east passed, south
/// from there 700.
fn moves() -> Moves {
    Moves {
        origin: wire(5, 5),
        points: 1000,
        steps: vec![2, 8 + 2, 3, 8 * 3 + 4],
        left: vec![901, 801, 0, 701],
    }
}

#[test]
fn a_reach_holds_each_tile_s_path_and_cost() {
    let reach = Reach::of(&moves(), &table()).expect("reads");
    let ends: Vec<Pos> = reach
        .paths
        .iter()
        .filter_map(|p| p.last().map(|a| a.at))
        .collect();
    assert_eq!(ends, [at(6, 5), at(7, 5), at(6, 7)]);
    assert_eq!(
        reach.paths[0],
        [Arrow {
            at: at(6, 5),
            arrow: DESTINATION,
        }]
    );
    // Into east (2), out east: frame 2 + 8 × 2, plus 2.
    assert_eq!(reach.paths[1][0].arrow, 18 + 2);
    assert_eq!(
        reach.paths[2],
        timeline::arrows(&table(), at(5, 5), &[at(6, 6), at(6, 7)]).expect("arrows")
    );
    let costs: Vec<((i32, i32), u32)> = reach.costs.into_iter().collect();
    assert_eq!(costs, [((6, 5), 100), ((6, 7), 300), ((7, 5), 200)]);
}

#[test]
fn a_reach_of_a_tree_that_does_not_read_is_none() {
    // A malformed tree.
    let mut bad = moves();
    bad.left.pop();
    assert_eq!(Reach::of(&bad, &table()), None);
    // Its origin off the map's columns, or rows.
    let mut far = moves();
    far.origin = wire(300, 5);
    assert_eq!(Reach::of(&far, &table()), None);
    far.origin = wire(5, 300);
    assert_eq!(Reach::of(&far, &table()), None);
    // A step off the map's 256 columns.
    let edge = Moves {
        origin: wire(255, 0),
        points: 100,
        steps: vec![2],
        left: vec![51],
    };
    assert_eq!(Reach::of(&edge, &table()), None);
    // No arrow for a pair of steps.
    assert_eq!(Reach::of(&moves(), &[]), None);
    // No tile reached: an empty reach.
    let none = Moves {
        origin: wire(5, 5),
        points: 0,
        steps: Vec::new(),
        left: Vec::new(),
    };
    assert_eq!(Reach::of(&none, &table()), Some(Reach::default()));
}

#[test]
fn the_host_s_facts_by_stand_and_walk() {
    let stand = Stand {
        hero: 1,
        walks: 2,
        turns: 0,
    };
    let reach = Arc::new(Reach::default());
    let hosted = Hosted {
        reaches: vec![(stand, Arc::clone(&reach))],
        refused: vec![(40, vec![at(6, 5)])],
    };
    assert_eq!(hosted.reach(stand), Some(&reach));
    assert_eq!(hosted.reach(Stand { walks: 3, ..stand }), None);
    assert!(hosted.refuses(40, &[at(6, 5)]));
    assert!(!hosted.refuses(41, &[at(6, 5)]));
    assert!(!hosted.refuses(40, &[at(4, 5)]));
    let asked = WalkAsked {
        at: 40,
        hero: 1,
        tiles: vec![at(6, 5), at(7, 255)],
    };
    assert_eq!(asked.path(), [wire(6, 5), wire(7, 255)]);
}
