//! The server's configuration file (TOML; ADR-0018 points 2 and 9; E16 M2 T1's sections). Relative
//! paths are taken from the file's directory.
//!
//! ```toml
//! [server]                           # every field optional, defaults below
//! bind = "127.0.0.1:7878"            # loopback by default; the internet only behind a TLS proxy
//! data = "data"                      # journals, replays (below); the server's user only
//! checkpoint_every = 25              # a state hash in the journal every this many commands
//! public_url = "wss://example.org"   # the server's id, which logins sign: what clients dial
//! anonymous = true                   # whether a game may be joined or created without a login
//! trusted_proxies = ["127.0.0.1"]    # reverse proxies whose X-Forwarded-For names the client
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
//! ping_every_ms = 15000              # a WebSocket ping to each connection (0: none)
//! ping_timeout_ms = 30000            # nothing heard within it, not even a pong, closes it
//! hello_timeout_ms = 10000           # no hello within it closes the connection
//! lobby_timeout_ms = 60000           # in no game for this long closes the connection
//! outbound_queue = 256               # frames waiting for a slow client; full drops it
//! outbound_bytes = 2097152           # bytes waiting for a slow client; full drops it
//! record_bytes = 524288              # a viewer's record: its latest messages within this
//! replays_kept = 10000               # finished games' replays kept, listed and closed each
//! games = 16                         # games not over, in total
//! games_per_address = 2              # games not over, created from one address
//! failed_logins_per_address = 10     # a minute; beyond, the address's logins close
//! account_changes_per_address = 20   # an hour: registrations, re-keys, renames, preferences
//! commands_per_game = 10000          # a game that reaches it without an end is void
//!
//! [[offer]]                          # what a client may create a game from; at least one
//! id = "duel"                        # as `CreateGame` names it
//! scenario = "../packs/synthetic/scenarios/duel.toml"
//! profile = "synthetic:faithful"     # optional: overrides the scenario's profile
//! bots = ["defender"]                # the seats the server's bot plays (none by default)
//!
//! [[ladder]]                         # a ranked game configuration (E16 M4); none by default
//! id = "arena"                       # a-z 0-9 _ -, at most 32: its results log's file name
//! name = "Arena"                     # shown to clients, as a display name
//! scenarios = ["../packs/synthetic/scenarios/duel.toml"]   # the pool, on one ruleset
//! profile = "synthetic:faithful"     # optional: overrides the scenarios' profile
//! accept_ms = 30000                  # both players accept a match within it
//! grace_ms = 300000                  # a player away longer at once forfeits
//! grace_total_ms = 600000            # a player away longer in a game forfeits
//! pauses = 0                         # pauses each player may propose in a game
//! pause_ms = 0                       # the longest pause
//! spectators = true                  # whether its games take spectators
//! bot_after_ms = 60000               # a player waiting this long without a match plays a bot
//!
//! [[ladder.bot]]                     # one of the server's bots on the ladder (E16 M4 T5)
//! login = "randy"                    # its account's login name, never logged in
//! display = "Randy"                  # its display name
//! kind = "random"                    # which bot plays (the only one so far)
//! ```
//!
//! The data directory holds `games/<game>.journal` for every game that is not over (its seed and
//! its seats' tokens: the operator's copy of them; E16 M2 T2), `replays/` for the finished games
//! and `void/` for the journals a start could not restore, `players.log`, the accounts (E16 M3
//! T2a), `ladders/<id>.log`, each ladder's results (E16 M4 T1), and `lock`, which a running server
//! holds (E16 M3 T2b: one server, or the operator's `rebind`, at a time). It must be readable by the server's user only: on Unix the server creates
//! the directories owner-only and its files owner-read-write.
//!
//! Provisional (E16 M1 and M2 reviews): the format and the defaults.

use std::fs;
use std::net::IpAddr;
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

/// Which of the server's bots plays a bot account's seats (E16 M4 T5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BotKind {
    /// The sim's random agent.
    #[default]
    Random,
}

