//! `cargo xtask check-lane`: a branch changes only the paths of its lane (`lanes.toml`,
//! docs/08-work-process.md, "Lanes").
//!
//! Each tracked path has one owner: a lane, `shared` (changed only by separate pull requests that
//! merge first) or `common` (any lane). The longest matching pattern in `lanes.toml` decides.
//! The check compares the working tree (commits and uncommitted changes, untracked files
//! included) with its merge base on `master`:
//!
//! - a path no pattern matches fails (add it to `lanes.toml` first);
//! - with a declared lane (`--lane`, or a branch named `<lane>/…` or `shared/…`), every changed
//!   path must belong to that lane or to `common`;
//! - without one, the lane is inferred: the change may touch one lane, or shared paths, but not
//!   two lanes and not a lane and shared paths together;
//! - `--cross-lane` (CI: the pull request label `cross-lane`) accepts any owned path and only
//!   reports what was touched; it is for deliberate cross-lane changes the user reviews.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::util::{git, workspace_root};

/// The ownership file at the workspace root.
pub const LANES_FILE: &str = "lanes.toml";

/// Branch prefix (and `--lane` value) of a pull request that changes shared paths.
pub const SHARED: &str = "shared";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub shared: PathSet,
    pub common: PathSet,
    #[serde(default)]
    pub lane: Vec<Lane>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathSet {
    pub paths: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lane {
    /// Lane name: branch prefix, `docs/lanes/<name>/`, `benches/<name>/`.
    pub name: String,
    /// Prefix of the lane's new question, quirk and stub ids (`ADV-Q1`, `ADV-QUIRK-1`,
    /// `ADV-STUB-1`).
    pub code: String,
    pub paths: Vec<String>,
    /// Lanes whose crates, golden replays and benchmark workloads the lane's scoped verify runs
    /// too: a lane that owns sub-paths of their crates (the town lane's modules in the adventure
    /// lane's crates).
    #[serde(default)]
    pub verify_with: Vec<String>,
}

/// Who owns a path.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Owner {
    Lane(String),
    Shared,
    Common,
    Unowned,
}

impl fmt::Display for Owner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Owner::Lane(name) => write!(f, "lane `{name}`"),
            Owner::Shared => f.write_str("shared"),
            Owner::Common => f.write_str("common"),
            Owner::Unowned => f.write_str("no owner"),
        }
    }
}

/// Whether `pattern` (`dir/`, `prefix*` or an exact file) matches `path`.
pub fn matches(pattern: &str, path: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix('*') {
        path.starts_with(prefix)
    } else if pattern.ends_with('/') {
        path.starts_with(pattern)
    } else {
        path == pattern
    }
}

impl Config {
    /// Parses and validates `lanes.toml` text.
    pub fn parse(text: &str) -> Result<Self, String> {
        let config: Config = toml::from_str(text).map_err(|e| format!("{LANES_FILE}: {e}"))?;
        let problems = config.problems();
        if problems.is_empty() {
            Ok(config)
        } else {
            Err(format!("{LANES_FILE}: {}", problems.join("; ")))
        }
    }

    /// Loads `lanes.toml` from the workspace root.
    pub fn load(root: &Path) -> Result<Self, String> {
        let path = root.join(LANES_FILE);
        let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        Self::parse(&text)
    }

    fn entries(&self) -> impl Iterator<Item = (&str, Owner)> {
        let shared = self
            .shared
            .paths
            .iter()
            .map(|p| (p.as_str(), Owner::Shared));
        let common = self
            .common
            .paths
            .iter()
            .map(|p| (p.as_str(), Owner::Common));
        let lanes = self.lane.iter().flat_map(|l| {
            l.paths
                .iter()
                .map(move |p| (p.as_str(), Owner::Lane(l.name.clone())))
        });
        shared.chain(common).chain(lanes)
    }

