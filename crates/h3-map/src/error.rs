//! Typed errors of reading a map file. Every failure of the reader is one of these; the reader
//! never panics, whatever the bytes (`docs/06-security.md`, "Untrusted files").

use core::fmt;

/// Why a map file could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadError {
    /// The file is larger than [`crate::Limits::max_file_bytes`].
    FileTooLarge {
        /// The file's size in bytes.
        size: usize,
        /// The limit.
        max: usize,
    },
    /// The file is not a gzip stream as the game writes them (RFC 1952: magic bytes, deflate,
    /// no reserved flags), or its deflate data is corrupt.
    NotGzip,
    /// The gzip stream inflates to more than [`crate::Limits::max_map_bytes`].
    MapTooLarge {
        /// The limit.
        max: usize,
    },
    /// The gzip trailer disagrees with the inflated data (its CRC-32 or its length).
    ChecksumMismatch,
    /// The map's format version is not RoE (`0x0E`), AB (`0x15`) or SoD (`0x1C`).
    UnsupportedVersion(u32),
    /// The data ends inside `field` (or a count says more records follow than the bytes left can
    /// hold).
    Truncated {
        /// The field being read.
        field: &'static str,
        /// Byte offset in the inflated map where it starts.
        offset: usize,
    },
    /// A size, count or string length exceeds its configured limit ([`crate::Limits`]).
    LimitExceeded {
        /// The field being read.
        field: &'static str,
        /// The value in the file.
        value: u64,
        /// The limit.
        max: u64,
        /// Byte offset in the inflated map.
        offset: usize,
    },
    /// A value that decides the layout of what follows (a condition's kind) has no meaning in
    /// this format version.
    InvalidValue {
        /// The field being read.
        field: &'static str,
        /// The value in the file.
        value: u64,
        /// Byte offset in the inflated map.
        offset: usize,
    },
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileTooLarge { size, max } => {
                write!(f, "map file of {size} bytes exceeds the limit of {max}")
            }
            Self::NotGzip => f.write_str("not a gzip-compressed map file"),
            Self::MapTooLarge { max } => write!(f, "map inflates to more than {max} bytes"),
            Self::ChecksumMismatch => f.write_str("gzip checksum or length mismatch"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported map format version {v:#x}"),
            Self::Truncated { field, offset } => {
                write!(f, "map ends inside {field} (offset {offset})")
            }
            Self::LimitExceeded {
                field,
                value,
                max,
                offset,
            } => write!(
                f,
                "{field} = {value} exceeds the limit {max} (offset {offset})"
            ),
            Self::InvalidValue {
                field,
                value,
                offset,
            } => write!(f, "invalid {field} {value} (offset {offset})"),
        }
    }
}

impl std::error::Error for ReadError {}
