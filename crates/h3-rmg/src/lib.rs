//! The map generation context (ADR-0029, E14): a port of the original's random map generator
//! (`Heroes3.exe`'s, SoD's code), so that the same template table, choices and seed give the same
//! map as the original, byte for byte once written. One pure entry, `generate` (ADR-0029 point
//! 2), returns a generated map that `h3_map`'s writer serialises; the generator writes no file.
//! Its stream is the C runtime's `rand`, `h3_kernel::rng::CrtRand` (point 3); its floats are
//! binary64 through `h3_kernel::num` (point 4); its tables are the rmg part, compiled from the
//! packs' `rmg.toml` by `h3-pack-rmg` (point 5).
//!
//! So far: the rmg part ([`RmgRuleset`], E14 T2b): the three tables compiled from the packs.
//! The settings, the template's choice and `generate`'s frame come with T2c.
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

mod part;

pub use part::{
    Connection, ConnectionOverride, ObjectTemplate, ObjectTemplateLine, ObjectTemplateOverride,
    Obstacle, RmgCompileError, RmgPack, RmgRuleset, Strength, Template, Zone, ZoneKind,
};

/// Version of [`RmgRuleset`]'s canonical hash (ADR-0029 point 7): bumped whenever the bytes the
/// part hashes change for the same tables.
pub const RMG_RULESET_HASH_VERSION: u32 = 1;
