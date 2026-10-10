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
    /// An obstacle hex is outside the standable field or listed twice.
    InvalidObstacle(u8),
    /// An obstacle's anchor hex is outside the standable field (E04b T5b).
    InvalidAnchor(u8),
    /// More removable obstacles than a battle holds, or one with more hexes than a removable
    /// obstacle has (E04b T5b; `MAX_REMOVABLE_OBSTACLES`, `MAX_REMOVABLE_HEXES`).
    TooManyObstacles,
    /// No free hex is left for a stack.
    NoRoom {
        /// The side.
        side: BattleSide,
        /// The slot.
        slot: u8,
    },
    /// A war machine is in an army slot (BAT-WAR-001).
    WarMachineInArmy {
        /// The side.
        side: BattleSide,
        /// The slot.
        slot: u8,
    },
    /// A war machine is not a known war machine of a supported kind, or a kind is listed twice
    /// (BAT-WAR-001).
    InvalidWarMachine {
        /// The side.
        side: BattleSide,
    },
    /// The battlefield's terrain is not part of the ruleset (BAT-TER-001).
    UnknownTerrain,
    /// A side's hero cannot be used (BAT-HER-001).
    InvalidHero {
        /// The side.
        side: BattleSide,
        /// What is wrong with the hero.
        error: HeroError,
    },
}

/// Why the battle cannot use a hero (BAT-HER-001). The battle checks only what its rules need
/// to run soundly, not whether a real hero could have the values (OPEN-QUESTIONS Q31).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeroError {
    /// The hero type is not part of the ruleset.
    UnknownHeroType,
    /// The Tactics level is above Expert (3).
    TacticsLevel,
    /// The damage reduction exceeds 10 000 basis points.
    DamageReduction,
    /// A commanded Ballista has no shots or a double-damage chance above 10 000 basis points.
    BallistaControl,
    /// The surrender discount exceeds 10 000 basis points.
    SurrenderDiscount,
    /// More effects than `MAX_BATTLE_EFFECTS`.
    TooManyEffects,
    /// An effect names a creature type that is not part of the ruleset.
    UnknownCreature,
    /// An alignment group names a faction that is not part of the ruleset.
    UnknownFaction,
    /// A spell mastery is not Basic (1) to Expert (3).
    MasteryLevel,
    /// A spell in the spell book, a mastery or a spell effect is not part of the ruleset.
    UnknownSpell,
    /// A spell-damage effect names a spell school the ruleset lacks.
    UnknownSchool,
    /// More than `MAX_SPELL_BONUSES` spells have spell-damage or specialty effects of their own.
    TooManySpellBonuses,
    /// A spell is in the spell book twice.
    DuplicateSpell,
    /// A spell in the spell book is a creature ability.
    NotAHeroSpell,
}

impl fmt::Display for HeroError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnknownHeroType => "unknown hero type",
            Self::TacticsLevel => "Tactics level above 3",
            Self::DamageReduction => "damage reduction above 10000 basis points",
            Self::BallistaControl => {
                "commanded Ballista without shots or with a chance above 10000 basis points"
            }
            Self::SurrenderDiscount => "surrender discount above 10000 basis points",
            Self::TooManyEffects => "too many effects",
            Self::UnknownCreature => "an effect names an unknown creature type",
            Self::UnknownFaction => "an alignment group names an unknown faction",
            Self::MasteryLevel => "spell mastery not 1..=3",
            Self::UnknownSpell => "unknown spell in the spell book, the masteries or an effect",
            Self::UnknownSchool => "an effect for an unknown spell school",
            Self::TooManySpellBonuses => "too many spells with bonuses of their own",
            Self::DuplicateSpell => "a spell is listed twice in the spell book or the masteries",
            Self::NotAHeroSpell => "a creature ability in the spell book",
        })
    }
}

