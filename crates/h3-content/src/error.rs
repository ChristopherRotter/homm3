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
    /// A pack depends on a pack that is not enabled.
    MissingDependency {
        /// The depending pack.
        pack: String,
        /// The missing dependency.
        depends: String,
    },
    /// The dependencies of the enabled packs form a cycle; the payload is one pack on it.
    DependencyCycle(String),
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
    /// More entries of a kind (`resource`, `faction`) than handles can address.
    TooManyEntries(&'static str),
    /// A reference (faction, resource, override target) names a key that is not defined.
    UnknownReference {
        /// What kind of entity was expected.
        kind: &'static str,
        /// The key.
        key: String,
    },
    /// A pack refers to (or overrides) content of a pack outside its dependency closure.
    ReferenceOutsideDependencies {
        /// The referenced key.
        key: String,
        /// The referring pack.
        pack: String,
    },
    /// A creature names a flag the engine does not know.
    UnknownFlag(String),
    /// No pack provides battle parameters.
    MissingBattleParams,
    /// More than one pack provides battle parameters.
    DuplicateBattleParams(String),
    /// The battle parameters are invalid.
    InvalidBattleParams(&'static str),
    /// An obstacle type is invalid.
    InvalidObstacle {
        /// The obstacle's key.
        key: String,
        /// What is wrong.
        reason: &'static str,
    },
    /// A spell, secondary skill, hero class or hero type is invalid (E04a T3).
    InvalidContent {
        /// What kind of entry.
        kind: &'static str,
        /// Its key.
        key: String,
        /// What is wrong.
        reason: &'static str,
    },
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
                    "pack `{pack}` depends on `{depends}`, which is not enabled"
                )
            }
            Self::DependencyCycle(pack) => {
                write!(f, "pack dependencies form a cycle through `{pack}`")
            }
            Self::KeyOutsidePack { key, pack } => {
                write!(f, "pack `{pack}` defines `{key}` outside its namespace")
            }
            Self::DuplicateKey(key) => write!(f, "`{key}` is defined twice"),
            Self::InvalidCreature { key, reason } => write!(f, "creature `{key}`: {reason}"),
            Self::TooManyCreatures => f.write_str("too many creature types"),
            Self::TooManyEntries(kind) => write!(f, "too many {kind} entries"),
            Self::UnknownReference { kind, key } => write!(f, "unknown {kind} `{key}`"),
            Self::ReferenceOutsideDependencies { key, pack } => write!(
                f,
                "pack `{pack}` refers to `{key}`, which is not in its dependencies"
            ),
            Self::UnknownFlag(name) => write!(f, "unknown creature flag `{name}`"),
            Self::MissingBattleParams => f.write_str("no pack provides battle parameters"),
            Self::DuplicateBattleParams(pack) => {
                write!(f, "pack `{pack}` provides battle parameters a second time")
            }
            Self::InvalidBattleParams(reason) => write!(f, "battle parameters: {reason}"),
            Self::InvalidObstacle { key, reason } => write!(f, "obstacle `{key}`: {reason}"),
            Self::InvalidContent { kind, key, reason } => write!(f, "{kind} `{key}`: {reason}"),
            Self::UnknownProfile(key) => write!(f, "unknown profile `{key}`"),
            Self::UnknownQuirk { profile, flag } => {
                write!(f, "profile `{profile}` sets unknown quirk flag `{flag}`")
            }
        }
    }
}

impl core::error::Error for CompileError {}
