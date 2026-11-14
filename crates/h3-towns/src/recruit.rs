//! Recruitment at a town (E09 increment 3, design note §49 to §51; survey `E09-growth.md` §5, §8,
//! the original's `recruit_dialog_init_town` 0x551560 and `town_portal_hire` 0x5D2950): what a
//! building sells ([`recruit_source`]: a standing dwelling, horde or Portal of Summoning), its
//! creatures with their counts ([`recruit_stock`]), a purchase's town tests ([`check_recruit`])
//! and the town after it ([`recruit`]). Pure functions over a town's record; the game opens the
//! hire dialog, checks the room and the treasury, pays and joins the creatures (E09 T27,
//! `h3-game`'s `town::recruit`). TWN-REC-001, -002.

use alloc::vec::Vec;
use core::fmt;

use h3_content::CreatureTypeId;

use crate::building::{BuildingId, BuildingSet};
use crate::command::TownCommandError;
use crate::derive::Roles;
use crate::growth::index_creature;
use crate::record::{CREATURE_INDEXES, Town};
use crate::ruleset::{BuildingRoles, Dwelling, Horde, MAX_CREATURE_LEVELS, TownRuleset};

/// What a town's building sells (TWN-REC-001, -002): the creatures of one of its fourteen indexes
/// ([`crate::CREATURE_INDEXES`]: `level - 1` a level's, `level - 1 + 7` its upgrade's, and then the
/// base creature too, from the one count), or its Portal of Summoning's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecruitSource {
    /// A dwelling's or a horde's index.
    Index(u8),
    /// The Portal of Summoning.
    Portal,
}

/// Why a purchase failed a town's tests ([`check_recruit`], in this order); the game gives them as
/// the hire dialog's `NotOffered` and `Amount`, one vocabulary whatever the source (E09 design
/// §50, *agent's call*).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecruitError {
    /// The creature is not one the source sells.
    NotOffered(CreatureTypeId),
    /// The amount is 0 or above the source's count.
    Amount(u32),
}

impl fmt::Display for RecruitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotOffered(creature) => {
                write!(f, "creature {} is not the source's", creature.raw())
            }
            Self::Amount(amount) => write!(f, "{amount} is not an amount the source holds"),
        }
    }
}

impl core::error::Error for RecruitError {}

/// The index of a level, from 1, and its upgrade flag; none past the levels.
fn index_of(level: u8, upgraded: bool) -> Option<u8> {
    let levels = u8::try_from(MAX_CREATURE_LEVELS).unwrap_or(u8::MAX);
    let at = level.checked_sub(1).filter(|&at| at < levels)?;
    Some(at + u8::from(upgraded) * levels)
}

/// What `building` of `town` sells (TWN-REC-001, -002; E09 design §49, survey `E09-growth.md` §8,
/// the original's click paths into `recruit_dialog_init_town` 0x551560 and `town_portal_hire`
/// 0x5D2950), with its faction's record in `part`: a standing building (ever built, no ever-built
/// building superseding it) of role `dwelling` or `horde` sells its level's index (`level - 1`,
/// `+ 7` upgraded), of role `portal` the Portal. Refused, in this order:
///
/// - a building the record lacks ([`TownCommandError::UnknownBuilding`]);
/// - none of the three roles ([`TownCommandError::NotRecruiting`]);
/// - not standing, or a base index whose level's upgraded dwelling was ever built (its count was
///   moved to the upgrade's index, its growth is none: a base horde the upgraded horde did not
///   replace; [`TownCommandError::NotStanding`]);
/// - a Portal that holds nothing ([`TownCommandError::NothingSummoned`]: the original's message;
///   no draw, E09 decision 71; *agent's call:* tested here, with the record, rather than in the
///   game).
///
/// A count of 0 is no refusal: the original's dialog opens and its only answer is to close it.
///
/// # Errors
///
/// The refusals above; the town is unchanged in every case.
// rule: TWN-REC-001, TWN-REC-002
pub fn recruit_source(
    town: &Town,
    part: &TownRuleset,
    building: BuildingId,
) -> Result<RecruitSource, TownCommandError> {
    let roles = Roles::of(town, part);
    let of = roles
        .record()
        .and_then(|record| record.building(building))
        .ok_or(TownCommandError::UnknownBuilding)?;
    source_of(&roles, town, building, &of.roles)
}

/// The buildings of `town` whose recruitment opens a dialog, each [`recruit_source`] accepts,
/// with its faction's record in `part` (TWN-REC-001, -002; E09 T27, design §49: the legal
/// commands' recruitments, in the record's order): one pass over its buildings.
// rule: TWN-REC-001, TWN-REC-002
pub fn recruiting(town: &Town, part: &TownRuleset) -> BuildingSet {
    let roles = Roles::of(town, part);
    roles
        .standing_roles()
        .filter(|&(id, of)| source_of(&roles, town, id, of).is_ok())
        .fold(BuildingSet::EMPTY, |set, (id, _)| set.with(id))
}

