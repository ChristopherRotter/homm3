//! The mage guild (E09 increment 5, design note §85 to §87; survey `E09-guild.md` §3 to §5): a
//! town's guild slots drawn once at the game's build ([`draw_guild`], TWN-GLD-002), what the guild
//! shows of them as it is built ([`guild_level`], [`shown`], TWN-GLD-003) and the town's Aurora
//! set ([`aurora_set`], the set TWN-GLD-004 teaches). Pure functions over a town record, the part
//! and the core with the spells the map and the scenario give ([`GuildSets`]); the game draws
//! every town's guild at the build from the towns' stream (E09 decision 179) and keeps the slots
//! and the set in the town's record (the store v4, E09 T47a2).

use h3_content::{AURORA_IGNORES_MAP_BANS, MAX_SPELLS, Ruleset, Spell, SpellCategory, SpellId};
use h3_kernel::Rng;

use crate::record::Town;
use crate::ruleset::{TownParams, TownRecord, TownRuleset};

/// The spell levels a guild holds, 1 to 5 (the part's `guild_slots`, TWN-GLD-001).
const LEVELS: usize = 5;

/// The most slots a guild level holds: the part's at most 5 (TWN-GLD-001) and the spare (the
/// original's six a level, +0x44 + 0x18 by level).
pub const MAX_GUILD_SLOTS: usize = 6;

/// An empty slot's byte in [`GuildSlots`] (a spell's handle is below [`MAX_SPELLS`]).
const EMPTY_SLOT: u8 = u8::MAX;

// A spell set is one `u128` and a slot one byte.
const _: () = assert!(MAX_SPELLS <= 128);

/// A set of spells, a bit per spell handle (the content allows at most [`MAX_SPELLS`]); a handle
/// past them is never held.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct SpellSet(u128);

impl SpellSet {
    /// No spell.
    pub const EMPTY: Self = Self(0);

    /// The set of `spells`.
    pub fn of(spells: &[SpellId]) -> Self {
        spells
            .iter()
            .fold(Self::EMPTY, |set, &spell| set.with(spell))
    }

    /// The set's bits, bit `i` for the spell of handle `i` (the canonical hashes' encoding).
    pub const fn bits(self) -> u128 {
        self.0
    }

    /// Whether the set holds `spell`.
    pub fn contains(self, spell: SpellId) -> bool {
        self.0 & bit(spell) != 0
    }

    /// The set with `spell` added (a handle past [`MAX_SPELLS`]: the set unchanged).
    #[must_use]
    pub fn with(self, spell: SpellId) -> Self {
        Self(self.0 | bit(spell))
    }

    /// The spells of either set.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// The set's spells, in handle order.
    pub fn iter(self) -> impl Iterator<Item = SpellId> {
        let mut bits = self.0;
        core::iter::from_fn(move || {
            // The lowest bit's index; 128 (past every spell) once no bit is left, which ends it.
            let index = Some(bits.trailing_zeros() as usize).filter(|&i| i < MAX_SPELLS)?;
            bits &= bits.wrapping_sub(1);
            SpellId::from_index(index)
        })
    }
}

/// `spell`'s bit in a [`SpellSet`]; 0 past [`MAX_SPELLS`].
fn bit(spell: SpellId) -> u128 {
    1u128.checked_shl(u32::from(spell.raw())).unwrap_or(0)
}

/// The spells a town's guild draw and its Aurora set read (E09 design §84 to §86): the town's
/// **required** and **banned** spells (the map's town record, TWN-SET-012) and the scenario's
/// bans (GAM-SET-003). The game makes them of its setup's lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GuildSets {
    /// The spells the map requires in the town's guild.
    pub required: SpellSet,
    /// The spells the map bans from the town's guild.
    pub banned: SpellSet,
    /// The spells the scenario bans map-wide.
    pub scenario: SpellSet,
}

