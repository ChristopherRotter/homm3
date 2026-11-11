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
  autotuner in the update (`[device] cudnn_autotune`, on), `[ppo] chunk_align`; a `hexconv` run
  sets `accumulate = 4`, `chunk_align = 32`: 958 to 649 µs a sample-pass, 98 to 67 s an update
  on the GTX 1070; `gpu.cap` now passes the card's index (every CUDA entry point since T28
  stopped there). **T33 in progress** (`QUEUE.md`): round 1 done (`E25-log.md` T33): no arm
  gains, head to head against #2 the bonus at 0 0.494, a quarter of the rate 0.493, the
  imitation loss held 0.478 (± 0.002), the ladders inside #2's interval; each ran at 5 to 20
  times #2's final rate, and the entropy rose without the bonus. Round 2 prepared (the user's
  choice): `e25-continue-anneal-1`, the quarter rate annealed to #2's floor over 2 M; **next: the
  user runs it** (about 2 hours; the commands in `QUEUE.md`), then its read closes T33 and says
  whether T35's long run starts from #2 or from T32's clone; then T32.
- **Open pull requests:** the one of `ai/e25-t33-round1` (T33's round 1 and round 2's
  preparation). Merged: #829 (T30), #835 (T31), #843 (T33's preparation).
- **Versions:** battle action encoding v13, observation encoding v22, replay format v32
  (CLT-Q40), the adventure env's action encoding v6 (E08 T85b), C ABI v37 (E25 T16:
  `h3_batch_choice`; v36 the fair system actor in seat 4, the all-seeing one in seat 5)
  (`docs/08-work-process.md`, "Versions"). Next
  ids: AI-Q18, AI-QUIRK-1, AI-STUB-2.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

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

### 2026-11-10 — E25 T31

- #829 merged; the branch started from `master` (the setup's baseline only the light gates);
  `cargo build --release -p h3-ffi` 56 s, `uv sync --group train` about 30 minutes (3 MB/s).
  Built `h3env.train.speed`; the user ran it five times (about 6 minutes each): the first
  stopped at `gpu.cap` (fixed), then the variants, the sweep by rows (its autotuner column void:
  cuDNN's cache, `TRAPS.md`), the sweep again, the confirmation. The Python suite 254 (2.4
  minutes).