/// [`recruit_source`]'s tests of `building` of `town`, read as `roles`, with its roles `of`.
fn source_of(
    roles: &Roles<'_>,
    town: &Town,
    building: BuildingId,
    of: &BuildingRoles,
) -> Result<RecruitSource, TownCommandError> {
    let level = match (of.dwelling, of.horde) {
        (Some(Dwelling { level, upgraded }), _) | (None, Some(Horde { level, upgraded })) => {
            Some((level, upgraded))
        }
        (None, None) if of.portal => None,
        (None, None) => return Err(TownCommandError::NotRecruiting),
    };
    if !roles.standing().contains(building) {
        return Err(TownCommandError::NotStanding);
    }
    let Some((level, upgraded)) = level else {
        return match town.portal() {
            Some(_) => Ok(RecruitSource::Portal),
            None => Err(TownCommandError::NothingSummoned),
        };
    };
    let moved = !upgraded
        && roles
            .built_roles()
            .any(|(_, r)| r.dwelling.is_some_and(|d| d.level == level && d.upgraded));
    match index_of(level, upgraded) {
        Some(index) if !moved => Ok(RecruitSource::Index(index)),
        _ => Err(TownCommandError::NotStanding),
    }
}

/// The creatures `source` of `town` sells, with their counts as they stand, in the dialog's order
/// (TWN-REC-001, -002; E09 design §50, survey `E09-growth.md` §8, the original's
/// `recruit_dialog_init_town` 0x551560: the upgrade selected first), with its faction's record in
/// `part`: a base index its creature; an upgraded index its upgrade, then its base creature,
/// **both with the index's one count**; the Portal its creature and count. Empty for a Portal
/// that holds nothing, an index past the fourteen or a level the record lacks.
// rule: TWN-REC-001, TWN-REC-002
pub fn recruit_stock(
    town: &Town,
    part: &TownRuleset,
    source: RecruitSource,
) -> Vec<(CreatureTypeId, u16)> {
    let index = match source {
        RecruitSource::Portal => {
            return town
                .portal()
                .map(|held| (held.creature, held.count))
                .into_iter()
                .collect();
        }
        RecruitSource::Index(index) => usize::from(index),
    };
    let record = part.town(town.faction());
    let Some(record) = record.filter(|_| index < CREATURE_INDEXES) else {
        return Vec::new();
    };
    let count = town.available(index);
    let base = index.checked_sub(MAX_CREATURE_LEVELS);
    [Some(index), base]
        .into_iter()
        .flatten()
        .map_while(|at| index_creature(record, at))
        .map(|creature| (creature, count))
        .collect()
}

/// Checks a purchase of `amount` `creature`s from `source` of `town` (TWN-REC-001, -002; E09
/// design §50), the town's part of it, before the game's room and treasury: the creature one of
/// [`recruit_stock`]'s ([`RecruitError::NotOffered`]), then the amount from 1 to its count
/// ([`RecruitError::Amount`]).
///
/// # Errors
///
/// The two refusals above.
// rule: TWN-REC-001, TWN-REC-002
pub fn check_recruit(
    town: &Town,
    part: &TownRuleset,
    source: RecruitSource,
    (creature, amount): (CreatureTypeId, u32),
) -> Result<(), RecruitError> {
    let stock = recruit_stock(town, part, source);
    let &(_, count) = stock
        .iter()
        .find(|&&(sold, _)| sold == creature)
        .ok_or(RecruitError::NotOffered(creature))?;
    if (1..=u32::from(count)).contains(&amount) {
        Ok(())
    } else {
        Err(RecruitError::Amount(amount))
    }
}

/// `town` after a purchase of `amount` creatures from `source` (TWN-REC-001, -002; E09 design §50,
/// §51), which [`check_recruit`] accepted: the index's one count lowered by the amount, whichever
/// of its creatures was bought; the Portal's count lowered, its creature kept until a draw sets it
/// again (the original's +0x3C stays). Saturating: a purchase never drives a count below 0.
// rule: TWN-REC-001, TWN-REC-002
pub fn recruit(town: &Town, source: RecruitSource, amount: u32) -> Town {
    let lowered = |count: u16| count.saturating_sub(u16::try_from(amount).unwrap_or(u16::MAX));
    match source {
        RecruitSource::Index(index) => {
            let index = usize::from(index);
            town.with_available(index, lowered(town.available(index)))
        }
        RecruitSource::Portal => town.with_portal(town.portal().map(|mut held| {
            held.count = lowered(held.count);
            held
        })),
    }
}

#[cfg(test)]
mod tests;
