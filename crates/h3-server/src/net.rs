//! The network shell (ADR-0018 points 2, 3 and 9): a listener whose connections are counted, and a
//! task per connection over socket, outbound queue and idle timer. The protocol's hello gate
//! (`h3_protocol::connection`) and the lobby's messages are the task's; a game's messages go to
//! the thread of the game the connection is bound to (E16 M2 T1).

use std::collections::BTreeMap;
use std::future::Future;
use std::net::IpAddr;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use h3_protocol::codec;
use h3_protocol::connection::{Connection, Received};
use h3_protocol::session::ConnId;
use h3_protocol::wire::{ClientMessage, GoodbyeReason, Refusal, RejectCode, ServerMessage};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::time::{Instant, sleep_until, timeout};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;

use crate::config::Limits;
use crate::game::{Frame, ToGame};
use crate::registry::{self, Shared, lock};

/// What a transport received.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Incoming {
    /// One text message.
    Text(String),
    /// One binary message (not this protocol).
    Binary,
}

/// One connection's messages, whatever carries them (ADR-0018 point 3: WebSocket now; another
/// transport, native TLS for one, later behind the same trait).
pub(crate) trait Transport: Send {
    /// The next message; `None` once the peer closed or the connection failed.
    fn recv(&mut self) -> impl Future<Output = Option<Incoming>> + Send;
    /// Sends one text message; `false` if the connection failed.
    fn send(&mut self, text: String) -> impl Future<Output = bool> + Send;
    /// Closes the connection.
    fn close(&mut self) -> impl Future<Output = ()> + Send;
}

impl Transport for WebSocketStream<TcpStream> {
    async fn recv(&mut self) -> Option<Incoming> {
        loop {
            match self.next().await? {
                Ok(Message::Text(text)) => return Some(Incoming::Text(text.as_str().to_owned())),
                Ok(Message::Binary(_)) => return Some(Incoming::Binary),
                Ok(Message::Ping(_) | Message::Pong(_) | Message::Frame(_)) => {}
                Ok(Message::Close(_)) | Err(_) => return None,
            }
        }
    }

    async fn send(&mut self, text: String) -> bool {
        SinkExt::send(self, Message::text(text)).await.is_ok()
    }

    async fn close(&mut self) {
        // The close frame's write is bounded too: a peer that stops reading would hold the task,
        // and its counted place, forever (the M1 review).
        let _ = timeout(LINGER, WebSocketStream::close(self, None)).await;
        // Read on until the peer's close, for a moment: a socket closed over unread data is reset,
        // and the reset makes the peer drop what it has not read yet, the goodbye with it.
        let drain = async { while let Some(Ok(_)) = self.next().await {} };
        let _ = timeout(LINGER, drain).await;
    }
}

/// How long a closing connection reads on for the peer's close.
const LINGER: Duration = Duration::from_secs(1);

/// The WebSocket's own bounds: well above the protocol's frame limit, so that a frame just over
/// it is answered with `Goodbye { TooLarge }`, and anything larger is dropped unread.
fn websocket_config(limits: &Limits) -> WebSocketConfig {
    let cap = limits.max_frame.saturating_mul(16).max(1 << 16);
    WebSocketConfig::default()
        .max_message_size(Some(cap))
        .max_frame_size(Some(cap))
}

/// Messages per second of one connection.
#[derive(Debug)]
struct Rate {
    /// The limit.
    limit: u32,
    /// Where the current second started.
    since: Instant,
    /// Messages in it.
    count: u32,
}

impl Rate {
    fn new(limit: u32) -> Self {
        Self {
            limit,
            since: Instant::now(),
            count: 0,
        }
    }

    /// Counts a message at `now`; `false` beyond the limit.
    fn admit(&mut self, now: Instant) -> bool {
        if now.duration_since(self.since) >= Duration::from_secs(1) {
            self.since = now;
            self.count = 0;
        }
        self.count = self.count.saturating_add(1);
        self.count <= self.limit
    }
}

/// The game a connection is bound to (E16 M2 T1): its inbox and the connection's queue for it.
#[derive(Debug)]
struct Bound {
    game: u32,
    to_game: mpsc::Sender<ToGame>,
    from_game: mpsc::Receiver<Frame>,
}

/// What the connection's task waits for.
enum Wake {
    /// The client sent something, or closed (`None`).
    Client(Option<Incoming>),
    /// The bound game sent a frame, or dropped the queue (`None`: too slow).
    Game(Option<Frame>),
    /// The idle timeout passed.
    Idle,
}

/// The next frame of the bound game; never, while the connection is in no game.
async fn from_game(bound: &mut Option<Bound>) -> Option<Frame> {
    match bound {
        Some(bound) => bound.from_game.recv().await,
        None => std::future::pending().await,
    }
}

