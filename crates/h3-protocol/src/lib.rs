//! The server's published language and session core (E16, ADR-0018): the wire messages, their
//! bounded codec and the mapping to and from the engine's types, and the sans-I/O session core
//! that hosts one game for its seats. Nothing here touches a socket, a clock or a file: the host
//! (`h3-server`) passes bytes and the time in and sends what comes out.
//!
//! - [`wire`]: the published language, every message in both directions (M1 T1);
//! - [`codec`]: one JSON document per frame, bounded and panic-free (M1 T1);
//! - [`map`]: the mapping to and from the battle's commands, events and observation, which
//!   projects everything outbound for its viewer (M1 T1);
//! - [`connection`]: a connection's own messages, the hello and the ping (M1 T2);
//! - [`session`]: the session core, one hosted battle: seats, tokens, the authoritative loop, the
//!   bot hook, the viewers' records and the journal (M1 T2);
//! - [`lobby`]: the server's offers and games, created within limits and listed (M2 T1);
//! - [`account`]: login and display names, keys as hex, the bytes a login signs (M3 T1);
//! - [`ladder`]: the ladders' queues, the matcher and a match's answers (M4 T2);
//! - [`card`]: the server's signed card, its bounds and its reading (M6 T2), the chain of vouching
//!   of the cards it relays (M6 T3);
//! - [`list`]: lists of servers signed by a list's key, their bounds and their reading (M6 T5).
//!
//! Logic crate in the adapter layer (ADR-0018 point 1): pure, deterministic, `no_std` + `alloc`
//! (ADR-0003, ADR-0005). The lint header below is required and checked by `cargo xtask check-arch`.
#![no_std]
#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::float_arithmetic,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::exit
)]
#![warn(missing_docs)]

extern crate alloc;
#[cfg(test)]
extern crate std;

pub mod account;
pub mod card;
pub mod codec;
pub mod connection;
pub mod ladder;
pub mod list;
pub mod lobby;
pub mod map;
pub mod session;
pub mod wire;

/// The version of the wire protocol: sent in the hello and bumped on every change of the messages
/// or their encoding (ADR-0018 point 4). Before 1.0 a server accepts exactly its own version.
/// v21 (R01, the content split): the manifest and the offers carry the battle part's hash beside
/// the core's. v22 (E17 M1, CLT-Q3): the observation names the natural obstacles' types, the key
/// tables list the obstacle types. v23: a two-hex stack's position (its hex, the hexes of moves
/// and attacks) is its rear hex, as in the original (BAT-GEO-002). v24 (E16 M6 T1, ADR-0021 point
/// 1): the hello names a nonce and the address the client dialed; the welcome carries the server's
/// key and its signature. v25 (E16 M6 T2, ADR-0021 point 2): `Describe` and `Described`, the
/// server's signed card. v26 (E16 M6 T3, ADR-0021 point 3): `ListPeers` and `Peers`, the cards a
/// server relays. v27 (E16 M6 T5, ADR-0021 point 5): `Lists` and `ListsAre`, the signed lists of
/// servers a server serves. v28 (E04c T2b, ADR-0022): the observation names the special
/// battlefield and the anti-magic garrison; the key tables list the battlefields. v29 (E17
/// CLT-Q1): a walker's `Moved` comes hex by hex, one per position of its path (BAT-MOV-003); the
/// message is unchanged, its meaning is new. v30 (E04c T5, ADR-0023): the observation names the
/// town the battle is fought at (`wire::Siege`); the events `GateOpened`, `GateClosed`,
/// `MoatDamage`. v31 (E04c T6, ADR-0023 point 6): the wall shots: the command `ShootWall`, the
/// events `WallShot`, `WallQuaked`, `WallDestroyed`, the rejections `CannotShootWalls` and
/// `NoSuchPart`, the town's wall parts, a hero's command of its Catapult. v32 (E04c T7d, ADR-0023
/// point 7): the observation names the game's difficulty and, per side, whether its player is
/// human. v33 (E04c T7e, ADR-0013 point 4): the own hero's Ballista names its Artillery level.
/// v34 (the server lane's inbox, 2026-10-19): the query `CreatureCastTargets`; a side's
/// `may_retreat` is as the viewer knows it (a town's Escape Tunnel shows to its defender only,
/// BAT-Q50 (4)): the field is unchanged, its meaning is new.
pub const PROTOCOL_VERSION: u32 = 34;

/// The version of the record's format (`wire::RecordHeader`, the `Record` message): bumped when
/// its layout changes.
pub const RECORD_FORMAT_VERSION: u32 = 2;

#[cfg(test)]
mod tests;
