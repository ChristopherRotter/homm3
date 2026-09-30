//! Errors of the ruleset compiler.

use alloc::string::String;
use core::fmt;

/// Why a set of content packs could not be compiled into a [`crate::Ruleset`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    /// A pack id is not a valid key part.
    InvalidPackId(String),
    /// Two packs share an id.
    DuplicatePack(String),
    /// A pack depends on a pack that is not loaded before it.
    MissingDependency {
        /// The depending pack.
        pack: String,
        /// The missing dependency.
        depends: String,
    },
    /// A pack defines content outside its own namespace.
    KeyOutsidePack {
        /// The offending key.
        key: String,
        /// The pack defining it.
        pack: String,
    },
    /// Two definitions share a key.
    DuplicateKey(String),
    /// A creature definition is invalid.
    InvalidCreature {
        /// The creature's key.
        key: String,
        /// What is wrong.
        reason: &'static str,
    },
    /// More creature types than handles can address.
    TooManyCreatures,
    /// No pack provides battle parameters.
    MissingBattleParams,
    /// More than one pack provides battle parameters.
    DuplicateBattleParams(String),
    /// The battle parameters are invalid.
    InvalidBattleParams(&'static str),
    /// The selected profile does not exist.
    UnknownProfile(String),
    /// A profile sets a quirk flag the engine does not know.
    UnknownQuirk {
        /// The profile.
        profile: String,
        /// The flag name.
        flag: String,
    },
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPackId(id) => write!(f, "invalid pack id `{id}`"),
            Self::DuplicatePack(id) => write!(f, "pack `{id}` is loaded twice"),
            Self::MissingDependency { pack, depends } => {
                write!(
                    f,
                    "pack `{pack}` depends on `{depends}`, which is not loaded before it"
                )
            }
            Self::KeyOutsidePack { key, pack } => {
                write!(f, "pack `{pack}` defines `{key}` outside its namespace")
            }
            Self::DuplicateKey(key) => write!(f, "`{key}` is defined twice"),
            Self::InvalidCreature { key, reason } => write!(f, "creature `{key}`: {reason}"),
            Self::TooManyCreatures => f.write_str("too many creature types"),
            Self::MissingBattleParams => f.write_str("no pack provides battle parameters"),
            Self::DuplicateBattleParams(pack) => {
                write!(f, "pack `{pack}` provides battle parameters a second time")
            }
            Self::InvalidBattleParams(reason) => write!(f, "battle parameters: {reason}"),
            Self::UnknownProfile(key) => write!(f, "unknown profile `{key}`"),
            Self::UnknownQuirk { profile, flag } => {
                write!(f, "profile `{profile}` sets unknown quirk flag `{flag}`")
            }
        }
    }
}

impl core::error::Error for CompileError {}
