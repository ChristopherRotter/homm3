//! Tests of the town context's commands through the game (E09 T18b1, the town epic's design note
//! §29, §30): the owner's build in its turn and its price, the day's mark and the day's start, each
//! refusal changing nothing, the build's event and who sees it.

use alloc::string::ToString;
use alloc::vec::Vec;

use h3_content::ResourceId;
use h3_contracts::{PlayerId, TownId};
use h3_towns::{BuildingId, BuildingsSetup, SlotSet, TownCommand, TownCommandError, TownFaction};

use super::{CAPITOL_SLOT, faction, rules_of, town_pack, towns_core, towns_setup};
use crate::tests::key;
use crate::{
    Actor, GameCommand, GameCommandError, GameEvent, GameRuleset, GameSetup, GameState, Viewer,
    apply, project,
};

const RED: PlayerId = PlayerId::new(0);
const BLUE: PlayerId = PlayerId::new(1);
const T0: TownId = TownId::new(0);
const T1: TownId = TownId::new(1);
const GOLD: ResourceId = ResourceId::from_raw(0);
/// [`priced`]'s Town Hall (`b1`, 1 500 gold), its City Hall (`b2`, it requires the Town Hall),
/// its Capitol (`b3`), its Fort (`b4`, free), its Citadel (`b5`, 2 500 gold, more than a human's
/// 2 000 at the start) and its silo (`b10`, free; blue's standard town has its Fort).
const TOWN_HALL: usize = 1;
const CITY_HALL: usize = 2;
const CAPITOL: usize = 3;
const FORT: usize = 4;
const CITADEL: usize = 5;
const SILO: usize = 10;

/// The game's rules with [`town_pack`]'s record priced and with one requirement (the rest free
/// and without one).
fn priced() -> GameRuleset {
    let mut pack = town_pack(false);
    let buildings = &mut pack.towns[0].buildings;
    buildings[TOWN_HALL].cost = [(key("t:gold"), 1500)].to_vec();
    buildings[CITY_HALL].requires = ["b1".into()].to_vec();
    buildings[CITADEL].cost = [(key("t:gold"), 2500)].to_vec();
    rules_of(towns_core(false), &pack, false, 4)
}

/// [`towns_setup`] with red's town 0 holding its Village Hall alone (a custom set) and blue's
/// town 1 of `t:town`.
fn setup(rules: &GameRuleset) -> GameSetup {
    let mut setup = towns_setup(rules);
    setup.towns[0].buildings = BuildingsSetup::Custom {
        built: SlotSet::EMPTY,
        forbidden: SlotSet::EMPTY,
    };
    setup.towns[1] = super::town(TownFaction::Fixed(faction(rules, "t:town")));
    setup
}

fn building(index: usize) -> BuildingId {
    BuildingId::from_index(index).unwrap()
}

fn build(town: TownId, index: usize) -> GameCommand {
    GameCommand::Town(TownCommand::Build {
        town,
        building: building(index),
    })
}

/// `player`'s `command`: its events, or its error with the state and the events unchanged.
fn run(
    rules: &GameRuleset,
    game: &mut GameState,
    player: PlayerId,
    command: GameCommand,
) -> Result<Vec<GameEvent>, GameCommandError> {
    let before = (game.state_hash(), game.clone());
    let mut events = Vec::new();
    let result = apply(game, rules, Actor::Player(player), command, &mut events);
    if result.is_err() {
        assert!(events.is_empty(), "{events:?}");
        assert_eq!(game.state_hash(), before.0);
        assert_eq!(game.towns(), before.1.towns());
    }
    result.map(|()| events)
}

fn gold(game: &GameState, player: PlayerId) -> u32 {
    game.treasury(player).unwrap()[usize::from(GOLD.raw())]
}

// covers: TWN-BLD-001, TWN-BLD-005
#[test]
fn an_owner_builds_in_its_turn_and_pays_the_price() {
    let rules = priced();
    let mut game = GameState::new(&rules, &setup(&rules), 7).unwrap();
    assert_eq!(gold(&game, RED), 2000);
    let events = run(&rules, &mut game, RED, build(T0, TOWN_HALL)).unwrap();
    let built = GameEvent::TownBuilt {
        player: RED,
        town: T0,
        building: building(TOWN_HALL),
        event: None,
    };
    let paid = GameEvent::ResourcePaid {
        player: RED,
        resource: GOLD,
        amount: 1500,
    };
    assert_eq!(events, [paid, built]);
    assert_eq!(gold(&game, RED), 500);
    let town = game.towns().get(T0).unwrap();
    assert!(town.built().contains(building(TOWN_HALL)));
    assert!(town.built_today());
    // The build is its owner's to see (design note §31).
    assert_eq!(project(&built, Viewer::Player(RED)), Some(built));
    assert_eq!(project(&built, Viewer::Player(BLUE)), None);
    // A free building costs nothing and pays nothing: blue's silo in its turn.
    end_turn(&rules, &mut game, RED);
    let events = run(&rules, &mut game, BLUE, build(T1, SILO)).unwrap();
    assert_eq!(
        events,
        [GameEvent::TownBuilt {
            player: BLUE,
            town: T1,
            building: building(SILO),
            event: None,
        }]
    );
    assert_eq!(gold(&game, BLUE), 2000);
}

