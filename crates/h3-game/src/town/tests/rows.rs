//! Tests of a town's two rows in the game (E09 T28a, the town epic's design note §52, §53;
//! TWN-ROW-001 to -004, GAM-MAN-001 and -002 amended): `TownCommand::Heroes` is the town's
//! owner's in its turn, each refusal changing nothing, its events and the town's changed views
//! told; the legal commands list each own town's moves after its recruitments; outside a
//! decision the owner manages an own town's upper bar alone or with its visitor, and a garrison
//! hero as its own; a town's purchase joins its garrison hero's army; a captured town's garrison
//! hero goes back to the pool (`TWN-STUB-14`). On the growth tests' game: red's T0 on (1, 1) with
//! red's hero on its entrance, blue's T1, red's T2 on (0, 3).

use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{AdventureEvent, AdventureEventKind, ArmyCommand, HeroOnMap, Pos};
use h3_content::{ArtifactClass, ArtifactDef, ContentPack, Formation, HeroEffect, ResourceId};
use h3_contracts::{ArmyStack, HeroId, TownId};
use h3_heroes::wearing;
use h3_towns::{BuildingId, GrowthInputs, TownCommand, TownCommandError, TownHeroChange};

use super::command::{BLUE, RED, end_turn, run};
use super::growth::{DWELLING_1, T0, T1, built, setup};
use crate::tests::key;
use crate::{
    Actor, Answer, ArmyRef, ArmySlot, GameCommand, GameCommandError, GameEvent, GameRuleset,
    GameSetup, GameState, HeroArtifactsSetup, HireAnswer, Manage, ManageError, Viewer,
    legal_commands, legal_management, legal_town_management, observe, split_range,
};
use h3_adventure::ArmyCommandError;

const GOLD: ResourceId = ResourceId::from_raw(0);

fn rules() -> GameRuleset {
    super::growth::rules_on([1, 0, 0], |_| {})
}

fn heroes(town: TownId, change: TownHeroChange) -> GameCommand {
    GameCommand::Town(TownCommand::Heroes { town, change })
}

fn stack(slot: u8, creature: h3_content::CreatureTypeId, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature,
        count,
    }
}

/// The growth tests' setup with a second hero of red's on (0, 0).
fn two_heroes(rules: &GameRuleset) -> GameSetup {
    let mut setup = setup(rules);
    let mut second = setup.heroes[0].clone();
    second.pos = Pos::new(0, 0, 0);
    second.details = crate::HeroDetailsSetup::default();
    setup.heroes.push(second);
    setup
}

/// The game of `setup` with red holding 2 000 gold and T0's garrison `garrison`.
fn game(rules: &GameRuleset, setup: &GameSetup, garrison: Vec<ArmyStack>) -> GameState {
    let mut game = built(rules, setup);
    game.set_resource(RED, GOLD, 2_000);
    let (adventure, _) = game.adventure_with_heroes();
    adventure.set_town_garrison(T0, garrison);
    game
}

/// T0's visitor, red's hero on its entrance.
fn visitor(game: &GameState) -> HeroId {
    game.adventure().town_visitor(T0).unwrap()
}

/// The town commands of `listed` that move heroes.
fn moves(listed: &[GameCommand]) -> Vec<GameCommand> {
    listed
        .iter()
        .filter(|c| matches!(c, GameCommand::Town(TownCommand::Heroes { .. })))
        .cloned()
        .collect()
}

fn slot(army: ArmyRef, slot: u8) -> ArmySlot {
    ArmySlot { army, slot }
}

