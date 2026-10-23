//! The last retaliation's gain (0x41F3B0; `docs/formats/installation.md` §7.38; BAT-AI-009): on
//! Hard and above or for a human side, a walker values an enemy with one retaliation left by what
//! its side's other stacks gain when the walker takes that retaliation. It reads the side's
//! threat records as the walker's turn builds them (0x421F80's 0x422B20 with flag 1, 0x448840;
//! §7.31): each stack of the side, but the walker, the dead, the war machines (not the
//! Ballista), a stack blind or turned to stone for more than one round, a hypnotized or berserk
//! one and one that can shoot now, searches its map with its speed as the range
//! ([`computer::turn_map`]) and records the living enemies but the turrets that act after it in
//! the computer's order of action ([`order::ranks`]) and that it can strike (the marks): their
//! bits as its enemies within a turn, and the one with the fewest turns, then the strictly higher
//! melee value (0x435C70 with the cost as its run-up within its speed), as its nearest with that
//! value. Outside the tactics phase every enemy marked is within a turn; in the phase the map
//! searches the whole field (BAT-AI-023) and the turns count at the stack's speed.
//!
//! The original also records a dead enemy whose place the stack can stand on this turn, by the
//! rank its record kept from an earlier order (stale: the order leaves out the dead); the port
//! records no dead enemy (a nearest of value 0 the original may keep instead of a living one of
//! value 0 or less).
//!
//! The desperate kills follow BAT-QUIRK-25 (slip 4): the original adds the top creature's
//! remaining hit points where its lost ones belong.
//!
//! On a predicted turn (E23 T4b2) the hit points are those left after the damage planned so far
//! ([`Context::left`]), and neither a stack nor an enemy without any left is recorded.

use h3_battle::computer::{self, Context};
use h3_battle::{BattleSide, BattleState, STACK_CAPACITY, Stack, StackId, StackTraits, Status};
use h3_content::ORIGINAL_AI_SLIPS;
use h3_content_battle::BattleRuleset;
use h3_kernel::arith::X87;

use crate::order;

/// A stack's threat record on the walker's turn (+0x538 to +0x544): its nearest enemy, that
/// one's value, cost and turns, and its enemies within a turn by record bit.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Record {
    pub(crate) nearest: Option<StackId>,
    pub(crate) value: i32,
    pub(crate) cost: u16,
    pub(crate) turns: u32,
    pub(crate) mask: u32,
}

/// The threat records of a side by stack id.
pub(crate) type Records = [Record; STACK_CAPACITY];

/// The record bit of stack `id` among its side's (+0xF8: the stacks of its side before it).
fn bit(state: &BattleState, id: StackId) -> u32 {
    let side = state.stacks()[id.index()].side;
    let before = state.stacks().iter().take(id.index());
    let r = before.filter(|s| s.side == side).count();
    1u32 << (r & 31)
}

/// The stack id bits of `side`'s stacks whose record bits (+0xF8) `mask` holds.
pub(crate) fn stack_bits(state: &BattleState, side: BattleSide, mask: u32) -> u64 {
    (0u8..)
        .zip(state.stacks())
        .filter(|(_, s)| s.side == side)
        .zip(0u32..32)
        .filter(|&(_, r)| mask & (1 << r) != 0)
        .fold(0, |bits, ((i, _), _)| bits | 1 << i)
}

/// Whether `s` is the Ballista: a shooting war machine that is neither the Catapult nor a turret.
fn is_ballista(s: &Stack) -> bool {
    s.has(StackTraits::SHOOTER) && !s.is_catapult() && !s.is_turret()
}

/// Whether the side's threat records on the walker's turn search stack `id` (0x422B20 with flag
/// 1): a living stack of `side` but `walker`, with hit points left in a prediction, no war machine
/// but the Ballista, not kept out ([`computer::kept_out`]), hypnotized or berserk, that cannot
/// shoot now.
fn recorded(
    state: &BattleState,
    ruleset: &BattleRuleset,
    ctx: &Context,
    (walker, side): (StackId, BattleSide),
    id: StackId,
) -> bool {
    let o = &state.stacks()[id.index()];
    let status = o.spells.status;
    o.side == side
        && id != walker
        && o.is_alive()
        && !ctx.spent(state, id)
        && (!o.has(StackTraits::WAR_MACHINE) || is_ballista(o))
        && !computer::kept_out(state, ruleset, id)
        && !status.contains(Status::HYPNOTIZED)
        && !status.contains(Status::BERSERK)
        && !computer::shoots(state, ruleset, id)
}

/// The stack the side's threat records on the walker's turn search last (0x422B20 with flag 1:
/// [`records`]' last), none without one. The destination map keeps its search's stops when the
/// hex scores search none (E23 T5b2a, survey slip 30).
// rule: BAT-AI-020
pub(crate) fn last_recorded(
    state: &BattleState,
    ruleset: &BattleRuleset,
    ctx: &Context,
    (walker, side): (StackId, BattleSide),
) -> Option<StackId> {
    (0u8..)
        .map(StackId)
        .take(state.stacks().len())
        .filter(|&id| recorded(state, ruleset, ctx, (walker, side), id))
        .last()
}

