//! Baseline agents: the uniformly random legal agent (also the benchmark workload).

use h3_battle::{Command, LegalCommands, legal_commands};
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

/// Picks a uniformly random legal command for `side` among those that do not leave the battle:
/// the baseline agent fights every battle out (Retreat and Surrender stay legal for other
/// agents; user decision 2026-10-04, so benchmarks and golden replays play whole battles).
pub fn pick(
    env: &BattleEnv,
    side: BattleSide,
    rng: &mut Rng,
    legal: &mut LegalCommands,
) -> Command {
    legal_commands(env.state(), env.ruleset(), side, legal);
    choose(legal, rng)
}

/// The baseline agent's choice from a legal set: uniform among the commands that do not leave
/// the battle (they come last in `legal_commands`). Shared by [`pick`] and
/// [`crate::Replay::record_random`].
pub fn choose(legal: &LegalCommands, rng: &mut Rng) -> Command {
    let staying = legal.iter().take_while(|c| !c.leaves_battle()).count();
    let n = u32::try_from(staying).unwrap_or(u32::MAX);
    legal
        .get(rng.below(n) as usize)
        .copied()
        .unwrap_or(Command::Defend)
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
        // Brea may retreat on every turn of the full scenario (BAT-FLE-001); the agent never does.
        let (r, setup) = h3_pack::synthetic(h3_pack::SYNTHETIC_FULL).unwrap();
        let (r, setup) = (Arc::new(r), Arc::new(setup));
        for seed in 1..4 {
            let mut env = BattleEnv::new(r.clone(), setup.clone(), seed, 0).unwrap();
            assert!(env.state().may_retreat(BattleSide::Attacker));
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
