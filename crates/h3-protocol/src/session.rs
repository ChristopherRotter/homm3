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
//! - **Accounts** (E16 M3 T3, ADR-0019 point 1). The host says which account a connection is
//!   logged in as ([`Session::logged_in`]). A seat such a connection takes belongs to its account:
//!   any connection logged in as it re-takes the seat, with no token, and no token takes it
//!   ([`refusal`]); its `Joined` carries none. A seat an anonymous connection takes keeps its
//!   token.
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
//! - **Spectators** (E16 M2 T4). A connection that joins as a spectator, up to the game's limit
//!   ([`GameConfig::spectators`]), is a viewer of its own for as long as it watches: the battle's
//!   spectator viewer (SRV-Q4) gives its snapshot and its events, what is public to both sides;
//!   it gets every viewer's messages (`Events`, `GameEnded`, `PauseAsked`), never a turn, a
//!   token or the replay, and whatever it sends for a seat is rejected `NotSeated`. Nothing of
//!   it is journaled: after a disconnect or a restart it joins again, numbered afresh.
//! - **The host's spectator setting** (E16 M2 T4c). Whether the game takes spectators at all is
//!   its host's ([`GameConfig::watchable`]): the host changes it with `SetSpectators` until the
//!   game starts, when the configuration is fixed; closing it ends every spectator's watch
//!   (`Goodbye { SpectatorsClosed }`). The setting is an entry at the start, not at each change:
//!   a game that never started is not restored, so only the setting it started with must last.
//! - **The journal.** The seed, the setup, the accepted commands with their sides and a state
//!   hash every `checkpoint_every` commands: the replay in progress (ADR-0018 point 12). The host
//!   renders it as the replay file at the end ([`Session::publish_replay`]).
//! - **Write-ahead** (E16 M2 T2). What the session did that a restart must redo comes out as
//!   [`Entry`]s, which the host drains after every call ([`Session::drain_entries`]) and makes
//!   durable before it sends that call's frames, so nothing a client was told is lost.
//!   [`Session::restore`] re-runs them after a restart: the same battle, the same seats taken, no
//!   seat held. A viewer's numbers and record start afresh there (they are not journaled).
//! - **Pause** (E16 M2 T3). In a running game the *host*, the seat taken first (a created game's
//!   creator), pauses it with `Pause`; another seat's ask is told to every viewer (`PauseAsked`),
//!   for the host to decide. A pause is an entry; every holder gets `Goodbye { Paused }`, and the
//!   host may then drop the session: a paused game is its journal. A seat's join with its token
//!   resumes a restored one ([`Session::paused`]), an entry too.
//! - **Ranked games** (E16 M4 T3a; ADR-0019 points 6 and 8; [`GameConfig::ranked`]). No host:
//!   `Pause` and `SetSpectators` are rejected `NotHost`. The host's clock ([`Session::elapse`])
//!   times each player's seat while no connection holds it, from the session's start: a stretch
//!   beyond the grace, or stretches beyond its total, forfeit (`GameEnded` with
//!   `Outcome::Forfeit`, the battle left where it stood); if the other seat is empty too, the game
//!   is void (`Outcome::NoShow`). A finished stretch is an entry, so a restore keeps the totals;
//!   a stretch running at a restart starts afresh with the restored session.
//! - **Integrity** (E16 M2 T3, `E16-design.md` §8). The host verifies a finished game before it
//!   publishes the replay, and withholds it if the game does not verify
//!   ([`Session::withhold_replay`]). With the *shadow instance* on ([`GameConfig::shadow`]), every
//!   command is applied to a second battle too and the two hashes compared; on a difference both
//!   are rebuilt from the journal, which is the truth, and the command applied again
//!   ([`Session::drain_mismatches`]).
//!
//! Provisional (E16 M1 T2, M2 T2–T4, M3 T3, M4 T3a): the names, the connection numbering, frames
//! as text, the record's layout, the entries, who pauses, a spectator's numbered rejections, a
//! seat's account fixed when it is first taken, one account in both seats allowed, a void game
//! when the other seat is empty as one's grace runs out (the epic's review list).

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use h3_battle::{
    BattleSetup, BattleSide, BattleState, CommandError, LegalCommands, NoopSink, SetupError,
};
use h3_content::Ruleset;

