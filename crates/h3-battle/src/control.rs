//! Control spells (E04b T4c): a berserk stack's automatic turn (BAT-MAG-025). The stacks the
//! other side controls (Hypnotize, BAT-MAG-026) need no code of their own here: their controller
//! ([`crate::Stack::controller`]) commands them and the rules read it.
//!
//! A berserk walker picks its target from the original's pathfinder's marks and walks its path
//! ([`crate::pathfinder`]; `docs/formats/installation.md` §7.14, §7.40, §7.41).

use h3_content::BERSERK_WALKS_TO_POSITION_HEX;
use h3_content_battle::BattleRuleset;
use h3_kernel::FixedVec;

use crate::aim;
use crate::combat::{can_shoot, melee, shoot};
use crate::event::{Event, EventKind, Sink};
use crate::geometry::{Direction, Field, HexSet};
use crate::movement::reachable;
use crate::obstacles;
use crate::pathfinder::{AI_RANGE, Goal, Query, Search};
use crate::state::{BattleState, STACK_CAPACITY, Stack, StackId};

/// The automatic turn of the berserk stack `id` (BAT-MAG-025; the original's 0x4456D0,
/// `installation.md` §7.14). Its candidates are every other living stack (BAT-SIE-019: no
/// turret). A stack that can shoot (BAT-RNG-001, BAT-MAG-023) takes those nearest by hex distance
/// between the front hexes (BAT-GEO-002) and shoots at one of them, drawn with equal chances.
/// Any other takes those the pathfinder marks at the least cost over the whole field
/// ([`walk_and_attack`]). Without candidates its turn passes (`TurnPassed`). Returns whether it
/// moved, attacked or shot.
// rule: BAT-MAG-025
pub(crate) fn berserk<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    id: StackId,
    sink: &mut S,
) -> bool {
    let field = Field::new(ruleset.battle().field);
    let me = *state.at(id);
    // rule: BAT-SIE-019
    let others = (0u8..)
        .zip(state.stacks.iter())
        .filter(|&(i, t)| i != id.0 && t.is_alive() && !t.is_turret())
        .map(|(i, t)| (StackId(i), t));
    if can_shoot(state, field, id) {
        let front = me.footprint(field).front;
        let nearest =
            nearest(others.map(|(t, s)| (t, field.distance(front, s.footprint(field).front))));
        let Some(target) = draw(state, &nearest) else {
            return passes(id, sink);
        };
        let at = state.at(target).footprint(field).front;
        shoot(state, ruleset, id, target, at, sink);
        obstacles::stand(state, ruleset, id, sink);
        return true;
    }
    // At a town the computer keeps a weak walker of a player who is no human out of its stops
    // (BAT-SIE-022: both searches, 0x445582 and 0x42232C, without a side's records).
    // rule: BAT-SIE-022
    let avoid = aim::avoids_stops(state, ruleset, &me, None);
    let query = Query {
        goal: Goal::Map,
        range: AI_RANGE,
        avoid,
        spare: None,
    };
    let map = Search::run(state, field, (id, &me), query);
    let nearest =
        nearest(others.filter_map(|(t, s)| map.mark_of(s.hex).map(|c| (t, u32::from(c)))));
    let Some(target) = draw(state, &nearest) else {
        return passes(id, sink);
    };
    walk_and_attack(state, ruleset, (id, &me), target, avoid, sink)
}

