//! The bounded codec: the size is checked before parsing, malformed frames are refused, and no
//! input panics.

use std::vec;
use std::vec::Vec;

use proptest::prelude::*;

use crate::codec::{DEFAULT_MAX_CLIENT_FRAME, DecodeError, decode_client, decode_server, encode};
use crate::wire::{ClientMessage, ServerMessage};

#[test]
fn a_frame_decodes_to_its_message_and_back() {
    let ping = ClientMessage::Ping { nonce: 9 };
    let frame = encode(&ping);
    assert_eq!(frame, br#"{"Ping":{"nonce":9}}"#.to_vec());
    assert_eq!(decode_client(&frame, DEFAULT_MAX_CLIENT_FRAME), Ok(ping));
    let pong = ServerMessage::Pong { nonce: 9 };
    assert_eq!(decode_server(&encode(&pong), 64), Ok(pong));
}

#[test]
fn a_frame_over_the_limit_is_refused_unparsed() {
    let frame = encode(&ClientMessage::Ping { nonce: 9 });
    let max = frame.len() - 1;
    assert_eq!(
        decode_client(&frame, max),
        Err(DecodeError::TooLarge {
            len: frame.len(),
            max
        })
    );
    // Exactly at the limit is fine.
    assert!(decode_client(&frame, frame.len()).is_ok());
}

#[test]
fn malformed_frames_are_refused() {
    let bad: [&[u8]; 6] = [
        b"",
        b"{",
        br#"{"Ping":{"nonce":9}} trailing"#,
        br#"{"Ping":{"nonce":-1}}"#,
        br#"{"Pong":{"nonce":9}}"#, // a server message from a client
        &[0xff, 0xfe],
    ];
    for frame in bad {
        assert_eq!(
            decode_client(frame, DEFAULT_MAX_CLIENT_FRAME),
            Err(DecodeError::Malformed),
            "{frame:?}"
        );
    }
    // Deep nesting is bounded by the decoder, not by the stack.
    let deep: Vec<u8> = vec![b'['; 3000];
    assert_eq!(decode_client(&deep, 4096), Err(DecodeError::Malformed));
}

proptest! {
    #[test]
    fn arbitrary_bytes_never_panic(bytes in proptest::collection::vec(any::<u8>(), 0..600)) {
        let client = decode_client(&bytes, 512);
        let server = decode_server(&bytes, 512);
        if bytes.len() > 512 {
            prop_assert!(matches!(client, Err(DecodeError::TooLarge { .. })), "client");
            prop_assert!(matches!(server, Err(DecodeError::TooLarge { .. })), "server");
        }
    }

    #[test]
    fn arbitrary_json_like_text_never_panics(text in r#"[{}\[\]":,0-9a-zA-Z ]{0,200}"#) {
        let _ = decode_client(text.as_bytes(), DEFAULT_MAX_CLIENT_FRAME);
        let _ = decode_server(text.as_bytes(), DEFAULT_MAX_CLIENT_FRAME);
    }
}
