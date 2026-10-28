//! The video archives (`VIDEO.VID`, `H3ab_ahd.vid`; E17 M5 T4b2a): a `u32` count, then per entry
//! 44 bytes: a 40-byte name field (the file's name with its extension, `WIN3.BIK`,
//! `PGTRNLFT.SMK`, then a NUL) and the offset of its bytes. No size is stored: an entry runs to
//! the next entry's offset in the archive (the directory sorted by offset; the user's archives
//! list them in ascending order) or to the archive's end. The entries are Bink and Smacker
//! files; `h3-bink` decodes the Bink ones.

use alloc::string::String;
use alloc::vec::Vec;

use crate::Error;
use crate::read::{index, slice, u32_at};

/// The bytes of one directory entry.
const ENTRY_BYTES: usize = 44;
/// The bytes of an entry's name field.
const NAME_BYTES: usize = 40;

/// One entry of a video archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VidEntry {
    /// The name with its extension, as stored (the field up to its first NUL).
    pub name: String,
    /// Where its bytes start in the archive.
    pub offset: u32,
    /// How many bytes it has: up to the next entry's offset or the archive's end.
    pub size: u32,
}

/// A parsed video archive: its directory over the archive's bytes.
#[derive(Debug, Clone)]
pub struct VidArchive<'a> {
    bytes: &'a [u8],
    entries: Vec<VidEntry>,
}

/// How many bytes an archive's directory takes, its count included, by the count its first four
/// bytes give (none when they are missing): what to read of a large archive before
/// [`directory`] (E17 M5 T4b2b: `VIDEO.VID` is 629 MB).
#[must_use]
pub fn directory_bytes(head: &[u8]) -> Option<usize> {
    let count = u64::from(u32_at(head, 0)?);
    let entry = u64::try_from(ENTRY_BYTES).unwrap_or(u64::MAX);
    usize::try_from(count * entry + 4).ok()
}

/// The directory of an archive of `end` bytes from its first bytes `head` (at least
/// [`directory_bytes`] of them; E17 M5 T4b2b): the entries [`VidArchive::parse`] finds, without
/// the rest of the archive. Every entry's offset must lie inside the archive (at its end, the
/// entry is empty).
pub fn directory(head: &[u8], end: u32) -> Result<Vec<VidEntry>, Error> {
    let count = index(u32_at(head, 0).ok_or(Error::Truncated)?);
    let table = slice(head, 4, count.saturating_mul(ENTRY_BYTES)).ok_or(Error::Truncated)?;
    let mut fields = Vec::with_capacity(count);
    for record in table.chunks_exact(ENTRY_BYTES) {
        let field = record.get(..NAME_BYTES).unwrap_or_default();
        let name = field.split(|&b| b == 0).next().unwrap_or_default();
        let offset = u32_at(record, NAME_BYTES).unwrap_or_default();
        if offset > end {
            return Err(Error::Truncated);
        }
        fields.push((String::from_utf8_lossy(name).into_owned(), offset));
    }
    // The offsets in order, each entry ending where the next larger one starts.
    let mut starts: Vec<u32> = fields.iter().map(|&(_, offset)| offset).collect();
    starts.sort_unstable();
    let entries = fields
        .into_iter()
        .map(|(name, offset)| {
            let next = starts.partition_point(|&s| s <= offset);
            let until = starts.get(next).copied().unwrap_or(end);
            VidEntry {
                name,
                offset,
                size: until - offset,
            }
        })
        .collect();
    Ok(entries)
}

impl<'a> VidArchive<'a> {
    /// Parses the directory ([`directory`] over the whole archive).
    pub fn parse(bytes: &'a [u8]) -> Result<Self, Error> {
        let len = u32::try_from(bytes.len()).unwrap_or(u32::MAX);
        let entries = directory(bytes, len)?;
        Ok(Self { bytes, entries })
    }

    /// The entries in directory order.
    #[must_use]
    pub fn entries(&self) -> &[VidEntry] {
        &self.entries
    }

