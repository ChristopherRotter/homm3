//! A seat the window plays on a server (E17 M5 T6b1; design §11.2): T6a's connection
//! ([`crate::net`], `h3_view::remote`) held for the window's battle ([`crate::battle`]). Only the
//! battle's loading waits on it, for the seat's record to open before the window shows
//! ([`Online::open`]); from then the window takes what came at each of its frames
//! ([`Online::poll`]) and never waits. A turn the server asks the seat to play asks the server's
//! rules for the targets of each spell the seat may cast and of its active stack's own cast (the
//! protocol's `Query`): the aims need them, as no engine runs here. A dropped connection is dialed
//! again (at most once a second, until the server answers) and the seat re-joined by its token;
//! the record fetched then extends the one before, so the battle keeps its timeline, or a
//! restarted server's opens afresh at the re-join's snapshot: a new timeline (E17 M5 T6b2). With
//! an account (E17 M5 T7, [`crate::account`]), each connection logs in once welcomed and joins
//! without a token (the seat is the account's); a goodbye after which the seat is not taken again
//! (`h3_view::remote::farewell`: the account re-keyed elsewhere, a void or abandoned game, the
//! seat taken by another connection) ends the run. The content check compiles the packs the game
//! names here: the same ruleset gives the window its own previews, another or none a visible
//! note. An adapter: the socket, the pins and accounts files, the clock.

use std::path::{Path, PathBuf};
use std::time::Duration;

use h3_content_battle::BattleRuleset;
use h3_protocol::map;
use h3_protocol::wire::{Command, Query, Seat, ServerMessage, Target};
use h3_view::record::Record;
use h3_view::remote::{self, Client, Content, Note, Pins, Rejoined, rejoined};
use rustls::RootCertStore;

use crate::account::Account;
use crate::connect::{self, Game, PROGRAM, hex, printable, read_pins};
use crate::net::{self, Connection, Incoming};

/// The least time between two dials of a dropped connection, in milliseconds.
const REDIAL_MS: u64 = 1_000;

/// Where and how a window plays its seat on a server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Server {
    /// The address dialed.
    pub address: String,
    /// The game joined, or the server's offer a game is created of.
    pub game: Game,
    /// The seat played (an attacker's or a defender's).
    pub seat: Seat,
    /// The seat's token (a seat taken anew needs none).
    pub token: Option<String>,
    /// The player's display name in the hello.
    pub name: String,
    /// The pins file.
    pub pins: PathBuf,
    /// Drops the connection after this many commands sent and re-joins (a test of the re-join).
    pub drop: Option<u32>,
    /// The account logged in to (E17 M5 T7); none: anonymous.
    pub account: Option<Account>,
}

/// The seat's connection and what it gathered (the module's text).
#[derive(Debug)]
pub struct Online {
    server: Server,
    roots: fn() -> RootCertStore,
    pins: Pins,
    client: Client,
    connection: Option<Connection>,
    /// When the connection was last dialed, by the caller's clock in milliseconds (none: it may
    /// be dialed at once).
    dialed: Option<u64>,
    /// The game joined: a re-join joins it.
    joined: Option<u32>,
    token: Option<String>,
    /// The seat's record as the window took it: the first connection's, extended by later ones'
    /// (none before it opened).
    record: Option<Record>,
    /// The content check, and the ruleset compiled here from the packs the game names (none:
    /// they did not compile).
    content: Content,
    ruleset: Option<BattleRuleset>,
    /// A command was sent for the turn and not answered yet.
    asked: bool,
    sent: u32,
    drop: Option<u32>,
    /// The turn the queries were asked for (the record's length at it), and each query's spell
    /// (none: the active stack's own cast) with its sequence number.
    queried: Option<usize>,
    queries: Vec<(Option<u16>, u32)>,
    /// How often the seat was re-joined.
    pub rejoined: u32,
    /// How often its record opened afresh ([`Online::timelines`]).
    timelines: u32,
    /// The connection dropped and no server welcomed the seat since.
    down: bool,
    /// What happened, a line each.
    pub lines: Vec<String>,
}

