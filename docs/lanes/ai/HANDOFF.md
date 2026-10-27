# AI lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **Opened 2026-10-18** (AI-Q1): the lane owns `h3-sim` (its `src/adventure/` too since E08 T45, but `replay.rs`), `h3-ffi`, `h3-cli`'s `src/ai/`, `python/`, `benches/ai/`, E05 and E06 (`lanes.toml`).
- **E05 closed 2026-10-20**: its record and design in `E05-log.md` (cited as §n), the
  environment as built in `docs/07-simulation-and-ai.md` §1.1.
- **E06** (AI-Q8; `docs/epics/E06-battle-ai.md`, design `E06-design.md`, record `E06-log.md`,
  closed increments in `E06-archive.md`): increments 1 to 7 done; the last two levers (the turn
  order, `order-1`; the search, T8c) negative. At increment 7's checkpoint (2026-10-27) the user
  revised decision 7's targets to "beat every heuristic bot", which `t7-1` meets (0.828, 0.551,
  0.528 at positive values; the neutral set +0.245 above Greedy's +0.223), and chose the
  closure: `t7-1` is the battle AI v1, the system actor (0.374 against it) E25's target (Battle
  AI v2, a sketch; AI-STUB-1 moved there). **Next: increment 8, T9a the audit** (QUEUE; then
  T9b the review, T9c the verifies on the user's machine, T9d the closure).
- **Open pull requests:** `shared/e06-search-docs` (ADR-0025 point 12a, the env doc's v22 and
  C ABI v33/v34, BACKLOG's E06 row and E25; merges first) and the lane's
  (`claude/beautiful-wozniak-zjmkkw`: the checkpoint, increment 8 refined): watch both to
  their merge.
- **Versions:** battle action encoding v13, observation encoding v22, replay format v31
  (CLT-Q4/Q6), the adventure env's action encoding v4 (E08 T45), C ABI v34 (T6f and T8b: the turn
  order, the worlds) (`docs/08-work-process.md`, "Versions"). Next ids: AI-Q10, AI-QUIRK-1,
  AI-STUB-2.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

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

### 2026-10-26 to 27 — T6f, the turn order as an input; T8b, the search's plumbing

- The user (before leaving): the turn order first, as observation v22 with the engine's order
  (`h3_battle::turn_order`, `cross-lane`), then T8b, one pull request with one C ABI bump (v34),
  and a 20 M A/B against `t7-1` prepared (`configs/e06-order.toml`). Built both (E06-log T6f,
  T8b); `h3env.search` also has T8c's `--report`. One gap against ADR-0025 point 12a
  registered (AI-STUB-1); the battle lane's inbox names the two additions to `h3-battle`.
