//! Tests of a flying or water-walking hero's passage (E08 T41; ADV-MOV-009, ADV-MOV-010; design
//! note §7's table): what `MoveHero` lets it pass over and stop on, its visits and guards, and the
//! reachable-tiles query's two kinds of node. Batch E5's rows as sources: E5-5 (a water walk),
//! E5-7 (no stop on a tree or water), E5-9 (no flight over unexplored tiles, `ADV-QUIRK-15`),
//! E5-11 (a flight over a border gate without its key).

use alloc::sync::Arc;

use h3_heroes::HeroDetails;

use super::*;
use crate::command::{AdventureCommand, AdventureCommandError, AdventureEvent, AdventureEventKind};
use crate::setup::ObjectSetup;
use crate::testing::{FLYING, PEASANT, WATER_WALKING, hero_at, hero_details, hero_rules, world};

const RED: PlayerId = PlayerId::new(0);

fn p(x: u16, y: u16) -> Pos {
    Pos::new(x, y, 0)
}

/// The store of heroes' details: hero 1 with `skills`, hero 0 and the rest without.
fn store(skills: &[(h3_content::SkillId, u8)]) -> Vec<Arc<HeroDetails>> {
    vec![Arc::new(hero_details(&[])), Arc::new(hero_details(skills))]
}

/// An Expert flyer's store (the fly price 100, as Angel Wings').
fn flyer() -> Vec<Arc<HeroDetails>> {
    store(&[(FLYING, 3)])
}

/// An Expert water walker's store (the water-walk price 100, as Boots of Levitation's).
fn water_walker() -> Vec<Arc<HeroDetails>> {
    store(&[(WATER_WALKING, 3)])
}

/// The started state of [`world`]'s `rows` under the faithful [`hero_rules`], red's hero 1 (an
/// army of peasants: 1300 points) on `at` and `others`.
fn flight_world(
    rules: &(Ruleset, AdventureRuleset),
    rows: &[&str],
    at: Pos,
    others: Vec<HeroSetup>,
) -> AdventureState {
    let mut heroes = vec![hero_at(1, 0, at, &[PEASANT])];
    heroes.extend(others);
    crate::testing::started(rules, &world(rows, 1, heroes)).unwrap()
}

/// Moves red's hero 1 along `path` with `store`; the events, or the error.
fn fly(
    state: &mut AdventureState,
    rules: &(Ruleset, AdventureRuleset),
    store: &[Arc<HeroDetails>],
    path: &[Pos],
) -> Result<(AdventureOutcome, Vec<AdventureEvent>), AdventureCommandError> {
    let before = state.clone();
    let mut events = Vec::new();
    let command = AdventureCommand::MoveHero {
        hero: HeroId::new(1),
        path: path.to_vec(),
    };
    let result = state.apply(&rules.0, &rules.1, store, RED, &command, &mut events);
    if result.is_err() {
        assert!(events.is_empty(), "a rejected move emitted events");
        assert_eq!(*state, before, "a rejected move changed the state");
    }
    result.map(|outcome| (outcome, events))
}

fn at(state: &AdventureState) -> (Pos, u32) {
    let hero = state.hero(HeroId::new(1)).unwrap();
    (hero.pos, hero.movement)
}

fn row(x: u16) -> Vec<Pos> {
    (1..=x).map(|x| p(x, 0)).collect()
}

// covers: ADV-MOV-009, ADV-MOV-010, ADV-MOV-007
#[test]
fn a_flight_passes_over_obstacles_and_water_and_ends_only_on_land() {
    // §7: a flight passes blocked tiles and water, a water walk plain water only, a walk neither;
    // a move ends only where the hero may stop (D-25, batch E5's E5-7: no path onto a tree or
    // water). The fly price 100 from the obstacles and the water, the lesser from grass.
    let rules = hero_rules(true);
    let rows = ["gBB~~gg"];
    let mut state = flight_world(&rules, &rows, p(0, 0), vec![]);
    let (_, events) = fly(&mut state, &rules, &flyer(), &row(5)).unwrap();
    assert_eq!(at(&state), (p(5, 0), 800));
    let steps = events
        .iter()
        .filter(|e| matches!(e.kind, AdventureEventKind::HeroStepped { .. }))
        .count();
    assert_eq!(steps, 5);
    let mut state = flight_world(&rules, &rows, p(0, 0), vec![]);
    for (end, step) in [(2, 1), (3, 2), (4, 3)] {
        assert_eq!(
            fly(&mut state, &rules, &flyer(), &row(end)),
            Err(AdventureCommandError::NoStop { step })
        );
    }
    // A walk and a water walk stop at the obstacle.
    let blocked = Err(AdventureCommandError::Blocked {
        step: 0,
        error: StandError::Blocked,
    });
    assert_eq!(fly(&mut state, &rules, &[], &row(1)).map(|_| ()), blocked);
    assert_eq!(
        fly(&mut state, &rules, &water_walker(), &row(1)).map(|_| ()),
        blocked
    );
}

