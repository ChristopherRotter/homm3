//! Tests of the observation and the events' projection (E08 T8b1): explored tiles, the fringe
//! under both profiles (ADV-FOG-004, ADV-FOG-005), the heroes a player sees (ADV-FOG-006), a
//! viewer that is not a player, what hidden state never changes, and the steps' viewers on the
//! fringe; the typed objects a player sees and the tiles removed objects free (T12b).

extern crate std;

use std::string::String;
use std::vec;
use std::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::{HeroId, PlayerId};

use super::*;
use crate::command::{AdventureCommand, AdventureEventKind};
use crate::fog::{Sight, on_fringe};
use crate::ruleset::AdventureRuleset;
use crate::setup::HeroSetup;
use crate::testing::{
    GNOLL, PEASANT, PIKEMAN, WOOD_PILE, hero_at, movement_rules, movement_rules_up_to, world_state,
};

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);

fn p(x: u16, y: u16) -> Pos {
    Pos::new(x, y, 0)
}

/// 16 × 16 tiles of grass, with `marks` (column, row, character of [`crate::testing::world`])
/// set.
fn rows(marks: &[(usize, usize, char)]) -> Vec<String> {
    let mut rows: Vec<Vec<char>> = vec![vec!['g'; 16]; 16];
    for &(x, y, c) in marks {
        rows[y][x] = c;
    }
    rows.into_iter()
        .map(|row| row.into_iter().collect())
        .collect()
}

fn state_of(
    rules: &(Ruleset, AdventureRuleset),
    marks: &[(usize, usize, char)],
    heroes: Vec<HeroSetup>,
) -> AdventureState {
    let rows = rows(marks);
    let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
    world_state(rules, &rows, heroes)
}

/// Red's hero 1 at (2, 2); blue's hero 2 at (2, 8), on red's fringe ((2, 7) is 5 rows from red's
/// hero, (2, 8) 6); blue's hero 3 at (13, 13), hidden from red; blue's hero 4 at (6, 3), on a tile
/// red explored (4² + 1² ≤ 30).
fn four_heroes() -> Vec<HeroSetup> {
    vec![
        hero_at(1, 0, p(2, 2), &[PIKEMAN]),
        hero_at(2, 1, p(2, 8), &[PEASANT]),
        hero_at(3, 1, p(13, 13), &[PEASANT, GNOLL]),
        hero_at(4, 1, p(6, 3), &[GNOLL]),
    ]
}

/// What a viewer should see of each tile, tile by tile: explored tiles whole, fringe tiles whole
/// under the flag and their ground without it, nothing else.
fn expected_tiles(state: &AdventureState, viewer: PlayerId, faithful: bool) -> Vec<TileView> {
    let map = state.map();
    let explored = state.explored(viewer).unwrap();
    let mut tiles = Vec::new();
    for level in 0..map.levels() {
        for y in 0..map.height() {
            for x in 0..map.width() {
                let pos = Pos::new(x, y, level);
                let tile = *map.tile(pos).unwrap();
                tiles.push(
                    if explored.get(pos) || (faithful && on_fringe(explored, pos)) {
                        TileView::Seen(tile)
                    } else if on_fringe(explored, pos) {
                        TileView::Ground {
                            terrain: tile.terrain,
                            road: tile.road,
                            river: tile.river,
                        }
                    } else {
                        TileView::Hidden
                    },
                );
            }
        }
    }
    tiles
}

fn hero_ids(observation: &AdventureObservation) -> Vec<u16> {
    observation.heroes.iter().map(|h| h.id.raw()).collect()
}

