//! The adventure environment: one game on an adventure map (ADR-0017 point 8, E08 T9a), its
//! battles included (E08 T19b1: a player in a battle acts in the battle phase), a battle's unowned
//! side played from the neutral seat (E08 T20b3a: [`Seat::Neutral`], as the system actor).

use std::fmt;
use std::sync::Arc;

use h3_battle::LegalCommands;
use h3_contracts::PlayerId;
use h3_game::{
    GameCommand, GameCommandError, GameObservation, GameRuleset, GameSetup, GameSetupError,
    GameState, Sink, Viewer, apply, observe,
};

use super::action::{self, Action, EntryMask, Phase, Seat};

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
    /// The action does not decode to a command of the seat ([`action::decode`]).
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
/// it: each step is one seat's command, validated against the state and the actor it is issued
/// as (a player, the trusted identity of the agent; the system actor for the neutral seat, which
/// plays a battle's unowned side). Cheap to [`fork`](Self::fork): the rules and the setup are
/// shared, and the game state shares its static map and unchanged grid leaves (decision 6).
#[derive(Debug, Clone)]
pub struct AdventureEnv {
    rules: GameRuleset,
    setup: Arc<GameSetup>,
    state: GameState,
    steps: u32,
    max_days: u32,
}

impl AdventureEnv {
    /// A new environment; the game starts with `seed` at the setup's difficulty. Episodes are
    /// truncated when day `max_days + 1` starts, after `max_days` whole days (0 = no limit).
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

    /// Restarts the game with a new seed at the last game's difficulty.
    ///
    /// # Errors
    ///
    /// The setup's [`GameSetupError`] (none, since the last game was built from it).
    pub fn reset(&mut self, seed: u64) -> Result<(), GameSetupError> {
        self.reset_with_difficulty(seed, self.difficulty())
    }

    /// Restarts the game with a new seed at `difficulty`, which later resets keep (E08 T14b; a
    /// map's import sets [`h3_game::DEFAULT_DIFFICULTY`]). The difficulty is setup input, checked
    /// by the game's build as the rest of the setup.
    ///
    /// # Errors
    ///
    /// The setup's [`GameSetupError`]: [`GameSetupError::Difficulty`] for a difficulty the rules
    /// have no starting resources for. The environment is then unchanged.
    pub fn reset_with_difficulty(
        &mut self,
        seed: u64,
        difficulty: u8,
    ) -> Result<(), GameSetupError> {
        let mut setup = Arc::clone(&self.setup);
        if setup.difficulty != difficulty {
            Arc::make_mut(&mut setup).difficulty = difficulty;
        }
        self.state = GameState::new(&self.rules, &setup, seed)?;
        self.setup = setup;
        self.steps = 0;
        Ok(())
    }

    /// The game's difficulty (`GameSetup::difficulty`).
    pub fn difficulty(&self) -> u8 {
        self.setup.difficulty
    }

    /// Whether the episode is over: the day limit passed.
    pub fn is_done(&self) -> bool {
        self.max_days != 0 && self.state.day() > self.max_days
    }

