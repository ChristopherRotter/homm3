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
  one, the neutral target stays +0.285). **T15 done:** `e25-sharp-1` #2 0.424 against the fair
  actor, 0.425 against the all-seeing one, the neutral set +0.245 (target +0.285). **T16 done**
  (C ABI v37, `h3_batch_choice`; the clone mode `distill --teacher system`): 4 M samples on the
  sandbox, agreement 0.531, the clone 0.396 against the fair actor and +0.244 on the neutral set
  with no RL; its `students/mlp.pt` sent to the user. **Next:** T17's A/B on the user's machine
  (`configs/e25-neutral.toml`, from scratch and `--init-from` the clone; lengths agreed then).
- **Open pull requests:** #657 (`claude/quirky-cori-hjtoat`: T16). Merged: #646 (T9b to
  T12), #651 (T13 to T15).
- **Versions:** battle action encoding v13, observation encoding v22, replay format v31
  (CLT-Q4/Q6), the adventure env's action encoding v6 (E08 T85b), C ABI v37 (E25 T16:
  `h3_batch_choice`; v36 the fair system actor in seat 4, the all-seeing one in seat 5)
  (`docs/08-work-process.md`, "Versions"). Next
  ids: AI-Q15, AI-QUIRK-1, AI-STUB-2.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

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

### 2026-11-02 — E25 T8 to T12

- #610 merged; the branch started from `master`. T8: the user chose increment 4 and kept #3
  (AI-Q12; the user's caveat on T7c's structured networks in `E25-log.md` T8). T9a and T10a's
  schedule built, #640 merged; the branch restarted from `master`. T9b on the sandbox's CPU from
  #3's uploaded `best.pt` while the user ran `e25-sharp-1` (T10b, its record committed). T11
  (AI-Q13): increment 4 closed, increment 5 waits for the fair system actor. T12 on both uploaded
  `best.pt`: `e25-sharp-1` #2 promoted against #3, the current best. A VM restart cost nothing
  (all committed or in `runs/`).