// covers: ADV-MOV-010
#[test]
fn a_water_walk_crosses_plain_water_and_lands_on_the_far_shore() {
    // Batch E5's E5-5 (Boots of Levitation): across the lake 450, the sand's 150 onto the water
    // and 100 a step after; no stop on the water; water an object blocks is not walked.
    let rules = hero_rules(true);
    let mut state = flight_world(&rules, &["s~~~s"], p(0, 0), vec![]);
    assert_eq!(
        fly(&mut state, &rules, &water_walker(), &row(3)),
        Err(AdventureCommandError::NoStop { step: 2 })
    );
    fly(&mut state, &rules, &water_walker(), &row(4)).unwrap();
    assert_eq!(at(&state), (p(4, 0), 850));
    // Without the levels, water is no land (ADV-MOV-007).
    let mut state = flight_world(&rules, &["s~~~s"], p(0, 0), vec![]);
    assert_eq!(
        fly(&mut state, &rules, &[], &row(1)),
        Err(AdventureCommandError::Blocked {
            step: 0,
            error: StandError::Terrain
        })
    );
    // An obstacle on the water: a flight passes it, a water walk does not.
    let mut setup = world(&["g~~g"], 1, vec![hero_at(1, 0, p(0, 0), &[PEASANT])]);
    setup.map.objects.push(ObjectSetup {
        anchor: p(1, 0),
        blocked: TileMask::from_bits(1),
        visitable: TileMask::default(),
        object_type: None,
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
    });
    let mut state = crate::testing::started(&rules, &setup).unwrap();
    assert_eq!(
        fly(&mut state, &rules, &water_walker(), &row(3)),
        Err(AdventureCommandError::Blocked {
            step: 0,
            error: StandError::Blocked
        })
    );
    fly(&mut state, &rules, &flyer(), &row(3)).unwrap();
    assert_eq!(at(&state).0, p(3, 0));
}

// covers: ADV-MOV-009, ADV-QUIRK-15
#[test]
fn a_flight_never_crosses_an_unexplored_tile() {
    // ADV-QUIRK-15 (A-13 fixed for now): batch E5's E5-9 flyer flew straight through an
    // unexplored strip; the engine's moves stay on explored tiles (ADV-MOV-008). The hero has
    // explored up to (5, 0); the obstacles beyond are hidden.
    let rules = hero_rules(true);
    let mut state = flight_world(&rules, &["ggggBBBBBBgggggg"], p(0, 0), vec![]);
    assert_eq!(
        fly(&mut state, &rules, &flyer(), &row(11)),
        Err(AdventureCommandError::Unexplored { step: 5 })
    );
    let reach = state
        .reach(&rules.0, &rules.1, &flyer(), HeroId::new(1))
        .unwrap();
    let tiles: Vec<Pos> = reach.tiles().map(|(pos, _)| pos).collect();
    assert_eq!(tiles, vec![p(1, 0), p(2, 0), p(3, 0)]);
    // Off the map: as a walk.
    assert_eq!(
        fly(&mut state, &rules, &flyer(), &[p(0, 1)]),
        Err(AdventureCommandError::Blocked {
            step: 0,
            error: StandError::Outside
        })
    );
}

