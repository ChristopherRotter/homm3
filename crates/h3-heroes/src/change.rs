//! Changes to a hero's details (E08 T19a2a; ADR-0013 point 2, E08 decision 25): the typed changes
//! a context returns and the game applies to its store of heroes, each checked first
//! ([`check_change`], HER-CHG-001) so that applying it ([`apply_change`]) cannot fail; where a
//! received artifact goes (HER-CHG-002); what a defeated hero yields, in which order
//! (HER-ART-004); the artifact a destroyed war machine takes with it (HER-WAR-002); since E08
//! T19a2b a learned spell; since E08 T27 a level (HER-LVL-005) and the experience a gain gives
//! under the scenario's level limit (HER-EXP-003); since E08 T49a the worn artifacts by position
//! and an artifact placed at a given place; since E08 T49b the Spell Book an equipped Titan's
//! Thunder adds (HER-ART-007).

use h3_content::{
    ArtifactType, CreatureFlag, CreatureTypeId, MAX_HERO_SKILLS, MAX_SKILL_LEVEL, Ruleset, SkillId,
    SpellId,
};

use crate::details::{
    ArtifactInstance, HeroDetails, OfferLevels, PrimarySkill, check_artifact, hero_spell,
};
use crate::error::ChangeError;
use crate::level::{gained_under_limit, pending_levels};
use crate::positions::{Position, equip, first_free, fits};

/// Where an artifact is in a hero's details.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactPlace {
    /// Worn: its position ([`HeroDetails::worn`]).
    Worn(Position),
    /// Carried: its index in [`HeroDetails::backpack`].
    Backpack(usize),
}

/// A change to a hero's details (ADR-0013 point 2): what a context returns and the game applies
/// to its store, checked first ([`check_change`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeroChange {
    /// The hero gains experience points: the sum, at most the bound or the scenario's level
    /// limit's cap (HER-EXP-002, HER-EXP-003; [`apply_change_limited`]).
    GainExperience(u32),
    /// Its spell points become this many.
    SetSpellPoints(u16),
    /// It receives an artifact: worn at the first position that takes it, else at the end of the
    /// backpack (HER-CHG-002). Worn, it is equipped: Titan's Thunder adds a Spell Book
    /// (HER-ART-007, [`crate::equip`]).
    AddArtifact(ArtifactInstance),
    /// It loses the artifact at this place: a position is left empty, the backpack's later
    /// artifacts move up.
    RemoveArtifact(ArtifactPlace),
    /// It gets an artifact at a given place (E08 T49a; ADR-0013 point 14.4, a move's target): an
    /// empty position of the artifact's slot where the locks fit with it (HER-ART-001), or a
    /// backpack index up to the backpack's length, inserted there, for an artifact the backpack
    /// carries while it holds fewer than its most (HER-ART-002). Worn, it is equipped: Titan's
    /// Thunder adds a Spell Book (HER-ART-007, [`crate::equip`]).
    PlaceArtifact {
        /// The artifact.
        artifact: ArtifactInstance,
        /// Where it goes.
        at: ArtifactPlace,
    },
    /// It learns a spell, added at the end of its learned spells (E08 T19a2b: an Eagle Eye pick,
    /// HER-SPL-006).
    LearnSpell(SpellId),
    /// It is given its next level (E08 T27, HER-LVL-005; [`crate::LevelUp::change`]): one more
    /// level, one more of the primary skill (at most the ruleset's `primary_max`: A-9,
    /// `ADV-QUIRK-13`), the learned skill (a known one a level up, a new one at Basic after the
    /// others) and the offers' levels.
    GainLevel {
        /// The primary skill that gains 1.
        primary: PrimarySkill,
        /// The skill learned, if any.
        learned: Option<SkillId>,
        /// The levels at which Wisdom and a school were last offered, after this level.
        offered: OfferLevels,
    },
}

/// Checks a change against valid hero details (HER-CHG-001): the details stay valid with it
/// ([`crate::validate`]). Experience and spell points always pass; a received artifact must be
/// one of the ruleset holding a spell exactly when it is a scroll (HER-ART-003) and find room
/// (HER-CHG-002); a removed one must be there; a placed one must be the ruleset's and fit its
/// place ([`HeroChange::PlaceArtifact`]); a learned spell must be a hero spell of the
/// ruleset (HER-SPL-001) the hero has not learned; a level must be pending, its learned skill the
/// ruleset's, below Expert if known and new only below the most skills, the offers' levels at most
/// the new level (HER-LVL-005). Whether the skill was one of the level's offers is the caller's
/// check (it holds the drawn [`crate::LevelUp`]).
///
/// # Errors
/// [`ChangeError::Artifact`] for an artifact the ruleset refuses, [`ChangeError::NoRoom`] when it
/// fits no free position and the backpack cannot take it, [`ChangeError::NoArtifact`] when there
/// is none at the place, [`ChangeError::CannotPlace`] for a place that refuses a placed artifact,
/// [`ChangeError::Spell`] for a spell the ruleset refuses,
/// [`ChangeError::KnownSpell`] for one the hero has learned; [`ChangeError::NoLevelPending`],
/// [`ChangeError::CannotLearn`] and [`ChangeError::OfferedAboveLevel`] for a level.
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
        HeroChange::PlaceArtifact { artifact, at } => {
            let artifact = check_artifact(artifact, ruleset).map_err(ChangeError::Artifact)?;
            fits_at(details, ruleset, artifact, at)
                .then_some(())
                .ok_or(ChangeError::CannotPlace(at))
        }
        HeroChange::LearnSpell(spell) => {
            hero_spell(spell, ruleset).map_err(ChangeError::Spell)?;
            (!details.spells.contains(&spell))
                .then_some(())
                .ok_or(ChangeError::KnownSpell(spell))
        }
        HeroChange::GainLevel {
            learned, offered, ..
        } => check_level(details, ruleset, learned, offered),
    }
}