// covers: TWN-ROW-002, TWN-ROW-003, TWN-ROW-004
#[test]
fn a_hero_move_is_the_town_s_owner_s_and_tells_what_it_changes() {
    let rules = rules();
    let pike = rules.core().creature_by_key("t:imp").unwrap();
    let mut game = game(&rules, &two_heroes(&rules), vec![stack(0, pike, 3)]);
    let hero = visitor(&game);
    let refused =
        |game: &mut GameState, player, command| run(&rules, game, player, command).unwrap_err();
    let town = GameCommandError::Town;
    assert_eq!(
        refused(
            &mut game,
            RED,
            heroes(TownId::new(9), TownHeroChange::Enter)
        ),
        town(TownCommandError::UnknownTown)
    );
    assert_eq!(
        refused(&mut game, RED, heroes(T1, TownHeroChange::Enter)),
        town(TownCommandError::NotOwner)
    );
    assert_eq!(
        refused(&mut game, RED, heroes(T0, TownHeroChange::Leave)),
        town(TownCommandError::NoGarrisonHero)
    );
    // Listed: `Enter` alone, after the town's builds and recruitments.
    let listed = legal_commands(&game, &rules, Actor::Player(RED));
    assert_eq!(moves(&listed), [heroes(T0, TownHeroChange::Enter)]);
    let at = listed
        .iter()
        .position(|c| *c == heroes(T0, TownHeroChange::Enter))
        .unwrap();
    let town_of = |c: &GameCommand| match *c {
        GameCommand::Town(TownCommand::Build { town, .. } | TownCommand::Recruit { town, .. }) => {
            Some(town)
        }
        _ => None,
    };
    assert_eq!(town_of(&listed[at - 1]), Some(T0));
    assert!(listed[at + 1..].iter().all(|c| town_of(c) != Some(T0)));
    let events = run(&rules, &mut game, RED, heroes(T0, TownHeroChange::Enter)).unwrap();
    let held = game.adventure().town_hero(T0).unwrap();
    assert_eq!(held.id, hero);
    // The imps join the hero's imps (its slot 1, 9 of them), the town's garrison empties.
    assert_eq!(held.army, [stack(1, pike, 12)]);
    let entered = GameEvent::Adventure(AdventureEvent {
        kind: AdventureEventKind::GarrisonEntered {
            hero,
            town: T0,
            pos: Pos::new(1, 1, 0),
        },
        seen_by: h3_adventure::Viewers::NONE.with(RED).with(BLUE),
    });
    assert_eq!(events[0], entered);
    let changed = |at, stack| GameEvent::StackChanged {
        player: RED,
        at,
        stack,
    };
    assert!(events.contains(&changed(
        slot(ArmyRef::Hero(hero), 1),
        Some(stack(1, pike, 12))
    )));
    assert!(events.contains(&changed(slot(ArmyRef::Garrison(T0), 0), None)));
    // The owner's view of T0 is told (its garrison emptied).
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::TownChanged {
                player: RED,
                town: T0,
                ..
            }
        )),
        "{events:?}"
    );
    // Then `Leave` alone (no visitor).
    let listed = legal_commands(&game, &rules, Actor::Player(RED));
    assert_eq!(moves(&listed), [heroes(T0, TownHeroChange::Leave)]);
    // Red's other hero steps onto the entrance: `Swap` too, `Leave` no more.
    let other = game
        .adventure()
        .heroes()
        .iter()
        .find(|h| h.owner == RED)
        .map(|h| h.id)
        .unwrap();
    let step = GameCommand::Adventure(h3_adventure::AdventureCommand::MoveHero {
        hero: other,
        path: vec![Pos::new(1, 1, 0)],
    });
    run(&rules, &mut game, RED, step).unwrap();
    let listed = legal_commands(&game, &rules, Actor::Player(RED));
    assert_eq!(moves(&listed), [heroes(T0, TownHeroChange::Swap)]);
    run(&rules, &mut game, RED, heroes(T0, TownHeroChange::Swap)).unwrap();
    assert_eq!(game.adventure().town_hero(T0).unwrap().id, other);
    assert_eq!(visitor(&game), hero);
    // Blue's turn: red's moves are refused as any command out of turn.
    end_turn(&rules, &mut game, RED);
    assert_eq!(
        refused(&mut game, RED, heroes(T0, TownHeroChange::Leave)),
        GameCommandError::NotYourTurn(RED)
    );
    assert_eq!(
        moves(&legal_commands(&game, &rules, Actor::Player(BLUE))),
        []
    );
}

