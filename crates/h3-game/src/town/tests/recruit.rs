//! Tests of recruitment at a town in the game (E09 T27, the town epic's design note §49 to §51;
//! TWN-REC-001, -002, GAM-DEC-008 amended): `TownCommand::Recruit` opens the hire dialog of what
//! a building sells for the town's owner in its turn, its target the town's garrison; a purchase
//! is checked by the town, the garrison's room and the treasury, each refusal changing nothing,
//! then paid, the town's count lowered and the creatures joined into the garrison, each change
//! told; the legal commands list each town's recruitments after its builds. On the growth tests'
//! game: red's T0 with its level-1 and level-2 dwellings (15 imps at 50 gold, 8 gogs at 100), red's
//! T2 with its level-1 one, blue's T1.

use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{HireDialog, HireError, HireSource, HireTarget};
use h3_content::{CreatureTypeId, ResourceId};
use h3_contracts::{ArmyStack, PlayerId, TownId};
use h3_towns::{
    BuildingId, BuildingRoles, Dwelling, RecruitSource, Summoned, TownCommand, TownCommandError,
};

use super::command::{BLUE, RED, end_turn, run};
use super::garrison::building;
use super::growth::{DWELLING_1, T0, T1, T2, built, rules_on, rules_with, setup};
use crate::tests::key;
use crate::{
    Actor, Answer, ArmyRef, ArmySlot, DecisionView, GameCommand, GameCommandError, GameEvent,
    GameRuleset, GameState, HireAnswer, HireCreature, HireSeen, HireView, Viewer, legal_commands,
    observe, project,
};

const GOLD: ResourceId = ResourceId::from_raw(0);
/// The level-2 dwelling (`b8`, the gogs').
const DWELLING_2: usize = 8;
/// The Portal (`b13` of [`rules_portal`]) or the upgraded level-1 dwelling (`b13` of
/// [`rules_upgraded`]).
const B13: usize = 13;
/// The imp's upgrade of [`rules_upgraded`].
const IMP2: &str = "t:elf";

fn b(index: usize) -> BuildingId {
    BuildingId::from_index(index).unwrap()
}

fn recruit(town: TownId, index: usize) -> GameCommand {
    GameCommand::Town(TownCommand::Recruit {
        town,
        building: b(index),
    })
}

fn buy(creature: CreatureTypeId, amount: u32) -> GameCommand {
    GameCommand::Answer(Answer::Hire(HireAnswer::Buy { creature, amount }))
}

fn creature(rules: &GameRuleset, k: &str) -> CreatureTypeId {
    rules.core().creature_by_key(k).unwrap()
}

/// The growth tests' rules.
fn rules() -> GameRuleset {
    rules_on([1, 0, 0], |_| {})
}

/// [`rules`] with the Portal of Summoning as `b13` (no slot).
fn rules_portal() -> GameRuleset {
    rules_on([1, 0, 0], |towns| {
        let portal = BuildingRoles {
            portal: true,
            ..BuildingRoles::default()
        };
        towns.towns[0].buildings.push(building("b13", None, portal));
    })
}

/// [`rules`] with an upgraded level 1, the imp's upgrade (60 gold), sold by `b13`, which supersedes
/// `b7`: the core's fourth creature, which the battle tests' pack keys `t:elf`.
fn rules_upgraded() -> GameRuleset {
    rules_with(
        [1, 0, 0],
        |core| {
            let mut imp2 = core.creatures[0].clone();
            imp2.key = key(IMP2);
            imp2.info.cost = vec![(key("t:gold"), 60)];
            core.creatures.push(imp2);
            core.creatures[0].info.upgrades = vec![key(IMP2)];
        },
        |towns| {
            let record = &mut towns.towns[0];
            record.creatures[0].upgrade = Some(key(IMP2));
            let upgraded = BuildingRoles {
                dwelling: Some(Dwelling {
                    level: 1,
                    upgraded: true,
                }),
                ..BuildingRoles::default()
            };
            let mut b13 = building("b13", None, upgraded);
            b13.supersedes = vec!["b7".into()];
            record.buildings.push(b13);
        },
    )
}

