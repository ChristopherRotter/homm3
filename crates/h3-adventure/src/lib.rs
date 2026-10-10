//! The adventure context (ADR-0016, E08): the adventure map, heroes on the map with their armies,
//! movement and pathing, map objects as closed behaviour kinds with data parameters, and fog of
//! war; its ruleset part and that part's compiler (ADR-0015 point 2). No domain crate depends on
//! another: players, turns and the calendar belong to `h3-game`, which drives this context.
//!
//! So far: the ruleset part ([`AdventureRuleset`], E08 T4). The setup and the state with its
//! persistent grids arrive in T5, movement in T7, fog of war and the projection in T8. Pointer identity (`ptr_eq`, `as_ptr`) is allowed
//! only in the persistent module `src/persistent.rs` (`arch.toml`, ADR-0016 point 2).
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

mod ruleset;

pub use ruleset::{
    AdventureCompileError, AdventurePack, AdventureParams, AdventureParamsDef, AdventureRuleset,
    MAX_LAND_POINTS, Passage, RiverDef, RiverId, RiverKind, RoadDef, RoadId, RoadKind,
    TerrainMovement, TerrainMovementDef,
};

/// Version of the adventure state's canonical hash encoding (`docs/08-work-process.md`,
/// "Versions"; owned by the adventure lane). Bump it with any change to what the hash covers.
pub const ADVENTURE_STATE_HASH_VERSION: u32 = 1;

/// Version of the adventure ruleset part's canonical hash encoding (ADR-0015 point 2; owned by the
/// adventure lane).
pub const ADVENTURE_RULESET_HASH_VERSION: u32 = 1;
