//! Legal commands (ADR-0004 §7): the exact set of commands `apply` accepts for an actor, built
//! from the same reachability and adjacency rules. Consistency with `apply` is property-tested.

use h3_content::{MAX_FIELD_HEXES, Ruleset};
use h3_contracts::BattleSide;
use h3_kernel::FixedVec;

use crate::command::Command;
use crate::geometry::{Field, Hex};
use crate::movement::reachable;
use crate::state::{BattleState, STACK_CAPACITY, StackFlags, StackId};

/// Upper bound on the number of legal commands: Wait, Defend, one move per hex, and six attack
/// positions per enemy stack.
pub const MAX_LEGAL_COMMANDS: usize = 2 + MAX_FIELD_HEXES + 6 * STACK_CAPACITY;

/// Fixed-capacity buffer for [`legal_commands`] (no allocation in the hot path).
pub type LegalCommands = FixedVec<Command, MAX_LEGAL_COMMANDS>;

/// Fills `out` with every command `actor` may issue now (empty if it is not the actor's turn or
/// the battle is over). Order: Wait, Defend, moves by hex, attacks by target then position.
pub fn legal_commands(
    state: &BattleState,
    ruleset: &Ruleset,
    actor: BattleSide,
    out: &mut LegalCommands,
) {
    out.clear();
    let Some((_, mover)) = state.active_stack().filter(|(_, s)| s.side == actor) else {
        return;
    };
    // `out` has room for every command by construction (MAX_LEGAL_COMMANDS), so pushes cannot fail.
    if !mover.flags.contains(StackFlags::WAITED) {
        let _ = out.push(Command::Wait);
    }
    let _ = out.push(Command::Defend);
    let field = Field::new(ruleset.battle().field);
    let reach = reachable(state, field, &mover);
    for i in 0..field.hex_count() {
        let hex = Hex(u8::try_from(i).unwrap_or(u8::MAX));
        if reach.can_move_to(hex) {
            let _ = out.push(Command::Move { to: hex });
        }
    }
    for (id, target) in (0u8..).zip(state.stacks.iter()) {
        if !target.is_alive() || target.side == actor {
            continue;
        }
        for from in field.neighbors(target.hex).into_iter().flatten() {
            if field.is_standable(from) && (from == mover.hex || reach.can_move_to(from)) {
                let _ = out.push(Command::Attack {
                    target: StackId(id),
                    from,
                });
            }
        }
    }
}
