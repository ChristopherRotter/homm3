//! The battle environment (docs/07-simulation-and-ai.md §1).

use std::fmt;
use std::sync::Arc;

use h3_battle::{
    BattleState, CommandError, LegalCommands, NoopSink, Observation, SetupError, apply,
    generate_obstacles, observe,
};
use h3_content::Ruleset;
use h3_contracts::{BattleResult, BattleSetup, BattleSide};
use h3_kernel::{Rng, StreamId};

use crate::action::{self, Action, ActionMask, SpellMask, TargetMask};
use crate::observation;

/// Result of one environment step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StepResult {
    /// The episode is over (battle decided or truncated).
    pub done: bool,
    /// The episode was cut off by the step limit (an RL truncation, not a game rule).
    pub truncated: bool,
    /// How the battle ended, if it did (a victory, a draw, a retreat or a surrender).
    pub result: Option<BattleResult>,
    /// Reward per side, indexed by [`BattleSide::index`]: +1 win, −1 loss (a side that retreats
    /// or surrenders loses), 0 otherwise (also for a draw, where both armies are gone).
    pub reward: [f32; 2],
}

/// The rewards for a battle `result` ([`StepResult::reward`]).
fn rewards(result: Option<BattleResult>) -> [f32; 2] {
    match result.and_then(BattleResult::winner) {
        Some(w) => {
            let mut r = [-1.0; 2];
            r[w.index()] = 1.0;
            r
        }
        None => [0.0; 2],
    }
}

/// Why a step was refused. The environment is unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepError {
    /// The action does not decode to a command.
    InvalidAction,
    /// The command was rejected by the battle's validation.
    Rejected(CommandError),
    /// The episode is over; call `reset`.
    EpisodeOver,
}

