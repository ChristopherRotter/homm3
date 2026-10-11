//! The games' journals (E16 M2 T2; ADR-0018 point 12; `E16-design.md` §7): one file per game,
//! `<data>/games/<game>.journal`, one JSON document per line. The first line starts the game
//! ([`Start`]): the journal's format, the game, its offer, its creator's address, the seats'
//! tokens and the game's replay before its first command (the CLI's replay file: the replay
//! header, the seed, the setup, the checkpoint interval). Every further line is one of the session
//! core's entries ([`Entry`]): a seat taken, the start, an accepted command (in the replay's text
//! form), a state hash, a pause and a resume (format 2, E16 M2 T3), whether the game takes
//! spectators as it started (format 3, E16 M2 T4c).
//!
//! - **Write-ahead.** The game's thread appends the entries of each call and syncs them before it
//!   sends that call's frames ([`Writer::append`]); a new journal is synced with its directory
//!   before the game is listed.
//! - **Secrets.** The journal holds the battle's seed and the seats' tokens: it is the operator's
//!   copy of the tokens (a player who lost its `Joined` gets its seat's token from the first
//!   line). On Unix the server makes the directories it creates owner-only and the journals
//!   owner-read-write; the data directory must stay readable by the server's user only.
//! - **The end.** A finished game's journal goes once its replay is written (the replay is the
//!   finished journal); a game that never started leaves none. A journal the server cannot restore
//!   moves to `<data>/void/`.
//!
//! Provisional (E16 M2 T2): the layout, the names, the void directory (the epic's review list).

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::net::IpAddr;
use std::path::{Path, PathBuf};

use h3_battle::BattleSide;
use h3_protocol::session::Entry;
use serde::{Deserialize, Serialize};

/// The version of the journal's format: bumped when its lines change.
pub const JOURNAL_FORMAT_VERSION: u32 = 3;

/// The journal's first line: the game as it started.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Start {
    /// [`JOURNAL_FORMAT_VERSION`] when it was written.
    pub format: u32,
    /// The game's id.
    pub game: u32,
    /// The offer it was created from.
    pub offer: String,
    /// Where its creation came from (the lobby's limit per address).
    pub origin: IpAddr,
    /// The attacker's and the defender's token; none for the bot's seat.
    pub tokens: [Option<String>; 2],
    /// The game's replay before its first command, as the CLI's replay file.
    pub replay: String,
}

/// A side in a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Side {
    Attacker,
    Defender,
}

/// One line after the first: an entry of the session core.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
enum Line {
    /// [`Entry::Taken`].
    Taken { side: Side },
    /// [`Entry::Started`].
    Started {},
    /// [`Entry::Command`] in the replay's text form (`h3_cli::format_command`).
    Command { command: String },
    /// [`Entry::Checkpoint`] as 16 hex digits.
    Checkpoint { hash: String },
    /// [`Entry::Paused`] (E16 M2 T3).
    Paused {},
    /// [`Entry::Resumed`] (E16 M2 T3).
    Resumed {},
    /// [`Entry::Spectators`] (E16 M2 T4c).
    Spectators { open: bool },
}

impl Line {
    fn of(entry: Entry) -> Self {
        match entry {
            Entry::Taken(BattleSide::Attacker) => Self::Taken {
                side: Side::Attacker,
            },
            Entry::Taken(BattleSide::Defender) => Self::Taken {
                side: Side::Defender,
            },
            Entry::Started => Self::Started {},
            Entry::Command(side, command) => Self::Command {
                command: h3_cli::format_command(side, command),
            },
            Entry::Checkpoint(hash) => Self::Checkpoint {
                hash: format!("{hash:#018x}"),
            },
            Entry::Paused => Self::Paused {},
            Entry::Resumed => Self::Resumed {},
            Entry::Spectators(open) => Self::Spectators { open },
        }
    }

