//! Control spells (E04b T4c): a berserk stack's automatic turn (BAT-MAG-025). The stacks the
//! other side controls (Hypnotize, BAT-MAG-026) need no code of their own here: their controller
//! ([`crate::Stack::controller`]) commands them and the rules read it.
//!
//! The original picks a berserk stack's target from its AI pathfinder's map
//! (`docs/formats/installation.md` §7.14); the engine uses its own reachability instead (survey
//! S-11 unconfirmed, OPEN-QUESTIONS Q53).

use h3_content_battle::BattleRuleset;
use h3_kernel::FixedVec;

use crate::binds;
use crate::combat::{can_shoot, melee, shoot};
use crate::event::{Event, EventKind, Sink};
use crate::geometry::{Field, Hex};
use crate::movement::{Reach, reachable_within};
use crate::obstacles;
use crate::state::{BattleState, STACK_CAPACITY, Stack, StackId, StackTraits};

/// A range beyond any battlefield: a walk of every reachable position.
const ANY_RANGE: u8 = u8::MAX - 1;

/// A stack a berserk stack may attack: its id, where to attack it from (its head hex for a
/// shot), the distance.
type Choice = (StackId, Hex, u32);

/// The automatic turn of the berserk stack `id` (BAT-MAG-025; the original's 0x4456D0,
/// `installation.md` §7.14). Its candidates are every other living stack: by hex distance from its
/// head if it can shoot (BAT-RNG-001, BAT-MAG-023), else those with an attack position it can walk
/// or fly to, by the steps to the nearest such position (the first in canonical order on ties).
/// One of the nearest is drawn with equal chances. A shooter shoots at it; a stack that reaches
/// the position this turn walks there and attacks; one that does not walks towards it as far as
/// its speed takes it. A bound stack does not move: its candidates are those next to it
/// (BAT-MAG-045). Without candidates, or unable to move, its turn passes (`TurnPassed`).
/// Returns whether it moved, attacked or shot.
// rule: BAT-MAG-025
pub(crate) fn berserk<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    id: StackId,
    sink: &mut S,
) -> bool {
    let field = Field::new(ruleset.battle().field);
    let me = *state.at(id);
    let shooting = can_shoot(state, field, id);
    // rule: BAT-MAG-045
    let range = if binds::is_bound(state, id) {
        0
    } else {
        ANY_RANGE
    };
    let reach = reachable_within(state, field, id, &me, range);
    let mut nearest: FixedVec<Choice, STACK_CAPACITY> = FixedVec::new();
    let candidates = (0u8..)
        .zip(state.stacks.iter())
        .filter(|&(i, t)| i != id.0 && t.is_alive())
        .filter_map(|(i, t)| {
            let choice = if shooting {
                Some((t.hex, field.distance(me.hex, t.hex)))
            } else {
                attack_position(field, &me, t, &reach)
            };
            choice.map(|(from, d)| (i, from, d))
        });
    for (i, from, d) in candidates {
        if nearest
            .as_slice()
            .first()
            .is_some_and(|&(_, _, best)| d < best)
        {
            nearest.clear();
        }
        if nearest
            .as_slice()
            .first()
            .is_none_or(|&(_, _, best)| d == best)
        {
            // At most one entry per stack: room by construction.
            let _ = nearest.push((StackId(i), from, d));
        }
    }
    let n = u32::try_from(nearest.len()).unwrap_or(u32::MAX);
    let pick = if n > 1 { state.rng.below(n) } else { 0 };
    let Some(&(target, from, d)) = nearest.get(usize::try_from(pick).unwrap_or(0)) else {
        sink.emit(Event::public(EventKind::TurnPassed { stack: id }));
        return false;
    };
    if shooting {
        shoot(state, ruleset, id, target, from, sink);
        obstacles::stand(state, ruleset, id, sink);
        return true;
    }
    let speed = u32::from(me.speed_now());
    if d <= speed {
        // `d <= speed <= 255`.
        let charge = u8::try_from(d).unwrap_or(u8::MAX);
        // As a commanded attack (BAT-MAG-028).
        if from == me.hex {
            melee(state, ruleset, id, target, me.hex, charge, sink);
            obstacles::stand(state, ruleset, id, sink);
        } else if obstacles::travel(state, ruleset, id, (me.hex, from), sink) {
            melee(state, ruleset, id, target, me.hex, charge, sink);
        }
        return true;
    }
    match towards(state, field, (id, &me), &reach, from) {
        Some(to) => {
            obstacles::travel(state, ruleset, id, (me.hex, to), sink);
            true
        }
        None => {
            sink.emit(Event::public(EventKind::TurnPassed { stack: id }));
            false
        }
    }
}

/// The position from which `me` attacks `target` with the fewest steps of `reach` (its own
/// position counts 0), the first in canonical order on ties, and the steps; never one where its
/// walk stops in a pit its side sees, as for a commanded attack (BAT-MAG-032).
fn attack_position(field: Field, me: &Stack, target: &Stack, reach: &Reach) -> Option<(Hex, u32)> {
    field
        .attack_positions(me.is_two_hex(), me.side, target.footprint(field))
        .into_iter()
        .flatten()
        .filter_map(|p| {
            let steps = if p == me.hex {
                Some(0)
            } else {
                reach.steps(p).filter(|_| !reach.stops_at(p))
            };
            steps.map(|s| (p, u32::from(s)))
        })
        .min_by_key(|&(_, s)| s)
}

/// Where `me` (stack `id`) ends a turn walking towards `goal`, beyond its speed: a walker the
/// position on a shortest path to `goal` that its speed reaches, a flyer the position within its
/// speed nearest to `goal`; the lowest hex on ties. `None` if it cannot move.
fn towards(
    state: &BattleState,
    field: Field,
    (id, me): (StackId, &Stack),
    reach: &Reach,
    goal: Hex,
) -> Option<Hex> {
    let speed = me.speed_now();
    let positions = (0u8..)
        .take(field.hex_count())
        .map(Hex)
        .filter(|&h| h != me.hex && reach.steps(h).is_some_and(|s| s <= speed));
    if me.has(StackTraits::FLIES) {
        return positions.min_by_key(|&h| (field.distance(h, goal), h.0));
    }
    // A walk is symmetric: the steps from `goal` back to a position complete the path.
    let back = reachable_within(state, field, id, &Stack { hex: goal, ..*me }, ANY_RANGE);
    let total = reach.steps(goal).map_or(u32::MAX, u32::from);
    positions
        .filter(|&h| {
            reach.steps(h) == Some(speed)
                && back
                    .steps(h)
                    .is_some_and(|b| u32::from(speed) + u32::from(b) == total)
        })
        .min_by_key(|h| h.0)
}
