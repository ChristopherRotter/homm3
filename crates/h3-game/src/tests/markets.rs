//! Tests of the market's decision (GAM-DEC-011; E08 T133b, increment 14's design note §13): a
//! market's visit opens it for its visitor's owner alone, it blocks the turn until its `Leave`
//! (the system's default), every other answer keeps it open; a trade by whole lots (ADV-MKT-003),
//! a transfer 1 : 1 to another player (ADV-MKT-004), a sale through the adventure's check
//! (ADV-MKT-005), a purchase refused before paying (ADV-MKT-007); the view with the rates
//! (ADV-MKT-008). A rejected answer changes nothing ([`play`] checks it).

use h3_adventure::{Holds, Market, MarketParamsDef, ObjectKindDef};
use h3_content::{ArtifactClass, ResourceDef, ResourceId};

use super::decisions::{BLUE, GOLD, RED, move_hero, play};
use super::rewards::{AT, HERO, answered, changed, reward_at, visit};
use super::*;

const WOOD: ResourceId = ResourceId::from_raw(1);

/// [`rules`] with wood beside the gold (and its mine), every artifact at 2 000 gold and a minor
/// one but the ring (the one treasure), and three entered markets at five Marketplaces: `t:post`
/// (trade, transfer), `t:guild` (sell) and `t:black` (buy: one treasure, the ring), with the base
/// pack's rates for the two resources.
fn market_rules() -> GameRuleset {
    let mut pack = core_pack(false);
    pack.resources.push(ResourceDef {
        key: key("t:wood"),
        name: "Wood".into(),
    });
    for artifact in &mut pack.artifacts {
        artifact.cost = vec![(key("t:gold"), 2000)];
        if artifact.key != key("t:ring") {
            artifact.class = ArtifactClass::Minor;
        }
    }
    let core = Arc::new(Ruleset::compile(&[pack], "t:faithful").unwrap());
    let mut adventure = adventure_pack(vec![88, 25], 2);
    let market = |k: &str, screens, stock| {
        entered(
            k,
            ObjectKindDef::Market(Market {
                screens,
                count: 5,
                stock,
            }),
        )
    };
    adventure.object_types.extend([
        // The abandoned mine's check: a mine of every core resource.
        entered(
            "t:wood_mine",
            ObjectKindDef::Mine {
                resource: key("t:wood"),
                daily: 2,
                battle_terrain: None,
            },
        ),
        market(
            "t:post",
            vec![MarketScreen::Trade, MarketScreen::Transfer],
            vec![],
        ),
        market("t:guild", vec![MarketScreen::Sell], vec![]),
        market(
            "t:black",
            vec![MarketScreen::Buy],
            vec![ArtifactClass::Treasure],
        ),
    ]);
    adventure.params.as_mut().unwrap().market = Some(MarketParamsDef {
        cost: key("t:gold"),
        values: vec![(key("t:wood"), 250), (key("t:gold"), 1)],
        artifact_values: vec![(key("t:wood"), 125), (key("t:gold"), 1)],
        resources: vec![10, 15, 20, 25, 30],
        artifacts: vec![20, 25, 30, 35, 40],
        creatures: vec![30, 45, 50, 65, 70],
    });
    let part = AdventureRuleset::compile(&core, &[adventure]).unwrap();
    let battle = battle_part(&core, 7);
    GameRuleset::new(
        Arc::clone(&core),
        Arc::new(part),
        Arc::new(battle),
        no_towns(&core),
    )
    .unwrap()
}

/// A game with a market of type `k` at [`AT`], red's hero visiting it: the game and the events of
/// the visit.
fn at_market(rules: &GameRuleset, k: &str) -> (GameState, Vec<GameEvent>) {
    let mut setup = setup();
    setup.map.objects = vec![reward_at(rules, k, Holds::Nothing)];
    let mut state = GameState::new(rules, &setup, 7).unwrap();
    let events = visit(&mut state, rules);
    (state, events)
}

/// A market's answer as red's command.
const fn market(answer: MarketAnswer) -> GameCommand {
    answered(Answer::Market(answer))
}

