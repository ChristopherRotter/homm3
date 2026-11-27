//! Tests of a boat built at a town's Shipyard (E09 T59a, design note `E09-design.md` §111;
//! TWN-SHY-001, TWN-SET-011): `TownCommand::BuildBoat` launches an empty boat of the part's type on
//! the town's spot for its builder, then takes the part's price, without a hero; it is refused
//! while an empty boat or a hero stands on the spot (`TWN-QUIRK-19`); each refusal changes
//! nothing; the legal commands list it where `apply` accepts it. On the town tests' game (red's T0
//! on (1, 1), its Shipyard (map slot 6) built, and blue's T1 on (3, 0)) with the sea on (0, 3) and
//! (1, 3): T0's spot is (0, 3), two rows below its entrance, one column left; the part's boat
//! 1 000 gold, type 1.

use alloc::vec::Vec;

use h3_adventure::{AdventureCommand, AdventureEventKind, Pos};
use h3_content::{ResourceId, TerrainId};
use h3_contracts::TownId;
use h3_towns::{TownCommand, TownCommandError};

use super::command::{BLUE, RED, T0, T1, end_turn, run};
use super::learn::{RED_HERO, game, setup, town_command};
use super::{SHIPYARD_SLOT, towns_rules};
use crate::{Actor, GameCommand, GameCommandError, GameEvent, GameRuleset, GameState};

const GOLD: ResourceId = ResourceId::from_raw(0);
/// T0's spot.
const SPOT: Pos = Pos::new(0, 3, 0);

fn boat(town: TownId) -> GameCommand {
    town_command(TownCommand::BuildBoat { town })
}

/// The game with both towns built by `slots` and the sea on (0, 3) and (1, 3); red holds 10 000
/// gold.
fn wet(rules: &GameRuleset, slots: &[u8]) -> GameState {
    let mut setup = setup(rules, slots);
    for (x, y) in [(0, 3), (1, 3)] {
        setup.map.tiles[y * 4 + x].terrain = TerrainId::from_raw(2);
    }
    game(rules, &setup)
}

fn listed(rules: &GameRuleset, game: &GameState) -> bool {
    crate::legal_commands(game, rules, Actor::Player(RED)).contains(&boat(T0))
}

fn refused(rules: &GameRuleset, game: &mut GameState, town: TownId, error: GameCommandError) {
    assert_eq!(run(rules, game, RED, boat(town)), Err(error));
}

/// The kinds of the adventure's events of `events` and the other events, in order.
fn kinds(events: &[GameEvent]) -> Vec<Result<AdventureEventKind, GameEvent>> {
    events
        .iter()
        .map(|event| match event {
            GameEvent::Adventure(event) => Ok(event.kind),
            other => Err(other.clone()),
        })
        .collect()
}

// covers: TWN-SHY-001, TWN-SET-011
#[test]
fn a_boat_is_launched_on_the_towns_spot_then_paid() {
    let rules = towns_rules();
    let mut game = wet(&rules, &[SHIPYARD_SLOT]);
    assert_eq!(game.adventure().free_town_boat_spot(T0), Some(SPOT));
    assert!(listed(&rules, &game));
    let events = run(&rules, &mut game, RED, boat(T0)).unwrap();
    assert_eq!(
        kinds(&events),
        [
            Ok(AdventureEventKind::BoatBuilt {
                tile: SPOT,
                boat_type: 1
            }),
            Err(GameEvent::ResourcePaid {
                player: RED,
                resource: GOLD,
                amount: 1000,
            }),
        ]
    );
    assert!(game.adventure().is_empty_boat(SPOT));
    assert_eq!(game.treasury(RED).unwrap()[GOLD.index()], 9_000);
    // `TWN-QUIRK-19`: a second boat is refused while the first lies on the spot, before the
    // treasury.
    assert!(!listed(&rules, &game));
    let taken = GameCommandError::Town(TownCommandError::BoatSpotTaken);
    let mut poor = game.clone();
    poor.set_resource(RED, GOLD, 0);
    refused(&rules, &mut poor, T0, taken);
    // Red's hero boards it: the spot still taken, by the hero in its boat.
    let path = [Pos::new(1, 2, 0), SPOT];
    let board = GameCommand::Adventure(AdventureCommand::MoveHero {
        hero: RED_HERO,
        path: path.to_vec(),
    });
    run(&rules, &mut game, RED, board).unwrap();
    assert!(!game.adventure().is_empty_boat(SPOT));
    assert!(!listed(&rules, &game));
    refused(&rules, &mut game, T0, taken);
    // It sails off the next day: the spot frees, a boat is built again.
    end_turn(&rules, &mut game, RED);
    end_turn(&rules, &mut game, BLUE);
    let sail = GameCommand::Adventure(AdventureCommand::MoveHero {
        hero: RED_HERO,
        path: [Pos::new(1, 3, 0)].to_vec(),
    });
    run(&rules, &mut game, RED, sail).unwrap();
    assert!(listed(&rules, &game));
    run(&rules, &mut game, RED, boat(T0)).unwrap();
    assert!(game.adventure().is_empty_boat(SPOT));
}

// covers: TWN-SHY-001
#[test]
fn each_refusal_changes_nothing_in_its_order() {
    let rules = towns_rules();
    let mut game = wet(&rules, &[SHIPYARD_SLOT]);
    refused(
        &rules,
        &mut game,
        TownId::new(9),
        GameCommandError::Town(TownCommandError::UnknownTown),
    );
    refused(
        &rules,
        &mut game,
        T1,
        GameCommandError::Town(TownCommandError::NotOwner),
    );
    // The treasury below the price.
    let mut poor = game.clone();
    poor.set_resource(RED, GOLD, 999);
    assert!(!listed(&rules, &poor));
    refused(&rules, &mut poor, T0, GameCommandError::CannotPay);
    // No Shipyard: not standing, whatever the treasury and the spot.
    let mut dry = wet(&rules, &[]);
    assert!(!listed(&rules, &dry));
    refused(
        &rules,
        &mut dry,
        T0,
        GameCommandError::Town(TownCommandError::NotStanding),
    );
}
