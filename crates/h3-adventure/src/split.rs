//! A wandering monster's split into stacks for its battle (ADV-MON-009, E08 T20b2; survey
//! `docs/rules/surveys/E08-battles.md` §4.1; the executable's `monster_battle`, 0x4AC580): the
//! number of stacks from the visitor's army against the monster's strength by the original's
//! table, two draws of 1 to 100 (one stack more or fewer; a town creature's middle stack
//! upgraded), even stacks. The draws are the caller's: under `ADV-QUIRK-6` (`faithful`) the C
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
/// original's three-type form (its other callers) is not built.
// rule: ADV-MON-009
pub fn split(
    core: &Ruleset,
    visitor: &[ArmyStack],
    (creature, count): (CreatureTypeId, u32),
    mut percent: impl FnMut() -> u32,
) -> Vec<ArmyStack> {
    let ai_value = |c: CreatureTypeId| core.creature(c).map_or(0, |info| u64::from(info.ai_value));
    let value = visitor.iter().fold(0u64, |sum, stack| {
        sum.saturating_add(ai_value(stack.creature) * u64::from(stack.count))
    });
    let strength = ai_value(creature) * u64::from(count);
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
    let stacks = stacks.clamp(1, count.max(1));
    let (each, more) = (count / stacks, count % stacks);
    let mut army: Vec<ArmyStack> = (0..stacks)
        .map(|i| ArmyStack {
            slot: u8::try_from(i).unwrap_or(u8::MAX),
            creature,
            count: each + u32::from(i < more),
        })
        .collect();
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

#[cfg(test)]
mod tests;
