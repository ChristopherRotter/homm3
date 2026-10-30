//! A game on a server (E17 M5 T6a; the server lane's asks in `docs/lanes/client/NOTES.md`, "The
//! live client"; design §11.1): the client's side of a connection to `h3-server`, without its I/O.
//! The adapter dials the address, sends [`Client::hello`], hands every message the server sends to
//! [`Client::receive`] and sends what it returns; the cryptography is the adapter's (`verify`, as
//! `h3_protocol::card::read` takes it), and so is the nonce (drawn from the OS).
//!
//! - **The welcome** (ADR-0021 point 1): its protocol this client's (before 1.0 a server greets
//!   only its own), its `id` the address dialed (a login signs that id: a hostile server could
//!   relay it otherwise), its signature by its key over the id, the hello's nonce and the
//!   challenge (`account::welcome_message`), and its key the one pinned for the address ([`Pins`]:
//!   pinned at first contact, [`Note::Pinned`]; a key that changed ends the connection, as SSH
//!   warns).
//! - **The join** by a seat's token (a spectator's without; a seat not taken yet without, its
//!   token then the game's), or a game created and joined (`CreateGame`), then `FetchRecord`: the record is the
//!   viewer's as the server keeps it ([`crate::record`]), so a re-join after a drop has what it
//!   missed, and one after a server restart starts afresh, its numbers too. The numbered messages
//!   that come meanwhile are held, and those after the record's last added. The record opens at
//!   its first snapshot (a server's bound may have let its start go).
//! - **The record grows** by each numbered message, in order: a gap ends the connection.
//! - **Commands** carry the client's sequence numbers; whose turn it is, is the record's
//!   ([`Client::turn`]).
//! - **The content check** ([`content`]): the game's manifest against the one the client compiles
//!   from the packs and the profile it names.
//! - **The login** (E17 M5 T7, [`Client::login`]): once welcomed, signed for the address dialed
//!   (the welcome's id, checked above) and the welcome's challenge, by the account's key; after a
//!   password change, by the earlier key still due at this server ([`Accounts`]), then `Rekey`
//!   to the account's key, or by the account's key when the server holds the earlier one no more.
//!   A seat taken logged in is the account's: joined without a token, taken again after a drop by
//!   logging in and joining. A goodbye after which a seat is not taken again: [`farewell`].

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use h3_protocol::PROTOCOL_VERSION;
use h3_protocol::account::{self, CHALLENGE_BYTES, KEY_BYTES, NONCE_BYTES, SIGNATURE_BYTES};
use h3_protocol::codec;
use h3_protocol::wire::{
    ClientMessage, Command, GoodbyeReason, Manifest, Query, Refusal, Seat, ServerMessage, Target,
};

use crate::record::{self, Record, RecordError};

mod accounts;
mod seen;
#[cfg(test)]
mod tests;

pub use accounts::{Accounts, SEED_BYTES};
pub use seen::seen;

/// Where a connection stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    /// The hello sent, the welcome awaited.
    Greeting,
    /// Welcomed, no game joined.
    Lobby,
    /// A login sent at this step, its answer awaited (E17 M5 T7).
    LoggingIn(Step),
    /// A game asked for, to take this seat in.
    Creating(Seat),
    /// A join sent for this game and seat.
    Joining(u32, Seat),
    /// Joined this game and seat, the record asked for: the numbered messages are held.
    Fetching(u32, Seat),
    /// The record open, growing.
    Playing,
    /// The server said goodbye.
    Closed,
}

/// A login's step (E17 M5 T7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    /// By an earlier key of the account, still due at this server.
    Old,
    /// The re-key from it to the account's key.
    Rekey,
    /// By the account's key.
    Key,
}

