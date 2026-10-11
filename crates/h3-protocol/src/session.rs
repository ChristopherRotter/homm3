//! The session core (ADR-0018 points 6–8; E16 M1 T2): one hosted battle, its two seats and the
//! authoritative loop, sans I/O. The host numbers its connections ([`ConnId`]), passes in what
//! each one sends after its hello ([`crate::connection`]) and sends the frames that come out
//! ([`Outbound`]); the session never sees a socket, a clock or a file.
//!
//! - **Seats.** Each side is a player's seat, taken with the token the host supplied, or a bot's
//!   ([`SeatKind`]). A connection that holds a seat is its actor: commands are applied with the
//!   seat's side, never with anything the client wrote (ADR-0018 point 6). A seat is taken
//!   without a token only the first time, and re-taken after a disconnect with its token; a
//!   re-take replaces a connection that still holds it.
//! - **The loop.** The game starts when every seat is held (a bot's always is). A command is
//!   validated and applied by the engine; a rejected one changes neither the battle nor the
//!   log. An accepted one is acknowledged to its seat (`Accepted`, echoed), then every viewer gets
//!   the events projected for it (an `Events` message for every accepted command, even an empty
//!   one, so that what a viewer receives does not depend on what it may not see), then bots act
//!   while it is their turn, then the seat to act gets `YourTurn` or every viewer `GameEnded`.
//! - **Viewers.** Each side is a viewer with its own message numbers `n` and its *record*, the
//!   log of every numbered message it was sent (also while no connection holds the seat).
//!   `Replay` and `Record` answers carry the last number and are not recorded. Everything a viewer
//!   gets is projected for it by construction ([`crate::map`]).
//! - **The journal.** The seed, the setup, the accepted commands with their sides and a state
//!   hash every `checkpoint_every` commands: the replay in progress (ADR-0018 point 12). The host
//!   renders it as the replay file at the end ([`Session::publish_replay`]).
//! - **Write-ahead** (E16 M2 T2). What the session did that a restart must redo comes out as
//!   [`Entry`]s, which the host drains after every call ([`Session::drain_entries`]) and makes
//!   durable before it sends that call's frames, so nothing a client was told is lost.
//!   [`Session::restore`] re-runs them after a restart: the same battle, the same seats taken, no
//!   seat held. A viewer's numbers and record start afresh there (they are not journaled).
//!
//! Provisional (E16 M1 T2, M2 T2): the names, the connection numbering, frames as text, the
//! record's layout, the entries (the epic's review list).

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use h3_battle::{BattleSetup, BattleSide, BattleState, CommandError, LegalCommands, SetupError};
use h3_content::Ruleset;

use crate::RECORD_FORMAT_VERSION;
use crate::codec;
use crate::map;
use crate::wire::{
    ClientMessage, Command, GoodbyeReason, Manifest, Outcome, Phase, Query, RecordHeader, Refusal,
    RejectCode, Seat, ServerMessage,
};

/// A connection, as the host numbers them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConnId(pub u64);

/// A frame for a connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outbound {
    /// The connection.
    pub to: ConnId,
    /// The frame's text: one message.
    pub frame: String,
    /// Whether the host closes the connection after it (a `Goodbye`).
    pub close: bool,
}

/// A bot that plays a seat: the host's (ADR-0018 point 6). It is asked for a command whenever its
/// side is to act, with the side's legal commands; casts are not listed there but are valid
/// choices. It must use only what its side may know (the engine's per-side functions:
/// `legal_commands`, `castable_spells`, `cast_targets`, `observe`). A choice the engine rejects is
/// replaced by the first legal command.
pub trait Bot: fmt::Debug {
    /// The command for `side`.
    fn command(
        &mut self,
        state: &BattleState,
        ruleset: &Ruleset,
        side: BattleSide,
        legal: &LegalCommands,
    ) -> h3_battle::Command;
}

/// Who plays a side.
#[derive(Debug)]
pub enum SeatKind {
    /// A player, who takes the seat with this token (opaque; the host draws it, ADR-0018 point 6).
    Player {
        /// The seat's token.
        token: String,
    },
    /// The host's bot.
    Bot(Box<dyn Bot + Send>),
}

