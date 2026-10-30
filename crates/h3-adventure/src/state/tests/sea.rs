//! Tests of moving at sea (E08 T73a; design §7, §8 and §10; ADV-SEA-002 to -005): boarding from a
//! walk, a flight and a water walk, a boat's steps and prices with Favorable Winds, the drop and
//! D-43's exception, visits from a boat, landing onto a free coastal tile only (D-42), the
//! conversion of the points with free boarding (D-41) and the day's maximum switched (A-28 not
//! reproduced), the landing's facing, the events, D-46's water tiles closed to a flight, and the
//! reachable tiles of a boat and of a walker beside one (E08 T73b). Batch E8's rows as sources:
//! E8-1 and E8-2 (the conversion after the drop), E8-3 (no landing on a hole or an event), E8-4
//! (the diagonal at sea on Favorable Winds), E8-15 (a guarded landing, the boat's facing).

use std::sync::Arc;

use h3_content::{SkillId, TerrainId};
use h3_heroes::HeroDetails;

use super::*;
use crate::command::{AdventureCommand, AdventureCommandError, AdventureEvent, AdventureEventKind};
use crate::setup::{ObjectSetup, Pos, TileMask};
use crate::testing::{
    FLYING, HOLE, PEASANT, ROCK, SEA, WATER_WALKING, WINDS, coasted, hero_at, hero_details,
    hero_skills, lying, movement_rules_with, sea_skills, started, world,
};

type Rules = (Ruleset, AdventureRuleset);

const RED: PlayerId = PlayerId::new(0);
const HERO: HeroId = HeroId::new(1);

fn p(x: u16, y: u16) -> Pos {
    Pos::new(x, y, 0)
}

/// [`hero_rules`]' skills, Navigation at levels 1 to 3 (the sea table's entries 1 to 3) and a
/// skill of free boarding (the Admiral's Hat's effect), in `faithful` or the strict profile.
fn sea_rules(faithful: bool) -> (Rules, SkillId, SkillId) {
    let rules = movement_rules_with(
        faithful,
        16,
        |core| core.skills = [hero_skills(), sea_skills()].concat(),
        |_| {},
    );
    let navigation = rules.0.skill_by_key("t:navigation").unwrap();
    let admiral = rules.0.skill_by_key("t:admiral").unwrap();
    (rules, navigation, admiral)
}

/// The store of heroes' details: hero 1 with `skills`, heroes 0, 2 and 3 without.
fn store(skills: &[(SkillId, u8)]) -> Vec<Arc<HeroDetails>> {
    vec![
        Arc::new(hero_details(&[])),
        Arc::new(hero_details(skills)),
        Arc::new(hero_details(&[])),
        Arc::new(hero_details(&[])),
    ]
}

/// The sea of these tests: land on columns 0 to 2 and rows 6, 7, an island at (5, 3), empty boats
/// at (3, 2) (boat 0) and (6, 3) (boat 1).
const SEA_ROWS: [&str; 8] = [
    "ggg~~~~~", "ggg~~~~~", "gggb~~~~", "ggg~~gb~", "ggg~~~~~", "ggg~~~~~", "gggggggg", "gggggggg",
];

/// Who a guard's visit is with: the battle's tile or the offer's anchor (an aggressive monster
/// facing a weaker hero may flee, offering a pursuit).
fn guard_visit(outcome: &AdventureOutcome) -> Option<Pos> {
    outcome
        .battle
        .as_ref()
        .map(|b| b.tile)
        .or(outcome.offer.as_ref().map(|o| o.anchor))
}

/// Sets the terrain of `pos` on a one-level setup.
fn set_terrain(setup: &mut AdventureSetup, pos: Pos, terrain: TerrainId) {
    let width = usize::from(setup.map.width);
    setup.map.tiles[usize::from(pos.y) * width + usize::from(pos.x)].terrain = terrain;
}

/// The started state of [`SEA_ROWS`] with red's hero 1 (peasants) on `at`, blue's hero 2 at
/// (7, 7) and `edit` applied to the setup first.
fn sea_state(rules: &Rules, at: Pos, edit: impl FnOnce(&mut AdventureSetup)) -> AdventureState {
    let mut setup = coasted(
        &SEA_ROWS,
        vec![
            hero_at(1, 0, at, &[PEASANT]),
            hero_at(2, 1, p(7, 7), &[PEASANT]),
        ],
    );
    edit(&mut setup);
    started(rules, &setup).unwrap()
}

/// Red's hero 1 aboard `boat`, on its tile, with `points` and the sea's first maximum.
fn aboard(state: &mut AdventureState, boat: usize, points: u32) {
    state.put_aboard(HERO, boat);
    let hero = &mut state.heroes[0];
    (hero.movement, hero.max_movement) = (points, 1500);
}

/// Moves red's hero 1 along `path` with `store`; the outcome and the events, or the error (a
/// rejected move leaves the state and emits nothing). The boats keep their invariants.
fn sail(
    state: &mut AdventureState,
    rules: &Rules,
    store: &[Arc<HeroDetails>],
    path: &[Pos],
) -> Result<(AdventureOutcome, Vec<AdventureEvent>), AdventureCommandError> {
    let before = state.clone();
    let mut events = Vec::new();
    let command = AdventureCommand::MoveHero {
        hero: HERO,
        path: path.to_vec(),
    };
    let result = state.apply(&rules.0, &rules.1, store, RED, &command, &mut events);
    if result.is_err() {
        assert!(events.is_empty(), "a rejected move emitted events");
        assert_eq!(*state, before, "a rejected move changed the state");
    }
    assert!(state.check_boats(&rules.1));
    result.map(|outcome| (outcome, events))
}

