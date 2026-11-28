//! Tests of a town's Skeleton Transformer (E09 T59b2, design note `E09-design.md` §113;
//! TWN-TRF-001): `TownCommand::Transform` changes one whole stack of the town's visitor's army or
//! of its upper bar's (the garrison hero's, else the town's garrison) into the part's target of
//! its creature, its count kept, free, nothing merged; each refusal changes nothing; the legal
//! commands list each stack that changes. On the town tests' game (red's T0 on (1, 1) and blue's
//! T1 on (3, 0), each built by its setup's slots only) with a Transformer (map slot 23) making the
//! elf an ent (the part's "dragon") and every other creature an imp (its "Skeleton").

use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{AdventureCommand, ArmyRef, ArmySlot, Pos};
use h3_content::{CreatureTypeId, ResourceId};
use h3_contracts::{ArmyStack, PlayerId, TownId};
use h3_towns::{
    BuildingDef, BuildingRoles, GarrisonCreature, GarrisonStack, TownArmy, TownCommand,
    TownCommandError, TownHeroChange,
};

use super::command::{RED, T0, T1, run};
use super::learn::{RED_HERO, game, setup, town_command};
use super::{rules_edited, town_pack, towns_core_pack};
use crate::tests::key;
use crate::{Actor, GameCommand, GameCommandError, GameEvent, GameRuleset, GameState, HeroArmy};

const GOLD: ResourceId = ResourceId::from_raw(0);
/// The Transformer's map slot.
const TRANSFORMER_SLOT: u8 = 23;

/// The tests' rules: the town tests' core with the elf and the ent, the town part with a
/// Transformer, its table the imp for every creature but the elf, the ent for the elf; seven army
/// slots.
fn rules() -> GameRuleset {
    let core = h3_content::Ruleset::compile(&[towns_core_pack(true)], "t:faithful").unwrap();
    let mut towns = town_pack(true);
    towns.towns[0].buildings.push(BuildingDef {
        key: "transformer".into(),
        name: "Skeleton Transformer".into(),
        slot: Some(TRANSFORMER_SLOT),
        supersedes: vec![],
        forbidden_with: None,
        auto_with: vec![],
        requires: vec![],
        cost: vec![],
        roles: BuildingRoles {
            transformer: true,
            ..BuildingRoles::default()
        },
        war_machine: None,
    });
    let params = towns.params.as_mut().unwrap();
    params.transformer_into = key("t:imp");
    params.transformer_dragons_into = key("t:ent");
    towns.transformer_dragons = vec![key("t:elf")];
    // The original's seven army slots (the tests' adventure part has two).
    rules_edited(core, &towns, true, 4, |adventure| {
        adventure.params.as_mut().unwrap().army_slots = core::num::NonZeroU8::new(7).unwrap();
    })
}

fn creature(rules: &GameRuleset, k: &str) -> CreatureTypeId {
    rules.core().creature_by_key(k).unwrap()
}

fn stack(slot: u8, creature: CreatureTypeId, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature,
        count,
    }
}

fn transform(town: TownId, army: TownArmy, slot: u8) -> GameCommand {
    town_command(TownCommand::Transform { town, army, slot })
}

/// The game with both towns built by `slots`, red's hero on T0's entrance with 4 gogs in slot 0,
/// 9 imps in slot 1, 2 elves in slot 3 and an ent in slot 6, T0's garrison 3 gogs in slot 2; red
/// holds 10 000 gold.
fn built(rules: &GameRuleset, slots: &[u8]) -> GameState {
    let mut setup = setup(rules, slots);
    let [imp, gog, elf, ent] = ["t:imp", "t:gog", "t:elf", "t:ent"].map(|k| creature(rules, k));
    setup.heroes[0].army = HeroArmy::Custom(vec![
        stack(0, gog, 4),
        stack(1, imp, 9),
        stack(3, elf, 2),
        stack(6, ent, 1),
    ]);
    setup.towns[0].garrison = Some(vec![GarrisonStack {
        slot: 2,
        creature: GarrisonCreature::Creature(gog),
        count: 3,
    }]);
    game(rules, &setup)
}