fn end_turn(rules: &GameRuleset, game: &mut GameState, player: PlayerId) {
    apply(
        game,
        rules,
        Actor::Player(player),
        GameCommand::EndTurn,
        &mut Vec::new(),
    )
    .unwrap();
}

// covers: TWN-BLD-004
#[test]
fn a_town_is_built_in_once_a_day_and_the_day_s_start_clears_the_mark() {
    let rules = priced();
    let mut game = GameState::new(&rules, &setup(&rules), 7).unwrap();
    run(&rules, &mut game, RED, build(T0, TOWN_HALL)).unwrap();
    assert_eq!(
        run(&rules, &mut game, RED, build(T0, FORT)),
        Err(GameCommandError::Town(TownCommandError::BuiltToday))
    );
    // Blue's town keeps its own day: no mark of red's crosses to it.
    end_turn(&rules, &mut game, RED);
    run(&rules, &mut game, BLUE, build(T1, SILO)).unwrap();
    assert!(game.towns().get(T0).unwrap().built_today());
    end_turn(&rules, &mut game, BLUE);
    // Day 2: every mark cleared at the day's start, before red's turn.
    assert_eq!(game.day(), 2);
    assert!(!game.towns().get(T0).unwrap().built_today());
    assert!(!game.towns().get(T1).unwrap().built_today());
    run(&rules, &mut game, RED, build(T0, FORT)).unwrap();
    // The requirement now stands: the City Hall after the Town Hall.
    end_turn(&rules, &mut game, RED);
    end_turn(&rules, &mut game, BLUE);
    run(&rules, &mut game, RED, build(T0, CITY_HALL)).unwrap();
}

// covers: TWN-BLD-001, TWN-BLD-002
#[test]
fn every_refused_build_changes_nothing() {
    let rules = priced();
    let mut game = GameState::new(&rules, &setup(&rules), 7).unwrap();
    let refusals = [
        (
            RED,
            build(TownId::new(2), FORT),
            TownCommandError::UnknownTown,
        ),
        (RED, build(T1, SILO), TownCommandError::NotOwner),
        (RED, build(T0, 0), TownCommandError::Built),
        (RED, build(T0, 40), TownCommandError::UnknownBuilding),
        (RED, build(T0, CITY_HALL), TownCommandError::Requirements),
    ];
    for (player, command, error) in refusals {
        let refused = run(&rules, &mut game, player, command.clone());
        assert_eq!(refused, Err(GameCommandError::Town(error)), "{command:?}");
    }
    // The treasury after the hall's tests: 2 500 gold against 2 000.
    assert_eq!(
        run(&rules, &mut game, RED, build(T0, CITADEL)),
        Err(GameCommandError::CannotPay)
    );
    // The turn before anything: blue's own town in red's turn.
    assert_eq!(
        run(&rules, &mut game, BLUE, build(T1, SILO)),
        Err(GameCommandError::NotYourTurn(BLUE))
    );
    // The system actor commands no town.
    let mut events = Vec::new();
    let system = apply(
        &mut game,
        &rules,
        Actor::System,
        build(T0, FORT),
        &mut events,
    );
    assert_eq!(system, Err(GameCommandError::NotForSystem));
    assert_eq!(
        GameCommandError::Town(TownCommandError::NotOwner).to_string(),
        "the town is not the player's"
    );
}

// covers: TWN-BLD-002
#[test]
fn a_capitol_is_refused_while_its_player_holds_another() {
    let rules = priced();
    let mut setup = setup(&rules);
    setup.map.objects[1].owner = Some(RED);
    setup.towns[1].buildings = BuildingsSetup::Custom {
        built: SlotSet::EMPTY.with(CAPITOL_SLOT),
        forbidden: SlotSet::EMPTY,
    };
    let game = GameState::new(&rules, &setup, 7).unwrap();
    assert!(game.towns().capitol(T1, rules.towns()));
    let mut held = game.clone();
    assert_eq!(
        run(&rules, &mut held, RED, build(T0, CAPITOL)),
        Err(GameCommandError::Town(TownCommandError::Capitol))
    );
    // Its own Capitol is no other: with town 1 blue's again, red builds its Capitol.
    setup.map.objects[1].owner = Some(BLUE);
    let mut free = GameState::new(&rules, &setup, 7).unwrap();
    run(&rules, &mut free, RED, build(T0, CAPITOL)).unwrap();
    assert!(free.towns().capitol(T0, rules.towns()));
}
