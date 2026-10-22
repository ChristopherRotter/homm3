//! The computer's numbers the battle's rules use, public for the original's battle AI
//! (`h3-battle-ai`; ADR-0027 point 6): one implementation, which the rules (the automatic
//! turrets and Ballista, `aim.rs`) and the policy call alike, so a change to a rule the
//! computer estimates moves both. It grows with E23's tasks: what the policy needs from the
//! battle's private state is added here by the battle lane.

use h3_content::{CreatureTypeId, SchoolId, SpellId};
use h3_content_battle::{BattleRuleset, BattleSpell, SpellTarget};
use h3_contracts::BattleSide;
use h3_kernel::arith::X87;

use crate::aim;
use crate::combat::can_shoot;
use crate::effects::Status;
use crate::geometry::{Field, Hex, HexSet};
use crate::magic;
use crate::movement::MAX_PATH;
use crate::pathfinder::{AI_RANGE, Goal, Query, Search};
use crate::siege::{self, SiegeState};
use crate::state::{BattleState, StackId, StackTraits};

pub use crate::aim::{Context, Planned, Shot, Threat};

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
/// which draws). With it the value of the target the choice settled on (0x41EB80's best: the
/// shot value 0x435CB0 of a plain shooter's target, an area shooter's value at its hex; the first
/// pick's for a wall shot; 0 without a target): what a shooting caster's cast must beat in the
/// policy's `strict-fixed` scan (E23, survey slip 25; the user, 2026-10-22). `shooter` must be a
/// stack of `state` that can shoot now ([`shoots`]).
pub fn valued_shot(
    state: &BattleState,
    ruleset: &BattleRuleset,
    shooter: StackId,
    wall: impl FnOnce(&Context, i32) -> Option<u8>,
) -> (Shot, i32) {
    aim::valued_shot(
        state,
        ruleset,
        Field::new(ruleset.battle().field),
        shooter,
        wall,
    )
}

/// What the computer's shooter `shooter` does on a predicted turn for `side` with the damage
/// `planned` so far (0x41F060 with its prediction flag, from the side's run 0x422880; E23 T4b2):
/// [`valued_shot`]'s shot in `side`'s context predicting it ([`Context::predicting`]: the hit points less the
/// planned damage, no target without any left), never a wall shot.
pub fn predicted_shot(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (shooter, side): (StackId, BattleSide),
    planned: &Planned,
) -> Shot {
    let field = Field::new(ruleset.battle().field);
    let ctx = aim::context(state, ruleset, field, side).predicting(planned);
    aim::shot_in(state, (ruleset, field), (&ctx, shooter), |_, _| None)
}

/// The computer's estimate of what Fire Shield on `struck` reflects onto `striker` from a strike
/// of `damage` on `struck` with `left` hit points (0x422440, and inline in the predicted strike
/// 0x4224E0; E23 T4b2): the rule's reflection of the lesser of the two (none for a fire-immune
/// striker or without a shield).
pub fn fire_shield_estimate(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (striker, struck): (StackId, StackId),
    damage: u64,
    left: u64,
) -> u64 {
    aim::reflection_estimate(state, ruleset, (striker, struck), damage, left)
}