/// Red's listed transformations, in the listing's order.
fn listed(rules: &GameRuleset, game: &GameState) -> Vec<GameCommand> {
    crate::legal_commands(game, rules, Actor::Player(RED))
        .into_iter()
        .filter(|c| matches!(c, GameCommand::Town(TownCommand::Transform { .. })))
        .collect()
}

/// `player`'s transformation at `town` refused with `error` (`run` checks the state unchanged).
fn refused(
    rules: &GameRuleset,
    game: &mut GameState,
    (player, command): (PlayerId, GameCommand),
    error: TownCommandError,
) {
    assert_eq!(
        run(rules, game, player, command),
        Err(GameCommandError::Town(error))
    );
}

/// The army of red's hero, by slot.
fn army(game: &GameState) -> Vec<ArmyStack> {
    let adventure = game.adventure();
    let hero = adventure
        .hero(RED_HERO)
        .or_else(|| adventure.garrisoned(RED_HERO).map(|(_, on)| on))
        .unwrap();
    hero.army.clone()
}

/// The stack-changed event of the slot `slot` of `army`, red's, now holding `stack`.
fn changed(army: ArmyRef, slot: u8, stack: ArmyStack) -> GameEvent {
    GameEvent::StackChanged {
        player: RED,
        at: ArmySlot { army, slot },
        stack: Some(stack),
    }
}

// covers: TWN-TRF-001
#[test]
fn the_visitor_transforms_a_whole_stack_into_its_target_for_free() {
    let rules = rules();
    let (imp, ent) = (creature(&rules, "t:imp"), creature(&rules, "t:ent"));
    let mut game = built(&rules, &[TRANSFORMER_SLOT]);
    // Each stack that changes: the visitor's gogs, elves and ent, then the garrison's gogs; the
    // imps are their own target.
    assert_eq!(
        listed(&rules, &game),
        [
            transform(T0, TownArmy::Visitor, 0),
            transform(T0, TownArmy::Visitor, 3),
            transform(T0, TownArmy::Visitor, 6),
            transform(T0, TownArmy::Garrison, 2),
        ]
    );
    let gold = game.treasury(RED).unwrap()[GOLD.index()];
    let before = game.state_hash();
    // The gogs become imps, the count kept, beside the imps of slot 1 (nothing merged), in one
    // command (`TWN-QUIRK-23`: no holding army to lose).
    let events = run(&rules, &mut game, RED, transform(T0, TownArmy::Visitor, 0)).unwrap();
    assert_eq!(
        events,
        [changed(ArmyRef::Hero(RED_HERO), 0, stack(0, imp, 4))]
    );
    assert_ne!(game.state_hash(), before);
    // The elves become ents (the part's dragons), the ent an imp (the undead too).
    run(&rules, &mut game, RED, transform(T0, TownArmy::Visitor, 3)).unwrap();
    run(&rules, &mut game, RED, transform(T0, TownArmy::Visitor, 6)).unwrap();
    assert_eq!(
        army(&game),
        [
            stack(0, imp, 4),
            stack(1, imp, 9),
            stack(3, ent, 2),
            stack(6, imp, 1)
        ]
    );
    // Free, any number a turn: the ents of slot 3 listed again.
    assert_eq!(game.treasury(RED).unwrap()[GOLD.index()], gold);
    assert_eq!(
        listed(&rules, &game),
        [
            transform(T0, TownArmy::Visitor, 3),
            transform(T0, TownArmy::Garrison, 2),
        ]
    );
    // The garrison's gogs, with the visitor there: the town's own garrison.
    let events = run(&rules, &mut game, RED, transform(T0, TownArmy::Garrison, 2)).unwrap();
    assert_eq!(
        events[0],
        changed(ArmyRef::Garrison(T0), 2, stack(2, imp, 3))
    );
    let index = game.adventure().map().town_object(T0).unwrap();
    assert_eq!(
        game.adventure().town_garrison(index).unwrap(),
        [stack(2, imp, 3)]
    );
    assert_eq!(army(&game)[0], stack(0, imp, 4));
}

