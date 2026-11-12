//! A town's growth (E09 increment 3, design note §41, §46; survey `E09-growth.md` §1, §6): a
//! dwelling's growth a week (TWN-GRO-001) from the town and the inputs the game computes
//! ([`GrowthInputs`]), and a dwelling's first creatures when it is built (TWN-GRO-005). The week
//! and the month read them (`week.rs`, E09 T25b1), a neutral town's garrison week and the
//! Portal's draw too (T25b2).

use h3_content::{CreatureTypeId, Ruleset};
use h3_contracts::FortLevel;

use crate::building::BuildingSet;
use crate::derive::Roles;
use crate::record::{CREATURE_INDEXES, MAX_AVAILABLE, Town};
use crate::ruleset::{Dwelling, Horde, MAX_CREATURE_LEVELS, TownRecord, TownRuleset};
use crate::setup::indexed;
use crate::siege::fort_level;

/// What a town's growth reads beside the town (E09 design §41), computed by the game once per
/// town and step: its owner's share of the Statue of Legion, its heroes' Legion pieces, its owner's
/// dwellings on the map. A neutral town's are [`GrowthInputs::NEUTRAL`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GrowthInputs {
    /// Whether the town has an owner (the adventure's).
    pub owned: bool,
    /// The share a week more, in basis points of the base and the castle's: the highest
    /// `owner_town_growth` among the owner's heroes (the Statue of Legion's 5 000,
    /// `player_hero_wears_artifact` 0x4BACB0); read only for an owned town.
    pub share_bp: u16,
    /// What each level grows more, from level 1: the town's heroes' `town_growth`, each hero once
    /// (the Legion's pieces, `town_legion_growth`); read only for an owned town.
    pub legion: [u16; MAX_CREATURE_LEVELS],
    /// What each index grows more: the owner's dwellings on the map whose kind's first creature is
    /// the index's (a base creature's at its upgrade's index too; the original's +0x118).
    pub dwellings: [u16; CREATURE_INDEXES],
}

impl GrowthInputs {
    /// A neutral town's inputs: no owner, nothing more.
    pub const NEUTRAL: Self = Self {
        owned: false,
        share_bp: 0,
        legion: [0; MAX_CREATURE_LEVELS],
        dwellings: [0; CREATURE_INDEXES],
    };
}

/// The level, from 1, and whether it is the upgrade, of a creature index ([`CREATURE_INDEXES`]).
pub(crate) fn level_of(index: usize) -> Option<(u8, bool)> {
    (index < CREATURE_INDEXES).then(|| {
        let level = index % MAX_CREATURE_LEVELS;
        (
            u8::try_from(level + 1).unwrap_or(u8::MAX),
            index >= MAX_CREATURE_LEVELS,
        )
    })
}

/// The creature of `record` at `index`: index `level - 1` the level's creature, `level - 1 + 7`
/// its upgrade; none for a level the record lacks or a level without an upgrade.
pub fn index_creature(record: &TownRecord, index: usize) -> Option<CreatureTypeId> {
    let (level, upgraded) = level_of(index)?;
    let level = record.creatures.get(usize::from(level) - 1)?;
    if upgraded {
        level.upgrade
    } else {
        Some(level.base)
    }
}

/// What the dwelling at `index` of `town` grows a week (TWN-GRO-001; E09 design §41, survey
/// `E09-growth.md` §1, the original's `town_dwelling_growth`), with the town's faction's record in
/// `part`, the creatures' growths in `core` and the game's `inputs`, summed in the original's
/// order and held at [`MAX_AVAILABLE`]:
///
/// 1. none without the index's dwelling ever built (a building of role `dwelling` of its level
///    and upgrade flag), and none at a base index whose level's upgraded dwelling is ever built
///    (the upgrade's index grows then);
/// 2. g, the index's creature's core `growth`;
/// 3. the castle's: + g with a standing fort level 3, + g ÷ 2 with level 2;
/// 4. for an owned town: + (g + the castle's) × `share_bp` ÷ 10 000 (truncated: the Statue's
///    5 000 is the original's ÷ 2), + the level's `legion`;
/// 5. + the index's creature's `horde_growth` with a horde of the index ever built;
/// 6. + the index's `dwellings`;
/// 7. with a `grail` building ever built, + the sum so far ÷ 2 (truncated).
///
/// 0 for a faction without a record in `part`, an index past [`CREATURE_INDEXES`] or a creature
/// the core lacks.
// rule: TWN-GRO-001
pub fn growth(
    town: &Town,
    part: &TownRuleset,
    core: &Ruleset,
    index: usize,
    inputs: &GrowthInputs,
) -> u16 {
    let roles = Roles::of(town, part);
    let (Some(record), Some((level, upgraded))) = (roles.record(), level_of(index)) else {
        return 0;
    };
    let ever =
        |pick: &dyn Fn(&crate::BuildingRoles) -> bool| roles.built_roles().any(|(_, r)| pick(r));
    let dwelling = |upgraded| Some(Dwelling { level, upgraded });
    if !ever(&|r| r.dwelling == dwelling(upgraded))
        || (!upgraded && ever(&|r| r.dwelling == dwelling(true)))
    {
        return 0;
    }
    let Some(info) = index_creature(record, index).and_then(|c| core.creature(c)) else {
        return 0;
    };
    let g = u64::from(info.growth);
    let castle = match fort_level(town, part) {
        FortLevel::Castle => g,
        FortLevel::Citadel => g / 2,
        FortLevel::None | FortLevel::Fort => 0,
    };
    let mut sum = g + castle;
    if inputs.owned {
        let legion = inputs.legion[usize::from(level) - 1];
        sum += (g + castle) * u64::from(inputs.share_bp) / 10_000 + u64::from(legion);
    }
    if ever(&|r| r.horde == Some(Horde { level, upgraded })) {
        sum += u64::from(info.horde_growth);
    }
    sum += u64::from(inputs.dwellings[index]);
    if ever(&|r| r.grail) {
        sum += sum / 2;
    }
    u16::try_from(sum.min(u64::from(MAX_AVAILABLE))).unwrap_or(MAX_AVAILABLE)
}

/// `town` with the first creatures of the dwellings among `new`, buildings just made ever built
/// (TWN-GRO-005; E09 design §46, §40: the original's `town_build`, in id order), of `record` (its
/// faction's), the growths in `core`: each base dwelling first, in index order, sets its index's
/// count to its creature's growth (the count before overwritten); then each upgraded one sets its
/// index's count to its base index's and the base index's to 0. A horde adds nothing at once; a
/// creature the core lacks gives 0.
// rule: TWN-GRO-005
pub(crate) fn first_creatures(
    town: Town,
    record: &TownRecord,
    core: &Ruleset,
    new: BuildingSet,
) -> Town {
    let dwellings = |upgraded: bool| {
        indexed(record)
            .filter(move |(id, _)| new.contains(*id))
            .filter_map(move |(_, b)| b.roles.dwelling.filter(|d| d.upgraded == upgraded))
            .map(|d| usize::from(d.level) - 1)
    };
    let base = dwellings(false).fold(town, |town, index| {
        let growth = index_creature(record, index)
            .and_then(|c| core.creature(c))
            .map_or(0, |c| c.growth);
        town.with_available(index, growth)
    });
    dwellings(true).fold(base, |town, index| {
        town.with_available(index + MAX_CREATURE_LEVELS, town.available(index))
            .with_available(index, 0)
    })
}

#[cfg(test)]
mod tests;
