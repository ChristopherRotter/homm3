//! The client's pure crates stay pure (*provisional*, E17 M0). The engine's `cargo xtask
//! check-arch` does not see the client workspace. `no_std` makes I/O, clocks, threads and the
//! environment unavailable at compile time; this test keeps that in place: each pure library root
//! carries the header, no source file relaxes a denied lint or brings `std` in outside the tests,
//! and each pure crate depends only on what its allowlist names (a dependency's I/O would be
//! reachable from a `no_std` crate too).

use std::fs;
use std::path::{Path, PathBuf};

/// The pure crates (their libraries; `h3-view`'s binary under `src/bin/` is an adapter), each with
/// the dependencies it may have. A new dependency of a pure crate is added here deliberately.
const PURE: [(&str, &[&str]); 3] = [
    ("h3-assets", &["h3-formats"]),
    ("h3-render", &["h3-assets"]),
    ("h3-view", &["h3-assets", "h3-formats", "h3-render"]),
];

/// Attributes every pure crate root carries.
const REQUIRED: [&str; 2] = ["#![no_std]", "#![forbid(unsafe_code)]"];

/// Lints every pure crate root denies (the engine's logic header, `arch.toml` `logic_deny`).
const DENIED: [&str; 10] = [
    "clippy::unwrap_used",
    "clippy::expect_used",
    "clippy::panic",
    "clippy::todo",
    "clippy::unimplemented",
    "clippy::unreachable",
    "clippy::float_arithmetic",
    "clippy::print_stdout",
    "clippy::print_stderr",
    "clippy::exit",
];

fn crate_dir(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(name)
}

/// The `.rs` files under `dir`, `bin/` left out.
fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n != "bin") {
                sources(&path, out);
            }
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// The text of the root's `#![deny(...)]` attribute, or an empty string.
fn deny_block(source: &str) -> &str {
    source
        .split_once("#![deny(")
        .and_then(|(_, rest)| rest.split_once(")]"))
        .map_or("", |(block, _)| block)
}

/// The keys of the manifest's `[dependencies]` table.
fn dependencies(manifest: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let mut inside = false;
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            inside = line == "[dependencies]";
        } else if inside && !line.is_empty() && !line.starts_with('#') {
            let key = line.split(['=', '.']).next().unwrap_or_default().trim();
            keys.push(key.to_owned());
        }
    }
    keys
}

/// What is wrong with one source file of a pure crate.
fn violations(source: &str, root: bool) -> Vec<String> {
    let mut found = Vec::new();
    if root {
        for attribute in REQUIRED {
            if !source.contains(attribute) {
                found.push(format!("missing `{attribute}`"));
            }
        }
        let block = deny_block(source);
        for lint in DENIED {
            if !block.contains(lint) {
                found.push(format!("`{lint}` is not denied"));
            }
        }
    }
    let lines: Vec<&str> = source.lines().map(str::trim).collect();
    for (i, line) in lines.iter().enumerate() {
        let relaxes = ["allow(", "expect(", "warn("]
            .iter()
            .any(|a| line.contains(a));
        if line.starts_with("#") && relaxes && DENIED.iter().any(|l| line.contains(l)) {
            found.push(format!("line {}: relaxes a denied lint", i + 1));
        }
        if line.starts_with("extern crate std") && (i == 0 || lines[i - 1] != "#[cfg(test)]") {
            found.push(format!("line {}: `std` outside the tests", i + 1));
        }
    }
    found
}

#[test]
fn pure_crates_stay_pure() {
    let mut problems = Vec::new();
    for (name, allowed) in PURE {
        let dir = crate_dir(name);
        let mut files = Vec::new();
        sources(&dir.join("src"), &mut files);
        assert!(
            !files.is_empty(),
            "{name}: no sources found in {}",
            dir.display()
        );
        for file in files {
            let root = file.ends_with("src/lib.rs");
            for v in violations(&fs::read_to_string(&file).unwrap(), root) {
                problems.push(format!("{}: {v}", file.display()));
            }
        }
        let manifest = fs::read_to_string(dir.join("Cargo.toml")).unwrap();
        for dep in dependencies(&manifest) {
            if !allowed.contains(&dep.as_str()) {
                problems.push(format!(
                    "{name}: dependency `{dep}` is not in its allowlist"
                ));
            }
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn the_checks_catch_what_they_should() {
    assert_eq!(deny_block("#![deny(a, b)]\n"), "a, b");
    assert_eq!(deny_block("no attribute"), "");
    assert_eq!(violations("", true).len(), REQUIRED.len() + DENIED.len());
    assert_eq!(
        violations("#[allow(clippy::unwrap_used)]\nfn f() {}", false).len(),
        1
    );
    assert_eq!(
        violations("#[expect(clippy::panic, reason = \"x\")]", false).len(),
        1
    );
    assert_eq!(
        violations("#[allow(dead_code)]", false),
        Vec::<String>::new()
    );
    assert_eq!(violations("extern crate std;", false).len(), 1);
    assert!(violations("#[cfg(test)]\nextern crate std;", false).is_empty());
    let manifest = "[package]\nname = \"x\"\n[dependencies]\n# c\na.workspace = true\nb = \"1\"\n\n[dev-dependencies]\nc = \"1\"\n";
    assert_eq!(dependencies(manifest), ["a", "b"]);
}
