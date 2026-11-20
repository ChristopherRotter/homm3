//! The Hill Fort's decision (GAM-DEC-013; E08 T134b1, increment 14's design note §15; survey
//! `E08-trades.md` §13, the original's Hill Fort dialog): a Hill Fort's visit
//! (`AdventureOutcome::service`, ADV-HFT-001) as its visitor's owner's pending decision, told to
//! it alone, open until its `Leave`; a row's upgrade or all of them (ADV-HFT-002) through the
//! adventure's checks, then the treasury, each row paid and its stack upgraded; its view with the
//! original's states. The rows and prices are the state's answer (ADR-0004: nothing from the
//! payload).

use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use h3_adventure::{HillFortRow, HillFortUpgrade, HillFortVisit};
use h3_content::ResourceId;
use h3_contracts::PlayerId;

use super::{Decision, GameState};
use crate::GameRuleset;
use crate::command::{GameCommandError, GameEvent, HillFortAnswer, Sink};
use crate::observe::{HillFortRowView, HillFortState, HillFortView};

/// The sum of `prices` by resource in the core's order, saturating (the whole's price, the
/// original's +0x298 test).
fn sum_prices<'a>(prices: impl Iterator<Item = &'a [(ResourceId, u32)]>) -> Vec<(ResourceId, u32)> {
    let mut sum = BTreeMap::new();
    for &(resource, amount) in prices.flatten() {
        let held: &mut u32 = sum.entry(resource).or_default();
        *held = held.saturating_add(amount);
    }
    sum.into_iter().collect()
}

impl GameState {
    /// Opens the Hill Fort's decision a visit of `player`'s hero made (GAM-DEC-013, ADV-HFT-001;
    /// E08 T134b1), told to it alone (`HillFortOpened`), whatever its army holds and its treasury
    /// pays (the original's dialog opens with its rows' states).
    // rule: GAM-DEC-013
    pub(super) fn open_hill_fort<S: Sink>(
        &mut self,
        player: PlayerId,
        visit: HillFortVisit,
        sink: &mut S,
    ) {
        sink.emit(GameEvent::HillFortOpened {
            player,
            hero: visit.hero,
            object_type: visit.object_type,
            anchor: visit.anchor,
        });
        self.push_decision(Decision::HillFort {
            owner: player,
            visit,
        });
    }

    /// A Hill Fort's answer (GAM-DEC-013; E08 T134b1, increment 14's design note §15), its
    /// decision at `at`:
    /// - an upgrade (ADV-HFT-002): the adventure's check of the row
    ///   ([`h3_adventure::AdventureState::check_upgrade`]) or of all
    ///   ([`h3_adventure::AdventureState::check_upgrade_all`]), then their summed price against
    ///   the treasury (the row's or the whole's state 1: nothing else is tested, as the
    ///   original's press), then each in slot order paid (`ResourcePaid` per resource, nothing for
    ///   a free one) and its stack upgraded (`StackChanged`); the Hill Fort stays open;
    /// - `Leave`: closed (`DecisionClosed`).
    ///
    /// # Errors
    ///
    /// [`GameCommandError::HillFort`] (the adventure's refusal), then
    /// [`GameCommandError::CannotPay`]; the state is then unchanged.
    // rule: GAM-DEC-013, ADV-HFT-002
    pub(super) fn answer_hill_fort<S: Sink>(
        &mut self,
        rules: &GameRuleset,
        (player, at): (PlayerId, usize),
        visit: &HillFortVisit,
        answer: HillFortAnswer,
        sink: &mut S,
    ) -> Result<(), GameCommandError> {
        let checks = (rules.core(), rules.adventure());
        let checked = match answer {
            HillFortAnswer::Upgrade(slot) => self
                .adventure
                .check_upgrade(checks, visit, slot)
                .map(|upgrade| vec![upgrade]),
            HillFortAnswer::All => self.adventure.check_upgrade_all(checks, visit),
            HillFortAnswer::Leave => {
                self.close(at);
                sink.emit(GameEvent::DecisionClosed { player });
                return Ok(());
            }
        };
        let upgrades = checked.map_err(GameCommandError::HillFort)?;
        let price = sum_prices(upgrades.iter().map(HillFortUpgrade::price));
        if !self.can_pay(player, &price) {
            return Err(GameCommandError::CannotPay);
        }
        for upgrade in upgrades {
            self.pay_price(player, upgrade.price(), sink);
            let at = self.adventure.upgrade(upgrade);
            let stack = self.stack_at(player, at);
            sink.emit(GameEvent::StackChanged { player, at, stack });
        }
        Ok(())
    }

    /// `visit` as its owner `player` sees it (ADV-HFT-002, GAM-DEC-013; E08 T134b1, increment
    /// 14's design note §15; survey §13): the hero, the Hill Fort by type and anchor (never by
    /// index, ADR-0016 point 6), each row ([`h3_adventure::AdventureState::hill_fort_rows`]) with
    /// its state against the treasury, and the whole's price and state.
    pub(crate) fn hill_fort_view(
        &self,
        rules: &GameRuleset,
        player: PlayerId,
        visit: &HillFortVisit,
    ) -> HillFortView {
        let checks = (rules.core(), rules.adventure());
        // The visit's hero stays on the map while the decision is open: the empty rows are
        // defensive only.
        let rows = self
            .adventure
            .hill_fort_rows(checks, visit)
            .unwrap_or_default();
        let state = |upgrades: bool, price: &[(ResourceId, u32)]| match upgrades {
            false => HillFortState::NoUpgrade,
            true if self.can_pay(player, price) => HillFortState::Affordable,
            true => HillFortState::TooDear,
        };
        let upgrading = |row: &&HillFortRow| row.upgrade.is_some();
        let all = sum_prices(rows.iter().filter(upgrading).map(|row| &row.price[..]));
        let all_state = state(rows.iter().any(|row| upgrading(&row)), &all);
        HillFortView {
            hero: visit.hero,
            object_type: visit.object_type,
            anchor: visit.anchor,
            rows: rows
                .into_iter()
                .map(|row| HillFortRowView {
                    state: state(row.upgrade.is_some(), &row.price),
                    row,
                })
                .collect(),
            all,
            all_state,
        }
    }
}
