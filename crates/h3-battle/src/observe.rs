//! Observations and event projection (ADR-0004 §6, docs/06-security.md "Information hiding").
//!
//! The battlefield is fully visible to both sides, so the observation is the state minus what
//! the viewer may not know: the RNG state (it would predict rolls) and the private part of the
//! other side's hero (ADR-0013 point 9: its skill values, surrender discount, funds and price)
//! the other side's hidden spell obstacles the viewer does not see (ADR-0014 point 7,
//! BAT-MAG-031), the spell a random caster the viewer does not control drew (ADR-0014 point 8,
//! BAT-MAG-049) and, at a town, the defender's Escape Tunnel, which shows to the defender only,
//! in its option to retreat too (BAT-SIE-024, BAT-Q50 (4)). The viewer's own hero's private part is shown to the viewer ([`OwnHeroView`]):
//! its spell book, masteries and spell bonuses among them (ADR-0014 point 8), and its hidden
//! magic, which the other side infers only from the casts it may not make (BAT-Q26). Timed
//! effects on stacks are public.
//!
//! A spectator ([`Viewer::Spectator`], SRV-Q4) sees what is public to both sides: neither hero's
//! private part, no drawn spell, no cast candidates, and only the spell obstacles both sides
//! see; it is told only the public events. Its observation is seen from the attacker's seat.
//!
//! The observation is complete (E04b T9a): everything of the state a rule reads that the viewer
//! may know is in it, directly or through the ruleset's data. The RNG, the outcome's bookkeeping
//! (the creatures a vanished clone took along, a surrender's price once paid) and the templates
//! of the stacks a side's spells would create are not.
//!
//! One gap is registered (BAT-STUB-1, BAT-Q35): since the E04b T11b review a rule reads whether a
//! dead created stack vanished when its rounds ran out (its record is then not reused,
//! BAT-MAG-037), and the observation does not tell such a stack from a killed one.

use alloc::vec::Vec;

use h3_content::CreatureTypeId;
use h3_content_battle::{BattleRuleset, BattlefieldId, ObstacleTypeId};
use h3_contracts::{BattleResult, BattleSide};
use h3_kernel::FixedVec;

use crate::effects::ActiveEffect;
use crate::event::{Event, Visibility};
use crate::geometry::{Hex, HexSet};
use crate::hero::{HeroRecord, HeroView, OwnHeroView};
use crate::magic::resistance_bp;
use crate::natural::{MAX_REMOVABLE_OBSTACLES, RemovableObstacle};
use crate::obstacles::{self, SpellObstacle};
use crate::siege::{self, SiegeState};
use crate::state::{BattleState, STACK_CAPACITY, Stack, StackId};
use crate::tactics::TacticsPhase;

/// Who an observation or an event's projection is for (SRV-Q4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Viewer {
    /// One side: what it may know.
    Side(BattleSide),
    /// A spectator: what is public to both sides.
    Spectator,
}

impl From<BattleSide> for Viewer {
    fn from(side: BattleSide) -> Self {
        Self::Side(side)
    }
}

