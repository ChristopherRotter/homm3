//! Combination artifacts (E08 T49b; ADR-0013 point 14.2, HER-ART-006): a worn part assembles its
//! combination when the hero wears every part; a worn combination disassembles into its parts.
//! Neither in the backpack (D-28, the original's). The combinations are the ruleset's: the
//! game's configuration decides which expansions it holds (ADR-0017 point 1).

use alloc::vec::Vec;

use h3_content::{ArtifactTypeId, Ruleset};

use crate::change::{ArtifactPlace, HeroChange};
use crate::details::{ArtifactInstance, HeroDetails};
use crate::error::CombineError;
use crate::positions::Position;

/// The combination the part worn at `at` assembles (HER-ART-006): the first of the ruleset's
/// combinations, in artifact order, that lists the part's type among its parts and every part of
/// which the hero wears on a position (each part's type once at least; a part inside a worn
/// combination does not count). The original's 0x4DBE80 (a part names one combination, setup
/// +0x18; the packs list each part in one).
///
/// # Errors
/// [`CombineError::NoArtifact`] for an empty position, [`CombineError::Incomplete`] when the
/// artifact there completes no combination.
pub fn check_assemble(
    details: &HeroDetails,
    ruleset: &Ruleset,
    at: Position,
) -> Result<ArtifactTypeId, CombineError> {
    // rule: HER-ART-006
    let part = worn_at(details, at)?;
    let wears = |id: &ArtifactTypeId| details.worn.iter().flatten().any(|i| i.artifact == *id);
    (0_u16..)
        .zip(ruleset.artifacts())
        .find(|(_, c)| c.parts.contains(&part.artifact) && c.parts.iter().all(wears))
        .map(|(raw, _)| ArtifactTypeId::from_raw(raw))
        .ok_or(CombineError::Incomplete(at))
}

/// The hero changes that assemble the combination of the part at `at` ([`check_assemble`]): the
/// part there leaves, then one copy of each other part, taken from the positions in descending
/// order; the combination is received last, at its first position (HER-CHG-002: the parts'
/// positions and locks make room; equipped, HER-ART-007: Titan's Thunder adds a Spell Book).
/// The original's `hero_build_combination` (0x4DBF30) walks the positions 18 down to 0; the
/// order within a slot is the same here (no combination has parts among the machines or the
/// book, which the two orders place apart).
///
/// # Errors
/// [`check_assemble`]'s.
pub fn assemble_changes(
    details: &HeroDetails,
    ruleset: &Ruleset,
    at: Position,
) -> Result<Vec<HeroChange>, CombineError> {
    let combination = check_assemble(details, ruleset, at)?;
    let parts = ruleset
        .artifact(combination)
        .map(|c| c.parts.as_slice())
        .unwrap_or_default();
    // Which positions give a part: `at`'s for one copy of its type, then for each other part
    // the highest position of its type not taken yet.
    let kind = |i: usize| details.worn.get(i).copied().flatten().map(|i| i.artifact);
    let mut own = kind(at.0);
    let mut taken = Vec::from([at.0]);
    for &part in parts {
        if own == Some(part) {
            own = None;
            continue;
        }
        let highest = (0..details.worn.len())
            .rev()
            .find(|&i| !taken.contains(&i) && kind(i) == Some(part));
        taken.extend(highest);
    }
    let rest = taken.split_off(1);
    let descending = (0..details.worn.len()).rev().filter(|i| rest.contains(i));
    Ok(taken
        .into_iter()
        .chain(descending)
        .map(|i| HeroChange::RemoveArtifact(ArtifactPlace::Worn(Position(i))))
        .chain(core::iter::once(HeroChange::AddArtifact(
            combination.into(),
        )))
        .collect())
}

/// The combination worn at `at`, if it disassembles (HER-ART-006): a worn combination does; one in
/// the backpack never (D-28: the original shows its window; VCMI disassembles it there too).
///
/// # Errors
/// [`CombineError::NoArtifact`] for an empty position, [`CombineError::NotCombination`] for an
/// artifact that is no combination.
pub fn check_disassemble(
    details: &HeroDetails,
    ruleset: &Ruleset,
    at: Position,
) -> Result<ArtifactTypeId, CombineError> {
    // rule: HER-ART-006
    let item = worn_at(details, at)?;
    ruleset
        .artifact(item.artifact)
        .filter(|a| a.is_combination())
        .map(|_| item.artifact)
        .ok_or(CombineError::NotCombination(at))
}

/// The hero changes that disassemble the combination at `at` ([`check_disassemble`]): it leaves,
/// then each part, in the ruleset's artifact order, is received at its first position
/// (HER-CHG-002: the locks freed make room, so none goes to the backpack). The Spell Book Titan's
/// Thunder added stays (HER-ART-007). The original's `hero_disassemble_combination` (0x4DC070).
///
/// # Errors
/// [`check_disassemble`]'s.
pub fn disassemble_changes(
    details: &HeroDetails,
    ruleset: &Ruleset,
    at: Position,
) -> Result<Vec<HeroChange>, CombineError> {
    let combination = check_disassemble(details, ruleset, at)?;
    let parts = ruleset
        .artifact(combination)
        .map(|c| c.parts.as_slice())
        .unwrap_or_default();
    let ordered = (0_u16..)
        .map(ArtifactTypeId::from_raw)
        .take(ruleset.artifacts().len())
        .flat_map(|id| parts.iter().filter(move |&&p| p == id).map(move |_| id));
    Ok(
        core::iter::once(HeroChange::RemoveArtifact(ArtifactPlace::Worn(at)))
            .chain(ordered.map(|id| HeroChange::AddArtifact(id.into())))
            .collect(),
    )
}

/// The artifact worn at `at`.
fn worn_at(details: &HeroDetails, at: Position) -> Result<ArtifactInstance, CombineError> {
    details
        .worn
        .get(at.0)
        .copied()
        .flatten()
        .ok_or(CombineError::NoArtifact(at))
}
