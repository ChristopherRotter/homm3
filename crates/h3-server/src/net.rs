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
//!
//! The peer's liveness (SRV-Q20): the server pings every connection (`ping_every_ms`; a browser's
//! WebSocket answers by itself), and one it heard nothing from, not even a pong, for
//! `ping_timeout_ms` closes. A client gone without a close (a cable pulled, a network gone) leaves
//! its seat within that, not the idle timeout, and a server-side outage drops every seat within a
//! ping of each other. The time the task spends writing to the connection does not count: a large
//! frame to a slow client delays its pong.
//!
//! The login (E16 M3 T2a): an account's request is answered off the task ([`Accounts`]: the
//! signatures, the player log's sync). Failed logins count per address, also across its
//! connections' lifetimes: beyond `failed_logins_per_address` a minute, a login closes its
//! connection `RateLimited`, as does any login from the address until the minute is over. The
//! requests that may write the player log (a registration, a re-key, a rename, a preference) count
//! per address too: beyond `account_changes_per_address` an hour, one closes its connection
//! `RateLimited` (E16 M3's review: each is a synced line, and the logins wait on the log's lock).
//!
//! Behind a reverse proxy (E16 M3 T4b): a connection from one of `[server] trusted_proxies`
//! counts, after its handshake, as the client the last entry of its `X-Forwarded-For` header
//! names (*provisional*: that header alone, its last entry); until then it holds a place in the
//! total only. A forwarded header from any other peer is ignored. Every limit per address counts
//! an IPv6 address by its /64 prefix, the block one host usually has ([`counted`]).
//!
//! Accounts in games (E16 M3 T3): a connection opens a game with the account it is logged in as.
//! The server knows each account's logged-in connections and their keys: once one re-keyed the
//! account, the others logged in with another key close (`Goodbye { Rekeyed }`), before anything
//! else they sent is handled, so a session opened with a leaked password does not outlive the
//! re-key.

use std::collections::BTreeMap;
use std::future::Future;
use std::net::{IpAddr, Ipv6Addr};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use futures_util::{FutureExt, SinkExt, StreamExt};
use h3_protocol::account::KEY_BYTES;
use h3_protocol::codec;
use h3_protocol::connection::{AccountRequest, Connection, Host, Received};
use h3_protocol::session::ConnId;
use h3_protocol::wire::{
    ClientMessage, GoodbyeReason, LeaderboardRow, Refusal, RejectCode, ServerMessage,
};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Notify, mpsc, watch};
use tokio::time::{Instant, sleep_until, timeout};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::server::{
    Callback, ErrorResponse, Request, Response,
};
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;

