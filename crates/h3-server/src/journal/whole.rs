//! A whole game's journal (E16 M5 T9a): one file per game, `<data>/whole/<game>.journal`, one JSON
//! document per line, written ahead as a battle's journal is ([`Writer`]). The first line starts
//! the game ([`GameStart`]): the format, the game, its map offer, its creator's address, the
//! seats' tokens by player and the game's adventure replay before its first command (the replay
//! file of `h3_files::adventure`: the header, the map, the seed, the difficulty, the checkpoint
//! interval). Every further line is one of the whole game's session's entries ([`GameEntry`]): a
//! seat taken (by an account, with its login name), the start, a command (the host's with its
//! actor, a seat's with its number; in the replay's text form), a state hash, a pause and a
//! resume. A whole game's journal has its own format ([`GAME_JOURNAL_FORMAT_VERSION`]), beside the
//! battle's.
//!
//! Provisional (the 2a review list): the directory, the lines, the replay's map named by its
//! file's name (its hash is what counts).

use std::io;
use std::net::IpAddr;
use std::path::{Path, PathBuf};

use h3_contracts::PlayerId;
use h3_game::Actor;
use h3_protocol::session::game::GameEntry;
use serde::{Deserialize, Serialize};

use super::{Writer, read_as};

/// The version of a whole game's journal's format: bumped when its lines change (1 since E16 M5
/// T9a).
pub const GAME_JOURNAL_FORMAT_VERSION: u32 = 1;

/// A whole game's journal's first line: the game as it started.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GameStart {
    /// [`GAME_JOURNAL_FORMAT_VERSION`] when it was written.
    pub format: u32,
    /// The game's id.
    pub game: u32,
    /// The map offer it was created from.
    pub offer: String,
    /// Where its creation came from (the lobby's limit per address).
    pub origin: IpAddr,
    /// The seats' tokens, by player.
    pub tokens: Vec<String>,
    /// The game's adventure replay before its first command, as its file.
    pub replay: String,
}

/// One line after the first: an entry of the whole game's session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
enum GameLine {
    /// [`GameEntry::Command`] in the replay's text form (`h3_files::adventure::format_command`),
    /// with its actor.
    Command { command: String },
    /// [`GameEntry::Numbered`]: the seat's number and the command in the replay's text form, as
    /// the seat's player's.
    Numbered { seq: u32, command: String },
    /// [`GameEntry::Checkpoint`] as 16 hex digits.
    Checkpoint { hash: String },
    /// [`GameEntry::Taken`], or [`GameEntry::TakenBy`] with the account's login name.
    Taken {
        player: u8,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        account: Option<String>,
    },
    /// [`GameEntry::Started`].
    Started {},
    /// [`GameEntry::Paused`].
    Paused {},
    /// [`GameEntry::Resumed`].
    Resumed {},
}

impl GameLine {
    fn of(entry: GameEntry) -> Self {
        let text = h3_files::adventure::format_command;
        match entry {
            GameEntry::Command(actor, command) => Self::Command {
                command: text(actor, &command),
            },
            GameEntry::Numbered(player, seq, command) => Self::Numbered {
                seq,
                command: text(Actor::Player(player), &command),
            },
            GameEntry::Checkpoint(hash) => Self::Checkpoint {
                hash: format!("{hash:#018x}"),
            },
            GameEntry::Taken(player) => Self::Taken {
                player: player.raw(),
                account: None,
            },
            GameEntry::TakenBy(player, login) => Self::Taken {
                player: player.raw(),
                account: Some(login),
            },
            GameEntry::Started => Self::Started {},
            GameEntry::Paused => Self::Paused {},
            GameEntry::Resumed => Self::Resumed {},
        }
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "E16 M5 T9b's restore at start reads it (SRV-STUB-12)"
        )
    )]
    fn entry(self) -> Result<GameEntry, String> {
        let parse =
            |text: &str| h3_files::adventure::parse_command(text).map_err(|e| e.to_string());
        Ok(match self {
            Self::Command { command } => {
                let (actor, command) = parse(&command)?;
                GameEntry::Command(actor, command)
            }
            Self::Numbered { seq, command } => match parse(&command)? {
                (Actor::Player(player), command) => GameEntry::Numbered(player, seq, command),
                (Actor::System, _) => return Err(String::from("a seat's command is no player's")),
            },
            Self::Checkpoint { hash } => hash
                .strip_prefix("0x")
                .and_then(|hex| u64::from_str_radix(hex, 16).ok())
                .map(GameEntry::Checkpoint)
                .ok_or_else(|| String::from("bad checkpoint"))?,
            Self::Taken { player, account } => {
                let player = PlayerId::new(player);
                match account {
                    Some(login) => GameEntry::TakenBy(player, login),
                    None => GameEntry::Taken(player),
                }
            }
            Self::Started {} => GameEntry::Started,
            Self::Paused {} => GameEntry::Paused,
            Self::Resumed {} => GameEntry::Resumed,
        })
    }
}

/// The whole games' journals' directory in the data directory `data`.
pub(crate) fn dir(data: &Path) -> PathBuf {
    data.join("whole")
}

/// Creates the journal of the whole game `start` describes in `data`, its first line synced, and
/// its directory entry too. An existing file is never overwritten.
pub(crate) fn create(data: &Path, start: &GameStart) -> io::Result<Writer> {
    Writer::create_in(&dir(data), start.game, start)
}

impl Writer {
    /// Appends a whole game's `entries` as lines and syncs them: the write-ahead before a call's
    /// frames are sent. Nothing happens without entries.
    pub(crate) fn append_game(
        &mut self,
        entries: impl Iterator<Item = GameEntry>,
    ) -> io::Result<()> {
        self.append_lines(entries.map(GameLine::of))
    }
}

/// A whole game's journal as read back.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "E16 M5 T9b's restore at start reads it (SRV-STUB-12)"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GameRead {
    /// Its first line.
    pub start: GameStart,
    /// The entries after it.
    pub entries: Vec<GameEntry>,
    /// The length of its complete lines, in bytes.
    pub complete: u64,
}

/// Reads the whole game's journal at `path`; why it cannot be read, otherwise.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "E16 M5 T9b's restore at start reads it (SRV-STUB-12)"
    )
)]
pub(crate) fn read(path: &Path) -> Result<GameRead, String> {
    let (start, entries, complete) =
        read_as::<GameStart, _, _>(path, GAME_JOURNAL_FORMAT_VERSION, GameLine::entry)?;
    Ok(GameRead {
        start,
        entries,
        complete,
    })
}
