//! The prisoner freed (ADV-PRI-002, GAM-SET-001 amended: the game's half; E08 T141a, increment
//! 15's design note §15, §16): what the game does with a Prison's visit
//! ([`h3_adventure::AdventureOutcome::prison`]). No decision, no choice (ADR-0017 point 10).

use h3_adventure::PrisonVisit;
use h3_contracts::PlayerId;

use super::GameState;
use crate::GameRuleset;
use crate::command::{AdventureSink, GameEvent, Sink};
use crate::town::hire::HiredPart;
use crate::town::pool::HeroStatus;
use crate::town::rows::most_heroes;
use crate::town::tavern::TavernHeroView;

impl GameState {
    /// Settles `player`'s hero's visit of a Prison (ADV-PRI-002; the original's `visit_prison`
    /// 0x4A3EB0, survey `E08-spells.md` §2.2; batch E15's E15-1, E15-2), in the original's order:
    ///
    /// 1. a prisoner whose type the pool no longer holds as a prisoner's (only a campaign's
    ///    carried hero rewrites it: unreachable here): the Prison taken off the map
    ///    ([`h3_adventure::AdventureState::empty_prison`]), [`GameEvent::PrisonEmptied`];
    /// 2. the player at the town part's most heroes on the map, its towns' garrison heroes not
    ///    counted (as a hire's room, TWN-TAV-003): [`GameEvent::PrisonFull`], nothing changed;
    /// 3. else freed: its type taken by the player's colour, its bit added to the mask, no mask
    ///    tested ([`crate::town::HeroPool::free_prisoner`]), the hero placed on the Prison's
    ///    visitable tile last in the player's list with a full day's land points
    ///    ([`h3_adventure::AdventureState::free_prisoner`]: the Prison's `ObjectRemoved` first),
    ///    then [`GameEvent::PrisonerFreed`] with, for a sink that keeps events, the owner's view's
    ///    lists ([`GameEvent::PrisonerPartSeen`]) and its other parts
    ///    ([`GameEvent::HiredPartSeen`]).
    ///
    /// The visitor stays on its tile, untouched.
    // rule: ADV-PRI-002
    pub(crate) fn settle_prison<S: Sink>(
        &mut self,
        rules: &GameRuleset,
        player: PlayerId,
        visit: PrisonVisit,
        sink: &mut S,
    ) {
        let core = rules.core();
        let hero_type = self
            .hero(visit.prisoner)
            .map(|details| details.hero_type)
            .filter(|&t| self.pool().status(t) == Some(HeroStatus::Prisoner));
        let Some(hero_type) = hero_type else {
            self.adventure
                .empty_prison(core, &visit, &mut AdventureSink(sink));
            sink.emit(GameEvent::PrisonEmptied {
                player,
                hero: visit.hero,
            });
            return;
        };
        if !self.adventure.has_room(player, most_heroes(rules)) {
            sink.emit(GameEvent::PrisonFull {
                player,
                hero: visit.hero,
            });
            return;
        }
        let colour = self.colour(player).unwrap_or(u8::MAX);
        alloc::sync::Arc::make_mut(&mut self.pool).free_prisoner(hero_type, colour);
        let part = (core, rules.adventure(), self.heroes.as_slice());
        let seen_by = self
            .adventure
            .free_prisoner(part, &visit, player, &mut AdventureSink(sink));
        let details = self.hero(visit.prisoner);
        let on_map = self.adventure.hero(visit.prisoner);
        let view = details.zip(on_map).map(|(details, hero)| {
            TavernHeroView::new(
                visit.prisoner,
                details,
                (&hero.army, hero.formation, hero.tactics),
            )
        });
        sink.emit(GameEvent::PrisonerFreed {
            player,
            hero: visit.prisoner,
            hero_type,
            pos: visit.entrance,
            seen_by,
            view: view.as_ref().map(|view| view.hero),
        });
        if S::RECORDS {
            for part in view.iter().flat_map(TavernHeroView::parts) {
                sink.emit(GameEvent::PrisonerPartSeen {
                    player,
                    hero: visit.prisoner,
                    part,
                });
            }
            for part in details.into_iter().flat_map(HiredPart::of) {
                sink.emit(GameEvent::HiredPartSeen {
                    player,
                    hero: visit.prisoner,
                    part,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests;
