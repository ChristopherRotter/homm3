//! A connection's own messages (ADR-0018 points 4 and 9): the hello, the ping, the frames that
//! do not decode, and the requests about an account (E16 M3 T1, ADR-0019 point 1). Everything
//! else goes to the host: the lobby's messages and the game's ([`crate::session`]). One per
//! connection; the host feeds it every frame the client sends.
//!
//! - **The welcome** (E16 M6 T1, ADR-0021 point 1). The hello names a nonce and the address the
//!   client dialed; the welcome names the server's id on this connection (that address when it is
//!   one of the server's other addresses, otherwise [`Host::id`]), and carries the server's key and
//!   the key's signature over the id, the nonce and the challenge ([`account::welcome_message`]),
//!   made by the host's [`Signer`]. A hello whose nonce is not 32 bytes of hex closes the
//!   connection.
//! - **The login** (E16 M3 T1). The welcome names the server's id and the connection's challenge,
//!   which the host drew from the OS; a login signs both. The gate checks the form of an account's
//!   request (the names, the hex) and the connection's state (logged in or not), and hands the host
//!   what it is to check ([`AccountRequest`]): the signatures over the exact bytes, the account
//!   against the server's players. Only the host's yes ([`Connection::logged_in`]) logs the
//!   connection in. On a server that takes no anonymous players, joining or creating a game waits
//!   for the login; on any server, so does asking for one's own row on a leaderboard (E16 M4 T4).
//!
//! Provisional (E16 M1 T2, M3 T1, M6 T1): a hello without a nonce and a message out of place
//! (anything before the hello, a second hello) close the connection with `Protocol`, as a client
//! that does not follow this protocol; an account's request out of place is refused, the
//! connection stays open.

use alloc::string::String;
use alloc::vec::Vec;

use crate::PROTOCOL_VERSION;
use crate::account::{self, CHALLENGE_BYTES, KEY_BYTES, NONCE_BYTES, SIGNATURE_BYTES};
use crate::codec::{self, DecodeError};
use crate::wire::{ClientMessage, GoodbyeReason, Refusal, ServerMessage};

/// What a frame from the client asks of the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Received {
    /// A message for the lobby or the game: the host answers it or passes it to its
    /// [`crate::session::Session`].
    Game(ClientMessage),
    /// A frame to send back on this connection, after which the host closes it if `close` (a
    /// `Goodbye`).
    Reply {
        /// The frame's text.
        frame: String,
        /// Whether the connection closes after it.
        close: bool,
    },
    /// A request about an account, for the host to check and answer (E16 M3 T1).
    Account(AccountRequest),
}

/// A request about an account whose form the gate checked (E16 M3 T1). The host answers
/// `Account` (the account as it stands) or `Refused`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountRequest {
    /// A login, or a registration with a display name: the host checks `signature` by `key` over
    /// `signed` ([`account::login_message`]) and the account (a known login name with this key,
    /// or a free login name with a free display name), then answers and calls
    /// [`Connection::logged_in`], or refuses (`BadLogin`, `NameTaken`).
    Login {
        /// The client's sequence number.
        seq: u32,
        /// The login name.
        login: String,
        /// The public key.
        key: [u8; KEY_BYTES],
        /// The signature.
        signature: [u8; SIGNATURE_BYTES],
        /// The display name a registration asks for.
        display: Option<String>,
        /// The bytes the signature is over.
        signed: Vec<u8>,
    },
    /// Re-key the logged-in account `login` to `key`: the host checks `by_old` by the account's
    /// key and `by_new` by `key`, both over `signed` ([`account::rekey_message`]).
    Rekey {
        /// The client's sequence number.
        seq: u32,
        /// The account's login name.
        login: String,
        /// The new public key.
        key: [u8; KEY_BYTES],
        /// The signature by the account's key so far.
        by_old: [u8; SIGNATURE_BYTES],
        /// The signature by the new key.
        by_new: [u8; SIGNATURE_BYTES],
        /// The bytes both signatures are over.
        signed: Vec<u8>,
    },
    /// Rename the logged-in account `login`; the display name keeps its rules.
    Rename {
        /// The client's sequence number.
        seq: u32,
        /// The account's login name.
        login: String,
        /// The new display name.
        display: String,
    },
    /// Whether the replays of the logged-in account `login`'s finished games may be listed.
    SetListed {
        /// The client's sequence number.
        seq: u32,
        /// The account's login name.
        login: String,
        /// Whether they may.
        listed: bool,
    },
}

