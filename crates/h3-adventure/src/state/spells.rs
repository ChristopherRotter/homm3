//! The heroes' day spells and the adventure's half of a cast on the map (E08 T142a; increment
//! 15's design note §8, §10.3, §10.4, §12; ADV-SPL-002, ADV-SPL-005, ADV-SPL-006): what a cast of
//! Fly or Water Walk sets, what the new day and a boarding clear, and the cast's own tests. The
//! game checks the rest first (GAM-SPL-001: the spell known, the part's binding, the cost, the
//! ground) and spends the spell points; the adventure never reads or writes them (ADR-0017 point
//! 10). Summon Boat and Scuttle Boat since E08 T143a (`boat_spells`, ADV-SPL-003, ADV-SPL-004:
//! the chance drawn from the caster's player's stream, a failure charged, [`CastFailure`]);
//! Dimension Door since E08 T143b (`door`, ADV-SPL-007 to ADV-SPL-009: its movement points and
//! its casts of the day, the relocation, a wrong target charged).

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt;

use h3_content::{Ruleset, SPELL_RECAST_REFUSED, SpellId};
use h3_contracts::{HeroId, PlayerId};
use h3_heroes::HeroDetails;
use h3_kernel::{Rng, StateHasher};

use super::AdventureState;
use crate::command::Sink;
use crate::movement::{details_of, hero_movement};
use crate::ruleset::{AdventureRuleset, AdventureSpell};
use crate::setup::Pos;

/// A hero's spells of the day (ADV-SPL-002; increment 15's design note §8): the levels its Fly and
/// Water Walk set, cast at a mastery from 0 (none) to 3 (Expert), and the Dimension Door casts it
/// made today. The original's hero record's +0x112, +0x116 (−1 when off) and +0x10D. Its owner's
/// alone (the own hero view, ADR-0016 point 6); every hero has none at the start of a day. Kept
/// by hero id in the state, not on [`crate::HeroOnMap`] (*agent's call*, E08 T142a: a field there
/// costs every fork, the lane's trap; a hero keeps its spells into a town's garrison the same
/// day, as the original's record does).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DaySpells {
    /// The flying level Fly set (ADV-SPL-005), none when off.
    pub fly: Option<u8>,
    /// The water-walking level Water Walk set (ADV-SPL-006), none when off.
    pub water_walk: Option<u8>,
    /// The Dimension Door casts of the day (ADV-SPL-007; from E08 T143b).
    pub doors: u8,
}

impl DaySpells {
    /// None of them: a new day's, or a hero's that never cast.
    pub const NONE: Self = Self {
        fly: None,
        water_walk: None,
        doors: 0,
    };

    /// Hashes the spells (the adventure state hash v29): each level (0 for none, else 1 and the
    /// level), then the door's casts.
    pub(crate) fn hash_into(self, h: &mut StateHasher) {
        for level in [self.fly, self.water_walk] {
            match level {
                None => h.write_u8(0),
                Some(level) => {
                    h.write_u8(1);
                    h.write_u8(level);
                }
            }
        }
        h.write_u8(self.doors);
    }
}

/// A cast on the map as the game hands it to the adventure (increment 15's design note §10, §14;
/// E08 T143a): the game has made its own checks (GAM-SPL-001) and found the spell's behaviour, the
/// mastery on the hero's ground and the target's shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cast {
    /// The core spell: its effect column by mastery is Summon Boat's and Scuttle Boat's chance
    /// (the original's spell table +0x34: 50, 50, 75, 100), read through `h3_content`.
    pub spell: SpellId,
    /// What it does on the map (the part's binding, [`AdventureRuleset::adventure_spell`]).
    pub effect: AdventureSpell,
    /// The mastery on the hero's ground, 0 (none) to 3 (Expert).
    pub mastery: u8,
    /// The target tile: Scuttle Boat's boat and Dimension Door's arrival; none for the other three
    /// (the game refuses a cast whose target does not fit its spell).
    pub target: Option<Pos>,
}

