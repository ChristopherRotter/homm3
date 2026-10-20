//! Movement (BAT-MOV-001 walking, BAT-MOV-002 flying and teleporting, BAT-MOV-003 a walker's
//! path).
//!
//! A position is a one-hex stack's hex or a two-hex stack's rear hex, whose other hex comes along
//! in front of it (BAT-GEO-002). A position is free for a stack when every hex of its footprint
//! there is standable and not blocked by an obstacle (a Force Field among them, BAT-MAG-027) or
//! another living stack (its own current hexes do not block it), and in a battle at a town by the
//! walls or the closed gate (BAT-SIE-002, BAT-SIE-003). A walker's move ends at a position where
//! it enters a pit its controller sees (Quicksand, BAT-MAG-032) or a town's moat (BAT-SIE-004): it
//! may stop there but not go on.

use h3_content_battle::MAX_FIELD_HEXES;

use crate::binds;
use crate::geometry::{Field, Hex, HexSet};
use crate::obstacles;
use crate::pathfinder::{Goal, Query, Search};
use crate::siege;
use crate::state::{BattleState, Stack, StackId, StackTraits};

const UNREACHABLE: u8 = u8::MAX;

/// Positions a stack can reach this turn, with their step counts, and those where a walk stops.
#[derive(Debug, Clone)]
pub(crate) struct Reach {
    start: Hex,
    dist: [u8; MAX_FIELD_HEXES + 1],
    stops: HexSet,
}

impl Reach {
    /// Steps needed to reach `hex`, if reachable within the range.
    pub(crate) fn steps(&self, hex: Hex) -> Option<u8> {
        self.dist
            .get(hex.index())
            .copied()
            .filter(|&d| d != UNREACHABLE)
    }

    /// Whether the stack can move to `hex` (reachable and not its own position).
    pub(crate) fn can_move_to(&self, hex: Hex) -> bool {
        hex != self.start && self.steps(hex).is_some()
    }

    /// Whether a walk to `hex` stops there because the stack enters a pit its controller sees
    /// (BAT-MAG-032): an attack ordered with that move would not happen.
    pub(crate) const fn stops_at(&self, hex: Hex) -> bool {
        self.stops.contains(hex)
    }
}

/// Whether a stack shaped like `mover` fits at the position `hex`: every hex of the footprint is
/// standable and not in `blocked`.
#[inline]
pub(crate) fn fits(field: Field, mover: &Stack, hex: Hex, blocked: HexSet) -> bool {
    // Both hexes tested directly: the footprint's hex iterator is not always inlined in this hot
    // path (BACKLOG, codegen swings).
    let free = |h: Hex| field.is_standable(h) && !blocked.contains(h);
    field
        .footprint(hex, mover.is_two_hex(), mover.side)
        .is_some_and(|fp| free(fp.front) && fp.rear.is_none_or(free))
}

/// Computes where `mover` (stack `id`) can move this turn: within its speed now (BAT-MAG-011),
/// nowhere while it is bound (BAT-MAG-045).
pub(crate) fn reachable(state: &BattleState, field: Field, id: StackId, mover: &Stack) -> Reach {
    reachable_within(state, field, id, mover, range(state, id, mover))
}

/// How far `mover` (stack `id`) moves this turn: its speed now (BAT-MAG-011), 0 while it is
/// bound (BAT-MAG-045).
// rule: BAT-MAG-045
pub(crate) fn range(state: &BattleState, id: StackId, mover: &Stack) -> u8 {
    if binds::is_bound(state, id) {
        0
    } else {
        mover.speed_now()
    }
}

/// Computes where `mover` (stack `id`) can move within `range`. Walkers take up to `range` steps
/// between neighbouring free positions (BAT-MOV-001) and go on from none where they enter a pit
/// their controller sees or a town's moat, other than those they start on (BAT-MAG-032,
/// BAT-SIE-004; the original's stops, `installation.md` §7.16, §7.28); flyers and teleporters
/// reach every free position whose hex distance is at most `range`, whatever lies in between
/// (BAT-MOV-002). Dead stacks do not block.
// rule: BAT-MOV-001, BAT-MOV-002
pub(crate) fn reachable_within(
    state: &BattleState,
    field: Field,
    id: StackId,
    mover: &Stack,
    range: u8,
) -> Reach {
    // rule: BAT-MAG-032, BAT-SIE-004
    reach(state, field, id, mover, range, || {
        let side = mover.controller();
        obstacles::seen_pits(state, side).union(siege::stops(state, side))
    })
}

