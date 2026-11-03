//! The town context (ADR-0028, E09): every town's own record (its faction, the buildings built and
//! forbidden, its map events, its name), the store of towns by `h3_contracts::TownId`, the rules
//! and derivations over a town (the hall and the day's income, the fort level, the siege's facts,
//! the reveal radius, the Tavern, the Capitol's limit), its ruleset part and that part's compiler.
//! The town on the map (its tiles and entrance, owner, garrison and visiting hero) is the
//! adventure's; `h3-game` composes the two, and no domain crate depends on this one (ADR-0028
//! points 1, 2 and 6).
//!
//! So far a skeleton (E09 T4b): the empty ruleset part ([`TownRuleset`]), the empty store
//! ([`Towns`]) and the two version constants. E09 increment 1's design note §2 (the part, T5a) and
//! §4 to §6 (the record, the store, the setup, the derivations, T6) say what comes.
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

mod ruleset;
mod store;

pub use ruleset::TownRuleset;
pub use store::Towns;

/// Version of the store of towns' canonical hash encoding, which the game state hash folds in
/// (ADR-0028 points 4 and 9; `docs/08-work-process.md`, "Versions"; owned by the town lane). Bump
/// it with any change to what the hash covers. v1 (E09 T4b): the constant, before any encoding;
/// T6 writes the first (the town count, each town's faction, built and forbidden buildings, then
/// the static part, in `TownId` order; the design note §4).
pub const TOWN_STATE_HASH_VERSION: u32 = 1;

/// Version of the town ruleset part's canonical hash encoding (ADR-0028 points 5 and 9; owned by
/// the town lane). v1 (E09 T4b): the constant, before any encoding; T5a writes the first (the
/// factions' town records and the part's parameters; the design note §2).
pub const TOWN_RULESET_HASH_VERSION: u32 = 1;
