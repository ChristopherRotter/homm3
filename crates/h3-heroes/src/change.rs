//! Changes to a hero's details (E08 T19a2a; ADR-0013 point 2, E08 decision 25): the typed changes
//! a context returns and the game applies to its store of heroes, each checked first
//! ([`check_change`], HER-CHG-001) so that applying it ([`apply_change`]) cannot fail; where a
//! received artifact goes (HER-CHG-002); what a defeated hero yields, in which order
//! (HER-ART-004); the artifact a destroyed war machine takes with it (HER-WAR-002); since E08
//! T19a2b a learned spell.

use alloc::vec;

use h3_content::{ArtifactType, CreatureFlag, CreatureTypeId, Ruleset, SpellId};

use crate::details::{ArtifactInstance, HeroDetails, check_artifact, hero_spell};
use crate::error::ChangeError;

/// Where an artifact is in a hero's details.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactPlace {
    /// Worn: its index in [`HeroDetails::worn`].
    Worn(usize),
    /// Carried: its index in [`HeroDetails::backpack`].
    Backpack(usize),
}

/// A change to a hero's details (ADR-0013 point 2): what a context returns and the game applies
/// to its store, checked first ([`check_change`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeroChange {
    /// The hero gains experience points (the sum saturates).
    GainExperience(u32),
    /// Its spell points become this many.
    SetSpellPoints(u16),
    /// It receives an artifact: worn if it fits, else at the end of the backpack (HER-CHG-002).
    AddArtifact(ArtifactInstance),
    /// It loses the artifact at this place; the later ones of that list move up.
    RemoveArtifact(ArtifactPlace),
    /// It learns a spell, added at the end of its learned spells (E08 T19a2b: an Eagle Eye pick,
    /// HER-SPL-006).
    LearnSpell(SpellId),
}

/// Checks a change against valid hero details (HER-CHG-001): the details stay valid with it
/// ([`crate::validate`]). Experience and spell points always pass; a received artifact must be
/// one of the ruleset holding a spell exactly when it is a scroll (HER-ART-003) and find room
/// (HER-CHG-002); a removed one must be there; a learned spell must be a hero spell of the
/// ruleset (HER-SPL-001) the hero has not learned.
///
/// # Errors
/// [`ChangeError::Artifact`] for an artifact the ruleset refuses, [`ChangeError::NoRoom`] when it
/// fits no free position and the backpack cannot take it, [`ChangeError::NoArtifact`] when there
/// is none at the place, [`ChangeError::Spell`] for a spell the ruleset refuses,
/// [`ChangeError::KnownSpell`] for one the hero has learned.
pub fn check_change(
    details: &HeroDetails,
    ruleset: &Ruleset,
    change: HeroChange,
) -> Result<(), ChangeError> {
    // rule: HER-CHG-001
    match change {
        HeroChange::GainExperience(_) | HeroChange::SetSpellPoints(_) => Ok(()),
        HeroChange::AddArtifact(item) => {
            let artifact = check_artifact(item, ruleset).map_err(ChangeError::Artifact)?;
            has_room(details, ruleset, artifact)
                .then_some(())
                .ok_or(ChangeError::NoRoom(item.artifact))
        }
        HeroChange::RemoveArtifact(place) => artifact_at(details, place)
            .map(|_| ())
            .ok_or(ChangeError::NoArtifact(place)),
        HeroChange::LearnSpell(spell) => {
            hero_spell(spell, ruleset).map_err(ChangeError::Spell)?;
            (!details.spells.contains(&spell))
                .then_some(())
                .ok_or(ChangeError::KnownSpell(spell))
        }
    }
}

/// Applies a change if [`check_change`] passes it (HER-CHG-001, HER-CHG-002); one it refuses
/// changes nothing.
pub fn apply_change(details: &mut HeroDetails, ruleset: &Ruleset, change: HeroChange) {
    if check_change(details, ruleset, change).is_err() {
        return;
    }
    match change {
        HeroChange::GainExperience(points) => {
            details.experience = details.experience.saturating_add(points);
        }
        HeroChange::SetSpellPoints(points) => details.mana = points,
        HeroChange::AddArtifact(item) => {
            // Checked: the artifact is the ruleset's and finds room.
            let worn = ruleset
                .artifact(item.artifact)
                .is_some_and(|artifact| fits_worn(details, ruleset, artifact));
            if worn {
                details.worn.push(item);
            } else {
                details.backpack.push(item);
            }
        }
        HeroChange::RemoveArtifact(place) => {
            let (list, index) = match place {
                ArtifactPlace::Worn(i) => (&mut details.worn, i),
                ArtifactPlace::Backpack(i) => (&mut details.backpack, i),
            };
            // Checked: there is an artifact at `index`.
            let rest = list.split_off(index.min(list.len()));
            list.extend(rest.into_iter().skip(1));
        }
        HeroChange::LearnSpell(spell) => details.spells.push(spell),
    }
}

