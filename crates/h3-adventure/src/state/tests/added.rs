//! Tests of the objects added in play and the monsters' fractions (E08 T83b2a; increment 9's
//! design note §6): an added monster read as a placed one (its tile, guard, visit, battle, reach
//! and view, walking and flying), its removal, the occupancy's growth past a row of objects, the
//! forks, the invariants and the hash.

use std::format;
use std::string::String;

use h3_kernel::hash::hash_of;

use super::*;
use crate::command::{AdventureCommand, AdventureEvent};
use crate::testing::{
    FLYING, GUARD, PEASANT, army, hero_at, hero_details, hero_rules, started, world,
};

type Rules = (Ruleset, AdventureRuleset);

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const HERO: HeroId = HeroId::new(1);

fn p(x: u16, y: u16) -> Pos {
    Pos::new(x, y, 0)
}

/// A guard at (1, 1), index 0; with `placed` five pikemen at (4, 2), index 1, else grass there;
/// with `boat` an empty boat at (6, 0) beside water at (5, 0) (a walker's search then looks at
/// the water tiles too).
fn rows(placed: bool, boat: bool) -> [&'static str; 3] {
    [
        if boat { "ggggg~b" } else { "ggggggg" },
        "gKggggg",
        if placed { "ggggMgg" } else { "ggggggg" },
    ]
}

/// The started state of [`rows`] under the faithful [`hero_rules`], red's hero 1 (peasants) on
/// (0, 1).
fn twin(rules: &Rules, placed: bool, boat: bool) -> AdventureState {
    let heroes = vec![hero_at(1, 0, p(0, 1), &[PEASANT])];
    started(rules, &world(&rows(placed, boat), 1, heroes)).unwrap()
}

/// The placed twin and the other with the placed monster added in play: the same objects in the
/// same order, the monster at index 1 in both.
fn twins(rules: &Rules) -> (AdventureState, AdventureState) {
    twins_with(rules, false)
}

/// [`twins`] with or without the boat.
fn twins_with(rules: &Rules, boat: bool) -> (AdventureState, AdventureState) {
    let placed = twin(rules, true, boat);
    let mut added = twin(rules, false, boat);
    let monster = placed.map().objects()[1].clone();
    assert_eq!(added.add_object(monster), 1);
    (placed, added)
}

/// The store of heroes' details: hero 1 with `skills`.
fn store(skills: &[(h3_content::SkillId, u8)]) -> Vec<Arc<HeroDetails>> {
    vec![Arc::new(hero_details(&[])), Arc::new(hero_details(skills))]
}

/// Every tile of the 7 × 3 map.
fn tiles() -> impl Iterator<Item = Pos> {
    (0..3).flat_map(|y| (0..7).map(move |x| p(x, y)))
}

/// What a state says of every tile: its guard, whether a hero may stand there, the tile as it
/// stands, the object a visit there concerns.
fn read(rules: &Rules, state: &AdventureState) -> Vec<String> {
    tiles()
        .map(|pos| {
            format!(
                "{:?} {:?} {:?} {:?}",
                state.guard_of(&rules.1, pos),
                state.check_stand(&rules.1, pos),
                state.live(pos, state.map().at(pos)),
                state
                    .tile_object(&rules.1, pos)
                    .map(|(index, id, _)| (index, id)),
            )
        })
        .collect()
}

/// The reachable tiles of red's hero 1 with `heroes`, with their paths.
fn reach(rules: &Rules, state: &AdventureState, heroes: &[Arc<HeroDetails>]) -> Vec<String> {
    let reach = state.reach(&rules.0, &rules.1, heroes, HERO).unwrap();
    reach
        .tiles()
        .map(|(pos, left)| format!("{pos:?} {left} {:?}", reach.path(pos)))
        .collect()
}

/// Moves red's hero 1 along `path` with `heroes`: the outcome and the events.
fn walk(
    rules: &Rules,
    state: &mut AdventureState,
    heroes: &[Arc<HeroDetails>],
    path: &[Pos],
) -> (AdventureOutcome, Vec<AdventureEvent>) {
    let mut events = Vec::new();
    let command = AdventureCommand::MoveHero {
        hero: HERO,
        path: path.to_vec(),
    };
    let outcome = state
        .apply(&rules.0, &rules.1, heroes, RED, &command, &mut events)
        .unwrap();
    (outcome, events)
}