/// The market's view red sees.
fn view(state: &GameState, rules: &GameRuleset) -> MarketView {
    match observe(state, rules, Viewer::Player(RED)).decision {
        Some(DecisionView::Market(view)) => view,
        other => panic!("{other:?}"),
    }
}

/// Red's amount of `resource`.
fn held(state: &GameState, resource: ResourceId) -> u32 {
    state.treasury(RED).unwrap()[resource.index()]
}

/// The refusal of a market's answer.
const fn refused(error: MarketError) -> Result<Vec<GameEvent>, GameCommandError> {
    Err(GameCommandError::Market(error))
}

// covers: GAM-DEC-011, ADV-MKT-001, ADV-MKT-008, ADV-MKT-003, ADV-MKT-004
#[test]
fn a_markets_visit_opens_its_decision_for_its_owner_until_it_is_left() {
    let rules = market_rules();
    let post = rules.adventure().object_type_by_key("t:post").unwrap();
    let (mut state, events) = at_market(&rules, "t:post");
    let place = MarketSeen::Object {
        object_type: post,
        anchor: AT,
    };
    let opened = GameEvent::MarketOpened {
        player: RED,
        hero: Some(HERO),
        place,
    };
    assert_eq!(events, [opened]);
    assert_eq!(project(&opened, Viewer::Player(BLUE)), None);
    let open = state.market(RED).copied().unwrap();
    assert_eq!(
        (open.hero, MarketSeen::of(&open.place)),
        (Some(HERO), place)
    );
    assert_eq!(state.market(BLUE), None);
    // The view: the screens, the rates of both trades (the core's order: gold, then wood) and
    // the recipient; nothing to sell or buy.
    let lot = |give, take| Lot { give, take };
    let trade = |give, take, lot| TradeOffer { give, take, lot };
    let seen = MarketView {
        hero: Some(HERO),
        place,
        offers: MarketOffers {
            screens: vec![MarketScreen::Trade, MarketScreen::Transfer],
            count: 5,
            trades: vec![
                trade(GOLD, WOOD, lot(833, 1)),
                trade(WOOD, GOLD, lot(1, 75)),
            ],
            sales: vec![],
            stock: vec![],
            artifacts: vec![],
        },
        recipients: vec![BLUE],
    };
    assert_eq!(view(&state, &rules), seen);
    // Listed: a lot of gold for wood (E08 T135a: no wood to give, no transfer), then its `Leave`,
    // the system's default; the turn waits for it.
    let leave = Answer::Market(MarketAnswer::Leave);
    let lot_of_gold = MarketAnswer::Trade {
        give: GOLD,
        take: WOOD,
        lots: 1,
    };
    assert_eq!(
        legal_commands(&state, &rules, Actor::Player(RED)),
        [market(lot_of_gold), answered(leave)]
    );
    assert_eq!(default_answer(&state, &rules, RED), Some(leave));
    let other_kinds = [
        Answer::Offer(Consent::Decline),
        Answer::University(UniversityAnswer::Leave),
        Answer::Tavern(TavernAnswer::Close),
        Answer::Close,
    ];
    for answer in other_kinds {
        let wrong = play(&mut state, &rules, RED, answered(answer));
        assert_eq!(wrong, Err(GameCommandError::WrongAnswer), "{answer:?}");
    }
    let ended = play(&mut state, &rules, RED, GameCommand::EndTurn);
    assert_eq!(ended, Err(GameCommandError::DecisionOpen));
    // Hashed with the decision (game state hash v35).
    let mut left = state.clone();
    assert_eq!(
        play(&mut left, &rules, RED, answered(leave)),
        Ok(vec![GameEvent::DecisionClosed { player: RED }])
    );
    assert_ne!(state.state_hash(), left.state_hash());
    assert_eq!(left.market(RED), None);
    assert_eq!(default_answer(&left, &rules, RED), None);
    // Leaving loses nothing.
    assert_eq!(left.treasury(RED), state.treasury(RED));
    assert_eq!(
        left.adventure().hero(HERO).unwrap().army,
        state.adventure().hero(HERO).unwrap().army
    );
}

