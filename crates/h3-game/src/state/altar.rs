//! The Altar of Sacrifice's decision (GAM-DEC-012; E08 T134a, increment 14's design note §14;
//! survey `E08-trades.md` §12, the original's altar dialog): an Altar's visit
//! (`AdventureOutcome::service`, ADV-ALT-001) as its visitor's owner's pending decision, told to it
//! alone, open until its `Leave`; a sacrifice of artifacts (ADV-ALT-002) or creatures (ADV-ALT-003)
//! through the adventure's checks, named whole in one answer (A-48 fixed as `ADV-QUIRK-50`), its
//! experience given with its levels; the Altar closed before a level-up its levels open and its
//! visit held as the visit's rest, so it returns once the level-ups are answered (GAM-DEC-007
//! amended, decision 371); its view. The experience is the state's answer (ADR-0004: nothing from
//! the payload).

use alloc::vec::Vec;

use h3_adventure::{ALTAR_SLOTS, AltarPlaces, AltarVisit};
use h3_contracts::PlayerId;
use h3_heroes::HeroChange;

use super::levels::open_level_up;
use super::rests::Rest;
use super::{Decision, GameState};
use crate::GameRuleset;
use crate::command::{AltarAnswer, GameCommandError, GameEvent, Sink};
use crate::observe::AltarView;

impl GameState {
    /// Opens the Altar's decision a visit of `player`'s hero made (GAM-DEC-012, ADV-ALT-001; E08
    /// T134a), told to it alone (`AltarOpened`), whatever its hero holds (the original's dialog
    /// opens with its Sacrifice disabled while the total is 0). Also the return after the
    /// level-ups a sacrifice opened (the held visit, GAM-DEC-007).
    // rule: GAM-DEC-012
    pub(super) fn open_altar<S: Sink>(
        &mut self,
        player: PlayerId,
        visit: AltarVisit,
        sink: &mut S,
    ) {
        sink.emit(GameEvent::AltarOpened {
            player,
            hero: visit.hero,
            object_type: visit.object_type,
            anchor: visit.anchor,
        });
        self.push_decision(Decision::Altar {
            owner: player,
            visit,
        });
    }

    /// An Altar's answer (GAM-DEC-012; E08 T134a, increment 14's design note §14), its decision
    /// at `at`:
    /// - a sacrifice (ADV-ALT-002, -003): the adventure's check
    ///   ([`h3_adventure::AdventureState::check_artifact_sacrifice`],
    ///   [`h3_adventure::AdventureState::check_creature_sacrifice`]), then the artifacts removed
    ///   (`HeroChanged` each: the worn ones by position, then the backpack's from the last index
    ///   down) or the stacks lowered (`StackChanged` each, in slot order), then the experience
    ///   (`HeroChanged`; the scenario's level limit's cap, HER-EXP-003), then the levels
    ///   (GAM-DEC-002): those with one offer or none at once (`LevelGained`), the Altar staying
    ///   open; at the first with two offers the Altar closes (`DecisionClosed`), its visit is held
    ///   as the visit's rest and the level-up opens (`LevelUpOpened`): once the level-ups are
    ///   answered the rest visits the Altar again (`AltarOpened`), so a later sacrifice reads the
    ///   new level's Learning (the original's dialog open beneath its level-up dialogs; decision
    ///   371);
    /// - `Leave`: closed (`DecisionClosed`), nothing lost (nothing lies on the altar between
    ///   commands: A-48 fixed).
    ///
    /// # Errors
    ///
    /// [`GameCommandError::Altar`] (the adventure's refusal); the state is then unchanged.
    // rule: GAM-DEC-012, ADV-ALT-002, ADV-ALT-003, GAM-DEC-002, GAM-DEC-007
    pub(super) fn answer_altar<S: Sink>(
        &mut self,
        rules: &GameRuleset,
        (player, at): (PlayerId, usize),
        visit: &AltarVisit,
        answer: AltarAnswer,
        sink: &mut S,
    ) -> Result<(), GameCommandError> {
        let checks = (rules.core(), rules.adventure(), self.heroes.as_slice());
        let sacrifice = match answer {
            AltarAnswer::Artifacts(places) => self
                .adventure
                .check_artifact_sacrifice(checks, visit, places),
            AltarAnswer::Creatures(amounts) => self
                .adventure
                .check_creature_sacrifice(checks, visit, amounts),
            AltarAnswer::Leave => {
                self.close(at);
                sink.emit(GameEvent::DecisionClosed { player });
                return Ok(());
            }
        }
        .map_err(GameCommandError::Altar)?;
        let made = self.adventure.sacrifice(sacrifice);
        let (core, hero) = (rules.core(), made.hero);
        for place in made.removed {
            let change = HeroChange::RemoveArtifact(place);
            self.change_hero(core, hero, Some(player), change, sink);
        }
        for at in made.lowered {
            let stack = self.stack_at(player, at);
            sink.emit(GameEvent::StackChanged { player, at, stack });
        }
        let change = HeroChange::GainExperience(made.experience);
        self.change_hero(core, hero, Some(player), change, sink);
        // rule: GAM-DEC-002, GAM-DEC-007
        if let Some((view, offered)) = self.give_plain_levels(rules, (hero, player), sink) {
            self.close(at);
            sink.emit(GameEvent::DecisionClosed { player });
            self.hold(player, hero, Rest::Visit);
            let level_up = open_level_up(player, view, offered, sink);
            self.push_decision(level_up);
        }
        Ok(())
    }

