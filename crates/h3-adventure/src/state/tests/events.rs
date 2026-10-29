//! Tests of events (E08 T62b2; design §12, §13): the step that fires one and ends the move there,
//! the colours and the computer flag, the repeats, the guards at once and their battle's end, an
//! event on a guarded tile after its guard's visit (E08 T63b), a flyer stopped on one, and what no
//! player sees of one: never in a view, nothing in the paths or the reachable tiles.

use std::boxed::Box;
use std::sync::Arc;
use std::vec;
use std::vec::Vec;

use h3_heroes::HeroDetails;

use super::draws::object;
use super::rewards::{GOLD, HERO, RED, Rules, hero, rules, store};
use super::*;
use crate::command::{AdventureCommand, AdventureCommandError, Defender};
use crate::setup::{EventSetup, Guards, Holds, MonsterSetup, RewardRecord, Seat};
use crate::testing::{FLYING, MONSTER, PEASANT, PIKEMAN, hero_at, hero_details, started, world};

/// Red's colour bit.
const RED_ONLY: u8 = 0b01;
/// Blue's colour bit.
const BLUE_ONLY: u8 = 0b10;

/// A record giving 100 gold.
fn gold() -> RewardRecord {
    RewardRecord {
        resources: vec![(GOLD, 100)],
        ..RewardRecord::default()
    }
}

/// An event lying on `(x, 1)` giving 100 gold, firing for `colours` (and for computers with
/// `computer`), leaving the map after it fires with `remove`. Its template blocks its one
/// visitable tile, as every shipped map's event's does: an event blocks nothing all the same.
fn event(rules: &Rules, x: u16, colours: u8, computer: bool, remove: bool) -> ObjectSetup {
    let holds = Holds::Event(Box::new(EventSetup {
        record: gold(),
        colours,
        computer,
        remove,
    }));
    ObjectSetup {
        anchor: Pos::new(x, 1, 0),
        ..object(rules, x, "t:event", holds)
    }
}

/// A started 16 × 2 grass map with these objects, red's hero 1 at (1, 1) and blue's hero 2 at
/// (14, 1); red counts as human when `human`.
fn map(rules: &Rules, objects: Vec<ObjectSetup>, human: bool) -> AdventureState {
    let mut setup = world(
        &["gggggggggggggggg", "gggggggggggggggg"],
        1,
        vec![
            hero_at(1, 0, Pos::new(1, 1, 0), &[PEASANT]),
            hero_at(2, 1, Pos::new(14, 1, 0), &[PEASANT]),
        ],
    );
    setup.map.objects.extend(objects);
    setup.seats[0] = Seat { colour: 0, human };
    started(rules, &setup).unwrap()
}

/// The tiles `(x, 1)` for these columns.
fn row(columns: core::ops::RangeInclusive<u16>) -> Vec<Pos> {
    columns.map(|x| Pos::new(x, 1, 0)).collect()
}

/// Red's hero 1 moves along `path` with `heroes`: the outcome and the events.
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

/// The `EventFired` events among `events`, with their viewers.
fn fired(events: &[AdventureEvent]) -> Vec<(AdventureEventKind, Viewers)> {
    events
        .iter()
        .filter(|e| matches!(e.kind, AdventureEventKind::EventFired { .. }))
        .map(|e| (e.kind, e.seen_by))
        .collect()
}

/// The tile and the points left of red's hero 1.
fn standing(state: &AdventureState) -> (Pos, u32) {
    let hero = state.hero(HERO).unwrap();
    (hero.pos, hero.movement)
}