/// Why the adventure refuses a cast on the map (increment 15's design note §10; ADV-SPL-003 to
/// ADV-SPL-006). A refusal changes nothing, draws nothing and spends nothing (the game spends the
/// points only after the cast is accepted).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastError {
    /// The hero is not one of the player's heroes on the map.
    NotYourHero(HeroId),
    /// Fly in a boat (ADV-SPL-005; the original's GENRLTXT 193), or Summon Boat on water, in a
    /// boat (ADV-SPL-003; GENRLTXT 334): a hero on water is in a boat (a water walk never stops
    /// there).
    InBoat,
    /// Under `ADV-QUIRK-54` the spell is on already, or the hero's artifacts give the level (Angel
    /// Wings for Fly, Boots of Levitation for Water Walk on land): the original's silent nothing,
    /// free (survey `E08-spells.md` §7.1; batch E15's E15-11).
    AlreadyOn,
    /// With `ADV-QUIRK-54` off the cast's level would not exceed the hero's level now (the
    /// artifacts' 3 included).
    NotHigher,
    /// Summon Boat without a free water tile among the eight around the hero (ADV-SPL-003; the
    /// original's GENRLTXT 335).
    NoWater,
    /// A target outside the window around the hero (`[params.spells]`' `target_range`: at most 9
    /// columns and 8 rows away, on its level), or off the map (ADV-SPL-004; the original's dialog
    /// offers no other tile).
    OutOfWindow,
    /// Scuttle Boat's target holds no empty boat it sinks (ADV-SPL-004; the original's dialog
    /// offers no cursor there, a click does nothing): an explored tile without one (a boat with a
    /// hero aboard is the hero's tile), and with `ADV-QUIRK-52` on a hidden tile without one too;
    /// with the flag off every hidden tile of the window is a target (increment 15's design note
    /// §11.3).
    NoBoat,
    /// Dimension Door with no movement points left (ADV-SPL-007; the original's GENRLTXT 126:
    /// the points not above 0).
    NoPoints,
    /// Dimension Door cast today as often as the mastery allows (ADV-SPL-007; GENRLTXT 339): the
    /// core spell's effect column at the mastery, 1 to 4 casts from None to Expert.
    NoCastsLeft,
    /// Dimension Door's target is not free (ADV-SPL-008; the original's dialog offers no cursor
    /// there): an object's blocked tile or rock, an object's entrance, a hero's tile or an empty
    /// boat's, on an explored tile, and with `ADV-QUIRK-51` on on a hidden one too; with the flag
    /// off every hidden tile of the window is a target (increment 15's design note §11.2).
    NotFree,
    /// With `ADV-QUIRK-55` off Dimension Door's explored target is of the wrong kind: land for a
    /// hero in a boat, water for one on land (ADV-SPL-009; increment 15's design note §10.5; on,
    /// the cast is accepted and fails, charged).
    WrongKind,
}

/// Why an accepted cast on the map failed (increment 15's design note §10, §14; E08 T143a): a
/// **charged failure**, its spell points spent as for an effect, told as the game's `CastFailed`
/// after its `SpellCast`. Nothing else changed but the target's reveal under `ADV-QUIRK-52` off.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastFailure {
    /// The chance draw, one of 1 to 100 from the caster's player's stream, came above the
    /// spell's chance at the mastery (ADV-SPL-003, ADV-SPL-004; the original's GENRLTXT 337 for
    /// Summon Boat, 338 for Scuttle Boat): drawn at a chance of 100 too.
    Chance,
    /// Summon Boat found no boat to bring and made none (ADV-SPL-003; GENRLTXT 336): at None or
    /// Basic, or with `[params.spells]`' `boats_most` boats in use.
    NoBoat,
    /// The target, once revealed, held nothing the spell acts on (ADV-SPL-004: Scuttle Boat's
    /// hidden target without an empty boat under `ADV-QUIRK-52` off, increment 15's design note
    /// §11.3), or Dimension Door's could not take the hero (ADV-SPL-009, §10.5, §11.2; the
    /// original's GENRLTXT 71): of the wrong kind, or under `ADV-QUIRK-51` off a hidden one not
    /// free; its movement points and its cast of the day spent as for an arrival.
    Target,
}

