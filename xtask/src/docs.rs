//! `cargo xtask check-docs`: keeps the start-of-session reading small (docs/08-work-process.md,
//! "HANDOFF"). Every session starts by reading `docs/HANDOFF.md`, so it has a size budget and
//! holds at most [`MAX_SESSIONS`] entries under "## Recent sessions"; older entries live in git
//! history only.

use std::fs;

use crate::util::workspace_root;

/// The file every session reads first.
pub const HANDOFF: &str = "docs/HANDOFF.md";
/// Size budget of [`HANDOFF`] in bytes (about 2 500 tokens).
pub const MAX_BYTES: usize = 10_000;
/// Entries allowed under [`SESSIONS_HEADING`].
pub const MAX_SESSIONS: usize = 3;
/// The section whose `### ` headings are the session entries.
pub const SESSIONS_HEADING: &str = "## Recent sessions";

/// What is wrong with a HANDOFF text; empty if it keeps the budget.
pub fn handoff_problems(text: &str) -> Vec<String> {
    let mut problems = Vec::new();
    if text.len() > MAX_BYTES {
        problems.push(format!(
            "{HANDOFF} is {} bytes, the budget is {MAX_BYTES}: shorten the session entries and \
             prune \"Standing traps\" (docs/08-work-process.md, \"HANDOFF\")",
            text.len()
        ));
    }
    let mut in_sessions = false;
    let mut found = false;
    let mut sessions = 0usize;
    for line in text.lines() {
        if line.starts_with("## ") {
            in_sessions = line.trim_end() == SESSIONS_HEADING;
            found |= in_sessions;
        } else if in_sessions && line.starts_with("### ") {
            sessions += 1;
        }
    }
    if !found {
        problems.push(format!("{HANDOFF} has no `{SESSIONS_HEADING}` section"));
    }
    if sessions > MAX_SESSIONS {
        problems.push(format!(
            "{HANDOFF} has {sessions} entries under `{SESSIONS_HEADING}`, at most \
             {MAX_SESSIONS}: delete the oldest (git history keeps them)"
        ));
    }
    problems
}

pub fn run(_args: &[String]) -> Result<(), String> {
    let path = workspace_root().join(HANDOFF);
    let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let problems = handoff_problems(&text);
    if problems.is_empty() {
        println!(
            "check-docs: ok ({HANDOFF}: {} of {MAX_BYTES} bytes)",
            text.len()
        );
        Ok(())
    } else {
        Err(problems.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn handoff(sessions: usize) -> String {
        let mut text = String::from("# HANDOFF\n\n## Current state\n\nx\n\n## Recent sessions\n\n");
        for i in 0..sessions {
            text.push_str(&format!("### Session {i}\n\ntext\n\n"));
        }
        text
    }

    #[test]
    fn accepts_a_small_handoff() {
        assert!(handoff_problems(&handoff(MAX_SESSIONS)).is_empty());
    }

    #[test]
    fn rejects_too_many_sessions_and_bytes() {
        assert_eq!(handoff_problems(&handoff(MAX_SESSIONS + 1)).len(), 1);
        let mut big = handoff(1);
        big.push_str(&"x".repeat(MAX_BYTES));
        assert_eq!(handoff_problems(&big).len(), 1);
    }

    #[test]
    fn requires_the_sessions_section_and_counts_only_inside_it() {
        assert_eq!(handoff_problems("# HANDOFF\n").len(), 1);
        let text = "## Standing traps\n### a\n### b\n### c\n### d\n## Recent sessions\n### one\n";
        assert!(handoff_problems(text).is_empty());
    }
}
