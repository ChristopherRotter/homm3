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
  (ADR-0026 point 8 as amended).
  **Done:** increments 1 to 6 (the search's diagnostics and shaping; the tactics bound and
  scale; the network's architecture; the policy's sharpness; the fair system actor; better
  imitation), in `E25-archive.md`. Since increment 5 (AI-Q14) `system` is the battle lane's fair
  actor everywhere, `system-all-seeing` the target (C ABI v36), the neutral kinds train again with
  the fair defender, and `h3_batch_choice` (v37) clones it. **`e25-imitate-1` #2 the current
  best** (T23, AI-Q16: PPO from T20's 16 M clone with the imitation loss, weight 0.5 falling to 0
  at 10 M; **0.458 and 0.462 against the fair and the all-seeing actor, value −0.020, the neutral
  set +0.287**, target +0.285; 0.534 head to head against `e25-clone-1` #2). Increment 6's read:
  the fair actor repeats itself 0.972 (T19), the MLP's clone reaches 0.576 (T20), PPO leaves the
  teacher unless the loss holds it (0.317 against 0.276), and the lineage moved about 0.01 (T22).
  **Increment 7, a network that reads the battle's structure** (refined at T23 with the user: the
  legal mask an input, the cost cap hexconv 64 × 6's 466 µs a sample-pass on the GTX 1070),
  measured first on the cloning bench against the fair actor. **T24 done** (`hexconv`'s `norm`,
  `feed_forward`, the bench's warm-up, AdamW and cost, `network --bench`): on the CPU decision 2's
  ranges mostly exceed decision 3's cap (`E25-log.md`, T24; T26's GPU table decides, the user's
  call there). **Next:** T25 in the sandbox (the heads read the geometry).
- **Open pull requests:** the one of `claude/cool-mccarthy-cfodpk` (T24). Merged: #696 (T21),
  #718 (T22, T23).
- **Versions:** battle action encoding v13, observation encoding v22, replay format v32
  (CLT-Q40), the adventure env's action encoding v6 (E08 T85b), C ABI v37 (E25 T16:
  `h3_batch_choice`; v36 the fair system actor in seat 4, the all-seeing one in seat 5)
  (`docs/08-work-process.md`, "Versions"). Next
  ids: AI-Q17, AI-QUIRK-1, AI-STUB-2.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-11-04 — E25 T24

- #718 merged; the branch `claude/cool-mccarthy-cfodpk` started from `master`. The setup's
  baseline ran only the light gates (CI had verified `master`'s code); `cargo build --release -p
  h3-ffi` 90 s, `uv sync --group train` a few minutes (no stall). The Python suite 205 passed
  (3 minutes), the grid's CPU table 197 s, the CPU clone smoke 35 minutes (the hex transformer
  alone 25 of them).

### 2026-11-04 — E25 T22 and T23

- #696 merged; the branch started from `master`. The user ran both arms before T22 was prepared
  (the weight and horizon as proposed, confirmed after) and uploaded each run's `record.json`,
  `metrics.jsonl` and `best.pt`, then `e25-clone-1`'s `best.pt` (identified by T19's tool). In the
  sandbox `cargo build --release -p h3-ffi` 79 s and `uv sync --group train` 63 s (no stall); T19's
  tool 40 s a run, the head-to-heads 24 to 215 s. T23 with the user (AI-Q16); increment 7's survey
  by a sub-agent. Without the setup script the coverage step needed `cargo-llvm-cov`, `llvm-20`
  and `scripts/sandbox-verify.sh` (the scoped verify then passed, 8 minutes).

### 2026-11-03 — E25 T21

- #687 merged; the branch started from `master`. The setup's full baseline was stopped (CI is the
  record), the scoped verify ran instead. `uv sync --group train`'s cuDNN download stalled at
  about 70 KB/s for over an hour; killed and restarted, it finished in 9 s from uv's cache (the
  trap in `TRAPS.md`). T21 built and its CPU check run in the sandbox (the clone 5 minutes, each
  PPO run 5 to 6, T19's tool 38 s); the VM restarted once, nothing lost.
