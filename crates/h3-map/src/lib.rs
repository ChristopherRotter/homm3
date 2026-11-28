//! Adapter: the original's `.h3m` maps (ADR-0015 point 4, E08). *Parse* reads a map file's bytes
//! into a raw model faithful to its version (RoE, AB, SoD), within configured input bounds, with
//! typed errors and no panics; *import* maps the raw model onto content keys and handles and
//! builds `h3_game::GameSetup`, drawing nothing; `h3-game` and the contexts then validate it as
//! if commands had built it. Map files are untrusted input (`docs/06-security.md`): parse,
//! import and validation are property-tested together on arbitrary bytes (`cargo-fuzz` later).
//! Format notes: `docs/formats/h3m.md`. The [`writer`] writes the SoD subset that experiments in
//! the original game need (moved here from `h3-formats` with the content split, R01a).
//!
//! E08 T3: the reader ([`read`], [`parse`] into [`model::RawMap`]); T5a and T6a: the [`import`]
//! into `h3_game::GameSetup`.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod cursor;
mod error;
mod gzip;
mod import;
mod limits;
pub mod model;
mod parse;
pub mod writer;

#[cfg(test)]
mod tests;

pub use error::ReadError;
pub use gzip::{crc32, gunzip};
pub use import::{ImportError, Unsupported, import, import_map};
pub use limits::Limits;
pub use model::RawMap;
pub use parse::parse;

