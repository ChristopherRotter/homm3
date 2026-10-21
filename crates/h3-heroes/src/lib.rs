//! Hero details (ADR-0013, E04a T4): the lasting details of a hero — level, experience, primary and
//! secondary skills, artifacts, spells, mana — their validation against a ruleset, what the hero
//! has from them in every context (its available spells and their masteries, E04b T8a), and the
//! pure translations to what a battle needs, and the typed changes the game applies to its store of
//! heroes (E08 T19a2a: a battle's outcome; T19a2b: a hero's Necromancy, the spell level it may
//! learn and the Eagle Eye picks it learns), and what the adventure map reads of a hero (E08 T20a1:
//! its Attack and Defense, its join bonus), and since E08 T27 its progression: the levels of its
//! experience, the scenario's level limit, a level-up's draws and the answer as a change (ADR-0013
//! point 12), and since E08 T28 the original's choice between two offers. No commands and no map or
//! battle state: who levels a hero up, when, and equipping and learning arrive with the adventure
//! and game epics (E08–E10); the battle never depends on this crate and reads only the battle hero
//! of the seam (`h3_contracts::BattleHero`).
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

mod change;
mod choice;
mod derive;
mod details;
mod error;
mod level;
mod level_up;
mod spells;

pub use change::{
    ArtifactPlace, HeroChange, apply_change, apply_change_limited, check_change, lost_war_machine,
    next_capture,
};
pub use choice::{ArmyWorth, TerrainShares, original_choice};
pub use derive::{
    MANA_PER_KNOWLEDGE, MAX_JOIN_BONUS, Necromancy, attack_defense, derive_battle_hero,
    gained_experience, join_bonus, max_mana, necromancy,
};
pub use details::{
    ArtifactInstance, HeroDetails, OfferLevels, PrimarySkill, validate, war_machines,
};
pub use error::{ChangeError, DetailsError};
pub use level::{
    experience_bound, experience_cap, experience_for, gained_under_limit, level_of, pending_levels,
};
pub use level_up::{Choice, Draws, LevelUp, Offers, OriginalDraws, draw_level_up};
pub use spells::{available_spells, learn_level, learns_spell, spell_masteries};

#[cfg(test)]
mod tests;