// covers: ADV-FOG-004, ADV-FOG-005
#[test]
fn a_player_sees_its_explored_tiles_and_the_fringe() {
    // On red's fringe: an object's blocked tile at (2, 8)'s neighbour (3, 8), a road at (4, 8);
    // on an explored tile, a town's gate at (4, 4); hidden, a blocked tile at (12, 12).
    let marks = [(3, 8, 'B'), (4, 8, 'G'), (4, 4, 'T'), (12, 12, 'B')];
    for faithful in [false, true] {
        let rules = movement_rules(faithful);
        let state = state_of(&rules, &marks, four_heroes());
        let seen = state.observe(&rules.0, &rules.1, RED);
        assert_eq!(seen.viewer, RED);
        assert_eq!((seen.width, seen.height, seen.levels), (16, 16, 1));
        let sight = Sight::of(state.explored(RED).unwrap());
        assert_eq!(seen.stride, 1);
        assert_eq!(
            (&seen.explored, &seen.fringe),
            (&sight.explored, &sight.fringe)
        );
        assert_eq!(seen.tiles, expected_tiles(&state, RED, faithful));
        // The cases by hand.
        let gate = *state.map().tile(p(4, 4)).unwrap();
        assert!(gate.blocked && gate.visitable);
        assert_eq!(seen.tile(p(4, 4)), Some(&TileView::Seen(gate)));
        assert!(seen.is_fringe(p(3, 8)) && !seen.is_explored(p(3, 8)));
        let object = *state.map().tile(p(3, 8)).unwrap();
        let road = *state.map().tile(p(4, 8)).unwrap();
        assert!(object.blocked && road.road.is_some());
        if faithful {
            assert_eq!(seen.tile(p(3, 8)), Some(&TileView::Seen(object)));
        } else {
            assert_eq!(
                seen.tile(p(3, 8)),
                Some(&TileView::Ground {
                    terrain: object.terrain,
                    road: None,
                    river: None,
                })
            );
            assert_eq!(
                seen.tile(p(4, 8)),
                Some(&TileView::Ground {
                    terrain: road.terrain,
                    road: road.road,
                    river: None,
                })
            );
        }
        assert_eq!(seen.tile(p(12, 12)), Some(&TileView::Hidden));
        assert!(!seen.is_explored(p(12, 12)) && !seen.is_fringe(p(12, 12)));
    }
}

// covers: ADV-FOG-004, ADV-FOG-005
#[test]
fn the_planes_and_the_tiles_line_up_on_a_wide_map_with_two_levels() {
    // 70 columns (two words a row), 9 rows, two levels: red's heroes reveal across the first
    // word's edge on the surface and at the far right underground; a road and objects on either
    // side of the edge. The tile views must follow each tile's own bit, level by level.
    let mut rows: Vec<Vec<char>> = vec![vec!['g'; 70]; 18];
    for (x, y, c) in [
        (62, 4, 'B'),
        (64, 4, 'G'),
        (66, 5, 'T'),
        (69, 13, 'V'),
        (60, 17, 's'),
    ] {
        rows[y][x] = c;
    }
    let rows: Vec<String> = rows
        .into_iter()
        .map(|row| row.into_iter().collect())
        .collect();
    let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
    let heroes = vec![
        hero_at(1, 0, p(63, 4), &[PEASANT]),
        hero_at(2, 0, Pos::new(66, 5, 1), &[PEASANT]),
        hero_at(3, 1, p(5, 4), &[PEASANT]),
    ];
    for faithful in [false, true] {
        let rules = movement_rules_up_to(faithful, 70);
        let setup = crate::testing::world(&rows, 2, heroes.clone());
        let state = crate::testing::started(&rules, &setup).unwrap();
        let seen = state.observe(&rules.0, &rules.1, RED);
        assert_eq!(
            (seen.width, seen.height, seen.levels, seen.stride),
            (70, 9, 2, 2)
        );
        let sight = Sight::of(state.explored(RED).unwrap());
        assert_eq!(
            (&seen.explored, &seen.fringe),
            (&sight.explored, &sight.fringe)
        );
        assert_eq!(seen.tiles.len(), 70 * 9 * 2);
        assert_eq!(seen.tiles, expected_tiles(&state, RED, faithful));
        assert!(seen.is_explored(p(64, 4)) && seen.is_explored(Pos::new(69, 4, 1)));
        assert!(!seen.is_explored(Pos::new(58, 4, 1)) && !seen.is_explored(p(5, 4)));
        assert_eq!(hero_ids(&seen), [1, 2]);
    }
}