/// Red's hero 1: its tile, its boat, its points and its day's maximum.
fn hero(state: &AdventureState) -> (Pos, Option<u32>, u32, u32) {
    let hero = state.hero(HERO).unwrap();
    (hero.pos, hero.boat, hero.movement, hero.max_movement)
}

/// The events but the steps.
fn told(events: &[AdventureEvent]) -> Vec<AdventureEventKind> {
    events
        .iter()
        .filter(|e| !matches!(e.kind, AdventureEventKind::HeroStepped { .. }))
        .map(|e| e.kind)
        .collect()
}

fn blocked(step: usize, error: StandError) -> AdventureCommandError {
    AdventureCommandError::Blocked { step, error }
}

// covers: ADV-SEA-002, ADV-MOV-006
#[test]
fn a_walker_boards_an_empty_boat_and_its_walk_ends_there() {
    let (rules, navigation, admiral) = sea_rules(true);
    let fresh = sea_state(&rules, p(2, 2), |_| {});
    let land = fresh.hero(HERO).unwrap().movement;
    assert_eq!(fresh.hero(HERO).unwrap().max_movement, land);
    let mut state = fresh.clone();
    let (outcome, events) = sail(&mut state, &rules, &[], &[p(3, 2)]).unwrap();
    // On the boat's tile in it, the sea's maximum (the table's first entry without details), no
    // points without free boarding (ADV-SEA-002); nothing else happens.
    assert_eq!(outcome, AdventureOutcome::default());
    assert_eq!(hero(&state), (p(3, 2), Some(0), 0, 1500));
    let boat = *state.boat(0).unwrap();
    assert_eq!(
        (boat.tile, boat.aboard, boat.captain, boat.owner),
        (p(3, 2), true, Some(HERO), Some(RED))
    );
    assert!(!state.is_empty_boat(p(3, 2)) && state.is_empty_boat(p(6, 3)));
    // The step, then the boarding, told to those that see the hero on the boat's tile.
    let AdventureEventKind::HeroStepped {
        to: Some(to),
        seen_to,
        ..
    } = events[0].kind
    else {
        panic!("{events:?}");
    };
    assert_eq!((to, events[1].seen_by), (p(3, 2), seen_to));
    assert_eq!(
        told(&events),
        [AdventureEventKind::Boarded {
            hero: HERO,
            boat: 0,
            at: p(3, 2),
        }]
    );
    // The boarding ends the walk; plain water is no step for a walker.
    let mut state = fresh.clone();
    assert_eq!(
        sail(&mut state, &rules, &[], &[p(3, 2), p(4, 2)]),
        Err(AdventureCommandError::PastVisit { step: 1 })
    );
    assert_eq!(
        sail(&mut state, &rules, &[], &[p(3, 1)]),
        Err(blocked(0, StandError::Terrain))
    );
    // With free boarding the points after the step, × the sea's maximum ÷ the land's; with
    // Navigation the sea's maximum its level's.
    let mut state = fresh.clone();
    let free = store(&[(admiral, 1)]);
    sail(&mut state, &rules, &free, &[p(3, 2)]).unwrap();
    let expected = (land - 100) * 1500 / land;
    assert_eq!(hero(&state), (p(3, 2), Some(0), expected, 1500));
    let mut state = fresh.clone();
    let both = store(&[(admiral, 1), (navigation, 2)]);
    sail(&mut state, &rules, &both, &[p(3, 2)]).unwrap();
    assert_eq!(hero(&state).2, (land - 100) * 3000 / land);
    // E8-1: the step's drop first (75 left after it, below the boat's cheapest step), then the
    // conversion: none.
    let mut state = fresh.clone();
    state.heroes[0].movement = 175;
    sail(&mut state, &rules, &free, &[p(3, 2)]).unwrap();
    assert_eq!(hero(&state).2, 0);
}

// covers: ADV-SEA-002, ADV-MON-006
#[test]
fn no_guard_attacks_on_a_boat_s_tile_and_a_water_monster_attacks_a_boat_at_once() {
    let (rules, _, _) = sea_rules(true);
    // A water monster at (4, 3), guarding the boat's tile (3, 2) and (4, 2) on water.
    let mut state = sea_state(&rules, p(2, 2), |setup| {
        let monster = world(&["M"], 1, vec![]).map.objects.remove(0);
        setup.map.objects.push(ObjectSetup {
            anchor: p(4, 3),
            ..monster
        });
    });
    assert_eq!(state.guard_of(&rules.1, p(3, 2)), Some(0));
    let (outcome, _) = sail(&mut state, &rules, &[], &[p(3, 2)]).unwrap();
    assert_eq!(outcome, AdventureOutcome::default());
    // Leaving onto a guarded water tile: the monster attacks at once.
    state.heroes[0].movement = 1500;
    let (outcome, _) = sail(&mut state, &rules, &[], &[p(4, 2)]).unwrap();
    assert_eq!(guard_visit(&outcome), Some(p(4, 3)));
    assert_eq!(hero(&state).0, p(4, 2));
}

fn row(xs: core::ops::RangeInclusive<u16>) -> Vec<Pos> {
    xs.map(|x| p(x, 0)).collect()
}