    fn problems(&self) -> Vec<String> {
        let mut v = Vec::new();
        let mut patterns = BTreeSet::new();
        for (pattern, owner) in self.entries() {
            if pattern.is_empty() || pattern.starts_with('/') || pattern.contains('\\') {
                v.push(format!(
                    "{owner}: `{pattern}` must be a relative path with `/`"
                ));
            }
            if !patterns.insert(pattern) {
                v.push(format!("`{pattern}` is listed twice"));
            }
        }
        let mut names = BTreeSet::new();
        let mut codes = BTreeSet::new();
        for lane in &self.lane {
            let name_ok = !lane.name.is_empty()
                && lane
                    .name
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c == '-');
            if !name_ok || lane.name == SHARED || lane.name == "common" {
                v.push(format!(
                    "lane name `{}` must be lowercase letters and `-`, and not `shared` or `common`",
                    lane.name
                ));
            }
            if !names.insert(lane.name.as_str()) {
                v.push(format!("lane `{}` is declared twice", lane.name));
            }
            let code_ok = (2..=4).contains(&lane.code.len())
                && lane.code.chars().all(|c| c.is_ascii_uppercase());
            if !code_ok {
                v.push(format!(
                    "lane `{}`: code `{}` must be 2 to 4 uppercase letters",
                    lane.name, lane.code
                ));
            }
            if !codes.insert(lane.code.as_str()) {
                v.push(format!("lane code `{}` is used twice", lane.code));
            }
            if lane.paths.is_empty() {
                v.push(format!("lane `{}` owns no paths", lane.name));
            }
            for with in &lane.verify_with {
                if *with == lane.name || self.lane(with).is_none() {
                    v.push(format!(
                        "lane `{}`: `verify_with` names `{with}`, not another lane",
                        lane.name
                    ));
                }
            }
        }
        v
    }

    /// The owner of a workspace-relative path (forward slashes): the longest matching pattern.
    pub fn owner(&self, path: &str) -> Owner {
        self.entries()
            .filter(|(pattern, _)| matches(pattern, path))
            .max_by_key(|(pattern, _)| pattern.len())
            .map_or(Owner::Unowned, |(_, owner)| owner)
    }

    pub fn lane(&self, name: &str) -> Option<&Lane> {
        self.lane.iter().find(|l| l.name == name)
    }

    /// The lane codes, the prefixes of lane ids.
    pub fn codes(&self) -> BTreeSet<String> {
        self.lane.iter().map(|l| l.code.clone()).collect()
    }

    pub fn lane_names(&self) -> Vec<&str> {
        self.lane.iter().map(|l| l.name.as_str()).collect()
    }

    /// The lane (or `shared`) a branch name declares: its part before the first `/`.
    pub fn declared_by_branch(&self, branch: &str) -> Option<String> {
        let prefix = branch.split_once('/')?.0;
        (prefix == SHARED || self.lane(prefix).is_some()).then(|| prefix.to_owned())
    }

    /// The lanes a scoped verify of `lane` covers: `lane`, then its `verify_with` lanes.
    pub fn verified_lanes(&self, lane: &str) -> Vec<String> {
        let mut lanes = vec![lane.to_owned()];
        if let Some(l) = self.lane(lane) {
            lanes.extend(l.verify_with.iter().cloned());
        }
        lanes
    }

    /// Workspace packages (name, workspace-relative directory) owned by `lane`.
    pub fn lane_packages<'a>(&self, lane: &str, packages: &'a [(String, String)]) -> Vec<&'a str> {
        packages
            .iter()
            .filter(|(_, dir)| {
                self.owner(&format!("{dir}/Cargo.toml")) == Owner::Lane(lane.to_owned())
            })
            .map(|(name, _)| name.as_str())
            .collect()
    }
}

/// Whether `id` is a lane id `<CODE>-<KIND>-<n>` (`ADV-STUB-1`, `SRV-QUIRK-12`): a lane code from
/// `codes`, the kind, and a number without leading zeros. Ids from before the lanes keep their
/// old form (`STUB-0001`, `Q-0001`).
pub fn is_lane_id(id: &str, kind: &str, codes: &BTreeSet<String>) -> bool {
    let mut parts = id.splitn(3, '-');
    let (Some(code), Some(k), Some(n)) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    codes.contains(code)
        && k == kind
        && !n.is_empty()
        && !n.starts_with('0')
        && n.chars().all(|c| c.is_ascii_digit())
}

