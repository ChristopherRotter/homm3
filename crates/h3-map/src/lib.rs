//! Adapter: the original's `.h3m` maps (ADR-0015 point 4, E08). *Parse* reads a map file's bytes
//! into a raw model faithful to its version (RoE, AB, SoD), within configured input bounds, with
//! typed errors and no panics; *import* maps the raw model onto content keys and handles and
//! builds `h3_game::GameSetup`, drawing nothing; `h3-game` and the contexts then validate it as
//! if commands had built it. Map files are untrusted input (`docs/06-security.md`): parse,
//! import and validation are property-tested together on arbitrary bytes (`cargo-fuzz` later).
//! Format notes: `docs/formats/h3m.md`. The `.h3m` writer moves here from `h3-formats` after E04b.
//!
//! Skeleton (E08 T2): the reader and the import arrive in T3.

/// Version of the import from a map file into the game's setup (ADR-0015, ADV-Q2): a replay names
/// its map by content hash and path plus this version, and verifying it re-imports the map. Bump
/// it with any change that alters an imported setup (`docs/08-work-process.md`, "Versions").
pub const MAP_IMPORT_VERSION: u32 = 1;