/// Whether the computer records no enemy for the living stack `id` (0x422B20's +0x538 = 0): a war
/// machine but the Ballista, a hypnotized or berserk stack, one blind or turned to stone for
/// more than a round, or a walker that reaches no enemy within the computer's range of 127.
pub fn records_no_enemy(state: &BattleState, ruleset: &BattleRuleset, id: StackId) -> bool {
    aim::turns(
        state,
        (ruleset, Field::new(ruleset.battle().field)),
        None,
        id,
    )
    .is_none()
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

/// [`damage_estimate`] without its cap at the target's hit points (0x442780 with the cap flag
/// clear; E23 T4c: the spell context's threat tables, Fortune's value).
pub fn uncapped_damage(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (attacker, target): (StackId, StackId),
    count: u64,
    shot: bool,
    jousting: u8,
) -> u64 {
    aim::uncapped_estimate(state, ruleset, (attacker, target), count, shot, jousting)
}

/// One creature's damage of stack `id` for the computer (0x4426F0; E23 T4c: Bless's value), a
/// double: its maximum and Bless's bonus while blessed, its minimum less Curse's penalty but at
/// least 1 while cursed, else the mean of the two.
pub fn damage_per_creature(state: &BattleState, ruleset: &BattleRuleset, id: StackId) -> X87 {
    aim::per_creature(state, ruleset, state.at(id))
}

/// The least damage one creature of stack `id` deals as the computer reads it (+0xD0; E23 T4c:
/// Curse's value): a Ballista's without its hero's multiplier.
pub fn minimum_damage(state: &BattleState, ruleset: &BattleRuleset, id: StackId) -> u64 {
    aim::damage_range(state, ruleset, state.at(id)).0
}

/// The most damage one creature of stack `id` deals as the computer reads it (+0xD4; E23 T4c:
/// Bless's value): a Ballista's without its hero's multiplier.
pub fn maximum_damage(state: &BattleState, ruleset: &BattleRuleset, id: StackId) -> u64 {
    aim::damage_range(state, ruleset, state.at(id)).1
}

/// The luck of stack `id` as the original's record keeps it (+0x4EC; E23 T4c: Fortune's value):
/// the engine's luck before the roll levels' clamp (BAT-LCK-001), so below 0 too; 0 on a field
/// without luck.
pub fn luck(state: &BattleState, ruleset: &BattleRuleset, id: StackId) -> i32 {
    crate::morale::unclamped_luck(state, ruleset, id)
}

/// The spells the helpful caster `caster`'s cast could draw for stack `s` when `side` commands it
/// (BAT-MAG-050; the original's 0x447EB0 with 0x447A80, which the computer's Master Genie asks
/// for each spell, 0x43C4A0; E23 T4c): its creature's drawn spells useful on that stack, in
/// their creature's order.
pub fn helpful_spells<'a>(
    state: &'a BattleState,
    ruleset: &'a BattleRuleset,
    side: BattleSide,
    (caster, s): (StackId, StackId),
) -> impl Iterator<Item = SpellId> + 'a {
    crate::casters::useful_spells(state, ruleset, side, (caster, s)).map(|(spell, _)| spell)
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

/// What the computer reads of a hero's cast (0x43C800's candidate; E23 T5a, BAT-AI-017).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HeroCast {
    /// The hero's mastery of the spell on the battlefield (0x4E5080; BAT-MAG-002, BAT-TER-005).
    pub mastery: u8,
    /// The spell's cost to the hero (0x4E5240; BAT-MAG-054).
    pub cost: u16,
    /// The hero's spell points (+0x18).
    pub mana: u16,
    /// The hero's spell power (the battle's +0x53D4).
    pub power: u16,
    /// The rounds its timed spells last as the computer counts them: the power and the duration
    /// bonus (0x4E4DB0: the Collar, Ring and Cape of Conjuring, the Ring of the Magi; HER-BAT-014).
    pub rounds: u16,
}

/// What the computer reads of `side`'s hero's cast of `spell` (0x43C800; E23 T5a, BAT-AI-017):
/// its mastery, cost, spell points, power and rounds; all 0 without a hero.
pub fn hero_cast(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    (id, spell): (SpellId, &BattleSpell),
) -> HeroCast {
    state.heroes[side.index()]
        .as_ref()
        .map_or(HeroCast::default(), |hero| {
            let mastery = magic::mastery(state, ruleset, hero, id);
            let power = u16::from(hero.view.power);
            HeroCast {
                mastery,
                cost: crate::mana::cost(spell, mastery, hero.magic.cost_change),
                mana: hero.view.mana,
                power,
                rounds: power.saturating_add(hero.magic.duration_bonus),
            }
        })
}

/// Whether the dispel spell `dispel` ends spell `spell` lasting on a stack, `permanent` when it
/// lasts to the battle's end, as the rule ends it (BAT-MAG-016; E23 T5a: Dispel's estimate
/// without BAT-QUIRK-25).
pub fn dispel_ends(
    ruleset: &BattleRuleset,
    dispel: &BattleSpell,
    (spell, permanent): (SpellId, bool),
) -> bool {
    crate::effects::ends_spell(ruleset, dispel, (spell, permanent))
}

/// Whether a hero in the battle forbids every cast (BAT-MAG-061: an anti-magic garrison, the Orb
/// of Inhibition on either hero; the original's 0x422DA0 tests both before the hero's cast).
pub fn casting_forbidden(state: &BattleState) -> bool {
    magic::casting_forbidden(state)
}