/// A town's guild slots (E09 design §86; the original's +0x44): per spell level 1 to 5 its
/// [`MAX_GUILD_SLOTS`] slots in order, each a spell or empty. The draw fills each level's first
/// slots, the part's count and the **spare** (the slot a Library shows); the rest stay empty, and
/// an empty slot is only ever followed by empty ones of its level. Drawn once, at the game's build
/// ([`draw_guild`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuildSlots([[u8; MAX_GUILD_SLOTS]; LEVELS]);

impl GuildSlots {
    /// Every slot empty.
    pub const EMPTY: Self = Self([[EMPTY_SLOT; MAX_GUILD_SLOTS]; LEVELS]);

    /// The spell of `level`'s slot `slot` (from 0), if any; none for a level outside 1 to 5 or a
    /// slot past [`MAX_GUILD_SLOTS`].
    pub fn slot(&self, level: u8, slot: usize) -> Option<SpellId> {
        usize::from(level)
            .checked_sub(1)
            .and_then(|index| self.0.get(index))
            .and_then(|slots| slots.get(slot))
            .filter(|&&raw| raw != EMPTY_SLOT)
            .map(|&raw| SpellId::from_raw(u16::from(raw)))
    }

    /// `level`'s [`MAX_GUILD_SLOTS`] slots in order (each empty for a level outside 1 to 5).
    pub fn level(&self, level: u8) -> impl Iterator<Item = Option<SpellId>> + '_ {
        (0..MAX_GUILD_SLOTS).map(move |slot| self.slot(level, slot))
    }

    /// Every slot, by level 1 to 5 then slot (the canonical hash's order, the store v4).
    pub fn all(&self) -> impl Iterator<Item = Option<SpellId>> + '_ {
        (1..).take(LEVELS).flat_map(|level| self.level(level))
    }

    /// The slots with `spell` in slot `slot` of the level at `index` (level `index + 1`); a slot
    /// past the slots changes nothing.
    pub(crate) fn with(mut self, index: usize, slot: usize, spell: SpellId) -> Self {
        let raw = u8::try_from(spell.raw()).unwrap_or(EMPTY_SLOT);
        self.0
            .get_mut(index)
            .and_then(|slots| slots.get_mut(slot))
            .into_iter()
            .for_each(|at| *at = raw);
        self
    }
}

/// The core's hero spells a guild may hold, with their handles in spell order: every spell but
/// the creature abilities and those granted by name only (Titan's Lightning Bolt, the original's
/// flag 0x2000, which `new_game_bans` bans; survey §3.1).
fn guild_spells(core: &Ruleset) -> impl Iterator<Item = (SpellId, &Spell)> {
    core.spells()
        .iter()
        .zip((0..).map_while(SpellId::from_index))
        .filter(|(spell, _)| {
            spell.category != SpellCategory::Ability && !spell.granted_by_name_only
        })
        .map(|(spell, id)| (id, spell))
}

/// A town's guild drawn (E09 design §85; survey §3; the original's `town_setup_spells`
/// 0x5BE600): `record` the town's faction's, `sets` its required and banned spells and the
/// scenario's bans, `draws` the towns' stream (E09 decision 179).
///
/// The excluded spells start as the town's banned, the scenario's and every spell a guild never
/// holds (a creature ability, a spell granted by name only); for each level 1 to 5 and each of its `params.guild_slots` slots and
/// the spare, in order: among the level's spells not excluded, in spell order, the first the
/// town requires is taken whatever its weight; else, with a total of their weights (`record`'s,
/// TWN-GLD-001) of 0, the slot stays empty without a draw; else one draw of 1 to the total and
/// the first spell whose running weight reaches it (a weight of 0 is never chosen). The taken
/// spell is excluded from then on. So a spell the town both requires and bans is never in its
/// guild (TD-25, E09 decision 168), and a level's required spells fill its first slots.
// rule: TWN-GLD-002
pub fn draw_guild(
    record: &TownRecord,
    params: &TownParams,
    core: &Ruleset,
    sets: &GuildSets,
    draws: &mut Rng,
) -> GuildSlots {
    let mut excluded = sets.banned.union(sets.scenario);
    let mut slots = GuildSlots::EMPTY;
    for ((index, level), &count) in (0..LEVELS).zip(1..).zip(&params.guild_slots) {
        for slot in (0..=usize::from(count)).take(MAX_GUILD_SLOTS) {
            let open = move || {
                guild_spells(core)
                    .filter(move |&(id, spell)| spell.level == level && !excluded.contains(id))
                    .map(move |(id, _)| (id, u32::from(record.guild_weight(id))))
            };
            let taken = open()
                .find(|&(id, _)| sets.required.contains(id))
                .or_else(|| {
                    let total: u32 = open().map(|(_, weight)| weight).sum();
                    let roll = (total > 0).then(|| draws.range_inclusive(1, total))?;
                    let mut sum = 0;
                    open().find(|&(_, weight)| {
                        sum += weight;
                        sum >= roll
                    })
                });
            if let Some((spell, _)) = taken {
                slots = slots.with(index, slot, spell);
                excluded = excluded.with(spell);
            }
        }
    }
    slots
}

