//! Tests of the battles at sea (E08 T74b; design §13; GAM-BAT-001, GAM-BAT-003 amended,
//! ADV-SEA-006): a battle with a hero in a boat on either side is fought on the sea terrain and
//! the deck, one with both on the two boats, before the underground and under a special ground's
//! battlefield; a monster's battle against a boat on the deck, a landing's guards' on land; after
//! the battle a beaten hero in a boat leaves the map with its boat, a winner stays in it. Batch
//! E8's rows as sources: E8-11 (the deck from the shore, the beaten hero's boat freed), E8-12 (the
//! two boats, a retreated hero's boat gone, the winner still in its boat), E8-13 (a water
//! monster's battle at once, on the deck), E8-15 (a landing's guards on land).

use std::vec;
use std::vec::Vec;

use h3_content::ContentKey;

use super::*;
use crate::command::{AdventureCommand, AdventureEvent, AdventureEventKind, Viewers};
use crate::setup::{ObjectSetup, Pos};
use crate::testing::{GRASS, HALLOW, PEASANT, SWAMP, coasted, hero_at, lying, started};

type Rules = (Ruleset, AdventureRuleset);

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const HERO: HeroId = HeroId::new(1);
const BLUE_HERO: HeroId = HeroId::new(2);

fn p(x: u16, y: u16) -> Pos {
    Pos::new(x, y, 0)
}

fn rules() -> Rules {
    crate::testing::movement_rules(true)
}

/// The part's sea terrain, deck and two boats.
fn sea(rules: &Rules) -> (TerrainId, Option<ContentKey>, Option<ContentKey>) {
    let battle = rules.1.params().battle_terrains().unwrap();
    (
        battle.sea,
        Some(battle.deck.clone()),
        Some(battle.boats.clone()),
    )
}

/// Water on (1, 0), (2, 0) and the two boats' tiles (1, 1) (boat 0) and (2, 1) (boat 1, of type
/// 2); land on (0, *) and the bottom row. Red's hero 1 on (2, 2) below blue's boat, blue's hero 2
/// in boat 1 (it stood on (3, 1)); `edit` applied to the setup first.
fn boats(rules: &Rules, edit: impl FnOnce(&mut AdventureSetup)) -> AdventureState {
    let mut setup = coasted(
        &["g~~g", "gbbg", "gggg"],
        vec![
            hero_at(1, 0, p(2, 2), &[PEASANT]),
            hero_at(2, 1, p(3, 1), &[PEASANT]),
        ],
    );
    setup.map.boats[1].kind = 2;
    edit(&mut setup);
    let mut state = started(rules, &setup).unwrap();
    state.put_aboard(BLUE_HERO, 1);
    state
}

/// Red's hero 1 aboard boat 0 at (1, 1) with a boat's day of points.
fn red_aboard(state: &mut AdventureState) {
    state.put_aboard(HERO, 0);
    let hero = &mut state.heroes[0];
    (hero.movement, hero.max_movement) = (1500, 1500);
}

/// Red's hero 1 moves along `path`: the battle it starts, its tile, terrain and battlefield.
fn battle(
    state: &mut AdventureState,
    rules: &Rules,
    path: &[Pos],
) -> Option<(Pos, TerrainId, Option<ContentKey>)> {
    let command = AdventureCommand::MoveHero {
        hero: HERO,
        path: path.to_vec(),
    };
    let outcome = state
        .apply(&rules.0, &rules.1, &[], RED, &command, &mut Vec::new())
        .unwrap();
    assert!(state.check_boats(&rules.1));
    outcome.battle.map(|b| (b.tile, b.terrain, b.battlefield))
}

/// A wandering monster of [`crate::testing::world`]'s `M` anchored on `at`, savage and never
/// fleeing: its visit is a battle, never an offer.
fn monster(at: Pos) -> ObjectSetup {
    let mut monster = crate::testing::world(&["M"], 1, vec![])
        .map
        .objects
        .remove(0);
    let wanderer = monster.monster.as_mut().unwrap();
    (wanderer.disposition, wanderer.never_flees) = (4, true);
    ObjectSetup {
        anchor: at,
        ..monster
    }
}

