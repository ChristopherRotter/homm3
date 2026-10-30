//! `h3-view account` and the keys a connection logs in by (E17 M5 T7; design §11.4; the server
//! lane's asks in `docs/lanes/client/NOTES.md`, "The live client", 2026-10-12). The account's key
//! is derived from its login name and the password as every client derives it (Argon2id, then
//! the seed as Ed25519's secret key) once per device, and kept in the accounts file
//! (`h3_view::remote::Accounts`), never the password. A password change keeps the key so far
//! due at every server in the pins file; `connect` and `window --server` log in there by it at
//! their next login and re-key the account to the new one (`h3_view::remote::Client::login`). An
//! adapter: the derivation, the signatures, the files, standard input.

use std::io::{BufRead, IsTerminal as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use argon2::{Algorithm, Argon2, Params, Version};
use ed25519_dalek::{Signer as _, SigningKey};
use h3_protocol::account::{self, KEY_BYTES};
use h3_protocol::wire::ClientMessage;
use h3_view::remote::{Accounts, Client, Login, SEED_BYTES, Signer};

use crate::connect::{hex, read_pins};

/// The accounts file's default name, in the working directory (*provisional*, as the pins file's).
pub const ACCOUNTS: &str = "accounts.txt";

/// The first lines the accounts file is written with.
const HEADER: &str = "# h3-view's account keys (E17 M5 T7): a seed is its account's key, which \
                      logs in as the account.\n# Keep this file to yourself; never the password.\n";

/// The seed of the key of `login`'s account by `password`: Argon2id (64 MiB, 3 passes, 1 lane,
/// version 0x13) of the password, salted with `h3-account-v1:` and the login name, 32 bytes; the
/// server lane's derivation (`h3-server`'s test client), the same on every device.
///
/// # Errors
/// Argon2's, which these parameters never meet.
pub fn derive(login: &str, password: &str) -> Result<[u8; SEED_BYTES], String> {
    let params =
        Params::new(64 * 1024, 3, 1, Some(SEED_BYTES)).map_err(|e| format!("argon2: {e}"))?;
    let mut seed = [0; SEED_BYTES];
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into(
            password.as_bytes(),
            format!("h3-account-v1:{login}").as_bytes(),
            &mut seed,
        )
        .map_err(|e| format!("argon2: {e}"))?;
    Ok(seed)
}

/// The public key of the key of `seed`.
pub fn public(seed: &[u8; SEED_BYTES]) -> [u8; KEY_BYTES] {
    SigningKey::from_bytes(seed).verifying_key().to_bytes()
}

/// The login name the player typed, lowercased (the derivation is salted with exactly it).
///
/// # Errors
/// Not a login name: 3 to 20 of letters, digits, `_` and `-`.
pub fn login_name(typed: &str) -> Result<String, String> {
    let login = typed.to_ascii_lowercase();
    if account::is_login_name(&login) {
        Ok(login)
    } else {
        Err(format!(
            "`{typed}` is no login name: 3 to 20 of letters, digits, `_` and `-`"
        ))
    }
}

/// The command's options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// The account's login name, lowercased.
    pub login: String,
    /// A password change.
    pub change: bool,
    /// The accounts file.
    pub accounts: PathBuf,
    /// The pins file: the servers this client knows.
    pub pins: PathBuf,
}

impl Options {
    /// Parses `<login> [--change] [--accounts <file>] [--pins <file>]`.
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let (mut login, mut change, mut accounts, mut pins) = (None, false, None, None);
        let mut rest = args.iter();
        while let Some(arg) = rest.next() {
            let mut value = || {
                rest.next()
                    .map(PathBuf::from)
                    .ok_or_else(|| format!("account: `{arg}` needs a value"))
            };
            match arg.as_str() {
                "--change" => change = true,
                "--accounts" => accounts = Some(value()?),
                "--pins" => pins = Some(value()?),
                flag if flag.starts_with("--") => {
                    return Err(format!("account: unknown option `{flag}`"));
                }
                typed if login.is_none() => login = Some(login_name(typed)?),
                extra => return Err(format!("account: unexpected `{extra}`")),
            }
        }
        Ok(Self {
            login: login.ok_or("account: needs the login name")?,
            change,
            accounts: accounts.unwrap_or_else(|| PathBuf::from(ACCOUNTS)),
            pins: pins.unwrap_or_else(|| PathBuf::from("pins.txt")),
        })
    }
}

/// Runs the command, the passwords read from standard input.
pub fn run(options: &Options) -> ExitCode {
    let stdin = std::io::stdin();
    match keep(options, &mut stdin.lock()) {
        Ok(lines) => {
            for line in lines {
                println!("{line}");
            }
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("h3-view account: {message}");
            ExitCode::FAILURE
        }
    }
}

