//! A record (ADR-0020 point 8; `docs/epics/E16-design.md` §6): one viewer's message log and the
//! viewer's file format. Its first line is a [`RecordHeader`], every further line one numbered
//! server message, each a JSON document as the wire carries it. A live game is the same messages
//! arriving over a socket.

use alloc::vec::Vec;
use core::fmt;

use h3_protocol::codec::{self, DecodeError};
use h3_protocol::wire::{Observation, RecordHeader, ServerMessage};
use h3_protocol::{PROTOCOL_VERSION, RECORD_FORMAT_VERSION};

/// The most bytes one line may hold (*provisional*): a server bounds a whole record (E16 M2
/// T5c), the client a line, as its codec bounds a frame.
pub const MAX_LINE: usize = 1 << 24;

/// A record, read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// What the game is made of and whose record it is.
    pub header: RecordHeader,
    /// The viewer's messages, in order.
    pub messages: Vec<ServerMessage>,
}

impl Record {
    /// The battle as the record's first snapshot shows it: the view at the record's start.
    pub fn first_snapshot(&self) -> Option<&Observation> {
        self.messages.iter().find_map(|m| match m {
            ServerMessage::Snapshot { observation, .. } => Some(observation),
            _ => None,
        })
    }
}

/// Why a record does not read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordError {
    /// The text has no header line.
    Empty,
    /// A line (counting from 1) is not a header or a message of this protocol version: why, in
    /// numbers only, never the line's own text (E17 M5 T8b2, review R1's rest: a record may come
    /// from a server).
    Line(usize, DecodeError),
    /// The record's format is not [`RECORD_FORMAT_VERSION`].
    Format(u32),
    /// The game's protocol version is not [`PROTOCOL_VERSION`]: before 1.0 a client reads only its
    /// own (ADR-0018 point 4).
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

/// Reads a record's text; blank lines are skipped.
///
/// # Errors
/// The [`RecordError`] of the first line that does not read.
pub fn parse(text: &str) -> Result<Record, RecordError> {
    let mut lines = text
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty());
    let (_, first) = lines.next().ok_or(RecordError::Empty)?;
    let header: RecordHeader = decode(1, first)?;
    if header.format != RECORD_FORMAT_VERSION {
        return Err(RecordError::Format(header.format));
    }
    if header.manifest.protocol != PROTOCOL_VERSION {
        return Err(RecordError::Protocol(header.manifest.protocol));
    }
    let messages = lines
        .map(|(i, line)| decode(i + 1, line))
        .collect::<Result<_, _>>()?;
    Ok(Record { header, messages })
}

fn decode<T: serde::de::DeserializeOwned>(n: usize, line: &str) -> Result<T, RecordError> {
    codec::decode(line.as_bytes(), MAX_LINE).map_err(|e| RecordError::Line(n, e))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use alloc::string::{String, ToString};
    use alloc::vec;
    use h3_protocol::wire::{
        Algorithm, Field, KeyTables, Manifest, Observation, Seat, Side, SideView, WireU64,
    };

    /// A manifest with the given key tables.
    pub(crate) fn manifest(keys: KeyTables) -> Manifest {
        Manifest {
            protocol: PROTOCOL_VERSION,
            engine: "test".to_string(),
            packs: Vec::new(),
            profile: "base:faithful".to_string(),
            ruleset_hash: WireU64(0),
            battle_ruleset_hash: WireU64(0),
            rng: Algorithm {
                name: "rng".to_string(),
                version: 1,
            },
            hash: Algorithm {
                name: "hash".to_string(),
                version: 1,
            },
            battle_state_hash_version: 1,
            field: Field {
                width: 17,
                height: 11,
            },
            keys,
        }
    }

    /// An empty battle as a spectator sees it.
    pub(crate) fn observation() -> Observation {
        let side = SideView {
            may_retreat: true,
            may_surrender: true,
            periodic_rounds: 0,
            summoned: None,
            cost_change: 0,
            human: false,
            colour: None,
            hero: None,
        };
        Observation {
            viewer: Side::Attacker,
            spectator: true,
            round: 0,
            tactics: None,
            wait_phase: false,
            active: None,
            last_side_acted: None,
            result: None,
            drawn: None,
            terrain: None,
            battlefield: None,
            anti_magic: false,
            difficulty: 0,
            background: None,
            siege: None,
            sides: [side, side],
            own_hero: None,
            stacks: Vec::new(),
            obstacles: Vec::new(),
            obstacle_anchors: Vec::new(),
            removable_obstacles: Vec::new(),
            absolute_obstacle: None,
            obstacles_cleared: Vec::new(),
            spell_obstacles: Vec::new(),
        }
    }

    fn text(header: &RecordHeader, messages: &[ServerMessage]) -> String {
        let mut lines = vec![codec::encode_text(header)];
        lines.extend(messages.iter().map(codec::encode_text));
        lines.join("\n")
    }

    fn header() -> RecordHeader {
        RecordHeader {
            format: RECORD_FORMAT_VERSION,
            game: 1,
            seat: Seat::Spectator,
            manifest: manifest(KeyTables::default()),
            first: 1,
        }
    }

    #[test]
    fn a_record_reads_and_names_its_first_snapshot() {
        let mut later = observation();
        later.round = 3;
        let messages = [
            ServerMessage::Pong { nonce: 7 },
            ServerMessage::Snapshot {
                n: 1,
                observation: observation(),
            },
            ServerMessage::Snapshot {
                n: 2,
                observation: later,
            },
        ];
        let record = parse(&(text(&header(), &messages) + "\n\n")).unwrap();
        assert_eq!(record.header, header());
        assert_eq!(record.messages.len(), 3);
        assert_eq!(record.first_snapshot(), Some(&observation()));
        let none = parse(&text(&header(), &[])).unwrap();
        assert_eq!(none.first_snapshot(), None);
    }

    #[test]
    fn a_bad_record_is_an_error() {
        assert_eq!(parse(" \n"), Err(RecordError::Empty));
        let line = |text: &str| parse(text).unwrap_err().to_string();
        assert!(line("{}").starts_with("line 1: "));
        assert!(line(&(text(&header(), &[]) + "\n\nnot json")).starts_with("line 3: "));
        let mut old = header();
        old.format = 1;
        assert_eq!(parse(&text(&old, &[])), Err(RecordError::Format(1)));
        let mut other = header();
        other.manifest.protocol = 1;
        assert_eq!(parse(&text(&other, &[])), Err(RecordError::Protocol(1)));
        for (e, shown) in [
            (RecordError::Empty, "the record is empty"),
            (
                RecordError::Line(2, DecodeError::Malformed),
                "line 2: not a message of this protocol version",
            ),
            (
                RecordError::Line(3, DecodeError::TooLarge { len: 9, max: 8 }),
                "line 3: 9 bytes, at most 8",
            ),
        ] {
            assert_eq!(e.to_string(), shown);
        }
        assert!(RecordError::Format(1).to_string().contains("reads 2"));
        assert!(
            RecordError::Protocol(1)
                .to_string()
                .starts_with("protocol 1")
        );
    }
}