// covers: ADV-MKT-003, GAM-RES-001, GAM-DEC-011
#[test]
fn a_trade_pays_whole_lots_of_the_given_for_the_wanted() {
    let rules = market_rules();
    let (mut state, _) = at_market(&rules, "t:post");
    assert_eq!(held(&state, GOLD), 2000);
    let trade = |give, take, lots| market(MarketAnswer::Trade { give, take, lots });
    // Two lots of 833 gold for a wood each; the window stays open.
    let events = play(&mut state, &rules, RED, trade(GOLD, WOOD, 2)).unwrap();
    let paid = GameEvent::ResourcePaid {
        player: RED,
        resource: GOLD,
        amount: 1666,
    };
    let gained = GameEvent::ResourceGained {
        player: RED,
        resource: WOOD,
        amount: 2,
    };
    assert_eq!(events, [paid, gained]);
    assert_eq!((held(&state, GOLD), held(&state, WOOD)), (334, 2));
    assert!(state.market(RED).is_some());
    // The most is what the treasury holds of the given, by lots; 0 lots are none.
    for lots in [0, 1] {
        let refused_lots = play(&mut state, &rules, RED, trade(GOLD, WOOD, lots));
        assert_eq!(refused_lots, refused(MarketError::Amount(lots)));
    }
    assert_eq!(
        play(&mut state, &rules, RED, trade(WOOD, GOLD, 3)),
        refused(MarketError::Amount(3))
    );
    // A resource for itself, one the market does not value.
    assert_eq!(
        play(&mut state, &rules, RED, trade(WOOD, WOOD, 1)),
        refused(MarketError::SameResource)
    );
    let unknown = ResourceId::from_raw(7);
    assert_eq!(
        play(&mut state, &rules, RED, trade(WOOD, unknown, 1)),
        refused(MarketError::NoPrice)
    );
    // The other way: 75 gold a wood.
    play(&mut state, &rules, RED, trade(WOOD, GOLD, 2)).unwrap();
    assert_eq!((held(&state, GOLD), held(&state, WOOD)), (484, 0));
    // A gain past the treasury's most saturates (GAM-RES-001): every wood of the most for gold.
    state.set_resource(RED, WOOD, u32::MAX);
    let most = held(&state, WOOD);
    let events = play(&mut state, &rules, RED, trade(WOOD, GOLD, most)).unwrap();
    let to_most = GameEvent::ResourceGained {
        player: RED,
        resource: GOLD,
        amount: most - 484,
    };
    assert_eq!(events[1], to_most);
    assert_eq!((held(&state, GOLD), held(&state, WOOD)), (most, 0));
    // A guild trades nothing.
    let (mut guild, _) = at_market(&rules, "t:guild");
    assert_eq!(
        play(&mut guild, &rules, RED, trade(GOLD, WOOD, 1)),
        refused(MarketError::Screen)
    );
}

// covers: ADV-MKT-004, GAM-DEC-011
#[test]
fn a_transfer_gives_another_player_the_amount_one_to_one() {
    let rules = market_rules();
    let (mut state, _) = at_market(&rules, "t:post");
    let blue = state.treasury(BLUE).unwrap()[0];
    let give = |resource, amount, to| {
        market(MarketAnswer::Transfer {
            resource,
            amount,
            to,
        })
    };
    let events = play(&mut state, &rules, RED, give(GOLD, 500, BLUE)).unwrap();
    let paid = GameEvent::ResourcePaid {
        player: RED,
        resource: GOLD,
        amount: 500,
    };
    let received = GameEvent::ResourceGained {
        player: BLUE,
        resource: GOLD,
        amount: 500,
    };
    assert_eq!(events, [paid, received]);
    // The recipient alone sees its gain; nothing names the giver.
    assert_eq!(project(&received, Viewer::Player(RED)), None);
    assert_eq!(project(&paid, Viewer::Player(BLUE)), None);
    assert_eq!(held(&state, GOLD), 1500);
    assert_eq!(state.treasury(BLUE).unwrap()[0], blue + 500);
    assert!(state.market(RED).is_some());
    // The actor and a player not in the game are no recipients; the amount is from 1 to the
    // treasury's.
    for to in [RED, PlayerId::new(5)] {
        let wrong = play(&mut state, &rules, RED, give(GOLD, 1, to));
        assert_eq!(wrong, refused(MarketError::Recipient), "{to:?}");
    }
    for amount in [0, 1501] {
        let wrong = play(&mut state, &rules, RED, give(GOLD, amount, BLUE));
        assert_eq!(wrong, refused(MarketError::Amount(amount)));
    }
    assert_eq!(
        play(&mut state, &rules, RED, give(WOOD, 1, BLUE)),
        refused(MarketError::Amount(1))
    );
    let (mut guild, _) = at_market(&rules, "t:guild");
    assert_eq!(
        play(&mut guild, &rules, RED, give(GOLD, 1, BLUE)),
        refused(MarketError::Screen)
    );
    // The game's refusal reads as the market's reason.
    let text = GameCommandError::Market(MarketError::Recipient).to_string();
    assert_eq!(text, MarketError::Recipient.to_string());
}

