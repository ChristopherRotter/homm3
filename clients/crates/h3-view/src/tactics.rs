//! The tactics phase on the battlefield (E17 M5 T4c2a; `docs/epics/E17-design.md` §7.23): the
//! hexes the battle shades for a human's stack in it.
//!
//! At a turn's start `battle_controls_state` (0x4782D0) has the hex overlays drawn again
//! (`draw_hex_overlays` 0x493930 with its second argument 1): for a stack a human plays here and
//! that does not play itself, `battle_movement_overlay` (0x4937D0) fills the overlay +0x107 from
//! the computer's reach map (`ai_pathfinder_map` 0x4B2DA0) when the movement shade option
//! (0x6987C4) is on or the tactics phase runs; the harness keeps the option off, so only the
//! phase shades. A hex of the overlay is shaded under everything but the background
//! (`ccellshd.pcx` at its box's corner, 0x44E6A0: every pixel of a non-zero index halved). An
//! action's start clears it (0x478E51).

use alloc::collections::{BTreeSet, VecDeque};

use h3_protocol::wire::{Observation, Side, Stack, Tactics};

use crate::battle_log::Logged;
use crate::mouse::neighbour;
use crate::siege::{BRIDGES, GATE_HEX};

/// Hexes in a row (`hex mod 17` is the column).
const COLUMNS: u8 = 17;
/// Hexes on the field.
const HEXES: u8 = 187;

/// Whether the battle `observation` opens with the tactics phase's message (`battle_start`
/// 0x4630EE to 0x4631E5, GENRLTXT 373): in the phase, when a human plays its side here, before
/// its first turn. The original shows it once a game to each player (its record's +0x88, set
/// when the game starts, 0x4B9EDA; cleared at 0x463203); a battle in the client is a game of its
/// own (*provisional*), so every battle opening with a human's phase shows it.
#[must_use]
pub fn told(observation: &Observation) -> bool {
    observation
        .tactics
        .as_ref()
        .is_some_and(|t| observation.sides[usize::from(t.side == Side::Defender)].human)
}

/// Whether `hex` lies in the tactics phase's zone (`outside_tactics_zone` 0x4763F0, BAT-TAC-001):
/// the phase's side's `columns` standable columns nearest its edge (the attacker's from column
/// 1, the defender's up to column 15; more columns than the field has, as a hostile server may
/// send, the whole field: E17 M5 T8b2, R5c-2).
#[must_use]
pub fn in_zone(tactics: &Tactics, hex: u8) -> bool {
    let column = hex % COLUMNS;
    match tactics.side {
        Side::Attacker => (1..=tactics.columns).contains(&column),
        Side::Defender => {
            ((COLUMNS - 1).saturating_sub(tactics.columns)..COLUMNS - 1).contains(&column)
        }
    }
}

/// The hexes the battle shades for the active stack in the tactics phase of `observation` (none
/// outside it, without a living active stack, or for a stack no human plays: 0x4939B0's +0x54A6,
/// the observation's `human`): the overlay of `battle_movement_overlay` (0x4937D0) over the reach
/// map `ai_pathfinder_map` (0x4B2DA0) builds with its tactics flag. A stack Auto Combat plays
/// (0x493988) is the caller's to leave unshaded.
///
/// The map's search (`battle_pathfinder` 0x4B3400) takes no step limit in the phase: from the
/// stack's position it reaches every position where the stack fits (a walker only through such
/// positions, a flyer over any hex: *provisional*, the original's flyer passes over fewer than its
/// speed of them in a row, 0x4B39E7), and a neighbour of a reached position (directions 0 to 5)
/// that holds a stack of the other side marks that stack's hexes (+0x4A, 0x4B384C), for a two-hex
/// mover also the stack on the hex its front would take there (0x4B38C4). The map keeps
/// a reached position in the zone (+0x4A, 0x4B2E69), a two-hex stack's only with its front hex in
/// the zone too, which it marks (+0x4B, 0x4B2EF4). The overlay shades the stack's own hexes, every
/// position and front hex the map kept, and the hexes of the other side's stacks it marked
/// (0x4938F9): the enemies a stack could reach are shaded wherever they stand.
#[must_use]
pub fn shade(observation: &Observation) -> BTreeSet<u8> {
    let mut shaded = BTreeSet::new();
    let Some(tactics) = &observation.tactics else {
        return shaded;
    };
    let Some(mover) = observation
        .active
        .and_then(|id| observation.stacks.iter().find(|s| s.id == id && s.alive))
    else {
        return shaded;
    };
    if !observation.sides[usize::from(mover.controller == Side::Defender)].human {
        return shaded;
    }
    let field = Field::of(observation, mover);
    shaded.extend(field.hexes(mover, mover.hex));
    let mut seen = BTreeSet::from([mover.hex]);
    let mut queue = VecDeque::from([mover.hex]);
    while let Some(hex) = queue.pop_front() {
        let fits = field.fits(hex);
        if fits && in_zone(tactics, hex) {
            let [_, front] = field.hexes(mover, hex);
            if in_zone(tactics, front) {
                shaded.extend([hex, front]);
            }
        }
        // A hex queued is a position, or a hex a flyer passes over (below).
        for direction in 0..6 {
            let Some(next) = neighbour(hex, direction) else {
                continue;
            };
            // The stack there, and a two-hex mover's front hex's there (0x4B38C4).
            for there in field.hexes(mover, next) {
                if let Some(stack) = field.stack_at(there)
                    && stack.controller != mover.controller
                {
                    shaded.extend(field.hexes(stack, stack.hex));
                }
            }
            // A flyer passes over a hex where it does not fit; a walker goes on only through
            // positions (0x4B39C4).
            if seen.insert(next) && (mover.flies || field.fits(next)) {
                queue.push_back(next);
            }
        }
    }
    shaded
}

