//! Tests of the markets' visits and the adventure's checks of their answers (E08 T133a2;
//! increment 14's design note §10, §11): a market entered and its window at every visit
//! (ADV-MKT-001), a sale by whole lots keeping the army a creature (ADV-MKT-005), a purchase at
//! its price, refused before paying when the hero cannot receive the artifact (ADV-MKT-007, A-46
//! fixed as `ADV-QUIRK-48`); since E08 T133b a trade's lot (ADV-MKT-003), a transfer's screen
//! (ADV-MKT-004) and what the window offers (ADV-MKT-008); since E09 T58a1 every check through a
//! resolved window ([`MarketWindow`]: a map market's from its kind, a town's from the game), the
//! Artifact Merchants' stock at a town's window (TWN-MKT-004).

use std::collections::BTreeSet;
use std::string::ToString;
use std::sync::Arc;
use std::vec;
use std::vec::Vec;

use h3_content::{
    ArtifactClass, ArtifactDef, ArtifactSlotDef, ArtifactTypeId, ContentPack, CreatureTypeId,
    HeroParamsDef, ResourceId,
};
use h3_contracts::{ArmyStack, TownId};
use h3_heroes::{ArtifactPlace, HeroDetails, Position};

use super::draws::{object, rules_with};
use super::revisits::{revisit, revisitable};
use super::rewards::{
    AT, CHARM, GOLD, HERO, RED, RING, Rules, WOOD, draw, full, hero, map_of, store, type_of,
};
use super::*;
use crate::army::{ArmyRef, ArmySlot};
use crate::command::{MarketError, MarketPlace, MarketVisit, ServiceVisit};
use crate::market::Lot;
use crate::ruleset::{Market, MarketScreen, ObjectKindDef, ObjectTypeDef, Visit};
use crate::setup::Holds;
use crate::state::Resolved;
use crate::state::{
    ArtifactSaleOffer, MarketOffers, MarketWindow, SaleOffer, StockEntry, TradeOffer,
};
use crate::testing::{GNOLL, PEASANT, PIKEMAN, WOLF, key, stream};

const CLOAK: ArtifactTypeId = ArtifactTypeId::from_raw(3);

/// The draws' rules with hero parameters (a backpack of two), costs in gold (the peasant 25, the
/// pikeman 60, the wolf 4 000 000 000, the gnoll none: 5 wood; the ring 2 000, the charm 1, the
/// cloak none), the market's cost resource gold (ADV-Q60 (a)), and three entered markets at five Marketplaces: `t:black_market` (buy, the base pack's
/// stock), `t:trading_post` (trade, transfer) and `t:guild` (sell).
fn rules() -> Rules {
    market_rules(|_| {})
}

/// [`rules`] with the core changed by `more_core` after its own changes.
fn market_rules(more_core: impl FnOnce(&mut ContentPack)) -> Rules {
    let market = |k: &str, screens: Vec<MarketScreen>, stock: Vec<ArtifactClass>| ObjectTypeDef {
        key: key(k),
        name: "Market".into(),
        visit: Visit::Enter,
        from_above: false,
        kind_mark: false,
        kind: ObjectKindDef::Market(Market {
            screens,
            count: 5,
            stock,
        }),
    };
    use ArtifactClass::{Major, Minor, Treasure};
    rules_with(
        true,
        |core| {
            let gold = |amount| vec![(key("t:gold"), amount)];
            for (creature, cost) in [(PIKEMAN, 60), (PEASANT, 25), (WOLF, 4_000_000_000)] {
                core.creatures[creature.index()].info.cost = gold(cost);
            }
            core.creatures[GNOLL.index()].info.cost = vec![(key("t:wood"), 5)];
            core.artifacts[RING.index()].cost = gold(2000);
            core.artifacts[CHARM.index()].cost = gold(1);
            core.hero_params = Some(HeroParamsDef {
                primary_min: [0, 0, 1, 1],
                primary_max: 99,
                backpack: 2,
                learn_level: 2,
                experience: vec![0, 1000, 2000],
                max_level: 74,
                wisdom: key("t:logistics"),
                schools: vec![],
                mana_ceiling: 999,
                base_mana_regain: 1,
                regain_specialist_bonus: 1,
            });
            more_core(core);
        },
        |types| {
            types.extend([
                market(
                    "t:black_market",
                    vec![MarketScreen::Buy],
                    vec![Treasure, Treasure, Treasure, Minor, Minor, Minor, Major],
                ),
                market(
                    "t:trading_post",
                    vec![MarketScreen::Trade, MarketScreen::Transfer],
                    vec![],
                ),
                market("t:guild", vec![MarketScreen::Sell], vec![]),
            ]);
        },
    )
}

