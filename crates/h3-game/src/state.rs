//! The whole game's state (ADR-0017, E08 T6b): the players with their RNG streams, the day's
//! schedule of turn groups, the calendar, the game's own stream and the contexts' states
//! (the adventure now). Built from a validated [`GameSetup`]; the steps only the game triggers
//! (the start of a day, the start of a turn) are infallible, and commands go through
//! [`crate::apply`].

use alloc::vec::Vec;

use h3_adventure::AdventureState;
use h3_contracts::PlayerId;
use h3_kernel::hash::hash_of;
use h3_kernel::{CanonicalHash, Rng, StateHasher, StreamId};

use crate::command::{GameCommandError, GameEvent, NoopSink, Sink};
use crate::{GAME_STATE_HASH_VERSION, GameRuleset, GameSetup, GameSetupError};

/// A player of a running game.
#[derive(Debug, Clone, PartialEq, Eq)]
struct PlayerState {
    /// Its colour (0 red to 7 pink).
    colour: u8,
    /// Its stream: randomness its commands cause draws only from it (ADR-0017 point 4).
    rng: Rng,
    /// Whether its turn is running: its group's turn has started and it has not ended its turn.
    acting: bool,
}

/// The state of a whole game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    /// The master seed (ADR-0017 point 5: server-only, never in an observation).
    seed: u64,
    players: Vec<PlayerState>,
    /// The day's schedule: turn groups in order, each the players who act together
    /// (ADR-0017 point 3). Sequential play, the only schedule of increment 1, is one player per
    /// group in player order.
    groups: Vec<Vec<PlayerId>>,
    /// The group whose turn is running.
    group: usize,
    /// The day, from 1.
    day: u32,
    /// The game's own stream (the day advance; ADR-0017 point 4).
    rng: Rng,
    adventure: AdventureState,
}

impl GameState {
    /// Builds a game from its setup and master seed: the setup's checks
    /// ([`GameSetup::validate`]), its random placeholders resolved from the setup stream (the
    /// starting armies, GAM-SET-001, then the map objects' contents, ADR-0016 point 4), a stream
    /// per player, sequential play (GAM-TRN-001) on day 1
    /// (GAM-CAL-001), and the first group's turn started, so the first day starts as every later
    /// one (ADR-0017 point 3; no context has a rule at the start of a day yet).
    ///
    /// # Errors
    ///
    /// The first [`GameSetupError`] found, in the order of [`GameSetup::validate`].
    pub fn new(rules: &GameRuleset, setup: &GameSetup, seed: u64) -> Result<Self, GameSetupError> {
        let mut draws = Rng::for_stream(seed, StreamId::SETUP);
        let part = setup.adventure_setup(rules, &mut draws)?;
        let adventure = AdventureState::new(rules.core(), rules.adventure(), &part, &mut draws)
            .map_err(GameSetupError::Adventure)?;
        let players: Vec<PlayerState> = setup
            .players
            .iter()
            .zip(0..=u8::MAX)
            .map(|(player, index)| PlayerState {
                colour: player.colour,
                rng: Rng::for_stream(seed, StreamId::player(index)),
                acting: false,
            })
            .collect();
        // rule: GAM-TRN-001
        let groups = (0..=u8::MAX)
            .take(players.len())
            .map(|index| alloc::vec![PlayerId::new(index)])
            .collect();
        let mut state = Self {
            seed,
            players,
            groups,
            group: 0,
            day: 1,
            rng: Rng::for_stream(seed, StreamId::GAME),
            adventure,
        };
        state.start_group(rules, &mut NoopSink);
        Ok(state)
    }

    /// Ends `player`'s turn (`EndTurn`). When every player of the group has ended its turn the
    /// next group's turn starts; after the last group the day advances (GAM-TRN-002) and the
    /// first group's turn starts. A rejected command changes nothing.
    pub(crate) fn end_turn<S: Sink>(
        &mut self,
        rules: &GameRuleset,
        player: PlayerId,
        sink: &mut S,
    ) -> Result<(), GameCommandError> {
        self.check_acting(player)?;
        self.players[player.index()].acting = false;
        sink.emit(GameEvent::TurnEnded { player });
        if self.players.iter().any(|p| p.acting) {
            return Ok(());
        }
        self.group += 1;
        if self.group == self.groups.len() {
            self.group = 0;
            self.advance_day(sink);
        }
        self.start_group(rules, sink);
        Ok(())
    }