/// The hexes shaded at `t` of a recorded battle (E17 M5 T4c2b, design §7.24): its opening
/// `snapshot`, the battle as each later turn's start found it (`turns`: one for each
/// `Logged::Turn` of `log`, `crate::panel::turns`'s) and its timeline's `log`. A human's turn in
/// the tactics phase shades from its start, the battle redrawn and shown at once (0x478375,
/// 0x4783B1), until an action starts, which clears the overlay and shows the battle at once
/// (0x478E51, 0x478E79): the [`shade`] of the last turn started at or before `t` when no action
/// started after it by then (the opening's turn until the first action). The log's actions and
/// turns are in time order (its lines need not be).
#[must_use]
pub fn shade_at(
    snapshot: &Observation,
    turns: &[Observation],
    log: &[(u32, Logged)],
    t: u32,
) -> BTreeSet<u8> {
    // The turn shading: 0 the opening's, k the k-th started since; none while an action plays.
    let (mut turn, mut started) = (Some(0), 0);
    for (at, entry) in log {
        match entry {
            Logged::Action | Logged::Turn(_) if *at > t => break,
            Logged::Action => turn = None,
            Logged::Turn(_) => {
                started += 1;
                turn = Some(started);
            }
            _ => {}
        }
    }
    let battle = match turn {
        Some(0) => Some(snapshot),
        Some(k) => turns.get(k - 1),
        None => None,
    };
    battle.map_or_else(BTreeSet::new, shade)
}

/// The field as the moving stack's search sees it: the hexes obstacles and a town's walls block
/// and the other living stacks.
struct Field<'a> {
    /// The hexes natural obstacles and a siege's walls block.
    blocked: BTreeSet<u8>,
    /// The other living stacks.
    stacks: alloc::vec::Vec<&'a Stack>,
    /// The moving stack.
    mover: &'a Stack,
}

impl<'a> Field<'a> {
    fn of(observation: &'a Observation, mover: &'a Stack) -> Self {
        let mut field = Self {
            blocked: observation.obstacles.iter().copied().collect(),
            stacks: observation
                .stacks
                .iter()
                .filter(|s| s.alive && s.id != mover.id)
                .collect(),
            mover,
        };
        // A siege (the engine runs the phase there too, BAT-TAC-002; E17 M5 T8b2, R5c-6): the
        // standing walls block, and while the gate is closed its hex and the drawbridge's, unless
        // the mover is the defender's, whose step opens it, with no stack or corpse on a bridge
        // (BAT-SIE-002, BAT-SIE-003: the engine's `siege::blocked_for`).
        if let Some(siege) = &observation.siege {
            let on_bridge = observation
                .stacks
                .iter()
                .any(|s| field.hexes(s, s.hex).iter().any(|h| BRIDGES.contains(h)));
            let passes = mover.controller == Side::Defender && !on_bridge;
            let gate = [GATE_HEX, BRIDGES[0]];
            if siege.gate_open || passes {
                let walls = siege.walls.iter().filter(|h| !gate.contains(h));
                field.blocked.extend(walls);
            } else {
                field.blocked.extend(siege.walls.iter().chain(&gate[1..]));
            }
        }
        field
    }

    /// A stack's hexes standing at `hex`: the hex and, two hexes wide, its front hex towards its
    /// side's enemy (the phase runs before any stack turns: each faces its side's way).
    fn hexes(&self, stack: &Stack, hex: u8) -> [u8; 2] {
        let front = match (stack.two_hex, stack.side) {
            (false, _) => hex,
            (true, Side::Attacker) => hex.saturating_add(1),
            (true, Side::Defender) => hex.saturating_sub(1),
        };
        [hex, front]
    }

    /// The living stack other than the mover with a hex on `hex`.
    fn stack_at(&self, hex: u8) -> Option<&'a Stack> {
        self.stacks
            .iter()
            .copied()
            .find(|s| self.hexes(s, s.hex).contains(&hex))
    }

    /// Whether the mover fits with its position on `hex` (`stack_fits_at_hex` 0x446C40): each of
    /// its hexes on a standable column, free of obstacles and of the other stacks.
    fn fits(&self, hex: u8) -> bool {
        self.hexes(self.mover, hex).into_iter().all(|h| {
            h < HEXES
                && (1..COLUMNS - 1).contains(&(h % COLUMNS))
                && !self.blocked.contains(&h)
                && self.stack_at(h).is_none()
        })
    }
}

#[cfg(test)]
mod tests;
