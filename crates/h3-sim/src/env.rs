//! The battle environment (docs/07-simulation-and-ai.md §1).

use std::fmt;
use std::sync::Arc;

use h3_battle::{
    BattleState, CommandError, LegalCommands, NoopSink, Observation, SetupError, apply, observe,
};
use h3_content::Ruleset;
use h3_contracts::{BattleSetup, BattleSide};

use crate::action::{self, ActionMask};
use crate::observation;

/// Result of one environment step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StepResult {
    /// The episode is over (battle decided or truncated).
    pub done: bool,
    /// The episode was cut off by the step limit (an RL truncation, not a game rule).
    pub truncated: bool,
    /// The winner, if the battle was decided.
    pub winner: Option<BattleSide>,
    /// Reward per side, indexed by [`BattleSide::index`]: +1 win, −1 loss, 0 otherwise.
    pub reward: [f32; 2],
}

/// Why a step was refused. The environment is unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepError {
    /// The action index does not decode to a command.
    InvalidAction,
    /// The command was rejected by the battle's validation.
    Rejected(CommandError),
    /// The episode is over; call `reset`.
    EpisodeOver,
}

impl fmt::Display for StepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAction => f.write_str("invalid action index"),
            Self::Rejected(e) => write!(f, "rejected: {e}"),
            Self::EpisodeOver => f.write_str("the episode is over"),
        }
    }
}

impl std::error::Error for StepError {}

/// One battle as an RL environment. Cheap to [`fork`](BattleEnv::fork): the ruleset and setup
/// are shared, the state is a fixed-size value.
#[derive(Debug, Clone)]
pub struct BattleEnv {
    ruleset: Arc<Ruleset>,
    setup: Arc<BattleSetup>,
    state: BattleState,
    steps: u32,
    max_steps: u32,
    truncated: bool,
}

impl BattleEnv {
    /// A new environment; the battle starts with `seed`. Episodes are truncated after
    /// `max_steps` accepted steps (0 = no limit).
    pub fn new(
        ruleset: Arc<Ruleset>,
        setup: Arc<BattleSetup>,
        seed: u64,
        max_steps: u32,
    ) -> Result<Self, SetupError> {
        let state = BattleState::new(&ruleset, &setup, seed)?;
        Ok(Self {
            ruleset,
            setup,
            state,
            steps: 0,
            max_steps,
            truncated: false,
        })
    }

    /// Restarts the battle with a new seed.
    pub fn reset(&mut self, seed: u64) -> Result<(), SetupError> {
        self.state = BattleState::new(&self.ruleset, &self.setup, seed)?;
        self.steps = 0;
        self.truncated = false;
        Ok(())
    }

    /// The side that must act, or `None` if the episode is over.
    pub fn to_move(&self) -> Option<BattleSide> {
        if self.truncated {
            None
        } else {
            self.state.side_to_act()
        }
    }

    /// Whether the episode is over.
    pub fn is_done(&self) -> bool {
        self.to_move().is_none()
    }

    /// Fills `mask` with the legal actions of `actor`.
    pub fn legal_mask(&self, actor: BattleSide, legal: &mut LegalCommands, mask: &mut ActionMask) {
        if self.truncated {
            mask.fill(false);
            return;
        }
        action::legal_mask(&self.state, &self.ruleset, actor, legal, mask);
    }

    /// Plays action `action` for `actor` (the trusted identity of the agent issuing it).
    pub fn step(&mut self, actor: BattleSide, action: usize) -> Result<StepResult, StepError> {
        if self.is_done() {
            return Err(StepError::EpisodeOver);
        }
        let command =
            action::decode(&self.state, &self.ruleset, action).ok_or(StepError::InvalidAction)?;
        apply(
            &mut self.state,
            &self.ruleset,
            actor,
            command,
            &mut NoopSink,
        )
        .map_err(StepError::Rejected)?;
        self.steps += 1;
        let winner = self.state.winner();
        if winner.is_none() && self.max_steps != 0 && self.steps >= self.max_steps {
            self.truncated = true;
        }
        let reward = match winner {
            Some(w) => {
                let mut r = [-1.0; 2];
                r[w.index()] = 1.0;
                r
            }
            None => [0.0; 2],
        };
        Ok(StepResult {
            done: self.is_done(),
            truncated: self.truncated,
            winner,
            reward,
        })
    }

    /// An independent copy (for search and rollouts).
    pub fn fork(&self) -> Self {
        self.clone()
    }

    /// Canonical hash of the battle state.
    pub fn state_hash(&self) -> u64 {
        self.state.state_hash()
    }

    /// The observation for `viewer`.
    pub fn observation(&self, viewer: BattleSide) -> Observation {
        observe(&self.state, viewer)
    }

