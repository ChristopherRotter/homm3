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
mod army;
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

pub use army::{ArmyCommand, ArmyCommandError, ArmyRef, ArmySlot, split_range};
pub use command::{
    AdventureCommand, AdventureCommandError, AdventureEvent, AdventureEventKind, AdventureOutcome,
    BattleStart, Choice, ChoiceKind, Combatant, Defender, HeroGains, Meeting, NoRoom, Offer,
    OfferKind, Pick, Sink, UniversityVisit, Viewers,
};
pub use movement::{day_points, diagonal_cost, land_table_points, native_terrain, sea_day_points};
pub use observe::{
    AdventureObservation, BoatView, HeroView, Known, KnownObject, MonsterView, ObjectView,
    OwnHeroView, TileView,
};
pub use persistent::BitGrid;
pub use reach::Reach;
pub use ruleset::{
    AdventureCompileError, AdventurePack, AdventureParams, AdventureParamsDef, AdventureRuleset,
    BattleTerrains, BattleTerrainsDef, CalendarParams, Chest, ChestDef, Find, FindContent,
    FindContentDef, FindDef, Gift, GiftDef, Gives, KEY_COLOURS, MAX_DIFFICULTIES, MAX_GAIN,
    MAX_LAND_POINTS, MAX_MONSTER_COUNT, MAX_PICKUP_AMOUNT, MAX_PLAYERS, MonsterParams,
    MonsterParamsDef, MonthParams, MonthParamsDef, ObjectKind, ObjectKindDef, ObjectType,
    ObjectTypeDef, ObjectTypeId, OnceBonus, Passage, Pickup, PickupDef, Pyramid, PyramidDef,
    RiverDef, RiverId, RiverKind, RoadDef, RoadId, RoadKind, StartingResources,
    StartingResourcesDef, TerrainMovement, TerrainMovementDef, TerrainWalker, TerrainWalkerDef,
    Turn, TurnDef, Visit,
};
pub use setup::{
    AdventureSetup, ArmyError, Bans, BoatSetup, EventSetup, Guards, HeroSetup, Holds, MapSetup,
    MonsterSetup, ObjectSetup, Pos, RewardRecord, Scenery, Seat, SetupError, StandError, TileMask,
    TileSetup,
};
pub use split::{TileRand, split, split_with_second};

/// The hero model's types the setup and the kinds name (E08 T61a): a gift's choice of primary
/// skills ([`Gives::Choice`]), an artifact on an object ([`Holds::Artifact`]), a reward record's
/// hero half ([`RewardRecord::hero`]) and a Scholar's gift ([`Holds::Gift`]), so the import needs
/// no edge to the hero model.
pub use h3_heroes::{ArtifactInstance, HeroReward, PrimarySkill, ScholarGift};
pub use state::{
    AdventureState, BACKGROUND_RADIUS, Boat, HeroOnMap, MapObject, Monster, Resolved, StaticMap,
    StaticTile, Visits, Weekly,
};

/// Version of the adventure state's canonical hash encoding (`docs/08-work-process.md`,
/// "Versions"; owned by the adventure lane). Bump it with any change to what the hash covers.
/// v1 (E08 T5a): the first encoding ([`AdventureState`]'s `CanonicalHash`). v2 (E08 T6b): a
/// hero's movement points. v3 (E08 T8a): each player's explored tiles. v4 (E08 T12): each
/// object's type and contents. v5 (E08 T14b): the Cartographer types each player bought. v6 (E08
/// T15a): the keys each player holds. v7 (E08 T18b): a hero's formation. v8 (E08 T19a): whether a
/// tile of the static map is on the coast. v9 (E08 T20a2a): the tiles each object lies on, each
/// monster's creature, count, aggression, flags and treasure, the armies battles left on objects
/// and the game's difficulty. v10 (E08 T21a): each guarded pile's guards. v11 (E08 T50): each
/// hero's tactics flag. v12 (E08 T54): each object's scenery. v13 (E08 T61a): what each object
/// holds from the map (increment 7's kinds) and a monster's treasure artifact. v14 (E08 T61b): what
/// the build drew for each object, and the visits' block (none, or the heroes' marks, the
/// players' knowledge and the emptied objects). v15 (E08 T62b2): each player's seat (its colour
/// and whether it counts as human). v16 (E08 T72): the boats, each hero's boat and day's maximum.
/// v17 (E08 T83a): an abandoned mine's resources in the static map's objects ([`Holds`]' tag 7).
/// v18 (E08 T83b1): what the build drew for abandoned mines and Shipyards ([`Resolved`]' tags 7
/// and 8), the ownable objects' owners and the weekly objects' contents and marks. v19 (E08
/// T83b2a): the objects added in play (in the occupancy's encoding) and the monsters' fractions.
pub const ADVENTURE_STATE_HASH_VERSION: u32 = 19;

/// Version of the adventure ruleset part's canonical hash encoding (ADR-0015 point 2; owned by the
/// adventure lane). v2 (E08 T5a): the setup bounds and the army slots. v3 (E08 T6b): the starting
/// stack chances. v4 (E08 T12a): the object types. v5 (E08 T13a): the starting resources by
/// difficulty. v6 (E08 T19a): the battle terrains by tile. v7 (E08 T20a2a): the monster and
/// ground kinds, the monsters' parameters and the creatures no random monster becomes. v8 (E08
/// T20b1): the resource a monster's price to join is paid in. v9 (E08 T29): the terrains' costs
/// by Pathfinding level. v10 (E08 T40): the fly and water-walk prices, the terrain walkers and a
/// ground's loss of the native price (Cursed Ground's `no_native`). v11 (E08 T50): a stack's most
/// creatures. v12 (E08 T61a): increment 7's kinds and their parameters. v13 (E08 T72): the sea's
/// kinds (the find's fixed entries, its lost turn and its leaving, a message, a ground of the sea,
/// a hole), the sea points, the sea ground's share and the battle terrains at sea. v14 (E08 T83a):
/// increment 9's seven kinds (mines, abandoned mines, Lighthouses, Shipyards, Windmills, Water
/// Wheels, Gardens), the monsters' growth, the calendar and the months. v15 (E08 T83b1): the flag
/// radius.
pub const ADVENTURE_RULESET_HASH_VERSION: u32 = 15;
