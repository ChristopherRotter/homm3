//! Rounds and turn order (BAT-TRN-001..003), regeneration (BAT-TRN-005) and the morale checks
//! around a turn (BAT-MOR-003/004).
//!
//! A round has three phases. In the normal phase stacks act from fastest to slowest; then stacks
//! that waited during a good-morale turn, then the other stacks that waited, each from slowest to
//! fastest. Speed ties go to the side that did not act last (the attacker at the start of the
//! battle), then to the lower army slot. A blind stack's turn starts and passes (BAT-MAG-024).
//! When every living stack has acted or passed blind, a new round starts and per-round flags are
//! reset.

use h3_content::Ruleset;
use h3_contracts::BattleSide;

use crate::effects::Status;
use crate::event::{Event, EventKind, Sink};
use crate::geometry::Field;
use crate::state::{BattleState, Stack, StackFlags, StackId, StackTraits};
use crate::{control, effects, morale, war};

/// Whether stack `s` is a blind stack whose turn has passed this round (BAT-MAG-024): it is passed
/// over until its blindness ends.
fn passed_blind(s: &Stack) -> bool {
    s.spells.status.contains(Status::BLIND) && s.flags.contains(StackFlags::STARTED)
}

/// The next stack to act this round, or `None` if every living stack has acted or passed blind.
// rule: BAT-TRN-001, BAT-TRN-002
pub(crate) fn next_active(state: &BattleState) -> Option<StackId> {
    let preferred = state
        .last_side_acted
        .map_or(BattleSide::Attacker, BattleSide::opponent);
    (0u8..)
        .zip(state.stacks.iter())
        // Ammunition supplies never act (BAT-WAR-004).
        .filter(|(_, s)| {
            s.is_alive()
                && !s.flags.contains(StackFlags::ACTED)
                && !s.has(StackTraits::AMMO_SUPPLY)
                && !passed_blind(s)
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

/// Starts a new round: resets retaliation (with Counterstrike's extra ones, BAT-MAG-015), wait,
/// morale, started and acted flags (the Defend bonus lasts until the stack's next activation,
/// BAT-TRN-004) and the heroes' casts (BAT-MAG-001); then the timed effects lose a round or end
/// (BAT-MAG-010).
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
        s.flags.set(StackFlags::STARTED, false);
        s.retaliations_left = s.retaliations;
    }
    effects::round_retaliations(state, ruleset);
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

/// Activates the next stack: its Defend bonus ends (BAT-TRN-004); at its first turn of the round
/// it regenerates (BAT-TRN-005) and its effects counted by its own turns count down
/// (BAT-MAG-019); a blind stack then passes without acting (BAT-MAG-024), and bad morale may
/// freeze it (BAT-MOR-004), in which case it loses its turn; either way the next stack is
/// activated, as after a war machine that acts on its own (BAT-WAR-002/003; a commanded one
/// waits for its side's command, BAT-WAR-006) and after a berserk stack's automatic turn
/// (BAT-MAG-025; unless good morale gives it, no longer berserk, another turn). While the battle
/// is not over some stack is alive, so a stack is always found.
pub(crate) fn activate_next<S: Sink>(state: &mut BattleState, ruleset: &Ruleset, sink: &mut S) {
    let mut next = pick(state, ruleset, sink);
    while let Some(id) = next {
        let stack = state.at_mut(id);
        // rule: BAT-TRN-004
        stack.flags.set(StackFlags::DEFENDING, false);
        let first = !stack.flags.contains(StackFlags::STARTED);
        stack.flags.set(StackFlags::STARTED, true);
        sink.emit(Event::public(EventKind::TurnStarted { stack: id }));
        if first {
            regenerate(state, id, sink);
            effects::own_turn(state, ruleset, id, sink);
        }
        // rule: BAT-MAG-024
        if state.at(id).spells.status.contains(Status::BLIND) {
            sink.emit(Event::public(EventKind::TurnPassed { stack: id }));
            next = pick(state, ruleset, sink);
            continue;
        }
        // rule: BAT-MOR-004
        if morale::bad_morale(state, ruleset, id) {
            sink.emit(Event::public(EventKind::BadMorale { stack: id }));
        } else if state.at(id).has(StackTraits::WAR_MACHINE)
            && !war::is_commanded(state, Field::new(ruleset.battle().field), id)
        {
            // rule: BAT-WAR-002, BAT-WAR-003
            war::act(state, ruleset, id, sink);
        } else if !state.at(id).spells.status.contains(Status::BERSERK)
            || berserk_turn(state, ruleset, id, sink)
        {
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

/// The automatic turns of the berserk stack `id` (BAT-MAG-025): it acts ([`control::berserk`]);
/// good morale may give it another turn (BAT-MOR-003), automatic while it is still berserk.
/// Returns whether the stack keeps the turn, no longer berserk, for its side's command.
// rule: BAT-MAG-025
pub(crate) fn berserk_turn<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    id: StackId,
    sink: &mut S,
) -> bool {
    loop {
        let acted = control::berserk(state, ruleset, id, sink);
        state.last_side_acted = Some(state.at(id).side);
        if state.result.is_some() || !acted || !morale::good_morale(state, ruleset, id) {
            return false;
        }
        state.at_mut(id).flags.set(StackFlags::HAD_MORALE, true);
        sink.emit(Event::public(EventKind::GoodMorale { stack: id }));
        sink.emit(Event::public(EventKind::TurnStarted { stack: id }));
        if !state.at(id).spells.status.contains(Status::BERSERK) {
            return true;
        }
    }
}

/// If a cast made the active stack berserk during its own turn, it takes its automatic turn at
/// once (the original's battle loop checks the active stack, `installation.md` §7.14) and, unless
/// it keeps the turn, its turn ends (BAT-MAG-025).
pub(crate) fn berserk_now<S: Sink>(state: &mut BattleState, ruleset: &Ruleset, sink: &mut S) {
    let Some(id) = state
        .active
        .filter(|&id| state.at(id).spells.status.contains(Status::BERSERK))
    else {
        return;
    };
    if berserk_turn(state, ruleset, id, sink) {
        return;
    }
    state.at_mut(id).flags.set(StackFlags::ACTED, true);
    if state.result.is_some() {
        state.active = None;
    } else {
        activate_next(state, ruleset, sink);
    }
}

/// At the start of its first turn of the round (not after waiting), a regenerating stack's top
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
    let hit_points = (s.max_hp - s.top_hp).min(most);
    if hit_points > 0 {
        s.top_hp += hit_points;
        sink.emit(Event::public(EventKind::Regenerated {
            stack: id,
            hit_points,
        }));
    }
}
