//! The original's battle AI (ADR-0027, E23): the policy of a battle side that the original's
//! computer plays, one pure, deterministic function of the battle's state, its ruleset, the
//! side and the caller's stream. It reads the computer's numbers the battle's rules use
//! through `h3-battle`'s public `computer` module (ADR-0027 point 6). The crate is registered
//! here (the workspace, `arch.toml`, `lanes.toml`); its policy comes with the battle lane's E23
//! T2a.
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