    /// An entry's bytes (empty for an entry of another archive that does not fit this one).
    #[must_use]
    pub fn data(&self, entry: &VidEntry) -> &'a [u8] {
        slice(self.bytes, index(entry.offset), index(entry.size)).unwrap_or_default()
    }

    /// The bytes of the first entry named `name` (with its extension), ignoring ASCII case.
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
    fn the_directory_reads_from_the_head_alone() {
        let bytes = build::vid(&[("WIN3.BIK", b"abc"), ("x.smk", b"12")]);
        let head = directory_bytes(&bytes).unwrap();
        assert_eq!(head, 4 + 2 * 44);
        let len = u32::try_from(bytes.len()).unwrap();
        let entries = directory(&bytes[..head], len).unwrap();
        assert_eq!(entries, VidArchive::parse(&bytes).unwrap().entries());
        assert_eq!(directory_bytes(&[1, 0]), None);
        assert_eq!(directory_bytes(&[0xFF; 4]), Some(4 + 0xFFFF_FFFF * 44));
        assert_eq!(directory(&bytes[..head - 1], len), Err(Error::Truncated));
    }

    #[test]
    fn entries_run_to_the_next_offset_and_are_found_ignoring_case() {
        let bytes = build::vid(&[
            ("WIN3.BIK", b"abc"),
            ("surrender.bik", b"s"),
            ("x.smk", b"12"),
        ]);
        let archive = VidArchive::parse(&bytes).unwrap();
        let names: Vec<_> = archive.entries().iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["WIN3.BIK", "surrender.bik", "x.smk"]);
        assert_eq!(archive.find("win3.bik"), Some(&b"abc"[..]));
        assert_eq!(archive.find("SURRENDER.BIK"), Some(&b"s"[..]));
        assert_eq!(archive.find("X.SMK"), Some(&b"12"[..]));
        assert_eq!(archive.find("WIN3"), None);
        assert_eq!(archive.entries()[2].size, 2);
    }

    #[test]
    fn sizes_follow_the_offsets_not_the_directory_order() {
        // Two entries listed in reverse order of their bytes, a third sharing the second's.
        let mut bytes = build::vid(&[("A", b"aaaa"), ("B", b"bb")]);
        let (a, b) = (4 + 40, 4 + 44 + 40);
        let (offset_a, offset_b) = (bytes[a], bytes[b]);
        bytes[a] = offset_b;
        bytes[b] = offset_a;
        let archive = VidArchive::parse(&bytes).unwrap();
        assert_eq!(archive.find("A"), Some(&b"bb"[..]));
        assert_eq!(archive.find("B"), Some(&b"aaaa"[..]));
        bytes[a] = offset_a;
        let archive = VidArchive::parse(&bytes).unwrap();
        assert_eq!(archive.find("A"), Some(&b"aaaabb"[..]));
        assert_eq!(archive.find("B"), Some(&b"aaaabb"[..]));
    }

    #[test]
    fn a_short_directory_or_an_offset_past_the_end_is_rejected() {
        assert_eq!(VidArchive::parse(&[1, 0]).err(), Some(Error::Truncated));
        assert_eq!(
            VidArchive::parse(&[1, 0, 0, 0]).err(),
            Some(Error::Truncated)
        );
        let mut bytes = build::vid(&[("A", b"abc")]);
        let offset = 4 + NAME_BYTES;
        bytes[offset..offset + 4].copy_from_slice(&52u32.to_le_bytes());
        assert_eq!(VidArchive::parse(&bytes).err(), Some(Error::Truncated));
        bytes[offset..offset + 4].copy_from_slice(&51u32.to_le_bytes());
        assert_eq!(VidArchive::parse(&bytes).unwrap().find("A"), Some(&b""[..]));
        assert!(VidArchive::parse(&[0; 4]).unwrap().entries().is_empty());
        // A name of all 40 bytes has no NUL.
        let mut full = build::vid(&[("A", b"")]);
        full[4..44].fill(b'Q');
        assert_eq!(
            VidArchive::parse(&full).unwrap().entries()[0].name.len(),
            40
        );
    }

    #[test]
    fn an_entry_of_another_archive_reads_as_empty() {
        let big = build::vid(&[("A", &[7; 100])]);
        let small = build::vid(&[("B", b"")]);
        let big = VidArchive::parse(&big).unwrap();
        let small = VidArchive::parse(&small).unwrap();
        assert_eq!(small.data(&big.entries()[0]), b"");
    }

    proptest! {
        #[test]
        fn parse_never_panics(bytes in proptest::collection::vec(any::<u8>(), 0..300)) {
            if let Ok(archive) = VidArchive::parse(&bytes) {
                for entry in archive.entries() {
                    let _ = archive.data(entry);
                }
            }
        }
    }
}
