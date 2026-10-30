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
  **Done 2026-10-28/29:** increments 1 (the search's diagnostics, shaping: +0.04; C ABI v35) and
  2 (the tactics bound; the scale run flat from 20 M), in `E25-archive.md`; **`e25-scale-1` #3
  the current best** (T6, the user; 0.418 against the system actor). Increment 3, the network's
  architecture (AI-Q11): T7a (`hexconv`, `hexformer`, the MLP's `layers`), T7b (the bench,
  `h3env.train.distill`).
  **Done 2026-10-30:** T7c (the bench on the user's GPU, `records/distill/distill-1.json`: the
  structured networks fit ten times worse at 5 to 25 times the cost; the wide MLP 2 × 1 024 leads,
  its value past the teacher's; the user: it alone to RL, from scratch), T7d (`e25-wide-1`,
  20 M: `e25-scale-1` #3's ladder, 0.414 against the system actor, at 0.7 times its speed).
  **Next:** T8, increment 3's checkpoint: **AI-Q12 waits for the user** (increment 4's options,
  the current best).
- **Open pull requests:** the T7d branch's (`claude/bold-archimedes-vivw35`). Merged: #585,
  #604 (E25 T7b, T7c).
- **Versions:** battle action encoding v13, observation encoding v22, replay format v31
  (CLT-Q4/Q6), the adventure env's action encoding v5 (E08 T63a), C ABI v35 (E25 T1a, T1b, T2a:
  a world's reward, the true world, the potential) (`docs/08-work-process.md`, "Versions"). Next
  ids: AI-Q13, AI-QUIRK-1, AI-STUB-2.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

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

### 2026-10-29 — E25 T5, T6, T7a, T7b

- #576 (T4b) merged. The user ran `e25-scale-1`, stopped it at 41.3 M (flat), uploaded its
  record, metrics and `best.pt`: read (`E25-log.md` T5; no final ladder, so its best replayed on
  the sandbox's CPU). T6 (the user): `e25-scale-1` #3 the current best, increment 3 the
  network's architecture (AI-Q11). T7a (`hexnet`, the MLP's depth) and T7b (the distillation
  bench) built and tested; a CPU smoke of the bench (PPO's minibatches ran out of memory:
  `TRAPS.md`). The sandbox setup's full verify stopped (CI's record).
