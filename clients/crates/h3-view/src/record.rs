//! A record (ADR-0020 point 8; `docs/epics/E16-design.md` §6): one viewer's message log and the
//! viewer's file format. Its first line is a [`RecordHeader`], every further line one numbered
//! server message, each a JSON document as the wire carries it. A live game is the same messages
//! arriving over a socket.

use alloc::vec::Vec;
use core::fmt;

use h3_protocol::codec::{self, DecodeError};
use h3_protocol::wire::{Event, Observation, RecordHeader, ServerMessage};
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

    /// The battle's start's events, if the record holds them ([`opening`]).
    #[must_use]
    pub fn opening(&self) -> Option<&[Event]> {
        opening(&self.messages)
    }

    /// Each command's events, in order ([`commands`]).
    pub fn commands(&self) -> impl Iterator<Item = &[Event]> {
        commands(&self.messages)
    }

    /// Every command's event, in order: the battle's start's left out ([`commands`]).
    pub fn events(&self) -> impl Iterator<Item = &Event> {
        self.commands().flatten()
    }

    /// The battle as it ended, as the viewer sees it ([`ended`]).
    #[must_use]
    pub fn ended(&self) -> Option<&Observation> {
        ended(&self.messages)
    }
}

/// The battle as it ended in `messages`, a record's, as its viewer sees it (protocol v38,
/// CLT-STUB-3): the `Snapshot` right before `GameEnded`; none before the end, nor in a record
/// whose `GameEnded` follows something else (an older server's).
#[must_use]
pub fn ended(messages: &[ServerMessage]) -> Option<&Observation> {
    messages.windows(2).find_map(|pair| match pair {
        [
            ServerMessage::Snapshot { observation, .. },
            ServerMessage::GameEnded { .. },
        ] => Some(observation),
        _ => None,
    })
}

/// Whether `events`, a record's first `Events` message's, are the battle's start's (protocol v38,
/// record format 3; CLT-Q20, E17 M6, *provisional*): the start opens with the defender's unseen
/// mines (`ObstaclePlaced`), a tactics phase (`TacticsStarted`) or the first activation
/// (`TurnStarted`, a war machine's or a turret's acting on its own too), each before anything
/// else; a command's open with its act (a move, a strike, a wait, a cast…), which none of these
/// is. The record does not say otherwise: a seat's record holds the start from its first
/// message, a spectator's only when it watched the start (the session's protocol), and both
/// open with the same snapshot.
#[must_use]
pub fn opens(events: &[Event]) -> bool {
    matches!(
        events.first(),
        Some(
            Event::ObstaclePlaced { .. } | Event::TacticsStarted { .. } | Event::TurnStarted { .. }
        )
    )
}

/// The index in `messages`, a record's (or the part of one a viewer holds so far), of the
/// `Events` message that tells the battle's start: its first `Events` message, if it [`opens`].
#[must_use]
pub fn opening_at(messages: &[ServerMessage]) -> Option<usize> {
    start(messages).map(|(i, _)| i)
}

/// The battle's start's events in `messages` ([`opening_at`]): what the battle's start did, which
/// the record's first snapshot already shows (the session's engine has no state before its
/// start), so no reader applies them to it again.
#[must_use]
pub fn opening(messages: &[ServerMessage]) -> Option<&[Event]> {
    start(messages).map(|(_, events)| events)
}

/// The first `Events` message of `messages` with its index, if it [`opens`].
fn start(messages: &[ServerMessage]) -> Option<(usize, &[Event])> {
    let (i, events) = messages.iter().enumerate().find_map(|(i, m)| match m {
        ServerMessage::Events { events, .. } => Some((i, events.as_slice())),
        _ => None,
    })?;
    opens(events).then_some((i, events))
}