/// One `[[ladder.bot]]` (E16 M4 T5): an account of the server's, which never logs in, whose seats
/// its bot plays. It waits in the ladder's queue at all times, for a player who waited the
/// ladder's `bot_after_ms` without a match.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LadderBot {
    /// The account's login name.
    pub login: String,
    /// Its display name, unique as any account's.
    pub display: String,
    /// Which bot plays.
    #[serde(default)]
    pub kind: BotKind,
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
    /// The ranked game configurations (E16 M4 T1; ADR-0019 point 4).
    #[serde(rename = "ladder", default)]
    pub ladders: Vec<Ladder>,
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
    /// The server's id (E16 M3 T2a; ADR-0019 point 1): the address its clients dial (`ws://` or
    /// `wss://`), which a login signs; without one, `ws://` and the address it listens on.
    pub public_url: Option<String>,
    /// Whether a connection may join or create a game without logging in (E16 M3 T2a), with seat
    /// tokens as in M2.
    pub anonymous: bool,
    /// The reverse proxies in front of the server (E16 M3 T4b): a connection from one of them
    /// counts as the client its handshake's `X-Forwarded-For` names. None by default.
    pub trusted_proxies: Vec<IpAddr>,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            bind: String::from("127.0.0.1:7878"),
            data: PathBuf::from("data"),
            checkpoint_every: 25,
            shadow_instance: false,
            public_url: None,
            anonymous: true,
            trusted_proxies: Vec::new(),
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

/// One `[[ladder]]` (E16 M4 T1; ADR-0019 points 4–6; `E16-design.md` §11): a ranked game
/// configuration, its scenario pool on one ruleset and the bounds of its games.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Ladder {
    /// The ladder's id: its results log's file name.
    pub id: String,
    /// What clients see of it: a display name's characters.
    pub name: String,
    /// The scenario pool (files as `h3 run` takes them), on one ruleset.
    pub scenarios: Vec<PathBuf>,
    /// A profile instead of the scenarios'.
    pub profile: Option<String>,
    /// Both players accept a match within it (E16 M4 T2).
    pub accept_ms: u64,
    /// A player away longer than this at once forfeits (E16 M4 T3a; the user, SRV-Q19).
    pub grace_ms: u64,
    /// A player away longer than this in all in one game forfeits (E16 M4 T3a).
    pub grace_total_ms: u64,
    /// The pauses each player may propose in a game (E16 M4 T6; 0: none).
    pub pauses: u32,
    /// The longest pause (E16 M4 T6).
    pub pause_ms: u64,
    /// Whether its games take spectators (with `[limits] spectators_per_game`).
    pub spectators: bool,
    /// Its bots (E16 M4 T5; none by default).
    #[serde(rename = "bot")]
    pub bots: Vec<LadderBot>,
    /// How long a player waits without a match before it plays a bot (E16 M4 T5).
    pub bot_after_ms: u64,
}

impl Default for Ladder {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            scenarios: Vec::new(),
            profile: None,
            accept_ms: 30_000,
            grace_ms: 300_000,
            grace_total_ms: 600_000,
            pauses: 0,
            pause_ms: 0,
            spectators: true,
            bots: Vec::new(),
            bot_after_ms: 60_000,
        }
    }
}

impl Ladder {
    /// Its id names a file (`a–z 0–9 _ -`, 1 to 32), its name is a display name, it has a pool,
    /// and its bounds hold.
    fn check(&self) -> Result<(), Error> {
        let fail = |why: String| Err(Error(format!("ladder `{}`: {why}", self.id)));
        let id = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-';
        if !(1..=32).contains(&self.id.len()) || !self.id.bytes().all(id) {
            return fail(String::from("its id is not 1 to 32 of a-z 0-9 _ -"));
        }
        if !h3_protocol::account::is_display_name(&self.name) {
            return fail(format!("name = {:?}: not a display name", self.name));
        }
        if self.scenarios.is_empty() {
            return fail(String::from("no scenarios"));
        }
        let bounds = [
            ("accept_ms", self.accept_ms, 1_000, 3_600_000),
            ("grace_ms", self.grace_ms, 1_000, YEAR_MS),
            ("grace_total_ms", self.grace_total_ms, 1_000, YEAR_MS),
            ("pauses", u64::from(self.pauses), 0, 100),
            ("pause_ms", self.pause_ms, 0, YEAR_MS),
            ("bot_after_ms", self.bot_after_ms, 0, YEAR_MS),
        ];
        for (name, value, min, max) in bounds {
            if !(min..=max).contains(&value) {
                return fail(format!("{name} = {value}: outside {min}..={max}"));
            }
        }
        if self.pauses > 0 && self.pause_ms == 0 {
            return fail(String::from("pauses without pause_ms"));
        }
        // Its bots' names follow an account's rules, each bot once (E16 M4 T5).
        for (i, bot) in self.bots.iter().enumerate() {
            if !h3_protocol::account::is_login_name(&bot.login)
                || !h3_protocol::account::is_display_name(&bot.display)
            {
                return fail(format!("bot `{}`: not a name", bot.login));
            }
            if self.bots[..i].iter().any(|b| b.login == bot.login) {
                return fail(format!("bot `{}`: listed twice", bot.login));
            }
        }
        Ok(())
    }
}