/// [`check_change`] of a level (HER-LVL-005).
fn check_level(
    details: &HeroDetails,
    ruleset: &Ruleset,
    learned: Option<SkillId>,
    offered: OfferLevels,
) -> Result<(), ChangeError> {
    // rule: HER-LVL-005
    let pending = ruleset
        .hero_params()
        .is_some_and(|params| pending_levels(details, params) > 0);
    if !pending {
        return Err(ChangeError::NoLevelPending);
    }
    if let Some(skill) = learned {
        let known = details.skills.iter().find(|&&(s, _)| s == skill);
        let learnable = ruleset.skill(skill).is_some()
            && known.map_or(details.skills.len() < MAX_HERO_SKILLS, |&(_, level)| {
                level < MAX_SKILL_LEVEL
            });
        if !learnable {
            return Err(ChangeError::CannotLearn(skill));
        }
    }
    let level = u16::from(details.level) + 1;
    (u16::from(offered.wisdom.max(offered.school)) <= level)
        .then_some(())
        .ok_or(ChangeError::OfferedAboveLevel)
}

/// Applies a change if [`check_change`] passes it (HER-CHG-001, HER-CHG-002); one it refuses
/// changes nothing. A scenario without a level limit: [`apply_change_limited`] with limit 0.
pub fn apply_change(details: &mut HeroDetails, ruleset: &Ruleset, change: HeroChange) {
    apply_change_limited(details, ruleset, 0, change);
}

/// [`apply_change`] in a scenario with the level limit `level_limit` (0 none): experience gains
/// stop at the limit's cap and lower experience above it (HER-EXP-003,
/// [`crate::gained_under_limit`]).
pub fn apply_change_limited(
    details: &mut HeroDetails,
    ruleset: &Ruleset,
    level_limit: u8,
    change: HeroChange,
) {
    if check_change(details, ruleset, change).is_err() {
        return;
    }
    match change {
        HeroChange::GainExperience(points) => {
            // rule: HER-EXP-003
            details.experience = ruleset.hero_params().map_or(details.experience, |params| {
                gained_under_limit(details, params, points, level_limit)
            });
        }
        HeroChange::SetSpellPoints(points) => details.mana = points,
        HeroChange::AddArtifact(item) => {
            // Checked: the artifact is the ruleset's and finds room.
            let at = ruleset
                .artifact(item.artifact)
                .and_then(|artifact| first_free(&details.worn, ruleset, artifact));
            match at {
                Some(at) => place(details, ruleset, item, ArtifactPlace::Worn(at)),
                None => details.backpack.push(item),
            }
        }
        HeroChange::RemoveArtifact(ArtifactPlace::Worn(at)) => {
            // Checked: there is an artifact at `at`.
            details
                .worn
                .get_mut(at.0)
                .into_iter()
                .for_each(|item| *item = None);
        }
        HeroChange::RemoveArtifact(ArtifactPlace::Backpack(i)) => {
            // Checked: there is an artifact at `i`.
            let backpack = &mut details.backpack;
            let rest = backpack.split_off(i.min(backpack.len()));
            backpack.extend(rest.into_iter().skip(1));
        }
        HeroChange::PlaceArtifact { artifact, at } => place(details, ruleset, artifact, at),
        HeroChange::LearnSpell(spell) => details.spells.push(spell),
        HeroChange::GainLevel {
            primary,
            learned,
            offered,
        } => {
            // rule: HER-LVL-005
            details.level = details.level.saturating_add(1);
            // A-9: the gain saturates at `primary_max`.
            // rule: HER-LVL-001
            let max = ruleset.hero_params().map_or(0, |params| params.primary_max);
            let base = &mut details.primary[primary.index()];
            *base = base.saturating_add(1).min(max);
            if let Some(skill) = learned {
                match details.skills.iter_mut().find(|(s, _)| *s == skill) {
                    Some((_, level)) => *level += 1,
                    None => details.skills.push((skill, 1)),
                }
            }
            details.offered = offered;
        }
    }
}

