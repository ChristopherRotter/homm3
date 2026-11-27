//! Tests of the market at a town (E09 T58a1, design note `E09-design.md` §103 to §106;
//! TWN-MKT-001 to -004, GAM-DEC-011 amended): `TownCommand::Market` opens E08's market decision
//! with the town as its place, its hero the town's (the visitor; with `TWN-QUIRK-15`'s flag off the
//! garrison hero too; none at a Marketplace alone), offering the trades and the transfer always,
//! the merchants' purchase and the guild's sale with a hero, at the trader's count of Marketplaces
//! (at least 1, at most the parameters' rows); each refusal changes nothing. On the town tests'
//! game (red's T0 on (1, 1) and blue's T1 on (3, 0), each built by its setup's slots only) with
//! wood beside the gold, every artifact at 2 000 gold, a Marketplace (map slot 14), the Artifact
//! Merchants (16) and the Freelancer's Guild (19) in `t:town`'s record, and E08's market tests'
//! rates (five Marketplaces' rows).

use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{ArmyRef, ArmySlot, MarketParamsDef, ObjectKindDef, ObjectTypeDef, Pos, Visit};
use h3_content::{ProfileDef, ResourceDef, ResourceId, Ruleset};
use h3_contracts::{PlayerId, TownId};
use h3_heroes::HeroChange;
use h3_towns::{BuildingDef, BuildingRoles, TownCommand, TownCommandError, TownHeroChange};

use super::command::{BLUE, RED, T0, T1, run};
use super::learn::{RED_HERO, game, setup, town_command};
use super::rules_edited;
use super::{town_pack, towns_core_pack};
use crate::tests::key;
use crate::{
    Answer, DecisionView, GameCommand, GameCommandError, GameEvent, GameRuleset, GameState,
    MarketAnswer, MarketError, MarketPlace, MarketScreen, MarketSeen, MarketVisit, Viewer, observe,
};

const GOLD: ResourceId = ResourceId::from_raw(0);
const WOOD: ResourceId = ResourceId::from_raw(1);
/// The map slots of the Marketplace, the Artifact Merchants and the Freelancer's Guild.
const MARKET_SLOT: u8 = 14;
const MERCHANTS_SLOT: u8 = 16;
const GUILD_SLOT: u8 = 19;

/// The market tests' rules under `profile`: the test pack's `t:faithful` (no quirk flag on, so the
/// garrison hero trades) or `t:visitor` (`TWN-QUIRK-15`'s flag on: the visitor alone), with the
/// market parameters' rows `resources` (five in E08's market tests).
fn rules_in(profile: &str, resources: Vec<u16>) -> GameRuleset {
    let mut core = towns_core_pack(false);
    core.resources.push(ResourceDef {
        key: key("t:wood"),
        name: "Wood".into(),
    });
    for artifact in &mut core.artifacts {
        artifact.cost = vec![(key("t:gold"), 2000)];
    }
    core.profiles.push(ProfileDef {
        key: key("t:visitor"),
        quirks: vec![("quirk.town.visitor_only_trades".to_string(), true)],
    });
    let core = Ruleset::compile(&[core], profile).unwrap();
    let mut towns = town_pack(false);
    let building = |k: &str, slot, roles| BuildingDef {
        key: k.to_string(),
        name: "Building".into(),
        slot: Some(slot),
        supersedes: vec![],
        forbidden_with: None,
        auto_with: vec![],
        requires: vec![],
        cost: vec![],
        roles,
        war_machine: None,
    };
    let marketplace = BuildingRoles {
        marketplace: true,
        ..BuildingRoles::default()
    };
    let merchants = BuildingRoles {
        artifact_merchants: true,
        ..BuildingRoles::default()
    };
    let freelancers = BuildingRoles {
        freelancers: true,
        ..BuildingRoles::default()
    };
    towns.towns[0].buildings.extend([
        building("marketplace", MARKET_SLOT, marketplace),
        building("merchants", MERCHANTS_SLOT, merchants),
        building("freelancers", GUILD_SLOT, freelancers),
    ]);
    let rows = resources.len();
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
        adventure.params.as_mut().unwrap().market = Some(MarketParamsDef {
            cost: key("t:gold"),
            values: vec![(key("t:wood"), 250), (key("t:gold"), 1)],
            artifact_values: vec![(key("t:wood"), 125), (key("t:gold"), 1)],
            artifacts: vec![20, 25, 30, 35, 40][..rows].to_vec(),
            creatures: vec![30, 45, 50, 65, 70][..rows].to_vec(),
            resources,
        });
    })
}

fn rules() -> GameRuleset {
    rules_in("t:faithful", vec![10, 15, 20, 25, 30])
}

fn market(town: TownId) -> GameCommand {
    town_command(TownCommand::Market { town })
}

