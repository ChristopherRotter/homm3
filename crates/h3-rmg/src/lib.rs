//! The map generation context (ADR-0029, E14): a port of the original's random map generator
//! (`Heroes3.exe`'s, SoD's code), so that the same template table, choices and seed give the same
//! map as the original, byte for byte once written. One pure entry, `generate` (ADR-0029 point
//! 2), returns a generated map that `h3_map`'s writer serialises; the generator writes no file.
//! Its stream is the C runtime's `rand`, `h3_kernel::rng::CrtRand` (point 3); its floats are
//! binary64 through `h3_kernel::num` (point 4); its tables are the rmg part, compiled from the
//! packs' `rmg.toml` by `h3-pack-rmg` (point 5).
//!
//! So far: the rmg part ([`RmgRuleset`], E14 T2b; the generator's tables, T2c): the three tables
//! and the executable's constants compiled from the packs; the settings ([`Settings`],
//! [`resolve_random`]); [`generate`]'s frame up to step 1 (E14 T2c): the seed, water's Random,
//! the templates kept and drawn, the banned heroes, the positions, with its [`Trace`]. The steps
//! come with increments 3 to 6, the writer's content with 7.
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

mod generate;
mod part;
mod settings;
mod template;

pub use generate::{
    GenerateError, GeneratedMap, NoTrace, RMG_GENERATOR_VERSION, SITE_TEMPLATE, SITE_WATER,
    Trace, generate,
};
pub use part::{
    Connection, ConnectionOverride, GeneratorTables, NO_LIMIT, OBJECT_TYPES, ObjectTemplate,
    ObjectTemplateLine, ObjectTemplateOverride, Obstacle, RmgCompileError, RmgPack, RmgRuleset,
    Strength, Template, TypeLimits, Zone, ZoneKind,
};
pub use settings::{
    COLOURS, Choices, MapFormat, MapSize, Monsters, Settings, SettingsError, TOWN_TYPES, Water,
    resolve_random,
};
pub use template::{Fit, KeptConnection, KeptTemplate, PositionError, kept_templates, positions};

/// The original's `rand` (`h3_kernel::rng::CrtRand`), the stream [`resolve_random`] draws from.
pub use h3_kernel::rng::CrtRand;

/// Version of [`RmgRuleset`]'s canonical hash (ADR-0029 point 7): bumped whenever the bytes the
/// part hashes change for the same tables (2: the generator's tables, E14 T2c).
pub const RMG_RULESET_HASH_VERSION: u32 = 2;

#[cfg(test)]
mod tests;