/// An account a connection logs in to (E17 M5 T7): its login name, its public key, the public key
/// of an earlier one still due at this server (a password changed since: [`Accounts::due`]), and
/// the display name a registration asks for. The adapter holds the keys: [`Client::login`] has it
/// sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Login<'a> {
    /// The login name ([`account::is_login_name`]).
    pub login: &'a str,
    /// The account's public key.
    pub key: [u8; KEY_BYTES],
    /// An earlier key's, still due at this server.
    pub old: Option<[u8; KEY_BYTES]>,
    /// The display name to register the account with; none to log in to it.
    pub display: Option<&'a str>,
}

/// Which of a login's keys signs (E17 M5 T7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signer {
    /// The account's key.
    Key,
    /// The earlier key due at this server.
    Old,
}

/// The client's side of one connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Client {
    /// The address dialed.
    address: String,
    /// The hello's nonce.
    nonce: [u8; NONCE_BYTES],
    /// The key pinned for the address, if any.
    pinned: Option<[u8; KEY_BYTES]>,
    phase: Phase,
    /// The last sequence number sent.
    seq: u32,
    /// The seat's token, which takes it again after a drop.
    token: Option<String>,
    /// The numbered messages that came while the record was asked for, with their numbers.
    held: Vec<(u32, ServerMessage)>,
    record: Option<Record>,
    /// The number of the record's last message.
    last: u32,
    /// The answers to this connection's queries, by their sequence numbers (E17 M5 T6b1).
    answers: Vec<(u32, Vec<Target>)>,
    /// The welcome's challenge, which a login signs (E17 M5 T7).
    challenge: Option<[u8; CHALLENGE_BYTES]>,
    /// A login's messages still to send (E17 M5 T7): after one by an earlier key, the re-key and
    /// the login by the account's key, one of which follows.
    later: Vec<ClientMessage>,
    /// The account's display name once logged in (E17 M5 T7).
    account: Option<String>,
}

/// What a message from the server did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Note {
    /// A message to send the server.
    Send(ClientMessage),
    /// The welcome held: the server program's name.
    Welcomed(String),
    /// The server's key, pinned for the address at this first contact: the adapter keeps it.
    Pinned([u8; KEY_BYTES]),
    /// The game created (`CreateGame`'s answer); its join follows.
    Created(u32),
    /// The game and the seat joined; the record is asked for.
    Joined(u32, Seat),
    /// A request refused (a join's: no such game, a wrong token, a seat taken…).
    Refused(Refusal),
    /// The record opened afresh ([`Client::record`]), at a join or a re-join.
    Opened,
    /// The record grew by one message, at its end.
    Grew,
    /// The server's last message: the connection closes.
    Goodbye(GoodbyeReason),
    /// Logged in (E17 M5 T7): the account's display name on this server. A game may be joined.
    LoggedIn(String),
    /// The account re-keyed on this server from the earlier key due here to its key (E17 M5 T7):
    /// the earlier key is due here no more.
    Rekeyed,
    /// The server holds not the earlier key due here (E17 M5 T7: re-keyed from another device, or
    /// never registered there): due here no more; the login by the account's key follows.
    OldKeyRefused,
    /// A message this client does not act on (a lobby's answer, a pong, the rejection of a
    /// connection without a seat).
    Other(Box<ServerMessage>),
}

/// Why a connection ends on the client's side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fault {
    /// The welcome's protocol version is not this client's.
    Protocol(u32),
    /// The welcome's id is not the address dialed.
    NotDialed(String),
    /// The welcome's key, challenge or signature is not lowercase hex of its size.
    Malformed,
    /// The welcome's signature does not verify by its key.
    Signature,
    /// The address's key is not the one pinned: another server answers there, or its key changed.
    KeyChanged {
        /// The key pinned.
        pinned: [u8; KEY_BYTES],
        /// The welcome's.
        offered: [u8; KEY_BYTES],
    },
    /// A message the connection's state does not expect, by its kind.
    OutOfPlace(String),
    /// The record does not read.
    Record(RecordError),
    /// The record is not the joined seat's (its game or its seat).
    NotTheSeat,
    /// The record holds no snapshot to open at.
    NoSnapshot,
    /// A numbered message out of order.
    Gap {
        /// The number expected.
        expected: u32,
        /// The one that came.
        got: u32,
    },
}

impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Protocol(v) => write!(
                f,
                "the server speaks protocol {v}, this client {PROTOCOL_VERSION}"
            ),
            Self::NotDialed(id) => write!(f, "the welcome names {id}, not the address dialed"),
            Self::Malformed => write!(f, "the welcome's key, challenge or signature is malformed"),
            Self::Signature => write!(f, "the welcome's signature does not verify"),
            Self::KeyChanged { pinned, offered } => write!(
                f,
                "THE SERVER'S KEY CHANGED: pinned {}, offered {}; another server may answer at \
                 this address (remove its pin only if its operator confirms the new key)",
                account::to_hex(pinned),
                account::to_hex(offered)
            ),
            Self::OutOfPlace(kind) => write!(f, "a message out of place: {kind}"),
            Self::Record(e) => write!(f, "the record: {e}"),
            Self::NotTheSeat => write!(f, "the record is not the joined seat's"),
            Self::NoSnapshot => write!(f, "the record holds no snapshot"),
            Self::Gap { expected, got } => {
                write!(f, "message {got} came where {expected} was expected")
            }
        }
    }
}

impl Client {
    /// A connection to `address` (the URL dialed, `ws://…` or `wss://…`), its hello naming `nonce`
    /// (fresh: the adapter draws it from the OS), the address's key `pinned` if any.
    pub fn new(address: &str, nonce: [u8; NONCE_BYTES], pinned: Option<[u8; KEY_BYTES]>) -> Self {
        Self {
            address: String::from(address),
            nonce,
            pinned,
            phase: Phase::Greeting,
            seq: 0,
            token: None,
            held: Vec::new(),
            record: None,
            last: 0,
            answers: Vec::new(),
            challenge: None,
            later: Vec::new(),
            account: None,
        }
    }

    /// Logs in to `login`'s account (E17 M5 T7) once welcomed and before a login or a game; none
    /// otherwise. `sign` signs by one of its keys: the login's bytes
    /// (`account::login_message`: the address dialed, which the welcome's id is, the welcome's
    /// challenge, the login name, a registration's display name) by the account's key; with an
    /// earlier key due here, the login by that one (no registration), then the re-key's bytes
    /// (`account::rekey_message`) by both, and the login by the account's key held in case the
    /// server refuses the earlier one. The first message.
    pub fn login(
        &mut self,
        login: &Login<'_>,
        sign: impl Fn(Signer, &[u8]) -> [u8; SIGNATURE_BYTES],
    ) -> Option<ClientMessage> {
        let challenge = self
            .challenge
            .filter(|_| self.phase == Phase::Lobby && self.account.is_none())?;
        let address = self.address.clone();
        let by = |seq, signer, key: &[u8; KEY_BYTES], display: Option<&str>| {
            let signed = account::login_message(&address, &challenge, login.login, display);
            ClientMessage::Login {
                seq,
                login: String::from(login.login),
                key: account::to_hex(key),
                signature: account::to_hex(&sign(signer, &signed)),
                display: display.map(String::from),
            }
        };
        let Some(old) = login.old else {
            self.phase = Phase::LoggingIn(Step::Key);
            return Some(by(self.next_seq(), Signer::Key, &login.key, login.display));
        };
        let first = by(self.next_seq(), Signer::Old, &old, None);
        let rekeyed = account::rekey_message(&address, &challenge, login.login, &login.key);
        let rekey = ClientMessage::Rekey {
            seq: self.next_seq(),
            key: account::to_hex(&login.key),
            by_old: account::to_hex(&sign(Signer::Old, &rekeyed)),
            by_new: account::to_hex(&sign(Signer::Key, &rekeyed)),
        };
        let current = by(self.next_seq(), Signer::Key, &login.key, login.display);
        self.later = vec![rekey, current];
        self.phase = Phase::LoggingIn(Step::Old);
        Some(first)
    }