use crate::accounts::Accounts;
use crate::config::Limits;
use crate::game::{self, Frame, FromGame, ToGame};
use crate::ladders::{self, Ladders};
use crate::registry::{self, Opening, Shared, ToConn, lock};

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
    /// Sends a ping, which the peer answers (SRV-Q20); `false` if the connection failed.
    fn ping(&mut self) -> impl Future<Output = bool> + Send;
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

    async fn ping(&mut self) -> bool {
        SinkExt::send(self, Message::Ping(Default::default()))
            .await
            .is_ok()
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

/// Messages per second of one connection, or of one address (E16 M2 T5b); an address's failed
/// logins a minute (E16 M3 T2a).
#[derive(Debug)]
struct Rate {
    /// The limit.
    limit: u32,
    /// How long a window lasts.
    window: Duration,
    /// Where the current window started.
    since: Instant,
    /// Events in it.
    count: u32,
}

impl Rate {
    /// A limit a second.
    fn new(limit: u32) -> Self {
        Self::per(limit, Duration::from_secs(1))
    }

    /// A limit per `window`.
    fn per(limit: u32, window: Duration) -> Self {
        Self {
            limit,
            window,
            since: Instant::now(),
            count: 0,
        }
    }

    /// Whether the window that holds `now` is over.
    fn over(&self, now: Instant) -> bool {
        now.duration_since(self.since) >= self.window
    }

    /// Counts an event at `now`; `false` beyond the limit.
    fn admit(&mut self, now: Instant) -> bool {
        if self.over(now) {
            self.since = now;
            self.count = 0;
        }
        self.count = self.count.saturating_add(1);
        self.count <= self.limit
    }

    /// Whether the limit is reached at `now`: the next event would be beyond it.
    fn full(&self, now: Instant) -> bool {
        !self.over(now) && self.count >= self.limit
    }
}

/// The frames a connection's queue may hold for it (E16 M4 T2): a few at a time come.
const QUEUE_FRAMES: usize = 16;

/// The window of an address's failed logins (E16 M3 T2a).
const FAILED_LOGINS_WINDOW: Duration = Duration::from_secs(60);

/// The window of an address's requests that may write the player log (E16 M3's review).
const ACCOUNT_CHANGES_WINDOW: Duration = Duration::from_secs(3600);

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
    /// Time to ping the client (SRV-Q20).
    Ping,
    /// The client said nothing, not even a pong, for the ping timeout (SRV-Q20).
    Silent,
    /// The server stops (E16 M2 T5a).
    Stop,
    /// The connection's account re-keyed on another connection (E16 M3 T3).
    Rekeyed,
    /// The connection's queue told it something (E16 M4 T2).
    Queue(Option<(String, bool)>),
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

/// What every connection's task shares (E16 M3): the server as its welcome names it, whether it
/// takes anonymous players, its accounts.
#[derive(Debug)]
pub(crate) struct Service {
    /// The program's name and version.
    pub program: String,
    /// The server's id: the address its clients dial, which a login signs.
    pub id: String,
    /// Whether a connection may join or create a game without logging in (E16 M3 T2a).
    pub anonymous: bool,
    /// The accounts (E16 M3 T2a).
    pub accounts: Arc<Accounts>,
    /// The logged-in connections by account, each with the key it logged in with and what closes
    /// it when another connection re-keys the account (E16 M3 T3).
    pub sessions: Mutex<Sessions>,
    /// The reverse proxies whose forwarded header names the client (E16 M3 T4b).
    pub proxies: Vec<IpAddr>,
    /// The ladders (E16 M4 T2): a queue goes by an account's rating on its ladder.
    pub ladders: Arc<Ladders>,
}

/// The logged-in connections by account (E16 M3 T3): the key each logged in with, and what closes
/// it when the account re-keys elsewhere.
pub(crate) type Sessions = BTreeMap<(String, ConnId), ([u8; KEY_BYTES], Arc<Notify>)>;

impl Service {
    /// The account's logged-in connections, locked.
    fn sessions(&self) -> std::sync::MutexGuard<'_, Sessions> {
        self.sessions.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The account `login` re-keyed to `key` on `conn` (E16 M3 T3): every other connection logged
    /// in as it with another key closes.
    fn rekeyed(&self, login: &str, conn: ConnId, key: [u8; KEY_BYTES]) {
        let mut sessions = self.sessions();
        let from = (login.to_owned(), ConnId(0));
        let to = (login.to_owned(), ConnId(u64::MAX));
        for ((_, other), (with, rekeyed)) in sessions.range_mut(from..=to) {
            if *other == conn {
                *with = key;
            } else if *with != key {
                rekeyed.notify_one();
            }
        }
    }
}

/// Whether the connection's account re-keyed on another connection (E16 M3 T3): takes the signal.
fn kicked(rekeyed: &Notify) -> bool {
    rekeyed.notified().now_or_never().is_some()
}

/// One connection's task: reads frames through the hello gate, answers the lobby's messages and
/// passes the game's to the game it is bound to, writes the game's frames out, until either side
/// closes. A message within the idle timeout keeps it open; more messages per second than the
/// limits (the connection's and its address's in `place`; WebSocket pings count), a frame that does
/// not decode, no hello within its timeout or no game within the lobby's timeout after it or after
/// leaving a game it was in (a refused join does not restart it), a full outbound queue (the game
/// drops the queue), a write that stalls for the idle timeout, no answer to its pings (SRV-Q20) or
/// the server's stop (`stop`) close it.
pub(crate) async fn serve<T: Transport>(
    mut transport: T,
    conn: ConnId,
    place: Place,
    registry: Shared,
    limits: Limits,
    server: Arc<Service>,
    mut stop: watch::Receiver<bool>,
) {
    // The login's challenge (E16 M3 T1): OS randomness, or no connection.
    let Ok(challenge) = game::challenge() else {
        transport.close().await;
        return;
    };
    let host = Host {
        program: &server.program,
        id: &server.id,
    };
    let idle = Duration::from_millis(limits.idle_timeout_ms);
    let lobby = Duration::from_millis(limits.lobby_timeout_ms);
    let mut deadline = Instant::now() + idle;
    // The pings' period and the silence that closes the connection (SRV-Q20), none without pings;
    // when the next ping goes, and when the client was last heard.
    let pings = (limits.ping_every_ms > 0).then(|| {
        let every = Duration::from_millis(limits.ping_every_ms);
        (every, Duration::from_millis(limits.ping_timeout_ms))
    });
    let mut next_ping = pings.map(|(every, _)| Instant::now() + every);
    let mut heard = Instant::now();
    // The deadline to be in a game: the hello's first, then the lobby's (E16 M2 T5b).
    let mut unjoined = Some(Instant::now() + Duration::from_millis(limits.hello_timeout_ms));
    let mut gate = Connection::new(limits.max_frame, challenge, server.anonymous);
    // Notified when another connection re-keys the account this one logs in as (E16 M3 T3).
    let rekeyed = Arc::new(Notify::new());
    let mut rate = Rate::new(limits.messages_per_second);
    // Where the connection hears from a ladder's queue it waits in (E16 M4 T2).
    let (to_me, mut from_queue) = mpsc::channel(QUEUE_FRAMES);
    let mut link = Link {
        conn,
        origin: place.address,
        registry,
        queue: (limits.outbound_queue.max(1), limits.outbound_bytes),
        bound: None,
        ladders: Arc::clone(&server.ladders),
        accounts: Arc::clone(&server.accounts),
        to_me,
        queued: false,
    };
    loop {
        // The lobby's deadline waits while the connection is bound to a game, and starts afresh
        // only once it leaves a game it was in: a refused join does not restart it (E16 M2 T5e).
        // It waits while the connection waits in a ladder's queue too, and starts afresh once it
        // no longer does (E16 M4 T2: a match's players have it to join their game).
        if link.bound.is_none() && unjoined.is_none() {
            unjoined = Some(Instant::now() + lobby);
        }
        let lobby_due = if link.bound.is_some() || link.queued {
            None
        } else {
            unjoined
        };
        let was_queued = link.queued;
        let wake = tokio::select! {
            incoming = transport.recv() => Wake::Client(incoming),
            frame = from_game(&mut link.bound) => Wake::Game(frame),
            () = sleep_until(deadline) => Wake::Idle,
            () = until(lobby_due) => Wake::Idle,
            () = until(next_ping) => Wake::Ping,
            () = until(pings.map(|(_, silence)| heard + silence)) => Wake::Silent,
            () = stopped(&mut stop) => Wake::Stop,
            () = rekeyed.notified() => Wake::Rekeyed,
            told = from_queue.recv() => Wake::Queue(told),
        };
        let now = Instant::now();
        if matches!(wake, Wake::Client(Some(_))) {
            heard = now;
        }
        let frame = match wake {
            Wake::Client(None) => break,
            Wake::Client(Some(_)) if !(rate.admit(now) && place.admit(now)) => {
                goodbye(GoodbyeReason::RateLimited)
            }
            // A re-key elsewhere closes the connection before it acts again (E16 M3's review).
            Wake::Client(Some(_)) if kicked(&rekeyed) => goodbye(GoodbyeReason::Rekeyed),
            Wake::Client(Some(Incoming::Control)) => continue,
            Wake::Client(Some(Incoming::Binary)) => goodbye(GoodbyeReason::Malformed),
            Wake::Client(Some(Incoming::Text(text))) => {
                deadline = now + idle;
                let greeted = gate.greeted();
                let received = gate.receive(text.as_bytes(), &host);
                if !greeted && gate.greeted() {
                    unjoined = Some(now + lobby);
                }
                match received {
                    Received::Game(message) => match link.route(message, gate.account()).await {
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
                    Received::Account(request) => {
                        let at = (&place, conn, &rekeyed);
                        account(&server, &mut gate, at, request).await
                    }
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
            Wake::Ping => {
                next_ping = pings.map(|(every, _)| now + every);
                let pinged = timeout(idle, transport.ping()).await;
                heard += now.elapsed();
                if !matches!(pinged, Ok(true)) {
                    break;
                }
                continue;
            }
            // Gone, most likely: its goodbye gets a moment, not the idle timeout.
            Wake::Silent => {
                let silent = goodbye(GoodbyeReason::IdleTimeout).text;
                let _ = timeout(LINGER, transport.send(silent)).await;
                break;
            }
            Wake::Stop => goodbye(GoodbyeReason::Shutdown),
            Wake::Rekeyed => goodbye(GoodbyeReason::Rekeyed),
            Wake::Queue(Some((text, waits))) => {
                link.queued = waits;
                Frame {
                    text,
                    close: false,
                    left: None,
                    held: false,
                }
            }
            // The task holds a sender of its own: never.
            Wake::Queue(None) => continue,
        };
        if was_queued && !link.queued {
            unjoined = Some(now + lobby);
        }
        let writing = Instant::now();
        let sent = timeout(idle, transport.send(frame.text)).await;
        heard += writing.elapsed();
        if frame.close || !matches!(sent, Ok(true)) {
            break;
        }
    }
    link.unbind().await;
    lock(&link.registry).closed(conn);
    if let Some(login) = gate.account() {
        server.sessions().remove(&(login.to_owned(), conn));
    }
    transport.close().await;
}

/// Answers an account's request (E16 M3 T2a) off the connection's task: the answer, or the
/// connection's goodbye when its address failed too many logins. A connection logged in is known
/// by its account with `rekeyed`, which another connection's re-key of the account notifies; its
/// own re-key notifies the others (E16 M3 T3).
async fn account(
    server: &Arc<Service>,
    gate: &mut Connection,
    (place, conn, rekeyed): (&Place, ConnId, &Arc<Notify>),
    request: AccountRequest,
) -> Frame {
    // The login name and key of a login; whether the request may write the player log.
    let (seq, login, changing) = match &request {
        AccountRequest::Login {
            seq,
            login,
            key,
            display,
            ..
        } => (*seq, Some((login.clone(), *key)), display.is_some()),
        AccountRequest::Rekey { seq, .. }
        | AccountRequest::Rename { seq, .. }
        | AccountRequest::SetListed { seq, .. } => (*seq, None, true),
    };
    let now = Instant::now();
    if (login.is_some() && place.barred(now)) || (changing && !place.changing(now)) {
        return goodbye(GoodbyeReason::RateLimited);
    }
    let rekey = match &request {
        AccountRequest::Rekey { key, .. } => Some(*key),
        _ => None,
    };
    // Known by its account before its key is checked: a re-key committed after the check reaches
    // it (one committed before refuses the key).
    let session = login.as_ref().map(|(login, _)| (login.clone(), conn));
    if let (Some(session), Some((_, key))) = (&session, &login) {
        let entry = (*key, Arc::clone(rekeyed));
        server.sessions().insert(session.clone(), entry);
    }
    let shared = Arc::clone(server);
    let answer = tokio::task::spawn_blocking(move || shared.accounts.answer(request))
        .await
        .unwrap_or(Err(Refusal::BadLogin));
    if let (Err(_), Some(session)) = (&answer, &session) {
        server.sessions().remove(session);
        // A re-key meanwhile signalled this attempt, not the account it may log in as later.
        kicked(rekeyed);
    }
    match answer {
        Ok((account, changed)) => {
            if let Some((login, _)) = login {
                gate.logged_in(login);
            } else if let (true, Some(login), Some(key)) = (changed, gate.account(), rekey) {
                server.rekeyed(login, conn, key);
            }
            Frame {
                text: codec::encode_text(&ServerMessage::Account {
                    seq,
                    display: account.display,
                    listed: account.listed,
                }),
                close: false,
                left: None,
                held: false,
            }
        }
        Err(Refusal::BadLogin) if login.is_some() && !place.failed(Instant::now()) => {
            goodbye(GoodbyeReason::RateLimited)
        }
        Err(reason) => refused(reason),
    }
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
    /// The ladders, for the account's rating (E16 M4 T2).
    ladders: Arc<Ladders>,
    /// The accounts, for the display names on a leaderboard (E16 M4 T4).
    accounts: Arc<Accounts>,
    /// Where the connection hears from a ladder's queue.
    to_me: ToConn,
    /// Whether the connection waits in a ladder's queue, matched or not.
    queued: bool,
}

impl Link {
    /// Answers a lobby message, or passes a game message to the game it names or is bound to. A
    /// connection is in at most one game: joining or creating another leaves the first, as a
    /// disconnect would (its seat waits for its token or its account). It opens a game logged in
    /// as `login` (E16 M3 T3).
    async fn route(&mut self, message: ClientMessage, login: Option<&str>) -> Routed {
        let login = login.map(str::to_owned);
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
            ClientMessage::ListLadders { seq } => {
                Routed::Reply(text(lock(&self.registry).ladders(seq)))
            }
            ClientMessage::Leaderboard {
                seq,
                ladder,
                season,
                page,
            } => Routed::Reply(
                self.leaderboard(seq, ladder, Asked::Page { season, page })
                    .await,
            ),
            // The gate let it through logged in only (`Connection::receive`).
            ClientMessage::Standing { seq, ladder } => {
                let own = Asked::Own(login.unwrap_or_default());
                Routed::Reply(self.leaderboard(seq, ladder, own).await)
            }
            ClientMessage::Queue { seq, ladder, bots } => {
                // Outside the registry's lock: the ladders have their own.
                let rating =
                    (login.as_deref()).map_or(0, |login| self.ladders.points(&ladder, login));
                let to = self.to_me.clone();
                // Left out: bots welcome (E16 M4 T5b, SRV-Q21).
                let asked = (seq, ladder.as_str(), bots.unwrap_or(true));
                let (answer, waits) =
                    lock(&self.registry).queue(self.conn, asked, login.as_deref(), (rating, to));
                self.queued = waits;
                Routed::Reply(text(answer))
            }
            ClientMessage::Dequeue { seq } | ClientMessage::DeclineMatch { seq } => {
                let decline = matches!(message, ClientMessage::DeclineMatch { .. });
                let (answer, waits) = lock(&self.registry).dequeue(self.conn, seq, decline);
                self.queued = waits;
                Routed::Reply(text(answer))
            }
            ClientMessage::AcceptMatch { .. } => {
                match registry::accept(&self.registry, self.conn) {
                    Some(refused) => Routed::Reply(text(refused)),
                    None => Routed::Passed,
                }
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
                let creator = (self.conn, to, login);
                match registry::create(&self.registry, self.origin, asked, creator) {
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
                    login,
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
        if let Some(Opening {
            conn,
            to,
            login,
            join,
        }) = opening
        {
            let opened = to_game.send(ToGame::Open { conn, to, login }).await.is_ok()
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

    /// The answer to `Leaderboard` or `Standing` on the ladder `ladder` (E16 M4 T4), read off the
    /// connection's task: the ladders' lock and the player log's wait on their syncs. A row names
    /// its account's display name as it stands.
    async fn leaderboard(&self, seq: u32, ladder: String, asked: Asked) -> Frame {
        let (tables, players) = (Arc::clone(&self.ladders), Arc::clone(&self.accounts));
        let answer = tokio::task::spawn_blocking(move || -> Result<ServerMessage, Refusal> {
            let board = match &asked {
                Asked::Page { season, page } => tables.page(&ladder, *season, *page),
                Asked::Own(login) => tables.own(&ladder, login),
            }?;
            let names = players.displays(board.rows.iter().map(|(_, login, _)| login.as_str()));
            let rows = (board.rows.into_iter().zip(names))
                .map(|((rank, _, row), (display, bot))| LeaderboardRow {
                    rank,
                    display,
                    rating: ladders::points(row.rating),
                    games: row.games,
                    wins: row.wins,
                    losses: row.losses,
                    draws: row.draws,
                    bot,
                })
                .collect::<Vec<_>>();
            let (season, total) = (board.season, board.total);
            Ok(match asked {
                Asked::Page { .. } => ServerMessage::LeaderboardPage {
                    seq,
                    ladder,
                    season,
                    total,
                    rows,
                },
                Asked::Own(_) => ServerMessage::OwnStanding {
                    seq,
                    ladder,
                    season,
                    total,
                    row: rows.into_iter().next(),
                },
            })
        });
        match answer.await.unwrap_or(Err(Refusal::NoSuchLadder)) {
            Ok(message) => text(codec::encode_text(&message)),
            Err(reason) => refused(reason),
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

/// What a leaderboard's request asks for (E16 M4 T4).
enum Asked {
    /// A page of the season's (the current one for `None`).
    Page { season: Option<u32>, page: u32 },
    /// The account's own row in the current season.
    Own(String),
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

/// A frame of `text` that keeps the connection open.
const fn text(text: String) -> Frame {
    Frame {
        text,
        close: false,
        left: None,
        held: false,
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
        | ClientMessage::FetchArchived { seq, .. }
        | ClientMessage::Login { seq, .. }
        | ClientMessage::Rekey { seq, .. }
        | ClientMessage::Rename { seq, .. }
        | ClientMessage::SetListed { seq, .. }
        | ClientMessage::ListLadders { seq }
        | ClientMessage::Queue { seq, .. }
        | ClientMessage::Dequeue { seq }
        | ClientMessage::AcceptMatch { seq }
        | ClientMessage::DeclineMatch { seq }
        | ClientMessage::Leaderboard { seq, .. }
        | ClientMessage::Standing { seq, .. }
        | ClientMessage::ProposePause { seq, .. }
        | ClientMessage::AnswerPause { seq, .. } => *seq,
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

/// The open connections, in total and per address; the addresses' failed logins (E16 M3 T2a),
/// kept beyond their connections for the window.
#[derive(Debug, Default)]
struct Counts {
    total: usize,
    by_address: BTreeMap<IpAddr, Address>,
    failures: BTreeMap<IpAddr, Rate>,
    /// The addresses' requests that may write the player log, kept beyond their connections for
    /// the window (E16 M3's review).
    changes: BTreeMap<IpAddr, Rate>,
}

/// The address a connection counts as in the limits per address (E16 M3 T4b): an IPv4 address
/// itself, also one mapped into IPv6; an IPv6 address its /64 prefix, the block one host usually
/// has, so that a host does not escape the limits by changing its interface identifier.
pub(crate) fn counted(address: IpAddr) -> IpAddr {
    match address.to_canonical() {
        IpAddr::V6(v6) => IpAddr::V6(Ipv6Addr::from(u128::from(v6) & !u128::from(u64::MAX))),
        v4 => v4,
    }
}

/// The client's address a trusted proxy names in the handshake (E16 M3 T4b): the last entry of
/// its last `X-Forwarded-For` header, the address the proxy itself saw; none if it does not read.
fn forwarded_for(request: &Request) -> Option<IpAddr> {
    let header = request
        .headers()
        .get_all("x-forwarded-for")
        .iter()
        .next_back()?;
    let last = header.to_str().ok()?.rsplit(',').next()?.trim();
    last.parse::<IpAddr>().ok()
}

/// What a handshake keeps of its request (E16 M3 T4b): the client a trusted proxy names.
struct Forwarded {
    /// Whether the peer is a trusted proxy: only then is the header read.
    proxied: bool,
    /// The client it names.
    client: Arc<Mutex<Option<IpAddr>>>,
}

impl Callback for Forwarded {
    fn on_request(self, request: &Request, response: Response) -> Result<Response, ErrorResponse> {
        if self.proxied {
            *self.client.lock().unwrap_or_else(PoisonError::into_inner) = forwarded_for(request);
        }
        Ok(response)
    }
}

/// A counted connection; dropping it releases its place.
#[derive(Debug)]
pub(crate) struct Place {
    counts: Arc<Mutex<Counts>>,
    address: IpAddr,
    /// Whether it counts for its address too, or in the total only (E16 M3 T4b: a trusted proxy's
    /// connection before its handshake names the client).
    per_address: bool,
    /// The address's failed logins a minute.
    failed_logins: u32,
    /// The address's requests that may write the player log, an hour.
    account_changes: u32,
}

impl Place {
    /// A place for a connection from `address` (as [`counted`]), if the limits leave one.
    fn take(counts: &Arc<Mutex<Counts>>, address: IpAddr, limits: &Limits) -> Option<Self> {
        let address = counted(address);
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
            per_address: true,
            failed_logins: limits.failed_logins_per_address,
            account_changes: limits.account_changes_per_address,
        })
    }

    /// A place in the total only for a connection from the trusted proxy `proxy` (E16 M3 T4b),
    /// until its handshake names the client, if the limit leaves one.
    fn proxied(counts: &Arc<Mutex<Counts>>, proxy: IpAddr, limits: &Limits) -> Option<Self> {
        let mut c = counts.lock().unwrap_or_else(PoisonError::into_inner);
        if c.total >= limits.connections {
            return None;
        }
        c.total += 1;
        Some(Self {
            counts: Arc::clone(counts),
            address: proxy,
            per_address: false,
            failed_logins: limits.failed_logins_per_address,
            account_changes: limits.account_changes_per_address,
        })
    }

    /// Whether the address failed its logins of the minute at `now` (E16 M3 T2a).
    fn barred(&self, now: Instant) -> bool {
        let c = self.counts.lock().unwrap_or_else(PoisonError::into_inner);
        c.failures.get(&self.address).is_some_and(|r| r.full(now))
    }

    /// Counts a failed login of the address at `now`; `false` beyond its limit (E16 M3 T2a). The
    /// addresses whose minute is over are forgotten.
    fn failed(&self, now: Instant) -> bool {
        let mut c = self.counts.lock().unwrap_or_else(PoisonError::into_inner);
        c.failures.retain(|_, rate| !rate.over(now));
        let limit = self.failed_logins;
        c.failures
            .entry(self.address)
            .or_insert_with(|| Rate::per(limit, FAILED_LOGINS_WINDOW))
            .admit(now)
    }

    /// Counts a request of the address that may write the player log at `now`; `false` beyond its
    /// limit (E16 M3's review). The addresses whose hour is over are forgotten.
    fn changing(&self, now: Instant) -> bool {
        let mut c = self.counts.lock().unwrap_or_else(PoisonError::into_inner);
        c.changes.retain(|_, rate| !rate.over(now));
        let limit = self.account_changes;
        c.changes
            .entry(self.address)
            .or_insert_with(|| Rate::per(limit, ACCOUNT_CHANGES_WINDOW))
            .admit(now)
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
        if !self.per_address {
            return;
        }
        if let Some(a) = c.by_address.get_mut(&self.address) {
            a.connections = a.connections.saturating_sub(1);
            if a.connections == 0 {
                c.by_address.remove(&self.address);
            }
        }
    }
}

/// Accepts connections until the server stops (`stop`, E16 M2 T5a): a connection beyond the
/// limits is closed at once, every other one gets its WebSocket handshake and its task. A trusted
/// proxy's connection is counted by its client's address after the handshake, and closed then if
/// that address has no place (E16 M3 T4b). Once stopped, it waits for the open connections to
/// close, for a grace.
pub(crate) async fn listen(
    listener: TcpListener,
    registry: Shared,
    limits: Limits,
    server: Arc<Service>,
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
        let peer = peer.ip().to_canonical();
        let proxied = server.proxies.contains(&peer);
        let place = if proxied {
            Place::proxied(&counts, peer, &limits)
        } else {
            Place::take(&counts, peer, &limits)
        };
        let Some(place) = place else {
            drop(stream);
            continue;
        };
        next += 1;
        let conn = ConnId(next);
        let (registry, server, stop) = (Arc::clone(&registry), Arc::clone(&server), stop.clone());
        let counts = Arc::clone(&counts);
        tokio::spawn(async move {
            let config = websocket_config(&limits);
            let client = Arc::new(Mutex::new(None));
            let read = Forwarded {
                proxied,
                client: Arc::clone(&client),
            };
            let handshake =
                tokio_tungstenite::accept_hdr_async_with_config(stream, read, Some(config));
            let idle = Duration::from_millis(limits.idle_timeout_ms);
            let Ok(Ok(mut socket)) = timeout(idle, handshake).await else {
                return;
            };
            let place = if proxied {
                let client = client.lock().unwrap_or_else(PoisonError::into_inner).take();
                drop(place);
                match Place::take(&counts, client.unwrap_or(peer), &limits) {
                    Some(place) => place,
                    None => return Transport::close(&mut socket).await,
                }
            } else {
                place
            };
            serve(socket, conn, place, registry, limits, server, stop).await;
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
    use crate::ids::GameIds;
    use crate::registry::Registry;

    #[test]
    fn an_address_counts_by_its_ipv6_prefix() {
        let ip = |text: &str| text.parse::<IpAddr>().unwrap();
        assert_eq!(counted(ip("203.0.113.9")), ip("203.0.113.9"));
        assert_eq!(counted(ip("::ffff:203.0.113.9")), ip("203.0.113.9"));
        assert_eq!(counted(ip("2001:db8:1:2:3:4:5:6")), ip("2001:db8:1:2::"));
    }

    #[test]
    fn a_proxys_header_names_the_client_by_its_last_entry() {
        let forwarded = |headers: &[&str]| {
            let mut request = Request::builder();
            for value in headers {
                request = request.header("X-Forwarded-For", *value);
            }
            forwarded_for(&request.body(()).unwrap())
        };
        let ip = |text: &str| Some(text.parse::<IpAddr>().unwrap());
        assert_eq!(forwarded(&["203.0.113.1"]), ip("203.0.113.1"));
        assert_eq!(forwarded(&["10.0.0.1, 203.0.113.1 "]), ip("203.0.113.1"));
        assert_eq!(forwarded(&["10.0.0.1", "2001:db8::1"]), ip("2001:db8::1"));
        assert_eq!(forwarded(&[]), None);
        assert_eq!(forwarded(&["203.0.113.1, unknown"]), None);
    }

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

        async fn ping(&mut self) -> bool {
            self.outbound.send(String::from(PING)).is_ok()
        }

        async fn close(&mut self) {}
    }

    /// What the fake transport sends for a ping.
    const PING: &str = "<ping>";

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
            Self::with(Limits::default())
        }

        /// A rig within `limits`, one game in total.
        fn with(limits: Limits) -> Self {
            let (client, inbound) = mpsc::unbounded_channel();
            let (outbound, sent) = mpsc::unbounded_channel();
            let (game, inbox) = mpsc::channel(8);
            let limits = Limits {
                games: 1,
                games_per_address: 1,
                ..limits
            };
            let accounts = Arc::new(
                Accounts::open(
                    &std::env::temp_dir().join(format!("h3-net-{:016x}", game::os_u64().unwrap())),
                    Box::new(crate::accounts::Ed25519),
                )
                .unwrap(),
            );
            let server = crate::config::Server::default();
            let ladders = Arc::new(Ladders::default());
            let stores = game::Stores {
                accounts: Arc::clone(&accounts),
                ladders: Arc::clone(&ladders),
                ids: Arc::new(GameIds::open(&server.data).unwrap()),
            };
            let mut registry = Registry::new(vec![], &limits, server, stores);
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
                Arc::new(Service {
                    program: String::from("s"),
                    id: String::from("ws://s"),
                    anonymous: true,
                    accounts,
                    sessions: Mutex::default(),
                    proxies: Vec::new(),
                    ladders,
                }),
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
            let Some(ToGame::Open { conn, to, login }) = self.told().await else {
                panic!("no open")
            };
            assert_eq!((conn, login), (ConnId(4), None));
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

    /// SRV-Q20: a client that answers the pings stays, one that falls silent is gone within the
    /// ping timeout, long before the idle timeout, and leaves its game.
    #[tokio::test]
    async fn a_connection_that_stops_answering_pings_closes() {
        let mut rig = Rig::with(Limits {
            ping_every_ms: 100,
            ping_timeout_ms: 300,
            ..Limits::default()
        });
        let _to = rig.joined(Seat::Attacker).await;
        let begun = Instant::now();
        while begun.elapsed() < Duration::from_millis(800) {
            assert_eq!(rig.next().await.as_deref(), Some(PING));
            rig.client.send(Incoming::Control).unwrap();
        }
        // Silent from here: pings, then the goodbye, and the game hears the connection close.
        let silent = Instant::now();
        let goodbye = goodbye(GoodbyeReason::IdleTimeout).text;
        let mut pings = 0;
        loop {
            match rig.next().await {
                Some(text) if text == PING => pings += 1,
                Some(text) => {
                    assert_eq!(text, goodbye);
                    break;
                }
                None => panic!("no goodbye"),
            }
        }
        assert!(pings >= 2, "{pings} pings");
        assert!(
            silent.elapsed() < Duration::from_secs(2),
            "{:?}",
            silent.elapsed()
        );
        assert!(matches!(
            rig.told().await,
            Some(ToGame::Closed { conn: ConnId(4) })
        ));
        rig.task.await.unwrap();
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