// covers: ADV-FOG-005, ADV-FOG-006
#[test]
fn a_player_sees_where_other_heroes_stand_and_its_own_in_full() {
    for faithful in [false, true] {
        let rules = movement_rules(faithful);
        let state = state_of(&rules, &[], four_heroes());
        let red = state.observe(&rules.0, &rules.1, RED);
        // Hero 2 stands on red's fringe: seen under ADV-QUIRK-1 only. Hero 3 is hidden.
        let expected: &[u16] = if faithful { &[1, 2, 4] } else { &[1, 4] };
        assert_eq!(hero_ids(&red), expected);
        let own = state.hero(HeroId::new(1)).unwrap();
        assert_eq!(
            red.hero(HeroId::new(1)),
            Some(&HeroView {
                id: own.id,
                owner: RED,
                pos: p(2, 2),
                boat: None,
                own: Some(OwnHeroView {
                    army: own.army.clone(),
                    movement: own.movement,
                    max_movement: own.max_movement,
                    formation: own.formation,
                    tactics: true,
                    marks: Vec::new(),
                }),
            })
        );
        // Of another player's hero: its tile, owner and identity, not its army or points.
        assert_eq!(
            red.hero(HeroId::new(4)),
            Some(&HeroView {
                id: HeroId::new(4),
                owner: BLUE,
                pos: p(6, 3),
                boat: None,
                own: None,
            })
        );
        assert_eq!(red.hero(HeroId::new(3)), None);
        // Blue sees its three heroes in full, and red's on a tile hero 4 explored.
        let blue = state.observe(&rules.0, &rules.1, BLUE);
        assert_eq!(hero_ids(&blue), [1, 2, 3, 4]);
        assert!(blue.heroes[0].own.is_none());
        assert!(blue.heroes[1..].iter().all(|h| h.own.is_some()));
    }
}

#[test]
fn a_viewer_that_is_not_a_player_sees_no_tile_and_no_hero() {
    let rules = movement_rules(true);
    let state = state_of(&rules, &[], four_heroes());
    let seen = state.observe(&rules.0, &rules.1, PlayerId::new(2));
    assert_eq!(seen.tiles, vec![TileView::Hidden; 256]);
    assert!(seen.heroes.is_empty());
    assert_eq!((seen.explored.len(), seen.fringe.len()), (16, 16));
    assert!(seen.explored.iter().chain(&seen.fringe).all(|&w| w == 0));
}

#[test]
fn the_observation_answers_off_the_map_as_unseen() {
    let rules = movement_rules(true);
    let state = state_of(&rules, &[], four_heroes());
    let seen = state.observe(&rules.0, &rules.1, RED);
    assert!(seen.is_explored(p(2, 2)));
    for pos in [p(16, 2), p(2, 16), Pos::new(2, 2, 1)] {
        assert!(!seen.is_explored(pos) && !seen.is_fringe(pos));
        assert_eq!(seen.tile(pos), None);
    }
    assert_eq!(seen.hero(HeroId::new(9)), None);
}

/// Red's observation of a state built from `marks` and `heroes`.
fn red_sees(
    rules: &(Ruleset, AdventureRuleset),
    marks: &[(usize, usize, char)],
    heroes: Vec<HeroSetup>,
) -> AdventureObservation {
    state_of(rules, marks, heroes).observe(&rules.0, &rules.1, RED)
}

// covers: ADV-FOG-004, ADV-FOG-005, ADV-FOG-006
#[test]
fn hidden_state_never_changes_an_observation() {
    for faithful in [false, true] {
        let rules = movement_rules(faithful);
        let base = red_sees(&rules, &[], four_heroes());
        // A hidden tile's terrain, road or object.
        for mark in [(14, 14, 's'), (14, 14, 'G'), (12, 12, 'B')] {
            assert_eq!(red_sees(&rules, &[mark], four_heroes()), base, "{mark:?}");
        }
        // Another player's hidden hero elsewhere (so another explored layer), its army; the army
        // of a hero red sees.
        let mut heroes = four_heroes();
        heroes[2] = hero_at(3, 1, p(14, 10), &[PIKEMAN]);
        heroes[3] = hero_at(4, 1, p(6, 3), &[PEASANT, PEASANT]);
        assert_eq!(red_sees(&rules, &[], heroes), base);
        // A fringe tile's object shows under ADV-QUIRK-1 only.
        let fringe = red_sees(&rules, &[(3, 8, 'B')], four_heroes());
        assert_eq!(fringe == base, !faithful);
    }
}