/// The server's key, which signs its welcomes (E16 M6 T1, ADR-0021 point 1): the host holds the
/// key and the cryptography, this crate says what is signed. `Sync`: a host's connections share it.
pub trait Signer: core::fmt::Debug + Sync {
    /// The public key.
    fn key(&self) -> [u8; KEY_BYTES];

    /// The key's signature over `message`.
    fn sign(&self, message: &[u8]) -> [u8; SIGNATURE_BYTES];
}

/// The server as the welcome names it.
#[derive(Debug, Clone, Copy)]
pub struct Host<'a> {
    /// The server program's name and version (informational).
    pub program: &'a str,
    /// The server's id: the address its clients dial (E16 M3 T1), its first.
    pub id: &'a str,
    /// The server's other addresses (E16 M6 T1): a client that dialed one of them gets it as the
    /// id, so a login works at every address of the server.
    pub addresses: &'a [String],
    /// The server's key (E16 M6 T1).
    pub signer: &'a dyn Signer,
}

/// One client connection before and after its hello, and its login.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connection {
    /// The largest frame accepted, in bytes.
    max_frame: usize,
    /// Whether the hello was answered.
    greeted: bool,
    /// The connection's challenge, which a login signs (E16 M3 T1).
    challenge: [u8; CHALLENGE_BYTES],
    /// The server's id on this connection, which its logins sign (E16 M6 T1): set by the hello.
    id: String,
    /// Whether the connection may join or create a game without logging in.
    anonymous: bool,
    /// The login name of the account the connection is logged in as.
    account: Option<String>,
}

impl Connection {
    /// A new connection that accepts frames of at most `max_frame` bytes
    /// ([`codec::DEFAULT_MAX_CLIENT_FRAME`] by default), whose login signs `challenge` (drawn by
    /// the host from the OS), and that may play without logging in if `anonymous`.
    pub const fn new(max_frame: usize, challenge: [u8; CHALLENGE_BYTES], anonymous: bool) -> Self {
        Self {
            max_frame,
            greeted: false,
            challenge,
            id: String::new(),
            anonymous,
            account: None,
        }
    }

    /// Whether the hello was answered.
    pub const fn greeted(&self) -> bool {
        self.greeted
    }

    /// The login name of the account the connection is logged in as.
    pub fn account(&self) -> Option<&str> {
        self.account.as_deref()
    }

    /// The host's yes to a login (E16 M3 T1): the connection is logged in as `login`.
    pub fn logged_in(&mut self, login: String) {
        self.account = Some(login);
    }

    /// What the client's `frame` asks for; `host` is named in the welcome.
    pub fn receive(&mut self, frame: &[u8], host: &Host<'_>) -> Received {
        let message = match codec::decode_client(frame, self.max_frame) {
            Ok(message) => message,
            Err(DecodeError::TooLarge { .. }) => return goodbye(GoodbyeReason::TooLarge),
            Err(DecodeError::Malformed) => return goodbye(GoodbyeReason::Malformed),
        };
        match (self.greeted, message) {
            (
                false,
                ClientMessage::Hello {
                    protocol,
                    nonce,
                    address,
                    ..
                },
            ) if protocol == PROTOCOL_VERSION => self.welcome(&nonce, address, host),
            (false, _) | (true, ClientMessage::Hello { .. }) => goodbye(GoodbyeReason::Protocol),
            (true, ClientMessage::Ping { nonce }) => reply(&ServerMessage::Pong { nonce }),
            (
                true,
                ClientMessage::Login {
                    seq,
                    login,
                    key,
                    signature,
                    display,
                },
            ) => self.login(seq, (login, display), (&key, &signature)),
            (
                true,
                ClientMessage::Rekey {
                    seq,
                    key,
                    by_old,
                    by_new,
                },
            ) => self.rekey(seq, &key, (&by_old, &by_new)),
            (true, ClientMessage::Rename { seq, display }) => self.rename(seq, display),
            (true, ClientMessage::SetListed { seq, listed }) => match self.account.clone() {
                Some(login) => Received::Account(AccountRequest::SetListed { seq, login, listed }),
                None => refused(Refusal::NotLoggedIn),
            },
            (true, ClientMessage::Join { .. } | ClientMessage::CreateGame { .. })
                if !self.anonymous && self.account.is_none() =>
            {
                refused(Refusal::NotLoggedIn)
            }
            // One's own row on a leaderboard needs one's account (E16 M4 T4).
            (true, ClientMessage::Standing { .. }) if self.account.is_none() => {
                refused(Refusal::NotLoggedIn)
            }
            (true, message) => Received::Game(message),
        }
    }

