//! The tactics phase before round 1 (BAT-TAC-001/002; ADR-0013 point 7).
//!
//! The side whose hero has the higher Tactics level places its army stacks in a zone at its edge
//! before the first round; the battle is in round 0 meanwhile. The game hands the side its army
//! stacks one at a time in rounds like regular ones (batch 2a R7): a placement ends the stack's
//! turn, Wait (the game's "next creature") makes it wait or, if it has waited, ends its turn, and
//! when every stack has had its turn the next round of the phase starts. `EndTactics` starts
//! round 1. War machines are not placed.

use h3_content_battle::BattleRuleset;
use h3_contracts::BattleSide;
use h3_kernel::{CanonicalHash, StateHasher};

use crate::event::{Event, EventKind, Sink};
use crate::geometry::{Field, Hex};
use crate::hero::HeroRecord;
use crate::movement::{Reach, reachable_within};
use crate::state::{BattleState, Stack, StackFlags, StackId, StackTraits, side_code};
use crate::turn;

/// A running tactics phase: the side that places its stacks and the width of its zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TacticsPhase {
    /// The side that places its army stacks.
    pub side: BattleSide,
    /// Width of the zone in columns, counted from the side's first standable column
    /// (BAT-TAC-001).
    pub columns: u8,
}

impl TacticsPhase {
    /// Whether `hex` lies in the zone: the side's `columns` standable columns nearest its edge,
    /// as raw columns of every row, so the zone's inner edge zig-zags (BAT-TAC-001).
    // rule: BAT-TAC-001
    pub fn contains(self, field: Field, hex: Hex) -> bool {
        self.contains_column(field, field.coords(hex).0)
    }

    /// Whether the raw column `x` lies in the zone.
    fn contains_column(self, field: Field, x: i32) -> bool {
        let columns = i32::from(self.columns);
        match self.side {
            BattleSide::Attacker => {
                let first = field.first_standable_column();
                x >= first && x < first + columns
            }
            BattleSide::Defender => {
                let last = field.last_standable_column();
                x <= last && x > last - columns
            }
        }
    }

    /// Whether a stack shaped like `stack` lies in the zone at `position` with its whole
    /// footprint (BAT-GEO-002, BAT-TAC-002).
    pub(crate) fn holds(self, field: Field, stack: &Stack, position: Hex) -> bool {
        // A two-hex stack's other hex is its row's next column towards the enemy.
        let (x, _) = field.coords(position);
        let front = match (stack.is_two_hex(), stack.side) {
            (false, _) => x,
            (true, BattleSide::Attacker) => x + 1,
            (true, BattleSide::Defender) => x - 1,
        };
        self.contains_column(field, x) && self.contains_column(field, front)
    }
}

impl CanonicalHash for TacticsPhase {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(side_code(self.side));
        h.write_u8(self.columns);
    }
}

/// The rounds of a tactics phase as the original counts them for its computer (the phase's round
/// counter +0x13D6C and the round's mark of an action +0x14030, which a placement sets;
/// `installation.md` §7.60): the computer's policy ends its phase by them (BAT-AI-023). No rule
/// reads them; they stay as the phase left them after its end.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TacticsRounds {
    /// The rounds of the phase completed (every stack had its turn).
    pub completed: u8,
    /// Whether a stack was placed in the running round.
    pub placed: bool,
    /// Whether a stack was placed in the last completed round.
    pub placed_last: bool,
}

impl CanonicalHash for TacticsRounds {
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u8(self.completed);
        h.write_bool(self.placed);
        h.write_bool(self.placed_last);
    }
}

/// The tactics phase of a battle led by `heroes`, if any: the side whose Tactics level exceeds
/// the enemy's gets a zone as wide as the ruleset's entry for the difference; equal levels
/// cancel, and a side without a hero has level 0.
// rule: BAT-TAC-001
pub(crate) fn phase(
    ruleset: &BattleRuleset,
    heroes: &[Option<HeroRecord>; 2],
) -> Option<TacticsPhase> {
    let level = |side: BattleSide| heroes[side.index()].map_or(0, |h| h.tactics);
    let (attacker, defender) = (level(BattleSide::Attacker), level(BattleSide::Defender));
    let (side, difference) = match attacker.cmp(&defender) {
        core::cmp::Ordering::Greater => (BattleSide::Attacker, attacker - defender),
        core::cmp::Ordering::Less => (BattleSide::Defender, defender - attacker),
        core::cmp::Ordering::Equal => return None,
    };
    // Levels are at most MAX_SKILL_LEVEL (BAT-HER-001), and the table has an entry per level.
    let columns = ruleset
        .battle()
        .tactics_zone_columns
        .get(usize::from(difference) - 1)
        .copied()
        .unwrap_or(0);
    Some(TacticsPhase { side, columns })
}

