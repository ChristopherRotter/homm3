//! The computer's numbers the battle's rules use, public for the original's battle AI
//! (`h3-battle-ai`; ADR-0027 point 6): one implementation, which the rules (the automatic
//! turrets and Ballista, `aim.rs`) and the policy call alike, so a change to a rule the
//! computer estimates moves both. It grows with E23's tasks: what the policy needs from the
//! battle's private state is added here by the battle lane.

use h3_content_battle::BattleRuleset;
use h3_contracts::BattleSide;
use h3_kernel::arith::X87;

use crate::aim;
use crate::combat::can_shoot;
use crate::geometry::{Field, Hex, HexSet};
use crate::magic;
use crate::movement::MAX_PATH;
use crate::pathfinder::{AI_RANGE, Goal, Query, Search};
use crate::siege::{self, SiegeState};
use crate::state::{BattleState, StackId};

pub use crate::aim::{Context, Shot};

/// Whether stack `id` of `state` can shoot now (the original's 0x4428F0 as the battle's rule,
/// BAT-RNG-001: a shooter with a shot to use, not blocked or a war machine, not forgetful): the
/// computer's dispatch sends such a stack to its shooter (BAT-AI-001). `id` must be a stack of
/// `state`.
pub fn shoots(state: &BattleState, ruleset: &BattleRuleset, id: StackId) -> bool {
    can_shoot(state, Field::new(ruleset.battle().field), id)
}

/// The stack the computer's shooter `shooter` of `state` shoots (0x41F060, 0x41EB80): the
/// automatic Ballista's target (BAT-WAR-008) and every computer shooter's (BAT-AI-002), the best
/// by not disabled, then the computer's shot value, ties to the later stack; `None` without a
/// stack it may shoot. `shooter` must be a stack of `state`.
pub fn shot_target(
    state: &BattleState,
    ruleset: &BattleRuleset,
    shooter: StackId,
) -> Option<StackId> {
    aim::ballista_target(state, ruleset, Field::new(ruleset.battle().field), shooter)
}

/// What the computer's shooter `shooter` of `state` does (0x41F060; BAT-AI-002, BAT-AI-003): the
/// automatic Ballista's choice for any shooter, an area shooter's target by its value at a hex
/// (0x41EDA0); a wall shooter (the Cyclops) asks `wall` with its side's context and its target's
/// value (0 without one) whether it batters a wall instead, and which (0x41EEA0: the policy's,
/// which draws). `shooter` must be a stack of `state` that can shoot now ([`shoots`]).
pub fn shot(
    state: &BattleState,
    ruleset: &BattleRuleset,
    shooter: StackId,
    wall: impl FnOnce(&Context, i32) -> Option<u8>,
) -> Shot {
    aim::shot(
        state,
        ruleset,
        Field::new(ruleset.battle().field),
        shooter,
        wall,
    )
}

/// Whether the computer records no enemy for the living stack `id` (0x422B20's +0x538 = 0): a war
/// machine but the Ballista, a hypnotized or berserk stack, one blind or turned to stone for
/// more than a round, or a walker that reaches no enemy within the computer's range of 127.
pub fn records_no_enemy(state: &BattleState, ruleset: &BattleRuleset, id: StackId) -> bool {
    aim::turns(state, (ruleset, Field::new(ruleset.battle().field)), id).is_none()
}

/// The computer's strength of stack `id` in the context `ctx` (0x442E60): its worth times its
/// creatures, as a side's strength sums it.
pub fn strength(state: &BattleState, ruleset: &BattleRuleset, ctx: &Context, id: StackId) -> i32 {
    aim::stack_strength(
        state,
        (ruleset, Field::new(ruleset.battle().field)),
        ctx,
        id,
    )
}

/// The standing walls of the siege with their hit points, as wall-shot targets in index order
/// (the original's list of the upper, mid-upper, mid-lower and lower walls, 0x63ABD0); none
/// outside a siege.
pub fn walls(state: &BattleState) -> impl Iterator<Item = (u8, u8)> + '_ {
    aim::walls(state)
}

/// The game's difficulty, 0 (Easy) to 4 (Impossible), which the computer reads (`[game]+0x1F6D8`;
/// `BattleSetup.difficulty`).
pub fn difficulty(state: &BattleState) -> u8 {
    state.difficulty
}

/// Whether `side`'s player is human (the side's +0x54A4; `BattleSetup.human`): the computer's
/// branches marked for Hard and above take such a side as Hard (ADR-0027 point 8).
pub fn human(state: &BattleState, side: BattleSide) -> bool {
    state.human[side.index()]
}

