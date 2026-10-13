//! The server's card (E16 M6 T2; ADR-0021 point 2, `docs/epics/E16-design.md` §12): a server's
//! description of itself, signed by its key, which any client gets with `Describe` after its hello
//! and which servers relay for each other (T3). The card travels as its JSON text exactly as the
//! key signed it ([`SignedCard`]): a relayer passes the text on and never re-encodes it.
//!
//! This crate builds the text and the bytes the key signs, bounds the card, and reads a signed card
//! with the cryptography the host or the client passes in: the signature by the card's own key,
//! the card's form and bounds, its expiry against the reader's clock, and its issue time at most
//! [`SKEW_MS`] ahead of it. That the card's key is the one expected (the welcome's, a configured
//! peer's) is the reader's check.
//!
//! A server relays the cards of the servers it vouches for, its peers (E16 M6 T3; ADR-0021 point
//! 3), with `ListPeers`, in pages of [`PAGE`]; a client checks each card relayed as any card, and
//! the chain of vouching from the relaying server's own card to it ([`chain`]).
//!
//! Provisional (E16 M6 T2, T3): the bounds below and the error's grain.

use crate::account::{self, KEY_BYTES, SIGNATURE_BYTES};
use crate::codec;
use crate::connection::Signer;
use crate::wire::{CardContent, CardLadder, CardPeer, ServerCard, SignedCard};

/// The longest card text, in bytes.
pub const MAX_CARD_BYTES: usize = 8192;

/// The most addresses a card names.
pub const MAX_ADDRESSES: usize = 8;

/// The most peers a card names.
pub const MAX_PEERS: usize = 64;

/// The most rulesets a card names, and the most ladders.
pub const MAX_LISTED: usize = 16;

/// The most packs in a ruleset of a card.
pub const MAX_PACKS: usize = 16;

/// The longest name, id or version on a card, in bytes.
pub const MAX_TEXT: usize = 64;

/// The longest description, in bytes.
pub const MAX_DESCRIPTION: usize = 1024;

/// The longest operator contact, in bytes.
pub const MAX_OPERATOR: usize = 256;

/// How far a card's issue time may be ahead of its reader's clock (clock skew), in milliseconds.
pub const SKEW_MS: u64 = 600_000;

/// The most cards a page of `Peers` holds (E16 M6 T3).
pub const PAGE: usize = 50;

/// Why a signed card does not hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardError {
    /// Its text is longer than [`MAX_CARD_BYTES`].
    TooLarge,
    /// Its text is not a card, or breaks a bound ([`fits`]), or its signature is not 64 bytes of hex.
    Malformed,
    /// Its signature does not verify by its key over its text.
    Signature,
    /// It expired.
    Expired,
    /// It was issued more than [`SKEW_MS`] ahead of the reader's clock.
    Ahead,
}

/// Whether `url` is a WebSocket address: `ws://` or `wss://`, then 1 to 200 visible ASCII
/// characters (a server's configured addresses follow the same rule).
pub fn is_address(url: &str) -> bool {
    let rest = (url.strip_prefix("ws://"))
        .or_else(|| url.strip_prefix("wss://"))
        .unwrap_or_default();
    (1..=200).contains(&rest.len()) && rest.bytes().all(|b| b.is_ascii_graphic())
}

/// Whether `text` is at most `max` bytes without control characters (but `\n` where `lines`).
fn plain(text: &str, max: usize, lines: bool) -> bool {
    text.len() <= max
        && text
            .chars()
            .all(|c| !c.is_control() || (lines && c == '\n'))
}

/// A name, an id or a version: 1 to [`MAX_TEXT`] bytes, one line.
fn short(text: &str) -> bool {
    !text.is_empty() && plain(text, MAX_TEXT, false)
}

/// Whether `key` is a public key: 32 bytes as lowercase hex.
fn is_key(key: &str) -> bool {
    account::from_hex::<KEY_BYTES>(key).is_some()
}

