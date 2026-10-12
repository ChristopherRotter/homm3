//! Decoders for the original game's assets (E17, ADR draft `docs/lanes/client/adr-drafts/client.md`
//! point 2): bytes in, typed data out. The LOD archives themselves are read by the engine's
//! `h3-formats`; this crate decodes their entries.
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

pub mod def;
pub mod msk;
pub mod palette;
pub mod pcx;
