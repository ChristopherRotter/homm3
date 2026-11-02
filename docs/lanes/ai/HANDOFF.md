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
  **Done:** increments 1 to 4 (the search's diagnostics and shaping; the tactics bound and
  scale; the network's architecture; the policy's sharpness), in `E25-archive.md`; every lever
  ends at 0.41 to 0.43 against the system actor. **`e25-sharp-1` #2 the current best** (T11,
  AI-Q13; 0.422 against the system actor; T12: 0.523 head to head against `e25-scale-1` #3).
  **Increment 5** is built around the battle lane's fair system actor v1 (landed 2026-11-02,
  E26, #647; the AI queue's inbox). **Next:** T13, increment 5's refinement, with the user.
- **Open pull requests:** #646 (`claude/quirky-dirac-h5xdj2`: T9b to T12). Merged: #640 (T8,
  T9a, T10a).
- **Versions:** battle action encoding v13, observation encoding v22, replay format v31
  (CLT-Q4/Q6), the adventure env's action encoding v5 (E08 T63a), C ABI v35 (E25 T1a, T1b, T2a:
  a world's reward, the true world, the potential) (`docs/08-work-process.md`, "Versions"). Next
  ids: AI-Q14, AI-QUIRK-1, AI-STUB-2.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-11-02 — E25 T8 to T12

- #610 merged; the branch started from `master`. T8: the user chose increment 4 and kept #3
  (AI-Q12; the user's caveat on T7c's structured networks in `E25-log.md` T8). T9a and T10a's
  schedule built, #640 merged; the branch restarted from `master`. T9b on the sandbox's CPU from
  #3's uploaded `best.pt` while the user ran `e25-sharp-1` (T10b, its record committed). T11
  (AI-Q13): increment 4 closed, increment 5 waits for the fair system actor. T12 on both uploaded
  `best.pt`: `e25-sharp-1` #2 promoted against #3, the current best. A VM restart cost nothing
  (all committed or in `runs/`).

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
