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
//!   the restore at start;
//! - `net`: the listener and a `tokio` task per connection, behind the transport trait;
//! - [`client`]: a headless client for tests and tools;
//! - [`bench`]: the session core's workload (`benches/server/`).
//!
//! Provisional (E16 M1 review): the module layout, the thread and channel layout, the log lines
//! on standard error.

use std::fmt;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use tokio::net::TcpListener;

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
    /// The accepting task.
    pub listener: tokio::task::JoinHandle<()>,
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
    registry::restore(&registry).await?;
    let listener = TcpListener::bind(&config.server.bind)
        .await
        .map_err(|e| Error(format!("bind {}: {e}", config.server.bind)))?;
    let address = listener
        .local_addr()
        .map_err(|e| Error(format!("bind {}: {e}", config.server.bind)))?;
    let server: Arc<str> = Arc::from(server_name());
    let listener = tokio::spawn(net::listen(listener, registry, config.limits, server));
    Ok(Running { address, listener })
}
