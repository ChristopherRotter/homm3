//! What a whole game is made of (E16 M5 T4; ADR-0018 point 10 for the whole game): the
//! adventure replay header's fields with every part's hash (ADR-0015 point 2), the map it is
//! played on, the battlefield's size and the handle-to-key tables of every content kind the
//! game's messages name. Sent once, before the game's first observation (T5: at join, and as a
//! game record's header); a client previews with its own ruleset only when the hashes equal its
//! own.
//!
//! Provisional (E16 M5 part 1's review): the shapes, the map named by its file.

use alloc::string::String;
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::wire::{Algorithm, Field, KeyTables, PackRef, WireU64};

/// What a whole game is made of: its rules' identity (the adventure replay header's fields,
/// `h3_sim::adventure::replay::ReplayHeader`, and the battle part's hash), its map, the
/// battlefield and the key tables.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameManifest {
    /// [`crate::PROTOCOL_VERSION`].
    pub protocol: u32,
    /// The engine's version (informational).
    pub engine: String,
    /// The packs, in load order.
    pub packs: Vec<PackRef>,
    /// The profile's key.
    pub profile: String,
    /// The core ruleset's content hash.
    pub ruleset_hash: WireU64,
    /// The battle part's content hash (the game's battles; ADR-0015 point 2).
    pub battle_ruleset_hash: WireU64,
    /// The adventure part's content hash.
    pub adventure_ruleset_hash: WireU64,
    /// The town part's content hash.
    pub town_ruleset_hash: WireU64,
    /// The random number generator.
    pub rng: Algorithm,
    /// The state hash.
    pub hash: Algorithm,
    /// The battle state hash's encoding version.
    pub battle_state_hash_version: u32,
    /// The adventure state hash's encoding version.
    pub adventure_state_hash_version: u32,
    /// The game state hash's encoding version.
    pub game_state_hash_version: u32,
    /// The map the game is played on.
    pub map: MapIdentity,
    /// The battlefield's size.
    pub field: Field,
    /// The battle's content keys by handle (the battle manifest's tables): creatures, spells,
    /// skills, hero types, artifacts, factions, terrains, obstacles, battlefields.
    pub keys: KeyTables,
    /// The content keys of the kinds only the whole game names.
    pub game_keys: GameKeyTables,
}

/// The map a game is played on, as an adventure replay names it
/// (`h3_sim::adventure::replay::MapRef`, ADV-Q2) but for its path, which is the server's: the
/// hash of the map file's bytes and the import's version. A client draws the map from its own
/// copy only when both are equal; the map itself never travels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapIdentity {
    /// The file's content hash (`h3_sim::adventure::replay::map_content_hash`).
    pub content_hash: WireU64,
    /// `h3_map::MAP_IMPORT_VERSION` of the import the game started from.
    pub import_version: u32,
}

/// The content keys by handle of the kinds only the whole game names: entry `i` is the key of
/// handle `i` (`base:gold`), so clients find assets by key.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameKeyTables {
    /// Adventure object types.
    pub object_types: Vec<String>,
    /// Resources.
    pub resources: Vec<String>,
    /// Roads.
    pub roads: Vec<String>,
    /// Rivers.
    pub rivers: Vec<String>,
    /// Each faction's town buildings, by faction handle: entry `f` holds the keys within the
    /// faction's town record by building index (a town's buildings and a build name them by that
    /// index), empty for a faction without a town.
    pub buildings: Vec<Vec<String>>,
}
