//! The common crate stays pure (E24 M3 T1a; `h3-map-view`'s `tests/purity.rs`, E24 M1 T2, with the
//! fixes of its reviewer's R2-15, trimmed to a crate without a binary). The engine's `cargo xtask
//! check-arch` does not see the client workspace. `no_std` makes I/O, clocks, threads and the environment unavailable at compile
//! time; this test keeps that in place: the library root carries the header, no source file
//! relaxes a denied lint or brings `std` in outside the tests, and the crate depends only on what
//! its allowlist names (a dependency's I/O would be reachable from a `no_std` crate too).

use std::fs;
use std::path::{Path, PathBuf};

/// The pure crate with the dependencies it may have (it has no binary: every dependency is the
/// library's). A new dependency is added here deliberately: `h3-assets` the decoded DEF, `h3-render`
/// the compositor's drawer and `Bugs`, `serde`, `serde_json` and `toml` the layouts, the language
/// packs and the user's key file.
const PURE: [(&str, &[&str]); 1] = [(
    "h3-ui",
    &["h3-assets", "h3-render", "serde", "serde_json", "toml"],
)];

/// Attributes every pure crate root carries.
const REQUIRED: [&str; 2] = ["#![no_std]", "#![forbid(unsafe_code)]"];

/// The lint groups that hold a denied lint: relaxing one relaxes them (every lint of `DENIED` is
/// one of clippy's restriction lints; E24 M1 T6b, the reviewer's R2-15).
const GROUPS: [&str; 1] = ["clippy::restriction"];

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

/// The `.rs` files under `src`, every folder's (the crate has no binary).
fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Each attribute of a source (`#[...]` or `#![...]`, over as many lines as it takes) with its
/// first line's number.
fn attributes(lines: &[&str]) -> Vec<(usize, String)> {
    let mut found = Vec::new();
    let mut open: Option<(usize, String)> = None;
    for (i, line) in lines.iter().enumerate() {
        if open.is_none() && line.starts_with('#') {
            open = Some((i + 1, String::new()));
        }
        if let Some((_, text)) = &mut open {
            text.push_str(line);
            if line.ends_with(']') {
                found.extend(open.take());
            }
        }
    }
    found.extend(open);
    found
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
    let lines: Vec<&str> = source.lines().map(str::trim).collect();
    if root {
        // A line of its own each (a commented-out one is no attribute).
        for attribute in REQUIRED {
            if !lines.contains(&attribute) {
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
    for (line, attribute) in attributes(&lines) {
        let relaxes = ["allow(", "expect(", "warn("]
            .iter()
            .any(|a| attribute.contains(a));
        let names = DENIED.iter().chain(&GROUPS).any(|l| attribute.contains(l));
        if relaxes && names {
            found.push(format!("line {line}: relaxes a denied lint"));
        }
    }
    for (i, line) in lines.iter().enumerate() {
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
            let source = fs::read_to_string(&file).unwrap();
            for v in violations(&source, root) {
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
    // Over several lines, a group, a header commented out (E24 M1 T6b, R2-15).
    assert_eq!(
        violations(
            "#[allow(\n    dead_code,\n    clippy::panic\n)]\nfn f() {}",
            false
        ),
        ["line 1: relaxes a denied lint"]
    );
    assert_eq!(violations("#![allow(clippy::restriction)]", false).len(), 1);
    assert_eq!(
        violations("#[allow(\n  dead_code", false),
        Vec::<String>::new()
    );
    let commented = "// #![no_std]\n#![forbid(unsafe_code)]\n";
    assert!(
        violations(commented, true)
            .iter()
            .any(|v| v == "missing `#![no_std]`")
    );
    assert_eq!(violations("extern crate std;", false).len(), 1);
    assert!(violations("#[cfg(test)]\nextern crate std;", false).is_empty());
    let manifest = "[package]\nname = \"x\"\n[dependencies]\n# c\na.workspace = true\nb = \"1\"\n\n[dev-dependencies]\nc = \"1\"\n";
    assert_eq!(dependencies(manifest), ["a", "b"]);
}
