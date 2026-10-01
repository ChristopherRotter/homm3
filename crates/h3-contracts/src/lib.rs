//! Seam types exchanged between bounded contexts (docs/03-domain.md, "Seams").
//!
//! Logic crate: pure, deterministic, `no_std` + `alloc` (ADR-0003, ADR-0005). The lint header
//! below is required and checked by `cargo xtask check-arch`.
#![no_std]
#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::float_arithmetic,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::exit
)]
#![warn(missing_docs)]

extern crate alloc;

use alloc::vec::Vec;

use h3_content::{CreatureTypeId, TerrainId};

/// The two sides of a battle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BattleSide {
    /// The side that started the engagement (acts first on speed ties at the start, BAT-TRN-001).
    Attacker,
    /// The other side.
    Defender,
}

impl BattleSide {
    /// Both sides, attacker first.
    pub const BOTH: [Self; 2] = [Self::Attacker, Self::Defender];

    /// 0 for the attacker, 1 for the defender.
    pub const fn index(self) -> usize {
        match self {
            Self::Attacker => 0,
            Self::Defender => 1,
        }
    }

    /// The other side.
    pub const fn opponent(self) -> Self {
        match self {
            Self::Attacker => Self::Defender,
            Self::Defender => Self::Attacker,
        }
    }
}

/// One stack of an army: `count` creatures of one type in an army slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArmyStack {
    /// Army slot (0-based; slot order decides placement and speed ties, BAT-SET-001/BAT-TRN-001).
    pub slot: u8,
    /// Creature type, a handle of the ruleset the battle runs with.
    pub creature: CreatureTypeId,
    /// Number of creatures (≥ 1).
    pub count: u32,
}

/// An army as it enters or leaves a battle.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Army {
    /// Stacks, in any order (slots must be distinct).
    pub stacks: Vec<ArmyStack>,
}

/// Everything the battle context needs to start a battle (docs/03-domain.md, "Seams"). All
/// derived values are computed by the caller before the seam.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BattleSetup {
    /// The attacking army.
    pub attacker: Army,
    /// The defending army.
    pub defender: Army,
    /// Hexes blocked by obstacles (row-major hex indices of the battlefield, each once; E03).
    pub obstacles: Vec<u8>,
    /// Whether each side (indexed by [`BattleSide::index`]) is led by a hero (E03, Q17; the hero
    /// itself arrives in E04). Without a hero a side gets no lucky strikes in profiles with the
    /// quirk `quirk.battle.no_luck_without_hero` (Q-0003).
    pub led_by_hero: [bool; 2],
    /// The battlefield's terrain, if any: creatures of factions native to it get the native
    /// terrain bonus (BAT-TER-001). Obstacles for it are generated before the seam (BAT-GEO-004).
    pub terrain: Option<TerrainId>,
    /// War machines per side (indexed by [`BattleSide::index`]): creature types with flag
    /// `war_machine`, at most one of each kind (E03, BAT-WAR-001; heroes bring them in E04).
    pub war_machines: [Vec<CreatureTypeId>; 2],
}

impl BattleSetup {
    /// The army of `side`.
    pub fn army(&self, side: BattleSide) -> &Army {
        match side {
            BattleSide::Attacker => &self.attacker,
            BattleSide::Defender => &self.defender,
        }
    }
}

/// The result of a finished battle, applied back by the orchestrator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleOutcome {
    /// The winning side.
    pub winner: BattleSide,
    /// Surviving stacks per side (indexed by [`BattleSide::index`]), in slot order.
    pub survivors: [Army; 2],
    /// Surviving war machines per side (indexed by [`BattleSide::index`]).
    pub war_machines: [Vec<CreatureTypeId>; 2],
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn sides() {
        assert_eq!(BattleSide::Attacker.index(), 0);
        assert_eq!(BattleSide::Defender.index(), 1);
        assert_eq!(BattleSide::Attacker.opponent(), BattleSide::Defender);
        assert_eq!(BattleSide::Defender.opponent(), BattleSide::Attacker);
        assert_eq!(
            BattleSide::BOTH,
            [BattleSide::Attacker, BattleSide::Defender]
        );
    }

    #[test]
    fn setup_armies() {
        let stack = ArmyStack {
            slot: 0,
            creature: CreatureTypeId::from_raw(1),
            count: 5,
        };
        let setup = BattleSetup {
            attacker: Army {
                stacks: vec![stack],
            },
            defender: Army::default(),
            obstacles: vec![],
            led_by_hero: [true, false],
            terrain: None,
            war_machines: [vec![], vec![]],
        };
        assert_eq!(setup.army(BattleSide::Attacker).stacks, [stack]);
        assert!(setup.army(BattleSide::Defender).stacks.is_empty());
    }
}
