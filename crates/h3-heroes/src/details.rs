//! `HeroDetails`, their validation (HER-DET-001, HER-SEC-001, HER-ART-001 to 003, HER-SPL-001)
//! and the war machines a hero brings to a battle (HER-WAR-001).

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use h3_content::{
    ArtifactType, ArtifactTypeId, CreatureTypeId, HeroEffect, HeroParams, HeroTypeId,
    MAX_HERO_SKILLS, MAX_SKILL_LEVEL, Ruleset, SkillId, SpellCategory, SpellId,
};

use crate::error::DetailsError;
use crate::level::{experience_bound, level_of};
use crate::positions::{full_slot, positions};

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

/// The levels at which a level-up last offered Wisdom and a magic school (E08 T27, HER-LVL-004:
/// the original's +0x90 and +0x3F): the next is due once the hero's level reaches one of them
/// plus its class's interval. 0 at the start; at most the hero's level.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OfferLevels {
    /// The level at which Wisdom was last offered.
    pub wisdom: u8,
    /// The level at which a magic school was last offered.
    pub school: u8,
}

/// The bonuses a hero holds until the end of its next battle (E08 T60a; ADR-0013 point 1,
/// HER-ADV-008): the morale and luck that objects add (a Pandora's Box's and an event's, the
/// Warrior's Tomb's −3, a looted Pyramid's −2 luck), summed across objects, and the once-marks of
/// the bonuses an object gives a hero once until then. The original's +0x11A and +0x11B and the
/// status bits 21 (the Tomb) and 12 (the Pyramid), cleared together at a battle's end
/// (`hero_battle_bonuses_clear` 0x4DA510). Any value is valid.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BattleBonuses {
    /// The morale added to the hero's own side in its next battle, held saturating (the
    /// original's byte wraps past 127: A-9's class, fixed).
    pub morale: i8,
    /// The luck added to the hero's own side in its next battle, held saturating.
    pub luck: i8,
    /// The once-marks, one bit per kind of once-bonus ([`BattleBonuses::MARKS`] kinds): the
    /// object types name theirs in the adventure's content, so this crate knows no object.
    pub marks: u8,
}

impl BattleBonuses {
    /// How many kinds of once-mark a hero holds: the bits of [`BattleBonuses::marks`].
    pub const MARKS: u8 = 8;

    /// Whether the once-mark `mark` is set; `false` for a mark of no kind (8 and above).
    pub const fn has_mark(self, mark: u8) -> bool {
        mark < Self::MARKS && self.marks & (1 << mark) != 0
    }
}

/// The lasting details of a hero (ADR-0013 point 1): what the hero is between battles, apart
/// from its army. Values are *base* values; what artifacts and the specialty add is derived
/// ([`crate::derive_battle_hero`]).
///
/// Artifacts are worn on positions (E08 T49a; ADR-0013 point 1): one per position of the
/// ruleset's slots ([`crate::positions`]), an artifact on a position of its own slot, and a
/// combination's locks counted per slot (HER-ART-001).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeroDetails {
    /// The hero type: class, specialty and identity.
    pub hero_type: HeroTypeId,
    /// Level (at least 1): the levels given so far, at most the level of the experience
    /// ([`crate::level_of`]). Fewer are the hero's pending level-ups ([`crate::pending_levels`],
    /// E08 T27), which the game gives one at a time.
    pub level: u8,
    /// Experience points, at most the ruleset's bound ([`crate::experience_bound`], HER-EXP-002).
    pub experience: u32,
    /// The levels at which Wisdom and a school were last offered (E08 T27, HER-LVL-004).
    pub offered: OfferLevels,
    /// Base primary skills ([`PrimarySkill`] order), without artifacts; at most the ruleset's
    /// `primary_max`. A value below the ruleset's minimum is allowed and counts as the minimum
    /// (a map may set Knowledge 0).
    pub primary: [u8; 4],
    /// Secondary skills with their levels (1 Basic to 3 Expert), in display order.
    pub skills: Vec<(SkillId, u8)>,
    /// Worn artifacts by position ([`crate::Position`]): one entry per position of
    /// [`crate::positions`], empty or holding an artifact of that position's slot; the same type
    /// may appear more than once. [`crate::wearing`] places a list.
    pub worn: Vec<Option<ArtifactInstance>>,
    /// Artifacts carried in the backpack, in backpack order.
    pub backpack: Vec<ArtifactInstance>,
    /// Spells the hero has learned (the original's `+0x3EA`): kept whatever it wears. The spells
    /// it can cast add those of its worn artifacts ([`crate::available_spells`], HER-SPL-002)
    /// and need a worn spell book (HER-BAT-011).
    pub spells: Vec<SpellId>,
    /// Spell points (E04b). May exceed the maximum (a Mana Vortex), so it is not bounded here.
    pub mana: u16,
    /// The morale and luck bonuses and once-marks until the end of the next battle (E08 T60a,
    /// HER-ADV-008): none (the default) at a map's start and after a battle.
    pub bonuses: BattleBonuses,
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
    // rule: HER-EXP-002
    if details.experience > experience_bound(params) {
        return Err(DetailsError::ExperienceAboveBound);
    }
    if details.level > level_of(params, details.experience) {
        return Err(DetailsError::LevelAboveExperience);
    }
    // rule: HER-LVL-004
    if details.offered.wisdom.max(details.offered.school) > details.level {
        return Err(DetailsError::OfferedAboveLevel);
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

/// One entry per position; every worn artifact on a position of its own slot; the positions
/// they and their locks occupy fit every slot's count (HER-ART-001).
fn validate_worn(worn: &[Option<ArtifactInstance>], ruleset: &Ruleset) -> Result<(), DetailsError> {
    // rule: HER-ART-001
    let slots = positions(ruleset);
    if worn.len() != slots.len() {
        return Err(DetailsError::Positions);
    }
    let mut types = Vec::new();
    for (&slot, &item) in slots.iter().zip(worn) {
        let Some(item) = item else { continue };
        let id = item.artifact;
        let artifact = ruleset
            .artifact(id)
            .ok_or(DetailsError::UnknownArtifact(id))?;
        let own = artifact.slot.ok_or(DetailsError::NotWearable(id))?;
        if own != slot {
            return Err(DetailsError::WrongPosition(id));
        }
        validate_scroll(item, ruleset)?;
        types.push(artifact);
    }
    match full_slot(ruleset, types.into_iter()) {
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
pub(crate) fn hero_spell(id: SpellId, ruleset: &Ruleset) -> Result<(), DetailsError> {
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
/// every worn artifact that places one, in position order. The Catapult's artifact places none:
/// the battle sets the Catapult up itself (BAT-SIE-009). Artifacts the ruleset does not know are
/// skipped; validate first.
pub fn war_machines(details: &HeroDetails, ruleset: &Ruleset) -> Vec<CreatureTypeId> {
    // rule: HER-WAR-001
    details
        .worn
        .iter()
        .flatten()
        .filter_map(|item| ruleset.artifact(item.artifact).and_then(|a| a.war_machine))
        .collect()
}