use crate::RECORD_FORMAT_VERSION;
use crate::codec;
use crate::map;
use crate::wire::{
    ClientMessage, Command, GoodbyeReason, Manifest, Outcome, Phase, Query, RecordHeader, Refusal,
    RejectCode, Seat, ServerMessage, Side,
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
    /// Whether a shadow instance checks the live battle after every command (E16 M2 T3).
    pub shadow: bool,
    /// How many spectators may watch at once (E16 M2 T4; 0: none).
    pub spectators: usize,
    /// Whether the game takes spectators: its creator's choice (E16 M2 T4c), which the host may
    /// change until the game starts. A restored game that started takes its journal's
    /// ([`Entry::Spectators`]).
    pub watchable: bool,
    /// The most a viewer's record holds, in bytes of its lines (E16 M2 T5c): beyond it the oldest
    /// messages go; the newest is always kept.
    pub record_bytes: usize,
    /// A ranked game's bounds of its players' absences (E16 M4 T3a): it has no host, and a seat
    /// empty beyond them forfeits. `None` for an unranked game.
    pub ranked: Option<Grace>,
}

/// How long a ranked game's seat may stay empty (E16 M4 T3a; ADR-0019 point 6), in milliseconds
/// of the host's clock ([`Session::elapse`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grace {
    /// The longest in one stretch.
    pub stretch_ms: u64,
    /// The longest over the game, its stretches added up.
    pub total_ms: u64,
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    /// The seat of the side was taken: its token was handed out, so it is re-taken only with it.
    Taken(BattleSide),
    /// The seat of the side was taken by a connection logged in as the account with this login
    /// name (E16 M3 T3): it belongs to the account, which alone re-takes it.
    TakenBy(BattleSide, String),
    /// Every seat was held at once: the game started.
    Started,
    /// The engine accepted the command from the side.
    Command(BattleSide, h3_battle::Command),
    /// The state's hash after the commands so far, every `checkpoint_every` commands.
    Checkpoint(u64),
    /// The host paused the game (E16 M2 T3).
    Paused,
    /// A seat was taken in the paused game: it runs again (E16 M2 T3).
    Resumed,
    /// Whether the game takes spectators, as it started (E16 M2 T4c): made just before
    /// [`Entry::Started`], since its host may change it until then.
    Spectators(bool),
    /// The game is ranked (E16 M4 T3a): its first entry, made by [`Session::new`].
    Ranked,
    /// The side's seat was empty for a stretch of this many milliseconds, which ended when a
    /// connection took it again (E16 M4 T3a): a restart keeps the total.
    Absent(BattleSide, u64),
    /// The side's seat stayed empty beyond the grace: its player forfeits (E16 M4 T3a). Both
    /// sides: the game is void, a no-show each.
    Forfeit(BattleSide),
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
#[derive(Debug, Clone, PartialEq, Eq)]
struct Viewer {
    /// The number of the last message sent.
    n: u32,
    /// The latest numbered messages sent, as their frames, the newest last: those numbered from
    /// `n - record.len() + 1` to `n`.
    record: VecDeque<String>,
    /// The bytes of the record's lines, each with its line break (E16 M2 T5c).
    bytes: usize,
    /// The most `bytes` may be; the oldest lines go first, the newest stays (E16 M2 T5c).
    limit: usize,
}

impl Viewer {
    /// A viewer with no message yet, whose record holds at most `limit` bytes.
    const fn new(limit: usize) -> Self {
        Self {
            n: 0,
            record: VecDeque::new(),
            bytes: 0,
            limit,
        }
    }

    /// Sends the viewer its next numbered message and records it; the frame goes out only if a
    /// connection `to` is there.
    fn post(
        &mut self,
        to: Option<ConnId>,
        message: impl FnOnce(u32) -> ServerMessage,
        out: &mut Vec<Outbound>,
    ) {
        self.n = self.n.saturating_add(1);
        let frame = codec::encode_text(&message(self.n));
        if let Some(to) = to {
            out.push(Outbound {
                to,
                frame: frame.clone(),
                close: false,
            });
        }
        self.bytes = self.bytes.saturating_add(frame.len() + 1);
        self.record.push_back(frame);
        while self.bytes > self.limit && self.record.len() > 1 {
            let oldest = self.record.pop_front().unwrap_or_default();
            self.bytes = self.bytes.saturating_sub(oldest.len() + 1);
        }
    }

    /// The number of the record's first line (one more than `n` while it holds none).
    fn first(&self) -> u32 {
        let held = u32::try_from(self.record.len()).unwrap_or(u32::MAX);
        self.n.saturating_sub(held).saturating_add(1)
    }

    /// Its record: the header line, then one line per numbered message it holds.
    fn record(&self, header: &RecordHeader) -> String {
        let mut record = codec::encode_text(header);
        for line in &self.record {
            record.push('\n');
            record.push_str(line);
        }
        record
    }
}

/// A spectator (E16 M2 T4): its connection and itself as a viewer.
#[derive(Debug)]
struct Watcher {
    conn: ConnId,
    viewer: Viewer,
}

