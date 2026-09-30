//! `cargo xtask check-stubs`: no silent debt (docs/08-work-process.md, ADR-0010).
//!
//! Every `todo!`, `unimplemented!`, `#[ignore]` and `TODO`/`FIXME` comment in `crates/` must carry
//! a `// stub: STUB-NNNN` marker on the same line or the line above, and the id must be registered
//! in `docs/stubs.toml`. Every registered stub must still have a marker in the code.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::arch::report;
use crate::scan::{self, Scanned};
use crate::util::{rel, rs_files, workspace_root};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    #[serde(default)]
    pub stub: Vec<Stub>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stub {
    pub id: String,
    #[serde(rename = "where")]
    pub location: String,
    pub reason: String,
    pub epic: String,
}

/// Debt markers found in one file: `(line, what)`.
pub fn debt_sites(scanned: &Scanned) -> Vec<(usize, &'static str)> {
    let mut out = Vec::new();
    let toks = &scanned.tokens;
    for (i, t) in toks.iter().enumerate() {
        let next = toks.get(i + 1).map(|t| t.text.as_str());
        match (t.text.as_str(), next) {
            ("todo", Some("!")) => out.push((t.line, "todo!")),
            ("unimplemented", Some("!")) => out.push((t.line, "unimplemented!")),
            ("#", Some("[")) if toks.get(i + 2).is_some_and(|n| n.text == "ignore") => {
                out.push((t.line, "#[ignore]"));
            }
            _ => {}
        }
    }
    for c in &scanned.comments {
        let has_word = |w: &str| {
            c.text
                .split(|ch: char| !ch.is_ascii_alphanumeric())
                .any(|part| part == w)
        };
        if has_word("TODO") {
            out.push((c.line, "TODO"));
        } else if has_word("FIXME") {
            out.push((c.line, "FIXME"));
        }
    }
    out.sort_unstable();
    out
}

/// `stub: STUB-NNNN` markers anywhere inside comments (e.g. `// TODO(E03) stub: STUB-0001`).
fn stub_markers(scanned: &Scanned) -> Vec<(usize, bool, String)> {
    let mut out = Vec::new();
    for c in &scanned.comments {
        let mut rest = c.text.as_str();
        while let Some(pos) = rest.find("stub:") {
            rest = &rest[pos + "stub:".len()..];
            let id: String = rest
                .trim_start()
                .chars()
                .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '-')
                .collect();
            if !id.is_empty() {
                out.push((c.line, c.in_test, id));
            }
        }
    }
    out
}

fn valid_stub_id(id: &str) -> bool {
    id.strip_prefix("STUB-")
        .is_some_and(|n| n.len() == 4 && n.chars().all(|c| c.is_ascii_digit()))
}

/// Checks one scanned file against the registry. `used` collects the ids that were referenced.
pub fn check_file(
    file: &str,
    scanned: &Scanned,
    registered: &BTreeSet<&str>,
    used: &mut BTreeSet<String>,
) -> Vec<String> {
    let mut v = Vec::new();
    let markers = stub_markers(scanned);
    let marker_lines: BTreeMap<usize, Vec<&str>> =
        markers
            .iter()
            .fold(BTreeMap::new(), |mut m, (line, _, id)| {
                m.entry(*line).or_insert_with(Vec::new).push(id.as_str());
                m
            });
    for (line, _, id) in &markers {
        if !registered.contains(id.as_str()) {
            v.push(format!(
                "{file}:{line}: `stub: {id}` is not registered in docs/stubs.toml"
            ));
        }
        used.insert(id.clone());
    }
    for (line, what) in debt_sites(scanned) {
        let marked = marker_lines.contains_key(&line)
            || (line > 1 && marker_lines.contains_key(&(line - 1)));
        if !marked {
            v.push(format!(
                "{file}:{line}: `{what}` without a `// stub: STUB-NNNN` marker (register it in docs/stubs.toml)"
            ));
        }
    }
    v
}

pub fn check(registry: &Registry, files: &[(String, Scanned)]) -> Vec<String> {
    let mut v = Vec::new();
    let mut registered = BTreeSet::new();
    for s in &registry.stub {
        if !valid_stub_id(&s.id) {
            v.push(format!(
                "docs/stubs.toml: `{}` must look like STUB-NNNN",
                s.id
            ));
        }
        if !registered.insert(s.id.as_str()) {
            v.push(format!("docs/stubs.toml: duplicate stub id `{}`", s.id));
        }
        if s.location.trim().is_empty() || s.reason.trim().is_empty() || s.epic.trim().is_empty() {
            v.push(format!(
                "docs/stubs.toml: `{}` needs where, reason and epic",
                s.id
            ));
        }
    }
    let mut used = BTreeSet::new();
    for (file, scanned) in files {
        v.extend(check_file(file, scanned, &registered, &mut used));
    }
    for id in &registered {
        if !used.contains(*id) {
            v.push(format!("docs/stubs.toml: `{id}` has no `// stub: {id}` marker in the code (remove it if done)"));
        }
    }
    v
}

