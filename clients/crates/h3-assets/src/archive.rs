//! The archives' contents at a glance (E17 M0 T3, the real-asset gate): per archive, how many
//! entries of each extension it holds, how many are compressed and their unpacked size. The
//! archives are parsed by the engine's `h3-formats`; the lookup across archives (which one wins
//! for a name) follows the code reading (M1 T1).

use alloc::collections::BTreeMap;
use alloc::string::String;

use h3_formats::lod::LodArchive;

/// Counts over a set of entries.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Tally {
    /// Number of entries.
    pub entries: u32,
    /// How many of them are stored compressed (zlib).
    pub compressed: u32,
    /// Their size after decompression, in bytes.
    pub bytes: u64,
}

impl Tally {
    fn add(&mut self, size: u32, compressed: bool) {
        self.entries = self.entries.saturating_add(1);
        self.compressed = self.compressed.saturating_add(u32::from(compressed));
        self.bytes = self.bytes.saturating_add(u64::from(size));
    }
}

/// What one archive holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inventory {
    /// The archive kind from its header.
    pub kind: u32,
    /// All entries.
    pub total: Tally,
    /// The entries by extension ([`extension`]), in ascending order.
    pub by_extension: BTreeMap<String, Tally>,
}

/// The inventory of a parsed archive.
#[must_use]
pub fn inventory(archive: &LodArchive<'_>) -> Inventory {
    let mut total = Tally::default();
    let mut by_extension: BTreeMap<String, Tally> = BTreeMap::new();
    for entry in archive.entries() {
        let compressed = entry.compressed_size != 0;
        total.add(entry.size, compressed);
        by_extension
            .entry(extension(&entry.name))
            .or_default()
            .add(entry.size, compressed);
    }
    Inventory {
        kind: archive.kind(),
        total,
        by_extension,
    }
}

/// An entry name's extension in upper case, without the dot; empty if the name has none. The
/// archives mix cases (`h3-formats` looks names up ignoring ASCII case).
#[must_use]
pub fn extension(name: &str) -> String {
    name.rsplit_once('.')
        .map_or_else(String::new, |(_, ext)| ext.to_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use h3_formats::lod::build;

    #[test]
    fn counts_entries_by_extension() {
        let bytes = build(
            200,
            &[
                ("A.def", &[1, 2, 3], true),
                ("b.DEF", &[4; 10], false),
                ("c.pcx", &[5; 7], true),
                ("NOEXT", &[], false),
            ],
        );
        let archive = LodArchive::parse(&bytes).unwrap();
        let inv = inventory(&archive);
        assert_eq!(inv.kind, 200);
        assert_eq!(
            inv.total,
            Tally {
                entries: 4,
                compressed: 2,
                bytes: 20
            }
        );
        let keys: alloc::vec::Vec<&str> = inv.by_extension.keys().map(String::as_str).collect();
        assert_eq!(keys, ["", "DEF", "PCX"]);
        assert_eq!(
            inv.by_extension["DEF"],
            Tally {
                entries: 2,
                compressed: 1,
                bytes: 13
            }
        );
        assert_eq!(inv.by_extension[""].entries, 1);
    }

    #[test]
    fn an_empty_archive_has_an_empty_inventory() {
        let bytes = build(500, &[]);
        let inv = inventory(&LodArchive::parse(&bytes).unwrap());
        assert_eq!(inv.kind, 500);
        assert_eq!(inv.total, Tally::default());
        assert!(inv.by_extension.is_empty());
    }

    #[test]
    fn extension_takes_the_last_dot() {
        assert_eq!(extension("a.b.msk"), "MSK");
        assert_eq!(extension("trailing."), "");
        assert_eq!(extension("none"), "");
    }

    #[test]
    fn tallies_saturate() {
        let mut t = Tally {
            entries: u32::MAX,
            compressed: u32::MAX,
            bytes: u64::MAX,
        };
        t.add(1, true);
        assert_eq!(
            t,
            Tally {
                entries: u32::MAX,
                compressed: u32::MAX,
                bytes: u64::MAX
            }
        );
    }
}