/// Version of the import from a map file into the game's setup (ADR-0015, ADV-Q2): a replay names
/// its map by content hash and path plus this version, and verifying it re-imports the map. Bump
/// it with any change that alters an imported setup (`docs/08-work-process.md`, "Versions").
/// v2 (E08 T12a): object types and the piles' fixed amounts; guarded piles unsupported. v3 (E08
/// T13a): the setup's difficulty, normal (the map holds none). v4 (E08 T18b): the placed heroes'
/// formation and customised details; a hero with an experience of 1 000 or more unsupported. v5
/// (E08 T19a): each tile's coast flag. v6 (E08 T20a2a): the monsters with their data and the
/// special grounds with their tiles; a monster with an artifact unsupported. v7 (E08 T21a): the
/// piles' guards (guarded piles supported; a random stack among them unsupported). v8 (E08 T29):
/// the scenario's level limit and banned skills; a hero with an experience of 1 000 or more
/// supported (its levels at load are the game's). v9 (E08 T49a): a placed hero's worn artifacts
/// at the map's positions. v10 (E08 T49b): an equipped Titan's Thunder adds a Spell Book. v11
/// (E08 T54): each player's human flag from the map's player kinds, each object's scenery by its
/// class, a placed hero's sex. v12 (E08 T61a): increment 7's 31 classes typed with what the map
/// puts in them (a placed artifact, a scroll's spell, a Shrine's spell, a Witch Hut's mask, a
/// Scholar's gift, Pandora's Box's and an event's reward record and the event's flags), the
/// guards of artifacts, scrolls, boxes and events, a monster's treasure artifact (a monster with
/// one supported); a reward of more creature stacks than an army's slots unsupported (A-24). v13
/// (E08 T61b): the map's banned spells and artifacts into the game's setup (GAM-SET-003). v14
/// (E08 T72): the boats (class 8) as the map's boats, the sea's four pickups (29, 59, 82, 86), the
/// holes (124) and Favorable Winds (225) typed, the holes and the winds on their pictures' tiles;
/// a hero placed on water unsupported (D-44). v15 (E08 T83a): the mines (53), the abandoned mines
/// (53.7 and 220), the Lighthouses (42), the Shipyards (87), the Windmills (112), the Water Wheels
/// (109) and the Mystical Gardens (55) typed; an ownable object's owner, an abandoned mine's set
/// of resources; the map's timed events into the game's setup; an owner nobody plays, an
/// abandoned mine without a resource of the seven refused, an ownable object entered from water
/// unsupported. v16 (E08 T93b): the dwellings (17 and 20), the Refugee Camp (78) and the random
/// dwellings (216 to 218) typed, a dwelling's and a random dwelling's owner, a random dwelling's
/// faction set and level range, one tied to a town of the map a blocker; classes 18 and 19, a
/// dwelling's subclass past its table, a random dwelling's empty faction set, unknown town or bad
/// levels refused; the random dwellings ownable among a battle's surroundings. v17 (E09 T7a1):
/// the towns (77, 98), each player's start (its factions, choice, main town and random-hero flag)
/// and the hero pool's inputs (the allowed heroes, SoD's colour masks, the prisoners' types) into
/// the game's setup; a random hero (70) without its type and a main town generating a hero
/// supported; a placed hero right of a town's entrance moved onto it (TWN-SET-010), one on a
/// town's entrance supported; a main town that is no town's entrance and a random town's
/// alignment past the colours refused. v18 (E09 T8a1): the towns (77, 98) typed (the type `town`),
/// each with its owner (a colour nobody plays refused) and its garrison's formation. v19 (E08
/// T97b1): a dwelling or a random dwelling entered from water supported (the other ownable kinds
/// stay unsupported there); a random dwelling tied to a town of the map has its levels and owner
/// checked before it stays a blocker. v20 (E09 T11): a random dwelling tied to a town of the map
/// typed, holding its town (the last with its identifier) and its levels, which takes the town's
/// faction when the game is built. v21 (E08 T103b): the teleporters typed (the monoliths 43 to 45
/// by subclass, the channel; the Subterranean Gate 103 and the Whirlpool 111), a monolith subclass
/// past 7 refused (A-39); class 220 typed by the mines' subclasses as 53 (a crafted 220.0 a
/// Sawmill, 220.8 refused; every shipped one is 220.7). v22 (E08 T113b): the garrisons (33, 219)
/// typed by subclass, each with its owner (a colour nobody plays refused), its troops (a count of
/// 0 an empty slot) and its removable flag (true on an RoE map), a random stack in one refused for
/// good (`ImportError::RandomStack`), a subclass past 1 refused; the fifteen bonus visitables (11,
/// 14, 28, 30, 31, 38, 48, 49, 52, 56, 64, 92, 94, 96, 110) typed, a subclass past 0 refused; 219
/// ownable among a battle's surroundings. v23 (E09 T26a2): the setup's `roe`, the map's format
/// Restoration of Erathia's (`GameSetup::roe`: a creature's week names only RoE's town records'
/// creatures, TWN-GRO-002). v24 (E08 T123b): the creature banks typed, class 16 by subclass (the
/// seven land kinds, a subclass past 6 refused) and 85, 24, 84 and 25 each at subclass 0 (a
/// subclass past 0 refused), none with a body. v25 (E09 T37b): SoD's customised hero types into
/// the game's setup (`GameSetup::custom_types`: each entry that sets a field the game keeps, as a
/// placed hero's customisations; a type code past the table refused). v26 (E09 T39a): the map's
/// Tavern (class 95) and the Den of Thieves (97) typed, without a body. v27 (E08 T132b): the trades
/// and services typed at subclass 0, none with a body: the Altar of Sacrifice (2), the Black Market
/// (7), the Hill Fort (35), the Trading Post (99, and the snow one 221 as 99), the War Machine
/// Factory (106) and the Freelancer's Guild (213); a subclass past 0 refused. v28 (E09 T47a2): a
/// town's required and banned spells into its setup (`TownSetup::required_spells`,
/// `banned_spells`, TWN-SET-012: the spell of each set bit 0 to 69, bits 70 and 71 dropped; an RoE
/// map's required set empty).
/// v29 (E08 T141a, increment 15's design note §7): the Prisons (class 62) typed with their
/// prisoners' bodies read as placed heroes' (`GameSetup::prisoners`; a Prison entered from water
/// refused, `Unsupported::PrisonerOnWater`) and the Market of Time (class 50) typed.
/// v30 (E10 T3a, E10 design note §1): each player's team from the map's teams, else its own
/// (`PlayerSetup::team`, GAM-TEM-001).
pub const MAP_IMPORT_VERSION: u32 = 30;

/// Reads a map file as the game stores it (gzip-compressed) within `limits`.
///
/// # Errors
///
/// [`ReadError`]: a file over [`Limits::max_file_bytes`], a broken gzip stream, a map over
/// [`Limits::max_map_bytes`], or any error of [`parse`].
pub fn read(file: &[u8], limits: &Limits) -> Result<RawMap, ReadError> {
    if file.len() > limits.max_file_bytes {
        return Err(ReadError::FileTooLarge {
            size: file.len(),
            max: limits.max_file_bytes,
        });
    }
    parse(&gunzip(file, limits.max_map_bytes)?, limits)
}
