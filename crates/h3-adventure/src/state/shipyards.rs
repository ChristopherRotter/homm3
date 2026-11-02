//! The Shipyards (ADV-OWN-001, ADV-SHY-002, ADV-SHY-003; E08 T84b2; increment 9's design note §7,
//! §8; survey `E08-economy.md` §5): a visit flags a neutral Shipyard or another team's, then offers
//! a boat on its boat tile for its kind's price, or tells its visitor's owner why it cannot; the
//! player's or an ally's Shipyard also builds without a visit (`BuildBoat`). A built boat is an
//! empty boat of the Shipyard's type on the tile, its builder its owner.

use h3_content::Ruleset;
use h3_contracts::{HeroId, PlayerId};

use super::{AdventureState, allied};
use crate::command::{
    AdventureCommandError, AdventureEvent, AdventureEventKind, AdventureOutcome, Offer, OfferKind,
    ShipyardBlock, Sink, Viewers,
};
use crate::ruleset::{AdventureRuleset, ObjectKind, ObjectTypeId, Resources};
use crate::setup::Pos;

/// The price and boat type of a Shipyard's type, if `object_type` is one.
fn shipyard(part: &AdventureRuleset, object_type: ObjectTypeId) -> Option<(&Resources, u8)> {
    match &part.object_type(object_type)?.kind {
        ObjectKind::Shipyard {
            price, boat_type, ..
        } => Some((price, *boat_type)),
        _ => None,
    }
}

impl AdventureState {
    /// The visit of the Shipyard at object index `index`, of type `object_type`, by `player`'s hero
    /// `hero` on its entrance (ADV-OWN-001, ADV-SHY-002; E08 T84b2; increment 9's design note §7;
    /// survey `E08-economy.md` §5.2, the original's `visit_shipyard` 0x49E2E0): a neutral one or
    /// another team's is flagged for the visitor's owner first ([`Self::flag`]); then, whoever
    /// owns it, its boat offer for the kind's price while its boat tile is free
    /// ([`OfferKind::Boat`]; the game opens it when the treasury can pay, else tells the player it
    /// cannot), or, without a tile or with a boat or a hero on it, the reason told to the visitor's
    /// owner alone ([`AdventureEventKind::ShipyardBlocked`]: the original's messages). A computer
    /// player's hero is offered the boat as a human's (the original's computer builds through its
    /// own code, E10).
    // rule: ADV-OWN-001, ADV-SHY-002
    pub(super) fn visit_shipyard<S: Sink>(
        &mut self,
        rules: (&Ruleset, &AdventureRuleset),
        (player, hero): (PlayerId, HeroId),
        (index, object_type): (usize, ObjectTypeId),
        price: &Resources,
        sink: &mut S,
    ) -> AdventureOutcome {
        if !self.owner(index).is_some_and(|owner| allied(owner, player)) {
            self.flag(rules, (index, object_type), player, sink);
        }
        let anchor = self.object_at(index).anchor;
        match self.boat_site(index) {
            Ok(_) => AdventureOutcome {
                offer: Some(Offer {
                    hero,
                    object_type,
                    anchor,
                    kind: OfferKind::Boat {
                        price: price.clone(),
                    },
                }),
                ..AdventureOutcome::default()
            },
            Err(reason) => {
                sink.emit(AdventureEvent {
                    kind: AdventureEventKind::ShipyardBlocked {
                        object_type,
                        anchor,
                        reason,
                    },
                    seen_by: Viewers::NONE.with(player),
                });
                AdventureOutcome::default()
            }
        }
    }

    /// The boat tile of the Shipyard at object index `index` while a boat can be built there
    /// (ADV-SHY-002; survey `E08-economy.md` §5.2): [`ShipyardBlock::NoTile`] without one (none was
    /// free water when the game was built, ADV-SHY-001), [`ShipyardBlock::Taken`] while an empty
    /// boat lies on it or a hero stands there (in a boat: the original's test of a boat's or a
    /// hero's entrance on it). Nothing else ever stands on a boat tile: the build found no object
    /// there and the objects added in play stand on land (ADV-MON-014).
    pub(super) fn boat_site(&self, index: usize) -> Result<Pos, ShipyardBlock> {
        let tile = self
            .object_at(index)
            .resolved
            .boat_tile()
            .ok_or(ShipyardBlock::NoTile)?;
        if self.is_empty_boat(tile) || self.heroes.iter().any(|hero| hero.pos == tile) {
            return Err(ShipyardBlock::Taken);
        }
        Ok(tile)
    }

    /// Builds a boat at the Shipyard at object index `index` for `player` (ADV-SHY-002,
    /// ADV-SHY-003; E08 T84b2; increment 9's design note §7; the original's `create_boat` 0x4BB250
    /// from the boat dialog, survey `E08-economy.md` §5.2): an empty boat of the Shipyard's type on
    /// its boat tile, facing 2, its owner `player`, no captain ([`Self::launch`]), told as
    /// [`AdventureEventKind::BoatBuilt`] to `player` and to the players that see heroes on the
    /// tile, as they see empty boats there (ADV-FOG-008). The caller has checked the tile
    /// ([`Self::offer_stands`] for an accepted offer, [`Self::boat_price`] for `BuildBoat`) and the
    /// game takes the price; nothing happens without a free tile.
    // rule: ADV-SHY-002, ADV-SHY-003
    pub(super) fn build_boat<S: Sink>(
        &mut self,
        (core, part): (&Ruleset, &AdventureRuleset),
        index: usize,
        player: PlayerId,
        sink: &mut S,
    ) {
        let built = self.boat_site(index).ok().zip(
            self.object_at(index)
                .object_type
                .and_then(|id| shipyard(part, id)),
        );
        let Some((tile, (_, boat_type))) = built else {
            return;
        };
        self.launch(tile, boat_type, player);
        sink.emit(AdventureEvent {
            kind: AdventureEventKind::BoatBuilt { tile, boat_type },
            seen_by: self.viewers(core, player, tile),
        });
    }

