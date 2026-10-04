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

mod hero;

use alloc::vec::Vec;

pub use hero::{
    BallistaControl, BattleEffect, BattleHero, BattleMagic, CatapultControl, DamageBonuses,
    MAX_BATTLE_EFFECTS, MachineControl, StatOp, TentControl,
};

use h3_content::{CreatureTypeId, Formation, TerrainId};

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
    /// The hero leading each side, if any (indexed by [`BattleSide::index`]; ADR-0013 point 5).
    /// Without a hero a side gets no lucky strikes in profiles with the quirk
    /// `quirk.battle.no_luck_without_hero` (Q-0003). The battle checks a hero only for soundness
    /// (BAT-HER-001, OPEN-QUESTIONS Q31).
    pub heroes: [Option<BattleHero>; 2],
    /// The battlefield's terrain, if any: creatures of factions native to it get the native
    /// terrain bonus (BAT-TER-001). Obstacles for it are generated before the seam (BAT-GEO-004).
    pub terrain: Option<TerrainId>,
    /// War machines per side (indexed by [`BattleSide::index`]): creature types with flag
    /// `war_machine`, at most one of each kind (E03, BAT-WAR-001); a hero's come from its
    /// artifacts (`h3_heroes::war_machines`).
    pub war_machines: [Vec<CreatureTypeId>; 2],
    /// Each side's formation (indexed by [`BattleSide::index`]), which decides the starting
    /// rows of its army (BAT-SET-001; E03, Q25). Default: loose.
    pub formations: [Formation; 2],
    /// What each side's owner can pay for a surrender (indexed by [`BattleSide::index`]), in the
    /// ruleset's surrender resource (BAT-FLE-002; ignored for a side without a hero). Private to
    /// the side.
    pub funds: [u32; 2],
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

/// How a battle ended (BAT-END-001/002, BAT-FLE-001/002).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BattleResult {
    /// The side that still has living stacks after the other side has none.
    Victory(BattleSide),
    /// Neither side has living stacks left: both were wiped out by the same action (Q24).
    Draw,
    /// The side retreated (BAT-FLE-001); the other side wins. The consequences (a retreating
    /// side loses its army) are the orchestrator's (ADR-0013 point 5).
    Retreat(BattleSide),
    /// The side surrendered (BAT-FLE-002); the other side wins. The consequences (a
    /// surrendering side pays and keeps its army) are the orchestrator's.
    Surrender(BattleSide),
}

impl BattleResult {
    /// The winning side, or `None` for a draw: the opponent of a side that retreated or
    /// surrendered.
    pub const fn winner(self) -> Option<BattleSide> {
        match self {
            Self::Victory(side) => Some(side),
            Self::Draw => None,
            Self::Retreat(side) | Self::Surrender(side) => Some(side.opponent()),
        }
    }
}

/// The result of a finished battle, applied back by the orchestrator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleOutcome {
    /// How the battle ended: a victory of one side, a draw, a retreat or a surrender.
    pub result: BattleResult,
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
    fn results() {
        assert_eq!(
            BattleResult::Victory(BattleSide::Defender).winner(),
            Some(BattleSide::Defender)
        );
        assert_eq!(BattleResult::Draw.winner(), None);
        assert_eq!(
            BattleResult::Retreat(BattleSide::Attacker).winner(),
            Some(BattleSide::Defender)
        );
        assert_eq!(
            BattleResult::Surrender(BattleSide::Defender).winner(),
            Some(BattleSide::Attacker)
        );
    }

    #[test]
    fn new_battle_hero_is_neutral() {
        let hero = BattleHero::new(h3_content::HeroTypeId::from_raw(3));
        assert_eq!(hero.hero_type.raw(), 3);
        assert_eq!(
            [hero.attack, hero.defense, hero.power, hero.knowledge],
            [0; 4]
        );
        assert_eq!(hero.tactics, 0);
        assert_eq!(hero.damage, DamageBonuses::default());
        assert_eq!(hero.war_machines, MachineControl::default());
        assert_eq!(hero.surrender_discount_bp, 0);
        assert!(hero.effects.is_empty());
        assert_eq!(hero.magic, BattleMagic::default());
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
            heroes: [
                Some(BattleHero::new(h3_content::HeroTypeId::from_raw(0))),
                None,
            ],
            terrain: None,
            war_machines: [vec![], vec![]],
            formations: [Formation::Tight, Formation::Loose],
            funds: [1000, 0],
        };
        assert_eq!(setup.army(BattleSide::Attacker).stacks, [stack]);
        assert!(setup.army(BattleSide::Defender).stacks.is_empty());
        assert!(setup.heroes[0].is_some() && setup.heroes[1].is_none());
    }
}