/// Whether the round is in its wait phase (+0x13DE4), where a stack cannot wait again.
pub fn wait_phase(state: &BattleState) -> bool {
    state.wait_phase
}

/// The computer's context for `side` (0x435EC0; BAT-SIE-023): the least attack and defense
/// bonus on the field, the desperate mode and both sides' strengths.
pub fn context(state: &BattleState, ruleset: &BattleRuleset, side: BattleSide) -> Context {
    aim::context(state, ruleset, Field::new(ruleset.battle().field), side)
}

/// The computer's worth of one creature of stack `id` in the context `ctx` (0x442A50), with the
/// shot flag `shot` (kept only for a stack that can shoot now, the Ballista and a turret).
pub fn worth(
    state: &BattleState,
    ruleset: &BattleRuleset,
    ctx: &Context,
    id: StackId,
    shot: bool,
) -> X87 {
    aim::worth(
        state,
        (ruleset, Field::new(ruleset.battle().field)),
        ctx,
        id,
        shot,
    )
}

/// The computer's value of `damage` to stack `id` (0x442FD0): its kills at the worth in `ctx`
/// with the shot flag `shot`, or at 1000 per creature when `desperate`.
pub fn damage_value(
    state: &BattleState,
    ruleset: &BattleRuleset,
    ctx: &Context,
    (id, damage): (StackId, u64),
    (shot, desperate): (bool, bool),
) -> i32 {
    aim::damage_value(
        state,
        (ruleset, Field::new(ruleset.battle().field)),
        ctx,
        (id, damage),
        (shot, desperate),
    )
}

/// The computer's damage of `count` creatures of `attacker` on `target` (0x442780 with the cap
/// flag): a shot with `shot`, else a strike after `jousting` hexes, at least 1, at most the
/// target's hit points.
pub fn damage_estimate(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (attacker, target): (StackId, StackId),
    count: u64,
    shot: bool,
    jousting: u8,
) -> u64 {
    aim::damage_estimate(state, ruleset, (attacker, target), count, shot, jousting)
}

/// The computer's value of `a` striking `t` after `jousting` hexes (0x435C70 without the shot
/// flag; BAT-AI-005): the exchange's damage to `t` less `a`'s losses, valued in `ctx`.
pub fn strike_value(
    state: &BattleState,
    ruleset: &BattleRuleset,
    ctx: &Context,
    (a, t): (StackId, StackId),
    jousting: u8,
) -> i32 {
    let field = Field::new(ruleset.battle().field);
    aim::strike_value(state, (ruleset, field), ctx, (a, t), jousting)
}

/// Whether stack `id` can shoot now for the computer with `blocker` left out of its blockers
/// (0x4428F0 with it): whether a walker beside it would block it.
pub fn shoots_without(
    state: &BattleState,
    ruleset: &BattleRuleset,
    id: StackId,
    blocker: StackId,
) -> bool {
    aim::shoots_without(state, Field::new(ruleset.battle().field), id, blocker)
}

/// The total hit points of stack `id` for the computer (0x443080): 1 for a clone.
pub fn hit_points(state: &BattleState, id: StackId) -> u64 {
    aim::total_hit_points(state, id)
}

/// A map of the computer's pathfinder over the whole field for one stack (0x4B2DA0, 0x4B3400):
/// the positions it reaches and at what cost, the stacks it can attack (marked) and the walk's
/// stops. Cells the search never touched hold cost 0, as the original's zeroed cells.
#[derive(Debug, Clone)]
pub struct Map(Search);

impl Map {
    /// The cost of the position `hex` when the stack reaches it standing there (the cell's bit 0
    /// with none of its top six bits: no hex flown over), or `None`.
    pub fn position(&self, hex: Hex) -> Option<u16> {
        (self.0.reached(hex) && self.0.flown(hex) == 0).then(|| self.0.cell_cost(hex))
    }

    /// The cost the cell of `hex` holds (+0x18): a position's or a mark's, 0 when untouched.
    pub fn cost(&self, hex: Hex) -> u16 {
        let c = self.0.cell_cost(hex);
        if c == u16::MAX { 0 } else { c }
    }

    /// Whether `hex` is a stop of the stack's walk (the pathfinder's +0x68).
    pub fn is_stop(&self, hex: Hex) -> bool {
        self.0.is_stop(hex)
    }

    /// Whether the search reached `hex` (the cell's bit 0): a position or a hex flown over, the
    /// start included.
    pub fn reached(&self, hex: Hex) -> bool {
        self.0.reached(hex)
    }

