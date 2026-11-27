//! The market at a town in the game (TWN-MKT-001 to -004; E09 T58a1, the town epic's design note
//! §103 to §106, decision 209; survey `E09-trades.md` §2, §3.3): `TownCommand::Market` opens E08's
//! market decision with the town as its place (GAM-DEC-011 amended), its hero the town's; every
//! answer and view of that decision reads the town's window as [`window`] resolves it (the
//! screens the town's buildings and the window's hero allow, the trader's count of Marketplaces),
//! the adventure's checks (`h3_adventure`'s `state/markets.rs`) the rest: E08's trades, transfer
//! and creature sale at the trader's count (TWN-MKT-003) and the Artifact Merchants' stock for the
//! purchase (TWN-MKT-004, `AdventureState::merchants_stock`).

use h3_adventure::{
    HeroOnMap, MarketError, MarketParams, MarketPlace, MarketScreen, MarketVisit, MarketWindow,
};
use h3_content::VISITOR_ONLY_TRADES;
use h3_contracts::{HeroId, PlayerId, TownId};
use h3_towns::{TownCommandError, TownMarket};

use super::command::owner;
use super::guild::heroes;
use crate::command::{GameCommandError, Sink};
use crate::{GameRuleset, GameState};

/// A town window's screens by what it offers beside the trades and the transfer (the original's
/// screens 0, 1, 2 and 4 in its order; survey §2.2): indexed by the merchants' purchase plus twice
/// the Freelancer's Guild's sale.
const SCREENS: [&[MarketScreen]; 4] = [
    &[MarketScreen::Trade, MarketScreen::Transfer],
    &[
        MarketScreen::Trade,
        MarketScreen::Transfer,
        MarketScreen::Buy,
    ],
    &[
        MarketScreen::Trade,
        MarketScreen::Transfer,
        MarketScreen::Sell,
    ],
    &[
        MarketScreen::Trade,
        MarketScreen::Transfer,
        MarketScreen::Buy,
        MarketScreen::Sell,
    ],
];

/// Opens `player`'s market window at `town` (TWN-MKT-002; E09 T58a1, design §104, decision 209,
/// as `Recruit` opens the hire dialog), `player`'s turn without an open decision checked by
/// `GameState::check_ready`, in this order, each refusal changing nothing:
///
/// 1. the town in the store of towns, else [`TownCommandError::UnknownTown`];
/// 2. its owner `player`, else [`TownCommandError::NotOwner`] (an ally's town waits for E10's
///    teams, `TWN-STUB-21`);
/// 3. the window's hero ([`window_hero`]), then [`h3_towns::check_market`]'s tests with it: a
///    Marketplace standing, or the Artifact Merchants or the Freelancer's Guild with a hero, else
///    [`TownCommandError::NoMarket`] or [`TownCommandError::NoVisitor`].
///
/// Then the window is `player`'s open decision (`GameState::open_market`, `MarketOpened` to it),
/// its place the town, its hero the window's (none possible), at any treasury and army (the
/// original's window opens and disables what it cannot trade). No movement points, no mark: any
/// number a turn.
///
/// # Errors
///
/// [`GameCommandError::Town`] with the refusal.
// rule: TWN-MKT-002, GAM-DEC-011
pub(crate) fn open<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    player: PlayerId,
    town: TownId,
    sink: &mut S,
) -> Result<(), GameCommandError> {
    let record = state
        .towns()
        .get(town)
        .ok_or(GameCommandError::Town(TownCommandError::UnknownTown))?;
    if owner(state, town) != Some(player) {
        return Err(GameCommandError::Town(TownCommandError::NotOwner));
    }
    let hero = window_hero(state, rules, (town, player));
    h3_towns::check_market(record, rules.towns(), hero.is_some())
        .map_err(GameCommandError::Town)?;
    let visit = MarketVisit {
        hero,
        place: MarketPlace::Town(town),
    };
    state.open_market(player, visit, sink);
    Ok(())
}

/// The window's hero at `player`'s `town` (TWN-MKT-002; design §101 call 2, ADR-0028 point 6, T-28,
/// `TWN-QUIRK-15`): the town's visitor when it is `player`'s; with the flag
/// `quirk.town.visitor_only_trades` off (`strict-fixed`) the town's garrison hero when no visitor
/// of `player`'s stands there (the original's merchants and guild serve the visitor alone, batch
/// TWN-6's TWN-6-5; its spell book, University, Transformer and Grail take the garrison hero);
/// none else (a Marketplace's window opens without one).
// rule: TWN-MKT-002
fn window_hero(
    state: &GameState,
    rules: &GameRuleset,
    (town, player): (TownId, PlayerId),
) -> Option<HeroId> {
    hero_of(rules, player, heroes(state, town))
}

/// [`window_hero`] of the town's visitor and garrison hero, each with its owner (the adventure's
/// records, `guild::heroes`; the listing's from the town's entrance, [`listed`]).
fn hero_of(
    rules: &GameRuleset,
    player: PlayerId,
    [visitor, garrison]: [Option<(HeroId, PlayerId)>; 2],
) -> Option<HeroId> {
    let own = |hero: Option<(HeroId, PlayerId)>| {
        hero.filter(|&(_, owner)| owner == player)
            .map(|(hero, _)| hero)
    };
    // stub: TWN-STUB-21 (an ally's visitor at an own town is not the window's hero until E10's
    // teams; its owner's trades at another's town wait for them too)
    let visitor = own(visitor);
    if rules.core().quirks().is_enabled(VISITOR_ONLY_TRADES) {
        visitor
    } else {
        visitor.or_else(|| own(garrison))
    }
}