    /// The welcome to a hello that named `nonce` and dialed `address`, signed by the host's key.
    fn welcome(&mut self, nonce: &str, address: String, host: &Host<'_>) -> Received {
        let Some(nonce) = account::from_hex::<NONCE_BYTES>(nonce) else {
            return goodbye(GoodbyeReason::Protocol);
        };
        self.greeted = true;
        self.id = if host.addresses.contains(&address) {
            address
        } else {
            String::from(host.id)
        };
        let signed = account::welcome_message(&self.id, &nonce, &self.challenge);
        reply(&ServerMessage::Welcome {
            protocol: PROTOCOL_VERSION,
            server: String::from(host.program),
            id: self.id.clone(),
            challenge: account::to_hex(&self.challenge),
            key: account::to_hex(&host.signer.key()),
            signature: account::to_hex(&host.signer.sign(&signed)),
        })
    }

    /// A login, or a registration with `display`, for the host to check.
    fn login(
        &self,
        seq: u32,
        (login, display): (String, Option<String>),
        (key, signature): (&str, &str),
    ) -> Received {
        if self.account.is_some() {
            return refused(Refusal::AlreadyLoggedIn);
        }
        let display_ok = display.as_deref().is_none_or(account::is_display_name);
        if !account::is_login_name(&login) || !display_ok {
            return refused(Refusal::BadName);
        }
        let (Some(key), Some(signature)) = (account::from_hex(key), account::from_hex(signature))
        else {
            return refused(Refusal::BadLogin);
        };
        let signed = account::login_message(&self.id, &self.challenge, &login, display.as_deref());
        Received::Account(AccountRequest::Login {
            seq,
            login,
            key,
            signature,
            display,
            signed,
        })
    }

    /// A re-key of the logged-in account to `key`, for the host to check.
    fn rekey(&self, seq: u32, key: &str, (by_old, by_new): (&str, &str)) -> Received {
        let Some(login) = self.account.clone() else {
            return refused(Refusal::NotLoggedIn);
        };
        let hex = (
            account::from_hex(key),
            account::from_hex(by_old),
            account::from_hex(by_new),
        );
        let (Some(key), Some(by_old), Some(by_new)) = hex else {
            return refused(Refusal::BadLogin);
        };
        let signed = account::rekey_message(&self.id, &self.challenge, &login, &key);
        Received::Account(AccountRequest::Rekey {
            seq,
            login,
            key,
            by_old,
            by_new,
            signed,
        })
    }

    /// A new display name for the logged-in account, for the host to check.
    fn rename(&self, seq: u32, display: String) -> Received {
        let Some(login) = self.account.clone() else {
            return refused(Refusal::NotLoggedIn);
        };
        if !account::is_display_name(&display) {
            return refused(Refusal::BadName);
        }
        Received::Account(AccountRequest::Rename {
            seq,
            login,
            display,
        })
    }
}

/// An answer that keeps the connection open.
fn reply(message: &ServerMessage) -> Received {
    Received::Reply {
        frame: codec::encode_text(message),
        close: false,
    }
}

/// A refusal; the connection stays open.
fn refused(reason: Refusal) -> Received {
    reply(&ServerMessage::Refused { reason })
}

/// The last frame on a connection.
fn goodbye(reason: GoodbyeReason) -> Received {
    Received::Reply {
        frame: codec::encode_text(&ServerMessage::Goodbye { reason }),
        close: true,
    }
}