impl Online {
    /// Dials `server`, joins (or creates) its game and waits for the seat's record to open, each
    /// message at most `wait` after the one before; the content check compiles the game's packs
    /// from `packs`; `roots` checks a `wss://` server's certificate.
    ///
    /// # Errors
    /// The pins file, a fault, a refusal, a closed connection or a server silent for `wait`.
    pub fn open(
        server: Server,
        packs: &Path,
        roots: fn() -> RootCertStore,
        wait: Duration,
    ) -> Result<Self, String> {
        let pins = read_pins(&server.pins)?;
        let client = Client::new(&server.address, net::nonce()?, pins.key(&server.address));
        let connection = Connection::open(&server.address, roots);
        connection.send(client.hello(PROGRAM, &server.name));
        let mut online = Self {
            token: server.token.clone(),
            drop: server.drop,
            server,
            roots,
            pins,
            client,
            connection: Some(connection),
            dialed: None,
            joined: None,
            record: None,
            content: Content::Absent,
            ruleset: None,
            asked: false,
            sent: 0,
            queried: None,
            queries: Vec::new(),
            rejoined: 0,
            timelines: 0,
            down: false,
            lines: Vec::new(),
        };
        while online.record.is_none() {
            let connection = online.connection.as_ref().ok_or("no connection")?;
            match connection.wait(wait) {
                None => return Err(format!("nothing from the server in {wait:?}")),
                Some(Incoming::Closed(why)) => return Err(why),
                Some(Incoming::Message(message)) => online.receive(*message)?,
            }
        }
        let manifest = &online.record().ok_or("the record")?.header.manifest;
        let ruleset = crate::scenario::ruleset(packs, manifest).ok();
        let own = ruleset.as_ref().map(map::manifest);
        online.content = remote::content(manifest, own.as_ref());
        online.ruleset = ruleset;
        online.lines.push(format!("content: {:?}", online.content));
        Ok(online)
    }

    /// Takes what came since, without waiting, at `now` (the caller's clock in milliseconds):
    /// whether the seat's record grew. A dropped connection is dialed again ([`REDIAL_MS`] after
    /// the last dial) and the seat re-joined.
    ///
    /// # Errors
    /// A fault, a refusal, or a record fetched at a re-join that does not extend the one before.
    pub fn poll(&mut self, now: u64) -> Result<bool, String> {
        let before = self.taken();
        loop {
            let Some(connection) = self.connection.as_ref() else {
                self.redial(now);
                break;
            };
            match connection.poll() {
                None => break,
                Some(Incoming::Message(message)) => self.receive(*message)?,
                Some(Incoming::Closed(why)) => {
                    if self.over() || self.client.closed() {
                        self.connection = None;
                        break;
                    }
                    // Dialed again until the server answers (E17 M5 T6b2: a restart's
                    // downtime), the drop said once.
                    if !self.down {
                        self.lines
                            .push(format!("the connection dropped ({why}): re-joining"));
                        self.down = true;
                    }
                    self.connection = None;
                }
            }
        }
        self.ask_targets();
        Ok(self.taken() > before)
    }

    /// Dials again at `now` once [`REDIAL_MS`] passed since the last dial: a new client, the
    /// hello; the welcome's join re-joins the game by the seat's token.
    fn redial(&mut self, now: u64) {
        let early = self
            .dialed
            .is_some_and(|at| now.saturating_sub(at) < REDIAL_MS);
        if early || self.over() {
            return;
        }
        let Ok(nonce) = net::nonce() else {
            return;
        };
        self.client = Client::new(
            &self.server.address,
            nonce,
            self.pins.key(&self.server.address),
        );
        let connection = Connection::open(&self.server.address, self.roots);
        connection.send(self.client.hello(PROGRAM, &self.server.name));
        self.connection = Some(connection);
        self.dialed = Some(now);
        (self.asked, self.queried) = (false, None);
        self.queries.clear();
        self.rejoined += 1;
    }

