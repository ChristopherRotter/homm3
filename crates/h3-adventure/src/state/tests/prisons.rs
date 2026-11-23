//! Tests of the Prison and the Market of Time (E08 T141a; increment 15's design note §1, §2, §8,
//! §9; survey `E08-spells.md` §2, §3): a Prison holding its prisoner from the build, its visit from
//! a neighbour naming the prisoner, the freeing and the emptying (ADV-PRI-001, -002); an object
//! entered and never visited (ADV-OBJ-008); and each player's list of heroes (GAM-HER-003, decision
//! 413).

use std::sync::Arc;
use std::vec;
use std::vec::Vec;

use h3_content::Formation;
use h3_contracts::{ArmyStack, HeroId, PlayerId};
use h3_heroes::HeroDetails;

use super::draws::{object, rules_with};
use super::rewards::{AT, HERO, RED, Rules};
use super::*;
use crate::command::{AdventureCommandError, PrisonVisit};
use crate::ruleset::{ObjectKindDef, ObjectTypeDef, Visit};
use crate::setup::{Holds, PrisonerSetup};
use crate::testing::{PEASANT, hero_at, hero_details, key, started, stream, world};

const BLUE: PlayerId = PlayerId::new(1);
/// The prisoner of these tests.
const PRISONER: HeroId = HeroId::new(3);

/// The draws' rules with a Prison `t:prison` (visited from a neighbour) and the Market of Time
/// `t:market_of_time` (entered), as the base pack's.
fn rules() -> Rules {
    rules_with(
        true,
        |_| {},
        |types| {
            let ty = |k: &str, visit, kind| ObjectTypeDef {
                key: key(k),
                name: "Object".into(),
                visit,
                from_above: false,
                kind_mark: false,
                kind,
            };
            types.push(ty("t:prison", Visit::Neighbour, ObjectKindDef::Prison));
            types.push(ty("t:market_of_time", Visit::Enter, ObjectKindDef::Empty));
        },
    )
}

/// The prisoner's army: two stacks, given out of slot order.
fn prisoner_army() -> Vec<ArmyStack> {
    vec![
        ArmyStack {
            slot: 4,
            creature: PEASANT,
            count: 7,
        },
        ArmyStack {
            slot: 1,
            creature: PEASANT,
            count: 2,
        },
    ]
}

/// A 16 × 2 grass map with these objects (on the top row), red's hero 1 at (1, 1), blue's hero 2
/// at (14, 1), and `prisoners`.
fn setup_of(objects: Vec<ObjectSetup>, prisoners: Vec<PrisonerSetup>) -> AdventureSetup {
    let mut setup = world(
        &["gggggggggggggggg", "gggggggggggggggg"],
        1,
        vec![
            hero_at(1, 0, Pos::new(1, 1, 0), &[PEASANT]),
            hero_at(2, 1, Pos::new(14, 1, 0), &[PEASANT]),
        ],
    );
    setup.map.objects.extend(objects);
    setup.prisoners = prisoners;
    setup
}

/// [`setup_of`] with a Prison on [`AT`] holding [`PRISONER`] with [`prisoner_army`], tight.
fn prison_setup(rules: &Rules) -> AdventureSetup {
    let prisoner = PrisonerSetup {
        id: PRISONER,
        army: prisoner_army(),
        formation: Formation::Tight,
    };
    setup_of(
        vec![object(rules, 2, "t:prison", Holds::Nothing)],
        vec![prisoner],
    )
}

/// The store: three heroes of no skill.
fn heroes() -> Vec<Arc<HeroDetails>> {
    (0..4).map(|_| Arc::new(hero_details(&[]))).collect()
}

/// Red's hero steps toward [`AT`]: the outcome and the events.
fn step(
    rules: &Rules,
    state: &mut AdventureState,
    heroes: &[Arc<HeroDetails>],
) -> (AdventureOutcome, Vec<AdventureEvent>) {
    let mut events = Vec::new();
    let command = AdventureCommand::MoveHero {
        hero: HERO,
        path: vec![AT],
    };
    let outcome = state
        .apply(
            &rules.0,
            &rules.1,
            heroes,
            (RED, Weekday::FIRST, &mut stream()),
            &command,
            &mut events,
        )
        .unwrap();
    (outcome, events)
}

