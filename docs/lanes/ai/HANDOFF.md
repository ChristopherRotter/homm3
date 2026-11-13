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
  user's GPU: the update's speed, architectures, PPO's settings, training regimes; the aim a
  margin beyond the interval). **T30 done** (`E25-log.md` T30): the margin lies in the hero
  battles (#2 keeps more army there but wins as often as the fair actor), sharpening gains
  +0.011 (temperature 0.5), no encoding change asked. **T31 done** (`E25-log.md` T31): cuDNN's
  autotuner in the update, `[ppo] chunk_align`; a `hexconv` run sets `accumulate = 4`,
  `chunk_align = 32`: 98 to 67 s an update on the GTX 1070. **T33 done** (`E25-log.md` T33):
  no continuation from #2 gains; head to head against #2 the bonus at 0 0.494, a quarter of the
  rate 0.493, the imitation loss held 0.478, that rate annealed to #2's floor 0.503 (± 0.002,
  level), the ladders inside #2's interval: **#2 sits at this setup's plateau**, so T35's long
  run starts from T32's best clone. **T32 prepared** (`QUEUE.md`; the arms the user's: the
  control `hexconv` 96 × 6, it at 2e-3, 128 × 6, 64 × 10, 4 M each on T26b's bench with T27's
  ladders, `configs/distill/e25-arch.toml`): **next: the user runs it** (about 7 hours in parts
  under 2 hours; the commands in `QUEUE.md`), then its read names the long run's network; then
  T34, T35.
- **Open pull requests:** the one of `claude/e25-continuation-e2ydg6` (T33's close, T32's
  preparation). Merged: #835 (T31), #843 (T33's preparation), #862 (T33's round 1).
- **Versions:** battle action encoding v13, observation encoding v22, replay format v32
  (CLT-Q40), the adventure env's action encoding v6 (E08 T85b), C ABI v37 (E25 T16:
  `h3_batch_choice`; v36 the fair system actor in seat 4, the all-seeing one in seat 5)
  (`docs/08-work-process.md`, "Versions"). Next
  ids: AI-Q18, AI-QUIRK-1, AI-STUB-2.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

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

### 2026-11-11 — E25 T33 (prepared)

- #835 merged; the branch started from `master`. `cargo build --release -p h3-ffi` 64 s, `uv sync
  --group train` a few minutes; the setup's engine baseline in the background, so the edits in a
  worktree, its tests on the main clone's venv (`H3_LIBRARY`). Built `[ppo] imitation_floor`, the
  gate's `--fixed` and `--json`, round 1's configs; a CPU smoke of the imitation arm (stopped by
  a timeout, resumed, its end gate); the scoped verify, the Python suite 258 (2.6 minutes).
