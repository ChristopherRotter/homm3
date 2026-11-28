//! Tests of a war machine bought at a town (E09 T59a, design note `E09-design.md` §110;
//! TWN-SMT-001): `TownCommand::BuyWarMachine` pays the price of the machine's creature, every
//! resource as itself (`TWN-QUIRK-22`), and gives the town's hero (the visitor; with
//! `TWN-QUIRK-15`'s flag off the garrison hero too) the machine's artifact; each refusal changes
//! nothing; the legal commands list each purchase `apply` accepts, in the record's order. On the
//! town tests' game (red's T0 on (1, 1) and blue's T1 on (3, 0), each built by its setup's slots
//! only) with wood beside the gold, the cart at 1 000 gold and 2 wood, and `t:town`'s Blacksmith
//! (map slot 16) and a second building selling the cart (slot 22, as the Stronghold's Ballista
//! Yard beside its Blacksmith).

use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{ObjectKindDef, ObjectTypeDef, Pos, Visit};
use h3_content::{ArtifactTypeId, ProfileDef, ResourceDef, ResourceId, Ruleset};
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_heroes::HeroChange;
use h3_towns::{
    BuildingDef, BuildingId, BuildingRoles, TownCommand, TownCommandError, TownHeroChange,
};

use super::command::{RED, T0, T1, run};
use super::learn::{RED_HERO, building, game, setup, town_command};
use super::{rules_edited, town_pack, towns_core_pack};
use crate::tests::key;
use crate::{
    Actor, GameCommand, GameCommandError, GameEvent, GameRuleset, GameState, HeroArtifactsSetup,
};

const GOLD: ResourceId = ResourceId::from_raw(0);
const WOOD: ResourceId = ResourceId::from_raw(1);
/// The map slots of the Blacksmith and the second building selling the cart.
const SMITH_SLOT: u8 = 16;
const YARD_SLOT: u8 = 22;

/// The tests' rules under `profile`: the test pack's `t:faithful` (no quirk flag on, so the
/// garrison hero buys) or `t:visitor` (`TWN-QUIRK-15`'s flag on: the visitor alone).
fn rules_in(profile: &str) -> GameRuleset {
    let mut core = towns_core_pack(false);
    core.resources.push(ResourceDef {
        key: key("t:wood"),
        name: "Wood".into(),
    });
    let cart = core
        .creatures
        .iter_mut()
        .find(|c| c.key == key("t:cart"))
        .unwrap();
    cart.info.cost = vec![(key("t:gold"), 1000), (key("t:wood"), 2)];
    core.profiles.push(ProfileDef {
        key: key("t:visitor"),
        quirks: vec![("quirk.town.visitor_only_trades".to_string(), true)],
    });
    let core = Ruleset::compile(&[core], profile).unwrap();
    let mut towns = town_pack(false);
    let building = |k: &str, slot| BuildingDef {
        key: k.to_string(),
        name: "Building".into(),
        slot: Some(slot),
        supersedes: vec![],
        forbidden_with: None,
        auto_with: vec![],
        requires: vec![],
        cost: vec![],
        roles: BuildingRoles::default(),
        war_machine: Some(key("t:cart_art")),
    };
    towns.towns[0]
        .buildings
        .extend([building("smith", SMITH_SLOT), building("yard", YARD_SLOT)]);
    rules_edited(core, &towns, false, 4, |adventure| {
        // The abandoned mine's check: a mine of every core resource.
        adventure.object_types.push(ObjectTypeDef {
            key: key("t:wood_mine"),
            name: "Mine".into(),
            visit: Visit::Enter,
            from_above: true,
            kind_mark: false,
            kind: ObjectKindDef::Mine {
                resource: key("t:wood"),
                daily: 2,
                battle_terrain: None,
            },
        });
    })
}

fn rules() -> GameRuleset {
    rules_in("t:faithful")
}

fn cart(rules: &GameRuleset) -> ArtifactTypeId {
    rules.core().artifact_by_key("t:cart_art").unwrap()
}

fn machine(town: TownId, building: BuildingId) -> GameCommand {
    town_command(TownCommand::BuyWarMachine { town, building })
}

/// The game with both towns built by `slots`, red's hero on T0's entrance wearing the cart when
/// `cart`, else nothing; red holds 10 000 gold and 5 wood.
fn built(rules: &GameRuleset, slots: &[u8], cart: bool) -> GameState {
    let mut setup = setup(rules, slots);
    let worn: Vec<_> = if cart {
        vec![self::cart(rules).into()]
    } else {
        vec![]
    };
    setup.heroes[0].details.artifacts = Some(HeroArtifactsSetup {
        worn: h3_heroes::wearing(rules.core(), &worn),
        backpack: vec![],
    });
    let mut game = game(rules, &setup);
    game.set_resource(RED, WOOD, 5);
    game
}

fn holds_cart(rules: &GameRuleset, game: &GameState, hero: HeroId) -> bool {
    let cart = cart(rules);
    game.hero(hero)
        .unwrap()
        .worn
        .iter()
        .flatten()
        .any(|item| item.artifact == cart)
}

/// Red's listed purchases, in the listing's order.
fn listed(rules: &GameRuleset, game: &GameState) -> Vec<GameCommand> {
    crate::legal_commands(game, rules, Actor::Player(RED))
        .into_iter()
        .filter(|c| matches!(c, GameCommand::Town(TownCommand::BuyWarMachine { .. })))
        .collect()
}