// covers: ADV-PRI-001
#[test]
fn a_prison_holds_its_prisoner_from_the_build_by_slot() {
    let rules = rules();
    let state = started(&rules, &prison_setup(&rules)).unwrap();
    let prison = &state.map().objects()[0];
    assert_eq!(
        prison.resolved,
        Resolved::Prisoner {
            hero: PRISONER,
            army: vec![prisoner_army()[1], prisoner_army()[0]],
            formation: Formation::Tight,
        }
    );
    // Off the map, in no list.
    assert!(state.hero(PRISONER).is_none());
    assert_eq!(state.hero_list(RED), [HERO]);
    // The prisoner is part of the static map's hash.
    let mut other = prison_setup(&rules);
    other.prisoners[0].army[0].count = 8;
    let other = started(&rules, &other).unwrap();
    assert_ne!(other.map().content_hash(), state.map().content_hash());
    assert_ne!(other.state_hash(), state.state_hash());
}

// covers: ADV-PRI-001
#[test]
fn a_setup_gives_one_valid_prisoner_per_prison_with_an_id_of_its_own() {
    let rules = rules();
    let built = |setup: &AdventureSetup| started(&rules, setup).map(|_| ());
    // A Prison without a prisoner, a prisoner without a Prison.
    let mut setup = prison_setup(&rules);
    setup.prisoners.clear();
    assert_eq!(built(&setup), Err(SetupError::Prisoners(0)));
    let mut setup = prison_setup(&rules);
    setup.map.objects.clear();
    assert_eq!(built(&setup), Err(SetupError::Prisoners(1)));
    // An invalid army, as a hero's.
    let mut setup = prison_setup(&rules);
    setup.prisoners[0].army.clear();
    assert!(matches!(
        built(&setup),
        Err(SetupError::Army { hero: PRISONER, .. })
    ));
    // An id a hero has, or an earlier prisoner.
    let mut setup = prison_setup(&rules);
    setup.prisoners[0].id = HERO;
    assert_eq!(built(&setup), Err(SetupError::DuplicateHero(HERO)));
    let mut setup = prison_setup(&rules);
    setup
        .map
        .objects
        .push(object(&rules, 5, "t:prison", Holds::Nothing));
    setup.prisoners.push(setup.prisoners[0].clone());
    assert_eq!(built(&setup), Err(SetupError::DuplicateHero(PRISONER)));
    // Two Prisons hold two prisoners in object order.
    setup.prisoners[1].id = HeroId::new(4);
    let state = started(&rules, &setup).unwrap();
    let held: Vec<_> = state
        .map()
        .objects()
        .iter()
        .map(|o| match o.resolved {
            Resolved::Prisoner { hero, .. } => Some(hero),
            _ => None,
        })
        .collect();
    assert_eq!(held, [Some(PRISONER), Some(HeroId::new(4))]);
    assert_eq!(
        SetupError::Prisoners(2).to_string(),
        "2 prisoners do not give one per prison"
    );
}

// covers: ADV-PRI-002
#[test]
fn a_prisons_visit_from_a_neighbour_names_its_prisoner_and_changes_nothing() {
    let rules = rules();
    let heroes = heroes();
    let mut state = started(&rules, &prison_setup(&rules)).unwrap();
    let before = state.clone();
    let (outcome, _) = step(&rules, &mut state, &heroes);
    let visit = PrisonVisit {
        hero: HERO,
        index: 0,
        entrance: AT,
        prisoner: PRISONER,
    };
    assert_eq!(
        outcome,
        AdventureOutcome {
            prison: Some(visit),
            ..AdventureOutcome::default()
        }
    );
    // The visitor stays beside it, the step's price paid as at every visit from a neighbour
    // (batch E15's E15-1); the Prison stands until the game decides, nobody else changed.
    let visitor = state.hero(HERO).unwrap();
    let was = before.hero(HERO).unwrap();
    assert_eq!(visitor.pos, was.pos);
    assert!(visitor.movement < was.movement);
    assert!(!state.is_removed(0));
    assert_eq!(state.heroes()[1..], before.heroes()[1..]);
    assert!(state.hero(PRISONER).is_none());
}

