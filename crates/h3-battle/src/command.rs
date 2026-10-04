//! Commands and the single mutation entry point (ADR-0004, ADR-0008).
//!
//! `apply` validates the whole command against the state and the *trusted* actor before touching
//! anything; a rejected command leaves the state unchanged. Commands carry intent only: which hex
//! to move to, which enemy to attack from where. The acting stack, its owner, and every derived
//! value (reach, damage) come from the state.

use h3_content::Ruleset;
use h3_contracts::BattleSide;

use crate::combat::{can_shoot, melee, shoot};
use crate::error::CommandError;
use crate::event::{Event, EventKind, Sink};
use crate::geometry::{Field, Hex};
use crate::movement::reachable;
use crate::state::{BattleState, StackFlags, StackId, StackTraits};
use crate::turn::{TurnEnd, end_turn};
use crate::war;

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
        /// The head position to strike from; the footprint there must touch the target.
        from: Hex,
    },
    /// Shoot at `target` (BAT-RNG-001).
    Shoot {
        /// The enemy stack.
        target: StackId,
        /// The target's hex aimed at: its head, or (area shots only) its tail (BAT-RNG-004).
        at: Hex,
    },
    /// Postpone the turn to the wait phase (BAT-TRN-002).
    #[default]
    Wait,
    /// End the turn with the Defend bonus (BAT-TRN-004).
    Defend,
    /// Heal `target` with the commanded First Aid Tent (BAT-WAR-007).
    Heal {
        /// The allied stack to heal.
        target: StackId,
    },
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
        charge: u8,
        target: StackId,
    },
    Shoot {
        mover: StackId,
        target: StackId,
        at: Hex,
    },
    Wait {
        mover: StackId,
    },
    Defend {
        mover: StackId,
    },
    Heal {
        mover: StackId,
        target: StackId,
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

/// The living enemy stack `target` of `actor`.
fn enemy(state: &BattleState, actor: BattleSide, target: StackId) -> Result<(), CommandError> {
    let Some(victim) = state.stack(target).filter(|s| s.is_alive()) else {
        return Err(CommandError::NoSuchTarget);
    };
    if victim.side == actor {
        return Err(CommandError::NotAnEnemy);
    }
    Ok(())
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
            enemy(state, actor, target)?;
            // rule: BAT-RNG-001
            if can_shoot(state, field, id) {
                return Err(CommandError::MustShoot);
            }
            // rule: BAT-WAR-001
            if mover.has(StackTraits::WAR_MACHINE) {
                return Err(CommandError::NoMelee);
            }
            let Some(position) = field
                .footprint(from, mover.is_two_hex(), mover.side)
                .filter(|fp| fp.hexes().all(|h| field.is_standable(h)))
            else {
                return Err(CommandError::InvalidHex);
            };
            // rule: BAT-ATK-001
            if !field.touches(position, state.at(target).footprint(field)) {
                return Err(CommandError::NotAdjacent);
            }
            // The hexes travelled count for jousting (BAT-DMG-005).
            let charge = if from == mover.hex {
                0
            } else {
                let Some(steps) = reachable(state, field, id, &mover).steps(from) else {
                    return Err(CommandError::Unreachable);
                };
                steps
            };
            Ok(Plan::Attack {
                mover: id,
                from: mover.hex,
                to: from,
                charge,
                target,
            })
        }
        Command::Shoot { target, at } => {
            enemy(state, actor, target)?;
            // rule: BAT-RNG-001
            if !can_shoot(state, field, id) {
                return Err(CommandError::CannotShoot);
            }
            let fp = state.at(target).footprint(field);
            // rule: BAT-RNG-004
            let aimable = at == fp.head || (mover.has(StackTraits::AREA_SHOT) && fp.covers(at));
            if !aimable {
                return Err(CommandError::NotOnTarget);
            }
            Ok(Plan::Shoot {
                mover: id,
                target,
                at,
            })
        }
        Command::Heal { target } => {
            // rule: BAT-WAR-007
            if war::commanded_tent(state, id).is_none() {
                return Err(CommandError::CannotHeal);
            }
            if state.stack(target).is_none_or(|s| !s.is_alive()) {
                return Err(CommandError::NoSuchTarget);
            }
            if !war::is_patient(state, id, target) {
                return Err(CommandError::NotAPatient);
            }
            Ok(Plan::Heal { mover: id, target })
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
            end_turn(state, ruleset, mover, TurnEnd::Waited, sink);
        }
        Plan::Defend { mover } => {
            state.at_mut(mover).flags.set(StackFlags::DEFENDING, true);
            sink.emit(Event::public(EventKind::Defended { stack: mover }));
            end_turn(state, ruleset, mover, TurnEnd::Defended, sink);
        }
        Plan::Move { mover, from, to } => {
            move_stack(state, mover, from, to, sink);
            end_turn(state, ruleset, mover, TurnEnd::Acted, sink);
        }
        Plan::Attack {
            mover,
            from,
            to,
            charge,
            target,
        } => {
            if from != to {
                move_stack(state, mover, from, to, sink);
            }
            melee(state, ruleset, mover, target, from, charge, sink);
            end_turn(state, ruleset, mover, TurnEnd::Acted, sink);
        }
        Plan::Shoot { mover, target, at } => {
            shoot(state, ruleset, mover, target, at, sink);
            end_turn(state, ruleset, mover, TurnEnd::Acted, sink);
        }
        Plan::Heal { mover, target } => {
            war::heal(state, ruleset, mover, target, sink);
            end_turn(state, ruleset, mover, TurnEnd::Acted, sink);
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
