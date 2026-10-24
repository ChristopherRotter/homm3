//! A hero's army management (ADR-0016 point 3; E08 T50, `E08-design.md` §7): its owner moves a
//! stack onto another slot (into a stack of its creature, into an empty slot, else a swap),
//! splits a stack or dismisses one, within a hero's army, between the two heroes of a meeting,
//! or between a hero and the group of an army dialog (GAM-MET-001, GAM-DEC-004: the game holds
//! the decisions and the group). Counts keep their full width (A-15 fixed, ADV-ARM-005) and never
//! pass a stack's most (ADV-ARM-007). This module plans a command on the armies it names, as new
//! stacks; [`crate::AdventureState::manage_army`] finds the armies and applies the plan whole.

use alloc::vec::Vec;
use core::fmt;
use core::ops::RangeInclusive;

use h3_contracts::{ArmyStack, HeroId};

/// An army a management command names (ADR-0016 point 3): a hero's on the map, or the group of
/// its owner's open army dialog (GAM-DEC-004), an army of the ruleset's slots without a hero,
/// which the game holds and passes in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArmyRef {
    /// A hero's army.
    Hero(HeroId),
    /// The open army dialog's group.
    Group,
}

/// A slot of an army.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArmySlot {
    /// The army.
    pub army: ArmyRef,
    /// The slot, from 0.
    pub slot: u8,
}

/// A management command on a player's armies (ADV-ARM-001 to -004; E08 T50). Between two
/// armies, a hero's last stack never leaves its army (it may swap); the group has no protected
/// stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmyCommand {
    /// Moves the stack at `from` onto `to` (ADV-ARM-001, -002): into the stack there if it is of
    /// the same creature (the counts added, at most a stack's most: ADV-ARM-007), into the slot
    /// if it is empty, else the two stacks swap. A move onto its own slot is refused.
    Move {
        /// The stack's slot.
        from: ArmySlot,
        /// Where it goes.
        to: ArmySlot,
    },
    /// Splits the stack at `from` with `to` (ADV-ARM-003): `to`, empty or holding the same
    /// creature, ends with `count` creatures and `from` with the rest of the two stacks' total;
    /// both stay, each within a stack's most ([`split_range`]).
    Split {
        /// The stack split.
        from: ArmySlot,
        /// The slot that takes a part.
        to: ArmySlot,
        /// `to`'s count after the split.
        count: u32,
    },
    /// Dismisses the stack at `at` (ADV-ARM-004): it leaves; a hero's last stack stays.
    Dismiss {
        /// The stack.
        at: ArmySlot,
    },
}

impl ArmyCommand {
    /// The army the command takes a stack from, and the other army it changes, if any.
    pub(crate) fn armies(&self) -> (ArmyRef, Option<ArmyRef>) {
        match *self {
            Self::Move { from, to } | Self::Split { from, to, .. } => {
                (from.army, (to.army != from.army).then_some(to.army))
            }
            Self::Dismiss { at } => (at.army, None),
        }
    }
}

/// Why an army command was rejected; nothing changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmyCommandError {
    /// A hero is not on the map or not the player's (one error for both, as `MoveHero`'s).
    NotYourHero(HeroId),
    /// The command names the group, and no army dialog's group was given.
    NoGroup,
    /// A slot is beyond the ruleset's army slots.
    NoSlot(u8),
    /// The slot a stack is taken from holds none.
    NoStack(u8),
    /// The command moves or splits a stack onto its own slot.
    SameSlot,
    /// A merge would pass a stack's most creatures (ADV-ARM-007).
    TooManyCreatures,
    /// A hero's last stack would leave its army: merged into or moved to another army, or
    /// dismissed (ADV-ARM-002, -004).
    LastStack,
    /// A split's target holds another creature.
    OtherCreature,
    /// A split's count is outside the range the two stacks allow ([`split_range`]).
    SplitCount(u32),
}

impl fmt::Display for ArmyCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotYourHero(hero) => write!(f, "hero {} is not the player's", hero.raw()),
            Self::NoGroup => f.write_str("no army dialog's group is open"),
            Self::NoSlot(slot) => write!(f, "slot {slot} is beyond the army's slots"),
            Self::NoStack(slot) => write!(f, "slot {slot} holds no stack"),
            Self::SameSlot => f.write_str("the stack would go onto its own slot"),
            Self::TooManyCreatures => f.write_str("the stack would hold more than a stack holds"),
            Self::LastStack => f.write_str("a hero's last stack would leave its army"),
            Self::OtherCreature => f.write_str("the split's target holds another creature"),
            Self::SplitCount(count) => write!(f, "a split cannot leave {count} in its target"),
        }
    }
}

impl core::error::Error for ArmyCommandError {}

/// The counts a split may leave in its target (ADV-ARM-003, ADV-ARM-007; E08 decision 98): a
/// source of `source` creatures and a target of `target` (0 for an empty slot) hold `t` together;
/// the target may end with any count that leaves both stacks at least one and at most `most`
/// creatures: from max(1, t − most) to min(t − 1, most); empty when `t` is 1. The original's
/// dialog allows [m₂, t − m₁], whose bounds refuse only the 0 and the full count a move covers
/// (survey `E08-army.md` §3.3; A-16 by E6-4).
// rule: ADV-ARM-003, ADV-ARM-007
pub fn split_range(source: u32, target: u32, most: u32) -> RangeInclusive<u32> {
    let total = u64::from(source) + u64::from(target);
    let low = total.saturating_sub(u64::from(most)).max(1);
    let high = total.saturating_sub(1).min(u64::from(most));
    let narrow = |n: u64| u32::try_from(n).unwrap_or(u32::MAX);
    narrow(low)..=narrow(high)
}

