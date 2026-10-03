//! Why hero details are invalid.

use core::fmt;

use h3_content::{ArtifactSlotId, ArtifactTypeId, SkillId, SpellId};

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
    /// A spell is not in the ruleset.
    UnknownSpell(SpellId),
    /// A spell is listed twice.
    DuplicateSpell(SpellId),
    /// A spell is a creature ability, which never stands in a spell book.
    NotAHeroSpell(SpellId),
}

impl fmt::Display for DetailsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoHeroParams => f.write_str("the ruleset has no hero parameters"),
            Self::UnknownHeroType => f.write_str("unknown hero type"),
            Self::LevelZero => f.write_str("the level must be at least 1"),
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
            Self::UnknownSpell(id) => write!(f, "unknown spell {}", id.raw()),
            Self::DuplicateSpell(id) => write!(f, "spell {} listed twice", id.raw()),
            Self::NotAHeroSpell(id) => write!(f, "spell {} is a creature ability", id.raw()),
        }
    }
}

impl core::error::Error for DetailsError {}
