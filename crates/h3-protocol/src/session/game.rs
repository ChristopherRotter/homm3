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
//!   an empty list, so that what a viewer receives does not depend on what it may not see), its
//!   battle as its side saw it when it ended (`GameLastSnapshot`, E16 M5 T5c: the session's sink
//!   keeps each owner's last view of a battle the command ends, `h3_game::Sink::battle_ended`;
//!   SRV-Q27 (1)), its `GameReveal` when the command explored tiles for it (E16 M5 T5c, SRV-Q26 (a): the session
//!   keeps each viewer's explored grid as it was last told it and asks `h3_game::reveal` when the
//!   grid's count changed) and a battle's own `Snapshot` when one opens for it (SRV-Q27 (1): its
//!   events are then the battle client's, inside `GameEvents`), then every player that may act
//!   its `GameTurn`. A rejected command changes nothing and is told to its player alone
//!   (`GameRejected`).
//! - **The journal** (E16 M5 T5b). The accepted commands with their actors and a state hash every
//!   `checkpoint_every` commands: the adventure replay in progress ([`GameJournal`]; its header,
//!   map and difficulty are the host's). What a restart must redo comes out as [`GameEntry`]s,
//!   which the host drains after every call ([`GameSession::drain_entries`]) and makes durable
//!   before it delivers that call's frames. [`GameSession::restore`] re-runs them: the same game,
//!   every player then told it afresh as at the start (its numbers and record begin again: they
//!   are not journaled; a rejection changes nothing and is not journaled either).
//! - **Records** (E16 M5 T5b, record format v4). Each player's record is the log of every
//!   numbered message it was sent, within the host's bound ([`GameOptions::record_bytes`]: the
//!   oldest lines go first): [`GameSession::record`] gives it with its header line
//!   ([`GameRecordHeader`]: the game's manifest and the player), as a record file holds it.
//! - **Not here yet:** a hero's moves, which
//!   `GameTurn` leaves out as `h3_game::legal_commands` does (`h3_game::legal_moves`; a query
//!   with part 2's network seats); the journal's file and the hosting over the network (part 2).
//!
//! Provisional (E16 M5 T5a, T5b; the epic's review list): the names, the start as the constructor,
//! the message order, a `GameTurn` after every accepted command to every player that may act, a
//! system command told to nobody, a player outside the game told nothing; the entries, a restore
//! telling every player its snapshot and turn afresh, the record's header with the game's id; a
//! reveal as a message of its own after the events, a battle's last view as one of its own
//! before it (T5c).

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use h3_battle::BattleSide;
use h3_contracts::PlayerId;
use h3_game::Sink;
use h3_game::{
    Actor, BitGrid, GameCommandError, GameRuleset, GameSetup, GameSetupError, GameState,
    OpenBattle, Viewer as EngineViewer,
};

use super::Record;
use crate::RECORD_FORMAT_VERSION;
use crate::codec;
use crate::map;
use crate::wire::game::{GameCommand, GameEvent, GameManifest, GameRecordHeader, GameReject};
use crate::wire::{Observation, ServerMessage};

/// A frame for a player: the host delivers it to the player's client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameOutbound {
    /// The player.
    pub to: PlayerId,
    /// The frame's text: one message.
    pub frame: String,
}

/// How the host hosts a game (E16 M5 T5b).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameOptions {
    /// The game's id, as its records name it.
    pub game: u32,
    /// A state hash after every this many accepted commands (0: none), as an adventure replay's.
    pub checkpoint_every: u32,
    /// The most a player's record holds, in bytes of its lines (each with its line break): beyond
    /// it the oldest go, the newest stays.
    pub record_bytes: usize,
}

/// The journal (E16 M5 T5b): the adventure replay in progress but for what the host names (the
/// header, the map, the difficulty: `h3_sim::adventure::replay::Replay`'s). Private to the host
/// until the game is over.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameJournal {
    /// The game's seed.
    pub seed: u64,
    /// The accepted commands in order, each with its actor.
    pub commands: Vec<(Actor, h3_game::GameCommand)>,
    /// A state hash after every this many commands (0: none).
    pub checkpoint_every: u32,
    /// The state hashes after commands `k · checkpoint_every`, k = 1, 2, …
    pub checkpoints: Vec<u64>,
}