/// The visit of hero 1 to the market of type `k` at object index `index` and [`AT`].
fn visit(rules: &Rules, k: &str, index: usize) -> MarketVisit {
    MarketVisit {
        hero: Some(HERO),
        place: MarketPlace::Object {
            index,
            object_type: type_of(rules, k),
            anchor: AT,
        },
    }
}

/// The window of [`visit`]'s market as its kind gives it: its screens and its count (E09 T58a1;
/// [`AdventureState::market_window`] resolves the same for a state that holds the market).
fn window<'r>(rules: &'r Rules, k: &str, index: usize) -> MarketWindow<'r> {
    let kind = rules.1.object_type(type_of(rules, k)).map(|ty| &ty.kind);
    let Some(ObjectKind::Market(market)) = kind else {
        panic!("{k} is no market");
    };
    MarketWindow {
        visit: visit(rules, k, index),
        screens: &market.screens,
        count: market.count,
    }
}

/// A stack.
fn stack(slot: u8, creature: CreatureTypeId, count: u32) -> ArmyStack {
    ArmyStack {
        slot,
        creature,
        count,
    }
}

/// [`map_of`] with a market of type `k` at [`AT`] and hero 1's army `army`.
fn at_market(rules: &Rules, k: &str, army: Vec<ArmyStack>) -> AdventureState {
    let mut state = map_of(rules, vec![object(rules, AT.x, k, Holds::Nothing)]);
    state.hero_mut(HERO).unwrap().army = army;
    state
}

// covers: ADV-MKT-001, ADV-OBJ-003, ADV-TEL-008
#[test]
fn a_markets_step_ends_the_move_and_opens_its_window_at_every_visit() {
    let rules = rules();
    for k in ["t:black_market", "t:trading_post", "t:guild"] {
        let mut state = map_of(&rules, vec![object(&rules, AT.x, k, Holds::Nothing)]);
        let heroes = store(hero());
        let window = ServiceVisit::Market(visit(&rules, k, 0));
        let outcome = rewards::visit(&rules, &mut state, &heroes);
        let opened = AdventureOutcome {
            service: Some(window),
            ..AdventureOutcome::default()
        };
        assert_eq!(outcome, opened, "{k}");
        // Entered: the hero stands on the entrance, the move ended there.
        assert_eq!(state.hero(HERO).unwrap().pos, AT, "{k}");
        // Every visit opens it again: no mark, no owner, no day.
        assert_eq!(revisitable(&state, &rules, RED), [HERO], "{k}");
        let (again, events) = revisit(&mut state, &rules, (RED, HERO), &mut stream()).unwrap();
        assert_eq!(again, opened, "{k}");
        assert!(events.is_empty(), "{k}");
        assert_eq!(state.visits(), None, "{k}");
    }
}

