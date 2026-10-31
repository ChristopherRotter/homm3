//! The texts the panels show, in the adapters (E24 M3 T5a, *provisional*;
//! `docs/epics/E24-design.md` §4.7): the keys the viewer uses ([`h3_map_view::panels::KEYS`],
//! since T5b1 [`h3_map_view::status::KEYS`] and the families [`FAMILIES`], §4.8) taken from the
//! client's key catalogue (`clients/text/keys.toml`, common with the battle client; its other
//! families, the battle's, left out), their original forms read from the installation copy's text
//! tables (the archives as the game searches a name) and from its `Heroes3.exe` (the table
//! `EXE`), as the battle client's `h3-view text` reads them.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use h3_formats::lod::LodArchive;
use h3_formats::text::TextTable;
use h3_map_view::{hover, panels, status};
use h3_ui::text::Catalogue;
use h3_ui::text::keys::{EXECUTABLE, Entry, Keys, KeysFile, Row, address_field};
use h3_ui::text::pack::{Pack, Tables};

use crate::files;

/// The families the viewer names by (E24 M3 T5b1, design §4.8): a hero type's name and a town's,
/// each keyed by its original id (`hero.12`, `town-name.37`: [`status::hero_key`],
/// [`status::town_key`]); since T6a1 the hints' (design §4.10): a panel item's, an object class's,
/// a hero class's, a town type's and a creature's names (`map-hint.7`, `object.54`,
/// `hero-class.2`, `town-type.0`, `creature.13`); since T6a2a an object's bonus (`map-bonus.3`,
/// design §4.11); since T6a2b the owners' and contents' names (design §4.12: a colour's, a
/// dwelling's, a resource's, a mine's, a bank's, a spell's, a Tree's price, a skill's, a seer's,
/// an artifact's).
pub const FAMILIES: [&str; 22] = [
    "hero",
    "town-name",
    "map-hint",
    "object",
    "hero-class",
    "town-type",
    "creature",
    "map-bonus",
    "keymaster",
    "player-owned",
    "dwelling1",
    "dwelling4",
    "resource",
    "mine",
    "bank",
    "spell",
    "tree-price",
    "secondary-skill",
    "seer-name",
    "primary-skill",
    "artifact",
    "player-colour",
];

/// The committed key catalogue (`clients/text/keys.toml`).
#[cfg(test)]
pub fn default_keys() -> PathBuf {
    keys_beside(&files::default_map())
}

/// The key catalogue beside the map client's data folder `map` (`<map>/../text/keys.toml`, as
/// `clients/map` and `clients/text` lie in a checkout): a build run elsewhere, such as a Windows
/// build on the user's machine with `--map out\win\map`, reads the catalogue copied beside it
/// (E24 M3 T7c1, *provisional*; the build's own folder is not on that machine).
pub fn keys_beside(map: &Path) -> PathBuf {
    map.join("../text/keys.toml")
}

/// The viewer's keys of the catalogue `text`: the panels' and the status window's
/// ([`panels::KEYS`], [`status::KEYS`]), each one it must hold, and its families' ([`FAMILIES`])
/// over as many ids as `ids` gives each.
pub fn keys(text: &str, ids: &BTreeMap<String, usize>) -> Result<Keys, String> {
    let file: KeysFile = toml::from_str(text).map_err(|e| e.to_string())?;
    // The other families' entries are the battle client's asset tables: none here.
    let entries = file
        .family
        .iter()
        .map(|f| {
            let list = f.list.clone().unwrap_or_else(|| f.id.clone());
            let count = ids.get(&list).copied().unwrap_or_default();
            let named = (0..count)
                .map(|id| Entry {
                    key: id.to_string(),
                    id,
                })
                .collect();
            (list, named)
        })
        .collect();
    let mut all = Keys::parse(text, &entries).map_err(|e| e.to_string())?;
    let mut keys = Keys::default();
    for id in panels::KEYS
        .into_iter()
        .chain(status::KEYS)
        .chain(hover::KEYS)
        .chain(hover::contents::KEYS)
    {
        let key = all
            .keys
            .remove(id)
            .ok_or_else(|| format!("no key `{id}`"))?;
        keys.keys.insert(id.to_string(), key);
    }
    for family in FAMILIES {
        let prefix = format!("{family}.");
        keys.keys
            .extend(all.keys.extract_if(.., |id, _| id.starts_with(&prefix)));
    }
    Ok(keys)
}

