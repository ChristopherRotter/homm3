//! The trades and services in the plans' properties (E08 T135b2; increment 14's design note §18):
//! the plans lay out the seven classes near the heroes ([`trade_plan`]), random play walks to them,
//! answers their decisions from the listing or from the view ([`seek_trade`]), and the oracles
//! check the listing ([`trade_answers`], [`check_trade_listing`]), each accepted answer's effects
//! ([`check_trade_answer`]) and what lasts between commands ([`check_trades`]): ADV-MKT-001 to
//! -008, ADV-ALT-001 to -003, ADV-HFT-001, -002, ADV-WMF-001, GAM-DEC-008, GAM-DEC-011 to -013.
//! The cover test [`the_plans_trade_and_use_the_services`] counts what the games meet.

use super::*;
use h3_adventure::MarketScreen;
use h3_game::{ALTAR_SLOTS, AltarAnswer, AltarPlaces, HillFortAnswer, HillFortState, MarketAnswer};

/// A plan's trades and services (E08 T135b2), drawn as [`trades`] draws them; [`trade_plan`]
/// places them.
#[derive(Debug, Clone, Default)]
pub(in crate::tests) struct Trades {
    /// Each one's anchor at an offset from a hero's tile (two columns and rows left of it to two
    /// right) and the hero (modulo the plan's), on its level, so that random play enters them.
    near: Vec<((u8, u8), u8, usize)>,
    /// Artifact codes added to the backpack of the hero the first one stands near (as [`plan`]'s
    /// backpacks: 7 to 40), so that an Altar has artifacts to take and a Black Market's buyer
    /// carries some.
    artifacts: Vec<u16>,
}

/// A plan's trades and services (E08 T135b2): in two plans of three, one to three near the
/// heroes, each drawn among the seven ([`TRADE_OBJECTS`]); in one case of two artifacts for the
/// first one's hero.
fn trades() -> impl Strategy<Value = Trades> {
    let object = proptest::sample::select(TRADE_OBJECTS.to_vec());
    let near = vec(((0u8..5, 0u8..5), any::<u8>(), object), 1..=3);
    let artifacts = prop_oneof![1 => Just(Vec::new()), 1 => vec(7u16..=40, 1..=4)];
    prop_oneof![
        1 => Just(Trades::default()),
        2 => (near, artifacts).prop_map(|(near, artifacts)| Trades { near, artifacts }),
    ]
}

/// [`tavern_plan`]'s plans with trades and services (E08 T135b2, increment 14's design note §18;
/// [`Plan::trades`], [`trades`]), each at an offset from a hero's tile on its level; an RoE plan
/// lays out the Trading Post (99) for AB's two classes ([`AB_TRADES`]). The properties draw these;
/// the cover tests draw [`plan`]'s (the town lane's `TRAPS.md`: a new draw there would shift every
/// deterministic case).
pub(in crate::tests) fn trade_plan() -> impl Strategy<Value = Plan> {
    (tavern_plan(), trades()).prop_map(|(mut plan, trades)| {
        let (side, levels) = (plan.side, 1 + u8::from(plan.two_levels));
        let at = |h: u8, d: u8| (h % side + side + d - 2) % side;
        let roe = plan.version == Version::RoE;
        plan.trades = trades
            .near
            .iter()
            .map(|&((dx, dy), hero, object)| {
                let ((hx, hy, hz), _) = plan.heroes[usize::from(hero) % plan.heroes.len()];
                let object = if roe && AB_TRADES.contains(&object) {
                    TRADE_OBJECTS[0]
                } else {
                    object
                };
                ((at(hx, dx), at(hy, dy), hz % levels), object)
            })
            .collect();
        if let Some(&(_, hero, _)) = trades.near.first()
            && !trades.artifacts.is_empty()
        {
            let index = usize::from(hero) % plan.heroes.len();
            if plan.backpacks.len() <= index {
                plan.backpacks.resize(index + 1, Vec::new());
            }
            plan.backpacks[index].extend(&trades.artifacts);
        }
        plan
    })
}

/// [`steps`]' runs with (E08 T135b2) runs of trades among them in one case of two: `Step::Trade`s
/// with ends of turn among them, which walk heroes to the trades and services, answer their
/// decisions and revisit them. The properties draw these; the cover tests [`steps`]' (as
/// [`trade_plan`]).
pub(in crate::tests) fn trade_steps() -> impl Strategy<Value = Vec<Step>> {
    let trade = vec(
        prop_oneof![8 => any::<u16>().prop_map(Step::Trade), 1 => Just(Step::EndTurn)],
        16..96,
    );
    prop_oneof![
        1 => steps(),
        1 => (steps(), trade, steps()).prop_map(|(mut a, b, c)| {
            a.extend(b);
            a.extend(c);
            a
        }),
    ]
}

/// A trade's or service's kind (E08 T135b2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TradeKind {
    /// A market: a Trading Post, the Freelancer's Guild or a Black Market (ADV-MKT-001).
    Market,
    /// An Altar of Sacrifice (ADV-ALT-001).
    Altar,
    /// A Hill Fort (ADV-HFT-001).
    HillFort,
    /// A War Machine Factory (ADV-WMF-001).
    Factory,
}

/// The trade kind of objects of type `ty`, if one.
pub(super) fn trade_kind(rules: &GameRuleset, ty: ObjectTypeId) -> Option<TradeKind> {
    match rules.adventure().object_type(ty)?.kind {
        ObjectKind::Market(_) => Some(TradeKind::Market),
        ObjectKind::Altar(_) => Some(TradeKind::Altar),
        ObjectKind::HillFort(_) => Some(TradeKind::HillFort),
        ObjectKind::WarMachineFactory(_) => Some(TradeKind::Factory),
        _ => None,
    }
}

/// The trade at object index `index` of `state`, if it is one, with its kind.
fn trade_of(state: &GameState, rules: &GameRuleset, index: usize) -> Option<TradeKind> {
    trade_kind(rules, state.adventure().object(index)?.object_type?)
}

/// Whether `player`'s first open decision is a market's, an Altar's or a Hill Fort's (GAM-DEC-011
/// to -013): its hero, type and anchor.
pub(super) fn trade_open(
    state: &GameState,
    player: PlayerId,
) -> Option<(HeroId, ObjectTypeId, TilePos)> {
    let market = state
        .market(player)
        .map(|v| (v.hero, v.object_type, v.anchor));
    let altar = || {
        state
            .altar(player)
            .map(|v| (v.hero, v.object_type, v.anchor))
    };
    let fort = || {
        state
            .hill_fort(player)
            .map(|v| (v.hero, v.object_type, v.anchor))
    };
    market.or_else(altar).or_else(fort)
}