    /// What the server's `message` does.
    fn receive(&mut self, message: ServerMessage) -> Result<(), String> {
        let notes = self
            .client
            .receive(message, net::verify)
            .map_err(|f| f.to_string())?;
        for note in notes {
            match note {
                Note::Send(message) => self.send_raw(message),
                Note::Pinned(key) => {
                    self.pins = connect::pin(&self.server.pins, &self.server.address, key)?;
                    self.lines
                        .push(format!("pinned the server's key {}", hex(&key)));
                }
                Note::Welcomed(server) => {
                    self.lines
                        .push(format!("welcomed by {}", printable(&server)));
                    self.down = false;
                    let ask = match &self.server.account {
                        Some(account) => account.login(&mut self.client, &self.server.address)?,
                        None => self.join()?,
                    };
                    self.send_raw(ask);
                }
                Note::LoggedIn(display) => {
                    self.lines
                        .push(format!("logged in as {}", printable(&display)));
                    // Registered: a re-dial logs in without the display name, which the server
                    // counts as an account change (rate-limited) at every login (E17 M5 review
                    // R1-2).
                    if let Some(account) = &mut self.server.account {
                        account.register = None;
                    }
                    let ask = self.join()?;
                    self.send_raw(ask);
                }
                Note::Rekeyed | Note::OldKeyRefused => {
                    self.lines.push(connect::rekeyed(&note));
                    if let Some(account) = &self.server.account {
                        account.done(&self.server.address)?;
                    }
                }
                Note::Created(game) => self.lines.push(format!("created game {game}")),
                Note::Joined(game, seat) => {
                    self.joined = Some(game);
                    self.lines.push(format!("joined game {game} as {seat:?}"));
                }
                Note::Refused(reason) => return Err(format!("refused: {reason:?}")),
                Note::Opened | Note::Grew => self.take()?,
                Note::Goodbye(reason) => {
                    self.lines.push(format!("goodbye: {reason:?}"));
                    if let Some(why) = remote::farewell(reason).filter(|_| !self.over()) {
                        return Err(why.to_owned());
                    }
                }
                Note::Other(_) => {}
            }
        }
        if let Some(token) = self.client.token() {
            self.token = Some(token.to_owned());
        }
        Ok(())
    }

    /// The join once welcomed (and logged in, with an account): the game joined before, or the
    /// server's game, or a new one of its offer.
    fn join(&mut self) -> Result<h3_protocol::wire::ClientMessage, String> {
        let (game, seat) = (&self.server.game, self.server.seat);
        connect::join(&mut self.client, game, self.joined, seat, &self.token)
    }

    /// The connection's record taken as the seat's: at the first opening as it is; at a
    /// re-join's, the one taken with what it adds when its numbers go on, else (E17 M5 T6b2) as
    /// a new timeline: a restarted server's restored game numbers afresh from the re-join's
    /// snapshot ([`Online::timelines`]). Another game's or seat's ends the run. A command
    /// answered, the next turn may be played.
    fn take(&mut self) -> Result<(), String> {
        let Some(record) = self.client.record() else {
            return Ok(());
        };
        let taken = match self.record.as_ref().map(|taken| rejoined(taken, record)) {
            None => record.clone(),
            Some(Rejoined::Extends(merged)) => *merged,
            Some(Rejoined::Afresh) => {
                self.timelines += 1;
                self.lines.push(format!(
                    "the record opens afresh at message {}: a new timeline",
                    record.header.first
                ));
                (self.queried, self.asked) = (None, false);
                self.queries.clear();
                record.clone()
            }
            Some(Rejoined::Other) => {
                return Err("the record fetched at the re-join is another game's".to_owned());
            }
        };
        if matches!(
            record.messages.last(),
            Some(ServerMessage::Accepted { .. } | ServerMessage::Rejected { .. })
        ) {
            self.asked = false;
        }
        self.record = Some(taken);
        Ok(())
    }