// covers: GAM-BAT-001, ADV-SEA-005
#[test]
fn a_battle_with_a_hero_in_a_boat_is_fought_on_the_deck_and_with_both_on_the_two_boats() {
    let rules = rules();
    let (water, deck, two) = sea(&rules);
    // From the shore red's walker attacks blue's hero in its boat: the deck, at blue's tile
    // (batch E8's E8-11).
    let mut state = boats(&rules, |_| {});
    assert_eq!(
        battle(&mut state, &rules, &[p(2, 1)]),
        Some((p(2, 1), water, deck.clone()))
    );
    // From the boat beside it red's hero in a boat attacks: the two boats (E8-12).
    let mut state = boats(&rules, |_| {});
    red_aboard(&mut state);
    assert_eq!(
        battle(&mut state, &rules, &[p(2, 1)]),
        Some((p(2, 1), water, two))
    );
    // A water monster on (2, 0) attacks red's boat at once on a water tile it guards: the deck,
    // at the monster's tile (E8-13). Blue's hero stays ashore on (3, 1), its boat empty.
    let mut setup = coasted(
        &["g~~g", "gbbg", "gggg"],
        vec![
            hero_at(1, 0, p(2, 2), &[PEASANT]),
            hero_at(2, 1, p(3, 1), &[PEASANT]),
        ],
    );
    setup.map.objects.push(monster(p(2, 0)));
    let mut state = started(&rules, &setup).unwrap();
    red_aboard(&mut state);
    assert_eq!(
        battle(&mut state, &rules, &[p(1, 0)]),
        Some((p(2, 0), water, deck))
    );
}

// covers: GAM-BAT-001, ADV-SEA-003
#[test]
fn a_landing_s_guards_fight_on_land() {
    // The landing's guards attack once the hero is ashore, out of its boat (E8-15): the guards'
    // tile's ground, here grass off the coast, not the deck.
    let rules = rules();
    let mut setup = coasted(
        &["gggg~", "gggb~", "ggggg"],
        vec![hero_at(1, 0, p(4, 2), &[PEASANT])],
    );
    setup.map.objects.push(monster(p(1, 1)));
    let mut state = started(&rules, &setup).unwrap();
    red_aboard(&mut state);
    assert_eq!(
        battle(&mut state, &rules, &[p(2, 1)]),
        Some((p(1, 1), GRASS, None))
    );
    assert_eq!(state.hero(HERO).unwrap().boat, None);
}

// covers: GAM-BAT-001, ADV-GRD-001
#[test]
fn a_battle_at_sea_comes_before_the_underground_and_under_a_special_ground() {
    let rules = rules();
    let (water, deck, two) = sea(&rules);
    // Two levels of two rows: the surface's land at (0, 0), all water underground.
    let mut setup = crate::testing::world(&["g~", "~~", "~~", "~~"], 2, vec![]);
    // A ground lying on the water tile (0, 1) of the surface.
    setup.map.objects.push(lying(HALLOW, p(0, 1), 1));
    let state = started(&rules, &setup).unwrap();
    let hallow = Some(crate::testing::key("t:hallow"));
    let under = Pos::new(1, 0, 1);
    // None in a boat: the tile's own ground.
    assert_eq!(
        state.sea_battle_ground(&rules.1, p(1, 1), 0),
        state.battle_ground(&rules.1, p(1, 1))
    );
    assert_eq!(state.sea_battle_ground(&rules.1, under, 0), (SWAMP, None));
    // One: the deck; two: the two boats; underground too.
    assert_eq!(
        state.sea_battle_ground(&rules.1, p(1, 1), 1),
        (water, deck.clone())
    );
    assert_eq!(state.sea_battle_ground(&rules.1, p(1, 1), 2), (water, two));
    assert_eq!(state.sea_battle_ground(&rules.1, under, 1), (water, deck));
    // A special ground replaces the battlefield and keeps the sea terrain.
    assert_eq!(
        state.sea_battle_ground(&rules.1, p(0, 1), 1),
        (water, hallow.clone())
    );
    assert_eq!(
        state.sea_battle_ground(&rules.1, p(0, 1), 2),
        (water, hallow)
    );
    // Without the part's battle terrains, the tile's own.
    let plain = crate::testing::rules();
    let setup = crate::testing::world(&["gg"], 1, vec![]);
    let state = started(&plain, &setup).unwrap();
    assert_eq!(state.sea_battle_ground(&plain.1, p(1, 0), 2), (GRASS, None));
}