/// Whether [`crate::legal_commands`] lists `player`'s `TownCommand::Market` at its own `town`
/// (TWN-MKT-002; E09 T58a2, design §104, §115, ADR-0017 point 6; `town::offer::legal_builds`
/// asks it in `player`'s turn without a decision): the window [`open`] would open
/// ([`h3_towns::check_market`] with the window's hero, [`hero_of`] of the town's visitor and
/// garrison hero as the listing found them, read only where the merchants or the guild stand: a
/// Marketplace's window alone offers the same with a hero or without) offers an answer the listing
/// gives inside it: a trade whose lot the treasury pays at every count
/// (`MarketParams::cheapest_at_every_count`, no count needed), else at `player`'s count
/// (`GameState::offers_an_answer`: such a trade, else a sale or a purchase). `counted` is that
/// count of Marketplaces ([`count`]), computed at the listing's first window that needs it and
/// kept for the rest (one walk over the player's towns a listing at most).
// rule: TWN-MKT-002
pub(crate) fn listed(
    state: &GameState,
    rules: &GameRuleset,
    (town, player): (TownId, PlayerId),
    (visitor, garrison): (Option<&HeroOnMap>, Option<&HeroOnMap>),
    counted: &mut Option<u8>,
) -> bool {
    // The town's market buildings first, as if a hero stood there (a town without one lists
    // none, whoever stands there; a town the store lacks neither: no game holds one).
    let market = |hero: bool| {
        let record = state.towns().get(town);
        record.and_then(|record| h3_towns::check_market(record, rules.towns(), hero).ok())
    };
    let Some(with) = market(true) else {
        return false;
    };
    // Only the merchants' and the guild's screens read the window's hero: a Marketplace's
    // window alone offers the same without one.
    let reads = with.merchants || with.freelancers;
    let heroes = [visitor, garrison].map(|on| on.map(|on| (on.id, on.owner)));
    let hero = reads.then(|| hero_of(rules, player, heroes)).flatten();
    let offer = if reads && hero.is_none() {
        market(false)
    } else {
        Some(with)
    };
    let Some(offer) = offer else {
        return false;
    };
    // A trade's lot the treasury pays at every count: listed without the count (every town's
    // window shows the trades).
    if state.pays_a_lot(rules, player, MarketParams::cheapest_at_every_count) {
        return true;
    }
    let count = *counted.get_or_insert_with(|| count(state, rules, player));
    let window = MarketWindow {
        visit: MarketVisit {
            hero,
            place: MarketPlace::Town(town),
        },
        screens: screens(offer),
        count,
    };
    state.offers_an_answer(rules, player, &window)
}

/// The screens of a town's window offering `offer` ([`SCREENS`]' order).
fn screens(offer: TownMarket) -> &'static [MarketScreen] {
    SCREENS[usize::from(offer.merchants) + 2 * usize::from(offer.freelancers)]
}

/// The window of `player`'s open market `visit` at `town` (TWN-MKT-001 to -004; design §103,
/// §104): the screens [`h3_towns::check_market`] allows with the visit's hero (the trades and the
/// transfer, with a hero the merchants' purchase and the guild's sale where their building
/// stands; [`SCREENS`]' order) and the trader's count ([`count`]). The town cannot change while
/// the window is open (a decision refuses every other command), so [`MarketError::Visit`] for a
/// town the store lost or a window it no longer opens is defensive only.
// rule: TWN-MKT-001, TWN-MKT-002
pub(crate) fn window<'r>(
    state: &GameState,
    rules: &'r GameRuleset,
    player: PlayerId,
    (town, visit): (TownId, &MarketVisit),
) -> Result<MarketWindow<'r>, MarketError> {
    state
        .towns()
        .get(town)
        .and_then(|record| h3_towns::check_market(record, rules.towns(), visit.hero.is_some()).ok())
        .map(|offer| MarketWindow {
            visit: *visit,
            screens: screens(offer),
            count: count(state, rules, player),
        })
        .ok_or(MarketError::Visit)
}

/// The Marketplaces `player` trades as (TWN-MKT-001; design §103, D-70, TD-29): its own towns with
/// a Marketplace ever built ([`h3_towns::marketplaces`], whoever owns the town traded at, a
/// capture moving a town's Marketplace to its new owner's count at once), at most the market
/// parameters' counts (10) and **at least 1** (E09 decision 203: a trader without one trades as
/// with one, so the merchants never give an artifact for nothing; T-19 fixed, `TWN-QUIRK-17`).
// rule: TWN-MKT-001
pub(crate) fn count(state: &GameState, rules: &GameRuleset, player: PlayerId) -> u8 {
    let adventure = state.adventure();
    let map = adventure.map();
    let own = map
        .towns()
        .iter()
        .filter(|&&index| adventure.owner(index) == Some(player))
        .filter_map(|&index| map.town_id(index));
    let counted = h3_towns::marketplaces(state.towns(), rules.towns(), own);
    let most = rules
        .adventure()
        .params()
        .market()
        .map_or(1, MarketParams::counts);
    u8::try_from(counted)
        .unwrap_or(u8::MAX)
        .clamp(1, most.max(1))
}
