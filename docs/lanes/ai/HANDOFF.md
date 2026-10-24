# AI lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **Opened 2026-10-18** (AI-Q1): the lane owns `h3-sim` (its `src/adventure/` too since E08 T45, but `replay.rs`), `h3-ffi`, `h3-cli`'s `src/ai/`, `python/`, `benches/ai/`, E05 and E06 (`lanes.toml`).
- **E05 closed 2026-10-20**: its record and design in `E05-log.md` (cited as §n), the
  environment as built in `docs/07-simulation-and-ai.md` §1.1.
- **E06** (AI-Q8; `docs/epics/E06-battle-ai.md`, design `E06-design.md`, record `E06-log.md`):
  increments 1 to 5 done (#306 to #432); T6a (the behaviour analysis) done 2026-10-23. Best so
  far `t7-1` (20 M, a third neutral): 0.828, 0.551, 0.528 against Random, Aggressor, Greedy, the
  system actor 0.374; the neutral set's value +0.245 below the target +0.285. **Next:**
  increment 6's lever (QUEUE: the user's choice).
- **Versions:** battle action encoding v13, observation encoding v21, replay format v31 (CLT-Q4/Q6), the adventure env's action encoding v4 (E08 T45), C ABI v33 (E06 T5: the kill preview) (`docs/08-work-process.md`, "Versions"). Next ids: AI-Q10, AI-QUIRK-1, AI-STUB-1.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-10-23 — E06 T6a, the behaviour analysis

- Built `python -m h3env.train.inspect`, ran it on the user's `t7-1` best (records
  `inspect/t7-1-*.json`; E06-log T6a). Found the turret shot's preview (battle lane's inbox).
  Increment 6's lever put to the user (QUEUE). The user: the
  System-seat benchmark (ADR-0027 point 11) into the same pull request.

### 2026-10-23 — T7d's mix; T5's rest beside it

- The user: T7d at a third neutral (their machine); here the mana price's preparation (#429),
  then AI-Q7 (b) part 1 (#432): the preview of every entry halves the engine's step (29 µs),
  still far above training's 5 000 samples/s. `t7-1` recorded; the user: diagnose first (T6a),
  in a fresh session.