/// The growth tests' game on day 1 with red holding `gold`.
fn game(rules: &GameRuleset, gold: u32) -> GameState {
    let mut game = built(rules, &setup(rules));
    game.set_resource(RED, GOLD, gold);
    game
}

fn gold(game: &GameState, player: PlayerId) -> u32 {
    game.treasury(player).unwrap()[usize::from(GOLD.raw())]
}

/// The garrison of `town`.
fn garrison(game: &GameState, town: TownId) -> Vec<ArmyStack> {
    let adventure = game.adventure();
    let index = adventure.map().town_object(town).unwrap();
    adventure.town_garrison(index).unwrap().to_vec()
}

/// `player`'s open hire dialog as its observation shows it.
fn seen(game: &GameState, rules: &GameRuleset, player: PlayerId) -> Option<HireView> {
    match observe(game, rules, Viewer::Player(player)).decision? {
        DecisionView::Hire(view) => Some(view),
        _ => None,
    }
}

/// The owner's counts `events` last told for `town`, if any.
fn told_counts(events: &[GameEvent], player: PlayerId, town: TownId) -> Option<[u16; 14]> {
    events.iter().rev().find_map(|e| match *e {
        GameEvent::TownChanged {
            player: p,
            town: t,
            buildings: Some(buildings),
            ..
        } if p == player && t == town => Some(buildings.available),
        _ => None,
    })
}

// covers: TWN-REC-001, GAM-DEC-008
#[test]
fn a_recruitment_opens_the_towns_dialog_for_its_owner_alone() {
    let rules = rules();
    let mut game = game(&rules, 5_000);
    let imp = creature(&rules, "t:imp");
    let events = run(&rules, &mut game, RED, recruit(T0, DWELLING_1)).unwrap();
    let source = HireSeen::Town { town: T0, index: 0 };
    let opened = GameEvent::HireOpened {
        player: RED,
        target: HireTarget::Garrison(T0),
        source,
    };
    assert_eq!(events, [opened]);
    assert_eq!(project(&opened, Viewer::Player(RED)), Some(opened));
    assert_eq!(project(&opened, Viewer::Player(BLUE)), None);
    let dialog = game.hire_dialog(RED).copied().unwrap();
    assert_eq!(dialog.target, HireTarget::Garrison(T0));
    assert_eq!(dialog.source, HireSource::Town { town: T0, index: 0 });
    assert_eq!(
        seen(&game, &rules, RED),
        Some(HireView {
            target: HireTarget::Garrison(T0),
            source,
            creatures: vec![HireCreature {
                creature: imp,
                count: 15,
                cost: vec![(GOLD, 50)],
            }],
        })
    );
    assert_eq!(seen(&game, &rules, BLUE), None);
    // Its answers: the most of the imps (the count, the treasury's 100 and the room's), then
    // `Close`; nothing else while it is open.
    assert_eq!(
        legal_commands(&game, &rules, Actor::Player(RED)),
        [
            buy(imp, 15),
            GameCommand::Answer(Answer::Hire(HireAnswer::Close))
        ]
    );
    assert_eq!(
        run(&rules, &mut game, RED, recruit(T0, DWELLING_2)),
        Err(GameCommandError::DecisionOpen)
    );
}

// covers: TWN-REC-001, TWN-REC-002
#[test]
fn every_refused_recruitment_changes_nothing() {
    let rules = rules_portal();
    let mut game = game(&rules, 5_000);
    let towns = game.towns_mut();
    towns.build(T0, b(B13), rules.towns(), rules.core(), false);
    let refused = [
        (
            recruit(TownId::new(9), DWELLING_1),
            TownCommandError::UnknownTown,
        ),
        (recruit(T1, DWELLING_1), TownCommandError::NotOwner),
        (recruit(T0, 63), TownCommandError::UnknownBuilding),
        (recruit(T0, 0), TownCommandError::NotRecruiting),
        (recruit(T2, DWELLING_2), TownCommandError::NotStanding),
        (recruit(T0, B13), TownCommandError::NothingSummoned),
    ];
    for (command, error) in refused {
        let result = run(&rules, &mut game, RED, command.clone());
        assert_eq!(result, Err(GameCommandError::Town(error)), "{command:?}");
    }
    // Not blue's turn.
    assert!(run(&rules, &mut game, BLUE, recruit(T1, DWELLING_1)).is_err());
    // A count of 0 opens the dialog all the same: its only answer is to close it.
    let empty = game.towns().get(T2).unwrap().with_available(0, 0);
    let _ = game.towns_mut().write(T2, empty);
    run(&rules, &mut game, RED, recruit(T2, DWELLING_1)).unwrap();
    assert_eq!(
        legal_commands(&game, &rules, Actor::Player(RED)),
        [GameCommand::Answer(Answer::Hire(HireAnswer::Close))]
    );
}

