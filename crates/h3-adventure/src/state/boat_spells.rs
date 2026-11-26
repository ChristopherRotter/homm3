//! Summon Boat and Scuttle Boat (E08 T143a; increment 15's design note §10.1, §10.2, §11.3;
//! ADV-SPL-003, ADV-SPL-004): the adventure's tests and effects of the two boat spells, read in
//! the original's `summon_boat_cast` 0x41C8A0 with `summon_boat_choice` 0x4E54A0 and
//! `scuttle_boat_cast` 0x41CDF0 (survey `E08-spells.md` §5, §6). Each accepted cast draws its
//! chance first, one of 1 to 100 from the caster's player's stream (ADR-0017 point 4) against the
//! core spell's effect column at the mastery; a draw above it is a charged failure
//! ([`CastFailure::Chance`]). A boat keeps its record by index (ADR-0016 point 1): a summoned boat
//! moves, a made one takes the next index, a sunk one stays out of use (`gone`).

use core::cmp::Reverse;

use h3_content::{FRINGE_SHOWS_OBJECTS, Ruleset, SCUTTLE_READS_HIDDEN, SUMMON_BOAT_BY_STEPS};
use h3_contracts::{HeroId, PlayerId};
use h3_kernel::Rng;

use super::spells::{Cast, CastError, CastFailure};
use super::{AdventureState, Boat};
use crate::command::{AdventureEvent, AdventureEventKind, Sink, Viewers};
use crate::fog::{on_fringe, reveal};
use crate::movement::DIRECTIONS;
use crate::ruleset::{AdventureRuleset, Passage};
use crate::setup::Pos;

/// The lowest mastery at which Summon Boat makes a boat when it finds none to bring (ADV-SPL-003;
/// the original's `create_boat` at Advanced or Expert, survey `E08-spells.md` §5).
const MAKES_FROM: u8 = 2;

/// The type of a boat Summon Boat makes (the original's `create_boat` with type 0).
const MADE_TYPE: u8 = 0;

/// Whether `cast`'s chance draw, one of 1 to 100 from `draws`, fails: above the core spell's
/// effect column at the mastery (ADV-SPL-003, ADV-SPL-004; the original's `random_between(1, 100)`
/// against the spell table's +0x34, drawn at 100 too). A spell without the column fails.
fn chance_fails(core: &Ruleset, cast: Cast, draws: &mut Rng) -> bool {
    let chance = core
        .spell(cast.spell)
        .and_then(|spell| spell.effect.get(usize::from(cast.mastery)).copied())
        .unwrap_or(0);
    draws.range_inclusive(1, 100) > u32::from(chance)
}

/// The steps between two tiles, levels ignored: |dx| + |dy| (`ADV-QUIRK-53` on, the original's
/// `summon_boat_choice`).
fn steps(a: Pos, b: Pos) -> u32 {
    u32::from(a.x.abs_diff(b.x)) + u32::from(a.y.abs_diff(b.y))
}

/// The squared straight distance between two tiles: dx² + dy² (`ADV-QUIRK-53` off).
fn squared(a: Pos, b: Pos) -> u32 {
    let (dx, dy) = (u32::from(a.x.abs_diff(b.x)), u32::from(a.y.abs_diff(b.y)));
    dx * dx + dy * dy
}

impl AdventureState {
    /// The water tile beside `pos` on which Summon Boat puts its boat (ADV-SPL-003; the original's
    /// first of the eight neighbours clockwise from up, `step_directions` 0x678150, on the map,
    /// water and free of objects): its terrain water, no object's blocked or visitable tile there
    /// as the map stands (`live`), no empty boat and no hero (in a boat) on it. None: the cast is
    /// refused (GENRLTXT 335). *Agent's call, change freely* (E08 T143a): the original's "no
    /// object" (the tile's class 0) read as neither an object's blocked nor its visitable tile,
    /// as a boat of the map is placed (ADV-SEA-001).
    pub(super) fn summon_tile(&self, part: &AdventureRuleset, pos: Pos) -> Option<Pos> {
        DIRECTIONS.iter().find_map(|&(dx, dy)| {
            let x = pos.x.checked_add_signed(i16::from(dx))?;
            let y = pos.y.checked_add_signed(i16::from(dy))?;
            let at = Pos::new(x, y, pos.level);
            let tile = self.live(at, *self.map.tile(at)?);
            let free = matches!(part.passage(tile.terrain), Some(Passage::Water(_)))
                && !tile.blocked
                && !tile.visitable
                && !self.is_empty_boat(at)
                && !self.heroes.iter().any(|hero| hero.pos == at);
            free.then_some(at)
        })
    }

