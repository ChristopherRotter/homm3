//! The asset tables in the adapters (E17 M1 T4): reading `clients/assets/*.toml` and the real-asset
//! gate's check of every name they hold against the archives.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use h3_view::tables::{self, Tables};

/// The table files, dependencies first, and the selection that holds them all.
pub const TABLES: [&str; 3] = ["base", "ab", "sod"];
pub const ALL: &str = "sod";

/// The committed tables' directory.
pub fn default_dir() -> PathBuf {
    clients_dir().join("assets")
}

/// The client workspace (`clients/`): where it was built, or else the first directory above the
/// executable that holds the asset tables (an executable copied into a checkout, such as a
/// Windows build run on the user's machine; E17 M2 T5a), or else the current directory.
pub fn clients_dir() -> PathBuf {
    let built = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let holds = |d: &Path| d.join("assets").join("base.toml").is_file();
    if holds(&built) {
        return built;
    }
    std::env::current_exe()
        .ok()
        .and_then(|exe| {
            exe.ancestors()
                .skip(1)
                .find(|d| holds(d))
                .map(Path::to_path_buf)
        })
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Reads and loads the selection `top` from `dir`.
pub fn load(dir: &Path, top: &str) -> Result<Tables, String> {
    let mut files = Vec::new();
    for id in TABLES {
        let path = dir.join(format!("{id}.toml"));
        let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        files.push(tables::parse(&text).map_err(|e| format!("{}: {e}", path.display()))?);
    }
    tables::load(&files, top).map_err(|e| format!("{}: {e}", dir.display()))
}

/// Where the game searches a name (`docs/epics/E17-design.md` §3.1): DEFs in `H3sprite`, then
/// `H3ab_spr`; pictures in `H3bitmap`, then `H3ab_bmp` (indices into the gate's archive list).
pub fn search(name: &str) -> &'static [usize] {
    if name.ends_with(".DEF") {
        &[1, 3]
    } else {
        &[0, 2]
    }
}

/// Checks every name of `tables`: found in the first archive of its search list that holds it,
/// and decoded there. `decoded` maps (archive, upper-cased name) to whether it decoded.
pub fn check(tables: &Tables, decoded: &BTreeMap<(usize, String), bool>) -> (usize, Vec<String>) {
    let names = tables.names();
    let mut problems = Vec::new();
    for name in &names {
        let found = search(name)
            .iter()
            .find_map(|&a| decoded.get(&(a, name.clone())));
        match found {
            Some(true) => {}
            Some(false) => problems.push(format!("{name}: does not decode")),
            None => problems.push(format!("{name}: in no archive")),
        }
    }
    (names.len(), problems)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_taken_from_the_first_archive_that_holds_it() {
        let tables = load(&default_dir(), ALL).unwrap();
        let mut decoded: BTreeMap<(usize, String), bool> = tables
            .names()
            .into_iter()
            .map(|n| ((search(&n)[1], n), true))
            .collect();
        assert_eq!(check(&tables, &decoded).1, Vec::<String>::new());
        // The base archive's copy wins, decoded or not.
        decoded.insert((1, "CPKMAN.DEF".to_string()), false);
        decoded.remove(&(2, "CMBKDRDD.PCX".to_string()));
        let (_, problems) = check(&tables, &decoded);
        assert_eq!(
            problems,
            ["CMBKDRDD.PCX: in no archive", "CPKMAN.DEF: does not decode"]
        );
    }
}