// covers: ADV-MKT-005, ADV-MKT-002
#[test]
fn a_sale_takes_whole_lots_and_leaves_the_army_a_creature() {
    let rules = rules();
    let rules2 = (&rules.0, &rules.1);
    let army = vec![
        stack(0, PEASANT, 40),
        stack(2, PIKEMAN, 3),
        stack(3, GNOLL, 5),
    ];
    let mut state = at_market(&rules, "t:guild", army);
    let guild = window(&rules, "t:guild", 0);
    // A peasant of 25 gold at five Marketplaces: 17 gold each (survey §4's 25 → 17), or 14 for
    // one wood; so many lots of the creatures, or of the wood.
    let sale = state.check_sale(rules2, &guild, (0, GOLD, 40)).unwrap();
    assert_eq!((sale.count(), sale.gain()), (40, (GOLD, 680)));
    let wood = state.check_sale(rules2, &guild, (0, WOOD, 2)).unwrap();
    assert_eq!((wood.count(), wood.gain()), (28, (WOOD, 2)));
    for lots in [0, 3] {
        let refused = state.check_sale(rules2, &guild, (0, WOOD, lots));
        assert_eq!(refused, Err(MarketError::Amount(lots)));
    }
    assert_eq!(
        state.check_sale(rules2, &guild, (0, GOLD, 41)),
        Err(MarketError::Amount(41))
    );
    // An empty slot, a creature of no cost in the market's cost resource (the gnoll's 5 wood are
    // not read: ADV-Q60 (a), gold).
    assert_eq!(
        state.check_sale(rules2, &guild, (1, GOLD, 1)),
        Err(MarketError::Slot(1))
    );
    assert_eq!(
        state.check_sale(rules2, &guild, (3, GOLD, 1)),
        Err(MarketError::NoPrice)
    );
    // Sold whole, the slot empties; the slot that changed is the hero's.
    let slot = state.sell(sale);
    let changed = ArmySlot {
        army: ArmyRef::Hero(HERO),
        slot: 0,
    };
    assert_eq!(slot, changed);
    let after = vec![stack(2, PIKEMAN, 3), stack(3, GNOLL, 5)];
    assert_eq!(state.hero(HERO).unwrap().army, after);
    // A part of a stack: its count goes down.
    let two = state.check_sale(rules2, &guild, (2, GOLD, 2)).unwrap();
    assert_eq!(two.gain(), (GOLD, 84));
    state.sell(two);
    let after = vec![stack(2, PIKEMAN, 1), stack(3, GNOLL, 5)];
    assert_eq!(state.hero(HERO).unwrap().army, after);
    // The army's lone stack keeps a creature: its count less one is the most.
    state.hero_mut(HERO).unwrap().army = vec![stack(2, PIKEMAN, 3)];
    assert_eq!(
        state.check_sale(rules2, &guild, (2, GOLD, 3)),
        Err(MarketError::LastCreature)
    );
    let sale = state.check_sale(rules2, &guild, (2, GOLD, 2)).unwrap();
    state.sell(sale);
    assert_eq!(state.hero(HERO).unwrap().army, [stack(2, PIKEMAN, 1)]);
    // A gain past 32 bits saturates (a wolf of four billion gold: 2 800 000 000 each).
    state.hero_mut(HERO).unwrap().army = vec![stack(0, WOLF, 5)];
    let rich = state.check_sale(rules2, &guild, (0, GOLD, 2)).unwrap();
    assert_eq!(rich.gain(), (GOLD, u32::MAX));
}

