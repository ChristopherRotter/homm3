//! The accounts file (E17 M5 T7; `docs/lanes/client/NOTES.md`, "The live client", 2026-10-12):
//! each account's key as this device keeps it, derived once from the login name and the password
//! (the adapter's Argon2id) and never the password. A password change keeps the key so far as an
//! earlier one, due at each server the client knew then (its pins) until it re-keys the account
//! there at its next login (`Rekey`, [`super::Client::login`]).
//!
//! The file (*provisional*): a line per key, the login name, `key` or `old`, the key's seed as
//! lowercase hex (secret: the seed is the key) and, for an earlier key, the addresses of the
//! servers it is still due at; an account's `key` line before its `old` lines, the newest earlier
//! key first. Blank lines and lines starting with `#` are skipped.
//!
//! ```text
//! alice key a6d07dbd0bc53fa424145cbb87da7a85b49b8905044250b913940d4871c2ba69
//! alice old 0101010101010101010101010101010101010101010101010101010101010101 ws://127.0.0.1:7000
//! ```

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use h3_protocol::account;

/// The bytes of a key's seed (Ed25519's secret key).
pub const SEED_BYTES: usize = 32;

/// The accounts this device keeps keys of, by login name.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Accounts(Vec<Account>);

/// One account's keys.
#[derive(Clone, PartialEq, Eq)]
struct Account {
    login: String,
    seed: [u8; SEED_BYTES],
    /// Its earlier keys, the newest first, each with the servers it is still due at.
    old: Vec<([u8; SEED_BYTES], Vec<String>)>,
}

/// The logins and the servers due, never the seeds.
impl fmt::Debug for Accounts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut list = f.debug_list();
        for a in &self.0 {
            let due: Vec<&Vec<String>> = a.old.iter().map(|(_, due)| due).collect();
            list.entry(&(&a.login, due));
        }
        list.finish()
    }
}

impl Accounts {
    /// Reads an accounts file's `text`.
    ///
    /// # Errors
    /// The number (from 1) of the first line that is not a login name, its kind and a seed (an
    /// earlier key's with at least one address), or that names an account's key a second time,
    /// or an earlier key before the account's key.
    pub fn parse(text: &str) -> Result<Self, usize> {
        let mut accounts = Self::default();
        for (i, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut words = line.split_whitespace();
            let (Some(login), Some(kind), Some(seed)) = (words.next(), words.next(), words.next())
            else {
                return Err(i + 1);
            };
            let Some(seed) = account::from_hex::<SEED_BYTES>(seed) else {
                return Err(i + 1);
            };
            let due: Vec<String> = words.map(String::from).collect();
            let at = accounts.at(login);
            match (kind, at, due.is_empty()) {
                ("key", Err(at), true) if account::is_login_name(login) => {
                    accounts.0.insert(
                        at,
                        Account {
                            login: String::from(login),
                            seed,
                            old: Vec::new(),
                        },
                    );
                }
                ("old", Ok(at), false) => accounts.0[at].old.push((seed, due)),
                _ => return Err(i + 1),
            }
        }
        Ok(accounts)
    }

    /// The accounts file's text.
    pub fn text(&self) -> String {
        let mut text = String::new();
        for a in &self.0 {
            let lines = core::iter::once(("key", &a.seed, &[][..])).chain(
                a.old
                    .iter()
                    .map(|(seed, due)| ("old", seed, due.as_slice())),
            );
            for (kind, seed, due) in lines {
                for word in [a.login.as_str(), kind, &account::to_hex(seed)] {
                    text.push_str(word);
                    text.push(' ');
                }
                for address in due {
                    text.push_str(address);
                    text.push(' ');
                }
                text.pop();
                text.push('\n');
            }
        }
        text
    }

    /// Where `login`'s account stands among the accounts, or where it would.
    fn at(&self, login: &str) -> Result<usize, usize> {
        self.0.binary_search_by(|a| a.login.as_str().cmp(login))
    }

    /// The seed of `login`'s key.
    pub fn key(&self, login: &str) -> Option<[u8; SEED_BYTES]> {
        let at = self.at(login).ok()?;
        Some(self.0[at].seed)
    }

    /// The seed of `login`'s earlier key still due at the server at `address` (a password changed
    /// since: the account is re-keyed there at the next login).
    pub fn due(&self, login: &str, address: &str) -> Option<[u8; SEED_BYTES]> {
        let at = self.at(login).ok()?;
        let mut old = self.0[at].old.iter();
        old.find(|(_, due)| due.iter().any(|a| a == address))
            .map(|(seed, _)| *seed)
    }

    /// Keeps `seed` as `login`'s key: a new account's, or the key the account has now after it
    /// was re-keyed on another device (its earlier keys kept).
    pub fn set(&mut self, login: &str, seed: [u8; SEED_BYTES]) {
        match self.at(login) {
            Ok(at) => self.0[at].seed = seed,
            Err(at) => self.0.insert(
                at,
                Account {
                    login: String::from(login),
                    seed,
                    old: Vec::new(),
                },
            ),
        }
    }

