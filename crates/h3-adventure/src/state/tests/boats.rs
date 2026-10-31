//! Tests of the boats (E08 T72, increment 8's design note §6; ADR-0016 point 1): the setup's boats,
//! their checks and invariants, a hero's boat and its day's maximum by its surface, the hash; the
//! static map's marks of the sea's grounds and the holes (increment 8's design note §5).

use std::sync::Arc;

use h3_heroes::HeroDetails;

use super::*;
use crate::setup::{BoatSetup, ObjectSetup, Pos, TileMask};
use crate::state::boats::SETUP_FACING;
use h3_content::{HeroEffectDef, SecondarySkillDef, SkillId};

use crate::testing::{
    BARRENS, HALLOW, HOLE, PIKEMAN, WINDS, hero_at, hero_details, hero_rules, hero_skills, key,
    movement_rules_with, world,
};

type Rules = (Ruleset, AdventureRuleset);

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);

fn p(x: u16, y: u16) -> Pos {
    Pos::new(x, y, 0)
}

/// A 16 × 16 map: grass but for the sea on columns 4 to 15, empty boats at (5, 2) and (5, 6);
/// red's hero 1 on the shore at (3, 2), blue's hero 2 at (1, 12).
fn rows() -> Vec<&'static str> {
    let mut rows = vec!["gggg~~~~~~~~~~~~"; 16];
    rows[2] = "gggg~b~~~~~~~~~~";
    rows[6] = "gggg~b~~~~~~~~~~";
    rows
}

fn heroes() -> Vec<HeroSetup> {
    vec![
        hero_at(1, 0, p(3, 2), &[PIKEMAN]),
        hero_at(2, 1, p(1, 12), &[PIKEMAN]),
    ]
}

fn build(rules: &Rules, setup: &AdventureSetup) -> Result<AdventureState, SetupError> {
    AdventureState::new(&rules.0, &rules.1, setup, &mut draws())
}

fn boat(tile: Pos, kind: u8) -> Boat {
    Boat {
        tile,
        kind,
        facing: SETUP_FACING,
        owner: None,
        captain: None,
        aboard: false,
        gone: false,
    }
}

// covers: ADV-SEA-001
#[test]
fn the_map_s_boats_lie_empty_on_water() {
    let rules = hero_rules(true);
    let mut setup = world(&rows(), 1, heroes());
    setup.map.boats[1].kind = 2;
    let state = build(&rules, &setup).unwrap();
    // Each empty, facing 2, without owner or captain, in the setup's order (ADV-SEA-001).
    let boats: Vec<(usize, Boat)> = state.boats().map(|(i, b)| (i, *b)).collect();
    assert_eq!(boats, [(0, boat(p(5, 2), 0)), (1, boat(p(5, 6), 2))]);
    assert_eq!(state.boat(1), Some(&boat(p(5, 6), 2)));
    assert_eq!(state.boat(2), None);
    assert!(state.is_empty_boat(p(5, 2)) && state.is_empty_boat(p(5, 6)));
    assert!(!state.is_empty_boat(p(5, 3)) && !state.is_empty_boat(Pos::new(5, 2, 1)));
    assert!(state.check_boats(&rules.1));
    // A map without boats has none.
    let plain = build(&rules, &world(&["gggg~~~~"; 8], 1, vec![])).unwrap();
    assert_eq!(plain.boats().count(), 0);
    assert_eq!(plain.boat(0), None);
    assert!(!plain.is_empty_boat(p(5, 2)));
    assert!(plain.check_boats(&rules.1));
}

/// A blocker on `tile`, blocked and visited there as `masks` say.
fn object_on(tile: Pos, (blocked, visitable): (u64, u64)) -> ObjectSetup {
    ObjectSetup {
        scenery: Default::default(),
        anchor: tile,
        blocked: TileMask::from_bits(blocked),
        visitable: TileMask::from_bits(visitable),
        object_type: None,
        amount: None,
        drawn: TileMask::default(),
        monster: None,
        guards: None,
        holds: Default::default(),
        owner: None,
    }
}

