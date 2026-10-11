//! A connection's own messages: the hello gates everything, the ping is answered, and a frame
//! that does not decode or comes out of place closes the connection (E16 M1 T2). The login
//! (E16 M3 T1): the gate checks an account's request's form and the connection's state, hands the
//! host what to check, and logs in only on the host's yes.

use std::string::{String, ToString};
use std::vec::Vec;

use proptest::prelude::*;

use crate::PROTOCOL_VERSION;
use crate::account::{self, login_message, rekey_message};
use crate::codec::{DEFAULT_MAX_CLIENT_FRAME, encode};
use crate::connection::{AccountRequest, Connection, Host, Received};
use crate::wire::{ClientMessage, GoodbyeReason, Refusal, Seat};

const HOST: Host<'static> = Host {
    program: "h3-server 0.1.0",
    id: "wss://h3.example.org",
};

const CHALLENGE: [u8; 32] = [7; 32];

fn hello(protocol: u32) -> ClientMessage {
    ClientMessage::Hello {
        protocol,
        client: "test".to_string(),
        name: "Ann".to_string(),
    }
}

fn receive(c: &mut Connection, message: &ClientMessage) -> Received {
    c.receive(&encode(message), &HOST)
}

/// A connection past its hello.
fn greeted(anonymous: bool) -> Connection {
    let mut c = Connection::new(DEFAULT_MAX_CLIENT_FRAME, CHALLENGE, anonymous);
    receive(&mut c, &hello(PROTOCOL_VERSION));
    c
}

