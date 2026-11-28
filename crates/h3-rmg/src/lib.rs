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
//! terrain painted (the boxes, centroids and islands, and the game's terrain painter, E14 T3e);
//! the object model the steps from 4 place with ([`Objects`]: the prototypes, the placed objects
//! and the cells' lists, the entrances' distance flood, the placing test; E14 T4b); steps 4 and
//! 5, the towns ([`Placed`], E14 T4b); step 6's obstacle plan (the lanes, the zone walls, the
//! cells of no zone, the islets, the zones' free cells joined, the water shores sealed; E14 T4c);
//! step 6's connections by land with their guard monsters ([`Connected`], [`Monster`], E14 T4d1).
//! Step 6's other ways and the other steps come with increments 4 to 6, the writer's content
//! with 7.
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
mod guards;
mod layout;
mod objects;
mod painter;
mod part;
mod partition;
mod plan;
mod settings;
mod template;
mod terrain;
mod towns;
mod triangulation;

pub use cells::{Cell, Cells};
pub use generate::{
    GenerateError, GeneratedMap, NoTrace, RMG_GENERATOR_VERSION, SITE_TEMPLATE, SITE_WATER,
    StepState, Trace, generate,
};
pub use guards::{
    GUARD_CREATURES, GuardCreature, Monster, SITE_GUARD_COUNT_MINUS, SITE_GUARD_COUNT_PLUS,
    SITE_GUARD_CREATURE, guard_creatures,
};
pub use layout::{
    SITE_ZONE_POSITION, SITE_ZONE_TERRAIN, SITE_ZONE_TERRAIN_TOWN, SITE_ZONE_TOWN_TYPE,
    ZoneGenerator,
};
pub use objects::{NO_ENTRANCE, Object, ObjectFault, Objects, PlaceZone, Prototype, Shape};
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
pub use plan::{
    Connected, SITE_ISLET_CENTRE, SITE_ISLET_HEIGHTS, SITE_ISLET_RADIUS, SITE_ISLET_TERRAIN,
    SITE_LANE_JITTER, SITE_LANE_ORIENTATION, SITE_PASSAGE_CELL, SITE_PASSAGE_GUARD_SIDE,
};
pub use settings::{
    COLOURS, Choices, MapFormat, MapSize, Monsters, Settings, SettingsError, TOWN_TYPES, Water,
    resolve_random,
};
pub use template::{Fit, KeptConnection, KeptTemplate, PositionError, kept_templates, positions};
pub use terrain::{SITE_COAST_JITTER, TerrainFault};
pub use towns::{
    Placed, SITE_TOWN_CENTRAL_CELL, SITE_TOWN_FAR_CELL, SITE_TOWN_TYPE_ANY, SITE_TOWN_TYPE_ZONE,
    Town,
};

/// The original's `rand` (`h3_kernel::rng::CrtRand`), the stream [`resolve_random`] draws from.
pub use h3_kernel::rng::CrtRand;

/// Version of [`RmgRuleset`]'s canonical hash (ADR-0029 point 7): bumped whenever the bytes the
/// part hashes change for the same tables (2: the generator's tables, E14 T2c; 3: step 1's
/// tables, E14 T3c; 4: the terrain painter's picture sets, E14 T3e; 5: the object templates'
/// `.msk` footprints and colour masks, the object settings, E14 T4b).
pub const RMG_RULESET_HASH_VERSION: u32 = 5;

#[cfg(test)]
mod tests;
