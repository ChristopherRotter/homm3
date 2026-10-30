//! `h3-view connect` (E17 M5 T6a): a game on a server, headless. It dials the server
//! ([`crate::net`]), says hello, checks the welcome and pins the server's key per address
//! (`h3_view::remote`, the pins file), joins a game by its seat's token, runs the content check
//! (the manifest's packs and profile compiled here, the ruleset hashes compared), plays the seat
//! with the plain player ([`plain`]) until the game ends, re-joining after a drop by the token,
//! and writes the seat's record as the server kept it. The window's live play over a server
//! builds on it (M5 T6b). An adapter: the socket, the files, the clock's timeouts.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use h3_protocol::codec;
use h3_protocol::map;
use h3_protocol::wire::{Command, Seat, ServerMessage};
use h3_view::record::Record;
use h3_view::remote::{self, Client, Content, Note, Pins};
use rustls::RootCertStore;

use crate::assets;
use crate::net::{self, Connection, Incoming};

#[cfg(test)]
mod tests;

/// The program's name in the hello (the window's too, E17 M5 T6b1).
pub const PROGRAM: &str = concat!("h3-view ", env!("CARGO_PKG_VERSION"));

/// How long a headless session waits for the server's next message.
const SILENCE: Duration = Duration::from_secs(30);

/// The most commands the plain player sends before it gives up (*provisional*, as `play`'s
/// `FOUGHT_LIMIT`).
const COMMANDS: u32 = 5_000;

/// The command's options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// The address dialed.
    pub address: String,
    /// The game joined, or the server's offer a game is created of.
    pub game: Game,
    pub seat: Seat,
    pub token: Option<String>,
    /// The player's display name in the hello.
    pub name: String,
    /// The pins file.
    pub pins: PathBuf,
    /// The packs the content check compiles.
    pub packs: PathBuf,
    /// Where the record goes.
    pub out: Option<PathBuf>,
    /// Drops the connection after this many commands sent and re-joins (a test of the re-join).
    pub drop: Option<u32>,
}

/// The game a session plays.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Game {
    /// A hosted game, by its id.
    Join(u32),
    /// A new game of the server's offer, by the offer's id.
    Create(String),
}

impl Options {
    /// Parses `<ws://… | wss://…> (--game <id> | --create <offer>)
    /// --seat <attacker|defender|spectator>
    /// [--token <t>] [--name <display>] [--pins <file>] [--packs <dir>] [--out <file>]
    /// [--drop <n>]`.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let clients = assets::clients_dir();
        let (mut address, mut game, mut seat, mut token) = (None, None, None, None);
        let mut create = None;
        let (mut name, mut pins, mut packs, mut out, mut drop) = (None, None, None, None, None);
        let mut rest = args.iter();
        while let Some(arg) = rest.next() {
            let mut value = || {
                rest.next()
                    .cloned()
                    .ok_or_else(|| format!("connect: `{arg}` needs a value"))
            };
            let number = |v: String| {
                v.parse::<u32>()
                    .map_err(|_| format!("connect: bad number `{v}` for `{arg}`"))
            };
            match arg.as_str() {
                "--game" => game = Some(number(value()?)?),
                "--create" => create = Some(value()?),
                "--seat" => {
                    seat = Some(match value()?.as_str() {
                        "attacker" => Seat::Attacker,
                        "defender" => Seat::Defender,
                        "spectator" => Seat::Spectator,
                        other => return Err(format!("connect: unknown seat `{other}`")),
                    });
                }
                "--token" => token = Some(value()?),
                "--name" => name = Some(value()?),
                "--pins" => pins = Some(PathBuf::from(value()?)),
                "--packs" => packs = Some(PathBuf::from(value()?)),
                "--out" => out = Some(PathBuf::from(value()?)),
                "--drop" => drop = Some(number(value()?)?),
                flag if flag.starts_with("--") => {
                    return Err(format!("connect: unknown option `{flag}`"));
                }
                url if address.is_none() => address = Some(url.to_owned()),
                extra => return Err(format!("connect: unexpected `{extra}`")),
            }
        }
        Ok(Self {
            address: address.ok_or("connect: needs the server's address (ws://… or wss://…)")?,
            game: match (game, create) {
                (Some(id), None) => Game::Join(id),
                (None, Some(offer)) => Game::Create(offer),
                (None, None) => return Err("connect: needs --game or --create".to_owned()),
                (Some(_), Some(_)) => {
                    return Err("connect: --game or --create, not both".to_owned());
                }
            },
            seat: seat.ok_or("connect: needs --seat")?,
            token,
            name: name.unwrap_or_else(|| "player".to_owned()),
            pins: pins.unwrap_or_else(|| PathBuf::from("pins.txt")),
            packs: packs.unwrap_or_else(|| clients.join("../packs")),
            out,
            drop,
        })
    }

    /// Where the record of `game` goes: `clients/out/connect/g<game>-<seat>.jsonl` by default.
    fn record_path(&self, game: u32) -> PathBuf {
        self.out.clone().unwrap_or_else(|| {
            let seat = format!("{:?}", self.seat).to_lowercase();
            assets::clients_dir()
                .join("out/connect")
                .join(format!("g{game}-{seat}.jsonl"))
        })
    }
}

