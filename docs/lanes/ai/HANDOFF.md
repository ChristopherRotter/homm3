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
- **Open pull requests:** T2c's (`ai/e31-t2c`, `cross-lane`: the number in the session entry
  below). Merged: #1177 (T2b), #1172 (T2a), #1163 (the shared module PR), #1164 (increment 2's
  cut). **Pending for the lane's next `shared/` pull request:** the `benches/README.md` row of
  T2a's workload; ADR-0030's note that the simulated combat is `h3_battle::estimate` (AI-Q34).
- **Versions:** battle action encoding v13, observation encoding v23 (E25 T38a: the reach
  block), replay format v32 (CLT-Q40), the adventure env's action encoding v13 (E09 T39a), C ABI
  v38 (E25 T38a; v37 `h3_batch_choice`); the computer player's `POLICY_VERSION` 1 and memory
  encoding v1 (`MEMORY_ENCODING_VERSION`, E31 T2a) (`docs/08-work-process.md`, "Versions"). Next
  ids: AI-Q35, AI-QUIRK-5 (AI-QUIRK-1 reserved for the slips), AI-STUB-10; CPU-SIM-013.
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
  the simulated combat's sides and melee (CPU-SIM-001 to -005); §5.5's #63 and #64. **T2c done**
  (2026-11-25, not cut, AI-Q34): the simulated combat moved to the battle lane's public
  `h3_battle::estimate` (`cross-lane`) with its casts (CPU-SIM-006 to -010); the fight estimate
  and Necromancy `computer/fight.rs` (CPU-SIM-011, -012; `expect(dead_code)` until T2d, T2h or
  T2i calls it); AI-STUB-8 (the aggressiveness 1.0), AI-STUB-9 (the conditions, E10); §5.5's #65
  and #66. **Next: T2d**, the start's values (`QUEUE.md`), `Heroes3.exe` staged for the
  readings. Then E13 (refined in shape,
  `docs/epics/E13-adventure-ai.md`) once E10 is done whole.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in four groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **The computer player (E31)** (its coverage, its workload, the memory), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-11-25 — E31 T2c: the casts, Necromancy and the fight estimate

- A cloud clone; `Heroes3.exe`, the two bitmap LODs, `maps.tar` and `extracted.tar` staged from
  the user's repo folder at the start (the PC was leaving). Measured 7.6 KB, 4.9 KB never read:
  the user chose no cut, then, the casts needing four of the battle's helpers, that the battle
  provide the estimate entirely (AI-Q34). The VM restarted once (the baseline lost; `master`'s CI
  green at 02ccd93 stood for it). Four of my test expectations were wrong, the port right.
  `h3-battle`, `h3-game`, `h3-sim` pass; the scoped verify with the original data.

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
