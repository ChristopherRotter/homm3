//! `HeroDetails`, their validation (HER-DET-001, HER-SEC-001, HER-ART-001 to 003, HER-SPL-001)
//! and the war machines a hero brings to a battle (HER-WAR-001).

use alloc::collections::BTreeSet;
use alloc::vec;
use alloc::vec::Vec;

use h3_content::{
    ArtifactSlotId, ArtifactType, ArtifactTypeId, CreatureTypeId, HeroEffect, HeroParams,
    HeroTypeId, MAX_HERO_SKILLS, MAX_SKILL_LEVEL, Ruleset, SkillId, SpellCategory, SpellId,
};

use crate::error::DetailsError;

/// The four primary skills, in the order of every `[u8; 4]` of primary skills.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrimarySkill {
    /// Attack: added to the attack of the hero's creatures.
    Attack,
    /// Defense: added to the defense of the hero's creatures.
    Defense,
    /// Spell power (E04b).
    Power,
    /// Knowledge: the hero's maximum mana (E04b).
    Knowledge,
}

impl PrimarySkill {
    /// All values, in array order.
    pub const ALL: [Self; 4] = [Self::Attack, Self::Defense, Self::Power, Self::Knowledge];

    /// Position in a `[u8; 4]` of primary skills.
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Lower-case name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Attack => "attack",
            Self::Defense => "defense",
            Self::Power => "power",
            Self::Knowledge => "knowledge",
        }
    }
}

/// One artifact a hero wears or carries (E04b T8a): its type and, for an artifact whose type
/// has the `scroll` effect (the Spell Scroll), the spell this copy holds (HER-ART-003).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtifactInstance {
    /// The artifact type.
    pub artifact: ArtifactTypeId,
    /// The spell of a scroll; `None` for every other artifact.
    pub spell: Option<SpellId>,
}

impl ArtifactInstance {
    /// A scroll holding `spell`.
    pub const fn scroll(artifact: ArtifactTypeId, spell: SpellId) -> Self {
        Self {
            artifact,
            spell: Some(spell),
        }
    }
}

impl From<ArtifactTypeId> for ArtifactInstance {
    /// An artifact that holds no spell.
    fn from(artifact: ArtifactTypeId) -> Self {
        Self {
            artifact,
            spell: None,
        }
    }
}

/// The lasting details of a hero (ADR-0013 point 1): what the hero is between battles, apart
/// from its army. Values are *base* values; what artifacts and the specialty add is derived
/// ([`crate::derive_battle_hero`]).
///
/// Artifacts are worn by kind of slot: an artifact occupies a position of its own slot, and a
/// combination one more position of each slot in its `locks` (HER-ART-001). Positions of one
/// slot are interchangeable, so `worn` lists artifacts, not positions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeroDetails {
    /// The hero type: class, specialty and identity.
    pub hero_type: HeroTypeId,
    /// Level (at least 1).
    pub level: u8,
    /// Experience points. Not checked against the level: the experience table arrives with
    /// hero progression (E08–E10).
    pub experience: u32,
    /// Base primary skills ([`PrimarySkill`] order), without artifacts; at most the ruleset's
    /// `primary_max`. A value below the ruleset's minimum is allowed and counts as the minimum
    /// (a map may set Knowledge 0).
    pub primary: [u8; 4],
    /// Secondary skills with their levels (1 Basic to 3 Expert), in display order.
    pub skills: Vec<(SkillId, u8)>,
    /// Worn artifacts, in any order; the same type may appear more than once.
    pub worn: Vec<ArtifactInstance>,
    /// Artifacts carried in the backpack, in backpack order.
    pub backpack: Vec<ArtifactInstance>,
    /// Spells the hero has learned (the original's `+0x3EA`): kept whatever it wears. The spells
    /// it can cast add those of its worn artifacts ([`crate::available_spells`], HER-SPL-002)
    /// and need a worn spell book (HER-BAT-011).
    pub spells: Vec<SpellId>,
    /// Spell points (E04b). May exceed the maximum (a Mana Vortex), so it is not bounded here.
    pub mana: u16,
}

/// Checks hero details against a ruleset: every reference exists and every limit holds.
/// Returns the first problem in field order.
///
/// # Errors
/// The [`DetailsError`] of the first rule broken.
pub fn validate(details: &HeroDetails, ruleset: &Ruleset) -> Result<(), DetailsError> {
    check(details, ruleset).map(|_| ())
}

/// [`validate`], returning the ruleset's hero parameters the details were checked against.
pub(crate) fn check<'r>(
    details: &HeroDetails,
    ruleset: &'r Ruleset,
) -> Result<&'r HeroParams, DetailsError> {
    let params = ruleset.hero_params().ok_or(DetailsError::NoHeroParams)?;
    // rule: HER-DET-001
    ruleset
        .hero_type(details.hero_type)
        .ok_or(DetailsError::UnknownHeroType)?;
    if details.level == 0 {
        return Err(DetailsError::LevelZero);
    }
    if let Some(skill) = PrimarySkill::ALL
        .into_iter()
        .find(|s| details.primary[s.index()] > params.primary_max)
    {
        return Err(DetailsError::PrimaryAboveLimit(skill));
    }
    validate_skills(&details.skills, ruleset)?;
    validate_worn(&details.worn, ruleset)?;
    // rule: HER-ART-002
    if details.backpack.len() > usize::from(params.backpack) {
        return Err(DetailsError::BackpackFull);
    }
    for &item in &details.backpack {
        let id = item.artifact;
        let artifact = ruleset
            .artifact(id)
            .ok_or(DetailsError::UnknownArtifact(id))?;
        let fixed = artifact
            .slot
            .and_then(|s| ruleset.artifact_slot(s))
            .is_some_and(|s| !s.backpack);
        if fixed {
            return Err(DetailsError::NotForBackpack(id));
        }
        validate_scroll(item, ruleset)?;
    }
    validate_spells(&details.spells, ruleset)?;
    Ok(params)
}

