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
  **Done:** increments 1 to 5 (the search's diagnostics and shaping; the tactics bound and
  scale; the network's architecture; the policy's sharpness; the fair system actor), in
  `E25-archive.md`. Since increment 5 (AI-Q14) `system` is the battle lane's fair actor
  everywhere, `system-all-seeing` the target (C ABI v36), the neutral kinds train again with
  the fair defender, and `h3_batch_choice` (v37) clones it. **`e25-clone-1` #2 the current best**
  (T18, AI-Q15: PPO from T16's clone, 20 M; **0.454 against both system actors, value −0.030,
  the neutral set +0.282**, target +0.285; 0.564 head to head against `e25-sharp-1` #2), the
  first past the 0.41 to 0.43 every earlier lever ended at. **Increment 6, better imitation**
  (refined at T18): the teacher's ceiling, a longer clone, an imitation loss decaying in PPO,
  then their A/B against `e25-clone-1`. **T19 done:** the fair actor repeats its own choice
  0.972 of the time (0.044 nats); T16's student reached 0.531, `e25-clone-1` #2 0.270 (PPO
  leaves the teacher): the gap is the network's. **Next:** T20, the longer clone
  (`configs/distill/e25-clone-2.toml`, 16 M; the user's machine or the sandbox, the user's
  choice), and T21, the imitation loss (a sandbox build).
- **Open pull requests:** #682 (`claude/quirky-cerf-26fspy`: T17 to T19).
  Merged: #651 (T13 to T15), #657 (T16).
- **Versions:** battle action encoding v13, observation encoding v22, replay format v31
  (CLT-Q4/Q6), the adventure env's action encoding v6 (E08 T85b), C ABI v37 (E25 T16:
  `h3_batch_choice`; v36 the fair system actor in seat 4, the all-seeing one in seat 5)
  (`docs/08-work-process.md`, "Versions"). Next
  ids: AI-Q16, AI-QUIRK-1, AI-STUB-2.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-11-03 — E25 T17 to T19

- #657 merged; the branch started from `master`. T17's records and `metrics.jsonl` uploaded by
  the user with the three `best.pt` (`e25-neutral-1`, `e25-clone-1`, `e25-sharp-1` #2); the
  head-to-heads on the sandbox's CPU (25 to 53 s each, `cargo build --release -p h3-ffi` and
  `uv sync --group train` first, about 1 and 7 minutes). The setup's baseline verify passed.
  T18 with the user (AI-Q15): increment 5 closed, increment 6 refined (T19 to T23). T19 on the
  sandbox's CPU (37 s a run; the PPO networks scored beside the teacher).

### 2026-11-02 — E25 T16

- #651 merged; the branch started from `master`. T16 built (C ABI v37's `h3_batch_choice`, the
  clone mode, `configs/e25-neutral.toml`) and run on the sandbox's CPU at the user's 4 M
  (40 minutes; `E25-log.md` T16). The setup's baseline ran only the light gates (`master`'s
  code verified by CI); `uv sync --group train` took about 3 minutes.

### 2026-11-02 — E25 T13

- #646 merged; the branch started from `master`. T13 with the user (AI-Q14): increment 5's
  decisions and tasks T14 to T18 in the epic and the queue (`E25-log.md` T13). Found: the
  adventure env has no system actor to switch (its neutral seat is the caller's; E13's). T14
  built in the sandbox (the bots' tables regenerated, the neutral bench re-baselined); the
  setup's full baseline verify was stopped (CI is the record), the scoped verify ran instead.
  T15 on the uploaded `best.pt` (sandbox CPU, 3 minutes).
