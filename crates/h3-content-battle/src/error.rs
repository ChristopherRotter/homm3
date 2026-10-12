//! Errors of the battle part's compiler.

use alloc::string::String;
use core::fmt;

use h3_content::CompileError;

/// Why the battle packs could not be compiled into a [`crate::BattleRuleset`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BattleCompileError {
    /// A core pack has no battle pack.
    MissingPack(String),
    /// A battle pack's manifest names no pack of the core ruleset (id and version).
    UnknownPack(String),
    /// Two battle packs belong to one core pack.
    DuplicatePack(String),
    /// A pack's dependency does not load before it.
    UnknownDependency {
        /// The depending pack.
        pack: String,
        /// The dependency.
        dependency: String,
    },
    /// A core creature has no battle row.
    MissingCreature(String),
    /// A core spell that is not an adventure spell has no battle row.
    MissingSpell(String),
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
    /// An error the core compiler also knows, raised by the battle part: a key outside its
    /// pack or defined twice, an unknown reference or one outside the dependencies, too many
    /// entries, an invalid creature or spell, an override's gate that is no data quirk.
    Content(CompileError),
}

impl From<CompileError> for BattleCompileError {
    fn from(e: CompileError) -> Self {
        Self::Content(e)
    }
}

impl fmt::Display for BattleCompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPack(p) => write!(f, "pack `{p}` has no battle part"),
            Self::UnknownPack(p) => write!(f, "battle part of `{p}`, which the core does not load"),
            Self::DuplicatePack(p) => write!(f, "pack `{p}` has two battle parts"),
            Self::UnknownDependency { pack, dependency } => write!(
                f,
                "pack `{pack}` depends on `{dependency}`, which does not load before it"
            ),
            Self::MissingCreature(key) => write!(f, "creature `{key}` has no battle row"),
            Self::MissingSpell(key) => write!(f, "spell `{key}` has no battle row"),
            Self::MissingBattleParams => f.write_str("no pack provides battle parameters"),
            Self::DuplicateBattleParams(pack) => {
                write!(f, "pack `{pack}` provides battle parameters a second time")
            }
            Self::InvalidBattleParams(reason) => write!(f, "battle parameters: {reason}"),
            Self::InvalidObstacle { key, reason } => write!(f, "obstacle `{key}`: {reason}"),
            Self::Content(e) => e.fmt(f),
        }
    }
}

impl core::error::Error for BattleCompileError {}