    /// Scuttle Boat's tests of `target` for `player`'s hero on `from` (ADV-SPL-004; increment 15's
    /// design note §10.2, §11.3): in the window ([`Self::in_window`]: else
    /// [`CastError::OutOfWindow`]), then
    /// an empty boat on it, or under `ADV-QUIRK-52` off a tile `player` has not explored
    /// (else [`CastError::NoBoat`]). With the flag on (faithful) a hidden empty boat is a target
    /// and a hidden tile without one is not, as the original's cursor tells (A-50); off
    /// (strict-fixed) the refusals depend only on what the player knows.
    pub(super) fn check_scuttle(
        &self,
        core: &Ruleset,
        part: &AdventureRuleset,
        (player, from): (PlayerId, Pos),
        target: Option<Pos>,
    ) -> Result<(), CastError> {
        let target = self.in_window(part, from, target)?;
        self.scuttle_target(core, player, target)
    }

    /// [`Self::check_scuttle`]'s test of a target in the window (E08 T144a1: shared with
    /// [`Self::cast_targets`]): an empty boat, or under `ADV-QUIRK-52` off a tile `player` has
    /// not explored.
    pub(super) fn scuttle_target(
        &self,
        core: &Ruleset,
        player: PlayerId,
        target: Pos,
    ) -> Result<(), CastError> {
        let hidden = !self.explored[usize::from(player.raw())].get(target);
        if self.is_empty_boat(target) || (hidden && !core.quirks().is_enabled(SCUTTLE_READS_HIDDEN))
        {
            Ok(())
        } else {
            Err(CastError::NoBoat)
        }
    }

    /// Summon Boat's effect for `player`'s hero `hero` (ADV-SPL-003; increment 15's design note
    /// §10.1; the original's `summon_boat_cast` after its tests): the chance draw, then the boat
    /// [`Self::summoned_boat`] finds is moved onto the hero's water tile ([`Self::summon_tile`]),
    /// its facing and owner kept (boarding writes the owner), told as
    /// [`AdventureEventKind::BoatSummoned`]; none found, at Advanced or Expert a boat of type 0 of
    /// `player` is made there ([`Self::launch`], facing 2) while fewer than `[params.spells]`'
    /// `boats_most` boats are in use (not gone), told as [`AdventureEventKind::BoatCreated`].
    ///
    /// # Errors
    ///
    /// [`CastFailure::Chance`] for the draw (GENRLTXT 337), [`CastFailure::NoBoat`] when no boat
    /// came (GENRLTXT 336): both charged.
    // rule: ADV-SPL-003
    pub(super) fn summon_boat<S: Sink>(
        &mut self,
        (core, part): (&Ruleset, &AdventureRuleset),
        (player, hero): (PlayerId, HeroId),
        cast: Cast,
        draws: &mut Rng,
        sink: &mut S,
    ) -> Result<(), CastFailure> {
        // `check_cast` found the hero on the map and its water tile.
        let pos = self.hero(hero).map_or(Pos::new(0, 0, 0), |on| on.pos);
        let to = self.summon_tile(part, pos).unwrap_or(pos);
        if chance_fails(core, cast, draws) {
            return Err(CastFailure::Chance);
        }
        if let Some((index, boat)) = self.summoned_boat(core, (player, hero), pos) {
            let seen_from = self.tile_viewers(core, boat.tile);
            let seen_to = self.viewers(core, player, to);
            let kind = self
                .boats
                .as_mut()
                .and_then(|boats| boats.shift(index, to))
                .unwrap_or(boat.kind);
            sink.emit(AdventureEvent {
                kind: AdventureEventKind::BoatSummoned {
                    boat: kind,
                    from: Some(boat.tile),
                    to: Some(to),
                    facing: boat.facing,
                    seen_from,
                    seen_to,
                },
                seen_by: seen_from.union(seen_to),
            });
            return Ok(());
        }
        let in_use = self.boats().filter(|(_, boat)| !boat.gone).count();
        let most = part
            .params()
            .spells()
            .map_or(0, |params| usize::from(params.boats_most.get()));
        if cast.mastery < MAKES_FROM || in_use >= most {
            return Err(CastFailure::NoBoat);
        }
        self.launch(to, MADE_TYPE, player);
        sink.emit(AdventureEvent {
            kind: AdventureEventKind::BoatCreated {
                tile: to,
                boat_type: MADE_TYPE,
            },
            seen_by: self.viewers(core, player, to),
        });
        Ok(())
    }

