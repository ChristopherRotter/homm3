//! The adventure context (ADR-0016, E08): the adventure map, heroes on the map with their armies,
//! movement and pathing, map objects as closed behaviour kinds with data parameters, and fog of
//! war; its ruleset part and that part's compiler (ADR-0015 point 2). No domain crate depends on
//! another: players, turns and the calendar belong to `h3-game`, which drives this context.
//!
//! So far: the ruleset part ([`AdventureRuleset`], E08 T4), the setup ([`AdventureSetup`]) and
//! the state with its static map ([`AdventureState`], T5a), a day's movement points and the
//! start-of-turn step (T6b), the price of a step, `MoveHero` ([`AdventureCommand`]) and the
//! reachable-tiles query ([`Reach`], T7b), the persistent grid ([`BitGrid`], T5b), each player's
//! explored tiles with the reveals, movement on explored tiles only and the events' viewers
//! ([`Viewers`], T8a), the fringe under `ADV-QUIRK-1`, the observation per player
//! ([`AdventureObservation`]) and the events' projection ([`AdventureEvent::project`], T8b1),
//! object types in the ruleset part and the setup ([`ObjectType`], T12a), the objects' contents
//! drawn at the build, the objects that left the map and the tiles they freed, and the typed
//! objects a viewer sees ([`ObjectView`], T12b), visits from a neighbour and entered (T13b, T14a),
//! the Cartographer's offer ([`Offer`]) and its acceptance (T14b), the observatories, the
//! Keymaster's tents with the keys and the border guards with their offer (T15a), the wandering
//! monsters (T20), the guarded piles ([`Guards`], T21a).
//! Pointer identity (`ptr_eq`, `as_ptr`) is allowed only in the persistent module
//! `src/persistent.rs` (`arch.toml`, ADR-0016 point 2).
//!
//! Logic crate: pure, deterministic, `no_std` + `alloc` (ADR-0003, ADR-0005). The lint header
//! below is required and checked by `cargo xtask check-arch`.
#![no_std]
#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::float_arithmetic,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::exit
)]
#![warn(missing_docs)]

extern crate alloc;

mod answer;
mod command;
mod fog;
mod movement;
mod observe;
mod persistent;
mod reach;
mod ruleset;
mod setup;
mod split;
mod state;

#[cfg(test)]
mod testing;

pub use command::{
    AdventureCommand, AdventureCommandError, AdventureEvent, AdventureEventKind, AdventureOutcome,
    BattleStart, Combatant, Defender, Meeting, NoRoom, Offer, OfferKind, Sink, Viewers,
};
pub use movement::{day_points, diagonal_cost, land_table_points, native_terrain};
pub use observe::{AdventureObservation, HeroView, MonsterView, ObjectView, OwnHeroView, TileView};
pub use persistent::BitGrid;
pub use reach::Reach;
pub use ruleset::{
    AdventureCompileError, AdventurePack, AdventureParams, AdventureParamsDef, AdventureRuleset,
    BattleTerrains, BattleTerrainsDef, KEY_COLOURS, MAX_DIFFICULTIES, MAX_GAIN, MAX_LAND_POINTS,
    MAX_MONSTER_COUNT, MAX_PICKUP_AMOUNT, MAX_PLAYERS, MonsterParams, MonsterParamsDef, ObjectKind,
    ObjectKindDef, ObjectType, ObjectTypeDef, ObjectTypeId, Passage, Pickup, PickupDef, RiverDef,
    RiverId, RiverKind, RoadDef, RoadId, RoadKind, StartingResources, StartingResourcesDef,
    TerrainMovement, TerrainMovementDef, TerrainWalker, TerrainWalkerDef, Visit,
};
pub use setup::{
    AdventureSetup, ArmyError, Guards, HeroSetup, MapSetup, MonsterSetup, ObjectSetup, Pos,
    SetupError, StandError, TileMask, TileSetup,
};
pub use split::{TileRand, split};
pub use state::{AdventureState, HeroOnMap, MapObject, Monster, StaticMap, StaticTile};

/// Version of the adventure state's canonical hash encoding (`docs/08-work-process.md`,
/// "Versions"; owned by the adventure lane). Bump it with any change to what the hash covers.
/// v1 (E08 T5a): the first encoding ([`AdventureState`]'s `CanonicalHash`). v2 (E08 T6b): a
/// hero's movement points. v3 (E08 T8a): each player's explored tiles. v4 (E08 T12): each
/// object's type and contents. v5 (E08 T14b): the Cartographer types each player bought. v6 (E08
/// T15a): the keys each player holds. v7 (E08 T18b): a hero's formation. v8 (E08 T19a): whether a
/// tile of the static map is on the coast. v9 (E08 T20a2a): the tiles each object lies on, each
/// monster's creature, count, aggression, flags and treasure, the armies battles left on objects
/// and the game's difficulty. v10 (E08 T21a): each guarded pile's guards.
pub const ADVENTURE_STATE_HASH_VERSION: u32 = 10;

/// Version of the adventure ruleset part's canonical hash encoding (ADR-0015 point 2; owned by the
/// adventure lane). v2 (E08 T5a): the setup bounds and the army slots. v3 (E08 T6b): the starting
/// stack chances. v4 (E08 T12a): the object types. v5 (E08 T13a): the starting resources by
/// difficulty. v6 (E08 T19a): the battle terrains by tile. v7 (E08 T20a2a): the monster and
/// ground kinds, the monsters' parameters and the creatures no random monster becomes. v8 (E08
/// T20b1): the resource a monster's price to join is paid in. v9 (E08 T29): the terrains' costs
/// by Pathfinding level. v10 (E08 T40): the fly and water-walk prices, the terrain walkers and a
/// ground's loss of the native price (Cursed Ground's `no_native`). v11 (E08 T50): a stack's most
/// creatures.
pub const ADVENTURE_RULESET_HASH_VERSION: u32 = 11;