// covers: ADV-SEA-006, GAM-BAT-003
#[test]
fn a_beaten_hero_in_a_boat_leaves_the_map_with_it_and_a_winner_stays_in_its_boat() {
    let rules = rules();
    let mut state = boats(&rules, |_| {});
    red_aboard(&mut state);
    let survivors = crate::testing::army(&[PEASANT]);
    let mut events = Vec::new();
    let fought = state.clone();
    state.end_battle(
        &rules.0,
        &[(HERO, Some(&survivors[..])), (BLUE_HERO, None)],
        &mut events,
    );
    // Blue's hero leaves the map and its boat with it, told to the same players (both see
    // heroes on its tile), the boat by its type, never its index.
    let seen_by = Viewers::NONE.with(RED).with(BLUE);
    assert_eq!(
        events,
        vec![
            AdventureEvent {
                kind: AdventureEventKind::HeroRemoved {
                    hero: BLUE_HERO,
                    pos: p(2, 1),
                },
                seen_by,
            },
            AdventureEvent {
                kind: AdventureEventKind::BoatRemoved {
                    hero: BLUE_HERO,
                    boat: 2,
                    at: p(2, 1),
                },
                seen_by,
            },
        ]
    );
    assert_eq!(events[1].project(RED).unwrap().kind, events[1].kind);
    assert_eq!(
        events[1].project(RED).unwrap().seen_by,
        Viewers::NONE.with(RED)
    );
    // The record stays by index: gone, nobody aboard, its owner and captain kept (E8-11); no
    // empty boat lies on its tile and nobody boards it.
    let boat = *state.boat(1).unwrap();
    assert_eq!(
        (boat.tile, boat.aboard, boat.gone, boat.owner, boat.captain),
        (p(2, 1), false, true, Some(BLUE), Some(BLUE_HERO))
    );
    assert!(!state.is_empty_boat(p(2, 1)));
    assert_eq!(state.empty_boat(p(2, 1)), None);
    assert_eq!(state.boats().count(), 2);
    // The winner stays in its boat on its tile (E8-12).
    let red = state.hero(HERO).unwrap();
    assert_eq!((red.pos, red.boat), (p(1, 1), Some(0)));
    assert!(state.boat(0).unwrap().aboard);
    assert!(state.check_boats(&rules.1));
    assert_ne!(state.state_hash(), fought.state_hash());
    // Neither sees a boat there any more.
    for viewer in [RED, BLUE] {
        let view = state.observe(&rules.0, &rules.1, viewer);
        assert!(view.boats.is_empty(), "{viewer:?}");
    }
    // A draw takes both, each with its boat, in the order given.
    let mut state = fought;
    let mut events = Vec::new();
    state.end_battle(&rules.0, &[(HERO, None), (BLUE_HERO, None)], &mut events);
    let removed: Vec<(HeroId, Option<u8>)> = events
        .iter()
        .map(|e| match e.kind {
            AdventureEventKind::HeroRemoved { hero, .. } => (hero, None),
            AdventureEventKind::BoatRemoved { hero, boat, .. } => (hero, Some(boat)),
            _ => panic!("{e:?}"),
        })
        .collect();
    assert_eq!(
        removed,
        [
            (HERO, None),
            (HERO, Some(0)),
            (BLUE_HERO, None),
            (BLUE_HERO, Some(2))
        ]
    );
    assert!(state.boats().all(|(_, boat)| boat.gone && !boat.aboard));
    assert!(state.heroes().is_empty());
    assert!(state.check_boats(&rules.1));
}