// covers: ADV-OBJ-007, ADV-MOV-011, ADV-RWD-001, ADV-FOG-007
#[test]
fn an_event_fires_on_the_step_onto_its_tile_and_ends_the_move_there() {
    let rules = rules(true);
    let heroes = store(hero());
    let mut s = map(&rules, vec![event(&rules, 4, RED_ONLY, false, true)], true);
    let (outcome, events) = walk(&rules, &mut s, &heroes, &row(2..=6));
    // The record, the rest of the path dropped, the event gone.
    assert_eq!((outcome.gains, outcome.dropped), (vec![(GOLD, 100)], 2));
    assert!(s.is_removed(0));
    // The hero stands on its tile with the points a move to it leaves.
    let mut plain = map(&rules, vec![], true);
    walk(&rules, &mut plain, &heroes, &row(2..=4));
    assert_eq!(standing(&s), standing(&plain));
    // Three steps, then the firing told to red alone; the removal is told to nobody.
    let tile = Pos::new(4, 1, 0);
    assert_eq!(
        fired(&events),
        [(
            AdventureEventKind::EventFired { hero: HERO, tile },
            Viewers::NONE.with(RED)
        )]
    );
    let steps = |e: &&AdventureEvent| matches!(e.kind, AdventureEventKind::HeroStepped { .. });
    assert_eq!(events.iter().filter(steps).count(), 3);
    assert_eq!(events.len(), 4);
    // An event at the path's end drops nothing.
    let mut s = map(&rules, vec![event(&rules, 4, RED_ONLY, false, true)], true);
    let (outcome, _) = walk(&rules, &mut s, &heroes, &row(2..=4));
    assert_eq!((outcome.gains, outcome.dropped), (vec![(GOLD, 100)], 0));
}

// covers: ADV-OBJ-007, ADV-MOV-011
#[test]
fn an_event_fires_for_its_colours_and_a_computer_only_with_its_flag() {
    let rules = rules(true);
    let heroes = store(hero());
    let crossed = |object, human| {
        let mut s = map(&rules, vec![object], human);
        let (outcome, events) = walk(&rules, &mut s, &heroes, &row(2..=6));
        (outcome.gains, !fired(&events).is_empty(), standing(&s).0)
    };
    let end = Pos::new(6, 1, 0);
    let stop = Pos::new(4, 1, 0);
    // Blue's only: red's hero walks on over it, nothing changes.
    let blue = event(&rules, 4, BLUE_ONLY, true, true);
    assert_eq!(crossed(blue, true), (vec![], false, end));
    // Red's, for a red that does not count as human: only with the computer flag.
    let red = |computer| event(&rules, 4, RED_ONLY, computer, true);
    assert_eq!(crossed(red(false), false), (vec![], false, end));
    assert_eq!(crossed(red(true), false), (vec![(GOLD, 100)], true, stop));
    assert_eq!(crossed(red(false), true), (vec![(GOLD, 100)], true, stop));
}

// covers: ADV-OBJ-007, ADV-MOV-011
#[test]
fn an_event_without_its_removal_flag_fires_at_every_entry_and_never_from_its_start() {
    let rules = rules(true);
    let heroes = store(hero());
    let mut s = map(&rules, vec![event(&rules, 2, RED_ONLY, false, false)], true);
    let at = Pos::new(2, 1, 0);
    let (outcome, _) = walk(&rules, &mut s, &heroes, &[at]);
    assert_eq!(outcome.gains, [(GOLD, 100)]);
    assert!(!s.is_removed(0));
    // Leaving it fires nothing; entering it again fires it again.
    let (outcome, events) = walk(&rules, &mut s, &heroes, &row(3..=3));
    assert_eq!(
        (outcome, fired(&events).len()),
        (AdventureOutcome::default(), 0)
    );
    let (outcome, _) = walk(&rules, &mut s, &heroes, &[at]);
    assert_eq!(outcome.gains, [(GOLD, 100)]);
    // A path that comes back onto it fires it at that step.
    let (outcome, _) = walk(
        &rules,
        &mut s,
        &heroes,
        &[Pos::new(3, 1, 0), at, Pos::new(1, 1, 0)],
    );
    assert_eq!((outcome.gains, outcome.dropped), (vec![(GOLD, 100)], 1));
}

