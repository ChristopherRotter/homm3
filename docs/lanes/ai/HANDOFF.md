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
  `feed_forward`, the bench's warm-up, AdamW and cost, `network --bench`). **T25 done** (the
  heads read the geometry: `geometry`'s port of the attack positions, `encode(obs, mask)`,
  `[network] mask_input`, `order`, `attack_hex`; `E25-log.md` T25). The user's GPU grid and cap:
  about 700 µs a sample-pass (decision 3 as amended). T26's first segment (the user, stopped at
  0.45 M; `E25-log.md` T26): every structured student ahead of the MLPs, `hexconv` 96 × 6 at
  0.619. **T26a done** (the benches resume, both modes: `distill.go`, `checkpoint.pt`, Ctrl-C at
  the next minibatch, `--resume DIR`; `E25-log.md` T26a). **Next:** T26b on the user's GPU in
  parts under 2 hours (its commands in `QUEUE.md`).
- **Open pull requests:** #730 (`claude/gifted-keller-2dtp2o`: T26's records and T26a; the
  branch `claude/determined-mayer-t1waz5` opened none, its commit is in it). Merged: #721 (T24),
  #725 (T25).
- **Versions:** battle action encoding v13, observation encoding v22, replay format v32
  (CLT-Q40), the adventure env's action encoding v6 (E08 T85b), C ABI v37 (E25 T16:
  `h3_batch_choice`; v36 the fair system actor in seat 4, the all-seeing one in seat 5)
  (`docs/08-work-process.md`, "Versions"). Next
  ids: AI-Q17, AI-QUIRK-1, AI-STUB-2.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-11-05 — E25 T26a

- The branch started from `master` with T26's records (`claude/determined-mayer-t1waz5`, no pull
  request). The setup's baseline ran only the light gates (code verified on `master`); `cargo
  build --release -p h3-ffi` 56 s, `uv sync --group train` from the cache. The Python suite 241
  (3.5 minutes); the stop-and-resume smoke of T25's clone by the command line 27 minutes
  (`E25-log.md` T26a), which showed the held-out file's 156 MB (now compressed).

### 2026-11-04 — E25 T25

- #721 merged; the branch started from `master`. The setup's full baseline was stopped (CI is
  the record); `cargo build --release -p h3-ffi` 78 s, `uv sync --group train` from the cache.
  The user's GPU grid arrived with the task (in `E25-log.md` T24), the cap raised at the end
  (AI-Q16's decision 3). Four narrow workers in turn (the port, the mask through `encode`, the
  structured kinds, the smokes); the VM restarted once during the smokes, the finished PPO runs
  kept and the rest rerun (the clone smoke 22 minutes).

### 2026-11-04 — E25 T24

- #718 merged; the branch `claude/cool-mccarthy-cfodpk` started from `master`. The setup's
  baseline ran only the light gates (CI had verified `master`'s code); `cargo build --release -p
  h3-ffi` 90 s, `uv sync --group train` a few minutes (no stall). The Python suite 205 passed
  (3 minutes), the grid's CPU table 197 s, the CPU clone smoke 35 minutes (the hex transformer
  alone 25 of them).
