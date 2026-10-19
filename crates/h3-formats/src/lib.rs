//! Adapter: bounded readers for the files of the user's original game installation
//! (`docs/formats/installation.md`).
//!
//! These readers exist for the harvesting tool (`h3-harvest`, E02), which turns the original
//! tables into the committed TOML packs once (OPEN-QUESTIONS Q13), and for the clients' asset
//! loading (`clients/`, ADR-0020; the LOD table alone, CLT-Q2). The engine itself never reads
//! original files at run time. The experiment maps' writer (`.h3m`) moved to `h3-map` with the
//! content split (R01a).
//!
//! Every reader treats its input as untrusted bytes: all offsets and sizes are checked before use,
//! allocations are bounded by the input size or an explicit limit, and no reader panics on any
//! input (property tests in `tests/robustness.rs`). No `unsafe`.

#![forbid(unsafe_code)]

pub mod exe;
pub mod lod;
pub mod text;

use std::fmt;

/// Errors of all readers in this crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
    /// The input ended before a structure was complete.
    Truncated {
        /// What was being read.
        what: &'static str,
        /// Byte offset where the structure starts.
        offset: usize,
    },
    /// A magic number or signature did not match.
    BadMagic(&'static str),
    /// A value is outside the range the format allows.
    OutOfRange {
        /// What was being read.
        what: &'static str,
        /// The offending value.
        value: u64,
    },
    /// Compressed data could not be inflated to the declared size.
    Inflate(String),
    /// A text table is malformed (e.g. an unterminated quoted field).
    Text {
        /// 1-based record number.
        record: usize,
        /// What is wrong.
        message: &'static str,
    },
    /// The executable is not a build whose static tables are known.
    UnknownBuild(String),
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated { what, offset } => write!(f, "truncated {what} at offset {offset}"),
            Self::BadMagic(what) => write!(f, "bad magic: not a {what}"),
            Self::OutOfRange { what, value } => write!(f, "{what} out of range: {value}"),
            Self::Inflate(msg) => write!(f, "cannot inflate entry: {msg}"),
            Self::Text { record, message } => write!(f, "text table record {record}: {message}"),
            Self::UnknownBuild(msg) => write!(f, "unknown executable build: {msg}"),
        }
    }
}

impl std::error::Error for FormatError {}

/// Little-endian reads with bounds checks.
pub(crate) mod le {
    use super::FormatError;

    /// Reads a `u32` at `offset`.
    pub(crate) fn u32_at(
        data: &[u8],
        offset: usize,
        what: &'static str,
    ) -> Result<u32, FormatError> {
        let bytes = offset
            .checked_add(4)
            .and_then(|end| data.get(offset..end))
            .ok_or(FormatError::Truncated { what, offset })?;
        let mut buf = [0u8; 4];
        buf.copy_from_slice(bytes);
        Ok(u32::from_le_bytes(buf))
    }

    /// Reads a `u16` at `offset`.
    pub(crate) fn u16_at(
        data: &[u8],
        offset: usize,
        what: &'static str,
    ) -> Result<u16, FormatError> {
        let bytes = offset
            .checked_add(2)
            .and_then(|end| data.get(offset..end))
            .ok_or(FormatError::Truncated { what, offset })?;
        let mut buf = [0u8; 2];
        buf.copy_from_slice(bytes);
        Ok(u16::from_le_bytes(buf))
    }

    /// Reads an `i32` at `offset`.
    pub(crate) fn i32_at(
        data: &[u8],
        offset: usize,
        what: &'static str,
    ) -> Result<i32, FormatError> {
        u32_at(data, offset, what).map(|v| i32::from_le_bytes(v.to_le_bytes()))
    }
}
