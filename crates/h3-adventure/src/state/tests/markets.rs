//! Tests of the markets' visits and the adventure's checks of their answers (E08 T133a2;
//! increment 14's design note §10, §11): a market entered and its window at every visit
//! (ADV-MKT-001), a sale by whole lots keeping the army a creature (ADV-MKT-005), a purchase at
//! its price, refused before paying when the hero cannot receive the artifact (ADV-MKT-007, A-46
//! fixed as `ADV-QUIRK-48`).

use std::collections::BTreeSet;
use std::string::ToString;
use std::sync::Arc;
use std::vec;
use std::vec::Vec;

use h3_content::{ArtifactClass, ArtifactTypeId, CreatureTypeId, HeroParamsDef};
use h3_contracts::ArmyStack;
use h3_heroes::HeroDetails;

use super::draws::{object, rules_with};
use super::revisits::{revisit, revisitable};
use super::rewards::{
    AT, CHARM, GOLD, HERO, RED, RING, Rules, WOOD, draw, full, hero, map_of, store, type_of,
};
use super::*;
use crate::army::{ArmyRef, ArmySlot};
use crate::command::{MarketError, MarketVisit, ServiceVisit};
use crate::ruleset::{Market, MarketScreen, ObjectKindDef, ObjectTypeDef, Visit};
use crate::setup::Holds;
use crate::state::Resolved;
use crate::testing::{GNOLL, PEASANT, PIKEMAN, WOLF, key, stream};

const CLOAK: ArtifactTypeId = ArtifactTypeId::from_raw(3);