// covers: ADV-MKT-007, ADV-MKT-002, ADV-MKT-006
#[test]
fn a_purchase_pays_the_price_gives_the_artifact_and_empties_the_entry() {
    let rules = rules();
    let mut state = at_market(&rules, "t:black_market", vec![stack(0, PEASANT, 1)]);
    draw(
        &mut state,
        0,
        Resolved::Stock(vec![RING, CHARM, CLOAK]),
        &[],
    );
    let market = window(&rules, "t:black_market", 0);
    let heroes = store(hero());
    let rules3 = (&rules.0, &rules.1, &heroes[..]);
    // The ring of 2 000 at five Marketplaces: 5 000 gold (batch E14's E14-4), 40 wood.
    let purchase = state.check_buy(rules3, &market, (0, GOLD)).unwrap();
    assert_eq!(
        (purchase.price(), purchase.artifact(), purchase.hero()),
        ((GOLD, 5000), RING, HERO)
    );
    let wood = state.check_buy(rules3, &market, (0, WOOD)).unwrap();
    assert_eq!(wood.price(), (WOOD, 40));
    // The charm of 1 gold: 2.5 at binary32, rounded half up.
    let charm = state.check_buy(rules3, &market, (1, GOLD)).unwrap();
    assert_eq!(charm.price(), (GOLD, 3));
    // An entry past the stock, an artifact without a price in the resource.
    assert_eq!(
        state.check_buy(rules3, &market, (3, GOLD)),
        Err(MarketError::Entry(3))
    );
    assert_eq!(
        state.check_buy(rules3, &market, (2, GOLD)),
        Err(MarketError::NoPrice)
    );
    // Bought: the entry is sold for everyone, never again.
    state.buy(purchase);
    assert!(state.is_sold(0, 0) && !state.is_sold(0, 1));
    assert_eq!(
        state.check_buy(rules3, &market, (0, GOLD)),
        Err(MarketError::Entry(0))
    );
    assert_eq!(
        state.map().objects()[0].resolved.stock(),
        [RING, CHARM, CLOAK]
    );
}

// covers: ADV-MKT-007, HER-CHG-002
#[test]
fn a_purchase_is_refused_before_paying_when_the_hero_cannot_receive_the_artifact() {
    let rules = rules();
    let mut state = at_market(&rules, "t:black_market", vec![stack(0, PEASANT, 1)]);
    draw(&mut state, 0, Resolved::Stock(vec![RING]), &[]);
    let market = window(&rules, "t:black_market", 0);
    let buy = |heroes: &[Arc<HeroDetails>]| {
        state.check_buy((&rules.0, &rules.1, heroes), &market, (0, GOLD))
    };
    // A free finger with a full backpack takes it (not the pickups' count, A-25), as a worn
    // finger with room in the backpack does.
    assert!(buy(&store(full(hero()))).is_ok());
    let worn = HeroDetails {
        worn: vec![Some(CHARM.into())],
        ..hero()
    };
    assert!(buy(&store(worn.clone())).is_ok());
    // Neither: refused (the original paid and lost it, A-46).
    assert_eq!(buy(&store(full(worn))), Err(MarketError::NoRoom));
    // A hero the store does not hold has no room either (defensive).
    assert_eq!(buy(&store(hero())[..1]), Err(MarketError::NoRoom));
}

// covers: ADV-MKT-005, ADV-MKT-007
#[test]
fn a_markets_answer_is_refused_for_another_screen_or_a_window_that_is_not_one() {
    let rules = rules();
    let army = vec![stack(0, PEASANT, 4), stack(1, PEASANT, 4)];
    let mut state = at_market(&rules, "t:black_market", army);
    draw(&mut state, 0, Resolved::Stock(vec![RING]), &[]);
    let heroes = store(hero());
    let (rules2, rules3) = ((&rules.0, &rules.1), (&rules.0, &rules.1, &heroes[..]));
    let market = window(&rules, "t:black_market", 0);
    // The Black Market sells nothing it buys; a guild's window buys nothing.
    assert_eq!(
        state.check_sale(rules2, &market, (0, GOLD, 1)),
        Err(MarketError::Screen)
    );
    let guild = at_market(&rules, "t:guild", vec![stack(0, PEASANT, 4)]);
    let sells = window(&rules, "t:guild", 0);
    assert_eq!(
        guild.check_buy(rules3, &sells, (0, GOLD)),
        Err(MarketError::Screen)
    );
    // The map's window of its visit is its kind's; none for a hero not on the map or none at
    // all, an index past the objects, an object that is not a market, a town's place (E09 T58a1:
    // the game resolves a town's window).
    let seen = market.visit;
    assert_eq!(state.market_window(&rules.1, &seen), Ok(market));
    let elsewhere = [
        MarketVisit {
            hero: Some(HeroId::new(9)),
            ..seen
        },
        MarketVisit { hero: None, ..seen },
        visit(&rules, "t:black_market", 5),
        MarketVisit {
            place: MarketPlace::Town(TownId::new(0)),
            ..seen
        },
    ];
    for visit in elsewhere {
        assert_eq!(
            state.market_window(&rules.1, &visit),
            Err(MarketError::Visit)
        );
    }
    let other = map_of(
        &rules,
        vec![object(
            &rules,
            AT.x,
            "t:artifact",
            Holds::Artifact(RING.into()),
        )],
    );
    assert_eq!(
        other.market_window(&rules.1, &seen),
        Err(MarketError::Visit)
    );
    // A window whose hero is gone or that has none sells nothing and gives nothing (defensive: a
    // town's window shows neither screen without a hero).
    let both = MarketWindow {
        screens: &[MarketScreen::Sell, MarketScreen::Buy],
        ..market
    };
    let gone = MarketWindow {
        visit: MarketVisit {
            hero: Some(HeroId::new(9)),
            ..seen
        },
        ..both
    };
    let heroless = MarketWindow {
        visit: MarketVisit { hero: None, ..seen },
        ..both
    };
    for window in [gone, heroless] {
        assert_eq!(
            state.check_sale(rules2, &window, (0, GOLD, 1)),
            Err(MarketError::Visit)
        );
    }
    assert_eq!(
        state.check_buy(rules3, &heroless, (0, GOLD)),
        Err(MarketError::Visit)
    );
    assert_eq!(
        state.check_buy(rules3, &gone, (0, GOLD)),
        Err(MarketError::NoRoom)
    );
}

