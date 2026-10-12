//! Accounts (E16 M3 T2a, T2b; ADR-0019 point 1; `E16-design.md` §10). A login is a signature by
//! the account's key over the bytes the hello gate names (`h3_protocol::account::login_message`:
//! the server's id, the connection's challenge, the names), so it holds on this connection of this
//! server only. The [`Authenticator`] checks signatures; the player log holds the accounts.
//!
//! - A new key with a free login name and a free display name registers; a known login name with
//!   its key logs in (a display name it names is ignored).
//! - A signature that does not hold, a known login name with another key, and an unknown one
//!   without a display name are refused alike (`BadLogin`): a refusal does not say which. A
//!   registration whose display name is taken, ignoring case, is refused `NameTaken`.
//! - A logged-in account (E16 M3 T2b) re-keys with both keys' signatures over
//!   `h3_protocol::account::rekey_message` (else `BadLogin`): from then on only the new key logs
//!   in. A re-key that changed the key says so, and the account's other connections, logged in
//!   with the key it had, close (`net`, E16 M3 T3). It renames itself to a display name no other
//!   account has, ignoring case (else `NameTaken`), and sets its listing preference.
//! - Each change's line is synced before its answer: a change the server answered survives a
//!   crash; a request that changes nothing writes none. A log that cannot be written refuses the
//!   changes (`BadLogin`, the reason on standard error); the accounts as they stand still log in.
//!
//! Provisional (E16 M3 T2a, T2b): the module layout; a display name sent with the login of a known
//! account is ignored; a failed re-key does not count as a failed login.

use std::fmt;
use std::path::Path;
use std::sync::{Mutex, PoisonError};

use ed25519_dalek::{Signature, VerifyingKey};
use h3_protocol::account::{KEY_BYTES, SIGNATURE_BYTES};
use h3_protocol::connection::AccountRequest;
use h3_protocol::wire::Refusal;

use crate::Error;
use crate::players::{Account, Event, Hex, Players};

/// What checks an account's signatures (ADR-0019 point 1: the one seam of the scheme).
pub(crate) trait Authenticator: Send + Sync + fmt::Debug {
    /// Whether `signature` is `key`'s over `message`.
    fn verify(
        &self,
        key: &[u8; KEY_BYTES],
        message: &[u8],
        signature: &[u8; SIGNATURE_BYTES],
    ) -> bool;

    /// Whether a signature by `key` can hold at all (the operator's rebind checks the key it is
    /// given, E16 M3 T2b).
    fn accepts(&self, key: &[u8; KEY_BYTES]) -> bool;
}

/// Ed25519 (`ed25519-dalek`), strict: a key of small order or a signature that is not canonical
/// never holds, so no second signature of the same message passes for the first.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Ed25519;

impl Authenticator for Ed25519 {
    fn verify(
        &self,
        key: &[u8; KEY_BYTES],
        message: &[u8],
        signature: &[u8; SIGNATURE_BYTES],
    ) -> bool {
        VerifyingKey::from_bytes(key).is_ok_and(|key| {
            key.verify_strict(message, &Signature::from_bytes(signature))
                .is_ok()
        })
    }

    fn accepts(&self, key: &[u8; KEY_BYTES]) -> bool {
        VerifyingKey::from_bytes(key).is_ok_and(|key| !key.is_weak())
    }
}

/// The server's accounts: who checks the signatures and the player log.
#[derive(Debug)]
pub(crate) struct Accounts {
    authenticator: Box<dyn Authenticator>,
    players: Mutex<Players>,
}

impl Accounts {
    /// The accounts the data directory's player log holds, checked by `authenticator`.
    pub(crate) fn open(data: &Path, authenticator: Box<dyn Authenticator>) -> Result<Self, Error> {
        Ok(Self {
            authenticator,
            players: Mutex::new(Players::open(data).map_err(Error)?),
        })
    }

    /// Answers an account's request whose form the hello gate checked: the account as it stands
    /// and whether its key changed (a re-key), or why not. Blocks on the player log: call it off a
    /// connection's task.
    pub(crate) fn answer(&self, request: AccountRequest) -> Result<(Account, bool), Refusal> {
        let unchanged = |account| (account, false);
        match request {
            AccountRequest::Login {
                login,
                key,
                signature,
                display,
                signed,
                ..
            } => self
                .login(&login, &key, &signature, display.as_deref(), &signed)
                .map(unchanged),
            AccountRequest::Rekey {
                login,
                key,
                by_old,
                by_new,
                signed,
                ..
            } => self.rekey(login, &key, (&by_old, &by_new), &signed),
            AccountRequest::Rename { login, display, .. } => {
                self.rename(login, display).map(unchanged)
            }
            AccountRequest::SetListed { login, listed, .. } => {
                self.set_listed(login, listed).map(unchanged)
            }
        }
    }

    /// The account `login` as it stands, if the server knows it (E16 M3 T3: a game names its
    /// seats' accounts in the lobby and asks their listing preferences). Blocks on the player log:
    /// call it neither on a connection's task nor under the registry's lock.
    pub(crate) fn get(&self, login: &str) -> Option<Account> {
        self.players().get(login).cloned()
    }

