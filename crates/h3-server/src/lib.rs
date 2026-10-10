//! `h3-server`: hosts the engine's games for untrusted clients over WebSocket (E16, ADR-0018). The
//! I/O shell around `h3-protocol`'s sans-I/O core: sockets, clock, OS randomness, files,
//! configuration and the bot seats. M1: one hosted battle per server.
//!
//! - [`config`]: the TOML configuration (bind address, packs, scenario, profile, bots, limits);
//! - [`game`]: the game, prepared from the configuration with secrets from the OS, run on its own
//!   OS thread; the replay written as the CLI's file at the end;
//! - `net`: the listener and a `tokio` task per connection, behind the transport trait;
//! - [`client`]: a headless client for tests and tools;
//! - [`bench`]: the session core's workload (`benches/server/`).
//!
//! Provisional (E16 M1 review): the module layout, the thread and channel layout, the log lines
//! on standard error.

use std::fmt;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::mpsc;

pub mod bench;
pub mod client;
pub mod config;
pub mod game;
mod net;

#[cfg(test)]
mod tests;

pub use config::{BotSeat, Config, Limits};

/// The game's inbox: messages waiting for the game's thread.
const GAME_INBOX: usize = 1024;

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
    /// Where the replay goes at the end of the game.
    pub replay_path: PathBuf,
    /// The players' seat tokens (attacker, defender; `None` for the bot's). The server hands a
    /// token to the first connection that takes the seat; an operator may give it out instead.
    pub tokens: [Option<String>; 2],
    /// The accepting task.
    pub listener: tokio::task::JoinHandle<()>,
}

/// The server program's name in the welcome.
pub fn server_name() -> String {
    format!("h3-server {}", env!("CARGO_PKG_VERSION"))
}

/// Prepares the game, starts its thread and listens: call from a `tokio` runtime.
pub async fn start(config: &Config) -> Result<Running, Error> {
    let prepared = game::Prepared::new(config)?;
    let replay_path = prepared.replay_path.clone();
    let tokens = prepared.tokens.clone();
    let listener = TcpListener::bind(&config.bind)
        .await
        .map_err(|e| Error(format!("bind {}: {e}", config.bind)))?;
    let address = listener
        .local_addr()
        .map_err(|e| Error(format!("bind {}: {e}", config.bind)))?;
    let (to_game, inbox) = mpsc::channel(GAME_INBOX);
    prepared.spawn(inbox)?;
    let server: Arc<str> = Arc::from(server_name());
    let listener = tokio::spawn(net::listen(listener, to_game, config.limits, server));
    Ok(Running {
        address,
        replay_path,
        tokens,
        listener,
    })
}