pub fn load_registry(path: &Path) -> Result<Registry, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    toml::from_str(&text).map_err(|e| format!("docs/stubs.toml: {e}"))
}

pub fn scan_crates(root: &Path, crates_dir: &Path) -> Result<Vec<(String, Scanned)>, String> {
    let mut out = Vec::new();
    for file in rs_files(crates_dir)? {
        let text = fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
        let rel_path = rel(root, &file);
        let scanned = scan::scan(&text, scan::is_test_path(Path::new(&rel_path)));
        out.push((rel_path, scanned));
    }
    Ok(out)
}

pub fn run(_args: &[String]) -> Result<(), String> {
    let root = workspace_root();
    let registry = load_registry(&root.join("docs/stubs.toml"))?;
    let files = scan_crates(&root, &root.join("crates"))?;
    let violations = check(&registry, &files);
    println!("check-stubs: {} registered stub(s)", registry.stub.len());
    report("check-stubs", &violations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::test_support::TempDir;

    fn registry(text: &str) -> Registry {
        toml::from_str(text).unwrap()
    }

    const ONE: &str =
        "[[stub]]\nid = \"STUB-0001\"\nwhere = \"a.rs::f\"\nreason = \"later\"\nepic = \"E03\"\n";

    fn files(src: &str) -> Vec<(String, Scanned)> {
        vec![("a.rs".to_owned(), scan::scan(src, false))]
    }

    #[test]
    fn registered_and_marked_debt_passes() {
        let src =
            "fn f() {\n    // stub: STUB-0001\n    todo!()\n}\n// TODO(later) stub: STUB-0001\n";
        let v = check(&registry(ONE), &files(src));
        assert!(v.is_empty(), "{v:?}");
    }

    #[test]
    fn unmarked_debt_fails() {
        let src = "fn f() { todo!() }\nfn g() { unimplemented!() }\n#[test]\n#[ignore]\nfn h() {}\n// TODO: later\n// FIXME\n";
        let v = check(&registry("stub = []"), &files(src));
        for needle in [
            "`todo!`",
            "`unimplemented!`",
            "`#[ignore]`",
            "`TODO`",
            "`FIXME`",
        ] {
            assert!(
                v.iter().any(|m| m.contains(needle)),
                "missing {needle}: {v:?}"
            );
        }
    }

    #[test]
    fn registry_consistency() {
        // Marker for an unregistered id; registered id without a marker; malformed entries.
        let v = check(&registry(ONE), &files("// stub: STUB-0002\nfn f() {}\n"));
        assert!(
            v.iter()
                .any(|m| m.contains("`stub: STUB-0002` is not registered")),
            "{v:?}"
        );
        assert!(
            v.iter()
                .any(|m| m.contains("`STUB-0001` has no `// stub: STUB-0001` marker")),
            "{v:?}"
        );
        let bad = "[[stub]]\nid = \"S-1\"\nwhere = \"\"\nreason = \"r\"\nepic = \"E\"\n[[stub]]\nid = \"S-1\"\nwhere = \"w\"\nreason = \"r\"\nepic = \"E\"\n";
        let v = check(&registry(bad), &files("// stub: S-1\n"));
        assert!(
            v.iter().any(|m| m.contains("must look like STUB-NNNN")),
            "{v:?}"
        );
        assert!(v.iter().any(|m| m.contains("duplicate stub id")), "{v:?}");
        assert!(
            v.iter().any(|m| m.contains("needs where, reason and epic")),
            "{v:?}"
        );
    }

    #[test]
    fn words_not_substrings() {
        // `TODOS`, `todo_list` and strings do not count.
        let v = check(
            &registry("stub = []"),
            &files("// TODOS are fine\nfn todo_list() {}\nlet s = \"TODO\";\n"),
        );
        assert!(v.is_empty(), "{v:?}");
    }

    #[test]
    fn scans_from_disk_and_real_registry_passes() {
        let dir = TempDir::new("stubs");
        dir.write("crates/a/src/lib.rs", "fn f() { todo!() }\n");
        let found = scan_crates(dir.path(), &dir.path().join("crates")).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(check(&registry("stub = []"), &found).len(), 1);

        let root = workspace_root();
        let reg = load_registry(&root.join("docs/stubs.toml")).unwrap();
        let files = scan_crates(&root, &root.join("crates")).unwrap();
        assert_eq!(check(&reg, &files), Vec::<String>::new());
    }
}
