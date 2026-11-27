//! The market's decision (GAM-DEC-011; E08 T133b, increment 14's design note §13; survey
//! `E08-trades.md` §5 to §7, the original's market dialog): a market's visit
//! (`AdventureOutcome::service`, ADV-MKT-001) as its visitor's owner's pending decision, told to it
//! alone, open until its `Leave`; a trade and a transfer by the treasury (ADV-MKT-003, -004), a
//! sale and a purchase through the adventure's checks (ADV-MKT-005, -007); its view (ADV-MKT-008).
//! The rate, the price and the most are the state's answer (ADR-0004: nothing from the payload);
//! the screens, the rates and the stock are the adventure's, read at each answer and view. Since E09
//! T58a1 (the town lane's, `cross-lane`; the town epic's design note §104) a town opens the same
//! decision with the town as its place (`TownCommand::Market`, TWN-MKT-002): every answer and view
//! reads the window as [`GameState::market_window`] resolves it, a town's by `crate::town::market`.

use alloc::vec::Vec;

use h3_adventure::{
    MarketError, MarketParams, MarketPlace, MarketScreen, MarketVisit, MarketWindow,
};
use h3_content::ResourceId;
use h3_contracts::PlayerId;
use h3_heroes::HeroChange;

use super::{Decision, GameState};
use crate::GameRuleset;
use crate::command::{GameCommandError, GameEvent, MarketAnswer, Sink};
use crate::observe::{MarketSeen, MarketView};

impl GameState {
    /// Opens the market's decision a visit of `player`'s hero made (GAM-DEC-011, ADV-MKT-001; E08
    /// T133b) or since E09 T58a1 a town's window (TWN-MKT-002, `crate::town::market::open`), told
    /// to it alone (`MarketOpened`), whatever its treasury and army hold (the original's window
    /// opens and disables what it cannot trade).
    // rule: GAM-DEC-011
    pub(crate) fn open_market<S: Sink>(
        &mut self,
        player: PlayerId,
        visit: MarketVisit,
        sink: &mut S,
    ) {
        sink.emit(GameEvent::MarketOpened {
            player,
            hero: visit.hero,
            place: MarketSeen::of(&visit.place),
        });
        self.push_decision(Decision::Market {
            owner: player,
            visit,
        });
    }