impl core::error::Error for HeroError {}

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
            Self::InvalidObstacle(hex) => {
                write!(f, "obstacle hex {hex} is not a standable hex or repeats")
            }
            Self::InvalidAnchor(hex) => write!(f, "obstacle anchor {hex} is not a standable hex"),
            Self::TooManyObstacles => write!(
                f,
                "more than 16 removable obstacles, or one blocking more than 8 hexes"
            ),
            Self::NoRoom { side, slot } => write!(f, "no free hex for {side:?} slot {slot}"),
            Self::WarMachineInArmy { side, slot } => {
                write!(f, "{side:?} slot {slot} holds a war machine")
            }
            Self::InvalidWarMachine { side } => {
                write!(f, "{side:?} war machine is not supported or listed twice")
            }
            Self::UnknownTerrain => f.write_str("the terrain is not part of the ruleset"),
            Self::InvalidHero { side, error } => write!(f, "{side:?} hero: {error}"),
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
    /// The hex cannot be reached this turn (too far, occupied or blocked, BAT-MOV-001), or, for
    /// an attack, the walk there stops in a pit the side sees (BAT-MAG-032, quirk BAT-QUIRK-2
    /// fixed).
    Unreachable,
    /// The target stack does not exist or is dead.
    NoSuchTarget,
    /// The target is not an enemy stack.
    NotAnEnemy,
    /// The attack position is not adjacent to the target (BAT-ATK-001).
    NotAdjacent,
    /// The active stack can shoot, so it may not attack in melee (BAT-RNG-001).
    MustShoot,
    /// The active stack cannot shoot: not a shooter, no shots left, or blocked (BAT-RNG-001).
    CannotShoot,
    /// The aimed hex is not one the stack may aim at on the target (BAT-RNG-004).
    NotOnTarget,
    /// The active stack is a war machine, which never attacks in melee (BAT-WAR-001).
    NoMelee,
    /// The active stack is not a First Aid Tent its side commands (BAT-WAR-007).
    CannotHeal,
    /// The target is not a hurt allied stack the tent may heal (BAT-WAR-003, BAT-WAR-007).
    NotAPatient,
    /// The command is not allowed during the tactics phase (BAT-TAC-002).
    InTacticsPhase,
    /// No tactics phase is running (BAT-TAC-002).
    NoTacticsPhase,
    /// The hex lies outside the tactics zone (BAT-TAC-001).
    OutsideZone,
    /// The side may not retreat: it has no hero, or a hero has a `NoRetreat` effect
    /// (BAT-FLE-001).
    CannotRetreat,
    /// The side may not surrender: it may not retreat, the enemy has no hero, or the ruleset
    /// prices no surrenders (BAT-FLE-002).
    CannotSurrender,
    /// The side's funds do not cover the price of its surrender (BAT-FLE-002).
    CannotAffordSurrender,
    /// The side has no hero with this spell in its spell book (BAT-MAG-001).
    NotInSpellBook,
    /// The spell is not a combat spell (BAT-MAG-001).
    NotACombatSpell,
    /// The side's hero has cast all its spells this round (BAT-MAG-001).
    AlreadyCast,
    /// The spell has no effects the battle implements: its pack lists none yet (the effect kinds
    /// after damage come with E04b T3–T7, ADR-0014 point 1).
    NotCastable,
    /// The hero's mana does not cover the spell's cost (BAT-MAG-002).
    NotEnoughMana,
    /// The target is not one the spell may be cast at (BAT-MAG-003, BAT-MAG-048).
    InvalidSpellTarget,
    /// The active stack casts no spell as its action, has no casts left, or no one may cast in
    /// the battle (BAT-MAG-048, BAT-MAG-061).
    CannotCast,
    /// A hero in the battle forbids every cast (BAT-MAG-061: the Orb of Inhibition).
    CastingForbidden,
    /// The spell's level is above the battle's spell level limit (BAT-MAG-062: Recanter's
    /// Cloak).
    SpellLevelBarred,
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
            Self::MustShoot => "the stack can shoot, so it cannot attack in melee",
            Self::CannotShoot => "the stack cannot shoot",
            Self::NotOnTarget => "the stack cannot aim at that hex of the target",
            Self::NoMelee => "war machines do not attack in melee",
            Self::CannotHeal => "the stack cannot heal",
            Self::NotAPatient => "the target is not a hurt allied stack",
            Self::InTacticsPhase => "not allowed during the tactics phase",
            Self::NoTacticsPhase => "no tactics phase is running",
            Self::OutsideZone => "the hex lies outside the tactics zone",
            Self::CannotRetreat => "the side cannot retreat",
            Self::CannotSurrender => "the side cannot surrender",
            Self::CannotAffordSurrender => "the side cannot pay for its surrender",
            Self::NotInSpellBook => "no hero with that spell in its spell book",
            Self::NotACombatSpell => "not a combat spell",
            Self::AlreadyCast => "the hero has already cast this round",
            Self::NotCastable => "the spell cannot be cast in battle yet",
            Self::NotEnoughMana => "not enough mana",
            Self::InvalidSpellTarget => "not a target of that spell",
            Self::CannotCast => "the stack cannot cast",
            Self::CastingForbidden => "no one may cast in this battle",
            Self::SpellLevelBarred => "the spell's level is above the battle's limit",
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
        assert_eq!(
            SetupError::InvalidObstacle(3).to_string(),
            "obstacle hex 3 is not a standable hex or repeats"
        );
        assert_eq!(
            SetupError::InvalidAnchor(0).to_string(),
            "obstacle anchor 0 is not a standable hex"
        );
        assert_eq!(
            SetupError::TooManyObstacles.to_string(),
            "more than 16 removable obstacles, or one blocking more than 8 hexes"
        );
        assert_eq!(
            SetupError::NoRoom {
                side: Defender,
                slot: 4
            }
            .to_string(),
            "no free hex for Defender slot 4"
        );
        assert_eq!(
            SetupError::WarMachineInArmy {
                side: Defender,
                slot: 1
            }
            .to_string(),
            "Defender slot 1 holds a war machine"
        );
        assert_eq!(
            SetupError::InvalidWarMachine { side: Attacker }.to_string(),
            "Attacker war machine is not supported or listed twice"
        );
        assert_eq!(
            SetupError::UnknownTerrain.to_string(),
            "the terrain is not part of the ruleset"
        );
        assert_eq!(
            SetupError::InvalidHero {
                side: Defender,
                error: HeroError::TacticsLevel
            }
            .to_string(),
            "Defender hero: Tactics level above 3"
        );
        let heroes = [
            (HeroError::UnknownHeroType, "unknown hero type"),
            (HeroError::TacticsLevel, "Tactics level above 3"),
            (
                HeroError::DamageReduction,
                "damage reduction above 10000 basis points",
            ),
            (
                HeroError::BallistaControl,
                "commanded Ballista without shots or with a chance above 10000 basis points",
            ),
            (
                HeroError::SurrenderDiscount,
                "surrender discount above 10000 basis points",
            ),
            (HeroError::TooManyEffects, "too many effects"),
            (
                HeroError::UnknownCreature,
                "an effect names an unknown creature type",
            ),
            (
                HeroError::UnknownFaction,
                "an alignment group names an unknown faction",
            ),
            (HeroError::MasteryLevel, "spell mastery not 1..=3"),
            (
                HeroError::UnknownSpell,
                "unknown spell in the spell book, the masteries or an effect",
            ),
            (
                HeroError::UnknownSchool,
                "an effect for an unknown spell school",
            ),
            (
                HeroError::TooManySpellBonuses,
                "too many spells with bonuses of their own",
            ),
            (
                HeroError::DuplicateSpell,
                "a spell is listed twice in the spell book or the masteries",
            ),
            (
                HeroError::NotAHeroSpell,
                "a creature ability in the spell book",
            ),
        ];
        for (error, text) in heroes {
            assert_eq!(error.to_string(), text);
        }
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
            (
                CommandError::MustShoot,
                "the stack can shoot, so it cannot attack in melee",
            ),
            (CommandError::CannotShoot, "the stack cannot shoot"),
            (
                CommandError::NotOnTarget,
                "the stack cannot aim at that hex of the target",
            ),
            (CommandError::NoMelee, "war machines do not attack in melee"),
            (CommandError::CannotHeal, "the stack cannot heal"),
            (
                CommandError::NotAPatient,
                "the target is not a hurt allied stack",
            ),
            (
                CommandError::InTacticsPhase,
                "not allowed during the tactics phase",
            ),
            (CommandError::NoTacticsPhase, "no tactics phase is running"),
            (
                CommandError::OutsideZone,
                "the hex lies outside the tactics zone",
            ),
            (CommandError::CannotRetreat, "the side cannot retreat"),
            (CommandError::CannotSurrender, "the side cannot surrender"),
            (
                CommandError::CannotAffordSurrender,
                "the side cannot pay for its surrender",
            ),
            (
                CommandError::NotInSpellBook,
                "no hero with that spell in its spell book",
            ),
            (CommandError::NotACombatSpell, "not a combat spell"),
            (
                CommandError::AlreadyCast,
                "the hero has already cast this round",
            ),
            (
                CommandError::NotCastable,
                "the spell cannot be cast in battle yet",
            ),
            (CommandError::NotEnoughMana, "not enough mana"),
            (
                CommandError::InvalidSpellTarget,
                "not a target of that spell",
            ),
            (CommandError::CannotCast, "the stack cannot cast"),
            (
                CommandError::CastingForbidden,
                "no one may cast in this battle",
            ),
            (
                CommandError::SpellLevelBarred,
                "the spell's level is above the battle's limit",
            ),
        ];
        for (e, text) in all {
            assert_eq!(e.to_string(), text);
        }
    }
}
