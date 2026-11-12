//! A neutral town's garrison week (E09 increment 3, design note §44; survey `E09-growth.md` §4, the
//! original's `neutral_garrison_week` 0x4BF570, read whole): each week a town without an owner may
//! draw a stack of its faction's creatures into its garrison, out of its available creatures
//! (TWN-GRO-006). A pure function over the draws given; the game calls it for each town without an
//! owner in `TownId` order after the adventure's week step, from the towns' stream, and writes the
//! garrison and the town (E09 T26b).

use alloc::vec::Vec;

use h3_content::{CreatureTypeId, Ruleset};
use h3_contracts::{ArmyStack, FortLevel};
use h3_kernel::{Rng, order};

use crate::record::Town;
use crate::ruleset::{MAX_CREATURE_LEVELS, TownRuleset};
use crate::siege::fort_level;

/// What a neutral town's garrison week changed (TWN-GRO-006): its garrison and the town.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GarrisonWeek {
    /// The garrison after the week, in slot order.
    pub stacks: Vec<ArmyStack>,
    /// The town, its counts at the level's two indexes lowered by the new stack's count.
    pub town: Town,
}

/// The level, from 1, a garrison week's sum `sum` draws with the parameters' `weights` (the
/// original's `neutral_garrison_level_weights` 0x6779B0): the first level whose weight `sum` does
/// not exceed, `sum` lessened by each weight passed; past them all, the level after the last.
fn level(weights: &[u8], sum: u32) -> usize {
    let mut rest = sum;
    for (index, &weight) in weights.iter().enumerate() {
        if rest <= u32::from(weight) {
            return index + 1;
        }
        rest -= u32::from(weight);
    }
    weights.len() + 1
}

/// The index in `stacks` (in slot order) where `count` `creature`s go, as the adventure's
/// `army_add_stack` finds room (ADV-ARM-007, `rules` the army's slots and a stack's most
/// creatures): its first stack of the creature that takes them within the most, else a stack in
/// its first free slot when they fit one.
fn room(
    stacks: &[ArmyStack],
    (slots, most): (u8, u32),
    creature: CreatureTypeId,
    count: u32,
) -> Option<Room> {
    if let Some(index) = stacks
        .iter()
        .position(|s| s.creature == creature && count <= most.saturating_sub(s.count))
    {
        return Some(Room::Stack(index));
    }
    (0..slots)
        .find(|&slot| count <= most && stacks.iter().all(|s| s.slot != slot))
        .map(Room::Slot)
}

/// Where a stack goes in a garrison ([`room`]).
#[derive(Debug, Clone, Copy)]
enum Room {
    /// Onto the stack at this index.
    Stack(usize),
    /// As a stack in this free slot.
    Slot(u8),
}

/// `stacks` (in slot order) with `count` `creature`s added at `room`, kept in slot order.
fn join(stacks: &mut Vec<ArmyStack>, room: Room, creature: CreatureTypeId, count: u32) {
    match room {
        Room::Stack(index) => stacks[index].count += count,
        Room::Slot(slot) => {
            let at = stacks
                .iter()
                .position(|s| s.slot > slot)
                .unwrap_or(stacks.len());
            stacks.insert(
                at,
                ArmyStack {
                    slot,
                    creature,
                    count,
                },
            );
        }
    }
}