const fn answer(answer: MarketAnswer) -> GameCommand {
    GameCommand::Answer(Answer::Market(answer))
}

/// The game of both towns built by `slots`, red's hero on T0's entrance or (`visitor` false) on
/// (0, 0), red with 10 000 gold.
fn built(rules: &GameRuleset, slots: &[u8], visitor: bool) -> GameState {
    let mut setup = setup(rules, slots);
    if !visitor {
        setup.heroes[0].pos = Pos::new(0, 0, 0);
    }
    game(rules, &setup)
}

/// The market's view red sees.
fn view(rules: &GameRuleset, game: &GameState) -> crate::MarketView {
    match observe(game, rules, Viewer::Player(RED)).decision {
        Some(DecisionView::Market(view)) => view,
        other => panic!("{other:?}"),
    }
}

fn held(game: &GameState, resource: ResourceId) -> u32 {
    game.treasury(RED).unwrap()[resource.index()]
}

/// `command` refused with the town's `error`, nothing changed (`run`'s checks).
fn refused(rules: &GameRuleset, game: &mut GameState, player: PlayerId, error: TownCommandError) {
    let result = run(rules, game, player, market(T0));
    assert_eq!(result, Err(GameCommandError::Town(error)));
}

// covers: TWN-MKT-002, TWN-MKT-001, TWN-MKT-003, GAM-DEC-011
#[test]
fn a_marketplace_opens_the_window_without_a_hero_for_trades_and_transfers() {
    let rules = rules();
    let rates = rules.adventure().params().market().unwrap();
    let mut game = built(&rules, &[MARKET_SLOT], false);
    assert!(game.adventure().town_visitor(T0).is_none());
    let events = run(&rules, &mut game, RED, market(T0)).unwrap();
    let place = MarketSeen::Town(T0);
    assert_eq!(
        events,
        [GameEvent::MarketOpened {
            player: RED,
            hero: None,
            place,
        }]
    );
    assert_eq!(crate::project(&events[0], Viewer::Player(BLUE)), None);
    let visit = MarketVisit {
        hero: None,
        place: MarketPlace::Town(T0),
    };
    assert_eq!(game.market(RED), Some(&visit));
    // The view: the town's two screens at red's one Marketplace, no sale, no stock; blue the
    // recipient.
    let seen = view(&rules, &game);
    assert_eq!((seen.hero, seen.place), (None, place));
    assert_eq!(
        seen.offers.screens,
        [MarketScreen::Trade, MarketScreen::Transfer]
    );
    assert_eq!(seen.offers.count, 1);
    assert!(seen.offers.sales.is_empty() && seen.offers.stock.is_empty());
    assert_eq!(seen.recipients, [BLUE]);
    // A trade at one Marketplace: 10 wood a lot, never a division by the original's 0.
    let lot = rates.resource_lot(1, GOLD, WOOD).unwrap();
    assert_eq!(seen.offers.trades.len(), 2);
    let traded = run(
        &rules,
        &mut game,
        RED,
        answer(MarketAnswer::Trade {
            give: GOLD,
            take: WOOD,
            lots: 2,
        }),
    )
    .unwrap();
    assert_eq!(
        traded,
        [
            GameEvent::ResourcePaid {
                player: RED,
                resource: GOLD,
                amount: 2 * lot.give,
            },
            GameEvent::ResourceGained {
                player: RED,
                resource: WOOD,
                amount: 2 * lot.take,
            },
        ]
    );
    assert_eq!(
        (held(&game, GOLD), held(&game, WOOD)),
        (10_000 - 2 * lot.give, 2 * lot.take)
    );
    // The transfer to blue, 1 : 1.
    let sent = answer(MarketAnswer::Transfer {
        resource: GOLD,
        amount: 100,
        to: BLUE,
    });
    assert_eq!(run(&rules, &mut game, RED, sent).unwrap().len(), 2);
    // No hero: neither the sale nor the purchase (the town has neither building either).
    let sell = answer(MarketAnswer::Sell {
        slot: 0,
        take: GOLD,
        lots: 1,
    });
    let buy = answer(MarketAnswer::Buy {
        entry: 0,
        pay: GOLD,
    });
    let screen = Err(GameCommandError::Market(MarketError::Screen));
    assert_eq!(run(&rules, &mut game, RED, sell), screen);
    assert_eq!(run(&rules, &mut game, RED, buy), screen);
    // The listing offers the trades the treasury pays, no transfer; leaving closes it.
    let legal = crate::legal_commands(&game, &rules, crate::Actor::Player(RED));
    assert!(legal.contains(&answer(MarketAnswer::Trade {
        give: GOLD,
        take: WOOD,
        lots: 1,
    })));
    let left = run(&rules, &mut game, RED, answer(MarketAnswer::Leave)).unwrap();
    assert_eq!(left, [GameEvent::DecisionClosed { player: RED }]);
    assert_eq!(game.market(RED), None);
}