/// The entrances of the trades and services standing on `state`'s map, in the walk's order.
fn trade_tiles(state: &GameState, rules: &GameRuleset) -> Vec<TilePos> {
    let map = state.adventure().map();
    let mut tiles = Vec::new();
    for z in 0..map.levels() {
        for y in 0..map.height() {
            for x in 0..map.width() {
                let pos = TilePos::new(x, y, z);
                if visited_at(state, pos).is_some_and(|i| trade_of(state, rules, i).is_some()) {
                    tiles.push(pos);
                }
            }
        }
    }
    tiles
}

/// The core's resources, in its order.
fn resources(rules: &GameRuleset) -> impl Iterator<Item = ResourceId> + '_ {
    (0..rules.core().resources().len()).filter_map(ResourceId::from_index)
}

/// The market parameters' cost resource (the original's gold): the one the listed sales and
/// purchases name (increment 14's design note §18).
fn cost_resource(rules: &GameRuleset) -> ResourceId {
    rules.adventure().params().market().unwrap().cost()
}

/// A drawn answer to `player`'s open market, Altar or Hill Fort (E08 T135b2; [`seek_trade`]): in
/// one case of six `Leave`; else in three of five the `n`-th listed answer; else one drawn from
/// the view, which `apply` may refuse: a market's trade of 1 to 3 lots of a drawn pair (a pair
/// of one resource too), a transfer of 1 to 10 of a drawn resource to a drawn player (the actor
/// too), a sale of 1 to 3 lots of a drawn slot for a drawn resource, a purchase of a drawn entry
/// (one past the stock too) in a drawn resource; an Altar's sacrifice of the first of the listed
/// all's artifacts (none too), or of half of each stack (the lowest's all but one when it is
/// alone); a Hill Fort's upgrade of a drawn slot (one past the army's too) or its `All`. None
/// without such a decision.
pub(super) fn drawn_answer(
    state: &GameState,
    rules: &GameRuleset,
    legal: &[GameCommand],
    n: u16,
) -> Option<GameCommand> {
    let player = acting(state);
    trade_open(state, player)?;
    let leave = match () {
        () if state.market(player).is_some() => Answer::Market(MarketAnswer::Leave),
        () if state.altar(player).is_some() => Answer::Altar(AltarAnswer::Leave),
        () => Answer::HillFort(HillFortAnswer::Leave),
    };
    if n % 6 == 5 {
        return Some(GameCommand::Answer(leave));
    }
    let n = n / 6;
    if n % 5 < 3 || legal.is_empty() {
        return legal
            .get(usize::from(n / 5) % legal.len().max(1))
            .cloned()
            .or(Some(GameCommand::Answer(leave)));
    }
    let n = n / 5;
    let count = rules.core().resources().len();
    let resource = |k: u16| ResourceId::from_index(usize::from(k) % count).unwrap();
    let small = |k: u16| u32::from(k % 3) + 1;
    let answer = if state.market(player).is_some() {
        let players = u8::try_from(state.players()).unwrap();
        Answer::Market(match n % 4 {
            0 => MarketAnswer::Trade {
                give: resource(n / 4),
                take: resource(n / 28),
                lots: small(n / 196),
            },
            1 => MarketAnswer::Transfer {
                resource: resource(n / 4),
                amount: u32::from(n / 28 % 10) + 1,
                to: PlayerId::new(u8::try_from(n / 280).unwrap() % players),
            },
            2 => MarketAnswer::Sell {
                slot: u8::try_from(n / 4 % 7).unwrap(),
                take: resource(n / 28),
                lots: small(n / 196),
            },
            _ => MarketAnswer::Buy {
                entry: u8::try_from(n / 4 % 9).unwrap(),
                pay: resource(n / 36),
            },
        })
    } else if let Some(visit) = state.altar(player) {
        let hero = state.adventure().hero(visit.hero)?;
        Answer::Altar(if n.is_multiple_of(2) {
            let listed = legal.iter().find_map(|c| match c {
                GameCommand::Answer(Answer::Altar(AltarAnswer::Artifacts(places))) => Some(*places),
                _ => None,
            });
            let all = listed.map_or(Vec::new(), |p| p.places().collect());
            let take = usize::from(n / 2) % (all.len() + 1);
            let places = all[..take]
                .iter()
                .fold(AltarPlaces::NONE, |p, &place| p.with(place).unwrap_or(p));
            AltarAnswer::Artifacts(places)
        } else {
            let mut amounts = [0; ALTAR_SLOTS];
            for stack in &hero.army {
                amounts[usize::from(stack.slot)] = stack.count / 2;
            }
            if let [only] = hero.army[..] {
                amounts[usize::from(only.slot)] = only.count - 1;
            }
            AltarAnswer::Creatures(amounts)
        })
    } else if n % 9 == 8 {
        Answer::HillFort(HillFortAnswer::All)
    } else {
        Answer::HillFort(HillFortAnswer::Upgrade(u8::try_from(n % 9).unwrap()))
    };
    Some(GameCommand::Answer(answer))
}