// covers: TWN-ROW-001, GAM-MAN-001, GAM-MAN-002
#[test]
fn the_owner_manages_its_town_s_bars_outside_a_decision() {
    let rules = rules();
    let imp = rules.core().creature_by_key("t:imp").unwrap();
    let gog = rules.core().creature_by_key("t:gog").unwrap();
    let mut setup = two_heroes(&rules);
    setup.heroes[0].army = crate::HeroArmy::Custom(vec![stack(0, gog, 2), stack(1, imp, 9)]);
    let mut game = game(&rules, &setup, vec![stack(0, imp, 4)]);
    let hero = visitor(&game);
    let other = game
        .adventure()
        .heroes()
        .iter()
        .find(|h| h.owner == RED && h.id != hero)
        .map(|h| h.id)
        .unwrap();
    let army = |command| GameCommand::Manage(Manage::Army(command));
    let out = ArmyCommand::Move {
        from: slot(ArmyRef::Garrison(T0), 0),
        to: slot(ArmyRef::Hero(hero), 1),
    };
    // Listed for the visitor (toward the upper bar) and for the town.
    let red = Actor::Player(RED);
    assert!(
        legal_management(&game, &rules, red, hero).contains(&Manage::Army(ArmyCommand::Move {
            from: slot(ArmyRef::Hero(hero), 0),
            to: slot(ArmyRef::Garrison(T0), 1),
        }))
    );
    let town = legal_town_management(&game, &rules, red, T0);
    assert!(town.contains(&Manage::Army(out)));
    assert!(town.contains(&Manage::Army(ArmyCommand::Dismiss {
        at: slot(ArmyRef::Garrison(T0), 0)
    })));
    assert_eq!(legal_town_management(&game, &rules, Actor::System, T0), []);
    assert_eq!(
        legal_town_management(&game, &rules, Actor::Player(BLUE), T0),
        []
    );
    assert_eq!(
        split_range(
            &game,
            &rules,
            red,
            (
                slot(ArmyRef::Garrison(T0), 0),
                slot(ArmyRef::Garrison(T0), 1)
            )
        ),
        Some(1..=3)
    );
    // Red's other hero is not the town's visitor: the garrison is not admitted with it.
    let away = ArmyCommand::Move {
        from: slot(ArmyRef::Garrison(T0), 0),
        to: slot(ArmyRef::Hero(other), 0),
    };
    assert_eq!(
        run(&rules, &mut game.clone(), RED, army(away)).unwrap_err(),
        GameCommandError::Manage(ManageError::Army(ArmyCommandError::Garrison(T0)))
    );
    // Nor are two heroes on the map that do not meet.
    let apart = ArmyCommand::Move {
        from: slot(ArmyRef::Hero(hero), 0),
        to: slot(ArmyRef::Hero(other), 0),
    };
    assert_eq!(
        run(&rules, &mut game.clone(), RED, army(apart)).unwrap_err(),
        GameCommandError::Manage(ManageError::NotTogether)
    );
    // Blue's town's garrison is not red's to manage.
    let blue = ArmyCommand::Dismiss {
        at: slot(ArmyRef::Garrison(T1), 0),
    };
    assert_eq!(
        run(&rules, &mut game.clone(), RED, army(blue)).unwrap_err(),
        GameCommandError::Manage(ManageError::Army(ArmyCommandError::Garrison(T1)))
    );
    let events = run(&rules, &mut game, RED, army(out)).unwrap();
    assert!(events.contains(&GameEvent::StackChanged {
        player: RED,
        at: slot(ArmyRef::Hero(hero), 1),
        stack: Some(stack(1, imp, 13)),
    }));
    assert_eq!(game.adventure().town_garrison(0), Some(&[][..]));
    // The garrison alone: a move within it, here from the hero back into its empty slot.
    let back = ArmyCommand::Move {
        from: slot(ArmyRef::Hero(hero), 1),
        to: slot(ArmyRef::Garrison(T0), 0),
    };
    run(&rules, &mut game, RED, army(back)).unwrap();
    let within = ArmyCommand::Move {
        from: slot(ArmyRef::Garrison(T0), 0),
        to: slot(ArmyRef::Garrison(T0), 1),
    };
    run(&rules, &mut game, RED, army(within)).unwrap();
    // A garrison hero manages itself (its formation, its artifacts) and meets its visitor.
    run(&rules, &mut game, RED, heroes(T0, TownHeroChange::Enter)).unwrap();
    let listed = legal_management(&game, &rules, red, hero);
    assert_eq!(legal_town_management(&game, &rules, red, T0), listed);
    let worn = listed
        .iter()
        .find(|m| matches!(m, Manage::Artifact { .. }))
        .copied()
        .unwrap();
    run(&rules, &mut game, RED, GameCommand::Manage(worn)).unwrap();
    let tight = Manage::Formation {
        hero,
        formation: Formation::Tight,
    };
    run(&rules, &mut game, RED, GameCommand::Manage(tight)).unwrap();
    assert_eq!(
        game.adventure()
            .town_hero(T0)
            .map(|h: &HeroOnMap| h.formation),
        Some(Formation::Tight)
    );
    assert!(!listed.iter().any(|m| matches!(
        m,
        Manage::Army(ArmyCommand::Move { to, .. }) if to.army == ArmyRef::Hero(other)
    )));
    let step = GameCommand::Adventure(h3_adventure::AdventureCommand::MoveHero {
        hero: other,
        path: vec![Pos::new(1, 1, 0)],
    });
    run(&rules, &mut game, RED, step).unwrap();
    let with_visitor = legal_management(&game, &rules, red, hero);
    let toward = Manage::Army(ArmyCommand::Move {
        from: slot(ArmyRef::Hero(hero), 1),
        to: slot(ArmyRef::Hero(other), 1),
    });
    assert!(with_visitor.contains(&toward));
    run(&rules, &mut game, RED, GameCommand::Manage(toward)).unwrap();
    // The visitor's own listing names the garrison hero as its other army.
    assert!(
        legal_management(&game, &rules, red, other)
            .iter()
            .any(|m| matches!(
                m,
                Manage::Army(ArmyCommand::Move { to, .. }) if to.army == ArmyRef::Hero(hero)
            ))
    );
}

