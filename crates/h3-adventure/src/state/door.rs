//! Dimension Door (E08 T143b; increment 15's design note §10.5, §11.2; ADV-SPL-007 to
//! ADV-SPL-009): the adventure's tests and effect of the door, read in the original's
//! `adventure_spell_cast` 0x41D090 with its target dialog `door_target_create` 0x4916F0 and
//! `door_target_proc` 0x491900 (survey `E08-spells.md` §8). The cast spends the hero's movement
//! points (`[params.spells]`' `door_points` at the mastery, 300 or 200 at Expert, at least 0) and
//! one of its casts of the day (the core spell's effect column, 1 to 4), whether the hero arrives
//! or the target fails; the arrival is a teleporter's (increment 11's `hero_relocate`
//! 0x41D930: no guard, no visit, the reveal by the scouting radius), a hero in a boat taking it
//! along. Nothing is drawn.

use alloc::sync::Arc;

use h3_content::{DOOR_FAILURE_CHARGED, DOOR_READS_HIDDEN, Ruleset};
use h3_contracts::{HeroId, PlayerId};
use h3_heroes::HeroDetails;

use super::spells::{Cast, CastError, CastFailure};
use super::{AdventureState, HeroOnMap};
use crate::command::Sink;
use crate::fog::reveal;
use crate::ruleset::{AdventureRuleset, Passage};
use crate::setup::Pos;

/// The casts of the day the door allows at `cast`'s mastery (ADV-SPL-007; the original's spell
/// table +0x34, 1 to 4 for None to Expert): none for a spell without the column.
fn casts(core: &Ruleset, cast: Cast) -> u32 {
    core.spell(cast.spell)
        .and_then(|spell| spell.effect.get(usize::from(cast.mastery)).copied())
        .map_or(0, u32::from)
}

impl AdventureState {
    /// Dimension Door's tests for `player`'s hero `on`, `doors` its casts of the day (ADV-SPL-007,
    /// ADV-SPL-008; increment 15's design note §10.5, §11.2), in the original's order: refused with
    /// no movement points left ([`CastError::NoPoints`], GENRLTXT 126), with the day's casts at
    /// the mastery's count ([`CastError::NoCastsLeft`], 339), for a target outside the window
    /// ([`Self::in_window`]); then a target `player` has explored, or with `ADV-QUIRK-51` on any
    /// target, that is not free ([`Self::door_free`]: [`CastError::NotFree`], the dialog's
    /// missing cursor), and with `ADV-QUIRK-55` off an explored target of the wrong kind
    /// ([`CastError::WrongKind`]). With `ADV-QUIRK-51` off (strict-fixed) every hidden tile of the
    /// window is a target, so the refusals depend only on what the player knows; on (faithful) a
    /// hidden tile is one when it is free, as the original's cursor tells (A-53).
    pub(super) fn check_door(
        &self,
        (core, part): (&Ruleset, &AdventureRuleset),
        (player, on): (PlayerId, &HeroOnMap),
        doors: u8,
        cast: Cast,
    ) -> Result<(), CastError> {
        if on.movement == 0 {
            return Err(CastError::NoPoints);
        }
        if u32::from(doors) >= casts(core, cast) {
            return Err(CastError::NoCastsLeft);
        }
        let target = self.in_window(part, on.pos, cast.target)?;
        let hidden = !self.explored[usize::from(player.raw())].get(target);
        if hidden && !core.quirks().is_enabled(DOOR_READS_HIDDEN) {
            return Ok(());
        }
        if !self.door_free(part, target) {
            return Err(CastError::NotFree);
        }
        let charged = hidden || core.quirks().is_enabled(DOOR_FAILURE_CHARGED);
        if !charged && !self.door_fits(part, on.boat.is_some(), target) {
            return Err(CastError::WrongKind);
        }
        Ok(())
    }

    /// Dimension Door's effect for `player`'s hero `hero` (ADV-SPL-009; increment 15's design note
    /// §10.5, §11.2; the original's `adventure_spell_cast` after its dialog): under `ADV-QUIRK-51`
    /// off a target `player` has not explored is revealed to it first, that tile alone (decision
    /// 399); then a free target of the hero's kind (water for a hero in a boat, land for one on
    /// land) takes the hero, **the arrival** of a teleporter ([`Self::arrival`]: no guard's
    /// battle, no visit, the reveal by its scouting radius, its boat with it at sea, told as
    /// [`crate::AdventureEventKind::HeroTeleported`]); any other fails. In both cases the hero's
    /// movement points go down by `[params.spells]`' `door_points` at the mastery, at least to 0,
    /// and its casts of the day by one (the original spends them after GENRLTXT 71 too, A-52).
    ///
    /// # Errors
    ///
    /// [`CastFailure::Target`] for a target that took no hero (GENRLTXT 71): charged.
    // rule: ADV-SPL-009
    pub(super) fn dimension_door<S: Sink>(
        &mut self,
        (core, part, heroes): (&Ruleset, &AdventureRuleset, &[Arc<HeroDetails>]),
        (player, hero): (PlayerId, HeroId),
        cast: Cast,
        sink: &mut S,
    ) -> Result<(), CastFailure> {
        // `check_cast` found the hero on the map and its target in the window.
        let index = self.heroes.partition_point(|h| h.id < hero);
        let (from, afloat) = (self.heroes[index].pos, self.heroes[index].boat.is_some());
        let target = cast.target.unwrap_or(from);
        let explored = &mut self.explored[usize::from(player.raw())];
        if !explored.get(target) && !core.quirks().is_enabled(DOOR_READS_HIDDEN) {
            reveal(explored, target, 0);
        }
        let arrives = self.door_free(part, target) && self.door_fits(part, afloat, target);
        if arrives {
            self.arrival((core, part, heroes), (player, hero), (from, target), sink);
        }
        let points = part.params().spells().map_or(0, |params| {
            params.door_points[usize::from(cast.mastery.min(3))]
        });
        let on = &mut self.heroes[index];
        on.movement = on.movement.saturating_sub(points);
        let mut spells = self.day_spells(hero);
        spells.doors = spells.doors.saturating_add(1);
        self.set_day_spells(hero, spells);
        if arrives {
            Ok(())
        } else {
            Err(CastFailure::Target)
        }
    }

    /// Whether the door may put a hero on `pos` as the map stands (ADV-SPL-008; the original's
    /// map item's word +0xC with neither 0x100, blocked, nor 0x1000, an entrance): no object's
    /// blocked tile and no rock, no object's visitable tile, no hero (in a boat or not) and no
    /// empty boat there.
    fn door_free(&self, part: &AdventureRuleset, pos: Pos) -> bool {
        self.map.tile(pos).is_some_and(|&tile| {
            let tile = self.live(pos, tile);
            let ground = matches!(
                part.passage(tile.terrain),
                Some(Passage::Land(_) | Passage::Water(_))
            );
            ground
                && !tile.blocked
                && !tile.visitable
                && !self.is_empty_boat(pos)
                && !self.heroes.iter().any(|hero| hero.pos == pos)
        })
    }

    /// Whether `pos` is of the kind a hero in a boat (`afloat`) or on land needs (ADV-SPL-009; the
    /// original's terrain 8 test): water for one, land for the other.
    fn door_fits(&self, part: &AdventureRuleset, afloat: bool, pos: Pos) -> bool {
        let water = self
            .map
            .tile(pos)
            .is_some_and(|tile| matches!(part.passage(tile.terrain), Some(Passage::Water(_))));
        water == afloat
    }
}
