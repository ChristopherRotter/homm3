//! Observations and event projection (ADR-0004 §6, docs/06-security.md "Information hiding").
//!
//! The battlefield is fully visible to both sides, so the observation is the state minus what
//! the viewer may not know: the RNG state (it would predict rolls) and the private part of the
//! other side's hero (ADR-0013 point 9: its skill values, surrender discount, funds and price).
//! The viewer's own hero's private part is shown to the viewer ([`OwnHeroView`]): its spell book,
//! masteries and spell bonuses among them (ADR-0014 point 8). Timed effects on stacks are public.

use alloc::vec::Vec;

use h3_content::Ruleset;
use h3_contracts::{BattleResult, BattleSide};
use h3_kernel::FixedVec;

use crate::effects::ActiveEffect;
use crate::event::{Event, Visibility};
use crate::geometry::HexSet;
use crate::hero::{HeroView, OwnHeroView};
use crate::obstacles::SpellObstacle;
use crate::state::{BattleState, STACK_CAPACITY, Stack, StackId};
use crate::tactics::TacticsPhase;

/// What one side may know about a battle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    /// The side this observation is for.
    pub viewer: BattleSide,
    /// Current round (0 during the tactics phase).
    pub round: u16,
    /// The running tactics phase: its side and zone (public, ADR-0013 point 9).
    pub tactics: Option<TacticsPhase>,
    /// Whether each side may retreat (BAT-FLE-001; public), indexed by [`BattleSide::index`].
    pub may_retreat: [bool; 2],
    /// Whether each side may surrender at all (BAT-FLE-002; public, ADR-0013 point 9), indexed
    /// by [`BattleSide::index`]. Whether the viewer can pay is in [`own_hero`](Self::own_hero).
    pub surrender_possible: [bool; 2],
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
    /// The heroes leading the sides, their public part (identity, primary skills, mana, the war
    /// machines they command; ADR-0013 point 9), indexed by [`BattleSide::index`].
    pub heroes: [Option<HeroView>; 2],
    /// The private part of the viewer's own hero, if the viewer is led by one.
    pub own_hero: Option<OwnHeroView>,
    /// The battlefield's terrain (public).
    pub terrain: Option<h3_content::TerrainId>,
    /// The timed effects on stacks, by stack and spell (public; BAT-MAG-009).
    pub effects: Vec<ActiveEffect>,
    /// The spell obstacles the viewer sees, in the order they were placed (BAT-MAG-027; Force
    /// Field and Fire Wall are public, ADR-0014 point 7).
    pub spell_obstacles: Vec<SpellObstacle>,
}

/// The observation of `state` for `viewer`. `ruleset` must be the one the battle was built
/// with (as for [`apply`](crate::apply)); the surrender rules read it.
pub fn observe(state: &BattleState, ruleset: &Ruleset, viewer: BattleSide) -> Observation {
    Observation {
        viewer,
        round: state.round,
        tactics: state.tactics,
        may_retreat: BattleSide::BOTH.map(|side| state.may_retreat(side)),
        surrender_possible: BattleSide::BOTH.map(|side| state.surrender_possible(ruleset, side)),
        active: state.active,
        last_side_acted: state.last_side_acted,
        result: state.result,
        stacks: state.stacks,
        obstacles: state.obstacles,
        heroes: BattleSide::BOTH.map(|side| state.hero(side)),
        own_hero: state.heroes[viewer.index()].map(|h| OwnHeroView {
            tactics: h.tactics,
            damage: h.damage,
            ballista: h.ballista,
            tent: h.tent,
            surrender_discount_bp: h.surrender_discount_bp,
            funds: h.funds,
            surrender_price: state.surrender_price(ruleset, viewer),
            spell_book: h.magic.book,
            schools: h.magic.schools,
            spell_bonuses: h.magic.bonuses,
        }),
        terrain: state.terrain,
        effects: state.effects().to_vec(),
        spell_obstacles: state.spell_obstacles().to_vec(),
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