/// A year in milliseconds: the longest deadline of an abandoned game.
const YEAR_MS: u64 = 365 * 24 * 3_600_000;

/// The most bytes one command adds to a replay's frame (E16 M3 T4a): its line in the replay's
/// text (at most 48 bytes: `"A cast 65535 stack 65535 stack 65535",` indented) and a checkpoint's
/// (at most 32), each doubled for the escapes of the frame's JSON.
const REPLAY_BYTES_PER_COMMAND: usize = 160;

/// The most bytes of a replay's frame besides its commands: the message, the header, the setup.
const REPLAY_HEAD_BYTES: usize = 64 << 10;

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
    /// A WebSocket ping to each connection every this many milliseconds (SRV-Q20; 0: none, and
    /// no `ping_timeout_ms` either).
    pub ping_every_ms: u64,
    /// A connection the server heard nothing from, not even a pong, for this many milliseconds
    /// is closed (SRV-Q20): a client gone without a close is noticed within it, not after the idle
    /// timeout. Longer than `ping_every_ms`.
    pub ping_timeout_ms: u64,
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
    /// Finished games' replays kept in the archive (E16 M2 T5d), the listed and the closed ones
    /// each: beyond, the oldest go.
    pub replays_kept: usize,
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
    /// Failed logins a minute from one address (E16 M3 T2a): a login beyond them, and any login
    /// from the address until the minute is over, closes its connection.
    pub failed_logins_per_address: u32,
    /// Requests from one address an hour that may write the player log (E16 M3's review): a
    /// registration, a re-key, a rename, a listing preference; the one beyond closes its
    /// connection.
    pub account_changes_per_address: u32,
    /// Commands one game may hold (E16 M3 T4a): a game that reaches them without an end is void.
    /// It bounds a game's journal, its memory and its replay, which must fit `outbound_bytes`.
    pub commands_per_game: u32,
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
            ping_every_ms: 15_000,
            ping_timeout_ms: 30_000,
            hello_timeout_ms: 10_000,
            lobby_timeout_ms: 60_000,
            outbound_queue: 256,
            outbound_bytes: 2 << 20,
            record_bytes: 512 << 10,
            replays_kept: 10_000,
            games: 16,
            games_per_address: 2,
            spectators_per_game: 8,
            empty_game_ms: 1_800_000,
            paused_game_ms: 259_200_000,
            failed_logins_per_address: 10,
            account_changes_per_address: 20,
            commands_per_game: 10_000,
        }
    }
}

