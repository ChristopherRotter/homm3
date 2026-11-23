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
  **Increments 8 to 10 closed** (`E25-archive.md`; `E25-log.md` T30 to T44): no probe, the long
  run nor v23 passed #2 (AI-Q18, AI-Q20); increment 10, the ceiling (`h3 ai ceiling`): against
  the fair actor +0.085 legitimately (+0.115 seeing all), #2 +0.026 of it, its regret spread over
  every kind of decision (casts the costliest a decision, missed moves the largest sum). A
  `hexconv` run sets `accumulate = 4`, `chunk_align = 32` (T31); a long run stops early (`[gate]
  futility`, T36); a bench A/B sets `same_start` (T38b; `TRAPS.md`, Python). **Paused after
  increment 10** (the user at T45, AI-Q23): increment 11 is expert iteration when E25 resumes.
- **Open pull requests:** #1139 (`ai/e31-t1a3`: E31 T1a3a and T1a3b, docs and the map). Merged: #1137 (T1a2), #1133 (T1a1b, T1a1c), #1131 (T1a1a).
- **Versions:** battle action encoding v13, observation encoding v23 (E25 T38a: the reach
  block), replay format v32 (CLT-Q40), the adventure env's action encoding v13 (E09 T39a), C ABI
  v38 (E25 T38a; v37 `h3_batch_choice`) (`docs/08-work-process.md`, "Versions"). Next
  ids: AI-Q27, AI-QUIRK-1, AI-STUB-2.
- **E31 in progress** (the original's computer player; refined 2026-11-21, AI-Q22:
  `docs/epics/E31-original-computer-player.md`, record `E31-log.md`, the survey
  `docs/rules/surveys/E31-computer-player.md`): T1a1 cut in three (AI-Q24); **T1a1a done** (the
  turn's loop, the records), **T1a1b** (the planning; two quirk candidates for T1b) and **T1a1c**
  (the walk) done: **T1a1 done**; **T1a2** (the values) done in four parts, three by sub-agents
  (AI-Q25; the survey §2); **T1a3** cut in four (AI-Q26): **T1a3a** (the build choice) and
  **T1a3b** (the hires) done (the survey §3.1, §3.2). **Next: E31 T1a3c** (the town visit), then
  T1a3d (the answers and the table), in `QUEUE.md`, `Heroes3.exe` staged in a cloud sandbox. Then E13 (refined in shape,
  `docs/epics/E13-adventure-ai.md`) once E10 is done whole.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-11-23 — E31 T1a3a and T1a3b: the build choice and the hires

- A cloud clone, `Heroes3.exe` staged from the user's repo folder. T1a3 measured about 6 000
  instructions at its start, so the cut asked (AI-Q26): four parts, this session T1a3a and T1a3b,
  read by the main session; the edits in a `git worktree` while the setup's baseline ran in the
  main clone. The pull request: #1139.

### 2026-11-23 — E31 T1a2: the values, in four parts

- A cloud clone (the PC's shell worked, but readings stay in the sandbox: `docs/exe/README.md`),
  `Heroes3.exe` staged from the user's repo folder. The cut asked at the start (AI-Q25): four
  parts; the main session read T1a2a, three general-purpose sub-agents T1a2b to T1a2d at once,
  each in a `git worktree` of its own editing only `map.toml` and `structs.toml` and writing its
  survey text to a scratch file; the main session merged the lines by address (six overlaps
  reconciled), renamed for consistency and wrote §2. The scoped verify without original data.

### 2026-11-23 — E31 T1a1b and T1a1c: the planning and the walk

- A cloud clone, `Heroes3.exe` staged from the user's PC (its local `master` was behind GitHub's,
  which already had #1131). T1a1b: the danger map, the destination's choice, the candidates, the
  claims, the score and its draw (21 map entries, `pathfinder_node`, `ai_candidate`, the survey's
  §1.9). T1a1c on the same branch: the walk (10 entries, §1.10; 0x40EC90 renamed `hero_dig`, the
  map-client lane's inbox told). Scratch scripts over `scripts/exe.py` (not committed). The scoped
  verify with `--with-original-data` fails the harvester step without the LODs (only the
  executable was staged): run it without the flag.
