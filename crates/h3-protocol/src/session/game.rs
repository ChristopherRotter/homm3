//! The whole game's session (E16 M5 T5a; SRV-Q25; ADR-0018 points 6 and 7 for the whole game):
//! one game hosted in-process, its players as seats and viewers, the authoritative loop over
//! `h3_game::apply`, sans I/O. The host passes in each player's command with the player it
//! vouches for ([`GameSession::play`]) and the system actor's ([`GameSession::system`]), and
//! delivers the frames that come out ([`GameOutbound`]); the session never sees a socket, a clock
//! or a file.
//!
//! - **Seats.** Every player of the game is a seat of the host's: in-process the host plays them
//!   all (a replay's commands, a test's); network seats, computer seats, turn timers and the
//!   level-up hold's timer are part 2's (the epic). A command is applied with the player the host
//!   names, never with anything the client wrote (ADR-0018 point 6): the wire's commands name no
//!   player.
//! - **The system actor** (`h3_game::Actor::System`, E08 T20b2): its commands come from the host
//!   ([`GameSession::system`]; its legal ones [`GameSession::system_legal`]): an unowned battle
//!   side's, a wandering monster's. No player is told its acceptance or its rejection; its events
//!   are told as every command's.
//! - **Viewers.** Each player is a viewer with its own message numbers `n`; everything it gets is
//!   projected for it by construction ([`crate::map::game`]). When the game starts it gets its
//!   `GameSnapshot`. After every accepted command: the player that sent it `GameAccepted`
//!   (echoed), then every viewer `GameEvents` with the command's events as it may see them (also
//!   an empty list, so that what a viewer receives does not depend on what it may not see) and a
//!   battle's own `Snapshot` when one opens for it (SRV-Q27 (1): its events are then the battle
//!   client's, inside `GameEvents`), then every player that may act its `GameTurn`. A rejected
//!   command changes nothing and is told to its player alone (`GameRejected`).
//! - **Not here yet:** the journal, the restore and each viewer's record (T5b, record format v4);
//!   what each viewer newly sees on the tiles it explored (T5c, SRV-Q26; `SRV-STUB-1`); a battle's
//!   last snapshot when it ends (`SRV-STUB-14`); a hero's moves, which `GameTurn` leaves out as
//!   `h3_game::legal_commands` does (`h3_game::legal_moves`; a query with part 2's network seats).
//!
//! Provisional (E16 M5 T5a; the epic's review list): the names, the start as the constructor, the
//! message order, a `GameTurn` after every accepted command to every player that may act, a
//! system command told to nobody, a player outside the game told nothing.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use h3_contracts::PlayerId;
use h3_game::{Actor, GameRuleset, GameSetup, GameSetupError, GameState, Viewer as EngineViewer};

use crate::codec;
use crate::map;
use crate::wire::ServerMessage;
use crate::wire::game::{GameCommand, GameEvent, GameManifest, GameReject};

/// A frame for a player: the host delivers it to the player's client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameOutbound {
    /// The player.
    pub to: PlayerId,
    /// The frame's text: one message.
    pub frame: String,
}

/// One viewer of the game: a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Viewer {
    /// The number of the last message sent.
    n: u32,
    /// Whether its view holds a battle: one whose snapshot it was sent, its end not told since.
    battle: bool,
}

impl Viewer {
    /// Sends the viewer `to` its next numbered message.
    fn post(
        &mut self,
        to: PlayerId,
        message: impl FnOnce(u32) -> ServerMessage,
        out: &mut Vec<GameOutbound>,
    ) {
        self.n = self.n.saturating_add(1);
        out.push(GameOutbound {
            to,
            frame: codec::encode_text(&message(self.n)),
        });
    }
}

/// One whole game hosted in-process: its state, its manifest and its players as viewers.
#[derive(Debug)]
pub struct GameSession<'r> {
    rules: &'r GameRuleset,
    state: GameState,
    manifest: GameManifest,
    /// By player index.
    viewers: Vec<Viewer>,
}

impl<'r> GameSession<'r> {
    /// Starts the game of `setup` with `seed` under `rules` (`h3_game::GameState::new`), made of
    /// what `manifest` names (the caller's: [`crate::map::game::manifest`]): every player gets its
    /// `GameSnapshot`, then each that may act its `GameTurn`.
    ///
    /// # Errors
    ///
    /// The engine's when the setup starts no game; nothing was sent then.
    pub fn start(
        rules: &'r GameRuleset,
        setup: &GameSetup,
        seed: u64,
        manifest: GameManifest,
        out: &mut Vec<GameOutbound>,
    ) -> Result<Self, GameSetupError> {
        let state = GameState::new(rules, setup, seed)?;
        let mut session = Self {
            rules,
            viewers: vec![Viewer::default(); state.players()],
            state,
            manifest,
        };
        for (player, viewer) in players().zip(&mut session.viewers) {
            let observation =
                map::game::observe(&session.state, rules, EngineViewer::Player(player));
            viewer.battle = observation.battle.is_some();
            viewer.post(
                player,
                |n| ServerMessage::GameSnapshot { n, observation },
                out,
            );
        }
        session.turns(out);
        Ok(session)
    }

