//! Observations and event projection (ADR-0004 §6, docs/06-security.md "Information hiding").
//!
//! The battlefield is fully visible to both sides, so the observation is the state minus what
//! no player may know: the RNG state (it would predict rolls) and the private part of the heroes
//! (their Tactics level; ADR-0013 point 9). Hidden battle information added later (an opponent's
//! spell book, E04b) is projected away here too.

use h3_contracts::{BattleResult, BattleSide};
use h3_kernel::FixedVec;

use crate::event::{Event, Visibility};
use crate::geometry::HexSet;
use crate::hero::HeroView;
use crate::state::{BattleState, STACK_CAPACITY, Stack, StackId};

/// What one side may know about a battle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    /// The side this observation is for.
    pub viewer: BattleSide,
    /// Current round.
    pub round: u16,
    /// The stack whose turn it is.
    pub active: Option<StackId>,
    /// Side of the last stack that took a turn.
    pub last_side_acted: Option<BattleSide>,
    /// How the battle ended, once it is over.
    pub result: Option<BattleResult>,
    /// All stacks (public in battle).
    pub stacks: FixedVec<Stack, STACK_CAPACITY>,
    /// Hexes blocked by obstacles (public).
    pub obstacles: HexSet,
    /// The heroes leading the sides, their public part (identity, primary skills, mana;
    /// ADR-0013 point 9), indexed by [`BattleSide::index`].
    pub heroes: [Option<HeroView>; 2],
    /// The battlefield's terrain (public).
    pub terrain: Option<h3_content::TerrainId>,
}

/// The observation of `state` for `viewer`.
pub fn observe(state: &BattleState, viewer: BattleSide) -> Observation {
    Observation {
        viewer,
        round: state.round,
        active: state.active,
        last_side_acted: state.last_side_acted,
        result: state.result,
        stacks: state.stacks,
        obstacles: state.obstacles,
        heroes: BattleSide::BOTH.map(|side| state.hero(side)),
        terrain: state.terrain,
    }
}

/// The event as `viewer` may see it, or `None` if it is hidden from them.
pub fn project(event: &Event, viewer: BattleSide) -> Option<Event> {
    match event.visibility {
        Visibility::Public => Some(*event),
        Visibility::Side(side) if side == viewer => Some(*event),
        Visibility::Side(_) => None,
    }
}
