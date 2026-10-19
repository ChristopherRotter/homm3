//! LOD archives (`Data/*.lod`), layout in `docs/formats/installation.md` §3.
//!
//! Header: magic `LOD\0`, u32 kind, u32 entry count, 80 unknown bytes; entry table of 32-byte
//! records at offset 92 (capacity 10 000); entry = 16-byte NUL-terminated name, u32 absolute data
//! offset, u32 uncompressed size, u32 type tag, u32 compressed size (`0` = stored, otherwise zlib).
//!
//! Two ways to read an archive, with the same checks:
//! - **Table only** (OPEN-QUESTIONS CLT-Q2, ADR-0020 point 7): [`LodTable::parse`] takes the
//!   archive's first bytes (at most [`MAX_TABLE_BYTES`]) and the file's length, and
//!   [`read_entry`] inflates one entry from its stored bytes ([`LodEntry::stored_range`]). The
//!   caller reads only the directory and the entries it needs.
//! - **Whole archive:** [`LodArchive`] borrows every byte of the file and rests on both (the
//!   harvester and the tools).

use std::ops::Range;

use crate::{FormatError, le};

/// Offset of the entry table.
pub const TABLE_OFFSET: usize = 92;
/// Size of one entry record.
pub const ENTRY_SIZE: usize = 32;
/// Number of entry slots in the table (observed in all four archives of the installation).
pub const TABLE_CAPACITY: usize = 10_000;
/// Largest uncompressed entry this reader inflates (the largest observed entry is ≈ 1.5 MB).
pub const MAX_ENTRY_SIZE: u32 = 64 * 1024 * 1024;
/// Bytes from an archive's start that hold its header and a full entry table (320 092; the
/// first entry's data starts there in all four observed archives). Reading
/// `min(file length, MAX_TABLE_BYTES)` bytes once gives [`LodTable::parse`] all it needs.
pub const MAX_TABLE_BYTES: usize = TABLE_OFFSET + TABLE_CAPACITY * ENTRY_SIZE;

/// One entry of the table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LodEntry {
    /// File name as stored (mixed case, ASCII).
    pub name: String,
    /// Absolute offset of the data in the archive.
    pub offset: u32,
    /// Size after decompression.
    pub size: u32,
    /// Type tag (e.g. `2` for text tables in the observed archives).
    pub type_tag: u32,
    /// Size of the zlib stream, or `0` if the data is stored uncompressed.
    pub compressed_size: u32,
}

impl LodEntry {
    /// Number of bytes the entry occupies in the archive.
    #[must_use]
    pub fn stored_size(&self) -> u32 {
        if self.compressed_size == 0 {
            self.size
        } else {
            self.compressed_size
        }
    }

    /// The bytes the entry occupies in the archive: `offset .. offset + stored_size` (in `u64`,
    /// so the end cannot overflow). [`LodTable::parse`] has checked it against the file's length.
    #[must_use]
    pub fn stored_range(&self) -> Range<u64> {
        let start = u64::from(self.offset);
        start..start + u64::from(self.stored_size())
    }
}

/// An archive's directory: the header's kind and the entry table, parsed without the entries'
/// data (CLT-Q2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LodTable {
    kind: u32,
    entries: Vec<LodEntry>,
}