// covers: TWN-REC-001, TWN-ROW-001
#[test]
fn a_town_s_purchase_joins_its_garrison_hero_s_army() {
    let rules = rules();
    let imp = rules.core().creature_by_key("t:imp").unwrap();
    let mut game = game(&rules, &setup(&rules), vec![]);
    let hero = visitor(&game);
    run(&rules, &mut game, RED, heroes(T0, TownHeroChange::Enter)).unwrap();
    let recruit = GameCommand::Town(TownCommand::Recruit {
        town: T0,
        building: BuildingId::from_index(DWELLING_1).unwrap(),
    });
    run(&rules, &mut game, RED, recruit).unwrap();
    let buy = GameCommand::Answer(Answer::Hire(HireAnswer::Buy {
        creature: imp,
        amount: 2,
    }));
    let events = run(&rules, &mut game, RED, buy).unwrap();
    assert!(events.contains(&GameEvent::StackChanged {
        player: RED,
        at: slot(ArmyRef::Hero(hero), 1),
        stack: Some(stack(1, imp, 11)),
    }));
    assert_eq!(
        game.adventure().town_hero(T0).unwrap().army,
        [stack(1, imp, 11)]
    );
}

// covers: TWN-GRO-001, TWN-ROW-005
#[test]
fn a_garrison_hero_s_legion_and_statue_count_in_its_owner_s_towns_growth() {
    let rules = rules();
    let mut setup = two_heroes(&rules);
    // Red's second hero wears the Legs alone (no Statue).
    let legs = rules.core().artifact_by_key("t:legs").unwrap();
    setup.heroes.last_mut().unwrap().details.artifacts = Some(HeroArtifactsSetup {
        worn: wearing(rules.core(), &[legs.into()]),
        backpack: vec![],
    });
    let mut game = game(&rules, &setup, vec![]);
    let owned = |legion: u16| {
        let mut inputs = GrowthInputs {
            owned: true,
            share_bp: 5_000,
            ..GrowthInputs::NEUTRAL
        };
        inputs.legion[0] = legion;
        inputs.dwellings[0] = 1;
        inputs
    };
    let inputs = |game: &GameState| crate::town::growth::inputs(game, &rules);
    // The visitor with the Legs and the Statue enters T0's garrison: its Legs count at T0 and its
    // Statue's share at every town of red's (the original's `player_hero_wears_artifact`
    // 0x4BACB0 reads the towns' garrison heroes after the player's list).
    run(&rules, &mut game, RED, heroes(T0, TownHeroChange::Enter)).unwrap();
    let grown = inputs(&game);
    assert_eq!((grown[0], grown[2]), (owned(5), owned(0)));
    // Red's other hero steps onto the entrance: the visitor's Legs and the garrison hero's, each
    // hero once.
    let other = game.adventure().heroes().iter().find(|h| h.owner == RED);
    let step = GameCommand::Adventure(h3_adventure::AdventureCommand::MoveHero {
        hero: other.unwrap().id,
        path: vec![Pos::new(1, 1, 0)],
    });
    run(&rules, &mut game, RED, step).unwrap();
    assert_eq!(inputs(&game)[0], owned(10));
}

