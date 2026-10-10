//! Input bounds of the reader (ADR-0015 point 4, `docs/06-security.md`): a map file above a limit
//! is rejected with [`crate::ReadError::LimitExceeded`] before anything is allocated for it.
//! The defaults admit the largest map the supported versions allow (144×144, two levels, eight
//! players) and every count and string of the maps shipped with the game, with a margin; the
//! numbers observed in the shipped maps are in `docs/formats/h3m.md`.

/// Configured input bounds. Counts are checked twice: against their limit here, and against the
/// bytes left in the file (a count whose records cannot fit is [`crate::ReadError::Truncated`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Limits {
    /// Largest compressed map file, in bytes.
    pub max_file_bytes: usize,
    /// Largest inflated map, in bytes.
    pub max_map_bytes: usize,
    /// Largest map side in tiles (maps are square).
    pub max_size: u32,
    /// Longest string (name, description, message), in bytes.
    pub max_string: u32,
    /// Most heroes listed for one player (AB and SoD).
    pub max_player_heroes: u32,
    /// Most campaign hero placeholders (AB and SoD).
    pub max_placeholders: u32,
    /// Most rumors.
    pub max_rumors: u32,
    /// Most secondary skills of one hero.
    pub max_secondary_skills: u32,
    /// Most artifacts in one hero's backpack.
    pub max_backpack: u32,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_file_bytes: 4 << 20,
            max_map_bytes: 16 << 20,
            max_size: 144,
            max_string: 1 << 16,
            max_player_heroes: 64,
            max_placeholders: 156,
            max_rumors: 256,
            max_secondary_skills: 28,
            max_backpack: 256,
        }
    }
}