/// The command with the passwords read from `input` (*provisional*: a line each, shown as typed;
/// the user's choice, 2026-10-30): the key derived and kept in the accounts file, the account's
/// key replaced (a new device, or the account re-keyed on another one: its earlier keys kept);
/// with `change`, the current password (unless its key is kept here) and the new one, the key so
/// far due at every server pinned. What happened, a line each.
///
/// # Errors
/// A file, a password missing or unchanged.
pub fn keep(options: &Options, input: &mut impl BufRead) -> Result<Vec<String>, String> {
    let path = &options.accounts;
    let mut accounts = read_accounts(path)?;
    let login = &options.login;
    let mut lines = Vec::new();
    if options.change {
        let before = match accounts.key(login) {
            Some(seed) => seed,
            None => derive(login, &password(input, "current password")?)?,
        };
        let seed = derive(login, &password(input, "new password")?)?;
        if seed == before {
            return Err("the new password is the current one".to_owned());
        }
        let pins = read_pins(&options.pins)?;
        let servers: Vec<&str> = pins.addresses().collect();
        if accounts.key(login).is_none() {
            accounts.set(login, before);
        }
        accounts.change(login, seed, &servers);
        lines.push(format!("{login}'s new key {}", hex(&public(&seed))));
        for server in servers.iter().filter(|s| accounts.due(login, s).is_some()) {
            lines.push(format!("re-keyed at the next login at {server}"));
        }
    } else {
        let seed = derive(login, &password(input, "password")?)?;
        accounts.set(login, seed);
        lines.push(format!("{login}'s key {}", hex(&public(&seed))));
    }
    write_accounts(path, &accounts)?;
    lines.push(format!("kept in {}", path.display()));
    Ok(lines)
}

/// A password read from `input` after its prompt naming `what` on standard error when standard
/// input is a terminal: a line, its end of line left out.
fn password(input: &mut impl BufRead, what: &str) -> Result<String, String> {
    if std::io::stdin().is_terminal() {
        eprint!("{what}: ");
        let _ = std::io::stderr().flush();
    }
    let mut line = String::new();
    input
        .read_line(&mut line)
        .map_err(|e| format!("standard input: {e}"))?;
    let password = line.strip_suffix('\n').unwrap_or(&line);
    let password = password.strip_suffix('\r').unwrap_or(password);
    if password.is_empty() {
        return Err(format!("no {what}"));
    }
    Ok(password.to_owned())
}

/// The accounts file at `path`; none yet when it does not exist.
pub fn read_accounts(path: &Path) -> Result<Accounts, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => {
            Accounts::parse(&text).map_err(|line| format!("{}: line {line}", path.display()))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Accounts::default()),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

/// Writes `accounts` to `path` (*provisional*: readable by its owner alone where the system has
/// such modes, when the file is made).
fn write_accounts(path: &Path, accounts: &Accounts) -> Result<(), String> {
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let mut open = std::fs::OpenOptions::new();
    open.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut open, 0o600);
    let text = format!("{HEADER}{}", accounts.text());
    open.open(path)
        .and_then(|mut file| file.write_all(text.as_bytes()))
        .map_err(|e| format!("{}: {e}", path.display()))
}

/// The account a connection logs in to (E17 M5 T7): its login name, the display name a
/// registration asks for, and the accounts file that keeps its keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    /// The login name, lowercased.
    pub login: String,
    /// A registration's display name.
    pub register: Option<String>,
    /// The accounts file.
    pub accounts: PathBuf,
}

impl Account {
    /// The login of `client`, welcomed at `address`: by the account's key the file keeps, and by
    /// an earlier one still due there, then the re-key (the file read at each login: a re-key on
    /// another connection of this client counts).
    ///
    /// # Errors
    /// The file, no key of the account in it, or a login out of place.
    pub fn login(&self, client: &mut Client, address: &str) -> Result<ClientMessage, String> {
        let accounts = read_accounts(&self.accounts)?;
        let seed = accounts.key(&self.login).ok_or_else(|| {
            format!(
                "{}: no key of `{}` (`h3-view account {}` derives it)",
                self.accounts.display(),
                self.login,
                self.login
            )
        })?;
        let key = SigningKey::from_bytes(&seed);
        let old = (accounts.due(&self.login, address)).map(|seed| SigningKey::from_bytes(&seed));
        let login = Login {
            login: &self.login,
            key: key.verifying_key().to_bytes(),
            old: old.as_ref().map(|k| k.verifying_key().to_bytes()),
            display: self.register.as_deref(),
        };
        client
            .login(&login, |signer, message| {
                let by = match (signer, &old) {
                    (Signer::Old, Some(old)) => old,
                    _ => &key,
                };
                by.sign(message).to_bytes()
            })
            .ok_or_else(|| "a login out of place".to_owned())
    }

    /// The account's earlier keys are due at `address` no more (re-keyed there, or the server held
    /// none of them).
    ///
    /// # Errors
    /// The file.
    pub fn done(&self, address: &str) -> Result<(), String> {
        let mut accounts = read_accounts(&self.accounts)?;
        accounts.done(&self.login, address);
        write_accounts(&self.accounts, &accounts)
    }
}

#[cfg(test)]
mod tests;