/// What the host hosts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameConfig {
    /// The game's id, as clients name it in `Join`.
    pub game: u32,
    /// The battle, with its obstacles placed (ADR-0018 point 8).
    pub setup: BattleSetup,
    /// The battle's seed: secret, from the OS (ADR-0018 point 8).
    pub seed: u64,
    /// A state hash goes into the journal after every this many commands (0: none).
    pub checkpoint_every: u32,
}

/// The journal: everything the replay needs except its header (ADR-0018 point 12). Private to the
/// server until the game is over.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Journal {
    /// The battle's seed.
    pub seed: u64,
    /// The battle.
    pub setup: BattleSetup,
    /// The accepted commands in order, with the side that gave each.
    pub commands: Vec<(BattleSide, h3_battle::Command)>,
    /// A state hash after every this many commands (0: none).
    pub checkpoint_every: u32,
    /// The state hashes after commands `k · checkpoint_every`, k = 1, 2, …
    pub checkpoints: Vec<u64>,
}

impl Journal {
    /// Appends an accepted command and, when one is due, the state's hash after it; both as
    /// entries to `entries` too.
    fn append(
        &mut self,
        side: BattleSide,
        command: h3_battle::Command,
        state: &BattleState,
        entries: &mut Vec<Entry>,
    ) {
        self.commands.push((side, command));
        entries.push(Entry::Command(side, command));
        let every = usize::try_from(self.checkpoint_every).unwrap_or(usize::MAX);
        if self.commands.len().checked_rem(every) == Some(0) {
            let hash = state.state_hash();
            self.checkpoints.push(hash);
            entries.push(Entry::Checkpoint(hash));
        }
    }
}

/// One entry of the journal the host keeps on disk (E16 M2 T2), in the order the session made
/// them: what a restart redoes ([`Session::restore`]). Private to the server, as the journal is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entry {
    /// The seat of the side was taken: its token was handed out, so it is re-taken only with it.
    Taken(BattleSide),
    /// Every seat was held at once: the game started.
    Started,
    /// The engine accepted the command from the side.
    Command(BattleSide, h3_battle::Command),
    /// The state's hash after the commands so far, every `checkpoint_every` commands.
    Checkpoint(u64),
}

/// Why a journal does not restore its game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreError {
    /// The setup does not start a battle.
    Setup(SetupError),
    /// The engine rejected the command of entry `at`.
    Rejected {
        /// The entry's index.
        at: usize,
        /// Why.
        error: CommandError,
    },
    /// From entry `at` on, the journal is not what its re-run makes: a checkpoint's hash differs,
    /// or a checkpoint is extra, out of place, or missing before the journal's end.
    Diverged {
        /// The entry's index.
        at: usize,
    },
}

/// One viewer: its message numbers and its record.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Viewer {
    /// The number of the last message sent.
    n: u32,
    /// Every numbered message sent, as its frame.
    record: Vec<String>,
}

/// A side's seat.
#[derive(Debug)]
struct SeatState {
    /// Who plays it.
    kind: SeatKind,
    /// Whether its token was handed out (the seat was taken once).
    issued: bool,
    /// The connection that holds it.
    holder: Option<ConnId>,
    /// The side as a viewer.
    viewer: Viewer,
}

impl SeatState {
    /// Whether it is held: by a connection, or by its bot.
    const fn held(&self) -> bool {
        matches!(self.kind, SeatKind::Bot(_)) || self.holder.is_some()
    }
}

/// One hosted battle and its seats.
#[derive(Debug)]
pub struct Session<'r> {
    /// The server's ruleset, the only one in the game (ADR-0018 point 10).
    ruleset: &'r Ruleset,
    /// The game's id.
    game: u32,
    /// What the game is made of, sent with every `Joined`.
    manifest: Manifest,
    /// The battle.
    state: BattleState,
    /// The attacker's and the defender's seat.
    seats: [SeatState; 2],
    /// Whether every seat was held at once.
    started: bool,
    /// The replay in progress.
    journal: Journal,
    /// The entries the host has not drained yet.
    entries: Vec<Entry>,
    /// The replay file's text, once the host published it.
    replay: Option<String>,
}