    /// The display names of the accounts `logins` as they stand, under one lock (E16 M4 T4: a
    /// leaderboard's rows), each with whether it is a bot's (E16 M4 T5); an empty name for an
    /// account the server does not know (none: a ladder's accounts are the player log's). Blocks
    /// on the player log: call it neither on a connection's task nor under the registry's lock.
    pub(crate) fn displays<'a>(
        &self,
        logins: impl IntoIterator<Item = &'a str>,
    ) -> Vec<(String, bool)> {
        let players = self.players();
        (logins.into_iter())
            .map(|login| {
                (players.get(login))
                    .map_or_else(Default::default, |a| (a.display.clone(), a.is_bot()))
            })
            .collect()
    }

    /// The configuration's bots (E16 M4 T5), each a login name and a display name: an account the
    /// player log does not know is registered as a bot's, one whose display name changed renamed.
    /// At start, before the ladders read their logs; an account of a player's under the name, a
    /// display name another account has or a log that does not write stops the start.
    pub(crate) fn bots<'a>(
        &self,
        bots: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Result<(), Error> {
        let mut players = self.players();
        for (login, display) in bots {
            let fail = |why: &dyn fmt::Display| Err(Error(format!("bot `{login}`: {why}")));
            let event = match players.get(login) {
                Some(account) if !account.is_bot() => return fail(&"a player's account"),
                Some(account) if account.display == display => continue,
                Some(_) => Event::Renamed {
                    login: login.to_owned(),
                    display: display.to_owned(),
                },
                None => Event::Bot {
                    login: login.to_owned(),
                    display: display.to_owned(),
                },
            };
            if let Err(e) = players.record(event) {
                return fail(&e);
            }
        }
        Ok(())
    }

    /// The player log, locked.
    fn players(&self) -> std::sync::MutexGuard<'_, Players> {
        self.players.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// A login as `login` by `key`, or its registration as `display`.
    fn login(
        &self,
        login: &str,
        key: &[u8; KEY_BYTES],
        signature: &[u8; SIGNATURE_BYTES],
        display: Option<&str>,
        signed: &[u8],
    ) -> Result<Account, Refusal> {
        if !self.authenticator.verify(key, signed, signature) {
            return Err(Refusal::BadLogin);
        }
        let mut players = self.players();
        if let Some(account) = players.get(login) {
            return if account.key == Some(*key) {
                Ok(account.clone())
            } else {
                Err(Refusal::BadLogin)
            };
        }
        let Some(display) = display else {
            return Err(Refusal::BadLogin);
        };
        if !players.display_free(display, login) {
            return Err(Refusal::NameTaken);
        }
        record(
            &mut players,
            Event::Registered {
                login: login.to_owned(),
                key: Hex(*key),
                display: display.to_owned(),
            },
        )
    }

    /// The logged-in account `login` re-keys to `key`, signed by its key so far and by `key`;
    /// whether the key changed.
    fn rekey(
        &self,
        login: String,
        key: &[u8; KEY_BYTES],
        (by_old, by_new): (&[u8; SIGNATURE_BYTES], &[u8; SIGNATURE_BYTES]),
        signed: &[u8],
    ) -> Result<(Account, bool), Refusal> {
        if !self.authenticator.verify(key, signed, by_new) {
            return Err(Refusal::BadLogin);
        }
        let mut players = self.players();
        let account = players.get(&login).cloned().ok_or(Refusal::BadLogin)?;
        // A bot's account has no key (E16 M4 T5), and none logs in as it.
        let old = account.key.ok_or(Refusal::BadLogin)?;
        if !self.authenticator.verify(&old, signed, by_old) {
            return Err(Refusal::BadLogin);
        }
        if old == *key {
            return Ok((account, false));
        }
        let key = Hex(*key);
        record(&mut players, Event::Rekeyed { login, key }).map(|account| (account, true))
    }

    /// The logged-in account `login` takes the display name `display`.
    fn rename(&self, login: String, display: String) -> Result<Account, Refusal> {
        let mut players = self.players();
        let account = players.get(&login).cloned().ok_or(Refusal::BadLogin)?;
        if !players.display_free(&display, &login) {
            return Err(Refusal::NameTaken);
        }
        if account.display == display {
            return Ok(account);
        }
        record(&mut players, Event::Renamed { login, display })
    }

    /// The logged-in account `login` sets whether its finished games' replays may be listed.
    fn set_listed(&self, login: String, listed: bool) -> Result<Account, Refusal> {
        let mut players = self.players();
        let account = players.get(&login).cloned().ok_or(Refusal::BadLogin)?;
        if account.listed == listed {
            return Ok(account);
        }
        record(&mut players, Event::Listed { login, listed })
    }
}

/// Records `event` in the player log; a log that cannot be written refuses it.
fn record(players: &mut Players, event: Event) -> Result<Account, Refusal> {
    players.record(event).map_err(|e| {
        eprintln!("h3-server: the player log: {e}; the change is refused");
        Refusal::BadLogin
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::Key;

    #[test]
    fn ed25519_checks_strictly() {
        let key = Key::from_seed([7; 32]);
        let signature = key.sign(b"login");
        assert!(Ed25519.verify(&key.public(), b"login", &signature));
        assert!(!Ed25519.verify(&key.public(), b"logon", &signature));
        assert!(!Ed25519.verify(&Key::from_seed([8; 32]).public(), b"login", &signature));
        // The identity point as the key, and the identity with s = 0 as the signature: a check
        // that is not strict accepts this for any message.
        let mut identity = [0u8; KEY_BYTES];
        identity[0] = 1;
        let mut forged = [0u8; SIGNATURE_BYTES];
        forged[0] = 1;
        assert!(!Ed25519.verify(&identity, b"login", &forged));
        assert!(
            !Ed25519.verify(&[0xff; KEY_BYTES], b"login", &signature),
            "not a point"
        );
    }
}
