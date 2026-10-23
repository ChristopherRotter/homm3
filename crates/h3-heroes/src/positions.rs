//! The positions of a hero's worn artifacts (E08 T49a; ADR-0013 point 1, HER-ART-001): one per
//! position of the ruleset's artifact slots, in the slots' order and each slot's count; the
//! locks of the worn combinations counted per slot; the first position that takes an artifact
//! (HER-CHG-002); a list of artifacts placed at their first positions ([`wearing`]).

use alloc::vec;
use alloc::vec::Vec;

use h3_content::{ArtifactSlotId, ArtifactType, ArtifactTypeId, Ruleset};

use crate::details::{ArtifactInstance, HeroDetails};

/// A position of a hero's worn artifacts: an index into [`positions`] and
/// [`HeroDetails::worn`]. SoD has 19: head 0, shoulders 1, neck 2, right hand 3, left hand 4,
/// torso 5, rings 6 and 7, feet 8, misc 9 to 13, war machines 14 to 17, spell book 18 (the
/// original numbers its fifth misc position 18, after the book; no walk tells the two orders
/// apart, ADR-0013 point 1). Before SoD the fourth misc position is the last misc (12).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position(pub usize);

impl core::fmt::Display for Position {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Each position's slot (HER-ART-001): the ruleset's artifact slots in their order, each its
/// `count` times. [`HeroDetails::worn`] has one entry per position.
pub fn positions(ruleset: &Ruleset) -> Vec<ArtifactSlotId> {
    (0_u16..)
        .zip(ruleset.artifact_slots())
        .flat_map(|(raw, slot)| {
            core::iter::repeat_n(ArtifactSlotId::from_raw(raw), usize::from(slot.count))
        })
        .collect()
}

/// The first position that takes `artifact` (HER-CHG-002): the lowest empty position of its slot
/// when, with it there, no slot's free positions less the worn combinations' locks go below zero
/// (HER-ART-001). None for an artifact without a slot (the Grail), one the ruleset does not know,
/// or one that does not fit. The original's `hero_equip_artifact` (0x4E2A00) with position −1.
pub fn first_position(
    details: &HeroDetails,
    ruleset: &Ruleset,
    artifact: ArtifactTypeId,
) -> Option<Position> {
    ruleset
        .artifact(artifact)
        .and_then(|a| first_free(&details.worn, ruleset, a))
}

/// [`first_position`] in a list of positions.
pub(crate) fn first_free(
    worn: &[Option<ArtifactInstance>],
    ruleset: &Ruleset,
    artifact: &ArtifactType,
) -> Option<Position> {
    // rule: HER-CHG-002
    let free = positions(ruleset)
        .iter()
        .zip(worn)
        .position(|(&slot, item)| artifact.slot == Some(slot) && item.is_none());
    let worn = worn
        .iter()
        .flatten()
        .filter_map(|item| ruleset.artifact(item.artifact));
    free.filter(|_| full_slot(ruleset, worn.chain(core::iter::once(artifact))).is_none())
        .map(Position)
}

/// The positions of a hero wearing `items` (E08 T49a): each placed in list order at the first
/// position that takes it ([`first_position`]); one that none takes is left out. For callers
/// that list artifacts (tests, experiments, generated heroes); a caller that needs a given
/// placement, or an invalid one, writes the positions itself. Validate what it builds.
pub fn wearing(ruleset: &Ruleset, items: &[ArtifactInstance]) -> Vec<Option<ArtifactInstance>> {
    let mut worn = vec![None; positions(ruleset).len()];
    for &item in items {
        let at = ruleset
            .artifact(item.artifact)
            .and_then(|artifact| first_free(&worn, ruleset, artifact));
        at.and_then(|at| worn.get_mut(at.0))
            .into_iter()
            .for_each(|place| *place = Some(item));
    }
    worn
}

/// The first slot whose positions the artifacts and their locks need more of than it has
/// (HER-ART-001: each artifact takes a position of its slot and one more of each slot in its
/// `locks`), if any.
pub(crate) fn full_slot<'a>(
    ruleset: &Ruleset,
    worn: impl Iterator<Item = &'a ArtifactType>,
) -> Option<ArtifactSlotId> {
    let slots = ruleset.artifact_slots();
    let mut used = vec![0_u16; slots.len()];
    for artifact in worn {
        // The compiler resolved every slot and lock against the ruleset's slots.
        for s in artifact
            .slot
            .into_iter()
            .chain(artifact.locks.iter().copied())
        {
            used.get_mut(s.index())
                .into_iter()
                .for_each(|n| *n = n.saturating_add(1));
        }
    }
    (0_u16..)
        .zip(used.iter().zip(slots))
        .find(|(_, (n, slot))| **n > u16::from(slot.count))
        .map(|(raw, _)| ArtifactSlotId::from_raw(raw))
}
