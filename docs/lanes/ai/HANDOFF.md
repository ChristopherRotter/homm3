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
- **Open pull requests:** #1158 (`ai/e31-t1b`: E31 T1b, docs only; the README row #1157 `shared/e31-increment-1` merges first). Merged: #1150 (T1a4), #1145 (T1a3d), #1142 (T1a3c), #1139 (T1a3a, T1a3b), #1137 (T1a2), #1133 (T1a1b, T1a1c), #1131 (T1a1a).
- **Versions:** battle action encoding v13, observation encoding v23 (E25 T38a: the reach
  block), replay format v32 (CLT-Q40), the adventure env's action encoding v13 (E09 T39a), C ABI
  v38 (E25 T38a; v37 `h3_batch_choice`) (`docs/08-work-process.md`, "Versions"). Next
  ids: AI-Q32, AI-QUIRK-1, AI-STUB-2.
- **E31 in progress** (the original's computer player; refined 2026-11-21, AI-Q22:
  `docs/epics/E31-original-computer-player.md`, record `E31-log.md`, closed increments
  `E31-archive.md`, the survey `docs/rules/surveys/E31-computer-player.md`, the design note
  `E31-design.md`). **Increment 1 closed 2026-11-24** (AI-Q31): the reading (255 functions,
  T1a1a to T1a4) and the survey's close §5 (T1b: the rules by phase, the hidden reads, nine rule
  rows in the battle, adventure and town lanes' inboxes, the gaps by epic, the 61 quirk candidates
  classified). **The ADR draft** `docs/lanes/ai/adr-drafts/original-computer-player.md` (the user
  at AI-Q31: a module `h3_game::computer` of `h3-game` for now, refactored into a crate later; the
  memory the policy's; every battle fought until the battle lane decides; AI-QUIRK-1 for the
  slips), awaiting its acceptance and number. **Next: E31 increment 2** (refine and cut; first the
  `shared/` pull request of the draft's points 1 and 9), in `QUEUE.md`, `Heroes3.exe` staged for
  the blind re-reads. Then E13 (refined in shape, `docs/epics/E13-adventure-ai.md`) once E10 is
  done whole.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-11-24 — E31 T1b: the survey's close, the ADR draft, the checkpoint

- A cloud clone; the user's computer unreachable for `Heroes3.exe` (not needed: the blind re-reads
  stay with each slip's port task). The cut asked (AI-Q30): none, sub-agents for the narrow
  tables (the quirk candidates, the hidden reads and gaps, the wiki's and VCMI's positions), each
  checked by the main session. The setup's baseline stopped (CI green on `master`); the scoped
  verify without original data. The checkpoint in two `AskUserQuestion` rounds (AI-Q31); the
  README's row in `shared/e31-increment-1`.

### 2026-11-24 — E31 T1a4: the rest of the code, in three

- A cloud clone, `Heroes3.exe` staged from the user's repo folder. T1a4 measured about 3 500
  instructions plus the lists, so the cut asked (AI-Q29): three parts; two general-purpose
  sub-agents read T1a4a (the artifact effects) and T1a4b (the initialisers, the leftovers) one
  after the other, each in its own worktree (only `map.toml` and `structs.toml`, its survey text to
  a scratch file); the main session made the lists (T1a4c) and merged. A VM restart killed the
  setup's baseline (CI's run on `master` stands for it); the scoped verify without original data.
  The pull request: #1150.

### 2026-11-24 — E31 T1a3d: the answers and the table, in two

- A cloud clone, `Heroes3.exe` staged from the user's repo folder. T1a3d measured about 2 800
  instructions plus the table, so the cut asked (AI-Q28): two parts; the main session read the
  answers to the visits, one general-purpose sub-agent the Seer, the markets, the allies' shares and
  the battle's split in its own worktree (only `map.toml`, its survey text to a scratch file); its
  diff applied with `git apply --3way`, the main session wrote §3.4 and the table §3.6. The setup's
  baseline stopped (CI green on `master`); the scoped verify without original data. Two
  informational inbox notes moved to `QUEUE-ARCHIVE.md`.
