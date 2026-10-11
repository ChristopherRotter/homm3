//! Tests of the observation and the events' projection (E08 T8b1): explored tiles, the fringe
//! under both profiles (ADV-FOG-004, ADV-FOG-005), the heroes a player sees (ADV-FOG-006), a
//! viewer that is not a player, what hidden state never changes, and the steps' viewers on the
//! fringe.

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
use crate::testing::{GNOLL, PEASANT, PIKEMAN, hero_at, movement_rules, world_state};

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
    let mut tiles = Vec::new();
    for y in 0..map.height() {
        for x in 0..map.width() {
            let tile = *map.tile(p(x, y)).unwrap();
            let explored = state.explored(viewer).unwrap();
            tiles.push(
                if explored.get(p(x, y)) || (faithful && on_fringe(explored, p(x, y))) {
                    TileView::Seen(tile)
                } else if on_fringe(explored, p(x, y)) {
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
        let seen = state.observe(&rules.0, RED);
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

// covers: ADV-FOG-005, ADV-FOG-006
#[test]
fn a_player_sees_where_other_heroes_stand_and_its_own_in_full() {
    for faithful in [false, true] {
        let rules = movement_rules(faithful);
        let state = state_of(&rules, &[], four_heroes());
        let red = state.observe(&rules.0, RED);
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
                own: Some(OwnHeroView {
                    army: own.army.clone(),
                    movement: own.movement,
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
                own: None,
            })
        );
        assert_eq!(red.hero(HeroId::new(3)), None);
        // Blue sees its three heroes in full, and red's on a tile hero 4 explored.
        let blue = state.observe(&rules.0, BLUE);
        assert_eq!(hero_ids(&blue), [1, 2, 3, 4]);
        assert!(blue.heroes[0].own.is_none());
        assert!(blue.heroes[1..].iter().all(|h| h.own.is_some()));
    }
}

#[test]
fn a_viewer_that_is_not_a_player_sees_no_tile_and_no_hero() {
    let rules = movement_rules(true);
    let state = state_of(&rules, &[], four_heroes());
    let seen = state.observe(&rules.0, PlayerId::new(2));
    assert_eq!(seen.tiles, vec![TileView::Hidden; 256]);
    assert!(seen.heroes.is_empty());
    assert_eq!((seen.explored.len(), seen.fringe.len()), (16, 16));
    assert!(seen.explored.iter().chain(&seen.fringe).all(|&w| w == 0));
}

#[test]
fn the_observation_answers_off_the_map_as_unseen() {
    let rules = movement_rules(true);
    let state = state_of(&rules, &[], four_heroes());
    let seen = state.observe(&rules.0, RED);
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
    state_of(rules, marks, heroes).observe(&rules.0, RED)
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
            .apply(&rules.0, &rules.1, RED, &command, &mut events)
            .unwrap();
        let seen: Vec<bool> = events.iter().map(|e| e.seen_by.contains(BLUE)).collect();
        // The step onto (4, 0) enters blue's fringe, the one onto (5, 0) its explored tiles.
        let expected = if faithful {
            [false, false, false, true, true]
        } else {
            [false, false, false, false, true]
        };
        assert_eq!(seen, expected);
        assert!(events.iter().all(|e| e.seen_by.contains(RED)));
    }
}

#[test]
fn an_event_shows_to_its_viewers_naming_only_the_viewer() {
    let event = AdventureEvent {
        kind: AdventureEventKind::HeroStepped {
            hero: HeroId::new(1),
            from: p(0, 0),
            to: p(1, 0),
        },
        seen_by: Viewers::NONE.with(RED).with(BLUE),
    };
    assert_eq!(
        event.project(BLUE),
        Some(AdventureEvent {
            kind: event.kind,
            seen_by: Viewers::NONE.with(BLUE),
        })
    );
    assert_eq!(
        event.project(RED).map(|e| e.seen_by),
        Some(Viewers::NONE.with(RED))
    );
    assert_eq!(event.project(PlayerId::new(2)), None);
}
