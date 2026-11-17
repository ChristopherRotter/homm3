//! The creature banks on the map (E08 T124b1; increment 13's design note part B): their draw when
//! the game is built (ADV-BNK-002, design note §5), called by `draws::resolve_rewards`' second
//! walk in object order. The visit, the battle's map half and the reward come with T124b2.

use alloc::vec::Vec;

use h3_content::{CreatureTypeId, Ruleset};
use h3_contracts::ArmyStack;
use h3_kernel::Rng;

use super::Resolved;
use super::draws::ArtifactPool;
use crate::ruleset::{BANK_ARTIFACT_CLASSES, Bank};
use crate::setup::Guards;

/// A creature bank's draws (ADV-BNK-002, E08 T124b1; increment 13's design note §5, survey
/// `E08-banks.md` §3: `bank_draw` 0x47AD90 in `setup_map_objects`' walk):
/// 1. **the level:** r in 1 to 100; the first level but the last whose running sum of chances
///    reaches r, else the last (the original's subtraction over its levels 0 to 2, else 3);
/// 2. **the artifacts:** the level's counts by class, the relics first, then the majors, the
///    minors and the treasures (D-68), each a draw of its class from the random artifacts' pool
///    (ADV-OBJ-006, the original's `random_artifact_of_level`); a draw that finds none gives none;
/// 3. **the split** of the level's guards into their slots, without a draw ([`bank_split`]);
/// 4. **the upgrade:** when the level's chance is above 0 and the split's upgrade slot holds a
///    creature with an upgrade (`Ruleset::upgrades`, the first; the original's
///    `creature_is_unupgraded` 0x47B120 and `creature_upgrade` 0x47B1A0 agree on every shipped
///    guard), r in 1 to 100: at most the chance, the stack becomes that upgrade, its count kept.
///    The original draws r for every bank; the engine only where it can change a stack (E08
///    decision 17's form).
///
/// The guards ask (their [`Guards::question`]; a bank's visit asks by its kind's question).
// rule: ADV-BNK-002
pub(super) fn bank_draw(
    core: &Ruleset,
    bank: &Bank,
    artifacts: &mut ArtifactPool<'_>,
    draws: &mut Rng,
) -> (Guards, Resolved) {
    let r = draws.range_inclusive(1, 100);
    // A bank has one to four levels (`compile_bank`).
    let last = bank.levels.len() - 1;
    let mut sum = 0;
    let index = (0..last)
        .find(|&l| {
            sum += u32::from(bank.levels[l].chance);
            r <= sum
        })
        .unwrap_or(last);
    let level = &bank.levels[index];
    let mut drawn = Vec::new();
    for (class, &count) in BANK_ARTIFACT_CLASSES.iter().zip(&level.artifacts).rev() {
        for _ in 0..count {
            drawn.extend(artifacts.draw(&[*class], draws));
        }
    }
    let (mut army, slot) = bank_split(&level.guards);
    if level.upgrade > 0
        && let Some(stack) = army.iter_mut().find(|stack| stack.slot == slot)
        && let Some(&upgrade) = core.upgrades(stack.creature).first()
        && draws.range_inclusive(1, 100) <= u32::from(level.upgrade)
    {
        stack.creature = upgrade;
    }
    let guards = Guards {
        army,
        question: true,
    };
    let level = u8::try_from(index).unwrap_or(u8::MAX);
    let resolved = Resolved::Bank {
        level,
        artifacts: drawn,
    };
    (guards, resolved)
}

/// A bank level's guards in their slots and the slot its upgrade tests (ADV-BNK-002; increment
/// 13's design note §5 step 3, survey `E08-banks.md` §3 step 3): the named creatures in slots 0
/// up, then by their number n, each new stack in the next slot as the original's first empty slot
/// puts it:
/// - n = 1: slots 1 to 4 take ⌊c / 5⌋, then of what is left ⌊· / 4⌋, ⌊· / 3⌋ and ⌊· / 2⌋, slot 0
///   the rest; the upgrade's slot 2;
/// - n = 2: the second's count halved into slot 2, the first's divided by 3 into slot 3 and what
///   is left of it halved into slot 4; the upgrade's slot 3;
/// - n = 3: the second's halved into slot 3, the first's into slot 4; the upgrade's slot 0;
/// - n = 4: no split; the upgrade's slot 0.
///
/// A quotient of 0 leaves its slot empty (*agent's call*: unreachable with the shipped counts,
/// every one at least 10; the original's write of a stack of 0 is unread). What moves is the
/// quotient; what stays the rest, so no stack named is ever empty.
// rule: ADV-BNK-002
fn bank_split(guards: &[(CreatureTypeId, u32)]) -> (Vec<ArmyStack>, u8) {
    let mut slots = guards.to_vec();
    // Moves `divisor`'s quotient of slot `from`'s count into a new slot.
    let mut split = |from: usize, divisor: u32| {
        let (creature, count) = slots[from];
        let part = count / divisor;
        slots[from].1 = count - part;
        slots.push((creature, part));
    };
    let upgrade = match guards.len() {
        1 => {
            for divisor in [5, 4, 3, 2] {
                split(0, divisor);
            }
            2
        }
        2 => {
            split(1, 2);
            split(0, 3);
            split(0, 2);
            3
        }
        3 => {
            split(1, 2);
            split(0, 2);
            0
        }
        _ => 0,
    };
    let army = slots
        .into_iter()
        .enumerate()
        .filter(|&(_, (_, count))| count > 0)
        .map(|(slot, (creature, count))| ArmyStack {
            slot: u8::try_from(slot).unwrap_or(u8::MAX),
            creature,
            count,
        })
        .collect();
    (army, upgrade)
}