impl<'r> Session<'r> {
    /// Hosts the battle of `config` with `seats` (attacker, defender). Nothing is sent until a
    /// connection joins; a game with two bot seats never starts.
    pub fn new(
        ruleset: &'r Ruleset,
        config: GameConfig,
        seats: [SeatKind; 2],
    ) -> Result<Self, SetupError> {
        let state = BattleState::new(ruleset, &config.setup, config.seed)?;
        let seats = seats.map(|kind| SeatState {
            kind,
            issued: false,
            holder: None,
            viewer: Viewer::default(),
        });
        Ok(Self {
            ruleset,
            game: config.game,
            manifest: map::manifest(ruleset),
            state,
            seats,
            started: false,
            journal: Journal {
                seed: config.seed,
                setup: config.setup,
                commands: Vec::new(),
                checkpoint_every: config.checkpoint_every,
                checkpoints: Vec::new(),
            },
            entries: Vec::new(),
            replay: None,
        })
    }

    /// Restores a game from its journal's `entries` after a restart (E16 M2 T2): hosts the battle
    /// of `config` with `seats` as [`Session::new`] does, then re-runs the entries in order (the
    /// seats taken, the start, the commands through the engine). The re-run makes its own entries,
    /// which the journal must begin: a checkpoint whose hash differs, or one out of place, is a
    /// divergence. No seat is held after it; a taken one is re-taken with its
    /// token. The first drained entries then are what the journal lacks: the checkpoint after its
    /// last command if a crash cut the journal between the two, and the bots' commands if a started
    /// game is theirs to act (a crash between a player's command and the bots' answers).
    pub fn restore(
        ruleset: &'r Ruleset,
        config: GameConfig,
        seats: [SeatKind; 2],
        entries: &[Entry],
    ) -> Result<Self, RestoreError> {
        let mut session = Self::new(ruleset, config, seats).map_err(RestoreError::Setup)?;
        for (at, &entry) in entries.iter().enumerate() {
            match entry {
                Entry::Taken(side) => {
                    session.seats[side.index()].issued = true;
                    session.entries.push(entry);
                }
                Entry::Started => {
                    session.started = true;
                    session.entries.push(entry);
                }
                Entry::Command(side, command) => {
                    session
                        .apply(side, command)
                        .map_err(|error| RestoreError::Rejected { at, error })?;
                }
                // The re-run's own follows its command; the two are compared below.
                Entry::Checkpoint(_) => {}
            }
        }
        if !session.entries.starts_with(entries) {
            let same = entries.iter().zip(&session.entries);
            let at = same
                .take_while(|(recorded, rerun)| recorded == rerun)
                .count();
            return Err(RestoreError::Diverged { at });
        }
        session.entries.drain(..entries.len());
        if session.started {
            session.run_bots(&mut Vec::new());
        }
        Ok(session)
    }

    /// Handles what connection `conn` sent after its hello; the frames to send go to `out`. The
    /// hello and the ping are the connection's ([`crate::connection`]), the lobby's messages the
    /// host's ([`crate::lobby`]); they are ignored here.
    pub fn handle(&mut self, conn: ConnId, message: ClientMessage, out: &mut Vec<Outbound>) {
        match message {
            ClientMessage::Join { game, seat, token } => self.join(conn, (game, seat), token, out),
            ClientMessage::Command { seq, command } => self.command(conn, seq, command, out),
            ClientMessage::Query { seq, query } => self.query(conn, seq, query, out),
            ClientMessage::FetchReplay { seq } => self.fetch_replay(conn, seq, out),
            ClientMessage::FetchRecord { seq } => self.fetch_record(conn, seq, out),
            ClientMessage::Leave {} => {
                self.disconnected(conn);
                out.push(goodbye(conn, GoodbyeReason::Left));
            }
            // The connection's and the lobby's (`crate::lobby`), not the game's.
            ClientMessage::Hello { .. }
            | ClientMessage::Ping { .. }
            | ClientMessage::ListGames { .. }
            | ClientMessage::CreateGame { .. } => {}
        }
    }

