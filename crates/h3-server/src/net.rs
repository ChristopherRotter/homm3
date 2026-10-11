//! The network shell (ADR-0018 points 2, 3 and 9): a listener whose connections are counted, and a
//! task per connection over socket, outbound queue and idle timer. The protocol's hello gate
//! (`h3_protocol::connection`) and the lobby's messages are the task's; a game's messages go to
//! the thread of the game the connection is bound to (E16 M2 T1). When the server stops (E16 M2
//! T5a), the listener accepts no more and every connection hears `Goodbye { Shutdown }`.
//!
//! The connection's bounds (E16 M2 T5b): its messages per second, and those of its address across
//! all its connections, WebSocket pings included; the idle timeout; a deadline for the hello, then
//! one for being in a game (a connection in the lobby holds a counted place). A join the game
//! refused leaves the game, as if the connection had never joined.

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
use tokio::sync::{mpsc, watch};
use tokio::time::{Instant, sleep_until, timeout};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;

use crate::config::Limits;
use crate::game::{self, Frame, FromGame, ToGame};
use crate::registry::{self, Opening, Shared, lock};

/// What a transport received.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Incoming {
    /// One text message.
    Text(String),
    /// One binary message (not this protocol).
    Binary,
    /// A WebSocket control frame, a ping or a pong: no message, but counted in the rate (E16 M2
    /// T5b).
    Control,
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
        match self.next().await? {
            Ok(Message::Text(text)) => Some(Incoming::Text(text.as_str().to_owned())),
            Ok(Message::Binary(_)) => Some(Incoming::Binary),
            Ok(Message::Ping(_) | Message::Pong(_) | Message::Frame(_)) => Some(Incoming::Control),
            Ok(Message::Close(_)) | Err(_) => None,
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

/// Messages per second of one connection, or of one address (E16 M2 T5b).
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

/// The game a connection is bound to (E16 M2 T1): its inbox and the connection's queue for it, and
/// how many messages the connection passed it (E16 M2 T5b: a refused join's frame names its
/// number).
#[derive(Debug)]
struct Bound {
    game: u32,
    to_game: mpsc::Sender<ToGame>,
    from_game: FromGame,
    passed: u64,
}

/// What the connection's task waits for.
enum Wake {
    /// The client sent something, or closed (`None`).
    Client(Option<Incoming>),
    /// The bound game sent a frame, or dropped the queue (`None`: too slow).
    Game(Option<Frame>),
    /// The idle timeout passed, or the deadline for the hello or for a game (E16 M2 T5b).
    Idle,
    /// The server stops (E16 M2 T5a).
    Stop,
}

/// Resolves once the server stops (E16 M2 T5a); never if the signal's sender is gone unset.
pub(crate) async fn stopped(stop: &mut watch::Receiver<bool>) {
    if stop.wait_for(|&stopping| stopping).await.is_err() {
        std::future::pending::<()>().await;
    }
}

/// Resolves at `deadline`; never without one.
async fn until(deadline: Option<Instant>) {
    match deadline {
        Some(deadline) => sleep_until(deadline).await,
        None => std::future::pending().await,
    }
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
/// limits (the connection's and its address's in `place`; WebSocket pings count), a frame that does
/// not decode, no hello within its timeout or no game within the lobby's timeout after it or after
/// leaving a game it was in (a refused join does not restart it), a full outbound queue (the game
/// drops the queue), a write that stalls for the idle timeout or the server's stop (`stop`) close
/// it.
pub(crate) async fn serve<T: Transport>(
    mut transport: T,
    conn: ConnId,
    place: Place,
    registry: Shared,
    limits: Limits,
    server: Arc<str>,
    mut stop: watch::Receiver<bool>,
) {
    let idle = Duration::from_millis(limits.idle_timeout_ms);
    let lobby = Duration::from_millis(limits.lobby_timeout_ms);
    let mut deadline = Instant::now() + idle;
    // The deadline to be in a game: the hello's first, then the lobby's (E16 M2 T5b).
    let mut unjoined = Some(Instant::now() + Duration::from_millis(limits.hello_timeout_ms));
    let mut gate = Connection::new(limits.max_frame);
    let mut rate = Rate::new(limits.messages_per_second);
    let mut link = Link {
        conn,
        origin: place.address,
        registry,
        queue: (limits.outbound_queue.max(1), limits.outbound_bytes),
        bound: None,
    };
    loop {
        // The lobby's deadline waits while the connection is bound to a game, and starts afresh
        // only once it leaves a game it was in: a refused join does not restart it (E16 M2 T5e).
        if link.bound.is_none() && unjoined.is_none() {
            unjoined = Some(Instant::now() + lobby);
        }
        let lobby_due = if link.bound.is_some() { None } else { unjoined };
        let wake = tokio::select! {
            incoming = transport.recv() => Wake::Client(incoming),
            frame = from_game(&mut link.bound) => Wake::Game(frame),
            () = sleep_until(deadline) => Wake::Idle,
            () = until(lobby_due) => Wake::Idle,
            () = stopped(&mut stop) => Wake::Stop,
        };
        let now = Instant::now();
        let frame = match wake {
            Wake::Client(None) => break,
            Wake::Client(Some(_)) if !(rate.admit(now) && place.admit(now)) => {
                goodbye(GoodbyeReason::RateLimited)
            }
            Wake::Client(Some(Incoming::Control)) => continue,
            Wake::Client(Some(Incoming::Binary)) => goodbye(GoodbyeReason::Malformed),
            Wake::Client(Some(Incoming::Text(text))) => {
                deadline = now + idle;
                let greeted = gate.greeted();
                let received = gate.receive(text.as_bytes(), &server);
                if !greeted && gate.greeted() {
                    unjoined = Some(now + lobby);
                }
                match received {
                    Received::Game(message) => match link.route(message).await {
                        Routed::Reply(frame) => frame,
                        Routed::Passed => continue,
                        Routed::GameGone => break,
                    },
                    Received::Reply { frame, close } => Frame {
                        text: frame,
                        close,
                        left: None,
                        held: false,
                    },
                }
            }
            Wake::Game(Some(frame)) => {
                if frame.held {
                    unjoined = None;
                }
                if frame.left.is_some() && frame.left == link.bound.as_ref().map(|b| b.passed) {
                    link.unbind().await;
                }
                frame
            }
            Wake::Game(None) => goodbye(GoodbyeReason::TooSlow),
            Wake::Idle => goodbye(GoodbyeReason::IdleTimeout),
            Wake::Stop => goodbye(GoodbyeReason::Shutdown),
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
    /// The capacity of the connection's queue for a game, in frames and in bytes (E16 M2 T5c).
    queue: (usize, usize),
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
                left: None,
                held: false,
            }),
            ClientMessage::ListReplays { seq, from } => Routed::Reply(Frame {
                text: lock(&self.registry).replays(seq, from),
                close: false,
                left: None,
                held: false,
            }),
            ClientMessage::FetchArchived { seq, replay } => {
                Routed::Reply(archived(&self.registry, seq, &replay).await)
            }
            ClientMessage::CreateGame {
                seq,
                offer,
                seat,
                spectators,
            } => {
                let watchable = spectators.unwrap_or(true);
                let (to, from_game) = game::outbox(self.queue.0, self.queue.1);
                let asked = (offer.as_str(), seat, watchable);
                match registry::create(&self.registry, self.origin, asked, (self.conn, to)) {
                    Err(reason) => Routed::Reply(refused(reason)),
                    Ok((game, to_game)) => {
                        let created = ServerMessage::GameCreated { seq, game };
                        self.switch(game, (to_game, from_game), None, Some(created))
                            .await
                    }
                }
            }
            ClientMessage::Join {
                game,
                seat,
                ref token,
            } if self.bound.as_ref().is_none_or(|bound| bound.game != game) => {
                let token = token.clone();
                let (to, from_game) = game::outbox(self.queue.0, self.queue.1);
                let opening = Opening {
                    conn: self.conn,
                    to,
                    join: message,
                };
                let asked = (game, seat, token.as_deref());
                match registry::bind(&self.registry, asked, opening) {
                    Err(reason) => Routed::Reply(refused(reason)),
                    Ok((to_game, opening)) => {
                        self.switch(game, (to_game, from_game), opening, None).await
                    }
                }
            }
            message => match &mut self.bound {
                Some(bound) => {
                    let conn = self.conn;
                    bound.passed += 1;
                    let sent = bound.to_game.send(ToGame::Message { conn, message }).await;
                    if sent.is_ok() {
                        Routed::Passed
                    } else {
                        Routed::GameGone
                    }
                }
                None => Routed::Reply(unbound(&message)),
            },
        }
    }

    /// Leaves the bound game for `game` (whose binding the registry counted already, its inbox
    /// `to_game`), passes it `opening` unless the registry queued it already (a new or a resumed
    /// game, E16 M2 T5e), and answers `first` if there is one; a game that is gone meanwhile
    /// refuses.
    async fn switch(
        &mut self,
        game: u32,
        (to_game, from_game): (mpsc::Sender<ToGame>, FromGame),
        opening: Option<Opening>,
        first: Option<ServerMessage>,
    ) -> Routed {
        self.unbind().await;
        if let Some(Opening { conn, to, join }) = opening {
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
        }
        self.bound = Some(Bound {
            game,
            to_game,
            from_game,
            passed: 1,
        });
        match first {
            Some(message) => Routed::Reply(Frame {
                text: codec::encode_text(&message),
                close: false,
                left: None,
                held: false,
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

/// The answer to `FetchArchived` (E16 M2 T5d): the archive's replay `id` read from its file off
/// the connection's task, or `Refused { NoSuchReplay }` for an id the archive does not list (the
/// file is never named by the client) or a file gone meanwhile.
async fn archived(registry: &Shared, seq: u32, id: &str) -> Frame {
    let Some(path) = lock(registry).replay(id) else {
        return refused(Refusal::NoSuchReplay);
    };
    let read = tokio::task::spawn_blocking(move || std::fs::read_to_string(path)).await;
    match read {
        Ok(Ok(replay)) => Frame {
            text: codec::encode_text(&ServerMessage::Replay { n: 0, seq, replay }),
            close: false,
            left: None,
            held: false,
        },
        _ => refused(Refusal::NoSuchReplay),
    }
}

/// `Refused` for a lobby's or a join's refusal; the connection stays open.
fn refused(reason: Refusal) -> Frame {
    Frame {
        text: codec::encode_text(&ServerMessage::Refused { reason }),
        close: false,
        left: None,
        held: false,
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
        | ClientMessage::SetSpectators { seq, .. }
        | ClientMessage::ListGames { seq }
        | ClientMessage::CreateGame { seq, .. }
        | ClientMessage::ListReplays { seq, .. }
        | ClientMessage::FetchArchived { seq, .. } => *seq,
        ClientMessage::Hello { .. } | ClientMessage::Ping { .. } | ClientMessage::Join { .. } => 0,
    };
    Frame {
        text: codec::encode_text(&ServerMessage::Rejected {
            n: 0,
            seq,
            code: RejectCode::NotSeated,
        }),
        close: false,
        left: None,
        held: false,
    }
}

/// The last frame on a connection.
fn goodbye(reason: GoodbyeReason) -> Frame {
    Frame {
        text: codec::encode_text(&ServerMessage::Goodbye { reason }),
        close: true,
        left: None,
        held: false,
    }
}

/// One address's open connections and their messages per second (E16 M2 T5b).
#[derive(Debug)]
struct Address {
    connections: usize,
    rate: Rate,
}

/// The open connections, in total and per address.
#[derive(Debug, Default)]
struct Counts {
    total: usize,
    by_address: BTreeMap<IpAddr, Address>,
}

/// A counted connection; dropping it releases its place.
#[derive(Debug)]
pub(crate) struct Place {
    counts: Arc<Mutex<Counts>>,
    address: IpAddr,
}

impl Place {
    /// A place for a connection from `address`, if the limits leave one.
    fn take(counts: &Arc<Mutex<Counts>>, address: IpAddr, limits: &Limits) -> Option<Self> {
        let mut c = counts.lock().unwrap_or_else(PoisonError::into_inner);
        let here = c.by_address.get(&address).map_or(0, |a| a.connections);
        if c.total >= limits.connections || here >= limits.connections_per_address {
            return None;
        }
        c.total += 1;
        let rate = || Rate::new(limits.messages_per_address);
        let entry = c.by_address.entry(address);
        entry
            .or_insert_with(|| Address {
                connections: 0,
                rate: rate(),
            })
            .connections += 1;
        Some(Self {
            counts: Arc::clone(counts),
            address,
        })
    }

    /// Counts a message of the address at `now`; `false` beyond its limit (E16 M2 T5b).
    fn admit(&self, now: Instant) -> bool {
        let mut c = self.counts.lock().unwrap_or_else(PoisonError::into_inner);
        c.by_address
            .get_mut(&self.address)
            .is_some_and(|a| a.rate.admit(now))
    }
}

impl Drop for Place {
    fn drop(&mut self) {
        let mut c = self.counts.lock().unwrap_or_else(PoisonError::into_inner);
        c.total = c.total.saturating_sub(1);
        if let Some(a) = c.by_address.get_mut(&self.address) {
            a.connections = a.connections.saturating_sub(1);
            if a.connections == 0 {
                c.by_address.remove(&self.address);
            }
        }
    }
}

/// Accepts connections until the server stops (`stop`, E16 M2 T5a): a connection beyond the
/// limits is closed at once, every other one gets its WebSocket handshake and its task. Once
/// stopped, it waits for the open connections to close, for a grace.
pub(crate) async fn listen(
    listener: TcpListener,
    registry: Shared,
    limits: Limits,
    server: Arc<str>,
    mut stop: watch::Receiver<bool>,
) {
    let counts = Arc::new(Mutex::new(Counts::default()));
    let mut next = 0u64;
    loop {
        let accepted = tokio::select! {
            accepted = listener.accept() => accepted,
            () = stopped(&mut stop) => break,
        };
        let Ok((stream, peer)) = accepted else {
            continue;
        };
        let Some(place) = Place::take(&counts, peer.ip(), &limits) else {
            drop(stream);
            continue;
        };
        next += 1;
        let conn = ConnId(next);
        let (registry, server, stop) = (Arc::clone(&registry), Arc::clone(&server), stop.clone());
        tokio::spawn(async move {
            let config = websocket_config(&limits);
            let handshake = tokio_tungstenite::accept_async_with_config(stream, Some(config));
            let idle = Duration::from_millis(limits.idle_timeout_ms);
            if let Ok(Ok(socket)) = timeout(idle, handshake).await {
                serve(socket, conn, place, registry, limits, server, stop).await;
            }
        });
    }
    // Stopped: no new connection; the open ones close, each after its goodbye, within a grace.
    drop(listener);
    let grace = Instant::now() + SHUTDOWN_GRACE;
    let open = || counts.lock().unwrap_or_else(PoisonError::into_inner).total;
    while open() > 0 && Instant::now() < grace {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

/// How long a stopping server waits for its connections to close: each one's goodbye and close
/// take at most two `LINGER`s.
const SHUTDOWN_GRACE: Duration = Duration::from_secs(3);

#[cfg(test)]
mod tests {
    use h3_protocol::wire::{Phase, Seat};

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

    /// A connection's task over a fake transport, with game 7 hosted in its registry.
    struct Rig {
        /// What the client sends.
        client: mpsc::UnboundedSender<Incoming>,
        /// What the task sent the client.
        sent: mpsc::UnboundedReceiver<String>,
        /// What the task told game 7.
        inbox: mpsc::Receiver<ToGame>,
        registry: Shared,
        task: tokio::task::JoinHandle<()>,
    }

    impl Rig {
        fn new() -> Self {
            let (client, inbound) = mpsc::unbounded_channel();
            let (outbound, sent) = mpsc::unbounded_channel();
            let (game, inbox) = mpsc::channel(8);
            let limits = Limits {
                games: 1,
                games_per_address: 1,
                ..Limits::default()
            };
            let mut registry = Registry::new(vec![], &limits, crate::config::Server::default());
            registry.host(7, game);
            let registry = Arc::new(Mutex::new(registry));
            let counts = Arc::new(Mutex::new(Counts::default()));
            let place = Place::take(&counts, [127, 0, 0, 1].into(), &limits).unwrap();
            let task = tokio::spawn(serve(
                Fake { inbound, outbound },
                ConnId(4),
                place,
                Arc::clone(&registry),
                limits,
                "s".into(),
                watch::channel(false).1,
            ));
            Self {
                client,
                sent,
                inbox,
                registry,
                task,
            }
        }

        /// The client sends `message`.
        fn says(&self, message: &ClientMessage) {
            let text = codec::encode_text(message);
            self.client.send(Incoming::Text(text)).unwrap();
        }

        /// The next frame the task sent the client, within a moment.
        async fn next(&mut self) -> Option<String> {
            timeout(Duration::from_secs(5), self.sent.recv())
                .await
                .ok()?
        }

        /// The next thing the task told the game, within a moment.
        async fn told(&mut self) -> Option<ToGame> {
            timeout(Duration::from_secs(5), self.inbox.recv())
                .await
                .ok()?
        }

        /// The client's hello and its join of game 7's `seat`: the game's queue to the client.
        async fn joined(&mut self, seat: Seat) -> game::Outbox {
            self.says(&ClientMessage::Hello {
                protocol: h3_protocol::PROTOCOL_VERSION,
                client: String::new(),
                name: String::new(),
            });
            self.says(&join(seat));
            assert!(self.next().await.unwrap().contains("Welcome"));
            let Some(ToGame::Open { conn, to }) = self.told().await else {
                panic!("no open")
            };
            assert_eq!(conn, ConnId(4));
            assert!(matches!(
                self.told().await,
                Some(ToGame::Message {
                    message: ClientMessage::Join { game: 7, .. },
                    ..
                })
            ));
            to
        }
    }

    /// A join of game 7's `seat`.
    fn join(seat: Seat) -> ClientMessage {
        ClientMessage::Join {
            game: 7,
            seat,
            token: None,
        }
    }

    /// A frame from the game, `left` after the connection's `k`-th message if `Some(k)`, else to
    /// a connection in the game.
    fn frame(text: &str, left: Option<u64>) -> Frame {
        Frame {
            text: String::from(text),
            close: false,
            left,
            held: left.is_none(),
        }
    }

    #[tokio::test]
    async fn a_connection_whose_queue_the_game_dropped_is_too_slow() {
        let mut rig = Rig::new();
        let to = rig.joined(Seat::Attacker).await;
        assert!(to.offer(frame("first", None)));
        drop(to);
        assert_eq!(rig.next().await.as_deref(), Some("first"));
        let too_slow = goodbye(GoodbyeReason::TooSlow).text;
        assert_eq!(rig.next().await, Some(too_slow));
        assert!(matches!(
            rig.told().await,
            Some(ToGame::Closed { conn: ConnId(4) })
        ));
        rig.task.await.unwrap();
        // The connection left the game it was bound to.
        assert!(lock(&rig.registry).close_if_idle(7, Phase::Waiting));
    }

    /// E16 M2 T5b: a frame marked `left` after the connection's last message to the game makes it
    /// leave the game; one marked after an earlier message does not (a join that followed may
    /// have taken a seat).
    #[tokio::test]
    async fn a_refused_join_leaves_the_game_unless_another_message_followed() {
        let mut rig = Rig::new();
        let to = rig.joined(Seat::Attacker).await;
        // A second join, passed before the game's answer to the first.
        rig.says(&join(Seat::Defender));
        assert!(matches!(rig.told().await, Some(ToGame::Message { .. })));
        assert!(to.offer(frame("refused", Some(1))));
        assert!(to.offer(frame("joined", None)));
        assert_eq!(rig.next().await.as_deref(), Some("refused"));
        assert_eq!(
            rig.next().await.as_deref(),
            Some("joined"),
            "still in the game"
        );
        assert_eq!(
            to.queued(),
            0,
            "the task took off what it took out (E16 M2 T5c)"
        );
        rig.says(&join(Seat::Attacker));
        assert!(matches!(rig.told().await, Some(ToGame::Message { .. })));
        assert!(to.offer(frame("refused", Some(3))));
        assert_eq!(rig.next().await.as_deref(), Some("refused"));
        assert!(matches!(
            rig.told().await,
            Some(ToGame::Closed { conn: ConnId(4) })
        ));
        assert!(lock(&rig.registry).close_if_idle(7, Phase::Waiting));
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
        // The address's connections share its rate (E16 M2 T5b); another address has its own.
        let now = Instant::now();
        let shared = (0..limits.messages_per_address).all(|i| {
            let place = if i % 2 == 0 { &first } else { &second };
            place.admit(now)
        });
        assert!(shared && !first.admit(now) && !second.admit(now));
        assert!(third.admit(now));
        drop(first);
        assert!(Place::take(&counts, a, &limits).is_some());
        drop((second, third));
        let c = counts.lock().unwrap();
        assert_eq!((c.total, c.by_address.len()), (0, 0));
    }
}
