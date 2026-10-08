//! Baseline agents: the uniformly random legal agent (also the benchmark workload).

use h3_battle::{
    BattleState, CastTargets, Command, LegalCommands, cast_targets, castable_spells,
    creature_cast_targets, legal_commands, second_targets,
};
use h3_content::Ruleset;
use h3_contracts::{BattleResult, BattleSide};
use h3_kernel::{Rng, StreamId};

use crate::env::{BattleEnv, StepError, StepResult};

/// Result of a rollout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RolloutResult {
    /// Steps played.
    pub steps: u32,
    /// How the battle ended, if it did.
    pub result: Option<BattleResult>,
}

/// Plays both sides with uniformly random legal commands until the episode ends (or
/// `max_steps` if the env has no limit and `max_steps > 0`). Agent randomness comes from the
/// AGENT stream of `agent_seed`, so it never perturbs the battle's own RNG stream.
///
/// Never inlined: `cargo xtask bench-check` counts the instructions inside this function, which
/// thin LTO may otherwise fold into its caller (E04b T3b2).
#[inline(never)]
pub fn random_rollout(env: &mut BattleEnv, agent_seed: u64, max_steps: u32) -> RolloutResult {
    let mut rng = Rng::for_stream(agent_seed, StreamId::AGENT);
    let mut legal = LegalCommands::new();
    let mut steps = 0;
    while let Some(side) = env.to_move() {
        if max_steps != 0 && steps >= max_steps {
            break;
        }
        let command = pick(env, side, &mut rng, &mut legal);
        // Legal commands are always accepted (property-tested in h3-battle).
        let _ = step_command(env, side, command);
        steps += 1;
    }
    RolloutResult {
        steps,
        result: env.state().result(),
    }
}

/// Picks a random legal command for `side` with [`choose`].
pub fn pick(
    env: &BattleEnv,
    side: BattleSide,
    rng: &mut Rng,
    legal: &mut LegalCommands,
) -> Command {
    legal_commands(env.state(), env.ruleset(), side, legal);
    choose(env.state(), env.ruleset(), side, legal, rng)
}

/// The baseline agent's choice for `side` from its legal commands `legal`: uniform among the
/// entries of the action encoding that do not leave the battle (the commands before Retreat and
/// Surrender, which come last in `legal_commands`, the hero's cast if a spell is castable now and
/// the active stack's cast if it has a target; user decision 2026-10-04: the agent fights every
/// battle out, so benchmarks and golden replays play whole battles). A hero's cast then takes a
/// uniform castable spell and a uniform target of it, a stack's cast a uniform target, as an
/// agent samples the factorized action head by head (ADR-0014 point 9). Shared by [`pick`] and
/// [`crate::Replay::record_random`].
pub fn choose(
    state: &BattleState,
    ruleset: &Ruleset,
    side: BattleSide,
    legal: &LegalCommands,
    rng: &mut Rng,
) -> Command {
    let staying = legal.iter().take_while(|c| !c.leaves_battle()).count();
    let spells = castable_spells(state, ruleset, side);
    let mut creature = CastTargets::new();
    creature_cast_targets(state, ruleset, side, &mut creature);
    let hero = usize::from(!spells.is_empty());
    let entries = staying + hero + usize::from(!creature.is_empty());
    let i = rng.below(u32::try_from(entries).unwrap_or(u32::MAX)) as usize;
    if i < staying {
        return legal.get(i).copied().unwrap_or(Command::Defend);
    }
    if i >= staying + hero {
        let n = u32::try_from(creature.len()).unwrap_or(u32::MAX);
        let target = creature
            .get(rng.below(n) as usize)
            .copied()
            .unwrap_or_default();
        return Command::CreatureCast { target };
    }
    let count = u32::try_from(spells.iter().count()).unwrap_or(u32::MAX);
    let nth = rng.below(count) as usize;
    // A castable spell has at least one target (`castable_spells`), and each target a second
    // (`cast_targets`): none for every spell but Teleport, whose destination is drawn too.
    spells.iter().nth(nth).map_or(Command::Defend, |spell| {
        let mut targets = CastTargets::new();
        cast_targets(state, ruleset, side, spell, &mut targets);
        let n = u32::try_from(targets.len()).unwrap_or(u32::MAX);
        let target = targets
            .get(rng.below(n) as usize)
            .copied()
            .unwrap_or_default();
        second_targets(state, ruleset, side, (spell, target), &mut targets);
        let n = u32::try_from(targets.len()).unwrap_or(u32::MAX);
        let k = if n > 1 { rng.below(n) as usize } else { 0 };
        let second = targets.get(k).copied().unwrap_or_default();
        Command::Cast {
            spell,
            target,
            second,
        }
    })
}

/// Steps the environment with a domain command (encoded through the action encoding).
pub fn step_command(
    env: &mut BattleEnv,
    side: BattleSide,
    command: Command,
) -> Result<StepResult, StepError> {
    let action = crate::action::encode(env.state(), env.ruleset(), command)
        .ok_or(StepError::InvalidAction)?;
    env.step(side, action)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn random_rollouts_are_reproducible() {
        let (r, setup) = h3_pack::synthetic(h3_pack::SYNTHETIC_SKIRMISH).unwrap();
        let (r, setup) = (Arc::new(r), Arc::new(setup));
        let mut a = BattleEnv::new(r.clone(), setup.clone(), 1, 0).unwrap();
        let mut b = BattleEnv::new(r.clone(), setup.clone(), 1, 0).unwrap();
        let ra = random_rollout(&mut a, 9, 300);
        let rb = random_rollout(&mut b, 9, 300);
        assert_eq!(ra, rb);
        assert_eq!(a.state_hash(), b.state_hash());
        assert!(ra.steps <= 300);
        let mut c = BattleEnv::new(r, setup, 1, 50).unwrap();
        let rc = random_rollout(&mut c, 9, 0);
        assert_eq!(rc.steps, 50);
        assert!(c.is_done());
    }

    #[test]
    fn the_random_agent_fights_battles_out() {
        // Both heroes of the full scenario may retreat and surrender on every turn (BAT-FLE-001,
        // BAT-FLE-002); the agent never does.
        let (r, setup) = h3_pack::synthetic(h3_pack::SYNTHETIC_FULL).unwrap();
        let (r, setup) = (Arc::new(r), Arc::new(setup));
        for seed in 1..4 {
            let mut env = BattleEnv::new(r.clone(), setup.clone(), seed, 0).unwrap();
            for side in BattleSide::BOTH {
                assert!(env.state().may_retreat(side));
                assert!(env.state().may_surrender(&r, side));
            }
            let result = random_rollout(&mut env, seed, 5000).result;
            assert!(matches!(
                result,
                Some(BattleResult::Victory(_) | BattleResult::Draw)
            ));
        }
    }

    #[test]
    fn step_command_handles_unencodable_commands() {
        let (r, setup) = h3_pack::synthetic(h3_pack::SYNTHETIC_DUEL).unwrap();
        let mut env = BattleEnv::new(Arc::new(r), Arc::new(setup), 1, 0).unwrap();
        let side = env.to_move().unwrap();
        let bogus = Command::Attack {
            target: h3_battle::StackId(1),
            from: h3_battle::Hex(0),
        };
        assert_eq!(
            step_command(&mut env, side, bogus),
            Err(StepError::InvalidAction)
        );
        assert!(step_command(&mut env, side, Command::Defend).is_ok());
    }
}
