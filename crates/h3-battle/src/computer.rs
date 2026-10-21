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

pub use crate::aim::{Context, Shot};

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

/// What the computer's shooter `shooter` of `state` does (0x41F060; BAT-AI-002, BAT-AI-003): the
/// automatic Ballista's choice for any shooter, an area shooter's target by its value at a hex
/// (0x41EDA0); a wall shooter (the Cyclops) asks `wall` with its side's context and its target's
/// value (0 without one) whether it batters a wall instead, and which (0x41EEA0: the policy's,
/// which draws). `shooter` must be a stack of `state` that can shoot now ([`shoots`]).
pub fn shot(
    state: &BattleState,
    ruleset: &BattleRuleset,
    shooter: StackId,
    wall: impl FnOnce(&Context, i32) -> Option<u8>,
) -> Shot {
    aim::shot(
        state,
        ruleset,
        Field::new(ruleset.battle().field),
        shooter,
        wall,
    )
}

/// Whether the computer records no enemy for the living stack `id` (0x422B20's +0x538 = 0): a war
/// machine but the Ballista, a hypnotized or berserk stack, one blind or turned to stone for
/// more than a round, or a walker that reaches no enemy within the computer's range of 127.
pub fn records_no_enemy(state: &BattleState, ruleset: &BattleRuleset, id: StackId) -> bool {
    aim::turns(state, (ruleset, Field::new(ruleset.battle().field)), id).is_none()
}

/// The computer's strength of stack `id` in the context `ctx` (0x442E60): its worth times its
/// creatures, as a side's strength sums it.
pub fn strength(state: &BattleState, ruleset: &BattleRuleset, ctx: &Context, id: StackId) -> i32 {
    aim::stack_strength(
        state,
        (ruleset, Field::new(ruleset.battle().field)),
        ctx,
        id,
    )
}

/// The standing walls of the siege with their hit points, as wall-shot targets in index order
/// (the original's list of the upper, mid-upper, mid-lower and lower walls, 0x63ABD0); none
/// outside a siege.
pub fn walls(state: &BattleState) -> impl Iterator<Item = (u8, u8)> + '_ {
    aim::walls(state)
}
