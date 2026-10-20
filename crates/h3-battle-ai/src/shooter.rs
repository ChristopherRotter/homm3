//! The computer's shooter (0x41F060; `docs/formats/installation.md` §7.33, §7.35; BAT-AI-002):
//! a stack that can shoot now shoots the stack the computer picks for it (0x41EB80), the
//! automatic Ballista's choice (BAT-WAR-008), and defends without one (action 3).

use h3_battle::{BattleState, Command, Field, StackId, computer};
use h3_content_battle::BattleRuleset;

/// The shot of the computer's shooter `id`: at [`computer::shot_target`]'s stack, aimed at its
/// front hex (the original aims at its position; a single target is hit whichever hex is aimed
/// at), or Defend without a target. The area shooters' value at a hex (0x41EDA0) and the
/// Cyclops's wall shot (0x41EEA0) are E23 T2b's: until then they shoot as the others.
// rule: BAT-AI-002
pub(crate) fn shoot(state: &BattleState, ruleset: &BattleRuleset, id: StackId) -> Command {
    // stub: BAT-STUB-6
    let field = Field::new(ruleset.battle().field);
    computer::shot_target(state, ruleset, id).map_or(Command::Defend, |target| Command::Shoot {
        target,
        at: state.stacks()[target.index()].footprint(field).front,
    })
}
