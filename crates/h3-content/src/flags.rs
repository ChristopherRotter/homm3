//! Creature flags: the closed set of yes/no properties a creature type can have (E02 pack format
//! v2). Packs name them; the compiler turns them into a bitset. Behavior that reads a flag comes
//! with the rule that needs it (E03); until then flags are data only.
//!
//! The set mirrors the flag bits of the original executable's creature table
//! (`docs/formats/installation.md` §5), minus the unidentified bit.

/// One creature flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CreatureFlag {
    /// Occupies two hexes.
    TwoHex,
    /// Flies over obstacles and stacks.
    Flying,
    /// Can shoot.
    Shooter,
    /// Its melee attack also hits the hex behind the target.
    BreathAttack,
    /// A living creature (not undead, elemental, golem or machine).
    Alive,
    /// Can attack walls in sieges.
    Catapult,
    /// A war machine.
    WarMachine,
    /// Takes extra damage from the Slayer spell, tier 1.
    King1,
    /// Tier 2.
    King2,
    /// Tier 3.
    King3,
    /// Immune to mind spells.
    MindImmune,
    /// No penalty when shooting in melee range.
    NoMeleePenalty,
    /// Immune to fire spells.
    FireImmune,
    /// Attacks twice.
    StrikesTwice,
    /// The attacked stack cannot retaliate.
    NoEnemyRetaliation,
    /// Not affected by morale.
    NoMorale,
    /// Undead.
    Undead,
    /// Attacks all adjacent enemies.
    AttacksAllAdjacent,
    /// Its ranged attack hits an area.
    AreaAttack,
    /// A dragon (for effects that target dragons).
    Dragon,
}

impl CreatureFlag {
    /// All flags, in bit order.
    pub const ALL: [Self; 20] = [
        Self::TwoHex,
        Self::Flying,
        Self::Shooter,
        Self::BreathAttack,
        Self::Alive,
        Self::Catapult,
        Self::WarMachine,
        Self::King1,
        Self::King2,
        Self::King3,
        Self::MindImmune,
        Self::NoMeleePenalty,
        Self::FireImmune,
        Self::StrikesTwice,
        Self::NoEnemyRetaliation,
        Self::NoMorale,
        Self::Undead,
        Self::AttacksAllAdjacent,
        Self::AreaAttack,
        Self::Dragon,
    ];

    /// The name used in packs.
    pub const fn name(self) -> &'static str {
        match self {
            Self::TwoHex => "two_hex",
            Self::Flying => "flying",
            Self::Shooter => "shooter",
            Self::BreathAttack => "breath_attack",
            Self::Alive => "alive",
            Self::Catapult => "catapult",
            Self::WarMachine => "war_machine",
            Self::King1 => "king_1",
            Self::King2 => "king_2",
            Self::King3 => "king_3",
            Self::MindImmune => "mind_immune",
            Self::NoMeleePenalty => "no_melee_penalty",
            Self::FireImmune => "fire_immune",
            Self::StrikesTwice => "strikes_twice",
            Self::NoEnemyRetaliation => "no_enemy_retaliation",
            Self::NoMorale => "no_morale",
            Self::Undead => "undead",
            Self::AttacksAllAdjacent => "attacks_all_adjacent",
            Self::AreaAttack => "area_attack",
            Self::Dragon => "dragon",
        }
    }

    /// Looks a flag up by its pack name.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|f| f.name() == name)
    }

    /// The flag's bit in [`CreatureFlags`].
    pub const fn bit(self) -> u32 {
        1 << (self as u32)
    }
}

/// A set of creature flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CreatureFlags(u32);

impl CreatureFlags {
    /// The set with `flag` added.
    #[must_use]
    pub const fn with(self, flag: CreatureFlag) -> Self {
        Self(self.0 | flag.bit())
    }

    /// Whether `flag` is in the set.
    pub const fn contains(self, flag: CreatureFlag) -> bool {
        self.0 & flag.bit() != 0
    }

    /// The raw bits (bit `i` = `CreatureFlag::ALL[i]`).
    pub const fn bits(self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip_and_bits_are_distinct() {
        let mut all = CreatureFlags::default();
        for (i, f) in CreatureFlag::ALL.iter().enumerate() {
            assert_eq!(CreatureFlag::from_name(f.name()), Some(*f));
            assert_eq!(f.bit(), 1 << i);
            assert!(!all.contains(*f));
            all = all.with(*f);
            assert!(all.contains(*f));
        }
        assert_eq!(all.bits(), (1 << CreatureFlag::ALL.len()) - 1);
        assert_eq!(CreatureFlag::from_name("swims"), None);
    }
}