/// The heroes' day spells by hero id ([`AdventureState`]'s block): only heroes with some, in id
/// order.
pub(crate) type DaySpellBook = Vec<(HeroId, DaySpells)>;

/// `hero`'s day spells in `book` (none without a block or an entry).
pub(crate) fn spells_in(book: Option<&DaySpellBook>, hero: HeroId) -> DaySpells {
    book.and_then(|book| {
        book.binary_search_by_key(&hero, |&(id, _)| id)
            .ok()
            .map(|at| book[at].1)
    })
    .unwrap_or_default()
}

impl fmt::Display for CastError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotYourHero(hero) => {
                write!(f, "hero {} is not the player's on the map", hero.raw())
            }
            Self::InBoat => f.write_str("Fly is not cast in a boat"),
            Self::AlreadyOn => f.write_str("the spell's level is on already"),
            Self::NotHigher => f.write_str("the cast would not raise the hero's level"),
            Self::NoWater => f.write_str("no free water lies around the hero"),
            Self::OutOfWindow => f.write_str("the target lies outside the spell's window"),
            Self::NoBoat => f.write_str("no empty boat lies on the target"),
            Self::NoPoints => f.write_str("the hero has no movement points left"),
            Self::NoCastsLeft => {
                f.write_str("the hero cast the spell today as often as its mastery allows")
            }
            Self::NotFree => f.write_str("the target is blocked or an entrance"),
            Self::WrongKind => f.write_str("the target is land from a boat or water from land"),
        }
    }
}

impl core::error::Error for CastError {}

impl AdventureState {
    /// The day spells of the hero `hero` (ADV-SPL-002): none unless it cast today. Out of line:
    /// the reach's search asks once (E08 T142a: inlined there it moved the search's loop, +1.5 %
    /// on `reach-towns-36`).
    #[inline(never)]
    pub fn day_spells(&self, hero: HeroId) -> DaySpells {
        spells_in(self.day_spells.as_deref(), hero)
    }

    /// The day spells' block as it is (a pointer's copy: the block is shared, a write copies),
    /// for [`crate::OwnHeroes`]' comparison after a command.
    pub(crate) fn day_spell_book(&self) -> Option<Arc<DaySpellBook>> {
        self.day_spells.clone()
    }

    /// Sets `hero`'s day spells: the block keeps only heroes with some, so a block that would be
    /// empty is dropped (no block and an empty one are one state).
    pub(super) fn set_day_spells(&mut self, hero: HeroId, spells: DaySpells) {
        let book = Arc::make_mut(self.day_spells.get_or_insert_default());
        match (book.binary_search_by_key(&hero, |&(id, _)| id), spells) {
            (Ok(at), DaySpells::NONE) => {
                book.remove(at);
            }
            (Ok(at), _) => book[at].1 = spells,
            (Err(_), DaySpells::NONE) => {}
            (Err(at), _) => book.insert(at, (hero, spells)),
        }
        if book.is_empty() {
            self.day_spells = None;
        }
    }

    /// Hashes the day spells' block (the adventure state hash v29): the heroes with some, each its
    /// id and spells.
    pub(super) fn hash_day_spells(&self, h: &mut StateHasher) {
        let book = self.day_spells.as_deref().map_or(&[][..], |book| &book[..]);
        h.write_len(book.len());
        for &(hero, spells) in book {
            h.write_u16(hero.raw());
            spells.hash_into(h);
        }
    }

    /// The target of a cast by a hero on `from` when it lies in the window (ADV-SPL-004,
    /// ADV-SPL-008; `[params.spells]`' `target_range`: at most 9 columns and 8 rows away, the
    /// original's dialog's view less its extra row, `ADV-QUIRK-56` fixed), on the hero's level and
    /// on the map.
    ///
    /// # Errors
    ///
    /// [`CastError::OutOfWindow`] for none, or one outside.
    pub(super) fn in_window(
        &self,
        part: &AdventureRuleset,
        from: Pos,
        target: Option<Pos>,
    ) -> Result<Pos, CastError> {
        let [columns, rows] = part
            .params()
            .spells()
            .map_or([0, 0], |params| params.target_range);
        target
            .filter(|target| {
                target.level == from.level
                    && target.x.abs_diff(from.x) <= u16::from(columns)
                    && target.y.abs_diff(from.y) <= u16::from(rows)
                    && self.map.tile(*target).is_some()
            })
            .ok_or(CastError::OutOfWindow)
    }

