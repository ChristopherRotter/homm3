//! The server's configuration file (TOML; ADR-0018 points 2 and 9; E16 M2 T1's sections). Relative
//! paths are taken from the file's directory.
//!
//! ```toml
//! [server]                           # every field optional, defaults below
//! bind = "127.0.0.1:7878"            # loopback by default; the internet only behind a TLS proxy
//! data = "data"                      # journals, replays (below); the server's user only
//! checkpoint_every = 25              # a state hash in the journal every this many commands
//!
//! [content]
//! packs = "../packs"                 # the packs directory
//!
//! [limits]                           # every field optional, defaults below
//! max_frame = 4096                   # bytes per client message
//! messages_per_second = 20           # per connection
//! messages_per_address = 40          # per second, across one address's connections
//! connections = 64                   # in total
//! connections_per_address = 8
//! idle_timeout_ms = 300000           # no message within it closes the connection
//! hello_timeout_ms = 10000           # no hello within it closes the connection
//! lobby_timeout_ms = 60000           # in no game for this long closes the connection
//! outbound_queue = 256               # frames waiting for a slow client; full drops it
//! outbound_bytes = 2097152           # bytes waiting for a slow client; full drops it
//! record_bytes = 524288              # a viewer's record: its latest messages within this
//! games = 16                         # games not over, in total
//! games_per_address = 2              # games not over, created from one address
//!
//! [[offer]]                          # what a client may create a game from; at least one
//! id = "duel"                        # as `CreateGame` names it
//! scenario = "../packs/synthetic/scenarios/duel.toml"
//! profile = "synthetic:faithful"     # optional: overrides the scenario's profile
//! bots = ["defender"]                # the seats the server's bot plays (none by default)
//! ```
//!
//! The data directory holds `games/<game>.journal` for every game that is not over (its seed and
//! its seats' tokens: the operator's copy of them; E16 M2 T2), `replays/` for the finished games
//! and `void/` for the journals a start could not restore. It must be readable by the server's
//! user only: on Unix the server creates `games/` and `void/` owner-only and the journals
//! owner-read-write.
//!
//! Provisional (E16 M1 and M2 reviews): the format and the defaults.

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
    /// Where it listens and keeps its files.
    #[serde(default)]
    pub server: Server,
    /// Where the content comes from.
    pub content: Content,
    /// The bounds (ADR-0018 point 9).
    #[serde(default)]
    pub limits: Limits,
    /// What a client may create a game from.
    #[serde(rename = "offer")]
    pub offers: Vec<Offer>,
}

/// The `[server]` section.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Server {
    /// The address to listen on.
    pub bind: String,
    /// The data directory: the games' journals, the replays, the void journals.
    pub data: PathBuf,
    /// A state hash in the journal after every this many commands (0: none).
    pub checkpoint_every: u32,
    /// Whether every game runs a shadow instance that checks its battle after every command
    /// (E16 M2 T3, `E16-design.md` §8).
    pub shadow_instance: bool,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            bind: String::from("127.0.0.1:7878"),
            data: PathBuf::from("data"),
            checkpoint_every: 25,
            shadow_instance: false,
        }
    }
}

/// The `[content]` section.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Content {
    /// The packs directory.
    pub packs: PathBuf,
}

/// One `[[offer]]`: a game configuration a client may create a game from.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Offer {
    /// The offer's id, as `CreateGame` names it.
    pub id: String,
    /// The scenario file (as `h3 run` takes it).
    pub scenario: PathBuf,
    /// A profile instead of the scenario's.
    #[serde(default)]
    pub profile: Option<String>,
    /// The seats the server's bot plays.
    #[serde(default)]
    pub bots: Vec<BotSeat>,
}

/// A year in milliseconds: the longest deadline of an abandoned game.
const YEAR_MS: u64 = 365 * 24 * 3_600_000;

/// The bounds of the server (ADR-0018 point 9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Limits {
    /// The largest client message, in bytes.
    pub max_frame: usize,
    /// Messages per second per connection; more closes it. WebSocket pings count too.
    pub messages_per_second: u32,
    /// Messages per second across the connections of one address (E16 M2 T5b); the message
    /// beyond it closes its connection.
    pub messages_per_address: u32,
    /// Connections in total.
    pub connections: usize,
    /// Connections per remote address.
    pub connections_per_address: usize,
    /// No message within this many milliseconds closes a connection.
    pub idle_timeout_ms: u64,
    /// No hello within this many milliseconds of the handshake closes a connection (E16 M2 T5b).
    pub hello_timeout_ms: u64,
    /// A connection in no game for this many milliseconds after its hello, or after it left one
    /// (a refused join), is closed (E16 M2 T5b).
    pub lobby_timeout_ms: u64,
    /// Frames waiting for a connection; a full queue drops it.
    pub outbound_queue: usize,
    /// Bytes waiting for a connection (E16 M2 T5c); a queue that would hold more drops it.
    pub outbound_bytes: usize,
    /// The most a viewer's record holds, in bytes (E16 M2 T5c): its oldest messages go first.
    pub record_bytes: usize,
    /// Games not over, in total.
    pub games: usize,
    /// Games not over, created from one address.
    pub games_per_address: usize,
    /// Spectators watching one game at once (E16 M2 T4; 0: none).
    pub spectators_per_game: usize,
    /// A running game none of whose seats is held for this many milliseconds pauses (E16 M2 T5a;
    /// 0: never).
    pub empty_game_ms: u64,
    /// A paused game nobody resumes within this many milliseconds is void (E16 M2 T5a; 0: never).
    pub paused_game_ms: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_frame: DEFAULT_MAX_CLIENT_FRAME,
            messages_per_second: 20,
            messages_per_address: 40,
            connections: 64,
            connections_per_address: 8,
            idle_timeout_ms: 300_000,
            hello_timeout_ms: 10_000,
            lobby_timeout_ms: 60_000,
            outbound_queue: 256,
            outbound_bytes: 2 << 20,
            record_bytes: 512 << 10,
            games: 16,
            games_per_address: 2,
            spectators_per_game: 8,
            empty_game_ms: 1_800_000,
            paused_game_ms: 259_200_000,
        }
    }
}

