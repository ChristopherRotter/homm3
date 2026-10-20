//! The computer's numbers the battle's rules use, public for the original's battle AI
//! (`h3-battle-ai`; ADR-0027 point 6): one implementation, which the rules (the automatic
//! turrets and Ballista, `aim.rs`) and the policy call alike, so a change to a rule the
//! computer estimates moves both. It grows with E23's tasks: what the policy needs from the
//! battle's private state is added here by the battle lane.

use h3_content_battle::BattleRuleset;

use crate::aim;
use crate::combat::can_shoot;
use crate::geometry::Field;
use crate::state::{BattleState, StackId};

/// Whether stack `id` of `state` can shoot now (the original's 0x4428F0 as the battle's rule,
/// BAT-RNG-001: a shooter with a shot to use, not blocked or a war machine, not forgetful): the
/// computer's dispatch sends such a stack to its shooter (BAT-AI-001). `id` must be a stack of
/// `state`.
pub fn shoots(state: &BattleState, ruleset: &BattleRuleset, id: StackId) -> bool {
    can_shoot(state, Field::new(ruleset.battle().field), id)
}

/// The stack the computer's shooter `shooter` of `state` shoots (0x41F060, 0x41EB80): the
/// automatic Ballista's target (BAT-WAR-008) and every computer shooter's (BAT-AI-002), the best
/// by not disabled, then the computer's shot value, ties to the later stack; `None` without a
/// stack it may shoot. `shooter` must be a stack of `state`.
pub fn shot_target(
    state: &BattleState,
    ruleset: &BattleRuleset,
    shooter: StackId,
) -> Option<StackId> {
    aim::ballista_target(state, ruleset, Field::new(ruleset.battle().field), shooter)
}
