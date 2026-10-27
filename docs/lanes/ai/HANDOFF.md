# AI lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **Opened 2026-10-18** (AI-Q1): the lane owns `h3-sim` (its `src/adventure/` too since E08 T45, but `replay.rs`), `h3-ffi`, `h3-cli`'s `src/ai/`, `python/`, `benches/ai/`, E05 and E06 (`lanes.toml`).
- **E05 closed 2026-10-20**: its record and design in `E05-log.md` (cited as §n), the
  environment as built in `docs/07-simulation-and-ai.md` §1.1.
- **E06** (AI-Q8; `docs/epics/E06-battle-ai.md`, design `E06-design.md`, record `E06-log.md`,
  closed increments in `E06-archive.md`): increments 1 to 7 done; `t7-1` is the battle AI v1
  (decision 7 as revised 2026-10-27, met), the system actor E25's target. Increment 8 (T9, the
  closure): T9a the audit and T9b the review done 2026-10-27 (T8c's search verdict inconclusive,
  measured again in E25; the world's gaps in AI-STUB-1). **Next: T9c on the user's machine**
  (Windows only, the user: exit criterion 6 amended; the script in QUEUE's T9c, after this
  lane's pull request merged), then T9d, the closure (`shared/e06-closure` first).
- **Open pull requests:** the lane's (`claude/kind-bohr-wfa9oq`: T9a, T9b): watch it to its
  merge.
- **Versions:** battle action encoding v13, observation encoding v22, replay format v31
  (CLT-Q4/Q6), the adventure env's action encoding v4 (E08 T45), C ABI v34 (T6f and T8b: the turn
  order, the worlds) (`docs/08-work-process.md`, "Versions"). Next ids: AI-Q10, AI-QUIRK-1,
  AI-STUB-2.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-10-27 — T9a the audit, T9b the independent review

- T9a: exit criteria 1 to 5 met (5's neutral comparison spans `POLICY_VERSION` 18 and 19: T9c
  plays `t7-1` again); the inbox sorted by destination. T9b: three reviewers, 14 findings, the
  fixable fixed with tests (trainer files on a refused replace, NaN minibatches, the search's
  escapes, a world of the side not to move, the system actor's retreat, the encounter data
  pinned); the bots' tables regenerated (adventure hash). The user: T8c inconclusive (a terminal
  scored ±1), E25 measures again; the world's gaps to AI-STUB-1; T9c on Windows only.

### 2026-10-27 — Increment 7's checkpoint: E06 to its closure, the targets revised

- The user: close E06 with `t7-1` as the battle AI v1, decision 7 revised to "beat every
  heuristic bot", a stronger agent as E25; both docs parts now. Done: the epic file, the archive,
  the log, the design's §14, AI-STUB-1 to E25, the draft retired (this lane's PR); ADR-0025
  point 12a, the env doc, BACKLOG (`shared/e06-search-docs`, a second branch the user allowed).

### 2026-10-27 — T6g, the run `order-1` recorded; T8c, the search measured

- The user uploaded `order-1` (T6g): `t7-1`'s ladder and play; the probe again: the trunk drops
  the order it reads (the block checked, no fault). T8c in the sandbox with `order-1`'s best:
  the search played as the sampled policy (σ's fixed scale; fixed: rescaled per node, as
  `mctx`), then no gain at N 16 and 64 against the system actor nor at N 64 on the neutral set.
  Put to the user: the checkpoint (E06's end, T8d, a diagnostic) and AI-STUB-1.