// covers: TWN-REC-001, GAM-DEC-008, ADV-ARM-007
#[test]
fn a_purchase_pays_lowers_the_count_and_joins_the_garrison() {
    let rules = rules();
    let mut game = game(&rules, 5_000);
    let imp = creature(&rules, "t:imp");
    run(&rules, &mut game, RED, recruit(T0, DWELLING_1)).unwrap();
    let before = game.state_hash();
    let events = run(&rules, &mut game, RED, buy(imp, 4)).unwrap();
    let stack = ArmyStack {
        slot: 0,
        creature: imp,
        count: 4,
    };
    assert_eq!(
        events[..2],
        [
            GameEvent::ResourcePaid {
                player: RED,
                resource: GOLD,
                amount: 200,
            },
            GameEvent::StackChanged {
                player: RED,
                at: ArmySlot {
                    army: ArmyRef::Garrison(T0),
                    slot: 0,
                },
                stack: Some(stack),
            },
        ]
    );
    // The owner's view told with the count lowered, then the one-creature dialog closed.
    assert_eq!(told_counts(&events, RED, T0).map(|c| c[0]), Some(11));
    assert_eq!(
        events.last(),
        Some(&GameEvent::DecisionClosed { player: RED })
    );
    assert!(!game.has_decision(RED));
    assert_ne!(game.state_hash(), before);
    assert_eq!(game.towns().get(T0).unwrap().available(0), 11);
    assert_eq!(game.towns().get(T0).unwrap().available(1), 8);
    assert_eq!(gold(&game, RED), 4_800);
    assert_eq!(garrison(&game, T0), [stack]);
    // A second purchase joins the stack.
    run(&rules, &mut game, RED, recruit(T0, DWELLING_1)).unwrap();
    let events = run(&rules, &mut game, RED, buy(imp, 11)).unwrap();
    assert!(events.contains(&GameEvent::StackChanged {
        player: RED,
        at: ArmySlot {
            army: ArmyRef::Garrison(T0),
            slot: 0,
        },
        stack: Some(ArmyStack { count: 15, ..stack }),
    }));
    assert_eq!(game.towns().get(T0).unwrap().available(0), 0);
}

// covers: TWN-REC-001, GAM-DEC-008, ADV-ARM-007
#[test]
fn every_refused_purchase_changes_nothing() {
    let rules = rules();
    let (imp, gog) = (creature(&rules, "t:imp"), creature(&rules, "t:gog"));
    let mut game = game(&rules, 5_000);
    run(&rules, &mut game, RED, recruit(T0, DWELLING_1)).unwrap();
    let mut poor = game.clone();
    poor.set_resource(RED, GOLD, 149);
    let mut full = game.clone();
    let stacks: Vec<ArmyStack> = (0..7)
        .map(|slot| ArmyStack {
            slot,
            creature: CreatureTypeId::from_raw(u16::from(slot) + 20),
            count: 1,
        })
        .collect();
    full.adventure_with_heroes().0.set_town_garrison(T0, stacks);
    let refused = [
        (&mut game.clone(), buy(gog, 1), HireError::NotOffered(gog)),
        (&mut game.clone(), buy(imp, 0), HireError::Amount(0)),
        (&mut game.clone(), buy(imp, 16), HireError::Amount(16)),
        // The town's tests before the room, the room before the treasury.
        (&mut full.clone(), buy(imp, 16), HireError::Amount(16)),
        (&mut full.clone(), buy(imp, 1), HireError::NoRoom),
    ];
    for (state, command, error) in refused {
        let result = run(&rules, state, RED, command.clone());
        assert_eq!(result, Err(GameCommandError::Hire(error)), "{command:?}");
        assert!(state.has_decision(RED));
    }
    assert_eq!(
        run(&rules, &mut poor, RED, buy(imp, 3)),
        Err(GameCommandError::CannotPay)
    );
    assert!(run(&rules, &mut poor, RED, buy(imp, 2)).is_ok());
}