    /// Checks `player`'s cast `cast` by its hero `hero` as [`Self::apply_cast`] makes it
    /// (increment 15's design note §10), its mastery the game's derivation on the hero's ground
    /// (`h3_heroes::spell_masteries_on`), its details in `heroes`, the game's store. Nothing
    /// changes and nothing is drawn.
    ///
    /// - **Summon Boat** (ADV-SPL-003): refused on water, in a boat, then without a free water
    ///   tile around the hero ([`Self::summon_tile`]).
    /// - **Scuttle Boat** (ADV-SPL-004): refused for a target outside the window or off the map,
    ///   then for one without an empty boat, under `ADV-QUIRK-52` (§11.3: on, a hidden tile too;
    ///   off, every hidden tile of the window a target).
    /// - **Fly** (ADV-SPL-005): refused in a boat; with `ADV-QUIRK-54` on while Fly is on or the
    ///   hero's artifacts give it a flying level (Angel Wings), off when the mastery would not
    ///   exceed its flying level now (the artifacts' and the spell's, the higher).
    /// - **Water Walk** (ADV-SPL-006): in a boat **accepted without either test** (A-49 accepted,
    ///   `ADV-QUIRK-57`: the original skips them there, and the level holds after landing the same
    ///   day; batch E15's E15-12); on land refused as Fly is, by its water-walking level (Boots of
    ///   Levitation).
    /// - **Dimension Door** (ADV-SPL-007, ADV-SPL-008): refused without movement points, with the
    ///   day's casts at the mastery's count, for a target outside the window, then for one not
    ///   free and (`ADV-QUIRK-55` off) one of the wrong kind ([`Self::check_door`]).
    ///
    /// # Errors
    ///
    /// The [`CastError`] of the first test the cast fails.
    // rule: ADV-SPL-003, ADV-SPL-004, ADV-SPL-005, ADV-SPL-006, ADV-SPL-007, ADV-SPL-008
    pub fn check_cast(
        &self,
        (core, part): (&Ruleset, &AdventureRuleset),
        heroes: &[Arc<HeroDetails>],
        (player, hero): (PlayerId, HeroId),
        cast: Cast,
    ) -> Result<(), CastError> {
        let index = self
            .heroes
            .binary_search_by_key(&hero, |h| h.id)
            .ok()
            .filter(|&index| self.heroes[index].owner == player)
            .ok_or(CastError::NotYourHero(hero))?;
        let on = &self.heroes[index];
        let afloat = on.boat.is_some();
        let (own, day) = (
            hero_movement(core, details_of(heroes, hero)),
            self.day_spells(hero),
        );
        let (level, worn) = match cast.effect {
            AdventureSpell::SummonBoat if afloat => return Err(CastError::InBoat),
            AdventureSpell::SummonBoat => {
                return self
                    .summon_tile(part, on.pos)
                    .map(|_| ())
                    .ok_or(CastError::NoWater);
            }
            AdventureSpell::ScuttleBoat => {
                return self.check_scuttle(core, part, (player, on.pos), cast.target);
            }
            AdventureSpell::Fly if afloat => return Err(CastError::InBoat),
            AdventureSpell::Fly => (day.fly, own.flying),
            AdventureSpell::WaterWalk if afloat => return Ok(()),
            AdventureSpell::WaterWalk => (day.water_walk, own.water_walking),
            AdventureSpell::DimensionDoor => {
                return self.check_door((core, part), (player, on), day.doors, cast);
            }
        };
        if core.quirks().is_enabled(SPELL_RECAST_REFUSED) {
            if level.is_some() || worn.is_some() {
                return Err(CastError::AlreadyOn);
            }
        } else if level.max(worn).is_some_and(|level| level >= cast.mastery) {
            return Err(CastError::NotHigher);
        }
        Ok(())
    }

