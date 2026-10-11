//! The player log (E16 M3 T2a, T2b; ADR-0019 point 7; `E16-design.md` §10): `<data>/players.log`,
//! one JSON document per line, each an [`Event`] of an account: registered, re-keyed by itself or
//! rebound by the operator, renamed, its listing preference. The accounts are read back into
//! memory at start; a last line a crash cut short is dropped. Each line is synced before the
//! request it records is answered. The log holds login names, display names and public keys:
//! nothing a login can be forged from; on Unix it is owner-read-write, as the journals are.
//!
//! Provisional (E16 M3 T2a, T2b): the layout, the names.

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
    /// Whether the replays of its finished games may be listed.
    pub listed: bool,
}

/// A public key in a line: hex in the log, bytes in memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub(crate) struct Hex(pub [u8; KEY_BYTES]);

impl TryFrom<String> for Hex {
    type Error = &'static str;

    fn try_from(hex: String) -> Result<Self, Self::Error> {
        account::from_hex(&hex).map(Self).ok_or("not a key")
    }
}

impl From<Hex> for String {
    fn from(key: Hex) -> Self {
        account::to_hex(&key.0)
    }
}

/// An event of an account: one line of the log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum Event {
    /// A new account (E16 M3 T2a).
    Registered {
        login: String,
        key: Hex,
        display: String,
    },
    /// The account re-keyed itself, signed by both keys (E16 M3 T2b).
    Rekeyed { login: String, key: Hex },
    /// The operator bound the account to a new key, the server stopped (E16 M3 T2b).
    Rebound { login: String, key: Hex },
    /// A new display name (E16 M3 T2b).
    Renamed { login: String, display: String },
    /// Whether the replays of the account's finished games may be listed (E16 M3 T2b).
    Listed { login: String, listed: bool },
}

impl Event {
    /// The login name of the account it is about.
    fn login(&self) -> &str {
        match self {
            Self::Registered { login, .. }
            | Self::Rekeyed { login, .. }
            | Self::Rebound { login, .. }
            | Self::Renamed { login, .. }
            | Self::Listed { login, .. } => login,
        }
    }
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
            let event = serde_json::from_str::<Event>(line).map_err(|e| journal::json(&e));
            event
                .and_then(|event| {
                    players.check(&event)?;
                    players.update(event);
                    Ok(())
                })
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

    /// Whether `event` follows from the accounts as they stand.
    fn check(&self, event: &Event) -> Result<(), String> {
        let known = self.accounts.contains_key(event.login());
        let fail = |why: &str| Err(String::from(why));
        match event {
            Event::Registered { login, display, .. } => {
                if !account::is_login_name(login) || !account::is_display_name(display) {
                    return fail("not a name");
                }
                if known || !self.display_free(display, login) {
                    return fail("a name registered before");
                }
            }
            Event::Renamed { login, display } => {
                if !account::is_display_name(display) {
                    return fail("not a name");
                }
                if !known {
                    return fail("no such account");
                }
                if !self.display_free(display, login) {
                    return fail("a name registered before");
                }
            }
            Event::Rekeyed { .. } | Event::Rebound { .. } | Event::Listed { .. } if !known => {
                return fail("no such account");
            }
            Event::Rekeyed { .. } | Event::Rebound { .. } | Event::Listed { .. } => {}
        }
        Ok(())
    }

    /// Applies a checked event to the tables.
    fn update(&mut self, event: Event) {
        match event {
            Event::Registered {
                login,
                key,
                display,
            } => {
                self.displays
                    .insert(account::display_key(&display), login.clone());
                let account = Account {
                    key: key.0,
                    display,
                    listed: true,
                };
                self.accounts.insert(login, account);
            }
            Event::Rekeyed { login, key } | Event::Rebound { login, key } => {
                if let Some(account) = self.accounts.get_mut(&login) {
                    account.key = key.0;
                }
            }
            Event::Renamed { login, display } => {
                if let Some(account) = self.accounts.get_mut(&login) {
                    self.displays
                        .remove(&account::display_key(&account.display));
                    self.displays
                        .insert(account::display_key(&display), login.clone());
                    account.display = display;
                }
            }
            Event::Listed { login, listed } => {
                if let Some(account) = self.accounts.get_mut(&login) {
                    account.listed = listed;
                }
            }
        }
    }

    /// The account of login name `login`.
    pub(crate) fn get(&self, login: &str) -> Option<&Account> {
        self.accounts.get(login)
    }

    /// Whether the display name `display` is free, ignoring case, or the account `login`'s.
    pub(crate) fn display_free(&self, display: &str, login: &str) -> bool {
        self.displays
            .get(&account::display_key(display))
            .is_none_or(|holder| holder == login)
    }