// covers: ADV-SEA-002, ADV-MOV-009, ADV-MOV-010
#[test]
fn a_flyer_and_a_water_walker_board_from_land_only() {
    let (rules, _, _) = sea_rules(true);
    let flyer = store(&[(FLYING, 3)]);
    let walker = store(&[(WATER_WALKING, 3)]);
    let state = |rows: &[&str]| {
        started(
            &rules,
            &coasted(rows, vec![hero_at(1, 0, p(0, 0), &[PEASANT])]),
        )
        .unwrap()
    };
    // From land (a stop node) each boards as the walker does; a water walk's boarding ends it,
    // a flight passes on over the boat (and stops nowhere on water).
    for (heroes, past) in [
        (&flyer, AdventureCommandError::NoStop { step: 1 }),
        (&walker, AdventureCommandError::PastVisit { step: 1 }),
    ] {
        let mut s = state(&["gb~gg"]);
        let (_, events) = sail(&mut s, &rules, heroes, &[p(1, 0)]).unwrap();
        assert_eq!(hero(&s), (p(1, 0), Some(0), 0, 1500));
        assert_eq!(told(&events).len(), 1);
        let mut s = state(&["gb~gg"]);
        assert_eq!(sail(&mut s, &rules, heroes, &row(1..=2)), Err(past));
    }
    // A flight passes over a boat from land or water (batch E8c's E8-17d, E8-17e), boarding none
    // from water; a water walk never steps onto it from water.
    let mut s = state(&["gb~bg"]);
    sail(&mut s, &rules, &flyer, &row(1..=4)).unwrap();
    assert_eq!(hero(&s).0, p(4, 0));
    let mut s = state(&["g~b~g"]);
    assert_eq!(
        sail(&mut s, &rules, &flyer, &row(1..=2)),
        Err(AdventureCommandError::NoStop { step: 1 })
    );
    assert_eq!(
        sail(&mut s, &rules, &walker, &row(1..=2)),
        Err(AdventureCommandError::Boat { step: 1 })
    );
}

// covers: ADV-SEA-005, GAM-BAT-004, GAM-MET-001, ADV-MOV-009, ADV-MOV-010
#[test]
fn a_hero_in_a_boat_is_visited_from_a_stop_and_flown_over() {
    let (rules, _, _) = sea_rules(true);
    let flyer = store(&[(FLYING, 3)]);
    let walker = store(&[(WATER_WALKING, 3)]);
    // Blue's hero 2 in the boat at (1, 0), red's own hero 3 in the one at (5, 0); red's hero 1 on
    // (0, 0).
    let start = || {
        let setup = coasted(
            &["gb~g~bg"],
            vec![
                hero_at(1, 0, p(0, 0), &[PEASANT]),
                hero_at(2, 1, p(6, 0), &[PEASANT]),
                hero_at(3, 0, p(3, 0), &[PEASANT]),
            ],
        );
        let mut state = started(&rules, &setup).unwrap();
        state.put_aboard(HeroId::new(2), 0);
        state.put_aboard(HeroId::new(3), 1);
        state
    };
    // From the shore a walker, a flyer and a water walker attack blue's boat (on the deck since
    // T74b: `sea_battles.rs`).
    for heroes in [&store(&[])[..], &flyer, &walker] {
        let mut s = start();
        let (outcome, _) = sail(&mut s, &rules, heroes, &[p(1, 0)]).unwrap();
        assert_eq!(outcome.battle.map(|b| b.tile), Some(p(1, 0)));
        assert_eq!(hero(&s).0, p(0, 0));
    }
    // A flight passes over it, from a stop node too.
    let mut s = start();
    sail(&mut s, &rules, &flyer, &row(1..=3)).unwrap();
    assert_eq!(hero(&s).0, p(3, 0));
    // From (3, 0): over the water at (4, 0) a flight passes the own boat to (6, 0), ends on it
    // nowhere, and a water walk neither stops nor visits from water.
    let path = [p(4, 0), p(5, 0)];
    let mut aside = s.clone();
    assert_eq!(
        sail(&mut aside, &rules, &flyer, &path),
        Err(AdventureCommandError::NoStop { step: 1 })
    );
    assert_eq!(
        sail(&mut aside, &rules, &walker, &path),
        Err(AdventureCommandError::NoStop { step: 1 })
    );
    sail(&mut aside, &rules, &flyer, &row(4..=6)).unwrap();
    assert_eq!(hero(&aside).0, p(6, 0));
    // From the shore at (6, 0) the own hero in its boat is met.
    let (outcome, _) = sail(&mut aside, &rules, &[], &[p(5, 0)]).unwrap();
    assert_eq!(
        outcome.meeting.map(|m| (m.visitor, m.visited)),
        Some((HERO, HeroId::new(3)))
    );
}