/// [`Step::Trade`] (E08 T135b2; increment 14's design note §18): in a battle as [`fight`]; an open
/// market's, Altar's or Hill Fort's decision answered as [`drawn_answer`] draws it; a War Machine
/// Factory's question accepted, declined when `n % 4 == 3`; another decision settled
/// ([`settle`]); on the map, in one step of four the `n`-th listed `Revisit` of a hero standing on
/// a trade's entrance, else the `n`-th of its options that end on a trade's entrance, else one that
/// ends nearer to one than its hero stands, else `EndTurn`; as [`seek_bank`] on a map without
/// trades.
pub(super) fn seek_trade(
    state: &GameState,
    rules: &GameRuleset,
    legal: &[GameCommand],
    n: u16,
) -> GameCommand {
    let tiles = trade_tiles(state, rules);
    if tiles.is_empty() {
        return seek_bank(state, rules, legal, n);
    }
    let actor = acting(state);
    if battle_of(state, super::actor(state)).is_some() {
        return fight(state, rules, legal, n);
    }
    if let Some(answer) = drawn_answer(state, rules, legal, n) {
        return answer;
    }
    let asked = state
        .decision(actor)
        .is_some_and(|offer| trade_kind(rules, offer.object_type) == Some(TradeKind::Factory));
    if asked {
        return ANSWERS[usize::from(n % 4 == 3)].clone();
    }
    if let Some(answer) = settle(state, legal, n) {
        return answer;
    }
    let adventure = state.adventure();
    let on_trade = |hero: HeroId| adventure.hero(hero).is_some_and(|h| tiles.contains(&h.pos));
    let revisits: Vec<&GameCommand> = legal
        .iter()
        .filter(|c| {
            matches!(c, GameCommand::Adventure(AdventureCommand::Revisit { hero }) if on_trade(*hero))
        })
        .collect();
    if n % 4 == 3 && !revisits.is_empty() {
        return revisits[usize::from(n / 4) % revisits.len()].clone();
    }
    let moves: Vec<(&GameCommand, HeroId, &[TilePos])> = legal
        .iter()
        .filter_map(|c| as_move(c).map(|(hero, path)| (c, hero, path)))
        .collect();
    let entering: Vec<&GameCommand> = (moves.iter())
        .filter(|(_, _, path)| path.last().is_some_and(|to| tiles.contains(to)))
        .map(|&(c, _, _)| c)
        .collect();
    if !entering.is_empty() {
        return entering[usize::from(n / 4) % entering.len()].clone();
    }
    let distance = |pos: TilePos| {
        (tiles.iter())
            .filter(|t| t.level == pos.level && **t != pos)
            .map(|t| t.x.abs_diff(pos.x).max(t.y.abs_diff(pos.y)))
            .min()
    };
    let mut best: Vec<&GameCommand> = Vec::new();
    let mut nearest = u16::MAX;
    for &(command, hero, path) in &moves {
        let Some(at) = adventure.hero(hero).map(|h| h.pos) else {
            continue;
        };
        let (here, there) = (distance(at), path.last().and_then(|&to| distance(to)));
        if let (Some(here), Some(there)) = (here, there)
            && there < here
            && there <= nearest
        {
            if there < nearest {
                best.clear();
                nearest = there;
            }
            best.push(command);
        }
    }
    best.get(usize::from(n) % best.len().max(1))
        .map_or(GameCommand::EndTurn, |&command| command.clone())
}

/// Whether `apply` accepts `answer` of `player` on `state` (a probe; `state` unchanged).
fn accepts(state: &GameState, rules: &GameRuleset, player: PlayerId, answer: Answer) -> bool {
    let mut probe = state.clone();
    let command = GameCommand::Answer(answer);
    apply(
        &mut probe,
        rules,
        Actor::Player(player),
        command,
        &mut NoopSink,
    )
    .is_ok()
}

/// The answers to `player`'s open market, Altar or Hill Fort as `legal_commands` must list them
/// (GAM-DEC-011 to -013; E08 T135a, increment 14's design note §18), by an oracle of the listing's
/// rule over candidates built from the decision, the hero and the core alone, in this order, each
/// kept when `apply` accepts it, then the kind's `Leave`:
/// - a market's trade of one lot of every pair, the given then the wanted in the core's order; a
///   sale of one lot of every slot of the army, in slot order, for the cost resource; a purchase
///   of every entry of the stock (the market's resolved draw) in the cost resource; no transfer;
/// - an Altar's artifacts' all, every place of the hero's artifacts, the worn ones by position,
///   then the backpack's in its order, but those the Altar keeps (the Spell Book, the Catapult),
///   until the Altar's places; then its creatures' all, every stack whole but one creature of the
///   lowest occupied slot;
/// - a Hill Fort's upgrade of every slot of the army, in slot order; then its `All`.
///
/// None without such a decision.
pub(super) fn trade_answers(
    state: &GameState,
    rules: &GameRuleset,
    player: PlayerId,
) -> Option<Vec<GameCommand>> {
    let accepted = |answer: &Answer| accepts(state, rules, player, *answer);
    let cost = cost_resource(rules);
    let (answers, leave): (Vec<Answer>, Answer) = if let Some(visit) = state.market(player) {
        let hero = state.adventure().hero(visit.hero)?;
        let trades = resources(rules).flat_map(|give| {
            resources(rules).map(move |take| MarketAnswer::Trade {
                give,
                take,
                lots: 1,
            })
        });
        let sales = hero.army.iter().map(|stack| MarketAnswer::Sell {
            slot: stack.slot,
            take: cost,
            lots: 1,
        });
        let object = state.adventure().object(visit.index)?;
        let stock = object.resolved.stock().len();
        let purchases = (0..=u8::MAX)
            .take(stock)
            .map(|entry| MarketAnswer::Buy { entry, pay: cost });
        let answers = trades.chain(sales).chain(purchases).map(Answer::Market);
        (
            answers.filter(accepted).collect(),
            Answer::Market(MarketAnswer::Leave),
        )
    } else if let Some(visit) = state.altar(player) {
        let ObjectKind::Altar(altar) = &rules.adventure().object_type(visit.object_type)?.kind
        else {
            return None;
        };
        let details = state.hero(visit.hero)?;
        let worn = (details.worn.iter().enumerate()).filter_map(|(p, a)| {
            a.as_ref()
                .map(|a| (ArtifactPlace::Worn(Position(p)), a.artifact))
        });
        let carried = (details.backpack.iter().enumerate())
            .map(|(i, a)| (ArtifactPlace::Backpack(i), a.artifact));
        let places = worn
            .chain(carried)
            .filter(|(_, artifact)| !altar.keeps.contains(artifact))
            .take(usize::from(altar.places))
            .fold(AltarPlaces::NONE, |p, (place, _)| {
                p.with(place).unwrap_or(p)
            });
        let army = &state.adventure().hero(visit.hero)?.army;
        let mut amounts = [0; ALTAR_SLOTS];
        for stack in army {
            amounts[usize::from(stack.slot)] = stack.count;
        }
        if let Some(lowest) = amounts.iter_mut().find(|a| **a > 0) {
            *lowest -= 1;
        }
        let answers = [
            AltarAnswer::Artifacts(places),
            AltarAnswer::Creatures(amounts),
        ];
        let answers = answers.into_iter().map(Answer::Altar);
        (
            answers.filter(accepted).collect(),
            Answer::Altar(AltarAnswer::Leave),
        )
    } else {
        let visit = state.hill_fort(player)?;
        let army = &state.adventure().hero(visit.hero)?.army;
        let upgrades = army.iter().map(|stack| HillFortAnswer::Upgrade(stack.slot));
        let answers = upgrades.chain([HillFortAnswer::All]).map(Answer::HillFort);
        (
            answers.filter(accepted).collect(),
            Answer::HillFort(HillFortAnswer::Leave),
        )
    };
    Some(
        answers
            .into_iter()
            .chain([leave])
            .map(GameCommand::Answer)
            .collect(),
    )
}

