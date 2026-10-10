//! The server's configuration file (TOML; ADR-0018 points 2 and 9). Relative paths are taken from
//! the file's directory.
//!
//! ```toml
//! bind = "127.0.0.1:7878"            # loopback by default; the internet only behind a TLS proxy
//! packs = "../packs"                 # the packs directory
//! scenario = "../packs/synthetic/scenarios/full.toml"
//! profile = "synthetic:faithful"     # optional: overrides the scenario's profile
//! bots = ["defender"]                # the seats the server's bot plays (none by default)
//! data = "data"                      # the replays go to <data>/replays/
//! checkpoint_every = 25              # a state hash in the journal every this many commands
//!
//! [limits]                           # every field optional, defaults below
//! max_frame = 4096                   # bytes per client message
//! messages_per_second = 20           # per connection
//! connections = 64                   # in total
//! connections_per_address = 8
//! idle_timeout_ms = 300000           # no message within it closes the connection
//! outbound_queue = 256               # frames waiting for a slow client; full drops it
//! ```
//!
//! Provisional (E16 M1 review): the format and the defaults.

use std::fs;
use std::path::{Path, PathBuf};

use h3_protocol::codec::DEFAULT_MAX_CLIENT_FRAME;
use serde::Deserialize;

use crate::Error;

/// A seat the server's bot plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BotSeat {
    /// The attacker's seat.
    Attacker,
    /// The defender's seat.
    Defender,
}

/// The server's configuration.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The address to listen on.
    #[serde(default = "default_bind")]
    pub bind: String,
    /// The packs directory.
    pub packs: PathBuf,
    /// The scenario file of the hosted battle (as `h3 run` takes it).
    pub scenario: PathBuf,
    /// A profile instead of the scenario's.
    #[serde(default)]
    pub profile: Option<String>,
    /// The seats the server's bot plays.
    #[serde(default)]
    pub bots: Vec<BotSeat>,
    /// The data directory: replays go to `<data>/replays/`.
    #[serde(default = "default_data")]
    pub data: PathBuf,
    /// A state hash in the journal after every this many commands (0: none).
    #[serde(default = "default_checkpoint_every")]
    pub checkpoint_every: u32,
    /// The bounds (ADR-0018 point 9).
    #[serde(default)]
    pub limits: Limits,
}

/// The bounds of the server (ADR-0018 point 9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Limits {
    /// The largest client message, in bytes.
    pub max_frame: usize,
    /// Messages per second per connection; more closes it.
    pub messages_per_second: u32,
    /// Connections in total.
    pub connections: usize,
    /// Connections per remote address.
    pub connections_per_address: usize,
    /// No message within this many milliseconds closes a connection.
    pub idle_timeout_ms: u64,
    /// Frames waiting for a connection; a full queue drops it.
    pub outbound_queue: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_frame: DEFAULT_MAX_CLIENT_FRAME,
            messages_per_second: 20,
            connections: 64,
            connections_per_address: 8,
            idle_timeout_ms: 300_000,
            outbound_queue: 256,
        }
    }
}

fn default_bind() -> String {
    String::from("127.0.0.1:7878")
}

fn default_data() -> PathBuf {
    PathBuf::from("data")
}

const fn default_checkpoint_every() -> u32 {
    25
}

impl Config {
    /// Parses a configuration; relative paths are taken from `base`.
    pub fn parse(text: &str, base: &Path) -> Result<Self, Error> {
        let mut config: Self = toml::from_str(text).map_err(|e| Error(e.to_string()))?;
        for path in [&mut config.packs, &mut config.scenario, &mut config.data] {
            *path = base.join(&*path);
        }
        Ok(config)
    }

    /// Reads and parses a configuration file.
    pub fn load(path: &Path) -> Result<Self, Error> {
        let text =
            fs::read_to_string(path).map_err(|e| Error(format!("{}: {e}", path.display())))?;
        let base = path.parent().unwrap_or(Path::new("."));
        Self::parse(&text, base).map_err(|e| Error(format!("{}: {e}", path.display())))
    }
}
