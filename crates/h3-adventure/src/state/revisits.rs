//! The revisit (E08 T105b2; ADV-TEL-008, decision 268; increment 11's design note §13, survey
//! `E08-teleporters.md` §11): the original's space key, which visits the object under the
//! selected hero again (`visit_and_update` alone: no points tested or spent, no step, no guard
//! test; `after_step_guard_check` 0x4800A5 runs only at a step's end). Its gate is the original's:
//! any entered object, of any kind whose visit is built (the original's every entered class but
//! the Anchor Point, which the engine does not type).

use alloc::sync::Arc;

use h3_content::Ruleset;
use h3_contracts::{HeroId, PlayerId};
use h3_heroes::HeroDetails;
use h3_kernel::Rng;

use super::{AdventureState, StepVisit};
use crate::command::{AdventureCommandError, AdventureOutcome, Sink};
use crate::ruleset::{AdventureRuleset, Visit};
use crate::setup::Pos;

type Rules<'a> = (&'a Ruleset, &'a AdventureRuleset, &'a [Arc<HeroDetails>]);

impl AdventureState {
    /// `Revisit`: `player`'s hero `id` visits the entered object it stands on again
    /// ([`Self::revisit_at`]), as a step's last step onto it visits it ([`Self::visit`]), with its
    /// decisions, events and outcome; the hero's points and day's maximum stay as they are, no
    /// step event is emitted, nothing is revealed but what the visit reveals, and a monster
    /// guarding the tile does not attack. `draws` is the player's stream, as for [`Self::apply`]:
    /// a monolith's or a Whirlpool's exit is drawn again (a Whirlpool's loss too), after the
    /// checks.
    ///
    /// # Errors
    ///
    /// [`AdventureCommandError::NotYourHero`] for a hero not on the map or another player's,
    /// then [`AdventureCommandError::NothingToRevisit`]; the state and `draws` are unchanged.
    // rule: ADV-TEL-008
    pub(super) fn revisit<S: Sink>(
        &mut self,
        rules: Rules<'_>,
        (player, id): (PlayerId, HeroId),
        draws: &mut Rng,
        sink: &mut S,
    ) -> Result<AdventureOutcome, AdventureCommandError> {
        let at = self
            .hero(id)
            .filter(|hero| hero.owner == player)
            .ok_or(AdventureCommandError::NotYourHero(id))?
            .pos;
        let visit = self
            .revisit_at(rules.1, player, at)
            .ok_or(AdventureCommandError::NothingToRevisit)?;
        Ok(self.visit(rules, (player, id), visit, draws, sink))
    }

    /// What a revisit of `player`'s hero standing on `at` visits: the object a step onto `at`
    /// meets as it stands now ([`Self::visit_at`]) when the step enters it and visits it, none
    /// for a tile no object is visited from, a neighbour-visited object's (a hero never stands
    /// there; a town defended against `player` is visited from the tile before, TWN-VIS-001), a
    /// border gate the player passes (ADV-VIS-005) or an object whose visit is not built.
    // rule: ADV-TEL-008
    fn revisit_at<'p>(
        &self,
        part: &'p AdventureRuleset,
        player: PlayerId,
        at: Pos,
    ) -> Option<StepVisit<'p>> {
        if !self.live(at, self.map.at(at)).visitable {
            return None;
        }
        self.visit_at(part, player, at)
            .filter(|visit| visit.mode == Visit::Enter && !visit.pass)
    }

    /// `player`'s heroes whose `Revisit` [`Self::apply`] accepts now, in the order of their ids
    /// (ADV-TEL-008; E08 T105b2: `h3_game::legal_commands`' `Revisit`s): each on the visitable
    /// tile of an entered object whose visit is built ([`Self::revisit_at`]).
    pub fn revisitable<'a>(
        &'a self,
        part: &'a AdventureRuleset,
        player: PlayerId,
    ) -> impl Iterator<Item = HeroId> + 'a {
        self.heroes
            .iter()
            .filter(move |hero| {
                hero.owner == player && self.revisit_at(part, player, hero.pos).is_some()
            })
            .map(|hero| hero.id)
    }
}