/// The open trade's view, as its owner `player` sees it (GAM-DEC-011 to -013).
fn trade_view(state: &GameState, rules: &GameRuleset, player: PlayerId) -> Option<DecisionView> {
    observe(state, rules, Viewer::Player(player)).decision
}

/// What an answer pays, in order, and gains, in order, each gain with its gainer
/// ([`trade_moves`]).
type Moves = (Vec<(ResourceId, u32)>, Vec<(PlayerId, ResourceId, u32)>);

/// A treasury's gain of `amount` of a resource it holds `held` of: what it actually gains, at most
/// to 2³¹ − 1 (GAM-RES-001).
fn gained(held: u32, amount: u64) -> u32 {
    let room = u64::from(i32::MAX.unsigned_abs()).saturating_sub(u64::from(held));
    u32::try_from(amount.min(room)).unwrap()
}

/// How much of `resource` `player`'s treasury holds on `state`.
fn held(state: &GameState, player: PlayerId, resource: ResourceId) -> u32 {
    state.treasury(player).unwrap()[usize::from(resource.raw())]
}

/// What a market's or a Hill Fort's answer `answer` of `player` pays and what it gains, by player,
/// as its view on `before` gives them (ADV-MKT-003 to -005, -007, ADV-HFT-002): the paid
/// resources in order, the gains in order (each the amount the treasury actually gains, with its
/// gainer). Empty for every other answer, and for one the view does not price.
pub(super) fn trade_moves(
    rules: &GameRuleset,
    before: &GameState,
    player: PlayerId,
    answer: &Answer,
) -> Moves {
    let view = trade_view(before, rules, player);
    match (view, *answer) {
        (Some(DecisionView::Market(view)), Answer::Market(answer)) => match answer {
            MarketAnswer::Trade { give, take, lots } => {
                let offer = view
                    .offers
                    .trades
                    .iter()
                    .find(|o| (o.give, o.take) == (give, take));
                let Some(lot) = offer.map(|o| o.lot) else {
                    return Default::default();
                };
                let amount = u64::from(lots) * u64::from(lot.take);
                let gain = gained(held(before, player, take), amount);
                (vec![(give, lots * lot.give)], vec![(player, take, gain)])
            }
            MarketAnswer::Transfer {
                resource,
                amount,
                to,
            } => {
                let gain = gained(held(before, to, resource), u64::from(amount));
                (vec![(resource, amount)], vec![(to, resource, gain)])
            }
            MarketAnswer::Sell { slot, take, lots } => {
                let sale = view.offers.sales.iter().find(|s| s.slot == slot);
                let lot = sale.and_then(|s| s.lots.iter().find(|&&(r, _)| r == take));
                let Some(&(_, lot)) = lot else {
                    return Default::default();
                };
                let amount = u64::from(lots) * u64::from(lot.take);
                let gain = gained(held(before, player, take), amount);
                (Vec::new(), vec![(player, take, gain)])
            }
            MarketAnswer::Buy { entry, pay } => {
                let entry = view.offers.stock.get(usize::from(entry));
                let price = entry.and_then(|e| e.prices.iter().find(|&&(r, _)| r == pay));
                (price.into_iter().copied().collect(), Vec::new())
            }
            MarketAnswer::Leave => Default::default(),
        },
        (Some(DecisionView::HillFort(view)), Answer::HillFort(answer)) => {
            let rows = view.rows.iter().filter(|r| r.row.upgrade.is_some());
            let paid = match answer {
                HillFortAnswer::Upgrade(slot) => rows
                    .filter(|r| r.row.slot == slot)
                    .flat_map(|r| r.row.price.iter().copied())
                    .collect(),
                HillFortAnswer::All => rows.flat_map(|r| r.row.price.iter().copied()).collect(),
                HillFortAnswer::Leave => Vec::new(),
            };
            (paid, Vec::new())
        }
        _ => Default::default(),
    }
}

