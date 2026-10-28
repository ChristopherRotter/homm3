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
  more** (ADR-0026 point 8 as amended; a fair variant is the battle lane's, later). **T1a, T1b
  done 2026-10-28** (C ABI v35: a world scores by its batch's reward v1, `h3_batch_true_world`).
  **Next:** T1c waits for a network (no `best.pt` in the sandbox: the user uploads `t7-1`'s or
  `order-1`'s); T2a for the user's choice of the neutral kinds' training mix (QUEUE). The
  adventure env's typed answers (E08 T63a) are in the inbox.
- **Waiting for a `shared/` change** (a second branch needs the user's consent): ADR-0025 point
  12a and `docs/07-simulation-and-ai.md` §1.1's "Information" to name v35 (a world's reward v1,
  the true world as trusted tooling, a diagnostic; E25 decision 6 approved it).
- **Open pull requests:** E25 T1a, T1b (`claude/adoring-turing-5sit47`): watch it to its merge.
  Merged: #528 (the E25 refinement).
- **Versions:** battle action encoding v13, observation encoding v22, replay format v31
  (CLT-Q4/Q6), the adventure env's action encoding v5 (E08 T63a), C ABI v35 (E25 T1a, T1b: a
  world's reward, the true world) (`docs/08-work-process.md`, "Versions"). Next ids: AI-Q11,
  AI-QUIRK-1, AI-STUB-2.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-10-28 — E25 T1a, T1b (C ABI v35)

- T1a: a world scores a decided battle by its batch's reward v1 (`BattleEnv::with_reward`, set
  by `determinize`; the agent's choice over a new function); found and fixed with it: a world's
  setup held the guess's starting mana, which reward v1's mana term reads. T1b:
  `BattleBatch::true_world`, `h3_batch_true_world`, `h3env.search --true-worlds`. Rust, C and
  Python tests; the scoped verify; the Python tests with torch.

### 2026-10-27 — E25's refinement (AI-Q10)

- E06's lane closure (T9c, T9d), left unopened on `claude/kind-bohr-wfa9oq`, merged as #524
  meanwhile. The user: target the faithful system actor; increment 1 the search's
  diagnostics and shaping; run lengths per run; asked what the system actor reads that a player
  may not (the hidden part listed in AI-Q10): it trains no agent any more, a fair variant goes to
  the battle lane's inbox, imitating that one is a candidate. Done: the epic file, increment 1's
  tasks, #525 (the shared part).

### 2026-10-27 — E06's closure: T9a the audit, T9b the review, T9c, T9d

- T9a: criteria 1 to 5 met, the inbox sorted. T9b: three reviewers, 14 findings, the fixable
  fixed with tests (#519); the user: T8c inconclusive (a terminal scored ±1), the world's gaps
  to AI-STUB-1, T9c on Windows only. T9c (the user): the verify with the original data and the
  Python tests green, `t7-1` played again (the targets hold). T9d: #522, the design note into
  the log, E06 closed.