// covers: ADV-MOV-009, ADV-OBJ-003, ADV-VIS-005
#[test]
fn a_flight_visits_only_at_its_end_and_from_a_stop() {
    // §7: a flight passes over a pile without the visit, visits it as its last step from a tile it
    // may stop on (the visit from the tile before, ADV-OBJ-003), never from a tile it only passes
    // over; an entered object (a tower) is visited from any node; batch E5's E5-11: a flight
    // passes over a border gate without its key. A water walk visits from a stop only.
    let rules = hero_rules(true);
    let rows = ["gPgBPg~OgX~Pg"];
    let mut state = flight_world(&rules, &rows, p(0, 0), vec![]);
    // Over the pile: no visit, the pile stays.
    let (outcome, _) = fly(&mut state, &rules, &flyer(), &row(2)).unwrap();
    assert_eq!(outcome, AdventureOutcome::default());
    assert_eq!(at(&state), (p(2, 0), 1100));
    // From (2, 0) over the obstacle onto the pile: a visit from a pass, refused.
    assert_eq!(
        fly(&mut state, &rules, &flyer(), &[p(3, 0), p(4, 0)]),
        Err(AdventureCommandError::NoStop { step: 1 })
    );
    // The pile beside (0, 0), from a stop: visited, the hero stays.
    let mut state = flight_world(&rules, &rows, p(0, 0), vec![]);
    let (outcome, _) = fly(&mut state, &rules, &flyer(), &[p(1, 0)]).unwrap();
    assert!(!outcome.gains.is_empty());
    assert_eq!(at(&state), (p(0, 0), 1200));
    // Over the water onto the tower: entered from a pass.
    let mut state = flight_world(&rules, &rows, p(5, 0), vec![]);
    fly(&mut state, &rules, &flyer(), &[p(6, 0), p(7, 0)]).unwrap();
    assert_eq!(at(&state).0, p(7, 0));
    // Over the gate without its key and over the water, onto the pile from a pass: refused; to
    // the land beyond: landed.
    let mut state = flight_world(&rules, &rows, p(8, 0), vec![]);
    let path = [p(9, 0), p(10, 0), p(11, 0)];
    assert_eq!(
        fly(&mut state, &rules, &flyer(), &path),
        Err(AdventureCommandError::NoStop { step: 2 })
    );
    let path = [p(9, 0), p(10, 0), p(11, 0), p(12, 0)];
    fly(&mut state, &rules, &flyer(), &path).unwrap();
    assert_eq!(at(&state).0, p(12, 0));
    // A water walk: onto the tower from the water (entered), onto a pile from the water refused.
    let mut state = flight_world(&rules, &rows, p(5, 0), vec![]);
    fly(&mut state, &rules, &water_walker(), &[p(6, 0), p(7, 0)]).unwrap();
    assert_eq!(at(&state).0, p(7, 0));
    let mut state = flight_world(&rules, &["g~P"], p(0, 0), vec![]);
    assert_eq!(
        fly(&mut state, &rules, &water_walker(), &[p(1, 0), p(2, 0)]),
        Err(AdventureCommandError::NoStop { step: 1 })
    );
    // A gate whose key the owner holds is a free tile, stopped on.
    let mut state = flight_world(&rules, &rows, p(8, 0), vec![]);
    state.give_key(RED, 0);
    fly(&mut state, &rules, &flyer(), &[p(9, 0)]).unwrap();
    assert_eq!(at(&state).0, p(9, 0));
}

