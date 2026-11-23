//! The engine's walks (E24 M4 T4a2, design §5.13): a record's `MoveHero`s played, each walk the
//! selected hero starts taking the next one's facts: its steps checked against its tiles, its
//! points at its steps' ends, the tiles each step reveals at its start (the view from its first
//! frame, the minimap at its end); a step off the record refuses the millisecond.

use super::*;

use crate::live::KingdomHero;

fn at(x: u8, y: u8) -> Pos {
    Pos { x, y, z: 0 }
}

/// `walking`'s loop, its kingdom's selected hero at 1560 points, with the engine's `moves`.
fn on_record(speed: usize, paths: Vec<Vec<Arrow>>, moves: Vec<Move>) -> Loop {
    let mut l = walking(speed, paths);
    l.kingdom.heroes = vec![KingdomHero {
        points: 1560,
        ..KingdomHero::default()
    }];
    l.kingdom.selected = Selected::Hero(0);
    l.moves = Some(Arc::new(moves));
    l
}

/// [`east`]'s walk as a record gives it (MCL-Q39 (c)): its points at its last step, its two new
/// tiles at its first.
fn east_move() -> Move {
    Move {
        tiles: vec![at(18, 17), at(19, 17), at(20, 17), at(21, 17)],
        points: vec![None, None, None, Some(1160)],
        revealed: vec![
            vec![at(22, 17), at(23, 17)],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ],
        sea: None,
        battle: None,
        boats: Vec::new(),
    }
}

/// Runs `l` from `from` to `to` without actions, each millisecond's state.
fn run_to(l: &mut Loop, from: u32, to: u32) -> Vec<State> {
    (from..=to)
        .map(|t| {
            l.run(t, &[]).unwrap();
            l.state(t)
        })
        .collect()
}

#[test]
fn a_record_s_walk_reveals_at_its_first_step_and_pays_at_its_last() {
    let mut l = on_record(2, vec![east()], vec![east_move()]);
    let click = Action::Click(tile(21, 17));
    l.run(1, &[click]).unwrap();
    l.run(2, &[click]).unwrap();
    let revealed = vec![at(22, 17), at(23, 17)];
    let s = l.state(2);
    // The view shows the reveal from the first step's first frame, the minimap at its end.
    assert_eq!(*s.drawn.walked, revealed);
    assert!(s.drawn.minimap.walked.is_empty());
    assert_eq!(s.points, Some(1560));
    let states = run_to(&mut l, 3, 1000);
    let first_end = states
        .iter()
        .find(|s| s.hero.unwrap().at == at(19, 17))
        .unwrap();
    assert_eq!(*first_end.drawn.minimap.walked, revealed);
    // The points stay until the last step's end, then the walk's.
    let last = states.last().unwrap();
    assert_eq!(last.hero.unwrap().at, at(21, 17));
    assert_eq!(last.points, Some(1160));
    assert!(
        states
            .iter()
            .filter(|s| s.hero.unwrap().at != at(21, 17))
            .all(|s| s.points == Some(1560))
    );
    assert_eq!((l.walking, l.next_move), (None, 1));
}

#[test]
fn a_key_s_step_takes_the_next_walk() {
    // A key's step is a walk of one tile; the next walk after the last is off the record.
    let step = Move {
        tiles: vec![at(18, 17)],
        points: vec![Some(1460)],
        revealed: vec![Vec::new()],
        sea: None,
        battle: None,
        boats: Vec::new(),
    };
    let mut l = on_record(4, Vec::new(), vec![step]);
    l.run(1, &[Action::Step(Direction::East)]).unwrap();
    let s = l.state(1);
    assert_eq!((s.hero.unwrap().at, s.points), (at(18, 17), Some(1460)));
    assert!(s.drawn.walked.is_empty());
    assert_eq!(
        l.run(2, &[Action::Step(Direction::East)]),
        Err(Unplayable::Moved([19, 17, 0]))
    );
    // Past the last walk the points are as they were.
    assert_eq!(l.state(2).points, Some(1460));
}

#[test]
fn a_step_off_the_record_refuses_its_millisecond() {
    // Another tile than the walk's: the first one refuses it (speed 4: the walk's steps all in
    // the millisecond).
    let wrong = Move {
        tiles: vec![at(18, 18), at(19, 18), at(20, 18), at(21, 18)],
        ..east_move()
    };
    let mut l = on_record(4, vec![east()], vec![wrong]);
    let click = Action::Click(tile(21, 17));
    l.run(1, &[click]).unwrap();
    assert_eq!(l.run(2, &[click]), Err(Unplayable::Moved([18, 17, 0])));
    // A walk the record takes further than the path shown ends short of it.
    let longer = Move {
        tiles: vec![at(18, 17), at(19, 17), at(20, 17), at(21, 17), at(22, 17)],
        points: vec![None; 5],
        revealed: vec![Vec::new(); 5],
        sea: None,
        battle: None,
        boats: Vec::new(),
    };
    let mut l = on_record(4, vec![east()], vec![longer]);
    l.run(1, &[click]).unwrap();
    assert_eq!(l.run(2, &[click]), Err(Unplayable::Moved([22, 17, 0])));
    assert_eq!(
        Unplayable::Moved([22, 17, 0]).to_string(),
        "a step onto (22, 17, 0) the record's walks do not take"
    );
}

#[test]
fn a_record_s_path_s_arrows() {
    let table = &rules(Bugs::Fixed).walk.arrows;
    // Four tiles east, then the turn to the south-east and a step that way (`game-walk`'s).
    let from = at(12, 12);
    let tiles = [at(13, 12), at(14, 12), at(15, 12), at(16, 12)];
    let shown = arrows(table, from, &tiles).unwrap();
    let numbers: Vec<u16> = shown.iter().map(|a| a.arrow).collect();
    assert_eq!(numbers, [12, 12, 12, DESTINATION]);
    let tiles = [at(17, 12), at(18, 12), at(19, 13), at(20, 14)];
    let numbers: Vec<u16> = arrows(table, at(16, 12), &tiles)
        .unwrap()
        .iter()
        .map(|a| a.arrow)
        .collect();
    assert_eq!(numbers, [12, 21, 13, DESTINATION]);
    // Every pair of steps names an arrow 2 to 25 (green).
    for into in Direction::ALL {
        for out in Direction::ALL {
            let [dx, dy] = into.delta();
            let [ox, oy] = out.delta();
            let mid = |d: i32| u8::try_from(17 + d).unwrap();
            let one = at(mid(dx), mid(dy));
            let two = at(mid(dx + ox), mid(dy + oy));
            let arrow = arrows(table, at(17, 17), &[one, two]).unwrap()[0].arrow;
            assert!((2..=25).contains(&arrow), "{into:?} {out:?}");
        }
    }
    // No path: a tile no neighbour, another level, a table without the pair.
    assert_eq!(arrows(table, from, &[at(14, 12)]), None);
    assert_eq!(arrows(table, from, &[Pos { x: 13, y: 12, z: 1 }]), None);
    assert_eq!(arrows(&table[..8], from, &[at(13, 12), at(14, 12)]), None);
    assert_eq!(arrows(table, from, &[]), Some(Vec::new()));
}