// covers: ADV-MKT-005, ADV-MKT-008, GAM-DEC-011
#[test]
fn a_sale_lowers_the_stack_then_gains_its_lots_and_keeps_a_creature() {
    let rules = market_rules();
    let guild = rules.adventure().object_type_by_key("t:guild").unwrap();
    let (mut state, events) = at_market(&rules, "t:guild");
    assert_eq!(
        events,
        [GameEvent::MarketOpened {
            player: RED,
            hero: Some(HERO),
            place: MarketSeen::Object {
                object_type: guild,
                anchor: AT,
            },
        }]
    );
    // Red's nine imps of 50 gold: 35 gold each at five Marketplaces, or a wood for some.
    let seen = view(&state, &rules);
    assert_eq!(seen.recipients, []);
    let lots = &seen.offers.sales;
    assert_eq!(lots.len(), 1);
    assert_eq!(lots[0].slot, 1);
    let in_gold = lots[0].lots.iter().find(|(r, _)| *r == GOLD).unwrap().1;
    assert_eq!(in_gold, Lot { give: 1, take: 35 });
    assert!(lots[0].lots.iter().any(|(r, _)| *r == WOOD));
    let sell = |slot, take, lots| market(MarketAnswer::Sell { slot, take, lots });
    let events = play(&mut state, &rules, RED, sell(1, GOLD, 3)).unwrap();
    let at = ArmySlot {
        army: ArmyRef::Hero(HERO),
        slot: 1,
    };
    let lowered = GameEvent::StackChanged {
        player: RED,
        at,
        stack: Some(ArmyStack {
            slot: 1,
            creature: CreatureTypeId::from_raw(0),
            count: 6,
        }),
    };
    let gained = GameEvent::ResourceGained {
        player: RED,
        resource: GOLD,
        amount: 105,
    };
    assert_eq!(events, [lowered, gained]);
    assert_eq!(held(&state, GOLD), 2105);
    assert!(state.market(RED).is_some());
    // The army's last creature is kept; past the stack is too many; an empty slot sells none.
    assert_eq!(
        play(&mut state, &rules, RED, sell(1, GOLD, 6)),
        refused(MarketError::LastCreature)
    );
    assert_eq!(
        play(&mut state, &rules, RED, sell(1, GOLD, 7)),
        refused(MarketError::Amount(7))
    );
    assert_eq!(
        play(&mut state, &rules, RED, sell(0, GOLD, 1)),
        refused(MarketError::Slot(0))
    );
    play(&mut state, &rules, RED, sell(1, GOLD, 5)).unwrap();
    assert_eq!(state.adventure().hero(HERO).unwrap().army[0].count, 1);
    // A Trading Post buys no creature.
    let (mut post, _) = at_market(&rules, "t:post");
    assert_eq!(
        play(&mut post, &rules, RED, sell(1, GOLD, 1)),
        refused(MarketError::Screen)
    );
}