    /// `BuildBoat`'s validation, read-only (ADV-SHY-003; E08 T84b2; increment 9's design note §8),
    /// and the price the game checks against the treasury before the command and takes after it:
    /// the Shipyard named by its type and anchor, in this order: the object is on the map with that
    /// type, a Shipyard, with a tile `player` has explored (one error for both,
    /// [`AdventureCommandError::UnknownObject`], so a hidden Shipyard stays hidden); its owner is
    /// `player` or an ally (`NotOwner`); it has a boat tile (`NoBoatTile`); the tile is free
    /// (`BoatTileTaken`). The original's click on an own Shipyard (§5.3), as VCMI's and the
    /// wiki's.
    ///
    /// # Errors
    ///
    /// The first of the checks above that fails.
    // rule: ADV-SHY-003
    pub fn boat_price<'p>(
        &self,
        part: &'p AdventureRuleset,
        player: PlayerId,
        object_type: ObjectTypeId,
        anchor: Pos,
    ) -> Result<&'p Resources, AdventureCommandError> {
        self.buildable(part, player, object_type, anchor)
            .map(|(_, price)| price)
    }

    /// `BuildBoat` (ADV-SHY-003; E08 T84b2): [`Self::boat_price`]'s checks, then the boat built
    /// ([`Self::build_boat`]); the game has checked the treasury and takes the price after.
    pub(super) fn build_boat_command<S: Sink>(
        &mut self,
        rules: (&Ruleset, &AdventureRuleset),
        player: PlayerId,
        (object_type, anchor): (ObjectTypeId, Pos),
        sink: &mut S,
    ) -> Result<AdventureOutcome, AdventureCommandError> {
        let (index, _) = self.buildable(rules.1, player, object_type, anchor)?;
        self.build_boat(rules, index, player, sink);
        Ok(AdventureOutcome::default())
    }

    /// The Shipyards at which `player` may build a boat now, by type and anchor in object order
    /// (ADV-SHY-003; E08 T84b2: `h3_game::legal_commands`' `BuildBoat`s), each with its price:
    /// those [`Self::boat_price`] accepts (the setup refuses two typed objects of one type on one
    /// anchor, so each names its own). None on a map without ownable objects.
    pub fn buildable_shipyards<'p>(
        &self,
        part: &'p AdventureRuleset,
        player: PlayerId,
    ) -> impl Iterator<Item = (ObjectTypeId, Pos, &'p Resources)> {
        self.map.ownables().iter().filter_map(move |&index| {
            let object = self.object_at(index);
            let object_type = object.object_type?;
            self.check_shipyard(part, player, (index, object_type))
                .ok()
                .map(|price| (object_type, object.anchor, price))
        })
    }

    /// [`Self::boat_price`]'s checks: the Shipyard's object index and its price.
    fn buildable<'p>(
        &self,
        part: &'p AdventureRuleset,
        player: PlayerId,
        object_type: ObjectTypeId,
        anchor: Pos,
    ) -> Result<(usize, &'p Resources), AdventureCommandError> {
        // A Shipyard is ownable and never leaves the map; the setup refuses two typed objects of
        // one type on one anchor (`SharedAnchor`), so at most one matches.
        let index = self
            .map
            .ownables()
            .iter()
            .copied()
            .find(|&index| {
                let object = self.object_at(index);
                object.object_type == Some(object_type) && object.anchor == anchor
            })
            .ok_or(AdventureCommandError::UnknownObject)?;
        self.check_shipyard(part, player, (index, object_type))
            .map(|price| (index, price))
    }

    /// [`Self::boat_price`]'s checks of the ownable object at object index `index`, of type
    /// `object_type`: its price.
    fn check_shipyard<'p>(
        &self,
        part: &'p AdventureRuleset,
        player: PlayerId,
        (index, object_type): (usize, ObjectTypeId),
    ) -> Result<&'p Resources, AdventureCommandError> {
        let (price, _) = shipyard(part, object_type)
            .filter(|_| {
                self.object_at(index)
                    .tiles(&self.map)
                    .any(|pos| self.is_explored(player, pos))
            })
            .ok_or(AdventureCommandError::UnknownObject)?;
        if !self.owner(index).is_some_and(|owner| allied(owner, player)) {
            return Err(AdventureCommandError::NotOwner);
        }
        self.boat_site(index).map_err(|reason| match reason {
            ShipyardBlock::NoTile => AdventureCommandError::NoBoatTile,
            ShipyardBlock::Taken => AdventureCommandError::BoatTileTaken,
        })?;
        Ok(price)
    }
}
