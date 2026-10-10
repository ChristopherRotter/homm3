//! A connection's own messages: the hello gates everything, the ping is answered, and a frame
//! that does not decode or comes out of place closes the connection.

use std::string::{String, ToString};

use crate::PROTOCOL_VERSION;
use crate::codec::{DEFAULT_MAX_CLIENT_FRAME, encode};
use crate::connection::{Connection, Received};
use crate::wire::{ClientMessage, GoodbyeReason};

fn hello(protocol: u32) -> ClientMessage {
    ClientMessage::Hello {
        protocol,
        client: "test".to_string(),
        name: "Ann".to_string(),
    }
}

fn receive(c: &mut Connection, message: &ClientMessage) -> Received {
    c.receive(&encode(message), "h3-server 0.1.0")
}

fn goodbye(reason: GoodbyeReason) -> Received {
    let reason = std::format!("{reason:?}");
    Received::Reply {
        frame: std::format!(r#"{{"Goodbye":{{"reason":"{reason}"}}}}"#),
        close: true,
    }
}

#[test]
fn the_hello_is_welcomed_then_the_game_gets_the_rest() {
    let mut c = Connection::new(DEFAULT_MAX_CLIENT_FRAME);
    assert!(!c.greeted());
    let welcome = std::format!(
        r#"{{"Welcome":{{"protocol":{PROTOCOL_VERSION},"server":"h3-server 0.1.0"}}}}"#
    );
    assert_eq!(
        receive(&mut c, &hello(PROTOCOL_VERSION)),
        Received::Reply {
            frame: welcome,
            close: false
        }
    );
    assert!(c.greeted());
    assert_eq!(
        receive(&mut c, &ClientMessage::Ping { nonce: 4 }),
        Received::Reply {
            frame: String::from(r#"{"Pong":{"nonce":4}}"#),
            close: false
        }
    );
    let leave = ClientMessage::Leave {};
    assert_eq!(receive(&mut c, &leave), Received::Game(leave));
    // A second hello is out of place.
    assert_eq!(
        receive(&mut c, &hello(PROTOCOL_VERSION)),
        goodbye(GoodbyeReason::Protocol)
    );
}

#[test]
fn another_version_or_a_message_before_the_hello_closes_the_connection() {
    for first in [
        hello(PROTOCOL_VERSION + 1),
        ClientMessage::Ping { nonce: 1 },
        ClientMessage::Leave {},
    ] {
        let mut c = Connection::new(DEFAULT_MAX_CLIENT_FRAME);
        assert_eq!(receive(&mut c, &first), goodbye(GoodbyeReason::Protocol));
        assert!(!c.greeted());
    }
}

#[test]
fn a_frame_that_does_not_decode_closes_the_connection() {
    let mut c = Connection::new(16);
    assert_eq!(
        c.receive(b"{\"Ping\":", "s"),
        goodbye(GoodbyeReason::Malformed)
    );
    assert_eq!(
        receive(&mut c, &hello(PROTOCOL_VERSION)),
        goodbye(GoodbyeReason::TooLarge)
    );
}
