//! Tests of the sea's pickups (E08 T74a; design §11; ADV-PIK-014 to -018): Flotsam's, the Sea
//! Chest's and the Shipwreck Survivor's visits from a boat (their contents, the full backpack's
//! turn and loss under `ADV-QUIRK-22`), each leaving the map; the Ocean Bottle's message read by
//! its visitor's owner alone, then leaving; and that only a boat takes a pickup on water. Batch
//! E8's rows as sources: E8-6 and E8-8 (the visits, a full backpack's chest and survivor).

use std::sync::Arc;
use std::vec;
use std::vec::Vec;

use h3_heroes::HeroDetails;

use super::draws::object;
use super::rewards::{
    BLUE, GOLD, HERO, RED, RING, Rules, WOOD, add, draw, full, gives, hero, rules, store, type_of,
};
use super::*;
use crate::command::{AdventureCommand, AdventureCommandError, AdventureEvent, AdventureEventKind};
use crate::setup::{Holds, ObjectSetup, Pos};
use crate::state::Resolved;
use crate::testing::{FLYING, PEASANT, WATER_WALKING, coasted, hero_at, hero_details, started};

fn p(x: u16, y: u16) -> Pos {
    Pos::new(x, y, 0)
}

/// The pickup's tile, on water: beside the boat's (1, 1) and the shore's (2, 2).
const AT: Pos = Pos::new(2, 1, 0);

/// Red's hero 1 at (0, 1) on the coast and an empty boat at (1, 1); blue's hero 2 at (6, 2), who
/// sees the pickup's tile (its radius).
const ROWS: [&str; 3] = ["g~~~~~~g", "gb~~~~~g", "gggggggg"];

/// The started state of [`ROWS`] with one object of type `k` on [`AT`], red's hero 1 on `at`.
fn sea(rules: &Rules, k: &str, at: Pos) -> AdventureState {
    let mut setup = coasted(
        &ROWS,
        vec![
            hero_at(1, 0, at, &[PEASANT]),
            hero_at(2, 1, p(6, 2), &[PEASANT]),
        ],
    );
    setup.map.objects.push(ObjectSetup {
        anchor: AT,
        ..object(rules, 0, k, Holds::Nothing)
    });
    started(rules, &setup).unwrap()
}

/// [`sea`] with red's hero 1 in the boat at (1, 1), its points 1 500.
fn afloat(rules: &Rules, k: &str) -> AdventureState {
    let mut state = sea(rules, k, p(0, 1));
    state.put_aboard(HERO, 0);
    let hero = &mut state.heroes[0];
    (hero.movement, hero.max_movement) = (1500, 1500);
    state
}

/// Red's hero 1 moves along `path` with `heroes`: the outcome and the events, or the error (a
/// rejected move leaves the state and emits nothing).
fn step(
    rules: &Rules,
    state: &mut AdventureState,
    heroes: &[Arc<HeroDetails>],
    path: &[Pos],
) -> Result<(AdventureOutcome, Vec<AdventureEvent>), AdventureCommandError> {
    let before = state.clone();
    let mut events = Vec::new();
    let command = AdventureCommand::MoveHero {
        hero: HERO,
        path: path.to_vec(),
    };
    let result = state.apply(&rules.0, &rules.1, heroes, RED, &command, &mut events);
    if result.is_err() {
        assert!(events.is_empty(), "a rejected move emitted events");
        assert_eq!(*state, before, "a rejected move changed the state");
    }
    result.map(|outcome| (outcome, events))
}

/// The events but the steps.
fn told(events: &[AdventureEvent]) -> Vec<AdventureEvent> {
    events
        .iter()
        .filter(|e| !matches!(e.kind, AdventureEventKind::HeroStepped { .. }))
        .copied()
        .collect()
}

/// The pickup of type `object_type` leaving [`AT`], seen by both players.
fn removed(object_type: ObjectTypeId) -> AdventureEvent {
    AdventureEvent {
        kind: AdventureEventKind::ObjectRemoved {
            object_type,
            anchor: AT,
        },
        seen_by: Viewers::NONE.with(RED).with(BLUE),
    }
}

/// The boat visits the pickup of type `k` drawn as `resolved` with `contents`, its hero of
/// `details`: the outcome, after checking that the boat stayed on its tile with the step's price
/// paid, that the pickup left the map, told as it is seen, and that its visit wrote nothing in
/// the visits' block (nothing stays to learn or to empty).
fn visit(
    rules: &Rules,
    k: &str,
    (resolved, contents): (Resolved, &[(ResourceId, u32)]),
    details: HeroDetails,
) -> AdventureOutcome {
    let mut state = afloat(rules, k);
    draw(&mut state, 0, resolved, contents);
    let (outcome, events) = step(rules, &mut state, &store(details), &[AT]).unwrap();
    let hero = state.hero(HERO).unwrap();
    assert_eq!(
        (hero.pos, hero.boat, hero.movement),
        (p(1, 1), Some(0), 1400)
    );
    assert!(state.is_removed(0));
    assert_eq!(told(&events), vec![removed(type_of(rules, k))]);
    assert!(state.visits().is_none());
    outcome
}

