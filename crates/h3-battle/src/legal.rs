//! Legal commands (ADR-0004 §7): the exact set of commands `apply` accepts for an actor, built
//! from the same reachability and adjacency rules. Consistency with `apply` is property-tested.

use h3_content::{MAX_FIELD_HEXES, Ruleset};
use h3_contracts::BattleSide;
use h3_kernel::FixedVec;

use crate::combat::can_shoot;
use crate::command::Command;
use crate::geometry::{ATTACK_SLOTS, Field, Hex};
use crate::movement::reachable;
use crate::state::{BattleState, STACK_CAPACITY, StackFlags, StackId, StackTraits};
use crate::war;

/// Upper bound on the number of legal commands: Wait, Defend, one move per hex, and per enemy
/// stack either its canonical attack positions or two aimed shots (a commanded First Aid Tent:
/// Wait, Defend and at most one heal per stack).
pub const MAX_LEGAL_COMMANDS: usize = 2 + MAX_FIELD_HEXES + ATTACK_SLOTS * STACK_CAPACITY;

/// Fixed-capacity buffer for [`legal_commands`] (no allocation in the hot path).
pub type LegalCommands = FixedVec<Command, MAX_LEGAL_COMMANDS>;

/// Fills `out` with every command `actor` may issue now (empty if it is not the actor's turn or
/// the battle is over). Order: Wait, Defend, moves by hex, then per enemy stack its shots (a
/// stack that can shoot, BAT-RNG-001: head, then tail for area shots) or its melee attacks by
/// canonical position slot (`Field::attack_positions`; never for war machines, BAT-WAR-001),
/// then for a commanded First Aid Tent its heals by stack id (BAT-WAR-007).
pub fn legal_commands(
    state: &BattleState,
    ruleset: &Ruleset,
    actor: BattleSide,
    out: &mut LegalCommands,
) {
    out.clear();
    let Some((id, mover)) = state.active_stack().filter(|(_, s)| s.side == actor) else {
        return;
    };
    // `out` has room for every command by construction (MAX_LEGAL_COMMANDS), so pushes cannot fail.
    if !mover.flags.contains(StackFlags::WAITED) {
        let _ = out.push(Command::Wait);
    }
    let _ = out.push(Command::Defend);
    let field = Field::new(ruleset.battle().field);
    let reach = reachable(state, field, id, &mover);
    for i in 0..field.hex_count() {
        let hex = Hex(u8::try_from(i).unwrap_or(u8::MAX));
        if reach.can_move_to(hex) {
            let _ = out.push(Command::Move { to: hex });
        }
    }
    let shooting = can_shoot(state, field, id);
    for (tid, target) in (0u8..).zip(state.stacks.iter()) {
        if !target.is_alive() || target.side == actor {
            continue;
        }
        let fp = target.footprint(field);
        if shooting {
            let aims = [
                Some(fp.head),
                fp.tail.filter(|_| mover.has(StackTraits::AREA_SHOT)),
            ];
            for at in aims.into_iter().flatten() {
                let _ = out.push(Command::Shoot {
                    target: StackId(tid),
                    at,
                });
            }
            continue;
        }
        if mover.has(StackTraits::WAR_MACHINE) {
            continue;
        }
        let positions = field.attack_positions(mover.is_two_hex(), mover.side, fp);
        for from in positions.into_iter().flatten() {
            if from == mover.hex || reach.can_move_to(from) {
                let _ = out.push(Command::Attack {
                    target: StackId(tid),
                    from,
                });
            }
        }
    }
    if war::commanded_tent(state, id).is_some() {
        for target in war::patients(state, id).iter().copied() {
            let _ = out.push(Command::Heal { target });
        }
    }
}
