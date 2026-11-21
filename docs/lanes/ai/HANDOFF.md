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
  the target is beating the faithful system actor (`t7-1`: 0.372), which **trains no agent**
  (ADR-0026 point 8 as amended). Since increment 5 (AI-Q14) `system` is the battle lane's fair
  actor everywhere, `system-all-seeing` the target (C ABI v36), the neutral kinds train with the
  fair defender, and `h3_batch_choice` (v37) clones it.
  **Done:** increments 1 to 7 (the search's diagnostics and shaping; the tactics bound and
  scale; the network's architecture; the policy's sharpness; the fair system actor; better
  imitation; the structured network), in `E25-archive.md`. Increment 7's read: `hexconv` 96 × 6
  with the legal mask clones the fair actor 0.809 (the MLP 0.574, T27), and PPO from that clone
  with the imitation loss carries the lead through RL (T28).
  **`e25-structure-1` #2 the current best** (T29, AI-Q17: **0.511 ± 0.019, +0.027 against the
  fair actor, 0.507 ± 0.019, +0.026 against the all-seeing one, the neutral set +0.302**, target
  +0.285; 0.552 head to head against `e25-imitate-1` #2): exit criterion 1 met on its point
  estimates, the faithful actor's win rate inside its interval; 217 samples a second, 28 hours a
  20 M on the GTX 1070. The GPU's memory capped at 90 % of the card (`gpu.cap`, `TRAPS.md`).
  **Increments 8 and 9 closed** (`E25-archive.md`; `E25-log.md` T30 to T39): no probe (size,
  PPO's settings, the search at play, the KL anchor, the outcome's weight), the long run (0.511
  against the faithful actor) nor v23 (the reach block, the kill preview: T39's bench, 0.457,
  0.460, 0.459 ± 0.018) passed #2 (AI-Q18, AI-Q20). A `hexconv` run sets `accumulate = 4`,
  `chunk_align = 32` (T31); a long run stops early (`[gate] futility`, T36); a bench A/B sets
  `same_start` (T38b; `TRAPS.md`, Python). **Increment 10: the ceiling** (the user, AI-Q20): a
  lower bound on the play possible against the fair actor by rollouts of the fair actor (every
  legal command, common random numbers), on the true battle and the side's worlds, then where
  #2's choices lose value. **T42 done** (`E25-log.md` T42): `h3 ai ceiling`, the rollout-improved
  fair actor on `h3 ai eval`'s sets, sequential halving, a resumable games file; about 190
  thread-seconds a game at budget 8. **T43's budget check done** (`E25-log.md` T43): budget 4,
  the rollout-improved fair actor +0.083 over the fair actor's own play (paired; 0.543), three
  times #2's +0.027; 8 and 16 within noise of 4. **Next: T43's measurement** at budget 4 on the
  user's CPU (the commands in `QUEUE.md`; the games files uploaded too), T44 (where #2 diverges)
  beside it.
- **Open pull requests:** #1117 (`ai/e31-refinement`, E31's refinement) after #1116 (`shared/e31-refinement`, its backlog and README rows; merge first). Merged: #1110 (E13's refinement). Merged: #1084
  (the stop fix), #1083 (`shared/e25-ceiling`), #1075 (T39, T41, T42).
- **Versions:** battle action encoding v13, observation encoding v23 (E25 T38a: the reach
  block), replay format v32 (CLT-Q40), the adventure env's action encoding v13 (E09 T39a), C ABI
  v38 (E25 T38a; v37 `h3_batch_choice`) (`docs/08-work-process.md`, "Versions"). Next
  ids: AI-Q23, AI-QUIRK-1, AI-STUB-2.
- **After E25 increment 10** (T43 to T45): **E31** (the original's computer player; refined
  2026-11-21, AI-Q22: `docs/epics/E31-original-computer-player.md`, increment 1 the reading's
  map and the survey; next E31 T1a1 in `QUEUE.md`), then E13 (refined in shape,
  `docs/epics/E13-adventure-ai.md`) once E10 is done whole.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-11-21 — E31 refined (AI-Q22)

- A refinement session with the user, docs only (the PC's shell down, a cloud clone;
  `Heroes3.exe` staged alone, SHA-256 checked, for the first look's size by a scratch script
  over `scripts/exe.py`'s `Code`, not committed). E31's epic file, AI-Q22, the queue's E31 T1a1;
  the backlog's and README's rows in `shared/e31-refinement`.

### 2026-11-21 — E13 refined in shape (AI-Q21), E31 sketched

- A refinement session with the user, docs only (the PC's shell down, a cloud clone). E13 and
  E31 move to this lane (`shared/e13-refinement`, `lanes.toml`, `BACKLOG.md`); E13's epic file,
  AI-Q21, the adventure lane's inbox told. No original data (none needed).

### 2026-11-21 — E25 T43: the budget check read, budget 4

- #1083, #1084 merged. The user ran the budget check (16 threads) and uploaded the three games
  files; `cargo build --release -p h3-cli`, the JSON records rebuilt from them (identity
  matched) and `--budget 0` played as the pairing's base (`python/records/ceiling/`). No
  original data (none needed). The measurement's commands at budget 4 in `QUEUE.md`.
