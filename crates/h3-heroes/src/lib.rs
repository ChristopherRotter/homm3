//! Hero details (ADR-0013, E04a T4): the lasting details of a hero — level, experience, primary
//! and secondary skills, artifacts, spells, mana — their validation against a ruleset, what the
//! hero has from them in every context (its available spells and their masteries, E04b T8a), and
//! the pure translations to what a battle needs. No commands and no map or battle state: equipping,
//! learning and levelling up arrive with the adventure and game epics (E08–E10); the battle never
//! depends on this crate and reads only the battle hero of the seam (`h3_contracts::BattleHero`).
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

mod derive;
mod details;
mod error;
mod spells;

pub use derive::{MANA_PER_KNOWLEDGE, derive_battle_hero, max_mana};
pub use details::{ArtifactInstance, HeroDetails, PrimarySkill, validate, war_machines};
pub use error::DetailsError;
pub use spells::{available_spells, spell_masteries};

#[cfg(test)]
mod tests;