    /// Writes the flat observation encoding for `viewer` into `out`
    /// (length [`crate::OBSERVATION_LEN`]).
    pub fn encode_observation(&self, viewer: BattleSide, out: &mut [f32]) -> Result<(), usize> {
        observation::encode(
            &self.observation(viewer),
            self.ruleset.battle().field.width,
            out,
        )
    }

    /// Accepted steps in this episode.
    pub fn steps(&self) -> u32 {
        self.steps
    }

    /// The battle state (for tools and tests; agents should use observations).
    pub fn state(&self) -> &BattleState {
        &self.state
    }

    /// The ruleset.
    pub fn ruleset(&self) -> &Arc<Ruleset> {
        &self.ruleset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::ACTION_COUNT;

    fn make(max_steps: u32) -> BattleEnv {
        let (r, setup) = h3_pack::synthetic(h3_pack::SYNTHETIC_DUEL).unwrap();
        BattleEnv::new(Arc::new(r), Arc::new(setup), 3, max_steps).unwrap()
    }

    #[test]
    fn step_mask_and_errors() {
        let mut env = make(0);
        let side = env.to_move().unwrap();
        let mut legal = LegalCommands::new();
        let mut mask = [false; ACTION_COUNT];
        env.legal_mask(side, &mut legal, &mut mask);
        let action = mask.iter().position(|m| *m).unwrap();
        let before = env.state_hash();
        assert_eq!(
            env.step(side, ACTION_COUNT + 1),
            Err(StepError::InvalidAction)
        );
        assert_eq!(
            env.step(side.opponent(), action),
            Err(StepError::Rejected(CommandError::NotYourTurn))
        );
        assert_eq!(env.state_hash(), before, "refused steps change nothing");
        let result = env.step(side, action).unwrap();
        assert!(!result.done);
        assert_eq!(result.reward, [0.0, 0.0]);
        assert_eq!(env.steps(), 1);
        assert_eq!(StepError::InvalidAction.to_string(), "invalid action index");
        assert_eq!(StepError::EpisodeOver.to_string(), "the episode is over");
        assert_eq!(
            StepError::Rejected(CommandError::NotYourTurn).to_string(),
            "rejected: it is not your turn"
        );
    }

    #[test]
    fn truncation_and_reset() {
        let mut env = make(2);
        let mut legal = LegalCommands::new();
        let mut mask = [false; ACTION_COUNT];
        for i in 0..2 {
            let side = env.to_move().unwrap();
            // Action 1 = Defend is always legal.
            let r = env.step(side, 1).unwrap();
            assert_eq!(r.truncated, i == 1);
        }
        assert!(env.is_done());
        assert_eq!(env.to_move(), None);
        assert_eq!(
            env.step(BattleSide::Attacker, 1),
            Err(StepError::EpisodeOver)
        );
        env.legal_mask(BattleSide::Attacker, &mut legal, &mut mask);
        assert!(mask.iter().all(|m| !m));
        env.reset(3).unwrap();
        assert!(!env.is_done());
        assert_eq!(env.steps(), 0);
        assert_eq!(env.state_hash(), make(2).state_hash());
    }

    #[test]
    fn fork_is_independent_and_equivalent() {
        let mut a = make(0);
        let mut b = a.fork();
        for _ in 0..4 {
            let side = a.to_move().unwrap();
            a.step(side, 1).unwrap();
            b.step(side, 1).unwrap();
            assert_eq!(a.state_hash(), b.state_hash());
        }
        let side = a.to_move().unwrap();
        a.step(side, 0).unwrap();
        assert_ne!(a.state_hash(), b.state_hash());
        assert_eq!(a.ruleset().creature_count(), 3);
        assert_eq!(a.state().round(), b.state().round());
    }

    #[test]
    fn rewards_on_decision() {
        let mut env = make(0);
        let mut agent = h3_kernel::Rng::from_seed(5);
        let mut legal = LegalCommands::new();
        let mut mask = [false; ACTION_COUNT];
        let mut last = None;
        while let Some(side) = env.to_move() {
            env.legal_mask(side, &mut legal, &mut mask);
            let actions: Vec<usize> = (0..ACTION_COUNT).filter(|i| mask[*i]).collect();
            // Prefer attacks to finish quickly.
            let attacks: Vec<usize> = actions.iter().copied().filter(|a| *a >= 257).collect();
            let pool = if attacks.is_empty() {
                &actions
            } else {
                &attacks
            };
            let a = pool[agent.below(pool.len() as u32) as usize];
            last = Some(env.step(side, a).unwrap());
        }
        let r = last.unwrap();
        let w = r.winner.unwrap();
        assert!(r.done && !r.truncated);
        assert_eq!(r.reward[w.index()], 1.0);
        assert_eq!(r.reward[w.opponent().index()], -1.0);
    }
}