// covers: ADV-MON-005, ADV-MON-006, ADV-OBJ-003, ADV-FOG-006
#[test]
fn an_added_monster_reads_as_a_placed_one() {
    // Design §6: the live presence's added objects are read where the static map's are, so a
    // monster added in play guards, blocks, is visited, is reached and is seen as the same
    // monster placed on the map; walking and flying.
    let rules = hero_rules(true);
    let (placed, added) = twins(&rules);
    assert_eq!(added.added_objects().len(), 1);
    assert_eq!(added.object(1), placed.map().objects().get(1));
    assert_eq!(added.object(2), None);
    assert_eq!(added.monster_count(1), Some(5));
    assert_eq!(added.monster_count(2), None);
    assert_eq!(read(&rules, &added), read(&rules, &placed));
    assert_eq!(added.guard_of(&rules.1, p(3, 1)), Some(1));
    for heroes in [Vec::new(), store(&[(FLYING, 3)])] {
        assert_eq!(
            reach(&rules, &added, &heroes),
            reach(&rules, &placed, &heroes)
        );
    }
    let (afloat, beside) = twins_with(&rules, true);
    assert_eq!(read(&rules, &beside), read(&rules, &afloat));
    assert_eq!(reach(&rules, &beside, &[]), reach(&rules, &afloat, &[]));
    assert!(!reach(&rules, &afloat, &[]).is_empty());
    for viewer in [RED, BLUE] {
        assert_eq!(
            added.observe(&rules.0, &rules.1, viewer),
            placed.observe(&rules.0, &rules.1, viewer)
        );
    }
    let seen = added.observe(&rules.0, &rules.1, RED);
    assert!(
        seen.objects
            .iter()
            .any(|o| o.monster.is_some() && o.anchor == p(4, 2))
    );
    assert!(added.check_added(&rules.1));
    assert!(placed.check_added(&rules.1));
}

// covers: ADV-MON-006, ADV-MON-009, ADV-MON-010
#[test]
fn an_added_monster_guards_fights_and_leaves_as_a_placed_one() {
    // A walk onto a tile the added monster guards starts its battle; its survivors stay as its
    // count; a battle that leaves none takes it off the map, its tile freed, as the placed one's.
    let rules = hero_rules(true);
    let path = [p(1, 2), p(2, 2), p(3, 2)];
    for survivors in [vec![], army(&[PEASANT])] {
        let (mut placed, mut added) = twins(&rules);
        let heroes = store(&[]);
        let moved = walk(&rules, &mut placed, &heroes, &path);
        assert_ne!(moved.0, AdventureOutcome::default());
        assert_eq!(walk(&rules, &mut added, &heroes, &path), moved);
        let ends = [&mut placed, &mut added].map(|state| {
            let mut events = Vec::new();
            let outcome = state.end_monster_battle(
                (&rules.0, &rules.1, &heroes),
                (HERO, p(4, 2)),
                (&survivors, true),
                &mut events,
            );
            (outcome, events)
        });
        assert_eq!(ends[0], ends[1]);
        assert!(!ends[0].1.is_empty());
        assert_eq!(added.is_removed(1), placed.is_removed(1));
        assert_eq!(added.monster_count(1), placed.monster_count(1));
        assert_eq!(read(&rules, &added), read(&rules, &placed));
        assert_eq!(
            added.observe(&rules.0, &rules.1, RED),
            placed.observe(&rules.0, &rules.1, RED)
        );
        assert!(added.check_added(&rules.1));
    }
}

/// The state's first object, a monster as [`world`] places them, moved to `x` of row 0.
fn pikemen_at(state: &AdventureState, x: u16) -> MapObject {
    moved(state, p(x, 0))
}

/// The state's first object moved to `pos`.
fn moved(state: &AdventureState, pos: Pos) -> MapObject {
    let mut monster = state.map().objects()[0].clone();
    monster.anchor = pos;
    monster
}

#[test]
fn removals_hold_as_the_added_objects_grow_past_a_row() {
    // The removed objects' grid grows a row as the added indices need it (a row is
    // `Occupancy::ROW` objects), keeping every removal; a removal frees an added object's tile.
    let rules = hero_rules(true);
    let rows = ["Mggggggg", "gggggggg", "gggggggg", "gggggggg", "gggggggg"];
    let mut state = started(&rules, &world(&rows, 1, vec![])).unwrap();
    // Object `i` on tile `i` in row order: the static monster 0, then the 39 added ones.
    let tile = |index: u16| p(index % 8, index / 8);
    state.remove_object(0);
    assert!(state.added_objects().is_empty());
    for index in 1..40 {
        let monster = moved(&state, tile(index));
        assert_eq!(state.add_object(monster), usize::from(index));
    }
    assert!(state.added_objects().len() > Occupancy::ROW);
    for index in [5, 35] {
        state.remove_object(index);
    }
    state.remove_object(40);
    let removed: Vec<usize> = (0..40).filter(|&i| state.is_removed(i)).collect();
    assert_eq!(removed, [0, 5, 35]);
    for (index, free) in [(0, true), (5, true), (35, true), (6, false), (39, false)] {
        let pos = tile(index);
        let live = state.live(pos, state.map().at(pos));
        assert_eq!(!live.blocked && !live.visitable, free, "{index}");
    }
    assert_eq!(state.tile_object(&rules.1, tile(35)), None);
    assert!(state.tile_object(&rules.1, tile(34)).is_some());
    assert!(state.check_added(&rules.1));
    // A fork shares the added objects; an addition to one leaves the other's.
    let fork = state.clone();
    let monster = moved(&state, tile(35));
    state.add_object(monster);
    assert_eq!(fork.added_objects().len(), 39);
    assert_eq!(state.added_objects().len(), 40);
    assert!(state.check_added(&rules.1));
}

