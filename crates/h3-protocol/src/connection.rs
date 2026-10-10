//! A connection's own messages (ADR-0018 points 4 and 9): the hello, the ping, and the frames
//! that do not decode. Everything else goes to the game ([`crate::session`]). One per connection;
//! the host feeds it every frame the client sends.
//!
//! Provisional (E16 M1 T2): a message out of place (anything before the hello, a second hello)
//! closes the connection with `Protocol`, as a client that does not follow this protocol.

use alloc::string::String;

use crate::PROTOCOL_VERSION;
use crate::codec::{self, DecodeError};
use crate::wire::{ClientMessage, GoodbyeReason, ServerMessage};

/// What a frame from the client asks of the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Received {
    /// A message for the game: the host passes it to its [`crate::session::Session`].
    Game(ClientMessage),
    /// A frame to send back on this connection, after which the host closes it if `close` (a
    /// `Goodbye`).
    Reply {
        /// The frame's text.
        frame: String,
        /// Whether the connection closes after it.
        close: bool,
    },
}

/// One client connection before and after its hello.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connection {
    /// The largest frame accepted, in bytes.
    max_frame: usize,
    /// Whether the hello was answered.
    greeted: bool,
}

impl Connection {
    /// A new connection that accepts frames of at most `max_frame` bytes
    /// ([`codec::DEFAULT_MAX_CLIENT_FRAME`] by default).
    pub const fn new(max_frame: usize) -> Self {
        Self {
            max_frame,
            greeted: false,
        }
    }

    /// Whether the hello was answered.
    pub const fn greeted(&self) -> bool {
        self.greeted
    }

    /// What the client's `frame` asks for; `server` names the server program in the welcome.
    pub fn receive(&mut self, frame: &[u8], server: &str) -> Received {
        let message = match codec::decode_client(frame, self.max_frame) {
            Ok(message) => message,
            Err(DecodeError::TooLarge { .. }) => return goodbye(GoodbyeReason::TooLarge),
            Err(DecodeError::Malformed) => return goodbye(GoodbyeReason::Malformed),
        };
        match (self.greeted, message) {
            (false, ClientMessage::Hello { protocol, .. }) if protocol == PROTOCOL_VERSION => {
                self.greeted = true;
                reply(&ServerMessage::Welcome {
                    protocol,
                    server: String::from(server),
                })
            }
            (true, ClientMessage::Ping { nonce }) => reply(&ServerMessage::Pong { nonce }),
            (false, _) | (true, ClientMessage::Hello { .. }) => goodbye(GoodbyeReason::Protocol),
            (true, message) => Received::Game(message),
        }
    }
}

/// An answer that keeps the connection open.
fn reply(message: &ServerMessage) -> Received {
    Received::Reply {
        frame: codec::encode_text(message),
        close: false,
    }
}

/// The last frame on a connection.
fn goodbye(reason: GoodbyeReason) -> Received {
    Received::Reply {
        frame: codec::encode_text(&ServerMessage::Goodbye { reason }),
        close: true,
    }
}