/// An accepted answer `answer` of `actor` to its open market, Altar or Hill Fort, from `before` to
/// `after` with `events` (E08 T135b2; increment 14's design note §18, `check_trades`), against the
/// decision's view on `before` alone:
/// - a market's trade (ADV-MKT-003) of a pair the view values pays its lots of the given and
///   gains its lots of the wanted, saturating; a transfer (ADV-MKT-004) to a recipient the view
///   lists pays the amount and gains it to the recipient alone, the sum kept; a sale (ADV-MKT-005)
///   lowers the slot by its lots of the view's creatures and gains its lots of the resource, the
///   army keeping a creature; a purchase (ADV-MKT-007) of an entry the view shows pays its price
///   in the resource and gives the hero the artifact, the entry shown sold after; each told as
///   exactly those events, the market still open, the adventure unchanged but for the sale's
///   stack and the purchase's sold entry;
/// - an Altar's sacrifice (ADV-ALT-002, -003) of artifacts the view prices removes each from its
///   place in the change's order and gives the sum of their experience, of creatures lowers each
///   slot by its amount and gives each stack's experience (its creatures' with Learning), the
///   army keeping one; then its levels (`LevelGained`), the Altar open unless a level with two
///   offers opened, before which it closes (GAM-DEC-007, decision 371);
/// - a Hill Fort's upgrade (ADV-HFT-002) of a row the view shows affordable pays its price and
///   makes the stack its upgrade, its count kept; `All` each such row in slot order; the fort
///   still open;
/// - `Leave` closes the decision, nothing else.
pub(super) fn check_trade_answer(
    rules: &GameRuleset,
    (before, after): (&GameState, &GameState),
    actor: PlayerId,
    answer: &Answer,
    events: &[GameEvent],
) -> Result<(), TestCaseError> {
    let view = trade_view(before, rules, actor);
    let paid_events = |paid: &[(ResourceId, u32)]| -> Vec<GameEvent> {
        paid.iter()
            .map(|&(resource, amount)| GameEvent::ResourcePaid {
                player: actor,
                resource,
                amount,
            })
            .collect()
    };
    let gain_events = |gains: &[(PlayerId, ResourceId, u32)]| -> Vec<GameEvent> {
        gains
            .iter()
            .map(|&(player, resource, amount)| GameEvent::ResourceGained {
                player,
                resource,
                amount,
            })
            .collect()
    };
    let stack_event =
        |hero: HeroId, slot: u8, stack: Option<h3_contracts::ArmyStack>| GameEvent::StackChanged {
            player: actor,
            at: ArmySlot {
                army: ArmyRef::Hero(hero),
                slot,
            },
            stack,
        };
    let leaves = matches!(
        answer,
        Answer::Market(MarketAnswer::Leave)
            | Answer::Altar(AltarAnswer::Leave)
            | Answer::HillFort(HillFortAnswer::Leave)
    );
    if leaves {
        prop_assert_eq!(events, &[GameEvent::DecisionClosed { player: actor }][..]);
        prop_assert!(trade_open(after, actor).is_none());
        prop_assert_eq!(before.treasury(actor), after.treasury(actor));
        prop_assert_eq!(before.heroes(), after.heroes());
        prop_assert_eq!(before.adventure(), after.adventure());
        return Ok(());
    }
    let open = trade_open(before, actor).ok_or_else(|| fail(format!("{answer:?} unopened")))?;
    let hero = open.0;
    let (paid, gains) = trade_moves(rules, before, actor, answer);
    let army_of = |state: &GameState| state.adventure().hero(hero).map(|h| h.army.clone());
    match (view, *answer) {
        (Some(DecisionView::Market(view)), Answer::Market(market)) => {
            let mut expected = paid_events(&paid);
            match market {
                MarketAnswer::Trade { give, take, .. } => {
                    prop_assert!(give != take);
                    prop_assert!(!paid.is_empty(), "{:?} of an unvalued pair", answer);
                    prop_assert!(view.offers.screens.contains(&MarketScreen::Trade));
                    prop_assert_eq!(before.adventure(), after.adventure());
                }
                MarketAnswer::Transfer { to, .. } => {
                    prop_assert!(view.recipients.contains(&to), "{:?}", answer);
                    prop_assert!(view.offers.screens.contains(&MarketScreen::Transfer));
                    prop_assert_eq!(before.adventure(), after.adventure());
                    let sum = |state: &GameState| -> u64 {
                        (0..state.players())
                            .map(|p| PlayerId::new(u8::try_from(p).unwrap()))
                            .map(|p| u64::from(state.treasury(p).unwrap().iter().sum::<u32>()))
                            .sum()
                    };
                    // The transfer's sum kept, but what saturates at the recipient.
                    let lost = u64::from(paid[0].1) - u64::from(gains[0].2);
                    prop_assert_eq!(sum(after) + lost, sum(before));
                }
                MarketAnswer::Sell { slot, take, lots } => {
                    let sale = view.offers.sales.iter().find(|s| s.slot == slot);
                    let lot = sale.and_then(|s| s.lots.iter().find(|&&(r, _)| r == take));
                    let &(_, lot) = lot.ok_or_else(|| fail(format!("{answer:?} unpriced")))?;
                    let mut army = army_of(before).unwrap();
                    let at = army.iter().position(|s| s.slot == slot).unwrap();
                    let sold = lots * lot.give;
                    prop_assert!(army[at].count >= sold, "{:?}", answer);
                    army[at].count -= sold;
                    let lowered = Some(army[at]).filter(|s| s.count > 0);
                    army.retain(|s| s.count > 0);
                    prop_assert!(!army.is_empty(), "{:?} sold the army's last", answer);
                    expected.push(stack_event(hero, slot, lowered));
                    prop_assert_eq!(army_of(after), Some(army));
                }
                MarketAnswer::Buy { entry, .. } => {
                    let shown = view.offers.stock.get(usize::from(entry));
                    let artifact = shown.and_then(|e| e.artifact);
                    let artifact = artifact.ok_or_else(|| fail(format!("{answer:?} unshown")))?;
                    prop_assert_eq!(paid.len(), 1, "{:?} unpriced", answer);
                    expected.push(GameEvent::HeroChanged {
                        player: actor,
                        hero,
                        change: HeroChange::AddArtifact(artifact.into()),
                    });
                    let now = match trade_view(after, rules, actor) {
                        Some(DecisionView::Market(now)) => now,
                        other => return Err(fail(format!("{answer:?} left {other:?}"))),
                    };
                    let entry = now.offers.stock.get(usize::from(entry));
                    prop_assert_eq!(entry.and_then(|e| e.artifact), None, "{:?}", answer);
                    prop_assert_eq!(army_of(before), army_of(after));
                }
                MarketAnswer::Leave => unreachable!(),
            }
            expected.extend(gain_events(&gains));
            prop_assert_eq!(events, &expected[..], "{:?}", answer);
            prop_assert_eq!(trade_open(after, actor), Some(open));
        }
        (Some(DecisionView::Altar(view)), Answer::Altar(altar)) => {
            let details = before.hero(hero).unwrap();
            let mut expected = Vec::new();
            let experience = match altar {
                AltarAnswer::Artifacts(places) => {
                    prop_assert!(view.offers.screens.artifacts, "{:?}", answer);
                    prop_assert!(places.len() <= u32::from(view.offers.places));
                    let mut sum = 0_u32;
                    for place in places.places() {
                        let shown = view.offers.artifacts.iter().find(|a| a.place == place);
                        let shown = shown.and_then(|a| a.experience);
                        let shown = shown.ok_or_else(|| fail(format!("{answer:?} kept")))?;
                        sum = sum.saturating_add(shown);
                        expected.push(GameEvent::HeroChanged {
                            player: actor,
                            hero,
                            change: HeroChange::RemoveArtifact(place),
                        });
                    }
                    prop_assert_eq!(army_of(before), army_of(after));
                    sum
                }
                AltarAnswer::Creatures(amounts) => {
                    prop_assert!(view.offers.screens.creatures, "{:?}", answer);
                    let mut army = army_of(before).unwrap();
                    let mut sum = 0_u32;
                    for (slot, amount) in (0_u8..).zip(amounts).filter(|&(_, a)| a > 0) {
                        let shown = view.offers.stacks.iter().find(|s| s.slot == slot);
                        let shown = shown.ok_or_else(|| fail(format!("{answer:?} of no stack")))?;
                        let at = army.iter().position(|s| s.slot == slot).unwrap();
                        prop_assert!(army[at].count >= amount, "{:?}", answer);
                        army[at].count -= amount;
                        let base = amount.saturating_mul(shown.each);
                        let each = h3_heroes::gained_experience(details, rules.core(), base);
                        sum = sum.saturating_add(each);
                        let lowered = Some(army[at]).filter(|s| s.count > 0);
                        expected.push(stack_event(hero, slot, lowered));
                    }
                    army.retain(|s| s.count > 0);
                    prop_assert!(!army.is_empty(), "{:?} took the army's last", answer);
                    prop_assert_eq!(army_of(after), Some(army));
                    sum
                }
                AltarAnswer::Leave => unreachable!(),
            };
            prop_assert!(experience > 0, "{:?} for nothing", answer);
            expected.push(GameEvent::HeroChanged {
                player: actor,
                hero,
                change: HeroChange::GainExperience(experience),
            });
            prop_assert_eq!(events.get(..expected.len()), Some(&expected[..]));
            prop_assert_eq!(before.treasury(actor), after.treasury(actor));
            // Then its plain levels, and before a level of two offers the Altar's close.
            let rest = &events[expected.len()..];
            let levels = rest
                .iter()
                .take_while(|e| matches!(e, GameEvent::LevelGained { hero: h, .. } if *h == hero))
                .count();
            match &rest[levels..] {
                [] => prop_assert_eq!(trade_open(after, actor), Some(open)),
                [
                    GameEvent::DecisionClosed { player },
                    GameEvent::LevelUpOpened {
                        player: p, hero: h, ..
                    },
                ] => {
                    prop_assert_eq!((*player, *p, *h), (actor, actor, hero));
                    prop_assert!(trade_open(after, actor).is_none());
                    prop_assert!(after.level_up(actor).is_some());
                }
                other => return Err(fail(format!("{answer:?} then {other:?}"))),
            }
        }
        (Some(DecisionView::HillFort(view)), Answer::HillFort(fort)) => {
            let rows: Vec<&h3_game::HillFortRowView> = match fort {
                HillFortAnswer::Upgrade(slot) => {
                    view.rows.iter().filter(|r| r.row.slot == slot).collect()
                }
                HillFortAnswer::All => view
                    .rows
                    .iter()
                    .filter(|r| r.row.upgrade.is_some())
                    .collect(),
                HillFortAnswer::Leave => unreachable!(),
            };
            prop_assert!(!rows.is_empty(), "{:?} of no row", answer);
            if let HillFortAnswer::Upgrade(_) = fort {
                prop_assert_eq!(rows[0].state, HillFortState::Affordable, "{:?}", answer);
            } else {
                prop_assert_eq!(view.all_state, HillFortState::Affordable, "{:?}", answer);
            }
            let mut army = army_of(before).unwrap();
            let mut expected = Vec::new();
            for row in rows {
                let upgrade = row.row.upgrade.unwrap();
                expected.extend(paid_events(&row.row.price));
                let at = army.iter().position(|s| s.slot == row.row.slot).unwrap();
                prop_assert_eq!(
                    (army[at].creature, army[at].count),
                    (row.row.creature, row.row.count)
                );
                army[at].creature = upgrade;
                expected.push(stack_event(hero, row.row.slot, Some(army[at])));
            }
            prop_assert_eq!(events, &expected[..], "{:?}", answer);
            prop_assert_eq!(army_of(after), Some(army));
            prop_assert_eq!(trade_open(after, actor), Some(open));
        }
        (view, _) => return Err(fail(format!("{answer:?} answered {view:?}"))),
    }
    // The treasuries as paid and gained.
    let mut treasuries: Vec<Vec<u32>> = (0..before.players())
        .map(|p| {
            before
                .treasury(PlayerId::new(u8::try_from(p).unwrap()))
                .unwrap()
                .to_vec()
        })
        .collect();
    for &(resource, amount) in &paid {
        treasuries[usize::from(actor.raw())][usize::from(resource.raw())] -= amount;
    }
    for &(player, resource, amount) in &gains {
        treasuries[usize::from(player.raw())][usize::from(resource.raw())] += amount;
    }
    for (p, treasury) in treasuries.iter().enumerate() {
        let player = PlayerId::new(u8::try_from(p).unwrap());
        prop_assert_eq!(
            after.treasury(player).unwrap(),
            &treasury[..],
            "{:?}",
            answer
        );
    }
    Ok(())
}

