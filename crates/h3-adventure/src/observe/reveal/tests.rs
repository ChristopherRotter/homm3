//! Tests of the reveal (SRV-Q26 (a)): against the observation of the same state through the older
//! grid, under both profiles; a step's newly explored and fringe tiles, the heroes, objects,
//! monsters and boats on them; nothing since the same grid; everything since nothing; a grid of
//! other dimensions and a viewer that is not a player.

extern crate std;

use std::string::String;
use std::vec;
use std::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::{HeroId, PlayerId};

use crate::command::AdventureCommand;
use crate::observe::{AdventureObservation, TileView};
use crate::persistent::BitGrid;
use crate::ruleset::AdventureRuleset;
use crate::setup::Pos;
use crate::state::AdventureState;
use crate::testing::{PEASANT, PIKEMAN, hero_at, movement_rules, stream, world};

use super::{AdventureReveal, RevealedTile};

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);

fn p(x: u16, y: u16) -> Pos {
    Pos::new(x, y, 0)
}

/// What `viewer` sees of `state` through the grid `grid` in place of its own: the same state, so
/// every difference from its observation is the grids'.
fn through(
    rules: &(Ruleset, AdventureRuleset),
    state: &AdventureState,
    viewer: PlayerId,
    grid: &BitGrid,
) -> AdventureObservation {
    let mut other = state.clone();
    *other.explored_mut(viewer).0 = grid.clone();
    other.observe(&rules.0, &rules.1, viewer)
}

/// The tiles of the map, in tile order.
fn positions(state: &AdventureState) -> Vec<Pos> {
    let map = state.map();
    let mut all = Vec::new();
    for level in 0..map.levels() {
        for y in 0..map.height() {
            for x in 0..map.width() {
                all.push(Pos::new(x, y, level));
            }
        }
    }
    all
}

/// The oracle: what `viewer`'s observation of `state` shows that the same state seen through
/// `since` does not, part by part (its towns are the game's, `h3_game`'s tests).
fn expected(
    rules: &(Ruleset, AdventureRuleset),
    state: &AdventureState,
    viewer: PlayerId,
    since: &BitGrid,
) -> AdventureReveal {
    let now = state.observe(&rules.0, &rules.1, viewer);
    let old = through(rules, state, viewer, since);
    let sight = |o: &AdventureObservation, pos| (o.is_explored(pos), o.is_fringe(pos));
    let tiles = positions(state)
        .into_iter()
        .filter(|&pos| sight(&now, pos) != sight(&old, pos))
        .map(|pos| RevealedTile {
            pos,
            explored: now.is_explored(pos),
            view: *now.tile(pos).unwrap(),
        })
        .collect();
    AdventureReveal {
        viewer,
        tiles,
        heroes: new(&now.heroes, &old.heroes),
        objects: new(&now.objects, &old.objects),
        boats: new(&now.boats, &old.boats),
        towns: Vec::new(),
    }
}

/// The entries of `list` that `old` does not hold.
fn new<T: Clone + PartialEq>(list: &[T], old: &[T]) -> Vec<T> {
    list.iter().filter(|x| !old.contains(x)).cloned().collect()
}

/// 16 × 16 tiles of grass with `marks` (column, row, character of [`crate::testing::world`]).
fn rows(marks: &[(usize, usize, char)]) -> Vec<String> {
    let mut rows: Vec<Vec<char>> = vec![vec!['g'; 16]; 16];
    for &(x, y, c) in marks {
        rows[y][x] = c;
    }
    rows.into_iter()
        .map(|row| row.into_iter().collect())
        .collect()
}

/// Red's hero 1 at (2, 2) and blue's heroes 2 at (9, 2) and 3 at (10, 2); a wood pile at (3, 3),
/// which red sees from the start, and one at (9, 3); a monster at (10, 4); an empty boat at
/// (9, 6). After red's step to (4, 2) (radius 5, r² + r = 30): (9, 2) and (9, 3), hidden before,
/// explored, (10, 2), (10, 4) and (9, 6), hidden before, on its fringe.
fn state(rules: &(Ruleset, AdventureRuleset)) -> AdventureState {
    let marks = [(3, 3, 'P'), (9, 3, 'P'), (10, 4, 'M'), (9, 6, 'b')];
    let rows = rows(&marks);
    let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
    let heroes = vec![
        hero_at(1, 0, p(2, 2), &[PIKEMAN]),
        hero_at(2, 1, p(9, 2), &[PEASANT]),
        hero_at(3, 1, p(10, 2), &[PEASANT]),
    ];
    crate::testing::started(rules, &world(&rows, 1, heroes)).unwrap()
}

/// Red's step from (2, 2) to (4, 2).
fn step(rules: &(Ruleset, AdventureRuleset), state: &mut AdventureState) {
    let command = AdventureCommand::MoveHero {
        hero: HeroId::new(1),
        path: vec![p(3, 2), p(4, 2)],
    };
    state
        .apply(
            &rules.0,
            &rules.1,
            &[],
            (RED, &mut stream()),
            &command,
            &mut Vec::new(),
        )
        .unwrap();
}