/// At most [`MAX_HERO_SKILLS`] skills of the ruleset, each once, at Basic to Expert.
fn validate_skills(skills: &[(SkillId, u8)], ruleset: &Ruleset) -> Result<(), DetailsError> {
    // rule: HER-SEC-001
    if skills.len() > MAX_HERO_SKILLS {
        return Err(DetailsError::TooManySkills);
    }
    let mut seen = BTreeSet::new();
    for &(id, level) in skills {
        ruleset.skill(id).ok_or(DetailsError::UnknownSkill(id))?;
        if !seen.insert(id) {
            return Err(DetailsError::DuplicateSkill(id));
        }
        if !(1..=MAX_SKILL_LEVEL).contains(&level) {
            return Err(DetailsError::SkillLevel(id));
        }
    }
    Ok(())
}

/// Every worn artifact has a slot, and the positions they and their locks occupy fit every
/// slot's count.
fn validate_worn(worn: &[ArtifactInstance], ruleset: &Ruleset) -> Result<(), DetailsError> {
    // rule: HER-ART-001
    let slots = ruleset.artifact_slots();
    let mut used: Vec<u16> = vec![0; slots.len()];
    for &item in worn {
        let id = item.artifact;
        let artifact = ruleset
            .artifact(id)
            .ok_or(DetailsError::UnknownArtifact(id))?;
        let slot = artifact.slot.ok_or(DetailsError::NotWearable(id))?;
        validate_scroll(item, ruleset)?;
        // The compiler resolved every slot and lock against the ruleset's slots.
        for s in core::iter::once(slot).chain(artifact.locks.iter().copied()) {
            used.get_mut(s.index())
                .into_iter()
                .for_each(|n| *n = n.saturating_add(1));
        }
    }
    match used
        .iter()
        .zip(slots)
        .position(|(&n, slot)| n > u16::from(slot.count))
        .and_then(ArtifactSlotId::from_index)
    {
        Some(full) => Err(DetailsError::SlotFull(full)),
        None => Ok(()),
    }
}

/// The type of an artifact a hero may hold (HER-ART-003): an artifact of the ruleset, holding a
/// spell exactly when it is a scroll ([`validate_scroll`]).
pub(crate) fn check_artifact(
    item: ArtifactInstance,
    ruleset: &Ruleset,
) -> Result<&ArtifactType, DetailsError> {
    let artifact = ruleset
        .artifact(item.artifact)
        .ok_or(DetailsError::UnknownArtifact(item.artifact))?;
    validate_scroll(item, ruleset)?;
    Ok(artifact)
}

/// An artifact holds a spell exactly when its type has the `scroll` effect, and that spell is a
/// spell of the ruleset that is not a creature ability. The artifact is known.
fn validate_scroll(item: ArtifactInstance, ruleset: &Ruleset) -> Result<(), DetailsError> {
    // rule: HER-ART-003
    let scroll = ruleset
        .artifact(item.artifact)
        .is_some_and(|a| a.effects.iter().any(|e| matches!(e, HeroEffect::Scroll)));
    match (scroll, item.spell) {
        (true, None) => Err(DetailsError::ScrollWithoutSpell(item.artifact)),
        (false, Some(_)) => Err(DetailsError::SpellOnArtifact(item.artifact)),
        (true, Some(id)) => hero_spell(id, ruleset),
        (false, None) => Ok(()),
    }
}

/// `id` is a spell of the ruleset and not a creature ability.
fn hero_spell(id: SpellId, ruleset: &Ruleset) -> Result<(), DetailsError> {
    let spell = ruleset.spell(id).ok_or(DetailsError::UnknownSpell(id))?;
    if spell.category == SpellCategory::Ability {
        return Err(DetailsError::NotAHeroSpell(id));
    }
    Ok(())
}

/// Spells of the ruleset, each once, and none of them a creature ability.
fn validate_spells(spells: &[SpellId], ruleset: &Ruleset) -> Result<(), DetailsError> {
    // rule: HER-SPL-001
    let mut seen = BTreeSet::new();
    for &id in spells {
        ruleset.spell(id).ok_or(DetailsError::UnknownSpell(id))?;
        if !seen.insert(id) {
            return Err(DetailsError::DuplicateSpell(id));
        }
        hero_spell(id, ruleset)?;
    }
    Ok(())
}

/// The war machines a hero brings to a battle (`BattleSetup.war_machines`): the war machine of
/// every worn artifact that places one, in the order worn. The Catapult's artifact places none
/// until sieges arrive (E04c). Artifacts the ruleset does not know are skipped; validate first.
pub fn war_machines(details: &HeroDetails, ruleset: &Ruleset) -> Vec<CreatureTypeId> {
    // rule: HER-WAR-001
    details
        .worn
        .iter()
        .filter_map(|item| ruleset.artifact(item.artifact).and_then(|a| a.war_machine))
        .collect()
}
