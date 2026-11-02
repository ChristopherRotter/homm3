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
  **Increment 5** (refined at T13, AI-Q14): the battle lane's fair system actor (E26, #647)
  becomes `system` everywhere, the all-seeing one `system-all-seeing` beside it (the target stays
  the all-seeing one); the neutral kinds back in training with the fair defender; a warm start by
  cloning it. **T14 done** (the switch, C ABI v36: the fair actor 0.500 against the all-seeing
  one, the neutral target stays +0.285). **Next:** T15 (needs `e25-sharp-1` #2's `best.pt`
  uploaded again) and T16 (the cloning bench, sandbox); T17's A/B on the user's machine.
- **Open pull requests:** #651 (`claude/relaxed-ptolemy-eyjild`: T13, T14). Merged:
  #640 (T8, T9a, T10a), #646 (T9b to T12).
- **Versions:** battle action encoding v13, observation encoding v22, replay format v31
  (CLT-Q4/Q6), the adventure env's action encoding v5 (E08 T63a), C ABI v36 (E25 T14: the fair
  system actor in seat 4, the all-seeing one in seat 5) (`docs/08-work-process.md`, "Versions"). Next
  ids: AI-Q15, AI-QUIRK-1, AI-STUB-2.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-11-02 — E25 T13

- #646 merged; the branch started from `master`. T13 with the user (AI-Q14): increment 5's
  decisions and tasks T14 to T18 in the epic and the queue (`E25-log.md` T13). Found: the
  adventure env has no system actor to switch (its neutral seat is the caller's; E13's). T14
  built in the sandbox (the bots' tables regenerated, the neutral bench re-baselined); the
  setup's full baseline verify was stopped (CI is the record), the scoped verify ran instead.

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