/// [`rules`] with a purse `t:purse` of 500 gold a day in the ring slot.
fn rules_purse() -> GameRuleset {
    let purse = |core: &mut ContentPack| {
        core.artifacts.push(ArtifactDef {
            key: key("t:purse"),
            name: "Artifact".into(),
            class: ArtifactClass::Treasure,
            cost: vec![],
            slot: Some(key("t:ring_slot")),
            war_machine: None,
            needs_skill: None,
            effects: vec![HeroEffect::DailyResource {
                resource: key("t:gold"),
                amount: 500,
            }],
            parts: vec![],
        });
    };
    super::growth::rules_with([1, 0, 0], purse, |_| {})
}

/// Red's gold gained from the start of `game`'s next day to the start of `day`'s.
fn gained(rules: &GameRuleset, mut game: GameState, day: u32) -> u32 {
    super::growth::to_day_told(rules, &mut game, day)
        .iter()
        .map(|e| match *e {
            GameEvent::ResourceGained {
                player: RED,
                resource,
                amount,
            } if resource == GOLD => amount,
            _ => 0,
        })
        .sum()
}

// covers: GAM-RES-004, TWN-ROW-005, ADV-MOV-002
#[test]
fn a_garrison_hero_s_income_and_points_are_those_it_had_on_the_map() {
    let rules = rules_purse();
    let mut setup = setup(&rules);
    let purse = rules.core().artifact_by_key("t:purse").unwrap();
    setup.heroes[0].details.artifacts = Some(HeroArtifactsSetup {
        worn: wearing(rules.core(), &[purse.into()]),
        backpack: vec![],
    });
    let on_map = game(&rules, &setup, vec![]);
    let mut held = on_map.clone();
    let hero = visitor(&held);
    run(&rules, &mut held, RED, heroes(T0, TownHeroChange::Enter)).unwrap();
    setup.heroes[0].details.artifacts = None;
    let bare = game(&rules, &setup, vec![]);
    // Day 2's income: the garrison hero's purse is paid as on the map (the original's
    // `count_worn_artifact` 0x4BABD0 and `town_gold_income` read the towns' garrison heroes).
    let paid = gained(&rules, on_map.clone(), 2);
    assert_eq!(paid, gained(&rules, bare, 2) + 500);
    assert_eq!(gained(&rules, held.clone(), 2), paid);
    // Red's day 2 turn start gives the garrison hero the full points the hero on the map has,
    // after its spent ones (batch TWN-3's row 6: the full points on Red's next day).
    let (mut on_map, mut spent) = (on_map.clone(), on_map);
    for to in [Pos::new(1, 2, 0), Pos::new(1, 1, 0)] {
        let step = GameCommand::Adventure(h3_adventure::AdventureCommand::MoveHero {
            hero,
            path: vec![to],
        });
        run(&rules, &mut spent, RED, step).unwrap();
    }
    run(&rules, &mut spent, RED, heroes(T0, TownHeroChange::Enter)).unwrap();
    let left = spent.adventure().town_hero(T0).map(|h| h.movement);
    assert!(left < spent.adventure().town_hero(T0).map(|h| h.max_movement));
    for game in [&mut on_map, &mut spent, &mut held] {
        super::growth::to_day(&rules, game, 2);
    }
    let full = on_map
        .adventure()
        .hero(hero)
        .map(|h| (h.max_movement, h.movement));
    for game in [&spent, &held] {
        let points = game
            .adventure()
            .town_hero(T0)
            .map(|h| (h.max_movement, h.movement));
        assert_eq!(points, full);
    }
    assert_ne!(full.map(|(max, _)| max), Some(0));
}

