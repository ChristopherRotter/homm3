//! The town context (ADR-0028, E09): every town's own record (its faction, the buildings built and
//! forbidden, its map events, its name), its setup at the game's build, the store of towns by
//! `h3_contracts::TownId`, the rules and derivations over a town (the hall and the day's income,
//! the fort level, the siege's facts, the reveal radius, the Tavern, the Capitol's limit), its
//! ruleset part and that part's compiler.
//! The town on the map (its tiles and entrance, owner, garrison and visiting hero) is the
//! adventure's; `h3-game` composes the two, and no domain crate depends on this one (ADR-0028
//! points 1, 2 and 6).
//!
//! So far: the ruleset part ([`TownRuleset`], its compiler and canonical hash; E09 T5a1), the
//! buildings' ids and sets, a town's record ([`Town`], [`TownStatic`]) and the store of towns
//! ([`Towns`]) with its canonical hash (E09 T6a), the derivations over a town and the part (E09
//! T6b): the hall and the day's income ([`hall_level`], [`daily_income`]), the siege's facts
//! ([`fort_level`], [`siege`], [`siege_obstacles`], [`defender_bonus`]), the reveal, the Tavern
//! and the Capitol's limit with the store's first write ([`reveal`], [`has_tavern`],
//! [`Towns::apply_capitol_limit`]), and the two version constants; the town's setup at the
//! game's build (E09 T6c): what the map gives ([`TownSetup`]) and the functions the game calls
//! with the setup stream ([`town_buildings_setup`], [`town_garrison`], [`town_setup`]). What
//! comes next: the map's import of the towns and the game's build that calls the setup (E09 T7a,
//! T7b2; the design note §13, §14).
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

mod building;
mod capture;
mod derive;
#[cfg(test)]
mod fixture;
mod income;
mod record;
mod ruleset;
mod setup;
mod siege;
mod store;
mod view;

pub use building::{BuildingId, BuildingSet, MAX_BUILDINGS};
pub use capture::{capitol_limit, reveal};
/// The moat's kind a town record gives (the battle's contract type, ADR-0023 point 2).
pub use h3_contracts::Moat;
pub use income::{daily_income, hall_level};
pub use record::{Town, TownEvent, TownStatic};
pub use ruleset::{
    BUILDING_SLOTS, Building, BuildingDef, BuildingRoles, CreatureLevel, CreatureLevelDef,
    DefenderBonus, Dwelling, GarrisonRow, MAX_CREATURE_LEVELS, Reveal, TownCompileError, TownDef,
    TownOverride, TownPack, TownParams, TownParamsDef, TownPatch, TownRecord, TownRuleset,
};
pub use setup::{
    BuildingsSetup, BuiltTown, GarrisonCreature, GarrisonStack, SlotSet, TownEventSetup,
    TownFaction, TownSetup, TownSetupError, town_buildings_setup, town_garrison, town_setup,
};
pub use siege::{defender_bonus, fort_level, siege, siege_obstacles};
pub use store::{MAX_TOWNS, Towns};
pub use view::has_tavern;

/// Version of the store of towns' canonical hash encoding, which the game state hash folds in
/// (ADR-0028 points 4 and 9; `docs/08-work-process.md`, "Versions"; owned by the town lane). Bump
/// it with any change to what the hash covers. v1 (E09 T6a; the constant since T4b): the first
/// encoding, the town count, each town's faction, ever-built and forbidden buildings, then each
/// town's static part (its name, its events), in `TownId` order ([`Towns`]' `hash_into`).
pub const TOWN_STATE_HASH_VERSION: u32 = 1;

/// Version of the town ruleset part's canonical hash encoding (ADR-0028 points 5 and 9; owned by
/// the town lane). v1 (E09 T5a1; the constant since T4b): the first encoding, the parameters and
/// the factions' town records ([`TownRuleset`]'s `hash_into`). v2 (E09 T5b2a): each building's
/// automatic set (`auto_with`, the hordes' upgrade). v3 (E09 T8c2): Visions' factor, least
/// radius and creatures in the parameters (TWN-VEW-003). v4 (E09 T16a): each building's
/// requirements (the closure) and cost, and the role `shipyard` (E09 design §24).
pub const TOWN_RULESET_HASH_VERSION: u32 = 4;