impl LodTable {
    /// Parses the header and the entry table from `head`, the archive's first bytes, and checks
    /// every entry's data range against `file_len`, the length of the whole archive. `head` needs
    /// the header and the used part of the table (`92 + 32 × entries` bytes); a prefix of
    /// [`MAX_TABLE_BYTES`] (or the whole file, if shorter) always has them.
    ///
    /// # Errors
    /// `head` longer than `file_len`, bad magic, a truncated header or table, an entry count above
    /// the table capacity, a name that is not NUL-terminated printable ASCII, or a data range
    /// that ends after `file_len`.
    pub fn parse(head: &[u8], file_len: u64) -> Result<Self, FormatError> {
        if head.len() as u64 > file_len {
            return Err(FormatError::OutOfRange {
                what: "LOD head longer than the file",
                value: head.len() as u64,
            });
        }
        if head.get(..4) != Some(b"LOD\0".as_slice()) {
            return Err(FormatError::BadMagic("LOD archive"));
        }
        let kind = le::u32_at(head, 4, "LOD header")?;
        let count = le::u32_at(head, 8, "LOD header")?;
        let count = usize::try_from(count)
            .ok()
            .filter(|&c| c <= TABLE_CAPACITY)
            .ok_or(FormatError::OutOfRange {
                what: "LOD entry count",
                value: u64::from(count),
            })?;
        let table = head
            .get(TABLE_OFFSET..TABLE_OFFSET + count * ENTRY_SIZE)
            .ok_or(FormatError::Truncated {
                what: "LOD entry table",
                offset: TABLE_OFFSET,
            })?;
        let mut entries = Vec::with_capacity(count);
        for record in table.chunks_exact(ENTRY_SIZE) {
            let entry = LodEntry {
                name: entry_name(&record[..16])?,
                offset: field(record, 16),
                size: field(record, 20),
                type_tag: field(record, 24),
                compressed_size: field(record, 28),
            };
            let end = entry.stored_range().end;
            if end > file_len {
                return Err(FormatError::OutOfRange {
                    what: "LOD entry data end",
                    value: end,
                });
            }
            entries.push(entry);
        }
        Ok(Self { kind, entries })
    }

    /// The archive kind from the header (`200` or `500` in the observed archives).
    #[must_use]
    pub fn kind(&self) -> u32 {
        self.kind
    }

    /// All entries in table order.
    #[must_use]
    pub fn entries(&self) -> &[LodEntry] {
        &self.entries
    }

    /// Finds an entry by name, ignoring ASCII case (the archives mix cases).
    #[must_use]
    pub fn find(&self, name: &str) -> Option<&LodEntry> {
        self.entries
            .iter()
            .find(|e| e.name.eq_ignore_ascii_case(name))
    }
}

/// A `u32` field of a 32-byte entry record; `at + 4 <= ENTRY_SIZE` at every call.
fn field(record: &[u8], at: usize) -> u32 {
    let mut buf = [0u8; 4];
    buf.copy_from_slice(&record[at..at + 4]);
    u32::from_le_bytes(buf)
}

/// Returns an entry's data from its stored bytes, `stored`: exactly the archive's bytes in
/// [`LodEntry::stored_range`], inflated if the entry is compressed. The result has exactly
/// `entry.size` bytes.
///
/// # Errors
/// The declared size exceeds [`MAX_ENTRY_SIZE`], `stored` is not `entry.stored_size()` bytes
/// long, the zlib stream is corrupt, or it inflates to a different size than declared.
pub fn read_entry(entry: &LodEntry, stored: &[u8]) -> Result<Vec<u8>, FormatError> {
    if entry.size > MAX_ENTRY_SIZE {
        return Err(FormatError::OutOfRange {
            what: "LOD entry size",
            value: u64::from(entry.size),
        });
    }
    if stored.len() as u64 != u64::from(entry.stored_size()) {
        return Err(FormatError::OutOfRange {
            what: "LOD entry stored bytes",
            value: stored.len() as u64,
        });
    }
    if entry.compressed_size == 0 {
        return Ok(stored.to_vec());
    }
    let out = miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(stored, entry.size as usize)
        .map_err(|e| FormatError::Inflate(format!("{}: {:?}", entry.name, e.status)))?;
    if out.len() != entry.size as usize {
        return Err(FormatError::Inflate(format!(
            "{}: inflated to {} bytes, header says {}",
            entry.name,
            out.len(),
            entry.size
        )));
    }
    Ok(out)
}

/// A parsed archive borrowing the archive bytes: a [`LodTable`] over the whole file, read with
/// [`read_entry`].
#[derive(Debug)]
pub struct LodArchive<'a> {
    data: &'a [u8],
    table: LodTable,
}

impl<'a> LodArchive<'a> {
    /// Parses the header and entry table. Every entry's data range is checked to lie inside the
    /// archive; decompression happens only in [`LodArchive::read`].
    ///
    /// # Errors
    /// As [`LodTable::parse`] with the whole archive as its head.
    pub fn parse(data: &'a [u8]) -> Result<Self, FormatError> {
        let table = LodTable::parse(data, data.len() as u64)?;
        Ok(Self { data, table })
    }

