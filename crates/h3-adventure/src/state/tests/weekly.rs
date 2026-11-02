//! Tests of the weekly objects' visits (E08 T84b1; increment 9's design note §9): entered by any
//! hero, no question; the marks first (the team's, a Garden's too: `ADV-QUIRK-35`), then this
//! week's contents once for whoever comes first, or the visitor's owner told it is empty.

use super::economy::{GRASS_ROWS, Rules, object, rules, setup};
use super::*;
use crate::testing::started;

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const HERO: HeroId = HeroId::new(1);
/// The objects' entrance in these tests, two steps right of red's hero at (0, 7).
const ENTRANCE: Pos = Pos::new(2, 7, 0);
const GOLD: ResourceId = ResourceId::from_raw(1);

/// The game of the grass map with an object of type `k` at [`ENTRANCE`] (its weekly place 0).
fn game(rules: &Rules, k: &str) -> AdventureState {
    started(
        rules,
        &setup(&GRASS_ROWS, vec![object(rules, ENTRANCE.x, ENTRANCE.y, k)]),
    )
    .unwrap()
}

/// Red's hero's walk onto [`ENTRANCE`]: the outcome and the events after its steps.
fn visit(rules: &Rules, state: &mut AdventureState) -> (AdventureOutcome, Vec<AdventureEvent>) {
    let mut events = Vec::new();
    let walk = AdventureCommand::MoveHero {
        hero: HERO,
        path: vec![Pos::new(1, 7, 0), ENTRANCE],
    };
    let outcome = state
        .apply(&rules.0, &rules.1, &[], RED, &walk, &mut events)
        .unwrap();
    events.retain(|e| !matches!(e.kind, AdventureEventKind::HeroStepped { .. }));
    (outcome, events)
}

fn told(kind: AdventureEventKind, player: PlayerId) -> AdventureEvent {
    AdventureEvent {
        kind,
        seen_by: Viewers::NONE.with(player),
    }
}

// covers: ADV-WEK-001, ADV-VIS-006
#[test]
fn a_windmill_gives_its_contents_once_to_whoever_comes_first() {
    let rules = rules();
    let windmill = rules.1.object_type_by_key("t:windmill").unwrap();
    let marked = AdventureEventKind::ObjectMarked {
        object_type: windmill,
        anchor: ENTRANCE,
    };
    let empty = AdventureEventKind::ObjectEmpty {
        object_type: windmill,
        anchor: ENTRANCE,
    };
    let mut s = game(&rules, "t:windmill");
    let week = *s.weekly(0).unwrap();
    let before = s.state_hash();
    // Red's visit: entered, no question; red's mark told to red, then the week's contents as the
    // outcome's gain, the object emptied, its resource kept.
    let (outcome, events) = visit(&rules, &mut s);
    assert_eq!(s.hero(HERO).unwrap().pos, ENTRANCE);
    assert_eq!(
        outcome,
        AdventureOutcome {
            gains: vec![(week.resource, week.amount)],
            ..AdventureOutcome::default()
        }
    );
    assert_eq!(events, [told(marked, RED)]);
    assert_eq!(
        s.weekly(0),
        Some(&Weekly {
            amount: 0,
            marks: 0b01,
            ..week
        })
    );
    assert!(s.visited_by(0, RED) && !s.visited_by(0, BLUE));
    assert!(s.check_weekly(&rules.1));
    assert_ne!(s.state_hash(), before);
    // Blue's visit this week: its mark set and told, then nothing for it, told it is empty.
    let mut events = Vec::new();
    let outcome = s.visit_weekly(BLUE, (0, windmill), &mut events);
    assert_eq!(outcome, AdventureOutcome::default());
    assert_eq!(events, [told(marked, BLUE), told(empty, BLUE)]);
    assert_eq!(s.weekly(0).map(|w| (w.amount, w.marks)), Some((0, 0b11)));
    // Red again: its mark held, so only the empty object told; nothing written.
    let again = s.clone();
    let mut events = Vec::new();
    let outcome = s.visit_weekly(RED, (0, windmill), &mut events);
    assert_eq!(outcome, AdventureOutcome::default());
    assert_eq!(events, [told(empty, RED)]);
    assert_eq!(s.state_hash(), again.state_hash());
    // Filled again (as the week's step will): the next visit gives, without a new mark.
    s.set_weekly(
        0,
        Weekly {
            amount: 4,
            ..*s.weekly(0).unwrap()
        },
    );
    let mut events = Vec::new();
    let outcome = s.visit_weekly(RED, (0, windmill), &mut events);
    assert_eq!(outcome.gains, [(week.resource, 4)]);
    assert_eq!(events, []);
    assert_eq!(s.weekly(0).map(|w| (w.amount, w.marks)), Some((0, 0b11)));
    // An object that is no weekly one gives nothing and tells nothing.
    let mut events = Vec::new();
    let outcome = s.visit_weekly(RED, (1, windmill), &mut events);
    assert_eq!((outcome, events), (AdventureOutcome::default(), vec![]));
}

// covers: ADV-WEK-002, ADV-WEK-003, ADV-VIS-006
#[test]
fn a_water_wheel_and_a_garden_give_their_week_s_amounts() {
    let rules = rules();
    // The Water Wheel's first week: 500 gold.
    let mut wheel = game(&rules, "t:water_wheel");
    let (outcome, events) = visit(&rules, &mut wheel);
    assert_eq!(outcome.gains, [(GOLD, 500)]);
    assert_eq!(events.len(), 1);
    assert!(matches!(
        events[0].kind,
        AdventureEventKind::ObjectMarked { .. }
    ));
    assert_eq!(
        wheel.weekly(0).map(|w| (w.amount, w.marks)),
        Some((0, 0b01))
    );
    // The Garden's drawn gift; its mark the visitor's team's (`ADV-QUIRK-35` not reproduced: the
    // original's marks the visitor alone), until E10's teams the visitor's own.
    let mut garden = game(&rules, "t:garden");
    let gift = *garden.weekly(0).unwrap();
    let (outcome, _) = visit(&rules, &mut garden);
    assert_eq!(outcome.gains, [(gift.resource, gift.amount)]);
    assert!(garden.visited_by(0, RED) && !garden.visited_by(0, BLUE));
    assert!(garden.check_weekly(&rules.1));
}