// covers: ADV-PIK-014, ADV-PIK-015, ADV-PIK-016, ADV-SEA-005
#[test]
fn the_sea_s_finds_give_their_contents_from_a_boat_and_leave() {
    let strict = rules(false);
    let rules = rules(true);
    let gains = |gains: &[(ResourceId, u32)]| AdventureOutcome {
        gains: gains.to_vec(),
        ..AdventureOutcome::default()
    };
    // Flotsam: its resources (survey `E08-sea.md` §8.2; a drawn nothing gives nothing).
    let flotsam = [(WOOD, 5), (GOLD, 200)];
    assert_eq!(
        visit(&rules, "t:flotsam", (Resolved::Nothing, &flotsam), hero()),
        gains(&flotsam)
    );
    assert_eq!(
        visit(&rules, "t:flotsam", (Resolved::Nothing, &[]), hero()),
        AdventureOutcome::default()
    );
    // The Sea Chest (§8.3): the artifact with the 1 000 gold; with a full backpack 1 500 gold
    // instead of both (E8-8); its 1 500 gold alone.
    let chest = || (Resolved::Artifact(RING), &[(GOLD, 1000)][..]);
    assert_eq!(
        visit(&rules, "t:sea_chest", chest(), hero()),
        AdventureOutcome {
            gains: vec![(GOLD, 1000)],
            ..gives(&[add(RING)])
        }
    );
    assert_eq!(
        visit(&rules, "t:sea_chest", chest(), full(hero())),
        gains(&[(GOLD, 1500)])
    );
    let gold = (Resolved::Nothing, &[(GOLD, 1500)][..]);
    assert_eq!(
        visit(&rules, "t:sea_chest", gold, hero()),
        gains(&[(GOLD, 1500)])
    );
    // The Shipwreck Survivor (§8.4): its artifact; with a full backpack nothing, and it leaves
    // all the same (E8-8).
    let survivor = || (Resolved::Artifact(RING), &[][..]);
    assert_eq!(
        visit(&rules, "t:survivor", survivor(), hero()),
        gives(&[add(RING)])
    );
    assert_eq!(
        visit(&rules, "t:survivor", survivor(), full(hero())),
        AdventureOutcome::default()
    );
    // Without `ADV-QUIRK-22`'s flag the free finger takes the artifact of a full backpack.
    assert_eq!(
        visit(&strict, "t:survivor", survivor(), full(hero())),
        gives(&[add(RING)])
    );
}

// covers: ADV-PIK-017, ADV-FOG-008
#[test]
fn a_bottle_s_message_is_read_by_its_visitor_s_owner_alone_and_the_bottle_leaves() {
    let rules = rules(true);
    let read = |k| AdventureEvent {
        kind: AdventureEventKind::MessageRead {
            hero: HERO,
            object_type: type_of(&rules, k),
            anchor: AT,
        },
        seen_by: Viewers::NONE.with(RED),
    };
    // The bottle: its message to red alone, then it leaves where both see it; nothing given.
    let mut state = afloat(&rules, "t:bottle");
    let (outcome, events) = step(&rules, &mut state, &[], &[AT]).unwrap();
    assert_eq!(outcome, AdventureOutcome::default());
    assert_eq!(
        told(&events),
        vec![read("t:bottle"), removed(type_of(&rules, "t:bottle"))]
    );
    assert!(state.is_removed(0));
    // Red is told it as it is; blue nothing of the message, only the bottle's leaving.
    assert_eq!(read("t:bottle").project(RED), Some(read("t:bottle")));
    let projected: Vec<_> = events.iter().filter_map(|e| e.project(BLUE)).collect();
    assert!(
        !projected
            .iter()
            .any(|e| matches!(e.kind, AdventureEventKind::MessageRead { .. }))
    );
    // A message that stays (a sign's kind) is read at every visit.
    let mut state = afloat(&rules, "t:sign");
    for _ in 0..2 {
        let (_, events) = step(&rules, &mut state, &[], &[AT]).unwrap();
        assert_eq!(told(&events), vec![read("t:sign")]);
    }
    assert!(!state.is_removed(0));
}

// covers: ADV-PIK-018, ADV-MOV-009
#[test]
fn only_a_boat_takes_a_pickup_on_water() {
    let rules = rules(true);
    // From the shore at (2, 2) a walker, a flyer and a water walker never step toward it, and
    // none of them reaches its tile (D-45 reverted, D-46).
    let flyer = store(hero_details(&[(FLYING, 3)]));
    let wader = store(hero_details(&[(WATER_WALKING, 3)]));
    for (heroes, error) in [
        (
            store(hero()),
            AdventureCommandError::Blocked {
                step: 0,
                error: StandError::Terrain,
            },
        ),
        (
            flyer,
            AdventureCommandError::Blocked {
                step: 0,
                error: StandError::Blocked,
            },
        ),
        (
            wader,
            AdventureCommandError::Blocked {
                step: 0,
                error: StandError::Blocked,
            },
        ),
    ] {
        let mut state = sea(&rules, "t:flotsam", p(2, 2));
        assert_eq!(step(&rules, &mut state, &heroes, &[AT]), Err(error));
        let reach = state.reach(&rules.0, &rules.1, &heroes, HERO).unwrap();
        assert_eq!(reach.points_left(AT), None);
    }
    // A boat's reachable tiles end there, its path the move's.
    let state = afloat(&rules, "t:flotsam");
    let reach = state.reach(&rules.0, &rules.1, &[], HERO).unwrap();
    assert_eq!(reach.path(AT), Some(vec![AT]));
    assert_eq!(reach.points_left(AT), Some(1400));
}
