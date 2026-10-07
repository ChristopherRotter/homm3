//! `cargo xtask check-rules`: rule-catalog validity and traceability
//! (docs/04-rules-and-sources.md §3, ADR-0010).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::arch::report;
use crate::scan;
use crate::util::{rel, workspace_root};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogFile {
    #[serde(default)]
    pub rule: Vec<Rule>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub id: String,
    pub title: String,
    pub context: String,
    pub status: String,
    pub summary: String,
    #[serde(default)]
    pub quirks: Vec<String>,
    #[serde(default = "default_numeric")]
    pub numeric: String,
    #[serde(default)]
    pub source: Vec<Source>,
}

fn default_numeric() -> String {
    "integer".to_owned()
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub kind: String,
    #[serde(rename = "ref")]
    pub reference: String,
    pub url: Option<String>,
    pub retrieved: Option<String>,
    pub confidence: String,
    pub note: Option<String>,
}

const CONTEXTS: &[(&str, &str)] = &[
    ("BAT", "battle"),
    ("ADV", "adventure"),
    ("TWN", "town"),
    ("HER", "heroes"),
    ("MAG", "magic"),
    ("GAM", "game"),
    ("KER", "kernel"),
];
const STATUSES: &[&str] = &["specified", "implemented", "verified", "retired"];
const NUMERICS: &[&str] = &["integer", "float"];
const KINDS: &[&str] = &[
    "original-data",
    "observation",
    "reverse-engineering",
    "vcmi",
    "wiki",
    "community",
    "design",
];
const KINDS_NEEDING_URL: &[&str] = &["reverse-engineering", "vcmi", "wiki", "community"];
const CONFIDENCES: &[&str] = &["high", "medium", "low"];

/// One `rule:` or `covers:` marker found in the sources.
#[derive(Debug, Clone)]
pub struct Marker {
    pub file: String,
    pub line: usize,
    pub in_test: bool,
    pub id: String,
}

/// Loads all catalog files (sorted by name) as `(file name, parsed)`.
pub fn load_catalog(dir: &Path) -> Result<Vec<(String, CatalogFile)>, String> {
    let mut files = Vec::new();
    let entries = fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for entry in entries {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().is_some_and(|e| e == "toml") {
            files.push(path);
        }
    }
    files.sort();
    let mut out = Vec::new();
    for path in files {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let parsed: CatalogFile = toml::from_str(&text).map_err(|e| format!("{name}: {e}"))?;
        out.push((name, parsed));
    }
    Ok(out)
}

/// Quirk IDs declared as `### <id>` headings in a quirks ledger: `Q-NNNN` (from before the
/// lanes) or `<LANE CODE>-QUIRK-<n>` with a code from `codes` (lanes.toml).
pub fn ledger_ids(text: &str, codes: &BTreeSet<String>) -> BTreeSet<String> {
    text.lines()
        .filter_map(|l| l.strip_prefix("### "))
        .filter_map(|rest| rest.split_whitespace().next())
        .filter(|id| id.starts_with("Q-") || crate::lanes::is_lane_id(id, "QUIRK", codes))
        .map(str::to_owned)
        .collect()
}

/// The quirk IDs of every ledger: `docs/rules/quirks-ledger.md` (battle's) and the other lanes'
/// `docs/rules/quirks-<lane>.md`.
pub fn load_ledgers(root: &Path, codes: &BTreeSet<String>) -> Result<BTreeSet<String>, String> {
    let dir = root.join("docs/rules");
    let mut ids = BTreeSet::new();
    let entries = fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut files: Vec<_> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("quirks"))
                && p.extension().is_some_and(|e| e == "md")
        })
        .collect();
    files.sort();
    if files.is_empty() {
        return Err("docs/rules/: no quirks ledger".to_owned());
    }
    for path in files {
        let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        ids.extend(ledger_ids(&text, codes));
    }
    Ok(ids)
}

/// Collects `rule:` and `covers:` markers from all `.rs` files below `crates_dir`.
pub fn collect_markers(
    root: &Path,
    crates_dir: &Path,
) -> Result<(Vec<Marker>, Vec<Marker>), String> {
    let mut rules = Vec::new();
    let mut covers = Vec::new();
    for (file, is_test) in scan::classify(crates_dir)? {
        let text = fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
        let rel_path = rel(root, &file);
        let scanned = scan::scan(&text, is_test);
        for (line, in_test, id) in scan::markers(&scanned, "rule") {
            rules.push(Marker {
                file: rel_path.clone(),
                line,
                in_test,
                id,
            });
        }
        for (line, in_test, id) in scan::markers(&scanned, "covers") {
            covers.push(Marker {
                file: rel_path.clone(),
                line,
                in_test,
                id,
            });
        }
    }
    Ok((rules, covers))
}

