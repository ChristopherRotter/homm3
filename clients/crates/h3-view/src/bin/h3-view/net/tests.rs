//! The connection's deadlines and what counts as heard (E17 M5 review R1-3).

use std::net::TcpListener;
use std::time::Duration;

use super::*;

/// A server that takes the TCP connection and never answers the handshake fails the dial within
/// its wait.
#[test]
fn a_silent_server_fails_the_dial_within_its_wait() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("ws://{}", listener.local_addr().unwrap());
    let connection =
        Connection::open_within(&address, RootCertStore::empty, Duration::from_millis(200));
    // The listener's backlog holds the connection: nothing is ever read from it.
    match connection.wait(Duration::from_secs(5)) {
        Some(Incoming::Closed(why)) => assert!(why.contains("no answer to the handshake"), "{why}"),
        other => panic!("{other:?}"),
    }
    drop(listener);
}

/// A server's ping counts as heard, though it carries no message.
#[test]
fn a_ping_is_heard() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("ws://{}", listener.local_addr().unwrap());
    let server = std::thread::Builder::new()
        .spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut socket = tungstenite::accept(stream).unwrap();
            socket.send(Message::Ping(Vec::new().into())).unwrap();
            // Until the client goes.
            while socket.read().is_ok() {}
        })
        .unwrap();
    let connection = Connection::open(&address, RootCertStore::empty);
    assert!(matches!(
        connection.wait(Duration::from_secs(5)),
        Some(Incoming::Heard)
    ));
    drop(connection);
    server.join().unwrap();
}

/// An address needs no lookup; a name's lookup that outlasts its wait fails the dial's step, one
/// that fails says why (E17 M5 T8b2, review R1's rest).
#[test]
fn the_name_lookup_waits_at_most_its_wait() {
    let local = lookup("127.0.0.1", 9, Duration::ZERO).unwrap();
    assert_eq!(local, vec![SocketAddr::from(([127, 0, 0, 1], 9))]);
    let v6 = lookup("::1", 9, Duration::ZERO).unwrap();
    assert_eq!(v6, vec![SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 1], 9))]);
    let slow = || {
        std::thread::sleep(Duration::from_secs(2));
        Ok(Vec::new())
    };
    // Its thread sleeps on after the caller stopped waiting.
    let late = lookup_by(slow, Duration::from_millis(50)).unwrap_err();
    assert!(late.contains("took longer than 50ms"), "{late}");
    let refused = || Err(io::Error::other("no such name"));
    assert_eq!(
        lookup_by(refused, Duration::from_secs(5)).unwrap_err(),
        "no such name"
    );
    let found = || Ok(vec![SocketAddr::from(([10, 0, 0, 1], 80))]);
    assert_eq!(
        lookup_by(found, Duration::from_secs(5)).unwrap(),
        vec![SocketAddr::from(([10, 0, 0, 1], 80))]
    );
}
