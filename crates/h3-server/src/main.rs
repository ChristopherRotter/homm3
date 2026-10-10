//! `h3-server`: hosts the engine's games for untrusted clients over WebSocket (E16, ADR-0015). The
//! I/O shell around `h3-protocol`'s sans-I/O session core: sockets, clock, OS randomness, files,
//! configuration and the bot seats.
//!
//! Skeleton (E16 M0): the binary names its versions and exits; hosting arrives with M1 T3
//! (`docs/lanes/server/QUEUE.md`).

use std::process::ExitCode;

fn main() -> ExitCode {
    // stub: SRV-STUB-1
    eprintln!(
        "h3-server {} (protocol {}): hosting is not implemented yet (E16 M1)",
        env!("CARGO_PKG_VERSION"),
        h3_protocol::PROTOCOL_VERSION
    );
    ExitCode::FAILURE
}
