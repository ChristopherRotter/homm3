//! The heroes' day spells and the adventure's half of a cast on the map (E08 T142a; increment
//! 15's design note §8, §10.3, §10.4, §12; ADV-SPL-002, ADV-SPL-005, ADV-SPL-006): what a cast of
//! Fly or Water Walk sets, what the new day and a boarding clear, and the cast's own tests. The
//! game checks the rest first (GAM-SPL-001: the spell known, the part's binding, the cost, the
//! ground) and spends the spell points; the adventure never reads or writes them (ADR-0017 point
//! 10). Summon Boat, Scuttle Boat and Dimension Door are E08 T143's (`ADV-STUB-22`).

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt;

use h3_content::{Ruleset, SPELL_RECAST_REFUSED};
use h3_contracts::{HeroId, PlayerId};
use h3_heroes::HeroDetails;
use h3_kernel::StateHasher;

use super::AdventureState;
use crate::movement::{details_of, hero_movement};
use crate::ruleset::AdventureSpell;

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

/// Why the adventure refuses a cast on the map (increment 15's design note §10; ADV-SPL-005,
/// ADV-SPL-006). A refusal changes nothing and spends nothing (the game spends the points only
/// after the cast is accepted).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastError {
    /// The hero is not one of the player's heroes on the map.
    NotYourHero(HeroId),
    /// Fly in a boat (ADV-SPL-005; the original's GENRLTXT 193).
    InBoat,
    /// Under `ADV-QUIRK-54` the spell is on already, or the hero's artifacts give the level (Angel
    /// Wings for Fly, Boots of Levitation for Water Walk on land): the original's silent nothing,
    /// free (survey `E08-spells.md` §7.1; batch E15's E15-11).
    AlreadyOn,
    /// With `ADV-QUIRK-54` off the cast's level would not exceed the hero's level now (the
    /// artifacts' 3 included).
    NotHigher,
    /// A spell this build does not cast on the map yet (Summon Boat, Scuttle Boat and Dimension
    /// Door until E08 T143: `ADV-STUB-22`).
    NotBuilt(AdventureSpell),
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
            Self::NotBuilt(spell) => write!(f, "`{}` is not cast on the map yet", spell.key()),
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

    /// Checks `player`'s cast of `spell` by its hero `hero` at `mastery` (0 to 3: the game's
    /// derivation on the hero's ground, `h3_heroes::spell_masteries_on`) as [`Self::apply_cast`]
    /// makes it (increment 15's design note §10.3, §10.4), its details in `heroes`, the game's
    /// store. Nothing changes.
    ///
    /// - **Fly** (ADV-SPL-005): refused in a boat; with `ADV-QUIRK-54` on while Fly is on or the
    ///   hero's artifacts give it a flying level (Angel Wings), off when the mastery would not
    ///   exceed its flying level now (the artifacts' and the spell's, the higher).
    /// - **Water Walk** (ADV-SPL-006): in a boat **accepted without either test** (A-49 accepted,
    ///   `ADV-QUIRK-57`: the original skips them there, and the level holds after landing the same
    ///   day; batch E15's E15-12); on land refused as Fly is, by its water-walking level (Boots of
    ///   Levitation).
    ///
    /// # Errors
    ///
    /// The [`CastError`] of the first test the cast fails.
    // rule: ADV-SPL-005, ADV-SPL-006
    pub fn check_cast(
        &self,
        core: &Ruleset,
        heroes: &[Arc<HeroDetails>],
        (player, hero): (PlayerId, HeroId),
        spell: AdventureSpell,
        mastery: u8,
    ) -> Result<(), CastError> {
        let index = self
            .heroes
            .binary_search_by_key(&hero, |h| h.id)
            .ok()
            .filter(|&index| self.heroes[index].owner == player)
            .ok_or(CastError::NotYourHero(hero))?;
        let afloat = self.heroes[index].boat.is_some();
        let (own, day) = (
            hero_movement(core, details_of(heroes, hero)),
            self.day_spells(hero),
        );
        let (cast, worn) = match spell {
            AdventureSpell::Fly if afloat => return Err(CastError::InBoat),
            AdventureSpell::Fly => (day.fly, own.flying),
            AdventureSpell::WaterWalk if afloat => return Ok(()),
            AdventureSpell::WaterWalk => (day.water_walk, own.water_walking),
            AdventureSpell::SummonBoat
            | AdventureSpell::ScuttleBoat
            | AdventureSpell::DimensionDoor => {
                // stub: ADV-STUB-22
                return Err(CastError::NotBuilt(spell));
            }
        };
        if core.quirks().is_enabled(SPELL_RECAST_REFUSED) {
            if cast.is_some() || worn.is_some() {
                return Err(CastError::AlreadyOn);
            }
        } else if cast.max(worn).is_some_and(|level| level >= mastery) {
            return Err(CastError::NotHigher);
        }
        Ok(())
    }

    /// The cast of `spell` by the hero `hero` at `mastery` that [`Self::check_cast`] accepted
    /// (ADV-SPL-005, ADV-SPL-006; increment 15's design note §10.3, §10.4), infallible: Fly sets
    /// the hero's flying level for the day to the mastery, Water Walk its water-walking level. Every
    /// reader of the hero's levels takes the higher of its artifacts' and its spell's from then on
    /// (the paths, the prices, the drop to 0: [`crate::movement::day_movement`]). No event of its
    /// own: the game tells the cast, and the changed spells are told after the command
    /// ([`crate::AdventureEventKind::DaySpellsChanged`]).
    // rule: ADV-SPL-005, ADV-SPL-006
    pub fn apply_cast(&mut self, hero: HeroId, spell: AdventureSpell, mastery: u8) {
        let mut spells = self.day_spells(hero);
        match spell {
            AdventureSpell::Fly => spells.fly = Some(mastery),
            AdventureSpell::WaterWalk => spells.water_walk = Some(mastery),
            // `check_cast` refuses the rest (`ADV-STUB-22`).
            AdventureSpell::SummonBoat
            | AdventureSpell::ScuttleBoat
            | AdventureSpell::DimensionDoor => {}
        }
        self.set_day_spells(hero, spells);
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