/// What one side, or a spectator, may know about a battle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    /// The side this observation is for; the attacker for a spectator's.
    pub viewer: BattleSide,
    /// A spectator's observation ([`Viewer::Spectator`]): no side's private part.
    pub spectator: bool,
    /// Current round (0 during the tactics phase).
    pub round: u16,
    /// The running tactics phase: its side and zone (public, ADR-0013 point 9).
    pub tactics: Option<TacticsPhase>,
    /// Whether each side may retreat (BAT-FLE-001; public), indexed by [`BattleSide::index`]; at a
    /// town the defender's rests on its Escape Tunnel and shows to the defender only, `false`
    /// (unknown) to the attacker and a spectator (BAT-SIE-024, BAT-Q50 (4)).
    pub may_retreat: [bool; 2],
    /// Whether each side may surrender at all (BAT-FLE-002; public, ADR-0013 point 9), indexed
    /// by [`BattleSide::index`]. Whether the viewer can pay is in [`own_hero`](Self::own_hero).
    pub surrender_possible: [bool; 2],
    /// The round's wait phase has begun: only stacks that waited are left to act (public).
    pub wait_phase: bool,
    /// The stack whose turn it is.
    pub active: Option<StackId>,
    /// The side the speed tie-break counts as activated last (BAT-TRN-001): the side of the last
    /// stack that took a turn, but the defender after the attacker's Catapult (BAT-SIE-011).
    pub last_side_acted: Option<BattleSide>,
    /// How the battle ended, once it is over.
    pub result: Option<BattleResult>,
    /// All stacks (public in battle).
    pub stacks: FixedVec<Stack, STACK_CAPACITY>,
    /// The rounds left of each clone, by stack id (0: not a clone; public, BAT-MAG-038).
    pub clone_rounds: [u16; STACK_CAPACITY],
    /// Per stack, by stack id: for a clone its original, for a stack with a living clone that
    /// clone; under quirk BAT-QUIRK-7 a crowded original that became its own clone links to
    /// itself (public, BAT-MAG-038).
    pub clone_links: [Option<StackId>; STACK_CAPACITY],
    /// Per side, by [`BattleSide::index`], the creature its Summon Elemental casts are bound to
    /// once it cast one (public: every cast is; BAT-MAG-039).
    pub summoned: [Option<CreatureTypeId>; 2],
    /// Per stack, by stack id, the creatures raised for this battle only (public, BAT-MAG-035).
    pub temporary: [u32; STACK_CAPACITY],
    /// Per stack, by stack id, its corpse: 0 while it lives or until its death is marked, then the
    /// order of its death among the battle's corpses, [`NO_CORPSE`](crate::NO_CORPSE) if it left
    /// none (public, BAT-MAG-035, BAT-MAG-036).
    pub corpses: [u16; STACK_CAPACITY],
    /// Per stack, by stack id, its binders: the stacks whose strike bound it, a bit per stack id
    /// (public, BAT-MAG-045).
    pub binders: [u64; STACK_CAPACITY],
    /// Per stack, by stack id, the tenths of its hit points Poison took for good (public,
    /// BAT-MAG-044).
    pub poison: [u8; STACK_CAPACITY],
    /// Per stack, by stack id, its hit points per creature at the start if Poison or Age changed
    /// them, else 0 (public, BAT-MAG-044).
    pub start_hp: [u32; STACK_CAPACITY],
    /// The casts each stack has left, by stack id (public: every cast is; BAT-MAG-048).
    pub casts_left: [u8; STACK_CAPACITY],
    /// The spell the active stack drew at its activation, if it is a random caster the viewer
    /// controls (private to its controller until it casts, ADR-0014 point 8; BAT-MAG-049).
    pub drawn: Option<h3_content::SpellId>,
    /// Per stack, by stack id, how many spells the active stack's cast could draw for it, if it is
    /// a helpful caster the viewer commands and may cast now (0 otherwise and for a stack that is
    /// no target; ADR-0014 point 9, BAT-MAG-050).
    pub cast_candidates: [u8; STACK_CAPACITY],
    /// Per side, by [`BattleSide::index`], the round starts since its periodic casters cast, at
    /// most 255, which every battle starts with (public: every cast is; BAT-MAG-053).
    pub periodic_rounds: [u8; 2],
    /// Hexes blocked by natural obstacles (public).
    pub obstacles: HexSet,
    /// The natural obstacles a spell can remove, with their anchors and types (public;
    /// BAT-MAG-034).
    pub removable_obstacles: FixedVec<RemovableObstacle, MAX_REMOVABLE_OBSTACLES>,
    /// The absolute obstacle's type, if the setup named one (public; for viewers, BAT-GEO-003).
    pub absolute_obstacle: Option<ObstacleTypeId>,
    /// The anchor hexes that belong to their obstacles (quirk BAT-QUIRK-3; public, BAT-GEO-003).
    pub obstacle_anchors: HexSet,
    /// The hexes a removal of a natural obstacle cleared: they lead to no obstacle any more
    /// (public, BAT-MAG-034).
    pub obstacles_cleared: HexSet,
    /// The heroes leading the sides, their public part (identity, primary skills, mana, the war
    /// machines they command; ADR-0013 point 9), indexed by [`BattleSide::index`].
    pub heroes: [Option<HeroView>; 2],
    /// The change of each side's hero's spell costs from the creatures in the armies, indexed by
    /// [`BattleSide::index`] (BAT-MAG-054; public through the armies; 0 without a hero).
    pub cost_change: [i8; 2],
    /// The private part of the viewer's own hero, if the viewer is led by one.
    pub own_hero: Option<OwnHeroView>,
    /// The battlefield's terrain (public).
    pub terrain: Option<h3_content::TerrainId>,
    /// The special battlefield, if any (public; BAT-TER-002).
    pub battlefield: Option<BattlefieldId>,
    /// The battle is at an anti-magic garrison (public; BAT-MAG-061).
    pub anti_magic: bool,
    /// The game's difficulty, 0 (Easy) to 4 (Impossible) (public; ADR-0023 point 7, BAT-SIE-023).
    pub difficulty: u8,
    /// Whether each side's player is human, indexed by [`BattleSide::index`] (public; ADR-0023
    /// point 7, BAT-SIE-023).
    pub human: [bool; 2],
    /// The town the battle is fought at, if any: its walls, gate and moat (public; BAT-SIE-001);
    /// its Escape Tunnel for the defender only (BAT-SIE-024, BAT-Q50 (4)).
    pub siege: Option<SiegeState>,
    /// The timed effects on stacks, by stack and spell (public; BAT-MAG-009).
    pub effects: Vec<ActiveEffect>,
    /// The spell obstacles the viewer sees, in the order they were placed (BAT-MAG-027; Force
    /// Field and Fire Wall are public, a hidden one only when the viewer sees it, BAT-MAG-031); a
    /// spectator's, those both sides see. Here `revealed` tells whether the other side of the
    /// obstacle's sees it (a hidden one placed while its natives saw included, BAT-MAG-031).
    pub spell_obstacles: Vec<SpellObstacle>,
}

