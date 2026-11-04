//! The hire dialog (GAM-DEC-008; E08 T95b, increment 10's design note §13; survey
//! `E08-dwellings.md` §5, the original's `H3RecruitMgr`): an accepted hire question's dialog
//! (`AdventureOutcome::hire`, ADV-HIR-002) as its owner's pending decision, told to it alone; a
//! purchase checked by the map (the creature, the amount, the room), then against the treasury,
//! then paid and made by the map, a single creature's dialog closing after it; the most its owner
//! may buy of each creature, which bounds the legal set and [`crate::hire_range`]; its view. The
//! treasury and the most are the game's (ADR-0004: the price is the state's answer, nothing from
//! the payload); the dialog's creatures and counts are the adventure's, read at each step.

use alloc::vec::Vec;

use h3_adventure::{HireDialog, HireSource, hire_price};
use h3_content::CreatureTypeId;
use h3_contracts::PlayerId;

use super::{Decision, GameState};
use crate::GameRuleset;
use crate::command::{AdventureSink, GameCommandError, GameEvent, Sink};
use crate::observe::{HireCreature, HireView};

impl GameState {
    /// Opens the hire dialog an accepted hire question of `player`'s hero gave (GAM-DEC-008, E08
    /// T95b) as a pending decision of the player, whatever its treasury holds (the original's
    /// dialog opens and disables what it cannot pay), told to it alone (`HireOpened`: the army,
    /// the source by type and anchor).
    // rule: GAM-DEC-008
    pub(super) fn open_hire<S: Sink>(
        &mut self,
        player: PlayerId,
        dialog: HireDialog,
        sink: &mut S,
    ) {
        let HireSource::Object {
            object_type,
            anchor,
            ..
        } = dialog.source;
        self.push_decision(Decision::Hire {
            owner: player,
            dialog,
        });
        sink.emit(GameEvent::HireOpened {
            player,
            target: dialog.target,
            object_type,
            anchor,
        });
    }

    /// A hire dialog's purchase of `amount` `creature`s (GAM-DEC-008, ADV-HIR-002; E08 T95b; the
    /// original's `recruit_dialog_proc` 0x550940, survey `E08-dwellings.md` §5.3): the map checks
    /// it ([`h3_adventure::AdventureState::check_hire`]), then the treasury must pay its price;
    /// then the price is paid (`ResourcePaid` per resource, in the core's order) and the map makes
    /// the purchase (the source's count lowered, the creatures joined, `CreaturesJoined` to the
    /// owner). A dialog whose source sells one creature then closes (`DecisionClosed`), as the
    /// original's; one of several stays open, its counts read again at the next step.
    ///
    /// # Errors
    ///
    /// `Hire` (the map's refusal: the creature not the source's, the amount 0 or above its count,
    /// no room in the army), then `CannotPay`; the state is then unchanged.
    // rule: GAM-DEC-008, ADV-HIR-002
    pub(super) fn hire<S: Sink>(
        &mut self,
        rules: &GameRuleset,
        (player, at): (PlayerId, usize),
        dialog: &HireDialog,
        buy: (CreatureTypeId, u32),
        sink: &mut S,
    ) -> Result<(), GameCommandError> {
        let purchase = self
            .adventure
            .check_hire((rules.core(), rules.adventure()), dialog, buy)
            .map_err(GameCommandError::Hire)?;
        if !self.treasuries.can_pay(player, purchase.price()) {
            return Err(GameCommandError::CannotPay);
        }
        self.pay_price(player, purchase.price(), sink);
        self.adventure.hire(purchase, &mut AdventureSink(sink));
        let single = self
            .adventure
            .hire_stock(rules.adventure(), &dialog.source)
            .is_some_and(|stock| stock.len() == 1);
        if single {
            self.close(at);
            sink.emit(GameEvent::DecisionClosed { player });
        }
        Ok(())
    }

    /// The hire dialog of `player`'s first open decision, if it is one (GAM-DEC-008, E08 T95b),
    /// as [`Self::university`] gives a University: its target and source by name; the source's
    /// creatures and counts are the adventure's (`AdventureState::hire_stock`). The env reads
    /// its creatures' order from it (action encoding v7).
    pub fn hire_dialog(&self, player: PlayerId) -> Option<&HireDialog> {
        match self.first_decision(player)? {
            Decision::Hire { dialog, .. } => Some(dialog),
            _ => None,
        }
    }

    /// Each creature `player`'s hire `dialog` sells, in the source's order, with the most the
    /// player may buy of it now (GAM-DEC-008; the original's `recruit_dialog_select` 0x5503A0,
    /// survey `E08-dwellings.md` §5.2): the least of its count, the treasury divided by its cost
    /// for each resource it costs more than 0 of (integer division), and the room's bound
    /// ([`h3_adventure::AdventureState::hire_room`]). *Agent's call* (increment 10's design note
    /// §13): the room in the bound, so every amount from 1 to the most is accepted; the original's
    /// slider leaves it out and refuses at Recruit, the same rule. None for a source that is gone
    /// (defensive: nothing moves the hero or the source while the dialog is open).
    // rule: GAM-DEC-008, ADV-HIR-002
    pub(crate) fn hire_mosts(
        &self,
        rules: &GameRuleset,
        player: PlayerId,
        dialog: &HireDialog,
    ) -> Vec<(CreatureTypeId, u32)> {
        let treasury = self.treasuries.of(player).unwrap_or_default();
        let stock = self.adventure.hire_stock(rules.adventure(), &dialog.source);
        stock
            .unwrap_or_default()
            .into_iter()
            .map(|(creature, count)| {
                let room = self
                    .adventure
                    .hire_room(rules.adventure(), &dialog.target, creature);
                let most = hire_price(rules.core(), creature, 1)
                    .into_iter()
                    .map(|(resource, each)| {
                        let held = treasury.get(usize::from(resource.raw())).copied();
                        held.unwrap_or(0) / each
                    })
                    .fold(count.min(room), u32::min);
                (creature, most)
            })
            .collect()
    }

    /// `dialog` as its owner sees it (GAM-DEC-008, E08 T95b; ADR-0016 point 6): the army, the
    /// source by type and anchor, and each of its creatures in its order with its count and the
    /// cost of one by resource ([`hire_price`]). The treasury is the owner's own observation's, so
    /// no most shows: a client draws the slider.
    pub(crate) fn hire_view(&self, rules: &GameRuleset, dialog: &HireDialog) -> HireView {
        let HireSource::Object {
            object_type,
            anchor,
            ..
        } = dialog.source;
        let stock = self.adventure.hire_stock(rules.adventure(), &dialog.source);
        HireView {
            target: dialog.target,
            object_type,
            anchor,
            creatures: stock
                .unwrap_or_default()
                .into_iter()
                .map(|(creature, count)| HireCreature {
                    creature,
                    count,
                    cost: hire_price(rules.core(), creature, 1),
                })
                .collect(),
        }
    }
}
