//! Rounds and turn order (BAT-TRN-001..003), regeneration (BAT-TRN-005) and the morale checks
//! around a turn (BAT-MOR-003/004).
//!
//! A round has three phases. In the normal phase stacks act from fastest to slowest; then stacks
//! that waited during a good-morale turn, then the other stacks that waited, each from slowest to
//! fastest. Speed ties go to the side that did not act last (the attacker at the start of the
//! battle), then to the lower army slot. When every living stack has acted, a new round starts
//! and per-round flags are reset.

use h3_content::Ruleset;
use h3_contracts::BattleSide;

use crate::event::{Event, EventKind, Sink};
use crate::geometry::Field;
use crate::state::{BattleState, StackFlags, StackId, StackTraits};
use crate::{effects, morale, war};

/// The next stack to act this round, or `None` if every living stack has acted.
// rule: BAT-TRN-001, BAT-TRN-002
pub(crate) fn next_active(state: &BattleState) -> Option<StackId> {
    let preferred = state
        .last_side_acted
        .map_or(BattleSide::Attacker, BattleSide::opponent);
    (0u8..)
        .zip(state.stacks.iter())
        // Ammunition supplies never act (BAT-WAR-004).
        .filter(|(_, s)| {
            s.is_alive() && !s.flags.contains(StackFlags::ACTED) && !s.has(StackTraits::AMMO_SUPPLY)
        })
        .min_by_key(|(id, s)| {
            // Phases: normal, waited during a good-morale turn, waited (BAT-MOR-003).
            let phase = match (
                s.flags.contains(StackFlags::WAITED),
                s.flags.contains(StackFlags::HAD_MORALE),
            ) {
                (false, _) => 0,
                (true, true) => 1,
                (true, false) => 2,
            };
            // Smaller keys act earlier. Normal phase: faster first; wait phases: slower first.
            let speed = if phase == 0 {
                u8::MAX - s.speed_now()
            } else {
                s.speed_now()
            };
            // The trailing id makes the key unique, so the order is total (ADR-0003 §9).
            (phase, speed, s.side != preferred, s.slot, *id)
        })
        .map(|(id, _)| StackId(id))
}

/// Starts a new round: resets retaliation, wait, morale and acted flags (the Defend bonus lasts
/// until the stack's next activation, BAT-TRN-004) and the heroes' casts (BAT-MAG-001); the
/// timed effects lose a round or end (BAT-MAG-010).
// rule: BAT-TRN-003, BAT-MAG-001
pub(crate) fn start_round<S: Sink>(state: &mut BattleState, ruleset: &Ruleset, sink: &mut S) {
    state.round = state.round.saturating_add(1);
    for hero in state.heroes.iter_mut().flatten() {
        hero.view.casts = 0;
    }
    for s in state.stacks.as_mut_slice() {
        s.flags.set(StackFlags::ACTED, false);
        s.flags.set(StackFlags::WAITED, false);
        s.flags.set(StackFlags::RETALIATED, false);
        s.flags.set(StackFlags::HAD_MORALE, false);
        s.retaliations_left = s.retaliations;
    }
    sink.emit(Event::public(EventKind::RoundStarted {
        round: state.round,
    }));
    effects::count_down(state, ruleset, sink);
}

/// How a turn ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TurnEnd {
    /// The stack moved, attacked or shot.
    Acted,
    /// The stack waited (BAT-TRN-002).
    Waited,
    /// The stack defended (BAT-TRN-004).
    Defended,
}

/// Ends `mover`'s turn. After moving, attacking or shooting, good morale may give it another turn
/// right away (BAT-MOR-003); otherwise it is marked as waited or acted and the next stack is
/// activated, starting a new round if needed. Once the battle is over no stack is active.
pub(crate) fn end_turn<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    mover: StackId,
    how: TurnEnd,
    sink: &mut S,
) {
    state.last_side_acted = Some(state.at(mover).side);
    // rule: BAT-MOR-003
    if state.result.is_none() && how == TurnEnd::Acted && morale::good_morale(state, ruleset, mover)
    {
        state.at_mut(mover).flags.set(StackFlags::HAD_MORALE, true);
        sink.emit(Event::public(EventKind::GoodMorale { stack: mover }));
        sink.emit(Event::public(EventKind::TurnStarted { stack: mover }));
        return;
    }
    let flag = if how == TurnEnd::Waited {
        StackFlags::WAITED
    } else {
        StackFlags::ACTED
    };
    state.at_mut(mover).flags.set(flag, true);
    if state.result.is_some() {
        state.active = None;
        return;
    }
    activate_next(state, ruleset, sink);
}

/// The next stack to act, starting a new round if every living stack has acted.
fn pick<S: Sink>(state: &mut BattleState, ruleset: &Ruleset, sink: &mut S) -> Option<StackId> {
    next_active(state).or_else(|| {
        start_round(state, ruleset, sink);
        next_active(state)
    })
}

/// Activates the next stack: its Defend bonus ends (BAT-TRN-004), it regenerates (BAT-TRN-005),
/// and bad morale may freeze it (BAT-MOR-004), in which case it loses its turn and the next stack
/// is activated; so is the next after a war machine that acts on its own (BAT-WAR-002/003; a
/// commanded one waits for its side's command, BAT-WAR-006). While the battle is not over some
/// stack is alive, so a stack is always found.
pub(crate) fn activate_next<S: Sink>(state: &mut BattleState, ruleset: &Ruleset, sink: &mut S) {
    let mut next = pick(state, ruleset, sink);
    while let Some(id) = next {
        // rule: BAT-TRN-004
        state.at_mut(id).flags.set(StackFlags::DEFENDING, false);
        sink.emit(Event::public(EventKind::TurnStarted { stack: id }));
        regenerate(state, id, sink);
        // rule: BAT-MOR-004
        if morale::bad_morale(state, ruleset, id) {
            sink.emit(Event::public(EventKind::BadMorale { stack: id }));
        } else if state.at(id).has(StackTraits::WAR_MACHINE)
            && !war::is_commanded(state, Field::new(ruleset.battle().field), id)
        {
            // rule: BAT-WAR-002, BAT-WAR-003
            war::act(state, ruleset, id, sink);
        } else {
            break;
        }
        let done = state.at_mut(id);
        done.flags.set(StackFlags::ACTED, true);
        let side = done.side;
        state.last_side_acted = Some(side);
        // A war machine's shot can end the battle.
        next = if state.result.is_some() {
            None
        } else {
            pick(state, ruleset, sink)
        };
    }
    state.active = next;
}

/// At the start of its turn in the normal phase (not after waiting), a regenerating stack's top
/// creature regains all lost hit points (Q20: at the stack's own turn, as VCMI does); a stack its
/// hero makes regenerate (BAT-HER-006) regains at most the stack's `regeneration`.
// rule: BAT-TRN-005
fn regenerate<S: Sink>(state: &mut BattleState, id: StackId, sink: &mut S) {
    let s = state.at_mut(id);
    let most = if s.has(StackTraits::REGENERATES) {
        u32::MAX
    } else {
        u32::from(s.regeneration)
    };
    if !s.flags.contains(StackFlags::WAITED) {
        let hit_points = (s.max_hp - s.top_hp).min(most);
        if hit_points > 0 {
            s.top_hp += hit_points;
            sink.emit(Event::public(EventKind::Regenerated {
                stack: id,
                hit_points,
            }));
        }
    }
}