/// Magic Mirror's chance on stack `id` as the computer's one-target evaluation scales a value by
/// it (0x43A76D with 0x4487F0; E23 T5a): while the cast Magic Mirror lasts on the stack its
/// chance, at least the creature's own mirror's (BAT-MAG-022, BAT-MAG-057); 0 without the spell.
pub fn cast_mirror_chance(state: &BattleState, ruleset: &BattleRuleset, id: StackId) -> u8 {
    let s = state.at(id);
    if s.spells.mirror == 0 {
        0
    } else {
        magic::mirror_chance(ruleset, s)
    }
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

/// Whether the creatures may not cast now as the computer's creature cast entry asks (0x421280;
/// BAT-AI-012, E23 T4a): on a battlefield that forbids creature casts (Cursed Ground), at an
/// anti-magic garrison or while a hero forbids casting (the Orb of Inhibition), whatever a
/// creature's `ignores_magic_block` (BAT-MAG-061, BAT-TER-004).
pub fn creature_casts_blocked(state: &BattleState) -> bool {
    magic::creature_casting_forbidden(state)
}

/// The mastery of a creature's cast at the ability's `mastery` (BAT-TER-005): Expert where the
/// battlefield raises the casts that are not a hero's (Magic Plains).
pub fn creature_mastery(state: &BattleState, mastery: u8) -> u8 {
    magic::other_mastery(state, mastery)
}

/// Whether the battlefield raises one of `spell`'s schools to Expert for a hero (BAT-TER-005):
/// the computer's estimate of the Faerie Dragon's spell takes Expert there too (0x43C620; E23
/// T4a, survey slip 7).
pub fn field_raises(state: &BattleState, ruleset: &BattleRuleset, spell: SpellId) -> bool {
    state
        .battlefield
        .and_then(|b| ruleset.battlefield(b))
        .zip(ruleset.spell(spell))
        .is_some_and(|(b, s)| b.raises(&s.schools))
}

/// The creatures the raiser `caster` raises from stack `target` by the computer's count
/// (0x447330; E23 T4a): an Archangel's (a stack raiser's) `min(lost creatures, its hit points
/// div the target's per creature)`, whole creatures only; a Pit Lord's (a corpse raiser's) as its
/// raise (BAT-MAG-052); 0 for a stack that raises nothing.
pub fn raise_count(
    state: &BattleState,
    ruleset: &BattleRuleset,
    caster: StackId,
    target: StackId,
) -> u32 {
    crate::casters::raise_count(state, ruleset, (caster, target))
}

/// The computer's worth of one creature of `creature` that the battle would create for `side`
/// (0x442A50 of the record 0x43D730 sets up: the Pit Lord's Demon, 0x421000; E23 T4a): the
/// side's stack of it as at the battle's start (its hero's bonuses), without the shot flag; 0
/// when the battle creates no such stack for the side.
pub fn created_worth(
    state: &BattleState,
    ruleset: &BattleRuleset,
    ctx: &Context,
    side: BattleSide,
    creature: CreatureTypeId,
) -> X87 {
    aim::created_worth(state, ruleset, ctx, (side, creature))
}

/// The computer's value of a damage spell of the context's side with the value `damage` on stack
/// `id` (0x436E30; E23 T4a): the kills of its damage (with the side's hero's bonuses when
/// `bonuses`) after the stack's reductions, protection and resistance, at most its hit points;
/// a disabled stack's value less its strength's excess over it. 0 for a spell the ruleset lacks.
pub fn spell_damage_value(
    state: &BattleState,
    ruleset: &BattleRuleset,
    ctx: &Context,
    (spell, damage): (SpellId, u64),
    bonuses: bool,
    id: StackId,
) -> i32 {
    let field = Field::new(ruleset.battle().field);
    ruleset.spell(spell).map_or(0, |s| {
        aim::spell_damage_value(
            state,
            (ruleset, field),
            ctx,
            ((spell, s), damage),
            bonuses,
            id,
        )
    })
}

/// The stacks `side`'s chain spell `spell` at `mastery` strikes from stack `first` as the
/// computer estimates it, each with its value from `value` on (0x437190 with 0x5A61F0 not for
/// real; E23 T4a): the rules' chain (BAT-MAG-006) without resistance rolls; nothing for a spell
/// the ruleset lacks or without a chain.
pub fn chain<'a>(
    state: &'a BattleState,
    ruleset: &'a BattleRuleset,
    side: BattleSide,
    (spell, mastery): (SpellId, u8),
    (first, value): (StackId, u64),
) -> impl Iterator<Item = (StackId, u64)> + 'a {
    ruleset.spell(spell).into_iter().flat_map(move |s| {
        magic::ChainEstimate::new(
            (state, ruleset),
            side,
            ((spell, s), mastery),
            (first, value),
        )
    })
}

