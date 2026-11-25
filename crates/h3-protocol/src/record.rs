//! A record's reader (E24 M5 T2c, MCL-Q57: the reader beside its writer, shared by every client):
//! the record format ([`RECORD_FORMAT_VERSION`]) read back. A record is one viewer's file, as
//! `session` (a battle's, [`RecordHeader`]) and `session::game` (a whole game's,
//! [`GameRecordHeader`]) write it: its first line the header, every further line one numbered
//! server message the viewer was sent, each a JSON document as the wire carries it. Blank lines
//! are skipped.
//!
//! - **Only this build's records read**: a header whose format is not [`RECORD_FORMAT_VERSION`] or
//!   whose game's protocol is not [`PROTOCOL_VERSION`] is refused (before 1.0 a client reads only
//!   its own, ADR-0018 point 4).
//! - **A record may come from a server**, so it is untrusted input: each line is bounded
//!   ([`MAX_LINE`]) and a refusal names a line by its number and the codec's reason, never by its
//!   text.
//!
//! What a record holds beyond its lines (a battle's opening, a whole game's battles) is each
//! client's own reading.
//!
//! *Provisional* (E24 M5 T2c): the names, the line bound.

use alloc::vec::Vec;
use core::fmt;

use serde::de::DeserializeOwned;

use crate::codec::{self, DecodeError};
use crate::wire::game::GameRecordHeader;
use crate::wire::{RecordHeader, ServerMessage};
use crate::{PROTOCOL_VERSION, RECORD_FORMAT_VERSION};

/// The most bytes one line may hold: a line is one message, as a live client bounds a message (a
/// server sends a whole record in one).
pub const MAX_LINE: usize = 64 << 20;

/// Why a record does not read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordError {
    /// The text has no header line.
    Empty,
    /// A line (counting from 1; the header's is 1) is not a header or a message of this protocol
    /// version: why, in numbers only, never the line's own text.
    Line(usize, DecodeError),
    /// The record's format is not [`RECORD_FORMAT_VERSION`].
    Format(u32),
    /// The game's protocol version is not [`PROTOCOL_VERSION`].
    Protocol(u32),
}

impl fmt::Display for RecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "the record is empty"),
            Self::Line(n, DecodeError::Malformed) => {
                write!(f, "line {n}: not a message of this protocol version")
            }
            Self::Line(n, DecodeError::TooLarge { len, max }) => {
                write!(f, "line {n}: {len} bytes, at most {max}")
            }
            Self::Format(v) => write!(
                f,
                "record format {v}, this client reads {RECORD_FORMAT_VERSION}"
            ),
            Self::Protocol(v) => write!(f, "protocol {v}, this client reads {PROTOCOL_VERSION}"),
        }
    }
}

/// Reads a battle's record: its header and the viewer's messages, in order.
///
/// # Errors
/// The [`RecordError`] of the first line that does not read; a format or protocol not this
/// build's.
pub fn read_battle(text: &str) -> Result<(RecordHeader, Vec<ServerMessage>), RecordError> {
    read(text, MAX_LINE, |h: &RecordHeader| {
        (h.format, h.manifest.protocol)
    })
}

/// Reads a whole game's record: its header and the player's messages, in order.
///
/// # Errors
/// The [`RecordError`] of the first line that does not read; a format or protocol not this
/// build's.
pub fn read_game(text: &str) -> Result<(GameRecordHeader, Vec<ServerMessage>), RecordError> {
    read(text, MAX_LINE, |h: &GameRecordHeader| {
        (h.format, h.manifest.protocol)
    })
}

/// Whether `text` is a whole game's record: its first line not blank reads as a
/// [`GameRecordHeader`] (a battle's record's first line, a [`RecordHeader`], does not: both deny
/// unknown fields).
#[must_use]
pub fn is_game(text: &str) -> bool {
    let first = text.lines().find(|line| !line.trim().is_empty());
    first.is_some_and(|line| codec::decode::<GameRecordHeader>(line.as_bytes(), MAX_LINE).is_ok())
}

/// Reads a record whose header is an `H`, its lines bounded at `max` bytes; `versions` gives a
/// header's format and protocol.
pub(crate) fn read<H: DeserializeOwned>(
    text: &str,
    max: usize,
    versions: impl Fn(&H) -> (u32, u32),
) -> Result<(H, Vec<ServerMessage>), RecordError> {
    let mut lines = text
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty());
    let (_, first) = lines.next().ok_or(RecordError::Empty)?;
    let header: H = decode(1, first, max)?;
    let (format, protocol) = versions(&header);
    if format != RECORD_FORMAT_VERSION {
        return Err(RecordError::Format(format));
    }
    if protocol != PROTOCOL_VERSION {
        return Err(RecordError::Protocol(protocol));
    }
    let messages = lines
        .map(|(i, line)| decode(i + 1, line, max))
        .collect::<Result<_, _>>()?;
    Ok((header, messages))
}

/// Decodes line `n` (from 1) of a record.
fn decode<T: DeserializeOwned>(n: usize, line: &str, max: usize) -> Result<T, RecordError> {
    codec::decode(line.as_bytes(), max).map_err(|e| RecordError::Line(n, e))
}
