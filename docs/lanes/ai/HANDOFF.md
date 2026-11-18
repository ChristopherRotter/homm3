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
  **Increment 8 closed 2026-11-18** (`E25-archive.md`; `E25-log.md` T30 to T36): no probe (size,
  PPO's settings, the search at play, the KL anchor, the outcome's weight) nor the long run
  (`e25-anchor-1` to 20 M: 0.511 ± 0.018 against the faithful actor, level with #2) passed #2,
  which stays the current best (AI-Q18). A `hexconv` run sets `accumulate = 4`, `chunk_align =
  32` (T31). **A long run stops early** (the user, 2026-11-18): `[gate] futility` (built at T36;
  `TRAPS.md`, Python). **Increment 9: a richer observation (v23)** (AI-Q18; the decisions the
  agent's proposal, the user reviews the design): **T37, v23's design, next** (the sandbox, no
  GPU; `QUEUE.md`).
- **Open pull requests:** the one of `claude/adoring-turing-tl1huu` (T36's close, `[gate]
  futility`, increment 9 refined) and its `shared/` companion (the README's E25 row). Merged:
  #954, #965 (T35's close, T36).
- **Versions:** battle action encoding v13, observation encoding v22, replay format v32
  (CLT-Q40), the adventure env's action encoding v6 (E08 T85b), C ABI v37 (E25 T16:
  `h3_batch_choice`; v36 the fair system actor in seat 4, the all-seeing one in seat 5)
  (`docs/08-work-process.md`, "Versions"). Next
  ids: AI-Q19, AI-QUIRK-1, AI-STUB-2.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-11-18 — E25 T36 and increment 8 closed, the early stop, increment 9 refined

- #965 merged. The user uploaded the 20 M run's `record.json` and `metrics.jsonl`, then its
  ladders (the first measure files and the head-to-head were of the 2 M network: identical to
  T34's; the 20 M head-to-head skipped by the user). No computer folder linked. Built `[gate]
  futility` (the user's directive); `uv sync --group train` and `cargo build --release -p
  h3-ffi` (1.5 minutes), the Python suite 270 (3.5 minutes). The user chose increment 9.

### 2026-11-16 — E25 T35 closed, T36 prepared

- #954 merged. The user uploaded `e25-win-bonus-1`'s `record.json`, `metrics.jsonl` and the three
  measures (the measures with CRLF, stripped; the update records read from `metrics.jsonl`, not
  committed). The probe within (0.5007 head to head): reward v1 stays. The user chose the long
  run of four offered: the anchor resumed to 20 M. Only `uv sync --group dev` and the records'
  test run (no code changed).

### 2026-11-15 — E25 T32 closed, T34 prepared

- #910 merged. The user uploaded T32's `report.json` and `metrics.jsonl` (no CRLF; the metrics
  not committed: the report holds every evaluation). No setup script: `uv sync --group train`
  and `cargo build --release -p h3-ffi` (53 s) in the background; the Python suite 264 (3.3
  minutes). The user chose T34's arms (the search at play, the KL anchor). #940 merged; #2's
  `best.pt` uploaded (in `python/runs/`, ignored): (a) run in the sandbox, N 16 72 minutes and
  the sampled policy 18, N 64 on the hero battles 27; #948 merged. (b)'s five files uploaded
  (three with CRLF, stripped); T34 closed. At T35 the user chose the win-bonus probe; built
  and its config tested.