    fn entry(self) -> Result<Entry, String> {
        Ok(match self {
            Self::Taken {
                side: Side::Attacker,
            } => Entry::Taken(BattleSide::Attacker),
            Self::Taken {
                side: Side::Defender,
            } => Entry::Taken(BattleSide::Defender),
            Self::Started {} => Entry::Started,
            Self::Command { command } => {
                let (side, command) = h3_cli::parse_command(&command).map_err(|e| e.to_string())?;
                Entry::Command(side, command)
            }
            Self::Checkpoint { hash } => hash
                .strip_prefix("0x")
                .and_then(|hex| u64::from_str_radix(hex, 16).ok())
                .map(Entry::Checkpoint)
                .ok_or_else(|| String::from("bad checkpoint"))?,
            Self::Paused {} => Entry::Paused,
            Self::Resumed {} => Entry::Resumed,
            Self::Spectators { open } => Entry::Spectators(open),
        })
    }
}

/// Creates `dir` and its missing parents, owner-only on Unix.
pub(crate) fn create_dir(dir: &Path) -> io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder.create(dir)
}

/// The journals' directory in the data directory `data`.
pub(crate) fn dir(data: &Path) -> PathBuf {
    data.join("games")
}

/// An open journal, appended to by its game's thread.
#[derive(Debug)]
pub(crate) struct Writer {
    file: File,
    path: PathBuf,
}

impl Writer {
    /// Creates the journal of the game `start` describes in `data`, its first line synced, and
    /// its directory entry too. An existing file is never overwritten.
    pub(crate) fn create(data: &Path, start: &Start) -> io::Result<Self> {
        let dir = dir(data);
        create_dir(&dir)?;
        let path = dir.join(format!("{}.journal", start.game));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        let mut file = options.open(&path)?;
        let mut line = serde_json::to_string(start).map_err(io::Error::other)?;
        line.push('\n');
        file.write_all(line.as_bytes())?;
        file.sync_all()?;
        #[cfg(unix)]
        File::open(&dir)?.sync_all()?;
        Ok(Self { file, path })
    }

    /// Opens a journal that [`read`] read, to append to it: a last line a crash cut short goes
    /// first (it was never synced, so nothing that depends on it was sent). A journal without one
    /// is left as it is: a truncation refreshes the file's last write even to its own length, and a
    /// paused game's deadline counts from it (E16 M2 T5e: the reviewer's finding).
    pub(crate) fn reopen(path: &Path, complete: u64) -> io::Result<Self> {
        let file = OpenOptions::new().write(true).open(path)?;
        if file.metadata()?.len() != complete {
            file.set_len(complete)?;
            file.sync_all()?;
        }
        let mut writer = Self {
            file,
            path: path.to_path_buf(),
        };
        io::Seek::seek(&mut writer.file, io::SeekFrom::End(0))?;
        Ok(writer)
    }

    /// Appends `entries` as lines and syncs them: the write-ahead before a call's frames are sent.
    /// Nothing happens without entries.
    pub(crate) fn append(&mut self, entries: impl Iterator<Item = Entry>) -> io::Result<()> {
        let mut text = String::new();
        for entry in entries {
            let line = serde_json::to_string(&Line::of(entry)).map_err(io::Error::other)?;
            text.push_str(&line);
            text.push('\n');
        }
        if text.is_empty() {
            return Ok(());
        }
        self.file.write_all(text.as_bytes())?;
        self.file.sync_data()
    }

    /// The journal's file.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    /// The game is over and its replay written, or it left without having started: its journal
    /// goes. A failure is logged.
    pub(crate) fn remove(self) {
        if let Err(e) = fs::remove_file(&self.path) {
            eprintln!("h3-server: journal {}: {e}", self.path.display());
        }
    }
}

/// A journal as read back at start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Read {
    /// Its first line.
    pub start: Start,
    /// The entries after it.
    pub entries: Vec<Entry>,
    /// The length of its complete lines, in bytes (a last line without its newline is a write a
    /// crash cut short).
    pub complete: u64,
}