/// Whether `card` keeps the card's bounds: its key and its peers' keys read; a name; at most
/// [`MAX_DESCRIPTION`] bytes of description (lines) and [`MAX_OPERATOR`] of operator contact (one
/// line); 1 to [`MAX_ADDRESSES`] addresses and at most [`MAX_PEERS`] peers, each address one
/// [`is_address`] takes; at most [`MAX_LISTED`] rulesets of at most [`MAX_PACKS`] packs and as
/// many ladders, their names, ids and versions short texts; issued before it expires. The text's
/// length is bounded apart ([`MAX_CARD_BYTES`]).
pub fn fits(card: &ServerCard) -> bool {
    let content = |c: &CardContent| {
        c.packs.len() <= MAX_PACKS
            && (c.packs.iter()).all(|p| short(&p.id) && short(&p.version))
            && short(&c.profile)
    };
    let ladder = |l: &CardLadder| short(&l.id) && short(&l.name);
    let peer = |p: &CardPeer| is_key(&p.key) && is_address(&p.address);
    is_key(&card.key)
        && short(&card.name)
        && plain(&card.description, MAX_DESCRIPTION, true)
        && (1..=MAX_ADDRESSES).contains(&card.addresses.len())
        && card.addresses.iter().all(|a| is_address(a))
        && short(&card.engine)
        && card.content.len() <= MAX_LISTED
        && card.content.iter().all(content)
        && card.ladders.len() <= MAX_LISTED
        && card.ladders.iter().all(ladder)
        && plain(&card.operator, MAX_OPERATOR, false)
        && card.issued.0 < card.expires.0
        && card.peers.len() <= MAX_PEERS
        && card.peers.iter().all(peer)
}

/// `card` signed by `signer`: its text and the signature over it ([`account::card_message`]). The
/// host keeps its card within the bounds ([`fits`], [`MAX_CARD_BYTES`]); this signs what it gets.
pub fn sign(card: &ServerCard, signer: &dyn Signer) -> SignedCard {
    let card = codec::encode_text(card);
    let signature = account::to_hex(&signer.sign(&account::card_message(&card)));
    SignedCard { card, signature }
}

/// The card `signed` holds at `now` (milliseconds since the Unix epoch, the reader's clock): its
/// text within [`MAX_CARD_BYTES`], a card within its bounds ([`fits`]), signed by its own key as
/// `verify` checks (the key, the bytes signed, the signature), not expired and issued at most
/// [`SKEW_MS`] ahead of `now`; or why not.
pub fn read(
    signed: &SignedCard,
    now: u64,
    verify: impl Fn(&[u8; KEY_BYTES], &[u8], &[u8; SIGNATURE_BYTES]) -> bool,
) -> Result<ServerCard, CardError> {
    if signed.card.len() > MAX_CARD_BYTES {
        return Err(CardError::TooLarge);
    }
    let card: ServerCard = serde_json::from_str(&signed.card).map_err(|_| CardError::Malformed)?;
    let key = account::from_hex(&card.key).filter(|_| fits(&card));
    let (Some(key), Some(signature)) = (key, account::from_hex(&signed.signature)) else {
        return Err(CardError::Malformed);
    };
    if !verify(&key, &account::card_message(&signed.card), &signature) {
        return Err(CardError::Signature);
    }
    if card.expires.0 <= now {
        return Err(CardError::Expired);
    }
    if card.issued.0 > now.saturating_add(SKEW_MS) {
        return Err(CardError::Ahead);
    }
    Ok(card)
}

/// Whether `cards` is a chain of vouching (E16 M6 T3; ADR-0021 point 3): each card names the next
/// one's key among its peers. A relayed card (`wire::RelayedCard`) holds when the relaying server's
/// card, the cards of the keys its `via` names in that order, and the card itself are one; the
/// reader checks each card as [`read`] does and that each of them is the one its key names.
pub fn chain(cards: &[&ServerCard]) -> bool {
    (cards.iter().zip(cards.iter().skip(1)))
        .all(|(voucher, card)| voucher.peers.iter().any(|p| p.key == card.key))
}