// covers: ADV-MOV-009, ADV-MOV-010
#[test]
fn a_flight_passes_objects_whose_visit_is_not_built_and_heroes() {
    // §7: an object's visitable tile no visit is built for (`V`, a town's `T`) a flight passes
    // and a water walk does not enter (as a walk, `Visit`); an own hero's tile a flight passes, a
    // water walk does not; another player's hero is visited from a stop (a battle) and passed by
    // a flight.
    let rules = hero_rules(true);
    let rows = ["gVTgggg"];
    let mut state = flight_world(&rules, &rows, p(0, 0), vec![]);
    assert_eq!(
        fly(&mut state, &rules, &water_walker(), &row(1)),
        Err(AdventureCommandError::Visit { step: 0 })
    );
    assert_eq!(
        fly(&mut state, &rules, &flyer(), &row(2)),
        Err(AdventureCommandError::NoStop { step: 1 })
    );
    fly(&mut state, &rules, &flyer(), &row(3)).unwrap();
    assert_eq!(at(&state).0, p(3, 0));
    // Heroes: red's 2 on (4, 0), blue's 3 on (6, 0).
    let others = vec![
        hero_at(2, 0, p(4, 0), &[PEASANT]),
        hero_at(3, 1, p(6, 0), &[PEASANT]),
    ];
    let rows = ["gggggggg"];
    let mut state = flight_world(&rules, &rows, p(3, 0), others.clone());
    assert_eq!(
        fly(&mut state, &rules, &water_walker(), &[p(4, 0)]),
        Err(AdventureCommandError::Blocked {
            step: 0,
            error: StandError::Occupied
        })
    );
    assert_eq!(
        fly(&mut state, &rules, &flyer(), &[p(4, 0)]),
        Err(AdventureCommandError::NoStop { step: 0 })
    );
    // Over the own hero onto (5, 0), then over the blue hero to (7, 0).
    fly(&mut state, &rules, &flyer(), &[p(4, 0), p(5, 0)]).unwrap();
    fly(&mut state, &rules, &flyer(), &[p(6, 0), p(7, 0)]).unwrap();
    assert_eq!(at(&state).0, p(7, 0));
    // From a stop, the blue hero is visited: a battle; from over the own hero, refused.
    let mut state = flight_world(&rules, &rows, p(3, 0), others.clone());
    let (outcome, _) = fly(&mut state, &rules, &flyer(), &[p(4, 0), p(5, 0), p(6, 0)]).unwrap();
    assert!(outcome.battle.is_some());
    assert_eq!(at(&state).0, p(5, 0));
    let blue = vec![hero_at(3, 1, p(4, 0), &[PEASANT])];
    let mut state = flight_world(&rules, &["ggg~ggg"], p(2, 0), blue);
    for store in [water_walker(), flyer()] {
        assert_eq!(
            fly(&mut state, &rules, &store, &[p(3, 0), p(4, 0)]),
            Err(AdventureCommandError::NoStop { step: 1 })
        );
    }
}

// covers: ADV-MOV-009, ADV-MON-006
#[test]
fn a_flight_meets_a_guard_only_where_it_ends() {
    // §7, the original's search (0x56BBCE) and move (0x48147D): a flight goes on over guarded
    // tiles and a monster's tile, the guard visiting only at its last tile; a walk and a water
    // walk end on the first guarded tile. The monster on (2, 0) guards (1, 0) and (3, 0).
    let rules = hero_rules(true);
    let rows = ["ggMgg"];
    let mut state = flight_world(&rules, &rows, p(0, 0), vec![]);
    let (outcome, _) = fly(&mut state, &rules, &flyer(), &row(4)).unwrap();
    assert_eq!(outcome, AdventureOutcome::default());
    assert_eq!(at(&state), (p(4, 0), 900));
    let mut state = flight_world(&rules, &rows, p(0, 0), vec![]);
    let (outcome, _) = fly(&mut state, &rules, &flyer(), &row(3)).unwrap();
    assert_ne!(outcome, AdventureOutcome::default());
    assert_eq!(at(&state).0, p(3, 0));
    for store in [Vec::new(), water_walker()] {
        let mut state = flight_world(&rules, &rows, p(0, 0), vec![]);
        assert_eq!(
            fly(&mut state, &rules, &store, &row(2)),
            Err(AdventureCommandError::PastVisit { step: 1 })
        );
        let reach = state
            .reach(&rules.0, &rules.1, &store, HeroId::new(1))
            .unwrap();
        assert_eq!(reach.points_left(p(1, 0)), Some(1200));
        assert_eq!(reach.points_left(p(3, 0)), None);
    }
    let reach = state_reach(
        &flight_world(&rules, &rows, p(0, 0), vec![]),
        &rules,
        &flyer(),
    );
    assert_eq!(reach.points_left(p(4, 0)), Some(900));
    assert_eq!(reach.path(p(4, 0)), Some(row(4)));
    // The monster's tile: visited from (1, 0), a stop; passed over.
    assert_eq!(reach.points_left(p(2, 0)), Some(1100));
    assert_eq!(reach.path(p(2, 0)), Some(row(2)));
}