    /// `player`'s command `command`, which the host vouches came from it, numbered `seq` by its
    /// client: applied as the player's (`Actor::Player`), then told (the module's order). A player
    /// outside the game is told nothing.
    ///
    /// # Errors
    ///
    /// Why the game rejected it (also told to the player); nothing changed.
    pub fn play(
        &mut self,
        player: PlayerId,
        seq: u32,
        command: GameCommand,
        out: &mut Vec<GameOutbound>,
    ) -> Result<(), GameReject> {
        match self.apply(Actor::Player(player), command) {
            Err(code) => {
                if let Some(viewer) = self.viewers.get_mut(player.index()) {
                    viewer.post(
                        player,
                        |n| ServerMessage::GameRejected { n, seq, code },
                        out,
                    );
                }
                Err(code)
            }
            Ok((command, events)) => {
                if let Some(viewer) = self.viewers.get_mut(player.index()) {
                    viewer.post(
                        player,
                        |n| ServerMessage::GameAccepted { n, seq, command },
                        out,
                    );
                }
                self.tell(&events, out);
                Ok(())
            }
        }
    }

    /// The system actor's command (`Actor::System`), from the host: applied, then its events and
    /// the turns told as a player's command's are.
    ///
    /// # Errors
    ///
    /// Why the game rejected it (told to nobody); nothing changed.
    pub fn system(
        &mut self,
        command: GameCommand,
        out: &mut Vec<GameOutbound>,
    ) -> Result<(), GameReject> {
        let (_, events) = self.apply(Actor::System, command)?;
        self.tell(&events, out);
        Ok(())
    }

    /// The system actor's legal commands now (`h3_game::legal_commands`): an unowned battle
    /// side's while it is to act, else none.
    pub fn system_legal(&self) -> Vec<GameCommand> {
        h3_game::legal_commands(&self.state, self.rules, Actor::System)
            .iter()
            .map(map::game::command)
            .collect()
    }

    /// The game's state (the host's: the journal, the hashes).
    pub const fn state(&self) -> &GameState {
        &self.state
    }

    /// What the game is made of.
    pub const fn manifest(&self) -> &GameManifest {
        &self.manifest
    }

    /// `command` of `actor` mapped to the engine's and applied: the command as applied, on the
    /// wire, and its events.
    fn apply(
        &mut self,
        actor: Actor,
        command: GameCommand,
    ) -> Result<(GameCommand, Vec<h3_game::GameEvent>), GameReject> {
        let command = map::game::command_to_engine(command)?;
        let echo = map::game::command(&command);
        let mut events = Vec::new();
        h3_game::apply(&mut self.state, self.rules, actor, command, &mut events)
            .map_err(map::game::reject)?;
        Ok((echo, events))
    }

    /// Every viewer gets `events` as it may see them and a battle's snapshot when one opens for
    /// it; then each player that may act its turn.
    fn tell(&mut self, events: &[h3_game::GameEvent], out: &mut Vec<GameOutbound>) {
        for (player, viewer) in players().zip(&mut self.viewers) {
            let events = map::game::events(events, EngineViewer::Player(player));
            let started = events
                .iter()
                .any(|e| matches!(e, GameEvent::BattleStarted { .. }));
            // The fold ends a view's battle at its end (`GameObservation::fold`), without the
            // battle's last snapshot, which h3_game no longer holds (stub: SRV-STUB-14).
            let held = viewer.battle
                && !events
                    .iter()
                    .any(|e| matches!(e, GameEvent::BattleEnded { .. }));
            viewer.post(player, |n| ServerMessage::GameEvents { n, events }, out);
            let open = self
                .state
                .battle(player)
                .and_then(|battle| Some((battle, battle.side_of(player)?)));
            if let Some((battle, side)) = open
                && (started || !held)
            {
                let observation = map::observe(battle.state(), self.rules.battle(), side);
                viewer.post(player, |n| ServerMessage::Snapshot { n, observation }, out);
            }
            viewer.battle = open.is_some();
        }
        self.turns(out);
    }

    /// Each player that may act gets its legal commands and, while its battle side is to act,
    /// the spells its hero may cast.
    fn turns(&mut self, out: &mut Vec<GameOutbound>) {
        for (player, viewer) in players().zip(&mut self.viewers) {
            let legal: Vec<GameCommand> =
                h3_game::legal_commands(&self.state, self.rules, Actor::Player(player))
                    .iter()
                    .map(map::game::command)
                    .collect();
            if legal.is_empty() {
                continue;
            }
            let castable = self
                .state
                .battle(player)
                .and_then(|battle| {
                    let side = battle.side_of(player)?;
                    (battle.state().side_to_act() == Some(side))
                        .then(|| map::castable(battle.state(), self.rules.battle(), side))
                })
                .unwrap_or_default();
            viewer.post(
                player,
                |n| ServerMessage::GameTurn { n, legal, castable },
                out,
            );
        }
    }
}

/// The players by index, as many as the game has (at most 256: `PlayerId`'s range).
fn players() -> impl Iterator<Item = PlayerId> {
    (0..=u8::MAX).map(PlayerId::new)
}
