//! Commands and the single mutation entry point (ADR-0004, ADR-0008).
//!
//! `apply` validates the whole command against the state and the *trusted* actor before touching
//! anything; a rejected command leaves the state unchanged. Commands carry intent only: which hex
//! to move to, which enemy to attack from where. The acting stack, its owner, and every derived
//! value (reach, damage) come from the state.

use h3_content::Ruleset;
use h3_contracts::BattleSide;

use crate::damage::strike;
use crate::error::CommandError;
use crate::event::{Event, EventKind, Sink};
use crate::geometry::{Field, Hex};
use crate::movement::reachable;
use crate::state::{BattleState, Stack, StackFlags, StackId};
use crate::turn::end_turn;

/// A player's intent for the active stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Command {
    /// Walk to a hex (BAT-MOV-001).
    Move {
        /// Destination.
        to: Hex,
    },
    /// Walk to `from` (or stay, if already there) and strike `target` in melee (BAT-ATK-001).
    Attack {
        /// The enemy stack.
        target: StackId,
        /// The hex to strike from, adjacent to the target.
        from: Hex,
    },
    /// Postpone the turn to the wait phase (BAT-TRN-002).
    #[default]
    Wait,
    /// End the turn with the Defend bonus (BAT-TRN-004).
    Defend,
}

/// A command that passed validation, with everything `apply` needs.
enum Plan {
    Move {
        mover: StackId,
        from: Hex,
        to: Hex,
    },
    Attack {
        mover: StackId,
        from: Hex,
        to: Hex,
        target: StackId,
    },
    Wait {
        mover: StackId,
    },
    Defend {
        mover: StackId,
    },
}

/// Checks `command` for `actor` without changing anything. Shared by `apply` and
/// `legal_commands` consistency tests (docs/06-security.md "Legal-command consistency").
pub fn validate(
    state: &BattleState,
    ruleset: &Ruleset,
    actor: BattleSide,
    command: Command,
) -> Result<(), CommandError> {
    plan(state, ruleset, actor, command).map(|_| ())
}

fn plan(
    state: &BattleState,
    ruleset: &Ruleset,
    actor: BattleSide,
    command: Command,
) -> Result<Plan, CommandError> {
    let Some((id, mover)) = state.active_stack() else {
        return Err(CommandError::BattleOver);
    };
    if mover.side != actor {
        return Err(CommandError::NotYourTurn);
    }
    let field = Field::new(ruleset.battle().field);
    match command {
        Command::Wait => {
            // rule: BAT-TRN-002
            if mover.flags.contains(StackFlags::WAITED) {
                return Err(CommandError::AlreadyWaited);
            }
            Ok(Plan::Wait { mover: id })
        }
        Command::Defend => Ok(Plan::Defend { mover: id }),
        Command::Move { to } => {
            if !field.is_standable(to) {
                return Err(CommandError::InvalidHex);
            }
            if !reachable(state, field, id, &mover).can_move_to(to) {
                return Err(CommandError::Unreachable);
            }
            Ok(Plan::Move {
                mover: id,
                from: mover.hex,
                to,
            })
        }
        Command::Attack { target, from } => {
            let Some(victim) = state.stack(target).filter(|s| s.is_alive()) else {
                return Err(CommandError::NoSuchTarget);
            };
            if victim.side == actor {
                return Err(CommandError::NotAnEnemy);
            }
            let Some(position) = field
                .footprint(from, mover.is_two_hex(), mover.side)
                .filter(|fp| fp.hexes().all(|h| field.is_standable(h)))
            else {
                return Err(CommandError::InvalidHex);
            };
            // rule: BAT-ATK-001
            if !field.touches(position, victim.footprint(field)) {
                return Err(CommandError::NotAdjacent);
            }
            if from != mover.hex && !reachable(state, field, id, &mover).can_move_to(from) {
                return Err(CommandError::Unreachable);
            }
            Ok(Plan::Attack {
                mover: id,
                from: mover.hex,
                to: from,
                target,
            })
        }
    }
}

/// Applies `command` on behalf of `actor` (supplied by the trusted caller, never decoded from the
/// command). On error the state is unchanged. Events go to `sink`.
pub fn apply<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    actor: BattleSide,
    command: Command,
    sink: &mut S,
) -> Result<(), CommandError> {
    let plan = plan(state, ruleset, actor, command)?;
    // Validation is complete: nothing below can fail, and every stack id in the plan is valid.
    match plan {
        Plan::Wait { mover } => {
            sink.emit(Event::public(EventKind::Waited { stack: mover }));
            end_turn(state, mover, true, sink);
        }
        Plan::Defend { mover } => {
            state.at_mut(mover).flags.set(StackFlags::DEFENDING, true);
            sink.emit(Event::public(EventKind::Defended { stack: mover }));
            end_turn(state, mover, false, sink);
        }
        Plan::Move { mover, from, to } => {
            move_stack(state, mover, from, to, sink);
            end_turn(state, mover, false, sink);
        }
        Plan::Attack {
            mover,
            from,
            to,
            target,
        } => {
            if from != to {
                move_stack(state, mover, from, to, sink);
            }
            melee(state, ruleset, mover, target, sink);
            end_turn(state, mover, false, sink);
        }
    }
    Ok(())
}

fn move_stack<S: Sink>(state: &mut BattleState, mover: StackId, from: Hex, to: Hex, sink: &mut S) {
    state.at_mut(mover).hex = to;
    sink.emit(Event::public(EventKind::Moved {
        stack: mover,
        from,
        to,
    }));
}

/// A melee exchange: the strike, then the target's retaliation if it survived and has not
/// retaliated this round (BAT-RET-001), then the end-of-battle check (BAT-END-001).
fn melee<S: Sink>(
    state: &mut BattleState,
    ruleset: &Ruleset,
    attacker: StackId,
    target: StackId,
    sink: &mut S,
) {
    let params = ruleset.battle();
    exchange(state, params, attacker, target, false, sink);
    let t = state.at(target);
    // rule: BAT-RET-001
    if t.is_alive() && !t.flags.contains(StackFlags::RETALIATED) {
        state.at_mut(target).flags.set(StackFlags::RETALIATED, true);
        exchange(state, params, target, attacker, true, sink);
    }
    check_end(state, sink);
}

fn exchange<S: Sink>(
    state: &mut BattleState,
    params: &h3_content::BattleParams,
    attacker: StackId,
    target: StackId,
    retaliation: bool,
    sink: &mut S,
) {
    let striker = *state.at(attacker);
    let (damage, kills) = {
        let BattleState { stacks, rng, .. } = state;
        strike(
            rng,
            &striker,
            &mut stacks.as_mut_slice()[target.index()],
            params,
        )
    };
    sink.emit(Event::public(EventKind::Struck {
        attacker,
        target,
        damage,
        kills,
        retaliation,
    }));
}

/// The battle ends when one side has no living stacks; the other side wins.
// rule: BAT-END-001
fn check_end<S: Sink>(state: &mut BattleState, sink: &mut S) {
    let alive = |side: BattleSide| {
        state
            .stacks
            .iter()
            .any(|s: &Stack| s.side == side && s.is_alive())
    };
    let winner = match (alive(BattleSide::Attacker), alive(BattleSide::Defender)) {
        (true, false) => Some(BattleSide::Attacker),
        (false, true) => Some(BattleSide::Defender),
        _ => None,
    };
    if let Some(w) = winner {
        state.winner = Some(w);
        sink.emit(Event::public(EventKind::BattleEnded { winner: w }));
    }
}
