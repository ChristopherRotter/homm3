# AI lane — QUEUE

Pre-refined tasks of the AI lane, in order, then the inbox of discovered work. The task format, the runner/worker model and the `in progress` mark: `docs/08-work-process.md`, "Queues and long runs".

## Tasks

**E05** (`docs/epics/E05-battle-env.md`, refined 2026-10-18, AI-Q1). Increments 1 and 2 done;
increment 3 done 2026-10-19 (#253, #255, #257, #258); increment 4 (T8) done 2026-10-19 (#260,
#264; exit criterion 7 met).

Increment 6 (T9, the closure) cut 2026-10-19 (the epic: T9a to T9d; AI-Q7 moves to E06).

1. **T9a The audit** and **T9b The independent review** (in the sandbox): `in progress`
   2026-10-19 (`claude/festive-brown-igukig`).
2. **T9c The user's machine**: the bench sweep and the Windows verify with original data; the
   Linux verify with original data in a sandbox with the user's files staged (criteria 8, 9).
   On `master` after this lane's T9 pull request, in PowerShell (the conda env of
   `python/README.md` for the Python line):
   ```powershell
   cargo build --release -p h3-cli -p h3-ffi
   foreach ($n in 64, 256, 1024) { foreach ($t in 1, 6, 12, 24) {
     .\target\release\h3.exe ai bench --encounters default --envs $n --threads $t --calls (1048576 / $n) } }
   .\target\release\h3.exe ai bench --encounters default --envs 256 --threads 12 --calls 4096
   uv --directory python run python -m h3env.bench --encounters --envs 256 --threads 12 --calls 4096
   cargo xtask verify --with-original-data --summary
   ```
   Record the 12 lines' two steps/s, the Rust/Python pair and the verify's steps in
   `E05-log.md`. If another lane's step fails the full verify (E04c's precedent: `h3-server`'s
   coverage on Windows), add `cargo xtask verify --scope ai --with-original-data --summary`.
3. **T9d The closure**: the `shared/` pull request, the design note folded, the epic closed.

## Inbox

Sorted by destination at E05's audit (T9a, 2026-10-19); the origin stays in brackets. Dropped as
done: BAT-Q33's point for E05 (AI-Q3, ADR-0025 point 6), the `h3-game` coverage gap of a pending
Pass offer (fixed by the adventure lane's fixed tests, #266).

**For E06** (training at scale, evaluation, search):
- **Evaluation:** greedy play (the most likely heads) repeats itself against Random until the
  step limit (up to 17 % of the battles in T8b's sandbox run); the sampled policy never. E06's
  evaluation and any search agent need a rule for it (sampled play, or a tie-break or repetition
  rule); the 2 000-step limit is only a guard (T8b). The VCMI oracle harness was never built:
  revisit with E06's evaluation needs (E04 refinement, 2026-10-01).
- **Baselines and AI-Q7:** Greedy value beats Aggressor only 0.519 on the held-out episodes: the
  preview of its own kills buys little without spells or a plan; a stronger baseline (casts, a
  one-ply look at the opponent's reply) is E06's, and AI-Q7 (whether the agents get the preview)
  moved there (T7c; the user, 2026-10-19).
- **Encodings:** `h3_sim::action::encode` answers without a viewer (the same root as F-sec-2's
  leak, which the sim's `step` closes); it leaks nothing today (E04b T11b, the battle lane).
- **Performance**, each worth a look when a profile of training shows it: the flat observation
  costs about a third of a step (`observe` ≈ 1 µs, `observe + encode` ≈ 3.5 µs; the buffer is
  zeroed and mostly rewritten zeros); `state_hash` ≈ 4.3 µs at 6.3 KB, too slow for per-node
  hashing (E04b T11a); an encounter sample costs 50 µs (two heroes derived, each artifact through
  `apply_change`'s checks; a `Vec` per level-up's skill draw), 2 % of an episode (T5b2); the
  trainer copies each call's observations (9 MB at 256 envs) into the rollout, which the batch
  could write in place (`H3BatchBuffers.obs` per call) (T8b). The battle lane's `full` workload
  is lopsided (8 attacker victories in 2 399 agent seeds; E04b T11a): the encounter workload
  (`batch-random-encounters-v1`) is the realistic one.

- **Encounters v2** (with the adventure's logged encounters, decision 7; E05 T9b): A7's "+0.5 a
  week" in weeks 9 to 16 is rounded down in `encounters-v1.toml` (odd weeks half a level low);
  untested: A4's cap and redraws in weeks 1 to 6, the forts of weeks 2 to 6, A7's war-machine
  chance and class split, A6's upgraded share, the guards' anti-magic share.
- **The C ABI** (E05 T9b): `h3_batch_create` takes any N (an absurd one aborts on its
  allocation, which no `catch_unwind` catches; the binding range-checks it, a trusted local
  consumer, ADR-0011); `h3env.tools.Env` keeps its handle after `H3_ERR_PANIC` (tests only).

**For E23** (the system actor; the neutral kinds' share): A3 grows a wandering stack by a tenth
of its count, VCMI 1.7.5's `CGCreature::newTurn` a tenth of its power (a thousand per creature:
5 creatures are 9 after seven weeks, the count's tenth leaves them 5); VCMI's growth makes
late-week stacks pass A4's cap and be drawn again, so decide both together (E05 T9b).

**For E13** (the adventure env's ABI): reading both of the adventure env's mask heads doubles
its step; the current state's queries belong in the env (the adventure lane, E08 T9b).
