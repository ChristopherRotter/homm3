# AI lane — HANDOFF

Read after `docs/HANDOFF.md`. Budgets (`check-docs`: 10 000 bytes, three sessions; aim for 5 000): `docs/08-work-process.md`, "Keeping the start-of-session reading small". Tasks: `QUEUE.md`. Traps: `TRAPS.md` (read the group for the area you touch).

## Current state

- **Opened 2026-10-18** (AI-Q1): the lane owns `h3-sim` (its `src/adventure/` too since E08 T45, but `replay.rs`), `h3-ffi`, `h3-cli`'s `src/ai/`, `python/`, `benches/ai/`, E05, E06 and E25; since AI-Q31 `h3-game`'s `src/computer/` and E31 (`lanes.toml`).
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
  **Increments 8 to 10 closed** (`E25-archive.md`; `E25-log.md` T30 to T44): no probe, the long
  run nor v23 passed #2 (AI-Q18, AI-Q20); increment 10, the ceiling (`h3 ai ceiling`): against
  the fair actor +0.085 legitimately (+0.115 seeing all), #2 +0.026 of it, its regret spread over
  every kind of decision (casts the costliest a decision, missed moves the largest sum). A
  `hexconv` run sets `accumulate = 4`, `chunk_align = 32` (T31); a long run stops early (`[gate]
  futility`, T36); a bench A/B sets `same_start` (T38b; `TRAPS.md`, Python). **Paused after
  increment 10** (the user at T45, AI-Q23): increment 11 is expert iteration when E25 resumes.
- **Open pull requests:** the one of `ai/e31-t2b` (T2b, the simulated combat's sides and
  melee). Merged: #1172 (T2a), #1163 (the shared module PR), #1164 (increment 2's cut).
- **Versions:** battle action encoding v13, observation encoding v23 (E25 T38a: the reach
  block), replay format v32 (CLT-Q40), the adventure env's action encoding v13 (E09 T39a), C ABI
  v38 (E25 T38a; v37 `h3_batch_choice`); the computer player's `POLICY_VERSION` 1 and memory
  encoding v1 (`MEMORY_ENCODING_VERSION`, E31 T2a) (`docs/08-work-process.md`, "Versions"). Next
  ids: AI-Q34, AI-QUIRK-5 (AI-QUIRK-1 reserved for the slips), AI-STUB-8.
- **E31 in progress** (the original's computer player; refined 2026-11-21, AI-Q22:
  `docs/epics/E31-original-computer-player.md`, record `E31-log.md`, closed increments
  `E31-archive.md`, the survey `docs/rules/surveys/E31-computer-player.md`, the design note
  `E31-design.md`). **Increment 1 closed 2026-11-24** (AI-Q31): the reading (255 functions,
  T1a1a to T1a4) and the survey's close §5 (T1b: the rules by phase, the hidden reads, nine rule
  rows in the battle, adventure and town lanes' inboxes, the gaps by epic, the 61 quirk candidates
  classified). **ADR-0030** (accepted 2026-11-25, AI-Q32): a module `h3_game::computer` of
  `h3-game` for now, refactored into a crate later; the memory the policy's; every battle fought
  until the battle lane decides; `SignedX87`; AI-QUIRK-1 for the slips. **Increment 2 refined**
  (AI-Q32): eleven tasks T2a to T2k (the epic file), about 26 000 of the original's instructions.
  **T2a done** (2026-11-25): `h3_game::computer::choose`, `Memory`, the turn's frame with each
  hero done at once (AI-STUB-2 to -6), `h3_sim::computer::play` (the property tests, every
  shipped map at every difficulty in the original-data check), `h3 ai computer` and the workload
  `computer-turns-towns-36`. **T2b done** (2026-11-25, not cut, AI-Q33):
  the simulated combat's sides and melee in `computer/sim.rs` (CPU-SIM-001 to -005; the casts
  AI-STUB-7; no caller until T2c, `expect(dead_code)` on the module); §5.5's #63 and #64.
  **Next: T2c**, the casts, Necromancy and the fight estimate (`QUEUE.md`), `Heroes3.exe`
  staged for the blind re-reads. Then E13 (refined in shape,
  `docs/epics/E13-adventure-ai.md`) once E10 is done whole.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in four groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **The computer player (E31)** (its coverage, its workload, the memory), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-11-25 — E31 T2b: the simulated combat's sides and melee

- A cloud clone; `Heroes3.exe`, the two bitmap LODs, `maps.tar` and `extracted.tar` staged from
  the user's repo folder at the start. Measured 7.4 KB (1.4 KB container code): the user chose no
  cut (AI-Q33). The blind re-reads agreed with the survey but four details (`E31-log.md` T2b). The VM
  restarted once (the setup's baseline lost); `master`'s CI had failed on a server-lane timing
  test only (`h3-server`'s `a_spectator_watches_a_timed_game…`), its failed job re-run. The
  scoped verify with the original data; an IEEE double model checked the port on 400 runs.

### 2026-11-25 — E31 T2a: the frame

- A cloud clone; `Heroes3.exe`, the two bitmap LODs, `maps.tar` and `extracted.tar` staged from
  the user's repo folder before the bridge dropped (enough for the task and its verify). Measured
  3.8 KB, about 1 400 instructions: done alone. The blind re-reads of the three frame functions
  agreed with the survey. CI green on `master` stood for the setup's baseline. The scoped verify
  with the original data; the shared `benches/README.md` row for the new workload waits for the
  lane's next `shared/` pull request.

### 2026-11-25 — E31 increment 2's start: the shared pull request, the measurement, the cut

- A cloud clone; the user's computer reachable but not needed (no reading this session). The
  `shared/` pull request #1163 built and checked (the kernel's and xtask's tests, clippy, the
  workspace's check, the docs and rules gates; `arith.rs` 100 % by llvm-cov); master's ADR-0029
  (#1159) merged into it. Increment 2 measured from the address map's sizes; the user asked once
  (AI-Q32): ADR-0030 accepted, eleven tasks, the `lib.rs` line in the shared pull request, the
  loader's crashes the adventure lane's.