impl fmt::Display for StepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAction => f.write_str("invalid action"),
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
    /// Each episode gets obstacles generated for the setup's terrain from its seed.
    random_obstacles: bool,
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
        Self::build(ruleset, setup, false, seed, max_steps)
    }

    /// Like [`new`](BattleEnv::new), but every episode (this one and each [`reset`](Self::reset))
    /// gets obstacles generated for the setup's terrain from stream `OBSTACLES` of its seed, as
    /// `h3 run` does for a scenario with `random_obstacles` (BAT-GEO-004); the setup's own
    /// obstacles are replaced. Without a terrain the setup is used as it is.
    pub fn with_random_obstacles(
        ruleset: Arc<Ruleset>,
        setup: Arc<BattleSetup>,
        seed: u64,
        max_steps: u32,
    ) -> Result<Self, SetupError> {
        Self::build(ruleset, setup, true, seed, max_steps)
    }

    fn build(
        ruleset: Arc<Ruleset>,
        setup: Arc<BattleSetup>,
        random_obstacles: bool,
        seed: u64,
        max_steps: u32,
    ) -> Result<Self, SetupError> {
        let state = Self::episode(&ruleset, &setup, random_obstacles, seed)?;
        Ok(Self {
            ruleset,
            setup,
            random_obstacles,
            state,
            steps: 0,
            max_steps,
            truncated: false,
        })
    }

    /// The first state of an episode with `seed`.
    fn episode(
        ruleset: &Ruleset,
        setup: &BattleSetup,
        random_obstacles: bool,
        seed: u64,
    ) -> Result<BattleState, SetupError> {
        match (random_obstacles, setup.terrain) {
            (true, Some(terrain)) => {
                let mut rng = Rng::for_stream(seed, StreamId::OBSTACLES);
                let episode = BattleSetup {
                    obstacles: generate_obstacles(ruleset, terrain, &mut rng),
                    ..setup.clone()
                };
                BattleState::new(ruleset, &episode, seed)
            }
            _ => BattleState::new(ruleset, setup, seed),
        }
    }

    /// Restarts the battle with a new seed (and, with random obstacles, new obstacles).
    pub fn reset(&mut self, seed: u64) -> Result<(), SetupError> {
        self.state = Self::episode(&self.ruleset, &self.setup, self.random_obstacles, seed)?;
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

    /// Fills `mask` with the legal entries of `actor` (the first head of an [`Action`]).
    pub fn legal_mask(&self, actor: BattleSide, legal: &mut LegalCommands, mask: &mut ActionMask) {
        mask.fill(false);
        if !self.truncated {
            action::legal_mask(&self.state, &self.ruleset, actor, legal, mask);
        }
    }

    /// Fills `mask` with the legal spells of `actor` after choosing `entry`
    /// ([`action::spell_mask`]).
    pub fn spell_mask(
        &self,
        actor: BattleSide,
        entry: usize,
        legal: &mut LegalCommands,
        mask: &mut SpellMask,
    ) {
        mask.fill(false);
        if !self.truncated {
            action::spell_mask(&self.state, &self.ruleset, actor, entry, legal, mask);
        }
    }

    /// Fills `mask` with the legal targets of `actor` after choosing `entry` and `spell`
    /// ([`action::target_mask`]).
    pub fn target_mask(
        &self,
        actor: BattleSide,
        prefix: (usize, u16),
        legal: &mut LegalCommands,
        mask: &mut TargetMask,
    ) {
        mask.fill(false);
        if !self.truncated {
            action::target_mask(&self.state, &self.ruleset, actor, prefix, legal, mask);
        }
    }

    /// Fills `mask` with the legal second targets of `actor` after choosing `entry`, `spell`
    /// and `target` ([`action::second_mask`]).
    pub fn second_mask(
        &self,
        actor: BattleSide,
        prefix: (usize, u16, u16),
        legal: &mut LegalCommands,
        mask: &mut TargetMask,
    ) {
        mask.fill(false);
        if !self.truncated {
            action::second_mask(&self.state, &self.ruleset, actor, prefix, legal, mask);
        }
    }

    /// Plays action `action` for `actor` (the trusted identity of the agent issuing it).
    pub fn step(&mut self, actor: BattleSide, action: Action) -> Result<StepResult, StepError> {
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
        self.steps = self.steps.saturating_add(1);
        let result = self.state.result();
        if result.is_none() && self.max_steps != 0 && self.steps >= self.max_steps {
            self.truncated = true;
        }
        Ok(StepResult {
            done: self.is_done(),
            truncated: self.truncated,
            result,
            reward: rewards(result),
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
        observe(&self.state, &self.ruleset, viewer)
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
    use crate::action::{ACTION_COUNT, SPELL_COUNT, TARGET_COUNT};

    fn make(max_steps: u32) -> BattleEnv {
        let (r, setup) = h3_pack::synthetic(h3_pack::SYNTHETIC_DUEL).unwrap();
        BattleEnv::new(Arc::new(r), Arc::new(setup), 3, max_steps).unwrap()
    }

    #[test]
    fn random_obstacles_are_drawn_for_each_episode_from_its_seed() {
        // The duel scenario has a terrain and `random_obstacles` (BAT-GEO-004).
        let (r, setup) = h3_pack::synthetic(h3_pack::SYNTHETIC_DUEL).unwrap();
        let terrain = setup.terrain.unwrap();
        let (r, setup) = (Arc::new(r), Arc::new(setup));
        let expected = |seed| {
            let mut rng = Rng::for_stream(seed, StreamId::OBSTACLES);
            let hexes = generate_obstacles(&r, terrain, &mut rng);
            let mut set = h3_battle::HexSet::new();
            hexes.iter().for_each(|&h| set.insert(h3_battle::Hex(h)));
            set
        };
        let mut env = BattleEnv::with_random_obstacles(r.clone(), setup.clone(), 3, 0).unwrap();
        assert!(!env.state().obstacles().is_empty());
        assert_eq!(env.state().obstacles(), expected(3));
        env.reset(4).unwrap();
        assert_eq!(env.state().obstacles(), expected(4));
        assert_ne!(expected(3), expected(4));
        // Without the option, or without a terrain, the setup's own obstacles (none here).
        assert!(make(0).state().obstacles().is_empty());
        let bare = Arc::new(BattleSetup {
            terrain: None,
            ..(*setup).clone()
        });
        let env = BattleEnv::with_random_obstacles(r, bare, 3, 0).unwrap();
        assert!(env.state().obstacles().is_empty());
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
        // The heads of a legal non-cast entry allow exactly 0.
        let mut spells = [false; SPELL_COUNT];
        let mut targets = [false; TARGET_COUNT];
        env.spell_mask(side, action, &mut legal, &mut spells);
        assert_eq!(spells.iter().position(|m| *m), Some(0));
        env.target_mask(side, (action, 0), &mut legal, &mut targets);
        assert_eq!(targets.iter().position(|m| *m), Some(0));
        env.second_mask(side, (action, 0, 0), &mut legal, &mut targets);
        assert_eq!(targets.iter().position(|m| *m), Some(0));
        assert_eq!(
            env.step(side, Action::entry(ACTION_COUNT + 1)),
            Err(StepError::InvalidAction)
        );
        assert_eq!(
            env.step(side.opponent(), Action::entry(action)),
            Err(StepError::Rejected(CommandError::NotYourTurn))
        );
        assert_eq!(env.state_hash(), before, "refused steps change nothing");
        let result = env.step(side, Action::entry(action)).unwrap();
        assert!(!result.done);
        assert_eq!(result.reward, [0.0, 0.0]);
        assert_eq!(env.steps(), 1);
        assert_eq!(StepError::InvalidAction.to_string(), "invalid action");
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
            let r = env.step(side, Action::entry(1)).unwrap();
            assert_eq!(r.truncated, i == 1);
        }
        assert!(env.is_done());
        assert_eq!(env.to_move(), None);
        assert_eq!(
            env.step(BattleSide::Attacker, Action::entry(1)),
            Err(StepError::EpisodeOver)
        );
        env.legal_mask(BattleSide::Attacker, &mut legal, &mut mask);
        assert!(mask.iter().all(|m| !m));
        // The heads are empty too.
        let mut spells = [true; SPELL_COUNT];
        let mut targets = [true; TARGET_COUNT];
        env.spell_mask(BattleSide::Attacker, 1, &mut legal, &mut spells);
        assert!(spells.iter().all(|m| !m));
        env.target_mask(BattleSide::Attacker, (1, 0), &mut legal, &mut targets);
        assert!(targets.iter().all(|m| !m));
        targets = [true; TARGET_COUNT];
        env.second_mask(BattleSide::Attacker, (1, 0, 0), &mut legal, &mut targets);
        assert!(targets.iter().all(|m| !m));
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
            a.step(side, Action::entry(1)).unwrap();
            b.step(side, Action::entry(1)).unwrap();
            assert_eq!(a.state_hash(), b.state_hash());
        }
        let side = a.to_move().unwrap();
        a.step(side, Action::entry(0)).unwrap();
        assert_ne!(a.state_hash(), b.state_hash());
        assert_eq!(a.ruleset().creature_count(), 6);
        assert_eq!(a.state().round(), b.state().round());
    }

    #[test]
    fn rewards_on_decision() {
        let mut env = make(0);
        let mut agent = Rng::from_seed(5);
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
            last = Some(env.step(side, Action::entry(a)).unwrap());
        }
        let r = last.unwrap();
        let w = r.result.and_then(BattleResult::winner).unwrap();
        assert!(r.done && !r.truncated);
        assert_eq!(r.reward[w.index()], 1.0);
        assert_eq!(r.reward[w.opponent().index()], -1.0);
    }

    #[test]
    fn rewards_by_result() {
        let won = |side| rewards(Some(BattleResult::Victory(side)));
        assert_eq!(won(BattleSide::Attacker), [1.0, -1.0]);
        assert_eq!(won(BattleSide::Defender), [-1.0, 1.0]);
        assert_eq!(rewards(Some(BattleResult::Draw)), [0.0, 0.0]);
        assert_eq!(rewards(None), [0.0, 0.0]);
        let left = BattleResult::Retreat(BattleSide::Attacker);
        assert_eq!(rewards(Some(left)), [-1.0, 1.0]);
        let left = BattleResult::Surrender(BattleSide::Defender);
        assert_eq!(rewards(Some(left)), [1.0, -1.0]);
    }
}
