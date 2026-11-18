//! A player's management of its heroes on the map (ADR-0017 points 1 and 6, GAM-MAN-001; E08 T51a,
//! T51b; increment 6's design note §10 in `E08-log.md`): the army commands of the adventure
//! (ADR-0016 point 3), the formation and the tactics flag, and the artifact commands of `h3-heroes`
//! (ADR-0013 point 14), and since E09 T38b2 a hero's dismissal (GAM-MAN-003). The game checks the actor, its turn and its open decision, which armies may
//! meet (two heroes in their meeting, a hero and the group in its army dialog), then the context's
//! rules, before anything changes; the commands themselves apply in `state/manage.rs`, the per-hero
//! listing is [`crate::legal_management`].

use core::fmt;

use h3_adventure::{ArmyCommand, ArmyCommandError};
use h3_content::Formation;
use h3_contracts::HeroId;
use h3_heroes::{ArtifactPlace, CombineError, MoveError, Position};

/// A management command (GAM-MAN-001): what a player does to its heroes on the map at no cost,
/// any number of times in its turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Manage {
    /// An army command (ADV-ARM-001 to -005): a stack moved, split or dismissed.
    Army(ArmyCommand),
    /// Sets a hero's formation (ADV-ARM-006).
    Formation {
        /// The hero.
        hero: HeroId,
        /// The formation.
        formation: Formation,
    },
    /// Sets a hero's tactics flag (ADV-ARM-006; GAM-BAT-009 reads it).
    Tactics {
        /// The hero.
        hero: HeroId,
        /// Whether the flag is on.
        on: bool,
    },
    /// Moves an artifact from a place of a hero to a place of the same hero, or of the hero it
    /// meets (HER-ART-005): a worn position or a backpack index, the target's artifact swapped
    /// back.
    Artifact {
        /// The hero and the place the artifact is taken from.
        from: (HeroId, ArtifactPlace),
        /// The hero and the place it goes to.
        to: (HeroId, ArtifactPlace),
    },
    /// Assembles the combination the artifact worn at `at` completes (HER-ART-006).
    Assemble {
        /// The hero.
        hero: HeroId,
        /// The position of a part.
        at: Position,
    },
    /// Disassembles the combination worn at `at` (HER-ART-006).
    Disassemble {
        /// The hero.
        hero: HeroId,
        /// The combination's position.
        at: Position,
    },
    /// Dismisses a hero (GAM-MAN-003; E09 T38b2, the town lane's design note `E09-design.md`
    /// §69): it leaves the game with every artifact it holds, its type free in the hero pool
    /// (TWN-TAV-006). Only outside a decision.
    Dismiss {
        /// The hero.
        hero: HeroId,
    },
}

/// Why a management command was rejected; nothing changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManageError {
    /// A hero the command names is not the player's on the map.
    NotYourHero(HeroId),
    /// The command names two armies (two heroes, or a hero and an army dialog's group), and no
    /// meeting or army dialog holds them together (ADR-0017 point 6).
    NotTogether,
    /// The adventure refused the army command.
    Army(ArmyCommandError),
    /// The hero model refused the artifact's move.
    Artifact(MoveError),
    /// The hero model refused the assembly or the disassembly.
    Combine(CombineError),
    /// The hero to dismiss is the player's garrison hero, not on the map (GAM-MAN-003: the town
    /// screen offers no dismissal).
    NotOnMap,
    /// The hero to dismiss visits a town, standing on its entrance (GAM-MAN-003, TD-22).
    InTown,
    /// The hero to dismiss is the player's one hero on the map, and it owns no town (GAM-MAN-003).
    LastHero,
}

impl ManageError {
    /// The adventure's refusal, a hero not the player's named as the game names it.
    pub(crate) const fn of_army(error: ArmyCommandError) -> Self {
        match error {
            ArmyCommandError::NotYourHero(hero) => Self::NotYourHero(hero),
            error => Self::Army(error),
        }
    }
}

impl fmt::Display for ManageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotYourHero(hero) => {
                write!(f, "hero {} is not the player's on the map", hero.raw())
            }
            Self::NotTogether => f.write_str("the two armies are not together"),
            Self::Army(error) => write!(f, "{error}"),
            Self::Artifact(error) => write!(f, "{error}"),
            Self::Combine(error) => write!(f, "{error}"),
            Self::NotOnMap => f.write_str("the hero is in a town's garrison, not on the map"),
            Self::InTown => f.write_str("the hero visits a town"),
            Self::LastHero => {
                f.write_str("the hero is the player's last on the map, and it owns no town")
            }
        }
    }
}

impl core::error::Error for ManageError {}
