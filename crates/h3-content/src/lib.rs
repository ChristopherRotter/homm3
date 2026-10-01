//! Content definitions, content-pack model and the pure `Ruleset` compiler (ADR-0006).
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

mod error;
mod flags;
mod key;
mod pack;
mod quirks;
mod ruleset;

pub use error::CompileError;
pub use flags::{CreatureFlag, CreatureFlags};
pub use key::{ContentKey, KeyError};
pub use pack::{
    BattleParamsDef, ContentPack, CreatureDef, CreatureInfoDef, CreatureOverride, CreaturePatch,
    FactionDef, PackManifest, ProfileDef, ResourceDef,
};
pub use quirks::{QuirkFlag, QuirkRegistry, Quirks, REGISTRY};
pub use ruleset::{
    BattleParams, CreatureInfo, CreatureType, CreatureTypeId, FactionId, FactionInfo, FieldParams,
    MAX_ARMY_SLOTS, MAX_FIELD_HEXES, RULESET_HASH_VERSION, ResourceId, ResourceInfo, Ruleset,
};
