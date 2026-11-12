//! The whole game's session (E16 M5 T5a; SRV-Q25; ADR-0018 points 6 and 7 for the whole game):
//! one hosted game, its players as seats and viewers, the authoritative loop over
//! `h3_game::apply`, sans I/O. In-process the host passes in each player's command with the
//! player it vouches for ([`GameSession::play`]) and the system actor's ([`GameSession::system`]),
//! and delivers the frames that come out ([`GameOutbound`]); over the network it passes in what
//! each connection sends ([`GameSession::handle`]) and sends the frames for connections
//! ([`Outbound`]). The session never sees a socket, a clock or a file.
//!
//! - **Seats** (E16 M5 T8a). Every player of the map is a seat. In-process the host holds them
//!   all at once and plays them (a replay's commands, a test's: [`GameSession::start`]). Hosted
//!   for connections ([`GameSession::new`]), a seat is taken as a battle's is (`GameJoin`): with
//!   the token the host drew, or as the account a logged-in connection is
//!   ([`GameSession::logged_in`]; the battle's rules, `super::refusal`); taken without a token
//!   only the first time, re-taken after a disconnect with its token or by its account, a re-take
//!   replacing a connection that still holds it. The game starts when every seat is held at
//!   once; until then a command is rejected `NotStarted`. A joining seat gets `GameJoined`, its
//!   `GameSnapshot` of the game as it stands and, once the game started, its `GameTurn` if it may
//!   act; a seat's record goes on while no connection holds it (its frames are recorded, not
//!   sent). A command is applied with the seat's player, never with anything the client wrote
//!   (ADR-0018 point 6): the wire's commands name no player; one from a connection without a seat
//!   is rejected `NotSeated`, numbered 0. Computer seats, turn timers and the level-up hold's
//!   timer are part 2's (the epic).
//! - **The system actor** (`h3_game::Actor::System`, E08 T20b2): its commands come from the host
//!   ([`GameSession::system`]; its legal ones [`GameSession::system_legal`]): an unowned battle
//!   side's, a wandering monster's. No player is told its acceptance or its rejection; its events
//!   are told as every command's.
//! - **Viewers.** Each player is a viewer with its own message numbers `n`; everything it gets is
//!   projected for it by construction ([`crate::map::game`]). When the game starts it gets its
//!   `GameSnapshot`. After every accepted command: the player that sent it `GameAccepted`
//!   (echoed), then every viewer `GameEvents` with the command's events as it may see them (also
//!   an empty list, so that what a viewer receives does not depend on what it may not see), its
//!   battle's last `Snapshot`, as its side saw it when the command ended it (E16 M5 T5c: the
//!   session's sink keeps each owner's last view, `h3_game::Sink::battle_ended`; SRV-Q27 (1);
//!   none when the same events open another battle for it, whose opening `Snapshot` a reader
//!   could not tell from it), its `GameReveal` when the command explored tiles for it (E16 M5 T5c, SRV-Q26 (a): the session
//!   keeps each viewer's explored grid as it was last told it and asks `h3_game::reveal` when the
//!   grid changed, T7b) and a battle's own `Snapshot` when one opens for it (SRV-Q27 (1): its
//!   events are then the battle client's, inside `GameEvents`), a fresh `GameSnapshot` when its
//!   record needs one (below), then every player that may act its `GameTurn`. A rejected command changes nothing and is told to its player alone
//!   (`GameRejected`).
//! - **The journal** (E16 M5 T5b). The accepted commands with their actors and a state hash every
//!   `checkpoint_every` commands: the adventure replay in progress ([`GameJournal`]; its header,
//!   map and difficulty are the host's). What a restart must redo comes out as [`GameEntry`]s,
//!   which the host drains after every call ([`GameSession::drain_entries`]) and makes durable
//!   before it delivers that call's frames. [`GameSession::restore`] re-runs them: the same game,
//!   every player then told it afresh as at the start (its numbers and record begin again: they
//!   are not journaled; a rejection changes nothing and is not journaled either).
//! - **Records** (E16 M5 T5b, record format v4). Each player's record is the log of every
//!   numbered message it was sent, within the host's bound ([`GameOptions::record_bytes`]): the
//!   oldest lines go first, but only up to a `GameSnapshot`, which the rest folds onto (T7a1).
//!   Once a player was sent more than half the bound since its last snapshot it gets a fresh one
//!   after a command's messages (before its turn) or after its own rejection (T7b), unless a
//!   battle is open in its view (whose own `Snapshot` opens it), so a record holds about one to
//!   two such stretches; one command's messages, or a battle's (its own rejections there too), may
//!   take it beyond its bound. [`GameSession::record`] gives it with
//!   its header line ([`GameRecordHeader`]: the game's manifest and the player), as a record file
//!   holds it.
//! - **Not here yet:** a hero's moves, which
//!   `GameTurn` leaves out as `h3_game::legal_commands` does (`h3_game::legal_moves`, T11); the
//!   seats' entries (a restore with the seats taken), the commands' numbers against resends and
//!   the host's pause (T8b); the journal's file and the hosting over the network (T9).
//!
//! Provisional (E16 M5 T5a, T5b, T8a; the epic's review list): the names, the start as the
//! constructor, the message order, a `GameTurn` after every accepted command to every player that
//! may act, a system command told to nobody, a player outside the game told nothing; the entries,
//! a restore telling every player its snapshot and turn afresh, the record's header with the
//! game's id; a reveal as a message of its own after the events, a battle's last `Snapshot`
//! before it (T5c, the client lane's reader, 2026-11-12); the seats' messages of their own
//! (`GameJoin`, `GameCommand`, `GameJoined`), the session's rejections inside `GameReject`, a
//! battle's messages ignored here but its join and command (answered as a battle answers a whole
//! game's), the host's tokens drawn through a closure (T8a).