    /// The account's display name on this server once logged in (E17 M5 T7).
    pub fn account(&self) -> Option<&str> {
        self.account.as_deref()
    }

    /// The hello: this protocol version, the program `client`, the player's display name `name`,
    /// the nonce and the address dialed.
    pub fn hello(&self, client: &str, name: &str) -> ClientMessage {
        ClientMessage::Hello {
            protocol: PROTOCOL_VERSION,
            client: String::from(client),
            name: String::from(name),
            nonce: account::to_hex(&self.nonce),
            address: self.address.clone(),
        }
    }

    /// Joins `game` as `seat` by `token` (none: a spectator), once welcomed and before a game is
    /// joined; none otherwise.
    pub fn join(&mut self, game: u32, seat: Seat, token: Option<String>) -> Option<ClientMessage> {
        if self.phase != Phase::Lobby {
            return None;
        }
        self.phase = Phase::Joining(game, seat);
        self.token.clone_from(&token);
        Some(ClientMessage::Join { game, seat, token })
    }

    /// Creates a game of the server's offer `offer` and takes `seat` in it, once welcomed and
    /// before a game is joined; none otherwise.
    pub fn create(&mut self, offer: &str, seat: Seat) -> Option<ClientMessage> {
        if self.phase != Phase::Lobby {
            return None;
        }
        self.phase = Phase::Creating(seat);
        self.token = None;
        Some(ClientMessage::CreateGame {
            seq: self.next_seq(),
            offer: String::from(offer),
            seat,
            spectators: None,
        })
    }

    /// Sends `command` for the seat, once its record is open; none otherwise.
    pub fn command(&mut self, command: Command) -> Option<ClientMessage> {
        (self.phase == Phase::Playing).then(|| ClientMessage::Command {
            seq: self.next_seq(),
            command,
        })
    }

    /// Asks the server's rules `query` for the seat (E17 M5 T6b1: the targets of a spell it may
    /// cast), once its record is open; none otherwise. The message, and its sequence number, which
    /// the answer names ([`Client::answer`]).
    pub fn query(&mut self, query: Query) -> Option<(u32, ClientMessage)> {
        (self.phase == Phase::Playing).then(|| {
            let seq = self.next_seq();
            (seq, ClientMessage::Query { seq, query })
        })
    }

    /// The answer to this connection's query numbered `seq`, once it came: the targets the
    /// server's rules allow.
    pub fn answer(&self, seq: u32) -> Option<&[Target]> {
        self.answers
            .iter()
            .find(|(s, _)| *s == seq)
            .map(|(_, targets)| targets.as_slice())
    }

    /// The record, once open.
    pub const fn record(&self) -> Option<&Record> {
        self.record.as_ref()
    }

    /// The seat's token, which takes it again after a drop (the join's, or the one `Joined` gave).
    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    /// Whether the server said goodbye.
    pub fn closed(&self) -> bool {
        self.phase == Phase::Closed
    }

    /// The turn the seat is asked to play: the record's last `YourTurn` (its legal commands and
    /// the spells its hero may cast), unless a command was accepted or the game ended after it.
    pub fn turn(&self) -> Option<(&[Command], &[u16])> {
        for message in self.record.as_ref()?.messages.iter().rev() {
            match message {
                ServerMessage::YourTurn {
                    legal, castable, ..
                } => return Some((legal, castable)),
                ServerMessage::Accepted { .. } | ServerMessage::GameEnded { .. } => return None,
                _ => {}
            }
        }
        None
    }

    /// Whether the record holds the game's end.
    pub fn over(&self) -> bool {
        self.record.as_ref().is_some_and(|r| {
            r.messages
                .iter()
                .any(|m| matches!(m, ServerMessage::GameEnded { .. }))
        })
    }