/// The ids of [`FAMILIES`] in the catalogue `text`: each family's non-blank records of its table
/// in `archives` from its first (with `by`, those whose column `by` is filled), as the original's
/// pack finds a family's record.
pub fn family_ids(
    text: &str,
    archives: &[LodArchive<'_>],
) -> Result<BTreeMap<String, usize>, String> {
    let file: KeysFile = toml::from_str(text).map_err(|e| e.to_string())?;
    let mut ids = BTreeMap::new();
    for family in file
        .family
        .iter()
        .filter(|f| FAMILIES.contains(&f.id.as_str()))
    {
        if let Some(count) = family.count {
            // A family of records one by one (E24 M3 T6a1): its count.
            ids.insert(family.id.clone(), count);
            continue;
        }
        let name = format!("{}.TXT", family.table);
        let (_, bytes) = files::find(archives, &name)?.ok_or_else(|| format!("no {name}"))?;
        let table = TextTable::parse(&bytes).map_err(|e| format!("{name}: {e:?}"))?;
        let count = table
            .records
            .iter()
            .skip(family.first)
            .filter(|r| match family.by {
                Some(column) => r.get(column).is_some_and(|f| !f.trim().is_empty()),
                None => !r.iter().all(|f| f.trim().is_empty()),
            })
            .count();
        ids.insert(family.id.clone(), count);
    }
    Ok(ids)
}

/// The text tables `keys` name: a table's records from its `.TXT` in `archives` (where the game
/// searches it: `H3bitmap`, then `H3ab_bmp`), the executable's strings from `exe` (the table
/// [`EXECUTABLE`]: each string's address and text).
pub fn tables(keys: &Keys, archives: &[LodArchive<'_>], exe: &[u8]) -> Result<Tables, String> {
    let sources = || {
        keys.keys
            .values()
            .flat_map(|k| k.original.iter().map(|(_, source, _)| source))
    };
    let names: BTreeSet<&str> = sources().map(|s| s.table.as_str()).collect();
    let mut tables = BTreeMap::new();
    for name in names {
        if name == EXECUTABLE {
            let (_, image) =
                h3_formats::exe::identify(exe).map_err(|e| format!("Heroes3.exe: {e:?}"))?;
            let mut records = Vec::new();
            for source in sources() {
                if let Row::Address(va) = source.row {
                    let text = exe_string(exe, image.file_offset(va))
                        .ok_or_else(|| format!("Heroes3.exe: 0x{va:X}: not a string"))?;
                    records.push(vec![address_field(va), text]);
                }
            }
            tables.insert(name.to_string(), records);
            continue;
        }
        let file = format!("{name}.TXT");
        let (_, bytes) = files::find(archives, &file)?.ok_or_else(|| format!("no {file}"))?;
        let table = TextTable::parse(&bytes).map_err(|e| format!("{file}: {e:?}"))?;
        tables.insert(name.to_string(), table.records);
    }
    Ok(tables)
}

/// The executable's string at file offset `at`: at most 256 bytes to a NUL, each printable ASCII
/// or a space; none for any other or without the NUL.
fn exe_string(exe: &[u8], at: Option<usize>) -> Option<String> {
    let tail = exe.get(at?..)?;
    let len = tail.iter().take(256).position(|&b| b == 0)?;
    let bytes = &tail[..len];
    bytes
        .iter()
        .all(|b| b.is_ascii_graphic() || *b == b' ')
        .then(|| bytes.iter().map(|&b| char::from(b)).collect())
}

/// The panels' texts for the installation copy `install` (its `Heroes3.exe`) and its `archives`:
/// the original's pack of the viewer's [`keys`] of the catalogue beside the data folder `map`
/// ([`keys_beside`]); a form that cannot be read is an error.
pub fn catalogue(
    install: &Path,
    map: &Path,
    archives: &[LodArchive<'_>],
) -> Result<Catalogue, String> {
    let path = keys_beside(map);
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let ids = family_ids(&text, archives).map_err(|e| format!("{}: {e}", path.display()))?;
    let keys = keys(&text, &ids).map_err(|e| format!("{}: {e}", path.display()))?;
    let exe_path = crate::sounds::find_ci(install, "Heroes3.exe")
        .map_err(|e| format!("{}: Heroes3.exe: {e}", install.display()))?;
    let exe = fs::read(&exe_path).map_err(|e| format!("{}: {e}", exe_path.display()))?;
    let (pack, problems) = Pack::original(&keys, &tables(&keys, archives, &exe)?);
    match problems.first() {
        Some(p) => Err(format!("the original's texts: {p}")),
        None => Ok(Catalogue { packs: vec![pack] }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_viewers_keys_and_its_families_alone() {
        let text = fs::read_to_string(default_keys()).unwrap();
        let ids = BTreeMap::from([("hero".to_string(), 2), ("town-name".to_string(), 1)]);
        let keys = keys(&text, &ids).unwrap();
        assert_eq!(keys.keys.keys().map(String::as_str).collect::<Vec<_>>(), {
            let mut ids: Vec<&str> = panels::KEYS
                .into_iter()
                .chain(status::KEYS)
                .chain(hover::KEYS)
                .chain(hover::contents::KEYS)
                .collect();
            ids.extend(["hero.0", "hero.1", "town-name.0"]);
            ids.sort_unstable();
            ids
        });
        // Without ids a family has no keys; the battle's families none whatever `ids` says.
        let few = super::keys(&text, &BTreeMap::new()).unwrap();
        assert!(!few.keys.keys().any(|k| k.starts_with("hero.")));
        let missing = "[[key]]\nid = \"map.other\"\n";
        assert_eq!(keys_err(missing), "no key `map.resources.amount`");
        assert!(keys_err("[[key]]\nnot = 1\n").contains("unknown field"));
        assert!(keys_err("[[key]]\nid = \"Bad\"\n").contains("is not a key id"));
    }

    fn keys_err(text: &str) -> String {
        keys(text, &BTreeMap::new()).err().unwrap()
    }

    #[test]
    fn an_executables_string() {
        let exe = b"ab\0c\x01\0d e\0";
        assert_eq!(exe_string(exe, Some(0)).as_deref(), Some("ab"));
        assert_eq!(exe_string(exe, Some(6)).as_deref(), Some("d e"));
        assert_eq!(exe_string(exe, Some(3)), None);
        assert_eq!(exe_string(exe, None), None);
        assert_eq!(exe_string(exe, Some(99)), None);
        assert_eq!(exe_string(b"no end", Some(0)), None);
    }

    #[test]
    fn the_catalogue_beside_the_data_folder_given() {
        // A Windows build run in the user's checkout reads its copy beside `--map` (T7c1).
        let path = keys_beside(Path::new("out/win/map"));
        assert_eq!(path, Path::new("out/win/map/../text/keys.toml"));
        assert!(default_keys().is_file());
    }
}