/// The stacks `side`'s area spell `spell` at `mastery` aimed at `hex` reaches (BAT-MAG-007), by
/// id, as the computer's estimate takes them (0x437040 with 0x5A4920; E23 T4a); none for a spell
/// the ruleset lacks.
pub fn area_stacks<'a>(
    state: &'a BattleState,
    ruleset: &'a BattleRuleset,
    side: BattleSide,
    (spell, mastery): (SpellId, u8),
    hex: Hex,
) -> impl Iterator<Item = StackId> + 'a {
    ruleset
        .spell(spell)
        .into_iter()
        .flat_map(move |s| magic::area_estimate((state, ruleset), side, ((spell, s), mastery), hex))
}

/// The threat record of stack `id` in the context `ctx` as the spell context builds it (0x422B20
/// with flag 0; E23 T4b1): its nearest enemy and the turns to attack it, or none (a stack the
/// computer gives no target, or one that reaches no living enemy within the range of 127).
pub fn threat(
    state: &BattleState,
    ruleset: &BattleRuleset,
    ctx: &Context,
    id: StackId,
) -> Option<Threat> {
    aim::threat(state, ruleset, ctx, id)
}

/// The computer's damage of the base `base` of `striker` on `target`, a shot with `shot`, with
/// `bonus` more attack (0x443E30 on the striker's record or, with a bonus, a copy of it, 0x437800;
/// E23 T4b1): every stage of the strike without its draws and without a run-up, at least 1.
pub fn base_damage(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (striker, target): (StackId, StackId),
    (base, shot): (u64, bool),
    bonus: i16,
) -> u64 {
    aim::base_damage(state, ruleset, (striker, target), (base, shot), (bonus, 0))
}

/// [`base_damage`] on a copy of `target`'s record with `bonus` more defense (0x443E30 on the
/// copy 0x438910 makes, a defense bonus's value; E23 T4c).
pub fn base_damage_on_defense(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (striker, target): (StackId, StackId),
    (base, shot): (u64, bool),
    bonus: i16,
) -> u64 {
    aim::base_damage(state, ruleset, (striker, target), (base, shot), (0, bonus))
}

/// The computer's exchange of `a` with `a_hp` hit points and `t` with `t_hp` (0x4359B0; E23 T4c:
/// Frenzy's value), the context's computer `side`: with `shot`, while `a` can shoot now, `a`'s
/// shot alone, else its strike after `jousting` hexes with Fire Shield's reflection, `t`'s
/// retaliation (none on Easy for a computer side) and a second strike; the hit points both have
/// left.
pub fn exchange(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    ((a, a_hp), (t, t_hp)): ((StackId, u64), (StackId, u64)),
    (shot, jousting): (bool, u8),
) -> (u64, u64) {
    exchange_of(
        ruleset,
        side,
        (((state, a), a_hp), ((state, t), t_hp)),
        (shot, jousting),
    )
}

/// [`exchange`] of the striker's record on the struck stack's, each read from its own state.
fn exchange_of(
    ruleset: &BattleRuleset,
    side: BattleSide,
    ((pa, a_hp), (pt, t_hp)): ((aim::Party<'_>, u64), (aim::Party<'_>, u64)),
    (shot, jousting): (bool, u8),
) -> (u64, u64) {
    let field = Field::new(ruleset.battle().field);
    if shot && can_shoot(pa.0, field, pa.1) {
        let count = a_hp.div_ceil(u64::from(pa.0.at(pa.1).max_hp.max(1)));
        let dealt = aim::damage_between(ruleset, (pa, pt), count, (true, 0));
        return (a_hp, t_hp.saturating_sub(dealt));
    }
    aim::exchange_between(ruleset, side, ((pa, a_hp), (pt, t_hp)), jousting)
}

/// The best kill value a protection of stack `id` keeping `kept` percent takes off the other
/// side's hero's damage spells of `schools` (`None`: every school) up to `level`, for the spell
/// context `ctx` of its side (0x4396E0; E23 T4c): 0 while casting is forbidden, without that
/// hero, or for a clone; with `slip` (BAT-QUIRK-25, survey slip 9) the protected side's hero's
/// caster bonuses and the caster's own army's cost changes.
pub fn spell_protection(
    state: &BattleState,
    ruleset: &BattleRuleset,
    ctx: &Context,
    id: StackId,
    (schools, level, kept): (Option<&[SchoolId]>, u8, u16),
    slip: bool,
) -> i32 {
    aim::spell_protection(state, ruleset, ctx, id, (schools, level, kept), slip)
}

