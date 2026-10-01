//! Movement (BAT-MOV-001 walking, BAT-MOV-002 flying and teleporting).
//!
//! A position is the hex of a stack's head; a two-hex stack's tail follows it (BAT-GEO-002). A
//! position is free for a stack when every hex of its footprint there is standable and not
//! blocked by an obstacle or another living stack (its own current hexes do not block it).

use h3_content::MAX_FIELD_HEXES;

use crate::geometry::{Field, Hex, HexSet};
use crate::state::{BattleState, Stack, StackId, StackTraits};

const UNREACHABLE: u8 = u8::MAX;

/// Positions a stack can reach this turn, with their step counts.
#[derive(Debug, Clone)]
pub(crate) struct Reach {
    start: Hex,
    dist: [u8; MAX_FIELD_HEXES + 1],
}

impl Reach {
    /// Steps needed to reach `hex`, if reachable within the stack's speed.
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
}

/// Whether a stack shaped like `mover` fits with its head on `hex`: every hex of the footprint is
/// standable and not in `blocked`.
pub(crate) fn fits(field: Field, mover: &Stack, hex: Hex, blocked: HexSet) -> bool {
    field
        .footprint(hex, mover.is_two_hex(), mover.side)
        .is_some_and(|fp| {
            fp.hexes()
                .all(|h| field.is_standable(h) && !blocked.contains(h))
        })
}

/// Computes where `mover` (stack `id`) can move. Walkers take up to `speed` steps between
/// neighbouring free positions (BAT-MOV-001); flyers and teleporters reach every free position
/// whose hex distance is at most `speed`, whatever lies in between (BAT-MOV-002). Dead stacks do
/// not block.
// rule: BAT-MOV-001, BAT-MOV-002
pub(crate) fn reachable(state: &BattleState, field: Field, id: StackId, mover: &Stack) -> Reach {
    let blocked = state.blocked_for(field, id);
    let mut reach = Reach {
        start: mover.hex,
        dist: [UNREACHABLE; MAX_FIELD_HEXES + 1],
    };
    reach.dist[mover.hex.index()] = 0;
    if mover.traits.contains(StackTraits::FLIES) {
        for i in 0..field.hex_count() {
            let hex = Hex(u8::try_from(i).unwrap_or(u8::MAX));
            let d = field.distance(mover.hex, hex);
            if d <= u32::from(mover.speed) && fits(field, mover, hex, blocked) {
                // `d <= speed <= 255`, and the start keeps 0.
                reach.dist[i] = u8::try_from(d).unwrap_or(UNREACHABLE);
            }
        }
        return reach;
    }
    let mut queue = [Hex(0); MAX_FIELD_HEXES + 1];
    let (mut head, mut tail) = (0usize, 1usize);
    queue[0] = mover.hex;
    while head < tail {
        let hex = queue[head];
        head += 1;
        let d = reach.dist[hex.index()];
        if d >= mover.speed {
            continue;
        }
        for next in field.neighbors(hex).into_iter().flatten() {
            let i = next.index();
            if reach.dist[i] == UNREACHABLE && fits(field, mover, next, blocked) {
                reach.dist[i] = d + 1;
                queue[tail] = next;
                tail += 1;
            }
        }
    }
    reach
}