    /// How often the seat's record opened afresh (E17 M5 T6b2: a server's restart): each a new
    /// timeline, which the window's battle starts again from.
    pub const fn timelines(&self) -> u32 {
        self.timelines
    }

    fn send_raw(&self, message: h3_protocol::wire::ClientMessage) {
        if let Some(connection) = &self.connection {
            connection.send(message);
        }
    }

    /// Asks the server's rules for the targets of the turn's casts once per turn: each spell the
    /// hero may cast, and the active stack's own cast.
    fn ask_targets(&mut self) {
        let Some((_, castable)) = self.client.turn() else {
            return;
        };
        let turn = self.taken();
        if self
            .queried
            .is_some_and(|q| q >= self.turn_start().unwrap_or(turn))
        {
            return;
        }
        let castable = castable.to_vec();
        self.queries.clear();
        let asks = castable
            .into_iter()
            .map(|spell| (Some(spell), Query::CastTargets { spell }))
            .chain(std::iter::once((None, Query::CreatureCastTargets {})));
        for (spell, query) in asks {
            if let Some((seq, message)) = self.client.query(query) {
                self.send_raw(message);
                self.queries.push((spell, seq));
            }
        }
        self.queried = Some(turn);
    }

    /// Where the record's last `YourTurn` stands (its index).
    fn turn_start(&self) -> Option<usize> {
        self.record
            .as_ref()?
            .messages
            .iter()
            .rposition(|m| matches!(m, ServerMessage::YourTurn { .. }))
    }

    /// How many messages of the seat's record were taken.
    fn taken(&self) -> usize {
        self.record.as_ref().map_or(0, |r| r.messages.len())
    }

    /// The turn the server asks the seat to play (its legal commands and the spells its hero may
    /// cast), once every target the turn asked for came, while no command is sent for it.
    pub fn turn(&self) -> Option<(&[Command], &[u16])> {
        if self.asked || self.queried.is_none() {
            return None;
        }
        let answered = self
            .queries
            .iter()
            .all(|&(_, seq)| self.client.answer(seq).is_some());
        self.client.turn().filter(|_| answered)
    }

    /// The server's targets of the turn's cast of `spell` (none: the active stack's own).
    pub fn targets(&self, spell: Option<u16>) -> Option<&[Target]> {
        let &(_, seq) = self.queries.iter().find(|(s, _)| *s == spell)?;
        self.client.answer(seq)
    }

    /// Sends the seat's `command` for its turn; with `--drop`, the connection drops after the
    /// commands it names.
    pub fn send(&mut self, command: Command) -> Result<(), String> {
        let message = self
            .client
            .command(command)
            .ok_or("a command before the record")?;
        self.send_raw(message);
        (self.asked, self.sent) = (true, self.sent + 1);
        if self.drop.is_some_and(|after| self.sent >= after) {
            self.drop = None;
            self.lines.push(format!(
                "dropped the connection after {} commands",
                self.sent
            ));
            // Dropped, the I/O thread closes the socket; the next poll dials again at once.
            self.connection = None;
            self.dialed = None;
        }
        Ok(())
    }

    /// The seat's record as the window took it (none before it opened: [`Online::open`] waits
    /// for it).
    pub const fn record(&self) -> Option<&Record> {
        self.record.as_ref()
    }

    /// The seat played.
    pub const fn seat(&self) -> Seat {
        self.server.seat
    }

    /// The content check's verdict.
    pub const fn content(&self) -> Content {
        self.content
    }

    /// The ruleset compiled here from the game's packs (none: they did not compile), taken once
    /// by the battle's loading (`crate::battle::Ground`).
    pub const fn take_rules(&mut self) -> Option<BattleRuleset> {
        self.ruleset.take()
    }

    /// Whether the seat's record holds the game's end.
    pub fn over(&self) -> bool {
        self.record.as_ref().is_some_and(|r| {
            r.messages
                .iter()
                .any(|m| matches!(m, ServerMessage::GameEnded { .. }))
        })
    }
}

#[cfg(test)]
mod tests;