/// The stack at `slot` of `army`, if one.
fn at(army: &[ArmyStack], slot: u8) -> Option<ArmyStack> {
    army.iter().find(|s| s.slot == slot).copied()
}

/// `army` without its stack at `slot`.
fn without(army: &[ArmyStack], slot: u8) -> Vec<ArmyStack> {
    army.iter().filter(|s| s.slot != slot).copied().collect()
}

/// `army` with `stack` in its slot, replacing the stack there, in slot order.
fn with(army: &[ArmyStack], stack: ArmyStack) -> Vec<ArmyStack> {
    let mut out = without(army, stack.slot);
    let index = out
        .iter()
        .position(|s| s.slot > stack.slot)
        .unwrap_or(out.len());
    out.insert(index, stack);
    out
}

/// The source army's and, for another army, the target army's stacks after a command.
pub(crate) type Plan = (Vec<ArmyStack>, Option<Vec<ArmyStack>>);

/// Plans `command` on its source army `source` (its stacks by slot, and whether it is a hero's,
/// whose last stack is protected) and, when the command names another army, `target` (`rules`:
/// the army slots and a stack's most creatures): the armies' stacks after it, or why it is
/// rejected. Checked in order: the slots, the source's stack, the same slot, then the command's
/// own rule.
// rule: ADV-ARM-001, ADV-ARM-002, ADV-ARM-003, ADV-ARM-004, ADV-ARM-005, ADV-ARM-007
pub(crate) fn plan(
    command: &ArmyCommand,
    (source, hero): (&[ArmyStack], bool),
    target: Option<&[ArmyStack]>,
    (slots, most): (u8, u32),
) -> Result<Plan, ArmyCommandError> {
    let check = |slot: u8| {
        if slot < slots {
            Ok(())
        } else {
            Err(ArmyCommandError::NoSlot(slot))
        }
    };
    let (from, to) = match *command {
        ArmyCommand::Move { from, to } | ArmyCommand::Split { from, to, .. } => (from, Some(to)),
        ArmyCommand::Dismiss { at } => (at, None),
    };
    check(from.slot)?;
    to.map_or(Ok(()), |to| check(to.slot))?;
    let stack = at(source, from.slot).ok_or(ArmyCommandError::NoStack(from.slot))?;
    if to.is_some_and(|to| target.is_none() && to.slot == from.slot) {
        return Err(ArmyCommandError::SameSlot);
    }
    // A hero's last stack never leaves its army.
    let last = hero && source.len() == 1;
    let leave = |rest: Vec<ArmyStack>| {
        if last {
            Err(ArmyCommandError::LastStack)
        } else {
            Ok(rest)
        }
    };
    match *command {
        ArmyCommand::Dismiss { .. } => Ok((leave(without(source, from.slot))?, None)),
        ArmyCommand::Move { to, .. } => {
            let other = at(target.unwrap_or(source), to.slot);
            let moved = ArmyStack {
                slot: to.slot,
                ..stack
            };
            match (other, target) {
                // Into an empty slot.
                (None, None) => Ok((with(&without(source, from.slot), moved), None)),
                (None, Some(target)) => {
                    let rest = leave(without(source, from.slot))?;
                    Ok((rest, Some(with(target, moved))))
                }
                // Into a stack of the same creature.
                (Some(other), _) if other.creature == stack.creature => {
                    let count = stack
                        .count
                        .checked_add(other.count)
                        .filter(|&count| count <= most)
                        .ok_or(ArmyCommandError::TooManyCreatures)?;
                    let merged = ArmyStack { count, ..moved };
                    match target {
                        None => Ok((with(&without(source, from.slot), merged), None)),
                        Some(target) => Ok((
                            leave(without(source, from.slot))?,
                            Some(with(target, merged)),
                        )),
                    }
                }
                // A swap.
                (Some(other), _) => {
                    let back = ArmyStack {
                        slot: from.slot,
                        ..other
                    };
                    match target {
                        None => Ok((with(&with(source, back), moved), None)),
                        Some(target) => Ok((with(source, back), Some(with(target, moved)))),
                    }
                }
            }
        }
        ArmyCommand::Split { to, count, .. } => {
            let other = at(target.unwrap_or(source), to.slot);
            if other.is_some_and(|o| o.creature != stack.creature) {
                return Err(ArmyCommandError::OtherCreature);
            }
            let held = other.map_or(0, |o| o.count);
            if !split_range(stack.count, held, most).contains(&count) {
                return Err(ArmyCommandError::SplitCount(count));
            }
            // In the range, so both counts are from 1 to `most`.
            let total = u64::from(stack.count) + u64::from(held);
            let rest = u32::try_from(total - u64::from(count)).unwrap_or(u32::MAX);
            let kept = ArmyStack {
                count: rest,
                ..stack
            };
            let part = ArmyStack {
                slot: to.slot,
                count,
                ..stack
            };
            match target {
                None => Ok((with(&with(source, kept), part), None)),
                Some(target) => Ok((with(source, kept), Some(with(target, part)))),
            }
        }
    }
}

#[cfg(test)]
mod tests;
