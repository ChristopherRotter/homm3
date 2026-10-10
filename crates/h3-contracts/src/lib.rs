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
mod ids;

use alloc::vec::Vec;

pub use hero::{
    BallistaControl, BattleEffect, BattleHero, BattleMagic, CatapultControl, DamageBonuses,
    EagleEye, FactionGroup, MAX_BATTLE_EFFECTS, MachineControl, Rate, StatOp, TentControl,
};
pub use ids::{HeroId, PlayerId};

use h3_content::{CreatureTypeId, Formation, SpellId, TerrainId};

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

/// One natural obstacle of a battlefield (E03; as pieces since E04b T5b, OPEN-QUESTIONS Q55):
/// the hexes it blocks and, for a usual obstacle, its anchor hex, which makes it removable
/// (Remove Obstacle, BAT-MAG-034). An absolute obstacle has no anchor and cannot be removed. The
/// anchor (the obstacle's bottom-left hex, BAT-GEO-004) belongs to the obstacle even where the
/// obstacle does not block it, under quirk BAT-QUIRK-3 (BAT-GEO-003).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Obstacle {
    /// The hexes it blocks (row-major hex indices of the battlefield).
    pub hexes: Vec<u8>,
    /// Its anchor hex, if it is a usual (removable) obstacle.
    pub anchor: Option<u8>,
}

/// Everything the battle context needs to start a battle (docs/03-domain.md, "Seams"). All
/// derived values are computed by the caller before the seam.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BattleSetup {
    /// The attacking army.
    pub attacker: Army,
    /// The defending army.
    pub defender: Army,
    /// The natural obstacles (E03; pieces with anchors since E04b T5b): every blocked hex once
    /// across all of them. Generated before the seam for a terrain (BAT-GEO-004).
    pub obstacles: Vec<Obstacle>,
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
    /// Neither side won: both were wiped out by the same action (Q24), or the side still standing
    /// has only stacks created in the battle (summoned stacks, clones: both lose, as in the
    /// original; ADR-0014 point 6). A Pit Lord's Demons are no created stack: they win.
    Draw,
    /// The side retreated (BAT-FLE-001); the other side wins. The consequences (a retreating
    /// side loses its army) are the orchestrator's (ADR-0013 point 5). An opponent left with only
    /// stacks created in the battle (no army survivors, [`BattleOutcome::survivors`]) has lost
    /// too, as in the original (BAT-Q7).
    Retreat(BattleSide),
    /// The side surrendered (BAT-FLE-002); the other side wins. The consequences (a
    /// surrendering side pays and keeps its army) are the orchestrator's. An opponent left with
    /// only stacks created in the battle (no army survivors) has lost too and still receives the
    /// price, as in the original (BAT-Q7).
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

/// Where a battle stack came from ([`StackRecord::origin`]; ADR-0014 point 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackOrigin {
    /// A stack of the side's army, by army slot.
    Army(u8),
    /// One of the side's war machines.
    WarMachine,
    /// A stack the battle created for the side (a summoned stack or a clone, or an army stack a
    /// Clone turned into one under quirk BAT-QUIRK-7): it ends with the battle and gives no
    /// experience; Necromancy counts its killed creatures (BAT-Q7).
    Created,
    /// A Pit Lord's Demons, raised from a dead stack of the side's army slot (or from Demons
    /// raised there before): a stack of that army slot from then on, which stays after the battle
    /// (BAT-Q13).
    Raised(u8),
}

/// What became of one battle stack (BAT-END-003). The creatures it lost are `start - end`, the
/// original's count for the battle's experience and Necromancy (per stack).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackRecord {
    /// Where it came from.
    pub origin: StackOrigin,
    /// Creature type.
    pub creature: CreatureTypeId,
    /// Creatures when it entered the battle: at the battle's start, or when the battle created or
    /// raised it.
    pub start: u32,
    /// Creatures it had when the battle ended, whatever the result, without those raised only for
    /// the battle; a created stack that vanished (a clone whose rounds ran out) keeps those it
    /// vanished with, which were not killed.
    pub end: u32,
}

/// What became of a side's hero in battle (BAT-END-003).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeroOutcome {
    /// Spell points left.
    pub mana: u16,
    /// The spells the hero's Eagle Eye picked in the battle (E04b T8d, BAT-MAG-069): those of
    /// [`EagleEye::spells`] for which one of the enemy hero's casts hit its chance, by spell
    /// handle, each once. Whether the hero learns them is the orchestrator's (BAT-Q24).
    pub learned: Vec<SpellId>,
}

/// The result of a finished battle, applied back by the orchestrator (BAT-END-003). It reports
/// facts; the consequences (casualties, a retreating side's lost army, a surrender's payment,
/// captured artifacts, experience, Necromancy) are the orchestrator's (ADR-0013 point 5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleOutcome {
    /// How the battle ended: a victory of one side, a draw, a retreat or a surrender.
    pub result: BattleResult,
    /// Surviving stacks per side (indexed by [`BattleSide::index`]), in slot order: the army and
    /// raised stacks of [`BattleOutcome::stacks`] with creatures left (a raised stack in the army
    /// slot of the dead stack it rose from).
    pub survivors: [Army; 2],
    /// Surviving war machines per side (indexed by [`BattleSide::index`]): the war machines of
    /// [`BattleOutcome::stacks`] with creatures left.
    pub war_machines: [Vec<CreatureTypeId>; 2],
    /// Every stack of each side (indexed by [`BattleSide::index`]) in stack order, dead ones
    /// included: army stacks in slot order (one a Clone turned into a created stack keeps its
    /// place, BAT-QUIRK-7), war machines, then the stacks the battle created or raised, in the
    /// order of the battle's stack slots. A slot a later created or raised stack took over
    /// reports only that one, as the original's records (ADR-0014 point 6).
    pub stacks: [Vec<StackRecord>; 2],
    /// What the side that surrendered pays (BAT-FLE-002); `None` for any other result.
    pub surrender_cost: Option<u32>,
    /// What became of each side's hero (indexed by [`BattleSide::index`]).
    pub heroes: [Option<HeroOutcome>; 2],
    /// The last round started (0 if the battle ended in the tactics phase, BAT-TAC-001).
    pub rounds: u16,
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
        assert_eq!(hero.damage.melee, Rate::bp(0));
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
            obstacles: vec![Obstacle {
                hexes: vec![40, 41],
                anchor: Some(40),
            }],
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
        assert_eq!(setup.obstacles[0].anchor, Some(40));
        assert_eq!(Obstacle::default().anchor, None);
    }
}