    /// The connection `conn` is gone: its seat waits for a re-take with the token.
    pub fn disconnected(&mut self, conn: ConnId) {
        for seat in &mut self.seats {
            seat.holder = seat.holder.filter(|&holder| holder != conn);
        }
    }

    /// The battle's result, once it is over: then the host renders the journal as the replay
    /// and publishes it.
    pub fn result(&self) -> Option<h3_battle::BattleResult> {
        self.state.result()
    }

    /// The replay in progress.
    pub const fn journal(&self) -> &Journal {
        &self.journal
    }

    /// The entries made since the last call, in order (E16 M2 T2): the host makes them durable
    /// before it sends the frames of the call that made them (write-ahead, ADR-0018 point 12).
    pub fn drain_entries(&mut self) -> alloc::vec::Drain<'_, Entry> {
        self.entries.drain(..)
    }

    /// The live battle's state hash (the replay's final hash once the game is over).
    pub fn state_hash(&self) -> u64 {
        self.state.state_hash()
    }

    /// Whether the game started (every seat was held at once).
    pub const fn started(&self) -> bool {
        self.started
    }

    /// Where the game is, for the lobby's list (E16 M2 T1).
    pub fn phase(&self) -> Phase {
        if self.state.is_over() {
            Phase::Over
        } else if self.started {
            Phase::Running
        } else {
            Phase::Waiting
        }
    }

    /// The seats a newcomer may take without a token: a player's seat whose token was not handed
    /// out yet (E16 M2 T1).
    pub fn open_seats(&self) -> Vec<Seat> {
        BattleSide::BOTH
            .into_iter()
            .filter(|side| {
                let held = &self.seats[side.index()];
                matches!(held.kind, SeatKind::Player { .. }) && !held.issued
            })
            .map(seat)
            .collect()
    }

    /// The replay file's text, for the participants once the game is over (ADR-0018 point 8): a
    /// `FetchReplay` before the end is refused even if the text is there.
    pub fn publish_replay(&mut self, replay: String) {
        self.replay = Some(replay);
    }

    /// `Join`: takes or re-takes a seat.
    fn join(
        &mut self,
        conn: ConnId,
        (game, seat): (u32, Seat),
        token: Option<String>,
        out: &mut Vec<Outbound>,
    ) {
        if game != self.game {
            return out.push(refused(conn, Refusal::NoSuchGame));
        }
        let side = match seat {
            Seat::Attacker => BattleSide::Attacker,
            Seat::Defender => BattleSide::Defender,
            Seat::Spectator => return out.push(refused(conn, Refusal::NoSpectators)),
        };
        if self.seat_of(conn).is_some() {
            return out.push(refused(conn, Refusal::AlreadyJoined));
        }
        let held = &mut self.seats[side.index()];
        let SeatKind::Player { token: expected } = &held.kind else {
            return out.push(refused(conn, Refusal::SeatTaken));
        };
        let refusal = match token {
            Some(given) if !same_token(expected, &given) => Some(Refusal::BadToken),
            None if held.issued => Some(Refusal::SeatTaken),
            _ => None,
        };
        if let Some(reason) = refusal {
            return out.push(refused(conn, reason));
        }
        let token = Some(expected.clone());
        let taken = !core::mem::replace(&mut held.issued, true);
        if let Some(old) = held.holder.replace(conn) {
            out.push(goodbye(old, GoodbyeReason::Replaced));
        }
        if taken {
            self.entries.push(Entry::Taken(side));
        }
        out.push(reply(
            conn,
            &ServerMessage::Joined {
                game,
                seat,
                token,
                manifest: self.manifest.clone(),
            },
        ));
        let observation = map::observe(&self.state, self.ruleset, side);
        self.send(side, |n| ServerMessage::Snapshot { n, observation }, out);
        if !self.started && self.seats.iter().all(SeatState::held) {
            self.started = true;
            self.entries.push(Entry::Started);
            self.advance(out);
        } else if self.started && self.state.side_to_act() == Some(side) {
            self.your_turn(side, out);
        }
    }

    /// `Command`: the seat's command, applied with the seat's side.
    fn command(&mut self, conn: ConnId, seq: u32, command: Command, out: &mut Vec<Outbound>) {
        let Some(side) = self.seat_of(conn) else {
            return out.push(unseated(conn, seq));
        };
        if !self.started {
            return self.reject(side, seq, RejectCode::NotStarted, out);
        }
        let command = map::command_to_engine(command);
        match self.apply(side, command) {
            Err(error) => self.reject(side, seq, map::reject(error), out),
            Ok(events) => {
                let echo = map::command(command);
                self.send(
                    side,
                    |n| ServerMessage::Accepted {
                        n,
                        seq,
                        command: echo,
                    },
                    out,
                );
                self.tell(&events, out);
                self.advance(out);
            }
        }
    }

    /// `Query`: what the server's rules allow the seat now.
    fn query(&mut self, conn: ConnId, seq: u32, query: Query, out: &mut Vec<Outbound>) {
        let Some(side) = self.seat_of(conn) else {
            return out.push(unseated(conn, seq));
        };
        let targets = map::query(&self.state, self.ruleset, side, query);
        self.send(
            side,
            |n| ServerMessage::QueryResult { n, seq, targets },
            out,
        );
    }

    /// `FetchReplay`: the replay, for a seat once the game is over.
    fn fetch_replay(&mut self, conn: ConnId, seq: u32, out: &mut Vec<Outbound>) {
        let Some(side) = self.seat_of(conn) else {
            return out.push(unseated(conn, seq));
        };
        let over = self.state.is_over();
        match self.replay.as_ref().filter(|_| over).cloned() {
            Some(replay) => {
                let n = self.seats[side.index()].viewer.n;
                out.push(reply(conn, &ServerMessage::Replay { n, seq, replay }));
            }
            None => self.reject(side, seq, RejectCode::NotOver, out),
        }
    }

    /// `FetchRecord`: the seat's record, its header line and then one line per numbered message.
    fn fetch_record(&mut self, conn: ConnId, seq: u32, out: &mut Vec<Outbound>) {
        let Some(side) = self.seat_of(conn) else {
            return out.push(unseated(conn, seq));
        };
        let header = RecordHeader {
            format: RECORD_FORMAT_VERSION,
            game: self.game,
            seat: seat(side),
            manifest: self.manifest.clone(),
        };
        let viewer = &self.seats[side.index()].viewer;
        let mut record = codec::encode_text(&header);
        for line in &viewer.record {
            record.push('\n');
            record.push_str(line);
        }
        let n = viewer.n;
        out.push(reply(conn, &ServerMessage::Record { n, seq, record }));
    }

    /// Applies `command` for `side` and journals it; the events, if the engine accepted it.
    fn apply(
        &mut self,
        side: BattleSide,
        command: h3_battle::Command,
    ) -> Result<Vec<h3_battle::Event>, CommandError> {
        let mut events = Vec::new();
        h3_battle::apply(&mut self.state, self.ruleset, side, command, &mut events)?;
        self.journal
            .append(side, command, &self.state, &mut self.entries);
        Ok(events)
    }

    /// Every viewer gets `events` as it may see them.
    fn tell(&mut self, events: &[h3_battle::Event], out: &mut Vec<Outbound>) {
        for viewer in BattleSide::BOTH {
            let events = map::events(events, viewer);
            self.send(viewer, |n| ServerMessage::Events { n, events }, out);
        }
    }

    /// After the battle started or changed: bots act while it is their turn; then the seat to
    /// act gets its turn, or every viewer the end.
    fn advance(&mut self, out: &mut Vec<Outbound>) {
        self.run_bots(out);
        match self.state.side_to_act() {
            Some(side) => self.your_turn(side, out),
            None => {
                // Over: the result is set.
                let result = self.state.result().map_or(Outcome::Draw {}, map::outcome);
                for viewer in BattleSide::BOTH {
                    self.send(viewer, |n| ServerMessage::GameEnded { n, result }, out);
                }
            }
        }
    }

    /// Bots act while it is their side's turn. A choice the engine rejects is replaced by the
    /// first legal command that it accepts (it accepts every legal command, so a bot always
    /// moves; the loop ends when a player is to act or the battle is over).
    fn run_bots(&mut self, out: &mut Vec<Outbound>) {
        let mut legal = LegalCommands::new();
        let mut moved = true;
        while moved && let Some(side) = self.state.side_to_act() {
            let SeatKind::Bot(bot) = &mut self.seats[side.index()].kind else {
                return;
            };
            h3_battle::legal_commands(&self.state, self.ruleset, side, &mut legal);
            let choice = bot.command(&self.state, self.ruleset, side, &legal);
            let mut choices = core::iter::once(choice).chain(legal.iter().copied());
            moved = choices.any(|command| match self.apply(side, command) {
                Ok(events) => {
                    self.tell(&events, out);
                    true
                }
                Err(_) => false,
            });
        }
    }

    /// `YourTurn` for `side`: its legal commands and castable spells.
    fn your_turn(&mut self, side: BattleSide, out: &mut Vec<Outbound>) {
        let legal = map::legal_commands(&self.state, self.ruleset, side);
        let castable = map::castable(&self.state, self.ruleset, side);
        self.send(
            side,
            |n| ServerMessage::YourTurn { n, legal, castable },
            out,
        );
    }

    /// `Rejected` for the seat of `side`: nothing changed.
    fn reject(&mut self, side: BattleSide, seq: u32, code: RejectCode, out: &mut Vec<Outbound>) {
        self.send(side, |n| ServerMessage::Rejected { n, seq, code }, out);
    }

    /// Sends viewer `side` its next numbered message and records it; the message reaches a
    /// connection only if one holds the seat.
    fn send(
        &mut self,
        side: BattleSide,
        message: impl FnOnce(u32) -> ServerMessage,
        out: &mut Vec<Outbound>,
    ) {
        let seat = &mut self.seats[side.index()];
        seat.viewer.n = seat.viewer.n.saturating_add(1);
        let frame = codec::encode_text(&message(seat.viewer.n));
        if let Some(to) = seat.holder {
            out.push(Outbound {
                to,
                frame: frame.clone(),
                close: false,
            });
        }
        seat.viewer.record.push(frame);
    }

    /// The side whose seat `conn` holds.
    fn seat_of(&self, conn: ConnId) -> Option<BattleSide> {
        BattleSide::BOTH
            .into_iter()
            .find(|side| self.seats[side.index()].holder == Some(conn))
    }
}