/// The town's guild level (E09 design §87; survey §3.3, the original's +0x14): the highest level
/// of a standing building of role `guild` (TWN-GLD-001), 0 without one (or without its
/// faction's record in `part`). The faction's highest level is its record's buildings.
// rule: TWN-GLD-003
pub fn guild_level(town: &Town, part: &TownRuleset) -> u8 {
    part.town(town.faction()).map_or(0, |record| {
        town.standing(record)
            .iter()
            .filter_map(|id| record.building(id).and_then(|b| b.roles.guild))
            .max()
            .unwrap_or(0)
    })
}

/// The spells `town`'s guild shows at `level` of its `slots` (E09 design §87; survey §3.3, §4,
/// the original's counts +0xBC): for a level from 1 to the [`guild_level`], its first
/// `guild_slots` slots of the part's, one more with a building of role `library` ever built, cut
/// at the first empty slot; none above the guild level. The counts are derived, never stored, so
/// a level or a Library built later shows at once.
// rule: TWN-GLD-003
pub fn shown<'a>(
    slots: &'a GuildSlots,
    town: &Town,
    part: &TownRuleset,
    level: u8,
) -> impl Iterator<Item = SpellId> + 'a {
    let count = part
        .town(town.faction())
        .zip(part.params())
        .filter(|_| (1..=guild_level(town, part)).contains(&level))
        .and_then(|(record, params)| {
            let library = town
                .built()
                .iter()
                .filter_map(|id| record.building(id))
                .any(|b| b.roles.library);
            usize::from(level)
                .checked_sub(1)
                .and_then(|index| params.guild_slots.get(index))
                .map(|&n| usize::from(n) + usize::from(library))
        })
        .unwrap_or(0);
    slots
        .level(level)
        .take(count)
        .map_while(core::convert::identity)
}

/// A town's Aurora set (E09 design §86; survey §5; the original's `town_heroes_learn_spells`
/// 0x5BE030, its first branch): what a building of role `aurora` teaches there, fixed at the
/// build: every spell a guild may hold (not a creature ability, not granted by name only) that the town does not ban, less the scenario's bans when the core's
/// `quirk.town.aurora_ignores_map_bans` is off (`TWN-QUIRK-14`, T-18 `fix-optional`: on in
/// `faithful`, where the original never tests them). The town's required spells do not matter.
// rule: TWN-GLD-004
pub fn aurora_set(core: &Ruleset, sets: &GuildSets) -> SpellSet {
    let scenario = if core.quirks().is_enabled(AURORA_IGNORES_MAP_BANS) {
        SpellSet::EMPTY
    } else {
        sets.scenario
    };
    let excluded = sets.banned.union(scenario);
    guild_spells(core)
        .map(|(id, _)| id)
        .filter(|&id| !excluded.contains(id))
        .fold(SpellSet::EMPTY, SpellSet::with)
}

#[cfg(test)]
mod tests;
