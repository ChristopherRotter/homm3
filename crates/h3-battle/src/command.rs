//! Commands and the single mutation entry point (ADR-0004, ADR-0008).
//!
//! `apply` validates the whole command against the state and the *trusted* actor before touching
//! anything; a rejected command leaves the state unchanged. Commands carry intent only: which hex
//! to move to, which enemy to attack from where. The acting stack, its owner, and every derived
//! value (reach, damage) come from the state.

use alloc::boxed::Box;

use h3_content::{Ruleset, SpellId};
use h3_contracts::{BattleResult, BattleSide};

use crate::combat::{can_shoot, melee, shoot};
use crate::effects;
use crate::error::CommandError;
use crate::event::{Event, EventKind, Sink};
use crate::geometry::{Field, Hex};
use crate::magic::{self, Cast, CastTarget};
use crate::movement::reachable;
use crate::state::{BattleState, Stack, StackFlags, StackId, StackTraits};
use crate::tactics::{self, TacticsPhase};
use crate::turn::{TurnEnd, end_turn};
use crate::war;

/// A player's intent for the active stack. In the tactics phase (BAT-TAC-002) `Move` places the
/// active stack in the zone, `Wait` passes to the next stack and `EndTactics` ends the phase.
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
    /// End the tactics phase; round 1 starts (BAT-TAC-002).
    EndTactics,
    /// Leave the battle: the actor's side retreats (BAT-FLE-001).
    Retreat,
    /// Leave the battle: the actor's side surrenders and pays its price (BAT-FLE-002).
    Surrender,
    /// The actor's hero casts `spell` at `target` during the active stack's turn, which goes on
    /// (BAT-MAG-001).
    Cast {
        /// The spell.
        spell: SpellId,
        /// What it is aimed at.
        target: CastTarget,
    },
}

impl Command {
    /// Whether the command ends the battle by leaving it (Retreat, Surrender).
    pub const fn leaves_battle(self) -> bool {
        matches!(self, Self::Retreat | Self::Surrender)
    }
}

/// A command that passed validation, with everything `apply` needs.
enum Plan<'r> {
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
    Place {
        mover: StackId,
        from: Hex,
        to: Hex,
        phase: TacticsPhase,
    },
    Pass {
        mover: StackId,
        phase: TacticsPhase,
    },
    EndTactics,
    Retreat {
        side: BattleSide,
    },
    Surrender {
        side: BattleSide,
        price: u32,
    },
    /// Boxed: a cast carries the caster's bonuses, far larger than the other plans.
    Cast(Box<Cast<'r>>),
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

fn plan<'r>(
    state: &BattleState,
    ruleset: &'r Ruleset,
    actor: BattleSide,
    command: Command,
) -> Result<Plan<'r>, CommandError> {
    let Some((id, mover)) = state.active_stack() else {
        return Err(CommandError::BattleOver);
    };
    if mover.side != actor {
        return Err(CommandError::NotYourTurn);
    }
    let field = Field::new(ruleset.battle().field);
    if let Some(phase) = state.tactics {
        return plan_tactics(state, ruleset, phase, (id, &mover), command);
    }
    match command {
        Command::EndTactics => Err(CommandError::NoTacticsPhase),
        Command::Retreat => retreat(state, actor),
        Command::Surrender => surrender(state, ruleset, actor),
        Command::Cast { spell, target } => {
            magic::plan_cast(state, ruleset, actor, spell, target).map(|c| Plan::Cast(Box::new(c)))
        }
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

/// Validates a retreat of `side`, whose stack is active or whose tactics phase runs.
// rule: BAT-FLE-001
fn retreat(state: &BattleState, side: BattleSide) -> Result<Plan<'static>, CommandError> {
    if state.may_retreat(side) {
        Ok(Plan::Retreat { side })
    } else {
        Err(CommandError::CannotRetreat)
    }
}

/// Validates a surrender of `side`, whose stack is active or whose tactics phase runs.
// rule: BAT-FLE-002
fn surrender(
    state: &BattleState,
    ruleset: &Ruleset,
    side: BattleSide,
) -> Result<Plan<'static>, CommandError> {
    let Some(price) = state.surrender_price(ruleset, side) else {
        return Err(CommandError::CannotSurrender);
    };
    if state.may_surrender(ruleset, side) {
        Ok(Plan::Surrender { side, price })
    } else {
        Err(CommandError::CannotAffordSurrender)
    }
}

/// Validates `command` for the active stack `mover` in the tactics phase: a placement in the
/// zone, Wait (the game's "next creature", always allowed) or `EndTactics`; nothing else.
// rule: BAT-TAC-001, BAT-TAC-002
fn plan_tactics(
    state: &BattleState,
    ruleset: &Ruleset,
    phase: TacticsPhase,
    (id, mover): (StackId, &Stack),
    command: Command,
) -> Result<Plan<'static>, CommandError> {
    let field = Field::new(ruleset.battle().field);
    match command {
        Command::Move { to } => {
            if !field.is_standable(to) {
                return Err(CommandError::InvalidHex);
            }
            if !phase.contains(field, to) {
                return Err(CommandError::OutsideZone);
            }
            if !tactics::reach(state, field, id, mover).can_move_to(to) {
                return Err(CommandError::Unreachable);
            }
            Ok(Plan::Place {
                mover: id,
                from: mover.hex,
                to,
                phase,
            })
        }
        Command::Wait => Ok(Plan::Pass { mover: id, phase }),
        Command::EndTactics => Ok(Plan::EndTactics),
        Command::Retreat => retreat(state, mover.side),
        Command::Surrender => surrender(state, ruleset, mover.side),
        Command::Attack { .. }
        | Command::Shoot { .. }
        | Command::Defend
        | Command::Heal { .. }
        | Command::Cast { .. } => Err(CommandError::InTacticsPhase),
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
        Plan::Place {
            mover,
            from,
            to,
            phase,
        } => {
            move_stack(state, mover, from, to, sink);
            tactics::placed(state, phase, mover, sink);
        }
        Plan::Pass { mover, phase } => tactics::skip(state, phase, mover, sink),
        Plan::EndTactics => tactics::end(state, ruleset, sink),
        Plan::Retreat { side } => leave(state, BattleResult::Retreat(side), sink),
        Plan::Surrender { side, price } => {
            state.surrender_cost = Some(price);
            leave(state, BattleResult::Surrender(side), sink);
        }
        Plan::Cast(cast) => magic::apply_cast(state, ruleset, *cast, sink),
    }
    // rule: BAT-MAG-010
    effects::remove_dead(state);
    Ok(())
}

/// Ends the battle at once with `result` (a retreat or a surrender, BAT-FLE-001/002), also from
/// the tactics phase.
fn leave<S: Sink>(state: &mut BattleState, result: BattleResult, sink: &mut S) {
    state.tactics = None;
    state.active = None;
    state.result = Some(result);
    sink.emit(Event::public(EventKind::BattleEnded { result }));
}

fn move_stack<S: Sink>(state: &mut BattleState, mover: StackId, from: Hex, to: Hex, sink: &mut S) {
    state.at_mut(mover).hex = to;
    sink.emit(Event::public(EventKind::Moved {
        stack: mover,
        from,
        to,
    }));
}
