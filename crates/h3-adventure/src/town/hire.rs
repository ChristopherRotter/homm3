//! A hired hero's placement on its town's entrance (TWN-TAV-003; E09 T38a1, the town epic's
//! design note §64; ADR-0016 point 3, E09's increment 4): the adventure's half of the game's
//! `TownCommand::Hire`. The game checks the town, its owner, its Tavern, the player's slot and the
//! treasury; here the player's room on the map and the entrance, each refusal changing nothing;
//! then the hero placed with the away record the game gives. No reveal and no visit (the
//! original's `town_tavern_hire` 0x5C12E0 and `hero_place` 0x4D7900, read whole: no reveal call).
//! Since E09 T39a the map's Tavern's too (ADV-VIS-007, GAM-DEC-010; design note §66): its visit
//! ([`TavernVisit`]) and its hire's room and tile ([`AdventureState::check_map_tavern_hire`]),
//! the hero placed on its visitable tile.

use h3_content::Ruleset;
use h3_contracts::{HeroId, PlayerId, TownId};

use super::rows::{TownHeroError, arrive};
use crate::command::Viewers;
use crate::ruleset::ObjectTypeId;
use crate::setup::Pos;
use crate::state::{AdventureState, HeroOnMap};

/// A visit of the adventure map's Tavern (ADV-VIS-007, GAM-DEC-010; E09 T39a, the town epic's
/// design note §66; the original's `map_tavern_visit` 0x5D7E90): the visiting hero, the Tavern by
/// type and anchor, and its visitable tile, where a hire places the hero (`map_tavern_hire`
/// 0x4D7890: `hero_place` at the visited tile).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TavernVisit {
    /// The visiting hero, on a tile beside the Tavern's visitable tile.
    pub hero: HeroId,
    /// The Tavern's type.
    pub object_type: ObjectTypeId,
    /// The Tavern's anchor.
    pub anchor: Pos,
    /// The Tavern's visitable tile.
    pub entrance: Pos,
}

impl AdventureState {
    /// Whether `player` may place a hired hero on a map Tavern's visitable tile `entrance`
    /// (GAM-DEC-010; E09 T39a, the town epic's design note §66): fewer than `most` heroes of the
    /// player's on the map, a town's garrison heroes not counted ([`TownHeroError::TooManyHeroes`],
    /// the tavern dialog's Recruit button, 0x5D81EE), then no hero on the tile, another player's
    /// included ([`TownHeroError::EntranceHeld`]: VCMI's refusal; the original never reaches it,
    /// its visit skipped while the visitor stands there).
    ///
    /// # Errors
    ///
    /// The first refusal, in that order.
    // rule: GAM-DEC-010
    pub fn check_map_tavern_hire(
        &self,
        player: PlayerId,
        (entrance, most): (Pos, u8),
    ) -> Result<(), TownHeroError> {
        if !self.has_room(player, most) {
            return Err(TownHeroError::TooManyHeroes);
        }
        if !self.tile_free(entrance) {
            return Err(TownHeroError::EntranceHeld);
        }
        Ok(())
    }

    /// Whether no hero stands on `tile`, another player's included: a map Tavern's hire's tile
    /// test ([`Self::check_map_tavern_hire`]; E09 T39a: a listing's too).
    // rule: GAM-DEC-010
    pub fn tile_free(&self, tile: Pos) -> bool {
        self.heroes().iter().all(|on| on.pos != tile)
    }

    /// Whether `player` may place a hired hero on `town`'s entrance (TWN-TAV-003; the tavern
    /// dialog's Recruit button, 0x5D81EE: disabled with `most` heroes on the map or a visitor in
    /// the town), giving the entrance: the map's town ([`TownHeroError::UnknownTown`]), fewer than
    /// `most` heroes of the player's on the map, a town's garrison heroes not counted
    /// ([`TownHeroError::TooManyHeroes`], TWN-ROW-003's most), no hero on the entrance, another
    /// player's included ([`TownHeroError::EntranceHeld`]).
    ///
    /// # Errors
    ///
    /// The first refusal, in that order.
    // rule: TWN-TAV-003
    pub fn check_tavern_hire(
        &self,
        player: PlayerId,
        (town, most): (TownId, u8),
    ) -> Result<Pos, TownHeroError> {
        let (_, entrance) = self.town_entrance(town)?;
        if !self.has_room(player, most) {
            return Err(TownHeroError::TooManyHeroes);
        }
        if self.heroes().iter().any(|on| on.pos == entrance) {
            return Err(TownHeroError::EntranceHeld);
        }
        Ok(entrance)
    }

    /// Whether `player` has fewer than `most` heroes on the map, a town's garrison heroes not
    /// counted: [`Self::check_tavern_hire`]'s room (TWN-TAV-003; E09 T38a2: a listing's test once).
    // rule: TWN-TAV-003
    pub fn has_room(&self, player: PlayerId, most: u8) -> bool {
        let on_map = self.heroes().iter().filter(|on| on.owner == player).count();
        on_map < usize::from(most)
    }

    /// Places `hero`, a hired hero with the away record the game gives (its army, points left and
    /// day's maximum, formation and tactics flag; no battle bonuses, no timed marks), on its tile,
    /// the entrance [`Self::check_tavern_hire`] gave, on land, among the heroes on the map in id order:
    /// it is the town's visitor (TWN-VIS-002); since E09 T39a a map Tavern's visitable tile
    /// ([`Self::check_map_tavern_hire`], GAM-DEC-010). Nothing is revealed or visited
    /// (TWN-TAV-003).
    /// Returns the players that see the tile, its owner among them (ADV-FOG-004, -005: a step's
    /// `seen_to`), whom the game tells the hire.
    // rule: TWN-TAV-003, GAM-DEC-010
    pub fn place_hired(&mut self, core: &Ruleset, hero: HeroOnMap) -> Viewers {
        let (owner, entrance) = (hero.owner, hero.pos);
        let seen_by = self.viewers(core, owner, entrance);
        let (heroes, _) = self.rows_mut();
        arrive(heroes, hero, entrance);
        seen_by
    }
}

#[cfg(test)]
mod tests;