/// Lists up to a few paths for a message.
fn listing(paths: &[&str]) -> String {
    const SHOWN: usize = 12;
    let mut text = paths
        .iter()
        .take(SHOWN)
        .map(|p| format!("\n    {p}"))
        .collect::<String>();
    if paths.len() > SHOWN {
        text.push_str(&format!("\n    … and {} more", paths.len() - SHOWN));
    }
    text
}

/// Judges a set of changed paths. `declared` is a lane name or [`SHARED`]. Returns a one-line
/// verdict, or the violations.
pub fn judge(
    config: &Config,
    changed: &[String],
    declared: Option<&str>,
    cross_lane: bool,
) -> Result<String, String> {
    let mut by_owner: BTreeMap<Owner, Vec<&str>> = BTreeMap::new();
    for path in changed {
        by_owner
            .entry(config.owner(path))
            .or_default()
            .push(path.as_str());
    }
    let touched = by_owner
        .iter()
        .map(|(owner, paths)| format!("{owner}: {}", paths.len()))
        .collect::<Vec<_>>()
        .join(", ");
    let touched = if touched.is_empty() {
        "no changes".to_owned()
    } else {
        touched
    };
    if let Some(paths) = by_owner.get(&Owner::Unowned) {
        return Err(format!(
            "paths without an owner in {LANES_FILE} (add them there first, a shared change):{}",
            listing(paths)
        ));
    }
    let lanes: Vec<&str> = by_owner
        .keys()
        .filter_map(|o| match o {
            Owner::Lane(name) => Some(name.as_str()),
            _ => None,
        })
        .collect();
    let shared = by_owner.contains_key(&Owner::Shared);
    if cross_lane {
        return Ok(format!("cross-lane change ({touched})"));
    }
    match declared {
        Some(name) => {
            if name != SHARED && config.lane(name).is_none() {
                return Err(format!(
                    "unknown lane `{name}`; lanes: {}, or `{SHARED}`",
                    config.lane_names().join(", ")
                ));
            }
            let outside: Vec<&str> = by_owner
                .iter()
                .filter(|(owner, _)| match owner {
                    Owner::Common => false,
                    Owner::Shared => name != SHARED,
                    Owner::Lane(lane) => lane != name,
                    Owner::Unowned => true,
                })
                .flat_map(|(_, paths)| paths.iter().copied())
                .collect();
            if outside.is_empty() {
                Ok(format!("`{name}` change ({touched})"))
            } else {
                Err(format!(
                    "a `{name}` change touches paths outside its lane ({touched}); move them to \
                     their own pull request, or label a deliberate cross-lane change \
                     `cross-lane`:{}",
                    listing(&outside)
                ))
            }
        }
        None if lanes.len() > 1 => Err(format!(
            "the change touches the lanes {}: one lane per pull request, or label a deliberate \
             cross-lane change `cross-lane` ({touched})",
            lanes.join(", ")
        )),
        None if shared && !lanes.is_empty() => Err(format!(
            "the change touches lane `{}` and shared paths: shared changes go in their own pull \
             request first ({touched}); shared paths:{}",
            lanes.join(", "),
            listing(&by_owner[&Owner::Shared])
        )),
        None => {
            let inferred = lanes
                .first()
                .copied()
                .unwrap_or(if shared { SHARED } else { "common" });
            Ok(format!("`{inferred}` change, inferred ({touched})"))
        }
    }
}

/// Paths changed since the merge base with `base`: committed, uncommitted and untracked.
pub fn changed_paths(base: &str) -> Result<Vec<String>, String> {
    let merge_base = git(&["merge-base", "HEAD", base])
        .ok_or_else(|| format!("no merge base between HEAD and `{base}` (fetch it first)"))?;
    let diff = git(&["diff", "--name-only", "--no-renames", merge_base.trim()])
        .ok_or("git diff failed")?;
    let untracked =
        git(&["ls-files", "--others", "--exclude-standard"]).ok_or("git ls-files failed")?;
    let paths: BTreeSet<String> = diff
        .lines()
        .chain(untracked.lines())
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect();
    Ok(paths.into_iter().collect())
}

