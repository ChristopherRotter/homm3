//! The player log (E16 M3 T2a; ADR-0019 point 7; `E16-design.md` §10): `<data>/players.log`, one
//! JSON document per line, each an event of an account: registered (E16 M3 T2b adds re-keyed,
//! renamed and the listing preference). The accounts are read back into memory at start; a last
//! line a crash cut short is dropped. Each line is synced before the request it records is
//! answered. The log holds login names, display names and public keys: nothing a login can be
//! forged from; on Unix it is owner-read-write, as the journals are.
//!
//! Provisional (E16 M3 T2a): the layout, the names.

use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use h3_protocol::account::{self, KEY_BYTES};
use serde::{Deserialize, Serialize};

use crate::journal;

/// An account as the server holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Account {
    /// Its public key.
    pub key: [u8; KEY_BYTES],
    /// Its display name.
    pub display: String,
    /// Whether the replays of its finished games may be listed (E16 M3 T2b changes it).
    pub listed: bool,
}

/// One line of the log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
enum Line {
    /// A new account: its login name, its public key in hex and its display name.
    Registered {
        login: String,
        key: String,
        display: String,
    },
}

/// The accounts, and the log they are read from and written to.
#[derive(Debug)]
pub(crate) struct Players {
    /// The accounts by login name.
    accounts: BTreeMap<String, Account>,
    /// The login names by display name as it is compared ([`account::display_key`]).
    displays: BTreeMap<String, String>,
    /// The log, open for appending; `None` once a write failed: nothing more is written, so a
    /// line it cut short stays the last one, and the operator reads why on standard error.
    file: Option<File>,
}

/// The log's path in the data directory.
fn path(data: &Path) -> std::path::PathBuf {
    data.join("players.log")
}

