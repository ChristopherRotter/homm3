//! The common crate stays pure (E24 M3 T1a; `h3-map-view`'s `tests/purity.rs`, E24 M1 T2, with the
//! fixes of its reviewer's R2-15, trimmed to a crate without a binary; with `h3-view`'s fixes of
//! E17 M6 T6a's reviewer and T7b's R11-3, ported in T7b, R11-2). The engine's `cargo xtask
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

/// A line's code: the line up to its `//` comment (one inside a string literal is code), and its
/// brackets' balance outside string literals (`[` counts 1, `]` −1).
fn code_of(line: &str) -> (&str, i32) {
    let (mut quoted, mut escaped, mut depth) = (false, false, 0);
    let bytes = line.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            _ if escaped => escaped = false,
            b'\\' if quoted => escaped = true,
            b'"' => quoted = !quoted,
            b'/' if !quoted && bytes.get(i + 1) == Some(&b'/') => return (&line[..i], depth),
            b'[' if !quoted => depth += 1,
            b']' if !quoted => depth -= 1,
            _ => {}
        }
    }
    (line, depth)
}

/// Each attribute of a source (`#[...]` or `#![...]`, over as many lines as it takes: until its
/// brackets close, comments left out; E17 M6 T6a's reviewer, ported in T7b, R11-2: a `]` in a
/// comment closed it) with its first line's number.
fn attributes(lines: &[&str]) -> Vec<(usize, String)> {
    let mut found = Vec::new();
    let mut open: Option<(usize, String, i32)> = None;
    for (i, line) in lines.iter().enumerate() {
        let (code, balance) = code_of(line);
        if open.is_none() && code.starts_with('#') {
            open = Some((i + 1, String::new(), 0));
        }
        if let Some((_, text, depth)) = &mut open {
            text.push_str(code);
            *depth += balance;
            if *depth <= 0 {
                found.extend(open.take().map(|(at, text, _)| (at, text)));
            }
        }
    }
    found.extend(open.map(|(at, text, _)| (at, text)));
    found
}

/// The text of the root's `#![deny(...)]` attribute (its lines joined, comments left out; a
/// commented-out one is none: E17 M6 T6a's reviewer, ported in T7b, R11-2), or an empty string.
fn deny_block(lines: &[&str]) -> String {
    attributes(lines)
        .into_iter()
        .find_map(|(_, attribute)| {
            let rest = attribute.strip_prefix("#![deny(")?;
            Some(rest.trim_end_matches(']').trim_end_matches(')').to_owned())
        })
        .unwrap_or_default()
}

/// A TOML key path's parts (`target.'cfg(unix)'.dependencies` gives `target`, `cfg(unix)`,
/// `dependencies`): split at the dots outside quotes, the quotes and spaces left out.
fn parts(path: &str) -> Vec<String> {
    let mut out = vec![String::new()];
    let mut quote = None;
    for c in path.chars() {
        match (quote, c) {
            (None, '\'' | '"') => quote = Some(c),
            (Some(q), _) if c == q => quote = None,
            (None, '.') => out.push(String::new()),
            (None, c) if c.is_whitespace() => {}
            _ => {
                if let Some(part) = out.last_mut() {
                    part.push(c);
                }
            }
        }
    }
    out
}

/// The dependency a full key path names: `dependencies.<key>` or
/// `target.<cfg>.dependencies.<key>`, and anything under either.
fn dependency(path: &[String]) -> Option<&str> {
    match path {
        [table, key, ..] if table == "dependencies" => Some(key),
        [target, _, table, key, ..] if target == "target" && table == "dependencies" => Some(key),
        _ => None,
    }
}

/// The keys of the manifest's dependencies: the `[dependencies]` table's, a
/// `[dependencies.<key>]` table's and a `[target.<cfg>.dependencies]` table's (E17 M6 T7b, R11-3:
/// only the first was read).
fn dependencies(manifest: &str) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    let mut table = Vec::new();
    for line in manifest.lines().map(str::trim) {
        let path = if let Some(header) = line.strip_prefix('[') {
            let header = header.trim_start_matches('[');
            table = parts(header.split_once(']').map_or(header, |(h, _)| h));
            table.clone()
        } else if let Some((key, _)) = line.split_once('=').filter(|_| !line.starts_with('#')) {
            table.iter().cloned().chain(parts(key)).collect()
        } else {
            continue;
        };
        if let Some(key) = dependency(&path)
            && !keys.iter().any(|k| k == key)
        {
            keys.push(key.to_owned());
        }
    }
    keys
}

/// The lints a deny block names, each exactly (E17 M6 T7b, R11-3: `clippy::panic_in_result_fn`
/// passed for `clippy::panic`).
fn names(block: &str) -> Vec<&str> {
    block.split(',').map(str::trim).collect()
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
        let block = deny_block(&lines);
        let denied = names(&block);
        for lint in DENIED {
            if !denied.contains(&lint) {
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
    assert_eq!(deny_block(&["#![deny(a, b)]"]), "a, b");
    assert_eq!(deny_block(&["no attribute"]), "");
    assert_eq!(
        deny_block(&["#![deny(", "a, // the first", "b", ")]"]),
        "a, b"
    );
    // A deny block commented out denies nothing (E17 M6 T6a's reviewer, ported in T7b, R11-2).
    assert_eq!(deny_block(&["// #![deny(a)]", "#![forbid(b)]"]), "");
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
    // One left open to the end still counts; a `]` in a comment does not close one; a `//` in a
    // string is no comment (E17 M6 T6a's reviewer, ported in T7b, R11-2).
    assert_eq!(violations("#[allow(\n  clippy::panic", false).len(), 1);
    assert_eq!(
        violations(
            "#[allow(\n    dead_code, // see [1]\n    clippy::panic\n)]",
            false
        ),
        ["line 1: relaxes a denied lint"]
    );
    assert_eq!(
        violations(
            "#[doc = \"http://a\"]\n#[allow(clippy::panic)]\nfn f() {}",
            false
        ),
        ["line 2: relaxes a denied lint"]
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
    // A dependency's own table, a target's, a dotted key at the root; not a dev-dependency or
    // the workspace's (E17 M6 T7b, R11-3).
    let tables = "dotted.dependencies = 1\ndependencies.h = \"1\"\n[package]\nname = \"x\"\n\
        [dependencies.d] # c\nversion = \"1\"\nfeatures = [\"x\"]\n\
        [target.'cfg(unix)'.dependencies]\ne = { version = \"1\" }\n\
        [target.\"cfg(any(unix, windows))\".dependencies.\"f\"]\nversion = \"1\"\n\
        [target.x.dev-dependencies]\ng = \"1\"\n[dev-dependencies.i]\nversion = \"1\"\n\
        [workspace.dependencies]\nj = \"1\"\n[[bin]]\nname = \"k\"\n# dependencies.l = 1\n";
    assert_eq!(dependencies(tables), ["h", "d", "e", "f"]);
    // The denied lints named exactly: a longer one with the same start is another.
    let lookalike = "#![no_std]\n#![forbid(unsafe_code)]\n#![deny(clippy::panic_in_result_fn)]\n";
    assert!(
        violations(lookalike, true)
            .iter()
            .any(|v| v == "`clippy::panic` is not denied")
    );
    assert_eq!(
        names(&deny_block(&["#![deny(", "a,", "b", ")]"])),
        ["a", "b"]
    );
}
