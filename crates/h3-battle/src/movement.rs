//! Walking movement (BAT-MOV-001): breadth-first search over free, standable hexes.

use h3_content::MAX_FIELD_HEXES;

use crate::geometry::{Field, Hex};
use crate::state::{BattleState, Stack};

const UNREACHABLE: u8 = u8::MAX;

/// Hexes a stack can reach this turn, with their step counts.
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

    /// Whether the stack can move to `hex` (reachable and not its own hex).
    pub(crate) fn can_move_to(&self, hex: Hex) -> bool {
        hex != self.start && self.steps(hex).is_some()
    }
}

/// Computes where `mover` can walk: up to `speed` steps, each to a neighbouring standable hex not
/// occupied by a living stack. Dead stacks do not block.
// rule: BAT-MOV-001
pub(crate) fn reachable(state: &BattleState, field: Field, mover: &Stack) -> Reach {
    let mut occupied = [false; MAX_FIELD_HEXES + 1];
    for s in state.stacks.iter().filter(|s| s.is_alive()) {
        // Hexes are `u8`, so every index fits the 256-entry table.
        occupied[s.hex.index()] = true;
    }
    let mut reach = Reach {
        start: mover.hex,
        dist: [UNREACHABLE; MAX_FIELD_HEXES + 1],
    };
    let mut queue = [Hex(0); MAX_FIELD_HEXES + 1];
    let (mut head, mut tail) = (0usize, 1usize);
    queue[0] = mover.hex;
    reach.dist[mover.hex.index()] = 0;
    while head < tail {
        let hex = queue[head];
        head += 1;
        let d = reach.dist[hex.index()];
        if d >= mover.speed {
            continue;
        }
        for next in field.neighbors(hex).into_iter().flatten() {
            let i = next.index();
            if field.is_standable(next) && !occupied[i] && reach.dist[i] == UNREACHABLE {
                reach.dist[i] = d + 1;
                queue[tail] = next;
                tail += 1;
            }
        }
    }
    reach
}
