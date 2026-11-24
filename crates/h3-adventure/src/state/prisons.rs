//! The Prison's freeing (ADV-PRI-002; E08 T141a, increment 15's design note §9): the adventure's
//! half of a visit whose outcome named a prisoner ([`crate::AdventureOutcome::prison`]). The game
//! decides (the town part's most heroes on the map, the store's pool) and calls one of the two,
//! each infallible on the visit the outcome named.

use alloc::sync::Arc;

use h3_content::Ruleset;
use h3_contracts::PlayerId;
use h3_heroes::HeroDetails;

use super::{AdventureState, HeroOnMap, next_listed};
use crate::command::{PrisonVisit, Sink, Viewers};
use crate::movement::{day_points, details_of};
use crate::ruleset::AdventureRuleset;

impl AdventureState {
    /// Frees the prisoner of `visit` for `owner` (ADV-PRI-002; the original's `visit_prison`
    /// 0x4A3EB0, survey `E08-spells.md` §2.2 point 3; batch E15's E15-1): the Prison taken off the
    /// map (`ObjectRemoved` to the players that see it), then the hero put on its visitable tile,
    /// on land, with the army and formation the build gave it ([`crate::Resolved::Prisoner`]), its
    /// tactics flag on, no boat, last in `owner`'s list ([`HeroOnMap::listed`]), and as its points
    /// and the day's maximum a full day's land points for that army over its details in
    /// `heroes`, the game's store ([`day_points`] without week marks: it has none, as a tavern
    /// hero's, TWN-TAV-005). Nothing is revealed or visited, and the visitor is untouched.
    /// Returns the players that see the tile, `owner` among them (as a hire's,
    /// [`Self::place_hired`]), whom the game tells the freeing. A visit that names no Prison with
    /// that prisoner changes nothing and returns none (the game never gives one).
    // rule: ADV-PRI-002
    pub fn free_prisoner<S: Sink>(
        &mut self,
        (core, part, heroes): (&Ruleset, &AdventureRuleset, &[Arc<HeroDetails>]),
        visit: &PrisonVisit,
        owner: PlayerId,
        sink: &mut S,
    ) -> Viewers {
        let Some((army, formation)) = self.held_prisoner(visit) else {
            return Viewers::NONE;
        };
        self.empty_prison(core, visit, sink);
        let points = day_points(core, part, details_of(heroes, visit.prisoner), &army, 0);
        let hero = HeroOnMap {
            id: visit.prisoner,
            owner,
            pos: visit.entrance,
            army,
            formation,
            tactics: true,
            movement: points,
            boat: None,
            max_movement: points,
            listed: next_listed(&self.heroes, owner),
        };
        let seen_by = self.viewers(core, owner, visit.entrance);
        let at = self.heroes.partition_point(|on| on.id < hero.id);
        self.heroes.insert(at, hero);
        seen_by
    }

    /// Takes `visit`'s Prison off the map (ADV-PRI-002; survey `E08-spells.md` §2.2 point 1: a
    /// prisoner that is no prisoner any more, unreachable without campaigns; and the first step of
    /// [`Self::free_prisoner`]): `ObjectRemoved` to the players that see it, by its type and
    /// anchor. Nothing for a visit that names no Prison on the map.
    // rule: ADV-PRI-002
    pub fn empty_prison<S: Sink>(&mut self, core: &Ruleset, visit: &PrisonVisit, sink: &mut S) {
        if self.is_removed(visit.index) {
            return;
        }
        if let Some(object_type) = self.object(visit.index).and_then(|o| o.object_type) {
            self.take_object(core, (visit.index, object_type), sink);
        }
    }

    /// The army and formation `visit`'s Prison holds for its prisoner, if it still stands on the
    /// map and holds that prisoner.
    fn held_prisoner(
        &self,
        visit: &PrisonVisit,
    ) -> Option<(
        alloc::vec::Vec<h3_contracts::ArmyStack>,
        h3_content::Formation,
    )> {
        let held = (!self.is_removed(visit.index))
            .then(|| self.object(visit.index))
            .flatten()
            .and_then(|object| object.resolved.prisoner());
        held.filter(|&(hero, ..)| hero == visit.prisoner)
            .map(|(_, army, formation)| (army.to_vec(), formation))
    }
}
