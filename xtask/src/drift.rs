//! `cargo xtask doc-drift`: warns when logic crates change without any documentation change
//! (docs/05-testing.md, "Doc-drift check"). A heuristic: it compares the working tree and the
//! commits since the merge base with `master` and never fails verify (there is no CI yet to
//! turn it into an error for large changes, OPEN-QUESTIONS Q3).

use std::process::Command;

use crate::util::workspace_root;
use crate::verify::Status;

/// Paths whose change counts as "documentation kept current".
const DOC_PATHS: &[&str] = &["docs/HANDOFF.md", "docs/rules/", "docs/adr/", "docs/epics/"];
/// Logic crates (see arch.toml).
const LOGIC_PATHS: &[&str] = &[
    "crates/h3-kernel/",
    "crates/h3-content/",
    "crates/h3-contracts/",
    "crates/h3-battle/",
];

/// Whether a set of changed paths touches logic code without touching documentation.
pub fn drifted(changed: &[String]) -> bool {
    let logic = changed
        .iter()
        .any(|p| LOGIC_PATHS.iter().any(|l| p.starts_with(l)));
    let docs = changed
        .iter()
        .any(|p| DOC_PATHS.iter().any(|d| p.starts_with(d)));
    logic && !docs
}

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .current_dir(workspace_root())
        .args(args)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

pub fn status() -> Result<Status, String> {
    let Some(base) = git(&["merge-base", "HEAD", "master"])
        .or_else(|| git(&["merge-base", "HEAD", "origin/master"]))
    else {
        return Ok(Status::NotApplicable(
            "no git or no `master` branch to compare with".to_owned(),
        ));
    };
    let mut changed: Vec<String> = git(&["diff", "--name-only", base.trim()])
        .unwrap_or_default()
        .lines()
        .map(str::to_owned)
        .collect();
    changed.extend(
        git(&["status", "--porcelain"])
            .unwrap_or_default()
            .lines()
            .filter_map(|l| l.get(3..))
            .map(str::to_owned),
    );
    if drifted(&changed) {
        println!(
            "doc-drift: WARNING: logic crates changed since master but none of {} did. \
             Update HANDOFF, the rule catalog, ADRs or the epic file in the same change.",
            DOC_PATHS.join(", ")
        );
    } else {
        println!(
            "doc-drift: ok ({} changed path(s) since master)",
            changed.len()
        );
    }
    Ok(Status::Passed)
}

pub fn run(_args: &[String]) -> Result<(), String> {
    status().map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(p: &[&str]) -> Vec<String> {
        p.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn detects_logic_changes_without_docs() {
        assert!(drifted(&paths(&["crates/h3-battle/src/damage.rs"])));
        assert!(!drifted(&paths(&[
            "crates/h3-battle/src/damage.rs",
            "docs/rules/catalog/battle.toml"
        ])));
        assert!(!drifted(&paths(&["crates/h3-cli/src/main.rs"])));
        assert!(!drifted(&paths(&[])));
    }
}