/// One connection's task: reads frames through the hello gate, answers the lobby's messages and
/// passes the game's to the game it is bound to, writes the game's frames out, until either side
/// closes. A message within the idle timeout keeps it open; more messages per second than the
/// limit, a frame that does not decode, a full outbound queue (the game drops the queue) or a
/// write that stalls for the idle timeout close it.
pub(crate) async fn serve<T: Transport>(
    mut transport: T,
    conn: ConnId,
    origin: IpAddr,
    registry: Shared,
    limits: Limits,
    server: Arc<str>,
) {
    let idle = Duration::from_millis(limits.idle_timeout_ms);
    let mut deadline = Instant::now() + idle;
    let mut gate = Connection::new(limits.max_frame);
    let mut rate = Rate::new(limits.messages_per_second);
    let mut link = Link {
        conn,
        origin,
        registry,
        queue: limits.outbound_queue.max(1),
        bound: None,
    };
    loop {
        let wake = tokio::select! {
            incoming = transport.recv() => Wake::Client(incoming),
            frame = from_game(&mut link.bound) => Wake::Game(frame),
            () = sleep_until(deadline) => Wake::Idle,
        };
        let frame = match wake {
            Wake::Client(None) => break,
            Wake::Client(Some(incoming)) => {
                deadline = Instant::now() + idle;
                match incoming {
                    Incoming::Binary => goodbye(GoodbyeReason::Malformed),
                    Incoming::Text(_) if !rate.admit(Instant::now()) => {
                        goodbye(GoodbyeReason::RateLimited)
                    }
                    Incoming::Text(text) => match gate.receive(text.as_bytes(), &server) {
                        Received::Game(message) => match link.route(message).await {
                            Routed::Reply(frame) => frame,
                            Routed::Passed => continue,
                            Routed::GameGone => break,
                        },
                        Received::Reply { frame, close } => Frame { text: frame, close },
                    },
                }
            }
            Wake::Game(frame) => frame.unwrap_or_else(|| goodbye(GoodbyeReason::TooSlow)),
            Wake::Idle => goodbye(GoodbyeReason::IdleTimeout),
        };
        let sent = timeout(idle, transport.send(frame.text)).await;
        if frame.close || !matches!(sent, Ok(true)) {
            break;
        }
    }
    link.unbind().await;
    transport.close().await;
}

/// What became of a message for the lobby or a game.
enum Routed {
    /// An answer to send.
    Reply(Frame),
    /// Passed to the bound game, which answers.
    Passed,
    /// The bound game is gone: the connection closes.
    GameGone,
}

/// A connection's link to the lobby and to the game it is bound to (E16 M2 T1).
struct Link {
    conn: ConnId,
    origin: IpAddr,
    registry: Shared,
    /// The capacity of the connection's queue for a game.
    queue: usize,
    bound: Option<Bound>,
}

impl Link {
    /// Answers a lobby message, or passes a game message to the game it names or is bound to. A
    /// connection is in at most one game: joining or creating another leaves the first, as a
    /// disconnect would (its seat waits for its token).
    async fn route(&mut self, message: ClientMessage) -> Routed {
        match message {
            ClientMessage::ListGames { seq } => Routed::Reply(Frame {
                text: lock(&self.registry).list(seq),
                close: false,
            }),
            ClientMessage::CreateGame { seq, offer, seat } => {
                match registry::create(&self.registry, self.origin, &offer, seat) {
                    Err(reason) => Routed::Reply(refused(reason)),
                    Ok((game, to_game)) => {
                        let join = ClientMessage::Join {
                            game,
                            seat,
                            token: None,
                        };
                        let created = ServerMessage::GameCreated { seq, game };
                        self.switch(game, to_game, join, Some(created)).await
                    }
                }
            }
            ClientMessage::Join {
                game,
                seat,
                ref token,
            } if self.bound.as_ref().is_none_or(|bound| bound.game != game) => {
                match registry::bind(&self.registry, game, seat, token.as_deref()) {
                    Err(reason) => Routed::Reply(refused(reason)),
                    Ok(to_game) => self.switch(game, to_game, message, None).await,
                }
            }
            message => match &self.bound {
                Some(bound) => {
                    let conn = self.conn;
                    let passed = bound.to_game.send(ToGame::Message { conn, message }).await;
                    if passed.is_ok() {
                        Routed::Passed
                    } else {
                        Routed::GameGone
                    }
                }
                None => Routed::Reply(unbound(&message)),
            },
        }
    }

