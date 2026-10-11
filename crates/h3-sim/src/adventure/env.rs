//! The adventure environment: one game on an adventure map (ADR-0017 point 8, E08 T9a).

use std::fmt;
use std::sync::Arc;

use h3_contracts::PlayerId;
use h3_game::{
    Actor, GameCommand, GameCommandError, GameObservation, GameRuleset, GameSetup, GameSetupError,
    GameState, Sink, Viewer, apply, observe,
};

use super::action::{self, Action, EntryMask};

/// Result of one environment step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepResult {
    /// The episode is over. Increment 1 has no victory or defeat: an episode ends only by its day
    /// limit.
    pub done: bool,
    /// The episode was cut off by the day limit (an RL truncation, not a game rule).
    pub truncated: bool,
}

/// Why a step was refused. The environment is unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepError {
    /// The action does not decode to a command of the player ([`action::decode`]).
    InvalidAction,
    /// The game rejected the command.
    Rejected(GameCommandError),
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

/// A game on an adventure map as an RL environment, driven through `h3-game` as a player drives
/// it: each step is one player's command, validated against the state and the player it is
/// issued for (the trusted identity of the agent). Cheap to [`fork`](Self::fork): the rules and
/// the setup are shared, and the game state shares its static map and unchanged grid leaves
/// (decision 6).
#[derive(Debug, Clone)]
pub struct AdventureEnv {
    rules: GameRuleset,
    setup: Arc<GameSetup>,
    state: GameState,
    steps: u32,
    max_days: u32,
}

impl AdventureEnv {
    /// A new environment; the game starts with `seed`. Episodes are truncated when day
    /// `max_days + 1` starts, after `max_days` whole days (0 = no limit).
    ///
    /// # Errors
    ///
    /// The setup's [`GameSetupError`].
    pub fn new(
        rules: GameRuleset,
        setup: Arc<GameSetup>,
        seed: u64,
        max_days: u32,
    ) -> Result<Self, GameSetupError> {
        let state = GameState::new(&rules, &setup, seed)?;
        Ok(Self {
            rules,
            setup,
            state,
            steps: 0,
            max_days,
        })
    }

    /// Restarts the game with a new seed.
    ///
    /// # Errors
    ///
    /// The setup's [`GameSetupError`] (none, since [`new`](Self::new) built it).
    pub fn reset(&mut self, seed: u64) -> Result<(), GameSetupError> {
        self.state = GameState::new(&self.rules, &self.setup, seed)?;
        self.steps = 0;
        Ok(())
    }

    /// Whether the episode is over: the day limit passed.
    pub fn is_done(&self) -> bool {
        self.max_days != 0 && self.state.day() > self.max_days
    }

    /// The players that may act now, in id order: those of the turn group whose turn is running
    /// (one in sequential play); none once the episode is over.
    pub fn to_move(&self) -> impl Iterator<Item = PlayerId> + '_ {
        let done = self.is_done();
        self.state
            .turn_group()
            .iter()
            .copied()
            .filter(move |&player| !done && self.state.is_acting(player))
    }

    /// The size of an action's tile head: the map's tiles.
    pub fn tile_count(&self) -> usize {
        action::tile_count(&self.state)
    }

    /// Fills `mask` with the legal entries of `player` ([`action::legal_mask`]); none once the
    /// episode is over.
    pub fn legal_mask(&self, player: PlayerId, mask: &mut EntryMask) {
        action::legal_mask(&self.state, &self.rules, player, mask);
        if self.is_done() {
            mask.fill(false);
        }
    }

    /// Fills `mask` with the legal tiles of `player` after choosing `entry`
    /// ([`action::tile_mask`]); none once the episode is over.
    pub fn tile_mask(&self, player: PlayerId, entry: usize, mask: &mut Vec<bool>) {
        action::tile_mask(&self.state, &self.rules, player, entry, mask);
        if self.is_done() {
            mask.fill(false);
        }
    }

    /// Plays `action` for `player` (the trusted identity of the agent issuing it).
    ///
    /// # Errors
    ///
    /// A [`StepError`]; the environment is then unchanged.
    pub fn step(&mut self, player: PlayerId, action: Action) -> Result<StepResult, StepError> {
        self.step_with(player, action, &mut h3_game::NoopSink)
    }

    /// Like [`step`](Self::step), with the game's events (unprojected: every player's view of
    /// them is [`h3_game::project`]'s) going to `sink`.
    ///
    /// # Errors
    ///
    /// A [`StepError`]; the environment is then unchanged and no event was emitted.
    pub fn step_with<S: Sink>(
        &mut self,
        player: PlayerId,
        action: Action,
        sink: &mut S,
    ) -> Result<StepResult, StepError> {
        let command = self.decode(player, action)?;
        self.step_command(player, command, sink)
    }

    /// Like [`step_with`](Self::step_with), also returning the game command the action decoded to
    /// (what a replay records, E08 T9b): one reachable-tiles query, as a step; the command is
    /// cloned for the caller, which a plain step does not pay.
    ///
    /// # Errors
    ///
    /// A [`StepError`]; the environment is then unchanged and no event was emitted.
    pub fn step_decoded<S: Sink>(
        &mut self,
        player: PlayerId,
        action: Action,
        sink: &mut S,
    ) -> Result<(StepResult, GameCommand), StepError> {
        let command = self.decode(player, action)?;
        let result = self.step_command(player, command.clone(), sink)?;
        Ok((result, command))
    }

    /// The command `action` decodes to for `player`, unless the episode is over.
    fn decode(&self, player: PlayerId, action: Action) -> Result<GameCommand, StepError> {
        if self.is_done() {
            return Err(StepError::EpisodeOver);
        }
        action::decode(&self.state, &self.rules, player, action).ok_or(StepError::InvalidAction)
    }

    /// Plays a game command for `player` as a step: a path of the player's own choice, which an
    /// action cannot name (a client computes its own paths, T8b2a).
    ///
    /// # Errors
    ///
    /// [`StepError::EpisodeOver`] or [`StepError::Rejected`]; the environment is then unchanged
    /// and no event was emitted.
    pub fn step_command<S: Sink>(
        &mut self,
        player: PlayerId,
        command: GameCommand,
        sink: &mut S,
    ) -> Result<StepResult, StepError> {
        if self.is_done() {
            return Err(StepError::EpisodeOver);
        }
        apply(
            &mut self.state,
            &self.rules,
            Actor::Player(player),
            command,
            sink,
        )
        .map_err(StepError::Rejected)?;
        self.steps = self.steps.saturating_add(1);
        let done = self.is_done();
        Ok(StepResult {
            done,
            truncated: done,
        })
    }

    /// An independent copy (for search and rollouts).
    pub fn fork(&self) -> Self {
        self.clone()
    }

    /// Canonical hash of the game state.
    pub fn state_hash(&self) -> u64 {
        self.state.state_hash()
    }

    /// What `viewer` may know of the game ([`h3_game::observe`]).
    pub fn observation(&self, viewer: PlayerId) -> GameObservation {
        observe(&self.state, &self.rules, Viewer::Player(viewer))
    }

    /// Accepted steps in this episode.
    pub fn steps(&self) -> u32 {
        self.steps
    }

    /// The day, from 1.
    pub fn day(&self) -> u32 {
        self.state.day()
    }

    /// The game state (for tools and tests; agents should use observations).
    pub fn state(&self) -> &GameState {
        &self.state
    }

    /// The rules.
    pub fn rules(&self) -> &GameRuleset {
        &self.rules
    }
}