/// The threat records of `side`'s stacks on the turn of its walker `walker` in the context `ctx`
/// (0x422B20 with flag 1; BAT-AI-009). The order's first side among equal keys is the active
/// stack's (+0x132B8: the walker's own on its turn, the deciding stack's in a prediction).
// rule: BAT-AI-009
pub(crate) fn records(
    state: &BattleState,
    ruleset: &BattleRuleset,
    ctx: &Context,
    (walker, side): (StackId, BattleSide),
) -> Records {
    let mut out = [Record::default(); STACK_CAPACITY];
    let first = state.active().unwrap_or(walker);
    let ranks = order::ranks(state, ruleset, state.stacks()[first.index()].side);
    for id in (0u8..).map(StackId).take(state.stacks().len()) {
        if !recorded(state, ruleset, ctx, (walker, side), id) {
            continue;
        }
        // The range is the stack's speed and no mark comes from a stop: every enemy marked is
        // within one turn (0x448840's turns 1, a cost within the speed as the run-up), so each
        // sets its bit and the nearest is the one of the strictly higher value; but in the
        // tactics phase, whose map reaches the whole field (BAT-AI-023).
        let map = computer::turn_map(state, ruleset, id, side);
        let speed = u16::from(state.stacks()[id.index()].speed_now());
        let mut record = Record::default();
        for (j, e) in (0u8..).zip(state.stacks()) {
            let enemy = StackId(j);
            if e.side == side
                || !e.is_alive()
                || e.is_turret()
                || ranks[enemy.index()] >= ranks[id.index()]
                || ctx.spent(state, enemy)
            {
                continue;
            }
            let Some(cost) = map.mark(e.hex) else {
                continue;
            };
            let within = cost <= speed;
            let run_up = u8::try_from(cost).unwrap_or(u8::MAX) * u8::from(within);
            let value = computer::strike_value(state, ruleset, ctx, (id, enemy), run_up);
            // 0x448840: `max(1, ⌈cost / speed⌉)`, the bit at one turn.
            let turns = u32::from(cost.div_ceil(speed.max(1))).max(1);
            if turns == 1 {
                record.mask |= bit(state, enemy);
            }
            if record.nearest.is_none()
                || turns < record.turns
                || (turns == record.turns && value > record.value)
            {
                record.nearest = Some(enemy);
                record.value = value;
                record.cost = cost;
                record.turns = turns;
            }
        }
        out[id.index()] = record;
    }
    out
}

/// The low 32 bits of `x` as the original's `int`.
fn int32(x: u64) -> i32 {
    x as u32 as i32
}

/// The creatures of stack `id` by its hit points in the context `ctx` (0x435980's count: its hit
/// points left over its hit points per creature, rounded up).
fn creatures(state: &BattleState, ctx: &Context, id: StackId) -> u64 {
    let s = &state.stacks()[id.index()];
    ctx.left(state, id).div_ceil(u64::from(s.max_hp.max(1)))
}

/// The last retaliation's gain of `walker` of the acting `side` against `enemy` in the context
/// `ctx` with the side's threat records `records` (0x41F3B0; BAT-AI-009): 0 unless the enemy is
/// not turned to stone and has exactly one retaliation left; `r` its hit points left less the
/// walker's strike on it (the damage estimate by the walker's creatures, no shot, capped), 0 when
/// the strike kills it. Each stack of the acting side (+0x132C0, a hypnotized walker's
/// controller's) but the turrets whose enemies within a turn hold the enemy: `D` its
/// strike on it, at most `r`; `g = trunc(D × w / hp) − v` with `w` the enemy's worth (no shot
/// flag), `hp` its hit points per creature and `v` the stack's recorded value; in desperate mode
/// `trunc(k × w) − v` with `k = ⌊(t + D) / hp⌋`, `t` the top creature's remaining hit points `r
/// mod hp` with BAT-QUIRK-25 (slip 4), else its lost ones. The gain: the positive `g` of the
/// stacks whose nearest enemy it is, plus the largest `g` from 0 of the others.
// rule: BAT-AI-009
pub(crate) fn gain(
    state: &BattleState,
    ruleset: &BattleRuleset,
    ctx: &Context,
    records: &Records,
    (walker, side, enemy): (StackId, BattleSide, StackId),
) -> i32 {
    let e = &state.stacks()[enemy.index()];
    if e.spells.status.contains(Status::PETRIFIED) || e.retaliations_left != 1 {
        return 0;
    }
    let hit = |a: StackId| {
        let damage = computer::damage_estimate(
            state,
            ruleset,
            (a, enemy),
            creatures(state, ctx, a),
            false,
            0,
        );
        int32(damage)
    };
    let r = int32(ctx.left(state, enemy)).wrapping_sub(hit(walker));
    if r <= 0 {
        return 0;
    }
    let hp = i32::try_from(e.max_hp).unwrap_or(i32::MAX).max(1);
    let w = computer::worth(state, ruleset, ctx, enemy, false);
    let slips = ruleset.quirks().is_enabled(ORIGINAL_AI_SLIPS);
    let bit = bit(state, enemy);
    let (mut sum, mut best) = (0i32, 0i32);
    for (i, o) in (0u8..).zip(state.stacks()) {
        let id = StackId(i);
        let record = &records[id.index()];
        if o.side != side || o.is_turret() || record.mask & bit == 0 {
            continue;
        }
        let d = hit(id).min(r);
        let kills = if ctx.desperate() {
            let top = if slips { r % hp } else { (hp - r % hp) % hp };
            let k = top.wrapping_add(d) / hp;
            X87::int(u64::try_from(k).unwrap_or(0)).times(w)
        } else {
            let dw = X87::int(u64::try_from(d).unwrap_or(0)).times(w);
            dw.over(X87::int(hp.unsigned_abs().into()))
                .unwrap_or(X87::ZERO)
        };
        let g = computer::ftol(kills).wrapping_sub(record.value);
        if record.nearest == Some(enemy) && g > 0 {
            sum = sum.wrapping_add(g);
        } else if g > best {
            best = g;
        }
    }
    sum.wrapping_add(best)
}