impl GameJournal {
    /// Appends an accepted command and, when one is due, the state's hash after it; both as
    /// entries to `entries` too.
    fn append(
        &mut self,
        actor: Actor,
        command: h3_game::GameCommand,
        state: &GameState,
        entries: &mut Vec<GameEntry>,
    ) {
        entries.push(GameEntry::Command(actor, command.clone()));
        self.commands.push((actor, command));
        let every = usize::try_from(self.checkpoint_every).unwrap_or(usize::MAX);
        if self.commands.len().checked_rem(every) == Some(0) {
            let hash = state.state_hash();
            self.checkpoints.push(hash);
            entries.push(GameEntry::Checkpoint(hash));
        }
    }
}

/// One entry of the journal the host keeps durable (E16 M5 T5b), in the order the session made
/// them: what a restart redoes ([`GameSession::restore`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameEntry {
    /// The engine accepted the command from the actor.
    Command(Actor, h3_game::GameCommand),
    /// The state's hash after the commands so far, every `checkpoint_every` commands.
    Checkpoint(u64),
}

/// Why a journal does not restore its game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameRestoreError {
    /// The setup starts no game.
    Setup(GameSetupError),
    /// The engine rejected the command of entry `at`.
    Rejected {
        /// The entry's index.
        at: usize,
        /// Why.
        error: GameCommandError,
    },
    /// From entry `at` on, the journal is not what its re-run makes: a checkpoint's hash differs,
    /// or a checkpoint is extra, out of place, or missing before the journal's end.
    Diverged {
        /// The entry's index.
        at: usize,
    },
}

/// What a command told (E16 M5 T5c): its events and, for each battle it ended, each owner's last
/// view of it, as its side saw it as it ended.
struct Told<'r> {
    rules: &'r GameRuleset,
    events: Vec<h3_game::GameEvent>,
    ended: Vec<(PlayerId, Observation)>,
}

impl Sink for Told<'_> {
    fn emit(&mut self, event: h3_game::GameEvent) {
        self.events.push(event);
    }

    fn battle_ended(&mut self, battle: &OpenBattle) {
        let owned = BattleSide::BOTH.into_iter().zip(battle.owners());
        for (side, owner) in owned.filter_map(|(side, owner)| owner.map(|o| (side, o))) {
            let last = map::observe(battle.state(), self.rules.battle(), side);
            self.ended.push((owner, last));
        }
    }
}

/// One viewer of the game: a player.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Viewer {
    /// The number of the last message sent.
    n: u32,
    /// Whether its view holds a battle: one whose snapshot it was sent, its end not told since.
    battle: bool,
    /// The latest numbered messages sent (E16 M5 T5b).
    record: Record,
    /// Its explored tiles as it was last told them, with their count (E16 M5 T5c): the grid
    /// `h3_game::reveal` compares with; none before the game is opened.
    explored: Option<(BitGrid, u32)>,
}

impl Viewer {
    /// Sends the viewer `to` its next numbered message and records it.
    fn post(
        &mut self,
        to: PlayerId,
        message: impl FnOnce(u32) -> ServerMessage,
        out: &mut Vec<GameOutbound>,
    ) {
        self.n = self.n.saturating_add(1);
        let frame = codec::encode_text(&message(self.n));
        self.record.push(frame.clone());
        out.push(GameOutbound { to, frame });
    }
}

/// One whole game hosted in-process: its state, its manifest, its journal and its players as
/// viewers.
#[derive(Debug)]
pub struct GameSession<'r> {
    rules: &'r GameRuleset,
    state: GameState,
    manifest: GameManifest,
    /// The game's id.
    game: u32,
    /// The replay in progress.
    journal: GameJournal,
    /// The entries the host has not drained yet.
    entries: Vec<GameEntry>,
    /// By player index.
    viewers: Vec<Viewer>,
}

impl<'r> GameSession<'r> {
    /// Starts the game of `setup` with `seed` under `rules` (`h3_game::GameState::new`), made of
    /// what `manifest` names (the caller's: [`crate::map::game::manifest`]) and hosted as
    /// `options` say: every player gets its `GameSnapshot`, then each that may act its
    /// `GameTurn`.
    ///
    /// # Errors
    ///
    /// The engine's when the setup starts no game; nothing was sent then.
    pub fn start(
        rules: &'r GameRuleset,
        setup: &GameSetup,
        seed: u64,
        manifest: GameManifest,
        options: GameOptions,
        out: &mut Vec<GameOutbound>,
    ) -> Result<Self, GameSetupError> {
        let mut session = Self::new(rules, setup, seed, manifest, options)?;
        session.open(out);
        Ok(session)
    }

