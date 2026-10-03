//! Hero details (ADR-0013, E04a T4): the lasting details of a hero — level, experience, primary
//! and secondary skills, artifacts, spells, mana — their validation against a ruleset, and the
//! pure translations to what a battle needs. No commands and no map or battle state: equipping,
//! learning and levelling up arrive with the adventure and game epics (E08–E10); the battle never
//! depends on this crate and reads only the battle hero of the seam (`h3-contracts`, T4b).
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

mod details;
mod error;

pub use details::{HeroDetails, PrimarySkill, validate, war_machines};
pub use error::DetailsError;

#[cfg(test)]
mod tests;