/// A side's seat.
#[derive(Debug)]
struct SeatState {
    /// Who plays it.
    kind: SeatKind,
    /// Whether its token was handed out (the seat was taken once).
    issued: bool,
    /// The login name of the account it belongs to (E16 M3 T3): the account of the connection that
    /// took it first, if that one was logged in.
    account: Option<String>,
    /// The connection that holds it.
    holder: Option<ConnId>,
    /// The side as a viewer.
    viewer: Viewer,
    /// In a ranked game that is not over, since when no connection holds the player's seat (the
    /// host's clock; E16 M4 T3a).
    away: Option<u64>,
    /// The milliseconds of its finished stretches without a connection (E16 M4 T3a).
    absent: u64,
}

impl SeatState {
    /// Whether it is held: by a connection, or by its bot.
    const fn held(&self) -> bool {
        matches!(self.kind, SeatKind::Bot(_)) || self.holder.is_some()
    }
}

/// The shadow instance (E16 M2 T3): a second battle the commands are applied to in step with the
/// live one, and the battle as it began, to rebuild both from the journal.
#[derive(Debug)]
struct Shadow {
    state: BattleState,
    initial: BattleState,
}

/// What became of the replay once the game is over.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Published {
    /// Not yet anything.
    Not,
    /// The replay file's text.
    Replay(String),
    /// Withheld: the game did not verify (E16 M2 T3).
    Withheld,
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
    /// The replay, once the host published or withheld it.
    replay: Published,
    /// The side whose seat was taken first: the host, who pauses the game (E16 M2 T3).
    host: Option<BattleSide>,
    /// Whether the game paused and no seat was taken since.
    paused: bool,
    /// The shadow instance, if the game has one.
    shadow: Option<Box<Shadow>>,
    /// The steps at which the shadow instance disagreed, not drained yet.
    mismatches: Vec<usize>,
    /// The spectators, in the order they joined (E16 M2 T4).
    watchers: Vec<Watcher>,
    /// How many may watch at once.
    watchers_limit: usize,
    /// Whether the game takes spectators: the host's setting (E16 M2 T4c).
    watchable: bool,
    /// The most a viewer's record holds, in bytes (E16 M2 T5c).
    record_bytes: usize,
    /// The login names of the logged-in connections, as the host said (E16 M3 T3).
    logins: BTreeMap<ConnId, String>,
    /// A ranked game's grace (E16 M4 T3a).
    grace: Option<Grace>,
    /// The host's clock, in milliseconds since the session began (E16 M4 T3a).
    clock: u64,
    /// The sides whose players forfeited (E16 M4 T3a): the game is over.
    forfeited: [bool; 2],
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
        // A ranked game's player seats are empty from its start (E16 M4 T3a).
        let ranked = config.ranked.is_some();
        let seats = seats.map(|kind| SeatState {
            away: (ranked && matches!(kind, SeatKind::Player { .. })).then_some(0),
            kind,
            issued: false,
            account: None,
            holder: None,
            viewer: Viewer::new(config.record_bytes),
            absent: 0,
        });
        let shadow = config.shadow.then(|| {
            Box::new(Shadow {
                state: state.clone(),
                initial: state.clone(),
            })
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
            entries: config.ranked.map(|_| Entry::Ranked).into_iter().collect(),
            replay: Published::Not,
            host: None,
            paused: false,
            shadow,
            mismatches: Vec::new(),
            watchers: Vec::new(),
            watchers_limit: config.spectators,
            watchable: config.watchable,
            record_bytes: config.record_bytes,
            logins: BTreeMap::new(),
            grace: config.ranked,
            clock: 0,
            forfeited: [false; 2],
        })
    }

    /// Restores a game from its journal's `entries` after a restart (E16 M2 T2): hosts the battle
    /// of `config` with `seats` as [`Session::new`] does, then re-runs the entries in order (the
    /// seats taken, the start, the commands through the engine). The re-run makes its own entries,
    /// which the journal must begin: a checkpoint whose hash differs, or one out of place, is a
    /// divergence. No seat is held after it; a taken one is re-taken with its
    /// token. The first drained entries then are what the journal lacks: the checkpoint after its
    /// last command if a crash cut the journal between the two, and the bots' commands if a started
    /// game is theirs to act (a crash between a player's command and the bots' answers). A journal
    /// that ends with a pause restores a paused session ([`Session::paused`]).
    pub fn restore(
        ruleset: &'r Ruleset,
        config: GameConfig,
        seats: [SeatKind; 2],
        entries: &[Entry],
    ) -> Result<Self, RestoreError> {
        let mut session = Self::new(ruleset, config, seats).map_err(RestoreError::Setup)?;
        for (at, entry) in entries.iter().enumerate() {
            match *entry {
                Entry::Taken(side) => session.taken(side, None),
                Entry::TakenBy(side, ref login) => session.taken(side, Some(login.clone())),
                Entry::Started => {
                    session.started = true;
                    session.entries.push(Entry::Started);
                }
                Entry::Paused | Entry::Resumed => {
                    session.paused = *entry == Entry::Paused;
                    session.entries.push(entry.clone());
                }
                Entry::Spectators(open) => {
                    session.watchable = open;
                    session.entries.push(Entry::Spectators(open));
                }
                Entry::Absent(side, ms) => {
                    let seat = &mut session.seats[side.index()];
                    seat.absent = seat.absent.saturating_add(ms);
                    session.entries.push(Entry::Absent(side, ms));
                }
                Entry::Forfeit(side) => session.forfeit(side),
                Entry::Command(side, command) => {
                    session
                        .apply(side, command)
                        .map_err(|error| RestoreError::Rejected { at, error })?;
                }
                // The re-run's own follows its command, and a ranked game's is the first `new`
                // made; the two are compared below.
                Entry::Checkpoint(_) | Entry::Ranked => {}
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
        if session.started && !session.over() {
            session.run_bots(&mut Vec::new());
        }
        Ok(session)
    }

    /// Handles what connection `conn` sent after its hello; the frames to send go to `out`. The
    /// hello, the ping and the account's requests are the connection's ([`crate::connection`]),
    /// the lobby's messages the host's ([`crate::lobby`]); they are ignored here.
    pub fn handle(&mut self, conn: ConnId, message: ClientMessage, out: &mut Vec<Outbound>) {
        match message {
            ClientMessage::Join { game, seat, token } => self.join(conn, (game, seat), token, out),
            ClientMessage::Command { seq, command } => self.command(conn, seq, command, out),
            ClientMessage::Query { seq, query } => self.query(conn, seq, query, out),
            ClientMessage::FetchReplay { seq } => self.fetch_replay(conn, seq, out),
            ClientMessage::FetchRecord { seq } => self.fetch_record(conn, seq, out),
            ClientMessage::Pause { seq } => self.pause(conn, seq, out),
            ClientMessage::SetSpectators { seq, open } => self.set_spectators(conn, seq, open, out),
            ClientMessage::Leave {} => {
                self.disconnected(conn);
                out.push(goodbye(conn, GoodbyeReason::Left));
            }
            // The connection's (`crate::connection`: the hello, the ping, the account's) and the
            // lobby's (`crate::lobby`), not the game's.
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
            | ClientMessage::DeclineMatch { .. } => {}
        }
    }

    /// The host's word that connection `conn` is logged in as the account `login` (E16 M3 T3): a
    /// seat it takes belongs to the account, and it re-takes the account's seats. Call it before
    /// the connection's join; it changes nothing else.
    pub fn logged_in(&mut self, conn: ConnId, login: String) {
        self.logins.insert(conn, login);
    }

    /// The connection `conn` is gone: its seat waits for a re-take with the token, or by its
    /// account, and in a ranked game that is not over its absence runs from the host's clock (E16
    /// M4 T3a); a spectator stops watching.
    pub fn disconnected(&mut self, conn: ConnId) {
        let away = (self.grace.is_some() && !self.over()).then_some(self.clock);
        for seat in &mut self.seats {
            if seat.holder.take_if(|holder| *holder == conn).is_some() {
                seat.away = away;
            }
        }
        self.watchers.retain(|w| w.conn != conn);
        self.logins.remove(&conn);
    }

    /// The host's clock reads `now`, in milliseconds since the session began (E16 M4 T3a; a clock
    /// that goes back is ignored): call it before every other call, and while nothing else
    /// happens, often enough for the grace's precision. In a ranked game that is not over, a seat
    /// empty beyond the grace in one stretch, or beyond its total, forfeits; if the other seat is
    /// empty too, both do (void, a no-show each). Every viewer then hears `GameEnded` with the
    /// forfeit. Whether the game ended now.
    pub fn elapse(&mut self, now: u64, out: &mut Vec<Outbound>) -> bool {
        self.clock = self.clock.max(now);
        let clock = self.clock;
        let Some(grace) = self.grace.filter(|_| !self.over()) else {
            return false;
        };
        let beyond = self.seats.each_ref().map(|seat| {
            seat.away.is_some_and(|since| {
                let stretch = clock.saturating_sub(since);
                stretch > grace.stretch_ms || seat.absent.saturating_add(stretch) > grace.total_ms
            })
        });
        if beyond == [false; 2] {
            return false;
        }
        let away = self.seats.each_ref().map(|seat| seat.away.is_some());
        let sides = if away == [true; 2] { away } else { beyond };
        for side in BattleSide::BOTH {
            if sides[side.index()] {
                self.forfeit(side);
            }
        }
        let result = match sides {
            [true, true] => Outcome::NoShow {},
            [true, false] => Outcome::Forfeit {
                side: Side::Attacker,
            },
            _ => Outcome::Forfeit {
                side: Side::Defender,
            },
        };
        for viewer in BattleSide::BOTH {
            self.send(viewer, |n| ServerMessage::GameEnded { n, result }, out);
        }
        self.spectate(|n| ServerMessage::GameEnded { n, result }, out);
        true
    }

    /// The side's player forfeits (E16 M4 T3a): the game is over, and no absence counts any more.
    fn forfeit(&mut self, side: BattleSide) {
        self.forfeited[side.index()] = true;
        self.settle();
        self.entries.push(Entry::Forfeit(side));
    }

    /// The game is over: no seat's absence runs any more (E16 M4 T3a).
    fn settle(&mut self) {
        for seat in &mut self.seats {
            seat.away = None;
        }
    }

    /// Whether the game is over: its battle ended, or a player forfeited (E16 M4 T3a).
    pub fn over(&self) -> bool {
        self.state.is_over() || self.forfeited != [false; 2]
    }

    /// The sides whose players forfeited, the attacker's first (E16 M4 T3a): one, a loss for it;
    /// both, the game is void (a no-show each). The battle itself did not end.
    pub const fn forfeited(&self) -> [bool; 2] {
        self.forfeited
    }

    /// The login names of the accounts the attacker's and the defender's seat belong to (E16 M3
    /// T3): the host names them in the lobby, and their preferences decide the replay's listing.
    pub fn accounts(&self) -> [Option<&str>; 2] {
        self.seats.each_ref().map(|seat| seat.account.as_deref())
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
        if self.over() {
            Phase::Over
        } else if self.paused {
            Phase::Paused
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

    /// Whether a connection holds a seat (E16 M2 T5a): the host times a running game whose seats
    /// all stay empty.
    pub fn seated(&self) -> bool {
        self.seats.iter().any(|seat| seat.holder.is_some())
    }

    /// Whether connection `conn` holds a seat or watches (E16 M2 T5b): one that does neither after
    /// its join, a refused one, is in the game no more, and the host lets it go.
    pub fn holds(&self, conn: ConnId) -> bool {
        self.seat_of(conn).is_some() || self.watcher(conn).is_some()
    }

    /// Pauses the running game for the host when none of its seats is held (E16 M2 T5a: the
    /// deadline of an abandoned game), as its host's `Pause` would; `false`, and nothing changes,
    /// if the game does not run or a seat is held. Its spectators hear `Goodbye { Paused }`.
    pub fn suspend(&mut self, out: &mut Vec<Outbound>) -> bool {
        if self.phase() != Phase::Running || self.seated() {
            return false;
        }
        self.halt(out);
        true
    }

    /// Whether the game's host left it open to spectators (E16 M2 T4c): the setting itself, whatever
    /// the server's limit. A finished game that was is archived publicly (E16 M2 T5d).
    pub const fn watchable(&self) -> bool {
        self.watchable
    }

    /// Whether the game takes spectators now, for the lobby's list (E16 M2 T4c): its host's
    /// setting, a limit above 0, and not paused (a paused game takes none).
    pub const fn takes_spectators(&self) -> bool {
        self.watchable && self.watchers_limit > 0 && !self.paused
    }

    /// The replay file's text, for the participants once the game is over (ADR-0018 point 8): a
    /// `FetchReplay` before the end is refused even if the text is there.
    pub fn publish_replay(&mut self, replay: String) {
        self.replay = Published::Replay(replay);
    }

    /// The game did not verify (E16 M2 T3): its replay is withheld, and a `FetchReplay` after the
    /// end is rejected as `Unverified`.
    pub fn withhold_replay(&mut self) {
        self.replay = Published::Withheld;
    }

    /// Whether the game paused (E16 M2 T3), or a restored journal ended with its pause, and no
    /// seat was taken since: the host may drop the session, the journal is the game.
    pub const fn paused(&self) -> bool {
        self.paused
    }

    /// The steps at which the shadow instance disagreed with the live battle since the last call
    /// (E16 M2 T3), each the number of the command that found it: the host logs them.
    pub fn drain_mismatches(&mut self) -> alloc::vec::Drain<'_, usize> {
        self.mismatches.drain(..)
    }

    /// Replaces the live battle, as a fault in memory would (tests of the shadow instance).
    #[cfg(test)]
    pub(crate) fn corrupt(&mut self, state: BattleState) {
        self.state = state;
    }

    /// Replaces the shadow instance's battle, as a fault in memory would.
    #[cfg(test)]
    pub(crate) fn corrupt_shadow(&mut self, state: &BattleState) {
        let shadows = self.shadow.iter_mut();
        shadows.for_each(|shadow| shadow.state = state.clone());
    }

    /// `Join`: takes or re-takes a seat, or watches.
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
        if self.seat_of(conn).is_some() || self.watcher(conn).is_some() {
            return out.push(refused(conn, Refusal::AlreadyJoined));
        }
        let side = match seat {
            Seat::Attacker => BattleSide::Attacker,
            Seat::Defender => BattleSide::Defender,
            Seat::Spectator => return self.watch(conn, out),
        };
        let login = self.logins.get(&conn).cloned();
        let held = &mut self.seats[side.index()];
        let SeatKind::Player { token: expected } = &held.kind else {
            return out.push(refused(conn, Refusal::SeatTaken));
        };
        let claim = (expected.as_str(), held.issued, held.account.as_deref());
        if let Some(reason) = refusal(claim, token.as_deref(), login.as_deref()) {
            return out.push(refused(conn, reason));
        }
        // An account's seat is re-taken by the account, so its token is never handed out.
        let mine = held.account.is_some() || (!held.issued && login.is_some());
        let token = (!mine).then(|| expected.clone());
        if let Some(old) = held.holder.replace(conn) {
            out.push(goodbye(old, GoodbyeReason::Replaced));
        }
        let issued = held.issued;
        // A ranked game's seat is back: its stretch without a connection ends (E16 M4 T3a).
        if let Some(since) = held.away.take() {
            let ms = self.clock.saturating_sub(since);
            held.absent = held.absent.saturating_add(ms);
            self.entries.push(Entry::Absent(side, ms));
        }
        if !issued {
            self.taken(side, login);
        }
        if core::mem::take(&mut self.paused) {
            self.entries.push(Entry::Resumed);
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
        // Nothing more happens in a game that is over (a forfeit leaves its battle where it stood).
        if self.over() {
            return;
        }
        if !self.started && self.seats.iter().all(SeatState::held) {
            self.started = true;
            self.entries.push(Entry::Spectators(self.watchable));
            self.entries.push(Entry::Started);
            self.advance(out);
        } else if self.started && self.state.side_to_act() == Some(side) {
            self.your_turn(side, out);
        }
    }

    /// The seat of `side` belongs to the account `login` from the start (E16 M4 T2: a match's
    /// game): no token is handed out for it, and only a connection logged in as the account takes
    /// it. Does nothing to a seat taken already or a bot's.
    pub fn reserve(&mut self, side: BattleSide, login: String) {
        let seat = &self.seats[side.index()];
        if !seat.issued && matches!(seat.kind, SeatKind::Player { .. }) {
            self.taken(side, Some(login));
        }
    }

    /// The seat of `side` was taken the first time, by a connection logged in as `login` if any:
    /// its token handed out, or it belongs to the account (E16 M3 T3); the first seat taken is the
    /// host's, but for a ranked game, which has none (E16 M4 T3a).
    fn taken(&mut self, side: BattleSide, login: Option<String>) {
        let seat = &mut self.seats[side.index()];
        seat.issued = true;
        seat.account.clone_from(&login);
        if self.grace.is_none() {
            self.host.get_or_insert(side);
        }
        self.entries.push(match login {
            Some(login) => Entry::TakenBy(side, login),
            None => Entry::Taken(side),
        });
    }

    /// `Join` as a spectator (E16 M2 T4): in a game that takes spectators (T4c), within its
    /// limit, the spectator's snapshot.
    fn watch(&mut self, conn: ConnId, out: &mut Vec<Outbound>) {
        if !self.watchable || self.watchers.len() >= self.watchers_limit {
            return out.push(refused(conn, Refusal::NoSpectators));
        }
        out.push(reply(
            conn,
            &ServerMessage::Joined {
                game: self.game,
                seat: Seat::Spectator,
                token: None,
                manifest: self.manifest.clone(),
            },
        ));
        let observation = map::observe(&self.state, self.ruleset, h3_battle::Viewer::Spectator);
        let mut viewer = Viewer::new(self.record_bytes);
        viewer.post(
            Some(conn),
            |n| ServerMessage::Snapshot { n, observation },
            out,
        );
        self.watchers.push(Watcher { conn, viewer });
    }

    /// `Command`: the seat's command, applied with the seat's side.
    fn command(&mut self, conn: ConnId, seq: u32, command: Command, out: &mut Vec<Outbound>) {
        let Some(side) = self.seat_of(conn) else {
            return self.not_seated(conn, seq, out);
        };
        if !self.started {
            return self.reject(side, seq, RejectCode::NotStarted, out);
        }
        // Over by a forfeit, the battle did not end (E16 M4 T3a); over by its end, the engine
        // would say the same.
        if self.over() {
            return self.reject(side, seq, RejectCode::BattleOver, out);
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
            return self.not_seated(conn, seq, out);
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
            return self.not_seated(conn, seq, out);
        };
        let published = if self.over() {
            &self.replay
        } else {
            &Published::Not
        };
        match published.clone() {
            Published::Replay(replay) => {
                let n = self.seats[side.index()].viewer.n;
                out.push(reply(conn, &ServerMessage::Replay { n, seq, replay }));
            }
            Published::Withheld => self.reject(side, seq, RejectCode::Unverified, out),
            Published::Not => self.reject(side, seq, RejectCode::NotOver, out),
        }
    }

    /// `Pause`: the host's ask pauses a running game; another seat's is told to every viewer.
    fn pause(&mut self, conn: ConnId, seq: u32, out: &mut Vec<Outbound>) {
        let Some(side) = self.seat_of(conn) else {
            return self.not_seated(conn, seq, out);
        };
        let code = match self.phase() {
            Phase::Running => None,
            Phase::Over => Some(RejectCode::BattleOver),
            // A game whose seats are held is not paused.
            Phase::Waiting | Phase::Paused => Some(RejectCode::NotStarted),
        };
        if let Some(code) = code {
            return self.reject(side, seq, code, out);
        }
        // A ranked game has no host: it pauses only by agreement (ADR-0019 point 8; E16 M4 T6).
        if self.grace.is_some() {
            return self.reject(side, seq, RejectCode::NotHost, out);
        }
        // The host or every seat may pause (`E16-design.md` §7); the host holds one of the two
        // seats, so its ask decides, and another seat's is for the host to hear.
        if self.host != Some(side) {
            let seat = seat(side);
            for viewer in BattleSide::BOTH {
                self.send(viewer, |n| ServerMessage::PauseAsked { n, seat }, out);
            }
            return self.spectate(|n| ServerMessage::PauseAsked { n, seat }, out);
        }
        self.halt(out);
    }

    /// Pauses the game: an entry, and every holder and spectator hears `Goodbye { Paused }`.
    fn halt(&mut self, out: &mut Vec<Outbound>) {
        self.paused = true;
        self.entries.push(Entry::Paused);
        for held in &mut self.seats {
            if let Some(holder) = held.holder.take() {
                out.push(goodbye(holder, GoodbyeReason::Paused));
            }
        }
        for watcher in self.watchers.drain(..) {
            out.push(goodbye(watcher.conn, GoodbyeReason::Paused));
        }
    }

    /// `SetSpectators` (E16 M2 T4c): the host opens the game to spectators or closes it, until the
    /// game starts; closing it ends every spectator's watch. The host gets `SpectatorsSet`.
    fn set_spectators(&mut self, conn: ConnId, seq: u32, open: bool, out: &mut Vec<Outbound>) {
        let Some(side) = self.seat_of(conn) else {
            return self.not_seated(conn, seq, out);
        };
        let code = if self.started {
            Some(RejectCode::AlreadyStarted)
        } else if self.host != Some(side) {
            Some(RejectCode::NotHost)
        } else {
            None
        };
        if let Some(code) = code {
            return self.reject(side, seq, code, out);
        }
        self.watchable = open;
        if !open {
            for watcher in self.watchers.drain(..) {
                out.push(goodbye(watcher.conn, GoodbyeReason::SpectatorsClosed));
            }
        }
        self.send(side, |n| ServerMessage::SpectatorsSet { n, seq, open }, out);
    }

    /// `FetchRecord`: the viewer's record, its header line and then one line per numbered message;
    /// a spectator's since it joined.
    fn fetch_record(&mut self, conn: ConnId, seq: u32, out: &mut Vec<Outbound>) {
        let (seat, viewer) = match (self.seat_of(conn), self.watcher(conn)) {
            (Some(side), _) => (seat(side), &self.seats[side.index()].viewer),
            (None, Some(watcher)) => (Seat::Spectator, &watcher.viewer),
            (None, None) => return out.push(unseated(conn, seq)),
        };
        let header = RecordHeader {
            format: RECORD_FORMAT_VERSION,
            game: self.game,
            seat,
            manifest: self.manifest.clone(),
            first: viewer.first(),
        };
        let (n, record) = (viewer.n, viewer.record(&header));
        out.push(reply(conn, &ServerMessage::Record { n, seq, record }));
    }

    /// Applies `command` for `side` and journals it; the events, if the engine accepted it. With a
    /// shadow instance, the command is applied there too; if the two battles then differ (in their
    /// hashes, or in whether they accepted it), both are rebuilt from the journal, the truth, and
    /// the command is applied again (E16 M2 T3).
    fn apply(
        &mut self,
        side: BattleSide,
        command: h3_battle::Command,
    ) -> Result<Vec<h3_battle::Event>, CommandError> {
        let ruleset = self.ruleset;
        let mut events = Vec::new();
        let mut applied = h3_battle::apply(&mut self.state, ruleset, side, command, &mut events);
        if let Some(shadow) = &mut self.shadow {
            let again = h3_battle::apply(&mut shadow.state, ruleset, side, command, &mut NoopSink);
            if again.is_ok() != applied.is_ok()
                || shadow.state.state_hash() != self.state.state_hash()
            {
                let mut state = shadow.initial.clone();
                for &(side, command) in &self.journal.commands {
                    // Accepted once by both battles, so accepted again.
                    let _ = h3_battle::apply(&mut state, ruleset, side, command, &mut NoopSink);
                }
                shadow.state = state.clone();
                let _ = h3_battle::apply(&mut shadow.state, ruleset, side, command, &mut NoopSink);
                events.clear();
                applied = h3_battle::apply(&mut state, ruleset, side, command, &mut events);
                self.state = state;
                self.mismatches.push(self.journal.commands.len() + 1);
            }
        }
        applied?;
        self.journal
            .append(side, command, &self.state, &mut self.entries);
        Ok(events)
    }

    /// Every viewer gets `events` as it may see them; the spectators what is public.
    fn tell(&mut self, events: &[h3_battle::Event], out: &mut Vec<Outbound>) {
        for viewer in BattleSide::BOTH {
            let events = map::events(events, viewer);
            self.send(viewer, |n| ServerMessage::Events { n, events }, out);
        }
        if !self.watchers.is_empty() {
            let events = map::events(events, h3_battle::Viewer::Spectator);
            self.spectate(
                |n| ServerMessage::Events {
                    n,
                    events: events.clone(),
                },
                out,
            );
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
                self.settle();
                let result = self.state.result().map_or(Outcome::Draw {}, map::outcome);
                for viewer in BattleSide::BOTH {
                    self.send(viewer, |n| ServerMessage::GameEnded { n, result }, out);
                }
                self.spectate(|n| ServerMessage::GameEnded { n, result }, out);
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
        seat.viewer.post(seat.holder, message, out);
    }

    /// Sends every spectator its next numbered message (E16 M2 T4).
    fn spectate(&mut self, message: impl Fn(u32) -> ServerMessage, out: &mut Vec<Outbound>) {
        for watcher in &mut self.watchers {
            watcher.viewer.post(Some(watcher.conn), &message, out);
        }
    }

    /// `Rejected { NotSeated }` for a connection that holds no seat: numbered and recorded for a
    /// spectator, a viewer; numbered 0 for a connection in no seat and not watching, no viewer.
    fn not_seated(&mut self, conn: ConnId, seq: u32, out: &mut Vec<Outbound>) {
        let code = RejectCode::NotSeated;
        match self.watchers.iter_mut().find(|w| w.conn == conn) {
            Some(w) => w.viewer.post(
                Some(conn),
                |n| ServerMessage::Rejected { n, seq, code },
                out,
            ),
            None => out.push(unseated(conn, seq)),
        }
    }

    /// The side whose seat `conn` holds.
    fn seat_of(&self, conn: ConnId) -> Option<BattleSide> {
        BattleSide::BOTH
            .into_iter()
            .find(|side| self.seats[side.index()].holder == Some(conn))
    }

    /// The spectator `conn` is, if it watches.
    fn watcher(&self, conn: ConnId) -> Option<&Watcher> {
        self.watchers.iter().find(|w| w.conn == conn)
    }
}

/// A side's seat on the wire.
const fn seat(side: BattleSide) -> Seat {
    match side {
        BattleSide::Attacker => Seat::Attacker,
        BattleSide::Defender => Seat::Defender,
    }
}

/// Whether `given` is the seat's token `expected`; the time does not depend on where they differ
/// (the host checks a paused game's tokens with it, E16 M2 T3).
pub fn same_token(expected: &str, given: &str) -> bool {
    expected.len() == given.len()
        && expected
            .bytes()
            .zip(given.bytes())
            .fold(0, |diff, (a, b)| diff | (a ^ b))
            == 0
}

/// Why a connection logged in as `login` (or anonymous) may not take a player's seat with
/// `token`, or `None` if it may (E16 M3 T3): `claim` is the seat's token, whether it was handed
/// out, and the account the seat belongs to. An account's seat is its account's alone, whatever
/// the token; another seat is taken without a token the first time and re-taken with its token.
/// The host checks a paused game's seats with it before it restores the game.
pub fn refusal(
    (expected, issued, account): (&str, bool, Option<&str>),
    token: Option<&str>,
    login: Option<&str>,
) -> Option<Refusal> {
    match (account, token) {
        (Some(account), _) => (login != Some(account)).then_some(Refusal::SeatTaken),
        (None, Some(given)) => (!same_token(expected, given)).then_some(Refusal::BadToken),
        (None, None) => issued.then_some(Refusal::SeatTaken),
    }
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
