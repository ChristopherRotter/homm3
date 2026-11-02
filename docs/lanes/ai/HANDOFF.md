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
  the target is beating the faithful system actor (`t7-1`: 0.372); it **trains no agent any
  more** (ADR-0026 point 8 as amended; no neutral battles in training for now, the user at T2a).
  **Done:** increments 1 (the search's diagnostics, shaping; C ABI v35), 2 (the tactics bound;
  the scale run flat from 20 M) and 3 (the network's architecture: the wide MLP leads the
  distillation bench, in RL it ends at #3's ladder; the structured networks, as configured, fit
  ten times worse: the user's caveat at T8), in `E25-archive.md`. **`e25-scale-1` #3 the
  current best** (T6, kept at T8; 0.418 against the system actor).
  **Increment 4** (T8, AI-Q12, the user, 2026-11-02): the policy's sharpness and PPO's settings,
  T9a to T11 in `QUEUE.md`. T9a (`ladder --temperature`) and T10a's entropy schedule
  (`[ppo] entropy_floor`, `configs/e25-sharp.toml`) built. **Next:** the user's machine: T9b
  (#3's ladder at temperatures 1, 0.7, 0.5, 0) and `e25-sharp-1` (20 M); then T10a's second
  lever.
- **Open pull requests:** this session's (`claude/quirky-dirac-h5xdj2`: T8, T9a, T10a's
  schedule). Merged: #610 (T7d).
- **Versions:** battle action encoding v13, observation encoding v22, replay format v31
  (CLT-Q4/Q6), the adventure env's action encoding v5 (E08 T63a), C ABI v35 (E25 T1a, T1b, T2a:
  a world's reward, the true world, the potential) (`docs/08-work-process.md`, "Versions"). Next
  ids: AI-Q13, AI-QUIRK-1, AI-STUB-2.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-11-02 — E25 T8, T9a, T10a (the schedule)

- #610 merged; the branch started from `master`. T8: the user chose increment 4 (a) and kept #3
  (AI-Q12 answered; increment 3 archived; T9a to T11 queued; the user's caveat on T7c's
  structured networks in `E25-log.md` T8 and the epic's candidates). The sandbox's baseline: the
  light gates (CI verified `master`'s code). T9a and T10a's schedule built (177 Python tests
  pass); T9b and `e25-sharp-1` wait for the user's machine (or #3's `best.pt` uploaded).

### 2026-10-30 — E25 T7d

- #604 merged; the branch started from `master`. The user ran `e25-wide-1` (20 M, 94 minutes)
  and uploaded its record and metrics: committed and read (`E25-log.md` T7d). Its `best.pt`
  (49.5 MB) is above the upload limit (`TRAPS.md`, Python), so `battles.py` was not run; T8's
  question AI-Q12 written. The sandbox's baseline: the light gates (CI verified the code).

### 2026-10-30 — E25 T7c

- #585 merged; the branch restarted from `master`. The user ran the distillation bench (4 M
  samples, GTX 1070) and uploaded its report and metrics: committed and read (`E25-log.md` T7c).
  The user: T7d is the wide MLP alone, from scratch at 20 M (`configs/e25-wide.toml`, its test in
  `test_run.py`). The bench's students stay on the user's machine (`runs/distill-1/students/`).
