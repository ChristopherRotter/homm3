//! Tests of the hall's offer and the legal builds (E09 T18b2b, the town epic's design note §29):
//! `legal_commands` lists each owned town's payable builds in `TownId` and record order, each one
//! `apply` accepts; a town built in today lists none until the day's start; the offer is the
//! owner's alone, the Capitol's limit read across the owner's towns.

use alloc::vec::Vec;

use h3_towns::{BuildingId, BuildingSet, BuildingsSetup, SlotSet};

use super::CAPITOL_SLOT;
use super::command::{
    BLUE, CAPITOL, CITADEL, CITY_HALL, FORT, RED, SILO, T0, T1, TOWN_HALL, build, building,
    end_turn, priced, run, setup,
};
use h3_contracts::{PlayerId, TownId};

use crate::{
    Actor, GameCommand, GameCommandError, GameRuleset, GameState, TownOffer, legal_commands,
};

/// The record's other buildings of [`super::town_pack`]: the fort's third level, the two
/// dwellings, the Grail's building, the Lookout.
const CASTLE: usize = 6;
const DWELLING_1: usize = 7;
const DWELLING_2: usize = 8;
const GRAIL: usize = 9;
const LOOKOUT: usize = 11;

fn indices(set: BuildingSet) -> Vec<usize> {
    set.iter().map(BuildingId::index).collect()
}

fn listed(game: &GameState, rules: &GameRuleset, player: PlayerId) -> Vec<GameCommand> {
    legal_commands(game, rules, Actor::Player(player))
}

/// `EndTurn`, then the builds of each `(town, offer)` in turn, as `legal_commands` lists them.
fn expected(offers: &[(TownId, TownOffer)]) -> Vec<GameCommand> {
    core::iter::once(GameCommand::EndTurn)
        .chain(
            offers.iter().flat_map(|&(town, offer)| {
                offer.payable.iter().map(move |id| build(town, id.index()))
            }),
        )
        .collect()
}

// covers: TWN-BLD-001, TWN-BLD-002
#[test]
fn the_legal_builds_are_the_payable_part_of_the_owner_s_hall() {
    let rules = priced();
    let game = GameState::new(&rules, &setup(&rules), 7).unwrap();
    let offer = game.town_offer(&rules, RED, T0).unwrap();
    // The Village Hall stands, the City Hall waits for the Town Hall, the Grail's building is
    // never the hall's and the Shipyard has no water (forbidden from the start, TWN-SET-011).
    assert_eq!(
        indices(offer.buildable),
        [
            TOWN_HALL, CAPITOL, FORT, CITADEL, CASTLE, DWELLING_1, DWELLING_2, SILO, LOOKOUT
        ]
    );
    // Some day: the Village Hall ever built, the City Hall and the Grail's building too.
    assert_eq!(
        indices(offer.possible),
        [
            0, TOWN_HALL, CITY_HALL, CAPITOL, FORT, CITADEL, CASTLE, DWELLING_1, DWELLING_2, GRAIL,
            SILO, LOOKOUT
        ]
    );
    // The Citadel's 2 500 gold against the treasury's 2 000.
    assert_eq!(
        offer.payable,
        offer
            .buildable
            .difference(BuildingSet::EMPTY.with(building(CITADEL)))
    );
    let commands = listed(&game, &rules, RED);
    assert_eq!(commands, expected(&[(T0, offer)]));
    // Each listed build is one `apply` accepts; the unpaid one is refused for its price.
    for command in &commands[1..] {
        run(&rules, &mut game.clone(), RED, command.clone()).unwrap();
    }
    assert_eq!(
        run(&rules, &mut game.clone(), RED, build(T0, CITADEL)),
        Err(GameCommandError::CannotPay)
    );
    // The offer is the owner's alone, in its turn or not; blue lists no build in red's turn.
    assert_eq!(game.town_offer(&rules, BLUE, T0), None);
    assert_eq!(game.town_offer(&rules, RED, T1), None);
    assert_eq!(game.town_offer(&rules, RED, TownId::new(2)), None);
    assert!(
        game.town_offer(&rules, BLUE, T1)
            .is_some_and(|blue| blue.payable.contains(building(SILO)))
    );
    assert_eq!(listed(&game, &rules, BLUE), []);
}

// covers: TWN-BLD-004
#[test]
fn a_town_built_in_today_lists_no_build_until_the_day_s_start() {
    let rules = priced();
    let mut game = GameState::new(&rules, &setup(&rules), 7).unwrap();
    run(&rules, &mut game, RED, build(T0, TOWN_HALL)).unwrap();
    let marked = game.town_offer(&rules, RED, T0).unwrap();
    assert_eq!(marked.buildable, BuildingSet::EMPTY);
    assert_eq!(marked.payable, BuildingSet::EMPTY);
    assert!(marked.possible.contains(building(CITY_HALL)));
    assert_eq!(listed(&game, &rules, RED), [GameCommand::EndTurn]);
    // Blue's town keeps its own day.
    end_turn(&rules, &mut game, RED);
    let blue = game.town_offer(&rules, BLUE, T1).unwrap();
    assert!(!blue.payable.is_empty());
    assert_eq!(listed(&game, &rules, BLUE), expected(&[(T1, blue)]));
    end_turn(&rules, &mut game, BLUE);
    // Day 2: the mark cleared, the City Hall's requirement stands.
    let offer = game.town_offer(&rules, RED, T0).unwrap();
    assert!(offer.buildable.contains(building(CITY_HALL)));
    assert_eq!(listed(&game, &rules, RED), expected(&[(T0, offer)]));
}

// covers: TWN-BLD-001, TWN-BLD-002
#[test]
fn an_owner_s_towns_list_their_builds_in_town_order_with_one_capitol() {
    let rules = priced();
    let mut setup = setup(&rules);
    setup.map.objects[1].owner = Some(RED);
    setup.towns[1].buildings = BuildingsSetup::Custom {
        built: SlotSet::EMPTY.with(CAPITOL_SLOT),
        forbidden: SlotSet::EMPTY,
    };
    let game = GameState::new(&rules, &setup, 7).unwrap();
    let first = game.town_offer(&rules, RED, T0).unwrap();
    let second = game.town_offer(&rules, RED, T1).unwrap();
    // Town 1's Capitol stands: town 0's is neither offered nor possible while it does.
    assert!(!first.possible.contains(building(CAPITOL)));
    assert!(!first.buildable.contains(building(CAPITOL)));
    assert!(first.payable.contains(building(FORT)));
    assert!(!second.payable.is_empty());
    assert_eq!(
        listed(&game, &rules, RED),
        expected(&[(T0, first), (T1, second)])
    );
}
