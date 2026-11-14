//! The LOD archives the viewer reads (CLT-Q2, ADR-0020 point 7; E27 M2 T3b): at the start only
//! their tables (`h3_formats::lod::LodTable`, from at most `lod::MAX_TABLE_BYTES` of each
//! archive's head), then an entry's stored bytes by its range when a lookup or a batch takes it;
//! a batch's entries archive by archive in the order of their ranges, through one open file each.
//! The set holds the archives' paths and tables, not their bytes (the four LODs' 212 MiB were
//! read whole at every start until T3b). The video archives are [`crate::videos`]'s, the sound
//! archives [`crate::sounds`]'.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use h3_formats::lod::{self, LodEntry, LodTable};

use crate::gate::find_ci;

/// The archives a DEF is looked up in, in the executable's order: the base archive first, so its
/// copy of a duplicate name wins (`docs/epics/E17-design.md` §3.1).
pub const SPRITES: [&str; 2] = ["H3sprite.lod", "H3ab_spr.lod"];
/// The archives every other entry is looked up in, in the same order.
pub const BITMAPS: [&str; 2] = ["H3bitmap.lod", "H3ab_bmp.lod"];

/// One archive: its name, where it is, its length and its table.
#[derive(Debug, Clone)]
pub struct Lod {
    name: &'static str,
    path: PathBuf,
    len: u64,
    head: usize,
    table: LodTable,
}

impl Lod {
    /// The archive `name` of the folder `data` (found ignoring case), listed from its head alone.
    ///
    /// # Errors
    /// No such file, a read failing, or a table `LodTable::parse` refuses.
    pub fn open(data: &Path, name: &'static str) -> Result<Self, String> {
        let path = find_ci(data, name).map_err(|e| format!("{}/{name}: {e}", data.display()))?;
        let shown = path.display().to_string();
        let failed = |e: std::io::Error| format!("{shown}: {e}");
        let mut file = File::open(&path).map_err(failed)?;
        let len = file.metadata().map_err(failed)?.len();
        let head =
            usize::try_from(len).map_or(lod::MAX_TABLE_BYTES, |n| n.min(lod::MAX_TABLE_BYTES));
        let mut bytes = vec![0; head];
        file.read_exact(&mut bytes).map_err(failed)?;
        let table = LodTable::parse(&bytes, len).map_err(|e| format!("{shown}: {e:?}"))?;
        Ok(Self {
            name,
            path,
            len,
            head,
            table,
        })
    }

    /// The archive's name as the game names it (`H3bitmap.lod`).
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Where the archive is.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The archive's length in bytes.
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Whether the archive is empty (never, once listed: its header alone has 92 bytes).
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The bytes read to list the archive: its head.
    pub fn head_len(&self) -> usize {
        self.head
    }

    /// The archive's table.
    pub fn table(&self) -> &LodTable {
        &self.table
    }

    /// The archive opened for reading entries.
    ///
    /// # Errors
    /// The file cannot be opened.
    pub fn reader(&self) -> Result<Reader<'_>, String> {
        let file = File::open(&self.path).map_err(|e| format!("{}: {e}", self.path.display()))?;
        Ok(Reader { lod: self, file })
    }

    /// The entry `name` (ignoring case), inflated; none when the archive lacks it.
    ///
    /// # Errors
    /// The entry cannot be read or inflated.
    pub fn read_by_name(&self, name: &str) -> Result<Option<Vec<u8>>, String> {
        match self.table.find(name) {
            Some(entry) => self.reader()?.read(entry).map(Some),
            None => Ok(None),
        }
    }
}

/// An archive open for reading entries by their ranges (reading them in the order of their
/// ranges moves forward through the file).
#[derive(Debug)]
pub struct Reader<'a> {
    lod: &'a Lod,
    file: File,
}

impl Reader<'_> {
    /// The bytes `entry` occupies in the archive (`LodEntry::stored_range`), as stored.
    ///
    /// # Errors
    /// A seek or read failing (the archive shorter than when it was listed among them).
    pub fn stored(&mut self, entry: &LodEntry) -> Result<Vec<u8>, String> {
        let name = self.lod.name;
        let failed = |e: std::io::Error| format!("{name}:{}: {e}", entry.name);
        let range = entry.stored_range();
        self.file
            .seek(SeekFrom::Start(range.start))
            .map_err(failed)?;
        let size = usize::try_from(entry.stored_size())
            .map_err(|_| format!("{name}:{}: too large", entry.name))?;
        let mut bytes = vec![0; size];
        self.file.read_exact(&mut bytes).map_err(failed)?;
        Ok(bytes)
    }

    /// `entry`'s data: its stored bytes, inflated (`lod::read_entry`).
    ///
    /// # Errors
    /// The bytes cannot be read, or `lod::read_entry` refuses them.
    pub fn read(&mut self, entry: &LodEntry) -> Result<Vec<u8>, String> {
        let stored = self.stored(entry)?;
        lod::read_entry(entry, &stored)
            .map_err(|e| format!("{}:{}: {e:?}", self.lod.name, entry.name))
    }
}