/// The artifact at `place`, if there is one.
fn artifact_at(details: &HeroDetails, place: ArtifactPlace) -> Option<&ArtifactInstance> {
    match place {
        ArtifactPlace::Worn(i) => details.worn.get(i),
        ArtifactPlace::Backpack(i) => details.backpack.get(i),
    }
}

/// Whether a received artifact finds room (HER-CHG-002): it is worn if its slot and each slot it
/// locks have a free position ([`fits_worn`]: HER-ART-001 holds with it), else carried at the
/// end of the backpack if its slot's artifacts may go there (HER-ART-002) and it is not full. The
/// original equips it at the first body position that takes it, else appends it to the backpack,
/// at most 64 and no war machine (`hero_give_artifact` 0x4E3070 through `hero_equip_artifact`
/// 0x4E2A00 with position −1 and `hero_give_backpack_artifact` 0x4E2F90); positions of one slot
/// are interchangeable here.
fn has_room(details: &HeroDetails, ruleset: &Ruleset, artifact: &ArtifactType) -> bool {
    // rule: HER-CHG-002
    let carried = artifact
        .slot
        .and_then(|s| ruleset.artifact_slot(s))
        .is_none_or(|slot| slot.backpack);
    let room = ruleset
        .hero_params()
        .is_some_and(|params| details.backpack.len() < usize::from(params.backpack));
    fits_worn(details, ruleset, artifact) || (carried && room)
}

/// Whether `artifact` has a slot and, with it and its locks, no slot needs more positions than
/// it has.
fn fits_worn(details: &HeroDetails, ruleset: &Ruleset, artifact: &ArtifactType) -> bool {
    let slots = ruleset.artifact_slots();
    let mut used = vec![0_u16; slots.len()];
    let worn = details
        .worn
        .iter()
        .filter_map(|item| ruleset.artifact(item.artifact));
    for a in worn.chain(core::iter::once(artifact)) {
        // The compiler resolved every slot and lock against the ruleset's slots.
        for s in a.slot.into_iter().chain(a.locks.iter().copied()) {
            used.get_mut(s.index())
                .into_iter()
                .for_each(|n| *n = n.saturating_add(1));
        }
    }
    artifact.slot.is_some()
        && used
            .iter()
            .zip(slots)
            .all(|(&n, slot)| n <= u16::from(slot.count))
}

/// The next artifact a defeated hero yields to the winner, with its place (HER-ART-004): the
/// first worn artifact of a slot whose artifacts may go into the backpack (so no war machine, no
/// Catapult and no spell book), in the order worn, then the last one of the backpack. The
/// original takes the worn ones by body position, skipping the spell book, the Grail and the four
/// war machines, then the backpack from its end (`battle_capture_artifacts` 0x46A070).
pub fn next_capture(
    details: &HeroDetails,
    ruleset: &Ruleset,
) -> Option<(ArtifactPlace, ArtifactInstance)> {
    // rule: HER-ART-004
    let capturable = |(_, item): &(usize, &ArtifactInstance)| {
        ruleset
            .artifact(item.artifact)
            .and_then(|a| a.slot)
            .and_then(|s| ruleset.artifact_slot(s))
            .is_some_and(|slot| slot.backpack)
    };
    let worn = details.worn.iter().enumerate().find(capturable);
    let worn = worn.map(|(i, &item)| (ArtifactPlace::Worn(i), item));
    worn.or_else(|| {
        let last = details.backpack.iter().enumerate().next_back();
        last.map(|(i, &item)| (ArtifactPlace::Backpack(i), item))
    })
}

/// The artifact a hero loses with its destroyed war machine `machine` (HER-WAR-002): the first
/// worn artifact that places it; none for a machine with the `catapult` flag, which stays.
pub fn lost_war_machine(
    details: &HeroDetails,
    ruleset: &Ruleset,
    machine: CreatureTypeId,
) -> Option<ArtifactPlace> {
    // rule: HER-WAR-002
    let catapult = ruleset
        .creature(machine)
        .is_some_and(|c| c.flags.contains(CreatureFlag::Catapult));
    let places = |item: &ArtifactInstance| {
        ruleset
            .artifact(item.artifact)
            .is_some_and(|a| a.war_machine == Some(machine))
    };
    let at = details.worn.iter().position(places);
    at.filter(|_| !catapult).map(ArtifactPlace::Worn)
}
