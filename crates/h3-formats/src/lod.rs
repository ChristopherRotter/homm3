//! LOD archives (`Data/*.lod`), layout in `docs/formats/installation.md` §3.
//!
//! Header: magic `LOD\0`, u32 kind, u32 entry count, 80 unknown bytes; entry table of 32-byte
//! records at offset 92 (capacity 10 000); entry = 16-byte NUL-terminated name, u32 absolute data
//! offset, u32 uncompressed size, u32 type tag, u32 compressed size (`0` = stored, otherwise zlib).

use crate::{FormatError, le};

/// Offset of the entry table.
pub const TABLE_OFFSET: usize = 92;
/// Size of one entry record.
pub const ENTRY_SIZE: usize = 32;
/// Number of entry slots in the table (observed in all four archives of the installation).
pub const TABLE_CAPACITY: usize = 10_000;
/// Largest uncompressed entry this reader inflates (the largest observed entry is ≈ 1.5 MB).
pub const MAX_ENTRY_SIZE: u32 = 64 * 1024 * 1024;

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
}

/// A parsed archive borrowing the archive bytes.
#[derive(Debug)]
pub struct LodArchive<'a> {
    data: &'a [u8],
    kind: u32,
    entries: Vec<LodEntry>,
}

impl<'a> LodArchive<'a> {
    /// Parses the header and entry table. Every entry's data range is checked to lie inside the
    /// archive; decompression happens only in [`LodArchive::read`].
    ///
    /// # Errors
    /// Bad magic, truncated header or table, an entry count above the table capacity, a name that
    /// is not NUL-terminated printable ASCII, or a data range outside the archive.
    pub fn parse(data: &'a [u8]) -> Result<Self, FormatError> {
        if data.get(..4) != Some(b"LOD\0".as_slice()) {
            return Err(FormatError::BadMagic("LOD archive"));
        }
        let kind = le::u32_at(data, 4, "LOD header")?;
        let count = le::u32_at(data, 8, "LOD header")?;
        let count = usize::try_from(count)
            .ok()
            .filter(|&c| c <= TABLE_CAPACITY)
            .ok_or(FormatError::OutOfRange {
                what: "LOD entry count",
                value: u64::from(count),
            })?;
        let table_end = TABLE_OFFSET + count * ENTRY_SIZE;
        if data.len() < table_end {
            return Err(FormatError::Truncated {
                what: "LOD entry table",
                offset: TABLE_OFFSET,
            });
        }
        let mut entries = Vec::with_capacity(count);
        for i in 0..count {
            let at = TABLE_OFFSET + i * ENTRY_SIZE;
            let name = entry_name(&data[at..at + 16])?;
            let entry = LodEntry {
                name,
                offset: le::u32_at(data, at + 16, "LOD entry")?,
                size: le::u32_at(data, at + 20, "LOD entry")?,
                type_tag: le::u32_at(data, at + 24, "LOD entry")?,
                compressed_size: le::u32_at(data, at + 28, "LOD entry")?,
            };
            let end = u64::from(entry.offset) + u64::from(entry.stored_size());
            if end > data.len() as u64 {
                return Err(FormatError::OutOfRange {
                    what: "LOD entry data end",
                    value: end,
                });
            }
            entries.push(entry);
        }
        Ok(Self {
            data,
            kind,
            entries,
        })
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

    /// Returns the entry's data, inflated if compressed. The result has exactly `entry.size`
    /// bytes.
    ///
    /// # Errors
    /// The declared size exceeds [`MAX_ENTRY_SIZE`], the zlib stream is corrupt, or it inflates
    /// to a different size than declared.
    pub fn read(&self, entry: &LodEntry) -> Result<Vec<u8>, FormatError> {
        if entry.size > MAX_ENTRY_SIZE {
            return Err(FormatError::OutOfRange {
                what: "LOD entry size",
                value: u64::from(entry.size),
            });
        }
        // Ranges were validated in `parse`; `get` keeps this total even for entries built by hand.
        let start = entry.offset as usize;
        let stored = start
            .checked_add(entry.stored_size() as usize)
            .and_then(|end| self.data.get(start..end))
            .ok_or(FormatError::Truncated {
                what: "LOD entry data",
                offset: start,
            })?;
        if entry.compressed_size == 0 {
            return Ok(stored.to_vec());
        }
        let out =
            miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(stored, entry.size as usize)
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
    }
}
