//! `cargo xtask check-arch`: enforces the declared crate graph and the logic-crate purity policy
//! (ADR-0005, ADR-0003, `arch.toml`).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::scan;
use crate::util::{cargo, rel, rs_files, workspace_root};

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
    pub logic_header: Vec<String>,
    pub forbidden: Vec<Forbidden>,
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
        let lib = package.dir.join("src").join("lib.rs");
        let lib_text = fs::read_to_string(&lib).map_err(|e| format!("{}: {e}", lib.display()))?;
        for required in &config.policy.logic_header {
            if !lib_text.contains(required.as_str()) {
                violations.push(format!(
                    "{}: logic crate header is missing `{required}` (see arch.toml [policy].logic_header)",
                    rel(root, &lib)
                ));
            }
        }
        for file in rs_files(&package.dir.join("src"))? {
            let text = fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
            let rel_path = rel(root, &file);
            let scanned = scan::scan(&text, scan::is_test_path(Path::new(&rel_path)));
            let exempt: BTreeSet<usize> = scanned
                .comments
                .iter()
                .filter(|c| c.text.contains("arch-ok:"))
                .map(|c| c.line)
                .collect();
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

pub fn run(_args: &[String]) -> Result<(), String> {
    let root = workspace_root();
    let config = load_config(&root)?;
    let packages = load_packages()?;
    let mut violations = check_graph(&config, &packages);
    violations.extend(check_sources(&config, &root, &packages)?);
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

    /// E01 exit criterion 4: forbidden APIs in a logic crate are rejected.
    #[test]
    fn rejects_forbidden_tokens_and_missing_header() {
        let (config, _) = real();
        let dir = TempDir::new("arch");
        let krate = dir.path().join("crates/h3-battle");
        dir.write(
            "crates/h3-battle/src/lib.rs",
            "#![no_std]\n//! missing the rest of the header\n",
        );
        dir.write(
            "crates/h3-battle/src/bad.rs",
            "use std::collections::HashMap;\nfn f() -> u32 { let x = 1.5; v.sort(); 0 }\n\
             fn g() { let t = 2.0; } // arch-ok: TEST exemption\n\
             #[cfg(test)]\nmod tests { fn h() { let m: HashMap<u8, u8>; let f = 0.5f64; } }\n",
        );
        let packages = vec![Package {
            name: "h3-battle".to_owned(),
            dir: krate,
            deps: Vec::new(),
        }];
        let v = check_sources(&config, dir.path(), &packages).unwrap();
        assert!(
            v.iter()
                .any(|m| m.contains("header is missing `#![forbid(unsafe_code)]`")),
            "{v:?}"
        );
        assert!(
            v.iter()
                .any(|m| m.contains("bad.rs:1: forbidden in logic crates: `std`")),
            "{v:?}"
        );
        assert!(
            v.iter()
                .any(|m| m.contains("bad.rs:1: forbidden in logic crates: `HashMap`")),
            "{v:?}"
        );
        assert!(
            v.iter()
                .any(|m| m.contains("bad.rs:2: forbidden in logic crates: `f64`")),
            "{v:?}"
        );
        assert!(
            v.iter()
                .any(|m| m.contains("bad.rs:2: forbidden in logic crates: `. sort`")),
            "{v:?}"
        );
        assert!(
            !v.iter().any(|m| m.contains("bad.rs:3:")),
            "arch-ok exempts the line: {v:?}"
        );
        assert!(
            !v.iter().any(|m| m.contains("bad.rs:5:")),
            "test module is exempt: {v:?}"
        );
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
