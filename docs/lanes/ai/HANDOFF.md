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
  bound costs nothing, the loop gone; **`e25-bound-1` the current best**, the user). **Done
  2026-10-29:** T5 (`e25-scale-1`, three rounds, stopped by the user at 41.3 M of 60 M: best #3
  at 30 M at `e25-bound-1`'s level, 0.418 against the system actor; scale gains nothing).
  T6 (**`e25-scale-1` #3 the current best**; increment 2 in `E25-archive.md`; AI-Q11: increment
  3 is the network's architecture, refined: a distillation bench over a deeper and a wider MLP,
  a hex convolution network, a hex transformer and the entity network, then A/Bs); T7a (the
  kinds: `hexconv`, `hexformer`, the MLP's `layers`); T7b (the bench, `h3env.train.distill`,
  `configs/distill/e25.toml`). **Next:** T7c, the bench on the user's GPU (the queue's command).
- **Open pull requests:** none. Merged: #534, #539, #555, #558, #576, #585 (E25 T5 to T7b,
  2026-10-29).
- **Versions:** battle action encoding v13, observation encoding v22, replay format v31
  (CLT-Q4/Q6), the adventure env's action encoding v5 (E08 T63a), C ABI v35 (E25 T1a, T1b, T2a:
  a world's reward, the true world, the potential) (`docs/08-work-process.md`, "Versions"). Next
  ids: AI-Q12, AI-QUIRK-1, AI-STUB-2.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-10-29 — E25 T5, T6, T7a, T7b

- #576 (T4b) merged. The user ran `e25-scale-1`, stopped it at 41.3 M (flat), uploaded its
  record, metrics and `best.pt`: read (`E25-log.md` T5; no final ladder, so its best replayed on
  the sandbox's CPU). T6 (the user): `e25-scale-1` #3 the current best, increment 3 the
  network's architecture (AI-Q11). T7a (`hexnet`, the MLP's depth) and T7b (the distillation
  bench) built and tested; a CPU smoke of the bench (PPO's minibatches ran out of memory:
  `TRAPS.md`). The sandbox setup's full verify stopped (CI's record).

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
