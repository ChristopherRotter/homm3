//! The whole game's entry point (ADR-0017 point 1, ADR-0004): actors, the game's commands, its
//! events and their sink. A game command wraps a context's command or is the game's own; the game
//! checks turn and phase, the context its rules (E08 T6b: `EndTurn`; T7b: the adventure's
//! `MoveHero`; `observe` and `project` are in `observe.rs`, T8b1; `legal_commands` in
//! `legal.rs`, T8b2a; T14b: the pending decisions and their answers, in `state.rs`; T19a: battles,
//! in `state/battle.rs`).

use core::fmt;

use h3_adventure::{
    AdventureCommand, AdventureCommandError, AdventureEvent, ObjectTypeId, Pos, Viewers,
};
use h3_content::{CreatureTypeId, ResourceId};
use h3_contracts::{BattleResult, HeroId, PlayerId};
use h3_heroes::HeroChange;

use crate::{GameRuleset, GameState};

/// Who issues a command (ADR-0017 point 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Actor {
    /// A player, human or bot: the identity the session layer vouches for, never one taken from
    /// the command's payload.
    Player(PlayerId),
    /// The trusted layer that hosts the game (the server's session layer, the env), never a
    /// player (E08 T20b2): it plays a battle's unowned side (a wandering monster's) with a policy
    /// of its own, and commands no owned side. Its other commands (ending a player's turn,
    /// answering a decision with its default, removing a player) come with the server lane's M5.
    System,
}

/// A command to the whole game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameCommand {
    /// Ends the actor's turn.
    EndTurn,
    /// A command to the adventure context, given in the actor's turn.
    Adventure(AdventureCommand),
    /// Answers the actor's first open decision (ADR-0017 point 6, GAM-DEC-001; E08 T14b), in its
    /// turn or not.
    Answer(Answer),
    /// A command to the battle of the actor's first open decision, for the side it owns while that
    /// side is to act, in its turn or not (ADR-0017 point 6, GAM-BAT-001; E08 T19a).
    Battle(h3_battle::Command),
}

/// An answer to a pending decision (ADR-0017 point 6): increments 2's and 3's decisions are
/// offers, answered yes or no; later kinds add variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    /// Yes: the offer's price is paid and the offer carried out.
    Accept,
    /// No: the decision closes; nothing else changes but a wandering monster's answer, which
    /// goes on (its next offer or a fight after a declined join, its leaving after a declined
    /// flight; ADV-MON-007, ADV-MON-008). A later visit asks again.
    Decline,
}

/// Why a command was rejected; a rejected command changes nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameCommandError {
    /// The actor is not a player of the game.
    UnknownPlayer(PlayerId),
    /// The actor's turn is not running.
    NotYourTurn(PlayerId),
    /// The actor has an open decision: its only legal commands are the answers (GAM-DEC-001).
    DecisionOpen,
    /// An answer, but the actor has no open decision.
    NoDecision,
    /// An acceptance of an offer that no longer stands (ADR-0017 point 10): a border guard
    /// another player passed has left the map (E08 T15a); declining is the only legal answer.
    OfferGone,
    /// The adventure context rejected the command.
    Adventure(AdventureCommandError),
    /// An answer, but the actor's first open decision is a battle: its commands are the actor's
    /// only legal ones (E08 T19a).
    BattleOpen,
    /// A battle command, but the actor's first open decision is no battle (E08 T19a).
    NoBattle,
    /// A battle command, but the side to act is not the actor's (E08 T19a).
    NotYourSide,
    /// The battle rejected the command.
    Battle(h3_battle::CommandError),
    /// A command the system actor does not issue: its commands are an unowned side's battle
    /// commands (E08 T20b2).
    NotForSystem,
}

impl fmt::Display for GameCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownPlayer(p) => write!(f, "player {} is not in the game", p.raw()),
            Self::NotYourTurn(p) => write!(f, "it is not player {}'s turn", p.raw()),
            Self::DecisionOpen => f.write_str("an open decision waits for its answer"),
            Self::NoDecision => f.write_str("no decision is open"),
            Self::OfferGone => f.write_str("the offer no longer stands"),
            Self::Adventure(error) => write!(f, "{error}"),
            Self::BattleOpen => f.write_str("an open battle waits for its commands"),
            Self::NoBattle => f.write_str("no battle is open"),
            Self::NotYourSide => f.write_str("the side to act is not the player's"),
            Self::Battle(error) => write!(f, "{error}"),
            Self::NotForSystem => f.write_str("the system actor issues only an unowned side's commands"),
        }
    }
}

impl core::error::Error for GameCommandError {}