impl Limits {
    /// Checks every bound against its range: none is 0 but the spectators and the deadlines of
    /// abandoned games (0: none), a frame holds a hello, and the timeouts, the deadlines and the
    /// queue stay where the clock's and the channel's arithmetic hold; a game is a thread; a
    /// fetched record fits in the outbound queue.
    fn check(&self) -> Result<(), Error> {
        let wide = |n: usize| u64::try_from(n).unwrap_or(u64::MAX);
        let bounds = [
            ("max_frame", wide(self.max_frame), 256, 1 << 20),
            (
                "messages_per_second",
                u64::from(self.messages_per_second),
                1,
                u64::MAX,
            ),
            (
                "messages_per_address",
                u64::from(self.messages_per_address),
                1,
                u64::MAX,
            ),
            ("connections", wide(self.connections), 1, u64::MAX),
            (
                "connections_per_address",
                wide(self.connections_per_address),
                1,
                u64::MAX,
            ),
            ("idle_timeout_ms", self.idle_timeout_ms, 100, 86_400_000),
            ("hello_timeout_ms", self.hello_timeout_ms, 100, 86_400_000),
            ("lobby_timeout_ms", self.lobby_timeout_ms, 100, 86_400_000),
            ("outbound_queue", wide(self.outbound_queue), 1, 1 << 20),
            (
                "outbound_bytes",
                wide(self.outbound_bytes),
                64 << 10,
                1 << 30,
            ),
            ("record_bytes", wide(self.record_bytes), 4 << 10, 256 << 20),
            ("games", wide(self.games), 1, 4096),
            ("games_per_address", wide(self.games_per_address), 1, 4096),
            (
                "spectators_per_game",
                wide(self.spectators_per_game),
                0,
                1024,
            ),
            ("empty_game_ms", self.empty_game_ms, 0, YEAR_MS),
            ("paused_game_ms", self.paused_game_ms, 0, YEAR_MS),
        ];
        for (name, value, min, max) in bounds {
            if !(min..=max).contains(&value) {
                return Err(Error(format!(
                    "limits.{name} = {value}: outside {min}..={max}"
                )));
            }
        }
        // A fetched record's frame escapes its text once more, at most doubling it.
        if self.record_bytes.saturating_mul(2) >= self.outbound_bytes {
            return Err(Error(format!(
                "limits.record_bytes = {}: a record must fit twice in outbound_bytes = {}",
                self.record_bytes, self.outbound_bytes
            )));
        }
        Ok(())
    }
}

impl Config {
    /// Parses a configuration; relative paths are taken from `base`.
    pub fn parse(text: &str, base: &Path) -> Result<Self, Error> {
        let mut config: Self = toml::from_str(text).map_err(|e| Error(e.to_string()))?;
        config.limits.check()?;
        config.check_offers()?;
        config.server.data = base.join(&config.server.data);
        config.content.packs = base.join(&config.content.packs);
        for offer in &mut config.offers {
            offer.scenario = base.join(&offer.scenario);
        }
        Ok(config)
    }

    /// At least one offer; ids present and distinct; a player's seat in each.
    fn check_offers(&self) -> Result<(), Error> {
        if self.offers.is_empty() {
            return Err(Error(String::from("no [[offer]]: a server offers a game")));
        }
        for (i, offer) in self.offers.iter().enumerate() {
            if offer.id.is_empty() || self.offers[..i].iter().any(|o| o.id == offer.id) {
                return Err(Error(format!("offer {i}: its id is empty or taken")));
            }
            if offer.bots.contains(&BotSeat::Attacker) && offer.bots.contains(&BotSeat::Defender) {
                return Err(Error(format!(
                    "offer `{}`: bots play both seats: a hosted game needs a player",
                    offer.id
                )));
            }
        }
        Ok(())
    }

    /// Reads and parses a configuration file.
    pub fn load(path: &Path) -> Result<Self, Error> {
        let text =
            fs::read_to_string(path).map_err(|e| Error(format!("{}: {e}", path.display())))?;
        let base = path.parent().unwrap_or(Path::new("."));
        Self::parse(&text, base).map_err(|e| Error(format!("{}: {e}", path.display())))
    }
}
