//! A connection to a server (E17 M5 T6a; the user, 2026-10-30: the server's stack, blocking on
//! the I/O thread only): the WebSocket on a thread of its own, which dials the address (`ws://`
//! plain, `wss://` through rustls with graviola's cryptography and the operating system's roots,
//! as the server dials its peers, ADR-0021 point 4), sends what the caller queues and passes on
//! every message the server sends. The caller's thread (the window's, the headless command's)
//! never waits on the socket: it queues ([`Connection::send`]) and takes what came
//! ([`Connection::poll`]). The protocol's own checks are `h3_view::remote`'s; this module adds
//! the cryptography they take ([`verify`], [`nonce`]). An adapter.

use std::io::{self, ErrorKind};
use std::net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, TryRecvError};
use std::time::Duration;

use ed25519_dalek::{Signature, VerifyingKey};
use h3_protocol::account::{KEY_BYTES, NONCE_BYTES, SIGNATURE_BYTES};
use h3_protocol::codec;
use h3_protocol::wire::{ClientMessage, ServerMessage};
use rustls::{ClientConfig, RootCertStore};
use tungstenite::http::Uri;
use tungstenite::protocol::WebSocketConfig;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Connector, HandshakeError, Message, WebSocket};

/// The largest message read (*provisional*): a record comes whole in one, and a server bounds a
/// viewer's record at 512 KiB by default (`record_bytes`, up to 256 MiB).
pub const MAX_MESSAGE: usize = 64 << 20;

/// How long the I/O thread's read waits before it looks at the queue again.
const POLL: Duration = Duration::from_millis(5);

/// The longest a dial waits for the TCP connection, and then for each read of the TLS and
/// WebSocket handshakes (E17 M5 review R1-3, *provisional*): a silent server fails the dial.
pub const DIAL_WAIT: Duration = Duration::from_secs(10);

/// The longest a connection may be silent, pings included, before it counts as dropped (E17 M5
/// review R1-3, *provisional*): twice the server's ping interval, which neither the welcome nor
/// the client's options carry; 60 s, twice `h3-server`'s default `ping_timeout_ms`, four times
/// its `ping_every_ms`.
pub const SILENT: Duration = Duration::from_secs(60);

/// What came from the connection.
#[derive(Debug)]
pub enum Incoming {
    /// A message from the server.
    Message(Box<ServerMessage>),
    /// A frame without a message (a ping, a pong): the server is there (E17 M5 review R1-3; the
    /// server pings an idle connection, `h3-server`'s `ping_every_ms`).
    Heard,
    /// The connection ended, and why; nothing comes after it.
    Closed(String),
}

/// One connection, its socket on its I/O thread.
#[derive(Debug)]
pub struct Connection {
    /// The queue to the I/O thread; dropped, the thread closes the socket.
    out: Sender<ClientMessage>,
    incoming: Receiver<Incoming>,
}

type Socket = WebSocket<MaybeTlsStream<TcpStream>>;

impl Connection {
    /// Dials `address` (`ws://…` or `wss://…`) on a thread of its own, a `wss://` server's
    /// certificate checked against the roots `roots` gives ([`os_roots`]; tests pass their own).
    /// A failure to dial arrives as [`Incoming::Closed`], a silent server's after [`DIAL_WAIT`].
    pub fn open(address: &str, roots: fn() -> RootCertStore) -> Self {
        Self::open_within(address, roots, DIAL_WAIT)
    }

    /// As [`Connection::open`], the dial waiting at most `wait` for the TCP connection and for
    /// each handshake read (a test's shorter).
    pub fn open_within(address: &str, roots: fn() -> RootCertStore, wait: Duration) -> Self {
        let (out, queued) = mpsc::channel();
        let (sender, incoming) = mpsc::channel();
        let address = address.to_owned();
        let failed = sender.clone();
        let spawned = std::thread::Builder::new()
            .name("h3-view connection".to_owned())
            .spawn(move || {
                let why = match dial(&address, roots, wait) {
                    Ok(socket) => serve(socket, &queued, &sender),
                    Err(why) => why,
                };
                // The caller may have gone already.
                let _ = sender.send(Incoming::Closed(why));
            });
        if let Err(e) = spawned {
            let _ = failed.send(Incoming::Closed(format!("the connection's thread: {e}")));
        }
        Self { out, incoming }
    }