    /// A market's answer (GAM-DEC-011; E08 T133b, increment 14's design note §13), its decision
    /// at `at` kept open but for `Leave`:
    /// - a trade (ADV-MKT-003): the lot ([`h3_adventure::AdventureState::trade_lot`]), the lots
    ///   from 1 to what the treasury holds of the given divided by the lot's given, then the
    ///   given paid (`ResourcePaid`) and the wanted gained (`ResourceGained`, saturating,
    ///   GAM-RES-001; the product in 64 bits);
    /// - a transfer (ADV-MKT-004): the screen
    ///   ([`h3_adventure::AdventureState::check_screen`]), the recipient another player of the
    ///   game (every player until E10's elimination), the amount from 1 to the treasury's, then
    ///   paid (`ResourcePaid`, to the actor) and gained 1 : 1 by the recipient (`ResourceGained`,
    ///   to it alone, saturating), ally or enemy; no event names the giver to the recipient
    ///   (*agent's call*, design note §13);
    /// - a sale (ADV-MKT-005): the adventure's check
    ///   ([`h3_adventure::AdventureState::check_sale`]), then the stack lowered (`StackChanged`)
    ///   and the gain (`ResourceGained`);
    /// - a purchase (ADV-MKT-007): the adventure's check
    ///   ([`h3_adventure::AdventureState::check_buy`], A-46 fixed: the hero's room before
    ///   paying), then the price against the treasury, then paid (`ResourcePaid`), the artifact
    ///   given (`HeroChanged`) and the entry sold;
    /// - `Leave`: closed (`DecisionClosed`), nothing lost.
    ///
    /// # Errors
    ///
    /// [`GameCommandError::Market`] (the adventure's refusal; `Recipient`; `Amount` for a trade's
    /// lots or a transfer's amount), then [`GameCommandError::CannotPay`] for a purchase; the
    /// state is then unchanged.
    // rule: GAM-DEC-011, ADV-MKT-003, ADV-MKT-004, ADV-MKT-005, ADV-MKT-007, GAM-RES-001
    pub(super) fn answer_market<S: Sink>(
        &mut self,
        rules: &GameRuleset,
        (player, at): (PlayerId, usize),
        visit: &MarketVisit,
        answer: MarketAnswer,
        sink: &mut S,
    ) -> Result<(), GameCommandError> {
        let refused = GameCommandError::Market;
        // The window is resolved once; each answer's own check gives its refusal.
        let window = self.market_window(rules, player, visit);
        match answer {
            MarketAnswer::Trade { give, take, lots } => {
                let lot = window
                    .and_then(|w| {
                        self.adventure
                            .trade_lot(rules.adventure(), &w, (give, take))
                    })
                    .map_err(refused)?;
                // A lot gives at least one (ADV-MKT-002), so the most is the held amount at most.
                let most = self.held(player, give) / lot.give.max(1);
                if lots == 0 || lots > most {
                    return Err(refused(MarketError::Amount(lots)));
                }
                let gained = u64::from(lots) * u64::from(lot.take);
                self.pay_price(player, &[(give, lots * lot.give)], sink);
                let gained = u32::try_from(gained).unwrap_or(u32::MAX);
                self.gain(player, &[(take, gained)], sink);
            }
            MarketAnswer::Transfer {
                resource,
                amount,
                to,
            } => {
                window
                    .and_then(|w| w.check_screen(MarketScreen::Transfer))
                    .map_err(refused)?;
                if to == player || to.index() >= self.players.len() {
                    return Err(refused(MarketError::Recipient));
                }
                if amount == 0 || amount > self.held(player, resource) {
                    return Err(refused(MarketError::Amount(amount)));
                }
                self.pay_price(player, &[(resource, amount)], sink);
                self.gain(to, &[(resource, amount)], sink);
            }
            MarketAnswer::Sell { slot, take, lots } => {
                let checks = (rules.core(), rules.adventure());
                let sale = window
                    .and_then(|w| self.adventure.check_sale(checks, &w, (slot, take, lots)))
                    .map_err(refused)?;
                let gain = sale.gain();
                let at = self.adventure.sell(sale);
                let stack = self.stack_at(player, at);
                sink.emit(GameEvent::StackChanged { player, at, stack });
                self.gain(player, &[gain], sink);
            }
            MarketAnswer::Buy { entry, pay } => {
                let rules3 = (rules.core(), rules.adventure(), self.heroes.as_slice());
                let purchase = window
                    .and_then(|w| self.adventure.check_buy(rules3, &w, (entry, pay)))
                    .map_err(refused)?;
                let price = [purchase.price()];
                if !self.can_pay(player, &price) {
                    return Err(GameCommandError::CannotPay);
                }
                self.pay_price(player, &price, sink);
                let change = HeroChange::AddArtifact(purchase.artifact().into());
                self.change_hero(rules.core(), purchase.hero(), Some(player), change, sink);
                self.adventure.buy(purchase);
            }
            MarketAnswer::Leave => {
                self.close(at);
                sink.emit(GameEvent::DecisionClosed { player });
            }
        }
        Ok(())
    }