// covers: ADV-OBJ-007, ADV-PIK-004
#[test]
fn an_events_guards_fight_at_once_and_its_record_follows_a_win() {
    let rules = rules(true);
    let heroes = store(hero());
    let guards = vec![ArmyStack {
        slot: 0,
        creature: PIKEMAN,
        count: 4,
    }];
    let guarded = ObjectSetup {
        guards: Some(Guards {
            army: guards.clone(),
            question: false,
        }),
        ..event(&rules, 3, RED_ONLY, false, false)
    };
    let mut s = map(&rules, vec![guarded], true);
    let tile = Pos::new(3, 1, 0);
    let (outcome, events) = walk(&rules, &mut s, &heroes, &row(2..=4));
    // Announced, not asked: the battle against the guards at the event's tile.
    let battle = outcome.battle.unwrap();
    assert_eq!(
        (battle.defender, battle.tile, outcome.dropped, outcome.gains),
        (Defender::Guards { army: guards }, tile, 1, vec![])
    );
    assert_eq!(fired(&events).len(), 1);
    let ended = |s: &mut AdventureState, survivors: &[ArmyStack], won| {
        let rules = (&rules.0, &rules.1, &heroes[..]);
        s.end_unowned_battle(rules, (HERO, tile), (survivors, won), &mut Vec::new())
    };
    // Lost: the survivors stay as its guards.
    let left = [ArmyStack {
        slot: 0,
        creature: PIKEMAN,
        count: 1,
    }];
    let mut lost = s.clone();
    assert_eq!(ended(&mut lost, &left, false), AdventureOutcome::default());
    assert_eq!(lost.guards_left(0), left);
    // Won: the guards gone for good, the record after the winner's levels (E08 T63b,
    // GAM-DEC-007: `give_won`), so the next entry gives it without a battle.
    let after = AdventureOutcome {
        reward_after: Some((HERO, tile)),
        ..AdventureOutcome::default()
    };
    assert_eq!(ended(&mut s, &[], true), after);
    let parts = (&rules.0, &rules.1, &heroes[..]);
    assert_eq!(
        s.give_won(parts, (HERO, tile), &mut Vec::new()).gains,
        [(GOLD, 100)]
    );
    assert!(s.guards_left(0).is_empty() && !s.is_removed(0));
    walk(&rules, &mut s, &heroes, &row(4..=4));
    let (outcome, _) = walk(&rules, &mut s, &heroes, &[tile]);
    assert_eq!((outcome.battle, outcome.gains), (None, vec![(GOLD, 100)]));
}

/// Five savage pikemen at (5, 0), guarding (4, 1) and (6, 1) (ADV-MON-005).
fn pikemen(rules: &Rules) -> ObjectSetup {
    ObjectSetup {
        anchor: Pos::new(5, 0, 0),
        object_type: Some(MONSTER),
        monster: Some(MonsterSetup {
            creature: Some(PIKEMAN),
            count: 5,
            disposition: 4,
            never_flees: true,
            no_growth: false,
            treasure: vec![],
            artifact: None,
        }),
        ..object(rules, 5, "t:event", Holds::Nothing)
    }
}