#[test]
fn every_market_error_reads_apart() {
    let all = [
        MarketError::Visit,
        MarketError::Screen,
        MarketError::SameResource,
        MarketError::Amount(2),
        MarketError::Recipient,
        MarketError::Slot(3),
        MarketError::LastCreature,
        MarketError::Entry(4),
        MarketError::NoPrice,
        MarketError::NoRoom,
        MarketError::Place(ArtifactPlace::Worn(Position(3))),
        MarketError::Place(ArtifactPlace::Backpack(3)),
        MarketError::NotTradable,
    ];
    let texts: BTreeSet<_> = all.iter().map(ToString::to_string).collect();
    assert_eq!(texts.len(), all.len());
    assert!(texts.iter().all(|t| !t.is_empty()));
}

// covers: ADV-MKT-003, ADV-MKT-004, ADV-MKT-002
#[test]
fn a_trades_lot_is_the_rates_and_a_transfer_needs_its_screen() {
    let rules = rules();
    let part = &rules.1;
    let rates = part.params().market().unwrap();
    let state = at_market(&rules, "t:trading_post", vec![stack(0, PEASANT, 1)]);
    let post = window(&rules, "t:trading_post", 0);
    // The rate at the kind's five Marketplaces, each way.
    for (give, take) in [(WOOD, GOLD), (GOLD, WOOD)] {
        assert_eq!(
            state.trade_lot(part, &post, (give, take)),
            Ok(rates.resource_lot(5, give, take).unwrap())
        );
    }
    // A resource for itself, then one the parameters do not value.
    assert_eq!(
        state.trade_lot(part, &post, (GOLD, GOLD)),
        Err(MarketError::SameResource)
    );
    let unknown = ResourceId::from_raw(7);
    assert_eq!(
        state.trade_lot(part, &post, (WOOD, unknown)),
        Err(MarketError::NoPrice)
    );
    // The transfer's screen is the Trading Post's, not the guild's.
    assert_eq!(post.check_screen(MarketScreen::Transfer), Ok(()));
    assert_eq!(
        post.check_screen(MarketScreen::Sell),
        Err(MarketError::Screen)
    );
    let guild = at_market(&rules, "t:guild", vec![stack(0, PEASANT, 1)]);
    let sells = window(&rules, "t:guild", 0);
    assert_eq!(
        guild.trade_lot(part, &sells, (WOOD, GOLD)),
        Err(MarketError::Screen)
    );
}