fn valid_id(id: &str) -> Option<&'static str> {
    let mut parts = id.split('-');
    let (Some(ctx), Some(area), Some(num), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return None;
    };
    let area_ok = (2..=4).contains(&area.len()) && area.chars().all(|c| c.is_ascii_uppercase());
    let num_ok = num.len() == 3 && num.chars().all(|c| c.is_ascii_digit());
    if !area_ok || !num_ok {
        return None;
    }
    CONTEXTS.iter().find(|(p, _)| *p == ctx).map(|(_, c)| *c)
}

fn valid_date(d: &str) -> bool {
    let b = d.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

/// Validates the catalog and the markers. Returns violations.
pub fn check(
    catalog: &[(String, CatalogFile)],
    quirks: &BTreeSet<String>,
    rule_markers: &[Marker],
    cover_markers: &[Marker],
) -> Vec<String> {
    let mut v = Vec::new();
    let mut rules: BTreeMap<&str, &Rule> = BTreeMap::new();
    for (file, cat) in catalog {
        for r in &cat.rule {
            let at = format!("{file}: {}", r.id);
            if rules.insert(r.id.as_str(), r).is_some() {
                v.push(format!("{at}: duplicate rule id"));
            }
            match valid_id(&r.id) {
                None => v.push(format!("{at}: id must look like <CTX>-<AREA>-<NNN> with CTX in BAT/ADV/TWN/HER/MAG/GAM/KER")),
                Some(ctx) if ctx != r.context => {
                    v.push(format!("{at}: id prefix implies context `{ctx}` but context is `{}`", r.context));
                }
                Some(_) => {}
            }
            if r.title.trim().is_empty() || r.summary.trim().is_empty() {
                v.push(format!("{at}: title and summary must not be empty"));
            }
            if !STATUSES.contains(&r.status.as_str()) {
                v.push(format!("{at}: unknown status `{}`", r.status));
            }
            if !NUMERICS.contains(&r.numeric.as_str()) {
                v.push(format!(
                    "{at}: numeric must be integer or float, not `{}`",
                    r.numeric
                ));
            }
            for q in &r.quirks {
                if !quirks.contains(q) {
                    v.push(format!(
                        "{at}: quirk `{q}` is in no quirks ledger (docs/rules/quirks*.md)"
                    ));
                }
            }
            if r.source.is_empty() {
                v.push(format!("{at}: needs at least one source"));
            }
            for (i, s) in r.source.iter().enumerate() {
                let sat = format!("{at}: source #{}", i + 1);
                if !KINDS.contains(&s.kind.as_str()) {
                    v.push(format!("{sat}: unknown kind `{}`", s.kind));
                }
                if !CONFIDENCES.contains(&s.confidence.as_str()) {
                    v.push(format!("{sat}: unknown confidence `{}`", s.confidence));
                }
                if s.reference.trim().is_empty() {
                    v.push(format!("{sat}: ref must not be empty"));
                }
                if KINDS_NEEDING_URL.contains(&s.kind.as_str()) {
                    if !s
                        .url
                        .as_deref()
                        .is_some_and(|u| u.starts_with("https://") || u.starts_with("http://"))
                    {
                        v.push(format!("{sat}: kind `{}` needs an http(s) url", s.kind));
                    }
                    if !s.retrieved.as_deref().is_some_and(valid_date) {
                        v.push(format!(
                            "{sat}: kind `{}` needs retrieved = \"YYYY-MM-DD\"",
                            s.kind
                        ));
                    }
                }
                let _ = &s.note;
            }
        }
    }

    let mut implemented_by: BTreeMap<&str, usize> = BTreeMap::new();
    for m in rule_markers {
        let at = format!("{}:{}", m.file, m.line);
        match rules.get(m.id.as_str()) {
            None => v.push(format!("{at}: `rule: {}` references an unknown rule", m.id)),
            Some(r) => {
                *implemented_by.entry(r.id.as_str()).or_default() += 1;
                if m.in_test {
                    v.push(format!(
                        "{at}: `rule:` markers belong in implementation code; tests use `covers:`"
                    ));
                }
                if r.status == "specified" {
                    v.push(format!(
                        "{at}: rule {} has code but status is `specified`; set it to `implemented`",
                        m.id
                    ));
                }
                if r.status == "retired" {
                    v.push(format!(
                        "{at}: rule {} is retired but still referenced",
                        m.id
                    ));
                }
            }
        }
    }
    let mut covered_by: BTreeMap<&str, usize> = BTreeMap::new();
    for m in cover_markers {
        let at = format!("{}:{}", m.file, m.line);
        match rules.get(m.id.as_str()) {
            None => v.push(format!(
                "{at}: `covers: {}` references an unknown rule",
                m.id
            )),
            Some(r) => {
                *covered_by.entry(r.id.as_str()).or_default() += 1;
                if !m.in_test {
                    v.push(format!(
                        "{at}: `covers:` markers must be in test code (#[cfg(test)] or tests/)"
                    ));
                }
            }
        }
    }
    for r in rules.values() {
        if matches!(r.status.as_str(), "implemented" | "verified") {
            if !covered_by.contains_key(r.id.as_str()) {
                v.push(format!(
                    "{}: status `{}` but no test carries `covers: {}`",
                    r.id, r.status, r.id
                ));
            }
            if !implemented_by.contains_key(r.id.as_str()) {
                v.push(format!(
                    "{}: status `{}` but no code carries `rule: {}`",
                    r.id, r.status, r.id
                ));
            }
        }
    }
    v
}

pub fn run(_args: &[String]) -> Result<(), String> {
    let root = workspace_root();
    let catalog = load_catalog(&root.join("docs/rules/catalog"))?;
    let codes = crate::lanes::Config::load(&root)?.codes();
    let quirks = load_ledgers(&root, &codes)?;
    let (rule_markers, cover_markers) = collect_markers(&root, &root.join("crates"))?;
    let violations = check(&catalog, &quirks, &rule_markers, &cover_markers);
    let count: usize = catalog.iter().map(|(_, c)| c.rule.len()).sum();
    println!(
        "check-rules: {count} rule(s), {} rule marker(s), {} covers marker(s)",
        rule_markers.len(),
        cover_markers.len()
    );
    report("check-rules", &violations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::test_support::TempDir;

    const GOOD: &str = r#"
[[rule]]
id = "BAT-DMG-001"
title = "Base damage"
context = "battle"
status = "implemented"
summary = "Roll."
quirks = ["Q-0001"]
  [[rule.source]]
  kind = "wiki"
  ref = "Damage"
  url = "https://example.org/Damage"
  retrieved = "2026-09-30"
  confidence = "high"

[[rule]]
id = "KER-RNG-001"
title = "Uniform integers"
context = "kernel"
status = "specified"
summary = "Lemire."
  [[rule.source]]
  kind = "design"
  ref = "ADR-0003"
  confidence = "high"
"#;

    fn parse(text: &str) -> Vec<(String, CatalogFile)> {
        vec![("battle.toml".to_owned(), toml::from_str(text).unwrap())]
    }

    fn marker(id: &str, in_test: bool) -> Marker {
        Marker {
            file: "x.rs".to_owned(),
            line: 1,
            in_test,
            id: id.to_owned(),
        }
    }

    fn quirks() -> BTreeSet<String> {
        let codes = ["ADV".to_owned()].into();
        let ids = ledger_ids(
            "# Ledger\n### Q-0001 — A\n### Q-0002 — B\n### ADV-QUIRK-1 — C\n### XY-QUIRK-1 — D\n",
            &codes,
        );
        assert!(ids.contains("ADV-QUIRK-1") && !ids.contains("XY-QUIRK-1"));
        ids
    }

    #[test]
    fn good_catalog_with_markers_passes() {
        let v = check(
            &parse(GOOD),
            &quirks(),
            &[marker("BAT-DMG-001", false)],
            &[marker("BAT-DMG-001", true)],
        );
        assert_eq!(v, Vec::<String>::new());
    }

    /// E01 exit criterion 5: a rule without a test fails.
    #[test]
    fn implemented_rule_without_test_fails() {
        let v = check(
            &parse(GOOD),
            &quirks(),
            &[marker("BAT-DMG-001", false)],
            &[],
        );
        assert!(
            v.iter()
                .any(|m| m.contains("no test carries `covers: BAT-DMG-001`")),
            "{v:?}"
        );
        let v = check(&parse(GOOD), &quirks(), &[], &[marker("BAT-DMG-001", true)]);
        assert!(
            v.iter()
                .any(|m| m.contains("no code carries `rule: BAT-DMG-001`")),
            "{v:?}"
        );
    }

    /// E01 exit criterion 5: a marker for an unknown rule fails.
    #[test]
    fn unknown_markers_fail() {
        let v = check(
            &parse(GOOD),
            &quirks(),
            &[marker("BAT-DMG-001", false), marker("BAT-XXX-999", false)],
            &[marker("BAT-DMG-001", true), marker("BAT-YYY-001", true)],
        );
        assert!(
            v.iter()
                .any(|m| m.contains("`rule: BAT-XXX-999` references an unknown rule")),
            "{v:?}"
        );
        assert!(
            v.iter()
                .any(|m| m.contains("`covers: BAT-YYY-001` references an unknown rule")),
            "{v:?}"
        );
    }

    #[test]
    fn misplaced_markers_and_status_drift_fail() {
        let v = check(
            &parse(GOOD),
            &quirks(),
            &[marker("BAT-DMG-001", true), marker("KER-RNG-001", false)],
            &[marker("BAT-DMG-001", false)],
        );
        assert!(
            v.iter()
                .any(|m| m.contains("`rule:` markers belong in implementation code")),
            "{v:?}"
        );
        assert!(
            v.iter()
                .any(|m| m.contains("`covers:` markers must be in test code")),
            "{v:?}"
        );
        assert!(
            v.iter()
                .any(|m| m.contains("rule KER-RNG-001 has code but status is `specified`")),
            "{v:?}"
        );
    }

    #[test]
    fn malformed_rules_fail() {
        let bad = r#"
[[rule]]
id = "BAT-DMG-1"
title = ""
context = "adventure"
status = "done"
summary = "x"
numeric = "fixed"
quirks = ["Q-9999"]
  [[rule.source]]
  kind = "blog"
  ref = ""
  confidence = "sure"
  [[rule.source]]
  kind = "wiki"
  ref = "Page"
  url = "ftp://nope"
  retrieved = "30.09.2026"
  confidence = "low"

[[rule]]
id = "ADV-MOV-001"
title = "t"
context = "battle"
status = "retired"
summary = "s"

[[rule]]
id = "ADV-MOV-001"
title = "t"
context = "adventure"
status = "specified"
summary = "s"
  [[rule.source]]
  kind = "design"
  ref = "ADR"
  confidence = "high"
"#;
        let v = check(&parse(bad), &quirks(), &[marker("ADV-MOV-001", false)], &[]);
        for needle in [
            "id must look like",
            "title and summary must not be empty",
            "unknown status `done`",
            "numeric must be integer or float",
            "quirk `Q-9999`",
            "unknown kind `blog`",
            "unknown confidence `sure`",
            "ref must not be empty",
            "needs an http(s) url",
            "needs retrieved",
            "id prefix implies context `adventure` but context is `battle`",
            "needs at least one source",
            "duplicate rule id",
        ] {
            assert!(
                v.iter().any(|m| m.contains(needle)),
                "missing `{needle}` in {v:?}"
            );
        }
    }

    #[test]
    fn helpers() {
        assert_eq!(valid_id("BAT-DMG-001"), Some("battle"));
        assert_eq!(valid_id("KER-RNG-010"), Some("kernel"));
        assert_eq!(valid_id("XYZ-DMG-001"), None);
        assert_eq!(valid_id("BAT-d-001"), None);
        assert_eq!(valid_id("BAT-DMG-001-2"), None);
        assert!(valid_date("2026-09-30"));
        assert!(!valid_date("2026-9-30"));
        assert_eq!(quirks().len(), 3);
    }

    #[test]
    fn collects_markers_and_loads_catalog_from_disk() {
        let dir = TempDir::new("rules");
        dir.write("crates/a/src/lib.rs", "// rule: BAT-DMG-001\nfn f() {}\n#[cfg(test)]\nmod tests {\n // covers: BAT-DMG-001\n}\n");
        dir.write("crates/a/tests/t.rs", "// covers: KER-RNG-001\n");
        dir.write("catalog/battle.toml", GOOD);
        dir.write("catalog/README.md", "not toml");
        let (r, c) = collect_markers(dir.path(), &dir.path().join("crates")).unwrap();
        assert_eq!(r.len(), 1);
        assert!(!r[0].in_test);
        assert_eq!(c.len(), 2);
        assert!(c.iter().all(|m| m.in_test));
        let cat = load_catalog(&dir.path().join("catalog")).unwrap();
        assert_eq!(cat.len(), 1);
        assert_eq!(cat[0].1.rule.len(), 2);
    }

    #[test]
    fn the_real_catalog_passes() {
        let root = workspace_root();
        let catalog = load_catalog(&root.join("docs/rules/catalog")).unwrap();
        let codes = crate::lanes::Config::load(&root).unwrap().codes();
        let quirks = load_ledgers(&root, &codes).unwrap();
        assert!(quirks.contains("Q-0001"));
        let (r, c) = collect_markers(&root, &root.join("crates")).unwrap();
        assert_eq!(check(&catalog, &quirks, &r, &c), Vec::<String>::new());
    }
}