/// Hand-over order of the side's living army stacks, as in a regular round (BAT-TRN-001/002):
/// first those that have not waited, faster first, then those that waited, slower first; ties to
/// the lower army slot (the id makes the key unique).
fn order_key(id: u8, s: &Stack) -> (bool, u8, u8, u8) {
    let waited = s.flags.contains(StackFlags::WAITED);
    let speed = if waited {
        s.speed_now()
    } else {
        u8::MAX - s.speed_now()
    };
    (waited, speed, s.slot, id)
}

/// The side's next stack that has not had its turn in this round of the phase. War machines are
/// not placed (batch 1b R2).
fn pick(state: &BattleState, phase: TacticsPhase) -> Option<StackId> {
    (0u8..)
        .zip(state.stacks.iter())
        .filter(|(_, s)| {
            s.side == phase.side
                && s.is_alive()
                && !s.has(StackTraits::WAR_MACHINE)
                && !s.flags.contains(StackFlags::ACTED)
        })
        .min_by_key(|&(id, s)| order_key(id, s))
        .map(|(id, _)| StackId(id))
}

/// Hands the next stack to the phase's side; when every stack has had its turn, the next round
/// of the phase starts (the stacks' turn flags are cleared, batch 2a R7c) and the round is
/// counted ([`TacticsRounds`]).
// rule: BAT-TAC-002
fn activate<S: Sink>(state: &mut BattleState, phase: TacticsPhase, sink: &mut S) {
    let id = pick(state, phase).unwrap_or_else(|| {
        for s in state.stacks.as_mut_slice() {
            s.flags.set(StackFlags::ACTED, false);
            s.flags.set(StackFlags::WAITED, false);
        }
        let rounds = &mut state.tactics_rounds;
        rounds.completed = rounds.completed.saturating_add(1);
        rounds.placed_last = rounds.placed;
        rounds.placed = false;
        // The side has an army stack (the setup requires one), so a stack is always found.
        pick(state, phase).unwrap_or(StackId(0))
    });
    state.active = Some(id);
    sink.emit(Event::public(EventKind::TurnStarted { stack: id }));
}

/// Starts the phase in round 0 with the side's first stack.
pub(crate) fn begin<S: Sink>(state: &mut BattleState, phase: TacticsPhase, sink: &mut S) {
    state.round = 0;
    state.tactics = Some(phase);
    sink.emit(Event::public(EventKind::TacticsStarted {
        side: phase.side,
        columns: phase.columns,
    }));
    activate(state, phase, sink);
}

/// Ends `current`'s turn in the phase after a placement, which the round's count records; the
/// next stack follows.
pub(crate) fn placed<S: Sink>(
    state: &mut BattleState,
    phase: TacticsPhase,
    current: StackId,
    sink: &mut S,
) {
    state.at_mut(current).flags.set(StackFlags::ACTED, true);
    state.tactics_rounds.placed = true;
    activate(state, phase, sink);
}

/// Wait in the phase (the game's "next creature"): `current` waits (event `Waited`) or, if it has
/// waited in this round of the phase, its turn ends; the next stack follows (batch 2a R7b).
// rule: BAT-TAC-002
pub(crate) fn skip<S: Sink>(
    state: &mut BattleState,
    phase: TacticsPhase,
    current: StackId,
    sink: &mut S,
) {
    let stack = state.at_mut(current);
    if stack.flags.contains(StackFlags::WAITED) {
        stack.flags.set(StackFlags::ACTED, true);
    } else {
        stack.flags.set(StackFlags::WAITED, true);
        sink.emit(Event::public(EventKind::Waited { stack: current }));
    }
    activate(state, phase, sink);
}

/// Ends the phase: round 1 starts and its first stack is activated as usual.
pub(crate) fn end<S: Sink>(state: &mut BattleState, ruleset: &BattleRuleset, sink: &mut S) {
    state.tactics = None;
    sink.emit(Event::public(EventKind::TacticsEnded));
    turn::start_round(state, ruleset, sink);
    turn::activate_next(state, ruleset, sink);
}

/// The positions stack `id` reaches in the phase: those a walker reaches over the whole field
/// without a step limit (obstacles and stacks block its path, which may leave the zone; user
/// decision 2026-10-04, as VCMI), and every free position for flyers (BAT-MOV-001/002 without the
/// speed limit). A placement also needs its destination in the zone.
// rule: BAT-TAC-002
pub(crate) fn reach(state: &BattleState, field: Field, id: StackId, mover: &Stack) -> Reach {
    reachable_within(state, field, id, mover, u8::MAX)
}
