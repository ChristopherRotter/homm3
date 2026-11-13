//! Tests of the Portal of Summoning's draws in the game (E09 T26b, the town epic's design note §45,
//! decisions 71 and 82; TWN-GRO-007): each week from the towns' stream, at a command's build from
//! the builder's stream, at a town event's from the towns' stream, at the setup's from the setup
//! stream after every other draw, at a dwelling's flag and at a capture from the player's stream;
//! never in a neutral town; the owner's view told (TWN-VEW-004).

use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{AdventureCommand, Pos};
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_kernel::{Rng, StreamId};
use h3_towns::{BuildingId, BuildingRoles, BuildingsSetup, SlotSet, TownCommand, TownEventSetup};

use super::command::{BLUE, RED};
use super::garrison::building;
use super::growth::{T0, T1, T2, T3, built, rules_on, setup, to_day_told};
use crate::{
    Actor, GameCommand, GameEvent, GameRuleset, GameState, Summoned, TownBuildingsView, apply,
};

/// The Portal, building 13 of the test town's record, on map slot [`PORTAL_SLOT`].
const PORTAL: usize = 13;
const PORTAL_SLOT: u8 = 22;
const HERO: HeroId = HeroId::new(0);

/// The growth tests' rules with the Portal and the neutral garrisons' week never growing.
fn rules_portal() -> GameRuleset {
    rules_on([1, 0, 0], |towns| {
        let portal = BuildingRoles {
            portal: true,
            ..BuildingRoles::default()
        };
        towns.towns[0]
            .buildings
            .push(building("b13", Some(PORTAL_SLOT), portal));
    })
}

/// The Portal's id.
fn portal() -> BuildingId {
    BuildingId::from_index(PORTAL).unwrap()
}

/// `game` with the Portal built in each of `towns` by the store, which draws nothing.
fn with_portals(rules: &GameRuleset, game: &mut GameState, towns: &[TownId]) {
    for &town in towns {
        let (part, core) = (rules.towns(), rules.core());
        game.towns_mut().build(town, portal(), part, core, false);
    }
}

/// What a draw of `town`'s Portal in `game` from `stream` gives, `player`'s dwellings on the map
/// its offer (`h3_towns::summon`; the town's Portal standing).
fn drawn(
    game: &GameState,
    rules: &GameRuleset,
    (town, player): (TownId, PlayerId),
    stream: &mut Rng,
) -> Option<Summoned> {
    let offer: Vec<_> =
        crate::town::growth::owned_dwellings(game.adventure(), rules, player).collect();
    let record = game.towns().get(town).unwrap();
    h3_towns::summon(record, rules.towns(), rules.core(), &offer, stream).portal()
}

/// The Portal `player`'s last view of `town` in `events` shows, if it was told one with the
/// owner's facts.
fn told(events: &[GameEvent], player: PlayerId, town: TownId) -> Option<Option<Summoned>> {
    events.iter().rev().find_map(|e| match *e {
        GameEvent::TownChanged {
            player: p,
            town: t,
            buildings,
            ..
        } if p == player && t == town => Some(buildings.map(|b: TownBuildingsView| b.portal)),
        _ => None,
    })?
}

/// Red's or blue's stream of seed 7 as the game's build leaves it (no draw).
fn player_stream(player: PlayerId) -> Rng {
    Rng::for_stream(7, StreamId::player(player.raw()))
}

/// The creatures a Portal of red's may hold: the hut's imps or gogs at their growth.
fn of_the_hut(rules: &GameRuleset, summoned: Option<Summoned>) -> bool {
    let core = rules.core();
    summoned.is_some_and(|s| {
        [("t:imp", 15), ("t:gog", 8)]
            .iter()
            .any(|&(k, n)| core.creature_by_key(k) == Some(s.creature) && s.count == n)
    })
}