    /// The answers [`crate::legal_commands`] lists for `player`'s open market `visit` but its
    /// `Leave` (GAM-DEC-011; E08 T135a, increment 14's design note §18), each one `apply`
    /// accepts, a representative per action as the hire dialog's most, in this order: a trade of
    /// one lot for each pair the window values whose lot the treasury pays (the given, then the
    /// wanted, in the core's order: [`h3_adventure::MarketOffers::trades`]); a sale of one lot of
    /// each stack of the hero's army, in its order, for the market parameters' cost resource (the
    /// original's gold) whose check passes ([`h3_adventure::AdventureState::check_sale`]: the
    /// stack holds a lot, the army keeps a creature); a purchase in that resource of each entry
    /// of the stock whose check passes ([`h3_adventure::AdventureState::check_buy`]: unsold, the
    /// hero's room) and whose price the treasury holds. No transfer is listed (as the management
    /// commands, decision 99: a client sends one from the view); every other answer the view
    /// allows stays legal for `apply`.
    // rule: GAM-DEC-011, ADV-MKT-003, ADV-MKT-005, ADV-MKT-007
    pub(crate) fn market_answers(
        &self,
        rules: &GameRuleset,
        player: PlayerId,
        visit: &MarketVisit,
    ) -> Vec<MarketAnswer> {
        // As `market_view`'s: the window stays as it opened while the decision is open (its hero
        // on the map or in its town, no build), so none is defensive only.
        let window = self.market_window(rules, player, visit).ok();
        window
            .map(|window| self.window_answers(rules, player, &window))
            .unwrap_or_default()
    }

    /// Whether `window`, a window `player` would open, offers an answer [`Self::market_answers`]
    /// would list in it (E09 T58a2, the town epic's design note §104, §115; ADR-0017 point 6: the
    /// listing's `TownCommand::Market` where the window opens and offers a legal answer,
    /// `crate::town::market::listed`): a trade whose lot the treasury pays, asked first and alone
    /// ([`Self::pays_a_lot`] at the window's count: no list built, no lot's arithmetic; the
    /// listing asks it of every owned town whose window opens, `bench-check`'s `legal-towns-36`);
    /// without one and with a hero, the window's sales and purchases as [`Self::market_answers`]
    /// gives them.
    pub(crate) fn offers_an_answer(
        &self,
        rules: &GameRuleset,
        player: PlayerId,
        window: &MarketWindow<'_>,
    ) -> bool {
        let trade = window.shows(MarketScreen::Trade)
            && self.pays_a_lot(rules, player, |rates, give| {
                rates.cheapest_lot(window.count, give)
            });
        trade
            || (window.visit.hero.is_some()
                && !self.window_answers(rules, player, window).is_empty())
    }

    /// Whether `player`'s treasury holds, of a resource of the core, at least the least of it
    /// `least` gives a lot of a trade (E09 T58a2: [`MarketParams::cheapest_lot`] at a count, or
    /// [`MarketParams::cheapest_at_every_count`]): [`Self::market_answers`] lists a trade of that
    /// resource for another; none without the market parameters.
    pub(crate) fn pays_a_lot(
        &self,
        rules: &GameRuleset,
        player: PlayerId,
        least: impl Fn(&MarketParams, ResourceId) -> Option<u32>,
    ) -> bool {
        let rates = rules.adventure().params().market();
        rates.is_some_and(|rates| {
            (0..rules.core().resources().len())
                .filter_map(ResourceId::from_index)
                .any(|give| {
                    least(rates, give).is_some_and(|least| self.held(player, give) >= least)
                })
        })
    }

