//! The battle environment (docs/07-simulation-and-ai.md §1).

use std::fmt;
use std::sync::Arc;

use h3_battle::{
    BattleState, CommandError, LegalCommands, NoopSink, Observation, SetupError, apply,
    generate_setup_obstacles, observe,
};
use h3_content_battle::BattleRuleset;
use h3_contracts::{BattleHero, BattleResult, BattleSetup, BattleSide, RetreatFacts};
use h3_kernel::{Rng, StreamId};

use crate::action::{self, Action, ActionMask, SpellMask, TargetMask};
use crate::observation;
use crate::reward::{self, RewardConfig};

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
    /// or surrenders loses), 0 otherwise (also for a draw, where both armies are gone); for an
    /// env scored by reward v1 ([`BattleEnv::with_reward`], a world's) reward v1 at the step
    /// that decides the battle, 0 otherwise.
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
    ruleset: Arc<BattleRuleset>,
    setup: Arc<BattleSetup>,
    /// Each episode gets obstacles generated for the setup's terrain from its seed.
    random_obstacles: bool,
    state: BattleState,
    steps: u32,
    max_steps: u32,
    truncated: bool,
    /// How a decided battle is scored: `None` ±1 by the winner, else reward v1 with these
    /// settings ([`Self::with_reward`]).
    reward: Option<RewardConfig>,
}

impl BattleEnv {
    /// A new environment; the battle starts with `seed`. Episodes are truncated after
    /// `max_steps` accepted steps (0 = no limit).
    pub fn new(
        ruleset: Arc<BattleRuleset>,
        setup: Arc<BattleSetup>,
        seed: u64,
        max_steps: u32,
    ) -> Result<Self, SetupError> {
        Self::build(ruleset, setup, false, seed, max_steps)
    }

    /// Like [`new`](BattleEnv::new), but every episode (this one and each [`reset`](Self::reset))
    /// gets obstacles generated for the setup's terrain and battlefield from stream `OBSTACLES` of
    /// its seed, as
    /// `h3 run` does for a scenario with `random_obstacles` (BAT-GEO-004); the setup's own
    /// obstacles are replaced. Without a terrain the setup is used as it is.
    pub fn with_random_obstacles(
        ruleset: Arc<BattleRuleset>,
        setup: Arc<BattleSetup>,
        seed: u64,
        max_steps: u32,
    ) -> Result<Self, SetupError> {
        Self::build(ruleset, setup, true, seed, max_steps)
    }

