//! The starting resources by the game's difficulty (GAM-RES-002, E08 T13a; ADR-0017 point 1): a
//! row per difficulty from 0 (easy), each with a human and a computer player's treasury. The game
//! (`h3-game`) keeps the treasuries and reads the rows; they are data of the packs'
//! `adventure.toml` and compile here with the rest of the adventure part, beside the starting
//! army's chances (GAM-SET-001), so a game's rules stay one core ruleset and one part. Exactly
//! one enabled pack provides the rows, as the parameters.

use alloc::vec;
use alloc::vec::Vec;

use h3_content::ContentKey;
use h3_kernel::{CanonicalHash, StateHasher};

use super::AdventureCompileError;
use super::objects::{MAX_GAIN, Scope};

/// The most difficulty rows: a difficulty is a `u8`.
pub const MAX_DIFFICULTIES: usize = 256;

/// One difficulty's starting resources as a pack defines them: amounts by resource key; a
/// resource left out starts at 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartingResourcesDef {
    /// A human player's.
    pub human: Vec<(ContentKey, u32)>,
    /// A computer player's (recorded for a later human/computer distinction; survey
    /// `E08-objects.md` §7.2).
    pub computer: Vec<(ContentKey, u32)>,
}

/// One difficulty's compiled starting resources: an amount per core resource, in the core's
/// order, each at most [`MAX_GAIN`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartingResources {
    human: Vec<u32>,
    computer: Vec<u32>,
}

impl StartingResources {
    /// A human player's treasury at the start, by resource (every player's in the engine, which
    /// has no human/computer distinction: E08 decision 16).
    pub fn human(&self) -> &[u32] {
        &self.human
    }

    /// A computer player's treasury at the start, by resource.
    pub fn computer(&self) -> &[u32] {
        &self.computer
    }
}

impl CanonicalHash for StartingResources {
    fn hash_into(&self, h: &mut StateHasher) {
        self.human.hash_into(h);
        self.computer.hash_into(h);
    }
}

/// Compiles one pack's rows.
pub(super) fn compile_starting(
    scope: &Scope<'_>,
    defs: &[StartingResourcesDef],
) -> Result<Vec<StartingResources>, AdventureCompileError> {
    if defs.len() > MAX_DIFFICULTIES {
        return Err(AdventureCompileError::TooManyEntries("difficulty"));
    }
    defs.iter()
        .enumerate()
        .map(|(row, def)| {
            Ok(StartingResources {
                human: amounts(scope, row, &def.human)?,
                computer: amounts(scope, row, &def.computer)?,
            })
        })
        .collect()
}

/// A row's amounts by resource: each named once, at most [`MAX_GAIN`].
fn amounts(
    scope: &Scope<'_>,
    row: usize,
    list: &[(ContentKey, u32)],
) -> Result<Vec<u32>, AdventureCompileError> {
    let invalid = |reason| AdventureCompileError::InvalidDifficulty { row, reason };
    let mut out: Vec<Option<u32>> = vec![None; scope.core.resources().len()];
    for (key, amount) in list {
        let resource = scope.resource(key)?;
        if *amount > MAX_GAIN {
            return Err(invalid("an amount is above 2^31 - 1"));
        }
        let slot = &mut out[usize::from(resource.raw())];
        if slot.replace(*amount).is_some() {
            return Err(invalid("a resource is named twice"));
        }
    }
    Ok(out.into_iter().map(|a| a.unwrap_or(0)).collect())
}