impl Observation {
    /// The index in [`removable_obstacles`](Self::removable_obstacles) of the natural obstacle
    /// Remove Obstacle finds on `hex` (BAT-MAG-034): of those that block it or have it as their
    /// anchor (quirk BAT-QUIRK-3), the one placed last; none on a hex a removal cleared.
    pub fn removable_at(&self, hex: Hex) -> Option<usize> {
        if self.obstacles_cleared.contains(hex) {
            return None;
        }
        self.removable_obstacles.iter().rposition(|o| {
            o.hexes().contains(&hex) || (o.anchor() == hex && self.obstacle_anchors.contains(hex))
        })
    }
}

/// The observation of `state` for `viewer`. `ruleset` must be the one the battle was built
/// with (as for [`apply`](crate::apply)); the surrender rules read it.
pub fn observe(state: &BattleState, ruleset: &BattleRuleset, viewer: BattleSide) -> Observation {
    observe_as(state, ruleset, Viewer::Side(viewer))
}

/// The observation of `state` for `viewer`, a side or a spectator (SRV-Q4); see [`observe`].
pub fn observe_as(state: &BattleState, ruleset: &BattleRuleset, viewer: Viewer) -> Observation {
    let side = match viewer {
        Viewer::Side(side) => Some(side),
        Viewer::Spectator => None,
    };
    let seat = side.unwrap_or(BattleSide::Attacker);
    Observation {
        viewer: seat,
        spectator: side.is_none(),
        round: state.round,
        tactics: state.tactics,
        // rule: BAT-SIE-024
        may_retreat: BattleSide::BOTH.map(|of| siege::retreat_seen(state, of, side)),
        surrender_possible: BattleSide::BOTH.map(|side| state.surrender_possible(ruleset, side)),
        wait_phase: state.wait_phase,
        active: state.active,
        last_side_acted: state.last_side_acted,
        result: state.result,
        stacks: state.stacks,
        // stub: BAT-STUB-1
        // `state.created.vanished` (which dead created stacks ran out of rounds) is not shown yet.
        clone_rounds: state.created.rounds,
        clone_links: state.created.links(),
        summoned: state.created.summoned(),
        temporary: state.raised.temporary,
        corpses: state.raised.corpse,
        binders: state.binds.by,
        poison: state.vitality.poison,
        start_hp: state.vitality.base,
        casts_left: state.casts.left,
        // rule: BAT-MAG-049
        drawn: state
            .casts
            .drawn
            .filter(|&(s, _)| state.active == Some(s) && Some(state.at(s).controller()) == side)
            .map(|(_, spell)| spell),
        // rule: BAT-MAG-050
        cast_candidates: side.map_or([0; STACK_CAPACITY], |side| {
            crate::casters::candidates(state, ruleset, side)
        }),
        periodic_rounds: state.casts.periodic,
        obstacles: state.obstacles(),
        removable_obstacles: state.natural.removable_list(),
        absolute_obstacle: state.natural.absolute(),
        obstacle_anchors: state.natural.anchors(),
        obstacles_cleared: state.natural.cleared(),
        heroes: BattleSide::BOTH.map(|side| state.hero(side)),
        cost_change: BattleSide::BOTH
            .map(|side| state.heroes[side.index()].map_or(0, |h| h.magic.cost_change)),
        own_hero: side
            .and_then(|side| state.heroes[side.index()])
            .map(|h| own_hero(state, ruleset, &h, seat)),
        terrain: state.terrain,
        battlefield: state.battlefield,
        anti_magic: state.anti_magic,
        difficulty: state.difficulty,
        human: state.human,
        // rule: BAT-SIE-024
        siege: state.siege.map(|s| s.seen_by(side)),
        effects: state.effects().to_vec(),
        // rule: BAT-MAG-031
        spell_obstacles: obstacles::observed(state, side),
    }
}

