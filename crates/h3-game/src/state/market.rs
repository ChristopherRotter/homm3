//! The market's decision (GAM-DEC-011; E08 T133b, increment 14's design note §13; survey
//! `E08-trades.md` §5 to §7, the original's market dialog): a market's visit
//! (`AdventureOutcome::service`, ADV-MKT-001) as its visitor's owner's pending decision, told to it
//! alone, open until its `Leave`; a trade and a transfer by the treasury (ADV-MKT-003, -004), a
//! sale and a purchase through the adventure's checks (ADV-MKT-005, -007); its view (ADV-MKT-008).
//! The rate, the price and the most are the state's answer (ADR-0004: nothing from the payload);
//! the screens, the rates and the stock are the adventure's, read at each answer and view.

use alloc::vec::Vec;

use h3_adventure::{MarketError, MarketParams, MarketScreen, MarketVisit};
use h3_content::ResourceId;
use h3_contracts::PlayerId;
use h3_heroes::HeroChange;

use super::{Decision, GameState};
use crate::GameRuleset;
use crate::command::{GameCommandError, GameEvent, MarketAnswer, Sink};
use crate::observe::MarketView;

impl GameState {
    /// Opens the market's decision a visit of `player`'s hero made (GAM-DEC-011, ADV-MKT-001; E08
    /// T133b), told to it alone (`MarketOpened`), whatever its treasury and army hold (the
    /// original's window opens and disables what it cannot trade).
    // rule: GAM-DEC-011
    pub(super) fn open_market<S: Sink>(
        &mut self,
        player: PlayerId,
        visit: MarketVisit,
        sink: &mut S,
    ) {
        sink.emit(GameEvent::MarketOpened {
            player,
            hero: visit.hero,
            object_type: visit.object_type,
            anchor: visit.anchor,
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
        match answer {
            MarketAnswer::Trade { give, take, lots } => {
                let lot = self
                    .adventure
                    .trade_lot(rules.adventure(), visit, (give, take))
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
                self.adventure
                    .check_screen(rules.adventure(), visit, MarketScreen::Transfer)
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
                let sale = self
                    .adventure
                    .check_sale((rules.core(), rules.adventure()), visit, (slot, take, lots))
                    .map_err(refused)?;
                let gain = sale.gain();
                let at = self.adventure.sell(sale);
                let stack = self.stack_at(player, at);
                sink.emit(GameEvent::StackChanged { player, at, stack });
                self.gain(player, &[gain], sink);
            }
            MarketAnswer::Buy { entry, pay } => {
                let rules3 = (rules.core(), rules.adventure(), self.heroes.as_slice());
                let purchase = self
                    .adventure
                    .check_buy(rules3, visit, (entry, pay))
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
        let checks = (rules.core(), rules.adventure());
        // As `market_view`'s: the visit's hero stays on the map while the decision is open.
        let offers = self
            .adventure
            .market_offers(checks, visit)
            .unwrap_or_default();
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
                let sale = self.adventure.check_sale(checks, visit, (slot, take, 1));
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
                let purchase = self.adventure.check_buy(rules3, visit, (entry, pay));
                purchase.is_ok_and(|purchase| self.can_pay(player, &[purchase.price()]))
            })
            .map(|(entry, pay)| MarketAnswer::Buy { entry, pay });
        trades.chain(sales).chain(purchases).collect()
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
    /// design note §12, §13): the hero, the market by type and anchor (never by index, ADR-0016
    /// point 6), what its window offers ([`h3_adventure::AdventureState::market_offers`]: the
    /// screens, the count, the trades' lots, the stacks' lots, the stock with its prices) and with
    /// the transfer's screen the recipients, every other player of the game in order. The
    /// treasury is the owner's own observation's, so no most shows: a client draws the slider.
    pub(crate) fn market_view(
        &self,
        rules: &GameRuleset,
        player: PlayerId,
        visit: &MarketVisit,
    ) -> MarketView {
        // The visit's hero stays on the map while the decision is open: the empty offers are
        // defensive only.
        let offers = self
            .adventure
            .market_offers((rules.core(), rules.adventure()), visit)
            .unwrap_or_default();
        let transfers = offers.screens.contains(&MarketScreen::Transfer);
        let recipients: Vec<PlayerId> = (0..=u8::MAX)
            .take(self.players.len())
            .map(PlayerId::new)
            .filter(|&other| transfers && other != player)
            .collect();
        MarketView {
            hero: visit.hero,
            object_type: visit.object_type,
            anchor: visit.anchor,
            offers,
            recipients,
        }
    }
}