impl Players {
    /// Reads `<data>/players.log` back, or creates it: a last line without its end is dropped and
    /// cut from the file; any other line that does not read, or that contradicts the lines before
    /// it, stops the start (an error that names the line, never its text).
    pub(crate) fn open(data: &Path) -> Result<Self, String> {
        let path = path(data);
        let shown = |e: io::Error| format!("{}: {e}", path.display());
        journal::create_dir(data).map_err(shown)?;
        let new = !path.exists();
        let mut options = OpenOptions::new();
        options.read(true).append(true).create(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        let file = options.open(&path).map_err(shown)?;
        #[cfg(unix)]
        if new {
            File::open(data).and_then(|d| d.sync_all()).map_err(shown)?;
        }
        let text = fs::read_to_string(&path).map_err(shown)?;
        let complete = text.rfind('\n').map_or(0, |end| end + 1);
        let mut players = Self {
            accounts: BTreeMap::new(),
            displays: BTreeMap::new(),
            file: None,
        };
        for (i, line) in text[..complete].lines().enumerate() {
            players
                .apply(line)
                .map_err(|e| format!("{} line {}: {e}", path.display(), i + 1))?;
        }
        if complete < text.len() {
            let len = u64::try_from(complete).unwrap_or(u64::MAX);
            file.set_len(len)
                .and_then(|()| file.sync_all())
                .map_err(shown)?;
        }
        players.file = Some(file);
        Ok(players)
    }

    /// Applies one line read back.
    fn apply(&mut self, line: &str) -> Result<(), String> {
        let line: Line = serde_json::from_str(line).map_err(|e| journal::json(&e))?;
        match line {
            Line::Registered {
                login,
                key,
                display,
            } => {
                let key = account::from_hex(&key).ok_or("not a key")?;
                if !account::is_login_name(&login) || !account::is_display_name(&display) {
                    return Err(String::from("not a name"));
                }
                if self.accounts.contains_key(&login) || self.display_taken(&display) {
                    return Err(String::from("a name registered before"));
                }
                self.insert(login, key, display);
            }
        }
        Ok(())
    }

    /// Adds an account to the tables.
    fn insert(&mut self, login: String, key: [u8; KEY_BYTES], display: String) -> Account {
        self.displays
            .insert(account::display_key(&display), login.clone());
        let account = Account {
            key,
            display,
            listed: true,
        };
        self.accounts.insert(login, account.clone());
        account
    }

    /// The account of login name `login`.
    pub(crate) fn get(&self, login: &str) -> Option<&Account> {
        self.accounts.get(login)
    }

    /// Whether an account has the display name `display`, ignoring case.
    pub(crate) fn display_taken(&self, display: &str) -> bool {
        self.displays.contains_key(&account::display_key(display))
    }

    /// Registers the account `login` with `key` and `display`, both free (the caller checked): its
    /// line is written and synced first.
    pub(crate) fn register(
        &mut self,
        login: &str,
        key: [u8; KEY_BYTES],
        display: &str,
    ) -> io::Result<Account> {
        self.append(&Line::Registered {
            login: login.to_owned(),
            key: account::to_hex(&key),
            display: display.to_owned(),
        })?;
        Ok(self.insert(login.to_owned(), key, display.to_owned()))
    }

    /// Writes `line` and syncs it; after a failure nothing more is written.
    fn append(&mut self, line: &Line) -> io::Result<()> {
        let file = self
            .file
            .as_mut()
            .ok_or_else(|| io::Error::other("a write failed before"))?;
        let mut text = serde_json::to_string(line).map_err(io::Error::other)?;
        text.push('\n');
        let written = file
            .write_all(text.as_bytes())
            .and_then(|()| file.sync_data());
        if written.is_err() {
            self.file = None;
        }
        written
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data() -> std::path::PathBuf {
        let random = crate::game::os_u64().unwrap();
        std::env::temp_dir().join(format!("h3-players-{random:016x}"))
    }

    #[test]
    fn the_accounts_are_read_back_and_a_torn_line_is_dropped() {
        let data = data();
        let mut players = Players::open(&data).unwrap();
        assert_eq!(players.get("alice"), None);
        let alice = players.register("alice", [1; 32], "Alice").unwrap();
        players.register("bob", [2; 32], "Bob the Second").unwrap();
        assert!(players.display_taken("ALICE") && !players.display_taken("Carol"));
        drop(players);
        let text = fs::read_to_string(path(&data)).unwrap();
        let first = format!(
            "{{\"Registered\":{{\"login\":\"alice\",\"key\":\"{}\",\"display\":\"Alice\"}}}}\n",
            "01".repeat(32)
        );
        assert!(text.starts_with(&first), "{text}");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = |p: &Path| fs::metadata(p).unwrap().permissions().mode() & 0o777;
            assert_eq!((mode(&path(&data)), mode(&data)), (0o600, 0o700));
        }
        // A crash cut the next line short: it is dropped, and the file continues after it.
        let mut file = OpenOptions::new().append(true).open(path(&data)).unwrap();
        file.write_all(b"{\"Registered\":{\"login\":\"car").unwrap();
        let mut players = Players::open(&data).unwrap();
        assert_eq!(players.get("alice"), Some(&alice));
        assert_eq!(players.get("bob").map(|a| a.key), Some([2; 32]));
        assert_eq!(fs::read_to_string(path(&data)).unwrap(), text);
        players.register("carol", [3; 32], "Carol").unwrap();
        drop(players);
        let players = Players::open(&data).unwrap();
        assert!(players.display_taken("carol"));
        fs::remove_dir_all(&data).unwrap();
    }

    #[test]
    fn a_line_that_does_not_read_or_contradicts_stops_the_start() {
        let alice = format!(
            "{{\"Registered\":{{\"login\":\"alice\",\"key\":\"{}\",\"display\":\"Alice\"}}}}\n",
            "01".repeat(32)
        );
        let cases = [
            (format!("{alice}{{\"Registered\":{{}}}}\n"), "line 2: Data"),
            (alice.replace("alice", "Alice"), "line 1: not a name"),
            (alice.replace("\"01", "\"zz"), "line 1: not a key"),
            (
                format!("{alice}{alice}"),
                "line 2: a name registered before",
            ),
            (
                format!(
                    "{alice}{}",
                    alice.replace("login\":\"alice", "login\":\"bob")
                ),
                "line 2: a name registered before",
            ),
        ];
        for (text, expected) in cases {
            let data = data();
            journal::create_dir(&data).unwrap();
            fs::write(path(&data), &text).unwrap();
            let error = Players::open(&data).unwrap_err();
            assert!(error.contains(expected), "{error}");
            assert!(!error.contains("Alice"), "never the line's text: {error}");
            fs::remove_dir_all(&data).unwrap();
        }
    }

    #[test]
    fn after_a_failed_write_nothing_more_is_written() {
        let data = data();
        let mut players = Players::open(&data).unwrap();
        // A file opened for reading only: the write fails.
        players.file = Some(File::open(path(&data)).unwrap());
        assert!(players.register("alice", [1; 32], "Alice").is_err());
        assert!(players.file.is_none() && players.get("alice").is_none());
        let again = players.register("alice", [1; 32], "Alice").unwrap_err();
        assert_eq!(again.to_string(), "a write failed before");
        fs::remove_dir_all(&data).unwrap();
    }
}