/// The branch being checked: the pull request's head in GitHub Actions, else the checked-out one.
fn current_branch() -> Option<String> {
    std::env::var("GITHUB_HEAD_REF")
        .ok()
        .filter(|b| !b.is_empty())
        .or_else(|| {
            git(&["rev-parse", "--abbrev-ref", "HEAD"])
                .map(|b| b.trim().to_owned())
                .filter(|b| b != "HEAD")
        })
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Options {
    lane: Option<String>,
    base: Option<String>,
    cross_lane: bool,
}

fn parse(args: &[String]) -> Result<Options, String> {
    let mut options = Options::default();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--lane" => options.lane = Some(iter.next().ok_or("--lane needs a name")?.clone()),
            "--base" => options.base = Some(iter.next().ok_or("--base needs a ref")?.clone()),
            "--cross-lane" => options.cross_lane = true,
            other => return Err(format!("unknown check-lane option `{other}`")),
        }
    }
    Ok(options)
}

/// The default base: `origin/master` if it exists, else `master`.
fn default_base() -> String {
    if git(&["rev-parse", "--verify", "--quiet", "origin/master"]).is_some() {
        "origin/master".to_owned()
    } else {
        "master".to_owned()
    }
}

/// Runs the check for `lane` (or the declared/inferred lane when `None`).
pub fn check(lane: Option<&str>, base: Option<&str>, cross_lane: bool) -> Result<(), String> {
    let root = workspace_root();
    let config = Config::load(&root)?;
    let base = base.map_or_else(default_base, str::to_owned);
    // A deleted path that no pattern matches any more was moved or removed together with its
    // pattern (a lanes.toml change, itself shared): nothing to judge.
    let changed: Vec<String> = changed_paths(&base)?
        .into_iter()
        .filter(|p| root.join(p).exists() || config.owner(p) != Owner::Unowned)
        .collect();
    let declared = match lane {
        Some(name) => Some(name.to_owned()),
        None => current_branch().and_then(|b| config.declared_by_branch(&b)),
    };
    let verdict = judge(&config, &changed, declared.as_deref(), cross_lane)?;
    println!("check-lane: ok: {verdict}, against {base}");
    Ok(())
}