/// Runs the command.
pub fn run(options: &Options) -> ExitCode {
    let ran = session(options, net::os_roots).and_then(|done| {
        let path = options.record_path(done.record.header.game);
        write(&path, &text(&done.record))?;
        Ok((done, path))
    });
    match ran {
        Ok((done, path)) => {
            for line in &done.lines {
                println!("{line}");
            }
            println!(
                "content: {:?}; re-joined {} times",
                done.content, done.rejoined
            );
            println!("the record: {}", path.display());
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("h3-view connect: {message}");
            ExitCode::FAILURE
        }
    }
}

/// What a session ended with.
#[derive(Debug)]
pub struct Done {
    /// The seat's record, as the server kept it at the end.
    pub record: Record,
    /// The content check's verdict.
    pub content: Content,
    /// How often the connection was re-joined.
    pub rejoined: u32,
    /// What happened, a line each.
    pub lines: Vec<String>,
}

/// A session: dials, joins and plays until the game ends, re-joining once after the drop
/// `options` asks for; `roots` checks a `wss://` server's certificate.
pub fn session(options: &Options, roots: fn() -> RootCertStore) -> Result<Done, String> {
    let mut pins = read_pins(&options.pins)?;
    let mut token = options.token.clone();
    let mut sent = 0;
    let mut drop = options.drop;
    let mut lines = Vec::new();
    let mut content = None;
    let mut rejoined = 0;
    // The game joined: a re-join joins it, not a new one.
    let mut joined = None;
    loop {
        let pinned = pins.key(&options.address);
        let mut client = Client::new(&options.address, net::nonce()?, pinned);
        let connection = Connection::open(&options.address, roots);
        connection.send(client.hello(PROGRAM, &options.name));
        // Whether a command was sent for the turn and not answered yet.
        let mut asked = false;
        loop {
            let message = match connection.wait(SILENCE) {
                None => return Err(format!("nothing from the server in {SILENCE:?}")),
                Some(Incoming::Closed(why)) => return Err(why),
                Some(Incoming::Message(message)) => message,
            };
            let notes = client
                .receive(*message, net::verify)
                .map_err(|f| f.to_string())?;
            for note in notes {
                match note {
                    Note::Send(message) => connection.send(message),
                    Note::Pinned(key) => {
                        pins.pin(&options.address, key);
                        write(&options.pins, &pins.text())?;
                        lines.push(format!("pinned the server's key {}", hex(&key)));
                    }
                    Note::Welcomed(server) => {
                        lines.push(format!("welcomed by {server}"));
                        let ask = match (&options.game, joined) {
                            (_, Some(game)) | (&Game::Join(game), None) => {
                                client.join(game, options.seat, token.clone())
                            }
                            (Game::Create(offer), None) => client.create(offer, options.seat),
                        };
                        connection.send(ask.ok_or("the join after the welcome")?);
                    }
                    Note::Created(game) => lines.push(format!("created game {game}")),
                    Note::Joined(game, seat) => {
                        joined = Some(game);
                        lines.push(format!("joined game {game} as {seat:?}"));
                    }
                    Note::Refused(reason) => return Err(format!("refused: {reason:?}")),
                    Note::Opened => {
                        let record = client.record().ok_or("the record opened")?;
                        content = Some(check(&options.packs, &record.header.manifest));
                        lines.push(format!(
                            "the record opens at message {} ({} messages)",
                            record.header.first,
                            record.messages.len()
                        ));
                    }
                    Note::Grew => {
                        let last = client.record().and_then(|r| r.messages.last());
                        if matches!(
                            last,
                            Some(ServerMessage::Accepted { .. } | ServerMessage::Rejected { .. })
                        ) {
                            asked = false;
                        }
                    }
                    Note::Goodbye(reason) => lines.push(format!("goodbye: {reason:?}")),
                    Note::Other(_) => {}
                }
            }
            if let Some(seat) = client.token() {
                token = Some(seat.to_owned());
            }
            if client.over() {
                let record = client.record().cloned().ok_or("the record at the end")?;
                let content = content.ok_or("the content check")?;
                lines.push(format!("the game ended after {sent} commands sent here"));
                return Ok(Done {
                    record,
                    content,
                    rejoined,
                    lines,
                });
            }
            if client.closed() {
                return Err("the server said goodbye before the game's end".to_owned());
            }
            let Some((legal, _)) = client.turn().filter(|_| !asked) else {
                continue;
            };
            if sent >= COMMANDS {
                return Err(format!("no end after {COMMANDS} commands"));
            }
            let width = client
                .record()
                .map_or(17, |r| r.header.manifest.field.width);
            let command = plain(legal, options.seat, width).ok_or("a turn without a command")?;
            connection.send(
                client
                    .command(command)
                    .ok_or("a command before the record")?,
            );
            (sent, asked) = (sent + 1, true);
            if drop.is_some_and(|after| sent >= after) {
                drop = None;
                rejoined += 1;
                lines.push(format!("dropped the connection after {sent} commands"));
                break;
            }
        }
    }
}

