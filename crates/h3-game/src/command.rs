//! The whole game's entry point (ADR-0017 point 1, ADR-0004): actors, the game's commands, its
//! events and their sink. A game command wraps a context's command or is the game's own; the game
//! checks turn and phase, the context its rules (E08 T6b: `EndTurn`; T7b: the adventure's
//! `MoveHero`; `observe` and `project` are in `observe.rs`, T8b1; `legal_commands` in
//! `legal.rs`, T8b2a).

use core::fmt;

use h3_adventure::{AdventureCommand, AdventureCommandError, AdventureEvent};
use h3_contracts::PlayerId;

use crate::{GameRuleset, GameState};

/// Who issues a command (ADR-0017 point 2). The server's trusted system actor comes with the
/// server lane's M5 as an additive variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Actor {
    /// A player, human or bot: the identity the session layer vouches for, never one taken from
    /// the command's payload.
    Player(PlayerId),
}

/// A command to the whole game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameCommand {
    /// Ends the actor's turn.
    EndTurn,
    /// A command to the adventure context, given in the actor's turn.
    Adventure(AdventureCommand),
}

/// Why a command was rejected; a rejected command changes nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameCommandError {
    /// The actor is not a player of the game.
    UnknownPlayer(PlayerId),
    /// The actor's turn is not running.
    NotYourTurn(PlayerId),
    /// The adventure context rejected the command.
    Adventure(AdventureCommandError),
}

impl fmt::Display for GameCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownPlayer(p) => write!(f, "player {} is not in the game", p.raw()),
            Self::NotYourTurn(p) => write!(f, "it is not player {}'s turn", p.raw()),
            Self::Adventure(error) => write!(f, "{error}"),
        }
    }
}

impl core::error::Error for GameCommandError {}

/// What happened in the game. The game's own events are public facts: whose turn it is and the
/// day; an adventure event carries the players who may see it (E08 T8a).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameEvent {
    /// A player ended its turn.
    TurnEnded {
        /// The player.
        player: PlayerId,
    },
    /// A new day started.
    DayStarted {
        /// The day, from 1.
        day: u32,
    },
    /// A player's turn started.
    TurnStarted {
        /// The player.
        player: PlayerId,
    },
    /// Something happened on the adventure map.
    Adventure(AdventureEvent),
}

/// Receiver of the game's events.
pub trait Sink {
    /// Receives one event.
    fn emit(&mut self, event: GameEvent);
}

/// A sink that drops every event.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopSink;

impl Sink for NoopSink {
    #[inline]
    fn emit(&mut self, _event: GameEvent) {}
}

impl Sink for alloc::vec::Vec<GameEvent> {
    fn emit(&mut self, event: GameEvent) {
        self.push(event);
    }
}

/// The adventure context's sink inside a game's: its events become game events.
struct AdventureSink<'a, S>(&'a mut S);

impl<S: Sink> h3_adventure::Sink for AdventureSink<'_, S> {
    fn emit(&mut self, event: AdventureEvent) {
        self.0.emit(GameEvent::Adventure(event));
    }
}

/// Applies a command of `actor` to the game (ADR-0017 point 1): validated against the state and
/// the trusted actor, then applied; the events go to `sink`.
///
/// # Errors
///
/// A [`GameCommandError`]; the state is then unchanged and no event was emitted.
pub fn apply<S: Sink>(
    state: &mut GameState,
    rules: &GameRuleset,
    actor: Actor,
    command: GameCommand,
    sink: &mut S,
) -> Result<(), GameCommandError> {
    let Actor::Player(player) = actor;
    match command {
        GameCommand::EndTurn => state.end_turn(rules, player, sink),
        GameCommand::Adventure(command) => {
            state.check_acting(player)?;
            state
                .adventure_mut()
                .apply(
                    rules.core(),
                    rules.adventure(),
                    player,
                    &command,
                    &mut AdventureSink(sink),
                )
                .map_err(GameCommandError::Adventure)
        }
    }
}