/// `player`'s purchase at `town`'s `building` refused with `error` (`run` checks the state
/// unchanged).
fn refused(
    rules: &GameRuleset,
    game: &mut GameState,
    (player, town, building): (PlayerId, TownId, BuildingId),
    error: TownCommandError,
) {
    let command = machine(town, building);
    assert_eq!(
        run(rules, game, player, command),
        Err(GameCommandError::Town(error))
    );
}

// covers: TWN-SMT-001
#[test]
fn a_purchase_gives_the_visitor_the_machine_for_its_creatures_price() {
    let rules = rules();
    let (smith, yard) = (building(&rules, "smith"), building(&rules, "yard"));
    let mut game = built(&rules, &[SMITH_SLOT, YARD_SLOT], false);
    // Both buildings listed, in the record's order.
    assert_eq!(
        listed(&rules, &game),
        [machine(T0, smith), machine(T0, yard)]
    );
    let events = run(&rules, &mut game, RED, machine(T0, smith)).unwrap();
    // `TWN-QUIRK-22`: each resource of the cart's cost charged as itself, then the cart given.
    assert_eq!(
        events,
        [
            GameEvent::ResourcePaid {
                player: RED,
                resource: GOLD,
                amount: 1000,
            },
            GameEvent::ResourcePaid {
                player: RED,
                resource: WOOD,
                amount: 2,
            },
            GameEvent::HeroChanged {
                player: RED,
                hero: RED_HERO,
                change: HeroChange::AddArtifact(cart(&rules).into()),
            },
        ]
    );
    assert!(holds_cart(&rules, &game, RED_HERO));
    let treasury = game.treasury(RED).unwrap();
    assert_eq!((treasury[GOLD.index()], treasury[WOOD.index()]), (9_000, 3));
    // Once held, neither building sells it again: none listed.
    assert!(listed(&rules, &game).is_empty());
    for building in [smith, yard] {
        refused(
            &rules,
            &mut game,
            (RED, T0, building),
            TownCommandError::HasWarMachine,
        );
    }
}

// covers: TWN-SMT-001
#[test]
fn each_refusal_changes_nothing_in_its_order() {
    let rules = rules();
    let smith = building(&rules, "smith");
    let mut game = built(&rules, &[SMITH_SLOT], false);
    let unknown = TownId::new(9);
    refused(
        &rules,
        &mut game,
        (RED, unknown, smith),
        TownCommandError::UnknownTown,
    );
    refused(
        &rules,
        &mut game,
        (RED, T1, smith),
        TownCommandError::NotOwner,
    );
    let none = BuildingId::from_index(40).unwrap();
    refused(
        &rules,
        &mut game,
        (RED, T0, none),
        TownCommandError::UnknownBuilding,
    );
    let hall = BuildingId::from_index(0).unwrap();
    refused(
        &rules,
        &mut game,
        (RED, T0, hall),
        TownCommandError::NoWarMachine,
    );
    let yard = building(&rules, "yard");
    refused(
        &rules,
        &mut game,
        (RED, T0, yard),
        TownCommandError::NotStanding,
    );
    // The treasury: 1 wood of the 2.
    let mut poor = game.clone();
    poor.set_resource(RED, WOOD, 1);
    assert!(listed(&rules, &poor).is_empty());
    assert_eq!(
        run(&rules, &mut poor, RED, machine(T0, smith)),
        Err(GameCommandError::CannotPay)
    );
    // A hero that wears the cart: refused before the treasury, not listed.
    let mut armed = built(&rules, &[SMITH_SLOT], true);
    armed.set_resource(RED, WOOD, 0);
    assert!(listed(&rules, &armed).is_empty());
    refused(
        &rules,
        &mut armed,
        (RED, T0, smith),
        TownCommandError::HasWarMachine,
    );
    // No hero in the town: the hero leaves the entrance.
    let away = Pos::new(1, 2, 0);
    let step = GameCommand::Adventure(h3_adventure::AdventureCommand::MoveHero {
        hero: RED_HERO,
        path: vec![away],
    });
    run(&rules, &mut game, RED, step).unwrap();
    assert!(listed(&rules, &game).is_empty());
    refused(
        &rules,
        &mut game,
        (RED, T0, smith),
        TownCommandError::NoVisitor,
    );
}

// covers: TWN-SMT-001
#[test]
fn the_garrison_hero_buys_only_with_the_visitor_only_flag_off() {
    // `TWN-QUIRK-15`'s flag off (`strict-fixed`; the test pack's `t:faithful`): the garrison
    // hero alone buys.
    let rules = rules();
    let smith = building(&rules, "smith");
    let mut game = built(&rules, &[SMITH_SLOT], false);
    let enter = town_command(TownCommand::Heroes {
        town: T0,
        change: TownHeroChange::Enter,
    });
    run(&rules, &mut game, RED, enter.clone()).unwrap();
    assert!(game.adventure().town_visitor(T0).is_none());
    assert_eq!(listed(&rules, &game), [machine(T0, smith)]);
    run(&rules, &mut game, RED, machine(T0, smith)).unwrap();
    assert!(holds_cart(&rules, &game, RED_HERO));
    // On (`faithful`): the visitor alone (the original's GENRLTXT 274).
    let rules = rules_in("t:visitor");
    let mut game = built(&rules, &[SMITH_SLOT], false);
    run(&rules, &mut game, RED, enter).unwrap();
    assert!(listed(&rules, &game).is_empty());
    refused(
        &rules,
        &mut game,
        (RED, T0, smith),
        TownCommandError::NoVisitor,
    );
}