// covers: TWN-MKT-004, TWN-MKT-002, TWN-MKT-001, TWN-STK-001
#[test]
fn the_merchants_sell_the_stock_to_the_visitor_and_an_entry_empties_for_everyone() {
    let rules = rules();
    let rates = rules.adventure().params().market().unwrap();
    let mut game = built(&rules, &[MERCHANTS_SLOT], true);
    let stock = game.adventure().merchants_stock().to_vec();
    assert_eq!(stock.len(), 7);
    assert!(stock.iter().all(Option::is_some));
    let opened = run(&rules, &mut game, RED, market(T0)).unwrap();
    assert_eq!(
        opened,
        [GameEvent::MarketOpened {
            player: RED,
            hero: Some(RED_HERO),
            place: MarketSeen::Town(T0),
        }]
    );
    // No Marketplace: red trades as with one (`TWN-QUIRK-17`), so an artifact of 2 000 gold
    // costs the first row's price, never 0.
    let seen = view(&rules, &game);
    let price = rates.artifact_price(1, 2000, GOLD).unwrap();
    assert!(price > 0);
    assert_eq!(seen.offers.count, 1);
    assert_eq!(
        seen.offers.screens,
        [
            MarketScreen::Trade,
            MarketScreen::Transfer,
            MarketScreen::Buy
        ]
    );
    let shown: Vec<_> = seen.offers.stock.iter().map(|e| e.artifact).collect();
    assert_eq!(shown, stock);
    assert!(seen.offers.stock[0].prices.contains(&(GOLD, price)));
    let buy = |entry| answer(MarketAnswer::Buy { entry, pay: GOLD });
    let legal = crate::legal_commands(&game, &rules, crate::Actor::Player(RED));
    assert!(legal.contains(&buy(0)));
    let bought = run(&rules, &mut game, RED, buy(0)).unwrap();
    assert_eq!(
        bought,
        [
            GameEvent::ResourcePaid {
                player: RED,
                resource: GOLD,
                amount: price,
            },
            GameEvent::HeroChanged {
                player: RED,
                hero: RED_HERO,
                change: HeroChange::AddArtifact(stock[0].unwrap().into()),
            },
        ]
    );
    // The one stock: its entry empty for every town and player until the next month's draw.
    assert_eq!(game.adventure().merchants_stock()[0], None);
    assert_eq!(&game.adventure().merchants_stock()[1..], &stock[1..]);
    let entry = Err(GameCommandError::Market(MarketError::Entry(0)));
    assert_eq!(run(&rules, &mut game, RED, buy(0)), entry);
    assert_eq!(view(&rules, &game).offers.stock[0].artifact, None);
    // A short treasury: refused after the adventure's checks.
    game.set_resource(RED, GOLD, price - 1);
    assert_eq!(
        run(&rules, &mut game, RED, buy(1)),
        Err(GameCommandError::CannotPay)
    );
    // The merchants alone sell nothing without a hero (GENRLTXT 274).
    let mut away = built(&rules, &[MERCHANTS_SLOT], false);
    refused(&rules, &mut away, RED, TownCommandError::NoVisitor);
}

// covers: TWN-MKT-003, TWN-MKT-001
#[test]
fn the_freelancers_guild_buys_the_visitors_creatures_at_the_traders_count() {
    let rules = rules();
    let rates = rules.adventure().params().market().unwrap();
    let mut game = built(&rules, &[GUILD_SLOT, MARKET_SLOT], true);
    run(&rules, &mut game, RED, market(T0)).unwrap();
    let seen = view(&rules, &game);
    assert_eq!(
        seen.offers.screens,
        [
            MarketScreen::Trade,
            MarketScreen::Transfer,
            MarketScreen::Sell
        ]
    );
    let army = game.adventure().hero(RED_HERO).unwrap().army.clone();
    assert_eq!(seen.offers.sales.len(), army.len());
    let first = army[0];
    let cost = rules.core().creature(first.creature).unwrap().cost[0].1;
    let lot = rates.creature_lot(1, cost, GOLD).unwrap();
    let sold = run(
        &rules,
        &mut game,
        RED,
        answer(MarketAnswer::Sell {
            slot: first.slot,
            take: GOLD,
            lots: 1,
        }),
    )
    .unwrap();
    let at = ArmySlot {
        army: ArmyRef::Hero(RED_HERO),
        slot: first.slot,
    };
    assert!(matches!(sold[0], GameEvent::StackChanged { player: RED, at: a, .. } if a == at));
    assert_eq!(
        sold[1],
        GameEvent::ResourceGained {
            player: RED,
            resource: GOLD,
            amount: lot.take,
        }
    );
    let buy = answer(MarketAnswer::Buy {
        entry: 0,
        pay: GOLD,
    });
    let screen = Err(GameCommandError::Market(MarketError::Screen));
    assert_eq!(run(&rules, &mut game, RED, buy), screen);
}

