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
  measured first on the cloning bench against the fair actor. **T24 to T26a done** (the kinds
  built properly, the heads read the geometry, the cap about 700 µs a sample-pass by decision 3
  as amended, the benches resume; `E25-log.md`). **T26b done** (decision 5's first gate passed
  by `hexconv` normalised 96 × 6 at 1e-3, 0.755 at 4 M, 683 µs; the MLP 0.533). **T27 done** (the
  user's GPU, `E25-log.md` T27): `hexconv` **0.809 at 16 M** (moves 0.780, melee 0.776; the
  MLP 0.574), its value 0.658 (the MLP's 0.626); its ladder **0.481 ± 0.019, value −0.020**
  against the fair actor (the MLP 0.414), 0.473 against the all-seeing one, the neutral set
  +0.274: decision 5's second gate passed, the clone alone at the current best's level.
  **T28 done** (`E25-log.md` T28): `e25-structure-1` #2 (PPO from the clone with the loss) **0.511
  ± 0.019, +0.027 against the fair actor, 0.507, +0.026 against the all-seeing one, the neutral
  set +0.302**: exit criterion 1 met on its point estimates; 217 samples a second (28 hours a 20
  M). The GPU's memory capped at 90 % of the card (`gpu.cap`, `TRAPS.md`). **Next:** T29, the
  checkpoint (AI-Q17), first the head-to-head on the user's machine (`QUEUE.md`).
- **Open pull requests:** the one of `claude/adoring-heisenberg-t5hahk` (T28's records, the GPU's
  cap). Merged: #730 (T26a), #748 (T26b), #765 (T27).
- **Versions:** battle action encoding v13, observation encoding v22, replay format v32
  (CLT-Q40), the adventure env's action encoding v6 (E08 T85b), C ABI v37 (E25 T16:
  `h3_batch_choice`; v36 the fair system actor in seat 4, the all-seeing one in seat 5)
  (`docs/08-work-process.md`, "Versions"). Next
  ids: AI-Q18, AI-QUIRK-1, AI-STUB-2.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-11-10 — E25 T28

- #765 merged; the branch started from `master`, its code verified (only the light gates);
  `cargo build --release -p h3-ffi` 52 s, `uv sync --group train` a few minutes. The user ran
  T28 in 13 segments and uploaded the record, metrics and `best.pt`, with the GPU's slowdown:
  the cap first (the Python suite 248, 2.8 minutes), then T19's tool (66 s) and `inspect` (30
  minutes on the CPU). The head-to-head is the user's (T29).

### 2026-11-06 — E25 T27

- #748 merged; the branch started from `master`. The setup's baseline ran only the light gates;
  `cargo build --release -p h3-ffi` 65 s, `uv sync --group train` a few minutes. The user pasted
  T27's ladder, then its table and `report.json`, then the student (T19's tool, 2 minutes);
  T28's smoke from a CPU clone (72 s, then 2.3 minutes) showed `minibatches` 64 would change
  PPO's steps, so `[ppo] accumulate`; the Python suite 243 (2.6 minutes).

### 2026-11-05 — E25 T26a

- The branch started from `master` with T26's records (`claude/determined-mayer-t1waz5`, no pull
  request). The setup's baseline ran only the light gates (code verified on `master`); `cargo
  build --release -p h3-ffi` 56 s, `uv sync --group train` from the cache. The Python suite 241
  (3.5 minutes); the stop-and-resume smoke of T25's clone by the command line 27 minutes
  (`E25-log.md` T26a), which showed the held-out file's 156 MB (now compressed).
- #730 merged; the user ran T26b to its budget in parts and pasted the table; the branch
  restarted from `master` for its records and T27's config; its `report.json` uploaded after.
