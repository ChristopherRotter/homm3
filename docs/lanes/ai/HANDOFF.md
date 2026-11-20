# AI lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **Opened 2026-10-18** (AI-Q1): the lane owns `h3-sim` (its `src/adventure/` too since E08 T45, but `replay.rs`), `h3-ffi`, `h3-cli`'s `src/ai/`, `python/`, `benches/ai/`, E05, E06 and E25 (`lanes.toml`).
- **E05 closed 2026-10-20**: its record and design in `E05-log.md` (cited as §n), the
  environment as built in `docs/07-simulation-and-ai.md` §1.1.
- **E06 closed 2026-10-27** (AI-Q8; `docs/epics/E06-battle-ai.md`, record and design note
  `E06-log.md`, closed increments `E06-archive.md`; the pipeline as built:
  `docs/07-simulation-and-ai.md` §1.2): `t7-1` is the battle AI v1 and the baseline.
- **E25 refined 2026-10-27** (AI-Q10; `docs/epics/E25-battle-ai-v2.md`, record `E25-log.md`):
  the target is beating the faithful system actor (`t7-1`: 0.372), which **trains no agent**
  (ADR-0026 point 8 as amended). Since increment 5 (AI-Q14) `system` is the battle lane's fair
  actor everywhere, `system-all-seeing` the target (C ABI v36), the neutral kinds train with the
  fair defender, and `h3_batch_choice` (v37) clones it.
  **Done:** increments 1 to 7 (the search's diagnostics and shaping; the tactics bound and
  scale; the network's architecture; the policy's sharpness; the fair system actor; better
  imitation; the structured network), in `E25-archive.md`. Increment 7's read: `hexconv` 96 × 6
  with the legal mask clones the fair actor 0.809 (the MLP 0.574, T27), and PPO from that clone
  with the imitation loss carries the lead through RL (T28).
  **`e25-structure-1` #2 the current best** (T29, AI-Q17: **0.511 ± 0.019, +0.027 against the
  fair actor, 0.507 ± 0.019, +0.026 against the all-seeing one, the neutral set +0.302**, target
  +0.285; 0.552 head to head against `e25-imitate-1` #2): exit criterion 1 met on its point
  estimates, the faithful actor's win rate inside its interval; 217 samples a second, 28 hours a
  20 M on the GTX 1070. The GPU's memory capped at 90 % of the card (`gpu.cap`, `TRAPS.md`).
  **Increments 8 and 9 closed** (`E25-archive.md`; `E25-log.md` T30 to T39): no probe (size,
  PPO's settings, the search at play, the KL anchor, the outcome's weight), the long run (0.511
  against the faithful actor) nor v23 (the reach block, the kill preview: T39's bench, 0.457,
  0.460, 0.459 ± 0.018) passed #2 (AI-Q18, AI-Q20). A `hexconv` run sets `accumulate = 4`,
  `chunk_align = 32` (T31); a long run stops early (`[gate] futility`, T36); a bench A/B sets
  `same_start` (T38b; `TRAPS.md`, Python). **Increment 10: the ceiling** (the user, AI-Q20): a
  lower bound on the play possible against the fair actor by rollouts of the fair actor (every
  legal command, common random numbers), on the true battle and the side's worlds, then where
  #2's choices lose value. **T42 done** (`E25-log.md` T42): `h3 ai ceiling`, the rollout-improved
  fair actor on `h3 ai eval`'s sets, sequential halving, a resumable games file; about 190
  thread-seconds a game at budget 8. **Next: T43** on the user's CPU (the budget check, then the
  measurement; the commands in `QUEUE.md`), T44 (where #2 diverges) beside it.
- **Open pull requests:** none. Merged: #1075 (T39, T41, T42), #1040 (T38b). **Owed to a `shared/` pull request:** the
  README's E25 row (increment 9 closed, increment 10 the ceiling next; "as of" 2026-11-20) and
  a line for `h3 ai ceiling` in `docs/07-simulation-and-ai.md` (beside `h3 ai eval`).
- **Versions:** battle action encoding v13, observation encoding v23 (E25 T38a: the reach
  block), replay format v32 (CLT-Q40), the adventure env's action encoding v13 (E09 T39a), C ABI
  v38 (E25 T38a; v37 `h3_batch_choice`) (`docs/08-work-process.md`, "Versions"). Next
  ids: AI-Q21, AI-QUIRK-1, AI-STUB-2.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-11-20 — E25 T39 read, T41: increment 9 closed, increment 10 the ceiling

- #1040 merged. The user ran T39 on the GPU (two segments, 5.1 hours) and uploaded
  `report.json` (committed as `python/records/distill/reach-1.json`). The checkpoint (AI-Q20):
  the user proposed the ceiling; the rollout design agreed (the fair actor, both worlds). T42
  built (`cargo build --release -p h3-cli`, timing runs in the sandbox; the refused casts traced
  to BAT-STUB-19 by a temporary print, removed). #1075 merged. The user limits T43 to 16 threads
  (`TRAPS.md`); a stop during an append now drops the torn last game on resume.

### 2026-11-19 — E25 T38b: `[network] reach` and `preview`, the bench's preview rows

- #1036 and the `shared/` docs/07 line merged. No original data (none needed). `cargo build
  --release -p h3-ffi`, `uv sync --group train`; the scoped verify (coverage's tool installed
  by hand: the setup script was not run). T39's bench as a CPU smoke (`--set` small), stopped by
  SIGINT and resumed. Python only. #1040, watched until it merges.

### 2026-11-19 — E25 T38a: the reach query, observation v23, C ABI v38

- #1028 merged. No original data (none needed). `cargo build --release -p h3-ffi`, `uv sync
  --group train`; the Python suite took 70 minutes beside the benches (alone 3.5). The batch
  workloads measured right after the first build (+340 %), the strike hexes rewritten on a
  neighbour table (+176 %), then re-baselined; a `master` worktree (`target/wt-master`) for the
  siege workload's per-function check. #1036 (`cross-lane`), watched until it merges.