// covers: TWN-REC-002, GAM-DEC-008
#[test]
fn the_portals_purchase_keeps_its_creature() {
    let rules = rules_portal();
    let gog = creature(&rules, "t:gog");
    let mut game = game(&rules, 5_000);
    let towns = game.towns_mut();
    towns.build(T0, b(B13), rules.towns(), rules.core(), false);
    let held = Summoned {
        creature: gog,
        count: 6,
    };
    let record = game.towns().get(T0).unwrap().with_portal(Some(held));
    let _ = game.towns_mut().write(T0, record);
    let listed = legal_commands(&game, &rules, Actor::Player(RED));
    assert!(listed.contains(&recruit(T0, B13)));
    let events = run(&rules, &mut game, RED, recruit(T0, B13)).unwrap();
    assert_eq!(
        events,
        [GameEvent::HireOpened {
            player: RED,
            target: HireTarget::Garrison(T0),
            source: HireSeen::Portal { town: T0 },
        }]
    );
    let view = seen(&game, &rules, RED).unwrap();
    assert_eq!(
        view.creatures,
        [HireCreature {
            creature: gog,
            count: 6,
            cost: vec![(GOLD, 100)],
        }]
    );
    let events = run(&rules, &mut game, RED, buy(gog, 2)).unwrap();
    assert_eq!(
        events.last(),
        Some(&GameEvent::DecisionClosed { player: RED })
    );
    let left = Summoned { count: 4, ..held };
    assert_eq!(game.towns().get(T0).unwrap().portal(), Some(left));
    // The level-2 dwelling's count is the Portal's apart.
    assert_eq!(game.towns().get(T0).unwrap().available(1), 8);
    assert_eq!(
        garrison(&game, T0),
        [ArmyStack {
            slot: 0,
            creature: gog,
            count: 2,
        }]
    );
    // Bought out, the Portal keeps its creature at 0 and is still listed.
    run(&rules, &mut game, RED, recruit(T0, B13)).unwrap();
    run(&rules, &mut game, RED, buy(gog, 4)).unwrap();
    let out = Summoned { count: 0, ..held };
    assert_eq!(game.towns().get(T0).unwrap().portal(), Some(out));
    assert!(legal_commands(&game, &rules, Actor::Player(RED)).contains(&recruit(T0, B13)));
}

// covers: TWN-REC-001, GAM-DEC-008
#[test]
fn an_upgraded_index_sells_both_creatures_from_one_count() {
    let rules = rules_upgraded();
    let (imp, imp2) = (creature(&rules, "t:imp"), creature(&rules, IMP2));
    let mut game = game(&rules, 5_000);
    let towns = game.towns_mut();
    towns.build(T0, b(B13), rules.towns(), rules.core(), false);
    let count = game.towns().get(T0).unwrap().available(7);
    assert!(count > 3, "{count}");
    // The base dwelling is superseded: only the upgrade sells.
    assert_eq!(
        run(&rules, &mut game, RED, recruit(T0, DWELLING_1)),
        Err(GameCommandError::Town(TownCommandError::NotStanding))
    );
    run(&rules, &mut game, RED, recruit(T0, B13)).unwrap();
    let view = seen(&game, &rules, RED).unwrap();
    assert_eq!(view.source, HireSeen::Town { town: T0, index: 7 });
    let shown: Vec<(CreatureTypeId, u32)> = view
        .creatures
        .iter()
        .map(|c| (c.creature, c.count))
        .collect();
    assert_eq!(shown, [(imp2, u32::from(count)), (imp, u32::from(count))]);
    // A purchase of either lowers the one count; the two-creature dialog stays open.
    let events = run(&rules, &mut game, RED, buy(imp, 2)).unwrap();
    assert!(!events.contains(&GameEvent::DecisionClosed { player: RED }));
    let view = seen(&game, &rules, RED).unwrap();
    let left = u32::from(count) - 2;
    assert_eq!(view.creatures[0].count, left);
    assert_eq!(view.creatures[1].count, left);
    run(&rules, &mut game, RED, buy(imp2, 1)).unwrap();
    assert_eq!(game.towns().get(T0).unwrap().available(7), count - 3);
    assert_eq!(gold(&game, RED), 5_000 - 100 - 60);
    let army: Vec<(CreatureTypeId, u32)> = garrison(&game, T0)
        .iter()
        .map(|s| (s.creature, s.count))
        .collect();
    assert_eq!(army, [(imp, 2), (imp2, 1)]);
    // The hash names the dialog's source: another town's index hashes apart.
    let mut other = game.clone();
    run(
        &rules,
        &mut game,
        RED,
        GameCommand::Answer(Answer::Hire(HireAnswer::Close)),
    )
    .unwrap();
    run(
        &rules,
        &mut other,
        RED,
        GameCommand::Answer(Answer::Hire(HireAnswer::Close)),
    )
    .unwrap();
    run(&rules, &mut game, RED, recruit(T0, B13)).unwrap();
    run(&rules, &mut other, RED, recruit(T2, DWELLING_1)).unwrap();
    assert_ne!(game.state_hash(), other.state_hash());
}

