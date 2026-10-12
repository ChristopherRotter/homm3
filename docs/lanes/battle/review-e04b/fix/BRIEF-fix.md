# E04b T11b: fixing the review's findings — common brief for the fix workers

The independent review of epic E04b (battle magic) in `/home/claude/homm3` (a deterministic
Heroes of Might and Magic III rules engine in Rust) is done. Eleven reviewers wrote findings into
`<SCRATCH>/review/*.md`. The coordinator checked them and chose what is fixed now. You implement
one list of those fixes (your task prompt names your list file). Everything not in your list is
handled elsewhere: do not fix it, even if you see it.

`<SCRATCH>` is `/tmp/claude-0/-home-claude/71610cbe-304c-5a61-baae-07ea062d316d/scratchpad`.

## Where you work

- Your own git worktree and branch, already created (your list file names them). Never work in
  `/home/claude/homm3` itself, and never in the other worker's worktree.
- In every shell call set:
  `export CARGO_TARGET_DIR=<your worktree>/target CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_BUILD_JOBS=2`
- Another worker builds at the same time on this machine (2 cores, 7 GB of memory, about 19 GB of
  disk): run one cargo command at a time; keep your target directory below 7 GB (`du -sh`);
  do not run coverage (`cargo llvm-cov`) or `cargo xtask verify` (the coordinator does).
- A build or a test run outlasts a tool call: start it with `nohup … > <log> 2>&1 &` and poll the
  log; kill a job by pid, never with `pkill -f`. The VM can restart (background jobs die, files
  stay): check `uptime` and `ps` before waiting on a job.
- The original game's files are in `/home/claude/homm3/original-data` (read-only, never commit
  them). The code-reading tool works from the main checkout:
  `cd /home/claude/homm3 && python3 scripts/exe.py fn <address> --from <a> --to <b>`
  (`docs/exe/README.md`); VCMI 1.7.5 is at `/home/claude/vcmi`. Use them when a fix needs a detail
  the finding does not give. No disassembly listings in commits, comments or notes (ADR-0009):
  own words and addresses.

## Read first

- `AGENTS.md`: "Non-negotiables" and "Working rules".
- `docs/HANDOFF.md`: "Coverage, lints, formatting"; `docs/lanes/battle/HANDOFF.md`: "Standing
  traps" (test traps: `activate`, `BattleState::new`, the synthetic pack patched by text, golden
  replays).
- For each fix, the finding it names in the reviewers' reports (read only those sections; each
  has the evidence, the scenario and usually a test sketch). The robustness reviewer's failing
  tests are in `<SCRATCH>/review/robustness-failing-tests.patch` (and
  `robustness-hidden-obstacles-regression.patch`): take the tests your fixes need from there.

## How to do one fix

1. **Test first.** Write the test (from the sketch) and run it on the unchanged code. It must fail
   for the reason the finding gives. If it passes, or fails for another reason, do not force it:
   find out why. If the finding turns out to be wrong or the intended behaviour unclear, leave
   that fix, write what you saw into your notes and go on with the next.
2. Fix the code in the smallest way that gives the intended behaviour stated in your list. Keep
   `// rule: <ID>` markers; tests carry `// covers: <IDs>`. Update code comments that describe the
   old behaviour. A test that pinned the old behaviour is changed, with a short comment that names
   the finding.
3. Run the new test and the module's tests (`cargo test -p h3-battle --lib <module>`), then go on.
   Put a new test next to the existing tests of the same rule, never at the end of a file (the
   other worker adds tests to the same files; appended blocks collide when the branches join).
4. Commit each fix (or a tight group) on its own:
   `battle: E04b T11b review — <what changed> (<finding ids>)` (prefix `pack:`, `content:`, `sim:`
   for other crates), a body that says what the original does and by which source (the reviewer's
   re-read with addresses in own words, VCMI file, batch row), what the engine did, what it does
   now, and these two last lines exactly:
   ```
   Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>
   Claude-Session: https://claude.ai/code/session_01YL987pakGrLu1G5DK369iw
   ```
   Push after each commit: `git push -q -u origin <your branch>`.

## Hard limits (stop that fix and report instead)

- No change to `crates/h3-kernel` or `crates/h3-contracts` (shared paths), to `Cargo.toml`,
  `xtask`, CI or any `docs/` file. Rule texts, the quirks ledger, `installation.md` and
  `OPEN-QUESTIONS.md` are updated later by someone else from your notes.
- No new variant of `Command`, `EventKind`, `CommandError`, `CastTarget`, `DamageRoll` or
  `ObstacleKind`, and no new field in `Observation`'s structs (they are on the server's wire).
- No version bump (state hash, ruleset hash, replay, encodings, C ABI, pack format). If a fix
  cannot be done without a new state field or a changed encoding, stop and report.
- Do not regenerate golden replays or benchmark baselines. If `cargo test -p h3-cli` shows a
  golden replay changed, say which and why you think so.
- No new quirk flag and no change to which profile has which flag.
- Purity and determinism: no I/O, clock, threads, globals, `HashMap`/`HashSet`, no floats outside
  the kernel's audited surface (`h3_kernel::arith::X87` emulates the original's x87 at 53 bits;
  test against vectors from exact rational arithmetic), no sort without a unique tie-breaker.
- Coverage is gated at 100 % of lines and regions for `h3-battle`, `h3-content` and `h3-heroes`:
  every branch you add needs a fixed test that reaches it (not only a proptest); add no defensive
  dead branch; prefer saturating operations and infallible accessors.
- No `todo!`, `unimplemented!`, `#[ignore]`.

## When your list is done

- `cargo fmt --all`; `cargo clippy -p <each crate you touched> --all-targets -- -D warnings`.
- `cargo test -p h3-battle -p h3-content -p h3-heroes -p h3-pack -p h3-sim -p h3-cli -p h3-ffi -p h3-protocol`
  (the protocol crate maps the battle's types; the cli crate holds the golden replays). Run the
  battle crate's tests three times (property tests are randomly seeded).
- `cargo xtask check-rules`, `cargo xtask check-arch`, `cargo xtask check-stubs`.
- Write `<SCRATCH>/review/fix-notes-<your name>.md`: for each fix: done or not (why), the commit,
  the tests; **the exact sentences of rule summaries, ledger entries and `installation.md` that
  are now wrong and the wording you propose** (rule id, old clause, new clause); golden replays
  that changed; anything you met that the coordinator should know.

Your final message: at most 20 lines: each fix with done / not done and its commit id, test
failures you could not resolve, golden replay changes, open points.