/// The plain player's command (*provisional*, a test's player, not an opponent): ends a tactics
/// phase at once, shoots or strikes if it may, else walks toward the enemy's edge of the field
/// (`width` hexes a row: the attacker eastward, the defender westward), else defends or waits;
/// never leaves the battle. The first such command the server lists.
pub fn plain(legal: &[Command], seat: Seat, width: u8) -> Option<Command> {
    let first = |f: fn(&Command) -> bool| legal.iter().copied().find(f);
    let column = |to: u8| to % width.max(1);
    first(|c| matches!(c, Command::EndTactics {}))
        .or_else(|| first(|c| matches!(c, Command::Shoot { .. })))
        .or_else(|| first(|c| matches!(c, Command::Attack { .. })))
        .or_else(|| {
            let moves = legal.iter().filter_map(|c| match *c {
                Command::Move { to } => Some(to),
                _ => None,
            });
            let to = match seat {
                Seat::Defender => moves.min_by_key(|&to| column(to)),
                _ => moves.max_by_key(|&to| (column(to), u8::MAX - to)),
            };
            to.map(|to| Command::Move { to })
        })
        .or_else(|| first(|c| matches!(c, Command::Defend {})))
        .or_else(|| first(|c| matches!(c, Command::Wait {})))
}

/// The content check: the packs and the profile `manifest` names, compiled from `packs`.
fn check(packs: &Path, manifest: &h3_protocol::wire::Manifest) -> Content {
    let own = crate::scenario::ruleset(packs, manifest)
        .ok()
        .map(|ruleset| map::manifest(&ruleset));
    remote::content(manifest, own.as_ref())
}

/// The pins file at `path`; none yet when it does not exist.
pub fn read_pins(path: &Path) -> Result<Pins, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => Pins::parse(&text).map_err(|line| format!("{}: line {line}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Pins::default()),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

/// A record's text: its header, then a line per message.
pub fn text(record: &Record) -> String {
    let mut text = codec::encode_text(&record.header);
    for message in &record.messages {
        text.push('\n');
        text.push_str(&codec::encode_text(message));
    }
    text.push('\n');
    text
}

/// Writes `text` to `path`, its folder made first.
pub fn write(path: &Path, text: &str) -> Result<(), String> {
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))
}

/// `bytes` as lowercase hex.
pub fn hex(bytes: &[u8]) -> String {
    h3_protocol::account::to_hex(bytes)
}