/// The archives of an installation copy's `Data` folder, listed.
#[derive(Debug, Clone, Default)]
pub struct Lods {
    lods: Vec<Lod>,
}

impl Lods {
    /// The four LODs of `install`'s `Data` folder (found ignoring case), the sprites' first.
    ///
    /// # Errors
    /// An archive missing or its table unreadable.
    pub fn open(install: &Path) -> Result<Self, String> {
        let names: Vec<&'static str> = SPRITES.iter().chain(&BITMAPS).copied().collect();
        Self::open_these(install, &names)
    }

    /// The archives `names` of `install`'s `Data` folder, in that order.
    ///
    /// # Errors
    /// The folder or an archive missing, or a table unreadable.
    pub fn open_these(install: &Path, names: &[&'static str]) -> Result<Self, String> {
        let data = find_ci(install, "Data").map_err(|e| format!("{}: {e}", install.display()))?;
        let lods = names
            .iter()
            .map(|name| Lod::open(&data, name))
            .collect::<Result<_, _>>()?;
        Ok(Self { lods })
    }

    /// The archives listed, in their order.
    pub fn lods(&self) -> &[Lod] {
        &self.lods
    }

    /// The search list of `name`: [`SPRITES`] for a DEF, else [`BITMAPS`].
    fn list(name: &str) -> &'static [&'static str] {
        if name.to_ascii_uppercase().ends_with(".DEF") {
            &SPRITES
        } else {
            &BITMAPS
        }
    }

    /// The archive (its index) and entry of `name`: the first archive of its search list that
    /// holds it.
    fn find(&self, name: &str) -> Result<(usize, &LodEntry), String> {
        let list = Self::list(name);
        list.iter()
            .filter_map(|n| self.lods.iter().position(|l| l.name == *n))
            .find_map(|k| self.lods[k].table.find(name).map(|e| (k, e)))
            .ok_or_else(|| format!("{name}: in none of {}", list.join(", ")))
    }

    /// The entry `name` from the first archive of its search list that holds it.
    ///
    /// # Errors
    /// No archive of the list holds it, or it cannot be read.
    pub fn lookup(&self, name: &str) -> Result<Vec<u8>, String> {
        let (k, entry) = self.find(name)?;
        self.lods[k].reader()?.read(entry)
    }

