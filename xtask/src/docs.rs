//! `cargo xtask check-docs`: keeps the start-of-session reading small (docs/08-work-process.md,
//! "Keeping the start-of-session reading small"). Every session reads the global
//! `docs/HANDOFF.md` (an index plus the traps of every lane) and its lane's
//! `docs/lanes/<lane>/HANDOFF.md`, so both have a size budget, and a lane HANDOFF holds at most
//! [`MAX_SESSIONS`] entries under "## Recent sessions"; older entries live in git history only.
//! Every lane in `lanes.toml` has a HANDOFF and a `QUEUE.md`.

use std::fs;
use std::path::Path;

use crate::lanes::Config;
use crate::util::workspace_root;

/// The file every session reads first: the lane index and the traps of every lane.
pub const HANDOFF: &str = "docs/HANDOFF.md";
/// Size budget of [`HANDOFF`] in bytes.
pub const GLOBAL_MAX_BYTES: usize = 6_000;
/// Size budget of a lane's HANDOFF in bytes (about 2 500 tokens).
pub const MAX_BYTES: usize = 10_000;
/// Entries allowed under [`SESSIONS_HEADING`] in a lane's HANDOFF.
pub const MAX_SESSIONS: usize = 3;
/// The section whose `### ` headings are the session entries.
pub const SESSIONS_HEADING: &str = "## Recent sessions";

/// The HANDOFF of a lane.
pub fn lane_handoff(lane: &str) -> String {
    format!("docs/lanes/{lane}/HANDOFF.md")
}

/// The task queue of a lane.
pub fn lane_queue(lane: &str) -> String {
    format!("docs/lanes/{lane}/QUEUE.md")
}

/// What is wrong with the global HANDOFF text; empty if it keeps the budget.
pub fn global_problems(text: &str) -> Vec<String> {
    if text.len() > GLOBAL_MAX_BYTES {
        vec![format!(
            "{HANDOFF} is {} bytes, the budget is {GLOBAL_MAX_BYTES}: it holds the lane index and \
             the traps of every lane only; move lane traps into the lane's HANDOFF",
            text.len()
        )]
    } else {
        Vec::new()
    }
}

/// What is wrong with a lane HANDOFF text (`file` names it in messages); empty if it keeps the
/// budget.
pub fn lane_problems(file: &str, text: &str) -> Vec<String> {
    let mut problems = Vec::new();
    if text.len() > MAX_BYTES {
        problems.push(format!(
            "{file} is {} bytes, the budget is {MAX_BYTES}: shorten the session entries and \
             prune \"Standing traps\" (docs/08-work-process.md)",
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
        problems.push(format!("{file} has no `{SESSIONS_HEADING}` section"));
    }
    if sessions > MAX_SESSIONS {
        problems.push(format!(
            "{file} has {sessions} entries under `{SESSIONS_HEADING}`, at most {MAX_SESSIONS}: \
             delete the oldest (git history keeps them)"
        ));
    }
    problems
}

fn read(root: &Path, file: &str) -> Result<String, String> {
    fs::read_to_string(root.join(file)).map_err(|e| format!("{file}: {e}"))
}

pub fn run(_args: &[String]) -> Result<(), String> {
    let root = workspace_root();
    let config = Config::load(&root)?;
    let global = read(&root, HANDOFF)?;
    let mut problems = global_problems(&global);
    let mut sizes = vec![format!("{HANDOFF}: {} of {GLOBAL_MAX_BYTES}", global.len())];
    for lane in config.lane_names() {
        let file = lane_handoff(lane);
        match read(&root, &file) {
            Ok(text) => {
                problems.extend(lane_problems(&file, &text));
                sizes.push(format!("{lane}: {} of {MAX_BYTES}", text.len()));
            }
            Err(e) => problems.push(e),
        }
        let queue = lane_queue(lane);
        if !root.join(&queue).is_file() {
            problems.push(format!(
                "{queue} is missing (every lane in lanes.toml has one)"
            ));
        }
    }
    if problems.is_empty() {
        println!("check-docs: ok (bytes: {})", sizes.join(", "));
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
        assert!(lane_problems("h", &handoff(MAX_SESSIONS)).is_empty());
        assert!(lane_problems("h", &handoff(0)).is_empty());
        assert!(global_problems("# HANDOFF\n").is_empty());
    }

    #[test]
    fn rejects_too_many_sessions_and_bytes() {
        assert_eq!(lane_problems("h", &handoff(MAX_SESSIONS + 1)).len(), 1);
        let mut big = handoff(1);
        big.push_str(&"x".repeat(MAX_BYTES));
        assert_eq!(lane_problems("h", &big).len(), 1);
        assert_eq!(global_problems(&"x".repeat(GLOBAL_MAX_BYTES + 1)).len(), 1);
    }

    #[test]
    fn requires_the_sessions_section_and_counts_only_inside_it() {
        assert_eq!(lane_problems("h", "# HANDOFF\n").len(), 1);
        let text = "## Standing traps\n### a\n### b\n### c\n### d\n## Recent sessions\n### one\n";
        assert!(lane_problems("h", text).is_empty());
    }

    #[test]
    fn names_lane_files() {
        assert_eq!(lane_handoff("battle"), "docs/lanes/battle/HANDOFF.md");
        assert_eq!(lane_queue("server"), "docs/lanes/server/QUEUE.md");
    }
}
