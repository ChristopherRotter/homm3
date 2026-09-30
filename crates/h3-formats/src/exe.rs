//! Static tables inside `Heroes3.exe` (`docs/formats/installation.md` §5).
//!
//! Only builds listed in [`KNOWN_BUILDS`] are read: the table location is build-specific, so an
//! unknown executable is rejected instead of being read at a guessed offset.

use crate::{FormatError, le};

/// A build of the executable whose static tables are located.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnownBuild {
    /// Human-readable name.
    pub name: &'static str,
    /// COFF header `TimeDateStamp`.
    pub timestamp: u32,
    /// File length in bytes.
    pub file_len: usize,
    /// Virtual address of the static creature table.
    pub creature_table_va: u32,
    /// Number of creature records.
    pub creature_count: usize,
}

/// Builds known to this reader.
pub const KNOWN_BUILDS: &[KnownBuild] = &[KnownBuild {
    name: "Complete 4.0 (GOG, English), SHA-256 aac39e8c…3d7d",
    timestamp: 0x39B8_3835, // 2000-09-08 00:52:05 UTC
    file_len: 2_732_032,
    creature_table_va: 0x0067_03B8,
    creature_count: 150,
}];

/// Size of one creature record.
pub const CREATURE_RECORD_SIZE: usize = 116;

/// One record of the static creature table. The numeric fields of the record are zero in the file
/// (the game fills them from `CRTRAITS.TXT`), so only the static part is returned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatureRecord {
    /// Creature id (index in the table).
    pub id: u16,
    /// Town index, `-1` for neutral.
    pub town: i32,
    /// Level, 0-based; `-1` for unused ids.
    pub level: i32,
    /// Sound-file prefix.
    pub sound: String,
    /// Battle sprite (DEF) name.
    pub def_name: String,
    /// Ability flag bits (meanings in the format catalogue).
    pub flags: u32,
}

/// A section of the PE image.
#[derive(Debug, Clone, Copy)]
struct Section {
    virtual_address: u32,
    raw_size: u32,
    raw_offset: u32,
}

/// Minimal PE32 view: image base, sections and link timestamp.
#[derive(Debug)]
pub struct PeImage<'a> {
    data: &'a [u8],
    timestamp: u32,
    image_base: u32,
    sections: Vec<Section>,
}

/// Upper bound for the section count (the COFF field is 16 bits; real images have a handful).
const MAX_SECTIONS: u16 = 96;

impl<'a> PeImage<'a> {
    /// Parses the DOS/PE headers and the section table.
    ///
    /// # Errors
    /// Missing signatures, a non-PE32 optional header, too many sections, or truncation.
    pub fn parse(data: &'a [u8]) -> Result<Self, FormatError> {
        if data.get(..2) != Some(b"MZ".as_slice()) {
            return Err(FormatError::BadMagic("DOS executable"));
        }
        let pe = le::u32_at(data, 0x3C, "DOS header")? as usize;
        let sig_end = pe.checked_add(4).ok_or(FormatError::Truncated {
            what: "PE header",
            offset: pe,
        })?;
        if data.get(pe..sig_end) != Some(b"PE\0\0".as_slice()) {
            return Err(FormatError::BadMagic("PE image"));
        }
        let coff = sig_end;
        let sections = le::u16_at(data, coff + 2, "COFF header")?;
        if sections > MAX_SECTIONS {
            return Err(FormatError::OutOfRange {
                what: "PE section count",
                value: u64::from(sections),
            });
        }
        let timestamp = le::u32_at(data, coff + 4, "COFF header")?;
        let optional_size = le::u16_at(data, coff + 16, "COFF header")? as usize;
        let optional = coff + 20;
        if le::u16_at(data, optional, "optional header")? != 0x10B {
            return Err(FormatError::BadMagic("PE32 optional header"));
        }
        let image_base = le::u32_at(data, optional + 28, "optional header")?;
        let table = optional + optional_size;
        let mut out = Vec::with_capacity(usize::from(sections));
        for i in 0..usize::from(sections) {
            let at = table + i * 40;
            out.push(Section {
                virtual_address: le::u32_at(data, at + 12, "section header")?,
                raw_size: le::u32_at(data, at + 16, "section header")?,
                raw_offset: le::u32_at(data, at + 20, "section header")?,
            });
        }
        Ok(Self {
            data,
            timestamp,
            image_base,
            sections: out,
        })
    }

