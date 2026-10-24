//! Artifact moves (E08 T49b; ADR-0013 point 14.1, HER-ART-005): an artifact taken from a place of
//! a hero to a place of the same hero or of the hero it meets, checked on both before either
//! changes and given as the hero changes each applies (ADR-0013 point 14.4).

use alloc::vec;
use alloc::vec::Vec;

use h3_content::{ArtifactType, CreatureFlag, Ruleset};

use crate::change::{ArtifactPlace, HeroChange};
use crate::details::{ArtifactInstance, HeroDetails};
use crate::error::MoveError;
use crate::positions::{Position, full_slot, positions};

/// A move of an artifact (HER-ART-005): from where it is to where it goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtifactMove {
    /// Where the artifact is: a worn position or a backpack index of the moving hero.
    pub from: ArtifactPlace,
    /// Where it goes: a position, or a backpack index from 0 to the backpack's length, the
    /// artifact inserted there (within one backpack, its length without the moved artifact).
    pub to: ArtifactPlace,
}

/// The hero changes of a move ([`move_changes`]), each checked to apply in order
/// ([`crate::check_change`]): the moving hero's, and the receiving hero's for a move between two
/// heroes. A move within one hero has all its changes in `from`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveChanges {
    /// The moving hero's changes: the artifact removed, then the target's artifact put back in
    /// its place (a swap).
    pub from: Vec<HeroChange>,
    /// The receiving hero's changes: the target's artifact removed, then the artifact placed.
    pub to: Vec<HeroChange>,
}

/// What a checked move carries: the moved artifact and the target's, which goes back.
#[derive(Clone, Copy)]
struct Plan {
    item: ArtifactInstance,
    displaced: Option<ArtifactInstance>,
}

/// Checks a move of an artifact of `from` to `to` (the other hero, `None` within one hero;
/// HER-ART-005): the source holds an artifact that may move (not the Spell Book nor the
/// Catapult, [`unmovable`]) and the target differs from it; a position target is of the
/// artifact's slot with the locks fitting there once the target's artifact is gone (HER-ART-001),
/// and that artifact, if any, goes back to the source's place (a position where the locks fit
/// too, or the backpack index, inserted); a backpack target is an index up to the backpack's
/// length, for an artifact the backpack carries (HER-ART-002: not a war machine, which moves
/// between machine positions only; the Grail moves between backpacks only) while it holds fewer
/// than the ruleset's most (a move within one backpack does not count itself). The details are
/// valid. The original's swap through the held artifact, as one move (survey
/// `docs/rules/surveys/E08-army.md` §5.2: `hero_screen_proc` 0x4DD2D0, the exchange's
/// `swap_worn_click` 0x5AF590).
///
/// # Errors
/// [`MoveError::NoArtifact`] for an empty source, [`MoveError::Unmovable`] for the Spell Book or
/// the Catapult there or at the target, [`MoveError::SamePlace`] for a target that is the source,
/// [`MoveError::CannotPlace`] for a target that refuses the artifact, [`MoveError::CannotReturn`]
/// for a source whose locks refuse the target's artifact.
pub fn check_move(
    from: &HeroDetails,
    to: Option<&HeroDetails>,
    ruleset: &Ruleset,
    move_: ArtifactMove,
) -> Result<(), MoveError> {
    plan(from, to, ruleset, move_).map(|_| ())
}

/// The hero changes of a move [`check_move`] passes (HER-ART-005): within one hero the artifact
/// removed, the target's artifact removed, the artifact placed, the target's put at the source's
/// place; between two heroes the moving hero's removal and return, the receiving hero's removal
/// and placement. A worn placement equips (HER-ART-007: Titan's Thunder adds a Spell Book).
///
/// # Errors
/// [`check_move`]'s.
pub fn move_changes(
    from: &HeroDetails,
    to: Option<&HeroDetails>,
    ruleset: &Ruleset,
    move_: ArtifactMove,
) -> Result<MoveChanges, MoveError> {
    let plan = plan(from, to, ruleset, move_)?;
    let removed = plan.displaced.map(|_| HeroChange::RemoveArtifact(move_.to));
    let placed = HeroChange::PlaceArtifact {
        artifact: plan.item,
        at: move_.to,
    };
    let returned = plan.displaced.map(|artifact| HeroChange::PlaceArtifact {
        artifact,
        at: move_.from,
    });
    let left = HeroChange::RemoveArtifact(move_.from);
    Ok(match to {
        None => MoveChanges {
            from: [Some(left), removed, Some(placed), returned]
                .into_iter()
                .flatten()
                .collect(),
            to: vec![],
        },
        Some(_) => MoveChanges {
            from: [Some(left), returned].into_iter().flatten().collect(),
            to: [removed, Some(placed)].into_iter().flatten().collect(),
        },
    })
}

