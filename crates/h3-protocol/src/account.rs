//! Accounts and the server's key on the wire (E16 M3 T1, M6 T1, T2; ADR-0019 point 1, ADR-0021
//! points 1 and 2, `docs/epics/E16-design.md` §10, §12): the rules of a login name and of a display
//! name, keys and signatures as hex, and the bytes a login, a re-key, a welcome and a card sign. A client derives its
//! Ed25519 key from its login name and password; the host checks the signatures (`h3-server`'s
//! `Authenticator`) and signs its welcomes with its own key: this crate holds no cryptography.
//! Each signed message starts with its purpose and version, so no signature serves another.
//!
//! Provisional (E16 M3 T1, M6 T1, T2): a display name's rules (ASCII keeps lookalike letters off the
//! leaderboards), the signed bytes' layout.

use alloc::string::String;
use alloc::vec::Vec;

/// The bytes of a public key (Ed25519).
pub const KEY_BYTES: usize = 32;

/// The bytes of a signature (Ed25519).
pub const SIGNATURE_BYTES: usize = 64;

/// The bytes of a connection's challenge, which the host draws from the OS.
pub const CHALLENGE_BYTES: usize = 32;

/// The bytes of the nonce a client's hello names, which the welcome signs (E16 M6 T1).
pub const NONCE_BYTES: usize = 32;

/// What a login signs first: the purpose and its version.
const LOGIN: &[u8] = b"h3-login-v1";

/// What a re-key signs first.
const REKEY: &[u8] = b"h3-rekey-v1";

/// What a welcome signs first (E16 M6 T1).
const WELCOME: &[u8] = b"h3-welcome-v1";

/// What a server's card signs first (E16 M6 T2).
const CARD: &[u8] = b"h3-card-v1";

/// What a list of servers signs first (E16 M6 T5).
const LIST: &[u8] = b"h3-list-v1";

/// Whether `login` is a login name: 3 to 20 of `a`–`z`, `0`–`9`, `_` and `-`. A client lowercases
/// what its player typed: the key's derivation is salted with exactly this text, so the server
/// takes no other case.
pub fn is_login_name(login: &str) -> bool {
    (3..=20).contains(&login.len())
        && login
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

/// Whether `display` is a display name: 3 to 24 of ASCII letters, digits, `_`, `-`, `.` and
/// spaces, a space never first, last or next to another.
pub fn is_display_name(display: &str) -> bool {
    (3..=24).contains(&display.len())
        && display
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b' '))
        && !display.starts_with(' ')
        && !display.ends_with(' ')
        && !display.contains("  ")
}

/// A display name as its server compares it: unique ignoring case.
pub fn display_key(display: &str) -> String {
    display.to_ascii_lowercase()
}

/// `bytes` as lowercase hex.
pub fn to_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut hex = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        hex.push(char::from(DIGITS[usize::from(b >> 4)]));
        hex.push(char::from(DIGITS[usize::from(b & 0x0f)]));
    }
    hex
}

/// The `N` bytes `hex` spells in lowercase hex, or `None` for any other text.
pub fn from_hex<const N: usize>(hex: &str) -> Option<[u8; N]> {
    let digit = |c: u8| match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        _ => None,
    };
    let text = hex.as_bytes();
    if text.len() != N * 2 {
        return None;
    }
    let mut bytes = [0u8; N];
    for (byte, pair) in bytes.iter_mut().zip(text.chunks_exact(2)) {
        *byte = (digit(pair[0])? << 4) | digit(pair[1])?;
    }
    Some(bytes)
}

/// Appends `text` after its length (4 bytes, big-endian), so no two lists of texts sign alike.
fn push_text(out: &mut Vec<u8>, text: &str) {
    let len = u32::try_from(text.len()).unwrap_or(u32::MAX);
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(text.as_bytes());
}

/// The bytes a login signs: its purpose, the server's id (`id`, the address its clients dial),
/// the connection's challenge, the login name and the display name a registration asks for
/// (empty for a login). A signature over them is good on this connection of this server only.
pub fn login_message(
    id: &str,
    challenge: &[u8; CHALLENGE_BYTES],
    login: &str,
    display: Option<&str>,
) -> Vec<u8> {
    let mut out = Vec::from(LOGIN);
    push_text(&mut out, id);
    out.extend_from_slice(challenge);
    push_text(&mut out, login);
    push_text(&mut out, display.unwrap_or_default());
    out
}

/// The bytes both keys of a re-key sign (the account's key so far and `key`, the new one): its
/// purpose, the server's id, the connection's challenge, the login name and the new key.
pub fn rekey_message(
    id: &str,
    challenge: &[u8; CHALLENGE_BYTES],
    login: &str,
    key: &[u8; KEY_BYTES],
) -> Vec<u8> {
    let mut out = Vec::from(REKEY);
    push_text(&mut out, id);
    out.extend_from_slice(challenge);
    push_text(&mut out, login);
    out.extend_from_slice(key);
    out
}

/// The bytes the server's key signs in a welcome (E16 M6 T1, ADR-0021 point 1): its purpose, the
/// server's id on this connection (the address the client dialed, when it is one of the server's),
/// the client's nonce and the connection's challenge. A client that checks the signature knows the
/// key's holder answered its hello, at the address the id names.
pub fn welcome_message(
    id: &str,
    nonce: &[u8; NONCE_BYTES],
    challenge: &[u8; CHALLENGE_BYTES],
) -> Vec<u8> {
    let mut out = Vec::from(WELCOME);
    push_text(&mut out, id);
    out.extend_from_slice(nonce);
    out.extend_from_slice(challenge);
    out
}

/// The bytes the server's key signs in its card (E16 M6 T2, ADR-0021 point 2): its purpose and the
/// card's text (`crate::wire::SignedCard::card`), exactly as it travels.
pub fn card_message(text: &str) -> Vec<u8> {
    let mut out = Vec::from(CARD);
    push_text(&mut out, text);
    out
}

/// The bytes a list's key signs (E16 M6 T5, ADR-0021 point 5): its purpose and the list's text
/// (`crate::wire::SignedList::list`), exactly as it travels.
pub fn list_message(text: &str) -> Vec<u8> {
    let mut out = Vec::from(LIST);
    push_text(&mut out, text);
    out
}