    /// What the server's `message` does; `verify` checks a signature (the key, the bytes signed,
    /// the signature). A fault ends the connection.
    ///
    /// # Errors
    /// The [`Fault`] that ends it.
    pub fn receive(
        &mut self,
        message: ServerMessage,
        verify: impl Fn(&[u8; KEY_BYTES], &[u8], &[u8; SIGNATURE_BYTES]) -> bool,
    ) -> Result<Vec<Note>, Fault> {
        match (self.phase, message) {
            (Phase::Closed, m) => Err(out_of_place(&m)),
            (_, ServerMessage::Goodbye { reason }) => {
                self.phase = Phase::Closed;
                Ok(vec![Note::Goodbye(reason)])
            }
            (
                Phase::Greeting,
                ServerMessage::Welcome {
                    protocol,
                    server,
                    id,
                    challenge,
                    key,
                    signature,
                },
            ) => self.welcome(
                (protocol, server, id),
                [&challenge, &key, &signature],
                verify,
            ),
            (Phase::Greeting, m) => Err(out_of_place(&m)),
            (Phase::LoggingIn(step), ServerMessage::Account { display, .. }) => {
                Ok(self.logged_in(step, display))
            }
            (Phase::LoggingIn(step), ServerMessage::Refused { reason }) => {
                Ok(self.login_refused(step, reason))
            }
            (
                Phase::Joining(game, seat),
                ServerMessage::Joined {
                    game: g,
                    seat: s,
                    token,
                    ..
                },
            ) if (g, s) == (game, seat) => {
                if token.is_some() {
                    self.token = token;
                }
                self.phase = Phase::Fetching(game, seat);
                self.held.clear();
                let seq = self.next_seq();
                Ok(vec![
                    Note::Joined(game, seat),
                    Note::Send(ClientMessage::FetchRecord { seq }),
                ])
            }
            (Phase::Creating(seat), ServerMessage::GameCreated { game, .. }) => {
                self.phase = Phase::Joining(game, seat);
                Ok(vec![Note::Created(game)])
            }
            (phase, ServerMessage::Refused { reason }) => {
                if matches!(phase, Phase::Joining(..) | Phase::Creating(..)) {
                    self.phase = Phase::Lobby;
                }
                Ok(vec![Note::Refused(reason)])
            }
            (Phase::Fetching(game, seat), ServerMessage::Record { n, record, .. }) => {
                self.open((game, seat), n, &record)
            }
            (
                _,
                m @ (ServerMessage::Welcome { .. }
                | ServerMessage::GameCreated { .. }
                | ServerMessage::Joined { .. }
                | ServerMessage::Record { .. }),
            ) => Err(out_of_place(&m)),
            (_, m) => self.numbered(m),
        }
    }

    /// The welcome's checks: its protocol, its id (the address dialed), its signature by its key
    /// over the id, the nonce and the challenge, and its key the one pinned.
    fn welcome(
        &mut self,
        (protocol, server, id): (u32, String, String),
        [challenge, key, signature]: [&str; 3],
        verify: impl Fn(&[u8; KEY_BYTES], &[u8], &[u8; SIGNATURE_BYTES]) -> bool,
    ) -> Result<Vec<Note>, Fault> {
        if protocol != PROTOCOL_VERSION {
            return Err(Fault::Protocol(protocol));
        }
        if id != self.address {
            return Err(Fault::NotDialed(id));
        }
        let hex = (
            account::from_hex::<CHALLENGE_BYTES>(challenge),
            account::from_hex::<KEY_BYTES>(key),
            account::from_hex::<SIGNATURE_BYTES>(signature),
        );
        let (Some(challenge), Some(key), Some(signature)) = hex else {
            return Err(Fault::Malformed);
        };
        let signed = account::welcome_message(&id, &self.nonce, &challenge);
        if !verify(&key, &signed, &signature) {
            return Err(Fault::Signature);
        }
        let mut notes = Vec::new();
        match self.pinned {
            Some(pinned) if pinned != key => {
                return Err(Fault::KeyChanged {
                    pinned,
                    offered: key,
                });
            }
            Some(_) => {}
            None => {
                self.pinned = Some(key);
                notes.push(Note::Pinned(key));
            }
        }
        self.phase = Phase::Lobby;
        self.challenge = Some(challenge);
        notes.push(Note::Welcomed(server));
        Ok(notes)
    }

