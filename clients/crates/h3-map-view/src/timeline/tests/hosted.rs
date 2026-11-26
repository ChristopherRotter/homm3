//! The hot seat's host in play (E24 M5 T4a, design §5.26): a live view asks it the selected
//! hero's reach where it stands (a click's path, the cursor over the view) and each walk past the
//! engine's known ones (a path's, a key's step); until it answers the action changes nothing, and
//! the loop run again from before it plays it with the answer.

use super::*;

use crate::hover::frame;
use crate::live::KingdomHero;
use crate::reach::{Hosted, Reach, Stand, WalkAsked};

fn at(x: u8, y: u8) -> Pos {
    Pos { x, y, z: 0 }
}

/// `walking`'s loop at hero speed 3, live with a host and no walk known, its hero selected.
fn hosted() -> Loop {
    let mut start = walking_start(3, Vec::new());
    start.live = true;
    start.heroes = start.hero.into_iter().collect();
    start.index = Some(0);
    start.moves = Some(Vec::new());
    start.host = Some(Hosted::default());
    start.kingdom.heroes = vec![KingdomHero {
        points: 1560,
        ..KingdomHero::default()
    }];
    start.kingdom.selected = Selected::Hero(0);
    Loop::start(rules(Bugs::Fixed), &start).unwrap()
}

/// The stand of the selected hero after `walks` walks.
fn stand(walks: usize) -> Stand {
    Stand {
        hero: 0,
        walks,
        turns: 0,
    }
}

/// A reach of [`east`]'s path, each tile at 100 points a step.
fn east_reach() -> Arc<Reach> {
    Arc::new(Reach {
        paths: vec![east()],
        costs: (18..=21)
            .map(|x| ((x, 17), 100 * (x as u32 - 17)))
            .collect(),
    })
}

/// The engine's walk of [`east`]: its points at its last step.
fn east_walk() -> Move {
    Move {
        tiles: vec![at(18, 17), at(19, 17), at(20, 17), at(21, 17)],
        points: vec![None, None, None, Some(1160)],
        revealed: vec![Vec::new(); 4],
        sea: None,
        battle: None,
        boats: Vec::new(),
        stops: None,
    }
}

#[test]
fn a_click_asks_the_reach_where_the_hero_stands() {
    let click = Action::Click(tile(21, 17));
    let mut l = hosted();
    let before = l.clone();
    l.run(1, &[click]).unwrap();
    // Asked: nothing shown yet.
    assert_eq!(l.reach_asked(), Some((1, stand(0))));
    assert!(l.state(1).arrows.is_empty());
    // Given, the click played again: the engine's path.
    let mut l = before;
    l.add_reach(stand(0), &east_reach());
    assert_eq!(l.reach_asked(), None);
    l.run(1, &[click]).unwrap();
    assert_eq!(l.state(1).arrows.len(), 4);
    assert_eq!(l.reach_asked(), None);
    // A tile it does not reach: the pointer's move only.
    l.run(2, &[Action::Click(tile(10, 10))]).unwrap();
    assert_eq!(l.state(2).arrows.len(), 4);
}

#[test]
fn a_path_s_walk_asks_the_host_then_plays_its_facts() {
    let click = Action::Click(tile(21, 17));
    let mut l = hosted();
    l.add_reach(stand(0), &east_reach());
    l.run(1, &[click]).unwrap();
    let before = l.clone();
    // Its destination clicked again: the walk asked, the hero still standing, its path shown.
    l.run(2, &[click]).unwrap();
    let asked = WalkAsked {
        at: 2,
        hero: 0,
        tiles: vec![at(18, 17), at(19, 17), at(20, 17), at(21, 17)],
    };
    assert_eq!(l.walk_asked(), Some(&asked));
    let s = l.state(2);
    assert_eq!((s.hero.unwrap().at, s.arrows.len()), (at(17, 17), 4));
    // The host's facts given: the walk played as a record's.
    let mut l = before;
    l.add_walks(&[east_walk()], &[]);
    l.run(2, &[click]).unwrap();
    assert_eq!(l.walk_asked(), None);
    for t in 3..=600 {
        l.run(t, &[]).unwrap();
    }
    let s = l.state(600);
    assert_eq!((s.hero.unwrap().at, s.points), (at(21, 17), Some(1160)));
    // Standing again: the next click asks the reach of its new stand.
    l.run(601, &[Action::Click(tile(22, 17))]).unwrap();
    assert_eq!(l.reach_asked(), Some((601, stand(1))));
}

#[test]
fn a_key_s_step_the_engine_refuses_is_nothing() {
    let step = Action::Step(Direction::North);
    let mut l = hosted();
    let before = l.clone();
    l.run(1, &[step]).unwrap();
    let tiles = vec![at(17, 16)];
    assert_eq!(l.walk_asked().map(|w| w.tiles.clone()), Some(tiles.clone()));
    // Refused: played again, nothing is asked and nothing changes.
    let mut l = before.clone();
    l.refuse_walk(1, &tiles);
    assert_eq!(l.walk_asked(), None);
    l.run(1, &[step]).unwrap();
    let mut quiet = before.clone();
    quiet.run(1, &[]).unwrap();
    assert_eq!(l.walk_asked(), None);
    assert_eq!(l.state(1), quiet.state(1));
    // Taken: a step as a record's.
    let mut l = before;
    let north = Move {
        tiles: tiles.clone(),
        points: vec![Some(1460)],
        revealed: vec![Vec::new()],
        ..east_walk()
    };
    l.add_walks(&[north], &[]);
    l.run(1, &[step]).unwrap();
    for t in 2..=300 {
        l.run(t, &[]).unwrap();
    }
    assert_eq!(l.state(300).hero.unwrap().at, at(17, 16));
}

#[test]
fn the_cursor_over_the_view_by_the_host_s_reach() {
    let mut l = hovering(true, Bugs::Fixed);
    l.index = Some(0);
    l.moves = Some(Arc::new(Vec::new()));
    l.host = Some(Arc::new(Hosted::default()));
    let before = l.clone();
    // The monster's tile: the reach asked, the cursor not set yet.
    l.run(1, &[Action::Pointer(on(5, 4))]).unwrap();
    assert_eq!(l.reach_asked(), Some((1, stand(0))));
    let cursor = |reach: Reach| {
        let mut l = before.clone();
        l.add_reach(stand(0), &Arc::new(reach));
        l.run(1, &[Action::Pointer(on(5, 4))]).unwrap();
        l.state(1).cursor.frame
    };
    // Reached today: the attack's first day, the start's facts' second not read.
    let today = Reach {
        paths: Vec::new(),
        costs: [((5, 4), 50)].into_iter().collect(),
    };
    assert_eq!(cursor(today), frame::ATTACK);
    // Not reached today: the arrow (MCL-Q37: no later day's cost).
    assert_eq!(cursor(Reach::default()), rules(Bugs::Fixed).arrow);
}