// covers: ADV-PRI-002
#[test]
fn a_freed_prisoner_stands_on_the_entrance_last_in_its_owners_list_with_a_full_day() {
    let rules = rules();
    let heroes = heroes();
    let mut state = started(&rules, &prison_setup(&rules)).unwrap();
    let (outcome, _) = step(&rules, &mut state, &heroes);
    let visit = outcome.prison.unwrap();
    let visitor = state.hero(HERO).unwrap().clone();
    let mut events = Vec::new();
    let seen_by = state.free_prisoner((&rules.0, &rules.1, &heroes), &visit, RED, &mut events);
    assert_eq!(seen_by, state.viewers(&rules.0, RED, AT));
    assert!(seen_by.contains(RED));
    // The Prison's removal, told by its type and anchor.
    let prison = type_of_key(&rules, "t:prison");
    assert!(matches!(
        &events[..],
        [AdventureEvent {
            kind: AdventureEventKind::ObjectRemoved { object_type, anchor },
            ..
        }] if *object_type == prison && *anchor == AT
    ));
    assert!(state.is_removed(0));
    // The hero: on the entrance, the build's army and formation, tactics on, no boat, a full
    // day's land points for its army.
    let points = crate::day_points(&rules.0, &rules.1, Some(&heroes[3]), &prisoner_army(), 0);
    let freed = state.hero(PRISONER).unwrap();
    assert_eq!(
        freed,
        &HeroOnMap {
            id: PRISONER,
            owner: RED,
            pos: AT,
            army: vec![prisoner_army()[1], prisoner_army()[0]],
            formation: Formation::Tight,
            tactics: true,
            movement: points,
            boat: None,
            max_movement: points,
            listed: 1,
        }
    );
    assert!(points > 0);
    assert_eq!(state.hero_list(RED), [HERO, PRISONER]);
    assert_eq!(state.listed_hero(RED, 1), Some(PRISONER));
    // The visitor untouched; a second freeing of the same visit changes nothing.
    assert_eq!(state.hero(HERO), Some(&visitor));
    let after = state.clone();
    let mut events = Vec::new();
    let again = state.free_prisoner((&rules.0, &rules.1, &heroes), &visit, BLUE, &mut events);
    assert_eq!((again, events.len()), (Viewers::NONE, 0));
    assert_eq!(state, after);
    // A visit naming another prisoner changes nothing either.
    let mut fresh = started(&rules, &prison_setup(&rules)).unwrap();
    let before = fresh.clone();
    let wrong = PrisonVisit {
        prisoner: HeroId::new(2),
        ..visit
    };
    let none = fresh.free_prisoner((&rules.0, &rules.1, &heroes), &wrong, RED, &mut events);
    assert_eq!((none, events.len()), (Viewers::NONE, 0));
    assert_eq!(fresh, before);
}

// covers: ADV-PRI-002
#[test]
fn an_emptied_prison_leaves_the_map_once() {
    let rules = rules();
    let heroes = heroes();
    let mut state = started(&rules, &prison_setup(&rules)).unwrap();
    let (outcome, _) = step(&rules, &mut state, &heroes);
    let visit = outcome.prison.unwrap();
    let mut events = Vec::new();
    state.empty_prison(&rules.0, &visit, &mut events);
    assert_eq!(events.len(), 1);
    assert!(state.is_removed(0));
    assert!(state.hero(PRISONER).is_none());
    let after = state.clone();
    state.empty_prison(&rules.0, &visit, &mut events);
    assert_eq!((events.len(), &state), (1, &after));
    // A visit naming an index past the objects removes nothing.
    let past = PrisonVisit { index: 9, ..visit };
    state.empty_prison(&rules.0, &past, &mut events);
    assert_eq!((events.len(), &state), (1, &after));
}