/// [`check_move`], with the artifacts that move.
fn plan(
    from: &HeroDetails,
    to: Option<&HeroDetails>,
    ruleset: &Ruleset,
    move_: ArtifactMove,
) -> Result<Plan, MoveError> {
    // rule: HER-ART-005
    let (item, artifact) = at(from, move_.from)
        .and_then(|item| ruleset.artifact(item.artifact).map(|a| (item, a)))
        .ok_or(MoveError::NoArtifact(move_.from))?;
    if unmovable(ruleset, artifact) {
        return Err(MoveError::Unmovable(item.artifact));
    }
    if to.is_none() && move_.from == move_.to {
        return Err(MoveError::SamePlace);
    }
    let same = to.is_none();
    let target = to.unwrap_or(from);
    match move_.to {
        ArtifactPlace::Worn(q) => {
            let slot = positions(ruleset).get(q.0).copied();
            if slot.is_none() || slot != artifact.slot {
                return Err(MoveError::CannotPlace(move_.to));
            }
            let displaced = at(target, move_.to);
            let fixed = displaced.filter(|other| {
                ruleset
                    .artifact(other.artifact)
                    .is_some_and(|a| unmovable(ruleset, a))
            });
            if let Some(other) = fixed {
                return Err(MoveError::Unmovable(other.artifact));
            }
            // Within one hero a worn source leaves its position too, and the target's artifact
            // comes back to it: a swap keeps the set.
            let source = match move_.from {
                ArtifactPlace::Worn(p) if same => Some(p),
                _ => None,
            };
            let returning = displaced.filter(|_| source.is_some());
            let fits = locks_fit(
                ruleset,
                &target.worn,
                &[Some(q), source],
                [
                    Some(artifact),
                    returning.and_then(|i| ruleset.artifact(i.artifact)),
                ],
            );
            if !fits {
                return Err(MoveError::CannotPlace(move_.to));
            }
            // Between two heroes the target's artifact goes back to the source's position, where
            // the moving hero's locks must take it (its slot is the source's: both are the
            // artifact's).
            let back = match (move_.from, displaced) {
                (ArtifactPlace::Worn(p), Some(other)) if !same => locks_fit(
                    ruleset,
                    &from.worn,
                    &[Some(p), None],
                    [ruleset.artifact(other.artifact), None],
                ),
                _ => true,
            };
            if !back {
                return Err(MoveError::CannotReturn(move_.from));
            }
            Ok(Plan { item, displaced })
        }
        ArtifactPlace::Backpack(j) => {
            let left = usize::from(same && matches!(move_.from, ArtifactPlace::Backpack(_)));
            let length = target.backpack.len().saturating_sub(left);
            let most = ruleset
                .hero_params()
                .map_or(0, |params| usize::from(params.backpack));
            let fits = carried(ruleset, artifact) && j <= length && length < most;
            fits.then_some(Plan {
                item,
                displaced: None,
            })
            .ok_or(MoveError::CannotPlace(move_.to))
        }
    }
}

/// The artifact at `place`, if there is one.
fn at(details: &HeroDetails, place: ArtifactPlace) -> Option<ArtifactInstance> {
    match place {
        ArtifactPlace::Worn(p) => details.worn.get(p.0).copied().flatten(),
        ArtifactPlace::Backpack(i) => details.backpack.get(i).copied(),
    }
}

/// Whether the worn artifacts but those at `gone`, with `added`, fit every slot's count less the
/// locks (HER-ART-001).
fn locks_fit<'a>(
    ruleset: &'a Ruleset,
    worn: &[Option<ArtifactInstance>],
    gone: &[Option<Position>; 2],
    added: [Option<&'a ArtifactType>; 2],
) -> bool {
    let kept = (0..)
        .zip(worn)
        .filter(|&(i, _)| !gone.contains(&Some(Position(i))))
        .filter_map(|(_, item)| item.and_then(|item| ruleset.artifact(item.artifact)));
    full_slot(ruleset, kept.chain(added.into_iter().flatten())).is_none()
}

/// Whether an artifact never moves (HER-ART-005): it belongs to a slot the backpack does not
/// carry (HER-ART-002) and places no war machine but the Catapult or none at all: the Spell Book
/// and the Catapult. The war machines that place a Ballista, an Ammo Cart or a First Aid Tent
/// move between machine positions. The original refuses to pick up positions 16 and 17 ("The
/// Catapult must be equipped.", GENRLTXT 313; "This item can't be traded.", GENRLTXT 22); VCMI's
/// `ArtifactUtils::unmovableSlots` are the spell book and the fourth machine.
fn unmovable(ruleset: &Ruleset, artifact: &ArtifactType) -> bool {
    let catapult = |machine| {
        ruleset
            .creature(machine)
            .is_some_and(|c| c.flags.contains(CreatureFlag::Catapult))
    };
    !carried(ruleset, artifact) && artifact.war_machine.is_none_or(catapult)
}

/// Whether the backpack carries `artifact`: it has no slot (the Grail) or its slot's artifacts
/// may go there (HER-ART-002).
fn carried(ruleset: &Ruleset, artifact: &ArtifactType) -> bool {
    artifact
        .slot
        .and_then(|s| ruleset.artifact_slot(s))
        .is_none_or(|slot| slot.backpack)
}