    /// COFF link timestamp.
    #[must_use]
    pub fn timestamp(&self) -> u32 {
        self.timestamp
    }

    /// Maps a virtual address to a file offset inside a section's raw data.
    #[must_use]
    pub fn file_offset(&self, va: u32) -> Option<usize> {
        let rva = va.checked_sub(self.image_base)?;
        self.sections.iter().find_map(|s| {
            let delta = rva.checked_sub(s.virtual_address)?;
            (delta < s.raw_size).then(|| s.raw_offset as usize + delta as usize)
        })
    }

    /// Reads a NUL-terminated string at a virtual address (at most 256 bytes, printable ASCII).
    ///
    /// # Errors
    /// The address is not mapped, the string is unterminated, too long or not printable.
    pub fn c_string(&self, va: u32) -> Result<String, FormatError> {
        let start = self.file_offset(va).ok_or(FormatError::OutOfRange {
            what: "string address",
            value: u64::from(va),
        })?;
        let tail = self.data.get(start..).unwrap_or_default();
        let len = tail
            .iter()
            .take(256)
            .position(|&b| b == 0)
            .ok_or(FormatError::OutOfRange {
                what: "string length",
                value: u64::from(va),
            })?;
        let bytes = &tail[..len];
        if !bytes.iter().all(|b| b.is_ascii_graphic() || *b == b' ') {
            return Err(FormatError::OutOfRange {
                what: "string contents",
                value: u64::from(va),
            });
        }
        Ok(bytes.iter().map(|&b| char::from(b)).collect())
    }
}