// covers: ADV-SEA-001
#[test]
fn the_setup_refuses_a_boat_where_no_boat_may_lie() {
    let rules = hero_rules(true);
    let with = |f: &dyn Fn(&mut AdventureSetup)| {
        let mut setup = world(&rows(), 1, heroes());
        f(&mut setup);
        build(&rules, &setup).map(|_| ())
    };
    let push = |tile| move |s: &mut AdventureSetup| s.map.boats.push(BoatSetup { tile, kind: 0 });
    // Off the map, off its levels, on land or rock, on an earlier boat's tile.
    for tile in [p(16, 2), Pos::new(5, 2, 1), p(2, 2), p(5, 6)] {
        assert_eq!(with(&push(tile)), Err(SetupError::Boat(2)), "{tile:?}");
    }
    let rock = with(&|s| {
        s.map.tiles[3 * 16 + 9].terrain = crate::testing::ROCK;
        s.map.boats.push(BoatSetup {
            tile: p(9, 3),
            kind: 0,
        });
    });
    assert_eq!(rock, Err(SetupError::Boat(2)));
    // On a water tile an object blocks, or one an object is visited from.
    for masks in [(1, 0), (0, 1)] {
        let blocked = with(&|s| {
            s.map.objects.push(object_on(p(9, 9), masks));
            s.map.boats.push(BoatSetup {
                tile: p(9, 9),
                kind: 0,
            });
        });
        assert_eq!(blocked, Err(SetupError::Boat(2)), "{masks:?}");
    }
    // Beside such a tile, anywhere on water, any number of boats: fine (decision 164).
    let beside = with(&|s| {
        s.map.objects.push(object_on(p(9, 9), (1, 1)));
        for x in 6..16 {
            s.map.boats.push(BoatSetup {
                tile: p(x, 10),
                kind: 1,
            });
        }
    });
    assert_eq!(beside, Ok(()));
    assert_eq!(
        SetupError::Boat(3).to_string(),
        "boat 3 lies where no boat may"
    );
    assert_eq!(
        SetupError::ObjectOwner(4).to_string(),
        "object 4 has an owner it may not have"
    );
}

/// A store of heroes by id from 0.
fn store(details: Vec<HeroDetails>) -> Vec<Arc<HeroDetails>> {
    details.into_iter().map(Arc::new).collect()
}

/// [`hero_rules`]' skills and Navigation at levels 1 to 3 (the sea table's entries 1 to 3).
fn navigation_rules() -> (Rules, SkillId) {
    let rules = movement_rules_with(
        true,
        16,
        |core| {
            core.skills = hero_skills();
            core.skills.push(SecondarySkillDef {
                key: key("t:navigation"),
                name: "Navigation".into(),
                levels: [1, 2, 3].map(|level| vec![HeroEffectDef::Navigation { level }]),
            });
        },
        |_| {},
    );
    let navigation = rules.0.skill_by_key("t:navigation").unwrap();
    (rules, navigation)
}

