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
  +0.011 (temperature 0.5), no encoding change asked. **T31 in progress** (the update's speed,
  `claude/pensive-mendel-2ydhkl`): the profile tool `h3env.train.speed` built; next the user's
  GPU run of it (a few minutes), then the change that pays (`QUEUE.md`).
- **Open pull requests:** none. Merged: #765 (T27), #827 (T28, T29), #829 (T30).
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
  minutes on the CPU); #827 opened, CI green. The user then pasted the head-to-head and answered
  AI-Q17 (T29): the current best, increment 8 with its budget and probe families. #827 merged;
  the branch restarted from `master` for T30 (the sandbox's CPU: the temperature ladder 76
  minutes, the ceiling with its confusion table a few minutes beside it).

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