/// [`reachable_within`] for a walker that stops at the pits `seen` gives (asked only for a
/// walker). One instance per caller, so the commanded stacks' hot path keeps its code.
fn reach(
    state: &BattleState,
    field: Field,
    id: StackId,
    mover: &Stack,
    range: u8,
    seen: impl FnOnce() -> HexSet,
) -> Reach {
    let blocked = state.blocked_for(field, id);
    let mut reach = Reach {
        start: mover.hex,
        dist: [UNREACHABLE; MAX_FIELD_HEXES + 1],
        stops: HexSet::new(),
    };
    reach.dist[mover.hex.index()] = 0;
    if mover.traits.contains(StackTraits::FLIES) {
        for i in 0..field.hex_count() {
            let hex = Hex(u8::try_from(i).unwrap_or(u8::MAX));
            let d = field.distance(mover.hex, hex);
            if d <= u32::from(range) && fits(field, mover, hex, blocked) {
                // `d <= range <= 255`, and the start keeps 0.
                reach.dist[i] = u8::try_from(d).unwrap_or(UNREACHABLE);
            }
        }
        return reach;
    }
    let mut pits = seen();
    if !pits.is_empty() {
        let own = mover
            .footprint(field)
            .hexes()
            .fold(HexSet::new(), |mut own, h| {
                own.insert(h);
                own
            });
        pits = pits.without(own);
    }
    let any_pits = !pits.is_empty();
    let mut queue = [Hex(0); MAX_FIELD_HEXES + 1];
    let (mut head, mut tail) = (0usize, 1usize);
    queue[0] = mover.hex;
    while head < tail {
        let hex = queue[head];
        head += 1;
        let d = reach.dist[hex.index()];
        if d >= range {
            continue;
        }
        for next in field.neighbors(hex).into_iter().flatten() {
            let i = next.index();
            if reach.dist[i] == UNREACHABLE && fits(field, mover, next, blocked) {
                reach.dist[i] = d + 1;
                if any_pits && enters(field, mover, next, pits) {
                    reach.stops.insert(next);
                    continue;
                }
                queue[tail] = next;
                tail += 1;
            }
        }
    }
    reach
}

/// Whether `mover` at the position `hex` covers a hex of `pits`.
fn enters(field: Field, mover: &Stack, hex: Hex, pits: HexSet) -> bool {
    field
        .footprint(hex, mover.is_two_hex(), mover.side)
        .is_some_and(|fp| fp.hexes().any(|h| pits.contains(h)))
}

/// Most positions on a walker's path.
pub(crate) const MAX_PATH: usize = MAX_FIELD_HEXES;

/// The positions a walker `mover` (stack `id`) passes on its way to the position `to`, in order,
/// `to` last and its start left out (BAT-MOV-003): the path of the original's pathfinder within
/// its move (the walk 0x441FA0 asks 0x5239D0, which asks 0x4B3400 with the range its speed:
/// [`crate::pathfinder`], `installation.md` §7.40, §7.41). Among equally short paths between
/// free positions, each position comes from the first position the search took next to it
/// (cheapest, then queued first; each taken position queues its neighbours from top right
/// clockwise). It passes no position where the walk stops (a pit its controller sees,
/// BAT-MAG-032; a town's moat, BAT-SIE-004). Empty if `to` is the start or out of its reach this
/// turn (every walk is within the stack's speed, BAT-MOV-001). Only the hexes a stack enters
/// make the path matter: Fire Wall (BAT-MAG-028), an unseen pit (BAT-MAG-032), a mine
/// (BAT-MAG-033), the moat and the gate (BAT-SIE-003, BAT-SIE-005).
// rule: BAT-MOV-003
pub(crate) fn path(
    state: &BattleState,
    field: Field,
    id: StackId,
    mover: &Stack,
    to: Hex,
) -> ([Hex; MAX_PATH], usize) {
    let query = Query {
        goal: Goal::Reach(to),
        range: u16::from(mover.speed_now()),
        avoid: false,
    };
    Search::run(state, field, (id, mover), query).path()
}