// covers: ADV-MOV-009, ADV-MOV-010
#[test]
fn a_flight_neither_ends_on_nor_passes_a_water_tile_an_object_is_visited_from() {
    // D-46 (batch E8c's E8-17a, E8-17b): an object's visitable tile on water, a blocker's or a
    // typed one's, closes to a flight; an object's blocked cell stays a flight's pass and no
    // water walk's.
    let (rules, _, _) = sea_rules(true);
    let flyer = store(&[(FLYING, 3)]);
    let walker = store(&[(WATER_WALKING, 3)]);
    for (c, open) in [('V', false), ('P', false), ('B', true)] {
        let row_text: std::string::String = ['g', '~', c, '~', 'g'].iter().collect();
        let mut setup = coasted(&[&row_text], vec![hero_at(1, 0, p(0, 0), &[PEASANT])]);
        set_terrain(&mut setup, p(2, 0), SEA);
        let fresh = started(&rules, &setup).unwrap();
        let mut s = fresh.clone();
        let flown = sail(&mut s, &rules, &flyer, &row(1..=4)).map(|_| hero(&s).0);
        let closed = blocked(1, StandError::Blocked);
        assert_eq!(flown, if open { Ok(p(4, 0)) } else { Err(closed) }, "{c}");
        let reach = fresh.reach(&rules.0, &rules.1, &flyer, HERO).unwrap();
        assert_eq!(reach.points_left(p(4, 0)).is_some(), open, "{c}");
        let mut s = fresh.clone();
        assert_eq!(
            sail(&mut s, &rules, &walker, &row(1..=4)).map(|_| ()),
            Err(closed),
            "{c}"
        );
    }
}

// covers: ADV-MOV-009, ADV-OBJ-004, ADV-PIK-018
#[test]
fn a_flight_up_off_a_closed_top_never_passes_a_water_tile_an_object_is_visited_from() {
    // E08 T75b (found by the plans' property, `h3-map`'s `check_sea_move`): a step up off the
    // visitable tile of an object not visited from above (a chart) is a flight's pass only
    // (ADV-OBJ-004), and D-46 closes a water tile an object is visited from to a flight: the two
    // together refuse the step; over plain water the flight passes.
    let (rules, _, _) = sea_rules(true);
    let flyer = store(&[(FLYING, 3)]);
    for (c, open) in [('V', false), ('g', true)] {
        let top: std::string::String = ['g', '#', c].iter().collect();
        let mut setup = coasted(
            &[&top, "gCg", "ggg"],
            vec![hero_at(1, 0, p(0, 1), &[PEASANT])],
        );
        set_terrain(&mut setup, p(2, 0), SEA);
        let mut s = started(&rules, &setup).unwrap();
        let flown = sail(&mut s, &rules, &flyer, &[p(1, 1), p(2, 0), p(2, 1)]);
        let expected = if open {
            Ok(p(2, 1))
        } else {
            Err(blocked(1, StandError::Blocked))
        };
        assert_eq!(flown.map(|_| hero(&s).0), expected, "{c}");
    }
}

// covers: ADV-SEA-004, ADV-MOV-005, ADV-MOV-006
#[test]
fn a_boat_sails_at_the_sea_s_prices_between_two_waters() {
    let (rules, _, _) = sea_rules(true);
    let fresh = sea_state(&rules, p(2, 2), |setup| {
        // A blocked water tile at (4, 1), rock at (2, 0)'s neighbour (3, 0).
        setup.map.objects.push(ObjectSetup {
            blocked: TileMask::from_bits(1),
            ..lying(ObjectTypeId::from_raw(0), p(4, 1), 0)
        });
        setup.map.objects.last_mut().unwrap().object_type = None;
        set_terrain(setup, p(3, 0), ROCK);
    });
    let mut state = fresh.clone();
    aboard(&mut state, 0, 1500);
    // Straight 100, diagonal 141 between two waters; the hero in its boat.
    sail(&mut state, &rules, &[], &[p(4, 2)]).unwrap();
    assert_eq!(hero(&state), (p(4, 2), Some(0), 1400, 1500));
    sail(&mut state, &rules, &[], &[p(5, 1)]).unwrap();
    assert_eq!(hero(&state).2, 1259);
    // Never onto a blocked water tile, an empty boat or rock; a diagonal past the island's land
    // never (0x4B2703).
    assert_eq!(
        sail(&mut state, &rules, &[], &[p(4, 1)]),
        Err(blocked(0, StandError::Blocked))
    );
    state.heroes[0].pos = p(6, 2);
    assert_eq!(
        sail(&mut state, &rules, &[], &[p(6, 3)]),
        Err(AdventureCommandError::Boat { step: 0 })
    );
    let mut s = fresh.clone();
    aboard(&mut s, 0, 1500);
    assert_eq!(
        sail(&mut s, &rules, &[], &[p(3, 1), p(3, 0)]),
        Err(blocked(1, StandError::Terrain))
    );
    assert_eq!(
        sail(&mut s, &rules, &[], &[p(4, 3), p(5, 4)]),
        Err(AdventureCommandError::Coast { step: 1 })
    );
    // A flyer in a boat sails: water's price, not its flight's (140 at level 1), and no pass over
    // the blocked tile.
    let mut s = fresh.clone();
    aboard(&mut s, 0, 1500);
    let flyer = store(&[(FLYING, 1)]);
    sail(&mut s, &rules, &flyer, &[p(4, 2)]).unwrap();
    assert_eq!(hero(&s).2, 1400);
    assert_eq!(
        sail(&mut s, &rules, &flyer, &[p(4, 1), p(4, 0)]),
        Err(blocked(0, StandError::Blocked))
    );
    // The drop at sea: below water's price none left.
    let mut s = fresh.clone();
    aboard(&mut s, 0, 150);
    sail(&mut s, &rules, &[], &[p(4, 2)]).unwrap();
    assert_eq!(hero(&s).2, 0);
}

