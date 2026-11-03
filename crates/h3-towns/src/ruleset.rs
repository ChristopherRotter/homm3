//! The town ruleset part (ADR-0028 point 5): compiled on the core as the adventure's and the
//! battle's parts are (ADR-0015 point 2), from the enabled packs' `towns.toml` (parsed by
//! `h3-pack-towns`).

/// The town context's part of a game's rules: per faction a town record (its buildings with the
/// hall and fort level each gives and what it supersedes, the silo's resources, the moat and the
/// siege's natural obstacles) and the part's parameters (the gold by hall level and the Grail's,
/// the neutral garrison's odds, the standard set's level-2 dwelling chance, the hall a second
/// Capitol becomes); its own canonical hash ([`crate::TOWN_RULESET_HASH_VERSION`]).
///
/// Empty so far (E09 T4b): T5a gives it the records, the parameters, the compiler and the hash
/// (E09 increment 1's design note §2).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TownRuleset {}