/// Each command's events in `messages`, in order: their `Events` messages but the battle's
/// start's ([`opening_at`]; protocol v38: before it every `Events` message was a command's).
pub fn commands(messages: &[ServerMessage]) -> impl Iterator<Item = &[Event]> {
    let opening = opening_at(messages);
    (messages.iter().enumerate()).filter_map(move |(i, m)| match m {
        ServerMessage::Events { events, .. } if Some(i) != opening => Some(events.as_slice()),
        _ => None,
    })
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
    use alloc::format;
    use alloc::string::{String, ToString};
    use alloc::vec;
    use h3_protocol::wire::{
        Algorithm, Field, KeyTables, Manifest, Observation, Outcome, Seat, Side, SideView, WireU64,
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
            morale_and_luck: Default::default(),
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

    /// Protocol v38: the battle's start is the first `Events` message's when it opens with the
    /// mines, the tactics phase or the first activation; every other `Events` message is a
    /// command's.
    #[test]
    fn the_battle_s_start_is_no_command() {
        let events = |n, events| ServerMessage::Events { n, events };
        let snapshot = ServerMessage::Snapshot {
            n: 1,
            observation: observation(),
        };
        let mines = Event::ObstaclePlaced {
            side: Side::Defender,
            spell: 11,
            hexes: vec![40],
        };
        let tactics = Event::TacticsStarted {
            side: Side::Attacker,
            columns: 3,
        };
        let turn = Event::TurnStarted { stack: 2 };
        let wait = Event::Waited { stack: 2 };
        for start in [vec![mines, turn.clone()], vec![tactics], vec![turn.clone()]] {
            let messages = [
                snapshot.clone(),
                events(2, start.clone()),
                events(3, vec![wait.clone(), turn.clone()]),
                events(4, vec![turn.clone()]),
            ];
            assert_eq!(opening_at(&messages), Some(1));
            assert_eq!(opening(&messages), Some(&start[..]));
            let commands: Vec<_> = commands(&messages).collect();
            assert_eq!(
                commands,
                [
                    &[wait.clone(), turn.clone()][..],
                    core::slice::from_ref(&turn)
                ]
            );
            let record = Record {
                header: header(),
                messages: messages.to_vec(),
            };
            assert_eq!(record.opening(), Some(&start[..]));
            assert_eq!(record.commands().count(), 2);
            assert_eq!(record.events().collect::<Vec<_>>(), [&wait, &turn, &turn]);
        }
        // A record whose first `Events` is a command's (a spectator watching from a later turn),
        // or an empty one's, holds no start; nor does one without any.
        for first in [vec![wait.clone()], Vec::new()] {
            let messages = [
                snapshot.clone(),
                events(2, first),
                events(3, vec![turn.clone()]),
            ];
            assert_eq!((opening_at(&messages), opening(&messages)), (None, None));
            assert_eq!(commands(&messages).count(), 2);
        }
        assert_eq!(opening(&[snapshot]), None);
        assert!(!opens(&[]));
    }

    /// Protocol v38: the battle as it ended is the snapshot right before `GameEnded`.
    #[test]
    fn the_battle_as_it_ended_is_the_last_snapshot() {
        let mut end = observation();
        end.round = 9;
        let snapshot = |n, observation| ServerMessage::Snapshot { n, observation };
        let over = |n| ServerMessage::GameEnded {
            n,
            result: Outcome::Draw {},
        };
        let wait = ServerMessage::Events {
            n: 2,
            events: vec![Event::Waited { stack: 0 }],
        };
        let messages = [
            snapshot(1, observation()),
            wait.clone(),
            snapshot(3, end.clone()),
            over(4),
        ];
        let record = Record {
            header: header(),
            messages: messages.to_vec(),
        };
        assert_eq!(record.ended(), Some(&end));
        // Before the end, or a `GameEnded` after a command's events or first: none.
        assert_eq!(ended(&messages[..3]), None);
        assert_eq!(ended(&[snapshot(1, observation()), wait, over(3)]), None);
        assert_eq!(ended(&[over(1)]), None);
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
        let reads = format!("reads {RECORD_FORMAT_VERSION}");
        assert!(RecordError::Format(1).to_string().contains(&reads));
        assert!(
            RecordError::Protocol(1)
                .to_string()
                .starts_with("protocol 1")
        );
    }
}
