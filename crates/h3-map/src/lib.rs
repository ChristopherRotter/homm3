//! Adapter: the original's `.h3m` maps (ADR-0015 point 4, E08). *Parse* reads a map file's bytes
//! into a raw model faithful to its version (RoE, AB, SoD), within configured input bounds, with
//! typed errors and no panics; *import* maps the raw model onto content keys and handles and
//! builds `h3_game::GameSetup`, drawing nothing; `h3-game` and the contexts then validate it as
//! if commands had built it. Map files are untrusted input (`docs/06-security.md`): parse,
//! import and validation are property-tested together on arbitrary bytes (`cargo-fuzz` later).
//! Format notes: `docs/formats/h3m.md`. The [`writer`] writes the SoD subset that experiments in
//! the original game need (moved here from `h3-formats` with the content split, R01a).
//!
//! E08 T3: the reader ([`read`], [`parse`] into [`model::RawMap`]); T5a and T6a: the [`import`]
//! into `h3_game::GameSetup`.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod cursor;
mod error;
mod gzip;
mod import;
mod limits;
pub mod model;
mod parse;
pub mod writer;

#[cfg(test)]
mod tests;

pub use error::ReadError;
pub use gzip::{crc32, gunzip};
pub use import::{ImportError, Unsupported, import, import_map};
pub use limits::Limits;
pub use model::RawMap;
pub use parse::parse;

/// Version of the import from a map file into the game's setup (ADR-0015, ADV-Q2): a replay names
/// its map by content hash and path plus this version, and verifying it re-imports the map. Bump
/// it with any change that alters an imported setup (`docs/08-work-process.md`, "Versions").
/// v2 (E08 T12a): object types and the piles' fixed amounts; guarded piles unsupported. v3 (E08
/// T13a): the setup's difficulty, normal (the map holds none).
pub const MAP_IMPORT_VERSION: u32 = 3;

/// Reads a map file as the game stores it (gzip-compressed) within `limits`.
///
/// # Errors
///
/// [`ReadError`]: a file over [`Limits::max_file_bytes`], a broken gzip stream, a map over
/// [`Limits::max_map_bytes`], or any error of [`parse`].
pub fn read(file: &[u8], limits: &Limits) -> Result<RawMap, ReadError> {
    if file.len() > limits.max_file_bytes {
        return Err(ReadError::FileTooLarge {
            size: file.len(),
            max: limits.max_file_bytes,
        });
    }
    parse(&gunzip(file, limits.max_map_bytes)?, limits)
}