    fn build(
        ruleset: Arc<BattleRuleset>,
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
            reward: None,
        })
    }

    /// This env with a decided battle scored by reward v1 with `config` ([`reward::rewards`]
    /// from its setup and the battle's outcome, as the batch scores an episode) instead of ±1:
    /// a world's (E25 T1a), whose search compares it with a value head trained on reward v1.
    /// Its forks and worlds keep it.
    #[must_use]
    pub fn with_reward(self, config: RewardConfig) -> Self {
        Self {
            reward: Some(config),
            ..self
        }
    }

    /// The first state of an episode with `seed`.
    fn episode(
        ruleset: &BattleRuleset,
        setup: &BattleSetup,
        random_obstacles: bool,
        seed: u64,
    ) -> Result<BattleState, SetupError> {
        match (random_obstacles, setup.terrain) {
            (true, Some(_)) => {
                let mut rng = Rng::for_stream(seed, StreamId::OBSTACLES);
                let episode = BattleSetup {
                    obstacles: generate_setup_obstacles(ruleset, setup, &mut rng),
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

    /// Writes the engine's preview of each attack and shot in `legal` for `actor` into `out`
    /// ([`action::preview_kills`]; `legal` as [`Self::legal_mask`] left it).
    pub fn preview_kills(&self, actor: BattleSide, legal: &LegalCommands, out: &mut [f32]) {
        action::preview_kills(&self.state, &self.ruleset, actor, legal, out);
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

    /// Plays action `action` for `actor` (the trusted identity of the agent issuing it). A side
    /// that is not to move is refused as the battle refuses it
    /// ([`CommandError::NotYourTurn`]) before its action is decoded: decoding reads what only
    /// the side to move may know (the active stack's drawn spell, ADR-0014 point 8), and the kind
    /// of refusal must not tell it.
    pub fn step(&mut self, actor: BattleSide, action: Action) -> Result<StepResult, StepError> {
        let Some(side) = self.to_move() else {
            return Err(StepError::EpisodeOver);
        };
        if side != actor {
            return Err(StepError::Rejected(CommandError::NotYourTurn));
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
        let reward = match self.reward {
            // A decided battle has an outcome; a running or truncated one gets 0.
            Some(config) => self.state.outcome().map_or([0.0; 2], |outcome| {
                reward::rewards(config, &self.ruleset, &self.setup, &outcome)
            }),
            None => rewards(result),
        };
        Ok(StepResult {
            done: self.is_done(),
            truncated: self.truncated,
            result,
            reward,
        })
    }

    /// An independent copy (for search and rollouts).
    pub fn fork(&self) -> Self {
        self.clone()
    }

    /// A world of the battle for `viewer` (E06 T8, ADR-0025 point 12a;
    /// [`BattleState::determinized`]): the other side's hero's private part from `guess` (`None`:
    /// a hero of its type alone), the stream from `seed`; its steps, step limit and scoring are
    /// this env's. Its setup is this env's without what `viewer` may not know (the other side's
    /// hero the guess with the battle's starting mana, which its view showed at the start and
    /// reward v1's mana term reads, E25 T1a; its funds 0, its retreat facts the default, a town's
    /// Escape Tunnel as `viewer` sees it), so a reset of the world starts nothing hidden either.
    ///
    /// # Errors
    /// [`SetupError::InvalidHero`] for a guess the battle refuses.
    pub fn world(
        &self,
        viewer: BattleSide,
        guess: Option<&BattleHero>,
        seed: u64,
    ) -> Result<Self, SetupError> {
        let state = self
            .state
            .determinized(&self.ruleset, viewer, guess, seed)?;
        let other = viewer.opponent().index();
        let mut setup = (*self.setup).clone();
        setup.heroes[other] = setup.heroes[other].as_ref().map(|hero| {
            let mut guessed = guess
                .cloned()
                .unwrap_or_else(|| BattleHero::new(hero.hero_type));
            guessed.magic.mana = hero.magic.mana;
            guessed
        });
        setup.funds[other] = 0;
        setup.retreat[other] = RetreatFacts::default();
        if let Some(siege) = setup.siege.as_mut() {
            siege.escape_tunnel &= viewer == BattleSide::Defender;
        }
        Ok(Self {
            setup: Arc::new(setup),
            state,
            ..self.clone()
        })
    }

    /// Re-seeds the battle's stream (E06 T8: each of a world's simulations draws its own rolls;
    /// [`BattleState::reseed`]).
    pub fn reseed(&mut self, seed: u64) {
        self.state.reseed(seed);
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
    pub fn ruleset(&self) -> &Arc<BattleRuleset> {
        &self.ruleset
    }

    /// The setup every episode starts from (with random obstacles, each episode's obstacles are
    /// generated from its seed instead of these).
    pub fn setup(&self) -> &Arc<BattleSetup> {
        &self.setup
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{ACTION_COUNT, CREATURE_CAST, HERO_CAST, SPELL_COUNT, TARGET_COUNT};
    use h3_battle::Command;

    fn make(max_steps: u32) -> BattleEnv {
        let (r, setup) = h3_pack_battle::synthetic(h3_pack_battle::SYNTHETIC_DUEL).unwrap();
        BattleEnv::new(Arc::new(r), Arc::new(setup), 3, max_steps).unwrap()
    }

    #[test]
    fn random_obstacles_are_drawn_for_each_episode_from_its_seed() {
        // The duel scenario has a terrain and `random_obstacles` (BAT-GEO-004).
        let (r, setup) = h3_pack_battle::synthetic(h3_pack_battle::SYNTHETIC_DUEL).unwrap();
        let terrain = setup.terrain.unwrap();
        let (r, setup) = (Arc::new(r), Arc::new(setup));
        let expected = |seed| {
            let mut rng = Rng::for_stream(seed, StreamId::OBSTACLES);
            let pieces = h3_battle::generate_obstacles(&r, terrain, &mut rng);
            let mut set = h3_battle::HexSet::new();
            pieces
                .iter()
                .flat_map(|o| &o.hexes)
                .for_each(|&h| set.insert(h3_battle::Hex(h)));
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

    // covers: BAT-SIE-011, BAT-SIE-013
    /// Robustness at a town (E04c T10b review): over random play of the siege scenario (the
    /// baseline agent's choices, the wall shots among them) the mask agrees with `step` for every
    /// non-cast entry, the wall shots' included: a masked entry steps a fork, an unmasked one
    /// is refused and changes nothing; actions with out-of-range entries, spells and targets are
    /// refused without a change; the episode ends or reaches its bound.
    #[test]
    fn the_siege_scenario_s_mask_agrees_with_step_for_every_entry() {
        let (r, setup) = h3_pack_battle::synthetic(h3_pack_battle::SYNTHETIC_SIEGE).unwrap();
        let (r, setup) = (Arc::new(r), Arc::new(setup));
        let mut wall_shots = 0;
        for seed in 0..4u64 {
            let mut env = BattleEnv::new(r.clone(), setup.clone(), seed, 0).unwrap();
            let mut rng = Rng::for_stream(seed, StreamId::AGENT);
            let mut legal = LegalCommands::new();
            let mut mask = [false; ACTION_COUNT];
            for step in 0..250 {
                let Some(side) = env.to_move() else {
                    break;
                };
                env.legal_mask(side, &mut legal, &mut mask);
                let hash = env.state_hash();
                // Every entry every fifth step (a fork per entry).
                let entries = if step % 5 == 0 { ACTION_COUNT + 8 } else { 0 };
                for entry in 0..entries {
                    if entry == HERO_CAST || entry == CREATURE_CAST {
                        continue;
                    }
                    let mut fork = env.fork();
                    let stepped = fork.step(side, Action::entry(entry));
                    let allowed = mask.get(entry).copied().unwrap_or(false);
                    assert_eq!(
                        stepped.is_ok(),
                        allowed,
                        "seed {seed}: entry {entry}: {stepped:?}"
                    );
                    if !allowed {
                        assert_eq!(fork.state_hash(), hash, "seed {seed}: entry {entry}");
                    }
                }
                for _ in 0..4 {
                    let garbage = Action {
                        entry: rng.next_u32() >> (rng.below(32)),
                        spell: u16::try_from(rng.next_u32() & 0xffff).unwrap(),
                        target: u16::try_from(rng.next_u32() & 0xffff).unwrap(),
                        second: u16::try_from(rng.next_u32() & 0xffff).unwrap(),
                    };
                    let mut fork = env.fork();
                    if fork.step(side, garbage).is_err() {
                        assert_eq!(fork.state_hash(), hash, "seed {seed}: {garbage:?}");
                    }
                }
                // The observation encodes for both viewers (the turrets off the field among the
                // stacks); a buffer one short is refused.
                let mut out = vec![0.0; observation::OBSERVATION_LEN];
                for viewer in BattleSide::BOTH {
                    assert_eq!(env.encode_observation(viewer, &mut out), Ok(()));
                    assert!(env.encode_observation(viewer, &mut out[1..]).is_err());
                }
                let command = crate::rollout::pick(&env, side, &mut rng, &mut legal);
                wall_shots += u32::from(matches!(command, Command::ShootWall { .. }));
                crate::rollout::step_command(&mut env, side, command).unwrap();
            }
            assert_eq!(env.state().outcome().is_some(), env.is_done());
        }
        assert!(wall_shots > 10, "{wall_shots}");
    }

    /// E04b T11b review (F-sec-2): the side that is not to move is refused before its action
    /// is decoded. Decoding a stack's cast compares the action's spell with the active stack's
    /// (a random caster's drawn spell, which only its controller may know, ADR-0014 point 8), so
    /// the kind of refusal told the other side that spell: `Rejected` for it, `InvalidAction`
    /// for every other.
    // covers: BAT-MAG-049
    #[test]
    fn a_step_out_of_turn_is_refused_whatever_the_action() {
        use crate::action::{CREATURE_CAST, END_TACTICS};
        let (r, setup) = h3_pack_battle::synthetic(h3_pack_battle::SYNTHETIC_FULL).unwrap();
        let mut env = BattleEnv::new(Arc::new(r), Arc::new(setup), 3, 0).unwrap();
        // Everybody defends until a random caster (the Drakes) awaits its side's command.
        let controller = loop {
            let side = env.to_move().unwrap();
            if env.observation(side).drawn.is_some() {
                break side;
            }
            let entry = if env.state().tactics().is_some() {
                END_TACTICS
            } else {
                1
            };
            env.step(side, Action::entry(entry)).unwrap();
        };
        let other = controller.opponent();
        assert_eq!(env.observation(other).drawn, None);
        let before = env.state_hash();
        let cast = |spell| Action {
            entry: CREATURE_CAST as u32,
            spell,
            target: 0,
            second: 0,
        };
        // The other side probes every spell: one answer.
        for spell in 0..SPELL_COUNT as u16 {
            assert_eq!(
                env.step(other, cast(spell)),
                Err(StepError::Rejected(CommandError::NotYourTurn)),
                "{spell}"
            );
        }
        // Nor is an action that names no command decoded for it.
        assert_eq!(
            env.step(other, Action::entry(ACTION_COUNT + 1)),
            Err(StepError::Rejected(CommandError::NotYourTurn))
        );
        // The controller's own actions are decoded as before: only the drawn spell names its
        // stack's cast (which needs a target: rejected, not invalid).
        let drawn = env.observation(controller).drawn.unwrap().raw();
        for spell in 0..SPELL_COUNT as u16 {
            let refused = env.step(controller, cast(spell)).unwrap_err();
            assert_eq!(
                matches!(refused, StepError::Rejected(_)),
                spell == drawn,
                "{spell}: {refused}"
            );
        }
        assert_eq!(env.state_hash(), before, "refused steps change nothing");
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
        assert_eq!(a.ruleset().creature_count(), 15);
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