/// The factor of the original's effect test 0x5A8090 for `spell` of `side` at stack `id`
/// (BAT-MAG-008; E23 T4c: the computer's harmful spell values): 0 when it cannot affect the stack,
/// 1 for a positive spell, else the share it is not resisted (with the Unicorns' aura); 0 for a
/// spell the ruleset lacks.
pub fn effect_factor(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    spell: SpellId,
    id: StackId,
) -> X87 {
    ruleset.spell(spell).map_or(X87::ZERO, |s| {
        magic::effect_factor(state, ruleset, side, (spell, s), id)
    })
}

/// Ends `spell` on stack `id` of the computer's copy of the battle (0x444510 on a stack's copy,
/// the removals' estimate 0x439A80; E23 T4c): its effects with a duration end, its permanent ones
/// too with `permanent` (the original's end routine keeps only Disrupting Ray, BAT-MAG-010), and
/// the stack's values are recomputed.
pub fn end_spell(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    (id, spell): (StackId, SpellId),
    permanent: bool,
) {
    let mut sink = crate::NoopSink;
    let ends =
        |e: &crate::effects::ActiveEffect| e.spell == spell && (permanent || e.rounds.is_some());
    if crate::effects::end_matching(state, id, &mut sink, ends) {
        crate::effects::recompute(state, ruleset, id);
    }
}

/// The targets of stack `id` as a berserk stack's automatic turn would take them (0x445490;
/// BAT-MAG-025; E23 T4c: the computer's Berserk value), in record order.
pub fn berserk_candidates(
    state: &BattleState,
    ruleset: &BattleRuleset,
    id: StackId,
) -> impl Iterator<Item = StackId> {
    berserk_search(state, ruleset, id).0
}

/// [`berserk_candidates`] with the map their search left in the computer's pathfinder (0x445582:
/// none for a stack that can shoot, which searches nothing; E23 T4c, survey slip 22).
pub fn berserk_search(
    state: &BattleState,
    ruleset: &BattleRuleset,
    id: StackId,
) -> (impl Iterator<Item = StackId>, Option<Map>) {
    berserk_list(state, ruleset, id, false)
}

/// [`berserk_search`] for a removal's lone copy of stack `id`'s record, which `state` holds in
/// the stack's place (0x445490 on the copy; E23 T4d, survey slip 24): the original tells the
/// battle's records from the copy by pointer, so the stack's own record is a candidate too, at
/// distance 0 or at the search's start (cost 0), the least, in record order. The copy's search is
/// the stack's: the pathfinder tells a hex's stack by side and index (0x446C40).
pub fn berserk_search_of_copy(
    state: &BattleState,
    ruleset: &BattleRuleset,
    id: StackId,
) -> (impl Iterator<Item = StackId>, Option<Map>) {
    berserk_list(state, ruleset, id, true)
}

/// The berserk candidates of `id` and their search, the stack's own record among them with
/// `itself`.
fn berserk_list(
    state: &BattleState,
    ruleset: &BattleRuleset,
    id: StackId,
    itself: bool,
) -> (impl Iterator<Item = StackId>, Option<Map>) {
    let (list, search) = crate::control::searched(state, ruleset, id, itself);
    let candidates = (0..list.len()).filter_map(move |i| list.get(i).map(|&(t, _)| t));
    (candidates, search.map(Map))
}

/// The exchange of a removal's lone copy of stack `id` (its record in `copy`, `hp.0` hit points)
/// with the battle's record of the same stack (in `battle`, `hp.1`), as [`exchange`] for the
/// computer's `side` without a run-up (0x4359B0 on the copy and the record; E23 T4d, survey slip
/// 24: the computer's Berserk and Hypnotize values meet the stack itself among their targets).
/// The hit points both have left.
pub fn exchange_with_itself(
    (battle, copy): (&BattleState, &BattleState),
    ruleset: &BattleRuleset,
    side: BattleSide,
    (id, hp): (StackId, (u64, u64)),
    shot: bool,
) -> (u64, u64) {
    exchange_of(
        ruleset,
        side,
        (((copy, id), hp.0), ((battle, id), hp.1)),
        (shot, 0),
    )
}