    /// Checks that `player` is a player of the game whose turn is running (the game's part of
    /// every player command, ADR-0017 point 1).
    pub(crate) fn check_acting(&self, player: PlayerId) -> Result<(), GameCommandError> {
        let state = self
            .players
            .get(player.index())
            .ok_or(GameCommandError::UnknownPlayer(player))?;
        if !state.acting {
            return Err(GameCommandError::NotYourTurn(player));
        }
        Ok(())
    }

    /// The day advance (ADR-0017 point 3): the calendar, then each context's start-of-day step
    /// (none has a rule there yet).
    // rule: GAM-TRN-002, GAM-CAL-001
    fn advance_day<S: Sink>(&mut self, sink: &mut S) {
        self.day = self.day.saturating_add(1);
        sink.emit(GameEvent::DayStarted { day: self.day });
    }

    /// Starts the turn of the current group: each of its players acts, and each context runs its
    /// start-of-turn step for it (the adventure's movement points, ADV-MOV-002).
    fn start_group<S: Sink>(&mut self, rules: &GameRuleset, sink: &mut S) {
        let group = self.groups[self.group].clone();
        for player in group {
            self.players[player.index()].acting = true;
            self.adventure
                .start_turn(rules.core(), rules.adventure(), player);
            sink.emit(GameEvent::TurnStarted { player });
        }
    }

    /// The day, from 1.
    pub const fn day(&self) -> u32 {
        self.day
    }

    /// How many players the game has.
    pub fn players(&self) -> usize {
        self.players.len()
    }

    /// A player's colour, if the player exists.
    pub fn colour(&self, player: PlayerId) -> Option<u8> {
        self.players.get(player.index()).map(|p| p.colour)
    }

    /// The players of the group whose turn is running.
    pub fn turn_group(&self) -> &[PlayerId] {
        &self.groups[self.group]
    }

    /// Whether `player`'s turn is running: its group's turn has started and it has not ended its
    /// turn.
    pub fn is_acting(&self, player: PlayerId) -> bool {
        self.players.get(player.index()).is_some_and(|p| p.acting)
    }

    /// The adventure context's state.
    pub const fn adventure(&self) -> &AdventureState {
        &self.adventure
    }

    /// The adventure context's state, for the commands routed to it.
    pub(crate) const fn adventure_mut(&mut self) -> &mut AdventureState {
        &mut self.adventure
    }

    /// The canonical state hash (ADR-0003 point 12): content only. It covers the seed and the
    /// streams, so it never reaches a client (ADR-0017 point 5).
    pub fn state_hash(&self) -> u64 {
        hash_of(self)
    }
}

impl CanonicalHash for GameState {
    /// Encoding v1 (E08 T6b): version; the master seed; the day; the running group's index; the
    /// schedule (groups as length + player ids); the players (colour, acting, stream state); the
    /// game's stream; the adventure state's hash.
    fn hash_into(&self, h: &mut StateHasher) {
        h.write_u32(GAME_STATE_HASH_VERSION);
        h.write_u64(self.seed);
        h.write_u32(self.day);
        h.write_len(self.group);
        h.write_len(self.groups.len());
        for group in &self.groups {
            h.write_len(group.len());
            for player in group {
                h.write_u8(player.raw());
            }
        }
        h.write_len(self.players.len());
        for player in &self.players {
            h.write_u8(player.colour);
            h.write_bool(player.acting);
            player.rng.hash_into(h);
        }
        self.rng.hash_into(h);
        h.write_u64(self.adventure.state_hash());
    }
}

#[cfg(test)]
impl GameState {
    /// Replaces the schedule (tests of turn groups larger than sequential play) and restarts the
    /// first group's turn.
    pub(crate) fn set_schedule(&mut self, rules: &GameRuleset, groups: Vec<Vec<PlayerId>>) {
        for player in &mut self.players {
            player.acting = false;
        }
        self.groups = groups;
        self.group = 0;
        self.start_group(rules, &mut NoopSink);
    }

    /// The master seed.
    pub(crate) const fn seed(&self) -> u64 {
        self.seed
    }
}