/// The draws' rules with hero parameters (a backpack of two), costs in gold (the peasant 25, the
/// pikeman 60, the wolf 4 000 000 000, the gnoll none; the ring 2 000, the charm 1, the cloak
/// none) and three entered markets at five Marketplaces: `t:black_market` (buy, the base pack's
/// stock), `t:trading_post` (trade, transfer) and `t:guild` (sell).
fn rules() -> Rules {
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

/// The window of the market of type `k` at object index `index` and [`AT`] for hero 1.
fn window(rules: &Rules, k: &str, index: usize) -> MarketVisit {
    MarketVisit {
        hero: HERO,
        index,
        object_type: type_of(rules, k),
        anchor: AT,
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
        let window = ServiceVisit::Market(window(&rules, k, 0));
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
    let sale = state
        .check_sale(rules2, GOLD, &guild, (0, GOLD, 40))
        .unwrap();
    assert_eq!((sale.count(), sale.gain()), (40, (GOLD, 680)));
    let wood = state
        .check_sale(rules2, GOLD, &guild, (0, WOOD, 2))
        .unwrap();
    assert_eq!((wood.count(), wood.gain()), (28, (WOOD, 2)));
    for lots in [0, 3] {
        let refused = state.check_sale(rules2, GOLD, &guild, (0, WOOD, lots));
        assert_eq!(refused, Err(MarketError::Amount(lots)));
    }
    assert_eq!(
        state.check_sale(rules2, GOLD, &guild, (0, GOLD, 41)),
        Err(MarketError::Amount(41))
    );
    // An empty slot, a creature of no gold cost.
    assert_eq!(
        state.check_sale(rules2, GOLD, &guild, (1, GOLD, 1)),
        Err(MarketError::Slot(1))
    );
    assert_eq!(
        state.check_sale(rules2, GOLD, &guild, (3, GOLD, 1)),
        Err(MarketError::NoPrice)
    );
    // Valued in wood instead of gold, the peasant has no cost.
    assert_eq!(
        state.check_sale(rules2, WOOD, &guild, (0, GOLD, 1)),
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
    let two = state
        .check_sale(rules2, GOLD, &guild, (2, GOLD, 2))
        .unwrap();
    assert_eq!(two.gain(), (GOLD, 84));
    state.sell(two);
    let after = vec![stack(2, PIKEMAN, 1), stack(3, GNOLL, 5)];
    assert_eq!(state.hero(HERO).unwrap().army, after);
    // The army's lone stack keeps a creature: its count less one is the most.
    state.hero_mut(HERO).unwrap().army = vec![stack(2, PIKEMAN, 3)];
    assert_eq!(
        state.check_sale(rules2, GOLD, &guild, (2, GOLD, 3)),
        Err(MarketError::LastCreature)
    );
    let sale = state
        .check_sale(rules2, GOLD, &guild, (2, GOLD, 2))
        .unwrap();
    state.sell(sale);
    assert_eq!(state.hero(HERO).unwrap().army, [stack(2, PIKEMAN, 1)]);
    // A gain past 32 bits saturates (a wolf of four billion gold: 2 800 000 000 each).
    state.hero_mut(HERO).unwrap().army = vec![stack(0, WOLF, 5)];
    let rich = state
        .check_sale(rules2, GOLD, &guild, (0, GOLD, 2))
        .unwrap();
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
    let purchase = state.check_buy(rules3, GOLD, &market, (0, GOLD)).unwrap();
    assert_eq!(
        (purchase.price(), purchase.artifact(), purchase.hero()),
        ((GOLD, 5000), RING, HERO)
    );
    let wood = state.check_buy(rules3, GOLD, &market, (0, WOOD)).unwrap();
    assert_eq!(wood.price(), (WOOD, 40));
    // The charm of 1 gold: 2.5 at binary32, rounded half up.
    let charm = state.check_buy(rules3, GOLD, &market, (1, GOLD)).unwrap();
    assert_eq!(charm.price(), (GOLD, 3));
    // An entry past the stock, an artifact without a price in the resource.
    assert_eq!(
        state.check_buy(rules3, GOLD, &market, (3, GOLD)),
        Err(MarketError::Entry(3))
    );
    assert_eq!(
        state.check_buy(rules3, GOLD, &market, (2, GOLD)),
        Err(MarketError::NoPrice)
    );
    // Bought: the entry is sold for everyone, never again.
    state.buy(purchase);
    assert!(state.is_sold(0, 0) && !state.is_sold(0, 1));
    assert_eq!(
        state.check_buy(rules3, GOLD, &market, (0, GOLD)),
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
        state.check_buy((&rules.0, &rules.1, heroes), GOLD, &market, (0, GOLD))
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
        state.check_sale(rules2, GOLD, &market, (0, GOLD, 1)),
        Err(MarketError::Screen)
    );
    let guild = at_market(&rules, "t:guild", vec![stack(0, PEASANT, 4)]);
    let sells = window(&rules, "t:guild", 0);
    assert_eq!(
        guild.check_buy(rules3, GOLD, &sells, (0, GOLD)),
        Err(MarketError::Screen)
    );
    // A hero not on the map, an index past the objects, an object that is not a market.
    let elsewhere = [
        MarketVisit {
            hero: HeroId::new(9),
            ..market
        },
        MarketVisit { index: 5, ..market },
    ];
    for visit in elsewhere {
        assert_eq!(
            state.check_sale(rules2, GOLD, &visit, (0, GOLD, 1)),
            Err(MarketError::Visit)
        );
        assert_eq!(
            state.check_buy(rules3, GOLD, &visit, (0, GOLD)),
            Err(MarketError::Visit)
        );
    }
    let mut other = map_of(
        &rules,
        vec![object(
            &rules,
            AT.x,
            "t:artifact",
            Holds::Artifact(RING.into()),
        )],
    );
    other.hero_mut(HERO).unwrap().army = vec![stack(0, PEASANT, 4)];
    assert_eq!(
        other.check_sale(rules2, GOLD, &market, (0, GOLD, 1)),
        Err(MarketError::Visit)
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
    ];
    let texts: BTreeSet<_> = all.iter().map(ToString::to_string).collect();
    assert_eq!(texts.len(), all.len());
    assert!(texts.iter().all(|t| !t.is_empty()));
}