/// Reads the journal at `path`; why it cannot be read, otherwise.
pub(crate) fn read(path: &Path) -> Result<Read, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let complete = text.rfind('\n').map_or(0, |end| end + 1);
    let mut lines = text[..complete].lines();
    let start: Start = lines
        .next()
        .ok_or_else(|| String::from("no first line"))
        .and_then(|line| {
            serde_json::from_str(line).map_err(|e| format!("first line: {}", json(&e)))
        })?;
    if start.format != JOURNAL_FORMAT_VERSION {
        return Err(format!("journal format {}", start.format));
    }
    let entries = lines
        .enumerate()
        .map(|(i, line)| {
            serde_json::from_str::<Line>(line)
                .map_err(|e| json(&e))
                .and_then(Line::entry)
                .map_err(|e| format!("line {}: {e}", i + 2))
        })
        .collect::<Result<_, _>>()?;
    Ok(Read {
        start,
        entries,
        complete: u64::try_from(complete).unwrap_or(u64::MAX),
    })
}

/// Why a line does not read, without its text: a line may hold the seats' tokens, which never
/// reach a log line (ADR-0018 point 8).
pub(crate) fn json(e: &serde_json::Error) -> String {
    format!("{:?} error at column {}", e.classify(), e.column())
}

/// Moves the journal at `path` to `<data>/void/`, with a random part in its name, and logs why.
pub(crate) fn void(data: &Path, path: &Path, game: &str, why: &str) {
    let dir = data.join("void");
    let name = path.file_stem().map_or_else(
        || String::from("journal"),
        |s| s.to_string_lossy().into_owned(),
    );
    let random = crate::game::os_u64().unwrap_or_default();
    let to = dir.join(format!("{name}-{random:016x}.journal"));
    let moved = create_dir(&dir).and_then(|()| fs::rename(path, &to));
    match moved {
        Ok(()) => eprintln!(
            "h3-server: game {game} void: {why}; its journal is {}",
            to.display()
        ),
        Err(e) => eprintln!(
            "h3-server: game {game} void: {why}; {}: {e}",
            path.display()
        ),
    }
}

#[cfg(test)]
mod tests {
    use h3_battle::{Command, Hex};

    use super::*;

    fn start(game: u32) -> Start {
        Start {
            format: JOURNAL_FORMAT_VERSION,
            game,
            offer: String::from("duel"),
            origin: IpAddr::from([127, 0, 0, 1]),
            tokens: [Some(String::from("0123")), None],
            replay: String::from("[replay]\nformat = 1\n"),
        }
    }

    fn data() -> PathBuf {
        let random = crate::game::os_u64().unwrap();
        std::env::temp_dir().join(format!("h3-journal-{random:016x}"))
    }