/// The reachable tiles of red's hero 1 with `store`.
fn state_reach(
    state: &AdventureState,
    rules: &(Ruleset, AdventureRuleset),
    store: &[Arc<HeroDetails>],
) -> crate::reach::Reach {
    state
        .reach(&rules.0, &rules.1, store, HeroId::new(1))
        .unwrap()
}

// covers: ADV-MOV-009, ADV-MOV-010, ADV-OBJ-004
#[test]
fn a_flight_leaves_a_closed_top_in_the_air_only() {
    // §7 (the original's search, §3.1 point 3): a step up off the visitable tile of an object not
    // visited from above (a chart) is a flight's pass only, landing nowhere, and never a water
    // walk's; a step down onto it never, flying or not; rock never.
    let rules = hero_rules(true);
    let rows = ["g#g", "gCg", "ggg"];
    let mut state = flight_world(&rules, &rows, p(0, 1), vec![]);
    // Over the chart (passed, not entered), up off it: a pass.
    assert_eq!(
        fly(&mut state, &rules, &flyer(), &[p(1, 1), p(1, 0)]),
        Err(AdventureCommandError::Blocked {
            step: 1,
            error: StandError::Terrain
        })
    );
    assert_eq!(
        fly(&mut state, &rules, &flyer(), &[p(1, 1), p(2, 0)]),
        Err(AdventureCommandError::NoStop { step: 1 })
    );
    fly(&mut state, &rules, &flyer(), &[p(1, 1), p(2, 0), p(2, 1)]).unwrap();
    assert_eq!(at(&state).0, p(2, 1));
    // From above onto it: never.
    let mut state = flight_world(&rules, &rows, p(0, 0), vec![]);
    assert_eq!(
        fly(&mut state, &rules, &flyer(), &[p(1, 1)]),
        Err(AdventureCommandError::FromAbove { step: 0 })
    );
    // Standing on it after its visit: a water walk may not step up, a flight passes.
    let mut state = flight_world(&rules, &rows, p(0, 1), vec![]);
    fly(&mut state, &rules, &water_walker(), &[p(1, 1)]).unwrap();
    assert_eq!(
        fly(&mut state, &rules, &water_walker(), &[p(0, 0)]),
        Err(AdventureCommandError::FromAbove { step: 0 })
    );
    assert_eq!(
        fly(&mut state, &rules, &flyer(), &[p(0, 0)]),
        Err(AdventureCommandError::NoStop { step: 0 })
    );
    fly(&mut state, &rules, &flyer(), &[p(0, 0), p(0, 1)]).unwrap();
    assert_eq!(at(&state).0, p(0, 1));
    let reach = state_reach(&state, &rules, &flyer());
    assert_eq!(reach.points_left(p(1, 0)), None);
}

// covers: ADV-MOV-009, ADV-MOV-010, ADV-MOV-008
#[test]
fn the_query_reports_where_a_flight_or_a_water_walk_may_stop() {
    // E08 T41: the destinations are the stop nodes, their paths over pass nodes, as `MoveHero`
    // takes them (the property test `the_query_and_move_hero_agree` covers the rest).
    let rules = hero_rules(true);
    let state = flight_world(&rules, &["gBB~~gg"], p(0, 0), vec![]);
    let reach = state_reach(&state, &rules, &flyer());
    let tiles: Vec<(Pos, u32)> = reach.tiles().collect();
    assert_eq!(tiles, vec![(p(5, 0), 800)]);
    assert_eq!(reach.path(p(5, 0)), Some(row(5)));
    assert_eq!(reach.path(p(3, 0)), None);
    assert_eq!(reach.points_left(p(0, 0)), Some(1300));
    let state = flight_world(&rules, &["s~~~s"], p(0, 0), vec![]);
    let reach = state_reach(&state, &rules, &water_walker());
    assert_eq!(reach.tiles().collect::<Vec<_>>(), vec![(p(4, 0), 850)]);
    assert_eq!(reach.path(p(4, 0)), Some(row(4)));
    // A walker's query from the same tile: none.
    assert_eq!(state_reach(&state, &rules, &[]).tiles().count(), 0);
}