    /// Leaves the bound game for `game` (whose binding the registry counted already), passes it
    /// `join`, and answers `first` if there is one; a game that is gone meanwhile refuses.
    async fn switch(
        &mut self,
        game: u32,
        to_game: mpsc::Sender<ToGame>,
        join: ClientMessage,
        first: Option<ServerMessage>,
    ) -> Routed {
        self.unbind().await;
        let conn = self.conn;
        let (to, from_game) = mpsc::channel(self.queue);
        let opened = to_game.send(ToGame::Open { conn, to }).await.is_ok()
            && to_game
                .send(ToGame::Message {
                    conn,
                    message: join,
                })
                .await
                .is_ok();
        if !opened {
            lock(&self.registry).unbind(game, &to_game);
            return Routed::Reply(refused(Refusal::NoSuchGame));
        }
        self.bound = Some(Bound {
            game,
            to_game,
            from_game,
        });
        match first {
            Some(message) => Routed::Reply(Frame {
                text: codec::encode_text(&message),
                close: false,
            }),
            None => Routed::Passed,
        }
    }

    /// Leaves the bound game, if any: the game hears the connection closed.
    async fn unbind(&mut self) {
        if let Some(bound) = self.bound.take() {
            lock(&self.registry).unbind(bound.game, &bound.to_game);
            let conn = self.conn;
            let _ = bound.to_game.send(ToGame::Closed { conn }).await;
        }
    }
}

/// `Refused` for a lobby's or a join's refusal; the connection stays open.
fn refused(reason: Refusal) -> Frame {
    Frame {
        text: codec::encode_text(&ServerMessage::Refused { reason }),
        close: false,
    }
}

/// The answer to a game's message from a connection that is in no game: as a game answers a
/// connection without a seat (`NotSeated`; `Leave` closes).
fn unbound(message: &ClientMessage) -> Frame {
    let seq = match message {
        ClientMessage::Leave {} => return goodbye(GoodbyeReason::Left),
        ClientMessage::Command { seq, .. }
        | ClientMessage::Query { seq, .. }
        | ClientMessage::FetchReplay { seq }
        | ClientMessage::FetchRecord { seq }
        | ClientMessage::Pause { seq }
        | ClientMessage::ListGames { seq }
        | ClientMessage::CreateGame { seq, .. } => *seq,
        ClientMessage::Hello { .. } | ClientMessage::Ping { .. } | ClientMessage::Join { .. } => 0,
    };
    Frame {
        text: codec::encode_text(&ServerMessage::Rejected {
            n: 0,
            seq,
            code: RejectCode::NotSeated,
        }),
        close: false,
    }
}

/// The last frame on a connection.
fn goodbye(reason: GoodbyeReason) -> Frame {
    Frame {
        text: codec::encode_text(&ServerMessage::Goodbye { reason }),
        close: true,
    }
}

/// The open connections, in total and per address.
#[derive(Debug, Default)]
struct Counts {
    total: usize,
    by_address: BTreeMap<IpAddr, usize>,
}

/// A counted connection; dropping it releases its place.
#[derive(Debug)]
struct Place {
    counts: Arc<Mutex<Counts>>,
    address: IpAddr,
}

impl Place {
    /// A place for a connection from `address`, if the limits leave one.
    fn take(counts: &Arc<Mutex<Counts>>, address: IpAddr, limits: &Limits) -> Option<Self> {
        let mut c = counts.lock().unwrap_or_else(PoisonError::into_inner);
        let here = c.by_address.get(&address).copied().unwrap_or(0);
        if c.total >= limits.connections || here >= limits.connections_per_address {
            return None;
        }
        c.total += 1;
        c.by_address.insert(address, here + 1);
        Some(Self {
            counts: Arc::clone(counts),
            address,
        })
    }
}

impl Drop for Place {
    fn drop(&mut self) {
        let mut c = self.counts.lock().unwrap_or_else(PoisonError::into_inner);
        c.total = c.total.saturating_sub(1);
        if let Some(n) = c.by_address.get_mut(&self.address) {
            *n = n.saturating_sub(1);
            if *n == 0 {
                c.by_address.remove(&self.address);
            }
        }
    }
}