// covers: ADV-SEA-004, ADV-MOV-005, ADV-MOV-006
#[test]
fn favorable_winds_lower_a_boat_s_step_and_its_drop_but_not_the_exception_s_check() {
    // Winds on columns 4 to 6 of rows 4 and 5: from them 66 straight, 93 diagonally; the drop's
    // cheapest step 66 onto them; the diagonal exception compares with water's 100 (D-43,
    // batch E8's E8-4).
    let (rules, _, _) = sea_rules(true);
    let fresh = sea_state(&rules, p(2, 2), |setup| {
        setup.map.objects.push(lying(WINDS, p(6, 5), 0x707));
    });
    assert!(fresh.map().tile(p(5, 5)).unwrap().winds);
    assert!(!fresh.map().tile(p(3, 4)).unwrap().winds);
    let from = |points| {
        let mut s = fresh.clone();
        aboard(&mut s, 0, points);
        s.heroes[0].pos = p(5, 5);
        s
    };
    let left = |points, to| {
        let mut s = from(points);
        sail(&mut s, &rules, &[], &[to]).map(|_| hero(&s).2)
    };
    assert_eq!(left(1500, p(5, 4)), Ok(1434));
    assert_eq!(left(1500, p(4, 4)), Ok(1407));
    // E8-4: with 80 the diagonal (93) is refused, the straight step (66) taken, nothing left.
    assert_eq!(
        left(80, p(4, 4)),
        Err(AdventureCommandError::TooFewPoints {
            step: 0,
            cost: 93,
            points: 80,
        })
    );
    assert_eq!(left(80, p(5, 4)), Ok(0));
    // 93 to 99 pay the diagonal, 100 to 141 the straight step for it, 142 the diagonal again.
    assert_eq!(left(95, p(4, 4)), Ok(0));
    assert_eq!(left(141, p(4, 4)), Ok(75));
    assert_eq!(left(142, p(4, 4)), Ok(0));
    // Onto a tile off the winds the drop's cheapest step is water's: 150 − 66 = 84 below it.
    let mut s = from(150);
    s.heroes[0].pos = p(4, 4);
    sail(&mut s, &rules, &[], &[p(3, 4)]).unwrap();
    assert_eq!(hero(&s).2, 0);
}

// covers: ADV-SEA-005, ADV-OBJ-003
#[test]
fn a_boat_visits_objects_on_water_from_its_tile_and_enters_the_entered_ones() {
    let (rules, _, _) = sea_rules(true);
    // A wood pile at (3, 0), a chart (a water Cartographer) at (5, 0), a blocker's visitable
    // tile at (7, 0), all on water.
    let mut setup = coasted(&["gb~P~C~Vg"], vec![hero_at(1, 0, p(0, 0), &[PEASANT])]);
    for x in [3, 5, 7] {
        set_terrain(&mut setup, p(x, 0), SEA);
    }
    let mut state = started(&rules, &setup).unwrap();
    aboard(&mut state, 0, 1500);
    let (outcome, _) = sail(&mut state, &rules, &[], &row(2..=3)).unwrap();
    // The pile from the boat's tile: the step's price paid, the hero stays, the pile leaves.
    assert_eq!(outcome.gains.len(), 1);
    assert_eq!(hero(&state), (p(2, 0), Some(0), 1300, 1500));
    assert!(state.is_removed(0));
    // Its tile is plain water now; the chart is entered, the hero in its boat on its tile.
    let (outcome, _) = sail(&mut state, &rules, &[], &row(3..=5)).unwrap();
    assert!(outcome.offer.is_some());
    assert_eq!(hero(&state).0, p(5, 0));
    // A visit not built is refused.
    assert_eq!(
        sail(&mut state, &rules, &[], &row(6..=7)),
        Err(AdventureCommandError::Visit { step: 1 })
    );
}