/// The berserk walker `me` (stack `id`) goes for `target` (the original's 0x4222C0, §7.41). The
/// pathfinder searches over the whole field until a step onto the target's position hex is
/// blocked (onto its other hex for a two-hex target in an edge column: a war machine; for a
/// two-hex walker also a step whose beside hex is that hex), and the position it steps from is
/// where it attacks from. With BAT-QUIRK-24 (S-11) only that hex ends the search; without it, any
/// of the target's hexes. Nothing found: the turn passes. Found at its own position: it attacks
/// from there. Else, if the target's mark is within its speed, it attacks from that position
/// where its move reaches it (a seen pit there stops it before the attack, BAT-MAG-032; out of
/// its move, BAT-QUIRK-24's case, it does nothing, as the original's move fails, 0x445950); if
/// not, it walks along the path towards that position, at most its speed in steps, and ends after
/// a stop it enters (§7.38: 0x41F580 without scores, its own search to that position finding the
/// same path); a flyer ends where it does not pass over, and does nothing when that position is
/// one it passes over (the new search reaches it on no step it stands on).
// rule: BAT-MAG-025
fn walk_and_attack<S: Sink>(
    state: &mut BattleState,
    ruleset: &BattleRuleset,
    (id, me): (StackId, &Stack),
    target: StackId,
    avoid: bool,
    sink: &mut S,
) -> bool {
    let field = Field::new(ruleset.battle().field);
    let t = *state.at(target);
    let fp = t.footprint(field);
    let (x, _) = field.coords(t.hex);
    let edge = x == 0 || field.neighbor(t.hex, Direction::Right).is_none();
    let mut ends = HexSet::new();
    ends.insert(if t.is_two_hex() && edge {
        fp.front
    } else {
        t.hex
    });
    // BAT-QUIRK-24 (S-11): the original's test is the aimed hex alone.
    // rule: BAT-MAG-025
    if !ruleset.quirks().is_enabled(BERSERK_WALKS_TO_POSITION_HEX) {
        fp.hexes().for_each(|h| ends.insert(h));
    }
    let query = Query {
        goal: Goal::Attack(ends),
        range: AI_RANGE,
        avoid,
        spare: None,
    };
    let walk = Search::run(state, field, (id, me), query);
    let Some(from) = walk.end() else {
        return passes(id, sink);
    };
    if from == me.hex {
        melee(state, ruleset, id, target, me.hex, 0, sink);
        obstacles::stand(state, ruleset, id, sink);
        return true;
    }
    let speed = me.speed_now();
    let reach = reachable(state, field, id, me);
    if walk.cell_cost(t.hex) <= u16::from(speed) {
        let Some(charge) = reach.steps(from).filter(|_| reach.can_move_to(from)) else {
            return passes(id, sink);
        };
        // As a commanded attack (BAT-MAG-028).
        if obstacles::travel(state, ruleset, id, (me.hex, from), sink) {
            melee(state, ruleset, id, target, me.hex, charge, sink);
        }
        return true;
    }
    // The walk searches anew for that position (0x41F580), which reaches none it passes over:
    // a flyer whose search ended on such a hex does nothing.
    if walk.flown(from) > 0 {
        return passes(id, sink);
    }
    let (path, len) = walk.path();
    // One of its first `speed` positions is no pass: a flyer passes over fewer than its speed
    // in a row (0x4B39E7).
    let mut to = me.hex;
    for &hex in path.iter().take(len.min(usize::from(speed))) {
        if walk.flown(hex) == 0 {
            to = hex;
        }
        if reach.stops_at(hex) {
            break;
        }
    }
    obstacles::travel(state, ruleset, id, (me.hex, to), sink);
    true
}

/// The stacks of `candidates` at the least distance, in their order (the original's list,
/// 0x445490, in record order).
fn nearest(
    candidates: impl Iterator<Item = (StackId, u32)>,
) -> FixedVec<(StackId, u32), STACK_CAPACITY> {
    let mut nearest = FixedVec::new();
    for (t, d) in candidates {
        if nearest
            .as_slice()
            .first()
            .is_some_and(|&(_, best): &(StackId, u32)| d < best)
        {
            nearest.clear();
        }
        if nearest
            .as_slice()
            .first()
            .is_none_or(|&(_, best)| d == best)
        {
            // At most one entry per stack: room by construction.
            let _ = nearest.push((t, d));
        }
    }
    nearest
}

/// One of `nearest`, drawn with equal chances (the original's 0x50B230).
fn draw(
    state: &mut BattleState,
    nearest: &FixedVec<(StackId, u32), STACK_CAPACITY>,
) -> Option<StackId> {
    let n = u32::try_from(nearest.len()).unwrap_or(u32::MAX);
    let pick = if n > 1 { state.rng.below(n) } else { 0 };
    nearest
        .get(usize::try_from(pick).unwrap_or(0))
        .map(|&(t, _)| t)
}

/// The berserk stack `id` does nothing: its turn passes.
fn passes<S: Sink>(id: StackId, sink: &mut S) -> bool {
    sink.emit(Event::public(EventKind::TurnPassed { stack: id }));
    false
}