// covers: ADV-MKT-007, ADV-MKT-006, ADV-MKT-008, GAM-DEC-011
#[test]
fn a_purchase_pays_its_price_gives_the_artifact_and_empties_the_entry() {
    let rules = market_rules();
    let (mut state, _) = at_market(&rules, "t:black");
    // The ring drawn at the build, of 2 000 gold: 5 000 gold or 40 wood at five Marketplaces.
    let ring = rules.core().artifact_by_key("t:ring").unwrap();
    let seen = view(&state, &rules);
    assert_eq!(seen.offers.screens, [MarketScreen::Buy]);
    let entry = StockEntry {
        artifact: Some(ring),
        prices: vec![(GOLD, 5000), (WOOD, 40)],
    };
    assert_eq!(seen.offers.stock, [entry]);
    let buy = |entry, pay| market(MarketAnswer::Buy { entry, pay });
    // The treasury's 2 000 gold cannot pay; the adventure's checks come first.
    assert_eq!(
        play(&mut state, &rules, RED, buy(0, GOLD)),
        Err(GameCommandError::CannotPay)
    );
    assert_eq!(
        play(&mut state, &rules, RED, buy(1, GOLD)),
        refused(MarketError::Entry(1))
    );
    assert_eq!(
        play(&mut state, &rules, RED, buy(0, ResourceId::from_raw(7))),
        refused(MarketError::NoPrice)
    );
    state.set_resource(RED, GOLD, 6000);
    let events = play(&mut state, &rules, RED, buy(0, GOLD)).unwrap();
    let paid = GameEvent::ResourcePaid {
        player: RED,
        resource: GOLD,
        amount: 5000,
    };
    let given = changed(HeroChange::AddArtifact(ring.into()));
    assert_eq!(events, [paid, given]);
    assert_eq!(held(&state, GOLD), 1000);
    assert!(state.market(RED).is_some());
    // Bought: the entry shows empty, and is never sold again.
    let empty = StockEntry {
        artifact: None,
        prices: vec![],
    };
    assert_eq!(view(&state, &rules).offers.stock, [empty]);
    assert_eq!(
        play(&mut state, &rules, RED, buy(0, GOLD)),
        refused(MarketError::Entry(0))
    );
    // A guild sells no artifact.
    let (mut guild, _) = at_market(&rules, "t:guild");
    assert_eq!(
        play(&mut guild, &rules, RED, buy(0, GOLD)),
        refused(MarketError::Screen)
    );
}

// covers: ADV-MKT-006, ADV-MKT-007, ADV-MKT-008
#[test]
fn a_black_markets_stock_is_never_redrawn_by_a_new_month() {
    // Bought, left and visited again a month later (batch E14-3's day 29): the stock drawn at the
    // build, the bought entry still empty.
    let rules = market_rules();
    let (mut state, _) = at_market(&rules, "t:black");
    let ring = rules.core().artifact_by_key("t:ring").unwrap();
    let drawn = StockEntry {
        artifact: Some(ring),
        prices: vec![(GOLD, 5000), (WOOD, 40)],
    };
    assert_eq!(view(&state, &rules).offers.stock, [drawn]);
    state.set_resource(RED, GOLD, 5000);
    let buy = market(MarketAnswer::Buy {
        entry: 0,
        pay: GOLD,
    });
    play(&mut state, &rules, RED, buy.clone()).unwrap();
    let left = play(&mut state, &rules, RED, market(MarketAnswer::Leave)).unwrap();
    assert_eq!(left, [GameEvent::DecisionClosed { player: RED }]);
    // Left: the visitor's observation holds no decision (no stock seen outside the window).
    assert_eq!(observe(&state, &rules, Viewer::Player(RED)).decision, None);
    // Every player ends its turns until day 29, the second month's first day, told to red.
    let mut told = Vec::new();
    while state.day() < 29 {
        told.clear();
        for player in [RED, BLUE] {
            told.extend(play(&mut state, &rules, player, GameCommand::EndTurn).unwrap());
        }
    }
    let started = told
        .iter()
        .find(|e| matches!(e, GameEvent::MonthStarted { month: 2, .. }))
        .copied()
        .unwrap();
    assert_eq!(project(&started, Viewer::Player(RED)), Some(started));
    // Visited again (off its entrance and back): the build's one entry, still empty, refused.
    play(&mut state, &rules, RED, move_hero(0, &[Pos::new(0, 0, 0)])).unwrap();
    visit(&mut state, &rules);
    let empty = StockEntry {
        artifact: None,
        prices: vec![],
    };
    assert_eq!(view(&state, &rules).offers.stock, [empty]);
    assert_eq!(
        play(&mut state, &rules, RED, buy),
        refused(MarketError::Entry(0))
    );
}