/// Accepts connections forever: a connection beyond the limits is closed at once, every other
/// one gets its WebSocket handshake and its task.
pub(crate) async fn listen(
    listener: TcpListener,
    registry: Shared,
    limits: Limits,
    server: Arc<str>,
) {
    let counts = Arc::new(Mutex::new(Counts::default()));
    let mut next = 0u64;
    loop {
        let Ok((stream, peer)) = listener.accept().await else {
            continue;
        };
        let Some(place) = Place::take(&counts, peer.ip(), &limits) else {
            drop(stream);
            continue;
        };
        next += 1;
        let conn = ConnId(next);
        let (registry, server) = (Arc::clone(&registry), Arc::clone(&server));
        tokio::spawn(async move {
            let config = websocket_config(&limits);
            let handshake = tokio_tungstenite::accept_async_with_config(stream, Some(config));
            let idle = Duration::from_millis(limits.idle_timeout_ms);
            if let Ok(Ok(socket)) = timeout(idle, handshake).await {
                serve(socket, conn, peer.ip(), registry, limits, server).await;
            }
            drop(place);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::Registry;

    /// A transport over channels.
    #[derive(Debug)]
    struct Fake {
        inbound: mpsc::UnboundedReceiver<Incoming>,
        outbound: mpsc::UnboundedSender<String>,
    }

    impl Transport for Fake {
        async fn recv(&mut self) -> Option<Incoming> {
            self.inbound.recv().await
        }

        async fn send(&mut self, text: String) -> bool {
            self.outbound.send(text).is_ok()
        }

        async fn close(&mut self) {}
    }

    #[tokio::test]
    async fn a_connection_whose_queue_the_game_dropped_is_too_slow() {
        let (client, inbound) = mpsc::unbounded_channel();
        let (outbound, mut sent) = mpsc::unbounded_channel();
        let (game, mut inbox) = mpsc::channel(8);
        let limits = Limits {
            games: 1,
            games_per_address: 1,
            ..Limits::default()
        };
        let mut registry = Registry::new(vec![], &limits, crate::config::Server::default());
        registry.host(7, game);
        let registry = Arc::new(Mutex::new(registry));
        let fake = Fake { inbound, outbound };
        let origin = IpAddr::from([127, 0, 0, 1]);
        let task = tokio::spawn(serve(
            fake,
            ConnId(4),
            origin,
            Arc::clone(&registry),
            Limits::default(),
            "s".into(),
        ));
        for message in [
            ClientMessage::Hello {
                protocol: h3_protocol::PROTOCOL_VERSION,
                client: String::new(),
                name: String::new(),
            },
            ClientMessage::Join {
                game: 7,
                seat: h3_protocol::wire::Seat::Attacker,
                token: None,
            },
        ] {
            client
                .send(Incoming::Text(codec::encode_text(&message)))
                .unwrap();
        }
        assert!(sent.recv().await.unwrap().contains("Welcome"));
        let Some(ToGame::Open { conn, to }) = inbox.recv().await else {
            panic!("no open")
        };
        assert_eq!(conn, ConnId(4));
        assert!(matches!(
            inbox.recv().await,
            Some(ToGame::Message {
                message: ClientMessage::Join { game: 7, .. },
                ..
            })
        ));
        to.send(Frame {
            text: String::from("first"),
            close: false,
        })
        .await
        .unwrap();
        drop(to);
        assert_eq!(sent.recv().await.as_deref(), Some("first"));
        let too_slow = goodbye(GoodbyeReason::TooSlow).text;
        assert_eq!(sent.recv().await, Some(too_slow));
        assert!(matches!(
            inbox.recv().await,
            Some(ToGame::Closed { conn: ConnId(4) })
        ));
        task.await.unwrap();
        // The connection left the game it was bound to.
        assert!(lock(&registry).close_if_idle(7, h3_protocol::wire::Phase::Waiting));
    }

    #[test]
    fn the_rate_counts_messages_per_second() {
        let start = Instant::now();
        let mut rate = Rate::new(2);
        rate.since = start;
        assert!(rate.admit(start));
        assert!(rate.admit(start + Duration::from_millis(500)));
        assert!(!rate.admit(start + Duration::from_millis(900)));
        assert!(
            rate.admit(start + Duration::from_millis(1000)),
            "a new second"
        );
    }

    #[test]
    fn places_are_limited_in_total_and_per_address_and_released() {
        let counts = Arc::new(Mutex::new(Counts::default()));
        let limits = Limits {
            connections: 3,
            connections_per_address: 2,
            ..Limits::default()
        };
        let (a, b): (IpAddr, IpAddr) = ([127, 0, 0, 1].into(), [10, 0, 0, 1].into());
        let first = Place::take(&counts, a, &limits).unwrap();
        let second = Place::take(&counts, a, &limits).unwrap();
        assert!(Place::take(&counts, a, &limits).is_none(), "per address");
        let third = Place::take(&counts, b, &limits).unwrap();
        assert!(Place::take(&counts, b, &limits).is_none(), "in total");
        drop(first);
        assert!(Place::take(&counts, a, &limits).is_some());
        drop((second, third));
        let c = counts.lock().unwrap();
        assert_eq!((c.total, c.by_address.len()), (0, 0));
    }
}