    /// The cost at which the stack on `hex` can be attacked (the hex record's mark +0x4A and the
    /// cell's cost), or `None` when it is not marked.
    pub fn mark(&self, hex: Hex) -> Option<u16> {
        self.0.mark_of(hex)
    }
}

/// The walker's map of stack `id` for the computer's `side` (0x421680's 0x4B2DA0, 0x4217BE;
/// BAT-AI-005): range 127 (0 for a bound stack or one without speed); the stacks of `side` are not
/// marked; at a town the moat's damage may keep it out of its stops (BAT-SIE-022, the
/// computer's). (The original's range 0 for a stack blind, petrified or paralysed matters only for
/// the predictions' stacks, E23 T4: such a stack has no turn.)
// rule: BAT-AI-005, BAT-SIE-022
pub fn map(state: &BattleState, ruleset: &BattleRuleset, id: StackId, side: BattleSide) -> Map {
    let field = Field::new(ruleset.battle().field);
    let me = state.at(id);
    let query = Query {
        goal: Goal::Map,
        range: AI_RANGE,
        avoid: aim::fears_moat(state, ruleset, me, Some(side)),
        spare: Some(side),
        speed: None,
    };
    Map(Search::run(state, field, (id, me), query))
}

/// The map of an enemy's reach for the walker's hex scores (0x420260's 0x4B2DA0; BAT-AI-007): the
/// computer's pathfinder for stack `id` with its speed + 1 as both the range and the turn's speed
/// (a stop costs up to the next multiple of speed + 1, nothing is marked from a stop), the stacks
/// of its own side not marked; at a town a weak stack its controller's computer plays keeps out of
/// its stops (BAT-SIE-022, with the player's test: the enemy may be a human's).
// rule: BAT-AI-007, BAT-SIE-022
pub fn reach_map(state: &BattleState, ruleset: &BattleRuleset, id: StackId) -> Map {
    let field = Field::new(ruleset.battle().field);
    let me = state.at(id);
    let speed = u16::from(me.speed_now()) + 1;
    let query = Query {
        goal: Goal::Map,
        range: speed,
        avoid: aim::avoids_stops(state, ruleset, me, Some(me.side)),
        spare: Some(me.side),
        speed: Some(speed),
    };
    Map(Search::run(state, field, (id, me), query))
}

/// The damage of the Fire Wall on `hex` to stack `id` (BAT-MAG-028: its value with the bonuses of
/// the hero of the side that cast it, the stack's reductions and protection; the original's
/// 0x5A78E0) and whether its spell hurts the stack at all (the rule's immunities and resistance);
/// `None` without a Fire Wall there.
// rule: BAT-MAG-028
pub fn fire_wall_damage(
    state: &BattleState,
    ruleset: &BattleRuleset,
    hex: Hex,
    id: StackId,
) -> Option<(u64, bool)> {
    state.spell_obstacles.fire_at(hex).and_then(|fire| {
        ruleset.spell(fire.spell).map(|spell| {
            let (_, damage) = magic::obstacle_damage(state, ruleset, (&fire, spell), id);
            (damage, magic::hurts(state, ruleset, &fire, id))
        })
    })
}

/// The hexes of a moat that damages a stack entering it now and its damage (BAT-SIE-005: under
/// SoD rules; the drawbridge's not while the gate is open); `None` without one.
// rule: BAT-SIE-005
pub fn damaging_moat(state: &BattleState) -> Option<(HexSet, u64)> {
    siege::damaging_moat(state)
}

/// The computer's estimate of the best damage `caster`'s hero deals stack `id` with an area spell
/// it can pay (0x41F920's hero part; BAT-AI-007): none while casting is forbidden (an anti-magic
/// garrison, the Orb of Inhibition on either hero), without a hero or such a spell; else the
/// largest `trunc(f32(d) × f)` over the spells of its book aimed at a hex with an area and a damage
/// effect whose cost at its mastery its mana pays, `d` the damage (BAT-MAG-004) and `f` the
/// effect test's factor (BAT-MAG-008), at least 0.
// rule: BAT-AI-007
pub fn area_spell_damage(
    state: &BattleState,
    ruleset: &BattleRuleset,
    caster: BattleSide,
    id: StackId,
) -> i32 {
    aim::best_area_spell(state, ruleset, caster, id)
}