use alloc::collections::BTreeMap;
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

use super::{
    ConnId, Outbound, Record, game_unseated, goodbye, handed, refusal, refused, reply, unseated,
};
use crate::RECORD_FORMAT_VERSION;
use crate::codec;
use crate::map;
use crate::wire::game::{GameCommand, GameEvent, GameManifest, GameRecordHeader, GameReject};
use crate::wire::{ClientMessage, GoodbyeReason, Observation, Refusal, ServerMessage};

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
    /// it the oldest go up to a snapshot, the newest snapshot and what follows it stay; past half
    /// of it since its last snapshot a player gets a fresh one (E16 M5 T7a1). `usize::MAX`: the
    /// whole game, one snapshot.
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
    /// Its explored tiles as it was last told them (E16 M5 T5c): the grid `h3_game::reveal`
    /// compares with; none before the game is opened.
    explored: Option<BitGrid>,
    /// The bytes of the lines its record got since its last snapshot (E16 M5 T7a1).
    since: usize,
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
        let message = message(self.n);
        let snapshot = matches!(message, ServerMessage::GameSnapshot { .. });
        let frame = codec::encode_text(&message);
        // A record begins with a snapshot, which the rest folds onto (E16 M5 T7a1).
        self.since = if snapshot {
            0
        } else {
            self.since.saturating_add(frame.len() + 1)
        };
        self.record.push(frame.clone(), snapshot);
        out.push(GameOutbound { to, frame });
    }

    /// Past half its record's bound since its last snapshot, the viewer `to` gets a fresh one of
    /// `state` (E16 M5 T7a1), where its record may be cut; never inside a battle, which its own
    /// `Snapshot` opens.
    fn refresh(
        &mut self,
        to: PlayerId,
        state: &GameState,
        rules: &GameRuleset,
        out: &mut Vec<GameOutbound>,
    ) {
        if self.since > self.record.limit() / 2 && !self.battle {
            let observation = map::game::observe(state, rules, EngineViewer::Player(to));
            self.post(to, |n| ServerMessage::GameSnapshot { n, observation }, out);
        }
    }
}

/// A player's seat in a game hosted for connections (E16 M5 T8a): a battle's player seat's
/// claim (`super::refusal`).
#[derive(Debug, Clone, PartialEq, Eq)]
struct SeatState {
    /// Its token (opaque; the host drew it, ADR-0018 point 6).
    token: String,
    /// Whether it was taken once: its token handed out, or it belongs to an account.
    issued: bool,
    /// The login name of the account it belongs to: the account of the connection that took it
    /// first, if that one was logged in.
    account: Option<String>,
    /// The connection that holds it.
    holder: Option<ConnId>,
}

/// Who holds the players' seats (E16 M5 T8a).
#[derive(Debug, Clone, PartialEq, Eq)]
enum Seats {
    /// The host, every one at once: it plays them all in-process.
    Host,
    /// Connections take them, by player index.
    Taken(Vec<SeatState>),
}

/// One hosted whole game: its state, its manifest, its journal, its players as seats and as
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
    /// Who holds the seats (E16 M5 T8a).
    seats: Seats,
    /// Whether every seat was held at once: the game runs (E16 M5 T8a).
    started: bool,
    /// The login names of the logged-in connections, as the host said (E16 M5 T8a).
    logins: BTreeMap<ConnId, String>,
}

