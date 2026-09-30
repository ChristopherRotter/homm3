//! Golden replay fixtures (E01 exit criterion 2, ADR-0003 §11): every replay under
//! `fixtures/replays/` must reproduce its recorded state hashes on this platform and build
//! configuration. `cargo xtask determinism` runs this test in debug, release and release with
//! `-C target-cpu=native`; running it on another OS is the cross-platform check.

use std::path::PathBuf;

#[test]
fn golden_replays_reproduce_their_hashes() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir = root.join("fixtures/replays");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "toml"))
        .collect();
    files.sort();
    assert!(
        files.len() >= 3,
        "expected the E01 replay fixtures in {}",
        dir.display()
    );
    for file in &files {
        let verified = h3_cli::verify_file(file, &root.join("packs"));
        assert!(
            verified.is_ok(),
            "{}: {}",
            file.display(),
            verified.unwrap_err()
        );
    }
}
