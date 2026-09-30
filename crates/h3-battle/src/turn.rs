//! Rounds and turn order (BAT-TRN-001..003).
//!
//! A round has two phases. In the normal phase stacks act from fastest to slowest; stacks that
//! waited act afterwards, from slowest to fastest. Speed ties go to the side that did not act
//! last (the attacker at the start of the battle), then to the lower army slot. When every living
//! stack has acted, a new round starts and per-round flags are reset.

use h3_contracts::BattleSide;

use crate::event::{Event, EventKind, Sink};
use crate::state::{BattleState, StackFlags, StackId};

/// The next stack to act this round, or `None` if every living stack has acted.
// rule: BAT-TRN-001, BAT-TRN-002
pub(crate) fn next_active(state: &BattleState) -> Option<StackId> {
    let preferred = state
        .last_side_acted
        .map_or(BattleSide::Attacker, BattleSide::opponent);
    (0u8..)
        .zip(state.stacks.iter())
        .filter(|(_, s)| s.is_alive() && !s.flags.contains(StackFlags::ACTED))
        .min_by_key(|(id, s)| {
            let waited = s.flags.contains(StackFlags::WAITED);
            // Smaller keys act earlier. Normal phase: faster first; wait phase: slower first.
            let speed = if waited { s.speed } else { u8::MAX - s.speed };
            // The trailing id makes the key unique, so the order is total (ADR-0003 §9).
            (waited, speed, s.side != preferred, s.slot, *id)
        })
        .map(|(id, _)| StackId(id))
}

/// Starts a new round: resets retaliation, wait and acted flags (the Defend bonus lasts until the
/// stack's next activation, BAT-TRN-004).
// rule: BAT-TRN-003
pub(crate) fn start_round<S: Sink>(state: &mut BattleState, sink: &mut S) {
    state.round = state.round.saturating_add(1);
    for s in state.stacks.as_mut_slice() {
        s.flags.set(StackFlags::ACTED, false);
        s.flags.set(StackFlags::WAITED, false);
        s.flags.set(StackFlags::RETALIATED, false);
    }
    sink.emit(Event::public(EventKind::RoundStarted {
        round: state.round,
    }));
}

/// Ends `mover`'s turn (`waited` for Wait, otherwise it has acted) and activates the next stack,
/// starting a new round if needed. Once the battle is over no stack is active.
pub(crate) fn end_turn<S: Sink>(
    state: &mut BattleState,
    mover: StackId,
    waited: bool,
    sink: &mut S,
) {
    let stack = state.at_mut(mover);
    let flag = if waited {
        StackFlags::WAITED
    } else {
        StackFlags::ACTED
    };
    stack.flags.set(flag, true);
    let side = stack.side;
    state.last_side_acted = Some(side);
    if state.winner.is_some() {
        state.active = None;
        return;
    }
    let next = next_active(state).or_else(|| {
        start_round(state, sink);
        next_active(state)
    });
    // While the battle is not over some stack is alive, so `next` is always `Some` here.
    state.active = next.inspect(|&id| {
        // rule: BAT-TRN-004
        state.at_mut(id).flags.set(StackFlags::DEFENDING, false);
        sink.emit(Event::public(EventKind::TurnStarted { stack: id }));
    });
}
