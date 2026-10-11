//! The server's published language and session core (E16, ADR-0018): the wire messages, their
//! bounded codec and the mapping to and from the engine's types, and the sans-I/O session core
//! that hosts one game for its seats. Nothing here touches a socket, a clock or a file: the host
//! (`h3-server`) passes bytes and the time in and sends what comes out.
//!
//! - [`wire`]: the published language, every message in both directions (M1 T1);
//! - [`codec`]: one JSON document per frame, bounded and panic-free (M1 T1);
//! - [`map`]: the mapping to and from the battle's commands, events and observation, which
//!   projects everything outbound for its viewer (M1 T1);
//! - [`connection`]: a connection's own messages, the hello and the ping (M1 T2);
//! - [`session`]: the session core, one hosted battle: seats, tokens, the authoritative loop, the
//!   bot hook, the viewers' records and the journal (M1 T2);
//! - [`lobby`]: the server's offers and games, created within limits and listed (M2 T1).
//!
//! Logic crate in the adapter layer (ADR-0018 point 1): pure, deterministic, `no_std` + `alloc`
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

extern crate alloc;
#[cfg(test)]
extern crate std;

pub mod codec;
pub mod connection;
pub mod lobby;
pub mod map;
pub mod session;
pub mod wire;

/// The version of the wire protocol: sent in the hello and bumped on every change of the messages
/// or their encoding (ADR-0018 point 4). Before 1.0 a server accepts exactly its own version.
pub const PROTOCOL_VERSION: u32 = 4;

/// The version of the record's format (`wire::RecordHeader`, the `Record` message): bumped when
/// its layout changes.
pub const RECORD_FORMAT_VERSION: u32 = 1;

#[cfg(test)]
mod tests;
