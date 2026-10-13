//! The server's key (E16 M6 T1; ADR-0021 point 1, `E16-design.md` §12): an Ed25519 key, the
//! server's identity across its addresses, which signs its welcomes. Written once into the data
//! directory at the first start, from OS randomness, and read at every start. A key file that does
//! not read or parse stops the start: a new key is a new identity whose pins all break, so the
//! server never writes one over it. The file is the data directory's one secret (mode 0600 on
//! Unix); a client's pin of the key, not the file, is what an operator who loses it loses.
//!
//! A list's key (E16 M6 T5, `h3-server list-key`) is the same kind of key in a file of the
//! operator's choosing, which signs lists of servers (`crate::lists`).
//!
//! Provisional (E16 M6 T1): the file's name and form (the 32-byte seed as lowercase hex and a
//! newline, written whole: a `.part` file renamed).

use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use ed25519_dalek::{Signer as _, SigningKey};
use h3_protocol::account::{self, KEY_BYTES, SIGNATURE_BYTES};
use h3_protocol::connection::Signer;

use crate::Error;

/// The key file's name in the data directory.
pub(crate) const KEY_FILE: &str = "server.key";

/// The server's signing key.
pub(crate) struct ServerKey(SigningKey);

impl ServerKey {
    /// The key in the data directory `data`'s key file, written there first if there is none.
    pub(crate) fn open(data: &Path) -> Result<Self, Error> {
        Self::read(data, true)
    }

    /// The key in the data directory `data`'s key file, which must be there (E16 M6 T2: the
    /// operator's `h3-server card` names the running server by it).
    pub(crate) fn existing(data: &Path) -> Result<Self, Error> {
        Self::read(data, false)
    }

    /// The key in `data`'s key file, written there first if there is none and `create`.
    fn read(data: &Path, create: bool) -> Result<Self, Error> {
        Self::at(&data.join(KEY_FILE), create)
    }

    /// The key in the file `path`, written there first if there is none and `create` (E16 M6 T5:
    /// a list's key too). A file that does not parse is never replaced.
    pub(crate) fn at(path: &Path, create: bool) -> Result<Self, Error> {
        let failed = |e: &dyn fmt::Display| Error(format!("{}: {e}", path.display()));
        match fs::read_to_string(path) {
            Ok(text) => account::from_hex(text.trim())
                .map(Self::from_seed)
                .ok_or_else(|| {
                    failed(
                        &"not a key (64 hex digits); a new key would be a new identity, so it is \
                          not replaced",
                    )
                }),
            Err(e) if e.kind() == io::ErrorKind::NotFound && create => {
                let mut seed = [0u8; 32];
                getrandom::fill(&mut seed).map_err(|e| failed(&format!("OS randomness: {e}")))?;
                write_private(path, &format!("{}\n", account::to_hex(&seed)))
                    .map_err(|e| failed(&e))?;
                eprintln!("h3-server: a new key in {}", path.display());
                Ok(Self::from_seed(seed))
            }
            Err(e) => Err(failed(&e)),
        }
    }

    /// The key of `seed`.
    pub(crate) fn from_seed(seed: [u8; 32]) -> Self {
        Self(SigningKey::from_bytes(&seed))
    }
}

impl Signer for ServerKey {
    fn key(&self) -> [u8; KEY_BYTES] {
        self.0.verifying_key().to_bytes()
    }

    fn sign(&self, message: &[u8]) -> [u8; SIGNATURE_BYTES] {
        self.0.sign(message).to_bytes()
    }
}

/// Names the public key only.
impl fmt::Debug for ServerKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ServerKey({})", account::to_hex(&self.key()))
    }
}

/// Writes `text` to `path`, readable by its owner only, so that a crash leaves the whole file or
/// none: to a `.part` file beside it, synced, then renamed and its directory synced.
fn write_private(path: &Path, text: &str) -> io::Result<()> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(dir)?;
    let mut part = path.as_os_str().to_owned();
    part.push(".part");
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let mut file = options.open(&part)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()?;
    fs::rename(&part, path)?;
    #[cfg(unix)]
    fs::File::open(dir)?.sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn data(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("h3-key-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_key_is_written_once_and_read_back() {
        let dir = data("read-back");
        let key = ServerKey::open(&dir).unwrap().key();
        assert_eq!(
            ServerKey::open(&dir).unwrap().key(),
            key,
            "the same identity"
        );
        let text = fs::read_to_string(dir.join(KEY_FILE)).unwrap();
        assert_eq!(text.len(), 65, "{text:?}");
        assert!(!dir.join("server.key.part").exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(dir.join(KEY_FILE))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o600);
        }
        let shown = format!("{:?}", ServerKey::open(&dir).unwrap());
        assert_eq!(shown, format!("ServerKey({})", account::to_hex(&key)));
        assert!(!shown.contains(text.trim()), "the seed never shows");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_key_file_that_does_not_parse_is_never_replaced() {
        let dir = data("bad");
        for bad in ["", "not a key\n", &"0".repeat(63), &"0".repeat(66)] {
            fs::write(dir.join(KEY_FILE), bad).unwrap();
            let error = ServerKey::open(&dir).unwrap_err();
            assert!(error.0.contains("not a key"), "{bad:?}: {error}");
            assert_eq!(fs::read_to_string(dir.join(KEY_FILE)).unwrap(), bad);
        }
        // A directory where the file should be: it does not read.
        fs::remove_file(dir.join(KEY_FILE)).unwrap();
        fs::create_dir(dir.join(KEY_FILE)).unwrap();
        assert!(ServerKey::open(&dir).is_err());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_key_signs_as_ed25519() {
        let key = ServerKey::from_seed([3; 32]);
        let signature = key.sign(b"welcome");
        let accounts = crate::accounts::Ed25519;
        use crate::accounts::Authenticator as _;
        assert!(accounts.verify(&key.key(), b"welcome", &signature));
        assert!(!accounts.verify(&key.key(), b"welcone", &signature));
    }
}
