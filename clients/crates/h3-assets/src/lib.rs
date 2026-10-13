//! Decoders for the original game's assets (E17, ADR-0020 point 2): bytes in, typed data out. The
//! LOD archives themselves are read by the engine's `h3-formats`; this crate decodes their entries.
//!
//! Pure: no I/O, clock, threads or globals, `no_std` + `alloc`. Every decoder treats its input as
//! untrusted bytes and never panics, whatever the input (`clippy::indexing_slicing` is denied on
//! top of the engine's logic header; *provisional*). The header below is checked by
//! `h3-view`'s `tests/purity.rs`.
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
    clippy::exit,
    clippy::indexing_slicing
)]
#![warn(missing_docs)]

extern crate alloc;

pub mod archive;
pub mod build;
pub mod check;
pub mod def;
pub mod msk;
pub mod palette;
pub mod pcx;
mod read;
pub mod survey;

/// Why a decoder rejected its input. The decoders never panic: every malformed input ends in one
/// of these.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The bytes end before a field, a table or a run of pixels the format needs.
    Truncated,
    /// The bytes go on after the format's content (formats with an exact length only).
    TrailingBytes,
    /// A magic value or a chunk identifier is not the format's.
    Magic,
    /// A field holds a value the decoder does not accept; names the field.
    Field(&'static str),
    /// A run of pixels goes past the end of its row or block (DEF).
    Overrun,
    /// The image is larger than the decoder accepts ([`def::MAX_SIDE`], [`def::MAX_PIXELS`]).
    TooLarge,
}
