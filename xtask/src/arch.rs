//! `cargo xtask check-arch`: enforces the declared crate graph and the logic-crate purity policy
//! (ADR-0005, ADR-0003, `arch.toml`).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::scan;
use crate::util::{cargo, rel, workspace_root};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchConfig {
    pub policy: Policy,
    #[serde(rename = "crate")]
    pub crates: BTreeMap<String, CrateDecl>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub logic_attributes: Vec<String>,
    pub logic_forbid: Vec<String>,
    pub logic_deny: Vec<String>,
    #[serde(default)]
    pub lint_allow: Vec<LintAllow>,
    pub forbidden: Vec<Forbidden>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LintAllow {
    pub file: String,
    pub lint: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Forbidden {
    pub token: String,
    pub reason: String,
    #[serde(default)]
    pub allowed_in: Vec<String>,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Layer {
    Shared,
    Domain,
    Application,
    Adapter,
    Tool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CrateDecl {
    pub layer: Layer,
    pub logic: bool,
    pub deps: Vec<String>,
    pub external: Vec<String>,
    pub dev_deps: Vec<String>,
    pub dev_external: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepKind {
    Normal,
    Dev,
    Build,
}

#[derive(Debug, Clone)]
pub struct Dep {
    pub name: String,
    pub kind: DepKind,
}

/// A workspace member as reported by `cargo metadata`.
#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub dir: PathBuf,
    pub deps: Vec<Dep>,
}

pub fn load_config(root: &Path) -> Result<ArchConfig, String> {
    let path = root.join("arch.toml");
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    toml::from_str(&text).map_err(|e| format!("arch.toml: {e}"))
}

/// Reads the workspace members and their declared dependencies via `cargo metadata`.
pub fn load_packages() -> Result<Vec<Package>, String> {
    let output = cargo()
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--offline",
        ])
        .output()
        .map_err(|e| format!("cargo metadata: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    parse_metadata(&String::from_utf8_lossy(&output.stdout))
}

pub fn parse_metadata(json: &str) -> Result<Vec<Package>, String> {
    let v: serde_json::Value = serde_json::from_str(json).map_err(|e| format!("metadata: {e}"))?;
    let members: BTreeSet<&str> = v["workspace_members"]
        .as_array()
        .ok_or("metadata: no workspace_members")?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect();
    let mut packages = Vec::new();
    for p in v["packages"].as_array().ok_or("metadata: no packages")? {
        let id = p["id"].as_str().unwrap_or_default();
        if !members.contains(id) {
            continue;
        }
        let name = p["name"]
            .as_str()
            .ok_or("metadata: package without name")?
            .to_owned();
        let manifest = PathBuf::from(
            p["manifest_path"]
                .as_str()
                .ok_or("metadata: no manifest_path")?,
        );
        let dir = manifest.parent().map(Path::to_path_buf).unwrap_or_default();
        let mut deps = Vec::new();
        for d in p["dependencies"].as_array().into_iter().flatten() {
            let kind = match d["kind"].as_str() {
                Some("dev") => DepKind::Dev,
                Some("build") => DepKind::Build,
                _ => DepKind::Normal,
            };
            let dep_name = d["name"]
                .as_str()
                .ok_or("metadata: dependency without name")?
                .to_owned();
            deps.push(Dep {
                name: dep_name,
                kind,
            });
        }
        packages.push(Package { name, dir, deps });
    }
    packages.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(packages)
}

/// Checks the crate graph against the declaration. Returns human-readable violations.
pub fn check_graph(config: &ArchConfig, packages: &[Package]) -> Vec<String> {
    let mut violations = Vec::new();
    let members: BTreeSet<&str> = packages.iter().map(|p| p.name.as_str()).collect();
    for name in config.crates.keys() {
        if !members.contains(name.as_str()) {
            violations.push(format!(
                "arch.toml declares `{name}`, which is not a workspace member"
            ));
        }
    }
    for package in packages {
        let Some(decl) = config.crates.get(&package.name) else {
            violations.push(format!(
                "workspace member `{}` is not declared in arch.toml",
                package.name
            ));
            continue;
        };
        for dep in &package.deps {
            let is_workspace = members.contains(dep.name.as_str());
            let (allowed_ws, allowed_ext): (Vec<&String>, Vec<&String>) = match dep.kind {
                DepKind::Normal | DepKind::Build => {
                    (decl.deps.iter().collect(), decl.external.iter().collect())
                }
                DepKind::Dev => (
                    decl.deps.iter().chain(&decl.dev_deps).collect(),
                    decl.external.iter().chain(&decl.dev_external).collect(),
                ),
            };
            let kind = match dep.kind {
                DepKind::Normal => "dependency",
                DepKind::Build => "build-dependency",
                DepKind::Dev => "dev-dependency",
            };
            if is_workspace {
                if !allowed_ws.contains(&&dep.name) {
                    violations.push(format!(
                        "`{}` has undeclared {kind} on workspace crate `{}`",
                        package.name, dep.name
                    ));
                }
                if let Some(target) = config.crates.get(&dep.name)
                    && let Some(why) = layer_violation(decl.layer, target.layer)
                {
                    violations.push(format!(
                        "`{}` ({:?}) -> `{}` ({:?}): {why}",
                        package.name, decl.layer, dep.name, target.layer
                    ));
                }
            } else if !allowed_ext.contains(&&dep.name) {
                violations.push(format!(
                    "`{}` has undeclared external {kind} `{}` (declare it in arch.toml after review)",
                    package.name, dep.name
                ));
            }
        }
    }
    violations
}

/// Layer rule: dependencies point strictly downward, except shared->shared and adapter->adapter.
fn layer_violation(from: Layer, to: Layer) -> Option<&'static str> {
    if to == Layer::Tool {
        return Some("nothing may depend on tool crates");
    }
    if from == Layer::Tool {
        return None;
    }
    if from == Layer::Domain && to == Layer::Domain {
        return Some(
            "domain contexts must not depend on each other; use contracts and the game orchestrator",
        );
    }
    if to > from {
        return Some("dependencies must point to a lower layer");
    }
    if to == from && !matches!(from, Layer::Shared | Layer::Adapter) {
        return Some("same-layer dependencies are only allowed among shared or adapter crates");
    }
    None
}

/// Checks logic-crate sources: lint header and forbidden tokens.
pub fn check_sources(
    config: &ArchConfig,
    root: &Path,
    packages: &[Package],
) -> Result<Vec<String>, String> {
    let mut violations = Vec::new();
    for package in packages {
        let Some(decl) = config.crates.get(&package.name) else {
            continue;
        };
        if !decl.logic {
            continue;
        }
        violations.extend(check_header(
            config,
            root,
            &package.dir.join("src").join("lib.rs"),
        )?);
        for (file, is_test) in scan::classify_crate(&package.dir)? {
            let text = fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
            let rel_path = rel(root, &file);
            let scanned = scan::scan(&text, is_test);
            let mut exempt: BTreeSet<usize> = BTreeSet::new();
            for c in scanned
                .comments
                .iter()
                .filter(|c| c.text.contains("arch-ok:"))
            {
                if has_reference(&c.text) {
                    exempt.insert(c.line);
                } else {
                    violations.push(format!(
                        "{rel_path}:{}: `arch-ok:` needs a rule id or ADR reference (e.g. `// arch-ok: ADR-0003 reason`)",
                        c.line
                    ));
                }
            }
            let policy_lints: Vec<&String> = config
                .policy
                .logic_forbid
                .iter()
                .chain(&config.policy.logic_deny)
                .collect();
            for l in scan::lint_levels(&scanned) {
                let relaxes = matches!(l.level.as_str(), "allow" | "expect" | "warn");
                let allowed = config
                    .policy
                    .lint_allow
                    .iter()
                    .any(|a| a.file == rel_path && a.lint == l.lint);
                if relaxes && !l.in_test && policy_lints.contains(&&l.lint) && !allowed {
                    violations.push(format!(
                        "{rel_path}:{}: `{}({})` relaxes a logic-crate policy lint (see arch.toml [[policy.lint_allow]])",
                        l.line, l.level, l.lint
                    ));
                }
            }
            for rule in &config.policy.forbidden {
                if rule.allowed_in.iter().any(|a| a == &rel_path) {
                    continue;
                }
                for line in scan::find_pattern(&scanned, &rule.token) {
                    if exempt.contains(&line) {
                        continue;
                    }
                    violations.push(format!(
                        "{rel_path}:{line}: forbidden in logic crates: `{}` ({})",
                        rule.token, rule.reason
                    ));
                }
            }
        }
    }
    Ok(violations)
}

/// Checks the crate root of a logic crate for the required inner attributes and lint levels.
fn check_header(config: &ArchConfig, root: &Path, lib: &Path) -> Result<Vec<String>, String> {
    let text = fs::read_to_string(lib).map_err(|e| format!("{}: {e}", lib.display()))?;
    let scanned = scan::scan(&text, false);
    let at = rel(root, lib);
    let attributes = scan::inner_attributes(&scanned);
    let levels: Vec<scan::LintLevel> = scan::lint_levels(&scanned)
        .into_iter()
        .filter(|l| l.inner)
        .collect();
    let mut v = Vec::new();
    for a in &config.policy.logic_attributes {
        if !attributes.contains(a) {
            v.push(format!("{at}: logic crate root must have `#![{a}]`"));
        }
    }
    for lint in &config.policy.logic_forbid {
        if !levels
            .iter()
            .any(|l| l.level == "forbid" && &l.lint == lint)
        {
            v.push(format!("{at}: logic crate root must `#![forbid({lint})]`"));
        }
    }
    for lint in &config.policy.logic_deny {
        if !levels
            .iter()
            .any(|l| matches!(l.level.as_str(), "deny" | "forbid") && &l.lint == lint)
        {
            v.push(format!("{at}: logic crate root must deny `{lint}`"));
        }
    }
    Ok(v)
}

/// Whether an `arch-ok:` comment names a rule id (`BAT-DMG-001`) or an ADR (`ADR-0003`).
fn has_reference(comment: &str) -> bool {
    let after = comment.split("arch-ok:").nth(1).unwrap_or_default();
    let word = after.split_whitespace().next().unwrap_or_default();
    let parts: Vec<&str> = word.split('-').collect();
    let digits = |p: &str, n: usize| p.len() == n && p.chars().all(|c| c.is_ascii_digit());
    let upper = |p: &str| p.chars().all(|c| c.is_ascii_uppercase());
    match parts.as_slice() {
        ["ADR", n] => digits(n, 4),
        [ctx, area, n] => {
            ctx.len() == 3
                && upper(ctx)
                && (2..=4).contains(&area.len())
                && upper(area)
                && digits(n, 3)
        }
        _ => false,
    }
}

/// Directory for copies of the user's original game files (gitignored, ADR-0009).
pub const ORIGINAL_DATA_DIR: &str = "original-data/";

/// Violations for tracked files under [`ORIGINAL_DATA_DIR`] (given `git ls-files` output).
pub fn tracked_original_data(tracked: &str) -> Vec<String> {
    tracked
        .lines()
        .map(|l| l.trim().replace('\\', "/"))
        .filter(|l| l.starts_with(ORIGINAL_DATA_DIR))
        .map(|l| {
            format!(
                "{l}: original game files must never be committed (ADR-0009); `git rm --cached` it"
            )
        })
        .collect()
}

/// `git ls-files` of the workspace, or empty if this is not a git checkout.
fn git_tracked_files(root: &Path) -> String {
    std::process::Command::new("git")
        .current_dir(root)
        .args(["ls-files", "--", ORIGINAL_DATA_DIR])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

pub fn run(_args: &[String]) -> Result<(), String> {
    let root = workspace_root();
    let config = load_config(&root)?;
    let packages = load_packages()?;
    let mut violations = check_graph(&config, &packages);
    violations.extend(check_sources(&config, &root, &packages)?);
    violations.extend(tracked_original_data(&git_tracked_files(&root)));
    report("check-arch", &violations)
}

/// Prints violations and turns them into an error.
pub fn report(check: &str, violations: &[String]) -> Result<(), String> {
    if violations.is_empty() {
        println!("{check}: ok");
        return Ok(());
    }
    for v in violations {
        println!("  {v}");
    }
    Err(format!("{check}: {} violation(s)", violations.len()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::test_support::TempDir;

    fn real() -> (ArchConfig, Vec<Package>) {
        (
            load_config(&workspace_root()).unwrap(),
            load_packages().unwrap(),
        )
    }

    fn add_dep(packages: &mut [Package], from: &str, to: &str, kind: DepKind) {
        let p = packages.iter_mut().find(|p| p.name == from).unwrap();
        p.deps.push(Dep {
            name: to.to_owned(),
            kind,
        });
    }

    #[test]
    fn the_real_workspace_passes() {
        let (config, packages) = real();
        assert_eq!(check_graph(&config, &packages), Vec::<String>::new());
        assert_eq!(
            check_sources(&config, &workspace_root(), &packages).unwrap(),
            Vec::<String>::new()
        );
    }

    /// E01 exit criterion 4: a deliberately violating change is rejected.
    #[test]
    fn rejects_domain_to_adapter_and_undeclared_edges() {
        let (config, mut packages) = real();
        add_dep(&mut packages, "h3-battle", "h3-sim", DepKind::Normal);
        add_dep(&mut packages, "h3-kernel", "h3-battle", DepKind::Dev);
        add_dep(&mut packages, "h3-battle", "hashbrown", DepKind::Normal);
        add_dep(&mut packages, "h3-sim", "xtask", DepKind::Normal);
        let v = check_graph(&config, &packages);
        assert!(
            v.iter()
                .any(|m| m
                    .contains("`h3-battle` has undeclared dependency on workspace crate `h3-sim`")),
            "{v:?}"
        );
        assert!(
            v.iter()
                .any(|m| m.contains("h3-battle` (Domain) -> `h3-sim` (Adapter)")),
            "{v:?}"
        );
        assert!(
            v.iter()
                .any(|m| m.contains("`h3-kernel` has undeclared dev-dependency")),
            "{v:?}"
        );
        assert!(
            v.iter()
                .any(|m| m.contains("undeclared external dependency `hashbrown`")),
            "{v:?}"
        );
        assert!(
            v.iter()
                .any(|m| m.contains("nothing may depend on tool crates")),
            "{v:?}"
        );
    }

    #[test]
    fn layer_rules() {
        use Layer::*;
        assert!(layer_violation(Domain, Domain).is_some());
        assert!(layer_violation(Shared, Domain).is_some());
        assert!(layer_violation(Application, Application).is_some());
        assert!(layer_violation(Adapter, Adapter).is_none());
        assert!(layer_violation(Shared, Shared).is_none());
        assert!(layer_violation(Adapter, Domain).is_none());
        assert!(layer_violation(Tool, Adapter).is_none());
        assert!(layer_violation(Adapter, Tool).is_some());
    }

    #[test]
    fn undeclared_members_are_reported() {
        let (config, mut packages) = real();
        packages.push(Package {
            name: "h3-rogue".to_owned(),
            dir: PathBuf::new(),
            deps: Vec::new(),
        });
        packages.retain(|p| p.name != "h3-cli");
        let v = check_graph(&config, &packages);
        assert!(
            v.iter().any(|m| m.contains("`h3-rogue` is not declared")),
            "{v:?}"
        );
        assert!(
            v.iter()
                .any(|m| m.contains("declares `h3-cli`, which is not a workspace member")),
            "{v:?}"
        );
    }

    /// E01 exit criterion 4: forbidden APIs and a weakened lint header are rejected.
    #[test]
    fn rejects_forbidden_tokens_and_missing_header() {
        let (config, _) = real();
        let dir = TempDir::new("arch");
        let krate = dir.path().join("crates/h3-battle");
        dir.write(
            "crates/h3-battle/src/lib.rs",
            "#![no_std]\n// #![forbid(unsafe_code)] in a comment does not count\n\
             #![allow(clippy::unwrap_used, clippy::panic)]\nmod bad;\n",
        );
        dir.write(
            "crates/h3-battle/src/bad.rs",
            "use std::collections::HashMap;\nfn f() -> u32 { let x = 1.5; v.sort(); 0 }\n\
             fn g() { let t = 2.0; } // arch-ok: ADR-0003 exemption\n\
             #[cfg(test)]\nmod tests { fn h() { let m: HashMap<u8, u8>; let f = 0.5f64; } }\n\
             fn k() { let t = 2.0; } // arch-ok: because\n\
             #[allow(clippy::float_arithmetic)]\nfn m() { <[u8]>::sort_unstable_by_key(&mut [], |x| *x); }\n",
        );
        let packages = vec![Package {
            name: "h3-battle".to_owned(),
            dir: krate,
            deps: Vec::new(),
        }];
        let v = check_sources(&config, dir.path(), &packages).unwrap();
        let has = |needle: &str| v.iter().any(|m| m.contains(needle));
        assert!(has("must `#![forbid(unsafe_code)]`"), "{v:?}");
        assert!(has("must deny `clippy::unwrap_used`"), "{v:?}");
        assert!(
            has("lib.rs:3: `allow(clippy::unwrap_used)` relaxes"),
            "{v:?}"
        );
        assert!(has("bad.rs:1: forbidden in logic crates: `std`"), "{v:?}");
        assert!(
            has("bad.rs:1: forbidden in logic crates: `HashMap`"),
            "{v:?}"
        );
        assert!(has("bad.rs:2: forbidden in logic crates: `f64`"), "{v:?}");
        assert!(has("bad.rs:2: forbidden in logic crates: `sort`"), "{v:?}");
        assert!(
            !v.iter().any(|m| m.contains("bad.rs:3:")),
            "arch-ok with a reference exempts the line: {v:?}"
        );
        assert!(
            !v.iter().any(|m| m.contains("bad.rs:5:")),
            "test module is exempt: {v:?}"
        );
        assert!(has("bad.rs:6: `arch-ok:` needs a rule id"), "{v:?}");
        assert!(has("bad.rs:6: forbidden in logic crates: `f64`"), "{v:?}");
        assert!(
            has("bad.rs:7: `allow(clippy::float_arithmetic)` relaxes"),
            "{v:?}"
        );
        assert!(
            has("bad.rs:8: forbidden in logic crates: `sort_unstable_by_key`"),
            "{v:?}"
        );
        assert!(has_reference("// arch-ok: BAT-DMG-001 why"));
        assert!(!has_reference("// arch-ok: bat-dmg-001"));
        assert!(!has_reference("// arch-ok: ADR-12"));
    }

    #[test]
    fn original_data_must_not_be_tracked() {
        assert!(tracked_original_data("").is_empty());
        assert!(tracked_original_data("docs/x.md\ncrates/a/src/lib.rs\n").is_empty());
        let v = tracked_original_data("original-data/H3bitmap.lod\noriginal-data\\data\\x.txt\n");
        assert_eq!(v.len(), 2, "{v:?}");
        assert!(
            v[0].starts_with(
                "original-data/H3bitmap.lod: original game data must never be committed"
            )
        );
        // The real checkout has nothing tracked there.
        assert!(tracked_original_data(&git_tracked_files(&workspace_root())).is_empty());
    }

    #[test]
    fn parses_cargo_metadata_json() {
        let json = r#"{
            "workspace_members": ["a 0.1.0 (path+file:///w/a)"],
            "packages": [
              {"id": "a 0.1.0 (path+file:///w/a)", "name": "a", "manifest_path": "/w/a/Cargo.toml",
               "dependencies": [{"name": "b", "kind": null}, {"name": "c", "kind": "dev"}, {"name": "d", "kind": "build"}]},
              {"id": "z 1.0.0 (registry)", "name": "z", "manifest_path": "/r/z/Cargo.toml", "dependencies": []}
            ]}"#;
        let p = parse_metadata(json).unwrap();
        assert_eq!(p.len(), 1);
        assert_eq!(
            p[0].deps.iter().map(|d| d.kind).collect::<Vec<_>>(),
            [DepKind::Normal, DepKind::Dev, DepKind::Build]
        );
        assert!(parse_metadata("{}").is_err());
    }
}