// covers: ADV-FOG-004, ADV-FOG-005, ADV-FOG-006, ADV-FOG-008
#[test]
fn a_step_reveals_what_its_new_tiles_show() {
    for faithful in [false, true] {
        let rules = movement_rules(faithful);
        let mut state = state(&rules);
        let since = state.explored(RED).unwrap().clone();
        let before = state.observe(&rules.0, &rules.1, RED);
        step(&rules, &mut state);
        let reveal = state.reveal(&rules.0, &rules.1, RED, &since);
        assert_eq!(reveal, expected(&rules, &state, RED, &since), "{faithful}");
        // Every tile whose sight changed, and only those: the planes' difference.
        let after = state.observe(&rules.0, &rules.1, RED);
        let changed = positions(&state)
            .into_iter()
            .filter(|&pos| {
                (before.is_explored(pos), before.is_fringe(pos))
                    != (after.is_explored(pos), after.is_fringe(pos))
            })
            .count();
        assert_eq!(reveal.tiles.len(), changed);
        let tile = |pos| reveal.tiles.iter().find(|t| t.pos == pos).copied();
        // (9, 2) was hidden and is explored; (8, 2) was on the fringe and is explored.
        assert!(tile(p(9, 2)).unwrap().explored);
        assert!(before.is_fringe(p(8, 2)));
        let fringe_to_explored = tile(p(8, 2)).unwrap();
        assert!(fringe_to_explored.explored);
        assert!(matches!(fringe_to_explored.view, TileView::Seen(_)));
        // (10, 2) is on the fringe now: its ground, or under the quirk the whole tile.
        let edge = tile(p(10, 2)).unwrap();
        assert!(!edge.explored);
        assert_eq!(matches!(edge.view, TileView::Seen(_)), faithful);
        // Blue's hero 2 on an explored tile, hero 3 on the fringe under the quirk only; never
        // red's own.
        let ids: Vec<u16> = reveal.heroes.iter().map(|h| h.id.raw()).collect();
        assert_eq!(ids, if faithful { vec![2, 3] } else { vec![2] });
        assert!(reveal.heroes.iter().all(|h| h.own.is_none()));
        // The pile explored now, the monster on the fringe in either profile; not the pile red
        // saw before.
        let anchors: Vec<Pos> = reveal.objects.iter().map(|o| o.anchor).collect();
        assert_eq!(anchors, [p(9, 3), p(10, 4)]);
        // The boat on the fringe under the quirk only.
        let boats: Vec<Pos> = reveal.boats.iter().map(|b| b.tile).collect();
        assert_eq!(boats, if faithful { vec![p(9, 6)] } else { vec![] });
        assert_eq!(reveal.towns, []);
    }
}

// covers: ADV-FOG-004, ADV-FOG-005, ADV-FOG-006, ADV-FOG-008
#[test]
fn since_nothing_a_reveal_is_all_the_viewer_sees_but_its_own_heroes() {
    for faithful in [false, true] {
        let rules = movement_rules(faithful);
        let mut state = state(&rules);
        step(&rules, &mut state);
        for viewer in [RED, BLUE] {
            let nothing = BitGrid::new(16, 16, 1);
            let reveal = state.reveal(&rules.0, &rules.1, viewer, &nothing);
            assert_eq!(reveal, expected(&rules, &state, viewer, &nothing));
            let all = state.observe(&rules.0, &rules.1, viewer);
            let seen = positions(&state)
                .into_iter()
                .filter(|&pos| all.tile(pos) != Some(&TileView::Hidden))
                .count();
            assert_eq!(reveal.tiles.len(), seen);
            assert_eq!(reveal.objects, all.objects);
            assert_eq!(reveal.boats, all.boats);
            let others: Vec<_> = all
                .heroes
                .iter()
                .filter(|h| h.owner != viewer)
                .cloned()
                .collect();
            assert_eq!(reveal.heroes, others);
        }
    }
}

#[test]
fn the_same_grid_reveals_nothing() {
    let rules = movement_rules(true);
    let mut state = state(&rules);
    step(&rules, &mut state);
    let grid = state.explored(RED).unwrap().clone();
    assert_eq!(
        state.reveal(&rules.0, &rules.1, RED, &grid),
        AdventureReveal {
            viewer: RED,
            tiles: vec![],
            heroes: vec![],
            objects: vec![],
            boats: vec![],
            towns: vec![],
        }
    );
}

#[test]
fn a_grid_of_other_dimensions_counts_as_nothing_explored() {
    let rules = movement_rules(false);
    let state = state(&rules);
    let nothing = BitGrid::new(16, 16, 1);
    let mut other = BitGrid::new(8, 8, 1);
    other.set(p(2, 2), true);
    assert_eq!(
        state.reveal(&rules.0, &rules.1, RED, &other),
        state.reveal(&rules.0, &rules.1, RED, &nothing)
    );
}

#[test]
fn a_viewer_that_is_not_a_player_has_only_its_grid_s_tiles_hidden() {
    let rules = movement_rules(false);
    let state = state(&rules);
    let stranger = PlayerId::new(5);
    let nothing = BitGrid::new(16, 16, 1);
    let reveal = state.reveal(&rules.0, &rules.1, stranger, &nothing);
    assert_eq!(reveal.tiles, []);
    assert_eq!((reveal.heroes.len(), reveal.objects.len()), (0, 0));
    // A grid it never had: its tiles and their fringe are hidden from it now.
    let mut some = BitGrid::new(16, 16, 1);
    some.set(p(5, 5), true);
    let reveal = state.reveal(&rules.0, &rules.1, stranger, &some);
    assert_eq!(reveal.tiles.len(), 9);
    assert!(
        reveal
            .tiles
            .iter()
            .all(|t| !t.explored && t.view == TileView::Hidden)
    );
}