// covers: ADV-MOV-001, ADV-MOV-002, HER-ADV-009
#[test]
fn a_hero_in_a_boat_takes_the_sea_points_at_its_turn_start() {
    let (rules, navigation) = navigation_rules();
    let mut state = build(&rules, &world(&rows(), 1, heroes())).unwrap();
    // At the start of the game every hero is on land: the land's points, which are its maximum.
    state.start_game(&rules.0, &rules.1, &[]);
    let land = state.hero(HeroId::new(1)).unwrap().movement;
    assert!(land > 0);
    for hero in state.heroes() {
        assert_eq!(
            (hero.boat, hero.max_movement),
            (None, hero.movement),
            "{:?}",
            hero.id
        );
    }
    // Aboard the first boat (as T73's boarding will put it): the sea's table at the turn start,
    // its first entry without details, the Navigation level's with them (HER-ADV-009).
    state.put_aboard(HeroId::new(1), 0);
    assert!(state.check_boats(&rules.1));
    assert!(!state.is_empty_boat(p(5, 2)));
    let hero = state.hero(HeroId::new(1)).unwrap();
    assert_eq!((hero.pos, hero.boat), (p(5, 2), Some(0)));
    assert_eq!(
        state.boat(0),
        Some(&Boat {
            owner: Some(RED),
            captain: Some(HeroId::new(1)),
            aboard: true,
            ..boat(p(5, 2), 0)
        })
    );
    state.start_turn(&rules.0, &rules.1, &[], RED);
    let hero = state.hero(HeroId::new(1)).unwrap();
    assert_eq!((hero.movement, hero.max_movement), (1500, 1500));
    let navigator = store(vec![
        hero_details(&[]),
        hero_details(&[(navigation, 2)]),
        hero_details(&[]),
    ]);
    state.start_turn(&rules.0, &rules.1, &navigator, RED);
    let hero = state.hero(HeroId::new(1)).unwrap();
    assert_eq!((hero.movement, hero.max_movement), (3000, 3000));
    assert_eq!(crate::sea_day_points(&rules.0, &rules.1, None), 1500);
    // Blue's turn leaves red's hero as it is; blue's on land gets the land's.
    state.set_movement(HeroId::new(1), 7);
    state.start_turn(&rules.0, &rules.1, &navigator, BLUE);
    let (red, blue) = (
        state.hero(HeroId::new(1)).unwrap(),
        state.hero(HeroId::new(2)).unwrap(),
    );
    assert_eq!((red.movement, red.max_movement), (7, 3000));
    assert_eq!((blue.movement, blue.max_movement), (land, land));
}

// covers: ADV-SEA-001
#[test]
fn the_invariants_see_a_boat_out_of_step_with_its_hero() {
    let rules = hero_rules(true);
    let mut aboard = build(&rules, &world(&rows(), 1, heroes())).unwrap();
    aboard.put_aboard(HeroId::new(1), 0);
    assert!(aboard.check_boats(&rules.1));
    let broken: [&dyn Fn(&mut AdventureState); 7] = [
        // The hero no longer holds its boat, or holds an empty one, or one past the boats.
        &|s| s.heroes[0].boat = None,
        &|s| s.heroes[0].boat = Some(1),
        &|s| s.heroes[0].boat = Some(9),
        // Two heroes hold one boat.
        &|s| {
            s.heroes[1].pos = p(5, 3);
            s.heroes[1].boat = Some(0);
        },
        // A hero on land in a boat, a hero on water without one.
        &|s| s.heroes[0].pos = p(3, 2),
        &|s| s.heroes[1].pos = p(9, 9),
        // An empty boat's tile out of the grid.
        &|s| {
            let boats = s.boats.as_mut().unwrap();
            boats.records.set(1, Some(boat(p(9, 9), 0)));
        },
    ];
    for (i, f) in broken.iter().enumerate() {
        let mut state = aboard.clone();
        f(&mut state);
        assert!(!state.check_boats(&rules.1), "{i}");
    }
}

// covers: ADV-SEA-001
#[test]
fn the_hash_sees_the_boats_and_each_hero_s_boat_and_maximum() {
    let rules = hero_rules(true);
    let setup = world(&rows(), 1, heroes());
    let base = build(&rules, &setup).unwrap();
    let mut changed = Vec::new();
    let mut without = setup.clone();
    without.map.boats.clear();
    changed.push(build(&rules, &without).unwrap());
    for f in [
        |s: &mut AdventureSetup| s.map.boats[0].kind = 1,
        |s: &mut AdventureSetup| s.map.boats[1].tile = p(6, 6),
        |s: &mut AdventureSetup| s.map.boats.swap(0, 1),
    ] {
        let mut other = setup.clone();
        f(&mut other);
        changed.push(build(&rules, &other).unwrap());
    }
    let mut aboard = base.clone();
    aboard.put_aboard(HeroId::new(1), 0);
    changed.push(aboard);
    let mut maximum = base.clone();
    maximum.heroes[0].max_movement = 1;
    changed.push(maximum);
    for (i, state) in changed.iter().enumerate() {
        assert_ne!(state.state_hash(), base.state_hash(), "change {i}");
    }
    // The empty boats' grid follows from the records: equal states, equal hashes.
    assert_eq!(
        build(&rules, &setup).unwrap().state_hash(),
        base.state_hash()
    );
}

