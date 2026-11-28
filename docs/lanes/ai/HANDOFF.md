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
- **Open pull requests:** #1226 (`ai/e31-t2i`, T2i, `cross-lane`). Merged: #1222 (T2i0), #1212 (T2h), #1208 (T2g), #1203 (T2f),
  #1197 (T2e), #1192 (`master`'s build after #1190), #1190 (T2d), #1184 (T2c), #1177 (T2b),
  #1172 (T2a), #1163 (the shared module PR); #1195 (the same fix) closed. **Pending for the lane's next `shared/` pull request:** the
  `benches/README.md` row of T2a's workload; ADR-0030's note that the simulated combat (AI-Q34)
  and the spell values (AI-Q36) are `h3_battle::estimate`'s, and point 4's that the computer's
  query is the ai lane's `h3_adventure::search` (AI-Q41).
- **Versions:** battle action encoding v13, observation encoding v23 (E25 T38a: the reach
  block), replay format v32 (CLT-Q40), the adventure env's action encoding v24 (E09 T59a), C ABI
  v38 (E25 T38a; v37 `h3_batch_choice`); the computer player's `POLICY_VERSION` 1 and memory
  encoding v5 (`MEMORY_ENCODING_VERSION`, E31 T2i: the heroes' kept plans) (`docs/08-work-process.md`,
  "Versions"). Next ids: AI-Q43, AI-QUIRK-5 (AI-QUIRK-1 reserved for the slips), AI-STUB-24;
  CPU-SIM-013, CPU-STA-009, CPU-SPL-008, CPU-SKL-004, CPU-ART-012, CPU-ARM-006, CPU-TWN-006,
  CPU-OBJ-013, CPU-PTH-008, CPU-PLN-009; the survey's §5.5 #80.
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
  **T2a to T2h done** (2026-11-25 to -27; one line each in the epic file's "Tasks done", their
  records in `E31-log.md`): the frame (`choose`, `Memory`, the turn's passes); the simulated
  combat, since T2c the battle lane's public `h3_battle::estimate` (`cross-lane`, with T2e's
  caster record and spell values and T2f's morale and luck change value); the fight estimate
  (`computer/fight.rs`, `expect(dead_code)` until T2h or T2i calls it); the start's records
  (`computer/start.rs`, memory v4); the spells' and skills' values (`spells.rs`, `skills.rs`);
  the artifacts' values and the start's mean (`artifacts.rs`, CPU-ART-001 to -011, CPU-STA-008;
  the equipped value 0x433C60 moved to increment 3, AI-Q37); the army plan's values and the
  dwelling and town values (`armies.rs`, `towns.rs`, CPU-ARM-001 to -005, CPU-TWN-001 to -005,
  AI-QUIRK-2 fixed; not cut, AI-Q38); the object value and the vision circle (`objects.rs` with
  `fights.rs` and `visits.rs`, `vision.rs`, CPU-OBJ-001 to -012; not cut, AI-Q39;
  `AdventureState::tile_index` public, AI-Q40), all `expect(dead_code)` until T2i calls them;
  **T2i0** inserted and done (AI-Q41): the original's computer pathfinder in `h3-adventure`
  (`h3_adventure::search`, `cross-lane`, CPU-PTH-001 to -007; survey §1.11), the threat fold on it
  (AI-STUB-11 lifted). **T2i** done whole (AI-Q41): the planning (`computer/plan.rs` and
  `plan/`, CPU-PLN-001 to -008; the memory's kept plans, v5; `SearchStart::open`, `cross-lane`,
  AI-Q42; every `expect(dead_code)` lifted); each hero still done after planning.
  `computer-turns-towns-36` re-baselined at T2i0 and T2i (the searches; about 25.8 M Ir a turn).
  **Next: T2j**, the walk (`QUEUE.md`).
  Then E13 (refined in shape, `docs/epics/E13-adventure-ai.md`) once E10 is done whole.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in four groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **The computer player (E31)** (its coverage, its workload, the memory), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-11-28 — E31 T2i: the planning

- A cloud clone; `Heroes3.exe`, the two bitmap LODs, `maps.tar` and `extracted.tar` staged from
  the user's repo folder at the start (the PC behind GitHub, nothing uncommitted). The VM restarted
  during the setup's baseline; CI's green `master` stood for it. The main session read the value
  field, the choice and the search loop, a fresh reader blind-confirmed the two slips, one worker
  sub-agent built on a fixed design. The user approved `SearchStart::open` (AI-Q42).
  `computer-turns-towns-36` +368 % (re-baselined: the planning's searches).

### 2026-11-27 — E31 T2i0: the computer's search

- A cloud clone; `Heroes3.exe`, the two bitmap LODs, `maps.tar` and `extracted.tar` staged from
  the user's repo folder at the start (the PC left mid-session). T2i measured 2 860 instructions,
  but its searches are the original's computer pathfinder: the user chose it first and whole, in
  `h3-adventure`, the fog by the variant, T2i then whole (AI-Q41). Three fresh readers, a blind
  re-read of the five slips, two worker sub-agents in turn on a fixed design, one for the speed.
  The human reach's workloads flat; `computer-turns-towns-36` +300.6 % (re-baselined: the threat
  fold's computer search, halved by a tuning pass with every search's result unchanged).

### 2026-11-27 — E31 T2h: the object value

- A cloud clone; `Heroes3.exe`, the two bitmap LODs, `maps.tar` and `extracted.tar` staged from
  the user's repo folder at the start (the PC's shell left after it). Measured 12.8 KB, about
  4 300 instructions: the user kept it whole (AI-Q39) and made `tile_index` public (AI-Q40). The
  engine knows an object by its kind, not the original's class: `Class::of` maps one to the other. Seven of my hand
  expectations were wrong, the port right (fixture traps in `TRAPS.md`). The workload +0.00 %; the
  scoped verify with the original data passed (coverage 29 minutes, bench-check 24).