    /// The cast `cast` by `player`'s hero `hero` that [`Self::check_cast`] accepted (ADV-SPL-003
    /// to ADV-SPL-006; increment 15's design note §10), its draws from `draws`, the caster's
    /// player's stream (ADR-0017 point 4), its events to `sink`:
    ///
    /// - **Summon Boat** and **Scuttle Boat** ([`Self::summon_boat`], [`Self::scuttle_boat`]):
    ///   the chance draw, then a boat brought, made or sunk, told as
    ///   [`crate::AdventureEventKind::BoatSummoned`], `BoatCreated` or `BoatScuttled`.
    /// - **Dimension Door** ([`Self::dimension_door`]): the hero relocated onto the target as a
    ///   teleporter's arrival (no guard, no visit, the reveal; told as
    ///   [`crate::AdventureEventKind::HeroTeleported`]), or a wrong target charged; its movement
    ///   points less `[params.spells]`' `door_points` at the mastery and its casts of the day + 1
    ///   in both cases.
    /// - **Fly** sets the hero's flying level for the day to the mastery, **Water Walk** its
    ///   water-walking level: every reader of the hero's levels takes the higher of its
    ///   artifacts' and its spell's from then on (the paths, the prices, the drop to 0:
    ///   [`crate::movement::day_movement`]); no event of their own (the changed spells are told
    ///   after the command, [`crate::AdventureEventKind::DaySpellsChanged`]).
    ///
    /// # Errors
    ///
    /// A [`CastFailure`] for a charged failure: the cast stays accepted (the game spends its
    /// points and tells `CastFailed`), nothing changed but a target's reveal and, for Dimension
    /// Door, its points and its casts of the day.
    // rule: ADV-SPL-003, ADV-SPL-004, ADV-SPL-005, ADV-SPL-006, ADV-SPL-009
    pub fn apply_cast<S: Sink>(
        &mut self,
        (core, part, heroes): (&Ruleset, &AdventureRuleset, &[Arc<HeroDetails>]),
        (player, hero): (PlayerId, HeroId),
        cast: Cast,
        draws: &mut Rng,
        sink: &mut S,
    ) -> Result<(), CastFailure> {
        let mut spells = self.day_spells(hero);
        match cast.effect {
            AdventureSpell::SummonBoat => {
                return self.summon_boat((core, part), (player, hero), cast, draws, sink);
            }
            AdventureSpell::ScuttleBoat => {
                return self.scuttle_boat((core, part), player, cast, draws, sink);
            }
            AdventureSpell::DimensionDoor => {
                return self.dimension_door((core, part, heroes), (player, hero), cast, sink);
            }
            AdventureSpell::Fly => spells.fly = Some(cast.mastery),
            AdventureSpell::WaterWalk => spells.water_walk = Some(cast.mastery),
        }
        self.set_day_spells(hero, spells);
        Ok(())
    }

    /// The new day clears every hero's day spells (ADV-SPL-002; increment 15's design note §12,
    /// the original's `new_day` 0x4C7FE0 over all its hero records, survey `E08-spells.md` §4.3):
    /// the block goes, whoever holds it (a hero on the map, in a garrison or off the map).
    // rule: ADV-SPL-002
    pub(super) fn clear_day_spells(&mut self) {
        self.day_spells = None;
    }

    /// A boarding clears the hero's Fly and Water Walk (ADV-SPL-002; the original's
    /// `hero_boards_boat` 0x4A1010 sets +0x112 and +0x116 to −1, survey `E08-spells.md` §4.3;
    /// batch E15's E15-11); a landing keeps Water Walk's level a cast aboard set (A-49).
    // rule: ADV-SPL-002
    pub(super) fn board_day_spells(&mut self, hero: HeroId) {
        let spells = self.day_spells(hero);
        if spells.fly.is_some() || spells.water_walk.is_some() {
            self.set_day_spells(
                hero,
                DaySpells {
                    fly: None,
                    water_walk: None,
                    ..spells
                },
            );
        }
    }
}