/// Identifies the build of an executable.
///
/// # Errors
/// [`FormatError::UnknownBuild`] if the image is valid but not in [`KNOWN_BUILDS`], or a PE error.
pub fn identify(data: &[u8]) -> Result<(&'static KnownBuild, PeImage<'_>), FormatError> {
    let image = PeImage::parse(data)?;
    KNOWN_BUILDS
        .iter()
        .find(|b| b.timestamp == image.timestamp() && b.file_len == data.len())
        .map(|b| (b, image))
        .ok_or_else(|| {
            FormatError::UnknownBuild(format!(
                "timestamp {:#010x}, {} bytes; known: {}",
                PeImage::parse(data)
                    .map(|i| i.timestamp())
                    .unwrap_or_default(),
                data.len(),
                KNOWN_BUILDS
                    .iter()
                    .map(|b| b.name)
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        })
}

/// Reads the static creature table of a known build.
///
/// # Errors
/// Unknown build, unmapped table or strings, or values outside the ranges seen in the table
/// (town `-1..=8`, level `-1..=6`).
pub fn creature_table(data: &[u8]) -> Result<Vec<CreatureRecord>, FormatError> {
    let (build, image) = identify(data)?;
    creature_table_at(&image, build.creature_table_va, build.creature_count)
}

/// Reads `count` creature records at virtual address `va` (exposed for tests with synthetic
/// images).
///
/// # Errors
/// As [`creature_table`], without the build check.
pub fn creature_table_at(
    image: &PeImage<'_>,
    va: u32,
    count: usize,
) -> Result<Vec<CreatureRecord>, FormatError> {
    let base = image.file_offset(va).ok_or(FormatError::OutOfRange {
        what: "creature table address",
        value: u64::from(va),
    })?;
    let mut out = Vec::with_capacity(count.min(1024));
    for i in 0..count {
        let at = base + i * CREATURE_RECORD_SIZE;
        let town = le::i32_at(image.data, at, "creature record")?;
        let level = le::i32_at(image.data, at + 4, "creature record")?;
        if !(-1..=8).contains(&town) {
            return Err(FormatError::OutOfRange {
                what: "creature town",
                value: u64::from(town.unsigned_abs()),
            });
        }
        if !(-1..=6).contains(&level) {
            return Err(FormatError::OutOfRange {
                what: "creature level",
                value: u64::from(level.unsigned_abs()),
            });
        }
        // Make sure the whole record is present even though only the static part is decoded.
        if image.data.len() < at + CREATURE_RECORD_SIZE {
            return Err(FormatError::Truncated {
                what: "creature record",
                offset: at,
            });
        }
        out.push(CreatureRecord {
            id: u16::try_from(i).map_err(|_| FormatError::OutOfRange {
                what: "creature id",
                value: i as u64,
            })?,
            town,
            level,
            sound: image.c_string(le::u32_at(image.data, at + 8, "creature record")?)?,
            def_name: image.c_string(le::u32_at(image.data, at + 12, "creature record")?)?,
            flags: le::u32_at(image.data, at + 16, "creature record")?,
        });
    }
    Ok(out)
}

/// Builds a minimal PE32 image with one section that holds `payload` at RVA `0x1000`, image base
/// `0x400000` (tests only; the layout is the standard PE layout, not game data).
#[must_use]
pub fn build_test_image(timestamp: u32, payload: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; 0x200];
    out[0..2].copy_from_slice(b"MZ");
    out[0x3C..0x40].copy_from_slice(&0x80u32.to_le_bytes());
    out[0x80..0x84].copy_from_slice(b"PE\0\0");
    let coff = 0x84;
    out[coff..coff + 2].copy_from_slice(&0x14Cu16.to_le_bytes());
    out[coff + 2..coff + 4].copy_from_slice(&1u16.to_le_bytes());
    out[coff + 4..coff + 8].copy_from_slice(&timestamp.to_le_bytes());
    out[coff + 16..coff + 18].copy_from_slice(&0xE0u16.to_le_bytes());
    let opt = coff + 20;
    out[opt..opt + 2].copy_from_slice(&0x10Bu16.to_le_bytes());
    out[opt + 28..opt + 32].copy_from_slice(&0x0040_0000u32.to_le_bytes());
    let sec = opt + 0xE0;
    out[sec..sec + 5].copy_from_slice(b".data");
    let raw_size = u32::try_from(payload.len()).unwrap_or(u32::MAX);
    out[sec + 8..sec + 12].copy_from_slice(&raw_size.to_le_bytes());
    out[sec + 12..sec + 16].copy_from_slice(&0x1000u32.to_le_bytes());
    out[sec + 16..sec + 20].copy_from_slice(&raw_size.to_le_bytes());
    out[sec + 20..sec + 24].copy_from_slice(&0x200u32.to_le_bytes());
    out.extend_from_slice(payload);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A payload with strings at the start and two creature records after them.
    fn payload() -> (Vec<u8>, u32) {
        let mut p = Vec::new();
        p.extend_from_slice(b"snd\0");
        p.extend_from_slice(b"unit.def\0\0\0\0");
        let table_rva = 0x1000 + u32::try_from(p.len()).unwrap();
        for (town, level, flags) in [(0i32, 0i32, 0x10u32), (-1, 6, 0x8000_0003)] {
            let mut r = vec![0u8; CREATURE_RECORD_SIZE];
            r[0..4].copy_from_slice(&town.to_le_bytes());
            r[4..8].copy_from_slice(&level.to_le_bytes());
            r[8..12].copy_from_slice(&0x0040_1000u32.to_le_bytes());
            r[12..16].copy_from_slice(&0x0040_1004u32.to_le_bytes());
            r[16..20].copy_from_slice(&flags.to_le_bytes());
            p.extend_from_slice(&r);
        }
        (p, 0x0040_0000 + table_rva)
    }

    #[test]
    fn reads_records_from_a_synthetic_image() {
        let (p, va) = payload();
        let img = build_test_image(1, &p);
        let pe = PeImage::parse(&img).unwrap();
        assert_eq!(pe.timestamp(), 1);
        let recs = creature_table_at(&pe, va, 2).unwrap();
        assert_eq!(recs.len(), 2);
        assert_eq!(
            recs[0],
            CreatureRecord {
                id: 0,
                town: 0,
                level: 0,
                sound: "snd".into(),
                def_name: "unit.def".into(),
                flags: 0x10
            }
        );
        assert_eq!(
            (recs[1].town, recs[1].level, recs[1].flags),
            (-1, 6, 0x8000_0003)
        );
        // Reading past the section is an error, not a panic.
        assert!(creature_table_at(&pe, va, 3).is_err());
        assert!(creature_table_at(&pe, 0x10, 1).is_err());
    }

    #[test]
    fn unknown_builds_are_rejected() {
        let (p, _) = payload();
        let img = build_test_image(1, &p);
        assert!(matches!(identify(&img), Err(FormatError::UnknownBuild(_))));
        assert!(matches!(
            creature_table(&img),
            Err(FormatError::UnknownBuild(_))
        ));
    }

    #[test]
    fn out_of_range_values_and_bad_strings() {
        let (mut p, va) = payload();
        let rec = p.len() - 2 * CREATURE_RECORD_SIZE;
        p[rec..rec + 4].copy_from_slice(&9i32.to_le_bytes());
        let img = build_test_image(1, &p);
        let pe = PeImage::parse(&img).unwrap();
        assert!(matches!(
            creature_table_at(&pe, va, 1),
            Err(FormatError::OutOfRange {
                what: "creature town",
                ..
            })
        ));

        let (mut p, va) = payload();
        p[rec + 4..rec + 8].copy_from_slice(&7i32.to_le_bytes());
        let img = build_test_image(1, &p);
        let pe = PeImage::parse(&img).unwrap();
        assert!(creature_table_at(&pe, va, 1).is_err());

        let (mut p, _) = payload();
        p[0] = 0x01; // unprintable string
        let img = build_test_image(1, &p);
        let pe = PeImage::parse(&img).unwrap();
        assert!(pe.c_string(0x0040_1000).is_err());
        let (mut p, _) = payload();
        p[3] = b'x'; // no NUL until the next string: still terminated, so check a long run
        p.splice(0..0, std::iter::repeat_n(b'a', 300));
        let img = build_test_image(1, &p);
        let pe = PeImage::parse(&img).unwrap();
        assert!(pe.c_string(0x0040_1000).is_err());
    }

    #[test]
    fn header_errors() {
        assert!(matches!(
            PeImage::parse(b"ZZ"),
            Err(FormatError::BadMagic(_))
        ));
        let (p, _) = payload();
        let mut img = build_test_image(1, &p);
        img[0x80] = b'X';
        assert!(matches!(
            PeImage::parse(&img),
            Err(FormatError::BadMagic(_))
        ));
        let mut img = build_test_image(1, &p);
        img[0x84 + 20] = 0x0B;
        img[0x84 + 21] = 0x02; // PE32+ magic
        assert!(matches!(
            PeImage::parse(&img),
            Err(FormatError::BadMagic(_))
        ));
        let mut img = build_test_image(1, &p);
        img[0x86..0x88].copy_from_slice(&200u16.to_le_bytes());
        assert!(matches!(
            PeImage::parse(&img),
            Err(FormatError::OutOfRange { .. })
        ));
        let img = build_test_image(1, &p);
        assert!(PeImage::parse(&img[..0x90]).is_err());
    }
}