    /// The archive's directory.
    #[must_use]
    pub fn table(&self) -> &LodTable {
        &self.table
    }

    /// The archive kind from the header (`200` or `500` in the observed archives).
    #[must_use]
    pub fn kind(&self) -> u32 {
        self.table.kind()
    }

    /// All entries in table order.
    #[must_use]
    pub fn entries(&self) -> &[LodEntry] {
        self.table.entries()
    }

    /// Finds an entry by name, ignoring ASCII case (the archives mix cases).
    #[must_use]
    pub fn find(&self, name: &str) -> Option<&LodEntry> {
        self.table.find(name)
    }

    /// Returns the entry's data, inflated if compressed (see [`read_entry`]). The result has
    /// exactly `entry.size` bytes.
    ///
    /// # Errors
    /// The entry's range lies outside the archive (only for an entry built by hand), or as
    /// [`read_entry`].
    pub fn read(&self, entry: &LodEntry) -> Result<Vec<u8>, FormatError> {
        // Ranges were validated in `parse`; `get` keeps this total even for entries built by hand.
        let start = entry.offset as usize;
        let stored = start
            .checked_add(entry.stored_size() as usize)
            .and_then(|end| self.data.get(start..end))
            .ok_or(FormatError::Truncated {
                what: "LOD entry data",
                offset: start,
            })?;
        read_entry(entry, stored)
    }

    /// Reads an entry by name (see [`LodArchive::find`] and [`LodArchive::read`]); `Ok(None)` if
    /// there is no such entry.
    ///
    /// # Errors
    /// As [`LodArchive::read`].
    pub fn read_by_name(&self, name: &str) -> Result<Option<Vec<u8>>, FormatError> {
        self.find(name).map(|e| self.read(e)).transpose()
    }
}

/// Decodes a 16-byte NUL-terminated name field.
fn entry_name(field: &[u8]) -> Result<String, FormatError> {
    let len = field
        .iter()
        .position(|&b| b == 0)
        .ok_or(FormatError::OutOfRange {
            what: "LOD entry name (no NUL)",
            value: 16,
        })?;
    let name = &field[..len];
    if name.is_empty() || !name.iter().all(|b| b.is_ascii_graphic() || *b == b' ') {
        return Err(FormatError::OutOfRange {
            what: "LOD entry name (not printable ASCII)",
            value: len as u64,
        });
    }
    Ok(name.iter().map(|&b| char::from(b)).collect())
}

