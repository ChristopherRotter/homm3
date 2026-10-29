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
  **Done 2026-10-28:** T1a, T1b (a world's reward v1, the true world), T2a (shaping, the refused
  mixes, T2b's configs), all C ABI v35; T1c (the search gains nothing with either world: the
  network's evaluation limits it); T2b (`e25-control-1`, `e25-shaping-1`: shaping about +0.04
  against Aggressor, Greedy and the system actor, 0.409; loops in the tactics phase); T3 (the
  user: increment 2 is shaping, bounded, then scale; increment 1 in `E25-archive.md`); T4a
  (`[network] tactics_rounds`, the agent's tactics bound, Python only); T4b (`e25-bound-1`: the
  bound costs nothing, the loop gone, 73 minutes against 90; **`e25-bound-1` the current best**,
  the user). **Next:** T5, `e25-scale-1` on the user's machine (`configs/e25-scale.toml`: 60 M,
  three tactics rounds, the user at T4b), then T6's checkpoint.
- **Open pull requests:** E25 T4b's (`claude/practical-fermat-xmv41h`, restarted from `master`
  after #558): watch it to its merge. Merged: #534, #539, #555, #558.
- **Versions:** battle action encoding v13, observation encoding v22, replay format v31
  (CLT-Q4/Q6), the adventure env's action encoding v5 (E08 T63a), C ABI v35 (E25 T1a, T1b, T2a:
  a world's reward, the true world, the potential) (`docs/08-work-process.md`, "Versions"). Next
  ids: AI-Q11, AI-QUIRK-1, AI-STUB-2.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-10-29 — E25 T4b

- #558 (T4a) merged, `master` green. The user ran `e25-bound-1` (20 M) and uploaded its record,
  metrics and `best.pt`: the record committed and read (`E25-log.md` T4b; `battles.py` on its
  network in the sandbox, shaping's numbers T2b's: its `best.pt` is gone from the sandbox). The
  user: `e25-bound-1` the current best; T5 at 60 M with three tactics rounds
  (`configs/e25-scale.toml`).

### 2026-10-28 — E25 T4a

- #555 merged before the start (`master` green). T4a: the binding counts a tactics phase's
  turns from the observations (`BattleBatch.tactics_round`), `Policy.act` and the search's
  nodes bound the entry mask by the network's `tactics_rounds`; tests, `configs/e25-bound.toml`.
  The form put to the user: one round, each stack placed or passed once.

### 2026-10-28 — E25 T2b

- The user ran both configs at 20 M samples and uploaded both records and metrics and the
  control's `best.pt` (shaping's stays on their machine). The records committed and read
  (`E25-log.md` T2b): shaping is the first lever past the runs' spread since E06 T5d; its long
  battles (0.2 % cut in training, ladders three to four times slower) are a placement loop in
  the tactics phase (both `best.pt`s, `records/inspect/battles.py`). `master`'s CI
  red since #534's merge: a world's property test hit AI-STUB-1's magic limits; fixed here.
  T3: the user chose increment 2 (shaping, bounded, then scale), refined.
