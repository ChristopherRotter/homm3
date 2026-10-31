//! Hero details (ADR-0013, E04a T4): the lasting details of a hero — level, experience, primary and
//! secondary skills, artifacts, spells, mana — their validation against a ruleset, what the hero
//! has from them in every context (its available spells and their masteries, E04b T8a), and the
//! pure translations to what a battle needs, and the typed changes the game applies to its store of
//! heroes (E08 T19a2a: a battle's outcome; T19a2b: a hero's Necromancy, the spell level it may
//! learn and the Eagle Eye picks it learns), and what the adventure map reads of a hero (E08 T20a1:
//! its Attack and Defense, its join bonus), and since E08 T27 its progression: the levels of its
//! experience, the scenario's level limit, a level-up's draws and the answer as a change (ADR-0013
//! point 12), and since E08 T28 the original's choice between two offers, and since E08 T39 what
//! the map reads to move and scout (the day's land points, a creature's speed, the Pathfinding,
//! flying and water-walking levels, the scouting radius; ADR-0013 point 13), and since E08 T49a
//! the worn artifacts by position (`positions`, the first position that takes one, `wearing`;
//! ADR-0013 point 1), and since E08 T49b an artifact's moves, the combinations' assembly and
//! disassembly, Titan's Thunder's Spell Book and Scholar at a meeting (ADR-0013 point 14), and
//! since E08 T60a the battle bonuses until the next battle's end and the rewards' changes: primary
//! skills gained, a skill raised to a level, a bonus with its once-mark, the bonuses cleared
//! (ADR-0013 points 1, 2), and since E08 T60b a reward record's hero half (`grant`), whether a hero
//! can receive an artifact (A-25) and what a teacher may teach (ADR-0013 point 15), and since E08
//! T82 what a hero brings to the day's start: its income and its day's spell points (ADR-0013
//! point 16) and the Lighthouses' input to its sea points (point 13.6). No commands
//! and no map or battle state: who levels a hero up, when, and equipping and learning arrive with the
//! adventure and game epics (E08–E10); the battle never depends on this crate and reads only the
//! battle hero of the seam (`h3_contracts::BattleHero`).
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
mod combine;
mod day;
mod derive;
mod details;
mod error;
mod level;
mod level_up;
mod movement;
mod moves;
mod positions;
mod reward;
mod spells;
mod teach;

pub use change::{
    ArtifactPlace, HeroChange, apply_change, apply_change_limited, check_change, lost_war_machine,
    next_capture,
};
pub use choice::{ArmyWorth, TerrainShares, original_choice};
pub use combine::{assemble_changes, check_assemble, check_disassemble, disassemble_changes};
pub use day::{daily_income, day_mana};
pub use derive::{
    MANA_PER_KNOWLEDGE, MAX_JOIN_BONUS, Necromancy, attack_defense, derive_battle_hero,
    gained_experience, join_bonus, max_mana, necromancy,
};
pub use details::{
    ArtifactInstance, BattleBonuses, HeroDetails, OfferLevels, PrimarySkill, validate,
    validate_artifact, war_machines,
};
pub use error::{ChangeError, CombineError, DetailsError, MoveError};
pub use level::{
    experience_bound, experience_cap, experience_for, gained_under_limit, level_of, pending_levels,
};
pub use level_up::{Choice, Draws, LevelUp, Offers, OriginalDraws, draw_level_up};
pub use movement::{
    Movement, free_boarding, land_points, movement, movement_speed, scouting_radius, sea_points,
};
pub use moves::{ArtifactMove, MoveChanges, check_move, move_changes};
pub use positions::{Position, equip, first_position, positions, takes, wearing};
pub use reward::{Granted, HeroReward, can_receive_artifact, grant};
pub use spells::{
    available_spells, can_learn_spell, learn_level, learns_spell, meeting_spells, spell_masteries,
};
pub use teach::{
    ScholarGift, SlotState, hut_teaches, learning_experience, library_admits,
    next_level_experience, scholar_reward, university_slot,
};

#[cfg(test)]
mod tests;