/// Puts `item` at a checked place: equipped at an empty position (with the Spell Book Titan's
/// Thunder adds, HER-ART-007), or inserted at a backpack index up to its length.
fn place(details: &mut HeroDetails, ruleset: &Ruleset, item: ArtifactInstance, at: ArtifactPlace) {
    match at {
        ArtifactPlace::Worn(at) => {
            equip(&mut details.worn, ruleset, item, at);
        }
        ArtifactPlace::Backpack(i) => details.backpack.insert(i, item),
    }
}

/// The artifact at `place`, if there is one.
fn artifact_at(details: &HeroDetails, place: ArtifactPlace) -> Option<&ArtifactInstance> {
    match place {
        ArtifactPlace::Worn(at) => details.worn.get(at.0).and_then(Option::as_ref),
        ArtifactPlace::Backpack(i) => details.backpack.get(i),
    }
}

/// Whether `artifact` fits `at` ([`HeroChange::PlaceArtifact`]): an empty position of its slot
/// with the locks fitting (HER-ART-001), or a backpack index up to the backpack's length while the
/// backpack carries it and has room (HER-ART-002).
fn fits_at(
    details: &HeroDetails,
    ruleset: &Ruleset,
    artifact: &ArtifactType,
    at: ArtifactPlace,
) -> bool {
    match at {
        ArtifactPlace::Worn(at) => fits(&details.worn, ruleset, artifact, at),
        ArtifactPlace::Backpack(i) => {
            i <= details.backpack.len() && carried(ruleset, artifact) && has_space(details, ruleset)
        }
    }
}

/// Whether a received artifact finds room (HER-CHG-002): it is worn at the first position that
/// takes it ([`crate::first_position`]: HER-ART-001 holds with it), else carried at the end of the
/// backpack if its slot's artifacts may go there (HER-ART-002) and it is not full. The original
/// equips it at the first body position that takes it, else appends it to the backpack, at most
/// 64 and no war machine (`hero_give_artifact` 0x4E3070 through `hero_equip_artifact` 0x4E2A00
/// with position −1 and `hero_give_backpack_artifact` 0x4E2F90).
fn has_room(details: &HeroDetails, ruleset: &Ruleset, artifact: &ArtifactType) -> bool {
    // rule: HER-CHG-002
    first_free(&details.worn, ruleset, artifact).is_some()
        || (carried(ruleset, artifact) && has_space(details, ruleset))
}

/// Whether the backpack carries `artifact`: it has no slot (the Grail) or its slot's artifacts
/// may go there (HER-ART-002).
fn carried(ruleset: &Ruleset, artifact: &ArtifactType) -> bool {
    artifact
        .slot
        .and_then(|s| ruleset.artifact_slot(s))
        .is_none_or(|slot| slot.backpack)
}

/// Whether the backpack holds fewer than the ruleset's most (HER-ART-002).
fn has_space(details: &HeroDetails, ruleset: &Ruleset) -> bool {
    ruleset
        .hero_params()
        .is_some_and(|params| details.backpack.len() < usize::from(params.backpack))
}

/// The next artifact a defeated hero yields to the winner, with its place (HER-ART-004): the
/// first worn artifact of a slot whose artifacts may go into the backpack (so no war machine, no
/// Catapult and no spell book), in position order, then the last one of the backpack. The
/// original takes the worn ones by body position, skipping the spell book, the Grail and the four
/// war machines, then the backpack from its end (`battle_capture_artifacts` 0x46A070).
pub fn next_capture(
    details: &HeroDetails,
    ruleset: &Ruleset,
) -> Option<(ArtifactPlace, ArtifactInstance)> {
    // rule: HER-ART-004
    let capturable = |item: &ArtifactInstance| {
        ruleset
            .artifact(item.artifact)
            .and_then(|a| a.slot)
            .and_then(|s| ruleset.artifact_slot(s))
            .is_some_and(|slot| slot.backpack)
    };
    let worn = details.worn.iter().enumerate().find_map(|(i, item)| {
        item.filter(capturable)
            .map(|item| (ArtifactPlace::Worn(Position(i)), item))
    });
    worn.or_else(|| {
        let last = details.backpack.iter().enumerate().next_back();
        last.map(|(i, &item)| (ArtifactPlace::Backpack(i), item))
    })
}

/// The artifact a hero loses with its destroyed war machine `machine` (HER-WAR-002): the first
/// position holding an artifact that places it; none for a machine with the `catapult` flag,
/// which stays.
pub fn lost_war_machine(
    details: &HeroDetails,
    ruleset: &Ruleset,
    machine: CreatureTypeId,
) -> Option<ArtifactPlace> {
    // rule: HER-WAR-002
    let catapult = ruleset
        .creature(machine)
        .is_some_and(|c| c.flags.contains(CreatureFlag::Catapult));
    let places = |item: &Option<ArtifactInstance>| {
        item.and_then(|item| ruleset.artifact(item.artifact))
            .is_some_and(|a| a.war_machine == Some(machine))
    };
    let at = details.worn.iter().position(places);
    at.filter(|_| !catapult)
        .map(|at| ArtifactPlace::Worn(Position(at)))
}
