//! What a viewer may know of the whole game (ADR-0017 points 1 and 2, E08 T8b1;
//! `docs/06-security.md`, "Information hiding"): the game's public facts (the day, the players'
//! colours, whose turn is running), the viewer's own treasury (E08 T13a; other players' never
//! show) and each context's observation for the viewer, and the
//! projection of the game's events. The seed, the streams and the state hash never show
//! (ADR-0017 point 5).

use alloc::vec::Vec;

use h3_adventure::AdventureObservation;
use h3_contracts::PlayerId;

use crate::command::GameEvent;
use crate::{GameRuleset, GameState};

/// Who an observation or an event's projection is for (ADR-0017 point 2). The spectator, who
/// sees public facts only, comes with the server lane as an additive variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Viewer {
    /// A player: what it may know.
    Player(PlayerId),
}

/// What one viewer may know of a game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameObservation {
    /// The player this observation is for.
    pub viewer: PlayerId,
    /// The day, from 1.
    pub day: u32,
    /// The players, by id.
    pub players: Vec<PlayerView>,
    /// The players of the group whose turn is running.
    pub turn_group: Vec<PlayerId>,
    /// The viewer's treasury, by resource in the core's order (GAM-RES-001); empty for a viewer
    /// that is not a player of the game.
    pub treasury: Vec<u32>,
    /// The adventure map as the viewer sees it.
    pub adventure: AdventureObservation,
}

/// A player as every viewer sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerView {
    /// Its colour (0 red to 7 pink).
    pub colour: u8,
    /// Whether its turn is running: its group's turn has started and it has not ended its turn.
    pub acting: bool,
}

/// What `viewer` may know of the game: the public facts and the adventure map as it sees it
/// (ADV-FOG-004 to ADV-FOG-006). A viewer that is not a player of the game sees the public facts
/// and no tile.
pub fn observe(state: &GameState, rules: &GameRuleset, viewer: Viewer) -> GameObservation {
    let Viewer::Player(player) = viewer;
    GameObservation {
        viewer: player,
        day: state.day(),
        players: (0..=u8::MAX)
            .take(state.players())
            .map(PlayerId::new)
            .map(|p| PlayerView {
                colour: state.colour(p).unwrap_or(u8::MAX),
                acting: state.is_acting(p),
            })
            .collect(),
        turn_group: state.turn_group().to_vec(),
        treasury: state
            .treasury(player)
            .map_or_else(Vec::new, <[u32]>::to_vec),
        adventure: state.adventure().observe(rules.core(), player),
    }
}

/// The event as `viewer` may see it, or `None` if it is hidden from it: the game's own events are
/// public but a treasury's change, which shows to its owner only; an adventure event shows to the
/// players who see it, naming the viewer alone among them
/// ([`h3_adventure::AdventureEvent::project`]).
pub fn project(event: &GameEvent, viewer: Viewer) -> Option<GameEvent> {
    let Viewer::Player(player) = viewer;
    match event {
        GameEvent::Adventure(event) => event.project(player).map(GameEvent::Adventure),
        GameEvent::ResourceGained { player: owner, .. } => (*owner == player).then_some(*event),
        GameEvent::TurnEnded { .. }
        | GameEvent::DayStarted { .. }
        | GameEvent::TurnStarted { .. } => Some(*event),
    }
}