    /// The boat Summon Boat brings to `player`'s hero `hero` on `pos`, by index, with its record
    /// (ADV-SPL-003; the original's `summon_boat_choice` 0x4E54A0, survey `E08-spells.md` §5;
    /// batch E15's E15-9a, E15-9b): the first empty boat whose last captain is the hero, whatever
    /// its owner and level (the hero's last boat); else among the empty boats of `player` or of
    /// nobody, with `ADV-QUIRK-53` on (faithful) the least |dx| + |dy|, levels ignored, a tie to
    /// the later index (the original's `jg`); off the least dx² + dy² on the hero's level, a tie
    /// to the lower index (decision 400). None: no such boat.
    fn summoned_boat(
        &self,
        core: &Ruleset,
        (player, hero): (PlayerId, HeroId),
        pos: Pos,
    ) -> Option<(usize, Boat)> {
        let empty = || {
            self.boats()
                .filter(|(_, boat)| !boat.aboard && !boat.gone)
                .map(|(index, &boat)| (index, boat))
        };
        if let Some(last) = empty().find(|(_, boat)| boat.captain == Some(hero)) {
            return Some(last);
        }
        let ours = empty().filter(|(_, boat)| boat.owner.is_none_or(|owner| owner == player));
        if core.quirks().is_enabled(SUMMON_BOAT_BY_STEPS) {
            ours.min_by_key(|&(index, boat)| (steps(boat.tile, pos), Reverse(index)))
        } else {
            ours.filter(|(_, boat)| boat.tile.level == pos.level)
                .min_by_key(|&(index, boat)| (squared(boat.tile, pos), index))
        }
    }

    /// Scuttle Boat's effect for `player` (ADV-SPL-004; increment 15's design note §10.2, §11.3;
    /// the original's `scuttle_boat_cast` after its dialog): under `ADV-QUIRK-52` off a target
    /// `player` has not explored is revealed to it first, that tile alone, in every outcome
    /// (decision 402); a target without an empty boat then fails; else the chance draw, and the
    /// boat is sunk ([`AdventureEventKind::BoatScuttled`], told to those that saw heroes on the
    /// tile before the reveal). With the flag on nothing is revealed (batch E15's E15-10: the
    /// original reveals nothing at a sunk hidden boat).
    ///
    /// # Errors
    ///
    /// [`CastFailure::Target`] for a revealed target without an empty boat,
    /// [`CastFailure::Chance`] for the draw (GENRLTXT 338): both charged.
    // rule: ADV-SPL-004
    pub(super) fn scuttle_boat<S: Sink>(
        &mut self,
        (core, _): (&Ruleset, &AdventureRuleset),
        player: PlayerId,
        cast: Cast,
        draws: &mut Rng,
        sink: &mut S,
    ) -> Result<(), CastFailure> {
        // `check_cast` refused a cast without a target in the window.
        let target = cast.target.unwrap_or(Pos::new(0, 0, 0));
        // Those that saw the boat there, before the reveal: the caster of a hidden target sees
        // the tile, empty, in the command's reveal (`AdventureState::reveal`).
        let seen_by = self.tile_viewers(core, target);
        if !core.quirks().is_enabled(SCUTTLE_READS_HIDDEN) {
            reveal(&mut self.explored[usize::from(player.raw())], target, 0);
        }
        let Some(index) = self.empty_boat(target) else {
            return Err(CastFailure::Target);
        };
        if chance_fails(core, cast, draws) {
            return Err(CastFailure::Chance);
        }
        let index = usize::try_from(index).unwrap_or(usize::MAX);
        let kind = self
            .boats
            .as_mut()
            .and_then(|boats| boats.sink(index))
            .unwrap_or(0);
        sink.emit(AdventureEvent {
            kind: AdventureEventKind::BoatScuttled {
                tile: target,
                boat: kind,
            },
            seen_by,
        });
        Ok(())
    }

    /// The players that see heroes, and so empty boats, on `pos` (ADV-FOG-008): those that have
    /// explored it and, under `ADV-QUIRK-1`, those with it on their fringe; no owner among them
    /// unless it sees the tile ([`Self::viewers`] names one).
    fn tile_viewers(&self, core: &Ruleset, pos: Pos) -> Viewers {
        let fringe = core.quirks().is_enabled(FRINGE_SHOWS_OBJECTS);
        self.explored
            .iter()
            .zip((0..self.players).map(PlayerId::new))
            .filter(|(grid, _)| grid.get(pos) || (fringe && on_fringe(grid, pos)))
            .fold(Viewers::NONE, |viewers, (_, player)| viewers.with(player))
    }
}
