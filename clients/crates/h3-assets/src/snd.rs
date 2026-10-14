//! The sound archives (`Heroes3.snd`, `H3ab_ahd.snd`; E17 M2 T5b): a `u32` count, then per entry
//! 48 bytes: a 40-byte name field, the offset and the size of the entry's bytes in the archive.
//! The name field holds the stem, a NUL, then `WAV` and whatever the packer left there before
//! (`WAVVVWAV`, no NUL after it), so an entry is named by its stem only and looked up by it,
//! ignoring ASCII case. Every entry of the user's two archives is a WAV ([`crate::wav`]).

use alloc::string::String;
use alloc::vec::Vec;

use crate::Error;
use crate::read::{index, slice, u32_at};

/// The bytes of one directory entry.
const ENTRY_BYTES: usize = 48;
/// The bytes of an entry's name field.
const NAME_BYTES: usize = 40;

/// One entry of a sound archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SndEntry {
    /// The name's stem, as stored (the field up to its first NUL).
    pub name: String,
    /// Where its bytes start in the archive.
    pub offset: u32,
    /// How many bytes it has.
    pub size: u32,
}

/// A parsed sound archive: its directory over the archive's bytes.
#[derive(Debug, Clone)]
pub struct SndArchive<'a> {
    bytes: &'a [u8],
    entries: Vec<SndEntry>,
}

impl<'a> SndArchive<'a> {
    /// Parses the directory. Every entry's bytes must lie inside the archive.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, Error> {
        let count = index(u32_at(bytes, 0).ok_or(Error::Truncated)?);
        let table = slice(bytes, 4, count.saturating_mul(ENTRY_BYTES)).ok_or(Error::Truncated)?;
        let mut entries = Vec::with_capacity(count);
        for record in table.chunks_exact(ENTRY_BYTES) {
            let field = record.get(..NAME_BYTES).unwrap_or_default();
            let stem = field.split(|&b| b == 0).next().unwrap_or_default();
            let offset = u32_at(record, NAME_BYTES).unwrap_or_default();
            let size = u32_at(record, NAME_BYTES + 4).unwrap_or_default();
            slice(bytes, index(offset), index(size)).ok_or(Error::Truncated)?;
            entries.push(SndEntry {
                name: String::from_utf8_lossy(stem).into_owned(),
                offset,
                size,
            });
        }
        Ok(Self { bytes, entries })
    }

    /// The entries in directory order.
    #[must_use]
    pub fn entries(&self) -> &[SndEntry] {
        &self.entries
    }

    /// An entry's bytes (checked by [`SndArchive::parse`]; empty for an entry of another archive
    /// that does not fit this one).
    #[must_use]
    pub fn data(&self, entry: &SndEntry) -> &'a [u8] {
        slice(self.bytes, index(entry.offset), index(entry.size)).unwrap_or_default()
    }

    /// The bytes of the first entry named `name` (its stem), ignoring ASCII case.
    #[must_use]
    pub fn find(&self, name: &str) -> Option<&'a [u8]> {
        self.entries
            .iter()
            .find(|e| e.name.eq_ignore_ascii_case(name))
            .map(|e| self.data(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build;
    use proptest::prelude::*;

    #[test]
    fn entries_are_named_by_their_stem_and_found_ignoring_case() {
        let bytes = build::snd(&[("PIKEATTK", b"abc"), ("Azurmove", b""), ("x", b"12345")]);
        let archive = SndArchive::parse(&bytes).unwrap();
        let names: Vec<_> = archive.entries().iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["PIKEATTK", "Azurmove", "x"]);
        assert_eq!(archive.find("pikeattk"), Some(&b"abc"[..]));
        assert_eq!(archive.find("AZURMOVE"), Some(&b""[..]));
        assert_eq!(archive.find("X"), Some(&b"12345"[..]));
        assert_eq!(archive.find("pike"), None);
        assert_eq!(archive.entries()[2].size, 5);
    }

    #[test]
    fn the_name_field_ends_at_its_first_nul() {
        let mut bytes = build::snd(&[("ABC", b"z")]);
        // `ABC`, NUL, then `WAVVVWAV` and no NUL: the packer's leftovers.
        bytes[4 + 4..4 + 12].copy_from_slice(b"WAVVVWAV");
        let archive = SndArchive::parse(&bytes).unwrap();
        assert_eq!(archive.entries()[0].name, "ABC");
        // A name of all 40 bytes has no NUL at all.
        let mut full = build::snd(&[("A", b"")]);
        full[4..44].fill(b'Q');
        assert_eq!(
            SndArchive::parse(&full).unwrap().entries()[0].name.len(),
            40
        );
    }

    #[test]
    fn a_short_directory_or_an_entry_past_the_end_is_rejected() {
        assert_eq!(SndArchive::parse(&[1, 0]).err(), Some(Error::Truncated));
        assert_eq!(
            SndArchive::parse(&[1, 0, 0, 0]).err(),
            Some(Error::Truncated)
        );
        let mut bytes = build::snd(&[("A", b"abc")]);
        let size = 4 + NAME_BYTES + 4;
        bytes[size..size + 4].copy_from_slice(&4u32.to_le_bytes());
        assert_eq!(SndArchive::parse(&bytes).err(), Some(Error::Truncated));
        let huge = [0xFF, 0xFF, 0xFF, 0xFF];
        assert_eq!(SndArchive::parse(&huge).err(), Some(Error::Truncated));
        assert!(SndArchive::parse(&[0; 4]).unwrap().entries().is_empty());
    }

    #[test]
    fn an_entry_of_another_archive_reads_as_empty() {
        let big = build::snd(&[("A", &[7; 100])]);
        let small = build::snd(&[("B", b"")]);
        let big = SndArchive::parse(&big).unwrap();
        let small = SndArchive::parse(&small).unwrap();
        assert_eq!(small.data(&big.entries()[0]), b"");
    }

    proptest! {
        #[test]
        fn parse_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..300)) {
            let _ = SndArchive::parse(&bytes);
        }
    }
}