// covers: ADV-SEA-003, ADV-MOV-006
#[test]
fn a_boat_lands_on_a_free_coastal_tile_and_is_left_behind_facing_the_landing_s_way() {
    let (rules, _, admiral) = sea_rules(true);
    let fresh = sea_state(&rules, p(2, 2), |_| {});
    let land = fresh.hero(HERO).unwrap().movement;
    let mut state = fresh.clone();
    aboard(&mut state, 0, 1500);
    assert_eq!(
        state.observe(&rules.0, &rules.1, RED).heroes[0].boat,
        Some(0)
    );
    let (outcome, events) = sail(&mut state, &rules, &[], &[p(2, 2)]).unwrap();
    // On land with the land's maximum and no points without free boarding (A-28 not
    // reproduced); the boat empty on the water tile behind, facing west (6), its owner and
    // captain kept.
    assert_eq!(outcome, AdventureOutcome::default());
    assert_eq!(hero(&state), (p(2, 2), None, 0, land));
    let boat = *state.boat(0).unwrap();
    assert_eq!(
        (
            boat.tile,
            boat.facing,
            boat.aboard,
            boat.captain,
            boat.owner
        ),
        (p(3, 2), 6, false, Some(HERO), Some(RED))
    );
    assert!(state.is_empty_boat(p(3, 2)));
    // Told to those that saw the hero on the water tile, where the boat now lies.
    let AdventureEventKind::HeroStepped {
        to: Some(to),
        seen_from,
        ..
    } = events[0].kind
    else {
        panic!("{events:?}");
    };
    assert_eq!((to, events[1].seen_by), (p(2, 2), seen_from));
    assert_eq!(
        told(&events),
        [AdventureEventKind::Landed {
            hero: HERO,
            boat: 0,
            at: p(3, 2),
            facing: 6,
        }]
    );
    assert!(events[1].seen_by.contains(RED));
    let view = state.observe(&rules.0, &rules.1, RED);
    assert_eq!(view.heroes[0].boat, None);
    assert!(view.boats.contains(&crate::BoatView {
        tile: p(3, 2),
        kind: 0,
        facing: 6,
    }));
    // The landing ends the walk; the hero boards its boat again.
    let mut s = fresh.clone();
    aboard(&mut s, 0, 1500);
    assert_eq!(
        sail(&mut s, &rules, &[], &[p(2, 2), p(1, 2)]),
        Err(AdventureCommandError::PastVisit { step: 1 })
    );
    state.heroes[0].movement = land;
    sail(&mut state, &rules, &[], &[p(3, 2)]).unwrap();
    assert_eq!(hero(&state).1, Some(0));
    // With free boarding: the points after the step and its drop × the land's ÷ the sea's
    // maximum; E8-2: 59 after the step drop to 0 first.
    let free = store(&[(admiral, 1)]);
    let mut s = fresh.clone();
    aboard(&mut s, 0, 1500);
    sail(&mut s, &rules, &free, &[p(2, 2)]).unwrap();
    assert_eq!(hero(&s).2, 1400 * land / 1500);
    let mut s = fresh.clone();
    aboard(&mut s, 0, 159);
    sail(&mut s, &rules, &free, &[p(2, 2)]).unwrap();
    assert_eq!(hero(&s).2, 0);
    // A diagonal landing has no side test (the side (2, 3) is land) and faces its way: down
    // left (5); E8-15's south is 4.
    for (from, to, facing) in [(p(3, 3), p(2, 4), 5), (p(4, 5), p(4, 6), 4)] {
        let mut s = fresh.clone();
        aboard(&mut s, 0, 1500);
        s.heroes[0].pos = from;
        sail(&mut s, &rules, &[], &[to]).unwrap();
        assert_eq!(hero(&s).0, to);
        assert_eq!(
            (s.boat(0).unwrap().tile, s.boat(0).unwrap().facing),
            (from, facing)
        );
    }
}

// covers: ADV-SEA-003
#[test]
fn no_boat_lands_off_the_coast_nor_on_an_object_a_hole_or_a_hero() {
    // D-42 (batch E8's E8-3): only a free coastal tile.
    let (rules, _, _) = sea_rules(true);
    let state = sea_state(&rules, p(2, 2), |setup| {
        setup.map.tiles[8 + 2].coast = false;
        setup.map.objects.push(lying(HOLE, p(2, 3), 1));
        setup.map.objects.push(ObjectSetup {
            visitable: TileMask::from_bits(1),
            object_type: None,
            ..lying(ObjectTypeId::from_raw(0), p(2, 4), 0)
        });
        setup.heroes[1].pos = p(2, 5);
    });
    assert!(state.map().tile(p(2, 3)).unwrap().hole);
    for (from, to) in [
        (p(3, 1), p(2, 1)),
        (p(3, 3), p(2, 3)),
        (p(3, 4), p(2, 4)),
        (p(3, 5), p(2, 5)),
    ] {
        let mut s = state.clone();
        aboard(&mut s, 0, 1500);
        s.heroes[0].pos = from;
        assert_eq!(
            sail(&mut s, &rules, &[], &[to]),
            Err(AdventureCommandError::Landing { step: 0 }),
            "{to:?}"
        );
    }
}

// covers: ADV-SEA-003, ADV-MON-006
#[test]
fn a_landing_s_guards_attack_once_the_hero_is_ashore() {
    let (rules, _, _) = sea_rules(true);
    let mut state = sea_state(&rules, p(2, 2), |setup| {
        let monster = world(&["M"], 1, vec![]).map.objects.remove(0);
        setup.map.objects.push(ObjectSetup {
            anchor: p(1, 4),
            ..monster
        });
    });
    aboard(&mut state, 0, 1500);
    state.heroes[0].pos = p(3, 4);
    let (outcome, events) = sail(&mut state, &rules, &[], &[p(2, 4)]).unwrap();
    assert_eq!(guard_visit(&outcome), Some(p(1, 4)));
    assert_eq!(hero(&state).0, p(2, 4));
    assert_eq!(hero(&state).1, None);
    assert!(matches!(
        told(&events)[..],
        [AdventureEventKind::Landed { .. }]
    ));
}