/// An object of `ty` lying on the 3 × 2 tiles left of and above `anchor` (its picture only).
fn lying(ty: ObjectTypeId, anchor: Pos) -> ObjectSetup {
    ObjectSetup {
        object_type: Some(ty),
        drawn: TileMask::from_bits(0x707),
        ..object_on(anchor, (0, 0))
    }
}

// covers: ADV-MOV-004
#[test]
fn the_static_map_marks_the_sea_s_grounds_and_the_holes() {
    // Winds on columns 6 to 8 of rows 1 and 2, a hallow over their right part (columns 8 to 10),
    // a barrens on columns 10 to 12 with winds over its left part (10 to 12 of rows 1, 2 from
    // anchor 12): on each tile the last ground in map order decides (0x4FCE20). Two holes on one
    // anchor (nothing names them) on the shore.
    let rules = hero_rules(true);
    let mut setup = world(&rows(), 1, heroes());
    setup.map.objects.extend([
        lying(WINDS, p(8, 2)),
        lying(HALLOW, p(10, 2)),
        lying(BARRENS, p(12, 2)),
        lying(WINDS, p(12, 2)),
        lying(HOLE, p(2, 9)),
        lying(HOLE, p(2, 9)),
    ]);
    let state = build(&rules, &setup).unwrap();
    let marked = |f: fn(&StaticTile) -> bool, y: u16| -> Vec<u16> {
        (0..16)
            .filter(|&x| f(state.map().tile(p(x, y)).unwrap()))
            .collect()
    };
    assert_eq!(marked(|t| t.winds, 1), [6, 7, 10, 11, 12]);
    assert_eq!(marked(|t| t.winds, 2), [6, 7, 10, 11, 12]);
    assert!(marked(|t| t.no_native, 2).is_empty());
    assert_eq!(marked(|t| t.hole, 8), [0, 1, 2]);
    assert_eq!(marked(|t| t.hole, 9), [0, 1, 2]);
    assert!(marked(|t| t.winds || t.hole, 3).is_empty());
    // The marks follow from the objects: the static map's hash sees the objects, not the marks.
    let plain = build(&rules, &world(&rows(), 1, heroes())).unwrap();
    assert!(
        plain
            .map()
            .tiles()
            .iter()
            .all(|t| !t.winds && !t.hole && !t.no_native)
    );
    // A tile off the map has neither.
    assert_eq!(
        (StaticTile::OUTSIDE.winds, StaticTile::OUTSIDE.hole),
        (false, false)
    );
}

// covers: ADV-SEA-001
#[test]
fn a_map_holds_more_boats_than_the_original_s_64() {
    // `ADV-QUIRK-32` (decision 164): the original's 65th boat is dropped at load; every one here.
    let rules = hero_rules(true);
    let mut setup = world(&["~~~~~~~~~~~~~~~~"; 16], 1, vec![]);
    setup.map.boats = (0..80)
        .map(|i| BoatSetup {
            tile: p(i % 16, i / 16),
            kind: 0,
        })
        .collect();
    let state = build(&rules, &setup).unwrap();
    assert_eq!(state.boats().count(), 80);
    assert!(state.is_empty_boat(p(15, 4)) && !state.is_empty_boat(p(0, 5)));
    assert!(state.check_boats(&rules.1));
}