    /// Queues `message` for the server (dropped once the connection ended).
    pub fn send(&self, message: ClientMessage) {
        let _ = self.out.send(message);
    }

    /// What came, if anything, without waiting (E17 M5 T6b1: the window's, each frame).
    pub fn poll(&self) -> Option<Incoming> {
        match self.incoming.try_recv() {
            Ok(incoming) => Some(incoming),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(ended()),
        }
    }

    /// What comes within `wait` (a headless caller's).
    pub fn wait(&self, wait: Duration) -> Option<Incoming> {
        match self.incoming.recv_timeout(wait) {
            Ok(incoming) => Some(incoming),
            Err(RecvTimeoutError::Timeout) => None,
            Err(RecvTimeoutError::Disconnected) => Some(ended()),
        }
    }
}

/// What a poll after the connection's end gets.
fn ended() -> Incoming {
    Incoming::Closed("the connection ended".to_owned())
}

/// The WebSocket at `address`: the name's lookup, TCP, then TLS for `wss://`, then the handshake,
/// each step waiting at most `wait` for the connection or for a read (E17 M5 review R1-3; the
/// lookup's since T8b2: [`lookup`]); the socket's reads wait at most [`POLL`] from then on.
fn dial(address: &str, roots: fn() -> RootCertStore, wait: Duration) -> Result<Socket, String> {
    let failed = |e: &dyn std::fmt::Display| format!("{address}: {e}");
    let uri: Uri = address.parse().map_err(|e| failed(&e))?;
    let secure = match uri.scheme_str() {
        Some("wss") => true,
        Some("ws") => false,
        _ => return Err(failed(&"not a ws:// or wss:// address")),
    };
    let host = uri.host().ok_or_else(|| failed(&"no host"))?;
    let host = host.trim_start_matches('[').trim_end_matches(']');
    let port = uri.port_u16().unwrap_or(if secure { 443 } else { 80 });
    // Each of the name's addresses in turn, as `TcpStream::connect` tries them.
    let mut why = "the name has no address".to_owned();
    let mut connected = None;
    for at in lookup(host, port, wait).map_err(|e| failed(&e))? {
        match TcpStream::connect_timeout(&at, wait) {
            Ok(stream) => {
                connected = Some(stream);
                break;
            }
            Err(e) => why = e.to_string(),
        }
    }
    let stream = connected.ok_or_else(|| failed(&why))?;
    stream.set_nodelay(true).map_err(|e| failed(&e))?;
    // The socket's options are shared by its handles: the handshake's reads wait at most `wait`
    // (a timeout interrupts it: it fails), the I/O thread's at most `POLL`, set through this one.
    let handle = stream.try_clone().map_err(|e| failed(&e))?;
    handle
        .set_read_timeout(Some(wait))
        .map_err(|e| failed(&e))?;
    let connector = if secure {
        Connector::Rustls(tls(roots()).map_err(|e| failed(&e))?)
    } else {
        Connector::Plain
    };
    let bounds = WebSocketConfig::default()
        .max_message_size(Some(MAX_MESSAGE))
        .max_frame_size(Some(MAX_MESSAGE));
    let (socket, _) =
        tungstenite::client_tls_with_config(address, stream, Some(bounds), Some(connector))
            .map_err(|e| match e {
                HandshakeError::Interrupted(_) => {
                    failed(&format!("no answer to the handshake within {wait:?}"))
                }
                HandshakeError::Failure(e) => failed(&e),
            })?;
    handle
        .set_read_timeout(Some(POLL))
        .map_err(|e| failed(&e))?;
    Ok(socket)
}

/// The addresses of `host` at `port` (E17 M5 T8b2, review R1's rest): an address as it is, a
/// name looked up by the system on a thread of its own and waited for at most `wait` (the
/// system's lookup has no deadline the caller sets; one past `wait` is left to end by itself).
fn lookup(host: &str, port: u16, wait: Duration) -> Result<Vec<SocketAddr>, String> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(vec![SocketAddr::new(ip, port)]);
    }
    let name = host.to_owned();
    lookup_by(
        move || {
            (name.as_str(), port)
                .to_socket_addrs()
                .map(Iterator::collect)
        },
        wait,
    )
}