    /// [`Self::market_answers`] in `window`, the open visit's window as
    /// [`Self::market_window`] resolves it.
    fn window_answers(
        &self,
        rules: &GameRuleset,
        player: PlayerId,
        window: &MarketWindow<'_>,
    ) -> Vec<MarketAnswer> {
        let checks = (rules.core(), rules.adventure());
        let offers = self.adventure.market_offers(checks, window);
        // The part refuses a market kind without its parameters, so an open market has a cost.
        let cost = rules.adventure().params().market().map(MarketParams::cost);
        let trades = offers
            .trades
            .iter()
            .filter(|offer| self.held(player, offer.give) / offer.lot.give.max(1) > 0)
            .map(|offer| MarketAnswer::Trade {
                give: offer.give,
                take: offer.take,
                lots: 1,
            });
        let sales = offers
            .sales
            .iter()
            .flat_map(|offer| cost.map(|take| (offer.slot, take)))
            .filter(|&(slot, take)| {
                let sale = self.adventure.check_sale(checks, window, (slot, take, 1));
                sale.is_ok()
            })
            .map(|(slot, take)| MarketAnswer::Sell {
                slot,
                take,
                lots: 1,
            });
        let rules3 = (rules.core(), rules.adventure(), self.heroes.as_slice());
        let purchases = (0..=u8::MAX)
            .take(offers.stock.len())
            .flat_map(|entry| cost.map(|pay| (entry, pay)))
            .filter(|&(entry, pay)| {
                let purchase = self.adventure.check_buy(rules3, window, (entry, pay));
                purchase.is_ok_and(|purchase| self.can_pay(player, &[purchase.price()]))
            })
            .map(|(entry, pay)| MarketAnswer::Buy { entry, pay });
        trades.chain(sales).chain(purchases).collect()
    }

    /// The window of `player`'s open market `visit` (GAM-DEC-011; E09 T58a1, the town epic's
    /// design note §104): a map market's the adventure's
    /// ([`h3_adventure::AdventureState::market_window`]: its kind's screens and count), a town's
    /// the town's (`crate::town::market::window`: the screens its buildings and the window's hero
    /// allow, `player`'s count of Marketplaces). Refused as the adventure refuses a window
    /// ([`MarketError::Visit`]; defensive: the window cannot change while the decision is open).
    pub(crate) fn market_window<'r>(
        &self,
        rules: &'r GameRuleset,
        player: PlayerId,
        visit: &MarketVisit,
    ) -> Result<MarketWindow<'r>, MarketError> {
        match visit.place {
            MarketPlace::Town(town) => {
                crate::town::market::window(self, rules, player, (town, visit))
            }
            MarketPlace::Object { .. } => self.adventure.market_window(rules.adventure(), visit),
        }
    }

    /// How much of `resource` `player`'s treasury holds (none for a resource or a player the
    /// game lacks).
    fn held(&self, player: PlayerId, resource: ResourceId) -> u32 {
        self.treasury(player)
            .and_then(|held| held.get(resource.index()))
            .copied()
            .unwrap_or_default()
    }

    /// `visit` as its owner `player` sees it (ADV-MKT-008, GAM-DEC-011; E08 T133b, increment 14's
    /// design note §12, §13; E09 T58a1 at a town, TWN-VEW-005): the hero (none possible at a
    /// town), the place (a map market by type and anchor, never by index, ADR-0016 point 6; a town
    /// by its id), what its window offers ([`h3_adventure::AdventureState::market_offers`]: the
    /// screens, the count, the trades' lots, the stacks' lots, the stock with its prices) and with
    /// the transfer's screen the recipients, every other player of the game in order. The
    /// treasury is the owner's own observation's, so no most shows: a client draws the slider.
    pub(crate) fn market_view(
        &self,
        rules: &GameRuleset,
        player: PlayerId,
        visit: &MarketVisit,
    ) -> MarketView {
        // The window stays as it opened while the decision is open: the empty offers are
        // defensive only.
        let offers = self
            .market_window(rules, player, visit)
            .map(|w| {
                self.adventure
                    .market_offers((rules.core(), rules.adventure()), &w)
            })
            .unwrap_or_default();
        let transfers = offers.screens.contains(&MarketScreen::Transfer);
        let recipients: Vec<PlayerId> = (0..=u8::MAX)
            .take(self.players.len())
            .map(PlayerId::new)
            .filter(|&other| transfers && other != player)
            .collect();
        MarketView {
            hero: visit.hero,
            place: MarketSeen::of(&visit.place),
            offers,
            recipients,
        }
    }
}