/// What happened in the game. The game's own events are public facts (whose turn it is and the
/// day) but a treasury's change and an offer's, which their owner alone sees (E08 T13b, T14b); an
/// adventure event carries the players who may see it (E08 T8a). An offer is named as the
/// observation names its object, by type and anchor, never by index (ADR-0016 point 6).
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
    /// A player's treasury gained a resource (GAM-RES-001): one event per resource of a gain,
    /// in the gain's order, after the context's events; or a surrender's price, which the other
    /// side's owner receives (GAM-BAT-005, E08 T19a).
    ResourceGained {
        /// The player.
        player: PlayerId,
        /// The resource.
        resource: ResourceId,
        /// The amount the treasury actually gained (a gain saturates at 2³¹ − 1).
        amount: u32,
    },
    /// A visit's offer opened as a pending decision of its player (GAM-DEC-001, ADV-VIS-001):
    /// its treasury can pay the price, which the object's type gives.
    OfferOpened {
        /// The player.
        player: PlayerId,
        /// The hero whose visit made the offer.
        hero: HeroId,
        /// The object's type.
        object_type: ObjectTypeId,
        /// The object's anchor.
        anchor: Pos,
    },
    /// A visit's offer did not open: its player's treasury cannot pay the price (ADR-0017 point
    /// 10; ADV-VIS-001, the original's message).
    OfferRefused {
        /// The player.
        player: PlayerId,
        /// The hero whose visit made the offer.
        hero: HeroId,
        /// The object's type.
        object_type: ObjectTypeId,
        /// The object's anchor.
        anchor: Pos,
    },
    /// A player answered its open decision, which closed (GAM-DEC-001). An accepted offer's
    /// payment and effect follow.
    OfferAnswered {
        /// The player.
        player: PlayerId,
        /// The object's type.
        object_type: ObjectTypeId,
        /// The object's anchor.
        anchor: Pos,
        /// Whether the player accepted.
        accepted: bool,
    },
    /// A battle on the map started (GAM-BAT-001, E08 T19a): its heroes by side (attacker first)
    /// and the tile it is fought at. Seen by its owners.
    BattleStarted {
        /// The heroes leading the sides, by side index.
        heroes: [Option<HeroId>; 2],
        /// The tile the battle is fought at: the defender's.
        tile: Pos,
        /// Its owners (the viewer alone in a projection).
        seen_by: Viewers,
    },
    /// Something happened in a battle on the map (E08 T19a): a public event, seen by both owners;
    /// a side's, by that side's owner.
    Battle {
        /// The battle's event.
        event: h3_battle::Event,
        /// Who sees it (the viewer alone in a projection).
        seen_by: Viewers,
    },
    /// A battle on the map ended (GAM-BAT-001, E08 T19a): its heroes by side and the result; its
    /// outcome's effects follow. Seen by its owners.
    BattleEnded {
        /// The heroes that led the sides, by side index.
        heroes: [Option<HeroId>; 2],
        /// How it ended.
        result: BattleResult,
        /// Its owners (the viewer alone in a projection).
        seen_by: Viewers,
    },
    /// A hero's details changed (E08 T19a2a): a battle's outcome, one event per change in the
    /// order applied (GAM-BAT-002, GAM-BAT-004). Seen by the hero's owner.
    HeroChanged {
        /// The hero's owner.
        player: PlayerId,
        /// The hero.
        hero: HeroId,
        /// The change, as `h3_heroes` applied it.
        change: HeroChange,
    },
    /// A hero's Necromancy raised creatures into its army after its side won a battle
    /// (GAM-BAT-006, E08 T19a2b): those that joined, the upgrade at two thirds when the army was
    /// full. Seen by the hero's owner.
    CreaturesRaised {
        /// The hero's owner.
        player: PlayerId,
        /// The hero.
        hero: HeroId,
        /// The creature that joined.
        creature: CreatureTypeId,
        /// How many joined.
        count: u32,
    },
    /// A player's treasury paid a resource of an accepted offer's price (GAM-RES-001): one event
    /// per resource of the price, in its order; or a surrender's price (GAM-BAT-005, E08 T19a).
    ResourcePaid {
        /// The player.
        player: PlayerId,
        /// The resource.
        resource: ResourceId,
        /// The amount paid.
        amount: u32,
    },
}

/// Receiver of the game's events.
pub trait Sink {
    /// Whether the sink keeps what it receives, passed to a battle's sink (`h3_battle::Sink`'s
    /// own: work done only to report something, a walker's path hex by hex, is skipped for a sink
    /// that does not; the state is the same either way; E08 T19b1).
    const RECORDS: bool = true;

    /// Receives one event.
    fn emit(&mut self, event: GameEvent);
}

/// A sink that drops every event.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopSink;

impl Sink for NoopSink {
    const RECORDS: bool = false;

    #[inline]
    fn emit(&mut self, _event: GameEvent) {}
}

impl Sink for alloc::vec::Vec<GameEvent> {
    fn emit(&mut self, event: GameEvent) {
        self.push(event);
    }
}

/// The adventure context's sink inside a game's: its events become game events.
pub(crate) struct AdventureSink<'a, S>(pub(crate) &'a mut S);

impl<S: Sink> h3_adventure::Sink for AdventureSink<'_, S> {
    fn emit(&mut self, event: AdventureEvent) {
        self.0.emit(GameEvent::Adventure(event));
    }
}

/// Applies a command of `actor` to the game (ADR-0017 point 1): validated against the state and
/// the trusted actor, then applied; the events go to `sink`. The system actor issues only an
/// unowned side's battle commands ([`Actor::System`], E08 T20b2).
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
    let (player, command) = match (actor, command) {
        (Actor::System, GameCommand::Battle(command)) => {
            return state.system_battle_command(rules, command, sink);
        }
        (Actor::System, _) => return Err(GameCommandError::NotForSystem),
        (Actor::Player(player), command) => (player, command),
    };
    match command {
        GameCommand::EndTurn => state.end_turn(rules, player, sink),
        GameCommand::Answer(answer) => state.answer(rules, player, answer, sink),
        GameCommand::Battle(command) => state.battle_command(rules, player, command, sink),
        GameCommand::Adventure(command) => {
            state.check_ready(player)?;
            let (adventure, heroes) = state.adventure_with_heroes();
            let outcome = adventure
                .apply(
                    rules.core(),
                    rules.adventure(),
                    heroes,
                    player,
                    &command,
                    &mut AdventureSink(sink),
                )
                .map_err(GameCommandError::Adventure)?;
            state.settle_outcome(rules, player, outcome, sink);
            Ok(())
        }
    }
}