    /// A login's answer at `step` (E17 M5 T7): logged in as `display`; by an earlier key, the
    /// re-key follows.
    fn logged_in(&mut self, step: Step, display: String) -> Vec<Note> {
        self.account = Some(display.clone());
        match step {
            Step::Old => {
                self.phase = Phase::LoggingIn(Step::Rekey);
                let rekey = self.later.drain(..).next();
                rekey.map(Note::Send).into_iter().collect()
            }
            Step::Rekey => {
                self.phase = Phase::Lobby;
                vec![Note::Rekeyed, Note::LoggedIn(display)]
            }
            Step::Key => {
                self.phase = Phase::Lobby;
                vec![Note::LoggedIn(display)]
            }
        }
    }

    /// A login refused at `step` for `reason` (E17 M5 T7): by an earlier key the server does not
    /// hold, the login by the account's key follows; else [`Note::Refused`], on which the adapters
    /// end the run (E17 M5 review R1-8: a re-key refused after a login by an earlier key too; that
    /// key stays due there).
    fn login_refused(&mut self, step: Step, reason: Refusal) -> Vec<Note> {
        let current = self.later.pop();
        self.later.clear();
        if (step, reason) == (Step::Old, Refusal::BadLogin) {
            self.phase = Phase::LoggingIn(Step::Key);
            let send = current.map(Note::Send);
            return core::iter::once(Note::OldKeyRefused).chain(send).collect();
        }
        self.phase = Phase::Lobby;
        vec![Note::Refused(reason)]
    }

    /// The record `text` the server sent for the joined `game` and `seat`, its last message's
    /// number `n`, opened at its first snapshot; the messages held meanwhile after it added.
    fn open(&mut self, (game, seat): (u32, Seat), n: u32, text: &str) -> Result<Vec<Note>, Fault> {
        let mut record = record::parse(text).map_err(Fault::Record)?;
        if (record.header.game, record.header.seat) != (game, seat) {
            return Err(Fault::NotTheSeat);
        }
        let (start, first) = (record.messages.iter().enumerate())
            .find_map(|(i, m)| match m {
                ServerMessage::Snapshot { n, .. } => Some((i, *n)),
                _ => None,
            })
            .ok_or(Fault::NoSnapshot)?;
        record.messages.drain(..start);
        record.header.first = first;
        let mut last = n;
        for (k, message) in core::mem::take(&mut self.held) {
            if k > n {
                grow(&mut record, &mut last, k, message)?;
            }
        }
        self.record = Some(record);
        self.phase = Phase::Playing;
        self.last = last;
        Ok(vec![Note::Opened])
    }

    /// A message other than the connection's and the join's: held while the record is asked for,
    /// added to it once open, if numbered.
    fn numbered(&mut self, message: ServerMessage) -> Result<Vec<Note>, Fault> {
        let Some(n) = number(&message) else {
            return Ok(vec![Note::Other(Box::new(message))]);
        };
        match (self.phase, self.record.as_mut()) {
            (Phase::Fetching(..), _) => {
                self.held.push((n, message));
                Ok(Vec::new())
            }
            (Phase::Playing, Some(record)) => {
                if let ServerMessage::QueryResult { seq, targets, .. } = &message {
                    self.answers.push((*seq, targets.clone()));
                }
                grow(record, &mut self.last, n, message)?;
                Ok(vec![Note::Grew])
            }
            _ => Err(out_of_place(&message)),
        }
    }