/// The private part of `side`'s hero `h` (ADR-0013 point 9, ADR-0014 point 8, BAT-Q26).
fn own_hero(
    state: &BattleState,
    ruleset: &BattleRuleset,
    h: &HeroRecord,
    side: BattleSide,
) -> OwnHeroView {
    let m = &h.magic;
    OwnHeroView {
        tactics: h.tactics,
        damage: h.damage,
        ballista: h.ballista,
        tent: h.tent,
        catapult: h.catapult,
        surrender_discount_bp: h.surrender_discount_bp,
        funds: h.funds,
        surrender_price: state.surrender_price(ruleset, side),
        spell_book: m.book,
        masteries: m.masteries,
        spell_bonuses: m.bonuses,
        magic_resistance_bp: resistance_bp(ruleset, m.resistance),
        duration_bonus: m.duration_bonus,
        no_casting: m.no_casting,
        level_limit: (m.level_limit != u8::MAX).then_some(m.level_limit),
        defences: m.defences,
        negates: m.negates,
        opening: m.opening,
        opening_due: m.opening_due,
        eagle_chance: m.eagle_chance,
        eagle_spells: m.eagle_spells,
        learned: m.learned,
    }
}

/// The event as `viewer` may see it, or `None` if it is hidden from them.
pub fn project(event: &Event, viewer: BattleSide) -> Option<Event> {
    project_as(event, Viewer::Side(viewer))
}

/// The event as `viewer`, a side or a spectator, may see it, or `None` if it is hidden from them:
/// a spectator sees the public events only (SRV-Q4).
pub fn project_as(event: &Event, viewer: Viewer) -> Option<Event> {
    match (event.visibility, viewer) {
        (Visibility::Public, _) => Some(*event),
        (Visibility::Side(side), Viewer::Side(viewer)) if side == viewer => Some(*event),
        (Visibility::Side(_), _) => None,
    }
}