/// The lone copy's block (0x4428F0 → 0x469600 → 0x442880 on a removal's copy; E23 T4d, survey
/// slip 24): the original finds the battle's record of a two-hex stack beside its copy, at the
/// copy's other hex, and tells them apart by pointer. So the copy of a two-hex shooter, stack `id`
/// of `copy`, is blocked by the stack itself in `battle` when either is berserk or the copy is
/// hypnotized ([`Stack::is_hostile_to`]); a war machine and a stack that shoots while adjacent
/// shoot anyway, as [`shoots`] (BAT-WAR-001, BAT-HER-006). Such a copy is marked as one that
/// cannot shoot now; an end of a spell on it ([`end_spell`]) recomputes its marks, so the block
/// is asked again after each.
///
/// [`Stack::is_hostile_to`]: crate::Stack::is_hostile_to
pub fn block_by_itself(copy: &mut BattleState, battle: &BattleState, id: StackId) {
    let live = battle.at(id);
    let s = copy.at_mut(id);
    let blocked = s.is_two_hex()
        && s.has(StackTraits::SHOOTER)
        && !s.has(StackTraits::WAR_MACHINE)
        && !s.has(StackTraits::SHOOTS_WHEN_ADJACENT)
        && s.is_hostile_to(live);
    s.spells.status = s.spells.status.with(Status::NO_SHOOTING, blocked);
}

/// The stack whose map the computer's pathfinder holds after the threat records of both sides
/// (0x422B20 for side 0, then side 1, as the spell context 0x4369C0 runs them; E23 T4c, survey
/// slip 22): the last recorded stack that cannot shoot now, the defender's last before the
/// attacker's; none when no record searches. Its map is [`map`] for its side.
pub fn last_searched(state: &BattleState, ruleset: &BattleRuleset) -> Option<StackId> {
    aim::last_searched(state, ruleset)
}

/// [`map`] with the range `range` (0x4B2DA0 with a range of the caller's: the computer's Hypnotize
/// value, its turns times the stack's speed; E23 T4c).
pub fn map_within(
    state: &BattleState,
    ruleset: &BattleRuleset,
    (id, side): (StackId, BattleSide),
    range: u16,
) -> Map {
    let field = Field::new(ruleset.battle().field);
    let me = state.at(id);
    let query = Query {
        goal: Goal::Map,
        range,
        avoid: aim::fears_moat(state, ruleset, me, Some(side)),
        spare: Some(side),
        speed: None,
    };
    Map(Search::run(state, field, (id, me), query))
}

/// The speed of stack `id` with what its effects added but not slowed (the record's +0xC4,
/// which Slow scales through +0x4C8; E23 T4c: the computer's Hypnotize value).
pub fn unslowed_speed(state: &BattleState, id: StackId) -> u16 {
    let added: i32 = state
        .effects
        .of(id)
        .iter()
        .map(|e| i32::from(e.speed))
        .sum();
    u16::try_from((i32::from(state.at(id).speed) + added).clamp(0, 255)).unwrap_or(0)
}

/// The defense of stack `id` for the computer (0x442590 with flag 1): 0 while frenzied, else its
/// defense with the Defend bonus less the moat's penalty (E23 T4c: Frenzy's value without the
/// original's slip).
pub fn defense(state: &BattleState, ruleset: &BattleRuleset, id: StackId) -> i64 {
    aim::defense(state, ruleset, state.at(id))
}

/// Whether `side`'s cast of `spell` can affect stack `id` (the original's 0x5A3C80 as the spell
/// evaluation asks it, 0x43A910; E23 T4b1): a living stack that is no turret, the spell not barred
/// by the battle's level limit, its conditions and immunities met with the heroes' part, Anti-Magic
/// not keeping it off and no clone for a spell that makes its stacks immune; false for a spell the
/// ruleset lacks.
pub fn spell_affects(
    state: &BattleState,
    ruleset: &BattleRuleset,
    side: BattleSide,
    spell: SpellId,
    id: StackId,
) -> bool {
    ruleset
        .spell(spell)
        .is_some_and(|s| magic::affects_stack(state, ruleset, side, (spell, s), id))
}

/// Whether `spell` at `mastery` is aimed at a stack (0x59E060; E23 T4b1): not its mass version
/// (with the ruleset's quirk Q-0011 as the rules take it); false for a spell the ruleset lacks or
/// one not aimed at a stack.
pub fn takes_stack(ruleset: &BattleRuleset, spell: SpellId, mastery: u8) -> bool {
    ruleset
        .spell(spell)
        .is_some_and(|s| magic::target_at(ruleset, s, mastery) == SpellTarget::Stack)
}