// covers: ADV-FOG-005
#[test]
fn a_step_on_the_fringe_is_seen_under_the_quirk() {
    // Blue's hero at (10, 0) explored (5, 0) to (15, 0) of the single row; (4, 0) is on its
    // fringe. Red walks from (0, 0) to (5, 0).
    for faithful in [false, true] {
        let rules = movement_rules(faithful);
        let mut state = world_state(
            &rules,
            &["gggggggggggggggg"],
            vec![
                hero_at(1, 0, p(0, 0), &[PEASANT]),
                hero_at(2, 1, p(10, 0), &[PEASANT]),
            ],
        );
        let mut events = Vec::new();
        let path: Vec<Pos> = (1..=5).map(|x| p(x, 0)).collect();
        let command = AdventureCommand::MoveHero {
            hero: HeroId::new(1),
            path,
        };
        state
            .apply(&rules.0, &rules.1, &[], RED, &command, &mut events)
            .unwrap();
        // Then back from (5, 0) to (3, 0): out of blue's explored tiles onto its fringe, then off
        // the fringe onto a hidden tile.
        let back = AdventureCommand::MoveHero {
            hero: HeroId::new(1),
            path: vec![p(4, 0), p(3, 0)],
        };
        state
            .apply(&rules.0, &rules.1, &[], RED, &back, &mut events)
            .unwrap();
        // The step onto (4, 0) enters blue's fringe, the one onto (5, 0) its explored tiles; back,
        // the step onto (4, 0) leaves them, the one onto (3, 0) leaves the fringe.
        let seen: Vec<bool> = events.iter().map(|e| e.seen_by.contains(BLUE)).collect();
        let expected = if faithful {
            [false, false, false, true, true, true, true]
        } else {
            [false, false, false, false, true, true, false]
        };
        assert_eq!(seen, expected);
        // What blue is told: only the tiles it sees the hero on.
        let told: Vec<Option<(Option<Pos>, Option<Pos>)>> = events
            .iter()
            .map(|e| {
                e.project(BLUE).map(|e| match e.kind {
                    AdventureEventKind::HeroStepped { from, to, .. } => (from, to),
                    AdventureEventKind::ObjectRemoved { .. }
                    | AdventureEventKind::TilesRevealed { .. }
                    | AdventureEventKind::KeyGained { .. }
                    | AdventureEventKind::HeroRemoved { .. }
                    | AdventureEventKind::CreaturesJoined { .. }
                    | AdventureEventKind::EventFired { .. }
                    | AdventureEventKind::MonsterChanged { .. } => (None, None),
                })
            })
            .collect();
        let (s4, s5) = (Some(p(4, 0)), Some(p(5, 0)));
        let expected = if faithful {
            [
                None,
                None,
                None,
                Some((None, s4)),
                Some((s4, s5)),
                Some((s5, s4)),
                Some((s4, None)),
            ]
        } else {
            [
                None,
                None,
                None,
                None,
                Some((None, s5)),
                Some((s5, None)),
                None,
            ]
        };
        assert_eq!(told, expected);
        // Red, the owner, is told every step whole.
        for event in &events {
            assert!(matches!(
                event.project(RED).unwrap().kind,
                AdventureEventKind::HeroStepped {
                    from: Some(_),
                    to: Some(_),
                    ..
                }
            ));
        }
    }
}

#[test]
fn an_event_shows_to_its_viewers_naming_only_the_viewer() {
    // Red sees the hero on both tiles, blue on the tile it entered only.
    let red = Viewers::NONE.with(RED);
    let blue = Viewers::NONE.with(BLUE);
    let both = red.union(blue);
    let step = |from, to, seen_from, seen_to, seen_by| AdventureEvent {
        kind: AdventureEventKind::HeroStepped {
            hero: HeroId::new(1),
            from,
            to,
            seen_from,
            seen_to,
        },
        seen_by,
    };
    let (a, b) = (Some(p(0, 0)), Some(p(1, 0)));
    let event = step(a, b, red, both, both);
    assert_eq!(
        event.project(BLUE),
        Some(step(None, b, Viewers::NONE, blue, blue))
    );
    assert_eq!(event.project(RED), Some(step(a, b, red, red, red)));
    assert_eq!(event.project(PlayerId::new(2)), None);
    // The sets' algebra.
    assert_eq!(both.bits(), 0b11);
    assert_eq!(both.intersection(blue), blue);
    assert_eq!(red.intersection(blue), Viewers::NONE);
}

