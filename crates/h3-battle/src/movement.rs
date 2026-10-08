//! Movement (BAT-MOV-001 walking, BAT-MOV-002 flying and teleporting, BAT-MOV-003 a walker's
//! path).
//!
//! A position is the hex of a stack's head; a two-hex stack's tail follows it (BAT-GEO-002). A
//! position is free for a stack when every hex of its footprint there is standable and not
//! blocked by an obstacle (a Force Field among them, BAT-MAG-027) or another living stack (its
//! own current hexes do not block it). A walker's move ends at a position where it enters a pit
//! its controller sees (Quicksand, BAT-MAG-032): it may stop there but not go on.

use h3_content::MAX_FIELD_HEXES;

use crate::binds;
use crate::geometry::{Field, Hex, HexSet};
use crate::obstacles;
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

/// Whether a stack shaped like `mover` fits with its head on `hex`: every hex of the footprint is
/// standable and not in `blocked`.
#[inline]
pub(crate) fn fits(field: Field, mover: &Stack, hex: Hex, blocked: HexSet) -> bool {
    // Head and tail tested directly: the footprint's hex iterator is not always inlined in this
    // hot path (BACKLOG, codegen swings).
    let free = |h: Hex| field.is_standable(h) && !blocked.contains(h);
    field
        .footprint(hex, mover.is_two_hex(), mover.side)
        .is_some_and(|fp| free(fp.head) && fp.tail.is_none_or(free))
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
/// their controller sees, other than those they start on (BAT-MAG-032; the original's stops,
/// `installation.md` §7.16); flyers and teleporters reach every free position whose hex distance
/// is at most `range`, whatever lies in between (BAT-MOV-002). Dead stacks do not block.
// rule: BAT-MOV-001, BAT-MOV-002
pub(crate) fn reachable_within(
    state: &BattleState,
    field: Field,
    id: StackId,
    mover: &Stack,
    range: u8,
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
    // rule: BAT-MAG-032
    let mut pits = obstacles::seen_pits(state, mover.controller());
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

/// Whether `mover` with its head on `hex` covers a hex of `pits`.
fn enters(field: Field, mover: &Stack, hex: Hex, pits: HexSet) -> bool {
    field
        .footprint(hex, mover.is_two_hex(), mover.side)
        .is_some_and(|fp| fp.hexes().any(|h| pits.contains(h)))
}

/// Most positions on a walker's path.
pub(crate) const MAX_PATH: usize = MAX_FIELD_HEXES;

/// The positions a walker `mover` (stack `id`) passes on its way to the position `to`, in order,
/// `to` last and its start left out (BAT-MOV-003): a shortest path between free positions,
/// chosen from the end: each position's predecessor is its first neighbour, in
/// [`crate::Direction::ALL`] order, one step nearer the start, never one where the walk stops (a
/// seen pit, BAT-MAG-032). Empty if `to` is the start or out of its reach this turn (every walk is
/// within the stack's speed, BAT-MOV-001, so the search stops there). Only spell obstacles
/// that act on the hexes a stack enters (Fire Wall, BAT-MAG-028; an unseen pit, BAT-MAG-032) make
/// the path matter. The original walks the path its pathfinder chose (0x4B3400, not read: survey
/// S-11), so on ties it may pass other hexes.
// rule: BAT-MOV-003
pub(crate) fn path(
    state: &BattleState,
    field: Field,
    id: StackId,
    mover: &Stack,
    to: Hex,
) -> ([Hex; MAX_PATH], usize) {
    let reach = reachable(state, field, id, mover);
    let mut out = [Hex(0); MAX_PATH];
    let Some(steps) = reach.steps(to).map(usize::from) else {
        return (out, 0);
    };
    let mut at = to;
    for i in (0..steps).rev() {
        out[i] = at;
        // A reached position other than the start has a neighbour one step nearer.
        at = field
            .neighbors(at)
            .into_iter()
            .flatten()
            .find(|&n| reach.steps(n).is_some_and(|d| usize::from(d) == i) && !reach.stops_at(n))
            .unwrap_or(at);
    }
    (out, steps)
}