// covers: TWN-TRF-001
#[test]
fn each_refusal_changes_nothing_in_its_order() {
    let rules = rules();
    let mut game = built(&rules, &[TRANSFORMER_SLOT]);
    let visitor = |slot| transform(T0, TownArmy::Visitor, slot);
    refused(
        &rules,
        &mut game,
        (RED, transform(TownId::new(9), TownArmy::Visitor, 0)),
        TownCommandError::UnknownTown,
    );
    refused(
        &rules,
        &mut game,
        (RED, transform(T1, TownArmy::Garrison, 2)),
        TownCommandError::NotOwner,
    );
    let mut bare = built(&rules, &[]);
    assert!(listed(&rules, &bare).is_empty());
    refused(
        &rules,
        &mut bare,
        (RED, visitor(0)),
        TownCommandError::NoTransformer,
    );
    // Past the army's slots, an empty slot, the imps (their own target).
    for (slot, error) in [
        (7, TownCommandError::UnknownSlot),
        (2, TownCommandError::EmptySlot),
        (1, TownCommandError::NothingToTransform),
    ] {
        refused(&rules, &mut game, (RED, visitor(slot)), error);
    }
    refused(
        &rules,
        &mut game,
        (RED, transform(T0, TownArmy::Garrison, 0)),
        TownCommandError::EmptySlot,
    );
    // No visitor: the hero leaves the entrance; the garrison's army needs none.
    let step = GameCommand::Adventure(AdventureCommand::MoveHero {
        hero: RED_HERO,
        path: vec![Pos::new(1, 2, 0)],
    });
    run(&rules, &mut game, RED, step).unwrap();
    assert_eq!(
        listed(&rules, &game),
        [transform(T0, TownArmy::Garrison, 2)]
    );
    refused(
        &rules,
        &mut game,
        (RED, visitor(0)),
        TownCommandError::NoVisitor,
    );
    run(&rules, &mut game, RED, transform(T0, TownArmy::Garrison, 2)).unwrap();
    // The garrison's last stack transformed, then nothing more to list.
    assert!(listed(&rules, &game).is_empty());
}

// covers: TWN-TRF-001
#[test]
fn the_upper_bar_is_the_garrison_heros_army_where_one_stands() {
    let rules = rules();
    let (imp, gog) = (creature(&rules, "t:imp"), creature(&rules, "t:gog"));
    let mut game = built(&rules, &[TRANSFORMER_SLOT]);
    // The visitor enters the garrison, the garrison's gogs joining its army (TWN-ROW-002).
    let enter = town_command(TownCommand::Heroes {
        town: T0,
        change: TownHeroChange::Enter,
    });
    run(&rules, &mut game, RED, enter).unwrap();
    assert!(game.adventure().town_visitor(T0).is_none());
    let joined = army(&game);
    let at = joined.iter().find(|s| s.creature == gog).unwrap().slot;
    let listed = listed(&rules, &game);
    assert!(
        listed.contains(&transform(T0, TownArmy::Garrison, at)),
        "{listed:?}"
    );
    assert!(
        listed.iter().all(|c| matches!(
            c,
            GameCommand::Town(TownCommand::Transform {
                army: TownArmy::Garrison,
                ..
            })
        )),
        "{listed:?}"
    );
    let events = run(
        &rules,
        &mut game,
        RED,
        transform(T0, TownArmy::Garrison, at),
    )
    .unwrap();
    let count = joined.iter().find(|s| s.slot == at).unwrap().count;
    assert_eq!(
        events[0],
        changed(ArmyRef::Hero(RED_HERO), at, stack(at, imp, count))
    );
    assert_eq!(
        army(&game).iter().find(|s| s.slot == at).unwrap().creature,
        imp
    );
}
