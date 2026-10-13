//! Lists of servers (E16 M6 T5; ADR-0021 point 5, `docs/epics/E16-design.md` §12): servers named
//! by key and address, signed by the list's key. Anyone may make one (a community's, a
//! tournament's), valid until its own expiry; a server serves the lists its operator placed in its
//! data directory (`Lists`), fetches the servers they name as it fetches its peers and relays
//! their cards marked with the list's key; a client chooses which list keys to trust. A list
//! travels as its JSON text exactly as its key signed it ([`SignedList`]).
//!
//! This crate builds the text and the bytes the key signs, bounds the list, and reads a signed
//! list with the cryptography the reader passes in, as [`crate::card`] reads a card: the
//! signature by the list's own key, its form and bounds, its expiry against the reader's clock,
//! its issue time at most [`crate::card::SKEW_MS`] ahead of it. A card relayed for a list holds for the list
//! when the list names the card's key at one of the card's addresses ([`names`]).
//!
//! Provisional (E16 M6 T5): the bounds below.

use alloc::vec::Vec;

use crate::account::{KEY_BYTES, SIGNATURE_BYTES};
use crate::card::{CardError, Signed, is_address, is_key, read_signed, short};
use crate::codec;
use crate::connection::Signer;
use crate::wire::{ServerCard, ServerList, SignedList};

/// The longest list text, in bytes.
pub const MAX_LIST_BYTES: usize = 32 * 1024;

/// The most servers a list names.
pub const MAX_SERVERS: usize = 128;

/// The most lists a server serves (`ListsAre`).
pub const MAX_LISTS: usize = 16;

/// Whether `list` keeps a list's bounds: its key reads; a name; issued before it expires; at most
/// [`MAX_SERVERS`] servers, each with a key that reads and an address [`is_address`] takes, no key
/// twice. The text's length is bounded apart ([`MAX_LIST_BYTES`]).
pub fn fits(list: &ServerList) -> bool {
    let servers = &list.servers;
    is_key(&list.key)
        && short(&list.name)
        && list.issued.0 < list.expires.0
        && servers.len() <= MAX_SERVERS
        && (servers.iter()).all(|server| is_key(&server.key) && is_address(&server.address))
        && (servers.iter().enumerate())
            .all(|(i, server)| servers[..i].iter().all(|before| before.key != server.key))
}

impl Signed for ServerList {
    const MAX_BYTES: usize = MAX_LIST_BYTES;

    fn message(text: &str) -> Vec<u8> {
        crate::account::list_message(text)
    }

    fn key_if_fits(&self) -> Option<&str> {
        fits(self).then_some(self.key.as_str())
    }

    fn times(&self) -> (u64, u64) {
        (self.issued.0, self.expires.0)
    }
}

/// `list` signed by `signer`: its text and the signature over it ([`crate::account::list_message`]).
/// The maker keeps the list within its bounds ([`fits`], [`MAX_LIST_BYTES`]); this signs what it
/// gets.
pub fn sign(list: &ServerList, signer: &dyn Signer) -> SignedList {
    let list = codec::encode_text(list);
    let message = crate::account::list_message(&list);
    let signature = crate::account::to_hex(&signer.sign(&message));
    SignedList { list, signature }
}

/// The list `signed` holds at `now` (milliseconds since the Unix epoch, the reader's clock): its
/// text within [`MAX_LIST_BYTES`], a list within its bounds ([`fits`]), signed by its own key as
/// `verify` checks, not expired and issued at most [`crate::card::SKEW_MS`] ahead of `now`; or why
/// not.
pub fn read(
    signed: &SignedList,
    now: u64,
    verify: impl Fn(&[u8; KEY_BYTES], &[u8], &[u8; SIGNATURE_BYTES]) -> bool,
) -> Result<ServerList, CardError> {
    read_signed((&signed.list, &signed.signature), now, verify)
}

/// Whether `list` names the server of `card`: its key, at one of the card's addresses (a client's
/// check of a card relayed for the list, each read as it holds).
pub fn names(list: &ServerList, card: &ServerCard) -> bool {
    (list.servers.iter())
        .any(|server| server.key == card.key && card.addresses.contains(&server.address))
}