pub fn run(args: &[String]) -> Result<(), String> {
    let options = parse(args)?;
    check(
        options.lane.as_deref(),
        options.base.as_deref(),
        options.cross_lane,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: &str = r#"
[shared]
paths = ["Cargo.toml", "crates/h3-kernel/", "docs/adr/"]
[common]
paths = ["docs/OPEN-QUESTIONS.md"]
[[lane]]
name = "battle"
code = "BAT"
paths = ["crates/h3-battle/", "docs/epics/E04*", "docs/lanes/battle/"]
[[lane]]
name = "adventure"
code = "ADV"
paths = ["crates/h3-adventure/", "crates/h3-battle/src/seam/"]
[[lane]]
name = "town"
code = "TWN"
verify_with = ["adventure"]
paths = ["crates/h3-towns/", "crates/h3-adventure/src/town*"]
"#;

    fn config() -> Config {
        Config::parse(CONFIG).unwrap()
    }

    fn paths(p: &[&str]) -> Vec<String> {
        p.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn patterns_and_longest_match() {
        assert!(matches("crates/a/", "crates/a/src/lib.rs"));
        assert!(!matches("crates/a/", "crates/ab/src/lib.rs"));
        assert!(matches("docs/epics/E04*", "docs/epics/E04b-log.md"));
        assert!(matches("Cargo.toml", "Cargo.toml"));
        assert!(!matches("Cargo.toml", "crates/a/Cargo.toml"));
        let c = config();
        assert_eq!(c.owner("Cargo.toml"), Owner::Shared);
        assert_eq!(c.owner("docs/OPEN-QUESTIONS.md"), Owner::Common);
        assert_eq!(
            c.owner("crates/h3-battle/src/lib.rs"),
            Owner::Lane("battle".to_owned())
        );
        assert_eq!(
            c.owner("crates/h3-battle/src/seam/mod.rs"),
            Owner::Lane("adventure".to_owned())
        );
        assert_eq!(c.owner("README.md"), Owner::Unowned);
    }

    #[test]
    fn rejects_bad_configs() {
        let twice = "[shared]\npaths = [\"a\"]\n[common]\npaths = [\"a\"]\n";
        assert!(Config::parse(twice).unwrap_err().contains("listed twice"));
        let bad_lane = "[shared]\npaths = []\n[common]\npaths = []\n\
                        [[lane]]\nname = \"Shared\"\ncode = \"b\"\npaths = []\n";
        let err = Config::parse(bad_lane).unwrap_err();
        assert!(err.contains("lowercase") && err.contains("uppercase") && err.contains("no paths"));
        let dup = "[shared]\npaths = [\"/x\"]\n[common]\npaths = []\n\
                   [[lane]]\nname = \"a\"\ncode = \"AA\"\npaths = [\"a/\"]\n\
                   [[lane]]\nname = \"a\"\ncode = \"AA\"\npaths = [\"b/\"]\n";
        let err = Config::parse(dup).unwrap_err();
        assert!(
            err.contains("declared twice")
                && err.contains("used twice")
                && err.contains("relative")
        );
        assert!(Config::parse("nonsense").is_err());
        let with = "[shared]\npaths = []\n[common]\npaths = []\n\
                    [[lane]]\nname = \"a\"\ncode = \"AA\"\npaths = [\"a/\"]\n\
                    verify_with = [\"a\", \"b\"]\n";
        let err = Config::parse(with).unwrap_err();
        assert!(err.contains("`verify_with` names `a`") && err.contains("names `b`"));
    }

    #[test]
    fn verify_with_widens_the_scope() {
        let c = config();
        assert_eq!(c.verified_lanes("town"), vec!["town", "adventure"]);
        assert_eq!(c.verified_lanes("battle"), vec!["battle"]);
        assert_eq!(
            c.owner("crates/h3-adventure/src/town/visit.rs"),
            Owner::Lane("town".to_owned())
        );
        assert_eq!(
            c.owner("crates/h3-adventure/src/lib.rs"),
            Owner::Lane("adventure".to_owned())
        );
    }

    #[test]
    fn branches_declare_lanes() {
        let c = config();
        assert_eq!(
            c.declared_by_branch("battle/e04b-t5c").as_deref(),
            Some("battle")
        );
        assert_eq!(
            c.declared_by_branch("shared/lanes").as_deref(),
            Some(SHARED)
        );
        assert_eq!(c.declared_by_branch("claude/hopeful-mayer"), None);
        assert_eq!(c.declared_by_branch("e04b-battle-magic"), None);
    }

    #[test]
    fn judges_declared_lanes() {
        let c = config();
        let battle = paths(&["crates/h3-battle/src/lib.rs", "docs/OPEN-QUESTIONS.md"]);
        assert!(judge(&c, &battle, Some("battle"), false).is_ok());
        assert!(judge(&c, &battle, Some("adventure"), false).is_err());
        let mixed = paths(&["crates/h3-battle/src/lib.rs", "Cargo.toml"]);
        let err = judge(&c, &mixed, Some("battle"), false).unwrap_err();
        assert!(err.contains("Cargo.toml") && !err.contains("h3-battle/src"));
        assert!(
            judge(
                &c,
                &paths(&["Cargo.toml", "docs/OPEN-QUESTIONS.md"]),
                Some(SHARED),
                false
            )
            .is_ok()
        );
        assert!(judge(&c, &mixed, Some(SHARED), false).is_err());
        assert!(
            judge(&c, &battle, Some("nope"), false)
                .unwrap_err()
                .contains("unknown lane")
        );
    }

    #[test]
    fn infers_the_lane() {
        let c = config();
        let ok = judge(&c, &paths(&["crates/h3-battle/src/lib.rs"]), None, false).unwrap();
        assert!(ok.contains("`battle` change, inferred"));
        assert!(
            judge(&c, &paths(&["docs/adr/x.md"]), None, false)
                .unwrap()
                .contains("`shared`")
        );
        assert!(
            judge(&c, &paths(&["docs/OPEN-QUESTIONS.md"]), None, false)
                .unwrap()
                .contains("`common`")
        );
        assert!(judge(&c, &[], None, false).unwrap().contains("no changes"));
        let two = paths(&[
            "crates/h3-battle/src/lib.rs",
            "crates/h3-adventure/src/lib.rs",
        ]);
        assert!(
            judge(&c, &two, None, false)
                .unwrap_err()
                .contains("adventure, battle")
        );
        let mixed = paths(&["crates/h3-battle/src/lib.rs", "Cargo.toml"]);
        assert!(
            judge(&c, &mixed, None, false)
                .unwrap_err()
                .contains("own pull request first")
        );
        assert!(judge(&c, &two, None, true).unwrap().contains("cross-lane"));
    }

    #[test]
    fn unowned_paths_always_fail() {
        let c = config();
        let many: Vec<String> = (0..20).map(|i| format!("new/{i}.md")).collect();
        let err = judge(&c, &many, None, true).unwrap_err();
        assert!(err.contains("without an owner") && err.contains("and 8 more"));
    }

    #[test]
    fn lane_ids() {
        let codes = config().codes();
        assert!(is_lane_id("ADV-STUB-1", "STUB", &codes));
        assert!(is_lane_id("BAT-QUIRK-12", "QUIRK", &codes));
        assert!(!is_lane_id("ADV-STUB-01", "STUB", &codes));
        assert!(!is_lane_id("ADV-STUB-", "STUB", &codes));
        assert!(!is_lane_id("ADV-STUB-1a", "STUB", &codes));
        assert!(!is_lane_id("XYZ-STUB-1", "STUB", &codes));
        assert!(!is_lane_id("ADV-QUIRK-1", "STUB", &codes));
        assert!(!is_lane_id("STUB-0001", "STUB", &codes));
    }

    #[test]
    fn lane_packages_follow_ownership() {
        let c = config();
        let packages = vec![
            ("h3-battle".to_owned(), "crates/h3-battle".to_owned()),
            ("h3-kernel".to_owned(), "crates/h3-kernel".to_owned()),
        ];
        assert_eq!(c.lane_packages("battle", &packages), vec!["h3-battle"]);
        assert!(c.lane_packages("adventure", &packages).is_empty());
    }

    #[test]
    fn parses_options() {
        let args = paths(&[
            "--lane",
            "battle",
            "--base",
            "origin/master",
            "--cross-lane",
        ]);
        assert_eq!(
            parse(&args).unwrap(),
            Options {
                lane: Some("battle".to_owned()),
                base: Some("origin/master".to_owned()),
                cross_lane: true,
            }
        );
        assert!(parse(&paths(&["--lane"])).is_err());
        assert!(parse(&paths(&["--base"])).is_err());
        assert!(parse(&paths(&["--what"])).is_err());
    }

    #[test]
    fn the_repository_lanes_file_is_valid() {
        let c = Config::load(&workspace_root()).unwrap();
        for lane in [
            "battle",
            "adventure",
            "ai",
            "server",
            "client",
            "map-client",
            "town",
        ] {
            assert!(c.lane(lane).is_some(), "{lane}");
            assert_eq!(
                c.owner(&format!("docs/lanes/{lane}/HANDOFF.md")),
                Owner::Lane(lane.to_owned())
            );
            assert_eq!(
                c.owner(&format!("docs/lanes/{lane}/QUEUE.md")),
                Owner::Common
            );
        }
        // The client lane's split (2026-10-23): the foundation and the harness are common, the
        // battle viewer the client lane's, the map lane's crates and folder its own.
        for common in [
            "clients/crates/h3-render/src/lib.rs",
            "clients/capture/h3drive.py",
            "clients/text/keys.toml",
            "clients/Cargo.toml",
        ] {
            assert_eq!(c.owner(common), Owner::Common, "{common}");
        }
        assert_eq!(
            c.owner("clients/crates/h3-view/src/lib.rs"),
            Owner::Lane("client".to_owned())
        );
        assert_eq!(
            c.owner("clients/crates/h3-map-view/src/lib.rs"),
            Owner::Lane("map-client".to_owned())
        );
        assert_eq!(
            c.owner("clients/map/layouts/adventure.toml"),
            Owner::Lane("map-client".to_owned())
        );
        // The town lane (2026-10-31, TWN-Q1): its crate and the town modules of the adventure
        // lane's crates; the crates' common parts stay the adventure lane's.
        for (path, lane) in [
            ("crates/h3-towns/src/lib.rs", "town"),
            ("crates/h3-adventure/src/town/mod.rs", "town"),
            ("crates/h3-game/src/town.rs", "town"),
            ("crates/h3-map/src/import/town.rs", "town"),
            ("crates/h3-map/src/writer/town.rs", "town"),
            ("docs/epics/E09-towns.md", "town"),
            ("docs/rules/catalog/town.toml", "town"),
            ("crates/h3-adventure/src/lib.rs", "adventure"),
            ("crates/h3-game/src/state.rs", "adventure"),
            ("docs/epics/E10-orchestration.md", "adventure"),
        ] {
            assert_eq!(c.owner(path), Owner::Lane(lane.to_owned()), "{path}");
        }
        assert_eq!(c.verified_lanes("town"), vec!["town", "adventure"]);
        assert_eq!(c.owner("lanes.toml"), Owner::Shared);
        assert_eq!(c.owner("Cargo.lock"), Owner::Common);
        assert_eq!(c.owner("crates/h3-kernel/src/lib.rs"), Owner::Shared);
        assert_eq!(
            c.owner("experiments/specs/E04b-batch-2c.toml"),
            Owner::Lane("battle".to_owned())
        );
        assert_eq!(
            c.owner("experiments/specs/E08-batch-1.toml"),
            Owner::Lane("adventure".to_owned())
        );
        assert_eq!(c.owner("crates/h3-pack/src/lib.rs"), Owner::Shared);
        assert_eq!(
            c.owner("crates/h3-pack-battle/src/lib.rs"),
            Owner::Lane("battle".to_owned())
        );
        assert_eq!(
            c.owner("clients/situations/set.toml"),
            Owner::Lane("client".to_owned())
        );
        // Adventure-owned sub-paths of battle's crates and folders (longest match, E08 T2).
        let adventure = Owner::Lane("adventure".to_owned());
        let battle = Owner::Lane("battle".to_owned());
        for path in [
            "crates/h3-game/src/lib.rs",
            "crates/h3-sim/src/adventure/replay.rs",
            "fixtures/replays/adventure/walk-s1.toml",
            "crates/h3-cli/tests/adventure_replays.rs",
            "packs/sod/adventure.toml",
            "docs/formats/h3m.md",
        ] {
            assert_eq!(c.owner(path), adventure, "{path}");
        }
        for path in [
            "crates/h3-cli/src/main.rs",
            "fixtures/replays/duel-s1.toml",
            "packs/sod/battle.toml",
            "docs/formats/installation.md",
        ] {
            assert_eq!(c.owner(path), battle, "{path}");
        }
        // The AI lane's (E05, AI-Q1): the sim layer and the C ABI, the CLI's `src/ai/`; the
        // adventure env but its replay model (E08 T45, E08 decision 103).
        let ai = Owner::Lane("ai".to_owned());
        for path in [
            "crates/h3-sim/src/env.rs",
            "crates/h3-sim/src/adventure/env.rs",
            "crates/h3-sim/src/adventure/tests/battles.rs",
            "crates/h3-ffi/src/lib.rs",
            "crates/h3-cli/src/ai/mod.rs",
            "python/README.md",
            "docs/epics/E05-battle-env.md",
        ] {
            assert_eq!(c.owner(path), ai, "{path}");
        }
        assert_eq!(c.owner("docs/rules/surveys/README.md"), Owner::Common);
    }
}