fn goodbye(reason: GoodbyeReason) -> Received {
    let reason = std::format!("{reason:?}");
    Received::Reply {
        frame: std::format!(r#"{{"Goodbye":{{"reason":"{reason}"}}}}"#),
        close: true,
    }
}

fn refused(reason: Refusal) -> Received {
    let reason = std::format!("{reason:?}");
    Received::Reply {
        frame: std::format!(r#"{{"Refused":{{"reason":"{reason}"}}}}"#),
        close: false,
    }
}

fn login(login: &str, key: &str, signature: &str, display: Option<&str>) -> ClientMessage {
    ClientMessage::Login {
        seq: 3,
        login: login.to_string(),
        key: key.to_string(),
        signature: signature.to_string(),
        display: display.map(ToString::to_string),
    }
}

#[test]
fn the_hello_is_welcomed_then_the_game_gets_the_rest() {
    let mut c = Connection::new(DEFAULT_MAX_CLIENT_FRAME, CHALLENGE, true);
    assert!(!c.greeted());
    let challenge = "07".repeat(32);
    let welcome = std::format!(
        r#"{{"Welcome":{{"protocol":{PROTOCOL_VERSION},"server":"h3-server 0.1.0","id":"wss://h3.example.org","challenge":"{challenge}"}}}}"#
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
        login("ann", &"11".repeat(32), &"22".repeat(64), None),
    ] {
        let mut c = Connection::new(DEFAULT_MAX_CLIENT_FRAME, CHALLENGE, true);
        assert_eq!(receive(&mut c, &first), goodbye(GoodbyeReason::Protocol));
        assert!(!c.greeted());
    }
}

#[test]
fn a_frame_that_does_not_decode_closes_the_connection() {
    let mut c = Connection::new(16, CHALLENGE, true);
    assert_eq!(
        c.receive(b"{\"Ping\":", &HOST),
        goodbye(GoodbyeReason::Malformed)
    );
    assert_eq!(
        receive(&mut c, &hello(PROTOCOL_VERSION)),
        goodbye(GoodbyeReason::TooLarge)
    );
}

/// E16 M3 T1: a login goes to the host with the bytes its signature is over; the connection is
/// logged in only once the host says yes, and then logs in no more.
#[test]
fn a_login_is_handed_to_the_host_and_holds_only_on_its_yes() {
    let mut c = greeted(true);
    let (key, signature) = ("11".repeat(32), "22".repeat(64));
    assert_eq!(
        receive(&mut c, &login("ann", &key, &signature, None)),
        Received::Account(AccountRequest::Login {
            seq: 3,
            login: "ann".to_string(),
            key: [0x11; 32],
            signature: [0x22; 64],
            display: None,
            signed: login_message(HOST.id, &CHALLENGE, "ann", None),
        })
    );
    // A registration names its display name, which the signature covers.
    assert_eq!(
        receive(&mut c, &login("ann", &key, &signature, Some("Ann Smith"))),
        Received::Account(AccountRequest::Login {
            seq: 3,
            login: "ann".to_string(),
            key: [0x11; 32],
            signature: [0x22; 64],
            display: Some("Ann Smith".to_string()),
            signed: login_message(HOST.id, &CHALLENGE, "ann", Some("Ann Smith")),
        })
    );
    assert_eq!(c.account(), None, "not before the host's yes");
    c.logged_in("ann".to_string());
    assert_eq!(c.account(), Some("ann"));
    assert_eq!(
        receive(&mut c, &login("bob", &key, &signature, None)),
        refused(Refusal::AlreadyLoggedIn)
    );
    assert_eq!(c.account(), Some("ann"));
}

/// E16 M3 T1: a login of the wrong form is refused and changes nothing.
#[test]
fn a_login_of_the_wrong_form_is_refused_and_changes_nothing() {
    let (key, signature) = ("11".repeat(32), "22".repeat(64));
    let cases = [
        (login("an", &key, &signature, None), Refusal::BadName),
        (login("Ann", &key, &signature, None), Refusal::BadName),
        (login("ann", &key, &signature, Some("A")), Refusal::BadName),
        (login("ann", "11", &signature, None), Refusal::BadLogin),
        (
            login("ann", &key, &"2G".repeat(64), None),
            Refusal::BadLogin,
        ),
    ];
    for (message, reason) in cases {
        let mut c = greeted(true);
        let before = c.clone();
        assert_eq!(receive(&mut c, &message), refused(reason), "{message:?}");
        assert_eq!(c, before);
    }
}

/// E16 M3 T1: a re-key, a rename and the listing preference need a login, and name its account.
#[test]
fn an_accounts_requests_need_a_login() {
    let rekey = |key: &str| ClientMessage::Rekey {
        seq: 4,
        key: key.to_string(),
        by_old: "44".repeat(64),
        by_new: "55".repeat(64),
    };
    let rename = |display: &str| ClientMessage::Rename {
        seq: 5,
        display: display.to_string(),
    };
    let listed = ClientMessage::SetListed {
        seq: 6,
        listed: false,
    };
    let mut c = greeted(true);
    for message in [rekey(&"33".repeat(32)), rename("Ann B"), listed.clone()] {
        assert_eq!(receive(&mut c, &message), refused(Refusal::NotLoggedIn));
    }
    c.logged_in("ann".to_string());
    assert_eq!(
        receive(&mut c, &rekey(&"33".repeat(32))),
        Received::Account(AccountRequest::Rekey {
            seq: 4,
            login: "ann".to_string(),
            key: [0x33; 32],
            by_old: [0x44; 64],
            by_new: [0x55; 64],
            signed: rekey_message(HOST.id, &CHALLENGE, "ann", &[0x33; 32]),
        })
    );
    assert_eq!(receive(&mut c, &rekey("33")), refused(Refusal::BadLogin));
    assert_eq!(
        receive(&mut c, &rename("Ann B")),
        Received::Account(AccountRequest::Rename {
            seq: 5,
            login: "ann".to_string(),
            display: "Ann B".to_string(),
        })
    );
    assert_eq!(
        receive(&mut c, &rename("Ann  B")),
        refused(Refusal::BadName)
    );
    assert_eq!(
        receive(&mut c, &listed),
        Received::Account(AccountRequest::SetListed {
            seq: 6,
            login: "ann".to_string(),
            listed: false,
        })
    );
}

/// E16 M3 T1: on a server that takes no anonymous players, joining or creating a game waits for
/// the login; the lobby's lists do not.
#[test]
fn without_anonymous_play_a_game_waits_for_the_login() {
    let join = ClientMessage::Join {
        game: 1,
        seat: Seat::Attacker,
        token: None,
    };
    let create = ClientMessage::CreateGame {
        seq: 2,
        offer: "duel".to_string(),
        seat: Seat::Attacker,
        spectators: None,
    };
    let list = ClientMessage::ListGames { seq: 1 };
    let mut c = greeted(false);
    assert_eq!(receive(&mut c, &join), refused(Refusal::NotLoggedIn));
    assert_eq!(receive(&mut c, &create), refused(Refusal::NotLoggedIn));
    assert_eq!(receive(&mut c, &list), Received::Game(list.clone()));
    c.logged_in("ann".to_string());
    assert_eq!(receive(&mut c, &join), Received::Game(join.clone()));
    assert_eq!(receive(&mut c, &create), Received::Game(create));
    // Anonymous play: no login needed.
    assert_eq!(receive(&mut greeted(true), &join), Received::Game(join));
}

proptest! {
    /// E16 M3 T1: whatever a client sends, the gate logs it in only on the host's yes, and hands
    /// the host no request for an account it is not logged in to.
    #[test]
    fn a_client_never_logs_itself_in(
        picks in proptest::collection::vec(any::<prop::sample::Index>(), 0..24),
        anonymous in any::<bool>(),
    ) {
        let messages = super::wire::client_messages();
        let mut c = greeted(anonymous);
        for pick in picks {
            let received = receive(&mut c, pick.get(&messages));
            let request = match &received {
                Received::Account(request) => Some(request),
                Received::Game(_) | Received::Reply { .. } => None,
            };
            prop_assert!(
                request.is_none_or(|r| matches!(r, AccountRequest::Login { .. })),
                "{:?}",
                received
            );
            prop_assert_eq!(c.account(), None);
        }
    }
}

#[test]
fn hex_and_names_follow_their_rules() {
    assert_eq!(account::to_hex(&[0x00, 0xab, 0xff]), "00abff");
    assert_eq!(account::from_hex::<3>("00abff"), Some([0x00, 0xab, 0xff]));
    for bad in ["00abf", "00abff00", "00ABff", "g0abff", "0gabff"] {
        assert_eq!(account::from_hex::<3>(bad), None, "{bad}");
    }
    let twenty = "a".repeat(20);
    for good in ["ann", "a_b-c9", twenty.as_str()] {
        assert!(account::is_login_name(good), "{good}");
    }
    let long = "a".repeat(21);
    for bad in ["an", long.as_str(), "Ann", "an n", "ann!", "änn"] {
        assert!(!account::is_login_name(bad), "{bad}");
    }
    let twenty_four = "A".repeat(24);
    for good in ["Ann", "Ann Smith", "a.b-c_d", twenty_four.as_str()] {
        assert!(account::is_display_name(good), "{good}");
    }
    let long = "A".repeat(25);
    for bad in [
        "An",
        long.as_str(),
        " Ann",
        "Ann ",
        "Ann  Smith",
        "Ann!",
        "Ånn",
    ] {
        assert!(!account::is_display_name(bad), "{bad}");
    }
    assert_eq!(account::display_key("Ann Smith"), "ann smith");
}

#[test]
fn a_login_and_a_rekey_sign_their_purpose_the_server_the_challenge_and_the_names() {
    let mut expected: Vec<u8> = b"h3-login-v1".to_vec();
    expected.extend_from_slice(&[0, 0, 0, 2]);
    expected.extend_from_slice(b"id");
    expected.extend_from_slice(&[1; 32]);
    expected.extend_from_slice(&[0, 0, 0, 3]);
    expected.extend_from_slice(b"ann");
    let mut login = expected.clone();
    login.extend_from_slice(&[0, 0, 0, 0]);
    assert_eq!(login_message("id", &[1; 32], "ann", None), login);
    let mut register = expected;
    register.extend_from_slice(&[0, 0, 0, 3]);
    register.extend_from_slice(b"Ann");
    assert_eq!(login_message("id", &[1; 32], "ann", Some("Ann")), register);
    let mut rekey: Vec<u8> = b"h3-rekey-v1".to_vec();
    rekey.extend_from_slice(&[0, 0, 0, 2]);
    rekey.extend_from_slice(b"id");
    rekey.extend_from_slice(&[1; 32]);
    rekey.extend_from_slice(&[0, 0, 0, 3]);
    rekey.extend_from_slice(b"ann");
    rekey.extend_from_slice(&[9; 32]);
    assert_eq!(rekey_message("id", &[1; 32], "ann", &[9; 32]), rekey);
}