/// The listed answers of `actor`'s open market, Altar or Hill Fort (E08 T135b2; [`check_legal`]):
/// each applies with the effects [`check_trade_answer`] checks; refused with the state unchanged
/// and nothing told: a trade of a resource for itself and of one lot past the most, a transfer
/// to the actor and one past the treasury's, a sale of a slot past the army's, a purchase of an
/// entry past the stock, an Altar's empty sacrifices and its creatures' all with the lowest
/// creature too, a Hill Fort's upgrade of each row not affordable (a stack without an upgrade,
/// one too dear) and its `All` unless the whole is affordable.
pub(super) fn check_trade_listing(
    state: &GameState,
    rules: &GameRuleset,
    actor: PlayerId,
    legal: &[GameCommand],
) -> Result<(), TestCaseError> {
    let probe = |command: &GameCommand| {
        let mut probe = state.clone();
        let mut events = Vec::new();
        let result = apply(
            &mut probe,
            rules,
            Actor::Player(actor),
            command.clone(),
            &mut events,
        );
        (probe, events, result)
    };
    for command in legal {
        let (after, events, result) = probe(command);
        prop_assert!(result.is_ok(), "legal {:?} rejected: {:?}", command, result);
        let GameCommand::Answer(answer) = command else {
            return Err(fail(format!("{command:?} listed")));
        };
        check_trade_answer(rules, (state, &after), actor, answer, &events)?;
    }
    let mut refused: Vec<Answer> = Vec::new();
    let view = trade_view(state, rules, actor);
    let hero = trade_open(state, actor).unwrap().0;
    let army = state.adventure().hero(hero).unwrap().army.clone();
    match &view {
        Some(DecisionView::Market(view)) => {
            let gold = cost_resource(rules);
            refused.push(Answer::Market(MarketAnswer::Trade {
                give: gold,
                take: gold,
                lots: 1,
            }));
            for offer in &view.offers.trades {
                let most = held(state, actor, offer.give) / offer.lot.give.max(1);
                refused.push(Answer::Market(MarketAnswer::Trade {
                    give: offer.give,
                    take: offer.take,
                    lots: most + 1,
                }));
            }
            refused.push(Answer::Market(MarketAnswer::Transfer {
                resource: gold,
                amount: 1,
                to: actor,
            }));
            let other = PlayerId::new((actor.raw() + 1) % u8::try_from(state.players()).unwrap());
            refused.push(Answer::Market(MarketAnswer::Transfer {
                resource: gold,
                amount: held(state, actor, gold) + 1,
                to: other,
            }));
            refused.push(Answer::Market(MarketAnswer::Sell {
                slot: 7,
                take: gold,
                lots: 1,
            }));
            let stock = u8::try_from(view.offers.stock.len()).unwrap();
            refused.push(Answer::Market(MarketAnswer::Buy {
                entry: stock,
                pay: gold,
            }));
        }
        Some(DecisionView::Altar(_)) => {
            refused.push(Answer::Altar(AltarAnswer::Artifacts(AltarPlaces::NONE)));
            refused.push(Answer::Altar(AltarAnswer::Creatures([0; ALTAR_SLOTS])));
            let mut all = [0; ALTAR_SLOTS];
            for stack in &army {
                all[usize::from(stack.slot)] = stack.count;
            }
            refused.push(Answer::Altar(AltarAnswer::Creatures(all)));
        }
        Some(DecisionView::HillFort(view)) => {
            for row in &view.rows {
                if row.state != HillFortState::Affordable {
                    refused.push(Answer::HillFort(HillFortAnswer::Upgrade(row.row.slot)));
                }
            }
            if view.all_state != HillFortState::Affordable {
                refused.push(Answer::HillFort(HillFortAnswer::All));
            }
            let free = (0..7).find(|&slot| army.iter().all(|s| s.slot != slot));
            refused.extend(free.map(|slot| Answer::HillFort(HillFortAnswer::Upgrade(slot))));
        }
        other => return Err(fail(format!("a trade's listing with {other:?}"))),
    }
    let hash = state.state_hash();
    for answer in refused {
        let (after, events, result) = probe(&GameCommand::Answer(answer));
        prop_assert!(result.is_err(), "{:?} accepted", answer);
        prop_assert_eq!(&after, state);
        prop_assert_eq!(after.state_hash(), hash);
        prop_assert!(events.is_empty());
    }
    Ok(())
}

