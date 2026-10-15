# E04c T10b fold-in: common brief for the fix workers

You fix a few findings of the E04c independent review in the repository `/home/claude/homm3`
(branch `battle/e04c-t10b`, already checked out; a headless, deterministic Rust rules engine for
Heroes of Might and Magic III). Read `AGENTS.md` ("Non-negotiables", "Working rules") and the
"Standing traps" of `docs/lanes/battle/HANDOFF.md` first; your task prompt names your findings.

## The findings

Each finding is written up in a reviewer's report on the backup branch. Read only your findings'
sections: `git -C /home/claude/homm3 show origin/battle/e04c-t10b-review-reports:docs/lanes/battle/review-e04c/<report>.md`
(pipe through `awk '/^### F-r2a-1/,/^### F-r2a-2/'` or similar). The coordinator's verified
list is `DIGEST.md` on the same branch. The user's decisions are BAT-Q50 in
`docs/OPEN-QUESTIONS.md` ("Answered", battle block). Rule entries: `python3
/tmp/claude-0/-home-claude-homm3/cf1ab52b-ffb8-5850-96e9-2c1042241879/scratchpad/review/rules.py <ID>`
prints one (never read `docs/rules/catalog/battle.toml` whole; `grep -n` and edit in place).

Before you change code for a behaviour finding, check the claim yourself: read the engine's code
it names and, where the finding rests on the executable, the cited addresses
(`python3 scripts/exe.py fn <addr> --from <a> --to <b>`; `docs/exe/README.md`, "Reading code",
first; never read `docs/exe/map.toml` or `h3api.toml` whole; every address you cite needs a map
entry, `python3 scripts/exe.py cited`). If the claim does not hold, do not fix it: say so in
your report with the evidence.

## How to fix

- **Test first:** write a test that fails on the current code (run it, see it fail), then fix,
  then see it pass. Mark tests `// covers: <RULE-ID>` and code `// rule: <RULE-ID>` as the files
  around them do. Put tests where the rule's other tests are (`crates/h3-battle/src/tests/…`).
- **Docs in the same commit:** the rule's catalog entry (summary, and a source line for a new
  code reading: `kind = "original-data"`, the addresses and `installation.md`'s section), and a
  short "Review re-read (E04c T10b)" note at the end of the matching `installation.md` §7.2x
  section when the fix rests on a reading of the executable (your own words with addresses, no
  disassembly listings: ADR-0009). Edit docs with targeted replacements, never by rewriting a
  file.
- Keep the engine's rules: pure logic, no `HashMap`, no floats, no panics on reachable input;
  100 % line coverage of what you add (no defensive dead branches).
- **Wire types:** a new `Command`, `EventKind`, `CommandError`, `CastTarget`, `DamageRoll` or
  `ObstacleKind` variant breaks `h3-protocol` (another lane's crate). Use an existing variant
  when one fits; if none does, stop and say so in your report.
- **Golden replays and benchmarks:** a behaviour change makes `crates/h3-cli/tests/replay_fixtures.rs`
  (and the adventure lane's replays in `crates/h3-cli/tests/adventure_replays.rs`), the
  determinism gate and `bench-check` fail. That is expected: **do not regenerate** fixtures or
  baselines; the coordinator does it once after all fixes. Name the failing replay files in your
  report.
- Build with `CARGO_INCREMENTAL=0` (always: the disk is limited). The machine has two cores; run
  only the tests you need: `cargo test -p h3-battle --lib <filter>`, then once at the end
  `cargo test -p h3-battle` and the crates you touched, `cargo clippy -p <crate> --all-targets
  -- -D warnings`, `cargo fmt --all`, `cargo xtask check-rules`, `cargo xtask check-docs`,
  `cargo xtask check-exe-map`. Do not run `cargo xtask verify` (the coordinator does).
- **Commit** each finding (or a small group) on `battle/e04c-t10b` with a message
  `battle: E04c T10b — <what>` ending with these two lines:
  `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` and
  `Claude-Session: https://claude.ai/code/session_01EQhEs1ueH99Kdr8XNw7xCa`.
  Do not push (the coordinator pushes). Do not touch files outside your findings' scope.
- No sub-agents, no web access. Read big files in slices.

## Your report (final message, at most 15 lines)

Per finding: fixed (commit hash, the test's name) | not fixed (why, evidence) | needs the user.
Then: the replay files that now fail, anything left half-done, anything you noticed but did not
touch.
