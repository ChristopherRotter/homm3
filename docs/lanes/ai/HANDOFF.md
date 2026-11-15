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
  **Increment 8: probes, then one long run** (the epic file; the probes about 20 hours of the
  user's GPU). **T30 done** (`E25-log.md` T30): the margin lies in the hero battles, sharpening
  +0.011, no encoding change asked. **T31 done**: a `hexconv` run sets `accumulate = 4`,
  `chunk_align = 32` (98 to 67 s an update). **T33 done**: no continuation from #2 gains; **#2
  sits at this setup's plateau**. **T32 done** (`E25-log.md` T32, `records/distill/arch-1.json`):
  at 4 M the control 0.752, 2e-3 0.760, 128 × 6 0.761, 64 × 10 0.753, the ladders inside the
  control's interval: `hexconv` 96 × 6 stays, T35's long run starts from T27's 16 M clone (#2's
  start). **T34 prepared** (`QUEUE.md`; the arms the user's): (a) the search at play with #2 on
  the sandbox's CPU (about 3 hours; **needs #2's `best.pt` uploaded**), (b) the KL anchor,
  `configs/e25-anchor.toml` (`[ppo] imitation_teacher`, #2 teaching T27's clone, 2 M; **the
  user's GPU, about 2 hours**). Then T35, the probes' checkpoint.
- **Open pull requests:** the one of `claude/stoic-gauss-p3cmdq` (T32's close, T34's
  preparation). Merged: #910 (T33's close, T32's preparation).
- **Versions:** battle action encoding v13, observation encoding v22, replay format v32
  (CLT-Q40), the adventure env's action encoding v6 (E08 T85b), C ABI v37 (E25 T16:
  `h3_batch_choice`; v36 the fair system actor in seat 4, the all-seeing one in seat 5)
  (`docs/08-work-process.md`, "Versions"). Next
  ids: AI-Q18, AI-QUIRK-1, AI-STUB-2.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-11-15 — E25 T32 closed, T34 prepared

- #910 merged. The user uploaded T32's `report.json` and `metrics.jsonl` (no CRLF; the metrics
  not committed: the report holds every evaluation). No setup script: `uv sync --group train`
  and `cargo build --release -p h3-ffi` (53 s) in the background; the Python suite 264 (3.3
  minutes). The user chose T34's arms (the search at play, the KL anchor).

### 2026-11-13 — E25 T33 closed, T32 prepared

- #862 merged. No computer folder linked: the user uploaded round 2's `record.json`,
  `metrics.jsonl` and the three measures (the measures with CRLF, stripped on commit; the
  update records read from `metrics.jsonl`, not committed). The user chose T32's arms. No setup
  script run: cargo-llvm-cov, llvm-tools and llvm-20 installed by hand for the scoped verify,
  `origin/master` fetched first (`check-lane` read a stale base); `uv sync --group train` for
  T32's config test.

### 2026-11-11 — E25 T33 (round 1 read, round 2 prepared)

- #843 merged. The session linked to the user's computer: round 1's outcomes read in place
  (`python/runs/`, `device_bash`), the twelve measure files staged to the sandbox and committed;
  the user chose round 2's arm. The VM restarted early (the setup's baseline lost, CI's green
  `master` stood for it); `cargo build --release -p h3-ffi` 2 minutes, `uv sync --group train`
  about 10; the scoped verify 11 minutes, the Python suite 259 (2.9 minutes).