    #[test]
    fn a_journal_reads_back_what_was_written() {
        let data = data();
        let mut writer = Writer::create(&data, &start(4)).unwrap();
        assert!(
            Writer::create(&data, &start(4)).is_err(),
            "never overwritten"
        );
        let entries = [
            Entry::Taken(BattleSide::Attacker),
            Entry::Taken(BattleSide::Defender),
            Entry::Started,
            Entry::Command(BattleSide::Attacker, Command::Move { to: Hex(40) }),
            Entry::Command(BattleSide::Defender, Command::Wait),
            Entry::Checkpoint(0x0123_4567_89ab_cdef),
            Entry::Paused,
            Entry::Resumed,
            Entry::Spectators(false),
        ];
        writer.append(entries.into_iter()).unwrap();
        writer.append(std::iter::empty()).unwrap();
        let path = writer.path().to_path_buf();
        assert_eq!(path, data.join("games/4.journal"));
        let read = read(&path).unwrap();
        assert_eq!((read.start, &read.entries[..]), (start(4), &entries[..]));
        let text = fs::read_to_string(&path).unwrap();
        assert_eq!(read.complete, text.len() as u64);
        assert!(
            text.contains(r#"{"Command":{"command":"A move 40"}}"#),
            "{text}"
        );
        assert!(text.contains(r#"{"Checkpoint":{"hash":"0x0123456789abcdef"}}"#));
        assert!(
            text.ends_with(
                "{\"Paused\":{}}\n{\"Resumed\":{}}\n{\"Spectators\":{\"open\":false}}\n"
            ),
            "{text}"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = |p: &Path| fs::metadata(p).unwrap().permissions().mode() & 0o777;
            assert_eq!((mode(&path), mode(&dir(&data))), (0o600, 0o700));
        }
        writer.remove();
        assert!(!path.exists());
        fs::remove_dir_all(&data).unwrap();
    }

    #[test]
    fn a_line_a_crash_cut_short_is_dropped_and_overwritten() {
        let data = data();
        let mut writer = Writer::create(&data, &start(2)).unwrap();
        writer.append([Entry::Started].into_iter()).unwrap();
        let path = writer.path().to_path_buf();
        drop(writer);
        let mut file = OpenOptions::new().append(true).open(&path).unwrap();
        file.write_all(br#"{"Command":{"comm"#).unwrap();
        let read = read(&path).unwrap();
        assert_eq!(read.entries, [Entry::Started]);
        let mut writer = Writer::reopen(&path, read.complete).unwrap();
        writer
            .append([Entry::Taken(BattleSide::Defender)].into_iter())
            .unwrap();
        assert_eq!(
            super::read(&path).unwrap().entries,
            [Entry::Started, Entry::Taken(BattleSide::Defender)]
        );
        fs::remove_dir_all(&data).unwrap();
    }

    /// E16 M2 T5e (the reviewer's finding): a whole journal reopened keeps its last write, from
    /// which a paused game's deadline counts after a restart.
    #[test]
    fn a_whole_journal_reopened_keeps_its_last_write() {
        let data = data();
        let mut writer = Writer::create(&data, &start(3)).unwrap();
        writer.append([Entry::Paused].into_iter()).unwrap();
        let path = writer.path().to_path_buf();
        drop(writer);
        let long_ago = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_000_000_000);
        File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(long_ago)
            .unwrap();
        let read = read(&path).unwrap();
        let mut writer = Writer::reopen(&path, read.complete).unwrap();
        writer.append(std::iter::empty()).unwrap();
        assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), long_ago);
        fs::remove_dir_all(&data).unwrap();
    }

    #[test]
    fn journals_that_do_not_read_say_why() {
        let data = data();
        create_dir(&data).unwrap();
        let path = data.join("1.journal");
        let first = serde_json::to_string(&start(1)).unwrap();
        let cases = [
            (String::new(), "no first line"),
            (String::from("{}\n"), "first line"),
            (
                first.replace(
                    &format!("\"format\":{JOURNAL_FORMAT_VERSION}"),
                    "\"format\":9",
                ) + "\n",
                "journal format 9",
            ),
            (
                format!("{first}\n{{\"Started\":{{}}}}\n{{\"Moved\":{{}}}}\n"),
                "line 3",
            ),
            (
                format!("{first}\n{{\"Command\":{{\"command\":\"A fly\"}}}}\n"),
                "bad command `A fly`",
            ),
            (
                format!("{first}\n{{\"Checkpoint\":{{\"hash\":\"12\"}}}}\n"),
                "line 2: bad checkpoint",
            ),
        ];
        for (text, why) in cases {
            fs::write(&path, &text).unwrap();
            let error = read(&path).unwrap_err();
            assert!(error.contains(why), "{error} for {text}");
        }
        assert!(read(&data.join("none.journal")).is_err());
        // A journal that cannot be restored moves aside.
        void(&data, &path, "1", "a test");
        assert!(!path.exists());
        let moved: Vec<_> = fs::read_dir(data.join("void")).unwrap().collect();
        assert_eq!(moved.len(), 1);
        fs::remove_dir_all(&data).unwrap();
    }
}
