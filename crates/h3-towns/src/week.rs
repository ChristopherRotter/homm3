//! The week's and the month's growth of a town (E09 increment 3, design note §42, §43; survey
//! `E09-growth.md` §2, §3): the week's kind ([`WeekKind`], the town crate's type, held by the
//! game beside its month's kind), what a week adds to a town's counts (TWN-GRO-003) and what a
//! month's kind does to them ([`MonthGrowth`], TWN-GRO-004). Pure functions over one town; the
//! game draws the kinds (TWN-GRO-002, GAM-CAL-003), computes each town's [`GrowthInputs`] and
//! calls them in `TownId` order (E09 T26a).

use h3_content::{CREATURE_MONTH_EXACT, CreatureTypeId, NEUTRAL_PLAGUE_FULL_GROWTH, Ruleset};
use h3_kernel::{CanonicalHash, StateHasher};

use crate::growth::{GrowthInputs, growth, index_creature, level_of};
use crate::record::{CREATURE_INDEXES, MAX_AVAILABLE, Town};
use crate::ruleset::{TownRecord, TownRuleset};

/// The week's kind (E09 design §42; the original's +0x1F63C kind with its two creatures and
/// bonus, `new_day_week_events` 0x4C8780): what a dwelling of the week's creatures grows more.
/// The game draws it on a week's first day (TWN-GRO-002) and keeps it beside its month's kind;
/// the game's first week is [`WeekKind::Normal`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeekKind {
    /// A normal week: no creature grows more.
    #[default]
    Normal,
    /// A creature's week (the original's kind 1): a dwelling of that creature alone grows the
    /// part's [`crate::WeekParams::creature_bonus`] more (the original's 5).
    Creature(CreatureTypeId),
    /// A building's week (the original's kind 2, the Deity of Fire's; a building of role
    /// `week_of`): a dwelling of either of its level's creatures grows `bonus` more (the level's
    /// base creature's core growth, the Imp's 15).
    Of {
        /// The level's base creature and its upgrade.
        creatures: [Option<CreatureTypeId>; 2],
        /// What a dwelling of either grows more.
        bonus: u16,
    },
}

impl WeekKind {
    /// Whether `creature` is one of the week's: a creature's week its creature, a building's
    /// either of its level's.
    pub fn names(&self, creature: CreatureTypeId) -> bool {
        match self {
            Self::Normal => false,
            Self::Creature(c) => *c == creature,
            Self::Of { creatures, .. } => creatures.contains(&Some(creature)),
        }
    }

    /// What a dwelling of one of the week's creatures grows more, with `part`'s parameters (a
    /// part without parameters: a creature's week none).
    pub fn bonus(&self, part: &TownRuleset) -> u16 {
        match self {
            Self::Normal => 0,
            Self::Creature(_) => part.params().map_or(0, |p| p.week.creature_bonus),
            Self::Of { bonus, .. } => *bonus,
        }
    }
}

impl CanonicalHash for WeekKind {
    /// 0 a normal week; 1 a creature's with its id; 2 a building's with its two creatures (each
    /// an option) and its bonus (E09 design §42).
    fn hash_into(&self, h: &mut StateHasher) {
        match self {
            Self::Normal => h.write_u8(0),
            Self::Creature(creature) => {
                h.write_u8(1);
                creature.hash_into(h);
            }
            Self::Of { creatures, bonus } => {
                h.write_u8(2);
                for creature in creatures {
                    creature.hash_into(h);
                }
                h.write_u16(*bonus);
            }
        }
    }
}

/// What a month's kind does to the towns' counts (E09 design §43, TWN-GRO-004): the game maps
/// its own month's kind (GAM-CAL-003) to it one to one. *Agent's call* (E09 T25b1, change
/// freely): the town crate's own type, as the month's kind stays the game's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MonthGrowth {
    /// A normal month: the counts as they are.
    #[default]
    Normal,
    /// A creature's month: the counts of that creature double.
    Creature(CreatureTypeId),
    /// The Plague: every growing count loses its growth, then halves.
    Plague,
}

/// `town`'s counts after each index's `f(count, growth)`, for the indexes whose growth (TWN-GRO-001
/// with `inputs`) is above 0, each held at [`MAX_AVAILABLE`]; the rest of the town kept (the
/// original's loops over the indexes, `town_week_growth` 0x5BFDD0 and `new_month` 0x4C8F40).
fn each_growing(
    town: &Town,
    part: &TownRuleset,
    core: &Ruleset,
    inputs: &GrowthInputs,
    f: impl Fn(usize, u32, u32) -> u32,
) -> Town {
    (0..CREATURE_INDEXES).fold(town.clone(), |next, index| {
        let g = growth(town, part, core, index, inputs);
        if g == 0 {
            return next;
        }
        let count = f(index, u32::from(town.available(index)), u32::from(g));
        next.with_available(
            index,
            u16::try_from(count.min(u32::from(MAX_AVAILABLE))).unwrap_or(MAX_AVAILABLE),
        )
    })
}

