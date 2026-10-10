//! The server's published language and session core (E16, ADR-0015): the wire messages, their
//! bounded codec and the mapping to and from the engine's types, and the sans-I/O session core
//! that hosts one game for its seats. Nothing here touches a socket, a clock or a file: the host
//! (`h3-server`) passes bytes and the time in and sends what comes out.
//!
//! Skeleton (E16 M0): the messages and the codec arrive with M1 T1, the session core with M1 T2
//! (`docs/lanes/server/QUEUE.md`).
//!
//! Logic crate in the adapter layer (ADR-0015 point 1): pure, deterministic, `no_std` + `alloc`
//! (ADR-0003, ADR-0005). The lint header below is required and checked by `cargo xtask check-arch`.
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

/// The version of the wire protocol: sent in the hello and bumped on every change of the messages
/// or their encoding (ADR-0015 point 4). Before 1.0 a server accepts exactly its own version.
pub const PROTOCOL_VERSION: u32 = 0;