/// Builds an archive in memory (tests and fixtures; the layout mirrors the observed files but
/// with a table sized to the entries instead of 10 000 slots, which the parser accepts because
/// the capacity is only an upper bound).
///
/// Each item is `(name, data, compress)`.
#[must_use]
pub fn build(kind: u32, items: &[(&str, &[u8], bool)]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"LOD\0");
    out.extend_from_slice(&kind.to_le_bytes());
    out.extend_from_slice(&u32::try_from(items.len()).unwrap_or(u32::MAX).to_le_bytes());
    out.resize(TABLE_OFFSET + items.len() * ENTRY_SIZE, 0);
    for (i, (name, data, compress)) in items.iter().enumerate() {
        let stored = if *compress {
            miniz_oxide::deflate::compress_to_vec_zlib(data, 6)
        } else {
            data.to_vec()
        };
        let offset = u32::try_from(out.len()).unwrap_or(u32::MAX);
        let at = TABLE_OFFSET + i * ENTRY_SIZE;
        let mut field = [0u8; 16];
        for (dst, src) in field.iter_mut().zip(name.bytes().take(15)) {
            *dst = src;
        }
        out[at..at + 16].copy_from_slice(&field);
        out[at + 16..at + 20].copy_from_slice(&offset.to_le_bytes());
        let size = u32::try_from(data.len()).unwrap_or(u32::MAX);
        out[at + 20..at + 24].copy_from_slice(&size.to_le_bytes());
        out[at + 24..at + 28].copy_from_slice(&2u32.to_le_bytes());
        let csize = if *compress {
            u32::try_from(stored.len()).unwrap_or(u32::MAX)
        } else {
            0
        };
        out[at + 28..at + 32].copy_from_slice(&csize.to_le_bytes());
        out.extend_from_slice(&stored);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_stored_and_compressed() {
        let text = b"Name\tValue\r\nOne\t1\r\n".repeat(20);
        let lod = build(500, &[("A.TXT", b"hello", false), ("b.txt", &text, true)]);
        let archive = LodArchive::parse(&lod).unwrap();
        assert_eq!(archive.kind(), 500);
        assert_eq!(archive.entries().len(), 2);
        assert_eq!(archive.read_by_name("a.txt").unwrap().unwrap(), b"hello");
        assert_eq!(archive.read_by_name("B.TXT").unwrap().unwrap(), text);
        assert!(archive.entries()[1].compressed_size > 0);
        assert_eq!(archive.read_by_name("missing").unwrap(), None);
    }

    #[test]
    fn rejects_bad_magic_and_truncation() {
        assert_eq!(
            LodArchive::parse(b"NOPE").unwrap_err(),
            FormatError::BadMagic("LOD archive")
        );
        assert!(matches!(
            LodArchive::parse(b"LOD\0\x01"),
            Err(FormatError::Truncated { .. })
        ));
        let mut lod = build(200, &[("A.TXT", b"x", false)]);
        lod.truncate(TABLE_OFFSET + 10);
        assert!(matches!(
            LodArchive::parse(&lod),
            Err(FormatError::Truncated { .. })
        ));
    }

    #[test]
    fn rejects_count_above_capacity_and_out_of_range_data() {
        let mut lod = build(200, &[("A.TXT", b"x", false)]);
        lod[8..12].copy_from_slice(&10_001u32.to_le_bytes());
        assert!(matches!(
            LodArchive::parse(&lod),
            Err(FormatError::OutOfRange { .. })
        ));

        let mut lod = build(200, &[("A.TXT", b"x", false)]);
        let at = TABLE_OFFSET + 20;
        lod[at..at + 4].copy_from_slice(&1000u32.to_le_bytes());
        assert!(matches!(
            LodArchive::parse(&lod),
            Err(FormatError::OutOfRange { .. })
        ));
    }

    #[test]
    fn rejects_bad_names() {
        let mut lod = build(200, &[("A.TXT", b"x", false)]);
        lod[TABLE_OFFSET..TABLE_OFFSET + 16].copy_from_slice(&[b'A'; 16]);
        assert!(LodArchive::parse(&lod).is_err());
        let mut lod = build(200, &[("A.TXT", b"x", false)]);
        lod[TABLE_OFFSET] = 0x07;
        assert!(LodArchive::parse(&lod).is_err());
    }

    #[test]
    fn corrupt_or_mis_sized_streams_are_errors() {
        let text = b"abcdefgh".repeat(10);
        let mut lod = build(500, &[("A.TXT", &text, true)]);
        // Declare a larger uncompressed size than the stream produces.
        let at = TABLE_OFFSET + 20;
        lod[at..at + 4].copy_from_slice(&81u32.to_le_bytes());
        let archive = LodArchive::parse(&lod).unwrap();
        assert!(matches!(
            archive.read_by_name("A.TXT"),
            Err(FormatError::Inflate(_))
        ));

        let mut lod = build(500, &[("A.TXT", &text, true)]);
        let data_at = lod.len() - 3;
        lod[data_at] ^= 0xFF;
        let archive = LodArchive::parse(&lod).unwrap();
        assert!(matches!(
            archive.read_by_name("A.TXT"),
            Err(FormatError::Inflate(_))
        ));

        let entry = LodEntry {
            name: "X".into(),
            offset: 0,
            size: MAX_ENTRY_SIZE + 1,
            type_tag: 2,
            compressed_size: 1,
        };
        assert!(matches!(
            archive.read(&entry),
            Err(FormatError::OutOfRange { .. })
        ));
        let beyond = LodEntry {
            offset: u32::MAX,
            size: 1,
            compressed_size: 0,
            ..entry
        };
        assert!(matches!(
            archive.read(&beyond),
            Err(FormatError::Truncated { .. })
        ));
    }

    /// Three entries (stored, compressed, stored) and where the table ends.
    fn three() -> (Vec<u8>, usize) {
        let text = b"Name\tValue\r\nOne\t1\r\n".repeat(20);
        let lod = build(
            200,
            &[
                ("A.TXT", b"hello", false),
                ("b.txt", &text, true),
                ("C.PAL", b"", false),
            ],
        );
        (lod, TABLE_OFFSET + 3 * ENTRY_SIZE)
    }

    #[test]
    fn a_table_from_the_prefix_matches_the_whole_archive() {
        assert_eq!(MAX_TABLE_BYTES, 320_092);
        let (lod, table_end) = three();
        let archive = LodArchive::parse(&lod).unwrap();
        let len = lod.len() as u64;
        for head in [&lod[..table_end], &lod[..], &lod[..table_end + 7]] {
            let table = LodTable::parse(head, len).unwrap();
            assert_eq!(&table, archive.table());
            assert_eq!(table.kind(), 200);
            assert_eq!(table.find("c.pal"), Some(&table.entries()[2]));
            assert_eq!(table.find("missing"), None);
        }
        let table = LodTable::parse(&lod[..table_end], len).unwrap();
        for entry in table.entries() {
            let range = entry.stored_range();
            let stored = &lod[range.start as usize..range.end as usize];
            assert_eq!(
                read_entry(entry, stored).unwrap(),
                archive.read(entry).unwrap()
            );
        }
        assert_eq!(
            table.entries()[2].stored_range(),
            lod.len() as u64..lod.len() as u64
        );
    }

    #[test]
    fn the_table_checks_ranges_against_the_file_length_not_the_head() {
        let (lod, table_end) = three();
        let len = lod.len() as u64;
        // The head ends long before the entries' data: only the file's length bounds them.
        assert!(LodTable::parse(&lod[..table_end], len).is_ok());
        assert_eq!(
            LodTable::parse(&lod[..table_end], len - 1),
            Err(FormatError::OutOfRange {
                what: "LOD entry data end",
                value: len,
            })
        );
        assert_eq!(
            LodTable::parse(&lod, len - 1),
            Err(FormatError::OutOfRange {
                what: "LOD head longer than the file",
                value: len,
            })
        );
        assert_eq!(
            LodTable::parse(&lod[..table_end - 1], len),
            Err(FormatError::Truncated {
                what: "LOD entry table",
                offset: TABLE_OFFSET,
            })
        );
        assert_eq!(
            LodTable::parse(&lod[..10], len),
            Err(FormatError::Truncated {
                what: "LOD header",
                offset: 8,
            })
        );
    }

    #[test]
    fn stored_ranges_do_not_overflow() {
        let entry = LodEntry {
            name: "X".into(),
            offset: u32::MAX,
            size: u32::MAX,
            type_tag: 2,
            compressed_size: 0,
        };
        assert_eq!(
            entry.stored_range(),
            u64::from(u32::MAX)..2 * u64::from(u32::MAX)
        );
    }

    #[test]
    fn read_entry_wants_exactly_the_stored_bytes() {
        let (lod, table_end) = three();
        let table = LodTable::parse(&lod[..table_end], lod.len() as u64).unwrap();
        let [stored, compressed, _] = table.entries() else {
            panic!("three entries");
        };
        assert_eq!(read_entry(stored, b"hello").unwrap(), b"hello");
        for (entry, wrong) in [(stored, 4usize), (stored, 6), (compressed, 0)] {
            let bytes = vec![0u8; wrong];
            assert_eq!(
                read_entry(entry, &bytes),
                Err(FormatError::OutOfRange {
                    what: "LOD entry stored bytes",
                    value: wrong as u64,
                })
            );
        }
        let huge = LodEntry {
            size: MAX_ENTRY_SIZE + 1,
            ..stored.clone()
        };
        assert!(matches!(
            read_entry(&huge, b"hello"),
            Err(FormatError::OutOfRange {
                what: "LOD entry size",
                ..
            })
        ));
    }
}