// covers: ADV-MKT-008, ADV-MKT-002, ADV-MKT-005, ADV-MKT-007
#[test]
fn a_markets_window_offers_the_rates_of_its_screens_alone() {
    let rules = rules();
    let heroes = store(hero());
    let rules2 = (&rules.0, &rules.1, &heroes[..]);
    let rates = rules.1.params().market().unwrap();
    // A Trading Post: both trades' lots, no sale, no stock.
    let post = at_market(&rules, "t:trading_post", vec![stack(0, PEASANT, 40)]);
    let offers = post.market_offers(rules2, &window(&rules, "t:trading_post", 0));
    let trade = |give, take| TradeOffer {
        give,
        take,
        lot: rates.resource_lot(5, give, take).unwrap(),
    };
    assert_eq!(
        offers,
        MarketOffers {
            screens: vec![MarketScreen::Trade, MarketScreen::Transfer],
            count: 5,
            trades: vec![trade(WOOD, GOLD), trade(GOLD, WOOD)],
            sales: vec![],
            stock: vec![],
            artifacts: vec![],
        }
    );
    // The guild: each stack's lot per resource; the gnoll's 5 wood are no gold cost, so none.
    let army = vec![stack(0, PEASANT, 40), stack(3, GNOLL, 5)];
    let guild = at_market(&rules, "t:guild", army);
    let sells = window(&rules, "t:guild", 0);
    let offers = guild.market_offers(rules2, &sells);
    let peasant = |take| (take, rates.creature_lot(5, 25, take).unwrap());
    assert_eq!(peasant(GOLD).1, Lot { give: 1, take: 17 });
    let sales = vec![
        SaleOffer {
            slot: 0,
            lots: vec![peasant(WOOD), peasant(GOLD)],
        },
        SaleOffer {
            slot: 3,
            lots: vec![],
        },
    ];
    assert_eq!((offers.trades, offers.sales), (vec![], sales));
    // The Black Market: its stock in order, a sold entry empty, an artifact of no cost unpriced.
    let mut market = at_market(&rules, "t:black_market", vec![stack(0, PEASANT, 1)]);
    draw(
        &mut market,
        0,
        Resolved::Stock(vec![RING, CHARM, CLOAK]),
        &[],
    );
    market.visits_mut().sell(0, 0);
    let visit = window(&rules, "t:black_market", 0);
    let offers = market.market_offers(rules2, &visit);
    let prices = |cost| {
        [WOOD, GOLD]
            .into_iter()
            .filter_map(|pay| rates.artifact_price(5, cost, pay).map(|p| (pay, p)))
            .collect::<Vec<_>>()
    };
    assert!(prices(1).contains(&(GOLD, 3)));
    let stock = vec![
        StockEntry {
            artifact: None,
            prices: vec![],
        },
        StockEntry {
            artifact: Some(CHARM),
            prices: prices(1),
        },
        StockEntry {
            artifact: Some(CLOAK),
            prices: vec![],
        },
    ];
    assert_eq!(
        (offers.screens, offers.sales, offers.stock),
        (vec![MarketScreen::Buy], vec![], stock)
    );
    // A window without a hero offers no sale (E09 T58a1: a town's Marketplace alone).
    let heroless = MarketWindow {
        visit: MarketVisit {
            hero: None,
            ..sells.visit
        },
        ..sells
    };
    assert_eq!(guild.market_offers(rules2, &heroless).sales, vec![]);
}