// covers: ADV-OBJ-007, ADV-MON-005, ADV-MON-006
#[test]
fn an_event_on_a_guarded_tile_fires_after_the_guards_visit() {
    let rules = rules(true);
    let heroes = store(hero());
    let parts = (&rules.0, &rules.1, &heroes[..]);
    let tile = Pos::new(4, 1, 0);
    let mut s = map(
        &rules,
        vec![pikemen(&rules), event(&rules, 4, RED_ONLY, false, true)],
        true,
    );
    // The step onto it: the guard's battle, the event held (E08 T63b; before it, the event did
    // not fire, `ADV-STUB-10`), nothing fired yet.
    let (outcome, events) = walk(&rules, &mut s, &heroes, &row(2..=4));
    let battle = outcome.battle.unwrap();
    assert_eq!(
        (outcome.event_after, outcome.gains, fired(&events).len()),
        (Some((HERO, tile)), vec![], 0)
    );
    // The guard beaten: the held event fires for the hero still on its tile.
    let ended = (HERO, battle.tile);
    s.end_unowned_battle(parts, ended, (&[], true), &mut Vec::new());
    let mut held = s.clone();
    let mut events = Vec::new();
    let outcome = held.fire_held_event(parts, (RED, HERO), tile, &mut events);
    assert_eq!(
        (outcome.gains, fired(&events).len()),
        (vec![(GOLD, 100)], 1)
    );
    assert!(held.is_removed(1));
    // Gone, off its tile, or another player's hero: nothing (A-26 fixed, `ADV-QUIRK-29`).
    let mut quiet = Vec::new();
    let elsewhere = Pos::new(3, 1, 0);
    let blue = PlayerId::new(1);
    for (player, at) in [(RED, tile), (RED, elsewhere), (blue, tile)] {
        let mut state = if at == tile && player == RED {
            held.clone()
        } else {
            s.clone()
        };
        let outcome = state.fire_held_event(parts, (player, HERO), at, &mut quiet);
        assert_eq!(outcome, AdventureOutcome::default());
    }
    assert!(quiet.is_empty());
}

// covers: ADV-MOV-011, ADV-MOV-009, ADV-MON-006
#[test]
fn a_flyer_stopped_on_a_guarded_event_meets_its_guard() {
    let rules = rules(true);
    let flyer = store(hero_details(&[(FLYING, 3)]));
    let mut s = map(
        &rules,
        vec![pikemen(&rules), event(&rules, 4, RED_ONLY, false, true)],
        true,
    );
    // Over (4, 1) toward (6, 1): the event's tile ends the move, and there the guard visits
    // (before E08 T63b the flyer flew on to (6, 1) and met it there).
    let (outcome, events) = walk(&rules, &mut s, &flyer, &row(2..=6));
    assert_eq!(standing(&s).0, Pos::new(4, 1, 0));
    assert_eq!(
        (outcome.dropped, outcome.event_after, fired(&events).len()),
        (2, Some((HERO, Pos::new(4, 1, 0))), 0)
    );
    assert!(outcome.battle.is_some());
}

// covers: ADV-MOV-011, ADV-MOV-009
#[test]
fn a_flyer_stops_on_an_event_it_may_stand_on_and_passes_one_it_may_not() {
    let rules = rules(true);
    let flyer = store(hero_details(&[(FLYING, 3)]));
    // A decoration blocking (3, 1), the flyer passing over it.
    let blocker = ObjectSetup {
        anchor: Pos::new(3, 1, 0),
        visitable: TileMask::default(),
        object_type: None,
        ..object(&rules, 3, "t:event", Holds::Nothing)
    };
    let mut s = map(
        &rules,
        vec![blocker.clone(), event(&rules, 4, RED_ONLY, false, true)],
        true,
    );
    let (outcome, _) = walk(&rules, &mut s, &flyer, &row(2..=6));
    assert_eq!((outcome.gains, outcome.dropped), (vec![(GOLD, 100)], 2));
    assert_eq!(standing(&s).0, Pos::new(4, 1, 0));
    // An event under the blocker: passed over, never fired.
    let mut s = map(
        &rules,
        vec![blocker, event(&rules, 3, RED_ONLY, false, true)],
        true,
    );
    let (outcome, events) = walk(&rules, &mut s, &flyer, &row(2..=6));
    assert_eq!((outcome.dropped, fired(&events).len()), (0, 0));
    assert_eq!(standing(&s).0, Pos::new(6, 1, 0));
}

