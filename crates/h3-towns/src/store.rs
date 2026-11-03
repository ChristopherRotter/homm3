//! The store of towns (ADR-0028 point 4): every town's record by `h3_contracts::TownId`, held by
//! the game state beside the adventure's.

/// Every town's record by `TownId`, shared between forks and copied on its first write, as the
/// store of heroes (ADR-0017 point 1); nothing sees the sharing, and its canonical hash
/// ([`crate::TOWN_STATE_HASH_VERSION`]) folds in each town in `TownId` order. A map without towns
/// has an empty store.
///
/// Empty so far (E09 T4b): T6 gives it the records, their reads and writes and the hash (E09
/// increment 1's design note §4).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Towns {}
