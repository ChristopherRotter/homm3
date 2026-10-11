//! Accounts (E16 M3 T2a; ADR-0019 point 1; `E16-design.md` §10). A login is a signature by the
//! account's key over the bytes the hello gate names (`h3_protocol::account::login_message`: the
//! server's id, the connection's challenge, the names), so it holds on this connection of this
//! server only. The [`Authenticator`] checks signatures; the player log holds the accounts.
//!
//! - A new key with a free login name and a free display name registers; a known login name with
//!   its key logs in (a display name it names is ignored).
//! - A signature that does not hold, a known login name with another key, and an unknown one
//!   without a display name are refused alike (`BadLogin`): a refusal does not say which. A
//!   registration whose display name is taken, ignoring case, is refused `NameTaken`.
//! - A registration's line is synced before its answer: an account the server answered survives
//!   a crash. A log that cannot be written refuses registrations (`BadLogin`, the reason on
//!   standard error); the accounts already registered still log in.
//!
//! Provisional (E16 M3 T2a): the module layout; a display name sent with the login of a known
//! account is ignored.

use std::fmt;
use std::path::Path;
use std::sync::{Mutex, PoisonError};

use ed25519_dalek::{Signature, VerifyingKey};
use h3_protocol::account::{KEY_BYTES, SIGNATURE_BYTES};
use h3_protocol::connection::AccountRequest;
use h3_protocol::wire::Refusal;

use crate::Error;
use crate::players::{Account, Players};

/// What checks an account's signatures (ADR-0019 point 1: the one seam of the scheme).
pub(crate) trait Authenticator: Send + Sync + fmt::Debug {
    /// Whether `signature` is `key`'s over `message`.
    fn verify(
        &self,
        key: &[u8; KEY_BYTES],
        message: &[u8],
        signature: &[u8; SIGNATURE_BYTES],
    ) -> bool;
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

    /// Answers an account's request whose form the hello gate checked: the account as it stands,
    /// or why not. Blocks on the player log: call it off a connection's task.
    pub(crate) fn answer(&self, request: AccountRequest) -> Result<Account, Refusal> {
        match request {
            AccountRequest::Login {
                login,
                key,
                signature,
                display,
                signed,
                ..
            } => self.login(&login, &key, &signature, display.as_deref(), &signed),
            // stub: SRV-STUB-2 (re-keys, renames and the listing preference from E16 M3 T2b).
            AccountRequest::Rekey { .. }
            | AccountRequest::Rename { .. }
            | AccountRequest::SetListed { .. } => Err(Refusal::BadLogin),
        }
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
        let mut players = self.players.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(account) = players.get(login) {
            return if account.key == *key {
                Ok(account.clone())
            } else {
                Err(Refusal::BadLogin)
            };
        }
        let Some(display) = display else {
            return Err(Refusal::BadLogin);
        };
        if players.display_taken(display) {
            return Err(Refusal::NameTaken);
        }
        players.register(login, *key, display).map_err(|e| {
            eprintln!("h3-server: the player log: {e}; registrations are refused");
            Refusal::BadLogin
        })
    }
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