// covers: TWN-REC-001
#[test]
fn a_dialog_of_a_town_the_store_lacks_sells_nothing() {
    // Defensive: nothing removes a town while its dialog is open.
    let rules = rules();
    let mut game = game(&rules, 5_000);
    let imp = creature(&rules, "t:imp");
    let gone = TownId::new(9);
    let source = (gone, RecruitSource::Index(0));
    assert_eq!(crate::town::recruit::stock(&game, &rules, source), None);
    let dialog = HireDialog {
        target: HireTarget::Garrison(gone),
        source: HireSource::Town {
            town: gone,
            index: 0,
        },
    };
    let mut events = Vec::new();
    let refused = crate::town::recruit::buy(
        &mut game,
        &rules,
        RED,
        (&dialog, source),
        (imp, 1),
        &mut events,
    );
    assert_eq!(refused, Err(GameCommandError::Hire(HireError::NoSource)));
    assert!(events.is_empty());
}

// covers: TWN-REC-001, TWN-REC-002, GAM-DEC-001
#[test]
fn the_legal_recruitments_follow_each_towns_builds() {
    let rules = rules_portal();
    let mut game = game(&rules, 5_000);
    let listed = legal_commands(&game, &rules, Actor::Player(RED));
    let recruits: Vec<&GameCommand> = listed
        .iter()
        .filter(|c| matches!(c, GameCommand::Town(TownCommand::Recruit { .. })))
        .collect();
    // In `TownId` order, each town's in its record's order; a Portal holding nothing is not
    // listed (the test town's buildings are free: every town lists builds too).
    assert_eq!(
        recruits,
        [
            &recruit(T0, DWELLING_1),
            &recruit(T0, DWELLING_2),
            &recruit(T2, DWELLING_1)
        ]
    );
    let at = |command: &GameCommand| listed.iter().position(|c| c == command).unwrap();
    let builds = |town: TownId| {
        listed.iter().enumerate().filter_map(move |(i, c)| {
            matches!(c, GameCommand::Town(TownCommand::Build { town: t, .. }) if *t == town)
                .then_some(i)
        })
    };
    assert!(builds(T0).all(|i| i < at(&recruit(T0, DWELLING_1))));
    assert!(builds(T2).all(|i| at(&recruit(T0, DWELLING_2)) < i));
    assert!(builds(T2).all(|i| i < at(&recruit(T2, DWELLING_1))));
    assert!(builds(T2).next().is_some());
    // Blue lists nothing in red's turn; in its own, its T1's.
    assert!(legal_commands(&game, &rules, Actor::Player(BLUE)).is_empty());
    end_turn(&rules, &mut game, RED);
    let blue = legal_commands(&game, &rules, Actor::Player(BLUE));
    assert!(blue.contains(&recruit(T1, DWELLING_1)));
    assert!(!blue.contains(&recruit(T0, DWELLING_1)));
}