// covers: TWN-GRO-007, TWN-VEW-004
#[test]
fn a_portal_is_drawn_each_week_from_the_towns_stream_in_an_owned_town_only() {
    let rules = rules_portal();
    let (core, part) = (rules.core(), rules.towns());
    let mut game = built(&rules, &setup(&rules));
    with_portals(&rules, &mut game, &[T0, T3]);
    assert_eq!(game.towns().get(T0).unwrap().portal(), None);
    // The towns' stream on day 8: the week's kind, the neutral T3's garrison odds (never), then
    // red's T0 with red's hut (T2 has no Portal; the neutral T3's is never drawn).
    let mut stream = Rng::for_stream(7, StreamId::TOWNS);
    let _ = h3_towns::draw_week(game.towns(), part, core, false, false, &mut stream);
    let _ = stream.below(101);
    let want = drawn(&game, &rules, (T0, RED), &mut stream);
    assert!(of_the_hut(&rules, want));
    let events = to_day_told(&rules, &mut game, 8);
    assert_eq!(game.towns().get(T0).unwrap().portal(), want);
    assert_eq!(game.towns().get(T3).unwrap().portal(), None);
    // TWN-VEW-004: told to red with the towns' day; blue's view of T0 shows no Portal.
    assert_eq!(told(&events, RED, T0), Some(want));
    assert_eq!(told(&events, BLUE, T0).flatten(), None);
}

// covers: TWN-GRO-007, TWN-BLD-001
#[test]
fn a_command_s_portal_build_draws_it_from_the_builder_s_stream() {
    let rules = rules_portal();
    let mut game = built(&rules, &setup(&rules));
    let mut copy = game.clone();
    with_portals(&rules, &mut copy, &[T0]);
    let want = drawn(&copy, &rules, (T0, RED), &mut player_stream(RED));
    assert!(of_the_hut(&rules, want));
    let build = TownCommand::Build {
        town: T0,
        building: portal(),
    };
    let mut events = Vec::new();
    apply(
        &mut game,
        &rules,
        Actor::Player(RED),
        GameCommand::Town(build),
        &mut events,
    )
    .unwrap();
    assert_eq!(game.towns().get(T0).unwrap().portal(), want);
    let built = events
        .iter()
        .position(|e| matches!(e, GameEvent::TownBuilt { .. }))
        .unwrap();
    assert_eq!(told(&events[built..], RED, T0), Some(want));
}

// covers: TWN-GRO-007, TWN-EVT-003
#[test]
fn a_town_event_s_portal_build_draws_it_from_the_towns_stream() {
    let rules = rules_portal();
    let mut setup = setup(&rules);
    setup.towns[2].events = vec![TownEventSetup {
        players: vec![RED],
        human: true,
        computer: true,
        first_day: 2,
        interval: 0,
        resources: vec![],
        buildings: SlotSet::EMPTY.with(PORTAL_SLOT),
        creatures: [0; 7],
    }];
    let mut game = built(&rules, &setup);
    let mut copy = game.clone();
    with_portals(&rules, &mut copy, &[T2]);
    // The towns' stream draws nothing before day 8.
    let want = drawn(
        &copy,
        &rules,
        (T2, RED),
        &mut Rng::for_stream(7, StreamId::TOWNS),
    );
    assert!(of_the_hut(&rules, want));
    let events = to_day_told(&rules, &mut game, 2);
    assert_eq!(game.towns().get(T2).unwrap().portal(), want);
    assert_eq!(told(&events, RED, T2), Some(want));
}

// covers: TWN-GRO-007
#[test]
fn the_setup_s_portals_are_drawn_last_from_the_setup_stream() {
    let rules = rules_portal();
    let plain = setup(&rules);
    let mut with = plain.clone();
    for town in [0, 1, 3] {
        with.towns[town].buildings = BuildingsSetup::Custom {
            built: SlotSet::EMPTY.with(PORTAL_SLOT),
            forbidden: SlotSet::EMPTY,
        };
    }
    let a = GameState::new(&rules, &plain, 7).unwrap();
    let b = GameState::new(&rules, &with, 7).unwrap();
    // Red's T0 holds one of its hut's creatures; blue's T1 nothing (blue owns no dwelling); the
    // neutral T3 nothing.
    let portal = |town| b.towns().get(town).unwrap().portal();
    assert!(of_the_hut(&rules, portal(T0)));
    assert_eq!(portal(T1), None);
    assert_eq!(portal(T3), None);
    // The setup's other draws as without them: the Portals' are its last.
    assert_eq!(a.adventure().state_hash(), b.adventure().state_hash());
    assert_eq!(a.hero(HERO), b.hero(HERO));
}