/// A side's seat on the wire.
const fn seat(side: BattleSide) -> Seat {
    match side {
        BattleSide::Attacker => Seat::Attacker,
        BattleSide::Defender => Seat::Defender,
    }
}

/// Whether `given` is the seat's token; the time does not depend on where they differ.
fn same_token(expected: &str, given: &str) -> bool {
    expected.len() == given.len()
        && expected
            .bytes()
            .zip(given.bytes())
            .fold(0, |diff, (a, b)| diff | (a ^ b))
            == 0
}

/// An unnumbered message for `conn`.
fn reply(to: ConnId, message: &ServerMessage) -> Outbound {
    Outbound {
        to,
        frame: codec::encode_text(message),
        close: false,
    }
}

/// `Refused` for `conn`: the connection stays open.
fn refused(to: ConnId, reason: Refusal) -> Outbound {
    reply(to, &ServerMessage::Refused { reason })
}

/// `Rejected` for a connection that holds no seat: it is no viewer, so the number is 0.
fn unseated(to: ConnId, seq: u32) -> Outbound {
    reply(
        to,
        &ServerMessage::Rejected {
            n: 0,
            seq,
            code: RejectCode::NotSeated,
        },
    )
}

/// The last frame on `conn`.
fn goodbye(to: ConnId, reason: GoodbyeReason) -> Outbound {
    Outbound {
        close: true,
        ..reply(to, &ServerMessage::Goodbye { reason })
    }
}
