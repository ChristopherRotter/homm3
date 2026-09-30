//! Errors of the battle context (part of its domain language, 02-architecture §11).

use core::fmt;

use h3_contracts::BattleSide;

/// Why a [`crate::BattleSetup`] cannot start a battle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupError {
    /// A side has no stacks.
    EmptyArmy(BattleSide),
    /// A slot is outside `0..army_slots`.
    SlotOutOfRange {
        /// The side.
        side: BattleSide,
        /// The slot.
        slot: u8,
    },
    /// Two stacks of one side use the same slot.
    DuplicateSlot {
        /// The side.
        side: BattleSide,
        /// The slot.
        slot: u8,
    },
    /// A stack has no creatures.
    EmptyStack {
        /// The side.
        side: BattleSide,
        /// The slot.
        slot: u8,
    },
    /// A stack's creature type is not part of the ruleset.
    UnknownCreature {
        /// The side.
        side: BattleSide,
        /// The slot.
        slot: u8,
    },
}

impl fmt::Display for SetupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyArmy(side) => write!(f, "{side:?} army has no stacks"),
            Self::SlotOutOfRange { side, slot } => {
                write!(f, "{side:?} slot {slot} is out of range")
            }
            Self::DuplicateSlot { side, slot } => write!(f, "{side:?} slot {slot} is used twice"),
            Self::EmptyStack { side, slot } => write!(f, "{side:?} slot {slot} has no creatures"),
            Self::UnknownCreature { side, slot } => {
                write!(f, "{side:?} slot {slot} has an unknown creature type")
            }
        }
    }
}

impl core::error::Error for SetupError {}

/// Why a command was rejected. A rejected command leaves the state unchanged (ADR-0004 §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandError {
    /// The battle is already over.
    BattleOver,
    /// It is not the actor's turn (the active stack belongs to the other side).
    NotYourTurn,
    /// The active stack has already waited this round (BAT-TRN-002).
    AlreadyWaited,
    /// The hex is not a standable hex of the field.
    InvalidHex,
    /// The hex cannot be reached this turn (too far, occupied or blocked, BAT-MOV-001).
    Unreachable,
    /// The target stack does not exist or is dead.
    NoSuchTarget,
    /// The target is not an enemy stack.
    NotAnEnemy,
    /// The attack position is not adjacent to the target (BAT-ATK-001).
    NotAdjacent,
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::BattleOver => "the battle is over",
            Self::NotYourTurn => "it is not your turn",
            Self::AlreadyWaited => "the stack has already waited this round",
            Self::InvalidHex => "not a standable hex",
            Self::Unreachable => "the hex cannot be reached this turn",
            Self::NoSuchTarget => "no such living stack",
            Self::NotAnEnemy => "the target is not an enemy",
            Self::NotAdjacent => "the attack position is not adjacent to the target",
        })
    }
}

impl core::error::Error for CommandError {}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::string::ToString;

    #[test]
    fn messages() {
        use BattleSide::{Attacker, Defender};
        assert_eq!(
            SetupError::EmptyArmy(Attacker).to_string(),
            "Attacker army has no stacks"
        );
        assert_eq!(
            SetupError::SlotOutOfRange {
                side: Defender,
                slot: 9
            }
            .to_string(),
            "Defender slot 9 is out of range"
        );
        assert_eq!(
            SetupError::DuplicateSlot {
                side: Attacker,
                slot: 1
            }
            .to_string(),
            "Attacker slot 1 is used twice"
        );
        assert_eq!(
            SetupError::EmptyStack {
                side: Attacker,
                slot: 2
            }
            .to_string(),
            "Attacker slot 2 has no creatures"
        );
        assert_eq!(
            SetupError::UnknownCreature {
                side: Defender,
                slot: 0
            }
            .to_string(),
            "Defender slot 0 has an unknown creature type"
        );
        let all = [
            (CommandError::BattleOver, "the battle is over"),
            (CommandError::NotYourTurn, "it is not your turn"),
            (
                CommandError::AlreadyWaited,
                "the stack has already waited this round",
            ),
            (CommandError::InvalidHex, "not a standable hex"),
            (
                CommandError::Unreachable,
                "the hex cannot be reached this turn",
            ),
            (CommandError::NoSuchTarget, "no such living stack"),
            (CommandError::NotAnEnemy, "the target is not an enemy"),
            (
                CommandError::NotAdjacent,
                "the attack position is not adjacent to the target",
            ),
        ];
        for (e, text) in all {
            assert_eq!(e.to_string(), text);
        }
    }
}