    /// The next sequence number.
    const fn next_seq(&mut self) -> u32 {
        self.seq = self.seq.saturating_add(1);
        self.seq
    }
}

/// Adds message `n` to `record`, whose last message is numbered `last`: the next number only.
fn grow(record: &mut Record, last: &mut u32, n: u32, message: ServerMessage) -> Result<(), Fault> {
    let expected = last.saturating_add(1);
    if n != expected {
        return Err(Fault::Gap { expected, got: n });
    }
    *last = n;
    record.messages.push(message);
    Ok(())
}

/// The number a server message takes among the viewer's (the record's lines); none for one that
/// takes none (a `Rejected` numbered 0 is a connection's without a seat).
pub fn number(message: &ServerMessage) -> Option<u32> {
    match message {
        ServerMessage::Snapshot { n, .. }
        | ServerMessage::YourTurn { n, .. }
        | ServerMessage::Accepted { n, .. }
        | ServerMessage::QueryResult { n, .. }
        | ServerMessage::Events { n, .. }
        | ServerMessage::GameEnded { n, .. }
        | ServerMessage::PauseProposed { n, .. }
        | ServerMessage::PauseAnswered { n, .. }
        | ServerMessage::PauseAsked { n, .. }
        | ServerMessage::SpectatorsSet { n, .. }
        | ServerMessage::Rating { n, .. } => Some(*n),
        ServerMessage::Rejected { n, .. } => (*n > 0).then_some(*n),
        _ => None,
    }
}

/// Why a seat is not taken again after the server's goodbye `reason` (E17 M5 T7, *provisional*):
/// the account re-keyed elsewhere, a void game, an abandoned one, the seat taken by another
/// connection, another protocol version; none when the adapter dials again (a drop, a shutdown: a
/// restart's).
pub const fn farewell(reason: GoodbyeReason) -> Option<&'static str> {
    match reason {
        GoodbyeReason::Rekeyed => Some(
            "the account was re-keyed on another connection: the key kept here is an earlier \
             one (`h3-view account <login>` with the new password)",
        ),
        GoodbyeReason::CommandLimit => {
            Some("the game is void: it reached the server's limit of commands without an end")
        }
        GoodbyeReason::Abandoned => {
            Some("the game is gone: it had not started and none of its seats was held")
        }
        GoodbyeReason::Replaced => Some("another connection took the seat"),
        GoodbyeReason::Protocol => Some("the server speaks another protocol version"),
        _ => None,
    }
}

/// The fault of `message` out of place, named by its kind (its JSON's tag).
fn out_of_place(message: &ServerMessage) -> Fault {
    let text = codec::encode_text(message);
    Fault::OutOfPlace(String::from(text.split('"').nth(1).unwrap_or_default()))
}

/// Whether the client's own content is the game's (the content check, `docs/epics/E16-design.md`
/// §3): equal, its own previews (damage ranges, reachable hexes) are safe; otherwise the server's
/// queries only, and a visible note. The server never learns it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Content {
    /// The same ruleset: both hashes equal.
    Same,
    /// Compiled here, but not the game's ruleset.
    Different,
    /// Not compiled here: the packs are missing, or did not compile with the profile.
    Absent,
}

/// The content check of the game's manifest `game` against `own`, the manifest of the ruleset the
/// client compiled from the packs and the profile `game` names (none: it could not).
pub fn content(game: &Manifest, own: Option<&Manifest>) -> Content {
    match own {
        None => Content::Absent,
        Some(own)
            if (own.ruleset_hash, own.battle_ruleset_hash)
                == (game.ruleset_hash, game.battle_ruleset_hash) =>
        {
            Content::Same
        }
        Some(_) => Content::Different,
    }
}

