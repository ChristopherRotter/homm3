//! The record's reader (E24 M5 T2c, MCL-Q57): a whole game's record read back as its session
//! writes it, and every refusal (a battle's read back: `session`'s record test).

use std::format;
use std::string::{String, ToString};
use std::vec::Vec;

use h3_contracts::PlayerId;

use super::game_session::{MAP, duel, options};
use crate::codec::{self, DecodeError};
use crate::map::game::manifest;
use crate::record::{self, RecordError};
use crate::session::game::GameSession;
use crate::wire::ServerMessage;
use crate::wire::game::GameRecordHeader;
use crate::{PROTOCOL_VERSION, RECORD_FORMAT_VERSION};

/// A duel's first player's record as its session writes it at the start, and the messages the
/// player was sent.
fn written() -> (String, Vec<ServerMessage>) {
    let (rules, setup) = duel();
    let mut out = Vec::new();
    let session =
        GameSession::start(rules, &setup, 3, manifest(rules, MAP), options(0), &mut out).unwrap();
    let player = PlayerId::new(0);
    let sent = out
        .iter()
        .filter(|o| o.to == player)
        .map(|o| codec::decode_server(o.frame.as_bytes(), usize::MAX).unwrap())
        .collect();
    (session.record(player).unwrap(), sent)
}

/// A whole game's header line's versions.
fn versions(header: &GameRecordHeader) -> (u32, u32) {
    (header.format, header.manifest.protocol)
}

#[test]
fn a_whole_games_record_reads_back_as_written() {
    let (text, sent) = written();
    let (header, messages) = record::read_game(&text).unwrap();
    assert_eq!(
        (header.format, header.game, header.player, header.first),
        (RECORD_FORMAT_VERSION, 9, 0, 1)
    );
    assert_eq!(header.manifest.protocol, PROTOCOL_VERSION);
    assert!(!messages.is_empty());
    assert_eq!(messages, sent);
    assert!(record::is_game(&text));
    // Blank lines are skipped, before the header too.
    let spaced = format!("\n \n{}\n\n", text.replacen('\n', "\n\t\n", 1));
    assert!(record::is_game(&spaced));
    assert_eq!(record::read_game(&spaced), Ok((header, messages)));
}

#[test]
fn a_record_not_of_this_build_is_refused_by_its_line() {
    let (text, _) = written();
    assert_eq!(record::read_game(""), Err(RecordError::Empty));
    assert_eq!(record::read_game(" \n\t\n"), Err(RecordError::Empty));
    assert!(!record::is_game(" \n"));
    assert_eq!(
        record::read_game("nonsense"),
        Err(RecordError::Line(1, DecodeError::Malformed))
    );
    assert!(!record::is_game("nonsense"));
    // A message that does not read: its line's number in the text, the blank lines counted.
    let lines = text.lines().count();
    let bad = format!("{text}\n\n{{\"NoSuchMessage\":{{}}}}");
    assert_eq!(
        record::read_game(&bad),
        Err(RecordError::Line(lines + 2, DecodeError::Malformed))
    );
    // Another format, another protocol.
    let first = text.lines().next().unwrap();
    let mut header: GameRecordHeader = serde_json::from_str(first).unwrap();
    header.format = RECORD_FORMAT_VERSION + 1;
    assert_eq!(
        record::read_game(&codec::encode_text(&header)),
        Err(RecordError::Format(RECORD_FORMAT_VERSION + 1))
    );
    header.format = RECORD_FORMAT_VERSION;
    header.manifest.protocol = PROTOCOL_VERSION - 1;
    assert_eq!(
        record::read_game(&codec::encode_text(&header)),
        Err(RecordError::Protocol(PROTOCOL_VERSION - 1))
    );
    // A line past the bound, the header's or a message's.
    assert_eq!(
        record::read(&text, 8, versions),
        Err(RecordError::Line(
            1,
            DecodeError::TooLarge {
                len: first.len(),
                max: 8
            }
        ))
    );
    let long = format!("{first}\n{}", "x".repeat(first.len() + 1));
    assert_eq!(
        record::read(&long, first.len(), versions),
        Err(RecordError::Line(
            2,
            DecodeError::TooLarge {
                len: first.len() + 1,
                max: first.len()
            }
        ))
    );
}

#[test]
fn a_refusal_names_its_line_by_number_alone() {
    let told = [
        (RecordError::Empty, "the record is empty".to_string()),
        (
            RecordError::Line(3, DecodeError::Malformed),
            "line 3: not a message of this protocol version".to_string(),
        ),
        (
            RecordError::Line(3, DecodeError::TooLarge { len: 9, max: 8 }),
            "line 3: 9 bytes, at most 8".to_string(),
        ),
        (
            RecordError::Format(4),
            format!("record format 4, this client reads {RECORD_FORMAT_VERSION}"),
        ),
        (
            RecordError::Protocol(1),
            format!("protocol 1, this client reads {PROTOCOL_VERSION}"),
        ),
    ];
    for (error, text) in told {
        assert_eq!(error.to_string(), text);
    }
}