// covers: TWN-MKT-004, TWN-MKT-003, ADV-MKT-008
#[test]
fn a_towns_window_trades_at_its_count_and_sells_the_merchants_stock() {
    let rules = rules();
    let (rules2, heroes) = ((&rules.0, &rules.1), store(hero()));
    let rules3 = (&rules.0, &rules.1, &heroes[..]);
    let rates = rules.1.params().market().unwrap();
    let mut state = at_market(&rules, "t:trading_post", vec![stack(0, PEASANT, 4)]);
    let classes = [ArtifactClass::Treasure, ArtifactClass::Treasure];
    state.draw_merchants(&rules.0, &classes, &mut stream());
    let stock = state.merchants_stock().to_vec();
    // A town's window as the game resolves it: every screen, the trader's two Marketplaces.
    let town = MarketWindow {
        visit: MarketVisit {
            hero: Some(HERO),
            place: MarketPlace::Town(TownId::new(0)),
        },
        screens: &[
            MarketScreen::Trade,
            MarketScreen::Transfer,
            MarketScreen::Buy,
            MarketScreen::Sell,
        ],
        count: 2,
    };
    assert_eq!(
        state.trade_lot(&rules.1, &town, (WOOD, GOLD)),
        Ok(rates.resource_lot(2, WOOD, GOLD).unwrap())
    );
    let offers = state.market_offers(rules3, &town);
    assert_eq!(offers.count, 2);
    assert_eq!(offers.sales.len(), 1);
    let entries: Vec<_> = offers.stock.iter().map(|e| e.artifact).collect();
    assert_eq!(entries, stock);
    let sale = state.check_sale(rules2, &town, (0, GOLD, 1)).unwrap();
    let lot = rates.creature_lot(2, 25, GOLD).unwrap();
    assert_eq!(sale.gain(), (GOLD, lot.take));
    // The purchase empties its entry of the one stock, for every town; the rest stays. The entry
    // bought is one with a gold price (the draw's treasures: the ring, the charm, the cloak of
    // none).
    let priced = offers.stock.iter().position(|e| !e.prices.is_empty());
    let at = priced.unwrap();
    let entry = u8::try_from(at).unwrap();
    let purchase = state.check_buy(rules3, &town, (entry, GOLD)).unwrap();
    assert_eq!(
        (purchase.artifact(), purchase.hero()),
        (stock[at].unwrap(), HERO)
    );
    state.buy(purchase);
    let mut emptied = stock.clone();
    emptied[at] = None;
    assert_eq!(state.merchants_stock(), emptied);
    assert_eq!(
        state.check_buy(rules3, &town, (entry, GOLD)),
        Err(MarketError::Entry(entry))
    );
    assert_eq!(
        state.check_buy(rules3, &town, (2, GOLD)),
        Err(MarketError::Entry(2))
    );
    let shown = state.market_offers(rules3, &town).stock;
    assert_eq!((shown[at].artifact, shown[at].prices.len()), (None, 0));
    // An entry past the stock empties nothing (the purchase's check finds it first).
    let before = state.merchants_stock().to_vec();
    state.empty_merchants_entry(5);
    assert_eq!(state.merchants_stock(), before);
}

/// The catapult's handle in [`sale_rules`]' core: the artifact its machine slot takes (not
/// carried in the backpack: HER-ART-002), after the draws' seven.
const CATAPULT: ArtifactTypeId = ArtifactTypeId::from_raw(7);

/// [`rules`] with a second artifact slot, `t:machine` (one position after the finger's, its
/// artifacts not carried in the backpack), and the catapult worn there.
fn sale_rules() -> Rules {
    market_rules(|core| {
        core.artifact_slots.push(ArtifactSlotDef {
            key: key("t:machine"),
            name: "Machine".into(),
            count: 1,
            backpack: false,
        });
        core.artifacts.push(ArtifactDef {
            key: key("t:catapult"),
            name: "Catapult".into(),
            class: ArtifactClass::Special,
            cost: vec![(key("t:gold"), 2_500)],
            slot: Some(key("t:machine")),
            war_machine: None,
            needs_skill: None,
            effects: vec![],
            parts: vec![],
        });
    })
}