/// How a record fetched at a re-join stands to the one taken before it (E17 M5 T6b2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rejoined {
    /// Its numbers go on from the one taken (the same server's: a drop): the one taken with what
    /// the fetched one adds. The server's bound may have let the fetched one's start go, so it
    /// may start after the taken one's first message; where both hold a message, it is the same.
    Extends(Box<Record>),
    /// The same game's and seat's, its numbers afresh or past a gap: a restarted server's
    /// restored game, opened at the re-join's snapshot (`docs/lanes/client/NOTES.md`, "The live
    /// client", 2026-10-11), or one whose bound let go of what was taken. A new timeline.
    Afresh,
    /// Another game's, seat's or content's.
    Other,
}

/// How `fetched`, the record fetched at a re-join, stands to `taken`, the one taken before.
pub fn rejoined(taken: &Record, fetched: &Record) -> Rejoined {
    let (t, f) = (&taken.header, &fetched.header);
    if (t.format, t.game, t.seat, &t.manifest) != (f.format, f.game, f.seat, &f.manifest) {
        return Rejoined::Other;
    }
    let end = |r: &Record| u64::from(r.header.first) + r.messages.len() as u64;
    let skip = f.first.checked_sub(t.first).map(usize::try_from);
    if let Some(Ok(skip)) = skip
        && u64::from(f.first) <= end(taken)
        && end(fetched) >= end(taken)
    {
        let both = taken.messages.len() - skip;
        if fetched.messages[..both] == taken.messages[skip..] {
            let mut merged = taken.clone();
            merged.messages.extend_from_slice(&fetched.messages[both..]);
            return Rejoined::Extends(Box::new(merged));
        }
    }
    Rejoined::Afresh
}

/// The servers' keys pinned by address (ADR-0021 point 1: a bookmark is an address and a key), as
/// the adapter keeps them in a file: a line per address, the address and the key as lowercase hex
/// separated by a space, in the addresses' order; blank lines and lines starting with `#` are
/// skipped.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Pins(Vec<(String, [u8; KEY_BYTES])>);

impl Pins {
    /// Reads a pins file's `text`.
    ///
    /// # Errors
    /// The number (from 1) of the first line that is not an address and a key, or that names an
    /// address a line before named.
    pub fn parse(text: &str) -> Result<Self, usize> {
        let mut pins = Self::default();
        for (i, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut words = line.split_whitespace();
            let (Some(address), Some(key), None) = (words.next(), words.next(), words.next())
            else {
                return Err(i + 1);
            };
            match (account::from_hex(key), pins.key(address)) {
                (Some(key), None) => pins.pin(address, key),
                _ => return Err(i + 1),
            }
        }
        Ok(pins)
    }

    /// The pins file's text.
    pub fn text(&self) -> String {
        let mut text = String::new();
        for (address, key) in &self.0 {
            text.push_str(address);
            text.push(' ');
            text.push_str(&account::to_hex(key));
            text.push('\n');
        }
        text
    }

    /// The addresses pinned, in order: the servers this client knows (E17 M5 T7: a password
    /// change re-keys the account at each).
    pub fn addresses(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|(address, _)| address.as_str())
    }

    /// The key pinned for `address`.
    pub fn key(&self, address: &str) -> Option<[u8; KEY_BYTES]> {
        let at = self.0.binary_search_by(|(a, _)| a.as_str().cmp(address));
        at.ok().and_then(|i| self.0.get(i)).map(|(_, key)| *key)
    }

    /// Pins `key` for `address` (an address has one key: a pin before is replaced).
    pub fn pin(&mut self, address: &str, key: [u8; KEY_BYTES]) {
        let at = match self.0.binary_search_by(|(a, _)| a.as_str().cmp(address)) {
            Ok(i) => {
                self.0.remove(i);
                i
            }
            Err(i) => i,
        };
        self.0.insert(at, (String::from(address), key));
    }
}