// covers: ADV-MOV-007, ADV-SEA-002, GAM-BAT-004
#[test]
fn a_walker_reaches_a_boat_and_a_hero_in_a_boat_and_goes_no_further() {
    let (rules, _, admiral) = sea_rules(true);
    // The boat at (3, 2) beside the walker: a boarding, the points the move leaves (0 without
    // free boarding), and no water beyond it.
    for (store, free) in [(store(&[]), false), (store(&[(admiral, 1)]), true)] {
        let fresh = sea_state(&rules, p(2, 2), |_| {});
        let reach = fresh.reach(&rules.0, &rules.1, &store, HERO).unwrap();
        assert_eq!(reach.path(p(3, 2)), Some(vec![p(3, 2)]));
        let mut moved = fresh.clone();
        sail(&mut moved, &rules, &store, &[p(3, 2)]).unwrap();
        assert_eq!(hero(&moved).1, Some(0));
        assert_eq!(reach.points_left(p(3, 2)), Some(hero(&moved).2));
        assert_eq!(hero(&moved).2 > 0, free);
        assert_eq!(reach.points_left(p(4, 2)), None);
        assert_eq!(reach.points_left(p(4, 3)), None);
    }
    // On the island at (5, 3), blue's hero in the boat at (6, 3): visited from the island, the
    // island's only way out.
    let mut state = sea_state(&rules, p(5, 3), |_| {});
    state.put_aboard(HeroId::new(2), 1);
    let reach = state.reach(&rules.0, &rules.1, &[], HERO).unwrap();
    let tiles: Vec<(Pos, u32)> = reach.tiles().collect();
    assert_eq!(tiles.len(), 1);
    assert_eq!(tiles[0].0, p(6, 3));
    assert_eq!(reach.path(p(6, 3)), Some(vec![p(6, 3)]));
    let (outcome, _) = sail(&mut state, &rules, &[], &[p(6, 3)]).unwrap();
    assert_eq!(outcome.battle.map(|b| b.tile), Some(p(6, 3)));
    assert_eq!(hero(&state).0, p(5, 3));
    assert_eq!(hero(&state).2, tiles[0].1);
}

// covers: ADV-MOV-007, ADV-SEA-003, ADV-SEA-004, ADV-SEA-005, ADV-MON-006
#[test]
fn a_boat_reaches_the_sea_its_landings_and_its_visits_as_destinations() {
    let (rules, _, admiral) = sea_rules(true);
    // A water monster at (6, 0), guarding (5, 0), (5, 1), (6, 1), (7, 1) and (7, 0) (ADV-MON-005).
    let fresh = sea_state(&rules, p(2, 2), |setup| {
        let monster = world(&["M"], 1, vec![]).map.objects.remove(0);
        setup.map.objects.push(ObjectSetup {
            anchor: p(6, 0),
            ..monster
        });
    });
    let mut state = fresh.clone();
    aboard(&mut state, 0, 1500);
    let reach = state.reach(&rules.0, &rules.1, &[], HERO).unwrap();
    // At sea: 100 a straight step, 141 a diagonal between two waters.
    assert_eq!(reach.points_left(p(4, 2)), Some(1400));
    assert_eq!(reach.points_left(p(4, 3)), Some(1359));
    assert_eq!(reach.path(p(4, 3)), Some(vec![p(4, 3)]));
    // Never onto the empty boat at (6, 3); the island and the shore are landings (0 points
    // without free boarding), beyond which nothing is reached; land off the coast never.
    assert_eq!(reach.points_left(p(6, 3)), None);
    for landing in [p(5, 3), p(2, 2), p(3, 6)] {
        assert_eq!(reach.points_left(landing), Some(0), "{landing:?}");
    }
    for inland in [p(1, 2), p(3, 7)] {
        assert_eq!(reach.points_left(inland), None, "{inland:?}");
    }
    // The guarded tiles are destinations, where the monster attacks: (7, 0) and the monster's
    // own tile, reached only over them, are not reached.
    let path = reach.path(p(6, 1)).unwrap();
    let mut moved = state.clone();
    let (outcome, _) = sail(&mut moved, &rules, &[], &path).unwrap();
    assert_eq!(guard_visit(&outcome), Some(p(6, 0)));
    assert_eq!(reach.points_left(p(7, 0)), None);
    assert_eq!(reach.points_left(p(6, 0)), None);
    // Every path reported is the move's, ending on its tile with the points it leaves.
    for (pos, left) in reach.tiles() {
        let mut moved = state.clone();
        sail(&mut moved, &rules, &[], &reach.path(pos).unwrap()).unwrap();
        assert_eq!((hero(&moved).0, hero(&moved).2), (pos, left));
    }
    // With free boarding a landing keeps its points, converted to the land's maximum.
    let store = store(&[(admiral, 1)]);
    let reach = state.reach(&rules.0, &rules.1, &store, HERO).unwrap();
    let land = fresh.hero(HERO).unwrap().max_movement;
    assert_eq!(reach.points_left(p(2, 2)), Some(1400 * land / 1500));
    let mut moved = state.clone();
    sail(&mut moved, &rules, &store, &[p(2, 2)]).unwrap();
    assert_eq!(hero(&moved), (p(2, 2), None, 1400 * land / 1500, land));
}

// covers: ADV-MOV-007, ADV-OBJ-004, ADV-SEA-002
#[test]
fn a_walker_reaches_no_boat_it_cannot_pay_or_must_enter_across_an_object_s_top() {
    let (rules, _, _) = sea_rules(true);
    // 50 points pay no step from grass (100): the boat beside is not reached, nor boarded.
    let mut state = sea_state(&rules, p(2, 2), |_| {});
    state.heroes[0].movement = 50;
    let reach = state.reach(&rules.0, &rules.1, &[], HERO).unwrap();
    assert_eq!(reach.points_left(p(3, 2)), None);
    assert!(matches!(
        sail(&mut state, &rules, &[], &[p(3, 2)]),
        Err(AdventureCommandError::TooFewPoints { step: 0, .. })
    ));
    // Red's hero 2 in its boat on a chart's water tile (entered, not visited from above) at
    // (1, 1): hero 1 above meets it from no tile of the row above (ADV-OBJ-004), as the move.
    let mut setup = coasted(
        &["ggg", "~Cb", "ggg"],
        vec![
            hero_at(1, 0, p(1, 0), &[PEASANT]),
            hero_at(2, 0, p(1, 2), &[PEASANT]),
        ],
    );
    set_terrain(&mut setup, p(1, 1), SEA);
    let mut state = started(&rules, &setup).unwrap();
    state.put_aboard(HeroId::new(2), 0);
    state.heroes[1].pos = p(1, 1);
    assert!(state.check_boats(&rules.1));
    let reach = state.reach(&rules.0, &rules.1, &[], HERO).unwrap();
    assert_eq!(reach.points_left(p(1, 1)), None);
    for down in [p(0, 1), p(1, 1)] {
        let mut s = state.clone();
        assert!(sail(&mut s, &rules, &[], &[down]).is_err(), "{down:?}");
    }
}