// covers: GAM-DEC-001, GAM-DEC-011
#[test]
fn another_players_answer_to_a_markets_open_window_is_refused() {
    // Red's Trading Post open; blue (no decision of its own) answers it with a valid-looking
    // answer of its kind: refused, nothing changed or told ([`play`] checks it).
    let rules = market_rules();
    let (mut state, _) = at_market(&rules, "t:post");
    let answers = [
        MarketAnswer::Trade {
            give: GOLD,
            take: WOOD,
            lots: 1,
        },
        MarketAnswer::Leave,
    ];
    for answer in answers {
        assert_eq!(
            play(&mut state, &rules, BLUE, answered(Answer::Market(answer))),
            Err(GameCommandError::NoDecision),
            "{answer:?}"
        );
    }
    assert!(state.market(RED).is_some());
    assert_eq!(state.market(BLUE), None);
}

// covers: GAM-DEC-011, ADV-MKT-003, ADV-MKT-005, ADV-MKT-007
#[test]
fn the_listed_answers_are_a_lot_of_each_trade_sale_and_purchase_the_treasury_and_army_allow() {
    let rules = market_rules();
    let listed = |state: &GameState| legal_commands(state, &rules, Actor::Player(RED));
    let applies = |state: &GameState| {
        for command in listed(state) {
            let mut probe = state.clone();
            assert!(
                play(&mut probe, &rules, RED, command.clone()).is_ok(),
                "{command:?}"
            );
        }
    };
    let leave = market(MarketAnswer::Leave);
    let trade = |give, take| {
        market(MarketAnswer::Trade {
            give,
            take,
            lots: 1,
        })
    };
    // A Trading Post: each pair whose lot the treasury holds, in the core's order (gold, then
    // wood); none with nothing to give.
    let (mut post, _) = at_market(&rules, "t:post");
    post.set_resource(RED, WOOD, 1);
    assert_eq!(
        listed(&post),
        [trade(GOLD, WOOD), trade(WOOD, GOLD), leave.clone()]
    );
    applies(&post);
    post.set_resource(RED, GOLD, 832);
    post.set_resource(RED, WOOD, 0);
    assert_eq!(listed(&post), core::slice::from_ref(&leave));
    // A Freelancer's Guild: a lot of the imps for gold, the cost resource; none for the army's
    // last creature.
    let (mut guild, _) = at_market(&rules, "t:guild");
    let sell = market(MarketAnswer::Sell {
        slot: 1,
        take: GOLD,
        lots: 1,
    });
    assert_eq!(listed(&guild), [sell.clone(), leave.clone()]);
    applies(&guild);
    let eight = market(MarketAnswer::Sell {
        slot: 1,
        take: GOLD,
        lots: 8,
    });
    play(&mut guild, &rules, RED, eight).unwrap();
    assert_eq!(listed(&guild), core::slice::from_ref(&leave));
    // A Black Market: the ring in gold once the treasury pays its 5 000; none once bought.
    let (mut black, _) = at_market(&rules, "t:black");
    assert_eq!(listed(&black), core::slice::from_ref(&leave));
    black.set_resource(RED, GOLD, 5000);
    let buy = market(MarketAnswer::Buy {
        entry: 0,
        pay: GOLD,
    });
    assert_eq!(listed(&black), [buy.clone(), leave.clone()]);
    applies(&black);
    play(&mut black, &rules, RED, buy).unwrap();
    assert_eq!(listed(&black), [leave]);
}
