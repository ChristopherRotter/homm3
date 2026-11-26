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
- **Open pull requests:** #1203 (`ai/e31-t2f`, `cross-lane`: T2f). Merged: #1197 (T2e), #1192
  (`master`'s build after #1190), #1190 (T2d), #1184 (T2c), #1177 (T2b), #1172 (T2a), #1163 (the
  shared module PR); #1195 (the same fix) closed. **Pending for the lane's next `shared/` pull request:** the
  `benches/README.md` row of T2a's workload; ADR-0030's note that the simulated combat (AI-Q34)
  and the spell values (AI-Q36) are `h3_battle::estimate`'s.
- **Versions:** battle action encoding v13, observation encoding v23 (E25 T38a: the reach
  block), replay format v32 (CLT-Q40), the adventure env's action encoding v13 (E09 T39a), C ABI
  v38 (E25 T38a; v37 `h3_batch_choice`); the computer player's `POLICY_VERSION` 1 and memory
  encoding v4 (`MEMORY_ENCODING_VERSION`, E31 T2f: the artifacts' mean) (`docs/08-work-process.md`,
  "Versions"). Next ids: AI-Q38, AI-QUIRK-5 (AI-QUIRK-1 reserved for the slips), AI-STUB-18;
  CPU-SIM-013, CPU-STA-009, CPU-SPL-008, CPU-SKL-004, CPU-ART-012.
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
  **T2a to T2f done** (2026-11-25 and -26; one line each in the epic file's "Tasks done", their
  records in `E31-log.md`): the frame (`choose`, `Memory`, the turn's passes); the simulated
  combat, since T2c the battle lane's public `h3_battle::estimate` (`cross-lane`, with T2e's
  caster record and spell values and T2f's morale and luck change value); the fight estimate
  (`computer/fight.rs`, `expect(dead_code)` until T2h or T2i calls it); the start's records
  (`computer/start.rs`, memory v4); the spells' and skills' values (`spells.rs`, `skills.rs`);
  the artifacts' values and the start's mean (`artifacts.rs`, CPU-ART-001 to -011, CPU-STA-008;
  the equipped value 0x433C60 moved to increment 3, AI-Q37). `computer-turns-towns-36` at 309 M
  Ir (the threat fold's searches, the gains, the mean). **Next: T2g**, the army and town values (`QUEUE.md`):
  propose a cut by size (the user asked for smaller sessions). Then E13 (refined in shape, `docs/epics/E13-adventure-ai.md`) once E10 is
  done whole.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in four groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **The computer player (E31)** (its coverage, its workload, the memory), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-11-26 — E31 T2f: the artifacts

- A cloud clone; `Heroes3.exe`, the two bitmap LODs, `maps.tar` and `extracted.tar` staged from
  the user's repo folder at the start. Measured 7.0 KB, about 2 600 instructions: the user chose
  no cut, the morale value moved into `h3_battle::estimate`, the equipped value to increment 3
  (AI-Q37); the bans reader they approved was not needed (the mean reads the setup's flag, not
  the map's bans). The VM restarted during the setup's baseline; `master` had moved to dc70cf8
  (T57 merged), whose CI stood for it. The test core refuses foreign keys: the content is read by
  the original packs' handles (`TRAPS.md`). First build +64 % on the workload; two wastes removed
  (+34.8 %, re-baselined). Four of my test expectations were wrong, the port right.

### 2026-11-26 — E31 T2e: the spells' and skills' values

- A cloud clone; the PC left before any original data was staged: the user attached
  `Heroes3.exe` to the chat (readings only; no `--with-original-data` run). Measured 5.0 KB: the
  user chose no cut, the spell values in `h3_battle::estimate` and the flag-set skill path in
  `h3_heroes::choice` (both `cross-lane`, AI-Q36). Found `master` broken since #1190 met #1189
  (a test literal without T56's field): #1195, superseded by #1192. Since T56 the Marketplace is
  read by its role, not its key (the town lane's request dropped). The models agreed
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
