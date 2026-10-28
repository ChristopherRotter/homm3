//! Events (E08 T62b2; design §12 and §13 of `docs/epics/E08-design.md`, survey
//! `docs/rules/surveys/E08-rewards.md` §7.3 and §8.5): the step that fires one, its guards at once,
//! its reward record and its removal, and what its visitor's owner is told. An event is never
//! visited and never observed: it lies on its tile apart from the tiles objects are visited from
//! ([`super::StaticMap::events_at`]), so the movement and the reachable tiles cross it as an
//! empty tile (A-23 withdrawn, E08 decision 131) and the views leave it out (ADV-FOG-007).

use alloc::sync::Arc;
use alloc::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::{ArmyStack, HeroId, PlayerId};
use h3_heroes::HeroDetails;

use super::AdventureState;
use crate::command::{AdventureEvent, AdventureEventKind, AdventureOutcome, Sink, Viewers};
use crate::ruleset::AdventureRuleset;
use crate::setup::Pos;

/// The rules a visit reads: the core, the ruleset part and the game's store of hero details.
type Rules<'a> = (&'a Ruleset, &'a AdventureRuleset, &'a [Arc<HeroDetails>]);

impl AdventureState {
    /// The event lying on `pos` still on the map, by index in the static map, if one does: the
    /// last of them in map order, as the tile's one object (ADV-OBJ-003; the original keeps one
    /// object word per tile, the last placed).
    pub fn event_at(&self, pos: Pos) -> Option<usize> {
        self.map
            .events_at(pos)
            .iter()
            .rev()
            .copied()
            .find(|&index| !self.is_removed(index))
    }

    /// The event a step of `player`'s hero onto `pos` fires, if one does (ADV-OBJ-007; the
    /// original's `event_fires_for` 0x4FCDC0, tested at 0x4811A3): the event on the tile
    /// ([`Self::event_at`]) whose colours hold the player's seat's colour, for a player that does
    /// not count as human only with its computer flag. On a tile a monster guards the original
    /// lets the guard visit first and fires the event after it (0x48169D, 0x481741): the move
    /// asks the guard and holds the event ([`AdventureOutcome::event_after`], E08 T63b), fired
    /// for a hero still on its tile only ([`Self::fire_held_event`]; A-26 fixed, `ADV-QUIRK-29`).
    // rule: ADV-OBJ-007
    pub(crate) fn firing_event(&self, player: PlayerId, pos: Pos) -> Option<usize> {
        let seat = self.seats[usize::from(player.raw())];
        let colour = 1u16 << seat.colour.min(8);
        self.event_at(pos).filter(|&index| {
            self.map.objects()[index].event().is_some_and(|event| {
                u16::from(event.colours) & colour != 0 && (seat.human || event.computer)
            })
        })
    }

    /// The event at `tile` held while the monster guarding it visited `player`'s hero `hero`
    /// (ADV-OBJ-007, E08 T63b; [`AdventureOutcome::event_after`]): once the guard's visit is over
    /// (its offers answered, its battle and the levels it gave), it fires ([`Self::fire_event`])
    /// for a hero still on its tile that it still fires for ([`Self::firing_event`]). The
    /// original fires it whatever the guard's visit ended in, for a hero the battle took off the
    /// map too (A-26, state-corrupting: fixed, `ADV-QUIRK-29`). Nothing otherwise.
    // rule: ADV-OBJ-007, ADV-MON-006
    pub fn fire_held_event<S: Sink>(
        &mut self,
        rules: Rules<'_>,
        (player, hero): (PlayerId, HeroId),
        tile: Pos,
        sink: &mut S,
    ) -> AdventureOutcome {
        let on_tile = self
            .hero(hero)
            .is_some_and(|h| h.owner == player && h.pos == tile);
        match self.firing_event(player, tile).filter(|_| on_tile) {
            Some(index) => self.fire_event(rules, (player, hero), (index, tile), sink),
            None => AdventureOutcome::default(),
        }
    }

    /// Fires the event at `index` for `player`'s hero `hero`, which a step brought onto its tile
    /// `tile` (ADV-OBJ-007; the original's `event_fire` 0x4A0ED0): told to the player alone
    /// ([`AdventureEventKind::EventFired`], ADV-FOG-007); while its guards stand (the map's, or
    /// the survivors of a lost battle) the battle against them at once, announced and not asked,
    /// as Pandora's Box's ([`Self::guards_battle`]: unsplit, unowned, at its tile); else its
    /// reward record ([`Self::give_event`]).
    // rule: ADV-OBJ-007, ADV-FOG-007
    pub(super) fn fire_event<S: Sink>(
        &mut self,
        rules: Rules<'_>,
        (player, hero): (PlayerId, HeroId),
        (index, tile): (usize, Pos),
        sink: &mut S,
    ) -> AdventureOutcome {
        sink.emit(AdventureEvent {
            kind: AdventureEventKind::EventFired { hero, tile },
            seen_by: Viewers::NONE.with(player),
        });
        if self.guards_left(index).is_empty() {
            self.give_event(rules, hero, index, sink)
        } else {
            self.guards_battle(rules.1, hero, index)
        }
    }

    /// The event at `index` gives its reward record to `hero` ([`Self::give_record`],
    /// ADV-RWD-001) and, with its removal flag, leaves the map, told to no player: none saw it
    /// (ADV-FOG-007). Without the flag it stays and fires again at every later entry; guards a
    /// battle beat do not come back.
    // rule: ADV-OBJ-007, ADV-RWD-001
    pub(super) fn give_event<S: Sink>(
        &mut self,
        rules: Rules<'_>,
        hero: HeroId,
        index: usize,
        sink: &mut S,
    ) -> AdventureOutcome {
        let map = Arc::clone(&self.map);
        let event = map.objects()[index].event();
        let outcome = event
            .map(|event| self.give_record(rules, hero, &event.record, sink))
            .unwrap_or_default();
        if event.is_some_and(|event| event.remove) {
            self.remove_object(index);
        }
        outcome
    }

    /// The end of the battle `hero` fought at `tile` against the guards of the event lying there
    /// ([`Self::event_at`]; ADV-OBJ-007, the original's `event_fire` after its battle): lost, the
    /// guards' `survivors`, by slot, stay as its guards (none after a draw); won, its guards are
    /// gone for good and its reward record follows after the winner's levels
    /// ([`AdventureOutcome::reward_after`], GAM-DEC-007, E08 T63b: [`Self::give_won`]). Nothing
    /// when no event with guards lies at `tile`.
    // rule: ADV-OBJ-007, GAM-DEC-007
    pub(super) fn end_event_battle(
        &mut self,
        (hero, tile): (HeroId, Pos),
        (survivors, won): (&[ArmyStack], bool),
    ) -> AdventureOutcome {
        let Some(index) = self
            .event_at(tile)
            .filter(|&index| self.map.objects()[index].guards.is_some())
        else {
            return AdventureOutcome::default();
        };
        let left = if won { Vec::new() } else { survivors.to_vec() };
        self.army_left.set(index, Some(left));
        AdventureOutcome {
            reward_after: won.then_some((hero, tile)),
            ..AdventureOutcome::default()
        }
    }
}
