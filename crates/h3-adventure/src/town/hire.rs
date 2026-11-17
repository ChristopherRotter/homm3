//! A hired hero's placement on its town's entrance (TWN-TAV-003; E09 T38a1, the town epic's
//! design note §64; ADR-0016 point 3, E09's increment 4): the adventure's half of the game's
//! `TownCommand::Hire`. The game checks the town, its owner, its Tavern, the player's slot and the
//! treasury; here the player's room on the map and the entrance, each refusal changing nothing;
//! then the hero placed with the away record the game gives. No reveal and no visit (the
//! original's `town_tavern_hire` 0x5C12E0 and `hero_place` 0x4D7900, read whole: no reveal call).

use h3_content::Ruleset;
use h3_contracts::{PlayerId, TownId};

use super::rows::{TownHeroError, arrive};
use crate::command::Viewers;
use crate::setup::Pos;
use crate::state::{AdventureState, HeroOnMap};

impl AdventureState {
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
        let on_map = self.heroes().iter().filter(|on| on.owner == player).count();
        if on_map >= usize::from(most) {
            return Err(TownHeroError::TooManyHeroes);
        }
        if self.heroes().iter().any(|on| on.pos == entrance) {
            return Err(TownHeroError::EntranceHeld);
        }
        Ok(entrance)
    }

    /// Places `hero`, a hired hero with the away record the game gives (its army, points left and
    /// day's maximum, formation and tactics flag; no battle bonuses, no timed marks), on its tile,
    /// the entrance [`Self::check_tavern_hire`] gave, on land, among the heroes on the map in id order:
    /// it is the town's visitor (TWN-VIS-002). Nothing is revealed or visited (TWN-TAV-003).
    /// Returns the players that see the tile, its owner among them (ADV-FOG-004, -005: a step's
    /// `seen_to`), whom the game tells the hire.
    // rule: TWN-TAV-003
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