impl Limits {
    /// Checks every bound against its range: none is 0 but the spectators and the deadlines of
    /// abandoned games (0: none), a frame holds a hello, and the timeouts, the deadlines and the
    /// queue stay where the clock's and the channel's arithmetic hold; a game is a thread; a
    /// fetched record, and the replay of a game of `commands_per_game` commands, fit in the
    /// outbound queue.
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
            ("ping_every_ms", self.ping_every_ms, 0, 86_400_000),
            ("ping_timeout_ms", self.ping_timeout_ms, 100, 86_400_000),
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
            ("replays_kept", wide(self.replays_kept), 1, 10_000_000),
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
            (
                "failed_logins_per_address",
                u64::from(self.failed_logins_per_address),
                1,
                u64::MAX,
            ),
            (
                "account_changes_per_address",
                u64::from(self.account_changes_per_address),
                1,
                u64::MAX,
            ),
            (
                "commands_per_game",
                u64::from(self.commands_per_game),
                1,
                1_000_000,
            ),
        ];
        for (name, value, min, max) in bounds {
            if !(min..=max).contains(&value) {
                return Err(Error(format!(
                    "limits.{name} = {value}: outside {min}..={max}"
                )));
            }
        }
        // Between two pings a live client says nothing (SRV-Q20).
        if self.ping_every_ms > 0 && self.ping_timeout_ms <= self.ping_every_ms {
            return Err(Error(format!(
                "limits.ping_timeout_ms = {}: it must be longer than ping_every_ms = {}",
                self.ping_timeout_ms, self.ping_every_ms
            )));
        }
        // A fetched record's frame escapes its text once more, at most doubling it.
        if self.record_bytes.saturating_mul(2) >= self.outbound_bytes {
            return Err(Error(format!(
                "limits.record_bytes = {}: a record must fit twice in outbound_bytes = {}",
                self.record_bytes, self.outbound_bytes
            )));
        }
        let commands = usize::try_from(self.commands_per_game).unwrap_or(usize::MAX);
        let replay = commands
            .saturating_mul(REPLAY_BYTES_PER_COMMAND)
            .saturating_add(REPLAY_HEAD_BYTES);
        if replay > self.outbound_bytes {
            return Err(Error(format!(
                "limits.commands_per_game = {}: its replay (up to {replay} bytes) must fit in \
                 outbound_bytes = {}",
                self.commands_per_game, self.outbound_bytes
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
        config.check_server()?;
        config.check_offers()?;
        config.check_ladders()?;
        config.server.data = base.join(&config.server.data);
        config.content.packs = base.join(&config.content.packs);
        for offer in &mut config.offers {
            offer.scenario = base.join(&offer.scenario);
        }
        for ladder in &mut config.ladders {
            for scenario in &mut ladder.scenarios {
                *scenario = base.join(&*scenario);
            }
        }
        Ok(config)
    }

    /// A public URL is a WebSocket address: `ws://` or `wss://`, then at most 200 visible ASCII
    /// characters.
    fn check_server(&self) -> Result<(), Error> {
        let Some(url) = &self.server.public_url else {
            return Ok(());
        };
        let rest = url
            .strip_prefix("ws://")
            .or_else(|| url.strip_prefix("wss://"))
            .unwrap_or_default();
        if rest.is_empty() || rest.len() > 200 || !rest.bytes().all(|b| b.is_ascii_graphic()) {
            return Err(Error(format!(
                "server.public_url = {url:?}: not a ws:// or wss:// address"
            )));
        }
        Ok(())
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

    /// Each ladder checked; ids distinct (E16 M4 T1); no offer named as a ladder's pool names its
    /// scenarios (`<ladder>.<n>`, E16 M4 T2); a bot on several ladders the same account on each
    /// (E16 M4 T5).
    fn check_ladders(&self) -> Result<(), Error> {
        for (i, ladder) in self.ladders.iter().enumerate() {
            ladder.check()?;
            if self.ladders[..i].iter().any(|l| l.id == ladder.id) {
                return Err(Error(format!("ladder `{}`: its id is taken", ladder.id)));
            }
            let before = self.ladders[..i].iter().flat_map(|l| &l.bots);
            let other =
                |bot: &LadderBot| before.clone().any(|b| b.login == bot.login && *b != *bot);
            if let Some(bot) = ladder.bots.iter().find(|bot| other(bot)) {
                return Err(Error(format!(
                    "ladder `{}`: bot `{}` differs from its entry on another ladder",
                    ladder.id, bot.login
                )));
            }
            let prefix = format!("{}.", ladder.id);
            if let Some(offer) = self.offers.iter().find(|o| o.id.starts_with(&prefix)) {
                return Err(Error(format!(
                    "offer `{}`: ladder `{}` names its pool's scenarios so",
                    offer.id, ladder.id
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