// covers: TWN-GRO-007, ADV-OWN-001, TWN-VEW-004
#[test]
fn a_dwelling_s_flag_draws_its_owner_s_empty_portals_from_its_stream() {
    let rules = rules_portal();
    // Red's hero beside the neutral hut on (0, 2) (a hut is not entered from above).
    let mut setup = setup(&rules);
    setup.heroes[0].pos = Pos::new(1, 2, 0);
    let mut game = built(&rules, &setup);
    // Red's T0 and T2 with a Portal holding nothing (the store's builds draw nothing), blue's T1
    // too.
    with_portals(&rules, &mut game, &[T0, T1, T2]);
    // Red's hero steps onto the hut: flagged, then the hire question; red's empty Portals drawn
    // in `TownId` order from red's stream (the visit drew nothing).
    let step = GameCommand::Adventure(AdventureCommand::MoveHero {
        hero: HERO,
        path: [Pos::new(0, 2, 0)].to_vec(),
    });
    let mut events = Vec::new();
    apply(&mut game, &rules, Actor::Player(RED), step, &mut events).unwrap();
    assert_eq!(game.adventure().owner(5), Some(RED));
    let mut stream = player_stream(RED);
    let mut emptied = game.clone();
    for town in [T0, T2] {
        let record = emptied.towns().get(town).unwrap().with_portal(None);
        emptied.towns_mut().write(town, record);
    }
    let want = [T0, T2].map(|town| drawn(&emptied, &rules, (town, RED), &mut stream));
    assert!(want.iter().all(|&w| of_the_hut(&rules, w)));
    for (town, want) in [T0, T2].into_iter().zip(want) {
        assert_eq!(game.towns().get(town).unwrap().portal(), want);
        assert_eq!(told(&events, RED, town), Some(want));
    }
    // Blue's is blue's to draw.
    assert_eq!(game.towns().get(T1).unwrap().portal(), None);
}

// covers: TWN-GRO-007, TWN-CAP-001
#[test]
fn a_capture_draws_a_portal_that_holds_nothing_from_the_capturer_s_stream() {
    let rules = rules_portal();
    let mut setup = setup(&rules);
    setup.towns[3].garrison = Some(Vec::new());
    let mut game = built(&rules, &setup);
    with_portals(&rules, &mut game, &[T3]);
    // Red's hero steps from T0's entrance onto the undefended neutral T3's on (2, 0).
    let step = GameCommand::Adventure(AdventureCommand::MoveHero {
        hero: HERO,
        path: [Pos::new(2, 0, 0)].to_vec(),
    });
    let mut events = Vec::new();
    apply(&mut game, &rules, Actor::Player(RED), step, &mut events).unwrap();
    let index = game.adventure().map().town_object(T3).unwrap();
    assert_eq!(game.adventure().owner(index), Some(RED));
    let want = drawn(&game, &rules, (T3, RED), &mut player_stream(RED));
    assert!(of_the_hut(&rules, want));
    assert_eq!(game.towns().get(T3).unwrap().portal(), want);
    assert_eq!(told(&events, RED, T3), Some(want));
}

// covers: TWN-GRO-007
#[test]
fn a_capture_keeps_a_portal_that_holds_creatures() {
    let rules = rules_portal();
    let mut setup = setup(&rules);
    setup.towns[3].garrison = Some(Vec::new());
    let mut game = built(&rules, &setup);
    with_portals(&rules, &mut game, &[T3]);
    let held = Some(Summoned {
        creature: rules.core().creature_by_key("t:gog").unwrap(),
        count: 2,
    });
    let record = game.towns().get(T3).unwrap().with_portal(held);
    game.towns_mut().write(T3, record);
    let step = GameCommand::Adventure(AdventureCommand::MoveHero {
        hero: HERO,
        path: [Pos::new(2, 0, 0)].to_vec(),
    });
    apply(&mut game, &rules, Actor::Player(RED), step, &mut Vec::new()).unwrap();
    assert_eq!(game.towns().get(T3).unwrap().portal(), held);
}