    /// Restores a game from its journal's `entries` after a restart (E16 M5 T5b): builds the game
    /// as [`GameSession::start`] does, re-runs the entries' commands through the engine, each as
    /// its actor's, then tells every player the restored game afresh as `start` does (its
    /// numbers and record begin again). The re-run makes its own entries, which the journal must
    /// begin: a checkpoint whose hash differs, or one out of place, is a divergence. The first
    /// drained entries then are what the journal lacks: the checkpoint after its last command if
    /// a crash cut the journal between the two.
    ///
    /// # Errors
    ///
    /// Why the journal does not restore its game; nothing was sent then.
    pub fn restore(
        rules: &'r GameRuleset,
        setup: &GameSetup,
        seed: u64,
        manifest: GameManifest,
        options: GameOptions,
        entries: &[GameEntry],
        out: &mut Vec<GameOutbound>,
    ) -> Result<Self, GameRestoreError> {
        let mut session =
            Self::new(rules, setup, seed, manifest, options).map_err(GameRestoreError::Setup)?;
        let mut events = Vec::new();
        for (at, entry) in entries.iter().enumerate() {
            match entry {
                GameEntry::Command(actor, command) => {
                    events.clear();
                    h3_game::apply(
                        &mut session.state,
                        rules,
                        *actor,
                        command.clone(),
                        &mut events,
                    )
                    .map_err(|error| GameRestoreError::Rejected { at, error })?;
                    session.journal.append(
                        *actor,
                        command.clone(),
                        &session.state,
                        &mut session.entries,
                    );
                }
                // The re-run's own follows its command; the two are compared below.
                GameEntry::Checkpoint(_) => {}
            }
        }
        if !session.entries.starts_with(entries) {
            let same = entries.iter().zip(&session.entries);
            let at = same
                .take_while(|(recorded, rerun)| recorded == rerun)
                .count();
            return Err(GameRestoreError::Diverged { at });
        }
        session.entries.drain(..entries.len());
        session.open(out);
        Ok(session)
    }

    /// The game of `setup`, nothing told yet.
    fn new(
        rules: &'r GameRuleset,
        setup: &GameSetup,
        seed: u64,
        manifest: GameManifest,
        options: GameOptions,
    ) -> Result<Self, GameSetupError> {
        let state = GameState::new(rules, setup, seed)?;
        let viewer = Viewer {
            n: 0,
            battle: false,
            record: Record::new(options.record_bytes),
            explored: None,
        };
        Ok(Self {
            rules,
            viewers: vec![viewer; state.players()],
            state,
            manifest,
            game: options.game,
            journal: GameJournal {
                seed,
                commands: Vec::new(),
                checkpoint_every: options.checkpoint_every,
                checkpoints: Vec::new(),
            },
            entries: Vec::new(),
        })
    }

