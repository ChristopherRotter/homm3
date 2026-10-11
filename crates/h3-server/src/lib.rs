//! `h3-server`: hosts the engine's games for untrusted clients over WebSocket (E16, ADR-0018). The
//! I/O shell around `h3-protocol`'s sans-I/O core: sockets, clock, OS randomness, files,
//! configuration and the bot seats. M2: the games clients create from the server's offers.
//!
//! - [`config`]: the TOML configuration (server, content, limits, the offers);
//! - [`game`]: the offers, compiled at start; a game prepared from an offer with secrets from the
//!   OS, or restored from its journal, run on its own OS thread; the replay written as the CLI's
//!   file at the end;
//! - `journal`: each game's write-ahead journal in the data directory (M2 T2);
//! - `registry`: the lobby and the games' inboxes, shared by the tasks and the games' threads;
//!   the restore at start; the sweep of abandoned games;
//! - `net`: the listener and a `tokio` task per connection, behind the transport trait;
//! - [`client`]: a headless client for tests and tools;
//! - [`bench`]: the session core's workload (`benches/server/`).
//!
//! Provisional (E16 M1 review): the module layout, the thread and channel layout, the log lines
//! on standard error.

use std::fmt;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio::task::JoinHandle;

mod archive;
pub mod bench;
pub mod client;
pub mod config;
pub mod game;
mod journal;
mod net;
mod registry;

#[cfg(test)]
mod tests;

pub use config::{BotSeat, Config, Limits};
pub use journal::JOURNAL_FORMAT_VERSION;
use registry::Registry;

/// Why the server does not start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(pub String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

/// A running server.
#[derive(Debug)]
pub struct Running {
    /// The address it listens on.
    pub address: SocketAddr,
    /// The accepting task: it ends once the server stopped and its connections closed.
    pub listener: JoinHandle<()>,
    /// The lobby and the games.
    registry: registry::Shared,
    /// Set once the server stops (E16 M2 T5a).
    stop: watch::Sender<bool>,
    /// The sweep of abandoned games (E16 M2 T5a).
    sweeper: JoinHandle<()>,
}

/// How often a draining or stopping server looks at its games.
const POLL: Duration = Duration::from_millis(20);

/// How long a stopping server waits for its games' threads to end.
const STOP_GRACE: Duration = Duration::from_secs(3);

impl Running {
    /// Drains the server (E16 M2 T5a; `E16-design.md` §7): no game is created any more
    /// (`Refused { Draining }`), a game that waits for its players ends (`Goodbye { Shutdown }`),
    /// the running ones play on and paused ones may resume; [`drained`](Self::drained) resolves
    /// once none waits or runs.
    pub fn drain(&self) {
        registry::lock(&self.registry).drain();
    }

    /// How many games wait or run: a paused game and one that is over do not count.
    pub fn in_play(&self) -> usize {
        registry::lock(&self.registry).in_play()
    }

    /// Resolves once no game waits or runs.
    pub async fn drained(&self) {
        while self.in_play() > 0 {
            tokio::time::sleep(POLL).await;
        }
    }

    /// Stops the server (E16 M2 T5a): it accepts no connection any more, every game's thread ends
    /// and every connection hears `Goodbye { Shutdown }`. The journals stay: the running and the
    /// paused games restore at the next start. Resolves once the games' threads ended and the
    /// connections closed, each within a grace.
    pub async fn shutdown(self) {
        self.stop.send_replace(true);
        self.sweeper.abort();
        registry::lock(&self.registry).stop_all();
        let grace = tokio::time::Instant::now() + STOP_GRACE;
        while registry::lock(&self.registry).live() > 0 && tokio::time::Instant::now() < grace {
            tokio::time::sleep(POLL).await;
        }
        let _ = self.listener.await;
    }
}

/// The server program's name in the welcome.
pub fn server_name() -> String {
    format!("h3-server {}", env!("CARGO_PKG_VERSION"))
}

/// Compiles the offers, restores the games the data directory's journals hold, and listens: call
/// from a `tokio` runtime. Games are created by clients (`CreateGame`), each on its own thread.
pub async fn start(config: &Config) -> Result<Running, Error> {
    let offers = config
        .offers
        .iter()
        .map(|offer| game::Offer::load(offer, &config.content.packs).map(Arc::new))
        .collect::<Result<Vec<_>, _>>()?;
    let registry = Arc::new(Mutex::new(Registry::new(
        offers,
        &config.limits,
        config.server.clone(),
    )));
    registry::lock(&registry).load_archive();
    registry::restore(&registry).await?;
    let listener = TcpListener::bind(&config.server.bind)
        .await
        .map_err(|e| Error(format!("bind {}: {e}", config.server.bind)))?;
    let address = listener
        .local_addr()
        .map_err(|e| Error(format!("bind {}: {e}", config.server.bind)))?;
    let server: Arc<str> = Arc::from(server_name());
    let (stop, stopping) = watch::channel(false);
    let sweeper = tokio::spawn(registry::sweep_every(Arc::clone(&registry), config.limits));
    let shared = Arc::clone(&registry);
    let listener = tokio::spawn(net::listen(
        listener,
        shared,
        config.limits,
        server,
        stopping,
    ));
    Ok(Running {
        address,
        listener,
        registry,
        stop,
        sweeper,
    })
}
