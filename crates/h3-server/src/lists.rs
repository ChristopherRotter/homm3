//! Lists of servers (E16 M6 T5; ADR-0021 point 5, `E16-design.md` §12): the signed lists an
//! operator places in `<data>/lists/`, each a `SignedList` as JSON (as `h3-server sign-list`
//! prints it) in a file whose name ends in `.json`. They are read at start and at each round of
//! fetches, in the files' order. A file that does not read, a list that does not hold
//! ([`h3_protocol::list::read`]: its own key's signature, its bounds, its times), one whose key an
//! earlier file's list has, and any beyond [`MAX_LISTS`] are skipped and logged. `Lists` gives the
//! lists that hold; the servers they name are fetched as the peers are (`crate::peers`), their
//! cards relayed marked with the list's key.
//!
//! The operator's tools: `h3-server list-key <file>` ([`list_key`]: a list's key, written there if
//! there is none; its public key) and `h3-server sign-list --key <file> <list.toml>`
//! ([`sign_list`]: the signed list of a TOML list of servers, from now for `valid_days`).
//!
//! Provisional (E16 M6 T5): the directory, the files' names and form, the TOML list's form and its
//! `valid_days` (1 to 366), the log lines.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use h3_protocol::account;
use h3_protocol::card::CardError;
use h3_protocol::codec;
use h3_protocol::connection::Signer as _;
use h3_protocol::list::{self, MAX_LIST_BYTES, MAX_LISTS, MAX_SERVERS};
use h3_protocol::wire::{ListedServer, ServerList, SignedList, WireU64};
use serde::Deserialize;

use crate::Error;
use crate::accounts::{Authenticator as _, Ed25519};
use crate::card::wall_ms;
use crate::key::ServerKey;

/// The lists' directory in the data directory.
pub(crate) const LISTS_DIR: &str = "lists";

/// The largest list file read: a list's text, escaped in its JSON (at most twice its bytes), and
/// its signature.
const MAX_FILE_BYTES: usize = 2 * MAX_LIST_BYTES + 1024;

/// A day in milliseconds.
const DAY_MS: u64 = 86_400_000;

/// The fewest and the most days a signed list holds (`valid_days`).
const VALID_DAYS: std::ops::RangeInclusive<u64> = 1..=366;

/// A list that held when it was read.
#[derive(Debug, Clone)]
pub(crate) struct Held {
    /// As its key signed it.
    pub(crate) signed: SignedList,
    /// The list.
    pub(crate) list: ServerList,
}

/// The list `signed` holds at `now` (milliseconds since the Unix epoch), its own key's Ed25519
/// signature checked ([`list::read`]).
pub(crate) fn verify(signed: &SignedList, now: u64) -> Result<ServerList, CardError> {
    list::read(signed, now, |key, message, signature| {
        Ed25519.verify(key, message, signature)
    })
}

/// The lists in `dir` that hold at `now`, in the order of their files' names; each file skipped is
/// logged with why. No directory, no list.
pub(crate) fn read(dir: &Path, now: u64) -> Vec<Held> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Vec::new(),
        Err(e) => {
            eprintln!("h3-server: lists {}: {e}", dir.display());
            return Vec::new();
        }
    };
    let mut paths: Vec<PathBuf> = (entries.flatten())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    // Each path once: the order is the names'.
    paths.sort();
    let mut held: Vec<Held> = Vec::new();
    for path in paths {
        let why = match one(&path, now) {
            Ok(_) if held.len() >= MAX_LISTS => format!("more than {MAX_LISTS} lists"),
            Ok(read) if held.iter().any(|h| h.list.key == read.list.key) => {
                String::from("an earlier file holds a list of its key")
            }
            Ok(read) => {
                held.push(read);
                continue;
            }
            Err(why) => why,
        };
        eprintln!("h3-server: list {}: {why}; skipped", path.display());
    }
    held
}

/// The list in the file `path` if it holds at `now`, or why not (never the file's text: an error
/// that quotes it could carry anything).
fn one(path: &Path, now: u64) -> Result<Held, String> {
    let size = fs::metadata(path).map_err(|e| e.to_string())?.len();
    if size > u64::try_from(MAX_FILE_BYTES).unwrap_or(u64::MAX) {
        return Err(format!("larger than {MAX_FILE_BYTES} bytes"));
    }
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let signed: SignedList =
        serde_json::from_str(&text).map_err(|_| String::from("not a signed list"))?;
    let list = verify(&signed, now).map_err(|why| format!("it does not hold ({why:?})"))?;
    Ok(Held { signed, list })
}

/// A list of servers as its maker writes it for `h3-server sign-list` (TOML).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Unsigned {
    /// The list's name.
    name: String,
    /// How many days the signed list holds, from its signing: 1 to 366.
    valid_days: u64,
    /// The servers it names: `[[server]] key = "…" address = "wss://…"`.
    #[serde(rename = "server", default)]
    servers: Vec<ListedServer>,
}

/// The public key of the list key in the file `key` (lowercase hex), written there first if there
/// is none (`h3-server list-key`).
pub fn list_key(key: &Path) -> Result<String, Error> {
    Ok(account::to_hex(&ServerKey::at(key, true)?.key()))
}

/// The signed list (`SignedList` as JSON, for a file in a server's `<data>/lists/`) of the TOML
/// list `list`, signed now by the list key in the file `key` (`h3-server sign-list`); why not, if
/// the key file does not read, the TOML is not a list or the list breaks its bounds.
pub fn sign_list(key: &Path, list: &Path) -> Result<String, Error> {
    let shown = |e: &dyn std::fmt::Display| Error(format!("{}: {e}", list.display()));
    let key = ServerKey::at(key, false)?;
    let text = fs::read_to_string(list).map_err(|e| shown(&e))?;
    let unsigned: Unsigned = toml::from_str(&text).map_err(|e| shown(&e))?;
    if !VALID_DAYS.contains(&unsigned.valid_days) {
        return Err(shown(&format!(
            "valid_days = {}: from {} to {}",
            unsigned.valid_days,
            VALID_DAYS.start(),
            VALID_DAYS.end()
        )));
    }
    let issued = wall_ms();
    let made = ServerList {
        key: account::to_hex(&key.key()),
        name: unsigned.name,
        issued: WireU64(issued),
        expires: WireU64(issued.saturating_add(unsigned.valid_days * DAY_MS)),
        servers: unsigned.servers,
    };
    if !list::fits(&made) {
        return Err(shown(&format!(
            "a list names itself in one line of 1 to 64 bytes and at most {MAX_SERVERS} servers, \
             each by a key of 64 lowercase hex digits and a ws:// or wss:// address, no key twice"
        )));
    }
    let signed = list::sign(&made, &key);
    if signed.list.len() > MAX_LIST_BYTES {
        return Err(shown(&format!("longer than {MAX_LIST_BYTES} bytes signed")));
    }
    Ok(codec::encode_text(&signed))
}
