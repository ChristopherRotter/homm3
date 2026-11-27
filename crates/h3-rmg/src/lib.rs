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
//! the templates kept and drawn, the banned heroes, the positions, with its [`Trace`]; the
//! generator's map ([`Cells`], dumped as the phase oracle's) and step 1, the zones laid out
//! ([`ZoneGenerator`], E14 T3c); step 2, each level partitioned (the triangulation, the water
//! zones, the borders, the fills, the water connections: [`Partition`], E14 T3d); step 3, the
//! terrain painted (the boxes, centroids and islands, and the game's terrain painter, E14 T3e).
//! The other steps come with increments 4 to 6, the writer's content with 7.
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

mod cells;
mod generate;
mod layout;
mod painter;
mod part;
mod partition;
mod settings;
mod template;
mod terrain;
mod triangulation;

pub use cells::{Cell, Cells};
pub use generate::{
    GenerateError, GeneratedMap, NoTrace, RMG_GENERATOR_VERSION, SITE_TEMPLATE, SITE_WATER,
    StepState, Trace, generate,
};
pub use layout::{
    SITE_ZONE_POSITION, SITE_ZONE_TERRAIN, SITE_ZONE_TERRAIN_TOWN, SITE_ZONE_TOWN_TYPE,
    ZoneGenerator,
};
pub use painter::{
    PaintFault, SITE_PICTURE, SITE_PICTURE_FLAG, SITE_PICTURE_SHAPE, SITE_ROCK_PICTURE,
    SITE_ROCK_PICTURE_SHAPE,
};
pub use part::{
    Connection, ConnectionOverride, GeneratorTables, LAYOUT_POINTS, MAX_PICTURES, MAX_SHAPE,
    NO_LIMIT, OBJECT_TYPES, ObjectSettings, ObjectTemplate, ObjectTemplateLine,
    ObjectTemplateOverride, Obstacle, PictureSet, RmgCompileError, RmgPack, RmgRuleset,
    RockPicture, Strength, TERRAIN_TOWNS, TERRAINS, Template, TypeLimits, Zone, ZoneKind,
};
pub use partition::{
    MemoryZone, Partition, PartitionFault, SITE_BORDER_JITTER, UNREACHED, ZoneMemory,
};
pub use settings::{
    COLOURS, Choices, MapFormat, MapSize, Monsters, Settings, SettingsError, TOWN_TYPES, Water,
    resolve_random,
};
pub use template::{Fit, KeptConnection, KeptTemplate, PositionError, kept_templates, positions};
pub use terrain::{SITE_COAST_JITTER, TerrainFault};

/// The original's `rand` (`h3_kernel::rng::CrtRand`), the stream [`resolve_random`] draws from.
pub use h3_kernel::rng::CrtRand;

/// Version of [`RmgRuleset`]'s canonical hash (ADR-0029 point 7): bumped whenever the bytes the
/// part hashes change for the same tables (2: the generator's tables, E14 T2c; 3: step 1's
/// tables, E14 T3c; 4: the terrain painter's picture sets, E14 T3e; 5: the object templates'
/// `.msk` footprints and colour masks, the object settings, E14 T4b).
pub const RMG_RULESET_HASH_VERSION: u32 = 5;

#[cfg(test)]
mod tests;