    /// The entries `names`, each as [`Lods::lookup`] finds it, in the order of `names`: read
    /// archive by archive, each archive's in the order of their ranges through one open file.
    ///
    /// # Errors
    /// The first name no archive of its list holds, or an entry that cannot be read.
    pub fn batch(&self, names: &[String]) -> Result<Vec<Vec<u8>>, String> {
        let mut by_archive: BTreeMap<usize, Vec<(u64, usize, &LodEntry)>> = BTreeMap::new();
        for (i, name) in names.iter().enumerate() {
            let (k, entry) = self.find(name)?;
            by_archive
                .entry(k)
                .or_default()
                .push((entry.stored_range().start, i, entry));
        }
        let mut read: Vec<Option<Vec<u8>>> = vec![None; names.len()];
        for (k, mut entries) in by_archive {
            // The name's index breaks a tie (one entry named twice).
            entries.sort_unstable_by_key(|&(start, i, _)| (start, i));
            let mut reader = self.lods[k].reader()?;
            for (_, i, entry) in entries {
                read[i] = Some(reader.read(entry)?);
            }
        }
        Ok(read.into_iter().flatten().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use h3_formats::lod::build;
    use std::fs;

    /// A `Data` folder under a fresh temporary directory, the archives `lods` written there.
    fn install(tag: &str, lods: &[(&str, Vec<u8>)]) -> PathBuf {
        let root = std::env::temp_dir().join(format!("h3-view-lods-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let data = root.join("DATA");
        fs::create_dir_all(&data).unwrap();
        for (name, bytes) in lods {
            fs::write(data.join(name.to_ascii_lowercase()), bytes).unwrap();
        }
        root
    }

    fn four() -> Vec<(&'static str, Vec<u8>)> {
        vec![
            (
                "H3sprite.lod",
                build(
                    500,
                    &[("A.DEF", b"base a", true), ("b.def", b"base b", false)],
                ),
            ),
            (
                "H3ab_spr.lod",
                build(500, &[("A.def", b"ab a", false), ("C.DEF", b"ab c", true)]),
            ),
            (
                "H3bitmap.lod",
                build(
                    200,
                    &[("x.pcx", b"base x", true), ("Y.TXT", b"base y", false)],
                ),
            ),
            (
                "H3ab_bmp.lod",
                build(200, &[("X.PCX", b"ab x", false), ("z.pcx", b"ab z", false)]),
            ),
        ]
    }

    #[test]
    fn the_tables_alone_are_read_and_lookups_follow_the_search_lists() {
        let root = install("lookup", &four());
        let lods = Lods::open(&root).unwrap();
        let names: Vec<&str> = lods.lods().iter().map(Lod::name).collect();
        assert_eq!(
            names,
            [
                "H3sprite.lod",
                "H3ab_spr.lod",
                "H3bitmap.lod",
                "H3ab_bmp.lod"
            ]
        );
        let small = &lods.lods()[0];
        // A file shorter than the largest table is read whole to list it.
        assert_eq!(u64::try_from(small.head_len()).unwrap(), small.len());
        assert!(!small.is_empty());
        assert_eq!(small.table().entries().len(), 2);
        assert!(small.path().ends_with("h3sprite.lod"));
        // The base archive wins a duplicate; a DEF is never looked up in the bitmaps.
        assert_eq!(lods.lookup("a.def").unwrap(), b"base a");
        assert_eq!(lods.lookup("c.def").unwrap(), b"ab c");
        assert_eq!(lods.lookup("X.pcx").unwrap(), b"base x");
        assert_eq!(lods.lookup("z.PCX").unwrap(), b"ab z");
        assert_eq!(
            lods.lookup("y.def").unwrap_err(),
            "y.def: in none of H3sprite.lod, H3ab_spr.lod"
        );
        assert_eq!(
            lods.lookup("b.pcx").unwrap_err(),
            "b.pcx: in none of H3bitmap.lod, H3ab_bmp.lod"
        );
        assert_eq!(
            lods.lods()[1].read_by_name("A.DEF").unwrap().unwrap(),
            b"ab a"
        );
        assert_eq!(lods.lods()[1].read_by_name("b.def").unwrap(), None);
        // A set of some archives looks only in those.
        let bitmaps = Lods::open_these(&root, &["H3ab_bmp.lod"]).unwrap();
        assert_eq!(bitmaps.lookup("x.pcx").unwrap(), b"ab x");
        assert!(bitmaps.lookup("a.def").is_err());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_batch_comes_back_in_the_order_of_its_names() {
        let root = install("batch", &four());
        let lods = Lods::open(&root).unwrap();
        let names: Vec<String> = [
            "z.pcx", "B.DEF", "x.pcx", "a.def", "Y.txt", "a.def", "c.def",
        ]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
        let read = lods.batch(&names).unwrap();
        let read: Vec<&[u8]> = read.iter().map(Vec::as_slice).collect();
        assert_eq!(
            read,
            [
                b"ab z".as_slice(),
                b"base b",
                b"base x",
                b"base a",
                b"base y",
                b"base a",
                b"ab c"
            ]
        );
        assert_eq!(lods.batch(&[]).unwrap(), Vec::<Vec<u8>>::new());
        let missing = ["x.pcx".to_owned(), "q.pcx".to_owned()];
        assert_eq!(
            lods.batch(&missing).unwrap_err(),
            "q.pcx: in none of H3bitmap.lod, H3ab_bmp.lod"
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_missing_archive_or_folder_a_bad_table_and_a_cut_archive_fail() {
        let mut lods = four();
        lods.pop();
        let root = install("missing", &lods);
        assert!(Lods::open(&root).unwrap_err().contains("H3ab_bmp.lod"));
        assert!(
            Lods::open(&root.join("nowhere"))
                .unwrap_err()
                .contains("nowhere")
        );
        fs::write(root.join("DATA/h3ab_bmp.lod"), b"LOD\0").unwrap();
        assert!(Lods::open(&root).unwrap_err().contains("Truncated"));
        // An archive cut after it was listed: its entries no longer read.
        fs::write(root.join("DATA/h3ab_bmp.lod"), &four()[3].1).unwrap();
        let listed = Lods::open(&root).unwrap();
        let bytes = &four()[3].1;
        fs::write(root.join("DATA/h3ab_bmp.lod"), &bytes[..bytes.len() - 2]).unwrap();
        assert!(
            listed
                .lookup("z.pcx")
                .unwrap_err()
                .starts_with("H3ab_bmp.lod:z.pcx: ")
        );
        assert!(listed.batch(&["z.pcx".to_owned()]).is_err());
        // Bytes that do not inflate.
        fs::write(root.join("DATA/h3ab_bmp.lod"), bytes).unwrap();
        let mut bad = four()[2].1.clone();
        let len = u64::try_from(bad.len()).unwrap();
        let entry = LodTable::parse(&bad, len).unwrap().entries()[0].clone();
        let start = usize::try_from(entry.offset).unwrap();
        bad[start..start + 2].copy_from_slice(&[0xFF, 0xFF]);
        fs::write(root.join("DATA/h3bitmap.lod"), &bad).unwrap();
        let listed = Lods::open(&root).unwrap();
        assert!(listed.lookup("x.pcx").unwrap_err().contains("Inflate"));
        // An archive gone after it was listed.
        fs::remove_file(root.join("DATA/h3bitmap.lod")).unwrap();
        assert!(listed.lookup("Y.TXT").is_err());
        fs::remove_dir_all(&root).unwrap();
    }
}