/// A neutral town's garrison week (TWN-GRO-006; E09 design §44, survey `E09-growth.md` §4, the
/// original's `neutral_garrison_week` 0x4BF570), for `town` without an owner and its `garrison`
/// (its stacks, in any order), with its faction's record and the parameters' `garrison_week` in
/// `part`, the creatures' growths and AI values in `core`, `weeks` the weeks since the game's start
/// (the game's (day − 1) ÷ the week's length on a week's first day) and `rules` the army's slots
/// and a stack's most creatures (the adventure part's, ADV-ARM-007), from `draws` (the towns'
/// stream):
///
/// 1. `below(out_of)` under the odds (the original's 80 in 101 with a standing fort level, else
///    40); else none;
/// 2. n = min(`weeks`, the parameters' `weeks`), S the sum of three `range_inclusive(0, n + 1)`;
///    the level the weights give S ([`level`]: the original's S ≤ 2 level 1, ≤ 5 level 2, ≤ 9
///    level 3, ≤ 14 level 4, ≤ 18 level 5, ≤ 21 level 6, else level 7);
/// 3. the creature the level's base, its upgrade when a garrison stack holds the upgrade; the
///    count its core growth;
/// 4. room: a stack of the creature or an empty slot ([`room`]); else the stack of least AI value
///    × count below the new stack's, the first of equals in slot order, is cleared and the new
///    stack takes its slot; else none (nothing lowered, no further draw); the stack merged into
///    the creature's or put in the first empty slot;
/// 5. the town's counts at the level's base and upgraded indexes each lose the count (at least 0):
///    the garrison's growth comes out of the available creatures;
/// 6. for a base creature, `range_inclusive(1, 100)` at most the parameters' `upgrade` (the
///    original's 5) turns every stack of it into the upgrade (a level without one: unchanged).
///
/// None, with no draw, for a part without parameters or a faction without a record; none after
/// the sums for a level the record lacks, a creature growing none or a growth above a stack's
/// most. *Agent's calls* (E09 T25b2, change freely): `rules` passed by the game (the room is the
/// adventure's army rule, ADV-ARM-007); a level the record lacks, a creature growing none and a
/// growth no stack holds add nothing.
// rule: TWN-GRO-006
pub fn garrison_week(
    town: &Town,
    part: &TownRuleset,
    core: &Ruleset,
    weeks: u32,
    garrison: &[ArmyStack],
    rules: (u8, u32),
    draws: &mut Rng,
) -> Option<GarrisonWeek> {
    let params = &part.params()?.garrison_week;
    let record = part.town(town.faction())?;
    let fortified = fort_level(town, part) != FortLevel::None;
    let odds = params.odds[usize::from(fortified)];
    if draws.below(u32::from(params.out_of)) >= u32::from(odds) {
        return None;
    }
    let n = weeks.min(u32::from(params.weeks));
    let sum = (0..3).fold(0, |sum, _| sum + draws.range_inclusive(0, n + 1));
    let index = level(&params.weights, sum) - 1;
    let creatures = record.creatures.get(index)?;
    let upgraded = creatures
        .upgrade
        .filter(|&upgrade| garrison.iter().any(|s| s.creature == upgrade));
    let creature = upgraded.unwrap_or(creatures.base);
    let info = |c: CreatureTypeId| core.creature(c);
    let count = info(creature).map_or(0, |c| c.growth);
    if count == 0 {
        return None;
    }
    let count32 = u32::from(count);
    let mut stacks = garrison.to_vec();
    // An army holds one stack per slot, so the slot is a unique key.
    let _ = order::sort_by_unique_key(&mut stacks, |s| s.slot);
    let value = |c: CreatureTypeId, count: u32| {
        u64::from(info(c).map_or(0, |c| c.ai_value)) * u64::from(count)
    };
    let found = match room(&stacks, rules, creature, count32) {
        Some(found) => found,
        // Every slot taken: the weakest stack below the new one makes room in its slot.
        None if count32 <= rules.1 => {
            let worth = value(creature, count32);
            let (_, at) = stacks
                .iter()
                .enumerate()
                .map(|(at, s)| (value(s.creature, s.count), at))
                .filter(|&(v, _)| v < worth)
                .min()?;
            Room::Slot(stacks.remove(at).slot)
        }
        None => return None,
    };
    join(&mut stacks, found, creature, count32);
    let town = [index, index + MAX_CREATURE_LEVELS]
        .into_iter()
        .fold(town.clone(), |town, at| {
            let left = town.available(at).saturating_sub(count);
            town.with_available(at, left)
        });
    if upgraded.is_none()
        && draws.range_inclusive(1, 100) <= u32::from(params.upgrade)
        && let Some(upgrade) = creatures.upgrade
    {
        stacks
            .iter_mut()
            .filter(|s| s.creature == creatures.base)
            .for_each(|s| s.creature = upgrade);
    }
    Some(GarrisonWeek { stacks, town })
}

#[cfg(test)]
mod tests;
