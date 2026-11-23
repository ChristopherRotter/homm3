# E24 M4 review — the common brief (every reviewer reads this first)

You are a fresh reviewer of one area of E24 M4 ("the engine's games shown"), the adventure map
client of a Heroes of Might and Magic III project. The repo is at `/home/claude/homm3` (branch
`map-client/e24-m4-t6`, `master` at f1c4a53). The map client is the crate
`clients/crates/h3-map-view` (its own Cargo workspace under `clients/`; the library is pure, the
binary `src/bin/h3-map-view/` does the I/O, the window and the sound). The capture harness that
records the original game under Wine is `clients/capture/` (Python and C). Its data:
`clients/map/` (situations, sequences, timelines, tables).

## What M4 is

A whole game the engine played, shown in the window as the original shows it to one player:
Part A, the original's reactions to a game's events read from the executable and recorded under
Wine, the viewer reproducing them from the original's memory (kept "live parts"); Part B, the
engine's whole-game records (`h3-protocol`'s messages, one player's view) as the viewer's input;
Part C, the battles handed to the battle client's viewer (`h3-battle-client`) in the same window
and back. The milestone's scope, decisions, exit and its **review list** (the agent's
provisional choices, already listed for the user): `docs/epics/E24-map-client.md`, "M4" and
"M4's review list" (read those two sections, about 230 lines). The lane's rules: the
"Autonomy contract" in `docs/lanes/map-client/HANDOFF.md`. Standing traps:
`docs/lanes/map-client/TRAPS.md` (grep the area).

## Reading discipline (strict: your budget is about half a context)

- Never read a log, design note or big source file whole. `docs/epics/E24-log.md` (270 KB),
  `docs/epics/E24-design.md` (400 KB) and `src/timeline.rs` (290 KB) are read by heading or by
  grep: `grep -n '^### M4 T2a2' docs/epics/E24-log.md`, `grep -n '^### 5.3 ' docs/epics/E24-design.md`,
  `grep -n 'fn \|^impl\|^pub' src/timeline.rs`, then a `sed -n a,bp` slice.
- Read the code for your area, its tests' names (not every test body), the design's section for
  the intent, and the epic's review-list items for your area (already known to the user: report
  one only if the code does not do what it says, or it hides a real problem).
- **Do not build, run tests, or edit anything in the repo.** Two CPUs are busy with a baseline
  build. Reading and `grep`/`sed`/`git grep` only. The only file you write is your findings file.

## What to look for (in order of value)

1. **Bugs**: behaviour that differs from the design note or the original as recorded; a panic,
   `unwrap`, index or overflow reachable from an input file (a record, a sequence, a situation,
   a capture); a wrong state carried between events; an off-by-one in a timeline.
2. **Contract breaks**: the library doing I/O, a clock or globals (it must be pure); a game rule
   computed in the client instead of taken from the record or the original's memory (forbidden:
   "a fact the record lacks becomes an `MCL-Q<n>`, never a rule copied into the client"); the
   original's text (sentences, flavour) committed (ADR-0009: identifiers and numbers only); a
   change to another lane's paths.
3. **Risks**: an assumption that holds only for the recorded cases and fails on a golden replay
   or a shipped map; an error swallowed; a provisional choice that is not on the review list.
4. **Tests**: M4 behaviour no test pins; a test that cannot fail.
5. **Docs**: a doc comment, the design note or the runbook (`clients/capture/README.md`) saying
   something the code no longer does.
6. **Simplifications** worth doing (dead code, duplicated logic). Nits last, few.

Do not report style preferences, naming, or anything already on M4's review list unless it is
wrong.

## Output

Write `/home/claude/review-m4/R<n>.md`:

```
# R<n> — <area>
Read: <files and functions, briefly>
## Findings
### R<n>-<k> [bug|contract|risk|test|doc|simplify|nit] <one-line title>
Where: <path>:<line> (function). What: ... Evidence: <the lines, the design's words>.
Suggested fix: ... Effort: S|M|L.
```

Be concrete: a finding without a file and line, or without the evidence, is not useful. Verify
each bug claim against the code (follow the callers) before writing it; mark "unsure" where you
could not. Then answer with **only** a table: `id | kind | effort | one line`, and one line on
what you did not get to read.