/// What `resolve` gives on a thread of its own, waited for at most `wait`.
fn lookup_by(
    resolve: impl FnOnce() -> io::Result<Vec<SocketAddr>> + Send + 'static,
    wait: Duration,
) -> Result<Vec<SocketAddr>, String> {
    let (found, take) = mpsc::channel();
    std::thread::Builder::new()
        .name("h3-view lookup".to_owned())
        .spawn(move || {
            // The caller may have stopped waiting: nothing to tell then.
            let _ = found.send(resolve());
        })
        .map_err(|e| e.to_string())?;
    match take.recv_timeout(wait) {
        Ok(addresses) => addresses.map_err(|e| e.to_string()),
        Err(_) => Err(format!("the name's lookup took longer than {wait:?}")),
    }
}

/// The TLS client's configuration (ADR-0021 point 4): graviola's cryptography, the certificates
/// checked against `roots`.
fn tls(roots: RootCertStore) -> Result<Arc<ClientConfig>, rustls::Error> {
    let provider = Arc::new(rustls_graviola::default_provider());
    let config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()?
        .with_root_certificates(roots)
        .with_no_client_auth();
    Ok(Arc::new(config))
}

/// The operating system's root certificates, which a `wss://` server's certificate is checked
/// against (the server's `peers::os_roots`).
pub fn os_roots() -> RootCertStore {
    let found = rustls_native_certs::load_native_certs();
    for error in &found.errors {
        eprintln!("h3-view: the operating system's root certificates: {error}");
    }
    let mut roots = RootCertStore::empty();
    roots.add_parsable_certificates(found.certs);
    roots
}

/// The I/O thread's loop: what was queued sent, what the server sends passed on, until either
/// side closes; why it ended.
fn serve(
    mut socket: Socket,
    queued: &Receiver<ClientMessage>,
    incoming: &Sender<Incoming>,
) -> String {
    loop {
        loop {
            match queued.try_recv() {
                Ok(message) => {
                    let text = Message::text(codec::encode_text(&message));
                    if let Err(e) = socket.send(text) {
                        return format!("send: {e}");
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    let _ = socket.close(None);
                    let _ = socket.flush();
                    return "closed by the client".to_owned();
                }
            }
        }
        match socket.read() {
            Ok(Message::Text(text)) => match codec::decode_server(text.as_bytes(), MAX_MESSAGE) {
                Ok(message) => {
                    let _ = incoming.send(Incoming::Message(Box::new(message)));
                }
                Err(e) => return format!("a malformed message from the server: {e:?}"),
            },
            // A ping is answered by tungstenite at the next read or write; a close is answered
            // too, and the next read ends. Either says the server is there (E17 M5 review R1-3).
            Ok(_) => {
                let _ = incoming.send(Incoming::Heard);
            }
            Err(tungstenite::Error::Io(e))
                if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
            Err(tungstenite::Error::ConnectionClosed) => {
                return "the server closed the connection".to_owned();
            }
            Err(e) => return format!("read: {e}"),
        }
    }
}

/// A fresh nonce for the hello, from the operating system.
pub fn nonce() -> Result<[u8; NONCE_BYTES], String> {
    let mut nonce = [0; NONCE_BYTES];
    getrandom::fill(&mut nonce).map_err(|e| format!("the OS's randomness: {e}"))?;
    Ok(nonce)
}

/// Whether `signature` by `key` over `message` verifies, strictly (no weak key, the server's
/// check: `accounts::Ed25519`).
pub fn verify(key: &[u8; KEY_BYTES], message: &[u8], signature: &[u8; SIGNATURE_BYTES]) -> bool {
    VerifyingKey::from_bytes(key).is_ok_and(|key| {
        !key.is_weak()
            && key
                .verify_strict(message, &Signature::from_bytes(signature))
                .is_ok()
    })
}

#[cfg(test)]
mod tests;
