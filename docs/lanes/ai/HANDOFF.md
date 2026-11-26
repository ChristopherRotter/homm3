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
- **Open pull requests:** #1195 (`ai/fix-start-tests`: `master`'s build, T2d's start test
  without T56's `war_machine`; merge first), #1196 (`ai/e31-t2e`, `cross-lane`: T2e, on #1195).
  Merged: #1190 (T2d), #1184 (T2c), #1177 (T2b), #1172 (T2a), #1163 (the shared module PR).
  **Pending for the lane's next `shared/` pull request:** the `benches/README.md` row of T2a's
  workload; ADR-0030's note that the simulated combat (AI-Q34) and the spell values (AI-Q36) are
  `h3_battle::estimate`'s.
- **Versions:** battle action encoding v13, observation encoding v23 (E25 T38a: the reach
  block), replay format v32 (CLT-Q40), the adventure env's action encoding v13 (E09 T39a), C ABI
  v38 (E25 T38a; v37 `h3_batch_choice`); the computer player's `POLICY_VERSION` 1 and memory
  encoding v3 (`MEMORY_ENCODING_VERSION`, E31 T2e: the heroes' spell gains) (`docs/08-work-process.md`,
  "Versions"). Next ids: AI-Q37, AI-QUIRK-5 (AI-QUIRK-1 reserved for the slips), AI-STUB-16;
  CPU-SIM-013, CPU-STA-008, CPU-SPL-008, CPU-SKL-004.
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
  and Necromancy `computer/fight.rs` (CPU-SIM-011, -012; `expect(dead_code)` until T2h or T2i
  calls it); AI-STUB-8 (the aggressiveness 1.0), AI-STUB-9 (the conditions, E10); §5.5's #65 and
  #66. **T2d done** (2026-11-26, not cut, AI-Q35): the start's records `computer/start.rs`
  (CPU-STA-001 to -006) in the memory (v2): the threat fold with the capture estimate, the
  reserve, the weights (the library's `std::sort`, `order.rs`), the experience weight;
  `cross-lane` `reach_with`, `day_maximum_of`, `day_income`, `growth_inputs`; AI-STUB-10 to -15
  (the spell gains to T2e, the artifacts' mean to T2f, the vision circle to T2h, the Grail target
  to T2i); `computer-turns-towns-36` re-baselined (207 M Ir, the fold's searches). **T2e done**
  (2026-11-26, not cut, AI-Q36): the spell values `h3_battle::estimate`'s `caster.rs`
  (`cross-lane`, CPU-SPL-001 to -006; the battle's odds on them), a spell's gain and the heroes'
  spell gains (`computer/spells.rs`, CPU-SPL-007, CPU-STA-007; memory v3), the skill value's
  flag-set path in `h3_heroes::choice` (`cross-lane`, CPU-SKL-001), the schools' and the
  University's values (`computer/skills.rs`, CPU-SKL-002, -003); the workload re-baselined (229 M
  Ir). **Next: T2f**, the artifacts (`QUEUE.md`): propose a cut by size (the user asked for
  smaller sessions). Then E13 (refined in shape, `docs/epics/E13-adventure-ai.md`) once E10 is
  done whole.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in four groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **The computer player (E31)** (its coverage, its workload, the memory), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-11-26 — E31 T2e: the spells' and skills' values

- A cloud clone; the PC left before any original data was staged: the user attached
  `Heroes3.exe` to the chat (readings only; no `--with-original-data` run). Measured 5.0 KB: the
  user chose no cut, the spell values in `h3_battle::estimate` and the flag-set skill path in
  `h3_heroes::choice` (both `cross-lane`, AI-Q36). Found `master` broken since #1190 met #1189
  (a test literal without T56's field): #1195, which T2e's branch sits on. The models agreed
  with the port once `fight::battle_hero` carried the maximum mana. The workload +10.5 %
  (the heroes' caster records), re-baselined.

### 2026-11-26 — E31 T2d: the start's values

- A cloud clone; `Heroes3.exe`, the two bitmap LODs, `maps.tar` and `extracted.tar` staged from
  the user's repo folder at the start (the PC left mid-session). Measured 6.2 KB, about 2 300
  instructions: the user chose no cut, three values moved to later tasks, four small `cross-lane`
  helpers, the vision circle to T2h (AI-Q35). The setup's baseline stopped (`master` moved; CI
  green at abf981f stood for it); its instrumented binaries then spoiled a coverage report until
  `cargo llvm-cov clean --workspace`. Found a failing case of the adventure lane's reach property
  test on `master` (its inbox). Every expectation from the models agreed with the port at once.
  The scoped verify with the original data passed (but `check-lane`, by design); PR #1190.

### 2026-11-25 — E31 T2c: the casts, Necromancy and the fight estimate

- A cloud clone; `Heroes3.exe`, the two bitmap LODs, `maps.tar` and `extracted.tar` staged from
  the user's repo folder at the start (the PC was leaving). Measured 7.6 KB, 4.9 KB never read:
  the user chose no cut, then, the casts needing four of the battle's helpers, that the battle
  provide the estimate entirely (AI-Q34). The VM restarted once (the baseline lost; `master`'s CI
  green at 02ccd93 stood for it). Four of my test expectations were wrong, the port right.
  `h3-battle`, `h3-game`, `h3-sim` pass; the scoped verify with the original data.
