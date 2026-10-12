# E04b T11b, second round: common brief for the code workers `c` and `p`

The review's first two fix lists (A1–A11, B1–B10) are done and joined on `battle/e04b-t11b`
(head `64fe5cf`, rebased onto today's `master`). You do one list of the second round.

**Read `<SCRATCH>/review/BRIEF-fix.md` first. All of it applies** (where you work, how to do one
fix, the hard limits, what to do when the list is done, the form of the final message), with these
changes:

- `<SCRATCH>` is `/tmp/claude-0/-home-claude/71610cbe-304c-5a61-baae-07ea062d316d/scratchpad`.
- The workers of this round and their places:

  | worker | list | worktree | branch | logs | notes |
  |---|---|---|---|---|---|
  | `c` | `<SCRATCH>/review/FIX-c.md` | `/home/claude/wt-a` | `battle/e04b-t11b-c` | `<SCRATCH>/wc/` | `<SCRATCH>/review/fix-notes-c.md` |
  | `p` | `<SCRATCH>/review/FIX-p.md` | `/home/claude/wt-b` | `battle/e04b-t11b-p` | `<SCRATCH>/wp/` | `<SCRATCH>/review/fix-notes-p.md` |

  Both branches start at `64fe5cf`. The worktree's `target` directory is warm from the first round.
  Two more workers (`d`, `q`) edit only files under `docs/` in their own worktrees; they run short
  `cargo xtask` checks now and then. The machine: 2 cores, 7 GB of memory, **about 9 GB of free
  disk for everybody**: keep your `target` below 6 GB (`du -sh`; `cargo clean -p <crate>` or
  removing `target/debug/incremental` frees space), and say so in your notes if you had to clean.
- The first round's notes (`fix-notes-a.md`, `fix-notes-b.md`) tell you what changed in the code
  you are about to touch; read the parts on the functions you work in.
- **A test of behaviour that is right today** (most of the lists' test items) must pass on the
  unchanged code. If it fails, either the sketch is wrong or you found a defect: do not change
  engine code for it. Check your test against the rule's summary in
  `docs/rules/catalog/battle.toml` and the code; if the engine really differs from its rule, leave
  the test out, describe the case in your notes with its numbers (what the rule says, what the
  engine does, the smallest setup) and go on.
- **A test of a fix** (list C's three fixes) follows the brief: it fails first for the finding's
  reason.
- Commit messages: `battle: E04b T11b review — <what> (<finding ids>)` (prefix `pack:`, `content:`,
  `sim:` for other crates), with the two closing lines the brief gives. Push after each commit:
  `git push -q -u origin <your branch>`.
- The golden replays are not regenerated yet: `cargo test -p h3-cli`'s
  `golden_replays_reproduce_their_hashes` fails on the joined branch already (seven of eight
  replays, from A10, B1 and B2). That one failure is expected; do not regenerate.
- Keep the notes file current after each item (a restart must lose little). Its closing parts are
  as in the brief; for tests that add no behaviour, "rule texts now wrong" is empty.