/// The path of the computer's walk (0x41F580's 0x4B3400): the positions from the first step to
/// the end, each with whether it is flown over (no end), and the walk's stops.
#[derive(Debug, Clone)]
pub struct Path {
    steps: [(Hex, bool); MAX_PATH],
    len: usize,
    stops: HexSet,
}

impl Path {
    /// The positions from the first step on, each with whether it is flown over.
    pub fn steps(&self) -> &[(Hex, bool)] {
        self.steps.get(..self.len).unwrap_or(&[])
    }

    /// Whether `hex` is a stop of the walk (the pathfinder's +0x68).
    pub fn is_stop(&self, hex: Hex) -> bool {
        self.stops.contains(hex)
    }
}

/// The computer's path for stack `id` of the computer's `side` to the position `to` (0x41F580's
/// search: 0x4B3400 with `side`, range 127; BAT-AI-006): empty without one or for its own
/// position; at a town the moat's damage may keep it out of its stops (BAT-SIE-022).
// rule: BAT-AI-006
pub fn path(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (id, side): (StackId, BattleSide),
    to: Hex,
) -> Path {
    let field = Field::new(ruleset.battle().field);
    let me = state.at(id);
    let query = Query {
        goal: Goal::Reach(to),
        range: AI_RANGE,
        avoid: aim::fears_moat(state, ruleset, me, Some(side)),
        spare: Some(side),
        speed: None,
    };
    let search = Search::run(state, field, (id, me), query);
    let (hexes, len) = search.path();
    let mut steps = [(Hex(0), false); MAX_PATH];
    for (step, &hex) in steps.iter_mut().zip(hexes.get(..len).unwrap_or(&[])) {
        *step = (hex, search.flown(hex) > 0);
    }
    let stops = (0u8..)
        .take(field.hex_count())
        .map(Hex)
        .filter(|&h| search.is_stop(h))
        .fold(HexSet::new(), |mut set, h| {
            set.insert(h);
            set
        });
    Path { steps, len, stops }
}

/// A walker's map for its side's threat records on its turn (0x421F80's 0x422B20 with flag 1,
/// 0x4B2DA0 at 0x422C51; BAT-AI-009): the computer's pathfinder for stack `id` of the computer's
/// `side` with its speed as the range (0 for a bound stack or one without speed), so a stop
/// marks nothing; the stacks of `side` are not marked; at a town the moat avoided by a weak
/// stack (BAT-SIE-022, the computer's).
// rule: BAT-AI-009, BAT-SIE-022
pub fn turn_map(
    state: &BattleState,
    ruleset: &BattleRuleset,
    id: StackId,
    side: BattleSide,
) -> Map {
    let field = Field::new(ruleset.battle().field);
    let me = state.at(id);
    let query = Query {
        goal: Goal::Map,
        range: u16::from(me.speed_now()),
        avoid: aim::fears_moat(state, ruleset, me, Some(side)),
        spare: Some(side),
        speed: None,
    };
    Map(Search::run(state, field, (id, me), query))
}

/// Whether stack `id` is blind or turned to stone for more than one round (the durations above 1,
/// 0x422BD0 to 0x422BE9, 0x41F1CE to 0x41F1E3): the computer records no enemy for it and puts it
/// late in its order of action.
pub fn kept_out(state: &BattleState, ruleset: &BattleRuleset, id: StackId) -> bool {
    aim::kept_out(state, ruleset, id)
}

/// The computer's value of `side`'s hero in its odds (0x420A80 with 0x526D40 and 0x5275B0;
/// BAT-AI-008, `installation.md` §7.39), `strength` its side's whole strength: its best damage
/// spell valued by the casts its maximum mana allows against that strength, at most four times
/// it; 0 without a hero, a spell book, or while casting is forbidden.
// rule: BAT-AI-008
pub fn hero_value(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    strength: i32,
) -> i32 {
    aim::hero_value(state, ruleset, side, strength)
}

/// The battle's town (ADR-0023), which the computer reads whole: its fort, walls, gate, moat and
/// turrets; `None` outside a town.
pub fn siege(state: &BattleState) -> Option<&SiegeState> {
    state.siege.as_ref()
}

/// A double as the original's `_ftol` gives it to its callers (0x6179D4): truncated toward 0, the
/// low 32 bits kept.
pub fn ftol(x: X87) -> i32 {
    aim::ftol(x)
}

/// Whether stack `id` is bound (+0x2B8; the Dendroids' Bind, BAT-MAG-045): the computer's walker
/// then takes only the enemies next to it and does not walk.
pub fn bound(state: &BattleState, id: StackId) -> bool {
    crate::binds::is_bound(state, id)
}