// covers: TWN-MKT-005, ADV-MKT-002, ADV-MKT-008
#[test]
fn a_towns_window_buys_its_heros_artifact_at_the_place_it_names() {
    let rules = sale_rules();
    assert_eq!(
        rules.0.artifact(CATAPULT).map(|ty| ty.key.as_str()),
        Some("t:catapult")
    );
    let rates = rules.1.params().market().unwrap();
    // The hero wears the ring and the catapult, carries the charm (1 gold) and the cloak (no cost).
    let details = HeroDetails {
        worn: vec![Some(RING.into()), Some(CATAPULT.into())],
        backpack: vec![CHARM.into(), CLOAK.into()],
        ..hero()
    };
    let heroes = store(details);
    let rules3 = (&rules.0, &rules.1, &heroes[..]);
    let state = at_market(&rules, "t:trading_post", vec![stack(0, PEASANT, 4)]);
    let town = MarketWindow {
        visit: MarketVisit {
            hero: Some(HERO),
            place: MarketPlace::Town(TownId::new(0)),
        },
        screens: &[
            MarketScreen::Trade,
            MarketScreen::Transfer,
            MarketScreen::Buy,
            MarketScreen::SellArtifact,
        ],
        count: 1,
    };
    let sell = |place, take| state.check_artifact_sale(rules3, &town, (place, take));
    let worn = |at| ArtifactPlace::Worn(Position(at));
    // The ring (2 000) at one Marketplace: f32(2 000 × 0.2) / 1 gold, / 250 wood rounded half up.
    let ring = sell(worn(0), GOLD).unwrap();
    assert_eq!(
        (ring.hero(), ring.place(), ring.gain()),
        (HERO, worn(0), (GOLD, 400))
    );
    assert_eq!(sell(worn(0), WOOD).map(|s| s.gain()), Ok((WOOD, 2)));
    // At least one unit: the charm (1 gold), the cloak of no cost (TWN-MKT-005).
    let backpack = ArtifactPlace::Backpack;
    assert_eq!(sell(backpack(0), GOLD).map(|s| s.gain()), Ok((GOLD, 1)));
    assert_eq!(sell(backpack(1), WOOD).map(|s| s.gain()), Ok((WOOD, 1)));
    // Refused: the catapult's slot is not carried; no artifact at a place; no price.
    assert_eq!(sell(worn(1), GOLD), Err(MarketError::NotTradable));
    assert_eq!(sell(worn(2), GOLD), Err(MarketError::Place(worn(2))));
    assert_eq!(
        sell(backpack(2), GOLD),
        Err(MarketError::Place(backpack(2)))
    );
    let unvalued = ResourceId::from_raw(7);
    assert_eq!(sell(worn(0), unvalued), Err(MarketError::NoPrice));
    // The window's offers: each place that sells in place order, its price per resource.
    let prices = |cost| {
        [WOOD, GOLD]
            .into_iter()
            .filter_map(|take| rates.artifact_sale(1, cost, take).map(|p| (take, p)))
            .collect::<Vec<_>>()
    };
    let offer = |place, artifact, cost| ArtifactSaleOffer {
        place,
        artifact,
        prices: prices(cost),
    };
    let offers = state.market_offers(rules3, &town);
    assert_eq!(
        offers.artifacts,
        [
            offer(worn(0), RING, 2_000),
            offer(backpack(0), CHARM, 1),
            offer(backpack(1), CLOAK, 0),
        ]
    );
    // A window without the screen: refused and no offer; a window without a hero or with one the
    // store lacks (defensive).
    let merchants_less = MarketWindow {
        screens: &[MarketScreen::Trade, MarketScreen::Transfer],
        ..town
    };
    assert_eq!(
        state.check_artifact_sale(rules3, &merchants_less, (worn(0), GOLD)),
        Err(MarketError::Screen)
    );
    assert_eq!(state.market_offers(rules3, &merchants_less).artifacts, []);
    for hero in [None, Some(HeroId::new(9))] {
        let gone = MarketWindow {
            visit: MarketVisit { hero, ..town.visit },
            ..town
        };
        assert_eq!(
            state.check_artifact_sale(rules3, &gone, (worn(0), GOLD)),
            Err(MarketError::Visit)
        );
        assert_eq!(state.market_offers(rules3, &gone).artifacts, []);
    }
}