    /// Every player gets its `GameSnapshot` of the game as it stands, then each that may act its
    /// `GameTurn`.
    fn open(&mut self, out: &mut Vec<GameOutbound>) {
        for (player, viewer) in players().zip(&mut self.viewers) {
            let observation =
                map::game::observe(&self.state, self.rules, EngineViewer::Player(player));
            viewer.battle = observation.battle.is_some();
            viewer.explored = explored(&self.state, player);
            viewer.post(
                player,
                |n| ServerMessage::GameSnapshot { n, observation },
                out,
            );
        }
        self.turns(out);
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
            Ok((command, told)) => {
                // The engine accepts only a player of the game's command.
                self.viewers[player.index()].post(
                    player,
                    |n| ServerMessage::GameAccepted { n, seq, command },
                    out,
                );
                self.tell(told, out);
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
        let (_, told) = self.apply(Actor::System, command)?;
        self.tell(told, out);
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

    /// The game's state (the host's: the hashes, the end).
    pub const fn state(&self) -> &GameState {
        &self.state
    }

    /// What the game is made of.
    pub const fn manifest(&self) -> &GameManifest {
        &self.manifest
    }

    /// The replay in progress (E16 M5 T5b): the host renders it as the adventure replay.
    pub const fn journal(&self) -> &GameJournal {
        &self.journal
    }

    /// The entries made since the last drain, in order (E16 M5 T5b): the host makes them durable
    /// before it delivers the frames of the call that made them.
    pub fn drain_entries(&mut self) -> vec::Drain<'_, GameEntry> {
        self.entries.drain(..)
    }

    /// `player`'s record (E16 M5 T5b, record format v4): its header line ([`GameRecordHeader`]),
    /// then one line per numbered message it holds, the oldest first; `None` for a player outside
    /// the game.
    pub fn record(&self, player: PlayerId) -> Option<String> {
        let viewer = self.viewers.get(player.index())?;
        let header = GameRecordHeader {
            format: RECORD_FORMAT_VERSION,
            game: self.game,
            player: player.raw(),
            manifest: self.manifest.clone(),
            first: viewer.record.first(viewer.n),
        };
        Some(viewer.record.text(codec::encode_text(&header)))
    }

    /// `command` of `actor` mapped to the engine's, applied and journaled: the command as
    /// applied, on the wire, and what it told.
    fn apply(
        &mut self,
        actor: Actor,
        command: GameCommand,
    ) -> Result<(GameCommand, Told<'r>), GameReject> {
        let command = map::game::command_to_engine(command)?;
        let echo = map::game::command(&command);
        let mut told = Told {
            rules: self.rules,
            events: Vec::new(),
            ended: Vec::new(),
        };
        h3_game::apply(
            &mut self.state,
            self.rules,
            actor,
            command.clone(),
            &mut told,
        )
        .map_err(map::game::reject)?;
        self.journal
            .append(actor, command, &self.state, &mut self.entries);
        Ok((echo, told))
    }

    /// Every viewer gets the command's events as it may see them, its battle's last view when the
    /// command ended it, what it newly sees when its explored tiles grew and a battle's snapshot
    /// when one opens for it; then each player that may act its turn.
    fn tell(&mut self, told: Told<'_>, out: &mut Vec<GameOutbound>) {
        let Told { events, ended, .. } = told;
        for (player, viewer) in players().zip(&mut self.viewers) {
            let engine_viewer = EngineViewer::Player(player);
            let events = map::game::events(&events, engine_viewer);
            let started = events
                .iter()
                .any(|e| matches!(e, GameEvent::BattleStarted { .. }));
            // The fold ends a view's battle at its end (`GameObservation::fold`); the battle's
            // last view follows the events.
            let held = viewer.battle
                && !events
                    .iter()
                    .any(|e| matches!(e, GameEvent::BattleEnded { .. }));
            viewer.post(player, |n| ServerMessage::GameEvents { n, events }, out);
            for (_, observation) in ended.iter().filter(|(owner, _)| *owner == player) {
                let observation = observation.clone();
                viewer.post(
                    player,
                    |n| ServerMessage::GameLastSnapshot { n, observation },
                    out,
                );
            }
            let now = explored(&self.state, player);
            // The count changes exactly when the grid does (the design note §14).
            if let (Some((since, before)), Some((_, count))) = (&viewer.explored, &now)
                && before != count
            {
                let reveal = map::game::reveal(&self.state, self.rules, engine_viewer, since);
                viewer.post(player, |n| ServerMessage::GameReveal { n, reveal }, out);
            }
            viewer.explored = now;
            let open = own_battle(&self.state, player);
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
            let castable = own_battle(&self.state, player)
                .filter(|(battle, side)| battle.state().side_to_act() == Some(*side))
                .map(|(battle, side)| map::castable(battle.state(), self.rules.battle(), side))
                .unwrap_or_default();
            viewer.post(
                player,
                |n| ServerMessage::GameTurn { n, legal, castable },
                out,
            );
        }
    }
}

/// `player`'s battle, its first open decision's, with the side it owns there (it owns one: the
/// decision is its own).
fn own_battle(state: &GameState, player: PlayerId) -> Option<(&OpenBattle, BattleSide)> {
    state
        .battle(player)
        .and_then(|battle| battle.side_of(player).map(|side| (battle, side)))
}

/// `player`'s explored grid now with its count; none for a player outside the game.
fn explored(state: &GameState, player: PlayerId) -> Option<(BitGrid, u32)> {
    state
        .adventure()
        .explored(player)
        .map(|grid| (grid.clone(), grid.count()))
}

/// The players by index, as many as the game has (at most 256: `PlayerId`'s range).
fn players() -> impl Iterator<Item = PlayerId> {
    (0..=u8::MAX).map(PlayerId::new)
}