/// The adventure's refusals as the town command's, one to one (each the same name).
#[test]
fn each_refusal_of_a_move_is_the_town_command_s_own() {
    use TownCommandError as T;
    use h3_adventure::TownHeroError as E;
    let pairs = [
        (E::UnknownTown, T::UnknownTown),
        (E::GarrisonHeld, T::GarrisonHeld),
        (E::NoVisitor, T::NoVisitor),
        (E::ArmiesDoNotFit, T::ArmiesDoNotFit),
        (E::TooManyCreatures, T::TooManyCreatures),
        (E::NoGarrisonHero, T::NoGarrisonHero),
        (E::EntranceHeld, T::EntranceHeld),
        (E::TooManyHeroes, T::TooManyHeroes),
        (E::EmptyArmy, T::EmptyArmy),
    ];
    for (error, refused) in pairs {
        assert_eq!(crate::town::rows::town_error(error), refused);
    }
}

// covers: TWN-SET-005
// stub: TWN-STUB-14
#[test]
fn a_captured_town_s_garrison_hero_goes_back_to_the_pool() {
    let rules = rules();
    let mut game = game(&rules, &setup(&rules), vec![]);
    let hero = visitor(&game);
    let hero_type = game.hero(hero).unwrap().hero_type;
    run(&rules, &mut game, RED, heroes(T0, TownHeroChange::Enter)).unwrap();
    // Its owner sees it, with its details; blue does not.
    let red = observe(&game, &rules, Viewer::Player(RED));
    assert_eq!(red.adventure.garrison_heroes.len(), 1);
    assert!(
        red.heroes
            .iter()
            .any(|h| h.id == hero && h.details.is_some())
    );
    let blue = observe(&game, &rules, Viewer::Player(BLUE));
    assert!(blue.adventure.garrison_heroes.is_empty());
    assert!(!blue.heroes.iter().any(|h| h.id == hero));
    end_turn(&rules, &mut game, RED);
    // Blue's hero on (2, 2) enters the undefended town.
    let blue_hero = game
        .adventure()
        .heroes()
        .iter()
        .find(|h| h.owner == BLUE)
        .map(|h| h.id)
        .unwrap();
    let step = GameCommand::Adventure(h3_adventure::AdventureCommand::MoveHero {
        hero: blue_hero,
        path: vec![Pos::new(1, 1, 0)],
    });
    let events = run(&rules, &mut game, BLUE, step).unwrap();
    assert!(events.contains(&GameEvent::Adventure(AdventureEvent {
        kind: AdventureEventKind::HeroRemoved {
            hero,
            pos: Pos::new(1, 1, 0),
        },
        seen_by: h3_adventure::Viewers::NONE.with(RED),
    })));
    assert!(game.adventure().town_hero(T0).is_none());
    let index = game.adventure().map().town_object(T0).unwrap();
    assert_eq!(game.adventure().owner(index), Some(BLUE));
    assert_eq!(
        game.hero_pool().status(hero_type),
        Some(crate::town::pool::HeroStatus::Free)
    );
}