/// What lasts between commands at the trades (E08 T135b2; increment 14's design note §18), after
/// an accepted command of `actor` from `before` to `after`: each Black Market's stock is its
/// build's draw, unchanged (ADV-MKT-006), and its sold entries only grow, an entry sold only by
/// this command's purchase of it at that market (ADV-MKT-007); an accepted answer to the actor's
/// open market, Altar or Hill Fort has [`check_trade_answer`]'s effects, listed or not.
pub(super) fn check_trades(
    rules: &GameRuleset,
    (before, after): (&GameState, &GameState),
    (actor, command): (PlayerId, &GameCommand),
    events: &[GameEvent],
) -> Result<(), TestCaseError> {
    let (b, a) = (before.adventure(), after.adventure());
    let bought = match (command, before.market(actor)) {
        (GameCommand::Answer(Answer::Market(MarketAnswer::Buy { entry, .. })), Some(visit)) => {
            Some((visit.index, *entry))
        }
        _ => None,
    };
    for index in 0..b.map().objects().len() {
        if trade_of(before, rules, index) != Some(TradeKind::Market) {
            continue;
        }
        let (was, is) = (b.object(index).unwrap(), a.object(index).unwrap());
        prop_assert_eq!(was.resolved.stock(), is.resolved.stock());
        for entry in (0..=u8::MAX).take(was.resolved.stock().len()) {
            let (sold, sells) = (b.is_sold(index, entry), a.is_sold(index, entry));
            prop_assert!(!sold || sells, "{:?} unsold at {}", entry, index);
            prop_assert!(sold || !sells || bought == Some((index, entry)));
        }
    }
    if let GameCommand::Answer(
        answer @ (Answer::Market(_) | Answer::Altar(_) | Answer::HillFort(_)),
    ) = command
        && trade_open(before, actor).is_some()
    {
        check_trade_answer(rules, (before, after), actor, answer, events)?;
    }
    Ok(())
}

/// Whether `details` holds `artifact`, worn or carried (the original's `hero_has_artifact`).
pub(super) fn holds(details: &h3_heroes::HeroDetails, artifact: ArtifactTypeId) -> bool {
    details
        .worn
        .iter()
        .flatten()
        .any(|a| a.artifact == artifact)
        || details.backpack.iter().any(|a| a.artifact == artifact)
}

/// A War Machine Factory's stock for a dialog of `hero` (ADV-WMF-001; E08 T134b2): each machine
/// in its order, 1 while the hero holds none of its artifact, else 0.
pub(super) fn factory_stock(
    state: &GameState,
    factory: &h3_adventure::WarMachineFactory,
    hero: HeroId,
) -> Vec<(CreatureTypeId, u32)> {
    let details = state.hero(hero).unwrap();
    factory
        .machines
        .iter()
        .map(|&(machine, artifact)| (machine, u32::from(!holds(details, artifact))))
        .collect()
}

/// What the plans' games meet at the trades and services (E08 T135b2; increment 14's design note
/// §18, exit criteria 3 to 7 and 9): the seven laid out (by kind, [`TRADE_OBJECTS`]' order), the
/// decisions opened (a market, an Altar, a Hill Fort, a factory's question); the market's trades,
/// transfers, sales and purchases accepted and refused; the Altar's artifacts' and creatures'
/// sacrifices, those that opened a level-up and its return after it; the Hill Fort's single
/// upgrades and alls, the free ones among them; the factory's dialogs and purchases; the leaves.
#[derive(Debug, Default)]
struct Traded {
    laid: [usize; 7],
    opened: [usize; 4],
    trades: [usize; 2],
    transfers: [usize; 2],
    sales: [usize; 2],
    purchases: [usize; 2],
    artifacts: [usize; 2],
    creatures: [usize; 2],
    level_ups: usize,
    returns: usize,
    upgrades: [usize; 2],
    alls: [usize; 2],
    free: usize,
    dialogs: usize,
    machines: usize,
    leaves: usize,
}