    /// A password change of `login`'s account: `seed` becomes its key, and the key so far an
    /// earlier one due at the servers at `servers` (those an older key is still due at left to
    /// it: the re-key there goes from that one). Whether it changed: not for an account without a
    /// key here, nor for the same key.
    pub fn change(&mut self, login: &str, seed: [u8; SEED_BYTES], servers: &[&str]) -> bool {
        let Ok(at) = self.at(login) else {
            return false;
        };
        let a = &mut self.0[at];
        if a.seed == seed {
            return false;
        }
        let due: Vec<String> = (servers.iter())
            .filter(|s| !a.old.iter().any(|(_, due)| due.iter().any(|d| d == *s)))
            .map(|s| String::from(*s))
            .collect();
        let before = core::mem::replace(&mut a.seed, seed);
        if !due.is_empty() {
            a.old.insert(0, (before, due));
        }
        true
    }

    /// `login`'s earlier keys are due at the server at `address` no more (re-keyed there, or the
    /// server held none of them); an earlier key due nowhere is forgotten.
    pub fn done(&mut self, login: &str, address: &str) {
        let Ok(at) = self.at(login) else {
            return;
        };
        let old = &mut self.0[at].old;
        for (_, due) in old.iter_mut() {
            due.retain(|a| a != address);
        }
        old.retain(|(_, due)| !due.is_empty());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;
    use alloc::string::ToString;

    const A: [u8; SEED_BYTES] = [0xa1; SEED_BYTES];
    const B: [u8; SEED_BYTES] = [0xb2; SEED_BYTES];
    const C: [u8; SEED_BYTES] = [0xc3; SEED_BYTES];
    const X: &str = "ws://x:7000";
    const Y: &str = "wss://y";

    fn hex(seed: &[u8; SEED_BYTES]) -> String {
        account::to_hex(seed)
    }

    #[test]
    fn the_file_reads_and_writes_each_account_s_keys() {
        let text = format!(
            "# keys\n\nbob key {}\nalice key {}\n  alice old {} {X} {Y}\n",
            hex(&A),
            hex(&B),
            hex(&C)
        );
        let accounts = Accounts::parse(&text).unwrap();
        assert_eq!(accounts.key("alice"), Some(B));
        assert_eq!(accounts.key("bob"), Some(A));
        assert_eq!(accounts.key("carol"), None);
        assert_eq!(accounts.due("alice", Y), Some(C));
        assert_eq!(accounts.due("alice", "ws://z"), None);
        assert_eq!(accounts.due("bob", X), None);
        assert_eq!(accounts.due("carol", X), None);
        let written = format!(
            "alice key {}\nalice old {} {X} {Y}\nbob key {}\n",
            hex(&B),
            hex(&C),
            hex(&A)
        );
        assert_eq!(accounts.text(), written);
        assert_eq!(Accounts::parse(&written), Ok(accounts.clone()));
        // The seeds stay out of a debug print.
        let debug = format!("{accounts:?}");
        assert!(debug.contains("alice") && debug.contains(X), "{debug}");
        assert!(!debug.contains(&hex(&B)), "{debug}");
    }

    #[test]
    fn a_bad_line_is_an_error_with_its_number() {
        let key = format!("alice key {}", hex(&A));
        for (text, line) in [
            ("alice key".to_string(), 1),
            (format!("Alice key {}", hex(&A)), 1),
            (format!("alice key {} {X}", hex(&A)), 1),
            (format!("alice key {}0", hex(&A)), 1),
            (format!("alice old {} {X}", hex(&A)), 1),
            (format!("{key}\n{key}"), 2),
            (format!("{key}\nalice old {}", hex(&B)), 2),
            (format!("{key}\nalice new {} {X}", hex(&B)), 2),
        ] {
            assert_eq!(Accounts::parse(&text), Err(line), "{text}");
        }
    }

    #[test]
    fn a_password_change_leaves_the_key_so_far_due_at_the_servers_known() {
        let mut accounts = Accounts::default();
        assert!(!accounts.change("alice", B, &[X]), "no key here");
        accounts.set("alice", A);
        assert!(!accounts.change("alice", A, &[X]), "the same key");
        assert!(accounts.change("alice", B, &[X, Y]));
        assert_eq!(accounts.key("alice"), Some(B));
        assert_eq!(accounts.due("alice", X), Some(A));
        // Changed again before the re-key at Y: Y stays due from A, the re-key there goes to C.
        accounts.done("alice", X);
        assert!(accounts.change("alice", C, &[X, Y]));
        assert_eq!(accounts.due("alice", X), Some(B));
        assert_eq!(accounts.due("alice", Y), Some(A));
        // Re-keyed at both: forgotten.
        accounts.done("alice", X);
        accounts.done("alice", Y);
        accounts.done("bob", Y);
        assert_eq!(accounts.text(), format!("alice key {}\n", hex(&C)));
        // A change with every server due from an older key adds no earlier one.
        let mut accounts = Accounts::default();
        accounts.set("alice", A);
        accounts.change("alice", B, &[X]);
        assert!(accounts.change("alice", C, &[X]));
        assert_eq!(accounts.due("alice", X), Some(A));
        // The key set anew (re-keyed elsewhere) keeps the earlier ones.
        accounts.set("alice", B);
        assert_eq!(
            (accounts.key("alice"), accounts.due("alice", X)),
            (Some(B), Some(A))
        );
        // With no server known, the key so far is due nowhere.
        assert!(accounts.change("alice", C, &[]));
        assert_eq!(accounts.text().lines().count(), 2);
    }
}