// covers: ADV-OBJ-008
#[test]
fn the_market_of_time_is_entered_and_nothing_happens() {
    let rules = rules();
    let heroes = heroes();
    let market = object(&rules, 2, "t:market_of_time", Holds::Nothing);
    let mut state = started(&rules, &setup_of(vec![market], vec![])).unwrap();
    let (outcome, events) = step(&rules, &mut state, &heroes);
    assert_eq!(outcome, AdventureOutcome::default());
    // The step onto the entrance at its price, then nothing: no visit's event.
    assert_eq!(state.hero(HERO).unwrap().pos, AT);
    assert!(
        events
            .iter()
            .all(|e| matches!(e.kind, AdventureEventKind::HeroStepped { .. })),
        "{events:?}"
    );
    // Nothing to revisit there.
    assert_eq!(state.revisitable(&rules.1, RED).count(), 0);
    let revisit = AdventureCommand::Revisit { hero: HERO };
    let refused = state.apply(
        &rules.0,
        &rules.1,
        &heroes,
        (RED, Weekday::FIRST, &mut stream()),
        &revisit,
        &mut Vec::new(),
    );
    assert_eq!(refused, Err(AdventureCommandError::NothingToRevisit));
}

// covers: GAM-HER-003
#[test]
fn a_players_list_holds_its_heroes_on_the_map_in_the_order_they_joined_it() {
    let rules = rules();
    // Red's heroes in the setup's order 5, 1, 4 (not by id); blue's 2.
    let mut setup = setup_of(vec![], vec![]);
    setup
        .heroes
        .insert(0, hero_at(5, 0, Pos::new(5, 1, 0), &[PEASANT]));
    setup
        .heroes
        .push(hero_at(4, 0, Pos::new(7, 1, 0), &[PEASANT]));
    let mut state = started(&rules, &setup).unwrap();
    let ids = |list: &[u16]| list.iter().map(|&id| HeroId::new(id)).collect::<Vec<_>>();
    assert_eq!(state.hero_list(RED), ids(&[5, 1, 4]));
    assert_eq!(state.hero_list(BLUE), ids(&[2]));
    assert_eq!(state.hero_list(PlayerId::new(7)), []);
    let slots: Vec<_> = (0..4).map(|n| state.listed_hero(RED, n)).collect();
    assert_eq!(
        slots,
        [Some(HeroId::new(5)), Some(HERO), Some(HeroId::new(4)), None]
    );
    assert_eq!(state.listed_hero(RED, usize::MAX), None);
    // The heroes stay by id; each place from 0, per owner.
    let places: Vec<_> = state
        .heroes()
        .iter()
        .map(|h| (h.id.raw(), h.listed))
        .collect();
    assert_eq!(places, [(1, 1), (2, 0), (4, 2), (5, 0)]);
    // A hero that leaves the map leaves the list, the later ones moving up.
    state.remove_hero(&rules.0, (RED, HeroId::new(5)), &mut Vec::new());
    assert_eq!(state.hero_list(RED), ids(&[1, 4]));
    let places: Vec<_> = state
        .heroes()
        .iter()
        .map(|h| (h.id.raw(), h.listed))
        .collect();
    assert_eq!(places, [(1, 0), (2, 0), (4, 1)]);
}

// covers: GAM-HER-003
#[test]
fn the_order_of_one_owners_heroes_is_state_and_of_two_owners_not() {
    let rules = rules();
    let mut setup = setup_of(vec![], vec![]);
    let base = started(&rules, &setup).unwrap().state_hash();
    // Two owners' heroes in either order: the same lists, the same hash.
    setup.heroes.reverse();
    assert_eq!(started(&rules, &setup).unwrap().state_hash(), base);
    // One owner's two heroes in either order: two lists.
    setup.heroes[0].owner = RED;
    let one = started(&rules, &setup).unwrap();
    setup.heroes.reverse();
    let other = started(&rules, &setup).unwrap();
    assert_ne!(one.state_hash(), other.state_hash());
    assert_ne!(one.hero_list(RED), other.hero_list(RED));
}

/// The object type of key `k`.
fn type_of_key(rules: &Rules, k: &str) -> ObjectTypeId {
    rules.1.object_type_by_key(k).unwrap()
}