    /// The answers [`crate::legal_commands`] lists for an open Altar `visit` but its `Leave`
    /// (GAM-DEC-012; E08 T135a, increment 14's design note §18), each one `apply` accepts, in
    /// this order: the artifacts' all, every artifact of the hero the Altar takes (none for a
    /// kept one), the worn ones by position, then the backpack's in its order, until the Altar's
    /// places, when its check passes ([`h3_adventure::AdventureState::check_artifact_sacrifice`]:
    /// the screen, a total above 0); the creatures' all, every stack whole but one creature of the
    /// lowest occupied slot (the original's `altar_all_creatures` 0x564CF0: the army keeps one),
    /// when its check passes ([`h3_adventure::AdventureState::check_creature_sacrifice`]: the
    /// screen, a total above 0). The original's all button for artifacts (`altar_all_to_altar`
    /// 0x564340) takes the worn positions 0 to 15 alone: the listed all takes every position, the
    /// fifth misc slot too (decision 390; the Catapult and the Spell Book are kept anyway). Every
    /// other sacrifice the view allows stays legal for `apply`.
    // rule: GAM-DEC-012, ADV-ALT-002, ADV-ALT-003
    pub(crate) fn altar_answers(
        &self,
        rules: &GameRuleset,
        visit: &AltarVisit,
    ) -> Vec<AltarAnswer> {
        let checks = (rules.core(), rules.adventure(), self.heroes.as_slice());
        // As `altar_view`'s: the visit's hero stays on the map while the decision is open.
        let offers = self
            .adventure
            .altar_offers(checks, visit)
            .unwrap_or_default();
        let places = offers
            .artifacts
            .iter()
            .filter(|offer| offer.experience.is_some())
            .take(usize::from(offers.places))
            .fold(AltarPlaces::NONE, |places, offer| {
                places.with(offer.place).unwrap_or(places)
            });
        let army = self
            .adventure
            .hero(visit.hero)
            .map_or(&[][..], |hero| &hero.army[..]);
        let mut amounts = [0; ALTAR_SLOTS];
        for (slot, amount) in (0_u8..).zip(&mut amounts) {
            *amount = army
                .iter()
                .find(|stack| stack.slot == slot)
                .map_or(0, |stack| stack.count);
        }
        // The lowest occupied slot keeps a creature (its stack holds one at least).
        let occupied = amounts.iter_mut().filter(|amount| **amount > 0);
        occupied.take(1).for_each(|lowest| *lowest -= 1);
        let adventure = &self.adventure;
        let artifacts = adventure
            .check_artifact_sacrifice(checks, visit, places)
            .is_ok()
            .then_some(AltarAnswer::Artifacts(places));
        let creatures = adventure
            .check_creature_sacrifice(checks, visit, amounts)
            .is_ok()
            .then_some(AltarAnswer::Creatures(amounts));
        artifacts.into_iter().chain(creatures).collect()
    }

    /// `visit` as its owner sees it (ADV-ALT-001 to -003, GAM-DEC-012; E08 T134a, increment 14's
    /// design note §14): the hero, the Altar by type and anchor (never by index, ADR-0016 point
    /// 6) and what its window offers ([`h3_adventure::AdventureState::altar_offers`]: the screens,
    /// the places, each artifact's and each stack's experience, the experience to the next level).
    pub(crate) fn altar_view(&self, rules: &GameRuleset, visit: &AltarVisit) -> AltarView {
        let checks = (rules.core(), rules.adventure(), self.heroes.as_slice());
        // The visit's hero stays on the map while the decision is open: the empty offers are
        // defensive only.
        let offers = self
            .adventure
            .altar_offers(checks, visit)
            .unwrap_or_default();
        AltarView {
            hero: visit.hero,
            object_type: visit.object_type,
            anchor: visit.anchor,
            offers,
        }
    }
}