#[test]
fn the_added_objects_invariants_refuse_a_misplaced_or_untyped_one() {
    // An added object still on the map is a typed monster covering land tiles of the map that no
    // other object still on the map covers; one that left is not checked.
    let rules = hero_rules(true);
    let fresh = || started(&rules, &world(&["MK~g"], 1, vec![])).unwrap();
    let base = fresh();
    let at = |x| pikemen_at(&base, x);
    let untyped = MapObject {
        object_type: None,
        ..at(3)
    };
    let guard = MapObject {
        object_type: Some(GUARD),
        ..at(3)
    };
    let no_data = MapObject {
        monster: None,
        ..at(3)
    };
    let nowhere = MapObject {
        blocked: TileMask::default(),
        visitable: TileMask::default(),
        ..at(3)
    };
    let cases = [
        (vec![at(3)], true),
        (vec![at(2)], false),
        (vec![at(1)], false),
        (vec![at(0)], false),
        (vec![at(3), at(3)], false),
        (vec![untyped], false),
        (vec![guard], false),
        (vec![no_data], false),
        (vec![nowhere], false),
    ];
    for (objects, fits) in cases {
        let mut state = fresh();
        let indices: Vec<usize> = objects.into_iter().map(|o| state.add_object(o)).collect();
        assert_eq!(state.check_added(&rules.1), fits, "{indices:?}");
        state.remove_object(*indices.last().unwrap());
        assert!(state.check_added(&rules.1), "{indices:?}");
    }
    // A tile a removed object freed takes one.
    let mut state = fresh();
    state.remove_object(1);
    state.add_object(at(1));
    assert!(state.check_added(&rules.1));
}

#[test]
fn a_fraction_lies_below_the_unit_on_a_monster_on_the_map() {
    // Design §6: a monster's fraction of a creature, in the growth unit's parts, none for 0; on
    // a wandering monster still on the map only (it leaves with the monster).
    let rules = hero_rules(true);
    let unit = rules.1.params().monsters().unwrap().growth_unit();
    let fresh = || started(&rules, &world(&["MKg"], 1, vec![])).unwrap();
    let mut state = fresh();
    assert_eq!(state.fraction(0), 0);
    let below = u8::try_from(unit - 1).unwrap();
    state.set_fraction(0, below);
    assert_eq!(state.fraction(0), below);
    assert!(state.check_added(&rules.1));
    state.set_fraction(0, below + 1);
    assert!(!state.check_added(&rules.1));
    state.set_fraction(0, 0);
    assert_eq!(state.fractions.len(), 0);
    assert!(state.check_added(&rules.1));
    let mut guard = fresh();
    guard.set_fraction(1, 1);
    assert!(!guard.check_added(&rules.1));
    let mut past = fresh();
    past.set_fraction(3, 1);
    assert!(!past.check_added(&rules.1));
    let mut zero = fresh();
    zero.fractions.set(0, Some(0));
    assert!(!zero.check_added(&rules.1));
    let mut gone = fresh();
    gone.set_fraction(0, 1);
    gone.remove_object(0);
    assert_eq!(gone.fraction(0), 0);
    assert!(gone.check_added(&rules.1));
    let mut left = fresh();
    left.remove_object(0);
    left.fractions.set(0, Some(1));
    assert!(!left.check_added(&rules.1));
    // An added monster's fraction.
    let mut added = fresh();
    let monster = pikemen_at(&added, 2);
    let index = added.add_object(monster);
    added.set_fraction(index, 1);
    assert!(added.check_added(&rules.1));
}

#[test]
fn the_hash_covers_the_added_objects_and_the_fractions() {
    // Adventure state hash v19: the added objects in the occupancy's encoding, then the fractions
    // by object index.
    let rules = hero_rules(true);
    let fresh = || started(&rules, &world(&["Mgg"], 1, vec![])).unwrap();
    let plain = hash_of(&fresh());
    let monster = pikemen_at(&fresh(), 2);
    let with = |object: MapObject| {
        let mut state = fresh();
        state.add_object(object);
        hash_of(&state)
    };
    assert_ne!(with(monster.clone()), plain);
    assert_eq!(with(monster.clone()), with(monster.clone()));
    let mut more = monster.clone();
    more.monster.as_mut().unwrap().count = 6;
    assert_ne!(with(more), with(monster));
    let mut state = fresh();
    state.set_fraction(0, 3);
    let three = hash_of(&state);
    assert_ne!(three, plain);
    state.set_fraction(0, 4);
    assert_ne!(hash_of(&state), three);
    state.set_fraction(0, 0);
    assert_eq!(hash_of(&state), plain);
}
