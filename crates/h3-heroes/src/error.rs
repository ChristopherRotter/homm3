//! Why hero details are invalid, and why a change does not fit them.

use core::fmt;

use h3_content::{ArtifactSlotId, ArtifactTypeId, SkillId, SpellId};

use crate::change::ArtifactPlace;
use crate::details::PrimarySkill;

/// A reason why hero details do not fit a ruleset ([`crate::validate`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailsError {
    /// The ruleset has no hero parameters, so it cannot validate hero details.
    NoHeroParams,
    /// The hero type is not in the ruleset.
    UnknownHeroType,
    /// The level is 0 (heroes start at level 1).
    LevelZero,
    /// The experience exceeds the ruleset's bound (HER-EXP-002: level 75's threshold - 1).
    ExperienceAboveBound,
    /// The level exceeds the level of the experience (HER-EXP-002).
    LevelAboveExperience,
    /// A level at which Wisdom or a school was last offered exceeds the level (HER-LVL-004).
    OfferedAboveLevel,
    /// A base primary skill exceeds the ruleset's limit.
    PrimaryAboveLimit(PrimarySkill),
    /// More secondary skills than a hero can have.
    TooManySkills,
    /// A secondary skill is not in the ruleset.
    UnknownSkill(SkillId),
    /// A secondary skill is listed twice.
    DuplicateSkill(SkillId),
    /// A secondary skill's level is not Basic (1) to Expert (3).
    SkillLevel(SkillId),
    /// An artifact is not in the ruleset.
    UnknownArtifact(ArtifactTypeId),
    /// A worn artifact has no slot (it is carried in the backpack only, like the Grail).
    NotWearable(ArtifactTypeId),
    /// The worn artifacts and their locks need more positions of a slot than it has.
    SlotFull(ArtifactSlotId),
    /// More artifacts in the backpack than it holds.
    BackpackFull,
    /// An artifact in the backpack belongs to a slot whose artifacts never go there (war
    /// machines, the spell book).
    NotForBackpack(ArtifactTypeId),
    /// A scroll (an artifact with the `scroll` effect) holds no spell (HER-ART-003).
    ScrollWithoutSpell(ArtifactTypeId),
    /// An artifact that is not a scroll holds a spell (HER-ART-003).
    SpellOnArtifact(ArtifactTypeId),
    /// A spell is not in the ruleset.
    UnknownSpell(SpellId),
    /// A spell is listed twice.
    DuplicateSpell(SpellId),
    /// A spell is a creature ability, which never stands in a spell book.
    NotAHeroSpell(SpellId),
    /// The details are valid, but their battle effects exceed the seam's limit
    /// (`h3_contracts::MAX_BATTLE_EFFECTS`; HER-BAT-008).
    TooManyBattleEffects,
}

impl fmt::Display for DetailsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoHeroParams => f.write_str("the ruleset has no hero parameters"),
            Self::UnknownHeroType => f.write_str("unknown hero type"),
            Self::LevelZero => f.write_str("the level must be at least 1"),
            Self::ExperienceAboveBound => f.write_str("the experience exceeds the highest level's"),
            Self::LevelAboveExperience => f.write_str("the level exceeds the experience's"),
            Self::OfferedAboveLevel => f.write_str("an offer's level exceeds the level"),
            Self::PrimaryAboveLimit(skill) => {
                write!(f, "{} above the ruleset's limit", skill.name())
            }
            Self::TooManySkills => f.write_str("too many secondary skills"),
            Self::UnknownSkill(id) => write!(f, "unknown secondary skill {}", id.raw()),
            Self::DuplicateSkill(id) => write!(f, "secondary skill {} listed twice", id.raw()),
            Self::SkillLevel(id) => write!(f, "secondary skill {} needs level 1..=3", id.raw()),
            Self::UnknownArtifact(id) => write!(f, "unknown artifact {}", id.raw()),
            Self::NotWearable(id) => write!(f, "artifact {} has no slot", id.raw()),
            Self::SlotFull(id) => write!(f, "artifact slot {} has too few positions", id.raw()),
            Self::BackpackFull => f.write_str("too many artifacts in the backpack"),
            Self::NotForBackpack(id) => {
                write!(f, "artifact {} cannot be carried in the backpack", id.raw())
            }
            Self::ScrollWithoutSpell(id) => write!(f, "scroll {} holds no spell", id.raw()),
            Self::SpellOnArtifact(id) => {
                write!(f, "artifact {} is no scroll and holds no spell", id.raw())
            }
            Self::UnknownSpell(id) => write!(f, "unknown spell {}", id.raw()),
            Self::DuplicateSpell(id) => write!(f, "spell {} listed twice", id.raw()),
            Self::NotAHeroSpell(id) => write!(f, "spell {} is a creature ability", id.raw()),
            Self::TooManyBattleEffects => f.write_str("too many battle effects"),
        }
    }
}

impl core::error::Error for DetailsError {}

/// A reason why a change does not fit hero details ([`crate::check_change`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeError {
    /// The received artifact, or the spell it holds, does not fit the ruleset (HER-ART-003).
    Artifact(DetailsError),
    /// The received artifact fits no free position, and the backpack cannot take it.
    NoRoom(ArtifactTypeId),
    /// There is no artifact at the place.
    NoArtifact(ArtifactPlace),
    /// The learned spell is not a hero spell of the ruleset (HER-SPL-001).
    Spell(DetailsError),
    /// The hero has learned the spell already.
    KnownSpell(SpellId),
    /// A level is gained, but none is pending: the level is the experience's (HER-LVL-005).
    NoLevelPending,
    /// A level's learned skill is not the ruleset's, is at Expert, or is new to a hero who knows
    /// the most skills (HER-LVL-005).
    CannotLearn(SkillId),
    /// A level sets an offer's level above the new level (HER-LVL-005).
    OfferedAboveLevel,
}

impl fmt::Display for ChangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Artifact(error) => write!(f, "the artifact does not fit the ruleset: {error}"),
            Self::NoRoom(id) => write!(f, "no room for artifact {}", id.raw()),
            Self::NoArtifact(ArtifactPlace::Worn(i)) => write!(f, "no artifact worn at {i}"),
            Self::NoArtifact(ArtifactPlace::Backpack(i)) => {
                write!(f, "no artifact in the backpack at {i}")
            }
            Self::Spell(error) => write!(f, "the spell does not fit the ruleset: {error}"),
            Self::KnownSpell(id) => write!(f, "spell {} is learned already", id.raw()),
            Self::NoLevelPending => f.write_str("no level is pending"),
            Self::CannotLearn(id) => write!(f, "secondary skill {} cannot be learned", id.raw()),
            Self::OfferedAboveLevel => f.write_str("an offer's level exceeds the new level"),
        }
    }
}

impl core::error::Error for ChangeError {}