impl<'r> GameSession<'r> {
    /// Starts the game of `setup` with `seed` under `rules` (`h3_game::GameState::new`), made of
    /// what `manifest` names (the caller's: [`crate::map::game::manifest`]) and hosted as
    /// `options` say, in-process: the host holds every seat at once (E16 M5 T8a), so the game
    /// starts now; every player gets its `GameSnapshot`, then each that may act its `GameTurn`.
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
        let mut session = Self::build(rules, setup, seed, manifest, options)?;
        session.open(out);
        Ok(session)
    }

    /// Hosts the game of `setup` as [`GameSession::start`] builds it, for connections (E16 M5
    /// T8a): a seat per player of the map, its token `token(player)`'s (the host draws each from
    /// the OS, ADR-0018 point 6), taken by the connections' `GameJoin`s ([`GameSession::handle`]).
    /// Nothing is sent until a connection joins; the game starts when every seat is held.
    ///
    /// # Errors
    ///
    /// The engine's when the setup starts no game.
    pub fn new(
        rules: &'r GameRuleset,
        setup: &GameSetup,
        seed: u64,
        manifest: GameManifest,
        options: GameOptions,
        mut token: impl FnMut(PlayerId) -> String,
    ) -> Result<Self, GameSetupError> {
        let mut session = Self::build(rules, setup, seed, manifest, options)?;
        let seats = players()
            .take(session.viewers.len())
            .map(|player| SeatState {
                token: token(player),
                issued: false,
                account: None,
                holder: None,
            });
        session.seats = Seats::Taken(seats.collect());
        session.started = false;
        Ok(session)
    }

    /// Restores a game from its journal's `entries` after a restart (E16 M5 T5b): builds the game
    /// as [`GameSession::start`] does, re-runs the entries' commands through the engine, each as
    /// its actor's, then tells every player the restored game afresh as `start` does (its
    /// numbers and record begin again), in-process as `start` (its seats the host's; T8b restores
    /// a game of connections). The re-run makes its own entries, which the journal must
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
            Self::build(rules, setup, seed, manifest, options).map_err(GameRestoreError::Setup)?;
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

    /// The game of `setup`, nothing told yet, its seats the host's.
    fn build(
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
            since: 0,
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
            seats: Seats::Host,
            started: true,
            logins: BTreeMap::new(),
        })
    }

    /// Every player gets its `GameSnapshot` of the game as it stands, then each that may act its
    /// `GameTurn`.
    fn open(&mut self, out: &mut Vec<GameOutbound>) {
        for player in players().take(self.viewers.len()) {
            self.snapshot(player, out);
        }
        self.turns(out);
    }

    /// `player`'s `GameSnapshot` of the game as it stands: its view starts afresh from it.
    fn snapshot(&mut self, player: PlayerId, out: &mut Vec<GameOutbound>) {
        let observation = map::game::observe(&self.state, self.rules, EngineViewer::Player(player));
        let viewer = &mut self.viewers[player.index()];
        viewer.battle = observation.battle.is_some();
        viewer.explored = explored(&self.state, player);
        viewer.post(
            player,
            |n| ServerMessage::GameSnapshot { n, observation },
            out,
        );
    }

    /// Handles what connection `conn` sent after its hello (E16 M5 T8a); the frames to send go to
    /// `out`. `GameJoin` takes or re-takes a seat, `GameCommand` is applied as the seat's
    /// player's ([`GameSession::play`]), `Leave` frees the seat and closes. A battle's `Join` is
    /// refused `NoSuchGame` and its `Command` rejected `NotSeated`; the hello, the ping and the
    /// account's requests are the connection's ([`crate::connection`]), the lobby's the host's,
    /// a battle's other messages none of a whole game's (T8b and T9 take what it needs): they are
    /// ignored here.
    pub fn handle(&mut self, conn: ConnId, message: ClientMessage, out: &mut Vec<Outbound>) {
        match message {
            ClientMessage::GameJoin {
                game,
                player,
                token,
            } => self.join(conn, (game, player), token.as_deref(), out),
            ClientMessage::GameCommand { seq, command } => self.command(conn, seq, command, out),
            ClientMessage::Leave {} => {
                self.disconnected(conn);
                out.push(goodbye(conn, GoodbyeReason::Left));
            }
            // A battle's: a whole game is none, nor a seat of one.
            ClientMessage::Join { .. } => out.push(refused(conn, Refusal::NoSuchGame)),
            ClientMessage::Command { seq, .. } => out.push(unseated(conn, seq)),
            ClientMessage::Hello { .. }
            | ClientMessage::Ping { .. }
            | ClientMessage::Login { .. }
            | ClientMessage::Rekey { .. }
            | ClientMessage::Rename { .. }
            | ClientMessage::SetListed { .. }
            | ClientMessage::ListGames { .. }
            | ClientMessage::CreateGame { .. }
            | ClientMessage::ListReplays { .. }
            | ClientMessage::FetchArchived { .. }
            | ClientMessage::ListLadders { .. }
            | ClientMessage::Queue { .. }
            | ClientMessage::Dequeue { .. }
            | ClientMessage::AcceptMatch { .. }
            | ClientMessage::DeclineMatch { .. }
            | ClientMessage::Leaderboard { .. }
            | ClientMessage::Standing { .. }
            | ClientMessage::Describe { .. }
            | ClientMessage::ListPeers { .. }
            | ClientMessage::Lists { .. }
            | ClientMessage::Query { .. }
            | ClientMessage::FetchReplay { .. }
            | ClientMessage::FetchRecord { .. }
            | ClientMessage::Pause { .. }
            | ClientMessage::ProposePause { .. }
            | ClientMessage::AnswerPause { .. }
            | ClientMessage::SetSpectators { .. } => {}
        }
    }

    /// The host's word that connection `conn` is logged in as the account `login` (E16 M5 T8a,
    /// as a battle's `Session::logged_in`): a seat it takes belongs to the account, and it
    /// re-takes the account's seats. Call it before the connection's join; it changes nothing
    /// else.
    pub fn logged_in(&mut self, conn: ConnId, login: String) {
        self.logins.insert(conn, login);
    }

    /// The connection `conn` is gone (E16 M5 T8a): its seat waits for a re-take with its token,
    /// or by its account; its record goes on meanwhile.
    pub fn disconnected(&mut self, conn: ConnId) {
        for seat in self.taken_mut() {
            if seat.holder == Some(conn) {
                seat.holder = None;
            }
        }
        self.logins.remove(&conn);
    }

    /// Whether the game runs (E16 M5 T8a): every seat was held at once, or the host holds them
    /// (in-process).
    pub const fn started(&self) -> bool {
        self.started
    }

    /// `GameJoin` of `player`'s seat in `game` with `token` (E16 M5 T8a): takes or re-takes it.
    fn join(
        &mut self,
        conn: ConnId,
        (game, player): (u32, u8),
        token: Option<&str>,
        out: &mut Vec<Outbound>,
    ) {
        if game != self.game {
            return out.push(refused(conn, Refusal::NoSuchGame));
        }
        // In-process, the host holds every seat.
        let Seats::Taken(seats) = &mut self.seats else {
            return out.push(refused(conn, Refusal::SeatTaken));
        };
        if seats.iter().any(|seat| seat.holder == Some(conn)) {
            return out.push(refused(conn, Refusal::AlreadyJoined));
        }
        let Some(seat) = seats.get_mut(usize::from(player)) else {
            return out.push(refused(conn, Refusal::NoSuchSeat));
        };
        let login = self.logins.get(&conn);
        let claim = (seat.token.as_str(), seat.issued, seat.account.as_deref());
        if let Some(reason) = refusal(claim, token, login.map(String::as_str)) {
            return out.push(refused(conn, reason));
        }
        let token = handed(claim, login.map(String::as_str));
        if !seat.issued {
            seat.issued = true;
            seat.account = login.cloned();
        }
        if let Some(old) = seat.holder.replace(conn) {
            out.push(goodbye(old, GoodbyeReason::Replaced));
        }
        let held = seats.iter().all(|seat| seat.holder.is_some());
        let manifest = self.manifest.clone();
        out.push(reply(
            conn,
            &ServerMessage::GameJoined {
                game,
                player,
                token,
                manifest,
            },
        ));
        let (player, mut frames) = (PlayerId::new(player), Vec::new());
        self.snapshot(player, &mut frames);
        if self.started {
            self.turn(player, &mut frames);
        } else if held {
            self.started = true;
            self.turns(&mut frames);
        }
        self.route(frames, out);
    }

    /// `GameCommand` (E16 M5 T8a): the seat's command, applied as its player's.
    fn command(&mut self, conn: ConnId, seq: u32, command: GameCommand, out: &mut Vec<Outbound>) {
        let Some(player) = self.seat_of(conn) else {
            return out.push(game_unseated(conn, seq));
        };
        let mut frames = Vec::new();
        // A rejection is told to the player in the frames; it changed nothing.
        let _rejected = self.play(player, seq, command, &mut frames);
        self.route(frames, out);
    }

    /// The frames for players in `frames` (a call's: [`GameSession::system`]'s in a game of
    /// connections), each to the connection that holds its player's seat; none for a seat no
    /// connection holds, whose record has it (E16 M5 T8a). In-process, none: the host holds every
    /// seat and delivers the frames itself.
    pub fn route(&self, frames: Vec<GameOutbound>, out: &mut Vec<Outbound>) {
        let seats = self.taken();
        for GameOutbound { to, frame } in frames {
            if let Some(conn) = seats.get(to.index()).and_then(|seat| seat.holder) {
                out.push(Outbound {
                    to: conn,
                    frame,
                    close: false,
                });
            }
        }
    }

    /// The player whose seat `conn` holds.
    fn seat_of(&self, conn: ConnId) -> Option<PlayerId> {
        let mut seats = players().zip(self.taken());
        seats
            .find(|(_, seat)| seat.holder == Some(conn))
            .map(|(player, _)| player)
    }

    /// The seats connections take: none in-process, where the host holds them.
    fn taken(&self) -> &[SeatState] {
        match &self.seats {
            Seats::Host => &[],
            Seats::Taken(seats) => seats,
        }
    }

    /// The seats connections take, to change.
    fn taken_mut(&mut self) -> &mut [SeatState] {
        match &mut self.seats {
            Seats::Host => &mut [],
            Seats::Taken(seats) => seats,
        }
    }

    /// `player`'s command `command`, which the host vouches came from it, numbered `seq` by its
    /// client: applied as the player's (`Actor::Player`), then told (the module's order). A player
    /// outside the game is told nothing. In a game of connections it is the seat's command
    /// ([`GameSession::handle`]); before the game started it is rejected `NotStarted`.
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
                    // Its own rejections fill its record too (E16 M5 T7b's review).
                    viewer.refresh(player, &self.state, self.rules, out);
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
        if !self.started {
            return Err(GameReject::NotStarted {});
        }
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
            let last = ended
                .iter()
                .filter(|(owner, _)| *owner == player && !started);
            for (_, observation) in last {
                let observation = observation.clone();
                viewer.post(player, |n| ServerMessage::Snapshot { n, observation }, out);
            }
            let now = explored(&self.state, player);
            // The grid, not its count (T7b's review): a count stays when as many tiles are
            // hidden as explored (once Cover of Darkness hides them, E09).
            if let (Some(since), Some(now)) = (&viewer.explored, &now)
                && since != now
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
            viewer.refresh(player, &self.state, self.rules, out);
        }
        self.turns(out);
    }

    /// Each player that may act gets its turn.
    fn turns(&mut self, out: &mut Vec<GameOutbound>) {
        for player in players().take(self.viewers.len()) {
            self.turn(player, out);
        }
    }

    /// If `player` may act, its legal commands and, while its battle side is to act, the spells
    /// its hero may cast.
    fn turn(&mut self, player: PlayerId, out: &mut Vec<GameOutbound>) {
        let legal: Vec<GameCommand> =
            h3_game::legal_commands(&self.state, self.rules, Actor::Player(player))
                .iter()
                .map(map::game::command)
                .collect();
        if legal.is_empty() {
            return;
        }
        let castable = own_battle(&self.state, player)
            .filter(|(battle, side)| battle.state().side_to_act() == Some(*side))
            .map(|(battle, side)| map::castable(battle.state(), self.rules.battle(), side))
            .unwrap_or_default();
        self.viewers[player.index()].post(
            player,
            |n| ServerMessage::GameTurn { n, legal, castable },
            out,
        );
    }
}

/// `player`'s battle, its first open decision's, with the side it owns there (it owns one: the
/// decision is its own).
fn own_battle(state: &GameState, player: PlayerId) -> Option<(&OpenBattle, BattleSide)> {
    state
        .battle(player)
        .and_then(|battle| battle.side_of(player).map(|side| (battle, side)))
}

/// `player`'s explored grid now; none for a player outside the game.
fn explored(state: &GameState, player: PlayerId) -> Option<BitGrid> {
    state.adventure().explored(player).cloned()
}

/// The players by index, as many as the game has (at most 256: `PlayerId`'s range).
fn players() -> impl Iterator<Item = PlayerId> {
    (0..=u8::MAX).map(PlayerId::new)
}