    /// Records `event`, which the caller checked against the accounts: its line is written and
    /// synced first; the account as it then stands.
    pub(crate) fn record(&mut self, event: Event) -> io::Result<Account> {
        self.check(&event).map_err(io::Error::other)?;
        self.append(&event)?;
        let login = event.login().to_owned();
        self.update(event);
        self.get(&login)
            .cloned()
            .ok_or_else(|| io::Error::other("no such account"))
    }

    /// Writes `event`'s line and syncs it; after a failure nothing more is written.
    fn append(&mut self, event: &Event) -> io::Result<()> {
        let file = self
            .file
            .as_mut()
            .ok_or_else(|| io::Error::other("a write failed before"))?;
        let mut text = serde_json::to_string(event).map_err(io::Error::other)?;
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

    fn registered(login: &str, key: u8, display: &str) -> Event {
        Event::Registered {
            login: login.to_owned(),
            key: Hex([key; KEY_BYTES]),
            display: display.to_owned(),
        }
    }

    #[test]
    fn the_accounts_are_read_back_and_a_torn_line_is_dropped() {
        let data = data();
        let mut players = Players::open(&data).unwrap();
        assert_eq!(players.get("alice"), None);
        let alice = players.record(registered("alice", 1, "Alice")).unwrap();
        players
            .record(registered("bob", 2, "Bob the Second"))
            .unwrap();
        assert!(!players.display_free("ALICE", "bob") && players.display_free("Carol", "bob"));
        assert!(players.display_free("ALICE", "alice"), "its own");
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
        players.record(registered("carol", 3, "Carol")).unwrap();
        drop(players);
        let players = Players::open(&data).unwrap();
        assert!(!players.display_free("carol", "alice"));
        fs::remove_dir_all(&data).unwrap();
    }

    #[test]
    fn re_keys_rebinds_renames_and_the_preference_are_read_back() {
        let data = data();
        let mut players = Players::open(&data).unwrap();
        players.record(registered("alice", 1, "Alice")).unwrap();
        players.record(registered("bob", 2, "Bob")).unwrap();
        let login = || String::from("alice");
        let rekeyed = players.record(Event::Rekeyed {
            login: login(),
            key: Hex([3; KEY_BYTES]),
        });
        assert_eq!(rekeyed.unwrap().key, [3; KEY_BYTES]);
        let rebound = players.record(Event::Rebound {
            login: login(),
            key: Hex([4; KEY_BYTES]),
        });
        assert_eq!(rebound.unwrap().key, [4; KEY_BYTES]);
        // Its own name in another case, then a new one: the old one is free again.
        for display in ["ALICE", "Alice Liddell"] {
            let renamed = players.record(Event::Renamed {
                login: login(),
                display: display.to_owned(),
            });
            assert_eq!(renamed.unwrap().display, display);
        }
        assert!(players.display_free("alice", "bob"));
        let listed = players.record(Event::Listed {
            login: login(),
            listed: false,
        });
        assert!(!listed.unwrap().listed);
        let taken = players.record(Event::Renamed {
            login: String::from("bob"),
            display: String::from("alice liddell"),
        });
        assert_eq!(taken.unwrap_err().to_string(), "a name registered before");
        let unknown = players.record(Event::Listed {
            login: String::from("carol"),
            listed: true,
        });
        assert_eq!(unknown.unwrap_err().to_string(), "no such account");
        let alice = players.get("alice").cloned();
        drop(players);
        let players = Players::open(&data).unwrap();
        assert_eq!(players.get("alice").cloned(), alice);
        assert_eq!(
            alice.map(|a| (a.key, a.display, a.listed)),
            Some(([4; KEY_BYTES], String::from("Alice Liddell"), false))
        );
        let text = fs::read_to_string(path(&data)).unwrap();
        assert_eq!(text.lines().count(), 7, "two refused, not written: {text}");
        assert!(text.contains("{\"Rebound\":{\"login\":\"alice\",\"key\":\"0404"));
        assert!(text.ends_with("{\"Listed\":{\"login\":\"alice\",\"listed\":false}}\n"));
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
            (alice.replace("\"01", "\"zz"), "line 1: Data"),
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
            (
                format!("{alice}{{\"Listed\":{{\"login\":\"bob\",\"listed\":true}}}}\n"),
                "line 2: no such account",
            ),
            (
                format!("{alice}{{\"Renamed\":{{\"login\":\"alice\",\"display\":\" A\"}}}}\n"),
                "line 2: not a name",
            ),
        ];
        for (text, expected) in cases {
            let data = data();
            journal::create_dir(&data).unwrap();
            fs::write(path(&data), &text).unwrap();
            let error = Players::open(&data).unwrap_err();
            assert!(error.contains(expected), "{expected}: {error}");
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
        assert!(players.record(registered("alice", 1, "Alice")).is_err());
        assert!(players.file.is_none() && players.get("alice").is_none());
        let again = players.record(registered("alice", 1, "Alice")).unwrap_err();
        assert_eq!(again.to_string(), "a write failed before");
        fs::remove_dir_all(&data).unwrap();
    }
}