// covers: TWN-MKT-002, TWN-MKT-003
#[test]
fn the_garrison_hero_trades_only_with_the_visitor_only_flag_off() {
    let enter = town_command(TownCommand::Heroes {
        town: T0,
        change: TownHeroChange::Enter,
    });
    // `strict-fixed`'s (T-28 fixed): the garrison hero sells its army at the guild.
    let rules = rules();
    let mut game = built(&rules, &[MERCHANTS_SLOT, GUILD_SLOT], true);
    run(&rules, &mut game, RED, enter.clone()).unwrap();
    assert!(game.adventure().town_visitor(T0).is_none());
    let opened = run(&rules, &mut game, RED, market(T0)).unwrap();
    assert!(matches!(
        opened[..],
        [GameEvent::MarketOpened {
            hero: Some(RED_HERO),
            ..
        }]
    ));
    let screens = view(&rules, &game).offers.screens;
    assert_eq!(screens.len(), 4);
    let stack = game.adventure().town_hero(T0).unwrap().army[0];
    let sell = answer(MarketAnswer::Sell {
        slot: stack.slot,
        take: GOLD,
        lots: 1,
    });
    run(&rules, &mut game, RED, sell).unwrap();
    let after = game.adventure().town_hero(T0).unwrap().army[0];
    assert_eq!((after.slot, after.count), (stack.slot, stack.count - 1));
    // `faithful`'s (the flag on): the visitor alone, so the merchants and the guild open nothing,
    // and a Marketplace's window opens without a hero.
    let visitor = rules_in("t:visitor", vec![10, 15, 20, 25, 30]);
    let mut game = built(&visitor, &[MERCHANTS_SLOT, GUILD_SLOT], true);
    run(&visitor, &mut game, RED, enter.clone()).unwrap();
    refused(&visitor, &mut game, RED, TownCommandError::NoVisitor);
    let mut game = built(&visitor, &[MERCHANTS_SLOT, MARKET_SLOT], true);
    run(&visitor, &mut game, RED, enter.clone()).unwrap();
    let opened = run(&visitor, &mut game, RED, market(T0)).unwrap();
    assert!(matches!(
        opened[..],
        [GameEvent::MarketOpened { hero: None, .. }]
    ));
    assert_eq!(view(&visitor, &game).offers.screens.len(), 2);
}

// covers: TWN-MKT-001
#[test]
fn the_count_is_the_traders_marketplaces_from_one_to_the_parameters_rows() {
    let rules = rules();
    let count =
        |game: &GameState, rules: &GameRuleset| crate::town::market::count(game, rules, RED);
    // Both towns with a Marketplace: red's one, blue's one; none for a player without one.
    let game = built(&rules, &[MARKET_SLOT], true);
    assert_eq!(
        (
            count(&game, &rules),
            crate::town::market::count(&game, &rules, BLUE)
        ),
        (1, 1)
    );
    let none = built(&rules, &[], true);
    assert_eq!(count(&none, &rules), 1);
    // Red owning both: two, but never past the parameters' rows (one row here).
    let mut setup = setup(&rules, &[MARKET_SLOT]);
    setup.map.objects[1].owner = Some(RED);
    let both = GameState::new(&rules, &setup, 9).unwrap();
    assert_eq!(count(&both, &rules), 2);
    let one_row = rules_in("t:faithful", vec![10]);
    let both = GameState::new(&one_row, &setup, 9).unwrap();
    assert_eq!(count(&both, &one_row), 1);
}

// covers: TWN-MKT-002, GAM-DEC-001
#[test]
fn a_window_is_refused_where_no_market_stands_or_the_town_is_not_the_players() {
    let rules = rules();
    let mut game = built(&rules, &[], true);
    refused(&rules, &mut game, RED, TownCommandError::NoMarket);
    let mut game = built(&rules, &[MARKET_SLOT], true);
    let not_owner = run(&rules, &mut game, RED, market(T1));
    assert_eq!(
        not_owner,
        Err(GameCommandError::Town(TownCommandError::NotOwner))
    );
    let unknown = run(&rules, &mut game, RED, market(TownId::new(9)));
    assert_eq!(
        unknown,
        Err(GameCommandError::Town(TownCommandError::UnknownTown))
    );
    assert_eq!(
        run(&rules, &mut game, BLUE, market(T1)),
        Err(GameCommandError::NotYourTurn(BLUE))
    );
    // Open: no second window (nor any other command) until it is left.
    run(&rules, &mut game, RED, market(T0)).unwrap();
    assert_eq!(
        run(&rules, &mut game, RED, market(T0)),
        Err(GameCommandError::DecisionOpen)
    );
}
