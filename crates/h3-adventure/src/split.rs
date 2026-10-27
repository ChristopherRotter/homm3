//! A wandering monster's split into stacks for its battle (ADV-MON-009, E08 T20b2; survey
//! `docs/rules/surveys/E08-battles.md` §4.1; the executable's `monster_battle`, 0x4AC580): the
//! number of stacks from the visitor's army against the monster's strength by the original's
//! table, two draws of 1 to 100 (one stack more or fewer; a town creature's middle stack
//! upgraded), even stacks; since E08 T62b1 the form with a second creature (ADV-MON-012, the
//! Pyramid's guards). The draws are the caller's: under `ADV-QUIRK-6` (`faithful`) the C
//! runtime's generator seeded by the monster's tile ([`TileRand`]), else the attacking player's
//! stream (ADR-0017 point 4).

use alloc::vec::Vec;

use h3_content::{CreatureTypeId, Ruleset};
use h3_contracts::ArmyStack;
use h3_kernel::arith::{DOUBLE_BITS, X87};

use crate::setup::Pos;

/// The C runtime's `rand` (the executable's `crt_rand`, 0x617E5C) seeded by a monster's tile, as
/// `monster_battle` seeds it (`srand`, 0x4AC5E6): the split's draws under `ADV-QUIRK-6`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileRand(u32);

impl TileRand {
    /// The generator seeded with 248 071 x + 276 589 y + 310 111 z + 155 303, in 32 bits, for the
    /// monster's tile (x, y, level z; 0x4AC598 to 0x4AC5E3).
    pub fn new(tile: Pos) -> Self {
        let seed = u32::from(tile.x)
            .wrapping_mul(248_071)
            .wrapping_add(u32::from(tile.y).wrapping_mul(276_589))
            .wrapping_add(u32::from(tile.level).wrapping_mul(310_111))
            .wrapping_add(155_303);
        Self(seed)
    }

    /// The next `rand()`: the state times 214 013 plus 2 531 011, bits 16 to 30 of it.
    pub const fn rand(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(214_013).wrapping_add(2_531_011);
        (self.0 >> 16) & 0x7FFF
    }

    /// The next draw of 1 to 100: `random_between(1, 100)` (0x50B230), 1 + `rand()` mod 100.
    pub const fn percent(&mut self) -> u32 {
        1 + self.rand() % 100
    }
}

/// The most stacks a monster splits into: an army's seven slots.
const MOST_STACKS: u32 = 7;

/// The split's table (0x6776E0, read from its fifth entry down): a ratio of the visitor's army
/// value to the monster's strength below 0.5, 0.67, 1, 1.5, 2 gives 7, 6, 5, 4, 3 stacks, from 2
/// on 2. In basis points; each compared as the executable's double.
const STACK_RATIOS_BP: [u32; 5] = [5_000, 6_700, 10_000, 15_000, 20_000];

/// The first draw at most this many gives one stack fewer (0x4AC6B7).
const FEWER_AT_MOST: u32 = 20;

/// The first draw at least this many gives one stack more, below seven (0x4AC6BD).
const MORE_AT_LEAST: u32 = 80;

/// The second draw at most this many upgrades the middle stack (0x4AC781).
const UPGRADE_AT_MOST: u32 = 50;

/// How `monster` (its creature and count as it stands) splits into stacks for a battle with a
/// hero whose army is `visitor` (ADV-MON-009, E08 T20b2; the executable's 0x4AC580): the number
/// of stacks by the table from the ratio of the army's value (each stack's creature's AI value
/// times its count; no Attack or Defense factor) to the monster's strength (its AI value times
/// its count), both exact as doubles and divided in double precision (a strength of 0 gives 2
/// stacks, 7 when the army is worth nothing too: the FPU's infinity and NaN); one fewer when the
/// first draw is at most 20, one more when it is at least 80 and fewer than seven; at least one
/// and at most the count. The count divides evenly, the first `count mod n` stacks one more, in
/// slots from 0. Then, when the creature is a town's unupgraded one (a faction and an upgrade,
/// the original's 0x47B120) and there are at least two stacks, a second draw at most 50 turns the
/// middle stack (`n / 2`) into its first upgrade, of the same count. `percent` gives the draws,
/// each from 1 to 100; the second is taken only when its condition holds, as the original's. The
/// form with a second creature is [`split_with_second`]; the original's third creature (no
/// caller of the engine's) is not built.
// rule: ADV-MON-009
pub fn split(
    core: &Ruleset,
    visitor: &[ArmyStack],
    (creature, count): (CreatureTypeId, u32),
    mut percent: impl FnMut() -> u32,
) -> Vec<ArmyStack> {
    let strength = ai_value(core, creature) * u64::from(count);
    let stacks = stack_count(core, visitor, strength, &mut percent).clamp(1, count.max(1));
    let mut army = even(creature, count, stacks);
    let upgrade = core
        .creature(creature)
        .filter(|info| info.faction.is_some())
        .and_then(|_| core.upgrades(creature).first().copied());
    if let Some(upgrade) = upgrade.filter(|_| stacks > 1)
        && percent() <= UPGRADE_AT_MOST
    {
        army[usize::try_from(stacks / 2).unwrap_or(0)].creature = upgrade;
    }
    army
}