/// `town` after a week's growth (TWN-GRO-003; E09 design §42, survey `E09-growth.md` §2, the
/// original's `town_week_growth` 0x5BFDD0) in the week of `kind`, with its faction's record in
/// `part`, the growths in `core` and the game's `inputs`: each index whose growth g is above 0
/// gains g, + the week's [`WeekKind::bonus`] when the index's creature is one of the week's
/// ([`WeekKind::names`]), halved (truncated, after the bonus) in a town without an owner, the
/// count held at [`MAX_AVAILABLE`] (`TWN-QUIRK-8`). The Portal is untouched; a faction without a
/// record in `part` grows nothing.
// rule: TWN-GRO-003
pub fn week_growth(
    town: &Town,
    part: &TownRuleset,
    core: &Ruleset,
    kind: &WeekKind,
    inputs: &GrowthInputs,
) -> Town {
    let Some(record) = part.town(town.faction()) else {
        return town.clone();
    };
    let bonus = u32::from(kind.bonus(part));
    each_growing(town, part, core, inputs, |index, count, g| {
        let named = index_creature(record, index).is_some_and(|c| kind.names(c));
        let added = if named { g + bonus } else { g };
        count + if inputs.owned { added } else { added / 2 }
    })
}

/// Whether a creature's month of `creature` doubles the count at `index` of `record`: with
/// `exact` only when the index's creature is the month's (the original's), else when the
/// month's is either of the index's level's creatures (VCMI 1.7.5's level rule, `TWN-QUIRK-10`
/// off).
fn doubles(record: &TownRecord, index: usize, creature: CreatureTypeId, exact: bool) -> bool {
    if exact {
        return index_creature(record, index) == Some(creature);
    }
    level_of(index)
        .and_then(|(level, _)| record.creatures.get(usize::from(level) - 1))
        .is_some_and(|l| l.base == creature || l.upgrade == Some(creature))
}

/// `town` after a month's start of `month` (TWN-GRO-004; E09 design §43, survey `E09-growth.md`
/// §3, the original's `new_month` 0x4C8F40 at its towns), after the week's growth of the
/// month's first week, of `week`, with its faction's record in `part`, the growths and the quirk
/// flags in `core` and the game's `inputs`; each index whose growth g (TWN-GRO-001, unhalved, no
/// week's bonus) is above 0:
///
/// - a creature's month of c, unless `week` is a building's ([`WeekKind::Of`]: the month is then
///   its level's base creature's, which spawns on the map and doubles no count): the count
///   doubles, held at [`MAX_AVAILABLE`] (`TWN-QUIRK-8`), when the index's creature is c; with
///   `quirk.town.creature_month_exact` off when c is either of the level's creatures
///   (`TWN-QUIRK-10`);
/// - the Plague: the count less g (at least 0), then halved (truncated); with
///   `quirk.town.neutral_plague_full_growth` off a town without an owner loses g ÷ 2 (what its
///   week added), so every town keeps half of its count before the week (`TWN-QUIRK-9`).
///
/// A normal month, the Portal and the other indexes are untouched; a faction without a record in
/// `part` keeps its counts.
// rule: TWN-GRO-004
pub fn month(
    town: &Town,
    part: &TownRuleset,
    core: &Ruleset,
    month: MonthGrowth,
    week: &WeekKind,
    inputs: &GrowthInputs,
) -> Town {
    let Some(record) = part.town(town.faction()) else {
        return town.clone();
    };
    let quirks = core.quirks();
    match month {
        MonthGrowth::Normal => town.clone(),
        MonthGrowth::Creature(_) if matches!(week, WeekKind::Of { .. }) => town.clone(),
        MonthGrowth::Creature(creature) => {
            let exact = quirks.is_enabled(CREATURE_MONTH_EXACT);
            each_growing(town, part, core, inputs, |index, count, _| {
                if doubles(record, index, creature, exact) {
                    count * 2
                } else {
                    count
                }
            })
        }
        MonthGrowth::Plague => {
            let full = inputs.owned || quirks.is_enabled(NEUTRAL_PLAGUE_FULL_GROWTH);
            each_growing(town, part, core, inputs, |_, count, g| {
                count.saturating_sub(if full { g } else { g / 2 }) / 2
            })
        }
    }
}

#[cfg(test)]
mod tests;
