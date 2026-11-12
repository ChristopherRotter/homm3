//! What the lobby shows of the whole game (E16 M5 T9a, v47): the server's map offers and its
//! hosted whole games, beside a battle's offers and games in `ServerMessage::Games`.
//!
//! Provisional (the 2a review list): the shapes; a map named by its operator's configuration or
//! its file's header; the players by colour; no spectators (2c); the computer named by the offer
//! alone (T10).

use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use super::MapIdentity;
use crate::wire::{PackRef, Phase, WireU64};

/// A map a whole game may be created from (`CreateMapGame`): the map, its rules and its seats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapOffer {
    /// The offer's id, as `CreateMapGame` names it.
    pub id: String,
    /// The map's name.
    pub name: String,
    /// Its side in tiles (maps are square).
    pub size: u16,
    /// Its levels: 1, or 2 with the underground.
    pub levels: u8,
    /// Its players, each a seat, by index (`GameJoin`'s `player`): each one's colour in the
    /// original's order (0 red to 7 pink).
    pub players: Vec<u8>,
    /// The game's difficulty (0 easy to 4 impossible in the original packs).
    pub difficulty: u8,
    /// The packs, in load order.
    pub packs: Vec<PackRef>,
    /// The profile's key.
    pub profile: String,
    /// The core ruleset's content hash (the manifest's).
    pub ruleset_hash: WireU64,
    /// The map's file.
    pub map: MapIdentity,
    /// The computer that plays its games' unowned battle sides (E16 M5 T10, v49).
    pub computer: Computer,
}

/// The server's computer that plays a whole game's unowned battle sides (E16 M5 T10): a
/// wandering monster's, a guarded object's guards; the original's battle AI (ADR-0027), its
/// commands the system actor's (`h3_game::Actor::System`), journaled as such.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Computer {
    /// On its side's world of the battle: it reads only what its side may know
    /// (`h3_battle_ai::choose_fair`; ADR-0027 point 4 as amended). An offer's default.
    Fair,
    /// On the whole battle, as a trusted host reads it (`h3_battle_ai::choose`): an offer's
    /// option.
    AllSeeing,
}

/// A seat of a whole game that belongs to an account, with the account's display name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeatedPlayer {
    /// The player: its index among the map's players.
    pub player: u8,
    /// The account's display name.
    pub display: String,
}

/// One hosted whole game in the lobby's list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapGameInfo {
    /// The game's id, as `GameJoin` names it.
    pub game: u32,
    /// The map offer it was created from.
    pub offer: String,
    /// Where it is: waiting until every seat is held at once, then running, or paused by its
    /// host.
    pub phase: Phase,
    /// The players whose seats a newcomer may take without a token: those never taken.
    pub open: Vec<u8>,
    /// The seats that belong to accounts, with their display names; a seat an anonymous player
    /// holds is not named.
    pub seated: Vec<SeatedPlayer>,
}