    /// The seats that may act now (ADR-0017 point 8): the players in id order, those of the turn
    /// group whose turn is running (one in sequential play) and who are in no battle, and for each
    /// open battle the owner of its side to act (E08 T19b1: an attacked player acts in its battle
    /// outside its turn); then the neutral seat while a battle's side to act is owned by no player
    /// (a wandering monster's, E08 T20b3a: the caller plays it with a policy of its choice, the
    /// random agent until the battle lane ports the original's battle AI, decision 27); none once
    /// the episode is over.
    pub fn to_move(&self) -> impl Iterator<Item = Seat> + '_ {
        let done = self.is_done();
        let neutral = !done && self.state.unowned_battle().is_some();
        (0..=u8::MAX)
            .take(self.state.players())
            .map(PlayerId::new)
            .filter(move |&player| !done && self.may_act(player))
            .map(Seat::Player)
            .chain(neutral.then_some(Seat::Neutral))
    }

    /// Whether `player` may act now: the owner of its battle's side to act, or with no battle a
    /// player whose turn is running.
    fn may_act(&self, player: PlayerId) -> bool {
        self.state.battle(player).map_or_else(
            || self.state.is_acting(player),
            |battle| battle.owner_to_act() == Some(player),
        )
    }

    /// The phase `seat` acts in ([`action::phase`]): the battle phase while its player is in a
    /// battle, and always for the neutral seat.
    pub fn phase(&self, seat: impl Into<Seat>) -> Phase {
        action::phase(&self.state, seat)
    }

    /// The size of an action's tile head: the map's tiles.
    pub fn tile_count(&self) -> usize {
        action::tile_count(&self.state)
    }

    /// Fills `mask` with the legal map entries of `seat` ([`action::legal_mask`]); none in the
    /// battle phase (none for the neutral seat) or once the episode is over.
    pub fn legal_mask(&self, seat: impl Into<Seat>, mask: &mut EntryMask) {
        action::legal_mask(&self.state, &self.rules, seat, mask);
        if self.is_done() {
            mask.fill(false);
        }
    }

    /// Fills `mask` with the legal tiles of `seat` after choosing `entry`
    /// ([`action::tile_mask`]); none in the battle phase or once the episode is over.
    pub fn tile_mask(&self, seat: impl Into<Seat>, entry: usize, mask: &mut Vec<bool>) {
        action::tile_mask(&self.state, &self.rules, seat, entry, mask);
        if self.is_done() {
            mask.fill(false);
        }
    }

    /// Fills `mask` with the legal battle entries of `seat` ([`action::battle_mask`]): a player's
    /// for the side it owns, the neutral seat's for the unowned side to act; none in the map
    /// phase, while the other side is to act, or once the episode is over. `legal` is scratch
    /// space.
    pub fn battle_mask(
        &self,
        seat: impl Into<Seat>,
        legal: &mut LegalCommands,
        mask: &mut crate::action::ActionMask,
    ) {
        action::battle_mask(&self.state, &self.rules, seat, legal, mask);
        if self.is_done() {
            mask.fill(false);
        }
    }

    /// Fills `mask` with the legal spells of `seat` after the battle entry `entry`
    /// ([`action::spell_mask`]); none outside a battle or once the episode is over.
    pub fn spell_mask(
        &self,
        seat: impl Into<Seat>,
        entry: usize,
        legal: &mut LegalCommands,
        mask: &mut crate::action::SpellMask,
    ) {
        action::spell_mask(&self.state, &self.rules, seat, entry, legal, mask);
        if self.is_done() {
            mask.fill(false);
        }
    }

    /// Fills `mask` with the legal targets of `seat` after the battle entry and spell
    /// ([`action::target_mask`]); none outside a battle or once the episode is over.
    pub fn target_mask(
        &self,
        seat: impl Into<Seat>,
        prefix: (usize, u16),
        legal: &mut LegalCommands,
        mask: &mut crate::action::TargetMask,
    ) {
        action::target_mask(&self.state, &self.rules, seat, prefix, legal, mask);
        if self.is_done() {
            mask.fill(false);
        }
    }

    /// Fills `mask` with the legal second targets of `seat` after the battle entry, spell and
    /// target ([`action::second_mask`]); none outside a battle or once the episode is over.
    pub fn second_mask(
        &self,
        seat: impl Into<Seat>,
        prefix: (usize, u16, u16),
        legal: &mut LegalCommands,
        mask: &mut crate::action::TargetMask,
    ) {
        action::second_mask(&self.state, &self.rules, seat, prefix, legal, mask);
        if self.is_done() {
            mask.fill(false);
        }
    }

    /// Plays `action` for `seat` (a player: the trusted identity of the agent issuing it; the
    /// neutral seat: the system actor): a map action or, in its battle, a battle action
    /// ([`action::decode`]).
    ///
    /// # Errors
    ///
    /// A [`StepError`]; the environment is then unchanged.
    pub fn step(&mut self, seat: impl Into<Seat>, action: Action) -> Result<StepResult, StepError> {
        self.step_with(seat, action, &mut h3_game::NoopSink)
    }

    /// Like [`step`](Self::step), with the game's events (unprojected: every player's view of
    /// them is [`h3_game::project`]'s) going to `sink`.
    ///
    /// # Errors
    ///
    /// A [`StepError`]; the environment is then unchanged and no event was emitted.
    pub fn step_with<S: Sink>(
        &mut self,
        seat: impl Into<Seat>,
        action: Action,
        sink: &mut S,
    ) -> Result<StepResult, StepError> {
        let seat = seat.into();
        let command = self.decode(seat, action)?;
        self.step_command(seat, command, sink)
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
        seat: impl Into<Seat>,
        action: Action,
        sink: &mut S,
    ) -> Result<(StepResult, GameCommand), StepError> {
        let seat = seat.into();
        let command = self.decode(seat, action)?;
        let result = self.step_command(seat, command.clone(), sink)?;
        Ok((result, command))
    }

    /// The command `action` decodes to for `seat`, unless the episode is over.
    fn decode(&self, seat: Seat, action: Action) -> Result<GameCommand, StepError> {
        if self.is_done() {
            return Err(StepError::EpisodeOver);
        }
        action::decode(&self.state, &self.rules, seat, action).ok_or(StepError::InvalidAction)
    }

    /// Plays a game command for `seat` as a step, issued as the seat's actor ([`Seat::actor`]):
    /// a path of the player's own choice, which an action cannot name (a client computes its own
    /// paths, T8b2a).
    ///
    /// # Errors
    ///
    /// [`StepError::EpisodeOver`] or [`StepError::Rejected`]; the environment is then unchanged
    /// and no event was emitted.
    pub fn step_command<S: Sink>(
        &mut self,
        seat: impl Into<Seat>,
        command: GameCommand,
        sink: &mut S,
    ) -> Result<StepResult, StepError> {
        if self.is_done() {
            return Err(StepError::EpisodeOver);
        }
        apply(
            &mut self.state,
            &self.rules,
            seat.into().actor(),
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

    /// What the neutral seat may know while it is to act: the battle it plays as its unowned
    /// side sees it ([`action::battle_of`], `h3_battle::observe`); none while no battle waits for
    /// it. The neutral seat sees nothing of the map.
    pub fn neutral_observation(&self) -> Option<h3_battle::Observation> {
        action::battle_of(&self.state, Seat::Neutral)
            .map(|(battle, side)| h3_battle::observe(battle, self.rules.battle(), side))
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
