//! The files the commands read (E24 M1 T2, shared since T3b1): the default folders, the
//! committed tables and layout, the installation copy's four LOD archives and where the game
//! searches a name in them.

use std::fs;
use std::path::{Path, PathBuf};

use h3_formats::lod::LodArchive;
use h3_map_view::layout::AdventureLayout;
use h3_map_view::tables::{self, Tables};

/// The four LOD archives of the Complete edition.
pub const ARCHIVES: [&str; 4] = [
    "H3bitmap.lod",
    "H3sprite.lod",
    "H3ab_bmp.lod",
    "H3ab_spr.lod",
];

/// The table files, dependencies first, and the selection that holds them all.
const TABLES: [&str; 1] = ["base"];
const ALL: &str = "base";

/// The installation copy: `$H3_INSTALL`, else the repository's `original-data/install`.
pub fn default_install() -> PathBuf {
    // An adapter's ambient input, read once here.
    std::env::var_os("H3_INSTALL").map_or_else(
        || manifest().join("../../../original-data/install"),
        PathBuf::from,
    )
}

/// The map client's data folder (`clients/map`).
pub fn default_map() -> PathBuf {
    manifest().join("../../map")
}

fn manifest() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Reads and loads the committed tables of `<map>/assets`.
pub fn load_tables(map: &Path) -> Result<Tables, String> {
    let dir = map.join("assets");
    let mut files = Vec::new();
    for id in TABLES {
        let path = dir.join(format!("{id}.toml"));
        let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        files.push(tables::parse(&text).map_err(|e| format!("{}: {e}", path.display()))?);
    }
    tables::load(&files, ALL).map_err(|e| format!("{}: {e}", dir.display()))
}

/// Reads and checks `<map>/layouts/adventure.toml`.
pub fn load_layout(map: &Path) -> Result<AdventureLayout, String> {
    let path = map.join("layouts/adventure.toml");
    fs::read_to_string(&path)
        .map_err(|e| e.to_string())
        .and_then(|t| AdventureLayout::parse(&t))
        .map_err(|e| format!("{}: {e}", path.display()))
}

/// The four archives' bytes, in [`ARCHIVES`]' order.
pub fn read_archives(install: &Path) -> Result<Vec<Vec<u8>>, String> {
    let data = install.join("Data");
    ARCHIVES
        .iter()
        .map(|name| {
            let path = data.join(name);
            fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))
        })
        .collect()
}

/// The archives parsed.
pub fn parse_archives(bytes: &[Vec<u8>]) -> Result<Vec<LodArchive<'_>>, String> {
    bytes
        .iter()
        .zip(ARCHIVES)
        .map(|(b, name)| LodArchive::parse(b).map_err(|e| format!("{name}: {e}")))
        .collect()
}

/// Where the game searches a name (`docs/epics/E17-design.md` §3.1): DEFs in `H3sprite`, then
/// `H3ab_spr`; pictures and palettes in `H3bitmap`, then `H3ab_bmp`.
fn search(name: &str) -> [usize; 2] {
    if name.to_ascii_uppercase().ends_with(".DEF") {
        [1, 3]
    } else {
        [0, 2]
    }
}

/// The archive `name` is found in, as the game searches it, and its bytes.
pub fn find(
    archives: &[LodArchive<'_>],
    name: &str,
) -> Result<Option<(&'static str, Vec<u8>)>, String> {
    for a in search(name) {
        if let Some(read) = archives[a].read_by_name(name).map_err(|e| e.to_string())? {
            return Ok(Some((ARCHIVES[a], read)));
        }
    }
    Ok(None)
}