impl Traded {
    fn count(
        &mut self,
        rules: &GameRuleset,
        (before, after): (&GameState, &GameState),
        (actor, command): (PlayerId, &GameCommand),
        (accepted, events): (bool, &[GameEvent]),
    ) {
        let i = usize::from(!accepted);
        for event in events.iter().filter(|_| accepted) {
            match event {
                GameEvent::MarketOpened { .. } => self.opened[0] += 1,
                GameEvent::AltarOpened { .. } => self.opened[1] += 1,
                GameEvent::HillFortOpened { .. } => self.opened[2] += 1,
                GameEvent::OfferOpened { object_type, .. }
                    if trade_kind(rules, *object_type) == Some(TradeKind::Factory) =>
                {
                    self.opened[3] += 1;
                }
                GameEvent::HireOpened {
                    source: HireSeen::Object { object_type, .. },
                    ..
                } if trade_kind(rules, *object_type) == Some(TradeKind::Factory) => {
                    self.dialogs += 1;
                }
                _ => {}
            }
        }
        let GameCommand::Answer(answer) = command else {
            return;
        };
        if trade_open(before, actor).is_some() {
            match answer {
                Answer::Market(MarketAnswer::Trade { .. }) => self.trades[i] += 1,
                Answer::Market(MarketAnswer::Transfer { .. }) => self.transfers[i] += 1,
                Answer::Market(MarketAnswer::Sell { .. }) => self.sales[i] += 1,
                Answer::Market(MarketAnswer::Buy { .. }) => self.purchases[i] += 1,
                Answer::Altar(AltarAnswer::Artifacts(_)) => self.artifacts[i] += 1,
                Answer::Altar(AltarAnswer::Creatures(_)) => self.creatures[i] += 1,
                Answer::HillFort(HillFortAnswer::Upgrade(_)) => self.upgrades[i] += 1,
                Answer::HillFort(HillFortAnswer::All) => self.alls[i] += 1,
                _ if accepted => self.leaves += 1,
                _ => {}
            }
            let paid = events
                .iter()
                .any(|e| matches!(e, GameEvent::ResourcePaid { .. }));
            let upgrades = matches!(
                answer,
                Answer::HillFort(HillFortAnswer::Upgrade(_) | HillFortAnswer::All)
            );
            if accepted && upgrades && !paid {
                self.free += 1;
            }
            if accepted && matches!(answer, Answer::Altar(_)) && after.level_up(actor).is_some() {
                self.level_ups += 1;
            }
        }
        if accepted && before.level_up(actor).is_some() && after.altar(actor).is_some() {
            self.returns += 1;
        }
        let machine = events.iter().any(|e| {
            matches!(
                e,
                GameEvent::HeroChanged {
                    change: HeroChange::AddArtifact(_),
                    ..
                }
            )
        });
        if accepted && matches!(answer, Answer::Hire(HireAnswer::Buy { .. })) && machine {
            self.machines += 1;
        }
    }
}

/// Plans with trades and services for [`the_plans_trade_and_use_the_services`] (E08 T135b2): at
/// least one, the difficulty other than impossible in three cases of four (gold to trade and buy
/// with), steps that mostly seek them ([`Step::Trade`]), some other options and turn ends.
fn trade_plans() -> impl Strategy<Value = (Plan, bool, u64, Vec<Step>)> {
    let steps = vec(
        prop_oneof![
            10 => any::<u16>().prop_map(Step::Trade),
            1 => any::<u16>().prop_map(Step::Legal),
            1 => Just(Step::EndTurn),
        ],
        0..200,
    );
    let plan = (trade_plan(), 0u8..16)
        .prop_filter("trades", |(p, _)| !p.trades.is_empty())
        .prop_map(|(mut plan, difficulty)| {
            if difficulty < 12 {
                plan.difficulty = difficulty % 4;
            }
            plan
        });
    (plan, any::<bool>(), any::<u64>(), steps)
}

/// The plans trade at the markets and use the services (E08 T135b2; increment 14's design note §18,
/// exit criteria 3 to 7 and 9): over a fixed set of cases (proptest's deterministic runner), with
/// the oracles the trades meet after every command, random play visits the trades and services
/// laid out ([`trade_plans`]) and meets each count of [`Traded`], so that the properties' checks
/// ([`check_legal`]'s [`check_trade_listing`], [`check_trades`], [`check_decisions`],
/// [`check_isolation`], [`check_conservation`]) see each.
// covers: GAM-DEC-008, GAM-DEC-011, GAM-DEC-012, GAM-DEC-013, ADV-MKT-001, ADV-MKT-003
// covers: ADV-MKT-004, ADV-MKT-005, ADV-MKT-006, ADV-MKT-007, ADV-ALT-001, ADV-ALT-002
// covers: ADV-ALT-003, ADV-HFT-001, ADV-HFT-002, ADV-WMF-001, ADV-HIR-002, GAM-DEC-007
#[test]
fn the_plans_trade_and_use_the_services() {
    use proptest::strategy::ValueTree;
    let mut runner = proptest::test_runner::TestRunner::deterministic();
    let strategy = trade_plans();
    let mut traded = Traded::default();
    for _ in 0..256 {
        let (plan, strict, seed, steps) = strategy.new_tree(&mut runner).unwrap().current();
        let rules = if strict { &*STRICT } else { &*FAITHFUL };
        let mut state = start(&plan, rules, seed);
        let m = lay_out(&plan);
        for o in &m.objects {
            let class = m.templates[usize::try_from(o.template).unwrap()].class;
            if let Some(at) = TRADE_CLASSES.iter().position(|&c| c == class) {
                traded.laid[at] += 1;
            }
        }
        for (i, step) in steps.iter().enumerate() {
            check_invariants(rules, &state).unwrap();
            let legal = options(&state, rules);
            let player = acting(&state);
            // The listing's oracle probes every listed command: on every fourth step, and at
            // every open trade.
            if i % 4 == 0 || trade_open(&state, player).is_some() {
                let reaches = reaches(&state, rules);
                check_legal(&state, rules, &legal, &reaches).unwrap();
            }
            let (actor, command) = resolve(&state, rules, &legal, step);
            let before = state.clone();
            let mut events = Vec::new();
            let result = apply(&mut state, rules, actor, command.clone(), &mut events);
            let player = player_of(&before, actor);
            let after = (&before, &state);
            if result.is_ok() {
                check_decisions(rules, after, player, &events).unwrap();
                check_isolation(&before, &state, player, &events).unwrap();
                check_conservation(rules, after, &command, &events).unwrap();
                check_trades(rules, after, (player, &command), &events).unwrap();
            } else {
                assert_eq!(state, before);
            }
            traded.count(rules, after, (player, &command), (result.is_ok(), &events));
        }
    }
    let t = &traded;
    eprintln!("{t:?}");
    assert!(t.laid.iter().all(|&n| n > 0), "{t:?}");
    assert!(t.opened.iter().all(|&n| n > 0), "{t:?}");
    for counts in [t.trades, t.transfers, t.sales, t.purchases] {
        assert!(counts.iter().all(|&n| n > 0), "{t:?}");
    }
    for counts in [t.artifacts, t.creatures, t.upgrades, t.alls] {
        assert!(counts.iter().all(|&n| n > 0), "{t:?}");
    }
    assert!(t.level_ups > 0 && t.returns > 0 && t.free > 0, "{t:?}");
    assert!(t.dialogs > 0 && t.machines > 0 && t.leaves > 0, "{t:?}");
}

/// The trades' and services' classes in [`TRADE_OBJECTS`]' order (their [`KINDS`]).
const TRADE_CLASSES: [u32; 7] = [99, 221, 213, 7, 2, 35, 106];