/// The original's order of the slots of monsters with a second creature (`monster_stack_order`,
/// 0x63DF94, the rows 7 n with no third creature): for `n` stacks of the first creature (1 to 6),
/// slot `j` holds the stack the row names at `j`, the first creature's stacks numbered from 0,
/// the second's after them, a number past them an empty slot.
const SECOND_ORDER: [[u8; 7]; 6] = [
    [1, 2, 3, 0, 4, 5, 6],
    [2, 3, 0, 4, 1, 5, 6],
    [3, 0, 4, 1, 5, 2, 6],
    [0, 4, 1, 5, 2, 6, 3],
    [0, 1, 5, 2, 6, 3, 4],
    [0, 1, 2, 6, 3, 4, 5],
];

/// How monsters of two creatures split into stacks for a battle with a hero whose army is
/// `visitor` (ADV-MON-012, E08 T62b1; the executable's `monster_battle` 0x4AC580 with a second
/// creature, survey `E08-rewards.md` §7.5, D-40): the strength sums both creatures' (each AI
/// value times its count, the second's count its stacks' sum); the number of stacks n as
/// [`split`]'s from that strength and the first draw, **less the second creature's stacks**, at
/// least one and at most the first creature's count; the first creature's count divides evenly
/// into n stacks (the first `count mod n` one more), the second creature keeps its stacks' counts
/// in order, no stack is upgraded and no second draw is taken; then the slots in the original's
/// order ([`SECOND_ORDER`]'s row for n, the first creature's stacks numbered first). `percent`
/// gives the draw, from 1 to 100. The second creature fills fewer slots than an army's (the
/// ruleset checks it), so n plus its stacks fit in seven.
// rule: ADV-MON-012
pub fn split_with_second(
    core: &Ruleset,
    visitor: &[ArmyStack],
    (creature, count): (CreatureTypeId, u32),
    (second, stacks): (CreatureTypeId, &[u32]),
    mut percent: impl FnMut() -> u32,
) -> Vec<ArmyStack> {
    let total = stacks
        .iter()
        .fold(0u64, |sum, &each| sum.saturating_add(u64::from(each)));
    let strength = (ai_value(core, creature) * u64::from(count))
        .saturating_add(ai_value(core, second).saturating_mul(total));
    let fixed = u32::try_from(stacks.len()).unwrap_or(MOST_STACKS);
    let n = stack_count(core, visitor, strength, &mut percent)
        .saturating_sub(fixed)
        .clamp(1, count.max(1));
    let mut numbered = even(creature, count, n);
    numbered.extend(stacks.iter().map(|&each| ArmyStack {
        slot: 0,
        creature: second,
        count: each,
    }));
    let row = SECOND_ORDER[usize::try_from(n.clamp(1, 6) - 1).unwrap_or(0)];
    row.iter()
        .zip(0u8..)
        .filter_map(|(&from, slot)| {
            numbered
                .get(usize::from(from))
                .map(|stack| ArmyStack { slot, ..*stack })
        })
        .collect()
}

/// A creature's AI value (0 for one the ruleset does not have).
fn ai_value(core: &Ruleset, creature: CreatureTypeId) -> u64 {
    core.creature(creature)
        .map_or(0, |info| u64::from(info.ai_value))
}

/// The number of stacks before the clamps (ADV-MON-009; 0x4AC671 to 0x4AC6C7): by the table from
/// the ratio of the visitor's army value to `strength`, as doubles, then one fewer when the
/// first draw is at most 20 and one more when it is at least 80 and fewer than seven.
fn stack_count(
    core: &Ruleset,
    visitor: &[ArmyStack],
    strength: u64,
    percent: &mut impl FnMut() -> u32,
) -> u32 {
    let value = visitor.iter().fold(0u64, |sum, stack| {
        sum.saturating_add(ai_value(core, stack.creature) * u64::from(stack.count))
    });
    let worthless = if value == 0 { MOST_STACKS } else { 2 };
    let base = X87::int(value)
        .over(X87::int(strength))
        .map_or(worthless, |ratio| {
            let ratio = ratio.round(DOUBLE_BITS);
            let reached = STACK_RATIOS_BP
                .iter()
                .take_while(|&&bp| X87::ratio(bp, 10_000, DOUBLE_BITS).is_some_and(|t| ratio >= t))
                .count();
            MOST_STACKS - u32::try_from(reached).unwrap_or(5)
        });
    let first = percent();
    let mut stacks = base;
    if first <= FEWER_AT_MOST {
        stacks -= 1;
    }
    if first >= MORE_AT_LEAST && stacks < MOST_STACKS {
        stacks += 1;
    }
    stacks
}

/// `count` of `creature` divided evenly into `stacks` stacks in slots from 0, the first
/// `count mod stacks` one more.
fn even(creature: CreatureTypeId, count: u32, stacks: u32) -> Vec<ArmyStack> {
    let (each, more) = (count / stacks, count % stacks);
    (0..stacks)
        .map(|i| ArmyStack {
            slot: u8::try_from(i).unwrap_or(u8::MAX),
            creature,
            count: each + u32::from(i < more),
        })
        .collect()
}

#[cfg(test)]
mod tests;
