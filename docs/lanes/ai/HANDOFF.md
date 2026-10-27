# AI lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **Opened 2026-10-18** (AI-Q1): the lane owns `h3-sim` (its `src/adventure/` too since E08 T45, but `replay.rs`), `h3-ffi`, `h3-cli`'s `src/ai/`, `python/`, `benches/ai/`, E05, E06 and E25 (`lanes.toml`).
- **E05 closed 2026-10-20**: its record and design in `E05-log.md` (cited as §n), the
  environment as built in `docs/07-simulation-and-ai.md` §1.1.
- **E06 closed 2026-10-27** (AI-Q8; `docs/epics/E06-battle-ai.md`, record and design note
  `E06-log.md`, closed increments `E06-archive.md`; the pipeline as built:
  `docs/07-simulation-and-ai.md` §1.2): `t7-1` is the battle AI v1 and the baseline.
- **E25 refined 2026-10-27** (AI-Q10; `docs/epics/E25-battle-ai-v2.md`): the target is beating
  the faithful system actor (`t7-1`: 0.372); it **trains no agent any more** (ADR-0026 point 8
  as amended; a fair variant is the battle lane's, later). **Next: increment 1's T1a** (QUEUE:
  T1a to T1c the search's terminal, the true world, T8c again; T2a asks the user the neutral
  kinds' training mix first). The adventure env's typed answers wait for E08 T63.
- **Open pull requests:** #525 (`shared/e25-refinement`: `lanes.toml`, BACKLOG, ADR-0026 and
  0027; a second branch the user allowed), merged first; then this lane's
  (`claude/cool-sagan-qdfznc`: E06 T9c and T9d, which `claude/kind-bohr-wfa9oq` left unopened,
  and E25's refinement): watch both to their merge.
- **Versions:** battle action encoding v13, observation encoding v22, replay format v31
  (CLT-Q4/Q6), the adventure env's action encoding v4 (E08 T45), C ABI v34 (T6f and T8b: the turn
  order, the worlds) (`docs/08-work-process.md`, "Versions"). Next ids: AI-Q11, AI-QUIRK-1,
  AI-STUB-2.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-10-27 — E25's refinement (AI-Q10)

- Found E06's lane closure (T9c, T9d) finished on `claude/kind-bohr-wfa9oq` but never opened:
  carried on this branch. The user: target the faithful system actor; increment 1 the search's
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

### 2026-10-27 — Increment 7's checkpoint: E06 to its closure, the targets revised

- The user: close E06 with `t7-1` as the battle AI v1, decision 7 revised to "beat every
  heuristic bot", a stronger agent as E25; both docs parts now. Done: the epic file, the archive,
  the log, the design's §14, AI-STUB-1 to E25, the draft retired (this lane's PR); ADR-0025
  point 12a, the env doc, BACKLOG (`shared/e06-search-docs`, a second branch the user allowed).