// covers: ADV-FOG-004, ADV-FOG-005
#[test]
fn a_typed_object_shows_whole_once_a_tile_it_covers_is_seen() {
    // Wood piles at (3, 3) (explored by red's hero at (2, 2)), (2, 8) (red's fringe) and
    // (13, 13) (hidden); a blocker at (4, 4), never listed; blue's hero at (12, 12).
    let marks = [(3, 3, 'P'), (2, 8, 'P'), (13, 13, 'P'), (4, 4, 'T')];
    let heroes = || {
        vec![
            hero_at(1, 0, p(2, 2), &[PIKEMAN]),
            hero_at(2, 1, p(12, 12), &[PEASANT]),
        ]
    };
    let pile = |x: u16, y: u16| ObjectView {
        object_type: WOOD_PILE,
        anchor: p(x, y),
        blocked: TileMask::from_bits(1),
        visitable: TileMask::from_bits(1),
        drawn: TileMask::default(),
        monster: None,
    };
    for faithful in [true, false] {
        let rules = movement_rules(faithful);
        let state = state_of(&rules, &marks, heroes());
        let red = state.observe(&rules.0, &rules.1, RED);
        // Objects come in map order: (3, 3), (4, 4), (2, 8), (13, 13).
        let mut expected = vec![pile(3, 3)];
        if faithful {
            expected.push(pile(2, 8));
        }
        assert_eq!(red.objects, expected, "faithful {faithful}");
        let blue = state.observe(&rules.0, &rules.1, BLUE);
        assert_eq!(blue.objects, [pile(13, 13)]);
        // A viewer that is not a player sees none.
        assert_eq!(
            state.observe(&rules.0, &rules.1, PlayerId::new(5)).objects,
            []
        );
        // A removed object shows no more, and its tile is free.
        let mut removed = state.clone();
        removed.remove_object(0);
        let red = removed.observe(&rules.0, &rules.1, RED);
        assert_eq!(red.objects.len(), usize::from(faithful));
        let TileView::Seen(tile) = red.tile(p(3, 3)).copied().unwrap() else {
            panic!("(3, 3) is explored");
        };
        assert!(!tile.blocked && !tile.visitable);
        assert_eq!(
            state.observe(&rules.0, &rules.1, RED).tile(p(3, 3)),
            Some(&TileView::Seen(*state.map().tile(p(3, 3)).unwrap()))
        );
    }
}

// covers: ADV-FOG-006, ADV-FOG-008
#[test]
fn a_player_sees_the_empty_boats_and_each_seen_hero_s_boat() {
    // E08 T72: empty boats at (3, 3) and (6, 4), explored by red, (3, 8) on red's fringe and
    // (14, 13) hidden from it; blue's hero 4 boards the one at (6, 4).
    let marks = [(3, 3, 'b'), (6, 4, 'b'), (3, 8, 'b'), (14, 13, 'b')];
    for faithful in [false, true] {
        let rules = movement_rules(faithful);
        let rows = rows(&marks);
        let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
        let mut setup = crate::testing::world(&rows, 1, four_heroes());
        for (boat, kind) in setup.map.boats.iter_mut().zip([1, 2, 0, 1]) {
            boat.kind = kind;
        }
        let mut state = crate::testing::started(&rules, &setup).unwrap();
        state.put_aboard(HeroId::new(4), 1);
        let boat = |x, y, kind| BoatView {
            tile: p(x, y),
            kind,
            facing: 2,
        };
        // Where red sees heroes it sees the empty boats, in tile order; never the boarded one.
        let red = state.observe(&rules.0, &rules.1, RED);
        let mut expected = vec![boat(3, 3, 1)];
        if faithful {
            expected.push(boat(3, 8, 0));
        }
        assert_eq!(red.boats, expected, "faithful {faithful}");
        // Of a hero in a boat every viewer that sees it sees the boat's type, its owner its day's
        // maximum too.
        assert_eq!(red.hero(HeroId::new(4)).unwrap().boat, Some(2));
        assert_eq!(red.hero(HeroId::new(1)).unwrap().boat, None);
        let blue = state.observe(&rules.0, &rules.1, BLUE);
        assert_eq!(
            blue.boats,
            [boat(3, 3, 1), boat(3, 8, 0), boat(14, 13, 1)]
        );
        let own = blue.hero(HeroId::new(4)).unwrap();
        let hero = state.hero(HeroId::new(4)).unwrap();
        assert_eq!(own.boat, Some(2));
        assert_eq!(
            own.own.as_ref().map(|o| o.max_movement),
            Some(hero.max_movement)
        );
        // A viewer that is not a player sees no boat.
        assert_eq!(
            state.observe(&rules.0, &rules.1, PlayerId::new(5)).boats,
            []
        );
    }
}
