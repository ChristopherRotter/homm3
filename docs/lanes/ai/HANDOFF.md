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
  the target is beating the faithful system actor (`t7-1`: 0.372); it **trains no agent any
  more** (ADR-0026 point 8 as amended; no neutral battles in training for now, the user at T2a).
  **Done 2026-10-28:** T1a, T1b (a world's reward v1, the true world), T2a (shaping, the refused
  mixes, T2b's configs), all C ABI v35; T1c (the search gains nothing with either world: the
  network's evaluation limits it); T2b (`e25-control-1`, `e25-shaping-1`: shaping about +0.04
  against Aggressor, Greedy and the system actor, 0.409; a tail of long battles). **Next:** T3,
  the checkpoint: the user chooses increment 2 (`E25-log.md` T1c, T2b).
- **Open pull requests:** none (#534 and #539 merged); this session's branch
  `claude/e25-training-runs-cyvj0l` (T2b's records and read; `cross-lane`, the user: AI-STUB-1's
  magic limits fixed in `h3-battle`'s `world.rs`, which turned `master`'s verify red at random)
  until its pull request.
- **Versions:** battle action encoding v13, observation encoding v22, replay format v31
  (CLT-Q4/Q6), the adventure env's action encoding v5 (E08 T63a), C ABI v35 (E25 T1a, T1b, T2a:
  a world's reward, the true world, the potential) (`docs/08-work-process.md`, "Versions"). Next
  ids: AI-Q11, AI-QUIRK-1, AI-STUB-2.
- Working agreement: increments with a user checkpoint (Q41's pattern); the user reviews ADRs and seams.

## Standing traps

Moved to `TRAPS.md`, in three groups: **Ownership** (cross-lane encodings and replays, what `--scope ai` skips), **Environment** (what an agent sees, the batch, the pool's `unsafe`, benchmarks, the census, the encounter sampler, bots' tables) and **Python** (the trainer's resumable state, uv, PyTorch's pinned build, the sandbox's network).

## Recent sessions

### 2026-10-28 — E25 T2b

- The user ran both configs at 20 M samples and uploaded both records and metrics and the
  control's `best.pt` (shaping's stays on their machine). The records committed and read
  (`E25-log.md` T2b): shaping is the first lever past the runs' spread since E06 T5d; its long
  battles (0.2 % cut in training, ladders three to four times slower) are open. `master`'s CI
  red since #534's merge: a world's property test hit AI-STUB-1's magic limits; fixed here.

### 2026-10-28 — E25 T1a, T1b, T2a (C ABI v35), T1c

- T1a: a world scores a decided battle by its batch's reward v1 (found with it: a world's setup
  held the guess's starting mana). T1b: the true world, `--true-worlds`. The user: `t7-1`'s
  `best.pt` uploaded, no neutral battles for now, a second branch for the shared docs (#539).
  T2a: the batch's potential, `[ppo] shaping`, `run.trainable`, T2b's configs. T1c (six runs,
  about five hours in the sandbox): no gain with guessed or true worlds.

### 2026-10-27 — E25's refinement (AI-Q10)

- E06's lane closure (T9c, T9d), left unopened on `claude/kind-bohr-wfa9oq`, merged as #524
  meanwhile. The user: target the faithful system actor; increment 1 the search's
  diagnostics and shaping; run lengths per run; asked what the system actor reads that a player
  may not (the hidden part listed in AI-Q10): it trains no agent any more, a fair variant goes to
  the battle lane's inbox, imitating that one is a candidate. Done: the epic file, increment 1's
  tasks, #525 (the shared part).