// covers: ADV-SEA-004, ADV-MOV-005, ADV-MOV-006
#[test]
fn a_boat_s_reachable_tiles_follow_the_sea_s_prices_and_its_drop() {
    let (rules, _, _) = sea_rules(true);
    // Favorable Winds at (1, 1): (2, 1) is cheaper over them (141 + 66) than by the diagonal
    // from (1, 0) found first (100 + 141).
    let mut setup = coasted(
        &["b~~", "~~~", "ggg"],
        vec![hero_at(1, 0, p(0, 2), &[PEASANT])],
    );
    setup.map.objects.push(lying(WINDS, p(1, 1), 1));
    let mut state = started(&rules, &setup).unwrap();
    aboard(&mut state, 0, 1500);
    let reach = state.reach(&rules.0, &rules.1, &[], HERO).unwrap();
    assert_eq!(reach.points_left(p(1, 1)), Some(1359));
    assert_eq!(reach.points_left(p(2, 1)), Some(1293));
    assert_eq!(reach.path(p(2, 1)), Some(vec![p(1, 1), p(2, 1)]));
    let mut moved = state.clone();
    sail(&mut moved, &rules, &[], &[p(1, 1), p(2, 1)]).unwrap();
    assert_eq!(hero(&moved).2, 1293);
    // With 100 points the diagonal costs the straight step (ADV-MOV-005's exception against
    // water's price) and both drop to 0, from where nothing more is reached.
    aboard(&mut state, 0, 100);
    let reach = state.reach(&rules.0, &rules.1, &[], HERO).unwrap();
    let tiles: Vec<(Pos, u32)> = reach.tiles().collect();
    assert_eq!(tiles, vec![(p(1, 0), 0), (p(0, 1), 0), (p(1, 1), 0)],);
}

// covers: ADV-MOV-007, ADV-SEA-002
#[test]
fn a_flyer_s_boarding_in_the_reachable_tiles_drops_as_a_boat_s() {
    // A level-1 flyer with free boarding under the strict profile, whose own drop on water is
    // its flight's 140: its boarding keeps 130 of 230 (the boat's drop, 100), converted, as the
    // move does; without free boarding none.
    let (strict, _, admiral) = sea_rules(false);
    let mut s = started(
        &strict,
        &coasted(&["gb~gg"], vec![hero_at(1, 0, p(0, 0), &[PEASANT])]),
    )
    .unwrap();
    s.heroes[0].movement = 230;
    let land = s.hero(HERO).unwrap().max_movement;
    for (flyer, left) in [
        (store(&[(FLYING, 1), (admiral, 1)]), 130 * 1500 / land),
        (store(&[(FLYING, 1)]), 0),
    ] {
        let reach = s.reach(&strict.0, &strict.1, &flyer, HERO).unwrap();
        assert_eq!(reach.points_left(p(1, 0)), Some(left));
        assert_eq!(reach.path(p(1, 0)), Some(vec![p(1, 0)]));
        let mut moved = s.clone();
        sail(&mut moved, &strict, &flyer, &[p(1, 0)]).unwrap();
        assert_eq!(hero(&moved).2, left);
    }
}

// covers: ADV-SEA-005, GAM-BAT-004, ADV-VIS-005
#[test]
fn a_boat_attacks_another_boat_s_hero_past_a_gate_whose_key_it_holds() {
    let (rules, _, _) = sea_rules(true);
    // Red's hero 1 in the boat at (1, 0), blue's hero 2 in the one at (5, 0), a border gate on
    // the water at (3, 0) whose key red holds.
    let mut setup = coasted(
        &["gb~X~bg"],
        vec![
            hero_at(1, 0, p(0, 0), &[PEASANT]),
            hero_at(2, 1, p(6, 0), &[PEASANT]),
        ],
    );
    set_terrain(&mut setup, p(3, 0), SEA);
    let mut state = started(&rules, &setup).unwrap();
    aboard(&mut state, 0, 1500);
    state.put_aboard(HeroId::new(2), 1);
    state.give_key(RED, 0);
    let (outcome, _) = sail(&mut state, &rules, &[], &row(2..=5)).unwrap();
    // The gate's tile crossed as plain water; blue's boat attacked from the neighbouring water
    // tile (the step toward it paid, ADV-OBJ-003), where red's hero stays in its boat.
    assert_eq!(outcome.battle.map(|b| b.tile), Some(p(5, 0)));
    assert_eq!(hero(&state), (p(4, 0), Some(0), 1100, 1500));
}
