//! The bounded codec (ADR-0018 points 4 and 9): one JSON document per frame. A frame's size is
//! checked before it is parsed; decoding never panics (the crate's lint header and a property
//! test on arbitrary bytes); `serde_json` bounds the nesting depth.

use alloc::string::String;
use alloc::vec::Vec;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::wire::{ClientMessage, ServerMessage};

/// The default largest frame a server accepts from a client, in bytes. Client messages are
/// small; a hello with long names stays far below it. Provisional (E16 M1 review).
pub const DEFAULT_MAX_CLIENT_FRAME: usize = 4096;

/// Why a frame did not decode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    /// The frame is larger than the limit; it was not parsed.
    TooLarge {
        /// The frame's size.
        len: usize,
        /// The limit.
        max: usize,
    },
    /// The frame is not one message of this protocol version.
    Malformed,
}

/// Decodes one message from a frame of at most `max` bytes.
pub fn decode<T: DeserializeOwned>(frame: &[u8], max: usize) -> Result<T, DecodeError> {
    if frame.len() > max {
        return Err(DecodeError::TooLarge {
            len: frame.len(),
            max,
        });
    }
    serde_json::from_slice(frame).map_err(|_| DecodeError::Malformed)
}

/// Decodes a client's message from a frame of at most `max` bytes.
pub fn decode_client(frame: &[u8], max: usize) -> Result<ClientMessage, DecodeError> {
    decode(frame, max)
}

/// Decodes a server's message from a frame of at most `max` bytes (for clients and tests).
pub fn decode_server(frame: &[u8], max: usize) -> Result<ServerMessage, DecodeError> {
    decode(frame, max)
}

/// Encodes one message as a frame. The wire types have string keys only and no fallible
/// serializer, so encoding cannot fail.
pub fn encode<T: Serialize>(message: &T) -> Vec<u8> {
    serde_json::to_vec(message).unwrap_or_default()
}

/// Encodes one message as the text of a frame (a WebSocket text frame, a record's line).
pub fn encode_text<T: Serialize>(message: &T) -> String {
    serde_json::to_string(message).unwrap_or_default()
}