// covers: ADV-FOG-007, ADV-MOV-011
#[test]
fn no_player_sees_an_event_in_its_views_its_paths_or_its_reach() {
    let rules = rules(true);
    let heroes = store(hero());
    let with = map(&rules, vec![event(&rules, 3, BLUE_ONLY, true, true)], true);
    let without = map(&rules, vec![], true);
    let (core, part) = (&rules.0, &rules.1);
    for player in [RED, PlayerId::new(1)] {
        // The objects and the tiles the same (an event's tile no object's), red's sight on it.
        let (a, b) = (
            with.observe(core, part, player),
            without.observe(core, part, player),
        );
        assert_eq!(a, b);
    }
    let tile = Pos::new(3, 1, 0);
    assert!(with.is_explored(RED, tile));
    assert_eq!(with.map().events_at(tile), [0]);
    // Its template blocks its tile: the event neither blocks it nor is visited from it.
    let static_tile = with.map().tile(tile).unwrap();
    assert!(!static_tile.blocked && !static_tile.visitable);
    assert_eq!(with.map().objects()[0].covers(tile), (false, false));
    assert_eq!(with.map().events_at(Pos::new(99, 1, 0)), [] as [usize; 0]);
    // The reachable tiles and their paths the same; a path may end on the event of another colour.
    let tiles = |s: &AdventureState| {
        let reach = s.reach(core, part, &heroes, HERO).unwrap();
        let tiles: Vec<(Pos, u32)> = reach.tiles().collect();
        let paths: Vec<Option<Vec<Pos>>> = tiles.iter().map(|&(p, _)| reach.path(p)).collect();
        (tiles, paths)
    };
    assert_eq!(tiles(&with), tiles(&without));
    let mut ending = with.clone();
    let (outcome, _) = walk(&rules, &mut ending, &heroes, &row(2..=3));
    assert_eq!(outcome, AdventureOutcome::default());
    // A rejected path is rejected alike, at the same step.
    let far = row(2..=15);
    let reject = |s: &AdventureState| {
        let command = AdventureCommand::MoveHero {
            hero: HERO,
            path: far.clone(),
        };
        s.clone()
            .apply(core, part, &heroes, RED, &command, &mut Vec::new())
            .unwrap_err()
    };
    assert_eq!(reject(&with), reject(&without));
    assert!(matches!(
        reject(&with),
        AdventureCommandError::Unexplored { .. } | AdventureCommandError::TooFewPoints { .. }
    ));
}

// covers: ADV-OBJ-007, ADV-MOV-011, ADV-SEA-005
#[test]
fn an_event_on_water_fires_for_a_boat_s_step_and_ends_the_move_there() {
    // Batch E8's E8-14: a boat's path over an event on water stops on its tile, the event fired
    // (E08 T73); the rest of the path is dropped.
    let rules = rules(true);
    let mut setup = world(
        &["gggggggggggggggg", "gb~~~~~~~~~~~~~~"],
        1,
        vec![
            hero_at(1, 0, Pos::new(0, 1, 0), &[PEASANT]),
            hero_at(2, 1, Pos::new(14, 0, 0), &[PEASANT]),
        ],
    );
    setup
        .map
        .objects
        .push(event(&rules, 4, RED_ONLY, false, true));
    setup.seats[0] = Seat {
        colour: 0,
        human: true,
    };
    let mut state = started(&rules, &setup).unwrap();
    state.put_aboard(HERO, 0);
    state.heroes[0].movement = 1500;
    let (outcome, events) = walk(&rules, &mut state, &store(hero()), &row(2..=5));
    let tile = Pos::new(4, 1, 0);
    assert_eq!(state.hero(HERO).unwrap().pos, tile);
    assert_eq!(state.hero(HERO).unwrap().movement, 1200);
    assert_eq!(outcome.dropped, 1);
    assert_eq!(
        fired(&events),
        [(
            AdventureEventKind::EventFired { hero: HERO, tile },
            Viewers::NONE.with(RED)
        )]
    );
}
