//! A boat built at a town's Shipyard (TWN-SHY-001; E09 T59a, the town epic's design note §111,
//! TWN-SET-011; survey `E09-trades.md` §5): the town's spot, fixed when the game was built, while
//! no empty boat and no hero stands on it, and the boat launched there. The town part checks the
//! Shipyard and names the price and the boat's type (`h3_towns::check_boat`); the game checks the
//! rest and takes the price (`h3_game`'s `town/boat.rs`).

use h3_content::Ruleset;
use h3_contracts::{PlayerId, TownId};

use crate::command::{AdventureEvent, AdventureEventKind, Sink};
use crate::setup::Pos;
use crate::state::AdventureState;

impl AdventureState {
    /// The spot of `town` while a boat can be built there (TWN-SHY-001; design §111): its spot
    /// (TWN-SET-011, [`crate::StaticMap::town_boat_spot`]) while no empty boat lies on it and no hero stands
    /// there, in a boat (the map Shipyard's rule, ADV-SHY-002's boat tile). **`TWN-QUIRK-19`**
    /// (T-21 fixed): the original's town Shipyard tests neither and builds over them. None while
    /// taken, or for a town without a spot (a Shipyard stands only in a town with one).
    // rule: TWN-SHY-001
    pub fn free_town_boat_spot(&self, town: TownId) -> Option<Pos> {
        self.map().town_boat_spot(town).filter(|&tile| {
            !self.is_empty_boat(tile) && self.heroes().iter().all(|hero| hero.pos != tile)
        })
    }

    /// Builds a boat at `town`'s Shipyard for `player` (TWN-SHY-001; design §111, the original's
    /// `create_boat` 0x4BB250 from the town screen): an empty boat of `boat_type` on the town's
    /// spot while it is free ([`Self::free_town_boat_spot`]), facing 2, its owner `player` and no
    /// captain, told as [`AdventureEventKind::BoatBuilt`] to `player` and to the players that see
    /// heroes on the tile (ADV-FOG-008), as a map Shipyard's. The tile, or none and nothing built
    /// while the spot is taken; the game has checked the Shipyard and the treasury and takes the
    /// price after.
    // rule: TWN-SHY-001
    pub fn build_town_boat<S: Sink>(
        &mut self,
        core: &Ruleset,
        (town, boat_type): (TownId, u8),
        player: PlayerId,
        sink: &mut S,
    ) -> Option<Pos> {
        self.free_town_boat_spot(town).inspect(|&tile| {
            self.launch(tile, boat_type, player);
            sink.emit(AdventureEvent {
                kind: AdventureEventKind::BoatBuilt { tile, boat_type },
                seen_by: self.viewers(core, player, tile),
            });
        })
    }
}
